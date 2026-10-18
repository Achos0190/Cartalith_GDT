extends Node
## Story menus probe (`STORY_MENUS_PROPOSAL.md`, Ruling CK, 2026-10-18).
##
## Asserts the menu structure the approved milestones build, on the live shell
## `menus.gd` builds, and saves a screenshot of each new or changed menu so the
## result can be compared with `design/story-menus-2026-10-17/` by eye:
##
##   SM-1  the Data menu is titled World, with RECORDS / TECHNICAL bands, the
##         Import ▸ / Export ▸ / Sources & validation ▸ submenus holding every
##         `DataManagerWindow.ROUTES` row exactly once, Places… and Timeline
##         pressing through to CIVIL, and Places without notes… disabled with a
##         reason. Tablet: one World menu, pipeline rows first, Data's rows below.
##
## The expected row lists are written here as literals (the proposal's §3
## tables), never read back from `menus.gd`, so a reordering or a dropped row
## there fails here.
##
## Run WINDOWED (menus are drawn and captured; `--headless` delivers no
## texture), from `godot-project/`:
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _storymenus_probe.tscn -- --vp 1366x768 --out C:/abs/dir
##   ... _storymenus_probe.tscn -- --vp 1920x1080 --out C:/abs/dir
##   ... _storymenus_probe.tscn -- --force-touch --vp 2560x1600 --out C:/abs/dir   (tablet)
##   ... _storymenus_probe.tscn -- --force-touch --vp 1080x2340 --out C:/abs/dir   (phone)
##
## `--vp WxH` sizes the host SubViewport (default 1366x768); `--force-touch`
## is `dcc_shell.gd`'s own override and picks tablet or phone from the size;
## `--out DIR` is where the PNGs go (omitted: no screenshots, checks still run).
##
## Result: an assertion. Prints `STORY ok  |FAIL  <check>` rows, `STORY SHOT
## <path>` per saved capture, and `STORY RESULT <n> checks, <f> failed (<form>)`.
## Exit 0 when nothing failed, 1 when a check failed, 2 when the shell could not
## be measured (no extension, no menu bar).

var app: Node
var _vp: SubViewport
var _checks := 0
var _fails := 0
var _out := ""
var _form := "desktop"

## World's rows below the head row, as the proposal's §3.2 table lists them
## (separators by caption). Travel library… closes RECORDS until SM-2 moves it.
const WORLD_ROWS: Array = ["⧉ Data manager", "|RECORDS", "Factions…", "Places…",
	"Markdown vault…", "Browse & edit a note…", "Places without notes…", "Timeline",
	"Travel library…", "|TECHNICAL", "World data tables…", "Import", "Export",
	"Sources & validation"]

## The tablet canvas's own World rows (`menus.gd::_world`), which must stay on
## top and unchanged (Q2: "nothing already in the tablet World menu moves").
const TABLET_WORLD_HEAD: Array = ["Run pipeline", "|", "Reset generation parameters",
	"Reset one stage", "|", "Generation pipeline", "Sculpt", "|", "Seed & size…", "|"]

const DESKTOP_TITLES: Array = ["File", "Edit", "Assets", "World", "Preferences", "Window", "Help"]
const TABLET_TITLES: Array = ["☰", "File", "World"]

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("STORY %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _menu_button(title: String) -> MenuButton:
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == title:
			return ch
	return null

## A popup's rows as strings: an item's text, a separator as `|` plus its caption.
func _rows(p: PopupMenu) -> Array:
	var out: Array = []
	for i in p.item_count:
		if p.is_item_separator(i):
			out.append("|" + p.get_item_text(i))
		else:
			out.append(p.get_item_text(i))
	return out

func _index_of(p: PopupMenu, text: String) -> int:
	for i in p.item_count:
		if not p.is_item_separator(i) and p.get_item_text(i) == text:
			return i
	return -1

func _sub(p: PopupMenu, title: String) -> PopupMenu:
	var i := _index_of(p, title)
	if i < 0 or p.get_item_submenu(i) == "":
		return null
	return p.get_node_or_null(NodePath(p.get_item_submenu(i))) as PopupMenu

func _shot(name: String) -> void:
	if _out == "":
		return
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	if img == null:
		_check("capture %s returned an image" % name, false)
		return
	var path := _out.path_join("story_%s_%s.png" % [_form, name])
	img.save_png(path)
	print("STORY SHOT %s" % path)

func _close_popups(n: Node) -> void:
	if n is PopupMenu and (n as PopupMenu).visible:
		(n as PopupMenu).hide()
	for c in n.get_children(true):
		_close_popups(c)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("STORY WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("STORY FATAL extension did not load")
		get_tree().quit(2)
		return
	_out = _arg("--out", "")
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	var parts: PackedStringArray = _arg("--vp", "1366x768").split("x")
	var vw := int(parts[0])
	var vh := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)
	if app.get("menu_bar_row") == null:
		print("STORY FATAL no menu bar")
		get_tree().quit(2)
		return
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	_form += "_%dx%d" % [vw, vh]
	print("STORY form=%s touch=%s" % [_form, str(DccTheme.is_touch())])

	var titles: Array = []
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			titles.append((ch as MenuButton).text)
	var tablet := DccTheme.is_tablet()
	var want_titles: Array = TABLET_TITLES if tablet else DESKTOP_TITLES
	_check("menu bar titles are %s" % str(want_titles), titles == want_titles, str(titles))
	_check("no menu is titled Data any more", not titles.has("Data"), str(titles))

	## The bar must fit: every MenuButton's drawn rect inside the viewport. On
	## a pointer the bar shares its row with the readout cluster, so each
	## button is also checked against its row's visible rect (`MISTAKES.md`:
	## assert the drawn rect against the container's VISIBLE rect).
	if not DccTheme.is_phone():
		var row_rect: Rect2 = (app.menu_bar_row as Control).get_global_rect()
		for ch in app.menu_bar_row.get_children():
			if ch is MenuButton:
				var r: Rect2 = (ch as Control).get_global_rect()
				_check("menu '%s' is drawn inside the %dpx-wide window" % [(ch as MenuButton).text, vw],
					r.position.x >= 0.0 and r.end.x <= float(vw) and row_rect.encloses(r),
					"rect=%s row=%s" % [str(r), str(row_rect)])

	var world_mb := _menu_button("World")
	_check("a World menu exists", world_mb != null)
	if world_mb == null:
		_finish()
		return
	var world := world_mb.get_popup()
	world.about_to_popup.emit()
	var rows := _rows(world)
	if tablet:
		var head := rows.slice(0, TABLET_WORLD_HEAD.size())
		var head_norm := head.map(func(s): return "|" if String(s).begins_with("|") else s)
		_check("tablet World keeps its pipeline rows on top, unchanged",
			head_norm == TABLET_WORLD_HEAD, str(head))
		rows = rows.slice(TABLET_WORLD_HEAD.size())
	_check("World's rows are the proposal's §3.2 list, in order", rows == WORLD_ROWS, str(rows))

	## Every Data-manager route exactly once, across the three submenus, each
	## with the id `_on_data` resolves.
	var seen := {}
	for t in ["Import", "Export", "Sources & validation"]:
		var sp := _sub(world, t)
		_check("World ▸ %s is a submenu" % t, sp != null)
		if sp == null:
			continue
		for i in sp.item_count:
			var id := sp.get_item_id(i)
			if not sp.is_item_separator(i) and id >= DccMenus.ID_DATA_ROUTE_FIRST \
					and id < DccMenus.ID_DATA_ROUTE_FIRST + DataManagerWindow.ROUTES.size():
				seen[id] = int(seen.get(id, 0)) + 1
	var once := seen.size() == DataManagerWindow.ROUTES.size() and seen.values().all(func(v): return v == 1)
	_check("every DataManagerWindow route row appears exactly once (%d routes)" % DataManagerWindow.ROUTES.size(),
		once, str(seen))
	var exp := _sub(world, "Export")
	_check("World ▸ Export carries Export map image…",
		exp != null and _index_of(exp, "Export map image…") >= 0)

	var pwn := _index_of(world, "Places without notes…")
	_check("Places without notes… is disabled with its reason (SM-5 not built)",
		pwn >= 0 and world.is_item_disabled(pwn) and world.get_item_tooltip(pwn).findn("SM-5") >= 0,
		world.get_item_tooltip(pwn) if pwn >= 0 else "missing")

	## Places… and Timeline press through to CIVIL. Start from WORLD so the
	## press has to switch the domain.
	app.select_domain("world")
	await _frames(3)
	world.id_pressed.emit(DccMenus.ID_PLACES)
	await _frames(4)
	_check("Places… switches to CIVIL", app.active_domain() == "civilization", app.active_domain())
	app.select_domain("world")
	await _frames(3)
	world.id_pressed.emit(DccMenus.ID_WORLD_TIMELINE)
	await _frames(4)
	_check("Timeline switches to CIVIL", app.active_domain() == "civilization", app.active_domain())
	if not DccTheme.is_phone():
		_check("Timeline leaves the timeline bar shown", app.is_region_shown(DccMenus.ID_WIN_TIMELINE))
	app.select_domain("world")
	await _frames(3)

	## Screenshots: the World menu open, then its Import ▸ submenu beside it.
	if not DccTheme.is_phone():
		world_mb.show_popup()
		await _frames(4)
		await _shot("world")
		var imp := _sub(world, "Import")
		if imp != null:
			imp.popup(Rect2i(Vector2i(int(world.position.x + world.size.x), int(world.position.y)), Vector2i.ZERO))
			await _frames(4)
			await _shot("world_import")
		_close_popups(app)
		await _frames(3)
	else:
		var pm = app.get("_phone_menu")
		_check("the phone has a PhoneMenu", pm != null)
		if pm != null:
			pm.open()
			await _frames(4)
			await _shot("more")
			pm._push_screen("data")
			await _frames(4)
			await _shot("more_data")
			pm.close()
			await _frames(3)
	_finish()

func _finish() -> void:
	print("STORY RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)
