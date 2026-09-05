extends Node
## Committed verification harness for the two Markdown-Vault guards added
## 2026-09-05 — the snapshot **existence** check and the `vault.json` **write
## gate**.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vaultguard_probe.tscn
##
## One probe rather than two for `_vaultmap_probe.gd`'s own reason: both
## claims land in `LinkStore`, and each would otherwise have to generate its
## own world.
##
## ## Part 1 — a Map field must stop being offered when its PNG is deleted
##
## `vault_snapshot_radii` and `entity_values` used to read the path straight
## out of `LinkStore::snapshots` with no existence check, so deleting
## `.cartalith/maps/<key>_<radius>.png` from outside Cartalith left the
## checkbox offered and let `vault_block_body` write `![](…)` into the user's
## note pointing at nothing — `MARKDOWN_VAULT_SCOPE.md` §20's "must not expose
## information that the entity does not possess".
##
## The delete happens **through `DirAccess`, not through Cartalith**, which is
## the whole point: nothing tells the store the file is gone.
##
## And it must not lie the other way. Two states have to stay apart, so this
## reads the radii row in all three:
##
##   * never generated        `path == ""`,  no `missing` key
##   * generated and present  `path == rel`, no `missing` key
##   * generated then deleted `path == rel`, `missing == true`
##
## A stale `cartalith_godot.dll` fails part 1 loudly rather than passing it:
## the old engine offers the field after the delete and emits no `missing`
## key. Measured 2026-09-05 by putting each guard back the way it was and
## rebuilding — removing the `entity_values` check turns **1.7, 1.8 and 1.9**
## red, and never setting `missing` turns **1.10** red. There is no
## `has_method` guard here because the change added no method; see
## `MISTAKES.md`'s "Grade a Godot probe as evidence for a Rust change".
##
## ## Part 2 — the `vault.json` write gate, at its call site
##
## `project_save_with_documents` writes `vault.json` only when the store has
## something to say. That gate read `!store.links.is_empty()` — one member of
## a three-member store — so a project whose only vault state was a map
## snapshot wrote **no document at all** and lost it on save. Fixed in
## `52666b9` to `!store.is_empty()`. The predicate itself has a unit test that
## asserts each member alone (`a_store_holding_only_a_snapshot_is_not_an_empty_store`,
## `crates/cartalith-vault/src/links.rs`); **the call site had none, and could
## not** — `project_save_with_documents` takes gdext types on a `GodotClass`,
## so no Rust test can reach it. This is that test.
##
## Putting the old gate back and rebuilding turns **2.snapshots and 2.vaults**
## red (measured 2026-09-05) — so the superseded predicate lost a vaults-only
## store as well as a snapshot-only one, which is wider than the bug report
## said.
##
## It asks the aggregate, not one member: the case list is **derived from the
## store's own serialisation** (the keys of a fully populated `LinkStore`, less
## `version`), so a member this fixture populates joins the loop by itself.
## Its limit, stated rather than papered over: a future member that
## `_full_store()` does not populate is invisible here, exactly as it is to
## `a_store_holding_only_a_snapshot_is_not_an_empty_store` in
## `crates/cartalith-vault/src/links.rs`. Check 2.0 pins the key set so a
## renamed or dropped member trips it instead of silently shrinking the loop.
##
## Committed, like every probe scene in this folder — `STATUS.md`'s F8 row.

const SEED := 483920
const SNAP_PX := 128
const EXPECT_MEMBERS := ["links", "snapshots", "vaults"]

var _bridge
var _app: Node
var _root := ""
var _zip := ""
var _fails: Array = []
## The real profile's vault sidecars, put back exactly as found (this probe
## runs against the same `user://` a real session uses).
var _saved: Dictionary = {}


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("VGUARD   OK  %s" % label)
	else:
		_fails.append(label)
		print("VGUARD   !!  %s   %s" % [label, detail])


func _read(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return ""
	var s := f.get_as_text()
	f.close()
	return s


func _stash(path: String) -> void:
	_saved[path] = _read(path) if FileAccess.file_exists(path) else null
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(path))


func _restore() -> void:
	for path in _saved:
		if FileAccess.file_exists(path):
			DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
		if _saved[path] != null:
			var f := FileAccess.open(path, FileAccess.WRITE)
			f.store_string(String(_saved[path]))
			f.close()


## Whether `key` is in the entity's offer list — the §20 filter, asked through
## the same `#[func]` the panel uses.
func _offers(tid: int, key: String) -> bool:
	for fd in _bridge.vault_export_fields("settlement", tid):
		if String((fd as Dictionary).get("key", "")) == key:
			return true
	return false


## One radius' row out of `vault_snapshot_radii`, or an empty dictionary.
func _radius_row(tid: int, radius: String) -> Dictionary:
	for r in _bridge.vault_snapshot_radii("settlement", tid):
		if String((r as Dictionary).get("radius", "")) == radius:
			return r
	return {}


func _generate(seed_v: int) -> void:
	_bridge.generate({
		"seed": seed_v, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().process_frame
	await get_tree().process_frame


func _ready() -> void:
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(0.8).timeout
	_bridge = _app.bridge

	## `app.gd` runs `VaultStore.load_into(bridge)` at boot, so the store may
	## already hold this machine's real links. Every claim below is about a
	## store this probe built, so take the sidecars out of the way and empty
	## the store before starting. `_restore()` puts them back.
	_stash(VaultStore.PATH)
	_stash(VaultStore.PRE_PROJECT_PATH)
	_bridge.vault_restore_state("")

	await _generate(SEED)
	var settlements: Array = _bridge.settlements()
	if settlements.is_empty():
		_ok("world: settlements exist to snapshot", false)
		_finish()
		return
	var s: Dictionary = settlements[0]
	var tid := int(s.get("tid", 0))
	var sname := String(s.get("name", ""))

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultguard-probe"
	_zip = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultguard-probe.zip"
	## A clean vault every run: a leftover PNG from the last run would make the
	## delete in part 1 remove a file that was never the one under test.
	## Guarded, because `move_to_trash` on a path that is not there prints
	## `SHFileOperation error: 2`.
	if DirAccess.dir_exists_absolute(_root):
		OS.move_to_trash(ProjectSettings.globalize_path(_root))
	DirAccess.make_dir_recursive_absolute(_root + "/Locations")
	var nf := FileAccess.open(_root + "/Locations/Nareth.md", FileAccess.WRITE)
	nf.store_string("# Nareth\n\nA river town at the third ford.\n")
	nf.close()

	var conn: Dictionary = _bridge.vault_connect(_root, "GuardProbeVault")
	_ok("connect: the scratch vault binds", bool(conn.get("ok", false)), String(conn.get("error", "")))

	await _part1_deleted_snapshot(tid)
	_part2_write_gate(tid, sname)
	_finish()


# ---------------------------------------------------------------- part 1 ---

func _part1_deleted_snapshot(tid: int) -> void:
	# -- before anything is generated ---------------------------------------
	var virgin := _radius_row(tid, "local")
	_ok("1.1 never generated: no path and no `missing` key",
		String(virgin.get("path", "")) == "" and not virgin.has("missing"), str(virgin))

	# -- generated, and present ---------------------------------------------
	var snap: Dictionary = _bridge.vault_snapshot("settlement", tid, "local", "", SNAP_PX)
	_ok("1.2 the local map is written", bool(snap.get("ok", false)), String(snap.get("error", "")))
	var rel := String(snap.get("rel", ""))
	var abs_png := _root + "/" + rel
	_ok("1.3 the PNG is on disk", FileAccess.file_exists(abs_png), abs_png)
	_ok("1.4 present: the Map field is offered", _offers(tid, "map_local"))
	var present := _radius_row(tid, "local")
	_ok("1.5 present: the path is filed and NOT flagged missing",
		String(present.get("path", "")) == rel and not present.has("missing"), str(present))

	# -- deleted from outside Cartalith --------------------------------------
	## Through `DirAccess`, so nothing informs the store. This is the exact
	## thing a user does with a file manager, and the state the old code could
	## not see.
	DirAccess.remove_absolute(ProjectSettings.globalize_path(abs_png))
	_ok("1.6 the PNG is gone", not FileAccess.file_exists(abs_png), abs_png)

	_ok("1.7 deleted: the Map field is NO LONGER offered (§20)",
		not _offers(tid, "map_local"), "a checkbox is still offered for an image that is gone")
	var vals: Dictionary = _bridge.vault_entity_values("settlement", tid)
	_ok("1.8 deleted: the value key is ABSENT, not blank",
		not vals.has("map_local"), String(vals.get("map_local", "<absent>")))
	var body: String = _bridge.vault_block_body("settlement", tid, PackedStringArray(["name", "map_local"]))
	_ok("1.9 deleted: the note body carries no image pointing at nothing",
		body.find("![](") < 0 and body.find(rel) < 0, body)

	## The distinguishability requirement: "never generated" and "generated
	## then deleted" are different states and must not collapse into one.
	var gone := _radius_row(tid, "local")
	var ungenerated := _radius_row(tid, "regional")
	_ok("1.10 deleted: the row keeps the filed path AND says `missing`",
		String(gone.get("path", "")) == rel and bool(gone.get("missing", false)) == true, str(gone))
	_ok("1.11 and an ungenerated radius is still a different state",
		String(ungenerated.get("path", "")) == "" and not ungenerated.has("missing"), str(ungenerated))

	# -- and it is not a one-way latch --------------------------------------
	var again: Dictionary = _bridge.vault_snapshot("settlement", tid, "local", "", SNAP_PX)
	_ok("1.12 regenerating restores the offer", bool(again.get("ok", false)) and _offers(tid, "map_local"),
		String(again.get("error", "")))
	var restored := _radius_row(tid, "local")
	_ok("1.13 and clears the `missing` flag", not restored.has("missing"), str(restored))


# ---------------------------------------------------------------- part 2 ---

## Every member of `LinkStore` populated at once, through the real API — the
## fixture the per-member cases are sliced out of, so no link JSON is written
## by hand here and none can drift from `KnowledgeLink`'s actual shape.
func _full_store(tid: int, sname: String) -> Dictionary:
	var att: Dictionary = _bridge.vault_attach("settlement", tid, sname, "Locations/Nareth.md", "")
	if not bool(att.get("ok", false)):
		_ok("2.fixture: a link attaches", false, String(att.get("error", "")))
		return {}
	var parsed = JSON.parse_string(_bridge.vault_state_json())
	if typeof(parsed) != TYPE_DICTIONARY:
		_ok("2.fixture: the store serialises", false, _bridge.vault_state_json().substr(0, 200))
		return {}
	return parsed


## The raw JSON text of one top-level member, brace-matched out of the
## Rust-emitted document rather than re-encoded.
##
## `JSON.parse_string` + `JSON.stringify` cannot build these fixtures. Godot's
## JSON types every number as a float, so a round trip turns
## `KnowledgeLink::entity_id` from `1` into `1.0` and `LinkStore::from_json` —
## the strict parser `GUI_GAP_REGISTER.md` KV-04 is about — refuses the whole
## store. Measured here on the first run: the `links` fixture was rejected and
## the `snapshots` and `vaults` ones, which carry no integers, were not. That
## is the same hazard `vault_store.gd` handles by keeping the store on disk as
## a verbatim string, and this probe has to do the same.
##
## The needle is anchored to `serde_json::to_string_pretty`'s two-space
## top-level indent, so it cannot match a key name occurring inside a string
## value: a real newline inside a JSON string is escaped, so `\n  "links": `
## only ever occurs at the top level.
func _member_text(text: String, member: String) -> String:
	var needle := "\n  \"%s\": " % member
	var at := text.find(needle)
	if at < 0:
		return ""
	var start := at + needle.length()
	var i := start
	var depth := 0
	var in_str := false
	var esc := false
	while i < text.length():
		var c := text[i]
		if in_str:
			if esc:
				esc = false
			elif c == "\\":
				esc = true
			elif c == "\"":
				in_str = false
		elif c == "\"":
			in_str = true
		elif c == "[" or c == "{":
			depth += 1
		elif c == "]" or c == "}":
			depth -= 1
			if depth == 0:
				return text.substr(start, i - start + 1)
		i += 1
	return ""


## `vault.json` out of the archive, or `{}` when the archive does not carry
## one. `has_doc` says which — an absent document and an empty one are not the
## same answer.
func _archived_vault(has_doc: Array) -> Dictionary:
	has_doc.clear()
	var zr := ZIPReader.new()
	if zr.open(_zip) != OK:
		has_doc.append(false)
		return {}
	var files := zr.get_files()
	var present := Array(files).has("vault.json")
	has_doc.append(present)
	var out := {}
	if present:
		var parsed = JSON.parse_string(zr.read_file("vault.json").get_string_from_utf8())
		if typeof(parsed) == TYPE_DICTIONARY:
			out = parsed
	zr.close()
	return out


func _save() -> bool:
	var w: Dictionary = _bridge.project_save_with_documents(_zip, {})
	if not bool(w.get("ok", false)):
		_ok("2.save: the project writes", false, String(w.get("error", "")))
		return false
	return true


func _part2_write_gate(tid: int, sname: String) -> void:
	var full := _full_store(tid, sname)
	if full.is_empty():
		return
	var full_text: String = _bridge.vault_state_json()

	## The case list, derived from the store's own serialisation rather than
	## from a list written here. `version` is the envelope, not a member.
	var members: Array = []
	for k in full.keys():
		if String(k) != "version":
			members.append(String(k))
	members.sort()
	_ok("2.0 the members are exactly the three `is_empty()` conjoins",
		members == EXPECT_MEMBERS,
		"%s -- a member was added, renamed or dropped; give it a case and update EXPECT_MEMBERS" % str(members))

	## Negative control, and the half that makes the rest able to fail: an
	## empty store must write NO `vault.json`, so a gate replaced with `true`
	## is caught here rather than passing every case below.
	_bridge.vault_restore_state("")
	if _save():
		var flag: Array = []
		var doc := _archived_vault(flag)
		_ok("2.1 an empty store writes no vault.json at all",
			flag.size() == 1 and not bool(flag[0]), str(doc).substr(0, 200))

	## One case per member: a store holding **only** that member must survive
	## a real save and a real reopen. Snapshots-alone is the state that
	## silently lost data before `52666b9`; the other two are here so a future
	## member dropped from `is_empty()`'s conjunction fails at the member that
	## was dropped.
	for m in members:
		var raw := _member_text(full_text, m)
		var only := "{\"version\": %d, \"%s\": %s}" % [int(full.get("version", 1)), m, raw]
		var restored: bool = raw != "" and _bridge.vault_restore_state(only)
		_ok("2.%s: the single-member fixture loads" % m, restored, only.substr(0, 200))
		if not restored:
			continue
		if not _save():
			continue
		var flag: Array = []
		var doc := _archived_vault(flag)
		_ok("2.%s: the archive carries a vault.json" % m,
			flag.size() == 1 and bool(flag[0]),
			"a store holding only `%s` wrote no document -- the write gate asks one member again" % m)
		_ok("2.%s: and it carries the member itself" % m,
			doc.has(m) and JSON.stringify(doc[m]) == JSON.stringify(full[m]),
			str(doc).substr(0, 300))

		## Reopen, not just re-read: `project_open` clears the store and
		## restores it from the archive, so this is the round trip a user
		## makes, not a claim about the bytes alone.
		_bridge.vault_restore_state("")
		var opened: Dictionary = _bridge.world_gen.project_open(_zip)
		_ok("2.%s: the project reopens" % m, bool(opened.get("ok", false)), String(opened.get("error", "")))
		var back = JSON.parse_string(_bridge.vault_state_json())
		_ok("2.%s: and the member is back after the round trip" % m,
			typeof(back) == TYPE_DICTIONARY and (back as Dictionary).has(m)
			and JSON.stringify((back as Dictionary)[m]) == JSON.stringify(full[m]),
			str(back).substr(0, 300))


func _finish() -> void:
	_bridge.vault_restore_state("")
	_bridge.vault_disconnect()
	if _zip != "" and FileAccess.file_exists(_zip):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(_zip))
	if _root != "" and DirAccess.dir_exists_absolute(_root):
		OS.move_to_trash(ProjectSettings.globalize_path(_root))
	_restore()
	if _fails.is_empty():
		print("VGUARD   ALL CHECKS PASSED")
	else:
		print("VGUARD   %d FAILED: %s" % [_fails.size(), ", ".join(PackedStringArray(_fails))])
	get_tree().quit(0 if _fails.is_empty() else 1)
