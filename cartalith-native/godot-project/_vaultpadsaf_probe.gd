extends SceneTree
## Protects the 2026-11-23 fix for "connecting a vault doesn't work on the tablet" (evidence:
## `cartalith-native/docs/evidence/vault-pad-2026-11-23/`). The REAL Java driver (`shell/vault_saf_android.gd`) is run
## against fakes shaped like Godot 4.7.2's `JavaClassWrapper`: the fake `ContentResolver.query` has the two overloads
## the driver can reach and applies the wrapper's own argument rule (`java_class_wrapper.cpp`, `_call_method`: a
## `String` or `String[]` parameter accepts no `null`, an object parameter does). A call no overload accepts answers
## `null`, as the wrapper does, silently. Then the driver goes through the real handler and the real Rust
## `SafVaultProvider` (`vault_connect_saf`), as the vault window's Connect does after the picker.
##
## What it cannot prove: anything about a real provider. The device check is in the evidence README.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_vaultpadsaf_probe.gd
##
## Exit 0 = every check held, 1 = a check failed, 2 = could not run. Mutant: put the 5-argument `query(..., null,
## null, null)` back in `vault_saf_android.gd::_query` (B1-B6 and B8 go red), or `perms.get(i)` back in
## `grant_is_held` (B7 goes red). `cartalith-native/docs/evidence/vault-pad-2026-11-23/mutants.py` runs both.

const Connect := preload("res://shell/vault_saf_connect.gd")
const Handler := preload("res://shell/vault_saf_handler.gd")
const AndroidDriver := preload("res://shell/vault_saf_android.gd")

## Stands for "this argument was not passed", so the fake can tell 4 arguments from 5.
const NO_ARG := "<no-arg>"

var _fails := 0
var _checks := 0


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if cond:
		print("  ok: ", what)
	else:
		_fails += 1
		print("  FAIL: ", what)


## A cursor over fixed rows, with the `android.database.Cursor` calls the driver makes.
class FakeCursor:
	extends RefCounted
	var rows: Array
	var at := -1

	func _init(p_rows: Array) -> void:
		rows = p_rows

	func moveToNext() -> bool:
		at += 1
		return at < rows.size()

	func moveToFirst() -> bool:
		at = 0
		return rows.size() > 0

	func getString(i: int):
		return rows[at][i]

	func getInt(i: int) -> int:
		return int(rows[at][i])

	func getLong(i: int) -> int:
		return int(rows[at][i])

	func close() -> void:
		pass


class FakeIn:
	extends RefCounted
	var bytes: PackedByteArray

	func _init(b: PackedByteArray) -> void:
		bytes = b

	func readAllBytes() -> PackedByteArray:
		return bytes

	func close() -> void:
		pass


class FakeOut:
	extends RefCounted
	var store: Dictionary
	var id: String

	func _init(s: Dictionary, p_id: String) -> void:
		store = s
		id = p_id

	func write(b: PackedByteArray) -> void:
		store[id]["text"] = b.get_string_from_utf8()

	func close() -> void:
		pass


## `ContentResolver.getPersistedUriPermissions()`'s answer: a Java `List` of one `UriPermission`. It has `size()` and
## `iterator()` like the real list and no `get(int)` of its own, so `perms.get(i)` lands on Godot's `Object.get`, as a
## real `JavaObject` does first (`JavaObject::callp`), and answers null.
class FakeList:
	extends RefCounted
	var items: Array
	var at := 0

	func _init(p_items: Array) -> void:
		items = p_items

	func size() -> int:
		return items.size()

	func iterator():
		at = 0
		return self

	func hasNext() -> bool:
		return at < items.size()

	func next():
		at += 1
		return items[at - 1]


class FakePerm:
	extends RefCounted
	var uri: String

	func _init(u: String) -> void:
		uri = u

	func getUri():
		return self

	func toString() -> String:
		return uri

	func isReadPermission() -> bool:
		return true


## `ContentResolver` and `DocumentsContract` over one dictionary of documents: id -> {name, dir, parent, text}.
## Document uris are "doc|<id>", child-listing uris "kids|<id>". The root is "primary:Scratch", like a tree the
## picker grants on primary storage.
class FakeJava:
	extends RefCounted
	const ROOT := "primary:Scratch"
	var docs := {ROOT: {"name": "Scratch", "dir": true, "parent": "", "text": ""}}
	var query_calls := 0
	var refused_calls := 0

	func add(path: String, text: String) -> void:
		var cur := ROOT
		var parts := path.split("/")
		for i in parts.size():
			var id := "%s/%s" % [cur, parts[i]]
			if not docs.has(id):
				docs[id] = {"name": parts[i], "dir": i < parts.size() - 1, "parent": cur, "text": ""}
			cur = id
		docs[cur]["text"] = text

	## The wrapper's overload choice for `query`, from `java_class_wrapper.cpp` (4.7.2-stable): 4 arguments reach
	## `(Uri, String[], Bundle, CancellationSignal)`, objects, where null is accepted; 5 reach `(Uri, String[], String,
	## String[], String)`, where null is NOT accepted for a String or String[]. No match answers null, as there.
	func query(uri, cols, a = NO_ARG, b = NO_ARG, c = NO_ARG):
		query_calls += 1
		var four: bool = typeof(c) == TYPE_STRING and c == NO_ARG and not (typeof(a) == TYPE_STRING and a == NO_ARG)
		var five: bool = typeof(a) == TYPE_STRING and a != NO_ARG and typeof(b) == TYPE_PACKED_STRING_ARRAY and typeof(c) == TYPE_STRING and c != NO_ARG
		if four and (a != null or b != null):
			four = false   ## the fake only ever needs the nulls the driver passes
		if not (four or five) or typeof(cols) != TYPE_PACKED_STRING_ARRAY:
			refused_calls += 1
			return null
		var u := String(uri)
		var id := u.substr(u.find("|") + 1)
		var rows: Array = []
		if u.begins_with("kids|"):
			for k in docs:
				if docs[k]["parent"] == id:
					rows.append(_row(k, cols))
		elif docs.has(id):
			rows.append(_row(id, cols))
		return FakeCursor.new(rows)

	func _row(id: String, cols: PackedStringArray) -> Array:
		var d: Dictionary = docs[id]
		var row: Array = []
		for col in cols:
			match col:
				"document_id": row.append(id)
				"_display_name": row.append(d["name"])
				"mime_type": row.append("vnd.android.document/directory" if d["dir"] else "text/markdown")
				"flags": row.append(8 if d["dir"] else 2 | 4 | 64)
				"last_modified": row.append(1700000000000)
				"_size": row.append(String(d["text"]).to_utf8_buffer().size())
				_: row.append(null)
		return row

	func getPersistedUriPermissions():
		return FakeList.new([FakePerm.new("content://com.android.externalstorage.documents/tree/primary%3AScratch")])

	func openInputStream(uri):
		var id := String(uri).substr(4)
		return FakeIn.new(String(docs[id]["text"]).to_utf8_buffer()) if docs.has(id) else null

	func openOutputStream(uri, _mode: String):
		var id := String(uri).substr(4)
		return FakeOut.new(docs, id) if docs.has(id) else null

	# -- DocumentsContract (static on the real class) --
	func getTreeDocumentId(_tree):
		return ROOT

	func buildDocumentUriUsingTree(_tree, id: String) -> String:
		return "doc|" + id

	func buildChildDocumentsUriUsingTree(_tree, id: String) -> String:
		return "kids|" + id

	func getDocumentId(uri) -> String:
		return String(uri).substr(4)

	func createDocument(_res, parent_uri, mime: String, name: String):
		var parent := String(parent_uri).substr(4)
		var id := "%s/%s" % [parent, name]
		docs[id] = {"name": name, "dir": mime == "vnd.android.document/directory", "parent": parent, "text": ""}
		return "doc|" + id

	func deleteDocument(_res, uri) -> bool:
		return docs.erase(String(uri).substr(4))

	func renameDocument(_res, _uri, _name):
		return null


func _init() -> void:
	var bridge := EngineBridge.new()
	get_root().add_child(bridge)
	if not bridge.world_gen.has_method("vault_connect_saf"):
		print("  ABORT: the loaded extension has no vault_connect_saf; rebuild first")
		quit(2)
		return

	var java := FakeJava.new()
	java.add("Places/Nareth.md", "# Nareth\n")

	# -- A. Positive control: the fake refuses what Godot's wrapper refuses, so a pass below means something --
	var cols := PackedStringArray(["document_id"])
	_ok(java.query("doc|" + FakeJava.ROOT, cols, null, null, null) == null, "A1 control: 5 arguments with null Strings match no overload (answers null)")
	_ok(java.query("doc|" + FakeJava.ROOT, cols, null, null) != null, "A2 control: the 4-argument overload with null objects answers a cursor")

	# -- B. The real driver over the fake, then the real handler and Rust provider --
	var drv = AndroidDriver.new("content://com.android.externalstorage.documents/tree/primary%3AScratch")
	drv._resolver = java
	drv._dc = java
	drv._tree = "tree"
	java.refused_calls = 0
	var av: Dictionary = drv.available()
	_ok(bool(av.get("ok", false)) and bool(av.get("value", false)), "B1: the driver reports the granted folder available (stat of the root went through query)")
	var kids: Dictionary = drv.children(FakeJava.ROOT)
	_ok(bool(kids.get("ok", false)) and (kids["items"] as Array).size() == 1 and String(kids["items"][0]["name"]) == "Places", "B2: the root's listing names its one folder, Places")
	var r: Dictionary = Connect.connect_with_driver(bridge, "content://com.android.externalstorage.documents/tree/primary%3AScratch", "Scratch", drv)
	_ok(bool(r.get("ok", false)), "B3: the shell's connect binds the vault through vault_connect_saf: %s" % r.get("error", ""))
	var files: PackedStringArray = bridge.vault_list_files(0)
	_ok(Array(files) == ["Places/Nareth.md"], "B4: the connected vault lists exactly its one note: %s" % [files])
	var h := Handler.new(drv)
	var w: Dictionary = h._saf_dispatch("write", ["New/B.md", "hello"])
	_ok(bool(w.get("ok", false)) and java.docs.has(FakeJava.ROOT + "/New/B.md") and String(java.docs[FakeJava.ROOT + "/New/B.md"]["text"]) == "hello", "B5: a write creates the folder and the note and reads it back: %s" % w.get("error", ""))
	_ok(java.query_calls > 0 and java.refused_calls == 0, "B6: no query the driver made was refused (%d made, %d refused)" % [java.query_calls, java.refused_calls])

	_ok(drv.grant_is_held(), "B7: the persisted grant is found, so a saved vault reconnects at launch (grant_is_held)")
	_ok(drv.diagnose() == "root_id, stat and children all answered", "B8: the refusal diagnosis walks its three steps: %s" % drv.diagnose())

	bridge.vault_disconnect()
	print("vaultpadsaf probe: %d checks, %d failures" % [_checks, _fails])
	quit(0 if _fails == 0 else 1)
