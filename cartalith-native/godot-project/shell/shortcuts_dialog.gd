extends AcceptDialog
class_name ShortcutsDialog

## `Help ▸ Keyboard shortcuts`, which was a `_todo` reading *"No shortcut table
## yet."* since the menu was built.
##
## **The table is not written down anywhere; it is read back off the live
## menus.** `Menus` registers every accelerator with
## `PopupMenu.set_item_accelerator`, so walking the real `MenuBar` at the moment
## the dialog opens produces a list that cannot disagree with what the
## application actually does. A hand-maintained table is a second copy of the
## truth, and this repository has been bitten by that shape repeatedly -- most
## recently `PARITY_AUDIT.md` §23 F14, where the shell kept its own copy of
## three engine toggles and the two were free to drift after a load.
##
## What that walk cannot see is a shortcut with no menu row behind it: the
## Layers popover's `1`-`8`, the viewport's space-to-pan, `Escape`/`Delete`.
## Those are declared in [`UNLISTED`] with the file that owns each, and the
## dialog says plainly that they came from a different place -- rather than
## silently mixing two provenances in one list.
##
## Prompted by studying Nortantis 3.18, which carries the same row
## (`menu.help.keyboardShortcuts`). The lesson taken was not "add a dialog" but
## "a shortcut table should be generated, because a written one rots."
##
## ## 2026-09-03: this dialog also IS the editor -- PR-16/HE-02
##
## Owner-ruled build: *"A binding table in DccSettings, applied over the menu
## accelerators at build time. Per-context, not flat."* `GUI_GAP_REGISTER.md`
## §7.9 item 5 calls two separate dialogs a bug ("Help ▸ Keyboard shortcuts
## should open a read-only reference sheet; Preferences ▸ Keyboard shortcuts…
## opens the editor... two editors is a bug") -- so this stays **one class**,
## opened two ways:
##
## - `open()` -- Help's row, unchanged: read-only, exactly as it always was.
## - `open_editable()` -- Preferences' new row: the SAME live-menu walk, with
##   a rebind chip and a per-row Reset on every row `DccMenus` will actually
##   apply an override to (`DccMenus.is_rebindable_shortcut(id)`), and a
##   "Restore all to defaults" for the whole table.
##
## **What editable mode does NOT cover, and why**: `[UNLISTED]` below never
## grows a rebind control, in either mode. Those four rows are real
## accelerators, but none of them are `menus.gd` accelerators -- the layer
## digits are an `InputMap` action (`layers_popover.gd`), Space is a held
## modifier read straight off `Input.is_key_pressed` (`viewport_host.gd`), and
## Escape/Delete are keycodes `app.gd`'s `_unhandled_key_input` matches
## directly, dispatched by whichever tool is armed. `DccSettings.
## SHORTCUT_CONTEXT_MENU`'s own header names this the same way: rebinding
## those needs a pass that owns those files, which this one does not. A chip
## that looked clickable there would be exactly the failure mode this
## project keeps shipping -- **a control that silently does nothing** -- so
## editable mode marks those rows' tooltip instead of pretending they bind.
##
## **Conflicts are shown, never blocked** (`GUI_GAP_REGISTER.md` §7.9 item 3,
## Blender's rule over Photoshop's): rebinding a key already used by other
## rows in the SAME context (the only context this file can see -- see
## `_conflicts_within_menu()`) still applies the change and names **every**
## other row now sharing it, not just the first one found. Eighteen actions
## route through `DccMenus._bind_accelerator`, so three-way sharing is
## reachable in a handful of clicks, and a message naming one of two others
## is a list silently truncated to its first element -- the same shape as
## reporting a single sample as a measurement. A key already meaningful in a
## DIFFERENT context (the armed-tool ladder, a per-domain tool button, the
## Layers popover) is never flagged at all -- the owner's own example, "the
## same key means different things with a tool armed", is not a bug this
## dialog polices.

var _app: Node
var _list: VBoxContainer
var _phone := false

## Toggled by which entry point opened the dialog. Read-only Help traffic
## (`open()`) never sets this; `open_editable()` does. `_rebuild()` is the
## only place that reads it -- one row builder, one list, two renderings.
var _editable := false
var _edit_bar: HBoxContainer
var _status: Label

## Populated fresh by every `_rebuild()`, only for rows `_editable` actually
## draws a chip for. `id -> {"popup": PopupMenu, "index": int}` -- what a
## rebind or a reset needs to reach the live item again; `id -> String` the
## row's own label, for a conflict message naming every OTHER row that
## already answers to the chord -- so the message is a list, not one name.
var _capture_popups: Dictionary = {}
var _capture_labels: Dictionary = {}
var _capture_chips: Dictionary = {}

## The action id currently waiting for a keypress, or `-1`. Set by clicking a
## row's chip, cleared by Escape, a completed capture, or any `_rebuild()`
## (so reopening the dialog, or resetting a row, can never leave a chip
## stuck reading "Press a key…" for an action that no longer exists at that
## popup/index).
##
## **Never assigned directly -- always through [`_set_capturing`]**, which
## owns the `dialog_close_on_escape` half of the same state.
var _capturing_id := -1

## Shortcuts that exist but have no menu row to read them from. Each carries
## the file that owns it, so a reader can check the claim rather than trust it.
##
## `PARITY_AUDIT.md` §20 records the layer digits as an open owner decision --
## *"F10 — layer hotkeys `1–8` vs the reference's `0 B T F S W R`"*, unmade for
## three passes. Listing them here does not settle that; it makes the current
## answer visible, which is the first thing a decision needs.
const UNLISTED: Array = [
	["Map view", "1 – 8", "Switch the active layer (Layers popover)", "layers_popover.gd"],
	["Map view", "Space", "Hold to pan the map", "viewport_host.gd"],
	["Map view", "Esc", "Cancel the active tool or close the top sheet", "app.gd"],
	["Map view", "Delete / Backspace", "Remove the selected item", "app.gd"],
	["Civilization", "Shift+J", "Open the Journey planner", "app.gd"],
]

func setup(app: Node) -> void:
	_app = app
	title = "Keyboard shortcuts"
	size = Vector2i(560, 520)
	ok_button_text = "Close"
	_phone = DccWidgets.phone_window(self, app)

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 6)
	add_child(body)
	if _phone:
		DccWidgets.phone_head(body, "Keyboard shortcuts", "read from the live menus")

	var hint := DccWidgets.note(body,
		"Read from the menus themselves each time this opens, so it cannot fall out of step with what the application does.")
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD

	## Editable-only chrome. Built once here and toggled by `open()`/
	## `open_editable()` rather than rebuilt per open -- `_rebuild()` already
	## owns the row list below; this owns the controls around it.
	_edit_bar = HBoxContainer.new()
	_edit_bar.add_theme_constant_override("separation", 10)
	body.add_child(_edit_bar)
	DccWidgets.text_button(_edit_bar, "Restore all to defaults", func(): _reset_all())

	_status = DccTheme.label("", "text_dim", DccTheme.FS_SMALL)
	_status.autowrap_mode = TextServer.AUTOWRAP_WORD
	_status.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(_status)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	body.add_child(scroll)

	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 2)
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_list)

## `Help ▸ Keyboard shortcuts…` -- read-only, exactly as before this file
## grew an editable mode.
## Calls two and three of the phone protocol, in one place because there are two
## entry points and they had the same defect.
##
## `setup()` already made call one -- `phone_window(self, app)` at build, plus
## the `phone_head()` that a borderless window needs -- so this file looked
## converted and was not. **The fit and the presentation were simply absent**,
## and both `open()` and `open_editable()` ended in a bare `popup_centered()`,
## which on a phone opens a 560x520 desktop dialog at scale 1.0 inside a
## 1080-wide screen.
##
## It must run AFTER `_rebuild()`: `phone_fit()` walks the built tree to raise
## anything under the 44 dp floor, and this list is rebuilt on every open
## because it reads the live menus.
func _present_phone_aware() -> void:
	if _phone and _app != null and _app.has_method("phone_fit"):
		_app.phone_fit(self, 1.0)
	if not DccWidgets.phone_present(self, _app):
		popup_centered()

func open() -> void:
	_editable = false
	_edit_bar.visible = false
	_status.visible = false
	_rebuild()
	_present_phone_aware()

## `Preferences ▸ Keyboard shortcuts…` -- the same list, rebindable. See this
## file's own header on why it is a mode of this dialog and not a second one.
func open_editable() -> void:
	_editable = true
	_edit_bar.visible = true
	_status.visible = true
	_status.text = ""
	_rebuild()
	_present_phone_aware()

func _rebuild() -> void:
	for c in _list.get_children():
		c.queue_free()
	_capture_popups.clear()
	_capture_labels.clear()
	_capture_chips.clear()
	_set_capturing(-1)

	var found := 0
	for entry in _collect_from_menus():
		_row(entry[0], entry[1], entry[2], entry[3], entry[4], entry[5])
		found += 1

	if found == 0:
		## Never an empty dialog pretending to be a complete answer. This
		## repository's own rule: assert non-emptiness rather than let silently
		## empty output read as success.
		_group("No menu accelerators found")
		_note_row("The menu bar reported no shortcuts. That is a bug in this dialog or in the menu build, not an application with none -- see ShortcutsDialog's own doc comment.")

	_group("Not on a menu")
	for u in UNLISTED:
		_fixed_row(u[0], u[1], "%s   (%s)" % [u[2], u[3]])

## Walks the real `MenuBar` and reports `(menu title, accelerator, row label,
## item id, popup, item index)` for every row carrying an accelerator, in menu
## order. The last three fields exist only so editable mode can reach the
## live item again to rebind or reset it -- read-only mode ignores them.
func _collect_from_menus() -> Array:
	var out: Array = []
	var bar := _find_menu_bar(_app)
	if bar == null:
		return out
	var last_menu := ""
	for mb in bar:
		var popup: PopupMenu = mb.get_popup()
		if popup == null:
			continue
		last_menu = mb.text
		_walk_popup(popup, last_menu, out)
	return out

func _walk_popup(popup: PopupMenu, menu_name: String, out: Array) -> void:
	for i in popup.item_count:
		var accel: int = popup.get_item_accelerator(i)
		if accel != 0:
			out.append([menu_name, _accel_text(accel), popup.get_item_text(i),
				popup.get_item_id(i), popup, i])
		## Submenus carry accelerators too -- `Preferences ▸ Art packs` puts
		## Ctrl+Shift+P on a submenu row, so a walk that stopped at the top
		## level would miss it.
		var sub := popup.get_item_submenu(i)
		if sub != "":
			var node := popup.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_walk_popup(node as PopupMenu, menu_name, out)

## `MenuButton`s keep their popup as an INTERNAL child, so the default
## `get_children()` walk finds none of them -- the trap `_loddbg_probe.gd`
## documents paying for. `get_children(true)` is load-bearing here.
func _find_menu_bar(n: Node) -> Array:
	var buttons: Array = []
	_gather_menu_buttons(n, buttons)
	return buttons

func _gather_menu_buttons(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather_menu_buttons(c, out)

func _accel_text(accel: int) -> String:
	return OS.get_keycode_string(accel) if accel != 0 else "—"

func _group(text: String) -> void:
	var l := DccTheme.mono_label(text.to_upper(), "text_dim", DccTheme.FS_SMALL, 1)
	l.custom_minimum_size.y = 26
	l.vertical_alignment = VERTICAL_ALIGNMENT_BOTTOM
	_list.add_child(l)

func _note_row(text: String) -> void:
	var l := DccTheme.label(text, "text_dim", DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD
	_list.add_child(l)

## A row for a real `menus.gd` accelerator. In editable mode, when
## `DccMenus.is_rebindable_shortcut(id)` says this action is one the menu
## system will actually reapply an override to, the key column becomes a
## click-to-rebind chip plus a conditional "reset"; otherwise (or in
## read-only mode) it is the same plain mono label this dialog always drew.
func _row(menu_name: String, accel: String, label: String, id: int, popup: PopupMenu, index: int) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	## `_app` is typed `Node` (like every `DccApp` reference this dialog's
	## sibling windows hold), so `.menus` resolves dynamically -- `bool()`
	## rather than `:=` inference, which cannot type a Variant-returning call.
	var editable_here: bool = _editable and bool(_app.menus.is_rebindable_shortcut(id))
	if editable_here:
		_capture_popups[id] = {"popup": popup, "index": index}
		_capture_labels[id] = label
		var chip := DccWidgets.chip(row, accel, Callable())
		chip.custom_minimum_size.x = 140
		chip.tooltip_text = "Click, then press the new key combination. Esc cancels."
		if _phone:
			chip.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN
		chip.pressed.connect(func(): _begin_capture(id))
		_capture_chips[id] = chip
	else:
		var key := DccTheme.mono_label(accel, "accent", DccTheme.FS_SMALL)
		key.custom_minimum_size.x = 140
		row.add_child(key)

	var what := DccTheme.label("%s — %s" % [menu_name, label], "text", DccTheme.FS_SMALL)
	what.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	what.autowrap_mode = TextServer.AUTOWRAP_WORD
	row.add_child(what)

	## Only a row the user has actually rebound offers a way back to its
	## shipped key -- an always-present "reset" beside an already-default row
	## would be a control with nothing to do.
	if editable_here and DccSettings.has_shortcut_override(DccSettings.SHORTCUT_CONTEXT_MENU, id):
		DccWidgets.text_button(row, "reset", func(): _reset_one(id))

	if _phone:
		row.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN
	_list.add_child(row)

## The `[UNLISTED]` rows -- never a rebind control, in either mode, because
## none of them are a `menus.gd` accelerator to begin with. See this file's
## own header on why.
func _fixed_row(menu_name: String, accel: String, label: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var key := DccTheme.mono_label(accel, "accent", DccTheme.FS_SMALL)
	key.custom_minimum_size.x = 140
	row.add_child(key)

	var what := DccTheme.label("%s — %s" % [menu_name, label], "text", DccTheme.FS_SMALL)
	what.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	what.autowrap_mode = TextServer.AUTOWRAP_WORD
	row.add_child(what)

	if _editable:
		row.tooltip_text = "A different keyboard context (an armed tool, or a control with its own binding) -- not part of this table."
	if _phone:
		row.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN
	_list.add_child(row)

# -- Rebinding (editable mode only) --------------------------------------------

## `_capturing_id` and `dialog_close_on_escape` are one state, so one function
## writes both and no caller has to remember the second half.
##
## **Escape has to mean two different things here, and `AcceptDialog` only
## knows one of them.** Idle, Escape closes the dialog, which is how a reader
## gets out of a reference sheet and must keep working. Mid-capture, the
## chip's own tooltip promises "Esc cancels" -- and `AcceptDialog` consumes
## `ui_cancel` in `_input_from_window` whether or not `_unhandled_key_input`
## below already handled it, so before this the promise was **half true**: the
## capture did abort, and the whole dialog vanished with it, taking any
## conflict line in `_status` along. Measured with real routed input rather
## than reasoned about -- `_chordcap_probe.gd` §B pushes an `InputEventKey`
## through the owning viewport and asserts `visible` afterwards; the same
## assertion fails on the version before this function existed.
func _set_capturing(id: int) -> void:
	_capturing_id = id
	dialog_close_on_escape = (id == -1)

func _begin_capture(id: int) -> void:
	if _capturing_id != -1 and _capturing_id != id:
		_end_capture_visual(_capturing_id)
	_set_capturing(id)
	_status.text = ""
	var chip: Button = _capture_chips.get(id)
	if chip != null:
		chip.text = "Press a key… (Esc cancels)"
		chip.add_theme_color_override("font_color", DccTheme.c("accent"))

## Repaints one row's chip back to whatever its live item's accelerator
## actually is right now -- used both to abandon a capture (nothing was
## written, so this just re-reads the unchanged value) and after a commit.
func _end_capture_visual(id: int) -> void:
	var chip: Button = _capture_chips.get(id)
	var pi: Dictionary = _capture_popups.get(id, {})
	if chip == null or pi.is_empty():
		return
	var accel: int = (pi["popup"] as PopupMenu).get_item_accelerator(int(pi["index"]))
	chip.text = _accel_text(accel)
	chip.add_theme_color_override("font_color", DccTheme.c("text"))

## `Window`, so its own `Viewport` -- the key arrives here only while this
## dialog has focus, the same precedent `asset_library_window.gd`'s own
## Delete/Backspace capture documents. Guarded on `visible` too, defensively:
## a capture abandoned by closing the dialog outright (rather than Escape or
## a completed rebind) must not survive as a stray armed state.
func _unhandled_key_input(event: InputEvent) -> void:
	if not visible or _capturing_id == -1:
		return
	if not (event is InputEventKey and event.pressed and not event.echo):
		return
	var id := _capturing_id
	if event.keycode == KEY_ESCAPE:
		_set_capturing(-1)
		_end_capture_visual(id)
		get_viewport().set_input_as_handled()
		return
	if event.keycode in [KEY_CTRL, KEY_SHIFT, KEY_ALT, KEY_META]:
		return   ## a bare modifier -- keep waiting for the real key
	## **An unmappable key must not be committed, because `0` is this dialog's
	## own spelling of "no shortcut" and committing it DESTROYS the row.**
	## `get_keycode_with_modifiers()` returns 0 for a key Godot cannot map, and
	## `set_shortcut_binding(.., 0)` persists that as an override -- after which
	## `_walk_popup` skips the row (it collects only `accel != 0`), so the entry
	## leaves the generated table **and takes its own rebind chip with it**. Only
	## Restore all brings it back. Found by a verifier 2026-09-06 by pushing a
	## keycode-0 event at Undo, desktop and phone: `268435546 -> 0`, persisted,
	## row gone. The probe assertion that should have caught it checked a FRESH
	## boot, where no override exists, so it passed vacuously.
	##
	## Keep waiting rather than aborting: the user pressed something, and
	## silently closing the capture would read as "it took it".
	var accel: int = event.get_keycode_with_modifiers()
	if accel == 0:
		return
	get_viewport().set_input_as_handled()
	_commit_capture(id, accel)

func _commit_capture(id: int, accel: int) -> void:
	_set_capturing(-1)
	var pi: Dictionary = _capture_popups.get(id, {})
	if pi.is_empty():
		return
	var popup: PopupMenu = pi["popup"]
	var index: int = int(pi["index"])
	DccSettings.set_shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, id, accel)
	## The live reapply -- no restart, no menu rebuild. This one call is what
	## makes the write above actually true the instant it happens.
	popup.set_item_accelerator(index, accel)
	## Blender's rule, not Photoshop's (`GUI_GAP_REGISTER.md` §7.9 item 3):
	## shown, never blocked. Every one of these rows still fires; which one a
	## duplicate accelerator reaches first is Godot's popup match order, not
	## this dialog's business to hide -- so the reader is told the whole set
	## and decides, rather than being handed the first name and left to
	## discover the rest by pressing the key.
	_status.text = _collision_text(_conflicts_within_menu(id, accel))
	## Cheapest correct refresh: the row's own "reset" control just became
	## live-or-not, and a second code path that patches one row in place
	## would be a second place for that to drift from `_row()` itself.
	_rebuild()

## EVERY other action already bound to `accel` in this same walk, in menu
## order; empty when the chord is unique. Scoped to `_capture_popups`' own
## ids on purpose -- they are, by construction, exactly the ids
## `DccMenus.is_rebindable_shortcut()` knows about, which is the entire
## "menu" context. A key meaningful in a different context never appears here
## to begin with, so it can never be reported as a conflict -- the owner's
## ruling, enforced by what this dict does and does not contain rather than
## by a second check.
##
## Returns a list rather than the first hit: this walk sees eighteen
## rebindable actions and nothing stops three of them sharing a chord, at
## which point "the other row" is not a thing that exists.
func _conflicts_within_menu(exclude_id: int, accel: int) -> Array:
	var out: Array = []
	for other_id in _capture_popups.keys():
		if other_id == exclude_id:
			continue
		var pi: Dictionary = _capture_popups[other_id]
		if (pi["popup"] as PopupMenu).get_item_accelerator(int(pi["index"])) == accel:
			out.append(other_id)
	return out

## The status line for a conflict set -- `""` for none, so the caller can
## assign it unconditionally and an empty line always means "no conflict"
## rather than "not checked".
##
## The counting word is written from the set's real size (`both` for one
## other, `all three` for two, `all N` beyond) because a fixed "both" in
## front of three names is a sentence that contradicts its own list.
func _collision_text(others: Array) -> String:
	if others.is_empty():
		return ""
	var names := PackedStringArray()
	for other_id in others:
		names.append("\"%s\"" % String(_capture_labels.get(other_id, "?")))
	var joined := String(names[0])
	if names.size() > 1:
		joined = ", ".join(names.slice(0, names.size() - 1)) + " and " + names[names.size() - 1]
	var total := others.size() + 1
	var how_many := "both" if total == 2 else ("all three" if total == 3 else "all %d" % total)
	return "Also bound to %s -- %s will fire until one of them changes." % [joined, how_many]

func _reset_one(id: int) -> void:
	var pi: Dictionary = _capture_popups.get(id, {})
	if pi.is_empty():
		return
	DccSettings.clear_shortcut_binding(DccSettings.SHORTCUT_CONTEXT_MENU, id)
	(pi["popup"] as PopupMenu).set_item_accelerator(int(pi["index"]), int(_app.menus.shortcut_default(id)))
	_status.text = ""
	_rebuild()

## The reset path a user who binds themselves out of the shell needs --
## every rebindable action, back to its shipped key, in one action.
func _reset_all() -> void:
	DccSettings.reset_shortcuts(DccSettings.SHORTCUT_CONTEXT_MENU)
	for id in _capture_popups.keys():
		var pi: Dictionary = _capture_popups[id]
		(pi["popup"] as PopupMenu).set_item_accelerator(int(pi["index"]), int(_app.menus.shortcut_default(id)))
	_status.text = "Every menu shortcut restored to its shipped key."
	_rebuild()
