extends RefCounted
class_name PopoutGeometry

## Where a popped-out window should be put back (`POPOUT_WINDOWS_SCOPE.md` s.4 and PW-1,
## Ruling DK Q3: "remember each window's monitor and position per device, clamped on-screen").
##
## **A pure function of two plain values** -- the geometry `DccSettings.popout_geometry()` stored,
## and a list of the screens that exist NOW -- so the whole restore table is testable with literals
## and without a second monitor. `PopoutHost` is the only caller that touches `DisplayServer`
## (`capture()` and `screens_now()` below are the two reads it needs).
##
## What it must never do: return a rectangle when it has no screen to put it on. With zero screens
## reported, or nothing stored, `resolve()` answers `{"placed": false}` and OMITS the rectangle, so
## the caller leaves the window where the OS puts it; a plausible `(0, 0, w, h)` there would be a
## "no value" encoded as a value (`MISTAKES.md`, the recurring entry) and would strand a window.

## Height of the title row the operating system draws ABOVE a native window's client rectangle.
## **A labelled judgement**: 31 px was the dark Windows 11 title bar on the PW-0 machine at 96 dpi
## (`native_os_chrome.png`), and the figure is rounded up to 32. `Window.position` is the CLIENT
## origin, so a window at `y = usable.y` has its title bar off the top of the screen; the clamp
## keeps `y >= usable.y + TITLE_ROW` so the bar, the only handle for dragging it, stays reachable.
const TITLE_ROW := 32
## The least width of the title row that must stay on a screen: the scope's "at least 64 px of the
## title row visible" (PW-1 bar). The clamp below puts the WHOLE window inside the usable rectangle,
## which satisfies this by construction; the constant exists so the probe's independent check and
## the scope name the same number.
const MIN_TITLE_VISIBLE := 64

## `screens` is an Array of `{"rect": Rect2i, "usable": Rect2i}` (full and usable area of each
## screen, in the order `DisplayServer` reports them). `saved` is a `DccSettings.popout_geometry()`
## entry. `primary` is the primary screen's index (used only when no stored screen matches).
##
## Returns `{"placed": true, "rect": Rect2i, "screen": int, "how": String}` or `{"placed": false,
## "why": String}`. `how` says which rule chose the screen (`same_screen`, `same_origin`, `overlap`,
## `primary`) so a probe can tell a restore from a fallback.
##
## Screen choice, first rule that matches wins:
## 1. `same_screen`: a screen whose FULL rectangle equals the one stored (`sx sy sw sh`).
## 2. `same_origin`: a screen at the stored origin but another size (the resolution changed).
## 3. `overlap`: the screen whose usable area overlaps the stored window rectangle most.
## 4. `primary`: none of the above (the screen it was on is gone); the window is CENTRED there,
##    because its stored offset belonged to a monitor that no longer exists.
## The window is then shrunk to fit the usable area below the title row and clamped fully inside it.
static func resolve(saved: Dictionary, screens: Array, primary: int) -> Dictionary:
	if saved.is_empty():
		return {"placed": false, "why": "nothing stored"}
	for k in DccSettings.POPOUT_GEOMETRY_KEYS:
		if typeof(saved.get(k)) != TYPE_INT:
			return {"placed": false, "why": "stored geometry is malformed"}
	if int(saved["w"]) <= 0 or int(saved["h"]) <= 0:
		return {"placed": false, "why": "stored geometry is malformed"}
	var usable_screens: Array[int] = []
	for i in screens.size():
		var u: Rect2i = screens[i]["usable"]
		if u.size.x > 0 and u.size.y > TITLE_ROW:
			usable_screens.append(i)
	if usable_screens.is_empty():
		return {"placed": false, "why": "no screen reported"}

	var win := Rect2i(int(saved["x"]), int(saved["y"]), int(saved["w"]), int(saved["h"]))
	var stored_screen := Rect2i(int(saved["sx"]), int(saved["sy"]), int(saved["sw"]), int(saved["sh"]))
	var pick := -1
	var how := ""
	for i in usable_screens:
		if (screens[i]["rect"] as Rect2i) == stored_screen:
			pick = i
			how = "same_screen"
			break
	if pick < 0:
		for i in usable_screens:
			if (screens[i]["rect"] as Rect2i).position == stored_screen.position:
				pick = i
				how = "same_origin"
				break
	if pick < 0:
		var best := 0
		for i in usable_screens:
			var o: Rect2i = win.intersection(screens[i]["usable"])
			var area: int = o.size.x * o.size.y
			if area > best:
				best = area
				pick = i
				how = "overlap"
	var centre := false
	if pick < 0:
		pick = primary if usable_screens.has(primary) else usable_screens[0]
		how = "primary"
		centre = true

	var u: Rect2i = screens[pick]["usable"]
	var w := clampi(win.size.x, 1, u.size.x)
	var h := clampi(win.size.y, 1, u.size.y - TITLE_ROW)
	var x: int
	var y: int
	if centre:
		x = u.position.x + (u.size.x - w) / 2
		y = u.position.y + TITLE_ROW + (u.size.y - TITLE_ROW - h) / 2
	else:
		x = clampi(win.position.x, u.position.x, u.end.x - w)
		y = clampi(win.position.y, u.position.y + TITLE_ROW, u.end.y - h)
	return {"placed": true, "rect": Rect2i(x, y, w, h), "screen": pick, "how": how}

## How many pixels of the title row of `rect` lie on some screen's usable area, the best screen
## winning. The title row is the `TITLE_ROW` strip directly above the client rectangle, and it only
## counts if it is entirely within the usable area vertically (a bar half off the top cannot be
## grabbed). **The probe's independent oracle for the 64 px bar**; `resolve()` never calls it.
static func title_visible_px(rect: Rect2i, screens: Array) -> int:
	var best := 0
	var bar := Rect2i(rect.position.x, rect.position.y - TITLE_ROW, rect.size.x, TITLE_ROW)
	for s in screens:
		var u: Rect2i = s["usable"]
		if bar.position.y < u.position.y or bar.end.y > u.end.y:
			continue
		var lo := maxi(bar.position.x, u.position.x)
		var hi := mini(bar.end.x, u.end.x)
		best = maxi(best, hi - lo)
	return best

## The screens that exist right now, in `DisplayServer` order: the shape `resolve()` takes.
## Empty when `DisplayServer` reports none (the "0 screens" case).
static func screens_now() -> Array:
	var out: Array = []
	for i in DisplayServer.get_screen_count():
		out.append({
			"rect": Rect2i(DisplayServer.screen_get_position(i), DisplayServer.screen_get_size(i)),
			"usable": DisplayServer.screen_get_usable_rect(i),
		})
	return out

## The geometry of a NATIVE window as `DccSettings.set_popout_geometry()` stores it, or `{}` for an
## embedded one: an embedded window's `position` is relative to the main window and means nothing
## on a screen, so storing it would put the next native window at a meaningless place.
static func capture(win: Window) -> Dictionary:
	if win.is_embedded():
		return {}
	var s := win.current_screen
	var sr := Rect2i(DisplayServer.screen_get_position(s), DisplayServer.screen_get_size(s))
	if sr.size.x <= 0 or sr.size.y <= 0 or win.size.x <= 0 or win.size.y <= 0:
		return {}
	return {
		"x": win.position.x, "y": win.position.y, "w": win.size.x, "h": win.size.y,
		"sx": sr.position.x, "sy": sr.position.y, "sw": sr.size.x, "sh": sr.size.y,
	}
