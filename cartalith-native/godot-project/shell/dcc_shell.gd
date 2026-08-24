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

## `subnodes`: SH-01's rail-expansion sub-node list -- each domain's *real*
## top-level dock structure, not invented categories. Read off the domain's
## own left-dock builder rather than the spec (`DCC_CONTROL_INDEX.md`: "the
## builder has no source for them" -- this is that source): WORLD's two-mode
## switch plus its one TOOLS-block button (`world_workspace.gd::_build`, "the
## only button that belongs here is Biome paint"), and CIVIL/CARTO's own
## `DccWidgets.category()` L2 accordion titles, grepped verbatim from
## `workspaces/*.gd` (`_build_settlements`/`_build_population`/… etc.).
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
		"subtitle": "Terrain, hydrology, climate and ecology",
		"subnodes": "Generation pipeline · Sculpt · Biome paint"},
	{"id": "civilization", "label": "Civilization", "rail": "CIVIL", "icon": "domain_civ",
		"subtitle": "Settlements, factions, provinces, trade, roads, sea routes and journeys",
		"subnodes": "Settlements · Population · Economy · Politics · Culture · Timeline · Roads · Rivers · Ports · Trade · Logistics"},
	{"id": "cartography", "label": "Cartography", "rail": "CARTO", "icon": "domain_carto",
		"subtitle": "Layers, styles, labels, annotation and terrain appearance",
		"subnodes": "Layers · Layer properties · Annotation · Terrain appearance"},
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

# -- SH-01 rail expansion ------------------------------------------------------
var _rail_panel: PanelContainer
var _rail_collapsed_body: VBoxContainer  ## The domain-button column + spacer + foot -- hidden while expanded.
var _rail_subnodes_body: VBoxContainer   ## The expanded state's per-domain row list -- hidden while collapsed.
var _rail_expand_button: Button
var _rail_expanded := false

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
var _phone_gesture_inset: Control
var _phone_clock_label: Label
var _phone_battery_label: Label
var _phone_side_clock_label: Label     ## Landscape's rotated-pocket twins of
var _phone_side_battery_label: Label   ## the two above -- see `_build_phone_side_safe()`.
var _phone_drawer: Control
var _phone_panel_picker: Control
var _phone_overflow: Control
var _left_sheet_open := false
var _right_sheet_open := false

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
	th.set_color("title_color", "Window", DccTheme.c("text_bright"))
	th.set_color("title_outline_modulate", "Window", DccTheme.c("raised"))
	for box in ["embedded_border", "embedded_unfocused_border"]:
		th.set_stylebox(box, "Window", DccTheme.panel("raised",
			{"left": 1, "right": 1, "top": 1, "bottom": 1}))

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
		_phone_fit_tool_options(tool_options_row)
		(func(): phone_insets_changed.emit()).call_deferred()

## Owner, 2026-08-20: "the bottom menu butons on phone are near too small to
## use". They were, and the earlier "44 px lands at ~121 physical px" arithmetic
## was measuring the wrong thing -- it described the *chrome* (`_build_phone_app
## _bar()`, `_build_phone_rail()`), which does route every size through
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
func _phone_fit_tool_options(node: Node) -> void:
	for child in node.get_children():
		if child is Control:
			var ctl := child as Control
			## Explicit font-size overrides beat any theme we could hang on the
			## sheet, so they have to be re-written rather than inherited.
			if ctl.has_theme_font_size_override("font_size"):
				ctl.add_theme_font_size_override("font_size",
					_pscale(ctl.get_theme_font_size("font_size")))
			## Scale whatever the desktop row asked for, then floor anything
			## tappable at §13's 44 px -- the floor is the half the owner felt.
			var min_size := ctl.custom_minimum_size
			if min_size.x > 0.0:
				min_size.x = _pscale(min_size.x)
			if min_size.y > 0.0:
				min_size.y = _pscale(min_size.y)
			if ctl is BaseButton or ctl is LineEdit or ctl is Range or ctl is TextEdit:
				min_size.y = maxf(min_size.y, float(_ptap(DccTheme.PHONE_TAP_MIN)))
				if min_size.x > 0.0:
					min_size.x = maxf(min_size.x, float(_ptap(DccTheme.PHONE_TAP_MIN)))
			ctl.custom_minimum_size = min_size
		_phone_fit_tool_options(child)

## Read by dialogs that have to present themselves differently on a phone
## (`open_project_dialog.gd`); `_phone`/`_phone_scale` stay private because
## nothing outside should be *setting* them.
func is_phone() -> bool:
	return _phone

func phone_scale() -> float:
	return _phone_scale

# -- §3 Domain rail -----------------------------------------------------------

func _build_rail() -> Control:
	var rail := PanelContainer.new()
	_rail_panel = rail
	rail.custom_minimum_size.x = _scaled(DccTheme.W_RAIL_COLLAPSED)
	rail.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))
	rail_column = VBoxContainer.new()
	rail_column.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_top", 12)
	pad.add_child(rail_column)

	## The mockup opens the rail with a 29 px cell carrying the expand chevron,
	## ruled off from the domains below it. SH-01: now a real button -- pressing
	## it grows the rail to `W_RAIL_EXPANDED` (200 px, §1) and swaps the
	## collapsed domain-button column for `_rail_subnodes_body` below.
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	var head := Button.new()
	head.flat = true
	head.focus_mode = Control.FOCUS_NONE
	head.text = DccIcons.SYMBOLS["expand"]
	head.tooltip_text = "Expand rail"
	head.add_theme_font_override("font", DccTheme.mono())
	head.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	head.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	head.add_theme_stylebox_override("normal", DccTheme.empty())
	head.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
	head.custom_minimum_size.y = _scaled(29)
	head.pressed.connect(_toggle_rail_expansion)
	_rail_expand_button = head
	col.add_child(head)
	col.add_child(DccTheme.rule())

	## SH-01 (`GUI_GAP_REGISTER.md` §7.17): the register's own proposal --
	## "reuses `_phone_list_row()` verbatim" -- extended per owner instruction
	## to carry each domain's *real* sub-structure (`DOMAINS[i].subnodes`, see
	## that const's own header comment) as the row's subtitle line rather than
	## the tooltip blurb the phone drawer uses. There is no destination deeper
	## than the domain itself to jump to -- no engine hook exists to scroll a
	## dock to one specific L2 category -- so a row's only real action is what
	## the phone drawer's own row already does: select that domain, then
	## collapse the rail back (`_pick_rail_domain`), mirroring
	## `_pick_drawer_domain`'s own close-after-pick.
	_rail_subnodes_body = VBoxContainer.new()
	_rail_subnodes_body.add_theme_constant_override("separation", 0)
	_rail_subnodes_body.visible = false
	for d in DOMAINS:
		_rail_subnodes_body.add_child(_phone_list_row(String(d.label), String(d.subnodes),
			_pick_rail_domain.bind(d.id)))
		_rail_subnodes_body.add_child(DccTheme.rule())
	col.add_child(_rail_subnodes_body)

	## Everything the collapsed rail shows -- the domain-button column, the
	## spacer, and the rail foot -- lives in one wrapper so expansion can hide
	## and restore it in a single `visible` flip rather than three.
	_rail_collapsed_body = VBoxContainer.new()
	_rail_collapsed_body.add_theme_constant_override("separation", 0)
	_rail_collapsed_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_rail_collapsed_body.add_child(pad)
	col.add_child(_rail_collapsed_body)
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

	_rail_collapsed_body.add_child(DccTheme.spacer())
	rail_foot = DccTheme.mono_label("", "text_ghost", DccTheme.FS_MICRO, 2)
	rail_foot.rotation = -PI / 2.0
	var foot_holder := Control.new()
	foot_holder.custom_minimum_size.y = 84
	foot_holder.add_child(rail_foot)
	_rail_collapsed_body.add_child(foot_holder)
	return rail

## SH-01: toggles the rail between the collapsed domain-button column and the
## expanded 200 px sub-node list. Symmetric with `_toggle_dock()`'s own
## chevron-flip pattern, just on the rail instead of a dock.
func _toggle_rail_expansion() -> void:
	_rail_expanded = not _rail_expanded
	_rail_panel.custom_minimum_size.x = float(_scaled(DccTheme.W_RAIL_EXPANDED)) \
		if _rail_expanded else float(_scaled(DccTheme.W_RAIL_COLLAPSED))
	_rail_collapsed_body.visible = not _rail_expanded
	_rail_subnodes_body.visible = _rail_expanded
	_rail_expand_button.text = DccIcons.SYMBOLS["collapse"] if _rail_expanded else DccIcons.SYMBOLS["expand"]
	_rail_expand_button.tooltip_text = "Collapse rail" if _rail_expanded else "Expand rail"

## A row in the expanded rail's sub-node list was pressed: jump to that
## domain, then close back to the collapsed rail, mirroring
## `_pick_drawer_domain()`'s own close-after-pick on the phone drawer.
func _pick_rail_domain(id: String) -> void:
	_select_domain(id)
	if _rail_expanded:
		_toggle_rail_expansion()

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
#   3. The chrome column: top safe area → app bar → [the rail floats over the
#      gap below it] → tool sheet → timeline → bottom gesture inset. Wrapped
#      in `_phone_chrome_margin`, whose left margin is what shifts in
#      landscape to clear the side safe area.
#   4. Overlays, all full-rect, all hidden until opened: the side safe area
#      (landscape only), the ☰ domain drawer, the ▤ panel picker, the ⋯
#      overflow sheet, and the left/right dock sheets.
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

	_phone_root.add_child(_build_phone_scrim())

	## Menu bar and status bar keep their exact desktop construction --
	## `add_menu()`/`set_status()` stay phone-unaware -- and relocate into the
	## ⋯ overflow sheet, built *first* so the app bar's own title label
	## (built after, below) claims the "top_world" status slot last and wins
	## it. Both builders register a Label under that key
	## (`_build_menu_bar()`'s status-readout loop, and the app bar's
	## subtitle); `_status_labels` keeps only the most recently registered
	## one live, so this ordering is load-bearing, not incidental.
	var menu_bar := _build_menu_bar()
	var status_bar := _build_status_bar()
	_phone_overflow = _build_phone_overflow(menu_bar, status_bar)
	_phone_root.add_child(_phone_overflow)

	_phone_chrome_margin = MarginContainer.new()
	_phone_chrome_margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	_phone_root.add_child(_phone_chrome_margin)

	var chrome := VBoxContainer.new()
	chrome.add_theme_constant_override("separation", 0)
	_phone_chrome_margin.add_child(chrome)

	_phone_top_safe = _build_phone_top_safe()
	chrome.add_child(_phone_top_safe)

	chrome.add_child(_build_phone_app_bar())

	## The gap between the app bar and the tool sheet. The rail floats over
	## it (`PRESET_LEFT_WIDE`, below) rather than sharing a row with it,
	## because the map underneath is meant to show through the rail's own
	## translucent background here, not stop at a container edge.
	_phone_content_gap = Control.new()
	_phone_content_gap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_phone_content_gap.mouse_filter = Control.MOUSE_FILTER_PASS
	chrome.add_child(_phone_content_gap)
	_phone_content_gap.add_child(_build_phone_rail())

	_phone_tool_sheet = _build_phone_tool_sheet()
	chrome.add_child(_phone_tool_sheet)

	timeline_bar = _build_timeline()
	chrome.add_child(timeline_bar)

	_phone_gesture_inset = _build_phone_gesture_inset()
	chrome.add_child(_phone_gesture_inset)

	_phone_side_safe = _build_phone_side_safe()
	_phone_root.add_child(_phone_side_safe)

	_phone_drawer = _build_phone_drawer()
	_phone_root.add_child(_phone_drawer)

	_phone_panel_picker = _build_phone_panel_picker()
	_phone_root.add_child(_phone_panel_picker)

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

	_apply_phone_orientation()

## Legibility over imagery without an opaque strip (inset rule "SCRIM, NOT A
## BAR"). A `GradientTexture2D`, not a flat colour -- the fade is the point,
## the map should still show faintly through the scrim's lower half. Height
## 96 px = the 44 px safe area plus the 52 px app bar, so the fade finishes
## exactly where the app bar's own opaque background takes over.
func _build_phone_scrim() -> Control:
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

	var scrim := TextureRect.new()
	scrim.texture = tex
	scrim.stretch_mode = TextureRect.STRETCH_SCALE
	scrim.mouse_filter = Control.MOUSE_FILTER_IGNORE
	scrim.set_anchors_preset(Control.PRESET_TOP_WIDE)
	scrim.offset_left = 0
	scrim.offset_right = 0
	scrim.offset_top = 0
	scrim.offset_bottom = _pscale(DccTheme.H_PHONE_TOP_SCRIM)
	return scrim

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

	row.add_child(_phone_bar_button(DccIcons.SYMBOLS["panels"], "Panels",
		func(): _set_panel_picker_open(true), "accent"))
	row.add_child(_phone_bar_button(DccIcons.SYMBOLS["overflow"], "Menu",
		func(): _set_overflow_open(true)))
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

## The domain rail (§13: "a 44 px column with each domain in a 44 px hit
## box"). The mockup's phone screen draws the rail *and* the ☰ drawer
## together (lines 1467 and 1477-1489) -- the drawer is not a replacement for
## the rail, it's a second, larger-target way to reach the same five domains.
## Unlike the desktop rail (`_build_rail()`), the phone cells are text-only:
## no icon, no measured icon/label offset math -- just a centred vertical
## label in a plain 44×44 box, so the centring formula is the simple case
## `_build_rail()`'s own long comment sets up for the icon-biased one: a Label
## rotated -90° about its own top-left corner maps local (u, v) to global
## (v, -u), so its rotated box spans x:[0, text_size.y], y:[-text_size.x, 0]
## relative to `position`. Solving for the position that puts each axis's
## midpoint at the cell's own centre gives the two lines below.
func _build_phone_rail() -> Control:
	var rail := PanelContainer.new()
	rail.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))
	rail.set_anchors_preset(Control.PRESET_LEFT_WIDE)
	rail.offset_left = 0
	rail.offset_right = _ptap(DccTheme.W_PHONE_RAIL)
	rail.offset_top = 0
	rail.offset_bottom = 0

	rail_column = VBoxContainer.new()
	rail_column.add_theme_constant_override("separation", 0)
	rail_column.set_anchors_preset(Control.PRESET_FULL_RECT)
	rail.add_child(rail_column)

	var cell_w := float(_ptap(DccTheme.W_PHONE_RAIL))
	var cell_h := float(_ptap(44))
	for i in DOMAINS.size():
		var d: Dictionary = DOMAINS[i]
		if i > 0:
			var sep := ColorRect.new()
			sep.color = DccTheme.c("line")
			sep.custom_minimum_size = Vector2(_pscale(14), 1)
			sep.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
			rail_column.add_child(sep)

		var b := Button.new()
		b.tooltip_text = "%s -- %s" % [d.label, d.subtitle]
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.custom_minimum_size.y = cell_h
		b.add_theme_stylebox_override("normal", DccTheme.empty())
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
		b.pressed.connect(_select_domain.bind(d.id))

		var vlabel := DccTheme.mono_label(String(d.rail).to_upper(),
			"text_faint", _pscale(9), 2, true)
		vlabel.rotation = -PI / 2.0
		var text_size := vlabel.get_minimum_size()
		vlabel.position = Vector2(cell_w * 0.5 - text_size.y * 0.5,
			cell_h * 0.5 + text_size.x * 0.5)
		b.add_child(vlabel)

		_domain_buttons[d.id] = b
		_domain_marks[d.id] = {"label": vlabel}
		rail_column.add_child(b)

	rail_column.add_child(DccTheme.spacer())
	## The mockup's own bottom "›" (line 1488) -- decorative on the desktop
	## rail too (`_build_rail()`'s equivalent head chevron is a bare Label,
	## never wired to a `pressed` signal). Matched here rather than invented
	## new behaviour for it.
	var foot := DccTheme.mono_label("›", "text_dim", _pscale(12))
	foot.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	foot.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	foot.custom_minimum_size.y = _pscale(32)
	rail_column.add_child(foot)
	return rail

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

## ☰ domain drawer: the same five `DOMAINS`, as full label + subtitle rows
## rather than the rail's tracked abbreviations -- a second, more legible way
## in, not a replacement for the rail (see `_build_phone_rail()`'s comment).
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

## ⋯ overflow: the desktop menu bar and status bar, relocated unmodified. Both
## are wrapped for width -- the menu bar's seven `MenuButton`s plus its
## wordmark and readout cluster are sized for a ~1500 px+ desktop bar and
## overflow 393 px, so it scrolls horizontally; each `MenuButton` still opens
## its own popup normally once tapped. The status bar's few short labels fit
## without one.
func _build_phone_overflow(menu_bar: Control, status_bar: Control) -> Control:
	var overlay := _phone_overlay_scrim(func(): _set_overflow_open(false))

	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.panel("raised", {"top": 1}))
	panel.set_anchors_preset(Control.PRESET_BOTTOM_WIDE)
	panel.offset_left = 0
	panel.offset_right = 0
	panel.offset_top = -_pscale(220)
	panel.offset_bottom = -_pscale(DccTheme.H_PHONE_GESTURE)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	panel.add_child(col)

	var head := HBoxContainer.new()
	head.custom_minimum_size.y = _ptap(44)
	head.add_child(DccTheme.header("Menu", ""))
	head.add_child(DccTheme.spacer())
	head.add_child(_sheet_close_button(func(): _set_overflow_open(false)))
	var hp := MarginContainer.new()
	hp.add_theme_constant_override("margin_left", 14)
	hp.add_theme_constant_override("margin_right", 6)
	hp.add_child(head)
	col.add_child(hp)
	col.add_child(DccTheme.rule())

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	scroll.add_child(menu_bar)
	col.add_child(scroll)
	col.add_child(DccTheme.rule())
	col.add_child(status_bar)

	overlay.add_child(panel)
	overlay.visible = false
	return overlay

# -- Phone overlay state ---------------------------------------------------
#
# Every phone overlay is mutually exclusive -- opening any one closes all the
# others, including a dock sheet -- so there is exactly one state variable per
# overlay and one shared teardown rather than a general stack.

func _close_all_phone_overlays() -> void:
	if _phone_drawer != null:
		_phone_drawer.visible = false
	if _phone_panel_picker != null:
		_phone_panel_picker.visible = false
	if _phone_overflow != null:
		_phone_overflow.visible = false
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

func _set_overflow_open(open: bool) -> void:
	_close_all_phone_overlays()
	_phone_overflow.visible = open

func _set_sheet_open(side: String, open: bool) -> void:
	if open:
		_close_all_phone_overlays()
	if side == "left":
		_left_sheet_open = open
		left_dock.visible = open
	else:
		_right_sheet_open = open
		right_dock.visible = open

## Re-applied on every resize while `_phone` is true (`_on_window_resized()`),
## which is the one part of the phone/tablet decision that genuinely must be
## live -- a device rotates at runtime even though its form factor never
## does. Only safe-area visibility, the chrome column's left margin, and the
## two dock sheets' rects change between orientations; the drawer/panel-
## picker/overflow overlays and the tool sheet/timeline/rail are unaffected,
## so this never touches anything a workspace has attached content to.
func _apply_phone_orientation() -> void:
	if _phone_top_safe == null:
		return  ## Not built yet -- called once more at the end of `_build_phone_shell()`.
	_phone_top_safe.visible = not _landscape
	_phone_side_safe.visible = _landscape

	## "The domain rail shifts inward" (inset rule, LANDSCAPE): the rail
	## floats inside this margin, so growing it by the side safe area's own
	## width is the entire mechanism -- no separate rail repositioning code.
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
	var left := float(_ptap(DccTheme.W_PHONE_RAIL))
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
	if _phone_tool_sheet != null and _phone_tool_sheet.size.y > 0.0:
		var h := _phone_tool_sheet.size.y + float(_pscale(DccTheme.H_PHONE_GESTURE))
		if timeline_bar != null and timeline_bar.visible:
			h += timeline_bar.size.y
		return h + float(_pscale(8))
	return float(_pscale(20 + 90 + DccTheme.H_PHONE_GESTURE))
