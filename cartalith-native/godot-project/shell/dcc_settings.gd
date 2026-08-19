extends RefCounted
class_name DccSettings

## Persisted, user-configurable shell state: the four storage roots
## (`DCC_SHELL_SPEC.md` §2.1's "Storage locations" / "Change locations…") and
## the recent-projects list (§2.1's "Recent worlds" submenu, "last 10
## projects").
##
## Backed by one `ConfigFile` at `user://cartalith_settings.cfg` -- the
## simplest persistence Godot offers, and the first thing in this shell that
## writes to `user://` (grepped for `ConfigFile`/`user://` across
## `godot-project/shell/` before adding this; nothing existed).
##
## Root defaults come from `OS.get_user_data_dir()`, not §2.1's own literal
## `~/Cartalith/Worlds` etc. -- that prose is macOS-flavored (`~` a home
## directory) and doesn't hold on Windows, where `get_user_data_dir()` is
## already the cross-platform-correct answer (`%APPDATA%\Godot\app_userdata\
## Cartalith` or similar). Read as directive intent ("four separate,
## sensible, per-purpose roots"), not as literal paths to reproduce.

const CONFIG_PATH := "user://cartalith_settings.cfg"
const _SEC_ROOTS := "storage_roots"
const _SEC_RECENT := "recent"
const MAX_RECENT := 10

## Order matches §2.1's own listing.
const ROOT_KEYS: Array[String] = ["projects", "atlas_cache", "asset_packs", "exports"]
const ROOT_LABELS := {
	"projects": "Projects",
	"atlas_cache": "Tile atlas cache",
	"asset_packs": "Asset packs",
	"exports": "Exports",
}

static var _cfg: ConfigFile
static var _loaded := false

static func _ensure_loaded() -> void:
	if _loaded:
		return
	_loaded = true
	_cfg = ConfigFile.new()
	## A missing/corrupt file is the expected first-run state, not an error
	## worth surfacing -- every read below falls back to `_default_root`.
	_cfg.load(CONFIG_PATH)

static func _save() -> void:
	_cfg.save(CONFIG_PATH)

static func _default_root(key: String) -> String:
	var base := OS.get_user_data_dir()
	match key:
		"projects": return base.path_join("Worlds")
		"atlas_cache": return base.path_join("Cache/atlas")
		"asset_packs": return base.path_join("Packs")
		"exports": return base.path_join("Exports")
		_: return base

# -- Storage roots --------------------------------------------------------------

static func storage_root(key: String) -> String:
	_ensure_loaded()
	return String(_cfg.get_value(_SEC_ROOTS, key, _default_root(key)))

static func set_storage_root(key: String, path: String) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_ROOTS, key, path)
	_save()

static func all_roots() -> Dictionary:
	var out := {}
	for k in ROOT_KEYS:
		out[k] = storage_root(k)
	return out

# -- Recent projects (§2.1: "last 10 projects", path as secondary text) --------

static func recent_projects() -> Array:
	_ensure_loaded()
	var raw = _cfg.get_value(_SEC_RECENT, "paths", [])
	var out: Array = []
	for p in raw:
		out.append(String(p))
	return out

## Moves an already-present path to the front instead of duplicating it, caps
## at `MAX_RECENT`. Called once per successful `load_save`.
static func remember_project(path: String) -> void:
	_ensure_loaded()
	var list := recent_projects()
	list.erase(path)
	list.push_front(path)
	if list.size() > MAX_RECENT:
		list.resize(MAX_RECENT)
	_cfg.set_value(_SEC_RECENT, "paths", list)
	_save()
