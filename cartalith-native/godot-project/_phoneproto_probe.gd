extends Node
## **Can a call site that completes the phone dialog protocol be told apart
## from one that does not, by measurement?**
##
## The protocol is three calls spread across build time and open time --
## `DccWidgets.phone_window()` before the body, `DccShell.phone_fit(dlg, 1.0)`
## after it, `DccWidgets.phone_present()` instead of `popup_centered()`.
##
## Re-censused 2026-09-07 against the tree rather than taken from the brief.
## **21** inline `AcceptDialog`/`ConfirmationDialog` constructions under
## `shell/`, and they land in four states, not two:
##
##   none        nothing runs                 9  faction_roster/_confirm_remove,
##                                               place_editor/confirm_delete,
##                                               right_dock/_confirm_revert,
##                                               civilization_workspace ×2,
##                                               world_workspace/_confirm_discard,
##                                               journey_planner_view ×2,
##                                               cartography_workspace/_prompt_label_name
##   backwards   fit, no window               2  vault_window ×2
##   partial     window + present, no fit     4  menus.gd ×4
##   ok          all three, or the card       6  app.gd ×3, asset_library slicer,
##                                               modal_card via modal_present,
##                                               and dcc_shell's desktop-only
##                                               Find-on-map dialog, which
##                                               `open_find_on_map()` never
##                                               reaches on a phone
##
## **Grep is where that started and not where it ended.** Grading the
## `extends AcceptDialog` class windows separately -- a population no
## `.new()` search reaches -- found a 16th non-conforming site the inline
## census cannot see: `shortcuts_dialog.gd` calls `phone_window()` in
## `setup()` and then opens with a bare `popup_centered()` twice, so on a
## phone it loses its title bar and stays desktop-sized and centred.
##
## **`backwards` is why this probe exists.** A site that gets it partly right
## performs a fit that is wrong rather than absent, and nothing in the tree
## caught that.
##
## Each specimen below is the shape of a REAL site, copied structurally rather
## than described, and every one is graded by the same
## `DccWidgets.phone_protocol_grade()` a future call site would be graded by.
## **The four bad specimens are asserted to FAIL first**; a conformance check
## that cannot fail is not evidence. Only then are the two new helpers, and the
## `modal_card()` path that deliberately skips `phone_fit()`, asserted to pass.
##
## Windowed, never `--headless`: a phone layout mode needs a real viewport size
## and every number here comes off a laid-out window.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _phoneproto_probe.tscn -- --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --path . _phoneproto_probe.tscn -- --desktop
##
## `--desktop` is the regression half: `confirm()` and `prompt()` must leave
## the desktop exactly as the six hand-built sites left it -- stock OS chrome,
## `dialog_text`, a centred window -- because converting that to the modal card
## is an owner call and not this lane's (`menus.gd::_open_pack_metadata`).

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("PROTO %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

# -- Specimens: the shapes the census actually found ---------------------------

## `faction_roster_window.gd::_confirm_remove` (and five siblings): a stock
## `ConfirmationDialog`, `dialog_text`, `popup_centered()`. Nothing else.
func _spec_none() -> AcceptDialog:
	var d := ConfirmationDialog.new()
	d.title = "Remove the last faction?"
	d.dialog_text = "Remove faction 3?\n\nAny settlements and territory using it will revert to Unclaimed."
	d.get_ok_button().text = "Remove"
	app.add_child(d)
	d.popup_centered()
	return d

## `vault_window.gd::_preview_dialog` -- the BACKWARDS shape. `phone_fit()`
## runs on a window `phone_window()` never touched, so `wrap_controls` is still
## on and the unscaled title bar is still there.
func _spec_backwards() -> AcceptDialog:
	var d := ConfirmationDialog.new()
	d.title = "Write to Markdown"
	d.size = Vector2i(620, 620)
	d.get_ok_button().text = "Write to Markdown"
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 6)
	d.add_child(col)
	DccWidgets.note(col, "This is what will be written into the note.")
	var te := TextEdit.new()
	te.text = "# heading\n\nbody line\n"
	te.editable = false
	te.custom_minimum_size.y = 260
	col.add_child(te)
	var cb := CheckBox.new()
	cb.text = "Don't ask again"
	col.add_child(cb)
	app.add_child(d)
	app.phone_fit(d, 1.0)
	d.popup_centered()
	return d

## `menus.gd::_prompt_forget_layout` -- the PARTIAL shape. Window and present
## both run; the fit never does, so the body keeps its desktop density.
##
## `phone_window()` last, exactly as that site orders it -- which is also what
## makes the OK-button check below a finding rather than a detail.
func _spec_partial() -> AcceptDialog:
	var d := ConfirmationDialog.new()
	d.title = "Forget layout"
	d.min_size = Vector2i(360, 0)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 8)
	body.add_child(DccTheme.label("Layout", "text_dim", DccTheme.FS_SMALL))
	var ob := OptionButton.new()
	ob.add_item("Default")
	ob.add_item("Layout 1")
	ob.selected = 0
	body.add_child(ob)
	d.add_child(body)
	d.ok_button_text = "Forget"
	DccWidgets.phone_window(d, app)
	app.add_child(d)
	if not DccWidgets.phone_present(d, app):
		d.popup_centered()
	return d

## **The `offscreen` term, exercised.** No hand-built site in the census
## reproduces it under `phone_present()`, which sets an explicit size through
## `Window.popup(rect)`; the recorded overflow -- the faction roster's
## Add/Remove row 1 750 dp below the bottom of the phone, in
## `DccShell.phone_fit()`'s own header -- came from a window still free to grow
## to its content's minimum. So this specimen places the control outside the
## client rect directly. Synthetic, and said to be: the term is what makes an
## unclipped `get_global_rect()` safe to read, and it survived mutation until
## something drove it.
func _spec_offscreen() -> AcceptDialog:
	var d := ConfirmationDialog.new()
	d.title = "Off the bottom"
	DccWidgets.phone_window(d, app)
	d.ok_button_text = "Add"
	## A bare `Control` parent, not a container: a `BoxContainer` would put the
	## button back inside on the next layout pass.
	var holder := Control.new()
	d.add_child(holder)
	var b := Button.new()
	b.text = "Add"
	b.custom_minimum_size = Vector2(120, 60)
	b.size = Vector2(120, 60)
	b.position = Vector2(0, 4000)
	holder.add_child(b)
	app.add_child(d)
	app.phone_fit(d, 1.0)
	if not DccWidgets.phone_present(d, app):
		d.popup_centered()
	return d

## A `modal_card()` opened with `popup_centered()` instead of
## `modal_present()`. **No site in the tree does this today** -- every
## `modal_card()` caller presents through `modal_present()` -- and it is here
## because without it the `wrap_controls` half of the shape leg is dead weight:
## every other specimen has `borderless` and `wrap_controls` agreeing, so
## dropping the term changed no verdict and survived mutation. The card is the
## one shape where they disagree, since `modal_card()` sets `borderless` on
## every platform and leaves `wrap_controls` **on** for the growth it wants.
func _spec_card_unpresented() -> AcceptDialog:
	var card := DccWidgets.modal_card(app, "Delete selection",
		DccWidgets.MODAL_DESTRUCTIVE)
	DccWidgets.modal_prose(card["body"], "Removes the selected packs.")
	DccWidgets.modal_choices(card, {"cancel": "Cancel",
		"destructive": {"text": "Delete", "on": func(): pass}})
	var d: AcceptDialog = card["dialog"]
	d.popup_centered()
	return d

# -- Measuring one specimen ---------------------------------------------------

func _grade(tag: String, d: Window) -> Dictionary:
	await _frames(6)
	var g: Dictionary = DccWidgets.phone_protocol_grade(d, app)
	print("PROTO      " + DccWidgets.phone_protocol_line(tag, g))
	return g

func _drop(d: Window) -> void:
	if is_instance_valid(d):
		d.hide()
		d.queue_free()
	await _frames(4)

func _ok_text(d: AcceptDialog) -> String:
	return String(d.get_ok_button().text)

# -- Entry --------------------------------------------------------------------

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)

	if "--desktop" in OS.get_cmdline_user_args():
		await _desktop_regression()
		print("PROTO  done fail=%d" % _fail)
		get_tree().quit(0 if _fail == 0 else 1)
		return

	## The probe must be able to ANSWER. Without `--force-touch` the shell is
	## not in phone mode and every grade below reads `desktop`, which is not a
	## pass -- so refuse to report at all.
	if not app.is_phone():
		print("PROTO  !! not in phone mode -- run windowed with -- --force-touch")
		get_tree().quit(1)
		return
	var scale: float = app.phone_scale()
	print("PROTO      phone=%s scale=%.4f viewport=%s tap_floor=%d dp" % [
		app.is_phone(), scale, str(app.get_viewport_rect().size),
		DccTheme.PHONE_TAP_MIN])
	_check("the handset scale is not 1.0, or nothing below can discriminate",
		scale > 1.5, "scale=%.4f" % scale)

	await _meta_literal()
	await _bad_specimens()
	await _good_specimens()

	print("PROTO  done fail=%d" % _fail)
	get_tree().quit(0 if _fail == 0 else 1)

## `DccWidgets.PHONE_FIT_META` is a literal copied out of `dcc_shell.gd`, and a
## copied literal that drifts would make every `fit_ran` below read 0 in
## silence. Checked by BEHAVIOUR -- run `phone_fit()` and watch the flag appear
## -- and not by comparing the constant against the constant it came from.
func _meta_literal() -> void:
	var holder := Control.new()
	var b := Button.new()
	b.text = "x"
	holder.add_child(b)
	app.add_child(holder)
	await _frames(2)
	_check("the fit flag is unset before phone_fit() runs",
		not b.has_meta(DccWidgets.PHONE_FIT_META))
	app.phone_fit(holder, 1.0)
	await _frames(2)
	_check("DccWidgets.PHONE_FIT_META is still the flag DccShell.phone_fit() writes",
		b.has_meta(DccWidgets.PHONE_FIT_META),
		"literal is '%s'" % DccWidgets.PHONE_FIT_META)
	holder.queue_free()
	await _frames(2)

## **The half that earns the check its authority.** Every one of these is the
## shape of a shipped site, and every one must be graded a failure.
func _bad_specimens() -> void:
	print("PROTO  -- known-bad specimens: the grade must FAIL on each --")

	var d1 := _spec_none()
	var g1: Dictionary = await _grade("none/faction_roster", d1)
	_check("none: graded a failure", String(g1["verdict"]).begins_with("fails:"),
		String(g1["verdict"]))
	_check("none: not phone-shaped", not bool(g1["shaped"]))
	_check("none: not phone-presented", not bool(g1["presented"]))
	await _drop(d1)

	var d2 := _spec_backwards()
	var g2: Dictionary = await _grade("backwards/vault", d2)
	_check("backwards: graded a failure", String(g2["verdict"]).begins_with("fails:"),
		String(g2["verdict"]))
	## The finding the census names and the reason a call list is not the rule:
	## the fit RAN, and it ran on a window that was never phone-shaped.
	_check("backwards: the fit ran and the window is still not phone-shaped",
		bool(g2["backwards"]),
		"fit_ran=%s shaped=%s" % [g2["fit_ran"], g2["shaped"]])
	await _drop(d2)

	var d3 := _spec_partial()
	var g3: Dictionary = await _grade("partial/menus", d3)
	_check("partial: graded a failure", String(g3["verdict"]).begins_with("fails:"),
		String(g3["verdict"]))
	## Shape and presentation are RIGHT here -- so a check that only looked at
	## the window would have passed this site and missed a body still laid out
	## at desktop density.
	_check("partial: it really is shaped and presented -- only the body is wrong",
		bool(g3["shaped"]) and bool(g3["presented"]) and not bool(g3["tap_ok"]),
		"worst %.0f dp is '%s'" % [g3["worst"], g3["worst_name"]])
	## Second defect at the same sites, and the one `confirm()` absorbs:
	## `phone_window()` assigns `ok_button_text = "Close"`, and these four sites
	## call it AFTER naming their own answer, so the phone loses the word.
	_check("partial: the caller's OK wording is lost to phone_window()'s 'Close'",
		_ok_text(d3) == "Close",
		"asked for 'Forget', drew '%s'" % _ok_text(d3))
	await _drop(d3)

	var d4 := _spec_offscreen()
	var g4: Dictionary = await _grade("offscreen/synthetic", d4)
	_check("offscreen: graded a failure", String(g4["verdict"]).begins_with("fails:"),
		String(g4["verdict"]))
	## Every control here clears the 44 dp floor. The ONLY thing wrong with it
	## is that one of them is outside the window -- which is the whole reason
	## the grade intersects against the client rect instead of trusting
	## `get_global_rect()`.
	_check("offscreen: it fails on position alone, with every control over the floor",
		int(g4["offscreen"]) > 0 and float(g4["worst"]) >= float(DccTheme.PHONE_TAP_MIN),
		"off=%d worst=%.0f dp" % [g4["offscreen"], g4["worst"]])
	await _drop(d4)

	var d5 := _spec_card_unpresented()
	var g5: Dictionary = await _grade("card-unpresented", d5)
	_check("card-unpresented: graded a failure",
		String(g5["verdict"]).begins_with("fails:"), String(g5["verdict"]))
	## The card is BORDERLESS on every platform, so `borderless` alone cannot
	## carry the shape leg -- only the pair with `wrap_controls` can.
	_check("card-unpresented: borderless is true and the shape leg still fails",
		d5.borderless and d5.wrap_controls and not bool(g5["shaped"]))
	await _drop(d5)

## The two new helpers, and the card path that deliberately skips `phone_fit()`
## and must still pass -- which is the whole reason the rule is written as a
## tap floor rather than as a list of calls.
func _good_specimens() -> void:
	print("PROTO  -- the helpers and the card: the grade must PASS --")

	var c := DccWidgets.confirm(app, "Remove the last faction?",
		"Remove faction 3?\n\nAny settlements and territory using it will revert to Unclaimed.",
		"Remove", func(): pass)
	var gc: Dictionary = await _grade("DccWidgets.confirm", c)
	_check("confirm(): conformant", String(gc["verdict"]) == "ok", String(gc["verdict"]))
	_check("confirm(): keeps the caller's OK wording on a phone",
		_ok_text(c) == "Remove", "drew '%s'" % _ok_text(c))
	## The borderless window gave up its title bar, so the question has to
	## carry the title itself or it is gone.
	_check("confirm(): the title survives the lost title bar",
		_has_text(c, "REMOVE THE LAST FACTION?"),
		"phone_head() draws it tracked-caps")
	await _drop(c)

	var p: Dictionary = DccWidgets.prompt(app, "Save journey",
		"Name for this journey:", "Journey 1", "Save", func(_t: String): pass,
		"Stored in this project -- written by File > Save project.")
	var gp: Dictionary = await _grade("DccWidgets.prompt", p["dialog"])
	_check("prompt(): conformant", String(gp["verdict"]) == "ok", String(gp["verdict"]))
	_check("prompt(): keeps the caller's OK wording on a phone",
		_ok_text(p["dialog"]) == "Save", "drew '%s'" % _ok_text(p["dialog"]))
	var field: LineEdit = p["field"]
	_check("prompt(): the field clears the tap floor",
		field.size.y >= float(DccTheme.PHONE_TAP_MIN),
		"%.0f dp" % field.size.y)
	await _drop(p["dialog"])

	var card := DccWidgets.modal_card(app, "Clear library", DccWidgets.MODAL_DESTRUCTIVE)
	DccWidgets.modal_prose(card["body"], "Removes every imported pack.")
	DccWidgets.modal_choices(card, {"cancel": "Cancel",
		"destructive": {"text": "Clear", "on": func(): pass}})
	DccWidgets.modal_present(card, app)
	var gk: Dictionary = await _grade("modal_card/modal_present", card["dialog"])
	## **The rule is not "all three calls ran".** `modal_present()` never calls
	## `phone_fit()`; the card clears the floor because `_modal_btn()` floors
	## its buttons at construction. A check written against the call list would
	## have failed the one path that was already right.
	_check("card: conformant without phone_fit() ever running",
		String(gk["verdict"]) == "ok" and not bool(gk["fit_ran"]),
		"%s fit_ran=%s" % [gk["verdict"], gk["fit_ran"]])
	await _drop(card["dialog"])

func _all(n: Node, out: Array) -> void:
	for c in n.get_children(true):
		out.append(c)
		_all(c, out)

func _has_text(n: Node, want: String) -> bool:
	var all: Array = []
	_all(n, all)
	for c in all:
		if c is Label and String((c as Label).text).strip_edges() == want:
			return true
	return false

## Desktop must not move. The six hand-built sites this replaces are stock OS
## chrome, and converting that to the modal card is an owner call.
func _desktop_regression() -> void:
	if app.is_phone():
		print("PROTO  !! --desktop asked for but the shell is in phone mode")
		get_tree().quit(1)
		return
	var c := DccWidgets.confirm(app, "Remove the last faction?",
		"Remove faction 3?", "Remove", func(): pass)
	await _frames(6)
	_check("desktop: still the stock dialog_text path",
		String(c.dialog_text) == "Remove faction 3?",
		"dialog_text='%s'" % c.dialog_text)
	_check("desktop: no phone header was built",
		not _has_text(c, "REMOVE THE LAST FACTION?"))
	_check("desktop: decoration kept", not c.borderless)
	_check("desktop: the OK wording is the caller's", _ok_text(c) == "Remove",
		"drew '%s'" % _ok_text(c))
	_check("desktop: not filling the screen",
		c.size.x < int(app.get_viewport_rect().size.x),
		"%d px of %d" % [c.size.x, int(app.get_viewport_rect().size.x)])
	_check("desktop: graded 'desktop', not silently passed as ok",
		String(DccWidgets.phone_protocol_grade(c, app)["verdict"]) == "desktop")
	await _drop(c)

	var p: Dictionary = DccWidgets.prompt(app, "Save journey",
		"Name for this journey:", "Journey 1", "Save", func(_t: String): pass)
	await _frames(6)
	var pd: AcceptDialog = p["dialog"]
	_check("desktop: prompt keeps its decoration", not pd.borderless)
	_check("desktop: prompt seeded the field", String((p["field"] as LineEdit).text) == "Journey 1")
	_check("desktop: prompt honours its width floor", pd.min_size.x == 360,
		"min_size=%s" % str(pd.min_size))
	await _drop(pd)
