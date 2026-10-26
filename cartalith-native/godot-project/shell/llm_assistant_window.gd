extends AcceptDialog
class_name LlmAssistantWindow

## **The Assistant panel** (`LLM_ASSISTANT_SCOPE.md` milestone LLM-3: prose and Vault assist,
## U1 / U3 / U6, streamed and cancellable; LLM-4: hosted OpenAI-compatible and Anthropic
## endpoints, the running token counter and its opt-in cap, the one automatic retry;
## Rulings DC, DD, DF, DI).
##
## Design: `design/llm-assistant-2026-10-25/index.html` (LLM-0, owner-approved): mode chips, a
## prompt box, Send / Stop, a token counter that says "no cap", a status line naming the
## endpoint, model-generated messages carrying the amber chip, and a disabled reason beside
## every disabled control. **Two labelled deviations from the mock**, both because the
## surrounding shell is not built for them yet: the mock draws a *docked* panel and a settings
## tab, whereas this ships as a floating window (the right dock is context-specific and owns
## its own tabs) with a "Settings..." button that opens `LlmSettingsWindow`; and the mock's
## "Proposed actions" list is the proposal CARD of LLM-6b (below), drawn in the message list.
##
## ## Commands: generation-setting proposals (LLM-6a/6b, scope s.5b)
##
## * **The model proposes; it never acts.** The Commands chip sends the request through
##   `LlmRun.begin(..., "commands", ...)`. Rust runs the two-step flow (schema, then proposal)
##   and validates every number (V1-V13, reject and never clamp: Ruling DI P1). What arrives
##   here is the validated card data; NO handler writes a setting except `_apply_card`, which
##   only a click on the card's Apply button reaches.
## * **One Apply is one undo step, and never regenerates** (s.5b.4). `_apply_card` opens an
##   `undo_group.gd` group, writes each change through `write_param` (which uses
##   `engine_bridge.gd::param_write`, the setter every dialog uses), and commits; a write the
##   engine refuses or clamps rolls the whole group back. No generate, refresh or recompute is
##   called; the card says which follow-up each change needs.
## * **Model text is drawn as data.** The card carries the MODEL PROPOSAL chip; the summary and
##   every reason are plain `Label`s in a muted, quoted style (a `Label` renders no markup and
##   makes no link). A refused change is listed under "Not applied" with its rule and reason;
##   the scope's struck-through text is a muted list here, because Godot's `Label` has no
##   strikethrough and a `RichTextLabel` would parse model text as markup (labelled deviation).
## * **Unavailability shows its reason** (V13): while a world generates or is finalized, Apply
##   is disabled with the reason beside it; a card with nothing valid has no Apply at all.
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
## * **Sends only ticked sources, except the subject of a place entry.** A Story request sends the
##   place's events only when "Send: current entity" is on; a named note's text, and the Vault notes
##   that bear on a place, only when "Send: named Vault notes" is on. A source that is not allowed
##   is absent from the request, not empty. **A "Describe place" entry always carries the place's
##   generated data** (the place IS its subject: `LlmPlaceFacts`, LLM-11, owner 2026-10-26), listed
##   exactly in the "This will send" line, and hosted providers still need their per-origin consent.
## * **Marks model text.** Every assistant message carries a "model-generated" chip and is
##   drawn in a muted ink so it cannot read as an app sentence.
## * **The token counter is Rust's.** The totals come from `LlmStream.token_counts()` (one
##   process-wide counter fed by every reply's `usage`), never from a GDScript tally, and a
##   total no server has reported is shown as "not reported", never `0`. The cap, if the user
##   set one, is checked in Rust before every request; here it is only displayed.
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
var _tok_reset: Button
var _save_reason: Label
var _input: TextEdit
var _send: Button
var _stop: Button
var _save: Button
var _building := false
var _chips: Dictionary = {}
## The gathered place facts (LLM-11): `{key, tid, name, bundle: <LlmStream.place_bundle result>,
## gather_ms, vault: <LlmPlaceFacts.gather_vault result or {}>}`, or `{}`. Rebuilt when the place, the
## picked note or the Vault setting changes, and always at Send so the model sees the world as it is.
var _facts: Dictionary = {}
## The facts the CURRENT reply was written from (the bundle's grounding corpus lives here), and the
## last grounding check, for probes.
var _sent_facts: Dictionary = {}
var last_grounding: Dictionary = {}
## The reply the grounding check last ran on (so a repeat finish does not add a second card).
var _grounded_reply := ""
## Files listed in the Vault picker, index + 1 -> relative path.
var _note_paths: PackedStringArray = PackedStringArray()

## LLM-6d: the shell half of the view and layer actions (`llm_view.gd`: the facts the validator is
## built from, and the one place a validated view change is applied). No `class_name` on purpose.
const _VIEW := preload("res://shell/llm_view.gd")
## "Choose groups" (scope s.5b.1 fallback, LLM-6d): the setting groups the USER ticked, at most
## `MAX_CHOSEN_GROUPS`. When non-empty, Send hands them to `LlmStream.prepare` as `args.groups` and
## the app performs the model's step one itself. Cleared after each Send.
var _chosen_groups: Array = []
## True when the LAST Send carried chosen groups (the ticks are cleared at Send), so an answer without
## a proposal does not tell a user who DID choose groups to choose them.
var _groups_were_chosen := false
var _group_chips: Dictionary = {}
var _groups_flow: HFlowContainer
## Most groups one request may open: the catalogue's `MAX_SCHEMA_GROUPS`, restated because the
## chip row must stop a fourth tick before Rust would refuse it (a labelled copy; Rust re-checks).
const MAX_CHOSEN_GROUPS := 3

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
## Reply cap for a grounded place entry (LLM-11). Labelled judgement: a 600-word entry with topics
## is about 900 tokens, and 1 024 cut two of eight stage-1 replies mid-sentence.
const PLACE_REPLY_MAX_TOKENS := 2048
## The proposal cards shown, oldest first, for probes: each `{info, box, apply, dismiss,
## reason, state}` where `state` is "new", "applied", "dismissed" or "failed".
var cards: Array = []
## The prompt box's hint, per kind of task.
const PLACEHOLDER_PROSE := "Ask for prose about the selected place, or for work on the chosen note…"
const PLACEHOLDER_COMMANDS := "Ask for a settings change (a drier climate), a view change (hide the roads, centre on a cell, switch the style) or a question about the settings or the world…"


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "Assistant"
	size = Vector2i(560, 800)
	min_size = Vector2i(440, 520)
	get_ok_button().text = "Close"
	_phone = DccWidgets.phone_window(self, a)
	## PW-3 (`POPOUT_WINDOWS_SCOPE.md`, Ruling DK Q2): the "Pop out / Dock" button in the button row. Desktop
	## form only. A move re-shows THIS node and never calls `open()`, so the prompt, the reply, the cards
	## and a reply still streaming are the ones the user left (the close handler below ignores the hide a
	## move makes, `PopoutHost.is_moving()`).
	PopoutHost.add_toggle_button(self)
	run = LlmRun.new()
	add_child(run)
	run.token.connect(_on_token)
	run.usage.connect(_on_usage)
	run.finished.connect(_on_finished)
	run.failed.connect(_on_failed)
	run.cancelled.connect(_on_cancelled)
	run.retrying.connect(_on_retrying)
	run.idle.connect(_refresh)
	run.params_step.connect(_on_params_step)
	run.proposal.connect(_on_proposal)
	run.answered.connect(_on_answered)
	run.view_proposal.connect(_on_view_proposal)
	run.sources_shown.connect(_on_sources)
	## The view and layer facts are read from the live app every turn (LLM-6d).
	run.view_source = func() -> String: return _VIEW.facts_json(app, bridge)
	## The next turn of a setting request reads the settings as they are THEN.
	run.params_source = func() -> Dictionary: return bridge.param_snapshot()
	## The model's read-only lookups (LLM-8) are answered from the engine's own getters, read by
	## `LlmFacts` and only when no generation is running.
	run.facts_source = func(needs: Dictionary) -> Dictionary: return LlmFacts.read(bridge, needs)
	## The model's knowledge lookups (RK-5) are executed against the in-memory index by `LlmKnowledge`,
	## under a plan Rust re-decides for EVERY turn (the checkboxes and the hosted opt-in may have changed).
	run.knowledge_source = func(needs: Dictionary) -> Dictionary: return LlmKnowledge.run(bridge, LlmKnowledge.plan(run.stream, "commands"), needs)
	var host := VBoxContainer.new()
	host.add_theme_constant_override("separation", 0)
	add_child(host)
	if _phone:
		DccWidgets.phone_head(host, "Assistant", "optional · your model")
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
	## **A move is not a close** (PW-3): `PopoutHost.toggle()` hides the window for the length of one
	## call to put it in, or take it out of, its own operating-system window, and that must not end a
	## reply that is streaming -- the whole point of a second monitor is to watch it run. `is_moving()` is
	## true only inside that call. Any other hide (Close, Escape, the title bar's X, a native window's X)
	## still cancels and joins exactly as before (the LLM-3 cancel bar, re-run with the window native).
	visibility_changed.connect(func():
		if not visible and run != null and not PopoutHost.is_moving(self):
			run.shutdown())


func open() -> void:
	_rebuild()
	if _phone:
		app.phone_fit(self, 1.0)
	if not DccWidgets.phone_present(self, app):
		var vp := get_tree().root.get_visible_rect().size
		size = Vector2i(560, int(minf(800.0, vp.y - 60.0)))
		## PW-3: re-decides native versus docked for this open.
		PopoutHost.prepare(self)
		popup_centered()
		_refit.call_deferred()
	_relayout_phone.call_deferred()


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
	## **Drop the cards with the widgets that held them.** `_clear(_body)` frees every card box
	## and its buttons; the array that indexes them must go in the same breath, or the next
	## `_refresh_cards` reads `card["apply"]` off a freed instance ("previously freed instance",
	## 132 to 437 times per arm log in the live comparison of 2026-10-26, REPORT.md Defects 2).
	## Decision: a reopened panel starts a fresh conversation (the transcript is rebuilt empty
	## too), so the cards are CLEARED, not re-created; the note below says so for any card that
	## was still waiting, because a silently vanished proposal would look like a lost change.
	## Nothing is applied or undone by this.
	var dropped := 0
	for card in cards:
		if card["state"] == "new" and card["apply"] != null:
			dropped += 1
	cards.clear()
	source_cards.clear()
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
	var cmd := DccWidgets.segment(chips, "Commands", func(): _pick_task("commands"))
	cmd.custom_minimum_size.y = maxi(int(cmd.custom_minimum_size.y), _field_h())
	cmd.tooltip_text = "Ask for generation-setting changes, or a question about the settings or the world. The model proposes; nothing changes until you press Apply on its card."
	_chips["commands"] = cmd
	controls["task_commands"] = cmd
	var cmd_note := _note_label(_body, "Commands proposes generation-setting changes and view changes (layers, style preset, landmark types, centring and zooming, opening a window) for you to review, and answers questions from read-only lookups (and from your notes and world data when ticked in Settings); nothing changes until you press Apply. It never generates, saves, closes, exports or deletes. Edit actions arrive in a later release.")
	controls["commands_note"] = cmd_note
	## "Choose groups" (scope s.5b.1): the fallback for a small model that cannot pick setting groups.
	var grow := HFlowContainer.new()
	grow.add_theme_constant_override("h_separation", 6)
	grow.add_theme_constant_override("v_separation", 6)
	_body.add_child(grow)
	controls["groups_row"] = grow
	var gtoggle := DccWidgets.segment(grow, "Choose groups", func(): _toggle_groups())
	gtoggle.custom_minimum_size.y = maxi(int(gtoggle.custom_minimum_size.y), _field_h())
	gtoggle.tooltip_text = "Optional. Tick up to %d setting groups and Send: the app opens them for the model itself, so a small model that cannot pick groups can still propose changes." % MAX_CHOSEN_GROUPS
	controls["groups_toggle"] = gtoggle
	_groups_flow = HFlowContainer.new()
	_groups_flow.add_theme_constant_override("h_separation", 6)
	_groups_flow.add_theme_constant_override("v_separation", 6)
	_groups_flow.visible = false
	_body.add_child(_groups_flow)
	controls["groups_flow"] = _groups_flow
	_group_chips.clear()
	for gname in _group_names():
		var gn: String = gname
		var gb := DccWidgets.segment(_groups_flow, gn, func(): _tick_group(gn))
		gb.custom_minimum_size.y = maxi(int(gb.custom_minimum_size.y), _field_h())
		_group_chips[gn] = gb
		controls["group_" + gn] = gb

	# -- subject and note -----------------------------------------------------------
	var place_l := DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	place_l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	place_l.custom_minimum_size.x = WRAP_MIN_W
	_body.add_child(place_l)
	controls["place"] = place_l
	## Grounded prose (LLM-11): the facts sent for the chosen place are viewable before and after
	## sending. Local only: this button shows the text, it sends nothing.
	var facts_row := HBoxContainer.new()
	facts_row.add_theme_constant_override("separation", 6)
	_body.add_child(facts_row)
	controls["facts_row"] = facts_row
	var facts_l := DccTheme.label("", "text_dim", DccTheme.FS_SMALL)
	facts_l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	facts_l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	facts_l.custom_minimum_size.x = WRAP_MIN_W
	facts_row.add_child(facts_l)
	controls["facts_status"] = facts_l
	var facts_btn := _button(facts_row, "View facts", _on_view_facts)
	facts_btn.tooltip_text = "Show the generated data and Vault notes that will be sent with a place entry. Nothing is sent by this."
	controls["facts_view"] = facts_btn
	var facts_re := _button(facts_row, "Refresh", _on_refresh_facts)
	facts_re.tooltip_text = "Read the place's data and notes again (it is also read fresh at every Send)."
	controls["facts_refresh"] = facts_re

	## One row on every density (a phone's stacked label, picker and Reload cost 55 px the sheet
	## does not have: with them the body overflowed the sheet by 30 px, measured on 393x852).
	var note_row := HBoxContainer.new()
	note_row.add_theme_constant_override("separation", 6)
	_body.add_child(note_row)
	controls["note_row"] = note_row
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

	## The "this will send" line itemises every section, count, cut and Vault note title (LLM-11), which
	## is long: it scrolls inside a bounded box so it never pushes the Send and Save rows off a phone
	## sheet (measured on 393x852: the unbounded line was 190 px and hid the Save row). Horizontal scroll is
	## disabled, so the label states a minimum width (a disabled axis folds its child's minimum into the
	## container's: `_note_label` sets one).
	var sent_scroll := ScrollContainer.new()
	sent_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	sent_scroll.custom_minimum_size.y = 64 if _phone else 80
	sent_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.add_child(sent_scroll)
	_sent_label = _note_label(sent_scroll, "", "text_secondary")
	controls["sent"] = _sent_label
	controls["sent_scroll"] = sent_scroll

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
	if dropped > 0:
		var gone := _note_label(_msgs, "%d proposal%s from before the panel was reopened %s cleared. Nothing was applied; ask again to get a new one." % [dropped, "" if dropped == 1 else "s", "was" if dropped == 1 else "were"], "text_dim")
		controls["cleared_note"] = gone
	_reply = ""
	_reply_label = null
	_add_msg("", "Choose what to do, type a request, and press Send. The reply is shown here; nothing is saved until you press Save to note.", "text_ghost")

	# -- input ------------------------------------------------------------------------------
	_input = TextEdit.new()
	_input.placeholder_text = PLACEHOLDER_PROSE
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
	_tok_label.tooltip_text = "No spend limit is enforced unless you set a cap in Settings. Counts are what servers reported."
	ctl.add_child(_tok_label)
	controls["tokens"] = _tok_label
	_tok_reset = _button(ctl, "Reset", _on_reset_counter)
	_tok_reset.tooltip_text = "Zero the counter for this session. It does not change any cap setting."
	controls["tokens_reset"] = _tok_reset
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
	controls["save_note"] = _note_label(_body, "Saved text is appended to the note, marked model-generated. Your own text is never changed.")

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
	if _task == "prose" and not _place.is_empty():
		## Grounded prose (LLM-11): the place's generated data (and, when allowed, its Vault notes)
		## is the SUBJECT of the entry, so it is attached whatever the "current entity" switch says;
		## that switch still governs the Story task. Hosted providers still need their consent (the
		## gate), and the key rule is unchanged.
		a["entity_label"] = String(_place["name"])
		a["max_tokens"] = PLACE_REPLY_MAX_TOKENS
		if not _facts.is_empty() and bool(_facts.get("bundle", {}).get("ok", false)):
			a["place_bundle"] = String(_facts["bundle"]["bundle"])
	elif _task == "story":
		if bool(DccSettings.llm_value("ctx_entity")) and not _place.is_empty():
			a["entity_label"] = String(_place["name"])
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

## The capability the current task needs: `commands` for setting proposals, else `prose`.
func _capability() -> String:
	return "commands" if _task == "commands" else "prose"


## The gate's answer for this capability: `{ok, ...}` or `{ok: false, code, reason}`.
func _gate() -> Dictionary:
	return bridge.llm_authorize(DccSettings.llm_gate_dict(), _capability(), DccSettings.llm_consented_origins())


## The arguments of a setting request: the user's words and the CURRENT settings (the engine's
## own getter; the snapshot while a generate runs). Rust builds the catalogue from these.
func _params_args(request: String) -> Dictionary:
	return {"request": request, "current": bridge.param_snapshot(), "max_tokens": REPLY_MAX_TOKENS, "include_usage": true}


## Recomputes every label and enabled state from stored settings and the current inputs.
func _refresh() -> void:
	if _building or _send == null:
		return
	var busy := run.busy()
	var g := _gate()
	var gate_ok := bool(g.get("ok", false))
	if gate_ok:
		## A hosted endpoint is named as such, because it is the one place text leaves the
		## machine for a third party; a local one is not alarming.
		_status_text.text = "%s (%s) · model %s · key is read from the %s" % [
			String(g["origin"]), "hosted, over the internet" if not bool(g.get("local", true)) else "this computer or network",
			String(DccSettings.llm_value("model")),
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
	## A setting proposal uses neither a place nor a note.
	(controls["place"] as Label).visible = _task != "commands"
	## The Commands explainer is about Commands only; hiding it elsewhere gives the place entry's longer
	## "this will send" line room on a tablet or phone sheet (measured: with it, the Save row was clipped).
	(controls["commands_note"] as Control).visible = _task == "commands"
	(controls["note_row"] as Control).visible = _task != "commands"
	## "Choose groups" opens SETTING groups, which a questions-only request (the parameter source unticked, a
	## knowledge box ticked) does not have; hiding it also gives the phone sheet its row back.
	var groups_ok: bool = _task == "commands" and bool(DccSettings.llm_value("ctx_params"))
	(controls["groups_row"] as Control).visible = groups_ok
	_groups_flow.visible = groups_ok and _groups_flow.visible
	for gn in _group_chips:
		var gbtn: Button = _group_chips[gn]
		var on: bool = _chosen_groups.has(gn)
		DccWidgets.set_segment_on(gbtn, on)
		gbtn.disabled = busy or (not on and _chosen_groups.size() >= MAX_CHOSEN_GROUPS)
		gbtn.tooltip_text = "At most %d groups; untick one first." % MAX_CHOSEN_GROUPS if gbtn.disabled and not busy else "Open the %s settings for the model." % gn
	(controls["groups_toggle"] as Button).disabled = busy
	DccWidgets.set_segment_on(controls["groups_toggle"], _groups_flow.visible or not _chosen_groups.is_empty())
	_input.placeholder_text = PLACEHOLDER_COMMANDS if _task == "commands" else PLACEHOLDER_PROSE
	(controls["note_reload"] as Button).disabled = busy or _note.is_empty()
	_stop.visible = busy
	_send.visible = true

	## Grounded prose: make sure the facts for the chosen place are gathered (cached by place,
	## picked note and Vault setting; Send always regathers).
	if _task == "prose" and not _place.is_empty() and not busy:
		_ensure_facts(false)
	(controls["facts_row"] as Control).visible = _task == "prose"
	(controls["facts_status"] as Label).text = _facts_status_text()
	(controls["facts_status"] as Label).tooltip_text = _facts_status_detail()
	(controls["facts_view"] as Button).disabled = busy or _facts.is_empty()
	(controls["facts_refresh"] as Button).disabled = busy or _place.is_empty()

	# What would be sent, and why Send is unavailable.
	var request := _input.text.strip_edges()
	var reason := ""
	var sent_text := ""
	var cap_check: Dictionary = run.stream.cap_check(int(DccSettings.llm_value("token_cap"))) if run.available() else {"ok": true}
	if not gate_ok:
		reason = String(g.get("reason", "Unavailable."))
	elif not bool(cap_check.get("ok", true)):
		reason = String(cap_check.get("reason", "The token cap is reached."))
	elif _task == "commands" and not bool(DccSettings.llm_value("ctx_params")) and not _knowledge_ticked():
		reason = "Tick \"Send: generation parameters and summary statistics\" in Settings first (a proposal or a question shows the model the current settings and figures), or tick \"Send: Vault notes found by search\" or \"Send: world data found by search\" to ask about your notes and world."
	elif request == "":
		reason = "Type a request first."
	elif _task == "commands":
		var pp: Dictionary = run.stream.params_preview(_params_args(request)) if run.available() else {"ok": false, "reason": "This build has no assistant support."}
		if bool(pp.get("ok", false)) and not bool(DccSettings.llm_value("ctx_params")):
			## Questions about the notes and the world only: no setting, figure or view is sent.
			sent_text = "This will send: your request (%d characters)." % int(pp["request_chars"])
		elif bool(pp.get("ok", false)):
			sent_text = "This will send: your request (%d characters); then, for the setting groups the model asks for (at most 3 of %d), each setting's name, range, description and current value. %s If you ask a question, it can also read any setting's value, whole-world figures (sea level, land fraction, elevation percentiles, mean rainfall and temperature, productivity) and up to 8 single map cells. Reading changes nothing." % [int(pp["request_chars"]), int(pp["groups"]), _view_sent_text()]
			if not _chosen_groups.is_empty():
				sent_text += " You chose the %s group%s, so those settings are sent with the first request." % [", ".join(PackedStringArray(_chosen_groups)), "" if _chosen_groups.size() == 1 else "s"]
		else:
			reason = String(pp.get("reason", "Unavailable."))
	else:
		var pv: Dictionary = run.stream.preview(_args(request)) if run.available() else {"ok": false, "reason": "This build has no assistant support."}
		if bool(pv.get("ok", false)):
			sent_text = _sent_text(pv["sent"])
		else:
			reason = String(pv.get("reason", "Unavailable."))
	if reason == "" and sent_text != "":
		sent_text += _knowledge_sent_text()
	if reason == "" and sent_text == "":
		sent_text = "This will send: nothing yet."
	_sent_label.text = sent_text
	## An empty line takes no room: the box only exists while there is something to itemise.
	(controls["sent_scroll"] as Control).visible = sent_text != ""
	_reason_label.text = reason
	_reason_label.visible = reason != ""
	_send.disabled = busy or (reason != "" and String(g.get("code", "")) != "consent_needed")
	_send.text = "Review consent…" if String(g.get("code", "")) == "consent_needed" else "Send"
	_send.tooltip_text = reason
	_input.editable = not busy
	_tok_label.visible = bool(DccSettings.llm_value("token_counter"))
	_tok_reset.visible = _tok_label.visible
	_tok_reset.disabled = busy
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
	(_save.get_parent() as Control).visible = _task != "commands"
	(controls["save_note"] as Control).visible = _task != "commands"
	_refresh_cards()


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
	if bool(s.get("place", false)):
		parts.append(_place_sent_text(s))
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


## What the grounded place facts amount to, in the user's words: the sections with their line
## counts, how many neighbours, what is stated as not available, and EXACTLY which Vault notes
## (by title) go too. Every figure is Rust's (`LlmStream.preview`'s `sent`), none is recomputed here.
func _place_sent_text(s: Dictionary) -> String:
	var secs: PackedStringArray = PackedStringArray()
	for sec in s.get("place_sections", []):
		secs.append("%s (%d)" % [String(sec[0]), int(sec[1])])
	var t := "the generated world data for %s (%d characters: %s; %d of %d settlements within 25 km listed" % [
		String(_place.get("name", "this place")), int(s.get("place_chars", 0)), ", ".join(secs),
		int(s.get("place_neighbours_listed", 0)), int(s.get("place_neighbours_within", 0))]
	if int(s.get("place_not_available", 0)) > 0:
		t += "; %d items stated as not available" % int(s["place_not_available"])
	t += ")"
	var titles: PackedStringArray = PackedStringArray(s.get("vault_titles", PackedStringArray()))
	if not titles.is_empty():
		var q := PackedStringArray()
		for ti in titles:
			q.append("\"%s\"" % ti)
		t += ", and %d Vault note%s (%d characters): %s" % [titles.size(), "" if titles.size() == 1 else "s", int(s.get("vault_chars", 0)), ", ".join(q)]
	elif not bool(s.get("vault_allowed", false)):
		t += ", and no Vault notes (tick \"Send: named Vault notes\" in Settings to add the notes that bear on this place)"
	else:
		t += ", and no Vault notes (none relevant were found)"
	if int(s.get("vault_withheld", 0)) > 0:
		t += "; %d note%s withheld by a private mark" % [int(s["vault_withheld"]), "" if int(s["vault_withheld"]) == 1 else "s"]
	var cuts: Array = s.get("place_truncated", [])
	if not cuts.is_empty():
		t += ". Cut or left out: %s" % "; ".join(PackedStringArray(cuts))
	return t


## The counter line. Totals are Rust's (`LlmStream.token_counts`); an absent total reads "not
## reported", never 0, and when some replies reported nothing the line says how many did, so a
## total is never presented as complete when it is not.
func _token_text() -> String:
	var c: Dictionary = run.stream.token_counts() if run.available() else {}
	var cap := int(DccSettings.llm_value("token_cap"))
	var cap_text := "no cap" if cap <= 0 else "cap %d" % cap
	if not c.has("prompt") and not c.has("completion"):
		return "Tokens: not reported yet · %s" % cap_text
	var t := "Tokens %s in / %s out" % [
		str(int(c["prompt"])) if c.has("prompt") else "not reported",
		str(int(c["completion"])) if c.has("completion") else "not reported"]
	if int(c.get("calls_reported", 0)) < int(c.get("calls", 0)):
		t += " · %d of %d replies reported usage" % [int(c.get("calls_reported", 0)), int(c.get("calls", 0))]
	return t + " · " + cap_text


# ---------------------------------------------------------------------------
# Grounded place facts (LLM-11)
# ---------------------------------------------------------------------------

## Gathers the facts for the current place now (engine getters and, when allowed, Vault notes), and
## stores `_facts`. `force` skips the cache. Read-only; opens no socket.
func _ensure_facts(force: bool) -> bool:
	if _place.is_empty() or not run.available():
		_facts = {}
		return false
	var allow_notes := bool(DccSettings.llm_value("ctx_notes"))
	var key := "%d|%s|%s|%s" % [int(_place["tid"]), allow_notes, String(_note.get("rel", "")), String(_note.get("hash", ""))]
	if not force and not _facts.is_empty() and String(_facts.get("key", "")) == key:
		return true
	var g: Dictionary = LlmPlaceFacts.gather(bridge, run.stream, int(_place["index"]))
	if not bool(g.get("ok", false)):
		## An empty key, so a failure is retried (a generation may have just finished), never cached.
		_facts = {"key": "", "tid": int(_place["tid"]), "bundle": {"ok": false, "reason": String(g.get("reason", ""))}}
		return false
	var vault := {}
	var read: Array = []
	var own_text := ""
	if allow_notes:
		vault = LlmPlaceFacts.gather_vault(bridge, run.stream, g, _note)
		if bool(vault.get("ok", false)):
			read = vault["read"]
			own_text = String(vault.get("own_note_text", ""))
	elif not _note.is_empty():
		## Not sent, but the place's note is still the grounding corpus: a name the user wrote counts.
		own_text = String(_note.get("text", ""))
	var b: Dictionary = run.stream.place_bundle(JSON.stringify(g["raw"]), JSON.stringify(read), own_text, allow_notes)
	_facts = {"key": key, "tid": int(_place["tid"]), "name": String(_place["name"]), "bundle": b, "gather_ms": int(g.get("ms", 0)), "vault": vault}
	return bool(b.get("ok", false))


## One short line: the long caveats (an unbuilt Vault index, why Vault notes are unavailable) are in the
## tooltip (`_facts_status_detail`) so the line does not grow the window.
func _facts_status_text() -> String:
	if _place.is_empty():
		return ""
	if _facts.is_empty():
		return "Place facts: not read yet."
	var b: Dictionary = _facts.get("bundle", {})
	if not bool(b.get("ok", false)):
		return "Place facts: not available. %s" % String(b.get("reason", ""))
	var t := "Place facts: %d sections, %d characters (%d ms)." % [(b.get("sections", []) as Array).size(), int(b.get("chars", 0)), int(_facts.get("gather_ms", 0))]
	var vault: Dictionary = _facts.get("vault", {})
	if bool(DccSettings.llm_value("ctx_notes")) and (not bool(vault.get("ok", true)) or not (vault.get("notes", []) as Array).is_empty()):
		t += " Vault: see the tooltip."
	return t


## What the short facts line leaves out: why the Vault half is unavailable, and the Vault's own caveats.
func _facts_status_detail() -> String:
	var vault: Dictionary = _facts.get("vault", {})
	var parts := PackedStringArray()
	if bool(DccSettings.llm_value("ctx_notes")) and not bool(vault.get("ok", true)):
		parts.append("Vault notes: %s" % String(vault.get("reason", "not available")))
	for n in vault.get("notes", []):
		parts.append(String(n))
	return "
".join(parts)


func _on_view_facts() -> void:
	if _facts.is_empty() or not bool(_facts.get("bundle", {}).get("ok", false)):
		return
	var b: Dictionary = _facts["bundle"]
	var bundle: Dictionary = JSON.parse_string(String(b["bundle"]))
	var text := "WORLD DATA (generated, authoritative for figures)\n" + String(b["text"])
	if String(bundle.get("vault_text", "")) != "":
		text += "\nVAULT NOTES (your writing, authoritative for names and story)\n" + String(bundle["vault_text"])
	_add_msg("Facts for %s (shown to you; sent only when you press Send)" % String(_facts.get("name", "")), text, "text_secondary")
	controls["facts_shown"] = true


func _on_refresh_facts() -> void:
	_place = _current_place()
	_ensure_facts(true)
	_refresh()


## The grounding check, shown on a card after a place entry finishes. It never blocks and never
## edits the reply: it lists which numbers and names came from the data (facts used) and which
## the data does not support (unverified claims), plus anything the model marked as flavour.
func _show_grounding() -> void:
	if _task != "prose" or _sent_facts.is_empty() or _reply.strip_edges() == "" or _reply == _grounded_reply:
		return
	_grounded_reply = _reply
	var g: Dictionary = run.stream.grounding_check(_reply, String(_sent_facts["bundle"].get("corpus", "")))
	last_grounding = g
	var used := PackedStringArray()
	for n in g.get("numbers_matched", []):
		used.append(String(n))
	for pair in g.get("numbers_approx", []):
		used.append("%s (about %s)" % [String(pair[0]), String(pair[1])])
	for n in g.get("names_matched", []):
		used.append(String(n))
	var bad := PackedStringArray()
	for n in g.get("numbers_unverified", []):
		bad.append("%s (number)" % String(n))
	for n in g.get("names_unverified", []):
		bad.append("%s (name)" % String(n))
	var t := "Facts used (%d): %s" % [used.size(), ", ".join(used) if not used.is_empty() else "none found in the reply"]
	t += "\nUnverified claims (%d): %s" % [bad.size(), ", ".join(bad) if not bad.is_empty() else "none"]
	var fl: Array = g.get("flavour", [])
	if not fl.is_empty():
		t += "\nMarked as invented flavour (%d): %s" % [fl.size(), " | ".join(PackedStringArray(fl))]
	t += "\nThis only compares numbers and names with the data; it cannot judge whether a sentence contradicts it. Nothing is blocked."
	var lab := _add_msg("Grounding check", t, "warn" if bad.size() > 0 else "good")
	controls["grounding"] = lab


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
	_relayout_phone.call_deferred()


## **Phone only: re-lay the sheet's content out after a task change.** The sheet's child (`wrap_controls` is off)
## keeps the size it got at the LAST window layout, and the first layout happens in the default Prose task, whose
## minimum height can exceed the sheet (measured 2026-10-26: 887 px in a 832 px window once the knowledge line was
## in the prose "This will send" text). Switching to Commands then left the content 55 px taller than the sheet and
## Send under the dialog's Close row, where a tap never reaches it. Nudging the window size makes the dialog
## recompute its child's rect from the window and the new, smaller minimum. A no-op off a phone.
func _relayout_phone() -> void:
	if not _phone or not visible:
		return
	await get_tree().process_frame
	if not is_instance_valid(self) or not visible:
		return
	var s := size
	size = s + Vector2i(0, 1)
	size = s


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


## True when either knowledge box (Vault notes, world data) is ticked.
func _knowledge_ticked() -> bool:
	return bool(DccSettings.llm_value("ctx_rag_vault")) or bool(DccSettings.llm_value("ctx_rag_world"))


## The sentence the "This will send" line adds for the knowledge base ("" when no box is ticked),
## quoting the caps from Rust (`knowledge_limits`) so the line equals what the code enforces.
func _knowledge_sent_text() -> String:
	if not _knowledge_ticked() or not run.available():
		return ""
	var p: Dictionary = LlmKnowledge.plan(run.stream, _capability())
	if not bool(p.get("ok", false)):
		return ""
	if not LlmKnowledge.any(p):
		var why := String(p.get("note", ""))
		return " Knowledge search is not used for this request." + ("" if why == "" else " " + why)
	var lim: Dictionary = run.stream.knowledge_limits()
	var what := PackedStringArray()
	if bool(p.get("vault", false)):
		what.append("Vault notes")
	if bool(p.get("world", false)):
		what.append("world data")
	var parts := PackedStringArray()
	if bool(p.get("pre", false)):
		parts.append("the best %d excerpts (up to %d characters) go first" % [int(lim["pack_hits"]), int(lim["pack_chars"])])
	if bool(p.get("tools", false)) and _task == "commands":
		parts.append("the model may look up to %d times (each result at most %d bytes)" % [int(lim["calls_per_request"]), int(lim["result_bytes"])])
	## Kept short: on a phone every extra line pushes Send off the sheet (measured 2026-10-26). The consent card
	## carries the full statement.
	return " Search in %s: %s. Never private notes; world data has no private marker." % [
		" and ".join(what), "; ".join(parts) if not parts.is_empty() else "off for this kind of request"]


## True once the last reply drew a sources card (so the empty-proposal note is not shown for a plain answer).
var _sources_seen := false
## The sources cards drawn, for probes: each `{info, box}`.
var source_cards: Array = []


## Draws the sources a reply drew on and the citations in it (RK-6, K8): a WORLD DATA / VAULT NOTE chip per
## source sent, then each citation, and a citation to something the app never retrieved as UNVERIFIED (never a
## link). Widgets only: nothing here acts, and nothing is written.
func _on_sources(info: Dictionary) -> void:
	_sources_seen = true
	var box := PanelContainer.new()
	box.add_theme_stylebox_override("panel", DccWidgets.box("border", "sunken", 8, 6))
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 3)
	box.add_child(col)
	var srcs: Array = info.get("sources", [])
	var cits: Array = info.get("citations", [])
	col.add_child(DccTheme.label("SOURCES SENT TO THE MODEL (%d)" % srcs.size(), "text_dim", DccTheme.FS_MICRO))
	if srcs.is_empty():
		_card_text(col, "No source matched this request, so none was sent. If the answer states facts, they did not come from your notes or world.", "text_ghost")
	for s in srcs:
		var sd: Dictionary = s
		## The chip keeps its own width and the text takes the rest of the row (an HFlow gave the wrapped
		## text only its minimum width, a column of three-word lines, seen in the desktop shot of 2026-10-26).
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		var chip := DccTheme.label(String(sd.get("label", "SOURCE")), "accent" if String(sd.get("source", "")) == "world" else "warn", DccTheme.FS_MICRO)
		chip.custom_minimum_size.x = 76
		chip.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
		chip.tooltip_text = "Id: %s" % String(sd.get("id", ""))
		row.add_child(chip)
		var t := DccTheme.label("%s  ·  %s" % [String(sd.get("title", "")), String(sd.get("locator", ""))], "text_secondary", DccTheme.FS_SMALL)
		t.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		t.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		t.custom_minimum_size.x = WRAP_MIN_W - 76
		t.tooltip_text = "Id: %s" % String(sd.get("id", ""))
		row.add_child(t)
		col.add_child(row)
	if not cits.is_empty():
		col.add_child(DccTheme.label("CITATIONS IN THE ANSWER", "text_dim", DccTheme.FS_MICRO))
	for c in cits:
		var cd: Dictionary = c
		if bool(cd.get("verified", false)):
			_card_text(col, "[%s]  %s" % [String(cd.get("label", "")), String(cd.get("title", cd.get("id", "")))], "text")
		elif bool(cd.get("mentioned", false)):
			## Not retrieved itself, but a source that WAS sent names it and the app knows it exists.
			_card_text(col, "NAMED, NOT READ  %s  (a source that was sent names it; its own text was not sent, so this is not verified)" % String(cd.get("id", "")), "text_secondary")
		else:
			_card_text(col, "UNVERIFIED CITATION  %s  (the app never retrieved this id; do not rely on it)" % String(cd.get("id", "")), "warn")
	_msgs.add_child(box)
	source_cards.append({"info": info, "box": box})
	_scroll_to_end()


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
	if _task == "prose" and not _place.is_empty():
		## Fresh at every Send: the model is given the world as it is NOW, not as the cache last saw it.
		_place = _current_place()
		if _place.is_empty() or not _ensure_facts(true):
			_add_msg("", "The place's data could not be read, so no entry was requested. %s" % String(_facts.get("bundle", {}).get("reason", "")), "block")
			_refresh()
			return
	_add_msg("You", request, "text")
	_reply = ""
	_reply_label = _add_msg("Assistant", "", "text_secondary", true)
	var args := _params_args(request) if _task == "commands" else _args(request)
	_sources_seen = false
	## RK-6: the pre-retrieval search, run HERE (the shell holds the engine) only when Rust's plan for
	## this endpoint and these settings asks for it. For a place the place's name is added to the
	## words, so the place's own document is found (the place-facts context draws on the same base).
	var kplan: Dictionary = LlmKnowledge.plan(run.stream, _capability()) if run.available() else {}
	var kquery := request
	if _task == "prose" and not _place.is_empty():
		kquery = "%s %s" % [String(_place["name"]), request]
	var kraw := LlmKnowledge.search_pack(bridge, kplan, kquery) if run.available() else ""
	if kraw != "":
		args["knowledge_pack"] = kraw
	if _task == "commands":
		## LLM-6d: the live view facts (JSON text) and the groups the user ticked, if any.
		args["view"] = _VIEW.facts_json(app, bridge)
		if not _chosen_groups.is_empty():
			args["groups"] = PackedStringArray(_chosen_groups)
	var r := run.begin(DccSettings.llm_gate_dict(), _capability(), DccSettings.llm_consented_origins(), args)
	if not bool(r.get("ok", false)):
		_reply_label.get_parent().get_parent().queue_free()
		_reply_label = null
		_add_msg("", String(r.get("reason", "The request could not be started.")), "block")
	else:
		## K8: say what the app sent ahead of the model, in plain words.
		if r.has("pack_hits"):
			if String(r.get("pack_note", "")) != "":
				_add_msg("", "Searching your sources first: %s" % String(r["pack_note"]), "text_ghost")
			else:
				_add_msg("", "Searched your sources first and sent %d excerpt%s (about %d tokens). Sources are listed under the answer." % [int(r["pack_hits"]), "" if int(r["pack_hits"]) == 1 else "s", int(r["pack_tokens"])], "text_ghost")
		if String(r.get("knowledge_note", "")) != "":
			_add_msg("", String(r["knowledge_note"]), "warn")
		_input.text = ""
		_groups_were_chosen = not _chosen_groups.is_empty()
		_chosen_groups.clear()
		_sent_facts = _facts.duplicate(true) if args.has("place_bundle") else {}
		_grounded_reply = ""
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


## The counter is Rust's and has already recorded this reply; only the line needs redrawing.
func _on_usage(_info: Dictionary) -> void:
	_refresh()


func _on_reset_counter() -> void:
	if run.available():
		run.stream.reset_token_counts()
	_refresh()


## The one automatic retry has started. `failed` is NOT emitted for the first attempt, so say it here.
func _on_retrying(reason: String) -> void:
	_add_msg("", "%s. Trying once more; this is the only automatic retry." % reason.rstrip("."), "warn")
	_refresh()


func _on_finished(info: Dictionary) -> void:
	if info.get("finish_reason", "") == "length":
		_add_msg("", "The reply was cut at the length limit.", "warn")
	_show_grounding()
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


# ---------------------------------------------------------------------------
# Setting proposals (LLM-6b)
# ---------------------------------------------------------------------------

## A setting request moved on: say so in plain words (the model's own text, if any, is already
## in its bubble). A retry names why the first proposal could not be used.
func _on_params_step(info: Dictionary) -> void:
	if String(info.get("type", "")) == "params_retry":
		var v: Dictionary = info.get("verdict", {})
		var why := _verdict_reasons(v) if info.has("verdict") else _view_reasons(info.get("view_verdict", {}))
		_add_msg("", "The model's proposal could not be used (%s). It was told why and asked once more." % why, "warn")
	else:
		var gs: PackedStringArray = info.get("groups", PackedStringArray())
		var qs: PackedStringArray = info.get("queries", PackedStringArray())
		if not qs.is_empty():
			## Read-only lookups (LLM-8): name them, so the user can see what the model read. The
			## answers themselves go to the model, not to this list.
			_add_msg("", "The model looked up: %s. Nothing was changed." % ", ".join(qs), "text_ghost")
		elif gs.is_empty():
			_add_msg("", "The model asked for setting groups that do not exist; it was told the valid ones.", "text_ghost")
		else:
			_add_msg("", "The model is reading the %s settings…" % ", ".join(gs), "text_ghost")
	_refresh()


## The model answered in words without proposing (an impossible request, a question).
func _on_answered(_text: String) -> void:
	_drop_empty_reply()
	if _sources_seen:
		## A knowledge answer: the text above is the answer and the sources card follows it.
		_refresh()
		return
	var hint := "" if _groups_were_chosen else " If the model could not pick setting groups, press Choose groups, tick up to %d and send again." % MAX_CHOSEN_GROUPS
	_add_msg("", "Nothing was proposed.%s" % hint, "text_ghost")
	_refresh()


## The refused changes or the structural failure, in one line.
func _verdict_reasons(v: Dictionary) -> String:
	if v.has("structural"):
		return "%s: %s" % [String(v.get("structural_rule", "")), String(v["structural"])]
	var parts: PackedStringArray = PackedStringArray()
	for r in v.get("rejected", []):
		var rd: Dictionary = r
		parts.append("%s %s: %s" % [String(rd.get("rule", "")), String(rd.get("key", "(no key)")), String(rd.get("why", ""))])
	return "; ".join(parts) if not parts.is_empty() else "nothing could be applied"


## A value with its unit for the card ("4000 m", "1.00 x", "on").
func _fmt_value(v, unit: String) -> String:
	var t := ""
	match typeof(v):
		TYPE_BOOL:
			t = "on" if bool(v) else "off"
		TYPE_INT:
			t = str(int(v))
		TYPE_FLOAT:
			## Two decimals below 100 (the scope's "1.00 x -> 0.70 x"), whole numbers above
			## (metres, years); the card shows the value, the model's number is never rounded
			## in what is applied (`_apply_card` writes `new` as received).
			var f := float(v)
			t = "%.2f" % f if absf(f) < 100.0 else "%d" % int(round(f))
		_:
			t = str(v)
	return t if unit == "" or typeof(v) == TYPE_BOOL else "%s %s" % [t, unit]


## Removes the assistant bubble of a setting request when the model streamed no words into
## it (a pure tool call), so the card is not preceded by an empty model-generated box.
func _drop_empty_reply() -> void:
	if _reply_label != null and _reply.strip_edges() == "" and is_instance_valid(_reply_label):
		_reply_label.get_parent().get_parent().queue_free()
		_reply_label = null


## Draws the proposal card (scope s.5b.3's text layout, in the DCC vocabulary). Builds
## widgets only: nothing is written here.
func _on_proposal(info: Dictionary) -> void:
	_drop_empty_reply()
	var box := PanelContainer.new()
	box.add_theme_stylebox_override("panel", DccWidgets.box("warn", "sunken", 8, 6))
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 4)
	box.add_child(col)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 6)
	var chip := DccTheme.label("MODEL PROPOSAL", "warn", DccTheme.FS_MICRO)
	chip.tooltip_text = "Proposed by a language model. Nothing has changed; review it and press Apply or Dismiss."
	head.add_child(chip)
	col.add_child(head)
	if info.has("summary"):
		_card_text(col, "\u201c%s\u201d" % String(info["summary"]), "text_secondary")
	var valid: Array = info.get("valid", [])
	for c in valid:
		var cd: Dictionary = c
		var unit := String(cd.get("unit", ""))
		var line := "%s  (%s)   %s  ->  %s" % [String(cd["label"]), String(cd["key"]),
			_fmt_value(cd["old"], unit) if cd.has("old") else "not readable", _fmt_value(cd["new"], unit)]
		if cd.has("pct"):
			line += "   %+d%%" % int(round(float(cd["pct"])))
		if cd.has("default") and cd.has("old") and str(cd["default"]) != str(cd["old"]):
			line += "   (default %s)" % _fmt_value(cd["default"], unit)
		var row_l := _card_text(col, line, "text")
		if bool(cd.get("advanced", false)):
			row_l.text += "   ADVANCED: ruled behaviour"
		if cd.has("reason"):
			_card_text(col, "    \u201c%s\u201d  (model's words)" % String(cd["reason"]), "text_dim")
		if cd.has("inert"):
			_card_text(col, "    " + String(cd["inert"]), "warn")
	var rejected: Array = info.get("rejected", [])
	if info.has("structural") or not rejected.is_empty():
		_card_text(col, "Not applied:", "text_dim")
		if info.has("structural"):
			_card_text(col, "    %s: %s" % [String(info.get("structural_rule", "")), String(info["structural"])], "text_ghost")
		for r in rejected:
			var rd: Dictionary = r
			_card_text(col, "    %s  %s: %s" % [String(rd.get("key", "(no key)")), String(rd["rule"]), String(rd["why"])], "text_ghost")
	var noop: PackedStringArray = info.get("noop", PackedStringArray())
	if not noop.is_empty():
		_card_text(col, "No change (already at that value): %s" % ", ".join(noop), "text_ghost")
	var apply_b: Button = null
	var reason_l: Label = null
	if valid.is_empty():
		_card_text(col, "Nothing could be applied.", "warn")
	else:
		for line in info.get("cost", PackedStringArray()):
			_card_text(col, String(line), "text_secondary")
		_card_text(col, "Applying changes settings only. It does not regenerate.", "text_secondary")
	## The reason sits on its OWN line above the buttons: beside them its wrap minimum plus the
	## two buttons was wider than a 393 px phone sheet, and a horizontally-disabled scroller
	## folds that width into every ancestor (measured: Send pushed off screen on the phone).
	reason_l = _note_label(col, "", "warn")
	reason_l.visible = false
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.alignment = BoxContainer.ALIGNMENT_END
	col.add_child(row)
	var card := {"info": info, "box": box, "apply": null, "dismiss": null, "reason": reason_l, "state": "new", "kind": "params", "group_id": 0}
	var dismiss_b := _button(row, "Dismiss", func(): _dismiss_card(card))
	card["dismiss"] = dismiss_b
	if not valid.is_empty():
		apply_b = _button(row, "Apply %d change%s" % [valid.size(), "" if valid.size() == 1 else "s"], func(): _apply_card(card), true)
		card["apply"] = apply_b
	_msgs.add_child(box)
	cards.append(card)
	_watch_undo()
	if info.has("view"):
		## A reply that carried both tools: the second card is the view half.
		_draw_view_card(info["view"])
	_refresh_cards()
	_scroll_to_end()


## One wrapped line of card text.
func _card_text(parent: Control, text: String, token: String) -> Label:
	var l := DccTheme.label(text, token, DccTheme.FS_SMALL)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	l.custom_minimum_size.x = WRAP_MIN_W
	parent.add_child(l)
	return l


## V13 for every open card: Apply is disabled with the reason beside it while a world
## generates or is finalized (`LlmStream.apply_availability`, the Rust wording).
func _refresh_cards() -> void:
	## A card whose widgets were freed (a rebuild that missed the array) is dropped, never read:
	## belt and braces over `_rebuild`'s clear, so a stale entry cannot raise again.
	cards = cards.filter(func(c): return is_instance_valid(c["box"]))
	for card in cards:
		var b: Button = card["apply"]
		var rl: Label = card["reason"]
		if card["state"] != "new":
			continue
		var why := _view_blocked(card["info"]) if String(card.get("kind", "params")) == "view" else _apply_blocked()
		if b != null:
			b.disabled = why != ""
			b.tooltip_text = why
		rl.text = why
		rl.visible = why != ""


## Why Apply cannot run now, or "".
func _apply_blocked() -> String:
	if not run.available():
		return "This build has no assistant support."
	return String(run.stream.apply_availability(bridge.generating, bridge.is_finalized()))


func _close_card(card: Dictionary, state: String, text: String, token: String) -> void:
	card["state"] = state
	for k in ["apply", "dismiss"]:
		var b: Button = card[k]
		if b != null and is_instance_valid(b):
			b.disabled = true
	var rl: Label = card["reason"]
	rl.text = text
	rl.visible = true
	rl.add_theme_color_override("font_color", DccTheme.c(token))


func _dismiss_card(card: Dictionary) -> void:
	if card["state"] != "new":
		return
	_close_card(card, "dismissed", "Dismissed. Nothing was changed.", "text_ghost")


## **The only writer of settings in this file**, reached only from the card's Apply button.
## One undo group: every change through `write_param` (`param_write`, the dialogs' setter); a
## refused or clamped write rolls the group back (`undo_group.gd`). Never regenerates.
func _apply_card(card: Dictionary) -> void:
	if card["state"] != "new":
		return
	var why := _apply_blocked()
	if why != "":
		_refresh_cards()
		return
	var info: Dictionary = card["info"]
	var ug = app.undo_groups
	var label := "Assistant: %s" % String(info.get("summary", "setting changes")).left(60)
	var refused: String = ug.begin(label)
	if refused != "":
		_close_card(card, "failed", "Not applied: %s." % refused, "block")
		return
	for c in info.get("valid", []):
		var cd: Dictionary = c
		if not ug.write_param(String(cd["key"]), cd["new"]):
			## `write_param` already rolled the whole group back.
			_close_card(card, "failed", "Not applied: %s. Nothing was changed." % String(ug.last_reason), "block")
			return
	var r: Dictionary = ug.commit()
	if not bool(r.get("ok", false)):
		_close_card(card, "failed", "Not applied: %s. Nothing was changed." % String(r.get("reason", "")), "block")
		return
	card["group_id"] = int(r.get("id", 0))
	card["applied_text"] = "Applied as one undo step (Edit ▸ Undo reverts all of it). Regenerate or Recompute to see it."
	_close_card(card, "applied", String(card["applied_text"]), "good")
	if app.has_method("set_status"):
		app.set_status("hint", "Assistant: %d setting%s changed. Nothing was regenerated." % [int(r.get("members", 0)), "" if int(r.get("members", 0)) == 1 else "s"])


# ---------------------------------------------------------------------------
# View and layer actions (LLM-6d)
# ---------------------------------------------------------------------------

## The view proposal of a reply that carried only `propose_view_actions`.
func _on_view_proposal(info: Dictionary) -> void:
	_drop_empty_reply()
	_draw_view_card(info.get("view", {}))
	_watch_undo()
	_refresh_cards()
	_scroll_to_end()


## The refused actions or the structural failure, in one line.
func _view_reasons(v: Dictionary) -> String:
	if v.has("structural"):
		return "%s: %s" % [String(v.get("structural_rule", "")), String(v["structural"])]
	var parts: PackedStringArray = PackedStringArray()
	for r in v.get("rejected", []):
		var rd: Dictionary = r
		parts.append("%s %s: %s" % [String(rd.get("rule", "")), String(rd.get("action", "(no action)")), String(rd.get("why", ""))])
	return "; ".join(parts) if not parts.is_empty() else "nothing could be applied"


## What the view facts disclose, for the "This will send" line: states and names the user can
## already see on screen, no world data.
func _view_sent_text() -> String:
	return "It also receives what the map shows (the layers and whether each is on, the style preset names, the landmark types this world has, the grid size and zoom range, and the names of windows it may open) so it can propose view changes."


## Draws the view card (the parameter card's vocabulary): each valid action as `label  was -> will
## be`, the model's reason as its own words, the refused actions with their rule, the undo
## statement Rust wrote, and Apply / Dismiss. Builds widgets only: nothing is applied here.
func _draw_view_card(v: Dictionary) -> void:
	var box := PanelContainer.new()
	box.add_theme_stylebox_override("panel", DccWidgets.box("warn", "sunken", 8, 6))
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 4)
	box.add_child(col)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 6)
	var chip := DccTheme.label("MODEL PROPOSAL · VIEW", "warn", DccTheme.FS_MICRO)
	chip.tooltip_text = "Proposed by a language model. Nothing has changed; review it and press Apply or Dismiss."
	head.add_child(chip)
	col.add_child(head)
	if v.has("summary"):
		_card_text(col, "\u201c%s\u201d" % String(v["summary"]), "text_secondary")
	var valid: Array = v.get("valid", [])
	for c in valid:
		var cd: Dictionary = c
		## A window has no "before" to show: it reads "<name>   will be opened". Anything else with no
		## readable old value says so (never a blank or a default).
		var line := "%s   will be opened" % String(cd["label"]) if String(cd.get("action", "")) == "open_window" else "%s   %s  ->  %s" % [String(cd["label"]), String(cd["from"]) if cd.has("from") else "not readable", String(cd["to"])]
		if bool(cd.get("undoable", false)):
			line += "   (one undo step)"
		_card_text(col, line, "text")
		if cd.has("reason"):
			_card_text(col, "    \u201c%s\u201d  (model's words)" % String(cd["reason"]), "text_dim")
		if cd.has("note"):
			_card_text(col, "    " + String(cd["note"]), "warn")
	var rejected: Array = v.get("rejected", [])
	if v.has("structural") or not rejected.is_empty():
		_card_text(col, "Not applied:", "text_dim")
		if v.has("structural"):
			_card_text(col, "    %s: %s" % [String(v.get("structural_rule", "")), String(v["structural"])], "text_ghost")
		for r in rejected:
			var rd: Dictionary = r
			_card_text(col, "    %s  %s: %s" % [String(rd.get("action", "(no action)")), String(rd["rule"]), String(rd["why"])], "text_ghost")
	var noop: PackedStringArray = v.get("noop", PackedStringArray())
	if not noop.is_empty():
		_card_text(col, "No change: %s" % "; ".join(noop), "text_ghost")
	if valid.is_empty():
		_card_text(col, "Nothing could be applied.", "warn")
	else:
		for line in v.get("undo", PackedStringArray()):
			_card_text(col, String(line), "text_secondary")
		_card_text(col, "Applying changes how the map is shown. It does not change the world or its settings.", "text_secondary")
	var reason_l := _note_label(col, "", "warn")
	reason_l.visible = false
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.alignment = BoxContainer.ALIGNMENT_END
	col.add_child(row)
	var card := {"info": v, "box": box, "apply": null, "dismiss": null, "reason": reason_l, "state": "new", "kind": "view", "group_id": 0}
	card["dismiss"] = _button(row, "Dismiss", func(): _dismiss_card(card))
	if not valid.is_empty():
		card["apply"] = _button(row, "Apply %d view change%s" % [valid.size(), "" if valid.size() == 1 else "s"], func(): _apply_view_card(card), true)
	_msgs.add_child(box)
	cards.append(card)


## Why a view card cannot be applied now, or "". Only the style preset is blocked by a running
## generation (it writes the engine's look); layers, landmark filters, the camera and windows
## are session state and do not depend on it.
func _view_blocked(info: Dictionary) -> String:
	if not run.available():
		return "This build has no assistant support."
	for c in info.get("valid", []):
		if bool((c as Dictionary).get("undoable", false)) and bridge.generating:
			return "A world is generating; the style cannot change until it finishes."
	return ""


## **The only writer of view state in this file**, reached only from a view card's Apply. The
## style preset goes through the grouped undo (one step); everything else is instant session state
## (`llm_view.gd::apply` has the order and the re-check). Never regenerates.
func _apply_view_card(card: Dictionary) -> void:
	if card["state"] != "new":
		return
	var why := _view_blocked(card["info"])
	if why != "":
		_refresh_cards()
		return
	var valid: Array = card["info"].get("valid", [])
	var r: Dictionary = _VIEW.apply(app, bridge, app.undo_groups, valid)
	if not bool(r.get("ok", false)):
		_close_card(card, "failed", String(r.get("reason", "Not applied.")), "block")
		return
	card["group_id"] = int(r.get("group_id", 0))
	var n := int(r.get("applied", 0))
	var text := "Applied %d view change%s." % [n, "" if n == 1 else "s"]
	if int(card["group_id"]) > 0:
		text += " Edit ▸ Undo reverts the style preset in one step; the other changes are session state and are not on the undo stack."
	elif n > 0:
		text += " They are session state, not on the undo stack."
	var failed: Array = r.get("failed", [])
	if not failed.is_empty():
		text += " Not applied: %s." % "; ".join(PackedStringArray(failed))
	card["applied_text"] = text
	_close_card(card, "applied", text, "warn" if not failed.is_empty() else "good")
	if app.has_method("set_status"):
		app.set_status("hint", "Assistant: %d view change%s applied." % [n, "" if n == 1 else "s"])


## Connects `app.undo_groups.changed` once, so a card learns when Edit ▸ Undo or Redo moved its group.
func _watch_undo() -> void:
	var ug = app.get("undo_groups")
	if ug != null and not ug.changed.is_connected(_sync_card_undo):
		ug.changed.connect(_sync_card_undo)


## The "(undone)" state (LLM-6d): a card that was applied inside an undo group follows that group.
## `applied` -> `undone` when Edit ▸ Undo took the group, back to `applied` on Redo, and a group that
## left the stack for good (a newer edit, a different world) is said to be gone, never shown as
## still revertible. Only cards that own a group (`group_id` > 0) are touched.
func _sync_card_undo() -> void:
	cards = cards.filter(func(c): return is_instance_valid(c["box"]))
	var ug = app.get("undo_groups")
	if ug == null:
		return
	for card in cards:
		var gid := int(card.get("group_id", 0))
		var st := String(card["state"])
		if gid <= 0 or (st != "applied" and st != "undone"):
			continue
		var is_view := String(card.get("kind", "params")) == "view"
		match String(ug.state_of(gid)):
			"applied":
				if st == "undone":
					_close_card(card, "applied", String(card.get("applied_text", "Applied.")), "good")
			"undone":
				if st == "applied":
					_close_card(card, "undone", ("Undone with Edit ▸ Undo: the style is back as it was, and the map was redrawn, which re-fits the view. Layer, landmark and window changes were session state and stay as they were applied. Edit ▸ Redo re-applies the style." if is_view
						else "Undone with Edit ▸ Undo: the settings are back to what they were before this card. Nothing was regenerated. Edit ▸ Redo re-applies them."), "text_dim")
			_:
				if st == "undone":
					_close_card(card, "undone", "Undone. A newer change replaced it, so it can no longer be redone.", "text_ghost")
				elif not bool(card.get("gone", false)):
					card["gone"] = true
					_close_card(card, "applied", String(card.get("applied_text", "Applied.")) + " It is no longer on the undo stack (a newer edit or a different world replaced it).", "text_dim")


## The setting groups, in the registry's order (read from the engine's own parameter table, not typed here).
func _group_names() -> Array:
	var seen: Array = []
	for k in bridge.param_keys():
		var g := String(bridge.param_info(String(k)).get("group", ""))
		if g != "" and not seen.has(g):
			seen.append(g)
	return seen


func _toggle_groups() -> void:
	_groups_flow.visible = not _groups_flow.visible
	_refresh()


## Ticks or unticks a setting group for the next Send. A fourth tick is refused (the chip is
## disabled with the reason); Rust re-validates the list anyway.
func _tick_group(name: String) -> void:
	if run.busy():
		return
	if _chosen_groups.has(name):
		_chosen_groups.erase(name)
	elif _chosen_groups.size() < MAX_CHOSEN_GROUPS:
		_chosen_groups.append(name)
	_refresh()


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
