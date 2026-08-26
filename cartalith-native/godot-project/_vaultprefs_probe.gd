extends SceneTree

## `PARITY_AUDIT.md` §23, wiring item 4: the vault's "confirm always"
## preferences now reach the disk through `VaultStore`, like every other piece
## of vault state, and they travel as **verbatim text** in both directions.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --script _vaultprefs_probe.gd
##
## Three claims, in the order they can break:
##
##   1. `save_prefs_from()` writes the engine's own JSON byte for byte — not a
##      Variant parsed and re-emitted, which is how KV-04 floated every integer
##      in the link store and discarded it on the next boot.
##   2. `load_prefs_into()` restores it: a flag set, dropped from the engine,
##      and read back off disk comes back set.
##   3. `VaultStore.load_into()` loads preferences even with no vault bound and
##      no link store on disk — the placement bug this move fixes, where they
##      were only read when the window happened to be constructed.

func _init() -> void:
	var fails := 0
	var bridge: EngineBridge = EngineBridge.new()
	get_root().add_child(bridge)

	if not bridge.world_gen.has_method("vault_set_write_pref"):
		print("  SKIP: this extension has no vault preference surface")
		quit(0)
		return

	# A clean slate: the sidecar may be left over from a real session.
	if FileAccess.file_exists(VaultStore.PREFS_PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PREFS_PATH))

	# -- 1. Written verbatim ---------------------------------------------------
	bridge.vault_set_write_pref("block", true)
	var engine_json := bridge.vault_prefs_json()
	VaultStore.save_prefs_from(bridge)
	if not FileAccess.file_exists(VaultStore.PREFS_PATH):
		print("  FAIL: nothing was written to ", VaultStore.PREFS_PATH)
		quit(1)
		return
	var on_disk := FileAccess.open(VaultStore.PREFS_PATH, FileAccess.READ).get_as_text()
	print("  engine  -> ", engine_json)
	print("  on disk -> ", on_disk)
	if on_disk != engine_json:
		print("  FAIL: the file is not the engine's own string byte for byte")
		fails += 1

	# -- 2. Restored verbatim --------------------------------------------------
	## Back to the defaults *through the engine*, so the only way the flag can
	## be true after the next call is that the file put it there.
	bridge.vault_set_write_pref("block", false)
	if bool(bridge.vault_write_prefs().get("block", false)):
		print("  FAIL: could not clear the flag before the reload")
		fails += 1
	VaultStore.load_prefs_into(bridge)
	if not bool(bridge.vault_write_prefs().get("block", false)):
		print("  FAIL: the preference did not come back off disk")
		fails += 1
	elif bridge.vault_prefs_json() != engine_json:
		print("  FAIL: it came back changed: ", bridge.vault_prefs_json())
		fails += 1
	else:
		print("  round trip: byte-identical, flag restored")

	# -- 3. Loaded with no vault bound ----------------------------------------
	## `load_into()` on a fresh bridge with no `markdown_vault.json` at all.
	## Before the move this returned at its first line and preferences were
	## restored only as a side effect of the window being constructed.
	var b2: EngineBridge = EngineBridge.new()
	get_root().add_child(b2)
	var store_backup := ""
	var had_store := FileAccess.file_exists(VaultStore.PATH)
	if had_store:
		store_backup = FileAccess.open(VaultStore.PATH, FileAccess.READ).get_as_text()
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	VaultStore.load_into(b2)
	if not bool(b2.vault_write_prefs().get("block", false)):
		print("  FAIL: load_into() skipped the preferences when no link store exists")
		fails += 1
	else:
		print("  unbound session: preferences restored with no vault connected")
	if had_store:
		var f := FileAccess.open(VaultStore.PATH, FileAccess.WRITE)
		f.store_string(store_backup)
		f.close()

	# Leave the real profile as it was found.
	DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PREFS_PATH))

	print("vault prefs probe: ", "PASS" if fails == 0 else "%d FAILURE(S)" % fails)
	quit(1 if fails > 0 else 0)
