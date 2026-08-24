extends RefCounted
class_name VaultStore

## Persistence for the Markdown Vault's knowledge links
## (`MARKDOWN_VAULT_INTEGRATION.md` §25/§26, `MARKDOWN_VAULT_SCOPE.md`
## milestone 1).
##
## Two things are stored, in one file, and the split between them is §5's:
##
## | Key | What | Portable? |
## |---|---|---|
## | `store` | The link store — vault ids, knowledge links, imported text | **Yes.** Copy it to another machine and the links come with it. |
## | `binding` | The absolute folder path this device resolved the vault to | **No.** Device-local, and the reason §5 insists a vault's identity is not its path. |
##
## ## Why `user://` and not the project `.zip`
##
## §26's model puts knowledge links in the Cartalith project. This port cannot
## honour that yet, and the reason is a property of the save format rather
## than a shortcut: `cartalith-io` writes the reference HTML app's own `.zip`
## (`SAVEFILE_COMPAT.md`), which carries **no civilisation layer at all** —
## `WorldGen::load_save` produces a world whose `get_settlements()` is empty by
## design. A link written into a save would come back pointing at a settlement
## `tid` that no longer exists in the loaded world.
##
## So links live beside the profile until the save format carries civ data.
## `MARKDOWN_VAULT_SCOPE.md` milestone 3 is that change; this file is the one
## place that has to move when it lands.
##
## ## It never blocks and never throws
##
## A missing file is an empty store. A malformed one is an empty store *and* a
## warning — never a crash and never a silent overwrite of links that are
## still in memory (`bridge.vault_restore_state` refuses malformed JSON and
## returns `false`, which is why this reports rather than assumes).

const PATH := "user://markdown_vault.json"


## Reads the sidecar and pushes it into the engine, re-binding the vault
## folder if this device still has it. Call once, after the bridge exists.
static func load_into(bridge: EngineBridge) -> void:
	if not FileAccess.file_exists(PATH):
		return
	var f := FileAccess.open(PATH, FileAccess.READ)
	if f == null:
		push_warning("Cartalith: could not open %s (%s)" % [PATH, error_string(FileAccess.get_open_error())])
		return
	var raw := f.get_as_text()
	f.close()
	var parsed = JSON.parse_string(raw)
	if typeof(parsed) != TYPE_DICTIONARY:
		push_warning("Cartalith: %s is not a JSON object; the vault links in it were not loaded." % PATH)
		return
	var doc: Dictionary = parsed
	var store = doc.get("store", null)
	if store != null and not bridge.vault_restore_state(JSON.stringify(store)):
		push_warning("Cartalith: %s holds a link store this engine could not read; nothing was loaded." % PATH)
		return
	## The binding is re-established silently and only if the folder is still
	## there. A vault on a disconnected drive leaves every link in §27's
	## "Unbound" state, which the panel says out loud — that is the designed
	## behaviour, not a failure to recover.
	var binding := String(doc.get("binding", ""))
	if binding != "" and DirAccess.dir_exists_absolute(binding):
		bridge.vault_connect(binding, String(doc.get("display_name", "")))


## Writes the engine's current link store and this device's binding. Called
## after every mutation — attaching, detaching, editing a working copy,
## connecting or disconnecting. Cheap: the store is small JSON, and this is
## never on a per-frame path.
static func save_from(bridge: EngineBridge) -> void:
	var state := bridge.vault_state_json()
	if state == "":
		return
	var parsed = JSON.parse_string(state)
	if parsed == null:
		return
	var info := bridge.vault_info()
	var doc := {
		"binding": String(info.get("root", "")),
		"display_name": String(info.get("display_name", "")),
		"store": parsed,
	}
	var f := FileAccess.open(PATH, FileAccess.WRITE)
	if f == null:
		push_warning("Cartalith: could not write %s (%s)" % [PATH, error_string(FileAccess.get_open_error())])
		return
	f.store_string(JSON.stringify(doc, "  "))
	f.close()
