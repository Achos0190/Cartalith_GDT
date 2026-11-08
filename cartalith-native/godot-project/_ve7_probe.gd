extends Node
## VE-7 of `VAULT_EDITOR_PROPOSAL.md` s.7: tables, images, embeds, task ticking, chronos rendering and callout
## colours in Preview. Bar: *each renders in Preview; a chronos row moves the timeline.*
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve7_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch   (the phone leg)
##
## **Windowed**, under a wiped user directory (`APPDATA=<empty dir>`), on a throw-away vault in TEMP. Exit 0 = every
## check held, 1 = a check failed, 2 = could not run. A box, a chronos row and an embed's header are clicked with a real
## mouse (or emulated touch) press at the pixel where the rendered paragraph is; the drawn pictures are read back from the
## frame buffer (pure red and pure blue squares, and a pure green one outside the vault that must never appear).
##
## What it proves: a note embed shows the embedded note's first section, a heading embed that heading, collapsed past six
## lines and expanding on a click; a missing target, a cycle (A embeds B embeds A, and a note that embeds itself), a chain
## deeper than the cap and a path out of the vault each draw a clear notice and the probe finishes; two pictures draw (one
## from the vault root, one from a subfolder with a percent-encoded name) and the picture OUTSIDE the vault does not;
## an embedded note's tasks are not clickable; a table, a callout in its type's colour and `_italic_` render; clicking a
## task box ticks the line in the editor (one undo step) and the autosave writes it; clicking a chronos row moves the
## shell's year through `tl_set_year` (and a year past the world's range is clamped and said so); the note on disk is
## never changed by a render.

const Ed := preload("res://shell/markdown_editor.gd")

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var ed
var te: TextEdit
var _root := ""
var _outside := ""
var _shots := ""
var _touch := false
var _phone := false
var _n := 0
var _fails: Array = []
var _year_events := 0


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VE7  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE7  !!  %s   %s" % [label, detail])


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


func _until(cond: Callable, ms: int) -> int:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < ms:
		if cond.call():
			return Time.get_ticks_msec() - t
		await get_tree().process_frame
	return -1


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _write_png(path: String, color: Color, size: int) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var img := Image.create(size, size, false, Image.FORMAT_RGBA8)
	img.fill(color)
	img.save_png(path)


func _disk(rel: String) -> String:
	return FileAccess.get_file_as_string(_root + "/" + rel)


func _find_all(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_find_all(c, cls, out)


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


func _tap(win: Window, local: Vector2) -> void:
	await _mouse(win, local, true)
	await _mouse(win, local, false)


## A REAL click on `b`: scrolls it into view if a scroll container hides it, then taps its centre.
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


func _key(code: Key, ctrl: bool = false) -> void:
	for pressed in [true, false]:
		var k := InputEventKey.new()
		k.pressed = pressed
		k.keycode = code
		k.physical_keycode = code
		k.ctrl_pressed = ctrl
		_vp.push_input(k)
	await _frames(3)


func _finish() -> void:
	DirAccess.remove_absolute(_outside)
	print("VE7 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _grab() -> void:
	ed = vw._editor
	te = ed.text_edit if ed != null else null


## Puts the editor in `m` with a real press on its mode button, only when it is not already there (the buttons are
## segments: pressing the one that is on does nothing useful, and a probe must not guess which state it is in).
func _set_mode(m: String) -> void:
	if ed.mode() != m:
		await _press(ed.buttons[m])
		await _frames(4)


func _open(rel: String) -> void:
	if _phone:
		vw._set_side(false)
	vw._link_back.clear()
	vw._enter_note(rel)
	await _frames(6)
	_grab()


## The paragraph (a rendered line) of Preview whose text contains `needle`, or -1.
func _para(needle: String) -> int:
	var parts: PackedStringArray = ed.preview.get_parsed_text().split("\n")
	for i in parts.size():
		if String(parts[i]).contains(needle):
			return i
	return -1


## A REAL click in Preview on the paragraph holding `needle`, `dx` pixels in from the label's left edge (the
## padded text starts about 14 in). Scrolls the paragraph into view first; returns false when it is not there.
func _click_para(needle: String, dx: float) -> bool:
	var idx := _para(needle)
	if idx < 0:
		return false
	var pv: RichTextLabel = ed.preview
	pv.scroll_to_paragraph(idx)
	await _frames(4)
	var bar := pv.get_v_scroll_bar()
	var y := float(pv.get_paragraph_offset(idx)) - (bar.value if bar != null else 0.0) + 20.0
	var pos := pv.get_global_rect().position + Vector2(dx, y)
	await _tap(vw, pos)
	await _frames(4)
	return true


## Pure red, pure blue and pure green pixels of the frame inside `r`, sampled on every second pixel of both axes (so
## a figure is a quarter of the true count; the bars below are set with that in mind). Tolerance 0.06 per channel.
func _count3(r: Rect2) -> Array:
	var img := _vp.get_texture().get_image()
	var red := 0
	var blue := 0
	var green := 0
	var x0 := maxi(0, int(r.position.x))
	var y0 := maxi(0, int(r.position.y))
	var x1 := mini(img.get_width(), int(r.end.x))
	var y1 := mini(img.get_height(), int(r.end.y))
	for y in range(y0, y1, 2):
		for x in range(x0, x1, 2):
			var p := img.get_pixel(x, y)
			if p.r > 0.94 and p.g < 0.06 and p.b < 0.06:
				red += 1
			elif p.b > 0.94 and p.r < 0.06 and p.g < 0.06:
				blue += 1
			elif p.g > 0.94 and p.r < 0.06 and p.b < 0.06:
				green += 1
	return [red, blue, green]


func _shot(name: String) -> void:
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	await _frames(4)
	_vp.get_texture().get_image().save_png(_shots + "/ve7_%s_%s.png" % [("phone" if _phone else ("touch" if _touch else "desk")), name])


func _mode_str() -> String:
	return "phone" if _phone else ("touch" if _touch else "desk")


func _on_timeline_changed() -> void:
	_year_events += 1


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots", "--force-touch", "--debug", "--density"]):
			print("VE7 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE7 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE7 WATCHDOG TIMEOUT")
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

	# == The fixture vault (names no generated place can share) ================
	var stamp := "%s-%d" % [_mode_str(), Time.get_ticks_msec()]
	var tmp := OS.get_environment("TEMP").replace("\\", "/")
	_root = tmp + "/cartalith-ve7-" + stamp
	_outside = tmp + "/cartalith-ve7-outside-" + stamp + ".png"
	_write_png(_outside, Color(0, 1, 0), 64)  ## pure green, one level ABOVE the vault folder: must never be drawn
	_write_png(_root + "/pic.png", Color(1, 0, 0), 64)  ## pure red
	_write_png(_root + "/pics/map one.png", Color(0, 0, 1), 64)  ## pure blue, a folder and a space in the name
	var hub := "---\ntags: [probe]\n---\n# Hub\n\n- [ ] first task\n- [x] second task\n\n![[Alpha]]\n\n![[Alpha#Origins]]\n\n"
	hub += "![[Missing target]]\n\n![[Loop1]]\n\n![[D1]]\n\n![[Long]]\n\n![[pic.png|120]]\n\n![alt](pics/map%20one.png)\n\n"
	hub += "![[../" + _outside.get_file() + "]]\n\n![escape](../" + _outside.get_file() + ")\n\n"
	hub += "> [!warning] Mind the gap\n> careful now\n\n| Name | Pop |\n|---|--:|\n| Brindle | 120 |\n\n_slanted_ words\n\n"
	hub += "```chronos\n@ [1000~1100] Peace {Reign}\n- [300] Founding | the first stone\n- [1300] Siege | the long one\n```\n"
	_write(_root + "/Hub.md", hub)
	_write(_root + "/Alpha.md", "Intro paragraph of alpha.\n\n# Origins\nOrigin text here.\n- [ ] embedded task\n\n# Later\nlater text\n")
	_write(_root + "/Loop1.md", "# Loop1\n\n![[Loop2]]\n")
	_write(_root + "/Loop2.md", "# Loop2\n\n![[Loop1]]\n")
	_write(_root + "/Self.md", "# Self\n\n![[Self]]\n")
	_write(_root + "/D1.md", "![[D2]]\n")
	_write(_root + "/D2.md", "![[D3]]\n")
	_write(_root + "/D3.md", "![[D4]]\n")
	_write(_root + "/D4.md", "![[D5]]\n")
	_write(_root + "/D5.md", "bottom of the chain\n")
	_write(_root + "/Long.md", "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\n")
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
	var range0: Dictionary = app.tl_range()
	_ok("CONTROL: a fresh world's strip range is the fallback axis -400..1200", not bool(range0["from_world"]) and int(range0["min"]) == -400 and int(range0["max"]) == 1200, str(range0))
	app.timeline_changed.connect(_on_timeline_changed)
	var hub_before := _disk("Hub.md")

	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	await _open("Hub.md")
	_ok("CONTROL: Hub.md is open in the editor", ed != null and te != null and vw._browse_path == "Hub.md")
	if ed == null:
		_finish()
		return
	_phone = ed._phone
	await _set_mode("preview")
	_ok("Preview is showing", ed.mode() == "preview" and ed.preview.is_visible_in_tree())
	await _frames(8)
	var bb: String = ed.preview.text
	var plain: String = ed.preview.get_parsed_text()
	print("VE7 info: viewport %s, phone=%s, touch=%s, window %s, preview %s" % [_vp.size, _phone, _touch, vw.size, ed.preview.size])
	await _shot("preview_top")

	# == 1. Embeds of notes ====================================================
	_ok("a note embed shows the embedded note's first section", plain.contains("Intro paragraph of alpha."), plain.substr(0, 400))
	_ok("... under a header naming the note", plain.contains("▪ Alpha") or plain.contains("▸ Alpha"))
	_ok("a heading embed shows that section (and the heading's title)", plain.contains("Alpha › Origins") and plain.contains("Origin text here."))
	_ok("... and not the rest of the note", not plain.contains("later text"))
	_ok("a missing target is a clear notice, not blank", plain.contains("Embed not found: Missing target"))
	_ok("A embeds B embeds A: the loop is cut with the chain named", plain.contains("Embed skipped, it would repeat itself: Hub → Loop1 → Loop2 → Loop1"), plain)
	_ok("a chain deeper than the cap (Hub > D1 > D2 > D3 > D4) stops at the fourth level", plain.contains("Embed skipped, more than 3 levels deep: D4"), plain)
	_ok("... and the notes under the cap are drawn (D1 holds D2 holds D3) but the bottom of the chain never is", plain.contains("▪ D3") and not plain.contains("bottom of the chain"))
	_ok("only Hub's own two tasks are clickable boxes: the embedded note's task is a read-only view's", _count_task_links(bb) == 2, str(_count_task_links(bb)))
	_ok("rendering wrote nothing: Hub.md on disk is untouched", _disk("Hub.md") == hub_before)

	# == 2. A long embed collapses, a click expands it, a click collapses it ==========
	_ok("a long embed shows six lines and says there is more", plain.contains("line 6") and not plain.contains("line 7") and plain.contains("more in this note"))
	var clicked := await _click_para("Long", 40.0)
	_ok("the Long embed's header was found and clicked", clicked)
	plain = ed.preview.get_parsed_text()
	_ok("a click on the header expands it to the whole section", plain.contains("line 8") and not plain.contains("more in this note"), plain.substr(0, 300))
	await _click_para("Long", 40.0)
	plain = ed.preview.get_parsed_text()
	_ok("a second click collapses it again", not plain.contains("line 7") and plain.contains("more in this note"))

	# == 3. Pictures: two inside the vault draw; the one outside does not ================
	ed.preview.scroll_to_paragraph(maxi(_para("careful"), 0))
	await _frames(4)
	_ok("a path that leaves the vault is refused in words, for the wikilink and the Markdown image alike", plain.count("Embed refused, outside this vault") == 2, str(plain.count("Embed refused, outside this vault")))
	# Pictures: scroll to the one and read the frame buffer.
	var found_red := 0
	var found_blue := 0
	var found_green := 0
	var pv: RichTextLabel = ed.preview
	var bar := pv.get_v_scroll_bar()
	var steps := int(ceil(float(pv.get_content_height()) / maxf(1.0, float(pv.size.y)))) + 1
	for i in steps:
		bar.value = minf(bar.max_value, float(i) * pv.size.y * 0.8)
		await _frames(5)
		## Control units -> frame pixels: the window's offset, and its content scale (2.8125 on the phone leg).
		var gr := pv.get_global_rect()
		var r := Rect2(Vector2(vw.position) + gr.position * vw.content_scale_factor, gr.size * vw.content_scale_factor)
		var c3 := _count3(r)
		found_red = maxi(found_red, int(c3[0]))
		found_blue = maxi(found_blue, int(c3[1]))
		found_green = maxi(found_green, int(c3[2]))
	_ok("the picture at the vault root is drawn (pure red pixels in Preview)", found_red > 400, str(found_red))
	_ok("the picture in a folder with a %20 name is drawn (pure blue pixels)", found_blue > 400, str(found_blue))
	_ok("the picture one folder ABOVE the vault is never drawn (no pure green pixel)", found_green == 0, str(found_green))
	_ok("the red picture is no wider than the |120 the note asked for (a bounded blob, not 560 wide)", bb.contains("[img width=120]"), bb.substr(bb.find("[img"), 80))

	# == 4. Callout colour, table, _italic_ =====================================
	_ok("a warning callout is drawn in the warn colour with its glyph", bb.contains("[color=#e0a840]▎ ⚠ [b]Mind the gap[/b]"), "")
	_ok("a table renders as a table with its header and a right-aligned cell", bb.contains("[table=2]") and plain.contains("Brindle") and bb.contains("[right]120[/right]"))
	_ok("_slanted_ is italic", bb.contains("[i]slanted[/i] words"))
	await _shot("preview_bottom")

	# == 5. Task ticking ========================================================
	var src_before: String = te.text
	var clicked_task := await _click_para("first task", 36.0)
	_ok("the first task's box was found and clicked", clicked_task)
	_ok("the click ticked the line in the editor's source", te.text.contains("- [x] first task") and not te.text.contains("- [ ] first task"), te.text.substr(0, 120))
	_ok("... and nothing else in the text changed", te.text.replace("- [x] first task", "- [ ] first task") == src_before)
	_ok("Preview shows it done", ed.preview.get_parsed_text().contains("☑  first task"))
	await _until(func(): return _disk("Hub.md").contains("- [x] first task"), 5000)
	_ok("the autosave wrote it", _disk("Hub.md").contains("- [x] first task"), _disk("Hub.md").substr(0, 100))
	await _set_mode("write")
	te.grab_focus()
	await _key(KEY_Z, true)
	_ok("one Ctrl+Z un-ticks it (one undo step)", te.text == src_before, te.text.substr(0, 120))
	await _set_mode("preview")
	await _wait(2600)
	await _click_para("second task", 36.0)
	_ok("clicking a done task's box unticks it", te.text.contains("- [ ] second task"))
	await _set_mode("write")
	te.grab_focus()
	await _key(KEY_Z, true)
	_ok("one more Ctrl+Z puts the second task back", te.text == src_before, te.text.substr(0, 120))
	await _until(func(): return _disk("Hub.md") == hub_before, 6000)
	_ok("the note on disk is back to its original bytes", _disk("Hub.md") == hub_before)

	# == 6. Chronos: the block renders, and a row moves the year ==================
	await _set_mode("preview")
	await _frames(6)
	plain = ed.preview.get_parsed_text()
	bb = ed.preview.text
	_ok("the chronos block draws as a timeline with its count and three rows in year order",
		plain.contains("Timeline") and plain.contains("3 entries") and plain.find("Founding") < plain.find("Peace") and plain.find("Peace") < plain.find("Siege"), plain.substr(plain.find("Timeline"), 300))
	_ok("... with its axis from the first year to the last", plain.contains("300 ├") and plain.contains("┤ 1300"))
	var years_before := _year_events
	var y0: int = app.tl_year()
	await _click_para("Peace", 110.0)
	await _frames(4)
	_ok("clicking the Peace row moves the shell's year to 1000 (through tl_set_year)", app.tl_year() == 1000 and _bridge.get_civ_year() == 1000, "%d / %d (was %d)" % [app.tl_year(), _bridge.get_civ_year(), y0])
	_ok("... and the timeline signal fired, so the strip and the map follow", _year_events > years_before, str(_year_events))
	await _click_para("Founding", 110.0)
	_ok("clicking the Founding row moves it to 300", app.tl_year() == 300, str(app.tl_year()))
	await _click_para("Siege", 110.0)
	_ok("a row past the world's range (1300) is clamped to the range's end, 1200, not written as 1300", app.tl_year() == 1200 and _bridge.get_civ_year() == 1200, str(app.tl_year()))
	_ok("... and the status line says where it went", String(app._status_labels["hint"].text).contains("outside this world's range") and String(app._status_labels["hint"].text).contains("1200"), String(app._status_labels["hint"].text))
	_ok("moving the year wrote nothing to the note", _disk("Hub.md") == hub_before)
	await _shot("chronos_clicked")

	# == 7. A note that embeds itself, opened directly ============================
	await _open("Self.md")
	await _set_mode("preview")
	await _frames(6)
	plain = ed.preview.get_parsed_text()
	_ok("a note embedding itself is cut at once, naming the loop", plain.contains("Embed skipped, it would repeat itself: Self → Self"), plain)

	_finish()


func _count_task_links(bbcode: String) -> int:
	var n := 0
	var i := bbcode.find("[url=task:")
	while i >= 0:
		n += 1
		i = bbcode.find("[url=task:", i + 1)
	return n
