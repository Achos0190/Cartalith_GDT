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
##   SM-2  an eighth menu, Story, after World (tablet ☰ · File · World · Story):
##         TIME / JOURNEYS / CONFLICTS & CAMPAIGNS / NOTES with the proposal's
##         §3.3 rows; the three unbuilt rows disabled with their milestone in
##         the reason; Ctrl+Shift+T and Ctrl+Shift+N bound through
##         `_bind_accelerator` (rebindable, listed by the shortcuts dialog, no
##         collision anywhere on the bar); Travel library moved here from World;
##         every enabled row pressed through to its destination over a small
##         generated world; the "this place" rows disabled with a reason until a
##         settlement is current, and the campaigns readout and place rows
##         answering "generating" mid-generation rather than reading the engine.
##         Phone: the MORE root's STORY section and its four band screens.
##   SM-3  File ▸ Export ▸ (FOR THE TABLE Player map… Ctrl+Shift+E / DM / full
##         map…, SHARE Export map image… plus the two disabled writers, DATA
##         the four Export routes); World ends on the "Export is in File ▸
##         Export" signpost; every route id 400+i still maps to route i; the
##         presets set the dialog's layer switches (Player: Journey routes off,
##         the rest on; DM: all on) and the Player row's tooltip says it hides
##         whole layers only.
##   SM-4  the CIVIL context card: Open in city viewer… and Open/Attach note
##         renamed; Add event to timeline here ▸ Battle · Siege · Front · Arrow
##         (Battle recorded at once, Front arms the Conflict tool), its tip
##         naming the gap; Plan journey from here / Journey to here driving a
##         real route into the planner; on the phone one Add event row and the
##         peek sheet's chips in A8's order. Screenshots of the card / sheet.
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
## (separators by caption). Travel library… closed RECORDS until SM-2 moved it.
const WORLD_ROWS: Array = ["⧉ Data manager", "|RECORDS", "Factions…", "Places…",
	"Markdown vault…", "Browse & edit a note…", "Places without notes…", "Timeline",
	"|TECHNICAL", "World data tables…", "Import", "Sources & validation",
	"Export is in File ▸ Export"]

## File ▸ Export ▸'s rows, the proposal's §3.1 table in A2's order. The DATA
## band is the four `ROUTES` Export rows as `_add_route_rows` labels them.
const FILE_EXPORT_ROWS: Array = ["|FOR THE TABLE", "Player map…", "DM / full map…",
	"|SHARE", "Export map image…", "Print sheet (PDF)…", "VTT battle map (grid + walls)…",
	"|DATA", "Maps   tiles", "GIS / GeoJSON   .geojson", "World Data   map + atlas", "Assets   .zip"]

## Story's rows, the proposal's §3.3 table in the mockup's (A2) order. The
## campaigns row is a live readout; its text is normalised to `Campaigns*`.
const STORY_ROWS: Array = ["|TIME", "Timeline", "Political history of this place…",
	"Faction history…", "|JOURNEYS", "Journey planner", "Saved journeys…", "Travel library…",
	"|CONFLICTS & CAMPAIGNS", "Conflicts…", "Draw a conflict", "Campaigns*",
	"|NOTES", "Notes for this place", "Places without notes…"]
## Row -> the milestone its disabled reason must name.
const STORY_TODO: Dictionary = {"Saved journeys…": "SM-6", "Conflicts…": "SM-6",
	"Places without notes…": "SM-5"}

## The tablet canvas's own World rows (`menus.gd::_world`), which must stay on
## top and unchanged (Q2: "nothing already in the tablet World menu moves").
const TABLET_WORLD_HEAD: Array = ["Run pipeline", "|", "Reset generation parameters",
	"Reset one stage", "|", "Generation pipeline", "Sculpt", "|", "Seed & size…", "|"]

const DESKTOP_TITLES: Array = ["File", "Edit", "Assets", "World", "Story", "Preferences", "Window", "Help"]
const TABLET_TITLES: Array = ["☰", "File", "World", "Story"]

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

	## Every Data-manager route exactly once, across World's two submenus and
	## File ▸ Export ▸ (SM-3), each row's id naming ITS route: id 400+i must
	## carry `ROUTES[i]`'s label (a build-order id would renumber them).
	var seen := {}
	var file_mb := _menu_button("File")
	var fexp: PopupMenu = _sub(file_mb.get_popup(), "Export") if file_mb != null else null
	_check("File ▸ Export is a submenu (SM-3)", fexp != null)
	var route_pops: Array = [fexp]
	for t in ["Import", "Sources & validation"]:
		var sp := _sub(world, t)
		_check("World ▸ %s is a submenu" % t, sp != null)
		route_pops.append(sp)
	var mislabelled: Array = []
	for sp in route_pops:
		if sp == null:
			continue
		for i in (sp as PopupMenu).item_count:
			var id: int = (sp as PopupMenu).get_item_id(i)
			if not (sp as PopupMenu).is_item_separator(i) and id >= DccMenus.ID_DATA_ROUTE_FIRST \
					and id < DccMenus.ID_DATA_ROUTE_FIRST + DataManagerWindow.ROUTES.size():
				seen[id] = int(seen.get(id, 0)) + 1
				var want_label := String(DataManagerWindow.ROUTES[id - DccMenus.ID_DATA_ROUTE_FIRST]["label"])
				if not (sp as PopupMenu).get_item_text(i).begins_with(want_label):
					mislabelled.append("%d=%s" % [id, (sp as PopupMenu).get_item_text(i)])
	var once := seen.size() == DataManagerWindow.ROUTES.size() and seen.values().all(func(v): return v == 1)
	_check("every DataManagerWindow route row appears exactly once (%d routes)" % DataManagerWindow.ROUTES.size(),
		once, str(seen))
	_check("route id 400+i carries ROUTES[i]'s label", mislabelled.is_empty(), str(mislabelled))
	_check("World has no Export ▸ submenu any more (SM-3)", _sub(world, "Export") == null)
	if fexp != null:
		await _sm3(fexp)

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

	await _sm2()

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
		var story_mb := _menu_button("Story")
		if story_mb != null:
			story_mb.show_popup()
			await _frames(4)
			await _shot("story")
			_close_popups(app)
			await _frames(3)
		## SM-3: File open with its Export ▸ submenu beside it.
		var file_mb2 := _menu_button("File")
		if file_mb2 != null:
			file_mb2.show_popup()
			await _frames(4)
			var fpop := file_mb2.get_popup()
			var fx := _sub(fpop, "Export")
			if fx != null:
				fx.popup(Rect2i(Vector2i(int(fpop.position.x + fpop.size.x), int(fpop.position.y)), Vector2i.ZERO))
				await _frames(4)
			await _shot("file_export")
			_close_popups(app)
			await _frames(3)
	else:
		var pm = app.get("_phone_menu")
		_check("the phone has a PhoneMenu", pm != null)
		if pm != null:
			pm.open()
			await _frames(4)
			await _shot("more")
			## SM-2: the root carries a STORY head and its four band rows.
			var root_txt: Array = []
			_labels(pm._screen_body, root_txt)
			for r in pm.STORY_ROWS:
				_check("phone MORE root draws the STORY row '%s'" % r["label"],
					root_txt.has(String(r["label"])), str(root_txt))
			pm._push_screen("data")
			await _frames(4)
			await _shot("more_data")
			## Every Story row lands on its band's screen.
			var smb := _menu_button("Story")
			var story_p: PopupMenu = smb.get_popup() if smb != null else null
			var band_txt: Array = []
			for r in pm.STORY_ROWS:
				pm.open()
				pm._push_screen("story", String(r["band"]))
				await _frames(4)
				_labels(pm._screen_body, band_txt)
				if String(r["band"]) == "JOURNEYS":
					await _shot("more_story_journeys")
			if story_p != null:
				for i in story_p.item_count:
					if story_p.is_item_separator(i):
						continue
					var t := story_p.get_item_text(i)
					_check("phone STORY screens draw '%s'" % t, band_txt.has(t), "")
			pm.close()
			await _frames(3)
	_finish()

## Every Label and Button text under `n`, depth first.
func _labels(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for c in n.get_children():
		_labels(c, out)

## A popup's rows with the campaigns readout normalised (its text is live).
func _story_rows(p: PopupMenu) -> Array:
	return _rows(p).map(func(s): return "Campaigns*" if String(s).begins_with("Campaigns") else s)

## Every accelerator under `p`, deep: `[menu path, accel]`.
func _all_accels(p: PopupMenu, path: String, out: Array) -> void:
	for i in p.item_count:
		var a := p.get_item_accelerator(i)
		if a != 0:
			out.append([path + " > " + p.get_item_text(i), a])
		var sub := p.get_item_submenu(i)
		if sub != "":
			var sp := p.get_node_or_null(NodePath(sub)) as PopupMenu
			if sp != null:
				_all_accels(sp, path + " > " + p.get_item_text(i), out)

## SM-3's checks on File ▸ Export ▸ (see the header). Runs before any world
## exists, so the dialog rows are first seen refused with "No world".
func _sm3(fexp: PopupMenu) -> void:
	fexp.about_to_popup.emit()
	_check("File ▸ Export's rows are the proposal's §3.1 list, in order",
		_rows(fexp) == FILE_EXPORT_ROWS, str(_rows(fexp)))
	var pl := _index_of(fexp, "Player map…")
	var dm := _index_of(fexp, "DM / full map…")
	_check("Player map… carries Ctrl+Shift+E, rebindable",
		pl >= 0 and fexp.get_item_accelerator(pl) == (KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_E)
			and app.menus.is_rebindable_shortcut(DccMenus.ID_EXPORT_PLAYER))
	var accs: Array = []
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			_all_accels((ch as MenuButton).get_popup(), (ch as MenuButton).text, accs)
	var users := accs.filter(func(e): return int(e[1]) == (KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_E))
	_check("Ctrl+Shift+E is used by exactly one row on the whole bar", users.size() == 1, str(users))
	var listed: Array = app.shortcuts_dialog._collect_from_menus()
	_check("Help ▸ Keyboard shortcuts lists Player map…",
		listed.any(func(e): return String(e[2]) == "Player map…"), "")
	for t in ["Print sheet (PDF)…", "VTT battle map (grid + walls)…"]:
		var i := _index_of(fexp, t)
		_check("%s is disabled with its reason" % t,
			i >= 0 and fexp.is_item_disabled(i) and fexp.get_item_tooltip(i).length() > 20,
			fexp.get_item_tooltip(i) if i >= 0 else "missing")
	_check("no world: Player / DM are refused with the No-world reason",
		fexp.is_item_disabled(pl) and fexp.is_item_disabled(dm)
			and fexp.get_item_tooltip(pl).begins_with("No world"), fexp.get_item_tooltip(pl))
	fexp.popup_hide.emit()
	_check("closing File ▸ Export re-enables the preset rows for the keyboard",
		not fexp.is_item_disabled(pl))
	## The Player row's tooltip is honest about what it hides (§7 risk 7).
	var tip := String(DccMenus.EXPORT_PLAYER_TIP)
	_check("Player map…'s tooltip says it hides whole layers, not DM-only items",
		tip.findn("Journey routes") >= 0 and tip.findn("DM-only") >= 0 and tip.findn("still exports") >= 0, tip)
	## A refused press opens nothing and says why.
	fexp.id_pressed.emit(DccMenus.ID_EXPORT_PLAYER)
	await _frames(3)
	_check("Player map… with no world opens no dialog",
		app.export_dialog == null or not app.export_dialog.visible)

## Presses a preset with a world loaded and reads the dialog's switches back.
func _sm3_presets() -> void:
	var fexp := _sub(_menu_button("File").get_popup(), "Export")
	fexp.about_to_popup.emit()
	var pl := _index_of(fexp, "Player map…")
	_check("with a world: Player map… is enabled", not fexp.is_item_disabled(pl), fexp.get_item_tooltip(pl))
	fexp.popup_hide.emit()
	for preset in [["player", DccMenus.ID_EXPORT_PLAYER, {"rivers": true, "ways": true, "labels": true,
			"icons": true, "territory": true, "routes": false}],
			["dm", DccMenus.ID_EXPORT_DM, {"rivers": true, "ways": true, "labels": true,
			"icons": true, "territory": true, "routes": true}]]:
		fexp.id_pressed.emit(int(preset[1]))
		await _frames(4)
		var d = app.export_dialog
		var got := {}
		if d != null:
			for k in d._layer_boxes:
				got[String(k)] = (d._layer_boxes[k] as CheckBox).button_pressed
		_check("%s preset opens the dialog with switches %s" % [preset[0], str(preset[2])],
			d != null and d.visible and got == preset[2] and d.title.findn(String(d.PRESETS[preset[0]]["title"])) >= 0,
			"visible=%s got=%s title=%s" % [d != null and d.visible, str(got), d.title if d != null else ""])
		if String(preset[0]) == "player":
			await _shot("export_player")
		if d != null:
			d.hide()
		await _frames(2)
	## A plain Export map image… clears the preset title and keeps the switches.
	fexp.id_pressed.emit(DccMenus.ID_EXPORT_MAP_IMAGE)
	await _frames(4)
	var d2 = app.export_dialog
	_check("Export map image… opens the dialog without a preset title",
		d2 != null and d2.visible and d2.title == "Export map image", d2.title if d2 != null else "")
	if d2 != null:
		d2.hide()
	await _frames(2)

## The row of `rows` with id `id`, or `{}`.
func _row_id(rows: Array, id: String) -> Dictionary:
	for r in rows:
		if String((r as Dictionary).get("id", "")) == id:
			return r
	return {}

## The context request a right-click (or a phone long-press) on settlement
## `i` makes, built through the broker so `form` and `armed_tool` are live.
func _settlement_req(i: int) -> Dictionary:
	var s: Dictionary = app.bridge.settlements()[i]
	var raw := {"gx": float(s.get("x", 0)) + 0.5, "gy": float(s.get("y", 0)) + 0.5,
		"screen_pos": Vector2(480, 320), "hits": [{"kind": "settlement", "id": i}], "source": "mouse"}
	return app.context_broker.build_request(raw)

## SM-4's checks (see the header) over the probe's generated world.
func _sm4() -> void:
	var bridge = app.bridge
	var setts: Array = bridge.settlements()
	if setts.size() < 2:
		_check("SM-4 needs two settlements", false, str(setts.size()))
		return
	var civ: Node = null
	for ws in app._workspaces:
		if ws.has_method("_story_ctx_rows"):
			civ = ws
	_check("the CIVIL workspace provides the story rows", civ != null)
	if civ == null:
		return
	app.select_domain("civilization")
	app.arm_tool("inspect")
	await _frames(3)
	var phone := DccTheme.is_phone()
	var req := _settlement_req(0)
	var rows: Array = app.context_broker.collect(req)
	var nm := String(setts[0].get("name", ""))
	_check("card: Open in city viewer… replaces Open city layout",
		String(_row_id(rows, "civ.open_city_layout").get("label", "")) == "Open in city viewer…",
		String(_row_id(rows, "civ.open_city_layout").get("label", "")))
	var vn := String(_row_id(rows, "civ.vault_note").get("label", ""))
	_check("card: the note row reads Open note for / Attach note to (no 'vault')",
		vn == "Open note for %s" % nm or vn == "Attach note to %s…" % nm, vn)
	var ev := _row_id(rows, "civ.add_event")
	_check("card: Add event is in Place here", String(ev.get("section", "")) == "place", str(ev.get("section")))
	if not phone:
		var kids: Array = ev.get("children", [])
		_check("card: Add event to timeline here ▸ Battle · Siege · Front · Arrow",
			kids.map(func(k): return String(k["label"])) == ["Battle", "Siege", "Front", "Arrow"],
			str(kids.map(func(k): return String(k["label"]))))
		_check("card: Add event's tooltip names the gap (Q7)",
			String(ev.get("tip", "")).findn("free timeline events") >= 0, String(ev.get("tip", "")))
		## Battle: recorded at once, at this cell, in the cursor's year.
		var before: int = bridge.conflict_list().size()
		if kids.size() == 4:
			(kids[0]["callable"] as Callable).call()
			await _frames(3)
			var after: Array = bridge.conflict_list()
			_check("Add event ▸ Battle records one battle at the cursor's year",
				after.size() == before + 1 and String(after[after.size() - 1].get("kind", "")) == "battle"
					and int(after[after.size() - 1].get("start_year", -999)) == int(bridge.get_civ_year()),
				"%d -> %d %s" % [before, after.size(), str(after[after.size() - 1]) if not after.is_empty() else ""])
			(kids[2]["callable"] as Callable).call()
			await _frames(3)
			_check("Add event ▸ Front arms the Conflict tool, kind front, first point placed",
				String(app.armed_tool) == "conflict" and String(civ.get("_conflict_kind")) == "front"
					and (civ.get("_conflict_points") as PackedVector2Array).size() == 1,
				"%s %s %d" % [app.armed_tool, civ.get("_conflict_kind"), (civ.get("_conflict_points") as PackedVector2Array).size()])
	else:
		_check("phone: Add event is one row, no submenu, chip 2",
			not ev.has("children") and int(ev.get("chip", 0)) == 2 and String(ev.get("chip_label", "")) == "Add event", str(ev.keys()))
		(ev["callable"] as Callable).call()
		await _frames(3)
		_check("phone: Add event arms the Conflict tool with the first point here",
			String(app.armed_tool) == "conflict" and (civ.get("_conflict_points") as PackedVector2Array).size() == 1)
	app.arm_tool("inspect")
	await _frames(2)

	var jf := _row_id(rows, "civ.journey_from")
	var jt := _row_id(rows, "civ.journey_to")
	_check("card: Plan journey from here is an enabled Go row",
		bool(jf.get("enabled", false)) and String(jf.get("section", "")) == "go")
	_check("card: Journey to here is disabled with its reason while no start is pending",
		not bool(jt.get("enabled", true)) and String(jt.get("reason", "")).begins_with("Pick a start first"),
		String(jt.get("reason", "")))

	## Screenshot the card (desktop/tablet) or the phone peek sheet, for a
	## settlement, before the journey legs change the armed tool.
	if not phone:
		app.context_broker.present(req, rows)
		await _frames(6)
		await _shot("card")
		if app.context_broker.card != null:
			app.context_broker.card.hide()
		await _frames(2)
	else:
		var pm = app.get("_phone_menu")
		pm.close()
		await _frames(2)
		pm.peek_card(req, rows, Callable())
		await _frames(6)
		var chips: PackedStringArray = pm.peek_chip_labels()
		var note_chip := "Open note" if vn.begins_with("Open note") else "Attach note"
		_check("phone peek chips are the four story verbs in A8's order",
			Array(chips) == [note_chip, "Add event", "Journey from here", "City viewer"], str(chips))
		await _shot("peek")
		pm._dismiss_peek_card()
		await _frames(3)

	## Journey: from settlement 0 to its nearest other settlement.
	var s0: Dictionary = setts[0]
	var best := 1
	var best_d := INF
	for i in range(1, setts.size()):
		var d := Vector2(float(setts[i]["x"]), float(setts[i]["y"])).distance_to(Vector2(float(s0["x"]), float(s0["y"])))
		if d < best_d:
			best_d = d
			best = i
	(jf["callable"] as Callable).call()
	await _frames(3)
	_check("Plan journey from here arms Route with this cell as its first stop",
		String(app.armed_tool) == "route" and (civ._infra._route_points as PackedVector2Array).size() == 1,
		"%s %d" % [app.armed_tool, (civ._infra._route_points as PackedVector2Array).size()])
	var rows2: Array = app.context_broker.collect(_settlement_req(best))
	var jt2 := _row_id(rows2, "civ.journey_to")
	_check("with a start pending, Journey to here is enabled", bool(jt2.get("enabled", false)))
	var rc: int = bridge.route_count()
	if bool(jt2.get("enabled", false)):
		(jt2["callable"] as Callable).call()
		await _frames(4)
		var civ_panel: Control = app._workspace_panels.get("civilization")
		_check("Journey to here commits the route and opens the planner on it",
			bridge.route_count() == rc + 1 and civ_panel != null and not civ_panel.visible,
			"routes %d -> %d" % [rc, bridge.route_count()])
	app.select_domain("world")
	await _frames(3)

## SM-2's checks (see the header). Runs on every form factor; generates a
## small world (256x192, as `_vfy_batch0905_probe.gd` does) for the
## press-through legs.
func _sm2() -> void:
	var story_mb := _menu_button("Story")
	_check("a Story menu exists", story_mb != null)
	if story_mb == null:
		return
	var story := story_mb.get_popup()
	story.about_to_popup.emit()
	_check("Story's rows are the proposal's §3.3 list, in order",
		_story_rows(story) == STORY_ROWS, str(_story_rows(story)))
	for t in STORY_TODO:
		var i := _index_of(story, String(t))
		_check("Story ▸ %s is disabled with its %s reason" % [t, STORY_TODO[t]],
			i >= 0 and story.is_item_disabled(i) and story.get_item_tooltip(i).findn(String(STORY_TODO[t])) >= 0,
			story.get_item_tooltip(i) if i >= 0 else "missing")
	var world := _menu_button("World").get_popup()
	_check("Travel library left World (SM-2 moved it to Story)",
		_index_of(world, "Travel library…") < 0)

	## Accelerators: the two new ones are Ctrl+Shift+T / Ctrl+Shift+N, through
	## `_bind_accelerator` (so rebindable); Travel library keeps Shift+L; Journey
	## planner draws none (Shift+J is `app.gd`'s, listed in UNLISTED).
	var want_acc := {"Timeline": KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_T,
		"Notes for this place": KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_N,
		"Travel library…": KEY_MASK_SHIFT | KEY_L, "Journey planner": 0}
	for t in want_acc:
		var i := _index_of(story, String(t))
		_check("Story ▸ %s accelerator is %s" % [t, OS.get_keycode_string(int(want_acc[t]))],
			i >= 0 and story.get_item_accelerator(i) == int(want_acc[t]),
			OS.get_keycode_string(story.get_item_accelerator(i)) if i >= 0 else "missing")
	for id in [DccMenus.ID_STORY_TIMELINE, DccMenus.ID_STORY_NOTES_HERE]:
		_check("id %d is rebindable (went through _bind_accelerator)" % id,
			app.menus.is_rebindable_shortcut(id))
	var accs: Array = []
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			_all_accels((ch as MenuButton).get_popup(), (ch as MenuButton).text, accs)
	for k in [KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_T, KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_N]:
		var users := accs.filter(func(e): return int(e[1]) == k)
		_check("%s is used by exactly one row on the whole bar" % OS.get_keycode_string(k),
			users.size() == 1, str(users))
	## The shortcuts dialog reads the live menus: both new keys must be listed.
	var listed: Array = app.shortcuts_dialog._collect_from_menus()
	for t in ["Timeline", "Notes for this place"]:
		_check("Help ▸ Keyboard shortcuts lists Story ▸ %s" % t,
			listed.any(func(e): return String(e[0]) == "Story" and String(e[2]) == t), str(listed.size()))

	## No world yet: the place rows say so, and the readout prints no count.
	story.about_to_popup.emit()
	var pi := _index_of(story, "Political history of this place…")
	var ni := _index_of(story, "Notes for this place")
	_check("no world: the place rows are disabled with a reason",
		story.is_item_disabled(pi) and story.is_item_disabled(ni)
			and story.get_item_tooltip(ni) != "", story.get_item_tooltip(ni))
	var ci := -1
	for i in story.item_count:
		if story.get_item_text(i).begins_with("Campaigns"):
			ci = i
	_check("no world: the campaigns readout prints no count",
		ci >= 0 and story.get_item_text(ci).findn("no world") >= 0,
		story.get_item_text(ci) if ci >= 0 else "")
	## Closing the menu re-enables the accelerator rows (`menus.gd::_story`).
	story.popup_hide.emit()
	_check("closing Story re-enables the accelerator rows for the keyboard",
		not story.is_item_disabled(ni) and not story.is_item_disabled(pi))

	var bridge = app.bridge
	bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	## Mid-generation gate: nothing may read the engine.
	await _frames(1)
	if bool(bridge.generating):
		story.about_to_popup.emit()
		_check("generating: the campaigns readout says so",
			story.get_item_text(ci).findn("generating") >= 0, story.get_item_text(ci))
		_check("generating: the place rows say a generation is running",
			story.is_item_disabled(ni) and story.get_item_tooltip(ni).findn("generation") >= 0,
			story.get_item_tooltip(ni))
		story.popup_hide.emit()
	else:
		_check("the generation was still running one frame in (gate leg reachable)", false)
	while bool(bridge.generating):
		await get_tree().create_timer(0.25).timeout
	await _frames(4)
	var setts: Array = bridge.settlements()
	_check("the probe world has settlements", setts.size() > 0, str(setts.size()))
	await _sm3_presets()
	await _sm4()

	story.about_to_popup.emit()
	_check("world, nothing selected: Notes for this place is disabled with the no-place reason",
		story.is_item_disabled(ni) and story.get_item_tooltip(ni).findn("Nothing selected") >= 0,
		story.get_item_tooltip(ni))
	_check("world: the campaigns readout counts this year",
		story.get_item_text(ci).begins_with("Campaigns in year"), story.get_item_text(ci))
	story.popup_hide.emit()

	## Select the first settlement the way a map pick does.
	if setts.size() > 0:
		app.right_dock_ctrl.on_settlement_selected(setts[0], 0)
		await _frames(2)
		story.about_to_popup.emit()
		_check("a settlement is current: both place rows are enabled",
			not story.is_item_disabled(pi) and not story.is_item_disabled(ni))
		story.popup_hide.emit()
		story.id_pressed.emit(DccMenus.ID_STORY_POLITICAL)
		await _frames(4)
		_check("Political history… opens the Place editor on its Political tab",
			app.place_editor_window.visible and String(app.place_editor_window.get("_active_tab")) == "political",
			"visible=%s tab=%s" % [app.place_editor_window.visible, app.place_editor_window.get("_active_tab")])
		app.place_editor_window.hide()
		story.id_pressed.emit(DccMenus.ID_STORY_NOTES_HERE)
		await _frames(4)
		_check("Notes for this place opens the vault window", app.vault_window.visible)
		app.vault_window.hide()
		app.right_dock_ctrl.on_settlement_selected(null, -1)
		await _frames(2)

	story.id_pressed.emit(DccMenus.ID_STORY_FACTION_HISTORY)
	await _frames(4)
	_check("Faction history… opens the Factions window", app.faction_roster_window.visible)
	app.faction_roster_window.hide()
	app.select_domain("world")
	await _frames(3)
	story.id_pressed.emit(DccMenus.ID_STORY_TIMELINE)
	await _frames(4)
	_check("Story ▸ Timeline switches to CIVIL", app.active_domain() == "civilization", app.active_domain())
	app.select_domain("world")
	await _frames(3)
	story.id_pressed.emit(DccMenus.ID_STORY_DRAW_CONFLICT)
	await _frames(4)
	_check("Draw a conflict arms the CIVIL Conflict tool",
		app.active_domain() == "civilization" and String(app.armed_tool) == "conflict",
		"%s / %s" % [app.active_domain(), app.armed_tool])
	app.arm_tool("inspect")
	await _frames(2)
	story.id_pressed.emit(DccMenus.ID_TRAVEL_LIBRARY)
	await _frames(4)
	_check("Travel library… opens its window", app.travel_library_window.visible)
	app.travel_library_window.hide()
	story.id_pressed.emit(DccMenus.ID_STORY_JOURNEY_PLANNER)
	await _frames(4)
	var civ_panel: Control = app._workspace_panels.get("civilization")
	_check("Journey planner opens the planner (CIVIL, the planner swaps the dock panel out)",
		app.active_domain() == "civilization" and civ_panel != null and not civ_panel.visible,
		app.active_domain())
	app.select_domain("world")
	await _frames(3)


func _finish() -> void:
	print("STORY RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)
