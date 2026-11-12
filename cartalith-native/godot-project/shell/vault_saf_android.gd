extends RefCounted
## The real driver `vault_saf_handler.gd` talks through on Android: `DocumentsContract` and
## `ContentResolver`, reached from GDScript with `JavaClassWrapper` (Godot 4.4+, the pattern
## `llm_net_hint.gd` already uses), so no Java is written or built here. It is the one file in the vault
## that **cannot run off a device**: every call below is read against the Android SDK documentation and
## was never executed. `OUTSTANDING_WORK.md` and `STATUS.md` say so; the owner's glass check is the proof.
##
## The driver contract is documented on the handler. Ids are SAF document ids; a document's `Uri` is
## rebuilt from the id with `buildDocumentUriUsingTree` each time, so nothing here holds a stale uri.
##
## A Java exception inside a wrapped call prints an engine error and returns `null`, so every call's
## result is tested for `null` and becomes `{ok: false, error}` rather than being assumed.

const MIME_DIR := "vnd.android.document/directory"
## `application/octet-stream` so a provider does not append an extension derived from the mime type to the
## name it was given (`Note.md` staying `Note.md`); `create` also checks the name it got back.
const MIME_FILE := "application/octet-stream"
## `DocumentsContract.Document.FLAG_SUPPORTS_DELETE` and `FLAG_DIR_SUPPORTS_CREATE`, the Android SDK values.
const FLAG_SUPPORTS_DELETE := 4
const FLAG_DIR_SUPPORTS_CREATE := 8

var _tree_uri_text: String
var _tree
var _dc
var _resolver
var _uri_cls


## True only on an Android build that has the Java bridge.
static func is_available() -> bool:
	return OS.get_name() == "Android" and Engine.has_singleton("AndroidRuntime")


func _init(tree_uri: String = "") -> void:
	_tree_uri_text = tree_uri


## Connects the Java side once. `false` with a reason when the bridge is missing, never an exception.
func prepare() -> Dictionary:
	if not is_available():
		return _fail("this is not an Android build with the Java bridge")
	var rt = Engine.get_singleton("AndroidRuntime")
	var activity = rt.getActivity()
	if activity == null:
		return _fail("no Android activity")
	_resolver = activity.getContentResolver()
	_dc = JavaClassWrapper.wrap("android.provider.DocumentsContract")
	_uri_cls = JavaClassWrapper.wrap("android.net.Uri")
	if _resolver == null or _dc == null or _uri_cls == null:
		return _fail("the Android document classes could not be wrapped")
	_tree = _uri_cls.parse(_tree_uri_text)
	if _tree == null:
		return _fail("the folder address is not a valid uri")
	return {"ok": true}


## Asks Android to keep the grant across restarts (`takePersistableUriPermission`, read+write = 3).
## Failure is reported, not fatal: the grant still works until the app is closed.
func persist_grant() -> Dictionary:
	var r := prepare()
	if not bool(r.get("ok", false)):
		return r
	_resolver.takePersistableUriPermission(_tree, 3)
	return {"ok": true}


## True when Android still lists this tree among the grants the app was given. A revoked folder answers
## `false`, so a reconnect on launch can say "the folder is no longer shared" instead of failing later.
func grant_is_held() -> bool:
	if not bool(prepare().get("ok", false)):
		return false
	var perms = _resolver.getPersistedUriPermissions()
	if perms == null:
		return false
	for i in perms.size():
		var p = perms.get(i)
		if p != null and String(p.getUri().toString()) == _tree_uri_text and bool(p.isReadPermission()):
			return true
	return false


static func _fail(msg: String) -> Dictionary:
	return {"ok": false, "error": msg}


func _doc_uri(id: String):
	return _dc.buildDocumentUriUsingTree(_tree, id)


func available() -> Dictionary:
	var r := root_id()
	if not bool(r.get("ok", false)):
		return {"ok": true, "value": false}
	return {"ok": true, "value": bool(stat(String(r["id"])).get("ok", false))}


func root_id() -> Dictionary:
	if _tree == null and not bool(prepare().get("ok", false)):
		return _fail("the Android folder is not reachable")
	var id = _dc.getTreeDocumentId(_tree)
	if id == null:
		return _fail("the folder address carries no document id")
	return {"ok": true, "id": String(id)}


## Create is the root's own flag; delete is read off the children (a root is never deletable, and a
## provider that cannot delete its notes cannot rename by copy). An empty folder reports deletable, the
## optimistic reading, which `_copy_delete` backs with an honest failure message if it was wrong.
func capabilities() -> Dictionary:
	var r := root_id()
	if not bool(r.get("ok", false)):
		return r
	var s := stat(String(r["id"]))
	if not bool(s.get("ok", false)):
		return s
	var kids := children(String(r["id"]))
	var can_delete := true
	if bool(kids.get("ok", false)) and not kids["items"].is_empty():
		can_delete = false
		var n := 0
		for it in kids["items"]:
			if (int(it["flags"]) & FLAG_SUPPORTS_DELETE) != 0:
				can_delete = true
				break
			n += 1
			if n >= 20:
				break
	return {"ok": true, "create": (int(s["flags"]) & FLAG_DIR_SUPPORTS_CREATE) != 0, "delete": can_delete}


func children(id: String) -> Dictionary:
	if _tree == null and not bool(prepare().get("ok", false)):
		return _fail("the Android folder is not reachable")
	var uri = _dc.buildChildDocumentsUriUsingTree(_tree, id)
	var cols := PackedStringArray(["document_id", "_display_name", "mime_type", "flags"])
	var cur = _resolver.query(uri, cols, null, null, null)
	if cur == null:
		return _fail("the folder could not be listed")
	var items: Array = []
	while cur.moveToNext():
		items.append({
			"id": String(cur.getString(0)),
			"name": String(cur.getString(1)),
			"is_dir": String(cur.getString(2)) == MIME_DIR,
			"flags": int(cur.getInt(3)),
		})
	cur.close()
	return {"ok": true, "items": items}


func stat(id: String) -> Dictionary:
	var cols := PackedStringArray(["last_modified", "_size", "flags"])
	var cur = _resolver.query(_doc_uri(id), cols, null, null, null)
	if cur == null:
		return _fail("the document could not be read")
	var out := _fail("the document is gone")
	if cur.moveToFirst():
		## A null column (a provider that does not know its modified time) reads 0, which the handler
		## contract defines as unknown; it is never replaced by a plausible time.
		out = {"ok": true, "modified": int(cur.getLong(0)) / 1000, "len": int(cur.getLong(1)), "flags": int(cur.getInt(2))}
	cur.close()
	return out


func read_text(id: String) -> Dictionary:
	var stream = _resolver.openInputStream(_doc_uri(id))
	if stream == null:
		return _fail("the note could not be opened")
	## `readAllBytes` is API 33; an older device answers an error here rather than a wrong text.
	var bytes = stream.readAllBytes()
	stream.close()
	if bytes == null:
		return _fail("the note could not be read")
	return {"ok": true, "text": (bytes as PackedByteArray).get_string_from_utf8()}


## Opened with mode `"wt"` so an existing file is truncated; the handler reads the file back to check it.
func write_text(id: String, text: String) -> Dictionary:
	var out = _resolver.openOutputStream(_doc_uri(id), "wt")
	if out == null:
		return _fail("the note could not be opened for writing")
	out.write(text.to_utf8_buffer())
	out.close()
	return {"ok": true}


func create(parent_id: String, name: String, is_dir: bool) -> Dictionary:
	var uri = _dc.createDocument(_resolver, _doc_uri(parent_id), MIME_DIR if is_dir else MIME_FILE, name)
	if uri == null:
		return _fail("could not create %s" % name)
	var cols := PackedStringArray(["_display_name"])
	var cur = _resolver.query(uri, cols, null, null, null)
	var got := ""
	if cur != null:
		if cur.moveToFirst():
			got = String(cur.getString(0))
		cur.close()
	if got != name:
		## The provider renamed it (an appended extension, a collision suffix): put the exact name back,
		## or give up and remove what it made, because a note under another name is a different note.
		var fixed = _dc.renameDocument(_resolver, uri, name)
		if fixed == null:
			_dc.deleteDocument(_resolver, uri)
			return _fail("the provider named it %s instead of %s" % [got, name])
		uri = fixed
	return {"ok": true, "id": String(_dc.getDocumentId(uri))}


func delete(id: String) -> Dictionary:
	if bool(_dc.deleteDocument(_resolver, _doc_uri(id))):
		return {"ok": true}
	return _fail("the provider refused to delete it")


func rename(id: String, new_name: String) -> Dictionary:
	var uri = _dc.renameDocument(_resolver, _doc_uri(id), new_name)
	if uri == null:
		return _fail("the provider refused to rename it")
	return {"ok": true, "id": String(_dc.getDocumentId(uri))}


func move(id: String, from_parent_id: String, to_parent_id: String) -> Dictionary:
	var uri = _dc.moveDocument(_resolver, _doc_uri(id), _doc_uri(from_parent_id), _doc_uri(to_parent_id))
	if uri == null:
		return _fail("the provider refused to move it")
	return {"ok": true, "id": String(_dc.getDocumentId(uri))}
