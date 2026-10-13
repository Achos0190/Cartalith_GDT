extends Node
## Ruling BK header (2026-10-13, probe adopted into git; first run passed on the debug DLL):
## Protects: project_save_with_documents writes a supplied preview_png into the archive's preview.png entry unchanged, and writes no entry when omitted.
## Run (Godot_v4.7.1-stable_win64_console.exe, from godot-project/): --headless --path . _previewpng_probe.tscn
## Result: 'PROBE PASS...' (exit 0) / 'PROBE FAIL: N assertion(s)' (exit 1)
## Sources: derived from this file's own assertions and a 2026-10-13 run; the feature symbols were grepped, not assumed.
##
## Verification for the `preview.png` archive slot -- OUTSTANDING_WORK.md's
## "nothing ever populates it" gap (`project_bridge.rs`'s `WorldGen::
## project_save_with_documents` now takes an optional `preview_png:
## PackedByteArray` and writes it through to `write.preview_png`).
##
## Headless, Rust-plumbing half only: this checks that bytes handed to
## `project_save_with_documents` land in the archive's `preview.png` entry
## unchanged, and that omitting them (the every-existing-call-site case)
## writes no entry at all. It does NOT exercise `DccApp._capture_preview_png()`
## -- that needs a real framebuffer and is `_previewcapture_probe.gd`,
## windowed only (`MISTAKES.md`'s "Run a pixel probe" row:
## `get_viewport().get_texture()` is a no-op headless).
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _previewpng_probe.tscn

func _ready() -> void:
	var dir := OS.get_user_data_dir()
	var fails: Array = []

	# A real, non-uniform PNG -- not a fake magic-byte stand-in -- so the
	# round trip proves an actual image decodes back out, not just that
	# bytes survived.
	var img := Image.create_empty(17, 11, false, Image.FORMAT_RGBA8)
	for y in 11:
		for x in 17:
			img.set_pixel(x, y, Color8((x * 13) % 256, (y * 23) % 256, ((x + y) * 7) % 256, 255))
	var png_bytes: PackedByteArray = img.save_png_to_buffer()
	if png_bytes.size() < 8 or png_bytes.slice(0, 8) != PackedByteArray([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]):
		fails.append("the fixture image itself did not encode real PNG magic bytes")

	var gen: WorldGen = WorldGen.new()
	gen.generate_sized(90210, 320.0, 32, 32)

	# --- case 1: preview supplied -----------------------------------------
	var with_preview := dir + "/_previewpng_with.ctl"
	var w1: Dictionary = gen.project_save_with_documents(with_preview, {}, png_bytes)
	if not bool(w1.get("ok", false)):
		fails.append("save with preview_png failed: %s" % str(w1))
	else:
		var zr := ZIPReader.new()
		if zr.open(with_preview) != OK:
			fails.append("could not reopen %s as a zip" % with_preview)
		else:
			var files: Array = zr.get_files()
			if not files.has("preview.png"):
				fails.append("preview.png entry is missing when preview_png bytes were supplied: %s" % str(files))
			else:
				var back: PackedByteArray = zr.read_file("preview.png")
				if back != png_bytes:
					fails.append("preview.png bytes came back changed (%d in, %d out)" % [png_bytes.size(), back.size()])
				var back_img := Image.new()
				if back_img.load_png_from_buffer(back) != OK:
					fails.append("preview.png did not decode as PNG")
				elif back_img.get_size() != Vector2i(17, 11):
					fails.append("decoded preview.png has the wrong size: %s" % str(back_img.get_size()))
				else:
					# Non-uniform: the fixture is a gradient, so more than
					# one distinct pixel colour must survive the round trip.
					var colors := {}
					for y in 11:
						for x in 17:
							colors[back_img.get_pixel(x, y)] = true
					if colors.size() < 2:
						fails.append("decoded preview.png is uniform (%d distinct colour(s)) -- looks like an unrendered/blank capture" % colors.size())
					else:
						print("preview.png round-tripped: %d bytes, %d distinct colours" % [back.size(), colors.size()])
			zr.close()

	# --- case 2: preview omitted -- must write NO entry, not an empty one -
	var no_preview := dir + "/_previewpng_without.ctl"
	var w2: Dictionary = gen.project_save_with_documents(no_preview, {})
	if not bool(w2.get("ok", false)):
		fails.append("save without preview_png failed: %s" % str(w2))
	else:
		var zr2 := ZIPReader.new()
		if zr2.open(no_preview) != OK:
			fails.append("could not reopen %s as a zip" % no_preview)
		else:
			var files2: Array = zr2.get_files()
			if files2.has("preview.png"):
				fails.append("preview.png entry present with no preview_png supplied -- default must stay absent: %s" % str(files2))
			else:
				print("no preview_png -> no preview.png entry, as before this change")
			zr2.close()

	# --- case 3: the plain 2-arg call every other call site still uses ----
	var plain := dir + "/_previewpng_plain.ctl"
	var w3: Dictionary = gen.project_save(plain)
	if not bool(w3.get("ok", false)):
		fails.append("plain project_save (no preview_png parameter at all) failed: %s" % str(w3))
	else:
		var zr3 := ZIPReader.new()
		if zr3.open(plain) != OK:
			fails.append("could not reopen %s as a zip" % plain)
		else:
			if Array(zr3.get_files()).has("preview.png"):
				fails.append("plain project_save wrote a preview.png -- it takes none")
			zr3.close()

	if fails.is_empty():
		print("PROBE PASS: preview_png plumbing round-trips, and its absence stays absent")
	else:
		for f in fails:
			print("PROBE FAIL: ", f)
	get_tree().quit(0 if fails.is_empty() else 1)
