extends Control
## No `class_name`, deliberately: `viewport_host.gd` preloads this file as
## `View3DScene`, so it works in a tree whose `.godot` class cache predates 3D-1.

## **The 3D view mode** (`3D_VIEWPORT_SCOPE.md` §5, milestones 3D-1 and 3D-2).
##
## What it is: the viewport mode `Window ▸ 3D view` and MAP's "2D / 3D" button
## switch to (§3.14; the navpad's own 3D pill left the row in MS-5). A `SubViewport` holding a `Camera3D` over an empty
## world, plus a status card that says which 3D tier this device runs and why,
## with the numbers the capability gate (`view3d_gate.gd`) measured (§4.3).
## 3D-1 built it before any terrain, so "3D unavailable, here is why" worked
## first (research §D, Milestone 0). **3D-2 draws the Basic tier's terrain**
## (`view3d_terrain.gd`: one displaced patch over tiles at the camera's level,
## draped with the 2D picture, unlit) under an orbit or top-down camera
## (`view3d_camera.gd`), wherever the tier is Basic. **3D-3 draws Standard and
## Full as a quadtree** (`view3d_quadtree.gd`: the pyramid's tiles from the LOD
## worker, one patch instance per node, the `L_3d` morph, skirts, frustum
## culling, prefetch and a node-level debug view), at the tier's patch size.
##
## Input (§3.9): on a mouse, left or middle drag pans, right drag orbits and
## tilts, the wheel zooms about the pivot. On touch, one finger pans, two
## fingers pinch to zoom, twist to turn and move up or down together to tilt
## (the 2D map's one-finger pan, Ruling EE). The map's − | + and ⌖ drive this
## camera through `ViewportHost.zoom_step()` and `reset_view()` (Ruling EB
## item 4).
##
## Why it covers the map rather than replacing it: in 3D-1 nothing 3D reads
## the 2D map, so the 2D view is left exactly as it was underneath, and leaving
## 3D shows it again. Since 3D-3 the 2D view's deep-zoom tiles are parked
## while 3D shows (§3.12, `ViewportHost.set_3d()`), so 3D's layers replace
## them rather than adding to them, and leaving 3D rebuilds the same picture.
##
## What it must never do:
## - **Change anything 2D draws.** It is created only on first entry
##   (`ViewportHost.set_3d()`), so a session that never opens 3D never builds
##   this node; and while it is shown it takes the pointer (`MOUSE_FILTER_STOP`)
##   so no 2D tool receives a click meant for the 3D view.
## - **Run the gate during a generate or without being visible.** The gate draws
##   into this node's own `SubViewport`, which must be on screen to be timed.
## - **Raise a tier on its own.** A tier above the measured one is only ever the
##   user's choice through `Force tier…`, behind a warning (§4.4).

## The gate, preloaded rather than reached by its `class_name` (see
## `menus.gd`'s `View3DGate` constant for why).
const View3DGate := preload("res://shell/view3d_gate.gd")
## 3D-2's camera state and terrain, preloaded for the same reason.
const View3DCamera := preload("res://shell/view3d_camera.gd")
const View3DTerrain := preload("res://shell/view3d_terrain.gd")
## 3D-3's quadtree terrain for Standard and Full, preloaded for the same reason.
const View3DQuadtree := preload("res://shell/view3d_quadtree.gd")

## The user asked to go back to the 2D map (the card's own button).
signal exit_requested()
## The tier, the forced tier or the disclosure changed (a test finished, a
## force was applied or dropped). Menus and the navpad re-read their text here.
signal state_changed()

## **Ruling CT (owner, 2026-10-19):** the 3D view's vertical exaggeration
## defaults to true scale, 1.0x, not 2D's 3.4x relief default, which draws
## needles (`3D_VIEWPORT_SCOPE.md` §8.3, captures 07-08). 3D-2's mesh opens at
## `DccSettings.view3d_exaggeration()`, which is this value until the user sets
## one (Ruling EB item 5); the live slider that sets it is its own backlog row.
const VERTICAL_EXAGGERATION_DEFAULT := 1.0
## Orbit rate for a right-button drag or a two-finger vertical drag, in degrees
## per physical pixel. A labelled judgement: a drag across a 1 600-px desktop
## view turns the camera 400 degrees, a phone-width drag about 270.
const ORBIT_DEG_PER_PX := 0.25
## The HEIGHT row's range and step: Ruling EB item 6 (*"Up to 4×"*), from Ruling
## CT's true-scale floor (below 1x would flatten real relief, so it is not
## offered), in the 0.25 steps of the slider's design (§7 Q4).
const HEIGHT_MIN := 1.0
const HEIGHT_MAX := 4.0
const HEIGHT_STEP := 0.25

## Card geometry in dp (multiplied by `_unit` on touch). Labelled judgements
## read off the shell's own floating chips: a 12 dp inset (`NAVPAD_EDGE`'s
## figure) and a card no wider than 460 dp so a sentence stays readable.
const CARD_INSET := 12
const CARD_MAX_W := 460
const CARD_PAD := 12
const CARD_GAP := 8
## The smallest SubViewport side the gate will time (labelled judgement): any
## real map area is hundreds of pixels; below this the layout has not run and
## a timing would not describe what the user sees.
const MIN_PROBE_PX := 64

var _box: SubViewportContainer
var _sv: SubViewport
var _cam: Camera3D
var _gate: View3DGate
var _card: PanelContainer
var _title: Label
var _status: Label
var _info: Label
var _back_btn: Button
var _retest_btn: Button
var _force_btn: Button
## 3D-2: switches between Orbit and Top-down (§3.9's two modes of this milestone).
var _mode_btn: Button
## The HEIGHT row (`_build_height_row()`); all null on the phone.
var _height_box: VBoxContainer
var _height_slider: HSlider
var _height_value: Label
var _force_menu: PopupMenu
## The open force warning, if any (kept so the acceptance probe can answer it).
var _warn: ConfirmationDialog
## Device-pixel multiplier for every dp figure here: the SAME scale
## `ViewportHost._apply_touch_scale()` gave the navpad (`set_frame()`), so the
## card and the pills agree on every composition. A first cut read
## `DccTheme.phone_scale()` on any touch device and drew 110 px buttons on a
## tablet composition whose navpad runs at 1.0 (`_view3d_probe.gd`, 1080 x 2340
## `--force-touch`, which composes as tablet).
var _unit := 1.0
## The host's safe insets (`ViewportHost._safe_insets`): on the phone the map
## runs edge to edge behind the app bar and the bottom sheet, and a card placed
## at a bare 12 px sat under the app bar in the first phone capture.
var _insets := {"left": 0.0, "top": 0.0, "right": 0.0, "bottom": 0.0}
## True while the gate is rendering; every button is disabled meanwhile.
var _busy := false
## The last forced-tier drop message (§4.4), shown before the disclosure until
## the next test or force replaces it. Not persisted: it describes this
## session's failure, not the device.
var _note := ""

## 3D-2: the camera state and the terrain node (the node is null until `_ready()`).
var _vcam: View3DCamera = View3DCamera.new()
var _terrain: View3DTerrain
## 3D-3: the Standard and Full tiers' quadtree (null until such a tier draws).
var _quad: View3DQuadtree = null
## The "Tile levels" toggle on the card (the node-level debug view), shown on
## Standard and Full only.
var _levels_btn: Button
## Whether `_quad` has been bound to the engine for its tier.
var _quad_bound := false
## The engine, bound by `ViewportHost.ensure_view3d()`: the `EngineBridge`, a
## `Callable() -> bool` false while a generate holds `WorldGen`, and a
## `Callable() -> Texture2D` answering the 2D base map texture.
var _bridge: Object = null
var _readable := Callable()
var _base_tex := Callable()
## The 2D view to open on (`ViewportHost.view_for_3d()`): `{centre: Vector2
## (cells), px_per_cell: float}`, or `{}` when 2D had no world on screen.
var _entry := {}
## True when the camera must be placed on the next frame that has a world: at
## `_entry` (tilting) after an entry, or framing the world after a new one.
var _place_pending := false
var _frame_world_pending := false
## Set by `world_changed()`: on the next bind, frame the world if its grid
## differs from the one the camera was set for (`_vcam.has_world()`).
var _rebind_check_grid := false
## Touch points by index (physical pixels) and the two-finger gesture's last
## distance, angle and centroid.
var _touches := {}
var _pinch := {}
## The mouse drag in progress: the button held, or 0.
var _drag_button := 0
## The camera's height above the drawn ground at the last frame, and how many
## clamp steps the last frame took: the acceptance probe's B4 readings.
var last_hag := 0.0
var last_clamp_steps := 0
func _ready() -> void:
	## `..._and_offsets_`, not `set_anchors_preset()`: this node is added to an
	## already-laid-out `ViewportHost`, and anchors alone left it at 0 x 0 with
	## its `SubViewport` at 2 x 2 (measured by `_view3d_probe.gd`'s first run,
	## whose gate then timed a 2 x 2 frame).
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP

	_box = SubViewportContainer.new()
	_box.stretch = true
	_box.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_box.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_box)
	resized.connect(_fit_physical)
	get_viewport().size_changed.connect(_fit_physical)
	## Once now as well: this node is laid out before `_ready()` connects, so neither signal fires for the
	## size it already has (measured: without this call a 692 x 1268 px view kept a 346 x 634 SubViewport).
	_fit_physical.call_deferred()
	_sv = SubViewport.new()
	## Its own World3D, so nothing 3D this mode adds can meet any other 3D
	## content a probe or a later milestone puts in the main viewport.
	_sv.own_world_3d = true
	_sv.render_target_update_mode = SubViewport.UPDATE_WHEN_VISIBLE
	_box.add_child(_sv)
	var env := WorldEnvironment.new()
	env.environment = Environment.new()
	env.environment.background_mode = Environment.BG_COLOR
	env.environment.background_color = DccTheme.c("bg")
	_sv.add_child(env)
	## Camera-relative from the start (§3.11): the camera stays at the origin
	## and the world will move under it. Top-down, because entering 3D starts
	## top-down at the 2D view (§3.9, owner question Q9's default).
	_cam = Camera3D.new()
	_cam.rotation = Vector3(-PI * 0.5, 0.0, 0.0)
	_cam.fov = View3DCamera.FOV_DEG
	_cam.keep_aspect = Camera3D.KEEP_HEIGHT
	_sv.add_child(_cam)
	_cam.current = true
	_terrain = View3DTerrain.new()
	_sv.add_child(_terrain)
	_vcam.min_span_m = ViewportHost.ZOOM_TARGET_SPAN_KM * 1000.0
	_terrain.set_exaggeration(DccSettings.view3d_exaggeration(VERTICAL_EXAGGERATION_DEFAULT))

	_gate = View3DGate.new()
	add_child(_gate)
	_build_card()
	_refresh()

## The status card: title, the tier disclosure, the terrain line
## (`_terrain_line()`) and four actions. Built once; `_refresh()` rewrites its text.
func _build_card() -> void:
	_card = PanelContainer.new()
	var sb := DccTheme.panel("panel")
	sb.set_corner_radius_all(int(8 * _unit))
	sb.set_border_width_all(1)
	sb.border_color = DccTheme.c("line")
	sb.content_margin_left = CARD_PAD * _unit
	sb.content_margin_right = CARD_PAD * _unit
	sb.content_margin_top = CARD_PAD * _unit
	sb.content_margin_bottom = CARD_PAD * _unit
	_card.add_theme_stylebox_override("panel", sb)
	_card.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(_card)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", int(CARD_GAP * _unit))
	_card.add_child(col)
	_title = DccTheme.mono_label("3D VIEW", "text_dim", _fs(DccTheme.FS_SMALL), 2)
	col.add_child(_title)
	_status = DccTheme.label("", "text", _fs(DccTheme.FS_BODY))
	_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	col.add_child(_status)
	_info = DccTheme.label("", "text_dim", _fs(DccTheme.FS_SMALL))
	_info.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	col.add_child(_info)
	_build_height_row(col)
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", int(CARD_GAP * _unit))
	row.add_theme_constant_override("v_separation", int(CARD_GAP * _unit))
	col.add_child(row)
	_back_btn = _button(row, "Back to 2D", func(): exit_requested.emit(), true)
	_mode_btn = _button(row, "Top-down", func(): set_top_down(not is_top_down()))
	_levels_btn = _button(row, "Tile levels", func(): set_debug_levels(1 if _quad == null or _quad.debug_levels == 0 else 0))
	_levels_btn.toggle_mode = true
	_retest_btn = _button(row, "Re-test 3D", retest)
	_force_btn = _button(row, "Force tier…", _open_force_menu)
	_force_menu = PopupMenu.new()
	_force_menu.add_radio_check_item("Automatic (as measured)", 100)
	for t in View3DGate.TIER_NAMES.size():
		_force_menu.add_radio_check_item(View3DGate.TIER_NAMES[t], t)
	_force_menu.id_pressed.connect(func(id: int): request_force(-1 if id == 100 else id))
	if DccTheme.is_touch():
		_force_menu.add_theme_font_size_override("font_size", _fs(DccTheme.FS_MENU_ITEM))
	add_child(_force_menu)
	resized.connect(_place_card)
	_place_card()

## **The HEIGHT row** (`3D_VIEWPORT_SCOPE.md` §7 Q4, the slider's design;
## Rulings CT and EB items 5 and 6): between the info line and the actions, on
## the PC and the tablet. 1x to 4x in steps of 0.25, opening at the remembered
## value; dragging rescales the drawn mesh live (a shader uniform, §3.1) and
## writes `Preferences ▸ Graphics ▸ 3D viewport defaults ▸ relief exaggeration`
## as it moves, so the last value is the next session's default. Never the
## world, its save or `WorldState`. **Not on the phone**: its home there is a
## row in MAP ▸ View (the design keeps the phone's card from growing), which is
## not built yet, so the phone opens at the remembered value with no slider.
## Touch: 44 dp tall, as every touch target.
func _build_height_row(col: VBoxContainer) -> void:
	if DccTheme.is_phone():
		_height_box = null
		return
	_height_box = VBoxContainer.new()
	_height_box.add_theme_constant_override("separation", int(4 * _unit))
	col.add_child(_height_box)
	var head := HBoxContainer.new()
	_height_box.add_child(head)
	head.add_child(DccTheme.mono_label("HEIGHT · 1× IS TRUE SCALE", "text_dim", _fs(DccTheme.FS_SMALL), 1))
	head.add_child(DccTheme.spacer())
	_height_value = DccTheme.mono_label("", "text", _fs(DccTheme.FS_SMALL), 0)
	head.add_child(_height_value)
	_height_slider = HSlider.new()
	_height_slider.min_value = HEIGHT_MIN
	_height_slider.max_value = HEIGHT_MAX
	_height_slider.step = HEIGHT_STEP
	_height_slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_height_slider.focus_mode = Control.FOCUS_NONE
	_height_slider.custom_minimum_size.y = maxf(14.0, 44.0 * _unit) if DccTheme.is_touch() else 14.0
	DccWidgets._style_slider(_height_slider)
	var x := _terrain.exaggeration() if _terrain != null else DccSettings.view3d_exaggeration(VERTICAL_EXAGGERATION_DEFAULT)
	_height_slider.set_value_no_signal(x)
	_height_value.text = "×%.2f" % x
	_height_slider.value_changed.connect(set_height)
	_height_box.add_child(_height_slider)
	var note := DccTheme.label("Remembered in Preferences ▸ 3D viewport defaults for the next session · never saved with the world", "text_dim", _fs(DccTheme.FS_SMALL))
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_height_box.add_child(note)

## Sets the drawn vertical scale and remembers it (the HEIGHT row's handler;
## also the acceptance probe's seam). Clamped to the row's range.
func set_height(v: float) -> void:
	var x := clampf(v, HEIGHT_MIN, HEIGHT_MAX)
	if _terrain != null:
		_terrain.set_exaggeration(x)
	if _quad != null:
		_quad.set_exaggeration(x)
	DccSettings.set_view3d_exaggeration(x)
	if _height_value != null:
		_height_value.text = "×%.2f" % x
	if _height_slider != null and not is_equal_approx(_height_slider.value, x):
		_height_slider.set_value_no_signal(x)
	_refresh()

## A dp font size in device pixels.
func _fs(dp: int) -> int:
	return int(round(dp * _unit))

## One card button: `DccWidgets.action()`'s look, floored on BOTH axes to the
## touch target on touch devices at the call site, because this node is built
## after the shell's own fitters ran (`MISTAKES.md`, "Floor a control that a
## rebuild creates").
func _button(parent: Control, text: String, cb: Callable, primary := false) -> Button:
	var b := DccWidgets.action(parent, text, cb, primary)
	if DccTheme.is_touch():
		var hit := maxi(DccTheme.PHONE_TAP_MIN, int(round(44.0 * _unit)))
		b.add_theme_font_size_override("font_size", _fs(DccTheme.FS_BODY))
		b.custom_minimum_size = Vector2(hit, hit)
	return b

## The host's safe insets and touch scale (`ViewportHost.set_safe_insets()`
## and `ensure_view3d()` call this). A scale change after the card exists
## rebuilds the card, since its fonts and floors are baked from `_unit`.
func set_frame(insets: Dictionary, unit: float) -> void:
	_insets = insets.duplicate()
	var u := maxf(1.0, unit)
	if not is_equal_approx(u, _unit):
		_unit = u
		if _card != null:
			_card.queue_free()
			_card = null
			_build_card()
			_refresh()
	_place_card()

## Top-left inside the host's safe insets, as wide as the content allows up
## to `CARD_MAX_W`.
func _place_card() -> void:
	if _card == null:
		return
	var inset := CARD_INSET * _unit
	var l := maxf(float(_insets.get("left", 0.0)), inset)
	var t := maxf(float(_insets.get("top", 0.0)), inset)
	var r := maxf(float(_insets.get("right", 0.0)), inset)
	var w := minf(CARD_MAX_W * _unit, maxf(size.x - l - r, 0.0))
	_card.position = Vector2(l, t)
	_card.custom_minimum_size.x = w
	_card.size = Vector2(w, 0.0)
	_card.reset_size()

# -- state --------------------------------------------------------------------

## The stored result, or `{}` when never tested.
func result() -> Dictionary:
	return DccSettings.view3d_result()

## The forced tier (-1 = automatic).
func forced_tier() -> int:
	return DccSettings.view3d_forced_tier()

## The tier this view runs at now (`View3DGate.effective_tier`).
func tier() -> int:
	return View3DGate.effective_tier(result(), forced_tier())

## Why the stored result must be re-measured, or "" if it stands (§4.2's
## adapter / driver / version triggers).
func stale_reason() -> String:
	return View3DGate.stale_reason(result(), View3DGate.device_identity())

## The §4.3 sentence for the current state, with its measured numbers.
func disclosure() -> String:
	return View3DGate.disclosure(result(), forced_tier(), _note)

## Whether the gate is rendering right now.
func busy() -> bool:
	return _busy

## The gate node, for the acceptance probe's test seams only.
func gate() -> View3DGate:
	return _gate

## **The 3D image at physical resolution under a content scale** (MS-1 S2, spec §2.6 and R4). A
## `SubViewportContainer` with `stretch` sizes its `SubViewport` in canvas units, so under a tablet's dp
## unit the 3D view rendered at dp resolution and was magnified (S0: a 400 x 300 container at a factor of 2
## got a 400 x 300 `SubViewport` for an 800 x 600 physical rect). Above a factor of 1 the container is
## therefore laid out at `size x factor` canvas units and drawn at `1 / factor`, which makes its
## `SubViewport` exactly the physical rect. At a factor of 1 (the PC, the phone) it keeps the full-rect
## anchors it always had, so nothing there changes. The container ignores the mouse, so no input passes
## through the scaled node.
func _fit_physical() -> void:
	if _box == null or not is_inside_tree():
		return
	var f := get_viewport().get_final_transform().x.x
	if f <= 1.0001:
		_box.scale = Vector2.ONE
		_box.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
		return
	_box.set_anchors_preset(Control.PRESET_TOP_LEFT)
	_box.position = Vector2.ZERO
	_box.scale = Vector2.ONE / f
	_box.size = size * f

## The `SubViewport` this mode draws into.
func subviewport() -> SubViewport:
	return _sv

## Rewrites the card from the stored state and tells listeners.
func _refresh() -> void:
	if _status == null:
		return
	_status.text = "Testing 3D on this device… about 3 seconds." if _busy else disclosure()
	if _busy:
		_info.text = "The test draws a few views and times them. The map is untouched."
	elif tier() == View3DGate.TIER_OFF:
		_info.text = "The 2D map is unaffected. You can re-test, or force a tier at your own risk."
	else:
		_info.text = _terrain_line()
	for b in [_back_btn, _retest_btn, _force_btn, _mode_btn]:
		b.disabled = _busy
	_mode_btn.visible = tier() != View3DGate.TIER_OFF
	_levels_btn.visible = tier() >= View3DGate.TIER_STANDARD
	_levels_btn.disabled = _busy
	_levels_btn.set_pressed_no_signal(_quad != null and _quad.debug_levels == 1)
	if _height_box != null:
		_height_box.visible = tier() != View3DGate.TIER_OFF
		_height_slider.editable = not _busy
	_mode_btn.text = "Orbit" if is_top_down() else "Top-down"
	_place_card()
	state_changed.emit()

## Called when the view becomes visible: runs the gate first if this device was
## never tested or its stored result is stale (§4.2: "on first entry to 3D,
## not at boot").
func entered() -> void:
	if not _busy and stale_reason() != "":
		await retest()
	else:
		_refresh()
	## A re-entry starts from the 2D view again, with fresh tiles: the look,
	## the world or a sculpt may have changed while 3D was hidden, and neither
	## Basic nor the quadtree keeps a tile across that (the quadtree keeps its
	## layer arrays allocated, not their contents).
	if _terrain != null:
		_terrain.clear("waiting for the first tile")
	if _quad != null:
		_quad.clear("waiting for the first tile")
	_place_pending = true

# -- 3D-2: terrain and camera ---------------------------------------------------

## Binds the engine (`ViewportHost.ensure_view3d()`); see `_bridge`'s comment.
func bind_engine(bridge: Object, readable: Callable, base_tex: Callable) -> void:
	_bridge = bridge
	_readable = readable
	_base_tex = base_tex

## The 2D view the next entry opens on (`ViewportHost.set_3d()`).
func set_entry_view(v: Dictionary) -> void:
	_entry = v.duplicate()

## The map was repainted (`ViewportHost.refresh()`: a new or reopened world, a
## look change, a colour-paint commit): forget every tile so the next ones are
## the new picture. The camera stays where it is unless the grid itself
## changed (a different world), which frames the new world instead.
func world_changed() -> void:
	if _terrain != null:
		_terrain.clear("waiting for the first tile")
	if _quad != null:
		_quad.clear("waiting for the first tile")
	_rebind_check_grid = true

## Whether terrain is drawn at the current tier (Basic and above).
func draws_terrain() -> bool:
	return tier() != View3DGate.TIER_OFF and not _busy

## The camera state and terrain node, for the acceptance probe.
func camera_state() -> View3DCamera:
	return _vcam

func terrain() -> View3DTerrain:
	return _terrain

## 3D-3's quadtree (null until Standard or Full has drawn), for the probe.
func quadtree() -> View3DQuadtree:
	return _quad

## Whether this tier draws the quadtree (Standard and Full) rather than
## Basic's single patch.
func uses_quadtree() -> bool:
	return tier() >= View3DGate.TIER_STANDARD

## The gate's workload key for the quadtree at this tier (§3.12's rows):
## "full", or Standard's phone or desktop row by `View3DGate.phone_class`.
func quad_key() -> String:
	if tier() >= View3DGate.TIER_FULL:
		return "full"
	return View3DGate.standard_key(View3DGate.phone_class(OS.get_name()))

## The node-level debug view (§5, 3D-3): 0 off, 1 tint each node by its level,
## 2 the per-pixel level index (the B1 probe's).
func set_debug_levels(mode: int) -> void:
	if _quad != null:
		_quad.debug_levels = mode
	_refresh()

## The surface the camera clamps against: the quadtree's or Basic's.
func _surface_bound(x: float, z: float) -> float:
	return _quad.surface_bound(x, z) if uses_quadtree() and _quad != null else _terrain.surface_bound(x, z)

func camera3d() -> Camera3D:
	return _cam

## **− and + in 3D** (Ruling EB item 4): the camera in by `factor`, or out.
func zoom_step(factor: float) -> void:
	_vcam.zoom(factor)

## **⌖ in 3D**: the whole world in frame (Ruling EB item 4).
func frame_world() -> void:
	_vcam.frame_world(float(_sv.size.x) / maxf(float(_sv.size.y), 1.0))

func is_top_down() -> bool:
	return _vcam.ortho

## Top-down (orthographic, straight down, at the current ground scale) or
## Orbit (§3.9).
func set_top_down(on: bool) -> void:
	_vcam.set_ortho(on)
	_refresh()

## The info line under the disclosure: what is drawn, at which level, and why
## when nothing is. Never a level for a window that is not drawn.
func _terrain_line() -> String:
	if uses_quadtree():
		var tn: String = View3DGate.TIER_NAMES[tier()]
		var qx := _quad.exaggeration() if _quad != null else VERTICAL_EXAGGERATION_DEFAULT
		if _quad == null or _quad.drawn_count == 0:
			var qwhy: String = _quad.reason if _quad != null else "not started"
			return "%s terrain: %s. Heights at %.2f× (1× is true scale)." % [tn, qwhy, qx]
		var lv := "level %d" % _quad.level_min if _quad.level_min == _quad.level_max else "levels %d–%d" % [_quad.level_min, _quad.level_max]
		return "%s terrain, %d tiles at %s. Heights at %.2f× (1× is true scale)." % [tn, _quad.drawn_count, lv, qx]
	var t := "Basic"
	var x := _terrain.exaggeration() if _terrain != null else VERTICAL_EXAGGERATION_DEFAULT
	var w: Dictionary = _terrain.window() if _terrain != null else {}
	if w.is_empty():
		var why: String = _terrain.reason if _terrain != null else "not started"
		return "%s terrain: %s. Heights at %.2f× (1× is true scale)." % [t, why, x]
	return "%s terrain, tile level %d. Heights at %.2f× (1× is true scale)." % [t, int(w["z"]), x]

## Physical pixels per canvas unit (the tablet's content scale; 1 elsewhere).
func _phys() -> float:
	return get_viewport().get_final_transform().x.x if is_inside_tree() else 1.0

## The per-frame step while terrain is drawn: bind and size the world once,
## place the camera after an entry or a new world, run the entry tilt, clamp
## to the ground (B4), pick and fill the tile window, then place everything
## camera-relative and set the depth range (§3.11).
func _process(delta: float) -> void:
	if not visible or _terrain == null or _bridge == null or not draws_terrain():
		return
	if not (_readable.is_valid() and bool(_readable.call())):
		return
	if uses_quadtree():
		_process_quad(delta)
		return
	if _quad != null and _quad.drawn_count > 0:
		_quad.clear("this tier draws Basic's patch")
	if not _terrain.world_known():
		var bt: Texture2D = _base_tex.call() if _base_tex.is_valid() else null
		_terrain.bind(_bridge.get("world_gen"), _readable, bt)
		## `update()` reads the world's size off the level-0 tile.
		_terrain.update(0, 0, Vector2.ZERO)
		if not _terrain.world_known():
			_refresh()
			return
		var g := _terrain.grid()
		var same := _vcam.has_world() and is_equal_approx(_vcam.world_w_m, g.x * _terrain.cell_m()) 			and is_equal_approx(_vcam.world_h_m, g.y * _terrain.cell_m())
		if _rebind_check_grid and not same:
			_frame_world_pending = true
		_rebind_check_grid = false
		_vcam.set_world(g.x, g.y, _terrain.cell_m())
	if _frame_world_pending:
		_frame_world_pending = false
		_place_pending = false
		frame_world()
	elif _place_pending:
		_place_pending = false
		if _entry.has("centre") and _entry.has("px_per_cell"):
			_vcam.enter_from_2d(_entry["centre"], float(_entry["px_per_cell"]), float(_sv.size.y), true)
		else:
			frame_world()
	_vcam.step(delta)
	last_clamp_steps = _vcam.clamp_to_ground(_terrain.surface_bound)
	## The level 2D would draw at this ground scale, through the engine's own
	## rule (`lod_level_for_zoom`), capped by the engine and the quality tier
	## exactly as `ViewportHost._update_lod()` caps 2D's.
	var ppc := _vcam.cell_m / maxf(_vcam.metres_per_px(float(_sv.size.y)), 1e-9)
	var z2d: int = _bridge.lod_level_for_zoom(ppc)
	var max_z: int = _bridge.lod_max_level()
	var b: Dictionary = _bridge.lod_budget()
	if int(b.get("max_level", 0)) > 0:
		max_z = mini(max_z, int(b["max_level"]))
	var changed := _terrain.update(z2d, max_z, Vector2(_vcam.pivot_x, _vcam.pivot_z) / _vcam.cell_m)
	var pos := _vcam.position()
	var hag: float = float(pos[1]) - _terrain.surface_bound(pos[0], pos[2])
	last_hag = hag
	_terrain.place(pos, hag)
	_cam.rotation = _vcam.rotation()
	_cam.projection = Camera3D.PROJECTION_ORTHOGONAL if _vcam.ortho else Camera3D.PROJECTION_PERSPECTIVE
	if _vcam.ortho:
		_cam.size = _vcam.ortho_size
	var near := View3DCamera.near_for(hag)
	_cam.near = near
	_cam.far = maxf(_vcam.far_for(pos), near * 2.0)
	if changed:
		_refresh()

## **3D-3's per-frame step for Standard and Full**: the same camera steps as
## Basic's (entry pose, tilt, ground clamp, §3.11's depth range), then the
## quadtree's selection against the camera's frustum. Basic's patch is hidden
## while the quadtree draws, so the two never draw over each other.
func _process_quad(delta: float) -> void:
	var wg: Object = _bridge.get("world_gen")
	if _quad == null:
		_quad = View3DQuadtree.new()
		_sv.add_child(_quad)
		_quad.set_exaggeration(_terrain.exaggeration())
	if _quad.tier_key() != quad_key() or not _quad_bound:
		_quad.bind(wg, _readable, quad_key())
		_quad_bound = true
	if _terrain.window().size() > 0 or _terrain.world_known():
		_terrain.clear("this tier draws the quadtree")
	_quad.set_boost(bool(_entry.get("ef9", false)), float(_entry.get("lod_entry_ppc", 0.0)))
	var vh := float(_sv.size.y)
	if not _quad.world_known() or not _vcam.has_world():
		_quad.update([0.0, 0.0, 0.0], [], 1.0, 0.0, _bridge)
		if not _quad.world_known():
			_refresh()
			return
		var g := _quad.grid()
		var same := _vcam.has_world() and is_equal_approx(_vcam.world_w_m, g.x * _quad.cell_m()) and is_equal_approx(_vcam.world_h_m, g.y * _quad.cell_m())
		if _rebind_check_grid and not same:
			_frame_world_pending = true
		_rebind_check_grid = false
		_vcam.set_world(g.x, g.y, _quad.cell_m())
	if _frame_world_pending:
		_frame_world_pending = false
		_place_pending = false
		frame_world()
	elif _place_pending:
		_place_pending = false
		if _entry.has("centre") and _entry.has("px_per_cell"):
			_vcam.enter_from_2d(_entry["centre"], float(_entry["px_per_cell"]), vh, true)
		else:
			frame_world()
	_vcam.step(delta)
	last_clamp_steps = _vcam.clamp_to_ground(_surface_bound)
	var pos := _vcam.position()
	var hag: float = float(pos[1]) - _quad.surface_bound(pos[0], pos[2])
	last_hag = hag
	_cam.rotation = _vcam.rotation()
	_cam.projection = Camera3D.PROJECTION_ORTHOGONAL if _vcam.ortho else Camera3D.PROJECTION_PERSPECTIVE
	if _vcam.ortho:
		_cam.size = _vcam.ortho_size
	var near := View3DCamera.near_for(hag)
	_cam.near = near
	_cam.far = maxf(_vcam.far_for(pos), near * 2.0)
	var f_px := vh / (2.0 * tan(deg_to_rad(View3DCamera.FOV_DEG) * 0.5))
	var ortho_ppc := _vcam.cell_m / maxf(_vcam.metres_per_px(vh), 1e-9) if _vcam.ortho else 0.0
	if _quad.update(pos, _cam.get_frustum(), f_px, ortho_ppc, _bridge):
		_refresh()

## §3.9's input, on this node (it is `MOUSE_FILTER_STOP` and covers the map;
## the card and the navpad sit above it and take their own events first).
func _gui_input(event: InputEvent) -> void:
	if not draws_terrain():
		return
	var f := _phys()
	var vh := float(_sv.size.y)
	if event is InputEventScreenTouch:
		var t := event as InputEventScreenTouch
		if t.pressed:
			_touches[t.index] = t.position * f
		else:
			_touches.erase(t.index)
		## The two-finger baseline is taken as the second finger lands, not on
		## the first drag after it: a first cut reset it here and lost the
		## first event's share of a twist (17.5° of 20° in the tablet probe,
		## 2026-11-12), because that event moves one finger before the other.
		_pinch = {}
		if _touches.size() >= 2:
			_two_finger()
		accept_event()
	elif event is InputEventScreenDrag:
		var d := event as InputEventScreenDrag
		if not _touches.has(d.index):
			return
		var prev: Vector2 = _touches[d.index]
		_touches[d.index] = d.position * f
		if _touches.size() == 1:
			_vcam.pan_px(d.position * f - prev, vh)
		else:
			_two_finger()
		accept_event()
	elif event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		## A finger's emulated mouse events (device -1, `ViewportHost._is_touch_mouse`'s
		## test): the touch branch above already handled the finger.
		if mb.device < 0:
			return
		if mb.pressed and mb.button_index == MOUSE_BUTTON_WHEEL_UP:
			_vcam.zoom(ViewportHost.ZOOM_WHEEL_STEP)
		elif mb.pressed and mb.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_vcam.zoom(1.0 / ViewportHost.ZOOM_WHEEL_STEP)
		elif mb.button_index in [MOUSE_BUTTON_LEFT, MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT]:
			_drag_button = mb.button_index if mb.pressed else 0
		accept_event()
	elif event is InputEventMouseMotion:
		var mm := event as InputEventMouseMotion
		if mm.device < 0 or _drag_button == 0:
			return
		var rel := mm.relative * f
		if _drag_button == MOUSE_BUTTON_RIGHT:
			_vcam.orbit(-deg_to_rad(rel.x * ORBIT_DEG_PER_PX), deg_to_rad(rel.y * ORBIT_DEG_PER_PX))
		else:
			_vcam.pan_px(rel, vh)
		accept_event()

## Two fingers: pinch zooms about the pivot, twist turns, and the pair's
## centroid moving up or down tilts. Each reading is against the last event's.
func _two_finger() -> void:
	var keys := _touches.keys()
	var a: Vector2 = _touches[keys[0]]
	var b: Vector2 = _touches[keys[1]]
	var dist := a.distance_to(b)
	var ang := (b - a).angle()
	var mid := (a + b) * 0.5
	if not _pinch.is_empty() and dist > 1.0 and float(_pinch["dist"]) > 1.0:
		_vcam.zoom(dist / float(_pinch["dist"]))
		_vcam.orbit(-wrapf(ang - float(_pinch["ang"]), -PI, PI), deg_to_rad((mid.y - (_pinch["mid"] as Vector2).y) * ORBIT_DEG_PER_PX))
	_pinch = {"dist": dist, "ang": ang, "mid": mid}

## Runs the §4.2 probe now and stores what it measured. The user's forced tier
## is kept (§4.4 forcing is a choice the user made, not a measurement).
func retest() -> void:
	if _busy:
		return
	_busy = true
	_note = ""
	_refresh()
	## Two frames so the "Testing" card is on screen before the probe's first
	## readback stalls the pipeline.
	await get_tree().process_frame
	await get_tree().process_frame
	_sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	## Never time a frame the user cannot see: if layout has not given the
	## SubViewport a real size yet, the result would describe a 2 x 2 render.
	if _sv.size.x < MIN_PROBE_PX or _sv.size.y < MIN_PROBE_PX:
		_sv.render_target_update_mode = SubViewport.UPDATE_WHEN_VISIBLE
		_busy = false
		_note = "The 3D test did not run: the 3D view had no size on screen (%d x %d px)." % [_sv.size.x, _sv.size.y]
		_refresh()
		return
	var r := await _gate.run(_sv)
	_sv.render_target_update_mode = SubViewport.UPDATE_WHEN_VISIBLE
	DccSettings.set_view3d_result(r)
	_busy = false
	_refresh()

func _open_force_menu() -> void:
	var f := forced_tier()
	_force_menu.set_item_checked(0, f < 0)
	for t in View3DGate.TIER_NAMES.size():
		_force_menu.set_item_checked(t + 1, f == t)
	var r := _force_btn.get_global_rect()
	_force_menu.popup(Rect2i(Vector2i(r.position + Vector2(0, r.size.y)), Vector2i.ZERO))

## §4.4's "Force tier…". Automatic (-1) or a tier at or below the measured one
## applies at once; a tier above it asks first, with the failure classes this
## project has already hit, then runs that tier's check.
func request_force(t: int) -> void:
	var measured := View3DGate.effective_tier(result(), -1)
	if t < 0 or t <= measured:
		DccSettings.set_view3d_forced_tier(t)
		_note = ""
		_refresh()
		return
	## A fresh dialog each time: `DccWidgets.confirm()` frees it on either
	## answer. Hosted on the app (found by capability, as
	## `ViewportHost._announce_pan_mode` finds the shell) so the phone gets
	## its own dialog form; a probe hosting this node alone gets the desktop one.
	_warn = DccWidgets.confirm(_app_host(), "Force 3D %s?" % View3DGate.TIER_NAMES[t],
		force_warning(t, measured), "Force %s" % View3DGate.TIER_NAMES[t],
		func(): apply_force(t))
	## Desktop path only (the phone form is borderless and already wraps its
	## prose): `confirm()` sets `dialog_text` unwrapped, which clipped this
	## three-sentence warning to one line in the first capture
	## (`05_force_warning.png`). Wrap it and re-centre at a readable width.
	## An explicit height, because an autowrapped label asked to size itself
	## from width 0 collapses the body to nothing (second capture: only the
	## title bar drew). 460 x 220 holds the three sentences at the desktop font
	## (labelled judgement, checked in `_view3d_probe.gd` capture 05).
	## `dialog_autowrap` alone did not wrap here (the third capture still drew
	## one clipped line: `confirm()` turns `wrap_controls` off), so the label is
	## given its own width to wrap against.
	if not _warn.borderless:
		_warn.dialog_autowrap = true
		var lbl := _warn.get_label()
		lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		lbl.custom_minimum_size.x = 430
		_warn.min_size = Vector2i(460, 220)
		_warn.popup_centered(Vector2i(460, 220))

## The nearest ancestor that can present a phone dialog (`DccApp`), else self.
func _app_host() -> Node:
	var n: Node = get_parent()
	while n != null:
		if n.has_method("phone_fit") and n.has_method("is_phone"):
			return n
		n = n.get_parent()
	return self

## The warning §4.4 asks for: not only "may be slow", but the two failure
## classes already seen on test hardware, and what happens if it fails.
static func force_warning(t: int, measured: int) -> String:
	return ("This device passed the test for %s, not %s. Forcing a higher tier can freeze the app or reset the graphics driver; "
		+ "both have happened on test hardware (a Vulkan device loss, and a crash on an AMD card under this renderer). "
		+ "If %s fails here, 3D drops one tier and tells you why.") % [
		View3DGate.TIER_NAMES[measured], View3DGate.TIER_NAMES[t], View3DGate.TIER_NAMES[t]]

## Applies a forced tier above the measured one: runs `View3DGate.check_forced()`
## and stores the tier that held, which is lower than asked when the check
## failed, with the drop named in the card (§4.4). Never crashes on a failure:
## the gate reports it as data.
func apply_force(t: int) -> void:
	if _busy:
		return
	_busy = true
	_refresh()
	await get_tree().process_frame
	var measured := View3DGate.effective_tier(result(), -1)
	_sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	var c := await _gate.check_forced(_sv, t, measured)
	_sv.render_target_update_mode = SubViewport.UPDATE_WHEN_VISIBLE
	var held: int = c["tier"]
	## A force that dropped all the way to the measured tier is no longer a
	## force; storing it as one would print "forced" over a measured state.
	DccSettings.set_view3d_forced_tier(held if held > measured else -1)
	_note = String(c["note"])
	_busy = false
	_refresh()
