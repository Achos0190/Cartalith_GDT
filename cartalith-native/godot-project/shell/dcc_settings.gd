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

## The one `ConfigFile` every setting here lives in. `user://` is per-install
## machine state, so nothing in it is ever written into a world `.zip`.
const CONFIG_PATH := "user://cartalith_settings.cfg"
## Section holding the four storage roots (`ROOT_KEYS`), one key each.
const _SEC_ROOTS := "storage_roots"
## Section holding the recent-projects list (`paths`, newest first, capped
## at `MAX_RECENT`).
const _SEC_RECENT := "recent"
## `DCC_SHELL_SPEC.md` §2.5's Performance group -- the four multi-GPU
## settings (`GUI_GAP_REGISTER.md` PR-01/PR-02/PR-04/PR-05). Machine state,
## not world state: it belongs here rather than in a `.zip`, because a device
## key names hardware this machine has and the next one may not.
const _SEC_GPU := "gpu"
## `DCC_SHELL_SPEC.md` §2.5's Performance group -- the Rayon worker-thread
## count (owner ruling, `LARGE_ITEM_RULINGS.md` "CPU worker threads"). Machine
## state for the same reason `_SEC_GPU` above is: how many workers *this
## machine's* pool should span says nothing about the world, and a `.zip`
## carrying it would impose one user's core count on everyone who opens it.
const _SEC_CPU := "cpu"
## `DCC_SHELL_SPEC.md` §2.1's Autosave toggle (`GUI_GAP_REGISTER.md` FI-01).
const _SEC_AUTOSAVE := "autosave"
## Section for the §2.5 Tiles & LOD preferences: `bake_depth` and `auto_on_zoom`.
const _SEC_LOD := "tiles_lod"
## Section for the project-level lighting-rig defaults (`LIGHTING_DEFAULTS`
## keys). A key exists only once the user stored it; see `appearance_defaults()`.
const _SEC_LIGHT := "lighting"
## `DCC_SHELL_SPEC.md` §2.5's Theme radio (`BUILD_ANSWERS.md:98-99`: "Device,
## theme and units persist ... and restore on load"). Machine state for the
## same reason the GPU block is: which palette *this install* boots into says
## nothing about the world, and a `.zip` carrying it would impose one user's
## eyes on everyone who opens the file.
const _SEC_THEME := "theme"
## `DCC_SHELL_SPEC.md` §2.5's Units row (`GUI_GAP_REGISTER.md` §10 / PR-15,
## `OUTSTANDING_WORK.md`). Machine state for the same reason theme is:
## `#calUnitSeg` is display-only in the reference (13711-13713) and stays that
## way here -- see `dcc_units.gd`'s own header.
const _SEC_UNITS := "units"
## §2.5's Graphics group. Today one key -- the project-level relief
## exaggeration a fresh Generate starts from; see `appearance_defaults()`.
const _SEC_GRAPHICS := "graphics"
## §2.6's `Save layout as...` -- the named list, and nothing else: the shell's
## live layout is not mirrored here, only the snapshots a user asked to keep.
const _SEC_LAYOUT := "layout"
## §2.5 Tiles & LOD > Atlas cache > Size cap.
const _SEC_ATLAS := "atlas"
## Section for tool-window sizes, keyed by `DccWidgets.window_key()`; see
## `window_size()`.
const _SEC_WINDOWS := "window_sizes"
## CM-4 (`MAP_CONTEXT_SCOPE.md` §7.1) -- the tablet's dominant-hand preference.
const _SEC_HANDEDNESS := "handedness"
## 3D-1 (`3D_VIEWPORT_SCOPE.md` §4.2): the 3D capability gate's measured result
## and the user's forced tier. Machine state: it names this device's GPU.
const _SEC_VIEW3D := "view3d"
## The landmark filter and zoom law (owner, 2026-10-19; `ViewportHost`'s
## "landmark filter" block): the master switch, the hidden kinds and the law.
## Viewing preference, so install state like the theme -- never a world's.
const _SEC_LANDMARKS := "landmark_layer"
## The Asset library's favourites and recently used slots (AP-9,
## `ASSET_PACK_TOOLS_PROPOSAL.md` gap 15). Browse conveniences of this
## install, like the recent-projects list, so they persist across a restart
## and never travel in a world or a pack (Ruling F keeps packs as they are).
const _SEC_ASSET_LIBRARY := "asset_library"
## The optional LLM assistant's settings (`LLM_ASSISTANT_SCOPE.md` s.5, milestone
## LLM-2). **Holds no secret, ever**: see `LLM_DEFAULTS` and `set_llm_value()`.
## Machine state like the rest of this file, never a world's: a `.zip` carrying
## "the assistant is on, talking to this URL" would switch the feature on for
## whoever opens the file.
const _SEC_LLM := "llm"
## How many recently used slots are kept. **A labelled judgement**: one
## phone screen of the two-column slot grid holds about a dozen tiles.
const MAX_RECENT_ASSETS := 12
## How many recent projects are kept (`DCC_SHELL_SPEC.md` §2.1: "last 10
## projects"); `remember_project()` truncates to it.
const MAX_RECENT := 10

## Order matches §2.1's own listing.
const ROOT_KEYS: Array[String] = ["projects", "atlas_cache", "asset_packs", "exports"]
## Display name for each `ROOT_KEYS` entry, as the Storage locations modal and
## the File menu readout label the rows.
const ROOT_LABELS := {
	"projects": "Projects",
	"atlas_cache": "Tile atlas cache",
	"asset_packs": "Asset packs",
	"exports": "Exports",
}

## The loaded `ConfigFile`. Null until `_ensure_loaded()` runs; every public
## reader and writer calls that first, so nothing reads it directly.
static var _cfg: ConfigFile
## Whether `_ensure_loaded()` has already run, so the file is read once per
## session and not on every accessor call.
static var _loaded := false

## Lazy one-time load of `CONFIG_PATH` into `_cfg`. Must be the first line of
## every accessor that touches `_cfg`; never reloads, so an external edit to the
## file is not seen until the next launch.
static func _ensure_loaded() -> void:
	if _loaded:
		return
	_loaded = true
	_cfg = ConfigFile.new()
	## A missing/corrupt file is the expected first-run state, not an error
	## worth surfacing -- every read below falls back to `_default_root`.
	_cfg.load(CONFIG_PATH)

## Writes `_cfg` back to `CONFIG_PATH`. Callers must have run
## `_ensure_loaded()` first (`_cfg` is null otherwise). The save result is not
## checked, so an unwritable `user://` fails silently (intent unverified).
static func _save() -> void:
	_cfg.save(CONFIG_PATH)

## Where a storage root points until the user changes it: a fixed
## sub-folder of `OS.get_user_data_dir()` per key, or the bare data dir for an
## unknown key (never an error).
static func _default_root(key: String) -> String:
	var base := OS.get_user_data_dir()
	match key:
		"projects": return base.path_join("Worlds")
		"atlas_cache": return base.path_join("Cache/atlas")
		"asset_packs": return base.path_join("Packs")
		"exports": return base.path_join("Exports")
		_: return base

# -- Storage roots --------------------------------------------------------------

## The path of one storage root (`ROOT_KEYS` entry): the stored value, else
## `_default_root(key)`.
static func storage_root(key: String) -> String:
	_ensure_loaded()
	return String(_cfg.get_value(_SEC_ROOTS, key, _default_root(key)))

## Stores a new path for one storage root and saves at once. Does not create
## the folder or validate `path`.
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

## The recent-projects list, newest first, as plain Strings (a missing key is
## an empty list). Entries are not checked for existence here.
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

## Stores the selected GPU device keys (see `gpu_devices()`; empty means
## automatic) and saves at once.
static func set_gpu_devices(keys: PackedStringArray) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GPU, "devices", keys)
	_save()

## `"single_device"` / `"split_tiles"` / `"alternate_frames"`. Empty string
## means "never set", which the bridge treats as "leave the engine default".
static func gpu_mode() -> String:
	_ensure_loaded()
	return String(_cfg.get_value(_SEC_GPU, "mode", ""))

## Stores the multi-GPU dispatch mode string (see `gpu_mode()`) and saves.
## The value is not validated here.
static func set_gpu_mode(mode: String) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GPU, "mode", mode)
	_save()

## GB, `0` for no cap (the default -- see the engine's own note on why §2.5's
## "75 % of the smallest active device" is not implementable).
static func gpu_vram_budget_gb() -> float:
	_ensure_loaded()
	return float(_cfg.get_value(_SEC_GPU, "vram_budget_gb", 0.0))

## Stores the GPU VRAM budget in GB (`0` = no cap, see
## `gpu_vram_budget_gb()`) and saves. Not clamped here.
static func set_gpu_vram_budget_gb(gb: float) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GPU, "vram_budget_gb", gb)
	_save()

## `"cpu_tile_pass"` / `"reduce_working_res"` / `"fail_with_error"`.
static func gpu_fallback() -> String:
	_ensure_loaded()
	return String(_cfg.get_value(_SEC_GPU, "fallback", ""))

## Stores the GPU-failure fallback name (see `gpu_fallback()`) and saves. The
## value is not validated here.
static func set_gpu_fallback(name: String) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_GPU, "fallback", name)
	_save()

# -- CPU worker threads (§2.5 Performance) -------------------------------------

## Rayon global-pool worker count, or `0` for "follow the engine's own
## default" (`WorldGen.cpu_thread_count`'s exact sentinel -- see its own doc
## comment). SS2.5 asks for "an integer from 1 to the logical core count", so
## `0` is never a value a chooser can land on, the same sentinel
## `gpu_vram_budget_gb`'s "no cap" above uses.
##
## **`0` is what an untouched install returns**, and the engine's own default
## at `0` is whatever Rayon would have picked with nobody calling
## `ThreadPoolBuilder` at all -- unset therefore changes nothing for a session
## that has never opened this preference, `appearance_defaults()`'s same rule.
static func cpu_thread_count() -> int:
	_ensure_loaded()
	return maxi(0, int(_cfg.get_value(_SEC_CPU, "threads", 0)))

## Stores the Rayon worker count (see `cpu_thread_count()`), floored at `0`
## (the "engine default" sentinel), and saves. Persistence only: its caller in
## `engine_bridge.gd` also applies the value to the engine, and does that
## separately.
static func set_cpu_thread_count(threads: int) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_CPU, "threads", maxi(0, threads))
	_save()

# -- Autosave (§2.1 File) ------------------------------------------------------

## Machine state, not world state, for the same reason the GPU block above is:
## how often *this install* writes a backup says nothing about the world, and
## a `.zip` carrying it would impose one user's habit on everyone who opens
## the file.
##
## **On by default** (`design/dcc-environment-2026-08-31/spec/03-menu-bar.md`
## §6.1 row 7: *"on = `prefs.autosave` (**default `true`**)"*, and the DCC Shell
## canvas draws the File menu's interval row as `every 5 min · on`). It shipped
## `false` until 2026-09-05, argued as "a background writer that starts without
## being asked is the wrong first impression for a tool that writes hundreds of
## megabytes per save" -- which was an argument against the design rather than a
## reading of it, and it is also not what this writer does:
## `DccApp._autosave_tick()` writes nothing at all until a project has been
## saved once (`current_project_path`), the world is dirty and no generate is in
## flight, and then writes *beside* the project, never over it.
##
## **What the flip changes, deliberately narrowed to one case.** `get_value`
## returns the stored value whenever the key exists, so moving this argument
## moves the **absent key only**: an install whose owner ever touched
## `File ▸ Autosave` or the interval submenu has an explicit `enabled` in
## `user://cartalith_settings.cfg` and keeps it, off included.
## `set_autosave_enabled()` below and `set_autosave_minutes()` are the only two
## writers of this section (grepped across `godot-project/`), and nothing calls
## either on boot -- so no start-up path can freeze the old default into a file
## and disguise it as a choice.
## A tool window's last size, keyed by `DccWidgets.window_key()` (its script
## and the density it was used at). `Vector2i.ZERO` means none stored -- no
## window has a zero size, so it cannot be mistaken for one. Written only by
## `set_window_size()`, from `DccWidgets.phone_window()`'s close hook.
static func window_size(key: String) -> Vector2i:
	_ensure_loaded()
	var v = _cfg.get_value(_SEC_WINDOWS, key, Vector2i.ZERO)
	return v if v is Vector2i else Vector2i.ZERO

## Stores a tool window's size under `key` and saves. Its caller
## (`DccWidgets.phone_window()`'s close hook) must not pass `Vector2i.ZERO`,
## which `window_size()` reads as "none stored".
static func set_window_size(key: String, size: Vector2i) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_WINDOWS, key, size)
	_save()

## Whether autosave is on; true when the key was never written. The rationale
## for that default is the comment block above `window_size()` (it was left
## there when this block was last edited, so it reads as `window_size()`'s).
static func autosave_enabled() -> bool:
	_ensure_loaded()
	return bool(_cfg.get_value(_SEC_AUTOSAVE, "enabled", true))

## Stores the autosave toggle and saves. One of only two writers of this
## section (with `set_autosave_minutes()`), so an absent key always means "the
## user never chose".
static func set_autosave_enabled(on: bool) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_AUTOSAVE, "enabled", on)
	_save()

## Minutes between autosaves. Floored at 1 so a corrupt config cannot ask for
## a save every frame.
##
## **5, the design's own default** (`03-menu-bar.md` §6.1 row 8: *"`off` · `1
## min` · `5 min` · `15 min` -- selected = `prefs.autoInt` (**default `5
## min`**)"*). It was 10 until 2026-09-05, which was not one of the four values
## the interval submenu offers -- so a fresh install opened `File ▸ Autosave
## interval` and found every radio row unchecked, reporting a number no control
## on that menu could have set. `DccMenus.AUTOSAVE_MINUTES` is `[1, 5, 15]` and
## was always right; this argument was the half that disagreed with it.
##
## The same absent-key-only rule `autosave_enabled()` above states: a stored
## `minutes` survives untouched. Only `set_autosave_minutes()` writes it, and
## only from `DccMenus._on_autosave_interval()`, so an existing value can only
## be one the user picked off that submenu -- or a hand-edited one, which the
## unchecked-row path below still reports rather than rounding into a neighbour.
static func autosave_minutes() -> int:
	_ensure_loaded()
	return maxi(1, int(_cfg.get_value(_SEC_AUTOSAVE, "minutes", 5)))

## Stores the autosave interval, floored at 1 minute, and saves. Its only
## caller is `DccMenus._on_autosave_interval()`.
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

## Stores the Bake ALL depth, clamped 0..8 like `bake_depth()`, and saves.
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

## Stores the Tiled-LOD auto/manual choice (see `lod_auto()`) and saves.
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

## Stores one lighting-rig default and saves. A key outside
## `LIGHTING_DEFAULTS` is silently ignored, so a typo cannot create a stray
## stored key that `appearance_defaults()` would never send. The value is not
## range-clamped here.
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

# -- CM-4 tablet dominant-hand preference --------------------------------------
##
## `MAP_CONTEXT_SCOPE.md` §7.1: "The card docks on the side *away from the
## dominant hand* (a preference, default right-handed -> card on the left)".
## Machine state, not world state, for the same reason theme/units below are:
## which hand the person using THIS device favours says nothing about the
## world a `.zip` carries, and belongs with the rest of this file's per-install
## preferences rather than in a save. `context_broker.gd`'s tablet touch-hold
## dock-side choice is the one reader; `menus.gd`'s `Preferences ▸ Handedness`
## (`_build_handedness_submenu`) is the writer -- the CM-4 residual
## `OUTSTANDING_WORK.md` left, closed alongside the touch ring-radius scaling.
## The two accepted dominant-hand values; anything else read back or offered
## to `set_dominant_hand()` is rejected.
const HANDS: Array[String] = ["left", "right"]

## The tablet's dominant hand, `"left"` or `"right"`; unset or corrupt reads
## as `"right"` (`MAP_CONTEXT_SCOPE.md` §7.1's default).
static func dominant_hand() -> String:
	_ensure_loaded()
	var h := String(_cfg.get_value(_SEC_HANDEDNESS, "hand", "right"))
	return h if HANDS.has(h) else "right"

## Stores the dominant hand and saves. An unknown value is ignored, not stored.
static func set_dominant_hand(hand: String) -> void:
	if not HANDS.has(hand):
		return
	_ensure_loaded()
	_cfg.set_value(_SEC_HANDEDNESS, "hand", hand)
	_save()

# -- §2.5 Application > Theme --------------------------------------------------

## `dark` / `light` / `system`, `Preferences > Theme`'s own three rows.
##
## `system` is stored as the *mode*, not as the palette it resolved to:
## §2.5 calls Follow system a one-shot resolve rather than a live
## subscription, and storing the resolved bit instead would silently demote the
## choice to whichever palette the OS happened to be in the day it was picked.
const THEME_MODES: Array[String] = ["dark", "light", "system"]

## The stored Theme choice, one of `THEME_MODES`; unset or corrupt reads as
## `"dark"`. The mode, not the resolved palette (see `THEME_MODES`).
static func theme_mode() -> String:
	_ensure_loaded()
	var m := String(_cfg.get_value(_SEC_THEME, "mode", "dark"))
	return m if THEME_MODES.has(m) else "dark"

## Stores the Theme mode and saves. An unknown value is ignored. Does not
## re-theme the live shell; that is `DccTheme`'s job.
static func set_theme_mode(mode: String) -> void:
	if not THEME_MODES.has(mode):
		return
	_ensure_loaded()
	_cfg.set_value(_SEC_THEME, "mode", mode)
	_save()

# -- §2.5 Application > Units --------------------------------------------------

## `dcc_units.gd`'s `DccUnits` is the formatter; this is only the persisted
## choice, the same split `theme_mode()`/`DccTheme` above has.
##
## `DCC_SHELL_SPEC.md` §2.5's own table offers two (`km · mi`, `#calUnitSeg`);
## a third is here because the owner's ruling on `OUTSTANDING_WORK.md`'s
## PR-15 (`LARGE_ITEM_RULINGS.md`: "Build, and add nautical miles") asked for
## it outright -- a deviation from the spec's own row, recorded rather than
## made silently (`CLAUDE.md`).
const UNIT_MODES: Array[String] = ["km", "mi", "nmi"]

## The stored distance-unit mode, one of `UNIT_MODES`; unset or corrupt reads
## as `"km"`.
static func units_mode() -> String:
	_ensure_loaded()
	var m := String(_cfg.get_value(_SEC_UNITS, "mode", "km"))
	return m if UNIT_MODES.has(m) else "km"

## Stores the unit mode and saves. An unknown value is ignored. Does not
## re-render any readout; `DccUnits` formats them.
static func set_units_mode(mode: String) -> void:
	if not UNIT_MODES.has(mode):
		return
	_ensure_loaded()
	_cfg.set_value(_SEC_UNITS, "mode", mode)
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

## Whether the user has stored a relief-exaggeration default, so "unset" (the
## engine's own value) can be told from a chosen rung.
static func has_relief_exaggeration_default() -> bool:
	_ensure_loaded()
	return _cfg.has_section_key(_SEC_GRAPHICS, "exag")

## The stored relief-exaggeration default, or `RELIEF_EXAG_ENGINE_DEFAULT`
## when none was stored. Check `has_relief_exaggeration_default()` to know
## which of the two this is.
static func relief_exaggeration_default() -> float:
	_ensure_loaded()
	return float(_cfg.get_value(_SEC_GRAPHICS, "exag", RELIEF_EXAG_ENGINE_DEFAULT))

## Stores the relief-exaggeration default, clamped 0..12 (`render.rs`'s
## tunable range, per the comment on `RELIEF_EXAG_CHOICES`), and saves.
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

# -- 3D capability gate (3D_VIEWPORT_SCOPE.md §4.2, milestone 3D-1) -------------

## The gate's last measured result (`View3DGate.run()`'s dictionary: the
## identity it was measured against, every control and workload number, and
## the classified tier), or an empty Dictionary when this install never ran
## it. Machine state for `_SEC_GPU`'s reason: it describes this device's GPU
## and driver, never a world, so it never enters a `.zip`.
## Empty means "never tested", and callers must read it that way: an absent
## result is not Off (`View3DGate.disclosure()` words the two differently).
static func view3d_result() -> Dictionary:
	_ensure_loaded()
	var v = _cfg.get_value(_SEC_VIEW3D, "result", {})
	return v if v is Dictionary else {}

## Stores a measured gate result and saves. Only `View3D` writes it, after a
## probe run; nothing writes a hand-made tier here.
static func set_view3d_result(r: Dictionary) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_VIEW3D, "result", r)
	_save()

## The tier the user forced (§4.4, `View3DGate.TIER_*`), or -1 for "use the
## measured tier". Stored separately from the result so a re-test never
## silently clears the user's choice, and a forced tier never overwrites what
## was measured.
static func view3d_forced_tier() -> int:
	_ensure_loaded()
	return int(_cfg.get_value(_SEC_VIEW3D, "forced", -1))

## Stores the forced tier (-1 clears it, back to the measured tier) and saves.
static func set_view3d_forced_tier(tier: int) -> void:
	_ensure_loaded()
	if tier < 0:
		if _cfg.has_section_key(_SEC_VIEW3D, "forced"):
			_cfg.erase_section_key(_SEC_VIEW3D, "forced")
	else:
		_cfg.set_value(_SEC_VIEW3D, "forced", tier)
	_save()

# -- The landmark filter and zoom law (owner, 2026-10-19) ----------------------
#
# Each value is stored only once the user changed it; `has_*` says whether it
# was, so a reader that finds nothing keeps the overlay's own default instead of
# a default restated here (the overlay is the one place those live).

## Whether the user has ever set the master Landmarks switch.
static func has_landmarks_visible() -> bool:
	_ensure_loaded()
	return _cfg.has_section_key(_SEC_LANDMARKS, "visible")

## The stored master switch. Only meaningful when `has_landmarks_visible()`.
static func landmarks_visible() -> bool:
	_ensure_loaded()
	return bool(_cfg.get_value(_SEC_LANDMARKS, "visible", true))

## Stores the master switch and saves.
static func set_landmarks_visible(on: bool) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_LANDMARKS, "visible", on)
	_save()

## The hidden landmark kinds (engine kind keys and `map_overlay.gd::LANDMARK_MANUAL_POI`).
## Empty when none is hidden or nothing was ever stored -- the same answer, since
## an empty hidden set IS the default. A malformed value reads as empty.
static func hidden_landmark_kinds() -> PackedStringArray:
	_ensure_loaded()
	var v = _cfg.get_value(_SEC_LANDMARKS, "hidden_kinds", PackedStringArray())
	if v is PackedStringArray:
		return v
	if v is Array:
		return PackedStringArray(v)
	return PackedStringArray()

## Stores the hidden set and saves. An empty set erases the key rather than
## storing `[]`, so "never changed" and "changed back" leave the file alike.
static func set_hidden_landmark_kinds(kinds: PackedStringArray) -> void:
	_ensure_loaded()
	if kinds.is_empty():
		if _cfg.has_section_key(_SEC_LANDMARKS, "hidden_kinds"):
			_cfg.erase_section_key(_SEC_LANDMARKS, "hidden_kinds")
	else:
		_cfg.set_value(_SEC_LANDMARKS, "hidden_kinds", kinds)
	_save()

## Whether the user has ever set the landmark zoom law switch.
static func has_landmark_zoom_law() -> bool:
	_ensure_loaded()
	return _cfg.has_section_key(_SEC_LANDMARKS, "zoom_law")

## The stored zoom law switch. Only meaningful when `has_landmark_zoom_law()`.
static func landmark_zoom_law() -> bool:
	_ensure_loaded()
	return bool(_cfg.get_value(_SEC_LANDMARKS, "zoom_law", true))

## Stores the zoom law switch and saves.
static func set_landmark_zoom_law(on: bool) -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_LANDMARKS, "zoom_law", on)
	_save()

# -- §2.5 Tiles & LOD > Atlas cache > Size cap ---------------------------------

## GB, `0` for no cap -- the default, and what the atlas did before a cap
## existed. A ladder rather than a spinner for `GPU_VRAM_CHOICES`' reason: the
## number only ever gates whole baked chunks, so a free-form field would offer
## a precision the decision does not have.
const ATLAS_CAP_CHOICES: Array[float] = [0.0, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0]

## The tile-atlas cache size cap in GB; `0` (the default) means no cap.
static func atlas_cap_gb() -> float:
	_ensure_loaded()
	return maxf(0.0, float(_cfg.get_value(_SEC_ATLAS, "cap_gb", 0.0)))

## Stores the atlas cache cap in GB (negative floored to `0`) and saves.
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

## One saved layout by name, or an empty Dictionary if absent or not a
## Dictionary.
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

## Deletes a saved layout by name and saves. An unknown name does nothing
## and does not save.
static func forget_layout(name: String) -> void:
	_ensure_loaded()
	var all := layouts()
	if not all.has(name):
		return
	all.erase(name)
	_cfg.set_value(_SEC_LAYOUT, "named", all)
	_save()

## Has the first-run seeding of `DccMenus.SEED_LAYOUTS` already happened on this
## install? One boolean beside `named`, in the same section, written once.
##
## **The flag exists because "is the list empty" is the wrong question.** An
## empty list is a state the user can reach deliberately -- forget all four
## seeds and it is empty again -- and a seeder that keyed on emptiness would
## put them straight back on the next launch, which is a delete that does not
## stick. It would also mean a user who keeps exactly one layout of their own
## never gets re-seeded while the user who keeps none is re-seeded forever:
## two different behaviours from one predicate that was never about seeding.
##
## So the recorded fact is *"seeding ran"*, not *"seeds are present"*. The two
## diverge the moment a user touches the list, and only the first one is what
## "seed once" means.
static func layouts_seeded() -> bool:
	_ensure_loaded()
	return bool(_cfg.get_value(_SEC_LAYOUT, "seeded", false))

## Records that first-run layout seeding ran (see `layouts_seeded()`) and saves.
## Nothing un-sets it, which is the point.
static func mark_layouts_seeded() -> void:
	_ensure_loaded()
	_cfg.set_value(_SEC_LAYOUT, "seeded", true)
	_save()

# -- §2.5 Application > Keyboard shortcuts -------------------------------------

## Rebindable menu accelerators. `DCC_SHELL_SPEC.md` §2.5: "Keyboard
## shortcuts… | Editable table, per-context." `GUI_GAP_REGISTER.md` PR-16/
## HE-02, owner-ruled build: *"A binding table in DccSettings, applied over
## the menu accelerators at build time. Per-context, not flat -- the same key
## means different things with a tool armed."*
##
## **Per-context means one `ConfigFile` section per context**, not one flat
## section keyed only by action -- so two contexts can use the same
## accelerator for different actions without this store ever having to tell
## them apart, which is the entire reason a flat table was ruled out.
## `SHORTCUT_CONTEXT_MENU` is the only context this pass populates:
## `menus.gd`'s seven Program menus, read and reapplied through the one
## function (`_bind_accelerator`) every accelerator in that file already
## passes through, so no second mechanism grows beside it.
##
## The shell has other real keyboard contexts -- `app.gd`'s armed-tool
## Escape/Backspace/Delete dispatch, the per-domain tool buttons' own
## `Shortcut`s (`dcc_widgets.gd`'s `_tools_row`, e.g. Freehand's bare `F`),
## the Layers popover's `1`-`8` -- named because `menus.gd`'s own header
## points at them, not guessed. **None of them read this store.** Rebinding
## them needs a pass that owns those files; this section is keyed by context
## rather than assumed to be the only one that will ever exist, so that pass
## adds its own section instead of reshaping this one.
const _SEC_SHORTCUTS_PREFIX := "shortcuts_"
## The context id for the Program-menu accelerators (the only populated
## context; see the block above).
const SHORTCUT_CONTEXT_MENU := "menu"

## The `ConfigFile` section name for one shortcut context
## (`_SEC_SHORTCUTS_PREFIX` + context), so contexts never share keys.
static func _shortcuts_section(context: String) -> String:
	return _SEC_SHORTCUTS_PREFIX + context

## `default_accel` back if the user never rebound `action_id` in `context`;
## the stored override otherwise. `action_id` is the menu system's own item
## id (`menus.gd`'s `ID_*` constants) -- already a stable, collision-checked
## namespace (see that file's own header on picking a free one), so this does
## not mint a second one just to have a config key.
static func shortcut_binding(context: String, action_id: int, default_accel: int) -> int:
	_ensure_loaded()
	return int(_cfg.get_value(_shortcuts_section(context), str(action_id), default_accel))

## Whether the user has stored a rebind for this action, so a caller can show
## "not the shipped key" and a way back -- the same distinction
## `has_lighting_default()` makes and `shortcut_binding()`'s return value
## alone cannot show.
static func has_shortcut_override(context: String, action_id: int) -> bool:
	_ensure_loaded()
	return _cfg.has_section_key(_shortcuts_section(context), str(action_id))

## Stores a rebound accelerator for `action_id` in `context` and saves. `accel`
## is the menu system's accelerator int and must not be `0` (the dialog's own
## "no shortcut" spelling; see `shortcuts_dialog.gd`'s capture guard). Duplicate
## bindings are not rejected here; the dialog reports them and never blocks.
static func set_shortcut_binding(context: String, action_id: int, accel: int) -> void:
	_ensure_loaded()
	_cfg.set_value(_shortcuts_section(context), str(action_id), accel)
	_save()

## Back to the shipped default for one action -- erases the key rather than
## writing the default value over it, `clear_relief_exaggeration_default()`'s
## reason: a stored copy of a default cannot follow the default if it moves.
static func clear_shortcut_binding(context: String, action_id: int) -> void:
	_ensure_loaded()
	var section := _shortcuts_section(context)
	if _cfg.has_section_key(section, str(action_id)):
		_cfg.erase_section_key(section, str(action_id))
		_save()

## `Preferences ▸ Keyboard shortcuts… ▸ Restore all to defaults`, one context
## at a time -- never every context in the file, so resetting the menu table
## can never reach into a different context's store once one exists.
static func reset_shortcuts(context: String) -> void:
	_ensure_loaded()
	var section := _shortcuts_section(context)
	if _cfg.has_section(section):
		_cfg.erase_section(section)
		_save()

# -- The optional LLM assistant (LLM_ASSISTANT_SCOPE.md s.5, milestone LLM-2) --------

## Every `[llm]` key that may exist, with its shipped default. **This table is the
## whole allow-list**: `set_llm_value()` refuses any other key, so no code path can
## write an API key (or anything else) into `cartalith_settings.cfg` by typing a new
## key name. A test (`_llmset_probe.gd`) reads the file back and greps it for the key
## it just entered. The key's own home is `WorldGen.llm_key_save` (OS credential store)
## or an environment variable the user set; neither is this file.
##
## **Defaults are the scope's s.5 table: everything off, nothing chosen.** An empty
## `preset_id`, `base_url` or `model` is "not set" and the gate refuses it; there is no
## default provider, address or model name.
const LLM_DEFAULTS := {
	"enabled": false,          ## master switch; off means zero sockets
	"prose": false,            ## Prose and Vault assist (LLM-3)
	"commands": false,         ## view / layer / parameter commands (LLM-5, LLM-6)
	"edit": false,             ## sculpt / paint edit actions (LLM-5, LLM-7); its own switch
	"local_allowed": false,    ## localhost and LAN endpoints
	"remote_allowed": false,   ## hosted endpoints
	"preset_id": "",           ## a `WorldGen.llm_presets()` id, or ""
	"base_url": "",
	"model": "",               ## free text, never a list
	"key_source": "env",       ## "env" or "store"; never a file
	"env_var": "CART_LLM_KEY", ## the variable NAME, not a key
	"token_counter": true,     ## visible by default; no cap is enforced (Ruling DD Q8)
	"gpu_override": "automatic", ## Ruling DF: automatic / always / never
	"ctx_entity": false,       ## context source: current entity
	"ctx_selection": false,    ## context source: selection
	"ctx_notes": false,        ## context source: named Vault notes
}

## The stored value of one `[llm]` key, else its `LLM_DEFAULTS` entry. An unknown
## key is a bug in the caller and returns `null` so it cannot be mistaken for a
## value. A stored value of the wrong type reads as the default (a hand-edited file
## can make the feature more closed, never more open: every boolean defaults off
## except the display-only `token_counter`).
static func llm_value(key: String):
	if not LLM_DEFAULTS.has(key):
		return null
	_ensure_loaded()
	var dflt = LLM_DEFAULTS[key]
	var v = _cfg.get_value(_SEC_LLM, key, dflt)
	return v if typeof(v) == typeof(dflt) else dflt

## Stores one `[llm]` key and saves. **Refuses a key not in `LLM_DEFAULTS` or a value
## of the wrong type** (returns false, writes nothing): that refusal is what keeps a
## secret out of the file. Text is stored trimmed.
static func set_llm_value(key: String, value) -> bool:
	if not LLM_DEFAULTS.has(key) or typeof(value) != typeof(LLM_DEFAULTS[key]):
		return false
	_ensure_loaded()
	_cfg.set_value(_SEC_LLM, key, String(value).strip_edges() if value is String else value)
	_save()
	return true

## The dictionary `WorldGen.llm_authorize` reads: only the gate-relevant keys.
static func llm_gate_dict() -> Dictionary:
	var d := {}
	for k in ["enabled", "prose", "commands", "edit", "local_allowed", "remote_allowed",
			"preset_id", "base_url", "model", "key_source", "env_var"]:
		d[k] = llm_value(k)
	return d

## The origins the user has consented to (`scheme://host[:port]`, from
## `WorldGen.llm_endpoint_info`). Consent is per endpoint (scope LLM-2: "appears before
## the first call to each new endpoint"). Not secret.
static func llm_consented_origins() -> PackedStringArray:
	_ensure_loaded()
	return _string_list(_cfg.get_value(_SEC_LLM, "consented_origins", PackedStringArray()))

## Records consent for `origin` (never duplicating it) and saves. An empty origin is
## ignored: consent with nothing named is not consent.
static func llm_add_consent(origin: String) -> void:
	if origin == "":
		return
	var list := llm_consented_origins()
	if list.has(origin):
		return
	list.append(origin)
	_ensure_loaded()
	_cfg.set_value(_SEC_LLM, "consented_origins", list)
	_save()

## Withdraws consent for `origin`, or for every endpoint when `origin` is `""`. The
## next request to a withdrawn endpoint asks again.
static func llm_withdraw_consent(origin: String = "") -> void:
	_ensure_loaded()
	var list := llm_consented_origins()
	if origin == "":
		list = PackedStringArray()
	elif list.has(origin):
		list.remove_at(list.find(origin))
	else:
		return
	if list.is_empty():
		if _cfg.has_section_key(_SEC_LLM, "consented_origins"):
			_cfg.erase_section_key(_SEC_LLM, "consented_origins")
	else:
		_cfg.set_value(_SEC_LLM, "consented_origins", list)
	_save()

# -- Asset library favourites and recent (AP-9) ---------------------------------

## The favourite slot uids, in the order they were starred (a missing or
## malformed value is an empty list). A uid is kept even when this session's
## library does not hold that slot (a custom slot from a pack not open now);
## the library window shows only the ones that resolve and counts the rest.
static func asset_favourites() -> PackedStringArray:
	_ensure_loaded()
	return _string_list(_cfg.get_value(_SEC_ASSET_LIBRARY, "favourites", PackedStringArray()))

## Stars or unstars `uid` and saves at once. Answers the new state.
static func toggle_asset_favourite(uid: String) -> bool:
	_ensure_loaded()
	var list := asset_favourites()
	var on := not list.has(uid)
	if on:
		list.append(uid)
	else:
		list.remove_at(list.find(uid))
	_cfg.set_value(_SEC_ASSET_LIBRARY, "favourites", list)
	_save()
	return on

## The recently used slot uids, newest first, at most `MAX_RECENT_ASSETS`.
static func asset_recent() -> PackedStringArray:
	_ensure_loaded()
	return _string_list(_cfg.get_value(_SEC_ASSET_LIBRARY, "recent", PackedStringArray()))

## Moves `uid` to the front of the recent list (never duplicating it), caps
## the list at `MAX_RECENT_ASSETS` and saves. An empty uid is ignored.
static func remember_asset(uid: String) -> void:
	if uid == "":
		return
	_ensure_loaded()
	var list := asset_recent()
	var at := list.find(uid)
	if at == 0:
		return
	if at > 0:
		list.remove_at(at)
	list.insert(0, uid)
	if list.size() > MAX_RECENT_ASSETS:
		list.resize(MAX_RECENT_ASSETS)
	_cfg.set_value(_SEC_ASSET_LIBRARY, "recent", list)
	_save()

## A stored list as a `PackedStringArray`, whichever array type `ConfigFile`
## read it back as; anything else is an empty list.
static func _string_list(v) -> PackedStringArray:
	if v is PackedStringArray:
		return v
	if v is Array:
		var out := PackedStringArray()
		for x in v:
			out.append(String(x))
		return out
	return PackedStringArray()
