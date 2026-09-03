extends Node
## World provenance survives a save -- OUTSTANDING_WORK.md 2.2, the format half.
##
## The Rust suite structurally cannot reach this. `WorldGen` is a cdylib
## `GodotClass`, so the line that copies `cartalith_io::SaveParams::origin`
## into `world_origin` on load, and the two writers that put it there, are
## only exercisable through the engine. Mutating that loader line back to its
## old `ORIGIN_GENERATED` SURVIVED `cargo test -p cartalith-godot` (measured);
## it must not survive this.
##
## Two round trips, one per writer:
##   flat -- save_project / load_save
##   tree -- project_save_with_documents / project_open
##
## The generated world is the CONTROL. It proves every *other* element of the
## atlas key round-trips, so an imported world whose key moves across a save
## has moved for the one remaining reason.
##
##   godot --headless --path . _saveorigin_probe.tscn

func _ready() -> void:
	var dir := OS.get_user_data_dir()
	var png := dir + "/_saveorigin_height.png"
	var fails: Array = []

	# --- a generated world at a fixed tuple ------------------------------
	var gen: WorldGen = WorldGen.new()
	gen.generate_sized(4242, 640.0, 64, 64)
	var key_gen: String = gen.atlas_world_key()
	print("key_generated  = ", key_gen)
	if key_gen == "":
		fails.append("generate_sized produced no world key")

	# --- an imported world at the same grid ------------------------------
	var img := Image.create_empty(64, 64, false, Image.FORMAT_L8)
	for y in 64:
		for x in 64:
			img.set_pixel(x, y, Color8((x * 4) % 256, (x * 4) % 256, (x * 4) % 256))
	if img.save_png(png) != OK:
		fails.append("could not write the heightmap PNG")
	var imp: WorldGen = WorldGen.new()
	if not imp.import_heightmap(png, 4242, 640.0, 64):
		fails.append("import_heightmap returned false")
	var key_imp: String = imp.atlas_world_key()
	print("key_imported   = ", key_imp)
	if key_imp == key_gen:
		fails.append("an import and a generate share one atlas namespace (the live half regressed)")

	# --- round trip 1: the flat writer -----------------------------------
	var flat := dir + "/_saveorigin_flat.zip"
	if not gen.save_project(flat):
		fails.append("flat: save_project(generated) returned false")
	var gen_flat: WorldGen = WorldGen.new()
	if not gen_flat.load_save(flat):
		fails.append("flat: load_save(generated) returned false")
	var key_gen_flat: String = gen_flat.atlas_world_key()
	print("flat  generated reopened = ", key_gen_flat)
	if key_gen_flat != key_gen:
		fails.append("flat CONTROL: a generated world changed atlas namespace across a save (%s -> %s)" % [key_gen, key_gen_flat])

	if not imp.save_project(flat):
		fails.append("flat: save_project(imported) returned false")
	var imp_flat: WorldGen = WorldGen.new()
	if not imp_flat.load_save(flat):
		fails.append("flat: load_save(imported) returned false")
	var key_imp_flat: String = imp_flat.atlas_world_key()
	print("flat  imported  reopened = ", key_imp_flat)
	if key_imp_flat != key_imp:
		fails.append("flat: a saved import reopened into another atlas (%s -> %s)" % [key_imp, key_imp_flat])
	if key_imp_flat == key_gen:
		fails.append("flat: a saved import reopened as a GENERATED world -- the exact collision")

	# --- round trip 2: the tree writer -----------------------------------
	var tree := dir + "/_saveorigin_tree.zip"
	var w1: Dictionary = gen.project_save_with_documents(tree, {})
	if not bool(w1.get("ok", false)):
		fails.append("tree: project_save_with_documents(generated) failed: %s" % str(w1))
	var gen_tree: WorldGen = WorldGen.new()
	var r1: Dictionary = gen_tree.project_open(tree)
	if not bool(r1.get("ok", false)):
		fails.append("tree: project_open(generated) failed: %s" % str(r1))
	var key_gen_tree: String = gen_tree.atlas_world_key()
	print("tree  generated reopened = ", key_gen_tree)
	if key_gen_tree != key_gen:
		fails.append("tree CONTROL: a generated world changed atlas namespace across a save (%s -> %s)" % [key_gen, key_gen_tree])

	var w2: Dictionary = imp.project_save_with_documents(tree, {})
	if not bool(w2.get("ok", false)):
		fails.append("tree: project_save_with_documents(imported) failed: %s" % str(w2))
	var imp_tree: WorldGen = WorldGen.new()
	var r2: Dictionary = imp_tree.project_open(tree)
	if not bool(r2.get("ok", false)):
		fails.append("tree: project_open(imported) failed: %s" % str(r2))
	var key_imp_tree: String = imp_tree.atlas_world_key()
	print("tree  imported  reopened = ", key_imp_tree)
	if key_imp_tree != key_imp:
		fails.append("tree: a saved import reopened into another atlas (%s -> %s)" % [key_imp, key_imp_tree])
	if key_imp_tree == key_gen:
		fails.append("tree: a saved import reopened as a GENERATED world -- the exact collision")

	if fails.is_empty():
		print("PROBE PASS: provenance survives both writers; the control is unmoved")
	else:
		for f in fails:
			print("PROBE FAIL: ", f)
	get_tree().quit(0 if fails.is_empty() else 1)
