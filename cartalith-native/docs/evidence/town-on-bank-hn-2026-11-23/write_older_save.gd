extends SceneTree
## Writes a world with the binary this project copy loads (the last build before `civ.bank_sites`), as a real
## older save:  -- OUT.cartalith SEED GW GH
func _init() -> void:
	var a := OS.get_cmdline_user_args()
	var wg: WorldGen = WorldGen.new()
	wg.set_villages_enabled(true)
	print("OLD has bank_sites param: ", wg.get_params().has("civ.bank_sites"), "; has offer func: ", wg.has_method("bank_sites_offer"))
	wg.generate_sized(int(a[1]), 800.0, int(a[2]), int(a[3]))
	var r: Dictionary = wg.project_save_with_documents(a[0], {})
	print("OLD saved ", a[0], ": ", str(r).left(200), "; settlements ", wg.get_settlements().size())
	quit(0)
