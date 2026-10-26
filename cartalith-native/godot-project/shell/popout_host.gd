extends RefCounted
class_name PopoutHost

## The one place that decides whether a tool window is a native OS window
## (`POPOUT_WINDOWS_SCOPE.md` s.4, milestone PW-1, Ruling DK).
##
## **Zero cost when off is the contract.** Every tool window passes through `prepare()` once, from
## `DccWidgets.phone_window()` (the call every one of them already makes in its `setup`). With the
## setting off, on a phone or a tablet, on a build without `FEATURE_SUBWINDOWS`, or for a window
## that is not in `LIVE_KEYS`, `prepare()` leaves `Window.force_native` at `false`: no native
## window is created and no signal is connected, so the shell is exactly what it was. PW-0 measured
## why that matters: each open native window costs a fixed redraw every frame (+1.2 ms for one,
## +1.7 ms for three at a continuous redraw, `POPOUT_WINDOWS_SCOPE.md` s.8).
##
## What it must never do: set `force_native` on a window that is visible (the engine only honours
## it before the first show), make a pop-out `exclusive` (the whole point is to use both screens at
## once, scope s.2.6), or store the geometry of an EMBEDDED window (`PopoutGeometry.capture()`
## refuses it).

## The window keys (`key_for()`) that actually pop out when the setting is on.
## **Empty in PW-1, on purpose.** A native window is outside `BusyOverlay`'s reach (a click landed
## in one during a generate, PW-0), and its keys never reach `DccApp._unhandled_key_input`; both are
## PW-2's first bars, and until they hold no window may be allowed to leave the main window, even if
## the setting is edited into the file by hand. PW-2 adds `faction_roster_window`; PW-3 the rest, in
## the scope's order: settlement_types_window, place_editor_window, generation_rules_window,
## city_viewer_window, vault_window, llm_assistant_window.
const LIVE_KEYS: PackedStringArray = []

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
## native only when the user's setting is on AND the platform supports it AND the window's key is
## one of `live`. Dropping any one term changes the answer for some row of the probe's table.
static func wants_native(setting_on: bool, platform_ok: bool, key: String, live: PackedStringArray) -> bool:
	return setting_on and platform_ok and live.has(key)

## Decides and applies `force_native` for `dlg`, and (only when native) wires its geometry memory.
## Returns whether the window is now native. Idempotent and safe to call before every open; a
## visible window is left as it is. `live` defaults to `LIVE_KEYS` and exists so the probe can drive
## the native branch with a scratch dialog without any real window being allowed to leave.
static func prepare(dlg: Window, live: PackedStringArray = LIVE_KEYS) -> bool:
	if dlg.visible:
		return dlg.force_native
	var native := wants_native(DccSettings.popout_enabled(), supported(), key_for(dlg), live)
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
	return true

## One handler for both directions. **Restore runs on SHOWING, not on `about_to_popup`**: measured in
## PW-1, `popup_centered()` sets its own centred rectangle AFTER `about_to_popup`, so a position set
## there was overwritten (the restore bar failed red with (580, 320) centred against the stored
## (200, 150)); `visibility_changed` fires after it.
static func _on_visibility(dlg: Window) -> void:
	if not is_instance_valid(dlg):
		return
	if dlg.visible:
		_restore(dlg)
	else:
		_remember(dlg)

## Puts a native window back where it was last left, on the screen it was on if that screen still
## exists, else clamped on-screen (`PopoutGeometry.resolve()`). Does nothing for an embedded window
## or when nothing is stored, so a first open keeps the engine's own placement.
static func _restore(dlg: Window) -> void:
	if not is_instance_valid(dlg) or dlg.is_embedded():
		return
	var r := PopoutGeometry.resolve(DccSettings.popout_geometry(key_for(dlg)),
		PopoutGeometry.screens_now(), DisplayServer.get_primary_screen())
	if not bool(r.get("placed", false)):
		return
	var rect: Rect2i = r["rect"]
	dlg.size = rect.size
	dlg.position = rect.position

## Stores a native window's rectangle and screen when it is hidden (not on every drag step: the
## same cadence as `DccWidgets._desktop_window_size`). An embedded window stores nothing.
static func _remember(dlg: Window) -> void:
	if not is_instance_valid(dlg) or dlg.visible or not dlg.is_inside_tree():
		return
	var geo := PopoutGeometry.capture(dlg)
	if not geo.is_empty():
		DccSettings.set_popout_geometry(key_for(dlg), geo)
