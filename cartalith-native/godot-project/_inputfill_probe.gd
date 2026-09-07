extends Node
## **The inputs the owner flagged on 2026-09-07**, read back from the built
## `OptionButton` / `CheckBox` / `SpinBox` / `LineEdit` rather than from the
## factory that built them.
##
##   Godot_v4.7.1 --headless --path . _inputfill_probe.tscn                 # pointer
##   Godot_v4.7.1 --headless --path . _inputfill_probe.tscn -- --force-touch # tablet
##
## `DccTheme._touch` is latched for the life of the process, so the two
## densities cannot share a run -- the same constraint `_btnfill_probe.gd`
## carries. Both palettes *can* share one, and do.
##
## **Every `want` is a literal from `design/mcp-2026-09-07/`, never a
## `DccTheme` symbol.** An assertion whose want side is `role_px("btn_radius")`
## passes against any value including the 0 this batch exists to remove.

const ACC := {"dark": "e0a34a", "light": "a4650f"}    ## --acc
const AINK := {"dark": "141005", "light": "f7f4ee"}   ## --accInk
const INS := {"dark": "191c1e", "light": "eceae4"}    ## --ins
const SUR := {"dark": "0d0e0f", "light": "f4f2ee"}    ## --sur
const INK := {"dark": "e8ebec", "light": "111210"}    ## --ink
const SEC := {"dark": "a9adb0", "light": "3d3f39"}    ## --sec
const DIS := {"dark": "5f6468", "light": "9a9d95"}    ## --dis

var fails: Array[String] = []
var checks := 0
var ctx := ""

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
		fails.append("%s %s: got %s want %s" % [ctx, label, got, want])

## A colour sampled out of an `Image` cannot be compared at hex precision.
## `Image.set_pixel()` on `FORMAT_RGBA8` writes `uint8(c * 255.0)` -- a
## *truncation* -- so `#e8ebec` round-trips as `e8ebeb` about as often as not.
## The tolerance is one 8-bit step and nothing more: every colour this probe
## distinguishes (`--acc` vs `--ins` vs `--sur` vs `--ink` vs `--dis`) is tens
## of steps apart in every channel, so no mutation can hide inside it.
## `tol` is in 8-bit steps and defaults to the quantisation alone. The switch's
## *edge* samples pass 5 instead, and the arithmetic is worth writing down: the
## touch track is 22 px, so its centre line falls between two rows and the
## nearest one is half a pixel off axis. At the knob's own edge that leaves
## 1.5 % of track showing through -- 3.3/255 in the light palette, where track
## and knob are furthest apart. A 50 % blend, which is what mutating the knob
## diameter produces, is 114/255. The window is 15x smaller than the smallest
## defect it has to catch, and the mutation run says so rather than this
## comment: five survivors before these samples were added, none after.
func near(label: String, got: Color, want_hex: String, tol: float = 1.5) -> void:
	checks += 1
	var w := Color(want_hex)
	var d := maxf(maxf(absf(got.r - w.r), absf(got.g - w.g)), absf(got.b - w.b))
	if d > tol / 255.0:
		fails.append("%s %s: got %s want %s" % [ctx, label, hx(got), want_hex])

func sb_of(c: Control, name: String) -> StyleBox:
	return c.get_theme_stylebox(name)

func flat(c: Control, name: String) -> StyleBoxFlat:
	return c.get_theme_stylebox(name) as StyleBoxFlat

func _ready() -> void:
	var touch := "--force-touch" in OS.get_cmdline_user_args()
	DccTheme.set_phone(false)
	DccTheme.set_touch(touch)
	## `--ctl` is 24 px pointer (ENV:25) and 36 px touch (ENV:1819); `--rCtl`
	## is 12 px and the PC canvas writes `border-radius:8px` inline.
	var R := 12 if touch else 8
	## `padding:3px 9px` on the PC chip (ENV:522); the touch pair is `ROLE`'s
	## own canvas-derived 16/9.
	var PX := 16 if touch else 9
	var PY := 9 if touch else 3
	## ENV:365 track/knob; AND:311 for the touch pair (geometry only).
	var SW := 40 if touch else 30
	var SH := 22 if touch else 17
	var SK := 18 if touch else 13
	print("=== _inputfill_probe  density=%s ===" % ("TABLET" if touch else "POINTER"))

	for theme in ["dark", "light"]:
		DccTheme.apply_theme(theme == "dark")
		ctx = "[%s]" % theme
		var col := VBoxContainer.new()
		add_child(col)

		# ── choice(): the closed dropdown ─────────────────────────────────
		var ob := DccWidgets.choice(col, "Projection", ["equirect", "mercator"], 0,
			func(_i): pass)
		for st in ["normal", "hover", "disabled"]:
			eq("dropdown %s fill" % st, hx(flat(ob, st).bg_color), INS[theme])
		eq("dropdown normal radius", flat(ob, "normal").corner_radius_top_left, R)
		eq("dropdown normal border", flat(ob, "normal").border_width_left, 0)
		eq("dropdown normal pad_x", flat(ob, "normal").content_margin_left, PX)
		eq("dropdown normal pad_y", flat(ob, "normal").content_margin_top, PY)
		## `pressed` is the one derived state: `--wash2` blended onto `--ins`
		## rather than used raw, exactly as `button_box()` derives it.
		eq("dropdown pressed != normal",
			flat(ob, "pressed").bg_color != flat(ob, "normal").bg_color, true)
		eq("dropdown ink", hx(ob.get_theme_color("font_color")), INK[theme])
		eq("dropdown hover ink", hx(ob.get_theme_color("font_hover_color")), ACC[theme])
		eq("dropdown disabled ink",
			hx(ob.get_theme_color("font_disabled_color")), DIS[theme])
		eq("dropdown caret follows font", ob.get_theme_constant("modulate_arrow"), 1)
		## The trap: `fit_to_longest_item` is managed by `DccShell.dock_fit()`
		## and `phone_fit()`, and this pass must leave Godot's default standing.
		eq("dropdown fit_to_longest_item untouched", ob.fit_to_longest_item, true)
		## `style_popup()` already owned the popup; this asserts it still runs.
		eq("dropdown popup styled",
			ob.get_popup().has_theme_stylebox_override("panel"), true)

		# ── toggle(): the canvas's switch ─────────────────────────────────
		var cb := DccWidgets.toggle(col, "Enable steering", false, func(_v): pass)
		for st in ["normal", "hover", "pressed", "disabled", "focus"]:
			eq("toggle %s box is empty" % st, sb_of(cb, st) is StyleBoxEmpty, true)
		var off: Texture2D = cb.get_theme_icon("unchecked")
		var on: Texture2D = cb.get_theme_icon("checked")
		eq("switch size", str(off.get_size()), str(Vector2(SW, SH)))
		var io_: Image = off.get_image()
		var ii: Image = on.get_image()
		var mid := int(SH / 2)
		## `left:2px` off / `left:15px` on for a 13 px knob on a 30 px track
		## (ENV:365, ENV:1354) -- asserted at the knob's own **edges**, not its
		## centre. A centre sample is what a first version of this probe did,
		## and mutating the knob diameter by 1 px in either direction survived
		## it: the middle of a 12 px knob and the middle of a 14 px knob are
		## both knob-coloured. The four pixels below are the two that must be
		## track and the two that must be knob, so diameter *and* inset are
		## both pinned, and the same four generalise to the 40/18 touch pair.
		near("switch off: track ends at x=1", io_.get_pixel(1, mid), SUR[theme], 5.0)
		near("switch off: knob starts at x=2", io_.get_pixel(2, mid), INK[theme], 5.0)
		near("switch off: knob ends at x=2+k-1",
			io_.get_pixel(2 + SK - 1, mid), INK[theme], 5.0)
		near("switch off: track resumes at x=2+k",
			io_.get_pixel(2 + SK, mid), SUR[theme], 5.0)
		near("switch on: track ends at x=w-2", ii.get_pixel(SW - 2, mid), ACC[theme], 5.0)
		near("switch on: knob ends at x=w-3", ii.get_pixel(SW - 3, mid), INK[theme], 5.0)
		near("switch on: knob starts at x=w-2-k",
			ii.get_pixel(SW - 2 - SK, mid), INK[theme], 5.0)
		near("switch on: track resumes at x=w-3-k",
			ii.get_pixel(SW - 3 - SK, mid), ACC[theme], 5.0)
		var offd: Image = (cb.get_theme_icon("unchecked_disabled") as Texture2D).get_image()
		near("switch disabled track", offd.get_pixel(SW - 3, mid), INS[theme])
		near("switch disabled knob", offd.get_pixel(2, mid), DIS[theme], 5.0)
		## Corners must be transparent -- a square switch is the failure mode
		## the rounded-rect coverage exists to prevent.
		eq("switch corner transparent", io_.get_pixel(0, 0).a < 0.5, true)

		# ── number(): the field and its arrows ────────────────────────────
		var spin := DccWidgets.number(col, "Octaves", 1.0, 12.0, 1.0, 6.0,
			func(_v): pass)
		var le := spin.get_line_edit()
		eq("spin field fill", hx(flat(le, "normal").bg_color), INS[theme])
		eq("spin field radius", flat(le, "normal").corner_radius_top_left, R)
		eq("spin field border", flat(le, "normal").border_width_left, 0)
		eq("spin field pad_x", flat(le, "normal").content_margin_left, PX)
		eq("spin field ink", hx(le.get_theme_color("font_color")), INK[theme])
		eq("spin caret", hx(le.get_theme_color("caret_color")), ACC[theme])
		## LEFT, and this pin is the second half of a defect worth stating.
		## `e830112` shipped `number()` right-aligning its field AND this line
		## asserting it, so the probe went green on the regression it was
		## supposed to catch and would have led a later session to "fix" the
		## correct alignment by reverting it. Found by the verifier, not by the
		## sweep -- `--check-only` and a passing probe both agreed with it.
		##
		## The canvas backs LEFT. `ENV:351` -- the `text-align:right` that
		## `e830112` cited -- is a `width:52px;flex:none` readout SPAN with no
		## ground, sitting between a slider and the row edge; right-aligning 52
		## px moves digits a few px. `number()`'s field is `SIZE_EXPAND_FILL`
		## (388 px in New World, 170 in the planner), so the same declaration
		## pushed the ink 343 px from its own label. The two real `<input>`s,
		## `ENV:222` and `ENV:498`, set no `text-align` at all.
		eq("spin readout left-aligned", le.alignment, HORIZONTAL_ALIGNMENT_LEFT)
		## `outline:none` on the canvas's input, so focus holds the ground and
		## adds the shell's own engaged edge -- the one derived field state.
		eq("spin focus holds ground", hx(flat(le, "focus").bg_color), INS[theme])
		eq("spin focus edge", hx(flat(le, "focus").border_color), ACC[theme])
		eq("spin focus edge width", flat(le, "focus").border_width_left, 1)
		eq("spin read_only ground", hx(flat(le, "read_only").bg_color), INS[theme])
		for slot in ["up_background", "down_background", "up_background_hovered",
				"down_background_disabled"]:
			eq("spin %s empty" % slot, sb_of(spin, slot) is StyleBoxEmpty, true)
		eq("spin arrow ink", hx(spin.get_theme_color("up_icon_modulate")), SEC[theme])
		eq("spin arrow hover ink",
			hx(spin.get_theme_color("down_hover_icon_modulate")), ACC[theme])
		eq("spin arrow disabled ink",
			hx(spin.get_theme_color("up_disabled_icon_modulate")), DIS[theme])

		# ── well(): the text field ────────────────────────────────────────
		var field := LineEdit.new()
		col.add_child(field)
		DccWidgets.well(field)
		eq("well fill", hx(flat(field, "normal").bg_color), INS[theme])
		eq("well radius", flat(field, "normal").corner_radius_top_left, R)
		eq("well border", flat(field, "normal").border_width_left, 0)
		eq("well ink", hx(field.get_theme_color("font_color")), INK[theme])

		# ── the ink the accent ground needs, so `--accInk` is not orphaned ──
		eq("accent ink token intact", hx(DccTheme.c("accent_ink")), AINK[theme])

		## Minimums, printed rather than asserted: the hazard this batch could
		## have introduced is a control whose minimum *grew* and propagated out
		## through a scroll container with its own axis disabled.
		print("  %s min: dropdown=%s switch=%s spin=%s" % [ctx,
			str(ob.get_combined_minimum_size()), str(cb.get_combined_minimum_size()),
			str(spin.get_combined_minimum_size())])
		col.queue_free()
		await get_tree().process_frame

	## **The palette watch**, exercised rather than asserted from its source:
	## flip the palette and confirm the switch repaints and does not recurse.
	var col2 := VBoxContainer.new()
	add_child(col2)
	DccTheme.apply_theme(true)
	var watched := DccWidgets.toggle(col2, "Watched", true, func(_v): pass)
	var before: Color = (watched.get_theme_icon("checked") as Texture2D).get_image().get_pixel(2, int(SH / 2))
	DccTheme.apply_theme(false)
	watched.notification(Control.NOTIFICATION_THEME_CHANGED)
	var after: Color = (watched.get_theme_icon("checked") as Texture2D).get_image().get_pixel(2, int(SH / 2))
	ctx = "[watch]"
	near("switch repaints on palette flip (was dark)", before, ACC["dark"])
	near("switch repaints on palette flip (now light)", after, ACC["light"])

	print("checks=%d fails=%d" % [checks, fails.size()])
	for f in fails:
		print("  FAIL ", f)
	get_tree().quit(1 if fails.size() > 0 else 0)
