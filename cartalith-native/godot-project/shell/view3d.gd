extends Control
## No `class_name`, deliberately: `viewport_host.gd` preloads this file as
## `View3DScene`, so it works in a tree whose `.godot` class cache predates 3D-1.

## **The 3D view mode, empty** (`3D_VIEWPORT_SCOPE.md` §5, milestone 3D-1).
##
## What it is: the viewport mode `Window ▸ 3D view` and the touch navpad's
## 3D pill switch to (§3.14). A `SubViewport` holding a `Camera3D` over an empty
## world, plus a status card that says which 3D tier this device runs and why,
## with the numbers the capability gate (`view3d_gate.gd`) measured (§4.3).
## It draws **no terrain**: that is 3D-2. It exists first so "3D unavailable,
## here is why" works before any terrain does (research §D, Milestone 0).
##
## Why it covers the map rather than replacing it: in 3D-1 nothing 3D reads
## the 2D map, so the 2D view is left exactly as it was underneath, and leaving
## 3D shows it again untouched. Parking the 2D tiles while 3D is on (§3.12) is
## 3D-3's job, when 3D has tiles of its own to budget against them.
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

## The user asked to go back to the 2D map (the card's own button).
signal exit_requested()
## The tier, the forced tier or the disclosure changed (a test finished, a
## force was applied or dropped). Menus and the navpad re-read their text here.
signal state_changed()

## **Ruling CT (owner, 2026-10-19):** the 3D view's vertical exaggeration
## defaults to true scale, 1.0x, not 2D's 3.4x relief default, which draws
## needles (`3D_VIEWPORT_SCOPE.md` §8.3, captures 07-08). 3D-1 draws no terrain,
## so the only reader today is the card's own line saying so; 3D-2's mesh
## takes its scale from here. The ruling does not cover a user slider, and 3D-1
## adds none.
const VERTICAL_EXAGGERATION_DEFAULT := 1.0

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
	_sv.add_child(_cam)
	_cam.current = true

	_gate = View3DGate.new()
	add_child(_gate)
	_build_card()
	_refresh()

## The status card: title, the tier disclosure, the "no terrain yet" line and
## three actions. Built once; `_refresh()` rewrites its text.
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
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", int(CARD_GAP * _unit))
	row.add_theme_constant_override("v_separation", int(CARD_GAP * _unit))
	col.add_child(row)
	_back_btn = _button(row, "Back to 2D", func(): exit_requested.emit(), true)
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
		_info.text = "Terrain is not drawn in 3D yet. When it is, heights are shown at true scale (%.1f×)." % VERTICAL_EXAGGERATION_DEFAULT
	for b in [_back_btn, _retest_btn, _force_btn]:
		b.disabled = _busy
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
