extends SceneTree
## Result: a load check (`extends SceneTree`, no `.tscn`, no header, no arguments): run with `--script
## res://_parsecheck.gd`. It loads seven hard-coded scripts (`civilization_workspace.gd`, `cartography_workspace.gd`,
## `world_workspace.gd`, `render_workspace.gd`, `faction_roster_window.gd`, `engine_bridge.gd` and `map_overlay.gd`) and
## prints `LOADED <path> -> true|false` for each, then `quit()` with exit code 0 whatever the loads returned; the verdict
## is in the printed lines only.
func _init():
	for p in ["res://shell/workspaces/civilization_workspace.gd",
			"res://shell/workspaces/cartography_workspace.gd",
			"res://shell/workspaces/world_workspace.gd",
			"res://shell/workspaces/render_workspace.gd",
			"res://shell/faction_roster_window.gd",
			"res://shell/engine_bridge.gd",
			"res://map_overlay.gd"]:
		var s = load(p)
		print("LOADED %s -> %s" % [p, str(s != null)])
	quit()
