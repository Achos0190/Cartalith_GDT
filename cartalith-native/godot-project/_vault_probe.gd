extends Node
## Committed verification harness for `MARKDOWN_VAULT_SCOPE.md`
## milestones 0 and 1.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vault_probe.tscn
##
## Drives the **real app through the real shell** against a **real folder of
## real Markdown files on disk**, because that is the only thing that proves
## the claim this subsystem makes: *a write touches one section and leaves a
## hand-authored note otherwise byte-identical.* The Rust tests assert that
## against a temp directory; this asserts it end to end, through the
## GDExtension boundary, with the shell's own panels open.
##
##   1. Boot, generate a world.
##   2. Milestone 0: continents exist, are ranked by area, and are named.
##   3. Write a hand-authored note to a scratch vault; connect it.
##   4. Attach a real settlement (by tid) to one section of that note.
##   5. Edit the working copy, preview, write back — assert on disk that the
##      section changed and every other byte did not.
##   6. Write the Cartalith block, update it, and assert the same.
##   7. Fill the author's own template fields — assert the filled one is
##      skipped and the empty one is written.
##   8. Assert the stale guard: edit the file behind Cartalith's back and
##      confirm the write refuses and changes nothing.
##   9. Assert the panels wired up: the place editor's KNOWLEDGE section and
##      the Civilization dock's Linked notes rows.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

const SEED := 483920

var _app: Node
var _bridge
var _root := ""
var _note := ""
var _tmpl := ""
var _fails: Array = []

## The hand-authored note. Every one of these lines is asserted intact after
## every write below.
const HAND := """---
tags: [worldbuilding]
---

# Nareth

A river town at the third ford. The author wrote this sentence by hand.

## History

Founded in the third age by the Ashfall clans.

## The Old Quarter

Narrow streets, older than the walls.

## Trade

Grain downriver, salt up.
"""

const TEMPLATE := """## Settlement Profile: [Name]

**Type:** [City-State / City / Town]
**Location:** Riverbend

### General Info

- **Size / Population:**
"""


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("VAULT    OK  %s" % label)
	else:
		_fails.append(label)
		print("VAULT    !!  %s   %s" % [label, detail])


func _read(path: String) -> String:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return ""
	var s := f.get_as_text()
	f.close()
	return s


func _write(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _texts(n: Node, out: Array) -> Array:
	if n is Label:
		out.append(String((n as Label).text))
	elif n is Button:
		out.append(String((n as Button).text))
	for c in n.get_children():
		_texts(c, out)
	return out


func _find(n: Node, cls) -> Node:
	if is_instance_of(n, cls):
		return n
	for c in n.get_children():
		var r := _find(c, cls)
		if r != null:
			return r
	return null


func _generate() -> void:
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
	await _generate()

	# -- §1 milestone 0: continents ----------------------------------------
	var cont: Array = _bridge.continents()
	_ok("continents: at least one landmass listed", cont.size() > 0, "got %d" % cont.size())
	if cont.size() > 0:
		var ranked := true
		var named := true
		for i in cont.size():
			var d: Dictionary = cont[i]
			if int(d.get("id", 0)) != i + 1:
				ranked = false
			if String(d.get("name", "")) == "":
				named = false
			if i > 0 and int(d.get("cells", 0)) > int((cont[i - 1] as Dictionary).get("cells", 0)):
				ranked = false
		_ok("continents: ids are 1..n by descending area", ranked)
		_ok("continents: every one is named", named)
		var c0: Dictionary = cont[0]
		_ok("continents: the biggest one has a real bounding box",
			int(c0.get("max_x", 0)) > int(c0.get("min_x", 0)) and int(c0.get("max_y", 0)) > int(c0.get("min_y", 0)),
			str(c0))
		print("VAULT    ... biggest continent: %s, %d cells, box (%d,%d)-(%d,%d)" % [
			String(c0.get("name", "")), int(c0.get("cells", 0)),
			int(c0.get("min_x", 0)), int(c0.get("min_y", 0)),
			int(c0.get("max_x", 0)), int(c0.get("max_y", 0))])

	# -- §2 a real vault on disk -------------------------------------------
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vault-probe"
	DirAccess.make_dir_recursive_absolute(_root + "/Locations")
	_note = _root + "/Locations/Nareth.md"
	_tmpl = _root + "/Locations/Template.md"
	_write(_note, HAND)
	_write(_tmpl, TEMPLATE)

	var conn: Dictionary = _bridge.vault_connect(_root, "ProbeVault")
	_ok("connect: a real folder binds", bool(conn.get("ok", false)), String(conn.get("error", "")))
	var files: PackedStringArray = _bridge.vault_list_files(100)
	_ok("browse: both .md files listed", files.size() == 2 and Array(files).has("Locations/Nareth.md"), str(files))
	var heads: Array = _bridge.vault_file_headings("Locations/Nareth.md")
	_ok("browse: four headings parsed", heads.size() == 4, str(heads))

	# -- §3 attach a real settlement ---------------------------------------
	var settlements: Array = _bridge.settlements()
	_ok("world: settlements exist to link", settlements.size() > 0)
	if settlements.is_empty():
		_finish()
		return
	var s: Dictionary = settlements[0]
	var tid := int(s.get("tid", 0))
	var sname := String(s.get("name", ""))
	_ok("world: the settlement carries a stable tid", tid != 0)

	var att: Dictionary = _bridge.vault_attach("settlement", tid, sname, "Locations/Nareth.md", "The Old Quarter")
	_ok("attach: a section link is created", bool(att.get("ok", false)), String(att.get("error", "")))
	var link := String(att.get("link_id", ""))
	var imported: String = _bridge.vault_link_text(link)
	_ok("attach: the imported text is the section, heading included",
		imported.begins_with("## The Old Quarter") and imported.find("Narrow streets") >= 0, imported)
	var bad: Dictionary = _bridge.vault_attach("settlement", tid, sname, "Locations/Nareth.md", "Nope")
	_ok("attach: a section that does not exist is refused", not bool(bad.get("ok", true)), String(bad.get("error", "")))

	var links: Array = _bridge.vault_links_for("settlement", tid)
	_ok("status: the fresh link is Connected", links.size() == 1
		and String((links[0] as Dictionary).get("status", "")) == "connected", str(links))

	# -- §4 the section write-back -----------------------------------------
	_bridge.vault_set_link_text(link, "## The Old Quarter\n\nRebuilt after the fire of 812.\n")
	links = _bridge.vault_links_for("settlement", tid)
	_ok("status: an edited working copy reports local_changes",
		String((links[0] as Dictionary).get("status", "")) == "local_changes", str(links))

	var prev: Dictionary = _bridge.vault_preview_section_write(link)
	_ok("preview: a section write previews", bool(prev.get("ok", false)), String(prev.get("error", "")))
	var wr: Dictionary = _bridge.vault_write_section(link, String(prev.get("hash", "")))
	_ok("write: the section is written back", bool(wr.get("ok", false)), String(wr.get("error", "")))

	var disk := _read(_note)
	_ok("write: what was previewed is exactly what landed", disk == String(prev.get("preview", "")))
	_ok("write: the section changed", disk.find("Rebuilt after the fire of 812.") >= 0)
	_ok("write: the old section text is gone", disk.find("Narrow streets") < 0)
	for kept in ["tags: [worldbuilding]", "# Nareth",
			"A river town at the third ford. The author wrote this sentence by hand.",
			"## History", "Founded in the third age by the Ashfall clans.",
			"## Trade", "Grain downriver, salt up."]:
		_ok("write: hand-authored content survived — %s" % kept, disk.find(kept) >= 0)
	links = _bridge.vault_links_for("settlement", tid)
	_ok("status: after writing, the link is Connected again",
		String((links[0] as Dictionary).get("status", "")) == "connected", str(links))

	# -- §5 the Cartalith block --------------------------------------------
	var fields: Array = _bridge.vault_export_fields("settlement", tid)
	_ok("block: fields are offered for a settlement", fields.size() > 0, str(fields))
	var keys := PackedStringArray()
	for f in fields:
		keys.append(String((f as Dictionary).get("key", "")))
	var body: String = _bridge.vault_block_body("settlement", tid, keys)
	_ok("block: the body names the settlement", body.find(sname) >= 0, body)

	var bp: Dictionary = _bridge.vault_preview_block("Locations/Nareth.md", "settlement", tid, body)
	_ok("block: first write previews as an insert",
		bool(bp.get("ok", false)) and String(bp.get("action", "")) == "inserted", str(bp))
	var bw: Dictionary = _bridge.vault_write_block("Locations/Nareth.md", "settlement", tid, body, String(bp.get("hash", "")))
	_ok("block: it writes", bool(bw.get("ok", false)), String(bw.get("error", "")))
	var after_block := _read(_note)
	_ok("block: the marker is in the file", after_block.find("<!-- CARTALITH:BEGIN") >= 0)
	_ok("block: it is plain Markdown, not an Obsidian construct", after_block.find("obsidian://") < 0)
	_ok("block: the author's prose survived the insert",
		after_block.find("A river town at the third ford. The author wrote this sentence by hand.") >= 0)
	_ok("block: the earlier section edit survived", after_block.find("Rebuilt after the fire of 812.") >= 0)

	var bp2: Dictionary = _bridge.vault_preview_block("Locations/Nareth.md", "settlement", tid, body)
	_ok("block: a second write previews as a replace", String(bp2.get("action", "")) == "replaced", str(bp2))
	_bridge.vault_write_block("Locations/Nareth.md", "settlement", tid, body, String(bp2.get("hash", "")))
	var after_block2 := _read(_note)
	_ok("block: updating does not add a second block",
		after_block2.count("<!-- CARTALITH:BEGIN") == 1, str(after_block2.count("<!-- CARTALITH:BEGIN")))
	_ok("block: updating left everything else alone", after_block2 == after_block)

	# -- §6 author-field population ----------------------------------------
	var att2: Dictionary = _bridge.vault_attach("settlement", tid, sname, "Locations/Template.md", "")
	_ok("attach: a whole-document link is created", bool(att2.get("ok", false)), String(att2.get("error", "")))
	var fp: Dictionary = _bridge.vault_preview_field_fill("Locations/Template.md", "settlement", tid, false)
	_ok("fields: the fill previews", bool(fp.get("ok", false)), String(fp.get("error", "")))
	var outcomes := {}
	for e in fp.get("report", []):
		outcomes[String((e as Dictionary).get("field", ""))] = String((e as Dictionary).get("outcome", ""))
	_ok("fields: an author-filled field is skipped, not clobbered",
		outcomes.get("Location", "") == "skipped_occupied", str(outcomes))
	_ok("fields: an empty field is written", outcomes.get("Size / Population", "") == "written", str(outcomes))
	_bridge.vault_write_field_fill("Locations/Template.md", "settlement", tid, false, String(fp.get("hash", "")))
	var tmpl_disk := _read(_tmpl)
	_ok("fields: the author's own value is byte-identical",
		tmpl_disk.find("**Location:** Riverbend") >= 0, tmpl_disk)
	_ok("fields: the empty one now carries a number",
		tmpl_disk.find("- **Size / Population:** ") >= 0 and tmpl_disk.find("- **Size / Population:**\n") < 0, tmpl_disk)

	# -- §7 the stale guard -------------------------------------------------
	_bridge.vault_set_link_text(link, "## The Old Quarter\n\nMine.\n")
	var prev3: Dictionary = _bridge.vault_preview_section_write(link)
	var theirs := _read(_note).replace("Grain downriver, salt up.", "Grain downriver, salt up, and wool.")
	_write(_note, theirs)
	links = _bridge.vault_links_for("settlement", tid)
	_ok("stale: an externally edited source is reported Stale",
		String((links[0] as Dictionary).get("status", "")) == "stale", str(links))
	var refused: Dictionary = _bridge.vault_write_section(link, String(prev3.get("hash", "")))
	_ok("stale: the write refuses", not bool(refused.get("ok", true)), String(refused.get("error", "")))
	_ok("stale: not one byte was written", _read(_note) == theirs)

	# -- §8 the panels ------------------------------------------------------
	_app.open_place_editor(0)
	await get_tree().process_frame
	var pe := _find(_app, PlaceEditorWindow)
	var pe_text := "\n".join(_texts(pe, []))
	_ok("panel: the place editor shows a KNOWLEDGE section", pe_text.findn("knowledge") >= 0)
	_ok("panel: it reports the linked notes", pe_text.find("linked note") >= 0, pe_text)

	pe.hide()
	await get_tree().process_frame
	_app.open_vault("settlement", tid, sname)
	await get_tree().process_frame
	var vw := _find(_app, VaultWindow)
	var vw_text := "\n".join(_texts(vw, []))
	_ok("panel: the vault window shows the connection", vw_text.findn("ProbeVault") >= 0, vw_text)
	_ok("panel: it lists the attached note", vw_text.findn("Locations/Nareth.md") >= 0, vw_text)

	## Open the link in the reader so §29's working copy and §20's checkbox
	## list are actually *built*, not just reachable -- `_build_feedback` is
	## the only place `DccWidgets.toggle` is called from this window and a
	## wrong argument there would never show up in any Rust test.
	vw._reader_link = link
	vw._rebuild()
	await get_tree().process_frame
	vw_text = "
".join(_texts(vw, []))
	_ok("panel: the reader and the feedback checkboxes build", vw_text.findn("Working copy") >= 0
		and vw_text.findn("Cartalith feedback") >= 0, vw_text)
	_ok("panel: a checkbox row carries the entity's real value",
		vw_text.findn("Population") >= 0 and vw_text.findn(sname) >= 0, vw_text)
	_ok("panel: the write actions are offered", vw_text.findn("Insert updated section into source") >= 0
		and vw_text.findn("Preview & write Cartalith block") >= 0, vw_text)

	_find(_app, VaultWindow).hide()
	await get_tree().process_frame
	_app.open_vault("continent", 1, "probe")
	await get_tree().process_frame
	vw_text = "\n".join(_texts(_find(_app, VaultWindow), []))
	_ok("panel: a continent opens the same window", vw_text.findn("Attach") >= 0 or vw_text.findn("Knowledge") >= 0, vw_text)

	# -- §9 persistence ------------------------------------------------------
	VaultStore.save_from(_bridge)
	_ok("persist: the sidecar is written", FileAccess.file_exists(VaultStore.PATH))
	var side = JSON.parse_string(_read(VaultStore.PATH))
	_ok("persist: it carries the links and the device binding",
		typeof(side) == TYPE_DICTIONARY and String((side as Dictionary).get("binding", "")) != ""
		and (((side as Dictionary).get("store", {}) as Dictionary).get("links", []) as Array).size() == 2, str(side))

	_finish()


func _finish() -> void:
	if _fails.is_empty():
		print("VAULT    ALL CHECKS PASSED")
	else:
		print("VAULT    %d FAILED: %s" % [_fails.size(), ", ".join(PackedStringArray(_fails))])
	get_tree().quit(0 if _fails.is_empty() else 1)
