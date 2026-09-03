extends PopupPanel
class_name LayersPopover

## The map canvas's Layers popover (`DCC_SHELL_SPEC.md` §9's layers button),
## ported from the reference's own canvas popover (`buildLayersPopover`,
## reference HTML line 13657) rather than invented.
##
## **What the layers button used to do.** It emitted `layers_button_pressed`
## and `app.gd` answered by selecting the Cartography domain, whose left dock
## has a "Visible layers" section. That was a stand-in for this: a button
## labelled Layers that jumped the whole workspace rather than opening
## anything. The stand-in is replaced, not extended -- but nothing it reached
## is removed: `cartography_workspace.gd`'s own toggles and
## `ViewportHost.set_layer_visible()` are untouched, still on the rail, and
## the note at the foot of this popover points at them.
##
## **The grouping is the reference's, verbatim.** `LAYER_GROUPS` in
## `sample_bridge.rs` keeps the original's Base / Climate / Tectonics /
## Hydrology / Surface / Civilization headings and their order, restricted to
## the views this port can actually draw from state generation already
## retains, plus four it adds for Sample fields the reference never had a
## view for (elevation, slope, aspect, resistance -- each says so in its own
## hint). The engine owns that table; this file restates nothing of it but
## the four ids in `GAP_LAYERS` below, each with the one sentence the popover
## prints under that row's engine hint -- and that one exception exists only
## because `layer_available()` answers a single `bool`, which cannot say
## *which* ground a refusal was on. Otherwise it is the same rule
## `journey_planner_view.gd` follows for `jp_options()`.
##
## **Rows that cannot work are disabled, not hidden.** A view whose one input
## this world lacks (Strahler order without river extraction, biomes/terrain/
## control on a loaded save) comes back `available: false` from the engine
## and is drawn greyed with its reason in the tooltip -- §6's own "fields
## from stale stages read —" rule, applied to a picker.
##
## Like the reference's, this popover stays open across picks: it holds many
## independent choices plus an opacity slider, and closing on every click
## would make comparing two views needlessly tedious. Click outside or press
## Escape to dismiss.

const FLOW_FX_SCRIPT := preload("res://shell/wind_fx_layer.gd")

## `sample_bridge.rs`'s `GAP_LAYERS` (`sample_bridge.rs:741 const GAP_LAYERS`),
## restated -- the one thing this file copies out of the engine's
## table, and only because `layer_available()` (`sample_bridge.rs:751 pub fn
## layer_available`) collapses every refusal into a single `bool` that cannot
## say which ground it was on. Four ids, not a second copy of `LAYER_GROUPS`,
## each carrying the sentence this popover prints beneath that row's own
## engine hint.
##
## **Two grounds, not one**, in the engine's own words (`GAP_LAYERS`' doc
## comment there): "Two remain honestly unavailable for a *missing
## computation* (`oro`, `velo`) and two for a missing composite
## (`popdensity`, `siteprofile`)." There is no third, "missing estimator"
## category -- an earlier draft of this file printed one for all four, which
## contradicted two of the hints it was printed under. Each sentence below is
## the substance of that id's own hint in `LAYER_GROUPS`, so the two lines of
## the tooltip cannot disagree.
##
## If a gap is ever closed on the Rust side, drop it here too; a stale entry
## cannot mislabel a working row, since a closed gap comes back `available`
## and never asks for a suffix at all.
const GAP_LAYERS := {
	## `sample_bridge.rs:597`, the `LAYER_GROUPS` `"oro"` hint -- "needs the
	## boundary-polyline structure generate_terrain folds into height and
	## never retains". Unretained input, so not a "no ... exists" sentence.
	"oro":
		"Never available: a missing computation. The boundary polylines it " +
		"draws from are folded into height and retained by no world.",
	## `sample_bridge.rs:626`, the `LAYER_GROUPS` `"velo"` hint -- "no hydraulic
	## velocity-erosion pass ... exists in cartalith-erosion".
	"velo":
		"Never available: a missing computation. No hydraulic velocity-" +
		"erosion pass exists in this engine.",
	## `sample_bridge.rs`'s `LAYER_GROUPS` `"popdensity"` hint -- "no regional
	## population-density estimator exists in this engine".
	##
	## **That engine hint is false, and it is the FIRST line of this row's
	## tooltip.** `cartalith_civ::estimate_regional_density_km2` exists, is
	## golden-tested (`golden_parity_carrying_capacity.rs`), and is already
	## reached from a shipped `#[func]`: `civ_regional_population()`
	## (`ops_bridge.rs`) builds the full per-cell `dens` field with it and then
	## integrates it away to one world total. The sentence below is corrected to
	## the true narrow gap. **Closed 2026-09-03:** the Rust hint was corrected in
	## the following pass, so the two lines of this tooltip now agree.
	## `sample_bridge.rs` names the real gap -- the estimator runs but its field
	## is integrated away to one total, so no drawable raster is retained.
	"popdensity":
		"Never available: a missing composite. The per-cell estimator does " +
		"exist and civ_regional_population() runs it, but that binding " +
		"integrates the field away to a single total -- nothing keeps it as a " +
		"drawable raster.",
	## `sample_bridge.rs:698`, the `LAYER_GROUPS` `"siteprofile"` hint -- "the
	## flood + slope buildability composite has no Rust equivalent beyond its
	## two inputs individually". Those two inputs are rows of their own here:
	## the `LAYER_GROUPS` `"flood"` row (`sample_bridge.rs:633`) and its
	## `"slope"` row (`sample_bridge.rs:655`), both available.
	"siteprofile":
		"Never available: a missing composite. Only its two inputs exist, " +
		"and they draw on their own rows: Flood and Slope.",
}

var bridge: EngineBridge
var host: ViewportHost

var _list: VBoxContainer
var _legend: VBoxContainer
var _rows: Dictionary = {}   ## view id -> its Button

## Hotkey badges 1-8 (`DCC_SHELL_SPEC.md` §10: "grouped rows with hotkey
## badges: SURFACE (Relief 1, Biome 2, Political 3), TERRAIN FIELDS
## (Elevation 4, Slope 5, Flow accumulation 6), CLIMATE (Temperature 7,
## Rainfall 8)"). That grouping does not exist in this popover's own data:
## `LAYER_GROUPS` (`sample_bridge.rs`) is the *reference's* verbatim
## Base/Climate/Tectonics/Hydrology/Surface/Civilization order (this file's
## own header comment above explains why it stays that way), which has no
## "Relief" row at all (the closest is `off`, "No overlay (base map)") and
## puts Political under Civilization, last, not third. Re-sorting rows
## client-side to chase the spec's naming would scatter hotkeys 1-8 across
## non-adjacent groups with no visual grouping to match -- a bigger, riskier
## change than this badge itself. Badging the first 8 rows in their real,
## already-built order instead (`DCC_CONTROL_INDEX.md`'s own tolerance for
## "uncertain" mappings). Noted here and in `GUI_GAP_REGISTER.md`.
##
## **Only *available* rows are badged**, which is why `rebuild()` counts them
## itself rather than badging `LAYER_GROUPS`' first eight entries outright.
## Eleven rows can come back unavailable, on **two different grounds**
## (`sample_bridge.rs:751 pub fn layer_available`). **Four** are the permanent
## engine gaps (`GAP_LAYERS` -- `oro`, `velo`, `popdensity`, `siteprofile`):
## disabled on every world that will ever exist, so a digit spent on one is a
## digit that does nothing, forever. The other **seven** (`strahler`,
## `bclass`, `cterrain`, `windthrow`, `wildlife`, `control`, `contested`) are
## refused only while *this* world lacks their one input, and become available
## the moment it gains it. Skipping unavailable rows is right for both, and
## for the second group it has to be re-decided rather than decided once --
## which it is: `rebuild()` re-counts from scratch on every open and on both
## `generation_finished` and `world_loaded`, so a row that turns available
## takes a digit and the rows after it shift down.
## Counting positionally spent a digit on one anyway: when the seven new
## Climate views landed (Wind, Ocean currents, ...), Köppen -- a gap row
## at the time, since ported -- shifted into slot 4, and pressing `4`
## silently no-opped from then on. Skipping unavailable rows keeps all
## eight digits live, and keeps them stable
## against a future row landing in the middle of a group.
const HOTKEY_COUNT := 8
const HOTKEY_ACTIONS: Array[String] = [
	"layers_hotkey_1", "layers_hotkey_2", "layers_hotkey_3", "layers_hotkey_4",
	"layers_hotkey_5", "layers_hotkey_6", "layers_hotkey_7", "layers_hotkey_8",
]
var _hotkey_ids: Array = []   ## index 0-7 -> the row id badged with that digit.

## Phone (§13) -- PH-12, and this one had to be checked before it was built:
## a popover may simply be the wrong control on a handset, and §13's phone
## composition routes several desktop affordances into the ⋯ overflow sheet
## instead. **It is reachable, by three routes**: the map's own Layers button
## (`viewport.layers_button_pressed`, `app.gd`), Cartography ▸ *Data overlays…*
## (`cartography_workspace.gd`) and the Render section's own entry
## (`render_workspace.gd`). So it needs real work, and the parallel device
## sweep measured what that means: **40 of 52 tappable controls under §13's
## floor**, rows at 22 dp and the opacity slider at 14.
##
## It becomes a full-screen sheet rather than a scaled-down popover. A popover
## is a pointer idiom -- it is anchored to the control that opened it and
## dismissed by clicking away from it, and a phone has neither a stable anchor
## (the Layers button moves with the safe insets) nor a reliable "away". §13's
## own answer for a panel on a phone is a sheet, so that is what this is.
##
## `DccWidgets.phone_window()` takes an `AcceptDialog` and this is a
## `PopupPanel`, so only the two halves that apply are used: `phone_present()`
## (which takes any `Window`) for the fill and the content scale, and a
## `phone_head()` with an explicit Close, because a sheet that covers the
## screen has no "outside" left to tap.
var _phone := false
var _close_row: Control

func setup(b: EngineBridge, h: ViewportHost) -> void:
	bridge = b
	host = h
	_register_hotkeys()
	add_theme_stylebox_override("panel",
		DccTheme.panel("panel", {"left": 1, "right": 1, "top": 1, "bottom": 1}))
	var shell: Node = get_parent()
	_phone = shell != null and shell.has_method("is_phone") and shell.is_phone()
	if _phone:
		## The half of `phone_window()` that applies to a `Popup`: with
		## `wrap_controls` on, the window grows to its content's minimum on every
		## `child_controls_changed()` and only ever grows, which fights a fill.
		## `phone_window()` itself is not callable here -- it takes an
		## `AcceptDialog`, for the `ok_button_text` and the borderless title bar
		## a popup does not have in the first place.
		wrap_controls = false

	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	if not _phone:
		## `--popW` (`ENV:902`), the token this popover is the only consumer of.
		## It replaces a bare `228` here, in the scroll below and in the `popup()`
		## rect -- one figure, written once. BUILD_ANSWERS §2.4 calls the width
		## one of "three unscaled values -- all three now scale. They were
		## oversights", so the pointer figure moves 228 -> 238 (the prototype's
		## own `--popW`, which the literal predated) and touch gets 300. `LAPTOP`
		## deliberately leaves `--popW` at base, so a 1366 frame keeps 238.
		outer.custom_minimum_size = Vector2(DccTheme.role_px("w_popover"), 0)
	add_child(outer)
	if _phone:
		DccWidgets.phone_head(outer, "Data overlays", "one field view at a time")

	var scroll := ScrollContainer.new()
	## PH-12: `--popW` x 420 is a popover's authored size. As a full-screen sheet
	## the width comes from the screen and the height from what is left under the
	## header, and a 420 dp FLOOR under a legend, a slider and a note would push
	## the foot off the bottom of a 393x852 reference screen. (The height stays a
	## literal: `ROLE` carries no popover height, and the tablet artboard's own
	## popover is measured by its rows, not by a fixed box.)
	if not _phone:
		scroll.custom_minimum_size = Vector2(DccTheme.role_px("w_popover"), 420)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(scroll)

	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 0)
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## PH-12: on a phone the foot goes INSIDE the scroll, under the list, rather
	## than being a fixed band below it. On a pointer the foot is a legend, a
	## slider and a two-line note -- small enough to keep pinned. At 393 dp the
	## same note is six lines, and pinned it pushed itself off the bottom edge
	## where no scroll could reach it (measured: the last two lines of the
	## Cartography cross-reference clipped at the screen edge).
	var scroll_body: Control = _list
	if _phone:
		var col := VBoxContainer.new()
		col.add_theme_constant_override("separation", 0)
		col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		col.add_child(_list)
		scroll_body = col
	scroll.add_child(scroll_body)

	var foot := VBoxContainer.new()
	foot.add_theme_constant_override("separation", 2)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 10)
	pad.add_theme_constant_override("margin_right", 10)
	pad.add_theme_constant_override("margin_top", 6)
	pad.add_theme_constant_override("margin_bottom", 8)
	pad.add_child(foot)
	if _phone:
		scroll_body.add_child(DccTheme.rule())
		scroll_body.add_child(pad)
	else:
		outer.add_child(DccTheme.rule())
		outer.add_child(pad)

	_legend = VBoxContainer.new()
	_legend.add_theme_constant_override("separation", 1)
	foot.add_child(_legend)

	DccWidgets.slider(foot, "Opacity", 0.0, 100.0, 1.0,
		host.debug_opacity() * 100.0, "%",
		func(v: float): host.set_debug_opacity(v / 100.0),
		"Blends the active field raster over the base map, so terrain reads " +
		"through it. The reference's own #dbgOpacity.")

	## PH-12: a full-screen sheet has no "outside" to tap, so the way out has to
	## be inside it. (Android back also closes it -- `DccShell::_notification`
	## hides the topmost subwindow first, and this is one -- but a visible
	## control is not optional for a gesture that has no on-screen affordance.)
	if _phone:
		var close := Button.new()
		close.text = "Close"
		close.focus_mode = Control.FOCUS_NONE
		close.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		close.pressed.connect(func(): hide())
		foot.add_child(close)
		_close_row = close

	DccWidgets.note(foot,
		"Settlement, road, sea-route and town-layout visibility live in " +
		"Cartography ▸ Layers on the rail; way types are Cartography ▸ Roads & " +
		"routes and the two political layers are Cartography ▸ Political " +
		"display. All of those are vector overlays drawn from world data, not " +
		"field rasters, and they toggle rather than replace one another. Town " +
		"layouts draw themselves once the map spans under 24 km.")

	bridge.generation_finished.connect(func(_ok: bool): if visible: rebuild())
	bridge.world_loaded.connect(func(): if visible: rebuild())

	_attach_flow_fx()

## The Wind and Ocean-currents rows are the only two field views the reference
## also *animates* (`#windFxCanvas`, reference HTML lines 2113-2209): particle
## streaks advected along the flow field, over the static raster. `app.gd`
## builds exactly one of this popover and this runs from `setup()`, so the one
## overlay node is created once here rather than on each pick -- it is idle
## (invisible, holding no field) until `host.debug_view()` actually reads back
## one of those two, which it polls for itself. See `wind_fx_layer.gd`.
##
## Parented under `host.overlay`, not this popover: the streaks belong in the
## map's own zoomed/panned coordinate space, and `map_overlay.gd` is already
## the node that carries it (and publishes the letterbox fit rect the
## particles project through). Attached from here because `set_debug_layer`
## lives on `ViewportHost`, which is owner-reserved for concurrent work --
## this popover is the only other place that knows a field view was picked.
func _attach_flow_fx() -> void:
	if host == null or host.overlay == null:
		return
	## `preload`, not the `WindFxLayer` global class name: a global name only
	## resolves once the editor has rescanned and written
	## `.godot/global_script_class_cache.cfg`, so a fresh clone (or any
	## editor-less run, which is how this port's capture harnesses drive the
	## shell) would fail to parse this file. `viewport_host.gd`'s own
	## `OVERLAY_SCRIPT` preload is here for the same reason.
	var fx: Control = FLOW_FX_SCRIPT.new()
	fx.name = "WindFxLayer"
	fx.setup(bridge, host)
	host.overlay.add_child(fx)

## Anchored under the viewport's own layers button rather than at a guessed
## corner offset -- the button moves with `set_safe_insets()` on phone.
func open() -> void:
	rebuild()
	## PH-12. `phone_present()` takes any `Window`, not only an `AcceptDialog`,
	## so a `PopupPanel` gets the identical fill and content scale every other
	## phone surface gets. Returns false on desktop and tablet, where the
	## anchored popover below is exactly right.
	if DccWidgets.phone_present(self, get_parent()):
		_phone_fit()
		return
	var r := host.layers_button_rect()
	popup(Rect2i(Vector2i(r.position.x, r.position.y + r.size.y + 4),
		Vector2i(DccTheme.role_px("w_popover"), 0)))

## `1.0`: `phone_present()` applies the scale once as `content_scale_factor`.
## Re-run after every `rebuild()`, because the rows are all fresh nodes; it is
## idempotent by meta-flag, so only the new ones are touched.
func _phone_fit() -> void:
	if not _phone:
		return
	var shell: Node = get_parent()
	if shell != null and shell.has_method("phone_fit"):
		shell.phone_fit(self, 1.0)

func rebuild() -> void:
	for child in _list.get_children():
		_list.remove_child(child)
		child.queue_free()
	_rows.clear()
	_hotkey_ids.clear()

	var groups := bridge.debug_layers()
	if groups.is_empty():
		DccWidgets.note(_list,
			"No field views: this build's engine has no debug_layers() binding.")
		_refresh_legend([])
		return

	var current := host.debug_view()
	var row_i := 0   ## Running count of *available* rows across every group --
		## `HOTKEY_ACTIONS`' own doc comment on why this badges the first 8 in
		## build order rather than the spec's own SURFACE/TERRAIN FIELDS/
		## CLIMATE grouping, and why a disabled row never consumes a digit.
	for g in groups:
		var group: Dictionary = g
		var body := DccWidgets.section(_list, String(group["group"]))
		for it in group["items"]:
			var item: Dictionary = it
			var hotkey := -1
			if bool(item["available"]) and row_i < HOTKEY_COUNT:
				hotkey = row_i
				_hotkey_ids.append(String(item["id"]))
				row_i += 1
			_rows[String(item["id"])] = _row(body, item, current, hotkey)
	_refresh_legend(_legend_for(current, groups))
	_phone_fit()   ## PH-12 -- every row above is a fresh node.

## `GUI_GAP_REGISTER.md` §57 / `UNWIRED_FUNCTIONS.md` "the tablet interior
## walk". §57's own DS-03 audit found the tablet artboard deletes this popover
## entirely to buy room for its 44 px rows elsewhere -- that is a content
## question for the owner (`UNWIRED_FUNCTIONS.md`'s companion row), explicitly
## not this pass's to decide, so nothing here is hidden. What this pass owns
## is sizing: a row the artboard would have kept still has to hit its target,
## which is `role_px("row_min_h")`/`"fs_prose"` -- a plain `Button` with no
## font override, i.e. prose, same as `DccWidgets._row()`'s own label.
func _row(parent: Control, item: Dictionary, current: String, hotkey: int = -1) -> Button:
	var id := String(item["id"])
	var available: bool = bool(item["available"])
	var tablet := DccTheme.is_tablet()
	var b := Button.new()
	b.text = String(item["label"])
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.custom_minimum_size.y = DccTheme.role_px("row_min_h") if tablet else 22
	b.add_theme_font_size_override("font_size",
		DccTheme.role_px("fs_prose") if tablet else DccTheme.FS_SMALL)
	## Two kinds of refusal, two kinds of sentence. The seven conditional ones
	## really are about which inputs *this* world retained, and keep the
	## original wording. A `GAP_LAYERS` row is refused on a ground no world can
	## change, so "for this world" beside it read as a self-contradiction; its
	## sentence comes per id from the table above, because the four hints do
	## not share one form. All four open "Not available:", but only `velo` and
	## `popdensity` go on to say "no ... exists" -- `oro` says its input is
	## never *retained*, and `siteprofile` that the *composite* has no Rust
	## equivalent though both its inputs do.
	var reason := String(GAP_LAYERS.get(id, "Not available for this world."))
	b.tooltip_text = String(item["hint"]) if available else \
		String(item["hint"]) + "\n\n" + reason
	b.disabled = not available
	b.add_theme_color_override("font_color",
		DccTheme.c("text" if available else "text_ghost"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	var font_color := DccTheme.c("text" if available else "text_ghost")
	if id == current:
		## **The one filled surface in the desktop canvas.** `GUI_GAP_REGISTER
		## .md` §48 (DS-02) searched `DCC shell 1920` for `background:#e0a34a`,
		## found slider fills and "exactly one other hit -- a *selected layer
		## row* in the layers popover", removed 141 filled action buttons and
		## did not then apply the one the search had found. The canvas's row is
		## `display:flex;align-items:center;padding:4px 8px;background:#e0a34a;
		## color:#1a1206;font-weight:600` -- a slab, not `active_row()`'s wash
		## and underline, which is the treatment for a menu-bar title and a dock
		## category header rather than for a picked item in a list.
		##
		## `#1a1206` is not a palette token and is not made one: it is the
		## reversed paper ink §11 requires on a filled accent surface, which
		## `bg` (`#0d0e0f` dark, `#f4f2ee` light) already is in both themes --
		## and unlike a literal it follows a theme switch.
		##
		## **`flat` has to come off.** A `Button` with `flat = true` skips its
		## `normal`/`hover`/`pressed` styleboxes entirely, so the override that
		## used to sit here -- `active_row()`, put on a flat button -- had never
		## drawn anything at all. The active row was distinguished only by its
		## accent ink and its badge opacity, and the first capture of this fix
		## showed a *blank* row where the slab was supposed to be, which is how
		## the older override was caught too.
		b.flat = false
		var slab := DccTheme.flat(DccTheme.c("accent"))
		slab.content_margin_left = 8
		slab.content_margin_right = 8
		slab.content_margin_top = 4
		slab.content_margin_bottom = 4
		for sb_name in ["normal", "hover", "pressed", "disabled"]:
			b.add_theme_stylebox_override(sb_name, slab)
		## The canvas also sets `font-weight:600` on this row. The shell's prose
		## face is `FiraSans-Regular.ttf` and no bold cut is loaded (`dcc_theme
		## .gd` preloads two Plex weights and one Fira), so the weight is left
		## alone rather than faked with an outline. Recorded, not silently
		## dropped.
		## Reversed ink on the filled accent slab: `accent_ink` since the
		## 2026-08-31 token re-base, `c("bg")` before it. The prototype agrees
		## on the rule and on the site -- `layerRows` at `ENV:1888` sets the
		## selected row to `col:'var(--accInk)'` and its key hint to the same,
		## against `var(--body)`/`var(--faint)` when unselected.
		b.add_theme_color_override("font_color", DccTheme.c("accent_ink"))
		b.add_theme_color_override("font_hover_color", DccTheme.c("accent_ink"))
		b.add_theme_color_override("font_pressed_color", DccTheme.c("accent_ink"))
		b.add_theme_color_override("font_disabled_color", DccTheme.c("accent_ink"))
		font_color = DccTheme.c("accent_ink")
	if available:
		b.pressed.connect(_on_pick.bind(id))
	parent.add_child(b)
	if hotkey >= 0:
		_add_hotkey_badge(b, hotkey, font_color, id == current)
	return b

## The mockup's own badge markup (`design/Cartalith DCC Shell.dc.html`, the
## Layers popover's Relief/Biome/Political/... rows): `font:9px 'IBM Plex
## Mono'`, `border:1px solid currentColor`, `padding:0 4px`, opacity .75 on
## the active row and .55 otherwise -- reproduced directly rather than
## invented, down to the opacity split. `currentColor` becomes `font_color`
## (the same colour the row's own label was just set to) at reduced alpha,
## since Godot StyleBox/Label colours have no "inherit the text colour"
## concept.
##
## A child of the row `Button`, not a sibling in a wrapping `HBoxContainer`:
## a `Button` is not itself a layout container, but it *is* a plain
## `Control`, so anchoring a child directly to its right edge reaches the
## same visual result -- flush against the row's own right edge, inside its
## already-existing background/hover/active stylebox -- without splitting
## the row's hit box in two. Anchors are computed from `get_minimum_size()`
## rather than baked via `set_anchors_preset()`, the same trap
## `ViewportHost._chrome()`'s own doc comment names: a preset call bakes
## offsets from the control's size *at that moment*, which is zero before
## the button has ever been laid out.
func _add_hotkey_badge(button: Button, hotkey: int, font_color: Color, is_active: bool) -> void:
	var badge := Label.new()
	badge.text = str(hotkey + 1)
	badge.add_theme_font_override("font", DccTheme.mono())
	badge.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
	var badge_color := font_color
	badge_color.a = 0.75 if is_active else 0.55
	badge.add_theme_color_override("font_color", badge_color)
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0, 0, 0, 0)
	sb.border_color = badge_color
	sb.set_border_width_all(1)
	sb.content_margin_left = 4
	sb.content_margin_right = 4
	badge.add_theme_stylebox_override("normal", sb)
	badge.mouse_filter = Control.MOUSE_FILTER_IGNORE
	button.add_child(badge)

	badge.anchor_left = 1.0
	badge.anchor_right = 1.0
	badge.anchor_top = 0.5
	badge.anchor_bottom = 0.5
	badge.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	var sz := badge.get_minimum_size()
	badge.offset_right = -10.0
	badge.offset_left = -10.0 - sz.x
	badge.offset_top = -sz.y * 0.5
	badge.offset_bottom = sz.y * 0.5

## Runs once -- `app.gd` builds exactly one `LayersPopover` and never frees
## it, so `setup()` (its one-time init point, matching every other method in
## this file) only ever calls this once per session. Guarded on `has_action`
## anyway, cheaply, rather than trusting that.
##
## Registered at runtime rather than declared in `project.godot`: this
## popover is the only place these eight digits mean anything, and `_input()`
## below already scopes them to "popover visibly open," so a project-wide
## `[input]` entry would only invite a second, unintended consumer.
func _register_hotkeys() -> void:
	for i in range(HOTKEY_ACTIONS.size()):
		var action := HOTKEY_ACTIONS[i]
		if InputMap.has_action(action):
			continue
		InputMap.add_action(action)
		var ev := InputEventKey.new()
		ev.physical_keycode = KEY_1 + i
		InputMap.action_add_event(action, ev)

## Hotkey badges 1-8 pick the same row their click already does. Scoped to
## "popover visibly open" by the `visible` guard below -- a `PopupPanel`
## stays in the scene tree while hidden (this node is never freed), so
## without that check the digit keys would fire from anywhere in the shell,
## which is not what a popover-local hotkey means.
func _input(event: InputEvent) -> void:
	if not visible:
		return
	for i in range(_hotkey_ids.size()):
		if event.is_action_pressed(HOTKEY_ACTIONS[i]):
			var id: String = _hotkey_ids[i]
			var row: Button = _rows.get(id)
			if row != null and not row.disabled:
				_on_pick(id)
			get_viewport().set_input_as_handled()
			return

func _on_pick(id: String) -> void:
	host.set_debug_layer(id)
	## Rebuilt rather than re-styled in place: `set_debug_layer` is allowed
	## to refuse (a view the engine could not draw falls back to "off"), and
	## reading `debug_view()` back is the only honest way to know which row
	## should be lit.
	rebuild()

func _legend_for(view: String, groups: Array) -> Array:
	for g in groups:
		var group: Dictionary = g
		for it in group["items"]:
			var item: Dictionary = it
			if String(item["id"]) == view:
				return item["legend"]
	return []

func _refresh_legend(entries: Array) -> void:
	for child in _legend.get_children():
		_legend.remove_child(child)
		child.queue_free()
	for e in entries:
		var entry: Dictionary = e
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		var sw := ColorRect.new()
		sw.color = Color8(int(entry["r"]), int(entry["g"]), int(entry["b"]))
		sw.custom_minimum_size = Vector2(11, 11)
		sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		row.add_child(sw)
		row.add_child(DccTheme.label(String(entry["label"]), "text_dim", DccTheme.FS_MICRO))
		_legend.add_child(row)
