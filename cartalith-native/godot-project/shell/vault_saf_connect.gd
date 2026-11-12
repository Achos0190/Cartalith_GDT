extends RefCounted
## Connecting an Android folder as the vault (Ruling AV; VE-6's Android half): the picker, the persisted
## grant, and the one place that builds a handler and hands it to `vault_connect_saf`. Static so the vault
## window, the launch-time restore (`vault_store.gd`) and the probe all go through the same three functions.
##
## Held without a `class_name`, like the other vault helpers, so no import pass is needed; callers
## `preload` it. Everything that touches Android is in `vault_saf_android.gd` and was not run on a device.

const Handler := preload("res://shell/vault_saf_handler.gd")
const AndroidDriver := preload("res://shell/vault_saf_android.gd")

## The handler the bridge's `Callable` points at. A `Callable` names a `RefCounted` object without owning
## it, so the connection would silently die with the handler; this static is what keeps it alive.
static var _live_handler = null


## True when this build should offer the Android folder grant instead of the in-app folder browser:
## on Android, with the OS dialog available. Desktop is never routed here (the in-app browser is the
## shell's chrome, and it works there).
static func should_use_picker() -> bool:
	return OS.get_name() == "Android" and DisplayServer.has_feature(DisplayServer.FEATURE_NATIVE_DIALOG_FILE)


## Raises Android's folder picker (`ACTION_OPEN_DOCUMENT_TREE` underneath). `on_picked(uri)` runs with the
## `content://` tree address; a cancel or an empty answer runs nothing. A picker that cannot be raised
## answers through `on_error(message)`, so it is never silent.
static func pick(title: String, on_picked: Callable, on_error: Callable) -> void:
	var err := DisplayServer.file_dialog_show(title, "", "", false, DisplayServer.FILE_DIALOG_MODE_OPEN_DIR,
		PackedStringArray(),
		func(ok: bool, picked: PackedStringArray, _filter: int):
			if not ok or picked.is_empty():
				return
			if on_picked.is_valid():
				on_picked.call(String(picked[0])))
	if err != OK and on_error.is_valid():
		on_error.call("the Android folder picker could not be opened (error %d)" % err)


## Connects `tree_uri` through the real Android driver: prepare, persist the grant, bind. A grant that
## cannot be persisted still connects (it works until the app closes) and says so in `warning`.
static func connect_android(bridge, tree_uri: String, display_name: String = "") -> Dictionary:
	var driver = AndroidDriver.new(tree_uri)
	var ready: Dictionary = driver.prepare()
	if not bool(ready.get("ok", false)):
		return ready
	var kept: Dictionary = driver.persist_grant()
	var r := connect_with_driver(bridge, tree_uri, display_name, driver)
	if bool(r.get("ok", false)) and not bool(kept.get("ok", false)):
		r["warning"] = "Android may forget this folder when the app closes: %s" % String(kept.get("error", "the grant was not persisted"))
	return r


## The shared tail: a handler over `driver`, held alive, bound through the bridge. The probe calls this
## with a fake driver; Android calls it with the real one.
static func connect_with_driver(bridge, tree_uri: String, display_name: String, driver) -> Dictionary:
	var handler = Handler.new(driver)
	_live_handler = handler
	var r: Dictionary = bridge.vault_connect_saf(tree_uri, display_name, Callable(handler, "_saf_dispatch"))
	if not bool(r.get("ok", false)):
		_live_handler = null
	return r


## Launch-time restore of a saved `content://` binding. Silent when Android no longer holds the grant (the
## panel then shows the vault as unbound, the same designed state a missing desktop folder leaves), and
## `false` in that case so the caller can say why.
static func restore(bridge, tree_uri: String, display_name: String) -> bool:
	if not tree_uri.begins_with("content://"):
		return false
	var driver = AndroidDriver.new(tree_uri)
	if not driver.grant_is_held():
		return false
	return bool(connect_with_driver(bridge, tree_uri, display_name, driver).get("ok", false))
