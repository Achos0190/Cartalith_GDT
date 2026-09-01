extends Node
## Throwaway diagnostic -- NOT part of the owned probe set, deleted after use.
## Measures whether `LayersPopover`'s fixed-228px non-phone width actually
## clips its Opacity slider row now that `DccWidgets.slider()` grows the
## track to `role_px("slider_track_w")` (90) on tablet.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	var vp := SubViewport.new()
	vp.size = Vector2i(2560, 1600)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(45)
	print("is_tablet=", DccTheme.is_tablet())
	var pop = app.layers_popover
	if pop == null:
		print("[FAIL] no layers_popover on app")
		get_tree().quit(1)
		return
	pop.open()
	await _frames(5)
	print("popover size=", pop.size, " visible=", pop.visible)
	# Walk to find the Opacity slider row's own minimum size.
	var stack: Array = [pop]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		for c in n.get_children(true):
			stack.append(c)
		if n is HSlider:
			var s := n as HSlider
			print("HSlider found: min_size=", s.get_combined_minimum_size(),
				" custom_minimum_size=", s.custom_minimum_size,
				" global_rect=", s.get_global_rect())
	print("popover combined_minimum_size=", pop.get_contents_minimum_size() if pop.has_method("get_contents_minimum_size") else "n/a")
	get_tree().quit(0)
