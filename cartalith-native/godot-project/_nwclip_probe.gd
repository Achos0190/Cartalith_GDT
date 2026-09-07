extends Node
## **Is the New World card's action row DRAWN inside the viewport it lives in?**
##
## `_nwaction_probe.gd` already asks a question that sounds like this one and is
## not it. Its check is
##
##   `CREATE WORLD's bottom edge (835.0 dp) is inside the window (872.8 dp)`
##
## and that passed -- `fail=0`, every leg green -- on the exact build a cold
## launch found the button unreachable on. The window is 872.8 dp; the button is
## not in the window, it is in a `ScrollContainer` **702 dp** tall holding
## **752 dp** of card, and `Control.get_global_rect()` reports a child's
## unclipped rect whatever the scroller does with it. So the row measured at
## 789..835 while the band that draws it ended fifty pixels earlier.
##
## This probe measures the rect that is actually painted: the control's own
## rect intersected with every **clipping** ancestor between it and the window.
## That is the number a finger can land on, and it is the only one that could
## have caught this.
##
## **It does not press anything.** Synthetic input cannot reach a control inside
## a phone-presented `AcceptDialog` (`_nwaction_probe.gd`'s header carries the
## measurement), and pressing a button by name is how the defect stayed hidden
## in the first place -- a staged press lands on the node whether or not a pixel
## of it is on screen. The dialog is opened by a staging call, disclosed here,
## and everything after that is geometry.
##
##   godot --headless --path . _nwclip_probe.tscn -- --force-touch --vp 1080x2340 --tag p1080
##   godot --headless --path . _nwclip_probe.tscn -- --force-touch --vp 1440x3168 --tag p1440
##   godot --headless --path . _nwclip_probe.tscn -- --force-touch --vp 720x1600  --tag p720
##
## Flags this probe actually reads, grepped from the body below:
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `nwclip`.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`.
## Any other `--flag` aborts rather than being silently ignored.

var app: Node
var _vp: SubViewport
var _tag := "nwclip"
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

func _all(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_all(c, out)

## The painted rect: the control's own rect narrowed by every ancestor that
## clips. `ScrollContainer` sets `clip_contents`, and so does anything else that
## means it, so the flag is the test rather than the class -- a `Panel` with
## `clip_contents` on cuts a child exactly as hard.
##
## Returns a zero-size `Rect2` when nothing survives, which is the case the
## unclipped measurement reads as a healthy 46 dp button.
func _painted_rect(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var q: Node = c.get_parent()
	while q != null and not (q is Window):
		if q is Control and (q as Control).clip_contents:
			r = r.intersection((q as Control).get_global_rect())
			if r.size.x <= 0.0 or r.size.y <= 0.0:
				return Rect2(r.position, Vector2.ZERO)
		q = q.get_parent()
	return r

## The chain of clipping ancestors, for the log -- so a failure names the node
## that ate the control rather than only the fact that something did.
func _clippers(c: Control) -> String:
	var out: Array = []
	var q: Node = c.get_parent()
	while q != null and not (q is Window):
		if q is Control and (q as Control).clip_contents:
			var g := (q as Control).get_global_rect()
			out.append("%s[y %.0f..%.0f]" % [q.get_class(), g.position.y, g.end.y])
		q = q.get_parent()
	return " <- ".join(out) if not out.is_empty() else "(none)"

func _ready() -> void:
	_tag = _arg("--tag", "nwclip")
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

	## Staging call, disclosed. See the header for why there is no press here.
	app.open_new_world()
	await _frames(20)
	var dlg: Window = app.new_world_dialog
	_check(dlg != null and dlg.visible, "the New World dialog is up")
	if dlg == null or not dlg.visible:
		_log("RESULT %s fail=%d" % [_tag, _fail])
		get_tree().quit(1)
		return

	var create: Button = dlg.get("_phone_create")
	var cancel: Button = dlg.get("_phone_cancel")
	_check(create != null and cancel != null, "the card carries its own action pair")
	if create == null or cancel == null:
		_log("RESULT %s fail=%d" % [_tag, _fail])
		get_tree().quit(1)
		return

	var card: Control = dlg.get("_card")
	var outer: ScrollContainer = dlg.get("_outer_scroll")

	## **Minimums, because pinning a row is a minimum-size change.**
	## `MISTAKES.md`: a minimum escaping a `SCROLL_MODE_DISABLED` container has
	## widened a dock twice here. The outer scroller disables its HORIZONTAL
	## axis, so anything the card demands sideways reaches the window; its
	## vertical axis is AUTO and absorbs height. Printed as a chain so the
	## reader can see which link grew rather than only the total.
	_log("  minimums (x, y):")
	_log("    window contents  %s" % str((dlg as AcceptDialog).get_contents_minimum_size()))
	for pair in [[dlg.get_child(0) as Control, "col"], [outer, "outer scroll"],
			[card, "card"], [dlg.get("_card_scroll"), "card scroll"],
			[dlg.get("_card_form"), "card form"]]:
		var n := pair[0] as Control
		if n != null:
			_log("    %-16s %s" % [String(pair[1]), str(n.get_combined_minimum_size())])
	if card != null:
		var cr := card.get_global_rect()
		_log("  card      y %.0f..%.0f  (%.0f x %.0f)"
			% [cr.position.y, cr.end.y, cr.size.x, cr.size.y])
	if outer != null:
		var bar := outer.get_v_scroll_bar()
		_log("  outer scroll y %.0f..%.0f  viewport %.0f over content %.0f"
			% [outer.get_global_rect().position.y, outer.get_global_rect().end.y,
				outer.size.y, bar.max_value])
		## A `ScrollContainer` lays its child inside `size` LESS its own panel
		## stylebox, so `size.y` overstates the room by that frame. Printed
		## because the first cut of `_fit_phone_card_height()` used `size.y` and
		## left the card's last 6 dp under the scroller's bottom edge.
		var sb := outer.get_theme_stylebox("panel")
		if sb != null:
			_log("  outer scroll panel frame: top %.0f bottom %.0f (min %.0f)"
				% [sb.get_margin(SIDE_TOP), sb.get_margin(SIDE_BOTTOM),
					sb.get_minimum_size().y])
		if card != null:
			var cr2 := card.get_global_rect()
			var over := cr2.end.y - outer.get_global_rect().end.y
			_check(over <= 0.5,
				"the card's own bottom edge is inside the scroller (%.0f dp over)" % over)

	## **The assertion.** Not "the rect is inside the window" -- the rect that is
	## PAINTED, against the rect the node claims. A row that has scrolled out of
	## its band loses area here and loses none there.
	for pair in [[cancel, "CANCEL"], [create, "CREATE WORLD"]]:
		var b := pair[0] as Button
		var name := String(pair[1])
		var full := b.get_global_rect()
		var seen := _painted_rect(b)
		_log("  %-12s rect y %.0f..%.0f  painted y %.0f..%.0f  (%.0f%% of its area)"
			% [name, full.position.y, full.end.y, seen.position.y, seen.end.y,
				100.0 * (seen.size.x * seen.size.y)
					/ maxf(1.0, full.size.x * full.size.y)])
		_log("    clipped by: %s" % _clippers(b))
		_check(absf(seen.size.y - full.size.y) < 1.0 and absf(seen.size.x - full.size.x) < 1.0,
			"%s is drawn WHOLE without scrolling anything (%.0f x %.0f of %.0f x %.0f)"
				% [name, seen.size.x, seen.size.y, full.size.x, full.size.y])

	## **Positive control, and it is the whole reason to trust the leg above.**
	## The form inside the card overflows on every handset this runs on, so its
	## last child -- with the card's own scroller at the top, which is where it
	## opens -- MUST come back clipped. If `_painted_rect()` reports it whole,
	## the measurement cannot see clipping at all and the green above is vacuous.
	var form: Control = dlg.get("_card_form")
	var inner: ScrollContainer = dlg.get("_card_scroll")
	if form != null and inner != null:
		var bar := inner.get_v_scroll_bar()
		_log("  inner scroll viewport %.0f over content %.0f  (offset %.0f)"
			% [inner.size.y, bar.max_value, inner.scroll_vertical])
		if bar.max_value > inner.size.y + 1.0:
			var last: Control = null
			for i in range(form.get_child_count() - 1, -1, -1):
				var ch := form.get_child(i)
				if ch is Control and (ch as Control).is_visible_in_tree():
					last = ch as Control
					break
			if last != null:
				var lf := last.get_global_rect()
				var lp := _painted_rect(last)
				_log("  [control] form's last visible child %s rect y %.0f..%.0f painted y %.0f..%.0f"
					% [last.get_class(), lf.position.y, lf.end.y, lp.position.y, lp.end.y])
				_check(lp.size.y < lf.size.y - 1.0,
					"the measurement CAN see a clip: the form's own tail is cut (%.0f of %.0f)"
						% [lp.size.y, lf.size.y])
		else:
			_check(false,
				"positive control unavailable: the form does not overflow (content %.0f, viewport %.0f)"
					% [bar.max_value, inner.size.y])
	else:
		_check(false, "the card exposes `_card_form` and `_card_scroll` to measure against")

	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
