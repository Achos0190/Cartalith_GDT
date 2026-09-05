extends Node
## DS-13 verifier -- the phone viewport's thumb cluster
## (`design/proposed-2026-09-05-round2/PhoneViewportControls.dc.html`).
##
## Run **windowed**, not `--headless`: the brief's claims are tap targets and a
## hit test, and `MISTAKES.md`'s own row says headless proves nothing for
## either. Layout still resolves under the dummy driver, so the probe runs
## there too -- it just prints a banner saying the density claim is not earned.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _ds13cluster_probe.tscn
##       -- --force-touch --nowelcome --vp 1080x2400
##
## **`--vp WxH` is read by this script**, at `_parse_vp()` below, and it is the
## flag that matters. `--resolution` asks the *OS* for a window and Windows
## clamps a 2400 px-tall request to the panel: measured once here before the
## size was set in code, that landed a 1031 px short side at aspect ~1.03,
## which fails `DccShell._PHONE_ASPECT_MAX` -- so the run reported desktop
## chrome and every phone claim in it was about the wrong composition.
## `window_set_size` plus `get_window().size` are both written, before the app
## is instantiated. An unrecognised or missing `--vp` is refused, not defaulted.
##
## Run it twice: `--vp 1080x2400` and `--vp 1440x3168`, the two densities
## `viewport_host.gd`'s own HD-03 note names. **Name the density beside every
## number this prints** -- the same disc is 7.4 mm at 395 ppi and 5.7 mm at
## 510, and neither figure means anything without the panel behind it.
##
## Dark is forced and asserted. This machine boots `mode="light"`
## (`cartalith_settings.cfg`) and the board's colours are the dark set, so a
## light run would compare the armed pill against the wrong palette entirely.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row.

var app: Node
var vp: Control
var fails := 0
var checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(label: String, got, want) -> void:
	checks += 1
	var good: bool = (got == want)
	if not good:
		fails += 1
	print("  %s %-52s got=%s want=%s" % ["PASS" if good else "FAIL", label, str(got), str(want)])

func _ok_true(label: String, cond: bool, detail: String = "") -> void:
	checks += 1
	if not cond:
		fails += 1
	print("  %s %-52s %s" % ["PASS" if cond else "FAIL", label, detail])

func _force_dark() -> bool:
	if not DccTheme.is_dark():
		DccTheme.apply_theme(true)
		app.rebuild_theme(false)
	return DccTheme.is_dark()

func _find_all(node: Node, pred: Callable, out: Array) -> Array:
	if pred.call(node):
		out.append(node)
	for c in node.get_children():
		_find_all(c, pred, out)
	return out

func _pad_buttons() -> Dictionary:
	var out := {}
	var found: Array = []
	_find_all(vp.get("_navpad"), func(n): return n is Button and n.has_meta("dcc_navpad_glyph"), found)
	for b in found:
		out[String((b as Button).get_meta("dcc_navpad_glyph"))] = b
	return out

func _hex(c: Color) -> String:
	return "#" + c.to_html(false)

## `-- --vp WxH`, accepted as one token or two. Refuses rather than defaults: a
## probe that silently measured whatever window it was handed is how the same
## box gets reported as three densities.
func _parse_vp() -> Vector2i:
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if not String(args[i]).begins_with("--vp"):
			continue
		var v := String(args[i]).substr(4).lstrip("= ")
		if v == "" and i + 1 < args.size():
			v = String(args[i + 1])
		var wh: PackedStringArray = v.split("x")
		if wh.size() == 2 and wh[0].is_valid_int() and wh[1].is_valid_int():
			return Vector2i(int(wh[0]), int(wh[1]))
		print("BAD --vp argument: %s (want WxH)" % v)
		return Vector2i.ZERO
	print("NO --vp WxH given; this probe will not guess a density.")
	return Vector2i.ZERO

func _ready() -> void:
	var headless := DisplayServer.get_name() == "headless"
	var wd := Timer.new()
	wd.wait_time = 200.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	## Both writes, and before the app exists: the OS frame may be clamped, the
	## root Window's own rect is what every `Control` lays out against, and
	## `DccShell._compute_layout_mode()` reads it once during `_ready()`.
	var want := _parse_vp()
	if want == Vector2i.ZERO:
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	vp = app.viewport
	var scr: Vector2 = get_viewport().get_visible_rect().size
	var pscale: float = float(app.get("_phone_scale"))
	var ppmm: float = (510.0 / 25.4) if scr.x >= 1400.0 else (395.0 / 25.4)

	print("=== DS-13 cluster · driver=%s screen=%s phone=%s _phone_scale=%.4f ppi=%.0f ==="
		% [DisplayServer.get_name(), str(scr), str(app.get("_phone")), pscale, ppmm * 25.4])
	if headless:
		print("  !! DRIVER IS HEADLESS -- layout below is real, the density and")
		print("     hit-test claims are NOT earned. Re-run windowed.")
	_ok_true("dark palette forced", _force_dark(), "is_dark=%s" % str(DccTheme.is_dark()))
	await _frames(4)

	var pad: Container = vp.get("_navpad")
	_ok_true("navpad built (touch composition)", pad != null,
		"" if pad != null else "not touch -- pass --force-touch")
	if pad == null:
		get_tree().quit(1)
		return

	# --- 1 · a ROW, not a column ---------------------------------------------
	print("\n=== 1 · the owner ruling: a bottom cluster, not a right-edge column ===")
	_ok("container class", pad.get_class(), "HBoxContainer")
	var r: Rect2 = pad.get_global_rect()
	print("  info cluster rect=%s  viewport size=%s" % [str(r), str(vp.size)])
	_ok_true("wider than tall (a row)", r.size.x > r.size.y,
		"%.0f x %.0f px" % [r.size.x, r.size.y])
	var btns := _pad_buttons()
	_ok("navpad buttons", btns.size(), 4)
	var ys: Array = []
	for k in btns:
		ys.append(snappedf((btns[k] as Button).get_global_rect().position.y, 0.5))
	_ok_true("all four share one y (one band)", ys.min() == ys.max(), str(ys))
	## §6.2's `right:12px`. Portrait reports `right: 0.0`, so `NAVPAD_EDGE` is
	## what actually places it -- assert the drawn gap, not the constant.
	_ok_true("right edge sits 12 px off the viewport edge",
		absf(vp.size.x - r.end.x - 12.0) < 0.51,
		"gap=%.1f px" % (vp.size.x - r.end.x))
	var coords: Control = vp.get("_coords_label")
	_ok_true("clears the coordinate readout below it",
		r.end.y <= coords.get_global_rect().position.y + 0.51,
		"cluster bottom=%.0f  coords top=%.0f" % [r.end.y, coords.get_global_rect().position.y])

	# --- 2 · tap targets, at a named density ---------------------------------
	print("\n=== 2 · tap targets at %.0f ppi, _phone_scale %.4f ===" % [ppmm * 25.4, pscale])
	for k in ["zoom_out", "zoom_in", "tool_pan", "view_fill"]:
		var br: Rect2 = (btns[k] as Button).get_global_rect()
		print("  info %-10s %.0f x %.0f px = %.1f x %.1f dp = %.2f x %.2f mm"
			% [k, br.size.x, br.size.y, br.size.x / pscale, br.size.y / pscale,
			   br.size.x / ppmm, br.size.y / ppmm])
		_ok_true("%s >= 44 dp both axes" % k,
			br.size.x / pscale >= 43.5 and br.size.y / pscale >= 43.5,
			"%.2f x %.2f dp" % [br.size.x / pscale, br.size.y / pscale])
		_ok_true("%s >= DccTheme.PHONE_TAP_MIN device px" % k,
			br.size.x >= float(DccTheme.PHONE_TAP_MIN) and br.size.y >= float(DccTheme.PHONE_TAP_MIN),
			"%.0f px vs floor %d" % [br.size.y, DccTheme.PHONE_TAP_MIN])

	# --- 3 · the rocker: one pill, two halves, one hairline ------------------
	print("\n=== 3 · zoom rocker ===")
	var lo: Rect2 = (btns["zoom_out"] as Button).get_global_rect()
	var hi: Rect2 = (btns["zoom_in"] as Button).get_global_rect()
	_ok_true("minus is left of plus", lo.position.x < hi.position.x,
		"minus x=%.0f  plus x=%.0f" % [lo.position.x, hi.position.x])
	var rule: ColorRect = vp.get("_zoom_rule")
	_ok_true("hairline exists", rule != null, "")
	if rule != null:
		var rr: Rect2 = rule.get_global_rect()
		print("  info rule rect=%s  colour=%s (line=%s)"
			% [str(rr), _hex(rule.color), _hex(DccTheme.c("line"))])
		_ok("rule colour is the `line` token", rule.color, DccTheme.c("line"))
		_ok_true("halves touch across the rule", absf(hi.position.x - lo.end.x - rr.size.x) < 0.51,
			"gap=%.1f  rule w=%.1f" % [hi.position.x - lo.end.x, rr.size.x])
		_ok_true("rule is vertically inset inside the pill", rr.size.y < lo.size.y,
			"rule h=%.0f  pill h=%.0f" % [rr.size.y, lo.size.y])
	var gap_pan: float = (btns["tool_pan"] as Button).get_global_rect().position.x - hi.end.x
	print("  info rocker->pan gap=%.1f px (NAVPAD_GAP 10 x %.4f = %.1f)"
		% [gap_pan, pscale, 10.0 * pscale])
	_ok_true("the three groups are separated, the two halves are not",
		gap_pan > (rule.size.x if rule != null else 1.0) * 3.0, "%.1f px" % gap_pan)

	## Corner radii read off the drawn styleboxes -- the capsule is only a
	## capsule if the halves round their outer ends and nothing else.
	var sb_lo := (btns["zoom_out"] as Button).get_theme_stylebox("normal") as StyleBoxFlat
	var sb_hi := (btns["zoom_in"] as Button).get_theme_stylebox("normal") as StyleBoxFlat
	var sb_pan := (btns["tool_pan"] as Button).get_theme_stylebox("normal") as StyleBoxFlat
	print("  info minus radii TL/TR/BR/BL = %d/%d/%d/%d" % [sb_lo.corner_radius_top_left,
		sb_lo.corner_radius_top_right, sb_lo.corner_radius_bottom_right, sb_lo.corner_radius_bottom_left])
	print("  info plus  radii TL/TR/BR/BL = %d/%d/%d/%d" % [sb_hi.corner_radius_top_left,
		sb_hi.corner_radius_top_right, sb_hi.corner_radius_bottom_right, sb_hi.corner_radius_bottom_left])
	_ok_true("minus rounds left only",
		sb_lo.corner_radius_top_left > 0 and sb_lo.corner_radius_top_right == 0, "")
	_ok_true("plus rounds right only",
		sb_hi.corner_radius_top_right > 0 and sb_hi.corner_radius_top_left == 0, "")
	_ok_true("discs round all four", sb_pan.corner_radius_top_left > 0
		and sb_pan.corner_radius_top_left == sb_pan.corner_radius_bottom_right, "")
	_ok("minus drops its shared border", sb_lo.border_width_right, 0)
	_ok("plus drops its shared border", sb_hi.border_width_left, 0)
	_ok_true("radius tracks the device hit size, not the raw 44",
		sb_pan.corner_radius_top_left == int(lo.size.y / 2.0),
		"radius=%d  hit/2=%d" % [sb_pan.corner_radius_top_left, int(lo.size.y / 2.0)])

	# --- 4 · the armed latch: §0.4's chip formula, NOT an accent fill ---------
	print("\n=== 4 · pan latch, armed ===")
	var pan_btn: Button = btns["tool_pan"]
	var off_sb := pan_btn.get_theme_stylebox("normal") as StyleBoxFlat
	var off_bg: Color = off_sb.bg_color
	var off_ink: Color = pan_btn.get_theme_color("icon_normal_color")
	print("  info OFF  bg=%s a=%.2f  border=%s  ink=%s"
		% [_hex(off_bg), off_bg.a, _hex(off_sb.border_color), _hex(off_ink)])
	_ok("off ink is the `text` token", off_ink, DccTheme.c("text"))
	_ok("off border is the `line` token", off_sb.border_color, DccTheme.c("line"))
	_ok_true("off fill is `panel` at .92", absf(off_bg.a - 0.92) < 0.005
		and absf(off_bg.r - DccTheme.c("panel").r) < 0.004,
		"a=%.3f rgb=%s panel=%s" % [off_bg.a, _hex(off_bg), _hex(DccTheme.c("panel"))])

	var toasts_before: Array = []
	_find_all(app, func(n): return n is Label and "one finger pans" in String(n.get("text")), toasts_before)
	vp.set_pan_mode(true)
	await _frames(3)
	var on_sb := pan_btn.get_theme_stylebox("normal") as StyleBoxFlat
	var on_bg: Color = on_sb.bg_color
	var on_ink: Color = pan_btn.get_theme_color("icon_normal_color")
	print("  info ON   bg=%s a=%.2f  border=%s  ink=%s"
		% [_hex(on_bg), on_bg.a, _hex(on_sb.border_color), _hex(on_ink)])
	print("  info tokens accent=%s accent_wash_2=%s(a=%.2f) accent_ink=%s bg=%s"
		% [_hex(DccTheme.c("accent")), _hex(DccTheme.c("accent_wash_2")),
		   DccTheme.c("accent_wash_2").a, _hex(DccTheme.c("accent_ink")), _hex(DccTheme.c("bg"))])
	_ok("armed border is the `accent` token", on_sb.border_color, DccTheme.c("accent"))
	_ok("armed ink is the `accent` token", on_ink, DccTheme.c("accent"))
	## The refutation the board asks for, stated as a discriminating test: the
	## pre-DS-13 paint was an accent FILL with a `bg`-token glyph. Both must be
	## false now, and a check that only asserted "the fill changed" would pass
	## on the very paint the board rejects.
	_ok_true("armed fill is NOT the accent fill it used to be",
		not (absf(on_bg.r - DccTheme.c("accent").r) < 0.01
			and absf(on_bg.g - DccTheme.c("accent").g) < 0.01), _hex(on_bg))
	_ok_true("armed glyph is NOT the dark `bg` ink it used to be",
		on_ink != DccTheme.c("bg"), _hex(on_ink))
	_ok_true("armed fill is the OFF fill washed toward accent",
		on_bg != off_bg and on_bg.r > off_bg.r and on_bg.b < on_bg.r,
		"off=%s on=%s" % [_hex(off_bg), _hex(on_bg)])
	_ok_true("armed fill keeps the scrim (still translucent)", on_bg.a > 0.9 and on_bg.a < 1.0,
		"a=%.4f" % on_bg.a)
	_ok_true("button reports pressed", pan_btn.button_pressed, "")
	_ok_true("engine agrees the mode is on", vp.pan_mode(), "")

	## The announcement. `_show_phone_toast` builds a Label carrying the text.
	var toasts_after: Array = []
	_find_all(app, func(n): return n is Label and "one finger pans" in String(n.get("text")), toasts_after)
	print("  info toast labels before=%d after=%d" % [toasts_before.size(), toasts_after.size()])
	_ok_true("arming raised the phone toast", toasts_after.size() > toasts_before.size(),
		"%d -> %d" % [toasts_before.size(), toasts_after.size()])
	if toasts_after.size() > 0:
		print("  info toast text = %s" % String((toasts_after[0] as Label).text))

	vp.set_pan_mode(false)
	await _frames(3)
	var back := (pan_btn.get_theme_stylebox("normal") as StyleBoxFlat).bg_color
	_ok("disarm restores the off fill", back, off_bg)

	# --- 5 · the detent rule: tracks the sheet, hides at `full` ---------------
	print("\n=== 5 · detents ===")
	for det in ["peek", "half", "full", "half"]:
		app.set_phone_detent(det)
		await get_tree().create_timer(0.6).timeout
		var ins: Dictionary = app.phone_content_insets()
		var band: float = vp.size.y - float(ins.get("top", 0.0)) - float(ins.get("bottom", 0.0))
		var pr: Rect2 = pad.get_global_rect()
		print("  info detent=%-5s bottom_inset=%.0f band=%.0f visible=%s cluster_bottom=%.0f"
			% [det, float(ins.get("bottom", 0.0)), band, str(pad.visible), pr.end.y])
		if det == "full":
			_ok_true("hidden at `full` (96 dp of map, nothing to navigate)",
				not pad.visible, "band=%.0f px, one tap target" % band)
		else:
			_ok_true("visible at `%s`" % det, pad.visible, "band=%.0f px" % band)
			_ok_true("`%s`: cluster sits above the sheet's top edge" % det,
				pr.end.y <= vp.size.y - float(ins.get("bottom", 0.0)) + 0.51,
				"bottom=%.0f  sheet top=%.0f" % [pr.end.y, vp.size.y - float(ins.get("bottom", 0.0))])

	# --- 6 · the 3D/2D FAB is gone, and its half that is real is not ---------
	print("\n=== 6 · §6.2's FAB column ===")
	var threed: Array = []
	var bare_3d := func(n: Node) -> bool:
		if not (n is Button or n is Label):
			return false
		return String(n.get("text")).strip_edges() in ["3D", "2D", "3D/2D"]
	_find_all(app, bare_3d, threed)
	for t in threed:
		print("  info node with a bare 2D/3D label: %s (%s) text=%s"
			% [t.name, t.get_class(), String(t.get("text"))])
	_ok("no 3D/2D toggle node anywhere in the shell", threed.size(), 0)
	_ok_true("recentre absorbed into the cluster", btns.has("view_fill"),
		"tooltip=%s" % (btns["view_fill"] as Button).tooltip_text)
	var want_exag: Array[float] = [1.0, 2.0, 4.0]
	_ok("relief exaggeration rungs still exist (the toast's real half)",
		DccSettings.RELIEF_EXAG_CHOICES, want_exag)

	print("\n=== %d checks, %d FAILED ===" % [checks, fails])
	get_tree().quit(1 if fails > 0 else 0)
