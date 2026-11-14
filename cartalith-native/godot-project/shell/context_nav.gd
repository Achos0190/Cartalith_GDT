extends RefCounted

## PC-7's leftover (`PC_SHELL_PROPOSAL.md` §3.6 "One host with a back stack", finding W2): the one place that
## remembers which context windows a selection opened, so one window opening another PUSHES onto a trail instead
## of closing the first or stacking a second window beside it, and Back and Forward walk that trail.
##
## **The five windows it hosts** are the faction roster, the Place editor, the City viewer, Conflicts and Saved
## journeys, on the desktop form only (`ContextWindow.applies()`); the tablet and the phone keep their modal
## windows and never register here, so nothing on them changes. A window registers once, from its `setup()`
## (`ContextWindow.attach_nav()`), and announces every open of its own with `ContextWindow.arrived()`.
##
## **What a visit is.** An entry is `{win, state}`: the window and what it showed (`context_state()`, a plain
## Dictionary the window itself defines and re-reads with `context_restore()`). A visit is recorded when a window
## is OPENED (its `open*()` ran), never when the selection moves under a following window: that is the Context
## panel's own selection history (PC-4), a different trail. Opening a window while another embedded context window
## is showing HIDES the first and puts the new one where the first was (the mockup's one `win` with a `stack`,
## `design/mobile-shell-2026-10-31/Shell.dc.html::_openWin`); it does not discard it. Back and Forward then swap
## them again, re-opening each through its own `open*()` path with the state it was left in, so a window never
## has a second way of being built.
##
## **What it must never do.**
## - Touch a popped-out (native) window: it is its own OS window on its own screen (Ruling DK), so it is neither
##   hidden nor moved nor recorded. `arrived()` ignores a window that is not embedded.
## - Record a visit while it is itself walking the trail (`_moving`): the `open*()` a Back press calls announces
##   itself like any open, and must not push onto the trail it is walking.
## - Keep a trail past the host: when no registered window is showing any more (the user closed the last one, or
##   a pop-out took it), the trail is dropped, and so is it on a world change (an index into
##   `bridge.settlements()` means nothing in the next world, `generation_finished` / `world_loaded`).
## - Hold the engine: `context_state()` reads window fields only.
##
## Preloaded by `app.gd` (`DccApp.context_nav`), no `class_name`, so adding it needs no `--headless --import`
## pass (CLAUDE.md: that pass strips `project.godot`'s `;` comments).

## The most visits one trail keeps; the oldest drops first. A labelled judgement: a person who has walked
## further than this back through windows opened by one selection has started something else.
const MAX_ENTRIES := 24

## Names a probe finds the nav row's parts by.
const BAR_NAME := "ContextNavBar"
const BACK_NAME := "ContextNavBack"
const FORWARD_NAME := "ContextNavForward"
const POS_NAME := "ContextNavPos"
## The wrapper `ContextWindow.attach_nav()` puts around a window's content so the bar can sit above it.
const WRAP_NAME := "ContextNavWrap"

## Emitted after the trail or the cursor changed, with no argument: a probe waits on it, the bars follow it.
signal changed

## `DccApp`.
var host
## `[{win: Window, state: Dictionary}]`, oldest first. `_cursor` indexes the window on screen.
var _entries: Array = []
var _cursor := -1
## True while `go()` re-opens a window: the open is not a visit (see "What it must never do").
var _moving := false
## `Window` -> `{bar, back, fwd, pos}`: the windows that registered, with the controls to keep in step.
var _bars: Dictionary = {}


func _init(h) -> void:
	host = h
	## A new world makes every stored index meaningless; a trail across it would walk to strangers.
	var bridge = h.get("bridge") if h != null else null
	if bridge != null:
		if bridge.has_signal("generation_finished"):
			bridge.generation_finished.connect(func(_ok: bool) -> void: clear())
		if bridge.has_signal("world_loaded"):
			bridge.world_loaded.connect(func() -> void: clear())


## Registers `win` and the controls of its nav row. Called once, by `ContextWindow.attach_nav()`.
func register(win: Window, bar: Control, back: Button, fwd: Button, pos: Label) -> void:
	_bars[win] = {"bar": bar, "back": back, "fwd": fwd, "pos": pos}
	win.visibility_changed.connect(func() -> void: _on_visibility(win))
	_sync()


## How many visits the trail holds (a probe reads it).
func size() -> int:
	return _entries.size()


## The cursor into the trail, `-1` when it is empty.
func cursor() -> int:
	return _cursor


## The window on screen for the cursor, or `null`.
func current_window() -> Window:
	if _cursor < 0 or _cursor >= _entries.size():
		return null
	var w = (_entries[_cursor] as Dictionary)["win"]
	return w if is_instance_valid(w) else null


func can_back() -> bool:
	return _cursor > 0


func can_forward() -> bool:
	return _cursor >= 0 and _cursor < _entries.size() - 1


## One line per visit, `Name` as its window words it (`context_label()`), the cursor's marked `>`; for a tooltip
## and a probe. A window no longer valid shows as `?`.
func trail() -> PackedStringArray:
	var out := PackedStringArray()
	for i in _entries.size():
		var e: Dictionary = _entries[i]
		out.append("%s%s" % ["> " if i == _cursor else "  ", _label_of(e)])
	return out


## Forgets the trail (the nav rows hide).
func clear() -> void:
	if _entries.is_empty() and _cursor == -1:
		return
	_entries.clear()
	_cursor = -1
	_sync()


## A registered window finished its own `open*()`: record the visit. `win` is the window announcing itself.
## The previous window (the one the cursor is on), when it is still showing and both are embedded, is
## hidden, and `win` takes its place on screen.
func arrived(win: Window) -> void:
	if _moving or win == null or not win.is_embedded():
		return
	var cur := current_window()
	var state := _state_of(win)
	if cur != null and _cursor >= 0:
		if cur == win:
			## A re-open of the window on screen: a new visit only when it shows something else.
			if state == (_entries[_cursor] as Dictionary)["state"]:
				_sync()
				return
		else:
			(_entries[_cursor] as Dictionary)["state"] = _state_of(cur)
			if cur.visible and cur.is_embedded():
				_take_place(win, cur)
	## A new visit cuts off whatever Forward would have walked to, as a browser does.
	_entries = _entries.slice(0, _cursor + 1)
	_entries.append({"win": win, "state": state})
	if _entries.size() > MAX_ENTRIES:
		_entries = _entries.slice(_entries.size() - MAX_ENTRIES)
	_cursor = _entries.size() - 1
	_sync()


## Back: the window the selection opened before this one, shown as it was left. A no-op at the start.
func back() -> void:
	_go(_cursor - 1)


## Forward: undoes a Back. A no-op at the end of the trail.
func forward() -> void:
	_go(_cursor + 1)


## Walks the trail to entry `to`: the current window's state is stored, it is hidden when the target is another
## window, and the target is re-opened through its own `context_restore()` where the current one stood.
func _go(to: int) -> void:
	if to < 0 or to >= _entries.size() or to == _cursor:
		return
	var cur := current_window()
	var target: Dictionary = _entries[to]
	var tw = target["win"]
	if not is_instance_valid(tw):
		_entries.remove_at(to)
		if to < _cursor:
			_cursor -= 1
		_sync()
		return
	if cur != null:
		(_entries[_cursor] as Dictionary)["state"] = _state_of(cur)
	_moving = true
	var at := Vector2i.ZERO
	var had_place := cur != null and cur.is_embedded() and cur.visible
	if had_place:
		at = cur.position
	if cur != null and cur != tw and cur.visible:
		cur.hide()
	(tw as Object).call("context_restore", target["state"])
	if had_place and cur != tw and (tw as Window).is_embedded():
		_place(tw as Window, at)
	_moving = false
	_cursor = to
	_sync()


## `win` was just shown by its own open; put it where `old` stands and hide `old`.
func _take_place(win: Window, old: Window) -> void:
	var at := old.position
	old.hide()
	if win.visible:
		_place(win, at)


## Moves an embedded window to `at`, clamped so its title bar and its frame stay inside the shell's window.
func _place(w: Window, at: Vector2i) -> void:
	var vs := Vector2i(w.get_tree().root.size) if w.is_inside_tree() else Vector2i(1280, 720)
	var title_h := int(w.get_theme_constant("title_height", "Window"))
	w.position = Vector2i(clampi(at.x, 0, maxi(0, vs.x - w.size.x)), clampi(at.y, title_h, maxi(title_h, vs.y - w.size.y)))


## The window's own account of what it shows, `{}` when it cannot say.
func _state_of(win: Object) -> Dictionary:
	if win != null and is_instance_valid(win) and win.has_method("context_state"):
		return win.call("context_state")
	return {}


## The window's own words for a visit ("Place: Vhal Serai"), its title when it has none.
func _label_of(e: Dictionary) -> String:
	var w = e["win"]
	if not is_instance_valid(w):
		return "?"
	if w.has_method("context_label"):
		return String(w.call("context_label", e["state"]))
	return String((w as Window).title)


## A window hid: when nothing registered is showing any more the trail is over. Deferred a frame, so a window that
## is hidden and shown again in one call (the pop-out toggle, a re-open) does not end it.
func _on_visibility(win: Window) -> void:
	if _moving:
		return
	if win.visible:
		## Shown again (docked back from a pop-out, re-opened): the row follows whether it is embedded now.
		_sync()
		return
	_check_over.call_deferred()


func _check_over() -> void:
	if _moving:
		return
	for w in _bars:
		if is_instance_valid(w) and (w as Window).visible:
			_sync()
			return
	clear()


## Drops visits whose window was freed (a class swap rebuilds the windows, `window_represent.gd`), keeping the cursor
## on the same visit when it can.
func _prune() -> void:
	var i := 0
	while i < _entries.size():
		if is_instance_valid((_entries[i] as Dictionary)["win"]):
			i += 1
			continue
		_entries.remove_at(i)
		if i <= _cursor:
			_cursor -= 1
	if _entries.is_empty():
		_cursor = -1
	elif _cursor < 0:
		_cursor = 0


## Updates every registered nav row: shown only on the window the cursor is on, only while there is a trail to
## walk, with Back and Forward enabled by the cursor and named by where they go.
func _sync() -> void:
	_prune()
	var cur := current_window()
	var stale: Array = []
	for w in _bars:
		if not is_instance_valid(w):
			stale.append(w)
			continue
		var b: Dictionary = _bars[w]
		var bar: Control = b["bar"]
		var showing: bool = _entries.size() > 1 and cur == w and (w as Window).is_embedded()
		bar.visible = showing
		(b["back"] as Button).disabled = not can_back()
		(b["fwd"] as Button).disabled = not can_forward()
		(b["pos"] as Label).text = "%d / %d" % [_cursor + 1, _entries.size()]
		(b["back"] as Button).tooltip_text = ("Back to %s" % _label_of(_entries[_cursor - 1])) if can_back() \
			else "Nothing earlier: this is the first window this selection opened."
		(b["fwd"] as Button).tooltip_text = ("Forward to %s" % _label_of(_entries[_cursor + 1])) if can_forward() \
			else "Nothing later."
		(b["pos"] as Label).tooltip_text = "\n".join(trail())
	for w in stale:
		_bars.erase(w)
	changed.emit()
