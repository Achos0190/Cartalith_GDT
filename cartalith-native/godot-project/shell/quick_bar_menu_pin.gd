extends Node
class_name QuickBarMenuPin

## **Right-click a menu row to pin or unpin it** -- Blender's "Add to Quick Favorites" gesture
## (`CUSTOM_QUICK_BAR_SCOPE.md` s.2 and Q11, milestone CQ-3, Ruling DH item 11).
##
## ## Why this exists, and the finding behind it
##
## Ruling DH item 11 allowed this gesture only if Godot's `PopupMenu` can report a right-click.
## CQ-0's headless scratch probe could not settle it (`get_focused_item()` read -1). CQ-3 settled
## it WINDOWED (`_quickbar_window_probe.gd`, section Q11), and the answer is the narrow one:
##
## * a right-click does NOT emit `PopupMenu.id_pressed` and leaves the menu open, so nothing
##   runs -- the gesture is safe to take over;
## * `PopupMenu.window_input` DOES receive the right-button press, and `get_focused_item()` then
##   names the row under the cursor -- provided the cursor really moved over it. A synthetic
##   `Input.parse_input_event` motion does not update the hover (it read -1 for the whole
##   popup); an OS-level motion (`DisplayServer.warp_mouse`) does. So the probe moves the real
##   cursor, and a user's mouse behaves as the probe's.
##
## ## What it does
##
## Listens on every `PopupMenu` in the tree (the ones present at `setup()` and every one added
## later, so a menu the shell rebuilds is covered). On a right-button press over a row it
## finds that row in a FRESH `CommandIndex` by (popup, item id), and, if the row is pinnable, pops
## a one-row menu at the cursor: "Pin to quick bar" or "Remove from quick bar". A row that is
## not a pinnable command (a check item, a parameter, a generated row, a popup this index does
## not know) is left alone: the click does what it did before, which is nothing. The reasons a
## row cannot be pinned are shown where they can be read, in the Customise window; a context
## row that only said "cannot pin" would be a second, weaker place for the same facts.
##
## It never runs a command and never reimplements one: the only effects are the same
## `DccSettings.quick_bar_add` / `quick_bar_remove` the window uses, then `DccQuickBar.refresh()`.

## Marks a popup this node already listens on, so a popup is hooked once.
const META_HOOKED := "_qb_pin_hooked"
## Item ids in the one-row menu.
const ID_PIN := 1
const ID_UNPIN := 2

var _app
var _menu: PopupMenu
## The row the open menu acts on (a copy, so a rebuild between right-click and choice is safe).
var _row: Dictionary = {}

## Hooks every popup already in the app, and every popup added from now on.
func setup(app) -> void:
	_app = app
	name = "QuickBarMenuPin"
	_menu = PopupMenu.new()
	add_child(_menu)
	_menu.id_pressed.connect(_on_choice)
	_hook_tree(app)
	get_tree().node_added.connect(_on_node_added)

func _hook_tree(n: Node) -> void:
	if n is PopupMenu:
		hook(n as PopupMenu)
	for c in n.get_children(true):
		_hook_tree(c)

func _on_node_added(n: Node) -> void:
	if n is PopupMenu and n != _menu:
		hook(n as PopupMenu)

## Listens to one popup (idempotent). Public so a probe can hook a popup it built itself.
func hook(p: PopupMenu) -> void:
	if p.has_meta(META_HOOKED):
		return
	p.set_meta(META_HOOKED, true)
	p.window_input.connect(_on_popup_input.bind(p))

func _on_popup_input(ev: InputEvent, p: PopupMenu) -> void:
	if not (ev is InputEventMouseButton):
		return
	var mb := ev as InputEventMouseButton
	if mb.button_index != MOUSE_BUTTON_RIGHT or not mb.pressed:
		return
	var i := p.get_focused_item()
	if i < 0 or i >= p.item_count:
		return
	offer(p, p.get_item_id(i))

## Pops the pin / unpin row for item `id` of `p` at the cursor when that item is a pinnable
## command; does nothing otherwise. Returns whether a menu was popped. Public for the probe.
func offer(p: PopupMenu, id: int) -> bool:
	var idx := CommandIndex.new()
	idx.build(_app, _app.bridge)
	_row = {}
	for r in idx.all():
		if r.get("_popup") == p and int(r.get("_id", -1)) == id:
			_row = r
			break
	if _row.is_empty() or not bool(_row.get("pinnable", false)):
		return false
	var pinned := false
	for e in DccSettings.quick_bar_value("bar"):
		if String(e["k"]) == String(_row["pin_key"]):
			pinned = true
	_menu.clear()
	if pinned:
		_menu.add_item("Remove from quick bar", ID_UNPIN)
	else:
		_menu.add_item("Pin to quick bar", ID_PIN)
	DccWidgets.style_popup(_menu)
	var at := Vector2i(_app.get_viewport().get_mouse_position())
	_menu.popup(Rect2i(at, Vector2i.ZERO))
	return true

func _on_choice(id: int) -> void:
	if _row.is_empty():
		return
	var key := String(_row["pin_key"])
	var title := String(_row["title"])
	var why := ""
	if id == ID_PIN:
		why = DccSettings.quick_bar_add("bar", key, title)
	elif id == ID_UNPIN:
		if not DccSettings.quick_bar_remove("bar", key):
			why = "It was not on the quick bar."
	if why != "":
		_app.set_status("hint", why, "warn")
		return
	_app.set_status("hint", ("Pinned \"%s\" to the quick bar." if id == ID_PIN else "Removed \"%s\" from the quick bar.") % title)
	if _app._quick_bar != null:
		_app._quick_bar.refresh()
