## DRAFT, not loaded by Godot. MS-4 (MENU), the menu half of Ruling DX Q13 ("the phone's MENU follows the
## mockup"). Draft 05 makes MENU open the eight menus row for row instead of the phone screens; this draft
## gives the two phone-only capabilities that lost a door a row in the menus, as the README's inventory
## ("Where each phone-only row goes") states:
##   - Help ▸ Gesture reference… on the phone: the mockup's own row (`menuTree()`'s Help, marked `mobile`). It
##     opens `PhoneMenu`'s `gestures` screen, whose doors were the Preferences and Help screens MENU no longer
##     opens.
##   - The landmark settings: Assets ▸ Landmark types ▸ any family's rows open CIVIL ▸ Landmarks, and on the
##     phone that dock is the left sheet, closed. Opening it there makes these rows the phone's door to the
##     landmark caps and spacing that the Civilization screen's "Landmark generation" row used to be.
## Target: `shell/menus.gd`, plus one function for `shell/dcc_shell.gd` (quoted below, because a draft holds one
## target's code). Depends on draft 05 (`PhoneMenu.open_screen()`). Applies to: the phone only; every edit
## below is guarded by `DccTheme.is_phone()`, so the desktop and the tablet build what they built.
##
## What to do. Each EDIT quotes its old text exactly as it stands at `7d2acd1`.
##   ADD      `ID_HELP_GESTURES`, `HELP_GESTURES_TIP` and `_build_gesture_reference_row()` (below), beside the
##            Help ids and `_build_help_shortcuts_row()`. 919 was free in `shell/` and in both draft folders
##            (`grep -rn "\b919\b"`), and it follows the PC drafts' 910-918.
##   EDIT G1  `_help()`: replace the three lines
##                if full:
##                    _build_help_shortcuts_row(p)
##                _live(p, "Credits & academic principles", ID_HELP_CREDITS)
##            with
##                if full:
##                    _build_help_shortcuts_row(p)
##                _build_gesture_reference_row(p)
##                _live(p, "Credits & academic principles", ID_HELP_CREDITS)
##            (one tab before `if` and the two calls, two inside). The mockup's order: Documentation, Keyboard
##            reference…, Gesture reference…, Credits….
##   EDIT G2  `_on_help()`: directly after the line
##                ID_HELP_ABOUT: _host.open_about()
##            insert the two lines (two tabs of indentation)
##                ## Deferred: `PhoneMenu._activate()` closes MENU once this returns; the screen opens after it.
##                ID_HELP_GESTURES: _shell.call_deferred("phone_open_menu_screen", "gestures")
##   EDIT G3  `_open_landmark_dock()`: directly after the line
##                _shell.select_domain_category("civilization", "Landmarks")
##            insert the five lines (one tab, two under the `if`)
##                ## Ruling DX Q13: on the phone the CIVIL dock is the left sheet, closed until something opens
##                ## it, so switching the category alone left the row looking dead (the reason
##                ## `phone_menu.gd::_go_civilization()` gives). Deferred past `PhoneMenu._activate()`'s close.
##                if DccTheme.is_phone():
##                    _shell.call_deferred("_set_sheet_open", "left", true)
##   ADD      to `shell/dcc_shell.gd`, beside `phone_present_popup()`, this function (written here as comments
##            so that this file's code is `menus.gd`'s alone):
##                ## Opens the phone's MENU with screen `id` pushed over its root (`PhoneMenu.open_screen()`), after
##                ## closing every other phone overlay, as every phone overlay opens. For a program-menu row whose
##                ## content is a phone screen: Help ▸ Gesture reference… (`menus.gd`). Does nothing off the phone,
##                ## where no such row is built.
##                func phone_open_menu_screen(id: String) -> void:
##                    if not _phone or _phone_menu == null:
##                        return
##                    _close_all_phone_overlays()
##                    _phone_menu.open_screen(id)
##
## Verify: the parse check on `res://shell/menus.gd` and `res://shell/dcc_shell.gd`; `check_symbols.sh`. On the
## phone: MENU ▸ Help lists Gesture reference… third and opens the gesture reference, and back returns to MENU;
## Assets ▸ Landmark types ▸ Physical ▸ Peak opens the left sheet on CIVIL ▸ Landmarks at Physical. On the desktop
## and the tablet: Help has no Gesture reference… row, and a landmark row switches the dock as before.
##
## NOT parse-checked (no Godot binary in the session that wrote it).

const ID_HELP_GESTURES := 919  ## Help ▸ Gesture reference… (phone only; Ruling DX Q13, mobile-shell draft 09).

## Help ▸ Gesture reference…'s tooltip, which the phone draws as the row's second line. Says what the screen is,
## in the words its own header uses (`SCREEN_TITLES`' "the touch vocabulary this build has").
const HELP_GESTURES_TIP := "The touch vocabulary this build has: every gesture the map and the sheets answer to, each checked against the code that answers it."

## Help ▸ Gesture reference…, on the phone only: opens `PhoneMenu`'s `gestures` screen through
## `DccShell.phone_open_menu_screen()`. It is the mockup's own row (`Shell.dc.html`, Help, `mobile: true`), and
## since Ruling DX Q13 it is that screen's only door: MENU no longer opens the Preferences and Help screens that
## carried it. Not built on the tablet, whose row has no surface to open (the gesture screen is `PhoneMenu`'s, and
## only the phone builds one); that row is the parity check's N2, not drafted. Must never be built where
## `phone_open_menu_screen()` does nothing, or the row would be a dead control.
func _build_gesture_reference_row(p: PopupMenu) -> void:
	if not DccTheme.is_phone():
		return
	_live(p, "Gesture reference…", ID_HELP_GESTURES)
	p.set_item_tooltip(p.item_count - 1, HELP_GESTURES_TIP)
