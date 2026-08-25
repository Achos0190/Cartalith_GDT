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

## `GUI_GAP_REGISTER.md` **VA-01**'s backlink index, in its **own** file.
##
## Deliberately not a key inside `PATH`. The link store there is portable
## project data (§5) -- copy it to another machine and the links come with it.
## This is a *cache of one folder on this device*, keyed to that folder's
## `(modified, len)` values, and it is rebuilt in a single press if it is
## lost. Mixing them would make the portable half unportable.
const INDEX_PATH := "user://markdown_vault_index.json"


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
	## The store is handed back to the engine **verbatim**, never re-serialised.
	## `JSON.stringify()` here was the other half of the same defect
	## `save_from()` documents: it floats every integer, so even a correctly
	## written sidecar was corrupted again on the way in. Fixing only the writer
	## would have left this one still failing.
	var store = doc.get("store", null)
	var store_json := ""
	if typeof(store) == TYPE_STRING:
		store_json = store
	elif store != null:
		## A sidecar written before 2026-08-25 holds a nested object whose
		## integers are already `1.0` on disk. It is unrecoverable by any reader
		## -- serde will refuse it however it is re-encoded -- so this branch
		## exists to fail with the warning below rather than to succeed. One
		## press rebuilds the index; the links themselves are re-attached by
		## hand, which is the loss this fix stops recurring.
		store_json = JSON.stringify(store)
	if store_json != "" and not bridge.vault_restore_state(store_json):
		push_warning("Cartalith: %s holds a link store this engine could not read; nothing was loaded." % PATH)
		return
	## The binding is re-established silently and only if the folder is still
	## there. A vault on a disconnected drive leaves every link in §27's
	## "Unbound" state, which the panel says out loud — that is the designed
	## behaviour, not a failure to recover.
	var binding := String(doc.get("binding", ""))
	if binding != "" and DirAccess.dir_exists_absolute(binding):
		bridge.vault_connect(binding, String(doc.get("display_name", "")))
		## The index is only meaningful against a bound folder, so it is
		## restored inside this branch and nowhere else. A refusal is silent:
		## the panel already reports "not built", which is exactly the state a
		## cache this engine could not read leaves us in, and a warning about
		## a *cache* would be noise.
		load_index_into(bridge)


## Reads the saved backlink index, if there is one. Never an error: a missing
## or unreadable index is "not built yet", which is a state the panel already
## draws and one press fixes.
static func load_index_into(bridge: EngineBridge) -> void:
	if not FileAccess.file_exists(INDEX_PATH):
		return
	var f := FileAccess.open(INDEX_PATH, FileAccess.READ)
	if f == null:
		return
	var raw := f.get_as_text()
	f.close()
	if raw.strip_edges() == "":
		return
	bridge.vault_restore_backlink_index(raw)


## Writes the backlink index. Called after a refresh or a rebuild, and not on
## every vault mutation -- attaching a note does not change what links to
## what.
static func save_index_from(bridge: EngineBridge) -> void:
	var json := bridge.vault_backlink_index_json()
	if json == "":
		return
	var f := FileAccess.open(INDEX_PATH, FileAccess.WRITE)
	if f == null:
		push_warning("Cartalith: could not write %s (%s)" % [INDEX_PATH, error_string(FileAccess.get_open_error())])
		return
	f.store_string(json)
	f.close()


## Writes the engine's current link store and this device's binding. Called
## after every mutation — attaching, detaching, editing a working copy,
## connecting or disconnecting. Cheap: the store is small JSON, and this is
## never on a per-frame path.
static func save_from(bridge: EngineBridge) -> void:
	var state := bridge.vault_state_json()
	if state == "":
		return
	## **Parsed for validation only. The parsed value must never be what gets
	## written back.** Godot's `JSON` has one number type and it is `float`, so
	## a round trip through `parse_string` -> `stringify` re-emits every integer
	## with a decimal point: `entity_id` (`links.rs`, `i64`) came back out as
	## `1.0` and `source_modified` (`u64`) as `1787605785.0`. serde refuses both,
	## `vault_restore_state()` returned false, and `load_into()` discarded the
	## whole store with the warning it prints on every boot -- silent loss of
	## every link the user had made, in shipped, milestone-complete
	## functionality. Nothing in Rust was wrong: the engine's own string
	## restores cleanly, which is how this was bisected.
	if JSON.parse_string(state) == null:
		return
	var info := bridge.vault_info()
	## The engine's JSON goes in **as a string**, not as a nested object, so
	## `stringify` escapes it rather than re-encoding the numbers inside it.
	## `load_into()` hands that string straight back with no parse of its own.
	## Still one portable JSON file, per the note on `PATH`.
	var doc := {
		"binding": String(info.get("root", "")),
		"display_name": String(info.get("display_name", "")),
		"store": state,
	}
	var f := FileAccess.open(PATH, FileAccess.WRITE)
	if f == null:
		push_warning("Cartalith: could not write %s (%s)" % [PATH, error_string(FileAccess.get_open_error())])
		return
	f.store_string(JSON.stringify(doc, "  "))
	f.close()
