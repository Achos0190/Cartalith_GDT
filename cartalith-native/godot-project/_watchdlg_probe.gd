extends Node
## Protects `app.gd::_watch_dialog` (2026-11-22): a window re-added to the tree (the phone re-opens its settings window)
## fires `node_added` again, and an unguarded `visibility_changed.connect` logs "Signal 'visibility_changed' is already
## connected" every time. This probe re-adds one AcceptDialog three times and prints how many `_fit_dialog` hooks it
## carries (must be exactly 1); and then requires that godot.log holds no "already connected" line (and is not empty, so a missing log cannot pass).
## Windowed: godot --path . --resolution 1280x720 _watchdlg_probe.tscn
func _ready() -> void:
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	var d := AcceptDialog.new()
	app.add_child(d)
	for i in 3:
		app.remove_child(d)
		await get_tree().process_frame
		app.add_child(d)
		await get_tree().process_frame
	var n := 0
	for c in d.get_signal_connection_list("visibility_changed"):
		if String((c["callable"] as Callable).get_method()).contains("_fit_dialog"):
			n += 1
	print("hooks=%d" % n)
	await get_tree().create_timer(0.5).timeout
	## The engine's own error output goes to user://logs/godot.log (file logging on desktop); the unguarded connect writes
	## "already connected" there, and the file is the only place a script can see it.
	var log := FileAccess.get_file_as_string("user://logs/godot.log")
	var dup := log.count("already connected")
	print("already-connected lines=%d (log %d bytes)" % [dup, log.length()])
	var bad := (0 if n == 1 else 1) + (0 if dup == 0 and log.length() > 0 else 1)
	print("RESULT: %s (%d failed)" % ["PASS" if bad == 0 else "FAIL", bad])
	get_tree().quit(0 if bad == 0 else 1)
