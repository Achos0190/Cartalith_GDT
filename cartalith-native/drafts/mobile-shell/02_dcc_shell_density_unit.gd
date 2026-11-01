## DRAFT, not loaded by Godot. MS-1 stages S1 and S2 (`MOBILE_SHELL_MS1_SPEC.md` §3.2, §3.3).
## Target: `shell/dcc_shell.gd`. It replaces `_compute_layout_mode()` and `_is_tablet_sized()`, and adds the
## density, the probe arguments and the unit.
##
## Stage S1 is behaviour-neutral: `_compute_layout_mode()` below gives the same `_phone`, `_landscape` and
## `_phone_scale` as today for every input an existing probe or device produces (spec B2, B10). Stage S2 then
## switches the two lines marked `S2:` and starts calling `_apply_unit()`.
##
## NOT parse-checked (no Godot binary in the session that wrote it). Prerequisite: draft 01 pasted into DccTheme.
## After pasting, run `godot --headless --check-only` on `shell/dcc_shell.gd` and on `shell/app.gd`.

## The last `DccTheme.classify()` result, fed back as its `current` argument for hysteresis.
var _layout: Dictionary = {}

## The value of a user argument (`-- --density 2.625` or `-- --density=2.625`), or "" when absent.
## `OS.get_cmdline_user_args()` is what `--force-touch` already reads.
func _user_arg_value(name: String) -> String:
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		var a := String(args[i])
		if a == name and i + 1 < args.size():
			return String(args[i + 1])
		if a.begins_with(name + "="):
			return a.substr(name.length() + 1)
	return ""

## The hosting viewport's size in PHYSICAL px. That is `Window.size` for the root and `SubViewport.size`
## inside a `--vp` probe host. Never `get_viewport_rect()`: once `_apply_unit()` has set a content scale
## that rect is in dp.
func _host_px() -> Vector2:
	var vp := get_viewport()
	if vp is Window:
		return Vector2((vp as Window).size)
	if vp is SubViewport:
		return Vector2((vp as SubViewport).size)
	return get_viewport_rect().size

## Today's effective scale for the composition this size would get: short side / 412 below aspect 0.6 (the
## phone's `_phone_scale`), otherwise 1.0 (the tablet and desktop draw unscaled). Used only when no real
## density is known, so every existing probe keeps its result until it is migrated (spec §3.2, B10).
func _legacy_density() -> float:
	var px := _host_px()
	var s := minf(px.x, px.y)
	var l := maxf(px.x, px.y)
	if l <= 0.0:
		return 1.0
	if _touch and s / l < DccTheme.PHONE_ASPECT_MAX:
		return maxf(1.0, s / DccTheme.PHONE_REF_SHORT)
	return 1.0

## The density MS-1 classifies and lays out with, as `{value: float, known: bool}`. `known` is false only
## for the legacy figure, which makes `classify()` skip the dp half of the line (spec §3.2).
##   - On a device: the source stage S0 picks. Until S0 has measured it, this is today's
##     `screen_get_dpi() / 160`, the figure `_is_tablet_sized()` used. A DPI of zero or less falls back to
##     the legacy figure, as `_is_tablet_sized()` fell back to the aspect alone.
##   - Off a device: `--density D` when given, else the legacy figure.
func _density() -> Dictionary:
	if OS.has_feature("mobile"):
		## S0: replace this read with the measured source. If it cannot be read, MQ-2's answer (Ruling DY,
		## 2026-11-01) accepts `screen_get_dpi() ÷ 160`, with S0's measured change written into DECISIONS.md.
		var dpi := float(DisplayServer.screen_get_dpi(DisplayServer.window_get_current_screen()))
		if dpi > 0.0:
			return {"value": dpi / 160.0, "known": true}
		return {"value": _legacy_density(), "known": false}
	var arg := _user_arg_value("--density")
	if arg != "" and arg.to_float() > 0.0:
		return {"value": arg.to_float(), "known": true}
	return {"value": _legacy_density(), "known": false}

## The UI scale (spec §3.6). `--ui-scale S` overrides the setting for a probe run without writing it.
## S1 to S4 read 1.0. S5 replaces the last line with `DccSettings.ui_scale()`, once that exists; calling a
## static function that does not exist yet is a parse error, which is why it is not written here.
## On the phone, under MQ-3 (a), which Ruling DY chose ("Keep as today"), the result is 1.0 regardless (spec §3.3).
func _ui_scale() -> float:
	var arg := _user_arg_value("--ui-scale")
	if arg != "" and arg.to_float() > 0.0:
		return arg.to_float()
	return 1.0

## Replaces today's `_compute_layout_mode()`. The classification goes through `DccTheme.classify()`.
## Everything after the classification (the narrow and portrait publications, the tablet dock widths) is
## today's code, moved unchanged. Do not trim it while pasting.
func _compute_layout_mode() -> void:
	var px := _host_px()
	if px.x <= 0.0 or px.y <= 0.0:
		return
	var d := _density()
	var density := float(d["value"])
	_layout = DccTheme.classify(px, density, _ui_scale(), _touch, bool(d["known"]), _layout)
	DccTheme.set_layout(_layout)
	## S1 reassigns `_phone` exactly as today's body did (today a phone's resize re-runs this too). From S3 on,
	## only the boot call writes it: a changed class goes to the swap (spec §3.5), never a flipped flag under a
	## composition built for the other class. S3 changes this line to `if not _booted: _phone = ...`, and
	## `_ready()` sets `_booted = true` after the composition is built.
	_phone = bool(_layout["phone"])
	_landscape = bool(_layout["landscape"])
	## S1: today's figure, for every composition.
	_phone_scale = maxf(1.0, minf(px.x, px.y) / DccTheme.PHONE_REF_SHORT)
	## S2: the line above becomes `_phone_scale = _phone_scale_for(px, density)`.
	# ---- everything below is today's `_compute_layout_mode()` from `DccTheme.set_narrow(...)` on, unchanged.
	# It reads `get_viewport_rect().size`, which is dp once the unit is applied, and that is intended:
	# the narrow and portrait bands and the dock widths are design figures, i.e. dp.

## S2 (spec §3.3, MQ-3 (a), Ruling DY's answer): the phone's unit from density. It is capped at today's short/412, so a
## phone narrower than 412 dp keeps its 412-unit layout until MS-2 to MS-6 rebuild those surfaces. It is
## 1.0 off the phone, which makes `DccTheme.phone_scale()`'s own comment true. On the OnePlus 6T at a
## density of 2.625 this returns today's 1080/412 = 2.6214 exactly (B7). UI scale is not applied on the
## phone under MQ-3 (a).
func _phone_scale_for(px: Vector2, density: float) -> float:
	if not _phone:
		return 1.0
	return maxf(1.0, minf(density, minf(px.x, px.y) / DccTheme.PHONE_REF_SHORT))

## S2 (spec §3.3): the tablet's unit. The window the shell lives in gets a content scale, so every tablet
## figure (the `TABLET` table, `role_px()`'s touch column, the static factories) reads as dp with no
## call-site edit. The phone and the PC keep a factor of 1.
##
## S0 decides the content-scale mode, and whether text needs `oversampling_override` on the root. HD-01
## found it needed on content-scaled sub-windows, and that a resize resets it, so it would be re-applied
## from `size_changed` (`DccWidgets.oversample()` is the in-tree precedent). Until S0 has answered, this
## sets only the factor.
func _apply_unit(factor: float) -> void:
	var vp := get_viewport()
	if vp is Window:
		(vp as Window).content_scale_factor = factor
	elif vp is SubViewport:
		## A `--vp` probe host. `size_2d_override` with stretch is the SubViewport's equivalent; S0 checks it
		## draws what the root factor draws.
		var sv := vp as SubViewport
		sv.size_2d_override = Vector2i(roundi(sv.size.x / factor), roundi(sv.size.y / factor))
		sv.size_2d_override_stretch = true

## S2: what `_ready()` calls after `_compute_layout_mode()`, and what S3 calls after a resize:
## the tablet takes density × UI scale; everything else takes 1.
func _unit_factor() -> float:
	if DccTheme.is_tablet():
		var d := _density()
		return float(d["value"]) * _ui_scale()
	return 1.0

# ===================== Stage S3: live re-evaluation within a class (spec §3.4) =====================

## True once `_ready()` has built the first composition. From then on only the swap (S4) changes `_phone`.
var _booted := false
## The class question's debounce: 150 ms, a labelled judgement revisited on glass. A child of the shell itself,
## never of a composition, so a later swap (S4) cannot free it.
const RESIZE_DEBOUNCE_S := 0.15
var _resize_timer: Timer

## Replaces `_on_window_resized()` (connected to `root.size_changed` in `_ready()`). Pointer builds keep
## today's early return, so the PC is untouched (spec B9). On touch, today's in-class handling runs at once,
## so a rotation re-lays out with no added delay; only the class question waits for the size to settle.
func _on_window_resized() -> void:
	if not _touch:
		return
	if _phone:
		var was_landscape := _landscape
		_compute_layout_mode()
		if _landscape != was_landscape:
			_apply_phone_orientation()
	else:
		## Today's tablet path, unchanged: it acts only on an orientation flip, so WI-04's dragged dock widths
		## survive a same-orientation resize (spec B4).
		_on_tablet_resized()
	if _resize_timer == null:
		_resize_timer = Timer.new()
		_resize_timer.one_shot = true
		_resize_timer.wait_time = RESIZE_DEBOUNCE_S
		_resize_timer.timeout.connect(_on_resize_settled)
		add_child(_resize_timer)
	_resize_timer.start()

## The size has settled. Classify against the current class (hysteresis): a changed class goes to the swap;
## otherwise the unit is re-applied in case the density or the UI scale moved (a window dragged to another
## screen, a changed setting).
func _on_resize_settled() -> void:
	var px := _host_px()
	if px.x <= 0.0 or px.y <= 0.0:
		return
	var d := _density()
	var next := DccTheme.classify(px, float(d["value"]), _ui_scale(), _touch, bool(d["known"]), _layout)
	if bool(next["phone"]) != _phone:
		_request_class_swap(next)
		return
	_apply_unit(_unit_factor())

## S4's entry point. Until the mount path exists (spec §3.5, after S4.0's inventory) it only says it was asked:
## the shell keeps its boot class, which is today's behaviour, and the log line lets a probe see the request.
func _request_class_swap(next: Dictionary) -> void:
	print("Cartalith: size class change requested (phone=%s, %s dp); kept the boot class until MS-1 S4" %
		[str(next["phone"]), str(next["dp"])])
