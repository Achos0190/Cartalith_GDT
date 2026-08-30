extends Control
class_name DccShell

## The DCC editor frame (`DCC_SHELL_SPEC.md` §1-§3, §6, §7, §9, §11).
##
## Six regions in DOM order plus the two bars that bracket them, built in code
## rather than in a `.tscn` so that the geometry table in §1 is readable as a
## table here, and so five workspace modules can attach without five people
## editing one scene file.
##
## This script owns the *frame* only: region sizes, dock collapse, which
## workspace is active, and the status bar. It owns no world state and calls no
## engine method -- `EngineBridge` does that, and the workspaces read it. The
## load-bearing rule from `UI_SHELL_DESIGN.md`: the top bar is about the
## program, the map is about the world.

signal workspace_changed(id: String)
signal tool_changed(tool_id: String)
signal phone_insets_changed()  ## §13: fires whenever a rotation changes where
	## the phone chrome's edges sit, so `ViewportHost`'s own corner chrome
	## (built and owned by `app.gd`, not this file) can re-read `phone_content_insets()`.

# -- Workspaces (§3) ----------------------------------------------------------
#
# Five domains on the rail. Generate / Simulate / Render / View are *not*
# menus in this shell -- that is the structural change this revision makes, and
# the reason the menu bar below is seven program menus and nothing else.

## A fourth key, `subnodes`, carried each domain's dock sub-structure for
## SH-01's expanded rail. **Removed 2026-08-24 with the expansion itself** --
## see `_build_rail()`'s header for why the rail no longer has two states, and
## therefore nothing left to list.
##
## **Domain merge (2026-08-20, owner instruction: "Infra can be dropped as a
## name and can be absorbed by civil... And render into carto.")** Five
## domains become three. INFRA's five subjects (Roads/Rivers/Ports/Trade/
## Logistics) and its Way/Route tools now live under CIVIL, via
## `civilization_workspace.gd` composing an `InfrastructureWorkspace` instance
## into its own dock rather than that class getting its own rail button.
## RENDER's one subject (Terrain appearance) now lives under CARTO the same
## way, via `cartography_workspace.gd` composing a `RenderWorkspace` instance.
## Nothing was deleted -- both classes still exist, still own their own
## category builders and tool click handlers, they are just reached through a
## different rail button now. See `DCC_SHELL_SPEC.md`'s own correction notice
## for the full disclosure and `GUI_GAP_REGISTER.md` §6.11-§6.14.
const DOMAINS: Array = [
	{"id": "world", "label": "World", "rail": "WORLD", "icon": "domain_world",
		"subtitle": "Terrain, hydrology, climate and ecology"},
	{"id": "civilization", "label": "Civilization", "rail": "CIVIL", "icon": "domain_civ",
		"subtitle": "Settlements, factions, provinces, trade, roads, sea routes and journeys"},
	{"id": "cartography", "label": "Cartography", "rail": "CARTO", "icon": "domain_carto",
		"subtitle": "Layers, styles, labels, annotation and terrain appearance"},
]

# -- Region handles -----------------------------------------------------------
#
# Everything a workspace module needs is reachable from here. Workspaces never
# reach past these into the frame's own containers.

var menu_bar_row: HBoxContainer
var tool_options_row: HBoxContainer
var rail_column: VBoxContainer
var left_dock: PanelContainer
var left_dock_title: Label
var left_dock_body: VBoxContainer      ## Workspace panels attach here.
var viewport_area: Control
var viewport_content: Control          ## The map surface; overlays are children.
var right_dock: PanelContainer
var right_dock_title: Label          ## §6: contents follow the selection, not a fixed
	## "Layers" chrome label -- kept live by `right_dock.gd`'s own `set_right_dock_title()` call at the end of every `_rebuild()`.
var right_dock_body: VBoxContainer
var timeline_bar: Control
var timeline_row: HBoxContainer
var status_row: HBoxContainer

var rail_foot: Label
var _domain_buttons: Dictionary = {}   ## id -> Button
var _domain_marks: Dictionary = {}     ## id -> {icon, label}
var _active_domain := "world"
var _left_collapsed := false
var _right_collapsed := false
var _left_width := float(DccTheme.W_LEFT_DOCK)
var _right_width := float(DccTheme.W_RIGHT_DOCK)
var _status_labels: Dictionary = {}    ## slot -> Label
var _collapse_buttons: Dictionary = {} ## "left"/"right" -> Button, so the chevron can flip
var _dock_readouts: Dictionary = {}    ## "left"/"right" -> the collapsed-state Label
var _workspace_panels: Dictionary = {} ## domain id -> Control
var _touch := false

# -- Domain rail ---------------------------------------------------------------
## What `Window ▸ Domain rail` hides. Not the same node in both compositions,
## which is the whole reason it is a named field rather than a walk up from
## `rail_column`: on desktop it is the rail panel itself (exactly what
## `rail_column.get_parent().get_parent()` used to reach), but on the phone the
## domains are three cells of the L1 bottom bar, and hiding the bar would take
## the MENU cell with it -- the only route back to the row that un-hides it.
## See `_build_phone_menu_bar()`.
var _rail_region: Control

# -- WI-04 dock width dragging --------------------------------------------------
var _dragging_dock := ""  ## "", "left" or "right" -- which handle (if any) owns the current drag.

# -- Phone layout (§13) --------------------------------------------------------
#
# `_touch` alone can't tell a tablet from a phone -- both are touch devices.
# The discriminator is the screen's own aspect: `min(w,h)/max(w,h)` is
# order-independent, so it survives rotation without flip-flopping between the
# two compositions (a phone rotated to landscape is still ~0.46, a tablet
# rotated to portrait is still ~0.625). `_phone` is decided once, at boot,
# because a device's form factor never changes at runtime; `_landscape` is
# re-decided on every resize, because rotation genuinely does, and §13 asks
# for a distinct landscape treatment.
const _PHONE_ASPECT_MAX := 0.6  ## Midpoint-ish between 19.5:9 (~0.46) phones
	## and 16:10 (~0.625) tablets -- every common handset aspect sits under it.
var _phone := false
var _landscape := false
var _phone_scale := 1.0  ## Maps `DccTheme.PHONE_REF_SHORT` phone-px onto the
	## real device's short side. Clamped to >= 1.0: the mockup's own numbers
	## already clear the 44 px floor at scale 1, so this only ever scales up,
	## never shrinks a target below spec.

# Phone region handles, built only when `_phone` is true.
var _phone_root: Control
var _phone_top_safe: Control
var _phone_side_safe: Control
var _phone_chrome_margin: MarginContainer  ## Shifts right in landscape so the
	## rail and app bar clear `_phone_side_safe` -- "the domain rail shifts
	## inward" (Phone inset rules, LANDSCAPE).
var _phone_content_gap: Control            ## Hosts the floating rail; its own
	## rect is the visible gap between the app bar and the tool sheet.
var _phone_tool_sheet: PanelContainer
var _phone_app_bar: PanelContainer  ## Held so a probe can measure it and so
	## `phone_content_insets()` reads its real height rather than recomputing it.
var _phone_gesture_inset: Control

# -- Phone bottom-sheet detents ------------------------------------------------
#
# `docs/ANDROID_UI_SPEC.md`, Locked decisions: *"Sheets: peek → half → full
# detents, drag handle; tab tap opens half"* and *"bar stays visible at full
# sheet"*.
#
# The three heights are the interactive prototype's own arithmetic, not derived
# ones -- `design/android-2026-08-30/Cartalith Android.dc.html`, `_detH()`:
#
#     fh   = frameH - 84            (portrait; the whole frame in landscape)
#     peek = 66                     a constant, not a fraction of anything
#     half = round(fh * 0.46)
#     full = fh - 96                so 96 dp of map is never covered
#
# That `84` is the prototype's `navH`, which is exactly this shell's bottom bar
# (`H_PHONE_BOTTOM_NAV` 64) plus its gesture inset (`H_PHONE_GESTURE` 20) -- the
# same figure reached from two constants instead of one literal.
# `_phone_nav_reserve()` reads it live rather than hard-coding 84, because this
# shell parks the timeline between the sheet and the bar and the prototype has
# no equivalent of that row.
#
# **"Bar stays visible at full sheet" is structural here, not arithmetic.** The
# sheet is a *sibling above* the bottom bar in the chrome column, so no detent
# height can put it over the bar; the prototype has to position the sheet at
# `bottom:${navH}px` to get the same result out of absolute positioning.
const PHONE_DETENT_PEEK := 66.0        ## `_detH`: `det==='peek'?66`.
const PHONE_DETENT_HALF_FRAC := 0.46   ## `Math.round(fh*0.46)`.
const PHONE_DETENT_FULL_GAP := 96.0    ## `full=fh-96`.
const PHONE_DETENT_ANIM := 0.28        ## `transition:height .28s cubic-bezier(.3,.9,.3,1)`.
	## Godot has no cubic-bezier easing; `TRANS_CUBIC`/`EASE_OUT` is the curve
	## that control point set approximates -- named as an approximation rather
	## than passed off as the mockup value.
const PHONE_DETENT_MIN_DRAG := 40.0    ## `_sm`: `Math.max(40, ...)`.
const PHONE_DETENT_DISMISS := 44.0     ## `_su`: `if(h<44)` closes the sheet.
## The prototype boots at `detent:'half'` because ITS sheet carries the tab's
## whole panel. This one carries the tool options row and nothing else (§13:
## "tool options become a bottom sheet"), so half opens a sheet that is mostly
## empty -- measured on the handset: one row of chips above roughly 700 px of
## nothing. Boot at `peek`, which is that content's own height, and leave half
## and full reachable by dragging the handle.
##
## This is a deliberate divergence from the prototype and it is the CONTENT
## that differs, not the geometry: the detent sizes below are the prototype's
## own numbers, unchanged.
var _phone_detent := "peek"
var _phone_sheet_grab: Control         ## The drag handle's own hit area.
var _phone_sheet_drag := {}            ## `{"y0","h0","h"}` while a drag is live; empty otherwise.
var _phone_sheet_tween: Tween          ## Held so a new snap kills the running one.

# -- Phone landscape: left rail + right-docked sheet ---------------------------
#
# `docs/ANDROID_UI_SPEC.md`: *"Landscape: nav becomes left rail, sheet docks
# right, map stays wide."* Both are the *same nodes* rotated into new positions
# rather than a second set built alongside -- `_apply_phone_orientation()`
# reparents them between the chrome column and `_phone_root`, and flips the
# bar's own box containers with `BoxContainer.vertical`.
var _phone_chrome_col: VBoxContainer   ## The portrait stack the two nodes above return to.
var _phone_bar_row: BoxContainer       ## The bottom bar's three nested boxes, held
var _phone_bar_domains: BoxContainer   ## only so `vertical` can be flipped on them.
var _phone_bar_cells: BoxContainer
const W_PHONE_LAND_RAIL := 72.0   ## Prototype, landscape branch: `width:72px;
	## background:var(--pan2);border-right:1px solid var(--hair2)`, and the map
	## host moves to `left:72px`.
const W_PHONE_LAND_SHEET_MAX := 440.0  ## `Math.min(440,Math.round(fw*0.46))`.
const PHONE_LAND_SHEET_FRAC := 0.46

var _phone_clock_label: Label
var _phone_battery_label: Label
var _phone_side_clock_label: Label     ## Landscape's rotated-pocket twins of
var _phone_side_battery_label: Label   ## the two above -- see `_build_phone_side_safe()`.
var _phone_panel_picker: Control
var _phone_bar_panels: Dictionary = {}  ## The bottom bar's two non-domain cells,
var _phone_bar_more: Dictionary = {}    ## held so `_refresh_phone_bar_lit()` can
	## light them; the three domain cells go through `_domain_marks` instead.
var _phone_menu_bar: Control    ## L1 of the phone disclosure tree -- the bottom
	## bar. Named handle because `_phone_bottom_reserve()` has to measure it.
var _phone_menu: PhoneMenu      ## L2-L5. Replaces the old `_phone_overflow`
	## sheet (`GUI_GAP_REGISTER.md` §15).
var _left_sheet_open := false
var _right_sheet_open := false
## The two dock `ScrollContainer`s, held only so `_set_sheet_open()` can zero
## their scroll on open -- neither dock exposes its `_scroll()` return value
## anywhere else, and a sheet's body (`left_dock_body`/`right_dock_body`) is
## never torn down between opens, so whatever scroll position was left from
## the previous open is still sitting on the node when it reopens.
var _left_dock_scroll: ScrollContainer
var _right_dock_scroll: ScrollContainer

# -- Build identity ------------------------------------------------------------

## One line in the boot log saying **which build this is**, and it exists
## because a device pass could not answer that question and drew the wrong
## conclusion from not being able to (`GUI_GAP_REGISTER.md` §56).
##
## The shell-side twin of `EngineBridge._has()`, which the 2026-08-24 pass added
## after a `.so` ran 21 commits behind its own shell in silence
## (`ANDROID_BUILD_SCOPE.md`). That guard speaks for the *native* half of the
## pair; nothing spoke for the GDScript half, so two APKs built forty minutes
## apart across a UI migration were indistinguishable on the handset except by
## looking at them -- and §54 recorded that difference as "the shell's chrome is
## not stable across boots", a startup race that does not exist.
##
## Hashes every file the project ships under `res://shell/`, name and content,
## in sorted order. Deliberately the whole directory rather than this one file:
## the two APKs §54 compared differed in `map_overlay.gd` alone in one pair and
## in `dcc_shell.gd` in the other, and a digest that only covers its own source
## would have missed the first. `map_overlay.gd` itself sits at the project root
## and is folded in by name for the same reason.
##
## Costs a few hundred KB of MD5 at boot, once, and cannot rot: there is no
## version constant for anyone to forget to bump. It changes when the shipped
## scripts change, which is exactly the question "is this the same build?"
##
## The digest is **not** comparable between an editor run and an export -- an
## export ships `.gdc` + `.gd.remap` where the editor has `.gd` + `.uid`. That is
## a feature: those genuinely are different builds of the same tree, and the two
## must never be mistaken for each other in a measurement log.
##
## And in an export it is a *behaviour* fingerprint rather than a source one,
## because what it hashes there is the compiled token stream. Measured, not
## assumed: appending one comment line to `shell/right_dock.gd` and re-exporting
## leaves `assets/shell/right_dock.gdc` **byte-identical** (`a8ee3535...`), while
## a one-line code change moves it (`ed7b699f...`). So two APKs differing only in
## comments carry the same id -- which is the right granularity for "is this the
## same build?" asked of a measurement, and the wrong one for "is this the same
## commit?". For the latter, hash the APK.
##
## It does not, and cannot, prove the *installed APK* is the one just built --
## only `sha256` against `adb shell pm path` does that. See §56's harness note.
static func build_id() -> String:
	var files: Array[String] = ["res://map_overlay.gd", "res://map_overlay.gdc"]
	var stack: Array[String] = ["res://shell"]
	while not stack.is_empty():
		var dir: String = stack.pop_back()
		for d in DirAccess.get_directories_at(dir):
			stack.append(dir.path_join(d))
		for f in DirAccess.get_files_at(dir):
			files.append(dir.path_join(f))
	files.sort()
	var ctx := HashingContext.new()
	ctx.start(HashingContext.HASH_MD5)
	for f in files:
		## `get_md5()` returns "" for a path that is not there, which is the
		## normal case for two of the three root entries above -- folding the
		## empty string in under its own name keeps the digest defined either way.
		ctx.update((f + ":" + FileAccess.get_md5(f)).to_utf8_buffer())
	return ctx.finish().hex_encode().substr(0, 12)

# -- Build --------------------------------------------------------------------

func _ready() -> void:
	## Before anything else, so it is the first Cartalith line in `logcat` and
	## survives a boot that fails after it. See `build_id()` above for why a
	## device pass needs it.
	print("Cartalith shell build ", build_id())
	## `--force-touch`: a testing-only override, same pattern as `_shot.gd`'s
	## own `--generate` flag. Real touch hardware is never present in this
	## dev/CI environment, so without it the phone/tablet composition below is
	## simply unreachable from `--resolution WxH` alone -- there is no device-
	## preview loop, and `--resolution` is otherwise the only lever the
	## verification harness has to exercise §13 at all.
	_touch = (DisplayServer.is_touchscreen_available() and OS.has_feature("mobile")) \
		or "--force-touch" in OS.get_cmdline_user_args()
	## Published for the static widget factories -- `DccWidgets.style_popup()`
	## sizes a menu off it and has no node to reach this one through.
	DccTheme.set_touch(_touch)
	set_anchors_preset(Control.PRESET_FULL_RECT)
	var ground := ColorRect.new()
	ground.color = DccTheme.c("bg")
	ground.set_anchors_preset(Control.PRESET_FULL_RECT)
	ground.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(ground)

	## Phone-vs-tablet is decided once, here, off the boot window size -- a
	## device's own form factor is not something that changes at runtime.
	## Orientation (`_landscape`) *is* re-decided on every resize below, which
	## is the half of §13 that genuinely needs to react live.
	_compute_layout_mode()
	## The narrower half of the same publication -- the 412 canvas asks for
	## things a tablet must not get, and a static factory has no way to tell the
	## two apart from `is_touch()` alone. See `DccTheme.is_phone()`.
	DccTheme.set_phone(_phone)
	_style_window_chrome()

	## Hand the Android back gesture to `_notification()` below instead of
	## letting the SceneTree quit on it -- there are levels to pop first
	## (`design/Cartalith Android Phone.dc.html`, PHONE RULES / BACK), and past
	## the last of them an unsaved world to protect.
	##
	## Set for EVERY layout mode, not only the phone. It was phone-only when
	## first written, which left every Android device the aspect test classifies
	## as a *tablet* taking the SceneTree default -- back quits, at once, with
	## no prompt. `quit_on_go_back` is inert on desktop, where no windowing
	## system ever sends the request, so there is nothing to guard it with.
	get_tree().quit_on_go_back = false
	## The desktop half of the same guard: the title bar's ×, Alt+F4 and the
	## taskbar's Close all arrive as `NOTIFICATION_WM_CLOSE_REQUEST`, and with
	## `auto_accept_quit` at its default the SceneTree ends the process on them
	## before any of this shell's code runs -- an unsaved world destroyed by one
	## click, the exact fault the back gesture had on Android.
	##
	## Turning it off means NOTHING quits the app unless our code asks, so
	## `_close_requested()` below carries the obligation to always resolve. See
	## `DccApp._close_requested()` for the proof that it cannot trap the user.
	get_tree().auto_accept_quit = false
	if _phone:
		_build_phone_shell()
	else:
		_build_desktop_shell()

	get_tree().root.size_changed.connect(_on_window_resized)
	_select_domain(_active_domain)

## The pointer-first / tablet composition: one continuous vertical stack of
## fixed-height bars around a horizontal row of rail/docks/viewport. This is
## the shell as it existed before phone support -- `_scaled()` already makes
## it tablet-safe, so nothing here is phone-aware.
func _build_desktop_shell() -> void:
	var shell := VBoxContainer.new()
	shell.set_anchors_preset(Control.PRESET_FULL_RECT)
	shell.add_theme_constant_override("separation", 0)
	add_child(shell)

	shell.add_child(_build_menu_bar())
	shell.add_child(_build_tool_options_bar())

	var main_row := HBoxContainer.new()
	main_row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_row.add_theme_constant_override("separation", 0)
	shell.add_child(main_row)

	main_row.add_child(_build_rail())
	main_row.add_child(_build_left_dock())
	main_row.add_child(_build_viewport())
	main_row.add_child(_build_right_dock())

	timeline_bar = _build_timeline()
	shell.add_child(timeline_bar)
	shell.add_child(_build_status_bar())

func _scaled(px: int) -> int:
	## §13: tablet scales every fixed height, with a 44 px floor on anything
	## tappable. Windows is pointer-first and takes the raw value. Phone does
	## not use this at all for its own chrome -- see `_pscale()`/`_ptap()` --
	## but still calls it for the desktop-shaped bars (menu bar, status bar)
	## that phone relocates into the ⋯ overflow sheet unmodified.
	if not _touch:
		return px
	## `DccTheme.TABLET` first: §1's tablet column is a table of exact figures,
	## not a multiplier, and the multiplier-plus-floor this used to be got two
	## of the five wrong in opposite directions. The 44 px floor still applies
	## to anything the table does not name, because an unnamed figure is a
	## control rather than a region.
	if DccTheme.TABLET.has(px):
		return int(DccTheme.TABLET[px])
	return maxi(44, int(round(px * DccTheme.TOUCH_SCALE)))

# -- §13 Phone layout mode ------------------------------------------------

## Order-independent aspect: see the field comment on `_phone` above for why.
func _compute_layout_mode() -> void:
	var size: Vector2 = get_viewport_rect().size
	if size.x <= 0.0 or size.y <= 0.0:
		return
	var short_side: float = minf(size.x, size.y)
	var long_side: float = maxf(size.x, size.y)
	_phone = _touch and (short_side / long_side) < _PHONE_ASPECT_MAX
	_landscape = size.x > size.y
	_phone_scale = maxf(1.0, short_side / DccTheme.PHONE_REF_SHORT)
	## §1's tablet column widens BOTH docks to 400 px, "so two-column readouts
	## survive the larger type" (`UI_SHELL_DESIGN.md`). Neither dock had ever
	## been told that: tablet ran the desktop 372/300 pair.
	if _touch and not _phone:
		_left_width = float(DccTheme.W_DOCK_TABLET)
		_right_width = float(DccTheme.W_DOCK_TABLET)

## **The early return is deliberate, and it was audited rather than assumed**
## (`GUI_GAP_REGISTER.md` §56). It reads as an asymmetry -- a shell that latched
## tablet by mistake can never correct itself -- and the obvious fix, hoisting
## `_compute_layout_mode()` above the guard, is the wrong one twice over:
##
## - There is nothing to correct. `_phone` needs `_touch`, which is fixed for
##   the life of the process, and the aspect it tests is `min/max`, which a
##   rotation cannot change. Measured on the OnePlus 6T over 52 cold starts:
##   `get_viewport_rect()` reports the real 1080 x 2340 at the first sample
##   inside `_ready()` and `root.size_changed` never fires at all. There is no
##   provisional size for the decision to race against on this handset.
## - Recomputing for tablets would *break* something real. The tablet branch of
##   `_compute_layout_mode()` assigns `_left_width`/`_right_width`, so running
##   it on every resize would reset a tablet user's dragged dock widths (WI-04)
##   to `W_DOCK_TABLET` on every rotation.
func _on_window_resized() -> void:
	if not _phone:
		return  ## Tablet/desktop windows resizing is not this shell's concern.
	var was_landscape := _landscape
	_compute_layout_mode()
	if _landscape != was_landscape:
		_apply_phone_orientation()

## Phone-only geometry: scaled off the real device's short side, no 44 px
## floor -- for chrome that is deliberately *not* tappable (the top safe area,
## the gesture inset) a floor would be wrong.
func _pscale(px: float) -> int:
	return maxi(1, int(round(px * _phone_scale)))

## Phone-only geometry for anything tappable: §13's floor, no exceptions.
func _ptap(px: float) -> int:
	return maxi(DccTheme.PHONE_TAP_MIN, _pscale(px))

# -- §2 Menu bar: program scope only ------------------------------------------

func _build_menu_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_MENU_BAR)
	bar.add_theme_stylebox_override("panel",
		DccTheme.panel("panel", {"bottom": 1}))

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(row)
	bar.add_child(pad)

	var wordmark := DccTheme.mono_label("CARTALITH", "text_bright", DccTheme.FS_MENU, 3, true)
	row.add_child(wordmark)
	## `margin-right:22px` on the canvas's own wordmark, and nothing else --
	## the reserved 150 px this used to claim opened a 74 px hole between
	## CARTALITH and File where the canvas has 22 px, which is the first thing
	## the eye lands on and the first thing that read as "not the design".
	var wordmark_gap := Control.new()
	wordmark_gap.custom_minimum_size.x = 22
	row.add_child(wordmark_gap)

	menu_bar_row = HBoxContainer.new()
	menu_bar_row.add_theme_constant_override("separation", 0)
	row.add_child(menu_bar_row)

	row.add_child(DccTheme.spacer())

	## The readout cluster: world, pass state, and the three cost meters. §11
	## keeps these in the menu bar because they describe the *program's* load,
	## not the world's content.
	for slot in ["world", "res", "cpu", "gpu", "mem"]:
		var l := DccTheme.mono_label("", "text_faint", DccTheme.FS_READOUT, 1)
		_status_labels["top_" + slot] = l
		row.add_child(l)
		var gap := Control.new()
		gap.custom_minimum_size.x = 22
		row.add_child(gap)
	return bar

## Register a program menu. The caller fills the PopupMenu through `on_built`,
## so this file never has to know what File contains.
func add_menu(title: String, on_built: Callable) -> MenuButton:
	var mb := MenuButton.new()
	mb.text = title
	## **`flat` off, and this is not cosmetic.** `MenuButton` constructs itself
	## flat, and a flat `Button` skips its `normal`/`hover`/`pressed` styleboxes
	## outright -- the trap `viewport_host.gd` records paying for twice on the
	## Layers button. So the `pressed` override three lines down, which is the
	## canvas's own open-menu indicator
	## (`color:#e0a34a;background:rgba(224,163,74,.08);border-bottom:1px solid
	## #e0a34a`), had **never drawn**: sampled off the framebuffer with the File
	## menu open, the title's background was `#121314` (18,19,20) -- the panel
	## behind it -- identical to the closed `Edit` beside it. The most prominent
	## state cue in the application was invisible. `normal`/`hover` are
	## `StyleBoxEmpty` with the canvas's padding, so nothing else changes.
	mb.flat = false
	mb.focus_mode = Control.FOCUS_NONE
	## Prose face, not Plex -- `<span style="padding:9px 11px">File</span>` sits
	## inside a `font-size:11.5px;color:#a9adb0` run in the canvas, with no
	## font-family of its own, so it inherits `'Helvetica Neue'` from the
	## artboard root. No font override here at all is how a Control keeps
	## `dark_theme.tres`'s `default_font` (Fira Sans).
	##
	## The tablet column is `font-size:14px;padding:15px 15px` in
	## `DCC shell tablet 2560` -- measured, not scaled. Until 2026-08-25 this
	## row ignored `_touch` entirely, so a 2560x1600 tablet drew a 52 px menu
	## bar (§48 fixed the *bar*) carrying seven 40x51 desktop-sized titles in
	## 11 px type, none of which is a 44 px target in either dimension.
	var mfs := DccTheme.menu("fs_bar", _touch)
	var mpx := DccTheme.menu("bar_pad_x", _touch)
	var mpy := DccTheme.menu("bar_pad_y", _touch)
	mb.add_theme_font_size_override("font_size", mfs)
	mb.add_theme_color_override("font_color", DccTheme.c("text_secondary"))
	mb.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	mb.add_theme_stylebox_override("normal", DccTheme.inset(mpx, mpy, mpx, mpy))
	mb.add_theme_stylebox_override("hover", DccTheme.inset(mpx, mpy, mpx, mpy))
	mb.add_theme_stylebox_override("focus", DccTheme.inset(mpx, mpy, mpx, mpy))
	mb.add_theme_stylebox_override("disabled", DccTheme.inset(mpx, mpy, mpx, mpy))
	## `active_row()` is `accent_wash` (.08) plus a 1 px accent underline, which
	## is exactly what the canvas draws on an open title -- and the .08 here is
	## right, unlike the *item* inside the dropdown, which is .10. Two literals,
	## a few lines apart in the same artboard.
	var open_box := DccTheme.active_row()
	open_box.content_margin_left = mpx
	open_box.content_margin_right = mpx
	open_box.content_margin_top = mpy
	open_box.content_margin_bottom = mpy
	mb.add_theme_stylebox_override("pressed", open_box)
	mb.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	menu_bar_row.add_child(mb)
	var popup := mb.get_popup()
	style_popup(popup)
	on_built.call(popup)
	return mb

## The canvas's own menu panel. **The body moved to `DccWidgets.style_popup()`
## 2026-08-25** and this is now a delegate, kept because `menus.gd` and
## `phone_menu.gd` both call it by this name.
##
## Why it moved: `PopupMenu` is not only the seven program menus. Every
## `OptionButton` in the shell owns one too, and none of them had ever been
## styled -- the whole dropdown vocabulary of the application (every dock
## picker, every dialog select, the paint target, the bake depth) was opening
## Godot's stock dark theme, a `#0f0f0f` panel with a grey selection bar, in a
## shell whose palette is `#121314` plus one amber. `dropdown()` is a static
## factory with no shell to reach, so the styling had to become static too --
## see `DccTheme.is_touch()`, which exists for exactly this call.
func style_popup(popup: PopupMenu) -> void:
	DccWidgets.style_popup(popup)

# -- PR-13/PR-14 Theme rebuild --------------------------------------------------
#
# `DccTheme.apply_theme()` only re-points which palette `c()` resolves
# against; it repaints nothing, because every node that already called `c()`
# baked a plain `Color` value into its own `add_theme_*_override`, not a live
# reference. This is the other half: walk the whole tree and re-derive every
# one of those baked values from the token that produced it.
#
# Godot exposes no "list every override this node has" call, so the two
# arrays below are the exhaustive set of override *names* this codebase
# actually uses -- grepped, not guessed:
#   grep -rhoE 'add_theme_(color|stylebox)_override\("[^"]+"' godot-project/shell
# The *values* need no such list: `DccTheme.remap()` reverse-looks-up
# whichever token in the *old* palette produced the colour already sitting on
# a node, and repaints it with that same token's new value. A colour that
# matches no token (a literal, e.g. a phone overlay's plain black scrim) is
# left alone -- there is nothing to remap it to.
#
# This walks every node under the shell root, so it reaches workspace panels,
# popups and dialogs too, not just the frame chrome `DccShell` itself builds
# -- but only nodes that already exist. A dialog that has never been opened
# yet builds itself fresh from `DccTheme.c()` the first time it opens, which
# already picks up the new palette; nothing extra is needed for those.

## Re-grepped 2026-08-20 (owner: "make sure the lightmode version is available
## everywhere"). Six names had accumulated since the lists were first written
## and were therefore never repainted by a theme switch -- exactly the drift
## the comment above predicts, introduced by the windows built *after* the
## theme pass. `caret_color`/`font_placeholder_color`/`font_uneditable_color`
## and the `read_only`/`disabled`/`focus` styleboxes all come from
## `dcc_widgets.gd`'s text fields, which is why every dialog with a text well
## (the browse dialogs, the asset library, the data manager, this file's own
## search) kept dark input wells under the light palette.
const _THEME_COLOR_OVERRIDES := [
	"caret_color", "default_color", "font_accelerator_color", "font_color",
	"font_disabled_color", "font_hover_color", "font_placeholder_color",
	"font_pressed_color", "font_separator_color", "font_uneditable_color",
	"icon_hover_color", "icon_normal_color", "icon_pressed_color",
]
const _THEME_STYLEBOX_OVERRIDES := [
	"disabled", "focus", "grabber_area", "grabber_area_highlight", "hover", "normal",
	"panel", "pressed", "read_only", "separator", "slider",
]

## Called by `menus.gd` immediately after `DccTheme.apply_theme()`, passing
## whichever palette was active a moment ago (`was_dark`) so the walk knows
## what it's reversing.
func rebuild_theme(was_dark: bool) -> void:
	var old_pal: Dictionary = DccTheme.DARK if was_dark else DccTheme.LIGHT
	_recolor_project_theme(old_pal)
	_style_window_chrome()
	_recolor_subtree(self, old_pal)
	## The phone top scrim used to need a third pass here: its colour lived
	## inside a `GradientTexture2D`, on no node and in no theme resource, so
	## neither walk above could see it, and a light-palette capture found a
	## charcoal band over a light screen. The 412 canvas draws no scrim -- the
	## status row is a plain `panel` stylebox now, which `_recolor_subtree()`
	## reaches like every other region.

## The other half of "everywhere", found 2026-08-20 by capturing every window
## under the light palette instead of trusting the walk.
##
## `project.godot` sets `gui/theme/custom` to a real, hand-authored dark
## `Theme` resource, and that resource is the fallback for every control state
## nothing overrides explicitly -- disabled buttons, focus rings, scrollbars,
## tooltips, popup separators, `SpinBox`/`OptionButton`/`CheckBox` chrome. None
## of it is a per-node `add_theme_*_override`, so `_recolor_subtree()` below
## could never have reached any of it: the colours live inside a `Resource`,
## not on the nodes. That is why a disabled `DccWidgets.action()` button (which
## sets `normal`/`hover` but no `disabled` stylebox) stayed a dark slab on a
## light shell -- "Bake ALL & finalize" and the world workspace's own
## "Finalize · LOD 0-3", both visibly wrong in the light capture.
##
## Remapping works because that resource was authored from these exact tokens:
## its header lists surface `#0d0e0f`, text `#c8cbcd`, accent `#e0a34a` and the
## rest, which are `DccTheme.DARK`'s values verbatim. So the same reverse
## lookup the node walk uses converts the whole resource.
##
## Mutating it is in-memory only -- `load()` returns the cached instance the
## whole tree is already resolving against, so this repaints every fallback at
## once and nothing is written back to disk. Switching back re-runs it with the
## palettes swapped. A `StyleBoxFlat` shared by several entries is visited more
## than once, which is harmless: after the first visit its colour no longer
## matches anything in `old_pal`, so `remap()` returns null and leaves it be.
func _recolor_project_theme(old_pal: Dictionary) -> void:
	var path := String(ProjectSettings.get_setting("gui/theme/custom", ""))
	if path == "":
		return
	var th := load(path) as Theme
	if th == null:
		return
	var extras := _theme_extras(was_dark_to_light(old_pal))
	## See `_bulk_theme_edit()`: without this the walk costs 27 s on a phone.
	th.set_block_signals(true)
	for type_name in th.get_color_type_list():
		for color_name in th.get_color_list(type_name):
			var nc = _remap_theme_color(th.get_color(color_name, type_name), old_pal, extras)
			if nc != null:
				th.set_color(color_name, type_name, nc)
	for type_name in th.get_stylebox_type_list():
		for box_name in th.get_stylebox_list(type_name):
			var sb := th.get_stylebox(box_name, type_name)
			if sb is StyleBoxFlat:
				var f := sb as StyleBoxFlat
				var nb = _remap_theme_color(f.bg_color, old_pal, extras)
				if nb != null:
					f.bg_color = nb
				var nr = _remap_theme_color(f.border_color, old_pal, extras)
				if nr != null:
					f.border_color = nr
	_bulk_theme_edit(th)

## Ends a batch of `Theme` mutations: unblocks the resource's signals and fires
## `changed` exactly once.
##
## **Every `set_color()`/`set_stylebox()` on a live `Theme` emits `changed`, and
## that re-propagates `NOTIFICATION_THEME_CHANGED` to every `Control` in the
## tree.** Each edit therefore costs a whole-tree relayout, and this shell's
## tree is large -- the phone chrome, the parked desktop menu/status model, the
## dock sheets and every runtime-built window at once.
##
## Measured on the real OnePlus 6T before this batching, switching
## `Preferences ▸ Theme` to Light froze the main thread for **29.6 s**
## (`projectTheme=27336ms windowChrome=1597ms subtree=670ms`). The giveaway is
## `windowChrome`: it performs just **5** theme writes and still cost 1.6 s, or
## ~320 ms *per write* -- the cost is per mutation, not per colour examined. A
## first attempt at memoising the colour lookups was therefore wasted (27336 ->
## 27632 ms, i.e. no change) and was removed rather than kept as decoration.
##
## Found only because the phone menu made `Theme` reachable by finger for the
## first time. On desktop the same freeze exists but reads as a hitch on a
## machine that is ~50x faster per node; on the phone it looked like a **dead
## tap**, and was twice mistaken for a lost touch event before the log
## timestamps showed a 29.6 s round trip between press and repaint.
func _bulk_theme_edit(th: Theme) -> void:
	th.set_block_signals(false)
	th.emit_changed()

func was_dark_to_light(old_pal: Dictionary) -> bool:
	return old_pal == DccTheme.DARK

## Embedded `Window` chrome -- the title bar and its close button that every
## `AcceptDialog` in this shell draws above its own branded header. The project
## theme resource defines no `Window` entries at all, so that chrome came from
## Godot's stock built-in theme, which is dark and fixed: under the light
## palette every dialog wore a charcoal title bar over light content, and no
## amount of remapping could reach it because there was nothing to remap.
##
## Written from tokens rather than remapped, so it is correct on a cold boot in
## either palette as well as after a switch -- hence the call from `_ready()`
## as well as from `rebuild_theme()`.
func _style_window_chrome() -> void:
	var path := String(ProjectSettings.get_setting("gui/theme/custom", ""))
	if path == "":
		return
	var th := load(path) as Theme
	if th == null:
		return
	## Five writes, and each one used to cost ~320 ms on the phone -- see
	## `_bulk_theme_edit()`, which is where that number was measured.
	th.set_block_signals(true)
	th.set_color("title_color", "Window", DccTheme.c("text_bright"))
	th.set_color("title_outline_modulate", "Window", DccTheme.c("raised"))
	for box in ["embedded_border", "embedded_unfocused_border"]:
		th.set_stylebox(box, "Window", DccTheme.panel("raised",
			{"left": 1, "right": 1, "top": 1, "bottom": 1}))
	## `AcceptDialog` draws its *own* `panel` on top of the Window border, and
	## nothing here had ever set it -- so Performance, Gen info, World data and
	## the footer band of every modal came up on Godot's stock `#404040` grey,
	## sampled off the framebuffer 2026-08-25. `#404040` is not a token in
	## either palette and is 20 steps brighter than anything the canvas draws.
	## `AcceptDialog` inherits `PanelContainer`'s type only for *some* boxes,
	## which is why the shell's own `PanelContainer` styling never reached it.
	var dlg := DccTheme.panel("panel",
		{"left": 1, "right": 1, "top": 1, "bottom": 1})
	dlg.border_color = DccTheme.c("border")
	dlg.content_margin_left = 0
	dlg.content_margin_right = 0
	dlg.content_margin_top = 0
	dlg.content_margin_bottom = 0
	th.set_stylebox("panel", "AcceptDialog", dlg)
	th.set_stylebox("panel", "PopupPanel", dlg)

	## Tabs. `TabContainer` draws its strip from an internal `TabBar` that no
	## walk in this file reaches, so World data, Travel library, the Asset
	## library and the Data manager were all showing Godot's stock tab chrome:
	## a raised grey pill with a white top rule on the selected tab. The
	## design's own two-way switch -- the left dock's `GENERATION PIPELINE |
	## SCULPT` header -- is `background:rgba(224,163,74,.10)` with
	## `border-bottom:1px solid #e0a34a` when on and nothing at all when off,
	## which is `active_row()`, the same shape the menu bar and every active
	## dock row already use. Set on both type names because a bare `TabBar`
	## does not inherit `TabContainer`'s.
	for type_name in ["TabContainer", "TabBar"]:
		var on := DccTheme.active_row(true)
		on.content_margin_left = 14
		on.content_margin_right = 14
		on.content_margin_top = 7
		on.content_margin_bottom = 7
		th.set_stylebox("tab_selected", type_name, on)
		var off := DccTheme.flat(Color(0, 0, 0, 0))
		off.content_margin_left = 14
		off.content_margin_right = 14
		off.content_margin_top = 7
		off.content_margin_bottom = 7
		th.set_stylebox("tab_unselected", type_name, off)
		th.set_stylebox("tab_hovered", type_name,
			DccTheme.flat(DccTheme.c("line_soft")))
		th.set_stylebox("tab_focus", type_name, DccTheme.empty())
		th.set_color("font_selected_color", type_name, DccTheme.c("accent"))
		th.set_color("font_unselected_color", type_name, DccTheme.c("text_dim"))
		th.set_color("font_hovered_color", type_name, DccTheme.c("text_bright"))
	th.set_stylebox("panel", "TabContainer", DccTheme.panel("panel", {"top": 1}))
	th.set_stylebox("tabbar_background", "TabContainer", DccTheme.empty())
	_bulk_theme_edit(th)

## `DccTheme.remap()` first, then the supplementary table below for the
## handful of colours the theme resource uses that are not tokens at all.
func _remap_theme_color(value: Color, old_pal: Dictionary, extras: Dictionary) -> Variant:
	var key := value.to_html(false)
	if extras.has(key):
		return Color(extras[key] as Color, value.a)
	return DccTheme.remap(value, old_pal)

## The theme resource predates `DccTheme` and its header claims to use the same
## values; six of them do not, so the token reverse-lookup cannot see them and
## they stayed dark under the light palette. Measured by dumping every distinct
## `Color(...)` in the `.tres` and diffing against both palettes, not guessed.
##
## Each entry is a derivation, not a new colour invented here: two are plain
## surfaces, one is a token with a one-digit typo, and two are the accent with
## the same lighten/darken the widgets already apply to it
## (`DccWidgets.action()` uses `c("accent").lightened(0.1)`).
##
## Deliberately absent, because they are correct in both palettes and are not
## misses: `#1a1206`, the near-black used for text sitting *on* the amber slab,
## which must stay dark on a light ground too, and `#e66b6b`, the error red --
## the same reasoning `DccTheme` already applies to `warn`/`block`/`water`.
func _theme_extras(to_light: bool) -> Dictionary:
	var accent: Color = DccTheme.LIGHT["accent"] if to_light else DccTheme.DARK["accent"]
	var out := {
		## Intended as `text_dim`; the resource has 0x96 where the token has
		## 0x92 in blue, which is enough to defeat an exact-match lookup.
		"8d9396": "text_dim",
		"131416": "panel",    ## The panel surface behind field chrome.
		"1a1b1d": "sunken",   ## Input well, resting.
		"252729": "raised",   ## Input well, hover.
	}
	var pal: Dictionary = DccTheme.LIGHT if to_light else DccTheme.DARK
	var map := {}
	for hex in out:
		map[hex] = pal[out[hex]] as Color
	map["c48c38"] = accent.darkened(0.125)   ## Accent, pressed.
	map["edb45f"] = accent.lightened(0.09)   ## Accent, hover.
	if to_light:
		return map
	## Reversing: the light run wrote the light values, so key the table by
	## those instead. Same four tokens, same two derivations.
	var rev := {}
	for hex in out:
		rev[(DccTheme.LIGHT[out[hex]] as Color).to_html(false)] = pal[out[hex]] as Color
	rev[(DccTheme.LIGHT["accent"] as Color).darkened(0.125).to_html(false)] = accent.darkened(0.125)
	rev[(DccTheme.LIGHT["accent"] as Color).lightened(0.09).to_html(false)] = accent.lightened(0.09)
	return rev

func _recolor_subtree(node: Node, old_pal: Dictionary) -> void:
	if node is Control or node is Window:
		for name in _THEME_COLOR_OVERRIDES:
			if node.has_theme_color_override(name):
				var nc = DccTheme.remap(node.get_theme_color(name), old_pal)
				if nc != null:
					node.add_theme_color_override(name, nc)
		for name in _THEME_STYLEBOX_OVERRIDES:
			if node.has_theme_stylebox_override(name):
				var sb: StyleBox = node.get_theme_stylebox(name)
				if sb is StyleBoxFlat:
					_recolor_stylebox(sb, old_pal)
				elif sb is StyleBoxLine:
					## The menu separator (`style_popup()`), the one non-Flat
					## box the shell authors. Silently skipped before it
					## existed, which would have left a dark hairline across a
					## light menu.
					var lc = DccTheme.remap((sb as StyleBoxLine).color, old_pal)
					if lc != null:
						(sb as StyleBoxLine).color = lc
	if node is ColorRect:
		var nc = DccTheme.remap((node as ColorRect).color, old_pal)
		if nc != null:
			(node as ColorRect).color = nc
	for child in node.get_children():
		_recolor_subtree(child, old_pal)

func _recolor_stylebox(sb: StyleBoxFlat, old_pal: Dictionary) -> void:
	var new_bg = DccTheme.remap(sb.bg_color, old_pal)
	if new_bg != null:
		sb.bg_color = new_bg
	var new_border = DccTheme.remap(sb.border_color, old_pal)
	if new_border != null:
		sb.border_color = new_border

# -- §4 Tool options bar ------------------------------------------------------

func _build_tool_options_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_TOOL_OPTIONS)
	bar.add_theme_stylebox_override("panel",
		DccTheme.panel("panel_alt", {"bottom": 1}))
	tool_options_row = HBoxContainer.new()
	tool_options_row.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(tool_options_row)
	bar.add_child(pad)
	return bar

## Replace the bar's contents. §4: it holds the active tool's frequently-changed
## values and its commit/discard, and never a control belonging to another tool
## -- so switching tools clears it rather than appending to it.
func set_tool_options(build: Callable) -> void:
	for child in tool_options_row.get_children():
		tool_options_row.remove_child(child)
		child.queue_free()
	build.call(tool_options_row)
	## Phone only: the tool sheet's height tracks its content
	## (`_build_phone_tool_sheet()` sets no fixed height), so a domain switch
	## can change how far `ViewportHost`'s corner chrome needs to clear it.
	## Deferred one frame so `_phone_bottom_reserve()` reads the sheet's real
	## post-layout size rather than its size from before this rebuild.
	if _phone:
		## `wide`: this row lives inside a horizontally scrolling sheet, so it
		## is the one subtree that must NOT be squeezed to fit the screen --
		## see `phone_fit()`'s own header.
		phone_fit(tool_options_row, _phone_scale, true)
		(func(): phone_insets_changed.emit()).call_deferred()

## Owner, 2026-08-20: "the bottom menu butons on phone are near too small to
## use". They were, and the earlier "44 px lands at ~121 physical px" arithmetic
## was measuring the wrong thing -- it described the *chrome* (`_build_phone_app
## _bar()`, the domain rail of the day), which does route every size through
## `_ptap()`. The sheet's *contents* never touched `_ptap()` at all: they are
## built by the workspaces' own `_build_*_tool_options_row()` callbacks against
## desktop pixel constants (`cartography_workspace.gd` sets buttons to a literal
## `Vector2(34, 20)`), and Godot's default stretch mode is disabled, so 20
## virtual px is 20 *physical* px -- about 1.6 mm on this 314 dpi panel.
##
## Fixed here rather than in the workspaces because `set_tool_options()` is the
## single choke point all of them already pass through, so one pass over the
## finished row phone-sizes every current and future tool row without making a
## dozen workspace files phone-aware (and without touching files another agent
## may be mid-flight in). Applied after `build.call()` so it sees the real
## nodes, and re-applied on every rebuild because each one makes fresh ones.
##
## Generalised 2026-08-24 from the tool row to any subtree, because the dock
## *sheets* and the three civ windows had exactly the same disease: every row
## in them comes from `dcc_widgets.gd`, which is authored in desktop pixels
## (`_row` is 24 px tall, `slider` 14, `action` 26, `tool_button` 30x30) and
## knows nothing about a phone. One walker fixes all of them; the alternative
## was making a dozen panel files phone-aware, several of which other agents
## are mid-flight in.
##
## `unit` is what one authored pixel is worth in this subtree's own space:
##   - `_phone_scale` for anything laid out in the main viewport (the docks,
##     the tool row) -- there is no content scale there, so a 24 px row really
##     is 24 physical px, about 2 mm.
##   - `1.0` for a `Window` that has already set `content_scale_factor` to
##     `_phone_scale` (the three civ windows, `open_project_dialog.gd`'s own
##     treatment): the scale is applied once by the compositor, and applying it
##     again here would double it.
##
## Idempotent by meta-flag, because the dock pass below re-runs on every
## rebuild and a second multiplication would grow every row without bound.
const _PHONE_FIT_META := "_phone_fitted"

## §13's touch-scroll deadzone, in authored pixels. See `phone_fit()`'s own
## `ScrollContainer` branch for why leaving it at Godot's 0 is not an option
## once a button forwards its drag.
const PHONE_SCROLL_DEADZONE := 10

## **The font-size override is not always called `font_size`.** Every control
## in this shell but one carries the generic name, so the walk below checked
## only that -- and a `RichTextLabel` does not have it at all. Its own sizes
## are five separate theme items, one per style, and setting `font_size` on
## one does nothing whatsoever. The right dock's "Why here?" causal chain
## (`right_dock.gd`, `normal_font_size` = `FS_SMALL`) was therefore skipped in
## silence and drew at a flat 11 *physical* px on a 1080-wide handset, about a
## third the height of every row above it. Measured on the device; there is no
## warning and no visible failure anywhere else.
const _FONT_SIZE_KEYS: PackedStringArray = ["font_size"]
const _RICH_FONT_SIZE_KEYS: PackedStringArray = [
	"normal_font_size", "bold_font_size", "italic_font_size",
	"bold_italic_font_size", "mono_font_size"]

## `wide` says the subtree scrolls horizontally, so nothing in it has to be
## made to fit a 393 dp column. Exactly one caller sets it -- the phone tool
## sheet, which `_build_phone_tool_sheet()` wraps in a `SCROLL_MODE_AUTO`
## `ScrollContainer` -- and it turns off the two width-shrinking measures
## below, both of which are wrong there and one of which was actively
## breaking it. PAINT ▸ Class is the case that found it: `fit_to_longest_item
## = false` plus `clip_text` leaves an `OptionButton` with **no** content-
## derived minimum width at all. Down a dock that is invisible, because the
## row is full width and the control expands into it; in the tool sheet the
## row is one of six side by side and none of them expands, so the control
## collapsed onto its own drop-down arrow -- 35 px, showing which class is
## selected nowhere. Sizing it from its longest item instead just makes the
## sheet a little wider, and the sheet already scrolls.
func phone_fit(node: Node, unit: float, wide: bool = false) -> void:
	for child in node.get_children():
		if child is Control and not child.has_meta(_PHONE_FIT_META):
			var ctl := child as Control
			ctl.set_meta(_PHONE_FIT_META, true)
			## Explicit font-size overrides beat any theme we could hang on the
			## sheet, so they have to be re-written rather than inherited.
			## See `_RICH_FONT_SIZE_KEYS` for why the name is asked for per
			## control class rather than assumed to be `font_size`.
			if unit != 1.0:
				var rich := ctl is RichTextLabel
				var scaled_any := false
				for key in (_RICH_FONT_SIZE_KEYS if rich else _FONT_SIZE_KEYS):
					if ctl.has_theme_font_size_override(key):
						ctl.add_theme_font_size_override(key,
							maxi(1, int(round(ctl.get_theme_font_size(key) * unit))))
						scaled_any = true
				## A `RichTextLabel` that overrides *nothing* still needs the
				## pass: it is pure text with no minimum-size floor to catch
				## it, so left alone it renders at the stock theme size, which
				## on a phone is the same unscaled physical pixel the override
				## case was. `app.gd`'s credits body is the other one in this
				## shell. Resolved off the theme rather than hard-coded, so a
				## re-themed default still lands right.
				if rich and not scaled_any:
					ctl.add_theme_font_size_override("normal_font_size",
						maxi(1, int(round(ctl.get_theme_font_size("normal_font_size") * unit))))
			## Scale whatever the desktop row asked for, then floor anything
			## tappable at §13's 44 px -- the floor is the half the owner felt.
			var tap := maxf(1.0, round(DccTheme.PHONE_TAP_MIN * unit))
			var min_size := ctl.custom_minimum_size
			if min_size.x > 0.0:
				min_size.x = round(min_size.x * unit)
			if min_size.y > 0.0:
				min_size.y = round(min_size.y * unit)
			if ctl is BaseButton or ctl is LineEdit or ctl is Range or ctl is TextEdit:
				min_size.y = maxf(min_size.y, tap)
				if min_size.x > 0.0:
					min_size.x = maxf(min_size.x, tap)
			ctl.custom_minimum_size = min_size
			## §4.5's TOOLS block. The floor above grew each tool's *box* to
			## 44 dp and left everything inside it exactly as authored, which is
			## the whole fault: `dcc_widgets.gd`'s `tool_button` is a 15 px
			## glyph, an **empty** `normal` stylebox, and the tool's name in a
			## tooltip. On a pointer that is a complete control -- hover names
			## it, and 30 px is a comfortable target. On a handset it is
			## neither. There is no hover, so the name is unreachable by any
			## route at all; and 15 px stays 15 *physical* px, about a
			## millimetre on a 400 ppi panel, sitting left-aligned in a 121 px
			## cell with no border to say where the button even is. CIVIL's
			## block is seven such marks (Inspect, Measure, Region select,
			## Settlement, Territory, Way, Route) with nothing to tell any of
			## them apart. Measured on the device, and exactly the class of
			## fault no headless check can see.
			if ctl is Button and ctl.has_meta(DccWidgets.TOOL_GLYPH_META):
				_phone_fit_tool_button(ctl as Button, unit)
			## The 412 canvas's action button: a 48 dp pill, filled for the
			## primary and outlined for the secondary. Only reached from here, so
			## the desktop chip is untouched everywhere else -- see
			## `DccWidgets.phone_pill()`.
			elif ctl is Button and ctl.has_meta(DccWidgets.ACTION_META):
				DccWidgets.phone_pill(ctl as Button, unit)
			## 3 px track, 22 dp round thumb, 32 dp row. The dock's slider has no
			## grabber at all by §11; the phone canvas draws one on every slider
			## it has, because a finger has no cursor to find the handle with.
			if ctl is HSlider:
				DccWidgets.phone_slider(ctl as HSlider, unit)
			## **A drag that starts on a row has to reach the scroll above it.**
			## `dcc_widgets.gd` builds every row as an `HBoxContainer`, and a
			## `Control` picks by default (`MOUSE_FILTER_STOP`), which ends the
			## event walk right there -- Godot delivers a GUI event to the picked
			## control and then up its parents, stopping at the first `STOP`. On a
			## pointer that costs nothing, because scrolling is the wheel. On a
			## phone it is the whole gesture: the left dock sheet could only be
			## scrolled by catching its 4 px scrollbar, which on a 400 ppi panel
			## is about a millimetre, so the NPR Painter block below the fold was
			## effectively unreachable. Found by driving the real handset -- a
			## flick on the rows did nothing, the same flick on the scrollbar
			## worked.
			##
			## `PASS`, not `IGNORE`: a `PASS` control is still picked, so the
			## row keeps its own tooltip and hover, and only *forwards* what it
			## does not handle. Layout containers only -- a `PanelContainer` is
			## excluded because several in this shell (`phone_menu.gd`'s rows,
			## the roster's folded bar) carry their own `gui_input` and must
			## keep stopping the event they consume.
			if (ctl is BoxContainer or ctl is MarginContainer) \
					and ctl.mouse_filter == Control.MOUSE_FILTER_STOP:
				ctl.mouse_filter = Control.MOUSE_FILTER_PASS
			## PH-05's last hole, found by re-running the flick sweep at
			## `_phone_scale` 2.748 rather than at the 393 dp reference every earlier
			## probe used: 6 of 8 points down the left sheet scrolled 329 px and two
			## did not. One is an `HSlider`, which is deliberate and is explained
			## below. The other is a **bare `Control`** -- `DccTheme.spacer()` and the
			## fixed-width gaps beside it -- which defaults to `MOUSE_FILTER_STOP` and
			## so ends the event walk on a node that exists only to take up room.
			##
			## Matched on the exact class rather than `is Control`, because every
			## control in this shell is one; and skipped if anything is listening on
			## `gui_input`, since a plain `Control` with a handler (a scrim, a drag
			## handle) is picking on purpose. A spacer with neither has nothing to
			## consume the event it is currently swallowing.
			if ctl.get_class() == "Control" and ctl.get_script() == null \
					and ctl.mouse_filter == Control.MOUSE_FILTER_STOP \
					and ctl.get_signal_connection_list("gui_input").is_empty():
				ctl.mouse_filter = Control.MOUSE_FILTER_PASS
			## PH-05, the other half of the same sentence -- and the half that was
			## actually load-bearing. A `Container` already defaults to `PASS`
			## (measured, 4.7.1: `MOUSE_FILTER_PASS`, not the `Control` default the
			## comment above assumed), so the rows were never the blocker. **A
			## `Button` is.** `_scrolldrag_probe.gd` flicked twenty points down the
			## left sheet: every point that failed to scroll was a `Button` or an
			## `HSlider`, and from the accordion down the sheet is nothing *but*
			## buttons -- the L2 `category()` headers, the L4 `group()` headers, every
			## `action()`. That is the "a flick on the content does nothing" the
			## handset found, and it is why the scrollbar still worked: only the
			## content was covered.
			##
			## `PASS` is safe on a button *because* `ScrollContainer` and `BaseButton`
			## already cooperate: past the deadzone the scroll propagates
			## `NOTIFICATION_SCROLL_BEGIN`, which cancels the button's pending press.
			## Measured, all four cases: a clean tap fires, a 2 px and a 6 px wobble
			## still fire, an eight-sample flick scrolls 96 px and fires nothing.
			##
			## An `HSlider` is deliberately **not** included: a drag that starts on a
			## slider means "move this slider", on every touch platform there is.
			##
			## Neither are the three `BaseButton`s that open a `Popup` on *press* --
			## and not for symmetry: such a control pops mid-flick, the popup grabs
			## the drag, and the gesture then neither scrolls nor is undone (measured
			## on `OptionButton`: popup open, scroll 0). Their rows still scroll from
			## the label beside them.
			if ctl is BaseButton and ctl.mouse_filter == Control.MOUSE_FILTER_STOP 					and not (ctl is OptionButton or ctl is MenuButton 						or ctl is ColorPickerButton):
				ctl.mouse_filter = Control.MOUSE_FILTER_PASS
			## That deadzone is not a default -- Godot's is **0**, at which the ~2 px
			## of wobble in a real thumb tap already counts as a drag and silently
			## eats the press. Without this, the fix above would trade "the sheet does
			## not scroll" for "the buttons do not press". Scaled with the rest of the
			## subtree, so it is the same physical distance in a dock (unscaled,
			## `unit` = `_phone_scale`) as in a content-scaled window (`unit` = 1.0).
			if ctl is ScrollContainer:
				(ctl as ScrollContainer).scroll_deadzone = maxi(
					PHONE_SCROLL_DEADZONE, int(round(PHONE_SCROLL_DEADZONE * unit)))
			## An `OptionButton`'s list is a `PopupMenu`, which is a `Window` and
			## not a `Control` -- so it is not in this walk and inherits none of
			## the above. Left alone its rows came out at ~21 dp inside a
			## content-scaled window (measured on the handset, City Viewer's
			## settlement picker: 40 names at half the tap floor), and at ~8 dp
			## in a dock, where nothing scales the stock font either.
			##
			## A `PopupMenu` has no row-height property; a row is its font plus
			## `v_separation`, so those are the two knobs. 22 is what brings a
			## default row to the 44 dp floor rather than an arbitrary bump.
			## **A `Button` reports its own text as its minimum width**, and a
			## `Window` cannot be narrower than its content's minimum -- so one
			## long label anywhere inside a dialog widens the whole window past
			## the screen, and everything laid out after the scrolling pane goes
			## with it. The faction roster is the case that found this: its
			## settlement sublist ("Draumr League — 13 settlements, 39 210") and
			## its vocabulary pickers put the content minimum at 473 px on a
			## 393 dp screen, the window grew to fit, and the Add/Remove row
			## ended up 1 750 px below the bottom of the phone. Measured with
			## `get_combined_minimum_size()` down the dialog's own tree in a
			## `--force-touch` run, not guessed.
			##
			## Trimming takes the text out of that calculation while still
			## drawing it in full wherever it fits -- which at 393 dp is nearly
			## everywhere -- and marks the rest with an ellipsis rather than
			## cutting mid-glyph.
			##
			## Only for a control that already stretches. A `Button` sized by
			## its own text and *not* expanding has nothing else to get a width
			## from, so trimming it collapses it to nothing -- the roster's
			## Add/Remove pair went to zero width the first time this was
			## applied to every button alike.
			if not wide and ctl is Button and (ctl.size_flags_horizontal & Control.SIZE_EXPAND) != 0:
				(ctl as Button).clip_text = true
				(ctl as Button).text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			if ctl is OptionButton:
				## `clip_text` alone does not shrink an `OptionButton`:
				## `fit_to_longest_item` is on by default, so it reports the
				## width of the **longest item in the list**, not of the
				## selection, specifically so the control does not resize when
				## you pick a different one. On a 393 dp screen that is the
				## expensive guarantee -- Ag. technology's "Traditional Agrarian
				## (ard plow, …)" alone asked for 287 px of a 393 dp row. A
				## phone row is full width and never sits beside anything, so
				## there is no reflow to protect against.
				if not wide:
					(ctl as OptionButton).fit_to_longest_item = false
				var pop := (ctl as OptionButton).get_popup()
				pop.add_theme_constant_override("v_separation", int(round(22.0 * unit)))
				if unit != 1.0:
					pop.add_theme_font_size_override("font_size",
						maxi(1, int(round(pop.get_theme_font_size("font_size") * unit))))
				## HD-01's other half. An embedded sub-window is drawn inside its
				## parent's canvas, so this list inherits the parent window's
				## content scale -- which is the whole reason the `unit != 1.0`
				## branch above exists -- but not the parent's font raster, which
				## is per-`Viewport`. So in the content-scaled case (`unit` 1.0,
				## magnify 3.664) the list's own rows smear exactly as the window
				## behind them did. In a dock the magnify is 1 and the branch
				## above already rasterised the font at its real size, so nothing
				## is set and nothing changes.
				DccWidgets.oversample(pop, _phone_magnify(unit))
		phone_fit(child, unit, wide)

## What one unit of `phone_fit()`'s own space is worth in physical pixels.
##
## The two spaces `phone_fit()` serves reach the same place by different routes:
## a dock lays out in real pixels and `unit` is `_phone_scale`, a content-scaled
## window lays out in dp and `unit` is 1.0 while the compositor supplies the
## rest. Their product is `_phone_scale` in both, which is why one expression
## covers both and why a caller never has to know which one it is in. Exactly
## 1.0 in a dock (`_phone_scale / _phone_scale` is exact in IEEE-754), so the
## dock path is byte-identical to what it did before this existed.
##
## Gated on `_phone` and not only on the callers being phone-only, because
## `_compute_layout_mode()` computes `_phone_scale` for **every** composition --
## a 1920 x 1080 desktop reads 2.75 -- and a `phone_fit()` call that ever
## reached a pointer build would otherwise quietly rasterise every glyph at
## nearly three times its size and minify it back down, which is worse than the
## fault this closes rather than merely wasteful.
func _phone_magnify(unit: float) -> float:
	if not _phone:
		return 1.0
	return _phone_scale / maxf(0.0001, unit)

## The phone half of a §4.5 tool button: a real glyph, a visible box, and the
## name the tooltip can no longer deliver. See the call site in `phone_fit()`
## for the fault this closes.
func _phone_fit_tool_button(b: Button, unit: float) -> void:
	## Re-rasterised from the SVG at the size it will actually be drawn at,
	## which is why `dcc_widgets.gd` stashes the glyph's *name*: `DccIcons`
	## caches per (name, drawn, raster), so the 15 px texture already in hand
	## cannot be grown without resampling it. 0.42 of the box leaves the caption
	## room and keeps the icon off the border.
	##
	## `_phone_magnify(unit)` is HD-02's half: `box` is in this subtree's own
	## space, so in a dock it is already physical and the magnify is exactly 1
	## (this call is unchanged there), while in a content-scaled window it is dp
	## and the raster has to be 3.664x finer than the number beside it.
	var box := maxf(b.custom_minimum_size.x, b.custom_minimum_size.y)
	b.icon = DccIcons.get_icon(String(b.get_meta(DccWidgets.TOOL_GLYPH_META)),
		maxi(1, int(round(box * 0.42))), _phone_magnify(unit))
	## The desktop button is invisible at rest on purpose -- a palette of eight
	## empty squares reads as one strip, and hover picks one out. Touch has no
	## hover, so at rest is the only state there is, and an unbounded mark is
	## not identifiable as a target. `pressed` keeps its accent wash, so armed
	## still reads differently from merely present.
	b.add_theme_stylebox_override("normal",
		DccTheme.outline("line_soft", "panel", maxi(1, int(round(unit)))))
	## `outline()` and `flat()` both carry a zero content margin, so without
	## this the caption is drawn hard against the border it just gained -- and
	## the *three* states have to agree, or `Button` re-lays its content out on
	## press and the label jumps sideways under the finger holding it.
	var pad: float = round(4.0 * unit)
	for state in ["normal", "hover", "pressed"]:
		var sb: StyleBox = b.get_theme_stylebox(state)
		sb.content_margin_left = pad
		sb.content_margin_right = pad
	if not b.has_meta(DccWidgets.TOOL_CAPTION_META):
		return
	b.text = String(b.get_meta(DccWidgets.TOOL_CAPTION_META))
	## Icon *above* the caption, not beside it: `Button` stacks the two
	## whenever `vertical_icon_alignment` is anything but CENTER, and stacking
	## is what keeps a tool close to square instead of turning the row into
	## four wide pills. The width this asks for is why `tools_block()` lays its
	## rows out in an `HFlowContainer`.
	b.vertical_icon_alignment = VERTICAL_ALIGNMENT_TOP
	b.alignment = HORIZONTAL_ALIGNMENT_CENTER
	b.add_theme_font_size_override("font_size",
		maxi(1, int(round(DccTheme.FS_MICRO * unit))))
	b.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	b.add_theme_color_override("font_pressed_color", DccTheme.c("accent"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))

## The dock sheets carry every workspace panel -- the NPR Painter block, the
## CIVIL dock's Faction roster button, the right dock's Settlement ▸ City
## layout -- and every one of them is rebuilt from a signal at some point after
## boot, so a one-shot pass over the dock at build time would be correct for
## about a second. `node_added` is the only hook that sees all of them without
## this file knowing which panels exist; the work is coalesced onto one
## deferred pass per frame, so a rebuild that adds 2000 nodes still fits once.
##
## Cheap because the fit itself is meta-flagged: the pass walks the sheet, but
## only *touches* the nodes it has not already sized.
var _phone_fit_pending := false

func _on_phone_node_added(node: Node) -> void:
	if _phone_fit_pending or not (node is Control):
		return
	## Only a dock descendant is our business. Walked rather than connected
	## per-panel because `child_entered_tree` fires for direct children only,
	## and every panel is several levels down.
	var p: Node = node.get_parent()
	while p != null:
		if p == left_dock or p == right_dock:
			_phone_fit_pending = true
			_run_phone_dock_fit.call_deferred()
			return
		p = p.get_parent()

func _run_phone_dock_fit() -> void:
	_phone_fit_pending = false
	for dock in [left_dock, right_dock]:
		if dock != null:
			phone_fit(dock, _phone_scale)

## Read by dialogs that have to present themselves differently on a phone
## (`open_project_dialog.gd`); `_phone`/`_phone_scale` stay private because
## nothing outside should be *setting* them.
func is_phone() -> bool:
	return _phone

func phone_scale() -> float:
	return _phone_scale

## The node `Window ▸ Domain rail` shows and hides, for `DccApp`'s region map.
## Falls back to `rail_column` rather than returning null, so a composition that
## somehow never set it hides *something* real instead of crashing the menu.
func rail_region() -> Control:
	return _rail_region if _rail_region != null else rail_column

# -- §3 Domain rail -----------------------------------------------------------

## **The rail has exactly one width, and no collapsed/expanded pair.**
##
## Between 2026-08-19 and 2026-08-24 it had both: SH-01 turned the mockup's head
## chevron into a `Button` that grew the rail to `W_RAIL_EXPANDED` (200 px) and
## swapped the domain column for a `_phone_list_row()` list of each domain's
## sub-structure. `DCC_SHELL_SPEC.md` §3 does ask for that ("the domain's
## sub-nodes as a 200 px list"), which is why it was built.
##
## It is gone because the canvas -- the ground truth the shell is measured
## against -- **never draws it**. Every one of the eight desktop artboards
## across `design/Cartalith DCC Shell.dc.html` and
## `design/Cartalith Measurement Toolbar.dc.html` opens the rail with the same
## literal `width:40px;flex:none`, in the dark theme, the light theme, the
## tablet composition and all three measurement states. There is no artboard of
## an expanded rail to build against, and the state that got built instead
## borrowed the *phone* drawer's type scale into a 200 px column: screenshotted
## live, "CARTOGRAPHY" ran straight under the left dock. The owner reported it
## as "the left rail is collapsible and shouldn't be".
##
## What the canvas *does* draw is kept: a 29 px head cell carrying a dim `›`,
## ruled off from the domains below. It is a `Label` now rather than a `Button`
## -- chrome the mockup specifies, not an affordance nothing behind it can
## honour. `Window ▸ Domain rail` still hides the whole region, unchanged: that
## is the same region toggle the other four layout regions have, reversible from
## the same menu, and not what "collapsible" meant here.
func _build_rail() -> Control:
	var rail := PanelContainer.new()
	_rail_region = rail
	rail.custom_minimum_size.x = _scaled(DccTheme.W_RAIL_COLLAPSED)
	rail.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))
	rail_column = VBoxContainer.new()
	rail_column.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_top", 12)
	pad.add_child(rail_column)

	## The mockup opens the rail with a 29 px cell carrying a dim `›`, ruled off
	## from the domains below it -- `MOUSE_FILTER_IGNORE` so it is unmistakably
	## chrome and never eats a hover.
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	var head := DccTheme.mono_label(DccIcons.SYMBOLS["expand"], "text_dim", DccTheme.FS_SMALL)
	head.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	head.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	head.custom_minimum_size.y = _scaled(29)
	head.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(head)
	col.add_child(DccTheme.rule())

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 0)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_child(pad)
	col.add_child(body)
	rail.add_child(col)

	var w := float(_scaled(DccTheme.W_RAIL_COLLAPSED))
	for i in DOMAINS.size():
		var d: Dictionary = DOMAINS[i]
		if i > 0:
			## A 14 px hairline between each pair, exactly as the mockup draws
			## it -- the rail's only ornament.
			var sep := ColorRect.new()
			sep.color = DccTheme.c("line")
			sep.custom_minimum_size = Vector2(14, 1)
			sep.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
			rail_column.add_child(sep)

		var b := Button.new()
		b.tooltip_text = "%s -- %s" % [d.label, d.subtitle]
		## Not flat: a flat `Button` skips its styleboxes, so the `hover` box on
		## the next line had never drawn and the rail gave no pointer feedback
		## at all. `normal` is empty, so nothing else changes. (Same trap as the
		## menu bar's open title, found in the same 2026-08-25 menu sweep.)
		b.flat = false
		b.focus_mode = Control.FOCUS_NONE
		b.add_theme_stylebox_override("normal", DccTheme.empty())
		b.add_theme_stylebox_override("focus", DccTheme.empty())
		b.add_theme_stylebox_override("disabled", DccTheme.empty())
		b.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
		b.pressed.connect(_select_domain.bind(d.id))

		## The reference rail is text only -- verified twice: once by reading
		## `design/Cartalith DCC Shell.dc.html`'s own markup (`writing-mode:
		## vertical-rl` labels, no icon element anywhere in the rail), and once
		## by the owner directly, after an earlier revision added icons anyway:
		## "those icons don't exist." Removed rather than hidden behind a flag --
		## an addition the design does not specify does not get to linger.
		## `font:10px 'IBM Plex Mono'; letter-spacing:.12em; color:#5f6468`,
		## regular weight -- the rail block's own inline style, verbatim. This
		## was 9 px at `spacing 2` (≈.22em) in Medium, which is a size down, a
		## tracking up and a weight up all at once: three small errors
		## compounding on the one piece of chrome that is always on screen.
		## `spacing` is whole pixels, so .12em at 10 px is 1.
		var vlabel := DccTheme.mono_label(String(d.rail).to_upper(),
			"text_ghost", DccTheme.FS_TINY, 1, false)
		vlabel.rotation = -PI / 2.0
		b.add_child(vlabel)
		## Provisional geometry only. The real numbers come from
		## `_layout_rail_labels()` below, once this rail is in the tree --
		## measuring here cannot work, and that is not a style preference:
		## nothing built by this function is in the `SceneTree` yet (the rail
		## is *returned* and added by the caller), so a `Label` asked for its
		## minimum size here has no theme and therefore no font to measure.
		## Measured before the fix: the three five-character labels WORLD /
		## CIVIL / CARTO produced button heights of **67, 53 and 62 px** from
		## this early read, against the identical `(34, 14)` all three report
		## once in-tree. Three different wrong answers to the same question.
		_layout_rail_label(b, vlabel, w)

		_domain_buttons[d.id] = b
		_domain_marks[d.id] = {"label": vlabel}
		rail_column.add_child(b)

	## Re-measure every rail label once the rail is actually in the tree. See
	## `_layout_rail_label()` for why this cannot be done above.
	call_deferred("_relayout_rail_labels")

	body.add_child(DccTheme.spacer())
	rail_foot = DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 2)
	rail_foot.rotation = -PI / 2.0
	var foot_holder := Control.new()
	foot_holder.custom_minimum_size.y = 84
	foot_holder.add_child(rail_foot)
	body.add_child(foot_holder)
	return rail

## Centre one rotated rail label inside its own button, and size the button to
## it.
##
## **The rotation is why this needs stating.** A `Control` rotates about its
## `pivot_offset`, which defaults to its top-left, and `rotation = -PI/2` maps
## local `(x, y)` to parent `(y, -x)`. So a label of length `L` positioned at
## `y` occupies the parent's vertical span **`[y - L, y]`** -- it grows
## *upward* from its own position, not downward. The pre-2026-08-30 code set
## that position to a flat `12.0`, which put the text `L - 12` px **above the
## top of its own button**: measured live, all three labels overflowed by 22 px
## and left 41-55 px of empty band beneath them, which is the misalignment the
## owner reported.
##
## Centred means the span `[y - L, y]` is centred in a button of height
## `L + 2·PAD`, which solves to `y = L + PAD`.
const RAIL_LABEL_PAD := 12.0

func _layout_rail_label(b: Button, vlabel: Control, rail_w: float) -> void:
	var text_size := vlabel.get_minimum_size()
	## `text_size.y` is the glyph height, which *after* the -90° rotation is
	## the label's horizontal extent -- hence `.y` against the rail's width.
	vlabel.position = Vector2(
		round(rail_w * 0.5 - text_size.y * 0.5),
		text_size.x + RAIL_LABEL_PAD)
	b.custom_minimum_size.y = text_size.x + RAIL_LABEL_PAD * 2.0

## Deferred from `_build_rail()`, because nothing that function builds is in the
## `SceneTree` while it runs, and an orphaned `Label` has no theme to measure a
## font with. Idempotent, so it is safe to call again after a theme or scale
## change.
func _relayout_rail_labels() -> void:
	if _rail_region == null or not is_instance_valid(_rail_region):
		return
	var w := float(_scaled(DccTheme.W_RAIL_COLLAPSED))
	for id in _domain_marks.keys():
		var b: Button = _domain_buttons.get(id)
		var lbl: Control = _domain_marks[id].get("label")
		if b != null and lbl != null and is_instance_valid(b) and is_instance_valid(lbl):
			_layout_rail_label(b, lbl, w)

## The rail foot carries the active context and, in World, the stage counter.
## Re-centred on every set because its width changes with the text.
func set_rail_foot(text: String) -> void:
	if rail_foot == null:
		return
	rail_foot.text = text
	var w := float(_scaled(DccTheme.W_RAIL_COLLAPSED))
	var m := rail_foot.get_minimum_size()
	rail_foot.position = Vector2(round(w * 0.5 - m.y * 0.5), 12.0 + m.x)

func _select_domain(id: String) -> void:
	_active_domain = id
	for key in _domain_buttons:
		var b: Button = _domain_buttons[key]
		var on: bool = key == id
		var marks_pre: Dictionary = _domain_marks.get(key, {})
		## The desktop rail's active cell is `background:rgba(224,163,74,.08)`
		## in its own artboard. The 412 phone canvas's active *tab* has no fill
		## at all -- `<div style="...;color:#e0a34a">` and nothing else -- so the
		## bar cell registers `"box": false` and states itself in ink, glyph and
		## caption together, the way a bottom-nav tab does everywhere.
		if bool(marks_pre.get("box", true)):
			b.add_theme_stylebox_override("normal",
				DccTheme.active_row(false) if on else DccTheme.empty())
		var marks: Dictionary = _domain_marks.get(key, {})
		## `text_ghost` (`#5f6468`) is the *desktop rail's* resting ink -- see
		## `_build_rail()`. The 412 phone canvas rests its bottom-nav tabs one
		## step brighter, at `#8d9296` (`text_dim`), so the cell that registers
		## the mark says which. Restoring to `text_faint` here used to mean the
		## rail brightened by one step the first time a domain was ever selected
		## and never went back.
		var off: String = String(marks.get("off", "text_ghost"))
		if marks.has("label"):
			(marks["label"] as Label).add_theme_color_override("font_color",
				DccTheme.c("accent") if on else DccTheme.c(off))
		## The phone bar's glyph. A `DccIcons` texture is drawn in white and
		## tinted by `modulate`, so this is the same one-asset/two-states
		## contract `dcc_icons.gd`'s header describes -- not a second texture.
		if marks.has("icon"):
			(marks["icon"] as CanvasItem).modulate = \
				DccTheme.c("accent") if on else DccTheme.c(off)
	for key in _workspace_panels:
		(_workspace_panels[key] as Control).visible = key == id
	for d in DOMAINS:
		if d.id == id:
			left_dock_title.text = String(d.label).to_upper()
			break
	workspace_changed.emit(id)

## A workspace module calls this once, from `_ready`, with the panel it wants in
## the left dock. Panels are built up front and hidden, not rebuilt on every
## switch -- §3 requires each domain's L2 open/closed state to persist.
func register_workspace(id: String, panel: Control) -> void:
	panel.visible = id == _active_domain
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_workspace_panels[id] = panel
	left_dock_body.add_child(panel)

func active_domain() -> String:
	return _active_domain

## WI-02's public entry point: `menus.gd`'s Window ▸ Workspace submenu jumps
## here rather than calling the underscore-prefixed `_select_domain()`
## directly across a file boundary.
func select_domain(id: String) -> void:
	_select_domain(id)

## Switch domain **and** open the category the caller named -- what every
## "→ Cartography ▸ Political display"-style jump button in the shell actually
## promises. See `Workspace.open_category()` for why half of it was not enough
## once v3 gave CIVIL fourteen categories and CARTO ten.
##
## Silent about a miss on purpose at the call site, loud in the log: a stale
## pointer must not swallow the domain switch the user asked for, and a
## `push_warning` is what a probe can assert on.
func select_domain_category(id: String, category: String) -> void:
	_select_domain(id)
	var panel: Control = _workspace_panels.get(id)
	if panel != null and panel.has_method("open_category"):
		if not panel.call("open_category", category):
			push_warning("Cartalith: no category '%s' in the %s dock -- stale cross-domain pointer." % [category, id])

## §6's right-dock header -- see `_build_right_dock()`'s own comment on why
## this exists instead of a fixed "LAYERS" label. `text` is already the
## upper-cased section name; `DccTheme.header()` built the sigil-free label
## once with an initial value, this just updates its text in place.
func set_right_dock_title(text: String) -> void:
	if right_dock_title != null:
		right_dock_title.text = text.to_upper()

# -- §6 Docks -----------------------------------------------------------------

## `as_sheet`: §13's phone treatment -- "docks become full-height sheets, one
## at a time". The header swaps its collapse chevron for a close button and
## the dock stops claiming a fixed desktop width, but the body underneath
## (`left_dock_body`, where every workspace attaches its panel) is built
## exactly as it is for desktop/tablet, unchanged -- this is the "minimal or
## no change" reuse the phone chrome depends on.
func _build_left_dock(as_sheet: bool = false) -> Control:
	left_dock = PanelContainer.new()
	if not as_sheet:
		left_dock.custom_minimum_size.x = _left_width
	left_dock.add_theme_stylebox_override("panel",
		DccTheme.panel("panel") if as_sheet else DccTheme.panel("panel", {"right": 1}))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if as_sheet:
		left_dock.add_child(col)
	else:
		## WI-04 (§1: "user-draggable within min/max"): a 6 px grip at the
		## dock's inner edge, facing the viewport -- carved out of the dock's
		## own reserved width rather than added to it, so `_left_width` still
		## means what it always has. `_dock_drag_handle()` owns the drag math;
		## the right dock mirrors this with the handle on its other side.
		var body_row := HBoxContainer.new()
		body_row.add_theme_constant_override("separation", 0)
		body_row.add_child(col)
		body_row.add_child(_dock_drag_handle(true))
		left_dock.add_child(body_row)

	var head := HBoxContainer.new()
	## 34 px, not 26: the canvas gives both dock headers `height:34px` -- the
	## same band the menu bar and the tool options bar get, so the three
	## horizontal rules across the top of the shell line up as one rhythm.
	## 26 is `H_STATUS`, borrowed here by mistake, and it left the dock title
	## sitting 4 px proud of the tool options bar beside it.
	head.custom_minimum_size.y = _ptap(44) if as_sheet else _scaled(34)
	left_dock_title = DccTheme.header("WORLD", "")
	head.add_child(left_dock_title)
	head.add_child(DccTheme.spacer())
	if as_sheet:
		head.add_child(_sheet_close_button(func(): _set_sheet_open("left", false)))
	else:
		head.add_child(_collapse_button(true))
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_right", 6)
	head_pad.add_child(head)
	col.add_child(head_pad)
	col.add_child(DccTheme.rule())
	## Always built, even in sheet mode, so `set_dock_readout("left", …)`
	## (`world_workspace.gd`) never hits the "no dock readout" error -- it just
	## stays permanently hidden, since a sheet has no collapsed state to
	## surface it in.
	col.add_child(_dock_readout("left"))

	var scroll := _scroll()
	_left_dock_scroll = scroll
	left_dock_body = VBoxContainer.new()
	left_dock_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_dock_body.add_theme_constant_override("separation", 0)
	scroll.add_child(left_dock_body)
	col.add_child(scroll)
	return left_dock

## `as_sheet`: see `_build_left_dock()` -- identical treatment, mirrored.
func _build_right_dock(as_sheet: bool = false) -> Control:
	right_dock = PanelContainer.new()
	if not as_sheet:
		right_dock.custom_minimum_size.x = _right_width
	right_dock.add_theme_stylebox_override("panel",
		DccTheme.panel("panel") if as_sheet else DccTheme.panel("panel", {"left": 1}))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if as_sheet:
		right_dock.add_child(col)
	else:
		## `as_sheet`: see `_build_left_dock()`'s own drag-handle comment --
		## identical treatment, mirrored so the grip faces the viewport here too.
		var body_row := HBoxContainer.new()
		body_row.add_theme_constant_override("separation", 0)
		body_row.add_child(_dock_drag_handle(false))
		body_row.add_child(col)
		right_dock.add_child(body_row)

	## §6's own header carries whatever context is showing (Sample, Settlement,
	## Route...), not a fixed label -- "Layers" was this dock's mockup-pictured
	## *default* state, not its permanent chrome title; a bare "LAYERS" left
	## painted here regardless of context was misleading once every other
	## context (Sample, Settlement, Route, River, Faction, Measure, Region
	## select, Stamp stack, Journey) had its own real section header one
	## scroll-step below it saying something else. `right_dock.gd`'s
	## `_rebuild()` keeps this in sync via `set_right_dock_title()`, the same
	## pattern `left_dock_title` already follows for the domain name.
	right_dock_title = DccTheme.header("SAMPLE", "")
	var head := HBoxContainer.new()
	## 34 px, not 26: the canvas gives both dock headers `height:34px` -- the
	## same band the menu bar and the tool options bar get, so the three
	## horizontal rules across the top of the shell line up as one rhythm.
	## 26 is `H_STATUS`, borrowed here by mistake, and it left the dock title
	## sitting 4 px proud of the tool options bar beside it.
	head.custom_minimum_size.y = _ptap(44) if as_sheet else _scaled(34)
	if as_sheet:
		head.add_child(right_dock_title)
		head.add_child(DccTheme.spacer())
		head.add_child(_sheet_close_button(func(): _set_sheet_open("right", false)))
	else:
		head.add_child(_collapse_button(false))
		head.add_child(right_dock_title)
		head.add_child(DccTheme.spacer())
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 6)
	head_pad.add_theme_constant_override("margin_right", 12)
	head_pad.add_child(head)
	col.add_child(head_pad)
	col.add_child(DccTheme.rule())
	col.add_child(_dock_readout("right"))

	var scroll := _scroll()
	_right_dock_scroll = scroll
	right_dock_body = VBoxContainer.new()
	right_dock_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_dock_body.add_theme_constant_override("separation", 0)
	scroll.add_child(right_dock_body)
	col.add_child(scroll)
	return right_dock

## Godot's default theme draws a rounded, outlined panel behind every
## ScrollContainer. §11 is explicit that regions are separated by hairlines
## only, with radius 0 everywhere, so the panel is removed rather than
## restyled -- the dock around it already draws the one border there should be.
func _scroll() -> ScrollContainer:
	var s := ScrollContainer.new()
	s.size_flags_vertical = Control.SIZE_EXPAND_FILL
	s.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	s.add_theme_stylebox_override("panel", DccTheme.empty())
	return s

## WI-04: the drag grip itself, a bare 6 px strip that lights up on hover so
## it reads as an affordance without adding chrome the spec doesn't draw.
## `is_left` picks which dock it belongs to -- purely to route the drag delta
## and to know which of `_left_width`/`_right_width` and which min/max pair
## (§1's geometry table) apply, since dragging right *widens* the left dock
## but *narrows* the right one.
func _dock_drag_handle(is_left: bool) -> Control:
	var handle := PanelContainer.new()
	handle.custom_minimum_size.x = 6
	handle.mouse_default_cursor_shape = Control.CURSOR_HSPLIT
	handle.mouse_filter = Control.MOUSE_FILTER_STOP
	handle.add_theme_stylebox_override("panel", DccTheme.empty())
	handle.mouse_entered.connect(func(): handle.add_theme_stylebox_override(
		"panel", DccTheme.flat(DccTheme.c("line_soft"))))
	handle.mouse_exited.connect(func():
		if _dragging_dock == "":
			handle.add_theme_stylebox_override("panel", DccTheme.empty()))
	handle.gui_input.connect(_on_dock_drag_input.bind(is_left))
	return handle

## Godot routes mouse motion to whichever Control had the initial press even
## once the cursor drifts off its rect (the same mechanism `SplitContainer`'s
## own internal dragger relies on), so this needs no separate `_input`
## override to track a drag past the handle's own 6 px width.
func _on_dock_drag_input(ev: InputEvent, is_left: bool) -> void:
	var side := "left" if is_left else "right"
	if is_dock_collapsed(side):
		return
	if ev is InputEventMouseButton and ev.button_index == MOUSE_BUTTON_LEFT:
		_dragging_dock = side if ev.pressed else ""
	elif ev is InputEventMouseMotion and _dragging_dock == side:
		## Dragging right widens the left dock (its handle sits on its right
		## edge) but narrows the right dock (its handle sits on its left edge).
		var delta: float = ev.relative.x if is_left else -ev.relative.x
		if is_left:
			_left_width = clampf(_left_width + delta,
				float(DccTheme.W_LEFT_DOCK_MIN), float(DccTheme.W_LEFT_DOCK_MAX))
			left_dock.custom_minimum_size.x = _left_width
		else:
			_right_width = clampf(_right_width + delta,
				float(DccTheme.W_RIGHT_DOCK_MIN), float(DccTheme.W_RIGHT_DOCK_MAX))
			right_dock.custom_minimum_size.x = _right_width

func _collapse_button(is_left: bool) -> Button:
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.text = DccIcons.SYMBOLS["collapse"] if is_left else DccIcons.SYMBOLS["expand"]
	b.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	b.custom_minimum_size = Vector2(_scaled(20), _scaled(20))
	b.pressed.connect(_toggle_dock.bind(is_left))
	_collapse_buttons["left" if is_left else "right"] = b
	return b

## §6's last line: "collapsed, the dock keeps its primary readout visible --
## elevation for Sample, layer dots for Layers, stamp count for the stack." So a
## collapsed dock is not an empty 40 px strip; it is a strip that still says the
## one thing you collapsed it in order to keep watching.
##
## The label lives outside the ScrollContainer precisely because collapsing
## hides that container -- putting the readout inside it would hide the thing
## the rule exists to preserve.
func _dock_readout(side: String) -> Control:
	var l := DccTheme.label("", "text_dim", DccTheme.FS_TINY)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 4)
	pad.add_theme_constant_override("margin_right", 4)
	pad.add_theme_constant_override("margin_top", 8)
	pad.add_child(l)
	pad.visible = false
	_dock_readouts[side] = l
	return pad

## Whatever the dock's current context considers its one essential number. Kept
## up to date whether or not the dock is collapsed, so collapsing never reveals
## a stale value.
func set_dock_readout(side: String, text: String) -> void:
	if not _dock_readouts.has(side):
		push_error("DccShell: no dock readout for side '%s'" % side)
		return
	(_dock_readouts[side] as Label).text = text

func is_dock_collapsed(side: String) -> bool:
	return _left_collapsed if side == "left" else _right_collapsed

## A collapsed dock shrinks to the rail width rather than disappearing, and
## swaps its body for the readout above.
func _toggle_dock(is_left: bool) -> void:
	var dock := left_dock if is_left else right_dock
	var side := "left" if is_left else "right"
	var collapsed := not (_left_collapsed if is_left else _right_collapsed)
	dock.custom_minimum_size.x = float(DccTheme.W_RAIL_COLLAPSED) if collapsed else (_left_width if is_left else _right_width)
	## `dock.get_child(0)` is the drag-handle `HBoxContainer` (`_dock_drag_handle()`
	## wraps `col` and the grip together), not the `ScrollContainer` -- which sits
	## one level deeper, inside `col`. Hiding the wrong child left the real
	## `ScrollContainer`'s content visible and its minimum size still forcing the
	## dock wider than `W_RAIL_COLLAPSED`. Go straight to the stored reference
	## instead of walking the tree.
	var scroll := _left_dock_scroll if is_left else _right_dock_scroll
	if scroll != null:
		scroll.visible = not collapsed
	(_dock_readouts[side] as Label).get_parent().visible = collapsed
	if is_left:
		## The title has no room at 40 px; the chevron is all that fits, and it
		## is the only affordance for getting the dock back.
		left_dock_title.visible = not collapsed
		_left_collapsed = collapsed
	else:
		_right_collapsed = collapsed
	var btn: Button = _collapse_buttons.get(side)
	if btn != null:
		var open_glyph: String = DccIcons.SYMBOLS["collapse"] if is_left else DccIcons.SYMBOLS["expand"]
		var shut_glyph: String = DccIcons.SYMBOLS["expand"] if is_left else DccIcons.SYMBOLS["collapse"]
		btn.text = shut_glyph if collapsed else open_glyph

# -- §9 Viewport --------------------------------------------------------------

func _build_viewport() -> Control:
	var area := PanelContainer.new()
	area.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	area.add_theme_stylebox_override("panel", DccTheme.flat(DccTheme.c("bg")))
	viewport_area = area
	viewport_content = Control.new()
	viewport_content.clip_contents = true
	area.add_child(viewport_content)
	return area

# -- §10 Timeline bar ---------------------------------------------------------

func _build_timeline() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_TIMELINE)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"top": 1}))
	timeline_row = HBoxContainer.new()
	timeline_row.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_theme_constant_override("margin_top", 8)
	pad.add_theme_constant_override("margin_bottom", 8)
	pad.add_child(timeline_row)
	bar.add_child(pad)
	return bar

# -- §11 Status bar -----------------------------------------------------------

func _build_status_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_STATUS)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel_alt", {"top": 1}))
	status_row = HBoxContainer.new()
	status_row.add_theme_constant_override("separation", 18)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(status_row)
	bar.add_child(pad)

	## `font:10.5px 'IBM Plex Mono'; color:#6f7478` on the canvas's status bar,
	## in both themes and on both the desktop and tablet artboards -- the whole
	## bar is Plex, including the modifier hints on the right, which were the
	## one thing here already drawn a shade quieter than everything else. Set
	## in the prose face at 11 until 2026-08-25.
	for slot in ["pass", "stale", "autosave", "atlas"]:
		var l := DccTheme.mono_label("", "text_faint", DccTheme.FS_SMALL, 0)
		_status_labels[slot] = l
		status_row.add_child(l)
	status_row.add_child(DccTheme.spacer())
	var hint := DccTheme.mono_label("", "text_faint", DccTheme.FS_SMALL, 0)
	_status_labels["hint"] = hint
	status_row.add_child(hint)
	return bar

## Set one status slot. Slots: pass, stale, autosave, atlas, hint, and the menu
## bar's top_world / top_pass / top_cpu / top_gpu / top_mem.
func set_status(slot: String, text: String, token: String = "text_faint") -> void:
	if not _status_labels.has(slot):
		push_error("DccShell: no status slot '%s'" % slot)
		return
	var l: Label = _status_labels[slot]
	l.text = text
	l.add_theme_color_override("font_color", DccTheme.c(token))

## Read one status slot back. `phone_menu.gd` re-presents the readout cluster
## as list rows on its root screen, because on the phone the desktop status bar
## and menu-bar readouts are parked in a hidden host rather than drawn
## (`GUI_GAP_REGISTER.md` §15 fault 2). Returns "" for a slot that has never
## been set, and for one that does not exist -- callers here are building a
## list, and an unset readout is a row that should simply not appear, not an
## error to push.
func status_slot_text(slot: String) -> String:
	if not _status_labels.has(slot):
		return ""
	return (_status_labels[slot] as Label).text

# -- §13 Phone chrome -----------------------------------------------------
#
# Phone reorganises rather than truncates: this is a distinct composition,
# not `_build_desktop_shell()` with `_scaled()` turned up further. It targets
# the same contract every workspace already depends on --
# `left_dock_body`/`right_dock_body` (workspaces attach here via
# `register_workspace()` and `right_dock.gd`), `tool_options_row`
# (`set_tool_options()`), `timeline_row`, `rail_column`/`_domain_buttons`/
# `_domain_marks` (`_select_domain()`) and `menu_bar_row`/`status_row`
# (`add_menu()`/`set_status()`) -- so nothing downstream of the frame needs to
# know which composition it is standing in. That contract is also this
# section's limit: the *content* those containers hold is reused verbatim,
# unchanged, from whatever `app.gd` and the workspaces already build for
# desktop/tablet. Only the frame around it differs.
#
# Z-order (back to front, matching draw order in `design/Cartalith DCC
# Shell.dc.html`'s "DCC shell android phone" screen):
#   1. The map, edge-to-edge, full rect -- underneath everything (inset rule
#      "DRAW EDGE-TO-EDGE, PAD BY INSET").
#   2. A gradient scrim over the top band (inset rule "SCRIM, NOT A BAR").
#   3. The chrome column: top safe area → app bar → [map gap] → tool sheet →
#      timeline → **L1 bottom bar** → bottom gesture inset. Wrapped in
#      `_phone_chrome_margin`, whose left margin is what shifts in landscape
#      to clear the side safe area. The bottom bar is the one part of this
#      column that comes from `design/Cartalith Android Phone.dc.html` rather
#      than the DCC shell canvas; it took the floating domain rail's place.
#   4. Overlays, all full-rect, all hidden until opened: the side safe area
#      (landscape only), the panel picker, the phone
#      menu (`phone_menu.gd`, L2-L5), and the left/right dock sheets.
#
# What this section does NOT build, named rather than silently skipped
# (`menus.gd`'s own discipline for what it can't honour):
#   - Any slide/drag animation. The mockup pictures exactly one static sheet
#     state; nothing here answers a drag gesture on the tool-sheet handle or
#     the gesture-inset handle -- both are decorative, matching what's shown.
#   - Touch-pan-while-drawing (v2.10's `#sculptNavpad` precedent, §13 alludes
#     to it via the sculpt tool options the phone tool sheet would host).
#     `main.gd` carries no such handling to port forward -- grepped for
#     sculpt/navpad/joystick/pan and found nothing -- so this is a genuine
#     gap for whoever wires sculpt-tool touch input, not a chrome omission.
#   - The mockup's decorative notch/punch-hole graphic (dashed box + dot,
#     lines 1452-1455 of the mockup). That reads as a mockup-authoring aid
#     showing where a *real* device's hardware cutout sits, not something a
#     shipped app should paint a fake copy of over an arbitrary point on an
#     arbitrary screen. The 108 px keep-clear reserve is honoured -- nothing
#     is ever placed there -- just not decorated.

func _build_phone_shell() -> void:
	_phone_root = Control.new()
	_phone_root.set_anchors_preset(Control.PRESET_FULL_RECT)
	_phone_root.mouse_filter = Control.MOUSE_FILTER_PASS
	add_child(_phone_root)

	var vp := _build_viewport()
	vp.set_anchors_preset(Control.PRESET_FULL_RECT)
	_phone_root.add_child(vp)

	## No gradient scrim any more: the 412 canvas paints a **solid ground** above
	## the map (`background:#101112` on the screen, the app bar carrying only a
	## `border-bottom`), not a fade. The status row builds its own opaque panel;
	## see `_build_phone_top_safe()`.

	## Menu bar and status bar keep their exact desktop construction --
	## `add_menu()`/`set_status()` stay phone-unaware -- and are parked in a
	## hidden host. They are the **model**, not a view: `menu_bar_row` is where
	## `add_menu()` puts the seven real `MenuButton`s and `_status_labels` is
	## where `set_status()` writes, and `phone_menu.gd` reads both. Nothing here
	## is drawn.
	##
	## §15 fault 2 was that these two bars were previously *reparented into the
	## phone sheet whole* -- a 150 px desktop wordmark and five readouts that are
	## empty before a generation, squeezing the menu row into a bottom strip.
	## Parking them is the fix; the readouts come back as real rows on the
	## menu's own root screen.
	##
	## Still built *first*, for the reason it always was: both this and the app
	## bar register a Label under the "top_world" status slot, `_status_labels`
	## keeps only the most recently registered one, and the app bar's subtitle
	## is the one that must win. That ordering is load-bearing.
	var model_host := Control.new()
	model_host.name = "PhoneMenuModel"
	model_host.visible = false
	model_host.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_phone_root.add_child(model_host)
	model_host.add_child(_build_menu_bar())
	model_host.add_child(_build_status_bar())

	## Both of these cover the whole screen, and a `Control` picks by default
	## (`MOUSE_FILTER_STOP`) -- so as pure layout scaffolding they were each,
	## on their own, enough to keep every tap off the map underneath. See
	## `_phone_content_gap`'s own comment below for the full diagnosis; these
	## two are the same bug one and two levels up, and all three had to go for
	## `map_overlay.gd` to see a finger.
	##
	## Their children (the app bar, the tool sheet, the bottom menu bar) are
	## picked independently of their parent's filter, so nothing tappable is
	## lost by taking the containers themselves out of picking.
	_phone_chrome_margin = MarginContainer.new()
	_phone_chrome_margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	_phone_chrome_margin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_phone_root.add_child(_phone_chrome_margin)

	var chrome := VBoxContainer.new()
	chrome.add_theme_constant_override("separation", 0)
	chrome.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_phone_chrome_margin.add_child(chrome)
	## Held because landscape lifts two of this column's children out of it --
	## the bottom bar becomes a left rail and the tool sheet docks right -- and
	## portrait has to put them back in the right slots. See
	## `_apply_phone_orientation()`.
	_phone_chrome_col = chrome

	_phone_top_safe = _build_phone_top_safe()
	chrome.add_child(_phone_top_safe)

	_phone_app_bar = _build_phone_app_bar()
	chrome.add_child(_phone_app_bar)

	## The gap between the app bar and the tool sheet: nothing but map. The
	## floating domain rail used to sit in it; the canvas moved the domains to
	## the bottom bar, so the map now has the whole width back.
	_phone_content_gap = Control.new()
	_phone_content_gap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	## **`IGNORE`, not `PASS`.** This was `PASS`, and `PASS` does not mean what
	## it reads like: a `PASS` control is still *picked* -- it receives the
	## event and then forwards it to its own **parent**, never to the nodes
	## behind it. So this spacer, which expands to fill the entire region
	## between the app bar and the tool sheet, was the control under every tap
	## on the map, and `map_overlay.gd`'s `_gui_input()` had never once run on a
	## phone. Only `IGNORE` takes a control out of picking so what is behind it
	## is reached.
	##
	## Everything on `map_overlay` that a finger should be able to do was dead
	## because of this one enum: tap-to-select a settlement, every registered
	## tool click/drag/release handler (Settlement, Territory, Way, Route,
	## Measure, and Sculpt/Paint dabs), and the press-and-hold that opens the
	## civ context sheet. It looked like the map worked because *camera* pan and
	## pinch come through `ViewportHost._input()`, which is a raw input hook and
	## never consults a `mouse_filter` at all -- so the half that was broken was
	## exactly the half nobody had driven on the device.
	##
	## Found with `gui_get_hovered_control()` over the map centre in a
	## `--force-touch` run: it named this node, not the overlay.
	##
	## Children are picked on their own, so the floating rail this hosts is
	## unaffected -- `IGNORE` on a parent does not disable its children.
	_phone_content_gap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	chrome.add_child(_phone_content_gap)

	_phone_tool_sheet = _build_phone_tool_sheet()
	chrome.add_child(_phone_tool_sheet)

	timeline_bar = _build_timeline()
	chrome.add_child(timeline_bar)

	_phone_menu_bar = _build_phone_menu_bar()
	chrome.add_child(_phone_menu_bar)

	_phone_gesture_inset = _build_phone_gesture_inset()
	chrome.add_child(_phone_gesture_inset)

	_phone_side_safe = _build_phone_side_safe()
	_phone_root.add_child(_phone_side_safe)

	_phone_panel_picker = _build_phone_panel_picker()
	_phone_root.add_child(_phone_panel_picker)

	## L2-L5. Added after the panel picker so it draws over it, and
	## before the dock sheets so a dock sheet still wins -- the same
	## mutually-exclusive rule `_close_all_phone_overlays()` enforces anyway.
	_phone_menu = PhoneMenu.new()
	_phone_root.add_child(_phone_menu)
	_phone_menu.setup(self)

	## Full-height sheets (§13), built by the exact same functions the
	## desktop/tablet dock uses -- `as_sheet = true` only swaps the header's
	## collapse chevron for a close button and drops the fixed desktop width.
	## `left_dock_body`/`right_dock_body` (what every workspace and
	## `right_dock.gd` actually attach content to) are unaffected either way.
	_phone_root.add_child(_build_left_dock(true))
	left_dock.set_anchors_preset(Control.PRESET_FULL_RECT)
	left_dock.visible = false
	_phone_root.add_child(_build_right_dock(true))
	right_dock.set_anchors_preset(Control.PRESET_FULL_RECT)
	right_dock.visible = false

	## Connected only once both docks exist, so `_on_phone_node_added()` never
	## walks toward a null. Every workspace panel is attached after this point,
	## which is exactly what it is here to catch.
	get_tree().node_added.connect(_on_phone_node_added)

	_apply_phone_orientation()

## The 412 canvas's status row, verbatim: `height:28px;padding:0 16px;
## font:10px 'IBM Plex Mono';color:#8d9296`, clock left, `LTE ▮▮ 84%` right at
## `letter-spacing:.14em`, and a **solid** ground rather than §13's gradient
## scrim over the map.
##
## What this used to be was §13's 44 dp *keep-clear reserve* with a 96 dp
## gradient behind it and a 108 dp centre lane nothing was allowed into. The
## newer canvas draws none of the three: 28 dp, edge to edge, opaque. The lane
## survives only in landscape, where no canvas exists -- see `W_PHONE_CUTOUT`.
func _build_phone_top_safe() -> Control:
	var ground := PanelContainer.new()
	ground.add_theme_stylebox_override("panel", DccTheme.panel("panel"))
	ground.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(16))
	pad.add_theme_constant_override("margin_right", _pscale(16))
	pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	ground.add_child(pad)
	var row := HBoxContainer.new()
	row.custom_minimum_size.y = _pscale(DccTheme.H_PHONE_TOP_SAFE)
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	pad.add_child(row)

	## Both spans are `#8d9296` in the canvas -- the right one was `text_faint`
	## here, one ink step quiet, and both were 11 px against the canvas's 10.
	_phone_clock_label = DccTheme.mono_label("", "text_dim", _pscale(10))
	_phone_clock_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(_phone_clock_label)
	row.add_child(DccTheme.spacer())
	_phone_battery_label = DccTheme.mono_label("", "text_dim", _pscale(10), 1)
	_phone_battery_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(_phone_battery_label)

	var timer := Timer.new()
	timer.wait_time = 30.0
	timer.autostart = true
	timer.timeout.connect(_refresh_phone_status_glyphs)
	ground.add_child(timer)
	_refresh_phone_status_glyphs()
	return ground

## The clock is the real system time (`Time`, not the mockup's static "9:41").
## Battery/signal/Wi-Fi stay the mockup's own decorative placeholder glyphs --
## checked against this Godot build's own `OS` class (`ClassDB.class_get_method_list`)
## rather than assumed: there is no `power`/`battery` method on it at all, so
## there is nothing real to back these three with cross-platform. Only the
## clock gets the honest-data treatment.
func _refresh_phone_status_glyphs() -> void:
	var t := Time.get_time_dict_from_system()
	var clock_text := "%02d:%02d" % [int(t["hour"]), int(t["minute"])]
	var battery_text := "▲ ▮▮ --"
	if _phone_clock_label != null:
		_phone_clock_label.text = clock_text
	if _phone_battery_label != null:
		_phone_battery_label.text = battery_text
	if _phone_side_clock_label != null:
		_phone_side_clock_label.text = clock_text
	if _phone_side_battery_label != null:
		_phone_side_battery_label.text = battery_text

## Landscape's side safe area (inset rule "LANDSCAPE": "the cutout moves to a
## side edge: apply the same reserve horizontally"). Judgment call, undocumented
## by the mockup (it only pictures the portrait screen): the cutout is placed
## on the *left* edge, and the portrait top row's "left/right pockets" become
## this column's "top/bottom pockets" -- the same pocket structure, rotated,
## rather than a different rule invented for landscape. The rail "shifts
## inward" (same inset rule) for free: it floats inside `_phone_chrome_margin`,
## whose left margin grows by this column's width in `_apply_phone_orientation()`.
func _build_phone_side_safe() -> Control:
	var wrap := Control.new()
	wrap.set_anchors_preset(Control.PRESET_LEFT_WIDE)
	wrap.offset_left = 0
	wrap.offset_right = _pscale(DccTheme.H_PHONE_TOP_SAFE)
	wrap.offset_top = 0
	wrap.offset_bottom = 0
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.visible = false

	var bg := ColorRect.new()
	bg.color = Color(DccTheme.c("bg"), 0.9)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(bg)

	var col := VBoxContainer.new()
	col.set_anchors_preset(Control.PRESET_FULL_RECT)
	col.add_theme_constant_override("separation", 0)
	wrap.add_child(col)

	var top_pad := MarginContainer.new()
	top_pad.add_theme_constant_override("margin_top", _pscale(10))
	_phone_side_clock_label = DccTheme.mono_label("", "text_dim", _pscale(10))
	_phone_side_clock_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	top_pad.add_child(_phone_side_clock_label)
	col.add_child(top_pad)
	col.add_child(DccTheme.spacer())
	## The keep-clear reserve -- see the "notch graphic" note in the section
	## header comment above: nothing is placed here, deliberately undecorated.
	var dead := Control.new()
	dead.custom_minimum_size.y = _pscale(DccTheme.W_PHONE_CUTOUT)
	col.add_child(dead)
	col.add_child(DccTheme.spacer())
	var bot_pad := MarginContainer.new()
	bot_pad.add_theme_constant_override("margin_bottom", _pscale(10))
	_phone_side_battery_label = DccTheme.mono_label("", "text_faint", _pscale(9))
	_phone_side_battery_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	bot_pad.add_child(_phone_side_battery_label)
	col.add_child(bot_pad)
	return wrap

## The app bar. `design/Cartalith Android Phone.dc.html` screen 01:
## `height:56px;display:flex;align-items:center;gap:14px;padding:0 12px;
## border-bottom:1px solid rgba(255,255,255,.09)`, carrying `☰` (16 px) / title
## over seed / `⌕` / `⋮` in 40 dp cells.
##
## Two of the canvas's four cells are not built, each for a stated reason:
##
##   - **`⋮`** is drawn in the app bar *and* as the bottom bar's fifth tab, and
##     that canvas's own note ("More is a grouped list, not a duplicate menu
##     bar") rules out carrying one destination twice. In the canvas the bar's
##     `⋮` is a *contextual* overflow -- it reappears on the L2 and L3 drill
##     headers, where it can only mean "this screen's own menu". This shell has
##     no per-screen overflow to put behind it, and a connected affordance with
##     nothing behind it is exactly what `GUI_GAP_REGISTER.md` exists to catch.
##   - **`⌕`** has no destination. `menus.gd`'s Edit ▸ Find on map… is a
##     `_todo()` row -- disabled, with "no search index yet" as its reason --
##     and there is no other map search in this build. A magnifier that opens
##     a disabled menu item is worse than no magnifier. Registered rather than
##     drawn; the moment a search index exists this is a three-line addition.
##   - **`▤`** was here and is the bottom bar's PANELS tab now.
func _build_phone_app_bar() -> PanelContainer:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _ptap(DccTheme.H_PHONE_APP_BAR)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", _pscale(14))
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(12))
	pad.add_theme_constant_override("margin_right", _pscale(12))
	pad.add_child(row)
	bar.add_child(pad)

	## The 412 canvas has **no side drawer**: its `02 Domain` screen is a
	## full-screen drill with a `←`, and the shell's own full-height left dock
	## sheet is that screen. So `☰` opens the sheet directly, and the 300 dp side
	## sheet that used to list the three domains with their subtitles is gone --
	## the bottom bar's three domain cells are the same three destinations, now
	## with a glyph each, and carrying them twice was the duplication the canvas
	## rules out.
	row.add_child(_phone_bar_button(DccIcons.SYMBOLS["drawer"], "Domain panel",
		func(): _set_sheet_open("left", true)))

	var title_col := VBoxContainer.new()
	title_col.add_theme_constant_override("separation", 0)
	title_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_col.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	## `font:500 12px 'IBM Plex Mono';letter-spacing:.2em;color:#e8ebec` -- .2em
	## of 12 px is 2.4 px, and `spacing_glyph` is whole pixels, so 2. This was 3.
	title_col.add_child(DccTheme.mono_label("CARTALITH", "text_bright", _pscale(12), 2, true))
	## Reuses the same "top_world" status slot the desktop menu bar's readout
	## cluster fills (`_wire_status()` in `app.gd` calls
	## `set_status("top_world", "ELDRA · %d" % seed)`) -- no phone-aware
	## branch needed in `app.gd` for this to stay live. `font:10px 'IBM Plex
	## Mono';color:#6f7478`, untracked in the canvas.
	var subtitle := DccTheme.mono_label("", "text_faint", _pscale(10), 0)
	_status_labels["top_world"] = subtitle
	title_col.add_child(subtitle)
	row.add_child(title_col)

	## **`▤` panels, restoring access the four-tab bar took away.**
	## `_phone_panel_picker` is how the phone reaches the left and right dock
	## content, and its ONLY entry used to be the PANELS cell in the bottom
	## bar. Replacing that bar with MAP/GENERATE/PLAN/MORE left the picker
	## built, alive and unreachable -- built-and-unwired, the defect class this
	## repository keeps finding, introduced by me this session and caught on the
	## device rather than by reading.
	##
	## `DCC_SHELL_SPEC.md` §13 puts it here anyway: the phone app bar is
	## "☰ (domain drawer), title + seed, ▤ (panels), ⋯ (overflow menu)". So the
	## fix and the spec agree.
	row.add_child(_phone_bar_button(DccIcons.SYMBOLS["panels"], "Panels",
		func(): _set_panel_picker_open(true)))
	return bar

## One app-bar glyph cell. The canvas draws a `40x40` box at `color:#c8cbcd`
## (`text`, not the `text_dim` this used) with `font:16px 'IBM Plex Mono'`; the
## box is a *layout* cell with no background, so the hit target still floors at
## the TARGETS card's 44 dp rather than shrinking to the drawn 40.
##
## **Not `flat`.** A `Button` with `flat = true` skips its `normal`/`hover`/
## `pressed` styleboxes outright, so the press feedback on the last two lines
## had never once appeared -- the fourth site of the trap `GUI_GAP_REGISTER.md`
## MN-13 found in three others, and the one on the phone's most-tapped control.
func _phone_bar_button(glyph: String, tip: String, on_press: Callable,
		token: String = "text") -> Button:
	var b := Button.new()
	b.text = glyph
	b.flat = false
	b.focus_mode = Control.FOCUS_NONE
	b.tooltip_text = tip
	b.custom_minimum_size = Vector2(_ptap(DccTheme.PHONE_ICON_BOX),
		_ptap(DccTheme.PHONE_ICON_BOX))
	b.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	b.add_theme_font_size_override("font_size", _pscale(16))
	b.add_theme_font_override("font", DccTheme.mono())
	b.add_theme_color_override("font_color", DccTheme.c(token))
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.add_theme_stylebox_override("pressed", DccTheme.active_row(false))
	b.pressed.connect(on_press)
	return b

## L1: the bottom bar (`design/Cartalith Android Phone.dc.html`, artboard
## "01 · VIEWPORT" -- five equal cells, 64 dp, glyph over a 9.5 px tracked
## caption, active cell in accent).
##
## **This replaces the floating left rail.** The canvas draws no rail: it moves
## the domains to the bottom, where a thumb reaches them, and its PHONE RULES
## make the bar level 1 of the disclosure tree outright ("L1 is the bottom
## bar"). Keeping both would have put the same three domains on screen twice.
## ## The one place the two authorities split
##
## The canvas's five tabs are `WORLD · GENERATE · SIMULATE · MAP · MORE` -- the
## **pre-v3** domain set. `design/Cartalith Menu Structure v3.dc.html` is newer
## and is the authority for domain content and naming, and it has three:
## `WORLD · CIVIL · CARTO` (INFRA merged into CIVIL, RENDER into CARTO, owner
## 2026-08-20). `DCC_SHELL_SCOPE.md`'s rule 1 -- "the newer canvas wins" --
## resolves it: this bar takes **412's geometry and v3's content**. Five slots,
## v3's three domains plus the two phone affordances the canvas's own fifth tab
## and app bar establish: PANELS (both docks) and MORE (the program menu tree).
##
## `MENU` was the fifth caption and is `MORE` now, which is the canvas's word
## for that exact destination.
##
## ## The glyph row
##
## `<span style="font:14px Plex">◈</span>` over `<span style="font:9.5px Plex;
## letter-spacing:.1em">WORLD</span>`, `gap:4px`, active `#e0a34a` and resting
## `#8d9296`. The row was captions only until this pass.
##
## The five marks are **drawn, not typed**, and three of them already existed:
## `DccIcons`' `domain_world`/`domain_civ`/`domain_carto` are this design
## system's own glyphs for these exact three subjects, authored to §12's rules.
## `nav_panels` and `nav_more` are new and are *designed rather than matched*
## under `DCC_SHELL_SCOPE.md`'s rule 2 -- each traces the canvas's own chosen
## symbol (▤, ⋯) at §12's stroke.
##
## This does **not** re-open the owner's *"those icons don't exist"* ruling. That
## was about the **desktop vertical rail**, whose artboard draws `writing-mode:
## vertical-rl` captions and no icon element at all (see `_build_rail()`). This
## artboard draws a glyph over every caption, explicitly, five times.
##
## `rail_column` stays the container the domain cells sit in, so
## `set_rail_foot()`/`_select_domain()` and anything else that already knows
## that name keeps working. What `Window ▸ Domain rail` hides is `_rail_region`,
## which here is **only the three domain cells** -- PANELS and MENU stay. Hiding
## the whole bar would hide the MENU cell, and the menu is the only place the
## row that un-hides it lives: a one-way door.
func _build_phone_menu_bar() -> Control:
	var bar := PanelContainer.new()
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"top": 1}))

	## **`BoxContainer`, not `HBoxContainer`** -- for all three of the boxes this
	## bar nests, and for no other reason than landscape. `HBoxContainer` is not
	## merely a `BoxContainer` with `vertical = false` preset: its `set_vertical`
	## *refuses* the write outright (`Can't change orientation of
	## HBoxContainer`, `scene/gui/box_container.cpp`). Assuming otherwise is
	## exactly what the first run of `_apply_phone_nav_orientation()` did, three
	## errors per rotation and a rail that stayed horizontal. A bare
	## `BoxContainer` starts horizontal and lets the flip through.
	var row := BoxContainer.new()
	row.vertical = false
	row.add_theme_constant_override("separation", 0)
	bar.add_child(row)

	var domains := BoxContainer.new()
	domains.vertical = false
	domains.add_theme_constant_override("separation", 0)
	domains.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	domains.size_flags_stretch_ratio = float(PHONE_TABS.size())
	_rail_region = domains
	row.add_child(domains)

	rail_column = VBoxContainer.new()
	rail_column.add_theme_constant_override("separation", 0)
	rail_column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	domains.add_child(rail_column)

	var cells := BoxContainer.new()   ## See `row` above for why not `HBox`.
	cells.vertical = false
	cells.add_theme_constant_override("separation", 0)
	rail_column.add_child(cells)

	## Landscape turns this bar on its side (`docs/ANDROID_UI_SPEC.md`: "nav
	## becomes left rail"). Every one of these three is an `HBoxContainer`,
	## which in Godot 4 is a `BoxContainer` with `vertical = false` -- so the
	## rotation is a property flip on the *same nodes*, not a second bar built
	## alongside this one. Held here because `_apply_phone_orientation()` has no
	## other route to them: `rail_column` is a documented public-ish name that
	## `set_rail_foot()`/`_select_domain()` already rely on, and the other three
	## were locals.
	_phone_bar_row = row
	_phone_bar_domains = domains
	_phone_bar_cells = cells

	## **`PHONE_TABS`, not `DOMAINS`.** The bar used to mirror the desktop's
	## three workspaces plus PANELS and MORE, which is how a phone ended up
	## with two tabs -- "PANELS" and "MORE" -- that name a container rather
	## than a destination. `docs/ANDROID_UI_SPEC.md` replaces them with four
	## task tabs, and CIVIL moves under MORE, which that spec states
	## explicitly ("MORE: Project, Civilization ..., Data manager, ...").
	for t in PHONE_TABS:
		var cell := _phone_bar_cell(String(t.caption), String(t.icon), String(t.tip),
			_pick_phone_tab.bind(String(t.id)))
		var key := String(t.id)
		_phone_tab_cells[key] = cell
		if String(t.domain) != "":
			_domain_buttons[String(t.domain)] = cell["button"]
			_domain_marks[String(t.domain)] = {"label": cell["label"], "icon": cell["icon"],
				"off": "text_dim", "box": false}
		cells.add_child(cell["button"] as Control)
	return bar

## The phone's four task tabs (`docs/ANDROID_UI_SPEC.md`: "bottom bar, task tabs
## MAP · GENERATE · PLAN · MORE").
##
## `domain` is the desktop workspace a tab selects, or `""` for a tab that is
## not a workspace at all. Two of the four map straight onto existing domains;
## PLAN opens the journey planner, and MORE is the overflow screen.
##
## `civilization` is deliberately absent as a *tab* and still fully reachable --
## the planner selects it, and MORE lists it. The spec moved it there rather
## than dropping it.
const PHONE_TABS: Array = [
	{"id": "map", "caption": "MAP", "icon": "domain_carto", "domain": "cartography",
		"tip": "Layers, style and annotation"},
	{"id": "gen", "caption": "GENERATE", "icon": "domain_world", "domain": "world",
		"tip": "The generation pipeline, and Sculpt"},
	{"id": "plan", "caption": "PLAN", "icon": "tool_route", "domain": "",
		"tip": "Journey planner"},
	{"id": "more", "caption": "MORE", "icon": "nav_more", "domain": "",
		"tip": "Project, Civilization, Data, Assets, Preferences, Help"},
]

var _phone_tab_cells: Dictionary = {}
var _phone_tab := "gen"   ## Which of `PHONE_TABS` is lit.

## One tab press. A workspace tab selects its domain; the two that are not
## workspaces do their own thing.
func _pick_phone_tab(id: String) -> void:
	var was_tab := _phone_tab
	_phone_tab = id
	match id:
		"more":
			_toggle_overflow()
		"plan":
			if _phone_menu != null and _phone_menu.is_open():
				_phone_menu.close()
			## `DccApp extends DccShell`, so `self` is the app; the planner
			## opener lives on the subclass. Guarded rather than assumed,
			## because `DccShell` is also instantiated bare by probes.
			if has_method("open_journey_planner"):
				call("open_journey_planner")
		_:
			if _phone_menu != null and _phone_menu.is_open():
				_phone_menu.close()
			for t in PHONE_TABS:
				if String(t.id) == id and String(t.domain) != "":
					_pick_bar_domain(String(t.domain))
	## "tab tap opens half" (`docs/ANDROID_UI_SPEC.md`). The prototype's `hTab`
	## is `{tab:t, detent: s.detent==='peek' ? 'half' : s.detent}` -- a tap
	## *lifts* a peeking sheet to half and leaves half and full where the user
	## put them, so switching tabs never shrinks a sheet someone has just pulled
	## open. Re-tapping the lit tab closes it there; here it collapses to peek
	## instead, for the reason `_on_phone_sheet_grab_input()` sets out at
	## length: this sheet is the tool options bar, and it has no "gone" state on
	## the other two form factors.
	##
	## Only the two *workspace* tabs drive the detent. In the prototype all four
	## tabs fill the one sheet, so all four move it; here PLAN opens the journey
	## planner and MORE opens `PhoneMenu`, both full overlays over the sheet
	## rather than content inside it. Lifting a peeking sheet behind an overlay
	## the user cannot see through would only surface after they closed it
	## again, as a sheet that had grown while they were elsewhere.
	if _phone_tab_drives_sheet(id):
		if was_tab == id and _phone_detent != "peek":
			_set_phone_detent("peek")
		elif _phone_detent == "peek":
			_set_phone_detent("half")
	_refresh_phone_tabs()

## Whether a tab's destination is the tool sheet itself. `domain` is `""` for
## exactly the two tabs that open an overlay instead -- see `PHONE_TABS`.
func _phone_tab_drives_sheet(id: String) -> bool:
	for t in PHONE_TABS:
		if String(t.id) == id:
			return String(t.domain) != ""
	return false

## The active tab wears the candidate's own pill -- `padding:5px 16px;
## border-radius:14px; background:rgba(224,163,74,.16)` behind the glyph
## (`candidates/Android Chrome B.dc.html`). Lighting only the caption, which is
## what the old bar did, left the row reading as four labels of equal weight.
func _refresh_phone_tabs() -> void:
	for key in _phone_tab_cells.keys():
		var cell: Dictionary = _phone_tab_cells[key]
		var on: bool = (String(key) == _phone_tab)
		var lbl: Label = cell.get("label")
		var pill: PanelContainer = cell.get("pill")
		if lbl != null and is_instance_valid(lbl):
			(lbl as Label).add_theme_color_override("font_color",
				DccTheme.c("accent" if on else "text_dim"))
		if pill != null and is_instance_valid(pill):
			## `rgba(224,163,74,.16)` verbatim from the candidate, NOT the
			## `accent_wash` token. That token is 8% (`#e0a34a14`), which is
			## the desktop's active-menu wash and is effectively invisible
			## behind a 14 px glyph on `#121314` -- checked on the handset, the
			## pill did not read at all at 8%. The candidate chose twice that
			## for this surface and it is right for it.
			var box := DccTheme.flat(Color(DccTheme.c("accent"), 0.16))
			box.set_corner_radius_all(_pscale(14))
			(pill as PanelContainer).add_theme_stylebox_override("panel",
				box if on else DccTheme.empty())


## Which of the two non-domain tabs is lit. The three domain cells go through
## `_select_domain()`; these two have no domain to select, and without this the
## bar said WORLD while the MORE screen was on top of it -- a tab bar naming a
## destination the user is not at.
##
## Read off live state rather than tracked, so every opener and
## `_close_all_phone_overlays()` need only call it, in any order.
func _refresh_phone_bar_lit() -> void:
	if _phone_bar_more.is_empty():
		return
	var more_on: bool = _phone_menu != null and _phone_menu.is_open()
	var panels_on: bool = _left_sheet_open or _right_sheet_open \
		or (_phone_panel_picker != null and _phone_panel_picker.visible)
	_light_bar_cell(_phone_bar_more, more_on)
	_light_bar_cell(_phone_bar_panels, panels_on)

func _light_bar_cell(cell: Dictionary, on: bool) -> void:
	var col := DccTheme.c("accent") if on else DccTheme.c("text_dim")
	(cell["label"] as Label).add_theme_color_override("font_color", col)
	(cell["icon"] as CanvasItem).modulate = col

## One bar cell: a `14px` glyph over a `9.5px/.1em` caption with `gap:4px`,
## centred in a 64 dp cell. Returns all three nodes because `_select_domain()`
## recolours the caption *and* the glyph, and only the domain cells register
## there.
func _phone_bar_cell(caption: String, glyph: String, tip: String,
		on_press: Callable) -> Dictionary:
	var b := Button.new()
	b.tooltip_text = tip
	## Not flat: a flat `Button` draws no stylebox, so both boxes below would be
	## comments. Same trap as `add_menu()`, `_build_rail()` and
	## `_phone_list_row()` -- see `GUI_GAP_REGISTER.md` MN-13.
	b.flat = false
	b.focus_mode = Control.FOCUS_NONE
	## Canvas: a 64 dp bar. `_ptap` floors it at the 44 px minimum and scales it
	## with everything else, so this is the same target arithmetic the app bar's
	## own buttons use -- not a second set of numbers.
	b.custom_minimum_size.y = _ptap(DccTheme.H_PHONE_BOTTOM_NAV)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.add_theme_stylebox_override("pressed", DccTheme.active_row(false))
	b.pressed.connect(on_press)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", _pscale(4))
	col.alignment = BoxContainer.ALIGNMENT_CENTER
	col.mouse_filter = Control.MOUSE_FILTER_IGNORE

	## `_pscale`d, not the raw 14: the main viewport has no content scale, so a
	## 14 px glyph would be 14 *physical* px -- under a millimetre on a 510 ppi
	## panel. `DccIcons.rect()` reads its own magnification off the canvas
	## transform, which here is 1, so this is the real raster size too.
	var ic := DccIcons.rect(glyph, _pscale(14), "text_dim")
	ic.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	## The glyph sits in a pill that only the ACTIVE tab fills --
	## `candidates/Android Chrome B.dc.html`: `padding:5px 16px;
	## border-radius:14px; background:rgba(224,163,74,.16)`. Lighting only the
	## caption (what the bar did before) left four labels of equal weight and no
	## sense of where you are. Empty stylebox until `_refresh_phone_tabs()`
	## fills it, so an inactive tab is byte-identical to what it drew before.
	var pill := PanelContainer.new()
	pill.add_theme_stylebox_override("panel", DccTheme.empty())
	pill.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	pill.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var pill_pad := MarginContainer.new()
	pill_pad.add_theme_constant_override("margin_left", _pscale(16))
	pill_pad.add_theme_constant_override("margin_right", _pscale(16))
	pill_pad.add_theme_constant_override("margin_top", _pscale(5))
	pill_pad.add_theme_constant_override("margin_bottom", _pscale(5))
	pill_pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	pill_pad.add_child(ic)
	pill.add_child(pill_pad)
	col.add_child(pill)

	## `9.5px` with `.1em` tracking -- just under 1 px at that size, so `spacing`
	## is 1. This was 9 px at 2 (≈.22em) in Medium: a size down, tracking up and
	## weight up all at once, the same three-error compound `_build_rail()`
	## records for the desktop rail's own caption.
	var l := DccTheme.mono_label(caption.to_upper(), "text_dim", _pscale(9.5), 1, false)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(l)

	b.add_child(col)
	col.set_anchors_preset(Control.PRESET_FULL_RECT)
	return {"button": b, "label": l, "icon": ic, "pill": pill}

## A bar domain was tapped: switch domain and drop any menu that was over it,
## so the result is visible immediately -- the canvas's "the map never leaves
## the screen" rule.
func _pick_bar_domain(id: String) -> void:
	_close_all_phone_overlays()
	_select_domain(id)

## §13: "tool options become a bottom sheet", now with
## `docs/ANDROID_UI_SPEC.md`'s three detents behind it.
##
## The handle used to be decorative, with a comment saying so: the 412 canvas
## pictured one static sheet state and answering a drag would have been invented
## behaviour. `docs/ANDROID_UI_SPEC.md` and the interactive prototype it ships
## with now specify the gesture exactly, so the handle is live -- see
## `_on_phone_sheet_grab_input()` for the drag and `_phone_detent_height()` for
## the three heights.
##
## `tool_options_row` is the same `HBoxContainer` `set_tool_options()` already
## rebuilds from `app.gd` -- wrapped in a `ScrollContainer` here because its
## desktop-tuned content (a run-pipeline row with several buttons and spacers)
## is wider than 412 dp and would otherwise clip.
func _build_phone_tool_sheet() -> PanelContainer:
	var sheet := PanelContainer.new()
	sheet.add_theme_stylebox_override("panel", _phone_sheet_box(false))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	sheet.add_child(col)

	## The grab handle -- and, as of the detents, the drag target for them.
	## `MOUSE_FILTER_STOP` (a bare `Control`'s default, set explicitly because
	## it is now load-bearing rather than incidental) so the row picks the
	## press; everything else in the phone chrome that is *not* meant to pick
	## is `IGNORE` for the reason `_phone_content_gap` documents at length.
	_phone_sheet_grab = Control.new()
	## `height:24px` with a `40x4` radius-2 bar
	## (`candidates/Android Chrome B.dc.html`, the sheet's own first row). This
	## was `20`/`34x4` from `design/Cartalith Android Phone.dc.html`; the
	## candidate is the newer canvas and `CLAUDE.md`'s first working rule gives
	## it the disagreement.
	_phone_sheet_grab.custom_minimum_size.y = _pscale(24)
	_phone_sheet_grab.mouse_filter = Control.MOUSE_FILTER_STOP
	_phone_sheet_grab.gui_input.connect(_on_phone_sheet_grab_input)
	var handle := ColorRect.new()
	## Token-derived, not a literal white: `DccTheme.remap()` can only repaint a
	## colour it can trace back to a token, so a flat `Color(1,1,1,0.25)` here
	## would stay white when the palette goes light and vanish into the panel.
	## The candidate's `rgba(255,255,255,.25)` is therefore matched in *weight*
	## rather than in value -- `text_ghost` at the alpha this handle already
	## carried, unchanged; only its geometry moved to the candidate's.
	handle.color = Color(DccTheme.c("text_ghost"), 0.55)
	var hw := _pscale(40)
	var hh := _pscale(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle.mouse_filter = Control.MOUSE_FILTER_IGNORE  ## The bar must not eat
		## the press its own row is there to receive.
	_phone_sheet_grab.add_child(handle)
	col.add_child(_phone_sheet_grab)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	## Was `DISABLED`, on the premise that the sheet always hugs its one content
	## row. A detent sets the height instead, so at `peek` (66 dp, of which the
	## handle row takes 24) that row no longer fits and needs somewhere to go.
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	## And it must take the height the detent hands the sheet, rather than its
	## own minimum -- otherwise `half` and `full` draw a tall empty panel with
	## the content still crushed against the handle.
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	tool_options_row = HBoxContainer.new()
	## Scaled like everything else in the sheet: left unscaled these read as a
	## hairline against `_phone_fit_tool_options()`-sized controls, which is
	## what put the first control flush against the screen edge on the device.
	tool_options_row.add_theme_constant_override("separation", _pscale(14))
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(14))
	pad.add_theme_constant_override("margin_right", _pscale(14))
	pad.add_theme_constant_override("margin_bottom", _pscale(10))
	pad.add_child(tool_options_row)
	scroll.add_child(pad)
	col.add_child(scroll)
	return sheet

## The sheet surface. Portrait: `background:#15171a; border-radius:22px 22px 0 0`
## (`candidates/Android Chrome B.dc.html`, and the prototype's own
## `sheetStyle`). Landscape: `border-radius:0` with `border-left:1px solid
## var(--hair)`, because a sheet docked to the right edge is a dock, not a
## sheet, and the prototype drops the radius there explicitly.
##
## `#15171a` is drawn as the `raised` token (`#17191a`) rather than a twelfth
## near-identical literal -- the accumulation `GUI_GAP_REGISTER.md` §48 exists
## because of -- and `raised` is already this palette's "anything floating",
## which is exactly what a sheet over the map is.
##
## The shadow is the candidate's `box-shadow:0 -8px 24px rgba(0,0,0,.35)`, all
## three values verbatim. It is an **approximation, not a match**:
## `StyleBoxFlat`'s shadow is an expanded, anti-aliased copy of the box, not a
## gaussian blur, so 24 px of it reads as a firmer edge than CSS's would. Both
## are only there to lift the sheet off the map, which it does; said plainly
## because everything else on this surface is exact.
func _phone_sheet_box(docked: bool) -> StyleBoxFlat:
	var box := DccTheme.panel("raised", {"left": 1} if docked else {"top": 1})
	if not docked:
		box.corner_radius_top_left = _pscale(22)
		box.corner_radius_top_right = _pscale(22)
	box.shadow_color = Color(0, 0, 0, 0.35)
	box.shadow_size = _pscale(24)
	box.shadow_offset = Vector2(0, -_pscale(8))
	return box

## What the bottom of the screen owes to chrome that is *not* the sheet: the
## gesture inset, the bottom bar, and (this shell's own addition, which the
## prototype has no row for) the timeline when it is up. The prototype folds the
## first two into one `navH = 84` literal.
func _phone_nav_reserve() -> float:
	var r := float(_pscale(DccTheme.H_PHONE_GESTURE))
	if _phone_menu_bar != null and _phone_menu_bar.visible:
		## `_ptap()` rather than the bar's measured `size.y`, for the same
		## reason `_apply_phone_orientation()` gives: this can run before the
		## first layout pass, where that is still zero.
		r += float(_ptap(DccTheme.H_PHONE_BOTTOM_NAV))
	if timeline_bar != null and timeline_bar.visible:
		r += timeline_bar.size.y
	return r

## The three detent heights, from the prototype's `_detH()` -- see the constant
## block above for the transcription. Landscape has no detents (the sheet is
## width-driven there), so this is portrait-only and its callers all guard.
func _phone_detent_height(det: String) -> float:
	var fh: float = get_viewport_rect().size.y - _phone_nav_reserve()
	var peek := float(_pscale(PHONE_DETENT_PEEK))
	match det:
		"peek":
			return peek
		"full":
			## `maxf` guards the degenerate small-window case (a `--force-touch`
			## desktop probe at a few hundred px) where `fh - 96` would come out
			## under the peek height and the sheet would *shrink* on "full".
			return maxf(peek, fh - float(_pscale(PHONE_DETENT_FULL_GAP)))
		_:
			return maxf(peek, round(fh * PHONE_DETENT_HALF_FRAC))

## Move to a detent. `animate` is false for the two cases where a transition
## would be wrong: the initial layout, and a rotation (where the whole chrome
## re-lays out in one frame anyway).
func _set_phone_detent(det: String, animate: bool = true) -> void:
	_phone_detent = det
	_snap_phone_sheet(animate)

## The prototype's `_snapSheet()`: writes the current detent's height onto the
## sheet, and does nothing at all in landscape.
##
## `custom_minimum_size.y` is the height here because the sheet is a child of a
## `VBoxContainer` -- the container owns the rect, and a minimum is the only
## thing a child gets to say about it. `_phone_content_gap` above it carries
## `SIZE_EXPAND_FILL`, so every pixel the sheet claims comes out of the map gap
## and none of it out of the bar below.
func _snap_phone_sheet(animate: bool) -> void:
	if _phone_tool_sheet == null or _landscape:
		return
	var target := _phone_detent_height(_phone_detent)
	if _phone_sheet_tween != null and _phone_sheet_tween.is_valid():
		_phone_sheet_tween.kill()
	if not animate:
		_phone_tool_sheet.custom_minimum_size.y = target
		phone_insets_changed.emit()
		return
	_phone_sheet_tween = create_tween()
	_phone_sheet_tween.set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	_phone_sheet_tween.tween_property(_phone_tool_sheet,
		"custom_minimum_size:y", target, PHONE_DETENT_ANIM)
	## Fired once, at rest, rather than per-frame: `phone_insets_changed` makes
	## `ViewportHost` re-read `phone_content_insets()` and re-place its floating
	## chrome, and doing that on every tween frame would animate the scale bar
	## and the coordinate readout along with the sheet for no stated reason.
	_phone_sheet_tween.finished.connect(func() -> void: phone_insets_changed.emit())

## One drag on the sheet's grab handle -- the prototype's `_sd`/`_sm`/`_su`
## triple:
##
## - press: record the finger's y and the height it started from, and kill the
##   height transition so the sheet tracks the finger exactly;
## - move: `h = h0 - (y - y0)`, clamped to `[40, frameH - 90]`;
## - release: snap to whichever of the three detent heights is nearest.
##
## Two departures from the prototype, both stated rather than quietly taken:
##
## 1. **Under 44 px this collapses to `peek`; the prototype closes the sheet.**
##    Its sheet is a tab panel with nothing in it when no tab is lit. This one
##    is the desktop's *tool options bar* (§13: "tool options become a bottom
##    sheet") -- the one row that is on screen at all times on desktop and
##    tablet -- so a state where it is gone entirely has no counterpart on the
##    other two form factors to keep parity with. `peek` is the closest honest
##    reading of "dismissed": still there, out of the way.
## 2. **Mouse and touch events are both handled.** Godot only synthesises mouse
##    events from touch while `emulate_mouse_from_touch` is on; it is on by
##    default, but this file may not edit `project.godot` to guarantee it, so
##    the screen events are read directly too. Handling both is safe rather
##    than double-counted: the maths is absolute (recomputed from the finger's
##    current position every event, never accumulated), so a duplicated move is
##    idempotent, a duplicated press re-records the same origin, and the second
##    of a duplicated release finds `_phone_sheet_drag` already empty.
##
## Positions are converted to *global* y. A local y would drift: this control is
## inside the sheet, and the sheet's origin moves upward as it grows, so local
## coordinates shift under a finger that has not moved.
func _on_phone_sheet_grab_input(event: InputEvent) -> void:
	if _landscape or _phone_tool_sheet == null:
		return  ## `_sd()` returns immediately in landscape too: a docked side
			## sheet has a width, not a detent.
	var press_state := 0  ## -1 release, +1 press, 0 not a press event at all.
	var moved := false
	var local_y := 0.0
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		press_state = 1 if mb.pressed else -1
		local_y = mb.position.y
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		press_state = 1 if st.pressed else -1
		local_y = st.position.y
	elif event is InputEventMouseMotion:
		moved = true
		local_y = (event as InputEventMouseMotion).position.y
	elif event is InputEventScreenDrag:
		moved = true
		local_y = (event as InputEventScreenDrag).position.y
	else:
		return

	var gy: float = _phone_sheet_grab.global_position.y + local_y
	if press_state == 1:
		if _phone_sheet_tween != null and _phone_sheet_tween.is_valid():
			_phone_sheet_tween.kill()
		_phone_sheet_drag = {"y0": gy, "h0": _phone_tool_sheet.size.y}
		_phone_sheet_grab.accept_event()
		return
	if _phone_sheet_drag.is_empty():
		return
	if moved:
		var lo := float(_pscale(PHONE_DETENT_MIN_DRAG))
		## The prototype's ceiling is `frameH - 90` on an absolutely-positioned
		## sheet, which cannot push anything: here the sheet is a row in a
		## `VBoxContainer`, and a minimum height that large would shove the
		## timeline, the bottom bar and the gesture inset off the bottom of the
		## screen mid-drag. The `full` detent's own height is the ceiling
		## instead -- 90 dp under the prototype's `frameH - 90` on a 412 x 892
		## frame (802 against 712), and it
		## makes "bar stays visible at full sheet" hold *during* the gesture and
		## not merely at rest.
		var hi: float = _phone_detent_height("full")
		var h: float = clampf(float(_phone_sheet_drag["h0"]) \
			- (gy - float(_phone_sheet_drag["y0"])), lo, maxf(lo, hi))
		_phone_sheet_drag["h"] = h
		_phone_tool_sheet.custom_minimum_size.y = h
		_phone_sheet_grab.accept_event()
		return
	## Release. `_su()`: nearest detent by absolute height difference, with the
	## prototype's own `best='half'` as the seed.
	var final_h: float = float(_phone_sheet_drag.get("h", _phone_sheet_drag["h0"]))
	_phone_sheet_drag = {}
	if final_h < float(_pscale(PHONE_DETENT_DISMISS)):
		_set_phone_detent("peek")
		return
	var best := "half"
	var best_d := INF
	for det in ["peek", "half", "full"]:
		var d: float = absf(_phone_detent_height(det) - final_h)
		if d < best_d:
			best_d = d
			best = det
	_set_phone_detent(best)

## The gesture inset: `height:20px` with a `112x4` radius-2 handle at
## `rgba(255,255,255,.22)`, on every one of the 412 canvas's eight screens.
## §13 reserved 26 dp with a 110 px handle; both figures moved.
##
## "No tappable target inside it" still holds, and `MOUSE_FILTER_IGNORE` all the
## way down enforces it structurally rather than visually -- there is nothing
## here a tap could hit even by accident.
func _build_phone_gesture_inset() -> Control:
	var wrap := Control.new()
	wrap.custom_minimum_size.y = _pscale(DccTheme.H_PHONE_GESTURE)
	wrap.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var bg := ColorRect.new()
	bg.color = Color(DccTheme.c("bg"), 0.9)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(bg)
	var handle := ColorRect.new()
	handle.color = Color(DccTheme.c("text_ghost"), 0.6)  ## Token-derived: see the
		## tool sheet's own handle for why a literal white is wrong here.
	var hw := _pscale(DccTheme.W_PHONE_GESTURE_HANDLE)
	var hh := _pscale(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(handle)
	return wrap

# -- Phone overlays: panel picker, overflow, dock sheets ------------------
#
# None of these four states are pictured in the mockup -- it ships exactly
# one static screen, chrome closed. Their *triggers* (☰/▤/⋯) and their
# *destination* (the reused dock/menu-bar/status-bar content) are spec'd;
# the overlay presentation itself is this file's own construction, built to
# the same visual language (colour tokens, hairlines, Plex Mono) as
# everything else in `DccTheme`/`DccWidgets` rather than invented from
# scratch. Said plainly because the rest of this file can cite a mockup line
# for nearly every choice, and these four can't.

## A dimmed full-rect scrim that closes its overlay when tapped outside the
## panel placed on top of it. Named handler rather than an inline lambda --
## `gui_input`'s own event argument plus a multi-statement body closed by the
## outer `connect(...)`'s `)` on the same line is the exact shape the
## match-in-a-lambda gotcha warns about, just with `if` instead of `match`.
func _phone_scrim_tap(ev: InputEvent, on_tap: Callable) -> void:
	var tapped: bool = (ev is InputEventMouseButton and ev.pressed) \
		or (ev is InputEventScreenTouch and ev.pressed)
	if tapped:
		on_tap.call()

func _phone_overlay_scrim(on_tap: Callable) -> Control:
	var wrap := Control.new()
	wrap.set_anchors_preset(Control.PRESET_FULL_RECT)
	var dim := ColorRect.new()
	dim.color = Color(0, 0, 0, 0.55)
	dim.set_anchors_preset(Control.PRESET_FULL_RECT)
	dim.mouse_filter = Control.MOUSE_FILTER_STOP
	dim.gui_input.connect(_phone_scrim_tap.bind(on_tap))
	wrap.add_child(dim)
	return wrap

func _sheet_close_button(on_press: Callable) -> Button:
	var b := Button.new()
	b.text = DccIcons.SYMBOLS["cross"]
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.tooltip_text = "Close"
	b.custom_minimum_size = Vector2(_ptap(44), _ptap(44))
	b.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.pressed.connect(on_press)
	return b

## A tappable title/subtitle row, shared by the drawer's domain list and the
## panel picker's two entries. `rpad`'s own full-rect anchors are what make it
## fill `row` -- `Button` isn't a `Container`, so a child's anchors resolve
## against its rect like any other Control parent, they just aren't
## auto-assigned the way a container's children would be.
func _phone_list_row(title: String, subtitle: String, on_press: Callable) -> Control:
	var row := Button.new()
	## Not flat -- see `add_menu()` and `_build_rail()`: a flat `Button` draws
	## no stylebox, so the press feedback on the next lines had never appeared
	## on a phone list row either.
	row.flat = false
	row.focus_mode = Control.FOCUS_NONE
	row.alignment = HORIZONTAL_ALIGNMENT_LEFT
	row.custom_minimum_size.y = _ptap(52)
	row.add_theme_stylebox_override("normal", DccTheme.empty())
	row.add_theme_stylebox_override("focus", DccTheme.empty())
	row.add_theme_stylebox_override("disabled", DccTheme.empty())
	row.add_theme_stylebox_override("pressed", DccTheme.flat(DccTheme.c("line_soft")))
	row.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))

	var rc := VBoxContainer.new()
	rc.add_theme_constant_override("separation", 1)
	rc.mouse_filter = Control.MOUSE_FILTER_IGNORE
	rc.add_child(DccTheme.mono_label(title.to_upper(), "text_bright", _pscale(11), 1, true))
	rc.add_child(DccTheme.label(subtitle, "text_faint", _pscale(9)))
	var rpad := MarginContainer.new()
	rpad.add_theme_constant_override("margin_left", _pscale(14))
	rpad.add_child(rc)
	row.add_child(rpad)
	rpad.set_anchors_preset(Control.PRESET_FULL_RECT)

	row.pressed.connect(on_press)
	return row

## The ☰ domain drawer -- a 300 dp side sheet listing the three `DOMAINS`
## with their subtitles, plus `_pick_drawer_domain()` and `_set_drawer_open()`
## -- was here and is **deleted** (2026-08-25, the 412 dp migration).
##
## `design/Cartalith Android Phone.dc.html` draws no drawer at any level. Its
## `02 Domain` screen is a full-screen drill with a `←`, which is exactly what
## this shell's full-height left dock sheet already is, so `☰` opens that
## instead. The three domains the drawer listed are the bottom bar's own first
## three cells -- with a glyph each as of this pass -- and a second,
## differently-shaped list of the same three destinations was the duplication
## that canvas's "More is a grouped list, not a duplicate menu bar" rule exists
## to prevent.

## ▤ panel picker: which dock to open as a full-height sheet. Anchored to the
## bottom rather than the drawer's side-panel treatment, so ☰ and ▤ read as
## two distinct affordances rather than the same drawer twice.
func _build_phone_panel_picker() -> Control:
	var overlay := _phone_overlay_scrim(func(): _set_panel_picker_open(false))

	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"top": 1}))
	panel.set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	panel.offset_left = 0
	panel.offset_right = 0
	panel.offset_top = -_pscale(160)
	panel.offset_bottom = -_pscale(DccTheme.H_PHONE_GESTURE)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)
	col.add_child(_phone_list_row("Left panel", "The active domain's workspace tools",
		func(): _set_sheet_open("left", true)))
	col.add_child(DccTheme.rule())
	col.add_child(_phone_list_row("Right panel", "Layers and selection detail",
		func(): _set_sheet_open("right", true)))

	overlay.add_child(panel)
	overlay.visible = false
	return overlay

# -- Phone overlay state ---------------------------------------------------
#
# Every phone overlay is mutually exclusive -- opening any one closes all the
# others, including a dock sheet -- so there is exactly one state variable per
# overlay and one shared teardown rather than a general stack. `PhoneMenu` keeps
# its own drill stack inside itself; from out here it is one more overlay.

func _close_all_phone_overlays() -> void:
	if _phone_panel_picker != null:
		_phone_panel_picker.visible = false
	if _phone_menu != null:
		_phone_menu.close()
	if left_dock != null:
		left_dock.visible = false
	if right_dock != null:
		right_dock.visible = false
	_left_sheet_open = false
	_right_sheet_open = false
	## `GUI_GAP_REGISTER.md` §46, raised by the concurrent phone pass and picked
	## up here because this is the function that owns the answer. Every entry
	## above is a `Control`; the Layers popover is a `PopupPanel`, which is a
	## `Window`, and no Control walk has ever reached it. Measured by that pass:
	## with the Layers sheet up, opening the ☰ overlay left **both** visible.
	##
	## `Popup` and deliberately not `Window`. A popover is transient -- going
	## somewhere else is what dismisses it -- while an `AcceptDialog` is a modal
	## the user is currently inside, and closing one out from under them would
	## trade a cosmetic overlap for lost input. `PopupMenu` is caught by the same
	## test, which is right for the same reason.
	##
	## `owned = false`: these are built in code and have no scene owner, so the
	## default `owned = true` would return an empty list and this would be
	## another silently-inert fix.
	for node in find_children("", "Popup", true, false):
		var pop := node as Popup
		if pop != null and pop.visible:
			pop.hide()
	_refresh_phone_bar_lit()

func _set_panel_picker_open(open: bool) -> void:
	_close_all_phone_overlays()
	_phone_panel_picker.visible = open
	_refresh_phone_bar_lit()

## Kept under its old name so `_shot_phone.gd --overflow` and anything else
## already driving it keeps working; what it opens is now `PhoneMenu`'s L2 root
## rather than the reparented desktop bar.
func _set_overflow_open(open: bool) -> void:
	_close_all_phone_overlays()
	if open:
		_phone_menu.open()
	_refresh_phone_bar_lit()

## The MORE tab is a toggle, because the canvas's `07 More` screen carries no
## close button of its own -- tapping the lit tab again is how you leave it, the
## way a bottom-nav tab behaves everywhere else. Without this, MORE would be the
## one tab in the bar that cannot be undone by pressing it.
func _toggle_overflow() -> void:
	var was_open: bool = _phone_menu != null and _phone_menu.is_open()
	_close_all_phone_overlays()
	if not was_open:
		_phone_menu.open()
	_refresh_phone_bar_lit()

## Offer a transient `PopupMenu` the phone's own sheet presentation. Returns
## **false** on desktop and tablet, where the caller should go on and call
## `PopupMenu.popup()` as it always has -- so a call site reads as one line
## with no `is_phone()` branch of its own, and a build with no phone chrome
## behaves identically to one that never heard of this function.
##
## Built for `civilization_workspace.gd`'s map context menu, which on a phone
## is opened by a press-and-hold (`map_overlay.gd`) and cannot use a stock
## popup: pointer-sized rows, and clipping rather than nudging when a finger
## lands near the screen edge.
func phone_present_popup(popup: PopupMenu, title: String, trail: String) -> bool:
	if not _phone or _phone_menu == null:
		return false
	_close_all_phone_overlays()
	_phone_menu.open_sheet(popup, title, trail)
	return true

## Android's back gesture -- the hardware `KEYCODE_BACK` and the edge swipe that
## replaced it -- arriving as `NOTIFICATION_WM_GO_BACK_REQUEST` because
## `_ready()` turned `quit_on_go_back` off. The canvas's BACK rule is "leaves a
## sheet, then the L2 screen, then the viewport", and one press leaves exactly
## ONE level, innermost first:
##
##   1. a dialog or popup window, wherever in the tree it is parented,
##   2. a phone-menu level (L5 → L4 → L3 → L2 → closed),
##   3. any other phone overlay -- panel picker, either dock sheet,
##   4. `_back_exhausted()`, which `DccApp` overrides to disarm a live tool and,
##      failing that, to put the SAME save/discard/cancel prompt File ▸ Close
##      project uses in front of the exit.
##
## Step 4 is why this was reopened. The first version ended in a bare
## `get_tree().quit()`, so a back gesture at the viewport with an unsaved
## generated world in memory destroyed it with no prompt at all. That is not a
## hypothetical: it happened to a tester on an OnePlus 6T, and the world was
## gone. Nothing in this shell may end the process without going through the
## same gate File ▸ Close project goes through.
func _notification(what: int) -> void:
	## The desktop system close, handled here for the one reason the whole gate
	## exists: it must not be possible to end this process with unsaved work in
	## it without being asked. Deliberately NOT routed through the back chain
	## above -- back means "leave the innermost thing", and the × means "close
	## the application", so it skips straight past dialogs, menu levels and
	## armed tools to the exit gate itself.
	##
	## Reaches this node only for the MAIN window: Godot propagates a window's
	## close request DOWN its own subtree (`Window::_propagate_window_notification`
	## stops at nested `Window`s), so closing a tool window or a dialog -- all of
	## which are children of this shell, not parents of it -- never lands here.
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		_close_requested()
		return
	if what != NOTIFICATION_WM_GO_BACK_REQUEST:
		return
	## Innermost first. An embedded dialog draws OVER the phone menu, so the
	## menu must not eat the gesture while one is open. Hidden rather than
	## freed: every dialog in this shell already frees itself from
	## `visibility_changed`, and hiding is precisely what its Cancel does.
	var top := _topmost_subwindow(get_tree().root)
	if top != null:
		top.hide()
		return
	if _phone_menu != null and _phone_menu.go_back():
		return
	if _phone_panel_picker != null and _phone_panel_picker.visible \
			or _left_sheet_open or _right_sheet_open:
		_close_all_phone_overlays()
		return
	_back_exhausted()

## The deepest visible `Window` under `root`, `root` itself excluded.
##
## Walked rather than read off a list, because a dialog is parented to whichever
## `Control` opened it and not to the root -- `DccApp`'s own prompts are children
## of `DccApp`, and `Viewport` exposes no subwindow list to GDScript. One walk
## per back press is not a cost worth optimising.
func _topmost_subwindow(node: Node) -> Window:
	var found: Window = null
	for child in node.get_children():
		if child is Window and not (child as Window).visible:
			continue  ## A hidden window's own children are unreachable too.
		var deeper := _topmost_subwindow(child)
		if deeper != null:
			found = deeper
		elif child is Window:
			found = child
	return found

## What back does once there is nothing left to leave. `DccShell` on its own
## holds no document, so quitting is correct here; `DccApp` overrides it to
## guard unsaved work first.
func _back_exhausted() -> void:
	get_tree().quit()

## The window manager asked for the app to close. `auto_accept_quit` is off, so
## this function OWNS the exit: if it neither quits nor puts a resolvable prompt
## on screen, the window cannot be closed at all. `DccShell` holds no document,
## so quitting outright is correct here; `DccApp` overrides it to guard unsaved
## work first, and carries the argument for why its version is still always
## escapable.
func _close_requested() -> void:
	get_tree().quit()

func _set_sheet_open(side: String, open: bool) -> void:
	if open:
		_close_all_phone_overlays()
	if side == "left":
		_left_sheet_open = open
		left_dock.visible = open
		if open:
			_reset_dock_scroll(_left_dock_scroll)
	else:
		_right_sheet_open = open
		right_dock.visible = open
		if open:
			_reset_dock_scroll(_right_dock_scroll)
	_refresh_phone_bar_lit()

## `phone_menu.gd::_render()` zeroes its own scroll the same way on every
## fill; a dock sheet's body, unlike the menu's, is never torn down and
## rebuilt between opens (`_build_left_dock`/`_build_right_dock` run once, at
## shell build time, `as_sheet = true` only swapping the header) -- so
## nothing else ever touches `scroll_vertical` back to 0, and whatever the
## sheet was scrolled to when it last closed is still sitting on the
## `ScrollContainer` when it reopens. Set once immediately (the common case)
## and once more deferred: a `ScrollContainer` that was `visible = false` a
## moment ago has not necessarily run its own sort/clamp pass yet, and a bare
## synchronous write here can still be re-clamped against a stale scrollbar
## range on the same frame the sheet becomes visible.
func _reset_dock_scroll(scroll: ScrollContainer) -> void:
	if scroll == null:
		return
	scroll.scroll_vertical = 0
	scroll.call_deferred("set", "scroll_vertical", 0)

## Re-applied on every resize while `_phone` is true (`_on_window_resized()`),
## which is the one part of the phone/tablet decision that genuinely must be
## live -- a device rotates at runtime even though its form factor never
## does. Only safe-area visibility, the chrome column's left margin, and the
## two dock sheets' rects change between orientations; the panel-
## picker/menu overlays and the tool sheet/timeline/bottom bar are unaffected,
## so this never touches anything a workspace has attached content to.
func _apply_phone_orientation() -> void:
	if _phone_top_safe == null:
		return  ## Not built yet -- called once more at the end of `_build_phone_shell()`.
	_phone_top_safe.visible = not _landscape
	_phone_side_safe.visible = _landscape

	## "The chrome shifts inward" (inset rule, LANDSCAPE): everything below the
	## safe area lives inside this margin, so growing it by the side safe area's
	## own width is the entire mechanism.
	var side_reserve := _pscale(DccTheme.H_PHONE_TOP_SAFE) if _landscape else 0
	## ...and in landscape two more regions now live at the edges: the nav rail
	## on the left and the docked sheet on the right
	## (`docs/ANDROID_UI_SPEC.md`: "nav becomes left rail, sheet docks right,
	## map stays wide"). The chrome column runs *between* them.
	##
	## The prototype instead lets the sheet overlay the app bar (its map host is
	## `left:72px;right:0` and the sheet sits on top at `z-index:12`). Insetting
	## the column is the deliberate departure: this app bar carries the search
	## and overflow buttons, which the prototype's does not, and a docked sheet
	## covering them would put two of the phone's five always-available controls
	## behind a sheet the user has to close to reach them. The *map* is still
	## edge-to-edge behind everything, so "map stays wide" is unaffected.
	var rail_w := _pscale(W_PHONE_LAND_RAIL) if _landscape else 0
	var sheet_w := _phone_land_sheet_width() if _landscape else 0
	_phone_chrome_margin.add_theme_constant_override("margin_left",
		side_reserve + rail_w)
	_phone_chrome_margin.add_theme_constant_override("margin_right", sheet_w)

	_apply_phone_nav_orientation(side_reserve, rail_w, sheet_w)

	var safe_top := 0 if _landscape else _pscale(DccTheme.H_PHONE_TOP_SAFE)
	for sheet in [left_dock, right_dock]:
		if sheet == null:
			continue
		## `+ rail_w`: a full-height dock sheet must clear the nav rail for the
		## same reason it clears the side safe area -- the rail is L1 of the
		## disclosure tree in landscape exactly as the bottom bar is in
		## portrait, and covering it would strand the user in the sheet.
		sheet.offset_left = side_reserve + rail_w
		sheet.offset_right = 0
		sheet.offset_top = safe_top
		sheet.offset_bottom = -_pscale(DccTheme.H_PHONE_GESTURE)

	## The menu takes the same rect as a dock sheet: over the app bar (its own
	## header replaces it, per the canvas's L2/L3 artboards), never over the
	## status safe area, the landscape side safe area or the gesture inset.
	##
	## **And never over the bottom bar**, as of the 412 migration. That bar is
	## L1 of the disclosure tree, and the canvas's `07 More` screen -- which is
	## what the menu's root is -- carries no close button of its own precisely
	## because it is a *tab destination*: you leave it by tapping another tab, or
	## the lit one. Covering the bar with the screen it opens would make MORE the
	## one tab in the bar that cannot be undone by pressing it, and (with the
	## canvas's two-`✕`-becomes-none change) would leave system back as the only
	## way out at all.
	##
	## `_ptap()` rather than the bar's measured `size.y`: this runs before the
	## first layout pass, where that is still zero, and it is the same expression
	## the bar sets its own minimum from.
	##
	## In landscape that same bar is the left rail, so its reserve moves from
	## the menu's bottom inset to its left one -- `rail_w`, folded into the
	## side reserve below. Getting this wrong would have been invisible in
	## portrait and would have covered the rail with the screen the rail opens.
	var bar_reserve := 0
	if _phone_menu_bar != null and _phone_menu_bar.visible and not _landscape:
		bar_reserve = _ptap(DccTheme.H_PHONE_BOTTOM_NAV)
	if _phone_menu != null:
		_phone_menu.apply_insets(float(safe_top), float(side_reserve + rail_w),
			float(_pscale(DccTheme.H_PHONE_GESTURE) + bar_reserve))

	phone_insets_changed.emit()

## `Math.min(440, Math.round(fw * 0.46))` -- the prototype's landscape sheet
## width, in its own dp, mapped onto real pixels. `fw` there is the frame's
## *long* side, which in landscape is the viewport width.
func _phone_land_sheet_width() -> int:
	return int(minf(float(_pscale(W_PHONE_LAND_SHEET_MAX)),
		round(get_viewport_rect().size.x * PHONE_LAND_SHEET_FRAC)))

## The landscape half of "nav becomes left rail, sheet docks right" -- and the
## portrait half that undoes it.
##
## Both regions are the **same nodes** in both orientations: rotating the bar is
## a `BoxContainer.vertical` flip (an `HBoxContainer` in Godot 4 is a
## `BoxContainer` with `vertical = false`, so this is a property, not a class),
## and relocating either one is a reparent between the chrome column and
## `_phone_root`. A second bar and a second sheet built alongside would have
## meant `set_tool_options()`, `_select_domain()`, `_domain_buttons`,
## `_phone_tab_cells` and `rail_column` each carrying two targets to keep in
## step -- five places for the two to drift apart.
func _apply_phone_nav_orientation(side_reserve: int, rail_w: int,
		sheet_w: int) -> void:
	if _phone_menu_bar == null or _phone_tool_sheet == null \
			or _phone_chrome_col == null:
		return
	var gesture := _pscale(DccTheme.H_PHONE_GESTURE)

	## `_phone_root` is a plain `Control`, so a child of it keeps whatever
	## anchors it is given -- the two dock sheets already rely on exactly that.
	var host: Node = _phone_root if _landscape else _phone_chrome_col
	for node in [_phone_tool_sheet, _phone_menu_bar]:
		if node.get_parent() == host:
			continue
		node.get_parent().remove_child(node)
		host.add_child(node)
	if _landscape:
		## Index 3 is directly above `_phone_chrome_margin` and below every
		## overlay (`_phone_side_safe`, the panel picker, `PhoneMenu`, the two
		## dock sheets) -- the same stacking order these two have in portrait,
		## where the overlays are added to `_phone_root` after the chrome.
		_phone_root.move_child(_phone_tool_sheet, 3)
		_phone_root.move_child(_phone_menu_bar, 3)
	else:
		## Back into their portrait slots. The order is re-established from the
		## top down, each target derived from the node just placed above it,
		## rather than by naming a neighbour's index directly.
		##
		## **`move_child()` inserts at the index it is given**, so the intuitive
		## "put the bar where the gesture inset currently is" lands it one slot
		## *below* the inset whenever the bar is already above it -- which is
		## the state the build-time call runs in. Caught on the device-shaped
		## probe: the bottom bar drew at y=827 in an 892 px frame, i.e. flush to
		## the bottom edge with the 20 px gesture inset stranded above it,
		## every launch, before any rotation.
		var order: Array[Node] = [_phone_content_gap, _phone_tool_sheet]
		if timeline_bar != null:
			order.append(timeline_bar)
		order.append(_phone_menu_bar)
		order.append(_phone_gesture_inset)
		for i in range(1, order.size()):
			_phone_chrome_col.move_child(order[i], order[i - 1].get_index() + 1)

	## -- the bar --------------------------------------------------------
	for box in [_phone_bar_row, _phone_bar_domains, _phone_bar_cells]:
		if box != null:
			box.vertical = _landscape
	if _phone_bar_domains != null:
		## The stretch that makes the four cells share the bar has to change
		## axis with it; `size_flags_stretch_ratio` is axis-agnostic and stays.
		_phone_bar_domains.size_flags_horizontal = \
			Control.SIZE_FILL if _landscape else Control.SIZE_EXPAND_FILL
		_phone_bar_domains.size_flags_vertical = \
			Control.SIZE_EXPAND_FILL if _landscape else Control.SIZE_FILL
	if _phone_bar_cells != null:
		## Prototype rail: `gap:6` between cells. Its `padding-top:40` is not
		## carried across -- that clears the prototype's status row, which sits
		## along the top edge in both orientations; this shell rotates its
		## status row onto the *left* edge in landscape (`_phone_side_safe`),
		## and the rail already starts to the right of it.
		_phone_bar_cells.add_theme_constant_override("separation",
			_pscale(6) if _landscape else 0)
	## `border-right:1px solid var(--hair2)` on `var(--pan2)` -- and `--pan2` is
	## `#121314`, which is the `panel` token the bar already draws in, so only
	## the edge moves.
	_phone_menu_bar.add_theme_stylebox_override("panel",
		DccTheme.panel("panel", {"right": 1} if _landscape else {"top": 1}))
	if _landscape:
		_phone_menu_bar.set_anchors_preset(Control.PRESET_LEFT_WIDE)
		_phone_menu_bar.offset_left = side_reserve
		_phone_menu_bar.offset_right = side_reserve + rail_w
		_phone_menu_bar.offset_top = 0
		_phone_menu_bar.offset_bottom = -gesture
		_phone_menu_bar.custom_minimum_size = Vector2(float(rail_w), 0.0)
	else:
		_phone_menu_bar.custom_minimum_size = Vector2.ZERO

	## -- the sheet ------------------------------------------------------
	if _landscape:
		_phone_tool_sheet.set_anchors_preset(Control.PRESET_RIGHT_WIDE)
		_phone_tool_sheet.offset_left = -sheet_w
		_phone_tool_sheet.offset_right = 0
		_phone_tool_sheet.offset_top = 0
		## §13: "Timeline and sheets stop above it" -- the gesture inset is the
		## one region a docked sheet still may not reach into.
		_phone_tool_sheet.offset_bottom = -gesture
		## The detent height is a portrait concept; in landscape the anchors own
		## the rect and a leftover minimum would fight them.
		_phone_tool_sheet.custom_minimum_size.y = 0.0
		_phone_tool_sheet.add_theme_stylebox_override("panel",
			_phone_sheet_box(true))
	else:
		_phone_tool_sheet.add_theme_stylebox_override("panel",
			_phone_sheet_box(false))
		## Un-animated: a rotation re-lays the whole chrome out in one frame,
		## and a 0.28 s height tween across that reads as a glitch, not a
		## transition.
		_snap_phone_sheet(false)

## What `ViewportHost`'s own corner chrome (layers button, coord readout,
## scale bar -- all built by `viewport_host.gd`, unchanged) should treat as
## "inside the safe area" now that the phone chrome sits on top of an
## edge-to-edge map instead of a flow container sizing the viewport to the
## gap between docks. `app.gd` calls this once, deferred, after building
## `ViewportHost` (deferred so the tool sheet has had one layout pass -- see
## `_phone_bottom_reserve()`), and again on every `phone_insets_changed`
## (rotation, and any tool-sheet content change -- `set_tool_options()` fires
## it too).
func phone_content_insets() -> Dictionary:
	if not _phone:
		return {"left": 10.0, "top": 10.0, "right": 10.0, "bottom": 10.0, "scale": 1.0}
	## No left reserve in portrait any more: the domain rail that used to float
	## there is the bottom bar now, so the map has the full width back and only
	## the landscape safe area still eats into it.
	var left := 0.0
	var right := 0.0
	if _landscape:
		left += float(_pscale(DccTheme.H_PHONE_TOP_SAFE))
		## Landscape's two new edge regions: the nav rail on the left, the
		## docked sheet on the right (`docs/ANDROID_UI_SPEC.md`). Both are
		## opaque, so `ViewportHost`'s floating chrome has to clear them the
		## same way it clears the bottom bar in portrait -- and `right` had
		## never been anything but 0 before there was something over there.
		left += float(_pscale(W_PHONE_LAND_RAIL))
		right = float(_phone_land_sheet_width())
	var top := float(_ptap(DccTheme.H_PHONE_APP_BAR))
	if not _landscape:
		top += float(_pscale(DccTheme.H_PHONE_TOP_SAFE))
	## `scale` rides along because it is the one number `ViewportHost` needs and
	## has no other route to (`GUI_GAP_REGISTER.md` HD-03). Its floating chrome
	## -- the Layers button and the four navpad pills -- is authored at a raw
	## 44 px, on a premise `NAVPAD_HIT`'s own comment states outright and which
	## is false: the main viewport is NOT content-scaled, so the shipped
	## handset's viewport is its full pixel width, not 393. Measured, 44 px
	## comes to 2.83 mm on a 1080/395 ppi panel and **2.19 mm** on the
	## OnePlus 12's 1440/510, against the ~7 mm a 44 dp floor is asking for.
	## Passing it through the existing dictionary rather than adding a setter
	## keeps `app.gd` -- which owns the one call site -- out of it.
	## The clamp is the detents' doing. At `full` the sheet covers everything
	## but 96 dp of map, so the *honest* bottom reserve exceeds the screen and
	## `ViewportHost` would place its coordinate readout and scale bar above the
	## top edge -- off screen, which is worse than occluded. One tap target of
	## band is left for them; they overlap the sheet's top edge at that detent,
	## which is the same trade `_phone_bottom_reserve()` already documents in
	## the other direction.
	var band: float = get_viewport_rect().size.y - top - float(_ptap(DccTheme.PHONE_TAP_MIN))
	var bottom: float = clampf(_phone_bottom_reserve(), 0.0, maxf(0.0, band))
	return {"left": left, "top": top, "right": right,
		"bottom": bottom, "scale": _phone_scale}

## The tool sheet's real height depends on whatever `tool_options_row`
## currently holds -- domain content this frame doesn't own -- so this reads
## the sheet's own actual size once it has one (non-zero after a layout pass)
## rather than guessing it. Before that first pass (the un-deferred instant
## `_build_phone_shell()` finishes in), falls back to a fixed estimate biased
## generous on purpose: an *under*-estimate leaves the coordinate readout and
## scale bar hidden behind the opaque sheet (found by screenshot -- the
## original flat 44 px guess did exactly this), while an *over*-estimate only
## leaves them floating a bit higher than strictly necessary. Wrong in the
## safe direction, in other words.
func _phone_bottom_reserve() -> float:
	if _landscape:
		## Neither the bar nor the sheet is at the bottom any more -- they are
		## the left rail and the right dock. Only the gesture inset and the
		## timeline still are, and both are reported to the caller as `left`
		## and `right` instead (see `phone_content_insets()`).
		var b := float(_pscale(DccTheme.H_PHONE_GESTURE))
		if timeline_bar != null and timeline_bar.visible:
			b += timeline_bar.size.y
		return b
	var bar := 0.0
	if _phone_menu_bar != null and _phone_menu_bar.visible:
		bar = _phone_menu_bar.size.y if _phone_menu_bar.size.y > 0.0 \
			else float(_ptap(64))
	if _phone_tool_sheet != null and _phone_tool_sheet.size.y > 0.0:
		var h := _phone_tool_sheet.size.y + float(_pscale(DccTheme.H_PHONE_GESTURE))
		if timeline_bar != null and timeline_bar.visible:
			h += timeline_bar.size.y
		return h + bar + float(_pscale(8))
	return float(_pscale(20 + 90 + DccTheme.H_PHONE_GESTURE)) + bar
