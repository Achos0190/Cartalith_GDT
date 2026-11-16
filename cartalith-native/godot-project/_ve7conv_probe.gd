extends "res://_pcl_base.gd"
## VE-7 follow-up, Ruling FO / PC-6 PQ5: the assistant conversation really travels in a project save, end to end, now that
## `assistant/conversation.json` is in `cartalith_io::DOCUMENT_SLOTS`. Drives the shell's real `_write_project_busy` and
## `open_project_busy` (not the document helpers alone), with a scratch project and scratch APPDATA, and reads the written
## archive back with `project_open`:
##   A. preference OFF: the saved archive carries NO `assistant/conversation.json` and the save succeeded (control: a slot the
##      engine did not list would have failed the whole save).
##   B. preference ON: the archive carries the slot, version 1, exactly the two parties' turns (no system line), and the
##      engine lists the slot in `project_document_slots()`.
##   C. the transcript is cleared (`restore_conversation_document` of nothing leaves it; a New-like clear via _rebuild), the
##      project is opened through `open_project_busy`, and the same turns come back in order with the "Restored" line.
##   D. an open of the OFF archive (A) leaves the session's conversation alone (PQ5).
## Mutants run by hand: slot removed from DOCUMENT_SLOTS (B fails: the save is refused or the slot is absent), the
## `conv_save` test removed from `conversation_document` (A fails).
##
##   godot --path . --resolution 1600x900 _ve7conv_probe.tscn -- [--shots <dir>]   (windowed, scratch APPDATA)

func _run() -> void:
	_prefix = "ve7conv"
	var aw = _app.assistant_window
	_app.open_context_tab("ask")
	await _settle(8)
	var tmp := OS.get_environment("TEMP").replace("\\", "/")
	var off_path := tmp + "/ve7conv_off_%d.zip" % Time.get_ticks_msec()
	var on_path := tmp + "/ve7conv_on_%d.zip" % Time.get_ticks_msec()
	aw._add_msg("You", "Where is the ford?", "text")
	aw._add_msg("Assistant", "North of the mill.", "text_secondary", true)
	aw._add_msg("", "Note Hidden.md is private and is never sent.", "block")
	_ok(_bridge.project_document_slots().has("assistant/conversation.json"), "the engine lists assistant/conversation.json")

	print("== A. preference off ==")
	DccSettings.set_llm_value("conv_save", false)
	await _app._write_project_busy(off_path)
	await _settle(4)
	_ok(FileAccess.file_exists(off_path), "the save wrote the archive")
	var r_off: Dictionary = _read_docs(off_path)
	_ok(not r_off.has("assistant/conversation.json"), "off: the archive has no conversation member")

	print("== B. preference on ==")
	DccSettings.set_llm_value("conv_save", true)
	await _app._write_project_busy(on_path)
	await _settle(4)
	_ok(FileAccess.file_exists(on_path), "the save with the slot succeeded (an unlisted slot would have failed it whole)")
	var docs := _read_docs(on_path)
	_ok(docs.has("assistant/conversation.json"), "on: the archive carries assistant/conversation.json")
	var parsed = JSON.parse_string(String(docs.get("assistant/conversation.json", "")))
	_ok(typeof(parsed) == TYPE_DICTIONARY and int(parsed.get("version", 0)) == 1, "version 1")
	var msgs: Array = parsed.get("messages", []) if typeof(parsed) == TYPE_DICTIONARY else []
	_ok(msgs.size() == 2 and msgs[0]["who"] == "You" and msgs[0]["text"] == "Where is the ford?" and msgs[1]["who"] == "Assistant" and msgs[1]["text"] == "North of the mill." and bool(msgs[1]["model"]), "exactly the two turns, in order (the system line is not in it): %s" % [msgs])

	print("== C. restore through a real open ==")
	aw._rebuild()
	await _settle(4)
	_ok(aw.conversation_document() == "", "CONTROL: the transcript is empty before the open")
	var opened: bool = await _app.open_project_busy(on_path)
	await _settle(8)
	_ok(opened, "the project opens")
	var doc2: String = aw.conversation_document()
	var p2 = JSON.parse_string(doc2)
	var m2: Array = p2.get("messages", []) if typeof(p2) == TYPE_DICTIONARY else []
	_ok(m2.size() == 2 and m2[0]["text"] == "Where is the ford?" and m2[1]["text"] == "North of the mill.", "the same two turns are back: %s" % [m2])
	await _shot("restored")

	print("== D. an archive without the slot leaves the session alone ==")
	opened = await _app.open_project_busy(off_path)
	await _settle(8)
	var p3 = JSON.parse_string(aw.conversation_document())
	_ok(opened and typeof(p3) == TYPE_DICTIONARY and (p3["messages"] as Array).size() >= 2, "the restored conversation is still there after opening the archive without one")
	DirAccess.remove_absolute(off_path)
	DirAccess.remove_absolute(on_path)


## The archive's conversation member as `{slot: text}` (empty when absent), read straight from the written file with
## `WorldGen.project_read_document`, so the check sees the archive and not the shell's own claim.
func _read_docs(path: String) -> Dictionary:
	var r: Dictionary = _bridge.world_gen.project_read_document(path, "assistant/conversation.json")
	_ok(bool(r.get("ok", false)), "project_read_document answers ok for %s: %s" % [path.get_file(), r.get("error", "")])
	return {"assistant/conversation.json": String(r["text"])} if bool(r.get("present", false)) else {}
