extends Node
## Reachability + layout verifier for the §6.6 bespoke MORE screens.
##
## The 2026-09-05 ruling replaced seven menu drills with five purpose-built
## screens, and the one thing that must not follow from that is a command that
## stops being reachable. §6.6's own `help` screen states the rule: *"The phone
## reorganises rather than truncates: every desktop function is reachable
## through MAP · GENERATE · PLAN · MORE."*
##
## So this walks the REAL built screens (it pushes each one onto the real
## `PhoneMenu` stack and reads the labels that actually got drawn) and checks
## every top-level item of every program menu against them. An item counts as
## covered when its own text was drawn, or -- for a submenu drawn inline as
## §6.6 `seg` chips -- when one of its children's texts was.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##       _phonemore_reach_probe.tscn -- --force-touch --nowelcome
##
## `--force-touch` is required: `PhoneMenu` is only built by
## `DccShell._build_phone_shell()`, and without it the run gets a desktop shell
## and nothing to walk.

var app: Node
var pm            ## PhoneMenu
var _fail := 0

## **One density per launch, and that is not a convenience.** The first version
## of this probe resized the window between measurements and reported the same
## `body_min_w=742` at 720, 1080 and 1440 -- because `PhoneMenu._scale` is read
## once, in `setup()`, from `DccShell.phone_scale()`, and nothing re-reads it on
## a resize. Three sizes in one process is therefore ONE layout measured against
## three screen widths, which is the shape of a measurement without being one.
##
## Pass `--size=WxH`; the window is sized before `app.tscn` is instantiated, so
## `_compute_layout_mode()` sees it. Default 1080x2400, the tested handset.
const DEFAULT_SIZE := Vector2i(1080, 2400)

func _arg_size() -> Vector2i:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if not s.begins_with("--size="):
			continue
		var wh := s.substr(7).split("x")
		if wh.size() == 2 and wh[0].is_valid_int() and wh[1].is_valid_int():
			return Vector2i(int(wh[0]), int(wh[1]))
	return DEFAULT_SIZE

## §6.6's root table, by label -- the eight rows that must be on the root.
const ROOT_MUST: Array = [
	"Project", "Civilization", "Data manager", "Asset library",
	"Travel library", "Simulation", "Preferences", "Help & about",
]

## The two program menus §6.6's root table has no row for and that must
## therefore be carried by the fallback band.
const ROOT_REST_MUST: Array = ["Edit", "Window"]

## screen id -> the program menu whose top-level rows it must carry.
const SCREEN_COVERS: Dictionary = {
	"project": "File", "data": "Data", "prefs": "Preferences",
}

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("REACH %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append(String((c as Label).text))
		elif c is Button:
			out.append(String((c as Button).text))
		_labels(c, out)

## Render one screen and return every string it drew.
func _screen_texts(id: String) -> Array:
	pm.open()
	if id != "more":
		pm._push_screen(id)
	await _frames(2)
	var out: Array = []
	_labels(pm._screen_body, out)
	return out

func _clean(t: String) -> String:
	var s := t.strip_edges()
	if s.ends_with("▸"):
		s = s.substr(0, s.length() - 1).strip_edges()
	return s

## `menus.gd::_readout()` writes `"%s   %s"` -- a label and a LIVE value
## ("Working set   244 MB of 31.2 GB"). The value moves between the frame that
## rendered the screen and the frame that reads the popup back, so identity is
## the label half, not the whole string. Split on the triple space `menus.gd`
## uses, and only when there is one.
func _key(t: String) -> String:
	var s := _clean(t)
	var cut := s.find("   ")
	return s.substr(0, cut).strip_edges() if cut > 0 else s

func _has(texts: Array, want: String) -> bool:
	var k := _key(want)
	for t in texts:
		if _key(String(t)) == k:
			return true
	return false

## Every top-level item of `p`, as `[text, submenu_node]`.
func _top_items(p: PopupMenu) -> Array:
	var out: Array = []
	for i in p.item_count:
		if p.is_item_separator(i):
			continue
		out.append([_clean(p.get_item_text(i)), p.get_item_submenu(i)])
	return out

func _menu_popup(title: String) -> PopupMenu:
	for child in app.menu_bar_row.get_children():
		if child is MenuButton and String(child.text) == title:
			var p := (child as MenuButton).get_popup()
			p.about_to_popup.emit()
			return p
	return null

## Every Control whose combined minimum width exceeds `limit`, with whatever
## text it or its descendants carry -- a bare container name says which node
## overflows and nothing about WHY, which is the half that matters.
func _widest(node: Node, limit: float, out: Array, depth: int = 0) -> void:
	if node is Control:
		var mw: float = (node as Control).get_combined_minimum_size().x
		if mw > limit:
			out.append("%s%s (%s) min_w=%.0f %s" % [
				"  ".repeat(depth), node.name, node.get_class(), mw,
				_own_text(node).substr(0, 90)])
	for c in node.get_children():
		_widest(c, limit, out, depth + 1)

func _own_text(n: Node) -> String:
	var bits: Array = []
	_labels(n, bits)
	if n is Label:
		bits.push_front(String((n as Label).text))
	var out := PackedStringArray()
	for b in bits:
		var t := String(b).strip_edges()
		if t != "":
			out.append(t)
	return " / ".join(out)

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 180.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := _arg_size()
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	pm = app.get("_phone_menu")
	if pm == null:
		print("REACH  !! no PhoneMenu -- run with -- --force-touch")
		get_tree().quit(1)
		return
	print("REACH phone_scale=%.3f" % app.phone_scale())

	## -- §6.6's glyph column, against the face that has to draw it ------------
	var f := DccTheme.mono(0)
	var missing := PackedStringArray()
	for g in ["⧉", "◍", "⇅", "▦", "≋", "◷", "⚙", "?"]:
		if f != null and not f.has_char(String(g).unicode_at(0)):
			missing.append(g)
	print("REACH glyphs without an outline in the mono face: %s"
		% ("none" if missing.is_empty() else " ".join(missing)))

	## -- Root -----------------------------------------------------------------
	var root: Array = await _screen_texts("more")
	for label in ROOT_MUST:
		_check("root row '%s'" % label, _has(root, String(label)))
	for label in ROOT_REST_MUST:
		_check("root fallback row '%s'" % label, _has(root, String(label)))

	## -- Every program menu is reachable --------------------------------------
	var menu_names: Array = []
	for child in app.menu_bar_row.get_children():
		if child is MenuButton:
			menu_names.append(String(child.text))
	print("REACH menus on the bar: %s" % ", ".join(menu_names))

	var screen_cache: Dictionary = {}
	for sid in ["project", "civ", "data", "sim", "prefs"]:
		screen_cache[sid] = await _screen_texts(String(sid))
		print("REACH screen '%s' drew %d strings" % [sid, (screen_cache[sid] as Array).size()])

	for m in menu_names:
		var covered := false
		if _has(root, String(m)):
			covered = true
		for sid in SCREEN_COVERS:
			if String(SCREEN_COVERS[sid]) == String(m):
				covered = true
		if String(m) == "Assets":
			covered = _has(root, "Asset library")
		if String(m) == "Help":
			covered = _has(root, "Help & about")
		_check("menu '%s' is reachable from the phone" % m, covered)

	## -- Every top-level row of a screen-owned menu is drawn ------------------
	for sid in SCREEN_COVERS:
		var menu := String(SCREEN_COVERS[sid])
		var p := _menu_popup(menu)
		if p == null:
			_check("%s menu exists" % menu, false)
			continue
		var texts: Array = screen_cache[sid]
		var lost := PackedStringArray()
		for entry in _top_items(p):
			var text := String(entry[0])
			var node := String(entry[1])
			if _has(texts, text):
				continue
			## A submenu drawn inline as chips contributes its CHILDREN's texts
			## and a caption of §6.6's choosing ("Interval" for File's
			## "Autosave interval   every 5 min"), so the parent text is not
			## what to look for.
			if node != "":
				var sub := p.get_node_or_null(NodePath(node)) as PopupMenu
				if sub != null:
					var any := false
					for j in sub.item_count:
						if sub.is_item_separator(j):
							continue
						if _has(texts, _clean(sub.get_item_text(j))):
							any = true
							break
					if any:
						continue
			lost.append(text)
		_check("%s: every top-level row reaches screen '%s'" % [menu, sid],
			lost.is_empty(), "missing: %s" % " | ".join(lost))

	## -- Layout, at this launch's density -------------------------------------
	print("REACH layout density %dx%d  phone_scale=%.3f" % [want.x, want.y, app.phone_scale()])
	for sid in ["more", "project", "civ", "data", "sim", "prefs"]:
		var t: Array = await _screen_texts(String(sid))
		var mw: float = pm._screen_body.get_combined_minimum_size().x
		var over: Array = []
		_widest(pm._screen_body, float(want.x), over, 0)
		print("REACH layout %dx%d  screen=%-8s body_min_w=%.0f  screen_w=%d  rows=%d"
			% [want.x, want.y, sid, mw, want.x, (t as Array).size()])
		for line in over:
			print("REACH   over: %s" % line)
		_check("%dx%d / %s fits the screen width" % [want.x, want.y, sid],
			mw <= float(want.x), "min_w=%.0f > %d" % [mw, want.x])

	pm.close()
	print("REACH done: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
