extends "res://_pcl_base.gd"
## PC-5 / PC-10 leftovers: Pop out for the Note tab and for the Context module windows (`PC_SHELL_PROPOSAL.md` 3.3).
##   A. `context_module_window` is in `PopoutHost.LIVE_KEYS`; a module window ("Links") opened from the panel has a
##      `PopoutButton` reading "Pop out" and, with the Pop out pressed for real, becomes a native window
##      (`not is_embedded()`) where the platform allows; pressing it again docks it back. Where subwindows are unsupported
##      the button is absent (`PopoutHost.supported()`).
##   B. The Note tab, with a settlement that has a linked note: the docked editor shows `NotePopOut`; a REAL click opens
##      the note in the vault window (`vault_window.visible`, the same path) and the Note tab reports it ("open in the vault
##      window"); the editor exists once (the tab's frame no longer holds it).
##   C. The Note tab with no note shows no Pop out.
##   D. TABLET: the module window and the Note pop-out are not offered (`NotePopOut` absent).
## Mutants: POP_KEY_MISSING (`context_module_window` removed from LIVE_KEYS), POP_NO_BUTTON (`add_toggle_button` removed),
## NOTE_POP_NO_RELEASE (`release_dock()` not called before `open_note`), NOTE_POP_SHOWN_TABLET (the tablet test dropped).

func _run() -> void:
	_prefix = "pop"
	var tablet := DccTheme.is_tablet()
	var tid := int((_bridge.settlements()[_anchor] as Dictionary).get("tid", 0))
	var vault_dir := OS.get_environment("APPDATA").path_join("pcl_vault")
	DirAccess.make_dir_recursive_absolute(vault_dir)
	var fa := FileAccess.open(vault_dir.path_join("PlaceNote.md"), FileAccess.WRITE)
	fa.store_string("# Place

A note.
")
	fa.close()
	var conn: Dictionary = _bridge.vault_connect(vault_dir, "Chronicle")
	_ok(bool(conn.get("ok", false)), "a vault is connected")
	_ok(bool(_bridge.vault_attach("settlement", tid, "Place", "PlaceNote.md", "").get("ok", false)), "the settlement has a linked note")
	_app.select("settlement", _anchor)
	await _settle(8)
	print("== A. module window ==")
	_ok(PopoutHost.LIVE_KEYS.has("context_module_window"), "context_module_window is a live pop-out key")
	if not tablet:
		_ok(_app.open_context_tab_window("links"), "the Links module opens in a window")
		await _settle(8)
		var w: Window = _app._module_windows.get("links")
		var pb := w.find_child(PopoutHost.BUTTON_NAME, true, false) as Button
		if PopoutHost.supported():
			_ok(pb != null and pb.text == "Pop out", "the window has a Pop out button")
			if pb != null and await _press(pb, "the module window's Pop out"):
				await _settle(10)
				_ok(not w.is_embedded() and w.visible, "pressed: the window is native (not embedded)")
				await _shot("module_popped")
				pb.pressed.emit()
				await _settle(10)
				_ok(w.is_embedded(), "pressed again: docked back (embedded)")
		else:
			_ok(pb == null, "unsupported platform: no Pop out button")
		w.hide()
	else:
		_ok(_app.open_context_tab_window("links") == false, "D. the tablet offers no module window")
	print("== B/C. the Note tab ==")
	_app.open_context_tab("note")
	await _settle(10)
	var body = _app.right_dock_ctrl
	var has_note: bool = _app.vault_window.is_docked()
	print("note docked: %s" % has_note)
	var nb: Node = _app.find_child("NotePopOut", true, false)
	if tablet:
		_ok(nb == null, "D. the tablet's Note tab has no Pop out")
	elif has_note:
		_ok(nb != null, "B. a docked note shows Pop out")
		## Unsaved typing in the docked editor must be saved before the note moves (`release_dock()` writes it).
		var te: TextEdit = _app.vault_window._editor.text_edit
		te.text = te.text + "
Typed before pop out."
		te.text_changed.emit()
		if nb != null and await _press(nb as Control, "the Note tab's Pop out"):
			await _settle(10)
			_ok(FileAccess.get_file_as_string(OS.get_environment("APPDATA").path_join("pcl_vault").path_join("PlaceNote.md")).contains("Typed before pop out."), "text typed in the tab just before Pop out is on disk (saved first, not lost)")
			_ok(_app.vault_window.visible, "the vault window shows the note")
			_ok(not _app.vault_window.is_docked(), "the tab's frame no longer holds the editor (one editor)")
			await _shot("note_popped")
	else:
		_ok(nb == null, "C. with no docked note there is no Pop out")
