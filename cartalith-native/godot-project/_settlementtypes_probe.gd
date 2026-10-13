extends SceneTree
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: SettlementTypeStore: round trip through project save/open, back-compat with no settlement_types slot, drop applies a faction's default type, deleting a type reverts its factions.
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): --headless --path . --script _settlementtypes_probe.gd (SceneTree script, no .tscn)
## Result: 'settlement types probe: PASS'
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##

## `lazy-riding-piglet.md` Batch D verification. Four checks, following
## `_savereopen_probe.gd`'s own real-`EngineBridge` shape (SP-1's cited
## template):
##
##   1. round trip -- create a type + a faction default through
##      `SettlementTypeStore` directly (driving the API rather than the
##      window, which the task brief allows when driving the UI itself is
##      impractical headless), save the project, reopen it in a second
##      `EngineBridge`, confirm every field survives.
##   2. backward compatibility -- a project saved with NO
##      `library/settlement_types.json` slot opens with no warning and an
##      empty library.
##   3. drop wiring -- a real before/after: dropping a settlement under a
##      faction WITH a default type actually applies the bundle
##      (kind/specialisation/traits/walls/age all checked against the
##      type's own fields), and dropping under a faction with NO default
##      behaves byte-identically to the same drop with
##      `apply_default_to_settlement` never called at all.
##   4. (FH-3) defaults after the editor moved -- deleting a type reverts
##      only its factions to None and drops them from the saved document. The
##      per-faction default is now EDITED in the Factions hub (Identity tab),
##      which `_factionhub_probe.gd`'s `_run_default_type` drives; this file
##      never instantiated the Settlement types window, so the removed column
##      needed no change here beyond this store-level leg.

func _init() -> void:
	var fails := 0

	var bridge: EngineBridge = EngineBridge.new()
	get_root().add_child(bridge)
	bridge.world_gen.set_params({"tect.plates": 9})
	bridge.world_gen.generate_sized(24601, 640.0, 96, 64)
	bridge.world_gen.recompute_civilisation()
	bridge.has_world = true
	bridge.save_api = bridge.world_gen.has_method("project_save")

	var factions: Array = bridge.get_factions()
	if factions.is_empty():
		print("  FAIL: fixture generated no factions")
		quit(1)
		return
	var fid := int((factions[0] as Dictionary).get("id", 1))
	var fid2 := int((factions[1] as Dictionary).get("id", 2)) if factions.size() > 1 else fid

	# -- 1. Round trip ----------------------------------------------------------
	SettlementTypeStore.clear()
	var tid := SettlementTypeStore.new_type("Customs town")
	SettlementTypeStore.set_field(tid, "kind", "city")
	var specs := bridge.civ_specialisation_vocabulary()
	var spec_key := String((specs[1] as Dictionary).get("key", "none")) if specs.size() > 1 else "none"
	SettlementTypeStore.set_field(tid, "specialisation", spec_key)
	var vocab := bridge.civ_trait_vocabulary()
	var trait_keys: Array = []
	for i in mini(2, vocab.size()):
		trait_keys.append(String((vocab[i] as Dictionary).get("key", "")))
		SettlementTypeStore.toggle_trait(tid, trait_keys[i])
	SettlementTypeStore.set_field(tid, "walls", 1)
	SettlementTypeStore.set_field(tid, "age_mode", "fixed")
	SettlementTypeStore.set_field(tid, "age_years", 250)
	SettlementTypeStore.set_faction_default(fid, tid)

	var doc := SettlementTypeStore.document()
	print("  document: %d bytes" % doc.length())
	var path := OS.get_user_data_dir().path_join("_settlementtypes_probe.ctl")
	if not bridge.save_project(path, {"library/settlement_types.json": doc}):
		print("  FAIL: save_project refused")
		quit(1)
		return

	# Simulate a fresh session: the in-memory store is gone until restored.
	SettlementTypeStore.clear()
	if not SettlementTypeStore.types().is_empty():
		print("  FAIL: clear() did not empty the store")
		fails += 1

	var b2: EngineBridge = EngineBridge.new()
	get_root().add_child(b2)
	if not b2.load_save(path):
		print("  FAIL: load_save refused")
		fails += 1
	var got := String(b2.last_documents.get("library/settlement_types.json", ""))
	if got != doc:
		print("  FAIL: the settlement-types document did not come back byte-identical")
		print("    wrote: ", doc)
		print("    read:  ", got)
		fails += 1
	else:
		print("  document round-tripped byte-identical")

	SettlementTypeStore.restore_document(got)
	var restored := SettlementTypeStore.types()
	if restored.size() != 1:
		print("  FAIL: expected 1 restored type, got %d" % restored.size())
		fails += 1
	else:
		var rt: Dictionary = restored[0]
		var ok := String(rt.get("name", "")) == "Customs town" \
			and String(rt.get("kind", "")) == "city" \
			and String(rt.get("specialisation", "")) == spec_key \
			and int(rt.get("walls", -99)) == 1 \
			and String(rt.get("age_mode", "")) == "fixed" \
			and int(rt.get("age_years", -1)) == 250 \
			and (rt.get("traits", []) as Array).size() == trait_keys.size()
		print("  restored type fields match: ", ok)
		if not ok:
			print("    got: ", rt)
			fails += 1
		var rt_id := String(rt.get("id", ""))
		if SettlementTypeStore.faction_default(fid) != rt_id:
			print("  FAIL: faction default did not survive (got '%s', want '%s')"
				% [SettlementTypeStore.faction_default(fid), rt_id])
			fails += 1
		else:
			print("  faction default survived")

	# -- 2. Backward compatibility ------------------------------------------------
	SettlementTypeStore.clear()
	var old_type_id := SettlementTypeStore.new_type("Should not leak")
	var path_old := OS.get_user_data_dir().path_join("_settlementtypes_probe_old.ctl")
	if not bridge.save_project(path_old, {}):
		print("  FAIL: save_project (no settlement-types slot) refused")
		fails += 1
	var b3: EngineBridge = EngineBridge.new()
	get_root().add_child(b3)
	if not b3.load_save(path_old):
		print("  FAIL: load_save (old-shape project) refused")
		fails += 1
	var got_old := String(b3.last_documents.get("library/settlement_types.json", ""))
	if got_old != "":
		print("  FAIL: a project saved with no settlement-types slot returned one anyway: ", got_old)
		fails += 1
	SettlementTypeStore.restore_document(got_old)
	if not SettlementTypeStore.types().is_empty():
		print("  FAIL: restoring an absent slot did not clear the in-memory library "
			+ "(a stale type leaked across projects)")
		fails += 1
	else:
		print("  a pre-existing project with no settlement-types slot opened clean, no leak")

	# -- 3. Drop wiring -----------------------------------------------------------
	SettlementTypeStore.clear()
	var drop_type := SettlementTypeStore.new_type("Fortified village")
	SettlementTypeStore.set_field(drop_type, "kind", "village")
	SettlementTypeStore.set_field(drop_type, "specialisation", spec_key)
	SettlementTypeStore.toggle_trait(drop_type, trait_keys[0]) if not trait_keys.is_empty() else null
	SettlementTypeStore.set_field(drop_type, "walls", 0)
	SettlementTypeStore.set_field(drop_type, "age_mode", "fixed")
	SettlementTypeStore.set_field(drop_type, "age_years", 40)
	SettlementTypeStore.set_faction_default(fid, drop_type)
	# fid2 is deliberately left on None.

	# Find two distinct dry land cells to drop on.
	var gw := bridge.world_gen.get_width()
	var gh := bridge.world_gen.get_height()
	var spots: Array = []
	var y := 4
	while y < gh - 4 and spots.size() < 2:
		var x := 4
		while x < gw - 4 and spots.size() < 2:
			spots.append(Vector2(x, y))
			x += 7
		y += 7
	if spots.size() < 2:
		print("  FAIL: could not find two drop spots")
		fails += 1
	else:
		# WITH a default.
		var idx_with := bridge.civ_drop_settlement(spots[0].x, spots[0].y, "town", fid, "", false)
		if idx_with < 0:
			print("  FAIL: control drop (with-default) refused")
			fails += 1
		else:
			SettlementTypeStore.apply_default_to_settlement(bridge, fid, idx_with)
			var d := bridge.civ_settlement_details(idx_with)
			## `kind` is on the settlement row itself (`bridge.settlements()`),
			## not on `civ_settlement_details()` -- the same split
			## `place_editor_window.gd::_build_class_and_polity` reads (`s`)
			## versus `_build_urban`/`_build_traits` (`details`).
			var row: Dictionary = (bridge.settlements() as Array)[idx_with]
			var traits: PackedStringArray = d.get("traits", PackedStringArray())
			var applied := String(row.get("kind", "")) == "village" \
				and String(d.get("specialisation", "")) == spec_key \
				and int(d.get("walls", -99)) == 0 \
				and int(d.get("age", -1)) == 40 \
				and (trait_keys.is_empty() or traits.has(trait_keys[0]))
			print("  WITH a default: bundle applied = ", applied)
			if not applied:
				print("    row: ", row)
				print("    details: ", d)
				fails += 1

		# WITHOUT a default (fid2), compared against the identical drop with
		# apply_default_to_settlement never called -- the "no call at all"
		# contract, checked as a real diff rather than assumed.
		var idx_a := bridge.civ_drop_settlement(spots[1].x, spots[1].y, "town", fid2, "", false)
		if idx_a < 0:
			print("  FAIL: no-default drop A refused")
			fails += 1
		else:
			var before_none := bridge.civ_settlement_details(idx_a)
			var row_before: Dictionary = ((bridge.settlements() as Array)[idx_a] as Dictionary).duplicate()
			SettlementTypeStore.apply_default_to_settlement(bridge, fid2, idx_a)
			var after_none := bridge.civ_settlement_details(idx_a)
			var row_after: Dictionary = (bridge.settlements() as Array)[idx_a]
			var identical := String(row_before.get("kind", "")) == String(row_after.get("kind", "")) \
				and String(before_none.get("specialisation", "")) == String(after_none.get("specialisation", "")) \
				and int(before_none.get("walls", -99)) == int(after_none.get("walls", -99)) \
				and int(before_none.get("age", -99)) == int(after_none.get("age", -99)) \
				and (before_none.get("traits", PackedStringArray()) as PackedStringArray) \
					== (after_none.get("traits", PackedStringArray()) as PackedStringArray)
			print("  faction with NO default: apply_default_to_settlement is a byte-identical no-op = ", identical)
			if not identical:
				print("    before: ", before_none)
				print("    after:  ", after_none)
				fails += 1

	# -- 4. FH-3: the defaults now edited in the Factions hub ---------------------
	## Protects: FH-3 moved the per-faction default's EDITOR out of the Settlement
	## types window into the hub's Identity tab (`_factionhub_probe.gd`'s
	## `_run_default_type` owns the UI). The store underneath is unchanged and this
	## leg pins what both windows rely on: deleting a type leaves every faction that
	## pointed at it on None (`""`) and out of the saved document, and a defaults-only
	## store still serialises and restores (the document is not gated on types).
	SettlementTypeStore.clear()
	var gone := SettlementTypeStore.new_type("Doomed")
	var kept := SettlementTypeStore.new_type("Kept")
	SettlementTypeStore.set_faction_default(fid, gone)
	SettlementTypeStore.set_faction_default(fid2, kept)
	SettlementTypeStore.delete_type(gone)
	var del_ok := SettlementTypeStore.faction_default(fid) == "" \
		and SettlementTypeStore.faction_default(fid2) == kept
	print("  deleting a type reverts only its factions to None = ", del_ok)
	if not del_ok:
		fails += 1
	var d4 := JSON.parse_string(SettlementTypeStore.document()) as Dictionary
	var in_doc := (d4.get("faction_defaults", {}) as Dictionary)
	if in_doc.has(str(fid)) or String(in_doc.get(str(fid2), "")) != kept:
		print("  FAIL: the saved document does not match the post-delete defaults: ", in_doc)
		fails += 1
	SettlementTypeStore.restore_document(SettlementTypeStore.document())
	if SettlementTypeStore.faction_default(fid2) != kept or SettlementTypeStore.faction_default(fid) != "":
		print("  FAIL: the post-delete document did not restore to the same defaults")
		fails += 1

	print("settlement types probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
