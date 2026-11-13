extends SceneTree
## Folder rename and move on Android storage (`OUTSTANDING_WORK.md`, the vault row), headless: the SAF handler's
## `move_dir` and the real Rust `SafVaultProvider` against a fake document tree. There is no Android here, so this
## proves the LOGIC (the rename, move, copy-and-delete ladder for a folder, the refusals, the honest failure when a
## copy cannot delete its original, the plan and link rewrites through the one planner) and that the Rust provider
## asks through the dispatcher. The Java driver (`vault_saf_android.gd`) is not run. Every assertion reads the fake
## tree back, never the handler's own claim.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_vfoldsaf_probe.gd
##
## Exit 0 = every check held, 1 = a check failed, 2 = could not run (the loaded extension predates folder moves).

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
		_erase_tree(id)
		return {"ok": true}

	## A provider delete of a folder takes everything under it (DocumentsContract.deleteDocument does).
	func _erase_tree(id: String) -> void:
		for k in docs.keys():
			if docs.has(k) and docs[k]["parent"] == id:
				_erase_tree(k)
		docs.erase(id)

	func rename(id: String, new_name: String) -> Dictionary:
		calls.append("rename:" + new_name)
		docs[id]["name"] = new_name
		return {"ok": true, "id": id}

	func move(id: String, _from: String, to_parent: String) -> Dictionary:
		calls.append("move:" + String(docs[id]["name"]))
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


## An older handler: knows every op but `move_dir`, as an app built before folder moves would.
class NoFolderHandler:
	extends RefCounted
	var inner

	func _saf_dispatch(op: String, args: Array) -> Dictionary:
		if op == "move_dir":
			return {"ok": false, "error": "unknown op: move_dir"}
		return inner._saf_dispatch(op, args)


func _connect(bridge, drv) -> Dictionary:
	bridge.vault_disconnect()
	return Connect.connect_with_driver(bridge, "content://fake/tree/1", "Fake", drv)


func _fixture(mode: String, with_image: bool) -> FakeDriver:
	var d := FakeDriver.new(mode)
	d.add("Places/Kelvhold.md", "# Kelvhold\n\nSee [[Ford]].\n")
	d.add("Places/Deep/Ford.md", "# Ford\n\nBack to [[Kelvhold]].\n")
	d.add("Notes/Trade.md", "Goods go to [[Places/Kelvhold]] and [[Places/Deep/Ford|the ford]].\n")
	if with_image:
		d.add("Places/pic.png", "png")
	return d


func _init() -> void:
	var bridge := EngineBridge.new()
	get_root().add_child(bridge)
	if not bridge.world_gen.has_method("vault_folder_rename_plan"):
		print("  ABORT: the loaded extension predates the folder rename bridge; rebuild first")
		quit(2)
		return
	for mode in ["rename", "move", "plain"]:
		_ladder(bridge, mode)
	_image_in_folder(bridge)
	_undeletable(bridge)
	_refusals(bridge)
	_old_handler(bridge)
	print("vfoldsaf probe: %d checks, %d failures" % [_checks, _fails])
	quit(0 if _fails == 0 else 1)


func _ladder(bridge, mode: String) -> void:
	var d := _fixture(mode, false)
	var r := _connect(bridge, d)
	_ok(bool(r.get("ok", false)), "%s: connects" % mode)
	## A same-parent rename: the provider rename rung where offered, else copy + delete.
	var plan: Dictionary = bridge.vault_folder_rename_plan("Places", "Realms")
	_ok(bool(plan.get("ok", false)), "%s: the folder plan is made on a SAF vault: %s" % [mode, plan.get("error", "")])
	if not bool(plan.get("ok", false)):
		return
	_ok((plan.get("moves", []) as Array).size() == 2, "%s: the plan lists both notes that move" % mode)
	var out: Dictionary = bridge.vault_folder_rename("Places", "Realms", String(plan["hash"]))
	_ok(bool(out.get("ok", false)), "%s: the rename succeeds: %s" % [mode, out.get("error", "")])
	_ok(d.text_of("Realms/Kelvhold.md") == "# Kelvhold\n\nSee [[Ford]].\n" and d.text_of("Realms/Deep/Ford.md") == "# Ford\n\nBack to [[Kelvhold]].\n", "%s: both notes are at the new path, byte for byte" % mode)
	_ok(d.text_of("Places/Kelvhold.md") == null and d.text_of("Places/Deep/Ford.md") == null, "%s: the old paths are gone, so there are not two copies" % mode)
	_ok(d.text_of("Notes/Trade.md") == "Goods go to [[Kelvhold]] and [[Ford|the ford]].\n", "%s: the links in the other note were rewritten (read back from the tree)" % mode)
	if mode == "plain":
		_ok(not ("rename:Realms" in d.calls), "plain: no rename call when the provider does not offer it")
		_ok("create:Realms" in d.calls and "delete" in d.calls, "plain: copy then delete was used")
	else:
		_ok("rename:Realms" in d.calls and not ("create:Realms" in d.calls), "%s: the provider own folder rename was used, no copy" % mode)
	## A cross-parent move keeping the name: the move rung where offered, else copy + delete.
	d.calls.clear()
	var plan2: Dictionary = bridge.vault_folder_rename_plan("Realms", "Atlas/Realms")
	var out2: Dictionary = bridge.vault_folder_rename("Realms", "Atlas/Realms", String(plan2.get("hash", "")))
	_ok(bool(out2.get("ok", false)), "%s: a move into a new parent succeeds (it is created): %s" % [mode, out2.get("error", "")])
	_ok(d.text_of("Atlas/Realms/Deep/Ford.md") != null and d.text_of("Realms/Deep/Ford.md") == null, "%s: nested note moved, old one gone" % mode)
	if mode == "move":
		_ok("move:Realms" in d.calls and not ("create:Realms" in d.calls), "move: the provider own move was used")
	else:
		_ok(not ("move:Realms" in d.calls), "%s: no move call without FLAG_SUPPORTS_MOVE" % mode)
	## Rename AND new parent at once: a provider move cannot rename, so even a move-capable one must copy.
	d.calls.clear()
	var plan3: Dictionary = bridge.vault_folder_rename_plan("Atlas/Realms", "Elsewhere/Fordham")
	var out3: Dictionary = bridge.vault_folder_rename("Atlas/Realms", "Elsewhere/Fordham", String(plan3.get("hash", "")))
	_ok(bool(out3.get("ok", false)) and d.text_of("Elsewhere/Fordham/Kelvhold.md") != null, "%s: a new parent and a new name together lands under the NEW name: %s" % [mode, out3.get("error", "")])
	_ok(not ("move:Realms" in d.calls), "%s: ... without the provider move" % mode)
	## One undo step: the same call, paths swapped, lands the notes and the links back.
	var plan4: Dictionary = bridge.vault_folder_rename_plan("Elsewhere/Fordham", "Places")
	var out4: Dictionary = bridge.vault_folder_rename("Elsewhere/Fordham", "Places", String(plan4.get("hash", "")))
	_ok(bool(out4.get("ok", false)) and d.text_of("Places/Deep/Ford.md") == "# Ford\n\nBack to [[Kelvhold]].\n", "%s: moved back" % mode)
	_ok(d.text_of("Notes/Trade.md") == "Goods go to [[Kelvhold]] and [[Ford|the ford]].\n", "%s: and the other note links still resolve (names are unique)" % mode)


## A folder holding a non-note file: the rename and move rungs carry it; the copy rung cannot, so it refuses
## BEFORE copying anything and says so.
func _image_in_folder(bridge) -> void:
	var d := _fixture("rename", true)
	_connect(bridge, d)
	var plan: Dictionary = bridge.vault_folder_rename_plan("Places", "Realms")
	var out: Dictionary = bridge.vault_folder_rename("Places", "Realms", String(plan.get("hash", "")))
	_ok(bool(out.get("ok", false)) and d.text_of("Realms/pic.png") == "png", "rename rung: the image went with the folder")
	var p := _fixture("plain", true)
	_connect(bridge, p)
	var plan2: Dictionary = bridge.vault_folder_rename_plan("Places", "Realms")
	var out2: Dictionary = bridge.vault_folder_rename("Places", "Realms", String(plan2.get("hash", "")))
	_ok(not bool(out2.get("ok", true)), "copy rung: a folder holding an image is refused")
	_ok(String(out2.get("error", "")).contains("not notes") and String(out2.get("error", "")).contains("nothing was changed"), "copy rung: the error says why and that nothing changed: %s" % out2.get("error", ""))
	_ok(p.text_of("Places/pic.png") == "png" and p.text_of("Realms/Kelvhold.md") == null and not ("create:Realms" in p.calls), "copy rung: the tree is untouched and no copy was started")


## The honest failure: a copy whose original cannot be deleted. `can_move` is false on such a grant, so the plan
## refuses up front; the handler is also driven directly for the case the capability check missed.
func _undeletable(bridge) -> void:
	var d := _fixture("undeletable", false)
	_connect(bridge, d)
	_ok(not bool(bridge.vault_folder_rename_plan("Places", "Realms").get("ok", true)), "undeletable: refused up front (no delete, so no honest move)")
	var h := Handler.new(d)
	var res: Dictionary = h._saf_dispatch("move_dir", ["Places", "Realms"])
	_ok(not bool(res.get("ok", true)), "undeletable: a copy whose original folder cannot be deleted is NOT reported as success")
	_ok(String(res.get("error", "")).contains("both now exist"), "undeletable: the error says two copies exist: %s" % res.get("error", ""))
	_ok(d.text_of("Places/Deep/Ford.md") != null and d.text_of("Realms/Deep/Ford.md") != null, "undeletable: the tree really holds both, as the error says")


func _refusals(bridge) -> void:
	var d := _fixture("rename", false)
	d.add("Realms/Other.md", "# Other\n")
	_connect(bridge, d)
	var plan: Dictionary = bridge.vault_folder_rename_plan("Places", "Realms")
	_ok(not bool(plan.get("ok", true)) and String(plan.get("error", "")).contains("already exists"), "a name that holds notes is refused by the planner: %s" % plan.get("error", ""))
	var into: Dictionary = bridge.vault_folder_rename_plan("Places", "Places/Deep/Places")
	_ok(not bool(into.get("ok", true)) and String(into.get("error", "")).contains("into itself"), "a folder into its own descendant is refused: %s" % into.get("error", ""))
	var h := Handler.new(d)
	var res: Dictionary = h._saf_dispatch("move_dir", ["Places", "Realms"])
	_ok(not bool(res.get("ok", true)) and String(res.get("error", "")).contains("never merges"), "the handler itself refuses an existing folder: %s" % res.get("error", ""))
	_ok(d.text_of("Places/Kelvhold.md") != null and d.text_of("Realms/Other.md") == "# Other\n", "refusals changed nothing")
	var file: Dictionary = h._saf_dispatch("move_dir", ["Notes/Trade.md", "X"])
	_ok(not bool(file.get("ok", true)) and String(file.get("error", "")).contains("not a folder"), "a note is not a folder")
	var gone: Dictionary = h._saf_dispatch("move_dir", ["Nope", "X"])
	_ok(not bool(gone.get("ok", true)), "a missing folder is refused")


func _old_handler(bridge) -> void:
	var d := _fixture("rename", false)
	var old := NoFolderHandler.new()
	old.inner = Handler.new(d)
	bridge.vault_disconnect()
	bridge.vault_connect_saf("content://fake/old", "Old", Callable(old, "_saf_dispatch"))
	var plan: Dictionary = bridge.vault_folder_rename_plan("Places", "Realms")
	_ok(bool(plan.get("ok", false)), "old handler: the plan is still made (can_move is shared)")
	var out: Dictionary = bridge.vault_folder_rename("Places", "Realms", String(plan.get("hash", "")))
	_ok(not bool(out.get("ok", true)) and String(out.get("error", "")).contains("unknown op"), "old handler: the folder move fails loudly, never guessed: %s" % out.get("error", ""))
	_ok(d.text_of("Places/Kelvhold.md") != null and d.text_of("Realms/Kelvhold.md") == null and d.text_of("Notes/Trade.md").contains("[[Places/Kelvhold]]"), "old handler: nothing was moved and no link was rewritten")
