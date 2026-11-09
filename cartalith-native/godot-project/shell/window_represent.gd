extends RefCounted

## MS-1 S4.5 (`MOBILE_SHELL_MS1_SPEC.md` section 3.5, Ruling DY MQ-5; `S4_INVENTORY.md` sections 5 and 18): the helpers
## the windows' re-present paths share, and the contract itself.
##
## **Why a window is re-presented by building it again.** A window decides its layout in three places: `setup()`'s
## `_phone` branches, `DccWidgets.phone_present()` (a content scale, a size, a borderless flag), and `phone_fit()` (font
## and minimum-size overrides with no inverse, `_PHONE_FIT_META`). None can be undone in place, so the one path that
## cannot differ from a cold boot is the cold boot's own: `DccApp._represent_windows()` frees the old instance, builds a new
## one through the same `_make_<window>()` that `_ready()` calls, and gives it back what the user had set. A window that
## takes part declares two methods (a `has_method` probe, as `quiescent_blockers()` does):
##
## - `represent_capture() -> Dictionary`: what the user set, read BEFORE the swap unmounts anything (it may commit a
##   half-typed field, as the window's own re-open does). Every key is a real value: a window that was not open says
##   `"open": false` and carries only what a later open would use. It must never hold a node (the window is freed).
## - `represent_restore(bag: Dictionary) -> void`: on the NEW instance, after the composition is mounted. Re-opens the
##   window if `bag["open"]` and puts the state back; it may be a coroutine (a scroll offset needs laid-out frames).
##
## A window with these two methods is no longer a reason for a swap to wait (`DccShell._collect_window_blockers()`); one
## that holds unconfirmed input still says so through `held_changes()` or `quiescent_blockers()` and still makes it wait
## (Q-S4-2). Preloaded, no `class_name`, so it needs no `--headless --import` pass.

## How many frames `restore_scroll()` keeps trying. A labelled judgement from `--leg represent`: six was too few for the
## faction roster, which re-measures its autowrapped overview one deferred call after `popup()` (`faction_roster_window.gd`,
## "2 602 -> 581"), so its scroll maximum grows for a few frames after it first shows; the retry stops as soon as it holds.
const SCROLL_RETRY_FRAMES := 30

## The `ScrollContainer` that holds `n`, or null. A window's content scroll is the one its body sits in, on the phone's
## single column and on the desktop's detail pane alike, so no window needs to know which of its scrolls that is.
static func scroll_above(n: Node) -> ScrollContainer:
	var p := n.get_parent() if n != null else null
	while p != null:
		if p is ScrollContainer:
			return p as ScrollContainer
		p = p.get_parent()
	return null

## The vertical offset of the scroll above `n`, or 0 when there is none (a real offset of 0, since an unscrolled
## window is at the top; the bag's reader never needs to tell "none" from "top").
static func scroll_value(n: Node) -> int:
	var sc := scroll_above(n)
	return 0 if sc == null else sc.scroll_vertical

## Puts the scroll above `n` back to `value`, after the layout: a `ScrollContainer`'s maximum is only right once its
## content has been laid out in the window's new size (the engine clamps an earlier write to the old maximum). A coroutine
## the caller does not await.
static func restore_scroll(n: Node, value: int) -> void:
	if value <= 0 or n == null or not n.is_inside_tree():
		return
	var tree := n.get_tree()
	## A bounded wait for the layout, not a fixed count: the engine clamps a write to the scroll's current maximum, so the
	## write is retried each frame until it holds or the frames run out (`SCROLL_RETRY_FRAMES`).
	for k in SCROLL_RETRY_FRAMES:
		await tree.process_frame
		if not is_instance_valid(n):
			return
		var sc := scroll_above(n)
		if sc != null:
			sc.scroll_vertical = value
			if sc.scroll_vertical == value:
				return
