extends Node
## Decisive, isolated proof of the mechanic every fix in this batch rests on:
## does `Button.flat = true` suppress an EXPLICIT `add_theme_stylebox_override`
## on "normal", or only the theme-default stylebox? `layers_popover.gd`'s own
## comment (~line 694) claims the override never draws at all while `flat` is
## true. Four buttons, windowed, sampled at their own centres:
##
##   A: flat=false, normal override = solid red            -- must be red (control)
##   B: flat=true,  normal override = solid red             -- the claim under test
##   C: flat=true then flat=false, same override            -- must be red (the fix pattern)
##   D: flat=true,  no override at all                      -- must be the empty bg (sanity)
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 800x400 \
##       _fillbug_probe.tscn

func _grab() -> Image:
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()

func _mk(x: int, flat_now: bool, flat_after: bool, override: bool) -> Button:
	var b := Button.new()
	b.position = Vector2(x, 100)
	b.size = Vector2(150, 80)
	b.flat = flat_now
	if override:
		var sb := StyleBoxFlat.new()
		sb.bg_color = Color(1, 0, 0, 1)
		b.add_theme_stylebox_override("normal", sb)
	b.flat = flat_after
	add_child(b)
	return b

func _mk_hover(x: int, flat_now: bool) -> Button:
	var b := Button.new()
	b.position = Vector2(x, 250)
	b.size = Vector2(150, 80)
	b.flat = flat_now
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0, 0, 1, 1)   ## blue -- distinct from the normal-state red
	b.add_theme_stylebox_override("hover", sb)
	add_child(b)
	return b

func _ready() -> void:
	var root := Control.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(root)
	var bg := ColorRect.new()
	bg.color = Color(0, 0.5, 0, 1)   ## distinct from red/blue and from theme greys
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_child(bg)

	var specs := [
		{"x": 20,  "flat_now": false, "flat_after": false, "override": true,  "label": "A flat=false + override"},
		{"x": 220, "flat_now": true,  "flat_after": true,  "override": true,  "label": "B flat=true + override"},
		{"x": 420, "flat_now": true,  "flat_after": false, "override": true,  "label": "C flat true->false + override"},
		{"x": 620, "flat_now": true,  "flat_after": true,  "override": false, "label": "D flat=true, no override"},
	]
	for s in specs:
		root.add_child(_mk(int(s["x"]), bool(s["flat_now"]), bool(s["flat_after"]), bool(s["override"])))

	## E: flat=true + hover override (the `category()`/`_sheet_close_button`
	## shape). F: flat=false + hover override, same position pattern, positive
	## control proving the warp-and-hover mechanism itself actually works.
	var e_btn := _mk_hover(20, true)
	var f_btn := _mk_hover(220, false)

	await get_tree().process_frame
	await get_tree().process_frame
	var img := await _grab()

	## Baseline: neither hovered yet, capture happens before the mouse moves.
	var e_idle: Color = img.get_pixel(20 + 75, 250 + 40)
	var f_idle: Color = img.get_pixel(220 + 75, 250 + 40)
	print("FILLBUG-HOVER idle  E(flat=true)=%s  F(flat=false)=%s" % [e_idle, f_idle])

	## Now inject a real `InputEventMouseMotion` onto each in turn (a hardware
	## `warp_mouse` alone moves the cursor but generates no motion event, so
	## `BaseButton`'s own hover tracking never sees it -- this pushes the event
	## through the same pipeline the OS would).
	var mm_e := InputEventMouseMotion.new()
	mm_e.position = Vector2(20 + 75, 250 + 40)
	mm_e.global_position = mm_e.position
	Input.parse_input_event(mm_e)
	await get_tree().process_frame
	await get_tree().process_frame
	var img_e := await _grab()
	var e_hover: Color = img_e.get_pixel(20 + 75, 250 + 40)

	var mm_f := InputEventMouseMotion.new()
	mm_f.position = Vector2(220 + 75, 250 + 40)
	mm_f.global_position = mm_f.position
	Input.parse_input_event(mm_f)
	await get_tree().process_frame
	await get_tree().process_frame
	var img_f := await _grab()
	var f_hover: Color = img_f.get_pixel(220 + 75, 250 + 40)

	var e_is_blue := e_hover.b > 0.6 and e_hover.r < 0.3
	var f_is_blue := f_hover.b > 0.6 and f_hover.r < 0.3
	print("FILLBUG-HOVER on-hover E(flat=true)=%s blue=%s   F(flat=false)=%s blue=%s" \
		% [e_hover, e_is_blue, f_hover, f_is_blue])
	if f_is_blue and not e_is_blue:
		print("FILLBUG-HOVER CONFIRMED: flat=true ALSO suppresses an explicit hover override.")
	elif f_is_blue and e_is_blue:
		print("FILLBUG-HOVER REFUTED: flat=true does NOT suppress hover overrides -- only normal.")
	else:
		print("FILLBUG-HOVER INCONCLUSIVE -- positive control F did not go blue; warp/hover mechanism untrusted.")

	var fail := 0
	for s in specs:
		var cx := int(s["x"]) + 75
		var cy := 100 + 40
		var col: Color = img.get_pixel(cx, cy)
		var is_red := col.r > 0.6 and col.g < 0.3 and col.b < 0.3
		print("FILLBUG %s @ (%d,%d) = %s  red=%s" % [String(s["label"]), cx, cy, col, is_red])

	## The decisive lines: does B disagree with A/C? Does D read as not-red
	## (proving red is never a fallback/background bleed-through)?
	var colA: Color = img.get_pixel(20 + 75, 140)
	var colB: Color = img.get_pixel(220 + 75, 140)
	var colC: Color = img.get_pixel(420 + 75, 140)
	var colD: Color = img.get_pixel(620 + 75, 140)
	var a_red := colA.r > 0.6 and colA.g < 0.3
	var b_red := colB.r > 0.6 and colB.g < 0.3
	var c_red := colC.r > 0.6 and colC.g < 0.3
	var d_red := colD.r > 0.6 and colD.g < 0.3
	print("FILLBUG verdict: A(control)=%s B(flat+override)=%s C(fix pattern)=%s D(flat,no override)=%s" \
		% [a_red, b_red, c_red, d_red])
	if a_red and c_red and not b_red and not d_red:
		print("FILLBUG CONFIRMED: flat=true suppresses an explicit stylebox override; flat=false restores it.")
	elif a_red and b_red and c_red:
		print("FILLBUG REFUTED: flat=true does NOT suppress an explicit override -- B drew red too.")
	else:
		print("FILLBUG INCONCLUSIVE -- see the four raw samples above.")
	get_tree().quit(0)
