class_name LlmNetHint
extends RefCounted
## The Android "network access may be off" hint and the button that opens the app's system page.
##
## WHY (owner finding 2026-10-26, OnePlus 6T, LineageOS 22): INTERNET is an install-time permission, so
## Settings > Apps > Cartalith > Permissions lists nothing to switch. LineageOS adds its own per-app
## "Allow network access" switch (App info > Wi-Fi data usage) that can block the app entirely (netpolicy
## REJECT_ALL); every connection then fails with "Timed out connecting". The user needs to be told where it is.
##
## RULES: the hint is shown ONLY on Android and ONLY for a transport-level failure (could not connect, timed
## out connecting, unreachable); never for an HTTP status error (the server answered, so the network works)
## and never on desktop or tablet builds. The existing failure text and the key-never-echoed rule are
## untouched: this class only ever appends a fixed sentence containing no user data.
##
## ALL USER-FACING TEXT OF THIS FEATURE IS IN THE CONSTANTS BELOW (one place, so a copy pass merges trivially).

const HINT := "Network access may be off for Cartalith. Turn it on: Settings > Apps > Cartalith > Wi-Fi data usage > Allow network access."
const FIRST_RUN_NOTE := "If connections fail, check Wi-Fi data usage in the app's system settings."
const BUTTON_TEXT := "Open app settings"
const BUTTON_TIP := "Opens Cartalith's page in the system settings."

## `LlmModelList` failure codes that mean "never reached a server". `transport` (a mid-exchange fault) and every
## HTTP-status code are deliberately absent.
const TRANSPORT_CODES := ["unreachable", "timeout"]

const ACTION := "android.settings.APPLICATION_DETAILS_SETTINGS"

const OPENED := "opened"
const UNAVAILABLE := "unavailable"

## Test seams. -1 = ask the engine; 0 / 1 = pretend. A probe sets these; nothing in the shipped shell does.
static var fake_android: int = -1
## Replaces the JNI call: `func(action: String, data_uri: String) -> bool`. Invalid = use the real one.
static var fake_opener: Callable = Callable()
static var fake_package: String = ""
## The last intent this class built (`{action, data}`), for a probe to assert. Empty until a call.
static var last_call: Dictionary = {}


static func is_android() -> bool:
	if fake_android >= 0:
		return fake_android == 1
	return OS.has_feature("android")


## True only for an Android build and a transport-level failure code.
static func applies(code: String) -> bool:
	return is_android() and TRANSPORT_CODES.has(code)


## `message` plus the hint when it applies (a space between), else `message` unchanged.
static func with_hint(message: String, transport: bool) -> String:
	if not (transport and is_android()):
		return message
	return message + " " + HINT


## Opens the app's own App info page. Returns OPENED or UNAVAILABLE; never throws. Desktop: UNAVAILABLE.
static func open_app_settings() -> String:
	if not is_android():
		return UNAVAILABLE
	if fake_opener.is_valid():
		var pkg := fake_package
		last_call = {"action": ACTION, "data": "package:" + pkg}
		return OPENED if bool(fake_opener.call(ACTION, "package:" + pkg)) else UNAVAILABLE
	## Real path: the AndroidRuntime singleton and JavaClassWrapper (Godot 4.4+; the Vault SAF picker uses
	## the engine's own dialog, so there is no in-repo JNI precedent to copy). A constructor is a method named
	## like the class; a static method is called on the wrapper; both per the JavaClassWrapper class reference.
	if not Engine.has_singleton("AndroidRuntime"):
		return UNAVAILABLE
	var rt = Engine.get_singleton("AndroidRuntime")
	var activity = rt.getActivity()
	if activity == null:
		return UNAVAILABLE
	var pkg_name := String(activity.getPackageName())
	if pkg_name == "":
		return UNAVAILABLE
	var intent_cls = JavaClassWrapper.wrap("android.content.Intent")
	var uri_cls = JavaClassWrapper.wrap("android.net.Uri")
	if intent_cls == null or uri_cls == null:
		return UNAVAILABLE
	var intent = intent_cls.Intent(ACTION)
	intent.setData(uri_cls.parse("package:" + pkg_name))
	last_call = {"action": ACTION, "data": "package:" + pkg_name}
	activity.startActivity(intent)
	return OPENED


## A "Open app settings" button added to `parent` (Android only; returns null elsewhere).
static func make_button(parent: Control, min_h: float = 0.0) -> Button:
	if not is_android():
		return null
	var b := DccWidgets.action(parent, BUTTON_TEXT, func(): open_app_settings())
	b.tooltip_text = BUTTON_TIP
	if min_h > 0.0:
		b.custom_minimum_size.y = min_h
	return b
