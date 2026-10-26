extends AcceptDialog
class_name NewWorldDialog

## File ▸ New world… (`DCC_SHELL_SPEC.md` §2.1): name, seed, extent
## (region/world) and working resolution.
##
## **NWP-3 (`NEW_WORLD_PREVIEW_PROPOSAL.md` §2-§3, Ruling CE, 2026-10-12):** a
## two-column window on desktop and tablet -- left, a 320-cell grey-on-blue
## land/ocean preview drawn by the engine's own early stages
## (`WorldGen.preview_land_mask`, through `EngineBridge.preview_land_mask`),
## with the seed row (dice, lock) and a four-thumbnail session history under it;
## right, Extent, Size (Map width, Width, Plates, Resolution, the derived panel),
## Archetype and a **More options** disclosure holding the grid and Generation
## controls unchanged; footer Cancel / Generate world. On a phone the same parts
## stack inside §6.7's card. **The preview carries Ruling CE's disclaimer on its
## frame at every density**, and dashes with its reason -- never an ocean
## rectangle -- on an engine build without the binding. Create writes the plate
## count it shows into `tect.plates` (Q1). Everything about the preview lives in
## the "Seed preview" and "Plates" blocks below.
##
## The dimension logic here is ported from `main.gd`'s `_build_world_setup`
## rather than rewritten -- it is hard-won: cells are square in km, grid
## height is a derived readout except when hand-typed, and world mode pins
## the aspect to 2:1 because X wraps 360° of longitude over the grid width
## (`GENERATION_PARAMETERS.md` "Map dimensions and aspect ratio").
##
## Sea level, the extent flag and the four experimental toggles below are all
## real `params.rs` entries (`bridge.param_get`/`param_set`), not local state
## -- so this dialog and World ▸ Generation Pipeline's own stage rows (02
## Extent & scale, 04 Tectonics, 05 Volcanism, 08 Climate) are two views of
## the same engine state rather than two copies that can disagree. Only what
## has no engine-side storage (seed, grid dimensions, archetype name, village
## seeding) is this dialog's own.

## The engine facade, assigned by `setup()`. Every read and write of engine state goes through it; an older build may lack newer `world_gen` methods, so callers test `has_method` first.
var bridge: EngineBridge

# -- Ported constants (main.gd, verbatim) --------------------------------------

## Map-width presets (label plus width in km) for the Size picker, ported from `main.gd`'s `_build_world_setup` per the "Ported constants" header above. A hand-typed width selects `SIZE_CUSTOM_INDEX`.
const SIZE_PRESETS: Array = [
	{"label": "Local · 200 km", "km": 200.0},
	{"label": "Province · 800 km", "km": 800.0},
	{"label": "Region · 2 000 km", "km": 2000.0},
	{"label": "Subcontinent · 5 000 km", "km": 5000.0},
	{"label": "Continent · 12 000 km", "km": 12000.0},
	{"label": "Planet · 40 075 km (Earth's equator)", "km": 40075.0},
]
const SIZE_CUSTOM_INDEX := 6 ## SIZE_PRESETS.size() -- one past the last preset.

## Aspect picker entries: label and width:height ratio. The indices of the first
## three (world 2:1, reference region frame) are pinned by `ASPECT_WORLD_INDEX` and
## `ASPECT_REGION_INDEX`, so keep those entries where they are.
const ASPECT_PRESETS: Array = [
	{"label": "2:1 · equirectangular", "ratio": 2.0},
	{"label": "16:9 · widescreen", "ratio": 16.0 / 9.0},
	{"label": "1.5625:1 · reference region frame", "ratio": 1.5625},
	{"label": "4:3 · classic landscape", "ratio": 4.0 / 3.0},
	{"label": "1:1 · square", "ratio": 1.0},
	{"label": "3:4 · portrait", "ratio": 0.75},
	{"label": "9:16 · tall portrait", "ratio": 9.0 / 16.0},
]
const ASPECT_WORLD_INDEX := 0   ## 2:1 -- the shape world mode is pinned to.
const ASPECT_REGION_INDEX := 2  ## The reference's own region frame; gh comes from the engine.
## Picker index of the Custom entry: one past the last preset (7 presets, so 7).
## Selected whenever a row count is typed by hand.
const ASPECT_CUSTOM_INDEX := 7
## What the aspect picker opens on: the reference's own region frame.
const ASPECT_DEFAULT_INDEX := ASPECT_REGION_INDEX

## Grid widths in cells offered by the Resolution picker (512 to 8K).
const RESOLUTION_PRESETS: Array[int] = [512, 1024, 2048, 4096, 8192]
## Picker text for `RESOLUTION_PRESETS`, index-parallel.
const RESOLUTION_LABELS: Array[String] = ["512", "1K", "2K", "4K", "8K"]
const RESOLUTION_DEFAULT_INDEX := 2 ## 2K, the reference's own default.
## Picker index of Custom: one past the last preset (5 presets, so 5).
const RESOLUTION_CUSTOM_INDEX := 5

const GRID_MIN := 4 ## generate_sized() clamps each dimension to >= 4; match it rather than let the engine clamp behind the dialog's back.
## Largest grid dimension the form accepts, equal to the top resolution preset
## (8K). A labelled judgement matching the preset list, not an engine limit.
const GRID_MAX := 8192

## `OUTSTANDING_WORK.md` §2.7 / `GENERATION_PARAMETERS.md`: this field used to
## go all the way to `i32::MAX` (2 147 483 647), which is exactly the seed
## magnitude `cartalith-noise`'s per-octave `s + o*131` offset overflowed
## `i32` on ("attempt to add with overflow" in a debug build; a silent, wrong
## wrap in release). That overflow is fixed at its root now (`cartalith-noise`
## uses `wrapping_add`/`wrapping_mul`, proven exact JS `ToInt32` parity, so no
## seed in `i32`'s full range can panic it any more) -- but several OTHER
## generation call sites still add a small constant straight to the raw seed
## in plain, unchecked `i32` arithmetic before a world exists to check
## (`opts.seed + 1`/`+2`/`+3` in `cartalith-assets`, `o.seed + 99` in
## `cartalith-climate`, `seed + 17/101/213/331` in `cartalith-terrain` and
## `cartalith-gpu`, `field_seed + o*131` -- up to `+262` -- in
## `generate_continentality_field`). The largest of those is `+331`; capping
## the field here at two billion leaves over 147 million of headroom below
## `i32::MAX` for every one of them at once (checked directly: the worst-case
## `field_seed` reachable from any seed in `[0, 2_000_000_000]` is
## `0x77f5ffff`, itself ~135 million below `i32::MAX`), without hunting down
## and rewriting every one of those call sites for a single dialog field.
## Round rather than the tightest safe value, so the number in the field
## reads as a deliberate limit and not an arbitrary one.
const SEED_MAX := 2_000_000_000
const DEGENERATE_ASPECT := 16.0 ## Past this, the coarse weather grid loses almost all resolution on the short axis.

## The two advisories the form opens with. Constants rather than literals at
## the call site because there are two call sites: the desktop and tablet form
## folds them into one line with both as its tooltip (NWP-3), and §6.7's phone
## card carries them inside itself in full.
const NOTE_CREATION_ONLY := "Creation-time only. Extent, resolution and archetype reallocate every field in the pipeline, so changing them later means a fresh Create, not a live edit — the reference itself refuses to make width mid-project editable for the same reason."
## Second advisory (see `NOTE_CREATION_ONLY`): says Create leaves Look, NPR, ramps and presets alone. Shown in full on the phone card and as half of the desktop advisory tooltip.
const NOTE_APPEARANCE_KEPT := "Your appearance settings are kept. Look, NPR, ramps and any saved preset are not part of a world, so Create changes the terrain and leaves the map's style exactly as you have it."

## `design/dcc-environment-2026-08-31/spec/06-phone.md` §6.7's New World modal:
## a card, `max-width:360px`, radius 22, centred over a scrim padded `22px` --
## a genuinely smaller form, not this one reflowed. The owner's "keeps
## everything and reflows" ruling (`DCC_SHELL_SCOPE.md`) is a **tablet**
## ruling; §6.7 is the phone's own drawing and these are its numbers.
const PHONE_CARD_MAX_W := 360
const PHONE_CARD_INSET := 22    ## The scrim's `padding:22px`, both sides.
const PHONE_CARD_RADIUS := 22  ## Card corner radius, 22 (§6.7); the card stylebox is `DccTheme.flat(..., PHONE_CARD_RADIUS)`.
const PHONE_CARD_MIN_W := 240   ## A floor, so a narrow viewport yields a card rather than a sliver.
const PHONE_CHIP_RADIUS := 14   ## §6.7's extent chips.
## **Landscape phone** (a rotated handset; found on an emulator at 3168x1440,
## where the one-column card was ~200 dp tall and the 4-line disclaimer filled
## it). §6.7 draws only a portrait card, so these are labelled judgements, not
## artboard numbers. `PHONE_LAND_MIN_DP`: the content width, in dp, from which
## the card goes two-column -- 700 dp holds a 316 dp preview column, a 16 dp gap
## and a right column no narrower than the 328 dp portrait form; a portrait
## handset is 360-430 dp, so nothing portrait can reach it.
const PHONE_LAND_MIN_DP := 700
const PHONE_CARD_MAX_W_LAND := 760   ## 316 + 16 + 396 right column + the card's 32 dp padding.
const PHONE_LAND_LEFT_W := 316       ## `PREVIEW_DISPLAY_W_PHONE` 296 + the frame's 2x6 margin + an 8 dp scroll bar.
const PHONE_LAND_STAGE_H := 150      ## Stage height (dp) in landscape: a 234x150 map at the mask's 320:205 shape, so the preview column fits the ~250 dp a rotated card leaves.
const PHONE_LAND_PAD_V := 8          ## Card padding top and bottom in landscape (portrait: 18 / 16, §6.7).

## §6.7's action row, read off `design/Cartalith-Android-2026-09-07.dc.html`
## (the `modalOpen` block) rather than taken from a summary:
##
##   <div style="display:flex;gap:10px;padding-top:16px">
##     <div … style="flex:1;min-height:46px;border-radius:23px;…;
##                   color:var(--sec);background:var(--chip)">CANCEL</div>
##     <div … style="flex:1.4;min-height:46px;border-radius:23px;…;
##                   color:var(--accInk);background:var(--acc)">CREATE WORLD</div>
##   </div>
##
## Both `font:500 10.5px 'IBM Plex Mono'` with `letter-spacing:.14em`.
const PHONE_ACTION_H := 46
const PHONE_ACTION_RADIUS := 23  ## Action-button corner radius, §6.7's `border-radius:23px` (half of `PHONE_ACTION_H`, so a pill).
const PHONE_ACTION_GAP := 10  ## Gap between Cancel and Create, §6.7's `gap:10px`.
const PHONE_ACTION_TOP := 16  ## Space above the action row, §6.7's `padding-top:16px`.
## `flex:1` and `flex:1.4`. A Godot `HBoxContainer` shares surplus width by
## `size_flags_stretch_ratio` among its `SIZE_EXPAND` children, which is what
## CSS `flex-grow` does with the free space after content -- the same
## distribution, not merely a similar one, once both children are `EXPAND_FILL`.
const PHONE_CANCEL_FLEX := 1.0
const PHONE_CREATE_FLEX := 1.4  ## Create button's stretch ratio, §6.7's `flex:1.4` (Cancel is `PHONE_CANCEL_FLEX`, 1.0).
## `10.5px`, rounded up. `Control.add_theme_font_size_override()` takes an int,
## so the half pixel is not expressible; recorded in `ANDROID_UI_SPEC.md`
## **§6.6** rather than left as a silent difference. `.14em` of 10.5 px is
## 1.47 px of tracking, which `DccTheme.mono()` takes as a whole pixel.
##
## This line said §1.11 when it shipped, which is the GENERATE column's
## deviations table and carries no such row -- the modal's departures live in
## its own §6.6. The lane's report said §6.6 and the comment was not moved with
## it, which is the cheapest kind of citation to get wrong and the hardest to
## notice: a section number is not checkable by reading the sentence it sits in.
const PHONE_ACTION_FS := 11
const PHONE_ACTION_TRACKING := 1  ## Whole-pixel stand-in for `.14em` tracking, passed to `DccTheme.mono()` (see the block above).

## Advisory shown under the Extent picker in Region mode.
const EXTENT_NOTE_REGION := "Region — a framed area of a world. The map's north and south edge latitudes are set in the Climate stage. X does not wrap. Any aspect ratio is physically fine here."
## Advisory shown under the Extent picker in World mode; it suggests 2:1 and
## explains why without locking anything (see `_update_extent_state`).
const EXTENT_NOTE_WORLD := "Whole world -- a seamless equirectangular sheet: X wraps a full 360° of longitude and Y spans 180° of latitude, pole to pole. 2:1 is the ratio that keeps the graticule true; anything else stretches it against the terrain. Advisory, not enforced -- pick any size you want."

## `earth`/`supercontinent`/… (`bridge.archetypes()`) with the reference's own
## friendlier names -- a display concern, so it lives here rather than in Rust.
const ARCHETYPE_LABELS := {
	"earth": "Earth-like", "supercontinent": "Supercontinent",
	"archipelago": "Archipelago", "volcanic": "Volcanic", "rift": "Rift",
}

# -- Seed preview (NWP-3, `NEW_WORLD_PREVIEW_PROPOSAL.md` §2-§3, Ruling CE) -----

## The desktop and tablet window: two columns, the preview left and the
## settings right. The proposal's §2.1 says "about 980 x 780"; 1000 is used so
## the right column clears the derived panel's widest line
## ("2048 × 1311 cells   (2.68 M cells)") without folding a horizontal minimum
## into the scroller (`MISTAKES.md`, "Read a layout that overflows the
## screen"). A labelled judgement, measured by `_nwprev_probe.gd` leg `v`.
## Same 780 height as the shipped 620 x 780 window.
const DESKTOP_SIZE := Vector2i(1000, 780)
## The smallest desktop size the two columns lay out in. It is ALSO what keeps
## a stale remembered size out: `DccWidgets._desktop_window_size()` restores a
## saved size only when it is at or above `min_size`, and every installation
## that opened the old dialog has `new_world_dialog@desktop` = 620 wide or so
## saved -- which would open this layout with its right column crushed.
## Set before `phone_window()` runs, because that is where the restore happens.
## `phone_present()` zeroes it on a phone.
const DESKTOP_MIN_SIZE := Vector2i(960, 720)
## Left column width: the 640 px canvas plus the frame's 6 px padding each side.
const LEFT_COL_W := 652

## The preview's own grid width, in cells. The owner's request (2026-10-11,
## quoted in the proposal's head): *"gray on blue ... in a 320,px resolution"*.
## Passed to `WorldGen.preview_land_mask` as `preview_w`; the binding derives
## the height from the target grid's aspect (`BINDING.md`, lane A).
const PREVIEW_W := 320
## How large the 320-cell mask is drawn: 2x on desktop and tablet (proposal
## §2.1, "drawn at 2x, with nearest-neighbour filtering"), a fixed 640 px stage.
## On a phone the stage is as wide as the card's form makes it and only its
## HEIGHT is fixed, at 296 dp's region height: a fixed 296 dp minimum WIDTH,
## plus the phone's thick scroll bar and the card padding, pushed §6.7's card
## to 370 dp past its 360 dp cap (measured, `_nwprev_probe.gd` leg `v`). The
## height is the region frame's (`round(w * 0.64)`, proposal §2.1); a
## whole-world mask (2:1) letterboxes inside it rather than moving every row.
const PREVIEW_DISPLAY_W_DESKTOP := 640
const PREVIEW_DISPLAY_W_PHONE := 296  ## Phone preview width in dp, used to size the stage height and the landscape column (see the block above).
## On a phone, the width floor given to the preview's wrapping labels (see the
## disclaimer's own comment for why they need one): small enough that the card
## form, not the label, decides the width.
const PHONE_PREVIEW_LABEL_FLOOR := 200
const PREVIEW_REGION_ASPECT := 0.64  ## Stage height / width, the region frame's `round(w * 0.64)` (proposal §2.1, per the block above).
## Grey land on blue ocean, the owner's colours and Q7's default (Ruling CE).
## Literals, deliberately independent of `DccTheme`'s palette and of the map's
## look: the preview is a diagnostic, and the owner asked for exactly these two.
const PREVIEW_LAND := Color("#8f9295")
## Ocean colour of the new-world mask preview, #2b4d6b: the owner's colour and Q7's default (Ruling
## CE, per the comment above); passed as the `ocean_color` shader parameter in `_build_preview`.
const PREVIEW_OCEAN := Color("#2b4d6b")
## **The disclaimer Ruling CE makes part of the screen** (owner, 2026-10-12:
## *"put the new screen in with a disclaimer that the preview is an
## approximation"*): visible on the preview frame at every density, in plain
## words, never only in a tooltip. The figure is from the owner-side measurement
## (Classic land/ocean agreement with the finished world ~97-98%; low-land
## archetypes barely informative), so it is a measured claim, not a judgement.
const PREVIEW_DISCLAIMER := "Approximate preview: plate layout and land/ocean split only, not the finished map. Classic agrees closely (about 97-98% in measured runs); low-land archetypes (Volcanic, Archipelago) are rough."
## The dash reasons. Each names the state the code branches on (`MISTAKES.md`,
## "Dash a field with a reason": the reason is a claim). The first is the
## proposal's own wording for the missing binding (§8 NWP-3).
const PREVIEW_REASON_NO_BINDING := "this engine build has no preview (WorldGen.preview_land_mask is missing; rebuild the engine)"
const PREVIEW_REASON_NO_PARAMS := "no preview: this engine build publishes no parameter table"  ## Dash reason when the build publishes no parameter table (state `no_params`).
const PREVIEW_REASON_REFUSED := "no preview: the engine refused these settings (the reason is in the console)"  ## Dash reason when the engine rejected the settings (state `refused`).
## While a world generates the worker thread owns `WorldGen`, so nothing here
## touches the engine (proposal §6, "Where it runs"); the last drawn mask stays
## up and the stats line says it is held.
const PREVIEW_HELD := "held while a world generates; redraws when it finishes"
## The redraw debounce, so a held spinner does not queue a run per step.
## Proposal §6: *"it can run on the main thread behind a debounce of about
## 120 ms"* -- a labelled judgement there, kept here.
const PREVIEW_DEBOUNCE_S := 0.12
## While the dialog is open, the live parameter table is re-read this often and
## the preview redraws if any key moved. A poll rather than a hand-written list
## of watched keys, because the staleness key must be derived from the
## definition (`MISTAKES.md`, "Add a capability, or write a staleness/cache
## key"): the key is the WHOLE `get_params()` table plus the dialog's own
## inputs, so a parameter added to the early stages later cannot leave the
## preview silently stale. 0.5 s is a labelled judgement: well above the
## debounce, and a `get_params()` call (one dictionary build) is cheap.
const PREVIEW_POLL_S := 0.5
## Session history: the last four previews (Q5's default, Ruling CE).
const HISTORY_SLOTS := 4
## Thumbnail button size. Desktop: four across the 652 px left column.
## Phone: four across the 296 dp card. Both are above the 44 dp touch floor
## on both axes (the phone one is floored there anyway).
const THUMB_SIZE_DESKTOP := Vector2(154, 116)
const THUMB_SIZE_PHONE := Vector2(68, 64)  ## History thumbnail size on a phone, four across the 296 dp card; no source found for the 68 x 64 pair (the comment above states only the 44 dp touch floor).
## `tect.plates`' own range in `params.rs` `PARAMS` (`Kind::Int`, 4 to 40). Used
## only when the running build publishes no `param_info("tect.plates")`.
const PLATES_MIN_FALLBACK := 4
const PLATES_MAX_FALLBACK := 40  ## Upper bound used when `param_info("tect.plates")` is absent; matches `params.rs` per the comment above.
## The phone seed row's label width: the mockup's phone frame draws
## `<label style="flex-basis:44px">Seed</label>`.
const PHONE_SEED_LABEL_W := 44
## The mockup's phone row label column, `.phone .row>label{flex:0 0 108px}`;
## used for the Plates row, the one new row with two 44 dp buttons in it.
const PHONE_ROW_LABEL_W := 108
## Desktop pointer size of the seed row's two icon buttons (the mockup's
## `.iconbtn`, `width:28px;height:24px`). Touch densities floor to 44 dp.
const ICON_BTN_DESKTOP := Vector2(28, 24)
## The canvas_item shader that turns the engine's 0/1 mask (an `L8` image, so
## no per-cell GDScript loop builds colours) into the two preview colours. A
## byte of 1 samples as 1/255, so anything above 0.001 is land.
const MASK_SHADER := "shader_type canvas_item;\nuniform vec4 land_color;\nuniform vec4 ocean_color;\nvoid fragment() {\n\tCOLOR = texture(TEXTURE, UV).r > 0.001 ? land_color : ocean_color;\n}\n"

# -- Controls -------------------------------------------------------------------

## The Seed spin box. Typed text is only committed by `apply()`; see `_seed_dirty` and `request()`.
var seed_input: SpinBox
## The Region / World picker (Extent); its value decides the aspect pinning and which advisory shows.
var extent_input: OptionButton
## The advisory under the Extent picker; set to `EXTENT_NOTE_WORLD` or `EXTENT_NOTE_REGION` (`_update_extent_state`).
var extent_note_label: Label
## The Size picker over `SIZE_PRESETS`, Custom at `SIZE_CUSTOM_INDEX`.
var size_preset_input: OptionButton
## Map width in km (spin box).
var width_input: SpinBox
## The Resolution picker over `RESOLUTION_PRESETS` / `RESOLUTION_LABELS`, Custom at `RESOLUTION_CUSTOM_INDEX`.
var resolution_input: OptionButton
## Grid width in cells, bounded by `GRID_MIN`..`GRID_MAX`. Hidden on a phone, but `request()` still reads it.
var grid_w_input: SpinBox
## The Aspect picker over `ASPECT_PRESETS` (Custom at `ASPECT_CUSTOM_INDEX`); `_derived_grid_h()` reads it.
var aspect_input: OptionButton
## Grid height in cells: derived from width and aspect, except when hand-typed. Read directly by `request()`.
var grid_h_input: SpinBox
## The Archetype picker, filled from `bridge.archetypes()` with `ARCHETYPE_LABELS` names; Classic is the empty `_archetype`.
var archetype_input: OptionButton
## Village-seeding toggle; its handler writes `_villages`.
var villages_check: CheckBox
## Metropolis toggle; its handler writes `_metropolis`.
var metropolis_check: CheckBox
## Biome carrying-capacity toggle; its handler writes `_biome_k`.
var biome_k_check: CheckBox
## The Recovery phase picker (names from `get_recovery_phase_names()`); its handler writes `_recovery_phase`.
var recovery_input: OptionButton
## Warning line under the derived panel: the joined warnings, hidden when there are none (`_update_derived`).
var dimension_warning_label: Label
var _derived_labels: Dictionary = {} ## "Grid"/"Extent"/"Cell size"/"Aspect" -> value Label

## Archetype keys read from `bridge.archetypes()` when the form is built.
var _archetype_names: PackedStringArray = []
var _archetype := "" ## Empty = Classic (World Structure disabled).
## Village-seeding choice posted by `request()`; refreshed from the engine by `_sync_from_engine()`.
var _villages := false
## Metropolis choice posted by `request()`; refreshed by `_sync_from_engine()`.
var _metropolis := false
## Recovery-phase index posted by `request()`; clamped to the picker's item count in `_sync_from_engine()`.
var _recovery_phase := 0
## `civBiomeKChk` (reference `_biomeK`, line 6441), default OFF -- "0 =
## biome carrying-capacity residual OFF (bit-identical)".
var _biome_k := false
## Phone (§13). `DccWidgets.phone_window()`'s header comment carries the whole
## treatment and why; here it decides whether the in-content header and the
## Cancel button exist, and whether the form is re-fitted for touch.
var _phone := false
## Re-entrancy guard: true while code (not the user) writes the dimension controls, so their handlers do not feed back.
var _dim_syncing := false
## Lane SEED, 2026-09-13 (the row found on glass 2026-09-07: a typed seed
## surfacing silently on a LATER Generate). True once the user has actually
## edited `seed_input`'s `LineEdit` since `.value` was last set by code or
## last committed. `request()` reads this to decide whether a pending typed
## seed needs `SpinBox.apply()` forced onto it before the read -- see
## `request()`'s own comment for why an unconditional `apply()` (the
## 2026-09-12 attempt, reverted) broke a rolled seed instead.
var _seed_dirty := false
## §6.7's card (phone only, `null` on desktop and tablet) and the two extent
## chips, which every density builds since NWP-3 (`_build_extent_chips()`).
var _card: PanelContainer
## The Region / World chips (§6.7); they drive the Extent picker.
var _extent_chips: Array[Button] = []
## §6.7's own action row, phone only. `null` on desktop and tablet, where
## `AcceptDialog`'s button row is the right control (Cancel / Generate world,
## `_build_desktop_footer()`).
var _phone_cancel: Button
## The phone's CREATE WORLD button in the action row; `null` on desktop and tablet.
var _phone_create: Button
## The three nodes `_fit_phone_card_height()` works between, phone only.
## `_outer_scroll` is the form scroller `setup()` builds for every density (on
## desktop and tablet it is the right column) and is the only one of the three
## that is non-null there.
var _outer_scroll: ScrollContainer
var _card_scroll: ScrollContainer   ## The card's own scrolling band.
var _card_form: VBoxContainer       ## What scrolls inside it.
var _card_actions: VBoxContainer    ## The card's outer column; holds the pinned row.
## Landscape phone (see `PHONE_LAND_MIN_DP`). `_card_bands` is the HBox that
## holds `_land_scroll` (left, preview column; hidden in portrait) beside
## `_card_scroll`. `_grp_preview` is the preview frame's group: the first child
## of `_card_form` in portrait, the content of `_land_scroll` in landscape.
var _card_bands: HBoxContainer
## Landscape phone only: the left scroller that holds `_grp_preview`; hidden in portrait.
var _land_scroll: ScrollContainer
## The preview frame's group; moves between `_card_form` (portrait) and `_land_scroll` (landscape). `null` on desktop and tablet.
var _grp_preview: VBoxContainer
var _grp_seed: VBoxContainer         ## Seed row + history; stays in `_card_form` in both orientations.
## The card's outer `MarginContainer`; landscape sets its top/bottom margins to `PHONE_LAND_PAD_V` (portrait: 18 / 16).
var _card_pad: MarginContainer
## The `nw_preview_stage` panel that holds the mask canvas.
var _preview_stage: PanelContainer
## The phone in-content header's outer node; hidden in landscape.
var _phone_head_wrap: Control
## True while the phone card is in its two-column landscape layout (see `PHONE_LAND_MIN_DP`).
var _phone_land := false

# -- Seed preview state (NWP-3) --------------------------------------------------

## The preview frame and its parts. Named nodes (`nw_preview_*`) so a probe
## finds them by name rather than by position.
var _preview_frame: PanelContainer
## Title label, text from `_set_preview_title()`.
var _preview_title: Label
## The `PREVIEW_DISCLAIMER` label, visible on the frame at every density (Ruling CE).
var _preview_disclaimer: Label
## The `TextureRect` that draws the mask through `_mask_material`.
var _preview_canvas: TextureRect
## The `ImageTexture` (an `L8` mask) the canvas shows.
var _preview_tex: ImageTexture
## The dash label shown in place of the canvas when there is no preview; the reason is `_preview_reason`.
var _preview_dash: Label
## Why there is no preview (a `PREVIEW_REASON_*` string).
var _preview_reason: Label
## The stats line under the preview: land share, plates and draw time, the `PREVIEW_HELD` note, or a dash with its reason.
var _preview_stats: Label
## The `ShaderMaterial` running `MASK_SHADER` with `PREVIEW_LAND` / `PREVIEW_OCEAN` as its uniforms.
var _mask_material: ShaderMaterial
## What the canvas shows now: "drawn", "held", "no_binding", "no_params",
## "refused", or "" before the first attempt. Read by the probe; the dash and
## the stats line are what a person reads.
var preview_state := ""
## The last mask drawn, as the engine returned it (1 = land, 0 = ocean), and its
## height. Empty until a draw succeeds. Never filled with a placeholder.
var preview_mask := PackedByteArray()
## Height in cells of `preview_mask`, as returned by the engine; 0 until a draw succeeds.
var preview_h := 0
## The full input key of the last successful draw (the whole parameter table
## plus the dialog's own inputs). A redraw with an unchanged key is skipped.
var _preview_key := ""
## One-shot timer of `PREVIEW_DEBOUNCE_S` that calls `_redraw_preview`.
var _preview_debounce: Timer
## Repeating timer of `PREVIEW_POLL_S` that also calls `_redraw_preview`; started and stopped with the dialog's visibility.
var _preview_poll: Timer
## Set by the inputs that push a history entry (dice, a typed seed, a width or
## extent change); consumed by the next successful draw. A restore clears it.
var _history_push_pending := false
## True while a history entry is being restored, so the controls it rewrites do not push another history entry.
var _restoring := false
## Session-only history, newest first: {seed, world, km, plates, manual, mask,
## h, land}. Never saved (Q5).
var _history: Array[Dictionary] = []
## The thumbnail buttons for `_history`.
var _history_buttons: Array[Button] = []
## The seed lock (Q4, Ruling CE: for the session, not saved in the project).
## `randomise_seed()` is a no-op while it is on, which is what makes the WORLD
## dock's and the tool bar's *New seed* (`app.gd::_new_seed()`) honour it too.
var seed_locked := false
## The seed row's dice (randomise) button, built by `_build_seed_dice()`.
var _dice: Button
## The seed row's lock toggle, built by `_build_seed_lock()`; mirrors `seed_locked`.
var _lock: Button
## Plates (Q1): `auto` follows `plates_for_km(width)`; a press of − or + sets
## it by hand until reset. Under an archetype the count is the engine's own
## derivation and the row is read-only (Q8).
var _plates_manual := false
## The hand-set plate count, written to `tect.plates` on Create only while `_plates_manual` is set (otherwise `plates_for_km` decides, via `_classic_plates()`). Initial 14 equals the engine default `plates: 14` (`cartalith-engine` `WorldParams::defaults()`); not cited at the declaration.
var _plates_value := 14
## The Plates row's minus button.
var _plates_minus: Button
## The Plates row's plus button.
var _plates_plus: Button
## The Plates row's number readout.
var _plates_readout: Label
## Small label saying where the count comes from: "auto from width", "set by hand" or "from archetype (read-only)".
var _plates_source: Label
## Button that returns the count to automatic (clears `_plates_manual`).
var _plates_reset: Button
## Desktop/tablet only: the disclosure that holds today's aspect, grid and
## Generation controls, unchanged. `null` on a phone, where those stay hidden.
var _more_toggle: Button
## The collapsible body of "More options" (aspect, grid, Generation controls); `null` on a phone.
var _more_body: VBoxContainer
## Desktop/tablet only: `AcceptDialog`'s own Cancel, added beside the OK
## button (which reads "Generate world" there).
var _desktop_cancel: Button

## Build the whole New-world form once: window size, the phone variant, the
## Create button wired to `_on_create`, and every section from the engine's
## parameter table. `b` must be set before anything reads `bridge`. Call once;
## `request()` and `open_*` assume it has run.
func setup(b: EngineBridge) -> void:
	bridge = b
	title = "New world"
	## NWP-3: the two-column window (was 620 x 780, one column). `min_size`
	## before `phone_window()`, which is where a remembered size is restored --
	## see `DESKTOP_MIN_SIZE` for why that order matters.
	size = DESKTOP_SIZE
	min_size = DESKTOP_MIN_SIZE
	## PH-06. Also turns `wrap_controls` off -- which this dialog already did
	## by hand, for the reason `_build_stage_dialog` records in `main.gd`: an
	## autowrap dialog grows to its full content height and can run off a
	## 1080p screen. The shared call is the same fix, so the hand-rolled line
	## goes rather than being kept beside it.
	##
	## Called *before* the OK button is renamed, deliberately:
	## `phone_window()` sets `ok_button_text` to "Close" for the read-only
	## windows it was written for, and here that button is the Create action.
	_phone = DccWidgets.phone_window(self, get_parent())
	## "Generate world" on desktop and tablet (the mockup's footer, proposal
	## §2.1), "Create" on a phone, where this button is hidden anyway and §6.7's
	## own CREATE WORLD stands in for it. Still `AcceptDialog`'s OK underneath,
	## so `confirmed -> _on_create()` is unchanged.
	get_ok_button().text = "Create" if _phone else "Generate world"
	confirmed.connect(_on_create)
	if not _phone:
		_build_desktop_footer()

	## A borderless phone window has no title bar, so it has no ✕ either -- and
	## unlike the three civ windows, this dialog's own OK button is not a way
	## out, it is "generate a world". Without this, opening New world on a
	## handset would be a one-way door.
	##
	## **On a phone the way out is §6.7's own CANCEL, drawn inside the card,
	## and `AcceptDialog`'s button row is hidden entirely.** It used to be
	## `add_cancel_button("Cancel")`, which put `[Create] [Cancel]` in the
	## dialog's footer at a REPORTED 59 x 44 and 60 x 44 dp (carried from an
	## earlier pass, not re-measured, and a verifier could not reproduce them
	## from this file at HEAD -- treat them as a quotation) against a 360 dp card -- about
	## a third of the width the artboard draws, in the opposite order, and 44
	## high where §6.7 says 46. The order came from Godot: `AcceptDialog` owns
	## its OK button and `add_cancel_button()` appends beside it, so the
	## primary is first on this platform. **"It is what `AcceptDialog` does" is
	## a mechanism, not a reason** -- and a phone wants the destructive-safe
	## action first and the primary as a full-width target regardless. So the
	## row is built by `_build_phone_actions()` and the footer is dropped;
	## `get_ok_button()` still exists (an `AcceptDialog` requires it) and is
	## simply never shown, which is why `confirmed` is still the signal
	## `_on_create()` hangs off.
	if _phone:
		get_ok_button().visible = false

	## One column rather than the margin directly, so the phone header has
	## somewhere to sit: an `AcceptDialog` gives its *first* content child the
	## whole rect, so a second sibling would simply overlap this one
	## (`place_editor_window.gd` made the same arrangement first).
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	add_child(col)

	var margin := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		margin.add_theme_constant_override("margin_" + side, 10)
	margin.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(margin)
	if _phone:
		_phone_head_wrap = DccWidgets.phone_head(col, "New world", "creation-time settings").get_parent().get_parent().get_parent() as Control

	## The two advisories (`NOTE_CREATION_ONLY`, `NOTE_APPEARANCE_KEPT`) used to
	## sit above the scroll here on desktop and tablet. NWP-3 folds them into
	## one line at the foot of the right column with the full text on hover
	## (proposal §2.1) -- see `_build()`. On a phone they ride inside §6.7's
	## card, as before. Why the appearance one exists at all: `look`, `npr`,
	## `appearance_over`, `appearance_ramp` and `appearance_preset` are session
	## state on `WorldGen` and none of them marks a generation stage stale, so
	## Create keeps a user's look -- the dialog simply never said so.
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_outer_scroll = scroll

	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 2)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(body)

	_archetype_names = bridge.archetypes()
	if _phone:
		## The phone keeps its one scroller holding §6.7's card.
		var root := VBoxContainer.new()
		root.add_theme_constant_override("separation", 4)
		margin.add_child(root)
		root.add_child(scroll)
		_build(body, null)
	else:
		## **Desktop and tablet: two columns** (proposal §2.1). The left column
		## (preview, seed, history) does not scroll -- it is the part the first
		## screen must hold (NWP-3 acceptance (vi)) -- and the right column is
		## `_outer_scroll`, the same scroller the one-column form used.
		var cols := HBoxContainer.new()
		cols.add_theme_constant_override("separation", 16)
		margin.add_child(cols)
		var left := VBoxContainer.new()
		left.add_theme_constant_override("separation", 4)
		left.custom_minimum_size.x = LEFT_COL_W
		cols.add_child(left)
		cols.add_child(scroll)
		_build(body, left)
		_narrow_dropdowns(body)
		if DccTheme.is_touch():
			_floor_touch_tree(body)
	_update_extent_state()
	_build_preview_timers()
	## `PARITY_AUDIT.md` §23 F14: this dialog is a persistent singleton
	## (`app.gd` builds one at startup and reuses it, `popup_centered()`
	## on every File > New world…), not a fresh instance per open, so
	## `_villages`/`_metropolis`/`_recovery_phase` above stay whatever they
	## were last set to -- by hand, or by never having been touched since
	## construction -- across loading an entirely different project.
	## `about_to_popup` (`Window`'s own pre-show signal) re-reads the live
	## engine getters every time the dialog is about to be shown, so a
	## reopened dialog offers to re-Create with the settings the world on
	## screen was actually generated with, not silently reset ones.
	about_to_popup.connect(_sync_from_engine)
	## `1.0`, not `phone_scale()`: `phone_present()` applies the scale once as
	## the window's `content_scale_factor`, and applying it again here would
	## square it. The form is built once, so one pass is enough.
	if _phone:
		get_parent().phone_fit(self, 1.0)
		## The card's width is a measurement, and a rotation changes both the
		## viewport it measures and the scale it is measured in --
		## `phone_window()` re-runs `phone_present()` on `phone_insets_changed`
		## for exactly that reason. Re-measuring on every open covers the same
		## ground from the one signal this dialog already listens to.
		about_to_popup.connect(_fit_phone_card)
		## A rotation while the dialog is open: the host's own "phone layout
		## moved" signal (the one `phone_window()`'s relay listens to), so the
		## card re-flows between its forms without waiting for the next open.
		if get_parent().has_signal("phone_insets_changed"):
			get_parent().phone_insets_changed.connect(_fit_phone_card)
		## **The height is not measurable from `about_to_popup`.** That signal
		## fires from inside `Window.popup()` *before* the new rect is applied
		## (the paragraph over `_fit_phone_card()` records the same trap for the
		## width, and solves it by measuring the HOST viewport instead). The
		## height has no host-side equivalent -- it depends on this window's own
		## margins, its `phone_head` and the gesture inset -- so it is taken off
		## the scroller once the layout it belongs to exists. `resized` is the
		## signal that says exactly that, and it fires again on rotation.
		scroll.resized.connect(_fit_phone_card_height)

## Builds every section of the form. `left` is the desktop/tablet left column
## (preview, seed, history); it is `null` on a phone, where everything goes into
## §6.7's card instead.
##
## Three containers decide where each control lands:
##
## * `card` -- what is visible and scrolls: §6.7's card on a phone, the right
##   column (`body`) on desktop and tablet.
## * `rest` -- what neither surface shows by default. On a phone it is HIDDEN;
##   on desktop and tablet it is the body of **More options** (NWP-3, proposal
##   §2.1): today's Grid columns, Aspect, Grid rows and the whole Generation
##   block, moved there unchanged and reachable by one click.
## * `hidden` -- built and never shown: `extent_input`, whose job the extent
##   chips do on every density now (they drive it, see `_build_extent_chips()`).
##
## `rest` is HIDDEN on a phone rather than not built, and that is the
## load-bearing half. Re-counted 2026-09-07, in the same edit that moved the
## width pair and the archetype onto the card, because this paragraph's own
## numbers are what the move changes: `request()` still reads **two** of the
## hidden controls directly (`grid_w_input` — which `heightmap_grid_summary()`
## reads again — and `grid_h_input`), and it posts **four** more values,
## `_villages`, `_metropolis`, `_biome_k` and `_recovery_phase`, whose only
## writers other than construction are those hidden controls' own handlers and
## `_sync_from_engine()`. (`width_input` was the third of that first group and
## is now on the card; `aspect_input` was never in `request()` at all, but
## `_derived_grid_h()` reads it, so it is hidden rather than dropped for the
## same reason.) A phone build that skipped them would post this dialog's
## construction-time defaults back into the engine on every Create, which is
## the exact defect `_sync_from_engine()`'s own biome-K clause below records,
## one control at a time. An invisible child contributes nothing to a
## `BoxContainer`'s minimum size, so the hidden form cannot widen the card
## either.
##
## **No control was deleted or renamed by NWP-3**: every `*_input`/`*_check`
## field above is still built, under the same name, with the same handler --
## other probes and `app.gd` find them by name.
func _build(body: VBoxContainer, left: VBoxContainer) -> void:
	var card := body
	var rest := body
	var hidden := VBoxContainer.new()
	hidden.visible = false
	if _phone:
		card = _phone_card(body)
		rest = VBoxContainer.new()
		rest.visible = false
		body.add_child(rest)
		rest.add_child(hidden)
		## Three groups, plain `VBoxContainer`s at the form's own separation 2, so
		## portrait lays out exactly as when these were direct children. The
		## preview group is the one that moves to its own column in landscape.
		_grp_preview = VBoxContainer.new()
		_grp_preview.add_theme_constant_override("separation", 2)
		card.add_child(_grp_preview)
		_grp_seed = VBoxContainer.new()
		_grp_seed.add_theme_constant_override("separation", 2)
		card.add_child(_grp_seed)
		left = _grp_preview
	else:
		body.add_child(hidden)
		## Created now so the sections below can be built into it; placed in
		## the right column after World structure, by `_build_more()`.
		rest = VBoxContainer.new()
		rest.add_theme_constant_override("separation", 2)
		rest.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	## -- The preview, then the seed row and the history (proposal §2.1/§2.2).
	_build_preview(left)

	var seed_parent: VBoxContainer = _grp_seed if _phone else left
	var seed_sec := DccWidgets.section(seed_parent, "Seed")
	seed_input = DccWidgets.number(seed_sec, "Seed", 0, SEED_MAX, 1, randi() % 1000000,
		_on_seed_changed,
		"Integer seed. The same seed and settings reproduce the same world. Capped at %s, not the full 32-bit range -- see SEED_MAX's own comment." % [SEED_MAX])
	## **The dirty flag's one source of truth.** Measured
	## (`_seedcommit_probe.gd`'s SIGNAL DIAGNOSTIC section) rather than
	## assumed from the class reference: a real keystroke landing in this
	## field while it has focus fires `LineEdit.text_changed`, and neither of
	## this dialog's own programmatic writers of `.value` (the construction
	## line just above, and `randomise_seed()`) does -- so this connection
	## needs no companion "was that write mine" check anywhere else. And even
	## if some future engine build made a `.value` write's internal display
	## resync also raise this signal, every writer below clears the flag on
	## the very next statement, and a plain `.connect()` delivers
	## synchronously in this engine -- so a same-call re-entrant `true` would
	## already be overwritten by the time that statement returns.
	## Belt-and-braces, not the load-bearing part.
	seed_input.get_line_edit().text_changed.connect(func(_t: String): _seed_dirty = true)
	## The dice on every density now (it was phone-only: proposal §1.3 item 3,
	## "on desktop the roll is outside the dialog"), and the lock beside it.
	_build_seed_dice()
	_build_seed_lock()
	## Phone: the seed row's label column is the mockup's own `flex-basis:44px`
	## rather than the shared row's 132 px. With the lock beside the dice, the
	## 132 px column made the row wider than the card's 328 dp form, and the
	## card grew to 392 dp past a 412 dp screen (measured by
	## `_nwcard_probe.gd`'s overflow walk and `_nwprev_probe.gd` leg `v`).
	if _phone:
		var seed_label := seed_input.get_parent().get_child(0) as Control
		if seed_label != null:
			seed_label.custom_minimum_size.x = PHONE_SEED_LABEL_W
	_build_history(seed_parent)

	## -- Extent. The dropdown is kept (three functions read
	## `extent_input.selected`) but hidden; the chips are the control.
	var extent_sec := DccWidgets.section(hidden, "Extent")
	extent_input = DccWidgets.choice(extent_sec, "Extent", ["Region", "Whole world"],
		1 if bool(bridge.param_get("world")) else 0, _on_extent_selected,
		"Reference control #extentSeg. Region = a framed area with user-set latitudes; Whole world = a seamless equirectangular sheet with toroidal X wrap.")
	var chips_sec := _build_extent_chips(card)
	## The extent note: under the chips on desktop and tablet; on a phone it
	## stays where it was (hidden), the chips carrying it as their tooltip.
	extent_note_label = DccWidgets.note(extent_sec if _phone else chips_sec, "")

	## **The phone card carries the world's SIZE, and that is an owner ruling
	## made over the canvas -- not a drift from it.**
	##
	## Owner, 2026-09-07, holding the APK: *"even the initial or new map setup
	## doesn't allow for a km/size input"*, reported as a defect. Checked before
	## acting rather than assumed: §6.7's modal (`design/
	## Cartalith-Android-2026-09-07.dc.html`, the `modalOpen` block) draws NAME,
	## SEED with a dice, EXTENT as two chips, CANCEL and CREATE WORLD -- and no
	## width field, no resolution, no archetype. So **the canvas and the card
	## this file shipped agreed with each other**, and the owner overruled both.
	## `CLAUDE.md`'s standing rule is what settles it: *an owner decision is
	## newer than any canvas*. Recorded here because the next reader will
	## otherwise diff this against §6.7 and "fix" it back.
	##
	## Which of the size controls came across, and why each:
	##
	## * **Map width** (preset) and **Width (km)** -- YES, both. This is the
	##   thing the owner named. A 200 km world and a 12 000 km world are
	##   different products, not different preferences: `width_km / grid_w` is
	##   the one quotient every distance, grade, route length and settlement
	##   spacing in the engine is derived from, it is creation-time only
	##   (`NOTE_CREATION_ONLY`), and 800 km was silently the only answer a
	##   handset could give. The preset dropdown is the tap-only route and the
	##   km spin box is the free entry; `_refresh_dimensions()` already keeps
	##   the two in step in both directions, so they are one value with two
	##   faces rather than two sources of truth.
	## * **Plates** -- YES (NWP-3, Q1): it follows the width, so it sits under it.
	## * **Grid columns** -- NO. It is the *same number* as Resolution, which is
	##   already on the card: `_on_resolution_selected()` writes `grid_w_input`
	##   and `_refresh_dimensions()` writes `resolution_input` back. Two views
	##   of one value inside 360 dp is the defect `MISTAKES.md` records as "add
	##   a second route to a control the user already has one to", and a custom
	##   column count is a desktop refinement, not a phone one. (On desktop it
	##   is under More options for the same reason.)
	## * **Aspect** -- NO, and this one is a judgement rather than a duplicate.
	##   The extent chips the card already draws pick both of the reference's
	##   own aspects and nothing else exists in the reference to lose: it
	##   hardcodes 2:1 in world mode and 1.5625:1 otherwise (this control's own
	##   tooltip below says so), and `_update_extent_state()` moves the
	##   selection between exactly those two. The other five ratios are this
	##   port's addition, and a wrong one yields a differently-shaped map rather
	##   than a broken one -- a preference, which is what a small form drops.
	## * **Grid rows** -- NO, for Grid columns' reason and more so: it is
	##   derived from aspect and only editable as a hand-typed override.
	##
	## The derived readout comes across with them, because it is what makes the
	## pair legible: `Width (km) 40 075` over `Resolution 512` is 78 km per
	## cell, and nothing else on the card would say so.
	var size_sec := DccWidgets.section(card, "Size & resolution" if _phone else "Size")
	var grid_sec := DccWidgets.section(rest, "Grid & aspect")
	var size_labels: Array = []
	for preset: Dictionary in SIZE_PRESETS:
		size_labels.append(String(preset["label"]))
	size_labels.append("Custom")
	size_preset_input = DccWidgets.choice(size_sec, "Map width", size_labels, 1,
		_on_size_preset_selected,
		"Real-world width of the map. Creation-time only, same as the reference: changing it silently rescales every derived distance, grade, route length and settlement spacing.")
	width_input = DccWidgets.number(size_sec, "Width (km)", 1.0, 100000.0, 1.0, 800.0,
		func(_v: float):
			if not _restoring:
				_history_push_pending = true
			_refresh_dimensions())
	_build_plates_row(size_sec)

	var res_labels: Array = []
	for l in RESOLUTION_LABELS:
		res_labels.append(l)
	res_labels.append("Custom")
	## §6.7 draws no resolution control; `DCC_SHELL_SPEC.md` §2.1 names one --
	## *"Modal: name, seed, extent (region/world), working resolution"* -- for
	## the same command, so this one field of the desktop form rides into the
	## card. It is the only creation-time value whose absence is a trap rather
	## than a preference: the default is 2 048 columns, `_on_create()` is the
	## only path that ever sizes a grid, and a handset is the device least able
	## to afford the difference. `dimension_warning_label` moves with it, so
	## the 4K/8K cost is still disclosed where the choice is made. It is also a
	## preview input: the binding rescales the cell-unit settings to this grid
	## (proposal §4.1), so a resolution change redraws.
	resolution_input = DccWidgets.choice(size_sec, "Resolution", res_labels, RESOLUTION_DEFAULT_INDEX,
		_on_resolution_selected,
		"The reference's own 512/1K/2K/4K/8K segment. Sets the grid WIDTH only; grid height follows below.")
	grid_w_input = DccWidgets.number(grid_sec, "Grid columns", GRID_MIN, GRID_MAX, 1,
		RESOLUTION_PRESETS[RESOLUTION_DEFAULT_INDEX], func(_v: float): _refresh_dimensions())

	var aspect_labels: Array = []
	for preset: Dictionary in ASPECT_PRESETS:
		aspect_labels.append(String(preset["label"]))
	aspect_labels.append("Custom")
	aspect_input = DccWidgets.choice(grid_sec, "Aspect", aspect_labels, ASPECT_DEFAULT_INDEX,
		func(_i: int): _refresh_dimensions(),
		"The frame's width:height. The reference has no aspect control -- it hardcodes 2:1 in world mode and 1.5625:1 otherwise; both are here by name.")
	## Set only at creation time, not saved as an editable setting afterwards:
	## it is a call argument to the engine's `generate_sized()`, not a stored
	## parameter.
	grid_h_input = DccWidgets.number(grid_sec, "Grid rows", GRID_MIN, GRID_MAX, 1, 1311,
		_on_grid_h_changed,
		"Fixed once the world is created. Changing it reallocates every field the generator produces, so it can only be set here, not edited afterwards.")

	_build_derived_panel(size_sec)
	dimension_warning_label = DccWidgets.note(size_sec, "")
	dimension_warning_label.add_theme_color_override("font_color", DccTheme.c("stale"))

	## **Archetype comes across too, and it closes a route that dead-ended.**
	##
	## `world_workspace.gd`'s phone GENERATE sheet draws `Archetype` as a
	## tappable row whose stated reason is *"Pick it in File ▸ New world"*
	## (`PHONE_GEN_ABSENT`, stage 2). That reason was true of the desktop form
	## and **false of the phone**, because this control was built into
	## `struct_sec` -- which was parented to the hidden `rest` -- so the row
	## opened a dialog that did not have it.
	##
	## Lifting it is the right half of that pair rather than rewording the
	## reason. It is creation-time only in the strongest sense of the phrase:
	## `_on_archetype_selected()` seeds five dials that stay editable
	## afterwards, but `request()["archetype"]` is what decides **which
	## generation call runs** (`generate_world_structure_sized` against the
	## plain path), and no dial on the GENERATE sheet can express that.
	##
	## The five-dial note goes with it: it is what tells a phone user that the
	## choice is a starting point and not a lock.
	var struct_sec := DccWidgets.section(card, "World structure")
	var archetype_labels: Array = ["Classic"]
	for name in _archetype_names:
		archetype_labels.append(String(ARCHETYPE_LABELS.get(String(name), String(name).capitalize())))
	archetype_input = DccWidgets.choice(struct_sec, "Archetype", archetype_labels, 0, _on_archetype_selected,
		"Reference ARCHETYPES. Choosing one applies its continentality/fragmentation/tectonic-energy/ocean-depth/hotspot-density preset immediately and routes Create through generate_world_structure_sized instead of the plain path.")
	DccWidgets.note(struct_sec,
		"The five dials that preset sets -- continentality, fragmentation, tectonic energy, ocean depth, hotspot density -- live in World ▸ Generate ▸ 03 World structure, editable there after Create.")

	if not _phone:
		_build_more(card, rest)

	var gen_sec := DccWidgets.section(rest, "Generation")
	villages_check = DccWidgets.toggle(gen_sec, "Village seeding (additive hamlets)", false,
		func(v: bool): _villages = v,
		"Reference civVillagesChk, default off. Seeds an extra tier of hamlets after the main settlement pass.")
	metropolis_check = DccWidgets.toggle(gen_sec, "Imperial-seat tier (metropolis ★)", false,
		func(v: bool): _metropolis = v,
		"Reference civMetropolisChk, default off. After the road network is scored, promotes up to three capitals that are both dominant trade hubs (normalised betweenness >= 0.85) and seats of a large polity (>= 6 settlements) to the metropolis tier -- at most one per faction. Off means auto-populate output is bit-identical to not having the pass at all.")
	biome_k_check = DccWidgets.toggle(gen_sec, "Biome carrying-capacity residual", false,
		func(v: bool): _biome_k = v,
		"Reference civBiomeKChk, default off. Applies a per-biome disease/climate correction to carrying capacity K -- and, where a cell is a lowland wetland, the wetland residual instead. K feeds settlement suitability and the food-shed ceiling, so turning it on moves where settlements are placed and how large they get. Off short-circuits the whole correction, so output is bit-identical to not having it (the reference's own comment).")
	## Filled from the engine's own `_CIV_RECOVERY_NAME` table rather than a
	## second transcription of it here -- `get_recovery_phase_names()`.
	var recovery_labels: Array = []
	if bridge.world_gen != null and bridge.world_gen.has_method("get_recovery_phase_names"):
		for n in bridge.world_gen.get_recovery_phase_names():
			recovery_labels.append(String(n))
	if recovery_labels.is_empty():
		recovery_labels = ["Stable"]
	recovery_input = DccWidgets.choice(gen_sec, "Recovery phase", recovery_labels, 0,
		func(i: int): _recovery_phase = i,
		"Reference civRecoveryPhase, default Stable. A world that is still recovering from a collapse runs BELOW its ecological ceiling: every settlement's population is scaled by a fraction drawn inside the phase's band, and a nucleus scaled below the labour its tier needs demotes into its own ruins. Phases I and II also abandon tiny settlements that are neither formerly-urban nor ports. Stable is a strict no-op.")
	_build_placement_dials(gen_sec)
	DccWidgets.note(gen_sec,
		"Sea level, dynamic lithology, volcanic provinces, terrain wind deflection and ocean currents are live engine parameters, not dialog state -- edit them in World ▸ World data (02 Extent & scale), Geology (04 Tectonics, 05 Volcanism & impacts) and Climate (08) before or after Create; Create reads whatever they currently hold.")

	## §6.7's card carries no advisory prose, and both of these are kept
	## anyway: one says the choice above cannot be revisited later and the
	## other says Create will not take the user's look away. A smaller form is
	## a smaller set of *controls*; dropping a warning because the artboard is
	## narrower is a different thing.
	if _phone:
		DccWidgets.note(card, NOTE_CREATION_ONLY)
		DccWidgets.note(card, NOTE_APPEARANCE_KEPT)
		## `_card_actions`, not `card`: `card` is the band that scrolls and the
		## action row is the band that must not. See `_phone_card()`.
		_build_phone_actions(_card_actions)
		_fit_phone_card()
	else:
		## Desktop and tablet: the two advisories folded into one line, the
		## full text on hover (proposal §2.1). `MOUSE_FILTER_PASS` because a
		## `Label` ignores the mouse by default, which would make the tooltip
		## unreachable.
		var advisory := DccWidgets.note(card, "Creation-time only · your look is kept. (Hover for why.)")
		advisory.name = "nw_advisory"
		advisory.tooltip_text = NOTE_CREATION_ONLY + "\n\n" + NOTE_APPEARANCE_KEPT
		advisory.mouse_filter = Control.MOUSE_FILTER_PASS

## Desktop and tablet: the **More options** disclosure (proposal §2.1) -- a
## header button and, under it, `rest`, collapsed by default. It holds today's
## Grid columns, Aspect, Grid rows and the whole Generation block, moved here
## unchanged so the first screen is the terrain choice and not ~300 px of
## settlement placement (§1.3 item 4). Not a `DccWidgets.group()` because that
## one's arrow-glyph header is a dock's L4 group; this is one disclosure in a
## dialog, drawn as the mockup's `.more` row.
func _build_more(parent: VBoxContainer, rest: VBoxContainer) -> void:
	_more_toggle = Button.new()
	_more_toggle.name = "nw_more"
	_more_toggle.focus_mode = Control.FOCUS_NONE
	_more_toggle.alignment = HORIZONTAL_ALIGNMENT_LEFT
	_more_toggle.add_theme_font_override("font", DccTheme.mono(2, true))
	_more_toggle.add_theme_font_size_override("font_size", DccTheme.FS_HEADER)
	for key in ["font_color", "font_pressed_color", "font_focus_color"]:
		_more_toggle.add_theme_color_override(key, DccTheme.c("text_secondary"))
	_more_toggle.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	## Not `flat`: a flat `Button` voids every stylebox override
	## (`MISTAKES.md`, "Override a stylebox on a Button"), and these empty boxes
	## are what make it read as a header row rather than a button slab.
	var pad := DccTheme.inset(14, 4, 4, 4)
	for sb_name in ["normal", "hover", "pressed", "focus", "disabled"]:
		_more_toggle.add_theme_stylebox_override(sb_name, pad)
	_floor_touch(_more_toggle)
	_more_toggle.tooltip_text = "Aspect, grid columns and rows, and the settlement-placement block -- the same controls as before, folded away."
	_more_toggle.pressed.connect(func(): set_more_open(not _more_body.visible))
	parent.add_child(_more_toggle)
	_more_body = rest
	_more_body.name = "nw_more_body"
	parent.add_child(_more_body)
	set_more_open(false)

## Desktop and tablet: every dropdown in the right column sizes to its row,
## not to its longest item. `OptionButton.fit_to_longest_item` (Godot's
## default) made Map width as wide as "Planet · 40 075 km (Earth's equator)",
## which pushed the column's minimum to 394 px in a 312 px column -- a
## `ScrollContainer` with its horizontal axis disabled folds that into its own
## minimum and the right column ran 110 px past the window's edge (measured,
## `_nwprev_probe.gd` leg `v`). `clip_text` plus the ellipsis is the pair
## `DccShell.phone_fit()` already applies to expanding dropdowns; safe here for
## the reason that function gives -- each of these rows is full width and the
## control expands into it, so it never collapses onto its arrow.
func _narrow_dropdowns(root: Node) -> void:
	for c in root.get_children(true):
		if c is OptionButton:
			var ob := c as OptionButton
			ob.fit_to_longest_item = false
			ob.clip_text = true
			ob.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		_narrow_dropdowns(c)

## Tablet (a touch density that is not a phone): floors every tappable control
## in the right column to 44 dp on both axes. The controls More options holds
## come from the shared builders, which floor height only (`toggle()`: a
## 40 x 44 `CheckBox`) or not at all (`slider()`: a 14 px track) -- measured by
## `_nwprev_probe.gd` leg `t` with More open. A floored slider also gets
## `DccWidgets.touch_slider()`'s gesture arbitration, because a taller slider
## widens the band where a vertical scroll starting on it would write a value
## instead (`MISTAKES.md`, "Raise a touch target to the 44 dp floor"). The phone
## never shows these controls, and `phone_fit()` treats the phone's own.
func _floor_touch_tree(root: Node) -> void:
	for c in root.get_children(true):
		if c is BaseButton or c is Slider:
			_floor_touch(c as Control)
			if c is HSlider:
				DccWidgets.touch_slider(c as HSlider, 8.0)
		_floor_touch_tree(c)

## Opens or folds the More options body and re-labels its header to say which.
## Public so a caller (or a probe) can open it without hunting the button.
func set_more_open(open: bool) -> void:
	if _more_body == null or _more_toggle == null:
		return
	_more_body.visible = open
	## The mockup's longer caption ("More options (aspect, grid, settlements)")
	## is 328 px in tracked Plex Mono, wider than the 312 px column, so the
	## contents list lives in the tooltip and the caption stays short.
	_more_toggle.text = "%s MORE OPTIONS" % (DccIcons.SYMBOLS["caret"] if open else DccIcons.SYMBOLS["submenu"])

# -- Desktop footer (NWP-3) -----------------------------------------------------

## Desktop and tablet: **Cancel** and **Generate world**, Generate the accent
## fill and rightmost (proposal §2.1; the shipped dialog had only Create and the
## title bar's ✕). Generate is still `AcceptDialog`'s OK button, so
## `confirmed -> _on_create()` is untouched; Cancel is `add_cancel_button()`'s,
## which hides the dialog and emits `canceled` and nothing else.
##
## **The order is set here, not left to Godot**: `add_cancel_button()` places
## Cancel by the platform's `swap_cancel_ok` rule, and `AcceptDialog` centres its
## row between expanding spacers. This keeps the first spacer expanding (pushing
## the pair right), stops the others, and puts Cancel directly before OK.
## `_nwprev_probe.gd` leg `l` measures the result rather than trusting this.
##
## On a touch density the pair is floored through `AcceptDialog`'s own
## `buttons_min_width`/`buttons_min_height` theme constants rather than
## `custom_minimum_size`: the dialog re-applies those constants to its buttons on
## every theme change, so a size set on the button itself does not stick.
func _build_desktop_footer() -> void:
	var ok := get_ok_button()
	_desktop_cancel = add_cancel_button("Cancel")
	_desktop_cancel.name = "nw_cancel"
	_desktop_cancel.focus_mode = Control.FOCUS_NONE
	var hb := ok.get_parent()
	if hb != null:
		hb.move_child(_desktop_cancel, ok.get_index())
		var first := true
		for c in hb.get_children():
			if c is BaseButton or not (c is Control):
				continue
			var ctl := c as Control
			if (ctl.size_flags_horizontal & Control.SIZE_EXPAND) == 0:
				continue
			if first:
				first = false
				hb.move_child(ctl, 0)
			else:
				ctl.size_flags_horizontal = Control.SIZE_FILL
	var px := 14
	var py := 6
	for state in ["normal", "hover", "pressed", "focus", "disabled"]:
		ok.add_theme_stylebox_override(state, DccTheme.button_box(true, state, px, py))
	for key in ["font_color", "font_hover_color", "font_pressed_color", "font_focus_color"]:
		ok.add_theme_color_override(key, DccTheme.c("accent_ink"))
	if DccTheme.is_touch():
		add_theme_constant_override("buttons_min_width", DccTheme.PHONE_TAP_MIN)
		add_theme_constant_override("buttons_min_height", DccTheme.PHONE_TAP_MIN)

## Floors `c` to the 44 dp touch target on BOTH axes on any touch density
## (`MISTAKES.md`, "Floor a control that a rebuild creates": `tablet_fit()`
## floors height only, and this dialog is never walked by it). A no-op on a
## pointer desktop.
func _floor_touch(c: Control) -> void:
	if not DccTheme.is_touch():
		return
	c.custom_minimum_size = Vector2(maxf(c.custom_minimum_size.x, DccTheme.PHONE_TAP_MIN),
		maxf(c.custom_minimum_size.y, DccTheme.PHONE_TAP_MIN))

# -- The preview canvas (NWP-3) ---------------------------------------------------

## The preview frame: a caption, the Ruling CE disclaimer, the canvas (with its
## dash state stacked over it) and the stats line. Built into `parent`, which is
## the left column on desktop and tablet and §6.7's card on a phone.
##
## The canvas is a `TextureRect` over an `ImageTexture` holding the engine's
## mask as an `L8` image (proposal §8 NWP-3), `TEXTURE_FILTER_NEAREST`, scaled up
## to fit with its aspect kept; `MASK_SHADER` maps the 0/1 bytes to the two
## colours, so no GDScript loop touches the 65 600 cells. The stage behind the
## canvas is the `bg` token, never ocean blue: a letterboxed world mask, or the
## dash with no mask at all, must not read as a sea (acceptance (ix)).
func _build_preview(parent: VBoxContainer) -> void:
	var shader := Shader.new()
	shader.code = MASK_SHADER
	_mask_material = ShaderMaterial.new()
	_mask_material.shader = shader
	_mask_material.set_shader_parameter("land_color", PREVIEW_LAND)
	_mask_material.set_shader_parameter("ocean_color", PREVIEW_OCEAN)

	var display_w := PREVIEW_DISPLAY_W_PHONE if _phone else PREVIEW_DISPLAY_W_DESKTOP
	_preview_frame = PanelContainer.new()
	_preview_frame.name = "nw_preview_frame"
	var fsb := DccTheme.flat(DccTheme.c("bg"))
	fsb.border_color = DccTheme.c("line")
	fsb.set_border_width_all(1)
	for side in ["left", "right", "top", "bottom"]:
		fsb.set("content_margin_" + side, 6)
	_preview_frame.add_theme_stylebox_override("panel", fsb)
	## Same reason as `_phone_card()`'s card: a `PanelContainer` defaults to
	## `MOUSE_FILTER_STOP`, which would end a drag that starts on the preview
	## before it reaches the phone card's scroller.
	_preview_frame.mouse_filter = Control.MOUSE_FILTER_PASS
	parent.add_child(_preview_frame)

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 4)
	_preview_frame.add_child(col)

	_preview_title = DccTheme.mono_label("", "text_faint", DccTheme.FS_MICRO, 2, true)
	_preview_title.name = "nw_preview_title"
	col.add_child(_preview_title)

	## Ruling CE's disclaimer: a bordered caption inside the frame, above the
	## canvas, wrapping at every width. Not a tooltip -- a tooltip is not on
	## screen, and on a phone it is not reachable at all.
	var badge := PanelContainer.new()
	badge.name = "nw_preview_badge"
	var bsb := DccTheme.flat(Color(0, 0, 0, 0))
	bsb.border_color = Color(DccTheme.c("stale"), 0.5)
	bsb.set_border_width_all(1)
	bsb.content_margin_left = 6
	bsb.content_margin_right = 6
	bsb.content_margin_top = 2
	bsb.content_margin_bottom = 2
	badge.add_theme_stylebox_override("panel", bsb)
	badge.mouse_filter = Control.MOUSE_FILTER_PASS
	col.add_child(badge)
	_preview_disclaimer = DccTheme.label(PREVIEW_DISCLAIMER, "stale",
		DccTheme.FS_TINY if not DccTheme.is_tablet() else DccTheme.FS_SMALL)
	_preview_disclaimer.name = "nw_preview_disclaimer"
	_preview_disclaimer.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_preview_disclaimer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## An autowrap `Label` with no width of its own reports the height of one
	## word per line until its first layout, and `AcceptDialog` sizes its
	## content child once from that (measured: a 2 880 px tall content rect in
	## a 780 px window, so the right column could not scroll). The badge's
	## inner width is known -- the canvas width less its padding and border --
	## so it is given as a floor.
	_preview_disclaimer.custom_minimum_size.x = PHONE_PREVIEW_LABEL_FLOOR if _phone else display_w - 14
	badge.add_child(_preview_disclaimer)

	var stage := PanelContainer.new()
	stage.name = "nw_preview_stage"
	stage.add_theme_stylebox_override("panel", DccTheme.flat(DccTheme.c("bg")))
	_preview_stage = stage
	stage.custom_minimum_size = Vector2(0.0 if _phone else float(display_w), round(display_w * PREVIEW_REGION_ASPECT))
	stage.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stage.mouse_filter = Control.MOUSE_FILTER_PASS
	col.add_child(stage)

	_preview_canvas = TextureRect.new()
	_preview_canvas.name = "nw_preview_canvas"
	_preview_canvas.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_preview_canvas.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	_preview_canvas.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_preview_canvas.material = _mask_material
	_preview_canvas.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_preview_canvas.visible = false
	stage.add_child(_preview_canvas)

	var center := CenterContainer.new()
	center.mouse_filter = Control.MOUSE_FILTER_IGNORE
	stage.add_child(center)
	var dash_col := VBoxContainer.new()
	dash_col.mouse_filter = Control.MOUSE_FILTER_IGNORE
	center.add_child(dash_col)
	_preview_dash = DccTheme.label("—", "text_faint", DccTheme.FS_HERO)
	_preview_dash.name = "nw_preview_dash"
	_preview_dash.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	dash_col.add_child(_preview_dash)
	_preview_reason = DccTheme.label("", "text_secondary", DccTheme.FS_SMALL)
	_preview_reason.name = "nw_preview_reason"
	_preview_reason.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_preview_reason.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_preview_reason.custom_minimum_size.x = PHONE_PREVIEW_LABEL_FLOOR if _phone else display_w - 40
	dash_col.add_child(_preview_reason)

	_preview_stats = DccTheme.mono_label("—", "text_secondary", DccTheme.FS_TINY)
	_preview_stats.name = "nw_preview_stats"
	_preview_stats.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_preview_stats.custom_minimum_size.x = PHONE_PREVIEW_LABEL_FLOOR if _phone else float(display_w)   ## same reason as the disclaimer's floor
	col.add_child(_preview_stats)
	_set_preview_title(0)
	## Nothing has been asked of the engine yet. The dash says so in words
	## until the first draw replaces it -- never a blank or a blue rectangle.
	_show_preview_dash("", "preview not drawn yet")

## The caption above the canvas: `§ PREVIEW · LAND VS OCEAN · 320 × 205`, its
## height the drawn mask's, or a dash before a mask exists (never a guessed one).
func _set_preview_title(h: int) -> void:
	if _preview_title == null:
		return
	_preview_title.text = "§ PREVIEW · LAND VS OCEAN · %d × %s" % [PREVIEW_W, str(h) if h > 0 else "—"]

## The debounce and the open-dialog poll, and the signals that start them.
## Built once, after the form exists, so no handler fires into half a dialog.
func _build_preview_timers() -> void:
	_preview_debounce = Timer.new()
	_preview_debounce.one_shot = true
	_preview_debounce.wait_time = PREVIEW_DEBOUNCE_S
	_preview_debounce.timeout.connect(_redraw_preview)
	add_child(_preview_debounce)
	_preview_poll = Timer.new()
	_preview_poll.wait_time = PREVIEW_POLL_S
	_preview_poll.timeout.connect(_redraw_preview)
	add_child(_preview_poll)
	visibility_changed.connect(func():
		if visible:
			## Phone: open with the card scrolled to its top, so the preview and
			## Ruling CE's disclaimer on it are the first thing on screen rather
			## than wherever the form was left last time (the dialog is a
			## persistent singleton, so its scroll position survives a close).
			if _card_scroll != null:
				_card_scroll.scroll_vertical = 0
			_preview_poll.start()
			_schedule_preview()
		else:
			_preview_poll.stop())
	## Not required for correctness -- the poll redraws a held preview once
	## `generating` clears -- but it removes the up-to-half-second wait.
	if bridge != null:
		bridge.generation_finished.connect(func(_ok: bool): _schedule_preview())

## Ask for a redraw `PREVIEW_DEBOUNCE_S` from now, restarting the wait, so a
## burst of changes (a held spinner, a typed number) costs one engine call.
## The history strip's "current" border follows the inputs at once (it reads
## no engine state), so it never marks an entry the dialog no longer holds.
func _schedule_preview() -> void:
	_refresh_history_strip()
	if _preview_debounce == null:
		return
	_preview_debounce.start()

## The redraw. **Never touches the engine while a world generates** (the worker
## owns `WorldGen`; proposal §6) -- `bridge.generating` is tested before
## anything else that could reach it, including the binding check. Skips the
## call when nothing it depends on moved since the last answer.
##
## The inputs are the WHOLE live parameter table (`bridge.params_snapshot()`)
## plus the dialog's own seed, width, grid, extent, plates and archetype --
## exactly the dictionary `BINDING.md` (lane A) defines -- so the preview and
## Create read the same values (proposal §3: "if preview and Create ever read
## different values, the preview lies").
func _redraw_preview() -> void:
	if not visible or bridge == null:
		return
	if bridge.generating:
		if preview_state != "held":
			preview_state = "held"
			if _preview_stats != null:
				_preview_stats.text = PREVIEW_HELD
		return
	if not bridge.has_preview_binding():
		_show_preview_dash("no_binding", PREVIEW_REASON_NO_BINDING)
		return
	var params: Dictionary = bridge.params_snapshot()
	if params.is_empty():
		_show_preview_dash("no_params", PREVIEW_REASON_NO_PARAMS)
		return
	params["seed"] = int(seed_input.value)
	params["width_km"] = float(width_input.value)
	params["grid_h"] = int(grid_h_input.value)
	params["archetype"] = _archetype
	params["world"] = extent_input.selected == 1
	## Under an archetype the engine derives its own count and ignores this
	## key (Q8); a Classic world sends the count Create will write.
	params["tect.plates"] = _classic_plates()
	var target_w := int(grid_w_input.value)
	var key := "%s|%d|%d" % [str(params), PREVIEW_W, target_w]
	if key == _preview_key and preview_state in ["drawn", "refused"]:
		return
	_preview_key = key
	var t0 := Time.get_ticks_usec()
	var mask: PackedByteArray = bridge.preview_land_mask(params, PREVIEW_W, target_w)
	var ms := float(Time.get_ticks_usec() - t0) / 1000.0
	if mask.is_empty() or mask.size() % PREVIEW_W != 0:
		_show_preview_dash("refused", PREVIEW_REASON_REFUSED)
		return
	_draw_preview_mask(mask, ms)

## Puts `mask` on the canvas, fills the stats line from it, and pushes a
## history entry if an input that pushes one is pending. `ms` is the measured
## time of this one engine call, shown as it was measured.
func _draw_preview_mask(mask: PackedByteArray, ms: float) -> void:
	var h := mask.size() / PREVIEW_W
	var img := Image.create_from_data(PREVIEW_W, h, false, Image.FORMAT_L8, mask)
	if _preview_tex == null:
		_preview_tex = ImageTexture.create_from_image(img)
	else:
		_preview_tex.set_image(img)
	_preview_canvas.texture = _preview_tex
	_preview_canvas.visible = true
	_preview_dash.get_parent().visible = false
	preview_mask = mask
	preview_h = h
	preview_state = "drawn"
	_set_preview_title(h)
	var land := 100.0 * float(mask.count(1)) / float(mask.size())
	var n := effective_plates()
	## Two explicit lines, and a no-break space inside "N ms": on the 2400x1080
	## emulator run the single wrapping line split "295 / ms" and clipped its
	## second row (STATUS 2026-10-12).
	_preview_stats.text = "land %d%% · %s plates · %s\ndrawn in %d ms" % [
		int(round(land)), str(n) if n >= 0 else "—",
		"whole world, x wraps" if extent_input.selected == 1 else "region", int(round(ms))]
	if _history_push_pending:
		_history_push_pending = false
		_push_history(img, land)
	_refresh_history_strip()

## The dash state: no canvas, a dash and its reason. `state` is what
## `preview_state` becomes ("" for "not drawn yet"). The last mask is dropped,
## not left on screen behind a reason that says there is none.
func _show_preview_dash(state: String, reason: String) -> void:
	preview_state = state
	preview_mask = PackedByteArray()
	preview_h = 0
	if _preview_canvas == null:
		return
	_preview_canvas.visible = false
	_preview_dash.get_parent().visible = true
	_preview_reason.text = reason
	_preview_stats.text = "—  (%s)" % reason
	_set_preview_title(0)

# -- Seed history (NWP-3, proposal §3; Q5: four, session only) -------------------

## The four-thumbnail strip under the seed row. Each slot is a `Button` holding
## a thumbnail and a caption; an empty slot is a disabled button reading "—".
func _build_history(parent: VBoxContainer) -> void:
	var row := HBoxContainer.new()
	row.name = "nw_history"
	row.add_theme_constant_override("separation", 6 if _phone else 8)
	parent.add_child(row)
	var thumb_size := THUMB_SIZE_PHONE if _phone else THUMB_SIZE_DESKTOP
	_history_buttons.clear()
	for i in HISTORY_SLOTS:
		var b := Button.new()
		b.name = "nw_hist_%d" % i
		b.focus_mode = Control.FOCUS_NONE
		b.custom_minimum_size = thumb_size
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		_floor_touch(b)
		var inner := VBoxContainer.new()
		inner.mouse_filter = Control.MOUSE_FILTER_IGNORE
		inner.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
		inner.offset_left = 4
		inner.offset_top = 4
		inner.offset_right = -4
		inner.offset_bottom = -4
		inner.add_theme_constant_override("separation", 2)
		b.add_child(inner)
		var thumb := TextureRect.new()
		thumb.name = "thumb"
		thumb.mouse_filter = Control.MOUSE_FILTER_IGNORE
		thumb.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		thumb.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		thumb.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		thumb.material = _mask_material
		thumb.size_flags_vertical = Control.SIZE_EXPAND_FILL
		inner.add_child(thumb)
		var cap := DccTheme.mono_label("—", "text_secondary", DccTheme.FS_MICRO)
		cap.name = "caption"
		cap.mouse_filter = Control.MOUSE_FILTER_IGNORE
		cap.clip_text = true
		inner.add_child(cap)
		var slot := i
		b.pressed.connect(func(): restore_history(slot))
		row.add_child(b)
		_history_buttons.append(b)
	_refresh_history_strip()

## Adds the just-drawn preview to the front of the history, dropping an older
## entry with the same seed, extent, width and plates (so a repeat does not
## churn the strip), and keeps `HISTORY_SLOTS`.
func _push_history(img: Image, land: float) -> void:
	var e := {
		"seed": int(seed_input.value),
		"world": extent_input.selected == 1,
		"km": float(width_input.value),
		"plates": _classic_plates(),
		"manual": _plates_manual,
		"tex": ImageTexture.create_from_image(img),
		"land": land,
	}
	for k in range(_history.size() - 1, -1, -1):
		var o: Dictionary = _history[k]
		if int(o["seed"]) == int(e["seed"]) and bool(o["world"]) == bool(e["world"]) \
				and is_equal_approx(float(o["km"]), float(e["km"])) and int(o["plates"]) == int(e["plates"]):
			_history.remove_at(k)
	_history.push_front(e)
	while _history.size() > HISTORY_SLOTS:
		_history.pop_back()

## Repaints the four slots from `_history`; the entry matching what the
## dialog now holds gets the accent border.
func _refresh_history_strip() -> void:
	for i in _history_buttons.size():
		var b: Button = _history_buttons[i]
		var thumb := b.find_child("thumb", true, false) as TextureRect
		var cap := b.find_child("caption", true, false) as Label
		var has := i < _history.size()
		b.disabled = not has
		var cur := false
		if has:
			var e: Dictionary = _history[i]
			thumb.texture = e["tex"]
			cap.text = ("%d" % int(e["seed"])) if _phone \
				else "%d · %d%%" % [int(e["seed"]), int(round(float(e["land"])))]
			b.tooltip_text = "Restore seed %d (%s, %s km, %d plates%s)" % [int(e["seed"]),
				"whole world" if bool(e["world"]) else "region", _format_km(float(e["km"])),
				int(e["plates"]), ", set by hand" if bool(e["manual"]) else ""]
			b.accessibility_name = "History: seed %d" % int(e["seed"])
			cur = int(e["seed"]) == int(seed_input.value) \
				and bool(e["world"]) == (extent_input.selected == 1) \
				and is_equal_approx(float(e["km"]), float(width_input.value)) \
				and int(e["plates"]) == _classic_plates()
		else:
			thumb.texture = null
			cap.text = "—"
			b.tooltip_text = "Empty: a dice roll, a typed seed or a size change adds a preview here."
		var token := "accent" if cur else "line"
		for sb_name in ["normal", "hover", "pressed", "disabled", "focus"]:
			var sb := DccWidgets.box(token, "sunken", 4, 4)
			if sb_name == "hover" and has:
				sb.border_color = DccTheme.c("border")
			b.add_theme_stylebox_override(sb_name, sb)

## Restores history slot `i`: seed, extent, width and plates (proposal §3), and
## redraws. **Does not push a history entry** -- a restore that pushed would
## reorder the strip under the user's finger.
func restore_history(i: int) -> void:
	if i < 0 or i >= _history.size():
		return
	var e: Dictionary = _history[i]
	_restoring = true
	seed_input.set_value_no_signal(float(e["seed"]))
	## Off screen a `SpinBox`'s `LineEdit` does not resync its text to a value
	## written by code (`_vfy_seedroll_probe.gd` S0), and this one is on screen,
	## but the restore must read back exactly whatever the case.
	seed_input.get_line_edit().text = str(int(e["seed"]))
	_seed_dirty = false
	var want := 1 if bool(e["world"]) else 0
	if extent_input.selected != want:
		_on_extent_chip(want)
	width_input.set_value_no_signal(float(e["km"]))
	_plates_manual = bool(e["manual"])
	_plates_value = int(e["plates"])
	_refresh_dimensions()
	_restoring = false
	_history_push_pending = false
	_refresh_history_strip()
	_schedule_preview()

## A seed committed in the field, or written by the dice: push and redraw.
func _on_seed_changed(_v: float) -> void:
	if not _restoring:
		_history_push_pending = true
	_schedule_preview()

# -- Plates (NWP-3, proposal §5; Q1, Q3, Q8 under Ruling CE) -------------------

## **The width rule, the one place it lives** (proposal §5, log-linear, Q3):
## `clamp(round(6 + 30 · ln(km/200) / ln(40075/200)), 4, 40)` -- the owner's two
## anchors (200 km, 6) and (40 075 km, 36), log-linear between them, clamped to
## `tect.plates`' own engine range. It passes 800 km at 13.85, i.e. today's
## default of 14, so a default Create is unchanged. A labelled judgement on the
## shape between the anchors; the anchors are the owner's. `km` below 1 cannot
## come from the dialog (the field's floor is 1 km) and is floored there rather
## than fed to `log`.
static func plates_for_km(km: float) -> int:
	var k := maxf(km, 1.0)
	return clampi(int(round(6.0 + 30.0 * log(k / 200.0) / log(40075.0 / 200.0))), 4, 40)

## The count Create writes into `tect.plates` for a Classic world: the hand
## value while it is set by hand, else the width rule.
func _classic_plates() -> int:
	if _plates_manual:
		return _plates_value
	return plates_for_km(width_input.value if width_input != null else 800.0)

## The count the engine will actually use: under an archetype the engine's own
## `clamp(round(4 + fragmentation·24), 4, 40)` (`generate_terrain_inner`; Q8 shows
## it read-only), else `_classic_plates()`. **-1 when it cannot be known** --
## an archetype on a build with no `world_structure.fragmentation` row -- and the
## readout dashes it; never a guessed count.
func effective_plates() -> int:
	if not _archetype.is_empty():
		var frag = bridge.param_get("world_structure.fragmentation") if bridge != null else null
		if frag == null:
			return -1
		return clampi(int(round(4.0 + float(frag) * 24.0)), 4, 40)
	return _classic_plates()

## `tect.plates`' range from the running build's own table, else its
## `params.rs` range.
func _plates_range() -> Vector2i:
	var info: Dictionary = bridge.param_info("tect.plates") if bridge != null else {}
	if info.is_empty():
		return Vector2i(PLATES_MIN_FALLBACK, PLATES_MAX_FALLBACK)
	return Vector2i(int(info.get("min", PLATES_MIN_FALLBACK)), int(info.get("max", PLATES_MAX_FALLBACK)))

## The Plates row: − *n* + with its source under it ("auto from width", or "set
## by hand ·" and a reset button, or "from archetype" read-only). Each button a
## 44 dp square on touch, the mockup's 28 x 24 on a pointer desktop.
func _build_plates_row(parent: VBoxContainer) -> void:
	var row := DccWidgets._row(parent, "Plates",
		"How many tectonic plates the world is built from (tect.plates). Auto follows the map width: 6 at 200 km, 14 at 800 km (today's default), 36 at 40 075 km. − and + set it by hand for this dialog; Create writes it into the engine parameter.")
	row.name = "nw_plates_row"
	## Phone: the mockup's phone row label column (`.phone .row>label{flex:0 0
	## 108px}`), not the shared 132 px: with two 44 dp buttons the 132 px column
	## made this the card's widest row and pushed §6.7's card to 364 dp past its
	## 360 dp cap (measured, `_nwprev_probe.gd` leg `v`).
	if _phone:
		(row.get_child(0) as Control).custom_minimum_size.x = PHONE_ROW_LABEL_W
	_plates_minus = _plates_button(row, "-", "nw_plates_minus", -1)
	_plates_readout = DccTheme.mono_label("—", "accent", DccTheme.FS_READOUT)
	_plates_readout.name = "nw_plates_value"
	_plates_readout.custom_minimum_size.x = 32
	_plates_readout.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	row.add_child(_plates_readout)
	_plates_plus = _plates_button(row, "+", "nw_plates_plus", 1)
	var src := HBoxContainer.new()
	src.add_theme_constant_override("separation", 4)
	var indent := Control.new()
	indent.custom_minimum_size.x = (PHONE_ROW_LABEL_W if _phone else DccWidgets.ROW_LABEL_W) + 8
	src.add_child(indent)
	_plates_source = DccTheme.mono_label("", "text_faint", DccTheme.FS_TINY)
	_plates_source.name = "nw_plates_source"
	src.add_child(_plates_source)
	_plates_reset = Button.new()
	_plates_reset.name = "nw_plates_reset"
	_plates_reset.text = "reset"
	_plates_reset.focus_mode = Control.FOCUS_NONE
	_plates_reset.add_theme_font_override("font", DccTheme.mono(0))
	_plates_reset.add_theme_font_size_override("font_size", DccTheme.FS_TINY)
	_plates_reset.add_theme_color_override("font_color", DccTheme.c("accent"))
	_plates_reset.add_theme_color_override("font_hover_color", DccTheme.c("accent_hover"))
	for sb_name in ["normal", "hover", "pressed", "focus"]:
		_plates_reset.add_theme_stylebox_override(sb_name, DccTheme.inset(4, 0, 4, 0))
	_plates_reset.tooltip_text = "Back to the count from the map width."
	_floor_touch(_plates_reset)
	_plates_reset.pressed.connect(func():
		_plates_manual = false
		_refresh_plates()
		_schedule_preview())
	src.add_child(_plates_reset)
	parent.add_child(src)
	_refresh_plates()

## One of the Plates row's two step buttons, styled like the seed dice.
func _plates_button(row: Control, glyph: String, node_name: String, step: int) -> Button:
	var b := Button.new()
	b.name = node_name
	b.text = glyph
	b.focus_mode = Control.FOCUS_NONE
	b.custom_minimum_size = Vector2(DccTheme.PHONE_TAP_MIN, DccTheme.PHONE_TAP_MIN) \
		if DccTheme.is_touch() else ICON_BTN_DESKTOP
	b.add_theme_font_override("font", DccTheme.mono(0, true))
	b.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
	var r := 12 if DccTheme.is_touch() else 2
	var quiet := DccTheme.pill(false, r, 4, 0)
	var lit := DccTheme.pill(false, r, 4, 0)
	lit.bg_color = DccTheme.c("line_soft")
	b.add_theme_stylebox_override("normal", quiet)
	b.add_theme_stylebox_override("hover", lit)
	b.add_theme_stylebox_override("pressed", lit)
	b.add_theme_stylebox_override("disabled", DccTheme.pill(false, r, 4, 0))
	b.accessibility_name = "%s plates" % ("Fewer" if step < 0 else "More")
	b.pressed.connect(func(): _plates_step(step))
	row.add_child(b)
	return b

## − or +: sets the count by hand from whatever is shown, within the engine's
## range. Refused under an archetype, whose count is the engine's (Q8).
func _plates_step(step: int) -> void:
	if not _archetype.is_empty():
		return
	var rng := _plates_range()
	_plates_value = clampi(_classic_plates() + step, rng.x, rng.y)
	_plates_manual = true
	_refresh_plates()
	_schedule_preview()

## Repaints the readout, its source line and the two buttons' enabled state.
func _refresh_plates() -> void:
	if _plates_readout == null:
		return
	var n := effective_plates()
	var arche := not _archetype.is_empty()
	var rng := _plates_range()
	_plates_readout.text = str(n) if n >= 0 else "—"
	_plates_minus.disabled = arche or n <= rng.x
	_plates_plus.disabled = arche or n >= rng.y
	if arche:
		_plates_source.text = "from archetype (read-only)" if n >= 0 \
			else "from archetype: this build publishes no fragmentation dial"
	elif _plates_manual:
		_plates_source.text = "set by hand ·"
	else:
		_plates_source.text = "auto from width"
	_plates_reset.visible = _plates_manual and not arche

## Q1 (Ruling CE): **Create writes the plate count it previewed** into
## `tect.plates`, the same engine parameter World ▸ Generate ▸ 04 Tectonics
## edits -- so the world has the plates the preview showed. At the default
## 800 km that is 14, today's default, so a default Create is unchanged; every
## other width changes generated output, which is exactly what the owner
## approved. Under an archetype nothing is written: the engine derives its own
## count from fragmentation and ignores `tect.plates` (Q8). Refused, like every
## `param_set`, while a world is generating.
func _apply_plates_to_engine() -> void:
	if not _archetype.is_empty() or bridge == null:
		return
	bridge.param_set("tect.plates", _classic_plates())

## §6.7's action row: CANCEL left at `flex:1`, CREATE WORLD right at `flex:1.4`,
## both spanning the card at `min-height:46px` and `border-radius:23px`.
##
## **Not `DccWidgets.phone_pill()`**, which is the 412 canvas's *other* button
## treatment -- a 48 dp pill that is either accent-filled or outlined. §6.7
## draws neither outline: CANCEL is a `--chip` wash (`rgba(255,255,255,.05)` on
## dark, which is exactly `Color(c("text_bright"), 0.05)`) with `--sec` ink, and
## CREATE WORLD is `--acc` with `--accInk`. Two washes, no border, and a 46 dp
## height rather than 48. Built here rather than added to `phone_pill()` so the
## shell's own pill treatment is untouched.
func _build_phone_actions(card: Control) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", PHONE_ACTION_GAP)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_top", PHONE_ACTION_TOP)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(row)
	card.add_child(pad)
	_phone_cancel = _phone_action(row, "CANCEL", false, PHONE_CANCEL_FLEX)
	_phone_create = _phone_action(row, "CREATE WORLD", true, PHONE_CREATE_FLEX)
	## `hide()` before the work, and `_on_create()` by name rather than
	## `confirmed.emit()`: the signal is connected in `setup()` and emitting it
	## from here would run the same handler through one more indirection with
	## nothing gained. `hide()` first because `generate()` is the long call and the
	## modal has no business staying up over it.
	_phone_cancel.pressed.connect(hide)
	_phone_create.pressed.connect(func():
		hide()
		_on_create())

## One pill button of the phone's pinned action row (CANCEL / CREATE WORLD).
## `primary` selects the accent fill, otherwise a faint wash; `flex` is the
## stretch ratio so Create is wider than Cancel. Hover and pressed are a lift on
## the same wash, not a second colour. Adds the button to `row` and returns it.
func _phone_action(row: Control, caption: String, primary: bool,
		flex: float) -> Button:
	var b := Button.new()
	b.text = caption
	b.focus_mode = Control.FOCUS_NONE
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	b.size_flags_stretch_ratio = flex
	b.custom_minimum_size.y = PHONE_ACTION_H
	b.add_theme_font_override("font", DccTheme.mono(PHONE_ACTION_TRACKING, true))
	b.add_theme_font_size_override("font_size", PHONE_ACTION_FS)
	var fill := DccTheme.c("accent") if primary else Color(DccTheme.c("text_bright"), 0.05)
	var ink := DccTheme.c("accent_ink") if primary else DccTheme.c("text_secondary")
	## The pressed/hover state is a lift on the same wash rather than a second
	## colour, so a finger that lands and lifts sees one thing move.
	var lit := DccTheme.c("accent_hover") if primary \
		else Color(DccTheme.c("text_bright"), 0.11)
	for sb_name in ["normal", "disabled"]:
		b.add_theme_stylebox_override(sb_name, DccTheme.flat(fill, PHONE_ACTION_RADIUS))
	for sb_name in ["hover", "pressed"]:
		b.add_theme_stylebox_override(sb_name, DccTheme.flat(lit, PHONE_ACTION_RADIUS))
	b.add_theme_stylebox_override("focus", DccTheme.empty())
	for key in ["font_color", "font_hover_color", "font_pressed_color"]:
		b.add_theme_color_override(key, ink)
	row.add_child(b)
	return b

## §6.7's card. The window around it is `DccWidgets.phone_present()`'s
## full-screen fill -- `app.gd` makes that call and owns it -- so what is
## buildable here is the card itself: a bounded column on its own rounded
## surface, rather than the desktop form run edge to edge. The scrim §6.7 draws
## the card over is this window's opaque panel instead, and that is the half
## this file cannot reach.
##
## Told apart from the dialog's surface by its **border**, not its fill:
## `dcc_shell.gd` themes `AcceptDialog`'s own panel `c("panel")`, and picking a
## card fill for contrast alone is the (2,3,4) mistake `MISTAKES.md` records
## against the 2026-08-31 token re-base. `sunken` over `panel` is (7,9,10) and
## the hairline is what actually draws the edge.
func _phone_card(parent: Control) -> VBoxContainer:
	var center := CenterContainer.new()
	center.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(center)

	_card = PanelContainer.new()
	var sb := DccTheme.flat(DccTheme.c("sunken"), PHONE_CARD_RADIUS)
	sb.border_color = DccTheme.c("border")
	sb.set_border_width_all(1)
	_card.add_theme_stylebox_override("panel", sb)
	## **A drag that starts on the card has to reach the scroll above it.**
	## PH-05's rule, in the one place `DccShell.phone_fit()` cannot apply it:
	## that walk converts a `BoxContainer`/`MarginContainer` at
	## `MOUSE_FILTER_STOP` to `PASS` and **excludes `PanelContainer`**, because
	## several in this shell carry their own `gui_input`. This one does not, and
	## a `PanelContainer` really does default to `STOP` where every other
	## `Container` defaults to `PASS` -- measured, not assumed:
	## `_nwaction_probe.gd` prints the filter chain from the card's last
	## advisory `Label` upward and it read
	## `Label=2 -> VBoxContainer=1 -> MarginContainer=1 -> PanelContainer=0
	##  -> CenterContainer=1 -> VBoxContainer=1 -> ScrollContainer=1`.
	## The `0` is this node, and it ended the event walk before the scroller.
	##
	## It cost nothing while the card fitted (688 dp of card in a 702 dp
	## viewport). §6.7's action row is 46 dp plus 16 dp of `padding-top`, which
	## took the card to **752 dp** and put CREATE WORLD 50 dp below the fold.
	## Found on glass: a real `adb shell input swipe` on the card moved **zero
	## pixels**.
	##
	## **The card no longer overflows** -- `_fit_phone_card_height()` caps it --
	## so this filter is not what makes the primary action reachable any more.
	## It is still required, and for the original reason: the FORM inside the
	## card is what scrolls now, and a drag that starts on the card's padding
	## still has to reach `_card_scroll` rather than die on this panel.
	_card.mouse_filter = Control.MOUSE_FILTER_PASS
	center.add_child(_card)

	## §6.7's `padding:18px 16px 16px`.
	var pad := MarginContainer.new()
	_card_pad = pad
	pad.add_theme_constant_override("margin_left", 16)
	pad.add_theme_constant_override("margin_right", 16)
	pad.add_theme_constant_override("margin_top", 18)
	pad.add_theme_constant_override("margin_bottom", 16)
	_card.add_child(pad)

	## **The card is two bands, not one column, and that is the first-run fix.**
	##
	## Everything used to be one `VBoxContainer` inside the outer scroller, so
	## the card grew to its content (752 dp measured) inside a 702 dp viewport
	## and §6.7's action row -- the last 62 dp of it -- was simply below the
	## fold. The pass above made the card draggable so the row could be reached;
	## a cold launch by hand then found the real defect, which is that
	## **nothing says so**: a tap on the visible orange sliver left the screen
	## pixel-identical, and a first-time user meets that before reaching a world.
	##
	## So the form scrolls and the row does not. `_card_scroll` is capped to
	## whatever is left after the panel's padding and the row
	## (`_fit_phone_card_height()`), which keeps the card inside the viewport by
	## construction rather than by any content being short enough -- and keeps
	## the row where §6.7 draws it, inside the card, rather than detaching it
	## into a footer below.
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(col)

	## Landscape (`PHONE_LAND_MIN_DP`): `_card_bands` seats the preview column
	## (`_land_scroll`, hidden in portrait) beside `_card_scroll`. In portrait it
	## holds only `_card_scroll`, and an HBox with one expanding child lays that
	## out exactly as the bare scroller was.
	_card_bands = HBoxContainer.new()
	_card_bands.add_theme_constant_override("separation", 16)
	_card_bands.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.add_child(_card_bands)
	_land_scroll = ScrollContainer.new()
	_land_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_land_scroll.custom_minimum_size.x = PHONE_LAND_LEFT_W
	_land_scroll.visible = false
	_card_bands.add_child(_land_scroll)
	## The right-hand column: the form's scroller over the pinned action row. In
	## portrait it is the whole card body (as `col` was), in landscape it sits
	## beside the preview column so that column gets the row's height too.
	_card_actions = VBoxContainer.new()
	_card_actions.add_theme_constant_override("separation", 0)
	_card_actions.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_card_bands.add_child(_card_actions)

	_card_scroll = ScrollContainer.new()
	## Same reason the outer scroller disables it: the form's width is §6.7's
	## `max-width:360px` minus the padding, and a horizontal scrollbar under a
	## 360 dp card would be a second way to lose a control off the edge.
	_card_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_card_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_card_actions.add_child(_card_scroll)

	_card_form = VBoxContainer.new()
	_card_form.add_theme_constant_override("separation", 2)
	_card_form.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_card_scroll.add_child(_card_form)

	_fit_phone_card()
	return _card_form

## `min(360, screen - 2x22)`, in the window's own content-scale units -- §6.7's
## `max-width` and its scrim padding, together.
##
## Measured off the HOST viewport rather than `self.size`, for two reasons that
## both bite: `setup()` runs at startup, long before this dialog is ever shown,
## and `about_to_popup` fires from inside `Window.popup()` *before* the new
## rect is applied. The host's viewport and `phone_scale()` are what
## `phone_present()` itself divides, so this is the same measurement it makes.
##
## **On every mainstream handset this returns 360, and that is not because the
## number is hardcoded.** `dcc_shell.gd` sets `_phone_scale` to
## `max(1.0, short_side / PHONE_REF_SHORT)`, so a device whose short side is at
## or above 412 px normalises to exactly 412 dp whatever its resolution:
## measured 412 dp at 1080x2400, 1080x2340, 720x1600 and 1440x3120 alike
## (`_nwcard_probe.gd`), and 915 dp in landscape at 2400x1080 — all four clamp
## to §6.7's `max-width`. The subtraction is what a screen *narrower* than the
## reference gets, where the scale floors at 1.0 and the dp is the real width:
## 380x800 measures 380 dp and a 336 px card, in the same probe.
func _fit_phone_card() -> void:
	if _card == null:
		return
	var host := get_parent()
	if host == null or not host.has_method("phone_scale"):
		return
	var scale: float = host.phone_scale()
	if scale <= 0.0:
		return
	var dp := int(host.get_viewport_rect().size.x / scale)
	var dp_h := int(host.get_viewport_rect().size.y / scale)
	var land := dp >= PHONE_LAND_MIN_DP and dp > dp_h
	_apply_phone_landscape(land)
	_card.custom_minimum_size.x = float(
		clampi(dp - 2 * PHONE_CARD_INSET, PHONE_CARD_MIN_W,
			PHONE_CARD_MAX_W_LAND if land else PHONE_CARD_MAX_W))

## Moves the card between its portrait form (one scrolling column, preview
## first) and its landscape form (preview column left, everything else right,
## each scrolling on its own, the Cancel / Create row still pinned under both).
## The preview group is the only node that changes parent, so no control is
## rebuilt and no handler re-bound; a no-op when the orientation is unchanged.
## It also trims what costs height a rotated card does not have: the card's
## vertical padding, the preview stage and the "NEW WORLD" header band (the
## dialog's identity stays in the Create / Cancel row and the preview caption).
## Must never touch desktop or tablet: `_grp_preview` is null there.
func _apply_phone_landscape(land: bool) -> void:
	if _grp_preview == null or _land_scroll == null or land == _phone_land:
		return
	_phone_land = land
	var from := _grp_preview.get_parent()
	if from != null:
		from.remove_child(_grp_preview)
	if land:
		_land_scroll.add_child(_grp_preview)
		_grp_preview.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	else:
		_card_form.add_child(_grp_preview)
		_card_form.move_child(_grp_preview, 0)
	_land_scroll.visible = land
	if _phone_head_wrap != null:
		_phone_head_wrap.visible = not land
	if _card_pad != null:
		_card_pad.add_theme_constant_override("margin_top", PHONE_LAND_PAD_V if land else 18)
		_card_pad.add_theme_constant_override("margin_bottom", PHONE_LAND_PAD_V if land else 16)
	if _preview_stage != null:
		_preview_stage.custom_minimum_size.y = float(PHONE_LAND_STAGE_H) if land \
			else round(PREVIEW_DISPLAY_W_PHONE * PREVIEW_REGION_ASPECT)
	## The history thumbnails drop to the 44 dp touch floor in landscape: at
	## their portrait height they pushed the extent chips under the card's fold.
	for hb: Button in _history_buttons:
		hb.custom_minimum_size.y = float(DccTheme.PHONE_TAP_MIN) if land else THUMB_SIZE_PHONE.y
	_land_scroll.scroll_vertical = 0
	_fit_phone_card_height.call_deferred()

## **The card's height, so its action row cannot leave the viewport.**
##
## `_card_scroll` is the only elastic thing in the card, so capping it caps the
## card: the panel's `18/16` padding and the pinned action row are fixed, and
## `_card`'s minimum is therefore `chrome + cap` -- at most the scroller it sits
## in. `CenterContainer` then draws it at exactly that, and the outer scroller
## has nothing left to scroll.
##
## **`chrome` is read off the live tree, not added up from the constants.** It
## is the card's own minimum less the band being capped, so `PHONE_ACTION_H`,
## `PHONE_ACTION_TOP`, the panel's padding and the stylebox's own margins are
## all counted whatever they become. Adding them here would be a second copy of
## the geometry that goes stale the first time one of them moves.
##
## **No new minimum escapes.** `_card_scroll`'s vertical mode is AUTO, so the
## form's height does not fold into it (`MISTAKES.md`: only a DISABLED axis
## does that), and the value written below is `<= avail - chrome` by
## construction, so the card can never demand more height than the scroller
## already has. The horizontal axis is untouched -- the card's width minimum is
## still `_fit_phone_card()`'s clamp above and nothing here contributes to it.
func _fit_phone_card_height() -> void:
	if _card == null or _card_scroll == null or _card_form == null or _card_bands == null:
		return
	if _outer_scroll == null:
		return
	## **`size.y` is not the room.** A `ScrollContainer` lays its child inside
	## its own panel stylebox, and the shell's is `6 px` top and bottom
	## (measured: `_nwclip_probe.gd` prints the frame). Capping against the
	## outer figure left the card 12 dp too tall -- CREATE WORLD cleared the
	## edge but the card's rounded bottom border was cut, which reads as *more
	## content below* and is the affordance this fix exists to remove.
	var frame := 0.0
	var sb := _outer_scroll.get_theme_stylebox("panel")
	if sb != null:
		frame = sb.get_minimum_size().y
	var avail := _outer_scroll.size.y - frame
	if avail <= 0.0:
		return
	## `_card_bands` (not `_card_scroll`) is the band whose height is capped: it
	## is `_card_scroll` alone in portrait and the taller of the two columns in
	## landscape, and an HBox's minimum height is its tallest child's.
	var chrome := _card.get_combined_minimum_size().y \
		- _card_bands.get_combined_minimum_size().y
	## The pinned row under the form scroller (the part of `_card_actions` that is
	## not `_card_scroll`); in landscape the preview column does not carry it.
	var row_h := _card_actions.get_combined_minimum_size().y \
		- _card_scroll.get_combined_minimum_size().y
	var capped := minf(_card_form.get_combined_minimum_size().y,
		maxf(0.0, avail - chrome - row_h))
	var capped_left := minf(_grp_preview.get_combined_minimum_size().y,
		maxf(0.0, avail - chrome)) if _phone_land else capped
	## Guarded so an assignment that changes nothing does not queue a layout
	## pass -- `custom_minimum_size` notifies unconditionally, and this runs
	## from `resized`.
	if absf(_card_scroll.custom_minimum_size.y - capped) > 0.5:
		_card_scroll.custom_minimum_size.y = capped
	if absf(_land_scroll.custom_minimum_size.y - capped_left) > 0.5:
		_land_scroll.custom_minimum_size.y = capped_left

## §6.7's dice button -- `44 x 42`, radius 12, glyph `⚄` -- drawn with
## `DccIcons`' own `dice` mark rather than U+2684. That is `dcc_icons.gd`'s
## standing rule and not a preference: a typographic symbol stays text only
## where the glyph exists somewhere in the fallback chain, and that file
## records 19 of its own 24 `SYMBOLS` missing from Plex Mono outright.
## `world_workspace.gd`'s sculpt-seed dice is the same mark at the same job.
##
## Height is the shell's `PHONE_TAP_MIN` (44), not §6.7's 42: the floor wins
## over the artboard where they disagree by two pixels. **Built on every
## density since NWP-3** (it was phone-only, which left desktop with no roll
## inside the dialog -- proposal §1.3 item 3): 44 x 44 on any touch density,
## the mockup's 28 x 24 `.iconbtn` on a pointer desktop. Disabled while the
## seed lock is on (`_set_seed_locked()`).
##
## Sits in the row `DccWidgets.number()` just built, so the label column and
## the spin box keep the shell's own row geometry, and it fires
## `randomise_seed()` -- the same action the **left dock's** WORLD ▸ Generate
## category reaches through `app.gd::_new_seed()`, so the phone gets the
## desktop's reroll rather than a second behaviour that happens to look like it.
## (Corrected 2026-09-05: this said `World ▸ Generate ▸ New seed`, implying a
## menu row. There is none — `grep 'New seed' menus.gd` and `command_index.gd`
## both return nothing, so it is not searchable either. Its only home is
## `world_workspace.gd`'s Generate category.)
func _build_seed_dice() -> void:
	var row := seed_input.get_parent() as Control
	if row == null:
		return
	var dice := Button.new()
	dice.name = "nw_dice"
	dice.icon = DccIcons.get_icon("dice", 14)
	dice.focus_mode = Control.FOCUS_NONE
	dice.icon_alignment = HORIZONTAL_ALIGNMENT_CENTER
	dice.custom_minimum_size = Vector2(DccTheme.PHONE_TAP_MIN, DccTheme.PHONE_TAP_MIN) \
		if DccTheme.is_touch() else ICON_BTN_DESKTOP
	dice.tooltip_text = "New seed — reroll to a fresh random world and redraw the preview. The same reroll as the WORLD dock's Generate category. Disabled while the seed is locked."
	## **A tooltip is not a name on a handset.** There is no hover, so an
	## icon-only button's tooltip is unreachable by any route -- the same fault
	## `dcc_shell.gd::_phone_fit_tool_button()` was written to fix for the tool
	## palette. `accessibility_name` is the route that survives without a
	## pointer, and `dcc_shell.gd` already fills it from `tooltip_text` for its
	## own icon buttons; one short name reads better than a whole sentence.
	dice.accessibility_name = "New seed"
	var quiet := DccTheme.pill(false, 12, 10, 4)
	var lit := DccTheme.pill(false, 12, 10, 4)
	lit.bg_color = DccTheme.c("line_soft")
	dice.add_theme_stylebox_override("normal", quiet)
	dice.add_theme_stylebox_override("hover", lit)
	dice.add_theme_stylebox_override("pressed", lit)
	## A disabled dice (seed locked) keeps the quiet box and is dimmed, so it
	## reads as unavailable rather than vanishing.
	var off := DccTheme.pill(false, 12, 10, 4)
	dice.add_theme_stylebox_override("disabled", off)
	dice.pressed.connect(randomise_seed)
	row.add_child(dice)
	_dice = dice

## The seed lock (proposal §3, Q4: on for the session, never saved). A toggle
## beside the dice, reading LOCK / LOCKED in Plex Mono rather than a glyph:
## `DccIcons` carries no lock mark, and `SYMBOLS["locked"]` (U+1F512) is not in
## Plex Mono, so it would draw only through `DccTheme.mono()`'s system-font
## fallback, without Plex's metrics; the word stays. (Corrected 2026-10-19:
## this said the bundled fonts could not draw the glyph at all. What failed
## was the entry's own escape, `"\U0001F512"`, which GDScript 4 reads as
## U+01F5 + "12" and drew as ǵ12; fixed in `dcc_icons.gd`, and the glyph now
## renders.) While on, the dice is
## disabled and `randomise_seed()` returns without rolling. A typed seed still
## wins: the user typed it (proposal §3).
func _build_seed_lock() -> void:
	var row := seed_input.get_parent() as Control
	if row == null:
		return
	var lock := Button.new()
	lock.name = "nw_lock"
	lock.toggle_mode = true
	lock.focus_mode = Control.FOCUS_NONE
	lock.add_theme_font_override("font", DccTheme.mono(1, true))
	lock.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
	lock.custom_minimum_size = Vector2(DccTheme.PHONE_TAP_MIN, DccTheme.PHONE_TAP_MIN) \
		if DccTheme.is_touch() else Vector2(52, ICON_BTN_DESKTOP.y)
	lock.tooltip_text = "Lock this seed: the dice, and New seed in the WORLD dock or tool bar, leave it alone for this session. Typing a seed still changes it."
	lock.toggled.connect(func(on: bool): _set_seed_locked(on))
	row.add_child(lock)
	_lock = lock
	_set_seed_locked(false)

## Sets the lock and repaints both buttons it affects. The lock's lit state is a
## border, a wash and an ink step, so its name carries the state for a reader
## with no sight of them (as `_set_extent_chips()` does for the chips).
func _set_seed_locked(on: bool) -> void:
	seed_locked = on
	if _lock != null:
		_lock.set_pressed_no_signal(on)
		_lock.text = "LOCKED" if on else "LOCK"
		_lock.accessibility_name = "Seed lock (%s)" % ("on" if on else "off")
		var token := "accent" if on else "border"
		var wash := "accent_wash_2" if on else ""
		var r := 12 if DccTheme.is_touch() else 2
		for sb_name in ["normal", "hover", "pressed", "focus"]:
			var sb := DccWidgets.box(token, wash, 6, 2)
			sb.set_corner_radius_all(r)
			_lock.add_theme_stylebox_override(sb_name, sb)
		var ink := DccTheme.c("accent") if on else DccTheme.c("text_secondary")
		for key in ["font_color", "font_hover_color", "font_pressed_color", "font_focus_color"]:
			_lock.add_theme_color_override(key, ink)
	if _dice != null:
		_dice.disabled = on
		_dice.modulate.a = 0.35 if on else 1.0

## §6.7's two extent chips -- `flex:1`, `min-height:42px`, radius 14, `REGION`
## (the default) and `WORLD` -- in place of the dropdown the desktop form used
## to show.
## Two mutually exclusive values is what a segmented pair is for, and the
## alternative on a handset is an `OptionButton` whose popup rows
## `dcc_shell.gd::phone_fit()` measured at about 21 dp.
##
## They **drive** the dropdown rather than replacing it: `extent_input` is
## still built (hidden, in `_build()`'s `hidden` holder), because `_update_extent_state()`,
## `_derived_grid_h()` and `_refresh_dimensions()` all read
## `extent_input.selected`, and a second source of truth for one fact is how
## those three come to disagree.
##
## **On every density since NWP-3** (proposal §2.1: "Extent: a segmented pair, as
## on the phone"), so the dropdown is hidden on desktop and tablet too. Height:
## the 44 dp floor on any touch density; the segment's own height on a pointer
## desktop. Returns the section so the caller can hang the extent note under it.
func _build_extent_chips(parent: Control) -> VBoxContainer:
	var sec := DccWidgets.section(parent, "Extent")
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10 if _phone else 0)
	sec.add_child(row)
	for i in 2:
		var label := ("REGION" if i == 0 else "WORLD") if _phone \
			else ("Region" if i == 0 else "Whole world")
		var chip := DccWidgets.segment(row, label, func(): _on_extent_chip(i))
		chip.name = "nw_extent_%s" % ("region" if i == 0 else "world")
		chip.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		if DccTheme.is_touch():
			chip.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN
		chip.tooltip_text = EXTENT_NOTE_REGION if i == 0 else EXTENT_NOTE_WORLD
		_extent_chips.append(chip)
	_set_extent_chips()
	return sec

## Assigning `OptionButton.selected` does not emit `item_selected` -- only a
## user's pick does -- so the handler is called by name here rather than left
## to fire twice or not at all.
func _on_extent_chip(index: int) -> void:
	extent_input.selected = index
	_on_extent_selected(index)

## Every route into an extent has to reach the chips, not only the tap that
## took one. There are three, and hanging the refresh off
## `_update_extent_state()` rather than off `_on_extent_chip()` is what covers
## all three: a chip tap; `setup()`'s own first call, for the value
## `param_get("world")` seeded the dropdown with; and `extent_input`'s own
## `item_selected`, which no phone user can reach today because the dropdown
## is hidden — and which stays covered anyway rather than depending on that.
func _set_extent_chips() -> void:
	for i in _extent_chips.size():
		var chip: Button = _extent_chips[i]
		var on := extent_input.selected == i
		DccWidgets.set_segment_on(chip, on)
		## The lit state is a border colour, a wash and an ink step -- three
		## visual facts and nothing a non-visual reader can reach. A `Button`
		## that is not in `toggle_mode` exposes no pressed state either, so the
		## selection rides in the name, refreshed here with it rather than set
		## once at build where it would immediately be wrong.
		chip.accessibility_name = "%s extent%s" % [
			chip.text.capitalize(), " (selected)" if on else ""]
		## §6.7's radius 14. `set_segment_on()` installs §11's square desktop
		## box; rounding the three instances it just made keeps the phone
		## radius here instead of putting it on every segment in the shell.
		## Phone only: desktop and tablet keep the square segment pair.
		if not _phone:
			continue
		for sb_name in ["normal", "pressed", "disabled"]:
			var sb := chip.get_theme_stylebox(sb_name) as StyleBoxFlat
			if sb != null:
				sb.set_corner_radius_all(PHONE_CHIP_RADIUS)

## The three settlement-placement dials the civ `PARAMS` group added
## (`LARGE_ITEM_RULINGS.md`, owner 2026-08-31). Written straight through
## `param_set` rather than held as dialog state, exactly as the extent switch
## above already does: they are live `WorldParams` rows, so Create reads
## whatever they currently hold and CIVIL ▸ Settlements ▸ Auto-populate
## re-reads them over an existing world without regenerating terrain -- which
## is the whole point of the ruling.
##
## Drawn only when the running extension publishes them. An older cdylib has
## no `civ.factions` row, and `param_set` on an unknown key is *reported*
## rejected rather than silently ignored -- but a control whose write the
## engine refuses is an inert control, which this shell does not draw.
func _build_placement_dials(parent: Control) -> void:
	if bridge.param_info("civ.factions").is_empty():
		return
	## `param_get` answers `null` while a generation holds the object and
	## before the table has been probed, so every read falls back to the row's
	## own `default` out of `param_info` -- the engine's number either way,
	## never one transcribed here.
	var fac: Dictionary = bridge.param_info("civ.factions")
	DccWidgets.number(parent, "Factions", float(fac.get("min", 1.0)), float(fac.get("max", 24.0)),
		1.0, _param_or_default("civ.factions", fac),
		func(v: float): bridge.param_set("civ.factions", int(v)),
		"How many rival polities settlement placement distributes into. The reference's own CIV_FACTIONS carries six. Faction 0 is \"Unclaimed\" and is not counted, so the floor is one. Renaming, recolouring and adding factions by hand stays in CIVIL ▸ Factions -- this is only what a fresh placement pass seeds into.")
	var thr: Dictionary = bridge.param_info("civ.seed_thresh")
	var thr_row: Dictionary = DccWidgets.slider(parent, "Suitability floor", float(thr.get("min", 0.1)), float(thr.get("max", 0.8)),
		float(thr.get("step", 0.01)), _param_or_default("civ.seed_thresh", thr), "",
		func(v: float): bridge.param_set("civ.seed_thresh", v),
		"Reference SETTLE_SEED_THRESH, default 0.42. How good a site has to be before anyone settles it at all. Raise it and only the best land is occupied; lower it and marginal ground gets villages. Suitability rarely exceeds 0.8, so the top of this range settles almost nothing.")
	var sup: Dictionary = bridge.param_info("civ.seed_suppress_div")
	var sup_row: Dictionary = DccWidgets.slider(parent, "Packing", float(sup.get("min", 8.0)), float(sup.get("max", 60.0)),
		float(sup.get("step", 1.0)), _param_or_default("civ.seed_suppress_div", sup), "",
		func(v: float): bridge.param_set("civ.seed_suppress_div", v),
		"How close two settlements may stand. The engine suppresses a new seed within max(6, grid width / this) cells of an existing one, so the reference's default of 22 is a 17-cell radius on a 384-wide grid. Higher packs them tighter; the six-cell floor is what stops a small grid from suppressing nothing.")
	_build_fixed_counts([thr_row, sup_row], parent)

## The reference's Auto-populate count inputs (`civNCap`..`civNHam`), as one
## switch and five counts -- `CivParams::want_counts` says why a switch rather
## than five blank-able fields. On, the engine SUBSTITUTES the two dials above
## (threshold 0.35, spacing derived from the total) exactly as the reference
## does, so they are greyed out rather than left looking live. Absent on an
## older cdylib, for the same reason the dials are.
const COUNT_ROWS := [["civ.n_capital", "Capitals"], ["civ.n_city", "Cities"],
	["civ.n_town", "Towns"], ["civ.n_village", "Villages"], ["civ.n_hamlet", "Hamlets"]]
## One `SpinBox` per `COUNT_ROWS` tier, in row order.
var _count_boxes: Array[SpinBox] = []
## The two automatic placement sliders (the `dials` argument) that are greyed while fixed counts are on.
var _count_dials: Array = []
## Explanatory line under the count boxes; its text depends on the toggle and the total.
var _count_note: Label
## The "Fixed settlement counts" check box.
var _count_toggle: CheckBox

## Fixed settlement counts: a toggle plus one spin box per tier in `COUNT_ROWS`.
## `dials` are the two automatic placement sliders, which `_refresh_fixed_counts`
## greys while the counts are in force. Draws nothing when the engine build has no
## `civ.fixed_counts` parameter, so an older cdylib shows no dead controls.
func _build_fixed_counts(dials: Array, parent: Control) -> void:
	var flag: Dictionary = bridge.param_info("civ.fixed_counts")
	if flag.is_empty():
		return
	_count_dials = dials
	var live = bridge.param_get("civ.fixed_counts")
	var on := bool(live if live != null else flag.get("default", false))
	_count_toggle = DccWidgets.toggle(parent, "Fixed settlement counts", on,
		func(v: bool):
			bridge.param_set("civ.fixed_counts", v)
			_refresh_fixed_counts(),
		"Reference Auto-populate count inputs (Capitals/Cities/Towns/Villages/Hamlets; blank = automatic). On, exactly that many of each tier are placed best-site-first, the suitability floor and packing dials are replaced by the reference's own 0.35 threshold and a spacing derived from the total, and the centrality re-tiering and imperial-seat passes are skipped so the counts hold. A world with fewer viable sites than asked fills the earlier tiers first.")
	_count_boxes.clear()
	for row in COUNT_ROWS:
		var key: String = row[0]
		var info: Dictionary = bridge.param_info(key)
		_count_boxes.append(DccWidgets.number(parent, row[1], float(info.get("min", 0.0)),
			float(info.get("max", 50.0)), 1.0, _param_or_default(key, info),
			func(v: float):
				bridge.param_set(key, int(v))
				_refresh_fixed_counts(),
			"Reference %s. Read only while Fixed settlement counts is on." % info.get("reference_control", key)))
	_count_note = DccWidgets.note(parent, "")
	_refresh_fixed_counts()

## Re-derive the counts block from the engine: the spin boxes are editable only
## while the toggle is on, and the two automatic dials grey out only when the
## counts are actually in force (on, and a total of at least 1 -- an all-zero
## request falls back to automatic placement). Updates the explanatory note to say
## which of the three states applies.
func _refresh_fixed_counts() -> void:
	var live = bridge.param_get("civ.fixed_counts")
	var on := bool(live) if live != null else false
	var total := 0
	for sb in _count_boxes:
		sb.editable = on
		total += int(sb.value)
	var active := on and total >= 1
	for d in _count_dials:
		(d["slider"] as HSlider).editable = not active
		(d["row"] as Control).modulate.a = 0.45 if active else 1.0
	if not on:
		_count_note.text = "Counts off: placement is automatic, from the two dials above."
	elif total < 1:
		_count_note.text = "Every count is 0, so placement stays automatic. (The reference refuses an all-zero request outright; here it falls back.)"
	else:
		_count_note.text = "%d settlements requested. Suitability floor and packing are overridden (threshold 0.35, spacing from the total)." % total

## The engine's live value for parameter `key`, else the default from its
## `info` record, else 0.0. Never encodes "absent" as a plausible value other than
## the documented default.
func _param_or_default(key: String, info: Dictionary) -> float:
	var live = bridge.param_get(key)
	return float(live if live != null else info.get("default", 0.0))

## The read-only readout under the dimension inputs: Grid, Extent, Cell size and
## Aspect. Creates the value labels and registers them in `_derived_labels`, which
## `_update_derived_readout` fills; every value starts as a dash.
func _build_derived_panel(parent: Control) -> void:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", DccTheme.flat(DccTheme.c("sunken")))
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 16)
	grid.add_theme_constant_override("v_separation", 4)
	var pad := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 10)
	pad.add_child(grid)
	panel.add_child(pad)
	for key in ["Grid", "Extent", "Cell size", "Aspect"]:
		grid.add_child(DccTheme.label(key, "text_faint", DccTheme.FS_TINY))
		var v := DccTheme.label("—", "accent", DccTheme.FS_SMALL)
		grid.add_child(v)
		_derived_labels[key] = v
	parent.add_child(panel)

# -- Extent / dimension logic (main.gd, ported) --------------------------------

## An extent pick (chip or dropdown): writes the live `world` parameter and
## re-derives everything that hangs off it. Pushes a preview history entry
## unless a history restore is what moved it.
func _on_extent_selected(index: int) -> void:
	if not _restoring:
		_history_push_pending = true
	bridge.param_set("world", index == 1)
	_update_extent_state()

## Choosing world extent *suggests* 2:1 and explains why; it does not lock the
## dimensions (owner, 2026-08-19: "don't lock out map sizes based on the choice
## of region or world map"). An earlier version disabled the aspect control and
## the row count in world mode. The physics that motivated it is real -- a
## seamless equirectangular sheet spans 360° of longitude against 180° of
## latitude, so anything but 2:1 stretches the graticule against the terrain --
## but that is a reason to *warn*, not to take the control away. The engine
## accepts any grid either way, so the only thing the lock bought was a
## narrower tool.
func _update_extent_state() -> void:
	var world := extent_input.selected == 1
	if world and aspect_input.selected == ASPECT_REGION_INDEX:
		## Move off the region default to the world one, once, as a starting
		## point. A deliberate choice made afterwards is left alone.
		aspect_input.selected = ASPECT_WORLD_INDEX
	aspect_input.disabled = false
	grid_h_input.editable = true
	extent_note_label.text = EXTENT_NOTE_WORLD if world else EXTENT_NOTE_REGION
	_set_extent_chips()
	_refresh_dimensions()

## Size preset picker: copy the preset's width in km into the width field
## without emitting its signal, then recompute the dependent readout (which
## also moves an auto plate count and redraws the preview). The Custom entry
## (past the preset list) changes nothing. Pushes a preview history entry.
func _on_size_preset_selected(index: int) -> void:
	if not _restoring:
		_history_push_pending = true
	if index < SIZE_PRESETS.size():
		width_input.set_value_no_signal(float(SIZE_PRESETS[index]["km"]))
	_refresh_dimensions()

## Resolution preset picker: copy the preset's grid width into the width field
## silently, then refresh the dependent rows. The Custom entry changes nothing.
func _on_resolution_selected(index: int) -> void:
	if index < RESOLUTION_PRESETS.size():
		grid_w_input.set_value_no_signal(RESOLUTION_PRESETS[index])
	_refresh_dimensions()

## Hand-typed row count: switch the aspect picker to Custom, since a typed value
## is by definition no preset, then refresh. Ignored while `_dim_syncing` -- our own
## `set_value_no_signal` writes must not flip the picker back.
func _on_grid_h_changed(_value: float) -> void:
	if _dim_syncing:
		return
	## A hand-typed row count is by definition no longer one of the presets.
	aspect_input.selected = ASPECT_CUSTOM_INDEX
	_refresh_dimensions()

## The grid height the current extent + aspect selection implies. Both
## reference ratios are asked of the engine (`bridge.reference_grid_height`,
## the reference app's own gridH) rather than recomputed from ASPECT_PRESETS,
## so the two constants live in exactly one place.
func _derived_grid_h(gw: int) -> int:
	if not bridge.sized_api:
		return gw
	## Both reference ratios are still asked of the engine, but only when the
	## matching preset is actually selected -- world extent no longer forces
	## its own answer over a chosen aspect.
	if aspect_input.selected == ASPECT_WORLD_INDEX and extent_input.selected == 1:
		return bridge.reference_grid_height(gw, true)
	if aspect_input.selected == ASPECT_REGION_INDEX:
		return bridge.reference_grid_height(gw, false)
	if aspect_input.selected == ASPECT_CUSTOM_INDEX:
		return maxi(GRID_MIN, int(grid_h_input.value))
	var ratio := float(ASPECT_PRESETS[aspect_input.selected]["ratio"])
	return maxi(GRID_MIN, int(round(gw / ratio)))

## Single re-entrant-safe sync: preset buttons follow their free entry, the
## row count follows the aspect selection, and the readout follows both. Every
## size input ends here, so it is also where the plate readout (the width rule)
## and the preview (width, resolution and grid rows are all redraw inputs)
## are told.
func _refresh_dimensions() -> void:
	if _dim_syncing:
		return
	_dim_syncing = true

	var gw := int(grid_w_input.value)
	resolution_input.selected = RESOLUTION_PRESETS.find(gw) if RESOLUTION_PRESETS.has(gw) else RESOLUTION_CUSTOM_INDEX

	var km := width_input.value
	var size_index := SIZE_CUSTOM_INDEX
	for i in SIZE_PRESETS.size():
		if is_equal_approx(float(SIZE_PRESETS[i]["km"]), km):
			size_index = i
	size_preset_input.selected = size_index

	var gh := int(grid_h_input.value)
	if aspect_input.selected != ASPECT_CUSTOM_INDEX:
		gh = _derived_grid_h(gw)
		grid_h_input.set_value_no_signal(gh)

	_dim_syncing = false
	_update_derived_readout(gw, gh, km)
	_refresh_plates()
	_schedule_preview()

## Fill the readout labels for a `gw` x `gh` grid spanning `km_w` kilometres:
## cells, extent, square cell size (the one quotient the engine derives distances
## from), and the aspect with a shape word. Also shows the size and degenerate-aspect
## warnings. Display only: it changes no parameter.
func _update_derived_readout(gw: int, gh: int, km_w: float) -> void:
	## The one quotient the whole engine derives distances from.
	var cell_km := km_w / float(gw)
	var km_h := cell_km * gh
	var ratio := float(gw) / float(gh)

	_derived_labels["Grid"].text = "%d × %d cells   (%s)" % [gw, gh, _format_count(gw * gh)]
	_derived_labels["Extent"].text = "%s km × %s km" % [_format_km(km_w), _format_km(km_h)]
	_derived_labels["Cell size"].text = "%s km per cell, square" % _format_km(cell_km)
	var shape := "landscape" if ratio > 1.005 else ("portrait" if ratio < 0.995 else "square")
	_derived_labels["Aspect"].text = ("%.3f : 1 · %s" % [ratio, shape]) if ratio >= 1.0 \
		else ("1 : %.3f · %s" % [1.0 / ratio, shape])

	var warnings: Array[String] = []
	if gw >= 4096 or gh >= 4096:
		## "CPU-only pipeline" removed 2026-09-24 (ALIGNMENT_AUDIT Part 2 B11):
		## `engine_bridge.gd::_ready` turns `use_gpu` on at boot.
		warnings.append("4K/8K grids are memory- and time-heavy — a single generate at this size can take minutes and allocates several GB.")
	if maxf(ratio, 1.0 / ratio) > DEGENERATE_ASPECT:
		warnings.append("Aspect ratios past about %d:1 are degenerate: the coarse weather grid has almost no resolution across the short axis and the plate frame (a uniform margin in cells) swallows a large fraction of the sheet. It generates without crashing, but the result is not a useful map." % int(DEGENERATE_ASPECT))
	dimension_warning_label.text = "\n".join(warnings)
	dimension_warning_label.visible = not warnings.is_empty()

## `2048 × 1311 → working grid 1024 × 656` for a picked heightmap, or `""`
## when there is nothing honest to say (no import API, unreadable file, or an
## engine too old to answer). `EngineBridge.heightmap_grid_size()`'s own doc
## names this consumer -- *"for a dialog that wants to show the working grid
## before committing"*. It went a stretch with no caller at all, and while it
## did, `Import ▸ Load heightmap…` picked a file and committed in the same
## breath -- so the first a user heard of the resample was the world coming
## back at a resolution they had not chosen. `app.gd::open_heightmap_import()`
## now folds this line into the status it was already setting on the pick.
##
## The width it resamples *to* is this form's own `grid_w` (`request()` hands
## the engine exactly that field), which is why the summary belongs here and
## not in the picker: this dialog is the only place that number exists.
##
## Returns the line rather than drawing it, because the pick happens while this
## dialog is closed -- `app.gd`'s `open_heightmap_import()` owns the file
## callback, and the surface a user is looking at then is the status line.
func heightmap_grid_summary(path: String) -> String:
	if bridge == null or not bridge.import_api:
		return ""
	var img := Image.load_from_file(path)
	if img == null or img.is_empty():
		return ""
	var src := img.get_size()
	var grid := bridge.heightmap_grid_size(int(grid_w_input.value), src)
	if grid == Vector2i.ZERO:
		return ""
	return "%d × %d → working grid %d × %d" % [src.x, src.y, grid.x, grid.y]

## Cell count as "x.xx M cells" from one million up, otherwise "x.x k cells".
## Display text only.
func _format_count(n: int) -> String:
	if n >= 1000000:
		return "%.2f M cells" % (n / 1000000.0)
	return "%.1f k cells" % (n / 1000.0)

## Kilometres to a width-appropriate string: whole numbers from 100, one decimal
## from 1, otherwise three decimals, so a tiny cell size never reads as 0.
func _format_km(v: float) -> String:
	if v >= 100.0:
		return "%.0f" % v
	if v >= 1.0:
		return "%.1f" % v
	return "%.3f" % v

# -- World structure --------------------------------------------------------

## An archetype pick: Classic clears it; a named one writes its five World
## Structure dials into the live parameters (`apply_archetype`). Either way the
## plate readout switches source (Q8: an archetype's count is the engine's own,
## shown read-only) and the preview redraws, since `world_structure.*` and the
## archetype are both redraw inputs.
func _on_archetype_selected(index: int) -> void:
	if index <= 0:
		_archetype = ""
	else:
		var name := String(_archetype_names[index - 1])
		_archetype = name
		bridge.apply_archetype(name)
	_refresh_plates()
	_schedule_preview()

# -- Create ---------------------------------------------------------------------

## Create generates. Unconditionally, which is what `File ▸ New world` does in
## the reference and what this dialog has always actually done.
##
## A `set_auto_generate()` setter and its `_auto_generate` flag used to sit
## here for "a future caller wanting to only stage values without generating
## yet". Nothing ever wrote the flag, so the branch was constant-true and the
## setter unreachable -- a stage-without-generate mode needs a caller and a
## control, not a private boolean, and would be added with them.
##
## **On a phone, a grid above Ruling AH's ceiling asks first** (Ruling AS,
## 2026-09-24): the phone keeps 4K and 8K rather than hiding them, but a Create
## above 2048 × 1311 cells stops at the memory question and generates only on
## its OK. Desktop and tablet never reach that branch -- `_phone` is false
## there -- so they generate exactly as before. Cancel here brings this form
## back (`_represent_form`), which is the one thing only Create's route wants.
##
## **NWP-3 (Q1, Ruling CE):** first writes the plate count the dialog shows into
## `tect.plates` (`_apply_plates_to_engine()`), so the world gets the plates the
## preview drew. Only Create does this; the other `generate_checked()` routes
## regenerate with whatever `tect.plates` holds.
func _on_create() -> void:
	_apply_plates_to_engine()
	generate_checked(_represent_form, true)

## **Every generate at this dialog's size goes through here** -- Create, WORLD ▸
## Generate / the live regenerate (`world_workspace.gd::_regenerate_now`) and the
## tool-options row's Generate (`app.gd::_run_pipeline`). Before 2026-09-24 only
## Create asked; the other two regenerated a phone at 4K or 8K without a word.
## `on_cancel` runs when the question is declined; nothing is generated then.
##
## `new_world` is true only for Create (File > New world): the result is a different world, so
## `EngineBridge.generate` flags it and the shell drops the old project path. Every other route
## (WORLD > Generate, the tool-options row) regenerates the open project and keeps its path.
func generate_checked(on_cancel: Callable = Callable(), new_world: bool = false) -> void:
	var r := request()
	confirm_phone_memory_then(int(r["grid_w"]), int(r["grid_h"]),
		func(): bridge.generate(r, new_world), on_cancel)

## The one gate, for any pipeline run whose grid is `gw` x `gh`: `proceed` runs
## at once on desktop, on tablet, and on a phone at or under the ceiling;
## above it on a phone, only on the question's OK. The heightmap import uses it
## directly (`app.gd::_import_heightmap_at`), because its grid is the picture's
## resample rather than this form's rows.
func confirm_phone_memory_then(gw: int, gh: int, proceed: Callable, on_cancel: Callable = Callable()) -> void:
	if _phone and exceeds_phone_ceiling(gw, gh):
		_confirm_phone_memory(gw, gh, proceed, on_cancel)
		return
	proceed.call()

## Ruling AH (2026-09-23) keeps **2048 × 1311** as the Android ceiling, and
## Ruling AS (2026-09-24) turns it into a warning rather than a limit. Counted
## in CELLS rather than as a width, because the generation peak is linear in
## cell count and in nothing else (`MEMORY_OPTIMIZATION_SCOPE.md`, "The
## generation peak, measured field by field (2026-08-25, second pass)", §3:
## 4x the cells is 3.998x the peak). So a 2K grid at the region aspect -- the
## ceiling itself -- asks nothing, and a 2K grid at a taller custom aspect,
## which costs more than the ceiling was measured at, does.
const PHONE_CEILING_CELLS := 2048 * 1311
## The measured slope, not a guess: 190.0 bytes of peak per cell, the
## pipeline's allocator peak at 2048 × 1311 on the desktop after the
## 2026-09-27 pass (`MEMORY_OPTIMIZATION_SCOPE.md`, "The 2026-09-27 pass");
## 4096 × 2622 measured 189.2 in the same pass, so the larger figure is kept.
## It gives 4096 × 2621 at 1.90 GiB and 8192 × 5243 at 7.60 GiB. **Projected
## for the phone, not measured there**: the 2026-08-25 section found desktop
## and handset agreeing to 0.01-0.15 %, and no handset pass has re-run it
## since. The 8192 figure is the slope extrapolated; nothing that size has
## been run on a phone. Was 241.3 until 2026-09-27 -- the pre-R3 audit's
## figure, stale since R1-R3 landed the same day (202.7 B/cell measured then).
const PEAK_BYTES_PER_CELL := 190.0
## What the same section measured beneath the pipeline, and the headroom it
## had: `~420 MB` of Godot process on a handset reporting 2.38 GB available.
const PHONE_APP_BASE_MB := 420
const PHONE_MEASURED_AVAILABLE_GB := 2.38  ## Available memory (GB) on the measured handset, from the memory-audit section cited above; used in the warning text, not read from the device.

## The open memory question, or `null` when none is up. Kept so a probe can
## find it; nothing else reads it.
var _memory_confirm: ConfirmationDialog

## True when `gw` x `gh` is above `PHONE_CEILING_CELLS`. Cell count, not width,
## because the generation peak is linear in cells and nothing else.
func exceeds_phone_ceiling(gw: int, gh: int) -> bool:
	return gw * gh > PHONE_CEILING_CELLS

## The GiB a generate of `gw` × `gh` peaks at, from the measured slope.
static func estimated_peak_gib(gw: int, gh: int) -> float:
	return float(gw) * float(gh) * PEAK_BYTES_PER_CELL / 1073741824.0

## The body of the Large-world memory question for a `gw` x `gh` grid: the peak
## estimate from `estimated_peak_gib`, the app's own footprint and the measured
## handset headroom. The figures are projected for the phone, not measured there
## (see `PEAK_BYTES_PER_CELL`); the text must keep saying so through its wording
## "about" and "can".
func phone_memory_warning_text(gw: int, gh: int) -> String:
	return ("A %d × %d grid is above 2048 × 1311, the largest recommended on a phone.\n\n"
		+ "Generating it needs about %.2f GiB of memory at its peak (measured at %.1f bytes per cell), "
		+ "on top of about %d MB for the app itself. 2048 × 1311 was the largest grid measured "
		+ "to fit on a handset with %.2f GB available; above it the system may close the app "
		+ "mid-generate and the world will be lost.\n\nA generate this size can also take minutes.") \
		% [gw, gh, estimated_peak_gib(gw, gh), PEAK_BYTES_PER_CELL,
			PHONE_APP_BASE_MB, PHONE_MEASURED_AVAILABLE_GB]

## `DccWidgets.confirm()`, the shell's protocol-complete question, hosted on
## the shell rather than on this dialog: it is itself an exclusive window, and
## on Create's route this dialog has already hidden (`_build_phone_actions()`'s
## CREATE WORLD hides before it calls `_on_create()`); on the other routes it
## was never open.
func _confirm_phone_memory(gw: int, gh: int, proceed: Callable, on_cancel: Callable) -> void:
	var host := get_parent()
	if is_instance_valid(_memory_confirm):
		_memory_confirm.queue_free()
	_memory_confirm = DccWidgets.confirm(host, "Large world",
		phone_memory_warning_text(gw, gh), "Generate anyway",
		proceed,
		func():
			## Hidden here, not left to Godot: `AcceptDialog` defers its own
			## `hide()` past the `canceled` emit, so the question is still the
			## host's exclusive child at this point and re-presenting the form
			## would be refused ("already has another exclusive child").
			if is_instance_valid(_memory_confirm):
				_memory_confirm.hide()
			if on_cancel.is_valid():
				on_cancel.call())

## Create's Cancel: **the form comes back** with everything as it was -- the
## dialog is a persistent singleton, so the values are still in its controls --
## rather than dropping the user on the map with nothing generated and no form
## to change the size in.
func _represent_form() -> void:
	if not DccWidgets.phone_present(self, get_parent()):
		popup_centered()

## The keys `EngineBridge.generate()` reads. Sea level and the four
## experimental flags are read live off `bridge` rather than cached locally --
## see this file's own header comment on why that keeps this dialog and the
## Generation Pipeline's stage rows from ever disagreeing.
## `#reseedBtn`'s own behaviour, verified live against the reference (Playwright,
## 2026-08-19): `state.tect.seed=(Math.random()*99999)|0; ...;
## withBusy('generating…',generate)` -- a reroll and nothing else. No dialog
## opens; the caller regenerates immediately with the new seed.
##
## Clears `_seed_dirty` in the same statement that writes `.value` -- see that
## field's own comment and `request()`'s own -- so a roll always reads back
## clean regardless of what a stale focused `LineEdit` might still show.
##
## **Honours the seed lock** (NWP-3, Q4): while `seed_locked` it returns
## without rolling. This is the single gate for every roll -- the dialog's own
## dice and `app.gd::_new_seed()` (the WORLD dock's and tool bar's *New seed*)
## both land here, so the lock covers them all for the session without a hook
## in `app.gd`. The `.value` write fires `value_changed`, which is what pushes a
## history entry and schedules the redraw (`_on_seed_changed`).
func randomise_seed() -> void:
	if seed_locked:
		return
	seed_input.value = randi() % 100000
	_seed_dirty = false

## `PARITY_AUDIT.md` §23 F14: `get_villages_enabled`/`get_metropolis_enabled`/
## `get_recovery_phase` are real getters -- `self.civ_options.villages`/
## `.metropolis`/`.recovery_phase` on the Rust side, always well-defined,
## not gated on a world existing -- that had no reader anywhere before this
## pass. Connected to `about_to_popup` in `setup()`; see that connection's
## own comment for why a re-sync on every open is needed at all.
##
## `set_pressed_no_signal`/`select()` rather than assigning `.button_pressed`/
## `.selected` directly: both would re-fire the control's own `toggled`/
## `item_selected` signal, which just sets `_villages`/`_metropolis`/
## `_recovery_phase` right back to the same value -- harmless here, but the
## no-signal setters are the honest way to say "this is a read, not a user
## edit" and cost nothing extra.
##
## Goes through `bridge.get_*` (the wrappers `engine_bridge.gd` already
## carries, each already guarded by its own `_has()` check) rather than
## `bridge.world_gen.*` directly -- `engine_bridge.gd` is finished and off
## limits this pass, but the read side it needed was already there.
func _sync_from_engine() -> void:
	if bridge == null:
		return
	if villages_check != null:
		_villages = bridge.get_villages_enabled()
		villages_check.set_pressed_no_signal(_villages)
	if metropolis_check != null:
		_metropolis = bridge.get_metropolis_enabled()
		metropolis_check.set_pressed_no_signal(_metropolis)
	## The fourth toggle F14 itself missed: the return of `DccWidgets.toggle()`
	## was discarded here, so this box had no handle to write back through and
	## no clause in this function. It read unchecked while the engine held it
	## ON, and the next Create posted that stale `false` straight back into
	## the engine, silently turning the residual off again.
	if biome_k_check != null:
		_biome_k = bridge.get_biome_k_enabled()
		biome_k_check.set_pressed_no_signal(_biome_k)
	if recovery_input != null and recovery_input.item_count > 0:
		_recovery_phase = clampi(bridge.get_recovery_phase(), 0, recovery_input.item_count - 1)
		recovery_input.select(_recovery_phase)
	## **The fifth reader F14 missed, and the one this pass made visible.**
	## `extent_input` is seeded from `param_get("world")` in `_build()` and was
	## never read again, so opening this dialog over a project whose extent had
	## since changed -- `project_open` and `load_save` both write that
	## parameter -- displayed the extent of the *previous* world. The other
	## four re-reads below it were added by F14; this one was not, and it is
	## the one that lies loudest, because `request()` carries no extent at all:
	## Create takes whatever the live parameter holds, so a stale REGION on
	## screen produces a whole-world map with no warning.
	##
	## Routed through `_update_extent_state()` rather than assigning
	## `selected`, because the extent note, the one-time aspect suggestion, the
	## derived readout and (on a phone) the two chips all hang off it.
	## `param_get` answers `null` before the table has been probed, and `null`
	## is not `false` -- an unprobed table leaves the control as it was rather
	## than being told the world is a region.
	var world_now = bridge.param_get("world")
	if extent_input != null and world_now != null \
			and (extent_input.selected == 1) != bool(world_now):
		extent_input.selected = 1 if bool(world_now) else 0
		_update_extent_state()

## **`seed_input.apply()` only when `_seed_dirty` says a real edit is
## actually pending** -- not unconditionally. The 2026-09-12 attempt called
## `apply()` on every `request()`, which is the right move for a seed typed
## but never committed (Android BACK dismisses the on-screen keyboard
## without Godot ever seeing a focus change or an Enter, so `.value` stays
## stale while the field goes on showing what was typed -- reported live as
## "Create -> world is X · 51127; the typed value was then used by the next
## Generate World") and the wrong one for a ROLLED seed read while this
## dialog is hidden: `_run_pipeline()` and the heightmap import both call
## this with the dialog never popped, and off-screen a `SpinBox`'s
## `LineEdit` does not resync its displayed text to a `.value` written by
## code (measured, `_vfy_seedroll_probe.gd` S0) -- so an unconditional
## `apply()` there re-parsed the STALE display text straight over the
## freshly rolled `.value`, silently discarding the roll. Gating on
## `_seed_dirty` -- true only while a real keystroke has landed in the field
## since the last programmatic write or the last commit -- fixes the first
## case without breaking the second: `randomise_seed()` clears the flag in
## the same statement that writes `.value`, so a roll is never seen as dirty
## regardless of what the (possibly stale, possibly hidden) `LineEdit` shows.
func request() -> Dictionary:
	if _seed_dirty:
		seed_input.apply()
		_seed_dirty = false
	return {
		"seed": int(seed_input.value),
		"width_km": width_input.value,
		"grid_w": int(grid_w_input.value),
		"grid_h": int(grid_h_input.value),
		"archetype": _archetype,
		"villages": _villages,
		"metropolis": _metropolis,
		"biome_k": _biome_k,
		"recovery_phase": _recovery_phase,
		"sea_level": bridge.param_get("sea_level"),
		"dynamic_lithology": bridge.param_get("tect.dynamic_lithology"),
		"volcanic_provinces": bridge.param_get("volc.provinces"),
		"wind_deflection": bridge.param_get("climate.terrain_wind_deflection"),
		"ocean_currents": bridge.param_get("climate.currents"),
	}
