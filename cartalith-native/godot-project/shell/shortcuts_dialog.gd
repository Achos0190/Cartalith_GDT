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

var _app: Node
var _list: VBoxContainer
var _phone := false

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

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	body.add_child(scroll)

	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 2)
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_list)

func open() -> void:
	_rebuild()
	popup_centered()

func _rebuild() -> void:
	for c in _list.get_children():
		c.queue_free()

	var found := 0
	for entry in _collect_from_menus():
		_row(entry[0], entry[1], entry[2])
		found += 1

	if found == 0:
		## Never an empty dialog pretending to be a complete answer. This
		## repository's own rule: assert non-emptiness rather than let silently
		## empty output read as success.
		_group("No menu accelerators found")
		_note_row("The menu bar reported no shortcuts. That is a bug in this dialog or in the menu build, not an application with none -- see ShortcutsDialog's own doc comment.")

	_group("Not on a menu")
	for u in UNLISTED:
		_row(u[0], u[1], "%s   (%s)" % [u[2], u[3]])

## Walks the real `MenuBar` and reports `(menu title, accelerator, row label)`
## for every row carrying an accelerator, in menu order.
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
			out.append([menu_name, _accel_text(accel), popup.get_item_text(i)])
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
	return OS.get_keycode_string(accel)

func _group(text: String) -> void:
	var l := DccTheme.mono_label(text.to_upper(), "text_dim", DccTheme.FS_SMALL, 1)
	l.custom_minimum_size.y = 26
	l.vertical_alignment = VERTICAL_ALIGNMENT_BOTTOM
	_list.add_child(l)

func _note_row(text: String) -> void:
	var l := DccTheme.label(text, "text_dim", DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD
	_list.add_child(l)

func _row(menu_name: String, accel: String, label: String) -> void:
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

	if _phone:
		row.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN
	_list.add_child(row)
