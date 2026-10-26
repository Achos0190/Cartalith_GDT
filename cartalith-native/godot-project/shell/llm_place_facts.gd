extends RefCounted
class_name LlmPlaceFacts

## Gathers the world's own data for ONE place, and the Vault notes that bear on it, for the
## assistant's grounded prose (`LLM_ASSISTANT_SCOPE.md` "Grounded place prose", LLM-11).
##
## ## What this file is, and is not
##
## It is the only code that reads the engine and the Vault for this feature. It calls the engine's
## OWN getters, passes their results through (with unit-free plumbing only), and hands the lot to
## Rust (`LlmStream.place_neighbours`, `place_bundle`, `place_vault_plan`), which selects (the 25 km
## radius, the caps), renders and checks. It computes no world figure of its own. A getter that
## returns nothing leaves its key ABSENT; Rust then says "not available" with the reason. It never
## fills a gap with a default (`MISTAKES.md`: never encode "no value" as a plausible value).
##
## ## Rules this file keeps
##
## * **Read-only.** Every call is a `&self` getter or a Vault read. It writes nothing, regenerates
##   nothing, builds no Vault index and opens no socket. The Vault write path (`vault_write_file`
##   with `expect_hash`, only on the Save click) is unchanged and not here.
## * **Never reads a generating engine** (`bridge.generating`): the worker holds `WorldGen`.
## * **No map image** (Ruling DI P5): text only.
## * **The Vault part honours the user's setting.** `gather_vault` is called only when "Send: named
##   Vault notes" is ticked, and reads at most `max_reads` notes (Rust's number).
##
## The key names of `raw` are documented in `crates/cartalith-llm/src/place_facts.rs`.

## The keys copied from each `sample_cell` result. Anything else it returns (plate, lithology
## ids, controlling faction names, ...) is not sent: smaller, and no free text from the world.
const SAMPLE_KEYS := ["x", "y", "elevation_m", "slope_deg", "biome", "water", "temperature_c", "precipitation",
	"river_order", "volcano_kind", "volcano_age_years", "volcano_activity", "boundary", "boundary_type", "rock"]


## Convert engine values to plain JSON-able ones: packed arrays to Arrays, vectors to pairs, names to
## Strings, non-finite floats to null (absent), recursively.
static func _plain(v):
	match typeof(v):
		TYPE_DICTIONARY:
			var d := {}
			for k in v:
				d[String(k)] = _plain(v[k])
			return d
		TYPE_ARRAY, TYPE_PACKED_STRING_ARRAY, TYPE_PACKED_INT32_ARRAY, TYPE_PACKED_INT64_ARRAY, TYPE_PACKED_FLOAT32_ARRAY, TYPE_PACKED_FLOAT64_ARRAY:
			var a := []
			for x in v:
				a.append(_plain(x))
			return a
		TYPE_PACKED_VECTOR2_ARRAY:
			var a := []
			for p in v:
				a.append(p.x)
				a.append(p.y)
			return a
		TYPE_VECTOR2:
			return [v.x, v.y]
		TYPE_STRING_NAME:
			return String(v)
		TYPE_FLOAT:
			return v if is_finite(v) else null
		_:
			return v


static func _vocab(rows: Array) -> Dictionary:
	var out := {}
	for r in rows:
		var rd: Dictionary = r
		if rd.has("key") and rd.has("label"):
			out[String(rd["key"])] = String(rd["label"])
	return out


## The world block `{map_width_km, gw, gh}`, or `{}` when the engine reports no size.
static func _world(bridge: Object) -> Dictionary:
	var wg: Object = bridge.get("world_gen")
	if wg == null or not wg.has_method("get_map_width_km"):
		return {}
	## `wrap_x` is the "world" parameter: on a map that wraps east-west a town across the seam is near.
	return {"map_width_km": float(wg.call("get_map_width_km")), "gw": int(wg.call("get_width")), "gh": int(wg.call("get_height")),
		"wrap_x": bool(bridge.param_get("world"))}


## Rows of `settlements()` as the plain list Rust reads, in the engine's own index order.
static func _settlement_rows(all: Array) -> Array:
	var rows := []
	for i in all.size():
		var s: Dictionary = all[i]
		var r := {"i": i, "tid": int(s.get("tid", i)), "name": String(s.get("name", "")), "x": float(s.get("x", 0)), "y": float(s.get("y", 0)),
			"pop": int(s.get("population", 0)), "kind": String(s.get("kind", "")), "faction": int(s.get("faction", 0)),
			"capital": bool(s.get("capital", false)), "coastal": bool(s.get("coastal", false))}
		if s.has("religion"):
			r["religion"] = String(s["religion"])
		rows.append(r)
	return rows


## Gathers everything for settlement `index`. Returns `{ok, reason, raw, ms}`; `raw` is the Dictionary
## `LlmStream.place_bundle` takes (as JSON). `ok` is false only when nothing at all can be read (no
## world, a generation running, an index out of range, no Rust half).
static func gather(bridge: Object, stream: Object, index: int) -> Dictionary:
	var t0 := Time.get_ticks_msec()
	if stream == null or not stream.has_method("place_bundle"):
		return {"ok": false, "reason": "This build of the Cartalith library cannot gather place facts. Update it."}
	if not bool(bridge.get("has_world")):
		return {"ok": false, "reason": "No world is open, so there are no place facts."}
	if bool(bridge.get("generating")):
		return {"ok": false, "reason": "A world is generating; place facts are not available until it finishes."}
	var wg: Object = bridge.get("world_gen")
	var all: Array = bridge.settlements()
	if index < 0 or index >= all.size():
		return {"ok": false, "reason": "That place is not in the world's settlement list."}
	var world := _world(bridge)
	var s: Dictionary = all[index]
	var raw := {"world": world, "place_index": index, "settlements": _settlement_rows(all), "unavailable": {}}
	var unavailable: Dictionary = raw["unavailable"]
	var have_size := not world.is_empty() and float(world["map_width_km"]) > 0.0
	var ck := float(world["map_width_km"]) / float(world["gw"]) if have_size else 0.0

	# ---- which neighbours (Rust decides; this reads only those) ----
	var picks: Dictionary = stream.place_neighbours(JSON.stringify({"world": world, "place_index": index, "settlements": raw["settlements"]}))
	var nb_indices: Array = []
	for i in PackedInt32Array(picks.get("indices", PackedInt32Array())):
		nb_indices.append(int(i))

	# ---- the place ----
	var tid := int(s.get("tid", index))
	var place := {}
	var details: Dictionary = bridge.civ_settlement_details(index)
	if not details.is_empty():
		place["specialisation"] = String(details.get("specialisation", ""))
		place["traits"] = _plain(details.get("traits", []))
		if String(details.get("history", "")).strip_edges() != "":
			place["history"] = String(details["history"])
		place["age"] = int(details.get("age", -1))
		place["age_inferred"] = int(details.get("age_inferred", -1))
	if s.has("religion_shares"):
		place["religion_shares"] = _plain(s["religion_shares"])
	var bal: Array = bridge.trade_balances()
	if index < bal.size():
		place["trade"] = _plain(bal[index])
	var mil: Dictionary = bridge.civ_military_summary()
	var mil_by_index := {}
	for r in mil.get("settlements", []):
		var rd: Dictionary = r
		mil_by_index[int(rd.get("index", -1))] = rd
	if mil_by_index.has(index):
		place["military"] = _plain(mil_by_index[index])
	raw["military_factions"] = _plain(mil.get("factions", []))
	var own: Array = bridge.civ_settlement_ownership_periods(tid)
	if not own.is_empty():
		place["ownership"] = _plain(own)
	var conflicts: Array = []
	var fac_names := {}
	var factions: Array = bridge.get_factions()
	for f in factions:
		var fd: Dictionary = f
		fac_names[int(fd.get("id", 0))] = String(fd.get("name", ""))
	for c in bridge.conflicts_touching_settlement(tid):
		var cd: Dictionary = c
		var sides := []
		for sid in cd.get("sides", []):
			sides.append(String(fac_names.get(int(sid), "faction %d" % int(sid))))
		conflicts.append({"name": String(cd.get("name", "")), "kind": String(cd.get("kind", "")), "start_year": cd.get("start_year", null),
			"end_year": cd.get("end_year", null), "sides_names": sides, "outcome": String(cd.get("outcome", ""))})
	if not conflicts.is_empty():
		place["conflicts"] = conflicts
	var journeys: Array = []
	for j in bridge.civ_settlement_journey_passes(tid):
		var jd: Dictionary = j
		journeys.append({"name": String(jd.get("name", "")), "date": String(jd.get("date", "")) if jd.has("date") else ""})
	if not journeys.is_empty():
		place["journeys"] = journeys

	# ---- trade flows that touch the place (the engine caps its list at 4000 rows) ----
	var tf: Dictionary = bridge.civ_trade_flows()
	if tf.is_empty():
		unavailable["trade flows"] = "the engine returned no trade network for this world"
	else:
		var rows: Array = tf.get("flows", [])
		if rows.size() >= 4000:
			unavailable["trade flows (complete list)"] = "the engine returned its 4000-row cap, so this place's flows may be incomplete"
		var mine := []
		for fr in rows:
			var f: Dictionary = fr
			if int(f.get("from", -1)) == index:
				mine.append({"direction": "out", "good": String(f.get("good", "")), "other": String(f.get("to_name", "")), "mode": String(f.get("mode", "")), "volume": f.get("volume", null)})
			elif int(f.get("to", -1)) == index:
				mine.append({"direction": "in", "good": String(f.get("good", "")), "other": String(f.get("from_name", "")), "mode": String(f.get("mode", "")), "volume": f.get("volume", null)})
		place["flows"] = mine
		place["flows_total"] = mine.size()
	raw["place"] = place

	# ---- neighbours' extras ----
	var nx := {}
	for ni in nb_indices:
		var ex := {}
		var nd: Dictionary = bridge.civ_settlement_details(ni)
		if not nd.is_empty():
			ex["specialisation"] = String(nd.get("specialisation", ""))
		if ni < bal.size():
			ex["trade"] = _plain(bal[ni])
		if mil_by_index.has(ni):
			ex["military"] = _plain(mil_by_index[ni])
		nx[str(ni)] = ex
	raw["neighbour_extra"] = nx

	# ---- factions, relations, economy, religion ----
	raw["factions"] = _plain(factions)
	raw["relations"] = _plain(bridge.civ_faction_relations())
	raw["faction_economy"] = _plain(bridge.civ_faction_economy())
	var fid := int(s.get("faction", 0))
	var custom: Array = bridge.civ_custom_religions()
	var axes_vocab: Array = bridge.civ_religion_axes_vocabulary()
	for f in factions:
		var fd: Dictionary = f
		if int(fd.get("id", 0)) == fid and String(fd.get("religion_choice", "")) != "":
			for cr in custom:
				var cd: Dictionary = cr
				if String(cd.get("key", "")) == String(fd["religion_choice"]):
					var axes := []
					var ax: Dictionary = cd.get("axes", {})
					for av in axes_vocab:
						var avd: Dictionary = av
						var k := String(avd.get("key", ""))
						if ax.has(k) and String(ax[k]) != "":
							var vl := String(ax[k])
							for vv in avd.get("values", []):
								if String((vv as Dictionary).get("key", "")) == vl:
									vl = String((vv as Dictionary).get("label", vl))
							axes.append({"label": String(avd.get("label", k)), "value_label": vl})
					raw["custom_religion"] = {"name": String(cd.get("name", "")), "base_label": String(cd.get("base_label", "")),
						"notes": String(cd.get("notes", "")), "axes": axes}
	raw["labels"] = {
		"religion": _vocab(bridge.civ_religion_vocabulary()),
		"specialisation": _vocab(bridge.civ_specialisation_vocabulary()),
		"traits": _vocab(bridge.civ_trait_vocabulary()),
		"government": _vocab(bridge.civ_government_vocabulary()),
		"policy": _vocab(bridge.civ_religious_policy_vocabulary()),
	}

	# ---- ground and surroundings (the engine's sample_cell at the place and on a lattice) ----
	if have_size:
		var cell: Dictionary = bridge.sample_cell(int(s.get("x", 0)), int(s.get("y", 0)))
		if not cell.is_empty():
			raw["place_cell"] = _trim_sample(cell)
		var pts: PackedInt32Array = stream.place_sample_points(int(s.get("x", 0)), int(s.get("y", 0)), ck, int(world["gw"]), int(world["gh"]), bool(world.get("wrap_x", false)))
		var samples := []
		for k in range(0, pts.size(), 2):
			var c: Dictionary = bridge.sample_cell(pts[k], pts[k + 1])
			if not c.is_empty():
				samples.append(_trim_sample(c))
		raw["samples"] = samples
		if wg != null and wg.has_method("river_at"):
			var rv: Dictionary = wg.call("river_at", float(s.get("x", 0)), float(s.get("y", 0)), 25.0 / ck, 2)
			if not rv.is_empty():
				rv.erase("render_points")
				raw["river"] = _plain(rv)
		raw["landmarks"] = _landmarks(wg)
	else:
		unavailable["distances and surroundings"] = "the world declares no map width in km"

	return {"ok": true, "reason": "", "raw": raw, "ms": Time.get_ticks_msec() - t0, "within": int(picks.get("within", 0)),
		"index": index, "tid": tid, "name": String(s.get("name", "")), "faction": fid, "neighbour_names": _names(all, nb_indices)}


static func _names(all: Array, idx: Array) -> Array:
	var out := []
	for i in idx:
		out.append(String((all[i] as Dictionary).get("name", "")))
	return out


static func _trim_sample(c: Dictionary) -> Dictionary:
	var out := {}
	for k in SAMPLE_KEYS:
		if c.has(k):
			out[k] = _plain(c[k])
	return out


## `{available, reason?, rows}`: landmarks exist only after a landmark pass, and a pass that never ran
## must read "not available", not "none".
static func _landmarks(wg: Object) -> Dictionary:
	if wg == null or not wg.has_method("landmarks"):
		return {"available": false, "reason": "this build of the library has no landmark getter"}
	var rows: Array = wg.call("landmarks")
	if rows.is_empty() and wg.has_method("landmark_last_run"):
		var lr: Dictionary = wg.call("landmark_last_run")
		if not bool(lr.get("ok", false)):
			return {"available": false, "reason": String(lr.get("error", "the landmark pass has not run for this world"))}
	return {"available": true, "rows": _plain(rows)}


# ----------------------------------------------------------------------------------------
# The Vault half
# ----------------------------------------------------------------------------------------

## Tags written in a note: the frontmatter `tags:` list and inline `#tags` (not `# Heading`), at most
## `limit`, in order of appearance, without the hash.
static func tags_of(text: String, limit: int = 3) -> PackedStringArray:
	var out := PackedStringArray()
	var re := RegEx.new()
	re.compile("(?:^|[\\s,;\\[])#([A-Za-z][A-Za-z0-9_\\-/]{2,})")
	for m in re.search_all(text):
		var t := m.get_string(1)
		if not out.has(t):
			out.append(t)
		if out.size() >= limit:
			return out
	var fm := RegEx.new()
	fm.compile("(?m)^tags:\\s*\\[?([^\\]\\n]+)\\]?\\s*$")
	var m2 := fm.search(text)
	if m2 != null:
		for t in m2.get_string(1).split(","):
			var tt := t.strip_edges().trim_prefix("#").trim_prefix("\"").trim_suffix("\"")
			if tt.length() >= 3 and not out.has(tt):
				out.append(tt)
			if out.size() >= limit:
				break
	return out


## The Vault notes that bear on the place. `facts` is `gather`'s result, `picked` the note the user
## chose in the panel (`{rel, text}` or `{}`), `stream` the Rust half. Returns
## `{ok, reason, read: [{rel, why, detail, text}], own_note_text, own_note_rel, notes: [String]}` where
## `notes` are plain sentences about what the Vault could not answer (an unbuilt index).
static func gather_vault(bridge: Object, stream: Object, facts: Dictionary, picked: Dictionary) -> Dictionary:
	var info: Dictionary = bridge.vault_info()
	if not bool(info.get("bound", false)):
		return {"ok": false, "reason": "No Vault is connected.", "read": [], "own_note_text": "", "own_note_rel": "", "notes": []}
	var limits: Dictionary = stream.place_limits()
	var files: PackedStringArray = bridge.vault_list_files(2000)
	var file_set := {}
	for f in files:
		file_set[f] = true
	var tid := int(facts.get("tid", 0))
	var fid := int(facts.get("faction", 0))
	var name := String(facts.get("name", ""))
	var notes: Array = []
	var stats: Dictionary = bridge.vault_backlink_stats()
	if not bool(stats.get("built", false)):
		notes.append("The Vault's backlink index is not built, so notes that link to this place are found by name only. Use Data > Vault > Refresh index.")

	# attached to the place, then the note the user picked
	var attached: Array = []
	for l in bridge.vault_links_for("settlement", tid):
		var p := String((l as Dictionary).get("path", ""))
		if file_set.has(p) and not attached.has(p):
			attached.append(p)
	var own_rel := ""
	var own_text := ""
	if not picked.is_empty() and file_set.has(String(picked.get("rel", ""))):
		own_rel = String(picked["rel"])
		own_text = String(picked.get("text", ""))
		if not attached.has(own_rel):
			attached.push_front(own_rel)
	elif not attached.is_empty():
		own_rel = String(attached[0])
		own_text = bridge.vault_read_file(own_rel)
	var backlinks: Array = []
	for b in bridge.vault_entity_backlinks("settlement", tid):
		backlinks.append(String((b as Dictionary).get("rel", "")))

	# attached to the faction or to a neighbour
	var related_attached: Array = []
	if fid > 0:
		for l in bridge.vault_links_for("faction", fid):
			related_attached.append({"rel": String((l as Dictionary).get("path", "")), "label": "its faction"})
	var raw: Dictionary = facts.get("raw", {})
	var settlements: Array = raw.get("settlements", [])
	var names: Array = []
	if name != "":
		names.append(name)
	for ni in facts.get("neighbour_names", []):
		names.append(String(ni))
	for ni_name in facts.get("neighbour_names", []):
		for srow in settlements:
			var sr: Dictionary = srow
			if String(sr.get("name", "")) == String(ni_name):
				for l in bridge.vault_links_for("settlement", int(sr.get("tid", 0))):
					related_attached.append({"rel": String((l as Dictionary).get("path", "")), "label": String(ni_name)})
				break
	var related_names: Array = []
	for f in raw.get("factions", []):
		var fd: Dictionary = f
		if int(fd.get("id", 0)) == fid:
			related_names.append(String(fd.get("name", "")))
			if fd.has("religion_name"):
				related_names.append(String(fd["religion_name"]))
	var rel_key := ""
	for srow in settlements:
		var sr2: Dictionary = srow
		if int(sr2.get("tid", -1)) == tid and sr2.has("religion"):
			rel_key = String(sr2["religion"])
	var rel_label: String = String((raw.get("labels", {}) as Dictionary).get("religion", {}).get(rel_key, ""))
	if rel_label != "":
		related_names.append(rel_label)
	for n in facts.get("neighbour_names", []):
		related_names.append(String(n))

	# tag and name searches (cheap; bounded reads)
	var tag_hits: Array = []
	for t in tags_of(own_text, 3):
		var r: Dictionary = bridge.vault_search("#" + t, 20, 20)
		var rels: Array = []
		for h in r.get("hits", []):
			rels.append(String((h as Dictionary).get("rel", "")))
		tag_hits.append({"tag": "#" + t, "rels": rels})
	var mentions: Array = []
	if name.length() >= 3:
		var r2: Dictionary = bridge.vault_search(name, 20, 40)
		for h in r2.get("hits", []):
			mentions.append(String((h as Dictionary).get("rel", "")))

	var plan_in := {"files": Array(files), "attached": attached, "backlinks": backlinks, "own_note_text": own_text, "own_note_rel": own_rel,
		"related_attached": related_attached, "related_names": related_names, "tag_hits": tag_hits, "mentions": mentions}
	var plan: Dictionary = stream.place_vault_plan(JSON.stringify(plan_in))
	var read: Array = []
	for c in plan.get("candidates", []):
		var cd: Dictionary = c
		var rel := String(cd.get("rel", ""))
		var text: String = own_text if rel == own_rel and own_text != "" else bridge.vault_read_file(rel)
		read.append({"rel": rel, "why": String(cd.get("why", "")), "detail": String(cd.get("detail", "")), "text": text})
	return {"ok": true, "reason": "", "read": read, "own_note_text": own_text, "own_note_rel": own_rel, "notes": notes,
		"max_reads": int(limits.get("max_reads", 10))}
