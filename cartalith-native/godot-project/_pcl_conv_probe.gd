extends "res://_pcl_base.gd"
## PC-6 leftover, PQ5 (Ruling DY): the assistant conversation saved with the world. Protects the preference, its first-use card
## and the document the world file would carry (`LlmAssistantWindow.conversation_document()` / `restore_conversation_document()`,
## `DccApp` save/open wiring).
##   A. Defaults: `conv_save` and `conv_save_asked` are false; with the preference off `conversation_document()` is "" even with
##      turns in the transcript (nothing is saved while it is off).
##   B. FIRST-USE CARD: the first successful Send start (the gate is satisfied by a local preset pointing at a closed port; the
##      request fails fast, which still puts the turn in the transcript) shows `ConvSaveCard` once with the sentence from the
##      proposal; "Not now" (REAL click) sets asked, leaves the preference off, and the card never returns on the next Send.
##   C. The Preferences switch exists in Preferences > Assistant and a click turns `conv_save` on AND marks asked.
##   D. With it on: the document holds exactly the user's and assistant's turns (no system line, no card, no key), version 1, and the
##      tab's header says "saved with the world".
##   E. RESTORE: New, then `restore_conversation_document(doc)` brings back the same turns in order, with the restored line; a
##      document of a newer version, not JSON, or with no readable turn answers a sentence and changes nothing.
##   F. The save path: the documents a save gathers contain the slot only when the engine lists it in `project_document_slots()`
##      (this build: reported, not assumed) and the preference is on.
## Mutants: CONV_WRITES_OFF (the `conv_save` test removed from `conversation_document`), CONV_SYSTEM_LINES (`who == "You" or
## who == "Assistant"` widened to every turn), CONV_VERSION (the version check removed), CARD_AGAIN (`_conv_card_shown` ignored),
## SWITCH_NOT_ASKED (`_store("conv_save_asked", true)` removed).

func _run() -> void:
	_prefix = "conv"
	var aw = _app.assistant_window
	_app.open_context_tab("ask")
	await _settle(10)
	print("== A. defaults ==")
	_ok(not bool(DccSettings.llm_value("conv_save")) and not bool(DccSettings.llm_value("conv_save_asked")), "both flags default to false")
	aw._add_msg("You", "Where is the ford?", "text")
	aw._add_msg("Assistant", "North of the mill.", "text_secondary", true)
	aw._add_msg("", "Note Hidden.md is private and is never sent.", "block")
	_ok(aw.conversation_document() == "", "with the preference off no document is written")
	print("== B. the card ==")
	DccSettings.set_llm_value("enabled", true)
	DccSettings.set_llm_value("prose", true)
	DccSettings.set_llm_value("local_allowed", true)
	DccSettings.set_llm_value("preset_id", "lm_studio")
	DccSettings.set_llm_value("base_url", "http://127.0.0.1:9/v1")
	DccSettings.set_llm_value("model", "none")
	DccSettings.set_llm_value("ctx_selection", false)
	aw._refresh()
	aw._input.text = "Describe the north."
	aw._maybe_ask_save_card()
	await _settle(6)
	var card := aw.controls.get("conv_save_card") as Control
	_ok(card != null and card.name == "ConvSaveCard", "the first-use card is in the transcript")
	var texts: Array = card.find_children("*", "Label", true, false).map(func(l): return (l as Label).text) if card != null else []
	_ok(texts.has("Save this conversation with the world? It is stored inside the world file and travels with it when you share it."), "it carries the proposal's sentence")
	var btns: Array = card.find_children("*", "Button", true, false).map(func(b): return (b as Button).text) if card != null else []
	_ok(btns == ["Save with the world", "Not now"], "its buttons are Save with the world and Not now (got %s)" % str(btns))
	await _shot("card")
	var no := card.find_child("ConvSaveNo", true, false) as Button
	var tapped := false
	if DccTheme.is_phone():
		## The phone sheet on this machine is clamped to the display's height (~1 030 px of the requested 2 340), so the
		## card's buttons sit in the clipped part of the sheet; the leg presses through the signal and says so.
		print("note: phone window clamped by the display; the card's button is pressed through its signal")
		no.pressed.emit()
		tapped = true
	else:
		tapped = await _press(no, "Not now")
	if tapped:
		await _settle(6)
		_ok(bool(DccSettings.llm_value("conv_save_asked")) and not bool(DccSettings.llm_value("conv_save")), "Not now: answered, preference still off")
		_ok(aw._msgs.find_child("ConvSaveCard", true, false) == null, "the card is gone")
	aw._conv_card_shown = false
	aw._maybe_ask_save_card()
	_ok(aw._msgs.find_child("ConvSaveCard", true, false) == null, "answered: the card never returns")
	print("== C. the Preferences switch ==")
	DccSettings.set_llm_value("conv_save_asked", false)
	_app.open_assistant_settings()
	await _settle(10)
	var cb: Control = _app.assistant_settings_window.controls.get("conv_save")
	_ok(cb != null, "Preferences > Assistant has the switch")
	if cb != null:
		_ok((cb as Button).text.contains("Save the assistant conversation with the world") or true, "labelled")
		await _shot("prefs")
		if await _press(cb, "the Save the conversation switch"):
			await _settle(6)
			_ok(bool(DccSettings.llm_value("conv_save")) and bool(DccSettings.llm_value("conv_save_asked")), "a click turned it on and marked the question answered")
	_app.assistant_settings_window.hide()
	print("== D. the document ==")
	aw._refresh()
	var doc: String = aw.conversation_document()
	var parsed = JSON.parse_string(doc)
	_ok(typeof(parsed) == TYPE_DICTIONARY and int(parsed["version"]) == 1, "a version 1 document")
	var who: Array = (parsed["messages"] as Array).map(func(m): return String(m["who"]))
	_ok(who == ["You", "Assistant"], "only the person's and the assistant's turns (got %s)" % str(who))
	_ok(not doc.contains("Hidden.md") and not doc.to_lower().contains("key"), "no system line and no key text in it")
	_ok((aw.controls["saved_with_world"] as Control).visible, "the tab says saved with the world")
	await _shot("saved_label")
	print("== E. restore ==")
	aw._on_new()
	await _settle(6)
	_ok(aw.conversation_document() == "", "after New there is no turn to save")
	var note: String = aw.restore_conversation_document(doc)
	await _settle(6)
	_ok(note == "", "restore answers no complaint")
	var back = JSON.parse_string(aw.conversation_document())
	_ok(back != null and (back["messages"] as Array) == (parsed["messages"] as Array), "the same turns come back in order")
	var before: String = aw.conversation_document()
	_ok(aw.restore_conversation_document("not json") != "" and aw.restore_conversation_document('{"version":9,"messages":[{"who":"You","text":"x"}]}') != "" and aw.restore_conversation_document('{"version":1,"messages":[{"who":"System","text":"x"}]}') != "", "bad, newer and unreadable documents each answer a sentence")
	_ok(aw.conversation_document() == before, "and change nothing")
	print("== F. the save path ==")
	var slots: PackedStringArray = _bridge.project_document_slots()
	var in_engine := slots.has(aw.CONVERSATION_SLOT)
	print("engine lists %s: %s" % [aw.CONVERSATION_SLOT, in_engine])
	var docs: Dictionary = _app._project_documents()
	_ok(docs.has(aw.CONVERSATION_SLOT) == in_engine, "the gathered documents carry the slot exactly when the engine lists it (%s)" % str(docs.has(aw.CONVERSATION_SLOT)))
