extends Node
## VE-8 of `VAULT_EDITOR_PROPOSAL.md` s.7: the phone's editing form. Bar: *the row stays above the keyboard on the
## 6T at both orientations.* Also the milestone's other items: `Open ›`, Done, long-press variants, and the Table,
## Rule, Undo and Redo buttons (deferred from VE-3).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve8_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch [--shots DIR]
##   ... -- --vp 2340x1080 --density 2.8125 --force-touch      (the 6T, landscape)
##   ... -- --vp 1600x1000                                     (desktop: nothing of the phone's form may appear)
##
## **Windowed**, under a wiped user directory (`APPDATA=<empty dir>`), on a throw-away vault in TEMP. Exit 0 = every
## check held, 1 = a check failed, 2 = could not run. Real mouse or emulated-touch presses and real key events drive
## everything (a toolbar button, the text, Done, the chip, a press-and-hold).
##
## **What is simulated, and what is not.** Windows has no soft keyboard, so `DisplayServer.virtual_keyboard_get_height()`
## is unavailable. The probe stands in for it at the two places the product reads it: the editor's `keyboard_height`
## hook (replaced after the editor is built) and the shell's cached `_phone_kb_height` (set, then
## `_apply_phone_orientation()` called: the very call the shell's own poll makes when the height changes, which is what
## re-presents the window above the keyboard). The pixels of a real keyboard are not drawn; the keyboard's top edge is
## computed as the frame's height less the simulated height. A real keyboard is `adb`'s leg (STATUS.md).

const Ed := preload("res://shell/markdown_editor.gd")

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var ed
var te: TextEdit
var _root := ""
var _shots := ""
var _touch := false
var _phone := false
var _n := 0
var _fails: Array = []
var _kb_sim := -1
## What the window's OWN `keyboard_height` hook answered before the probe replaced it (-99 = not read yet).
var _real_kb := -99


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VE8  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE8  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _wait(ms: int) -> void:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < ms:
		await get_tree().process_frame


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _disk(rel: String) -> String:
	return FileAccess.get_file_as_string(_root + "/" + rel)


func _mouse(win: Window, local: Vector2, pressed: bool) -> void:
	var p := Vector2(win.position) + local * win.content_scale_factor
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = pressed
	ev.position = p
	ev.global_position = p
	if _touch:
		ev.device = InputEvent.DEVICE_ID_EMULATION
	_vp.push_input(ev)
	await _frames(2)


## A pointer move over `local`, so `gui_get_hovered_control()` answers for what is really under it.
func _hover(win: Window, local: Vector2) -> void:
	var p := Vector2(win.position) + local * win.content_scale_factor
	var ev := InputEventMouseMotion.new()
	ev.position = p
	ev.global_position = p
	_vp.push_input(ev)
	await _frames(3)


## The control under `local` (window control units) after a real pointer move there.
func _hover_at(local: Vector2) -> Control:
	await _hover(vw, local)
	return vw.gui_get_hovered_control()


func _tap(win: Window, local: Vector2) -> void:
	await _mouse(win, local, true)
	await _mouse(win, local, false)


## `_hold` on a button, after scrolling the dock's row so the button is on screen (a hold at an off-screen
## position would press whatever is there instead).
func _hold_btn(b: Control, sec: float) -> void:
	var sc := b.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(b)
		await _frames(2)
	await _hold(vw, b.get_global_rect().get_center(), sec)


## A press that is held for `sec` before it lifts: the long-press the variants cards answer.
func _hold(win: Window, local: Vector2, sec: float) -> void:
	await _mouse(win, local, true)
	await _wait(int(sec * 1000.0))
	await _mouse(win, local, false)
	await _frames(3)


## A REAL tap on `b`, scrolling it into view first if a scroll container hides it.
func _press(b: Control) -> void:
	if b == null:
		_ok("a control to press exists", false)
		return
	var sc := b.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(b)
		await _frames(2)
	await _tap(vw, b.get_global_rect().get_center())
	await _frames(3)


func _type(s: String) -> void:
	for i in s.length():
		var code := s.unicode_at(i)
		for pressed in [true, false]:
			var k := InputEventKey.new()
			k.pressed = pressed
			k.unicode = code
			k.keycode = code as Key
			_vp.push_input(k)
		await _frames(1)
	await _frames(3)


func _finish() -> void:
	print("VE8 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _grab() -> void:
	ed = vw._editor
	te = ed.text_edit if ed != null else null


func _open(rel: String) -> void:
	vw._link_back.clear()
	vw._enter_note(rel)
	await _frames(8)
	_grab()
	if _real_kb == -99 and ed != null:
		_real_kb = int(ed.keyboard_height.call()) if ed.keyboard_height.is_valid() else -98
	_set_sim_hook()


func _mode_str() -> String:
	return "phone" if _phone else ("touch" if _touch else "desk")


func _shot(name: String) -> void:
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	await _frames(4)
	var vs := "%dx%d" % [_vp.size.x, _vp.size.y]
	_vp.get_texture().get_image().save_png(_shots + "/ve8_%s_%s_%s.png" % [_mode_str(), vs, name])


## The probe's stand-in for the soft keyboard at the editor (see the file header). -1 = "unknown", as on Windows.
func _set_sim_hook() -> void:
	if ed == null:
		return
	ed.keyboard_height = func() -> int: return _kb_sim


## Raises or lowers the simulated keyboard to `px` (0 = hidden) the way the product's own poll does.
func _kb_set(px: int) -> void:
	_kb_sim = px
	app._phone_kb_height = px
	app._apply_phone_orientation()
	await _frames(10)


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots", "--force-touch", "--debug", "--density"]):
			print("VE8 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE8 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE8 WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_shots = _arg("--shots", "")
	_touch = "--touch" in OS.get_cmdline_user_args() or "--force-touch" in OS.get_cmdline_user_args()
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_bridge = app.bridge

	# == Pure transforms (no UI) ================================================
	_pure()

	# == The fixture vault ======================================================
	var stamp := "%s-%d" % [_mode_str(), Time.get_ticks_msec()]
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve8-" + stamp
	var hub := "# Hub\n\nSee [[Alpha]] and [[Missing One]] and [site](https://example.com/x) today.\nPlain words here.\n\n## Part\n\ntext tail\n"
	_write(_root + "/Hub.md", hub)
	_write(_root + "/Alpha.md", "# Alpha\n\nalpha body\n")
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(2)
	var t0 := Time.get_ticks_msec()
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_bridge.vault_refresh_backlinks(2000)
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	await _open("Hub.md")
	_ok("CONTROL: Hub.md is open in the editor", ed != null and te != null and vw._browse_path == "Hub.md")
	if ed == null:
		_finish()
		return
	_phone = ed._phone
	print("VE8 info: viewport %s, phone=%s, touch=%s, window %s content_scale %s" % [_vp.size, _phone, _touch, vw.size, vw.content_scale_factor])
	if _phone:
		await _phone_leg()
	else:
		await _desk_leg()
	_finish()


# ===== pure ===================================================================

func repr_(s: Variant) -> String:
	return str(s).replace("\n", "\\n")


func _pure() -> void:
	var tb := "| Column 1 | Column 2 |\n| --- | --- |\n|   |   |\n|   |   |"
	var r: Dictionary = Ed.table_block("Intro line", 3, 3, 2, 2)
	_ok("table_block: a 2 x 2 goes after the caret's line behind a blank line, nothing removed", r["text"] == "Intro line\n\n" + tb + "\n", repr_(r["text"]))
	_ok("... the first header cell is selected to type over", String(r["text"]).substr(int(r["from"]), int(r["to"]) - int(r["from"])) == "Column 1", str(r))
	r = Ed.table_block("A\nB", 0, 0, 2, 2)
	_ok("table_block: text below gets a blank line between", r["text"] == "A\n\n" + tb + "\n\nB", repr_(r["text"]))
	r = Ed.table_block("A\n\nB", 0, 0, 2, 2)
	_ok("table_block: a blank line already below is not doubled", r["text"] == "A\n\n" + tb + "\n\nB", repr_(r["text"]))
	r = Ed.table_block("A\n\nB", 2, 2, 2, 2)
	_ok("table_block: on a blank line the line above gets its blank line and the blank line stays below", r["text"] == "A\n\n" + tb + "\n\nB", repr_(r["text"]))
	r = Ed.table_block("", 0, 0, 3, 3)
	_ok("table_block: an empty note gets a 3 x 3 (header, delimiter, three rows)", r["text"] == "| Column 1 | Column 2 | Column 3 |\n| --- | --- | --- |\n|   |   |   |\n|   |   |   |\n|   |   |   |\n", repr_(r["text"]))
	r = Ed.table_block("x", 0, 0, 99, 0)
	_ok("table_block: sizes are clamped to 1..8", String(r["text"]).count("Column") == 8 and String(r["text"]).split("\n").size() == 6, repr_(r["text"]))
	r = Ed.rule_block("Some text", 4, 4)
	_ok("rule_block: --- sits behind a blank line, so it cannot read as a setext underline", r["text"] == "Some text\n\n---\n", repr_(r["text"]))
	_ok("... and the caret lands after it", int(r["from"]) == String(r["text"]).length() and int(r["to"]) == int(r["from"]), str(r))
	r = Ed.rule_block("a\nb", 0, 0)
	_ok("rule_block: before a following line", r["text"] == "a\n\n---\n\nb", repr_(r["text"]))
	r = Ed.rule_block("one\ntwo\nthree", 0, 7)
	_ok("rule_block: a selection spanning lines goes after the line it ENDS on", r["text"] == "one\ntwo\n\n---\n\nthree", repr_(r["text"]))


# ===== desktop / tablet (not the phone form) ==================================

func _desk_leg() -> void:
	_ok("the window's own keyboard hook says -1 (unknown, no soft keyboard to ask) on this display, never 0 (hidden)", _real_kb == -1, str(_real_kb))
	_ok("DESK: there is no keyboard dock off the phone", ed._dock == null)
	_ok("DESK: the toolbar is the wrapping flow under the header", ed.find_child("EditorToolbar", true, false) is HFlowContainer)
	_ok("DESK: Normal, H1, H2, H3 are four chips and there is no single H chip", ed.buttons.has("h0") and ed.buttons.has("h3") and not ed.buttons.has("heading"))
	for k in ["table", "rule", "undo", "redo"]:
		_ok("DESK: the %s button is on the toolbar" % k, ed.buttons.has(k) and (ed.buttons[k] as Button).is_visible_in_tree())
	_ok("DESK: no Open chip, no Done", ed._open_chip == null and ed._done_button == null)
	te.release_focus()
	ed.goto_line(5)
	await _frames(4)
	_ok("DESK: jumping to a line still focuses the text (only the phone is spared the keyboard)", te.get_caret_line() == 5 and te.has_focus())
	te.grab_focus()
	te.set_caret_line(3)
	te.set_caret_column(2)
	await _frames(2)
	await _press(ed.buttons["table"])
	_ok("DESK: Table (tap) writes a 2 x 2 after the line", te.text.contains("Plain words here.\n\n| Column 1 | Column 2 |\n| --- | --- |\n|   |   |\n|   |   |\n\n## Part"), repr_(te.text))
	_ok("DESK: ... and a tap opened no variants card", not ed._vc_panel.visible)
	await _hold_btn(ed.buttons["table"], 0.7)
	_ok("DESK: holding Table opens the variants card with three sizes", ed._vc_panel.visible and ed._vc_box.get_child_count() == 3, str(ed._vc_box.get_child_count()))
	_ok("DESK: ... and the lift did not also write a table (still one)", te.text.count("| Column 1 |") == 1)
	var rows: Array = ed._vc_box.get_children()
	await _press(rows[1])
	_ok("DESK: the 3 x 3 row writes a three-column table", te.text.contains("| Column 1 | Column 2 | Column 3 |") and te.text.count("| Column 1 |") == 2, repr_(te.text))
	_ok("DESK: ... and closes the card", not ed._vc_panel.visible)
	var before := te.text
	await _press(ed.buttons["rule"])
	_ok("DESK: Rule writes --- on its own line", te.text.contains("\n\n---\n") and te.text.length() > before.length(), repr_(te.text))
	await _press(ed.buttons["undo"])
	_ok("DESK: Undo takes the rule back out", te.text == before, repr_(te.text))
	_ok("DESK: ... and Redo is live then", not (ed.buttons["redo"] as Button).disabled)
	await _press(ed.buttons["redo"])
	_ok("DESK: Redo puts it back", te.text.contains("\n\n---\n"), repr_(te.text))
	await _press(ed.buttons["preview"])
	_ok("DESK: in Preview Table, Rule, Undo and Redo are disabled", (ed.buttons["table"] as Button).disabled and (ed.buttons["rule"] as Button).disabled and (ed.buttons["undo"] as Button).disabled and (ed.buttons["redo"] as Button).disabled)
	await _press(ed.buttons["write"])
	await _shot("desk")


# ===== phone ==================================================================

## Frame pixels between the dock's bottom edge and the top of the simulated keyboard (negative = overlaps it).
func _dock_gap_px() -> float:
	var c := vw.content_scale_factor
	var bottom_px: float = float(vw.position.y) + ed._dock.get_global_rect().end.y * c
	return (float(_vp.size.y) - float(_kb_sim)) - bottom_px


## Done, if the keyboard row is showing it (leaves compact mode so a header button can be pressed again).
func _press_done_if_up() -> void:
	if ed._done_button != null and ed._done_button.is_visible_in_tree():
		await _press(ed._done_button)
		await _frames(3)


## Pixels of the frame inside `r` (window control units) that differ between two captures.
func _diff_px(a: Image, b: Image, r: Rect2) -> int:
	var c := vw.content_scale_factor
	var x0 := maxi(0, int(float(vw.position.x) + r.position.x * c))
	var y0 := maxi(0, int(float(vw.position.y) + r.position.y * c))
	var x1 := mini(a.get_width(), int(float(vw.position.x) + r.end.x * c))
	var y1 := mini(a.get_height(), int(float(vw.position.y) + r.end.y * c))
	var n := 0
	for y in range(y0, y1, 2):
		for x in range(x0, x1, 2):
			if a.get_pixel(x, y).is_equal_approx(b.get_pixel(x, y)) == false:
				n += 1
	return n


## The scene graph says a card is open; only the frame says whether it is DRAWN (an opaque sibling can cover a
## `visible` control). Capture with the card, hide it, capture again: its own rectangle must differ.
func _card_drawn(label: String, card: Control) -> void:
	var r := card.get_global_rect()
	await _frames(3)
	var with_card := _vp.get_texture().get_image()
	card.visible = false
	await _frames(3)
	var without := _vp.get_texture().get_image()
	card.visible = true
	await _frames(2)
	var moved := _diff_px(with_card, without, r)
	var area := int(r.size.x * r.size.y * vw.content_scale_factor * vw.content_scale_factor / 4.0)
	_ok("PHONE: %s is DRAWN: hiding it changes at least 40%% of its own rectangle in the frame" % label, moved >= int(0.4 * float(area)), "%d of %d sampled px moved" % [moved, area])


func _min_btn_h(root: Node) -> float:
	var m := 1e9
	for c in root.find_children("*", "Button", true, false):
		var b := c as Button
		if b.is_visible_in_tree() and not (b.get_parent() is Button):
			m = minf(m, b.get_global_rect().size.y)
	return m


func _col_of(line: int, needle: String) -> int:
	return te.get_line(line).find(needle)


func _phone_leg() -> void:
	var dock: PanelContainer = ed._dock
	_ok("PHONE: the editor has a keyboard dock, hidden until the text is used", dock != null and not dock.visible)
	_ok("PHONE: opening a note in Write did not take focus (so no keyboard)", not te.has_focus())
	_ok("PHONE: the dock is the editor's LAST child, under the footer", ed.get_child(ed.get_child_count() - 1) == dock)
	_ok("PHONE: the single H chip replaces Normal/H1/H2/H3", ed.buttons.has("heading") and not ed.buttons.has("h1"))
	for k in ["table", "rule", "undo", "redo", "heading"]:
		_ok("PHONE: the %s button is in the dock's row" % k, ed.buttons.has(k) and dock.is_ancestor_of(ed.buttons[k]))
	_ok("PHONE: Open and Done exist and start hidden", ed._open_chip != null and not ed._open_chip.visible and ed._done_button != null and not ed._done_button.visible)
	var win_h0 := vw.size.y
	_ok("the window's own keyboard hook says -1 (unknown, no soft keyboard to ask) on this display, never 0 (hidden)", _real_kb == -1, str(_real_kb))
	ed.goto_line(5)
	await _frames(4)
	_ok("PHONE: jumping to a line (a backlink, an outline row) puts the caret there but takes no focus, so no keyboard", te.get_caret_line() == 5 and not te.has_focus(), "line %d focus %s" % [te.get_caret_line(), te.has_focus()])
	await _shot("0_open")

	# -- 1. tap the text: focus, and the dock appears (keyboard height unknown here: the focus stands in)
	_kb_sim = -1
	await _tap(vw, te.get_global_rect().position + Vector2(40, 40))
	await _frames(6)
	_ok("PHONE: tapping the text focuses it", te.has_focus())
	_ok("PHONE: the dock shows (unknown keyboard height = the text's focus stands in, not 'hidden')", dock.visible)
	_ok("PHONE: ... with Done", ed._done_button.visible)
	var okb := vw.get_ok_button()
	_ok("PHONE: the dialog's own Close bar is hidden while the row is up (it would take the row's touches)", okb != null and not okb.is_visible_in_tree())
	var hv0 := (await _hover_at(ed.buttons["bold"].get_global_rect().get_center()))
	_ok("PHONE: with the keyboard height UNKNOWN the control under the Bold chip is the chip", hv0 == ed.buttons["bold"] or (hv0 != null and ed.buttons["bold"].is_ancestor_of(hv0)), str(hv0))
	_ok("PHONE: ... and the header is NOT folded while the height is unknown", not ed._compact and ed._header_row.visible)

	# -- 2. the keyboard comes up (simulated): the window sits above it and the row rides on it
	var kb_px := int(float(_vp.size.y) * (0.33 if _vp.size.y > _vp.size.x else 0.5))
	var ed_id: int = ed.get_instance_id()
	await _kb_set(kb_px)
	_ok("PHONE: raising the keyboard did not rebuild the editor (the same editor, still focused: a rebuild would drop the keyboard)", is_instance_valid(ed) and ed.get_instance_id() == ed_id and vw._editor == ed and te.has_focus())
	print("VE8 info: simulated keyboard %d px of %d, window now %s (was %d), dock rect %s, compact %s" % [kb_px, _vp.size.y, vw.size, win_h0, dock.get_global_rect(), ed._compact])
	_ok("PHONE: with the keyboard up the window was re-presented above it", vw.size.y < win_h0, "%d -> %d" % [win_h0, vw.size.y])
	_ok("PHONE: the dock is visible and its bottom edge is above the keyboard's top", dock.visible and _dock_gap_px() >= -1.0, "gap %.1f px" % _dock_gap_px())
	_ok("PHONE: ... and it is DOCKED to it, within 0.12 of the keyboard's height (not floating far above)", _dock_gap_px() <= 0.12 * float(kb_px), "gap %.1f px of kb %d" % [_dock_gap_px(), kb_px])
	_ok("PHONE: the header and footer folded away to give the text the room", ed._compact and not ed._header_row.visible and not ed._status_row.visible)
	var tr := te.get_global_rect()
	_ok("PHONE: the text still has room (at least 60 dp tall)", tr.size.y >= 60.0, str(tr))
	_ok("PHONE: the text sits above the dock", tr.end.y <= dock.get_global_rect().position.y + 1.0)
	_ok("PHONE: every dock button is at least 44 dp tall", _min_btn_h(dock) >= 43.5, str(_min_btn_h(dock)))
	await _shot("1_kb_up")

	# -- 3. a toolbar press keeps the text's focus (a button that took it would drop the keyboard)
	te.set_caret_line(3)
	te.set_caret_column(5)
	te.deselect()
	await _frames(2)
	var bc: Vector2 = ed.buttons["bold"].get_global_rect().get_center()
	await _hover(vw, bc)
	var hv := vw.gui_get_hovered_control()
	print("VE8 info: hovered at Bold's centre %s: %s %s parent %s" % [bc, hv, hv.get_path() if hv != null else "-", hv.get_parent() if hv != null else "-"])
	_ok("PHONE: the control under the Bold chip IS the chip (nothing of the window sits over the row)", hv == ed.buttons["bold"] or (hv != null and ed.buttons["bold"].is_ancestor_of(hv)), str(hv))
	await _press(ed.buttons["bold"])
	_ok("PHONE: pressing Bold in the dock leaves the focus on the text", vw.gui_get_focus_owner() == te, str(vw.gui_get_focus_owner()))
	_ok("PHONE: ... and it wrote the bold markers", te.text.contains("****"), repr_(te.text))
	await _press(ed.buttons["undo"])

	# -- 4. the Open chip follows the caret
	te.set_caret_line(2)
	te.set_caret_column(8)
	await _wait(450)
	_ok("PHONE: with the caret inside [[Alpha]] the Open chip shows", ed._open_chip.visible, str(ed._open_info))
	_ok("PHONE: ... and it is at least 44 dp tall", ed._open_chip.get_global_rect().size.y >= 43.5, str(ed._open_chip.get_global_rect()))
	await _shot("2_open_chip")
	te.set_caret_line(3)
	te.set_caret_column(3)
	await _wait(450)
	_ok("PHONE: on a plain line the chip is gone", not ed._open_chip.visible)
	te.set_caret_line(2)
	te.set_caret_column(_col_of(2, "[[Missing One]]") + 4)
	await _wait(450)
	_ok("PHONE: inside the unresolved link [[Missing One]] the chip shows too (follow offers to create, never silently)", ed._open_chip.visible and String(ed._open_info.get("target", "")) == "Missing One", str(ed._open_info))
	te.set_caret_line(2)
	te.set_caret_column(_col_of(2, "(https://example.com/x)") + 6)
	await _wait(450)
	_ok("PHONE: inside an external link the chip shows and is marked external", ed._open_chip.visible and bool(ed._open_info.get("external", false)), str(ed._open_info))
	te.set_caret_line(2)
	te.set_caret_column(8)
	await _wait(450)
	await _press(ed._open_chip)
	await _frames(8)
	_ok("PHONE: pressing Open on [[Alpha]] opened Alpha", vw._browse_path == "Alpha.md", vw._browse_path)
	await _open("Hub.md")
	await _tap(vw, te.get_global_rect().position + Vector2(40, 40))
	await _frames(4)
	dock = ed._dock

	# -- 5. long-press: the H chip's card, and the tap the lift would fire is swallowed
	te.set_caret_line(3)
	te.set_caret_column(2)
	te.deselect()
	await _frames(2)
	_ok("PHONE: the line starts as body text (H chip reads H)", (ed.buttons["heading"] as Button).text == "H", (ed.buttons["heading"] as Button).text)
	await _hold_btn(ed.buttons["heading"], 0.7)
	_ok("PHONE: holding H opens a card of Normal, H1, H2, H3", ed._vc_panel.visible and ed._vc_box.get_child_count() == 4, str(ed._vc_box.get_child_count()))
	_ok("PHONE: ... and the lift did NOT also step the heading", not te.get_line(3).begins_with("#"), te.get_line(3))
	var vr: Rect2 = ed._vc_panel.get_global_rect()
	_ok("PHONE: the card is on screen and clears the dock", vr.position.y >= 0.0 and vr.end.y <= dock.get_global_rect().position.y + 1.0 and vr.end.x <= vw.size.x / vw.content_scale_factor + 1.0, str(vr))
	_ok("PHONE: its rows are at least 44 dp", ed._vc_box.get_child(0).get_global_rect().size.y >= 43.5)
	await _shot("3_h_card")
	await _card_drawn("the H card", ed._vc_panel)
	await _press(ed._vc_box.get_child(3))
	_ok("PHONE: choosing H3 writes ### and closes the card", te.get_line(3).begins_with("### ") and not ed._vc_panel.visible, te.get_line(3))
	_ok("PHONE: the chip now reads H3", (ed.buttons["heading"] as Button).text == "H3", (ed.buttons["heading"] as Button).text)
	await _press(ed.buttons["heading"])
	_ok("PHONE: a plain tap on H3 steps to Normal", not te.get_line(3).begins_with("#"), te.get_line(3))
	await _press(ed.buttons["heading"])
	_ok("PHONE: the next tap makes it H1", te.get_line(3).begins_with("# ") and not te.get_line(3).begins_with("## "), te.get_line(3))
	await _press(ed.buttons["heading"])
	await _press(ed.buttons["heading"])
	await _press(ed.buttons["heading"])
	_ok("PHONE: H1, H2, H3, Normal: three more taps come back to Normal", not te.get_line(3).begins_with("#"), te.get_line(3))
	await _hold_btn(ed.buttons["heading"], 0.7)
	_ok("PHONE: a second hold reopens the card", ed._vc_panel.visible)
	await _tap(vw, te.get_global_rect().position + Vector2(40, 10))
	_ok("PHONE: a tap outside the card closes it", not ed._vc_panel.visible)
	var t_before := te.text
	await _hold_btn(ed.buttons["table"], 0.7)
	_ok("PHONE: holding Table offers 2 x 2, 3 x 3 and 4 x 4 and writes nothing yet", ed._vc_panel.visible and ed._vc_box.get_child_count() == 3 and te.text == t_before)
	await _press(ed._vc_box.get_child(1))
	_ok("PHONE: the 3 x 3 row wrote a three-column table with three body rows", te.text.contains("| Column 1 | Column 2 | Column 3 |") and te.text.count("|   |   |   |") == 3, repr_(te.text))
	_ok("PHONE: nothing that was typed before was lost", te.text.contains("See [[Alpha]] and [[Missing One]]") and te.text.contains("text tail") and te.text.contains("Plain words here."))
	await _press(ed.buttons["undo"])
	_ok("PHONE: Undo takes the whole table back in one step", te.text == t_before, repr_(te.text))
	await _press(ed.buttons["table"])
	_ok("PHONE: a plain tap on Table writes a 2 x 2", te.text.count("| Column 1 | Column 2 |") == 1 and te.text.count("|   |   |") == 2 and not te.text.contains("Column 3"))
	await _press(ed.buttons["undo"])
	await _press(ed.buttons["rule"])
	_ok("PHONE: Rule writes --- on its own line", te.text.contains("\n\n---\n"), repr_(te.text))
	await _press(ed.buttons["undo"])
	_ok("PHONE: Redo is live after an undo", not (ed.buttons["redo"] as Button).disabled)
	await _press(ed.buttons["redo"])
	await _press(ed.buttons["undo"])
	_ok("PHONE: the note is back to what it was", te.text == t_before, repr_(te.text))

	# -- 6. typing, Done, nothing lost
	te.set_caret_line(te.get_line_count() - 2)
	te.set_caret_column(te.get_line(te.get_line_count() - 2).length())
	te.grab_focus()
	await _frames(3)
	await _type(" ZEBRA")
	_ok("PHONE: real key events typed into the text", te.text.contains("text tail ZEBRA"), repr_(te.text))
	var typed := te.text
	await _press(ed._done_button)
	await _frames(4)
	_ok("PHONE: Done released the focus", not te.has_focus())
	_ok("PHONE: ... and left the note in Write with the text intact", ed.mode() == "write" and te.text == typed)
	await _kb_set(0)
	_ok("PHONE: with the keyboard hidden and no focus the dock is gone", not dock.visible)
	_ok("PHONE: ... and the header and footer are back", ed._header_row.visible and ed._status_row.visible and not ed._compact)
	_ok("PHONE: the window is back to its full height", vw.size.y == win_h0, "%d vs %d" % [vw.size.y, win_h0])
	await _wait(1800)
	_ok("PHONE: the typed text autosaved to disk (never lost)", _disk("Hub.md").contains("text tail ZEBRA"), _disk("Hub.md"))
	await _shot("4_done")

	# -- 7. keyboard hidden by the system with focus kept (a hardware keyboard): the row stays, Done goes
	await _tap(vw, te.get_global_rect().position + Vector2(40, 40))
	await _kb_set(kb_px)
	await _kb_set(0)
	_ok("PHONE: focus kept with the keyboard height 0: the dock stays", dock.visible)
	_ok("PHONE: ... and so does Done (it ends the editing and gives the window's Close bar back)", ed._done_button.visible)
	_ok("PHONE: ... and the header is back (the room was given back)", ed._header_row.visible and not ed._compact)

	# -- 8. Preview has no dock
	await _kb_set(kb_px)
	ed.set_mode("preview")
	await _frames(4)
	_ok("PHONE: in Preview the dock is hidden even with the keyboard up", not dock.visible)
	_ok("PHONE: ... and the editor is back in the page (not an overlay) with its header showing", not ed.top_level and ed._header_row.visible)
	ed.set_mode("write")
	await _frames(4)

	# -- 9. the VE-5 sheet under the keyboard: a field tapped low in the page is scrolled to, above the keys
	await _kb_set(kb_px)
	ed.set_mode("write")
	await _frames(3)
	await _press_done_if_up()
	vw._set_side(true)
	await _frames(8)
	var sheet = vw._side_panel
	if sheet != null:
		sheet.keyboard_height = func() -> int: return _kb_sim
	_ok("PHONE: the links/outline/properties sheet exists to check and replaced the editor", sheet != null and sheet.visible and not ed.visible)
	if sheet != null:
		sheet.show_tab("props")
		await _frames(6)
		await _kb_set(kb_px)
		var field: LineEdit = null
		for c in sheet.find_children("*", "LineEdit", true, false):
			if (c as LineEdit).is_visible_in_tree():
				field = c
				break
		_ok("PHONE: the properties sheet has a field to type into", field != null)
		if field != null:
			field.grab_focus()
			await _frames(10)
			var fr: Rect2 = field.get_global_rect()
			var vis := vw.get_visible_rect() if vw.has_method("get_visible_rect") else Rect2(Vector2.ZERO, Vector2(vw.size) / vw.content_scale_factor)
			var ss: ScrollContainer = sheet.find_child("SideScroll", true, false)
			print("VE8 info: sheet scroll: vertical %d, scroll_vertical %d, max %s, follow_focus %s" % [ss.vertical_scroll_mode, ss.scroll_vertical, ss.get_v_scroll_bar().max_value, ss.follow_focus])
			_ok("PHONE: with the keyboard up the focused field is scrolled to inside the window above it", fr.position.y >= 0.0 and fr.end.y <= vis.end.y + 1.0, "%s in %s" % [fr, vis])
			_ok("PHONE: ... the sheet became the window-filling overlay for it, and the dialog's own bar stepped aside", sheet.top_level and sheet._overlay, "top_level %s" % sheet.top_level)
			var ph := sheet.find_child("SideClose", true, false)
			_ok("PHONE: the sheet's own Back to the note is reachable in the overlay (inside the window)", ph != null and (ph as Control).get_global_rect().position.y >= 0.0 and (ph as Control).get_global_rect().end.y <= vis.end.y + 1.0)
			field.release_focus()
			await _frames(6)
			_ok("PHONE: leaving the field puts the sheet back in the page", not sheet.top_level and not sheet._overlay)
		await _shot("5_sheet_kb")
	vw._set_side(false)
	await _kb_set(0)
