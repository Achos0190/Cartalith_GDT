extends Node
## Isolates the vault sidecar round-trip refusal the audit-4 boot probe hit.
## Hypothesis: `VaultStore.save_from` re-parses the engine's own JSON through
## Godot's `JSON`, which types every number as float, so `entity_id` is written
## back as `1.0` and serde cannot read that into the `i64` `links.rs` declares.
##
##   godot --path . --headless _audit4_vault.tscn

func _ready() -> void:
	var gen: Object = ClassDB.instantiate("WorldGen")

	# A. The engine's own output, handed straight back. This is what
	#    `save_from` would write if it stored the string instead of re-parsing.
	print("[A] engine's own vault_state_json() -> restore: ",
		gen.call("vault_restore_state", gen.call("vault_state_json")))

	# B. The real sidecar the shipped shell wrote on this machine.
	var p := "user://markdown_vault.json"
	if FileAccess.file_exists(p):
		var f := FileAccess.open(p, FileAccess.READ)
		var doc = JSON.parse_string(f.get_as_text())
		f.close()
		var store = doc.get("store")
		print("[B] shipped sidecar as written  -> restore: ",
			gen.call("vault_restore_state", JSON.stringify(store)))

		# C. The same store with every entity_id coerced back to int.
		for l in store.get("links", []):
			l["entity_id"] = int(l["entity_id"])
		print("[C] same store, entity_id as int -> restore: ",
			gen.call("vault_restore_state", JSON.stringify(store)))

		# D. ... and source_modified too. Both are integer fields in Rust
		#     (`i64` / `u64`); Godot's JSON types every number as float.
		for l in store.get("links", []):
			if l.has("source_modified"):
				l["source_modified"] = int(l["source_modified"])
		if store.has("version"):
			store["version"] = int(store["version"])
		print("[D] + source_modified/version    -> restore: ",
			gen.call("vault_restore_state", JSON.stringify(store)))
	else:
		print("[B] no sidecar at ", p)
	get_tree().quit(0)
