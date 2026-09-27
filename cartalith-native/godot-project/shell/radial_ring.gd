extends Control
## `MAP_CONTEXT_SCOPE.md` §5 and §9.1 -- CM-3, the desktop ring. Draws the
## 8-slot compass §5.1 defines plus its one sub-ring (§5.2 rule 2), and is the
## whole gesture state machine §6's table describes: every timing decision
## (the no-wait flick, the still-hold that opens the ring *and* the card, the
## sticky Q ring) lives here. Driven entirely by plain calls from
## `context_broker.gd` -- `arm()`/`pointer()`/`release()`/`click_at()` -- which
## itself is driven by `map_overlay.gd`'s raw RMB press/motion/release and
## `app.gd`'s Q key handling. This file never reads the mouse or the keyboard
## itself: `mouse_filter` stays `IGNORE` throughout, so map_overlay's own
## `_gui_input` keeps owning every real input event, exactly as CM-1/CM-2 left
## it, and forwards to this file only the calls it needs.
##
## **Never a second implementation of arming** (§5.2 rule 5). Every leaf
## slot's `callable` is a `Callable` a provider built (`GlobalTools.
## ring_cardinals`, each workspace's own `ring_slots`) by calling the exact
## function its own dock control already calls -- this file only decides
## WHICH callable the gesture picked and runs it. It draws the compass; it
## does not invent what any slot does.
##
## `faction_banner.gd`'s `_draw()` is the in-tree precedent for a hand-drawn
## `Control` this shape (§9.1's own table names it as such).

signal hold_fired  ## §6's "hold >= 300ms, still": the ring is now visible and
	## sticky; the caller (`context_broker.gd`) opens the card beside it.
## CM-4 (`MAP_CONTEXT_SCOPE.md` §7.1's own table: "a `tool_arm` pulse on
## crossing into a slot"). Fired the moment `_hover`/`_sub.hover` transitions
## from empty (or a different slot) to a real one -- never on every sample, or
## a still finger sitting on one slot would buzz every frame. Top ring and
## sub-ring share it; `context_broker.gd` does not need to know which.
signal hover_entered

const RING_RADIUS := 60.0      ## §7.2's own desktop figure: "60 px on desktop"
const SLOT_SIZE := 46.0        ## drawn slot disc diameter -- tuned, no reference value
const DEAD_ZONE := 24.0        ## §5.2 rule 4, desktop
## Smaller than the main ring, mirroring the design canvas's own SUBR<RR
## relation (`design/map-context-2026-09-25/Main.dc.html`: RR=92, SUBR=84) --
## keeps the sub-ring reading as "inside" the parent slot it opened from.
const SUB_RING_RADIUS := 50.0
## Larger than `DEAD_ZONE`, mirroring that same canvas's SUBDEAD(32) >
## DEAD(22): the path back toward the parent ring's own centre should not
## read as an accidental cancel of the sub-ring.
const SUB_DEAD_ZONE := 30.0

## CM-4 residual (`OUTSTANDING_WORK.md`; `MAP_CONTEXT_SCOPE.md` §7.2): "Ring
## radius scales with the finger. Slots sit at 96 dp on touch (60 px on
## desktop) ... 96 dp leaves generous gaps and a clear angular sector per
## slot." This file drew the desktop `RING_RADIUS`/`SLOT_SIZE` unconditionally
## until now -- a tablet's touch-hold gesture (`MAP_CONTEXT_SCOPE.md` §7.1)
## opened the same 60 px ring a mouse gets, well under the 96 dp target.
##
## `_r*` below are what every drawing/hit-test function in this file actually
## reads; the `RING_RADIUS`/etc. consts above stay as the authored desktop
## figures the scaling is computed FROM, exactly as `DccTheme.TABLET`'s own
## header keeps a figure's desktop key beside its touch value rather than
## overwriting it.
var _r_ring := RING_RADIUS
var _r_slot := SLOT_SIZE
var _r_dead := DEAD_ZONE
var _r_sub_ring := SUB_RING_RADIUS
var _r_sub_dead := SUB_DEAD_ZONE

const HOLD_MS := 300     ## §6: "hold >= 300ms, still" -> ring + card
## §6: "the ring appears only if the button is still held after 150ms" --
## measured from the moment travel first exceeded the click slop, not from
## the press. `map_overlay.gd`'s own `_RMB_CLICK_SLOP` is the same 8 px; kept
## as a separate named constant here (not shared code) because the two files
## must not import from each other for a single number, but must not drift
## either -- if one changes, grep the other's citation of this comment.
const FLICK_MS := 150
const CLICK_SLOP := 8.0

const DIRS := ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]
const ANGLES := {
	"N": -90.0, "NE": -45.0, "E": 0.0, "SE": 45.0,
	"S": 90.0, "SW": 135.0, "W": 180.0, "NW": -135.0,
}

var app

## `_armed`: a press has been recorded and the gesture has not yet declared
## itself a drag or a hold (RMB only -- Q skips straight to `_active`).
## `_active`: the ring genuinely exists as a gesture in progress or a sticky
## open surface, whether or not it is currently *drawn* (`_visible`).
var _armed := false
var _active := false
var _visible := false
var _sticky := false
var _q := false
var _slots: Dictionary = {}
var _centre := Vector2.ZERO
var _press_pos := Vector2.ZERO
var _press_ms := 0
var _moved_ms := -1
var _hover := ""
var _sub: Dictionary = {}   ## {} closed; else {"dir","items","centre","hover"}


func setup(app_ref) -> void:
	app = app_ref
	## Drawn above the whole shell in its own space, independent of whatever
	## container `app.add_child(ring)` happened to land it under -- the same
	## reason `context_card.gd`'s popup converts through
	## `get_global_transform_with_canvas()` before it ever reaches this file.
	top_level = true
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	visible = false
	set_process(false)
	_apply_touch_scale()


## §7.2's touch figures, computed once -- `DccShell._compute_layout_mode()`'s
## own header is why: "Phone-vs-tablet is decided once ... a device's own form
## factor is not something that changes at runtime," so there is no resize
## hook to re-run this from, the same way `_phone_scale` itself is set once at
## boot and never revisited.
##
## `DccTheme.TOUCH_SCALE` (1.53) is this shell's one documented general-
## purpose touch multiplier -- reused here rather than inventing a ring-only
## factor, per its own header: "the fallback for any figure the table [below
## it] does not name." Applied plainly it turns the desktop 60 px ring into
## 91.8, short of the 96 dp §7.2 names outright, so the ring radius alone
## additionally takes 96 as a floor -- the same shape `DccShell._ptap()`
## already uses for its own named touch-target floor (scale, then never let
## the result fall under the figure the spec states). Every other figure here
## (slot size, dead zones, the sub-ring) has no such named target, so plain
## `TOUCH_SCALE` is all they take.
func _apply_touch_scale() -> void:
	if not DccTheme.is_tablet():
		return
	_r_ring = maxf(96.0, RING_RADIUS * DccTheme.TOUCH_SCALE)
	_r_slot = SLOT_SIZE * DccTheme.TOUCH_SCALE
	_r_dead = DEAD_ZONE * DccTheme.TOUCH_SCALE
	_r_sub_ring = SUB_RING_RADIUS * DccTheme.TOUCH_SCALE
	_r_sub_dead = SUB_DEAD_ZONE * DccTheme.TOUCH_SCALE


func is_open() -> bool:
	return _active


## Probe-only introspection (`_ctxring_probe.gd`) -- everything a test needs
## to assert through the real state machine rather than by re-deriving it.
func debug_state() -> Dictionary:
	return {
		"active": _active, "visible": _visible, "sticky": _sticky, "q": _q,
		"centre": _centre, "hover": _hover,
		"sub_open": not _sub.is_empty(),
		"sub_centre": _sub.get("centre", Vector2.ZERO),
		"sub_hover": int(_sub.get("hover", -1)),
		"sub_count": (_sub.get("items", []) as Array).size(),
		## CM-4 residual: the LIVE, possibly touch-scaled geometry
		## (`_apply_touch_scale()`) -- so a probe measures what this node
		## actually drew rather than re-declaring the desktop constants and
		## asserting them against themselves (`MISTAKES.md`'s preflight rule).
		"radius": _r_ring, "slot": _r_slot, "dead": _r_dead,
		"sub_radius": _r_sub_ring, "sub_dead": _r_sub_dead,
	}


## Fills the whole viewport so `_centre`/`_clamp_centre()` can work in plain
## screen pixels. Re-run on every `arm()` rather than once, so a window
## resize between two ring opens is never stale.
func _fit_to_viewport() -> void:
	var r := get_viewport().get_visible_rect()
	position = Vector2.ZERO
	size = r.size


## Begin a gesture at `pos` (already converted to this control's space by
## `context_broker.gd`). `slots` is the 8-entry compass `ring_collect()`
## built; `q` is true only for the Q-key path, which is visible immediately
## and carries no hold/flick timing at all (§6's row for Q has none).
func arm(pos: Vector2, slots: Dictionary, q: bool) -> void:
	_fit_to_viewport()
	_slots = slots
	_centre = _clamp_centre(pos)
	_press_pos = pos
	_press_ms = Time.get_ticks_msec()
	_moved_ms = -1
	_hover = ""
	_sub = {}
	_q = q
	_sticky = false
	if q:
		_armed = false
		_active = true
		_visible = true
		visible = true
		set_process(false)
	else:
		_armed = true
		_active = false
		_visible = false
		visible = false
		set_process(true)
	queue_redraw()


## Every pointer sample while the gesture is outstanding, RMB-held or
## Q-aiming alike (`context_broker.gd` forwards every `map_overlay.gd` motion
## event here regardless of button state, and this function no-ops unless a
## gesture is actually in progress).
func pointer(pos: Vector2) -> void:
	if _active:
		if not _sub.is_empty():
			_update_sub_hover(pos)
			queue_redraw()
			return
		_update_hover(pos)
		if _moved_ms < 0 and not _q and pos.distance_to(_press_pos) > CLICK_SLOP:
			_moved_ms = Time.get_ticks_msec()
		queue_redraw()
		return
	if _armed and pos.distance_to(_press_pos) > CLICK_SLOP:
		## Travel exceeded the slop before the still-hold timer fired: this is
		## a drag, and the ring itself now exists (though not necessarily
		## drawn yet -- see `_process()`'s flick-delay branch).
		_armed = false
		_active = true
		_moved_ms = Time.get_ticks_msec()
		_update_hover(pos)
		queue_redraw()


func _process(_dt: float) -> void:
	if _q:
		set_process(false)
		return
	var now := Time.get_ticks_msec()
	if _armed and not _active and now - _press_ms >= HOLD_MS:
		## §6's third row: a still hold opens the ring AND the card. NOT
		## `_sticky = true` here -- that is `release()`'s own decision (a
		## release with no hover goes sticky; `release()` needs to see
		## `_sticky` still false to know this is the gesture's FIRST release
		## rather than a later click into an already-sticky ring).
		_armed = false
		_active = true
		_visible = true
		visible = true
		hold_fired.emit()
		queue_redraw()
		return
	if _active and _moved_ms >= 0 and not _visible and now - _moved_ms >= FLICK_MS:
		_visible = true
		visible = true
		queue_redraw()
	if not _armed and not _active:
		set_process(false)


## The button-up (or Q key-up via `q_release()`). Returns the chosen leaf's
## `Callable`, or an invalid one when nothing was chosen -- `context_broker.
## gd` calls it if valid. Also the sticky click's own decision function
## (`click_at()` below re-uses it after re-aiming at the click position), so
## there is exactly one place that decides what a release means.
func release() -> Callable:
	_armed = false
	if not _active:
		set_process(false)
		return Callable()
	var already_sticky := _sticky
	if not _sub.is_empty():
		var hi := int(_sub.get("hover", -1))
		if hi >= 0:
			var items: Array = _sub["items"]
			var cb: Callable = (items[hi] as Dictionary).get("callable", Callable())
			close()
			return cb
		if already_sticky:
			## A sticky sub-ring, clicked outside every one of its own
			## slots: cancel rather than re-arm sticky a second time.
			close()
		else:
			_sticky = true
			_visible = true
			visible = true
			set_process(false)
		return Callable()
	if _hover != "":
		var row: Dictionary = _slots.get(_hover, {})
		if row.is_empty() or not bool(row.get("enabled", true)):
			close()
			return Callable()
		if row.has("children"):
			_open_sub(_hover, row["children"])
			return Callable()
		var cb: Callable = row.get("callable", Callable())
		close()
		return cb
	## The dead zone, or outside every sector.
	if already_sticky:
		close()
	else:
		_sticky = true
		_visible = true
		visible = true
		set_process(false)
	return Callable()


## Q's key-up. A no-op once sticky (§6: "a tap leaves it open, sticky" -- a
## later key-up must not re-resolve an already-sticky ring).
func q_release() -> Callable:
	if not _active or not _q or _sticky:
		return Callable()
	return release()


## The sticky click path (§6/§7.1's "tap outside to dismiss", the design
## canvas's own per-slot click handler): a left click while the ring is open
## and sticky selects whatever it lands on. Re-aims at the click point first,
## since a sticky ring may not have had its hover updated by a plain click
## (no drag motion precedes it).
func click_at(pos: Vector2) -> Callable:
	if not _active:
		return Callable()
	pointer(pos)
	return release()


func close() -> void:
	_armed = false
	_active = false
	_visible = false
	_sticky = false
	_sub = {}
	visible = false
	set_process(false)
	queue_redraw()


func _open_sub(dir: String, items: Array) -> void:
	var ang := deg_to_rad(ANGLES[dir])
	var sub_centre: Vector2 = _centre + Vector2(cos(ang), sin(ang)) * _r_ring
	_sub = {"dir": dir, "items": items, "centre": _clamp_centre(sub_centre), "hover": -1}
	_visible = true
	visible = true
	set_process(false)
	queue_redraw()


func _update_hover(pos: Vector2) -> void:
	var prev := _hover
	if pos.distance_to(_centre) <= _r_dead:
		_hover = ""
		return
	var ang := rad_to_deg(atan2(pos.y - _centre.y, pos.x - _centre.x))
	var best := ""
	var best_d := 361.0
	for dir in DIRS:
		var d: float = absf(wrapf(ang - float(ANGLES[dir]), -180.0, 180.0))
		if d < best_d:
			best_d = d
			best = dir
	_hover = best
	if _hover != "" and _hover != prev:
		hover_entered.emit()


func _update_sub_hover(pos: Vector2) -> void:
	var c: Vector2 = _sub["centre"]
	var items: Array = _sub["items"]
	var n := items.size()
	var prev := int(_sub.get("hover", -1))
	if n == 0 or pos.distance_to(c) <= _r_sub_dead:
		_sub["hover"] = -1
		return
	var ang := rad_to_deg(atan2(pos.y - c.y, pos.x - c.x))
	var best := 0
	var best_d := 361.0
	for i in n:
		var a := -90.0 + 360.0 * float(i) / float(n)
		var d := absf(wrapf(ang - a, -180.0, 180.0))
		if d < best_d:
			best_d = d
			best = i
	_sub["hover"] = best
	if best != prev:
		hover_entered.emit()


## Keeps the ring's own outer edge, plus room for the caption band under it,
## inside the viewport at all four screen edges (§5's "the ring stays on
## screen at the four edges" -- the probe's own check).
func _clamp_centre(pos: Vector2) -> Vector2:
	var pad := _r_ring + _r_slot * 0.5 + 40.0
	var sz := size
	if sz.x <= 0.0 or sz.y <= 0.0:
		return pos
	return Vector2(
		clampf(pos.x, pad, maxf(pad, sz.x - pad)),
		clampf(pos.y, pad, maxf(pad, sz.y - pad)))


# -- Drawing ------------------------------------------------------------------

func _draw() -> void:
	if not _active or not _visible:
		return
	_draw_ring(_centre, _r_ring, _r_dead, _slots, _hover, _sub.is_empty())
	if not _sub.is_empty():
		_draw_sub_ring(_sub["centre"], _sub["items"], int(_sub.get("hover", -1)))
	_draw_caption()


## One ring's worth of slots, shared by the top-level compass and (via
## `_draw_sub_ring`, its own simpler pass below) the one sub-ring §5.2 rule 2
## allows. Ground is `sunken` normally and `accent` when the slot's own
## `armed` flag is set (§5.2 rule 3, MN-21's precedent: reversed
## accent-ink type on a filled accent surface -- `GUI_GAP_REGISTER.md`
## MN-21, `DccTheme`'s `accent_ink` token). A slot with no provider row
## (a domain workspace that returned nothing for this direction) or an
## explicitly `enabled: false` one is drawn at 35% opacity with its reason in
## the centre caption when hovered (§5.2 rule 1).
func _draw_ring(centre: Vector2, radius: float, dead: float, slots: Dictionary,
		hover_dir: String, top_active: bool) -> void:
	draw_circle(centre, dead, DccTheme.c("panel_alt"))
	draw_arc(centre, dead, 0.0, TAU, 28, DccTheme.c("line"), 1.0, true)
	for dir in DIRS:
		var row: Dictionary = slots.get(dir, {})
		var ang := deg_to_rad(float(ANGLES[dir]))
		var pos: Vector2 = centre + Vector2(cos(ang), sin(ang)) * radius
		var present := not row.is_empty()
		var enabled := present and bool(row.get("enabled", true))
		var armed := present and bool(row.get("armed", false))
		var hovered := top_active and present and String(dir) == hover_dir
		var alpha := 1.0 if enabled else 0.35
		var ground := DccTheme.c("accent") if armed else DccTheme.c("sunken")
		draw_circle(pos, _r_slot * 0.5, Color(ground.r, ground.g, ground.b, ground.a * alpha))
		var border := DccTheme.c("accent") if (hovered or armed) else DccTheme.c("line")
		draw_arc(pos, _r_slot * 0.5, 0.0, TAU, 28,
			Color(border.r, border.g, border.b, alpha), 2.0 if hovered else 1.0, true)
		if not present:
			continue
		## MN-21: reversed, paper-coloured ink on the filled accent surface.
		var ink := DccTheme.c("accent_ink") if armed else DccTheme.c("text_bright")
		ink.a = alpha
		_draw_glyph(String(row.get("glyph", "")), pos, _r_slot * 0.52, ink)
		_draw_label(String(row.get("label", "")), pos + Vector2(0.0, _r_slot * 0.5 + 11.0), ink)


func _draw_sub_ring(centre: Vector2, items: Array, hover_i: int) -> void:
	draw_circle(centre, _r_sub_dead, DccTheme.c("panel_alt"))
	draw_arc(centre, _r_sub_dead, 0.0, TAU, 24, DccTheme.c("accent"), 2.0, true)
	var n := items.size()
	for i in n:
		var row: Dictionary = items[i]
		var ang := deg_to_rad(-90.0 + 360.0 * float(i) / float(maxi(1, n)))
		var pos: Vector2 = centre + Vector2(cos(ang), sin(ang)) * _r_sub_ring
		var hovered := i == hover_i
		var ground := DccTheme.c("accent") if hovered else DccTheme.c("sunken")
		draw_circle(pos, _r_slot * 0.44, ground)
		draw_arc(pos, _r_slot * 0.44, 0.0, TAU, 24, DccTheme.c("line"), 1.0, true)
		var ink := DccTheme.c("accent_ink") if hovered else DccTheme.c("text_bright")
		_draw_glyph(String(row.get("glyph", "")), pos, _r_slot * 0.42, ink)
		_draw_label(String(row.get("label", "")), pos + Vector2(0.0, _r_slot * 0.44 + 10.0), ink)


## §5.2 rule 4: "the centre shows the hovered slot's full label and shortcut,
## so labels on the ring can stay short."
func _draw_caption() -> void:
	var text := "ring · centre cancels"
	var below_centre := _centre
	if not _sub.is_empty():
		var items: Array = _sub["items"]
		var hi := int(_sub.get("hover", -1))
		var parent: Dictionary = _slots.get(String(_sub.get("dir", "")), {})
		var picked := "pick one"
		if hi >= 0 and hi < items.size():
			picked = String((items[hi] as Dictionary).get("label", ""))
		text = "%s ▸ %s" % [String(parent.get("label", "")), picked]
		below_centre = _sub["centre"]
	elif _hover != "":
		var row: Dictionary = _slots.get(_hover, {})
		var shortcut := String(row.get("shortcut", ""))
		text = "%s · %s%s" % [_hover, String(row.get("label", "")), (" ▸" if row.has("children") else "")]
		if not shortcut.is_empty():
			text += "  (%s)" % shortcut
		if not bool(row.get("enabled", true)):
			text += "  — %s" % String(row.get("reason", "unavailable"))
	var font := DccTheme.mono()
	var fs := DccTheme.FS_SMALL
	var ts := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
	var cap_centre := below_centre + Vector2(0.0, _r_ring + 30.0)
	var pad := Vector2(9.0, 5.0)
	var rect := Rect2(cap_centre - ts * 0.5 - pad, ts + pad * 2.0)
	draw_rect(rect, DccTheme.c("panel"))
	draw_rect(rect, DccTheme.c("line"), false, 1.0)
	draw_string(font, cap_centre - Vector2(ts.x * 0.5, -ts.y * 0.32), text,
		HORIZONTAL_ALIGNMENT_LEFT, -1, fs, DccTheme.c("text_bright"))


## `name` is a `DccIcons` glyph name, or `"sym:<key>"` for one of
## `DccIcons.SYMBOLS`'s typographic characters (Undo's `↶`) -- the same
## text-vs-drawn split §12 draws everywhere else in this shell.
func _draw_glyph(name: String, centre: Vector2, px: float, color: Color) -> void:
	if name.is_empty():
		return
	if name.begins_with("sym:"):
		var ch: String = DccIcons.SYMBOLS.get(name.substr(4), "?")
		var font := DccTheme.mono()
		var fs := int(px)
		var ts := font.get_string_size(ch, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		draw_string(font, centre - ts * 0.5 + Vector2(0.0, ts.y * 0.32), ch,
			HORIZONTAL_ALIGNMENT_LEFT, -1, fs, color)
		return
	var tex := DccIcons.get_icon(name, int(px))
	if tex == null:
		return
	draw_texture_rect(tex, Rect2(centre - Vector2(px, px) * 0.5, Vector2(px, px)), false, color)


func _draw_label(text: String, centre: Vector2, color: Color) -> void:
	if text.is_empty():
		return
	var font := DccTheme.mono()
	var fs := DccTheme.FS_MICRO
	var ts := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
	draw_string(font, centre - Vector2(ts.x * 0.5, 0.0), text,
		HORIZONTAL_ALIGNMENT_LEFT, -1, fs, color)
