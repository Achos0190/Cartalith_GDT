extends AcceptDialog

## PC-8 (`PC_SHELL_PROPOSAL.md` §3.7, milestone PC-8; Ruling DY PQ4) and PW-4 (`POPOUT_WINDOWS_SCOPE.md`,
## milestone PW-4; Ruling DK): the frame the Journey planner draws in on the desktop.
##
## The owner, verbatim (Ruling DY, PQ4): *"Planner should open in a separate window that doesn't block the
## map (that way a user could put it on a second screen for example)"*. Until PC-8 the planner was a view
## swap (`journey_planner_view.gd::_show()` hid CIVIL's dock and the live map and put its own two panels in
## their place). On the desktop it is now this window, and the map, both docks and the timeline strip stay
## exactly where they were.
##
## **What it is.** An empty frame with five named slots, built once by `JourneyPlannerView.setup()`, which
## then parents its own panels into them and keeps every rule about what they show:
##   - `toolbar`: the planner's row that used to sit on the tool-options bar (`_tool_options_journey()`:
##     the route picker, the party set-ups, the carriage readout, re-route…, the trim hint, save journey,
##     export table). Ruling DZ (§3.12) moves it here: the bar holds tool settings only.
##   - `stops_row`: Start here / Add as stop / End here from the selection (§3.7, flow F2).
##   - `left_slot`: the Journeys list and the party form (`_left_panel`), which lived in the left dock.
##   - `center_slot`: the route map, totals, profile, stops, inspector and stage matrix (`_center_panel`),
##     which lived where the map is.
##   - `results_body`: the results the right dock's appended Journey section drew (`build_results()`):
##     the verdict, time, load, supply, resupply, cost, risk, vessels and the calculation trace.
##   - `timeline_slot`: the journey's day band, which borrowed the main window's `timeline_row` (JP-13).
##
## **Why an `AcceptDialog`.** `PopoutHost.add_toggle_button()` takes only one, and Window ▸ Open windows
## (`menus.gd::_visible_windows()`) walks only those (`PC_SHELL_PROPOSAL.md` §5, the Planner row).
##
## **What it must never do.**
## - Block the map. `ContextWindow.make_context()` clears `exclusive` (an `AcceptDialog` is born exclusive)
##   and carries the two relays PC-7 measured are needed: a focused embedded window takes every mouse event
##   and every key, so a non-left press over the map is handed back to it and an unused Ctrl/Alt chord is
##   run on the menu bar (`context_window.gd`, `context_key_relay.gd`).
## - Exist on the tablet or the phone. They keep the shipped swap (§3.7: "The tablet and the phone keep the
##   planner as `MOBILE_SHELL_PROPOSAL.md` draws it"); `JourneyPlannerView` builds this only when
##   `DccApp.planner_in_window()` is true.
## - Stay clickable during a generate. As an embedded window the busy overlay mutes it with every other
##   embedded window (`busy_overlay.gd::_mute_subwindows`); popped out, it is in `PopoutHost.GROUP`, which
##   the overlay walks for native windows (`popout_host.gd::native_windows()`).
## - Treat a Pop out / Dock move as a close. `toggle()` hides and re-shows the same node; the view's close
##   handler asks `PopoutHost.is_moving()` first (`window_closed`).
##
## Preloaded by `journey_planner_view.gd`, no `class_name`, so adding it needs no `--headless --import`
## pass (CLAUDE.md: that pass strips `project.godot`'s comments). Its file name is its `PopoutHost` key,
## `planner_window`, which is in `PopoutHost.LIVE_KEYS` (PW-4).

const ContextWindow := preload("res://shell/context_window.gd")

## Emitted when the window is hidden by the user (Close, Escape, the title bar's ×, a `hide()`), never for
## the hide inside a Pop out / Dock move. `JourneyPlannerView` answers it by marking itself closed.
signal window_closed

## The shell; the `PopoutKeyForwarder` reads it to carry Ctrl+Z / Ctrl+Y / Ctrl+S out of a pop-out.
var app = null

## The planner's own toolbar row (see the header). `JourneyPlannerView._tool_options_journey()` clears and
## refills it.
var toolbar: HBoxContainer
## The Start here / Add as stop / End here row; `JourneyPlannerView` clears and refills it.
var stops_row: HBoxContainer
## Holds `_left_panel` inside a vertical scroller (the left dock's own scroller did this before PC-8).
var left_slot: VBoxContainer
## Holds `_center_panel`, which sizes itself to fill it.
var center_slot: Control
## The results column's body; `JourneyPlannerView._refresh_results()` clears it and calls
## `build_results()` on it.
var results_body: VBoxContainer
## The day band's row; `JourneyPlannerView._rebuild_timeline_band()` clears and refills it.
var timeline_slot: HBoxContainer

## Width of the left (party form) and right (results) columns, in pixels. A labelled judgement: the left
## dock's `LAPTOP` band is 330 px wide (`ENV:1819`, the figure `_build_left_panel()`'s note measures the
## party form against), and the results drew in the right dock at about the same width. Both are minimums;
## the centre column takes the rest.
const SIDE_COLUMN_W := 330
## The window's authored size and floor, in pixels. A labelled judgement: three columns of the widths above
## plus a centre of about 560 px, and the margins. The user's own size wins once saved
## (`DccWidgets._desktop_window_size`, per window key). `MIN_SIZE` is only the floor's least value: the
## real floor is the content's own minimum width (`_fit_floor()`), which is larger whenever the party form
## is (measured 466 px for the form at seed 4242, against the 330 px column, by the PC-8 probe's leg L).
const AUTHORED_SIZE := Vector2i(1280, 800)
const MIN_SIZE := Vector2i(1000, 560)
## The centre column's least width, in pixels. A labelled judgement: the route map and the profile stay
## legible at 400; below it the stops strip clips.
const CENTER_MIN_W := 400

## The frame's root column, whose minimum width `_fit_floor()` reads.
var _outer: VBoxContainer

## Builds the frame and makes it the desktop's non-blocking, poppable window. `host` is the `DccApp`.
## Call once, after `add_child()`: `DccWidgets.phone_window()` reads the tree for the saved size.
func setup(host) -> void:
	app = host
	name = "PlannerWindow"
	title = "Journey planner"
	size = AUTHORED_SIZE
	min_size = MIN_SIZE
	## Sizes the window, adds the resize grip, and runs `PopoutHost.prepare()` (the hook every tool window
	## already passes through). Returns false on the desktop, the only form this window exists on.
	DccWidgets.phone_window(self, host)
	var outer := VBoxContainer.new()
	_outer = outer
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)
	toolbar = HBoxContainer.new()
	toolbar.name = "PlannerToolbar"
	toolbar.add_theme_constant_override("separation", 8)
	DccWidgets.pad(outer, 10, 6, 10, 6).add_child(_row_scroll(toolbar, "PlannerToolbarScroll"))
	outer.add_child(DccTheme.rule())
	stops_row = HBoxContainer.new()
	stops_row.name = "PlannerStopsRow"
	stops_row.add_theme_constant_override("separation", 8)
	DccWidgets.pad(outer, 10, 4, 10, 4).add_child(_row_scroll(stops_row, "PlannerStopsScroll"))
	outer.add_child(DccTheme.rule())
	var body := HBoxContainer.new()
	body.name = "PlannerBody"
	body.add_theme_constant_override("separation", 0)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(body)
	left_slot = VBoxContainer.new()
	left_slot.name = "PlannerLeft"
	left_slot.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(_column(left_slot, "PlannerLeftScroll"))
	center_slot = Control.new()
	center_slot.name = "PlannerCenter"
	center_slot.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	center_slot.size_flags_vertical = Control.SIZE_EXPAND_FILL
	center_slot.clip_contents = true
	center_slot.custom_minimum_size.x = CENTER_MIN_W
	body.add_child(center_slot)
	results_body = VBoxContainer.new()
	results_body.name = "PlannerResults"
	results_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	results_body.add_theme_constant_override("separation", 0)
	var results_pad := MarginContainer.new()
	for side in ["left", "right", "top", "bottom"]:
		results_pad.add_theme_constant_override("margin_" + side, 8)
	results_pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	results_pad.add_child(results_body)
	body.add_child(_column(results_pad, "PlannerResultsScroll"))
	outer.add_child(DccTheme.rule())
	timeline_slot = HBoxContainer.new()
	timeline_slot.name = "PlannerTimeline"
	timeline_slot.add_theme_constant_override("separation", 8)
	timeline_slot.custom_minimum_size.y = 24
	DccWidgets.pad(outer, 10, 4, 10, 4).add_child(timeline_slot)
	## Ruling DK Q2: the per-window Pop out / Dock button, shown even with the global setting off (PW-4).
	PopoutHost.add_toggle_button(self)
	## PC-7's rules: not exclusive, the map-press relay and the menu-chord relay. No FOLLOW chip: the
	## planner reads the selection for its stop buttons, but what it shows is a journey, not a subject.
	ContextWindow.make_context(self, host, false)
	visibility_changed.connect(func() -> void:
		if not visible and not PopoutHost.is_moving(self):
			window_closed.emit())
	outer.minimum_size_changed.connect(_fit_floor)

## Shows the window: re-decides docked or popped out first (`PopoutHost.prepare`, the line `popout_host.gd`'s
## header asks before every `popup_centered()`), centred on the first open, then where the user left it.
## A second call while it is visible only raises it.
func open() -> void:
	if visible:
		grab_focus()
		return
	PopoutHost.prepare(self)
	_fit_floor()
	if has_meta("_planner_placed"):
		## `show()`, not `popup()`: measured in the PC-8 probe (leg C), `popup()` moved a reopened window
		## that was fully on screen by 22 px (384 -> 362), so it did not reopen where the user left it.
		show()
		_keep_reachable()
	else:
		set_meta("_planner_placed", true)
		popup_centered()

## Puts an embedded window's title row back inside the main window when a reopen finds it out of reach
## (the main window was made smaller since, say): at least `REACHABLE_PX` of the title row stays inside on
## the left and right, and the title row is never above the top. PW-1's clamp rule
## (`POPOUT_WINDOWS_SCOPE.md` §9, decision 3: 64 px of the title row visible) for this window's own reopen;
## a native window is placed by `PopoutHost`'s geometry memory instead and is left alone here.
func _keep_reachable() -> void:
	if not is_embedded() or get_parent() == null:
		return
	var vp := Vector2i(get_tree().root.get_visible_rect().size)
	var title_h := get_theme_constant("title_height", "Window")
	var p := position
	p.x = clampi(p.x, REACHABLE_PX - size.x, maxi(0, vp.x - REACHABLE_PX))
	p.y = clampi(p.y, title_h, maxi(title_h, vp.y - REACHABLE_PX))
	position = p

## Least run of the title row kept inside the main window by `_keep_reachable()`, in pixels: PW-1's figure.
const REACHABLE_PX := 64

## Keeps the window at least as wide as its content's minimum, so no column is ever cut off at its right
## edge. Needed because `DccWidgets.phone_window()` turns `wrap_controls` off (a window that grows to fit
## its content walks off the screen), and with it off a window narrower than its content clips the
## content: measured in the PC-8 probe's leg L, the results column started at x 866 of a 1000 px window.
## The two one-line rows scroll sideways (`_row_scroll`) and the columns scroll down, so only the columns'
## widths count. Never shrinks a window the user made wider.
func _fit_floor() -> void:
	if _outer == null:
		return
	## The dialog's own answer, which adds its panel margins around the content (`AcceptDialog`'s
	## `_get_contents_minimum_size`), not the content's bare minimum.
	var need := int(ceil(get_contents_minimum_size().x))
	min_size.x = maxi(MIN_SIZE.x, need)
	if size.x < min_size.x:
		size.x = min_size.x

## A one-line row that scrolls sideways instead of widening the window. Measured (the PC-8 probe's
## desktop capture at the 1000 px floor): the planner's toolbar row is wider than the window, and a plain
## row's minimum width is folded into every ancestor, so the whole body grew past the window's right edge
## and the results column was cut off. A scroller with its horizontal axis ENABLED reports no minimum
## width on that axis (`MISTAKES.md`, "Read a layout that overflows the screen"), so the row is clipped
## and scrolls, and the columns below keep the window's width.
func _row_scroll(row: HBoxContainer, scroll_name: String) -> ScrollContainer:
	var sc := ScrollContainer.new()
	sc.name = scroll_name
	sc.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	sc.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	sc.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sc.add_child(row)
	return sc

## A fixed-minimum-width side column: `content` inside a vertical-only scroller, so a long party form or
## results list scrolls inside its own column and never widens the window (`MISTAKES.md`, "Read a layout
## that overflows the screen": a scroller with its horizontal axis disabled folds its child's width in, so
## the content's own width is bounded by `SIDE_COLUMN_W` on the column, not left free).
func _column(content: Control, scroll_name: String) -> ScrollContainer:
	var sc := ScrollContainer.new()
	sc.name = scroll_name
	sc.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	sc.custom_minimum_size.x = SIDE_COLUMN_W
	sc.size_flags_vertical = Control.SIZE_EXPAND_FILL
	sc.add_child(content)
	return sc
