extends SceneTree
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
