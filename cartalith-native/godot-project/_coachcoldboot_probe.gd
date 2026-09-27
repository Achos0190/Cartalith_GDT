extends Node
## `OUTSTANDING_WORK.md`, "the first coach mark fires under the phone project
## picker at cold boot" -- found 2026-09-28 by the drag-hint lane running
## `_pandraghint_probe.gd`: on a genuine cold boot (`bottombar_tabs` unseen,
## no world yet), `dcc_shell.gd::_maybe_show_coach_marks()` used to start the
## coach-mark sequence one frame after the phone chrome was built, entirely
## independent of `phone_project_picker.gd`'s own full-screen entry screen --
## which then opens a few frames LATER (`app.gd::_open_welcome_when_drawn()`'s
## own two-`process_frame`-plus-`frame_post_draw` wait) and, being an embedded
## `Window`, always composites above the ordinary phone canvas the toast draws
## into (that file's own "Lane GATE part B" header). So `bottombar_tabs`
## painted, and was then immediately buried under the picker for the rest of
## its 3.2 s display, on every fresh install.
##
## This probe drives the REAL cold-boot sequence -- clears any prior "seen"
## state so the sequence is live, boots `app.tscn` with no world, and checks
## two things read off LIVE nodes, never a re-declared expectation:
##
##   (1) while `phone_project_picker` is genuinely open (`.visible == true`,
##       `bridge.has_world == false`), no coach-mark toast exists anywhere
##       under `_phone_root` at all -- not merely hidden-but-present, absent.
##       The fix holds the sequence off rather than starting it early and
##       relying on z-order, so "does not exist yet" is the correct assertion,
##       not "exists but is covered" (`MISTAKES.md`'s "Claim something covered
##       is now visible" row is about exactly that weaker, insufficient check).
##   (2) after a real world generation finishes (which is what actually hides
##       the picker, `phone_project_picker.gd`'s own `bridge.generation_
##       finished` wiring), the FIRST coach mark (`bottombar_tabs`) appears,
##       confirming the sequence was armed and waiting, not silently dropped.
##
## Screenshots both moments to `CTXPHONE`-style env-gated dir (see `_shot()`)
## so a human can look at the picker itself and confirm nothing is drawn
## behind/through it, and at the toast once it is actually the front-most
## thing on screen.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _coachcoldboot_probe.tscn -- --vp 1080x2340 --force-touch
##
## Windowed on purpose, not `--headless`: `--headless` never reaches the real
## render path some of this leans on for screenshots, and (MISTAKES.md)
## `ImageTexture.update()` is a no-op there.
##
## Protects: a future edit to `_maybe_show_coach_marks()` / `_start_or_defer_
## coach_marks()` / `_on_phone_entry_screen_visibility()` (`dcc_shell.gd`) or
## to `phone_project_picker.gd`'s `visibility_changed` forwarding that
## reintroduces the race -- either by starting the sequence unconditionally
## again, or by never starting it at all once the picker closes.

const BOTTOMBAR_TEXT := "MAP · GENERATE · PLAN switch tasks here — MORE reaches everything else."
const SHEETHANDLE_TEXT := "Drag this handle to expand tool options."
const HINT_TEXT := "A one-finger drag only moves the map while Pan (✋) is armed."
const COACH_TEXTS := [BOTTOMBAR_TEXT, SHEETHANDLE_TEXT, HINT_TEXT]

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[coachcoldboot] ", s)

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

## Any coach-mark toast anywhere under `root`, by exact text -- existence, not
## visibility: the whole point of assertion (1) above is that the fixed code
## has not yet CREATED the Label at all while the picker is open, so mere
## presence in the tree is already the right-shaped check here (unlike
## `_pandraghint_probe.gd`'s legibility checks, which are about something
## already on screen).
func _find_any_coach_toast(root: Node) -> Label:
	if root is Label and String((root as Label).text) in COACH_TEXTS:
		return root as Label
	for c in root.get_children():
		var found := _find_any_coach_toast(c)
		if found != null:
			return found
	return null

func _shot(app: Node, tag: String) -> void:
	var dir := OS.get_environment("COACHCOLDBOOT_SHOT_DIR")
	if dir == "":
		return
	await get_tree().process_frame
	var img: Image = get_viewport().get_texture().get_image()
	img.save_png(dir.path_join("coachcoldboot_%s.png" % tag))
	_log("saved screenshot: %s" % dir.path_join("coachcoldboot_%s.png" % tag))

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 60.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var vp_size := Vector2i(1080, 2340)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a: String = args[i]
		if a == "--vp" and i + 1 < args.size():
			var wh := args[i + 1].split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
			continue
		if a == "--force-touch":
			i += 1
			continue
		print("### COACHCOLDBOOT ABORT: unknown argument '%s' ###" % a)
		get_tree().quit(2)
		return

	DisplayServer.window_set_size(vp_size)
	get_window().size = vp_size
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	## Clear ANY prior "seen" state for the three coach marks so this run sees
	## the real first-launch sequence, not a config file a prior probe run
	## left behind on this machine (`DccSettings.CONFIG_PATH` is `user://`,
	## which persists across runs) -- same clearing `_pandraghint_probe.gd`
	## does, for the same reason.
	var settings_script = load("res://shell/dcc_settings.gd")
	var cfg := ConfigFile.new()
	cfg.load(settings_script.CONFIG_PATH)
	if cfg.has_section("coach_marks"):
		cfg.erase_section("coach_marks")
	cfg.save(settings_script.CONFIG_PATH)

	## Genuine cold boot: instantiate `app.tscn` fresh, with no world loaded,
	## and do NOT touch `phone_project_picker` -- that is the entry screen this
	## whole probe is about, and dismissing it early (the way `_pandraghint_
	## probe.gd` does for an unrelated reason) would defeat the point.
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	await _frames(8)

	if not app.is_phone():
		_log("ABORT not phone -- pass --force-touch")
		get_tree().quit(2)
		return

	var picker: Node = app.phone_project_picker
	_check(picker != null, "phone_project_picker exists on a phone boot")
	if picker == null:
		get_tree().quit(1)
		return

	## Fixture precondition, not the finding: this only tests the real bug if
	## the picker is genuinely up and no world exists yet.
	_check(not bool(app.bridge.has_world), "fixture: no world exists yet")
	_check(bool(picker.visible), "fixture: the phone project picker is genuinely open")

	var toast := _find_any_coach_toast(app._phone_root)
	_check(toast == null,
		"no coach-mark toast exists anywhere while the picker is open (was: bottombar_tabs painted under it)")
	await _shot(app, "01_picker_open_no_coach_mark")

	## Dismiss the picker the REAL way: generate a world, which fires
	## `bridge.generation_finished(true)` -- the same signal `phone_project_
	## picker.gd::setup()` already wires to hide itself -- rather than calling
	## `.hide()` on it directly, so this exercises the actual dismissal path
	## end to end, not a shortcut that only proves the signal handler works.
	var bridge = app.bridge
	bridge.generate({
		"seed": 552017, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(2)

	_check(not bool(picker.visible), "the picker actually closed after generation")

	## Poll for the first coach mark to appear -- bounded, not a fixed sleep,
	## the same discipline `_pandraghint_probe.gd::_await_full_opacity()` uses.
	var started := false
	for tries in 200:   ## ~20s at 3-frame steps
		var lbl := _find_any_coach_toast(app._phone_root)
		if lbl != null and String(lbl.text) == BOTTOMBAR_TEXT:
			started = true
			break
		await _frames(3)
	_check(started, "the coach-mark sequence started (bottombar_tabs) once the picker closed")
	await _shot(app, "02_after_dismiss_coach_mark_started")

	_log("RESULT %s failures=%d" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)
