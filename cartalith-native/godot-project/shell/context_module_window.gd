extends AcceptDialog

## PC-10 (`PC_SHELL_PROPOSAL.md` §3.3, "A window": a tab's menu ▸ Open in a window "shows that module in
## a §3.6 window. The window follows the selection until pinned"). One window per module, made by
## `DccApp.open_context_tab_window()`.
##
## **What it does.** Draws one Context panel module through the same builder the panel's tab calls
## (`RightDock.build_module_into()`, which runs the module's registry Callable), for the subject it
## follows. It is one of PC-7's non-blocking windows (`ContextWindow.make_context()`): the map, the
## menus and the docks keep working while it is open, and its FOLLOW / PINNED chip pins it to what it
## shows. It re-reads on a year move, a generate and an open, once a frame at most.
##
## **What it must never do.** Exist on the tablet or the phone: §3.11 keeps their windows modal and does
## not offer Open in a window there, so `DccApp.open_context_tab_window()` refuses before making one.
## Keep a copy of the selection: it holds only the subject it is pinned to, read from
## `DccApp.selection` while following. Draw a module that does not fit the subject: it says which kinds
## the module shows for instead (§3.3, "Fit"). It has no Pop out button (Ruling DK's per-window Pop out is
## for `PopoutHost.LIVE_KEYS`; a module window is not one of them, a stated gap).

## PC-7's rules, shared by every non-blocking window.
const ContextWindow := preload("res://shell/context_window.gd")
## The modules' specs (`fits()`, `spec()`, the title).
const ContextModules := preload("res://shell/context_modules.gd")
## The node name of the box the module draws into, so a probe finds it.
const BODY_NAME := "ModuleWindowBody"

## `DccApp`.
var _host
## The module this window draws (`context_modules.gd`'s key).
var module_key := ""
## The subject drawn: the selection while following, the pinned one while pinned. Written by `open()`,
## `follow_selection()` and nothing else; `{}` is no subject.
var _subject: Dictionary = {}
## The box the module is drawn into, inside the window's scroll.
var _body: VBoxContainer
## The subject line under the title row.
var _subject_label: Label
## True while a re-read is queued for the next idle frame (`rebuild_later()`).
var _queued := false

## Builds the frame once and wires its re-reads. Call once, before `open()`.
func setup(host, key: String) -> void:
	_host = host
	module_key = key
	name = "ContextModuleWindow_" + key
	title = ContextModules._title(key)
	## A labelled judgement: the dock's width and a little more, tall enough for a module's page.
	size = Vector2i(420, 560)
	min_size = Vector2i(320, 260)
	ok_button_text = "Close"
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)
	_subject_label = DccTheme.label("", "text_dim", DccTheme.FS_SMALL)
	_subject_label.name = "ModuleWindowSubject"
	_subject_label.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	DccWidgets.pad(outer, 12, 6, 12, 6).add_child(_subject_label)
	outer.add_child(DccTheme.rule())
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(scroll)
	_body = VBoxContainer.new()
	_body.name = BODY_NAME
	_body.add_theme_constant_override("separation", 4)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Tells `RightDock.build_here()` it is drawing into a window, not the panel.
	_body.set_meta("ctx_module_window", true)
	scroll.add_child(_body)
	host.bridge.generation_finished.connect(func(_ok: bool): rebuild_later())
	host.bridge.world_loaded.connect(rebuild_later)
	ContextWindow.make_context(self, host, true)
	if host.has_signal("timeline_changed"):
		host.timeline_changed.connect(rebuild_later)

## Shows the window following again, on the subject the panel's tab shows (what the user was looking at
## when they asked for the window), and raises it when it is already open.
func open() -> void:
	ContextWindow.reset_on_open(self)
	_subject = _host.right_dock_ctrl._module_subject() if _host.right_dock_ctrl != null else {}
	rebuild()
	if visible:
		grab_focus()
	else:
		popup_centered()

## PC-7. Called by `ContextWindow` when the selection moves while this window is visible and following.
func follow_selection(sel: Dictionary) -> void:
	_subject = sel.duplicate()
	rebuild()

## PC-7. Nothing is held: a module commits each edit through its window's own path as it is made.
func held_changes() -> String:
	return ""

## PC-7. Nothing to discard (see `held_changes()`).
func discard_held_changes() -> void:
	pass

## Queues one `rebuild()` for the next idle frame while visible (a scrub moves the year many times a
## second; a generate's signal arrives while the engine is still settling).
func rebuild_later() -> void:
	if not visible or _queued:
		return
	_queued = true
	(func() -> void:
		_queued = false
		if visible:
			rebuild()).call_deferred()

## Redraws the module for `_subject`: through `RightDock.build_module_into()` when it fits, else one
## note naming the kinds it shows for. Never while a generation runs (the engine is the worker's; the
## `generation_finished` re-read follows).
func rebuild() -> void:
	if _body == null:
		return
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	var dock = _host.right_dock_ctrl
	_subject_label.text = String(dock.subject_name(_subject)) if not _subject.is_empty() else "Nothing selected"
	if bool(_host.bridge.generating):
		DccWidgets.note(_body, "A world is being generated; this re-reads when it is done.")
		return
	if not ContextModules.fits(module_key, _subject):
		var kinds: Array = ContextModules.spec(module_key).get("kinds", [])
		DccWidgets.note(_body, "%s shows for a %s. Select one, or pin this window to keep what it shows." % [
			title, ", ".join(kinds).replace(", %s" % kinds[-1], " or %s" % kinds[-1]) if kinds.size() > 1 else (kinds[0] if not kinds.is_empty() else "subject")])
		return
	dock.build_module_into(_body, module_key, _subject)
