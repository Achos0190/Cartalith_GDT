extends Node
## Owner, 2026-11-14, OnePlus Pad 3: "On the tab it seems frozen." Tapping a world card on the
## welcome screen opened the world synchronously: `open_project_dialog.gd::_pick_path()` called
## `open_recent_project()`, which blocks the main thread for the whole open including the map's first
## repaint, so the welcome stayed on screen exactly as it was, with no sign the tap was taken, for the
## whole load (adb on the Pad, build `df4d3d18`: about 10 s with every screenshot byte-identical).
## The fix sends both pickers' world cards through `DccApp.open_project_busy()`, which paints the busy
## card before the open starts.
##
## **What this probe checks, the way the device runs it.** The app is mounted DIRECTLY under the window
## root, not in a `SubViewport` like most probes, because `app.gd::_enable_idle_mode()` turns on
## `OS.low_processor_usage_mode` only when the app owns the window, and the device runs with it on. A
## fixture world is saved into the scratch projects root before boot so the welcome lists it; the probe
## waits for the welcome (the desktop/tablet `OpenProjectDialog`, or the phone's `PhoneProjectPicker`
## under `--force-touch` at a phone size), then clicks the fixture's card with a REAL mouse event pushed
## into the root viewport at the card's centre, so the event takes the shipped path (embedded-window
## forwarding, then the card's own `gui_input`).
##
## Assertions:
##   1. the open ran under the busy card: `busy_overlay.work_started` fired with the overlay visible;
##   2. at least one frame was DRAWN between the click and the start of the blocking work, so the card
##      reached the screen before the main thread blocked (`Engine.get_frames_drawn()`);
##   3. the card itself is on screen over the welcome at that moment (its panel colour fills the window's
##      centre), not the scrim alone: the phone picker is presented at the phone scale, and the overlay's
##      window mirror scaled the card a second time, so on the 6T only the scrim was drawn;
##   4. the picker is hidden afterwards and `current_project_path` is the fixture (the open succeeded).
## Mutations, measured 2026-11-14: origin's `open_project_dialog.gd` fails 1, 2 and 3 at the Pad size (the open
## never goes through `busy_run`); origin's `busy_overlay.gd` fails 3 at the 6T size (panel share 0.00, 1.00 fixed).
##
## Run (WINDOWED: headless draws no frames, so assertion 2 cannot be measured headless), under the
## `C:/cw/.win_lock` convention and with a scratch `APPDATA` so `user://` and the projects root are not
## the owner's:
##   APPDATA=C:/cw/<scratch> godot --path . _welcomeopen_probe.tscn -- --force-touch            (tablet, 1600x1000)
##   APPDATA=C:/cw/<scratch> godot --path . _welcomeopen_probe.tscn -- --force-touch --size 450x1000   (phone)
##   APPDATA=C:/cw/<scratch> godot --path . _welcomeopen_probe.tscn                             (desktop)
##   ... -- --force-touch --density 2.225 --size 2400x3392   (the Pad 3 in portrait, tablet class)
##   ... -- --force-touch --size 1080x2340                    (the 6T, phone class: the size the card-on-screen
##                                                             mutation was measured at; smaller phone windows have a
##                                                             smaller phone scale and were not measured against it)
## The window may be taller than the monitor; Windows accepts it and the capture reads the whole frame.
## Result: `  OK  ` / `  FAIL  ` rows, a `[BOOT]` line naming the composition and the idle mode, a `[CLICK]` line
## with the click point and the welcome window's rect, and
## `_welcomeopen_probe (<composition>): <n> failure(s)`; exit 0 when none failed, 1 otherwise or on a
## `[FATAL]` (no extension, no welcome within the wait, no card, fixture not written) or the 90 s
## hard timeout. It deletes its fixture save on every path that reaches the end.

## How long the whole run may take before it gives up with exit 1. A labelled judgement: a cold boot
## plus a 64 x 48 fixture open is a few seconds on this machine; 90 s only guards against a hang.
const HARD_TIMEOUT_S := 90.0
## How long the welcome may take to appear after boot (it waits for a drawn frame, `app.gd::
## _open_welcome_when_drawn`). A labelled judgement, generous for a cold debug start.
const WELCOME_WAIT_S := 20.0

## The fixture world's seed; the card is found by it. Any seed works; this is `_openfailgate_probe.gd`'s.
const FIXTURE_SEED := 1310

var _fail := 0
var _fixture := ""
var _app: Node
var _click_frame := -1
var _work_frame := -1
var _work_overlay_visible := false
var _work_seen := false

func _ok(cond: bool, msg: String) -> void:
	print(("  OK    " if cond else "  FAIL  ") + msg)
	if not cond:
		_fail += 1

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if i >= 0 and i + 1 < args.size() else dflt

func _finish(code: int) -> void:
	if _fixture != "" and FileAccess.file_exists(_fixture):
		DirAccess.remove_absolute(_fixture)
	print("\n_welcomeopen_probe (%s): %d failure(s)" % [_composition(), _fail])
	get_tree().quit(code)

func _composition() -> String:
	if _app == null:
		return "none"
	return "phone" if bool(_app.call("is_phone")) else ("tablet" if DccTheme.is_tablet() else "desktop")

## The surface the welcome is shown on in this composition: the phone's picker, else the dialog.
func _welcome() -> Window:
	if bool(_app.call("is_phone")) and _app.get("phone_project_picker") != null:
		return _app.get("phone_project_picker") as Window
	return _app.get("open_project_dialog") as Window

## The fixture's card inside `win`: the Control above a Label that begins with the fixture's seed
## (the card's meta line, "1310 · fmt 2 · edited just now"; its title is the generated world name,
## which this probe does not pin) and that listens to `gui_input` (the card wrapper both pickers
## build). Null when absent. The scratch projects root holds this fixture only, so the seed is unique.
func _find_card(win: Node) -> Control:
	for n in win.find_children("*", "Label", true, false):
		if not String((n as Label).text).begins_with("%d " % FIXTURE_SEED):
			continue
		var c: Node = n.get_parent()
		while c != null and c != win:
			if c is Control and not (c as Control).gui_input.get_connections().is_empty():
				return c as Control
			c = c.get_parent()
	return null

## The share of pixels in a 160 x 40 box at the welcome window's centre that are the card's panel colour,
## read from the frame on screen when the blocking work starts. The card is centred in the window, so a
## drawn card covers that box with its fill (and some text); a scrim alone covers it with the dimmed
## picker. A labelled judgement for the bar: 0.3, well under a drawn card's share (0.89 to 1.00 across the five
## sizes in the header) and well over a scrim's (0.00 at the 6T size before the fix).
const CARD_SHARE_MIN := 0.3
var _card_share := -1.0
var _win: Window

func _on_work_started(_label: String) -> void:
	_work_seen = true
	_work_frame = Engine.get_frames_drawn()
	_work_overlay_visible = bool(_app.get("busy_overlay").visible)
	var img := get_viewport().get_texture().get_image()
	var panel: Color = DccTheme.c("panel")
	## The captured image is in window pixels; the window's rect is in root-viewport coordinates, which on the
	## tablet are the window pixels over the content scale (2.225 at the Pad 3's density).
	var k := float(img.get_width()) / maxf(1.0, get_viewport().get_visible_rect().size.x)
	var c := Vector2i((Vector2(_win.position) + Vector2(_win.size) * 0.5) * k)
	var hit := 0
	var total := 0
	for y in range(c.y - 20, c.y + 20):
		for x in range(c.x - 80, c.x + 80):
			if x < 0 or y < 0 or x >= img.get_width() or y >= img.get_height():
				continue
			total += 1
			var p := img.get_pixel(x, y)
			if absf(p.r - panel.r) + absf(p.g - panel.g) + absf(p.b - panel.b) < 0.03:
				hit += 1
	_card_share = float(hit) / float(maxi(1, total))

func _ready() -> void:
	get_tree().create_timer(HARD_TIMEOUT_S).timeout.connect(func():
		print("[FATAL] hard timeout after %.0f s" % HARD_TIMEOUT_S); _finish(1))
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var size := _arg("--size", "1600x1000").split("x")
	get_window().size = Vector2i(int(size[0]), int(size[1]))

	## The fixture: a tiny real save in the (scratch) projects root, so the welcome lists a card for it.
	var root := DccSettings.storage_root("projects")
	DirAccess.make_dir_recursive_absolute(root)
	_fixture = root.path_join("_welcomeopen_fixture.ctl")
	var wg: WorldGen = WorldGen.new()
	wg.set_params({"tect.plates": 6})
	wg.generate_sized(FIXTURE_SEED, 640.0, 64, 48)
	var saved: Dictionary = wg.project_save(_fixture)
	if not bool(saved.get("ok", false)) or not FileAccess.file_exists(_fixture):
		print("[FATAL] fixture not written: ", saved); _finish(1); return
	## The phone picker lists the recents list, not the folder; the desktop welcome reads both. The
	## write lands in the scratch `user://` settings this run is given, never the owner's.
	DccSettings.remember_project(_fixture)

	_app = load("res://shell/app.tscn").instantiate()
	get_tree().root.add_child.call_deferred(_app)
	var t0 := Time.get_ticks_msec()
	var win: Window = null
	while Time.get_ticks_msec() - t0 < WELCOME_WAIT_S * 1000.0:
		await get_tree().process_frame
		if _app.is_inside_tree() and _app.get("bridge") != null:
			win = _welcome()
			if win != null and win.visible:
				break
	print("[BOOT] composition=%s window=%s low_processor=%s" % [_composition(), str(get_window().size),
		str(OS.low_processor_usage_mode)])
	if win == null or not win.visible:
		print("[FATAL] no welcome within %.0f s" % WELCOME_WAIT_S); _finish(1); return
	_ok(OS.low_processor_usage_mode, "precondition: idle mode is on, as on the device")
	## Two frames so the cards' layout has settled before the card's rect is read.
	await get_tree().process_frame
	await get_tree().process_frame
	var card := _find_card(win)
	if card == null:
		var seen: Array[String] = []
		for n in win.find_children("*", "Label", true, false):
			seen.append(String((n as Label).text))
		print("[FATAL] no card for the fixture on the welcome; labels: ", seen); _finish(1); return

	_win = win
	_app.get("busy_overlay").work_started.connect(_on_work_started)
	## A real press and release at the card's centre, in root-viewport coordinates (an embedded window's
	## `position` is in its parent viewport's space, and `get_final_transform()` carries its content scale:
	## the phone picker is drawn at the phone scale), pushed through the root so the shipped forwarding runs.
	var at := Vector2(win.position) + win.get_final_transform() * card.get_global_rect().get_center()
	print("[CLICK] at %s; welcome window at %s size %s content_scale %.3f; root visible rect %s" % [str(at),
		str(win.position), str(win.size), win.content_scale_factor, str(get_viewport().get_visible_rect().size)])
	_click_frame = Engine.get_frames_drawn()
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		ev.position = at
		ev.global_position = at
		## `in_local_coords`: `at` is in the root viewport's own coordinates. On the tablet the root carries the
		## content scale (2.225 at the Pad 3's density), so a window-pixel event would land 2.225 times too close
		## to the origin.
		get_viewport().push_input(ev, true)

	var t1 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t1 < 20000 and (win.visible or not bool(_app.get("bridge").has_world)):
		await get_tree().process_frame
	_ok(_work_seen and _work_overlay_visible,
		"the open ran under the busy card (work_started fired: %s, overlay visible then: %s)" % [
			str(_work_seen), str(_work_overlay_visible)])
	_ok(_work_seen and _work_frame - _click_frame >= 1,
		"a frame was drawn between the tap and the blocking open (%s drawn)" % (
			str(_work_frame - _click_frame) if _work_seen else "n/a: no busy run"))
	_ok(_card_share >= CARD_SHARE_MIN,
		"the card itself is on screen over the welcome, not the scrim alone (panel share %.2f, bar %.2f)" % [
			_card_share, CARD_SHARE_MIN])
	_ok(not win.visible, "the welcome is hidden after the open")
	_ok(String(_app.get("current_project_path")) == _fixture,
		"the fixture is the open project (got '%s')" % String(_app.get("current_project_path")))
	_finish(1 if _fail > 0 else 0)
