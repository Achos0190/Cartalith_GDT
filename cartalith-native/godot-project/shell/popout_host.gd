extends RefCounted
class_name PopoutHost

## The one place that decides whether a tool window is a native OS window
## (`POPOUT_WINDOWS_SCOPE.md` s.4, milestones PW-1 and PW-2, Ruling DK).
##
## **Zero cost when off is the contract.** Every tool window passes through `prepare()` once, from
## `DccWidgets.phone_window()` (the call every one of them already makes in its `setup`), and again
## from the window's own `open()` so a setting changed at runtime applies to the next open. With the
## setting off and no manual choice, on a phone or a tablet, on a build without `FEATURE_SUBWINDOWS`,
## or for a window that is not in `LIVE_KEYS`, `prepare()` leaves `Window.force_native` at `false`: no
## native window is created, no signal is connected and no child is added, so the shell is exactly what
## it was. PW-0 measured why that matters: each open native window costs a fixed redraw every frame
## (+1.2 ms for one, +1.7 ms for three at a continuous redraw, `POPOUT_WINDOWS_SCOPE.md` s.8).
##
## What it must never do: set `force_native` on a window that is visible (the engine only honours it
## before the first show, so `toggle()` hides first), make a pop-out `exclusive` (the whole point is to
## use both screens at once, scope s.2.6), store the geometry of an EMBEDDED window
## (`PopoutGeometry.capture()` refuses it), or let a native window stay clickable during a generate
## (`native_windows()` is what `BusyOverlay` mutes, scope s.3).

## The window keys (`key_for()`) that actually pop out when the setting is on, or when the user
## presses the window's own "Pop out" button. **PW-2 added the faction roster**, the pilot: the two
## holes PW-0 measured are closed for it -- `BusyOverlay` mutes and mirrors a native window
## (`native_windows()`), and `PopoutKeyForwarder` carries Ctrl+Z, Ctrl+Y and Ctrl+S out of it
## (`forward_action()`). **PW-3 adds the other six v1 windows** (`POPOUT_WINDOWS_SCOPE.md` s.4 and Q4,
## Ruling DK): settlement types, place editor, generation rules, city viewer, vault and assistant.
## Both holes are closed generically (the overlay walks the `GROUP`, the forwarder is a child of every
## native window), so adding a key needs only the window's own two lines: `add_toggle_button(self)`
## in `setup()` and `prepare(self)` before each `popup_centered()`. A key joins this list only together
## with its button (`add_toggle_button`). Data Manager, Asset Library, Travel library, the assistant's
## settings, the quick-bar customiser and every file dialog are deliberately NOT here: they stay
## embedded (scope Q4), and a probe pins that.
const LIVE_KEYS: PackedStringArray = [
	"faction_roster_window", "settlement_types_window", "place_editor_window",
	"generation_rules_window", "city_viewer_window", "vault_window", "llm_assistant_window",
]

## The scene-tree group every window whose key is live joins (docked or not), so `BusyOverlay` can
## find the native ones without knowing which windows exist (`native_windows()`).
const GROUP := "popout_managed"
## Per-window, session-only memory of the button: present means "the user chose this window's mode by
## hand" (`true` = native). ABSENT means "follow the global setting". Absence is the no-choice value,
## never `false`, because `false` is a real choice (Dock while the setting is on).
const META_CHOICE := "_popout_choice"
## Present (and `true`) only while `toggle()` is between hiding a window and showing it again. A hide
## made by a MOVE is not a close: the assistant's `visibility_changed` handler ends a running reply
## on a hide (LLM-3, "no thread outlives the window"), and that must not fire for a move
## (`is_moving()`). Absent is the normal value; it is erased synchronously before `toggle()` yields.
const META_MOVING := "_popout_moving"
## Name of the per-window button (`add_toggle_button`), so a probe can find it by name.
const BUTTON_NAME := "PopoutButton"
## Name of the key-forwarding child (`PopoutKeyForwarder`).
const FORWARDER_NAME := "PopoutKeyForwarder"
## Frames `toggle()` waits after re-showing a window before it puts the scroll positions back: the
## scrollbars' maxima are only right once the window has been laid out in its new viewport.
const TOGGLE_SETTLE_FRAMES := 3
## Width of the per-window button and of its invisible balance control, in pixels. **A labelled
## judgement**: wider than the natural minimum of either label ("Pop out" measured 81 px with the DCC button padding, "Dock" is
## shorter, `_popout_roster_probe.gd` asserts both) so the label change never moves the button or, through
## the balance, Close; the probe pins the two widths equal.
const BUTTON_MIN_W := 84

const _META_PREPARED := "_popout_prepared"

## The window's key: its script's file name, else its title. Unlike `DccWidgets.window_key()` it has
## no `@desktop`/`@tablet` suffix, because only the desktop form can pop out at all (`supported()`).
static func key_for(dlg: Window) -> String:
	var s: Script = dlg.get_script()
	return s.resource_path.get_file().get_basename() if s != null else dlg.title

## Whether this process can show native windows at all: the display server says so
## (`DisplayServer.FEATURE_SUBWINDOWS`, absent on Android) and the shell is the desktop form. The
## phone and the tablet keep every window embedded (scope s.4 "Not offered").
static func supported() -> bool:
	return DisplayServer.has_feature(DisplayServer.FEATURE_SUBWINDOWS) \
		and not OS.has_feature("mobile") and not DccTheme.is_phone() and not DccTheme.is_tablet()

## The whole decision, as a pure function so a table of literals can pin each term: a window is
## native only when the user's wish is on AND the platform supports it AND the window's key is
## one of `live`. Dropping any one term changes the answer for some row of the probe's table.
## `setting_on` is the effective wish: the global setting, or the window's own manual choice
## (`META_CHOICE`) when it has one.
static func wants_native(setting_on: bool, platform_ok: bool, key: String, live: PackedStringArray) -> bool:
	return setting_on and platform_ok and live.has(key)

## The effective wish for one window: its own manual choice if it has made one, else the global
## setting. Pulled out of `prepare()` so the "button and setting agree" bar can be pinned by a table.
static func effective_wish(setting_on: bool, has_choice: bool, choice: bool) -> bool:
	return choice if has_choice else setting_on

## Decides and applies `force_native` for `dlg`, and (only when native) wires its geometry memory,
## its scale fit and its key forwarder. Returns whether the window is now native. Idempotent and
## safe to call before every open; a visible window is left as it is. `live` defaults to
## `LIVE_KEYS` and exists so the probe can drive the native branch with a scratch dialog without any
## real window being allowed to leave.
static func prepare(dlg: Window, live: PackedStringArray = LIVE_KEYS) -> bool:
	if dlg.visible:
		return dlg.force_native
	var key := key_for(dlg)
	var supported_here := supported()
	if supported_here and live.has(key) and not dlg.is_in_group(GROUP):
		dlg.add_to_group(GROUP)
	var wish := effective_wish(DccSettings.popout_enabled(), dlg.has_meta(META_CHOICE),
		bool(dlg.get_meta(META_CHOICE, false)))
	var native := wants_native(wish, supported_here, key, live)
	dlg.force_native = native
	if not native:
		return false
	## Transient, non-exclusive (scope s.2.6): destroyed with the main window, and the main window
	## stays usable while the pop-out is open.
	dlg.transient = true
	dlg.exclusive = false
	if not dlg.has_meta(_META_PREPARED):
		dlg.set_meta(_META_PREPARED, true)
		dlg.visibility_changed.connect(_on_visibility.bind(dlg))
		dlg.focus_entered.connect(_on_focus.bind(dlg))
	if dlg.get_node_or_null(FORWARDER_NAME) == null:
		var f := PopoutKeyForwarder.new()
		f.name = FORWARDER_NAME
		dlg.add_child(f)
	return true

## One handler for both directions. **Restore runs on SHOWING, not on `about_to_popup`**: measured in
## PW-1, `popup_centered()` sets its own centred rectangle AFTER `about_to_popup`, so a position set
## there was overwritten (the restore bar failed red with (580, 320) centred against the stored
## (200, 150)); `visibility_changed` fires after it.
static func _on_visibility(dlg: Window) -> void:
	if not is_instance_valid(dlg):
		return
	if dlg.visible:
		var placed := _restore(dlg)
		fit_scale(dlg, not placed)
	else:
		_remember(dlg)

## A focus gain is the one signal a window dragged to another monitor gives (`dpi_changed` is macOS
## only), so the scale is re-fitted there. A no-op when the window's screen scale is unchanged.
static func _on_focus(dlg: Window) -> void:
	if is_instance_valid(dlg) and dlg.visible:
		fit_scale(dlg, true)

## Puts a native window back where it was last left, on the screen it was on if that screen still
## exists, else clamped on-screen (`PopoutGeometry.resolve()`). Does nothing for an embedded window
## or when nothing is stored, so a first open keeps the engine's own placement. Returns whether a
## stored rectangle was applied (the caller must not then resize the window a second time for scale:
## a stored size is already in physical pixels).
static func _restore(dlg: Window) -> bool:
	if not is_instance_valid(dlg) or dlg.is_embedded():
		return false
	var screens := PopoutGeometry.screens_now()
	var r := PopoutGeometry.resolve(DccSettings.popout_geometry(key_for(dlg)),
		screens, DisplayServer.get_primary_screen())
	var placed := bool(r.get("placed", false))
	## PW-3: a window opened FROM another pop-out opens on the opener's screen (scope PW-3 bar). The
	## opener is the pop-out that has the keyboard focus right now (`opener_screen()`); a window opened
	## from the main window or a menu has no opener and keeps the PW-1 rules unchanged.
	var op := opener_screen(dlg)
	var rule := opener_rule(placed, int(r.get("screen", -1)), op, dlg.current_screen)
	var rect := Rect2i()
	if rule == "opener":
		rect = centred_in(screens[op]["usable"], dlg.size)
	elif rule == "stored":
		rect = r["rect"]
	else:
		return false
	dlg.size = rect.size
	dlg.position = rect.position
	return true

## Which rule places a window that is being shown, as a pure function so a table of literals can pin
## each term (PW-3, the "opened from another window" bar). `placed`/`stored_screen` are `resolve()`'s
## answer for the stored geometry, `opener` the screen of the pop-out that opened this window (-1 when
## there is none) and `current` the screen the engine just put it on. Returns:
## - `"opener"`: there IS an opener, and leaving the window where it would otherwise go would put it on
##   another screen (the stored rectangle is elsewhere, or nothing is stored and the engine chose
##   another screen), so it is centred on the opener's screen;
## - `"stored"`: the PW-1 restore applies (a rectangle is stored and it is on the opener's screen, or
##   there is no opener);
## - `"engine"`: nothing stored and nothing to override: the engine's own placement stands, which is
##   exactly today's behaviour and the only branch a one-monitor machine ever takes.
static func opener_rule(placed: bool, stored_screen: int, opener: int, current: int) -> String:
	if opener >= 0:
		if placed and stored_screen == opener:
			return "stored"
		if placed or current != opener:
			return "opener"
		return "engine"
	return "stored" if placed else "engine"

## The screen index of the pop-out that opened `dlg`: a visible NATIVE window of the live list, other
## than `dlg`, that has the keyboard focus (a menu or a map click lives in the main window, so the
## main window has the focus then and there is no opener). -1 when there is none -- never 0, which
## is a real screen.
static func opener_screen(dlg: Window) -> int:
	if dlg == null or not dlg.is_inside_tree():
		return -1
	for w in native_windows(dlg.get_tree()):
		if w != dlg and (w as Window).has_focus():
			return (w as Window).current_screen
	return -1

## `size` centred in `usable`, shrunk to fit it and never above its top edge. Pure. The title-row
## allowance is `PopoutGeometry.TITLE_ROW`, as in `resolve()`'s centred fallback.
static func centred_in(usable: Rect2i, size: Vector2i) -> Rect2i:
	var w := clampi(size.x, 1, maxi(usable.size.x, 1))
	var h := clampi(size.y, 1, maxi(usable.size.y - PopoutGeometry.TITLE_ROW, 1))
	return Rect2i(usable.position.x + (usable.size.x - w) / 2,
		usable.position.y + PopoutGeometry.TITLE_ROW + (usable.size.y - PopoutGeometry.TITLE_ROW - h) / 2, w, h)

## Whether `dlg` is between the hide and the show of a `toggle()`. See `META_MOVING`.
static func is_moving(dlg: Object) -> bool:
	return dlg != null and bool(dlg.get_meta(META_MOVING, false))

## Stores a native window's rectangle and screen when it is hidden (not on every drag step: the
## same cadence as `DccWidgets._desktop_window_size`). An embedded window stores nothing.
static func _remember(dlg: Window) -> void:
	if not is_instance_valid(dlg) or dlg.visible or not dlg.is_inside_tree():
		return
	var geo := PopoutGeometry.capture(dlg)
	if not geo.is_empty():
		DccSettings.set_popout_geometry(key_for(dlg), geo)


# -- PW-2: scale per screen ------------------------------------------------------------------------

## The content-scale factor a native window needs on its own screen, so it looks as large as it
## does in the main window: the main window's factor times the ratio of the two screens' scales.
## **Returns -1.0 when any input is not a usable positive number** (a screen that reports no scale),
## which no real factor can be, so the caller leaves the window alone rather than apply a plausible
## 1.0 (`MISTAKES.md`, never encode "no value" as a value). PW-0 measured why a factor alone is not
## enough: setting 1.5 on a native window scaled its content without re-laying it out, cropping it
## inside the unchanged window; `fit_scale()` resizes the window with the factor.
static func scale_for_screen(main_factor: float, main_screen_scale: float, win_screen_scale: float) -> float:
	if main_factor <= 0.0 or main_screen_scale <= 0.0 or win_screen_scale <= 0.0:
		return -1.0
	return main_factor * win_screen_scale / main_screen_scale

## The size a window of `size` pixels takes when its factor goes from `from` to `to`: the logical
## size stays the same, so the window grows with the factor. Pure, for the table in the probe.
static func scaled_size(size: Vector2i, from: float, to: float) -> Vector2i:
	if from <= 0.0 or to <= 0.0:
		return size
	return Vector2i(roundi(size.x * to / from), roundi(size.y * to / from))

## Applies `scale_for_screen()` to `dlg` and, when `resize` is true, resizes the window with the factor.
## The window's `min_size` is deliberately NOT scaled: under `CANVAS_ITEMS` the engine already
## multiplies it by the factor, and scaling it too measured 1395 x 945 from 880 x 620 at 1.5 (a
## double scale) in the first PW-2 run. `main_screen_scale` and `win_screen_scale` are read from
## `DisplayServer` when they are passed as -1.0 (the probe passes literals to simulate a second
## monitor on a one-monitor machine). With equal scales the factor is the main window's, which is
## 1.0 on the desktop form, so on one monitor this does nothing at all.
static func fit_scale(dlg: Window, resize: bool, main_screen_scale: float = -1.0, win_screen_scale: float = -1.0) -> void:
	if not is_instance_valid(dlg) or dlg.is_embedded() or not dlg.is_inside_tree():
		return
	var ms := main_screen_scale
	if ms < 0.0:
		ms = DisplayServer.screen_get_scale(DisplayServer.window_get_current_screen(DisplayServer.MAIN_WINDOW_ID))
	var ws := win_screen_scale
	if ws < 0.0:
		ws = DisplayServer.screen_get_scale(dlg.current_screen)
	var f := scale_for_screen(dlg.get_tree().root.content_scale_factor, ms, ws)
	if f < 0.0 or is_equal_approx(f, dlg.content_scale_factor):
		return
	var from := dlg.content_scale_factor
	dlg.content_scale_mode = Window.CONTENT_SCALE_MODE_CANVAS_ITEMS
	dlg.content_scale_factor = f
	if resize:
		dlg.size = scaled_size(dlg.size, from, f)


# -- PW-2: busy overlay ----------------------------------------------------------------------------

## Every window of the live list that is visible and NATIVE right now. `BusyOverlay` walks this
## beside `get_embedded_subwindows()` (which never lists a native window, PW-0), muting each one and
## drawing its card mirror into it. A docked (embedded) window is excluded: the overlay already
## reaches it, and listing it twice would mute it twice.
static func native_windows(tree: SceneTree) -> Array:
	var out: Array = []
	if tree == null:
		return out
	for n in tree.get_nodes_in_group(GROUP):
		if n is Window and (n as Window).visible and not (n as Window).is_embedded():
			out.append(n)
	return out


# -- PW-2: keys ------------------------------------------------------------------------------------

## Which app command a key typed in a pop-out stands for: `"undo"`, `"redo"`, `"save"`, or `""` for
## a key that is not one of them. A key event typed in a native window never reaches
## `DccApp._unhandled_key_input` (PW-0: it is a different viewport), and the menu accelerators live
## in the main window, so `PopoutKeyForwarder` carries exactly these. Pure, so a table of literals
## pins every term: Ctrl+Z undo, Ctrl+Shift+Z and Ctrl+Y redo, Ctrl+S save. Ctrl+Shift+S is "Save as"
## (it opens a dialog) and is NOT forwarded; Alt-chords and a key typed while a text field has focus
## (`typing`) are never forwarded, because the field owns its own Ctrl+Z.
static func forward_action(keycode: int, cmd_or_ctrl: bool, shift: bool, alt: bool, typing: bool) -> String:
	if typing or alt or not cmd_or_ctrl:
		return ""
	match keycode:
		KEY_Z:
			return "redo" if shift else "undo"
		KEY_Y:
			return "" if shift else "redo"
		KEY_S:
			return "" if shift else "save"
	return ""


# -- PW-2: the per-window button and the move ------------------------------------------------------

## Adds the window's "Pop out" / "Dock" button to its button row and returns it, or `null` (and adds
## nothing) when this window cannot pop out here: the platform does not support it, or its key is
## not in `LIVE_KEYS`. The button is the only visible difference between a roster with the setting
## off and today's, and it is what makes a manual pop-out possible with the setting off (Ruling DK
## Q2: both ship). Idempotent: a second call returns the existing button.
##
## `header_row` (PW-3, one window): a window that HIDES its dialog button row's Close (the Generation
## rules window draws its own Close in its header and calls `get_ok_button().hide()`) cannot take the
## button there -- with Close hidden the row is only the resize grip, and a button in it made the row taller
## and shrank the content above it by 10 px (`_popout_windows_id_probe.gd`: 11 601 differing pixels outside
## the button). Such a window passes the box that already holds its own Close; the button goes immediately
## before that Close, the box's spacer absorbs its width, and no other pixel moves (no balance control is
## needed: nothing is centred in that box).
static func add_toggle_button(dlg: AcceptDialog, header_row: BoxContainer = null) -> Button:
	if not supported() or not LIVE_KEYS.has(key_for(dlg)):
		return null
	var existing := dlg.find_child(BUTTON_NAME, true, false) as Button
	if existing != null:
		return existing
	if header_row != null:
		var hb := DccWidgets.action(header_row, "Pop out", func() -> void: toggle(dlg))
		header_row.move_child(hb, maxi(header_row.get_child_count() - 2, 0))
		hb.custom_minimum_size.x = BUTTON_MIN_W
		hb.name = BUTTON_NAME
		hb.tooltip_text = "Moves this panel into its own operating-system window, for a second monitor. Press again to dock it back. Your place in it (tab, selection, scroll) is kept."
		dlg.visibility_changed.connect(func() -> void: sync_button(dlg))
		sync_button(dlg)
		return hb
	## Built into the dialog's own button row, which is `[spacer][Close][spacer]`. `AcceptDialog.add_button`
	## was tried first and moved Close (measured by `_popout_roster_id_probe.gd`: it inserts ANOTHER spacer, so
	## the three spacers share the width and Close left the centre). The row is therefore made symmetric
	## by hand: the button goes first and an invisible balance control of the same width goes last, so
	## `[button][spacer][Close][spacer][balance]` keeps Close exactly where it was and the roster with
	## the setting off differs from the old one only by this button's own pixels.
	var row := dlg.get_ok_button().get_parent() as HBoxContainer
	if row == null:
		return null
	var b := DccWidgets.action(row, "Pop out", func() -> void: toggle(dlg))
	row.move_child(b, 0)
	b.custom_minimum_size.x = BUTTON_MIN_W
	var balance := Control.new()
	balance.name = BUTTON_NAME + "Balance"
	balance.mouse_filter = Control.MOUSE_FILTER_IGNORE
	balance.custom_minimum_size.x = BUTTON_MIN_W
	row.add_child(balance)
	## The row really ends `[spacer][Close][spacer][resize grip]` (`DccWidgets._desktop_window_size`), so the
	## balance goes BEFORE the grip: the grip keeps the window's corner, and each spacer still gains the same
	## width from the two sides, which is what keeps Close exactly where it was.
	var last := row.get_child(row.get_child_count() - 2)
	if last is Control and (last as Control).tooltip_text == "Drag to resize":
		row.move_child(balance, row.get_child_count() - 2)
	## The balance must equal the button's LAID-OUT width, not the constant: measured, the button lays out
	## 3 px narrower than `BUTTON_MIN_W` asks (81 against 84) and an equal-constant balance moved Close
	## by 1.5 px. So the balance follows the button's own size.
	b.resized.connect(func() -> void: balance.custom_minimum_size.x = b.size.x)
	b.name = BUTTON_NAME
	b.tooltip_text = "Moves this panel into its own operating-system window, for a second monitor. Press again to dock it back. Your place in it (tab, selection, scroll) is kept."
	dlg.visibility_changed.connect(func() -> void: sync_button(dlg))
	sync_button(dlg)
	return b

## Sets the button's label from the window's real state: "Dock" while the window is visible and
## native, "Pop out" otherwise.
static func sync_button(dlg: Window) -> void:
	var b := dlg.find_child(BUTTON_NAME, true, false) as Button
	if b != null:
		b.text = "Dock" if (dlg.visible and not dlg.is_embedded()) else "Pop out"

## The scroll positions of every `ScrollContainer` under `dlg`, by path, as `[{path, h, v}]`. Taken
## before a move so the same positions can be put back (`restore_scrolls`).
static func snapshot_scrolls(dlg: Window) -> Array:
	var out: Array = []
	for n in dlg.find_children("*", "ScrollContainer", true, false):
		var sc := n as ScrollContainer
		out.append({"path": dlg.get_path_to(sc), "h": sc.scroll_horizontal, "v": sc.scroll_vertical})
	return out

## Puts back what `snapshot_scrolls` took. A container that no longer exists is skipped.
static func restore_scrolls(dlg: Window, snap: Array) -> void:
	for e in snap:
		var sc := dlg.get_node_or_null(e["path"]) as ScrollContainer
		if sc != null:
			sc.scroll_horizontal = int(e["h"])
			sc.scroll_vertical = int(e["v"])

## Moves an open (or closed) window between the main window and its own OS window. The same node
## is hidden, flipped and shown again, never rebuilt, so everything it holds -- the selected faction,
## the tab, a half-typed field's text, the busy overlay's mute -- survives because it is the same
## object; only the scroll positions are re-applied, after the new layout (`TOGGLE_SETTLE_FRAMES`).
## A coroutine: callers that need to know it finished `await` it. Never rebuilds the window and never
## calls the window's `open()` (that would re-fetch and reset the scroll).
static func toggle(dlg: Window) -> void:
	if not is_instance_valid(dlg) or not dlg.is_inside_tree() or not supported() \
			or not LIVE_KEYS.has(key_for(dlg)):
		return
	var go_native: bool = dlg.is_embedded() if dlg.visible else not dlg.force_native
	var was_visible := dlg.visible
	var snap := snapshot_scrolls(dlg)
	var size_before := dlg.size
	## Where the window is on the desktop now, so a first pop-out appears over its docked self.
	var screen_pos := Vector2i(dlg.get_tree().root.position) + dlg.position
	dlg.set_meta(META_CHOICE, go_native)
	## Marked for exactly the span in which this move hides the window, so a handler that treats a hide
	## as "closed" (the assistant ends a running reply on one) can tell a move from a close. Erased
	## before the first yield, on every branch.
	dlg.set_meta(META_MOVING, true)
	if was_visible:
		dlg.hide()
	prepare(dlg)
	if not was_visible:
		dlg.remove_meta(META_MOVING)
		return
	if go_native:
		dlg.popup(Rect2i(screen_pos, size_before))
	else:
		dlg.popup_centered(size_before)
	dlg.remove_meta(META_MOVING)
	sync_button(dlg)
	for _i in TOGGLE_SETTLE_FRAMES:
		await dlg.get_tree().process_frame
		if not is_instance_valid(dlg):
			return
	restore_scrolls(dlg, snap)

## Forgets every window's manual choice, so each follows the global setting again. Called when the
## user flips `Preferences > Open panels in their own window`, so the setting and the buttons agree
## (PW-2 bar g). A window already open is left as it is until it is next opened.
static func forget_choices(tree: SceneTree) -> void:
	if tree == null:
		return
	for n in tree.get_nodes_in_group(GROUP):
		if n.has_meta(META_CHOICE):
			n.remove_meta(META_CHOICE)
