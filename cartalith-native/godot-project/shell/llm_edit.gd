extends RefCounted
## LLM-7 (`LLM_ASSISTANT_SCOPE.md` s.5c; owner Rulings DD Q10, DI): the shell half of the Assistant's
## EDIT ACTIONS -- the proposal card with its preview, and the one place a validated edit is applied.
## Like `llm_view.gd` this file holds no state; the Assistant window owns the card and calls in.
##
## ## The rules this file keeps
##
## * **A proposal is data until the click.** `draw_card` reads the validated rows Rust produced
##   (`edit_verdict_dict`: operation, numbers and ramp, never the model's text as a command) and asks
##   the ENGINE for a preview computed on a copy of the region (`region_edit_preview`). The preview is
##   the app's own arithmetic over the validated operations, never the model's (owner question E5's
##   default), and nothing has changed while the card is on screen.
## * **Apply is one undo step.** `apply` opens an `undo_group.gd` group, makes the engine call inside
##   it (the engine records one `height` ledger row, which the group collects) and commits; any
##   failure rolls the group back and says why. The group's `height_room()` is asked first, so the
##   engine's bounded undo stack cannot evict the group's own step.
## * **Never regenerates, erodes or touches settings.** The engine call re-runs hydrology and climate
##   for the touched tiles, exactly as a sculpt commit does, and nothing else.
## * **The region must still be the region.** The proposal names a region id; Apply re-reads the
##   region from the engine and refuses when it is not the same id (the user painted more since).
## * **Nothing is applied that the preview could not show.** Apply is disabled, with the reason beside
##   it, when the preview failed, when committed colour paint lies under the region (the engine would
##   refuse: Ruling AS clears paint under a sculpt and the undo cannot restore it), while a world
##   generates, or on a finalised world.
##
## No `class_name`: preloaded by the window, like `llm_view.gd`.

const _REGION := preload("res://shell/llm_region.gd")

## Thumbnail display scale per form. A labelled judgement from the first desktop capture (1.0 was too small to see
## a ridge when the Assistant was a 560 px window): the engine's 96 px thumbnails shown at 1.5x, so two side by side
## are about 300 px, inside the 393 dp phone sheet. **Desktop and tablet now dock the Assistant in a 304 to 313 px
## Ask tab whose cards have about 245 px**, and 300 px of thumbnails widened the whole dock past the display's right
## edge (tablet 1700x1000, 2026-11-17: the Ask scroller 378 px wide, Apply and the year field outside the screen), so
## those two forms show them at 1.0: 96 + 10 + 96 = 202 px. The row is also a flow, so a still narrower card wraps
## rather than widening anything.
const THUMB_SCALE_DESKTOP := 1.0
const THUMB_SCALE_TOUCH := 1.5
const THUMB_SCALE_TABLET := 1.0


## The validated rows as the engine's operation dictionaries (`{operation, amount_m?, strength?,
## passes?, falloff, softness}`), exactly as Rust validated them.
static func ops_of(valid: Array, digest := "") -> Array:
	var ops: Array = []
	for r in valid:
		var d: Dictionary = r
		if is_paint(d):
			continue
		var o := {"operation": String(d["operation"]), "falloff": String(d["falloff"]), "softness": float(d["softness"])}
		for k in ["amount_m", "strength", "passes"]:
			if d.has(k):
				o[k] = d[k]
		## Ruling FW item 1: a stamp carries the digest of the path the preview was made from, and the engine
		## refuses an Apply whose recomputed path differs.
		if digest != "" and STAMPS.has(String(d["operation"])):
			o["path_digest"] = digest
		ops.append(o)
	return ops


## The five stamps (`cartalith_llm::edit_actions::STAMPS`), by name.
const STAMPS := ["ridge", "cliff", "canyon", "mesa", "volcano"]


## True for the colour operation (Ruling FW item 2): it recolours, moves no ground and has no ramp.
static func is_paint(d: Dictionary) -> bool:
	return String(d.get("operation", "")) == "paint"


## The paint row's `{colour, blend, opacity}` (the validator allows one), or {} when the proposal has none.
static func paint_of(valid: Array) -> Dictionary:
	for r in valid:
		var d: Dictionary = r
		if is_paint(d):
			return {"colour": String(d["colour"]), "blend": String(d["blend"]), "opacity": float(d["opacity"])}
	return {}


## The preview for a verdict's valid rows over the CURRENT region. `{ok: false, reason}` when there is
## nothing to preview or the engine refuses.
static func preview(bridge, valid: Array) -> Dictionary:
	if valid.is_empty():
		return {"ok": false, "reason": "There is nothing to preview."}
	var hops := ops_of(valid)
	var out: Dictionary = {"ok": true, "height_ops": 0}
	if not hops.is_empty():
		out = bridge.region_edit_preview(_REGION.spec(), hops)
		out["height_ops"] = hops.size()
	var pv := paint_of(valid)
	if not pv.is_empty() and bool(out.get("ok", false)):
		var pp: Dictionary = bridge.region_paint_preview(_REGION.spec(), pv)
		if bool(pp.get("ok", false)):
			out["paint"] = pp
			out["paint_op"] = pv
		else:
			out = {"ok": false, "reason": String(pp.get("reason", "the colour could not be previewed")), "height_ops": hops.size()}
	return out


## One figure in metres for the card ("-12 m" style signs are explicit), whole metres.
static func metres(v: float) -> String:
	return "%d m" % int(roundf(v))


## The before/after lines of a preview, as plain text. Absent figures are not printed.
static func preview_lines(p: Dictionary) -> PackedStringArray:
	var lines := PackedStringArray()
	if int(p.get("height_ops", 1)) == 0:
		return lines
	var b: Dictionary = p.get("before", {})
	var a: Dictionary = p.get("after", {})
	if not b.is_empty() and not a.is_empty():
		lines.append("Heights in the region (metres above sea): lowest %s to %s, mean %s to %s, highest %s to %s." % [
			metres(float(b["min_m"])), metres(float(a["min_m"])), metres(float(b["mean_m"])), metres(float(a["mean_m"])),
			metres(float(b["max_m"])), metres(float(a["max_m"]))])
	lines.append("%d cell%s change; largest rise %s, largest fall %s." % [
		int(p["changed"]), "" if int(p["changed"]) == 1 else "s", metres(float(p["max_up_m"])), metres(float(p["max_down_m"]))])
	if int(p.get("became_sea", 0)) > 0:
		lines.append("%d land cell%s would go under the sea." % [int(p["became_sea"]), "" if int(p["became_sea"]) == 1 else "s"])
	if int(p.get("became_land", 0)) > 0:
		lines.append("%d sea cell%s would become land." % [int(p["became_land"]), "" if int(p["became_land"]) == 1 else "s"])
	if int(p.get("capped", 0)) > 0:
		lines.append("%d cell%s would be stopped at the world's ceiling, so they rise less than asked." % [int(p["capped"]), "" if int(p["capped"]) == 1 else "s"])
	if int(p.get("limited", 0)) > 0:
		lines.append("%d cell%s would have moved more than 600 m, so %s held to 600 m of change." % [int(p["limited"]), "" if int(p["limited"]) == 1 else "s", "it is" if int(p["limited"]) == 1 else "they are"])
	if int(p.get("floored", 0)) > 0:
		lines.append("%d cell%s would be stopped at the lowest level, so they fall less than asked." % [int(p["floored"]), "" if int(p["floored"]) == 1 else "s"])
	return lines


## Why a card's Apply cannot run now, or "". Cheap (no engine read besides the generating flag).
static func blocked(bridge, card: Dictionary) -> String:
	if bridge.generating:
		return "A world is generating; wait for it to finish."
	if bridge.is_finalized():
		return "The world is finalised; unfinalise it to edit its terrain."
	var p: Dictionary = card.get("preview", {})
	if not bool(p.get("ok", false)):
		return "No preview could be made: %s" % String(p.get("reason", "unknown reason"))
	if int(p.get("paint_cells", 0)) > 0:
		return "%d painted cells lie under the region. An edit would clear them and Undo could not bring them back; clear the paint there first." % int(p["paint_cells"])
	## Ruling FW item 2: the colour is written by the colour-paint tool. (E7 above is about the world's own
	## paint layer under a HEIGHT edit; a colour mark never conflicts with it.)
	if p.has("paint_op") and not bool(bridge.colour_paint_api):
		return "This build has no colour-paint layer, so the colour cannot be applied."
	return ""


## An RGB8 thumbnail from the engine's bytes, or null when the figures do not fit.
static func thumb_texture(w: int, h: int, bytes: PackedByteArray) -> Texture2D:
	if w <= 0 or h <= 0 or bytes.size() != w * h * 3:
		return null
	return ImageTexture.create_from_image(Image.create_from_data(w, h, false, Image.FORMAT_RGB8, bytes))


## Applies the validated rows as ONE undo step. `ug` is `app.undo_groups`. Returns `{ok, reason?,
## group_id?, changed?}`; on `ok == false` the group was rolled back (or never opened) and nothing
## changed. The ONLY writer of terrain in the Assistant.
static func apply(app, bridge, ug, info: Dictionary, valid: Array) -> Dictionary:
	if valid.is_empty():
		return {"ok": false, "reason": "There is nothing to apply."}
	var hops := ops_of(valid, String(info.get("path_digest", "")))
	var pv := paint_of(valid)
	## The region must still be the one the model was shown.
	var now: Dictionary = _REGION.facts(bridge)
	var rid := String(now.get("region", {}).get("id", ""))
	if rid == "" or rid != String(info.get("region_id", "")):
		return {"ok": false, "reason": "The region changed since this proposal (or could not be read). Nothing was applied; ask again."}
	var refused: String = ug.begin("Assistant: %s" % String(info.get("summary", "terrain edit")).left(60))
	if refused != "":
		return {"ok": false, "reason": refused}
	var r: Dictionary = {"ok": true, "changed": 0}
	if not hops.is_empty():
		if ug.height_room() < 1:
			ug.abort("the undo history has no room for another terrain step")
			return {"ok": false, "reason": "The undo history has no room for another terrain step."}
		r = bridge.region_edit_apply(_REGION.spec(), hops, "Assistant edit")
		if not bool(r.get("ok", false)):
			ug.abort(String(r.get("reason", "the edit was refused")))
			return {"ok": false, "reason": String(r.get("reason", "The edit was refused."))}
	## Ruling FW item 2: the colour, as the colour-paint tool's own one step inside the group (a failure
	## here rolls the height step back with the group).
	var pieces := 0
	if not pv.is_empty():
		var tool = ug.paint_tool()
		if tool == null:
			ug.abort("the colour-paint tool is not available")
			return {"ok": false, "reason": "The colour-paint tool is not available, so nothing was applied."}
		var m: Dictionary = bridge.region_paint_marks(_REGION.spec(), pv)
		if not bool(m.get("ok", false)):
			ug.abort(String(m.get("reason", "the colour was refused")))
			return {"ok": false, "reason": String(m.get("reason", "The colour was refused."))}
		var marks: Array = []
		for js in m["marks"]:
			var mk: Dictionary = JSON.parse_string(String(js))
			## Godot parses every JSON number as a float, and the document wants whole 0-255 channels.
			var rgb: Array = mk["rgb"]
			mk["rgb"] = [int(rgb[0]), int(rgb[1]), int(rgb[2])]
			marks.append(mk)
		if not bool(tool.commit_marks(marks, "Assistant paint")):
			ug.abort("the colour-paint tool refused the marks")
			return {"ok": false, "reason": "The colour-paint layer refused the colour (its limits), so nothing was applied."}
		pieces = int(m.get("pieces", 0))
	var c: Dictionary = ug.commit()
	if not bool(c.get("ok", false)):
		return {"ok": false, "reason": String(c.get("reason", "The undo step could not be made."))}
	## Show the new ground the way the sculpt commit does: a direct texture write (not a refresh,
	## which would re-fit the camera) and a fresh deep-zoom tile set.
	var vp = app.get("viewport")
	if vp != null:
		vp.map_view.texture = bridge.color_texture()
		vp.invalidate_lod_tiles()
		vp.set_preview_texture(null)
	return {"ok": true, "group_id": int(c.get("id", 0)), "changed": int(r.get("changed", 0)), "painted": not pv.is_empty(), "paint_pieces": pieces}


## Two thumbnails side by side (Before, After) from a preview dictionary's `thumb_*` keys, named
## `edit_thumb_<prefix>before` and `..after`. Returns whether they were drawn (the figures must fit).
static func _draw_pair(col: Control, p: Dictionary, prefix: String) -> bool:
	var tex_b := thumb_texture(int(p.get("thumb_w", 0)), int(p.get("thumb_h", 0)), p.get("thumb_before", PackedByteArray()))
	var tex_a := thumb_texture(int(p.get("thumb_w", 0)), int(p.get("thumb_h", 0)), p.get("thumb_after", PackedByteArray()))
	if tex_b == null or tex_a == null:
		return false
	var pics := HFlowContainer.new()
	pics.add_theme_constant_override("h_separation", 10)
	pics.add_theme_constant_override("v_separation", 4)
	col.add_child(pics)
	var sc := THUMB_SCALE_TOUCH if DccTheme.is_phone() else (THUMB_SCALE_TABLET if DccTheme.is_tablet() else THUMB_SCALE_DESKTOP)
	for pair in [["Before", tex_b], ["After", tex_a]]:
		var cell := VBoxContainer.new()
		pics.add_child(cell)
		cell.add_child(DccTheme.label(String(pair[0]), "text_dim", DccTheme.FS_MICRO))
		var tr := TextureRect.new()
		tr.texture = pair[1]
		tr.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		tr.stretch_mode = TextureRect.STRETCH_KEEP
		tr.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		tr.custom_minimum_size = Vector2(float(tex_b.get_width()) * sc, float(tex_b.get_height()) * sc)
		tr.name = "edit_thumb_%s%s" % [prefix, String(pair[0]).to_lower()]
		cell.add_child(tr)
	return true


## Draws the EDIT card into the window's message list and registers it in `w.cards` (kind `edit`).
## `w` is the Assistant window (its `_card_text`, `_note_label`, `_button`, `_msgs`, `cards`,
## `_dismiss_card` and `_apply_edit_card` are used, so the card behaves as the other two kinds do).
## The preview is computed here, once, from the validated rows; it is the app's, not the model's.
static func draw_card(w, v: Dictionary) -> void:
	var valid: Array = v.get("valid", [])
	var box := PanelContainer.new()
	box.add_theme_stylebox_override("panel", DccWidgets.box("warn", "sunken", 8, 6))
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 4)
	box.add_child(col)
	var chip := DccTheme.label("MODEL PROPOSAL · TERRAIN EDIT" if paint_of(valid).is_empty() else "MODEL PROPOSAL · REGION EDIT", "warn", DccTheme.FS_MICRO)
	chip.tooltip_text = "Proposed by a language model. Nothing has changed; the preview below is computed by the app. Press Apply or Dismiss."
	col.add_child(chip)
	if v.has("summary"):
		w._card_text(col, "\u201c%s\u201d" % String(v["summary"]), "text_secondary")
	var p: Dictionary = {}
	if not valid.is_empty():
		p = preview(w.bridge, valid)
		## Ruling FW item 1: Apply runs only the painted path this preview was made from.
		v["path_digest"] = String(p.get("path_digest", ""))
	var n := 0
	for c in valid:
		var cd: Dictionary = c
		n += 1
		var line := "%d. %s" % [n, String(cd["text"])]
		if bool(cd.get("defaulted", false)):
			line += "   (ramp shape not given, the default is used)"
		w._card_text(col, line, "text")
		if cd.has("reason"):
			w._card_text(col, "    \u201c%s\u201d  (model's words)" % String(cd["reason"]), "text_dim")
	var rejected: Array = v.get("rejected", [])
	if v.has("structural") or not rejected.is_empty():
		w._card_text(col, "Not applied:", "text_dim")
		if v.has("structural"):
			w._card_text(col, "    %s: %s" % [String(v.get("structural_rule", "")), String(v["structural"])], "text_ghost")
		for r in rejected:
			var rd: Dictionary = r
			w._card_text(col, "    %s  %s: %s" % [String(rd.get("operation", "(no operation)")), String(rd["rule"]), String(rd["why"])], "text_ghost")
	if valid.is_empty():
		w._card_text(col, "Nothing could be applied.", "warn")
	elif bool(p.get("ok", false)):
		w._card_text(col, "Preview (worked out by the app on a copy of the region; nothing has changed):", "text_secondary")
		for line in preview_lines(p):
			w._card_text(col, "    " + String(line), "text")
		if _draw_pair(col, p, "") and int(p.get("height_ops", 1)) > 0:
			w._card_text(col, "Both pictures use one height scale; cells outside the region are dimmed, blue is below sea level.", "text_ghost")
		if p.has("paint"):
			var pp: Dictionary = p["paint"]
			var po: Dictionary = p["paint_op"]
			w._card_text(col, "Colour: %s, %s blend, %d%% opacity over the region. %d land cell%s in %d piece%s change colour; water inside it keeps its colour, and nothing outside the region is touched." % [
				String(po["colour"]), String(po["blend"]), int(roundf(float(po["opacity"]) * 100.0)), int(pp["cells_recoloured"]),
				"" if int(pp["cells_recoloured"]) == 1 else "s", int(pp["pieces"]), "" if int(pp["pieces"]) == 1 else "s"], "text")
			if _draw_pair(col, pp, "paint_"):
				w._card_text(col, "The colour pictures show the blend on a grey height shading (the real map's lighting and any paint already there are not drawn); cells outside the region are dimmed.", "text_ghost")
		var does: Array = []
		if int(p.get("height_ops", 1)) > 0:
			does.append("edits the terrain inside this region only and re-runs rivers and climate for the area, as a sculpt commit does")
		if p.has("paint"):
			does.append("adds the colour as a colour-paint layer mark inside this region only")
		w._card_text(col, "Applying %s. It is one undo step (Edit \u25b8 Undo). It does not regenerate, erode or change settings." % " and ".join(does), "text_secondary")
	else:
		w._card_text(col, "No preview could be made: %s" % String(p.get("reason", "unknown reason")), "warn")
	var reason_l: Label = w._note_label(col, "", "warn")
	reason_l.visible = false
	## PC-6: the window's own card row (a flow when docked in the Context panel, so it cannot widen it).
	var row: Container = w.card_row(col)
	var card := {"info": v, "box": box, "apply": null, "dismiss": null, "reason": reason_l, "state": "new", "kind": "edit", "group_id": 0, "preview": p}
	card["dismiss"] = w._button(row, "Dismiss", func(): w._dismiss_card(card))
	if not valid.is_empty():
		card["apply"] = w._button(row, "Apply %d edit%s" % [valid.size(), "" if valid.size() == 1 else "s"], func(): w._apply_edit_card(card), true)
	w._msgs.add_child(box)
	w.cards.append(card)
