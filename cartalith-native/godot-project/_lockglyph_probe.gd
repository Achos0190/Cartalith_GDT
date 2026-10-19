extends Node
## Windowed proof that `DccIcons.SYMBOLS` strings are the codepoints they
## name and that `SYMBOLS["locked"]` renders as one glyph (2026-10-19).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _lockglyph_probe.tscn
##
## Why: `"locked"` was written `"\U0001F512"`. GDScript 4's `\U` takes six
## hex digits, so it parsed as U+01F5 followed by "12" and drew `ǵ12`; two
## comments blamed the fonts. This probe asserts, for every entry, that the
## string is exactly one codepoint (or the ASCII "+"), that "locked" is
## U+1F512, and that the lock draws as ONE glyph: a label of it through
## `DccTheme.mono()` (the shell's own font, Plex Mono plus its system
## fallback) has ink and is narrower than the three-glyph `ǵ12` the bug drew
## (a positive control drawn beside it), and the font reports the codepoint.
## Writes `user://lockglyph.png` (and prints its absolute path) for a look:
## on Windows the lock is Segoe UI Emoji's colour glyph. Fails on the
## pre-fix `dcc_icons.gd` (checked 2026-10-19: length 3, U+1F5).
##
## Exit 0 pass, 1 an assertion failed, 2 cannot run (headless: no
## rasteriser, `MISTAKES.md`).

var _fail := 0

func _check(ok: bool, what: String) -> void:
	print(("PASS " if ok else "FAIL ") + what)
	if not ok:
		_fail += 1

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("CANNOT RUN: headless has no rasteriser; run windowed")
		get_tree().quit(2)
		return
	for key in DccIcons.SYMBOLS:
		var s: String = DccIcons.SYMBOLS[key]
		_check(s.length() == 1, "SYMBOLS[%s] is one codepoint (U+%X, length %d)" % [key, s.unicode_at(0), s.length()])
	var lock: String = DccIcons.SYMBOLS["locked"]
	_check(lock.unicode_at(0) == 0x1F512, "SYMBOLS[locked] is U+1F512 LOCK")

	var bg := ColorRect.new()
	bg.color = Color(1, 1, 1)
	bg.size = Vector2(480, 140)
	add_child(bg)
	var labels := {}
	for i in 2:
		var l := Label.new()
		## Index 1 is the positive control: `\u` takes four digits, so this is
		## U+01F5 + "12", the string the bug drew.
		l.text = lock if i == 0 else "ǵ12"
		l.add_theme_font_override("font", DccTheme.mono())
		l.add_theme_font_size_override("font_size", 48)
		l.add_theme_color_override("font_color", Color(0, 0, 0))
		l.position = Vector2(20 + i * 220, 30)
		add_child(l)
		labels[i] = l
	for _f in 6:
		await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var path := ProjectSettings.globalize_path("user://lockglyph.png")
	img.save_png(path)
	print("capture: ", path)
	var ink := func(r: Rect2) -> int:
		var n := 0
		for y in range(int(r.position.y), int(r.end.y)):
			for x in range(int(r.position.x), int(r.end.x)):
				if img.get_pixel(x, y).v < 0.5:
					n += 1
		return n
	var lw: float = DccTheme.mono().get_string_size(lock, HORIZONTAL_ALIGNMENT_LEFT, -1, 48).x
	var bw: float = DccTheme.mono().get_string_size("ǵ12", HORIZONTAL_ALIGNMENT_LEFT, -1, 48).x
	var lock_ink: int = ink.call(Rect2(labels[0].position, labels[0].size))
	var bug_ink: int = ink.call(Rect2(labels[1].position, labels[1].size))
	print("lock width %.1f px, ink %d px; bug string width %.1f px, ink %d px" % [lw, lock_ink, bw, bug_ink])
	_check(bug_ink > 0, "positive control: the bug's ǵ12 draws ink")
	_check(lock_ink > 0, "the lock draws ink (a glyph, not empty)")
	## One glyph, not three: narrower than the bug's three. Not a tighter
	## ratio: on Windows the fallback that answers is Segoe UI Emoji, whose
	## lock is a ~1.4 em colour glyph (66 px at 48 against 86 px for ǵ12,
	## measured 2026-10-19).
	_check(lw < bw, "the lock is one glyph wide, not three")
	## A missing glyph would draw a one-glyph-wide tofu box with ink, so ask
	## the font (base and fallbacks) too, and look at the capture.
	_check(DccTheme.mono().has_char(0x1F512), "the shell's mono font (with its fallback) has U+1F512")
	print("lockglyph: %d failure(s)" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)
