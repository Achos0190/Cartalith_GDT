extends Node
## Session mode probe (SM-7, `STORY_MENUS_PROPOSAL.md` §3.4 / §6 / §9 Q5,
## Ruling CK, 2026-10-18).
##
## What it protects, on the live shell `app.tscn` builds:
##
##   1. Window ▸ Session mode exists as a check row carrying Ctrl+Shift+M,
##      bound through `_bind_accelerator` (rebindable), used by no other row
##      on the bar, and listed by Help ▸ Keyboard shortcuts.
##   2. Turning it on mid-generation is REFUSED with a reason and changes
##      nothing (the draft check it would make is an engine read, and the
##      worker owns the engine); turning it on over an uncommitted sculpt
##      draft is refused too (§7 risk 6: the draft's Commit/Discard live in
##      the WORLD domain the mode hides).
##   3. The gate hides EXACTLY its chrome and nothing else. Measured as a
##      difference of two states with the same layout: the `Session` seed
##      applied by hand with the gate down, then the real toggle. The only
##      Controls allowed to disappear are the WORLD rail mark (the phone's
##      GENERATE tab) and the hairline above CIVIL; nothing may appear. The
##      menu rows whose disabled state or tooltip changes must be exactly the
##      literal list below, each disabled with the Session reason.
##   4. In the mode: every route into WORLD is refused (rail domain, node,
##      category, phone tab, New world), WORLD's mode is not rewritten by a
##      refused route, the menus' own refreshers cannot re-enable a gated row,
##      the rail expansion draws no WORLD rows; the map, the timeline bar
##      (desktop/tablet: CIVIL draws it) and the Story menu stay.
##   5. Turning it off restores the prior arrangement byte for byte
##      (`_capture_layout()` serialised, plus every domain's mode, plus every
##      menu row's disabled state and tooltip), and leaves the generation
##      parameters and the undo ledger untouched. An arrangement change made in
##      the mode persists as the `Session` layout and comes back next time.
##   6. It is not persisted: a fresh boot comes up with the mode off.
##
## The expected gated rows are literals (the tooltip's own list,
## `menus.gd::SESSION_MODE_TIP`), never read back from `menus.gd`.
##
## Run WINDOWED (screenshots; `--headless` delivers no texture), from the
## project directory, with a CLEAN user dir (the probe seeds `Session`):
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _sessionmode_probe.tscn -- --vp 1366x768 --out C:/abs/dir
##   ... -- --vp 1920x1080 --out DIR
##   ... -- --force-touch --vp 1600x2560 --out DIR   (tablet portrait)
##   ... -- --force-touch --vp 2560x1600 --out DIR   (tablet landscape)
##   ... -- --force-touch --vp 393x852 --out DIR     (phone)
##
## `--force-touch` is a user argument (after `--`), read by `dcc_shell.gd`'s
## boot; the run's `SESSION form=` line prints the form factor the shell
## actually picked. `--vp WxH` sizes the host
## SubViewport; `--out DIR` is where the PNGs go.
##
## Result: `SESSION ok  |FAIL  <check>` rows, `SESSION SHOT <path>`, and
## `SESSION RESULT <n> checks, <f> failed (<form>)`. Exit 0 / 1 (a check
## failed) / 2 (the shell could not be measured).

var app: Node
var _vp: SubViewport
var _checks := 0
var _fails := 0
var _out := ""
var _form := "desktop"

## The rows Session mode must disable, by menu path, per form factor. The
## tablet has no Edit menu of its own and holds the generation rows in World
## (DS-03 / Ruling CK Q2); its Window menu is under ☰.
const GATED_DESKTOP: Array = ["File > New world…", "File > New world from selection…",
	"Edit > Reset generation parameters", "Edit > Reset one stage",
	"Window > Workspace > World"]
const GATED_TABLET: Array = ["File > New world…", "File > New world from selection…",
	"World > Run pipeline", "World > Reset generation parameters", "World > Reset one stage",
	"World > Generation pipeline", "World > Sculpt", "World > Seed & size…",
	"☰ > Window > Workspace > World"]
## The reason every gated row must carry (a literal copy, so an edit to the
## shipped string is seen here).
const GATED_TIP := "Hidden by Session mode, which keeps generation out of reach at the table. Window ▸ Session mode, Ctrl+Shift+M, turns it off and brings this back."
const ACCEL := KEY_MASK_CTRL | KEY_MASK_SHIFT | KEY_M

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
	print("SESSION %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _clean(t: String) -> String:
	var s := t.strip_edges()
	if s.ends_with("▸"):
		s = s.substr(0, s.length() - 1).strip_edges()
	return s

func _shot(name: String) -> void:
	if _out == "":
		return
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	if img == null:
		_check("capture %s returned an image" % name, false)
		return
	var path := _out.path_join("session_%s_%s.png" % [_form, name])
	img.save_png(path)
	print("SESSION SHOT %s" % path)

func _close_popups(n: Node) -> void:
	if n is PopupMenu and (n as PopupMenu).visible:
		(n as PopupMenu).hide()
	for c in n.get_children(true):
		_close_popups(c)

func _menu_button(title: String) -> MenuButton:
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton and (ch as MenuButton).text == title:
			return ch
	return null

## Every row of every menu on the bar, deep: `path -> [disabled, tooltip]`.
func _menu_state() -> Dictionary:
	var out := {}
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			_walk((ch as MenuButton).get_popup(), (ch as MenuButton).text, out)
	return out

func _walk(p: PopupMenu, path: String, out: Dictionary) -> void:
	for i in p.item_count:
		if p.is_item_separator(i):
			continue
		var key := "%s > %s" % [path, _clean(p.get_item_text(i))]
		out[key] = [p.is_item_disabled(i), p.get_item_tooltip(i)]
		var sub := p.get_item_submenu(i)
		if sub != "":
			var sp := p.get_node_or_null(NodePath(sub)) as PopupMenu
			if sp != null:
				_walk(sp, key, out)

## The paths whose `[disabled, tooltip]` differ between two `_menu_state()`s.
func _menu_diff(a: Dictionary, b: Dictionary) -> Array:
	var out: Array = []
	for k in a:
		if not b.has(k) or str(a[k]) != str(b[k]):
			out.append(k)
	for k in b:
		if not a.has(k):
			out.append(k)
	out.sort()
	return out

## Every Control under the app that is visible in the tree, by path, as a
## count per NORMALISED path: Godot's generated names (`@Button@11710`) have
## their serial stripped, because a surface rebuilt on every domain change
## (the tool palette, `app.gd::_rebuild_tool_strip()`) gets new serials for
## the same structure, and a rebuild is not something the gate hid or showed.
func _visible() -> Dictionary:
	var out := {}
	var rx := RegEx.create_from_string(NAME_SERIAL)
	for c in app.find_children("*", "Control", true, false):
		if (c as Control).is_visible_in_tree():
			var k := rx.sub(str(app.get_path_to(c)), "@$1@", true)
			out[k] = int(out.get(k, 0)) + 1
	return out

## Normalised the same way as `_visible()`'s keys.
func _norm(path: String) -> String:
	return RegEx.create_from_string(NAME_SERIAL).sub(path, "@$1@", true)

## A generated node name's serial: `@Button@11710` -> group `Button`.
const NAME_SERIAL := "@([A-Za-z0-9_]+)@[0-9]+"

## Opens and closes the menus whose rows refresh on popup, so two
## `_menu_state()` reads compare refreshed rows with refreshed rows.
func _refresh_menus() -> void:
	for title in ["☰", "File", "Edit", "World", "Window"]:
		var mb := _menu_button(title)
		if mb != null:
			mb.get_popup().about_to_popup.emit()
			mb.get_popup().popup_hide.emit()
	await _frames(2)

## Whether `path` is `root` or under it (both app-relative strings).
func _under(path: String, roots: Array) -> bool:
	for r in roots:
		if path == r or path.begins_with(String(r) + "/"):
			return true
	return false

func _modes() -> Dictionary:
	var m := {}
	for d in DccShell.DOMAINS:
		m[String(d["id"])] = app.active_mode(String(d["id"]))
	return m

func _status() -> String:
	var l = app._status_labels.get("hint")
	return (l as Label).text if l != null else ""

## The real key: Ctrl+Shift+M pushed into the host viewport the shell lives
## in, so the MenuBar's own shortcut path answers it (not a function call).
func _press_accel() -> void:
	for pressed in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = KEY_M
		ev.physical_keycode = KEY_M
		ev.ctrl_pressed = true
		ev.shift_pressed = true
		ev.pressed = pressed
		_vp.push_input(ev)
		await _frames(1)
	await _frames(6)

## Phone: MORE ▸ Window ▸ Session mode, tapped through each row's own
## `gui_input` the way a finger reaches it.
func _row_named(root: Node, title: String) -> Control:
	if root is PanelContainer:
		var mine: Array = []
		_labels(root, mine)
		for t in mine:
			if _clean(String(t)) == title:
				return root as Control
	for c in root.get_children():
		var hit := _row_named(c, title)
		if hit != null:
			return hit
	return null

func _labels(n: Node, out: Array) -> void:
	if n is Label:
		out.append((n as Label).text)
	elif n is Button:
		out.append((n as Button).text)
	for c in n.get_children():
		_labels(c, out)

func _tap(row: Control) -> void:
	var at := row.size * 0.5
	for pressed in [true, false]:
		var ev := InputEventScreenTouch.new()
		ev.index = 0
		ev.pressed = pressed
		ev.position = at
		row.gui_input.emit(ev)

## Toggles the mode the way this form factor's user does: the accelerator on
## desktop and tablet, MORE ▸ Window ▸ Session mode on the phone. `shot`
## (phone only) captures the Window screen before the tap.
func _toggle(shot: String = "") -> void:
	if not DccTheme.is_phone():
		await _press_accel()
		return
	var pm = app.get("_phone_menu")
	pm.open()
	await _frames(4)
	var win_row := _row_named(pm._screen_body, "Window")
	if win_row == null:
		_check("phone MORE root draws a Window row", false)
		return
	_tap(win_row)
	await _frames(4)
	var row := _row_named(pm._screen_body, "Session mode")
	_check("phone MORE ▸ Window draws a Session mode row", row != null)
	if shot != "":
		await _shot(shot)
	if row != null:
		_tap(row)
		await _frames(4)
	pm.close()
	await _frames(6)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("SESSION WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("SESSION FATAL extension did not load")
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
		print("SESSION FATAL no menu bar")
		get_tree().quit(2)
		return
	var phone := DccTheme.is_phone()
	var tablet := DccTheme.is_tablet()
	_form = "phone" if phone else ("tablet" if tablet else "desktop")
	_form += "_%dx%d" % [vw, vh]
	print("SESSION form=%s touch=%s" % [_form, str(DccTheme.is_touch())])
	var menus = app.menus

	## -- 6. not persisted: a boot is always off -----------------------------
	_check("boots with Session mode off", not app.is_session_mode())

	## -- 1. the row and its key ---------------------------------------------
	var winp: PopupMenu = menus._window_popup
	var si := winp.get_item_index(DccMenus.ID_WIN_SESSION_MODE) if winp != null else -1
	_check("Window ▸ Session mode is a check row",
		si >= 0 and winp.get_item_text(si) == "Session mode" and winp.is_item_checkable(si))
	if si < 0:
		_finish()
		return
	_check("it sits between the region toggles and Diagnostics overlay (A2)",
		winp.get_item_index(DccMenus.ID_WIN_DIAG_OVERLAY) == si + 1
			and winp.get_item_index(DccMenus.ID_WIN_RAIL) < si)
	_check("its accelerator is Ctrl+Shift+M and it is rebindable",
		winp.get_item_accelerator(si) == ACCEL and menus.is_rebindable_shortcut(DccMenus.ID_WIN_SESSION_MODE),
		OS.get_keycode_string(winp.get_item_accelerator(si)))
	var tip := winp.get_item_tooltip(si)
	for word in ["New world", "Reset generation parameters", "Workspace ▸ World", "Generate", "Sculpt",
			"timeline bar", "Journey planner", "Story menu", "Not remembered across restarts", "undo history"]:
		_check("the row's tooltip names '%s'" % word, tip.find(word) >= 0)
	var users: Array = []
	for k in _accels():
		if int(k[1]) == ACCEL:
			users.append(k[0])
	_check("Ctrl+Shift+M is used by exactly one row on the whole bar", users.size() == 1, str(users))
	var listed: Array = app.shortcuts_dialog._collect_from_menus()
	_check("Help ▸ Keyboard shortcuts lists Session mode",
		listed.any(func(e): return String(e[2]) == "Session mode"), "")

	## -- 2a. refused mid-generation -----------------------------------------
	var bridge = app.bridge
	bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(1)
	if bool(bridge.generating):
		var ok: bool = menus.set_session_mode(true)
		_check("generating: turning Session mode on is refused", not ok and not app.is_session_mode())
		_check("generating: the refusal says a generation is running", _status().findn("generation") >= 0, _status())
	else:
		_check("the generation was still running one frame in (gate leg reachable)", false)
	while bool(bridge.generating):
		await get_tree().create_timer(0.25).timeout
	await _frames(6)
	_check("the probe world exists", bool(bridge.has_world))

	## -- 2b. refused over an open sculpt draft ------------------------------
	app.select_domain_mode("world", "b")
	await _frames(3)
	bridge.sculpt_set_feature("freehand")
	bridge.sculpt_begin_stroke()
	for p in [Vector2(60, 60), Vector2(70, 64), Vector2(80, 70)]:
		bridge.sculpt_add_point(p.x, p.y)
	bridge.sculpt_end_stroke()
	await _frames(2)
	if bridge.sculpt_stamp_count() > 0:
		var ok2: bool = menus.set_session_mode(true)
		_check("open sculpt draft: turning Session mode on is refused", not ok2 and not app.is_session_mode())
		_check("open sculpt draft: the refusal names the draft", _status().findn("draft") >= 0, _status())
		bridge.sculpt_discard()
		await _frames(3)
	else:
		_check("the probe could open a sculpt draft (draft leg reachable)", false)

	## -- the prior arrangement: something a restore can get wrong -----------
	## WORLD ▸ Sculpt, CARTO left on Labels, the right dock hidden (desktop and
	## tablet; the phone's docks are sheets a layout does not arrange), and the
	## rail expanded last (any domain switch collapses it).
	app.select_domain_mode("world", "b")
	app.apply_domain_mode("cartography", "labels")
	if not phone:
		winp.id_pressed.emit(DccMenus.ID_WIN_RIGHT)
		app.set_rail_expanded(true)
	await _frames(6)
	var before_dict: Dictionary = menus._capture_layout()
	var before_layout := JSON.stringify(before_dict)
	var before_modes := _modes()
	await _refresh_menus()
	var before_menus := _menu_state()
	var before_params := JSON.stringify(bridge.params_snapshot())
	var before_ledger := JSON.stringify(bridge.undo_ledger())
	var before_vis := _visible()
	print("SESSION prior layout %s modes %s" % [before_layout, str(before_modes)])
	await _shot("before")

	## -- 3a. the same layout, gate down (the control state) -----------------
	menus._apply_layout(menus._session_seed())
	await _frames(8)
	var vis_a := _visible()
	var menus_a := _menu_state()
	var world_btn: Control = app._domain_buttons.get("world")
	var world_path := _norm(str(app.get_path_to(world_btn))) if world_btn != null else ""
	var sep: Control = app._rail_seps.get("civilization")
	var sep_path := _norm(str(app.get_path_to(sep))) if sep != null else "-"
	_check("control state: the WORLD mark is drawn with the gate down (positive control)",
		vis_a.has(world_path), world_path)
	## Back to the prior arrangement, and prove the restore helper got there,
	## so the real toggle starts from exactly `before`.
	for id in before_modes:
		if id != String(before_dict.get("domain", "")):
			app.apply_domain_mode(String(id), String(before_modes[id]))
	menus._apply_layout(before_dict)
	await _frames(8)
	_check("control state undone: the arrangement is the prior one again",
		JSON.stringify(menus._capture_layout()) == before_layout and _modes() == before_modes,
		JSON.stringify(menus._capture_layout()))

	## -- 3b. the real toggle ------------------------------------------------
	await _toggle("more_window" if phone else "")
	_check("the toggle turned Session mode on", app.is_session_mode(), _status())
	winp.about_to_popup.emit()
	_check("Window ▸ Session mode reads checked", winp.is_item_checked(winp.get_item_index(DccMenus.ID_WIN_SESSION_MODE)))
	var vis_b := _visible()
	var gone: Array = []
	var appeared: Array = []
	for k in vis_a:
		if int(vis_b.get(k, 0)) < int(vis_a[k]):
			gone.append(k)
	for k in vis_b:
		if int(vis_a.get(k, 0)) < int(vis_b[k]):
			appeared.append(k)
	var stray := gone.filter(func(k): return not _under(String(k), [world_path, sep_path]))
	_check("the gate hides the WORLD mark", gone.has(world_path), world_path)
	if not phone:
		_check("the gate hides the hairline that would lead the rail", gone.has(sep_path), sep_path)
	_check("the gate hides nothing else (%d controls gone)" % gone.size(), stray.is_empty(), str(stray.slice(0, 12)))
	_check("the gate shows nothing new", appeared.is_empty(), str(appeared.slice(0, 12)))
	var changed := _menu_diff(menus_a, _menu_state())
	var want: Array = (GATED_TABLET if tablet else GATED_DESKTOP).duplicate()
	want.sort()
	_check("the menu rows the gate changed are exactly the listed ones", changed == want,
		"changed=%s" % str(changed))
	var now := _menu_state()
	var all_gated := true
	for k in want:
		if not now.has(k) or not bool(now[k][0]) or String(now[k][1]) != GATED_TIP:
			all_gated = false
			print("SESSION   gated row %s = %s" % [k, str(now.get(k))])
	_check("each gated row is disabled with the Session reason", all_gated)

	## -- 4. in the mode -------------------------------------------------------
	_check("the shell left WORLD for %s" % ("CARTO" if phone else "CIVIL ▸ Journey planner"),
		app.active_domain() == ("cartography" if phone else "civilization"), app.active_domain())
	var dom := String(app.active_domain())
	app.select_domain("world")
	await _frames(3)
	_check("refused: selecting WORLD (rail mark, Window ▸ Workspace, any jump)",
		app.active_domain() == dom and _status().begins_with("Session mode is on"), _status())
	app.select_domain_mode("world", "a")
	await _frames(3)
	_check("refused: a WORLD rail node, and WORLD's mode is not rewritten",
		app.active_domain() == dom and app.active_mode("world") == "b", app.active_mode("world"))
	app.select_domain_category("world", "Generate")
	await _frames(3)
	_check("refused: a jump to a WORLD category", app.active_domain() == dom)
	app.open_new_world()
	await _frames(3)
	_check("refused: New world from a non-menu route (open-project / phone picker tile)",
		not app.new_world_dialog.visible, _status())
	if phone:
		var was_tab := String(app._phone_tab)
		app._pick_phone_tab("gen")
		await _frames(3)
		_check("refused: the GENERATE tab, which is also not drawn", String(app._phone_tab) == was_tab
			and not (app._phone_tab_cells["gen"]["button"] as Control).visible, String(app._phone_tab))
	## A popup's own refresher must not undo the gate.
	for title in ["☰", "File", "Edit", "World", "Window"]:
		var mb := _menu_button(title)
		if mb != null:
			mb.get_popup().about_to_popup.emit()
			mb.get_popup().popup_hide.emit()
	menus._workspace_popup.about_to_popup.emit()
	menus._reset_stage_popup.about_to_popup.emit()
	var after_refresh := _menu_state()
	var reenabled: Array = []
	for k in want:
		if not bool(after_refresh[k][0]):
			reenabled.append(k)
	_check("no menu refresher re-enables a gated row", reenabled.is_empty(), str(reenabled))
	if not phone:
		app.set_rail_expanded(true)
		await _frames(3)
		var wn: Control = app._rail_node_rows.get("world/a")
		var cn: Control = app._rail_node_rows.get("civilization/planner")
		var heads: Array = app._rail_exp_heads.get("world", [])
		_check("rail expansion: no WORLD header or node is drawn, CIVIL's are",
			wn != null and not wn.is_visible_in_tree() and cn != null and cn.is_visible_in_tree()
				and heads.size() > 0 and not (heads[0] as Control).is_visible_in_tree())
		await _shot("on_rail_expanded")
		app.set_rail_expanded(false)
		await _frames(3)
		_check("kept: the timeline bar", app.timeline_bar.is_visible_in_tree())
		_check("kept: the Story menu", _menu_button("Story") != null and _menu_button("Story").is_visible_in_tree())
	_check("kept: the map", (app.viewport_content as Control).is_visible_in_tree())
	await _shot("on")
	## Story ▸ Journey planner still works in the mode (desktop/tablet: the
	## Story menu; phone: the PLAN tab).
	if not phone:
		_menu_button("Story").get_popup().id_pressed.emit(DccMenus.ID_STORY_JOURNEY_PLANNER)
	else:
		app._pick_phone_tab("plan")
	await _frames(6)
	_check("kept: the Journey planner opens", String(app.armed_tool) == "journey", String(app.armed_tool))
	await _shot("on_planner")
	app.arm_tool("inspect")
	await _frames(4)
	if not phone:
		## Window menu open on the row (desktop: the bar; tablet: ☰ ▸ Window).
		winp.about_to_popup.emit()
		if tablet:
			var over := _menu_button("☰")
			over.show_popup()
			await _frames(4)
			var op := over.get_popup()
			winp.popup(Rect2i(Vector2i(int(op.position.x + op.size.x), int(op.position.y)), Vector2i.ZERO))
		else:
			_menu_button("Window").show_popup()
		await _frames(5)
		await _shot("on_window_menu")
		_close_popups(app)
		await _frames(3)
	## An arrangement change made at the table: the status bar hidden.
	winp.id_pressed.emit(DccMenus.ID_WIN_STATUS)
	await _frames(4)

	## -- 5. off: the prior arrangement, byte for byte ------------------------
	await _toggle()
	_check("the toggle turned Session mode off", not app.is_session_mode(), _status())
	var after_layout := JSON.stringify(menus._capture_layout())
	_check("off restores the prior layout byte for byte", after_layout == before_layout,
		"before=%s after=%s" % [before_layout, after_layout])
	_check("off restores every domain's mode", _modes() == before_modes, str(_modes()))
	await _refresh_menus()
	var mdiff := _menu_diff(before_menus, _menu_state())
	_check("off restores every menu row's disabled state and tooltip", mdiff.is_empty(), str(mdiff))
	_check("the generation parameters are untouched", JSON.stringify(bridge.params_snapshot()) == before_params)
	_check("the undo ledger is untouched", JSON.stringify(bridge.undo_ledger()) == before_ledger)
	var vis_after := _visible()
	_check("off: the WORLD mark is drawn again", vis_after.has(world_path))
	await _shot("off")
	var sess: Dictionary = DccSettings.layout("Session")
	_check("the table-side change persisted as the Session layout (status bar hidden)",
		sess.has("regions") and not bool((sess["regions"] as Dictionary).get(DccMenus.ID_WIN_STATUS, true)),
		JSON.stringify(sess))
	_check("Session is listed in Window ▸ Layouts", DccSettings.layout_names().has("Session"))

	## Again: the stored Session comes back, and off restores again.
	await _toggle()
	_check("on again: the stored Session layout is applied (status bar hidden)",
		app.is_session_mode() and not app.is_region_shown(DccMenus.ID_WIN_STATUS))
	await _toggle()
	_check("off again: the prior layout byte for byte",
		not app.is_session_mode() and JSON.stringify(menus._capture_layout()) == before_layout)
	_finish()

## Every accelerator on the bar, deep: `[path, accel]`.
func _accels() -> Array:
	var out: Array = []
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			_accel_walk((ch as MenuButton).get_popup(), (ch as MenuButton).text, out)
	return out

func _accel_walk(p: PopupMenu, path: String, out: Array) -> void:
	for i in p.item_count:
		var a := p.get_item_accelerator(i)
		if a != 0:
			out.append([path + " > " + p.get_item_text(i), a])
		var sub := p.get_item_submenu(i)
		if sub != "":
			var sp := p.get_node_or_null(NodePath(sub)) as PopupMenu
			if sp != null:
				_accel_walk(sp, path + " > " + p.get_item_text(i), out)

func _finish() -> void:
	print("SESSION RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)
