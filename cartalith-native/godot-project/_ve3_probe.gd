extends Node
## VE-3 of `VAULT_EDITOR_PROPOSAL.md` s.7: Write colouring, Split, smart keys, find and replace, word
## count. Bar: *caret and colours align at 3 widths; Split appears exactly from 760 dp.* Drives the real
## app, the real vault window and a real vault folder with real clicks and typed keys; every file
## assertion re-reads the disk.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve3_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch   (the phone leg, 393 x 852 dp: Split is never offered, the bar
##                                          fits and its buttons meet the 44 px floor; `--force-touch` is
##                                          read by `dcc_shell.gd`, which then builds the phone shell)
##
## **Windowed** (pixels and input routing). `--touch` makes pointer events carry the emulated-mouse
## device id a finger's do. Run under a wiped user directory (`APPDATA=<empty dir>`): it deletes
## `user://vault.json` and binds a throw-away vault. Exit 0 = every check held, 1 = a check failed,
## 2 = could not run.
##
## ## What each leg proves
##
## - **Pure.** Every literal in `markdown_highlighter.gd` and the editor's static functions is pinned
##   with exact values (role arrays, line kinds, Enter / Tab results, the pair rules, find and replace,
##   the 759 / 760 split rule), so a changed constant fails here whatever the UI does.
## - **Colour reaches the screen, and lines up.** A capture of the real editor is read: the drawn ink of a
##   coloured run starts at the x the `TextEdit` itself reports for that column, at three window widths
##   (one of them wrapping the line). Drawn colours are pinned against the canvas literals (dark) and
##   measured against the ground pixel (both palettes) -- never against `DccTheme.c()`, which moves with
##   the thing under test.
## - **Split** is offered at 760 and not at 759, both panes show side by side, Preview follows typing,
##   and a squeeze below 760 falls back to Write and returns.
## - **Smart keys, find and replace, the footer** are driven by real key events and real button taps.
##   Replace all must be ONE undo step; nothing the probe types may be lost, and no key but Ctrl+S
##   writes the file.
##
## A pointer note carried from `_ve0_probe.gd`: a SubViewport routes motion only after
## `NOTIFICATION_VP_MOUSE_ENTER`. And a script error inside an `await` hangs a probe, so every wait here
## is guarded and a missing control ends the run with the failure on record.

const H := preload("res://shell/markdown_highlighter.gd")
const Ed := preload("res://shell/markdown_editor.gd")

## Canvas literals (`dcc_theme.gd`, the dark and light token sets): the ground a Write field is filled
## with (`sunken`) and the tokens a role is drawn in when they already meet their floor.
const DARK_GROUND := "#191c1e"
const LIGHT_GROUND := "#eceae4"

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
		print("VE3  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE3  !!  %s   %s" % [label, detail])


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
	## Control rects are in the window's content units; input is in its pixels (the phone shell scales its
	## content, so the two differ there; on the desktop the factor is 1).
	var p := Vector2(vw.position) + local * vw.content_scale_factor
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


## Types `s` through real key events (focus must already be where the text should go).
func _type(s: String) -> void:
	for i in s.length():
		for pressed in [true, false]:
			var k := InputEventKey.new()
			k.pressed = pressed
			k.unicode = s.unicode_at(i)
			k.keycode = s.unicode_at(i) as Key
			_vp.push_input(k)
		await _frames(2)


## One non-printing key, optionally with Ctrl and / or Shift held.
func _key(code: Key, ctrl: bool = false, shift: bool = false) -> void:
	for pressed in [true, false]:
		var k := InputEventKey.new()
		k.pressed = pressed
		k.keycode = code
		k.physical_keycode = code
		k.ctrl_pressed = ctrl
		k.shift_pressed = shift
		_vp.push_input(k)
	await _frames(3)


func _finish() -> void:
	print("VE3 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _text(ed) -> String:
	return ed.text_edit.text


## Replaces the note's text without the keyboard (a fresh line to type on, and a fresh undo history),
## leaving the caret at its end and every card and bar closed.
func _reset(ed, t: String) -> void:
	ed._find_close(false)
	ed.text_edit.text = t
	ed._last_len = t.length()
	ed._sg_hide()
	var last: int = ed.text_edit.get_line_count() - 1
	ed.text_edit.set_caret_line(last)
	ed.text_edit.set_caret_column(ed.text_edit.get_line(last).length())
	ed.text_edit.grab_focus()


## The colour a `get_line_syntax_highlighting` dictionary gives column `col`.
func _color_at(d: Dictionary, col: int) -> Color:
	var best := -1
	for k in d.keys():
		if int(k) <= col and int(k) > best:
			best = int(k)
	return (d[best] as Dictionary)["color"] if best >= 0 else Color(0, 0, 0, 0)


## The editor's highlighter as a bare Variant, so its script methods (`color_of`) can be called.
func _hlr(te: TextEdit):
	return te.get("syntax_highlighter")


## The y band `[top, bottom)` (TextEdit-local) the text of `line` is drawn in, found with the editor's own
## hit test -- `get_rect_at_line_column` is not used (see `_align_leg`).
func _line_band(te: TextEdit, line: int) -> Vector2i:
	var top := -1
	var bot := -1
	for y in int(te.size.y):
		if te.get_line_column_at_pos(Vector2i(30, y), false, false).y == line:
			if top < 0:
				top = y
			bot = y + 1
	return Vector2i(top, bot)


func _same(a: Color, b: Color) -> bool:
	return absf(a.r - b.r) < 0.004 and absf(a.g - b.g) < 0.004 and absf(a.b - b.b) < 0.004


func _roles(text: String, kind: int = 0, links: Array = [], matches: Array = []) -> Array:
	return Array(H.line_roles(text, kind, links, matches))


# == Pure ======================================================================

func _pure() -> void:
	# -- line kinds
	_ok("kinds: front matter then a fence",
		Array(H.line_kinds(PackedStringArray(["---", "a: 1", "---", "x", "```", "code", "```", "y"]))) == [1, 1, 1, 0, 2, 3, 2, 0])
	_ok("kinds: an unclosed --- is not front matter", Array(H.line_kinds(PackedStringArray(["---", "x"]))) == [0, 0])
	_ok("kinds: --- below line 0 is not front matter", Array(H.line_kinds(PackedStringArray(["x", "---", "y", "---"]))) == [0, 0, 0, 0])
	_ok("kinds: an unclosed fence runs to the end", Array(H.line_kinds(PackedStringArray(["```", "a", "b"]))) == [2, 3, 3])
	_ok("kinds: ~~~ fences too", Array(H.line_kinds(PackedStringArray(["~~~", "a", "~~~", "b"]))) == [2, 3, 2, 0])
	# -- roles (0 body, 1 marker, 2 heading, 3 emph, 4 code, 5 link, 6 tag, 7 highlight, 8 strike, 9 quote, 10 front, 11 task, 12 match)
	_ok("roles: # Hi", _roles("# Hi") == [1, 1, 2, 2], str(_roles("# Hi")))
	_ok("roles: ###### six hashes are a heading, seven are not", _roles("###### a") == [1, 1, 1, 1, 1, 1, 1, 2] and _roles("####### a") == [0, 0, 0, 0, 0, 0, 0, 0, 0])
	_ok("roles: **b**", _roles("**b**") == [1, 1, 3, 1, 1], str(_roles("**b**")))
	_ok("roles: __b__", _roles("__b__") == [1, 1, 3, 1, 1])
	_ok("roles: *i* and _i_", _roles("*i*") == [1, 3, 1] and _roles("_i_") == [1, 3, 1])
	_ok("roles: snake_case_name is not italic", _roles("snake_case_name") == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
	_ok("roles: ==m==", _roles("==m==") == [1, 1, 7, 1, 1])
	_ok("roles: ~~s~~", _roles("~~s~~") == [1, 1, 8, 1, 1])
	_ok("roles: `c`", _roles("`c`") == [1, 4, 1])
	_ok("roles: code overrides bold inside it", _roles("`**x**`") == [1, 4, 4, 4, 4, 4, 1], str(_roles("`**x**`")))
	_ok("roles: #tag, and a lone # is not a tag", _roles("#tag x") == [6, 6, 6, 6, 0, 0] and _roles("a # b") == [0, 0, 0, 0, 0])
	_ok("roles: #2024 is not a tag (all digits)", _roles("#2024") == [0, 0, 0, 0, 0])
	_ok("roles: > q", _roles("> q") == [1, 1, 9], str(_roles("> q")))
	_ok("roles: - [ ] t", _roles("- [ ] t") == [1, 0, 11, 11, 11, 0, 0], str(_roles("- [ ] t")))
	_ok("roles: 1. a", _roles("1. a") == [1, 1, 0, 0], str(_roles("1. a")))
	_ok("roles: --- alone is a rule", _roles("---") == [1, 1, 1])
	_ok("roles: a heading's bold keeps the heading colour, markers dim", _roles("# **b**") == [1, 1, 1, 1, 2, 1, 1], str(_roles("# **b**")))
	_ok("roles: [!note] callout", _roles("> [!note] x") == [1, 1, 7, 7, 7, 7, 7, 7, 7, 9, 9], str(_roles("> [!note] x")))
	_ok("roles: a wiki link and its brackets",
		_roles("a [[K]] b", 0, [{"start": 2, "end": 7, "syntax": "wiki"}]) == [0, 0, 1, 1, 5, 1, 1, 0, 0])
	_ok("roles: an embed has three bracket characters", _roles("![[K]]", 0, [{"start": 0, "end": 6, "syntax": "embed"}]) == [1, 1, 1, 5, 1, 1])
	_ok("roles: a markdown link is all link", _roles("[a](b)", 0, [{"start": 0, "end": 6, "syntax": "markdown"}]) == [5, 5, 5, 5, 5, 5])
	_ok("roles: find hits override", _roles("abc abc", 0, [], [Vector2i(4, 7)]) == [0, 0, 0, 0, 12, 12, 12])
	_ok("roles: fenced code is all code, its fence a marker", _roles("x **y**", 3) == [4, 4, 4, 4, 4, 4, 4] and _roles("```js", 2) == [1, 1, 1, 1, 1])
	_ok("roles: front matter dim, its rule a marker", _roles("a: 1", 1) == [10, 10, 10, 10] and _roles("---", 1) == [1, 1, 1])
	_ok("roles: an empty line is empty", _roles("") == [])
	# -- contrast
	_ok("contrast: black on white is 21", absf(H.contrast(Color.BLACK, Color.WHITE) - 21.0) < 0.001)
	_ok("contrast: the dark accent on the dark ground is 7.75", absf(H.contrast(Color(DarkTok.ACCENT), Color(DARK_GROUND)) - 7.75) < 0.01, str(H.contrast(Color(DarkTok.ACCENT), Color(DARK_GROUND))))
	_ok("contrast: the light accent on the light ground is 3.93 (why a floor is needed)", absf(H.contrast(Color("#a4650f"), Color(LIGHT_GROUND)) - 3.93) < 0.01, str(H.contrast(Color("#a4650f"), Color(LIGHT_GROUND))))
	_ok("floors: 4.5 for text, 3.0 for markers", H.TEXT_FLOOR == 4.5 and H.MARKER_FLOOR == 3.0)
	# -- Enter
	_ok("enter: a bullet repeats", Ed.smart_enter("- a", 3) == {"kind": "continue", "insert": "\n- "}, str(Ed.smart_enter("- a", 3)))
	_ok("enter: * and + bullets repeat", Ed.smart_enter("* a", 3)["insert"] == "\n* " and Ed.smart_enter("+ a", 3)["insert"] == "\n+ ")
	_ok("enter: a number counts up", Ed.smart_enter("1. a", 4)["insert"] == "\n2. " and Ed.smart_enter("9) a", 4)["insert"] == "\n10) ")
	_ok("enter: a task comes back unticked", Ed.smart_enter("- [x] a", 7)["insert"] == "\n- [ ] " and Ed.smart_enter("- [X] a", 7)["insert"] == "\n- [ ] " and Ed.smart_enter("- [ ] a", 7)["insert"] == "\n- [ ] ")
	_ok("enter: a quote repeats, nested too", Ed.smart_enter("> a", 3)["insert"] == "\n> " and Ed.smart_enter("> > a", 5)["insert"] == "\n> > ")
	_ok("enter: a bullet in a quote repeats both", Ed.smart_enter("> - a", 5)["insert"] == "\n> - ")
	_ok("enter: indentation is kept", Ed.smart_enter("\t- a", 4)["insert"] == "\n\t- " and Ed.smart_enter("  1. a", 6)["insert"] == "\n  2. ")
	_ok("enter: an empty top-level item ends the list", Ed.smart_enter("- ", 2) == {"kind": "end", "line": ""} and Ed.smart_enter("1. ", 3) == {"kind": "end", "line": ""} and Ed.smart_enter("- [ ] ", 6) == {"kind": "end", "line": ""})
	_ok("enter: an empty indented item steps out and keeps its marker", Ed.smart_enter("\t- ", 3) == {"kind": "end", "line": "- "} and Ed.smart_enter("    - ", 6) == {"kind": "end", "line": "- "})
	_ok("enter: an empty quote ends the quote; an empty item in a quote keeps it", Ed.smart_enter("> ", 2) == {"kind": "end", "line": ""} and Ed.smart_enter("> - ", 4) == {"kind": "end", "line": "> "})
	_ok("enter: plain lines and rules are left to the editor", Ed.smart_enter("hello", 5).is_empty() and Ed.smart_enter("", 0).is_empty() and Ed.smart_enter("---", 3).is_empty() and Ed.smart_enter("**b**", 5).is_empty() and Ed.smart_enter("    code", 8).is_empty())
	_ok("enter: a caret inside the marker is left to the editor", Ed.smart_enter("- a", 1).is_empty() and Ed.smart_enter("1. a", 2).is_empty())
	_ok("enter: mid-line splits and the rest follows the marker", Ed.smart_enter("- abcd", 4)["insert"] == "\n- ")
	_ok("enter: an item with text after the marker but caret at its start is not 'empty'", Ed.smart_enter("- a", 2)["kind"] == "continue")
	_ok("list line: items yes, prose no", Ed.is_list_line("- a") and Ed.is_list_line("\t3. a") and Ed.is_list_line("> - a") and not Ed.is_list_line("> a") and not Ed.is_list_line("a"))
	_ok("outdent: a tab goes; spaces up to four; none stays", Ed.outdent_line("\t- a") == "- a" and Ed.outdent_line("      - a") == "  - a" and Ed.outdent_line("  - a") == "- a" and Ed.outdent_line("- a") == "- a")
	# -- bold pairing
	_ok("pair: ** at an empty line pairs", Ed.should_pair_bold("**", 2))
	_ok("pair: a third * does not", not Ed.should_pair_bold("***", 3))
	_ok("pair: not over an existing **", not Ed.should_pair_bold("****", 2))
	_ok("pair: a closer (an odd opener earlier) does not", not Ed.should_pair_bold("**bold**", 8))
	_ok("pair: a second opener on a line does", Ed.should_pair_bold("**a** **", 8))
	_ok("pair: one star is nothing", not Ed.should_pair_bold("*", 1))
	_ok("third star: ***|** gives ***", Ed.unpairs_third_star("*****", 3) and not Ed.unpairs_third_star("****", 3) and Ed.unpairs_third_star("a*****", 4) and not Ed.unpairs_third_star("******", 4))
	_ok("overtype: first closer star", Ed.overtypes_star("**bold***", 7), "")
	_ok("overtype: second closer star", Ed.overtypes_star("**bold***", 8))
	_ok("overtype: not in plain text", not Ed.overtypes_star("a**", 2) and not Ed.overtypes_star("a*b*", 2))
	# -- find and replace
	_ok("find: case-insensitive hits, in order", Ed.find_matches("Cat cat CAT", "cat", false) == [Vector2i(0, 3), Vector2i(4, 7), Vector2i(8, 11)])
	_ok("find: match case keeps one", Ed.find_matches("Cat cat CAT", "cat", true) == [Vector2i(4, 7)])
	_ok("find: hits do not overlap", Ed.find_matches("aaaa", "aa", true) == [Vector2i(0, 2), Vector2i(2, 4)])
	_ok("find: an empty query finds nothing", Ed.find_matches("abc", "", false).is_empty())
	_ok("pick: forward from a position, wrapping", Ed.pick_match([Vector2i(0, 1), Vector2i(5, 6)], 1, true) == 1 and Ed.pick_match([Vector2i(0, 1), Vector2i(5, 6)], 6, true) == 0 and Ed.pick_match([], 0, true) == -1)
	_ok("pick: backward, wrapping", Ed.pick_match([Vector2i(0, 1), Vector2i(5, 6)], 5, false) == 0 and Ed.pick_match([Vector2i(0, 1), Vector2i(5, 6)], 0, false) == 1)
	_ok("replace all: every hit, and the count", Ed.replace_all("a cat, a Cat", "cat", "dog", false) == {"text": "a dog, a dog", "count": 2})
	_ok("replace all: match case", Ed.replace_all("a cat, a Cat", "cat", "dog", true) == {"text": "a dog, a Cat", "count": 1})
	_ok("replace all: a replacement holding the query is not searched again", Ed.replace_all("a a", "a", "aa", true) == {"text": "aa aa", "count": 2})
	_ok("replace all: nothing found changes nothing", Ed.replace_all("abc", "x", "y", true) == {"text": "abc", "count": 0})
	_ok("replace all: an empty replacement deletes", Ed.replace_all("a-b-c", "-", "", true) == {"text": "abc", "count": 2})
	# -- split rule
	_ok("split: offered at exactly 760 and not at 759.9", Ed.split_offered(false, 760.0) and not Ed.split_offered(false, 759.9) and Ed.split_offered(false, 1200.0))
	_ok("split: never on the phone", not Ed.split_offered(true, 2000.0))
	_ok("split: the constant is 760", Ed.SPLIT_MIN_DP == 760)


## The dark tokens a role is drawn in untouched (they already meet their floor), as canvas literals.
class DarkTok:
	const ACCENT := "#e0a34a"
	const MARKER := "#8d9296"
	const LINK := "#6a9bc4"
	const BODY := "#c8cbcd"
	const CODE := "#c96a5a"


# == Pixels ====================================================================

## `_vp`'s current frame as an image.
func _grab() -> Image:
	await _frames(4)
	await RenderingServer.frame_post_draw
	return _vp.get_texture().get_image()


func _dist(a: Color, b: Color) -> float:
	return maxf(absf(a.r - b.r), maxf(absf(a.g - b.g), absf(a.b - b.b)))


## Leftmost x in `[x0, x1)` of rows `[y0, y1)` whose pixel is within `tol` of `ink`; -1 when none.
func _leftmost(img: Image, x0: int, x1: int, y0: int, y1: int, ink: Color, tol: float) -> int:
	var best := -1
	for y in range(maxi(y0, 0), mini(y1, img.get_height())):
		for x in range(maxi(x0, 0), mini(x1, img.get_width())):
			if _dist(img.get_pixel(x, y), ink) <= tol:
				if best < 0 or x < best:
					best = x
				break
	return best


## How many pixels in the box are clearly ink (far from the ground) and closer to `ink` than to `other`:
## a thin glyph's anti-aliased strokes never reach the exact colour, but they lean toward it.
func _lean_count(img: Image, x0: int, x1: int, y0: int, y1: int, ink: Color, other: Color, ground: Color) -> int:
	var n := 0
	for y in range(maxi(y0, 0), mini(y1, img.get_height())):
		for x in range(maxi(x0, 0), mini(x1, img.get_width())):
			var p := img.get_pixel(x, y)
			if _dist(p, ground) > 0.25 and _dist(p, ink) < _dist(p, other):
				n += 1
	return n


## The ink pixel closest to `ink` anywhere in the box (smallest distance), to prove a colour reaches
## the screen at all.
func _nearest(img: Image, x0: int, x1: int, y0: int, y1: int, ink: Color) -> float:
	var best := 9.0
	for y in range(maxi(y0, 0), mini(y1, img.get_height())):
		for x in range(maxi(x0, 0), mini(x1, img.get_width())):
			best = minf(best, _dist(img.get_pixel(x, y), ink))
	return best


# == The run ====================================================================

func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots", "--force-touch"]):
			print("VE3 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE3 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE3 WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_shots = _arg("--shots", "")
	_touch = "--touch" in OS.get_cmdline_user_args() or "--force-touch" in OS.get_cmdline_user_args()
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))

	_pure()

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

	# == The fixture vault =====================================================
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve3-%s-%d" % ["touch" if _touch else "desk", Time.get_ticks_msec()]
	var scratch := "# Scratch\n\nword here\n"
	_write(_root + "/Scratch.md", scratch)
	_write(_root + "/Kelvhold.md", "# Kelvhold\n\nA hold.\n")
	_write(_root + "/Xyloth.md", "# Xyloth\n\nA ferry town.\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_bridge.vault_refresh_backlinks(2000)
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
	print("VE3 info: open button rect %s, window %s size %s, phone=%s, pane=%s" % [open_btn.get_global_rect(), vw.position, vw.size, vw._phone, str(vw.get("_browse_phone_pane"))])
	await _press(vw, open_btn)
	await _frames(4)
	var ed = vw._editor
	_ok("the note opens in Write", ed != null and ed.mode() == "write")
	if ed == null:
		_finish()
		return
	var te: TextEdit = ed.text_edit
	var phone: bool = ed._phone
	print("VE3 info: viewport %s, phone=%s, touch=%s, editor width %d" % [_vp.size, phone, _touch, int(ed.size.x)])
	await _shot("0_open.png")

	# == 1. The highlighter is installed and reads the real text ==================
	_ok("the source has the Markdown highlighter", te.syntax_highlighter != null and te.syntax_highlighter.get_script() == H)
	var fx := "---\ntitle: x\n---\n# Heading one\nplain **bold** and *it* and ==mark== ~~gone~~ #tag\na [[Kelvhold]] link and [web](https://e.x) and `code`\n> quote text\n- [ ] task item\n```\n# not a heading\n```\n"
	_reset(ed, fx)
	await _frames(3)
	var hl = _hlr(te)
	var d_front: Dictionary = hl._get_line_syntax_highlighting(1)
	var d_head: Dictionary = hl._get_line_syntax_highlighting(3)
	var d_body: Dictionary = hl._get_line_syntax_highlighting(4)
	var d_link: Dictionary = hl._get_line_syntax_highlighting(5)
	var d_fence: Dictionary = hl._get_line_syntax_highlighting(9)
	var dark := DccTheme.is_dark()
	_ok("CONTROL: the probe starts in the dark palette (its literals are the dark tokens)", dark)
	_ok("the heading marker is dim and the heading text is not", not _same(_color_at(d_head, 0), _color_at(d_head, 3)))
	_ok("a heading's text is drawn in the accent (dark literal)", not dark or _same(_color_at(d_head, 3), Color(DarkTok.ACCENT)), str(_color_at(d_head, 3)))
	_ok("a marker is drawn in text_dim (dark literal)", not dark or _same(_color_at(d_head, 0), Color(DarkTok.MARKER)), str(_color_at(d_head, 0)))
	_ok("body text is drawn in the body ink (dark literal)", not dark or _same(_color_at(d_body, 0), Color(DarkTok.BODY)), str(_color_at(d_body, 0)))
	_ok("bold text differs from body text and its ** from both", not _same(_color_at(d_body, 8), _color_at(d_body, 0)) and not _same(_color_at(d_body, 6), _color_at(d_body, 8)))
	_ok("a link is drawn in the link colour (dark literal), its [[ dimmer", not dark or (_same(_color_at(d_link, 5), Color(DarkTok.LINK)) and _same(_color_at(d_link, 2), Color(DarkTok.MARKER))), "%s %s" % [_color_at(d_link, 5), _color_at(d_link, 2)])
	_ok("... and so is the [web](...) link, from the crate's span", not dark or _same(_color_at(d_link, 25), Color(DarkTok.LINK)), str(_color_at(d_link, 25)))
	_ok("inline code is drawn in the code colour (dark literal)", not dark or _same(_color_at(d_link, 48), Color(DarkTok.CODE)), str(_color_at(d_link, 48)))
	_ok("front matter is dim", not _same(_color_at(d_front, 0), _color_at(d_body, 0)))
	_ok("a # inside a fence is code, not a heading", _same(_color_at(d_fence, 3), _color_at(hl._get_line_syntax_highlighting(9), 0)) and not _same(_color_at(d_fence, 3), _color_at(d_head, 3)))
	# a fence opened later recolours the lines below it, without those lines being edited
	## (Pixels are mapped in content units; the phone shell scales its content, so this and the
	## alignment leg are desktop and tablet legs.)
	if not phone:
		_reset(ed, "a\n# h\nb\n")
		await _frames(2)
		await _tap(vw, te.get_global_rect().get_center())
		te.set_caret_line(0)
		te.set_caret_column(1)
		var img0 := await _grab()
		var org0 := Vector2(vw.position) + te.get_global_rect().position
		var row1 := _line_band(te, 1)
		var head_ink: Color = _hlr(te).color_of(H.HEADING)
		var code_ink: Color = _hlr(te).color_of(H.CODE)
		var h_before := _nearest(img0, int(org0.x) + 6, int(org0.x) + 120, int(org0.y) + row1.x + 3, int(org0.y) + row1.y - 3, head_ink)
		_ok("CONTROL: before the fence, '# h' is drawn in the heading ink (%.2f)" % h_before, h_before < 0.2)
		await _key(KEY_ENTER)
		await _type("```")
		## The caret and the focus border are drawn in the accent, the heading's own colour: the caret is
		## parked on the last line and the scan starts inside the border, away from both.
		te.set_caret_line(te.get_line_count() - 1)
		te.set_caret_column(0)
		var img1 := await _grab()
		var row2 := _line_band(te, 2)
		var h_after := _nearest(img1, int(org0.x) + 6, int(org0.x) + 120, int(org0.y) + row2.x + 3, int(org0.y) + row2.y - 3, head_ink)
		var c_after := _nearest(img1, int(org0.x) + 6, int(org0.x) + 120, int(org0.y) + row2.x + 3, int(org0.y) + row2.y - 3, code_ink)
		_ok("typing ``` recolours the line below it, which is not edited, as code (heading %.2f, code %.2f)" % [h_after, c_after], te.get_line(2) == "# h" and h_after > 0.2 and c_after < 0.2, te.text.replace("\n", "|"))
		if _shots != "":
			img1.get_region(Rect2i(int(org0.x), int(org0.y), 300, 140)).save_png(_shots + "/fence.png")

	# == 2. Contrast, dark ========================================================
	await _contrast_leg(ed, "dark", Color(DARK_GROUND))

	# == 3. Caret and colours align at three widths ==============================
	## (Pixels are mapped in content units; the phone shell scales its content, so the phone leg has no
	## pixel legs -- its window cannot change width, which is what the leg varies.)
	if not phone:
		await _align_leg(vw, ed, te)

	# == 4. Split ================================================================
	await _split_leg(vw, ed, te, phone)

	# == 5. Smart keys ===========================================================
	if not phone:
		await _smart_leg(vw, ed, te, scratch)

	# == 6. Find and replace =====================================================
	await _find_leg(vw, ed, te, phone)

	# == 7. Footer ===============================================================
	await _footer_leg(vw, ed, te)

	# == 8. Nothing but the user's own save ever wrote ===============================
	_ok("Scratch.md on disk is untouched by everything above (no autosave, no key but Ctrl+S writes)", _disk("Scratch.md") == scratch, _disk("Scratch.md"))
	_ok("no note was created or removed", _bridge.vault_list_files(5000).size() == 3)

	# == 9. Ctrl+S: the user's own save ============================================
	_reset(ed, "# Scratch\n\nword here\n\nsaved by ctrl+s\n")
	await _tap(vw, te.get_global_rect().get_center())
	await _key(KEY_S, true)
	await _frames(4)
	_ok("Ctrl+S saves through the window's own hash-guarded write", _disk("Scratch.md") == "# Scratch\n\nword here\n\nsaved by ctrl+s\n", _disk("Scratch.md"))
	_ok("... and the editor reads clean afterwards", not ed.is_dirty())

	# == 10. Light palette ===========================================================
	## Pixel reads of a standalone editor are in `_vp`'s units; the phone shell scales its own content, so
	## the light palette is read on the desktop and tablet legs only.
	if not phone:
		await _light_leg(vw)
	await _shot("9_end.png")
	_finish()


# == Contrast ====================================================================

## Every role's colour against the ground: the floors of `markdown_highlighter.gd`, measured from the
## drawn ground pixel and not from a token.
func _contrast_leg(ed, label: String, ground_literal: Color) -> void:
	var te: TextEdit = ed.text_edit
	_reset(ed, "x")
	var img := await _grab()
	var vwin: Window = ed.get_window()
	var p := Vector2(vwin.position) + (te.get_global_rect().position + Vector2(te.size.x - 24, te.size.y - 12)) * vwin.content_scale_factor
	var ground := img.get_pixel(int(p.x), int(p.y))
	_ok("%s: the field ground is the canvas literal" % label, _dist(ground, ground_literal) < 0.012, "%s vs %s" % [ground, ground_literal])
	var worst_text := 99.0
	var worst_marker := 99.0
	var rows: Array = []
	for role in [H.BODY, H.HEADING, H.EMPH, H.CODE, H.LINK, H.TAG, H.HIGHLIGHT, H.STRIKE, H.QUOTE, H.FRONT, H.TASK, H.MATCH]:
		var c: Color = _hlr(te).color_of(role)
		var r: float = H.contrast(c, ground_literal)
		rows.append("%d:%.2f" % [role, r])
		worst_text = minf(worst_text, r)
	var mr: float = H.contrast(_hlr(te).color_of(H.MARKER), ground_literal)
	worst_marker = mr
	print("VE3 info: %s contrast vs %s -- %s marker:%.2f" % [label, ground_literal.to_html(false), " ".join(rows), mr])
	_ok("%s: every text role is at least 4.5:1 on the ground (lowest %.2f)" % [label, worst_text], worst_text >= 4.5, str(rows))
	_ok("%s: the dimmed marker is at least 3:1 (%.2f)" % [label, worst_marker], worst_marker >= 3.0)
	_ok("%s: ... and still dimmer than the body text" % label, H.contrast(_hlr(te).color_of(H.MARKER), ground_literal) < H.contrast(_hlr(te).color_of(H.BODY), ground_literal))


# == Alignment ===================================================================

func _align_leg(vw: Window, ed, te: TextEdit) -> void:
	## The oracle is the editor's own hit test (`get_line_column_at_pos`, what a click uses to place the
	## caret), run on the pixels that carry the highlight colour. `get_rect_at_line_column` is NOT used:
	## on 4.7.2 it answers for the grapheme BEFORE the one asked about (column 1 reports column 0's x), so
	## it cannot say where a coloured run starts. The run `WWWW` sits at columns 132..135; a click on the
	## left half of a glyph reports its own column and on the right half the next one, so the ink's
	## columns must start at exactly 132 and end at 135 or 136. A colour boundary one column early or
	## late moves the minimum to 131 or 133.
	var prefix := "MMMM ".repeat(26)
	var line0 := prefix + "==WWWW== tail"
	var want_col := prefix.length() + 2
	var widths := [1280, 1000, 800] if not ed._phone else [vw.size.x]
	for w in widths:
		vw.size = Vector2i(w, vw.size.y)
		await _frames(8)
		_reset(ed, line0 + "
second line")
		te.set_caret_line(1)
		te.set_caret_column(0)
		te.release_focus()
		await _frames(3)
		var img := await _grab()
		var org := Vector2(vw.position) + te.get_global_rect().position
		var hi: Color = _hlr(te).color_of(H.HIGHLIGHT)
		var cols := {}
		var rows := {}
		for y in range(int(org.y), mini(int(org.y + te.size.y), img.get_height())):
			for x in range(int(org.x), mini(int(org.x + te.size.x), img.get_width())):
				if _dist(img.get_pixel(x, y), hi) <= 0.14:
					var lc := te.get_line_column_at_pos(Vector2i(x - int(org.x), y - int(org.y)), false, false)
					if lc.y == 0:
						cols[lc.x] = true
						rows[y] = true
		var lo := 99999
		var hiC := -1
		for c in cols.keys():
			lo = mini(lo, int(c))
			hiC = maxi(hiC, int(c))
		var wrapped_rows := te.get_line_wrap_count(0)
		print("VE3 info: width %d (editor %d): highlight ink maps to columns %d..%d (want %d..%d), line wraps to %d rows" % [w, int(ed.size.x), lo, hiC, want_col, want_col + 3, wrapped_rows + 1])
		_ok("width %d: the highlighted run's ink begins at the run's first column" % w, lo == want_col, "%d..%d" % [lo, hiC])
		_ok("width %d: ... and ends at its last (or the next, on the glyph's right half)" % w, hiC == want_col + 3 or hiC == want_col + 4, "%d..%d" % [lo, hiC])
		_ok("width %d: ... and the colour sits on a single row of the wrapped line" % w, rows.size() > 0 and rows.size() <= 24)
		if w == 800 or ed._phone:
			_ok("width %d: the long line wrapped, so the check crossed a wrap" % w, wrapped_rows >= 1)
	await _shot("3_align.png")


# == Split =======================================================================

func _fit_width(vw: Window, ed, target: int) -> int:
	for _i in 8:
		var cur := int(ed.size.x)
		if cur == target:
			break
		vw.size = Vector2i(maxi(vw.size.x + (target - cur), 400), vw.size.y)
		await _frames(8)
	return int(ed.size.x)


func _split_leg(vw: Window, ed, te: TextEdit, phone: bool) -> void:
	if phone:
		_ok("phone: Split is not offered", not (ed.buttons["split"] as Button).visible)
		ed.set_mode("split")
		_ok("phone: asking for Split gives Write", ed.mode() == "write")
		return
	vw.size = Vector2i(1500, vw.size.y)
	await _frames(8)
	var w759 := await _fit_width(vw, ed, 759)
	var split_btn: Button = ed.buttons["split"]
	_ok("CONTROL: the editor was fitted to 759 dp", w759 == 759, str(w759))
	_ok("759 dp: Split is not offered", not split_btn.visible)
	ed.set_mode("split")
	_ok("759 dp: asking for Split gives Write", ed.mode() == "write")
	var w760 := await _fit_width(vw, ed, 760)
	_ok("CONTROL: the editor was fitted to 760 dp", w760 == 760, str(w760))
	_ok("760 dp: Split is offered", split_btn.visible)
	_ok("760 dp: the wanted Split came back by itself (it was asked for at 759)", ed.mode() == "split", ed.mode())
	ed.set_mode("write")
	var w900 := await _fit_width(vw, ed, 900)
	_ok("900 dp: Split is offered", split_btn.visible and w900 == 900)
	_reset(ed, "# Title\n\nsome words")
	await _press(vw, split_btn)
	_ok("tapping Split enters Split", ed.mode() == "split", ed.mode())
	_ok("both panes show", te.visible and ed.preview.visible)
	var a := te.get_global_rect()
	var b: Rect2 = ed.preview.get_global_rect()
	_ok("... side by side, Write on the left, no overlap", a.end.x <= b.position.x + 1.0 and absf(a.position.y - b.position.y) < 2.0, "%s | %s" % [a, b])
	_ok("... of comparable width", absf(a.size.x - b.size.x) < 4.0, "%s | %s" % [a.size.x, b.size.x])
	_ok("the formatting buttons stay live in Split", not (ed.buttons["bold"] as Button).disabled)
	await _tap(vw, te.get_global_rect().get_center())
	te.set_caret_line(2)
	te.set_caret_column(te.get_line(2).length())
	await _type(" zebra")
	_ok("typing in Split lands in the source", _text(ed).ends_with("some words zebra"), _text(ed))
	await get_tree().create_timer(0.6).timeout
	_ok("Preview follows the typing", ed.preview.get_parsed_text().contains("zebra"), ed.preview.get_parsed_text())
	await _shot("4_split.png")
	await _key(KEY_E, true)
	_ok("Ctrl+E from Split goes to Preview", ed.mode() == "preview", ed.mode())
	_ok("... showing the typed text", ed.preview.get_parsed_text().contains("zebra"))
	await _key(KEY_E, true)
	_ok("Ctrl+E from Preview returns to Split", ed.mode() == "split", ed.mode())
	_ok("... with the text intact", _text(ed).ends_with("some words zebra"))
	# squeeze below 760 while in Split
	await _fit_width(vw, ed, 759)
	_ok("squeezed to 759 dp: Split falls back to Write", ed.mode() == "write" and not split_btn.visible, ed.mode())
	_ok("... the text is intact", _text(ed) == "# Title\n\nsome words zebra", _text(ed))
	await _fit_width(vw, ed, 800)
	_ok("grown back to 800 dp: Split returns", ed.mode() == "split", ed.mode())
	# an explicit Write is respected
	await _press(vw, ed.buttons["write"])
	await _fit_width(vw, ed, 759)
	await _fit_width(vw, ed, 900)
	_ok("after choosing Write, growing the pane does not push Split on the user", ed.mode() == "write", ed.mode())
	vw.size = Vector2i(1500, vw.size.y)
	await _frames(6)


# == Smart keys ===================================================================

func _smart_leg(vw: Window, ed, te: TextEdit, scratch: String) -> void:
	var tapc := func(): await _tap(vw, te.get_global_rect().get_center())
	_reset(ed, "")
	await tapc.call()
	await _type("- one")
	await _key(KEY_ENTER)
	_ok("Enter on a bullet continues it", _text(ed) == "- one\n- ", _text(ed))
	await _type("two")
	await _key(KEY_ENTER)
	await _key(KEY_ENTER)
	_ok("Enter on the empty item ends the list", _text(ed) == "- one\n- two\n", _text(ed))
	_ok("... caret on the empty line", te.get_caret_line() == 2 and te.get_caret_column() == 0)
	# undo of a continuation is one step
	_reset(ed, "")
	await tapc.call()
	await _type("- a")
	await _key(KEY_ENTER)
	await _key(KEY_Z, true)
	_ok("undoing a continued bullet takes back the marker AND the newline (one step)", _text(ed) == "- a", _text(ed))
	# numbers
	_reset(ed, "")
	await tapc.call()
	await _type("1. a")
	await _key(KEY_ENTER)
	await _type("b")
	await _key(KEY_ENTER)
	_ok("numbers count up", _text(ed) == "1. a\n2. b\n3. ", _text(ed))
	# tasks
	_reset(ed, "- [x] done")
	await tapc.call()
	te.set_caret_line(0)
	te.set_caret_column(10)
	await _key(KEY_ENTER)
	_ok("a task continues unticked", _text(ed) == "- [x] done\n- [ ] ", _text(ed))
	# quote
	_reset(ed, "> q")
	await tapc.call()
	te.set_caret_line(0)
	te.set_caret_column(3)
	await _key(KEY_ENTER)
	_ok("a quote continues", _text(ed) == "> q\n> ", _text(ed))
	await _key(KEY_ENTER)
	_ok("Enter on an empty quote line ends the quote", _text(ed) == "> q\n", _text(ed))
	# mid-line
	_reset(ed, "- abcd")
	await tapc.call()
	te.set_caret_line(0)
	te.set_caret_column(4)
	await _key(KEY_ENTER)
	_ok("Enter mid-line splits the item and keeps the rest", _text(ed) == "- ab\n- cd", _text(ed))
	# Tab / Shift+Tab
	_reset(ed, "- a\n- b")
	await tapc.call()
	te.set_caret_line(1)
	te.set_caret_column(3)
	await _key(KEY_TAB)
	_ok("Tab indents the list line", _text(ed) == "- a\n\t- b", _text(ed))
	_ok("... and the caret keeps its place in the text", te.get_caret_column() == 4, str(te.get_caret_column()))
	await _key(KEY_ENTER)
	await _key(KEY_ENTER)
	_ok("Enter on the empty indented item steps out one level", _text(ed) == "- a\n\t- b\n- ", _text(ed))
	await _key(KEY_TAB, false, true)
	_ok("Shift+Tab on an unindented item changes nothing", _text(ed) == "- a\n\t- b\n- ", _text(ed))
	await _key(KEY_TAB)
	_ok("Tab indents the empty item", _text(ed) == "- a\n\t- b\n\t- ", _text(ed))
	await _key(KEY_TAB, false, true)
	_ok("Shift+Tab outdents it again", _text(ed) == "- a\n\t- b\n- ", _text(ed))
	# a plain line keeps the editor's own Tab
	_reset(ed, "plain")
	await tapc.call()
	te.set_caret_line(0)
	te.set_caret_column(0)
	await _key(KEY_TAB)
	_ok("Tab on a plain line is the editor's own (a tab character)", _text(ed) == "\tplain", _text(ed))
	# Shift+Enter and fences
	_reset(ed, "- a")
	await tapc.call()
	te.set_caret_line(0)
	te.set_caret_column(3)
	await _key(KEY_ENTER, false, true)
	_ok("Shift+Enter is a plain line break", _text(ed) == "- a\n", _text(ed))
	_reset(ed, "```\n- x")
	await tapc.call()
	te.set_caret_line(1)
	te.set_caret_column(3)
	await _key(KEY_ENTER)
	_ok("Enter inside a fence is a plain newline", _text(ed) == "```\n- x\n", _text(ed))
	# a selection is replaced, not continued
	_reset(ed, "- ab")
	await tapc.call()
	te.select(0, 2, 0, 4)
	await _key(KEY_ENTER)
	_ok("Enter over a selection replaces it with a plain newline", _text(ed) == "- \n", _text(ed))
	# ** pairs, and the closer is typed over
	_reset(ed, "")
	await tapc.call()
	await _type("**")
	_ok("typing ** writes **|** with the caret between", _text(ed) == "****" and te.get_caret_column() == 2, "%s %d" % [_text(ed), te.get_caret_column()])
	await _type("bold")
	await _type("**")
	_ok("typing the closer over the pair gives **bold** and nothing more", _text(ed) == "**bold**" and te.get_caret_column() == 8, "%s %d" % [_text(ed), te.get_caret_column()])
	_reset(ed, "")
	await tapc.call()
	await _type("**")
	await _key(KEY_BACKSPACE)
	_ok("Backspace inside an empty ** pair removes the whole pair", _text(ed) == "", _text(ed))
	_reset(ed, "")
	await tapc.call()
	await _type("***")
	_ok("a third * pairs nothing", _text(ed) == "***", _text(ed))
	_reset(ed, "a*b")
	await tapc.call()
	te.set_caret_line(0)
	te.set_caret_column(1)
	await _type("*")
	_ok("a lone * typed before an unrelated * is just text", _text(ed) == "a**b", _text(ed))
	# shortcuts
	_reset(ed, "word")
	await tapc.call()
	te.set_caret_line(0)
	te.set_caret_column(4)
	await _key(KEY_L, true)
	_ok("Ctrl+L makes the line a task", _text(ed) == "- [ ] word", _text(ed))
	_reset(ed, "make this word")
	await tapc.call()
	te.select(0, 10, 0, 14)
	await _key(KEY_K, true)
	_ok("Ctrl+K writes [[|selected]] and opens the card", _text(ed) == "make this [[|word]]" and ed._sg_visible(), "%s %s" % [_text(ed), ed._sg_visible()])
	await _key(KEY_ESCAPE)
	_reset(ed, "")
	await _shot("5_keys.png")


# == Find and replace =============================================================

func _find_leg(vw: Window, ed, te: TextEdit, phone: bool) -> void:
	var bar: Control = ed._find_bar
	_ok("the find bar exists and starts closed", bar != null and not bar.visible)
	_reset(ed, "Cat sat. cat sat. CAT sat on a cat.\nsecond line cat")
	await _tap(vw, te.get_global_rect().get_center())
	if phone:
		var btn: Button = ed.buttons["find"]
		var sc := btn.get_parent().get_parent() as ScrollContainer
		if sc != null:
			sc.ensure_control_visible(btn)
			await _frames(4)
		await _press(vw, btn)
	else:
		await _key(KEY_F, true)
	_ok("opening Find shows the bar", bar.visible)
	_ok("the find field has focus", ed._find_edit.has_focus())
	if "dialog_close_on_escape" in vw:
		_ok("the window's own Escape is held while the bar is open", not bool(vw.get("dialog_close_on_escape")))
	var pre_img: Image = null
	var pre_d := 0.0
	if not phone:
		pre_img = await _grab()
		var orgp := Vector2(vw.position) + te.get_global_rect().position
		var bandp := _line_band(te, 0)
		pre_d = float(_lean_count(pre_img, int(orgp.x) + 6, int(orgp.x) + int(te.size.x) - 6, int(orgp.y) + bandp.x + 3, int(orgp.y) + bandp.y - 3, _hlr(te).color_of(H.MATCH), _hlr(te).color_of(H.BODY), Color(DARK_GROUND)))
	await _type("cat")
	_ok("the count reads all four case-insensitive hits on four lines' worth of text", ed._find_count.text.begins_with("1 of 5"), ed._find_count.text)
	_ok("... the first hit is selected", te.has_selection() and te.get_selected_text().to_lower() == "cat")
	if not phone:
		## The colour must reach the screen too (a cached line would keep its old colours): the second
		## hit on line 0 is not the selected one, and the focus is in the field, so no caret or focus
		## border is drawn in a similar amber.
		var img_f := await _grab()
		var orgf := Vector2(vw.position) + te.get_global_rect().position
		var band0 := _line_band(te, 0)
		var mink: Color = _hlr(te).color_of(H.MATCH)
		var nf := _lean_count(img_f, int(orgf.x) + 6, int(orgf.x) + int(te.size.x) - 6, int(orgf.y) + band0.x + 3, int(orgf.y) + band0.y - 3, mink, _hlr(te).color_of(H.BODY), Color(DARK_GROUND))
		_ok("the hits are drawn in the match colour on screen (%d warm ink pixels, %d before the query)" % [nf, int(pre_d)], nf >= 20 and nf > int(pre_d) + 15)
	var d0: Dictionary = _hlr(te)._get_line_syntax_highlighting(0)
	var mcol: Color = _hlr(te).color_of(H.MATCH)
	_ok("the hits are coloured in the source", _same(_color_at(d0, 5), mcol) or _same(_color_at(d0, 14), mcol) or _same(_color_at(d0, 0), mcol), str(d0))
	await _key(KEY_ENTER)
	_ok("Enter goes to the next hit", ed._find_count.text.begins_with("2 of 5"), ed._find_count.text)
	await _key(KEY_ENTER, false, true)
	_ok("Shift+Enter goes back", ed._find_count.text.begins_with("1 of 5"), ed._find_count.text)
	await _key(KEY_ENTER, false, true)
	_ok("... wrapping past the first", ed._find_count.text.begins_with("5 of 5"), ed._find_count.text)
	await _press(vw, ed.buttons["find_case"])
	_ok("Match case narrows the hits to the lower-case ones", ed._find_count.text.contains("of 3") or ed._find_count.text.begins_with("3 match"), ed._find_count.text)
	await _press(vw, ed.buttons["find_case"])
	_ok("... and off again widens them", ed._find_count.text.contains("of 5") or ed._find_count.text.begins_with("5 match"), ed._find_count.text)
	# reaching the replace row
	await _press(vw, ed.buttons["find_replace"])
	_ok("Replace… reveals the replace row", ed._find_replace_box.visible)
	ed._replace_edit.grab_focus()
	await _type("dog")
	var t0 := _text(ed)
	_ok("typing in the bar never touches the note", t0 == "Cat sat. cat sat. CAT sat on a cat.\nsecond line cat", t0)
	if phone:
		for k in ["find_prev", "find_next", "find_replace", "find_close", "replace_one", "replace_all", "find_case"]:
			var b: Button = ed.buttons[k]
			_ok("phone: %s is at least 44 px tall (%.0f)" % [k, b.size.y], b.size.y >= 43.9)
		var er: Rect2 = ed.get_global_rect()
		var br: Rect2 = bar.get_global_rect()
		_ok("phone: the bar lies inside the editor's width (no overflow)", br.position.x >= er.position.x - 1.0 and br.end.x <= er.end.x + 1.0, "%s in %s" % [br, er])
		var worst := 0.0
		for c in bar.get_children():
			if c is Control and (c as Control).visible:
				worst = maxf(worst, (c as Control).get_global_rect().end.x - er.end.x)
		_ok("phone: no control in the bar pokes out to the right", worst <= 1.0, str(worst))
	# Replace one: select the first hit, press Replace, then check
	await _press(vw, ed.buttons["find_next"])
	var sel0 := te.get_selected_text()
	await _press(vw, ed.buttons["replace_one"])
	_ok("Replace swaps the selected hit for the replacement and moves on", _text(ed).count("dog") == 1 and _text(ed).length() == t0.length() + 0, _text(ed))
	_ok("... one hit fewer is left", ed._find_count.text.contains("of 4") or ed._find_count.text.begins_with("4 match"), ed._find_count.text)
	_ok("CONTROL: the hit it replaced was a cat (%s)" % sel0, sel0.to_lower() == "cat")
	# Replace all is ONE undo step
	_reset(ed, "Cat sat. cat sat. CAT sat on a cat.\nsecond line cat")
	ed._find_open_bar(true)
	ed._find_edit.text = "cat"
	ed._find_refresh(false)
	ed._replace_edit.text = "dog"
	var t1 := _text(ed)
	var ra: Button = ed.buttons["replace_all"]
	# press its left edge: a centre press cannot see the width of the target it hits
	await _tap(vw, ra.get_global_rect().position + Vector2(3, ra.size.y * 0.5))
	await _frames(3)
	_ok("Replace all replaced every hit, case-insensitively", _text(ed) == "dog sat. dog sat. dog sat on a dog.\nsecond line dog", _text(ed))
	_ok("... and said how many", ed._find_count.text == "Replaced 5", ed._find_count.text)
	_ok("... without saving", _disk("Scratch.md") == "# Scratch\n\nword here\n")
	await _tap(vw, te.get_global_rect().get_center())
	await _key(KEY_Z, true)
	_ok("ONE Ctrl+Z puts every replaced hit back", _text(ed) == t1, _text(ed))
	await _key(KEY_Z, true, true)
	_ok("Ctrl+Shift+Z redoes them all at once", _text(ed) == "dog sat. dog sat. dog sat on a dog.\nsecond line dog", _text(ed))
	await _key(KEY_Z, true)
	# Escape closes the bar, the window stays
	ed._find_edit.grab_focus()
	await _key(KEY_ESCAPE)
	_ok("Escape closes the bar", not bar.visible)
	_ok("... the vault window is still open", vw.visible)
	if "dialog_close_on_escape" in vw:
		_ok("... and gets its own Escape back", bool(vw.get("dialog_close_on_escape")))
	_ok("... and the text is as it was before the replace", _text(ed) == t1, _text(ed))
	var d1: Dictionary = _hlr(te)._get_line_syntax_highlighting(0)
	_ok("a closed bar colours no hits", not _same(_color_at(d1, 0), mcol))
	# an empty query replaces nothing
	ed._find_open_bar(true)
	ed._find_edit.text = ""
	ed._find_refresh(false)
	ed._replace_edit.text = "x"
	var ra2: Button = ed.buttons["replace_all"]
	await _press(vw, ra2)
	_ok("Replace all with an empty query changes nothing", _text(ed) == t1, _text(ed))
	ed._find_close(false)
	# Ctrl+H opens it with the replace row
	if not phone:
		ed._find_replace_box.visible = false
		await _tap(vw, te.get_global_rect().get_center())
		await _key(KEY_H, true)
		_ok("Ctrl+H opens Find with the replace row", bar.visible and ed._find_replace_box.visible)
		ed._find_close(true)
	# a selection seeds the query
	_reset(ed, "alpha beta alpha")
	await _tap(vw, te.get_global_rect().get_center())
	te.select(0, 0, 0, 5)
	if not phone:
		await _key(KEY_F, true)
		_ok("a one-line selection seeds the query", ed._find_edit.text == "alpha", ed._find_edit.text)
		_ok("... and the count follows", ed._find_count.text.contains("2"), ed._find_count.text)
	ed._find_close(false)
	await _shot("6_find.png")


# == Footer =======================================================================

func _footer_leg(vw: Window, ed, te: TextEdit) -> void:
	_reset(ed, "")
	await _tap(vw, te.get_global_rect().get_center())
	await _type("one two")
	await get_tree().create_timer(0.6).timeout
	var f: Label = ed._footer_label
	_ok("the footer counts words and links after typing", f.text == "2 words  ·  0 links", f.text)
	te.insert_text_at_caret(" [[Kelvhold]] [[Xyloth]]")
	await get_tree().create_timer(0.6).timeout
	_ok("... counting the two note links and their words", f.text == "4 words  ·  2 links", f.text)
	_ok("... matching the crate's own count", ed.word_counter.call(_text(ed)) == _bridge.vault_word_count(_text(ed)) and _bridge.vault_word_count(_text(ed)) == 4)
	te.insert_text_at_caret(" see https://example.com/page and [site](https://example.org)")
	await get_tree().create_timer(0.6).timeout
	_ok("... a URL is not a note link", f.text.ends_with("2 links"), f.text)
	_reset(ed, "")
	await _type("a")
	await get_tree().create_timer(0.6).timeout
	_ok("one word reads '1 word'", f.text.begins_with("1 word  "), f.text)
	te.text = ""
	te.insert_text_at_caret(" ")
	await get_tree().create_timer(0.6).timeout
	_ok("an empty note is a real '0 words'", f.text.begins_with("0 words"), f.text)
	# a host that gives no counter shows no figure
	var bare = Ed.new()
	add_child(bare)
	bare.setup("x.md", "one two three", "one two three", false, Callable())
	_ok("with no counters the footer is empty, not '0 words'", bare._footer_label.text == "", bare._footer_label.text)
	bare.word_counter = func(_t: String) -> int: return -1
	bare._refresh_footer()
	_ok("a counter answering -1 (unavailable) is left out, not shown as -1", bare._footer_label.text == "", bare._footer_label.text)
	bare.word_counter = func(_t: String) -> int: return 5
	bare.link_counter = func(_t: String) -> int: return -1
	bare._refresh_footer()
	_ok("a link counter answering -1 is left out and the words stay", bare._footer_label.text == "5 words", bare._footer_label.text)
	bare.queue_free()
	await _shot("7_footer.png")


# == Light palette ===================================================================

func _light_leg(vw: Window) -> void:
	vw.position = Vector2i(9000, 9000)
	var was_dark := DccTheme.is_dark()
	DccTheme.apply_theme(false)
	app.rebuild_theme(was_dark)
	await _frames(6)
	var holder := PanelContainer.new()
	holder.position = Vector2(20, 20)
	holder.custom_minimum_size = Vector2(760, 380)
	holder.size = Vector2(760, 380)
	_vp.add_child(holder)
	var ed2 = Ed.new()
	holder.add_child(ed2)
	ed2.link_spans = func(line: String) -> Array: return _bridge.vault_link_spans(line)
	ed2.setup("light.md", "# Heading one\nplain **bold** [[Kelvhold]] #tag `code` ==mark==\n> quote", "", false, Callable())
	await _frames(8)
	await _contrast_leg_light(ed2)
	holder.queue_free()
	DccTheme.apply_theme(was_dark)
	app.rebuild_theme(false)
	await _frames(4)


func _contrast_leg_light(ed2) -> void:
	var te: TextEdit = ed2.text_edit
	var img := await _grab()
	var org := te.get_global_rect().position
	var ground := img.get_pixel(int(org.x + te.size.x - 24), int(org.y + te.size.y - 12))
	_ok("light: the field ground is the canvas literal", _dist(ground, Color(LIGHT_GROUND)) < 0.012, "%s vs %s" % [ground, LIGHT_GROUND])
	var hl = _hlr(te)
	var rows: Array = []
	var worst := 99.0
	for role in [H.BODY, H.HEADING, H.EMPH, H.CODE, H.LINK, H.TAG, H.HIGHLIGHT, H.STRIKE, H.QUOTE, H.FRONT, H.TASK, H.MATCH]:
		var r: float = H.contrast(hl.color_of(role), Color(LIGHT_GROUND))
		rows.append("%d:%.2f" % [role, r])
		worst = minf(worst, r)
	var mr: float = H.contrast(hl.color_of(H.MARKER), Color(LIGHT_GROUND))
	print("VE3 info: light contrast vs %s -- %s marker:%.2f" % [LIGHT_GROUND, " ".join(rows), mr])
	_ok("light: every text role is at least 4.5:1 on the ground (lowest %.2f)" % worst, worst >= 4.5, str(rows))
	_ok("light: the dimmed marker is at least 3:1 (%.2f)" % mr, mr >= 3.0)
	_ok("light: the heading is NOT the raw accent token (#a4650f is 3.93:1 here) -- it was nudged to the floor", not _same(hl.color_of(H.HEADING), Color("#a4650f")))
	# the colour reaches the screen: the heading line holds ink near the role's colour, and it reads
	var row0 := Rect2(te.get_rect_at_line_column(0, 0))
	var near := _nearest(img, int(org.x + 8), int(org.x + 200), int(org.y + row0.position.y), int(org.y + row0.end.y), hl.color_of(H.HEADING))
	_ok("light: the heading's drawn ink is close to its role colour (%.2f)" % near, near < 0.2)
	var htxt_body := _nearest(img, int(org.x + 8), int(org.x + 200), int(org.y + row0.position.y), int(org.y + row0.end.y), hl.color_of(H.BODY))
	_ok("light: ... and that is not the body colour's ink (heading differs from body: %.2f vs %.2f)" % [near, htxt_body], near < htxt_body or hl.color_of(H.HEADING) != hl.color_of(H.BODY))
	await _shot("8_light.png")
