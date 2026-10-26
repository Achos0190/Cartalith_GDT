extends RefCounted
class_name LlmGpuRule

## **The local-provider GPU rule, applied** (Ruling DF; `LLM_ASSISTANT_SCOPE.md` s.5a).
##
## `cartalith-llm::gpu::plan` (Rust, tested) decides what the rule says; this file does
## the one thing Rust cannot from there -- point the compute path (`wgpu`, via
## `WorldGen.gpu_set_selected_devices`) at another adapter for the session, and put the
## user's own choice back afterwards.
##
## ## What it switches, and what it does not
##
## **Only the compute path**, by device key, taking effect on the next generate (no
## restart). The Godot *display* adapter is not switchable from inside the running app
## under `gl_compatibility` (scope s.5a, verified 2026-10-25); for it the rule is the
## warning and the "detected as" line, and the plan's `effect` text says so.
##
## ## Safety properties
##
## * **Never persisted.** The switch goes through `EngineBridge.
##   gpu_set_selected_devices_session`, which does not write `DccSettings`. A restart
##   returns to the user's own persisted choice, and `sync()` reapplies the rule at boot.
## * **Restores, never forgets.** Restore sets the engine back to
##   `DccSettings.gpu_devices()` (the user's persisted choice; empty means automatic),
##   not to a value captured earlier, so a choice the user made in Preferences while the
##   rule was active is what returns.
## * **Active only when the feature is.** The switch is applied only while the master
##   switch is on, local providers are allowed and the chosen endpoint is local. The
##   *warning* is shown whenever a local preset is selected, even before that.
## * **Silent on a headless or GPU-less machine.** No adapters means no target and no
##   switch; the plan says "No integrated GPU was found".
##
## MUST NEVER: write the device key into `DccSettings` or a world, switch on a guess
## (the class comes from the URL text only), or call `WorldGen` directly (everything goes
## through `EngineBridge`, which refuses while a generation owns the engine).

## Whether this session has pointed the engine at the rule's target and not yet restored
## it. Static, because the rule is process-wide like the preference it drives.
static var _applied := false
## A sync was refused (a generation was running) and must be retried when it finishes.
static var _pending := false

## Whether `sync()` currently has the engine switched.
static func is_applied() -> bool:
	return _applied

## Whether a refused sync is waiting for the generation to end.
static func is_pending() -> bool:
	return _pending

## Is a local provider *selected* (the warning's condition): a local preset, or a custom
## endpoint classed same-machine or LAN. Independent of the master switch.
static func local_selected(bridge) -> bool:
	var id := String(DccSettings.llm_value("preset_id"))
	var url := String(DccSettings.llm_value("base_url"))
	if id == "" or url == "":
		return false
	var info: Dictionary = bridge.llm_endpoint_info(id, url)
	return bool(info.get("ok", false)) and bool(info.get("is_local", false))

## Is the provider *active* (the switch's condition): master on, local providers allowed,
## and the endpoint local.
static func is_active(bridge) -> bool:
	return bool(DccSettings.llm_value("enabled")) \
		and bool(DccSettings.llm_value("local_allowed")) \
		and local_selected(bridge)

## Whether the phone form of the rule applies: the running OS is Android. Decided by the
## platform, not the shell's density, so a desktop probe at phone size still gets the
## desktop rule (that machine does have a dedicated card to give away).
static func is_phone() -> bool:
	return OS.has_feature("android")

## The plan the settings screen shows right now: `WorldGen`'s `llm_gpu_plan` over this
## machine's adapters and the stored override. `{}` when the binding is missing.
static func current_plan(bridge) -> Dictionary:
	if not bridge.llm_api:
		return {}
	## Adapters are enumerated ONLY when a local provider is selected: enumeration brings up a
	## `wgpu` instance, which a default install (feature off, nothing chosen) must not pay for at
	## every boot and every settings change. With none selected the plan has no warning and no
	## target whatever the adapter list is, so an empty list answers the same.
	var selected := local_selected(bridge)
	## LLM-10: a phone has one GPU, so its plan never switches and never enumerates adapters
	## (no wgpu instance is brought up for it); the text is the phone's, not the desktop's.
	if is_phone():
		return bridge.llm_gpu_plan_phone(String(DccSettings.llm_value("base_url")), selected)
	var devices: Array = bridge.gpu_devices() if (selected and bridge.gpu_api) else []
	return bridge.llm_gpu_plan(String(DccSettings.llm_value("base_url")), selected,
		String(DccSettings.llm_value("gpu_override")), devices)

## Brings the engine's device selection in line with the rule. Call after any `[llm]`
## setting changes, at boot, and when a generation ends while `is_pending()`. Returns
## the plan it acted on (or `{}` without the bindings).
##
## Never throws and never leaves the engine switched when the rule no longer says to:
## every path that does not switch restores if it had.
static func sync(bridge) -> Dictionary:
	var plan := current_plan(bridge)
	if plan.is_empty() or not bool(plan.get("ok", false)):
		return plan
	var want := bool(plan.get("switch", false)) and plan.has("target") and is_active(bridge)
	if want:
		if bridge.gpu_set_selected_devices_session(PackedStringArray([String(plan["target"])])):
			_applied = true
			_pending = false
		else:
			_pending = true
	elif _applied:
		if bridge.gpu_set_selected_devices_session(DccSettings.gpu_devices()):
			_applied = false
			_pending = false
		else:
			_pending = true
	else:
		_pending = false
	return plan
