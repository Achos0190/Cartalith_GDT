extends Node
## Committed probe -- confirms the CARTO dock's "Ways · by type"
## group actually BUILDS its five rows in the real shell (the way-real probe
## only read the constant), and shoots the panel.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed"): these are kept as the evidence for
## the passes that wrote them, not deleted after them. Copy this line rather
## than the disposable-scratch-file boilerplate the earlier headers carried.

func _p(s: String) -> void:
	print("WAYFILTER  %s" % s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	if n is Control and "text" in n:
		out.append(String(n.text))
	for c in n.get_children():
		_walk(c, out)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(2))
	wd.start()

	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	app.open_project_dialog.hide()
	await _frames(2)

	var bridge = app.bridge
	bridge.generate({"seed": 483920, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(4)

	app.select_domain("cartography")
	await _frames(6)

	var texts: Array = []
	_walk(app, texts)
	var want := ["Trade highways", "Regional roads", "Roads", "Tracks", "Ancient routes"]
	for w in want:
		_p("row %-16s present: %s" % [w, str(texts.has(w))])
	_p("stale rows still present: %s" % str(texts.has("Ancient ways")))
	var hits: Array = []
	for t in texts:
		var ts := String(t)
		if ts.to_lower().contains("way") or ts.to_lower().contains("road") or ts.to_lower().contains("track") or ts.to_lower().contains("highway"):
			hits.append(ts)
	_p("way-ish texts in the tree (%d nodes total): %s" % [texts.size(), str(hits)])

	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png("user://wayfilter.png")
	_p("shot -> %s" % ProjectSettings.globalize_path("user://wayfilter.png"))
	get_tree().quit(0)
