extends Node
## MS-1 classifier probe (`MOBILE_SHELL_MS1_SPEC.md` §5, bar B1): calls `DccTheme.classify()` and
## `DccTheme.width_class_of()` directly, without booting a shell, over §2.2's fourteen windows in both
## orientations at UI scale 1.0 and 1.25, the OnePlus Pad 3 (owner request, 2026-11-07), and boundary rows
## at 899/900/901 dp and aspect 0.599/0.600/0.601, each approached from both sides (the hysteresis band).
##
##   godot --headless --path . _ms1class_probe.tscn -- --table
##
## And the live legs (B4, and S3's class-change request), windowed with the lane's `override.cfg` and a
## wiped user dir, booting the real shell in a `--vp` host:
##   godot --path . _ms1class_probe.tscn -- --live phone  --force-touch --vp 1080x2340 --density 2.625
##   godot --path . _ms1class_probe.tscn -- --live tablet --force-touch --vp 2388x1668 --density 2
##   godot --path . _ms1class_probe.tscn -- --live swap   --force-touch --vp 1800x3200 --density 1
## A live leg resizes its host `SubViewport` and emits `root.size_changed` itself, because a host resize
## does not resize the root window, and the shell listens to the root (as on a device, where the root IS
## its viewport).
##
## Headless is correct here: nothing is drawn and nothing is timed; it is arithmetic on a pure function.
## Arguments read: `--table`, `--live phone|tablet|swap`, `--vp WxH`, `--density D` (also the shell's),
## `--force-touch` (the shell's). Anything else aborts with exit 2 (`MISTAKES.md`, "Write a probe's usage
## header").
##
## **Every expected value below is a literal written by hand from the spec's text**, never computed by the
## function under test (`MISTAKES.md`, "Write a test that pins a constant"). The table must go red when
## any of `TABLET_MIN_DP` 900, `PHONE_ASPECT_MAX` 0.6, `BAND_DP` 16 or `BAND_ASPECT` 0.01 is mutated
## (B1), and when a width-class bound moves. It fails, by design, on the tree before S1, where
## `classify()` does not exist (`MISTAKES.md`, "Add a probe in the same commit that changes the
## behaviour").
##
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or no
## `classify()` to call.

var _fails := 0
var _checks := 0
## Loaded, not referenced by class name, so this file still parses on a tree that has no `classify()`
## and can report that as a failure instead of a parse error.
var _T: Script

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1

func _has_static(name: String) -> bool:
	for m in _T.get_script_method_list():
		if String(m["name"]) == name:
			return true
	return false

## One row: `px` physical size, density, UI scale, touch, density known, the current class
## ("" none, "phone", "tablet"), then the expected phone flag, landscape flag and width class.
func _row(label: String, px: Vector2, density: float, ui: float, touch: bool, known: bool,
		cur: String, want_phone: bool, want_land: bool, want_wc: String) -> void:
	## A complete previous result, written out (not produced by `classify()`), with a deliberately
	## implausible size so a row that wrongly echoes it back shows up in the printed dp.
	var current := {}
	if cur != "":
		current = {"dp": Vector2.ZERO, "short_dp": 0.0, "aspect": 0.0, "phone": cur == "phone",
			"landscape": false, "width_class": ""}
	var d: Dictionary = _T.call("classify", px, density, ui, touch, known, current)
	var got := "phone=%s land=%s wc=%s dp=%s" % [str(d.get("phone")), str(d.get("landscape")),
		str(d.get("width_class")), str(d.get("dp"))]
	_ok(d.get("phone") == want_phone and d.get("landscape") == want_land and d.get("width_class") == want_wc,
		"%s -> want phone=%s land=%s wc=%s; got %s" % [label, str(want_phone), str(want_land), want_wc, got])

func _wc(width_dp: float, want: String) -> void:
	var got := String(_T.call("width_class_of", width_dp))
	_ok(got == want, "width_class_of(%s) -> want %s; got %s" % [str(width_dp), want, got])

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a in ["--table", "--force-touch"]:
			i += 1
		elif a in ["--live", "--vp", "--density"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	if "--live" in args:
		await _live(String(args[args.find("--live") + 1]))
		print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
		get_tree().quit(0 if _fails == 0 else 1)
		return
	if not "--table" in args:
		print("ABORT: pass --table or --live <leg>")
		get_tree().quit(2)
		return
	_T = load("res://shell/dcc_theme.gd")
	if _T == null or not _has_static("classify") or not _has_static("width_class_of"):
		print("  FAIL  DccTheme.classify() / width_class_of() do not exist (the tree before MS-1 S1)")
		print("RESULT: FAIL (1 failed, 1 checks)")
		get_tree().quit(2)
		return

	print("-- §2.2's fourteen windows, density known (a device or --density), UI scale 1.0")
	# 360 x 780 dp, 720 x 1 560 px at 2.
	_row("360x780dp @2 portrait", Vector2(720, 1560), 2.0, 1.0, true, true, "", true, false, "compact")
	_row("360x780dp @2 landscape", Vector2(1560, 720), 2.0, 1.0, true, true, "", true, true, "medium")
	# OnePlus 6T, 1 080 x 2 340 px at 2.625 = 411.4 x 891.4 dp.
	_row("6T @2.625 portrait", Vector2(1080, 2340), 2.625, 1.0, true, true, "", true, false, "compact")
	_row("6T @2.625 landscape", Vector2(2340, 1080), 2.625, 1.0, true, true, "", true, true, "expanded")
	# 480 x 1 040 dp, 1 440 x 3 120 px at 3.
	_row("480x1040dp @3 portrait", Vector2(1440, 3120), 3.0, 1.0, true, true, "", true, false, "compact")
	_row("480x1040dp @3 landscape", Vector2(3120, 1440), 3.0, 1.0, true, true, "", true, true, "expanded")
	# Foldable inner screen 673 x 841 dp, aspect 0.80, 1 346 x 1 682 px at 2: a tablet today.
	_row("foldable 673x841dp portrait", Vector2(1346, 1682), 2.0, 1.0, true, true, "", false, false, "medium")
	_row("foldable 841x673dp landscape", Vector2(1682, 1346), 2.0, 1.0, true, true, "", false, true, "expanded")
	# TABLET 800: 1 600 x 2 560 px at 2, aspect 0.625: a tablet today.
	_row("TABLET 800 portrait", Vector2(1600, 2560), 2.0, 1.0, true, true, "", false, false, "medium")
	_row("TABLET 800 landscape", Vector2(2560, 1600), 2.0, 1.0, true, true, "", false, true, "large")
	# 16:9, 1 080 x 1 920 px at 1.25 = 864 x 1 536 dp: aspect 0.5625 and short 864 < 900, a phone.
	_row("16:9 864dp portrait", Vector2(1080, 1920), 1.25, 1.0, true, true, "", true, false, "expanded")
	_row("16:9 864dp landscape", Vector2(1920, 1080), 1.25, 1.0, true, true, "", true, true, "large")
	# 1 600 x 2 560 px at 1.8 = 888.9 x 1 422 dp, aspect 0.625: a tablet today.
	_row("889dp portrait", Vector2(1600, 2560), 1.8, 1.0, true, true, "", false, false, "expanded")
	_row("889dp landscape", Vector2(2560, 1600), 1.8, 1.0, true, true, "", false, true, "large")
	# 11-inch class 2 388 x 1 668 px at 2 = 1 194 x 834 dp.
	_row("11in landscape", Vector2(2388, 1668), 2.0, 1.0, true, true, "", false, true, "expanded")
	_row("11in portrait", Vector2(1668, 2388), 2.0, 1.0, true, true, "", false, false, "medium")
	# 13-inch class 2 732 x 2 048 px at 2 = 1 366 x 1 024 dp.
	_row("13in landscape", Vector2(2732, 2048), 2.0, 1.0, true, true, "", false, true, "large")
	_row("13in portrait", Vector2(2048, 2732), 2.0, 1.0, true, true, "", false, false, "expanded")
	# 2 560 x 1 600 px at 1.6 = 1 600 x 1 000 dp.
	_row("2560x1600 @1.6 landscape", Vector2(2560, 1600), 1.6, 1.0, true, true, "", false, true, "xl")
	_row("2560x1600 @1.6 portrait", Vector2(1600, 2560), 1.6, 1.0, true, true, "", false, false, "expanded")
	# 4K at 2 = 1 920 x 1 080 dp: aspect 0.5625 but short 1 080 >= 900, a tablet.
	_row("4K @2 landscape", Vector2(3840, 2160), 2.0, 1.0, true, true, "", false, true, "xl")
	_row("4K @2 portrait", Vector2(2160, 3840), 2.0, 1.0, true, true, "", false, false, "expanded")
	# 4K at 1.5 = 2 560 x 1 440 dp.
	_row("4K @1.5 landscape", Vector2(3840, 2160), 1.5, 1.0, true, true, "", false, true, "xl")
	_row("4K @1.5 portrait", Vector2(2160, 3840), 1.5, 1.0, true, true, "", false, false, "large")
	# 4K at 1.
	_row("4K @1 landscape", Vector2(3840, 2160), 1.0, 1.0, true, true, "", false, true, "xl")
	_row("4K @1 portrait", Vector2(2160, 3840), 1.0, 1.0, true, true, "", false, false, "xl")

	print("-- UI scale 1.25 (spec §2.2: '4K at density 2 is 1 536 x 864 dp, a phone under either 900 dp rule')")
	_row("4K @2 x1.25 landscape", Vector2(3840, 2160), 2.0, 1.25, true, true, "", true, true, "large")
	_row("4K @2 x1.25 portrait", Vector2(2160, 3840), 2.0, 1.25, true, true, "", true, false, "expanded")
	# 4K at 1.5 x 1.25 = 2 048 x 1 152 dp: still a tablet.
	_row("4K @1.5 x1.25 landscape", Vector2(3840, 2160), 1.5, 1.25, true, true, "", false, true, "xl")
	# 11-inch at 2 x 1.25 = 955.2 x 667.2 dp: aspect 0.70, a tablet whatever its size.
	_row("11in x1.25 landscape", Vector2(2388, 1668), 2.0, 1.25, true, true, "", false, true, "expanded")
	_row("11in x1.25 portrait", Vector2(1668, 2388), 2.0, 1.25, true, true, "", false, false, "medium")
	# 6T x 1.25 = 329.1 x 713.1 dp: a phone.
	_row("6T x1.25 portrait", Vector2(1080, 2340), 2.625, 1.25, true, true, "", true, false, "compact")
	_row("6T x1.25 landscape", Vector2(2340, 1080), 2.625, 1.25, true, true, "", true, true, "medium")
	# 16:9 864 dp x 1.25 = 691.2 x 1 228.8 dp.
	_row("16:9 864dp x1.25 landscape", Vector2(1920, 1080), 1.25, 1.25, true, true, "", true, true, "large")

	print("-- OnePlus Pad 3: 3 392 x 2 400 px, 13.2 in, 7:5, ~315 ppi (aspect 0.7075)")
	# At density 2.0 (the 320 dpi bucket): 1 696 x 1 200 dp.
	_row("Pad 3 @2 landscape", Vector2(3392, 2400), 2.0, 1.0, true, true, "", false, true, "xl")
	_row("Pad 3 @2 portrait", Vector2(2400, 3392), 2.0, 1.0, true, true, "", false, false, "large")
	# At 315/160 = 1.96875: 1 722.9 x 1 219.0 dp.
	_row("Pad 3 @1.96875 landscape", Vector2(3392, 2400), 1.96875, 1.0, true, true, "", false, true, "xl")
	_row("Pad 3 @1.96875 portrait", Vector2(2400, 3392), 1.96875, 1.0, true, true, "", false, false, "large")
	# At 2 x 1.25: 1 356.8 x 960 dp.
	_row("Pad 3 @2 x1.25 landscape", Vector2(3392, 2400), 2.0, 1.25, true, true, "", false, true, "large")
	_row("Pad 3 @2 x1.25 portrait", Vector2(2400, 3392), 2.0, 1.25, true, true, "", false, false, "expanded")
	# Off a device, no --density (the legacy figure, aspect 0.7075 >= 0.6): a tablet.
	_row("Pad 3 legacy landscape", Vector2(3392, 2400), 1.0, 1.0, true, false, "", false, true, "xl")

	print("-- the size line: 899 / 900 / 901 dp at aspect 0.5625, density 1, from both sides")
	_row("899x1600 boot", Vector2(899, 1600), 1.0, 1.0, true, true, "", true, false, "expanded")
	_row("900x1600 boot", Vector2(900, 1600), 1.0, 1.0, true, true, "", false, false, "expanded")
	_row("901x1600 boot", Vector2(901, 1600), 1.0, 1.0, true, true, "", false, false, "expanded")
	_row("899x1600 from tablet", Vector2(899, 1600), 1.0, 1.0, true, true, "tablet", false, false, "expanded")
	_row("900x1600 from phone", Vector2(900, 1600), 1.0, 1.0, true, true, "phone", true, false, "expanded")
	_row("901x1600 from phone", Vector2(901, 1600), 1.0, 1.0, true, true, "phone", true, false, "expanded")
	print("-- the aspect line: 0.599 / 0.600 / 0.601, legacy density (aspect only), from both sides")
	_row("aspect .599 boot", Vector2(599, 1000), 1.0, 1.0, true, false, "", true, false, "compact")
	_row("aspect .600 boot", Vector2(600, 1000), 1.0, 1.0, true, false, "", false, false, "medium")
	_row("aspect .601 boot", Vector2(601, 1000), 1.0, 1.0, true, false, "", false, false, "medium")
	_row("aspect .599 from tablet", Vector2(599, 1000), 1.0, 1.0, true, false, "tablet", false, false, "compact")
	_row("aspect .600 from phone", Vector2(600, 1000), 1.0, 1.0, true, false, "phone", true, false, "medium")
	_row("aspect .601 from phone", Vector2(601, 1000), 1.0, 1.0, true, false, "phone", true, false, "medium")

	print("-- B5's hysteresis walk (900 x 1 600 dp -> 890 x 1 582 -> 880 x 1 564 -> 900 x 1 600 -> 920 x 1 636)")
	_row("900x1600 boot", Vector2(900, 1600), 1.0, 1.0, true, true, "", false, false, "expanded")
	_row("890x1582 from tablet", Vector2(890, 1582), 1.0, 1.0, true, true, "tablet", false, false, "expanded")
	_row("880x1564 from tablet", Vector2(880, 1564), 1.0, 1.0, true, true, "tablet", true, false, "expanded")
	_row("900x1600 from phone", Vector2(900, 1600), 1.0, 1.0, true, true, "phone", true, false, "expanded")
	_row("920x1636 from phone", Vector2(920, 1636), 1.0, 1.0, true, true, "phone", false, false, "expanded")

	print("-- off a device and on a pointer build")
	# Legacy figure: today's off-device rule is the aspect alone (spec §2.2, last paragraph).
	_row("4K legacy touch", Vector2(3840, 2160), 1.0, 1.0, true, false, "", true, true, "xl")
	_row("1080x2340 legacy touch", Vector2(1080, 2340), 2.6214, 1.0, true, false, "", true, false, "compact")
	_row("720x1560 pointer", Vector2(720, 1560), 2.0, 1.0, false, true, "", false, false, "compact")
	_row("1920x1080 pointer", Vector2(1920, 1080), 1.0, 1.0, false, false, "", false, true, "xl")
	# A degenerate size keeps the current class, and answers not-phone without one, with no width
	# class at all: "" is "not classified", never a plausible class (`MISTAKES.md`, "Return, marshal or
	# default a value that can be absent").
	_row("0x0 boot", Vector2(0, 0), 1.0, 1.0, true, true, "", false, false, "")
	_row("0x0 from phone", Vector2(0, 0), 1.0, 1.0, true, true, "phone", true, false, "")

	print("-- width classes at their bounds (proposal §3: 600, 840, 1 200, 1 600)")
	_wc(0.0, "compact")
	_wc(599.9, "compact")
	_wc(600.0, "medium")
	_wc(839.9, "medium")
	_wc(840.0, "expanded")
	_wc(1199.9, "expanded")
	_wc(1200.0, "large")
	_wc(1599.9, "large")
	_wc(1600.0, "xl")

	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


# ---- live legs (B4 and S3's swap request) ----------------------------------------------------------

var _vp: SubViewport
var _app: Node

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	if k >= 0 and k + 1 < args.size():
		return String(args[k + 1])
	return dflt

## Resizes the host and tells the shell, as a device's root resize would, then waits past the shell's
## 150 ms debounce (0.4 s, so a settled result is read, not a mid-debounce one).
func _resize(w: int, h: int) -> void:
	_vp.size = Vector2i(w, h)
	await _frames(2)
	get_tree().root.size_changed.emit()
	await get_tree().create_timer(0.4).timeout
	await _frames(3)

func _live(leg: String) -> void:
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		_ok(false, "--live needs --vp WxH")
		return
	var vw := int(parts[0])
	var vh := int(parts[1])
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	match leg:
		"phone":
			## B4, phone half: rotating 1080 x 2340 at 2.625 gives the landscape layout without a rebuild.
			_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone")
			var id: int = _app._phone_root.get_instance_id() if _app._phone_root != null else 0
			await _resize(2340, 1080)
			_ok(_app._landscape and DccTheme.is_phone(), "rotated: landscape, still a phone")
			_ok(_app._phone_root != null and _app._phone_root.get_instance_id() == id,
				"the composition root is the same instance (no rebuild)")
			## The published classification follows the rotation. Before S3 the phone's own re-run of
			## `_compute_layout_mode()` already published it, so this check guards rather than pins.
			_ok(String(DccTheme.layout().get("width_class", "")) == "expanded",
				"published width class after rotation is 'expanded' (2340 / 2.625 = 891 dp); got '%s'" %
				String(DccTheme.layout().get("width_class", "")))
		"tablet":
			## B4, tablet half: a dock dragged to 320 dp keeps it through a same-orientation resize.
			_ok(DccTheme.is_tablet(), "boots as a tablet")
			_app._left_width = 320.0
			_app._apply_dock_widths()
			await _frames(4)
			var before: float = _app.left_dock.size.x
			await _resize(2200, 1668)
			var after: float = _app.left_dock.size.x
			_ok(absf(before - 320.0) <= 0.5 and absf(after - 320.0) <= 0.5,
				"left dock 320 dp before (%.2f) and after (%.2f) a 2388 -> 2200 px resize, +-0.5" % [before, after])
			## S3: the unit is re-applied to the new size (fails before S3: the override keeps 1194 x 834).
			_ok(_vp.size_2d_override == Vector2i(1100, 834),
				"the unit follows the resize: override %s, want (1100, 834)" % str(_vp.size_2d_override))
			_ok(String(DccTheme.layout().get("width_class", "")) == "expanded",
				"published width class 'expanded' (1100 dp); got '%s'" % String(DccTheme.layout().get("width_class", "")))
		"swap":
			## S3's class question with B5's hysteresis walk, density 1 (900 x 1 600 dp). Until S4 the
			## shell keeps its boot class and logs a request; the runner greps the log for it.
			_ok(DccTheme.is_tablet(), "boots 900 x 1600 dp as a tablet")
			await _resize(890, 1582)
			_ok(not bool(DccTheme.layout().get("phone", true)) and DccTheme.is_tablet(),
				"890 x 1582 stays a tablet (inside the 16 dp band), no request")
			await _resize(880, 1564)
			## The settled classification says phone; the published layout is not overwritten by a
			## request, and the composition is still the boot class.
			_ok(DccTheme.is_tablet(), "880 x 1564: still the boot composition (S4 not built)")
			print("LIVE swap step 880x1564 done")
		_:
			_ok(false, "unknown --live leg " + leg)
