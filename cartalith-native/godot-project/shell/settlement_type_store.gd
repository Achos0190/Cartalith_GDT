extends RefCounted
class_name SettlementTypeStore

## `lazy-riding-piglet.md` Batch D -- the Settlement Editor's artboard 1f,
## "Settlement types" (`design/settlement-editor-2026-09-21/Cartalith
## Settlement Editor.dc.html`, `id="1f"`).
##
## A type is a named bundle of fields a settlement already has -- kind,
## specialisation, traits, walls, age policy, name-pool source -- that the
## settlement-drop tool applies on drop, plus a per-faction default. It is
## **not** a new kind: the six kinds are a fixed enum written into save
## files, so a seventh would be a format migration; a type only ever pins
## one of the six.
##
## ## Why a static store, not a `WorldGen` field
##
## Nothing in Rust needs to *read* a type -- only this GDScript store needs
## to apply one's field bundle at creation time, through the exact
## `civ_edit_settlement`/`civ_settlement_toggle_trait` calls
## `place_editor_window.gd` already uses to edit a live settlement. So this
## follows `trade_store.gd`'s own shape (`extends RefCounted`,
## `class_name`, `static var`/`static func`) rather than becoming an engine
## document schema: one process-wide store every window and tool reads and
## writes through its class name, no autoload registration needed.
##
## ## Storage: `library/settlement_types.json`, caller-owned
##
## Registered in `cartalith-io/src/project.rs`'s `DOCUMENT_SLOTS` (Batch D)
## and **absent** from `cartalith-godot/src/project_bridge.rs`'s
## `ENGINE_OWNED_SLOTS` -- the same caller-owned shape
## `annotations/measurements.json` already has, for the identical reason: a
## settlement type is authored data with no engine model behind it, so the
## engine would have nothing to write even if it tried. `app.gd`'s
## `_project_documents()`/`_restore_project_documents()` are the only two
## callers of `document()`/`restore_document()` below, merging this slot's
## text into the same dictionary every other caller-owned document already
## goes through.
##
## ## What is deliberately absent
##
## **Placement rules** (terrain preference, minimum spacing, river/coast
## requirements): the canvas's own stated reason, carried here too --
## placement reads suitability and faction, not a type, and there is no
## generation-side hook for "a type influences where generation puts a
## settlement." Not built: that hook does not exist. See
## `settlement_types_window.gd`'s own Placement rules panel for the same
## disclosure on screen.
##
## **Name-pool override.** `civ_reroll_settlement_name` keys a re-roll off
## the settlement's own faction's stored `culture` field
## (`cartalith_civ::civ_faction_culture` over the roster), not off any
## per-type pool -- and no bound function draws from a pool it is handed,
## which `culture_profiles_window.gd`'s own doc comment states for why it
## cannot fabricate an arbitrary culture's sample. So "Inherit the faction's
## culture" is the only functional value; a type's `name_pool` field is
## stored (for a future generation-side hook, if one is ever built) and the
## picker offers no other option rather than drawing a control that changes
## nothing.

## `place_editor_window.gd::KIND_ORDER`, duplicated rather than referenced --
## a type's base-kind selector and a settlement's own Classification picker
## are the same fixed vocabulary, and `PlaceEditorWindow` is not something
## this file should depend on to get six strings.
const KIND_ORDER := ["metropolis", "capital", "city", "town", "village", "hamlet"]

static var _types: Array = []

## faction id (int) -> type id (String), `""` meaning no default (today's
## unmodified behaviour -- the canvas's own "None" column entry).
static var _faction_defaults: Dictionary = {}

static var _next_id := 1

static func types() -> Array:
	return _types

static func get_type(id: String) -> Dictionary:
	if id == "":
		return {}
	for t in _types:
		if String((t as Dictionary).get("id", "")) == id:
			return t
	return {}

## A fresh type at the recorded defaults -- kind `town` (the reference's own
## most common tier), no specialisation, no traits, Auto walls, Auto age,
## inherited name pool. Returns the new id.
static func new_type(name: String = "New type") -> String:
	var id := "t%d" % _next_id
	_next_id += 1
	_types.append({
		"id": id,
		"name": name,
		"kind": "town",
		"specialisation": "none",
		"traits": [],
		"walls": -1,
		"age_mode": "auto",
		"age_years": 100,
		"name_pool": "inherit",
		"usage_count": 0,
	})
	return id

## Returns the new id, or `""` if `id` does not exist.
static func duplicate_type(id: String) -> String:
	var src := get_type(id)
	if src.is_empty():
		return ""
	var new_id := "t%d" % _next_id
	_next_id += 1
	var copy: Dictionary = src.duplicate(true)
	copy["id"] = new_id
	copy["name"] = "%s copy" % String(src.get("name", "Type"))
	copy["usage_count"] = 0
	_types.append(copy)
	return new_id

## Also clears the id from every faction default that pointed at it -- a
## deleted type cannot stay a live default, silently applying nothing on the
## next drop (the same "faction left on None" behaviour, arrived at rather
## than chosen).
static func delete_type(id: String) -> void:
	for i in range(_types.size() - 1, -1, -1):
		if String((_types[i] as Dictionary).get("id", "")) == id:
			_types.remove_at(i)
	for fid in _faction_defaults.keys():
		if String(_faction_defaults[fid]) == id:
			_faction_defaults[fid] = ""

static func set_field(id: String, key: String, value) -> void:
	for t in _types:
		if String((t as Dictionary).get("id", "")) == id:
			(t as Dictionary)[key] = value
			return

static func toggle_trait(id: String, key: String) -> void:
	for t in _types:
		var d: Dictionary = t
		if String(d.get("id", "")) == id:
			var arr: Array = d.get("traits", [])
			if arr.has(key):
				arr.erase(key)
			else:
				arr.append(key)
			d["traits"] = arr
			return

static func faction_default(faction_id: int) -> String:
	return String(_faction_defaults.get(faction_id, ""))

static func set_faction_default(faction_id: int, type_id: String) -> void:
	_faction_defaults[faction_id] = type_id

static func mark_used(id: String) -> void:
	for t in _types:
		var d: Dictionary = t
		if String(d.get("id", "")) == id:
			d["usage_count"] = int(d.get("usage_count", 0)) + 1
			return

## The world its faction ids belonged to is gone -- called from
## `restore_document()`'s empty-document branch, the same "this project has
## none" clear `right_dock.gd::clear_measurements()` performs for its own
## slot.
static func clear() -> void:
	_types = []
	_faction_defaults = {}
	_next_id = 1

## Applies `faction_id`'s default type's field bundle to the just-dropped
## settlement at `index`, via the exact calls `place_editor_window.gd`'s own
## `_apply()`/traits chip already use to edit a live settlement --
## additive, and a **no-op** (no call made at all) when the faction has no
## default, matching `civilization_workspace.gd::_settlement_click`'s own
## contract that a faction left on None behaves byte-identically to today.
static func apply_default_to_settlement(bridge, faction_id: int, index: int) -> void:
	var tid := faction_default(faction_id)
	if tid == "":
		return
	var t := get_type(tid)
	if t.is_empty():
		return
	var fields := {
		"kind": String(t.get("kind", "town")),
		"specialisation": String(t.get("specialisation", "none")),
		"walls": int(t.get("walls", -1)),
		"age": int(t.get("age_years", 100)) if String(t.get("age_mode", "auto")) == "fixed" else -1,
	}
	if not bridge.civ_edit_settlement(index, fields):
		return
	for key in (t.get("traits", []) as Array):
		bridge.civ_settlement_toggle_trait(index, String(key))
	mark_used(tid)

## This store's half of the project file, as JSON **text**, or `""` when
## there is nothing to write -- the same empty-string-not-empty-document
## contract every other caller-owned slot in this shell states for itself
## (`right_dock.gd::measurements_document()`'s own doc comment).
static func document() -> String:
	if _types.is_empty() and _faction_defaults.is_empty():
		return ""
	var types_out: Array = []
	for t in _types:
		var d: Dictionary = t
		types_out.append({
			"id": String(d.get("id", "")),
			"name": String(d.get("name", "")),
			"kind": String(d.get("kind", "town")),
			"specialisation": String(d.get("specialisation", "none")),
			"traits": (d.get("traits", []) as Array).duplicate(),
			"walls": int(d.get("walls", -1)),
			"age_mode": String(d.get("age_mode", "auto")),
			"age_years": int(d.get("age_years", 100)),
			"name_pool": String(d.get("name_pool", "inherit")),
			"usage_count": int(d.get("usage_count", 0)),
		})
	## Keys become strings under `JSON.stringify` regardless -- written as
	## strings explicitly so the round trip through `restore_document()` is
	## the documented shape rather than an implicit Dictionary-key coercion.
	var defaults_out: Dictionary = {}
	for fid in _faction_defaults.keys():
		var tid := String(_faction_defaults[fid])
		if tid != "":
			defaults_out[str(int(fid))] = tid
	return JSON.stringify({"types": types_out, "faction_defaults": defaults_out})

## The inverse. Called unconditionally from `app.gd::_restore_project_documents()`,
## not only when the slot is present -- an **absent** slot is a genuine "this
## project has no types" and must clear the outgoing project's, the same
## data-loss guard `right_dock.gd::restore_measurements_document()`'s own doc
## comment states for its slot. A document this build cannot parse is treated
## the same as an absent one: cleared rather than left holding a stale
## mid-parse state.
static func restore_document(text: String) -> void:
	if text.strip_edges() == "":
		clear()
		return
	var parsed = JSON.parse_string(text)
	if not (parsed is Dictionary):
		clear()
		return
	var doc: Dictionary = parsed
	var arr = doc.get("types", [])
	var new_types: Array = []
	var max_id := 0
	if arr is Array:
		for raw in (arr as Array):
			if not (raw is Dictionary):
				continue
			var d: Dictionary = raw
			var id := String(d.get("id", ""))
			if id == "":
				continue
			var traits_raw = d.get("traits", [])
			var traits: Array = []
			if traits_raw is Array:
				for tv in (traits_raw as Array):
					traits.append(String(tv))
			new_types.append({
				"id": id,
				"name": String(d.get("name", "Type")),
				"kind": String(d.get("kind", "town")),
				"specialisation": String(d.get("specialisation", "none")),
				"traits": traits,
				"walls": int(d.get("walls", -1)),
				"age_mode": String(d.get("age_mode", "auto")),
				"age_years": int(d.get("age_years", 100)),
				"name_pool": String(d.get("name_pool", "inherit")),
				"usage_count": int(d.get("usage_count", 0)),
			})
			if id.begins_with("t"):
				max_id = maxi(max_id, int(id.substr(1)))
	_types = new_types
	var fd = doc.get("faction_defaults", {})
	var new_defaults: Dictionary = {}
	if fd is Dictionary:
		for k in (fd as Dictionary).keys():
			new_defaults[int(String(k))] = String((fd as Dictionary)[k])
	_faction_defaults = new_defaults
	_next_id = max_id + 1
