extends Node

# Independent verifier for the 2026-09-07 button fill/radius batch.
# Every `want` below is a literal read out of
#   design/mcp-2026-09-07/Cartalith DCC Environment.dc.html  (L25 dark, L1818 light)
#   design/mcp-2026-09-07/Cartalith Tablet.dc.html           (--rCtl:12px)
# No DccTheme symbol appears on the want side, so a token asserted against
# itself cannot pass.

const ACC := {"dark": "e0a34a", "light": "a4650f"}   # --acc
const ACCH := {"dark": "f0bd72", "light": "8a5309"}  # --accH
const AINK := {"dark": "141005", "light": "f7f4ee"}  # --accInk
const INS := {"dark": "191c1e", "light": "eceae4"}   # --ins
const SEC := {"dark": "a9adb0", "light": "3d3f39"}   # --sec
const DIS := {"dark": "5f6468", "light": "9a9d95"}   # --dis
const WASH2 := {"dark": Color(0.878431, 0.639216, 0.290196, 0.16),
				"light": Color(0.643137, 0.396078, 0.058824, 0.16)}

var fails: Array[String] = []
var checks := 0

func hx(c: Color) -> String:
	return "%02x%02x%02x" % [roundi(c.r * 255.0), roundi(c.g * 255.0), roundi(c.b * 255.0)]

func eq(label: String, got, want) -> void:
	checks += 1
	var same := false
	if (got is float or got is int) and (want is float or want is int):
		same = absf(float(got) - float(want)) < 0.001
	else:
		same = str(got) == str(want)
	if not same:
		fails.append("%s: got %s want %s" % [label, got, want])

func _ready() -> void:
	for theme in ["dark", "light"]:
		DccTheme.apply_theme(theme == "dark")
		for dens in ["pointer", "tablet"]:
			DccTheme.set_phone(false)
			DccTheme.set_touch(dens == "tablet")
			check(theme, dens)
	# phone must keep the desktop radius: is_tablet() is `_touch and not _phone_mode`
	DccTheme.apply_theme(true)
	DccTheme.set_touch(true)
	DccTheme.set_phone(true)
	var ph := build(true, "action")
	eq("phone radius", (ph.get_theme_stylebox("normal") as StyleBoxFlat).corner_radius_top_left, 8)
	DccTheme.set_phone(false)
	DccTheme.set_touch(false)

	print("CHECKS=%d FAILS=%d" % [checks, fails.size()])
	for f in fails:
		print("  FAIL ", f)
	print("RESULT=", "PASS" if fails.is_empty() else "FAIL")
	get_tree().quit(0 if fails.is_empty() else 1)

func build(primary: bool, kind: String) -> Button:
	var host := VBoxContainer.new()
	add_child(host)
	if kind == "action":
		return DccWidgets.action(host, "Run stage 3", func() -> void: pass, primary)
	return DccWidgets.modal_button(host, "Commit", func() -> void: pass, primary)

func check(theme: String, dens: String) -> void:
	var radius := 12 if dens == "tablet" else 8
	for kind in ["action", "modal"]:
		for primary in [true, false]:
			var b := build(primary, kind)
			var tag := "%s/%s/%s/%s" % [theme, dens, kind, "prim" if primary else "sec"]
			var box := {}
			for st in ["normal", "hover", "pressed", "disabled"]:
				box[st] = b.get_theme_stylebox(st) as StyleBoxFlat
				eq("%s %s radius" % [tag, st], box[st].corner_radius_top_left, radius)
				eq("%s %s radius_br" % [tag, st], box[st].corner_radius_bottom_right, radius)
				eq("%s %s border" % [tag, st], box[st].border_width_left, 0)
				eq("%s %s alpha" % [tag, st], box[st].bg_color.a, 1.0)
			# ---- fills, against canvas hex ----
			if primary:
				eq("%s normal fill" % tag, hx(box["normal"].bg_color), ACC[theme])
				eq("%s hover fill" % tag, hx(box["hover"].bg_color), ACCH[theme])
				eq("%s pressed fill" % tag, hx(box["pressed"].bg_color), ACC[theme])
				eq("%s disabled fill" % tag, hx(box["disabled"].bg_color), INS[theme])
			else:
				eq("%s normal fill" % tag, hx(box["normal"].bg_color), INS[theme])
				eq("%s hover fill" % tag, hx(box["hover"].bg_color), INS[theme])
				eq("%s disabled fill" % tag, hx(box["disabled"].bg_color), INS[theme])
				var want_press := Color(INS[theme]).blend(WASH2[theme])
				eq("%s pressed fill" % tag, hx(box["pressed"].bg_color), hx(want_press))
				# pressed must actually be visible against normal
				checks += 1
				if hx(box["pressed"].bg_color) == INS[theme]:
					fails.append("%s pressed indistinguishable from normal" % tag)
			# ---- ink, against canvas hex ----
			eq("%s ink" % tag, hx(b.get_theme_color("font_color")),
				AINK[theme] if primary else SEC[theme])
			eq("%s ink hover" % tag, hx(b.get_theme_color("font_hover_color")),
				AINK[theme] if primary else ACC[theme])
			eq("%s ink pressed" % tag, hx(b.get_theme_color("font_pressed_color")),
				AINK[theme] if primary else ACC[theme])
			eq("%s ink disabled" % tag, hx(b.get_theme_color("font_disabled_color")), DIS[theme])
			# ---- disabled must not read as enabled: fill OR ink must move ----
			checks += 1
			var fill_moved: bool = hx(box["disabled"].bg_color) != hx(box["normal"].bg_color)
			var ink_moved: bool = hx(b.get_theme_color("font_disabled_color")) \
				!= hx(b.get_theme_color("font_color"))
			if not (fill_moved or ink_moved):
				fails.append("%s disabled reads as enabled" % tag)
			# ---- padding preserved ----
			var want_px := 18 if kind == "modal" else (DccTheme.role_px("btn_pad_x") if dens == "tablet" else 10)
			var want_py := 8 if kind == "modal" else (DccTheme.role_px("btn_pad_y") if dens == "tablet" else 4)
			eq("%s pad_x" % tag, box["normal"].content_margin_left, want_px)
			eq("%s pad_y" % tag, box["normal"].content_margin_top, want_py)
			b.get_parent().queue_free()
