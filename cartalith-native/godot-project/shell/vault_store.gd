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
## ## What this file is now, and what it stopped being (corrected 2026-09-01)
##
## Until today this header said links live here **because the save format
## cannot carry them** — *"`cartalith-io` writes the reference HTML app's own
## `.zip` … which carries no civilisation layer at all"*. That sentence is
## false, and has been since 2026-08-25. `DECISIONS.md` §7h replaced the flat
## reference archive with the project tree, and its own closing paragraph names
## the difference: *"the format now carrying the civilisation layer, history
## and annotations that the flat one dropped on the floor"*. `cartalith-io`'s
## `DOCUMENT_SLOTS` (`project.rs`) lists `entities/settlements.json` and its
## eleven siblings — and `vault.json` beside them.
##
## §26's project-scoped link store is **built and shipping**, and not by this
## file: `project_bridge.rs`'s `WorldGen::project_save_with_documents` writes
## `SLOT_VAULT` (`"vault.json"`) from `LinkStore::to_json()` whenever the store
## is non-empty; `WorldGen::project_open` parses it back and reports a store it
## cannot read rather than clearing what is in memory; and `WorldGen::load_save`
## (`lib.rs`) clears the outgoing project's links first, so one project's notes
## cannot follow the user into the next.
##
## So this file is **no longer the project's link store**. What it still is:
##
## - **The device binding.** Genuinely not portable and never was — an
##   absolute folder path that means nothing on another machine. That is §5's
##   reason a vault's identity is not its path, and it is untouched by §7h.
## - **The fallback for a session with no project open.** Notes can be attached
##   before a project is ever saved, and a boot restores them from here so they
##   are not lost. A project opened afterwards replaces the links wholesale,
##   which is `project_open`'s documented behaviour and the right precedence:
##   the archive is the record, this is the scratch that survives a quit.
##
## ## The rule milestone 3 landed (2026-09-02), stated once
##
## The header above used to end by saying the remaining question — whether the
## `store` half should exist at all — was one this file could not settle
## alone. It is settled now, and this is the rule:
##
## > **While a project is open, the archive owns the links and this file does
## > not write them.** With no project open it writes them exactly as before,
## > because that session has no archive to write them to.
##
## Which closes a real defect and not only a tidiness one. Before it, the boot
## restore handed *whatever links were last in the sidecar* to whichever
## project was saved next: open project A, quit, boot, save a brand-new
## project C, and A's notes were in C's `vault.json`. `WorldGen::load_save`
## guards the *open* path against exactly that ("keeping the links would leave
## them pointing at settlements that no longer exist") and the boot path had
## no such guard.
##
## ### Nothing is orphaned and nothing is destroyed
##
## An existing `user://markdown_vault.json` is **read at boot as it always
## was**, so links made before projects existed are still restored — they are
## then carried into the first project saved, which is the migration. The
## first time a write would drop the `store` half, the whole sidecar is copied
## to `PRE_PROJECT_PATH` first, once. So the pre-project links survive on
## disk, under a name that says what they are, whatever happens next. This is
## a one-way move by design: there is no path that copies them back, because
## two writable copies of one link store is the state this change exists to
## end.
##
## The cost, stated rather than discovered: links attached **while a project
## is open** and never saved are lost on quit, exactly like a paint draft or an
## unsaved journey. `app.gd` marks the project dirty on every `store_changed`
## for that reason, so File ▸ Save is offered and the autosave picks them up.
##
## ### No `format_version` bump, and why not
##
## `SAVEFILE_COMPAT.md` §4's `format_version` versions the *archive*, and
## nothing about the archive changed: `vault.json` was already a slot
## (`cartalith_io::DOCUMENT_SLOTS`), already written by
## `project_save_with_documents` and already read by `project_open`. §13.3's
## own table says the link store's `version` is independent of it. This change
## is which of two existing writers is authoritative, which the format cannot
## see.
##
## ## It never blocks and never throws
##
## A missing file is an empty store. A malformed one is an empty store *and* a
## warning — never a crash and never a silent overwrite of links that are
## still in memory (`bridge.vault_restore_state` refuses malformed JSON and
## returns `false`, which is why this reports rather than assumes).

const PATH := "user://markdown_vault.json"

## Where `PATH`'s link store is kept when a project takes ownership of it —
## the one-way move's safety net, written at most once and never read back by
## this file.
##
## Deliberately **not** deleted afterwards and deliberately not versioned. It
## is a person's own knowledge links, made before this app had projects to put
## them in; the cost of keeping a few kilobytes forever is nothing against the
## cost of being wrong about whether the migration worked.
const PRE_PROJECT_PATH := "user://markdown_vault.pre_project.json"

## `GUI_GAP_REGISTER.md` **VA-01**'s backlink index, in its **own** file.
##
## Deliberately not a key inside `PATH`. The link store there is portable
## project data (§5) -- copy it to another machine and the links come with it.
## This is a *cache of one folder on this device*, keyed to that folder's
## `(modified, len)` values, and it is rebuilt in a single press if it is
## lost. Mixing them would make the portable half unportable.
const INDEX_PATH := "user://markdown_vault_index.json"

## The owner's 2026-08-25 *"option to confirm always"*, in its **own** file for
## the same reason `INDEX_PATH` has one: the link store is portable project
## data (§5), and a person's "stop asking me" is device state that must not
## travel with a project.
##
## Written by `vault_window.gd` until 2026-08-26, which worked and sat in the
## wrong file — that window's own comment said so and said it did not own this
## file. `PARITY_AUDIT.md` §23 moved it; the window's two functions are
## one-line forwards now.
const PREFS_PATH := "user://markdown_vault_prefs.json"


## Reads the sidecar and pushes it into the engine, re-binding the vault
## folder if this device still has it. Call once, after the bridge exists.
static func load_into(bridge: EngineBridge) -> void:
	## **Outside every branch below, on purpose.** Write preferences are
	## meaningful with no vault connected and with no link store on disk at
	## all: they govern whether a write is previewed, and the panel lists them
	## whether or not a folder is bound. Loading them inside the binding branch
	## — where the backlink index correctly lives, because a cache of one
	## folder is meaningless without that folder — would restore "stop asking
	## me" only for people who already have a vault.
	load_prefs_into(bridge)
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


## Reads the device-local write preferences, if there are any. Never an error:
## a missing or malformed file leaves the engine's own defaults, which are
## *ask every time* — the one direction a corrupt preferences file must fail
## in.
##
## The engine's JSON string goes back **verbatim**, never parsed into a Variant
## and re-emitted, exactly as `load_index_into()` above does and for the reason
## `save_from()` documents at length: Godot's `JSON` has one number type and it
## is `float`, which is how KV-04 discarded every knowledge link a user had
## made. Today's preferences are three booleans and would survive that round
## trip; the rule is about the habit, not about this payload, and the next
## field added here would not survive it.
static func load_prefs_into(bridge: EngineBridge) -> void:
	if not FileAccess.file_exists(PREFS_PATH):
		return
	var f := FileAccess.open(PREFS_PATH, FileAccess.READ)
	if f == null:
		return
	var raw := f.get_as_text()
	f.close()
	if raw.strip_edges() == "":
		return
	bridge.vault_restore_prefs(raw)


## Writes the device-local write preferences. Called after each toggle, which
## is the only thing that changes them.
static func save_prefs_from(bridge: EngineBridge) -> void:
	var json := bridge.vault_prefs_json()
	if json == "":
		return
	var f := FileAccess.open(PREFS_PATH, FileAccess.WRITE)
	if f == null:
		push_warning("Cartalith: could not write %s (%s)" % [PREFS_PATH, error_string(FileAccess.get_open_error())])
		return
	f.store_string(json)
	f.close()


## Writes this device's binding, and the link store **only when no project is
## open**. Called after every mutation — attaching, detaching, editing a
## working copy, connecting or disconnecting. Cheap: the store is small JSON,
## and this is never on a per-frame path.
##
## `project_open` is the caller's `current_project_path != ""`, and it defaults
## to `false` so that a probe with no shell around it keeps the pre-project
## behaviour rather than silently exercising a path it is not testing. The one
## caller that knows is `app.gd`; the header above is the rule.
static func save_from(bridge: EngineBridge, project_open := false) -> void:
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
	}
	## The archive owns the links while a project is open
	## (`project_save_with_documents` writes `vault.json`), so this write drops
	## them — but only once anything already on disk is safely copied aside.
	## A backup that could not be written keeps the old behaviour instead of
	## losing the store: the whole reason the copy exists is that this is the
	## one write whose failure is unrecoverable.
	if not project_open or not _keep_pre_project_copy():
		doc["store"] = state
	var f := FileAccess.open(PATH, FileAccess.WRITE)
	if f == null:
		push_warning("Cartalith: could not write %s (%s)" % [PATH, error_string(FileAccess.get_open_error())])
		return
	f.store_string(JSON.stringify(doc, "  "))
	f.close()


## Copies the sidecar to `PRE_PROJECT_PATH`, **once**, before a project takes
## ownership of the links. Returns whether it is now safe to drop the `store`
## half — which is `true` in the two cases where there is nothing to lose (no
## sidecar, or one carrying no `store`) as well as after a successful copy.
##
## Four guards, each of which is the whole point of one line:
##
## - a backup that already exists is never rewritten, so the *pre-project*
##   store is the one that is kept rather than whatever the most recent
##   project-scoped session happened to leave behind;
## - a sidecar with no `store` key has nothing to preserve and no backup is
##   made, so a fresh install never grows a file it has no use for;
## - a sidecar this build cannot parse is copied **anyway**, verbatim. It is
##   the case where a person's links are most at risk and least understood,
##   and copying bytes needs no parser to be right;
## - a copy that could not be written returns `false`, and `save_from()` then
##   keeps writing the store as it always did. Refusing to migrate is a
##   recoverable state; migrating with no copy behind it is not.
static func _keep_pre_project_copy() -> bool:
	if FileAccess.file_exists(PRE_PROJECT_PATH) or not FileAccess.file_exists(PATH):
		return true
	var f := FileAccess.open(PATH, FileAccess.READ)
	if f == null:
		push_warning("Cartalith: could not read %s to keep a copy of it (%s); the vault links were left in it."
			% [PATH, error_string(FileAccess.get_open_error())])
		return false
	var raw := f.get_as_text()
	f.close()
	var parsed = JSON.parse_string(raw)
	if typeof(parsed) == TYPE_DICTIONARY and not (parsed as Dictionary).has("store"):
		return true
	var out := FileAccess.open(PRE_PROJECT_PATH, FileAccess.WRITE)
	if out == null:
		## Reported rather than swallowed: this is the one write whose failure
		## means a link store would be dropped with no copy behind it.
		push_warning("Cartalith: could not keep %s before the project took over the vault links (%s); %s was left as it is."
			% [PRE_PROJECT_PATH, error_string(FileAccess.get_open_error()), PATH])
		return false
	out.store_string(raw)
	out.close()
	return true
