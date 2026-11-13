extends RefCounted
## The Android Storage Access Framework half of the vault, as the `_saf_dispatch` handler
## `crates/cartalith-godot/src/vault_saf.rs` specifies (`{op, args} -> {ok, ...}`), kept free of any
## Android call so everything here runs on a desktop against a fake driver.
##
## **Why a driver.** SAF names files by document id, not by path, and only a device can answer the
## Java questions. This script owns the part that is plain logic and so can be tested anywhere: walking a
## vault-relative path through folder listings, creating missing folders, the write with a read-back
## check, and (VE-6's Android half, `VAULT_EDITOR_PROPOSAL.md` s.6) the `move` op with its fallback
## ladder. `vault_saf_android.gd` is the driver that talks to `DocumentsContract`; the probe
## `_vesaf_probe.gd` is a second driver that is a dictionary in memory.
##
## **Driver contract** (every method answers a Dictionary `{ok, error, ...}`; ids are SAF document ids):
##   available() -> {ok, value:bool}                     is the grant reachable now
##   root_id() -> {ok, id}                               the tree's root document id
##   capabilities() -> {ok, create:bool, delete:bool}    what the root allows; gates `can_move`
##   children(id) -> {ok, items:[{id, name, is_dir, flags:int}]}
##   stat(id) -> {ok, modified:int (Unix s, 0 unknown), len:int, flags:int}
##   read_text(id) -> {ok, text}
##   write_text(id, text) -> {ok}                         replaces the whole file
##   create(parent_id, name, is_dir) -> {ok, id}          the id of what now exists with exactly that name
##   delete(id) -> {ok}
##   rename(id, new_name) -> {ok, id}                     only called when flags say FLAG_SUPPORTS_RENAME
##   move(id, from_parent_id, to_parent_id) -> {ok, id}   only called when flags say FLAG_SUPPORTS_MOVE
##
## **What it must never do.** Overwrite a destination on `move`; report success for a copy whose
## original it could not delete (two copies of a note are not a rename); or invent a modified time.

## `DocumentsContract.Document.FLAG_SUPPORTS_RENAME` and `FLAG_SUPPORTS_MOVE`, the Android SDK values.
const FLAG_SUPPORTS_RENAME := 64
const FLAG_SUPPORTS_MOVE := 256

var driver


func _init(p_driver = null) -> void:
	driver = p_driver


## The dispatcher entry point Rust calls. Any op the handler does not know answers `ok: false`, so an
## older Rust build talking to this one, or the reverse, fails loudly instead of reading as "no such file".
func _saf_dispatch(op: String, args: Array) -> Dictionary:
	if driver == null:
		return _fail("no Android storage driver is connected")
	match op:
		"available":
			var a: Dictionary = driver.available()
			return {"ok": true, "value": bool(a.get("ok", false)) and bool(a.get("value", false))}
		"list":
			return _op_list(int(args[0]) if args.size() > 0 else 0)
		"read":
			return _op_read(String(args[0]))
		"meta":
			return _op_meta(String(args[0]))
		"exists":
			var r := _resolve(String(args[0]))
			return {"ok": true, "value": bool(r.get("ok", false)) and not bool(r.get("is_dir", false))}
		"write":
			return _op_write(String(args[0]), String(args[1]))
		"can_move":
			var c: Dictionary = driver.capabilities()
			return {"ok": true, "value": bool(c.get("ok", false)) and bool(c.get("create", false)) and bool(c.get("delete", false))}
		"move":
			return _op_move(String(args[0]), String(args[1]))
		"move_dir":
			return _op_move_dir(String(args[0]), String(args[1]))
	return _fail("unknown op: %s" % op)


static func _fail(msg: String) -> Dictionary:
	return {"ok": false, "error": msg}


## `rel` split into non-empty segments. The Rust side has already refused `..` and absolute paths
## (`is_safe_relative_path`); a stray empty segment is dropped rather than treated as a name.
static func _segments(rel: String) -> PackedStringArray:
	var out := PackedStringArray()
	for s in rel.split("/", false):
		out.append(s)
	return out


## Walks `rel` from the root. `{ok, id, parent_id, is_dir, flags}`; `ok: false` with `missing: true`
## when a segment is not there (so a caller can tell absent from unreachable).
func _resolve(rel: String) -> Dictionary:
	var segs := _segments(rel)
	var root: Dictionary = driver.root_id()
	if not bool(root.get("ok", false)):
		return root
	var cur := String(root["id"])
	var parent := ""
	var is_dir := true
	var flags := 0
	for seg in segs:
		var kids: Dictionary = driver.children(cur)
		if not bool(kids.get("ok", false)):
			return kids
		var found := false
		for it in kids["items"]:
			if String(it["name"]) == seg:
				parent = cur
				cur = String(it["id"])
				is_dir = bool(it["is_dir"])
				flags = int(it.get("flags", 0))
				found = true
				break
		if not found:
			return {"ok": false, "missing": true, "error": "not found: %s" % rel}
	return {"ok": true, "id": cur, "parent_id": parent, "is_dir": is_dir, "flags": flags}


## Makes every folder of `segs` (missing ones created), answering the last folder's id.
func _ensure_dirs(segs: PackedStringArray) -> Dictionary:
	var root: Dictionary = driver.root_id()
	if not bool(root.get("ok", false)):
		return root
	var cur := String(root["id"])
	for seg in segs:
		var kids: Dictionary = driver.children(cur)
		if not bool(kids.get("ok", false)):
			return kids
		var next := ""
		for it in kids["items"]:
			if String(it["name"]) == seg:
				if not bool(it["is_dir"]):
					return _fail("%s is a file, not a folder" % seg)
				next = String(it["id"])
				break
		if next == "":
			var made: Dictionary = driver.create(cur, seg, true)
			if not bool(made.get("ok", false)):
				return made
			next = String(made["id"])
		cur = next
	return {"ok": true, "id": cur}


func _op_list(limit: int) -> Dictionary:
	var out := PackedStringArray()
	var root: Dictionary = driver.root_id()
	if not bool(root.get("ok", false)):
		return root
	var stack: Array = [[String(root["id"]), ""]]
	while not stack.is_empty():
		var top: Array = stack.pop_back()
		var kids: Dictionary = driver.children(String(top[0]))
		if not bool(kids.get("ok", false)):
			return kids
		var items: Array = kids["items"].duplicate()
		items.sort_custom(func(x, y): return String(x["name"]) < String(y["name"]))
		for it in items:
			var name := String(it["name"])
			if name.begins_with("."):
				continue
			var rel := (String(top[1]) + "/" + name) if String(top[1]) != "" else name
			if bool(it["is_dir"]):
				stack.append([String(it["id"]), rel])
			elif name.to_lower().ends_with(".md"):
				out.append(rel)
				if limit > 0 and out.size() >= limit:
					return {"ok": true, "value": out}
	out.sort()
	return {"ok": true, "value": out}


func _op_read(rel: String) -> Dictionary:
	var r := _resolve(rel)
	if not bool(r.get("ok", false)) or bool(r.get("is_dir", false)):
		return _fail(String(r.get("error", "not a file: %s" % rel)))
	var t: Dictionary = driver.read_text(String(r["id"]))
	if not bool(t.get("ok", false)):
		return t
	return {"ok": true, "value": String(t["text"])}


func _op_meta(rel: String) -> Dictionary:
	var r := _resolve(rel)
	if not bool(r.get("ok", false)):
		return r
	var s: Dictionary = driver.stat(String(r["id"]))
	if not bool(s.get("ok", false)):
		return s
	return {"ok": true, "modified": int(s.get("modified", 0)), "len": int(s.get("len", 0))}


## Writes, creating folders and the file as needed, then **reads it back**: a provider that ignores the
## truncate flag leaves the tail of a longer old file behind, and "ok" from the framework does not say so.
func _op_write(rel: String, text: String) -> Dictionary:
	var segs := _segments(rel)
	if segs.is_empty():
		return _fail("no file name")
	var name := segs[segs.size() - 1]
	segs.remove_at(segs.size() - 1)
	var dir := _ensure_dirs(segs)
	if not bool(dir.get("ok", false)):
		return dir
	var existing := _resolve(rel)
	var id := ""
	if bool(existing.get("ok", false)):
		if bool(existing.get("is_dir", false)):
			return _fail("%s is a folder" % rel)
		id = String(existing["id"])
	else:
		var made: Dictionary = driver.create(String(dir["id"]), name, false)
		if not bool(made.get("ok", false)):
			return made
		id = String(made["id"])
	var w: Dictionary = driver.write_text(id, text)
	if not bool(w.get("ok", false)):
		return w
	var back: Dictionary = driver.read_text(id)
	if not bool(back.get("ok", false)) or String(back.get("text", "")) != text:
		return _fail("the written note does not read back identically (%s); the provider may not truncate" % rel)
	return {"ok": true}


## VE-6's Android half. The ladder, best first, each step reported in `method`:
##   "rename"      same folder and the provider says it can rename (one call, atomic on the provider's side)
##   "move"        same file name, different folder, and the provider says it can move
##   "copy_delete" everything else: create the destination, copy the text, read it back, delete the original
## A provider call that fails drops to the next rung rather than failing the rename. The one state that is
## not a clean success is a copy whose original could not be deleted: that answers `ok: false` naming both
## paths, so a person is told two copies exist instead of seeing "renamed".
func _op_move(from: String, to: String) -> Dictionary:
	var src := _resolve(from)
	if not bool(src.get("ok", false)):
		return _fail("cannot move %s: %s" % [from, String(src.get("error", "not found"))])
	if bool(src.get("is_dir", false)):
		return _fail("moving a folder is not supported here (%s)" % from)
	if bool(_resolve(to).get("ok", false)):
		return _fail("%s already exists; a move never overwrites" % to)
	var from_segs := _segments(from)
	var to_segs := _segments(to)
	var to_name := to_segs[to_segs.size() - 1]
	var from_dir := "/".join(from_segs.slice(0, from_segs.size() - 1))
	var to_dir_segs := to_segs.slice(0, to_segs.size() - 1)
	var same_folder := from_dir == "/".join(to_dir_segs)
	var flags := int(src.get("flags", 0))
	var id := String(src["id"])
	if same_folder and (flags & FLAG_SUPPORTS_RENAME) != 0:
		var rn: Dictionary = driver.rename(id, to_name)
		if bool(rn.get("ok", false)) and bool(_resolve(to).get("ok", false)) and not bool(_resolve(from).get("ok", false)):
			return {"ok": true, "method": "rename"}
	var dir := _ensure_dirs(to_dir_segs)
	if not bool(dir.get("ok", false)):
		return _fail("cannot create the destination folder for %s: %s" % [to, String(dir.get("error", ""))])
	if not same_folder and (flags & FLAG_SUPPORTS_MOVE) != 0 and to_name == String(from_segs[from_segs.size() - 1]):
		var mv: Dictionary = driver.move(id, String(src["parent_id"]), String(dir["id"]))
		if bool(mv.get("ok", false)) and bool(_resolve(to).get("ok", false)) and not bool(_resolve(from).get("ok", false)):
			return {"ok": true, "method": "move"}
	return _copy_delete(from, to, to_name, id, String(dir["id"]))


func _copy_delete(from: String, to: String, to_name: String, src_id: String, dst_dir_id: String) -> Dictionary:
	var rd: Dictionary = driver.read_text(src_id)
	if not bool(rd.get("ok", false)):
		return _fail("cannot read %s to copy it: %s" % [from, String(rd.get("error", ""))])
	var text := String(rd["text"])
	var made: Dictionary = driver.create(dst_dir_id, to_name, false)
	if not bool(made.get("ok", false)):
		return _fail("cannot create %s: %s" % [to, String(made.get("error", ""))])
	var new_id := String(made["id"])
	var w: Dictionary = driver.write_text(new_id, text)
	var back: Dictionary = driver.read_text(new_id) if bool(w.get("ok", false)) else {}
	if not bool(back.get("ok", false)) or String(back.get("text", "")) != text:
		driver.delete(new_id)   ## best effort: a half-written copy must not be left beside the original
		return _fail("the copy of %s did not verify; the original is untouched" % from)
	var del: Dictionary = driver.delete(src_id)
	if not bool(del.get("ok", false)):
		return _fail("%s was copied to %s but the original could not be deleted (%s); both now exist" % [from, to, String(del.get("error", "the provider refused"))])
	return {"ok": true, "method": "copy_delete"}


## The folder form of `_op_move`: renames or moves the folder `from` and everything in it to `to`. The same
## ladder, best first, each step reported in `method`:
##   "rename"      same parent and the provider says it can rename the folder
##   "move"        same folder name, different parent, and the provider says it can move
##   "copy_delete" everything else: copy every note into a new folder tree, read each back, delete the original
## A rung that fails drops to the next, as for a note. The copy rung can only carry text, so a folder holding
## anything that is not a `.md` note is refused BEFORE a byte is copied (a rename or move rung has no such limit).
## `to` existing is refused, a folder is never merged. The one state that is not a clean success is a copy whose
## original folder could not be deleted: `ok: false` naming both places, so a person is told two copies exist.
func _op_move_dir(from: String, to: String) -> Dictionary:
	var src := _resolve(from)
	if not bool(src.get("ok", false)):
		return _fail("cannot move %s: %s" % [from, String(src.get("error", "not found"))])
	if not bool(src.get("is_dir", false)):
		return _fail("%s is not a folder" % from)
	if bool(_resolve(to).get("ok", false)):
		return _fail("%s already exists; a folder move never merges" % to)
	var from_segs := _segments(from)
	var to_segs := _segments(to)
	var to_name := to_segs[to_segs.size() - 1]
	var to_parent_segs := to_segs.slice(0, to_segs.size() - 1)
	var same_parent := "/".join(from_segs.slice(0, from_segs.size() - 1)) == "/".join(to_parent_segs)
	var flags := int(src.get("flags", 0))
	var id := String(src["id"])
	if same_parent and (flags & FLAG_SUPPORTS_RENAME) != 0:
		var rn: Dictionary = driver.rename(id, to_name)
		if bool(rn.get("ok", false)) and bool(_resolve(to).get("ok", false)) and not bool(_resolve(from).get("ok", false)):
			return {"ok": true, "method": "rename"}
	var parent := _ensure_dirs(to_parent_segs)
	if not bool(parent.get("ok", false)):
		return _fail("cannot create the destination folder for %s: %s" % [to, String(parent.get("error", ""))])
	if not same_parent and (flags & FLAG_SUPPORTS_MOVE) != 0 and to_name == String(from_segs[from_segs.size() - 1]):
		var mv: Dictionary = driver.move(id, String(src["parent_id"]), String(parent["id"]))
		if bool(mv.get("ok", false)) and bool(_resolve(to).get("ok", false)) and not bool(_resolve(from).get("ok", false)):
			return {"ok": true, "method": "move"}
	return _copy_delete_dir(from, to, id, String(parent["id"]), to_name)


## Every file under the folder `dir_id` as `[relative path, id]` pairs, or `{ok: false}` when a listing fails.
## `rel` is the path from the folder being copied. Folders are returned as `[rel, id, true]`, parents first.
func _walk_dir(dir_id: String, rel: String, out: Array) -> Dictionary:
	var kids: Dictionary = driver.children(dir_id)
	if not bool(kids.get("ok", false)):
		return kids
	for it in kids["items"]:
		var r := (rel + "/" + String(it["name"])) if rel != "" else String(it["name"])
		out.append([r, String(it["id"]), bool(it["is_dir"])])
		if bool(it["is_dir"]):
			var sub := _walk_dir(String(it["id"]), r, out)
			if not bool(sub.get("ok", true)):
				return sub
	return {"ok": true}


func _copy_delete_dir(from: String, to: String, src_id: String, dst_parent_id: String, to_name: String) -> Dictionary:
	var entries: Array = []
	var walked := _walk_dir(src_id, "", entries)
	if not bool(walked.get("ok", false)):
		return _fail("cannot list %s to copy it: %s" % [from, String(walked.get("error", ""))])
	var others := 0
	for e in entries:
		if not bool(e[2]) and not String(e[0]).to_lower().ends_with(".md"):
			others += 1
	if others > 0:
		return _fail("%s holds %d file%s that are not notes, and this storage can only copy notes, so the folder was not moved (nothing was changed)" % [from, others, "" if others == 1 else "s"])
	var made: Dictionary = driver.create(dst_parent_id, to_name, true)
	if not bool(made.get("ok", false)):
		return _fail("cannot create %s: %s" % [to, String(made.get("error", ""))])
	var new_root := String(made["id"])
	var ids := {"": new_root}   ## relative folder path -> its id in the copy
	for e in entries:
		var rel := String(e[0])
		var parent_rel := "/".join(PackedStringArray(rel.split("/").slice(0, -1)))
		var name := rel.get_file()
		if bool(e[2]):
			var d: Dictionary = driver.create(String(ids[parent_rel]), name, true)
			if not bool(d.get("ok", false)):
				driver.delete(new_root)   ## best effort: a half copy must not be left beside the original
				return _fail("cannot create the folder %s in the copy: %s; the original is untouched" % [rel, String(d.get("error", ""))])
			ids[rel] = String(d["id"])
			continue
		var rd: Dictionary = driver.read_text(String(e[1]))
		var f: Dictionary = driver.create(String(ids[parent_rel]), name, false) if bool(rd.get("ok", false)) else {}
		var w: Dictionary = driver.write_text(String(f["id"]), String(rd["text"])) if bool(f.get("ok", false)) else {}
		var back: Dictionary = driver.read_text(String(f["id"])) if bool(w.get("ok", false)) else {}
		if not bool(back.get("ok", false)) or String(back.get("text", "")) != String(rd.get("text", "")):
			driver.delete(new_root)
			return _fail("the copy of %s did not verify; the original folder is untouched" % rel)
	var del: Dictionary = driver.delete(src_id)
	if not bool(del.get("ok", false)):
		return _fail("%s was copied to %s but the original folder could not be deleted (%s); both now exist" % [from, to, String(del.get("error", "the provider refused"))])
	return {"ok": true, "method": "copy_delete"}
