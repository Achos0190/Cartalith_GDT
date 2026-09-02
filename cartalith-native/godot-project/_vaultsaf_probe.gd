extends SceneTree

## `MARKDOWN_VAULT_SCOPE.md` milestone 4 -- the Android Storage Access
## Framework provider, exercised end to end against a fake in-memory
## dispatcher standing in for a real `content://` tree. This probe proves
## the delegation mechanism actually works -- Rust `SafVaultProvider` builds
## the `(op, args)` call, the `Callable` crosses into GDScript, the handler
## answers, and the `{ok, ...}` result crosses back and is unwrapped
## correctly -- for list, read, the full attach/edit/preview/write cycle, and
## the same source-changed refusal `cartalith-vault`'s own
## `a_source_that_changed_since_the_preview_is_not_overwritten` proves for
## `FsVault`, now through this path instead.
##
## It does **not**, and cannot, prove anything about real Android SAF
## behaviour -- no picker, no permission grant, no device. See
## `crates/cartalith-godot/src/vault_saf.rs`'s own module doc for exactly
## what a real device pass still has to check that this cannot.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --script _vaultsaf_probe.gd

## Stands in for the Android-side handler `vault_saf.rs`'s module doc
## specifies: an in-memory map instead of a `content://` tree, reachable only
## through the one method the dispatch contract names.
class FakeSaf:
	extends RefCounted
	var files: Dictionary = {}

	func _saf_dispatch(op: String, args: Array) -> Dictionary:
		match op:
			"available":
				return {"ok": true, "value": true}
			"list":
				return {"ok": true, "value": PackedStringArray(files.keys())}
			"read":
				var rel: String = args[0]
				if files.has(rel):
					return {"ok": true, "value": files[rel]}
				return {"ok": false, "error": "not found: %s" % rel}
			"meta":
				var rel: String = args[0]
				if files.has(rel):
					return {"ok": true, "modified": 0, "len": String(files[rel]).length()}
				return {"ok": false, "error": "not found: %s" % rel}
			"exists":
				var rel: String = args[0]
				return {"ok": true, "value": files.has(rel)}
			"write":
				var rel: String = args[0]
				var text: String = args[1]
				files[rel] = text
				return {"ok": true}
			_:
				return {"ok": false, "error": "unknown op: %s" % op}


const HAND := "---\ntags: [worldbuilding]\n---\n\n# Nareth\n\nA river town at the third ford.\n\n## History\n\nFounded in the third age by the Ashfall clans.\n\n## The Old Quarter\n\nNarrow streets, older than the walls.\n\n## Trade\n\nGrain downriver, salt up.\n"

func _init() -> void:
	var fails := 0
	var bridge: EngineBridge = EngineBridge.new()
	get_root().add_child(bridge)

	## Not a SKIP -- see `_vaultprefs_probe.gd` for why this exits `2` rather
	## than reporting a green run over zero assertions.
	if not bridge.world_gen.has_method("vault_connect_saf"):
		print("  ABORT: vault_connect_saf absent -- the loaded extension "
			+ "predates the SAF provider; rebuild before believing this probe")
		quit(2)
		return

	var handler := FakeSaf.new()
	handler.files["Locations/Nareth.md"] = HAND
	var dispatch := Callable(handler, "_saf_dispatch")

	# -- 0. Refused up front: no URI, no valid handler ------------------------
	var refused_uri: Dictionary = bridge.vault_connect_saf("", "X", dispatch)
	if bool(refused_uri.get("ok", true)):
		print("  FAIL: an empty tree URI was accepted")
		fails += 1
	var refused_dispatch: Dictionary = bridge.vault_connect_saf("content://x", "X", Callable())
	if bool(refused_dispatch.get("ok", true)):
		print("  FAIL: an invalid Callable was accepted")
		fails += 1

	# -- 1. Connect -------------------------------------------------------------
	var conn: Dictionary = bridge.vault_connect_saf("content://fake/tree/123", "Fake SAF Vault", dispatch)
	if not bool(conn.get("ok", false)):
		print("  FAIL: vault_connect_saf refused a reachable fake provider: ", conn.get("error", ""))
		quit(1)
		return
	var vault_id: String = String(conn.get("vault_id", ""))
	if not vault_id.begins_with("vault_"):
		print("  FAIL: vault_id does not look like one: ", vault_id)
		fails += 1

	var info: Dictionary = bridge.vault_info()
	if not bool(info.get("bound", false)):
		print("  FAIL: vault_info says unbound right after connecting")
		fails += 1
	if String(info.get("root", "")) != "content://fake/tree/123":
		print("  FAIL: root does not report the tree URI: ", info.get("root", ""))
		fails += 1
	if String(info.get("display_name", "")) != "Fake SAF Vault":
		print("  FAIL: display_name mismatch: ", info.get("display_name", ""))
		fails += 1

	# -- 2. List and read through the Callable -----------------------------------
	var listed: PackedStringArray = bridge.vault_list_files(100)
	if not listed.has("Locations/Nareth.md"):
		print("  FAIL: vault_list_files did not see the fake file: ", listed)
		fails += 1
	var text0 := bridge.vault_read_file("Locations/Nareth.md")
	if text0 != HAND:
		print("  FAIL: vault_read_file did not round-trip the fake file's text")
		fails += 1

	# -- 3. Attach, edit, preview, write ------------------------------------------
	var att: Dictionary = bridge.vault_attach("settlement", 42, "Nareth", "Locations/Nareth.md", "History")
	if not bool(att.get("ok", false)):
		print("  FAIL: vault_attach refused: ", att.get("error", ""))
		quit(1)
		return
	var link_id: String = String(att.get("link_id", ""))

	bridge.vault_set_link_text(link_id, "## History\n\nRewritten through the fake SAF path.\n")
	var prev: Dictionary = bridge.vault_preview_section_write(link_id)
	if not bool(prev.get("ok", false)):
		print("  FAIL: vault_preview_section_write refused: ", prev.get("error", ""))
		quit(1)
		return
	var wr: Dictionary = bridge.vault_write_section(link_id, String(prev.get("hash", "")))
	if not bool(wr.get("ok", false)):
		print("  FAIL: vault_write_section refused: ", wr.get("error", ""))
		fails += 1

	var after := bridge.vault_read_file("Locations/Nareth.md")
	if not after.contains("Rewritten through the fake SAF path."):
		print("  FAIL: the edited section did not land: ", after)
		fails += 1
	if after.contains("Founded in the third age by the Ashfall clans."):
		print("  FAIL: the replaced section's old text is still there")
		fails += 1
	if not after.contains("Narrow streets, older than the walls."):
		print("  FAIL: a sibling section was disturbed")
		fails += 1
	if fails == 0:
		print("  attach/edit/write through the fake SAF path: section replaced, siblings untouched")

	# -- 4. The source-changed refusal, through this path -------------------------
	bridge.vault_set_link_text(link_id, "## History\n\nSecond edit.\n")
	var prev2: Dictionary = bridge.vault_preview_section_write(link_id)
	var stale_hash := String(prev2.get("hash", ""))
	# Someone else writes through the same grant, behind Cartalith's back.
	var external_edit: String = after.replace("Grain downriver, salt up.", "Grain downriver, salt up, and wool.")
	handler.files["Locations/Nareth.md"] = external_edit

	var links: Array = bridge.vault_links_for("settlement", 42)
	var status := ""
	for l in links:
		if String(l.get("link_id", "")) == link_id:
			status = String(l.get("status", ""))
	if status != "stale":
		print("  FAIL: status did not go stale after the external edit: ", status)
		fails += 1

	var refused_write: Dictionary = bridge.vault_write_section(link_id, stale_hash)
	if bool(refused_write.get("ok", true)):
		print("  FAIL: a stale hash was accepted -- this would have clobbered the external edit")
		fails += 1
	if String(handler.files.get("Locations/Nareth.md", "")) != external_edit:
		print("  FAIL: the refused write still changed the fake backend")
		fails += 1

	var prev3: Dictionary = bridge.vault_preview_section_write(link_id)
	var wr2: Dictionary = bridge.vault_write_section(link_id, String(prev3.get("hash", "")))
	if not bool(wr2.get("ok", false)):
		print("  FAIL: the re-previewed write was refused: ", wr2.get("error", ""))
		fails += 1
	var after2 := bridge.vault_read_file("Locations/Nareth.md")
	if not after2.contains("Second edit."):
		print("  FAIL: the second edit did not land: ", after2)
		fails += 1
	if not after2.contains("and wool."):
		print("  FAIL: the concurrent external edit was lost: ", after2)
		fails += 1
	if fails == 0:
		print("  source-changed refusal and recovery through the fake SAF path: both edits survived")

	# -- 5. The escape guard, exercised through the real #[func] boundary ---------
	if bridge.vault_read_file("../evil.md") != "":
		print("  FAIL: an escaping relative path was not refused")
		fails += 1

	# -- 6. Disconnect --------------------------------------------------------------
	bridge.vault_disconnect()
	if bool(bridge.vault_info().get("bound", true)):
		print("  FAIL: still bound after vault_disconnect")
		fails += 1

	print("vault SAF probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
