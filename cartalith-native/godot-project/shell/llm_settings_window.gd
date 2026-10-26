extends AcceptDialog
class_name LlmSettingsWindow

## **`Preferences ▸ Assistant…`** -- the settings section for the optional LLM assistant
## (`LLM_ASSISTANT_SCOPE.md` milestone LLM-2; Rulings DC, DD, DF).
##
## Design: `design/llm-assistant-2026-10-25/index.html` (LLM-0, owner-approved). The mock
## draws the settings as a tab of the assistant panel; the panel itself is LLM-3, so this
## ships the settings group as its own window, opened from Preferences next to Storage
## locations, which is the placement `LLM_ASSISTANT_SCOPE.md` s.6 names ("a settings group
## beside the Preferences and Performance groups"). One script, three densities: the pointer
## build is a centred window, the tablet takes `DccWidgets`' tablet floors, and the phone is
## a full-height borderless sheet with a titled header (`DccWidgets.phone_window`).
##
## ## What this window does, and what it never does
##
## * **Settings only, and no network.** Nothing here opens a socket or resolves a name;
##   the assistant panel and the wire are LLM-3. Every decision (address class, the gate,
##   the GPU rule, key state) is made by `cartalith-llm` through `EngineBridge.llm_*`, and
##   this file only displays the answer. A reason is shown in words, never a dead control.
## * **Default off.** Every switch ships off; the provider, address and model ship empty
##   (`DccSettings.LLM_DEFAULTS`). Turning the master switch on with a usable endpoint
##   opens the consent card first; declining leaves it off.
## * **The key is never echoed.** The entry field is a `secret` `LineEdit`, is cleared the
##   instant Save is pressed (success or not), and is cleared again on every rebuild; the
##   status line shows `masked` (`****` plus the last four characters) and nothing more.
##   The key is not written to `cartalith_settings.cfg` (that file's setter refuses unknown
##   keys), a save, an export or the log. Where there is no OS credential store the Save
##   control is disabled with the reason, and an environment variable is the only source.
## * **The GPU warning** (Ruling DF) shows whenever a local provider is selected, always
##   with a "Detected as" line and what will happen; the override (Automatic / Always
##   low-power GPU / Never change) is stored in `[llm]`, and `LlmGpuRule.sync` applies it
##   to the compute path (never persisted, restored on deselect).
##
## ## Structure
##
## `_rebuild()` rebuilds the whole body from stored state; it runs on open and after a
## structural change (provider, key source, master switch). Text fields commit on Enter or
## focus loss and call `_refresh_dynamic()` instead -- a full rebuild on focus loss would
## free the field the user was tabbing into. `controls` maps a stable name to each control
## so the probe can drive them.

## Emitted after any stored setting changed and the dependent state was resynced.
signal settings_changed()

## The owning `DccApp` (untyped, like sibling windows).
var app
## The engine bridge; every `llm_*` call goes through it.
var bridge: EngineBridge
## Stable name -> control, refilled on every `_rebuild()`. For probes.
var controls: Dictionary = {}
## The most recent consent card (`LlmConsent.present`'s return), for probes.
var last_consent: Dictionary = {}

var _phone := false
var _body: VBoxContainer
var _scroll: ScrollContainer
## `WorldGen.llm_presets()` rows, read at each rebuild (index + 1 is the picker row).
var _presets: Array = []
## Dynamic labels refreshed by `_refresh_dynamic()`.
var _status_dot: Label
var _status_text: Label
var _endpoint_text: Label
var _key_text: Label
var _key_msg: Label
var _consent_text: Label
var _gpu_box: VBoxContainer
var _gpu_headline: Label
var _gpu_detected: Label
var _gpu_effect: Label
## The rows whose enabled state follows the master switch.
var _dependent: Array = []
## True while `_rebuild()` is filling controls, so their signals do not write settings.
var _building := false

## The model dropdown's state lives HERE, not in the controls, so a rebuild (which frees them) does not
## lose it. `_ml_state` is `idle`, `loading`, `ok`, `failed` or `refused`.
var _ml: LlmModelList
var _ml_state := "idle"
var _ml_msg := ""
var _ml_code := ""
var _ml_key := ""
var _ml_manual := false
var _ml_tick := 0.0
## The "Model name" field's row, shown only while typing a name by hand.
var _model_field_row: Control
## The dropdown's rows, `[{id, text}]`, and how many hidden embedding models the last list held.
var _ml_rows: Array = []

## The switches, in the mock's order: key in `DccSettings`, label, sub-note, and whether the
## switch can be used in this build at all (`unavailable` is the reason when it cannot).
const _SWITCHES := [
	{"key": "enabled", "label": "Assistant enabled", "note": "The master switch. Off means nothing is sent."},
	{"key": "prose", "label": "Prose and Vault assist",
		"note": "Prose for a place, its story, or a Vault note, from Story ▸ Notes ▸ Assistant panel. Notes are written only on your click."},
	{"key": "commands", "label": "Commands: view, layer, parameter",
		"note": "Generation-setting proposals and view or layer commands (LLM-6): the model proposes, you review the card and press Apply; one Apply is one undo step and never regenerates. Also carries questions about the world and your notes."},
	{"key": "edit", "label": "Edit actions: sculpt, paint",
		"unavailable": "Not available in this build yet: edit actions (LLM-7) are not built. Grouped undo, which they need, exists."},
	{"key": "local_allowed", "label": "Local providers allowed", "note": "This computer and your local network.",
		"note_phone": "This phone and your local network, such as a model server on your computer."},
	{"key": "remote_allowed", "label": "Remote providers allowed", "note": "Hosted endpoints on the internet."},
]
## The context sources (scope s.5: "none; per source"), each its own switch.
const _CONTEXT := [
	{"key": "ctx_entity", "label": "Send: current entity"},
	{"key": "ctx_selection", "label": "Send: selection"},
	{"key": "ctx_notes", "label": "Send: named Vault notes"},
	## Ruling DI P3: generation parameters are their own source, off until ticked. A setting
	## proposal shows the model each offered setting's current value, so it needs this.
	{"key": "ctx_params", "label": "Send: generation parameters and summary statistics",
		"note": "Needed for setting proposals and for questions: the model is shown the current value of each setting it asks about, and can read whole-world figures (sea level, land fraction, elevation percentiles, mean rainfall and temperature, productivity) and up to 8 single map cells. Reading changes nothing."},
	## RK-5 (Rulings DM and DN K1): the knowledge base's two sources, each its own box, both off.
	{"key": "ctx_rag_vault", "label": "Send: Vault notes found by search",
		"note": "Excerpts of the notes that match your request, never a whole note unasked and never a note you marked private (dm or private in its properties, the tag dm-only or secret, or a DM, Private or Secrets folder)."},
	{"key": "ctx_rag_world", "label": "Send: world data found by search",
		"note": "The facts the app holds about this world that match your request: names, populations, who owns what, trade partners, faiths, history. World data has no private marker: everything in it is sent as found."},
]
## RK-5/RK-6 (K6): how the knowledge base reaches the model. Shown under the context sources.
const _RAG_PRE := [
	["auto", "Search before asking (automatic: on for a local model)"],
	["on", "Search before asking (always)"],
	["off", "Do not search before asking"],
]
## The three GPU override values, in the picker's order (ids match `GpuOverride::id`).
const _GPU_OVERRIDES := [
	["automatic", "Automatic"],
	["always", "Always low-power GPU"],
	["never", "Never change"],
]
## The key-source picker's rows (ids match `KeySource::id`).
const _KEY_SOURCES := [["env", "Environment variable"], ["store", "OS credential store"]]


## Builds the window shell once; `open()` fills it. Call before `add_child`'s siblings need it.
func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "Assistant"
	size = Vector2i(560, 760)
	min_size = Vector2i(440, 480)
	get_ok_button().text = "Close"
	_phone = DccWidgets.phone_window(self, a)
	var host := VBoxContainer.new()
	host.add_theme_constant_override("separation", 0)
	add_child(host)
	if _phone:
		DccWidgets.phone_head(host, "Assistant", "optional · off by default")
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	host.add_child(_scroll)
	var pad := DccWidgets.pad(_scroll, 14, 10, 14, 10)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 8)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)
	_ml = LlmModelList.new()
	add_child(_ml)
	_ml.finished.connect(_on_models_finished)
	## Only ticks the age text of a list that is already shown once a second; starts nothing.
	set_process(true)
	## Closing must not leave a key in a field. `_rebuild` clears it on open too.
	visibility_changed.connect(func():
		if not visible and controls.has("key_field") and is_instance_valid(controls["key_field"]):
			(controls["key_field"] as LineEdit).text = "")


## Opens the window with the stored state.
func open() -> void:
	_rebuild()
	## Opening the window asks the server for its models (when the gate allows: a refusal draws
	## its reason and dials nothing).
	_models_refresh()
	if _phone:
		app.phone_fit(self, 1.0)
	if not DccWidgets.phone_present(self, app):
		popup_centered()


# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

## Removes and frees every child of `node` (removal first, so the container lays out this
## frame and not after the deferred free).
func _clear(node: Control) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()


## A wrapped ghost note under a control.
func _note(parent: Control, text: String, token: String = "text_ghost") -> Label:
	var l := DccTheme.label(text, token, DccTheme.FS_MICRO)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(l)
	return l


## Phone only: an EMPTY note is hidden instead of drawn as a blank line. The three `Send:` switches carry no
## note, and the label, empty, still reserved a full text line plus the body's 8 dp separation under each of
## them: the large blank gaps the owner reported on the phone's Assistant settings (2026-10-26). Called wherever a
## note's text is set, because a later refresh can fill one (`Needs the master switch.`). Other forms are untouched.
func _hide_if_empty(l: Label) -> void:
	if _phone:
		l.visible = l.text != ""


## A tracked section heading.
func _heading(text: String) -> void:
	var l := DccTheme.mono_label(text, "text_dim", DccTheme.FS_HEADER, 1)
	var m := DccWidgets.pad(_body, 0, 8, 0, 0)
	m.add_child(l)


## A status line: a coloured dot and wrapped text. Returns `[dot, text]`.
func _status_line(parent: Control) -> Array:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	var dot := DccTheme.label("●", "text_ghost", DccTheme.FS_SMALL)
	dot.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	row.add_child(dot)
	var t := DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	t.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	t.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(t)
	parent.add_child(row)
	return [dot, t]


## Sets a status dot's colour from a token.
func _dot(dot: Label, token: String) -> void:
	dot.add_theme_color_override("font_color", DccTheme.c(token))


## The height a text field must be at this density: 44 on a phone (the touch floor), the
## tablet button floor on a tablet, and a compact figure on a pointer.
func _field_h() -> int:
	if DccTheme.is_phone():
		return DccTheme.PHONE_TAP_MIN
	if DccTheme.is_tablet():
		return DccTheme.role_px("btn_min_h")
	return 26


## Floors a control this window built to the touch size, **on both axes where it is a switch**.
## `DccWidgets.toggle` and `choice` floor only the tablet's HEIGHT (`btn_min_h`) and a phone not at
## all (a 24 px switch row measured on the phone pass), and `phone_fit()` runs once at open while
## this window rebuilds its body afterwards, so the floor is applied here, at every creation site
## (`MISTAKES.md`: "Floor a control that a rebuild creates ... on both axes"). A switch keeps its
## drawn 40 px track; the extra width is hit area only.
func _floor(c: Control, switch: bool = false) -> void:
	if DccTheme.is_phone():
		c.custom_minimum_size.y = maxf(c.custom_minimum_size.y, float(DccTheme.PHONE_TAP_MIN))
		if switch:
			c.custom_minimum_size.x = maxf(c.custom_minimum_size.x, float(DccTheme.PHONE_TAP_MIN))


## `DccWidgets.toggle` lays a row out as label (expands), spacer (expands), switch. The two
## expanders SHARE the slack, so on a phone the label got half of it and "Commands: view, layer,
## parameter" was cut to "Commands: view, layer, para..." with room to spare. The spacer draws
## nothing and holds no minimum (its own header says so), so hiding it gives the label all the
## slack and moves the switch not at all: it stays at the row's right edge.
func _give_label_the_slack(cb: CheckBox) -> void:
	var row := cb.get_parent()
	if row != null and row.get_child_count() == 3 and row.get_child(1) is Control:
		(row.get_child(1) as Control).visible = false


## Lets a dropdown shrink to the width it is given. `OptionButton.fit_to_longest_item` (on by
## default) makes its MINIMUM width the longest item, so the provider picker ("Custom (Anthropic
## Messages)") forced this window's body wider than a 393 dp phone and pushed the dropdown's right
## half, and every switch below it, off the screen (measured on the phone pass). `DccShell.phone_fit`
## clears the same flag on dropdowns it meets, but runs once at open, before this window rebuilds.
func _shrinkable(ob: OptionButton) -> void:
	ob.fit_to_longest_item = false
	ob.clip_text = true
	ob.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS


## A labelled text field. On a phone the label sits above the field (a 393 dp column has no
## room for both side by side); elsewhere they share a row. `on_commit` receives the text on
## Enter and on focus loss. Returns the `LineEdit`.
func _field(parent: Control, label_text: String, value: String, placeholder: String,
		on_commit: Callable, secret: bool = false, tooltip: String = "") -> LineEdit:
	var box: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	box.add_theme_constant_override("separation", 4 if _phone else 8)
	box.tooltip_text = tooltip
	var l := DccTheme.label(label_text, "text_secondary", DccTheme.FS_SMALL)
	if not _phone:
		l.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	box.add_child(l)
	var le := LineEdit.new()
	le.text = value
	le.placeholder_text = placeholder
	le.secret = secret
	le.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	le.custom_minimum_size.y = _field_h()
	le.tooltip_text = tooltip
	DccWidgets.well(le)
	## A phone keyboard's Done and a desktop Enter both commit; so does leaving the field.
	le.text_submitted.connect(func(t: String): on_commit.call(t))
	le.focus_exited.connect(func(): on_commit.call(le.text))
	box.add_child(le)
	parent.add_child(box)
	return le


## An action button that can be disabled with a reason shown beside it.
func _button(parent: Control, text: String, on_press: Callable, primary: bool = false) -> Button:
	var b := DccWidgets.action(parent, text, on_press, primary)
	if _phone:
		b.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN
	return b


## Rebuilds the whole body from stored state. Clears the key field (it is part of the rebuild).
func _rebuild() -> void:
	_building = true
	_clear(_body)
	controls.clear()
	_dependent.clear()
	_presets = bridge.llm_presets()

	if not bridge.llm_api:
		_note(_body, "This build has no assistant support. Update the Cartalith library to use it.",
			"block")
		_building = false
		return

	_note(_body, "Optional. Everything here is off until you turn it on, and with it off nothing is sent anywhere. " +
		"A world generates the same with or without a model.", "text_secondary")
	var st := _status_line(_body)
	_status_dot = st[0]
	_status_text = st[1]
	controls["status"] = _status_text

	# -- Assistant switches ---------------------------------------------------
	_heading("ASSISTANT")
	for sw in _SWITCHES:
		_switch_row(sw)
	_heading("CONTEXT SENT WITH A REQUEST")
	_note(_body, "Nothing is sent unless ticked, per source. Whole notes, the whole Vault, the save file, file paths and the machine name are never sent unasked.")
	for sw in _CONTEXT:
		_switch_row(sw)
	_note(_body, "Search is used only when a box above is ticked. The app can search your request first and hand the model the best five excerpts (needed by models that never call a tool), and/or let the model search and read sources itself.", "text_secondary")
	var pre_idx := 0
	for i in _RAG_PRE.size():
		if _RAG_PRE[i][0] == String(DccSettings.llm_value("rag_pre")):
			pre_idx = i
	var pre_names: Array = []
	for o in _RAG_PRE:
		pre_names.append(String(o[1]))
	var pre := DccWidgets.choice(_body, "Search first", pre_names, pre_idx, _on_rag_pre_picked,
		"Automatic searches your request before the model sees it when the model is on this computer or your network, and not for a hosted model unless you allow it below.")
	_floor(pre)
	_shrinkable(pre)
	controls["rag_pre"] = pre
	_dependent.append(pre)
	_switch_row({"key": "rag_tools", "label": "Let the model search and read sources",
		"note": "Offers the model three read-only tools (search, read one source, list what is connected to it). At most 12 lookups per question; each result is at most 12 000 bytes."})
	var hosted_cb := DccWidgets.toggle(_body, "Allow knowledge search on this hosted endpoint", DccSettings.llm_rag_hosted_origins().has(String(_endpoint().get("origin", "-"))),
		func(on: bool): _on_rag_hosted(on), "A hosted model is run by a third party. Excerpts of your notes and your world's data are sent to it only if you allow it here, per endpoint.")
	_floor(hosted_cb, true)
	_give_label_the_slack(hosted_cb)
	controls["rag_hosted"] = hosted_cb
	controls["rag_hosted_note"] = _note(_body, "")
	_hide_if_empty(controls["rag_hosted_note"] as Label)
	_dependent.append(hosted_cb)

	# -- Provider -------------------------------------------------------------
	_heading("PROVIDER")
	var names: Array = ["None chosen"]
	var chosen := String(DccSettings.llm_value("preset_id"))
	var sel := 0
	for i in _presets.size():
		names.append(String(_presets[i]["name"]))
		if String(_presets[i]["id"]) == chosen:
			sel = i + 1
	var pick := DccWidgets.choice(_body, "Provider", names, sel, _on_preset_picked,
		"A provider only fills in defaults. Your address, model and key always win; the model is free text.")
	_floor(pick)
	_shrinkable(pick)
	controls["preset"] = pick
	var url := _field(_body, "Base URL", String(DccSettings.llm_value("base_url")),
		"choose a provider, or type an address", _on_url_commit, false,
		"Where requests go. http:// is allowed only for this computer or your network; a remote address must be https://.")
	controls["base_url"] = url
	## LLM-10: a phone keyboard autocorrects ordinary text fields. The 6T's SwiftKey turned
	## "qwen3.8-27b" into "qwen3. 8-27b" (a space after every full stop, 2026-10-26), which names
	## a model no server has. KEYBOARD_TYPE_URL asks for Android's URI input type, which keyboards treat as an
	## identifier rather than prose; the base URL, the model name and the variable name all are.
	url.virtual_keyboard_type = LineEdit.KEYBOARD_TYPE_URL
	_endpoint_text = _note(_body, "")
	controls["endpoint"] = _endpoint_text
	## The model: a dropdown filled from the server (`LlmModelList`), plus a typed fallback.
	var model_pick := DccWidgets.choice(_body, "Model", ["Choose a model…"], 0, _on_model_picked,
		"The models this server offers, asked when this window opens and when the address, key or a switch changes. Pick \"Type a name…\" to enter one by hand.")
	_floor(model_pick)
	_shrinkable(model_pick)
	controls["model_pick"] = model_pick
	var refresh := _button(model_pick.get_parent(), "Refresh", func(): _models_refresh(), false)
	refresh.tooltip_text = "Ask the server for its models again."
	controls["model_refresh"] = refresh
	controls["model_status"] = _note(_body, "", "text_secondary")
	var model := _field(_body, "Model name", String(DccSettings.llm_value("model")),
		"the model name your provider expects", _on_model_commit, false,
		"Type a model id exactly as your provider expects it. Always available, whatever the list says.")
	controls["model"] = model
	_model_field_row = model.get_parent()
	for w in [pick, url, model, model_pick, refresh]:
		_dependent.append(w)

	# -- Key ------------------------------------------------------------------
	_heading("KEY")
	var src_idx := 1 if String(DccSettings.llm_value("key_source")) == "store" else 0
	## LLM-10: on Android the OS store is the Android Keystore, and an app's environment cannot
	## be set on a phone, so both labels say so there (the ids stay `env` / `store`).
	var on_android := OS.has_feature("android")
	var env_label := String(_KEY_SOURCES[0][1]) + (" (cannot be set on a phone)" if on_android else "")
	var store_label := "Android Keystore" if on_android else String(_KEY_SOURCES[1][1])
	var src := DccWidgets.choice(_body, "Key source", [env_label,
		store_label + ("" if _store_available() else " (unavailable here)")],
		src_idx, _on_key_source_picked,
		"Where the key is read from. Cartalith never writes a key to a file.")
	if not _store_available():
		src.set_item_disabled(1, true)
	_floor(src)
	_shrinkable(src)
	controls["key_source"] = src
	_dependent.append(src)
	if src_idx == 0:
		var env := _field(_body, "Variable name", String(DccSettings.llm_value("env_var")),
			"CART_LLM_KEY", _on_env_commit, false,
			"The NAME of an environment variable that holds the key. Set it before starting Cartalith.")
		controls["env_var"] = env
		env.virtual_keyboard_type = LineEdit.KEYBOARD_TYPE_URL
		_dependent.append(env)
	else:
		var kf := _field(_body, "Key", "", "paste the key, then Save key", func(_t: String): pass, true,
			"Stored in the OS credential store, never in a file. It is cleared from this field when you press Save and is never shown again.")
		kf.secret = true
		## LLM-10: `secret` masks the drawn text only; the on-screen keyboard is told the field's
		## type separately. Without PASSWORD a phone keyboard treats the key as ordinary text: it
		## shows it in the suggestion strip and may learn it into its dictionary (the 6T's SwiftKey
		## showed the variable-name field's text in its strip, 2026-10-26). PASSWORD turns them off.
		kf.virtual_keyboard_type = LineEdit.KEYBOARD_TYPE_PASSWORD
		controls["key_field"] = kf
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)
		_body.add_child(row)
		var save := _button(row, "Save key", _on_key_save, true)
		var forget := _button(row, "Forget key", _on_key_forget)
		controls["key_save"] = save
		controls["key_forget"] = forget
		_dependent.append(kf)
		var has_provider := String(DccSettings.llm_value("preset_id")) != ""
		var can := _store_available() and has_provider
		save.disabled = not can
		forget.disabled = not can
		if not can:
			var why := "No OS credential store is available here. Use an environment variable; Cartalith never saves a key to a file." \
				if not _store_available() else "Choose a provider first: a saved key belongs to one provider."
			save.tooltip_text = why
			forget.tooltip_text = why
	var ks := _status_line(_body)
	_key_text = ks[1]
	_key_dot = ks[0]
	controls["key_status"] = _key_text
	_key_msg = _note(_body, "")
	controls["key_message"] = _key_msg

	# -- GPU (Ruling DF) ---------------------------------------------------------
	_gpu_box = VBoxContainer.new()
	_gpu_box.add_theme_constant_override("separation", 6)
	_body.add_child(_gpu_box)
	controls["gpu_box"] = _gpu_box
	var gh := DccTheme.mono_label("GPU WHILE A LOCAL PROVIDER IS ACTIVE", "text_dim", DccTheme.FS_HEADER, 1)
	DccWidgets.pad(_gpu_box, 0, 8, 0, 0).add_child(gh)
	var warn_card := PanelContainer.new()
	var sb := DccTheme.flat(DccTheme.c("sunken"), 8)
	sb.border_color = DccTheme.c("warn")
	sb.border_width_left = 3
	sb.content_margin_left = 10
	sb.content_margin_right = 10
	sb.content_margin_top = 8
	sb.content_margin_bottom = 8
	warn_card.add_theme_stylebox_override("panel", sb)
	_gpu_box.add_child(warn_card)
	var wc := VBoxContainer.new()
	wc.add_theme_constant_override("separation", 4)
	warn_card.add_child(wc)
	_gpu_headline = DccTheme.label("", "text", DccTheme.FS_SMALL)
	_gpu_headline.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	wc.add_child(_gpu_headline)
	_gpu_detected = DccTheme.label("", "text_bright", DccTheme.FS_SMALL)
	_gpu_detected.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	wc.add_child(_gpu_detected)
	_gpu_effect = DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	_gpu_effect.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	wc.add_child(_gpu_effect)
	controls["gpu_headline"] = _gpu_headline
	controls["gpu_detected"] = _gpu_detected
	controls["gpu_effect"] = _gpu_effect
	var ov_idx := 0
	for i in _GPU_OVERRIDES.size():
		if _GPU_OVERRIDES[i][0] == String(DccSettings.llm_value("gpu_override")):
			ov_idx = i
	var ov_names: Array = []
	for o in _GPU_OVERRIDES:
		ov_names.append(String(o[1]))
	var ov := DccWidgets.choice(_gpu_box, "GPU override", ov_names, ov_idx, _on_gpu_override_picked,
		"Automatic switches only when the model is on this computer. Always switches whatever the address says (for Docker, WSL, a tunnel). Never leaves your GPU choice alone.")
	_floor(ov)
	_shrinkable(ov)
	controls["gpu_override"] = ov
	## LLM-10: a phone has one GPU, so the override has nothing to choose between and its row
	## is not shown; the plan's effect line says why (`LlmGpuRule.is_phone`).
	if LlmGpuRule.is_phone():
		ov.get_parent().visible = false

	# -- Cost -------------------------------------------------------------------
	_heading("COST")
	var tc := DccWidgets.toggle(_body, "Token counter visible", bool(DccSettings.llm_value("token_counter")),
		func(on: bool): _store("token_counter", on),
		"Shows how many tokens a session has used. It is a readout, not a limit.")
	_floor(tc, true)
	_give_label_the_slack(tc)
	controls["token_counter"] = tc
	var cap := DccWidgets.number(_body, "Token cap (0 = off)", 0, 1000000000, 1000,
		float(int(DccSettings.llm_value("token_cap"))),
		func(v: float): _store("token_cap", int(v)),
		"Optional. When the tokens servers have reported this session reach this number, Send is refused until you raise it, set it to 0, or reset the counter. 0 means no cap, which is the default.")
	_floor(cap)
	controls["token_cap"] = cap
	_note(_body, "No spend limit is enforced unless you set a cap above. A cap counts only what servers report, so a server that reports nothing is not counted. A hosted provider bills you directly.")

	# -- Consent ----------------------------------------------------------------
	_heading("CONSENT")
	_consent_text = _note(_body, "", "text_secondary")
	controls["consent_state"] = _consent_text
	var crow := HBoxContainer.new()
	crow.add_theme_constant_override("separation", 8)
	_body.add_child(crow)
	var review := _button(crow, "Review consent…", _on_review_consent)
	var withdraw := _button(crow, "Withdraw all consent", _on_withdraw_consent)
	controls["consent_review"] = review
	controls["consent_withdraw"] = withdraw

	_building = false
	_refresh_dynamic()
	_models_render()


## A switch's sub-note: its `note_phone` on Android where it has one (LLM-10: "this computer" is
## not where a phone user is), else its `note`, else "".
func _sw_note(sw: Dictionary) -> String:
	if OS.has_feature("android") and sw.has("note_phone"):
		return String(sw["note_phone"])
	return String(sw.get("note", ""))


## One switch row plus its sub-note. A switch that cannot be used in this build is shown off
## and disabled, with the reason as its note (never a dead control without a reason).
func _switch_row(sw: Dictionary) -> void:
	var key := String(sw["key"])
	var unavailable := String(sw.get("unavailable", ""))
	var value := bool(DccSettings.llm_value(key)) if unavailable == "" else false
	## Phone only (owner finding 2026-10-26, OnePlus 6T: "large blank gaps under the toggle rows"). The switch
	## row is floored to 44 dp, its label is centred in it, and the body adds 8 dp before the note, so the note
	## sat about 20 dp under the label it explains and looked like it belonged to the next row. On a phone the
	## row and its note go in one pair box whose separation is NEGATIVE (-6 dp): the note rides up into the
	## row's bottom padding. That strip is only padding, and a `Label` takes no mouse events, so the 44 dp
	## touch target and the tap on its lower edge are unchanged. Other forms keep the plain body spacing.
	var host: Control = _body
	if _phone:
		var pair := VBoxContainer.new()
		pair.add_theme_constant_override("separation", -6)
		_body.add_child(pair)
		host = pair
	var cb := DccWidgets.toggle(host, String(sw["label"]), value, func(on: bool): _on_switch(key, on),
		_sw_note(sw) if unavailable == "" else unavailable)
	_floor(cb, true)
	_give_label_the_slack(cb)
	controls[key] = cb
	var note := _note(host, unavailable if unavailable != "" else _sw_note(sw))
	controls[key + "_note"] = note
	_hide_if_empty(note)
	if unavailable != "":
		cb.disabled = true
	elif key != "enabled":
		_dependent.append(cb)


# ---------------------------------------------------------------------------
# State helpers
# ---------------------------------------------------------------------------

## Whether an OS credential store exists here (`llm_key_status`'s `store_available`).
func _store_available() -> bool:
	var r: Dictionary = bridge.llm_key_status("env", "X", "x")
	return bool(r.get("store_available", false))


## The picker row for the stored provider, or `{}` for none/unknown.
func _preset_row() -> Dictionary:
	var id := String(DccSettings.llm_value("preset_id"))
	for p in _presets:
		if String(p["id"]) == id:
			return p
	return {}


## `llm_endpoint_info` for the stored provider and address; `ok` false if unusable.
func _endpoint() -> Dictionary:
	return bridge.llm_endpoint_info(String(DccSettings.llm_value("preset_id")),
		String(DccSettings.llm_value("base_url")))


## The gate's answer for the status line (no capability).
func _gate() -> Dictionary:
	return bridge.llm_authorize(DccSettings.llm_gate_dict(), "", DccSettings.llm_consented_origins())


## Writes one `[llm]` value. Ignored while `_rebuild()` is filling controls.
func _store(key: String, value) -> void:
	if _building:
		return
	DccSettings.set_llm_value(key, value)


# ---------------------------------------------------------------------------
# Dynamic state
# ---------------------------------------------------------------------------

var _key_dot: Label

## Re-reads stored state and updates every label and enabled-state without rebuilding the
## controls. Safe to call while a field has focus.
func _refresh_dynamic() -> void:
	if _status_text == null:
		return
	var enabled := bool(DccSettings.llm_value("enabled"))
	# Switch rows depend on the master switch.
	for w in _dependent:
		if is_instance_valid(w):
			if w is BaseButton:
				(w as BaseButton).disabled = not enabled
			elif w is LineEdit:
				(w as LineEdit).editable = enabled
	for sw in _SWITCHES:
		var k := String(sw["key"])
		if k == "enabled" or not controls.has(k + "_note") or String(sw.get("unavailable", "")) != "":
			continue
		(controls[k + "_note"] as Label).text = _sw_note(sw) + ("" if enabled else " Needs the master switch.")
		_hide_if_empty(controls[k + "_note"] as Label)
	## The hosted opt-in applies to a hosted endpoint only (K2); a local one never needs it.
	if controls.has("rag_hosted"):
		var hep := _endpoint()
		var hosted := bool(hep.get("ok", false)) and not bool(hep.get("is_local", true))
		var cb := controls["rag_hosted"] as BaseButton
		cb.disabled = not enabled or not hosted
		cb.set_pressed_no_signal(hosted and DccSettings.llm_rag_hosted_origins().has(String(hep.get("origin", "-"))))
		(controls["rag_hosted_note"] as Label).text = "" if hosted else "Not needed: this endpoint is on this computer or your network."
		_hide_if_empty(controls["rag_hosted_note"] as Label)
	# Key-entry controls follow the master switch, but also the store/provider conditions.
	if controls.has("key_save"):
		var can := enabled and _store_available() and String(DccSettings.llm_value("preset_id")) != ""
		(controls["key_save"] as Button).disabled = not can
		(controls["key_forget"] as Button).disabled = not can
	# Status line: the gate's own answer.
	var g := _gate()
	if bool(g.get("ok", false)):
		_status_text.text = "Ready: requests go to %s (%s)." % [g["origin"], g["class_label"]]
		_dot(_status_dot, "good")
	else:
		_status_text.text = String(g.get("reason", "Unavailable."))
		_dot(_status_dot, "text_ghost" if String(g.get("code", "")) == "feature_off" else "warn")
	# Endpoint line.
	var ep := _endpoint()
	if bool(ep.get("ok", false)):
		_endpoint_text.text = "Endpoint: %s, detected as %s." % [ep["origin"], ep["class_label"]]
	else:
		var u := String(DccSettings.llm_value("base_url"))
		_endpoint_text.text = "No address yet." if u == "" else "This address cannot be used: " + String(ep.get("error", ""))
	# Key line.
	_refresh_key_line()
	# GPU group.
	var plan := LlmGpuRule.current_plan(bridge)
	var show := bool(plan.get("warn", false))
	_gpu_box.visible = show
	if show:
		_gpu_headline.text = String(plan["headline"])
		_gpu_detected.text = String(plan["detected"])
		_gpu_effect.text = String(plan["effect"])
	# Consent line.
	var origin := String(ep.get("origin", ""))
	var given := origin != "" and DccSettings.llm_consented_origins().has(origin)
	(controls["consent_review"] as Button).disabled = origin == ""
	(controls["consent_review"] as Button).tooltip_text = "" if origin != "" else \
		"Choose a provider and a valid address first: consent is for one endpoint."
	(controls["consent_withdraw"] as Button).disabled = DccSettings.llm_consented_origins().is_empty()
	if origin == "":
		_consent_text.text = "Consent is recorded per endpoint. Choose a provider and a valid address to review it."
	elif given:
		_consent_text.text = "Consent given for %s." % origin
	else:
		_consent_text.text = "No consent recorded for %s. The first request asks first; you can review it now." % origin


## The key status line: present + masked, or the named problem. Never the key.
func _refresh_key_line() -> void:
	var src := String(DccSettings.llm_value("key_source"))
	var r: Dictionary = bridge.llm_key_status(src, String(DccSettings.llm_value("env_var")),
		String(DccSettings.llm_value("preset_id")))
	if not bool(r.get("ok", false)):
		_key_text.text = String(r.get("error", ""))
		_dot(_key_dot, "block")
	elif bool(r.get("present", false)):
		_key_text.text = "Key present, %s. It is never shown, saved to a file, or exported." % String(r["masked"])
		_dot(_key_dot, "good")
	else:
		var need := bool(_endpoint().get("key_required", false))
		_key_text.text = String(r.get("problem", "")) + ("" if need else " (This endpoint does not need a key.)")
		_dot(_key_dot, "warn" if need else "text_ghost")


# ---------------------------------------------------------------------------
# Handlers
# ---------------------------------------------------------------------------

## After any stored change: resync the GPU rule and the labels, and tell listeners.
func _after_change() -> void:
	LlmGpuRule.sync(bridge)
	_refresh_dynamic()
	settings_changed.emit()


## A switch toggled. The master switch asks for consent first when an endpoint is ready and
## not yet consented; declining puts it back off.
func _on_switch(key: String, on: bool) -> void:
	if _building:
		return
	if key == "enabled" and on:
		var ep := _endpoint()
		var origin := String(ep.get("origin", ""))
		if origin != "" and not DccSettings.llm_consented_origins().has(origin):
			_ask_consent(ep, func():
				_store("enabled", true)
				_after_change()
				_models_refresh(),
				func():
					_store("enabled", false)
					_rebuild_deferred())
			return
	_store(key, on)
	if key == "enabled":
		_rebuild_deferred()
	_after_change()
	## The gate's own switches decide whether the list may be asked for at all.
	if key in ["enabled", "local_allowed", "remote_allowed"]:
		_models_refresh()


## The "Search first" picker (RK-6, K6).
func _on_rag_pre_picked(idx: int) -> void:
	if idx >= 0 and idx < _RAG_PRE.size():
		_store("rag_pre", String(_RAG_PRE[idx][0]))
		_after_change()


## The per-endpoint hosted opt-in (K2). A local endpoint needs none, so the toggle is disabled there.
func _on_rag_hosted(on: bool) -> void:
	if _building:
		return
	var ep := _endpoint()
	if bool(ep.get("ok", false)) and not bool(ep.get("is_local", true)):
		DccSettings.llm_set_rag_hosted(String(ep["origin"]), on)
	_after_change()


## Asks for consent for `ep`'s endpoint through the shared card.
func _ask_consent(ep: Dictionary, on_agree: Callable, on_decline: Callable = Callable()) -> void:
	last_consent = LlmConsent.present(app, ep, _preset_row(), on_agree, on_decline)


## Rebuilds after the current signal returns (the emitting control must not free itself).
func _rebuild_deferred() -> void:
	_rebuild.call_deferred()


## A provider picked: store its id, fill its default address (the user's later edit wins),
## then rebuild so provider-specific rows show.
func _on_preset_picked(index: int) -> void:
	if _building:
		return
	var id := ""
	var url := ""
	if index > 0 and index - 1 < _presets.size():
		id = String(_presets[index - 1]["id"])
		url = String(_presets[index - 1]["base_url"])
	_store("preset_id", id)
	_store("base_url", url)
	_after_change()
	_models_refresh()
	_rebuild_deferred()


## The address committed (Enter or focus loss).
func _on_url_commit(text: String) -> void:
	if _building or text.strip_edges() == String(DccSettings.llm_value("base_url")):
		return
	_store("base_url", text)
	_after_change()
	_models_refresh()


## The model name committed (typed by hand).
func _on_model_commit(text: String) -> void:
	if _building or text.strip_edges() == String(DccSettings.llm_value("model")):
		return
	_store("model", text)
	_after_change()
	_models_render()


# ---------------------------------------------------------------------------
# The model dropdown (the owner's 2026-10-26 request)
# ---------------------------------------------------------------------------

## The dropdown's metadata marker for the "Type a name…" row. Not a model id (it holds a control
## character, which the parser drops from every id), and that row is never stored as a model.
const _TYPE_A_NAME := "\u0001type-a-name"

## Asks the configured server for its models. Called when the window opens, when the address, the
## key, the provider or a gate switch changes, and by the Refresh button; never on a timer. The gate
## decides first (`LlmModels.begin`): a refusal is drawn as its reason and nothing is dialled.
func _models_refresh() -> void:
	if _ml == null:
		return
	var r := _ml.refresh(DccSettings.llm_gate_dict(), DccSettings.llm_consented_origins())
	if bool(r.get("ok", false)):
		_ml_state = "loading"
		_ml_msg = ""
		_ml_code = ""
	else:
		_ml_state = "refused"
		_ml_code = String(r.get("code", ""))
		_ml_msg = String(r.get("reason", "Models cannot be listed."))
	_models_render()


## A listing finished (the wire is joined). A result for an endpoint the user has since left is dropped.
func _on_models_finished(result: Dictionary) -> void:
	if String(result.get("key", "")) != LlmModelList.key_for(DccSettings.llm_gate_dict()):
		return
	if bool(result.get("ok", false)):
		_ml_state = "ok"
		_ml_msg = ""
		_ml_code = ""
	else:
		_ml_state = "failed"
		_ml_code = String(result.get("code", ""))
		_ml_msg = String(result.get("reason", ""))
	_models_render()


## "just now", "12 s ago", "3 min ago", "2 h ago".
static func _age_text(seconds: float) -> String:
	if seconds < 5.0:
		return "just now"
	if seconds < 60.0:
		return "%d s ago" % int(seconds)
	if seconds < 3600.0:
		return "%d min ago" % int(seconds / 60.0)
	return "%d h ago" % int(seconds / 3600.0)


## The last good list for the current endpoint (memory only), or `{}`.
func _ml_cached() -> Dictionary:
	return LlmModelList.last_for(LlmModelList.key_for(DccSettings.llm_gate_dict()))


## Draws the dropdown, the typed-name field and the status line from stored state. Safe to call at
## any time, including while the controls were just rebuilt or are about to be freed.
func _models_render() -> void:
	if not controls.has("model_pick") or not is_instance_valid(controls["model_pick"]):
		return
	var ob := controls["model_pick"] as OptionButton
	var saved := String(DccSettings.llm_value("model")).strip_edges()
	var cached := _ml_cached()
	var models: Array = cached.get("models", [])
	var was_building := _building
	_building = true
	ob.clear()
	_ml_rows.clear()
	var sel := -1
	var in_list := false
	for m in models:
		if String(m["id"]) == saved:
			in_list = true
	if saved == "":
		ob.add_item("Choose a model…")
		ob.set_item_metadata(0, "")
		_ml_rows.append({"id": "", "text": "Choose a model…"})
		sel = 0
	elif not in_list:
		var t0 := saved + (" (not in list)" if not models.is_empty() else "")
		ob.add_item(t0)
		ob.set_item_metadata(0, saved)
		_ml_rows.append({"id": saved, "text": t0})
		sel = 0
	for m in models:
		var t := LlmModelList.item_text(m)
		ob.add_item(t)
		var i := ob.item_count - 1
		ob.set_item_metadata(i, String(m["id"]))
		_ml_rows.append({"id": String(m["id"]), "text": t})
		if String(m["id"]) == saved:
			sel = i
	ob.add_item("Type a name…")
	ob.set_item_metadata(ob.item_count - 1, _TYPE_A_NAME)
	ob.select(sel if sel >= 0 else 0)
	_building = was_building
	## The typed field is shown while the user chose "Type a name…" and whenever there is no list to pick from.
	if _model_field_row != null and is_instance_valid(_model_field_row):
		_model_field_row.visible = _ml_manual or models.is_empty()
	var field := controls.get("model") as LineEdit
	if field != null and is_instance_valid(field) and not field.has_focus() and field.text != saved:
		field.text = saved
	_models_render_status()


## Only the status line (the age text changes every second without a rebuild).
func _models_render_status() -> void:
	if not controls.has("model_status") or not is_instance_valid(controls["model_status"]):
		return
	var cached := _ml_cached()
	var n := (cached.get("models", []) as Array).size()
	var stale := ""
	if not cached.is_empty():
		stale = "the list from %s" % _age_text(float(cached["age_s"]))
	var txt := ""
	match _ml_state:
		"loading":
			txt = "Listing models…" + ((" Showing " + stale + " meanwhile.") if stale != "" else "")
		"refused":
			txt = "Not listed. " + _ml_msg
		"failed":
			txt = "Could not list: " + _ml_msg + ((" Showing " + stale + ".") if stale != "" else " You can type a name below.")
		"ok":
			if n == 0:
				txt = "The server answered but lists no chat models. Load or download one, then press Refresh, or type a name below."
			else:
				txt = "%d model%s, listed %s." % [n, "" if n == 1 else "s", _age_text(float(cached.get("age_s", 0.0)))]
				if int(cached.get("hidden", 0)) > 0:
					txt += " %d embedding model%s hidden." % [int(cached["hidden"]), "" if int(cached["hidden"]) == 1 else "s"]
				if bool(cached.get("truncated", false)):
					txt += " Only the first 500 are shown."
		_:
			txt = "" if cached.is_empty() else "%d models, listed %s." % [n, _age_text(float(cached["age_s"]))]
	(controls["model_status"] as Label).text = txt
	var rb := controls.get("model_refresh") as Button
	if rb != null and is_instance_valid(rb):
		rb.text = "Listing…" if _ml_state == "loading" else "Refresh"


## A dropdown row chosen: a model is stored and made the one chat uses; "Type a name…" shows the field.
func _on_model_picked(index: int) -> void:
	if _building or not controls.has("model_pick"):
		return
	var ob := controls["model_pick"] as OptionButton
	var meta := String(ob.get_item_metadata(index))
	if meta == _TYPE_A_NAME:
		_ml_manual = true
		_models_render()
		var f := controls.get("model") as LineEdit
		if f != null:
			f.grab_focus()
		return
	if meta == "":
		return
	_ml_manual = false
	_store("model", meta)
	_after_change()
	_models_render()


## Re-ticks the status line's age once a second while the window is shown. Starts nothing.
func _process(delta: float) -> void:
	if not visible:
		return
	_ml_tick += delta
	if _ml_tick >= 1.0:
		_ml_tick = 0.0
		_models_render_status()


## The environment variable NAME committed.
func _on_env_commit(text: String) -> void:
	if _building or text.strip_edges() == String(DccSettings.llm_value("env_var")):
		return
	_store("env_var", text if text.strip_edges() != "" else String(DccSettings.LLM_DEFAULTS["env_var"]))
	_after_change()
	_models_refresh()


## Key source picked: `env` or `store`.
func _on_key_source_picked(index: int) -> void:
	if _building:
		return
	_store("key_source", String(_KEY_SOURCES[index][0]))
	_after_change()
	_models_refresh()
	_rebuild_deferred()


## GPU override picked.
func _on_gpu_override_picked(index: int) -> void:
	if _building:
		return
	_store("gpu_override", String(_GPU_OVERRIDES[index][0]))
	_after_change()


## Save key: hands the field's text to the OS credential store and clears the field at once,
## whatever the outcome. The text is held in a local only for the one call.
func _on_key_save() -> void:
	var field := controls.get("key_field") as LineEdit
	if field == null:
		return
	var typed := field.text
	field.text = ""
	var r := bridge.llm_key_save(String(DccSettings.llm_value("preset_id")), typed)
	typed = ""
	_key_msg.text = "Key saved in the OS credential store." if bool(r.get("ok", false)) else String(r.get("error", "Not saved."))
	_refresh_dynamic()
	## The Assistant chip's greyed/live state depends on whether a key exists (CQ-7).
	DccSettings.notify_assistant_state()
	settings_changed.emit()
	## A new or removed key changes what the server will answer: ask again.
	_models_refresh()


## Forget key: removes the saved key for the chosen provider.
func _on_key_forget() -> void:
	var r := bridge.llm_key_forget(String(DccSettings.llm_value("preset_id")))
	_key_msg.text = "Saved key removed." if bool(r.get("ok", false)) else String(r.get("error", "Not removed."))
	_refresh_dynamic()
	## The Assistant chip's greyed/live state depends on whether a key exists (CQ-7).
	DccSettings.notify_assistant_state()
	settings_changed.emit()
	## A new or removed key changes what the server will answer: ask again.
	_models_refresh()


## Review consent: shows the card for the current endpoint (agreeing records it).
func _on_review_consent() -> void:
	var ep := _endpoint()
	if not bool(ep.get("ok", false)):
		return
	_ask_consent(ep, func():
		_refresh_dynamic()
		settings_changed.emit()
		_models_refresh())


## Withdraw every recorded consent; the next request asks again.
func _on_withdraw_consent() -> void:
	DccSettings.llm_withdraw_consent()
	_refresh_dynamic()
	settings_changed.emit()
