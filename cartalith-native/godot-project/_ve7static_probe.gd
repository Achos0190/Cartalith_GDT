extends SceneTree
## VE-7 of `VAULT_EDITOR_PROPOSAL.md` s.7, the static half: what Preview draws for tables, callout types, `_italic_`,
## task boxes, embeds (notes, pictures, notices, a cycle that never ends), and chronos blocks -- the renderer's pure
## functions and its host hooks, with NO UI and no engine. `_ve7_probe.gd` is the windowed half (real clicks, real files).
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script _ve7static_probe.gd
##
## Every expected value is a literal typed here, derived by hand from the grammar (never read back out of the code
## under test). The two colours that appear (`e0a840`, `6fae7d`) are the dark palette's `warn` and `good` tokens, which
## already clear the 4.5:1 floor, so the floor's blend leaves them alone. Exit 0 = every check held, 1 = one failed.
## Prints `VE7S  ok  <label>`, `VE7S  !!  <label>` with `got` / `want`, and `VE7S  RESULT checks=<n> failed=<n>`.

const ED := preload("res://shell/markdown_editor.gd")

var _n := 0
var _fails := 0


func _check(label: String, got: Variant, want: Variant) -> void:
	_n += 1
	if typeof(got) == typeof(want) and got == want:
		print("VE7S  ok  %s" % label)
	else:
		_fails += 1
		print("VE7S  !!  %s\n        got  %s\n        want %s" % [label, var_to_str(got), var_to_str(want)])


func _init() -> void:
	# -- table rows and delimiter rows --------------------------------------------------------------
	_check("row: outer pipes dropped", ED.split_table_row("| a | b |"), PackedStringArray(["a", "b"]))
	_check("row: no outer pipes", ED.split_table_row("a | b"), PackedStringArray(["a", "b"]))
	_check("row: an escaped pipe stays in its cell", ED.split_table_row("| a \\| b | c |"), PackedStringArray(["a | b", "c"]))
	_check("row: an empty cell is kept", ED.split_table_row("| a |  | c |"), PackedStringArray(["a", "", "c"]))
	_check("row: a lone pipe is no cells", ED.split_table_row("|"), PackedStringArray())
	_check("delimiter: three alignments", ED.table_aligns("|---|:-:|--:|"), ["", "c", "r"])
	_check("delimiter: left and none", ED.table_aligns("| :-- | --- |"), ["l", ""])
	_check("delimiter: one column needs its pipes", ED.table_aligns("|---|"), [""])
	_check("delimiter: a lone rule is not a delimiter row", ED.table_aligns("---"), [])
	_check("delimiter: text in a cell is not one", ED.table_aligns("|--x|---|"), [])
	_check("delimiter: a row of words is not one", ED.table_aligns("a|b"), [])

	# -- a rendered table ----------------------------------------------------------------------------
	var cell := "[cell border=#123456 padding=6,3,6,3]"
	var hcell := "[cell border=#123456 bg=#abcdef26 padding=6,3,6,3]"
	_check("a table: header washed and bold, columns aligned, a short row padded, the table ends at the blank line",
		ED.to_bbcode("| H1 | H2 |\n|:--|--:|\n| a | b |\n| c |\n\nafter", "abcdef", "123456"),
		"[table=2]" + hcell + "[b]H1[/b][/cell]" + hcell + "[right][b]H2[/b][/right][/cell]"
		+ cell + "a[/cell]" + cell + "[right]b[/right][/cell]"
		+ cell + "c[/cell]" + cell + "[right][/right][/cell][/table]\n\nafter")
	_check("a table: a row longer than the header is cut",
		ED.to_bbcode("a | b\n--|--\n1 | 2 | 3", "abcdef", "123456"),
		"[table=2]" + hcell + "[b]a[/b][/cell]" + hcell + "[b]b[/b][/cell]" + cell + "1[/cell]" + cell + "2[/cell][/table]")
	_check("not a table: a line with a pipe over a plain rule",
		ED.to_bbcode("a | b\n---", "abcdef", "123456"), "a | b\n[color=#123456]" + "─".repeat(24) + "[/color]")
	_check("not a table: the delimiter width must match the header",
		ED.to_bbcode("a | b | c\n|--|--|\nrow", "abcdef", "123456"), "a | b | c\n|--|--|\nrow")
	_check("a table inside a fence is code, not a table",
		ED.to_bbcode("```\na | b\n--|--\n```", "abcdef", "123456"), "[code]a | b\n--|--[/code]")

	# -- callout types ------------------------------------------------------------------------------
	var fam := {
		"note": "water", "info": "water", "todo": "water", "abstract": "water", "summary": "water", "tldr": "water",
		"tip": "good", "hint": "good", "important": "good", "success": "good", "check": "good", "done": "good",
		"question": "warn", "help": "warn", "faq": "warn", "warning": "warn", "caution": "warn", "attention": "warn",
		"failure": "block", "fail": "block", "missing": "block", "danger": "block", "error": "block", "bug": "block",
		"example": "accent", "quote": "text_secondary", "cite": "text_secondary",
		"WARNING": "warn", " Tip ": "good", "madeup": "water", "": "water"}
	for k in fam:
		_check("callout type '%s' -> %s" % [k, fam[k]], ED.callout_style(k)["token"], fam[k])
	_check("callout glyphs differ by family", [ED.callout_style("note")["glyph"], ED.callout_style("tip")["glyph"],
		ED.callout_style("warning")["glyph"], ED.callout_style("bug")["glyph"]], ["✎", "✦", "⚠", "✖"])
	_check("a danger callout is the block colour and a success one the good colour",
		[ED.to_bbcode("> [!danger] x", "abcdef", "123456").begins_with("[indent][color=#c96a5a]▎ ✖ "),
		ED.to_bbcode("> [!success] x", "abcdef", "123456").begins_with("[indent][color=#6fae7d]▎ ✔ ")], [true, true])

	# -- _italic_ ------------------------------------------------------------------------------------
	_check("_italic_ is italic, snake_case_words is not, __bold__ is still bold",
		ED.inline_bbcode("_a_ snake_case_x __b__ _ c _", "abcdef"), "[i]a[/i] snake_case_x [b]b[/b] _ c _")

	_check("_italic_ needs a word edge on both sides: a_b_ c and _a_b are left alone",
		[ED.inline_bbcode("a_b_ c", "abcdef"), ED.inline_bbcode("_a_b", "abcdef")], ["a_b_ c", "_a_b"])
	_check("a quote after a callout has ended is a plain quote again",
		ED.to_bbcode("> [!tip] T\n> in\nplain\n> said", "abcdef", "123456"),
		"[indent][color=#6fae7d]▎ ✦ [b]T[/b][/color][/indent]\n[indent][color=#6fae7d]▎[/color] in[/indent]\nplain\n[indent][color=#123456][i]said[/i][/color][/indent]")

	# -- task boxes ----------------------------------------------------------------------------------
	_check("tasks without a toggler are plain glyphs (the pre-VE-7 text)",
		ED.to_bbcode("- [ ] one\n- [x] two", "abcdef", "123456"), "  ☐  one\n  ☑  [color=#123456][s]two[/s][/color]")
	_check("tasks with a toggler carry the line number in the WHOLE text, front matter counted",
		ED.to_bbcode("---\na: 1\n---\n- [ ] one\ntext\n- [x] two", "abcdef", "123456", Callable(), {"ticks": true}),
		"[color=#123456][code]---\na: 1\n---[/code][/color]\n[url=task:3]  ☐  [/url]one\ntext\n[url=task:5]  ☑  [/url][color=#123456][s]two[/s][/color]")
	_check("a task in a fence is not a task",
		ED.to_bbcode("```\n- [ ] one\n```", "abcdef", "123456", Callable(), {"ticks": true}), "[code]- [lb] ] one[/code]")

	# -- embeds through a fake host ------------------------------------------------------------------
	var seen_stacks: Array = []
	var world := {
		"B": {"ok": true, "kind": "note", "path": "B.md", "text": "- [ ] t\nbody ![[C]]", "truncated": false},
		"C": {"ok": true, "kind": "note", "path": "Dir/C.md", "text": "c text", "truncated": false},
		"Sec": {"ok": true, "kind": "note", "path": "B.md", "text": "## Sec\nline", "truncated": false, "heading": "Sec"},
		"Long": {"ok": true, "kind": "note", "path": "Long.md", "text": "1\n2\n3\n4\n5\n6\n7\n8", "truncated": false},
		"Gone": {"ok": true, "kind": "missing", "target": "Gone"},
		"A": {"ok": true, "kind": "cycle", "path": "A.md", "chain": ["A.md", "B.md", "A.md"]},
		"Deep": {"ok": true, "kind": "toodeep", "path": "D.md", "limit": 3},
		"../x.png": {"ok": true, "kind": "escape", "target": "../x.png"},
		"pic.png": {"ok": true, "kind": "image", "path": "pic.png", "stamp": "1:2"},
		"pic.png|200": {"ok": true, "kind": "image", "path": "pic.png", "stamp": "1:2", "width": 200},
		"pic.png|900": {"ok": true, "kind": "image", "path": "pic.png", "stamp": "1:2", "width": 900},
		"tiny.png|300": {"ok": true, "kind": "image", "path": "tiny.png", "stamp": "1:2", "width": 300},
		"tiny.png": {"ok": true, "kind": "image", "path": "tiny.png", "stamp": "1:2"},
		"pic x.png": {"ok": true, "kind": "image", "path": "pic x.png", "stamp": "1:2"},
		"a.gif": {"ok": true, "kind": "unsupported", "target": "a.gif", "why": "a .gif image is not drawn here"},
		"Amb": {"ok": true, "kind": "ambiguous", "target": "Amb", "candidates": ["x/Amb.md", "y/Amb.md"]},
		"Frag#H": {"ok": true, "kind": "fragment", "path": "B.md", "fragment": "#H"},
		"Big": {"ok": true, "kind": "toolarge", "path": "Big.png", "bytes": 20971520, "limit": 16777216},
		"Bad": {"ok": false, "error": "no vault"}}
	var hooks := {
		"stack": ["A.md"],
		"embed": func(inner: String, stack: Array) -> Dictionary:
			seen_stacks.append(stack.duplicate())
			return (world.get(inner, {"ok": true, "kind": "missing", "target": inner}) as Dictionary).duplicate(),
		"image": func(r: Dictionary) -> Dictionary:
			return {} if String(r["path"]) == "Big.png" else {"src": "res://reg/p.png", "w": 50 if String(r["path"]) == "tiny.png" else 800, "h": 600},
		"ticks": true}
	var W := "[color=#e0a840][i]⚠ "
	_check("block embed of a note: header, toggle mark, read-only body; the embed inside it is resolved from B",
		ED.to_bbcode("![[B]]", "abcdef", "123456", Callable(), hooks),
		"[color=#123456]▪ [url=embed:0]B[/url]   [url=wiki:B]open ›[/url][/color]\n[indent]  ☐  t\nbody [color=#abcdef][u][url=wiki:Dir/C]▪ C[/url][/u][/color][/indent]")
	_check("the chain the host was asked with grows by the embedded note", seen_stacks, [["A.md"], ["A.md", "B.md"]])
	_check("a heading embed names its heading in the title",
		ED.to_bbcode("![[Sec]]", "abcdef", "123456", Callable(), hooks),
		"[color=#123456]▪ [url=embed:0]B › Sec[/url]   [url=wiki:B]open ›[/url][/color]\n[indent][font_size=16][b]Sec[/b][/font_size]\nline[/indent]")
	_check("a long embed is collapsed to six lines with a toggle, and expands when its key is open",
		[ED.to_bbcode("![[Long]]", "abcdef", "123456", Callable(), hooks),
		ED.to_bbcode("![[Long]]", "abcdef", "123456", Callable(), {"stack": ["A.md"], "embed": hooks["embed"], "open": {"A.md>Long": true}})],
		["[color=#123456]▸ [url=embed:0]Long[/url]   [url=wiki:Long]open ›[/url][/color]\n[indent]1\n2\n3\n4\n5\n6\n[color=#123456]… more in this note[/color][/indent]",
		"[color=#123456]▾ [url=embed:0]Long[/url]   [url=wiki:Long]open ›[/url][/color]\n[indent]1\n2\n3\n4\n5\n6\n7\n8[/indent]"])
	_check("the embed keys are numbered in render order", ED.embed_keys, ["A.md>Long"])
	_check("a missing target", ED.to_bbcode("![[Gone]]", "abcdef", "123456", Callable(), hooks), W + "Embed not found: Gone[/i][/color]")
	_check("a cycle names the chain", ED.to_bbcode("![[A]]", "abcdef", "123456", Callable(), hooks), W + "Embed skipped, it would repeat itself: A → B → A[/i][/color]")
	_check("too deep says how deep", ED.to_bbcode("![[Deep]]", "abcdef", "123456", Callable(), hooks), W + "Embed skipped, more than 3 levels deep: Deep[/i][/color]")
	_check("a path escape is refused in words", ED.to_bbcode("![[../x.png]]", "abcdef", "123456", Callable(), hooks), W + "Embed refused, outside this vault: ../x.png[/i][/color]")
	_check("an unsupported file says why", ED.to_bbcode("![[a.gif]]", "abcdef", "123456", Callable(), hooks), W + "Embed not drawn: a .gif image is not drawn here[/i][/color]")
	_check("an ambiguous name counts its candidates", ED.to_bbcode("![[Amb]]", "abcdef", "123456", Callable(), hooks), W + "Embed Amb could mean 2 notes[/i][/color]")
	_check("a missing heading", ED.to_bbcode("![[Frag#H]]", "abcdef", "123456", Callable(), hooks), W + "B has nothing at #H[/i][/color]")
	_check("a host that could not answer says so, not 'missing'", ED.to_bbcode("![[Bad]]", "abcdef", "123456", Callable(), hooks), W + "Embed could not be read: no vault[/i][/color]")
	_check("a huge image says its size", ED.to_bbcode("![[Big]]", "abcdef", "123456", Callable(), hooks), W + "Image too large to show (20 MB, limit 16 MB): Big[/i][/color]")
	_check("a picture: the host's width, capped at 560",
		ED.to_bbcode("![[pic.png]]", "abcdef", "123456", Callable(), hooks), "[img width=560]res://reg/p.png[/img]")
	_check("a picture with |200 is 200 wide", ED.to_bbcode("![[pic.png|200]]", "abcdef", "123456", Callable(), hooks), "[img width=200]res://reg/p.png[/img]")
	_check("a |N width is honoured both ways (a 50 px picture asked for 300 is 300), capped at 560, and unasked it is its own",
		[ED.to_bbcode("![[tiny.png|300]]", "abcdef", "123456", Callable(), hooks), ED.to_bbcode("![[pic.png|900]]", "abcdef", "123456", Callable(), hooks), ED.to_bbcode("![[tiny.png]]", "abcdef", "123456", Callable(), hooks)],
		["[img width=300]res://reg/p.png[/img]", "[img width=560]res://reg/p.png[/img]", "[img width=50]res://reg/p.png[/img]"])
	_check("a picture in a sentence stays in it, and a Markdown image is decoded",
		ED.to_bbcode("see ![[pic.png|200]] and ![alt](pic%20x.png) ok", "abcdef", "123456", Callable(), hooks),
		"see [img width=200]res://reg/p.png[/img] and [img width=560]res://reg/p.png[/img] ok")
	_check("a note embed in a sentence is a link, not a block",
		ED.to_bbcode("x ![[C]] y", "abcdef", "123456", Callable(), hooks), "x [color=#abcdef][u][url=wiki:Dir/C]▪ C[/url][/u][/color] y")
	_check("a picture the host cannot decode is said so",
		ED.to_bbcode("![[Big]]", "abcdef", "123456", Callable(), {"stack": ["A.md"], "embed": func(_i, _s): return {"ok": true, "kind": "image", "path": "Big.png", "stamp": "1:1"}, "image": hooks["image"]}),
		W + "Image could not be read: Big[/i][/color]")
	_check("without the embed hook an embed is the labelled placeholder (the pre-VE-7 text)",
		ED.to_bbcode("![[map.png]]", "abcdef", "123456"), "[color=#abcdef][i]embedded: map.png[/i][/color]")
	_check("a user's [ in an embed name cannot open a tag", ED.to_bbcode("![[a[b]]", "abcdef", "123456", Callable(), hooks), W + "Embed not found: a[lb]b[/i][/color]")

	# -- the renderer's own backstop: a host that never says "cycle" still terminates ----------------
	var calls := [0]
	var liar := {"stack": ["N0.md"], "embed": func(_inner: String, stack: Array) -> Dictionary:
		calls[0] += 1
		return {"ok": true, "kind": "note", "path": "N%d.md" % stack.size(), "text": "![[x]]", "truncated": false}}
	var out := ED.to_bbcode("![[x]]", "abcdef", "123456", Callable(), liar)
	_check("a lying host: the render ends at the backstop, in a notice", [out.contains("Embed skipped, more than 8 levels deep"), calls[0]], [true, 8])
	_check("... and leaves no state behind for the next render", [ED._embed_stack, ED._readonly, ED._hooks], [[], 0, {}])

	# -- chronos ------------------------------------------------------------------------------------
	_check("axis: three points over five cells", ED.chronos_axis([{"start": 1000}, {"start": 1100}, {"start": 1200}], 5), "1000 ├●─●─●┤ 1200")
	_check("axis: a period is a bar between its ends", ED.chronos_axis([{"start": 0, "end": 10}, {"start": 20}], 5), "0 ├●━●─●┤ 20")
	_check("axis: one year has no axis", ED.chronos_axis([{"start": 1200}, {"start": 1200}]), "")
	_check("axis: no events has none", ED.chronos_axis([]), "")
	_check("axis: negative years", ED.chronos_axis([{"start": -300}, {"start": 200}], 3), "-300 ├●─●┤ 200")
	_check("years: one, a range, a range of one", [ED.chronos_years({"start": 5}), ED.chronos_years({"start": 5, "end": 9}), ED.chronos_years({"start": 5, "end": 5})], ["5", "5–9", "5"])
	var chron := {"ok": true, "blocks": 1, "skipped": ["junk line"], "events": [
		{"kind": "event", "start": 1300, "name": "Siege", "description": "of [Brindle]"},
		{"kind": "period", "start": 1200, "end": 1250, "name": "Peace", "group": "Reign", "color": "ff8800"},
		{"kind": "point", "start": 1300, "name": "Plague", "color": "red"}]}
	var ch_hooks := {"chronos": func(t: String) -> Dictionary:
		return chron if t.begins_with("```chronos\n") else {"ok": false}}
	_check("a chronos block: count, axis, rows in year order (ties as written), a bad colour name ignored, the unread line listed",
		ED.to_bbcode("```chronos\n- [1300] x\n```", "abcdef", "123456", Callable(), ch_hooks),
		"[indent][b]Timeline[/b] [color=#123456]· 3 entries[/color]\n"
		+ "[color=#123456][code]1200 ├●" + "━".repeat(17) + "●" + "─".repeat(16) + "●┤ 1300[/code][/color]\n"
		+ "[color=#ff8800]▬[/color] [url=year:1200][color=#abcdef]1200–1250[/color]  [b]Peace[/b] [color=#123456]{Reign}[/color][/url]\n"
		+ "[color=#abcdef]●[/color] [url=year:1300][color=#abcdef]1300[/color]  [b]Siege[/b] [color=#123456]— of [lb]Brindle][/color][/url]\n"
		+ "[color=#abcdef]◆[/color] [url=year:1300][color=#abcdef]1300[/color]  [b]Plague[/b][/url]\n"
		+ "[color=#e0a840]✖ not read: junk line[/color][/indent]")
	_check("a chronos block with no hook is the code it is", ED.to_bbcode("```chronos\n- [1300] x\n```", "abcdef", "123456"), "[code]- [lb]1300] x[/code]")
	_check("an empty chronos block says so", ED.to_bbcode("```chronos\n```", "abcdef", "123456", Callable(),
		{"chronos": func(_t): return {"ok": true, "blocks": 1, "events": [], "skipped": []}}),
		"[indent][b]Timeline[/b] [color=#123456]· empty[/color][/indent]")

	print("VE7S  RESULT checks=%d failed=%d" % [_n, _fails])
	quit(0 if _fails == 0 else 1)
