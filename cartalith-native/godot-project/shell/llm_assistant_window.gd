extends AcceptDialog
class_name LlmAssistantWindow

## **The Assistant panel** (`LLM_ASSISTANT_SCOPE.md` milestone LLM-3: prose and Vault assist,
## U1 / U3 / U6, streamed and cancellable; Rulings DC, DD, DF, DI).
##
## Design: `design/llm-assistant-2026-10-25/index.html` (LLM-0, owner-approved): mode chips, a
## prompt box, Send / Stop, a token counter that says "no cap", a status line naming the
## endpoint, model-generated messages carrying the amber chip, and a disabled reason beside
## every disabled control. **Two labelled deviations from the mock**, both because the
## surrounding shell is not built for them yet: the mock draws a *docked* panel and a settings
## tab, whereas this ships as a floating window (the right dock is context-specific and owns
## its own tabs) with a "Settings..." button that opens `LlmSettingsWindow`; and the mock's
## "Proposed actions" list is absent because commands arrive with LLM-5/6.
##
## ## What this window does and never does
##
## * **Everything goes through the gate.** Send calls `LlmRun.begin`, which calls
##   `LlmStream.prepare`, which runs the LLM-2 `authorize()`; a refusal comes back as words and
##   no thread, socket or `HTTPClient` is created. This window decides nothing about
##   permission and never builds a request itself.
## * **Writes to the Vault only from the Save button's handler.** Nothing else in this file
##   calls `vault_write_file`: not the stream's `finished`, not a timer, not `open()`. The
##   write goes through `vault_write_file(rel, text, expect_hash)` with the hash read when the
##   note was PICKED, so an edit made in between is refused and nothing is written. The text
##   is the picked note's own text with a model-generated block appended
##   (`LlmStream.compose_note`): the user's words are never edited.
## * **Sends only ticked sources.** The current place's fields and events are sent only when
##   "Send: current entity" is on; a note's text only when "Send: named Vault notes" is on. A
##   source that is not allowed is absent from the request, not empty, and the "This will send"
##   line says exactly what will leave.
## * **Marks model text.** Every assistant message carries a "model-generated" chip and is
##   drawn in a muted ink so it cannot read as an app sentence.
## * **Holds no key.** The key is resolved inside `LlmStream.prepare` and never returned to
##   this window; the status line shows the endpoint, not a secret.
## * **No thread outlives the window.** Closing it cancels a running reply and joins the worker.
##
## One script, three densities (pointer window, tablet touch floors, phone full-height sheet),
## like `LlmSettingsWindow`. `controls` maps a stable name to each control for the probes.

## Emitted after a note was written (the path), so the Vault window can refresh.
signal note_saved(rel: String)

## The owning `DccApp`.
var app
## The engine bridge.
var bridge: EngineBridge
## Stable name -> control, refilled by `_rebuild()`. For probes.
var controls: Dictionary = {}
## The reply coordinator (a child node).
var run: LlmRun
## The most recent consent card, for probes.
var last_consent: Dictionary = {}

var _phone := false
var _body: VBoxContainer
var _task := "prose"
var _note_action := "note_summarise"
## The current place `{index, tid, name}`, or `{}`.
var _place: Dictionary = {}
## The picked note `{rel, text, hash}` as read when it was picked, or `{}`.
var _note: Dictionary = {}
var _reply := ""
var _reply_label: Label
var _msgs: VBoxContainer
var _scroll: ScrollContainer
var _status_dot: Label
var _status_text: Label
var _sent_label: Label
var _reason_label: Label
var _tok_label: Label
var _save_reason: Label
var _input: TextEdit
var _send: Button
var _stop: Button
var _save: Button
var _building := false
## Tokens as the servers reported them this session; only fields a server sent are counted, and
## `reported` says whether any server ever did (so "not reported" is never shown as 0).
var _tot_prompt := 0
var _tot_completion := 0
var _reported := false
var _chips: Dictionary = {}
## Files listed in the Vault picker, index + 1 -> relative path.
var _note_paths: PackedStringArray = PackedStringArray()

const _TASKS := [
	["prose", "Describe place"],
	["story", "Story"],
	["note", "Note"],
]
const _NOTE_ACTIONS := [
	["note_summarise", "Summarise"],
	["note_links", "Suggest links"],
	["note_cleanup", "Clean up"],
]
## Most notes listed in the picker. Labelled judgement: a dropdown of thousands is unusable;
## the Vault window is the place to browse a large vault.
const MAX_NOTES := 400
## The least width a wrapped label may be given (px). Labelled judgement: wide enough that the
## first-layout measurement is a short paragraph, not a tower; narrow enough for a 393 dp phone.
const WRAP_MIN_W := 160
## Reply cap sent as `max_tokens`. Labelled judgement: about a page of prose.
const REPLY_MAX_TOKENS := 1024


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "Assistant"
	size = Vector2i(560, 800)
	min_size = Vector2i(440, 520)
	get_ok_button().text = "Close"
	_phone = DccWidgets.phone_window(self, a)
	run = LlmRun.new()
	add_child(run)
	run.token.connect(_on_token)
	run.usage.connect(_on_usage)
	run.finished.connect(_on_finished)
	run.failed.connect(_on_failed)
	run.cancelled.connect(_on_cancelled)
	run.idle.connect(_refresh)
	var host := VBoxContainer.new()
	host.add_theme_constant_override("separation", 0)
	add_child(host)
	if _phone:
		DccWidgets.phone_head(host, "Assistant", "optional · local model")
	## A phone sheet draws its Close button over the bottom of the content (measured: it covered the
	## closing note), so the phone body ends 60 px early; the pointer dialog reserves the row itself.
	var pad := DccWidgets.pad(host, 14, 10, 14, 60 if _phone else 10)
	pad.size_flags_vertical = Control.SIZE_EXPAND_FILL
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 8)
	_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)
	## Closing cancels a running reply and joins the worker: no thread outlives the window.
	visibility_changed.connect(func():
		if not visible and run != null:
			run.shutdown())


func open() -> void:
	_rebuild()
	if _phone:
		app.phone_fit(self, 1.0)
	if not DccWidgets.phone_present(self, app):
		var vp := get_tree().root.get_visible_rect().size
		size = Vector2i(560, int(minf(800.0, vp.y - 60.0)))
		popup_centered()
		_refit.call_deferred()


## **Re-fit one frame after opening.** A dialog sizes its child from the content's minimum on the
## first layout, when wrapped labels have not yet been given a width, and a child rect set from that
## measurement (a tower several thousand px tall, measured 5 317) is not recomputed when the labels
## settle -- the input row and the Save row then sit below the window's edge. `LlmConsent.present`
## has the same fix and the same story. Skipped on a phone, whose sheet is full-screen.
func _refit() -> void:
	if _phone or not visible:
		return
	await get_tree().process_frame
	if not visible:
		return
	var want := size
	reset_size()
	size = want


# ---------------------------------------------------------------------------
# Small builders
# ---------------------------------------------------------------------------

func _field_h() -> int:
	if DccTheme.is_phone():
		return DccTheme.PHONE_TAP_MIN
	if DccTheme.is_tablet():
		return DccTheme.role_px("btn_min_h")
	return 26


func _clear(node: Control) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()


func _note_label(parent: Control, text: String, token: String = "text_ghost") -> Label:
	var l := DccTheme.label(text, token, DccTheme.FS_MICRO)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## A stated minimum width: without one a wrapped label is measured at width 0 on the first
	## layout and reports a tower of one-letter lines (see `_refit`).
	l.custom_minimum_size.x = WRAP_MIN_W
	parent.add_child(l)
	return l


func _button(parent: Control, text: String, on_press: Callable, primary: bool = false) -> Button:
	var b := DccWidgets.action(parent, text, on_press, primary)
	b.custom_minimum_size.y = maxi(int(b.custom_minimum_size.y), _field_h())
	return b


func _shrinkable(ob: OptionButton) -> void:
	ob.fit_to_longest_item = false
	ob.clip_text = true
	ob.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS


func _dot(token: String) -> void:
	_status_dot.add_theme_color_override("font_color", DccTheme.c(token))


# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

func _rebuild() -> void:
	_building = true
	_clear(_body)
	controls.clear()
	_chips.clear()
	if not bridge.llm_api:
		_note_label(_body, "This build has no assistant support. Update the Cartalith library to use it.", "block")
		_building = false
		return
	_place = _current_place()

	# -- status line + settings -------------------------------------------------
	var top := HBoxContainer.new()
	top.add_theme_constant_override("separation", 6)
	_body.add_child(top)
	_status_dot = DccTheme.label("●", "text_ghost", DccTheme.FS_SMALL)
	top.add_child(_status_dot)
	_status_text = DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	_status_text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_status_text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_status_text.custom_minimum_size.x = WRAP_MIN_W
	top.add_child(_status_text)
	controls["status"] = _status_text
	var settings_btn := _button(top, "Settings…", _open_settings)
	controls["settings"] = settings_btn

	# -- task chips ----------------------------------------------------------------
	var chips := HFlowContainer.new()
	chips.add_theme_constant_override("h_separation", 6)
	chips.add_theme_constant_override("v_separation", 6)
	_body.add_child(chips)
	for t in _TASKS:
		var id: String = t[0]
		var b := DccWidgets.segment(chips, t[1], func(): _pick_task(id))
		b.custom_minimum_size.y = maxi(int(b.custom_minimum_size.y), _field_h())
		_chips[id] = b
		controls["task_" + id] = b
	var cmd := DccWidgets.segment(chips, "Commands", func(): pass)
	cmd.disabled = true
	cmd.custom_minimum_size.y = maxi(int(cmd.custom_minimum_size.y), _field_h())
	cmd.tooltip_text = "Commands need grouped undo first (a later release). Nothing here changes the map."
	controls["task_commands"] = cmd
	_note_label(_body, "Commands and edits arrive in a later release; nothing here changes the map.")

	# -- subject and note -----------------------------------------------------------
	var place_l := DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	place_l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	place_l.custom_minimum_size.x = WRAP_MIN_W
	_body.add_child(place_l)
	controls["place"] = place_l

	## One row on every density (a phone's stacked label, picker and Reload cost 55 px the sheet
	## does not have: with them the body overflowed the sheet by 30 px, measured on 393x852).
	var note_row := HBoxContainer.new()
	note_row.add_theme_constant_override("separation", 6)
	_body.add_child(note_row)
	var nl := DccTheme.label("Note", "text_secondary", DccTheme.FS_SMALL)
	nl.custom_minimum_size.x = 40 if _phone else 56
	note_row.add_child(nl)
	var notes := OptionButton.new()
	notes.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	notes.custom_minimum_size.y = _field_h()
	_shrinkable(notes)
	note_row.add_child(notes)
	notes.item_selected.connect(_on_note_picked)
	controls["note"] = notes
	var reload := _button(note_row, "Reload", _reload_note)
	controls["note_reload"] = reload

	var act := OptionButton.new()
	act.custom_minimum_size.y = _field_h()
	_shrinkable(act)
	for a in _NOTE_ACTIONS:
		act.add_item(a[1])
	act.item_selected.connect(func(i: int): _note_action = _NOTE_ACTIONS[i][0]; _refresh())
	_body.add_child(act)
	controls["note_action"] = act

	_sent_label = _note_label(_body, "", "text_secondary")
	controls["sent"] = _sent_label

	# -- messages -----------------------------------------------------------------------
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.custom_minimum_size.y = 100 if _phone else 160
	_body.add_child(_scroll)
	_msgs = VBoxContainer.new()
	_msgs.add_theme_constant_override("separation", 6)
	_msgs.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.add_child(_msgs)
	controls["messages"] = _msgs
	_reply = ""
	_reply_label = null
	_add_msg("", "Choose what to do, type a request, and press Send. The reply is shown here; nothing is saved until you press Save to note.", "text_ghost")

	# -- input ------------------------------------------------------------------------------
	_input = TextEdit.new()
	_input.placeholder_text = "Ask for prose about the selected place, or for work on the chosen note…"
	_input.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	_input.custom_minimum_size.y = 64
	_input.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_input.text_changed.connect(_refresh)
	_body.add_child(_input)
	controls["input"] = _input

	_reason_label = _note_label(_body, "", "warn")
	controls["reason"] = _reason_label

	var ctl := HBoxContainer.new()
	ctl.add_theme_constant_override("separation", 8)
	_body.add_child(ctl)
	_tok_label = DccTheme.label("", "text_dim", DccTheme.FS_SMALL)
	_tok_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_tok_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_tok_label.custom_minimum_size.x = WRAP_MIN_W
	_tok_label.tooltip_text = "No spend limit is enforced. Counts are what the server reported."
	ctl.add_child(_tok_label)
	controls["tokens"] = _tok_label
	_stop = _button(ctl, "Stop", _on_stop)
	controls["stop"] = _stop
	_send = _button(ctl, "Send", _on_send, true)
	controls["send"] = _send

	var save_row := HBoxContainer.new()
	save_row.add_theme_constant_override("separation", 8)
	_body.add_child(save_row)
	_save = _button(save_row, "Save to note", _on_save)
	controls["save"] = _save
	_save_reason = _note_label(save_row, "", "text_ghost")
	controls["save_reason"] = _save_reason
	_note_label(_body, "Saved text is appended to the note, marked model-generated. Your own text is never changed.")

	_fill_notes()
	_building = false
	_refresh()


## The current settlement `{index, tid, name}`, or `{}` (no world, generating, none selected).
func _current_place() -> Dictionary:
	if bridge.generating or not bridge.has_world:
		return {}
	var dock = app.get("right_dock_ctrl")
	if dock == null or not dock.has_method("current_settlement_index"):
		return {}
	var idx := int(dock.current_settlement_index())
	var all: Array = bridge.settlements()
	if idx < 0 or idx >= all.size():
		return {}
	var s: Dictionary = all[idx]
	return {"index": idx, "tid": int(s.get("tid", idx)), "name": String(s.get("name", "(unnamed)"))}


## Lists the Vault's notes in the picker; the first row is "No note chosen".
func _fill_notes() -> void:
	var ob := controls["note"] as OptionButton
	ob.clear()
	ob.add_item("No note chosen")
	_note_paths = PackedStringArray()
	var info: Dictionary = bridge.vault_info()
	if not bool(info.get("bound", false)):
		ob.set_item_text(0, "No Vault connected")
		ob.disabled = true
		(controls["note_reload"] as Button).disabled = true
		return
	ob.disabled = false
	var files: PackedStringArray = bridge.vault_list_files(MAX_NOTES)
	for f in files:
		ob.add_item(f)
		_note_paths.append(f)
	if not _note.is_empty():
		var i := _note_paths.find(String(_note["rel"]))
		ob.select(i + 1 if i >= 0 else 0)


# ---------------------------------------------------------------------------
# Context
# ---------------------------------------------------------------------------

## The entity's recorded fields as `key: value` lines, from the Vault's own export values.
func _entity_fields() -> String:
	if _place.is_empty():
		return ""
	var vals: Dictionary = bridge.vault_entity_values("settlement", int(_place["tid"]))
	var lines: PackedStringArray = PackedStringArray()
	var keys: Array = vals.keys()
	keys.sort()
	for k in keys:
		var v = vals[k]
		if typeof(v) in [TYPE_STRING, TYPE_INT, TYPE_FLOAT, TYPE_BOOL]:
			var s := String(v).strip_edges()
			if s != "":
				lines.append("%s: %s" % [k, s])
	return "\n".join(lines)


## Timeline events for the place: its recorded ownership periods (the Timeline's own data)
## then the dated events its attached notes record. One per line; empty when there are none.
func _entity_events() -> String:
	if _place.is_empty():
		return ""
	var lines: PackedStringArray = PackedStringArray()
	var names := {}
	for f in bridge.get_factions():
		var fd: Dictionary = f
		names[int(fd.get("id", 0))] = String(fd.get("name", "Faction %d" % int(fd.get("id", 0))))
	for p in bridge.civ_settlement_ownership_periods(int(_place["tid"])):
		var pd: Dictionary = p
		var who: String = names.get(int(pd.get("faction_id", 0)), "a faction")
		var end_text := "present" if bool(pd.get("current", false)) else str(int(pd.get("end_year", 0)))
		lines.append("%d to %s: held by %s" % [int(pd.get("start_year", 0)), end_text, who])
	var ch: Dictionary = bridge.vault_entity_chronos("settlement", int(_place["tid"]))
	if bool(ch.get("ok", false)):
		for e in ch.get("events", []):
			var ed: Dictionary = e
			var line := "%d" % int(ed.get("start", 0))
			if ed.has("end"):
				line += " to %d" % int(ed["end"])
			line += ": " + String(ed.get("name", ""))
			if ed.has("description"):
				line += " - " + String(ed["description"])
			lines.append(line)
	return "\n".join(lines)


## The request arguments for the current task and the ticked sources. A source that is not
## allowed or not available is ABSENT (not empty), so it is neither sent nor mentioned.
func _args(request: String) -> Dictionary:
	var a := {"task": _task_id(), "request": request, "max_tokens": REPLY_MAX_TOKENS, "include_usage": true}
	if _task == "prose" or _task == "story":
		if bool(DccSettings.llm_value("ctx_entity")) and not _place.is_empty():
			a["entity_label"] = String(_place["name"])
			if _task == "prose":
				a["entity_fields"] = _entity_fields()
			else:
				a["events"] = _entity_events()
	elif _task == "note":
		if bool(DccSettings.llm_value("ctx_notes")) and not _note.is_empty():
			a["note_path"] = String(_note["rel"])
			a["note_text"] = String(_note["text"])
	return a


func _task_id() -> String:
	return _note_action if _task == "note" else _task


# ---------------------------------------------------------------------------
# State
# ---------------------------------------------------------------------------

## The gate's answer for this capability: `{ok, ...}` or `{ok: false, code, reason}`.
func _gate() -> Dictionary:
	return bridge.llm_authorize(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins())


## Recomputes every label and enabled state from stored settings and the current inputs.
func _refresh() -> void:
	if _building or _send == null:
		return
	var busy := run.busy()
	var g := _gate()
	var gate_ok := bool(g.get("ok", false))
	if gate_ok:
		_status_text.text = "%s · model %s · key is read from the %s" % [
			String(g["origin"]), String(DccSettings.llm_value("model")),
			"OS credential store" if String(DccSettings.llm_value("key_source")) == "store" else "environment variable"]
		_dot("good")
	else:
		_status_text.text = String(g.get("reason", "Unavailable."))
		_dot("block" if String(g.get("code", "")) != "feature_off" else "text_ghost")
	for id in _chips:
		DccWidgets.set_segment_on(_chips[id], id == _task)
		(_chips[id] as Button).disabled = busy
	(controls["note_action"] as OptionButton).visible = _task == "note"
	(controls["note"] as OptionButton).disabled = busy or not bool(bridge.vault_info().get("bound", false))
	(controls["place"] as Label).text = _place_text()
	(controls["note_reload"] as Button).disabled = busy or _note.is_empty()
	_stop.visible = busy
	_send.visible = true

	# What would be sent, and why Send is unavailable.
	var request := _input.text.strip_edges()
	var reason := ""
	var sent_text := ""
	if not gate_ok:
		reason = String(g.get("reason", "Unavailable."))
	elif request == "":
		reason = "Type a request first."
	else:
		var pv: Dictionary = run.stream.preview(_args(request)) if run.available() else {"ok": false, "reason": "This build has no assistant support."}
		if bool(pv.get("ok", false)):
			sent_text = _sent_text(pv["sent"])
		else:
			reason = String(pv.get("reason", "Unavailable."))
	if reason == "" and sent_text == "":
		sent_text = "This will send: nothing yet."
	_sent_label.text = sent_text
	_reason_label.text = reason
	_reason_label.visible = reason != ""
	_send.disabled = busy or (reason != "" and String(g.get("code", "")) != "consent_needed")
	_send.text = "Review consent…" if String(g.get("code", "")) == "consent_needed" else "Send"
	_send.tooltip_text = reason
	_input.editable = not busy
	_tok_label.visible = bool(DccSettings.llm_value("token_counter"))
	_tok_label.text = _token_text()

	# Save: only with a finished reply and a picked note.
	var why := ""
	if busy:
		why = "Wait for the reply to finish."
	elif _reply.strip_edges() == "":
		why = "No reply to save yet."
	elif _note.is_empty():
		why = "Pick a note to save into."
	_save.disabled = why != ""
	_save.tooltip_text = why
	_save_reason.text = why
	_save_reason.visible = why != ""


func _place_text() -> String:
	if _place.is_empty():
		if bridge.generating:
			return "Place: unavailable while a generation runs."
		if not bridge.has_world:
			return "Place: no world is open."
		return "Place: none selected. Click a settlement on the map."
	return "Place: %s" % String(_place["name"])


## The "This will send" line: the request, then each ticked source with its size.
func _sent_text(s: Dictionary) -> String:
	var parts: PackedStringArray = PackedStringArray(["your request (%d characters)" % int(s["request_chars"])])
	if bool(s["entity"]):
		parts.append("the place's recorded fields (%d characters)" % int(s["entity_chars"]))
	if bool(s["events"]):
		parts.append("the place's timeline events (%d characters)" % int(s["events_chars"]))
	if bool(s["note"]):
		parts.append("the note's text (%d characters)" % int(s["note_chars"]))
	var t := "This will send: " + ", ".join(parts) + "."
	if bool(s["truncated"]):
		t += " A source is long and will be cut."
	return t


func _token_text() -> String:
	if not _reported:
		return "Tokens: not reported by the server yet · no cap"
	return "Tokens %d in / %d out · no cap" % [_tot_prompt, _tot_completion]


# ---------------------------------------------------------------------------
# Actions
# ---------------------------------------------------------------------------

func _open_settings() -> void:
	app.open_assistant_settings()


func _pick_task(id: String) -> void:
	if run.busy():
		return
	_task = id
	_refresh()


func _on_note_picked(index: int) -> void:
	if _building:
		return
	if index <= 0 or index - 1 >= _note_paths.size():
		_note = {}
	else:
		_read_note(_note_paths[index - 1])
	_refresh()


## Reads the note and remembers the hash it was read at: the guard for the later write.
func _read_note(rel: String) -> bool:
	var r: Dictionary = bridge.vault_read_file_for_edit(rel)
	if not bool(r.get("ok", false)):
		_note = {}
		_add_msg("", "Could not read that note: %s" % String(r.get("error", "unknown error")), "block")
		return false
	_note = {"rel": rel, "text": String(r["text"]), "hash": String(r["hash"])}
	return true


func _reload_note() -> void:
	if _note.is_empty():
		return
	if _read_note(String(_note["rel"])):
		_add_msg("", "Reloaded %s. Saving now guards against edits made after this moment." % String(_note["rel"]), "text_ghost")
	_refresh()


## Send: the gate, then the stream. A `consent_needed` gate turns the button into the consent card.
func _on_send() -> void:
	var g := _gate()
	if String(g.get("code", "")) == "consent_needed":
		var ep: Dictionary = bridge.llm_endpoint_info(String(DccSettings.llm_value("preset_id")), String(DccSettings.llm_value("base_url")))
		var preset := {}
		for p in bridge.llm_presets():
			if String(p["id"]) == String(DccSettings.llm_value("preset_id")):
				preset = p
		last_consent = LlmConsent.present(app, ep, preset, func(): _refresh(), func(): _refresh())
		return
	var request := _input.text.strip_edges()
	_add_msg("You", request, "text")
	_reply = ""
	_reply_label = _add_msg("Assistant", "", "text_secondary", true)
	var r := run.begin(DccSettings.llm_gate_dict(), "prose", DccSettings.llm_consented_origins(), _args(request))
	if not bool(r.get("ok", false)):
		_reply_label.get_parent().get_parent().queue_free()
		_reply_label = null
		_add_msg("", String(r.get("reason", "The request could not be started.")), "block")
	else:
		_input.text = ""
	_refresh()


func _on_stop() -> void:
	run.cancel()
	_stop.disabled = true
	_stop.tooltip_text = "Stopping…"


func _on_token(t: String) -> void:
	_reply += t
	if _reply_label != null:
		_reply_label.text = _reply
		_scroll_to_end()
	_refresh()


func _on_usage(info: Dictionary) -> void:
	if info.has("prompt"):
		_tot_prompt += int(info["prompt"])
		_reported = true
	if info.has("completion"):
		_tot_completion += int(info["completion"])
		_reported = true
	_refresh()


func _on_finished(info: Dictionary) -> void:
	if info.get("finish_reason", "") == "length":
		_add_msg("", "The reply was cut at the length limit.", "warn")
	_refresh()


func _on_failed(message: String) -> void:
	_add_msg("", message, "block")
	_refresh()


func _on_cancelled() -> void:
	_add_msg("", "Stopped.", "text_ghost")
	_stop.disabled = false
	_refresh()


## Appends a message card; returns its body label. An assistant card carries the amber
## "model-generated" chip.
func _add_msg(who: String, text: String, token: String, model_marked: bool = false) -> Label:
	var box := PanelContainer.new()
	box.add_theme_stylebox_override("panel", DccWidgets.box("border", "sunken", 8, 6))
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 2)
	box.add_child(col)
	if who != "":
		var head := HBoxContainer.new()
		head.add_theme_constant_override("separation", 6)
		head.add_child(DccTheme.label(who, "text_dim", DccTheme.FS_MICRO))
		if model_marked:
			var chip := DccTheme.label("MODEL-GENERATED", "warn", DccTheme.FS_MICRO)
			chip.tooltip_text = "Written by a language model. Check it before you rely on it."
			head.add_child(chip)
		col.add_child(head)
	var l := DccTheme.label(text, token, DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.custom_minimum_size.x = WRAP_MIN_W
	col.add_child(l)
	_msgs.add_child(box)
	_scroll_to_end()
	return l


func _scroll_to_end() -> void:
	if _scroll == null:
		return
	await get_tree().process_frame
	if is_instance_valid(_scroll):
		_scroll.scroll_vertical = int(_scroll.get_v_scroll_bar().max_value)


## **The only caller of `vault_write_file` in this file.** Writes the picked note's text as it
## was read, plus the model-generated block, guarded by the hash from the same read. A note
## edited since is refused by the Vault and nothing is written.
func _on_save() -> void:
	if run.busy() or _reply.strip_edges() == "" or _note.is_empty():
		return
	var date := Time.get_date_string_from_system()
	var c: Dictionary = run.stream.compose_note(String(_note["text"]), run.model, date, _reply)
	if not bool(c.get("ok", false)):
		_add_msg("", String(c.get("reason", "Nothing to save.")), "block")
		return
	var w: Dictionary = bridge.vault_write_file(String(_note["rel"]), String(c["text"]), String(_note["hash"]))
	if bool(w.get("ok", false)):
		_note["text"] = String(c["text"])
		_note["hash"] = String(w["hash"])
		_add_msg("", "Saved to %s, marked model-generated." % String(_note["rel"]), "good")
		note_saved.emit(String(_note["rel"]))
	else:
		var err := String(w.get("error", ""))
		if err.contains("changed since"):
			_add_msg("", "That note changed after you picked it, so nothing was written. Press Reload, check the note, then Save again.", "warn")
		else:
			_add_msg("", "The note was not written: %s" % err, "block")
	_refresh()
