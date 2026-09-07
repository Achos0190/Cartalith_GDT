extends Node
## **The buttons the owner flagged on 2026-09-07**, read back from the built
## `Button` rather than from the factory that built it.
##
##   Godot_v4.7.1 --headless --path . _btnfill_probe.tscn                 # pointer
##   Godot_v4.7.1 --headless --path . _btnfill_probe.tscn -- --force-touch # tablet
##
## `DccTheme._touch` is latched for the life of the process, so the two
## densities cannot share a run -- same constraint `_ds03fit_probe.gd` carries.
##
## **Every `want` below is a literal from `design/mcp-2026-09-07/`, never a
## `DccTheme` symbol.** Asserting `role_px("btn_radius") == role_px(
## "btn_radius")` would pass against any value including the 0 this pass
## exists to remove; these are the canvas's own numbers, so the test fails if
## the constant drifts away from the design.

var _fail := 0

func _ok(name: String, got, want) -> void:
	var pass_: bool = (got == want)
	if not pass_:
		_fail += 1
	print("%s  %-46s got=%s want=%s" % ["ok  " if pass_ else "FAIL", name, got, want])

func _sb(b: Button, state: String) -> StyleBoxFlat:
	return b.get_theme_stylebox(state) as StyleBoxFlat

func _ready() -> void:
	var touch := "--force-touch" in OS.get_cmdline_user_args()
	DccTheme.set_touch(touch)
	DccTheme.apply_theme(true)
	var den := "TABLET" if touch else "POINTER"
	print("=== _btnfill_probe  density=%s  theme=dark ===" % den)

	var col := VBoxContainer.new()
	add_child(col)
	var prim := DccWidgets.action(col, "Run stage", func(): pass, true)
	var sec := DccWidgets.action(col, "Run 1 -> 10", func(): pass, false)

	# ── Canvas literals, PC artboard ────────────────────────────────────────
	# `background:var(--acc);color:var(--accInk)`  --acc:#e0a34a --accInk:#141005
	# `background:var(--ins);color:var(--sec)`     --ins:#191c1e --sec:#a9adb0
	var ACC := Color("#e0a34a")
	var ACCINK := Color("#141005")
	var INS := Color("#191c1e")
	var SEC := Color("#a9adb0")
	var DIS := Color("#5f6468")          ## `--dis`, the undo/redo disabled ink.
	# PC canvas writes `border-radius:8px` inline; the tablet canvas writes
	# `border-radius:var(--rCtl)` and defines `--rCtl:12px`.
	var R := 12 if touch else 8

	_ok("primary   normal   fill",  _sb(prim, "normal").bg_color,   ACC)
	_ok("primary   hover    fill",  _sb(prim, "hover").bg_color,    Color("#f0bd72"))
	_ok("primary   pressed  fill",  _sb(prim, "pressed").bg_color,  ACC)
	_ok("primary   disabled fill",  _sb(prim, "disabled").bg_color, INS)
	_ok("secondary normal   fill",  _sb(sec, "normal").bg_color,    INS)
	_ok("secondary hover    fill",  _sb(sec, "hover").bg_color,     INS)
	_ok("secondary disabled fill",  _sb(sec, "disabled").bg_color,  INS)

	_ok("primary   normal   ink",   prim.get_theme_color("font_color"),          ACCINK)
	_ok("primary   hover    ink",   prim.get_theme_color("font_hover_color"),    ACCINK)
	_ok("secondary normal   ink",   sec.get_theme_color("font_color"),           SEC)
	# The canvas's `style-hover` on an `--ins` chip is `color:var(--acc)`.
	_ok("secondary hover    ink",   sec.get_theme_color("font_hover_color"),     ACC)
	_ok("primary   disabled ink",   prim.get_theme_color("font_disabled_color"), DIS)
	_ok("secondary disabled ink",   sec.get_theme_color("font_disabled_color"),  DIS)

	for st in ["normal", "hover", "pressed", "disabled"]:
		_ok("primary   %-8s radius" % st, _sb(prim, st).corner_radius_top_left, R)
		_ok("secondary %-8s radius" % st, _sb(sec, st).corner_radius_top_left, R)
	# The canvas puts NO border on either variant: 0 of its `--ins` chips carry
	# a `border:` declaration, and the `--acc` ones carry none either.
	_ok("primary   normal   border", _sb(prim, "normal").border_width_left, 0)
	_ok("secondary normal   border", _sb(sec, "normal").border_width_left, 0)

	# ── The correctness property, not a cosmetic one ────────────────────────
	# Four states needed four boxes; the shipped code had two, so `disabled`
	# was pixel-identical to `normal` and an unpressable button looked live.
	# **The signal is fill-or-ink, not fill.** The first version of this probe
	# asserted a differing FILL on both variants and failed on the secondary --
	# correctly, and the assertion was the thing that was wrong. The canvas's own
	# disabled control is the undo/redo pair, and it HOLDS `background:var(--ins)`
	# while dropping the ink to `var(--dis)`; a secondary button is already on
	# `--ins`, so there is no fill left to drop and the ink carries the whole
	# signal. Measured, that signal is large: 7.58:1 enabled vs 2.86:1 disabled
	# on the same ground. The primary drops both.
	for pair in [["primary", prim], ["secondary", sec]]:
		var btn: Button = pair[1]
		var moved: bool = _sb(btn, "disabled").bg_color != _sb(btn, "normal").bg_color 			or btn.get_theme_color("font_disabled_color") != btn.get_theme_color("font_color")
		_ok("%-9s disabled reads != normal" % pair[0], moved, true)
	_ok("primary   disabled drops fill", _sb(prim, "disabled").bg_color != _sb(prim, "normal").bg_color, true)
	_ok("secondary pressed  != normal", _sb(sec, "pressed").bg_color != _sb(sec, "normal").bg_color, true)
	_ok("primary   hover    != normal", _sb(prim, "hover").bg_color != _sb(prim, "normal").bg_color, true)
	# `--wash2` is 16% alpha; blended onto `--ins` it must come out OPAQUE, or
	# the button loses its ground over whatever panel it sits on.
	_ok("secondary pressed  opaque", _sb(sec, "pressed").bg_color.a, 1.0)

	# ── The two guards this factory already carried ─────────────────────────
	# A wrapped label in an HBox grew the tool bar to 265 px once.
	var row := HBoxContainer.new()
	add_child(row)
	var in_row := DccWidgets.action(row, "Generate world", func(): pass, true)
	_ok("HBox child does NOT autowrap", in_row.autowrap_mode, TextServer.AUTOWRAP_OFF)
	_ok("VBox child DOES autowrap", prim.autowrap_mode, TextServer.AUTOWRAP_WORD_SMART)
	_ok("primary carries ACTION_META", prim.get_meta(DccWidgets.ACTION_META), true)
	_ok("min height", int(prim.custom_minimum_size.y), 44 if touch else 26)

	# ── The layout claim, measured rather than reasoned ─────────────────────
	# Explicit content margins mean a StyleBoxFlat's border does NOT add to its
	# minimum size, so dropping the 1 px outline must move NOTHING. `MISTAKES
	# .md`: a laid-out size is not a minimum -- this reads the minimum.
	var pad_x := 18 if touch else 10
	var pad_y := 9 if touch else 4
	_ok("stylebox min size x", _sb(prim, "normal").get_minimum_size().x, float(pad_x * 2))
	_ok("stylebox min size y", _sb(prim, "normal").get_minimum_size().y, float(pad_y * 2))

	print("=== %s: %d failure(s) ===" % [den, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
