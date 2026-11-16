extends "res://_pcl_base.gd"
## PC-6 leftovers (`PC_SHELL_PROPOSAL.md` 3.5): the Ask note chip and note routing, the subject's task chips, names and years
## in an answer as links, note citations, and "Ask about this". All inside the real shell, real clicks, no model called
## (a reply is placed in the transcript by the harness, because routing and links are what is under test).
##   A. NOTE CHIP: settlement A has a linked public note, B a private one, C none. A: chip reads the note's file name, enabled; a REAL
##      click routes it (`_note` holds it, the picker shows it, the chip is lit); a second click lets it go. With "Send: named
##      Vault notes" off the tooltip says its text is NOT sent.
##   B. PRIVATE: B's chip reads "private, never sent" and is disabled; forcing `_toggle_note_chip()` routes nothing (`_note` empty),
##      and Save is disabled with the reason. C: "none linked", disabled.
##   C. SAVE ROUTING: with no note picked and A selected, a REAL click on Save to note appends to A's note ON DISK, marked
##      model-generated, and the user's own text is unchanged; for C Save is disabled.
##   D. TASK CHIPS: A offers four, a faction two, a culture none (row hidden). A press sets the task and puts the canned request
##      in the prompt, sends NOTHING (no `You` turn, `run.busy()` false); a typed request is never overwritten.
##   E. LINKS: a reply naming A, a faction, "year 312", "40 BC" and an invented place gets exactly four buttons; real clicks select A,
##      select the faction, move the engine's year to 312 and to -40. The invented name has none.
##   F. CITATIONS: a verified `V:` id becomes a link; with A selected, a click shows the Note tab; with C selected, it opens the vault window.
##   G. ASK ABOUT THIS: Inspect's `CtxAskAbout` (real click) opens Ask with the selection chip back on; the context card's
##      collected rows carry ONE `note.open` (Ruling FJ) and ONE `ask.about` for a settlement, none for two hits or a river.
##   H. PHONE-FREE: a pinned panel's subject is selected by `ask_about`.
## Mutants: NOTE_PRIVATE_GATE (the `_note_block` test in `_subject_note` removed), SAVE_NO_ROUTE (`_route_subject_note` call removed from
## `_on_save`), TASK_KEY_TABLE (`"faction"` table emptied), MENTION_BOUNDARY (`_has_word` boundary tests removed),
## MENTION_BC (the BC sign flip removed), CITE_NOTE (the `V:` branch of `_citation_target` removed), ASK_ROWS_KINDS (`landmark`->`river`),
## ASK_FIRST_SELECT (the select inside `ask_about` removed).

func _aw():
	return _app.assistant_window


func _fire_note_chip() -> void:
	await _press(_aw().controls["chip_note"] as Control, "the NOTE chip")
	await _settle(4)


func _run() -> void:
	_prefix = "ask"
	var all: Array = _bridge.settlements()
	var ia := _anchor
	var ib := (ia + 1) % all.size()
	var ic := (ia + 2) % all.size()
	var tid: Callable = func(i: int) -> int: return int((all[i] as Dictionary).get("tid", 0))
	var root := OS.get_environment("APPDATA").path_join("pcl_vault")
	DirAccess.make_dir_recursive_absolute(root)
	for f in [["PlaceNote.md", "# PlaceNote\n\nMy own words.\n"], ["Hidden.md", "---\nprivate: true\n---\n# Hidden\n\nSecret words.\n"], ["Other.md", "# Other\n\nx\n"]]:
		var fa := FileAccess.open(root.path_join(f[0]), FileAccess.WRITE)
		fa.store_string(f[1])
		fa.close()
	_ok(bool(_bridge.vault_connect(root, "Chronicle").get("ok", false)), "a vault is connected")
	_ok(bool(_bridge.vault_attach("settlement", tid.call(ia), "A", "PlaceNote.md", "").get("ok", false)), "A has a public linked note")
	_ok(bool(_bridge.vault_attach("settlement", tid.call(ib), "B", "Hidden.md", "").get("ok", false)), "B has a private linked note")
	DccSettings.set_llm_value("ctx_selection", true)
	_app.select("settlement", ia)
	_app.open_context_tab("ask")
	await _settle(12)
	var aw = _aw()
	_ok(aw.controls.has("chip_note") and aw.controls.has("subject_tasks"), "the Ask tab built the note chip and the task row")
	print("== A. the note chip ==")
	var chip := aw.controls["chip_note"] as Button
	_ok(chip.text == "Note: PlaceNote.md" and not chip.disabled, "A: chip names the note and is enabled (text '%s')" % chip.text)
	_ok(chip.tooltip_text.contains("NOT sent"), "with Send: named Vault notes off the tooltip says the text is not sent")
	await _shot("ask_note_chip_off")
	await _fire_note_chip()
	_ok(not aw._note.is_empty() and String(aw._note["rel"]) == "PlaceNote.md", "a real click routed the note into the window")
	_ok((aw.controls["note"] as OptionButton).selected > 0 and (aw.controls["note"] as OptionButton).get_item_text((aw.controls["note"] as OptionButton).selected).begins_with("PlaceNote.md"), "the picker shows it")
	_ok(aw._note_routed, "the routing flag behind the lit chip is set")
	await _shot("ask_note_chip_on")
	await _fire_note_chip()
	_ok(aw._note.is_empty() and not aw._note_routed, "a second click lets it go")
	print("== B. private and none ==")
	_app.select("settlement", ib)
	await _settle(8)
	chip = aw.controls["chip_note"] as Button
	_ok(chip.text == "Note: private, never sent" and chip.disabled, "B: chip says private, never sent, and is disabled ('%s')" % chip.text)
	aw._toggle_note_chip()
	_ok(aw._note.is_empty() and not aw._note_routed, "forcing the toggle routes nothing for a private note")
	_ok((aw.controls["save"] as Button).disabled, "Save is disabled for B")
	_app.select("settlement", ic)
	await _settle(8)
	chip = aw.controls["chip_note"] as Button
	_ok(chip.text == "Note: none linked" and chip.disabled, "C: 'none linked' and disabled")
	_ok((aw.controls["save"] as Button).disabled, "Save is disabled for C (nothing to save into)")
	print("== C. save routing ==")
	_app.select("settlement", ia)
	await _settle(8)
	aw._reply = "A drafted paragraph."
	aw._refresh()
	var sv := aw.controls["save"] as Button
	_ok(not sv.disabled, "A: Save is enabled with no note picked (the linked note is the target)")
	await _shot("ask_save_enabled")
	if await _press(sv, "Save to note"):
		await _settle(8)
		var txt := FileAccess.get_file_as_string(root.path_join("PlaceNote.md"))
		_ok(txt.begins_with("# PlaceNote\n\nMy own words.\n") and txt.contains("A drafted paragraph."), "the paragraph was appended to the note on disk, the user's text intact")
		_ok(txt.to_lower().contains("model"), "and marked as model-generated")
	aw._reply = ""
	aw._unroute_note()
	aw._note = {}
	print("== D. task chips ==")
	_app.select("settlement", ia)
	await _settle(8)
	var tr := aw.controls["subject_tasks"] as Control
	_ok(tr.visible and tr.get_child_count() == 4, "a settlement offers four task chips (got %d)" % tr.get_child_count())
	aw._input.text = ""
	var d := tr.find_child("SubjectTask_describe", true, false) as Button
	if await _press(d, "Describe"):
		_ok(aw._task == "prose" and aw._input.text == "Describe this place.", "Describe sets the prose task and its request")
		_ok(not aw.run.busy() and aw._conv.is_empty(), "nothing was sent (no turn, no running reply)")
	aw._input.text = "my own words"
	var h := tr.find_child("SubjectTask_history", true, false) as Button
	if await _press(h, "History"):
		_ok(aw._task == "story" and aw._input.text == "my own words", "a typed request is never overwritten")
	aw._input.text = ""
	var lk := tr.find_child("SubjectTask_links", true, false) as Button
	if await _press(lk, "Suggest links"):
		_ok(aw._task == "note" and aw._note_action == "note_links" and not aw._note.is_empty(), "Suggest links is a Note task and routed the subject's note")
	await _shot("ask_task_chips")
	aw._unroute_note()
	_app.select("faction", _f1)
	await _settle(8)
	tr = aw.controls["subject_tasks"] as Control
	_ok(tr.visible and tr.get_child_count() == 2, "a faction offers two (got %d)" % tr.get_child_count())
	_app.select("culture", 0)
	await _settle(8)
	tr = aw.controls["subject_tasks"] as Control
	_ok(not tr.visible, "a culture offers none: the row is hidden")
	print("== E. names and years as links ==")
	_app.select("settlement", ia)
	await _settle(6)
	var na := String((all[ia] as Dictionary).get("name", ""))
	var nf := String((_bridge.get_factions()[0] as Dictionary).get("name", ""))
	aw._reply = "%s lies under %s since year 312, and was sacked in 40 BC. Zzyzx Qwertia is unknown." % [na, nf]
	aw._reply_label = aw._add_msg("Assistant", aw._reply, "text_secondary", true)
	aw._add_mentions()
	await _settle(6)
	var mrow: Node = aw.controls["messages"].find_child("Mentions", true, false)
	_ok(mrow != null, "the reply got a MENTIONED row")
	var btns: Array = mrow.get_children().filter(func(c): return c is Button) if mrow != null else []
	var yrs := btns.filter(func(b): return (b as Node).has_meta("mention_year")).map(func(b): return int((b as Node).get_meta("mention_year")))
	yrs.sort()
	_ok(yrs == [-40, 312], "exactly the two years became links (got %s)" % str(yrs))
	var names := btns.filter(func(b): return (b as Node).has_meta("mention_target"))
	_ok(names.size() == 2, "exactly the settlement and the faction became links (got %d)" % names.size())
	_ok(not btns.any(func(b): return (b as Button).text.contains("Zzyzx")), "the invented name has no link")
	await _shot("ask_mentions")
	## A name INSIDE a longer word is not a mention (whole words only).
	var embedded: Array = aw._mentions_in("The %sish road and the x%s mill." % [na, na])
	_ok(embedded.filter(func(m): return m.has("kind") and m["kind"] == "settlement" and int(m["id"]) == ia).is_empty(), "a name inside a longer word has no link (got %d hits)" % embedded.size())
	_app.select("faction", _f2)
	await _settle(4)
	for b in names:
		if (b as Node).get_meta("mention_target")["kind"] == "settlement":
			if await _press(b, "the settlement mention"):
				_ok(_app.selection == {"kind": "settlement", "id": ia}, "it selected the settlement")
	for b in btns:
		if (b as Node).has_meta("mention_year") and int((b as Node).get_meta("mention_year")) == -40:
			if await _press(b, "the 40 BC mention"):
				_ok(_year() == -40, "the BC year moved the timeline to -40 (got %d)" % _year())
	for b in btns:
		if (b as Node).has_meta("mention_year") and int((b as Node).get_meta("mention_year")) == 312:
			if await _press(b, "the year 312 mention"):
				_ok(_year() == 312, "year 312 moved the timeline to 312 (got %d)" % _year())
	print("== F. note citations ==")
	var tgt: Dictionary = aw._citation_target("V:PlaceNote.md#Place > Sub")
	_ok(tgt == {"kind": "note", "rel": "PlaceNote.md", "heading": "Sub"}, "a V: id parses to the note and its last heading (got %s)" % str(tgt))
	_ok(aw._citation_target("V:") == {} and aw._citation_target("W:settlement:99999").is_empty(), "an empty V: id and an unknown world id name nothing")
	aw._on_sources({"sources": [], "citations": [{"id": "V:PlaceNote.md#Place", "verified": true, "label": "1", "title": "PlaceNote"}]})
	await _settle(6)
	_app.select("settlement", ia)
	await _settle(6)
	var link: Button = null
	for b in aw.controls["messages"].find_children("*", "Button", true, false):
		if (b as Node).has_meta("cite_target") and String(((b as Node).get_meta("cite_target"))["kind"]) == "note":
			link = b
	_ok(link != null, "the verified note citation is a link")
	if link != null and await _press(link, "the note citation"):
		await _settle(8)
		_ok(_app.right_dock_ctrl.current_tab() == "note", "with its own subject selected, the Note tab shows it (tab %s)" % _app.right_dock_ctrl.current_tab())
	_app.open_context_tab("ask")
	_app.select("settlement", ic)
	await _settle(6)
	if link != null and is_instance_valid(link) and await _press(link, "the note citation (other subject)"):
		await _settle(8)
		_ok(_app.vault_window.visible, "with another subject, the vault window opens the note")
		_app.vault_window.hide()
	print("== G. Ask about this ==")
	_app.open_context_tab("inspect")
	_app.select("settlement", ia)
	await _settle(8)
	aw._left_out["selection"] = true
	var turns_before: int = aw._conv.size()
	var ab := _app.right_dock_body.find_child("CtxAskAbout", true, false) as Button
	_ok(ab != null, "Inspect's header has Ask about this")
	if ab != null and await _press(ab, "Ask about this"):
		await _settle(10)
		_ok(_app.right_dock_ctrl.current_tab() == "ask" and not bool(aw._left_out["selection"]), "it opened Ask and put the selection chip back on")
		_ok(not aw.run.busy() and aw._conv.size() == turns_before, "and sent nothing")
	await _shot("ask_about")
	var br = _app.context_broker
	var rows: Array = br.collect({"hits": [{"kind": "settlement", "id": ia}], "domain": "civilization", "form": "desktop", "gx": 10.0, "gy": 10.0})
	var ids := rows.map(func(r): return String((r as Dictionary).get("id", "")))
	_ok(ids.count("note.open") == 1 and ids.count("ask.about") == 1, "the card carries ONE note.open and ONE ask.about (Ruling FJ) (ids %s)" % str(ids.filter(func(i): return i == "note.open" or i == "ask.about")))
	_ok(br.ask_rows({"hits": [{"kind": "settlement", "id": ia}, {"kind": "settlement", "id": ib}]}).is_empty(), "two hits: no ask row")
	_ok(br.ask_rows({"hits": [{"kind": "river", "id": 1}]}).is_empty(), "a river hit: no ask row")
	print("== H. a pinned subject ==")
	_app.select("faction", _f1)
	await _settle(6)
	_app.right_dock_ctrl.pin()
	_app.select("settlement", ib)
	await _settle(6)
	_ok(_app.ask_about(_app.right_dock_ctrl.shown_subject()), "ask_about(the pinned subject) opens")
	await _settle(6)
	_ok(_app.selection == {"kind": "faction", "id": _f1}, "and selected the pinned subject first (got %s)" % str(_app.selection))
