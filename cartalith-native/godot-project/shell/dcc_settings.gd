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
## `DCC_SHELL_SPEC.md` §2.5's Performance group -- the four multi-GPU
## settings (`GUI_GAP_REGISTER.md` PR-01/PR-02/PR-04/PR-05). Machine state,
## not world state: it belongs here rather than in a `.zip`, because a device
## key names hardware this machine has and the next one may not.
const _SEC_GPU := "gpu"
## `DCC_SHELL_SPEC.md` §2.1's Autosave toggle (`GUI_GAP_REGISTER.md` FI-01).
const _SEC_AUTOSAVE := "autosave"
const _SEC_LOD := "tiles_lod"
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

# -- Multi-GPU (§2.5 Performance) ----------------------------------------------

## Selected device keys, in dispatch order. **Empty is the default and means
## "automatic"** -- not "no device". Keys are `WorldGen.gpu_enumerate_devices`'s
## own stable ids, never array indices: enumeration order is the driver's, and
## adding a GPU renumbers it.
static func gpu_devices() -> PackedStringArray:
	_ensure_loaded()
	var raw = _cfg.get_value(_SEC_GPU, "devices", PackedStringArray())
	var out := PackedStringArray()
	for k in raw:
		out.append(String(k))
	return out

static func set_gpu_devices(keys: PackedStringArray) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GPU, "devices", keys)
	_save()

## `"single_device"` / `"split_tiles"` / `"alternate_frames"`. Empty string
## means "never set", which the bridge treats as "leave the engine default".
static func gpu_mode() -> String:
	_ensure_loaded()
	return String(_cfg.get_value(_SEC_GPU, "mode", ""))

static func set_gpu_mode(mode: String) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GPU, "mode", mode)
	_save()

## GB, `0` for no cap (the default -- see the engine's own note on why §2.5's
## "75 % of the smallest active device" is not implementable).
static func gpu_vram_budget_gb() -> float:
	_ensure_loaded()
	return float(_cfg.get_value(_SEC_GPU, "vram_budget_gb", 0.0))

static func set_gpu_vram_budget_gb(gb: float) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GPU, "vram_budget_gb", gb)
	_save()

## `"cpu_tile_pass"` / `"reduce_working_res"` / `"fail_with_error"`.
static func gpu_fallback() -> String:
	_ensure_loaded()
	return String(_cfg.get_value(_SEC_GPU, "fallback", ""))

static func set_gpu_fallback(name: String) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GPU, "fallback", name)
	_save()

# -- Autosave (§2.1 File) ------------------------------------------------------

## Machine state, not world state, for the same reason the GPU block above is:
## how often *this install* writes a backup says nothing about the world, and
## a `.zip` carrying it would impose one user's habit on everyone who opens
## the file. Off by default -- a background writer that starts without being
## asked is the wrong first impression for a tool that writes hundreds of
## megabytes per save.
static func autosave_enabled() -> bool:
	_ensure_loaded()
	return bool(_cfg.get_value(_SEC_AUTOSAVE, "enabled", false))

static func set_autosave_enabled(on: bool) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_AUTOSAVE, "enabled", on)
	_save()

## Minutes between autosaves. Floored at 1 so a corrupt config cannot ask for
## a save every frame.
static func autosave_minutes() -> int:
	_ensure_loaded()
	return maxi(1, int(_cfg.get_value(_SEC_AUTOSAVE, "minutes", 10)))

static func set_autosave_minutes(minutes: int) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_AUTOSAVE, "minutes", maxi(1, minutes))
	_save()

# -- §2.5 Tiles & LOD ---------------------------------------------------------

## **How deep `Bake ALL levels & finalize` goes.**
##
## `DCC_SHELL_SPEC.md` §2.5 asks for "levels 0-8" under Preferences ▸ Tiles &
## LOD. The number was already real -- `world_workspace.gd`'s `_bake_depth`,
## which is what `bake_all()` is called with -- but it was a private field with
## no key and no accessor, so a Preferences ladder would have been a SECOND
## copy free to disagree with the dock the user actually bakes from.
## `UNWIRED_FUNCTIONS.md` named that as the blocker; this is the store both
## surfaces now read.
##
## Default 3: the reference's own `bakeAllDepth`, and 85 tiles, the deepest
## bake that finishes in a plausible interactive wait.
##
## Clamped 0..8 to §2.5's own range. The ENGINE's ceiling is higher --
## `lod_bridge::MAX_LEVEL` is 10 -- so this clamp is the spec's, not a
## capability limit, and raising it needs only this line.
static func bake_depth() -> int:
	_ensure_loaded()
	return clampi(int(_cfg.get_value(_SEC_LOD, "bake_depth", 3)), 0, 8)

static func set_bake_depth(depth: int) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_LOD, "bake_depth", clampi(depth, 0, 8))
	_save()
