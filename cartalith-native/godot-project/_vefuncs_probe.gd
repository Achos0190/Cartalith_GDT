extends SceneTree
## The seventeen `vault_*` `#[func]`s the note editor added that no probe called (`OUTSTANDING_WORK.md`, "Vault editor
## and linking" row): `vault_create_file`, `vault_embed_open`, `vault_frontmatter_remove`, `vault_link_at`,
## `vault_link_mention`, `vault_mention_spans`, `vault_note_aliases`, `vault_note_chronos`, `vault_note_properties`,
## `vault_note_tags`, `vault_outgoing_links`, `vault_private_marks`, `vault_read_image`, `vault_resolve_link`,
## `vault_suggest`, `vault_suggest_notes` and `vault_toggle_task`. The checks live in `_vefuncs_checks.gd`, shared
## with the windowed form-factor probe `_vefuncs_form_probe.gd`; this is the headless run (logic only, no pixels),
## which starts in seconds, so it is the one the mutation passes use.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_vefuncs_probe.gd
##
## Exit 0 = every check held, 1 = a check failed, 2 = could not run. Writes a throw-away vault in TEMP, removed at the end.


func _init() -> void:
	var b := EngineBridge.new()
	get_root().add_child(b)
	var checks = load("res://_vefuncs_checks.gd").new()
	quit(checks.run(b))
