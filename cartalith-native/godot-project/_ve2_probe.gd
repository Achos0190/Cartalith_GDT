extends Node
## VE-2 of `VAULT_EDITOR_PROPOSAL.md` s.7: `[[` suggestions with headings, aliases and entities, plus
## auto-pair. Bar: *the picker is never unfiltered; Enter, Tab, Escape and arrows work; at least 44 dp
## rows on touch.* Drives the real app, the real vault window and a real vault folder with real clicks
## and typed keys; every file assertion re-reads the disk.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve2_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##
## **Windowed** (pixels and input routing). `--touch` makes pointer events carry the emulated-mouse
## device id a finger's do (rows are then 44 px); the window is NOT put in phone density (VE-8 owns the
## phone). Run under a wiped user directory (`APPDATA=<empty dir>`): it deletes `user://vault.json` and
## binds a throw-away vault. Exit 0 = every check held, 1 = a check failed, 2 = could not run.
##
## A pointer note carried from `_ve0_probe.gd`: a SubViewport routes motion only after
## `NOTIFICATION_VP_MOUSE_ENTER`. And a script error inside an `await` hangs a probe, so every wait
## here is guarded and a missing control ends the run with the failure on record.

const SEED := 552017
## Notes besides the fixtures, so that "the whole vault" is far more than the card's cap.
const FILLER := 30

var _vp: SubViewport
var app: Node
var _bridge
var _root := ""
var _shots := ""
var _touch := false
var _n := 0
var _fails: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VE2  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE2  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _disk(rel: String) -> String:
	return FileAccess.get_file_as_string(_root + "/" + rel)


func _shot(name: String) -> void:
	await _frames(6)
	await RenderingServer.frame_post_draw
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	_vp.get_texture().get_image().save_png(_shots + "/" + ("touch_" if _touch else "desk_") + name)


func _find_all(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_find_all(c, cls, out)


func _find_button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text).to_lower() == text.to_lower() and (n as Button).is_visible_in_tree():
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null


func _leaf_in(it: TreeItem, rel: String) -> TreeItem:
	if it == null:
		return null
	if it.get_metadata(0) is String and String(it.get_metadata(0)) == rel:
		return it
	var c := it.get_first_child()
	while c != null:
		var r := _leaf_in(c, rel)
		if r != null:
			return r
		c = c.get_next()
	return null


func _mouse(vw: Window, local: Vector2, pressed: bool) -> void:
	var p := Vector2(vw.position) + local
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = pressed
	ev.position = p
	ev.global_position = p
	if _touch:
		ev.device = InputEvent.DEVICE_ID_EMULATION
	_vp.push_input(ev)
	await _frames(2)


func _tap(vw: Window, local: Vector2) -> void:
	await _mouse(vw, local, true)
	await _mouse(vw, local, false)


func _press(vw: Window, b: Button) -> void:
	await _tap(vw, b.get_global_rect().get_center())
	await _frames(3)


## Types `s` through real key events (focus must already be in the text).
func _type(s: String) -> void:
	for i in s.length():
		for pressed in [true, false]:
			var k := InputEventKey.new()
			k.pressed = pressed
			k.unicode = s.unicode_at(i)
			k.keycode = s.unicode_at(i) as Key
			_vp.push_input(k)
		await _frames(2)


## One non-printing key (Enter, Tab, arrows, Escape, Backspace, Home).
func _key(code: Key) -> void:
	for pressed in [true, false]:
		var k := InputEventKey.new()
		k.pressed = pressed
		k.keycode = code
		_vp.push_input(k)
	await _frames(3)


func _finish() -> void:
	print("VE2 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


## The card's rows as text, top to bottom ("" while hidden).
func _rows(ed) -> Array:
	var out: Array = []
	if ed._sg_panel == null or not ed._sg_panel.visible:
		return out
	for b in ed._sg_box.get_children():
		out.append((b as Button).text)
	return out


func _card_visible(ed) -> bool:
	return ed._sg_panel != null and ed._sg_panel.visible


func _text(ed) -> String:
	return ed.text_edit.text


## Replaces the note's text without going through the keyboard (a fresh line to type on), leaving the
## caret at its end and the card closed.
func _reset(ed, t: String) -> void:
	ed.text_edit.text = t
	ed._last_len = t.length()
	ed._sg_hide()
	var last: int = ed.text_edit.get_line_count() - 1
	ed.text_edit.set_caret_line(last)
	ed.text_edit.set_caret_column(ed.text_edit.get_line(last).length())
	ed.text_edit.grab_focus()


func _pure(Ed) -> void:
	# suggest_context -- every literal pinned.
	var c: Dictionary = Ed.suggest_context("see [[Xyl", 9)
	_ok("context: after [[ is note mode with the query", c.get("mode") == "note" and c.get("query") == "Xyl" and c.get("open") == 6 and c.get("qstart") == 6, str(c))
	_ok("context: no [[ is empty", Ed.suggest_context("see Mar", 7).is_empty())
	_ok("context: a finished link is outside", Ed.suggest_context("[[Xyloth]] and", 14).is_empty())
	_ok("context: a caret inside a finished link is inside it", Ed.suggest_context("[[Xyloth]]", 5).get("query") == "Xyl")
	c = Ed.suggest_context("[[Kelvhold#Tra", 14)
	_ok("context: # switches to heading mode", c.get("mode") == "heading" and c.get("note") == "Kelvhold" and c.get("query") == "Tra" and c.get("qstart") == 11, str(c))
	c = Ed.suggest_context("[[#Lo", 5)
	_ok("context: [[# is this note's headings", c.get("mode") == "heading" and c.get("note") == "" and c.get("query") == "Lo", str(c))
	_ok("context: after | is alias mode (nothing to suggest)", Ed.suggest_context("[[Xyloth|the fe", 15).get("mode") == "alias")
	_ok("context: after ^ is not suggested", Ed.suggest_context("[[Xyloth^ab", 11).get("mode") == "alias")
	_ok("context: a second [[ restarts it", Ed.suggest_context("[[a [[b", 7).get("query") == "b")
	# suggest_apply
	var ctx: Dictionary = Ed.suggest_context("[[Xyl]]", 5)
	var r: Dictionary = Ed.suggest_apply("[[Xyl]]", 5, ctx, "Xyloth")
	_ok("apply: replaces the query, keeps the auto-pair, caret after it", r["line"] == "[[Xyloth]]" and r["caret"] == 10, str(r))
	ctx = Ed.suggest_context("[[Xyl", 5)
	r = Ed.suggest_apply("[[Xyl", 5, ctx, "Xyloth")
	_ok("apply: adds ]] when there is none", r["line"] == "[[Xyloth]]" and r["caret"] == 10, str(r))
	ctx = Ed.suggest_context("[[Fer]]", 5)
	r = Ed.suggest_apply("[[Fer]]", 5, ctx, "Aldis", "The Ferryman")
	_ok("apply: an alias match writes Name|Alias", r["line"] == "[[Aldis|The Ferryman]]" and r["caret"] == 22, str(r))
	ctx = Ed.suggest_context("[[Xyl|shown]]", 5)
	r = Ed.suggest_apply("[[Xyl|shown]]", 5, ctx, "Xyloth", "Other")
	_ok("apply: shown text already written is kept, and no second alias", r["line"] == "[[Xyloth|shown]]" and r["caret"] == 16, str(r))
	ctx = Ed.suggest_context("x [[Kelvhold#Tra]] y", 16)
	r = Ed.suggest_apply("x [[Kelvhold#Tra]] y", 16, ctx, "Trade routes")
	_ok("apply: a heading replaces only after #, text around untouched", r["line"] == "x [[Kelvhold#Trade routes]] y" and r["caret"] == 27, str(r))
	ctx = Ed.suggest_context("[[Ma", 4)
	r = Ed.suggest_apply("[[Ma and [[Zed]]", 4, ctx, "Xyloth")
	_ok("apply: a later link on the line is not taken as this link's closer", r["line"] == "[[Xyloth]] and [[Zed]]" and r["caret"] == 10, str(r))
	# link_wrap
	var w: Dictionary = Ed.link_wrap("ab cd", 1, 1)
	_ok("link_wrap: no selection writes [[]] with the caret inside", w["text"] == "a[[]]b cd" and w["from"] == 3 and w["to"] == 3, str(w))
	w = Ed.link_wrap("ab cd ef", 3, 5)
	_ok("link_wrap: a selection becomes the shown text", w["text"] == "ab [[|cd]] ef" and w["from"] == 5, str(w))
	w = Ed.link_wrap("ab\ncd", 0, 5)
	_ok("link_wrap: a multi-line selection is not wrapped", w["text"] == "[[]]ab\ncd" and w["from"] == 2, str(w))


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots"]):
			print("VE2 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE2 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_shots = _arg("--shots", "")
	_touch = "--touch" in OS.get_cmdline_user_args()
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))

	_pure(load("res://shell/markdown_editor.gd"))

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

	_bridge.generate({"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(4)
	var ss: Array = _bridge.settlements()
	_ok("CONTROL: the world has settlements", ss.size() >= 2, str(ss.size()))
	if ss.size() < 2:
		_finish()
		return
	var ent_name := String((ss[0] as Dictionary).get("name", "?"))
	var ent2 := String((ss[1] as Dictionary).get("name", "?"))

	# == The fixture vault =====================================================
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve2-%s-%d" % ["touch" if _touch else "desk", Time.get_ticks_msec()]
	_write(_root + "/Places/Kelvhold.md", "# Kelvhold\n\nA hold.\n\n## Lore\n\nOld.\n\n## Trade routes\n\nSalt.\n\n```\n## Not a heading\n```\n")
	_write(_root + "/Places/Xyloth.md", "# Xyloth\n\nA ferry town.\n\n## History\n\nLong.\n")
	_write(_root + "/People/Aldis.md", "---\naliases: [The Ferryman]\n---\nAldis rows.\n")
	_write(_root + "/Places/Odd.md", "# Odd

## A | B [c] ^d

x
")
	_write(_root + "/A/Twin.md", "Twin on the A road.\n")
	_write(_root + "/B/Twin.md", "Twin on the B road.\n")
	## A note that has the name of a place: the place must not be offered "with no note yet".
	_write(_root + "/Lore/" + ent2 + ".md", "Notes on %s.
" % ent2)
	_write(_root + "/Scratch.md", "# Scratch\n\nword here\n")
	for i in FILLER:
		_write(_root + "/Filler/Note %02d.md" % i, "Filler %d\n" % i)
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	var idx: Dictionary = _bridge.vault_refresh_backlinks(2000)
	_ok("CONTROL: the backlink index builds (aliases need it)", bool(idx.get("ok", false)), str(idx))
	var files_before: int = _bridge.vault_list_files(5000).size()
	_ok("CONTROL: the vault holds far more notes than the card's cap", files_before >= FILLER + 6, str(files_before))

	app.open_vault_browse()
	await _frames(4)
	var vw: VaultWindow = app.vault_window
	var ts: Array = []
	_find_all(vw, "Tree", ts)
	var tree: Tree = ts[0] if not ts.is_empty() else null
	var leaf := _leaf_in(tree.get_root(), "Scratch.md") if tree != null else null
	_ok("CONTROL: Scratch.md is in the tree", leaf != null)
	if leaf == null:
		_finish()
		return
	leaf.select(0)
	await _frames(4)
	var open_btn := _find_button(vw, "Open to edit")
	_ok("CONTROL: Open to edit is offered", open_btn != null)
	if open_btn == null:
		_finish()
		return
	await _press(vw, open_btn)
	var ed = vw._editor
	_ok("the note opens in Write", ed != null and ed.mode() == "write")
	if ed == null:
		_finish()
		return
	var te: TextEdit = ed.text_edit
	var scratch_disk := _disk("Scratch.md")
	var cap: int = ed._sg_limit()
	_ok("the row cap is eight on a pointer layout", cap == 8, str(cap))

	# == 1. [[ auto-pairs and opens a bounded card ============================
	_reset(ed, "")
	await _tap(vw, te.get_global_rect().get_center())
	await _type("[")
	_ok("one [ pairs nothing and opens nothing", _text(ed) == "[" and not _card_visible(ed), _text(ed))
	await _type("[")
	_ok("the second [ writes [[]] with the caret between", _text(ed) == "[[]]" and te.get_caret_column() == 2, "%s col %d" % [_text(ed), te.get_caret_column()])
	var rows := _rows(ed)
	_ok("the card opens at once", _card_visible(ed))
	_ok("... capped at eight rows though the vault holds %d notes" % files_before, rows.size() == 8, str(rows))
	var row0: Button = ed._sg_box.get_child(0) if ed._sg_box.get_child_count() > 0 else null
	var want_px: int = 44 if _touch else 28
	_ok("rows are %d px tall" % want_px, row0 != null and int(row0.size.y) == want_px, str(row0.size if row0 != null else "<none>"))
	_ok("... and no row is under 44 px on touch", not _touch or (row0 != null and row0.size.y >= 44.0))
	var pr: Rect2 = ed._sg_panel.get_global_rect()
	_ok("the card lies inside the window", Rect2(Vector2.ZERO, Vector2(vw.size)).encloses(pr), "%s in %s" % [pr, vw.size])
	_ok("an empty query lists notes before places with no note (Aldis first)", rows.size() > 0 and String(rows[0]).begins_with("Aldis"), str(rows))
	var hl0 := ed._sg_box.get_child(0).get_theme_stylebox("normal") as StyleBoxFlat
	var hl1 := ed._sg_box.get_child(1).get_theme_stylebox("normal") as StyleBoxFlat
	_ok("the highlighted row is drawn (its stylebox has a fill) and the next one is not", hl0 != null and hl1 != null and hl0.bg_color.a > 0.1 and hl1.bg_color.a < 0.01, "%s %s" % [hl0.bg_color if hl0 != null else "?", hl1.bg_color if hl1 != null else "?"])
	_ok("... and the rows are not flat buttons (a flat button draws no stylebox)", not (ed._sg_box.get_child(0) as Button).flat)
	await _shot("1_card.png")

	# == 2. Narrowing ==========================================================
	await _type("Xyl")
	rows = _rows(ed)
	_ok("typing narrows the list; Xyloth comes first", rows.size() >= 1 and rows.size() < 8 and String(rows[0]).begins_with("Xyloth"), str(rows))
	_ok("... and, with no exact match, the new-note row closes the list", rows.size() > 0 and String(rows[rows.size() - 1]).begins_with("+ Link to new note"), str(rows))
	_ok("typing inside leaves the pair", _text(ed) == "[[Xyl]]", _text(ed))

	# arrows
	await _key(KEY_DOWN)
	_ok("Down moves the highlight", ed._sg_index == 1, str(ed._sg_index))
	var moved := ed._sg_box.get_child(1).get_theme_stylebox("normal") as StyleBoxFlat
	_ok("... and the fill moves with it", moved != null and moved.bg_color.a > 0.1 and (ed._sg_box.get_child(0).get_theme_stylebox("normal") as StyleBoxFlat).bg_color.a < 0.01)
	await _key(KEY_UP)
	_ok("Up moves it back", ed._sg_index == 0)
	await _key(KEY_UP)
	_ok("Up from the top wraps to the last row", ed._sg_index == ed._sg_items.size() - 1, str(ed._sg_index))
	await _key(KEY_DOWN)
	await _key(KEY_ENTER)
	_ok("Enter accepts: [[Xyloth]] with the caret after it, card closed", _text(ed) == "[[Xyloth]]" and te.get_caret_column() == 10 and not _card_visible(ed), "%s col %d" % [_text(ed), te.get_caret_column()])
	_ok("... and wrote no file", _disk("Scratch.md") == scratch_disk and _bridge.vault_list_files(5000).size() == files_before)
	te.undo()
	await _frames(3)
	_ok("accepting is ONE undo step: Ctrl+Z gives back what was typed before it", _text(ed).begins_with("[[Xyl"), _text(ed))
	_reset(ed, "")

	# == 3. Tab, alias =========================================================
	await _type("[[Ferry")
	rows = _rows(ed)
	_ok("an alias is found: 'The Ferryman -> Aldis'", rows.size() > 0 and String(rows[0]).find("The Ferryman") >= 0 and String(rows[0]).find("Aldis") >= 0, str(rows))
	await _key(KEY_TAB)
	_ok("Tab accepts, writing Name|Alias", _text(ed) == "[[Aldis|The Ferryman]]" and not _card_visible(ed), _text(ed))
	_reset(ed, "")

	# == 4. Two notes of one name are written as paths ========================
	await _type("[[Twin")
	rows = _rows(ed)
	_ok("two notes called Twin both appear, with their folders", rows.size() >= 2 and String(rows[0]).find("Twin") >= 0 and String(rows[1]).find("Twin") >= 0 and String(rows[0]).find("A") >= 0 and String(rows[1]).find("B") >= 0, str(rows))
	await _key(KEY_ENTER)
	_ok("the link names one of them by path", _text(ed) == "[[A/Twin]]", _text(ed))
	_reset(ed, "")
	await _type("[[Kelvhold")
	await _key(KEY_ENTER)
	_ok("a note with a unique name is written by name", _text(ed) == "[[Kelvhold]]", _text(ed))
	_reset(ed, "")

	# == 4b. What the list leaves out ==========================================
	await _type("[[Kelvhold")
	rows = _rows(ed)
	_ok("an exact match leaves no new-note row", rows.size() >= 1 and str(rows).find("Link to new note") < 0, str(rows))
	_reset(ed, "")
	await _type("[[Scratch")
	rows = _rows(ed)
	_ok("this note is not offered to itself", not rows.has("Scratch") and rows.size() == 1 and String(rows[0]).begins_with("+ Link to new note"), str(rows))
	_reset(ed, "")
	await _type("[[" + ent2)
	rows = _rows(ed)
	var as_place := false
	var as_note := false
	for r in rows:
		as_place = as_place or String(r).begins_with("⌖ " + ent2 + "   ")
		as_note = as_note or String(r) == ent2 + "   · Lore"
	_ok("a place that has a note is offered as the note, not as 'no note yet'", as_note and not as_place, str(rows))
	_reset(ed, "")

	# == 5. Headings ===========================================================
	await _type("[[Kelvhold#")
	rows = _rows(ed)
	_ok("# lists that note's headings (fenced code skipped)", rows.size() == 3 and String(rows[0]).find("Kelvhold") >= 0 and String(rows[1]).find("Lore") >= 0 and String(rows[2]).find("Trade routes") >= 0 and str(rows).find("Not a heading") < 0, str(rows))
	await _type("Tra")
	rows = _rows(ed)
	_ok("... narrowed by what follows", rows.size() == 1 and String(rows[0]) == "# Trade routes", str(rows))
	await _key(KEY_ENTER)
	_ok("accepting writes [[Kelvhold#Trade routes]]", _text(ed) == "[[Kelvhold#Trade routes]]", _text(ed))
	_reset(ed, "")
	await _type("[[Odd#A")
	rows = _rows(ed)
	_ok("a heading holding | [ ] ^ is listed as written", rows.size() == 1 and String(rows[0]) == "# A | B [c] ^d", str(rows))
	await _key(KEY_ENTER)
	_ok("... and linked with those characters made spaces, so the link still parses", _text(ed) == "[[Odd#A B c d]]", _text(ed))
	_reset(ed, "")
	await _type("[[Nonesuch#")
	rows = _rows(ed)
	_ok("headings of a note that does not exist say so, in one disabled line", rows.size() == 1 and String(rows[0]).begins_with("No note called Nonesuch"), str(rows))
	var dis: Button = ed._sg_box.get_child(0) if ed._sg_box.get_child_count() > 0 else null
	_ok("... which is not a choice", dis != null and dis.disabled)
	await _key(KEY_ENTER)
	_ok("Enter on it writes nothing and leaves the card", _text(ed) == "[[Nonesuch#]]" and _card_visible(ed), _text(ed))
	_reset(ed, "# Scratch\n\nsee ")
	await _type("[[#")
	rows = _rows(ed)
	_ok("[[# lists THIS note's headings", rows.size() == 1 and String(rows[0]) == "# Scratch", str(rows))
	_reset(ed, "")

	# == 6. World entities with no note ========================================
	await _type("[[" + ent_name)
	rows = _rows(ed)
	var ent_row := -1
	for i in rows.size():
		if String(rows[i]).begins_with("⌖ " + ent_name + "   "):
			ent_row = i
	_ok("a place with no note is offered, marked and labelled", ent_row >= 0 and String(rows[maxi(ent_row, 0)]).find("no note yet") >= 0, str(rows))
	for i in maxi(ent_row, 0):
		await _key(KEY_DOWN)
	await _key(KEY_ENTER)
	_ok("accepting writes the place's name as the link", _text(ed) == "[[%s]]" % ent_name, _text(ed))
	_ok("... and creates no note", _bridge.vault_list_files(5000).size() == files_before)
	_reset(ed, "")

	# == 7. Nothing matches: offer a link to a new note, never create it ======
	await _type("[[zzqxj")
	rows = _rows(ed)
	_ok("no match leaves exactly the new-note row", rows.size() == 1 and String(rows[0]) == "+ Link to new note “zzqxj”", str(rows))
	await _key(KEY_ENTER)
	_ok("it writes the link and creates nothing", _text(ed) == "[[zzqxj]]" and _bridge.vault_list_files(5000).size() == files_before, _text(ed))
	_reset(ed, "")

	# == 8. Escape closes the card, not the window =============================
	await _type("[[Xyl")
	_ok("CONTROL: the card is open", _card_visible(ed))
	await _key(KEY_ESCAPE)
	_ok("Escape closes the card", not _card_visible(ed))
	_ok("... and not the window, and loses no text", vw.visible and _text(ed) == "[[Xyl]]", "%s visible=%s" % [_text(ed), vw.visible])
	await _type("r")
	_ok("typing on re-opens it", _card_visible(ed) and _text(ed) == "[[Xylr]]", _text(ed))
	await _key(KEY_ESCAPE)
	_ok("CONTROL: the window has a dialog_close_on_escape property", "dialog_close_on_escape" in vw)
	_ok("the window's own Escape is given back once the card is closed", "dialog_close_on_escape" in vw and bool(vw.get("dialog_close_on_escape")))
	_reset(ed, "")

	# == 9. A click on a row accepts ===========================================
	await _type("[[Xyl")
	var click_row: Button = ed._sg_box.get_child(0) if ed._sg_box.get_child_count() > 0 else null
	_ok("CONTROL: a row to click", click_row != null and _card_visible(ed))
	if click_row != null:
		await _tap(vw, click_row.get_global_rect().get_center())
		await _frames(3)
		_ok("a click on a row accepts it", _text(ed) == "[[Xyloth]]" and not _card_visible(ed), _text(ed))
		_ok("... and the text keeps focus", te.has_focus())
	_reset(ed, "")

	# == 10. The caret leaving closes the card =================================
	await _type("[[Xyl")
	await _key(KEY_HOME)
	_ok("moving the caret out of the link closes the card", not _card_visible(ed), str(te.get_caret_column()))
	te.set_caret_column(5)
	await _frames(3)
	_ok("a caret move alone never re-opens it", not _card_visible(ed))
	_reset(ed, "")

	# == 11. Clicking into a finished link does not pop suggestions ============
	_reset(ed, "A finished [[Xyloth]] link.\n")
	var rect := te.get_rect_at_line_column(0, 15)
	await _tap(vw, te.get_global_rect().position + Vector2(rect.position) + Vector2(2, rect.size.y * 0.5))
	_ok("clicking inside an existing link opens nothing", not _card_visible(ed))
	_reset(ed, "")

	# == 12. Pair edits ========================================================
	await _type("[[")
	await _key(KEY_BACKSPACE)
	_ok("Backspace inside an empty [[]] removes the whole pair", _text(ed) == "" and not _card_visible(ed), _text(ed))
	await _type("[[")
	await _type("[")
	_ok("a third [ does not pair again", _text(ed) == "[[[]]", _text(ed))
	_reset(ed, "[[")
	await _type("[")
	_ok("a third [ after an unpaired [[ pairs nothing either", _text(ed) == "[[[", _text(ed))
	_reset(ed, "")
	te.insert_text_at_caret("[[")
	await _frames(3)
	_ok("pasting [[ pairs nothing (a multi-character insert is not typing)", _text(ed) == "[[", _text(ed))
	_reset(ed, "")
	await _type("[[]]")
	_ok("CONTROL: typed text over an existing pair is just text", _text(ed).begins_with("[[]]"), _text(ed))
	_reset(ed, "")

	# == 13. The link button ===================================================
	await _press(vw, ed.buttons["link"])
	_ok("the button writes [[]] with the caret between and opens the card", _text(ed) == "[[]]" and te.get_caret_column() == 2 and _card_visible(ed), "%s %d" % [_text(ed), te.get_caret_column()])
	await _key(KEY_ESCAPE)
	_reset(ed, "make this word shown")
	te.select(0, 10, 0, 14)
	await _press(vw, ed.buttons["link"])
	_ok("with a selection it writes [[|selected]]", _text(ed) == "make this [[|word]] shown", _text(ed))
	_ok("... caret before the |, card open", te.get_caret_column() == 12 and _card_visible(ed), str(te.get_caret_column()))
	await _type("Xyl")
	await _key(KEY_ENTER)
	_ok("... accepting keeps the shown text: [[Xyloth|word]]", _text(ed) == "make this [[Xyloth|word]] shown", _text(ed))
	_reset(ed, "")

	# == 14. Preview has no card ===============================================
	await _type("[[Xyl")
	await _press(vw, ed.buttons["preview"])
	_ok("switching to Preview closes the card", not _card_visible(ed))
	await _press(vw, ed.buttons["write"])
	_reset(ed, "")

	# == 15. Nothing was ever written ==========================================
	_ok("Scratch.md on disk is untouched by everything above", _disk("Scratch.md") == scratch_disk)
	_ok("no note was created", _bridge.vault_list_files(5000).size() == files_before)
	_ok("the Twin notes are untouched", _disk("A/Twin.md") == "Twin on the A road.\n" and _disk("B/Twin.md") == "Twin on the B road.\n")
	await _shot("2_end.png")
	_finish()
