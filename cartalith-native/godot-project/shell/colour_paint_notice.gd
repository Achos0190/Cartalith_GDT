extends RefCounted
## The colour-paint **map-width notice** with **Scale with the map** (`COLOUR_PAINT_SCOPE.md` §5 and §17 Q5; owner
## Ruling EA item 15 confirmed that marks keep their kilometres across a regenerate, and Ruling EB item 7 added the
## Scale with the map button to the notice).
##
## What it does. Paint is stored in map kilometres, so a regenerate at another `map_width_km` leaves every mark at the
## same ground size and position while the plate moves under it. When that happens to paint that exists, this object
## says so, once, in two places:
##   - a toast (`app.set_status`): *Map width 800 km -> 1,200 km · paint marks kept their real size*;
##   - the sentence that stays, a `warn`-coloured row heading CARTO ▸ Colours ▸ Paint colour (paint's home on every
##     form: the desktop and tablet dock and the phone's CARTO sheet) until it is dismissed or resolved:
##     *Map width 800 km -> 1,200 km: 17 paint marks kept their real size and position; 3 now on water are hidden, not deleted.*
## with three buttons. **Show marks** outlines every mark on the map for a few seconds and changes nothing. **Dismiss**
## removes the row and changes nothing. **Scale with the map** multiplies every mark's lengths by new width / old
## width about the top-left edge (`WorldGen::colour_paint_scale`), as one undo step on the paint tool's own stack,
## *Scale paint with the map*; the notice goes with it, and one undo brings both the marks and the notice back.
##
## "The width the paint is at" (`_basis`) is session state, not saved: the document records kilometres and no width
## (changing `appearance_paint.json` for it was not asked for). It is set when paint first exists, set to the
## current width when a different document arrives (an opened project: its paint is at home on its own world), set
## to the new width by Scale with the map and by painting anything new while the notice is pending (a new mark is
## placed on the new map, so the paint is accepted as at the new width), and cleared with the last mark. A project
## reopened after a regenerate-and-save therefore reads as at home: the notice is for the session in which the width
## changed, as the scope words it ("after a regenerate").
##
## What it must never do: show with no mark, show when the width is unchanged, move or delete a mark except in Scale
## with the map (which deletes none), print "0 now on water" for an engine that could not count (the clause is then
## left out), or keep an outline drawn once its few seconds are up.

## How long Show marks leaves the outlines up, in seconds. Labelled judgement: long enough to find the marks on a
## large map, short enough that they never stay behind a forgotten tool.
const OUTLINE_SECONDS := 6.0

var cp  ## the owning `ColourPaintTool`

## The map width (km) the paint is at, or `null` when unknown / no paint.
var _basis: Variant = null
## The width at which the user pressed Dismiss, or `null`: the notice is not shown again at that same width.
var _dismissed: Variant = null
## Length and hash of the engine's document at the last edit or world change, to tell a regenerate (same marks) from
## a different document (an opened project). `-1` = none seen yet.
var _doc_len := -1
var _doc_hash := 0
var _box: VBoxContainer
var _outline_gen := 0

func _init(owner_tool) -> void:
	cp = owner_tool

## Builds the notice's container into `body` (the top of the Paint colour section). It is empty and invisible
## until `refresh` has something to say.
func build_into(body: Control) -> void:
	_box = VBoxContainer.new()
	_box.add_theme_constant_override("separation", 6)
	body.add_child(_box)
	refresh()

## The engine's current map width in km, or `null` (no binding, generating, no world, no scale).
func width_km() -> Variant:
	var info: Dictionary = cp.bridge.colour_paint_info() if cp.bridge != null else {}
	return float(info["map_width_km"]) if info.has("map_width_km") else null

## `{old, new, marks}` while the notice is due, else `{}`: paint exists, its width is known and differs from the
## world's, and the user has not dismissed it at this width.
func state() -> Dictionary:
	if cp.bridge == null:
		return {}
	var info: Dictionary = cp.bridge.colour_paint_info()
	if info.is_empty() or int(info.get("marks", 0)) <= 0 or _basis == null or not info.has("map_width_km"):
		return {}
	var w := float(info["map_width_km"])
	if is_equal_approx(float(_basis), w):
		return {}
	if _dismissed != null and is_equal_approx(float(_dismissed), w):
		return {}
	return {"old": float(_basis), "new": w, "marks": int(info["marks"])}

## Redraws the row from `state()`.
func refresh() -> void:
	if _box == null or not is_instance_valid(_box):
		return
	for c in _box.get_children():
		_box.remove_child(c)
		c.queue_free()
	var s := state()
	_box.visible = not s.is_empty()
	if s.is_empty():
		return
	var l := DccTheme.label(sentence(s), "warn", DccTheme.role_px("fs_prose") if DccTheme.is_tablet() else DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 190
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_box.add_child(l)
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 6)
	row.add_theme_constant_override("v_separation", 6)
	_box.add_child(row)
	var t := 44.0 if DccTheme.is_touch() else 28.0
	for spec in [
		["Show marks", func(): show_marks(), "Outlines every paint mark on the map for a few seconds. Moves and deletes nothing."],
		["Scale with the map", func(): scale_with_map(), "Rescales every mark by the new width over the old, about the map's top-left edge, so each keeps its place and size as a share of the map. One undo step."],
		["Dismiss", func(): dismiss(), "Removes this notice. The marks keep their real kilometres."],
	]:
		var b := DccWidgets.action(row, String(spec[0]), spec[1])
		b.tooltip_text = String(spec[2])
		b.custom_minimum_size = Vector2(maxf(b.custom_minimum_size.x, t), maxf(b.custom_minimum_size.y, t))

## The sentence that stays (the scope's words), in the Preferences ▸ Units distance unit. The on-water clause is
## left out when the engine could not count.
func sentence(s: Dictionary) -> String:
	var n := int(s["marks"])
	var out := "Map width %s → %s: %d paint mark%s kept their real size and position" % [_km(float(s["old"])), _km(float(s["new"])), n, "" if n == 1 else "s"]
	var wet: Variant = cp.bridge.colour_paint_marks_on_water()
	if wet != null and int(wet) > 0:
		out += "; %d now on water %s hidden, not deleted" % [int(wet), "is" if int(wet) == 1 else "are"]
	return out + "."

## A map width in the current unit, whole numbers, comma-grouped: `1,200 km`, `746 mi`.
func _km(km: float) -> String:
	var v := int(round(DccUnits.to_unit(km)))
	var s := str(absi(v))
	var out := ""
	while s.length() > 3:
		out = "," + s.substr(s.length() - 3) + out
		s = s.substr(0, s.length() - 3)
	return "%s%s%s %s" % ["-" if v < 0 else "", s, out, DccUnits.suffix()]

# -- Bookkeeping the tool calls ---------------------------------------------------

## After every edit, undo and redo: the paint's own width is set when paint first exists and cleared with the last mark.
func on_edit() -> void:
	var info: Dictionary = cp.bridge.colour_paint_info() if cp.bridge != null else {}
	if info.is_empty():
		return
	if int(info.get("marks", 0)) <= 0:
		_basis = null
		_dismissed = null
	elif _basis == null:
		_basis = width_km()
	_remember_doc()

## A new mark was just placed on the current map: paint pending at another width is accepted as at this one. The
## notice goes; Scale with the map is no longer offered for the older marks (they keep their kilometres, as ever).
func accept() -> void:
	if not state().is_empty():
		_basis = width_km()

## The world changed. Decides, by comparing the engine's document with the one last seen, whether this was a
## regenerate under the same paint (so a changed width raises the notice) or a different document (so the paint is
## taken to be at home on the world it came with).
func on_world_changed() -> void:
	var info: Dictionary = cp.bridge.colour_paint_info() if cp.bridge != null else {}
	if info.is_empty():
		return
	var w: Variant = width_km()
	if int(info.get("marks", 0)) <= 0:
		_basis = null
		_dismissed = null
		_remember_doc()
		return
	var doc: String = cp.bridge.colour_paint_document()
	var same := _doc_len >= 0 and doc.length() == _doc_len and doc.hash() == _doc_hash
	if not same or _basis == null:
		_basis = w
		_dismissed = null
	elif w != null and not is_equal_approx(float(_basis), float(w)):
		if _dismissed != null and not is_equal_approx(float(_dismissed), float(w)):
			_dismissed = null
		if cp.app != null and cp.app.has_method("set_status"):
			cp.app.set_status("hint", "Map width %s → %s · paint marks kept their real size" % [_km(float(_basis)), _km(float(w))], "warn")
	_remember_doc()

func _remember_doc() -> void:
	var doc: String = cp.bridge.colour_paint_document() if cp.bridge != null else ""
	_doc_len = doc.length()
	_doc_hash = doc.hash()

## One undo or redo step is being moved from `entry` to `moved` by the tool's `_step`: a Scale with the map step
## carries the width the paint was at, and stepping over it swaps that with the current one.
func on_step(entry: Dictionary, moved: Dictionary) -> void:
	if entry.has("basis"):
		moved["basis"] = _basis
		_basis = entry["basis"]

# -- The three buttons ------------------------------------------------------------

## Dismiss: removes the row at this width and changes nothing else.
func dismiss() -> void:
	var s := state()
	if s.is_empty():
		return
	_dismissed = s["new"]
	refresh()

## Scale with the map: every mark times `new / old` about the top-left edge, as one undo step. Returns whether it
## scaled. On a refusal (a stroke that would shrink below the minimum radius, a busy engine) nothing changes and the
## status line says why.
func scale_with_map() -> bool:
	var s := state()
	if s.is_empty() or cp.bridge == null or cp.bridge.generating:
		return false
	var before: String = cp.bridge.colour_paint_document()
	var r: Dictionary = cp.bridge.colour_paint_scale(float(s["new"]) / float(s["old"]))
	if not bool(r.get("ok", false)):
		if cp.app != null and cp.app.has_method("set_status"):
			cp.app.set_status("hint", "Not scaled: %s" % String(r.get("error", "the engine is busy")), "warn")
		return false
	cp._push_undo(before, "Scale paint with the map")
	(cp._undo.back() as Dictionary)["basis"] = s["old"]
	_basis = s["new"]
	cp._after_edit()
	return true

## Show marks: outlines every mark (every layer, hidden ones included) once, for `OUTLINE_SECONDS`.
func show_marks() -> void:
	var ov = cp.app.viewport.tool_overlay if cp.app != null and cp.app.viewport != null else null
	var kpc: float = cp.km_per_cell()
	if ov == null or kpc <= 0.0 or cp.bridge == null:
		return
	ov.set_mark_outlines(outlines(JSON.parse_string(cp.bridge.colour_paint_document()), kpc))
	_outline_gen += 1
	var gen := _outline_gen
	cp.app.get_tree().create_timer(OUTLINE_SECONDS).timeout.connect(func():
		if gen == _outline_gen and is_instance_valid(ov):
			ov.set_mark_outlines([]))

## Every mark's outline in grid cells from a parsed document (version 1's `marks`, version 2's `layers`): a stroke's
## centre line, an area's ring closed. `[]` for anything else, never a guess.
static func outlines(doc: Variant, kpc: float) -> Array:
	var out: Array = []
	if not (doc is Dictionary) or kpc <= 0.0:
		return out
	var marks: Array = []
	if (doc as Dictionary).has("layers"):
		for l in (doc as Dictionary)["layers"]:
			marks.append_array((l as Dictionary).get("marks", []))
	else:
		marks = (doc as Dictionary).get("marks", [])
	for m in marks:
		var shape: Dictionary = (m as Dictionary).get("shape", {})
		var src: Array = shape.get("points_km", shape.get("ring_km", []))
		var pts := PackedVector2Array()
		for p in src:
			pts.append(Vector2(float(p[0]), float(p[1])) / kpc)
		if String(shape.get("kind", "")) == "area" and pts.size() > 0:
			pts.append(pts[0])
		if pts.size() > 0:
			out.append(pts)
	return out
