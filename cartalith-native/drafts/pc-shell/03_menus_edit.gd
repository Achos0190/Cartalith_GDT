## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.1, §3.8 and §3.9, the Edit rows (milestone PC-9):
##   - Deselect clears the one selection once PC-1 lands (finding S1). The row and its key are
##     unchanged; PC-1 changes `app.gd::clear_selection()`, and this draft changes only the tooltip.
##   - Previous selection (Alt+←) and Next selection (Alt+→), new, after Deselect (§3.1 "History").
##   - Context card (Shift+F10) and Cartwheel at the pointer (Q): §3.9 moves both keys out of
##     `app.gd::_unhandled_key_input` "into the rebindable table, and into the reference" (finding M8).
##     The rebindable table IS the menu accelerators (`_bind_accelerator`, which `shortcuts_dialog.gd`
##     reads and rebinds), so each key needs a row. Neither the proposal nor the mockup says where:
##     these sit after Find on map…, with the other selection verbs (README, owner question K1).
##
## Target: `shell/menus.gd`. Depends on `01_menus_ids_helpers.gd`. **Lands in the same change as
## `09_app_keys.gd`**: the two map rows call `open_context_card()` and `open_cartwheel_at_pointer()`,
## which 09 adds, and 09 removes the old Q and Shift+F10 presses from `app.gd`. Either one alone breaks
## Q (09 alone: nothing opens the cartwheel; this alone: the row calls a method that does not exist).
## `08_shortcuts_dialog.gd` lists the Menu key, which stays in `app.gd`, in the same change.
##
## Applies to: the PC (Edit), the tablet (☰ ▸ Edit, built by `_edit(p, false)`) and the phone (MENU ▸
## Edit re-presents the popup), except the two map rows, which are not built on the phone.
##
## Forms:
##   before PC-1  Previous / Next selection are `_todo` rows with `SELECTION_HISTORY_TODO`; Deselect
##                keeps its shipped tooltip (`DESELECT_TIP`).
##   from PC-1    (the host has `select_back` / `select_forward`) both rows are live with Alt+← / Alt+→,
##                and Deselect's tooltip is `DESELECT_ONE_TIP`.
##   always       Context card and Cartwheel at the pointer are live (they need no milestone).
##
## What to do. Each EDIT quotes its old text exactly as it stands at `a38e0ac`.
##   EDIT E1  `_build_edit_actions()`: replace the three lines
##                _live(p, "Deselect", ID_DESELECT, KEY_MASK_CTRL | KEY_D)
##                p.set_item_tooltip(p.item_count - 1,
##                    "Clears the settlement, label or icon selection in the active domain. Not the same as Escape, which puts the tool down and deliberately leaves the selection alone.")
##            with
##                _live(p, "Deselect", ID_DESELECT, KEY_MASK_CTRL | KEY_D)
##                p.set_item_tooltip(p.item_count - 1, _deselect_tip())
##                _build_selection_rows(p)
##            (one tab of indentation; the old string's line has two).
##   EDIT E2  `_build_edit_actions()`: directly after the line
##                _live(p, "Find on map…", ID_FIND_ON_MAP, KEY_MASK_CTRL | KEY_F)
##            insert the line
##                _build_map_surface_rows(p)
##            Every row index the clipboard refresher captures was taken above Deselect, so nothing
##            inserted below it moves one.
##   EDIT E3  `_edit()`: after its last line, the `p.id_pressed.connect(_on_edit)` that follows
##                    _build_reset_generation(p)
##            insert the line
##                p.id_pressed.connect(_on_edit_selection)
##            **Only in `_edit()`.** The same `p.id_pressed.connect(_on_edit)` line also ends
##            `_overflow()` and `_world()` (three occurrences at a38e0ac), and neither of those popups
##            carries these rows. `_edit()` runs once per Edit popup (PC, phone, the tablet's ☰ ▸
##            Edit), so each gets one connection.
##   ADD      everything below, beside `_build_edit_actions()`.
##   PROSE    `_build_edit_actions()`'s Deselect comment ends "it is domain-scoped by the workspace
##            guard rather than by `_select_all()`'s own `match`": still true until PC-1, which must
##            rewrite it when `clear_selection()` clears the one selection.
##
## Verify: the parse check on `res://shell/menus.gd` and `res://shell/app.gd`; `check_symbols.sh`. By
## hand, on the PC: Edit shows Previous / Next selection disabled, each with its reason; Shift+F10 over
## the map opens the card; Q held over the map opens the cartwheel and releasing on a slot picks it;
## Q tapped leaves it open; typing Q or Shift+F10 in a text field (a settlement name) opens neither;
## Preferences ▸ Keyboard shortcuts… lists both rows and rebinding Q to another key moves the press
## AND the release. On the tablet with a keyboard: the same, from ☰ ▸ Edit. On the phone: MENU ▸ Edit
## has neither map row.

## Deselect's tooltip until PC-1: the shipped string, moved here unchanged.
const DESELECT_TIP := "Clears the settlement, label or icon selection in the active domain. Not the same as Escape, which puts the tool down and deliberately leaves the selection alone."

## Deselect's tooltip from PC-1, when `clear_selection()` clears the one shared selection
## (`PC_SHELL_PROPOSAL.md` §3.1, finding S1). **Its last sentence is true until §3.9's Escape change**
## ("With nothing open, it deselects"), which is not drafted here: that change must rewrite it.
const DESELECT_ONE_TIP := "Clears the selection: everything that shows it empties together, and Previous selection (Alt+←) brings it back. Not the same as Escape, which puts the tool down and deliberately leaves the selection alone."

## Why Previous / Next selection are disabled before PC-1. Checked 2026-11-01 at `a38e0ac`: the
## shell holds three separate selections (`app.gd::clear_selection()`'s own header: "none of them
## knew about the others"), and nothing in `shell/` records past ones (`grep -rn -i "selection
## history\|select_back\|previous selection" shell/` finds nothing).
const SELECTION_HISTORY_TODO := "Not built yet: each workspace keeps its own selection and nothing records the ones before, so there is nothing to step back or forward to. It arrives with the one shared selection (PC_SHELL_PROPOSAL.md §3.1, milestone PC-1)."

## Previous selection's tooltip once it is live.
const SELECT_BACK_TIP := "Selects what was selected before, like a browser's Back. Everything that shows the selection follows."
## Next selection's tooltip once it is live.
const SELECT_FORWARD_TIP := "Selects again what Previous selection stepped back from, like a browser's Forward."
## Status line when Previous selection has nowhere to go. A statement about the history, never an
## error: the row is enabled whenever PC-1 exists, and an empty history is an ordinary state.
const SELECT_BACK_NONE := "No earlier selection to go back to."
## Status line when Next selection has nowhere to go.
const SELECT_FORWARD_NONE := "No later selection to go forward to."

## Edit ▸ Context card's tooltip. What `app.gd::_open_context_card_from_keyboard()` does today, read at
## the symbol: `request_at(last_mouse_pos(), "keyboard")`, and the map's centre when that request is
## empty. **When §3.1's S3 lands (the card opens at the selection) this must change with it.**
const CONTEXT_CARD_TIP := "Opens the map's context card at the pointer's last position over the map, or at the map's centre when the pointer is off it. The Menu key does the same."

## Edit ▸ Cartwheel at the pointer's tooltip. The two gestures are `radial_ring.gd`'s: a key-up with no
## slot under the pointer leaves the ring open ("sticky") for a click, a key-up over a slot picks it
## (`release()`). Opened from the menu, a click on a slot picks it (`context_broker.gd::ring_click()`).
const CARTWHEEL_TIP := "Opens the cartwheel where the pointer last was over the map. Tap the key to leave it open and click a slot; hold the key, aim at a slot and let go to pick it."

## Deselect's tooltip for the form this build can honour: `DESELECT_ONE_TIP` once the host has the
## selection history (PC-1's `select_back`), `DESELECT_TIP` before. The row's behaviour is
## `app.gd::clear_selection()`'s in both forms; only the claim about it changes.
func _deselect_tip() -> String:
	return DESELECT_ONE_TIP if _host_has("select_back") else DESELECT_TIP

## Edit ▸ Previous selection (Alt+←) and Next selection (Alt+→), after Deselect, in the mockup's
## order (`design/mobile-shell-2026-10-31/Shell.dc.html`, the desktop Edit menu).
##
## Live only when the host has the method each row calls (PC-1's selection history); a `_todo` row
## with `SELECTION_HISTORY_TODO` before that, so the menu never offers a step it cannot take.
## Alt+← and Alt+→ were free: no row in this file binds an Alt chord, and the only Left/Right
## handlers in the project belong to a surface that is open at the time (`context_card.gd::_key()`
## while the card is open; `quick_bar_window.gd` uses Alt+Up/Down). `check_symbols.sh`, keys section.
##
## **Not guarded against a focused text field, on purpose, and to be decided at PC-1.** A menu
## accelerator runs after the focused control has had the key. On macOS a `LineEdit` takes Alt+← as
## word-left first; on Windows and Linux it does not, so Alt+← while typing changes the selection, as it
## does in a browser. `_typed_into_field()` is the guard if PC-1 decides otherwise.
func _build_selection_rows(p: PopupMenu) -> void:
	if _host_has("select_back"):
		_live(p, "Previous selection", ID_EDIT_SELECT_BACK, KEY_MASK_ALT | KEY_LEFT)
		p.set_item_tooltip(p.item_count - 1, SELECT_BACK_TIP)
	else:
		_todo(p, "Previous selection", SELECTION_HISTORY_TODO)
	if _host_has("select_forward"):
		_live(p, "Next selection", ID_EDIT_SELECT_FORWARD, KEY_MASK_ALT | KEY_RIGHT)
		p.set_item_tooltip(p.item_count - 1, SELECT_FORWARD_TIP)
	else:
		_todo(p, "Next selection", SELECTION_HISTORY_TODO)

## Edit ▸ Context card (Shift+F10) and Edit ▸ Cartwheel at the pointer (Q), after Find on map….
##
## **Why rows at all:** §3.9 makes the two keys rebindable and listed (finding M8). Both used to be
## matched in `app.gd::_unhandled_key_input`, which `shortcuts_dialog.gd` cannot read and
## `DccSettings`' shortcut store does not cover. A row's accelerator goes through `_bind_accelerator`,
## so it is rebindable and the reference lists it with no table written anywhere.
##
## **What stays in `app.gd`** (`09_app_keys.gd`): the cartwheel's key-UP, which follows this row's
## binding through `cartwheel_keycode()`, and the Menu key, the card's fixed second key (one `PopupMenu`
## item holds one accelerator; `shortcuts_dialog.gd::UNLISTED` lists it). Each handler keeps the old
## text-field guard (`_typed_into_field()`), so typing Q or Shift+F10 in a field opens nothing.
##
## **Not built on the phone.** The phone's menu bar lives in a hidden model node
## (`dcc_shell.gd`, `PhoneMenuModel`, `visible = false`), so no accelerator fires there, and both
## surfaces are pointer surfaces: the cartwheel's touch scale is the tablet's (`radial_ring.gd::
## _apply_touch_scale()`), and the phone opens the card by its own gesture (`MAP_CONTEXT_SCOPE.md`,
## fork F1). A phone with a hardware keyboard loses Q and Shift+F10, which it has today through
## `_unhandled_key_input`, and keeps the Menu key. That is a stated drop (README).
##
## Bare Q as a menu accelerator: a focused `LineEdit` or `TextEdit` consumes a printable key before
## `_shortcut_input` runs, and no tool button uses Q (the tool letters are B F I L M R S T V W).
## Behaviour change: Ctrl+Q and Shift+Q no longer open the cartwheel, which the old
## `event.keycode == KEY_Q` test let them do; a menu accelerator matches its modifiers exactly.
func _build_map_surface_rows(p: PopupMenu) -> void:
	if DccTheme.is_phone():
		return
	_live(p, "Context card", ID_EDIT_CONTEXT_CARD, KEY_MASK_SHIFT | KEY_F10)
	p.set_item_tooltip(p.item_count - 1, CONTEXT_CARD_TIP)
	_live(p, "Cartwheel at the pointer", ID_EDIT_CARTWHEEL, KEY_Q)
	p.set_item_tooltip(p.item_count - 1, CARTWHEEL_TIP)

## The dispatcher for the four rows above, connected beside `_on_edit` by `_edit()`. Ignores every id
## it does not own, like every dispatcher sharing a popup in this file.
##
## Previous / Next: their ids exist only on live rows, so the host has the method whenever one
## arrives here; a `false` (nothing to step to) says so in the status line rather than doing nothing.
## The map rows skip a press that came from the key while a text field has focus (the guard
## `app.gd::_unhandled_key_input` used to apply), and run on a click whatever has focus.
func _on_edit_selection(id: int) -> void:
	match id:
		ID_EDIT_SELECT_BACK:
			if not bool(_host.select_back()):
				_host.set_status("hint", SELECT_BACK_NONE, "text_ghost")
		ID_EDIT_SELECT_FORWARD:
			if not bool(_host.select_forward()):
				_host.set_status("hint", SELECT_FORWARD_NONE, "text_ghost")
		ID_EDIT_CONTEXT_CARD:
			if not _typed_into_field(id):
				_host.open_context_card()
		ID_EDIT_CARTWHEEL:
			if not _typed_into_field(id):
				_host.open_cartwheel_at_pointer()
