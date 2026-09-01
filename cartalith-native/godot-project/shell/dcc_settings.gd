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
const _SEC_LIGHT := "lighting"
## `DCC_SHELL_SPEC.md` §2.5's Theme radio (`BUILD_ANSWERS.md:98-99`: "Device,
## theme and units persist ... and restore on load"). Machine state for the
## same reason the GPU block is: which palette *this install* boots into says
## nothing about the world, and a `.zip` carrying it would impose one user's
## eyes on everyone who opens the file.
const _SEC_THEME := "theme"
## §2.5's Graphics group. Today one key -- the project-level relief
## exaggeration a fresh Generate starts from; see `appearance_defaults()`.
const _SEC_GRAPHICS := "graphics"
## §2.6's `Save layout as...` -- the named list, and nothing else: the shell's
## live layout is not mirrored here, only the snapshots a user asked to keep.
const _SEC_LAYOUT := "layout"
## §2.5 Tiles & LOD > Atlas cache > Size cap.
const _SEC_ATLAS := "atlas"
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

## Every root at once. **No code caller** (checked repo-wide 2026-09-01),
## and kept rather than deleted for one reason: `GUI_GAP_REGISTER.md`
## cites it by name as the evidence that the four storage roots are real.
##
## That citation is already one step off -- both surfaces that show the
## roots (`app.gd`'s Storage locations modal and `menus.gd`'s File-menu
## readout rows) walk `ROOT_KEYS` and call `storage_root(k)` per row,
## because each needs the KEY as well as the path: one to label the row,
## the other to hand `_browse_root()` something to write back. A Dictionary
## of key->path would serve either of them no better, so this is surplus
## rather than the missing link it is cited as. Deleting it is the right
## call the moment that document is corrected; deleting it now would leave
## the document pointing at nothing.
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

## §2.5's "Tiled LOD — `auto on zoom` (default) · `manual`", the reference's
## `state.lodAuto` (which its own save format carries, defaulting true).
## Persisted here rather than on the world because it is a preference about how
## the viewer behaves, not a property of the map -- §2.5 files it under
## Preferences and so does this.
static func lod_auto() -> bool:
	_ensure_loaded()
	return bool(_cfg.get_value(_SEC_LOD, "auto_on_zoom", true))

static func set_lod_auto(on: bool) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_LOD, "auto_on_zoom", on)
	_save()

# -- §2.5 Graphics ▸ Lighting rig defaults ------------------------------------

## §2.5 asks for "Azimuth, elevation, ambient, multidirectional on/off";
## §7's Layer properties LIGHT group draws the same rig per layer at azimuth
## 315°, elevation 45°, strength 0.62, multidirectional 8 lights.
##
## Both describe **one rig**, and the engine already binds every value of it as
## a `render.rs` tunable (`sun_az_deg` 0-360, `sun_alt_deg` 5-85,
## `relief_ambient` 0-1, `relief_lights` 1-12). What was missing was never
## rendering — it was the project-level DEFAULT those per-world values start
## from, which is a settings key.
##
## The four values below are the reference HTML's own rig -- 315° the
## cartographic convention (lighting from the south-east makes ridges read as
## valleys, as `render_workspace.gd`'s own tooltip says), 45° its companion,
## and `relief_lights` 1 its exact single-sun shading.
##
## **They are the ladder's fallback labels, not values this file sends**
## (corrected 2026-09-01). Until then `appearance_defaults()` emitted all four
## unconditionally, so a fresh install overwrote the engine's own tier on every
## Generate: `render.rs`'s `TerrainAppearance::default()` is `sun_alt_deg` 40,
## `relief_ambient` 0.34 and `relief_lights` **6** (10 on Ultra), and this dict
## shipped 45 / 0.35 / **1** over it -- turning the multi-light relief rig off
## on a machine whose owner had never opened this menu, while the comment here
## claimed the opposite ("renders exactly as it did before this key existed").
## `appearance_defaults()` now sends a key only once the user has stored one,
## which is what that sentence always promised.
const LIGHTING_DEFAULTS := {
	"sun_az_deg": 315.0,
	"sun_alt_deg": 45.0,
	"relief_ambient": 0.35,
	"relief_lights": 1.0,
}

## **Every project-level appearance default, not only the rig.** `set_appearance
## ()` takes any `render.rs` tunable, and the one caller -- `app.gd`'s
## `_apply_lighting_defaults()` -- hands it whatever this returns, so a second
## dict for the second key would have needed a second call site in a file this
## pass does not own. `exag` joins the four rig values here instead.
##
## **Every key here is one the user actually stored.** `exag` always worked
## that way (see `relief_exaggeration_default()`); since 2026-09-01 the four
## rig keys do too. An untouched install returns `{}`, `app.gd`'s
## `_apply_lighting_defaults()` hands the engine an empty dict, and the active
## quality tier's own rig stands -- which is what an install nobody has
## configured should render with, and what this comment used to claim while
## the loop below did the opposite.
##
## A stored key still wins over the tier, at every value including one equal to
## the tier's: choosing 6 lights explicitly and having the engine happen to
## agree are the same picture but not the same statement, and only the first
## survives a tier change.
static func appearance_defaults() -> Dictionary:
	_ensure_loaded()
	var out: Dictionary = {}
	for k in LIGHTING_DEFAULTS:
		if _cfg.has_section_key(_SEC_LIGHT, String(k)):
			out[k] = float(_cfg.get_value(_SEC_LIGHT, String(k), LIGHTING_DEFAULTS[k]))
	if has_relief_exaggeration_default():
		out["exag"] = relief_exaggeration_default()
	return out

## Whether the user has stored a rung for one rig key, so a menu can tell
## "chosen" from "following the engine" -- the distinction
## `appearance_defaults()` now makes and its return value alone cannot show.
static func has_lighting_default(key: String) -> bool:
	_ensure_loaded()
	return _cfg.has_section_key(_SEC_LIGHT, key)

## The name `app.gd`'s `_apply_lighting_defaults()` and `menus.gd`'s rig ladder
## both call. Kept as a delegate rather than renamed at those two call sites:
## one of them is another pass's file, and a rename that lands in only one of
## the two is worse than a name one word narrower than its answer.
static func lighting_defaults() -> Dictionary:
	return appearance_defaults()

static func set_lighting_default(key: String, value: float) -> void:
	if not LIGHTING_DEFAULTS.has(key):
		return
	_ensure_loaded()
	_cfg.set_value(_SEC_LIGHT, key, value)
	_save()

## Back to sending nothing. Erases the keys rather than writing values over
## them, which since 2026-09-01 is the whole act: with no stored key
## `appearance_defaults()` omits it, so the next Generate leaves the engine's
## own rig alone instead of transcribing one over it.
##
## **It does not un-send.** `set_appearance()` writes into `lib.rs`'s
## `appearance_over`, which only `reset_appearance()` and a preset load clear
## -- so a value an earlier Generate already applied is still overriding the
## tier after this runs. RENDER's `Reset to quality tier` is the control that
## takes those back; `menus.gd`'s row says so.
static func reset_lighting_defaults() -> void:
	_ensure_loaded()
	if _cfg.has_section(_SEC_LIGHT):
		_cfg.erase_section(_SEC_LIGHT)
	_save()

# -- §2.5 Application > Theme --------------------------------------------------

## `dark` / `light` / `system`, `Preferences > Theme`'s own three rows.
##
## `system` is stored as the *mode*, not as the palette it resolved to:
## §2.5 calls Follow system a one-shot resolve rather than a live
## subscription, and storing the resolved bit instead would silently demote the
## choice to whichever palette the OS happened to be in the day it was picked.
const THEME_MODES: Array[String] = ["dark", "light", "system"]

static func theme_mode() -> String:
	_ensure_loaded()
	var m := String(_cfg.get_value(_SEC_THEME, "mode", "dark"))
	return m if THEME_MODES.has(m) else "dark"

static func set_theme_mode(mode: String) -> void:
	if not THEME_MODES.has(mode):
		return
	_ensure_loaded()
	_cfg.set_value(_SEC_THEME, "mode", mode)
	_save()

# -- §2.5 Graphics > relief exaggeration default -------------------------------

## The three rungs `Preferences > Graphics` offers. `render.rs`'s own tunable
## range is 0..12; these are the round multipliers the design asks for, not the
## full span -- the per-world slider (`render_workspace.gd`'s `exag`) is where
## a value between them is chosen.
const RELIEF_EXAG_CHOICES: Array[float] = [1.0, 2.0, 4.0]
## `render.rs`'s own `exag: 3.4`, which is what an install renders with when
## this key has never been written. **Not one of the rungs**, deliberately: a
## default that silently rounded itself onto the nearest rung would change the
## shipped render the moment this preference existed, which is the opposite of
## what a default is for. Unset therefore reads as "the engine's", and the
## Graphics submenu says so on its own parent row.
const RELIEF_EXAG_ENGINE_DEFAULT := 3.4

static func has_relief_exaggeration_default() -> bool:
	_ensure_loaded()
	return _cfg.has_section_key(_SEC_GRAPHICS, "exag")

static func relief_exaggeration_default() -> float:
	_ensure_loaded()
	return float(_cfg.get_value(_SEC_GRAPHICS, "exag", RELIEF_EXAG_ENGINE_DEFAULT))

static func set_relief_exaggeration_default(v: float) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GRAPHICS, "exag", clampf(v, 0.0, 12.0))
	_save()

## Erases the key rather than writing 3.4 over it, for `reset_lighting_defaults
## ()`'s reason: a stored copy of a default is a default that cannot move.
static func clear_relief_exaggeration_default() -> void:
	_ensure_loaded()
	if _cfg.has_section_key(_SEC_GRAPHICS, "exag"):
		_cfg.erase_section_key(_SEC_GRAPHICS, "exag")
	_save()

# -- §2.5 Tiles & LOD > Atlas cache > Size cap ---------------------------------

## GB, `0` for no cap -- the default, and what the atlas did before a cap
## existed. A ladder rather than a spinner for `GPU_VRAM_CHOICES`' reason: the
## number only ever gates whole baked chunks, so a free-form field would offer
## a precision the decision does not have.
const ATLAS_CAP_CHOICES: Array[float] = [0.0, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0]

static func atlas_cap_gb() -> float:
	_ensure_loaded()
	return maxf(0.0, float(_cfg.get_value(_SEC_ATLAS, "cap_gb", 0.0)))

static func set_atlas_cap_gb(gb: float) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_ATLAS, "cap_gb", maxf(0.0, gb))
	_save()

# -- §2.6 Window > Save layout as... -------------------------------------------

## Named layout snapshots: `name -> {regions, domain, mode, rail_expanded,
## detent}`, exactly the dictionary `menus.gd` collects off the live shell.
##
## Machine state again, and more obviously so than the rest of this file: a
## layout is which docks *this screen* has room for. The built-in entry is not
## stored here at all -- `Reset layout` is code (`DccApp.toggle_region`'s
## `ID_WIN_RESET` branch), and a saved copy of it could go stale against the
## shell it resets.
static func layouts() -> Dictionary:
	_ensure_loaded()
	var raw = _cfg.get_value(_SEC_LAYOUT, "named", {})
	return raw if raw is Dictionary else {}

## Sorted, so menu order does not depend on save order -- a list that
## reshuffles itself as it grows cannot be used by muscle memory.
static func layout_names() -> Array:
	var names: Array = layouts().keys()
	names.sort()
	return names

static func layout(name: String) -> Dictionary:
	var d = layouts().get(name, {})
	return d if d is Dictionary else {}

## Overwrites a same-named entry rather than duplicating it, the way
## `remember_project()` moves a repeat instead of appending one.
static func save_layout(name: String, data: Dictionary) -> void:
	var trimmed := name.strip_edges()
	if trimmed == "":
		return
	_ensure_loaded()
	var all := layouts()
	all[trimmed] = data
	_cfg.set_value(_SEC_LAYOUT, "named", all)
	_save()

static func forget_layout(name: String) -> void:
	_ensure_loaded()
	var all := layouts()
	if not all.has(name):
		return
	all.erase(name)
	_cfg.set_value(_SEC_LAYOUT, "named", all)
	_save()
