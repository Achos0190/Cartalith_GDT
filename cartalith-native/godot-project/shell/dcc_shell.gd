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
var _phone_scrim: TextureRect  ## Held because its colour lives inside a
	## `GradientTexture2D`, which `_recolor_subtree()` cannot reach -- see
	## `rebuild_theme()`.
var _phone_gesture_inset: Control
var _phone_clock_label: Label
var _phone_battery_label: Label
var _phone_side_clock_label: Label     ## Landscape's rotated-pocket twins of
var _phone_side_battery_label: Label   ## the two above -- see `_build_phone_side_safe()`.
var _phone_drawer: Control
var _phone_panel_picker: Control
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

# -- Build --------------------------------------------------------------------

func _ready() -> void:
	## `--force-touch`: a testing-only override, same pattern as `_shot.gd`'s
	## own `--generate` flag. Real touch hardware is never present in this
	## dev/CI environment, so without it the phone/tablet composition below is
	## simply unreachable from `--resolution WxH` alone -- there is no device-
	## preview loop, and `--resolution` is otherwise the only lever the
	## verification harness has to exercise §13 at all.
	_touch = (DisplayServer.is_touchscreen_available() and OS.has_feature("mobile")) \
		or "--force-touch" in OS.get_cmdline_user_args()
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
	wordmark.custom_minimum_size.x = 150
	row.add_child(wordmark)

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
	mb.flat = true
	mb.focus_mode = Control.FOCUS_NONE
	mb.add_theme_font_size_override("font_size", DccTheme.FS_MENU)
	mb.add_theme_font_override("font", DccTheme.mono(0))
	mb.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	mb.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	mb.add_theme_stylebox_override("normal", DccTheme.inset(11, 9, 11, 9))
	mb.add_theme_stylebox_override("hover", DccTheme.inset(11, 9, 11, 9))
	mb.add_theme_stylebox_override("pressed", DccTheme.active_row())
	menu_bar_row.add_child(mb)
	var popup := mb.get_popup()
	style_popup(popup)
	on_built.call(popup)
	return mb

func style_popup(popup: PopupMenu) -> void:
	popup.add_theme_stylebox_override("panel", DccTheme.panel("raised",
		{"left": 1, "right": 1, "top": 1, "bottom": 1}))
	popup.add_theme_color_override("font_color", DccTheme.c("text"))
	popup.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	popup.add_theme_color_override("font_accelerator_color", DccTheme.c("text_faint"))
	popup.add_theme_font_size_override("font_size", DccTheme.FS_MENU)
	popup.add_theme_font_override("font", DccTheme.mono(0))
	popup.add_theme_constant_override("v_separation", 7)

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
# matches no token (a literal, e.g. the phone drawer's plain black scrim) is
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
	"font_pressed_color", "font_uneditable_color", "icon_hover_color",
	"icon_normal_color", "icon_pressed_color",
]
const _THEME_STYLEBOX_OVERRIDES := [
	"disabled", "focus", "grabber_area", "grabber_area_highlight", "hover", "normal",
	"panel", "pressed", "read_only", "slider",
]

## Called by `menus.gd` immediately after `DccTheme.apply_theme()`, passing
## whichever palette was active a moment ago (`was_dark`) so the walk knows
## what it's reversing.
func rebuild_theme(was_dark: bool) -> void:
	var old_pal: Dictionary = DccTheme.DARK if was_dark else DccTheme.LIGHT
	_recolor_project_theme(old_pal)
	_style_window_chrome()
	_recolor_subtree(self, old_pal)
	## The phone top scrim's colour is inside a `GradientTexture2D`, not on any
	## node and not in the theme resource, so neither of the two walks above can
	## see it. Found by capturing the phone menu under the light palette: a
	## charcoal status band sat above a light screen. Rebuilt rather than
	## remapped -- the builder already writes it from `c("bg")`.
	if _phone_scrim != null:
		_phone_scrim.texture = _phone_scrim_texture()

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
		phone_fit(child, unit, wide)

## The phone half of a §4.5 tool button: a real glyph, a visible box, and the
## name the tooltip can no longer deliver. See the call site in `phone_fit()`
## for the fault this closes.
func _phone_fit_tool_button(b: Button, unit: float) -> void:
	## Re-rasterised from the SVG at the size it will actually be drawn at,
	## which is why `dcc_widgets.gd` stashes the glyph's *name*: `DccIcons`
	## caches per `name@px`, so the 15 px texture already in hand cannot be
	## grown without resampling it. 0.42 of the box leaves the caption room and
	## keeps the icon off the border.
	var box := maxf(b.custom_minimum_size.x, b.custom_minimum_size.y)
	b.icon = DccIcons.get_icon(String(b.get_meta(DccWidgets.TOOL_GLYPH_META)),
		maxi(1, int(round(box * 0.42))))
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
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.add_theme_stylebox_override("normal", DccTheme.empty())
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
		b.pressed.connect(_select_domain.bind(d.id))

		## The reference rail is text only -- verified twice: once by reading
		## `design/Cartalith DCC Shell.dc.html`'s own markup (`writing-mode:
		## vertical-rl` labels, no icon element anywhere in the rail), and once
		## by the owner directly, after an earlier revision added icons anyway:
		## "those icons don't exist." Removed rather than hidden behind a flag --
		## an addition the design does not specify does not get to linger.
		var vlabel := DccTheme.mono_label(String(d.rail).to_upper(),
			"text_faint", DccTheme.FS_MICRO, 2, true)
		vlabel.rotation = -PI / 2.0
		var text_size := vlabel.get_minimum_size()
		var label_x: float = round(w * 0.5 - text_size.y * 0.5)
		vlabel.position = Vector2(label_x, 12.0)
		b.add_child(vlabel)
		b.custom_minimum_size.y = text_size.x + 24.0

		_domain_buttons[d.id] = b
		_domain_marks[d.id] = {"label": vlabel}
		rail_column.add_child(b)

	body.add_child(DccTheme.spacer())
	rail_foot = DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 2)
	rail_foot.rotation = -PI / 2.0
	var foot_holder := Control.new()
	foot_holder.custom_minimum_size.y = 84
	foot_holder.add_child(rail_foot)
	body.add_child(foot_holder)
	return rail

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
		b.add_theme_stylebox_override("normal",
			DccTheme.active_row(false) if on else DccTheme.empty())
		var marks: Dictionary = _domain_marks.get(key, {})
		if marks.has("label"):
			(marks["label"] as Label).add_theme_color_override("font_color",
				DccTheme.c("accent") if on else DccTheme.c("text_faint"))
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
	head.custom_minimum_size.y = _ptap(44) if as_sheet else 26
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
	head.custom_minimum_size.y = _ptap(44) if as_sheet else 26
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
	for child in dock.get_child(0).get_children():
		if child is ScrollContainer:
			child.visible = not collapsed
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

	for slot in ["pass", "stale", "autosave", "atlas"]:
		var l := DccTheme.label("", "text_faint", DccTheme.FS_SMALL)
		_status_labels[slot] = l
		status_row.add_child(l)
	status_row.add_child(DccTheme.spacer())
	var hint := DccTheme.label("", "text_ghost", DccTheme.FS_SMALL)
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
#      (landscape only), the ☰ domain drawer, the panel picker, the phone
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

	_phone_scrim = _build_phone_scrim()
	_phone_root.add_child(_phone_scrim)

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

	_phone_top_safe = _build_phone_top_safe()
	chrome.add_child(_phone_top_safe)

	chrome.add_child(_build_phone_app_bar())

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

	_phone_drawer = _build_phone_drawer()
	_phone_root.add_child(_phone_drawer)

	_phone_panel_picker = _build_phone_panel_picker()
	_phone_root.add_child(_phone_panel_picker)

	## L2-L5. Added after the drawer and picker so it draws over them, and
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

## Legibility over imagery without an opaque strip (inset rule "SCRIM, NOT A
## BAR"). A `GradientTexture2D`, not a flat colour -- the fade is the point,
## the map should still show faintly through the scrim's lower half. Height
## 96 px = the 44 px safe area plus the 52 px app bar, so the fade finishes
## exactly where the app bar's own opaque background takes over.
func _build_phone_scrim() -> TextureRect:
	var scrim := TextureRect.new()
	scrim.texture = _phone_scrim_texture()
	scrim.stretch_mode = TextureRect.STRETCH_SCALE
	scrim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	scrim.set_anchors_preset(Control.PRESET_TOP_WIDE)
	scrim.offset_left = 0
	scrim.offset_right = 0
	scrim.offset_top = 0
	scrim.offset_bottom = _pscale(DccTheme.H_PHONE_TOP_SCRIM)
	return scrim

## The gradient on its own, so `rebuild_theme()` can re-derive it from the new
## palette without building (and leaking) a second `TextureRect` to steal one
## from -- which is exactly what the first attempt did, and the engine reported
## it as a leaked GLES3 texture RID at exit.
func _phone_scrim_texture() -> GradientTexture2D:
	var grad := Gradient.new()
	grad.colors = PackedColorArray([
		Color(DccTheme.c("bg"), 0.94),
		Color(DccTheme.c("bg"), 0.86),
		Color(DccTheme.c("bg"), 0.0),
	])
	grad.offsets = PackedFloat32Array([0.0, 0.46, 1.0])
	var tex := GradientTexture2D.new()
	tex.gradient = grad
	tex.width = 4
	tex.height = 128
	tex.fill_from = Vector2(0, 0)
	tex.fill_to = Vector2(0, 1)
	return tex

## Inset rule "TOP 44 PX · KEEP CLEAR": glyphs only, in left/right pockets,
## nothing centred. The 108 px centre lane isn't modelled as a literal spacer
## -- there is simply nothing placed there, which trivially satisfies "nothing
## is centred there" -- so a plain two-child row with an expanding gap between
## does the whole job.
func _build_phone_top_safe() -> Control:
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(16))
	pad.add_theme_constant_override("margin_right", _pscale(16))
	pad.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var row := HBoxContainer.new()
	row.custom_minimum_size.y = _pscale(DccTheme.H_PHONE_TOP_SAFE)
	pad.add_child(row)

	_phone_clock_label = DccTheme.mono_label("", "text_dim", _pscale(11))
	_phone_clock_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(_phone_clock_label)
	row.add_child(DccTheme.spacer())
	_phone_battery_label = DccTheme.mono_label("", "text_faint", _pscale(11))
	_phone_battery_label.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(_phone_battery_label)

	var timer := Timer.new()
	timer.wait_time = 30.0
	timer.autostart = true
	timer.timeout.connect(_refresh_phone_status_glyphs)
	pad.add_child(timer)
	_refresh_phone_status_glyphs()
	return pad

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

## The app bar (inset rule / §13: "the first row allowed to hold controls" --
## ☰ domain drawer, title + seed, ▤ panels, ⋯ overflow). All four hit boxes
## are exactly 44 px, per the mockup's own app-bar row (lines 1466-1474).
func _build_phone_app_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _ptap(DccTheme.H_PHONE_APP_BAR)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"bottom": 1}))
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", _pscale(10))
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", _pscale(10))
	pad.add_theme_constant_override("margin_right", _pscale(10))
	pad.add_child(row)
	bar.add_child(pad)

	row.add_child(_phone_bar_button(DccIcons.SYMBOLS["drawer"], "Domains",
		func(): _set_drawer_open(true)))

	var title_col := VBoxContainer.new()
	title_col.add_theme_constant_override("separation", 0)
	title_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_col.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	title_col.add_child(DccTheme.mono_label("CARTALITH", "text_bright", _pscale(12), 3, true))
	## Reuses the same "top_world" status slot the desktop menu bar's readout
	## cluster fills (`_wire_status()` in `app.gd` calls
	## `set_status("top_world", "ELDRA · %d" % seed)`) -- no phone-aware
	## branch needed in `app.gd` for this to stay live.
	var subtitle := DccTheme.mono_label("", "text_faint", _pscale(9), 1)
	_status_labels["top_world"] = subtitle
	title_col.add_child(subtitle)
	row.add_child(title_col)

	## ▤ Panels and ⋯ Menu used to sit here as well. Both are slots 4 and 5 of
	## the L1 bottom bar now (`_build_phone_menu_bar()`), per `design/Cartalith
	## Android Phone.dc.html`, and carrying them twice would be two affordances
	## for one destination -- exactly the duplication that canvas's own "More is
	## a grouped list, not a duplicate menu bar" note rules out.
	return bar

func _phone_bar_button(glyph: String, tip: String, on_press: Callable,
		token: String = "text_dim") -> Button:
	var b := Button.new()
	b.text = glyph
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.tooltip_text = tip
	b.custom_minimum_size = Vector2(_ptap(44), _ptap(44))
	b.add_theme_font_size_override("font_size", _pscale(16))
	b.add_theme_font_override("font", DccTheme.mono())
	b.add_theme_color_override("font_color", DccTheme.c(token))
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
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
## The ☰ drawer stays, because the canvas's own app bar keeps it and it is the
## only place a domain's subtitle is legible.
##
## Five slots over three domains: WORLD · CIVIL · CARTO, then **PANELS** (the
## dock picker the app bar used to carry) and **MENU** (the whole program menu
## tree, `phone_menu.gd`). That is the canvas's own shape -- three-to-five
## subject tabs, then the two things entered "a few times per session, not per
## minute".
##
## Text-only, no glyph. The canvas draws ◈ ⌗ ◷ ▤ ⋯ over its captions, but the
## owner has already ruled on exactly this for the desktop rail -- *"those icons
## don't exist"* -- and the five glyphs it uses are outside the set this build
## has proven renders on the device (`dcc_icons.gd`'s own note: two symbols in
## that table are missing from Plex Mono *and* the fallback chain, and render as
## tofu). A caption that is definitely legible beats a glyph that might be a
## box; if the icons are ever drawn, they drop into `cell` with no other change.
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

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	bar.add_child(row)

	var domains := HBoxContainer.new()
	domains.add_theme_constant_override("separation", 0)
	domains.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	domains.size_flags_stretch_ratio = float(DOMAINS.size())
	_rail_region = domains
	row.add_child(domains)

	rail_column = VBoxContainer.new()
	rail_column.add_theme_constant_override("separation", 0)
	rail_column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	domains.add_child(rail_column)

	var cells := HBoxContainer.new()
	cells.add_theme_constant_override("separation", 0)
	rail_column.add_child(cells)

	for d in DOMAINS:
		var cell := _phone_bar_cell(String(d.rail), String(d.label) + " -- " + String(d.subtitle),
			_pick_bar_domain.bind(String(d.id)))
		_domain_buttons[d.id] = cell["button"]
		_domain_marks[d.id] = {"label": cell["label"]}
		cells.add_child(cell["button"] as Control)

	## Outside `_rail_region`, on purpose -- see this function's header.
	row.add_child((_phone_bar_cell("PANELS", "Left and right panels",
		func(): _set_panel_picker_open(true))["button"]) as Control)
	row.add_child((_phone_bar_cell("MENU", "File, Edit, Assets, Data, Preferences, Window, Help",
		func(): _set_overflow_open(true))["button"]) as Control)
	return bar

## One bar cell. Returns both nodes because `_select_domain()` recolours the
## caption and restyles the button, and only the domain cells register there.
func _phone_bar_cell(caption: String, tip: String, on_press: Callable) -> Dictionary:
	var b := Button.new()
	b.tooltip_text = tip
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	## Canvas: a 64 dp bar. `_ptap` floors it at the 44 px minimum and scales it
	## with everything else, so this is the same target arithmetic the app bar's
	## own buttons use -- not a second set of numbers.
	b.custom_minimum_size.y = _ptap(64)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.add_theme_stylebox_override("normal", DccTheme.empty())
	b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	b.add_theme_stylebox_override("pressed", DccTheme.active_row(false))
	b.pressed.connect(on_press)

	var l := DccTheme.mono_label(caption.to_upper(), "text_faint", _pscale(9), 2, true)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	b.add_child(l)
	l.set_anchors_preset(Control.PRESET_FULL_RECT)
	return {"button": b, "label": l}

## A bar domain was tapped: switch domain and drop any menu that was over it,
## so the result is visible immediately -- the canvas's "the map never leaves
## the screen" rule.
func _pick_bar_domain(id: String) -> void:
	_close_all_phone_overlays()
	_select_domain(id)

## §13: "tool options become a bottom sheet". The drag handle is decorative --
## the mockup pictures exactly one static sheet state, so nothing here answers
## a drag gesture (inventing one would be behaviour the design doesn't show).
## `tool_options_row` is the same `HBoxContainer` `set_tool_options()` already
## rebuilds from `app.gd` -- wrapped in a horizontal `ScrollContainer` here
## because its desktop-tuned content (a run-pipeline row with several buttons
## and spacers) is wider than 393 px and would otherwise clip. No fixed sheet
## height is set; it hugs the handle plus that one content row.
func _build_phone_tool_sheet() -> PanelContainer:
	var sheet := PanelContainer.new()
	sheet.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"top": 1}))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	sheet.add_child(col)

	var handle_wrap := Control.new()
	handle_wrap.custom_minimum_size.y = _pscale(20)
	var handle := ColorRect.new()
	## Token-derived, not a literal white: `DccTheme.remap()` can only repaint a
	## colour it can trace back to a token, so a flat `Color(1,1,1,0.22)` here
	## stayed white when the palette went light and vanished into the panel.
	handle.color = Color(DccTheme.c("text_ghost"), 0.55)
	var hw := _pscale(38)
	var hh := _pscale(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle_wrap.add_child(handle)
	col.add_child(handle_wrap)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
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

## §13: "bottom 26 px is the gesture inset -- no tappable target inside it."
## `MOUSE_FILTER_IGNORE` all the way down enforces that structurally, not just
## visually -- there is nothing here a tap could hit even by accident.
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
	var hw := _pscale(110)
	var hh := _pscale(4)
	handle.set_anchors_preset(Control.PRESET_CENTER)
	handle.size = Vector2(hw, hh)
	handle.position = Vector2(-hw / 2.0, -hh / 2.0)
	handle.mouse_filter = Control.MOUSE_FILTER_IGNORE
	wrap.add_child(handle)
	return wrap

# -- Phone overlays: drawer, panel picker, overflow, dock sheets ----------
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
	row.flat = true
	row.focus_mode = Control.FOCUS_NONE
	row.alignment = HORIZONTAL_ALIGNMENT_LEFT
	row.custom_minimum_size.y = _ptap(52)
	row.add_theme_stylebox_override("normal", DccTheme.empty())
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

func _pick_drawer_domain(id: String) -> void:
	_select_domain(id)
	_set_drawer_open(false)

## ☰ domain drawer: the same `DOMAINS`, as full label + subtitle rows rather
## than the bottom bar's tracked abbreviations -- a second, more legible way in,
## not a replacement for it (see `_build_phone_menu_bar()`'s comment).
func _build_phone_drawer() -> Control:
	var overlay := _phone_overlay_scrim(func(): _set_drawer_open(false))

	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"right": 1}))
	panel.set_anchors_preset(Control.PRESET_LEFT_WIDE)
	panel.offset_left = 0
	panel.offset_right = _pscale(300)
	panel.offset_top = 0
	panel.offset_bottom = 0

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)

	var head := HBoxContainer.new()
	head.custom_minimum_size.y = _ptap(44)
	head.add_child(DccTheme.header("Domains", ""))
	head.add_child(DccTheme.spacer())
	head.add_child(_sheet_close_button(func(): _set_drawer_open(false)))
	var hp := MarginContainer.new()
	hp.add_theme_constant_override("margin_left", 14)
	hp.add_theme_constant_override("margin_right", 6)
	hp.add_child(head)
	col.add_child(hp)
	col.add_child(DccTheme.rule())

	## Named rather than an inline lambda: a multi-statement lambda body on
	## one line is exactly the shape `DCC_SHELL_SPEC.md`-adjacent GDScript
	## gotchas live in, and a lambda closing over a `for` loop's own variable
	## is a second, separate risk -- `_build_rail()`'s desktop equivalent
	## already avoids both by binding rather than closing over `d`.
	for d in DOMAINS:
		col.add_child(_phone_list_row(String(d.label), String(d.subtitle),
			_pick_drawer_domain.bind(d.id)))
		col.add_child(DccTheme.rule())

	overlay.add_child(panel)
	overlay.visible = false
	return overlay

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
	if _phone_drawer != null:
		_phone_drawer.visible = false
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

func _set_drawer_open(open: bool) -> void:
	_close_all_phone_overlays()
	_phone_drawer.visible = open

func _set_panel_picker_open(open: bool) -> void:
	_close_all_phone_overlays()
	_phone_panel_picker.visible = open

## Kept under its old name so `_shot_phone.gd --overflow` and anything else
## already driving it keeps working; what it opens is now `PhoneMenu`'s L2 root
## rather than the reparented desktop bar.
func _set_overflow_open(open: bool) -> void:
	_close_all_phone_overlays()
	if open:
		_phone_menu.open()

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
##   3. any other phone overlay -- drawer, panel picker, either dock sheet,
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
	if _phone_drawer != null and _phone_drawer.visible \
			or _phone_panel_picker != null and _phone_panel_picker.visible \
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
## two dock sheets' rects change between orientations; the drawer/panel-
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
	_phone_chrome_margin.add_theme_constant_override("margin_left", side_reserve)

	var safe_top := 0 if _landscape else _pscale(DccTheme.H_PHONE_TOP_SAFE)
	for sheet in [left_dock, right_dock]:
		if sheet == null:
			continue
		sheet.offset_left = side_reserve
		sheet.offset_right = 0
		sheet.offset_top = safe_top
		sheet.offset_bottom = -_pscale(DccTheme.H_PHONE_GESTURE)

	## The menu takes the same rect as a dock sheet: over the app bar (its own
	## header replaces it, per the canvas's L2/L3 artboards), never over the
	## status safe area, the landscape side safe area or the gesture inset.
	if _phone_menu != null:
		_phone_menu.apply_insets(float(safe_top), float(side_reserve),
			float(_pscale(DccTheme.H_PHONE_GESTURE)))

	phone_insets_changed.emit()

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
		return {"left": 10.0, "top": 10.0, "right": 10.0, "bottom": 10.0}
	## No left reserve in portrait any more: the domain rail that used to float
	## there is the bottom bar now, so the map has the full width back and only
	## the landscape safe area still eats into it.
	var left := 0.0
	if _landscape:
		left += float(_pscale(DccTheme.H_PHONE_TOP_SAFE))
	var top := float(_ptap(DccTheme.H_PHONE_APP_BAR))
	if not _landscape:
		top += float(_pscale(DccTheme.H_PHONE_TOP_SAFE))
	return {"left": left, "top": top, "right": 0.0, "bottom": _phone_bottom_reserve()}

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
