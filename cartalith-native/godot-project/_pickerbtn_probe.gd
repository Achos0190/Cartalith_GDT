extends Node
## **The third button path**, read back from the built `Button` rather than
## from the factory that built it -- `open_project_dialog.gd::_picker_button()`
## and `phone_project_picker.gd`'s two `DccWidgets.action()` calls, the two
## compositions the cold-start screen has.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _pickerbtn_probe.tscn
##   ... _pickerbtn_probe.tscn -- --force-touch    # tablet: --rCtl:12px
##   ... _pickerbtn_probe.tscn -- --light          # the light palette
##   ... _pickerbtn_probe.tscn -- --phone          # PhoneProjectPicker
##
## `DccTheme._touch` and `is_phone()` are both latched for the life of the
## process, so the four cannot share a run -- the same constraint
## `_btnfill_probe.gd` and `_ds03fit_probe.gd` carry.
##
## **Every `want` is a literal from `design/mcp-2026-09-07/`, never a
## `DccTheme` symbol.** `role_px("btn_radius") == role_px("btn_radius")` passes
## against any value, including the 0 this pass exists to remove.
##
## Canvas source, `Cartalith DCC Environment.dc.html` lines 46-49:
##   primary   `padding:6px 18px;border-radius:8px;background:var(--acc);
##              color:var(--accInk)`
##   secondary `padding:6px 18px;border-radius:8px;background:var(--ins);
##              color:var(--sec)`
## and **no `border:` on either** -- asserted per side, per state, because that
## is the half of the old drawing that made these read as a different control
## from every other button on screen.

var app: Node
var _vp: SubViewport
var _fail := 0
var _checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(label: String, got, want) -> void:
	_checks += 1
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("%s  %-44s got=%s want=%s" % ["ok  " if good else "FAIL", label, got, want])

func _ne(label: String, a, b) -> void:
	_checks += 1
	var good: bool = str(a) != str(b)
	if not good:
		_fail += 1
	print("%s  %-44s a=%s b=%s (must differ)" % ["ok  " if good else "FAIL", label, a, b])

func _sb(b: Button, state: String) -> StyleBoxFlat:
	return b.get_theme_stylebox(state) as StyleBoxFlat

func _buttons(node: Node, out: Array) -> void:
	for c in node.get_children():
		if c is Button and (c as Button).visible:
			out.append(c)
		_buttons(c, out)

## Case-insensitive, and that is not tidiness: the phone composition's labels
## come back **uppercased** (`+ NEW WORLD`, `OPEN PROJECT .ZIP…`) because
## `DccShell.phone_fit()` re-letters an `ACTION_META` button on the way through
## `DccWidgets.phone_pill()`, matching the Android canvas's own
## `letter-spacing:.18em` all-caps actions. A case-sensitive needle silently
## found nothing and the phone half of this probe reported a missing button.
func _by_text(btns: Array, needle: String) -> Button:
	for b in btns:
		if String((b as Button).text).to_lower().find(needle.to_lower()) >= 0:
			return b as Button
	return null

# -- The desktop / tablet composition ----------------------------------------

## `want_fill` is indexed by state, already resolved to canvas literals by the
## caller. Radius, padding and the border-free box are checked on every state,
## because the block this replaces got three of the four states wrong by
## sharing one box between them.
func _shape(tag: String, b: Button, want_fill: Dictionary, radius: int) -> void:
	for state in ["normal", "hover", "pressed", "disabled"]:
		var sb := _sb(b, state)
		_ok("%s %-8s fill" % [tag, state], sb.bg_color, want_fill[state])
		_ok("%s %-8s radius tl" % [tag, state], sb.corner_radius_top_left, radius)
		_ok("%s %-8s radius br" % [tag, state], sb.corner_radius_bottom_right, radius)
		_ok("%s %-8s pad_x" % [tag, state], sb.content_margin_left, 18.0)
		_ok("%s %-8s pad_y" % [tag, state], sb.content_margin_top, 6.0)
		var edges := 0
		for side in [SIDE_LEFT, SIDE_TOP, SIDE_RIGHT, SIDE_BOTTOM]:
			edges += sb.get_border_width(side)
		_ok("%s %-8s no border" % [tag, state], edges, 0)

func _desktop(touch: bool, dark: bool) -> void:
	DccTheme.set_phone(false)
	DccTheme.set_touch(touch)
	DccTheme.set_narrow(false)
	DccTheme.apply_theme(dark)

	var dlg = OpenProjectDialog.new()
	app.add_child(dlg)
	dlg.setup(app)
	await _frames(2)
	dlg.open_welcome()
	dlg.size = Vector2i(1180, 760)
	await _frames(6)

	var btns: Array = []
	_buttons(dlg._picker, btns)
	var names := PackedStringArray()
	for b in btns:
		names.append((b as Button).text)
	print("[picker] buttons=%s" % [names])

	var prim := _by_text(btns, "New world")
	var zip := _by_text(btns, "Open project")
	var hm := _by_text(btns, "heightmap")
	_ok("all three routes present",
		prim != null and zip != null and hm != null, true)
	if prim == null or zip == null or hm == null:
		dlg.queue_free()
		return

	# Canvas literals, `Cartalith DCC Environment.dc.html`:
	# dark  --acc #e0a34a  --accH #f0bd72  --accInk #141005
	#       --ins #191c1e  --sec #a9adb0   --dis #5f6468
	#       --wash2 rgba(224,163,74,.16)
	# light --acc #a4650f  --accH #8a5309  --accInk #f7f4ee
	#       --wash2 rgba(164,101,15,.16)
	#       --ins #eceae4  --sec #3d3f39   --dis #9a9d95
	var ACC := Color("#e0a34a") if dark else Color("#a4650f")
	var ACCH := Color("#f0bd72") if dark else Color("#8a5309")
	var ACCINK := Color("#141005") if dark else Color("#f7f4ee")
	var INS := Color("#191c1e") if dark else Color("#eceae4")
	var SEC := Color("#a9adb0") if dark else Color("#3d3f39")
	var DIS := Color("#5f6468") if dark else Color("#9a9d95")
	## `--wash2` is **per palette and not the same amber**, which the first
	## run of this probe caught: the light block is `rgba(164,101,15,.16)`,
	## its own `--acc` at 16 %, not the dark `rgba(224,163,74,.16)`. Both are
	## grepped out of the canvas, and the pair the guess produced differed in
	## the third decimal -- exactly the drift asserting a `DccTheme` symbol
	## against itself would have hidden.
	var WASH2 := Color(224.0 / 255.0, 163.0 / 255.0, 74.0 / 255.0, 0.16)
	if not dark:
		WASH2 = Color(164.0 / 255.0, 101.0 / 255.0, 15.0 / 255.0, 0.16)
	## PC canvas writes `border-radius:8px` inline; the tablet canvas writes
	## `border-radius:var(--rCtl)` and defines `--rCtl:12px`.
	var R := 12 if touch else 8

	_shape("primary  ", prim, {
		"normal": ACC, "hover": ACCH, "pressed": ACC, "disabled": INS}, R)
	for pair in [["zip      ", zip], ["heightmap", hm]]:
		_shape(String(pair[0]), pair[1] as Button, {
			"normal": INS, "hover": INS,
			"pressed": INS.blend(WASH2), "disabled": INS}, R)

	_ok("primary   ink", prim.get_theme_color("font_color"), ACCINK)
	_ok("primary   hover ink", prim.get_theme_color("font_hover_color"), ACCINK)
	_ok("primary   pressed ink", prim.get_theme_color("font_pressed_color"), ACCINK)
	_ok("primary   disabled ink", prim.get_theme_color("font_disabled_color"), DIS)
	_ok("secondary ink", zip.get_theme_color("font_color"), SEC)
	_ok("secondary hover ink", zip.get_theme_color("font_hover_color"), ACC)
	_ok("secondary pressed ink", zip.get_theme_color("font_pressed_color"), ACC)
	_ok("secondary disabled ink", zip.get_theme_color("font_disabled_color"), DIS)

	## The four-states repair, stated as the thing that can fail. A box shared
	## between `normal` and `disabled` is why a disabled route was pixel-
	## identical to a live one, and these are the assertions the block this
	## replaces could not have passed.
	_ne("primary  normal != disabled", _sb(prim, "normal").bg_color,
		_sb(prim, "disabled").bg_color)
	_ne("secondary normal != pressed", _sb(zip, "normal").bg_color,
		_sb(zip, "pressed").bg_color)
	## And the variant split itself: the two must not draw the same ground.
	_ne("primary  != secondary ground", _sb(prim, "normal").bg_color,
		_sb(zip, "normal").bg_color)

	dlg.hide()
	await _frames(2)
	dlg.queue_free()

# -- The phone composition ---------------------------------------------------

## `phone_project_picker.gd` builds its two actions with `DccWidgets.action()`,
## which the same 2026-09-07 pass already repointed at `button_box()` -- but
## `DccShell.phone_fit()` then re-styles anything carrying `ACTION_META`
## through `DccWidgets.phone_pill()`, so what a phone actually draws is that
## factory's box, not this one. Asserted here anyway, because a desktop-only
## check proves half the surface: the claim under test is that **both** of this
## screen's compositions draw a filled, rounded action rather than the square
## hairline the owner flagged.
func _phone() -> void:
	_vp = SubViewport.new()
	_vp.size = Vector2i(1080, 2340)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(60)

	_ok("is_phone()", app.is_phone(), true)
	var picker: Node = app.get("phone_project_picker")
	_ok("PhoneProjectPicker exists", picker != null, true)
	if picker == null:
		return
	picker.call("open")
	await _frames(10)

	var btns: Array = []
	_buttons(picker, btns)
	var names := PackedStringArray()
	for b in btns:
		names.append((b as Button).text)
	print("[phone] buttons=%s" % [names])
	var prim := _by_text(btns, "New world")
	var zip := _by_text(btns, "Open project")
	_ok("phone: both actions present", prim != null and zip != null, true)
	if prim == null or zip == null:
		return
	for pair in [["phone primary", prim], ["phone secondary", zip]]:
		var b := pair[1] as Button
		var sb := _sb(b, "normal")
		print("[phone] %-16s fill=%s alpha=%.2f radius=%d h=%.0f" \
			% [pair[0], sb.bg_color, sb.bg_color.a, sb.corner_radius_top_left,
				b.size.y])
		_ok("%s is rounded" % pair[0], sb.corner_radius_top_left > 0, true)
		_ok("%s clears the tap floor" % pair[0],
			b.size.y >= DccTheme.PHONE_TAP_MIN, true)

	## **Two measured gaps on the phone action row, reported and not asserted.**
	## Both live in `DccWidgets.phone_pill()` / `DccTheme.pill()`, which another
	## lane owns this pass; a probe in this lane that failed on them would be
	## red for a defect this lane is not allowed to touch, and asserting the
	## current wrong value would be worse -- it would lock the gap in.
	##
	## Measured against `design/mcp-2026-09-07/Cartalith Android.dc.html` line
	## 55, `OPEN PROJECT .ZIP…`:
	##   `min-height:50px;border-radius:20px;background:var(--chip)`
	## with `--chip:rgba(255,255,255,.05)`.
	##
	## 1. **The secondary is transparent.** `pill(false, …)` gives it
	##    `Color(0,0,0,0)` plus a hairline border, which is exactly the drawing
	##    the owner flagged on 2026-09-07, surviving on the phone path after
	##    the desktop path was fixed. The canvas gives it a real (faint) ground.
	## 2. **The radius is 24, the canvas says 20** -- `phone_pill()` derives a
	##    fully-rounded 48 dp capsule from the Android canvas's *primary*
	##    button; this screen's own two actions are drawn at 20.
	var zsb := _sb(zip, "normal")
	if zsb.bg_color.a <= 0.0:
		print("GAP   phone secondary is transparent -- canvas says background:")
		print("GAP     var(--chip) rgba(255,255,255,.05); owner: DccWidgets.phone_pill()")
	_ok("phone primary is filled", _sb(prim, "normal").bg_color.a > 0.0, true)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	var argv := OS.get_cmdline_user_args()
	var touch := "--force-touch" in argv
	var phone := "--phone" in argv
	var dark := not ("--light" in argv)
	print("=== _pickerbtn_probe  mode=%s  theme=%s ===" \
		% ["phone" if phone else ("tablet" if touch else "pointer"),
			"dark" if dark else "light"])

	if phone:
		await _phone()
	else:
		app = load("res://shell/app.tscn").instantiate()
		add_child(app)
		await get_tree().create_timer(1.2).timeout
		if app.open_project_dialog != null:
			app.open_project_dialog.hide()
		await _frames(3)
		await _desktop(touch, dark)

	print("PB done -- %d check(s), %d failure(s)" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
