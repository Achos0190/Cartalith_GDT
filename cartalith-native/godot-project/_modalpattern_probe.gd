extends Node
## Committed verification harness for the one modal pattern
## (`design/proposed-2026-09-05/Modal.dc.html`, owner-approved 2026-09-05) and
## for the shell call sites that adopted it.
##
## Three things are asserted, and none of them is readable from the source:
##
##   1. **The ordering rule.** "The safe action is the filled one and sits
##      last. The destructive action is text-only, never filled, and never the
##      rightmost." Read off the drawn action row -- the button order, the
##      `MODAL_KIND_META` each carries, and the `bg_color` of the stylebox each
##      button is actually wearing.
##   2. **The keyboard path, which a restyle is the classic way to lose.** Esc
##      and Enter are driven for real: Esc as an `InputEventKey` pushed at the
##      shell viewport, Enter as an `InputEventKey` through the dialog's own
##      `window_input`, which is the signal `modal_choices()` listens on.
##      A drawing test that skipped these would grade a regression green.
##   3. **The dashes.** Every field the artboard draws that this engine cannot
##      supply must render an em dash carrying its reason, not a plausible
##      number. Counted off the drawn labels.
##
## Colours are checked twice over, and the second check is what makes the first
## mean anything: the palette token is pinned against the ARTBOARD's own hex
## (an independent source, not the value read back out of `dcc_theme.gd`), and
## then the drawn node is compared against the token. Forced dark, because the
## artboard is the dark canvas and this machine boots light.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _modalpattern_probe.tscn

var app: Node
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	var tag := "ok  " if cond else "FAIL"
	print("MOD %s  %s%s" % [tag, name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _eq(name: String, got, want) -> void:
	_check(name, got == want, "got=%s want=%s" % [got, want])

# -- Reading the drawn card, never the dictionary that built it ---------------

func _labels(n: Node, out: Array) -> void:
	for c in n.get_children(true):
		if c is Label:
			out.append(c)
		_labels(c, out)

func _texts(n: Node) -> Array:
	var out: Array = []
	var ls: Array = []
	_labels(n, ls)
	for l in ls:
		out.append((l as Label).text)
	return out

## Every VISIBLE button under the card's own content root, in draw order --
## `AcceptDialog`'s hidden internal OK is excluded by the visibility test, and
## the header close box by starting the walk at the action row.
func _buttons(n: Node) -> Array:
	var out: Array = []
	if n is Button and (n as Button).visible:
		out.append(n)
	for c in n.get_children(true):
		out.append_array(_buttons(c))
	return out

func _kinds(row: Node) -> Array:
	var out: Array = []
	for b in _buttons(row):
		out.append(String((b as Button).get_meta(DccWidgets.MODAL_KIND_META, "?")))
	return out

func _bg(b: Button) -> Color:
	return (b.get_theme_stylebox("normal") as StyleBoxFlat).bg_color

func _ready() -> void:
	## Forced, and refused rather than assumed: the artboard is the dark
	## canvas, and this machine's `cartalith_settings.cfg` boots `mode="light"`.
	DccTheme.apply_theme(true)
	if not DccTheme.is_dark():
		print("MOD FAIL  could not force the dark palette -- refusing to run")
		get_tree().quit(1)
		return

	# --- 0. The token mapping, pinned against the artboard's own hexes -------
	## Each literal below is copied from `Modal.dc.html`'s `.tok` block, which
	## is a different file from the one being asserted. Mutating any of these
	## `DccTheme.DARK` entries turns this red.
	_eq("--pan  -> panel", DccTheme.c("panel"), Color("#121314"))
	_eq("--ins  -> sunken", DccTheme.c("sunken"), Color("#191c1e"))
	_eq("--bor  -> border", DccTheme.c("border"), Color(1, 1, 1, 0.16))
	_eq("--div  -> line_soft", DccTheme.c("line_soft"), Color(1, 1, 1, 0.07))
	_eq("--faint-> text_faint", DccTheme.c("text_faint"), Color("#6f7478"))
	_eq("--body -> text", DccTheme.c("text"), Color("#c8cbcd"))
	_eq("--sec  -> text_secondary", DccTheme.c("text_secondary"), Color("#a9adb0"))
	_eq("--dis  -> text_ghost", DccTheme.c("text_ghost"), Color("#5f6468"))
	_eq("--acc  -> accent", DccTheme.c("accent"), Color("#e0a34a"))
	_eq("--accInk -> accent_ink", DccTheme.c("accent_ink"), Color("#141005"))
	_eq("--block-> block", DccTheme.c("block"), Color("#c96a5a"))
	## `--wash2:rgba(224,163,74,.16)` -- the alpha B's destructive fill borrows.
	_check("--wash2 alpha is .16",
		abs(DccTheme.c("accent_wash_2").a - 0.16) < 0.005,
		str(DccTheme.c("accent_wash_2").a))
	## `--pad:14px`. The ROLE entry, not a pasted 14.
	_eq("--pad -> role_px(bar_pad_x)", DccTheme.role_px("bar_pad_x"), 14)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)
	DccTheme.apply_theme(true)

	await _variant_a()
	await _variant_b()
	await _variant_c()
	await _keyboard()
	await _call_sites()

	print("=== MODAL PATTERN ", "OK" if _fail == 0 else "FAILED (%d)" % _fail, " ===")
	get_tree().quit(0 if _fail == 0 else 1)

# ---------------------------------------------------------------------------

func _variant_a() -> void:
	print("[A] confirm -- cancel / destructive / safe")
	var card := DccWidgets.modal_card(app, "Close world", DccWidgets.MODAL_CONFIRM)
	DccWidgets.modal_prose(card["body"], "Northern Reach has changes that are not saved.")
	var stats := DccWidgets.modal_inset(card["body"])
	DccWidgets.modal_stat(stats, "edits since save", "9")
	DccWidgets.modal_stat_absent(stats, "last autosave", "none written yet")
	var res := DccWidgets.modal_choices(card, {
		"cancel": "Cancel",
		"destructive": {"text": "Discard", "on": func(): pass},
		"safe": {"text": "Save and close", "on": func(): pass},
	})
	DccWidgets.modal_present(card, app)
	await _frames(3)
	var dlg: AcceptDialog = card["dialog"]

	# The card itself, read off the drawn stylebox.
	var panel := dlg.get_theme_stylebox("panel") as StyleBoxFlat
	_eq("card ground is `panel`", panel.bg_color, DccTheme.c("panel"))
	_eq("card border is `border`", panel.border_color, DccTheme.c("border"))
	_eq("card border is 1 px", panel.border_width_top, 1)
	## **Literals, not `DccWidgets.MODAL_*`.** These two lines read the constant
	## back out of the source and compared it with itself, so all seven modal
	## metrics survived mutation and the label went on printing "10" while
	## measuring 3. `MISTAKES.md`'s "never assert a constant against itself" row,
	## caught by the batch-34 verifier. Every number below is the artboard's own
	## figure (`design/proposed-2026-09-05/Modal.dc.html`), carried here so a
	## mutant in `dcc_widgets.gd` dies.
	_eq("one radius, 10", panel.corner_radius_top_left, 10)
	_check("the shared shadow is on it",
		panel.shadow_size == 34 and panel.shadow_offset == DccWidgets.MODAL_SHADOW_OFFSET,
		"size=%d offset=%s" % [panel.shadow_size, panel.shadow_offset])

	## The four metrics nothing asserted at all, read off drawn nodes.
	var closer: Control = card["close"]
	_eq("the close box is --ctl, 24", closer.custom_minimum_size, Vector2(24, 24))
	_eq("the card is 340 wide", (card["root"] as Control).custom_minimum_size.x, 340.0)
	## `modal_inset()` returns the inner `VBoxContainer`; the stylebox is on its
	## `PanelContainer` parent. Reading it off the returned node measured 0 and
	## the assertion failed honestly rather than passing on the wrong node.
	var inset_sb := (stats.get_parent() as Control).get_theme_stylebox("panel") as StyleBoxFlat
	_eq("the stat inset is radius 8", inset_sb.corner_radius_top_left, 8)
	_eq("a stat row is 19 high", (stats.get_child(0) as Control).custom_minimum_size.y, 19.0)
	## `MODAL_BTN_H` is asserted further down, on the action row's own `btns` --
	## the action row does not exist yet at this point in the build.

	# Title: tracked caps in the faint ink, and NOT tinted on variant A.
	var t: Label = card["title"]
	_eq("title is tracked caps", t.text, "CLOSE WORLD")
	_eq("A's title is the faint ink", t.get_theme_color("font_color"),
		DccTheme.c("text_faint"))

	# The rule this pattern exists for.
	var kinds := _kinds(res["row"])
	_eq("three answers in artboard order", kinds,
		["quiet", "destructive_text", "safe"])
	var btns := _buttons(res["row"])
	_eq("an action button is --btnH, 28",
		(btns[0] as Control).custom_minimum_size.y, 28.0)
	var last := btns[btns.size() - 1] as Button
	var destructive := res["destructive"] as Button
	_eq("the SAFE action is last", String(last.get_meta(DccWidgets.MODAL_KIND_META)), "safe")
	_eq("and it is the filled one", _bg(last), DccTheme.c("accent"))
	_eq("its ink reads on amber", last.get_theme_color("font_color"),
		DccTheme.c("accent_ink"))
	_check("the destructive action is NOT rightmost",
		btns.find(destructive) < btns.size() - 1,
		"index=%d of %d" % [btns.find(destructive), btns.size()])
	_eq("and is not filled -- it wears the quiet `sunken` box",
		_bg(destructive), DccTheme.c("sunken"))
	_check("and it is never the accent fill",
		_bg(destructive) != DccTheme.c("accent"))
	_eq("its ink is `block`", destructive.get_theme_color("font_color"),
		DccTheme.c("block"))

	# The dash, read off the drawn label.
	var texts := _texts(card["body"])
	_check("the absent field draws an em dash", texts.has("—"), str(texts))
	_check("EDITS SINCE SAVE is not dashed -- it was given a value",
		texts.has("9"), str(texts))
	var ls: Array = []
	_labels(card["body"], ls)
	var dashed: Label = null
	for l in ls:
		if (l as Label).text == "—":
			dashed = l
	_check("the dash carries its reason on the node",
		dashed != null and dashed.tooltip_text != "",
		"tooltip=%s" % ("" if dashed == null else dashed.tooltip_text))
	_eq("and it is drawn in the ghost ink, not a value ink",
		dashed.get_theme_color("font_color"), DccTheme.c("text_ghost"))

	_check("the card is modal", dlg.exclusive, str(dlg.exclusive))
	dlg.queue_free()
	await _frames(2)

func _variant_b() -> void:
	print("[B] destructive -- the act IS the dialog")
	var card := DccWidgets.modal_card(app, "Clear library", DccWidgets.MODAL_DESTRUCTIVE)
	DccWidgets.modal_prose(card["body"], "Remove every pack from the asset library.")
	var stats := DccWidgets.modal_inset(card["body"], true)
	DccWidgets.modal_stat(stats, "packs", "128")
	var res := DccWidgets.modal_choices(card, {
		"cancel": "Cancel",
		"destructive": {"text": "Clear 128 packs", "on": func(): pass},
	})
	DccWidgets.modal_present(card, app)
	await _frames(3)

	var t: Label = card["title"]
	_eq("B tints its title", t.get_theme_color("font_color"), DccTheme.c("block"))
	## "…and only its title": the header divider stays the quiet one.
	var rule: ColorRect = null
	for c in (card["root"] as Node).get_children():
		if c is ColorRect:
			rule = c
	_eq("the header rule is NOT tinted", rule.color, DccTheme.c("line_soft"))
	## The inset's left rule is the other tinted thing the artboard draws.
	var pc := (stats.get_parent() as PanelContainer)
	var isb := pc.get_theme_stylebox("panel") as StyleBoxFlat
	_eq("the inset's left rule is tinted", isb.border_color, DccTheme.c("block"))
	_eq("2 px, left only", [isb.border_width_left, isb.border_width_right], [2, 0])

	var kinds := _kinds(res["row"])
	_eq("two answers", kinds, ["quiet", "destructive_filled"])
	var d := res["destructive"] as Button
	_eq("the count is named in the button", d.text, "Clear 128 packs")
	var bg := _bg(d)
	_check("it is NEVER the accent fill", bg != DccTheme.c("accent"),
		"bg=%s accent=%s" % [bg, DccTheme.c("accent")])
	_check("it is a block wash at --wash2's alpha",
		abs(bg.r - DccTheme.c("block").r) < 0.001
			and abs(bg.a - DccTheme.c("accent_wash_2").a) < 0.005,
		"bg=%s" % bg)
	_eq("its ink is `block`", d.get_theme_color("font_color"), DccTheme.c("block"))
	(card["dialog"] as AcceptDialog).queue_free()
	await _frames(2)

func _variant_c() -> void:
	print("[C] warnings -- a result, not a question")
	var card := DccWidgets.modal_card(app, "Import · 2 warnings",
		DccWidgets.MODAL_WARNINGS, 420)
	var list := DccWidgets.modal_list(card["body"])
	DccWidgets.modal_list_row(list, "warn", "pier_04.png — not power of two")
	DccWidgets.modal_list_row(list, "warn", "manifest — no licence field")
	DccWidgets.modal_list_row(list, "", "2 more")
	## Two captions offered, ONE class actually drawn. The legend must name the
	## drawn one and nothing else -- this is the check that the legend is wired
	## to the dots rather than written beside them.
	var legend := DccWidgets.modal_legend(card["body"], list,
		{"warn": "exports anyway", "block": "dropped"})
	var res := DccWidgets.modal_choices(card, {
		"safe": {"text": "Done", "on": func(): pass},
	})
	DccWidgets.modal_present(card, app)
	await _frames(3)

	var kinds := _kinds(res["row"])
	_eq("no Cancel on a result", kinds, ["safe"])
	var lt := _texts(legend)
	_eq("the legend names exactly the classes drawn", lt, ["●", "exports anyway"])
	_check("and NOT the class the data never produced",
		not lt.has("dropped"), str(lt))
	var seen: PackedStringArray = list.get_meta(DccWidgets.MODAL_DOTS_META)
	_eq("the dot register holds one class", seen, PackedStringArray(["warn"]))
	## The dotless tail row contributes nothing to the register.
	_check("the quiet tail row draws no dot", _texts(list).has("2 more"),
		str(_texts(list)))
	(card["dialog"] as AcceptDialog).queue_free()
	await _frames(2)

func _keyboard() -> void:
	print("[K] the keyboard path the stock dialog gave for free")

	# --- Enter fires the LAST button, driven through the real handler -------
	var fired := {"safe": 0, "destructive": 0}
	var card := DccWidgets.modal_card(app, "Enter", DccWidgets.MODAL_CONFIRM)
	DccWidgets.modal_prose(card["body"], "x")
	DccWidgets.modal_choices(card, {
		"cancel": "Cancel",
		"destructive": {"text": "Discard", "on": func(): fired["destructive"] += 1},
		"safe": {"text": "Save", "on": func(): fired["safe"] += 1},
	})
	DccWidgets.modal_present(card, app)
	await _frames(3)
	var dlg: AcceptDialog = card["dialog"]
	var ev := InputEventKey.new()
	ev.keycode = KEY_ENTER
	ev.physical_keycode = KEY_ENTER
	ev.pressed = true
	_check("the synthetic key really is ui_accept",
		ev.is_action_pressed("ui_accept", false, true))
	dlg.window_input.emit(ev)
	await _frames(3)
	_eq("Enter took the SAFE answer", fired["safe"], 1)
	_eq("and not the destructive one", fired["destructive"], 0)
	_check("and the card closed", not is_instance_valid(dlg) or dlg.is_queued_for_deletion())
	await _frames(2)

	# --- Esc cancels, driven as a real key at the shell viewport -----------
	var cancelled := {"n": 0}
	var card2 := DccWidgets.modal_card(app, "Escape", DccWidgets.MODAL_CONFIRM)
	DccWidgets.modal_prose(card2["body"], "x")
	DccWidgets.modal_choices(card2, {
		"cancel": "Cancel",
		"on_cancel": func(): cancelled["n"] += 1,
		"safe": {"text": "Save", "on": func(): pass},
	})
	DccWidgets.modal_present(card2, app)
	await _frames(3)
	var dlg2: AcceptDialog = card2["dialog"]
	var esc := InputEventKey.new()
	esc.keycode = KEY_ESCAPE
	esc.physical_keycode = KEY_ESCAPE
	esc.pressed = true
	get_viewport().push_input(esc)
	await _frames(4)
	var by_key: bool = cancelled["n"] == 1
	_check("Esc cancelled the card (real key at the viewport)", by_key,
		"n=%d" % cancelled["n"])
	if not by_key:
		## Headless input routing into an embedded subwindow is the one thing
		## here that can fail for reasons that are not the pattern's. Fall back
		## to the signal `AcceptDialog` itself raises on `ui_cancel`, so the
		## connection is still proved rather than assumed -- and SAY which
		## route produced the pass.
		if is_instance_valid(dlg2):
			dlg2.canceled.emit()
		await _frames(3)
		_eq("Esc path proved via AcceptDialog.canceled instead", cancelled["n"], 1)
	_check("and the card closed",
		not is_instance_valid(dlg2) or dlg2.is_queued_for_deletion())
	await _frames(2)

	# --- The header close box takes the same path -------------------------
	var cancelled3 := {"n": 0}
	var card3 := DccWidgets.modal_card(app, "Close box", DccWidgets.MODAL_CONFIRM)
	DccWidgets.modal_prose(card3["body"], "x")
	DccWidgets.modal_choices(card3, {
		"cancel": "Cancel",
		"on_cancel": func(): cancelled3["n"] += 1,
		"safe": {"text": "Save", "on": func(): pass},
	})
	DccWidgets.modal_present(card3, app)
	await _frames(3)
	(card3["close"] as Button).pressed.emit()
	await _frames(3)
	_eq("the header close box cancels", cancelled3["n"], 1)
	await _frames(2)

func _call_sites() -> void:
	print("[S] the converted call sites, opened for real")

	# --- app.gd :: confirm_unsaved_world -----------------------------------
	var dlg: AcceptDialog = app.confirm_unsaved_world(
		"Exit Cartalith", "Exit the app?", "Discard and exit", "Save and exit",
		func(): pass)
	await _frames(4)
	_check("the exit gate went up", is_instance_valid(dlg) and dlg.visible)
	_eq("its title survives for _backnav_probe", dlg.title, "Exit Cartalith")
	var texts: Array = []
	for b in _buttons(dlg):
		texts.append((b as Button).text)
	_check("Cancel offered", texts.has("Cancel"), str(texts))
	_check("Discard and exit offered", texts.has("Discard and exit"), str(texts))
	_check("Save and exit offered", texts.has("Save and exit"), str(texts))
	## The ordering rule at the site that matters most in the shell.
	var order: Array = []
	for b in _buttons(dlg):
		if (b as Button).has_meta(DccWidgets.MODAL_KIND_META):
			order.append(String((b as Button).get_meta(DccWidgets.MODAL_KIND_META)))
	_eq("Save is last, Discard is not", order,
		["quiet", "destructive_text", "safe"])
	## Every dash on the gate, with its reason.
	var ls: Array = []
	_labels(dlg, ls)
	var dashes := 0
	var reasoned := 0
	for l in ls:
		if (l as Label).text == "—":
			dashes += 1
			if (l as Label).tooltip_text != "":
				reasoned += 1
	print("    dashes=%d with-reason=%d" % [dashes, reasoned])
	_check("EDITS SINCE SAVE is dashed, not invented", dashes >= 1, "dashes=%d" % dashes)
	_eq("every dash carries its reason", reasoned, dashes)
	## The probe that owns this dialog resolves it with `hide()`.
	dlg.hide()
	await _frames(4)
	_check("hide() still frees it (_backnav_probe depends on this)",
		not is_instance_valid(dlg) or dlg.is_queued_for_deletion())

	# --- asset_library_window.gd :: _on_clear_library ----------------------
	var alw = app.asset_library_window if "asset_library_window" in app else null
	if alw == null:
		_check("asset library window reachable from the app", false,
			"no `asset_library_window` member")
		return
	alw._on_clear_library()
	await _frames(4)
	var wins: Array = []
	for c in alw.get_children(true):
		if c is AcceptDialog and (c as AcceptDialog).visible:
			wins.append(c)
	_check("the clear-library card went up", wins.size() == 1, "n=%d" % wins.size())
	if wins.is_empty():
		return
	var cl: AcceptDialog = wins[0]
	var ctexts := _texts(cl)
	print("    clear-library labels: ", ctexts)
	var kinds: Array = []
	var btexts: Array = []
	for b in _buttons(cl):
		if (b as Button).has_meta(DccWidgets.MODAL_KIND_META):
			kinds.append(String((b as Button).get_meta(DccWidgets.MODAL_KIND_META)))
			btexts.append((b as Button).text)
	_eq("Cancel then a block-washed destructive, nothing filled", kinds,
		["quiet", "destructive_filled"])
	_check("the button names the count", String(btexts[1]).begins_with("Clear "),
		str(btexts))
	_check("ON DISK is dashed", ctexts.has("—"), str(ctexts))
	_check("it states what is NOT destroyed",
		_texts(cl).any(func(s): return String(s).contains("not touched")),
		str(ctexts))
	cl.hide()
	await _frames(3)

	# --- asset_library_window.gd :: _on_validate (variant C, real data) -----
	## The legend claim is only worth anything against the validator's OWN
	## output, so this opens the real one rather than a fixture.
	var raw: PackedStringArray = alw._bridge.as_validate()
	print("    as_validate() returned %d warning(s): %s" % [raw.size(), raw])
	alw._on_validate()
	await _frames(4)
	var vwins: Array = []
	for c in alw.get_children(true):
		if c is AcceptDialog and (c as AcceptDialog).visible:
			vwins.append(c)
	_check("the validation card went up", vwins.size() == 1, "n=%d" % vwins.size())
	if not vwins.is_empty():
		var vd: AcceptDialog = vwins[0]
		var vkinds: Array = []
		for b in _buttons(vd):
			if (b as Button).has_meta(DccWidgets.MODAL_KIND_META):
				vkinds.append(String((b as Button).get_meta(DccWidgets.MODAL_KIND_META)))
		_eq("a result has no Cancel, and Done is last and filled", vkinds,
			["quiet", "safe"])
		var vt := _texts(vd)
		print("    validation labels: ", vt)
		var missing: Array = []
		for w in raw:
			if not vt.has(String(w)):
				missing.append(String(w))
		_check("every real warning is on screen", missing.is_empty(),
			"missing=%s" % str(missing))
		## One dot class, because `AssetValidator::run()` carries one.
		_check("the legend names at most one class -- the data has no second",
			vt.count("dropped") == 0, str(vt))
		vd.hide()
		await _frames(3)

	# --- asset_library_window.gd :: _prompt_text (Enter inside a field) -----
	var got := {"text": ""}
	alw._prompt_text("Rename", "New name:", "Village",
		func(t: String): got["text"] = t)
	await _frames(4)
	var pwins: Array = []
	for c in alw.get_children(true):
		if c is AcceptDialog and (c as AcceptDialog).visible:
			pwins.append(c)
	_check("the text prompt went up", pwins.size() == 1, "n=%d" % pwins.size())
	if not pwins.is_empty():
		var pd: AcceptDialog = pwins[0]
		var les: Array = []
		var stack: Array = [pd]
		while not stack.is_empty():
			var n: Node = stack.pop_back()
			if n is LineEdit:
				les.append(n)
			for c in n.get_children(true):
				stack.append(c)
		_check("it carries a field", les.size() == 1, "n=%d" % les.size())
		if les.size() == 1:
			var le: LineEdit = les[0]
			le.text = "Hamlet"
			## Enter INSIDE the field: a focused `LineEdit` eats the key before
			## `window_input` sees it, so this is the separate route.
			le.text_submitted.emit("Hamlet")
			await _frames(3)
			_eq("Enter in the field took the safe answer", got["text"], "Hamlet")
			_check("and the prompt closed",
				not is_instance_valid(pd) or pd.is_queued_for_deletion())
