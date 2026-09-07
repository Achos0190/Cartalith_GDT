extends Node
## **The New World card's action row against §6.7's own drawing.**
##
## The canvas (`design/Cartalith-Android-2026-09-07.dc.html`, the `modalOpen`
## block) draws one row spanning the card:
##
##   display:flex; gap:10px; padding-top:16px
##     CANCEL       flex:1    min-height:46px  border-radius:23px  --chip/--sec
##     CREATE WORLD flex:1.4  min-height:46px  border-radius:23px  --acc/--accInk
##
## What shipped until 2026-09-07 was `AcceptDialog`'s own footer:
## `[Create] [Cancel]`, primary first, about a third of the card's width, 44 dp
## high. This probe asserts the row that replaced it -- the ORDER, the widths in
## the 1 : 1.4 the artboard gives them, the 46 dp height, the radius, and that
## the footer is gone.
##
## **What this probe cannot say.** `MISTAKES.md`: synthetic input cannot reach a
## control inside a phone-presented `AcceptDialog` -- `gui_get_hovered_control()`
## stays null at `content_scale_factor` 2.62 -- so the dialog is opened by a
## STAGING CALL and the two buttons are asserted on geometry and wiring, never
## on "a finger can press them". That claim is made on glass with
## `adb shell input tap`.
##
##   godot --path . _nwaction_probe.tscn -- --force-touch --vp 1080x2340 --tag p1080
##   godot --path . _nwaction_probe.tscn -- --force-touch --vp 1440x3168 --tag p1440
##   godot --path . _nwaction_probe.tscn -- --force-touch --vp 720x1600  --tag p720
##
## Flags this probe actually reads, grepped from the body below:
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `nwa`.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`.
## Any other `--flag` aborts rather than being silently ignored.

const CANVAS_H := 46.0
const CANVAS_RADIUS := 23.0
const CANVAS_GAP := 10.0
const CANVAS_TOP := 16.0
const CANVAS_FLEX_RATIO := 1.4   ## CREATE WORLD's `flex` over CANCEL's.

var app: Node
var _vp: SubViewport
var _tag := "nwa"
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

## dp, in the viewport the control is actually in -- `_nwsize_probe.gd`'s own
## expression, and copied rather than re-derived because getting it wrong is how
## the first run of this probe reported a 46 px button as 17.5 dp.
##
## `get_global_rect()` is in the control's viewport CANVAS space. In `_vp` that
## is physical pixels and dividing by `phone_scale()` gives dp. Inside a
## phone-presented `Window` the canvas is already the scaled space, so the final
## transform (`content_scale_factor`) has to be multiplied back on before the
## same division -- the two cancel, and the figure read off `size` is dp
## already. One expression covers both.
func _dp_in(c: Control, px: float) -> float:
	var f: float = c.get_viewport().get_final_transform().x.x
	return px * maxf(0.001, f) / maxf(0.001, float(app.phone_scale()))

func _all(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_all(c, out)

func _buttons_of(root: Node) -> Array:
	var out: Array = []
	var all: Array = []
	_all(root, all)
	for n in all:
		if n is Button and (n as Button).is_visible_in_tree():
			out.append(n)
	return out

func _ready() -> void:
	_tag = _arg("--tag", "nwa")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	_log("viewport %dx%d  phone=%s  scale=%.3f" %
		[_vp.size.x, _vp.size.y, app.is_phone(), app.phone_scale()])
	if not app.is_phone():
		_log("RESULT %s fail=0 (not a phone: §6.7's card is not built here)" % _tag)
		get_tree().quit(0)
		return

	## Staging call, disclosed. See the header.
	app.open_new_world()
	await _frames(20)
	var dlg: Window = app.new_world_dialog
	_check(dlg != null and dlg.visible, "the New World dialog is up")
	if dlg == null or not dlg.visible:
		_log("RESULT %s fail=%d" % [_tag, _fail])
		get_tree().quit(1)
		return

	var cancel: Button = dlg.get("_phone_cancel")
	var create: Button = dlg.get("_phone_create")
	_check(cancel != null and create != null,
		"the card carries its own CANCEL / CREATE WORLD pair")
	if cancel == null or create == null:
		_log("RESULT %s fail=%d" % [_tag, _fail])
		get_tree().quit(1)
		return

	_check(cancel.text == "CANCEL" and create.text == "CREATE WORLD",
		"captions read `%s` and `%s`" % [cancel.text, create.text])

	var cr := cancel.get_global_rect()
	var kr := create.get_global_rect()
	_log("  CANCEL       %.0f,%.0f  %.0f x %.0f px  (%.1f x %.1f dp)"
		% [cr.position.x, cr.position.y, cr.size.x, cr.size.y,
			_dp_in(cancel, cr.size.x), _dp_in(cancel, cr.size.y)])
	_log("  CREATE WORLD %.0f,%.0f  %.0f x %.0f px  (%.1f x %.1f dp)"
		% [kr.position.x, kr.position.y, kr.size.x, kr.size.y,
			_dp_in(create, kr.size.x), _dp_in(create, kr.size.y)])

	## **Order.** The canvas puts the safe action first. This is the assertion
	## that would have failed on what shipped: `AcceptDialog` draws its own OK
	## button before anything `add_cancel_button()` appends.
	_check(cr.position.x < kr.position.x,
		"CANCEL is LEFT of CREATE WORLD (%.0f < %.0f)" % [cr.position.x, kr.position.x])

	## **`flex:1` against `flex:1.4`.** Asserted as a ratio with a tolerance
	## rather than as two widths, because the widths depend on the card, which
	## depends on the screen -- and the artboard's claim is about the SHARE.
	var ratio := kr.size.x / maxf(1.0, cr.size.x)
	_check(absf(ratio - CANVAS_FLEX_RATIO) < 0.12,
		"CREATE WORLD is %.2fx CANCEL's width (canvas: %.2f)" % [ratio, CANVAS_FLEX_RATIO])

	## **46 dp, not 44.** The touch floor is 44 and both would pass it; the
	## artboard asks for 46 and that is the number checked. `phone_fit()` runs
	## over this dialog and floors every `BaseButton` to 44 on both axes, so a
	## height that came back 44 would mean the 46 never took.
	for pair in [[cancel, "CANCEL"], [create, "CREATE WORLD"]]:
		var b := pair[0] as Button
		var h := _dp_in(b, b.get_global_rect().size.y)
		_check(absf(h - CANVAS_H) < 1.5, "%s is %.1f dp high (canvas: %.0f)"
			% [String(pair[1]), h, CANVAS_H])
		var sb := b.get_theme_stylebox("normal") as StyleBoxFlat
		_check(sb != null and absf(float(sb.corner_radius_top_left) - CANVAS_RADIUS) < 0.5,
			"%s radius is %s (canvas: %.0f)"
				% [String(pair[1]), str(sb.corner_radius_top_left) if sb != null else "-",
					CANVAS_RADIUS])
		_check(sb != null and sb.border_width_left == 0,
			"%s is a wash with no border, as §6.7 draws it" % String(pair[1]))

	## **`gap:10px`.** Measured between the two rects rather than read off the
	## container constant, so a stray margin would show.
	var gap := _dp_in(create, kr.position.x - (cr.position.x + cr.size.x))
	_check(absf(gap - CANVAS_GAP) < 1.5, "the gap between them is %.1f dp (canvas: %.0f)"
		% [gap, CANVAS_GAP])

	## **`padding-top:16px`.** `CANVAS_TOP` was declared in this file and never
	## used in an assertion -- mutating `PHONE_ACTION_TOP` to 0 left this probe
	## at fail=0, so a dead constant was reading as coverage. Measured the same
	## way the gap is: the distance from the bottom of whatever sits above the
	## row to the top of the row, off the rects, not off the container's own
	## override -- reading the override back would assert the constant against
	## itself.
	var above := cancel.get_parent().get_parent()
	var prev: Control = null
	if above != null:
		var idx := above.get_index()
		var host := above.get_parent()
		if host != null and idx > 0:
			var sib := host.get_child(idx - 1)
			if sib is Control and (sib as Control).visible:
				prev = sib as Control
	if prev != null:
		var pr := prev.get_global_rect()
		var top := _dp_in(cancel, cr.position.y - (pr.position.y + pr.size.y))
		_check(absf(top - CANVAS_TOP) < 2.0,
			"the row sits %.1f dp below the block above it (canvas: %.0f)"
			% [top, CANVAS_TOP])
	else:
		## Not silently skipped: a missing neighbour means the row is the card's
		## first child, which the canvas does not draw, and that is a failure
		## rather than an untaken branch.
		_check(false, "the action row has a visible sibling above it to measure against")

	## **The row spans the card.** `flex:1 + flex:1.4 + gap` fills the width, so
	## the pair should reach both inner edges of the card's own padding.
	var card: Control = dlg.get("_card")
	if card != null:
		var card_r := card.get_global_rect()
		var span := (kr.position.x + kr.size.x) - cr.position.x
		_log("  the pair spans %.1f dp of a %.1f dp card (16 dp padding each side)"
			% [_dp_in(create, span), _dp_in(card, card_r.size.x)])
		_check(_dp_in(create, span) > _dp_in(card, card_r.size.x) - 40.0,
			"the row spans the card rather than sitting in a corner")

	## **Does the card still FIT?** `MISTAKES.md`: content added below the fold
	## evicts content that was above it, and the row is 46 + 16 dp of new
	## content at the very bottom of a card §6.4 measured at 688 dp. Reported as
	## numbers rather than as a pass, because the answer is the whole question.
	if card != null:
		var card_r := card.get_global_rect()
		var win_h := float(dlg.size.y) / maxf(0.001, dlg.content_scale_factor)
		_log("  card %.1f x %.1f dp   window %.1f dp tall   row bottom at %.1f dp"
			% [_dp_in(card, card_r.size.x), _dp_in(card, card_r.size.y), win_h,
				kr.position.y + kr.size.y])
		var sc: ScrollContainer = null
		var stack: Array = []
		_all(dlg, stack)
		for n in stack:
			if n is ScrollContainer:
				sc = n as ScrollContainer
				break
		if sc != null:
			var bar := sc.get_v_scroll_bar()
			_log("  the form's ScrollContainer is %.0f dp tall over %.0f dp of content"
				% [sc.size.y, bar.max_value])
			_check(bar.max_value <= sc.size.y + 1.0 or sc.vertical_scroll_mode
					!= ScrollContainer.SCROLL_MODE_DISABLED,
				"the form either fits or can be scrolled (content %.0f, viewport %.0f, mode %d)"
					% [bar.max_value, sc.size.y, sc.vertical_scroll_mode])
		_check(kr.position.y + kr.size.y <= win_h,
			"CREATE WORLD's bottom edge (%.1f dp) is inside the window (%.1f dp)"
				% [kr.position.y + kr.size.y, win_h])

	## **Who eats the scroll?** The card overflows its scroller, so the row is
	## below the fold and the only way to it is a finger drag -- and PH-05's
	## rule is that a `MOUSE_FILTER_STOP` anywhere on the path ends the event
	## walk before the `ScrollContainer` sees it. Printed as the whole chain
	## from the deepest advisory `Label` up, because "it should scroll" is a
	## claim about every node between, not about the scroller.
	if card != null:
		var deepest: Control = null
		var stack2: Array = []
		_all(card, stack2)
		for n in stack2:
			if n is Label and (n as Label).is_visible_in_tree():
				deepest = n as Control
		if deepest != null:
			var chain: Array = []
			var q: Node = deepest
			while q != null and not (q is Window):
				if q is Control:
					chain.append("%s=%d" % [q.get_class(), (q as Control).mouse_filter])
				q = q.get_parent()
			_log("  filter chain from the last advisory label up: %s" % " -> ".join(chain))
			## Every node between the content and the scroller must be PASS (1)
			## or IGNORE (2). A single STOP (0) is the whole defect.
			var stops: Array = []
			for entry in chain:
				if String(entry).ends_with("=0"):
					stops.append(entry)
			_check(stops.is_empty(),
				"nothing on the path to the scroller eats the drag; STOP at %s" % str(stops))

	## **And `AcceptDialog`'s own footer is gone.** The assertion that the
	## replacement replaced something: a visible `Create` or `Cancel` in the
	## dialog's button row would mean two action rows, not one.
	var ok_btn := (dlg as AcceptDialog).get_ok_button()
	_check(ok_btn != null and not ok_btn.is_visible_in_tree(),
		"`AcceptDialog`'s own OK button is not on screen")
	var stray: Array = []
	for b in _buttons_of(dlg):
		if b == cancel or b == create:
			continue
		if String(b.text) in ["Create", "Cancel", "Close"]:
			stray.append(String(b.text))
	_check(stray.is_empty(), "no leftover footer button is visible; found %s" % str(stray))

	## **Wiring**, by signal connection rather than by pressing: a press through
	## the embedded window does not arrive (see the header), so asserting on a
	## press would be asserting on the probe's own limitation.
	_check(cancel.pressed.get_connections().size() > 0,
		"CANCEL is connected to something")
	_check(create.pressed.get_connections().size() > 0,
		"CREATE WORLD is connected to something")

	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
