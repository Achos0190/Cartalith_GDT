extends SceneTree
## VE-6's Android half (`VAULT_EDITOR_PROPOSAL.md` s.6): the SAF handler's `move`, `can_move` and the rest of its
## ops, driven through the real Rust `SafVaultProvider` (`vault_connect_saf`) against a fake document tree. There is
## no Android here and nothing this proves is about a real provider: it proves the LOGIC (path walking, the
## rename -> move -> copy+delete ladder, the refusals, the honest failure when a copy cannot delete its original)
## and that the Rust provider asks and moves through the dispatcher. The Java driver (`vault_saf_android.gd`) is
## not run; `OUTSTANDING_WORK.md` says so.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_vesaf_probe.gd
##
## Exit 0 = every check held, 1 = a check failed, 2 = could not run (the loaded extension predates `can_move`).
## It reads the fake tree back after every move, never the handler's own claim.

const Handler := preload("res://shell/vault_saf_handler.gd")
const Connect := preload("res://shell/vault_saf_connect.gd")
const AndroidDriver := preload("res://shell/vault_saf_android.gd")

var _fails := 0
var _checks := 0


## A document tree in a dictionary: id -> {name, dir, parent, text, flags}. `mode` switches what the "provider"
## supports, so each rung of the ladder is exercised: "rename", "move", "plain" (neither), "undeletable"
## (copy works, delete is refused), "readonly" (the root cannot create).
class FakeDriver:
	extends RefCounted
	var docs: Dictionary = {}
	var next_id := 100
	var mode := "plain"
	var calls: Array = []
	var truncates := true

	func _init(p_mode: String = "plain") -> void:
		mode = p_mode
		docs["root"] = {"name": "vault", "dir": true, "parent": "", "text": "", "flags": 0}

	func add(path: String, text: String) -> String:
		var parts := path.split("/")
		var cur := "root"
		for i in parts.size() - 1:
			cur = _child(cur, parts[i], true)
		var id := _child(cur, parts[parts.size() - 1], false)
		docs[id]["text"] = text
		return id

	func _child(parent: String, name: String, is_dir: bool) -> String:
		for id in docs:
			if docs[id]["parent"] == parent and docs[id]["name"] == name:
				return id
		next_id += 1
		var id := "d%d" % next_id
		docs[id] = {"name": name, "dir": is_dir, "parent": parent, "text": "", "flags": _flags(is_dir)}
		return id

	func _flags(is_dir: bool) -> int:
		var f := 0 if is_dir else 2 | 4   ## write + delete
		if mode == "rename" or mode == "move":
			f |= 64
		if mode == "move":
			f |= 256
		if mode == "undeletable":
			f &= ~4
		return f

	## The text of the file at `path`, or null.
	func text_of(path: String) -> Variant:
		var cur := "root"
		for seg in path.split("/"):
			var nxt := ""
			for id in docs:
				if docs[id]["parent"] == cur and docs[id]["name"] == seg:
					nxt = id
			if nxt == "":
				return null
			cur = nxt
		return docs[cur]["text"]

	func available() -> Dictionary:
		return {"ok": true, "value": true}

	func root_id() -> Dictionary:
		return {"ok": true, "id": "root"}

	func capabilities() -> Dictionary:
		return {"ok": true, "create": mode != "readonly", "delete": mode != "undeletable" and mode != "readonly"}

	func children(id: String) -> Dictionary:
		var items: Array = []
		for k in docs:
			if docs[k]["parent"] == id:
				items.append({"id": k, "name": docs[k]["name"], "is_dir": docs[k]["dir"], "flags": docs[k]["flags"]})
		return {"ok": true, "items": items}

	func stat(id: String) -> Dictionary:
		if not docs.has(id):
			return {"ok": false, "error": "gone"}
		return {"ok": true, "modified": 0, "len": String(docs[id]["text"]).length(), "flags": docs[id]["flags"]}

	func read_text(id: String) -> Dictionary:
		if not docs.has(id):
			return {"ok": false, "error": "gone"}
		return {"ok": true, "text": docs[id]["text"]}

	func write_text(id: String, text: String) -> Dictionary:
		if not docs.has(id):
			return {"ok": false, "error": "gone"}
		var old := String(docs[id]["text"])
		## A provider that ignores the truncate flag leaves a longer old file's tail behind.
		docs[id]["text"] = text if (truncates or old.length() <= text.length()) else text + old.substr(text.length())
		return {"ok": true}

	func create(parent_id: String, name: String, is_dir: bool) -> Dictionary:
		calls.append("create:" + name)
		for k in docs:
			if docs[k]["parent"] == parent_id and docs[k]["name"] == name:
				return {"ok": false, "error": "exists"}
		return {"ok": true, "id": _child(parent_id, name, is_dir)}

	func delete(id: String) -> Dictionary:
		calls.append("delete")
		if mode == "undeletable":
			return {"ok": false, "error": "FLAG_SUPPORTS_DELETE is not set"}
		docs.erase(id)
		return {"ok": true}

	func rename(id: String, new_name: String) -> Dictionary:
		calls.append("rename")
		docs[id]["name"] = new_name
		return {"ok": true, "id": id}

	func move(id: String, _from: String, to_parent: String) -> Dictionary:
		calls.append("move")
		docs[id]["parent"] = to_parent
		return {"ok": true, "id": id}


## An old handler that does not know `can_move` or `move`: forwards the original six ops to a driver-backed
## handler and refuses the rest, as `vault_saf.rs` documents for an unknown op.
class OldHandler:
	extends RefCounted
	var inner

	func _saf_dispatch(op: String, args: Array) -> Dictionary:
		if op == "can_move" or op == "move":
			return {"ok": false, "error": "unknown op: " + op}
		return inner._saf_dispatch(op, args)


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("  FAIL: ", what)


func _init() -> void:
	var bridge := EngineBridge.new()
	get_root().add_child(bridge)
	if not bridge.world_gen.has_method("vault_rename_plan") or not bridge.world_gen.has_method("vault_connect_saf"):
		print("  ABORT: the loaded extension predates the vault rename or SAF bridge; rebuild first")
		quit(2)
		return

	# -- A. The ladder, one provider mode at a time, through the real Rust provider ------------------------------
	for mode in ["rename", "move", "plain"]:
		_ladder(bridge, mode)
	_undeletable(bridge)
	_old_handler(bridge)
	_readonly(bridge)
	_overwrite_and_missing(bridge)
	_write_readback(bridge)
	_desktop_guards(bridge)

	print("vesaf probe: %d checks, %d failures" % [_checks, _fails])
	quit(0 if _fails == 0 else 1)


func _connect(bridge, drv) -> Dictionary:
	bridge.vault_disconnect()
	return Connect.connect_with_driver(bridge, "content://fake/tree/1", "Fake", drv)


func _ladder(bridge, mode: String) -> void:
	var d := FakeDriver.new(mode)
	d.add("Places/Nareth.md", "# Nareth\n\nA river town.\n")
	d.add("Notes/Trade.md", "Goods go to [[Nareth]] and [[Places/Nareth|the town]].\n")
	var r := _connect(bridge, d)
	_ok(bool(r.get("ok", false)), "%s: connects" % mode)
	## Same folder: a rename. Different folder + different name: copy+delete in every mode (move keeps the name).
	var plan: Dictionary = bridge.vault_rename_plan("Places/Nareth.md", "Places/Nareth Ford.md")
	_ok(bool(plan.get("ok", false)), "%s: the plan is made on a SAF vault (can_move answered true): %s" % [mode, plan.get("error", "")])
	if not bool(plan.get("ok", false)):
		return
	var out: Dictionary = bridge.vault_rename("Places/Nareth.md", "Places/Nareth Ford.md", String(plan["hash"]))
	_ok(bool(out.get("ok", false)), "%s: the rename succeeds: %s" % [mode, out.get("error", "")])
	_ok(d.text_of("Places/Nareth Ford.md") == "# Nareth\n\nA river town.\n", "%s: the new path holds the text byte for byte" % mode)
	_ok(d.text_of("Places/Nareth.md") == null, "%s: the old path is gone, so there are not two copies" % mode)
	var trade := String(d.text_of("Notes/Trade.md"))
	_ok(trade == "Goods go to [[Nareth Ford]] and [[Nareth Ford|the town]].\n", "%s: both links in the other note were rewritten, the alias kept (read back from the tree; the crate shortens a path link whose name is unique)" % mode)
	var used: Array = d.calls
	if mode != "plain":
		_ok("rename" in used and not ("create:Nareth Ford.md" in used), "%s: the provider's own rename was used for a same-folder rename, no copy" % mode)
	else:
		_ok(not ("rename" in used), "%s: no rename call when the provider does not offer it" % mode)
		_ok("create:Nareth Ford.md" in used and "delete" in used, "%s: copy then delete was used" % mode)
	## A cross-folder move keeping the name: "move" rung where supported, copy+delete otherwise.
	d.calls.clear()
	var plan2: Dictionary = bridge.vault_rename_plan("Places/Nareth Ford.md", "Towns/Nareth Ford.md")
	var out2: Dictionary = bridge.vault_rename("Places/Nareth Ford.md", "Towns/Nareth Ford.md", String(plan2.get("hash", "")))
	_ok(bool(out2.get("ok", false)), "%s: a move to a new folder succeeds (the folder is created): %s" % [mode, out2.get("error", "")])
	_ok(d.text_of("Towns/Nareth Ford.md") != null and d.text_of("Places/Nareth Ford.md") == null, "%s: moved to Towns/, gone from Places/" % mode)
	if mode == "move":
		_ok("move" in d.calls and not ("create:Nareth Ford.md" in d.calls), "move: the provider's own move was used")
	else:
		_ok(not ("move" in d.calls), "%s: no move call without FLAG_SUPPORTS_MOVE" % mode)
	## A new folder AND a new name at once: a provider move cannot rename, so even a move-capable provider
	## must copy+delete, or the note would land under its OLD name.
	d.calls.clear()
	var plan3: Dictionary = bridge.vault_rename_plan("Towns/Nareth Ford.md", "Elsewhere/Fordham.md")
	var out3: Dictionary = bridge.vault_rename("Towns/Nareth Ford.md", "Elsewhere/Fordham.md", String(plan3.get("hash", "")))
	_ok(bool(out3.get("ok", false)) and d.text_of("Elsewhere/Fordham.md") == "# Nareth\n\nA river town.\n", "%s: a new folder and a new name together lands under the NEW name: %s" % [mode, out3.get("error", "")])
	_ok(d.text_of("Towns/Nareth Ford.md") == null and not ("move" in d.calls), "%s: ...by copy+delete, with the provider's move not used" % mode)


func _undeletable(bridge) -> void:
	var d := FakeDriver.new("undeletable")
	d.add("A.md", "alpha\n")
	_connect(bridge, d)
	## can_move is false here (capabilities say delete is not possible), so the plan refuses and nothing moves.
	var plan: Dictionary = bridge.vault_rename_plan("A.md", "B.md")
	_ok(not bool(plan.get("ok", true)), "undeletable: rename is refused up front (no delete, so no honest move)")
	_ok(d.text_of("A.md") == "alpha\n" and d.text_of("B.md") == null, "undeletable: nothing was copied")
	## Drive the handler directly for the case capabilities missed: the copy is made, the delete refused.
	var h := Handler.new(d)
	var res: Dictionary = h._saf_dispatch("move", ["A.md", "B.md"])
	_ok(not bool(res.get("ok", true)), "undeletable: a copy whose original cannot be deleted is NOT reported as success")
	_ok(String(res.get("error", "")).contains("both now exist"), "undeletable: the error says two copies exist: %s" % res.get("error", ""))
	_ok(d.text_of("A.md") == "alpha\n" and d.text_of("B.md") == "alpha\n", "undeletable: the tree really holds both, as the error says")


func _old_handler(bridge) -> void:
	var d := FakeDriver.new("rename")
	d.add("A.md", "alpha\n")
	var old := OldHandler.new()
	old.inner = Handler.new(d)
	bridge.vault_disconnect()
	var r: Dictionary = bridge.vault_connect_saf("content://fake/old", "Old", Callable(old, "_saf_dispatch"))
	_ok(bool(r.get("ok", false)), "old handler: still connects")
	var plan: Dictionary = bridge.vault_rename_plan("A.md", "B.md")
	_ok(not bool(plan.get("ok", true)), "old handler: a handler without can_move leaves rename refusing, never guessing true")
	_ok(bridge.vault_read_file("A.md") == "alpha\n", "old handler: the other ops still work")


func _readonly(bridge) -> void:
	var d := FakeDriver.new("readonly")
	d.add("A.md", "alpha\n")
	_connect(bridge, d)
	_ok(not bool(bridge.vault_rename_plan("A.md", "B.md").get("ok", true)), "readonly root: rename is refused")


func _overwrite_and_missing(bridge) -> void:
	var d := FakeDriver.new("rename")
	d.add("A.md", "alpha\n")
	d.add("B.md", "bravo\n")
	_connect(bridge, d)
	var h := Handler.new(d)
	var res: Dictionary = h._saf_dispatch("move", ["A.md", "B.md"])
	_ok(not bool(res.get("ok", true)) and String(res.get("error", "")).contains("never overwrites"), "a move onto an existing note is refused")
	_ok(d.text_of("A.md") == "alpha\n" and d.text_of("B.md") == "bravo\n", "...and neither note changed")
	var gone: Dictionary = h._saf_dispatch("move", ["Nope.md", "C.md"])
	_ok(not bool(gone.get("ok", true)), "a missing source is refused")
	var unk: Dictionary = h._saf_dispatch("frobnicate", [])
	_ok(not bool(unk.get("ok", true)) and String(unk.get("error", "")).contains("unknown op"), "an unknown op fails loudly")


func _write_readback(bridge) -> void:
	var d := FakeDriver.new("plain")
	d.add("N.md", "a much longer original text than the new one\n")
	d.truncates = false
	var h := Handler.new(d)
	var res: Dictionary = h._saf_dispatch("write", ["N.md", "short\n"])
	_ok(not bool(res.get("ok", true)), "write: a provider that does not truncate is caught by the read-back, not trusted")
	d.truncates = true
	var res2: Dictionary = h._saf_dispatch("write", ["Deep/New/Note.md", "hello\n"])
	_ok(bool(res2.get("ok", false)) and d.text_of("Deep/New/Note.md") == "hello\n", "write: creates the folders and the note, and it reads back")
	var lst: Dictionary = h._saf_dispatch("list", [100])
	var names: PackedStringArray = lst.get("value", PackedStringArray())
	_ok(names.has("Deep/New/Note.md") and names.has("N.md"), "list: finds notes at any depth")
	d.add(".trash/Old.md", "dropped\n")
	var lst2: Dictionary = h._saf_dispatch("list", [100])
	_ok(not (lst2.get("value", PackedStringArray()) as PackedStringArray).has(".trash/Old.md"), "list: a dot-folder's notes (.trash, .obsidian) are not vault notes")
	var lim: Dictionary = h._saf_dispatch("list", [1])
	_ok((lim.get("value", PackedStringArray()) as PackedStringArray).size() == 1, "list: honours the limit")


func _desktop_guards(bridge) -> void:
	_ok(not AndroidDriver.is_available(), "desktop: the Java driver reports unavailable")
	var drv := AndroidDriver.new("content://x")
	_ok(not bool(drv.prepare().get("ok", true)), "desktop: preparing the Java driver answers a reason, not an exception")
	_ok(not Connect.should_use_picker(), "desktop: the folder picker stays the in-app browser")
	_ok(not Connect.restore(bridge, "content://x", ""), "desktop: restoring a content:// binding is a quiet false")
	_ok(not Connect.restore(bridge, "C:/not/a/uri", ""), "restore refuses a non-content address")
	var h := Handler.new(null)
	_ok(not bool(h._saf_dispatch("available", []).get("ok", true)), "no driver: every op fails, never a silent empty list")
