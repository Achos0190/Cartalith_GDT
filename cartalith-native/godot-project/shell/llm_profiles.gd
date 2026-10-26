extends RefCounted
class_name LlmProfiles

## **Assistant profiles** -- the shell side (`LLM_ASSISTANT_SCOPE.md` s.5g, Ruling DQ, owner request
## 2026-10-26): a global Local / External switch backed by named, per-device bundles of provider,
## address, model, thinking mode, the six "Send: ..." ticks, a reply cap, a retry limit and the
## local GPU override. Every RULE (what is valid, how many, naming, what a delete hands back) is
## `cartalith_llm::profiles`, reached through `EngineBridge.llm_profiles_parse/op`; this file only
## moves values between that, `DccSettings` and the windows.
##
## ## How a profile relates to the live settings
##
## The live `[llm]` keys (`DccSettings.LLM_PROFILE_KEYS`) stay the single thing every existing reader
## consults. The ACTIVE profile (`set.last`) mirrors them: a change made anywhere (settings window,
## panel) is written THROUGH to it (`_write_through`, wired as `DccSettings.profile_hook`), and a
## SWITCH writes the whole profile onto them in one save (`apply`). So "switching applies
## everything at once" is one `set_llm_values` call, and an edit can never leave a profile stale.
##
## ## What a profile may and may not do (Ruling DQ rules)
##
## * **Never a key.** `live_values()` reads only `LLM_PROFILE_KEYS`; the stored text is built by Rust
##   from a type with no key field; `key_account` is the NAME of the credential-store entry.
## * **Never a switch.** `apply` writes the profile keys and nothing else: the master switch, Commands,
##   Edit and the Local / Remote provider switches are not in the profile and are never touched.
##   Switching to a profile whose endpoint the gate would refuse (a provider class that is switched
##   off) therefore shows the gate's reason; it does not turn the class on.
## * **Consent first.** If the master switch is on and the new endpoint has no recorded consent, the
##   profile is applied TENTATIVELY (so the card, which reads the live ticks, tells the truth about
##   what will be sent), the consent card is shown, and a decline puts every value back. While the
##   card is open the gate refuses the endpoint (`consent_needed`), so nothing can be sent.
## * **Remote requires https.** A profile with a problem (`Profile::problems`: a remote address over
##   http://, an unknown provider, an unusable address) IS applied, so the user can open it and fix it,
##   but no consent card is offered for it and the gate refuses every request until it is fixed
##   (`cleartext_remote`, `base_url`, `unknown_preset`); the switch says why in words (`warn`). Refusing the
##   switch instead would leave a profile nobody can reach to repair.
## * **Delete never removes a key unasked.** `request_delete` shows a card; only its "forget" answer
##   calls `llm_key_forget`.

## Emitted when the LIST changed (add, duplicate, rename, delete, default, reset) so a switcher
## redraws. Not emitted by the write-through, which changes no name.
class Hub extends RefCounted:
	signal changed
	## Emitted after a profile was APPLIED (a switch, or a decline put the old values back), so a
	## window rebuilds its controls from the new live values.
	signal switched

static var hub := Hub.new()

static var _app = null
static var _bridge = null
## The last consent card (`LlmConsent.present`'s return), for probes.
static var last_consent: Dictionary = {}

## Whether this device's set is the phone's (the built-in Local profile has no address there).
static func is_phone_set() -> bool:
	return OS.has_feature("android")


## Wires the profiles to the running app: the bridge they call and the write-through hook. Call once,
## after the bridge exists and before the settings window or panel is used. Reads and writes nothing.
static func install(app, bridge) -> void:
	_app = app
	_bridge = bridge
	DccSettings.profile_hook = _write_through


## True when the library this build loaded has the profile functions.
static func available() -> bool:
	return _bridge != null and bool(_bridge.get("llm_api"))


## The stored set as `llm_profiles_parse` answers it, or `{ok: false}` without a library. Never
## writes: opening a window must not create the section (`_llmset_probe.gd` asserts it).
static func load_set() -> Dictionary:
	if not available():
		return {"ok": false, "error": "This build has no assistant support."}
	return _bridge.llm_profiles_parse(DccSettings.llm_profiles_data(), is_phone_set())


## One profile row by id, or `{}`.
static func find(set: Dictionary, id: String) -> Dictionary:
	for p in set.get("profiles", []):
		if String(p["id"]) == id:
			return p
	return {}


## The id of the profile whose values the live settings mirror, or "" when none is active (a fresh
## install, or settings the user made before profiles existed and has not switched away from).
static func active_id() -> String:
	var set := load_set()
	if not bool(set.get("ok", false)):
		return ""
	var last := String(set.get("last", ""))
	return last if not find(set, last).is_empty() else ""


## The live profile-carried values, exactly `DccSettings.LLM_PROFILE_KEYS`.
static func live_values() -> Dictionary:
	var d := {}
	for k in DccSettings.LLM_PROFILE_KEYS:
		d[k] = DccSettings.llm_value(k)
	return d


## Whether the live settings name a provider or model at all (so there is something worth keeping
## before a first switch overwrites it).
static func _live_configured() -> bool:
	return String(DccSettings.llm_value("preset_id")) != "" or String(DccSettings.llm_value("model")) != "" \
		or String(DccSettings.llm_value("base_url")) != ""


## One edit of the stored text through Rust; stores the answer. Returns Rust's answer (`ok` false with
## `error` leaves the stored text untouched). `notify` emits `hub.changed` (list edits yes, the
## write-through no).
static func _op(op: String, args: Dictionary, notify: bool = true) -> Dictionary:
	if not available():
		return {"ok": false, "error": "This build has no assistant support."}
	var r: Dictionary = _bridge.llm_profiles_op(DccSettings.llm_profiles_data(), op, args, is_phone_set())
	if not bool(r.get("ok", false)):
		return r
	if not DccSettings.set_llm_profiles_data(String(r["json"])):
		return {"ok": false, "error": "The profiles could not be stored."}
	if notify:
		hub.changed.emit()
	return r


## The write-through (`DccSettings.profile_hook`): the live values of the active profile, stored on it.
static func _write_through() -> void:
	var id := active_id()
	if id == "":
		return
	_op("sync", {"id": id, "live": live_values()}, false)


## Writes `live` onto the `[llm]` settings in one save. Only the profile keys are accepted, whatever
## the dictionary holds, and `key_account` rides along (the account of the profile being applied).
static func _write_live(live: Dictionary) -> bool:
	var out := {}
	for k in DccSettings.LLM_PROFILE_KEYS:
		if live.has(k):
			out[k] = live[k]
	if live.has("key_account"):
		out["key_account"] = live["key_account"]
	DccSettings.profile_applying = true
	var ok := DccSettings.set_llm_values(out)
	DccSettings.profile_applying = false
	return ok


## Switches to profile `id` and applies all of it at once. Returns `{ok, pending, reason}`:
## `ok` false with a `reason` when the profile is unknown or the set cannot be read (nothing changed);
## `warn` is set when the profile has a problem (see the https rule above), applied all the same;
## `pending` true when the consent card is open (the new values are live, tentatively, and a
## decline restores the old ones); otherwise the switch is done. `on_done` runs after the switch is
## final (immediately, or after the card's "I understand, enable"), never after a decline.
static func switch_to(id: String, on_done: Callable = Callable()) -> Dictionary:
	var set := load_set()
	if not bool(set.get("ok", false)):
		return {"ok": false, "pending": false, "reason": String(set.get("error", "No profiles."))}
	var row := find(set, id)
	if row.is_empty():
		return {"ok": false, "pending": false, "reason": "That profile no longer exists."}
	if id == active_id():
		return {"ok": true, "pending": false, "reason": ""}
	var problems: Array = row.get("problems", [])
	var warn := ""
	if not problems.is_empty():
		var why: PackedStringArray = PackedStringArray()
		for pr in problems:
			why.append(String(pr["reason"]))
		warn = "%s is switched to, but nothing will be sent until this is fixed: %s." % [String(row["name"]), "; ".join(why)]
	## Settings made before profiles existed (or never saved as one) are kept as a custom profile
	## before this switch overwrites them: a switch must not silently destroy the user's set-up.
	var prev_last := active_id()
	var prev_live := live_values()
	var prev_account := String(DccSettings.llm_value("key_account"))
	if prev_last == "" and _live_configured():
		var kept := _op("adopt", {"live": prev_live, "account": DccSettings.llm_key_account()})
		if not bool(kept.get("ok", false)):
			## At the 16 cap there is no room to keep them: say so and change nothing.
			return {"ok": false, "pending": false, "reason": "Your current settings are not saved as a profile and there is no room to keep them (16 profiles). Delete a profile first."}
		## The adopted profile becomes the one to return to if the switch is declined.
		prev_last = String(kept["id"])
		_op("last", {"id": prev_last}, false)
	var live: Dictionary = row["live"]
	if not _write_live(live):
		return {"ok": false, "pending": false, "reason": "The profile could not be applied."}
	_op("last", {"id": id}, false)
	if _app != null and _bridge != null:
		LlmGpuRule.sync(_bridge)
	## Consent first, for the NEW endpoint, when the master switch is on and nothing can be sent
	## until it is given. Master off: nothing can be sent at all, and turning the master switch on
	## asks (`LlmSettingsWindow._on_switch`), so no card is due yet.
	var ep: Dictionary = _bridge.llm_endpoint_info(String(live["preset_id"]), String(live["base_url"]))
	var origin := String(ep.get("origin", ""))
	if warn == "" and bool(DccSettings.llm_value("enabled")) and bool(ep.get("ok", false)) \
			and not DccSettings.llm_consented_origins().has(origin):
		var preset_row := {}
		for p in _bridge.llm_presets():
			if String(p["id"]) == String(live["preset_id"]):
				preset_row = p
		var restore := func() -> void:
			## A decline (Not now, Esc, the close box): every value and the mark go back as they were.
			prev_live["key_account"] = prev_account
			_write_live(prev_live)
			_op("last", {"id": prev_last}, false)
			if _app != null and _bridge != null:
				LlmGpuRule.sync(_bridge)
			DccSettings.notify_assistant_state()
			hub.changed.emit()
			hub.switched.emit()
		var done := func() -> void:
			DccSettings.notify_assistant_state()
			hub.changed.emit()
			hub.switched.emit()
			if on_done.is_valid():
				on_done.call()
		last_consent = LlmConsent.present(_app, ep, preset_row, done, restore)
		DccSettings.notify_assistant_state()
		hub.changed.emit()
		hub.switched.emit()
		return {"ok": true, "pending": true, "reason": "", "warn": ""}
	DccSettings.notify_assistant_state()
	hub.changed.emit()
	hub.switched.emit()
	if on_done.is_valid():
		on_done.call()
	return {"ok": true, "pending": false, "reason": "", "warn": warn}


## Startup: restore the last used profile, else the default, but ONLY when profiles were ever stored
## (a fresh install changes nothing: the shipped assistant settings are empty by design). Applies
## silently, with no consent card: the consent and the master switch are exactly as they were when the
## app closed, and nothing is sent without both. Writes nothing when the live values already match.
static func restore() -> void:
	if DccSettings.llm_profiles_data() == "" or not available():
		return
	var set := load_set()
	if not bool(set.get("ok", false)):
		return
	var id := String(set.get("startup", ""))
	var row := find(set, id)
	if row.is_empty() or not (row.get("problems", []) as Array).is_empty():
		return
	var live: Dictionary = row["live"]
	var same := String(set.get("last", "")) == id and String(DccSettings.llm_value("key_account")) == String(live["key_account"])
	for k in DccSettings.LLM_PROFILE_KEYS:
		if DccSettings.llm_value(k) != live[k]:
			same = false
	if same:
		return
	if _write_live(live):
		_op("last", {"id": id}, false)


## Adds a custom profile (from `from_id`'s values, or empty). Returns Rust's answer.
static func add(name: String, from_id: String = "") -> Dictionary:
	return _op("add", {"name": name, "from": from_id})


static func copy_profile(id: String) -> Dictionary:
	return _op("duplicate", {"id": id})


static func rename(id: String, name: String) -> Dictionary:
	return _op("rename", {"id": id, "name": name})


static func set_default(id: String) -> Dictionary:
	return _op("default", {"id": id})


## Puts a built-in back to its shipped values. When it is the active profile the live settings follow
## (it is applied again), without a consent card: resetting can only move the endpoint to the shipped
## one, and the gate still refuses it until its consent exists.
static func reset(id: String) -> Dictionary:
	var r := _op("reset", {"id": id})
	if bool(r.get("ok", false)) and active_id() == id:
		var row := find(load_set(), id)
		if not row.is_empty():
			_write_live(row["live"])
			DccSettings.notify_assistant_state()
			hub.switched.emit()
	return r


## Deletes a custom profile. Never touches a stored key: the answer carries `account`, and the caller
## asks (`request_delete`) before calling `llm_key_forget`. Deleting the ACTIVE profile leaves the live
## settings as they are, now "custom" (no profile is active).
static func delete(id: String) -> Dictionary:
	return _op("delete", {"id": id})


## Whether a key is stored in the OS credential store under `account` (never its value).
static func key_present(account: String) -> bool:
	if _bridge == null or not available() or account == "":
		return false
	var r: Dictionary = _bridge.llm_key_status("store", "X", account)
	return bool(r.get("present", false))


## The delete flow: a confirmation card for profile `id`. When a key is stored for its account the card
## offers "Delete, forget key" and "Delete, keep key"; otherwise one "Delete". Cancel and Esc
## change nothing. Returns the card (for probes), or `{}` when the profile cannot be deleted. Never
## deletes by itself: only a card answer does.
static func request_delete(host, id: String, on_done: Callable = Callable()) -> Dictionary:
	var row := find(load_set(), id)
	if row.is_empty() or bool(row.get("builtin", false)):
		return {}
	var account := String((row.get("live", {}) as Dictionary).get("key_account", ""))
	var has_key := key_present(account)
	var card := DccWidgets.modal_card(host, "Delete profile", DccWidgets.MODAL_DESTRUCTIVE, 380)
	var body: VBoxContainer = card["body"]
	var msg := "Delete the profile \"%s\"?" % String(row["name"])
	var l := DccWidgets.modal_prose(body, msg)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var detail := DccWidgets.modal_prose(body, \
		("A key is stored for it in the OS credential store. Deleting the profile does not remove the key unless you choose to forget it." \
		if has_key else "No key is stored for it."))
	detail.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var finish := func(forget: bool) -> void:
		var r := delete(id)
		if bool(r.get("ok", false)) and forget and has_key:
			_bridge.llm_key_forget(String(r.get("account", account)))
			DccSettings.notify_assistant_state()
		if on_done.is_valid():
			on_done.call(bool(r.get("ok", false)), forget and has_key)
	var spec := {"cancel": "Cancel"}
	if has_key:
		spec["destructive"] = {"text": "Delete, forget key", "on": func(): finish.call(true)}
		spec["safe"] = {"text": "Delete, keep key", "on": func(): finish.call(false)}
	else:
		spec["destructive"] = {"text": "Delete", "on": func(): finish.call(false)}
	var choices := DccWidgets.modal_choices(card, spec)
	DccWidgets.modal_present(card, host)
	return {"card": card, "choices": choices, "has_key": has_key, "account": account}
