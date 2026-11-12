extends SceneTree
## The seventeen `vault_*` `#[func]`s the note editor added that no probe called (`OUTSTANDING_WORK.md`, "Vault editor
## and linking" row): `vault_create_file`, `vault_embed_open`, `vault_frontmatter_remove`, `vault_link_at`,
## `vault_link_mention`, `vault_mention_spans`, `vault_note_aliases`, `vault_note_chronos`, `vault_note_properties`,
## `vault_note_tags`, `vault_outgoing_links`, `vault_private_marks`, `vault_read_image`, `vault_resolve_link`,
## `vault_suggest`, `vault_suggest_notes` and `vault_toggle_task`. Each is called across the real `#[func]`
## boundary and checked against a LITERAL answer worked out from the function's documented contract, never against
## what the code returned the first time. Each also gets its refusal or absent case.
##
##   Godot_v4.7.2-stable_win64_console.exe --headless --path . --script res://_vefuncs_probe.gd
##
## Logic only, no pixels, so headless is right (`MISTAKES.md`: headless for logic). Exit 0 = every check held,
## 1 = a check failed, 2 = could not run. Reads and writes a throw-away vault in TEMP, removed at the end.

var _fails := 0
var _checks := 0
var _root := ""


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("  FAIL: ", what)


func _write(rel: String, text: String) -> void:
	var p := _root + "/" + rel
	DirAccess.make_dir_recursive_absolute(p.get_base_dir())
	var f := FileAccess.open(p, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _init() -> void:
	var b := EngineBridge.new()
	get_root().add_child(b)
	for fn in ["vault_create_file", "vault_embed_open", "vault_frontmatter_remove", "vault_link_at", "vault_link_mention", "vault_mention_spans", "vault_note_aliases", "vault_note_chronos", "vault_note_properties", "vault_note_tags", "vault_outgoing_links", "vault_private_marks", "vault_read_image", "vault_resolve_link", "vault_suggest", "vault_suggest_notes", "vault_toggle_task"]:
		if not b.world_gen.has_method(fn):
			print("  ABORT: the loaded extension has no %s; rebuild first" % fn)
			quit(2)
			return

	# -- Pure text functions (no vault needed) ---------------------------------------------------------------------
	var note := "---\naliases: [Narethia, \"The Ford\", Narethia]\ntags: [town, river]\n---\n# Nareth\n\nSee [[Trade Routes|trade]] and #harbour. `#nottag` and https://x.org/#frag\n"

	# vault_note_aliases: in order, the duplicate (a case variant) dropped, quotes stripped.
	var al: PackedStringArray = b.world_gen.vault_note_aliases(note)
	_ok(al == PackedStringArray(["Narethia", "The Ford"]), "aliases: %s" % [al])
	_ok(b.world_gen.vault_note_aliases("---\nalias: Lone\n---\n") == PackedStringArray(["Lone"]), "aliases: the singular `alias` key and a bare scalar")
	_ok(b.world_gen.vault_note_aliases("# no frontmatter\n").is_empty(), "aliases: none without front matter")

	# vault_note_tags: the two frontmatter tags (frontmatter true) and one inline; not the code span, the URL fragment or a heading.
	var tags: Array = b.world_gen.vault_note_tags(note)
	var seen := {}
	for t in tags:
		seen[String((t as Dictionary)["tag"])] = bool((t as Dictionary)["frontmatter"])
	_ok(seen == {"town": true, "river": true, "harbour": false}, "tags: %s" % [seen])

	# vault_note_properties.
	var pr: Dictionary = b.world_gen.vault_note_properties(note)
	_ok(bool(pr["present"]) and bool(pr["editable"]), "properties: present and editable")
	var keys := []
	for p in pr["properties"]:
		keys.append(String((p as Dictionary)["key"]) + ":" + str((p as Dictionary)["list"]))
	_ok(keys == ["aliases:true", "tags:true"], "properties: keys and list-ness %s" % [keys])
	var none: Dictionary = b.world_gen.vault_note_properties("# bare\n")
	_ok(not bool(none["present"]) and bool(none["editable"]) and (none["properties"] as Array).is_empty(), "properties: a note with none is present=false but editable (setting one creates the block)")
	var dup: Dictionary = b.world_gen.vault_note_properties("---\na: 1\na: 2\n---\n")
	_ok(bool(dup["present"]) and not bool(dup["editable"]), "properties: a repeated key is shown but not editable")

	# vault_frontmatter_remove: only that key goes; every other line is byte-identical.
	var rm: Dictionary = b.world_gen.vault_frontmatter_remove(note, "tags")
	_ok(bool(rm["ok"]) and bool(rm["removed"]), "remove: removed")
	_ok(String(rm.get("text", "")) == note.replace("tags: [town, river]\n", ""), "remove: only the tags line is gone, byte for byte")
	var rm2: Dictionary = b.world_gen.vault_frontmatter_remove(note, "nope")
	_ok(bool(rm2["ok"]) and not bool(rm2["removed"]) and not rm2.has("text"), "remove: a missing key is ok, removed false, and carries no text")

	# vault_link_at: offsets are characters. `see ` is 4 characters; the link is 22 characters long (4..26).
	var lt := "see [[Trade Routes|trade]] ok"
	var la: Dictionary = b.world_gen.vault_link_at(lt, 8)
	_ok(bool(la["found"]) and String(la["target"]) == "Trade Routes" and String(la["alias"]) == "trade" and String(la["syntax"]) == "wiki" and int(la["start"]) == 4 and int(la["end"]) == 26, "link_at: inside the link %s" % [la])
	_ok(bool(b.world_gen.vault_link_at(lt, 26)["found"]), "link_at: a caret just past ]] still finds it")
	_ok(not bool(b.world_gen.vault_link_at(lt, 3)["found"]), "link_at: before the link finds nothing")
	_ok(not bool(b.world_gen.vault_link_at(lt, -1)["found"]), "link_at: a negative offset finds nothing")
	_ok(not bool(b.world_gen.vault_link_at(lt, 500)["found"]), "link_at: past the end finds nothing")

	# vault_outgoing_links: notes only; not a URL, an attachment, or a same-note link.
	var ot := "[[A]] [[#Self]] https://x.org [md](B.md) ![[pic.png]] [[C#H|c]]"
	var og: Array = b.world_gen.vault_outgoing_links(ot)
	var targets := []
	for l in og:
		targets.append(String((l as Dictionary).get("target", "")))
	_ok(targets == ["A", "B.md", "C"], "outgoing links: %s" % [targets])

	# vault_mention_spans: whole word, any case; not inside a link, code or a URL; not part of a longer word.
	var mt := "Nareth and nareth, [[Nareth]], `Nareth`, Narethia, https://x.org/Nareth"
	var ms: Array = b.world_gen.vault_mention_spans(mt, "Nareth")
	var m_text := []
	var m_start := []
	for m in ms:
		m_text.append(String((m as Dictionary)["text"]))
		m_start.append(int((m as Dictionary)["start"]))
	_ok(m_text == ["Nareth", "nareth"] and m_start == [0, 11], "mention spans: %s at %s" % [m_text, m_start])
	_ok(b.world_gen.vault_mention_spans(mt, "").is_empty(), "mention spans: an empty name finds nothing")

	# vault_link_mention: the written text is kept as the alias when it differs from the target.
	var lm: Dictionary = b.world_gen.vault_link_mention("Nareth is old", 0, 6, "Places/Nareth")
	_ok(bool(lm["ok"]) and String(lm["text"]) == "[[Places/Nareth|Nareth]] is old", "link_mention: aliased when the written text differs %s" % [lm])
	var lm2: Dictionary = b.world_gen.vault_link_mention("Nareth is old", 0, 6, "Nareth")
	_ok(bool(lm2["ok"]) and String(lm2["text"]) == "[[Nareth]] is old", "link_mention: plain when the text is the target %s" % [lm2])
	_ok(not bool(b.world_gen.vault_link_mention("abc", 2, 99, "X")["ok"]), "link_mention: a span past the end is refused")

	# vault_toggle_task.
	var tt := "- [ ] a\n- [x] b\nplain\n```\n- [ ] code\n```\n"
	var t0: Dictionary = b.world_gen.vault_toggle_task(tt, 0)
	_ok(bool(t0["ok"]) and bool(t0["checked"]) and String(t0["text"]) == tt.replace("- [ ] a", "- [x] a"), "toggle: ticks line 0, nothing else moves")
	var t1: Dictionary = b.world_gen.vault_toggle_task(tt, 1)
	_ok(bool(t1["ok"]) and not bool(t1["checked"]) and String(t1["text"]) == tt.replace("- [x] b", "- [ ] b"), "toggle: unticks line 1")
	_ok(not bool(b.world_gen.vault_toggle_task(tt, 2)["ok"]), "toggle: a line that is not a task is refused")
	_ok(not bool(b.world_gen.vault_toggle_task(tt, 4)["ok"]), "toggle: a task inside a code fence is refused")
	_ok(not bool(b.world_gen.vault_toggle_task(tt, 99)["ok"]), "toggle: a line past the end is refused")

	# vault_note_chronos: a block of one event, one point and one unreadable line (reported, not dropped).
	var ct := "text\n```chronos\n- [1200~1250] #red {War} Siege | long\n* [1300] Founding\nnonsense line\n```\n"
	var ch: Dictionary = b.world_gen.vault_note_chronos(ct)
	_ok(bool(ch["ok"]) and int(ch["blocks"]) == 1 and (ch["events"] as Array).size() == 2, "chronos: one block, two events %s" % [ch])
	if (ch["events"] as Array).size() == 2:
		var e0: Dictionary = ch["events"][0]
		_ok(int(e0["start"]) == 1200 and int(e0["end"]) == 1250 and String(e0["name"]) == "Siege", "chronos: the range event keeps its dates and name %s" % [e0])
		_ok(int(ch["events"][1]["start"]) == 1300 and String(ch["events"][1]["name"]) == "Founding", "chronos: the point event")
	_ok((ch["skipped"] as Array).size() == 1, "chronos: the unreadable line is reported in skipped, not dropped")
	_ok(int(b.world_gen.vault_note_chronos("no block here\n")["blocks"]) == 0, "chronos: a note with no block has blocks 0")

	# vault_suggest: ranks the caller's own candidates; one without a name is skipped; `index` is the caller's position.
	var cands := [{"name": "Narrow Pass", "path": "p.md"}, {"name": "Nareth"}, {"nothing": 1}, {"name": "Harbour", "aliases": ["Nar Docks"]}, {"name": "Zed"}]
	var sg: Array = b.world_gen.vault_suggest("nar", cands, 10)
	var order := []
	for r in sg:
		order.append(String((r as Dictionary)["name"]))
	_ok(not order.has("Zed") and order.has("Nareth") and order.has("Narrow Pass") and order.has("Harbour"), "suggest: the three matches and not Zed %s" % [order])
	_ok(order.find("Nareth") < order.find("Narrow Pass"), "suggest: ties inside a class are alphabetical (Nareth before Narrow Pass) %s" % [order])
	for r in sg:
		var rd: Dictionary = r
		if String(rd["name"]) == "Harbour":
			_ok(String(rd.get("alias", "")) == "Nar Docks" and int(rd["index"]) == 3, "suggest: an alias match carries the alias and the candidate's index %s" % [rd])
		if String(rd["name"]) == "Narrow Pass":
			_ok(String(rd.get("path", "")) == "p.md" and int(rd["index"]) == 0, "suggest: path and index are the candidate's")
	_ok((b.world_gen.vault_suggest("nar", cands, 1) as Array).size() == 1, "suggest: limit 1 returns one")

	# -- Vault-backed functions: first with no vault bound -----------------------------------------------------------
	_ok(not bool(b.world_gen.vault_resolve_link("a.md", "x")["ok"]), "resolve_link: no vault bound answers ok false (never a status)")
	_ok(not bool(b.world_gen.vault_create_file("n.md", "x")["ok"]), "create_file: no vault bound is refused")
	_ok((b.world_gen.vault_suggest_notes("nar", 5) as Array).is_empty(), "suggest_notes: no vault bound suggests nothing")
	var pm0: Dictionary = b.world_gen.vault_private_marks(PackedStringArray(["a.md"]), 0)
	_ok((pm0.get("marks", {}) as Dictionary).is_empty() and (pm0.get("decided", []) as Array).is_empty(), "private_marks: with no vault a note is undecided, never public %s" % [pm0])
	_ok(not bool(b.world_gen.vault_embed_open("a.md", "b", PackedStringArray())["ok"]), "embed_open: no vault bound answers ok false")
	_ok(b.world_gen.vault_read_image("img/map.png").is_empty(), "read_image: no vault bound is empty")

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vefuncs-%d" % Time.get_ticks_msec()
	_write("Places/Nareth.md", "---\naliases: [Narethia]\n---\n# Nareth\n\nSee [[Trade Routes]].\n")
	_write("Trade Routes.md", "# Trade Routes\n\nGoes to [[Places/Nareth]].\n")
	_write("Sub/Dup.md", "# Dup one\n")
	_write("Other/Dup.md", "# Dup two\n")
	_write("Open.md", "# Open\n\nnothing secret\n")
	_write("Private/Hidden.md", "# Hidden\n")
	var img := Image.create(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(1, 0, 0))
	DirAccess.make_dir_recursive_absolute(_root + "/img")
	img.save_png(_root + "/img/map.png")
	var png: PackedByteArray = FileAccess.get_file_as_bytes(_root + "/img/map.png")
	var cn: Dictionary = b.vault_connect(_root, "Probe")
	_ok(bool(cn.get("ok", false)), "CONTROL: the fixture vault binds %s" % [cn])

	# vault_create_file.
	var cf: Dictionary = b.world_gen.vault_create_file("New/Made.md", "# Made\n")
	_ok(bool(cf["ok"]) and String(cf.get("hash", "")) != "" and FileAccess.get_file_as_string(_root + "/New/Made.md") == "# Made\n", "create_file: makes the note and its folder %s" % [cf])
	var cf2: Dictionary = b.world_gen.vault_create_file("New/Made.md", "# overwritten?\n")
	_ok(not bool(cf2["ok"]) and String(cf2.get("kind", "")) == "exists" and FileAccess.get_file_as_string(_root + "/New/Made.md") == "# Made\n", "create_file: an existing note answers kind exists and is untouched %s" % [cf2])
	_ok(not bool(b.world_gen.vault_create_file("../escape.md", "x")["ok"]) and not FileAccess.file_exists(_root.get_base_dir() + "/escape.md"), "create_file: a path that leaves the vault is refused and writes nothing")

	# vault_resolve_link.
	var rl: Dictionary = b.world_gen.vault_resolve_link("Places/Nareth.md", "Trade Routes")
	_ok(bool(rl["ok"]) and String(rl["status"]) == "unique" and String(rl["path"]) == "Trade Routes.md", "resolve_link: a unique name %s" % [rl])
	var rl2: Dictionary = b.world_gen.vault_resolve_link("Places/Nareth.md", "Dup")
	var cs: Array = rl2.get("candidates", [])
	cs.sort()
	_ok(bool(rl2["ok"]) and String(rl2["status"]) == "ambiguous" and cs == ["Other/Dup.md", "Sub/Dup.md"], "resolve_link: two notes of one name are ambiguous, both listed %s" % [rl2])
	_ok(String(b.world_gen.vault_resolve_link("Places/Nareth.md", "Nope")["status"]) == "none", "resolve_link: no such note is none")

	# vault_suggest_notes (reads the vault; aliases only once the index is built, so the name path is asserted).
	var sn: Array = b.world_gen.vault_suggest_notes("nar", 5)
	var sn_names := []
	for r in sn:
		sn_names.append(String((r as Dictionary)["path"]))
	_ok(sn_names.has("Places/Nareth.md") and not sn_names.has("Open.md"), "suggest_notes: ranks the vault's notes by name %s" % [sn_names])
	_ok((b.world_gen.vault_suggest_notes("nar", 1) as Array).size() <= 1, "suggest_notes: the limit holds")

	# vault_private_marks.
	var pm: Dictionary = b.world_gen.vault_private_marks(PackedStringArray(["Open.md", "Private/Hidden.md", "Missing.md"]), 0)
	var marks: Dictionary = pm["marks"]
	_ok(bool(pm["ok"]) and marks.has("Private/Hidden.md") and String(marks["Private/Hidden.md"]) == "a folder named private", "private_marks: a note in Private/ is marked, with the fixed reason %s" % [marks])
	_ok(not marks.has("Open.md") and (pm["decided"] as Array).has("Open.md") and (pm["decided"] as Array).has("Private/Hidden.md"), "private_marks: a read public note is decided and unmarked")
	_ok(not marks.has("Missing.md") and not (pm["decided"] as Array).has("Missing.md"), "private_marks: an unreadable note is UNDECIDED, never public")

	# vault_embed_open and vault_read_image.
	var en: Dictionary = b.world_gen.vault_embed_open("Places/Nareth.md", "Trade Routes", PackedStringArray(["Places/Nareth.md"]))
	_ok(bool(en["ok"]) and String(en["kind"]) == "note" and String(en["path"]) == "Trade Routes.md" and String(en["text"]).contains("Goes to"), "embed_open: a note embeds its text %s" % [en])
	var ei: Dictionary = b.world_gen.vault_embed_open("Places/Nareth.md", "img/map.png", PackedStringArray(["Places/Nareth.md"]))
	_ok(bool(ei["ok"]) and String(ei["kind"]) == "image" and String(ei["path"]) == "img/map.png", "embed_open: a picture is kind image %s" % [ei])
	_ok(String(b.world_gen.vault_embed_open("Places/Nareth.md", "Nope", PackedStringArray())["kind"]) == "missing", "embed_open: no such note is missing")
	_ok(String(b.world_gen.vault_embed_open("Places/Nareth.md", "Dup", PackedStringArray())["kind"]) == "ambiguous", "embed_open: two notes of one name are ambiguous")
	_ok(String(b.world_gen.vault_embed_open("Trade Routes.md", "Places/Nareth", PackedStringArray(["Trade Routes.md", "Places/Nareth.md"]))["kind"]) == "cycle", "embed_open: a note already being drawn is a cycle, not a recursion")
	_ok(String(b.world_gen.vault_embed_open("Places/Nareth.md", "../../etc/passwd", PackedStringArray())["kind"]) != "note", "embed_open: an address that leaves the vault is never a note")
	_ok(b.world_gen.vault_read_image("img/map.png") == png and png.size() > 0, "read_image: returns the picture's bytes unchanged")
	_ok(b.world_gen.vault_read_image("../escape.png").is_empty(), "read_image: a path that leaves the vault is empty")
	_ok(b.world_gen.vault_read_image("Open.md").is_empty(), "read_image: a note is not a picture")
	_ok(b.world_gen.vault_read_image("img/none.png").is_empty(), "read_image: a missing picture is empty")

	DirAccess.remove_absolute(_root + "/img/map.png")
	print("vefuncs probe: %d checks, %d failures" % [_checks, _fails])
	quit(0 if _fails == 0 else 1)
