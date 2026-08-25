extends SceneTree

## KV-04: proves the sidecar round trip no longer floats integers.
##
## The bug was never in Rust. `vault_store.gd` parsed the engine's JSON into a
## Godot Variant and re-emitted it, and Godot's JSON has one number type --
## `float`. Every `i64`/`u64` came back with a decimal point and serde refused
## the lot, so `load_into()` discarded every link on every boot.

func _init() -> void:
	## Shaped like the real store: an i64 entity_id and a u64 unix timestamp,
	## which are the two fields that actually failed.
	var engine_json := '{"links":[{"entity_id":1,"source_modified":1787605785,"note":"a.md"}]}'
	var fails := 0

	# -- The old path: parse then stringify (what shipped) --------------------
	var old_doc := {"binding": "C:\\v", "store": JSON.parse_string(engine_json)}
	var old_out: String = JSON.stringify(JSON.parse_string(JSON.stringify(old_doc)).get("store"))
	print("  old path -> ", old_out)
	if not ("1.0" in old_out and "1787605785.0" in old_out):
		print("  FAIL: expected the old path to float its integers; it did not")
		fails += 1

	# -- The new path: the engine's string carried verbatim --------------------
	var new_doc := {"binding": "C:\\v\"q", "display_name": "My \"Vault\"", "store": engine_json}
	var on_disk := JSON.stringify(new_doc, "  ")
	var back = JSON.parse_string(on_disk)
	if typeof(back) != TYPE_DICTIONARY:
		print("  FAIL: sidecar did not parse back as an object")
		fails += 1
	else:
		var store = back.get("store", null)
		if typeof(store) != TYPE_STRING:
			print("  FAIL: store came back as ", type_string(typeof(store)), ", not String")
			fails += 1
		elif store != engine_json:
			print("  FAIL: store changed.\n    in : ", engine_json, "\n    out: ", store)
			fails += 1
		else:
			print("  new path -> byte-identical, integers intact")
		## The escaping has to survive a Windows path and embedded quotes, or
		## the fix trades a number bug for a path bug.
		if String(back.get("binding", "")) != "C:\\v\"q":
			print("  FAIL: binding did not round trip: ", back.get("binding"))
			fails += 1
		if String(back.get("display_name", "")) != "My \"Vault\"":
			print("  FAIL: display_name did not round trip")
			fails += 1

	print("KV-04 probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
