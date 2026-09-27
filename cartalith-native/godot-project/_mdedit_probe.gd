extends SceneTree
## Ruling BE's Markdown editor, its markup transforms pinned with exact
## strings -- `shell/markdown_editor.gd`'s static half, no UI and no engine.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . --script _mdedit_probe.gd
##
## Every expected string below is a literal typed here, never a value read
## back out of the editor (`MISTAKES.md`: never assert a constant against
## itself). Exit 0 = every check held; 1 = at least one failed. The RESULT
## line carries the count, so no header can drift from it.

const ED := preload("res://shell/markdown_editor.gd")

var _n := 0
var _fails := 0


func _check(label: String, got: Variant, want: Variant) -> void:
	_n += 1
	if got == want:
		print("MDEDIT  ok  %s" % label)
	else:
		_fails += 1
		print("MDEDIT  !!  %s\n        got  %s\n        want %s" % [label, var_to_str(got), var_to_str(want)])


func _r(d: Dictionary) -> Array:
	return [d["text"], d["from"], d["to"]]


func _init() -> void:
	# -- wrap: a selection gains the markers, and stays selected inside them --
	_check("bold wraps a selection", _r(ED.wrap_inline("a word here", 2, 6, "**", "**")),
		["a **word** here", 4, 8])
	_check("italic wraps a selection", _r(ED.wrap_inline("a word here", 2, 6, "*", "*")),
		["a *word* here", 3, 7])
	_check("strikethrough wraps a selection", _r(ED.wrap_inline("a word here", 2, 6, "~~", "~~")),
		["a ~~word~~ here", 4, 8])
	_check("inline code wraps a selection", _r(ED.wrap_inline("run it now", 4, 6, "`", "`")),
		["run `it` now", 5, 7])

	# -- the underline tag (Markdown has none; the ruling's stated default) ---
	_check("underline writes <u>…</u>", _r(ED.wrap_inline("a word here", 2, 6, "<u>", "</u>")),
		["a <u>word</u> here", 5, 9])
	_check("underline toggles back off (markers outside the selection)",
		_r(ED.wrap_inline("a <u>word</u> here", 5, 9, "<u>", "</u>")), ["a word here", 2, 6])
	_check("underline toggles off when the tags are inside the selection",
		_r(ED.wrap_inline("a <u>word</u> here", 2, 13, "<u>", "</u>")), ["a word here", 2, 6])

	# -- unwrap -------------------------------------------------------------------
	_check("bold unwraps when the markers sit just outside the selection",
		_r(ED.wrap_inline("a **word** here", 4, 8, "**", "**")), ["a word here", 2, 6])
	_check("bold unwraps when the selection includes the markers",
		_r(ED.wrap_inline("a **word** here", 2, 10, "**", "**")), ["a word here", 2, 6])
	_check("italic on a bold word adds italic, it does not strip a bold star",
		_r(ED.wrap_inline("a **word** here", 4, 8, "*", "*")), ["a ***word*** here", 5, 9])
	_check("italic unwraps from bold-italic, leaving the bold",
		_r(ED.wrap_inline("a ***word*** here", 5, 9, "*", "*")), ["a **word** here", 4, 8])
	_check("bold unwraps from bold-italic, leaving the italic",
		_r(ED.wrap_inline("a ***word*** here", 5, 9, "**", "**")), ["a *word* here", 3, 7])

	# -- caret insert: an empty pair, caret between; again removes it -----------
	_check("bold at a caret inserts an empty pair with the caret inside",
		_r(ED.wrap_inline("ab", 1, 1, "**", "**")), ["a****b", 3, 3])
	_check("bold again with the caret still inside removes the empty pair",
		_r(ED.wrap_inline("a****b", 3, 3, "**", "**")), ["ab", 1, 1])
	_check("underline at a caret", _r(ED.wrap_inline("", 0, 0, "<u>", "</u>")), ["<u></u>", 3, 3])

	# -- heading rewrite: only the caret line's # prefix ---------------------------
	var doc := "Title\nbody line\nend"
	_check("H1 on a plain line", _r(ED.set_heading(doc, 2, 2, 1)), ["# Title\nbody line\nend", 4, 4])
	_check("H2 on the middle line, caret mid-line",
		_r(ED.set_heading(doc, 10, 10, 2)), ["Title\n## body line\nend", 13, 13])
	_check("H1 -> H3 rewrites the prefix instead of stacking it",
		_r(ED.set_heading("# Title\nx", 4, 4, 3)), ["### Title\nx", 6, 6])
	_check("Normal removes the prefix",
		_r(ED.set_heading("## Title\nx", 5, 5, 0)), ["Title\nx", 2, 2])
	_check("a hashtag is not a heading, and gains one in front of it",
		_r(ED.set_heading("#tag word", 0, 0, 2)), ["## #tag word", 3, 3])
	_check("heading_level_at reads the caret line", ED.heading_level_at("x\n### Three\ny", 4), 3)

	# -- lists and quotes: every selected line ------------------------------------
	var three := "alpha\nbeta\ngamma"
	_check("bullet list over three lines", _r(ED.prefix_lines(three, 1, 13, "bullet")),
		["- alpha\n- beta\n- gamma", 0, 22])
	_check("numbered list over three lines", _r(ED.prefix_lines(three, 0, 16, "number")),
		["1. alpha\n2. beta\n3. gamma", 0, 25])
	_check("bullet list again removes it", _r(ED.prefix_lines("- alpha\n- beta", 0, 14, "bullet")),
		["alpha\nbeta", 0, 10])
	_check("numbered over a bulleted list swaps the marker, not stacks it",
		_r(ED.prefix_lines("- alpha\n- beta", 0, 14, "number")), ["1. alpha\n2. beta", 0, 16])
	_check("a blank line inside the selection is left alone",
		_r(ED.prefix_lines("a\n\nb", 0, 4, "bullet")), ["- a\n\n- b", 0, 8])
	_check("quote over two lines", _r(ED.prefix_lines("one\ntwo\nthree", 0, 5, "quote")),
		["> one\n> two\nthree", 0, 11])
	_check("a selection ending at a line start does not take that line",
		_r(ED.prefix_lines("one\ntwo", 0, 4, "bullet")), ["- one\ntwo", 0, 5])
	_check("a bullet on an empty line starts the list, caret after the marker",
		_r(ED.prefix_lines("x\n\ny", 2, 2, "bullet")), ["x\n- \ny", 4, 4])

	# -- code block, note link ------------------------------------------------------
	_check("code block fences the selected lines", _r(ED.code_block("a\nb\nc", 2, 3)),
		["a\n```\nb\n```\nc", 6, 7])
	_check("code block again removes the fence", _r(ED.code_block("a\n```\nb\n```\nc", 6, 7)),
		["a\nb\nc", 2, 3])
	_check("note link at a caret", _r(ED.note_link("see ", 4, 4, "Settlements/Aldenmoor.md")),
		["see [[Aldenmoor]]", 17, 17])
	_check("note link over a selection keeps it as the shown text",
		_r(ED.note_link("see the town", 4, 12, "Settlements/Aldenmoor.md")),
		["see [[Aldenmoor|the town]]", 26, 26])

	# -- the Preview renderer --------------------------------------------------------
	var accent := "abcdef"
	_check("render: bold, italic, underline, strike", ED.inline_bbcode("**b** *i* <u>u</u> ~~s~~", accent),
		"[b]b[/b] [i]i[/i] [u]u[/u] [s]s[/s]")
	_check("render: inline code is not markup inside", ED.inline_bbcode("`**x**` y", accent),
		"[code]**x**[/code] y")
	_check("render: a wikilink and a bracket that must be escaped",
		ED.inline_bbcode("[[Aldenmoor|the town]] [x]", accent),
		"[color=#abcdef][url=Aldenmoor]the town[/url][/color] [lb]x]")
	_check("strip_frontmatter drops the YAML block", ED.strip_frontmatter("---\na: 1\n---\n# H\n"), "# H\n")

	print("MDEDIT  RESULT checks=%d failed=%d" % [_n, _fails])
	quit(0 if _fails == 0 else 1)
