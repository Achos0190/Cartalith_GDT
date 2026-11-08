extends RefCounted

## PC-7 (`PC_SHELL_PROPOSAL.md` §3.6, milestone PC-7; Ruling DY PQ3): the context windows that do
## not block the map and follow the selection until pinned. One place holds the rules, so the five
## windows (the faction roster, the Place editor, the City viewer, Conflicts and Saved journeys) carry
## two calls each and no copy of them.
##
## **What it does, on the desktop form only.** `make_context()` clears the window's `exclusive` flag
## (an `AcceptDialog` is born exclusive, so while one was open the map, the menus, the docks and the
## strip took no input: finding W1), and, for a window that has a subject, adds a FOLLOW / PINNED
## chip to its button row and connects it to `DccApp.selection_changed`. While the window is visible
## and following, a new selection is handed to the window's own `follow_selection(sel)`; a press on
## the chip pins it (the window keeps what it shows) and a second press follows again.
##
## **What it must never do.**
## - Change the phone or the tablet. The phone's windows are full-screen sheets, and §3.11 keeps the
##   tablet's windows modal ("a touch user cannot keep a floating window and the map both in reach"),
##   so `applies()` is false on both and nothing here runs there: no flag changes, no chip, no
##   connection. The phone captures are pixel-compared against the pre-PC-7 tree.
## - Discard a held change when the selection moves (the brief's hazard; §2 "a surface that must stay
##   on something stores its own subject as a pin"). A window answers `held_changes()` with the reason
##   it holds something that has no commit path yet (the Place editor's unsubmitted event, the
##   roster's open religion editor), or `""`. A new selection while it holds one PINS the window,
##   with that reason on the chip and the status line, instead of following; pressing the chip to
##   follow then asks before discarding (`DccWidgets.confirm`). A field the user is typing in is not a
##   held change in this sense: each window commits it through its own `_commit_focused_field()`, the
##   shipped path a re-open onto another subject already takes, so the text lands on the subject it
##   was typed for.
## - Follow while a generation runs. The engine belongs to the worker then (`bridge.generating`);
##   every window already re-reads on `generation_finished`, and the busy overlay mutes it meanwhile
##   (`busy_overlay.gd::_mute_subwindows`, which walks every embedded window, exclusive or not).
## - Let a key typed into a window's field reach the map's accelerators. Measured on Godot 4.7.2 before
##   this was written (`_pc7windows_probe.gd`, leg K): while an embedded window has the focus, the main
##   viewport forwards every key to it and its `_unhandled_key_input` never runs, so Q, Shift+A and
##   Delete stay in the field. The menu bar's chords were lost the same way (Ctrl+F opened no Find with
##   the window focused), so `context_key_relay.gd` carries Ctrl/Cmd and Alt chords,
##   and only those, to the menu bar; the probe pins both halves.
##
## Preloaded by each window (`const ContextWindow := preload(...)`), no `class_name`, so adding it
## needs no `--headless --import` pass (CLAUDE.md: that pass strips `project.godot`'s comments).

## The menu-chord relay every non-blocking window carries (`context_key_relay.gd`).
const KEY_RELAY := preload("res://shell/context_key_relay.gd")
## Name of that relay node, so a probe finds it.
const KEY_RELAY_NAME := "ContextKeyRelay"

## Name of the chip, so a probe finds it by name.
const CHIP_NAME := "FollowChip"
## Name of the invisible control that balances the chip's holder at the row's left end on a window
## with no Pop out button, as `PopoutHost.add_toggle_button()` balances its button, so the centred
## Close does not move.
const CHIP_BALANCE_NAME := "FollowChipBalance"
## Name of the chip's holder in the button row (see `make_context()` for why the chip has one).
const CHIP_HOLDER_NAME := "FollowChipHolder"
## Present (and `true`) while the window is pinned. ABSENT means following, which is the state every
## open starts in (`reset_on_open()`); there is no `false` value to confuse with "never set".
const META_PINNED := "_ctxwin_pinned"
## Why the window pinned itself (`held_changes()`'s reason), shown on the chip's tooltip. Absent when
## the user pinned it by hand.
const META_REASON := "_ctxwin_reason"
## The window's `held_changes()` answer is read through this method name.
const HELD_METHOD := "held_changes"
## The window's selection handler.
const FOLLOW_METHOD := "follow_selection"
## The window's "discard what you hold" handler, called after the user agreed (`follow()`).
const DISCARD_METHOD := "discard_held_changes"

## True where PC-7 applies: the desktop form. False on the phone and on the tablet (§3.11), and when
## `host` cannot say (a window built without a shell, as a probe may), so the shipped modal behaviour is
## kept wherever the answer is unknown.
static func applies(host) -> bool:
	if host == null or not host.has_method("is_phone"):
		return false
	return not host.is_phone() and not DccTheme.is_tablet()

## Makes `dlg` a non-blocking context window, from its `setup()` (after `DccWidgets.phone_window()`).
## `follows` is whether the window has a subject to follow (Saved journeys has none: it lists every
## journey in the world, and its rows open the planner). Returns the chip, or `null` when none was
## added (not the desktop, or `follows` false). Idempotent.
static func make_context(dlg: AcceptDialog, host, follows: bool) -> Button:
	if not applies(host):
		return null
	dlg.exclusive = false
	_wire_relay(dlg)
	if not follows:
		return null
	var existing := dlg.find_child(CHIP_NAME, true, false) as Button
	if existing != null:
		return existing
	var row := dlg.get_ok_button().get_parent() as HBoxContainer
	if row == null:
		return null
	## Where the chip goes. The row is `[Pop out][spacer][Close][spacer][Pop out's balance][grip]` on
	## a window with a Pop out button (`PopoutHost.add_toggle_button`), `[spacer][Close][spacer][grip]`
	## without one. **With Pop out, the chip takes its balance's place**: the balance is hidden and the
	## chip sits in a holder (`CHIP_HOLDER_NAME`) kept at the Pop out button's laid-out width, the same
	## rule the balance followed, so the two ends match and Close stays exactly where it was. Without Pop
	## out, the holder goes before the grip and a balance of the holder's width goes first.
	##
	## **The holder is not a Button, and that is load-bearing.** Measured (`_popout_windows_id_probe.gd`):
	## `AcceptDialog` resets the custom minimum size of every `Button` in its button row when its theme
	## applies (`buttons_min_width`, 0 in this theme), so a width set on the chip itself, or on Pop out,
	## was gone by the first draw; that is also why Pop out lays out at its own 81 px and not the 84 its
	## builder asks (`PopoutHost.BUTTON_MIN_W`). Two earlier builds moved Close by it: a chip beside Pop
	## out with a second balance outgrew the 400 px Place editor (Close 2 px, 475 pixels outside the
	## buttons), and a chip sized on itself lost its size (Close 3 px).
	var holder := MarginContainer.new()
	holder.name = CHIP_HOLDER_NAME
	holder.mouse_filter = Control.MOUSE_FILTER_IGNORE
	row.add_child(holder)
	var chip := DccWidgets.chip(holder, "FOLLOW", func() -> void: _on_chip(dlg, host), false, 7, 2)
	chip.name = CHIP_NAME
	chip.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var end := row.get_child_count() - 1
	if (row.get_child(end - 1) as Control) != null and (row.get_child(end - 1) as Control).tooltip_text == "Drag to resize":
		end -= 1
	var pop := row.get_node_or_null(PopoutHost.BUTTON_NAME) as Control
	var pop_bal := row.get_node_or_null(PopoutHost.BUTTON_NAME + "Balance") as Control
	if pop != null and pop_bal != null:
		row.move_child(holder, pop_bal.get_index())
		pop_bal.visible = false
		var fit := func() -> void: holder.custom_minimum_size.x = pop.size.x
		pop.resized.connect(fit)
		fit.call()
	else:
		row.move_child(holder, end)
		var balance := Control.new()
		balance.name = CHIP_BALANCE_NAME
		balance.mouse_filter = Control.MOUSE_FILTER_IGNORE
		row.add_child(balance)
		row.move_child(balance, 0)
		holder.resized.connect(func() -> void: balance.custom_minimum_size.x = holder.size.x)
	if host.has_signal("selection_changed"):
		host.selection_changed.connect(func(sel: Dictionary) -> void: _on_selection(dlg, host, sel))
	_sync_chip(dlg)
	return chip

## Wires the map-press relay (`_on_window_input()`) and the menu-chord relay (`KEY_RELAY`) on a
## non-blocking window. Called by
## `make_context()` for every window it makes non-exclusive, with or without a chip.
static func _wire_relay(dlg: AcceptDialog) -> void:
	if dlg.has_meta("_ctxwin_relay"):
		return
	dlg.set_meta("_ctxwin_relay", true)
	dlg.window_input.connect(func(ev: InputEvent) -> void: _on_window_input(dlg, ev))
	var keys: Node = KEY_RELAY.new()
	keys.name = KEY_RELAY_NAME
	dlg.add_child(keys)

## Whether `dlg` is following the selection now.
static func is_following(dlg: Object) -> bool:
	return dlg != null and not bool(dlg.get_meta(META_PINNED, false))

## Pins `dlg` to what it shows. `reason` is `""` for the user's own press, else the held change that
## made the window pin itself.
static func pin(dlg: Window, reason: String = "") -> void:
	dlg.set_meta(META_PINNED, true)
	if reason != "":
		dlg.set_meta(META_REASON, reason)
	elif dlg.has_meta(META_REASON):
		dlg.remove_meta(META_REASON)
	_sync_chip(dlg)

## Releases a pin without asking and without applying the selection (the window's own open path calls
## it, then shows its subject). See `follow()` for the chip's path.
static func reset_on_open(dlg: Window) -> void:
	if dlg.has_meta(META_PINNED):
		dlg.remove_meta(META_PINNED)
	if dlg.has_meta(META_REASON):
		dlg.remove_meta(META_REASON)
	_sync_chip(dlg)

## The chip's press on a pinned window: follow again, and show the current selection. When the window
## holds a change, ask first (finding: "follow must not discard edits: pin or ask"); Cancel keeps it
## pinned and changes nothing.
static func follow(dlg: Window, host) -> void:
	var held := _held(dlg)
	if held == "":
		_follow_now(dlg, host)
		return
	DccWidgets.confirm(host, "Follow the selection?",
		"This window holds %s. Following the selection discards it." % held, "Discard and follow",
		func() -> void:
			if is_instance_valid(dlg):
				if dlg.has_method(DISCARD_METHOD):
					dlg.call(DISCARD_METHOD)
				_follow_now(dlg, host))

## The text field inside `dlg` that has the keyboard focus, or `null`. A window keeps its focus owner
## while the user works on the map (measured: a click on the map takes the window's focus, not its
## field's), so this answers "is something half-typed in here" for the year handlers that must not
## tear a field down under the user (`defer_while_typing()`).
static func typing_field(dlg: Window) -> Control:
	if dlg == null or not dlg.is_inside_tree():
		return null
	var fo := dlg.gui_get_focus_owner()
	if fo == null or not dlg.is_ancestor_of(fo):
		return null
	if fo is LineEdit and (fo as LineEdit).editable:
		return fo
	if fo is TextEdit and (fo as TextEdit).editable:
		return fo
	return null

## Runs `refresh` now, or, when a field in `dlg` is being typed in, once that field gives up the focus
## (its own commit runs first, on the same `focus_exited`). Returns true when it deferred. Used by the
## year handlers: a rebuild tears every field down under its `_rebuilding` guard, which drops a
## half-typed value, and a year move is not a reason to drop it.
static func defer_while_typing(dlg: Window, refresh: Callable) -> bool:
	var f := typing_field(dlg)
	if f == null:
		refresh.call()
		return false
	var key := "_ctxwin_deferred"
	if f.has_meta(key):
		return true
	f.set_meta(key, true)
	f.focus_exited.connect(func() -> void:
		if is_instance_valid(dlg) and dlg.visible:
			refresh.call_deferred(), CONNECT_ONE_SHOT)
	return true

# -- the map-press relay -----------------------------------------------------------------------------

## A press on the map while a non-blocking window has the focus. **Measured on Godot 4.7.2**
## (`_pc7windows_probe.gd`, leg M): the main viewport hands EVERY mouse event to the focused embedded
## window, and only a LEFT press outside it takes that focus away (`Viewport::_sub_windows_forward_input`).
## So with the roster focused, a wheel notch over the map zoomed nothing, a right press opened no
## context card and a middle drag panned nothing, until the user left-clicked the map once. This gives
## such a press back to the map: it takes the window's focus away
## (`context_key_relay.gd::drop_focus()`), then pushes the
## same press into the main viewport, where it arrives exactly as it would have with no window open,
## and its release and drag follow it there by themselves (the window no longer has the focus).
##
## Only presses, never motion: moving the pointer off a window must not take its focus, or the next
## key typed into its field would reach the map's accelerators. Only outside the window's frame (the
## title bar and the resize margin count as the window). Never for a pop-out (a native window is its
## own OS window; a press outside it never reaches it). While the busy overlay mutes the window
## (`gui_disable_input`) nothing reaches this at all: the engine drops a muted window's input before
## `window_input` is emitted (measured: `_pc7windows_probe.gd` leg B, a wheel over the map from the
## muted, focused roster zooms nothing even with an explicit `gui_disable_input` test here removed),
## so a generate still blocks without a test of its own.
static func _on_window_input(dlg: Window, ev: InputEvent) -> void:
	if not (ev is InputEventMouseButton):
		return
	var mb := ev as InputEventMouseButton
	if not mb.pressed or mb.button_index == MOUSE_BUTTON_LEFT:
		return
	if not dlg.is_embedded() or not dlg.visible:
		return
	if frame_rect(dlg).has_point(mb.position):
		return
	var out := mb.duplicate() as InputEventMouseButton
	out.position = mb.position + Vector2(dlg.position)
	out.global_position = out.position
	KEY_RELAY.drop_focus(dlg)
	var root := dlg.get_tree().root
	root.push_input(out, true)

## The window's frame in its own coordinates: the content rect grown by the title bar above it and the
## theme's resize margin on every side, which is where the engine itself treats a press as the
## window's (a drag or a resize).
static func frame_rect(dlg: Window) -> Rect2:
	var title_h := float(dlg.get_theme_constant("title_height", "Window"))
	var m := float(dlg.get_theme_constant("resize_margin", "Window"))
	return Rect2(Vector2(-m, -title_h - m), Vector2(dlg.size) + Vector2(2.0 * m, title_h + 2.0 * m))

# -- internals ---------------------------------------------------------------------------------------

## A window's held-change reason, `""` when it has none or cannot say.
static func _held(dlg: Object) -> String:
	if dlg == null or not dlg.has_method(HELD_METHOD):
		return ""
	return String(dlg.call(HELD_METHOD))

## `selection_changed`: hands the new selection to a visible, following window, or pins the window
## when it holds a change. A hidden window does nothing (its next open reads the selection itself).
static func _on_selection(dlg: Window, host, sel: Dictionary) -> void:
	if not is_instance_valid(dlg) or not dlg.visible or not is_following(dlg):
		return
	if host.get("bridge") != null and bool(host.bridge.generating):
		return
	var held := _held(dlg)
	if held != "":
		pin(dlg, held)
		if host.has_method("set_status"):
			host.set_status("hint", "%s kept its subject: it holds %s. Press PINNED to follow the selection." \
				% [dlg.title, held], "accent")
		return
	if dlg.has_method(FOLLOW_METHOD):
		dlg.call(FOLLOW_METHOD, sel)

## The chip: FOLLOW (following) pins, PINNED follows.
static func _on_chip(dlg: Window, host) -> void:
	if is_following(dlg):
		pin(dlg)
	else:
		follow(dlg, host)

static func _follow_now(dlg: Window, host) -> void:
	reset_on_open(dlg)
	if dlg.has_method(FOLLOW_METHOD) and host.get("selection") != null:
		dlg.call(FOLLOW_METHOD, (host.selection as Dictionary).duplicate())

## Sets the chip's text, ink and tooltip from the window's state.
static func _sync_chip(dlg: Window) -> void:
	var b := dlg.find_child(CHIP_NAME, true, false) as Button
	if b == null:
		return
	var following := is_following(dlg)
	## The mockup's words and inks (`design/mobile-shell-2026-10-31/Shell.dc.html`, the window header:
	## `followLabel: fol ? 'FOLLOW' : 'PINNED'`, `followCol: fol ? 'var(--acc)' : 'var(--sec)'`).
	b.text = "FOLLOW" if following else "PINNED"
	b.add_theme_color_override("font_color", DccTheme.c("accent") if following else DccTheme.c("text_secondary"))
	if following:
		b.tooltip_text = "This window follows the selection. Press to pin it to what it shows now."
	elif dlg.has_meta(META_REASON):
		b.tooltip_text = "Pinned because it holds %s. Press to follow the selection (it asks before discarding)." \
			% String(dlg.get_meta(META_REASON))
	else:
		b.tooltip_text = "Pinned to what it shows. Press to follow the selection again."
