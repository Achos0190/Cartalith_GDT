extends Node
## **The canvas figures this lane re-anchored, asserted as literals.**
##
## Every number below is quoted from the live canvases under
## `design/mcp-2026-09-07/` and written here as a LITERAL, never as the
## constant it is compared against -- `MISTAKES.md`'s "Write a test that pins a
## constant": `assert_eq(x, THE_CONSTANT)` holds for every value of it and
## catches nothing. Each line names the canvas and the line number it came
## from, so a re-base can re-resolve it.
##
##   godot --path . _canvasfig_probe.tscn -- --vp 1920x1080 --tag pc
##   godot --path . _canvasfig_probe.tscn -- --force-touch --vp 2560x1600 --tag tab
##   godot --path . _canvasfig_probe.tscn -- --force-touch --vp 412x892 --tag phone
##
## Flags this probe actually reads, grepped from the body below:
##   `--vp WxH`      SubViewport size in physical px. Default 1920x1080.
##   `--tag NAME`    prefix on every output line. Default `cf`.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`.
## Any other `--flag` aborts rather than being silently ignored.
##
## Which legs run is decided by the composition the shell actually came up in,
## reported on the first line, so one command per frame covers all three.

var app: Node
var _vp: SubViewport
var _tag := "cf"
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _eq(got, want, what: String) -> void:
	var ok: bool = (got == want)
	if not ok:
		_fail += 1
	_log("%s  %s   got=%s want=%s" % ["PASS" if ok else "FAIL", what, str(got), str(want)])

func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	for i in a.size():
		if a[i] == name and i + 1 < a.size():
			return a[i + 1]
	return dflt

func _reject_unknown_args() -> bool:
	var known := ["--vp", "--tag", "--force-touch"]
	var takes := ["--vp", "--tag"]
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		if not a[i].begins_with("--"):
			i += 1
			continue
		if not known.has(a[i]):
			_log("ABORT unknown argument %s" % a[i])
			return false
		i += 2 if takes.has(a[i]) else 1
	return true

func _walk(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_walk(c, out)

func _all(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

func _ready() -> void:
	_tag = _arg("--tag", "cf")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1920x1080").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null and app.phone_project_picker.visible:
		app.phone_project_picker.hide()
	await _frames(6)

	var phone: bool = app.is_phone()
	var tablet: bool = DccTheme.is_tablet()
	_log("viewport %dx%d  phone=%s  tablet=%s" % [_vp.size.x, _vp.size.y, phone, tablet])

	if phone:
		await _phone_legs()   ## Awaited: the pill legs sit past an `await`.
	else:
		_menu_bar_legs(tablet)

	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

## `Cartalith DCC Environment.dc.html:57`
##   font:var(--m1) 'IBM Plex Mono',monospace;letter-spacing:.18em;
##   color:var(--acc);margin:0 12px 0 4px
## `--m1` is `10px` pointer (`ENV:25`) and `12px` touch (`ENV:1819`); `.18em`
## is 1.8 px and 2.16 px, both of which round to a Godot spacing of 2; `--acc`
## is `#e0a34a` dark. And `ENV:104` is ONE readout, `font:var(--m1)`,
## `color:var(--dim)` = `#8d9296`.
func _menu_bar_legs(tablet: bool) -> void:
	var row: Control = app.menu_bar_row
	if row == null:
		_eq(false, true, "the menu bar was built")
		return
	var bar_row: Node = row.get_parent()
	var wordmark: Label = null
	var gap: Control = null
	for c in bar_row.get_children():
		if wordmark == null and c is Label and (c as Label).text == "CARTALITH":
			wordmark = c as Label
		elif wordmark != null and gap == null and c.get_class() == "Control":
			gap = c as Control
	if wordmark == null:
		_eq(false, true, "found the CARTALITH wordmark in the menu bar")
		return
	_eq(wordmark.get_theme_font_size("font_size"), 12 if tablet else 10,
		"ENV:57 wordmark font: var(--m1) = 10 pointer / 12 touch")
	_eq(wordmark.get_theme_font(&"font").spacing_glyph, 2,
		"ENV:57 wordmark letter-spacing .18em -> 2 px at both densities")
	_eq(wordmark.get_theme_color("font_color"), DccTheme.c("accent"),
		"ENV:57 wordmark color: var(--acc), not the near-white --ink")
	if gap != null:
		_eq(int(gap.custom_minimum_size.x), 12, "ENV:57 wordmark margin-right: 12px")
	else:
		_eq(false, true, "found the wordmark's trailing gap")

	## The five readout cells. Only their type is asserted -- the canvas draws
	## one cell and this shell draws five deliberately, which is recorded at the
	## call site and is not this probe's to enforce.
	var readouts := 0
	for key in ["top_world", "top_res", "top_cpu", "top_gpu", "top_mem"]:
		var l: Label = app._status_labels.get(key)
		if l == null:
			continue
		readouts += 1
		_eq(l.get_theme_font_size("font_size"), 12 if tablet else 10,
			"ENV:104 readout %s font: var(--m1)" % key)
		_eq(l.get_theme_color("font_color"), DccTheme.c("text_dim"),
			"ENV:104 readout %s color: var(--dim)" % key)
	_eq(readouts, 5, "all five menu-bar readout cells were reached")

	## **The other half of `set_status()`'s per-strip default, pinned from the
	## opposite direction.** `ENV:1222` gives the status bar's key hints
	## `color:var(--faint)`, which is a DIFFERENT ink from the menu bar's
	## `var(--dim)` one row up -- so a single default cannot serve both, and a
	## mutation that collapses them has to go red on one of these two legs.
	## Written through `set_status()` with no token, which is the path every
	## real caller of a hint takes.
	app.set_status("hint", "canvasfig probe")
	var hint: Label = app._status_labels.get("hint")
	if hint != null:
		_eq(hint.get_theme_color("font_color"), DccTheme.c("text_faint"),
			"ENV:1222 status-bar hint ink: var(--faint), NOT the menu bar's --dim")
	else:
		_eq(false, true, "found the status bar's hint slot")
	app.set_status("hint", "")

## `Cartalith Android.dc.html`:
##   :177  sheet grab row `height:20px`, pill `42x4 r2 background:var(--bord)`
##   :599  bar cell `gap:3px`
##   :600  active pill `padding:4px 16px; border-radius:13px; background:t.bg`
##   :1460 `t.bg` for the active tab = `var(--wash)`
##   :31   `--wash: rgba(224,163,74,.14)`, `--bord: rgba(255,255,255,.16)`
##   :605  gesture row pill `112x4 r2 background:var(--bord)`
func _phone_legs() -> void:
	var sh: Node = app
	var grab: Control = sh._phone_sheet_grab
	if grab != null:
		_eq(int(grab.custom_minimum_size.y), sh._pscale(20),
			"AND:177 sheet grab row height 20 dp")
		for c in _all(grab):
			if c is ColorRect:
				_eq((c as ColorRect).color, DccTheme.c("border"),
					"AND:177 sheet grab pill: var(--bord)")
				break
	else:
		_eq(false, true, "found _phone_sheet_grab")

	var inset: Control = sh._phone_gesture_inset
	if inset != null:
		var rects: Array = []
		for c in _all(inset):
			if c is ColorRect:
				rects.append(c)
		## Two: the 90 %-opaque ground, then the pill. The pill is the one
		## whose width is the canvas's 112, asked for by size rather than by
		## child order so a reorder cannot silently retarget this.
		var pill: ColorRect = null
		for r in rects:
			if int((r as ColorRect).size.x) == sh._pscale(112):
				pill = r
		if pill != null:
			_eq(pill.color, DccTheme.c("border"), "AND:605 gesture pill: var(--bord)")
		else:
			_eq(false, true, "found the 112 dp gesture pill")
	else:
		_eq(false, true, "found _phone_gesture_inset")

	var cells: Dictionary = sh._phone_tab_cells
	if cells.is_empty():
		_eq(false, true, "found the phone tab cells")
		return
	var one: Dictionary = cells.values()[0]
	var pill_node: PanelContainer = one.get("pill")
	var col: Control = pill_node.get_parent() if pill_node != null else null
	if col != null:
		_eq(col.get_theme_constant("separation"), sh._pscale(3),
			"AND:599 bar cell gap: 3px")
	var pad: Control = pill_node.get_child(0) if pill_node != null else null
	if pad != null:
		_eq(pad.get_theme_constant("margin_top"), sh._pscale(4),
			"AND:600 active pill padding-top: 4px")
		_eq(pad.get_theme_constant("margin_left"), sh._pscale(16),
			"AND:600 active pill padding-left: 16px (unchanged)")

	## The active pill's own box. Forced live rather than read off the boot
	## state, so the leg cannot pass because nothing was lit.
	sh._phone_tab = String(cells.keys()[0])
	sh._refresh_phone_tabs()
	await _frames(2)
	var lit: Dictionary = cells[sh._phone_tab]
	var lit_pill: PanelContainer = lit.get("pill")
	var sb: StyleBox = lit_pill.get_theme_stylebox("panel")
	if sb is StyleBoxFlat:
		var f := sb as StyleBoxFlat
		_eq(f.bg_color, Color(DccTheme.c("accent"), 0.14),
			"AND:600+1460+31 active pill fill: var(--wash) rgba(224,163,74,.14)")
		_eq(f.corner_radius_top_left, sh._pscale(13),
			"AND:600 active pill border-radius: 13px")
	else:
		_eq(false, true, "the active pill carries a StyleBoxFlat")
