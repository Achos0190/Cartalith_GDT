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
## True while the first-run network note is on screen; closing the window then records it as seen.
var _net_note_shown := false
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
	{"key": "enabled", "label": "Assistant enabled", "note": "Off means nothing is ever sent."},
	{"key": "prose", "label": "Prose and Vault assist",
		"note": "Draft text for a place, its story or a note. Nothing is saved until you click."},
	{"key": "commands", "label": "Commands: view, layer, parameter",
		"note": "The assistant suggests settings, view and layer changes. You review each one and press Apply; one undo step reverses it."},
	{"key": "regen_on_apply", "label": "Regenerate when I apply a proposal",
		"note": "Applying a settings proposal also rebuilds the world with the new settings (asking first if you have hand edits that a rebuild would discard). Turn off to only change the settings and press Generate yourself."},
	{"key": "edit", "label": "Edit actions: raise, lower, smooth",
		"note": "The model proposes terrain edits to an area you paint or box on the map (Assistant > Edit). You see a preview computed by the app; nothing changes until you press Apply, and one Apply is one undo step. It never generates, erodes or changes settings. Only raise, lower and smooth in this release; painting colours comes later. Needs the region source below, and goes only to a model on this computer or your network."},
	{"key": "local_allowed", "label": "Local providers allowed", "note": "Models on this computer or your network.",
		"note_phone": "Models on this phone or your network."},
	{"key": "remote_allowed", "label": "Remote providers allowed", "note": "Models run by a company on the internet."},
]
## The context sources (scope s.5: "none; per source"), each its own switch.
const _CONTEXT := [
	{"key": "ctx_entity", "label": "Send: current entity"},
	{"key": "ctx_selection", "label": "Send: selection"},
	{"key": "ctx_notes", "label": "Send: named Vault notes"},
	## Ruling DI P3: generation parameters are their own source, off until ticked. A setting
	## proposal shows the model each offered setting's current value, so it needs this.
	## LLM-7 (scope s.5c, owner question E3's default): an edit request shows the model the region you
	## painted and its measured heights, so this is its own box, off until ticked.
	{"key": "ctx_region", "label": "Send: the painted region and its heights",
		"note": "Needed for edit actions: the model is shown the region's shape (a few rows of x ranges), its bounding box, cell count and area, and its lowest, mean and highest height, land share and mean slope, as numbers. Never a picture, never the rest of the map. Edit requests go only to a model on this computer or your local network."},
	{"key": "ctx_params", "label": "Send: generation parameters and summary statistics",
		"note": "Lets the assistant see your current settings and world-wide figures such as sea level and rainfall. It only reads."},
	## RK-5 (Rulings DM and DN K1): the knowledge base's two sources, each its own box, both off.
	{"key": "ctx_rag_vault", "label": "Send: Vault notes found by search",
		"note": "Short excerpts of notes that match your request. Notes marked private, dm-only or secret, or kept in a DM, Private or Secrets folder, are never sent."},
	{"key": "ctx_rag_world", "label": "Send: world data found by search",
		"note": "World facts that match your request (names, populations, trade, faiths, history). World data has no private marker, so everything found is sent."},
]
## RK-5/RK-6 (K6): how the knowledge base reaches the model. Shown under the context sources.
const _RAG_PRE := [
	["auto", "Search first (automatic for local models)"],
	["on", "Search first (always)"],
	["off", "Do not search first"],
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
	## A profile was applied (here, in the panel or from the chip's menu): redraw from the live values.
	LlmProfiles.hub.switched.connect(_on_profile_switched)
	## Only ticks the age text of a list that is already shown once a second; starts nothing.
	set_process(true)
	## Closing must not leave a key in a field. `_rebuild` clears it on open too.
	## 2026-10-26: a field commits on Enter or on focus loss, and on a phone neither is guaranteed (the soft
	## keyboard's Done may be dismissed by Back, and a window that closes does not always move focus first), so
	## text typed into the address, model and variable-name fields and then "closed over" was lost. Closing
	## commits whatever the fields hold (each commit is a no-op when the text equals the stored value).
	visibility_changed.connect(func():
		if not visible:
			_commit_pending_fields()
		if not visible and _net_note_shown:
			_net_note_shown = false
			DccSettings.set_llm_value("net_hint_seen", true)
		if not visible and controls.has("key_field") and is_instance_valid(controls["key_field"]):
			(controls["key_field"] as LineEdit).text = "")


## Commits the text currently held by the address, model and variable-name fields. Safe to call at any time:
## The address goes LAST: its handler asks for the model list, which can rebuild the body and replace the other fields. `_building` guards a rebuild, and each handler returns early when the text equals the stored value.
func _commit_pending_fields() -> void:
	if _building:
		return
	for pair in [["model", _on_model_commit], ["env_var", _on_env_commit], ["base_url", _on_url_commit]]:
		var f = controls.get(pair[0])
		if f is LineEdit and is_instance_valid(f):
			(pair[1] as Callable).call((f as LineEdit).text)


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
		_note(_body, "The assistant is not available in this version of Cartalith.",
			"block")
		_building = false
		return

	_note(_body, "Optional. Everything is off until you turn it on. Worlds generate the same with or without an assistant.", "text_secondary")
	var st := _status_line(_body)
	_status_dot = st[0]
	_status_text = st[1]
	controls["status"] = _status_text

	# -- Profiles (Ruling DQ) ---------------------------------------------------
	_build_profiles()

	# -- Assistant switches ---------------------------------------------------
	_heading("ASSISTANT")
	for sw in _SWITCHES:
		_switch_row(sw)
	_heading("CONTEXT SENT WITH A REQUEST")
	_note(_body, "Only ticked sources are sent. Whole notes, your whole Vault, save files, file paths and your computer name are never sent unasked.")
	for sw in _CONTEXT:
		_switch_row(sw)
	_note(_body, "Used only when a search box above is ticked. Cartalith can search for you first, or let the assistant search itself.", "text_secondary")
	var pre_idx := 0
	for i in _RAG_PRE.size():
		if _RAG_PRE[i][0] == String(DccSettings.llm_value("rag_pre")):
			pre_idx = i
	var pre_names: Array = []
	for o in _RAG_PRE:
		pre_names.append(String(o[1]))
	var pre := DccWidgets.choice(_body, "Search first", pre_names, pre_idx, _on_rag_pre_picked,
		"Automatic searches first for models on this computer or network, and for hosted models only if you allow it below.")
	_floor(pre)
	_shrinkable(pre)
	controls["rag_pre"] = pre
	_dependent.append(pre)
	_switch_row({"key": "rag_tools", "label": "Let the model search and read sources",
		"note": "The assistant can look things up itself, read-only, up to 12 times per question."})
	var hosted_cb := DccWidgets.toggle(_body, "Allow knowledge search on this hosted endpoint", DccSettings.llm_rag_hosted_origins().has(String(_endpoint().get("origin", "-"))),
		func(on: bool): _on_rag_hosted(on), "Hosted models are run by a third party. Excerpts of your notes and world are sent only if you allow it, per service.")
	_floor(hosted_cb, true)
	_give_label_the_slack(hosted_cb)
	controls["rag_hosted"] = hosted_cb
	controls["rag_hosted_note"] = _note(_body, "")
	_hide_if_empty(controls["rag_hosted_note"] as Label)
	_dependent.append(hosted_cb)

	# -- Provider -------------------------------------------------------------
	_heading("PROVIDER")
	_note(_body, "Setting up a provider, address, model and key sends nothing. Requests start only when Assistant enabled is on.")
	var names: Array = ["None chosen"]
	var chosen := String(DccSettings.llm_value("preset_id"))
	var sel := 0
	for i in _presets.size():
		names.append(String(_presets[i]["name"]))
		if String(_presets[i]["id"]) == chosen:
			sel = i + 1
	var pick := DccWidgets.choice(_body, "Provider", names, sel, _on_preset_picked,
		"A provider fills in defaults. Your own address, model and key always win.")
	_floor(pick)
	_shrinkable(pick)
	controls["preset"] = pick
	var url := _field(_body, "Base URL", String(DccSettings.llm_value("base_url")),
		"choose a provider, or type an address", _on_url_commit, false,
		"Where requests go. Use http:// only for this computer or your network; other addresses need https://.")
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
		"The models this server offers. Pick \"Type a name…\" to enter one yourself.")
	_floor(model_pick)
	_shrinkable(model_pick)
	controls["model_pick"] = model_pick
	var refresh := _button(model_pick.get_parent(), "Refresh", func(): _models_refresh(true), false)
	refresh.tooltip_text = "Ask the server for its models again."
	controls["model_refresh"] = refresh
	controls["model_status"] = _note(_body, "", "text_secondary")
	## Shown only while the list was refused because the master switch is off: one press turns it on.
	var enable_btn := _button(_body, "Turn on Assistant enabled", func(): _enable_assistant(), true)
	enable_btn.visible = false
	controls["model_enable"] = enable_btn
	## Android only (`LlmNetHint`): the way to the system page where LineageOS can block the app's network.
	if LlmNetHint.is_android():
		controls["net_settings"] = LlmNetHint.make_button(_body, DccTheme.PHONE_TAP_MIN if _phone else 0.0)
	var model := _field(_body, "Model name", String(DccSettings.llm_value("model")),
		"the model name your provider expects", _on_model_commit, false,
		"Type the model name exactly as your provider spells it.")
	controls["model"] = model
	_model_field_row = model.get_parent()
	## 2026-10-26 owner report ("I can't seem to save the LLM on my phone"): the provider, address, model and key
	## controls used to follow the master switch, so with it off (the shipped default, and the state an earlier
	## test left on the owner's handset) every one of them was greyed and READ-ONLY with nothing saying why: the
	## address and key could not be typed, so nothing could be saved. Setting them up sends nothing -- the gate
	## (`_gate()`) still refuses every request until the master switch is on -- so they no longer follow it. The
	## switches that choose WHAT may be sent still do (`_switch_row`, `rag_pre`, the hosted opt-in).

	# -- Reply: thinking, cap and retries (part of a profile, Ruling DQ) -------------------
	_build_reply()

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
		"Where the key comes from. Cartalith never saves a key to a file.")
	if not _store_available():
		src.set_item_disabled(1, true)
	_floor(src)
	_shrinkable(src)
	controls["key_source"] = src
	if src_idx == 0:
		var env := _field(_body, "Variable name", String(DccSettings.llm_value("env_var")),
			"CART_LLM_KEY", _on_env_commit, false,
			"The name of the environment variable that holds the key. Set it before starting Cartalith.")
		controls["env_var"] = env
		env.virtual_keyboard_type = LineEdit.KEYBOARD_TYPE_URL
	else:
		var kf := _field(_body, "Key", "", "paste the key, then Save key", func(_t: String): pass, true,
			"Kept in your system credential store, never in a file. The field clears when you press Save and the key is never shown again.")
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
		var has_provider := String(DccSettings.llm_value("preset_id")) != ""
		var can := _store_available() and has_provider
		save.disabled = not can
		forget.disabled = not can
		if not can:
			var why := "No OS credential store is available here. Use an environment variable; Cartalith never saves a key to a file." \
				if not _store_available() else "Choose a provider first: a saved key belongs to one profile."
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
		"Automatic switches only for models on this computer. Always switches for any address (Docker, WSL, tunnels). Never leaves your GPU alone.")
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
		"How many tokens this session has used. Not a limit.")
	_floor(tc, true)
	_give_label_the_slack(tc)
	controls["token_counter"] = tc
	var cap := DccWidgets.number(_body, "Token cap (0 = off)", 0, 1000000000, 1000,
		float(int(DccSettings.llm_value("token_cap"))),
		func(v: float): _store("token_cap", int(v)),
		"Optional. Send stops when this session reaches this many tokens. 0 means no cap.")
	_floor(cap)
	controls["token_cap"] = cap
	_note(_body, "No spend limit is enforced unless you set a cap above. A cap counts only what the server reports. A hosted provider bills you directly.")

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


## Why Edit actions cannot be turned on in this build, or "" when they can. The grouped undo
## (`undo_group.gd`: `begin`, `commit`, `abort`) must exist, and so must the engine's three region
## functions; a library older than the shell lacks them and says so.
func _edit_unavailable() -> String:
	var ug = app.get("undo_groups") if app != null else null
	if ug == null or not (ug.has_method("begin") and ug.has_method("commit") and ug.has_method("abort")):
		return "Not available in this build: edit actions need the grouped undo, which is missing."
	if not (bridge.has_method("region_describe") and bridge.has_method("region_edit_preview") and bridge.has_method("region_edit_apply")):
		return "Not available in this build: the shell has no region editing."
	if not (bridge._has("region_describe") and bridge._has("region_edit_preview") and bridge._has("region_edit_apply")):
		return "Not available: the Cartalith library is older than the shell and cannot edit a region. Update it."
	return ""


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
	## LLM-7 (scope LLM-7 bar: "refuses to enable unless LLM-5's grouped undo is present in the build"):
	## the Edit switch is available only when this build has the grouped undo AND the three region
	## functions. Presence is checked at run time (the tests that it WORKS are the probes and the
	## cargo suite, which this cannot run), and the reason is shown, never a dead switch.
	if key == "edit" and unavailable == "":
		unavailable = _edit_unavailable()
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
	## First time the Assistant is on, on Android: one line and the button (shown until the window closes once).
	if key == "enabled" and value and LlmNetHint.is_android() and not bool(DccSettings.llm_value("net_hint_seen")):
		_note(host, LlmNetHint.FIRST_RUN_NOTE, "text_secondary")
		LlmNetHint.make_button(host, DccTheme.PHONE_TAP_MIN if _phone else 0.0)
		_net_note_shown = true


# ---------------------------------------------------------------------------
# Profiles (Ruling DQ, LLM_ASSISTANT_SCOPE.md s.5g)
# ---------------------------------------------------------------------------

## The profile line under the buttons: the last refusal or confirmation, in words.
var _profile_msg: Label

## The PROFILE group: the switch, what a profile is, and the list operations. The switch applies a
## whole profile at once (`LlmProfiles.switch_to`); the buttons edit the LIST, never the live values.
func _build_profiles() -> void:
	_heading("PROFILE")
	var sw := LlmProfileSwitch.new()
	_body.add_child(sw)
	controls["profile_switch"] = sw
	controls["profile_option"] = sw.option
	sw.picked.connect(func(_r: Dictionary): _refresh_profile_buttons())
	_note(_body, "A profile is a saved set-up: provider, address, model and what is sent. Switching applies all of it. Assistant enabled, Commands and the Local / Remote switches are not part of a profile. Each profile keeps its own key in the credential store.")
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 8)
	row.add_theme_constant_override("v_separation", 8)
	_body.add_child(row)
	controls["profile_buttons"] = row
	controls["profile_new"] = _button(row, "New…", _on_profile_new)
	controls["profile_duplicate"] = _button(row, "Duplicate", _on_profile_duplicate)
	controls["profile_rename"] = _button(row, "Rename…", _on_profile_rename)
	controls["profile_delete"] = _button(row, "Delete…", _on_profile_delete)
	controls["profile_default"] = _button(row, "Make default", _on_profile_default)
	controls["profile_reset"] = _button(row, "Reset", _on_profile_reset)
	_profile_msg = _note(_body, "", "text_secondary")
	controls["profile_msg"] = _profile_msg
	_refresh_profile_buttons()


## Enables each list button for the active profile, with the reason on the one that cannot act.
func _refresh_profile_buttons() -> void:
	if not controls.has("profile_new") or not is_instance_valid(controls["profile_new"]):
		return
	var set := LlmProfiles.load_set()
	var id := LlmProfiles.active_id()
	var row := LlmProfiles.find(set, id)
	var count := (set.get("profiles", []) as Array).size()
	var full := count >= int(set.get("max_profiles", 16))
	var none_active := row.is_empty()
	var builtin := bool(row.get("builtin", false))
	var why_none := "No profile is active: the settings are your own. Pick a profile, or press New to start one."
	var why_full := "There are already %d profiles. Delete one first." % count
	_profile_button("profile_new", full, why_full)
	_profile_button("profile_duplicate", none_active or full, why_none if none_active else why_full)
	_profile_button("profile_rename", none_active or builtin, why_none if none_active else "The built-in profiles cannot be renamed.")
	_profile_button("profile_delete", none_active or builtin, why_none if none_active else "The built-in profiles cannot be deleted. You can edit and reset them.")
	_profile_button("profile_default", none_active or id == String(set.get("default", "")), why_none if none_active else "It is already the default.")
	_profile_button("profile_reset", none_active or not builtin, why_none if none_active else "Only a built-in profile can be reset to its shipped values.")


## Disables a profile-list button with its reason as the tooltip, or enables it and clears the tooltip.
func _profile_button(key: String, off: bool, reason: String) -> void:
	var btn := controls[key] as Button
	btn.disabled = off
	btn.tooltip_text = reason if off else ""


## Shows `r`'s refusal under the buttons, or `ok_text` when it succeeded.
func _profile_say(r: Dictionary, ok_text: String) -> void:
	if _profile_msg == null or not is_instance_valid(_profile_msg):
		return
	_profile_msg.text = ok_text if bool(r.get("ok", false)) else String(r.get("error", "Not done."))


## New: asks for a name, creates a BLANK profile (every "Send" tick off, nothing chosen) and
## switches to it, so the fields below are the new profile's to fill in. The settings that were live
## are kept as "Previous settings" first when no profile held them.
func _on_profile_new() -> void:
	DccWidgets.prompt(app, "New profile", "Name", "", "Create", func(name: String):
		var r := LlmProfiles.add(name)
		_profile_say(r, "Profile \"%s\" created." % name)
		if bool(r.get("ok", false)):
			var sw := LlmProfiles.switch_to(String(r["id"]))
			if not bool(sw.get("ok", false)):
				_profile_say(sw, "")
		_refresh_profile_buttons(),
		"1 to 40 characters. A new profile starts empty: pick a provider and a model, then save a key if it needs one.")


func _on_profile_duplicate() -> void:
	var id := LlmProfiles.active_id()
	if id == "":
		return
	var r := LlmProfiles.copy_profile(id)
	var copy := LlmProfiles.find(LlmProfiles.load_set(), String(r.get("id", "")))
	_profile_say(r, "Copied as \"%s\". The key is not copied: save one under the copy if it needs it." % String(copy.get("name", "")))
	_refresh_profile_buttons()


func _on_profile_rename() -> void:
	var id := LlmProfiles.active_id()
	var row := LlmProfiles.find(LlmProfiles.load_set(), id)
	if row.is_empty():
		return
	DccWidgets.prompt(app, "Rename profile", "Name", String(row["name"]), "Rename", func(name: String):
		_profile_say(LlmProfiles.rename(id, name), "Renamed.")
		_refresh_profile_buttons())


## Delete: a confirmation card; a stored key is forgotten only if the card's answer says so.
func _on_profile_delete() -> void:
	var id := LlmProfiles.active_id()
	if id == "":
		return
	LlmProfiles.request_delete(app, id, func(deleted: bool, forgot: bool):
		_profile_say({"ok": deleted, "error": "Not deleted."},
			"Deleted." + (" Its saved key was removed." if forgot else ""))
		_rebuild_deferred())


func _on_profile_default() -> void:
	var id := LlmProfiles.active_id()
	if id != "":
		_profile_say(LlmProfiles.set_default(id), "Now the default profile on this device.")
		_refresh_profile_buttons()


func _on_profile_reset() -> void:
	var id := LlmProfiles.active_id()
	if id != "":
		_profile_say(LlmProfiles.reset(id), "Reset to its shipped values.")
		_rebuild_deferred()


## A profile was applied somewhere: redraw every control from the live values it wrote and ask the
## new server for its models. Does nothing while hidden (the next `open()` rebuilds).
func _on_profile_switched() -> void:
	if not visible:
		return
	_rebuild_deferred()
	_models_refresh.call_deferred()
	settings_changed.emit()


## The REPLY group: thinking mode, a pinned reply cap and the retry limit. All three are profile
## values (a local reasoning model wants thinking off and a short cap; a hosted one does not).
func _build_reply() -> void:
	_heading("REPLY")
	var th_idx := 1 if String(DccSettings.llm_value("thinking")) == "none" else 0
	var th := DccWidgets.choice(_body, "Thinking", ["Provider default", "Off for reasoning models"], th_idx,
		func(i: int): _store("thinking", "none" if i == 1 else "default"),
		"Off asks a reasoning model not to think first. Providers that cannot do this are left alone.")
	_floor(th)
	_shrinkable(th)
	controls["thinking"] = th
	var cap := DccWidgets.number(_body, "Reply cap, tokens (0 = automatic)", 0, 8192, 1,
		float(int(DccSettings.llm_value("max_tokens"))),
		func(v: float): _store("max_tokens", 0 if v < 16.0 else int(v)),
		"The longest reply to ask for, 16 to 8192. 0 lets each task choose.")
	_floor(cap)
	controls["max_tokens"] = cap
	var rt := DccWidgets.choice(_body, "Retries", ["No automatic retry", "One automatic retry"],
		clampi(int(DccSettings.llm_value("retry_limit")), 0, 1),
		func(i: int): _store("retry_limit", clampi(i, 0, 1)),
		"After a brief failure, such as a busy server, try once more. Never more than once.")
	_floor(rt)
	_shrinkable(rt)
	controls["retry_limit"] = rt


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
		(controls[k + "_note"] as Label).text = _sw_note(sw) + ("" if enabled else " Turn on Assistant enabled first.")
		_hide_if_empty(controls[k + "_note"] as Label)
	## The hosted opt-in applies to a hosted endpoint only (K2); a local one never needs it.
	if controls.has("rag_hosted"):
		var hep := _endpoint()
		var hosted := bool(hep.get("ok", false)) and not bool(hep.get("is_local", true))
		var cb := controls["rag_hosted"] as BaseButton
		cb.disabled = not enabled or not hosted
		cb.set_pressed_no_signal(hosted and DccSettings.llm_rag_hosted_origins().has(String(hep.get("origin", "-"))))
		(controls["rag_hosted_note"] as Label).text = "" if hosted else "Not needed: this model is on this computer or your network."
		_hide_if_empty(controls["rag_hosted_note"] as Label)
	# Key-entry controls follow the master switch, but also the store/provider conditions.
	if controls.has("key_save"):
		var can := _store_available() and String(DccSettings.llm_value("preset_id")) != ""
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
		DccSettings.llm_key_account())
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
				_rebuild_deferred()
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
var _consent_open := false

## Turns the master switch on from the model list's own button (through the same path as the switch,
## so a missing consent still asks first), then redraws the switch row.
func _enable_assistant() -> void:
	_on_switch("enabled", true)
	_rebuild_deferred()


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
	_models_refresh(true)


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
func _models_refresh(ask: bool = false) -> void:
	if _ml == null:
		return
	var r := _ml.refresh(DccSettings.llm_gate_dict(), DccSettings.llm_consented_origins())
	## The only refusal that opens the consent card is "this address has no consent yet", and only when
	## the user asked for the list (Refresh, or an address they just entered). The master switch being
	## off is a different refusal: no card, and the status line offers the switch instead. A decline
	## leaves the refusal on screen and asks nothing more until the user presses Refresh again.
	if ask and not bool(r.get("ok", false)) and String(r.get("code", "")) == "consent_needed" and not _consent_open:
		var ep := _endpoint()
		if bool(ep.get("ok", false)):
			_consent_open = true
			_ask_consent(ep, func():
				_consent_open = false
				_refresh_dynamic()
				settings_changed.emit()
				_models_refresh(false),
				func():
					_consent_open = false)
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
			txt = "Turn on Assistant enabled first. Nothing is sent while it is off." if _ml_code == "feature_off" else "Not listed. " + _ml_msg
		"failed":
			txt = "Could not list: " + LlmNetHint.with_hint(_ml_msg, LlmNetHint.applies(_ml_code)) + ((" Showing " + stale + ".") if stale != "" else " You can type a name below.")
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
	var eb := controls.get("model_enable") as Button
	if eb != null and is_instance_valid(eb):
		eb.visible = _ml_state == "refused" and _ml_code == "feature_off"
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
	var r := bridge.llm_key_save(DccSettings.llm_key_account(), typed)
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
	var r := bridge.llm_key_forget(DccSettings.llm_key_account())
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
