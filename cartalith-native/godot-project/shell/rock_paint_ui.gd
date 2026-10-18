## GF-8 (`GEOLOGY_FIRST_SCOPE.md` §4.13, owner Ruling BJ): the shell's shared
## vocabulary for the **Rock** paint target, so the four surfaces that draw it --
## the tool bar's rows (desktop and tablet), the WORLD dock's Biome paint panel,
## the right dock's paint context and the phone's Paint column -- say the same
## thing in the same words and cannot drift.
##
## What it is: pure functions over the engine's own answers
## (`EngineBridge.rock_paint_status()`, `get_paint_palette("rock_*")`, the
## `rock` key of `paint_commit()`). It owns no state and never calls the
## engine itself; every caller passes what it read.
##
## What it must never do: invent a value the engine did not send (a missing
## key is a dash with its reason, `MISTAKES.md`), or describe Rock as saved --
## the geology column is not in the save format yet (GF-8's third slice).
##
## Preloaded, not a `class_name`: a new `class_name` needs an editor import
## pass, which strips `project.godot`'s `;` comments (`CLAUDE.md`).
extends RefCounted

## The Rock layer a fresh Rock selection opens on (§4.13's first layer, Surface).
const DEFAULT_LAYER := "rock_top"

## §4.13's reason for the disabled Erase, verbatim in its first clause.
const ERASE_DISABLED := "Erase (⇧) is off for Rock: a committed rock is reverted with Undo; the column has no unpainted state to fall back to. Discard still drops an uncommitted draft."

## The short form of the save caveat for a row too narrow for the sentence; the
## full sentence (`save_note`) is its tooltip.
const SAVE_SHORT := "not saved with the project yet"

## Whether `layer` is one of the three Rock layer keys.
static func is_rock(layer: String) -> bool:
	return layer == "rock_top" or layer == "rock_sub" or layer == "rock_cap"

## The user-facing name of a Rock layer from the engine's own `layers` list
## (Surface / Beneath / Cap thickness), or the key when the engine did not name it.
static func layer_label(st: Dictionary, key: String) -> String:
	for l in st.get("layers", []):
		var d: Dictionary = l
		if String(d.get("key", "")) == key:
			return String(d.get("label", key))
	return key

## The three Rock layer keys, in the engine's order; `[]` when the engine sent none.
static func layer_keys(st: Dictionary) -> PackedStringArray:
	var out := PackedStringArray()
	for l in st.get("layers", []):
		out.append(String((l as Dictionary).get("key", "")))
	return out

## One cap class in metres (`CAP_CLASS_M`, 10 m), from the engine.
static func cap_class_m(st: Dictionary) -> float:
	return float(st.get("cap_class_m", 10.0))

## The thickness a cap value means, in metres: value `k` is `k · 10` m (§4.13).
static func cap_metres(st: Dictionary, value: int) -> float:
	return float(clampi(value, 1, 255)) * cap_class_m(st)

## The cap value for a thickness in metres: the nearest class, `1 ..= 255`.
static func cap_value(st: Dictionary, metres: float) -> int:
	return clampi(int(round(metres / cap_class_m(st))), 1, 255)

## The tooltip a Rock target carries when it is enabled: what painting does, and
## the save caveat.
static func enabled_tip(st: Dictionary) -> String:
	return ("Paint rock into the geology column (GF-8). Commit bakes it and re-erodes the "
		+ "painted cells' box plus %d cells each side, so softer rock wears lower and harder "
		+ "rock stands up; Undo reverts both the rock and the erosion. %s") % [
			int(st.get("pad_cells", 16)), String(st.get("save_note", ""))]

## The status-bar line for a commit's `rock` result, and its colour token:
## `["", ""]` when no Rock dab was pending (the key is absent). Names the bake,
## the re-erosion window and how much it moved, every skip count the engine
## reported, and what it did not re-run -- the engine reports these instead of
## silently dropping them, so the shell says them.
static func commit_status(summary: Dictionary) -> Array:
	if not summary.has("rock"):
		return ["", ""]
	var r: Dictionary = summary["rock"]
	if not bool(r.get("ok", false)):
		return ["Rock not committed: %s The Rock draft is kept." % String(r.get("reason", "the engine refused it.")), "warn"]
	var parts := PackedStringArray()
	parts.append("rock painted: %d cell%s baked" % [int(r.get("cells_baked", 0)), "" if int(r.get("cells_baked", 0)) == 1 else "s"])
	var e: Dictionary = r.get("reerode", {})
	if bool(e.get("ran", false)):
		var w: Dictionary = e.get("window", {})
		var ring := ("ring step %.3f %% of relief" % float(e["ring_step_pct"])) if e.has("ring_step_pct") else "ring step not measured (flat window)"
		parts.append("re-eroded %d x %d cells at (%d, %d), %d moved, %s" % [
			int(w.get("w", 0)), int(w.get("h", 0)), int(w.get("x", 0)), int(w.get("y", 0)),
			int(e.get("cells_moved", 0)), ring])
	else:
		parts.append("not re-eroded: %s" % String(e.get("reason", "")))
	var cap_skip := int(r.get("cap_skipped", 0))
	if cap_skip > 0:
		parts.append("%d cap-thickness cell%s skipped (no rock beneath)" % [cap_skip, "" if cap_skip == 1 else "s"])
	var sub_skip := int(r.get("sub_skipped", 0))
	if sub_skip > 0:
		parts.append("%d rock-beneath cell%s skipped (single-layer cell with no cap thickness given)" % [sub_skip, "" if sub_skip == 1 else "s"])
	var nr: PackedStringArray = e.get("not_rerun", PackedStringArray())
	if nr.size() > 0:
		parts.append("not re-run: " + "; ".join(nr))
	parts.append(SAVE_SHORT)
	return [" -- ".join(parts), "text_ghost"]
