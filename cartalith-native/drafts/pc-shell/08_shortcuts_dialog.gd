## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.8 and §3.9 (milestone PC-9), the reference's half:
##   - **UNLISTED gains the Menu key.** §3.9 moves Q and Shift+F10 out of `app.gd::_unhandled_key_input`
##     into menu rows (Edit ▸ Cartwheel at the pointer, Edit ▸ Context card; `03_menus_edit.gd`), so
##     the walk of the live menus lists them and they are rebindable. The Menu key cannot follow
##     Shift+F10 onto the row, because a `PopupMenu` item holds one accelerator (the same reason Ctrl+Y
##     is listed here beside Ctrl+Shift+Z). It stays in `app.gd` and gets a row here, which
##     `MISTAKES.md` ("Move a command off the menu bar") requires of every accelerator that is not on a
##     menu. Neither Q nor Shift+F10 was in this table before: that is finding M8.
##   - **The window is titled for the row that opened it**: "Keyboard reference" from Help (read-only)
##     and "Keyboard shortcuts" from Preferences (the editor), the names the two rows carry since
##     `06_menus_help_assets.gd` (finding M2).
##
## Target: `shell/shortcuts_dialog.gd` (`ShortcutsDialog`). Depends on nothing. **Lands in the same
## change as `03_menus_edit.gd` and `09_app_keys.gd`** (the Menu key row describes what 09 leaves in
## `app.gd`).
## Applies to: every form factor (the phone's sheet head is retitled too).
## Forms: one.
##
## What to do. Each EDIT quotes its old text exactly as it stands at `a38e0ac`.
##   EDIT S1  `UNLISTED`: after the row
##                ["Map view", "Delete / Backspace", "Remove the selected item", "the main window"],
##            insert the row in the block marked below. It sits with the other map keys; the table's
##            order is how the reference lists them.
##   EDIT S2  `setup()`: replace
##                DccWidgets.phone_head(body, "Keyboard shortcuts", "read from the live menus")
##            with
##                _phone_title = DccWidgets.phone_head(body, TITLE_EDITOR, "read from the live menus")
##            (`phone_head()` returns the title `Label` "so a window whose title tracks its subject can
##            keep writing to it"; this one now does).
##   EDIT S3  `open()`: directly after its line `_editable = false`, insert `_set_mode_title(false)`.
##   EDIT S4  `open_editable()`: directly after its line `_editable = true`, insert
##            `_set_mode_title(true)`. (Each of those two lines occurs once in the file.)
##   ADD      `TITLE_REFERENCE`, `TITLE_EDITOR`, `_phone_title` and `_set_mode_title()` below, after
##            `_capturing_id`.
##   PROSE    The header's first line "`Help ▸ Keyboard shortcuts`, which was a `_todo`…" names Help's row
##            by its old title: "`Help ▸ Keyboard reference…` (titled Keyboard shortcuts… until PC-9),
##            which was…". `open()`'s doc "`Help ▸ Keyboard shortcuts…` -- read-only" likewise. The
##            header's "Those rows (six at time of writing; the first four are the ones this paragraph
##            names)" becomes seven. The header's "Eighteen actions route through
##            `DccMenus._bind_accelerator`" was stale before this change (23 bindings at a38e0ac, by
##            `check_symbols.sh`'s keys section), and PC-9 adds eleven to the PC's bar once every form
##            is live (Ctrl+R, Ctrl+Shift+F, Ctrl+Shift+V, Alt+←, Alt+→, Shift+F10, Q, Ctrl+1, Ctrl+2,
##            Ctrl+3, Ctrl+J): write it without a number ("Every accelerator in `menus.gd` routes
##            through …"), or measure it in the same edit.
##            Line 30's "Help ▸ Keyboard shortcuts should open a read-only reference sheet" is a quotation
##            of `GUI_GAP_REGISTER.md` §7.9 and stays as quoted.
##
## Verify: the parse check on `res://shell/shortcuts_dialog.gd` and `res://shell/app.gd`.
## `_shortcuts_probe.gd` opens the dialog both ways; it checks the "Not on a menu" group exists and
## that the row count is stable across a reopen, and pins no count of UNLISTED rows, so it needs no
## edit. By hand: Help ▸ Keyboard reference… opens a window titled Keyboard reference, listing Edit ▸
## Context card with Shift+F10, Edit ▸ Cartwheel at the pointer with Q, and "Menu" under NOT ON A
## MENU; Preferences ▸ Keyboard shortcuts… opens it titled Keyboard shortcuts, with a rebind chip on
## both new rows and none on the Menu key.

## ── EDIT S1: the row to insert into `UNLISTED`, after "Delete / Backspace" ─────────────────────────
	["Map view", "Menu", "Open the context card: the same as Edit ▸ Context card, whose key can be rebound", "the main window"],
## ── end of EDIT S1 ──────────────────────────────────────────────────────────────────────────────────

## The window's title when Help ▸ Keyboard reference… opened it (`open()`, read-only). The row's own
## name without its ellipsis, so the window says what the user picked (finding M2).
const TITLE_REFERENCE := "Keyboard reference"
## The window's title when Preferences ▸ Keyboard shortcuts… opened it (`open_editable()`), and the
## title `setup()` builds the phone head with before either has run.
const TITLE_EDITOR := "Keyboard shortcuts"

## The phone sheet's title `Label`, as `DccWidgets.phone_head()` returned it, so the title can follow
## the mode; null on desktop and tablet, which use the window title bar. Written once by `setup()`.
var _phone_title: Label

## Titles the window for the mode it is opening in: `TITLE_EDITOR` when `editable`, `TITLE_REFERENCE`
## otherwise, on the title bar and, on a phone, on the sheet's own head (upper-cased, as
## `phone_head()` draws it). Called by `open()` and `open_editable()` before they rebuild, so the
## title and the rows never disagree about the mode. Never reads or changes `_editable`.
func _set_mode_title(editable: bool) -> void:
	title = TITLE_EDITOR if editable else TITLE_REFERENCE
	if _phone_title != null:
		_phone_title.text = title.to_upper()
