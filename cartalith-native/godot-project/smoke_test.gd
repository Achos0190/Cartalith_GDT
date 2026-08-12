extends SceneTree

func _initialize() -> void:
	var wg: WorldGen = WorldGen.new()
	wg.generate(12345, 800.0, 64)
	print("width=", wg.get_width(), " height=", wg.get_height())
	var tex: ImageTexture = wg.build_color_texture()
	if tex == null:
		print("FAIL: build_color_texture returned null")
		quit(1)
		return
	var img: Image = tex.get_image()
	print("image size=", img.get_size())
	var sample_colors: Array = []
	for i in range(5):
		var x := i * 10
		var y := i * 10
		sample_colors.append(img.get_pixel(x, y))
	print("sample pixels=", sample_colors)
	img.save_png("res://smoke_test_output.png")
	print("PASS")
	quit(0)
