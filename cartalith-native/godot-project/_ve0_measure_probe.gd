extends Node
## VE-0 of `VAULT_EDITOR_PROPOSAL.md` s.7: the Godot questions the editor's design (s.5) leaves open,
## answered by measurement on the engine the project ships on.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve0_measure_probe.tscn
##
## **Windowed** (`MISTAKES.md`: input routing and layout are not exercised by the headless dummy
## driver). Takes no flags; any flag aborts. It PRINTS what it measured as `VE0M  <question>  <answer>`
## and asserts only the controls that make an answer trustworthy (a positive control per question),
## so a changed engine changes the printed answer without making the probe lie. Exit 0 = controls held.
##
## Questions:
##  Q1  Which per-span properties does a `SyntaxHighlighter` honour on a `TextEdit` / `CodeEdit`?
##      (Decides whether Live Preview, s.3.1, is possible: it needs size and weight per span.)
##  Q2  Does `RichTextLabel.meta_hover_started` fire for a finger (touch emulation), or only for a mouse?
##      (Decides how the touch peek, s.3.2, is built.)
##  Q3  Android folder access on the 6T (Ruling AV): NOT measurable here -- no device; stated, not guessed.

var _fails: Array = []
const SEL := true
var _hover_started := 0
var _hover_ended := 0
var _clicked: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("VE0M  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE0M  !!  %s   %s" % [label, detail])


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


## A highlighter that asks for `props` on the whole of line 0 and counts how often it was consulted
## (the positive control: an unconsulted highlighter makes every "ignored" answer meaningless).
class Probe extends SyntaxHighlighter:
	var props := {}
	var calls := 0

	func _get_line_syntax_highlighting(_line: int) -> Dictionary:
		calls += 1
		return {0: props}


func _line_metrics(te: TextEdit, props: Dictionary) -> Dictionary:
	var h := Probe.new()
	h.props = props
	te.syntax_highlighter = h
	te.text = "Heading text that should be large"
	te.queue_redraw()
	await _frames(4)
	return {"width": te.get_line_width(0), "height": te.get_line_height(), "calls": h.calls}


func _ready() -> void:
	if not OS.get_cmdline_user_args().is_empty():
		print("VE0M ABORT unknown flag %s" % [OS.get_cmdline_user_args()])
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		print("VE0M REFUSED: run windowed")
		get_tree().quit(2)
		return
	print("VE0M  engine  %s" % Engine.get_version_info().get("string", "?"))

	# == Q1 ===================================================================
	var te := TextEdit.new()
	te.custom_minimum_size = Vector2(500, 120)
	te.size = Vector2(500, 120)
	add_child(te)
	await _frames(3)
	var base := await _line_metrics(te, {})
	_ok("Q1 control: the highlighter is consulted at all", int(base.calls) > 0, str(base))
	var coloured := await _line_metrics(te, {"color": Color.RED})
	var sized := await _line_metrics(te, {"color": Color.RED, "font_size": 48})
	var bold := await _line_metrics(te, {"color": Color.RED, "font": load("res://fonts/FiraSans-Bold.ttf")})
	var size_moves := int(sized.width) != int(coloured.width) or int(sized.height) != int(coloured.height)
	var font_moves := int(bold.width) != int(coloured.width)
	print("VE0M  Q1  baseline width/height            %s / %s" % [base.width, base.height])
	print("VE0M  Q1  with color                       %s / %s" % [coloured.width, coloured.height])
	print("VE0M  Q1  with color + font_size 48        %s / %s  -> size honoured: %s" % [sized.width, sized.height, size_moves])
	print("VE0M  Q1  with color + bold font resource  %s / %s  -> font honoured: %s" % [bold.width, bold.height, font_moves])
	te.queue_free()

	# == Q2 ===================================================================
	var rtl := RichTextLabel.new()
	rtl.bbcode_enabled = true
	rtl.meta_underlined = true
	rtl.selection_enabled = SEL
	rtl.scroll_active = false
	rtl.position = Vector2(40, 200)
	rtl.size = Vector2(400, 60)
	rtl.text = "[url=link:Kelvhold]Kelvhold link text here[/url]"
	rtl.meta_hover_started.connect(func(_m): _hover_started += 1)
	rtl.meta_hover_ended.connect(func(_m): _hover_ended += 1)
	rtl.meta_clicked.connect(func(m): _clicked.append(str(m)))
	## (A) What a finger delivers. Touch emulation belongs to the `Input` singleton and the main
	## window (a SubViewport never sees it: `push_input(ScreenTouch)` into one produced no mouse
	## event), so the finger is sent to a label in the ROOT window and the events it SEES are logged.
	var seen: Array = []
	rtl.gui_input.connect(func(e): seen.append(e.get_class()))
	add_child(rtl)
	await _frames(4)
	var target := rtl.global_position + Vector2(30, 10)
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = true
	t.position = target
	Input.parse_input_event(t)
	await get_tree().create_timer(0.8).timeout
	var still := seen.duplicate()
	var d := InputEventScreenDrag.new()
	d.index = 0
	d.position = target + Vector2(2, 0)
	d.relative = Vector2(2, 0)
	Input.parse_input_event(d)
	await _frames(3)
	var dragged := seen.duplicate()
	t = InputEventScreenTouch.new()
	t.index = 0
	t.pressed = false
	t.position = target + Vector2(2, 0)
	Input.parse_input_event(t)
	await _frames(3)
	var emulate: bool = ProjectSettings.get_setting("input_devices/pointing/emulate_mouse_from_touch", true)
	print("VE0M  Q2  (emulate_mouse_from_touch=%s) events a label saw: finger held still 0.8 s: %s ; then a 2 px drag: %s" % [emulate, str(still), str(dragged)])
	_ok("Q2 control: the emulated finger reached the label at all", still.has("InputEventMouseButton"), str(still))
	var motion_while_still := still.has("InputEventMouseMotion")
	remove_child(rtl)

	## (B) Whether hover starts from the motion that exists, in a SubViewport (where, measured, the
	## pointer must first be declared inside it with NOTIFICATION_VP_MOUSE_ENTER -- without that the
	## motion never reaches the label at all, and every hover probe passes or fails vacuously).
	var vp := SubViewport.new()
	vp.size = Vector2i(800, 400)
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	vp.add_child(rtl)
	await _frames(4)
	vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_hover_started = 0
	_hover_ended = 0
	for off in [Vector2(-6, 0), Vector2.ZERO]:
		var mv := InputEventMouseMotion.new()
		mv.position = target + off
		mv.global_position = mv.position
		mv.relative = Vector2(3, 0)
		vp.push_input(mv)
		await _frames(3)
	var mouse_hover := _hover_started
	_ok("Q2 control: a mouse move over the link starts a hover", mouse_hover > 0, "started=%d" % mouse_hover)
	var away := InputEventMouseMotion.new()
	away.position = Vector2(700, 380)
	away.global_position = away.position
	away.relative = Vector2(5, 5)
	vp.push_input(away)
	await _frames(3)
	_ok("Q2 control: moving off the link ends the hover", _hover_ended > 0, "ended=%d" % _hover_ended)
	## A dragging finger arrives as a motion with the left button held.
	_hover_started = 0
	var held := InputEventMouseMotion.new()
	held.position = target
	held.global_position = target
	held.relative = Vector2(3, 0)
	held.button_mask = MOUSE_BUTTON_MASK_LEFT
	vp.push_input(held)
	await _frames(3)
	var drag_hover := _hover_started
	print("VE0M  Q2  mouse hover starts a peek = %s ; a motion with the left button held (what a dragging finger becomes) starts one = %s ; a still finger sends a motion at all = %s" % [mouse_hover > 0, drag_hover > 0, motion_while_still])
	print("VE0M  Q2  answer: a held finger does NOT start a hover (no motion event is produced) = %s" % (not motion_while_still))
	print("VE0M  Q3  Android folder access on the 6T: not measured (no device in this session)")

	print("VE0M RESULT %s" % ("OK" if _fails.is_empty() else "FAILED " + str(_fails)))
	get_tree().quit(0 if _fails.is_empty() else 1)
