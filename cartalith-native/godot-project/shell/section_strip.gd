extends PanelContainer
class_name SectionStrip

## The cross-section's own strip —
## `design/Cartalith Measurement Toolbar.dc.html` state 2: *"tool group in the
## existing options bar · cross-section in a **bottom strip** · readouts in the
## right dock."*
##
## ## Why this is an overlay and not a shell region
##
## The canvas draws the strip full width under the viewport, which in
## `dcc_shell.gd`'s stack would be a fourth bar between the viewport row and
## the timeline — a change to both the desktop *and* the phone composition,
## for a panel that is visible only while one mode of one tool is armed.
## Instead it is a bottom-anchored child of `viewport_content`, which
## `dcc_shell.gd` itself describes as *"the map surface; overlays are
## children"*, and which `resource_overlay.gd` already uses the same way. It
## costs the shell nothing when hidden, which is almost always.
##
## ## What it draws
##
## The elevation line always, filled to the baseline, with the vertical
## exaggeration the options bar sets. Under it, one **band** of the channel
## the CROSS-SECTION row selects — terrain class, lithology, rainfall or flow —
## painted as a run-length strip so a classification reads as regions rather
## than as a jagged line. Elevation selects no band; the profile is the
## reading.
##
## Scrubbing it reports the sample under the pointer *and* drops a marker on
## the map at that sample's own cell, which is the canvas's "scrub the profile
## to track the map". The marker goes through `ToolOverlay.set_handles()` —
## already a list of `{x, y, r}` circles in grid coords — rather than a sixth
## kind of overlay geometry.

## The strip at ×1 exaggeration. Vertical exaggeration grows this rather than
## squeezing the value window, which is the whole finding of the first live
## pass: a profile running from a -3 177 m trench to a 2 421 m ridge, drawn
## into a window narrowed by ×4 around its own midpoint, put every metre of
## land above the top of the plot. **Exaggeration is more screen height per
## metre at an unchanged horizontal scale** -- that is what the word means --
## so it is the panel that grows, and the value window always auto-fits the
## profile's own min..max. Nothing ever clips.
const H_STRIP := 168
const EXAG_MAX := 6.0
const PAD_L := 54
const PAD_R := 16
const PAD_T := 26
const H_BAND := 18
const H_AXIS := 16
## Fraction of the viewport the strip may not exceed however far the
## exaggeration slider is pushed -- past this it stops being a strip under the
## map and starts being the map's replacement.
const MAX_VIEWPORT_FRACTION := 0.42

var app: DccApp

var _profile: Dictionary = {}
var _channel := "elevation"
var _exaggeration := 4.0
var _samples: Array = []
var _cursor := -1          ## Index into `_samples`, `-1` for "not scrubbing".
## The continuous channels' own maximum over THIS section, for the band's
## bucketing. An absolute scale was the first attempt and it drew Hydrology as
## one solid bar: flow accumulation is near zero over most of any line and
## enormous in a channel, so every land sample landed in the same bucket. A
## band whose job is to show where a field changes has to be scaled to the
## field's own range along the line it is drawn under.
var _band_max := {"flow": 1.0, "rain": 1.0}

var _title: Label
var _cursor_label: Label
var _plot: Control

func setup(a: DccApp) -> void:
	app = a
	name = "SectionStrip"
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	custom_minimum_size.y = H_STRIP
	set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	offset_top = -H_STRIP
	add_theme_stylebox_override("panel", DccTheme.panel("panel", {"top": 1}))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	add_child(col)

	var head := DccWidgets.band(col, 14, 16, 24)
	_title = DccTheme.mono_label("SECTION", "accent", DccTheme.FS_SMALL, 2, true)
	head.add_child(_title)
	head.add_child(DccTheme.spacer())
	_cursor_label = DccTheme.mono_label("", "text_dim", DccTheme.FS_SMALL)
	head.add_child(_cursor_label)
	DccWidgets.text_button(head, "close", func(): clear())

	_plot = Control.new()
	_plot.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_plot.mouse_filter = Control.MOUSE_FILTER_STOP
	_plot.draw.connect(_draw_plot)
	_plot.gui_input.connect(_on_plot_input)
	_plot.mouse_exited.connect(func():
		_cursor = -1
		_cursor_label.text = ""
		if app != null:
			app.viewport.tool_overlay.set_handles([])
		_plot.queue_redraw())
	col.add_child(_plot)

## `result` is `bridge.measure_section()`'s own dict, straight through.
func show_profile(result: Dictionary, channel: String, exaggeration: float) -> void:
	_profile = result
	_channel = channel
	_exaggeration = clampf(exaggeration, 1.0, EXAG_MAX)
	_apply_height()
	_samples = result.get("samples", [])
	if _samples.is_empty():
		clear()
		return
	visible = true
	_band_max = {"flow": 0.0, "rain": 0.0}
	for smp in _samples:
		var sd: Dictionary = smp
		_band_max["flow"] = maxf(_band_max["flow"], float(sd.get("flow", 0.0)))
		_band_max["rain"] = maxf(_band_max["rain"], float(sd.get("rain", 0.0)))
	for k in _band_max.keys():
		_band_max[k] = maxf(1e-6, _band_max[k])
	var stats: Dictionary = result.get("stats", {})
	_title.text = "SECTION A → B   %.0f km   ×%.0f   %s" % [
		float(result.get("length_km", 0.0)), _exaggeration, _channel_label().to_lower()]

	_cursor = -1
	_cursor_label.text = ""
	_plot.queue_redraw()

func clear() -> void:
	_profile = {}
	_samples = []
	_cursor = -1
	visible = false
	if app != null:
		app.viewport.tool_overlay.set_handles([])

func _channel_label() -> String:
	for c in GlobalTools.SECTION_CHANNELS:
		var d: Dictionary = c
		if String(d["id"]) == _channel:
			return String(d["label"])
	return "Elevation"

# -- Scrubbing ------------------------------------------------------------------

func _on_plot_input(event: InputEvent) -> void:
	if not (event is InputEventMouseMotion) or _samples.is_empty():
		return
	var r := _plot_rect()
	if r.size.x <= 0.0:
		return
	var t: float = clampf((event.position.x - r.position.x) / r.size.x, 0.0, 1.0)
	_cursor = clampi(int(round(t * (_samples.size() - 1))), 0, _samples.size() - 1)
	var s: Dictionary = _samples[_cursor]
	## Elevation is already the first two readings, so the Elevation channel
	## adds nothing to the tail -- printing it there read as a stutter
	## ("… · 628 m · 0.6° · 628 m") in the first live pass.
	var tail := "" if _channel == "elevation" else " · " + _sample_channel_text(s)
	_cursor_label.text = "cursor %.0f km · %.0f m · %.1f°%s" % [
		float(s.get("km", 0.0)), float(s.get("elev_m", 0.0)),
		float(s.get("slope_deg", 0.0)), tail]
	## The canvas's "scrub the profile to track the map" -- a marker on the
	## map at exactly the cell this sample came from.
	if app != null:
		app.viewport.tool_overlay.set_handles([
			{"x": float(s.get("x", 0)), "y": float(s.get("y", 0)), "r": 4.0}])
	_plot.queue_redraw()

func _sample_channel_text(s: Dictionary) -> String:
	match _channel:
		"terrain":
			return String(s.get("biome", "—")).capitalize()
		"geology":
			return String(s.get("lithology", "—"))
		"climate":
			return "%.0f °C · rain %.2f" % [float(s.get("temp_c", 0.0)), float(s.get("rain", 0.0))]
		"hydrology":
			var o := int(s.get("river_order", 0))
			return "flow %.0f%s" % [float(s.get("flow", 0.0)), (" · order %d" % o) if o > 0 else ""]
		_:
			return "%.0f m" % float(s.get("elev_m", 0.0))

# -- Drawing ---------------------------------------------------------------------

## Vertical exaggeration, applied where it belongs: the same horizontal
## scale, more vertical pixels. Capped at `MAX_VIEWPORT_FRACTION` of the
## viewport so a slider pushed to the end still leaves a map to measure on.
func _apply_height() -> void:
	var want: float = H_STRIP * _exaggeration
	var ceiling: float = H_STRIP
	if app != null and app.viewport_content != null:
		ceiling = maxf(H_STRIP, app.viewport_content.size.y * MAX_VIEWPORT_FRACTION)
	var h: float = clampf(want, H_STRIP, ceiling)
	custom_minimum_size.y = h
	offset_top = -h

func _plot_rect() -> Rect2:
	var h: float = _plot.size.y - PAD_T - H_AXIS - (H_BAND if _channel != "elevation" else 0.0)
	return Rect2(Vector2(PAD_L, PAD_T), Vector2(maxf(1.0, _plot.size.x - PAD_L - PAD_R), maxf(1.0, h)))

func _draw_plot() -> void:
	if _samples.is_empty():
		return
	var r := _plot_rect()
	var stats: Dictionary = _profile.get("stats", {})
	var lo: float = float(stats.get("min_m", 0.0))
	var hi: float = float(stats.get("max_m", 1.0))
	## The window always fits the profile, with 4 % of headroom so the highest
	## and lowest samples are not drawn on the frame. Exaggeration is not in
	## this expression at all -- it is the panel's height (`_apply_height`),
	## which is what puts more pixels under the same metres.
	var pad: float = maxf(1.0, (hi - lo) * 0.04)
	var win_lo := lo - pad
	var win_hi := hi + pad
	var span: float = maxf(1e-6, win_hi - win_lo)

	var to_y := func(m: float) -> float:
		return r.position.y + r.size.y * (1.0 - clampf((m - win_lo) / span, 0.0, 1.0))

	# Horizontal gridlines and their metre labels.
	var font := DccTheme.mono(0)
	for i in 5:
		var v: float = win_lo + span * float(i) / 4.0
		var y: float = to_y.call(v)
		_plot.draw_line(Vector2(r.position.x, y), Vector2(r.end.x, y), DccTheme.c("line_soft"), 1.0)
		_plot.draw_string(font, Vector2(4, y + 3), "%.0f m" % v,
			HORIZONTAL_ALIGNMENT_LEFT, PAD_L - 8, 9, DccTheme.c("text_ghost"))

	# The channel band, run-length so a classification reads as regions.
	if _channel != "elevation":
		_draw_band(r)

	# The elevation line, filled to the baseline.
	var pts := PackedVector2Array()
	for i in _samples.size():
		var s: Dictionary = _samples[i]
		var x: float = r.position.x + r.size.x * float(i) / float(maxi(1, _samples.size() - 1))
		pts.append(Vector2(x, to_y.call(float(s.get("elev_m", 0.0)))))
	var fill := pts.duplicate()
	fill.append(Vector2(r.end.x, r.end.y))
	fill.append(Vector2(r.position.x, r.end.y))
	_plot.draw_colored_polygon(fill, DccTheme.c("accent_wash"))
	_plot.draw_polyline(pts, DccTheme.c("accent"), 1.4, true)

	# Sea level, when the window contains it -- the one line that means
	# something absolute on a relative axis.
	if win_lo < 0.0 and win_hi > 0.0:
		var sy: float = to_y.call(0.0)
		_plot.draw_dashed_line(Vector2(r.position.x, sy), Vector2(r.end.x, sy),
			DccTheme.c("text_faint"), 1.0, 5.0)

	# Crossings, as ticks with their own labels.
	var length: float = maxf(1e-6, float(_profile.get("length_km", 1.0)))
	for c in (_profile.get("crossings", []) as Array):
		var cd: Dictionary = c
		var cx: float = r.position.x + r.size.x * clampf(float(cd.get("km", 0.0)) / length, 0.0, 1.0)
		var col: Color = DccTheme.c("text_dim" if String(cd.get("kind", "")) == "ridge" else "accent_dim")
		_plot.draw_line(Vector2(cx, r.position.y), Vector2(cx, r.end.y), col, 1.0)

	# The distance axis, below the band where there is one -- `_plot_rect`
	# already reserves the band's height, so this only has to skip past it.
	var axis_y: float = r.end.y + 12.0 + (H_BAND if _channel != "elevation" else 0.0)
	for i in 5:
		var t := float(i) / 4.0
		var x: float = r.position.x + r.size.x * t
		_plot.draw_string(font, Vector2(x - 10, axis_y), "%.0f" % (length * t),
			HORIZONTAL_ALIGNMENT_LEFT, -1, 9, DccTheme.c("text_ghost"))
	_plot.draw_string(font, Vector2(r.end.x - 24, axis_y), "km",
		HORIZONTAL_ALIGNMENT_LEFT, -1, 9, DccTheme.c("text_ghost"))

	# The scrub cursor.
	if _cursor >= 0 and _cursor < _samples.size():
		var s: Dictionary = _samples[_cursor]
		var x: float = r.position.x + r.size.x * float(_cursor) / float(maxi(1, _samples.size() - 1))
		var y: float = to_y.call(float(s.get("elev_m", 0.0)))
		_plot.draw_line(Vector2(x, r.position.y), Vector2(x, r.end.y), DccTheme.c("accent"), 1.0)
		_plot.draw_circle(Vector2(x, y), 3.0, DccTheme.c("accent"), true, -1.0, true)

## One coloured run per contiguous stretch of the same value. Continuous
## channels (rainfall, flow) are quantised to eight steps first -- a
## per-sample gradient would be 1 024 one-pixel rectangles and read as noise.
func _draw_band(r: Rect2) -> void:
	var y := r.end.y + 2.0
	var n := _samples.size()
	var i := 0
	while i < n:
		var key := _band_key(_samples[i])
		var start := i
		while i < n and _band_key(_samples[i]) == key:
			i += 1
		var x0: float = r.position.x + r.size.x * float(start) / float(maxi(1, n - 1))
		var x1: float = r.position.x + r.size.x * float(i) / float(maxi(1, n - 1))
		_plot.draw_rect(Rect2(Vector2(x0, y), Vector2(maxf(1.0, x1 - x0), H_BAND - 4.0)),
			_band_color(key), true)

## Eight buckets over `[0, top]`, so a band always shows this section's own
## spread rather than an absolute scale most sections never reach.
func _bucket(v: float, top: float) -> int:
	return clampi(int(clampf(v / maxf(1e-9, top), 0.0, 1.0) * 7.0), 0, 7)

func _band_key(s: Variant) -> String:
	var d: Dictionary = s
	match _channel:
		"terrain":
			return String(d.get("biome", "—"))
		"geology":
			return String(d.get("lithology", "—"))
		"climate":
			return "r%d" % _bucket(float(d.get("rain", 0.0)), float(_band_max["rain"]))
		"hydrology":
			## log1p, not linear and not sqrt: flow accumulation is heavily
			## heavy-tailed -- one trunk channel carries a thousand times a
			## hillslope cell, so linear puts everything but the trunk in
			## bucket 0 and sqrt only narrowly improves on that. Checked on a
			## real 2 183 km section: linear and sqrt both drew one solid bar.
			return "f%d" % _bucket(log(1.0 + maxf(0.0, float(d.get("flow", 0.0)))), log(1.0 + float(_band_max["flow"])))
		_:
			return ""

## Deterministic from the key's own hash rather than a hand-picked palette:
## the band's job is to show *where a class changes*, and the legend for what
## each class is lives in the right dock's SAMPLED FIELDS list beside it. A
## hand-written palette here would be a fourth copy of a colour table
## (`sample_bridge.rs` already owns three) that nothing pins against.
func _band_color(key: String) -> Color:
	if key == "":
		return DccTheme.c("line_soft")
	var h := float(abs(key.hash()) % 360) / 360.0
	return Color.from_hsv(h, 0.35, 0.55, 0.9)
