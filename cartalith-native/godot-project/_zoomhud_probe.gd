extends Node
## FIX-VIEWPORT, 2026-09-07. The top-right readout against `Cartalith DCC
## Environment.dc.html` -- `ENV:913` for the composition and
## `cartalith-dcc-parts.js:221` for the zoom segment's writer,
## `'zoom '+Math.round(v.s*100)+'%'`.
##
##   godot --headless --path . _zoomhud_probe.tscn -- --vp 1920x1080 --tag pc
##   godot --headless --path . _zoomhud_probe.tscn -- --vp 2560x1600 --tag tab --force-touch
##
## **Headless is correct for this probe and stated rather than assumed**
## (`MISTAKES.md`, "write `--headless` into a brief"): every assertion below is
## label text or an integer, nothing rasterises and nothing is timed. It reads
## the flags it documents -- an unknown `--` argument is a hard failure, not a
## silent default.
##
## The zoom segment is recomputed here from the public `zoom()` accessor rather
## than from `_update_zoom_readout()`'s own expression, so replacing `100.0`,
## `roundi` or `%d` in that function makes this probe red.

var app: Node
var _vp: SubViewport
var _tag := "pc"
var _fail := 0
var _pass := 0

const KNOWN := ["--vp", "--tag", "--force-touch"]

## The retired `z1.1` form: a `z` immediately followed by a digit. Built once.
var _ZRE := RegEx.create_from_string("z[0-9]")

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	return String(a[i + 1]) if i >= 0 and i + 1 < a.size() else dflt

func _has(name: String) -> bool:
	return OS.get_cmdline_user_args().has(name)

func _ok(what: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if good:
		_pass += 1
	else:
		_fail += 1
	print("  %-4s %-52s got=%s want=%s" % ["ok" if good else "FAIL", what, str(got), str(want)])

## `MISTAKES.md`: a probe header is a claim about the probe's own code, and a
## harness must fail loudly on an argument it does not understand.
func _check_args() -> void:
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		var tok := String(a[i])
		if not KNOWN.has(tok):
			print("  FAIL unknown argument %s -- known: %s" % [tok, str(KNOWN)])
			_fail += 1
			i += 1
			continue
		i += 2 if tok != "--force-touch" else 1

## The canvas's own writer, transcribed once, here, and used as the oracle.
func _want_zoom_seg(z: float) -> String:
	return "zoom %d%%" % roundi(z * 100.0)

func _segments(vh) -> PackedStringArray:
	return String(vh._readout_label.text).split(" · ")

## **Never index the split directly.** A mutant that removes a segment made
## `_segments(vh)[3]` go out of range; the error aborts `_ready()`'s coroutine,
## `quit()` never runs and `--headless` **hangs** rather than failing -- one
## mutation run lost to a 600 s timeout (`MISTAKES.md`, "a failed script load
## makes `godot --headless` hang"). `_watchdog()` below is the belt to this
## brace.
func _seg(vh, i: int) -> String:
	var p := _segments(vh)
	return String(p[i]) if i < p.size() else "<missing>"

## Any path that never reaches `[RESULT]` -- a runtime error inside the
## coroutine, a generate that never settles -- exits non-zero instead of hanging
## a harness for its full timeout.
func _watchdog(seconds: float) -> void:
	await get_tree().create_timer(seconds).timeout
	print("[RESULT] %s WATCHDOG after %.0fs pass=%d failures=%d"
		% [_tag, seconds, _pass, _fail + 1])
	get_tree().quit(2)

func _assert_line(vh, when: String) -> void:
	var z: float = vh.zoom()
	print("     [%s] zoom=%.4f text='%s'" % [when, z, vh._readout_label.text])
	_ok("%s: segment 0 is the projection" % when, _seg(vh, 0), "equirect")
	_ok("%s: segment 1 is the canvas's zoom string" % when,
		_seg(vh, 1), _want_zoom_seg(z))
	_ok("%s: segment 2 is the field" % when,
		_seg(vh, 2), vh._vp_field)
	## A `%.1f` or a `z` prefix regression fails here even if the digits agree.
	## `contains(" z")` was the first form of the check and it matched " zoom"
	## -- it failed on correct output. A digit must follow the `z`.
	_ok("%s: zoom segment carries no decimal point" % when,
		String(_seg(vh, 1)).contains("."), false)
	_ok("%s: no retired 'z<digit>' multiplier form" % when,
		_ZRE.search(String(vh._readout_label.text)) != null, false)
	_ok("%s: single line" % when, String(vh._readout_label.text).contains("\n"), false)

func _ready() -> void:
	_watchdog(300.0)
	_check_args()
	var parts: PackedStringArray = _arg("--vp", "1920x1080").split("x")
	_tag = _arg("--tag", "pc")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	if _has("--force-touch"):
		Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(4)

	var bridge = app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.2).timeout
	await _frames(8)

	var vh = app.viewport
	print("=== %s vp=%s touch=%s phone=%s ===" % [_tag, str(_vp.size),
		str(DccTheme.is_touch()), str(DccTheme.is_phone())])

	# -- A. The rest view -----------------------------------------------------
	## **`_zoom` is NOT 1.0 at rest, and this probe's first draft asserted that
	## it was.** `reset_view()` sets `_zoom = cover`, `clampf(max(size.x/rect.x,
	## size.y/rect.y), 1.0, _zoom_max)` -- its own comment says `_zoom == 1` is
	## the *letterbox-fit* rect and that the owner ruled on 2026-08-23 that reset
	## covers instead. Measured 1.0831 at 1920x1080 over a 256x192 grid, so the
	## corner reads `zoom 108%`. Recorded here because the readout's doc comment
	## had the same wrong number in it until this run.
	print("-- A. rest view (ENV:913: equirect · zoom NN% · field)")
	_assert_line(vh, "rest")
	var z0: float = vh.zoom()
	_ok("rest zoom is reset_view()'s cover, >= the fit rect", z0 >= 1.0, true)

	# -- B. A non-integral zoom, where rounding is the whole claim -------------
	## `zoom_step` is `clampf(_zoom * factor, …)`, so the oracle is z0-relative.
	print("-- B. after two wheel notches (1.15^2)")
	vh.zoom_step(1.15)
	vh.zoom_step(1.15)
	await _frames(2)
	_assert_line(vh, "z^2")
	_ok("independent 1.15^2 recomputation", "%.4f" % vh.zoom(), "%.4f" % (z0 * pow(1.15, 2)))

	# -- C. A value where round() and floor() differ --------------------------
	## The whole point of porting `Math.round(v.s*100)` rather than a truncation.
	## Aimed at 1.749 -> round 175, floor 174; the precondition below is the
	## positive control, so a landing that does not separate the two fails loudly
	## instead of passing vacuously. `Math.round` and `roundi` agree here because
	## `_zoom` is positive throughout (`ZOOM_MIN` is 0.4).
	print("-- C. round vs floor (target 1.749…)")
	vh.zoom_step(1.749 / vh.zoom())
	await _frames(2)
	var zc: float = vh.zoom()
	_ok("precondition: round and floor disagree at this zoom",
		roundi(zc * 100.0) != floori(zc * 100.0), true)
	_assert_line(vh, "z1.749")
	_ok("rounds up, does not truncate", _seg(vh, 1),
		"zoom %d%%" % roundi(zc * 100.0))
	_ok("is not the truncated value", _seg(vh, 1) == "zoom %d%%" % floori(zc * 100.0), false)

	# -- D. The fourth segment -- A KNOWN DEPARTURE, pinned as such ------------
	## **These two assertions pin DRIFT, deliberately, and say so in their own
	## text -- which is the whole of the ruling.** An assertion that pins a
	## departure without naming it is the trap: the follow-up that removes the
	## segment then reads as a probe regression and gets reverted.
	##
	## The canvas: `ENV:913` writes the readout as exactly
	## `equirect · <span ref=zoomRef>zoom 100%</span> · {{ vpField }}` -- **three**
	## `·` segments, no preset among them. `ENV:1571` (`statusExtra()`) is where
	## the preset belongs: `'style edited — layers differ from preset '+
	## this.ca().preset`, in the STATUS BAR. The shipped line carries four.
	##
	## Kept for now rather than asserted at three, because a probe that is red on
	## a clean tree teaches nothing: removing the segment means editing
	## `viewport_host.gd::_update_zoom_readout()` and re-homing
	## `render_workspace.gd`'s two `set_style_readout()` calls onto the status
	## bar, neither of which this lane owns. **FIX-VIEWPORT lane ruling,
	## 2026-09-07: when that removal lands, the correct edit here is to assert
	## `size == 3` and delete the segment-3 line -- NOT to restore the segment.**
	## Sections A/B/C/E are the parity assertions; these two are the bookmark.
	print("-- D. the style-preset segment (KNOWN DEPARTURE from ENV:913)")
	vh.set_style_readout("Antique")
	await _frames(2)
	_ok("KNOWN DEPARTURE (ENV:913 draws 3): preset appends as a 4th segment",
		_segments(vh).size(), 4)
	_ok("KNOWN DEPARTURE (ENV:1571 homes it in the status bar): 4th is the preset",
		_seg(vh, 3), "Antique")
	vh.set_style_readout("")
	await _frames(2)
	_ok("empty preset drops the segment, no trailing separator",
		_segments(vh).size(), 3)
	_ok("no dangling ' · '", String(vh._readout_label.text).ends_with("·"), false)
	vh.set_style_readout("Default")

	# -- E. The field segment tracks the layer, not a constant ----------------
	print("-- E. field segment")
	var before: String = _seg(vh, 2)
	vh.set_debug_layer("slope")
	await _frames(2)
	_ok("field moved with the layer", _seg(vh, 2) != before, true)
	_assert_line(vh, "layer")

	print("[RESULT] %s pass=%d failures=%d" % [_tag, _pass, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
