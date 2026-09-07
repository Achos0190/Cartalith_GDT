extends Node
## Lane PANE-MIN, 2026-09-07 -- **what is the Data manager's route pane's own
## minimum width, per route, and which node demands it?**
##
## `_pcbrowse_probe.gd` established the symptom: `export_world`'s `Browse…` is
## `shown=0.000` at 1152x648, and `export_maps`' `Choose…` fails the same way.
## It could not say *why*, because a window-level `get_contents_minimum_size()`
## is one number and the pane is forty nodes.
##
## This probe walks DOWN. For every route it prints the chain of nodes whose
## `get_combined_minimum_size().x` exceeds the room the pane actually has, so a
## fix has a leaf to aim at rather than a total to argue with. It then measures
## every picker's DRAWN rect against its container's VISIBLE rect and against
## the owning `Window`'s client rect -- `get_global_rect()` is unclipped and
## reports a button at x=1291 inside a 1152 px window as perfectly healthy.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _panemin_probe.tscn -- --vp 1152x648
##   Godot_v4.7.1-stable_win64_console.exe --path . _panemin_probe.tscn -- --vp 1680x1010
##   Godot_v4.7.1-stable_win64_console.exe --path . _panemin_probe.tscn -- --vp 1024x640
##
## Flags this probe actually reads, grepped from the body below:
##   `--vp WxH`     OS window size in px. Default `1152x648`, the project's own.
##   `--tag NAME`   prefix on every line. Default `panemin`.
##   `--route ID`   walk only this route id. Default: every entry in `ROUTES`.
##   `--verbose`    print the full over-width chain per route, not just the leaf.
## Any other `--flag` aborts rather than being silently ignored -- the header is
## a claim about this file's own code and is checked by `_reject_unknown_args`.
##
## **Windowed on purpose.** No pixel is read here, so `--headless` would not be
## vacuous the way it is for a texture probe -- but an embedded `Window` clamps
## against the real screen, and the dummy driver's screen is not this machine's.

var app: Node
var _tag := "panemin"
var _fail := 0
var _verbose := false
var _only := ""
var _pre_worst := 0.0
var _pre_broke := 0
var _worst_body := 0.0
var _worst_body_route := ""
var _worst_foot := 0.0
var _worst_foot_route := ""
## Legs that could not be performed, as distinct from legs that failed. A probe
## that cannot answer must not report a pass, and it must not report a failure
## of the code either -- `MISTAKES.md`: "check every verification item's premise
## holds before you write the item". Three of this probe's premises do not hold
## on a phone, where `DccWidgets.phone_window()` clears `min_size` to (0, 0) and
## `_popup_full()` returns before it sizes anything.
var _skipped := 0

func _skip(why: String) -> void:
	_skipped += 1
	_log("  SKIP %s" % why)

## The width this window is actually promising to work at. `min_size.x` on a
## pointer or tablet build; on a phone that is 0 -- the phone window is the
## screen -- so the screen is the promise there, and the caller is told which it
## got rather than silently comparing against -288.
func _promised_w(win: Window) -> float:
	return float(win.min_size.x) if win.min_size.x > 0 else float(win.size.x)

func _promise_name(win: Window) -> String:
	return "declared min_size.x" if win.min_size.x > 0 else "the phone window's own width"

## Everything the route pane does NOT get: the routes rail plus the pane's own
## horizontal padding. **The rail is only a term on a pointer or tablet.** §13
## stacks it: `_build()` builds `main` as a `VBoxContainer` when `_phone` and an
## `HBoxContainer` otherwise, and `_build_rail()`'s `_phone` branch sets
## `size_flags_vertical = SIZE_EXPAND_FILL` instead of the
## `custom_minimum_size.x = W_RAIL` the pointer branch sets -- so on a handset
## the rail sits ABOVE the pane and takes none of its width. Subtracting 252 px
## there charged the pane for a column it does not sit beside: it made a 500 dp
## handset look like it had 212 px of footer room when it has 464, and turned
## five routes red for 12 px that do not exist.
func _chrome_w(w: Node) -> float:
	var pad := float(w.PANE_PAD_X) * 2.0
	return pad if DccTheme.is_phone() else float(w.W_RAIL) + pad

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
	## `--force-touch` is read by `dcc_shell.gd` out of `OS.get_cmdline_user_args()`,
	## not here -- but it has to be in this list or the run aborts on it, and a
	## tablet/phone pass over this window is exactly what a width fix needs.
	var known := ["--vp", "--tag", "--route", "--verbose", "--force-touch"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

## The rect a viewer can actually see: the control's own rect intersected with
## every ancestor `Control`'s rect and then with the owning `Window`'s client
## rect. The last term is the one `get_global_rect()` and a naive parent walk
## both miss -- when every container above the button is equally oversized, an
## intersection that stops at the last `Control` reports `shown=1.000` for a
## button 139 px past the window's right edge.
func _visible_rect(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var n: Node = c.get_parent()
	while n != null:
		if n is Control:
			r = r.intersection((n as Control).get_global_rect())
		elif n is Window:
			r = r.intersection(Rect2(Vector2.ZERO, Vector2((n as Window).size)))
			break
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			return Rect2(r.position, Vector2.ZERO)
		n = n.get_parent()
	if r.size.x <= 0.0 or r.size.y <= 0.0:
		return Rect2(r.position, Vector2.ZERO)
	return r

func _shown(c: Control) -> float:
	var own := c.get_global_rect()
	if own.size.x <= 0.0 or own.size.y <= 0.0:
		return 0.0
	var v := _visible_rect(c)
	return (v.size.x * v.size.y) / (own.size.x * own.size.y)

## Every `Button` under `root` that is a picker by its own drawn text, not by a
## name this probe guessed. `export_maps` spells its picker `Choose…` and a
## hunt for "Browse" reported that route as having none.
func _pickers(root: Node, out: Array) -> Array:
	if root is Button:
		var t := String((root as Button).text)
		if t.begins_with("Browse") or t.begins_with("Choose"):
			out.append(root)
	for ch in root.get_children():
		_pickers(ch, out)
	return out

## Walk DOWN from `n`, following at each level the child with the largest
## `get_combined_minimum_size().x`, for as long as that child's minimum is at
## least `floor`. The result names the leaf that demands the width, which is
## what a fix needs -- a total tells you only that something does.
func _widest_chain(n: Control, floor_px: float, depth: int = 0) -> Array:
	var out: Array = []
	var cur := n
	var d := depth
	while cur != null and d < 40:
		var m := cur.get_combined_minimum_size()
		out.append("%s(%s)=%.0f" % [cur.name, cur.get_class(), m.x])
		var best: Control = null
		var best_x := -1.0
		for ch in cur.get_children():
			if ch is Control and (ch as Control).is_visible_in_tree():
				var cm := (ch as Control).get_combined_minimum_size().x
				if cm > best_x:
					best_x = cm
					best = ch as Control
		if best == null or best_x < floor_px:
			break
		cur = best
		d += 1
	return out

## Sum of a horizontal box's children's minimum widths plus separations -- the
## same arithmetic the container itself does, printed so a row's total can be
## attributed to a term rather than to the row.
func _hbox_terms(h: BoxContainer) -> String:
	var sep := float(h.get_theme_constant("separation"))
	var parts: Array = []
	var total := 0.0
	var n := 0
	for ch in h.get_children():
		if ch is Control and (ch as Control).is_visible_in_tree():
			var m := (ch as Control).get_combined_minimum_size().x
			parts.append("%s:%.0f" % [(ch as Control).get_class(), m])
			total += m
			n += 1
	if n > 1:
		total += sep * float(n - 1)
	return "%s = %.0f (sep %.0f x %d)" % [" + ".join(parts), total, sep, maxi(0, n - 1)]

## The footer note's floor, checked from **both** sides and against numbers this
## probe derives rather than against `FOOT_NOTE_MIN_W` itself -- an assertion
## made against the constant holds for every value of it.
##
##   ceiling  the note may not ask for more than the row has left once the
##            chips are paid for **at the window's own declared minimum
##            width**. Raise the constant past that and the footer is once
##            again what decides how wide this window has to be.
##   floor    a literal 120 px, which is "writes into C:/Users/Vi…" and enough
##            to tell one note from another. Drop the constant to the 1 px a
##            trimmed `Label` reports and this goes red.
func _footer_note_legs(w: Node, win: Window, route: String, foot: BoxContainer) -> void:
	var note: Label = null
	var chips := 0.0
	var n := 0
	for ch in foot.get_children():
		if ch is Control and (ch as Control).is_visible_in_tree():
			n += 1
			if note == null and ch is Label:
				note = ch as Label
			else:
				chips += (ch as Control).get_combined_minimum_size().x
	if note == null:
		return
	var sep := float(foot.get_theme_constant("separation"))
	chips += sep * float(maxi(0, n - 1))
	var promise := _promised_w(win)
	var room := promise - _chrome_w(w)
	var note_min := note.get_combined_minimum_size().x
	var drawn := note.get_global_rect().size.x
	_log("    note min=%.0f drawn=%.0f  chips+sep=%.0f  room at %s (%.0f - %.0f chrome)=%.0f"
		% [note_min, drawn, chips, _promise_name(win), promise, _chrome_w(w), room])
	## Premise: this is a claim about the NOTE's floor, so it is only meaningful
	## where a zero-width note would have fitted. On a 500 dp handset
	## `export_world`'s three chips alone are 477 px against 212 px of room --
	## the row cannot fit whatever the note does, and failing it here would file
	## a chip-width defect under a footer-note fix.
	if chips > room:
		_skip("%s: note-floor leg unsatisfiable -- the chips alone are %.0f px against %.0f px of room, so no note width fits"
			% [route, chips, room])
	else:
		_check(note_min + chips <= room + 0.5,
			"%s: the footer note's floor still leaves the chips their width at %s (%.0f + %.0f <= %.0f)"
				% [route, _promise_name(win), note_min, chips, room])
	## Both floor legs are claims about `FOOT_NOTE_MIN_W`, which `_footer_note()`
	## deliberately does not apply on a handset -- 160 px is 39 % of a 393 dp
	## pane there, and applying it puts the route's own picker off the screen.
	## Asserting a floor that the code intentionally withholds is the same
	## unsatisfiable-premise error as the two already corrected above.
	if DccTheme.is_phone():
		_skip("%s: floor legs -- `FOOT_NOTE_MIN_W` is pointer/tablet only, and the note reads min=%.0f drawn=%.0f here"
			% [route, note_min, drawn])
	else:
		_check(note_min >= 120.0,
			"%s: the footer note keeps at least 120 px of itself (min=%.0f)" % [route, note_min])
		_check(drawn >= 120.0 and _shown(note) > 0.999,
			"%s: the note is DRAWN at %.0f px and fully inside the window (shown=%.3f)"
				% [route, drawn, _shown(note)])

	## **The property the fix is actually about, tested directly.**
	##
	## The defect was not "the window is 1372 px". It was that the window's
	## required width was a function of **how long the user's exports path
	## happens to be** -- three of `_footer_note()`'s eleven call sites
	## interpolate an absolute path into an unclipped `Label`, so a person with
	## a deeply nested Documents folder got a wider window than a person
	## without one. That is machine-dependent layout, and no fixed number
	## anywhere can assert it away.
	##
	## So: lengthen the note's own text by 120 characters and require the
	## footer's minimum not to move. It is density-agnostic, it needs no
	## constant, and it is the one assertion here that would have failed for
	## every route before the change rather than only the three that overflowed.
	## **Did removing `spacer()` move the chips?** The note now fills the row, so
	## a left-aligned `Label` is doing the spacer's old job -- but that is a
	## claim about layout, and layout claims get measured. The last chip's right
	## edge must still land on the pane's right padding.
	var edges: Array = []
	var last_right := 0.0
	for ch in foot.get_children():
		if ch is Control and (ch as Control).is_visible_in_tree():
			var g := (ch as Control).get_global_rect()
			edges.append("%s[%.0f..%.0f]" % [(ch as Control).get_class(), g.position.x, g.end.x])
			last_right = maxf(last_right, g.end.x)
	_log("    footer laid: %s | row right edge %.0f, pane right edge %.0f (pad %d)"
		% [" ".join(edges), last_right,
			foot.get_global_rect().end.x, w.PANE_PAD_X])
	_check(absf(last_right - foot.get_global_rect().end.x) < 1.5,
		"%s: the chips still sit on the row's right edge without the spacer (%.0f vs %.0f)"
			% [route, last_right, foot.get_global_rect().end.x])

	var foot_before := foot.get_combined_minimum_size().x
	var kept_text := note.text
	note.text = kept_text + "/deeply/nested/somewhere/else/entirely/that/a/user/might/plausibly/keep/their/exports/in/2026/september"
	await _frames(4)
	var foot_after := foot.get_combined_minimum_size().x
	_log("      note diag: custom=%.0f min=%.0f combined=%.0f overrun=%d clip=%s chars=%d"
		% [note.custom_minimum_size.x, note.get_minimum_size().x,
			note.get_combined_minimum_size().x, note.text_overrun_behavior,
			note.clip_text, note.text.length()])
	note.text = kept_text
	await _frames(4)
	_log("      note diag restored: custom=%.0f min=%.0f combined=%.0f chars=%d"
		% [note.custom_minimum_size.x, note.get_minimum_size().x,
			note.get_combined_minimum_size().x, note.text.length()])
	_check(absf(foot_after - foot_before) < 0.5,
		"%s: the footer's width does not depend on how long the path is (+104 chars moved it %.0f -> %.0f)"
			% [route, foot_before, foot_after])

## **Does the trimmed note still put ink on the screen?**
##
## Geometry says the `Label` has a box; it cannot say the box has letters in it,
## and `MISTAKES.md` records `clip_text` collapsing a label to nothing while
## every size assertion around it stayed green. So: read the framebuffer inside
## the note's own visible rect, blank the note's text, read the same rect again,
## and require the ink to disappear. That second reading is the control -- it is
## the only thing that proves the first one was measuring text rather than
## measuring a background.
##
## Palette-agnostic on purpose (this machine boots `light`): the measure is
## "pixels unlike the rect's own most common colour", not a threshold against a
## named ink.
func _ink_in(img: Image, r: Rect2) -> int:
	var x0 := maxi(0, int(r.position.x))
	var y0 := maxi(0, int(r.position.y))
	var x1 := mini(img.get_width(), int(r.end.x))
	var y1 := mini(img.get_height(), int(r.end.y))
	if x1 <= x0 or y1 <= y0:
		return -1
	var counts: Dictionary = {}
	for y in range(y0, y1):
		for x in range(x0, x1):
			var k := img.get_pixel(x, y).to_rgba32()
			counts[k] = int(counts.get(k, 0)) + 1
	var modal := 0
	for k in counts:
		modal = maxi(modal, int(counts[k]))
	return (x1 - x0) * (y1 - y0) - modal

func _grab() -> Image:
	await RenderingServer.frame_post_draw
	var t := get_viewport().get_texture()
	if t == null:
		return null
	return t.get_image()

func _ink_control(win: Window, route: String, foot: BoxContainer) -> void:
	var note: Label = null
	for ch in foot.get_children():
		if ch is Label:
			note = ch as Label
			break
	if note == null:
		_check(false, "%s: the footer carries a note to read pixels from" % route)
		return
	## Window-local rect -> root-viewport rect. An embedded sub-window is drawn
	## into the parent viewport at `win.position`, so the framebuffer offset is
	## that and not the window origin the geometry above works in.
	var vis := _visible_rect(note)
	## Premise: there has to be a drawn rect to read pixels out of. On a 500 dp
	## handset the whole footer is off the window and the rect is degenerate --
	## reading 0 ink there says nothing about the trim, only that nothing is
	## on screen, and reporting it as "the note does not render" would blame the
	## trim for the handset's own overflow.
	if vis.size.x < 8.0 or vis.size.y < 4.0:
		_skip("%s: no ink reading -- the note's drawn rect is %.0fx%.0f, so there is nothing on screen to measure"
			% [route, vis.size.x, vis.size.y])
		return
	var r := Rect2(vis.position + Vector2(win.position), vis.size)
	await _frames(3)
	var before := await _grab()
	if before == null:
		_check(false, "%s: the framebuffer is readable (windowed, not headless)" % route)
		return
	var ink_before := _ink_in(before, r)
	var kept := note.text
	note.text = ""
	await _frames(4)
	var after := await _grab()
	var ink_after := _ink_in(after, r) if after != null else -1
	note.text = kept
	await _frames(2)
	_log("  [control] note rect %s  ink with text=%d  ink blanked=%d"
		% [str(r), ink_before, ink_after])
	_check(ink_before > 40,
		"%s: the trimmed note actually RENDERS (%d non-background px in its drawn rect)"
			% [route, ink_before])
	_check(ink_after >= 0 and ink_after * 4 < ink_before,
		"the pixel measure CAN see the text go: blanking it drops %d -> %d"
			% [ink_before, ink_after])

## **The before state, reconstructed in the same process rather than recalled.**
##
## A fix that is only compared against a number written down earlier is a fix
## compared against a memory. This puts the footer note back to exactly what
## `_footer_note()` built before 2026-09-07 -- no expand flag, no trim, no
## floor, and the `spacer()` that used to follow it -- re-reads the window's
## contents minimum and the picker's drawn rect, and then puts it back.
##
## It is also the guard's own negative control: if the route passes with the
## note reverted, the assertions above are not testing anything.
##
## **The picker leg is premise-guarded, and the first cut of it was not.** It
## asserted "the pre-fix footer still breaks the picker" for every route with
## one, and `export_gis` failed it honestly: that route's pre-fix minimum was
## 1149 against a 1152 px window, three pixels of slack, so its `Browse…` drew
## at 821..891 and was never off the edge at this width. An item whose premise
## does not hold is not a finding, it is an unsatisfiable check -- so the break
## is asserted only where the pre-fix minimum actually exceeded the window, and
## the run as a whole asserts that at least one route did.
func _ab(w: Node, win: Window, route: String, foot: BoxContainer, post_min: float) -> void:
	var note: Label = null
	for ch in foot.get_children():
		if ch is Label:
			note = ch as Label
			break
	if note == null:
		return
	var pickers := _pickers(w, [])
	if pickers.is_empty():
		return
	## **`clip_text` is the fourth property, and leaving it out made the first
	## phone reading a lie.** `DccShell.phone_fit()` sets `clip_text` on any
	## `Label` carrying `SIZE_EXPAND` -- which the fixed note now does and the
	## pre-fix note did not -- so a reconstruction that restores three
	## properties and not the fourth measures a label the fix caused to be
	## clipped and calls the reading "pre-fix". It reported `note_min=1` and a
	## 430 px baseline at 500x1080, making the fix look like a +129 px
	## regression on the handset. With `clip_text` restored the same route reads
	## its real pre-fix minimum.
	var was_shown := _shown(pickers[0] as Control)
	var keep_flags := note.size_flags_horizontal
	var keep_overrun := note.text_overrun_behavior
	var keep_min := note.custom_minimum_size.x
	var keep_clip := note.clip_text
	note.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	note.text_overrun_behavior = TextServer.OVERRUN_NO_TRIMMING
	note.custom_minimum_size.x = 0.0
	note.clip_text = false
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	spacer.size_flags_vertical = Control.SIZE_EXPAND_FILL
	foot.add_child(spacer)
	foot.move_child(spacer, 1)
	await _frames(6)
	var pre_min: Vector2 = w.get_contents_minimum_size()
	var pre_shown := _shown(pickers[0] as Control)
	var pre_edge := (pickers[0] as Control).get_global_rect().end.x
	_log("    PRE-FIX %s: contents_min.x=%.0f  note_min=%.0f  %s right edge %.0f  shown=%.3f"
		% [route, pre_min.x, note.get_combined_minimum_size().x,
			(pickers[0] as Button).text, pre_edge, pre_shown])
	_pre_worst = maxf(_pre_worst, pre_min.x)
	_check(pre_min.x >= post_min - 0.5,
		"%s: the fix never made a route WIDER (pre %.0f, now %.0f)"
			% [route, pre_min.x, post_min])
	## The premise is about **this picker's own right edge**, not about the
	## route's total. `export_gis` overflows by 125 px at a 1024 px window and
	## its `Browse…` still draws at 821..891, because the overflow lands to the
	## right of it -- guarding on the total put a second unsatisfiable check
	## here after the first one was corrected.
	if pre_edge > float(win.size.x) + 0.5:
		_pre_broke += 1
		_check(pre_shown < 0.999,
			"%s: the PRE-FIX footer still breaks the picker, so the check above can fail (right edge %.0f > window %d, shown=%.3f)"
				% [route, pre_edge, win.size.x, pre_shown])
	else:
		_log("      (no picker leg: pre-fix right edge %.0f is inside this %d px window; the route's %.0f px of overflow lands elsewhere in the row)"
			% [pre_edge, win.size.x, maxf(0.0, pre_min.x - float(win.size.x))])
	foot.remove_child(spacer)
	spacer.queue_free()
	note.size_flags_horizontal = keep_flags
	note.text_overrun_behavior = keep_overrun
	note.custom_minimum_size.x = keep_min
	note.clip_text = keep_clip
	await _frames(6)
	## **Restoration is asserted against what was measured, not against 1.0.**
	## "Whole again" is the wrong claim on a composition where the fixed route
	## is not whole to begin with: at 500x1080 `export_maps` is still cut with
	## the fix in, because a 500 dp handset cannot hold this window's footer at
	## all. Asserting 1.0 there would report a restore failure for a defect
	## that has nothing to do with the restore.
	var restored := _shown(pickers[0] as Control)
	_check(absf(restored - was_shown) < 0.002,
		"%s: the A/B put the route back exactly as it found it (shown %.3f -> %.3f)"
			% [route, was_shown, restored])

## **The other fix, measured instead of dismissed.**
##
## The defect could be read as "the declared minimum is a lie" rather than "the
## pane is too wide", and widening a declared minimum is sometimes the right
## answer. This runs that alternative: it puts the pre-fix footer back, sets
## `min_size.x` to the 1372 px the pane actually wanted, re-runs the window's
## own `_popup_full()`, and asks whether the picker became reachable.
##
## The measurement that decides it is in ROOT-viewport coordinates, not
## window-local ones. An embedded sub-window that is wider than the viewport it
## is embedded in reports its own children as perfectly inside *itself* -- the
## clip that matters has moved one level out, which is the same reason
## `_visible_rect()` has to carry the `Window` term at all.
func _counterfactual(w: Node, win: Window, foot: BoxContainer) -> void:
	## Premise: `_popup_full()` is what turns `min_size` into a window size, and
	## on a phone its FIRST line returns before it sizes anything (§13 gives the
	## handset the whole screen). Raising a minimum that nothing reads measures
	## nothing.
	if DccTheme.is_phone():
		_skip("counterfactual: `_popup_full()` returns early on a phone, so `min_size` is never read")
		return
	_log("--- counterfactual: raise min_size.x to the 1372 px the pane wanted ---")
	var note: Label = null
	for ch in foot.get_children():
		if ch is Label:
			note = ch as Label
			break
	var pickers := _pickers(w, [])
	if note == null or pickers.is_empty():
		_check(false, "counterfactual: export_world offers a note and a picker to measure")
		return
	var picker := pickers[0] as Control
	var keep_flags := note.size_flags_horizontal
	var keep_overrun := note.text_overrun_behavior
	var keep_min := note.custom_minimum_size.x
	var keep_clip := note.clip_text
	var keep_win_min := win.min_size
	note.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	note.text_overrun_behavior = TextServer.OVERRUN_NO_TRIMMING
	note.custom_minimum_size.x = 0.0
	note.clip_text = false
	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	foot.add_child(spacer)
	foot.move_child(spacer, 1)
	await _frames(4)
	win.min_size = Vector2i(int(w.get_contents_minimum_size().x), keep_win_min.y)
	w._popup_full()
	await _frames(8)
	var root_vp: Vector2 = get_viewport().get_visible_rect().size
	var edge_in_root := picker.get_global_rect().end.x + float(win.position.x)
	_log("  min_size.x=%d -> window %s at %s inside a %s viewport"
		% [win.min_size.x, str(win.size), str(win.position), str(root_vp)])
	_log("  %s right edge: %.0f window-local, %.0f in the root viewport, shown-in-window=%.3f"
		% [(picker as Button).text, picker.get_global_rect().end.x, edge_in_root, _shown(picker)])
	## Premise: the counterfactual only says anything where the pane's wanted
	## width exceeds the viewport. On a 1600 px tablet screen a 1372 px window
	## fits, so raising the minimum there costs nothing and proves nothing --
	## which is exactly the "the fix is harmless here, not needed here" case,
	## and asserting through it produced a `-18 px` failure that was arithmetic,
	## not a finding.
	if float(win.min_size.x) > root_vp.x + 0.5:
		_check(edge_in_root > root_vp.x + 0.5,
			"raising min_size does NOT make the picker reachable: it moves %.0f px past the application's own %.0f px viewport"
				% [edge_in_root - root_vp.x, root_vp.x])
	else:
		_log("  (not exercised: a %d px window fits this %.0f px viewport, so raising"
			% [win.min_size.x, root_vp.x])
		_log("   the minimum costs nothing HERE. It is the narrow viewport that decides.)")
	## Put everything back, and assert that it went back -- a probe that leaves
	## a window's declared minimum moved has changed the thing under test.
	win.min_size = keep_win_min
	foot.remove_child(spacer)
	spacer.queue_free()
	note.size_flags_horizontal = keep_flags
	note.text_overrun_behavior = keep_overrun
	note.custom_minimum_size.x = keep_min
	note.clip_text = keep_clip
	w._popup_full()
	await _frames(8)
	_check(win.min_size == keep_win_min and _shown(picker) > 0.999,
		"the counterfactual left the window as it found it (min_size=%s, picker shown=%.3f)"
			% [str(win.min_size), _shown(picker)])

func _ready() -> void:
	_tag = _arg("--tag", "panemin")
	_only = _arg("--route", "")
	_verbose = "--verbose" in OS.get_cmdline_user_args()
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1152x648").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(Vector2i(int(parts[0]), int(parts[1])))
	await _frames(6)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)
	var vp: Vector2 = app.get_viewport_rect().size
	_log("window %s  viewport %s  phone=%s tablet=%s"
		% [DisplayServer.window_get_size(), vp, DccTheme.is_phone(), DccTheme.is_tablet()])

	var w: Node = app.data_manager_window
	app.open_data_manager()
	await _frames(8)
	var win := w as Window
	_log("data manager size=%s min_size=%s contents_min=%s"
		% [win.size, win.min_size, w.get_contents_minimum_size()])

	## **The declared minimum is a promise about the width the window works at.**
	## Measured against every route, not the one the defect was reported on.
	var worst := 0.0
	var worst_route := ""
	for r in w.ROUTES:
		var route := String(r["id"])
		if _only != "" and route != _only:
			continue
		w.open_route(route)
		await _frames(8)
		var cmin: Vector2 = w.get_contents_minimum_size()
		if cmin.x > worst:
			worst = cmin.x
			worst_route = route
		var body: Control = w._pane_body
		var body_min := body.get_combined_minimum_size().x if body != null else -1.0
		if body_min > _worst_body:
			_worst_body = body_min
			_worst_body_route = route
		var foot: Control = w._pane_footer
		var foot_min := foot.get_combined_minimum_size().x if foot != null else -1.0
		if foot_min > _worst_foot:
			_worst_foot = foot_min
			_worst_foot_route = route
		var room := float(win.size.x)
		_log("route %-12s contents_min.x=%7.1f  pane_body_min.x=%7.1f  pane_footer_min.x=%7.1f  window=%d  over=%+.0f"
			% [route, cmin.x, body_min, foot_min, win.size.x, cmin.x - room])
		## **Walk from the WINDOW's own root child, not from `_pane_body`.**
		## The first cut of this probe walked the body and reported a 546 px
		## leaf under a 1372 px total -- the body is not where the width comes
		## from, and a chain rooted below the offender can only ever exonerate
		## everything it visits.
		var top: Control = null
		for ch in win.get_children():
			if ch is Control:
				top = ch as Control
		if _verbose and top != null:
			_log("    chain: %s" % " > ".join(_widest_chain(top, 1.0)))
		elif top != null and cmin.x > room:
			var chain := _widest_chain(top, 1.0)
			_log("    widest leaf: %s" % chain[chain.size() - 1])
			_log("    chain: %s" % " > ".join(chain))
		if foot is BoxContainer:
			_log("    footer terms: %s" % _hbox_terms(foot as BoxContainer))
			## **`await`, and it is load-bearing.** This became a coroutine when
			## the path-length leg was added, and calling a coroutine without
			## `await` starts it and returns -- so it sat suspended at its own
			## `await _frames(4)` while `_ab()` below stripped the note's
			## properties for the pre-fix reconstruction, and then resumed and
			## read `custom=0 overrun=NO_TRIMMING` as if that were the state
			## the fix produces. It reported the fix failing its own headline
			## property on all three export routes. Diagnosed by printing the
			## four properties instead of reasoning about Godot's Label.
			await _footer_note_legs(w, win, route, foot as BoxContainer)
		## Every picker on this route, drawn rect against the VISIBLE rect.
		for b in _pickers(w, []):
			var btn := b as Button
			var own := btn.get_global_rect()
			var vis := _visible_rect(btn)
			var sh := _shown(btn)
			_log("    picker %-10s own=[%.0f..%.0f] vis=[%.0f..%.0f] shown=%.3f"
				% [btn.text, own.position.x, own.end.x, vis.position.x, vis.end.x, sh])
			## **The width claim is horizontal containment.** `shown` also
			## counts the vertical axis, and on a handset the route pane is one
			## tall scrolling column -- a picker below the fold reads
			## `shown=0.000` while sitting at x 404 inside a 1080 px window,
			## which is a scroll position, not an overflow. Measured at
			## `--force-touch --vp 1080x2340`. So the horizontal test is the one
			## asserted everywhere; `shown` is asserted where nothing has
			## scrolled it away.
			_check(own.position.x >= -0.5 and own.end.x <= float(win.size.x) + 0.5,
				"%s: %s is inside the window HORIZONTALLY (%.0f..%.0f vs 0..%d)"
					% [route, btn.text, own.position.x, own.end.x, win.size.x])
			if DccTheme.is_phone():
				_skip("%s: %s full-area leg -- a handset pane scrolls vertically, so shown=%.3f is a scroll position"
					% [route, btn.text, sh])
			else:
				_check(sh > 0.999,
					"%s: %s is fully drawn inside the window (shown=%.3f, right edge %.0f vs window %d)"
						% [route, btn.text, sh, own.end.x, win.size.x])
		_check(cmin.x <= room + 0.5,
			"%s: the route's contents fit the window it opens at (%.0f <= %d)"
				% [route, cmin.x, win.size.x])
		## Only the routes that actually carry a picker -- the A/B's assertion
		## is about a picker being pushed off the edge, and a route with none
		## would make it unsatisfiable.
		if foot is BoxContainer and not _pickers(w, []).is_empty():
			await _ab(w, win, route, foot as BoxContainer, cmin.x)

	## The pixel reading, on the route that carries the longest note -- the one
	## the trim actually bites on. Geometry has already said the box is there.
	if _only == "" or _only == "export_world":
		w.open_route("export_world")
		await _frames(8)
		await _ink_control(win, "export_world", w._pane_footer as BoxContainer)
		await _counterfactual(w, win, w._pane_footer as BoxContainer)

	## **A guard that cannot fail is not a guard.** Every per-route picker leg is
	## premise-guarded, so at a wide viewport all of them correctly stand down --
	## and the run would then be green having proved nothing about the fix. This
	## says out loud which of the two cases this run is, and requires the break
	## to have been demonstrated whenever the pre-fix minimum could reach it.
	_log("A/B: worst PRE-FIX minimum %.0f px, window %d, routes whose picker it broke: %d"
		% [_pre_worst, win.size.x, _pre_broke])
	_check(_pre_worst > float(win.min_size.x) + 0.5,
		"the PRE-FIX state really did exceed the window's DECLARED minimum (%.0f > %d)"
			% [_pre_worst, win.min_size.x])
	if _pre_worst > float(win.size.x) + 0.5:
		_check(_pre_broke > 0,
			"this run DID exercise the defect (%d route(s) broken before the fix)" % _pre_broke)
	else:
		_log("  note: %d px is wider than the worst pre-fix minimum (%.0f), so this run"
			% [win.size.x, _pre_worst])
		_log("        shows the fix is harmless here, NOT that it is needed here.")

	## **A READING, not an assertion.** The declared minimum has two numbers and
	## this lane fixed the width one. The height one is measured here and left
	## alone deliberately: `_popup_full()` opens at
	## `maxi(viewport.y - H_MENU_BAR, min_size.y)`, so at a viewport exactly as
	## tall as `min_size.y` the window is `H_MENU_BAR` px taller than the space
	## below the menu bar and its own bottom edge hangs off. Whether that is a
	## defect or an accepted floor is not a width question and is not decided
	## here -- it is printed so the next pass does not have to rediscover it.
	var root_vp: Vector2 = get_viewport().get_visible_rect().size
	var bottom_in_root := float(win.position.y) + float(win.size.y)
	_log("READING (vertical, not asserted): window y %d..%.0f inside a %.0f px viewport -> %+.0f px"
		% [win.position.y, bottom_in_root, root_vp.y, bottom_in_root - root_vp.y])

	## **Two claims, kept apart, because they have different owners.**
	##
	## The first is this lane's: the pane FOOTER is no longer what decides how
	## wide this window has to be. It is checked at every density.
	##
	## The second is the window's own declared-minimum promise, which the body
	## can break without the footer's help -- and at touch density it does:
	## measured 2026-09-07 at `--force-touch --vp 1600x1000`, `export_maps`'
	## body alone asks **887 px** (against 698 at pointer), putting the window
	## at 1176 over a declared 1024. That is a pre-existing touch-density
	## defect in the pane BODY, not in the footer, and this lane did not touch
	## it -- the message says so rather than leaving a red line to be
	## misattributed.
	var chrome := _chrome_w(w)
	_log("worst route %s at %.0f px against declared min_size.x=%d"
		% [worst_route, worst, win.min_size.x])
	_log("  widest body: %s at %.0f   widest footer: %s at %.0f   chrome (rail+pad): %.0f"
		% [_worst_body_route, _worst_body, _worst_foot_route, _worst_foot, chrome])
	## The driver is whichever of the two terms is larger, compared directly.
	## Deriving it from `worst - chrome` instead was wrong in both directions:
	## `contents_min` carries ~9 px more than `body + rail + pad` (the status
	## line and the window bar), so an exact identity test named the FOOTER at
	## 1152 where the body is 698 against the footer's 637, and again at tablet
	## where the body is 887.
	var driver := "BODY" if _worst_body >= _worst_foot else "FOOTER"
	_check(worst <= _promised_w(win) + 0.5,
		"every route fits %s (%.0f <= %.0f) -- widest term is the %s (body %.0f @%s vs footer %.0f @%s)%s"
			% [_promise_name(win), worst, _promised_w(win), driver,
				_worst_body, _worst_body_route, _worst_foot, _worst_foot_route,
				"; a BODY failure is pre-existing and outside the footer fix"
					if driver == "BODY" else ""])
	## The phone is a reading, not this lane's claim. `_worst_foot` there is
	## 637 px of which the note is 160 and the three `export_world` chips are
	## 477: deleting the note outright would still leave 765 px against a 393 dp
	## pane, so the note is not what breaks the handset and asserting it here
	## would file a chip-width defect under a footer-note fix.
	if DccTheme.is_phone():
		_log("  READING (phone, not asserted): footer %.0f + chrome %.0f vs a %.0f px window."
			% [_worst_foot, chrome, _promised_w(win)])
		_log("    The note contributes %d of that; the chips are the rest and are pre-existing."
			% w.FOOT_NOTE_MIN_W)
	else:
		_check(_worst_foot + chrome <= _promised_w(win) + 0.5,
			"THIS LANE'S CLAIM: the footer never decides the window's width above %s (%.0f + %.0f <= %.0f, widest at %s)"
				% [_promise_name(win), _worst_foot, chrome, _promised_w(win), _worst_foot_route])

	## **Positive control.** A measurement that cannot see a clip cannot fail,
	## and every green line above would be vacuous. A throwaway control is
	## parked deliberately past the window's right edge; `_shown()` must report
	## it mostly gone. Nothing else in this probe proves the intersection term
	## is doing any work.
	var spy := ColorRect.new()
	spy.size = Vector2(200.0, 20.0)
	spy.position = Vector2(float(win.size.x) - 20.0, 40.0)
	spy.mouse_filter = Control.MOUSE_FILTER_IGNORE
	win.add_child(spy)
	await _frames(3)
	var spy_shown := _shown(spy)
	_log("  [control] a 200 px strip parked 20 px inside the right edge: shown=%.3f" % spy_shown)
	_check(spy_shown < 0.2,
		"the measurement CAN see a control leaving the window (shown=%.3f, want <0.2)" % spy_shown)
	spy.queue_free()

	_log("RESULT %s fail=%d skipped=%d" % [_tag, _fail, _skipped])
	get_tree().quit(1 if _fail > 0 else 0)
