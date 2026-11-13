extends SceneTree
## Owner request 2026-11-13, the Android half: a note created in a subfolder, and in a folder that does not exist
## yet, through the real Rust `SafVaultProvider` over a fake document tree (the `_vesaf_probe.gd` fake). There is
## no Android here: this proves the LOGIC (the handler walks and makes the missing folders, the file lands in the
## right one, the existing-file refusal holds, `VaultFolderPick.folders` lists the folders the provider reports),
## not a real documents provider; the Java driver (`vault_saf_android.gd`) is not run.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_notefoldersaf_probe.gd
##
## Exit 0 = held, 1 = a check failed, 2 = could not run. Every file claim is read back from the fake tree.

const Fake := preload("res://_vesaf_probe.gd")
const Connect := preload("res://shell/vault_saf_connect.gd")
const Pick := preload("res://shell/vault_folder_pick.gd")

var _fails := 0
var _checks := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("  FAIL: ", what)


func _init() -> void:
	var bridge := EngineBridge.new()
	get_root().add_child(bridge)
	if not bridge.world_gen.has_method("vault_connect_saf"):
		print("  ABORT: the loaded extension has no SAF bridge")
		quit(2)
		return
	for mode in ["plain", "move"]:
		var d = Fake.FakeDriver.new(mode)
		d.add("Lore/Old.md", "# Old\n")
		d.add("Templates/T.md", "# {{title}}\n")
		bridge.vault_disconnect()
		var r: Dictionary = Connect.connect_with_driver(bridge, "content://fake/tree/1", "Fake", d)
		_ok(bool(r.get("ok", false)), "%s: connects" % mode)
		var folders: Array = Pick.folders(bridge.vault_list_files(0), "Settlements")
		_ok(folders.has("Lore") and folders.has("Templates") and folders.has("Settlements") and folders[0] == "", "%s: the chooser lists the provider's folders (and the default)" % mode)
		# An existing subfolder.
		var a: Dictionary = bridge.vault_create_file("Lore/Nareth.md", "# Nareth\n")
		_ok(bool(a.get("ok", false)), "%s: create in an existing subfolder: %s" % [mode, a.get("error", "")])
		_ok(d.text_of("Lore/Nareth.md") == "# Nareth\n", "%s: the file is in Lore, read back" % mode)
		# A folder that does not exist yet, two levels deep.
		var b: Dictionary = bridge.vault_create_file("Lore/Coast/Deep/Port.md", "# Port\n")
		_ok(bool(b.get("ok", false)), "%s: create in a new nested folder: %s" % [mode, b.get("error", "")])
		_ok(d.text_of("Lore/Coast/Deep/Port.md") == "# Port\n", "%s: the file is in the new folders, read back" % mode)
		# From a template into a new folder.
		var c: Dictionary = bridge.vault_create_from_template("Templates/T.md", "Places/Ford.md", "Ford")
		_ok(bool(c.get("ok", false)), "%s: template create in a new folder: %s" % [mode, c.get("error", "")])
		_ok(d.text_of("Places/Ford.md") != null and String(d.text_of("Places/Ford.md")).contains("Ford"), "%s: the template note is in Places, read back" % mode)
		# Never overwrites.
		var e: Dictionary = bridge.vault_create_file("Lore/Nareth.md", "# other\n")
		_ok(not bool(e.get("ok", true)) and d.text_of("Lore/Nareth.md") == "# Nareth\n", "%s: an existing file is refused and untouched" % mode)
		# One folder per path: the second create did not duplicate Lore.
		var lores := 0
		for k in d.docs:
			if d.docs[k]["name"] == "Lore" and d.docs[k]["dir"]:
				lores += 1
		_ok(lores == 1, "%s: Lore exists once, not once per create (%d)" % [mode, lores])
	print("notefoldersaf probe: %d checks, %d failures" % [_checks, _fails])
	quit(0 if _fails == 0 else 1)
