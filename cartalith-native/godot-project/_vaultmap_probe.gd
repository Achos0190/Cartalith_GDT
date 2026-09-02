extends Node
## Committed verification harness for `MARKDOWN_VAULT_SCOPE.md` **milestone 2**
## (the §21/§22 map snapshot) and **milestone 3** (§26's project-scoped links).
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _vaultmap_probe.tscn
##
## Both milestones land in the same place — `LinkStore`, which is what
## `project_bridge.rs` writes into the archive's `vault.json` — so they are
## one probe rather than two that would each have to generate a world.
##
## What it asserts, in the order a failure would matter:
##
##   1. Milestone 2, the offer: the three Map fields are **absent** from
##      `vault_export_fields` until a snapshot exists, which is §20's "must
##      not expose information the entity does not possess" enforced by the
##      data. This is the check that catches a Map field registered with the
##      wrong `kinds` or an availability test that always answers yes.
##   2. Milestone 2, the write: `vault_snapshot` puts a real PNG of the
##      requested size at a vault-**relative** path inside the accepted
##      folder, and regenerating replaces that one file rather than growing a
##      folder of orphans.
##   3. Milestone 2, containment: a folder that escapes the vault is refused
##      and writes nothing — §22's "must not silently pollute the Markdown
##      vault", and `FsVault::resolve`'s own rule reached through this path.
##   4. Milestone 2, the note: the block body carries the image under §19's
##      **Map** header, as a relative Markdown image and never a base64 blob
##      (§22).
##   5. Milestone 3, the archive: a real `project_save_with_documents` writes
##      a `vault.json` that carries both the link and the snapshot, read back
##      out of the `.zip` here rather than trusted. **This is the whole of
##      §26** and the claim `MARKDOWN_VAULT_SCOPE.md` filed as blocked.
##   6. Milestone 3, the sidecar: `VaultStore.save_from` keeps writing the
##      links with no project open and drops them with one open — and the
##      pre-project sidecar is copied aside, once, before that first drop.
##
## Committed, like every probe scene in this folder — `STATUS.md`'s F8 row:
## these are kept as the evidence for the passes that wrote them.

const SEED := 483920
const SNAP_PX := 256

var _app: Node
var _bridge
var _root := ""
var _fails: Array = []
## The real profile's vault files, put back exactly as found (this probe runs
## against the same `user://` a real session uses).
var _saved: Dictionary = {}


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("VMAP     OK  %s" % label)
	else:
		_fails.append(label)
		print("VMAP     !!  %s   %s" % [label, detail])


func _read(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return ""
	var s := f.get_as_text()
	f.close()
	return s


func _bytes(path: String) -> PackedByteArray:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return PackedByteArray()
	var b := f.get_buffer(f.get_length())
	f.close()
	return b


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


## Files directly inside one folder, sorted. Used to assert that regenerating
## a snapshot replaces a file rather than adding one.
func _dir_files(abs: String) -> PackedStringArray:
	var d := DirAccess.open(abs)
	if d == null:
		return PackedStringArray()
	var out := d.get_files()
	out.sort()
	return out


func _generate() -> void:
	## Small on purpose: this probe is about the crop's arithmetic and the
	## store, not about terrain, and every second here is paid on every run.
	_bridge.generate({
		"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
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

	## Not a SKIP, for `_vaultprefs_probe.gd`'s reason: `vault_snapshot` is
	## bound today, so the only way to reach this branch is a `.dll` older than
	## the shell — the condition that has twice made a whole verification pass
	## meaningless here. Exit 2, "could not run", never 0.
	if not _bridge.world_gen.has_method("vault_snapshot"):
		print("VMAP     ABORT: vault_snapshot absent -- the loaded extension predates "
			+ "the snapshot surface; rebuild before believing this probe")
		get_tree().quit(2)
		return

	_stash(VaultStore.PATH)
	_stash(VaultStore.PRE_PROJECT_PATH)
	await _generate()

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultmap-probe"
	## A clean vault every run: a leftover `.cartalith/maps` from the last run
	## would make check 2's "regenerating replaces one file" pass for the wrong
	## reason. Guarded, because `move_to_trash` on a path that is not there
	## prints `SHFileOperation error: 2` and a probe that cries wolf on a
	## first run teaches people to read past its output.
	if DirAccess.dir_exists_absolute(_root):
		OS.move_to_trash(ProjectSettings.globalize_path(_root))
	DirAccess.make_dir_recursive_absolute(_root + "/Locations")
	var f := FileAccess.open(_root + "/Locations/Nareth.md", FileAccess.WRITE)
	f.store_string("# Nareth\n\nA river town at the third ford.\n")
	f.close()

	var conn: Dictionary = _bridge.vault_connect(_root, "MapProbeVault")
	_ok("connect: the scratch vault binds", bool(conn.get("ok", false)), String(conn.get("error", "")))

	var settlements: Array = _bridge.settlements()
	if settlements.is_empty():
		_ok("world: settlements exist to snapshot", false)
		_finish()
		return
	var s: Dictionary = settlements[0]
	var tid := int(s.get("tid", 0))
	var sname := String(s.get("name", ""))

	# -- 1. The radii, and the offer before any snapshot exists --------------
	var radii: Array = _bridge.vault_snapshot_radii("settlement", tid)
	_ok("radii: §21's three are offered", radii.size() == 3, str(radii))
	var scaled := true
	var names := PackedStringArray()
	for r in radii:
		var d: Dictionary = r
		names.append(String(d.get("radius", "")))
		if int(d.get("cells", 0)) < 1:
			scaled = false
		if String(d.get("path", "")) != "":
			scaled = false
	_ok("radii: immediate/local/regional, in that order",
		Array(names) == ["immediate", "local", "regional"], str(names))
	_ok("radii: each scales to a real cell count and none is generated yet", scaled, str(radii))

	_ok("offer: no Map field before a snapshot exists", not _offers(tid, "map_local"),
		str(_field_keys(tid)))

	# -- 2. The write --------------------------------------------------------
	var snap: Dictionary = _bridge.vault_snapshot("settlement", tid, "local", "", SNAP_PX)
	_ok("snapshot: the local map is written", bool(snap.get("ok", false)), String(snap.get("error", "")))
	var rel := String(snap.get("rel", ""))
	_ok("snapshot: the path is vault-relative and inside §22's proposed folder",
		rel.begins_with(".cartalith/maps/") and rel.ends_with(".png") and rel.find("..") < 0, rel)
	var abs_png := _root + "/" + rel
	_ok("snapshot: the file is on disk", FileAccess.file_exists(abs_png), abs_png)
	var img := Image.new()
	var loaded := img.load(abs_png)
	_ok("snapshot: it is a real image at the size asked for",
		loaded == OK and img.get_width() == SNAP_PX and img.get_height() == SNAP_PX,
		"err=%d %dx%d" % [loaded, img.get_width(), img.get_height()])
	## The crop must be a *crop*: an image of the whole world would be uniform
	## at this scale only by accident, but a crop of ~1/10th of it must still
	## differ from the full-world render. Cheapest honest check is that the
	## picture is not a single flat colour, which is what a broken window
	## (`x0` clamped to zero size, or an empty `bake_rect`) produces.
	if loaded == OK:
		var a := img.get_pixel(4, 4)
		var b := img.get_pixel(SNAP_PX - 5, SNAP_PX - 5)
		var c := img.get_pixel(SNAP_PX / 2, SNAP_PX / 2)
		_ok("snapshot: it is a rendered map, not a flat fill", a != b or a != c, "%s %s %s" % [a, b, c])
	_ok("snapshot: it reports the cells it actually covered",
		float(snap.get("cells_across", 0.0)) > 1.0, str(snap))

	var maps_dir := _root + "/.cartalith/maps"
	var before := _dir_files(maps_dir)
	var again: Dictionary = _bridge.vault_snapshot("settlement", tid, "local", "", SNAP_PX)
	_ok("snapshot: regenerating replaces the file the note points at",
		bool(again.get("ok", false)) and String(again.get("rel", "")) == rel
		and _dir_files(maps_dir) == before, "%s -> %s" % [before, _dir_files(maps_dir)])

	# -- 3. Containment and refusals ----------------------------------------
	var escape: Dictionary = _bridge.vault_snapshot("settlement", tid, "immediate", "../outside", SNAP_PX)
	_ok("refuse: a folder that escapes the vault is refused",
		not bool(escape.get("ok", true)), String(escape.get("error", "")))
	_ok("refuse: and nothing was written outside the vault",
		not DirAccess.dir_exists_absolute(_root + "/../outside"))
	var bad_radius: Dictionary = _bridge.vault_snapshot("settlement", tid, "planetary", "", SNAP_PX)
	_ok("refuse: an unknown radius is refused rather than rounded",
		not bool(bad_radius.get("ok", true)), String(bad_radius.get("error", "")))
	var culture: Dictionary = _bridge.vault_snapshot("culture", 0, "local", "", SNAP_PX)
	_ok("refuse: a culture has no position, so it gets no map",
		not bool(culture.get("ok", true)), String(culture.get("error", "")))

	# -- 4. The note ---------------------------------------------------------
	_ok("offer: the generated radius is now offered", _offers(tid, "map_local"), str(_field_keys(tid)))
	_ok("offer: and the two that were not generated still are not",
		not _offers(tid, "map_immediate") and not _offers(tid, "map_regional"), str(_field_keys(tid)))
	var values: Dictionary = _bridge.vault_entity_values("settlement", tid)
	_ok("note: the value is a relative Markdown image, not a base64 blob",
		String(values.get("map_local", "")) == "![](%s)" % rel, String(values.get("map_local", "")))
	var body: String = _bridge.vault_block_body("settlement", tid, PackedStringArray(["name", "map_local"]))
	_ok("note: the block puts it under §19's Map header",
		body.find("**Map**") >= 0 and body.find("- Local map: ![](%s)" % rel) >= 0, body)

	## **The two ways the crop arithmetic can be silently wrong**, and the only
	## checks here that catch them. Everything above passes just as happily if
	## `vault_snapshot` ignores its radius and its centre and renders the whole
	## world every time — the file exists, it is 256², it is not flat, and the
	## note links it.
	##
	##   - **zoom**: three radii of one place must be three different pictures.
	##     A dropped `out_w` term makes them byte-identical.
	##   - **centre**: one radius of two places must be two different pictures.
	##     A dropped `x0`/`y0` makes them byte-identical.
	var wide: Dictionary = _bridge.vault_snapshot("settlement", tid, "regional", "", SNAP_PX)
	var tight: Dictionary = _bridge.vault_snapshot("settlement", tid, "immediate", "", SNAP_PX)
	_ok("crop: the other two radii write too",
		bool(wide.get("ok", false)) and bool(tight.get("ok", false)),
		"%s / %s" % [String(wide.get("error", "")), String(tight.get("error", ""))])
	var px_local := _bytes(abs_png)
	var px_wide := _bytes(_root + "/" + String(wide.get("rel", "")))
	var px_tight := _bytes(_root + "/" + String(tight.get("rel", "")))
	_ok("crop: a radius is a real zoom -- three radii are three pictures",
		px_local.size() > 0 and px_local != px_wide and px_local != px_tight and px_wide != px_tight,
		"%d / %d / %d bytes" % [px_tight.size(), px_local.size(), px_wide.size()])
	_ok("crop: a wider radius covers more cells",
		float(tight.get("cells_across", 0.0)) < float(snap.get("cells_across", 0.0))
		and float(snap.get("cells_across", 0.0)) < float(wide.get("cells_across", 0.0)),
		"%s / %s / %s" % [tight.get("cells_across", 0), snap.get("cells_across", 0), wide.get("cells_across", 0)])
	if settlements.size() > 1:
		var tid2 := int((settlements[1] as Dictionary).get("tid", 0))
		var other: Dictionary = _bridge.vault_snapshot("settlement", tid2, "local", "", SNAP_PX)
		_ok("crop: it is centred on the entity -- another settlement is another picture",
			bool(other.get("ok", false)) and _bytes(_root + "/" + String(other.get("rel", ""))) != px_local,
			String(other.get("error", "")))

	# -- 5. Milestone 3: the archive is the store ----------------------------
	var att: Dictionary = _bridge.vault_attach("settlement", tid, sname, "Locations/Nareth.md", "")
	_ok("archive: a link exists to travel with the project", bool(att.get("ok", false)), String(att.get("error", "")))
	var zip_path := OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultmap-probe.zip"
	var wrote: Dictionary = _bridge.project_save_with_documents(zip_path, {})
	_ok("archive: the project writes", bool(wrote.get("ok", false)), String(wrote.get("error", "")))
	var zr := ZIPReader.new()
	if zr.open(zip_path) != OK:
		_ok("archive: the .zip opens", false, zip_path)
	else:
		var doc := zr.read_file("vault.json").get_string_from_utf8()
		zr.close()
		var parsed = JSON.parse_string(doc)
		_ok("archive: vault.json is in the project tree", typeof(parsed) == TYPE_DICTIONARY, doc.substr(0, 200))
		if typeof(parsed) == TYPE_DICTIONARY:
			var vd: Dictionary = parsed
			_ok("archive: it carries the link (§26, the milestone-3 claim)",
				(vd.get("links", []) as Array).size() == 1, doc.substr(0, 400))
			var snaps: Dictionary = vd.get("snapshots", {})
			_ok("archive: and the snapshot path travels with it",
				String(snaps.get("settlement:%d|local" % tid, "")) == rel, str(snaps))
	DirAccess.remove_absolute(zip_path)

	# -- 6. Milestone 3: the sidecar's new rule ------------------------------
	VaultStore.save_from(_bridge, false)
	var side = JSON.parse_string(_read(VaultStore.PATH))
	_ok("sidecar: with no project open it still carries the links",
		typeof(side) == TYPE_DICTIONARY and (side as Dictionary).has("store")
		and String((side as Dictionary).get("binding", "")) != "", str(side).substr(0, 200))
	var pre_project := _read(VaultStore.PATH)

	VaultStore.save_from(_bridge, true)
	side = JSON.parse_string(_read(VaultStore.PATH))
	_ok("sidecar: with a project open the links are dropped and the binding stays",
		typeof(side) == TYPE_DICTIONARY and not (side as Dictionary).has("store")
		and String((side as Dictionary).get("binding", "")) != "", str(side).substr(0, 200))
	_ok("sidecar: the pre-project store was kept, byte for byte",
		FileAccess.file_exists(VaultStore.PRE_PROJECT_PATH)
		and _read(VaultStore.PRE_PROJECT_PATH) == pre_project,
		_read(VaultStore.PRE_PROJECT_PATH).substr(0, 200))

	## The move is one-way and happens once: a second project-scoped write must
	## not overwrite the backup with the already-emptied document, which would
	## destroy the very thing the copy exists to keep.
	VaultStore.save_from(_bridge, true)
	_ok("sidecar: the backup is written once and never rewritten",
		_read(VaultStore.PRE_PROJECT_PATH) == pre_project,
		_read(VaultStore.PRE_PROJECT_PATH).substr(0, 200))

	## Regression, 2026-09-02. Milestone 2 added `snapshots` to `LinkStore` and
	## added it to none of the places `links` is guarded, so a snapshot filed
	## against world A's `settlement:1` survived a regenerate into world B --
	## `export::offer` then put a Map checkbox in front of B's settlement and a
	## project save wrote A's image path into B's `vault.json`. `links` was
	## cleared correctly all along, which is exactly why the leak was invisible:
	## the guard existed and the new member simply was not added to it.
	##
	## Assert the pair together. A future member of `LinkStore` that is world
	## state belongs in the same clear, and this is where that gets caught.
	_bridge.generate({
		"seed": 777001, "width_km": 900.0, "grid_w": 128, "grid_h": 96,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().process_frame
	await get_tree().process_frame
	_ok("cross-world: the link from the previous world was cleared",
		not _offers(tid, "note"), "a stale link survived a regenerate")
	_ok("cross-world: and so was the SNAPSHOT (the leak this pins)",
		not _offers(tid, "map_local"), "world A's snapshot survived into world B")

	_finish()


## Whether `key` is in the entity's offer list — the §20 filter, asked through
## the same `#[func]` the panel uses.
func _offers(tid: int, key: String) -> bool:
	for fd in _bridge.vault_export_fields("settlement", tid):
		if String((fd as Dictionary).get("key", "")) == key:
			return true
	return false


func _field_keys(tid: int) -> PackedStringArray:
	var out := PackedStringArray()
	for fd in _bridge.vault_export_fields("settlement", tid):
		out.append(String((fd as Dictionary).get("key", "")))
	return out


func _finish() -> void:
	_restore()
	if _fails.is_empty():
		print("VMAP     ALL CHECKS PASSED")
	else:
		print("VMAP     %d FAILED: %s" % [_fails.size(), ", ".join(PackedStringArray(_fails))])
	get_tree().quit(0 if _fails.is_empty() else 1)
