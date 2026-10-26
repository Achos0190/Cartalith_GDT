extends VBoxContainer
class_name LlmProfileSwitch

## **The profile switch** (Ruling DQ, `LLM_ASSISTANT_SCOPE.md` s.5g): a "Profile" dropdown listing the
## device's profiles (Local, External and the user's own), used in the Assistant panel's header and in
## Assistant settings. Choosing a row calls `LlmProfiles.switch_to`, which applies the whole profile at
## once (and shows the consent card first when the new endpoint has none).
##
## It never decides anything itself and never writes a setting: the choice, the refusal text and the
## consent card are `LlmProfiles`'. A refusal (a profile with a problem, a full set) is shown under the
## dropdown in words, and the dropdown goes back to the profile that is actually active, so the
## control can never claim a profile that is not applied.
##
## When no profile is active (a fresh install, or settings made before profiles existed) the first
## row reads "Custom settings" and is not selectable as a target: it names the state, it is not a
## profile. Reads `LlmProfiles.load_set()` (no writes) on every redraw; redraws on `hub.changed`.

## Emitted after a pick was handled (applied, refused or waiting on the card).
signal picked(result: Dictionary)

## The dropdown. Public so a probe can drive it.
var option: OptionButton
## The refusal / progress line under it.
var note: Label

var _title: Label
## True while `refresh()` fills the dropdown, so its own `select` is not read as a pick.
var _filling := false


func _init() -> void:
	name = "LlmProfileSwitch"
	add_theme_constant_override("separation", 2)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	add_child(row)
	_title = DccTheme.label("Profile", "text_secondary", DccTheme.FS_SMALL)
	row.add_child(_title)
	option = OptionButton.new()
	option.name = "ProfileOption"
	option.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	option.fit_to_longest_item = false
	option.clip_text = true
	option.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	option.tooltip_text = "Switch the whole assistant set-up at once: provider, address, model and what is sent. Assistant enabled and Commands are not part of a profile."
	option.item_selected.connect(_on_item_selected)
	row.add_child(option)
	note = DccTheme.label("", "warn", DccTheme.FS_MICRO)
	note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	note.custom_minimum_size.x = 160
	note.visible = false
	add_child(note)
	## The density floor (44 px on a phone, the tablet button floor on a tablet) is applied here, at
	## creation, because a window that rebuilds its body creates this control after `phone_fit` ran.
	if DccTheme.is_phone():
		option.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN
	elif DccTheme.is_tablet():
		option.custom_minimum_size.y = DccTheme.role_px("btn_min_h")
	else:
		option.custom_minimum_size.y = 26
	LlmProfiles.hub.changed.connect(_on_hub_changed)
	tree_exiting.connect(func():
		if LlmProfiles.hub.changed.is_connected(_on_hub_changed):
			LlmProfiles.hub.changed.disconnect(_on_hub_changed))
	refresh()


func _on_hub_changed() -> void:
	refresh.call_deferred()


## Redraws the dropdown from the stored set. Safe at any time.
func refresh() -> void:
	if not is_instance_valid(option):
		return
	var set := LlmProfiles.load_set()
	_filling = true
	option.clear()
	if not bool(set.get("ok", false)):
		option.add_item("Profiles unavailable")
		option.disabled = true
		_filling = false
		return
	option.disabled = false
	var active := LlmProfiles.active_id()
	var sel := 0
	if active == "":
		option.add_item("Custom settings")
		option.set_item_metadata(0, "")
		option.set_item_disabled(0, true)
	for p in set["profiles"]:
		option.add_item(String(p["name"]) + (" · default" if String(p["id"]) == String(set["default"]) else ""))
		var i := option.item_count - 1
		option.set_item_metadata(i, String(p["id"]))
		if String(p["id"]) == active:
			sel = i
	option.select(sel)
	_filling = false


## The profile id of the dropdown's selected row ("" for the Custom row).
func selected_id() -> String:
	var i := option.selected
	return String(option.get_item_metadata(i)) if i >= 0 and i < option.item_count else ""


func _on_item_selected(index: int) -> void:
	if _filling:
		return
	var id := String(option.get_item_metadata(index))
	if id == "":
		refresh()
		return
	var r := LlmProfiles.switch_to(id)
	## A refusal (`reason`) or a switch to a profile that cannot send yet (`warn`) is said in words.
	var said := String(r.get("reason", "Not switched.")) if not bool(r.get("ok", false)) else String(r.get("warn", ""))
	note.text = said
	note.visible = said != ""
	refresh()
	picked.emit(r)
