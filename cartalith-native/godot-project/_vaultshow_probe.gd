extends Node
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: Showcase, not a test: screenshots the Markdown Vault window (browse, edit, entity-scoped, place editor) into $VAULTSHOW_OUT.
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): windowed: VAULTSHOW_OUT=<existing dir> --path . _vaultshow_probe.tscn
## Result: 'VAULTSHOW saved <path>' lines; asserts nothing
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##
## Throwaway showcase (not a test): screenshots of the vault window in its
## three states for the owner -- browse, edit, and scoped to a map entity
## with an attached note -- plus the place editor's KNOWLEDGE block.
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultshow_probe.tscn

const SEED := 552017
var _vp: SubViewport
var app: Node
var _bridge
var OUT := ""


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _write(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _shot(name: String) -> void:
	await _frames(6)
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(OUT + "/" + name)
	print("VAULTSHOW saved %s/%s" % [OUT, name])


func _find_button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text) == text:
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null


func _ready() -> void:
	OUT = OS.get_environment("VAULTSHOW_OUT")
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 1000)
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_bridge = app.bridge
	_bridge.generate({"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(4)

	var ss: Array = _bridge.settlements()
	var idx := 0
	var s: Dictionary = ss[idx]
	var tid := int(s.get("tid", -1))
	var nm := String(s.get("name", "?"))
	print("VAULTSHOW settlement idx=%d tid=%d name=%s keys=%s" % [idx, tid, nm, str(s.keys())])

	var root := OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultshow"
	DirAccess.make_dir_recursive_absolute(root + "/Settlements")
	DirAccess.make_dir_recursive_absolute(root + "/Events")
	_write(root + "/Settlements/%s.md" % nm,
		"---\ntype: settlement\ntid: %d\nfounded: 212\nruler: Hale Amaret\n---\n\n# Founding legend\n\nThe old song claims the ford chose its own name. The first wardens kept a tally of every raft that crossed.\n\n## Customs Gate\n\nA river crossing grown into a customs town. Salt barges pay by the hand-width of their draught.\n\n## Notable people\n\n- Hale Amaret, reeve since 391\n- Orsa of the Weir, toll-keeper\n" % tid)
	_write(root + "/Events/Salt-Road.md",
		"---\ntype: event\nyear: 398\n---\n\nThree names on the customs-gate list of [[%s]].\n" % nm)
	var conn: Dictionary = _bridge.vault_connect(root, "Chronicle")
	print("VAULTSHOW connect %s" % str(conn))
	var idxr = _bridge.call("vault_build_index") if _bridge.has_method("vault_build_index") else null
	print("VAULTSHOW index %s" % str(idxr))

	# 1. browse, note selected
	app.open_vault_browse()
	await _frames(4)
	var vw: VaultWindow = app.vault_window
	vw.set("_pick_file", "Settlements/%s.md" % nm)
	vw.call("_rebuild")
	await _shot("1_browse.png")

	# 2. edit mode: press the real "Open to edit"
	var b := _find_button(vw, "Open to edit")
	print("VAULTSHOW open-to-edit button %s" % str(b != null))
	if b != null:
		b.pressed.emit()
	await _shot("2_edit.png")
	vw.hide()

	# 3. attach the note to the settlement, then open the vault scoped to it
	var a: Dictionary = _bridge.vault_attach("settlement", tid, nm, "Settlements/%s.md" % nm, "")
	print("VAULTSHOW attach %s" % str(a))
	app.open_vault("settlement", tid, nm)
	await _shot("3_entity_scoped.png")
	vw.hide()

	# 4. the settlement's own place editor, KNOWLEDGE block
	app.open_place_editor(idx)
	await _shot("4_place_editor.png")
	var pe: Node = app.place_editor_window
	var tb := _find_button(pe, "Vault notes")
	print("VAULTSHOW vault-notes tab %s" % str(tb != null))
	if tb != null:
		tb.pressed.emit()
	await _shot("5_place_editor_vault_tab.png")
	pe.hide()
	app.open_vault_browse()
	await _frames(4)
	vw.set("_pick_file", "Settlements/%s.md" % nm)
	vw.set("_browse_path", "")
	vw.call("_rebuild")
	await _shot("6_browse_after_attach.png")
	get_tree().quit(0)
