extends Node
## INDEPENDENT verification probe (adversarial). Not either lane's.
##   godot --headless --path . _verifylanes_probe.tscn

var _fail := 0

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _info(s: String) -> void:
	print("  info ", s)

const STORE_A := """{
  "version": 1,
  "vaults": [{"id": "v1", "display_name": "Probe vault"}],
  "links": [{
    "link_id": "L1",
    "entity_kind": "settlement",
    "entity_id": 1,
    "entity_label": "Aldermoor",
    "vault_id": "v1",
    "relative_path": "places/aldermoor.md",
    "selection": {"type": "whole_document"},
    "imported_text": "# Aldermoor\\n"
  }],
  "snapshots": {"settlement:1|local": "maps/aldermoor-local.png"}
}"""

func _snapcount(wg: Object) -> int:
	var j: Variant = JSON.parse_string(String(wg.vault_state_json()))
	if typeof(j) != TYPE_DICTIONARY: return -1
	return (j.get("snapshots", {}) as Dictionary).size()

func _vaultcount(wg: Object) -> int:
	var j: Variant = JSON.parse_string(String(wg.vault_state_json()))
	if typeof(j) != TYPE_DICTIONARY: return -1
	return (j.get("vaults", []) as Array).size()

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var ud := OS.get_user_data_dir()

	# ---------------------------------------------------------------
	print("\n=== V1: load_save is a world replacement too -- does it clear? ===")
	var src: Object = ClassDB.instantiate("WorldGen")
	src.generate_sized(9001, 640.0, 96, 48)
	var zipA := ud + "/_vl_a.zip"
	_ok("wrote a save to load back", src.save_project(zipA), true)

	var w: Object = ClassDB.instantiate("WorldGen")
	w.generate_sized(9001, 640.0, 96, 48)
	_ok("store seeded", w.vault_restore_state(STORE_A), true)
	_ok("  link present before", (w.vault_all_links() as Array).size(), 1)
	_ok("  snapshot present before", _snapcount(w), 1)
	_ok("load_save succeeded", w.load_save(zipA), true)
	_ok("load_save cleared links", (w.vault_all_links() as Array).size(), 0)
	_ok("load_save cleared snapshots", _snapcount(w), 0)
	_ok("load_save kept the device vault binding", _vaultcount(w), 1)

	# a failing load must NOT clear (it promises to leave the world alone)
	var w2: Object = ClassDB.instantiate("WorldGen")
	w2.generate_sized(9001, 640.0, 96, 48)
	w2.vault_restore_state(STORE_A)
	_ok("a load of a nonexistent file fails", w2.load_save(ud + "/_vl_nope.zip"), false)
	_ok("  and left the links alone", (w2.vault_all_links() as Array).size(), 1)

	# project_open: clears then restores the ARCHIVE's own vault
	print("\n=== V2: project_open -- previous project's links must not leak in ===")
	var p1: Object = ClassDB.instantiate("WorldGen")
	p1.generate_sized(9001, 640.0, 96, 48)
	p1.vault_restore_state(STORE_A)
	var zipV := ud + "/_vl_vault.zip"
	var sv: Dictionary = p1.project_save_with_documents(zipV, {})
	_ok("project with a vault saved", sv.get("ok", false), true)

	var p2: Object = ClassDB.instantiate("WorldGen")
	p2.generate_sized(4242, 640.0, 96, 48)
	# a DIFFERENT project's links sitting in memory
	p2.vault_restore_state(STORE_A.replace("\"L1\"", "\"OTHER\"").replace("Aldermoor", "Elsewhere"))
	_ok("  other project's link in memory", (p2.vault_all_links() as Array).size(), 1)
	var op: Dictionary = p2.project_open(zipV)
	_ok("project_open ok", op.get("ok", false), true)
	var links_after: Array = p2.vault_all_links() as Array
	_ok("exactly one link after open (no merge)", links_after.size(), 1)
	_ok("and it is the ARCHIVE's, not the stale one",
		String((links_after[0] as Dictionary).get("link_id", "")), "L1")

	var p3: Object = ClassDB.instantiate("WorldGen")
	p3.generate_sized(4242, 640.0, 96, 48)
	p3.vault_restore_state(STORE_A.replace("\"L1\"", "\"OTHER\""))
	var zipNV := ud + "/_vl_novault.zip"
	var p4: Object = ClassDB.instantiate("WorldGen")
	p4.generate_sized(4242, 640.0, 96, 48)
	p4.project_save_with_documents(zipNV, {})
	p3.project_open(zipNV)
	_ok("opening a vault-less project drops the stale links",
		(p3.vault_all_links() as Array).size(), 0)

	# ---------------------------------------------------------------
	print("\n=== V3: A2 -- does the origin survive a save/reopen? ===")
	var g: Object = ClassDB.instantiate("WorldGen")
	g.generate_sized(4242, 512.0, 160, 80)
	var refpng := ud + "/_vl_ref.png"
	g.export_heightmap_png(refpng, 2048)
	var gkey := String(g.atlas_world_key())
	var i: Object = ClassDB.instantiate("WorldGen")
	_ok("import ran", i.import_heightmap(refpng, 4242, 512.0, 160), true)
	var ikey := String(i.atlas_world_key())
	_info("live: generated=" + gkey + "  imported=" + ikey)
	_ok("live import key differs from generated", ikey != gkey, true)

	var zipI := ud + "/_vl_import.zip"
	_ok("the imported world saved", i.save_project(zipI), true)
	var i2: Object = ClassDB.instantiate("WorldGen")
	_ok("and read back", i2.load_save(zipI), true)
	var ikey2 := String(i2.atlas_world_key())
	_info("after save/reopen the imported world's key = " + ikey2)
	_info("EXPECTED-BY-LANE: it becomes the GENERATED key -> collision relocated, atlas orphaned")
	_ok("[DISCLOSED COST] reopened import no longer keeps its own namespace",
		ikey2 == ikey, false)
	_ok("[DISCLOSED COST] reopened import lands in the GENERATED namespace",
		ikey2 == gkey, true)

	# project_open path, same question
	var i3: Object = ClassDB.instantiate("WorldGen")
	i3.import_heightmap(refpng, 4242, 512.0, 160)
	var zipI3 := ud + "/_vl_import_proj.zip"
	i3.project_save_with_documents(zipI3, {})
	var i4: Object = ClassDB.instantiate("WorldGen")
	i4.project_open(zipI3)
	_info("project_open of an imported world -> key = " + String(i4.atlas_world_key()))
	_ok("[DISCLOSED COST] same through project_open",
		String(i4.atlas_world_key()) == gkey, true)

	# ---------------------------------------------------------------
	print("\n=== V4: Lane B -- paint round trip through a REAL archive ===")
	var pa: Object = ClassDB.instantiate("WorldGen")
	pa.generate_sized(2024, 640.0, 96, 48)
	_ok("paint layer armed", pa.paint_set_layer("biome"), true)
	pa.paint_set_brush(3, 3.0, 1.0, 0.0, false, false)
	for k in range(10, 40, 3):
		pa.paint_stroke_at(float(k), 20.0)
	## `paint_document_json` reads the COMMITTED layer, not the pending draft.
	var pc: Dictionary = pa.paint_commit()
	_info("paint_commit -> " + str(pc))
	var counts: Dictionary = pa.paint_painted_counts()
	var biome_n := int(counts.get("total", 0))
	_info("painted biome cells = " + str(biome_n))
	_ok("something was actually painted", biome_n > 0, true)
	var pdoc := String(pa.paint_document_json())
	_ok("paint_document_json is non-empty", pdoc != "", true)

	var zipP := ud + "/_vl_paint.zip"
	var docs: Dictionary = pa.project_engine_built_documents()
	_ok("engine-built documents carry drafts/paint.json", docs.has("drafts/paint.json"), true)
	_ok("... and drafts/sculpt.json", docs.has("drafts/sculpt.json"), true)
	_ok("paint project saved", pa.project_save_with_documents(zipP, docs).get("ok", false), true)

	var pb: Object = ClassDB.instantiate("WorldGen")
	pb.generate_sized(777, 640.0, 96, 48)   # a DIFFERENT world, same grid
	var po: Dictionary = pb.project_open(zipP)
	_ok("reopened", po.get("ok", false), true)
	var rest: PackedStringArray = po.get("restored", PackedStringArray())
	_info("restored = " + str(rest))
	_ok("restored names paint layers", rest.has("paint layers"), true)
	_ok("restored names sculpt draft", rest.has("sculpt draft"), true)
	pb.paint_set_layer("biome")
	var counts2: Dictionary = pb.paint_painted_counts()
	_ok("painted biome cells came back", int(counts2.get("total", 0)), biome_n)
	_ok("the paint document is byte-identical after the round trip",
		String(pb.paint_document_json()), pdoc)

	# ---------------------------------------------------------------
	print("\n=== V5: Lane B -- sculpt stamps round trip ===")
	var sa: Object = ClassDB.instantiate("WorldGen")
	sa.generate_sized(2024, 640.0, 96, 48)
	sa.sculpt_set_seed(31337)
	_ok("a feature is armed", sa.sculpt_set_feature("ridge"), true)
	_ok("stroke began", sa.sculpt_begin_stroke(), true)
	sa.sculpt_add_point(10.0, 10.0)
	sa.sculpt_add_point(30.0, 18.0)
	sa.sculpt_add_point(50.0, 26.0)
	var made: int = sa.sculpt_end_stroke()
	_info("sculpt_end_stroke -> " + str(made) + ", stamps=" + str(sa.sculpt_stamp_count()))
	_ok("a stamp exists", sa.sculpt_stamp_count() > 0, true)
	var sdoc := String(sa.sculpt_document_json())
	var stamps_before: int = sa.sculpt_stamp_count()
	var zipS := ud + "/_vl_sculpt.zip"
	sa.project_save_with_documents(zipS, sa.project_engine_built_documents())
	var sb: Object = ClassDB.instantiate("WorldGen")
	sb.generate_sized(777, 640.0, 96, 48)
	var so: Dictionary = sb.project_open(zipS)
	_ok("sculpt project reopened", so.get("ok", false), true)
	_ok("stamps came back", sb.sculpt_stamp_count(), stamps_before)
	_ok("the sculpt document is identical after the round trip",
		String(sb.sculpt_document_json()), sdoc)
	_ok("the stroke seed came back too", sb.sculpt_get_seed(), 31337)
	# and the claim that a loaded project still refuses to COMMIT
	## `sculpt_commit` signals refusal by returning an EMPTY dictionary.
	var cr: Dictionary = sb.sculpt_commit("probe")
	_ok("a loaded project still refuses to commit the draft", cr.is_empty(), true)
	_ok("  and the stamps are still there afterwards", sb.sculpt_stamp_count(), stamps_before)
	## contrast: over a GENERATED world the same call really does commit
	var sc: Dictionary = sa.sculpt_commit("probe")
	_ok("  over a generated world it commits", sc.is_empty(), false)
	_info("generated-world commit summary keys = " + str(sc.keys()))

	# ---------------------------------------------------------------
	print("\n=== V6: Lane B -- travel library round trip ===")
	var ta: Object = ClassDB.instantiate("WorldGen")
	ta.generate_sized(2024, 640.0, 96, 48)
	var tdoc_in := """{"animals":[{"id":"probe_mule","species":"mule","fields":{"name":"Probe Mule"}}],"vehicles":[],"vessels":[],"presets":[]}"""
	var tr: Dictionary = ta.travel_library_restore_document(tdoc_in)
	_ok("a custom travel entry was added", tr.get("ok", false), true)
	_info("restored=" + str(tr.get("restored", 0)) + " rejected=" + str(tr.get("rejected", [])))
	var tdoc := String(ta.travel_library_document_json())
	_ok("travel doc is non-empty", tdoc != "", true)
	var zipT := ud + "/_vl_travel.zip"
	ta.project_save_with_documents(zipT, ta.project_engine_built_documents())
	var tb: Object = ClassDB.instantiate("WorldGen")
	tb.generate_sized(777, 640.0, 96, 48)
	var topen: Dictionary = tb.project_open(zipT)
	var tdocs: Dictionary = topen.get("documents", {})
	_ok("library/travel.json comes back in `documents`", tdocs.has("library/travel.json"), true)
	_ok("... byte-identical", String(tdocs.get("library/travel.json", "")), tdoc)
	# the shell's own restore leg
	var tr2: Dictionary = tb.travel_library_restore_document(String(tdocs.get("library/travel.json", "")))
	_ok("shell-side restore ok", tr2.get("ok", false), true)
	_ok("and the library now re-emits the same doc", String(tb.travel_library_document_json()), tdoc)

	# ---------------------------------------------------------------
	print("\n=== V7: Lane B -- foreign entries survive open + re-save (BRIDGE level) ===")
	var fa: Object = ClassDB.instantiate("WorldGen")
	fa.generate_sized(2024, 640.0, 96, 48)
	var zipF := ud + "/_vl_foreign.zip"
	fa.project_save_with_documents(zipF, {})
	# graft two unknown entries in by rewriting the zip with Godot's own writer
	var zr := ZIPReader.new()
	_ok("reader opened", zr.open(zipF), OK)
	var names: PackedStringArray = zr.get_files()
	var payload := PackedByteArray([0x00, 0xFF, 0x10, 0x42, 0x00, 0x99])
	var kept := {}
	for nm in names:
		kept[nm] = zr.read_file(nm)
	zr.close()
	var zipG := ud + "/_vl_foreign2.zip"
	var zw := ZIPPacker.new()
	_ok("packer opened", zw.open(zipG), OK)
	for nm in kept.keys():
		zw.start_file(nm); zw.write_file(kept[nm]); zw.close_file()
	zw.start_file("cartography/tiles/0/0/0.png"); zw.write_file(payload); zw.close_file()
	zw.start_file("entities/dragons.json"); zw.write_file("{\"wyrm\":1}".to_utf8_buffer()); zw.close_file()
	zw.close()

	var fb: Object = ClassDB.instantiate("WorldGen")
	fb.generate_sized(777, 640.0, 96, 48)
	var fo: Dictionary = fb.project_open(zipG)
	_ok("the grafted archive opened", fo.get("ok", false), true)
	var fe: PackedStringArray = fo.get("foreign_entries", PackedStringArray())
	_info("foreign_entries = " + str(fe))
	_ok("the binary tile was seen as foreign", fe.has("cartography/tiles/0/0/0.png"), true)
	_ok("the unknown json was seen as foreign", fe.has("entities/dragons.json"), true)
	var zipH := ud + "/_vl_foreign3.zip"
	_ok("re-saved", fb.project_save_with_documents(zipH, {}).get("ok", false), true)
	var zr2 := ZIPReader.new()
	zr2.open(zipH)
	var out_names: PackedStringArray = zr2.get_files()
	_ok("the binary tile survived the re-save", out_names.has("cartography/tiles/0/0/0.png"), true)
	_ok("the unknown json survived too", out_names.has("entities/dragons.json"), true)
	if out_names.has("cartography/tiles/0/0/0.png"):
		_ok("... and its bytes are unchanged",
			zr2.read_file("cartography/tiles/0/0/0.png"), payload)
	zr2.close()
	# and a SECOND save owes the same entries (carried_foreign is cloned, not moved)
	var zipI2 := ud + "/_vl_foreign4.zip"
	fb.project_save_with_documents(zipI2, {})
	var zr3 := ZIPReader.new(); zr3.open(zipI2)
	_ok("a second save still owes them", (zr3.get_files() as PackedStringArray).has("entities/dragons.json"), true)
	zr3.close()
	# a generate must NOT graft them onto the next world
	fb.generate_sized(555, 640.0, 96, 48)
	var zipJ := ud + "/_vl_foreign5.zip"
	fb.project_save_with_documents(zipJ, {})
	var zr4 := ZIPReader.new(); zr4.open(zipJ)
	_ok("a regenerate drops the previous project's foreign entries",
		(zr4.get_files() as PackedStringArray).has("entities/dragons.json"), false)
	zr4.close()

	for f in ["_vl_a.zip","_vl_vault.zip","_vl_novault.zip","_vl_ref.png","_vl_import.zip",
			"_vl_import_proj.zip","_vl_paint.zip","_vl_sculpt.zip","_vl_travel.zip",
			"_vl_foreign.zip","_vl_foreign2.zip","_vl_foreign3.zip","_vl_foreign4.zip","_vl_foreign5.zip"]:
		DirAccess.remove_absolute(ud + "/" + f)
	print("\n_verifylanes_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
