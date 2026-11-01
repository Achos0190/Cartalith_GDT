## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.9 (milestone PC-9, finding M8): Q (the cartwheel at the pointer) and
## Shift+F10 (the context card) move out of `app.gd::_unhandled_key_input` "into the rebindable table,
## and into the reference". They are the accelerators of two Edit rows now (`03_menus_edit.gd`), so
## `_bind_accelerator` makes them rebindable and the reference lists them. Two halves cannot move and
## stay here: the cartwheel's key-UP (a hold-aim-release gesture; a menu accelerator fires on the press
## only), which now follows the row's binding, and the Menu key, the card's second key (one
## `PopupMenu` item holds one accelerator), which `08_shortcuts_dialog.gd` lists.
##
## Target: `shell/app.gd` (`DccApp`). Depends on `01_menus_ids_helpers.gd` (`menus.cartwheel_keycode()`).
## **Lands in the same change as `03_menus_edit.gd` and `08_shortcuts_dialog.gd`.** Alone, this would
## leave nothing opening the cartwheel; 03 alone would call the two methods below before they exist.
## Applies to: the PC and the tablet. On the phone the menu bar is a hidden model node, so no menu
## accelerator fires there and the two rows are not built: a phone with a hardware keyboard keeps the
## Menu key and loses Q and Shift+F10, which it has today. A stated drop (README).
## Forms: one.
##
## Behaviour changes, stated (`MISTAKES.md`, "Change behaviour"):
##   - Ctrl+Q and Shift+Q no longer open the cartwheel; the old `event.keycode == KEY_Q` test ignored
##     modifiers, and a menu accelerator matches them exactly.
##   - A rebind of the cartwheel's key moves both the press (the row) and the release (this file).
##   - Typing Q or Shift+F10 in a text field still opens nothing (`menus._typed_into_field()`).
##
## What to do:
##   REPLACE  `_unhandled_key_input()` together with its doc block: from the line
##            `## §4.5.6: "Escape commits an in-progress multi-click tool... and otherwise` down to the
##            function's last line, `get_viewport().set_input_as_handled()` under the ⇧J branch, i.e.
##            everything above `## Delete whatever is selected, in whichever workspace owns a
##            selection.` (125 lines at a38e0ac), with the version at the end of this file. It was
##            produced from the shipped text by five exact replacements, each checked to occur once,
##            so every branch it does not name (Escape, Backspace, Ctrl+Y, Delete) is byte-identical
##            to the shipped one: (1) a paragraph appended to the doc; (2) the key-up branch tests
##            `menus.cartwheel_keycode()` instead of `KEY_Q`; (3) the `KEY_Q` press branch is deleted;
##            (4) the Menu/Shift+F10 branch keeps `KEY_MENU` only and calls `open_context_card()`;
##            (5) one stale sentence in the ⇧J branch's comment is corrected (it said ⇧J was not yet
##            in `shortcuts_dialog.gd::UNLISTED`; the table's "Shift+J" row has been there since).
##   ADD      `open_context_card()` and `open_cartwheel_at_pointer()` below, directly after
##            `_open_context_card_from_keyboard()`. Both names are free in `shell/` at a38e0ac
##            (`check_symbols.sh`).
##
## Verify: the parse check on `res://shell/app.gd`, then `res://shell/menus.gd`. `check_symbols.sh`.
## `_ctxring_probe.gd` (windowed, clean user dir so no stored rebind moves Q): its Q leg pushes Q
## down and up through `_vp.push_input()`, which now reaches the row's accelerator for the press and
## this file for the release, and its M leg pushes the Menu key and Shift+F10; both legs must stay
## green unchanged. `_quickbar_ring_probe.gd`, `_quickbar_ringid_probe.gd` and
## `_quickbar_assistant_probe.gd` call `ring_key_press()` directly and are unaffected. None was run in
## this session (no Godot). By hand on the PC: Q held over the map opens the cartwheel and releasing
## over a slot picks it; Q tapped leaves it open; Edit ▸ Cartwheel at the pointer (mouse) opens it and
## a click on a slot picks it; rebind the row to another key in Preferences ▸ Keyboard shortcuts… and
## the hold gesture works on that key and not on Q; Shift+F10 and the Menu key both open the card;
## neither Q nor Shift+F10 does anything while a settlement name is being typed (real keys only: see
## `menus._typed_into_field()`'s note on probes).

## Edit ▸ Context card (Shift+F10 until rebound) and the Menu key: opens the map's context card at the
## pointer's last position over the map, or at its centre (`_open_context_card_from_keyboard()`, which
## stays the body). A public name, so `menus.gd` calls a public method rather than reaching into a
## private one. It opens at the pointer until §3.1's S3 anchors it at the selection; that change
## belongs in the body, and `menus.gd::CONTEXT_CARD_TIP` must change with it.
func open_context_card() -> void:
	_open_context_card_from_keyboard()

## Edit ▸ Cartwheel at the pointer (Q until rebound): opens the cartwheel where the pointer last was
## over the map, through the same `context_broker.ring_key_press()` the `KEY_Q` press branch of
## `_unhandled_key_input` called until PC-9, and, like that branch, does nothing while one is open.
##
## From the row's key, the key-UP of the hold-aim-release gesture arrives in `_unhandled_key_input`.
## From a mouse click on the row no key-up comes, and none is needed: while the ring is open,
## `map_overlay.gd` gives the next left press to `context_broker.ring_click()`, which re-aims at the
## click and calls `radial_ring.gd::release()`, so clicking a slot picks it. Must not release here to
## imitate a tap: telling a click from a key would need `Input.is_key_pressed()`, which a probe that
## pushes events through `Viewport.push_input()` (`_ctxring_probe.gd`'s Q leg) never updates.
func open_cartwheel_at_pointer() -> void:
	if context_broker == null or viewport == null or viewport.overlay == null:
		return
	if context_broker.ring_is_open():
		return
	context_broker.ring_key_press(viewport.overlay.last_mouse_pos())

## ── REPLACEMENT for `_unhandled_key_input()` and its doc block ──────────────────────────────────────

## §4.5.6: "Escape commits an in-progress multi-click tool... and otherwise
## disarms back to Inspect." A key, not a mouse button, so it belongs on
## `_unhandled_key_input` regardless of which control has focus.
##
## Delete joins it for the same reason (`PARITY_AUDIT.md` §5 item 4,
## reference block 2's own keydown at line 26096: Delete removes the
## selected place). Broadcast rather than dispatched, like the right click
## above -- and deliberately *after* the `LineEdit`/`TextEdit` guard below,
## because `_unhandled_key_input` still fires for a focused text field on
## some platforms and deleting a settlement while the user is editing its
## name would be the worst possible surprise.
##
## **⇧J is the odd one out and says so at its own branch.** It is not a
## tool-context key; it is a menu accelerator that lost its menu row when the
## Journey planner moved to the CIVIL rail on 2026-09-05, and this is where it
## landed so the shortcut did not die with the row.
##
## **Q and Shift+F10 left this function (PC_SHELL_PROPOSAL.md §3.9, finding
## M8).** They are the accelerators of two menu rows now, Edit ▸ Cartwheel at
## the pointer and Edit ▸ Context card (`menus.gd::_build_map_surface_rows()`),
## so they are rebindable (`_bind_accelerator`) and listed by Help ▸ Keyboard
## reference, which reads the live menus. Two halves stay here because no menu
## accelerator can carry them: the cartwheel's key-UP (a hold-aim-release
## gesture; a menu accelerator fires on the press only), which follows the
## row's binding through `menus.cartwheel_keycode()`, and the Menu key, the
## card's second key (a `PopupMenu` item holds one accelerator, the Ctrl+Y
## precedent below), listed in `shortcuts_dialog.gd::UNLISTED`. Each row's
## handler keeps this function's text-field guard (`menus._typed_into_field()`).
func _unhandled_key_input(event: InputEvent) -> void:
	## CM-3's Q: key-up matters as much as key-down (§6: "Q hold (key down ->
	## aim -> key up) / Q tap"), so this one branch has to run before the
	## function-wide `event.pressed` guard below rejects every key-up outright.
	## Not gated on the ring already being open -- a key-up with no matching
	## ring is a no-op in `context_broker.gd::ring_key_release()` regardless.
	## The PRESS is Edit ▸ Cartwheel at the pointer's accelerator since PC-9;
	## the key is that row's binding (Q until rebound), read on every key-up so
	## a rebind made in Preferences ▸ Keyboard shortcuts… applies at once.
	if event is InputEventKey and not event.pressed \
			and event.keycode == menus.cartwheel_keycode():
		if context_broker != null and context_broker.ring_key_release():
			get_viewport().set_input_as_handled()
		return
	if not (event is InputEventKey and event.pressed):
		return
	if event.keycode == KEY_ESCAPE:
		## An open ring closes on Esc rather than falling through to the
		## default disarm-to-Inspect (§9.4's "an open surface never commits or
		## discards... Esc and tap-outside close the surface only" -- the same
		## rule the card already follows).
		if context_broker != null and context_broker.ring_is_open():
			context_broker.ring_close()
			get_viewport().set_input_as_handled()
			return
		_escape_action()
		get_viewport().set_input_as_handled()
	elif event.keycode == KEY_BACKSPACE:
		## Same text-field guard Delete needs below, and for the same reason:
		## Backspace inside a `LineEdit` means "delete a character", never
		## "drop the last measured point".
		var typing := get_viewport().gui_get_focus_owner()
		if typing is LineEdit or typing is TextEdit or typing is SpinBox:
			return
		if _backspace_handlers.has(armed_tool):
			_backspace_handlers[armed_tool].call()
			get_viewport().set_input_as_handled()
	elif event.keycode == KEY_Y and event.ctrl_pressed and not (event.shift_pressed \
			or event.alt_pressed or event.meta_pressed):
		## Ctrl+Y: the Windows-convention SECOND Redo key. A `PopupMenu` item holds
		## exactly one accelerator (`Edit ▸ Redo` keeps Ctrl+Shift+Z, which is also
		## the macOS Cmd+Shift+Z via Godot's Ctrl/Cmd mapping), so the alias lives
		## here, next to the other keys that are not menu accelerators, and runs
		## the same `redo_last()` the menu row and the `↷` square run. `KEY_Y` was
		## unbound everywhere else (grepped across `shell/` 2026-10-10). A focused
		## text field keeps its own Ctrl+Y (it consumes the key before this runs;
		## the guard covers platforms where it still arrives).
		var typing_y := get_viewport().gui_get_focus_owner()
		if typing_y is LineEdit or typing_y is TextEdit or typing_y is SpinBox:
			return
		redo_last()
		get_viewport().set_input_as_handled()
	elif event.keycode == KEY_DELETE:
		var focused := get_viewport().gui_get_focus_owner()
		if focused is LineEdit or focused is TextEdit or focused is SpinBox:
			return
		if delete_selection():
			get_viewport().set_input_as_handled()
			return
	elif event.keycode == KEY_MENU:
		## CM-2 residual (`MAP_CONTEXT_SCOPE.md` §4.2, §6): "The Menu key /
		## Shift+F10 opens the card for the current selection, anchored at the
		## selection, or at the cursor when nothing is selected." This build has
		## no persistent object *selection* separate from the pointer (settlement
		## click selects for the dock, not for the card), so "the cursor" is the
		## only anchor a keyboard press can offer -- `open_context_card()`
		## falls back to the map's own centre when the cursor is off the plate.
		## **Shift+F10 is Edit ▸ Context card's accelerator since PC-9** (rebindable,
		## `menus.gd::_build_map_surface_rows()`); the Menu key stays here as its
		## fixed second key, because a `PopupMenu` item holds one accelerator (the
		## Ctrl+Y branch above is the same case), and `shortcuts_dialog.gd::
		## UNLISTED` lists it. `KEY_MENU` is matched nowhere else in the project.
		var typing := get_viewport().gui_get_focus_owner()
		if typing is LineEdit or typing is TextEdit or typing is SpinBox:
			return
		open_context_card()
		get_viewport().set_input_as_handled()
	elif event.keycode == KEY_J and event.shift_pressed \
			and not (event.ctrl_pressed or event.alt_pressed or event.meta_pressed):
		## **⇧J, which used to be a menu accelerator.** `Data ▸ Journey
		## planner… ⇧J` was retired on 2026-09-05 (owner ruling,
		## `LARGE_ITEM_RULINGS.md` item 4) and the planner moved to the CIVIL
		## rail node. A rail node has no accelerator column, so the key lands
		## here or nowhere -- and dropping a working shortcut to a layout change
		## is the same capability-loss-as-conformance the ruling forbids for
		## `Clear library…`.
		##
		## **Two consequences, stated because they are real costs, not nits.**
		## The key is no longer rebindable: `menus.gd::_shortcut_defaults` is
		## filled only by `_bind_accelerator()`, which only menu rows reach. And
		## the walk of the live menus that Help ▸ Keyboard reference does cannot
		## see it, so `shortcuts_dialog.gd::UNLISTED` lists it beside Esc, Delete
		## and Space (its "Shift+J" row; this said "Reported, not made" until PC-9
		## re-read the table, by which time the row had been installed).
		##
		## The text-field guard is Delete's and Backspace's, for the same reason:
		## `_unhandled_key_input` still fires for a focused `LineEdit` on some
		## platforms, and ⇧J while naming a settlement means the letter J.
		var typing := get_viewport().gui_get_focus_owner()
		if typing is LineEdit or typing is TextEdit or typing is SpinBox:
			return
		open_journey_planner()
		get_viewport().set_input_as_handled()
