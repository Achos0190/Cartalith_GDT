extends SceneTree
## VE-8's pure transforms (`insert_block`, `table_block`, `rule_block`), no UI, headless:
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_ve8static_probe.gd
## Exit 0 = every check held. The windowed `_ve8_probe.gd` repeats these against the live editor.

const Ed := preload("res://shell/markdown_editor.gd")
var n := 0
var fails := []


func ok(label: String, cond: bool, detail: String = "") -> void:
	n += 1
	if cond:
		print("VE8S ok  ", label)
	else:
		fails.append(label)
		print("VE8S !!  ", label, "  ", detail)


func r_(s: Variant) -> String:
	return str(s).replace("\n", "\\n")


## The non-blank lines of `before` appear in `after`, in order, with exactly one extra line `added`.
func _kept(before: String, after: String, added: String) -> bool:
	var a: Array = []
	for l in before.split("\n"):
		if String(l).strip_edges() != "":
			a.append(String(l))
	var b: Array = []
	var seen := 0
	for l in after.split("\n"):
		if String(l).strip_edges() == "":
			continue
		if String(l) == added and seen == 0:
			seen = 1
			continue
		b.append(String(l))
	return seen == 1 and a == b


func _init() -> void:
	var tb := "| Column 1 | Column 2 |\n| --- | --- |\n|   |   |\n|   |   |"
	var r: Dictionary = Ed.table_block("Intro line", 3, 3, 2, 2)
	ok("2x2 after the caret's line behind a blank line, nothing removed", r["text"] == "Intro line\n\n" + tb + "\n", r_(r["text"]))
	ok("first header cell selected", String(r["text"]).substr(int(r["from"]), int(r["to"]) - int(r["from"])) == "Column 1", str(r))
	r = Ed.table_block("A\nB", 0, 0, 2, 2)
	ok("text below gets a blank line between", r["text"] == "A\n\n" + tb + "\n\nB", r_(r["text"]))
	r = Ed.table_block("A\n\nB", 0, 0, 2, 2)
	ok("a blank line already below is not doubled", r["text"] == "A\n\n" + tb + "\n\nB", r_(r["text"]))
	r = Ed.table_block("A\n\nB", 2, 2, 2, 2)
	ok("on a blank line: blank above, blank below", r["text"] == "A\n\n" + tb + "\n\nB", r_(r["text"]))
	r = Ed.table_block("", 0, 0, 3, 3)
	ok("empty note, 3x3", r["text"] == "| Column 1 | Column 2 | Column 3 |\n| --- | --- | --- |\n|   |   |   |\n|   |   |   |\n|   |   |   |\n", r_(r["text"]))
	r = Ed.table_block("x", 0, 0, 99, 0)
	ok("sizes clamp to 1..8", String(r["text"]).count("Column") == 8 and String(r["text"]).split("\n").size() == 6, r_(r["text"]))
	r = Ed.rule_block("Some text", 4, 4)
	ok("rule behind a blank line", r["text"] == "Some text\n\n---\n", r_(r["text"]))
	ok("caret after the rule", int(r["from"]) == String(r["text"]).length() and int(r["to"]) == int(r["from"]), str(r))
	r = Ed.rule_block("a\nb", 0, 0)
	ok("rule before a following line", r["text"] == "a\n\n---\n\nb", r_(r["text"]))
	r = Ed.rule_block("one\ntwo\nthree", 0, 7)
	ok("a multi-line selection: after the line it ENDS on", r["text"] == "one\ntwo\n\n---\n\nthree", r_(r["text"]))
	r = Ed.rule_block("a\n\n\nb", 2, 2)
	ok("rule on a blank line between blank lines", r["text"] == "a\n\n---\n\n\nb", r_(r["text"]))
	for t in ["", "x", "x\n", "a\nb\nc", "# H\n\ntext\n", "  \n"]:
		for at in [0, t.length() / 2, t.length()]:
			var rr: Dictionary = Ed.rule_block(t, at, at)
			ok("rule never loses or reorders a non-blank line (%s @%d)" % [r_(t), at], _kept(t, String(rr["text"]), "---"), r_(rr["text"]))
	print("VE8S RESULT %d checks, %d failed%s" % [n, fails.size(), "" if fails.is_empty() else " :: " + str(fails)])
	quit(0 if fails.is_empty() else 1)
