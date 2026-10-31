## DRAFT, not loaded by Godot. MS-2: the domain fan. Holding the phone's domain tab opens three targets,
## WORLD · GENERATE, CIVIL and CARTO, in an arc above it; slide to one and lift, or lift in place and tap
## one (`MOBILE_SHELL_PROPOSAL.md` §2.1, §4; the mockup's `fan` overlay).
## Target: a new file, `shell/phone_domain_fan.gd`, loaded by path from `dcc_shell.gd`
## (`const _PhoneDomainFan := preload("res://shell/phone_domain_fan.gd")`). It has no `class_name`, so the import
## pass that strips `project.godot`'s comments is not needed (`MISTAKES.md`).
##
## It decides nothing about domains. The shell hands it the entries, with captions, glyphs and Session
## mode's refusals, and receives the pick back through `release_at()` or `_switch_domain_from_tab()`. The
## tool fan (`tool_fan.gd`) is the in-tree precedent for the press, pointer and release contract. It is not
## reused here because it is built around the cartwheel's eight compass slots (`ContextBroker.ring_collect()`).
##
## NOT parse-checked (no Godot binary in the session that wrote it).
extends Control

## Geometry in phone units, multiplied once by the shell's `phone_scale()`, the way `tool_fan.gd` scales
## its canvas figures. Labelled design figures, from the mockup's fan; revisited on glass in MS-10.
const RADIUS_UNITS := 112.0     ## from the top centre of the domain cell to each target's centre
const TARGET_UNITS := 64.0      ## each target's diameter: above PHONE_TAP_MIN (44) on both axes
const SPREAD_DEG := 42.0        ## the side targets' angle from straight up
const PICK_SLACK_UNITS := 8.0   ## extra reach around a target when picking

var _app                         ## DccShell (or DccApp, which extends it)
var _scale := 1.0
var _origin := Vector2.ZERO      ## the domain cell's top centre, canvas px
var _entries: Array = []         ## [{id, caption, glyph, refused}] in display order
var _centres: Dictionary = {}    ## id -> canvas position
var _hover := ""
var _current := ""
var _sticky := false             ## lifted in place: the targets now take taps

## Once, after `new()`. The node draws above the shell regardless of its parent (`top_level`), and lets
## every event through until it opens.
func setup(app) -> void:
	_app = app
	top_level = true
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE

## True while the fan is showing, held or sticky.
func is_open() -> bool:
	return visible

## Opens above `cell_rect` (the domain cell's global rect). `current` is the active domain, drawn ringed.
## `entries` come from the shell's `DOMAIN_TAB`. A target that Session mode refuses is drawn dim and cannot be
## picked.
func open_above(cell_rect: Rect2, current: String, entries: Array) -> void:
	_scale = float(_app.call("phone_scale")) if _app != null and _app.has_method("phone_scale") else 1.0
	_entries = entries
	_current = current
	_hover = ""
	_sticky = false
	var vp := get_viewport_rect()
	position = Vector2.ZERO
	size = vp.size
	_origin = Vector2(cell_rect.get_center().x, cell_rect.position.y)
	var r := RADIUS_UNITS * _scale
	var half := TARGET_UNITS * _scale * 0.5
	_centres.clear()
	var n := _entries.size()
	for i in n:
		var t := 0.0 if n <= 1 else float(i) / float(n - 1)
		var a := deg_to_rad(-SPREAD_DEG + 2.0 * SPREAD_DEG * t)
		var c := _origin + Vector2(sin(a), -cos(a)) * r
		## Slid inward rather than clipped near a screen edge (the cartwheel's rule, MAP_CONTEXT_SCOPE §7.2).
		c.x = clampf(c.x, half + 4.0 * _scale, vp.size.x - half - 4.0 * _scale)
		_centres[String((_entries[i] as Dictionary)["id"])] = c
	visible = true
	mouse_filter = Control.MOUSE_FILTER_STOP
	queue_redraw()

## The finger moved while the fan is held: hover the target under it, with a tick on each new one.
func pointer(g: Vector2) -> void:
	var h := _target_at(g)
	if h != _hover:
		_hover = h
		if h != "" and _app != null and _app.has_method("_haptic"):
			_app.call("_haptic", "tool_arm")
		queue_redraw()

## The finger lifted. Returns the picked domain id, or "" when nothing was picked. Lifting near the domain
## cell itself leaves the fan open and tappable; lifting anywhere else closes it.
func release_at(g: Vector2) -> String:
	var h := _target_at(g)
	if h != "":
		close()
		return h
	if g.distance_to(_origin) <= RADIUS_UNITS * _scale * 0.5:
		_sticky = true
		_hover = ""
		queue_redraw()
		return ""
	close()
	return ""

## Hides the fan and lets events through again.
func close() -> void:
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_hover = ""
	_sticky = false

## The pickable target under a canvas point, or "" (a refused target is never pickable).
func _target_at(g: Vector2) -> String:
	var best := ""
	var bd := (TARGET_UNITS * 0.5 + PICK_SLACK_UNITS) * _scale
	for e in _entries:
		var ed := e as Dictionary
		if bool(ed.get("refused", false)):
			continue
		var id := String(ed["id"])
		var d := g.distance_to(_centres.get(id, Vector2.INF) as Vector2)
		if d <= bd:
			best = id
			bd = d
	return best

## Sticky state only: a release on a target picks it, and a release anywhere else closes the fan. A press
## is accepted and ignored, so one tap is not counted twice under mouse emulation.
func _gui_input(event: InputEvent) -> void:
	if not visible:
		return
	var released := false
	var pos := Vector2.ZERO
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index != MOUSE_BUTTON_LEFT:
			return
		released = not mb.pressed
		pos = mb.position
	elif event is InputEventScreenTouch:
		var st := event as InputEventScreenTouch
		released = not st.pressed
		pos = st.position
	else:
		return
	accept_event()
	if not released or not _sticky:
		return
	var picked := _target_at(get_global_transform_with_canvas() * pos)
	close()
	if picked != "" and _app != null and _app.has_method("_switch_domain_from_tab"):
		_app.call("_switch_domain_from_tab", picked)

## A light scrim, then each target: a filled disc with its glyph and its caption beneath. The hovered target
## is outlined in the accent, the current domain gets an accent ring, a refused one is dim.
func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, size), Color(DccTheme.c("bg"), 0.35))
	var rad := TARGET_UNITS * _scale * 0.5
	var font := DccTheme.mono(1, true)
	var fs_cap := maxi(1, int(round(9.5 * _scale)))
	var fs_glyph := maxi(1, int(round(16.0 * _scale)))
	for e in _entries:
		var ed := e as Dictionary
		var id := String(ed["id"])
		var c: Vector2 = _centres.get(id, Vector2.ZERO)
		var on := id == _hover
		var refused := bool(ed.get("refused", false))
		draw_circle(c, rad, DccTheme.c("panel"))
		var ring := DccTheme.c("accent") if (on or id == _current) else DccTheme.c("line")
		draw_arc(c, rad, 0.0, TAU, 48, ring, maxf(1.0, (2.0 if on else 1.0) * _scale), true)
		var ink := DccTheme.c("text_ghost") if refused else (DccTheme.c("accent") if on else DccTheme.c("text"))
		var glyph := String(ed.get("glyph", ""))
		var gw := font.get_string_size(glyph, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_glyph).x
		draw_string(font, c + Vector2(-gw * 0.5, fs_glyph * 0.35), glyph, HORIZONTAL_ALIGNMENT_LEFT, -1,
			fs_glyph, ink)
		var cap := String(ed.get("caption", id))
		var cw := font.get_string_size(cap, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_cap).x
		draw_string(font, Vector2(c.x - cw * 0.5, c.y + rad + 14.0 * _scale), cap,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fs_cap, ink)
