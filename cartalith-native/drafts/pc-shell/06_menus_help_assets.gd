## DRAFT, not loaded by Godot. NOT parse-checked (no Godot binary in the session that wrote it).
##
## PC_SHELL_PROPOSAL.md §3.8, finding M2 ("one label, two behaviours", and one behaviour under two
## labels), milestone PC-9:
##   - Help ▸ "Keyboard shortcuts…" becomes "Keyboard reference…": it opens `ShortcutsDialog` read-only,
##     while Preferences ▸ Keyboard shortcuts… (`ID_PREF_SHORTCUTS`) opens the same dialog as the editor
##     and keeps its name.
##   - Assets ▸ "Import image…" is removed: `_on_assets()` sent it to the same `open_asset_library()` as
##     ⧉ Asset library. The import itself is the library's own "Import image…" chip (rail foot, into
##     the focused slot: `asset_library_window.gd`, `_import_btn`), which stays.
##
## Target: `shell/menus.gd`. Depends on nothing in this folder. **Lands in the same change as
## `07_command_index.gd`** (`MISTAKES.md`, "Move a command off the menu bar": the title "Import image"
## stops being a menu row, so search keeps it through an `EXTRAS` alias). `08_shortcuts_dialog.gd`
## retitles the dialog to match the row that opened it.
## Applies to: every form factor (the tablet's ☰ builds `_build_help_shortcuts_row()` and
## `_assets(p, false)` too; the phone re-presents both menus).
## Forms: one. Neither change needs a milestone.
##
## Inventory of the removed row (`MISTAKES.md`, "Delete a surface"), each item with its home or a
## stated drop:
##   - the command (open the Asset library): ⧉ Asset library, id 30, Shift+A, on the same menu;
##   - the import: the library's own "Import image…" chip, unchanged;
##   - search: `command_index.gd::EXTRAS` row "Import image" (07);
##   - a quick-bar pin on it (`menu:32`): reads "not in this build" (`command_index.gd::find_pin()`'s
##     contract). A stated drop; a pin on ⧉ Asset library does the same thing;
##   - id 32: kept defined, never reissued;
##   - probes: `_vfy_r2_menus_probe.gd`'s SURVIVOR_CLAIM ["Import image into slot", "Import image"]
##     still finds the title, through the EXTRAS row; `_apcmdcheck_probe.gd` and
##     `_lanebpickers_probe.gd` do not read this row (a census of other terms, and the library's own
##     picker). `grep -rn "ID_IMPORT_IMAGE" .` finds `menus.gd` only.
##
## What to do. Each EDIT quotes its old text exactly as it stands at `a38e0ac`.
##   REPLACE  `_build_help_shortcuts_row()` with its doc, with the version below.
##   ADD      `HELP_KEYBOARD_REFERENCE_TIP`, beside it.
##   EDIT H1  the line
##                const ID_HELP_SHORTCUTS := 72  ## Help ▸ Keyboard shortcuts… (read-only list).
##            becomes
##                const ID_HELP_SHORTCUTS := 72  ## Help ▸ Keyboard reference… (read-only; "Keyboard shortcuts…" until PC-9).
##   EDIT A1  `_assets()`: replace these five lines
##                ## `AssetDB::add_item`/`raster::decode_png` are real and bound now
##                ## (`as_import_item`) -- the window's own slot grid is where a target
##                ## slot gets focused, so this opens straight to it rather than
##                ## duplicating slot-selection at the menu level.
##                _live(p, "Import image…", ID_IMPORT_IMAGE)
##            with these comment lines (no code)
##                ## PC-9 (`PC_SHELL_PROPOSAL.md` §3.8, finding M2): "Import image…" left
##                ## this menu. It opened the Asset library under a second name (the same
##                ## `open_asset_library()` as ⧉ Asset library); the import itself is the
##                ## library's own "Import image…" chip, into the focused slot.
##                ## `command_index.gd::EXTRAS` keeps the words findable.
##   EDIT A2  `_on_assets()`: delete the line
##                ID_IMPORT_IMAGE: _host.open_asset_library()
##            (no row carries the id now; an arm would only answer a probe emitting it by hand).
##   EDIT A3  the line
##                const ID_IMPORT_IMAGE := 32  ## Assets ▸ Import image….
##            becomes
##                const ID_IMPORT_IMAGE := 32  ## Retired by PC-9: the row opened the Asset library under a second name. Kept so 32 is never reissued.
##   PROSE    `_assets()`'s comment "The canvas runs these four as **one unbroken block** -- there is
##            no rule between the slicer and Import image in `DCC shell 1920`": the block is three rows
##            now (Asset library, slicer, Import asset pack). Help: `_help()`'s doc ("omits Keyboard
##            shortcuts and About"), `_on_preferences()`'s comment ("`Help ▸ Keyboard shortcuts…`
##            reaches"), the Session-mode comment in `_window()` ("listed by Help ▸ Keyboard
##            shortcuts"), `app.gd::open_shortcuts()`'s doc and the ⇧J comment in
##            `app.gd::_unhandled_key_input` all say "Keyboard reference" for Help's row.
##            `shortcuts_dialog.gd`'s header is 08's.
##
## Verify: the parse check on `res://shell/menus.gd`; `check_symbols.sh`. `_tabletparity_probe.gd`
## (Help's title changes on both compositions, so the multiset still matches). By hand: Assets has no
## Import image row; typing "import image" in search finds the EXTRAS row naming the Asset library;
## Help ▸ Keyboard reference… opens a read-only list titled Keyboard reference, and Preferences ▸
## Keyboard shortcuts… opens the editor titled Keyboard shortcuts.

## Help ▸ Keyboard reference…'s tooltip: what it lists and where the editor is, so the two rows that
## open one dialog say which way each opens it. Enabled rows' tooltips do not enter the command index
## (its blurb for a live row is the menu's name), so this changes no search result.
const HELP_KEYBOARD_REFERENCE_TIP := "Every keyboard shortcut, read from the menus each time it opens, and the keys no menu carries. Read-only: Preferences ▸ Keyboard shortcuts… changes them."

## The one-line "Keyboard reference…" row (Help's read-only `ID_HELP_SHORTCUTS`),
## its own function since 2026-09-12 for the same reason as
## `_build_asset_library_row()`: the tablet's ☰ overflow and PC/phone's Help
## menu build the identical row from one source.
##
## **"Keyboard shortcuts…" until PC-9** (`PC_SHELL_PROPOSAL.md` §3.8, finding
## M2). Preferences ▸ Keyboard shortcuts… is the editor (`ShortcutsDialog.
## open_editable()`); this row opens the same dialog read-only
## (`app.gd::open_shortcuts()` -> `ShortcutsDialog.open()`), so it is named for
## what it opens. Same id: a quick-bar pin (`menu:72`) and the assistant's
## window list (`llm_view.gd::WINDOWS`, entry "shortcuts") still name it.
func _build_help_shortcuts_row(p: PopupMenu) -> void:
	_live(p, "Keyboard reference…", ID_HELP_SHORTCUTS)
	p.set_item_tooltip(p.item_count - 1, HELP_KEYBOARD_REFERENCE_TIP)
