extends Control
class_name ViewportHost

## The map surface (`DCC_SHELL_SPEC.md` §9).
##
## §9 lets the viewport carry exactly five things beyond the map itself: the
## brush cursor, the layers button, the scale bar, the projection/zoom readout
## and the cursor coordinates. Anything else that could live in a dock does.
## This node builds those and nothing more.
##
## The three raster layers stack in draw order -- terrain, territory fill,
## province boundaries -- and `map_overlay.gd` draws the vector layers
## (settlements, roads, sea routes) on top from world data, re-fitting itself
## every frame so a window resize needs no notification.

signal settlement_selected(data: Variant, index: int)
signal settlement_hovered(data: Variant, index: int)
signal cursor_sampled(gx: float, gy: float, valid: bool)
signal layers_button_pressed()
signal map_clicked(gx: float, gy: float)   ## §4.5 tool click-placement primitive.
signal map_dragged(gx: float, gy: float)   ## §4.5 tool drag-paint primitive.
signal map_released(gx: float, gy: float, valid: bool)   ## §4.5 tool drag-end primitive.
## `_civCtxShow`'s right-click (`map_overlay.gd`'s own signal, re-emitted).
signal map_right_clicked(gx: float, gy: float, hit: int, screen_pos: Vector2)

const OVERLAY_SCRIPT := preload("res://map_overlay.gd")
## Deep-zoom tile compositing -- see `_build_lod_tile()` and the shader's own
## header. A tile texture is a relief-detail shade ratio, not a picture; this
## is what turns it back into map pixels using `map_view`'s own colours.
const LOD_TILE_SHADER := preload("res://shell/lod_tile.gdshader")

var map_view: TextureRect
var territory_view: TextureRect
var province_view: TextureRect
var overlay: Control
var tool_overlay: ToolOverlay
var _preview_layer: TextureRect   ## Sculpt/Paint's live draft raster. See `set_preview_texture()`.
var _debug_layer: TextureRect     ## The Layers popover's field raster. See `set_debug_layer()`.
var _debug_view := "off"          ## Which view `_debug_layer` currently holds.

var _scale_label: Label
var _readout_label: Label
var _coords_label: Label
var _layers_btn: Button
## The touch navpad (`GUI_GAP_REGISTER.md` SH-14) and its one stateful member.
## `null` on desktop -- see `_build_navpad()` for the reachability call.
var _navpad: VBoxContainer
var _pan_btn: Button
var _bridge: EngineBridge
var _width_km := 0.0

## Default matches the original hardcoded 10 px inset every corner label and
## the layers button used before phone chrome existed. `DccShell` overrides
## this (`set_safe_insets()`) once it knows its own chrome's edges -- on
## phone the map (and this node with it) is edge-to-edge behind that chrome,
## so a flat 10 px would land this node's own chrome under the app bar/rail/
## tool sheet instead of in the visible gap between them.
var _safe_insets := {"left": 10.0, "top": 10.0, "right": 10.0, "bottom": 10.0}

## §13's 44 px floor applies here too -- this button is real, on-screen, and
## tappable on phone, not workspace/dock content this file is exempt from
## touching. Tablet gets the same floor (its own target range is "44-52 px");
## only pointer-first Windows keeps the original compact 26 px hit box.
##
## `--force-touch` is `DccShell._ready()`'s own testing-only override, adopted
## here verbatim for the same reason it exists there: no dev/CI box has touch
## hardware, so without it `_build_navpad()` below is unreachable outside a
## real device and could only ever be verified on one.
var _touch := (DisplayServer.is_touchscreen_available() and OS.has_feature("mobile")) \
	or "--force-touch" in OS.get_cmdline_user_args()

# -- Camera (§4.5.1 Pan / zoom) ------------------------------------------------
#
# `LOD_TILING_BASE_SCOPE.md` (2026-08-17) deliberately kept `cartalith-spatial`'s
## tiling standalone -- "no camera... revisit when a concrete need appears
## rather than building it speculatively." That need was named directly
## (owner, 2026-08-19: "don't forget to wire zoom/pan"), and §4.5.1 has always
## specified it as a global tool, present in every domain. This is that wiring
## -- a camera transform over the single existing raster, nothing about tiled
## LOD, which is its own, separately scoped decision
## (`LOD_TILING_INTEGRATION_SCOPE.md`).
var _camera: Control    ## Hosts map_view/territory_view/province_view/overlay;
	## `scale`/`position` on THIS node is the whole transform. Chrome
	## (`_scale_label` etc.) stays a direct child of `self`, outside it, so it
	## is never itself panned or zoomed.
const ZOOM_MIN := 0.4
const ZOOM_MAX := 8.0
const ZOOM_WHEEL_STEP := 1.15   ## Multiplicative per wheel notch.
var _zoom := 1.0
var _panning := false
var _pan_last_screen := Vector2.ZERO
## The navpad's ✋ (`GUI_GAP_REGISTER.md` SH-14) -- the reference's `panMode`
## (9582), which despite the ✋ glyph is a **latching toggle**, not a
## press-and-hold: `panBtn`'s whole handler is `panMode=!panMode` (13963).
## Checked before assuming, because "hold to pan" is what the button looks
## like it means. While it is on, the reference gives a plain button-0
## pointerdown to the pan drag (9623) and suppresses the armed tool
## (13924) -- both of which fall out of `_panning` below for free.
var _pan_mode := false

# -- Deep-zoom tile compositing (`LOD_TILING_INTEGRATION_SCOPE.md` milestone M1) ---
#
## The gap M1 exists to close: `_raster()` below sets `TEXTURE_FILTER_NEAREST`
## on `map_view`, so the moment a cell occupies more than one screen pixel the
## base raster shows visibly blocky single-cell squares -- the reference's own
## owner complaint the scope document quotes verbatim (`docs/HANDOFF.md`,
## "There is still a certain pixilated quality to the map when we zoom").
## `_update_lod()` below is what switches the affected screen region over to
## `amplify_region`/`refine_tile`-synthesized tiles (already ported and
## golden-tested, `cartalith-terrain::amplify`/`tile_render` -- this file adds
## no new numerical logic of its own, only screen<->grid geometry, the same
## kind `_zoom_at` below already does) once that threshold is actually
## crossed. Below the threshold this is a complete no-op and Z1 (the
## coordinating session's zoom/pan) is entirely unaffected -- `_lod_active`
## starts `false` and every early-return path above leaves it that way.
##
## Threshold: "more than roughly one screen pixel per grid cell" (the
## milestone's own wording) is exactly `native_px_per_cell * _zoom > 1.0`,
## where `native_px_per_cell` is the *unzoomed* fit scale `_update_lod()`
## computes the same way `map_overlay.gd`'s own `_displayed_rect()` does
## (`min(control_px / grid_cells)` per axis) -- `_zoom` alone is not
## sufficient, since a small preset (512) already exceeds one px/cell at
## `_zoom == 1.0` in most windows, while a large one (8192) needs real zoom
## to reach it. `LOD_PX_PER_CELL_THRESHOLD` names that "roughly one" plainly
## rather than burying `1.0` in the formula below.
const LOD_PX_PER_CELL_THRESHOLD := 1.0
## ...and a second gate the reference has and this file was missing: the
## camera must actually be zoomed IN. `LOD_AUTO_SCALE = 2.2` (reference
## 13952), tested against `viewT.scale` in the wheel handler (reference
## 13986), is a pure camera-zoom threshold, independent of grid resolution.
## Without it the px-per-cell test alone turns deep zoom on at the *fit* view
## for any grid narrower than the viewport -- a 512-cell world in a 900 px map
## rect is already at 1.7 px/cell before the user touches the wheel -- so the
## LOD layer was live on every freshly generated world, which is half of why
## the owner's "a zoom action exposes the underlying heightmap" was visible
## with no zoom action at all. Both conditions must hold, as in the reference:
## px-per-cell says the detail is *resolvable*, this says it was *asked for*.
const LOD_AUTO_ZOOM := 2.2
## `detail_level` doubles the synthesized tile's own resolution once per
## extra octave past the threshold (`lod_bridge::tile_px_for_level`,
## 256/512/1024px) -- 2 matches that function's own `MAX_DETAIL_LEVEL`
## clamp, so this file's idea of "how many tiers exist" cannot drift from
## the Rust side's.
const LOD_MAX_DETAIL_LEVEL := 2
## A real bound, not a cache: right at the threshold crossing is where the
## *most* tiles are visible at once (zooming in further shows fewer, larger
## ones), and synthesizing an unbounded burst in one call the instant a fast
## wheel-scroll crosses the line would stall a frame for no good reason.
## This bounds only the *synthesis* calls a single `_update_lod()` invocation
## may issue -- an already-built tile that is still wanted is never freed or
## re-synthesized just because it falls outside the closest-`N`-to-centre
## set, and whatever misses this call's budget is queued in `_lod_backlog`
## rather than dropped: `_process()` below drains a few more of it every
## frame until either the backlog empties or the next `_update_lod()` call
## replaces it with a freshly-computed one.
##
## Before `_lod_backlog` existed, a viewport wanting more than this many
## *new* tiles at once (a large window, or a fast zoom crossing the
## threshold) left the excess permanently missing: `_apply_lod_tiles`'s
## reconciliation only ever acted on the current `wanted` vs. `_lod_tiles`,
## and a camera that stops moving never calls `_update_lod()` again to give
## the dropped tiles a second chance -- confirmed with a real headless
## repro (grid 512, a 1920x1080 viewport, 64 tiles wanted at the default
## zoom) before this comment was updated: `built` stuck at exactly 48 across
## five redundant `_update_lod()` calls with nothing about the camera
## changing between them. Background synthesis is exactly what
## `_lod_backlog`/`_process()` now are; the one piece still out of this
## milestone's scope is a persistent bake across sessions (the Z5 atlas
## cache, `LOD_TILING_INTEGRATION_SCOPE.md` M3).
const MAX_LOD_TILES_PER_UPDATE := 48
## How many backlog tiles `_process()` synthesizes per frame once
## `_update_lod()` couldn't fit everything into one call's
## `MAX_LOD_TILES_PER_UPDATE` budget. Small on purpose -- this runs every
## frame while a backlog exists (not once per input event like the cap
## above), so it only needs to be large enough to drain a few hundred queued
## tiles over a couple of seconds without itself becoming a per-frame stall.
const MAX_LOD_TILES_PER_CATCHUP := 6

var _lod_layer: Control   ## Child of `_camera`, drawn directly above
	## `map_view` and below every overlay (territory, provinces, the Layers
	## debug raster, previews, vectors) -- it replaces the base raster at deep
	## zoom rather than sitting on top of the stack. See its `_ready()`
	## construction site for why that distinction is load-bearing.
var _lod_tile_cells := 0   ## `EngineBridge.lod_tile_cells()`, fetched once
	## (`0` before any world, or against a binary built before this
	## milestone -- both make `_update_lod()` a no-op, degrading cleanly to
	## Z1-only). Genuinely fetched only once, not once per world: `lod_
	## bridge::TILE_CELLS` is a fixed Rust constant, never tied to which
	## world is loaded (`WorldGen.lod_tile_cells()`'s own doc comment says so
	## directly), so caching past the first successful fetch never goes
	## stale.
var _lod_tiles: Dictionary = {}   ## `"%d,%d" % [tx, ty]` -> the live
	## `TextureRect` showing that tile, so a pan/zoom that doesn't touch a
	## given tile's index leaves its node (and the Rust call that built it)
	## alone -- see `_update_lod()`'s own doc comment on why this is what
	## keeps calling it once per mouse-motion sample affordable.
var _lod_tile_detail: Dictionary = {}   ## Same keys as `_lod_tiles` -> the
	## `detail_level` each was actually synthesized at. `_apply_lod_tiles`
	## compares this against the detail level the *current* call wants and
	## rebuilds a tile whose zoom has moved it into a different tier,
	## instead of leaving a now-too-coarse (or needlessly-fine) texture
	## stretched over its rect forever -- a key match on tile index alone
	## would silently keep showing the old resolution once the index itself
	## stops changing, the same "stops getting revisited once nothing about
	## the viewport changes" shape as the dropped-tile bug this file's
	## `MAX_LOD_TILES_PER_UPDATE` comment documents.
var _lod_backlog: Dictionary = {}   ## `"%d,%d" % [tx, ty]` -> `Vector2i` --
	## tiles the most recent `_update_lod()` wanted but couldn't fit inside
	## `MAX_LOD_TILES_PER_UPDATE`'s per-call synthesis budget. Replaced
	## wholesale on every `_update_lod()` call (never merged), so a camera
	## move that stops wanting a backlogged tile drops it for free the next
	## time `_update_lod()` runs, rather than `_process()` wastefully
	## building it anyway. See `MAX_LOD_TILES_PER_UPDATE`'s own doc comment
	## for why this queue exists at all.
var _lod_backlog_detail_level := 0   ## The geometry `_lod_backlog`'s entries
var _lod_backlog_grid := Vector2i.ZERO   ## were computed against --
var _lod_backlog_origin := Vector2.ZERO   ## `_process()` reuses it rather
var _lod_backlog_size := Vector2.ZERO   ## than re-deriving it from the
	## camera, since it is exactly what the `_update_lod()` call that filled
	## the backlog already had on hand, and is guaranteed current: any camera
	## motion that would invalidate it also calls `_update_lod()`, which
	## replaces the whole backlog (and this geometry) before `_process()`
	## next runs.
var _lod_active := false

func setup(bridge: EngineBridge) -> void:
	_bridge = bridge
	bridge.generation_finished.connect(func(ok: bool): if ok: refresh())
	bridge.world_loaded.connect(refresh)

func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_PASS

	## `_camera`'s own logical `size` always equals the viewport's real size
	## (full-rect, same as before) -- `scale`/`position` handle zoom/pan on top
	## of that, which is why `overlay`'s existing `_displayed_rect()` fit-scale
	## math (written entirely in its own local space) needed no changes at all
	## to keep working under a scaled parent.
	_camera = Control.new()
	_camera.set_anchors_preset(Control.PRESET_FULL_RECT)
	_camera.mouse_filter = Control.MOUSE_FILTER_PASS
	add_child(_camera)

	map_view = _raster()
	_camera.add_child(map_view)

	## Deep-zoom tile overlay (`LOD_TILING_INTEGRATION_SCOPE.md` milestone
	## M1): directly over `map_view` and under *every* overlay, because a
	## refined tile stands in for the blocky base pixels -- it is the base
	## map at deep zoom, not something drawn on top of it. Starts fully
	## transparent and empty; `_update_lod()` is the only thing that ever
	## adds children to it.
	##
	## Since 2026-08-23 a tile *literally* is the base map: its shader
	## samples `map_view`'s own texture for colour and uses the tile texture
	## only as a relief-detail shade ratio (`_build_lod_tile()`,
	## `lod_bridge.rs`). Before that it carried
	## `render_height_tile_rgba`'s hypsometric ramp, which is the reference's
	## *Relief* view mode, and covering the biome-coloured plate with it is
	## exactly the owner's "a zoom action exposes the underlying heightmap".
	##
	## **This node's position in the stack is load-bearing, and once got it
	## wrong.** It used to be added after `territory_view`, `province_view`
	## and `_debug_layer`, on the reading that "above the base raster" meant
	## "above everything raster." It does not: those three are overlays that
	## happen to be rasters. `_update_lod()` turns this layer on whenever the
	## fit scale exceeds one screen pixel per cell -- which a 384x256 grid in
	## a 900px-wide viewport already does at `_zoom == 1.0`, no zooming
	## needed -- and at `modulate.a == 1.0` its opaque tiles then covered the
	## faction fill, the province boundaries and every one of the Layers
	## popover's field views completely. The popover still highlighted the
	## picked row, `debug_view()` still echoed it back, and the map did not
	## change: the owner's "a host of options such as layers dont work"
	## (2026-08-20), reproduced live and fixed by these four lines moving up.
	_lod_layer = Control.new()
	_lod_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	_lod_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_lod_layer.modulate.a = 0.0
	_camera.add_child(_lod_layer)

	territory_view = _raster()
	province_view = _raster()
	territory_view.visible = false
	province_view.visible = false
	_camera.add_child(territory_view)
	_camera.add_child(province_view)

	## The Layers popover's field raster (`sample_bridge.rs`'s debug views),
	## directly over the three base rasters and under everything vector --
	## the reference's own debug overlay sits in exactly that slot, blended
	## over the base map by its `dbgOpacity` slider rather than replacing it.
	## Empty and fully opaque by default; nothing shows until a view is
	## picked, and `modulate.a` is what the popover's opacity slider drives.
	_debug_layer = _raster()
	_camera.add_child(_debug_layer)

	## One shared draft-preview layer for every tool whose result is a full
	## raster (`build_sculpt_preview_texture`, `build_paint_preview_texture`)
	## rather than vector geometry `tool_overlay` already covers -- so Sculpt/
	## Paint (and anything landing later) never need their own `TextureRect`
	## wired into this file. Above `_lod_layer` and below `overlay`, matching
	## the same "raster first, vectors on top" order the base layers already
	## use. Empty texture = invisible; nothing shows until a caller sets one.
	_preview_layer = _raster()
	_camera.add_child(_preview_layer)

	overlay = Control.new()
	overlay.set_script(OVERLAY_SCRIPT)
	overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	_camera.add_child(overlay)
	overlay.settlement_selected.connect(func(d, i): settlement_selected.emit(d, i))
	overlay.settlement_hovered.connect(_on_hovered)
	overlay.cursor_sampled.connect(_on_sampled)
	overlay.map_clicked.connect(func(gx, gy): map_clicked.emit(gx, gy))
	overlay.map_dragged.connect(func(gx, gy): map_dragged.emit(gx, gy))
	overlay.map_released.connect(func(gx, gy, valid): map_released.emit(gx, gy, valid))
	overlay.map_right_clicked.connect(func(gx, gy, hit, pos): map_right_clicked.emit(gx, gy, hit, pos))
	## The town-layout layer pulls its own data, one deferred batch at a time,
	## because generating a town is real engine work and only the overlay knows
	## which towns are on screen and large enough to be worth drawing. This is
	## the whole of the reference's `_umModelFor` scheduling queue that this
	## port keeps -- `URBAN_MORPHOLOGY_SCOPE.md` puts its LRU and its
	## `setTimeout(...,0)` pump explicitly out of scope (a workaround for the
	## browser's single thread), and the overlay's own index-keyed dictionary,
	## dropped whole on every `set_civ_data`, is the only invalidation this
	## shell needs.
	overlay.urban_layouts_needed.connect(_on_urban_layouts_needed)

	## §4.5.1's tool feedback (Region marquee, Measure ruler) -- above
	## `overlay` in draw order so it's never hidden behind terrain/vector
	## data, `MOUSE_FILTER_IGNORE` always so it never competes with
	## `overlay`'s own hit-testing for the click that feeds it.
	tool_overlay = ToolOverlay.new()
	tool_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	tool_overlay.overlay = overlay
	_camera.add_child(tool_overlay)

	## §9's chrome, all corner-anchored so it survives any dock width.
	_scale_label = _chrome(Control.PRESET_BOTTOM_LEFT, HORIZONTAL_ALIGNMENT_LEFT)
	_readout_label = _chrome(Control.PRESET_TOP_RIGHT, HORIZONTAL_ALIGNMENT_RIGHT)
	_coords_label = _chrome(Control.PRESET_BOTTOM_RIGHT, HORIZONTAL_ALIGNMENT_RIGHT)

	_layers_btn = Button.new()
	_layers_btn.flat = true
	_layers_btn.focus_mode = Control.FOCUS_NONE
	_layers_btn.icon = DccIcons.get_icon("layers", 15)
	_layers_btn.tooltip_text = "Layers"
	_layers_btn.modulate = DccTheme.c("text_dim")
	_layers_btn.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
	var hit := 44 if _touch else 26
	_layers_btn.custom_minimum_size = Vector2(hit, hit)
	_layers_btn.add_theme_stylebox_override("normal",
		DccTheme.flat(DccTheme.c("panel"), 3))
	_layers_btn.pressed.connect(func(): layers_button_pressed.emit())
	add_child(_layers_btn)

	_build_navpad()

	## A resize changes the native fit rect `_update_lod()` positions every
	## tile against (`displayed_origin`/`displayed_size`, computed from
	## `size`) -- without this, resizing the window while zoomed in past the
	## threshold would leave existing tiles positioned against the old
	## window size until the next zoom/pan event.
	resized.connect(_update_lod)
	## ...and the navpad is the one piece of chrome positioned absolutely
	## against the *right/bottom* edge rather than anchored there, so unlike
	## the three labels it does not track a resize on its own.
	resized.connect(_apply_safe_insets)

	## `_process()` (deep-zoom backlog catch-up, see `_lod_backlog`'s own doc
	## comment) has nothing to do until `_update_lod()` first populates a
	## backlog -- disabled here rather than paying a per-frame call for a
	## dictionary that starts, and usually stays, empty.
	set_process(false)

	_apply_safe_insets()

## `_input`, not `_gui_input` or `_unhandled_input` -- verified empirically,
## not assumed: `overlay` sits in front with `MOUSE_FILTER_STOP` for its own
## hover/click handling, and Godot's real dispatch order is `_input` (every
## node, tree order) -> GUI dispatch (`_gui_input`, front-to-back, stops at
## the first `MOUSE_FILTER_STOP` control) -> `_unhandled_input`. A first
## attempt at `_unhandled_input` logged the wheel event but never the MMB
## press or its motion -- GUI dispatch had already consumed them, since
## `MOUSE_FILTER_STOP` swallows every mouse event over a control's rect for
## dispatch purposes, not only the ones its own script chooses to act on.
## `_input` runs before that dispatch, so it sees everything unconditionally;
## `set_input_as_handled()` is then called only for the specific events this
## node actually consumes (wheel, MMB, magnify, and a motion event only while
## actively panning), so plain hover motion still reaches `overlay`'s own
## `_gui_input` exactly as before -- no change needed to `map_overlay.gd`.
##
## Pan is a held modifier (MMB, or Space+LMB), per §4.5.1 exactly: *"Pan / zoom
## | Space (held), MMB | Hand | Always available as a modifier, even with
## another tool armed."* Deliberately not bare LMB-drag -- that would collide
## with every future tool that drags with the primary button (Sculpt, Biome
## paint, Territory), which is precisely why the spec calls it a modifier and
## not a default drag.
func _input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_zoom_at(mb.position, ZOOM_WHEEL_STEP)
			get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_zoom_at(mb.position, 1.0 / ZOOM_WHEEL_STEP)
			get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_MIDDLE:
			_panning = mb.pressed
			_pan_last_screen = mb.position
			if mb.pressed:
				get_viewport().set_input_as_handled()
		elif mb.button_index == MOUSE_BUTTON_LEFT and _pan_mode:
			## The navpad's ✋ is on, so the primary button *is* the pan drag --
			## exactly the reference's `if(e.button===1||panMode||spaceDown)`
			## (9623). Reuses `_panning`, so the MouseMotion branch below needs
			## no change at all; and handling the press here, before GUI
			## dispatch, is what keeps the armed tool from also seeing it
			## (the reference's own `!panMode` tool guard, 13924).
			##
			## On the phone this is a single finger: Godot's
			## `emulate_mouse_from_touch` (default, and left alone by
			## `project.godot`'s pinch note) delivers a one-finger drag as
			## LEFT press + motion with the LEFT mask set. Two fingers keep
			## going to the magnify/pan gesture pair below, unaffected.
			##
			## One guard the reference does not need: its buttons live outside
			## the canvas element, while this column sits *over* the map. This
			## handler runs before GUI dispatch, so without it a tap on ✋ or ⟳
			## would start a pan drag as well as press the button.
			## `get_global_rect()` rather than `get_rect()` -- the former
			## carries every parent offset, so it is right whether or not this
			## control happens to start at the viewport origin.
			if mb.pressed and _navpad != null \
					and _navpad.get_global_rect().has_point(mb.position):
				return
			_panning = mb.pressed
			_pan_last_screen = mb.position
			get_viewport().set_input_as_handled()
	elif event is InputEventMouseMotion:
		var mm := event as InputEventMouseMotion
		var space_held := Input.is_key_pressed(KEY_SPACE)
		if _panning or (space_held and (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0):
			_camera.position += mm.position - _pan_last_screen
			_pan_last_screen = mm.position
			get_viewport().set_input_as_handled()
			## Every motion sample, not just on release: `_update_lod()`'s
			## own doc comment explains why this is cheap even at that
			## frequency (only a newly-revealed tile index triggers a real
			## Rust call) -- and updating live means a fast pan reveals new
			## deep-zoom tiles as it crosses into them, not only once the
			## drag ends.
			_update_lod()
		else:
			_pan_last_screen = mm.position
	elif event is InputEventMagnifyGesture:
		## Two-finger pinch, synthesized by the platform on touch. Single-
		## finger drag-to-pan is deliberately not wired on touch: it would
		## collide with an armed tool's own drag the same way bare LMB-drag
		## would on desktop, and the reference makes exactly the same call --
		## its own touch handler comments *"one finger keeps painting/drawing"*
		## (HTML line 13988) and gives the single finger to the tool, never to
		## the camera. Two fingers have no such conflict, so both halves of the
		## gesture pair are safe, and the reference drives both from that one
		## handler: zoom about the centroid **and** pan by the centroid's own
		## delta, in the same `touchmove` (HTML lines 14014-14015).
		var mg := event as InputEventMagnifyGesture
		_zoom_at(mg.position, mg.factor)
		get_viewport().set_input_as_handled()
	elif event is InputEventPanGesture:
		## The pan half of that pair, and until it existed a phone could not
		## move the camera *at all*: pan above is MMB or Space+LMB, and a
		## handheld has neither -- verified on the real device, where a
		## single-finger `input swipe` across the map moved only the hover
		## cursor and left every map pixel identical.
		##
		## Emitted by the same Android gesture detector `SH-10` turned on
		## (`input_devices/pointing/android/enable_pan_and_scale_gestures`) --
		## the setting name is not incidental, it gates *pan and scale*
		## together, and the shipped APK's own `GodotGestureHandler` carries
		## `handlePanEvent`/`setPanEvent` beside the `onScale` pair SH-10
		## already confirmed. So this arrives for free on a build that already
		## pinches; nothing new had to be enabled.
		##
		## Sign: Android's `GestureDetector` defines `distanceX/Y` as *old
		## focus minus new*, so the delta points opposite the fingers --
		## subtracting it makes the map travel *with* them, which is what
		## every map application does and what the reference's own
		## `viewT.panX += cx - pinch.cx` does with its own (already-inverted)
		## convention. Measured on device, not reasoned into place: the same
		## injected two-finger drag moves the map the way the fingers went.
		##
		## `_zoom` is deliberately untouched -- a pan is not a zoom, and the
		## magnify branch above already owns that half of the gesture, so a
		## real pinch (which fires both) still zooms and pans exactly once
		## each, like the reference's single `touchmove` does.
		var pg := event as InputEventPanGesture
		_camera.position -= pg.delta
		_update_lod()
		get_viewport().set_input_as_handled()

## Zooms so the world point under `screen_pt` stays under it: with `pan` the
## camera's position and `zoom` its scale, a local point maps to screen space
## as `screen = pan + local * zoom`. Solving for the new pan that keeps the
## same local point fixed under `screen_pt` after rescaling gives this directly
## -- no pivot_offset needed, which sidesteps Control's own layout-vs-transform
## interactions around pivot entirely.
func _zoom_at(screen_pt: Vector2, factor: float) -> void:
	var new_zoom: float = clampf(_zoom * factor, ZOOM_MIN, ZOOM_MAX)
	if is_equal_approx(new_zoom, _zoom):
		return
	var local_pt := (screen_pt - _camera.position) / _zoom
	_camera.position = screen_pt - local_pt * new_zoom
	_zoom = new_zoom
	_camera.scale = Vector2(_zoom, _zoom)
	_update_zoom_readout()
	## `overlay`'s settlement pins are re-scaled to compensate for
	## `_camera.scale` above (`map_overlay.gd`'s `_civ_zoom_k()`, see
	## `PIN_SCALE_REF_PX`'s own doc comment for why) -- that compensation is
	## baked into `_draw()`'s cached draw commands, so it needs telling every
	## time zoom actually changes, not just once.
	overlay.set_camera_zoom(_zoom)
	_update_lod()

## The navpad's + / − (`GUI_GAP_REGISTER.md` SH-14). The reference's own two
## buttons are `zoomAt(viewCenter(), 1.35)` and its inverse (13464-13465) --
## the *centre* of the viewport, not the last-touched point, because a button
## press carries no map position of its own. That is `_zoom_at` above with one
## argument filled in; no second zoom path, and every consequence `_zoom_at`
## already handles (the readout, `overlay`'s pin compensation, the deep-zoom
## tile pass) follows from calling it.
const ZOOM_BUTTON_STEP := 1.35   ## Reference 13464. Deliberately coarser than
	## `ZOOM_WHEEL_STEP` (1.15): a wheel notch is cheap and repeatable, a
	## 44 px tap is neither.

func zoom_step(factor: float) -> void:
	_zoom_at(size * 0.5, factor)

## The navpad's ✋. See `_pan_mode`'s own doc comment for why this latches
## rather than holding. Dropping `_panning` on the way out matters: turning
## the mode off mid-drag would otherwise leave a pan running with no button
## down to end it, since the release would no longer take this file's branch.
func set_pan_mode(on: bool) -> void:
	if _pan_mode == on:
		return
	_pan_mode = on
	if not on:
		_panning = false
	if _pan_btn != null:
		_pan_btn.set_pressed_no_signal(on)
		## Accent fill, dark glyph -- the phone canvas's own on-toggle idiom
		## (`#e0a34a` track, `#141617` knob), not a border or a tint, so the
		## one latched control in the column is unmistakable at arm's length.
		_navpad_paint(_pan_btn,
			DccTheme.c("accent") if on else DccTheme.c("panel"),
			DccTheme.c("bg") if on else DccTheme.c("text"))

func pan_mode() -> bool:
	return _pan_mode

## `_civMoveViewTo(x, y)` -- the reference's context-menu "📍 Move viewer to"
## op and the Faction Roster's "(focus camera)" link. Centres grid cell
## `(gx, gy)` in the viewport at the current zoom; no zoom change, matching
## the reference, which pans only.
##
## The camera's own contract (`_zoom_at` above) is `screen = position +
## local * zoom`, where `local` is `overlay`'s unscaled control space --
## `overlay` is a FULL_RECT child of `_camera`, so `displayed_rect()` is
## already in exactly that space. Solving that for "put this local point at
## the viewport centre" is the one line below; no other pan/zoom state is
## touched, so a following wheel-zoom still pivots correctly.
func move_view_to(gx: float, gy: float) -> void:
	var rect: Rect2 = overlay.displayed_rect()
	if rect.size.x <= 0.0:
		return
	var g := _bridge.grid_size()
	if g.x <= 0 or g.y <= 0:
		return
	var local := rect.position + Vector2((gx + 0.5) / float(g.x), (gy + 0.5) / float(g.y)) * rect.size
	_camera.position = size * 0.5 - local * _zoom
	_update_lod()

## `_viewFill()` (reference 13294) -- the default/reset view, which in this app
## means **cover**, not fit: the map fills the display and whichever axis has
## slack loses it off the edges. Called from `refresh()` so a new world never
## opens scrolled off into whatever corner the previous one was zoomed into,
## and from the navpad's ⟳, which is what the reference's own `zoomReset`
## calls (13466) -- deliberately *not* `resetView()` (13390, `scale=1, pan=0`),
## which has not been what that button does since the reference's v1.13.
##
## This used to be plain fit (`_zoom = 1`, `position = ZERO`), which is
## `_camera` at identity over a `STRETCH_KEEP_ASPECT_CENTERED` raster --
## visibly the letterboxed state the reference's own v1.01 was raised to fix
## ("eliminate unused letterbox space"), and on a portrait phone against a
## square world those dead bands are most of the screen. Owner decision,
## 2026-08-23: reset matches the reference and covers.
##
## The scale: `_viewCoverScale()` is `max(1, availW/natW, availH/natH)` over
## the canvas's *natural* size. This camera's `_zoom == 1` is already the
## letterbox-fit rect rather than a natural pixel size, so the same quantity
## here is the fit rect's own shortfall against the viewport -- and it is
## `>= 1` by construction, which is the reference's `max(1, ...)` floor for
## free. `overlay.displayed_rect()` is that rect, reused rather than
## recomputing the fit math a third time.
##
## The pan: the reference sets `panX/panY = 0` and lets `_viewClampFill()`
## (13295) settle it. Worked through, that lands the map exactly aligned on
## the tight axis and **asymmetrically cropped on the loose one** -- an
## artifact of `transform-origin: 0 0` over a flex-centred `.canvas-wrap`,
## not an intent; the reference's own comment at 13290 says "cover scale,
## centred". Centred is what this does, so the crop is even on both edges.
## Deviation recorded rather than taken silently (`CLAUDE.md`), and it is the
## only one -- the scale is the reference's, exactly.
##
## The reference's standing pan clamp is deliberately **not** ported. It runs
## on every `applyView()`, not just reset, so it is a change to all four pan
## paths (MMB, Space+LMB, pan gesture, navpad pan mode) rather than to this
## function -- and it would fight `ZOOM_MIN = 0.4`, which lets this camera
## zoom below fit where the reference floors at fit. Reset restores a known
## view, which is the whole point of the button; recorded as open in
## `GUI_GAP_REGISTER.md` rather than bundled in here.
func reset_view() -> void:
	var cover := 1.0
	var origin := Vector2.ZERO
	var rect: Rect2 = overlay.displayed_rect()
	if rect.size.x > 0.0 and rect.size.y > 0.0 and size.x > 0.0 and size.y > 0.0:
		cover = clampf(maxf(size.x / rect.size.x, size.y / rect.size.y), 1.0, ZOOM_MAX)
		origin = size * 0.5 - rect.get_center() * cover
	_zoom = cover
	_camera.scale = Vector2(_zoom, _zoom)
	_camera.position = origin
	## `zoomReset` clears `panMode` as well (13466) -- a reset that left the
	## hand latched would put the view back but not the input mode, which is
	## half a reset.
	set_pan_mode(false)
	_update_zoom_readout()
	overlay.set_camera_zoom(_zoom)   ## See `_zoom_at()`'s own comment on why this call exists.
	## Every existing deep-zoom tile belongs to whatever world/size was live
	## before this reset -- `refresh()` calls this on every new
	## `generate()`/`load_save()`, so a stale tile from a *different* world
	## must not survive to be shown (or, worse, silently reused because its
	## `(tx, ty)` key happens to overlap) against the new one.
	## `_update_lod()` below repopulates from scratch if the new view still
	## qualifies for deep zoom (a small preset can, even at `_zoom == 1.0` --
	## see this file's own `LOD_PX_PER_CELL_THRESHOLD` doc comment).
	_clear_lod_tiles()
	_update_lod()

func _update_zoom_readout() -> void:
	if _bridge == null or not _bridge.has_world:
		return
	var g := _bridge.grid_size()
	_readout_label.text = "%d x %d  ·  %.0f x %.0f km  ·  z%.1f" % [
		g.x, g.y, _bridge.last_width_km, _bridge.last_height_km, _zoom]

## Phone chrome (`DccShell._build_phone_shell()`) sits on top of this node's
## own edges once the map is edge-to-edge behind it (inset rule "DRAW
## EDGE-TO-EDGE, PAD BY INSET") -- without this, the layers button and the
## coordinate/scale-bar labels would land under the app bar, the rail or the
## tool sheet instead of in the visible gap between them. Desktop/tablet never
## call this, so `_safe_insets` stays at its flat 10 px default there.
func set_safe_insets(insets: Dictionary) -> void:
	_safe_insets = insets
	_apply_safe_insets()

## Sets offsets directly rather than `.position` -- found by screenshot that
## `.position` doesn't hold up here: it's computed against the control's
## *current* size, and for `_readout_label`/`_scale_label` that's still (0, 0)
## the first time this runs (before `refresh()` has ever set their text).
## `grow_horizontal`/`grow_vertical` then auto-expand the rect the next time
## the text changes, growing from whatever edge was implied by that stale
## zero-size baseline -- which, worked through by hand, is exactly the wrong
## edge for a right/bottom-anchored label, and landed the readout ~139 px off
## the left edge of a 393 px screen. Computing both edges from
## `get_minimum_size()` every call sidesteps the growth direction machinery
## entirely instead of trying to keep it fed a correct baseline.
func _apply_safe_insets() -> void:
	if _scale_label == null:
		return  ## Not built yet -- `_ready()` applies the default once itself.
	var l := float(_safe_insets.get("left", 10.0))
	var t := float(_safe_insets.get("top", 10.0))
	var r := float(_safe_insets.get("right", 10.0))
	var b := float(_safe_insets.get("bottom", 10.0))

	var scale_size := _scale_label.get_minimum_size()
	_scale_label.offset_left = l
	_scale_label.offset_right = l + scale_size.x
	_scale_label.offset_top = -b - scale_size.y
	_scale_label.offset_bottom = -b

	var readout_size := _readout_label.get_minimum_size()
	_readout_label.offset_right = -r
	_readout_label.offset_left = -r - readout_size.x
	_readout_label.offset_top = t
	_readout_label.offset_bottom = t + readout_size.y

	var coords_size := _coords_label.get_minimum_size()
	_coords_label.offset_right = -r
	_coords_label.offset_left = -r - coords_size.x
	_coords_label.offset_top = -b - coords_size.y
	_coords_label.offset_bottom = -b

	_layers_btn.position = Vector2(l, t)

	## The navpad rides the same insets, so it clears the app bar, the bottom
	## bar, the timeline and the gesture strip without a second set of
	## numbers -- and stacks *above* `_coords_label`, which owns this corner.
	if _navpad != null:
		var pad := _navpad.get_combined_minimum_size()
		_navpad.position = Vector2(size.x - maxf(r, float(NAVPAD_EDGE)) - pad.x,
			size.y - b - coords_size.y - float(NAVPAD_GAP) - pad.y)

func _raster() -> TextureRect:
	var t := TextureRect.new()
	t.set_anchors_preset(Control.PRESET_FULL_RECT)
	t.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	## Keep-aspect-centred, so a non-square map is letterboxed rather than
	## stretched -- the whole point of the sized API.
	t.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	t.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	t.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return t

## The touch navpad -- this port's answer to the reference's mobile-only
## `#zoomOverlay` (HTML 749-754), and `GUI_GAP_REGISTER.md` SH-14's whole
## content. Designed rather than transliterated (owner decision, 2026-08-23):
## the reference draws four bare floating web buttons, a mobile-web idiom this
## shell uses nowhere else, so this is the same four functions in the phone
## canvas's own language -- one right-edge column of 44 dp pills, exactly the
## floating cluster `design/Cartalith Android Phone.dc.html`'s artboard
## "01 · VIEWPORT" already puts at `right:14px` with a 10 px gap. Nothing here
## is a new mechanism; it is `_layers_btn` four more times.
##
## **Reachability: every touch device, not phones only.** `_touch` is the gate
## the reference's own `isMobile` gate means -- what that gate is really
## testing is "there is no wheel, no middle button and no space bar", which is
## as true of a tablet as of a phone. `DccShell._phone` would be the wrong
## gate: it is an *aspect-ratio* test (`_PHONE_ASPECT_MAX`) that exists to
## pick a layout, and a tablet fails it and takes the desktop shell -- desktop
## chrome with no mouse, which is precisely the case that needs this most.
## Desktop is excluded because it already has all four (wheel, MMB/Space,
## and now this file's `reset_view()` from the same call).
const NAVPAD_HIT := 44     ## §13's floor. Raw, not `_phone_scale`d, for the
	## same reason `_layers_btn` above is: the shipped phone's viewport is
	## ~393 px (see `_apply_safe_insets()`'s own note), where that scale is
	## 1.0 -- and `_safe_insets`, which positions this, arrives already
	## scaled from `DccShell.phone_content_insets()`.
const NAVPAD_GAP := 10     ## The canvas's own column gap, and above the 8 px
	## adjacent-target minimum.
const NAVPAD_EDGE := 14    ## The canvas's own `right:14px`, used as a *floor*
	## on the safe inset rather than instead of it. Portrait phone reports
	## `right: 0.0` (`DccShell.phone_content_insets()`) because no chrome
	## occupies that edge -- correct for a text readout, wrong for a round
	## 44 px target, which would sit against the bezel with half its area in
	## the palm-rejection zone.

func _build_navpad() -> void:
	if not _touch:
		return
	_navpad = VBoxContainer.new()
	_navpad.add_theme_constant_override("separation", NAVPAD_GAP)
	_navpad.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
	_navpad.mouse_filter = Control.MOUSE_FILTER_IGNORE   ## The buttons pick;
		## the column itself must not, or it would eat the map between them.
	add_child(_navpad)

	_navpad.add_child(_navpad_button("zoom_in", "Zoom in",
		func(): zoom_step(ZOOM_BUTTON_STEP)))
	_navpad.add_child(_navpad_button("zoom_out", "Zoom out",
		func(): zoom_step(1.0 / ZOOM_BUTTON_STEP)))

	_pan_btn = _navpad_button("tool_pan", "Pan mode", Callable())
	_pan_btn.toggle_mode = true
	## `set_pan_mode()` writes `button_pressed` back (⟳ clears the mode), which
	## would re-enter this handler -- it uses `set_pressed_no_signal()` for
	## exactly that, and this stays the only caller that comes from a finger.
	_pan_btn.toggled.connect(set_pan_mode)
	_navpad.add_child(_pan_btn)

	_navpad.add_child(_navpad_button("view_fill", "Reset view", reset_view))

func _navpad_button(glyph: String, tip: String, on_press: Callable) -> Button:
	var b := Button.new()
	## Deliberately **not** `flat`, unlike `_layers_btn` and every other button
	## in this shell: `Button.flat` suppresses the background stylebox
	## entirely, so a flat button with a `normal` override draws the override
	## nowhere. Caught by screenshot -- the first cut was flat, and the pills
	## were invisible over the terrain with only their glyphs showing.
	b.focus_mode = Control.FOCUS_NONE
	b.icon = DccIcons.get_icon(glyph, 17)
	## Icon-only, so the tooltip is the only accessible name it has.
	b.tooltip_text = tip
	b.custom_minimum_size = Vector2(NAVPAD_HIT, NAVPAD_HIT)
	_navpad_paint(b, DccTheme.c("panel"), DccTheme.c("text"))
	if on_press.is_valid():
		b.pressed.connect(on_press)
	return b

## One pill's fill and glyph colour, across all three states. Tinting the
## glyph through `icon_*_color` rather than `modulate` is what lets the fill
## and the glyph carry different colours -- `modulate` multiplies the whole
## control, so an accent pill would drag its own glyph to accent with it.
func _navpad_paint(b: Button, fill: Color, ink: Color) -> void:
	var pill := StyleBoxFlat.new()
	pill.bg_color = fill
	pill.set_corner_radius_all(NAVPAD_HIT / 2)
	## A hairline, because unlike every other button in this shell these float
	## over the *map*: `panel` against dark terrain reads as a shape, against
	## bright desert or ice it does not. The canvas's own floating chips carry
	## the same `rgba(255,255,255,.12)` edge for the same reason.
	pill.set_border_width_all(1)
	pill.border_color = DccTheme.c("line")
	b.add_theme_stylebox_override("normal", pill)
	## A visible press, which a 44 px target with no hover state on a
	## touchscreen otherwise has no feedback at all for. Lightened toward the
	## ink rather than swapped for a token, so it works for both fills.
	var down := pill.duplicate() as StyleBoxFlat
	down.bg_color = fill.lerp(ink, 0.22)
	b.add_theme_stylebox_override("hover", down)
	b.add_theme_stylebox_override("pressed", down)
	for state in ["normal", "hover", "pressed"]:
		b.add_theme_color_override("icon_%s_color" % state, ink)

func _chrome(preset: int, align: int) -> Label:
	var l := DccTheme.label("", "text_faint", DccTheme.FS_SMALL)
	l.horizontal_alignment = align
	## PRESET_MODE_MINSIZE bakes offsets from the control's size at call time --
	## zero, during `_ready` -- and nothing recomputes them on resize, which put
	## the right-hand readouts off the edge of the viewport. Anchor without
	## baking, and grow away from the anchored edge.
	l.set_anchors_preset(preset, true)
	var right: bool = align == HORIZONTAL_ALIGNMENT_RIGHT
	var bottom: bool = preset in [Control.PRESET_BOTTOM_LEFT, Control.PRESET_BOTTOM_RIGHT]
	l.grow_horizontal = Control.GROW_DIRECTION_BEGIN if right else Control.GROW_DIRECTION_END
	l.grow_vertical = Control.GROW_DIRECTION_BEGIN if bottom else Control.GROW_DIRECTION_END
	## Positioned properly by `_apply_safe_insets()`, called once at the end of
	## `_ready()` (and again by `set_safe_insets()` on phone rotation) -- left
	## at the anchor's own baseline here rather than duplicating the inset math.
	l.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(l)
	return l

# -- Refresh ------------------------------------------------------------------

func refresh() -> void:
	if _bridge == null or not _bridge.has_world:
		return
	map_view.texture = _bridge.color_texture()
	territory_view.texture = _bridge.territory_texture()
	province_view.texture = _bridge.province_boundary_texture()
	## A picked view survives a regenerate: same view id, new world's data.
	## `set_debug_layer` clears itself if the new world cannot answer it.
	set_debug_layer(_debug_view)
	var g := _bridge.grid_size()
	overlay.set_civ_data(_bridge.settlements(), _bridge.roads(),
		_bridge.sea_routes(), g.x, g.y, _bridge.border_inset_frac())
	## A town is sized in real metres, so the layout layer needs the map's own
	## km extent to know how many pixels 1.7 km is worth.
	overlay.set_map_width_km(_bridge.last_width_km)
	tool_overlay.set_grid(g.x, g.y)
	_width_km = _bridge.last_width_km
	_update_scale_bar()
	reset_view()   ## Also sets `_readout_label`'s text -- see its own doc.
	## Belt-and-suspenders after the two lines above change `_scale_label`'s
	## and `_readout_label`'s text (and so their minimum size): the fixed
	## edge each already has should carry Godot's own grow-direction resize
	## correctly on its own (see `_apply_safe_insets()`'s comment), but a full
	## recompute against the now-current size costs nothing and doesn't rely
	## on trusting that reasoning held.
	_apply_safe_insets()

## §4.5.5's Icon/Label tools (`UNIFIED_TOOL_PLAN.md` milestone F) -- lighter
## than `refresh()`: placing/editing one icon or label shouldn't re-fetch the
## terrain texture or civ data, only push the updated lists into `overlay`'s
## own annotation layers. See `map_overlay.gd`'s own `_manual_icons`/`_labels`
## doc comment, which already names this method.
func refresh_annotations() -> void:
	if _bridge == null or not _bridge.has_world:
		return
	overlay.set_manual_icons(_bridge.icon_list())
	overlay.set_labels(_bridge.label_list())

## The current pan/zoom camera scale (`_zoom`, `_zoom_at`'s own factor) --
## read-only exposure for `label_handles(index, zoom)` callers (`DCC_SHELL_
## SPEC.md` §4.5.5's Label tool), which need the real camera zoom so a
## selected label's on-canvas handles size and stem-offset consistently with
## how large the label itself is currently drawn.
func zoom() -> float:
	return _zoom

func set_layer_visible(layer: String, shown: bool) -> void:
	match layer:
		"territory": territory_view.visible = shown
		"provinces": province_view.visible = shown
		"settlements": overlay.set_show_settlements(shown)
		"roads": overlay.set_show_roads(shown)
		"sea_routes": overlay.set_show_sea_routes(shown)
		## `civUrbanLayoutsChk` (`GUI_GAP_REGISTER.md` UM-01). Reveals only
		## once a town's 1.7 km site box is worth pixels -- `map_overlay.gd`'s
		## own "Urban layouts" block owns that gate and states why it is not
		## the reference's `_umLayoutAlpha` km band.
		"urban_layouts": overlay.set_show_urban_layouts(shown)
		_: push_error("ViewportHost: unknown layer '%s'" % layer)

## Answers `map_overlay.gd`'s one-batch-at-a-time request for town layouts.
## Synchronous, on the main thread: generating a town is a few milliseconds,
## the batch is capped by the overlay, and this port has no worker path from
## GDScript into a `#[func]` -- said plainly rather than hidden behind a
## thread that does not exist.
func _on_urban_layouts_needed(indices: PackedInt32Array) -> void:
	if _bridge == null or not _bridge.has_world:
		overlay.set_urban_layouts(indices, [])
		return
	overlay.set_urban_layouts(indices, _bridge.urban_layouts(indices))

## The per-class / per-way-type half of the reference's own layer filters
## (`#explSettlementFilterList`, and `#explShowRoads`'s by-way-type list --
## `design/Cartalith Menu Structure v2.dc.html`, MAP ▸ LAYERS). Passed
## straight through to `map_overlay.gd`, which owns the draw-time test; kept
## as separate entry points from `set_layer_visible` above because these take
## a *sub*-key, not a layer id, and folding them into one string namespace
## would make "settlements" and "settlements/hamlet" collide.
func set_settlement_kind_visible(kind: String, shown: bool) -> void:
	overlay.set_settlement_kind_visible(kind, shown)

func set_way_type_visible(way_type: String, shown: bool) -> void:
	overlay.set_way_type_visible(way_type, shown)

## Sculpt/Paint's shared draft-preview raster (`_preview_layer`, built in
## `_ready()`). Pass `null` to clear it -- an armed tool's own disarm handler
## is expected to do this, the same way `GlobalTools` clears `tool_overlay`'s
## geometry on disarm, so a stale draft never lingers after switching tools.
func set_preview_texture(tex: Texture2D) -> void:
	_preview_layer.texture = tex

## The Layers popover's field raster. `"off"` (or a view this world has no
## input for) clears it -- the popover reads `debug_view()` back to keep its
## own selection honest, so a view that could not be drawn does not stay
## highlighted as though it had been.
func set_debug_layer(view: String) -> void:
	if _bridge == null or view == "off" or not _bridge.has_world:
		_debug_view = "off"
		_debug_layer.texture = null
		return
	var tex := _bridge.debug_texture(view)
	_debug_layer.texture = tex
	_debug_view = view if tex != null else "off"

## Which view `set_debug_layer` actually managed to draw.
func debug_view() -> String:
	return _debug_view

## The reference's own `#dbgOpacity` (0-100%): blends the active field
## raster over the base map so terrain reads through it.
func set_debug_opacity(a: float) -> void:
	_debug_layer.modulate.a = clampf(a, 0.0, 1.0)

func debug_opacity() -> float:
	return _debug_layer.modulate.a

## Screen-space rect of the layers button, so a popover can anchor itself to
## it rather than guessing at the viewport's corner inset (which changes with
## `set_safe_insets()` on phone).
func layers_button_rect() -> Rect2i:
	return Rect2i(Vector2i(_layers_btn.global_position), Vector2i(_layers_btn.size))

func _update_scale_bar() -> void:
	if _width_km <= 0.0:
		_scale_label.text = ""
		return
	var gw := _bridge.grid_size().x
	if gw <= 0:
		_scale_label.text = "%.0f km across" % _width_km
		return
	## Cells are square in km, so one quotient describes both axes.
	var per_cell := _width_km / float(gw)
	var cell_text := "%.2f" % per_cell if per_cell < 10.0 else "%.0f" % per_cell
	_scale_label.text = "%.0f km across  ·  %s km / cell" % [_width_km, cell_text]

func _on_hovered(data: Variant, index: int) -> void:
	settlement_hovered.emit(data, index)

func _on_sampled(gx: float, gy: float, valid: bool) -> void:
	_coords_label.text = _coords_text(gx, gy) if valid else ""
	cursor_sampled.emit(gx, gy, valid)

## §10's bottom-right readout (`4 812 km E · 1 093 km N · 1 462 m`): a real
## world-space position from the same km-per-cell conversion
## `_update_scale_bar()` already trusts ("cells are square in km, so one
## quotient describes both axes"), plus the *committed* elevation at that
## cell (`EngineBridge.sample_cell()`'s `elevation_m`). Grid row 0 is the
## map's north edge (screen-up, same convention `map_overlay.gd` draws
## against), so "km N" counts up from the *south* edge -- `(gh - gy)`, not
## `gy` -- the way a real northing reads on a map, increasing northward.
## `""` before any world, matching this label's own previous behaviour.
##
## **The design's own `→ 1 582 m` draft-stamp suffix
## (`DCC_SHELL_SPEC.md` §10) is deliberately not built here.**
## `sample_cell()` reads only `WorldState::field` -- `sample_bridge.rs`'s own
## `sample_refs()` wires `field: &ws.field` and nothing from `self.sculpt`'s
## draft `PassBuffer` -- so it can never see an uncommitted stroke.
## `build_sculpt_preview_texture()` does composite the draft, but only into a
## full-grid *colourised* RGB texture (`RenderCtx::with_appearance`'s
## hillshade/AO/appearance pipeline runs first); there is no `#[func]` that
## returns the draft's raw elevation at one cell, and reverse-engineering an
## elevation out of already-shaded pixel colour is not a real reading. This
## needs one new Rust entry point (something like `sample_bridge::
## sample_cell` but reading `scratch` -- the sculpt draft's own
## `preview_into` output -- instead of `ws.field`); `GUI_GAP_REGISTER.md`
## classified SH-06 (A) on the premise that this call already existed. It
## doesn't -- corrected there and in `CHANGELOG.md`.
func _coords_text(gx: float, gy: float) -> String:
	if _bridge == null or not _bridge.has_world:
		return ""
	var g := _bridge.grid_size()
	if g.x <= 0 or g.y <= 0 or _width_km <= 0.0:
		return ""
	var per_cell := _width_km / float(g.x)
	var east_km := gx * per_cell
	var north_km := float(g.y - gy) * per_cell
	var text := "%s km E  ·  %s km N" % [_fmt_thousands(east_km, 0), _fmt_thousands(north_km, 0)]
	var cell := _bridge.sample_cell(int(round(gx)), int(round(gy)))
	if cell.has("elevation_m"):
		text += "  ·  %s m" % _fmt_thousands(float(cell["elevation_m"]), 0)
	return text

## Space-grouped thousands ("4 812"), matching the design mockup's own
## formatting. The same small helper as `journey_planner_view.gd`'s own
## `_fmt_thousands` -- duplicated rather than shared, matching this
## project's existing pattern of a private per-file formatter rather than a
## new cross-file utility for one call site each.
func _fmt_thousands(v: float, decimals: int) -> String:
	var s := ("%.*f" % [decimals, v])
	var neg := s.begins_with("-")
	if neg:
		s = s.substr(1)
	var dot := s.find(".")
	var int_part := s if dot < 0 else s.substr(0, dot)
	var frac_part := "" if dot < 0 else s.substr(dot)
	var out := ""
	var count := 0
	for i in range(int_part.length() - 1, -1, -1):
		out = int_part[i] + out
		count += 1
		if count % 3 == 0 and i > 0:
			out = " " + out
	return ("-" if neg else "") + out + frac_part

# -- Deep-zoom tile compositing (`LOD_TILING_INTEGRATION_SCOPE.md` M1) --------

## Recomputes which deep-zoom tiles belong on screen and asks
## `EngineBridge.lod_synthesize_tile` for any not already displayed. Called
## from every place the camera or the control's own size can change what's
## visible: `_zoom_at` (wheel/pinch), the pan branch of `_input`'s
## `MouseMotion` handling, `reset_view` (new/loaded world), and the
## `resized` signal (`_ready`).
##
## Deliberately **not** debounced to "once per frame" or "only on pan-end":
## the real cost here is the Rust synthesis call, and that only happens for
## a tile index this method has not already built a `TextureRect` for
## (`_lod_tiles`, keyed by tile index) -- a pan or zoom that doesn't cross a
## tile boundary (`_lod_tile_cells` grid cells, which at any zoom past the
## threshold is already several hundred screen px) touches no new tiles and
## makes no Rust call at all, so running this once per input event costs
## only the screen<->grid geometry below, not a synthesis pass.
##
## Which tiles are "visible" is resolved with plain arithmetic here, in
## GDScript, rather than via `cartalith_spatial::QuadTree::query_region` --
## see `lod_bridge.rs`'s own module doc for why: building a quadtree over
## the live height field just to answer a pure index-range question would
## cost a real O(field size) scan for a query whose real cost, done
## directly, is O(tiles on screen). This file already owns the
## camera<->grid transform (`_zoom_at`, `map_overlay.gd`'s
## `_displayed_rect()`/`_cell_to_screen`) that the same arithmetic needs, so
## nothing new is invented here, only reused.
func _update_lod() -> void:
	if _bridge == null or not _bridge.has_world:
		_set_lod_active(false)
		return
	if _lod_tile_cells <= 0:
		_lod_tile_cells = _bridge.lod_tile_cells()
	if _lod_tile_cells <= 0:
		return   ## Binary built before this milestone -- stay Z1-only.

	var g := _bridge.grid_size()
	if g.x <= 0 or g.y <= 0 or size.x <= 0.0 or size.y <= 0.0:
		_set_lod_active(false)
		return

	## The native (`_zoom == 1.0`) fit rect -- identical math to
	## `map_overlay.gd`'s own `_displayed_rect()`, computed against `size`
	## (this control's own, which `_camera`'s always equals -- see its own
	## doc comment above) since that is the space every tile is positioned
	## in: `_camera`'s `scale`/`position` transform is what pan/zoom
	## actually is, so a tile positioned once in native space tracks the
	## camera for free, the same way `map_view` itself already does.
	var native_scale := minf(size.x / float(g.x), size.y / float(g.y))
	if native_scale <= 0.0:
		_set_lod_active(false)
		return
	var screen_px_per_cell := native_scale * _zoom
	if screen_px_per_cell <= LOD_PX_PER_CELL_THRESHOLD or _zoom <= LOD_AUTO_ZOOM:
		_set_lod_active(false)
		return

	var displayed_size := Vector2(g.x, g.y) * native_scale
	var displayed_origin := (size - displayed_size) * 0.5

	## Screen rect -> `_camera`-local rect, inverting `_zoom_at`'s own
	## `screen = position + local * zoom` relation.
	var local_top_left := (Vector2.ZERO - _camera.position) / _zoom
	var local_bottom_right := (size - _camera.position) / _zoom

	## Local rect -> grid-cell rect: the inverse of `map_overlay.gd`'s
	## `_cell_to_screen` mapping, minus its marker-placement `+0.5`
	## centering (irrelevant to the cell/pixel correspondence itself).
	## Clamped to the real grid -- a non-square map's own letterbox bars,
	## and panning past the map's edge, both map outside `[0, gw] x [0, gh]`
	## otherwise.
	var gx0 := clampf((local_top_left.x - displayed_origin.x) / displayed_size.x * g.x, 0.0, float(g.x))
	var gy0 := clampf((local_top_left.y - displayed_origin.y) / displayed_size.y * g.y, 0.0, float(g.y))
	var gx1 := clampf((local_bottom_right.x - displayed_origin.x) / displayed_size.x * g.x, 0.0, float(g.x))
	var gy1 := clampf((local_bottom_right.y - displayed_origin.y) / displayed_size.y * g.y, 0.0, float(g.y))
	if gx1 <= gx0 or gy1 <= gy0:
		_set_lod_active(false)
		return

	var tx0 := int(floor(gx0 / _lod_tile_cells))
	var ty0 := int(floor(gy0 / _lod_tile_cells))
	var tx1 := int(floor(maxf(gx0, gx1 - 0.001) / _lod_tile_cells))
	var ty1 := int(floor(maxf(gy0, gy1 - 0.001) / _lod_tile_cells))

	## One detail tier per doubling past the threshold, capped at
	## `LOD_MAX_DETAIL_LEVEL` to match `lod_bridge::MAX_DETAIL_LEVEL` --
	## `log(x)/log(2.0)` is GDScript's `log2`, since `log()` here is natural
	## log.
	var detail_level := clampi(
		int(floor(log(screen_px_per_cell / LOD_PX_PER_CELL_THRESHOLD) / log(2.0))),
		0, LOD_MAX_DETAIL_LEVEL)

	var wanted: Dictionary = {}   ## key -> Vector2i tile index
	for ty in range(ty0, ty1 + 1):
		for tx in range(tx0, tx1 + 1):
			wanted["%d,%d" % [tx, ty]] = Vector2i(tx, ty)

	## Only the tiles this call would actually have to *synthesize* --
	## already-built ones at the right detail level cost nothing but a
	## reposition below, so they stay in `wanted` (and so never get freed)
	## regardless of whether they'd have made the closest-`N` cut; only the
	## genuinely-missing set below competes for the per-call budget.
	## `_lod_tile_detail`'s own doc comment covers the "already built, wrong
	## tier" half of "missing".
	var missing: Dictionary = {}
	for key in wanted.keys():
		if not _lod_tiles.has(key) or _lod_tile_detail.get(key, -1) != detail_level:
			missing[key] = wanted[key]

	var build_keys: Dictionary = missing
	var trimmed := false
	if missing.size() > MAX_LOD_TILES_PER_UPDATE:
		build_keys = _nearest_tiles(missing, Vector2((tx0 + tx1) * 0.5, (ty0 + ty1) * 0.5))
		trimmed = true

	## Whatever didn't make this call's budget -- replaces the previous
	## backlog wholesale rather than merging into it, so a tile that scrolled
	## back out of `wanted` since the last call is dropped here for free
	## instead of `_process()` wastefully building it later. See
	## `_lod_backlog`'s own doc comment.
	_lod_backlog.clear()
	if trimmed:
		for key in missing.keys():
			if not build_keys.has(key):
				_lod_backlog[key] = missing[key]
	_lod_backlog_detail_level = detail_level
	_lod_backlog_grid = g
	_lod_backlog_origin = displayed_origin
	_lod_backlog_size = displayed_size
	set_process(not _lod_backlog.is_empty())

	_apply_lod_tiles(wanted, build_keys, detail_level, g, displayed_origin, displayed_size)
	_set_lod_active(true)

## Trims a candidate set (key -> `Vector2i` tile index -- `_update_lod()`'s
## own `missing`) to `MAX_LOD_TILES_PER_UPDATE` entries closest to `centre`
## (in tile-index space) -- see `MAX_LOD_TILES_PER_UPDATE`'s own doc comment
## for why this bound exists at all rather than synthesizing every candidate
## unconditionally.
func _nearest_tiles(wanted: Dictionary, centre: Vector2) -> Dictionary:
	var keys: Array = wanted.keys()
	keys.sort_custom(func(a, b) -> bool:
		var ai: Vector2i = wanted[a]
		var bi: Vector2i = wanted[b]
		return Vector2(ai).distance_squared_to(centre) < Vector2(bi).distance_squared_to(centre))
	var trimmed: Dictionary = {}
	for i in range(MAX_LOD_TILES_PER_UPDATE):
		trimmed[keys[i]] = wanted[keys[i]]
	return trimmed

## Reconciles `_lod_tiles` against `wanted` (key -> `Vector2i` tile index):
## frees whatever is no longer wanted, repositions whatever already exists
## at the right detail level (cheap, and correct after a resize -- see
## `_update_lod`'s own native-space reasoning), rebuilds whatever exists at
## the *wrong* detail level (`_lod_tile_detail`'s own doc comment), and only
## calls `EngineBridge.lod_synthesize_tile` for a key in `build_keys` --
## `_update_lod()`'s per-call budget subset of the keys actually missing.
## Anything missing but NOT in `build_keys` is left alone here: it is
## already queued in `_lod_backlog`, and `_process()` builds it shortly.
func _apply_lod_tiles(wanted: Dictionary, build_keys: Dictionary, detail_level: int, g: Vector2i, displayed_origin: Vector2, displayed_size: Vector2) -> void:
	for key in _lod_tiles.keys().duplicate():
		if not wanted.has(key):
			(_lod_tiles[key] as TextureRect).queue_free()
			_lod_tiles.erase(key)
			_lod_tile_detail.erase(key)

	for key in wanted.keys():
		var idx: Vector2i = wanted[key]

		if _lod_tiles.has(key):
			if _lod_tile_detail.get(key, -1) == detail_level:
				var rect = _lod_tile_rect(idx, g, displayed_origin, displayed_size)
				if rect != null:
					var existing := _lod_tiles[key] as TextureRect
					existing.position = rect.position
					existing.size = rect.size
					## Cheap, and the one thing about an already-built tile
					## that can go stale without its index or detail level
					## moving: the shader samples `map_view`'s texture for
					## colour, and a re-render (quality tier, appearance,
					## a Sculpt commit) replaces that texture in place.
					var mat := existing.material as ShaderMaterial
					if mat != null:
						mat.set_shader_parameter("base_tex", map_view.texture)
				continue
			## Detail level moved on since this tile was built (the camera
			## zoomed further within the same tile index) -- free the stale
			## texture and fall through to rebuild it at the tier the
			## current zoom actually wants, subject to the same
			## `build_keys` budget a brand-new tile is.
			(_lod_tiles[key] as TextureRect).queue_free()
			_lod_tiles.erase(key)
			_lod_tile_detail.erase(key)

		if build_keys.has(key):
			_build_lod_tile(key, idx, detail_level, g, displayed_origin, displayed_size)
		## Else: over this call's synthesis budget -- already queued in
		## `_lod_backlog` by `_update_lod()`, built by `_process()` shortly.

## The screen rect one tile occupies, clipped exactly like `lod_bridge::
## tile_bounds` on the Rust side -- both compute `min(TILE_CELLS, remaining)`
## from the same `gw`/`gh`, so this can never disagree with what a tile
## request actually returns (this file's own module-doc note on why that's
## the one piece of tile-bounds arithmetic duplicated here at all). `null`
## for a degenerate (fully-clipped) tile.
func _lod_tile_rect(idx: Vector2i, g: Vector2i, displayed_origin: Vector2, displayed_size: Vector2) -> Variant:
	var origin_x := idx.x * _lod_tile_cells
	var origin_y := idx.y * _lod_tile_cells
	var tile_w := mini(_lod_tile_cells, g.x - origin_x)
	var tile_h := mini(_lod_tile_cells, g.y - origin_y)
	if tile_w <= 0 or tile_h <= 0:
		return null
	return Rect2(
		displayed_origin + Vector2(origin_x, origin_y) / Vector2(g.x, g.y) * displayed_size,
		Vector2(tile_w, tile_h) / Vector2(g.x, g.y) * displayed_size)

## Synthesizes one tile (`idx`, `detail_level`) and stores it under `key` in
## `_lod_tiles`/`_lod_tile_detail`. Shared by `_apply_lod_tiles`'s own build
## path and `_process()`'s backlog catch-up -- exactly the same work, called
## from two different budgets (`MAX_LOD_TILES_PER_UPDATE` per input event,
## or `MAX_LOD_TILES_PER_CATCHUP` per idle frame).
func _build_lod_tile(key: String, idx: Vector2i, detail_level: int, g: Vector2i, displayed_origin: Vector2, displayed_size: Vector2) -> void:
	var rect = _lod_tile_rect(idx, g, displayed_origin, displayed_size)
	if rect == null:
		return
	var tex := _bridge.lod_synthesize_tile(idx.x, idx.y, detail_level)
	if tex == null:
		return
	var tile_node := TextureRect.new()
	tile_node.texture = tex
	tile_node.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	tile_node.stretch_mode = TextureRect.STRETCH_SCALE
	## `LINEAR`, not `_raster()`'s `NEAREST` -- the entire reason this
	## milestone exists is to stop showing blocky single-cell squares at
	## deep zoom, so the tile that replaces them must not reintroduce
	## the same artifact at its own, finer texel size.
	tile_node.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	tile_node.mouse_filter = Control.MOUSE_FILTER_IGNORE
	tile_node.position = rect.position
	tile_node.size = rect.size
	## `tex` is a relief-detail shade ratio, not a picture (`lod_bridge.rs`,
	## "What a tile actually contains"). The shader multiplies it into the
	## base map's own colour, sampled from `map_view.texture` over this
	## tile's footprint -- so a deep-zoom tile can no longer disagree with
	## the map it sits on, which is what the pre-2026-08-23 hypsometric
	## tile did at every pixel.
	var origin_x := idx.x * _lod_tile_cells
	var origin_y := idx.y * _lod_tile_cells
	var tile_w := mini(_lod_tile_cells, g.x - origin_x)
	var tile_h := mini(_lod_tile_cells, g.y - origin_y)
	var mat := ShaderMaterial.new()
	mat.shader = LOD_TILE_SHADER
	mat.set_shader_parameter("base_tex", map_view.texture)
	mat.set_shader_parameter("base_uv0",
		Vector2(float(origin_x) / g.x, float(origin_y) / g.y))
	mat.set_shader_parameter("base_uv1",
		Vector2(float(origin_x + tile_w) / g.x, float(origin_y + tile_h) / g.y))
	tile_node.material = mat
	_lod_layer.add_child(tile_node)
	_lod_tiles[key] = tile_node
	_lod_tile_detail[key] = detail_level

## Backlog catch-up (`_lod_backlog`'s own doc comment): drains up to
## `MAX_LOD_TILES_PER_CATCHUP` entries per frame, reusing the geometry the
## `_update_lod()` call that filled the backlog already computed -- safe to
## reuse because any camera motion that would make it stale also triggers a
## fresh `_update_lod()` call first, which replaces the whole backlog (and
## this geometry) before `_process()` next runs. Disables its own per-frame
## processing the moment the backlog empties, so this costs nothing once
## deep zoom has fully caught up.
func _process(_delta: float) -> void:
	if _lod_backlog.is_empty():
		set_process(false)
		return
	var n := 0
	for key in _lod_backlog.keys().duplicate():
		if n >= MAX_LOD_TILES_PER_CATCHUP:
			break
		var idx: Vector2i = _lod_backlog[key]
		_lod_backlog.erase(key)
		if not _lod_tiles.has(key):   ## Not already built by a call in between.
			_build_lod_tile(key, idx, _lod_backlog_detail_level,
				_lod_backlog_grid, _lod_backlog_origin, _lod_backlog_size)
		n += 1
	if _lod_backlog.is_empty():
		set_process(false)

## Frees every currently-displayed deep-zoom tile and clears the backlog.
## Called on every world reset (`reset_view`, since a stale tile belongs to
## whatever world/size was live before) and whenever `_set_lod_active(false)`
## actually changes state (crossing back below the threshold).
func _clear_lod_tiles() -> void:
	for key in _lod_tiles.keys():
		(_lod_tiles[key] as TextureRect).queue_free()
	_lod_tiles.clear()
	_lod_tile_detail.clear()
	_lod_backlog.clear()
	set_process(false)

## Fades `_lod_layer` in or out and frees its tiles on the way out. A no-op
## when `active` already matches `_lod_active` -- called from several
## early-return paths in `_update_lod()` that would otherwise re-trigger the
## tween on every single call while already below the threshold.
func _set_lod_active(active: bool) -> void:
	if active == _lod_active:
		return
	_lod_active = active
	if not active:
		_clear_lod_tiles()
	var tw := create_tween()
	tw.tween_property(_lod_layer, "modulate:a", 1.0 if active else 0.0, 0.15)
