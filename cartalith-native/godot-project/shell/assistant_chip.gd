extends RefCounted
class_name AssistantChip

## The conditional Assistant chip's STATE -- which of "hidden", "disabled" and "live" it is, and why
## (`CUSTOM_QUICK_BAR_SCOPE.md` s.5, milestone CQ-7, Ruling DH Q6, owner request 2026-10-25: *"put an llm
## button under the cartwheel as soon as an api/llm has been added by the user"*).
##
## ## The rule (Ruling DH item 6, exactly)
##
## Shown iff the assistant is enabled, a provider is configured, and the user has not turned the chip
## off. Two further conjuncts keep it from ever being a dead control: the build must have the
## assistant bindings (`bridge.llm_api`), and the assistant PANEL must exist to open (scope s.5's
## "No panel yet" row: a button that opens nothing is the control this project forbids). It is a
## built-in conditional item, not a user pin: it is in no pin list, so it cannot be removed from one,
## and it does not count against the dock's four pins or the bar's 24.
##
## "Provider configured" is `preset_id` or `base_url` non-empty, AND `model` non-empty (the gate's own
## requirement, `LLM_DEFAULTS` says an empty one is "not set"). It deliberately does not require a key:
## a local server needs none, and a missing key on a remote one is a reason to SHOW the chip greyed with
## the gate's sentence, not to hide it (the user has just added a provider and should see why it is not
## live yet).
##
## ## Two layers, so the logic can be table-tested with literals
##
## [`decide`] is the whole decision as a pure function of seven booleans: no settings, no bridge, no node.
## The probe walks all 2^7 = 128 combinations against an independently written truth table
## (`_quickbar_assistant_probe.gd`), and mutation-tests each conjunct. [`state_for_app`] gathers the
## inputs from `DccSettings` and the bridge and calls [`decide`]; it makes NO network call (the gate,
## `bridge.llm_authorize`, is a local pure function in Rust) and is consulted only once every other
## conjunct holds.
##
## **Never encode "no value" as a plausible value** (`MISTAKES.md`): the gate's `reason` is carried
## through verbatim; when the gate refuses and gives none, the chip says so in words instead of
## inventing a cause.

## The three states a chip can be in.
const HIDDEN := "hidden"
const DISABLED := "disabled"
const LIVE := "live"

## What the chip says when the gate refused and named no reason (it always names one today; this is
## the honest fallback rather than a made-up cause).
const NO_REASON := "The assistant is not ready, and the gate gave no reason."
## The chip's label, in the cluster and on the dock.
const LABEL := "Assistant"

## The whole decision. `gate_ok` is only meaningful when every earlier argument is true (the caller
## passes `false` otherwise without having consulted the gate); with any earlier one false the answer
## is HIDDEN whatever `gate_ok` says. Hidden reserves nothing (scope s.5: "no gap reserved").
static func decide(api_present: bool, panel_present: bool, enabled: bool, has_provider: bool,
		has_model: bool, button_on: bool, gate_ok: bool) -> String:
	if not (api_present and panel_present and enabled and has_provider and has_model and button_on):
		return HIDDEN
	return LIVE if gate_ok else DISABLED

## Whether the settings name a provider: a preset or an address, AND a model. Static and
## settings-free so the probe can feed it literals.
static func provider_configured(preset_id: String, base_url: String, model: String) -> bool:
	return (preset_id != "" or base_url != "") and model != ""

## The chip's state for `app` right now: `{state, reason}`. `reason` is "" unless the state is
## DISABLED, where it is the gate's own sentence. Reads only `DccSettings` and one local call into the
## bridge; opens nothing and contacts nothing.
static func state_for_app(app) -> Dictionary:
	var out := {"state": HIDDEN, "reason": ""}
	if app == null:
		return out
	var bridge = app.get("bridge")
	var api: bool = bridge != null and bool(bridge.get("llm_api"))
	## The panel is what a click opens. `assistant_window` exists from boot; checking the method as well
	## keeps a harness stub without a panel from showing a button that does nothing.
	var panel: bool = app.get("assistant_window") != null and app.has_method("open_assistant_panel")
	var enabled := bool(DccSettings.llm_value("enabled"))
	var has_provider := String(DccSettings.llm_value("preset_id")) != "" or String(DccSettings.llm_value("base_url")) != ""
	var has_model := String(DccSettings.llm_value("model")) != ""
	var button_on := bool(DccSettings.quick_bar_value("assistant_button"))
	var gate := {}
	var gate_ok := false
	## The gate is asked only when nothing else already hides the chip: no call for a state that
	## cannot be shown, and no way for a missing binding to be read as an answer.
	if api and panel and enabled and has_provider and has_model and button_on:
		gate = bridge.llm_authorize(DccSettings.llm_gate_dict(), "", DccSettings.llm_consented_origins())
		gate_ok = bool(gate.get("ok", false))
	var st := decide(api, panel, enabled, has_provider, has_model, button_on, gate_ok)
	out["state"] = st
	if st == DISABLED:
		var why := String(gate.get("reason", ""))
		out["reason"] = why if why != "" else NO_REASON
	return out

## The tooltip for `st` (a [`state_for_app`] result): what a click does, or why it does nothing. The
## key never appears here -- the reason is the gate's sentence, which names the variable, never its value.
static func tooltip_for(st: Dictionary) -> String:
	if String(st.get("state", HIDDEN)) == LIVE:
		return "Open the assistant panel"
	return "Assistant unavailable: %s" % String(st.get("reason", NO_REASON))

## Opens the panel through the app, the one method the menu row (`Story ▸ NOTES ▸ Assistant panel…`) also
## calls. A refused state opens nothing; the chip is disabled then, so this is a second line of defence
## for a caller that bypasses the button. Returns whether it opened.
static func open_panel(app) -> bool:
	if app == null or String(state_for_app(app)["state"]) != LIVE:
		return false
	app.open_assistant_panel()
	return true
