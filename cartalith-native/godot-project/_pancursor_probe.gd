extends Node
## The runnable check behind `ViewportHost.set_pan_mode()`'s cursor-shape line
## (`OUTSTANDING_WORK.md` "A drag on the map does nothing until the hand tool
## is armed, with no on-screen cue", 2026-09-13 -- the desktop half: a cursor
## that shows DRAG while the pan latch is armed and ARROW while it is not, the
## cue the row's own text names ("On a phone there is no cursor to change
## shape, which is the desktop's cue") and that this file had never actually
## set.
##
##   Godot_v4.7.1-stable_win64.exe --headless --path . _pancursor_probe.tscn
##
## Headless is correct here, not a shortcut: this reads a `Control` PROPERTY
## (`mouse_default_cursor_shape`), never a rendered pixel or a live viewport
## texture, so none of the pixel-probe headless blockers (`RenderingServer.
## frame_post_draw` never firing) apply. No world needs to be generated
## either -- `overlay` is built by `ViewportHost.setup()`, called from
## `app.gd` well before any `_run_pipeline()` -- so this probe never starts
## `bridge.generating` at all and stays fast.

var app: Node
var vp: Control

func _p(s: String) -> void:
	print("PANCURSOR  %s" % s)

var _fails := 0
func _bad(s: String) -> void:
	_fails += 1
	_p("FAIL  %s" % s)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	for f in 6:
		await get_tree().process_frame
	vp = app.viewport
	if vp == null or vp.overlay == null:
		_bad("app.viewport or app.viewport.overlay is null after boot -- nothing to check")
		_p("RESULT: FAIL -- %d check(s) failed" % _fails)
		get_tree().quit(1)
		return

	## Baseline: force a known starting state rather than assume boot leaves
	## one -- this probe's claim is about what `set_pan_mode()` DOES, not
	## about what state boot happens to leave.
	vp.set_pan_mode(false)
	var baseline: int = vp.overlay.mouse_default_cursor_shape
	_p("pan OFF (forced)  cursor=%d (expect ARROW=%d)" % [baseline, Control.CURSOR_ARROW])
	if baseline != Control.CURSOR_ARROW:
		_bad("pan mode off left the overlay cursor at %d, not CURSOR_ARROW (%d)"
			% [baseline, Control.CURSOR_ARROW])

	vp.set_pan_mode(true)
	var armed: int = vp.overlay.mouse_default_cursor_shape
	_p("pan ON            cursor=%d (expect DRAG=%d)" % [armed, Control.CURSOR_DRAG])
	if armed != Control.CURSOR_DRAG:
		_bad("pan mode on left the overlay cursor at %d, not CURSOR_DRAG (%d)"
			% [armed, Control.CURSOR_DRAG])
	if armed == baseline:
		_bad("cursor shape did not change between pan off and pan on -- the two "
			+ "states are indistinguishable, which is the defect this closes")

	vp.set_pan_mode(false)
	var disarmed: int = vp.overlay.mouse_default_cursor_shape
	_p("pan OFF again     cursor=%d (expect ARROW=%d)" % [disarmed, Control.CURSOR_ARROW])
	if disarmed != Control.CURSOR_ARROW:
		_bad("turning pan back off did not restore CURSOR_ARROW (got %d)" % disarmed)

	_p("RESULT: %s" % ("PASS" if _fails == 0 else "FAIL -- %d check(s) failed" % _fails))
	get_tree().quit(0 if _fails == 0 else 1)
