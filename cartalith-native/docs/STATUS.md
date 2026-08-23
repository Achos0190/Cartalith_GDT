# Status quicklist

A living checklist, not a narrative — read this first, before `CHANGELOG.md`,
to know what's done vs. open without re-reading the whole history each
session. Update it in the same commit as whatever changes its answer.
`CHANGELOG.md` stays the detailed record of *how*; this is only *what/done?*.

Last updated: 2026-08-23 (post **Deep-zoom LOD: the tiles were the reference's
*Relief* view, not its *Biome* view**. Owner: *"the zoom-lod bug where a zoom
action exposes the underlying heightmap is still there."* Reproduced on a real
screen before touching anything — the same camera with the LOD layer shown
(bare green/gold/grey elevation ramp) and hidden (the full plate: biome colour,
rivers, hillshade, paper frame). Root cause: the reference chooses its LOD tile
coloriser by **view mode** (`_lodBuildTileRGBA`, reference 11148 —
`biome ? renderBiomeTileRGBA : renderHeightTileRGBA`, with `'biome'` the
default), this port only ever ported the *Relief* half, and `lod_bridge.rs`
wired the compositor straight to it. Second divergence: deep zoom engaged on
px-per-cell alone, so it was live at the **fit** view of any world narrower
than the map rect; the reference also requires camera zoom `> 2.2`
(`LOD_AUTO_SCALE`, reference 13952). Fixed by making a tile carry only what the
base raster cannot have — the relief-detail **shade ratio**, `shade_tile(with
detail) / shade_tile(no detail)` — and multiplying it into `map_view`'s own
texture through a new `shell/lod_tile.gdshader`, so the two paths agree by
construction; plus `LOD_AUTO_ZOOM = 2.2`. A tile-boundary misalignment
(`amplify_region`'s sample convention vs. the raster's texel convention: 1.6%
stretch plus a half-cell offset) was fixed in the same pass by
`tile_sample_region` — that is **`GUI_GAP_REGISTER.md` CV-VS-01**, measured at
8x the local baseline on a `TILE_CELLS` row boundary before, gone after.
`renderBiomeTileRGBA` itself stays unported and is now the named next milestone
for this subsystem. Verified: 27 test suites pass (8 new), goldens untouched,
headless boot clean, and non-headless before/after captures at z1.0/2.31/3.51/
5.35/8.0 in both WORLD and CIVIL.
— previously, post **Three stranded items: timeline `tid`, Asset
Library Collections/drag-and-drop/slicer interaction, Fira fonts**. Three
independent, previously-disclosed gaps closed together as mechanical wiring,
not open-ended work — full account in `CHANGELOG.md`. **(1)**
`get_settlements()` (`lib.rs`) now carries `tid`; `civilization_workspace.gd`'s
Timeline **Exist only** filter is wired for real, filtering the settlement
array `map_overlay.gd` draws down to the active year's `civ_year_diff().present`
set (`GUI_GAP_REGISTER.md` CV-03, partly closed — Ghost removed/Highlight new
stay open, needing per-pin draw state only `map_overlay.gd` can add, and the
OLD snapshot's settlement data for "removed", which nothing exposes yet).
**(2)** Asset Library gained a real Collections rail (new `#[func]
as_collections`, `_refresh_collections_rail`/`_select_collection`), real
in-app drag-and-drop (drag selected tiles onto a Collections row →
`as_batch_collect`, through real `_get_drag_data`/`_can_drop_data`/
`_drop_data`), and the sprite-sheet slicer's canvas gained wheel-zoom
(reversible, centred on the cursor), middle-drag pan, click-to-select-a-cell,
and a draggable Margin handle (`GUI_GAP_REGISTER.md` AS-12/AS-17, both partly
closed — "Unassigned imports", OS-file-drop-onto-a-slot, and per-interior-
line dragging all stay open for reasons recorded there, not silent gaps).
**(3)** Fira Sans and Fira Code sourced for real (SIL OFL 1.1, license text
fetched from each upstream repo, `fonts/Fira{Sans,Code}-OFL.txt`); Fira Sans
wired as `dark_theme.tres`'s `default_font`, closing the "Deliberately
deferred" typography note `CHANGELOG.md` carried since the original
design-system match. Fira Code is sourced but deliberately left unwired — IBM
Plex Mono already fills that role, shipped and tested, and swapping it in for
zero visual-parity gain would be an undisclosed regression, not a fix.
Verified: `cargo build -p cartalith-godot` clean, `cargo test -p
cartalith-godot --lib` (247 passed), `--headless --path . --quit` clean
(after one `--editor` pass to import the six new font files — the plain
console runtime doesn't import new resources on its own). Drag-and-drop and
the slicer's canvas interaction were functionally verified via direct event
injection in two uncommitted harness scripts (deleted after), **not** a live
mouse-driven session — said plainly, not claimed as full interactive
verification.
— previously, post **Wind and Ocean currents: the animation the
reference has and this port did not**. Owner: *"the ocean current layer isnt
animated as the HTML version is. (same for wind)"* — and correct. Both views
were ported and both were *right*; what was never ported is that the reference
stacks a **second, independent overlay** on exactly these two: `#windFxCanvas`,
a particle-streak animation (`_windFx*`, HTML lines 2113-2209) the normal
render pipeline never touches. Measured before the fix on a real screen:
**0.0000** mean frame-to-frame pixel difference on both. Ported constant for
constant — 260/200 particles, `0.315` cells/tick advection, `50+rand*50` /
`60+rand*60` lifetimes, respawn on leaving the map, ageing out or beaching,
the ocean spawner's 30-try water rejection. `cartalith-climate::
current_ocean_field` is `currentOceanField()` split *out of* `ocean_sst_anomaly`
rather than written beside it (the two must agree about which way a current
runs; the reference itself shipped two disagreeing answers until its own
v1.78) — golden parity unchanged. **The one deliberate technique change is the
trail**: Godot clears its canvas every frame, so the reference's
`destination-out` fade would need a never-cleared `SubViewport` doing GPU work
behind a closed layer — the exact hazard the Devices crash below is a live bug
from. Each particle carries its last 12 positions and redraws them under the
same `0.86 ** k` decay instead: same streak, no retained target, literally zero
cost when off. **Verified non-headlessly, because headless cannot see motion**:
Wind 0.134/0.133/0.135/0.137, Ocean 0.052/0.050/0.048/0.048, layer off
0.000/0.000/0.000, re-armed after off 0.052/0.045/0.047, after a regenerate
under a live view 0.063/0.063/0.067 — all at 57-60 fps; 260/260 wind and
200/200 ocean particles on valid cells; the packed mask round-trips intact
(35 559 water / 161 049 land, matching the coastline). One thing is
**honestly a workaround**: the flow field reaches GDScript through
`build_debug_texture` as a packed `flowfx:` raster rather than a `#[func]`,
because `lib.rs` — this crate's sole `godot` boundary — was owner-reserved for
concurrent work. Worth replacing; only the GDScript decode changes.
— previously, post **The Devices menu crash: the backend mask was
on the wrong call**. Owner: *"There seems to be a crash in the program when you
get higher than 2k and start changing settings for resources such as GPU/CPU."*
The size is a red herring — the crash is **opening `Preferences ▸ Performance ▸
Devices` at all**, at every grid size tried (512² through 4096²), and it is the
same GL-context corruption `6a97911` chased on 2026-08-20. That commit fixed the
*launch* by deferring enumeration to the submenu's first open, which moved the
crash rather than removing it. **`multi.rs` was applying its non-GL backend mask
to `enumerate_adapters`, which is far too late**: `wgpu::Instance::new` stands up
a `hal::Instance` for every backend in its own *descriptor's* mask, and
`InstanceDescriptor::new_without_display_handle()` leaves that at
`Backends::all()` — so the OpenGL context was created inside Godot's
GL-Compatibility process the moment the instance was, before a single adapter
had been asked for. (`6a97911`'s own message records that restricting the
enumeration mask "was tried first, still crashed"; this is why.) Fixed with one
function: `multi::compute_instance()` is now the only place in the crate that
constructs a `wgpu::Instance` and it sets `backends: COMPUTE_BACKENDS` on the
descriptor; all five construction sites go through it. **A second real defect
found in the same interaction and closed**: `generate()` runs on a `Thread` and
gdext holds the whole `WorldGen` mutably borrowed for it, so every `#[func]`
reached from the main thread meanwhile fails `Gd<T>::bind()` — 360 panics
measured during one 4096×2624 generation, each returning a default, and the
Devices submenu latched `_gpu_enumerated` over the empty answer so it read "No
GPU detected" for the rest of the session. This build has no
`experimental-threads`, so the borrow state behind that check is a non-atomic
`Cell` and two threads racing it is UB, not merely an error. `engine_bridge.gd`
now serves the six multi-GPU readers and `param_get` from a cache for exactly
that window (exact, not approximate: the settings are process-global and this
file is their only writer) and refuses the setters, and `menus.gd` disables the
GPU row plus all four Performance submenu rows with the reason rather than
letting a click silently no-op. **Verified non-headlessly through the real menu
path** (`MenuButton.get_popup().popup()` then the `GpuDevices` submenu's own
`popup()`), because headless cannot see this bug class: opening Devices with
nothing running is clean and lists three devices; every GPU setting changed and
a 4096×2624 split-tiles generation completed; the same menu driven **479 times
during** that generation with **zero** `bind()` panics, zero Godot errors and
zero crashes, every row correctly disabled; afterwards the rows re-enable and
selection/mode/budget/estimate are intact. Also 8192×5248 split-tiles clean
(~140 s), `cargo test -p cartalith-gpu` 54 + 8 green, `_shot.tscn --generate`
re-shot, `--headless --quit` clean.
— previously, post **Data manager window: rebuilt against the
design canvas**. The second window from the same visual sweep with the same
history, passed the same too-lenient way — the sweep checked that the routes
worked and that the disclosures were honest, and never laid the layout against
`design/Cartalith DCC Shell.dc.html`'s `Data manager window 1920`.
`GUI_GAP_REGISTER.md` §14.2's row is corrected and **§14.7** carries the
20-item delta list. Rebuilt from that screen: a full-bleed window under the
menu bar with its own 34 px bar and 26 px status line (was a floating 920×600
`AcceptDialog` with an OS title bar and a stock OK button); a 252 px routes
rail with a `ROUTES` band, plain tracked group headers, one-line rows with the
canvas's quiet badges and an `accent_wash`-plus-`▸` selected row (was `§`-sigil
sections of autowrapping flat buttons); and **§9's route pane, built** — the
canvas's `1fr 1fr` grid with all seven column blocks (TILES / PROJECTION /
LAYERS INCLUDED / OUTPUT / ESTIMATE / MARKDOWN VAULT / RECENT RUNS), a
`120px label · control` row grammar of segments/wells/`☑` rows, and a
`Save as preset · Dry run · Export N tiles` footer, where the window used to
show one grey paragraph. **DM-13 closed, DM-02 half closed, DM-12 partly,
RD-09 closed.** `region_export_tiles` had been bound and golden-tested with
**no caller anywhere in the shell**; this pane is the caller, exporting the
live Region-select marquee as a zipped `cols × rows` tile grid — verified by
writing a real archive and reopening it (33 entries, `tiles/index.json`
present) — and `right_dock.gd`'s Region select ▸ *Send to Data ▸ Export*, dead
since it was written, now opens straight onto it. The **pyramid** the canvas
draws (XYZ/TMS/WMTS, CRS, world file, MBTiles, leaflet preview, ocean-tile
skipping, political/label/river layers) is not what the engine does and is
drawn in place and disabled with that reason; the MARKDOWN VAULT block is the
canvas's shape but quiet and titled `· NOT LINKED` (DM-14). The canvas's
`~ 214 MB` / `~ 3 min 40 s` estimates are a model this port lacks, so **Dry
run** measures both for real. The **CONVERSION group stays deleted** — the
canvas predates `17ccc18` and is not followed there. The chip/segment/well/
text-button/band vocabulary moved from `asset_library_window.gd` into
`dcc_widgets.gd`, as that file's own note asked. Three Godot traps found, **all
already shipped**: `AcceptDialog` enables `wrap_controls`, so the window grows
to its contents' minimum and never shrinks — this one popped at 997 px and was
grown to 2032 px inside a 1031 px viewport, dropping its own footer and status
line off the bottom edge (**a live regression in the Asset library too**, fixed
in the same commit); the autowrap-`Label`-with-no-min-width trap again, which
was what fed it; and `theme/dark_theme.tres` styling every `ScrollContainer`
with `SB_FieldDisabled`, an input well with a 10 px content margin and a 4 px
radius, which insets every scrolled region in the shell against its own header
band (overridden here; **the theme still carries it shell-wide**). Verified by
looking: non-headless boot on the **native GL driver** — no `opengl3_angle`
fallback needed, `6a97911`'s launcher fix holds — a real 2048×1311 world, a
real 1024×590-cell marquee, five screenshot/compare iterations, node-rect
probes that located two of the three traps, a real export and a dry run
(5.2 MB / 0.58 s), Escape close through `Input.parse_input_event`, and
`--headless --quit-after 120` clean.
— previously, post **Asset library window: rebuilt against the
design canvas**. The owner: *"The asset manager menu looks nothing like the
DCC work from Claude design."* True — and the visual sweep below had scored
this surface **PASS**, a wrong verdict reached by checking that the controls
worked rather than that the layout matched. `GUI_GAP_REGISTER.md` §14.2's
row is corrected and §14.5 carries the 19-item delta list. The window was
built from `DCC_SHELL_SPEC.md` §8's prose before its bindings existed and
never laid against `design/Cartalith DCC Shell.dc.html`'s `Asset library
window 1920`; it is now, geometry and all — borderless and full-bleed under
the app menu bar, a chip/segment/well vocabulary replacing stock Godot
slabs, rail 266 / inspector 330 / bands 28 / tile art 76 / slicer
760·274·296, every colour a `DccTheme` token and no hex in the file. The
slot grid is a real contact sheet (captions *inside* the tile, ×N badges, ☑
marks, visible checkerboard empties, 6 equal columns), the inspector is the
canvas's preview → file line → Scale/Fit/Reset/Replace/+Variant → swatches
→ anchor → tags → VARIANTS → PACK METADATA stack built once and refreshed
in place, and the slicer is the canvas's two-column 760 px card instead of a
stack that clipped its own labels. **Every live binding stayed on the
control it was already on**; the four recorded engine realities (AS-16's
eight families, AS-15's family-level anchor, AS-14's weighted variants,
read-only per-item transform) are reshaped around, not regressed, and keep
their honest tooltips. Replace… / ＋ Variant are newly real off
`as_import_item` + `as_remove_item`. Five Godot traps found and recorded in
`CHANGELOG.md`: an autowrap `Label` with no min width reports a giant min
*height* (the slicer first rendered 1700 px tall), a `flat` Button draws no
stylebox at all, `disabled` beats `normal` for stylebox *and* font colour, a
`StyleBoxFlat` shadow shows through a transparent fill, and
`ScrollContainer` folds its scrollbar into the minimum it hands upward.
Verified by looking: non-headless `opengl3_angle` boot, real 512×512 world,
`reference_pack.zip` loaded, 12 real items imported (one slot with three
variants), four screenshot/compare iterations, pixel probes confirming rail
= 266 px and the selected row's exact `accent_wash` blend, the slicer smoke
path re-run against a real 6×4 sheet (`24 cells detected · 24 non-empty`,
overlay on the boundaries), both close paths including Escape driven through
`Input.parse_input_event`, and `--headless --quit-after 120` clean.
Screenshots via an uncommitted `_al_sweep.gd`/`.tscn` harness — not
committed; screenshots are not source.
— previously, post **Visual sweep — every major surface driven
and screenshotted** — the first pass that actually looked, rather than
verifying structurally/headlessly and disclosing "nothing graphical
verified" like every prior milestone. Booted the real shell non-headlessly
(GLES3/Compatibility crashes on this AMD GPU the instant the welcome
dialog's `popup_centered()` first runs — reproduced against the project's
own pre-existing `_shot.tscn`, so pre-existing, not introduced here;
`--rendering-driver opengl3_angle` avoids it with an identical visual
result and was used throughout), generated a 512×512 world, and drove every
major surface: welcome prompt, shell default (dark/light), Generate World,
Generate Sculpt + stamp stack, CIVIL dock + Timeline + a selected
settlement + territory overlay, CARTO dock + Layers popover + a debug view
rendered over the map, Asset library + sprite-sheet slicer against a real
sheet, Data manager, Travel library, a real committed route through the
Journey Planner takeover, and the map at three zoom levels including
deep-zoom LOD — each compared against `design/Cartalith DCC Shell.dc.html`
and `design/Journey Planner DCC.dc.html`. **Three real defects found and
fixed**, all re-verified live through the exact code path a real user
triggers: the sprite-sheet slicer stayed open and on top of the entire app
after closing the Asset Library window (`asset_library_window.gd`'s Close
button hid only the parent dialog, never the slicer's own independent
`Window`); the right dock's Stamp Stack panel stayed stuck outside the
WORLD domain (`right_dock.gd` had no reset for `CTX_SCULPT` on a domain
switch, despite its own doc comment's stated invariant); and the Data
manager's header subtitle still advertised the Conversion group that was
deliberately deleted 2026-08-20, a leftover the deletion pass missed. **One
defect catalogued, not fixed** — a thin horizontal seam across the map,
CIVIL-domain-only and reproducible, confirmed *not* caused by any
difference in the map overlay's own drawable data (byte-identical before
and after the domain switch) and instead correlated with the letterboxed
map rect changing size when CIVIL's taller dock resizes the viewport; root
cause not pinned down within this pass's budget. **One UX question raised,
not resolved** — arming the Journey Planner from outside CIVIL produces no
visible change beyond a status-bar string, since the takeover is
deliberately gated on the active domain; unclear whether that gate itself
is a considered decision or an oversight. Full verdict table and both
catalogued items: `GUI_GAP_REGISTER.md` §14. Verified: non-headless boot
with real GPU-composited frame capture, `--headless --path godot-project
--quit` clean after all four edits, and the sweep itself re-running
end-to-end after each fix as the live behaviour check. 23 screenshots via a
temporary, uncommitted harness (`_visual_sweep.gd`/`.tscn`, same convention
as `_shot.gd`) — not committed; screenshots are not source.
— previously, post **Travel Library → party form: the last
connecting piece** — `TRAVEL_LIBRARY_SPEC.md` §6, `GUI_GAP_REGISTER.md`
**JP-02 and IN-06 closed**. §1's own promise ("everything defined here
becomes a selectable option in the planner's party form") is now true for
animals: `journey_planner_view.gd`'s Carriage section carries four new
**per-species animal-definition pickers** plus a library-backed **Mount**
picker, custom rows tagged `· custom` off `2b`'s own treatment and ⚠-marked
by §4 validation, and the choice reaches a real plan through `jp_compute`'s
one new request key `animal_entries` →
`travel_bridge::TravelLibrary::animal_overrides_selected` → `jp_plan_ex`'s
existing resolver. **Party set-ups** (JP-02) are live in the tool-options bar
— a dropdown over `tl_list("preset")` (stock *and* captured) plus
`capture party…` writing the form back via `tl_capture_preset_from_plan`;
deliberately not the reference's JS-only `JP_PRESETS`, which is the smaller
thing. Selecting a **stock** entry means "no override" and reproduces the
baseline *exactly* (`31.6792` days both ways on the drive below); an empty
selection reproduces `animal_overrides()`, so
`regression_stock_only_travel_library_matches_pre_dispatch_jp_plan` still
holds byte for byte. Real headless numbers, 96×96 world / 1082.32 km route:
a 12-mule Baggage Train computes **31.6792 d @ 42.1475 km/day** on the stock
Mule, **31.1925 d @ 42.9617** on a custom `Kharen dray-mule` (260 kg cap),
and **48.4610 d @ 27.4275** on a from-blank `Kharen dray-ox` whose
`substitutes for = mule`; a 4-rider Mounted party goes **32.8385 d** on the
stock Horse vs **18.5708 d @ 69.5093 km/day** on a 9 km/h custom courser.
**`JpParty` was re-examined and deliberately NOT widened**: the blocker is a
spec gap, not mechanics — `jp_capacity_ex` reads per-species seasonal
physiology (`jp_seasonal_animal`, 16 rows) and desert food/water multipliers
(`jp_desert_animal_mod`) that §3.1 has **no fields for**, so a new species
would silently take neutral `1.0` on both, exactly §5's "plans silently
wrong"; three golden-tested signatures also return `&'static str`
(`resolve_mount`/`jp_resolve_mount`/`jp_best_animal_for_context`) and
`jp_capacity_ex`'s summation order is explicitly pinned to `JP_ANIMAL_KEYS`.
The **substitutes-for path** shipped instead, and the party form names every
still-unofferable custom animal with the one edit that fixes it rather than
hiding it. Vessels/vehicles stay data-only: the Vessel picker lists custom
definitions but **disables** them with `— no engine hook` on the item, where
a user actually meets the limit. Verified: full `cargo test --workspace`
(zero failures), clean `cargo build -p cartalith-godot`, clean headless boot,
and a scripted headless drive of all of the above plus the party form's own
library helpers driven directly. Not verified: anything graphical.
— previously, post **Multi-GPU: enumeration, device selection,
split tiles, VRAM budget** — the owner's 2026-08-20 answer to
`GUI_GAP_REGISTER.md`'s own open decision PR-02, *build it*. Closes
PR-01/PR-02/PR-04/PR-05 and omission O3. `cartalith-gpu` had requested
exactly one `PowerPreference::HighPerformance` adapter and had no
enumeration, no choice, no accounting, no partitioning; new
`cartalith-gpu/src/multi.rs` is all four, and `Preferences ▸` carries the
four §2.5 Performance rows for real. This machine enumerates **6 adapter
rows for 3 physical GPUs** (RX 7800 XT discrete, integrated Radeon, and the
Windows software rasterizer), and the discrete card's OpenGL row reports
`vendor = device = 0` — so the physical-device fold keys on PCI identity
with an unambiguous-name fallback, because keying on name alone would have
merged two identical cards, which is exactly the rig this feature exists
for. `split tiles` is real but covers **one** stage (`gpu_warp`, the only
kernel reading nothing outside its own cell), and the measured answer is
**1.22-1.54× at 4096² and 0.73-0.81× at 2048² and below**, so the shipped
default is `single device` rather than §2.5's `split tiles`, with those
numbers in the row's tooltip rather than an asserted benefit; band sizes
come from measured per-device throughput (integrated = 0.17 × discrete),
not a guess. Three §2.5 choices are **refused at the API rather than
silently accepted**: `alternate frames`, `reduce working res`, and — as
values that cannot exist — §2.5's `71%` live utilisation and its "VRAM
budget default 75 % of the smallest active device", since `wgpu` 30 exposes
no system-wide utilisation and no VRAM size on any backend. What is shown
instead is `Device::generate_allocator_report()`, this app's own
allocations, verified by moving it with a real 64 MB buffer. `use_gpu =
false` is untouched and its structural test still passes. Verified: full
suites on all three touched crates, clean clippy, and two scripted headless
drives — one over the `#[func]` surface, one booting the real shell and
walking the Preferences submenus, with a simulated device click landing in
both the engine and `user://cartalith_settings.cfg`. Two real bugs were
found by running rather than reading (`gpu_vram_estimate` was asking
`WorldParams`, whose grid is `0×0` until the first generate; and a
working-set assertion claimed 4096² passed 2 GB when it is 640 MB). Still
open in §2.5: **PR-03**, CPU worker threads. Not verified: anything
graphical.
— previously, post **the two deferred auto-populate passes —
`_civSelectMetropolises` + `_civApplyRecovery` — and the deletion of Data ▸
Conversion**; three owner decisions of 2026-08-20). *Ports*: the v0.75
imperial-seat tier (reference **24961-24989**; the harness caught that the
function ends on 24989, not 24988 — the assert-your-line-ranges rule paying
for itself again) and the v0.82 static recovery phase (24619-24640). The
metropolis rule is Lawrence et al. 2016 as the reference states it: a
**capital** that is both a dominant trade hub (normalised betweenness >= 0.85)
and the seat of a large polity (faction holds >= 6 settlements), at most 1 per
faction and 3 in total, x-then-y tie-break. That required
`SettlementKind::Metropolis` and the reference's real rank-5 value in **every**
per-tier table this port had capped at Capital (catchment 2500, surplus 0.10,
trade-k 2.1, base pop 45000, tax 0.10, class rank 5, minDeg 5, tier floor
150000, first slot in `CIV_TIER_ORDER`). `rustc` found nine of those; it could
not find the three tier **predicates** (`==`/`matches!`: the faction-seat
filter, the province rank>=3 seed filter, `civ_is_exchange_tier`) — those came
from grepping the *reference* for its own 30 `metropolis` occurrences. Recovery
has three load-bearing details, each with its own fixture: `was_urban`
includes **Town** (wider than `civ_is_exchange_tier`); the RNG draw happens
**before** the abandonment test, so a dropped settlement still consumes one
value (every fixture also pins the stream position afterwards); and the
`max(8, pop)` floor applies **after** the tier decision. Both are wired
exactly where `_civIterativeAutoWorld` wires them (lines 25711 / 25761) behind
`set_metropolis_enabled` / `set_recovery_phase`, whose defaults are the
reference's own OFF/Stable — so an untouched engine still generates what it
generated before. The metropolis pass needed betweenness this port never built
a `_civNetworkMetrics` for; only the **ratio** is read, so the `(n-1)(n-2)`
normalisation cancels and the crate's existing Brandes over `topology.edges`
is bit-identical (a golden test pins that, rather than leaving it a comment).
Surfaced in `File ▸ New world ▸ Generation` as a checkbox and a five-entry
**Recovery phase** dropdown filled from the engine's own `_CIV_RECOVERY_NAME`
table; `metropolis` also joins `map_overlay.gd` (rank 5, glyph ★, LOD 0), the
layer popover's per-class filter, the Settlement tool's class dropdown,
`kind_from_str` and `get_settlements()`'s `kind` string. **Verified**: 28
golden fixtures from a transient Node `vm.runInContext` harness over three
verbatim slices, a **35-mutation sweep with every mutant killed**, 730 tests
green across `cartalith-civ`/`cartalith-godot` with zero regressions, clean
clippy/build, clean headless boot, and two scripted headless drives —
promotion produces exactly one metropolis (*Ushirsrest*, faction 3, pop 59 990)
on seed 31337, recovery phase 0 is bit-for-bit a no-op (20 settlements,
154 109 people) and phases I-IV move that to 11 769 / 34 087 / 83 588 /
135 891, strictly monotone. **Two pre-existing golden tests changed** — both
pinned this port's own documented Capital *cap*, not the reference, on
`TIMELINE_SCOPE.md` §9's explicit condition that they be revisited when this
landed; both were re-extracted from the reference, not hand-flipped. *Two
disclosed limitations*: the reference's `trade_hub`/`administrative` and
`ruins`/`fortified` trait writes have no home on `NamedSettlement` (the same
boundary `timeline_bridge.rs` already records) — `kind`/`pop` survive intact.
*Deletion*: **Data ▸ Conversion is gone** — all three rows, from
`menus.gd::_data()` and `data_manager_window.gd`'s `ROUTES`/`GROUP_ORDER`; the
Data manager now has four groups (*in · out · sources · checks*). Owner
accepted `GUI_GAP_REGISTER.md` §7.4's research, whose finding was a *naming*
one: no serious GIS app has a top-level Conversion route because conversion is
a parameter of an export and a property of a project, not a destination — and
two of the three rows were undefined in `DCC_SHELL_SPEC.md` itself. §7.4's
recommendation 3 (keep CRS as a project property) was declined: one flat km
projection, nothing to transform between. Closes `GUI_GAP_REGISTER.md`
**CV-04** and **CV-08**; **DM-07/08/09** resolved by deletion.
`PHASE2_SCOPE.md` milestone 21 carries the full record. Not verified: anything
graphical.
— previously, post **Heightmap import + tectonic inversion, and a
startup prompt** — the owner's *"when you open the app you should be prompted
to either load a project, import a heightmap and infer tectonics etc or
create a new world (akin to how the html works)"*. **The HTML's cold start
was checked first**: it opens onto a mandatory setup gate (`#onboard`, HTML
655-667) whose intro step is exactly three buttons — Generate / Load .zip /
Import a heightmap — so the owner's three options are the reference's three,
in its order. *Half 1*: `app.gd`'s `_ready()` now opens the world gallery in
a new **welcome mode** — two extra action tiles (*Create a new world*,
*Import a heightmap*) ahead of the existing `.zip` tile, a re-worded head,
and a *Continue without a world* opt-out. A mode rather than a third dialog
because the gallery already **is** the load-a-project surface and already
carries one action tile; the two extras appear only in welcome mode, so
`File ▸ Open project…` is unchanged. One deliberate departure: **not a
gate** — Escape leaves the empty shell exactly as it was, which a DCC shell
with eleven windows can afford and a blank web canvas cannot. *Half 2*: a
real reference feature this port lacked — new `cartalith-terrain/src/infer.rs`
(the six pure functions of the reference's `v0.106 TECTONIC INVERSION`, HTML
6641-6752: relief proxy → lowest-relief plate seeds → crust sign from mean
elevation → stress synthesised from relief and *updip*, since velocity
inversion is ill-posed → volcanic-arc chamfer decay → plate velocities) and
`cartalith-engine/src/import.rs` (PNG decode, Rec. 601 luma, aspect-derived
grid, then the forward machinery **verbatim** and the climate/flow tail).
**Bit-exact golden parity on the first run** — 4 harness-extracted fixtures
shaped to reach all six `BTYPE` codes, both wrap modes, both tie-breaks and
`stamp_volcanic_arcs`' empty early return; `assert_eq!` on `f32`, no
tolerance. **The mutation pass was the interesting part**: four survivors,
all real — every default radius is `max(FLOOR, W/DIVISOR)` and at fixture
widths the FLOOR always won, so a fifth 256×64 digest-pinned case was added;
two further survivors are documented as genuinely *unobservable* (plate floor
6→5 yields an identical grid; `js_hypot`/`js_exp` divergence is
input-specific and pinned by `cartalith-jsmath`'s own tests) rather than
chased. Two parity details found by reading rather than assuming: the
moisture correctors legitimately run against a **zero** flow field on this
path, and grid height comes from the **image's** aspect, not the caller's.
One disclosed carve-out: the pixel resample cannot be matched (the reference
goes through a `<canvas>` whose filter is implementation-defined), so
`PARITY_TESTING.md`'s own carve-out applies — everything downstream of the
field is exact. Two new `#[func]`s, threaded through `engine_bridge.gd` on
the existing generation signals; wired to the welcome tile and to
`Data ▸ Import ▸ Heightmaps (PNG)`, closing `GUI_GAP_REGISTER.md` **DM-01**
and the menu audit's **MS-02** (*Infer tectonics from heightmap*) — both
halves of MS-02 in one pass. Verified: 113 test binaries green with zero
regressions, clean clippy/build, and a scripted headless drive confirming the
prompt appears and cancels, that the welcome tiles do **not** leak into
`File ▸ Open project…`, and that a real PNG imports in 302 ms onto the
correct derived grid producing a live texture and **40 placed settlements**
— proof the inferred substrate reached the whole civ stack. A 2048×1024 PNG
imports to 512×256 in 391 ms, to 1024×512 in 988 ms. Still open: tile-map and
GeoJSON *import* (`DM-01b`); TIFF is now a closed question, not a pending
dependency decision — the reference's browser decode does not read it either.
Not verified: anything graphical.
— previously, post **Sprite-sheet slicer: the real slice
operation** — closes `GUI_GAP_REGISTER.md` AS-09/AS-10/AS-11, the last
engine gap behind the owner's "the asset slicer and management system lacks
the functionality the html had". New `cartalith-assets/src/slicer.rs`: a
golden-verified port of the reference's `SpriteSheetImporter` (HTML lines
**27465-27870**) — `computeCells`, `cropCell`, `applyChroma`, `isBlank`
(alpha **> 8**), `clampInt`'s 1..128 clamp, `addSlices`' targets and naming.
**The finding**: `computeCells`' spacing is a *half-gutter on interior edges
only*, not a pitch, so the outer cells come out `spacing/2` **wider** — the
equal-cell formula `asset_library_window.gd`'s overlay carried drew a grid
the slice would not have followed (golden fixture: 508px outer vs 504px
interior at 6 cols/spacing 8 over 3072px, against the equal-cell 505.33).
Four new `#[func]`s (`as_load_sheet`/`as_clear_sheet`/`as_slice_preview`/
`as_slice_apply`) over a sheet held on the session, so the live readout
re-runs the real detection pass without re-sending a multi-MB PNG per
keystroke. The modal is live end to end, and two things stay out of
GDScript: the `N cells detected · M non-empty` readout is now the engine's
**real** pass (the 8×8 sample it replaced is deleted), and the overlay draws
engine-computed cell spans. Slicing is non-destructive. **Two disclosed
deviations**: *Trim transparent edges* is a **port-side addition** — §8 asks
for it, the reference has no trim at all (its second pixel toggle is
`background → transparent` chroma keying, also wired now); and *Assign to
family / Fill from* is §8's framing of what the reference expresses as a
flat target-slot dropdown, composed from `add_item` with no new arithmetic
(the reference's own three targets are ported exactly). Verified: 5 new
golden tests over 27 reference-extracted fixtures, **seven mutations all
killed**, 502 Rust tests passing across both crates (zero regressions),
clean clippy/debug build, clean headless boot, and a scripted headless drive
of the whole surface through the real gdext boundary. Still open: the
slicer's *canvas interaction* — pan/zoom, draggable grid lines,
click-to-select cells — now `GUI_GAP_REGISTER.md` **AS-17**.
`ASSET_LIBRARY_SCOPE.md` §11 carries the full record.
— previously, post **Menu-structure audit: the v2.10 surface
inventory vs. the shipped shell** — GDScript-only, no Rust.
`design/Cartalith Menu Structure v2.dc.html` catalogues **202 menu rows**
across 9 columns and 41 categories (plus 22 navigator nodes, 6 inspector
contexts) — the most complete inventory of the *reference app's* control
surface this repo holds. It was **audited, not implemented**: its top bar
still shows the earlier seven menus and `DCC_SHELL_SPEC.md` §2 plus the
`42547d9` domain merge supersede that, so nothing was restructured. Split:
**71 live · 97 honestly disclosed · 17 absent with no disclosure anywhere
(including `GUI_GAP_REGISTER.md` itself) · 17 superseded**. The 17 cluster:
eleven are whole-network civ operations and generation passes `generate()`
absorbed, which a one-shot pipeline hides — there is no button missing from
a panel, there is a panel that never needed it. Nine became **disabled
controls with a real reason** (Center landmasses `#centerBtn`;
Auto-populate world; Clear places & routes; Recalculate territories; Clear
territory; Generate provinces; Add/remove faction — which closes **CV-07**;
Generate roads; Clear ways & journeys). Seven became **in-product prose**
(stage 04 Tectonics' `gap` string was *empty* and now names the three
structured-orogeny knobs; stage 06 Erosion's now also names Evolve
climate↔terrain and Sediment fill; the Data manager's `import_maps` reason
named **Infer tectonics from heightmap** as a *second* gap (**both halves
built 2026-08-20** — see the heightmap-import entry at the top of this file;
that route is now live and the reason text describes what remains); CARTO ▸ Layers
gained a "Not built" section; `right_dock.gd`'s Sample panel gained dashed
**Route cost** and **E–W profile** rows — both in §6's own list, both simply
absent). One **dangling pointer fixed**: `world_workspace.gd` sent readers
to Preferences ▸ Tiles & LOD for chunk debug and that tooltip never
mentioned it. `render_workspace.gd`'s one umbrella note gained an inventory
of the ~60 designed controls behind it. **Wired live** — the only omission
whose engine backing already existed: **per-class settlement and
by-way-type road filters** (`#explSettlementFilterList`/`#explShowRoads`),
a draw-time test in `map_overlay.gd` (hidden-sets, so empty means show-all)
behind two new `viewport_host.gd` entry points and two L4 groups in CARTO ▸
Layers; a hidden class stays hoverable and clickable. Verified: parse-check
on all 11 edited files clean, headless boot of `shell/app.tscn` clean, and a
scripted headless drive that exercised both filters in both directions and
tree-walked tooltips to confirm all nine disclosures reachable. Naming
insights from the canvas are **recommendations only** in
`GUI_GAP_REGISTER.md` §13.6, not applied. Full record: that file's new §13.
— previously, post **Asset Library: a real `#[func]` surface**
— closes `GUI_GAP_REGISTER.md` AS-01..AS-08/AS-13/DM-05. New
`cartalith-godot/src/asset_bridge.rs` (`AssetLibrarySession`: a live
`AssetDB` plus a parallel decoded-pixel store) behind a 20-function `as_*`
`#[func]` surface on `WorldGen` (a field that survives a re-generate, like
`travel_library`) — import, per-slot fill state, real baked thumbnails,
inspector queries, pack metadata, validate, export (`archive::write_pack`),
apply-to-map (the reference's own `applyToMap()`, no round trip through a
file), and five batch ops (tag/collect/rename/duplicate/delete) read
directly off the reference's own `alBatch*` handlers. One small new
`cartalith-assets` accessor, `AssetDB::item_mut`, for frozen-slot batch
rename (which renames item *variants*, not the slot — frozen slot names are
the constant `slot_title`, never editable, an honest spec/engine
disagreement kept rather than papered over). `asset_library_window.gd`'s
grid now shows real fill state/thumbnails instead of a permanent
checkerboard; `engine_bridge.gd` gained `has_method`-guarded `as_*`
wrappers (the `tl_*` convention); `menus.gd`'s `_assets()` gained the
`Assets ▸ Asset pack ▸` submenu (AS-13/omission O2); `data_manager_window
.gd`'s Export ▸ Assets route (DM-05) now routes to the real export. Still
gap, honestly: the sprite-sheet slicer's actual slice operation (AS-09/10/
11 — a real engine gap, out of this pass's scope) and per-item scale/pan
*editing* (reading is real, no `as_set_item_transform` writes one back
yet). Verified: 344 Rust tests passing across both crates (13 new), `cargo build -p
cartalith-godot` clean, a headless `WorldGen`-direct `--script` drive
printing `ALL PASS` over the full import→fill→thumbnail→batch→validate→
export→disk-round-trip→apply-to-map→delete→clear cycle, and a full shell
boot (`app.tscn`) with zero script errors — all run in an isolated `git
worktree` since a concurrent session held the shared build lock.
`ASSET_LIBRARY_SCOPE.md` §10 and `GUI_GAP_REGISTER.md` §6.3/§6.4 carry the
full record.
— previously, post **"Layers don't work" — LOD tile layer occluded
every overlay** (owner report, verbatim: *"the version i seem to open with godot
seems rather crude and incomplete. a host of options such as layers dont
work"*) — GDScript-only, no Rust. `viewport_host.gd`'s `_lod_layer` was added
to `_camera` after `territory_view`/`province_view`/`_debug_layer` and so drew
over all three at full opacity; `_update_lod()` activates at any fit scale above
1 px/cell, which the common small presets already exceed at `_zoom == 1.0`, so
every Layers field view, the faction fill and the province boundaries were being
covered from the first frame after each generate — measured as literally **0**
differing screen pixels between `off` and `temp` before the fix, 32,750 after.
`_lod_layer` moved directly above `map_view`; `layers_popover.gd`'s hotkey
badges now skip permanently-unavailable rows (digit `4` had been landing on
Köppen, a gap row, since the seven new Climate views landed). `TOOLCHAIN.md`
gained the two build hazards the investigation turned up: the running editor
locks `target/debug/cartalith_godot.dll` so `cargo build` fails while the stale
DLL keeps loading, and the debug entry is what every routine run takes. See the
section below for what is left open (`main.tscn`). — previously, post
**In-shell file dialogs: the world gallery and
the breadcrumb browser** — GDScript-only, no Rust. The re-vendored
`design/Cartalith DCC Shell.dc.html` (commit `419be0d`) gained two screens,
**"Open project dialog 1920"** and **"Select folder dialog 1920"**, both built
now; **no stock Godot `FileDialog` survives on any path this pass owns**. Open
project is a *world gallery*, not a browser (its own mockup comment: "gallery
grid — thumbnails, not a tree list"): search well, `Recent`/`All worlds`/
`Shared` scope chips, a four-column tile grid whose first cell is a dashed
`.zip` drop zone, `CURRENT` badge, `seed · edited N ago` captions, foot naming
the projects root — `open_project_dialog.gd` (`OpenProjectDialog`). Select
folder is the *breadcrumb browser* ("replaces the stock OS tree picker"):
clickable breadcrumb with `⌂ Home`, typeable path well, flat rows with an
`N items` meta, selected row accent-outlined with a `selected` tag, files
dimmed, a `＋ New folder…` row, `Cancel` / `Use this folder` —
`browse_dialog.gd` (`DccBrowseDialog`), with a `PickKind` of `FOLDERS` or
`FILES` (`Mode` is taken by `Window`; the clash was caught at boot). **Call
sites**: `app.gd::open_project_picker()` → gallery; `app.gd::_browse_root()`
(all four storage roots) → `choose_folder`; `app.gd::open_asset_pack_picker()`
→ `choose_file` filtered to `.zip`; `_pick_file()` deleted. Callback shapes
unchanged, so `menus.gd`/`data_manager_window.gd` needed no edit. **Asset
packs deliberately get the browser, not the gallery** — the gallery is
world-shaped in every part the design draws (seeds, edit times, a `CURRENT`
badge, a title reading "choose a world to continue"). Backing is real
`DirAccess`/`FileAccess`; `DccSettings.recent_projects()` feeds `Recent`, the
projects root feeds `All worlds`, and seeds are read from each save's
`params.json` via `ZIPReader`, cached per path+mtime. Three disclosed gaps,
not faked: `Shared` is drawn and disabled (no shared/remote project concept
exists), thumbnails are path-hashed radial gradients (a `.zip` save stores no
preview image), and the mockup's dashed borders are solid at the same colour
and weight (`StyleBoxFlat` has no dash pattern). Shared additions:
`DccTheme.FS_MODAL_TITLE`, `DccTheme.outline()`, `DccWidgets.modal_button()`,
and two drawn `DccIcons` glyphs (`search`/`import`, because U+2315 and U+2913
are tofu in Plex Mono *and* the fallback chain). **Verified**: a temporary
`_dialog_probe.gd` (deleted after use) built a real directory tree plus a
`ZIPPacker`-written save carrying seed `483920` and drove **25 checks, all
passing** — listing, dimming, breadcrumb navigation in and out, child-row and
current-folder confirms, file-mode enable/disable and extension rejection,
typed-path navigation, seed extraction, scope filtering, tile counts, search
match and non-match. Two real bugs caught by that drive and fixed:
`JSON.parse_string` rendering the seed as `483920.0`, and `queue_free()`'s
end-of-frame deferral letting two same-frame refreshes stack children.
— previously, post **Domain rail merge: five domains to three**
(owner instruction verbatim: *"Infra can be dropped as a name and can be
absorbed by civil"*, then *"And render into carto."*) — GDScript-only, no
Rust. `dcc_shell.gd`'s `DOMAINS` const goes from 5 entries to 3
(WORLD/CIVIL/CARTO); the surviving ids are `"civilization"` and
`"cartography"`, unchanged, so every existing `active_domain() ==
"civilization"` check stays correct. `InfrastructureWorkspace` and
`RenderWorkspace` are **not deleted** — both still own their real category
builders and tool click/drag/escape handlers, unmodified — they are now
*composed into* `CivilizationWorkspace`/`CartographyWorkspace` as nested
`VBoxContainer` children (`_infra`/`_render` fields) rather than getting their
own `app.gd::_register_workspaces()` entry and rail button. Both nested
classes gained one `_nested: bool` field: when true, their own
`_build_tools()` skips drawing a second, duplicate TOOLS row, since the host
(`civilization_workspace.gd`) now draws ONE combined row (Settlement ·
Territory · Way · Route) — the nested class still registers its own Way/Route
click/drag/escape handlers regardless, since those don't care which file drew
the button. CARTO's merge is simpler: RENDER never had a domain-specific tool
(`render_workspace.gd`'s own comment always said so), so nesting it only
adds its one "Terrain appearance" disclosed-placeholder section after CARTO's
three categories, no tools-row surgery needed. `app.gd` updated in three
places: `_register_workspaces()` (3 entries now), `_on_workspace_changed()`
(dropped the `"infrastructure"`/`"render"` match arms, merged their idle-state
text into the `"civilization"`/`"cartography"` arms), `_refresh_rail_foot()`'s
context dict. `journey_planner_view.gd`'s tool-takeover check
(`app.active_domain() == "infrastructure"`) and its two
`app._workspace_panels.get("infrastructure")` reads both moved to
`"civilization"` — Journey now correctly hides the *whole* CIVIL panel
(including the nested INFRA content) while armed, matching its own
documented contract ("swaps the whole domain viewport region"), not just an
INFRA slice of it as before. `DCC_SHELL_SPEC.md` (owner-supplied, explicitly
authorized for this one edit) and `GUI_GAP_REGISTER.md` both carry disclosed
correction notices, not silent rewrites — §3's domain table now shows three
rows, §4.5.4 (INFRA tools) and §4.5.5 (CARTO tools) keep their numbers and
gained a merge note each so `GUI_GAP_REGISTER.md`'s own IN-0x cross-references
still resolve. **Verified**: `select:` grep of the whole `godot-project/`
tree for `"infrastructure"`/`"render"` as domain-id string literals found
only comments (all updated) once the change was complete. A real headless
run (`Godot_v4.7.1-stable_win64_console.exe --headless --path godot-project
--script res://_domain_merge_check.gd`, temporary script, deleted after use
per this project's own "scripted headless drive" convention) instantiated the
real `app.tscn`, waited for `_ready()`, then confirmed live, not just
parsed: exactly 3 `_domain_buttons` (`world`/`civilization`/`cartography`);
selecting `civilization` shows all four tool tooltips (`Settlement (S)` /
`Territory (T)` / `Way (W)` / `Route (⇧R)`) in one row plus all seven
categories (Settlements/Population/Economy/Politics/Culture/Timeline/Roads/
Rivers/Ports/Trade/Logistics — the last five nested); selecting `cartography`
shows `Icon (I)`/`Label (L)` tools, Layers/Layer properties/Annotation
categories, and a `§ TERRAIN APPEARANCE` section with its real disclosed-gap
note; arming `journey` while `active_domain() == "civilization"` set
`journey_planner_view._active = true`, hid the civilization panel and the
map viewport, showed the Journey left panel and the timeline band, and
disarming restored both correctly. Plain `--headless --path godot-project
--quit` (no script) also exits 0 with zero script errors both before and
after the temporary check script's removal. — previously, post **Layer-visualization audit + seven new
debug views** (owner report: Ocean currents/Wind were missing from the
prior pass) — re-checked the reference's *real* `LAYER_GROUPS` (reference
HTML line 13639-13646: 32 rows, not the prior pass's 18-view list) directly
against the file rather than trusting that summary. Seven genuinely
buildable views added to `sample_bridge.rs`/`layers_popover.gd`: **Wind**
and **Ocean currents** (the two named examples — both drawn as the
reference's own hue-by-bearing/SST-anomaly colour rasters, *not* arrows;
verified against the reference's own pixel loop, lines 8510-8521; a new
`cartalith_climate::current_wind_field` plus the existing
`ocean_sst_anomaly`, both recomputed on demand rather than retained, matching
the reference's own uncached `currentWindField()`/`currentOceanField()`),
**Water access**, **Flood**, **Resources**, **Carrying capacity**, and
**Settlement suitability** (all reuse already golden-tested `cartalith-civ`
builder functions over already-retained `WorldState` fields — none need a
civilisation layer). The other eighteen reference rows are confirmed,
disclosed engine gaps, not unexposed data (grepped every subsystem crate for
each): Köppen, Orogeny's signed preview (needs the boundary-polyline
structure `generate_terrain` folds into height and never retains), Geoid,
Tides (both already-disclosed-unported per `PlanetParams`), river
Velocity-erosion, Fjord, Landform, Population density, Site profile,
Wildlife, Wind-throw — listed in `LAYER_GROUPS`, `available: false` always,
the real reason in each hint, never faked. `FieldRefs` gained `shear_field`
plus the climate/planet params the two new views need; `cartalith-climate`
promoted from a `[dev-dependencies]`-only to a regular `cartalith-godot`
dependency. **Verified**: `cartalith-climate`'s full suite passes (2 new
`current_wind_field` tests); `cartalith-godot --lib` 215/215 tests pass (20
in `sample_bridge`, 6 new this pass); `cargo build -p cartalith-godot` and a
Windows headless boot (`--headless --path godot-project --quit`) both clean;
a scripted headless drive (temporary, not committed, same convention the
Journey Planner pass below already used) generated a real world, called
`build_debug_texture` through the real gdext boundary for all seven new
views (confirmed non-uniform `ImageTexture` output, not a placeholder), and
confirmed all eleven gap views report `available: false` and build no
texture. `cartalith-godot/src/lib.rs`'s own small wiring addition (the
`FieldRefs` construction site in `sample_refs()`) sits alongside the
concurrently in-progress Travel Library pass's own edits to that same file
(see the entry directly below) — that file is left for whoever lands it to
commit with both diffs together; everything else this pass touched
(`cartalith-climate`, `cartalith-godot/Cargo.toml`, `sample_bridge.rs`, this
file, `CHANGELOG.md`) is committed on its own. — previously, post **Travel Library: the `#[func]` boundary, a
live `jp_compute` wiring, and the `Data ▸ Travel library…` window**
(`TRAVEL_LIBRARY_SPEC.md` §6, `GUI_GAP_REGISTER.md` DM-15/O1 done, JP-02/IN-06
unblocked) — milestone 1 (data model, stock content, validation, the
`jp_plan_ex`/`JpAnimalResolver` Rust-internal wiring) landed earlier the same
day; this pass is milestone 2, the boundary and the GUI. `WorldGen` gained a
`travel_library: travel_bridge::TravelLibrary` field, bootstrapped with stock
content in `init()` and **not reset by `absorb()`** — a deliberate choice
(user-editable project state, not civ-generation output, so it survives a
re-generate the same way `asset_pack`/`quality` already do). A full `tl_*`
`#[func]` surface (`tl_counts`/`tl_list`/`tl_get`/`tl_duplicate`/
`tl_add_blank`/`tl_delete`/`tl_reset_to_stock`/`tl_edit`/
`tl_capture_preset_from_plan`) dispatches over `kind` for all four §3
definition types rather than four separate surfaces; `travel_bridge.rs`
gained the `Variant`-shaped field-pairs conversion layer this needed
(`animal_to_pairs`/`animal_apply_pairs` and three siblings), reusing
`journey_bridge::JpValue`/`jp_pairs_dict`/`jp_dict_to_pairs` rather than a
second flattening convention — `journey_bridge.rs`'s own `JpValue::num/int/
text/flag` went from private to `pub(crate)` for exactly this reuse.
**`jp_compute` now actually calls `jp_plan_ex` with a resolver built from the
live library**, unconditionally — a stock-only library is regression-tested
(`assert_eq!`, full structural equality) identical to the old `jp_plan` call
it replaced. `travel_library_window.gd` is the real `2a`/`2b` window from the
mockup: `Data ▸ ⧉ Travel library… ⇧L` (own popup window, `menus.gd`/`app.gd`),
tabbed by type, Custom/Stock entries rail with filter/add/duplicate/delete,
a grouped field inspector matching §3's own group names, save/duplicate/
revert staged-edit footer, and ok/incomplete/conflicting validation banners
using `DccTheme`'s `warn`/`water`/`block` tokens (the mockup's own exact
hex, already-named tokens rather than re-hardcoded). Honestly disclosed in
the window itself, unchanged from before: the planner's own party form does
not yet offer a custom entry as a Transport/mount option
(`journey_planner_view.gd` was mid-edit by a concurrent pass and
deliberately untouched); only the four built-in species affect a computed
plan; vehicles/vessels are still data-only. `cargo test -p cartalith-civ -p
cartalith-godot`: 335 + 215 passed, 0 failed (civ's own suite grew during
this session from unrelated concurrent work; the godot suite's travel_bridge
module alone is 22 tests, up from 15). `cargo build -p cartalith-godot` and a
headless Godot boot (`--quit-after 60`, both before and after the concurrent
work in this shared tree finished landing) are clean. — previously, post
**Journey planner: timeline band,
blocked-stage inline resolutions, supply-reach per-leg bar, `auto ·
<resolved>` party-form labels** (`GUI_GAP_REGISTER.md` §6.9/§10 JP-12, JP-13,
JP-14, JP-15) — GDScript-only, `journey_planner_view.gd`: **JP-13** —
`app.timeline_row`, drawn visible and empty in INFRA the whole time JOURNEY
was armed (the register's own #1 priority, §11: "the one place in the shell
showing an empty region with no explanation"), now carries a real
`_draw()`-based day-band strip (`_TimelineBandView`, same convention as this
file's own `_RouteMapView`/`_ProfileView`) built from `results[i].days` per
stage (`accent` land / `water` token river-sea) plus one trailing `text_dim`
block for `rest_days + layover_days` combined — combined because the
engine's own model treats them as calendar time laid on top of travel, not
assigned to specific days. "Weather hold" (one of the spec's four
categories) is never lit: `jp_plan` carries no discrete weather-hold day
count anywhere, only `jp_weather_factor`'s continuous per-leg multiplier
already folded into `days` — the legend still names it, with a tooltip
saying why. **JP-14** — up to three inline buttons at a blocked stage (verdict
card + stage inspector), each a `_plan_values` edit plus a `_compute()`
recall: turn off seasonal closures (only when `blocked_seasonal`), force
Walking land-only (transport flip *and* zeroing carts/wagons — the
wheeled-vehicle block reads cart/wagon count, not `transport`), depart a
season earlier (only when not already first in the list). Deliberately not a
port of v2.10's real-pathfinding "re-route land-only" — that stays exactly
where JP-01/JP-03 already left it. **JP-12** — a real per-leg supply-reach
bar with resupply ticks, `_stop_fractions()` (already used by the stops
strip) turning `plan.stops`' positions into leg boundaries, each leg lit
`block` when its own km exceeds `resupply_reach.required_km`. **JP-15** —
party-form auto fields (`rest_cadence`/`route_cond`/`infra`/`mount_animal`/
`desert_water`) now show `Auto · <resolved>` (`weather_override` stays plain
`Auto` — its auto is a continuous blend, no single resolved value exists),
refreshed post-compute by relabelling the tracked `OptionButton`s in place
rather than a full `_rebuild_party_form()`, which would drop focus mid-edit.
No Rust touched. Headless boot clean; a scripted headless drive (temporary,
not committed) generated a real world, committed a real 14-stage route, hit
a genuinely blocked default plan (`Overloaded 167%…`), confirmed the JP-14
button correctly left it blocked (not a transport-mode cause) with the band
showing the disclosure line throughout, then cleared it to a real feasible
plan (`total_days=21.47`, `rest_days=3`) and read back 15 real timeline
segments summing exactly to `total_days`, real `Auto · <value>` labels, and
the reach bar's real `ColorRect` children (4 lit, 5 ticks, 2 legitimately
zero-width where the route's start coincided with a stop). — previously,
post **Light theme + follow system, Window menu
workspace/open-windows lists, dock width dragging, rail expansion sub-node
list** (`GUI_GAP_REGISTER.md` §6.5/§6.6/§6.15 PR-13, PR-14, WI-02, WI-03,
WI-04, SH-01) — GDScript-only, `dcc_shell.gd`/`menus.gd`/`dcc_theme.gd`:
**PR-13/PR-14** — `DccTheme.apply_theme(is_dark)` re-points the active
palette; `DccShell.rebuild_theme(was_dark)` is the other half the old code
never had — it walks the whole tree (frame chrome, workspace panels, popups,
already-open dialogs alike) and, for the exhaustive grepped set of theme
override names this codebase actually uses, calls the new `DccTheme.remap()`
to reverse-derive each node's *token* from the colour it already has under
the *old* palette and repaint it with that token's colour under the new one
— a colour that matches no token (a bare literal) is left alone. Preferences
▸ Theme ▸ Light is live rather than permanently disabled, and a third
**Follow system** choice reads `DisplayServer.is_dark_mode()` once and
applies it (§2.5's "three discrete choices, not a live subscription" — it
does not watch for a later OS change). **WI-02/WI-03** — the Window menu
gained two real submenus: **Workspace** (the five `DccShell.DOMAINS`, jumping
to one via a new public `DccShell.select_domain()`) and **Open windows**
(`DccApp`'s five `AcceptDialog`s — it had grown from four — listed live only
while `.visible`, rebuilt every `about_to_popup` like `Recent worlds`
already does; picking one calls `popup_centered()` again to raise it).
**WI-04** — both docks got a real 6 px drag grip at their inner edge
(`Control.CURSOR_HSPLIT`, mouse-focus-follows-the-initial-press the same way
`SplitContainer`'s own dragger works), clamped live to §1's real min/max
(left 300-520, right 260-460) and writing straight into the existing
`_left_width`/`_right_width` fields `_toggle_dock()` already trusted.
**SH-01** — the rail's expand chevron is a real button now: pressed, it
grows the rail to `W_RAIL_EXPANDED` (200 px) and swaps the domain-button
column for a `_phone_list_row()`-built list (the register's own §7.17
proposal, reused verbatim per its own recommendation) — one row per domain,
titled with its label and subtitled with its *real* dock sub-structure
(`DOMAINS[i].subnodes`, sourced from each workspace's own build order:
WORLD's Generation pipeline/Sculpt/Biome paint switch, CIVIL's six
`DccWidgets.category()` accordions, INFRA's five, CARTO's three, RENDER's
one) rather than invented categories; picking a row jumps to that domain and
collapses the rail back, mirroring the phone drawer's own close-after-pick.
No Rust touched; headless Godot 4.7.1 boot clean (`--headless --path
godot-project --quit`, zero errors/warnings); a scripted headless drive
(temporary, not committed) instantiated `app.tscn` directly and called into
all five features programmatically — rail toggle (40→200px, 10 rows),
dock drag (372→387px on a +15px delta, clamps at 520), theme rebuild (left
dock panel bg flipped from the dark `panel` token's exact RGB to the light
token's, and back), and both new Window submenus (workspace check-marks
tracked the active domain; the open-windows list showed exactly the one
open dialog, then "No windows open") — all read back exactly as built. Known
gap, disclosed rather than silently partial: `rebuild_theme()`'s token-match
walk only repaints nodes that already exist in the tree; a workspace panel
or dialog that used a hardcoded literal instead of `DccTheme.c()` (none
found by inspection, but not exhaustively proven) would not flip. — previously,
same day, post **Layers popover hotkey badges + viewport
coordinates/elevation readout** (`GUI_GAP_REGISTER.md` §6.15/§10 SH-05, SH-06)
— GDScript-only: `layers_popover.gd` now badges its first 8 rows with digits
1-8 (`_add_hotkey_badge`, the mockup's own `border:1px solid currentColor`/
opacity-.75-active-.55-inactive badge markup reproduced exactly) and wires
real runtime `InputMap` actions (`layers_hotkey_1`..`_8`, `KEY_1`..`KEY_8`,
registered once in `_register_hotkeys()`) scoped to "popover visibly open" by
a `visible` guard in a new `_input()`; the badge order is the popover's own
real build order (`LAYER_GROUPS`' verbatim Base/Climate/Tectonics/Hydrology/
Surface/Civilization, ported from the reference on purpose — see that file's
own header) rather than `DCC_SHELL_SPEC.md` §10's SURFACE/TERRAIN FIELDS/
CLIMATE grouping, which has no matching data (no "Relief" row exists at all,
"Political" sits under Civilization, last) — re-sorting client-side would
scatter the eight hotkeys across non-adjacent groups with nothing on screen
to explain the jump, a materially riskier change than the badge itself, so
this was noted rather than forced; **SH-06 turned out to be a real (B), not
the register's (A)** — `viewport_host.gd`'s bottom-right cursor readout
previously showed bare grid indices (`"%d, %d"`, no km, no elevation at all,
contradicting `DCC_CONTROL_INDEX.md`'s stale claim that it already read
`E · N (cell)`) and now shows the real thing (`4 812 km E · 1 093 km N ·
1 462 m`, `_coords_text()`, using `_width_km`/`grid_size()` for the
conversion and `EngineBridge.sample_cell()`'s `elevation_m` for the
committed elevation), but the design's own `→ 1 582 m` draft-stamp suffix
could not be built: `sample_cell()` reads only `WorldState::field`, never
`self.sculpt`'s draft `PassBuffer`, and `build_sculpt_preview_texture()`
composites the draft only into a full-grid *colourised* texture through the
appearance/hillshade pipeline — there is no `#[func]` that returns the
draft's raw elevation at one cell. Left honestly absent rather than faked;
closing it needs one new Rust entry point. `GUI_GAP_REGISTER.md` corrected in
place. No Rust touched; headless Godot 4.7.1 boot clean
(`--headless --path godot-project --quit`) — previously, same day, post
**right dock: RD-03/RD-06/RD-08/RD-11 wired**
(`GUI_GAP_REGISTER.md` §10 ranks 1, 2, 5) — the register's own top of the (A)
list, done in one pass, GDScript-only: Settlement ▸ Economy/Politics/
Logistics now open `world_data_window`'s Economy tab (new `WorldDataWindow
.open(tab)`/`DccApp.open_world_data(tab)` param, mirroring
`DataManagerWindow.open(group)`'s own "scope to X" shape), this dock's own
Faction context (`show_faction()`), and the Journey Planner tool takeover
(`app.open_journey_planner()`) respectively; Faction ▸ Territory/Roster now
read `civ_faction_territory_stats()`/`get_factions()` for real (culture,
colour swatch, settlement count, claimed cells/area/contested) instead of a
"—" placeholder and a bare province-name list; and `right_dock.gd` now calls
`DccShell.set_dock_readout("right", …)` at the end of every `_rebuild()` (and
live on every cursor sample, matching the elevation label it mirrors) —
elevation for Sample, settlement name for Settlement, faction culture for
Faction, route length for Route, chain/region/stamp counts elsewhere. See
"Right dock: RD-03/RD-06/RD-08/RD-11" below. No Rust touched; headless Godot
4.7.1 boot clean; a scripted drive (temporary, not committed) generated a
world and exercised all four live — see that section for the exact readback;
previously, same day, post **GUI gap register** (`GUI_GAP_REGISTER.md`,
new, repo root) — the owner asked to verify every GUI element is tested and
connected, and where not, that a design exists; the premise does not hold and
that is by design, so this pass built the "if not" branch: **123 catalogued
disconnected surfaces**, each classified **(A)** designed + engine-ready (17),
**(B)** designed but engine-blocked (71, each naming the specific missing
capability, with a wrapper/small/large cost axis — **22 are wrapper-cost**,
waiting on a boundary crossing rather than a capability), **(C)** undesigned
(23, each researched against Blender/Photoshop/Krita/Resolve/QGIS/ArcGIS Pro/
Mapbox/Gaea/World Machine/Wonderdraft/Inkarnate with a concrete proposal and
source URLs), **(D)** deliberate owner decision (12, no design proposed); plus
a **menu-naming audit** (findings all against the spec, not the shell, which
matches it exactly: `Data` is overloaded and carries `Journey planner… ⇧J`
which is a *tool*; `Preferences` mixes application and project scope where
every comparable splits them; `Edit` is ten disabled items and nothing else;
`CIVIL` reads ambiguously against `INFRA`; CARTO vs RENDER contradict each
other over terrain appearance); **five stale disclosed reasons corrected**
(reason text only — Faction ▸ Territory's "no per-faction query exists" when
`civ_faction_territory_stats` and `get_factions().claimed_cells` both exist;
`app.gd`'s "the §4.5 tool palette is not built yet" when both docks build one;
the Journey Planner's "the cost model has no Rust port" when
`cartalith_civ::jp_journey_cost` is ported, golden-tested and simply never
called; "No tile atlas yet" when deep-zoom LOD tiling is live; and
`cartalith-spatial` "standalone, unintegrated" since 2026-08-18); **nine
omissions found** — designed surfaces absent entirely, chief among them
`Data ▸ ⧉ Travel library… ⇧L` (whole store built and tested in
`travel_bridge.rs`, no `#[func]`, no menu item), `Assets ▸ Asset pack ▸`'s
entire 24-control submenu, the Journey Planner's timeline band, and the right
dock's `Layers` context; **one real defect found and left for its own
dispatch**: `timeline_bar` is drawn visible and empty in CIVIL and INFRA, the
one place the shell shows a region with nothing in it and no disclosure; the
`#[func]` surface was re-enumerated at **151 methods across 15 modules**
against the 38 `DCC_CONTROL_INDEX.md` counted, which is why several of its
"backed, unwired" rows have moved; no Rust changed, no GUI behaviour changed,
headless Godot 4.7.1 boot clean; see its own section below — previously, same
day, post **two owner-reported rendering bugs: deep-zoom
tile drops, settlement-pin fidelity** — both root-caused against real
headless repros rather than assumed, both fixed, GDScript-only:
`viewport_host.gd`'s `MAX_LOD_TILES_PER_UPDATE`-capped deep-zoom tiles were
silently dropped and never retried once the camera stopped moving (a
`_lod_backlog` + `_process()` staggered catch-up now drains them; also
fixed in the same file, a tile whose `detail_level` changed while its index
stayed fixed was never rebuilt at the new tier); `map_overlay.gd`'s
settlement pins had no inverse-camera-zoom compensation (the reference's
`_civZoomK()`) so grew unboundedly with zoom instead of holding a roughly
constant on-screen size, now fixed via a new `_camera_zoom`/`set_camera_
zoom()`/`_civ_zoom_k()` on the overlay, pushed from `ViewportHost._zoom_at`/
`reset_view`; `draw_circle`'s `antialiased` param (defaults `false` in
Godot 4) is now passed `true` at every settlement/icon/handle/measure-point
call site in `map_overlay.gd`/`tool_overlay.gd`. See "Two owner-reported
rendering bugs" below; previously, same day, post **DCC shell GUI audit** —
six findings fixed across `dcc_shell.gd`/`right_dock.gd`/`menus.gd`/
`data_manager_window.gd`/`performance_window.gd`/`world_data_window.gd`,
see "DCC shell GUI audit" below; previously, same day, post **Travel
Library milestone 1: data model + engine wiring** (`TRAVEL_LIBRARY_SPEC.md`) — a genuinely new, owner-supplied DCC-shell addition (no reference-HTML equivalent, no golden-parity target for any of it): the four §3 definition types (animals & mounts, vehicles, vessels, party set-ups) as a new godot-free `cartalith-civ::travel_library` module — data shapes, §4 validation with all three states (ok/incomplete/conflicting) reachable and unit-tested, and stock content (7 animals/5 vehicles/11 vessels/2 party presets — the four built-in party-form species, donkey/mule/camel/horse, mirror `jp_animal_stats`'s own golden-tested figures exactly; Ox/Yak/Reindeer and every vehicle/vessel constant are new, domain-plausible figures, disclosed as such) — plus a new `cartalith-godot::travel_bridge` module: the mutable stock-plus-custom CRUD store (`EntrySet<T>` generic over all four types, duplicate-to-edit/add-blank/delete/reset-to-stock, the same convention `cartalith-assets`' Asset Library established), and usage tracking (real for party-preset references by species; honestly always `0` for "saved journeys" — no persistent, referenceable journey exists anywhere in this port, `route_get`/`infra.routes` are drawn polylines with no attached plan); **real engine wiring, not a parallel table**: `cartalith-civ::jp_capacity`/`jp_calc_land`/`jp_plan` each gained an `_ex` sibling taking an optional `JpAnimalResolver` (two closures, falling back centrally to the built-in table field-by-field on anything unset), so a custom Travel Library entry overriding one of the four built-in species changes computed capacity and speed and can hard-block a terrain outright — the original three functions are now one-line wrappers passing `None`, confirmed byte-for-byte unchanged by the full existing Journey Planner test suite passing unmodified; two new `travel_bridge.rs` integration tests prove the wiring end to end against a real `jp_plan_ex` call (a slower/higher-capacity custom donkey changes `days`/`avg_km_day`; a `blocked` terrain on a custom entry actually blocks that stage, `blocked_idx.is_some()`); **disclosed, named gaps**: a wholly new species (the stock Ox/Yak/Reindeer) has no `JpParty` slot to occupy (a fixed four-field struct, not a generic map) so is validated/inspectable but computationally inert until that struct grows a generic shape; vehicles/vessels are data-only — no resolver wired for `jp_capacity`'s cart/wagon/sled/travois constants or `jp_ship_stats`; no `#[func]` boundary exists yet by design (`lib.rs`'s `WorldGen`/`jp_compute` untouched — the paired GUI dispatch's own job, matching how `timeline_bridge.rs` landed cleanly ahead of its UI); 18 new `cartalith-civ` lib tests + 13 new `cartalith-godot` lib tests (327/202 total), 0 regressions anywhere in either crate's full test suite, clippy clean; `cargo build -p cartalith-godot` (cdylib, not just `cargo test`) + headless Godot 4.7.1 boot (`--headless --path godot-project --quit`) both clean; see its own section below — post **Timeline milestone 6: UI playback controls** — closes `TIMELINE_SCOPE.md`'s milestone list; a sixth `DccWidgets.category()` ("Timeline") in `civilization_workspace.gd`'s left dock (not a new `right_dock.gd` CTX_* context — `CTX_SCULPT`/`CTX_JOURNEY` are both tool-armed, Timeline has no map tool of its own) with years pill row + Add year (`civ_add_year`/`civ_goto_year`/`civ_remove_year`, `_civFormatYear` ported verbatim), a real-time-scale scrub slider snapping to the nearest recorded year (`_civWireYearSlider`'s v0.91 behavior), Play/Pause (a real 1200ms `Timer`, the reference's own interval) plus a Step button (this milestone's own addition, not in the reference's markup), the three exist-only/ghost/highlight filter checkboxes driving a live `civ_year_diff()` present/removed/added count readout, and the full collapse/recovery simulation form with a real `ConfirmationDialog` for the `needs_confirm`/`clobber_years` overwrite-warning path; deliberately NOT wired into `dcc_shell.gd`'s own `timeline_bar`/`timeline_row` (`DCC_CONTROL_INDEX.md` §10's reserved-but-empty bottom strip) per `TIMELINE_SCOPE.md` §4's own instruction to default to a dedicated panel when unsure whether a shell region is this discrete mechanism or the still-undecided six-toggle continuous-simulation feature; **one real, disclosed gap**: the three filter checkboxes read/display real engine state but cannot filter/ghost/highlight individual settlement pins on the map, because `get_settlements()` (`lib.rs`) carries no `tid` field even though `NamedSettlement` gained one in milestone 1 — closing it needs a Rust-side change, out of this GDScript-only milestone's own constraint; GDScript only, no Rust file touched (a separate, unrelated, already-uncommitted change to `cartalith-civ`/`cartalith-godot`'s `lib.rs` files was sitting in the working tree throughout this pass and was left alone, not staged); a scripted, discarded smoke scene (`_smoke_timeline.gd`/`.tscn`, deleted after this pass) instanced the real `app.tscn`, generated a real 160×160 world, and drove every new code path directly with real, changing numbers (a 141/0/0 present/removed/added diff, a real collapse-sim result — "57111 died, 15086 migrated... 125 settlements failed" — the needs-confirm-then-confirm round trip actually popping and re-submitting a real `ConfirmationDialog`, play/stop, add/remove) with no crash; headless Godot 4.7.1 boot (`--headless --path godot-project --quit`) clean both before and after; see its own section below — post **Timeline milestone 5: the Godot boundary** — new godot-free `cartalith-godot/src/timeline_bridge.rs` (`journey_bridge.rs`'s exact isolation pattern) plus a new `#[godot_api(secondary)]` block in `lib.rs`; 7 `#[func]`s on `WorldGen` — `civ_add_year`/`civ_goto_year`/`civ_remove_year` (thin wrappers over `CivData`'s milestone-4 methods), `get_civ_year`/`get_civ_timeline_years`, `civ_year_diff` (passthrough), and `civ_run_collapse_simulation` (the one real new wiring, a straight port of `_civRunCollapseSimulation`'s impure half, reference lines 24896-24950); the reference's blocking `confirm()`-before-overwrite dialog (lines 24910-24911) has no prior precedent anywhere in this port to match, so it's new design — a first call whose simulated years would clobber already-recorded entries returns `{"ok": false, "needs_confirm": true, "clobber_years": [...]}` and writes nothing, the caller re-sends with `confirm_overwrite: true` to proceed, the same "response field the caller checks" shape `jp_compute`'s own `rejected` array already establishes; the anchor/carry-forward claim was verified against the real reference rather than trusted from the task brief's own summary, and an early test draft's wrong assumption (an earlier-year anchor reachable in one call) was caught by the test itself failing, not assumed correct; a disclosed, out-of-scope gap found while wiring — `CollapsePlace`'s `fortified`/`ruins` (milestone 3) don't survive into a stored `TimelineSnapshot` (milestone 4's `Vec<NamedSettlement>` has neither field, and extending it would ripple far beyond this milestone's scope) — threaded correctly within one simulation run, lost only in what's stored for later scrubbing, inert since milestone 6 isn't built; one additive field on `CivData` (`dens: Vec<f32>`, `civ_current_agrarian_density`'s output, computed once in `compute_civilisation` from locals it already builds, same reasoning `water_bodies` was already kept for) rather than re-deriving the soil/water-access/biome sub-pipeline on every simulate call; all 7 methods wired into `engine_bridge.gd` with the standard `has_method()` guard, ready for milestone 6, no UI built; 189 `cartalith-godot` lib tests (+11, no Godot runtime needed for any of them), 309 `cartalith-civ` lib tests unchanged, 0 regressions, clippy clean; `cargo build -p cartalith-godot` (cdylib) + headless Godot 4.7.1 boot clean + `--check-only --script` on the GDScript addition clean; see its own section below — post **Timeline milestone 4: snapshot data model + orchestrator** — the pure orchestrator `civ_simulate_timeline` (`_civSimulateTimeline`) plus the manual-authoring snapshot/diff logic (`TimelineSnapshot`, `civ_year_diff`, `civ_snapshot_save`/`civ_snapshot_load`) in `cartalith-civ::timeline`, and `CivData`'s new `timeline`/`year` fields plus thin `civ_add_year`/`civ_goto_year`/`civ_remove_year`/`civ_year_diff` methods in `cartalith-godot/src/lib.rs` — no `#[func]` surface yet, milestone 5's job; `civ_assign_tid`/`civ_resync_next_tid` were already half-built by milestone 1, so this milestone only added the sibling `civ_resync_next_tid_with_timeline` that also folds in snapshot history; `civ_simulate_timeline` golden-verified against the real reference over 4 new tests (`golden_parity_timeline_orchestrator.rs`) proving multi-step baseline-normB threading and step-to-step chaining, plus 6 new `timeline.rs` unit tests (tid-vs-name year-diff disambiguation named explicitly) and 8 new `cartalith-godot` unit tests (`civ_timeline_tests`) proving the reference's own snapshot semantics — adding a year never loses the active year's live edits, `civ_goto_year` never touches settlements/ways, `civ_remove_year` falls back to the earliest remaining year or 0; a deliberate 2000-year snapshot cap logged as a disclosed deviation from the reference's unbounded storage; 309 `cartalith-civ` lib tests (+6) and 178 `cartalith-godot` lib tests (+8), 0 regressions, clippy clean; headless Godot 4.7.1 boot clean; nothing wired to a `#[func]` caller yet (milestones 5-6 remain); see its own section below — post **Timeline milestone 3: collapse/recovery step functions** — the mechanistic core of the v0.85 stepper (`_civSettlementStress`/`_civMortalityMigrationRates`/`_civGravityMigrate`/`_civCollapseStep`/`_civRecoveryGrowthStep`), depending on milestones 1-2; a new settlement-only `CollapsePlace` type (decoupled from `NamedSettlement`, which gains no `traits`/`ruins` fields for this stepper's sake alone), `CollapseCharacter` as a closed enum in place of the reference's string-keyed lookup, and both the collapse step's demote-only and the recovery step's promote-only invariants confirmed against the actual reference lines (not assumed from the scope doc's own summary) and pinned by named tests; `fortified` is sticky once set, `ruins` clears only on promotion back into an exchange tier, both golden-verified; the reference's dead `_K`-null fallback branches dropped per milestone 1's already-logged precedent; golden-verified over 9 new tests (abandonment-floor boundary, a fortified-vs-unfortified pair proving exactly the 1.5x bonus ratio, the gravity model's multi-pass saturation plus a genuine unplaced/diaspora-loss case, all four characters on one fixture both at the raw-stress level with a real `L`-term baseline and end-to-end through `civ_collapse_step` — trade/disease/conflict/mixed produce `failed` counts of 0/1/2/1 on the identical fixture — and a ruins-clearing-vs-not recovery pair) plus 7 new unit tests; 303 lib tests, 0 regressions, clippy clean (two deliberate NaN-preserving `#[allow]`s, commented); headless Godot 4.7.1 boot clean; nothing wired to any caller yet (milestones 4-6 remain); see its own section below — post **DCC shell: Storage locations, Recent worlds, Data manager window** — the owner-supplied shell spec's file/folder-browsing menus (§2.1 File, §2.4 Data, §2.5 Preferences, §9 the Data manager window), real for the first time: a new `ConfigFile`-backed `DccSettings` (first `user://` write in this shell) persists the four storage roots and a real last-10 recent-projects list; File ▸ Storage locations/Change locations…/Show project on disk and Preferences ▸ Application ▸ Storage locations… (the spec's own "same modal as File") are all real, with the atlas-cache invalidation note honestly saying no atlas cache exists yet rather than inventing one; a new `DataManagerWindow` (§9) exists for the first time with a real routes rail and route pane — Import ▸ World Data and Import ▸ Assets are genuinely real (routed to the exact same `.zip` picker / asset-pack picker File and Assets already use, not reimplemented), every other route (Export World Data/Maps/GIS/Assets, all of Sources/Conversion/Validation) stays a disclosed gap with its own verified reason, `cartalith-io`'s read-only status re-confirmed by reading the crate directly rather than trusting the old comment (its only `ZipWriter` lives in `#[cfg(test)]`); `menus.gd`'s `_live()`/`_todo()` convention preserved exactly throughout; no Rust touched (a separate concurrent pass owned `cartalith-civ`/`cartalith-godot` for a stable-id field at the same time); headless boot clean, a scripted (and discarded) smoke run exercised every new entry point including all 15 Data manager routes; see its own section below — post **Journey Planner distance-spine takeover** — `journey_planner_window.gd`'s `AcceptDialog` (same day, earlier pass) replaced by `journey_planner_view.gd`, an in-shell INFRA tool takeover per `DCC_SHELL_SPEC.md` §4.5.4's addition: arming JOURNEY swaps the whole viewport region (map, both docks, tool options bar) for the mockup's distance-spine layout — real route-map/terrain-profile geometry sliced from `route_get()` by `plan.stages[i].{i0,i1}`, a real elevation sparkline from `plan.profile` (closed for real this pass, only disclosed-and-skipped before), stage inspector + stage matrix writing into `jp_compute`'s real `stage_overrides`, results panel reading real `jp_compute` fields throughout; Carriage Auto/party presets/re-route-for-mode/Cost all disclosed as genuinely unported reference-JS-only features rather than faked; ⇧-drag spine trim deferred (no request field exists for it); wired from both `Data ▸ Journey planner… ⇧J` and the INFRA dock's Logistics button; headless boot clean, a scripted (and discarded) smoke run exercised arm/compute/override/disarm/domain-round-trip end to end; see its own section below — post **Sample panel + Layers popover** — `DCC_SHELL_SPEC.md` §6's twelve permanently-dashed Sample fields are all live, plus §6's elevation accent readout, and **none of them needed new retention** — every reading is a raster generation already keeps or is derived at the one queried cell; `build_lithology`/`build_soil_fertility` are called on **one-element slices** so not one golden-tested branch is restated in `cartalith-godot`; the old "retaining the rasters would cost hundreds of MB" note on Biome was a real over-generalisation of `explain_settlement`'s doc comment and is corrected in place; aspect is genuinely new work (the reference's `aspectFactor` is a shading scalar, not a bearing) and its first implementation was 180° out; 18 debug views in the reference's own six `LAYER_GROUPS`, every reference-existing ramp ported from its own pixel loop and pinned by test, the four new ones flagged as new; a real Layers popover replaces the layers button's domain-jump stand-in without removing anything it reached; see its own section below — post **Journey Planner Godot boundary + GDScript form** — post **Phase 5 milestone 7** — urban morphology's `grow`, the epoch loop everything downstream accretes onto, plus `logisticRamp`/`estimateCarryingCapacity`/`wallOccupancy`/`supersedeWall`, as `cartalith-urban::growth`; 60 golden scenarios with a **per-epoch** graph hash so a divergence localises to an epoch, **all matching on the first run**; `buildWall` is milestone 10's and is injected as a trait object with the golden capture stubbing the reference's own copy the same way, which is what made the fire epoch, the age gate, the occupancy gate, the generation cap and the supersession testable now; 214 mutations over two rounds, 176 died, 38 survived, zero false survivors; two rounds of fixtures lost to the same lesson — a terrain raster in metres makes **every** slope test in the engine reject, and a hand-drawn wall ring can never be 80% full — and the stated line range understated the milestone by six lines, six for six; first consumer of `cartalith-jsmath` since the consolidation and it **needed nothing new**; see its own section below — post **`cartalith-jsmath`** — the JS-semantics audit's recommendation #2, carried out: **every helper in the catalogue now has exactly one implementation**, in a new leaf crate with **no dependencies at all** (not even a dev-dependency — its bulk goldens carry a four-line inline `mulberry32` rather than borrowing `cartalith-rng`'s, so the leaf property is a fact about the manifest). It absorbed 7 copies of `js_hypot` (5 distinct compensated sums), 7 of `js_round`, 3 of `js_min`/`js_max`, both `toFixed` ports, `u8_clamped`, the NaN-falsiness pair, and the FDLIBM family (`js_exp`/`js_sin`/`js_cos`/`js_log`/`js_atan2`, plus `js_atan`, now public) that had been trapped one-per-crate where nothing else could reach it — **`cartalith-urban` milestone 8 would have been a ninth FDLIBM copy site**, since milestone 6 fixed that crate's dependency list to `cartalith-rng` alone. **No call site had to change**: load-bearing module paths (`geom::js_hypot`, `sculpt::js_hypot`, `tile_render::u8_clamped`, `spatial::geo::js_to_fixed`) survive as `pub use` re-exports. **All three copy disagreements resolved rather than recorded** — `js_round` onto the fractional-part form (the six `(x+0.5).floor()` copies and `cartalith-terrain`'s false "standard exact equivalent" comment are gone); one compensated `js_hypot_n` with `js_hypot`/`js_hypot3` as wrappers, so the inf/NaN preamble cannot be lost from one entry point again; and `js_min`/`js_max`'s signed zero pinned to V8 in **both** argument orders, where it turned out **all three copies were wrong** — `-urban`/`-civ`'s `if b < a` and `-terrain`'s `if a < b` each failed the order the other got right. **Both remaining live `atan2` hazards closed**: `cartalith-urban::graph:607`'s half-edge sort key, where `f64::atan2` differs from V8 on **38%** of the edge deltas this graph really produces and puts a near-parallel pair in a **different order 4.7%** of the time (all 20 milestone-2 golden scenarios pass **unmodified**, and the ordering itself is now pinned against `node` — V8 agrees with `js_atan2` 5/5 and with `f64::atan2` 0/5); and `cartalith-terrain:372`'s world-wrap plate circular mean, which the audit had refused to half-fix and which **became fixable** once `js_sin`/`js_cos` shared a crate with `js_atan2` — over 2,000 synthetic plates the `(Σ sin, Σ cos)` pair already differs on **737**, final `plate.x` differs on **193** with Rust's libm and on **110** with `js_atan2` alone (the audit's "differently wrong", now measured here), and on **0** with all three, with both `world` cases of `golden_parity_plates.rs` passing unmodified. **1138 → 1134 tests, and the −4 is fully accounted for** (8 moved, 15 duplicate helper tests deleted, 16 in the new crate, 3 new at the fixed call sites); **no existing golden expectation modified anywhere**; the moved goldens — including the FNV-1a hashes over 54,000 sin / 54,000 cos / 30,000 log — passed on the **first run** in their new home, which is the check that the move was pure. **440 mutants, 258 killed, 182 survived, 0 broken**, private `CARGO_TARGET_DIR` per run, post-sweep baseline green and both files byte-compared, 20 survivors re-run in isolation with **zero false survivors** and every survivor class named (56 sub-ulp constant moves, 55 equal-operand comparison flips, 36 one-step threshold bumps, 24 guards Rust's saturating casts make redundant, 11 inside `rem_pio2`'s unreachable third correction round). The first pass left **206** alive and **101 were inside `js_exp`/`js_atan2`** — the two functions predating the hash technique — so both got the bulk FNV-1a golden they were missing (48,000 exp / 54,000 atan2 arguments, **both matching V8 on the first run**), and the sweep additionally found **four real gaps in this pass's own tests** plus one **real divergence**: `js_fixed` returned Rust's `inf` where JS spells it `Infinity`, now fixed from `node`. One tooling finding worth more than the numbers: **a mutation operator can manufacture its own survivors** — the first round mutated inside `//` comments and bumped float constants' last written decimal digit, which for FDLIBM's 21-significant-figure literals parses to the *same double*; both fixed (code half of the line only, and a genuine one-ulp bit-pattern perturbation). `js_acos`/`js_log10` deliberately **not** added — milestones 10 and 15 will need them and dead code with no golden is what this project avoids. Nothing Godot-scene-side touched (UI hold) — post **Phase 5 milestone 6** — urban morphology's anchors and primary routes (`placeAnchors`/`buildPrimaries`/`buildPrimariesFromPaths`, `cartalith-urban::routes`), the first milestone that produces a real street graph end to end; **the stated line range was wrong for the fifth time in five** (28743-28833, not 28744-28843 — the last ten lines are milestone 8's header comment, so milestone 8's own start moves to 28835); **three more V8 libm divergences, measured *before* a golden failed rather than after** — `f64::sin` disagrees with V8 on 1,942 of 80,214 arguments, `f64::cos` on 2,160, `f64::ln` on 1,647 of 60,009, and the ported FDLIBM `js_sin`/`js_cos`/`js_log` on **0** of each, which **retro-fixes milestone 1 a second time** because `rng::norm` (Box-Muller, and therefore every frontage width, plot depth and building dimension in a town through `logn`) had been on the platform `ln` and `cos`; FDLIBM's Payne-Hanek branch deliberately **not** ported, with a test asserting the hand-off; `Math.pow(x,2)` measured **bit-identical** to `x*x`, so the one `Math.pow` needs nothing; neither route builder **draws a random number** and both return values are **discarded by `generate()`**, so milestone 16 inherits only the graph and an 800-draw substream; the market's third `||` arm and its `best === null` fallback are both live, and the fallback is the one thing in the subsystem that can put the market **outside the site box**; `Math.max(0, rd-260)` proven **dead on every site the engine can build** by an invariant test rather than by argument; **a metre offset added to a metre coordinate cannot express a one-ulp boundary**, which rebuilt both boundary fixtures and which milestone 17's adapter will hit; 38 golden scenarios, everything bit-exact with no tolerance, **all of them matching on the first run**; **306 mutations, 233 killed, 73 survivors**, every one re-run in isolation with **zero false survivors**, all six graded perturbations dying, and 54 of the 73 inside the new FDLIBM block with a named invariant per class; two tooling findings — **a dozen hand-picked golden rows cannot test a bit-twiddling port** (the first sweep left 63 survivors inside the three new libm functions; an FNV-1a hash over 54,000 sin / 54,000 cos / 30,000 log results killed them) and **two mutation runners on one target directory left a live mutation in the source**, now prevented by a pristine snapshot, a lock file and a post-sweep baseline; `Graph::from_paths` added for milestone 10; `extractFaces` flagged as still using `f64::atan2`; tested and unwired, no Godot file touched; see its own section below — post **`js_atan2` + the `build_channels` receiver fix** — acted on the JS-semantics audit's recommendation #1 and it turned out to be a **live bug, not a latent one**: `cartalith-hydrology::build_channels` was steering rivers into the **wrong cell**. V8 does not use the platform libm for `Math.atan2` — it ships FDLIBM's `__ieee754_atan2` in `src/base/ieee754.cc`, *including* the FreeBSD `m &= 1` correction for `|y/x| > 2^60` that the original 1993 Sun source lacks (without it the port is one ulp off V8 on 777 of 240,000 arguments). Ported `js_atan2` disagrees with V8 on **0 of 240,000** arguments and **0 of 1,089** special-value pairs, where `f64::atan2` disagrees on **40,824** and **42**. The bug is structural, not a coincidence: a left-right-symmetric 3x3 makes `gx` exactly `0.0`, so `aspect` is exactly `-pi/2` and the two symmetric downhill diagonals have **exactly equal** `drop` — the argmax is then decided by one last bit, and Rust and V8 decide it differently. That is a ridge, saddle or plateau edge, i.e. ordinary terrain. Over **1,200,000** random 3x3 blocks `f64::atan2` picks a different receiver from V8 on **84** and `js_atan2` on **0**; on all **43** divergent blocks re-run through `node`, V8 agreed with `js_atan2` 43/43 and with `f64::atan2` **0/43**. **River output can therefore change** on maps containing such cells — though all three `golden_parity_river.rs` cases pass **unmodified**, and instrumentation shows why they had to: their 365 channel cells include **not one** with `gx == 0.0` or a top-two score gap below `1e-15`, so they were structurally blind to it. `sin`/`cos` deliberately **not** ported (measured: they cannot reach this argmax, since the wrap only decides ties between exact negatives and `sin`/`cos` preserve antisymmetry exactly — 600,000 blocks, every receiver agreed). The other seven `atan2` sites each got a verdict: `-terrain::poly_meta` **safe, proved** (arguments always in `{-1,0,1}²`, all eight D8 values bit-identical to V8); `-civ::labels` **safe** (live pointer input, no reproducible reference to diverge from); `-terrain:372` **cannot be fixed by `js_atan2` alone** — Rust's `sin`/`cos` already give a different `(Σ sin, Σ cos)` from V8 on 92/2000 plates *before* `atan2`, so a partial fix would leave it differently wrong (its quantised consumer differs 0/2000; its unrounded one feeds a Lloyd argmin); `-urban::graph:607` **a real hazard, audited not touched** — `ang` is the half-edge **sort key** the face traversal walks, so one ulp reorders two near-parallel edges and changes a city block. Also fixed: the missing `js_hypot` inf/NaN preamble in **all three** copies (plus `tile_render::js_hypot3`, a seventh entry point the audit's table had not listed), each with a `node`-derived spec test; and `cartalith-terrain`'s false "standard exact equivalent" `js_round` comment. Left, with reasons: the six `js_round` implementations, and `js_min`'s signed zero. `js_atan2` lives in `cartalith-hydrology::jsmath` — an eighth FDLIBM copy site — because the `cartalith-jsmath` consolidation is still blocked on the live `cartalith-urban` fork (607 uncommitted lines in `geom.rs`); **re-recommended, not performed**. **1062 → 1069 tests, delta exactly the seven added, no existing golden expectation modified** — post **JS-semantics fidelity audit** — the first workspace-wide sweep for JS-vs-Rust semantic divergences, `JS_SEMANTICS_AUDIT.md` (new, repo root); **two real bugs found and fixed, both in `cartalith-spatial`**, both proved with a test that fails before and passes after — `PaintStamp::apply` painted rim cells the reference skips (`f64::hypot` vs V8 disagree on 1,398 of 4,096 integer offsets; the first radius where a *cell* changes is **125**, the 35-120-125 triple), and `js_to_fixed` rounded **down on roughly one value in ten** (a first dropped digit of `5` with any nonzero tail) plus negative ties the wrong way — the latter on **every GeoJSON coordinate and way length**, with `golden_parity_geojson.rs` structurally unable to see it because its world is exactly 50 km/cell so every coordinate it rounds is an integer, and with a **unit test that asserted the bug** because it had been written from a paraphrase of ECMA-262 instead of from `node`; **one new divergence found and not yet ported — `Math.atan2`, at 22.98%, the largest in the workspace** (vs 9.52% `exp`, 3.40% `ln`, 2.34% `sin`/`cos`, 0% `sqrt`), eight live sites and no `js_atan2` anywhere, the structural one being `cartalith-hydrology::build_channels` whose steering factor differs on 12.97% of aspects and feeds the argmax that picks the cell a river flows into; **the helpers disagree with each other in three measured ways**, none live (six crates' `js_round` differ from V8 on exactly one double, `0.49999999999999994`; three of four `js_hypot` copies lack the inf/NaN preamble; `js_min` disagrees on `min(+0,-0)`); a `cartalith-jsmath` leaf crate **recommended and deliberately not done** while three forks are in flight; and a large reviewed-and-safe list with the invariant for each — D8 tables are bit-identical to V8 on all nine values, `f64::clamp` already propagates NaN exactly as JS does (which is why divergence #3 has almost no live surface left), and `build_npp`'s `exp` was *measured* rather than assumed at 0 differing `f32` stores in 10 million samples; 1131 tests pass against a 1128 baseline, no pre-existing golden moved, neither active fork's files touched; see its own section below — post **unified tool plan milestone E2** — the deferred half of Region select/export: per-tile PNG (`cartalith-terrain::tile_render`, the hypsometric tint and v1.29 seam-safe hillshade), gzip (`cartalith-io::gzip`), the `.zip` assembly (`cartalith-assets`' `zipStore` **generalised** rather than duplicated — one function in the reference, three callers), `exportGeoJSON` plus its raster-to-vector tracer (`cartalith-spatial::geo` + `cartalith-engine::geojson`) and `regionNewWorldBtn`'s non-UI core; the archive conventions matched `cartalith-assets`' exactly, but **one milestone 2 had deliberately skipped is real** — `zipStore` stores rather than deflates when deflate does not shrink, and a region export hits it on three of four entries; four reference corrections (`Uint8ClampedArray` rounds ties to **even** and is not a cast, `hypso` extrapolates into **negative** channels, `toFixed` rounds ties to the **larger n** where Rust rounds to even, and the tracer's JS-`Map` overwrite yields a genuinely **unclosed** ring); E2 ran the **real** `exportRegionTiles` — which milestone E could not — and a fourth-tile disagreement turned out to be a **harness** bug (block #1's deferred boot `generate()` firing during the `setTimeout(0)` the export awaits between tiles), fixed, after which all four tiles match E's hashes and its disclosure is discharged; 18 golden + 61 unit tests, **everything bit-exact with no tolerance anywhere**, both GeoJSON documents compared as whole strings; **58 mutations, 54 killed, 4 equivalent-mutant survivors**, and the first sweep's ten survivors included **six real fixture gaps** — with degenerate-ring reachability settled by brute-forcing all 65 536 masks on a 4x4 grid through the reference's own tracer rather than argued; tested and unwired, no Godot file touched; the unified tool plan now has **only milestone F** left; see its own section below — post **Phase 2 milestone 20** — `_civFactionAggregates`, the last unstarted piece of the economy layer, ported in full as `cartalith_civ::civ_faction_aggregates` with `_civFactionCapital`, the `CIV_TAX_RATE`/`CIV_PRIMARY_SPECIALISATION` tables and `_civOceanDistField`; taken now because it was a **real blocker for something already built** — the GUI parity audit had re-classified `civ_culture_terrain_fit` from "needs wiring" to genuinely blocked, since its `terrain_mix`/`world_mean_terrain` inputs were computed by nothing, and they are now computed and golden-verified; the heuristic five-axis "power" composite ported **verbatim** rather than simplified (the reference labels it honestly, and simplifying would mean inventing a different heuristic with nothing to check it against); `CIV_MAX_TIER_RANK` is **5, not 4** — the reference normalises by its full ten-entry class table whose top tier this port does not model, and using 4 would have inflated two power axes by 25%; the resource-residency tension `ECONOMY_SCOPE.md` expected to force **does not bind**, because the half that unblocks culture-terrain-fit needs no resource field and `resources` is an `Option` porting the reference's own nullable `pots`; one real JS-semantics trap found by re-reading — **`NaN` is falsy in JS**, so the reference's `p.pop||0` absorbs a bad settlement instead of poisoning a whole faction row, now ported as `js_num_or_zero`/`js_truthy_num`; golden-verified over two fixtures whose shapes reach the edges deliberately, with **six input hashes exact** and a disclosed **pre-existing 1-3 f32 ULP climate divergence** handled by stated tolerances rather than papered over; **58 mutations, 56 killed, 2 equivalent-mutant survivors** — both re-proved genuinely tested with discriminating variants rather than accepted on assertion, and the first pass's other four survivors were real fixture gaps (a saturating power normaliser, the territory guard's untested upper bound, `Math.round`'s negative half, and an elevation-denominator floor no real sea level activates), each closed with a unit test and re-killed; tested and unwired, no Godot file touched; see its own section below — post **Phase 5 milestone 5** — urban morphology's
site model, `cartalith-urban::site`: `shoreFromMask`/`buildSite`/
`terrainSuitability`, the input contract every later stage of a town reads
the world through, on both the synthetic-seed path and the real-water /
real-heightfield raster paths the host app actually uses; the stated line
range was wrong at **both** ends for the fourth milestone running; it found
**the second V8 libm divergence** — `f64::exp` disagrees with V8 on 20,721 of
240,000 arguments where the ported FDLIBM `js_exp` disagrees on none, which
also retro-fixes milestone 1's `rng::logn` and therefore every parcel and
building dimension milestones 12-13 will draw; 59 tests, 19 + 36 golden
scenarios at 106 probes each, all bit-exact; **271 mutations, 240 killed, 31
reported survivors** each with the invariant it rests on — and the *first*
sweep's 46 survivors turned out to be two fixture gaps rather than equivalent
mutants, which is the transferable lesson: a geometric subsystem needs its
fixtures derived from the geometry under test, not sampled on a grid of round
fractions; tested and unwired, no Godot file touched; see its own section
below — post **unified tool plan milestone E** — the
Annotation & measure group, which closes the four tool-group engine halves
(A-E done, only **F**, shell wiring, remains): Label
(`cartalith-civ::labels`, arc-text glyph layout split at text measurement so
the crate still never touches a canvas), Icon stamp
(`cartalith-assets::manual`), Measure (`cartalith-spatial::measure`, **an
addition** — the reference has no measuring tool, so it has no golden test and
cannot), and Region select/export's compute + encoding core
(`cartalith-spatial::region`, `cartalith-terrain::amplify`,
`cartalith-io::tiles`, `cartalith-engine::region_export`); **the plan
described the wrong icon function** — `_carIconBrushStamp` is a dart-throwing
scatter *brush*, not a single-icon stamp, and it is deliberately unseeded, so
parity needed an injected RNG on both sides; `amplifyRegion` turns out to have
a **real division by zero** (`outW == 1` returns an all-NaN tile), ported
rather than fixed and pinned by a golden; **Region select/export was split** —
its PNG/gzip/`.zip`/GeoJSON half is now **milestone E2**, smaller than the plan
feared because the geometry is done; 49 golden tests, everything exact except
**two ULPs** in one arc label from `Math.sin`, pinned exactly; **89 mutations,
86 killed, 3 equivalent-mutant survivors**, and the first pass exposed ten real
fixture-shape gaps including five brush constants no golden *could* have caught
because a dart always lands on an integer cell; tested and unwired, no Godot
file touched; see its own section below — post **unified tool plan milestone
D** — the
Civilization group: Place settlement's manual-insertion path, Draw route/way's
whole pathfinder and Territory/faction's override, all in a new
`cartalith-civ::tools`; the plan's claim that `road_dijkstra` already covered
the pathfinding turned out **wrong** — `_civDijkstraPath` is a caller of that
kernel and its three cost grids, way discount, gravity, wrap-aware smoothing
and `reachable` flag were all unported, so porting them **unblocks the
Journey Planner's last blocked function** `_jpRerouteForMode`; territory
paint is flagged as a **superset** since the reference never had algorithmic
territory at all; golden-verified bit-exact over 16 cases, which found **two
real bugs in already-verified code** (a `km` sum across wrap-seam run
boundaries, and the first fixture able to distinguish V8's `Math.hypot`);
tested and unwired, no Godot file touched; see its own section below — post
**GUI parity Category-1 sweep** —
`GUI_FEATURE_PARITY_SCOPE.md`'s Category 1 closed: `get_settlements()`/
`get_provinces()`/`get_trade_balances()`/`get_gpu_stages_used()` finally have
GUI consumers, as a three-tab world-data browser behind
`Simulate ▸ Statistics…`/`Simulate ▸ Economy…` and a `View ▸ Performance
readout…`; six of the ten rows turned out already done by other forks
(asset-pack import, layer granularity, click-to-pin from DCC shell m1;
planet params and the World-Structure sliders from the generation-parameter
API + Generate stage dialogs), one row (culture-terrain-fit) re-classified
as Category 2 because its inputs need the unstarted
`_civFactionAggregates`, and `use_gpu`'s toggle stays deferred while its
status is now reported honestly; GDScript only, no Rust and no `main.tscn`
change; verified with real windowed screenshots of a real 40-settlement
world and real mouse clicks through all three menu entry points; see its own
section below — post **unified tool plan milestone C** — the Water
& ecology group: River/Lake's special commit path (`enforce_river_channels`'s
re-clamp, per-stamp `enforce_channel_descent` + `river_mask`/`river_floor`
locking, Lake's `water_only` dry run into `lake_mask`) in a new
`cartalith-engine::sculpt_commit`, and the Cartography paint brush
(`PaintStamp`/`PaintLayer`) in a new `cartalith-spatial::paint`;
golden-verified **bit-exact** over 18 cases first run; reading the reference
found the paint brush has **three** layers not one and that its override
merges only at render and export, never into analysis; tested and unwired, no
Godot file touched; see its own section below — post **DCC shell milestone
3** — the World Setup
dialog: File ▸ New world grown into a real world-setup gate with map width in
km, working resolution, extent mode and frame aspect, a live derived
grid/extent/cell-size readout, and generation dispatched through
`generate_sized()`, so maps are no longer forced square; the GUI half of
`22ae75b`, no Rust changed; see its own section below and
`DCC_SHELL_SCOPE.md` — post **unified tool plan milestone B** — the
Sculpt-editor terrain port, the plan's largest single chunk: all thirteen
`SCULPT_FEATURES` landform stamps, three noise families, the stamp
bbox/coverage/domain-warp pipeline and the eight presets, in a new
`cartalith-terrain::sculpt` implementing milestone A's `Stamp` trait;
golden-verified **bit-exact** over 23 cases against the reference's own
`sculptApplyStamp` under a Node `vm` harness — which corrects the plan's own
prediction that no golden path existed here — tested and unwired, no Godot
file touched; see its own section below — post **unified tool plan
milestone A** — the
`PassBuffer`/staleness core, `UNIFIED_TOOL_PLAN.md`'s foundation layer that
every tool milestone B-F builds on: `PassBuffer<S>`/`Stamp`/`StageGraph` in
`cartalith-spatial`, Cartalith's own stage chain in `cartalith-engine`,
tested and unwired, no tool built yet; see its own section below — post
**non-square maps** — `generate_sized()`/
`generate_world_structure_sized()` unlock the independent `gw`/`gh` the
engine always had; every golden fixture in this workspace was already
non-square, so the squareness lived only in `cartalith-godot`'s
`call_params`; `map_height_km` is derived, not settable, because cells are
square in km — see its own section below and `GENERATION_PARAMETERS.md` —
post **generation-parameter API** — every
generation parameter in `cartalith-engine`'s eight parameter structs is
now reachable from GDScript, 7 -> 58, via one flat dotted-key table
(`get_params`/`get_param_info`/`set_params`/`reset_params`) rather than
~58 individual setters; see its own section below and
`GENERATION_PARAMETERS.md` — post DCC shell milestone 1 — `DCC_SHELL_SCOPE.md`, full structural replacement of the panel-browser shell with the owner-supplied DCC editor design: menu bar/workspace tabs/tool options bar/left tool rail/viewport/right dock/status bar, every real control re-parented, tool rail present and honestly inert, one real gap found and fixed — the status bar's own tool-hint slot wasn't wired — screenshot-verified end-to-end; see its own section below), post real Android device pass — MVP criterion 4 fully closed —, sea routes (Phase 2 milestone 13) wired into `cartalith-godot`'s rendering with a real render-loop crash found and fixed along the way, CPU-multithreading milestones 2-3 — `cartalith-civ` then `cartalith-climate`/`cartalith-erosion`/`cartalith-hydrology` Rayon-parallelized — Phase 1's two closeout items (credits screen, crate license audit) both done, GPU layer integration milestones 7-8 — GPU-backed weather simulation, shared GpuContext across `generate_terrain`'s stages — a new standalone `cartalith-spatial` crate (tiling/quadtree/dirty-tracking base for a future LOD integration — real, tested, referenced by nothing yet), Phase 2 milestone 16 (`_civGenerateProvinces` — resolved the milestone-9 territory-input blocker via milestone 10's own `assign_territory`, data/backend done and verified, rendering wired as a boundary-line overlay in a same-day follow-up), and Phase 2 milestone 17 (economy/Journey Planner investigated for real — two separate large subsystems found, not one; the ~70-function Journey Planner confirmed to genuinely need its own sub-phase per `ROADMAP.md`, not attempted; `civ_resource_trade_balance` ported/tested from the smaller economy layer — **now genuinely wired**, same day: `civ_world_mean_resources`/`civ_place_resource_context` give it real per-settlement inputs, `get_trade_balances()` exposes the result to Godot, and the memory-optimization tension (needs all 15 resource keys, six were being freed early) resolved by moving that free to after settlements are placed — full reasoning in `ECONOMY_SCOPE.md`), Phase 2 milestone 18 (culture beyond naming, investigated — confirmed one real computation exists beyond the already-ported syllable tables, `_civCultureTerrainFit`/`civ_culture_terrain_fit`, ported and tested but not yet wired since its real inputs depend on the still-unstarted `_civFactionAggregates` territory aggregation; Government/Religion/Ag-tech confirmed genuinely UI-only with zero derived computation; a completely unrelated "culture profiles" system found at reference lines 28193+ correctly identified as Phase 5 Urban Morphology scope, not Phase 2), Phase 2 milestone 19 (Journey Planner milestone 1 — the two fully self-contained categories of its ~70 functions ported: physical-modeling primitives and the reference's own "four deferred items" seasonal/closure cluster, 22 tests, full remaining milestone breakdown in new `JOURNEY_PLANNER_SCOPE.md`), Phase 3 milestone 1 (`TerrainAppearance` abstraction in `render.rs` — colour data now owned/structured, pixel-identical output verified, real audit finding that no elevation-breakpoint ramp exists in this renderer), Phase 3 milestone 2 (multidirectional hillshade + ambient occlusion — the first pass where the default render visibly improves; JS golden parity kept exact via a new `js_reference()` appearance rather than re-baselining, min-luma identical before/after so no black valleys, ~free at 45 ms/512²), Phase 3 milestones 3-4 (hydrology tint; then the atlas look — paper/vellum ground, forest stippling, physical plate border — closing three of `VISION.md`'s four remaining atlas elements, with the `js_reference()` gating extended by three more early-returning zeros so `golden_parity_render.rs` stays completely unmodified, and with the cross-world result *inverting* milestones 2-3: stronger on low-relief Archipelago than on mountainous Classic, because the paper acts on the whole sheet), Phase 3 milestone 5 (geological material exposure + local contrast — the world's real rock types from `build_lithology` reach the image for the first time, both as the rock material's own colour and as bedrock showing through thin soil, which matters because Classic's land is 45% shale / 33% metamorphic / 0.4% granite and granite is what the ported heuristic painted by default; plus a band-passed local-contrast pass whose gain *falls to zero* on strong edges so §18's "no haloing" is a property of the maths — interior contrast rises in all three test worlds including a non-square one while clipping falls, and two real corrections came out of measuring and looking: raw slope is resolution-dependent, so the first geology gate silently confined itself to the steepest ~5% of land at 2048², and a plain high-pass amplified milestone 4's own paper grain into a visible quilting), and the GUI shell redesign milestone 1 (`GUI_SHELL_SCOPE.md` — full 6-region professional-editor shell rebuilt in `main.tscn`/`main.gd` from an owner-supplied design import, zero Rust changes, every real control re-parented and screenshot-verified working end-to-end, every not-yet-real feature visibly present but honestly disabled), and the causal-chain explainer (`VISION.md` sequencing item 1 — hovering a settlement shows a real "WHY HERE?" decomposition of `build_settlement_suitability`'s own thirteen weighted terms; proved faithful by a test that reconstructs the real function's output at every cell from the explanation alone, and cross-checked against real terrain across all 40 settlements of a generated world with 0 violations; deliberately per-settlement rather than a general `explain_cell(x,y)`, since the source rasters aren't retained on `CivData` and holding them would undo the memory work), and Journey Planner milestone 2 (transport mode selection — 6 of 10 originally-listed functions shipped, given caller-supplied stage lists; the other 4 confirmed by reading the real reference code to depend on milestone 5's unbuilt route derivation or milestone 3's unbuilt `jpCalcLand`, re-flagged rather than forced; the biome-mapping question this doc worried about turned out to already be answered by the reference's own `jpLegacyBiomeOf`, ported as `jp_biome_key` rather than invented; 15 new tests, `JOURNEY_PLANNER_SCOPE.md` updated), and the GUI decluttering pass (`GUI_SHELL_SCOPE.md` — a design-lead-researched target IA implemented for real: `INFRASTRUCTURE`→`EXPLORE`, `CARTOGRAPHY:Layers` consolidated into the one real `LayersPanel` surface freeing a slot for `Paint`, `WORLD:Resources`→`Sculpt`, CIVILIZATION/CARTOGRAPHY subjects renamed to the reference's real buckets, the invented `GenerateMenu` 11-stage pipeline replaced with the reference's real Step 1→2→3 sequence, a real dark `Theme` resource replacing the light-parchment `SettingsCard` panels that had been sitting on the dark shell, a real `FooterVBox`-visibility bug fixed, before/after windowed screenshots confirming the full golden path unbroken) — see `CHANGELOG.md`), and Journey Planner milestone 3 (physical travel cost — 7 functions shipped including the v1.97 sail polar, the season×biome weather blend and the whole day-wage cost model; 2 of the 11 listed had already shipped with JP milestone 2; the remaining 2 (`jp_calc_land`/`jp_calc_water`) exposed a real dependency-ordering error in `JOURNEY_PLANNER_SCOPE.md` — they need milestone 4's consumption/resupply cluster, which that doc orders *after* them — so they are deferred and the doc is corrected rather than the dependency stubbed; the flagged `JP_BIOMES[...].weather` table confirmed unported and ported here; `jp_journey_cost` confirmed to need no milestone-5 plan object; milestone 2's four deferrals re-read and none resolved; golden-verified via a bare-`vm` Node run of the reference's own source lines, 12 new tests). **Phase 4 started** (`ASSET_LIBRARY_SCOPE.md`, new): the Asset Library investigated for real against the reference rather than its pre-implementation design docs — an asset is one PNG bound to one slot in a frozen ordered vocabulary (8 families), an asset pack is a real PKZIP+`pack.json`/`pack.csv` serialization format, a second `assetlib/library.json` project-embedded format also exists, and the renderer genuinely draws pack sprites with the vector glyphs as fallback; ~2,250+ lines total but only ~600-800 of them portable, so a real sub-phase of seven milestones. Milestone 1 done: new standalone `cartalith-assets` crate (pack manifest model/parse/validate/serialize, 28 tests, golden-verified against the real `parsePackCsv`/`parsePackManifest`/`packSummary`), wired to nothing. **Milestone 2 done**: pack `.zip` read/write, placed in `cartalith-assets::archive` behind an on-by-default `zip` feature after reading `cartalith-io` and finding nothing to share (its whole zip surface is three `zip`-crate calls) plus two reasons not to put it there (reading-only by explicit scope; the dependency would point the wrong way); what is actually ported is the reference's export *policy* — `.png` STORED, timestamps frozen at 1980-01-01 so exports are byte-reproducible, `pack.json` last, names verbatim — and it is verified **in both directions** against a pack the reference's own `PackManifestBuilder.build()` + `zipStore()` produced headlessly, including feeding this port's own output back through the reference's `unzipAny`/`parsePackManifest` (identical payloads, `pack.json`, summary and warnings; the two archives differ by 2 bytes total), 14 new tests, still wired to nothing. Milestone 3 done: scatter rules (`cartalith-assets::scatter` — the `ScatterRule` model, ten slot presets, keyed rule table, weighted variant selection, hardened normalizer), with the three v1.27 hardening fixes **re-derived for Rust rather than transcribed**: the `NaN`-density carpet is still reachable here but by the *opposite* IEEE rule (`f64::min` absorbs NaN where `Math.min` propagates it), the `NaN`-spacing bucket-grid collapse is real and `f64::max` would have masked it, and the `Object.assign` aliasing bug is structurally unreachable — not from ownership but because defaults and untrusted input are different *types* here, so no defensive code was written for it; plus a guarantee the reference cannot have (`Serialize` but deliberately no `Deserialize`, so the hardening cannot be bypassed). Golden-verified: `pick_weighted_variant` diffed exactly over 11 cases × 36 positions, and 37 normalizer fixtures caught a real first-run bug — `density`'s fallback is not symmetric with the other numeric fields (absent keeps the preset, *rejected* lands on a literal 1). 24 new tests; three corrections to milestone 4 recorded (it is not the first cross-crate dependency — this is; `pickIconVariant`/`spaceOf` shipped here; `biomes` is `Vec<f64>` because `Number.isFinite` does not coerce). **Milestone 4 done**: rule-driven icon placement (`cartalith-assets::placement` — `place_map_icons_ruled`/`icon_slot_for_item`/`sprite_draw_rect`), the first real placement golden-parity surface (positional and seeded, diffs exactly); both of milestone 4's own v1.27 fixes (most-specific-first priority sort, `requireWetland` ANDed with the biome test) confirmed **structurally necessary in Rust**, unlike one of milestone 3's three, and proven with a hand-traceable `tGap=1` fixture where the winner is shown independent of rule-insertion order; 23 new tests (12 unit + 11 golden), still wired to nothing. **GPU layer integration milestone 9** (flow accumulation — the first genuinely sequential algorithm in this pipeline redesigned for GPU rather than ported: per-cell D8 flow direction plus pointer-doubling subtree sums in `ceil(log2(n))` rounds, `atomic<u32>` fixed point for order-independent bit-reproducible accumulation; bit-exact against the real `compute_flow` for area seeding and 1.3e-4/3.3e-4 relative at and above the channel threshold for discharge seeding; **measured through to the civilisation layer — river network and settlement positions both come out identical, 104/104 and 125/125 seeds, zero moved**; 15.5× on the kernel at 2048² and the end-to-end `generate_terrain` ratio moving 0.98×→1.74× there; plus two honest "shouldn't run on GPU" findings for the water-body depression fill and `road_dijkstra`), Phase 4 milestone 4 (rule-driven icon placement, `cartalith-assets::placement`, both v1.27 fixes confirmed structurally necessary in Rust, 23 new tests), and Phase 4 milestone 5 (the Library model — `AssetDB`/`AssetCollections`/`AssetValidator.run()`/the `assetlib/library.json` shape, lining up with `SAVEFILE_COMPAT.md`'s existing "nothing to deserialise into yet" note; two real corrections found by reading — per-slot display names turned out load-bearing for the validator's own warning text, and the Library's `poi` vocabulary is ten slots, not the eight `PACK_POI_SLOTS` milestone 1 ported; the id-slugging/uid-collision hardening asked for by name found and ported with tests; 56 new tests, 32 golden-verified against a real reference run), and Phase 4 milestone 6 (image handling, `cartalith-assets::raster` — the first milestone that touches pixels, narrower than its own original description once milestone 5's own corrections are read literally: `image` crate for decode/encode/resize (`png`-only, no default-features), a real `item_hash` content hash deliberately **not** byte-matched to the reference's own browser output since the hash is never serialized on either side and the reference's own canvas-resample kernel is implementation-defined, `fit_to_bottom`/`finalize_pack_texture_inv_mean` golden-verified since they touch no DOM API, `render_item` porting the reference's own single shared thumbnail/preview/bake core, and `AssetDB::apply_library_file_with_items` wiring real item restoration end to end; 15 new tests), and **Phase 4 milestone 7 — closing Phase 4 entirely**: renderer + Godot integration, `cartalith-godot::pack` (the first workspace dependent on `cartalith-assets`) — real sprite compositing (pack art via a bilinear blit, a real procedural glyph fallback for all ten icon slots) and real ground-texture splat (the six `SPLAT_PAINT_SLOTS` channels blended via `land_color`'s already-computed `materialWeights`), with the two Cartography "painted layer" biome/terrain overrides honestly left out (this port has never ported the paint-brush tool that would drive them, a named follow-up rather than a silent gap); `golden_parity_render.rs` unmodified and passing; verified with a real windowed run against the milestone-2 fixture pack, confirmed by inspecting the native pixel output (a real sprite rectangle, a real irregular splat-checkerboard region, real glyph-fallback blobs), full writeup and honesty check against `ASSET_LIBRARY_SCOPE.md` §8's own "done means" in `STATUS.md`'s own Phase 4 section.

## JS-semantics fidelity audit (`JS_SEMANTICS_AUDIT.md`, done 2026-08-18)

Not a milestone — a verification pass over all fourteen crates, and the
document it produced is meant to be read *before* the next port rather than
after a fixture disagrees.

**Done**

- [x] Swept every crate for `f64::hypot`, `f64::exp`, float `.min`/`.max`,
      `.round()`, `as u8` and float-to-int casts. 44 `hypot` sites, 23 `exp`,
      206 float `min`/`max`, 47 `round`, 26 `as u8`/cast — each with a verdict
      in §4 of the audit.
- [x] **Fixed: `PaintStamp::apply` painted rim cells the reference skips.**
      `_paintAt`'s gate is `Math.hypot(dx,dy) > R`; `f64::hypot` and V8 disagree
      on 1,398 of the 4,096 integer offsets in `[0,64)²`. Exhaustive scan of
      `R = 1..=512`: 25 radii change a cell, first at **125** — `35² + 120² =
      125²`, so V8 returns `125.00000000000001421` and skips where
      `f64::hypot` returns exactly `125.0` and paints. Not live (sliders cap at
      40 and 20) but `PaintStamp::new` takes an uncapped `f64`, and the module's
      claimed invariant was false.
- [x] **Fixed: `js_to_fixed` rounded down on roughly one value in ten.** Two
      bugs in one expression — a first dropped digit of `5` with any nonzero
      tail rounded *down* (`9.051 → 9.0`, V8 `9.1`), and a negative tie rounded
      toward zero (ECMA-262 21.1.3.3 strips the sign *before* picking "the
      larger n"). Both collapse to `round_up = first >= 5`.
- [x] Verified the two `toFixed` ports agree: 60,000 differential cases against
      V8, 0 disagreements for both.
- [x] Measured every transcendental against V8 (200,000 samples each) so the
      remaining gaps are sized rather than guessed.

**Open, in priority order**

- [ ] **Port `js_atan2`** — 22.98% divergence, the largest in the workspace,
      eight sites, nothing ported. Land it in `cartalith-urban::geom`'s FDLIBM
      block beside `js_sin`/`js_cos`/`js_log` rather than opening a seventh copy
      site, then re-verify `cartalith-hydrology::build_channels` specifically.
- [ ] **`cartalith-jsmath` leaf crate.** 7 copies of `js_hypot`, 7 of
      `js_round`, 3 of `js_min`/`js_max`, 2 of `toFixed`. Blocked on the urban
      fork, which is actively editing the file that would move. Mechanical once
      it lands; one commit, no behaviour change, every golden untouched.
- [ ] **One debug-only NaN-freedom assertion on the pipeline's output fields**,
      instead of `js_min`/`js_max` at 200 sites. Converts §4.3's
      "believed safe" list to "checked" at a single site.
- [ ] `cartalith-godot/src/render.rs:1219-1220` — jitter offsets that can go
      negative into `.round()`, the one unexamined negative `Math.round` in the
      workspace. Fork territory; reported, not touched.

## GUI feature parity (`GUI_FEATURE_PARITY_SCOPE.md`) — Category 1 closed 2026-08-18

That document's own milestone 1. Its Category-1 table is the set of things
the Rust engine really does and no GUI ever read.

| # | Item | State |
|---|---|---|
| 1 | Import asset pack | done — DCC shell m1, `File ▸ Import asset pack…` |
| 2 | Settlements table | **done** — `Simulate ▸ Statistics…`, Settlements tab; sortable, filterable, row click pins the causal chain in Properties |
| 3 | Trade balance / Economy | **done** — `Simulate ▸ Economy…`; `get_trade_balances()`'s first consumer ever |
| 4 | Province list | **done** — `Simulate ▸ Statistics…`, Provinces tab; `get_provinces()`'s first consumer ever |
| 5 | Faction culture-terrain-fit | **not done, re-classified Category 2** — needs `_civFactionAggregates`' per-faction terrain mix, which nothing computes |
| 6 | Planet g / rotation / tilt | done — generation-parameter API + Generate stage dialogs, `Generate ▸ Climate…` PLANET section |
| 7 | GPU status / toggle | readout **done** — `View ▸ Performance readout…`, six stages GPU-or-CPU each; toggle deferred, present and disabled with its reason |
| 8 | World Structure raw sliders | done — same two commits, `Generate ▸ Tectonics…` WORLD STRUCTURE section |
| 9 | Layer granularity | done — DCC shell m1, three Layers-dock toggles |
| 10 | Click-to-pin selection | done — DCC shell m1, Properties dock |

**Zero Rust changed, `main.tscn` untouched** — every `#[func]` needed already
existed. Placement follows `UI_SHELL_DESIGN.md` (menu items open dialogs;
the right dock is Layers/Properties/Sample only), not this document's own
Category-3 recommendations, which were written against the panel-browser
shell the DCC shell replaced.

**Verified**: `godot4 --headless --quit main.tscn` clean, console output
byte-identical to `HEAD`'s `main.gd`; `cargo test --workspace` green at
`HEAD` in a clean worktree (the working tree couldn't be built — a
concurrent fork is mid-commit in `cartalith-civ`/`render.rs`; nothing here
touches Rust); real windowed screenshots of a real 512×328 seed-12345 world
(40 settlements, 9 provinces) showing real rows on every tab, sorting and
filtering working, and the province-boundary overlay confirmed still
rendering after two shell rebuilds; and all three new menu items driven by
real mouse clicks rather than by calling the handlers.

**Still open in this document**: everything in Categories 2, 3 and 4. The
biggest ready-to-build item is now the **Journey Planner GUI** — its engine
closed at `7bd0680`, so `Simulate ▸ Logistics` is a GUI-only milestone.
Category 4's theme gaps (no `PopupMenu`/tooltip/scrollbar entries in
`dark_theme.tres`) are confirmed still open — visible in this pass's own
screenshots, where every top-bar dropdown renders in Godot's default grey.

## DCC shell (`DCC_SHELL_SCOPE.md`, milestone 1 done 2026-08-18) — supersedes the GUI shell below in full

Owner-supplied design import (`UI_SHELL_DESIGN.md`, `design/Cartalith DCC
Shell.dc.html`), owner's own framing: *"to be certain this, the dcc shell,
is the design that should be followed religiously and needs to fully
replace the current gui."* Full structural replacement, not an extension —
the panel-browser shell described in the "GUI shell" section immediately
below (navigator + swapping subject panel) is gone; the DCC editor described
here is what `main.tscn`/`main.gd` build today.

**Milestone 1 done**: all six regions from `UI_SHELL_DESIGN.md`'s governing
table built as real Godot Control nodes — top menu bar (program-level only,
8 menus: File/Edit/Generate/Simulate/Render/Assets/View/Help, a real content
change per the design doc, not a rename), workspace tabs (WORLD/
CIVILIZATION/INFRASTRUCTURE/CARTOGRAPHY/RENDER, restyles tab row + tool-rail
group emphasis only, never touches the viewport), tool options bar (active
tool's name + an honest "not implemented yet" hint, no fabricated live
parameters), left tool rail (16 tools across 5 groups + a disabled tool-
preferences icon, all honestly inert — no pass-buffer/commit/discard engine
exists, `UNIFIED_TOOL_PLAN.md` scopes that separately), viewport (unchanged
map rendering plus scale bar/coordinates/2D readout), right dock (Layers/
Properties/Sample — Layers now three independent toggles instead of one that
hid the whole overlay, Properties holds a click-to-pinned settlement's full
causal "why here?" chain, Sample shows live hover data), status bar (pass
state, autosave, tile cache, and — after this pass's own fix — the active
tool's name in the modifier-hints slot). Every currently-real control
re-parented with zero Rust changes: generation params moved into a "New
World" dialog off File (a DCC's own New Document convention), the four
experimental flags + villages checkbox, load-save, credits, all three
map-overlay toggles, the causal-chain settlement inspector (now click-to-pin
rather than hover-only, `GUI_FEATURE_PARITY_SCOPE.md` Category-1 item #10).
`GUI_FEATURE_PARITY_SCOPE.md` Category-1 items folded in while these
controls were already being touched: #1 (asset-pack import wired to a real
File menu item), #9 (layer-toggle granularity), #10 (click-to-pin). Left for
later per that doc: #2-5 (settlements table/economy/province list/culture-
fit, each needs its own real table UI), #6 (planet params setter), #7 (GPU
toggle/readout — the noise redesign is still `GPU_LAYER_INTEGRATION_SCOPE.md`'s
current milestone), #8 (World Structure raw sliders).

Real gap found and fixed this pass: `StatusHintLabel` had no
`unique_name_in_owner` and was never written by `main.gd`, so selecting a
tool updated the Tool Options Bar but not the status bar's own hint slot —
two chrome regions disagreeing about the same state. Fixed by wiring
`_on_tool_selected` to set it honestly. Known pre-existing cosmetic issue,
not fixed (predates this milestone, not part of this diff): unchecked
`CheckBox` nodes render with no visible glyph against `theme/dark_theme.tres`
(`checkbox_unchecked_color` is set but Godot's `CheckBox` icon theme items
are a separate mechanism this theme resource doesn't populate) — functional
regardless, confirmed by screenshot.

Verified: `cargo build -p cartalith-godot`/`cargo test --workspace` both
clean, 0 regressions. `godot4 --headless --quit main.tscn` clean load. Real
windowed-app screenshot verification end-to-end (`PrintWindow`/`mouse_event`
automation, this session's established technique): New World dialog defaults
correct, Generate produced a real 2048×2048/seed 12345/800 km/40-settlement
world with terrain/settlements/roads/sea routes rendering; Territory/
Province overlay toggles both confirmed independently of Settlements/Roads/
Sea routes; settlement hover (on-canvas card + Sample dock) and click-to-pin
(Properties dock's full causal chain, survives subsequent layer toggles)
both confirmed; File > Open project (.zip) opened the real save dialog and
cancelled cleanly; Help > Credits opened with full content; tool-rail
selection and workspace-tab switching both confirmed structurally correct
per `UI_SHELL_DESIGN.md`'s own rules. Full record: `CHANGELOG.md`'s "DCC
shell milestone 1" entry.

**Milestone 2 done 2026-08-18 — the Generate menu's real parameter dialogs.**
The GUI half of the owner's "make all generation options active" directive
(the Rust half is the section immediately below). `UI_SHELL_DESIGN.md`'s
Generate menu spec built for real: **six live stage dialogs** (Tectonics,
Volcanism, Erosion, Hydrology, Climate, Settlements) carrying **57 controls,
every one wired end to end** from widget to `WorldParams` to the generated
world; the other four stages (Glacial & coastal, Ecology, Infrastructure,
Politics) stay visibly present and disabled with tooltips naming the real
reason. Dialogs, never persistent panels, per that document's governing rule.

- **No duplicated parameter metadata.** Ranges/steps/labels/units/defaults
  are read at runtime from `get_param_info()`/`get_param_defaults()`;
  `main.gd` owns only stage grouping, Advanced membership and prose. Adding a
  parameter stays one Rust row and no GDScript change. `main.tscn` is
  untouched — the dialogs are built at runtime.
- **Five-level disclosure**: menu bar → Generate menu → stage dialog →
  a section per `params.rs` group → that section's collapsed ADVANCED fold.
  Advanced membership follows a rule, not taste: the reference buried it, or
  the reference never exposed it and this port surfaces it as a superset.
- **Real reset** at two granularities (per-stage, and Generate → *Reset all
  generation parameters* calling the engine's own `reset_params()`).
- **Six parameters proxied, not duplicated** — the four experimental flags
  and village seeding drive File > New World's existing `CheckBox` nodes
  directly, so the two surfaces cannot disagree. Two deliberately excluded
  with reasons recorded in code: `sea_level` (New World owns it) and
  `use_gpu` (waits on the GPU-safe noise redesign; `DECISIONS.md` §7c).
- **Staleness — decided, not faked.** `UI_SHELL_DESIGN.md` says each stage
  "reports staleness", but no staleness system exists
  (`UNIFIED_TOOL_PLAN.md` milestone A) and the engine is a **one-shot
  generator**, so there is no per-stage incremental recompute to be stale
  against. Therefore **no per-stage staleness indicators** — a pip would
  advertise a pipeline that does not exist. Instead: an honest
  regenerate-to-apply footer stating the whole world is regenerated, a
  status-bar note on change, and a *Generate now* button running the same
  single full pass New World's Generate runs.

Verified: `cargo build -p cartalith-godot` clean, `cargo test --workspace`
**563 tests / 83 binaries / 0 failures**, `godot4 --headless --quit main.tscn`
clean load (`58 exposed, 2 excluded, 57 rows`). Real 1920×1080 windowed-app
screenshot verification, **one parameter at a time at a fixed seed**, proving
control → engine → visibly different world across five parameters in five
different structs: `tect.plates` 14→40 (`TectonicParams`, completely
different continent structure); `climate.equator_temp`/`pole_temp` to minimum
(`ClimateInputParams`, identical coastlines, fully glaciated world);
`volc.count` 20→100 (`VolcanismParams`); `crater.count` 100→200
(`CraterParams`, clear impact craters); `river_density` ×1→×3
(`WorldParams`, dense drainage networks). *Reset this stage* confirmed
restoring exact defaults. Golden path re-verified with no regressions:
generation from both entry points, all five overlay toggles, the causal-chain
Inspector on hover **and** click-to-pin (pin surviving layer toggles),
Credits, and the Open-project dialog. Full record: `CHANGELOG.md`'s "DCC
shell milestone 2" entry.

**Milestone 3 (GUI track) done 2026-08-18 — the World Setup dialog.** Owner's
own request: *"a proper base setup menu where we can pick map size,
resolution, dimensions - basically expanded from the current html version."*
The GUI half of the non-square work `22ae75b` landed in Rust; **no Rust
changed**, the API already existed. File ▸ New world gains a first section,
`MAP SIZE, RESOLUTION & DIMENSIONS`, built at runtime, four rows in one
grammar (**label · guided preset · exact value**): Extent (Region / Whole
world), Map width km (six scale presets, Local 200 km → Planet 40 075 km,
beside the reference's own free entry), Resolution/columns (the reference's
own 512/1K/2K/4K/8K segment + free 4–8192), Aspect/rows (2:1, 16:9, the
reference's own 1.5625:1 region frame, 4:3, 1:1, 3:4, 9:16, Custom + a free
row count). Under them a **live derived readout** — Grid, Extent km × km,
Cell size, Aspect — so a choice's real consequences are legible before
generating. Generation now dispatches through `generate_sized()` /
`generate_world_structure_sized()`.

Three engine rules the design is built around, not re-derived
(`GENERATION_PARAMETERS.md`): **cells are square in km**, so map height in km
is derived (`width_km × gh / gw`) and is a readout with no setter;
**world mode is physically 2:1**, so Whole world pins the aspect, takes rows
from `reference_grid_height(gw, true)`, and disables the aspect/row controls
**with the reason in prose above them** rather than silently; **grid height
is a call argument, not a stored parameter**, since it reallocates every
field. Nothing the engine owns is copied into GDScript — both reference
`gridH` factors come from `reference_grid_height()`, extent is stored through
`set_params({"world": …})`, and the post-generation summary reads
`get_map_width_km()`/`get_map_height_km()` back rather than echoing the
request. `world` became a `PROXY_KEYS` entry onto the Extent control, so the
Generate ▸ Climate dialog and the setup dialog drive one node. Two
conditional warnings surface real constraints: 4K/8K cost, and aspect ratios
past ~16:1 being degenerate. One real bug found and fixed: `%WidthInput`'s
`max_value` was 40 000 km, silently clamping Earth's 40 075 km equator.

Verified: `cargo build -p cartalith-godot` clean, `cargo test --workspace`
**719 tests / 88 binaries / 0 failures / 0 regressions**, `godot4 --headless
--quit main.tscn` clean with warnings byte-identical to the stashed baseline.
Real 1920×1080 windowed app driven through the dialog at four shapes, each
readout matched against the engine exactly: 1024×512 @ 2000×1000 km
(Earth-like), 768×1024 @ 1500×2000 km portrait (Classic), 1024×512 @
40000×20000 km Whole world (**visible polar caps top and bottom**), 640×360 @
1200×675 km 16:9 (Archipelago). None stretched, squashed or wrongly
letterboxed; `map_overlay.gd` needed no change since its fit is already
`min(size.x/gw, size.y/gh)`. Archetype dispatch re-verified against the
`a265b2b` bug. Golden path re-verified with no regressions: both generate
entry points, all five overlay toggles, the Inspector on hover **and**
click-to-pin (through the overlay's own real hit test), all six Generate
stage dialogs, Credits.

**Milestone 2 (parallel track, no code)**: `UNIFIED_TOOL_PLAN.md` —
investigate the reference's own Sculpt editor, scope Track 2 (the tool
system itself) honestly. **The tool system itself (not yet dispatched)**:
milestoned by whatever that investigation finds.

## Non-square maps (Rust half done 2026-08-18, `GENERATION_PARAMETERS.md`)

Owner's standing complaint: *"the map is always square, but the engine
doesn't require that"*, and the target it sets up: *"a proper base setup menu
where we can pick map size, resolution, dimensions."* The Rust half. **Done.**

- **The square-ness was never in the engine.** `WorldParams` has always had
  independent `gw`/`gh`, and **every golden-parity fixture in this workspace
  is already non-square** (14x11, 16x12, 24x18, 20x14, 48x40, 10x8) — so
  terrain/climate/hydrology/erosion/civ are already JS-verified at non-square
  dimensions. `cartalith-io` save loading was already correct too (10x8 and
  12x6 in its own tests). The restriction was two lines in
  `cartalith-godot/src/lib.rs`: `call_params`'s `p.gh = gw` and `absorb`'s
  `self.gh = gw`.
- **The reference is never square either**: `gridH(gw) = round(gw * 0.5)` in
  world mode, `round(gw * 0.64)` in region mode (reference line 5049), and
  its "Working resolution" segment sets the **width** only. This port's
  square default was an artifact of a one-argument `generate()`, not a parity
  match. It stays the default anyway, because every golden fixture and every
  existing `main.gd` call rests on it.
- **API** (additive, square by default, `generate()` unchanged):
  `generate_sized(seed, width_km, grid_w, grid_h)`,
  `generate_world_structure_sized(seed, width_km, grid_w, grid_h, archetype)`,
  `reference_grid_height(grid_w, world)`, `get_map_width_km()`,
  `get_map_height_km()`. Grid height is a call argument, not a stored
  parameter — like `resolution`, it reallocates every field.
- **`map_height_km` is derived, with no setter.** Every km-to-cell conversion
  in the workspace goes through the single quotient `map_width_km / gw`
  applied isotropically (`terrain_detail_k`, `river_flow_thresh`,
  `civ_catchment_radius_cells`, `suppression_radius_cells`), so cells are
  square in km and height is `width_km * gh / gw`. Setting it independently
  would silently contradict every distance, grade and spacing in the world.
- **Rendering**: `render.rs` audited per pixel — every index carries a real
  `gh` bound and every resolution-derived radius is isotropic. One real fix:
  the plate frame's uniform cell margin could exceed half the height on a very
  wide plate and cover the whole sheet, so `border_width_cells` now caps at
  `0.25 * gh` **only when `gh < gw`** (square and tall grids byte-unchanged).
  `pack.rs` needed no change.
- **`map_overlay.gd` was already correct** — verified, not assumed, and not
  touched: `_displayed_rect()` is a real aspect-preserving fit and
  `_interior_rect`'s width-fraction inset is right for a non-square plate
  because the frame is a uniform cell count under a uniform fit scale.
- Verified: `cargo test --workspace` 0 regressions, every golden fixture
  unmodified; 7 new engine tests (256x128, 128x256, 250x150, the reference's
  own 256x164 and 256x128 world shape, 512x32, World Structure at 192x96);
  7 new `cartalith-godot` tests including a real "the picture is the right
  *shape*" check (rendered water still coincides with `field < sea_level`
  above 95%); real PNG dumps at `target/nonsquare/`; clippy clean; headless
  Godot clean load.
- **Still open**: the setup dialog itself (GUI fork's — it should call
  `generate_sized`/`generate_world_structure_sized`, with
  `reference_grid_height()` for the default shape, and follow the reference's
  width-plus-extent model rather than two free spinboxes). `cartalith-civ`
  was read but deliberately not edited (sibling fork mid-milestone); nothing
  in it needs fixing. Aspect ratios beyond roughly 16:1 are degenerate but
  non-crashing, not a design target.

## Generation parameters exposed to the GUI (done 2026-08-18, `GENERATION_PARAMETERS.md`)

Owner directive: *"make all generation options active in the current
interface so that we have the same functional controls as the older html
version."* The Rust half. **Done.**

- **7 -> 58 parameters reachable.** Before: `sea_level`, four subsystem
  flags, and the World-Structure block only as five hardcoded named presets.
  Now: every field of all eight `cartalith-engine` parameter structs
  (`TectonicParams`/`VolcanismParams`/`CraterParams`/`PlanetParams`/
  `ClimateInputParams`/`StreamParams`/`WorldStructureParams`/`WorldParams`),
  minus the three that are `generate()` arguments by design
  (seed, resolution, map width — the reference itself refuses to make map
  width editable mid-project).
- **Shape**: one flat, dotted-key namespace (`"tect.plates"`,
  `"climate.lat_n"`) mirroring the `WorldParams` field path, driven by a
  table in `cartalith-godot/src/params.rs`. `get_param_info()` carries
  group/type/default/min/max/step/label/unit/reference-control per key, so
  the GUI builds its dialogs from the engine and hardcodes no ranges. New
  `#[func]`s: `get_params`, `get_param_defaults`, `get_param_info`,
  `get_param_groups`, `set_params`, `reset_params`, `get_gpu_stages_used`,
  `get_seed`, `get_villages_enabled`, `apply_archetype`, `get_archetypes`.
  Parameters **persist between generations**; the three pre-existing setters
  are unchanged in signature and now write into the same storage.
- **Ranges are the reference's own**, converted through each control's real
  `tparam`/`cparam`/`eparam` mapping — not invented. The 11 parameters the
  reference never exposed as controls are flagged with an empty
  `reference_control`, not passed off as parity (`DECISIONS.md` §7d).
- **Invalid values**: unknown key / wrong type / NaN / ±inf are **rejected**
  and reported; out-of-range is **clamped** and reported; a fractional value
  for an int parameter is **rounded** and reported. `set_params` returns
  `{rejected, clamped}` so a dialog can re-read the stored value.
- `GUI_FEATURE_PARITY_SCOPE.md` Category-1 items **6** (planet params), **7**
  (`use_gpu` + a read-only `gpu_stages_used` readout) and **8** (raw
  World-Structure sliders, plus `apply_archetype`) are closed on the Rust
  side by this pass — items 2-5 remain (each needs its own real table UI).
- Verified: `cargo test --workspace` 0 regressions with every golden fixture
  unmodified, clippy clean, 11 new mapping tests, and a headless Godot run
  in which the sibling fork's `main.gd` reads 58 entries out of
  `get_param_info()` and places 57 rows across the Generate menu.
- **Still open**: parameters belonging to pipeline stages this port has not
  ported at all (droplet/hillslope/velocity erosion, glacial, coastal), the
  three structured-orogeny T5 knobs, and geoid/tides/seasons — itemized with
  reasons in `GENERATION_PARAMETERS.md`.

## GUI shell (`GUI_SHELL_SCOPE.md`, milestone 1 done 2026-08-17; decluttering pass done 2026-08-17) — superseded in full by the DCC shell above

Owner-supplied design import (`claude_design` MCP) redesigning the whole
Godot UI as a professional-editor shell — top bar (7 domain menus),
workspace navigator (4 subject groups), a second panel that swaps with
navigator selection, mode bar + viewport, right context inspector, bottom
timeline bar. Owner decided: target this port not the JS reference app (the
mockup's own `#id`-re-parent notes describe `Cartalith Gen1 v2.10.html`'s
DOM, a different frozen file in a different repo); build the full shell
structure now, wire only what has real engine backing, leave the rest
visibly present but honestly `disabled`.

**Milestone 1 done**: the shell exists, every real control (seed/
resolution/width/sea level/world shape/experimental flags/villages/the
three map-overlay toggles/load-save/credits) re-parented with zero
`main.gd` reference changes (Godot's `%UniqueName` lookup is
position-independent) and zero Rust changes. New: a settlement-hover
signal (`map_overlay.gd`) feeding the new Inspector panel with real data.
Screenshot-verified end-to-end: generation, all overlay toggles, navigator
swapping, settlement-hover inspector, and the credits dialog all confirmed
working through the new shell on a real Windows run. Deferred, as scoped:
light theme, panel collapse/rails, all three responsive breakpoints,
terrain appearance's actual editing GUI.

**Decluttering pass done** (design-lead-researched target IA, implemented
in full): `INFRASTRUCTURE` (zero reference grounding) → `EXPLORE` (the
reference's real second mode); `CARTOGRAPHY:Layers` nav subject removed
(consolidated into the always-visible `LayersPanel`, the one real layer
surface), freeing a slot for `Paint`; `WORLD:Resources` → `WORLD:Sculpt`;
CIVILIZATION/CARTOGRAPHY subjects renamed to the reference's real buckets;
18-of-20 placeholder subjects now carry specific, reference-grounded honest
text instead of one generic string. Top bar: invented `New world.../Save
project` deleted, `GenerateMenu`'s fabricated 11-stage pipeline replaced
with the reference's real Step 1→2→3 sequence, `SimulateMenu`/`MapMenu`/
`ViewMenu` renamed, `AssetsMenu` converted `MenuButton`→`Button`, a
`ThemeToggleButton` added (disabled — light theme itself still deferred).
Real bug fixed: `FooterVBox` was visible on all 20 nav subjects instead of
`WORLD:Overview` alone. A real dark `Theme` resource
(`theme/dark_theme.tres`) now covers every control including SpinBox/
OptionButton/CheckBox, retiring `app_theme.tres` (the MVP's light-parchment
theme) from the live path; the three light-parchment `SettingsCard` panels
sitting on the dark shell — the single most visible inconsistency in the
prior shell — are gone, flattened into plain sections with one
`FoldableContainer` for Advanced Features. `CreditsDialog` explicitly
themed (Window nodes don't inherit Control-tree themes); map-overlay hover
card recolored dark. Real before/after windowed screenshots (the *before*
shot from genuinely running the old shell via `git stash`, not memory);
full golden path — generate/overlay toggles/causal-chain hover inspector/
load-save/credits — reconfirmed working through the restructured shell.
Full record: `CHANGELOG.md`'s "GUI decluttering pass" entry,
`GUI_SHELL_SCOPE.md`'s own dedicated section.

## MVP_SCOPE.md — "done means all seven"

| # | Criterion | Status |
|---|---|---|
| 1 | Height/temp/rain/flow match golden data | **Done.** Every pipeline stage golden-verified bit-exact/tight-tolerance against the real JS engine: tectonics/orogeny (graph-driven T1-T5), volcanism+provinces, climate (temp/wind/rain), ocean currents, terrain wind deflection, erosion, hydrology, world-structure archetypes, full carve pipeline. Nothing left pinned to a stale default. The Rust side was always correct; a separate UI-only bug (fixed 2026-08-17, see `CHANGELOG.md`'s "Fix: World Shape archetype selection had no effect on generation") meant the Godot UI's World Shape dropdown never actually reached `generate_world_structure()` — that gap is now closed, real screenshot-verified. |
| — | UI/UX (not one of the seven, but part of the `/goal` "feature and graphic parity" directive) | **Reskinned 2026-08-16, then re-themed same day per explicit owner feedback.** First pass: `ui-ux-pro-max` dark-dashboard design system, grouped World Parameters/World Structure/Advanced cards, visible keyboard-focus states. Owner preferred the reference HTML's own look, so the palette was swapped to a literal port of the reference's real `:root[data-theme="light"]` parchment theme (`#efe7d6`/`#fbf5e9`/`#b07f3f` accent) — not a fresh design-system search, the actual CSS values from `Cartalith Gen1 v2.10.html` line 271. Confirmed by real-window screenshot that the map's own pixels are untouched by the theme swap (JS/Rust colour ramps, not CSS/Theme — same guarantee the reference's own code comment makes). Deferred: real Fira font files (license-unverified, kept Godot's default font). **`MVP_SCOPE.md` point 9 (sea level) done 2026-08-17**: a new `Sea level` `SpinBox` (0-100%, matching the reference's own `#seaV` slider convention) in `WORLD PARAMETERS`, wired via a new `WorldGen.set_sea_level` `#[func]`. Real screenshot-verified: seed 12345/512²/Classic at 42% vs. 15% produced dramatically different coastlines (most of the ocean became land at 15%), confirming the control has a real effect, not just a cosmetic one. Only takes effect under the Classic world shape — named archetypes re-anchor sea level from their own land-fraction target (`apply_world_structure_sea_level`), a real, documented, pre-existing interaction, not a new limitation. See `CHANGELOG.md`'s UI reskin and "real Windows hands-on verification" entries. |
| 2 | Recognisable 2D map render | **Done (2026-08-16).** Replaced the placeholder elevation-only tint with the reference's real default-settings biome/hillshade renderer (`crates/cartalith-godot/src/render.rs`, new): `materialWeights` (snow/rock/sand/wetland/canopy/grass), the six climate-selected colour ramps, multi-scale hillshade, `bioBlend` desaturation, edge haze, and `seaColorCore` (smoothed-bathymetry depth/temperature banding — confirmed this is JS's real default, not a stretch feature). Two real bugs caught by golden verification, not by read-through: a missing final `ao*vignette` multiply (~40% too bright at corners) and sea colour needing the smoothed, not raw, depth field. Golden-verified against two real `generate()` runs at `1e-4` tolerance (`golden_parity_render.rs`). Deliberately excludes every `state.viz.*`-gated stretch feature (splat texturing, geology, NPR "Painter" styles, AO/SVF/shadow, SDF tinting) — all off at JS's own defaults; that's genuine Phase 3 scope, see below. |
| 3 | Windows `.exe` builds + owner has run it | **Done (2026-08-16).** Ran the actual windowed MVP UI (not `--headless`) on this session's real Windows desktop: launched, screenshotted via `PrintWindow`, drove real synthetic mouse clicks at real screen coordinates. Confirmed generation end-to-end (real biome-coloured map, correct status label) under the new light theme. Caught two real bugs this way that no amount of code review had surfaced: the World-Structure dropdown rendered blank (malformed hand-authored `.tscn` item properties; GDScript's negative-index fallback meant it may have silently been generating with the `Rift` archetype instead of `Classic` this whole time), and the window title was still "walking skeleton". Both fixed and re-verified by the same screenshot method. See `CHANGELOG.md`'s "real Windows hands-on verification" entry. |
| 4 | Android `.apk` builds + owner has installed/run | **Fully done, re-verified 2026-08-20** (real OnePlus 6T, Android 14) — third pass, prompted by the owner asking whether the new GUI had reached the APK. **It had not**: both APKs on disk predated the three-domain DCC shell merge, the rebuilt Asset Library, Travel Library, Journey Planner work, heightmap import, metropolis/recovery, Multi-GPU and the `6a97911` launcher fix. Rebuilt and re-verified on hardware: GDExtension proven loaded (`libcartalith_godot.so` mapped `r-xp` into the live process, not inferred from timestamps), GL ES 3.2 on the Adreno 630, and the golden path driven by touch — Open project → New world (2048×1311, 2.68 M cells) → Create → world rendered (`ELDRA · 311447`). **No crash, ANR, script error or OOM kill.** **The `6a97911` GL-context/wgpu-enumeration bug does not bite on Android** — verified, not assumed (zero `wgpu` lines in logcat; the GPU path is correctly inert there). **The §13 phone layout ran on a phone for the first time and works** — see the superseded open item below. Two config defects found and fixed en route: `project.godot`'s `[display]` section had been corrupted in the working tree into a garbage key (Godot's `ConfigFile` treats `;` and not `#` as its comment character, so it parsed silently and would have reverted the app to the landscape default), and `cartalith.gdextension` pointed Android at a directory the `android-dev` profile never writes. Prior pass detail follows. **Second pass, 2026-08-18:** First closed 2026-08-17; **re-run 2026-08-18** because the GUI had been replaced twice, 57 generation controls and the New world dialog added, non-square `gw`/`gh` landed, four crates were added and terrain appearance milestones 2-5 added per-pixel work — none of it device-tested. Second pass result: the grown workspace still builds for `aarch64-linux-android` clean, the APK still exports (68 MB, debug-signed) and installs, and the **golden path runs end to end driven purely by touch** — Generate → render → Layers overlays → settlement selection with the WHY HERE explainer → tool rail → Performance readout → a Climate slider dragged by swipe. No crash, ANR, OOM kill or `FATAL` anywhere; 60 FPS held throughout (generation is on a background `Thread`). **Memory has grown materially and is measured, not guessed**: like-for-like at 512×512, peak PSS **283,326 KB → 395,756 KB (+40%)** and steady-state 271,290 → 316,200 KB (+17%); at the app's own 2048×1311 default (2.68 M cells) the phone hits **894,968 KB peak (874 MB) over ~31 s**, completing correctly. **No leak** — regenerating at 512×512 afterwards returned steady-state to 309,200 KB. **Non-square works on device**: 1:1, whole-world 2:1 (aspect correctly pinned and the control disabled), 9:16 tall portrait and 2048×1311 all generate, render and report correctly. **One new required build step**: the debug `.so` has reached 400 MB of debuginfo and must be `llvm-strip --strip-debug`ed (→ 18 MB) before export. **One honest negative, recorded not fixed**: the phone UI is structurally intact but physically unusable by finger — see the open item below and `ANDROID_BUILD_SCOPE.md` §6. Full record in `ANDROID_BUILD_SCOPE.md`. |
| 5 | Map width scales feature size | **Done** — a consequence of criterion 1's parity, verified via the world-structure archetype port. |
| 6 | Changelog entry per milestone | **Ongoing** — `CHANGELOG.md` has an entry for every milestone so far; keep this up. |
| 7 | Opens a real HTML-app `.zip`, renders it, checked against the HTML app's own output | **Done (2026-08-16).** `cartalith-io::load_save` verified bit-exact against a real export produced by running the actual, unmodified reference engine (not just its own synthetic round-trip tests): `crates/cartalith-io/tests/golden_parity_real_export.rs` against `crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`. See `CHANGELOG.md`'s "cartalith-io verified against a real HTML-app export" entry for the harness technique (including a genuine `generate()`-name-collision gotcha found along the way). |

## ROADMAP.md phases

| Phase | Status |
|---|---|
| 0 — Walking skeleton | **Done.** Triangle/button/`ping()` confirmed on Windows and Android (build+package; Android run-on-device is the one open half, see criterion 4 above). |
| 1 — Terrain MVP | **7/7, all done, plus both closeout items, 2026-08-17.** Criteria 1/2/3/5/6/7 done; criterion 4 (see its own row above) fully closed 2026-08-17 — real device build/install/launch plus a real driven golden-path generation, both confirmed. The two "easy to forget" Phase-1 closeout items `ROADMAP.md` names are now also done: a real crate license audit (`cargo license --all-features`, ~190 of ~200 workspace dependencies permissive MIT/Apache-2.0/BSD/Zlib/ISC-family; the one weak-copyleft dependency is `gdext` itself under MPL-2.0, used unmodified as this port's own Rust-Godot binding; no GPL/LGPL/AGPL anywhere) and a real, reachable credits screen (header "ⓘ" button → `CreditsDialog`, `godot-project/credits.gd`) carrying forward the reference HTML's own `#creditsModal` attribution plus this port's own license-audit findings. Screenshot-verified reachable and scrollable through both sections. See `CHANGELOG.md`'s "Phase 1 closeout" entry. |
| 2 — Civilisation layer | **Started 2026-08-16, milestones 1–15 of an unknown-but-large number done** (milestone 10, territory/border generation, has an owner decision recorded — `DECISIONS.md` §7b, cost-distance Voronoi from capitals, strength-weighted — implementation status tracked separately, not this row's concern to restate). `cartalith-civ` crate (zero `gdext` dependency), every field golden-verified against the real reference engine. **1** lithology/soil fertility/water access. **2** water-body classification (ocean/lake, priority-flood depression fill). **3** biome classification (12 climate categories). **4** carrying capacity/NPP/population density. **5** resource potentials (15 geological fields). **6** route corridors/landmass quality/coast SDF. **7** `buildSettlementSuitability`/`findSettlementSeeds` — the "v1.30 one function" `ROADMAP.md` originally named as this phase's landmark, reached and golden-verified. **8** settlement placement + faction assignment — the pure core of `_civIterativeAutoWorld` (land-component labelling, snap-to-land/coast, `_civAssignLandmassFactions`'s capacity-weighted seat apportionment + multi-capital spacing, settlement tier classification), stopping deliberately before the DOM-coupled orchestration shell. **9** settlement population + naming — `_civBasePopForKind`/`_civSettleName` (RNG-driven, reuses `cartalith-rng`'s already-verified `mulberry32` — `_civRng` is the same algorithm under a different seed wrapper, proved by hand not assumed). A genuine, verified reference quirk found here: `state.seed` (distinct from the real per-world `state.tect.seed`) is never assigned anywhere in the reference, so the civ-naming RNG stream is seeded identically for every world regardless of its actual seed — same-rank, same-faction settlements across *different* worlds get identical generated names, a real mechanical consequence, not a bug. Full history, every real bug/gap found (a Node-harness seeding bug, a stale-vs-fresh river-network mismatch, a threshold ambiguity between two real reference call sites, several `WorldState`-retention fixes, a 4-vs-8-connectivity flood-fill distinction, a 4-script-block harness miscount, a snapped-position-vs-original-seed-score `.suit` mixup), and reasoning is in `CHANGELOG.md`'s "Phase 2 milestone 1–9" entries — this row stays a summary, not a repeat of it. **11** road network algorithm — `buildTravelCost`/`roadDijkstra`/`buildRoadNetwork` (a distinct `f64`-priority heap from milestone 2's, per the reference's own v1.89 perf comment; real terrain data exercised the "unreachable landmass" branch, not just a synthetic test). Landed in `cartalith-civ` (a deliberate placement decision, not a default — the functions live in the reference's block 1, weighed against `ARCHITECTURE.md`'s "civ" framing and decided the latter wins). **Investigated for milestone 12, found a real correction to the earlier assumption**: this port's own `_civSeedVillages` dependency reasoning (milestones 8/9's "villages need roads" note) pointed at the wrong system — `buildRoadNetwork` only ever serves the *manual* "Generate Roads" tool (`buildRoadsOp`, reads user-clicked `state.places`); the civ auto-populate flow's own road network (`civWays`) is built by a separate, larger algorithm (`_civHierarchicalNetwork` + `_civMstRoutes` + `_civPreferSeaRoutes`) not yet read or ported. **Milestone 11 does not unblock village seeding** — it's real, useful, tested code for a different (manual-tool) purpose. **12** the real civ-auto-populate road network — `civ_hierarchical_network_topology` (`_civHierarchicalNetwork`'s three real passes: Prim MST, min-degree-fill by settlement tier, Floyd-Warshall shortcut-detour-relief — confirmed a third pass beyond what milestone 11's own scoping estimated). This is the real `_civSeedVillages` dependency milestones 8/9/11 all pointed at without reaching. Split deliberately: ships the raw topology (what road-proximity queries actually need), defers corridor-consolidation/Catmull-Rom-smoothing/road-classification (needs `_civSmoothPath`, not yet ported — milestone 14) since that's presentation polish, not the graph structure itself. Both golden fixtures are real edge cases, not synthetic: one settlement genuinely unreachable in case0; case1's min-degree-fill hitting its natural ceiling (a complete K5 graph) rather than its per-tier target. A real `river_flow_thresh` parameter bug (hardcoded map width instead of the real per-world value) caught and fixed before it shipped. **10** territory assignment — cost-distance Voronoi from capitals, weighted by capital population (`DECISIONS.md` §7b's own design, implemented as designed). The first Phase 2 milestone with no JS reference to port at all — the reference has zero algorithmic territory generation (paint tool + save/load only) — so verification is 8 unit tests standing in for a golden test (equal-population capitals split at the geometric midpoint; a 100k-vs-5k-population pair moves that midpoint to the larger capital, the actual weighting behaviour measured, not just present; unreachable cells stay unowned; a two-capital faction's territory unions both zones). `pop_ref=15000.0` (== a capital's base population before variance) is a documented, non-arbitrary constant. **Rendered 2026-08-16**: `cartalith-godot`'s `build_territory_texture()` turns the per-cell `Vec<i32>` into a low-alpha (`~0.32`) RGBA8 overlay texture, Okabe-Ito-coloured by faction, toggleable via a new default-OFF "Show territory" checkbox — see this row's own catch-up note below and `CHANGELOG.md`'s "UI/UX catch-up: territory + villages" entry. **15** village seeding — `_civSeedVillages`/`_civVillageAcceptProb`/a milestone-12-topology-adapted `_civRoadProximityQuery`, the feature milestones 8/9/11/12 were all working toward unblocking. Closed a real RNG-sharing gap first: `name_and_populate_settlements_with_rng` now threads an external `Mulberry32` (purely additive alongside the existing zero-arg function) so village seeding continues the exact same stream naming left off at, matching the reference's own one-shared-`rng`-closure design. Golden-verified against the real reference (fully synthetic but reference-function-verified inputs, same standard milestone 12 already set) — bit-exact first attempt, including RNG-derived village names and nearest-capital faction inheritance; a second targeted extraction independently confirmed the downsampled-routing-grid-to-full-grid coordinate conversion by matching a hand-calculated `exp()` distance formula to 15 significant figures. **Flagged, not fixed here**: milestones 7-9's existing golden tests seed their candidate lists at a threshold (`0.65`) that traces to a *different* real call site (the standalone JSON-export default) than `_civIterativeAutoWorld`'s own real default (`SETTLE_SEED_THRESH=0.42`, confirmed by tracing why a headless harness's `wantCounts` is always falsy) — not a bug in those milestones' own pure-function correctness, but a pipeline-orchestration question for whatever in `cartalith-godot` builds the real base-settlement candidate list. **Wired 2026-08-16**: `cartalith-godot`'s `compute_civilisation()` now calls `civ_seed_villages` after base settlement naming, sharing the one `Mulberry32` stream this milestone's own doc comment requires (no second, desynced RNG instance), and merges the output `Hamlet`-tier settlements into the same list the UI already draws. The gating question is resolved the way flagged here: a new default-OFF `VillagesCheck` toggle in `cartalith-godot`, matching the reference's own real `_civVillages` default. **14** corridor consolidation + path smoothing — `civ_consolidate_and_smooth_ways`, milestone 12's own deferred tail (reference `_civHierarchicalNetwork` lines ~21670-21739): claims corridor cells busiest-edge-first so shared trunk segments render once, classifies each way by peak usage (`highway`/`regional`/`road`/`track`), auto-names it from its endpoint settlements, and Catmull-Rom-smooths the result (RDP-simplify then chord-length-parameterized spline sampling, both ported fresh from the reference's own `rdpSimplify`/`catmullRomSample`, reference lines 8701/8790) with a terrain-validity repair pass and an endpoint-snap pass so strokes land on their settlement pins. Also ports `_civSmoothPath`/`_civTerrainValidTest`/`_civNearestValidPt` (reference lines 21892/21843/21872), narrowed here to the one call shape this network uses (land-only validity) — the general terrain-validity test also has an ocean-only mode, generalized in by milestone 13's sea routes (this row's own **13** entry above). Golden-verified against two real cases reusing milestone 12's and milestone 9's own already-verified fixtures (no new settlement/topology data invented): a genuine short-segment Catmull-Rom oversampling quirk (a 2-cell path produces a 3-point output whose rounded midpoint coincides with its own start point) and a real K5 corridor-sharing case (10 edges, a mix of visible and fully-consolidated hidden ways). **13** sea routes — `civ_sea_routes` (`_civMstRoutes(ports,true)`, reference line 21240, `isSea` branch only — the `isSea=false` land branch has no confirmed real caller, `_civHierarchicalNetwork`/milestone 12 is what the real land network uses). Shares `_civSmoothPath`/Dijkstra/Prim's-MST shape with milestone 12 but is a genuinely separately-scoped algorithm: the cost grid marks land `Infinity` (impassable, not merely expensive — the reference's own fix-history comment explains a finite land cost let Dijkstra cut across jagged coastline pixels, and smoothing then exaggerated those cuts into visible loops), ports snap to the nearest navigable-ocean cell at radius 10 (deliberately wider than milestone 12/14's radius 6 on a different cost grid), and a v0.73 sea-lane augmentation pass adds each port's nearest reachable port as a direct lane (capped at 1.15x the MST's own longest hop) beyond the bare tree. `_civSeaTimeEdgeCost` (v1.98 current/wind-costed sea-lane pricing) deliberately not ported — its real inputs (ocean-current/wind u/v fields) aren't retained on `WorldState` past their internal use in `apply_ocean_currents`/`deflect_flow`, so this port takes the reference's own documented graceful-degradation fallback (uniform arithmetic cost) rather than adding new plumbing outside this milestone's scope — a real, flagged follow-up. Four existing helpers generalized (not duplicated) to support both land and ocean validity modes: `civ_snap_finite` (added a `max_r` parameter), `civ_is_valid_land`→`civ_is_valid_terrain` (added the `_civTerrainValidTest('ocean')` branch this row's milestone-14 note flagged as unported), `civ_nearest_valid_pt`/`civ_smooth_path` (both threaded the same `is_sea` flag through). Golden-verified against the real reference: a fresh Node harness caught and fixed a real bug in itself before trusting extraction (`generate()` is `async`, and a bare unawaited call left `field` at its default-zero fill, `currentWaterBodies()` reporting 100% ocean — fixed by awaiting properly, then cross-checked `field[0]` plus land/ocean/lake cell counts against already-trusted fixtures). Reused milestone 14's own case0/case1 fixtures (already-verified coastal settlements over genuine mixed land/ocean/lake geography at both grids) — both cases matched the Rust port's output exactly on the first run, including a real reference quirk where two of case1's four routes carry `km:0` despite having real points (`_civSmoothPath` accumulates `km` over rounded sample points before its own final step restores full-precision endpoints, so a short diagonal hop's only interior sample can round to coincide with the pre-restore rounded start point). **17** economy/Journey Planner investigated for real (2026-08-17), full reasoning in `ECONOMY_SCOPE.md` — two separate, both genuinely large subsystems turned out to exist under "economy": the Journey Planner (`jp*`/`_jp*`, reference lines ~17300-20400, ~70 functions covering transport-mode selection, physical travel cost, consumption/resupply, seasonal closures, multi-stage route derivation) confirms `ROADMAP.md`'s own "consider it a sub-phase" warning as accurate, comparable in size to this port's entire civ-layer effort to date — not attempted. The faction/settlement economy layer (`_civFactionAggregates`, ~165 lines; `_civPlaceTrade` and its dependency cluster) is smaller but still real, explicitly "NOT new simulation" per the reference's own header comment (a display/aggregation layer over already-computed state). `civ_resource_trade_balance` (`_civResourceTradeBalance`, reference line 24175, v1.33's unification of two drifted copies) ported and tested — the one fully self-contained piece, operating on caller-supplied catchment/world resource means with no new upstream dependency. Seven real unit tests (no golden harness needed — small, pure, branch-complete, no RNG/iteration-order risk, same precedent as territory/provinces). A real, disclosed tension found and left unresolved: the full trade layer needs all 15 `CIV_RESOURCE_KEYS` resident, but the memory-optimization pass (commit `62b9b51`) frees 6 of them after use — flagged for whoever ports the next slice. Not wired anywhere yet — no real caller exists until the broader orchestration is built, the same "don't wire in what nothing calls" discipline milestone 9's own territory note established. **Journey Planner sub-phase** (`JOURNEY_PLANNER_SCOPE.md`, the ~70-function subsystem milestone 17 confirmed genuinely needs its own sub-phase): **all six milestones done, 2026-08-18** (JP-3's own two deferrals closed by JP-4; JP-2's last two by JP-6's pass), nothing wired to any caller by design — it is real interactive per-journey tooling, a future GUI feature, not something auto-computed for every settlement pair. **JP-1** physical-modeling primitives plus the reference's own "four deferred items" seasonal/closure cluster (22 tests). **JP-2** transport mode selection — 6 of 10 listed functions shipped given caller-supplied stage lists, the other 4 confirmed by reading the real code to depend on unbuilt milestones; the biome-mapping question that doc worried about turned out already answered by the reference's own `jpLegacyBiomeOf`, ported as `jp_biome_key` rather than invented (15 tests). **JP-3** physical travel cost — 7 shipped (`jp_train_pace`, `jp_sail_factor` (v1.97's rig-class sail polar), `jp_wx_weighted`/`jp_weather_factor` (season×biome weather blend), `jp_column_length_km`/`jp_column_factor` (v1.51's road-capacity damping), `jp_journey_cost` (the whole day-wage cost model)), 2 of the 11 listed had already shipped with JP-2, and **the last 2 exposed a real ordering error in the scope doc itself**: `jp_calc_land`/`jp_calc_water` depend on milestone *4*'s consumption/resupply cluster (`jpCapacity`/`jpForaging`/`jpAssessResupply`/`_jpDesertTierForGap`), which that doc orders *after* them — so JP-4 must land first, and the doc is corrected rather than the dependency stubbed. Three flagged questions all answered by checking rather than assuming: `JP_BIOMES[...].weather` was indeed unported (JP-2 had deliberately narrowed its `JP_BIOMES` port) and is ported here; `jp_journey_cost` needs no milestone-5 plan object and is ported; JP-2's four deferrals were re-read and **none** resolved. First JP milestone to use a real golden harness rather than pure unit tests — the weather blend is a 48-cell five-term float sum where hand arithmetic would be the weak link, so the reference's own source lines were sliced out and run in a bare Node `vm.runInContext` with no DOM, and all 48 `jpWxWeighted` biome×season cells are verified as a block (12 tests). **JP-4** consumption/resupply, **built out of numbered order on purpose** (JP-3's finding above; the scope doc now carries an explicit build-order table at the head of its milestone breakdown, and the historical numbers are deliberately not renumbered). All thirteen listed functions shipped — `jp_human_water_rate`/`jp_human_water_carry_days`/`jp_animal_water_carry_days`/`jp_desert_tier_for_gap` (the real quick wins), `jp_consumption_factors`, `jp_capacity` (the whole seasonal-physiology/desert-multiplier/phantom-draft/saddlebag-credit mass model), `jp_foraging`, `jp_assess_resupply`, `jp_world_mean_richness`, `jp_wildlife_forage_mod`, `jp_resupply_reach`, `jp_drinking_coarse_ease`, `jp_stage_dry_km` — plus the four things the doc assigns here rather than to their own milestones: **JP-3's `jp_calc_land`/`jp_calc_water`** (so JP-3 is now fully complete), **JP-6's `jp_fmt_kg`** (both calculators format their blocked-message text with it), **JP-2's `_jpBestLandTransportForStage`** (checked against the real code rather than assumed — its `eff` parameter is only ever a plan, so `jp_calc_land` landing was genuinely all it needed; JP-2's other three deferrals remain blocked on JP-5), and the `JP_BIOMES` columns JP-2 and JP-3 each left out plus the four seasonal tables. **The one genuinely hard piece resolved by investigation, not transcription**: `jp_foraging` reads the world's wildlife *richness* through `_jpWildlifeForageMod`. Checked against this port's own Phase 2 ecology work rather than assumed — `build_npp`/`build_carrying_capacity` are real but are *inputs* to it, not the same quantity; `richness` is a per-ecoregion **species count** from an unported ecoregion-segmentation + species-roster subsystem that is on no JP milestone and is larger than this one. So it is caller-supplied (`jp_wildlife_forage_mod(region_richness, world_mean)`, `JpStage::wildlife_forage_mod` replacing the reference's `mx`/`my`), the same shape as `civ_resource_trade_balance`'s caller-supplied means — and the reference's own calibration anchor is preserved exactly: 1.0 means "no wildlife data", which is also what an exactly-average region gives, so a port with no ecoregion model behaves identically to the reference on a world whose wildlife layer was never built. Golden-verified via the same bare-`vm` Node harness JP-3 introduced, extended to the whole 17297-19252 span: every expected value in the 26 new tests is the reference's own output, including all eleven `jpCalcLand` and seven `jpCalcWater` cases with their exact verdict and blocked-message strings (165 lib tests total, 0 workspace regressions). **JP-5** route/stage derivation — the orchestration layer, and this doc's own “almost certainly the largest single milestone in this whole plan”: it did not survive as one flat pass and is recorded as the three sub-milestones the real code falls into (**5a** world sampling — `jp_road_cells`/`civ_walk_way_cells`, `jp_infra_context`, `jp_claimed_at`, `jp_stage_infra`, `jp_river_condition`, `jp_sea_condition`, `jp_coarse_idx`, `jp_stop_key`, `jp_mode_for_route`, `civ_transshipments`/`civ_transfer_overhead`, `civ_passed_settlements`; **5b** `jp_derive_stages`/`JpDerivedStage` plus the `JpWorld` borrowed context that replaces the reference's dozen globals; **5c** `jp_plan`/`JpJourneyPlan`, `jp_effective_stage_plan`/`JpStageOverride`, `jp_ensure_plan`, the v1.52 season-drift pre-pass, the per-stage vessel fallback, the supply forecast, the daily timeline and the roll-up), all three shipped in one pass. **The biggest finding is a gap this port had never noticed**: `_jpDeriveStages` samples `currentCartBiome()` *and* `currentCartTerrain()` on every route point and **neither Cartalith paint layer existed here at all** — the existing `build_biome_raster` is the *climate* raster, a different vocabulary `cartalith-assets` already documents as distinct — so `build_cart_biome`/`build_cart_terrain`/`CART_BIOMES`/`CART_TERRAINS`/`jp_legacy_biome_of` are ported here, with the one ordering detail that would have silently mis-mapped every biome checked rather than assumed (`ELEV_TO_CART` is indexed by `BIOME_INDEX`, whose order puts shrub before savanna — exactly this port's own `BIOME_*` numbering). Three more helpers on no milestone list came with it (`_civTransshipments`/`_civTransferOverhead` as predicted, `_civWalkWayCells`, `_civPassedSettlements`). Three listed functions are deliberately **not** Rust functions, with the reason recorded rather than left as a silent omission: `_jp_layovers` is a JS lazy-init idiom (a `HashMap` needs none), `_jp_settlements` is a runtime kind filter over one untyped array this port does not have (its settlements are already typed, so building the `JpPlace` list *is* the filter), and **`_jp_reroute_for_mode` is genuinely blocked** — its whole body is `_civDijkstraPath`, the interactive Route tool's unported multi-modal pathfinder, on no milestone here and a UI action besides; its pure half `jp_mode_for_route` is ported. **The `JpStage` question the scope doc wrote down in advance resolved with no change to `JpStage`**: `JpDerivedStage` carries the reference's `mx`/`my` because they are a genuine map measurement, `JpStage` correctly carries only the finished wildlife multiplier, and `to_stage(wildlife_forage_mod)` bridges — `jp_plan` takes a `&dyn Fn(f64,f64)->f64` in exactly the reference's `_jpWildlifeForageMod(mx,my)` position. `jp_auto_pick_vessel` (JP-2's) shipped here because `_jpEnsurePlan` cannot exist without it; JP-2's last two (`jp_auto_pick_transport`, `_jp_best_package_for_stage`) are now genuinely unblocked, re-read rather than assumed, and left to JP-2's own remainder. Two reference quirks reproduced as written and recorded so nobody “fixes” them (`||12000` vs `||800` map-width fallbacks two functions apart; `_jpRoadCells`' unreachable non-integral string keys). **Golden-verified** across eight reference slices in a bare-`vm` Node run, with milestone 4's block-comment balance assertion applied per slice — it caught **three** genuine boundary errors and the JS parser caught a fourth — over a synthetic but *exactly* reproducible world (closed forms in `+ - * /` only, so the Rust test rebuilds the identical `f32` grids and only the outputs are embedded): 24x16, ocean margin, lake, mountain ridge, river column, highway, road spur, claimed territory, five settlements, a 24-point route deriving into seven stages, one transshipment, a 41-day timeline and a genuinely unmet resupply requirement. 19 new tests (184 lib tests total), no new clippy warnings, 0 workspace regressions, still wired to nothing. **JP-6** verdict/reporting plus **JP-2's remainder**, closing the subsystem: `jp_verdict` (v1.49's five-band interpretive read of a finished plan, every contributing signal returned by name), `jp_confidence` (the deliberately asymmetric honesty band on the day count — the reference's own point is that the per-stage model is a best case and its optimism grows with duration), `jp_pack_range` (the wagon-equation ceiling, sharing one source of truth with the auto-picker's own divergence guard), `jp_fmt_days`, `jp_risk` (the campaign-duration advisory JP-5 correctly left here as a verdict string), `jp_auto_pick_transport`/`JpAutoTransport` (the whole route's transport/animal/vehicle mix, v1.48's analytically-detected `fodderInfeasible` divergence and the Walking→Baggage Train auto-promote included — the one missing `_jpEnsurePlan` default, `JpPlan::auto_promote`, added with it) and `jp_best_package_for_stage`/`JpPackageFix` (v1.66's per-stage species+vehicle suggestion, same “measure, never silently apply” contract as `jp_best_land_transport_for_stage`). Both reference functions' HTML hint strings are deliberately not ported — presentation is Godot's, and every value they print is a field on the structured returns. **A real bug in a shared helper, found by this pass's own golden run**: `js_fixed` (JP-4's reproduction of JS `toFixed`'s round-half-away-from-zero tie-break) decided the tie by scaling, which *fabricates* ties — `61.5/30` is `2.0499999999999998`, which JS renders `"2.0"`, but ×10 rounds to exactly `20.5` in `f64` and the `+0.5` then carried it to `"2.1"`. Rewritten to decide on the value's exact decimal expansion, and verified against `toFixed` on 30 cases including the pairs that look identical and are not (`1.25` is a real tie, `2.05` is not); no existing test's expected value changed. The harness reused JP-5's fixture unchanged and reproduced its numbers exactly; all eight slices passed the block-comment balance assertion first time, but it surfaced an error of a class that assertion cannot catch — JP-5's `2641-2675` slice starts one line *below* `TERRAIN_DETAIL_MAX_K`, and `_jpDeriveStages` swallows its own exceptions, so the whole world silently derived to zero stages with no error anywhere (found by instrumenting that `catch`; the slice is now `2640-2675`). 10 new tests (194 lib tests total), no new clippy warnings, 0 workspace regressions. **`_jp_reroute_for_mode` is the one unported function and stays that way**, the finding re-checked rather than inherited: its whole body is the interactive Route tool's unported multi-modal pathfinder (`_civDijkstraPath`/`_civWaterCostGrid`/`_civMixedCostGrid`), on no milestone in that doc, and a UI action besides. **The Journey Planner engine is therefore complete**; what remains is the interactive GUI that would give a player somewhere to enter a journey — see `JOURNEY_PLANNER_SCOPE.md`'s closing status. **Milestone 20 (2026-08-18) closes the economy layer's last unstarted piece**: `_civFactionAggregates` — per-faction population, territory km², food capacity/surplus, trade volume, tax, 15-key resource means, six-way sector output, the five-axis heuristic "power" composite (ported verbatim, not simplified), and v1.55's "Territory Fit" terrain mix — plus `_civFactionCapital`, `CIV_TAX_RATE`/`CIV_PRIMARY_SPECIALISATION` and `_civOceanDistField`. Taken because it was a real blocker: the GUI parity audit had re-classified `civ_culture_terrain_fit` as unexposable for want of exactly these two maps, and the milestone's own golden test now calls it straight off them for seven cultures × seven factions in two fixtures. `CIV_MAX_TIER_RANK` is 5, not 4 (the reference normalises by its full ten-entry class table, whose `metropolis` tier this port does not model — using 4 would have inflated the military and political axes by 25%). The four per-place fields this port has no producer for (`tradeVolume`/`economicImportance`/`specialisation`/`_umInferWalls`) are caller-supplied with the reference's own absent-field defaults, and the golden harness feeds the reference's real `_umInferWalls` verdicts back in so `fortifiedFraction` and the military axis are genuinely tested. The resource-residency tension `ECONOMY_SCOPE.md` expected does not bind — the Territory-Fit half needs no resource field, `resources` is an `Option` porting the reference's own nullable `pots`, and `compute_civilisation()`'s six-field free stays exactly where the memory-optimization pass put it. One real JS-semantics trap found by re-reading: **`NaN` is falsy in JS**, so the reference's `p.pop||0` absorbs a bad settlement at the place rather than turning a faction's whole row into `NaN`s — ported as `js_num_or_zero`/`js_truthy_num`. Golden-verified over two fixtures shaped to reach the edges (empty faction, territory-without-settlements faction, single-settlement faction, zero-population settlement, unmapped specialisation, out-of-range faction id, seam-spanning territory and settlements); six input hashes exact, and a disclosed pre-existing 1–3 f32 ULP climate divergence handled with stated tolerances rather than papered over; **58 mutations, 56 killed, 2 equivalent-mutant survivors** — both re-proved genuinely tested with discriminating variants rather than accepted on assertion, and the first pass's other four survivors were real fixture gaps (a saturating power normaliser, the territory guard's untested upper bound, `Math.round`'s negative half, and an elevation-denominator floor no real sea level activates), each closed with a unit test and re-killed. Tested and unwired — no `#[func]`, no Godot file touched (UI hold).

**Reached**: settlements with real names/populations/faction ownership, faction territory ownership per cell (wired and rendered), the real auto-populate road topology (12) consolidated, classified, and Catmull-Rom-smoothed (14) — **now wired into `cartalith-godot`'s `compute_civilisation()` and rendered as the map's actual road layer**, replacing both milestone 11's manual-tool stand-in and milestone 12's raw unsmoothed topology (fixed same-day as a third UI/UX catch-up pass — see below), and village seeding (15, wired and rendered, and now reading the real milestone-12 network for its own road-proximity check too, not the old stand-in), plus sea routes (13, `civ_sea_routes`, golden-verified, and now wired into `cartalith-godot`'s rendering too — dashed-style, distinct from land roads, see this row's own **13** entry above and `CHANGELOG.md`'s "Wire sea routes" entry). plus provinces (16, `civ_generate_provinces`, resolved a blocker recorded since milestone 9 once milestone 10's own `assign_territory` turned out to produce `civTerritory`'s exact needed shape — data wired into `cartalith-godot`, `get_provinces()`/`build_province_boundary_texture()` real and verified against live generated data, **and now rendered too** — a `ProvinceBoundaryView` overlay + `ProvinceLayerCheck` toggle, thin boundary lines layered on top of territory's own fill, sidestepping the unbounded-province-count palette problem entirely since a boundary line needs no palette; see `CHANGELOG.md`'s "UI/UX catch-up: render province boundaries" entry, including the direct headless pixel-count verification used after a static screenshot proved inconclusive for a 1px line). **Not reached**: culture (beyond naming flavour), economy, and the Journey Planner as a usable whole — its milestones 1-3 of 6 are ported and tested but deliberately unwired, and milestones 4-6 (consumption/resupply, route/stage derivation, verdict/reporting) are untouched. See `PHASE2_SCOPE.md` for the living milestone list. **UI/UX caught up 2026-08-16** (owner request: "with every milestone and phase the GUI and UX should be updated as well... use a separate agent", a continuous per-milestone practice) — **first pass**: settlements + the milestone-11 road network render on the map. **Second pass**: territory (10) and villages (15) wired and rendered (low-alpha faction-colour territory overlay, default OFF; villages merged into the settlement marker list, default OFF). **Third pass, same day**: found `compute_civilisation()` was still building its road data from milestone 11's manual-tool stand-in — not even milestone 12's own raw topology, a deeper gap than "just wire in milestone 14's smoothing." Fixed the real chain (`civ_hierarchical_network_topology` → `civ_consolidate_and_smooth_ways`, reordered so the smoothing/naming step runs *after* settlement naming, since it needs named endpoints); `get_roads()` now returns classified `Way` data (`points`/`brks`/`way_type`/`name`) instead of raw cell-index paths, `map_overlay.gd` gained a distinct continuous-coordinate `_point_to_screen` (settlement markers still use the cell-centering `_cell_to_screen` — using the wrong one for roads would have shifted every line half a cell) and break-aware polyline drawing so real internal gaps in a consolidated way don't render as a phantom straight line across them. Road width now varies by classification. Screenshot-verified: roads changed from straight/jagged MST approximations to visibly smooth curves following terrain. See `CHANGELOG.md`'s "UI/UX catch-up: wire milestone 14's smoothed roads into the map" entry. **Fourth pass, 2026-08-17**: sea routes (13) wired end-to-end — `CivData.sea_routes`, `get_sea_routes()`, `map_overlay.gd`'s dashed navy-underlay/light-dash rendering (reference's own line-~15511 convention). Real screenshot verification caught a genuine crash (an infinite-loop/buffer-overflow bug in the dashed-line draw routine, triggered by float drift over a long route) before it could ship — fixed and re-verified against the exact config that crashed. See `CHANGELOG.md`'s "Wire sea routes" entry. |
| 3 — Rendering and 3D | **Started 2026-08-17, milestones 1-6 done** (`TERRAIN_APPEARANCE_SCOPE.md`, owner-supplied `TERRAIN_APPEARANCE_RESEARCH.md`). **Milestone 6 (the GPU question answered by measurement, plus §29 quality tiers, 2026-08-18)**: research §21 was investigated for real and the answer changed what got built. GPU compute *is* reachable — not through Godot's renderer (`gl_compatibility` still cannot dispatch `RenderingDevice` compute) but through the standalone `wgpu` instance `cartalith-gpu` already owns, measured at 2.8 ms against 36.8 ms of single-thread CPU for a 2048² noise kernel — **but the renderer was not GPU-bound, it was single-core-bound**: `build_color_texture`'s per-pixel loop had grown to ~1 s at 2048² on one thread, the last O(gw·gh) serial loop in the workspace, while every engine crate feeding it has been Rayon-parallel since `CPU_MULTITHREADING_SCOPE.md` milestones 2-3. So this milestone parallelized the appearance pass (`cell_color` 1040→125 ms Classic 2048², 8.3×; real-app `build_color_texture` at 2048×1311 955→293 ms, 3.3×, measured as a true one-binary A/B via `RAYON_NUM_THREADS=1`) and **did not start a WGSL port** — appearance is now 5% of a generate+render, down from 15%, so a full port of `material_weights`/25 palettes/ten `vnoise` sites in `f32` would buy ~5% at the cost of a second renderer diverging from the golden-verified one under `DECISIONS.md` §7c. Bit-identical, proven three ways including all 48 A/B dumps re-diffed byte-for-byte. §29 tiers (`QualityTier` Performance/Balanced/Quality/Ultra, surfaced as four `#[func]`s on `WorldGen`) were designed from a new `cost_table` measurement that **contradicts §29's own recipe**: local contrast costs 30-53 ms and the paper's four `vnoise` calls ~6-18 ms, while AO, the hydrology tint and dropping five of six light directions all sit at or below the measurement noise floor — so the cheap tier keeps the relief and the AO §29 tells you to drop, and gives up texture and the second pass instead. `Quality` is `TerrainAppearance::default()` returned unchanged, byte-identical to milestone 5's look. Ladder cost at 2048²: Classic 74/101/162/163 ms (Performance 2.2-3.3× cheaper than Quality; Ultra costs the same as Quality, which is why the recommendation function never proposes it). **Policy stayed with the owner** — `WorldGen` still starts at `Quality` on every device and `get_recommended_quality_tier()` only offers one. `golden_parity_render.rs` still completely unmodified at its original `1e-4` tolerance, six milestones in. One pre-existing artifact found by looking and deliberately not fixed: rectangular blockiness in the open ocean from `seaColorCore`'s own `n_low` value-noise lattice, present in the `js_reference` dump too and more visible there. **Milestone 5 (geological material exposure + local contrast, 2026-08-18)**: research §12 and §18, the two every previous milestone explicitly deferred, picked because together they answer §30 from opposite directions — §12 puts *more real information* into the image, §18 makes information already there easier to separate. **The §12 plumbing question was checked before committing**: the brief's suggested source, Journey Planner milestone 5's `build_cart_terrain`/`CART_TERRAINS` (`dca5954`), turned out to be the wrong one — a party-movement *surface* vocabulary derived from field/water/temp/rain, i.e. from inputs `render.rs` already reads, so a coarse re-classification rather than new physical information. The right source is `cartalith_civ::build_lithology`: seven `LITH_KEYS` rock types built from the **tectonic substrate** (`age_field`/`volcanic_field`/`crust_field`/`resistance_field`), which the renderer genuinely could not derive — and no new cross-crate wiring, since `lib.rs` already calls that exact function for the soil chain. It matters more than it sounds: over Classic's land the vocabulary is **shale 45%, metamorphic 33%, basalt 11%, sandstone 7%, limestone 4%, granite 0.4%**, and granite is what the ported climate heuristic paints by default — the map had been showing one rock for a world that has seven. Built as two halves, neither touching `material_weights` (five milestones in, the golden-verified fraction blend has still never been edited): `rock_material_col` blends the reference's own `rock_col` toward the real rock's palette (five new palettes added), and bedrock **shows through thin soil** gated on §12's own list — slope, vegetation potential, effective moisture — scaled by the cover fraction not already rock or snow, so it is self-limiting and never bleeds through an icecap. The lithology index is sampled through a **coherent positional jitter** so a categorical contact reads as a ragged natural boundary rather than the vector line §30 forbids — the same idiom `bio_jitter` already uses for biomes. §18 is `apply_local_contrast`, **the first stage in `render.rs` that is not per-pixel**, and necessarily so: a neighbourhood of the *finished* colour does not exist until the raster does, so it runs over the output byte buffer in `lib.rs` (after the river tint, before the icon pass) and `cell_color` is untouched by it. §18's three constraints hold by construction rather than by tuning — the response `d·exp(−(d/knee)²)` makes gain **fall to zero** on strong edges (an unsharp halo is an overshoot proportional to edge strength; here gain is inversely related to it, so there is nothing to overshoot with), the correction is additive and equal on all three channels so chroma is provably unchanged, and the band is a ~20-cell blur rather than a 3×3 kernel. It fades under the plate frame via milestone 4's own `border_cover`. **Two real corrections, milestone 3's lesson holding a third time.** (a) *The geology gate was written in raw slope units, and raw slope is resolution-dependent* — `slope_at` is a per-**cell** height difference, so median land slope over Classic measures **0.00354 at 512² and 0.00054 at 2048²**, and the first threshold therefore confined the whole stage to the steepest ~5% of land *at the resolution the app actually runs at* while looking perfectly reasonable in source. Fixed by normalizing to `slope * gw` (this project's own convention — `build_slope_field` stores `slopeAt*GW`): affected Classic pixels went 1.17% → **6.61%**. The reference's own `material_weights` normalizers inherit the same dependence and were left exactly alone, being golden-verified. (b) *Local contrast as a plain high-pass amplified the sheet's own texture* — `luma − blur(luma)` sweeps in milestone 4's ~3-cell paper grain and the C¹ seams of its value-noise lattices, producing a faint rectangular quilting across land and sea (§30's "random texture noise", the same class as milestone 2's AO speckle and milestone 4's halftone stipple, found the same way — a downsampled real dump, not a statistic). Fixed by making it a **band-pass**, subtracting a small blur instead of the raw image, with the benefit intact (luma sd 33.10 before the fix, 33.08 after). **Anti-list numbers** (2048², seed 12345, frame band excluded; base = milestone 4's look): interior luma sd **31.94→32.85** (Classic), **28.34→28.98** (Archipelago), **27.28→28.80** (Wide 2048×1024) — contrast *rises* in all three, which is the point — while mean luma falls about one level (132.75→131.60, 105.98→105.31, 136.98→135.23) and clipping *falls* (0.78%→0.67% on Classic), so the separation is bought from the middle of the range rather than by pushing anything to black or white. Chroma moves at most 1.25/52 and the isolation dumps show that entire movement belongs to geology, local contrast measuring 51.79 against a 51.80 base — luminance-only as claimed. Luma min drops 2-7 levels from local contrast deepening the darkest concavity; 26.9/255 at worst is a deep shadow, not a black valley. **Which stage carries what** (pixels moved >3 levels/channel): geology 6.61%/0.94%/10.75%, local contrast 24.90%/11.69%/31.52% for Classic/Archipelago/Wide; within geology the halves split 0.94% (rock palette) to 5.29% (soil show-through) on Classic, the show-through carrying most of it because at 2048² the reference's own rock *fraction* is small except near summits — the same resolution finding again. **Cross-world honesty**: same direction as milestones 2-3, not milestone 4's inversion — geology is strong on mountainous Classic and the wide plate and nearly absent on Archipelago (0.94%), because a low-relief fragmented world simply has little steep thin-soiled ground, while local contrast is substantial in **all three** since every world has material boundaries whether or not it has mountains. That is exactly why the pair was worth doing together. **Non-square correctness** (`22ae75b`): every radius here is keyed to `gw`, so the local-contrast radius is capped against the short axis; a 2048×1024 world was added to the A/B harness and carried through every measurement, and its frame band is **bit-identical** before and after — 0 of 168,896 pixels changed, so `border_cover`'s fade is exact rather than approximate. **Golden parity: the same gating mechanism extended a fourth time** — `js_reference()` gains three more zeros and each stage early-returns on its own (`rock_material_col` returns the reference colour before touching a palette, the show-through block is inside an `if`, `apply_local_contrast` returns before allocating), with §12 additionally off *by data* since `with_lithology` is a builder the golden test never calls. `golden_parity_render.rs` remains **completely unmodified**, both tests at their original `1e-4`. One new non-`#[ignore]`d test asserts `LITHO_PALETTE_ORDER == cartalith_civ::LITH_KEYS`, guarding the one duplicate `render.rs` cannot check itself (it is `#[path]`-included standalone). **Cost** 2048²: 923→1110 ms Classic, 607→752 Archipelago, 501→599 Wide; real-app `build_color_texture` 1442/1085/761 ms, one-shot at generate time. Verified: `cargo test --workspace` **572/0**, clippy clean for this crate's files, headless load clean, and the real `build_color_texture` path (which the dump harness does *not* exercise) run headlessly end to end for all three worlds. **Milestone 4 (the atlas look)**: three of the four elements `VISION.md`'s sequencing item 2 still listed as ahead — a **paper/vellum ground** applied in `cell_color` after *both* the land and sea branches (an ocean not on the same sheet as the land makes the map read as terrain art pasted onto parchment), composed of a parchment tint divided by its own Rec.709 luma plus `paper_wash`, a pull toward a paper-coloured grey *of the same luminance*, so both parts are luminance-preserving and only chroma moves; **forest stippling** weighted by `material_weights`' own `canopy` fraction (real data, not decorative noise), `smoothstep`-gated and zero-mean so canopy gains texture without net darkening; and a **physical plate border** (paper margin, thick + thin neatline, ink density varied along the rule). None touches `material_weights` or the palettes. **Golden parity: same mechanism extended, not replaced** — `js_reference()` gains three more `0.0`s and each stage early-returns on its own zero (a dedicated branch, exactly as `relief_lights <= 1` established), so `golden_parity_render.rs` stays **completely unmodified**, both tests at their original `1e-4`. **Two corrections caught by looking, not by diff statistics** (milestone 3's lesson holding a second time): the parchment tint alone was only a hue rotation and read far too weakly until the chroma wash was added, and the first stipple field read as a regular diagonal halftone screen — §30's "random texture noise", the same class of regression as milestone 2's AO speckle — fixed by rotating the sampling lattice ~34°, domain-warping it, and flooring mark size at 4 cells. Anti-list numbers, terrain only (2048², frame band excluded): interior mean luma 132.8→**133.0** (Classic) and 106.3→**106.2** (Archipelago), contrast *rises* (sd 31.32→31.89, 27.66→28.30) so nothing is washed out, luma min drops just 1.4/0.8 levels from grain (no black valleys), terrain clipping unchanged. **Cross-world result inverts milestones 2 and 3**: those were strong on mountainous Classic and near-invisible on Archipelago; this one is stronger on Archipelago (−26% chroma vs Classic's −13%, its bright cyan sea becoming a muted teal-grey) because the paper acts on the whole sheet and that world is mostly ocean — and the two worlds converge from 18% apart in chroma to within 0.01 (51.960 vs 51.963), not by clamping but because a shared printing medium is what converges differently coloured subjects. **Not free**, unlike milestone 2: 2048² render 598→915 ms (Classic), 295→597 ms (Archipelago), four extra `vnoise` calls per pixel including ocean — accepted as a one-shot generate-time cost, and recorded as the first thing to optimize if the render ever needs to be fast. **Known limitation flagged, then fixed in a same-day follow-up**: `lib.rs`'s river channel tint and `map_overlay.gd`'s settlement markers both drew over the finished raster and knew nothing about the frame, so an edge settlement's marker landed partly on the plate margin. Resolved (see the milestone-4 follow-up entry in `CHANGELOG.md`) — and it was **four** systems, not two, the territory wash and province boundary lines having the same bug. `render.rs` now exports the frame geometry (`border_width_cells`/`border_cover`, plus `WorldGen::get_border_inset_frac()` as a fraction of texture width); the three Rust rasters fade by `1 - border_cover` and `map_overlay.gd` scissors to the plate interior. Insetting the overlay coordinate space was considered and rejected as the wrong shape for this frame: `apply_border` composites *over* the outermost cells rather than shrinking the map into a margin, so the terrain under the margin is covered, not moved, and inset markers would be displaced from the coastline they sit on. Instead linear features are clipped at the neatline (a road genuinely continues off the sheet) while point symbols are placed or omitted, never sliced. Margin overlay ink at 2048²/seed 12345/Classic: 268 px marker orange and 67 px river cyan before, 0 and 0 after, with all before/after difference confined to the frame band. Verified: `cargo test --workspace` 383/0, clippy clean for this milestone's files, headless load clean, real windowed app screenshotted at 2048² for **both** worlds, with the controlled before/after coming from `appearance_ab_dump.rs` extended with `noatlas`/`withatlas`/`paperonly`/`stippleonly` dumps at that same resolution. Hand-lettered glyphs, the fourth atlas element, are `map_overlay.gd`'s (GDScript overlay work, not renderer work). **Milestone 3 (hydrology tint)**: `land_color` gains a subtle cool/dark pull near high flow accumulation (`hydro_wet_strength`/`hydro_wet_radius_frac`, applied at the same final tonal stage as AO/vignette, never touching `material_weights`) — reuses the existing `flow` field already threaded through `RenderCtx` (zero `lib.rs` changes), log-compressed/min-max-normalized the same way `build_ao` already is, kept only above a `smoothstep` threshold, blurred into a soft halo. `js_reference()` sets it to `0.0` (a true no-op), both golden-parity render tests unchanged at `1e-4`. **Real tuning pass, disclosed**: the first parameter guess passed every mechanical check but a real crop at actual strength showed nothing perceptible (0.4% of pixels, mean diff 2.5/765) — caught by looking, not by the diff stats; retuned (0.20→0.38 strength, 0.004→0.006 radius, widened activation threshold) until a crop centred on the programmatically-found max-diff pixel showed a real, deliberately subtle valley-floor cooling. Cross-world honesty matching milestone 2's own AO finding: visible on Classic (2.19% of pixels), essentially imperceptible on low-relief Archipelago (0.75%) since there's simply less major drainage there — not a bug. Anti-list held: identical luma minimum before/after in both worlds (no new black valleys), no banding/haloing. Verified via the extended `appearance_ab_dump.rs` harness (an isolation pair holding milestone 2's own relief/AO fixed) rather than repeated windowed screenshots, following milestone 2's own finding that UI automation was unreliable this session — one real end-to-end windowed run confirmed correct generation/rendering, not a multi-shot comparison. **Milestone 2 (relief lighting)**: multidirectional hillshade (6 weighted lights, primary NW sun still dominant at 43%; the normal is computed once and dotted against a precomputed light table) plus heightfield ambient occlusion (`build_ao`, a two-scale cavity map over the existing box blur, replacing a `1.0` hardcoded in `land_color` since the renderer landed). Chosen because both act on the *lighting* term only, never on `material_weights`/the palettes — the golden-verified part, and the part §32 warns is easiest to improve for one terrain type while wrecking another. They're complementary: multi-light reveals ridgelines parallel to the single sun, but flattens depth; AO restores it from terrain concavity. AO normalizes each scale by its own RMS **over land cells only**, so occlusion is measured against each world's own relief statistics — a fixed threshold would give a flat world no AO and crush an alpine one. **Golden parity kept exact, not re-baselined and not loosened**: new `TerrainAppearance::js_reference()` reproduces the pre-milestone renderer bit-for-bit (`relief_lights: 1` takes a dedicated early-return branch; `ao_strength: 0.0` skips the precompute), and `golden_parity_render.rs` both tests still pass at their original `1e-4` tolerance with every expected value unchanged — the only edit is which appearance the context is built with. That follows `DECISIONS.md` §7a read strictly: its carve-out is for paths where JS parity is *impractical*, and it explicitly says the CPU rendering port stays golden-verified. Real before/after (deterministic dump + real windowed app, 2048², seed 12345): drainage networks, ridge/valley structure and coastal escarpments become legible where the single-sun render was a flat tan wash; measured against §30's anti-list, min luma is **identical** before/after in both test worlds (no black valleys) and mean luma moves only 133.3→128.8. A 3× zoom caught one real regression mid-pass (fine AO radius resolving to 1 cell read as speckle — "random texture noise") which was fixed before landing. Cost essentially nil: 512² render 45→45 ms. New `tests/appearance_ab_dump.rs` (`#[ignore]`d) is research doc §1.6's deterministic A/B comparison harness. **Milestone 1 (`TerrainAppearance` abstraction)** — `render.rs`'s colour logic (25 material/water palettes + shading constants, previously bare module consts) now lives behind a real, owned `TerrainAppearance` struct, pixel-identical output verified via `golden_parity_render.rs` unmodified. Real audit correction: there's no elevation-keyed colour *breakpoint ramp* in this renderer at all — colour comes from a continuous material-weight blend (temperature/moisture/slope/relative-elevation/aspect/curvature), not a MapTiler-style elevation lookup, so the research doc's own mental model doesn't map onto how this renderer actually works; a literal elevation ramp would be new visual-layer design work for a future milestone, not a re-encoding. Not yet wired to any UI — standalone-but-real, matching `cartalith-spatial`'s precedent. Three things to remember for what comes next: **(a)** criterion 2's renderer (above) ports the reference's *default-settings* material model only — real biome colours, real hillshade — explicitly excluding every `state.viz.*`-gated stretch feature (splat texturing, geology microtexture, NPR "Painter" styles, AO/SVF/shadows, multi-sun, SDF coast/river/biome tinting). Wiring any of those in is genuine Phase 3 work. **(b)** When that work lands, re-invoke `ui-ux-pro-max` for the UI side rather than bolting raw sliders onto the newly-exposed params — keep it consistent with the 2026-08-16 light parchment theme (ported from the reference's own `:root[data-theme="light"]`), not the earlier dark-dashboard match that theme replaced. **(c)** GPU compute *via Godot's own renderer* was researched 2026-08-16 (prompted by `godot-demo-projects/compute/heightmap`) and found not applicable *through that path*: `project.godot` uses the `gl_compatibility` renderer, which doesn't support `RenderingDevice` compute dispatch at all (engine-level constraint, already documented in `.claude/skills/godot-shell/SKILL.md`). That finding does **not** apply to a *standalone* `wgpu` instance created directly by Rust code — see the GPU-compute pilot section below, which tested exactly that and found the hardware path itself viable (the renderer choice is irrelevant to a `wgpu` instance that never touches Godot's own rendering pipeline). If Phase 3 revisits Godot's own renderer for other reasons (3D terrain drape, particles), GPU-accelerated presentation-layer work *through Godot* becomes reachable as a further, separate option — not before, and not for core generation (which must stay CPU-Rust for golden-parity reproducibility regardless of renderer). |
| 4 — Asset Library | **Done, 2026-08-17 — all 7 milestones, investigated and built for real** (`ASSET_LIBRARY_SCOPE.md`, new). `ROADMAP.md`'s own "Confirm before starting" note satisfied by the owner's direction to continue "until you've finished phase 4". **What it really is**, read out of the reference rather than out of the two pre-implementation design docs in `docs/`: an "asset" is not an arbitrary named image but **one PNG bound to one slot in a frozen, ordered vocabulary** — 8 families, 7 closed (7 splat channels / 15 biome grounds / 13 terrain grounds / 10 feature icons / 9 settlement pins / 7 trait overlays / 8 POI markers) plus one open-vocabulary `custom` family; slots hold 1..N variants picked by deterministic position hash. Order is load-bearing twice over (biome/terrain lists index-align 1:1 with the frozen `CART_BIOMES`/`CART_TERRAINS` paint vocabularies; structure lists mirror `CIV_SETTLEMENT_CLASSES`/`CIV_POI_TYPES`/`CIV_TRAITS`). An **asset pack is a real serialization format**, not a proposal — plain PKZIP via the same `zipStore()` the world save uses, `pack.json` (schema 1 or the schema-2 superset) or a real `pack.csv` alternative, manifest-is-source-of-truth, unknown keys warned rather than rejected. A **second, different** format also exists: `assetlib/library.json` + `assetlib/img/N.png` embedded in a project `.zip` (`_alExportEntries`/`_alImportProject`) — that is the "Asset Library payload" `SAVEFILE_COMPAT.md` already lists among ignored entries. The renderer genuinely draws pack sprites (`placeMapIcons`→`iconSlotForItem`→`pickWeightedVariant`→`drawMapIcons`, bottom-anchored); the vector glyphs are the fallback, not the reverse. Phase 5's urban morphology does **not** consume packs (checked). **Size, stated plainly**: ~2,250+ lines against the Journey Planner's ~3,100 — but only ~600-800 lines of that are portable logic, wrapped in 1,000+ lines of editor UI (the sprite-sheet slicer modal alone is ~408 lines of canvas/pointer interaction) plus an image/ZIP platform layer that is crate work, not porting. A real sub-phase, seven milestones. **Milestone 1 done**: new standalone crate `cartalith-assets` (no `gdext`, no dependency on any other Cartalith crate — `cartalith-spatial`'s precedent) carrying the pack manifest layer: the seven frozen vocabularies + a `Family` metadata enum, `RawManifest`/`PackManifest`, `parse_pack_csv`/`parse_pack_manifest`/`parse_pack_entries`, `pack_summary`, schema-2 `to_raw`/`to_pack_json`, and a ~40-line insertion-ordered map (needed because warning order follows the *author's* key order, `BTreeMap` would sort it away, and serde_json's `preserve_order` would have leaked into `cartalith-io` via workspace feature unification). **Golden-verified against the real reference** via a transient Node `vm` harness over `parsePackCsv`/`parsePackManifest`/`packSummary`; all five fixtures matched first run, targeting the plausibly-wrong cases (missing file vs. unknown slot, one variant missing vs. all missing, bare string as one-element list, stable CSV variant ordering, JSON-wins-over-CSV, empty path as missing file, exact wording *and order* of nine warnings). 28 tests. **Not wired to anything**, per the standing "don't wire in what nothing calls" discipline. **Milestone 2 done**: pack `.zip` read/write, placed in `cartalith-assets::archive` behind an on-by-default `zip` feature (the scope doc had left `cartalith-assets`-vs-`cartalith-io` open; reading `cartalith-io` first settled it — its whole zip surface is three `zip`-crate calls, so there is no helper to extract, it is reading-only by explicit scope so a pack *writer* would break that boundary, and the dependency would point the wrong way). What is actually ported is the reference's export *policy*, not the container: `.png` STORED and everything else DEFLATED, timestamps frozen at 1980-01-01 so exports are byte-reproducible (the `zip` crate's own default is the wall clock), `pack.json` written last, names read verbatim so a wrapping folder still fails the way the reference fails, directory entries kept, and an unreadable method erroring in the reference's own words. **Verified in both directions against a pack the reference itself exported** — the harness ran the reference's own `PackManifestBuilder.build()` + `zipStore()` headlessly (only the canvas rasteriser and three DOM inputs stubbed, stated in the test file); this port's read matches every name and CRC-32 and reproduces `pack.json` byte for byte, and its write reproduces order/method/CRC/size/timestamps *and* was fed back through the reference's own `unzipAny`+`parsePackManifest`, which read it with identical payloads, summary and warnings (the two archives differ by 2 bytes total, first divergence at the version-needed field). 14 new tests. **Milestone 3 done**: scatter rules (`cartalith-assets::scatter`) — `ScatterRule` + `ScatterMode`, the ten `SCATTER_RULE_PRESETS` that reproduce v1.25's hard-coded biome→asset switch, `scatter_rule_key`, `normalize_scatter_rule`, `current_scatter_rules`, `autopopulate_scatter_rules`, `pick_weighted_variant`/`pick_icon_variant` and `ScatterRule::spacing_cells`. The v1.27 hardening was **ported as fixes and re-derived for Rust**, one test naming each: the `NaN`-density carpet survives translation *by the opposite IEEE rule* (`f64::min` absorbs NaN where `Math.min` propagates it, and `keep >= 1.0` is false anyway); the `NaN`-spacing collapse of the relief bucket grid to 1×1 is real and Rust's `f64::max` would have masked it, so the `is_finite` guard stays explicit; and the `Object.assign` aliasing bug is **structurally unreachable — not because of ownership** but because defaults and untrusted input are different *types* (`ScatterRule` with `f64` fields vs. `serde_json::Value`), so no defensive code was written for it and the test asserts the observable outcome instead. Plus one guarantee the reference cannot have: `Serialize` but **deliberately no `Deserialize`**, making `normalize_scatter_rule` the only door in. **Golden-verified** by the same Node `vm` technique — `pick_weighted_variant` is deterministic-hash-driven and diffed exactly (11 cases × 36 positions, including the three degenerate weightings that must fall through to `pickIconVariant`'s untouched v1.25 hash), and 37 normalizer fixtures caught a real bug on the first run: `density`'s fallback is **not** symmetric with the other numeric fields — absent keeps the slot preset's own value (`cactus` stays 0.35) while a *rejected* one lands on a literal 1. 24 new tests, still wired to nothing. Three corrections to milestone 4 recorded: it is not the first milestone with a cross-crate dependency (milestone 3 is — `cartalith-noise`, for the variant hash); `pickIconVariant` and `spaceOf` shipped here rather than there; and `biomes` is `Vec<f64>` because `Number.isFinite` does not coerce, so a hand-edited `5.5` is kept and simply never matches. **Milestone 4 done**: rule-driven icon placement, `cartalith-assets::placement` — `place_map_icons_ruled` (the reference's `placeMapIconsRuled`), `icon_slot_for_item` with the `TREE_SLOT`/`SCATTER_SLOT` legacy fallback maps, and `sprite_draw_rect`; the reference's own legacy (non-ruled) `placeMapIcons` body is out of scope (nothing calls it, and `iconSlotForItem`'s legacy branches are ported for completeness without it). The first real placement golden-parity surface in this crate: positional and seeded, so it diffs **exactly**, not within a tolerance. **Both v1.27 fixes confirmed structurally necessary in Rust** (unlike one of milestone 3's three) — the most-specific-wins priority sort, because insertion-order dependence is a `Vec`/array property in any language; `requireWetland` ANDed with the biome test, because the old "replace" predicate is an algorithm defect a straight transcription would reproduce regardless of language. Proven with a hand-traceable 3-cell, `tGap=1` fixture (the scatter grid's jitter degenerates to zero at `tGap=1`, so sampling is exact per cell): a wetland+matching-biome cell, a dry+matching-biome cell, and a wetland+wrong-biome cell resolve to `wetland_grass`/`narrow_biome`/`generic_land` respectively, unchanged whether the rule array is inserted least-specific-first or reversed. **Golden-verified** against the real reference via the same Node `vm` technique: broad sweeps over a synthetic 10×8 grid across six seed/sea/density configurations match cell-for-cell and size-for-size (1e-9), including a dense case that exercises both relief bands, three different scatter specificities, and the `ghost_biome` non-integer-biome probe (`biomes:[5.5]`) placing nothing anywhere, confirming `biomeOk`'s `biome[i] as f64` cast. 23 new tests (12 unit + 11 golden), still wired to nothing. **Milestone 5 done**: the Library model, `cartalith-assets::library` — `AssetDB` (frozen bootstrap, custom-slot add/rename/remove, lazy scatter-rule attach, item store), `AssetCollections`, `run` (`AssetValidator.run()`), and the `assetlib/library.json` shape (`LibraryFile`/`SlotRecord`/`ItemRecord`, parse + `to_library_json`/`apply_library_file`), lining up with `SAVEFILE_COMPAT.md`'s existing "nothing to deserialise into yet" note — that something now exists. Pure data; every item's `hash` is caller-supplied rather than computed from pixels. **Two real corrections to this row's own §4 framing, found by reading**: per-slot display *names* turned out functionally load-bearing after all (`AssetValidator.run()`'s "Identical images" warning renders `slot.name`, golden-confirmed as `"Mountain#1 = Hill#1"`, so the 65-entry `mkSlots` title table is ported as `slot_title`), and the Library's own `poi` vocabulary is **ten** slots (`lake`/`bridge` included), not the eight `PACK_POI_SLOTS` milestone 1 ported for pack-import validation — both lists now exist. **The id-slugging/uid-collision hardening asked for by name, found and ported**: `addCustomSlot` returns the existing slot on a uid collision rather than duplicating it, `renameCustomSlot` refuses a colliding rename and keeps the old uid — neither carries a version-tagged comment like v1.27's fixes, reported as a finding rather than a named historical fix, both real defences against untrusted user text colliding on one slug. A companion finding: two of `run`'s six checks are structurally unreachable through the public API in both languages (the same shape of surprise as milestone 3's `Object.assign`-aliasing finding), ported anyway as defence-in-depth. **Golden-verified**: twelve constructed library states for the validator (matched on first run, pinning exact warning order) plus five for the export shape. 56 new tests (23 unit + 32 golden + 7 hardening). Corrections to milestone 6: its `itemHash` duplicate detection is already implemented (only the pixel-hash itself is missing); its per-item transform data shape already exists. **Milestone 6 done**: real pixels, `cartalith-assets::raster` — `decode_png`/`encode_png` (the `image` crate, `png`-only, no default-features), `item_hash` (real FNV-1a-with-stride-7 content hash over a 32×32 downsample, deliberately **not** byte-matched to the reference's own canvas-resample output — never serialized on either side, `_alExportEntries` writes `{img,name,t}` with no `hash` field and `_alImportProject` recomputes it fresh after decode, so no cross-run comparison is ever made, and the reference's own resample kernel is implementation-defined per the Canvas spec so it could not be matched even if it mattered), `fit_to_bottom` and `finalize_pack_texture_inv_mean` (pure arithmetic, golden-verified against the real reference — the only two pixel-adjacent functions in this milestone with no DOM dependency), and `render_item` (the reference's own shared `ThumbnailRenderer` core — thumbnail, inspector preview and pack-export bake all go through one function there, and now here too). `AssetDB::apply_library_file_with_items` is the milestone-5-flagged wrapper: calls `apply_library_file` then decodes/hashes/restores each item whose bytes the caller supplies, silently skipping one damaged image exactly like the reference's own `try/catch`. 15 new tests (10 raster unit + 3 library unit + 2 golden), real unit tests for the DOM-dependent functions since no headless execution path can reach a `CanvasRenderingContext2D`. **Milestone 7 done**: renderer + Godot integration, in a new `cartalith-godot::pack` module — the first thing in the workspace to depend on `cartalith-assets`. Real sprite compositing (`drawMapIcons`'s Y-sorted painter's pass, real pack art via a bilinear blit, plus a real procedural glyph fallback for all ten icon slots) and real ground-texture splat (the six `SPLAT_PAINT_SLOTS` channels blended into `land_color` via the already-computed `materialWeights` fractions and ramp colours, no new logic). The two "painted layers" (Cartography paint-brush biome/terrain override) are honestly out of scope — this port has never ported the paint-brush tool that would produce `pBio`/`pTer`, and building one is separate UI+state work the milestone's own "no GUI controls" boundary rules out; recorded as a named follow-up, not a silent gap. Splat (`state.viz.splat` defaults `0.7`, gated only by `assetPack.texAny`) and icons (`state.viz.icons` defaults `false`) are both genuinely additive/opt-in, no JS-parity gate needed — confirmed by `golden_parity_render.rs` passing unmodified at its original tolerance. Real, permanent new API: `WorldGen::load_asset_pack`/`has_asset_pack`, dormant since this port ships no default pack. Verified three ways: a new `tests/pack_compositing.rs` against the real `reference_pack.zip` fixture (sprite blit, glyph fallback, and the pack-with-no-icons no-op, all proven on a synthetic world), full static verification (build/test/clippy/headless load all clean, zero regressions), and a real windowed run — generated a real world, loaded the real fixture pack, and confirmed by looking at the native output pixels: a sharp flat-coloured rectangle (real sprite art) where a mountain relief peak should be, a large irregular checkerboard region following real land-material boundaries (real splat), and soft translucent blobs elsewhere (the glyph fallback). **Phase 4 is genuinely complete against `ASSET_LIBRARY_SCOPE.md` §8's own "done means"** — the Library-authoring UI is that document's own explicit carve-out, tracked separately in `GUI_SHELL_SCOPE.md`, not part of this phase's definition of done. **Milestone 8 added 2026-08-20** (after the GUI window found the gap the owner then reported): the **sprite-sheet slicer**, `cartalith-assets::slicer` — a golden-verified port of the reference's `SpriteSheetImporter` (HTML lines 27465-27870): `computeCells` (whose spacing is a **half-gutter on interior edges only**, not a pitch, so the outer cells come out wider — the equal-cell formula the Godot overlay carried drew a grid the slice would not have followed), `cropCell`'s rounding and clipped blit, `applyChroma`, and `isBlank`'s alpha>8 threshold; plus `as_load_sheet`/`as_slice_preview`/`as_slice_apply` and a live modal whose cell-detection readout is now the engine's real pass rather than an 8×8 sample. Two disclosed deviations from a straight port, both because `DCC_SHELL_SPEC.md` §8's control list and the reference's own do not match: *Trim transparent edges* is an **addition** (the reference has no trim; it has chroma keying, now also wired) and *Assign to family / Fill from* is §8's framing of the reference's flat target-slot dropdown, composed from `add_item` with no new arithmetic. Seven mutations killed; `ASSET_LIBRARY_SCOPE.md` §11. |
| 5 — Urban morphology | **Started 2026-08-18, milestones 1-4 of ~17 done** (`URBAN_MORPHOLOGY_SCOPE.md`, new). The roadmap's "ports cleanly" assumption was **verified, and half of it is wrong**: the boundary really is clean (block 4 is genuinely DOM-free — zero hits for `document`/`window`/`canvas`/`getElementById` in its whole range — and ships its own `hashModel` determinism golden and a `_test` export), but the size is not: **92 engine functions / 2,937 lines, plus a 28-function / 925-line civ adapter in block 2 — ~120 functions, ~3,860 lines, the largest single unported subsystem left**, bigger than the Journey Planner (~70 functions, 6 milestones) and the Asset Library (~2,250 lines, 7 milestones). The roadmap's "depending on `cartalith-civ` for settlement context" is also **wrong for the engine**: `generate(seed,opts)` takes only scalars and two plain rasters (water mask/DT/river centreline; heightfield), no civ types at all — the civ coupling lives entirely in the block-2 `_um*` adapter, which is milestone 17. So `cartalith-urban` depends on `cartalith-rng` **only**. Phase 4's finding that block 4 does not consume asset packs was re-checked independently and **confirmed**. **Milestone 2 done**: the planar street graph (15 functions, lines 28363-28512) as `cartalith-urban::graph` — dense `Vec`-with-tombstones settled for the whole crate, `nextN`/`nextE` proven redundant and dropped, `gKey` folded into an `(i64,i64)` key; 19 full-graph-state goldens through `UME._test` (nodes, adjacency, tombstoned edges, the uniform grid cell by cell, faces — exact, no tolerance), then **mutation-checked**, which is how two unexercised constants were found and two more scenarios written; `hashModel()` found **not** usable before milestone 16 (it needs a whole `generate()` model), correcting the scope doc; `js_hypot` shown to change graph *structure*, not just rounding, at the 11 m snap threshold; the block-comment assertion caught nothing but a negative control exposed a real hole in it, now half-fixed and half-covered by two structural asserts. Six findings written forward into milestones 6, 10, 11 and 12. **Milestone 3 done**: `astar` (lines **28514-28547**, not the planned 28514-28556 — the last nine belong to milestone 5's header comments) as `cartalith-urban::astar`, the hand-rolled heap ported literally because its tie-break is what makes the path reproducible. The finding that matters is about *verification*: seventeen hand-written scenarios reproduced the reference exactly on the first run and then **nine of fifteen mutations survived them** — because a continuously-valued cost raster essentially never produces two frontier entries with exactly equal `f`, so it cannot observe a tie-break at all. A search over ~800,000 combinations found a discriminator for every survivor, all of them **quantised** rasters, which is also what a real 8 m site cost field looks like away from the river; eight were added and **14 of 15 mutations now die**, the survivor being a provably dead branch reported rather than hidden. `js_hypot` vs `f64::hypot` quantified at **1,398 disagreements in 4,096** integer offsets. The reference's A\* documented as **reproducible, not optimal** (metres-vs-cells heuristic, no closed set, break on first pop) so nobody "fixes" it. See its own section below. |

## Phase 5 — Urban morphology (`URBAN_MORPHOLOGY_SCOPE.md`, started 2026-08-18)

**~17 milestones. Milestones 1-7 done (2026-08-18).** The scope doc carries the
full investigation; the four findings worth knowing without opening it:

1. **The roadmap's "self-contained DOM-free engine" is right, and then some.**
   Script block 4 (lines 28166-31104) is one `const UME = (() => {…})()` IIFE
   with **zero** hits for `document`/`window`/`canvas`/`ctx.`/`getElementById`/
   `localStorage`/`requestAnimationFrame` in its whole range. It ends with
   `module.exports=UME`, exports fourteen internals through a `_test` object,
   and ships `hashModel()` — a stable FNV serialisation the reference itself
   labels "for determinism goldens". This port did not have to invent a golden
   path; the reference built the door.
2. **The roadmap's "ports cleanly" is right about the boundary and wrong about
   the effort.** 92 engine functions / 2,937 lines, plus a 28-function /
   925-line civ adapter in block 2 = ~120 functions, ~3,860 lines — larger than
   the Journey Planner (~70 functions, 6 milestones) and the Asset Library
   (~2,250 lines, 7 milestones), and denser per line. **The largest single
   unported subsystem remaining.** It generates street networks *and* planar
   blocks *and* plot subdivision *and* building footprints *and* districts
   *and* walls *and* farmland — A\* primaries, an epoch-loop organic growth
   model, planar face extraction, bisector series-platting, curtain walls and
   bastioned star forts.
3. **The roadmap's "depending on `cartalith-civ` for settlement context" is
   wrong for the engine.** `generate(seed,opts)` takes scalars, strings,
   booleans and two plain rasters (`opts.water`: mask/DT/river centreline;
   `opts.terrain`: heightfield) — no civ types anywhere. The civ coupling lives
   one layer up in block 2's `_um*` adapter (milestone 17). `cartalith-urban`
   therefore depends on `cartalith-rng` **only**, which is also what let
   milestone 1 be built and verified while `cartalith-civ` was mid-edit by a
   sibling fork.
4. **Phase 4's asset-pack finding confirmed independently.** `assetPack`,
   `AssetLibrary` and `AssetDB` all return zero hits in block 4. It emits
   geometry with kind tags, never image references.

**Milestone 1 done** — new crate `cartalith-urban` (no `gdext`, no civ), two
modules: the labelled RNG substreams (`fnv1a`, `stream` and its
`range`/`int`/`pick`/`norm`/`logn`/`chance` draws) and the vector/polygon
geometry kernel (`js_hypot`, `Vec2`, `polyArea`, `polyCentroid`,
`pointInPoly`, `segInt`, `distPtSeg`, `polySelfIntersects`, `chaikin`,
`simplify`, `ensureCCW`, `insetPoly`, `clipConvex`, `convexHull`). 19 tests,
18 of them golden against the reference. **Not wired to anything.**

**RNG, checked not assumed:** block 4 deliberately does not define
`mulberry32` — it falls through to block 1's copy, the one `cartalith-rng`
already golden-verifies. So unlike Phase 2 milestone 9's `_civRng` (same
algorithm, different wrapper), this is literally the same function. What is new
is the seed derivation, `mulberry32(seed ^ fnv1a(label))`, giving labelled
substreams per stage. Draw order is load-bearing: `norm()` is Box-Muller and
consumes **two** draws, and `pick` consumes one even on an empty array.

**One real parity trap found, and it would have poisoned everything
downstream:** `V.len`/`V.dist` are `Math.hypot`, and **V8's `Math.hypot` is not
correctly rounded** — it scales by the largest magnitude and Kahan-sums the
squared ratios, so `Math.hypot(3,3)` is one ulp above Rust's `f64::hypot(3,3)`.
The first golden run of `dist_pt_seg` failed on exactly that. Every distance in
this engine flows through it, and many are threshold comparisons where being
*more* accurate than the reference is the wrong answer.
`cartalith_urban::geom::js_hypot` reproduces V8's algorithm, is golden-tested
against twelve captured values, and carries an explicit `assert_ne!` against
`f64::hypot` so nobody simplifies it away later.

**Milestone 2 done (2026-08-18)** — the planar street graph, all 15 functions of
reference lines **28363-28512** (the plan said 28513; `edgeBetween` ends at
28512 and `astar` starts at 28514), as `cartalith-urban::graph`: `makeGraph`,
`gKey`, `gridCellsForSeg`, `indexEdge`/`unindexEdge`/`edgesNear` (the
uniform-grid spatial index), `addNode`, `nearestNode`, `rawEdge`, `splitEdge`,
`attachPoint`, `addStreet`, `addPolylineStreet`, `extractFaces`, `edgeBetween`.
26 tests (up from 19). Dependencies still `cartalith-rng` only. **Not wired to
anything.** The planarity invariant lives here — `addStreet` snaps within 11 m,
T-junctions within 9 m, splits every crossing and promotes every node within
2.5 m of the segment's interior — and `extractFaces` (angularly-sorted half-edge
traversal with dead-end spur collapsing) is what makes blocks possible at all.

**Index design settled for the whole crate**, as the scope doc predicted: dense
`Vec` with tombstones, ids never reused, because `splitEdge` leaves dead edges
in place and later milestones walk `g.edges` by index. Two things the plan did
not say, verified rather than assumed: `nextN`/`nextE` are **not stored** (they
are unconditionally `len()`, asserted against the reference's own counters on
all 19 scenarios), and `gKey` **does not survive** (an `(i64,i64)` tuple key is
the same partition, and the grid is only ever probed, never iterated) — so 15
reference functions land as 14 Rust items. `cls` stays `&'static str` rather
than becoming an enum: the reference compares it by string in six places and
`hashModel` serialises it verbatim, and an enum would have to guess now at
classes later milestones introduce.

**Golden-verified through `_test`, then mutation-checked.** `UME._test` reaches
`makeGraph`/`addStreet`/`extractFaces`, and that is enough for all fifteen
because the harness dumps the **entire graph state** per scenario — every node
with adjacency, every edge including tombstoned ones, the uniform grid cell by
cell, and the faces — with floats as JSON shortest-round-trip decimals so
nothing is compared within a tolerance. 19 scenarios match exactly, including a
stress case driven by the reference's own exported `stream` (so it is a golden
over `cartalith-urban::rng` and the graph at once). Perturbing the 26 m index
cell, the 0.7 cell step, the 3×3 dilation, the 11 m snap, the 9 m edge snap,
both 3.5 m guards, the 2.5 m promotion radius, the `[0.03,0.97]` t clamp, the
spur-collapse stack rule, the outer-face strict `>`, or swapping `js_hypot` for
`f64::hypot` each breaks at least one golden — and the first mutation round
found two constants unexercised, which is why two scenarios exist at all.

**`hashModel()` was not usable here, correcting an assumption the scope doc
made**: it reads a finished `generate()` model's graph/blocks/parcels/buildings
and cannot be fed a partial subsystem, so it is a **milestone 16** instrument.
The state dump is stricter anyway — `hashModel` rounds to `Math.round(n.x*100)`.

**`js_hypot` earns its keep visibly.** At `dx = 7.778174593052022`, V8 gives
`Math.hypot(dx,dx) == 11` exactly while `f64::hypot` gives `10.999999999999998`,
and `attachPoint` snaps at strictly under 11 — so the reference builds a
**four-node** graph where an `f64::hypot` port builds a **three-node** one. Four
goldens straddle that boundary.

**The block-comment assertion caught nothing this time** (milestone 1's
boundaries are unchanged and correct) — but running it as a negative control
found a real hole in the assertion itself: a slice starting exactly one line
into the header comment escapes it, because the scanner reads an apostrophe at
depth 0 as a string delimiter and the comment prose contains `"Gen1's globals"`,
swallowing the stray `*/`. An orphan-close counter was added (it catches the
three-lines-late variant) and the residual hole is covered by the two
**structural** assertions. Recorded plainly: the balance assert is necessary,
not sufficient.

**Findings that change later milestones** (all in the scope doc, and written
into the milestones that must act on them): `cell`/`grid`/`nextE`/`nextN` are
touched **only** by milestone 2's functions across all 2,937 lines of block 4;
`g._fromPaths` is a dynamic JS property set by milestone 6 and read by milestone
10, deliberately **not** added here since nothing uses it yet; `splitEdge`'s
`adj` splice is **unguarded** where milestone 11's `_killEdge` guards the
identical one, reproduced rather than unified; `addStreet` leaves **orphan
nodes** when every link is rejected by the 3.5 m minimum; the stable hit sort is
a safety property because a tie is **unreachable** (proven by trying to build
one and failing, then by mutation); the `1e-4`/`1e-3` interior guards are
**redundant inside the 1700 × 1250 m site box** (they only bite past 35 km and
3.5 km respectively) — the two surviving mutations reported as a finding, not
hidden; `extractFaces`' `while (guard++ < 20000)` discards rather than truncates
a runaway traversal; and the outer-face tie-break is observable, a spurred loop
yielding two faces of equal `|area|` where the lowest index wins.

**Milestone 3 done (2026-08-18)** — `astar`, reference lines **28514-28547**, as
`cartalith-urban::astar`. 33 tests in the crate (up from 26). Dependencies still
`cartalith-rng` only. **Not wired to anything.** The plan said 28514-28556;
`astar`'s last line is at 28547 and 28548-28556 is a blank line plus milestone
5's own *site model* header comments, so the range over-claimed by nine lines
(milestone 5's 28557-28742 is right). A hand-rolled binary heap, 8-connected
`Math.SQRT2` diagonals, trapezoidal edge costs and a `0.9`-weighted Euclidean
heuristic **in cells** — ported literally rather than swapped for `BinaryHeap`,
because the heap's tie-break is what makes the path reproducible (sift-up stops
on `<=`, sift-down uses a strict `<`) and `BinaryHeap` has neither property.

**The important finding is about verification, not about A\*.** Seventeen
hand-written scenarios reproduced the reference exactly on the first run —
degenerate strips both ways, a walled detour, an infinite moat, a NaN band and a
NaN seal, zero cost, start-equals-goal, two `stream`-filled rasters, and a sweep
over every cell of a 6 x 5 raster as goal. Then fifteen mutations were run
against them and **nine survived**: the `0.9` weight, the `0.5` trapezoid, the
`DIRS` order, all three heap comparators, `js_hypot` vs `f64::hypot`, the
`i == gi` early break, and the dead `INFINITY` guard. **The cause generalises:**
a *continuously-valued* input essentially never produces two frontier entries
with exactly equal `f`, so it cannot observe a tie-break at all — only a
*quantised* one can. A search over ~800,000 (raster family x size x endpoint)
combinations found a discriminator for every survivor, and every tie-break
discriminator came from a quantised field (`{0.5, 1}`, `{1, 2}`,
`{1, 2, 3, 4}`). Eight such scenarios were added and **fourteen of fifteen
mutations now die.** That regime is the *normal* one for this engine:
`buildPrimaries` builds its raster as `(1 + (slope*3.2)^2)*8` and slope is flat
over most of a site, so the real 8 m cost field is mostly constant away from the
river. The one survivor — deleting `if (g0[i] === Infinity) continue;` — is
reported rather than papered over: it is unreachable in the reference too, since
`g0[ni]` is written on the line before every `push`.

**`js_hypot`, now quantified.** Over the 4,096 integer offsets a 64 x 64 raster
produces, it and `f64::hypot` disagree on **1,398** — better than a third, all
by one ulp. It still took a 64 x 48 quantised raster to build a golden that
notices, because one ulp only bites when it makes or breaks an exact tie.

**The reference's A\* is reproducible, not optimal**, and that is written down so
nobody "fixes" it: the heuristic is metres-vs-cells mismatched, there is no
closed set (cells are re-expanded), and `if (i === gi) break` stops on the first
*pop* of the goal. The golden path is the specification. `null` can only come
from non-finite cost — `Infinity` or `NaN`, both failing `c < g0[ni]`, the NaN
case being one of the few places where JS/Rust NaN agreement is load-bearing
rather than incidental. One deliberate divergence: an out-of-range endpoint
**panics** here where the reference silently reads `undefined` and sails past
its own guard; its only caller clamps first, so the branch cannot be reached.

**Harness improvements to inherit**: the first structural assertion is tightened
from "the slice *contains* the `UME` IIFE header" to "the slice's **first line
is** block 4's header comment opening", which catches milestone 2's documented
one-line-late hole directly rather than by luck; a fourth assertion runs as a
live negative control in the other direction (block 4 must **not** define
`mulberry32`); and the capture refuses to write a file unless every path is
non-empty, starts and ends where it should, the two sealed scenarios really
returned `null`, and the whole capture exceeds 300 cells. Also one tooling trap
worth knowing: the first mutation run reported two **false** survivors, because
`cargo`'s freshness check is mtime-based and because one mutation pattern
matched inside the function's own doc comment before it matched the code.

**Corrections written forward**: milestone 6 must not "improve" the search
(`buildPrimaries` reinforces used cells by `0.45` on a *copy* per route, so
order-dependence compounds); milestone 6 owns `toCell`'s clamp, since this
port's `astar` panics out of range; and milestones 12-13 will hit the same
coverage trap, so every milestone from here on should carry at least one
quantised or symmetric fixture and mutation-check its constants.

**Milestone 4 done (2026-08-18)** — generation rules and culture profiles
(`CULTURE_PROFILES`, `resolveProfile`, `DEFAULT_RULES`, `cloneRules`,
`resolveRules`, `clamp`, `applyWildness`, `applyPlotChaos`), reference lines
**28193-28280**, as `cartalith-urban::rules`. 43 tests in the crate (up from
33). Dependencies still `cartalith-rng` only. **Not wired to anything.**

**The stated range was wrong at both ends, in opposite directions** — the third
range in this plan to need correcting and the first whose *start* was wrong. The
plan said 28212-28289: the start was 13 lines late, so it **excluded
`CULTURE_PROFILES` entirely** (28212 is `resolveProfile`), and the end was 9
lines late, reaching into the `V` vector object milestone 1 already shipped.
Milestone 5's stated start (28557, `shoreFromMask`) was checked as a side effect
and is correct; the rest are still unverified.

**Data, and yet the most dangerous line in the subsystem so far.** `clamp` is
`Math.max(lo, Math.min(hi, v))`, and the obvious Rust transliteration
`lo.max(hi.min(v))` is **wrong**: JS propagates NaN, Rust absorbs it. A NaN
wildness slider leaves eight NaN street fields in the reference and lands
**every clamped field on its own upper bound** in a naive port — a
maximally-wild rule set that looks entirely plausible and feeds straight into
`grow`. Same trap `cartalith-assets` milestone 3 hit from the other direction.
Ported through explicit `js_min`/`js_max`, golden-pinned by `wild_NaN`/
`chaos_NaN`, with a `js_hypot`-style guard test so the simplification fails
loudly. One unreachable divergence remains (signed zero), and it is exactly why
two mutations survive.

**Findings.** `applyWildness` is **not idempotent** — ten of eleven assignments
recompute from a hardcoded literal, but `deadEndBias` accumulates off its own
value, walking 0.15 → 0.30 → 0.40 under repeated `w = 2` — and it silently
overwrites custom values it never reads. `profile.deadEndBias` **does not exist
on either live profile**, so milestone 11's profile-side term is always zero
(asserted against the reference's own key list). Four profile fields are read by
nothing at all, and **the reference's own provenance prose is stale** about one
of them (`venus`'s `prov` claims the UI reads `defaultWalls`; v2.10 has zero
reads anywhere). Nothing outside block 4 uses any of this milestone's exports —
the whole host app touches three names on `UME`. `resolveProfile` has a
**prototype-chain hole** (`'toString'` returns a function, `'__proto__'` returns
`Object.prototype`, both truthy, both past the `||` fallback) captured as the
reference's real behaviour with a golden asserting this port hardens all five to
`medieval`. `cloneRules` does not survive as a function, and is not quite a deep
clone either — a NaN round-trips to `null` through `JSON.stringify`, pinned and
unreachable inside the engine.

**Mutation-tested: 120 mutations, 114 died, 4 survived, 2 killed by the
compiler.** Every numeric literal on a non-comment line perturbed individually
(84) plus 36 structural ones. The survivors are reported with the invariant they
rest on: `js_min`'s `<` → `<=` and `js_max`'s `>` → `>=` differ only on `+0` vs
`-0` (the documented unreachable divergence), and the `1.0`/`4.0` bounds inside
`Math.round(clamp(2*c,1,4))` survive `+0.01` but **die** at `1.0 → 1.6`,
`1.0 → 0.0`, `4.0 → 4.6` and `4.0 → 3.0`. A fifth survivor — the `2` multiplier
— was killed by adding three scenarios, and it generalises: **a quantised
*output* hides a constant** exactly as milestone 3's continuous *input* hid a
tie-break, so the fixture that kills it is one whose input sits just below a
rounding boundary rather than exactly on it.

**And a tooling trap worth more than the milestone.** The first combined
mutation sweep reported **34 survivors**; every one died when re-run alone, and
two independent re-runs killed 34/36 and 114/120. The sweep had been reporting a
stale binary partway through. It was neither of milestone 3's two traps (both
guards were in place and held), did not reproduce on replay, and most likely
came from a sibling fork building in the shared `target/`. The durable rule, now
in the scope doc's verification convention: **re-run every mutation survivor in
isolation before reporting it** — a "did the tests run" gate cannot catch this,
because a stale binary reports a perfectly healthy `N passed`.

**Golden verification.** All eight items are on `UME`'s *public* export rather
than `_test`, so this is the first milestone in the subsystem needing no
indirection at all: 53 rule cases, both profiles field by field, 15
`resolveProfile` ids, compared **bit for bit** via `to_bits` with no tolerances
anywhere. The capture asserts the reference's `DEFAULT_RULES` still carries
exactly the captured key set in exactly that order, so a rule added upstream
cannot silently drop out of the comparison. Every golden matched on the first
run — which is why the mutation testing is the part that counts.

**Corrections written forward**: verify each remaining stated range before
slicing (three for three now); milestone 7's `grow` falls back to the **raw**
`DEFAULT_RULES` (`opts.rules||DEFAULT_RULES`, line 29446), not to a resolved
partial — reproduce that; milestone 11 gets a zero from the profile side of
`privatizeAlleys`' clamp; milestone 12 reads `subdivisionCap` as a float, whose
NaN-propagating `Math.min` is load-bearing; milestones 13-15 use `profile.id` as
a real **lookup key** into `GAMES_SPEC`/`FARM_SPEC`, which is why the profile
fields are `&'static str`; and every milestone from here that rounds, floors or
buckets an output should build a just-below-a-boundary fixture deliberately
rather than discovering it in a survivor list.

**Milestone 5 done (2026-08-18)** — the site model (`shoreFromMask`,
`buildSite`, `terrainSuitability`), reference lines **28549-28741**, as
`cartalith-urban::site`. 59 tests in the crate (up from 43). Dependencies still
`cartalith-rng` only. **Not wired to anything.** `buildSite` is the input
contract for everything downstream: it fixes the 1700 × 1250 m box, decides
where the water is, and hands back the five field closures (`height`, `slope`,
`riverDist`, `isWater`, `bankSide`) that anchors, routes, growth, walls, parcels
and buildings all query.

**The stated range was wrong at both ends again — four for four.** The plan said
28557-28742: 28742 is blank (`terrainSuitability` ends at 28741), and 28557 is
the first line of *code* but not of the milestone, since 28549-28556 are the
site-model archetype comment and `shoreFromMask`'s own v0.98 note. Milestones
6-16's ranges are still unverified.

**`Math.exp` is the second V8 libm divergence, and it dwarfs `Math.hypot`'s.**
The first golden run failed on `terrainSuitability` at one probe, one ulp out.
The platform `f64::exp` disagrees with V8 on **20,721 of 240,000** random
arguments; V8 calls FDLIBM's `__ieee754_exp`, ported here as `geom::js_exp`,
which disagrees on **0 of 240,000**. One measured special case is reported
rather than explained — across 244,000 arguments the two agree everywhere
**except at exactly `x == 1.0`**, where V8 returns the correctly-rounded `e`;
reproduced because it was measured, and unreachable from the site model, whose
`exp` arguments are never positive. **This retro-fixes milestone 1**:
`rng::logn` had been on `f64::exp` and its goldens passed by luck, and every
frontage width, plot depth and building dimension in the town is drawn through
it (five call sites in block 4).

**Findings.** `buildSite` is two sites wearing one name and which is live is
decided **per field, not per site**, so the port carries `Option<WaterCtx>` /
`Option<TerrainCtx>` rather than the single source enum the plan proposed.
`kind` is **not a closed vocabulary** — every unrecognised string takes the
coastline branch while still being returned verbatim, and milestone 9 compares
`site.kind === 'coast'` directly, so `kind` stays a `String`. `!!W.riverPath` is
truthy for a path **too short to be a river**. **A bay draws one fewer number
than a coast** (31 against 32), so their `routeEnds` diverge. One mask is read
**two different ways** (truthy in `shoreFromMask`, `=== 1` in `isWater`).
`shoreFromMask`'s principal axis can **collapse to `(0, 0)`**, after which the
sort is a no-op — and its fallback eigenvector fires on every plain horizontal
shoreline, invisible unless the shore has points in two rows. Out of bounds is
**`undefined`, not a panic**, reachable three ways, and the port diverges *the
other way* from milestone 3's `astar` — loud there because the case cannot
happen, quiet here because it can. `bankSide` **never returns 0**. `waterPoly`
is **empty on two of the four paths and read by nothing inside block 4**.

**Golden verification.** The first milestone here whose functions are on neither
`UME`'s public export nor its `_test` one, so the capture adds them to the
returned object with a single anchored replacement of the `return {` line,
asserted to match exactly once; the frozen reference is never edited. One thing
worth recording: `const UME = (() => {…})()` is a **lexical binding, not a
property of the `vm` context's global object**, so `ctx.UME` is `undefined`
however well the slice ran — the fourth appearance of this project's
silently-empty-output problem, met with an explicit `globalThis.__UME = UME;`
and an assertion. 19 shoreline scenarios and 36 site scenarios, each with **106
probes** of the five closures plus `terrainSuitability`, compared **bit for
bit** with no tolerances. Every golden matched on the first run except the one
probe that surfaced `Math.exp`.

**Mutation-tested: 271 mutations, 240 died (2 at the type level), 31 survived**,
every survivor re-run in isolation per milestone 4's rule. The survivors are
reported by class with the invariant each rests on: ten dead stores, six
equivalent by the surrounding arithmetic, two boundary tests whose branches
compute the same number, six guards against data the reference cannot produce,
four needing an exact tie a continuous field cannot make, and three unobservable
through Rust's stable sort (checked, not assumed — the stable sort reaches every
ordering decision through its `Less` arm).

**The first sweep is the finding.** It left **46** survivors and almost none
were equivalent mutants — they were two fixture gaps. Every hand-built water
raster was uniform along one axis, so no `maskIdx` `i`-clamp mutation was
visible; and a fixed `[0.1, 0.5, 0.9]²` probe grid never once entered the
10-40 m band around the river where every threshold in this milestone lives.
Rebuilding the probes **out of the site's own polyline** and rippling every mask
per column took the count 46 → 35 → 31 over three rounds, killing fifteen
constants by fixture rather than argument — several needing one built on
purpose: a **seed scan** for a channel whose drift actually saturates its upper
clamp, an 18.85 m-per-segment river whose quay walk reaches 94.25 m in five
steps (just under its own 95 m stop), a two-row shoreline (a one-row one cannot
show the fallback eigenvector, since sorting a row-major list by *y* is the
identity), the same cloud at 4 mm cells to push the eigenvalue discriminant
below 1, and a vertical shoreline so the harbour search's reference *y* decides.
**Milestone 3 asked for quantised inputs and milestone 4 for
just-below-a-boundary inputs; milestone 5 adds that a geometric subsystem needs
its fixtures derived from the geometry under test.**

**Corrections written forward**: every milestone from here must use
`geom::js_exp` for `Math.exp` (milestone 7's `logisticRamp` is the next direct
call site); milestone 6's `placeAnchors` can reach its literal market fallback,
because a landlocked site has neither a `bridgePt` nor a `harbour.pt`; milestone
9's `site.kind === 'coast'` is a string test an enum would have broken; and
milestone 10 must not read `site.waterPoly` as the town's water.

**Milestone 6 done (2026-08-18)** — anchors and primary routes (`placeAnchors`,
`buildPrimaries`, `buildPrimariesFromPaths`), reference lines **28743-28833**, as
`cartalith-urban::routes`. 69 tests in the crate (up from 59). Dependencies still
`cartalith-rng` only. **Not wired to anything.** The first milestone that
produces a real street graph end to end: `placeAnchors` picks the one point the
whole town is organised around, and the two builders lay the arterial backbone
that milestone 7's growth, milestone 10's enceinte and milestone 12's blocks all
accrete onto. `Graph::from_paths` — milestone 2's deferred dynamic property —
exists now, for milestone 10 to read.

**The stated range was wrong again — five for five.** The plan said 28744-28843:
`buildPrimariesFromPaths` ends at 28833, 28834 is blank, and 28835-28843 is the
radial-streets header comment belonging to milestone 8 (whose stated start
should therefore be 28835, not 28844); 28743 is the `anchors` section header,
which by milestones 4 and 5's convention belongs here. Milestones 7-16 are still
unverified.

**`Math.sin`, `Math.cos` and `Math.log` are the third, fourth and fifth V8 libm
divergences — and this time they were measured *before* a golden failed.**
`f64::sin` disagrees with V8 on **1,942 of 80,214** arguments, `f64::cos` on
**2,160**, `f64::ln` on **1,647 of 60,009**; `geom::js_sin`/`js_cos`/`js_log`
(FDLIBM's `__ieee754_*`, as V8 calls them) on **0** of each. `Math.sin`/`Math.cos`
are the third and fourth most-used functions in block 4 — 27 and 26 call sites,
behind only `Math.min`/`Math.max` — and `placeAnchors` calls both on every one
of its 400 candidates. **This retro-fixes milestone 1 a second time**: `rng::norm`
is `sqrt(-2·log(u1))·cos(2π·u2)` and had been on `f64::ln`/`f64::cos` with a
documented "they happen to agree" note; it is the highest-leverage function in
the subsystem, since `logn` sits on top of it and draws every frontage width,
plot depth and building dimension in the town. Its milestone-1 goldens pass
unchanged afterwards, which is the check. FDLIBM's Payne-Hanek branch
(`|x| ≥ 2^19·π/2`) is **deliberately not ported** — every trig argument here is
an angle inside `[-4π, 4π]` — and `js_sin`/`js_cos` hand off to the platform
above the threshold, with a test asserting they do.

**The rest of the libm bill, measured now so later milestones do not each
rediscover it:** `Math.atan2` disagrees on **10,615 of 60,000** (17.7%, the worst
yet, 7 call sites from milestone 8), `Math.log10` on 960/60,000 (milestone 15),
`Math.acos` on 544/60,000 (milestone 10). `Math.pow(x, 2)` is **bit-identical**
to `x * x`, so `buildPrimaries`' one `Math.pow` needs nothing.

**Findings.** Neither route builder **draws a random number** — both take a
`seed` and neither reads it, asserted from the other side by running each with a
different seed and requiring a byte-identical graph; `placeAnchors` draws
exactly 800 times, two per candidate, *before* any rejection test. **Both return
values are dead** — `generate()` discards them and keeps only the graph.
`riverthrough` shares `river`'s `[60, 240]` candidate band but **not** its 120 m
preferred distance (the score's ternary tests `'river'` alone). The market
reference's **third `||` arm is live** on a landlocked site, as milestone 5
predicted, and `best === null` is reachable on a small box — the one place in
the subsystem that can put the market **outside the site box**, with no clamp
anywhere. `Math.max(0, rd − 260)` is **dead on every site this engine can
build**, proven by an invariant test over all 38 fixtures rather than by
argument. `buildPrimariesFromPaths`' final `sm.length < 2` guard cannot fire and
its `path.length < 2` one is redundant, but its `pts.length < 2` one is not — a
path whose second point leaves the box would otherwise survive as a degenerate
two-identical-point street.

**One finding generalises past this milestone: a metre offset added to a metre
coordinate cannot express a one-ulp boundary.** Both boundary fixtures had to be
rebuilt for it — `(386.6 + 1.0000000000000002) − 386.6` is exactly `1.0`, so the
`> 1` unshift is straddled with 1 m and 1.25 m and the 6 m box tolerance with
−5/−6/−7 on all four sides. Milestone 17's adapter produces exactly these
offsets.

**Golden verification.** Same slice harness as milestones 3-5 verbatim, with
milestone 5's single anchored `return {` replacement and `globalThis` handoff
(the three functions are on neither export). 38 scenarios comparing market,
provenance, every route polyline, every node and edge, and the spatial index —
the last pinned by the reference's **own** `fnv1a` over its own canonical grid
dump rather than restating 400-odd cells per scenario. Bit for bit, no
tolerances. The capture's shape gate names the fixture behind each of its twenty
conditions (the 80 m margin must reject >20 and admit >20 on the mid-box
fixtures and **zero** on the full-size one; `lastCandidateWins` must win on
candidate 399; `shortDtWater` must admit >100 candidates and then score every
one `NaN`; and so on), and the Rust side mirrors it because `zip` stops at the
shorter side. **Every golden matched on the first run**, all 38, across three
rounds of fixture work.

**Mutation-tested: 306 mutations, 233 died, 73 survived**, every
survivor re-run in isolation and **zero false survivors** (milestone 4's
stale-binary problem, solved by giving the sweep its own `CARGO_TARGET_DIR`).
Four rounds took the count 98 → 79 → 73 → 74 → 73, and the fixtures that
closed the gap were all **scanned rather than guessed**: a seed whose winning
candidate is number 399; a site whose winner sits 80-110 m from a box edge (a
site that merely *rejects* candidates leaves the margin invisible, because
raising it only removes candidates that were losing anyway); a seed where the
market *coordinate* actually moves under `f64::cos` — a one-ulp cos error times
a 240 m arm is 2.4e-14 against a coordinate whose own ulp is 5.7e-14, so it
usually rounds away; a truncated `dt` beside a *real* heightfield, the only way
to get a NaN into the score without also NaN-ing the slope; and a polyline whose
Chaikin corners land between the 1.2 and 1.3 simplify tolerances.

**Two tooling incidents, both worth carrying forward.** **A dozen hand-picked
rows cannot test a bit-twiddling port** — the first sweep left **63 survivors
inside `js_sin`/`js_cos`/`js_log` alone**, by a golden table built exactly the
way `js_exp`'s and `js_hypot`'s were. The fix is four lines: an FNV-1a **hash**
over 54,000 sin results, 54,000 cos and 30,000 log, arguments drawn by the
reference's own `mulberry32` so both sides provably evaluate the same points,
bands chosen to enter each reduction branch on purpose — including two built
specifically for `rem_pio2`'s second and third correction rounds, which no
uniform band reaches. It matched V8 on the first run. Milestones 8, 10 and 15
each need one of these. And: **two mutation runners on one target directory left
a live mutation in the source** — the first was killed mid-mutation, the second
read the mutated file as its "original", and `routes.rs` carried `-(s * 5.61)`
where the reference has `-(s * 4)`; only the suite failing afterwards said so.
The runner now takes a pristine snapshot **before it writes anything**, restores
from it, re-runs the suite as a post-sweep baseline, and refuses to start while
a lock file exists.

**Corrections written forward**: milestone 8's range should start at 28835 and
it needs a **`js_atan2`** built against a bulk hash golden (17.7% divergence, 7
call sites); milestone 10 needs `js_acos` and milestone 15 `js_log10`; milestone
10's `builtMassHull` must read the new `Graph::from_paths`; milestone 16
inherits only the graph and the 800-draw `'anchors'` substream, since neither
builder touches the RNG; and milestones 7 and 10 should not assume
`anchors.market` lies inside the site box.

**Milestone 7 done (2026-08-18)** — organic growth (`logisticRamp`,
`estimateCarryingCapacity`, `wallOccupancy`, `grow`, `supersedeWall`), reference
lines **29384-29630**, as `cartalith-urban::growth`. 84 tests in the crate (up
from 69), dependencies unchanged (`cartalith-jsmath` +
`cartalith-rng`). **Not wired to anything.** `grow` is the heart of the whole
subsystem: an epoch loop that spends a population-derived street-length budget
on seeded candidate segments, branching off existing frontages at
near-perpendicular angles, with a decaying exploration share, a market-distance
density gradient, junction-angle and parallel-spacing rejection, bridgehead
rules for the far bank, and — behind an opt-in flag — successive wall
generations gated on real elapsed years. Everything downstream is accretion onto
what it lays down.

**The scope doc predicted this would be the hardest milestone and that its
golden would have to be a per-epoch graph hash so a divergence localises to an
epoch. Both held**, and **every one of the 60 goldens matched on the first run**
— the first 48 and the 12 the mutation sweep's second round added.

**The stated range understated the milestone by six lines at the start and got
its end right — the first of six whose end was right.** 29384-29389 is
`logisticRamp`'s own doc comment (the one flagging `k = 6.5` as tuned, not
measured), which by milestones 4/5/6's convention belongs here; 29630 is exactly
`supersedeWall`'s closing brace. **Six checked, six adjusted.**

**`buildWall` is milestone 10's, so the capture stubs it — on both sides.** It
arrives here as a `WallBuilder` trait object, and the golden capture stubs the
reference's own `buildWall` with a single anchored insertion into the sliced
text (frozen file untouched, asserted to match exactly once), so the fire epoch,
the M-GRW-2b age gate, the M-GRW-2a occupancy gate, the generation cap and the
supersession are all golden-verified now instead of in three milestones' time.
Said plainly: a stubbed builder never writes `wallState.ring` and never advances
`wallState.epoch`, so the supersession fixtures **preset** a ring and the age
gate is not re-armed between generations. Parity-neutral, but not the engine —
**milestone 10 should re-run all 60 with the real builder.** `ringCrossings`
(milestone 10's first function) and `distToLine` (milestone 9's first line) came
forward for the same reason and live in `growth` now.

**`WallState` carries only what this milestone touches**, exactly as milestone 2
left `Graph::_fromPaths` out until milestone 6 set it. `buildWall` writes nine
fields that are not modelled and `supersedeWall` copies six of them into its
history record: **milestone 10 must add them to `WallState` and to
`WallGeneration`'s copy list in the same pass**, or the history is silently
lossy and every structural test still passes.

**Findings.** `kept` is **dead** — pushed to and never read — and is omitted
rather than reproduced. The wet-crossing walk takes **six** samples, not five,
and the last is exactly `1.0`, the segment's own endpoint; the *reasoned* answer
(drift, `1.0000000000000002`, five samples) was wrong twice over, and the
accumulation turns out **not** to be load-bearing at these three constants —
`0.15 + k · 0.17` is bit-identical on all six. A **`NaN` slope does not reject**
(`NaN > 0.34` is false), so an all-`NaN` heightfield stops nothing; what it
poisons is `estimateCarryingCapacity`, which makes `maxR` `NaN` and therefore
**removes** the reach limit rather than stopping growth. `opts.rules ||
DEFAULT_RULES` is the **raw** table, milestone 4's correction now proved by
golden rather than by reading. `primEdges` is captured once per epoch, before
any street is placed. `wallState.generation || 1` reads a stored `0` as `1`.
`Math.max(3, Math.floor(epochs · 0.6))` needs **three** fixtures, not two — 3
and 5 epochs both fire at epoch 3, by different arms. A harbour with a one-point
quay is still a harbour and produces the no-harbour town. And `grow` always
enters from `generate()` with `ring: null` and a resolved rule set, because the
only pre-`grow` `buildWall` is in the **radial** branch, which does not call
`grow` at all — checked, because the first draft of that note said the opposite.

**Golden verification.** Same slice harness as milestones 3-6 verbatim, with
**three** anchored text edits this time (the `return {` replacement, the
`buildWall` stub, the per-epoch observer), each asserted to match exactly once.
Bit for bit through `to_bits`. `graph_hash` is the reference's own `fnv1a` over
its own canonical dump of every node and edge with each double as its exact 64
bits; the explicit node/edge dump is kept only under 170 edges so a failure is
readable, which took `golden.rs` from 785 KB to 244 KB — milestone 6's spatial-
index trade one scale up. `prov_hash` is a second `fnv1a` over every edge's
provenance string, pinning the Exploration/Densification split, the epoch stamp
and the ring road's interpolated `Math.round(fillFraction · 100)`.

**Two rounds of fixtures lost to milestone 5's rule, in two disguises.** First,
**the terrain rasters were in metres**: `site.height` reads the grid **raw** and
`site.slope` scales a per-metre central difference by **900**, so 40-95 m of
elevation gives slopes of 2 to 204 and `slope > 0.34` rejected every candidate
on every raster-backed site — fifteen fixtures grew nothing and the only two
that worked had no terrain raster. **Any raster-backed fixture in any later
milestone must be normalised** (this will hit milestones 10, 13 and 15). Second,
**a hand-drawn ring can never be 80% full**: the M-GRW-2a gate needs
`fillFraction >= 0.8` *and* `exteriorCount >= max(10, interior · 0.15)`, and
ellipses topped out at 0.44 while a sweep of scaled ones never passed 0.58; then
the first hull-derived attempt enclosed the finished town completely and left
`exteriorCount` at **zero**. What works is the town's own built-mass hull at
epoch 3, restricted to 260 m of the market and inflated 6% — roughly what
`buildWall` itself constructs.

**Mutation-tested: 214 mutations, 176 died, 38 survived**, every survivor
re-run in isolation and **zero false survivors** in either round; the first
sweep left 51. Round 2 added twelve fixtures aimed at what round 1 left
standing — including one **scanned** ring radius (592 m) whose
first supersession happens with an exterior count of *exactly* 10, and three
boundaries that are exact integer arithmetic rather than continuous distances
(`120 / 20 = 6.0`, `262.5 / 37.5 = 7.0`, and a closed square of four
exactly-38 m edges). Seven further survivors were turned into **executable
proofs** rather than paragraphs: a proof does not kill a mutant, so they are
still counted, but each now rests on an assertion — the carrying-capacity clamp
that cannot bind, the adjacency that cannot hold a dead edge, the angle wrap its
own following fold undoes, the twelve trig angles V8 and the platform agree on
(asserted *together with* >100 disagreements over arbitrary angles, so it cannot
be read as a licence elsewhere), the zero-area ring that cannot contain a node,
the hull whose winding never varies, and the two fallbacks assigned only when
they are not read.

**Corrections written forward**: milestone 9's range should start at **28967**
and it should not port `distToLine` again; milestone 10 should not port
`ringCrossings` again, must extend `WallState`/`WallGeneration` together, and
should re-run these 60 scenarios with the real builder; **milestone 14's stated
end overlapped this milestone and moves to 29382**; milestone 16 inherits that
`grow` always sees `ring: null` and a resolved rule set; and every later
milestone's raster fixtures must be normalised heightfields.

## Phase 4 — Asset Library (`ASSET_LIBRARY_SCOPE.md`, started 2026-08-17, done 2026-08-17)

Seven milestones, **all seven done (2026-08-17) — Phase 4 complete**. The scope doc carries the full investigation —
what an asset and an asset pack really are in the reference, the eight
families and their frozen slot vocabularies, how sprites actually reach the
map, the portable-vs-UI split with measured line counts, and what is
explicitly out of scope (the Library page UI, the sprite-sheet slicer modal,
the standalone pack compiler, and any wiring before milestone 7).

**Milestone 1 done** — `cartalith-assets`, the pack **manifest** layer:
data model, parser, validation warnings, schema-2 serialization. No images,
no archive, no renderer, no UI, and nothing in the workspace depends on it
yet — deliberately the piece every later milestone is defined against.
Golden-verified against the real reference implementation rather than
unit-tested by inspection, because a real headless execution path exists for
`parsePackCsv`/`parsePackManifest`/`packSummary`.

**Milestone 2 done** — pack `.zip` read/write, as `cartalith-assets::archive`
behind an on-by-default `zip` feature. The scope doc had deliberately left the
`cartalith-assets`-vs-`cartalith-io` placement open "until it starts"; reading
`cartalith-io` first is what decided it. Its whole zip surface is three
`zip`-crate calls, so milestone 1's "packs use the same `zipStore()` the world
save uses" implies a shared *crate*, not shared code; it is reading-only by
explicit scope, so a pack writer there would break that boundary; and the
dependency would point the wrong way, making the world-save loader drag in the
asset vocabulary. `default-features = false` still gives back exactly the
archive-free manifest model, and is tested that way.

The container is the crate's job; what is ported is the reference's own export
policy, which a plain `zip` call gets wrong by default — `.png` STORED and
everything else DEFLATED, timestamps frozen at 1980-01-01 so exports are
byte-reproducible, `pack.json` written last, names read verbatim (so zipping
the folder rather than its contents still fails exactly as the reference
fails), directory entries kept, and an unreadable compression method erroring
in the reference's own words. Two non-ports are stated rather than smuggled:
`zipStore`'s "only if it actually got smaller" fallback and `unzipStore`, both
browser-side concerns no reader can observe.

**Verified in both directions against a pack the reference itself exported.**
The harness ran the reference's own `PackManifestBuilder.build()` over its own
`FAMILIES`/`AssetDB` and its own `zipStore()` headlessly under Node's `vm`,
with only the canvas rasteriser and three DOM inputs stubbed — stated up front
in the test file rather than glossed. This port's read matches the reference's
`unzipAny` name for name and CRC-32 for CRC-32 and reproduces the exporter's
`pack.json` text byte for byte; its write reproduces entry order, method,
CRC-32, size and timestamps, and the bytes were fed back through the
reference's own `unzipAny` + `parsePackManifest`, which read all 18 entries
with identical payloads, summary and warnings. The two archives differ by 2
bytes in total. 14 new tests.

**Milestone 3 done** — scatter rules, as `cartalith-assets::scatter`: the
`ScatterRule` model that decides *where* an asset gets scattered, its ten slot
presets, the keyed rule table, weighted variant selection, and the hardened
normalizer. Still wired to nothing; the placement engine that consumes rules is
milestone 4.

**The three v1.27 hardening fixes were re-derived for Rust, not transcribed**,
with a test naming each. (1) A `NaN` `density` scattering on *every* cell is
**still a real hazard here, by the opposite IEEE rule** — JS reaches it through
`Math.min(1,NaN) === NaN`, Rust through `f64::min`'s NaN *absorption* giving
`1.0`, and `keep >= 1.0` is false anyway. (2) A `NaN` `spacing` collapsing the
relief bucket grid to 1×1 (an O(1) neighbour test becoming O(n²)) is real, and
Rust's `f64::max` would have masked it — so the `is_finite` check is kept
explicit rather than left to an IEEE corner, which fix 1 shows cannot be
trusted. (3) The `Object.assign` aliasing bug is **structurally unreachable**,
and *not* because of ownership: the bug needs defaults and untrusted input in
one mutable object, and here they are different *types* (`ScatterRule` with
`f64` fields vs. `serde_json::Value`), so a `"x"` can never be stored in the
field it would corrupt. No defensive code was written for it — the test pins
the reference's own probe case so a refactor toward a "merge" helper fails
loudly. A fourth guarantee the reference cannot have: `ScatterRule` implements
`Serialize` but **deliberately not `Deserialize`**, so the hardening is not
bypassable via `serde_json::from_str`.

**Golden-verified against the real reference**, same transient Node `vm`
technique. `pick_weighted_variant` is deterministic-hash-driven and diffed
exactly — 11 cases × 36 positions, index for index, including the three
degenerate weightings that must fall through to `pickIconVariant`'s untouched
v1.25 hash. 37 normalizer fixtures caught one real bug on the first run:
**`density`'s fallback is not symmetric with the other numeric fields** — an
absent `density` keeps the slot preset's own value (`cactus` stays 0.35) while
a *rejected* one lands on a literal `1`. 24 new tests. Three corrections to
milestone 4 recorded: it is not the first cross-crate dependency (this is —
`cartalith-noise`, for the variant hash); `pickIconVariant` and `spaceOf`
shipped here rather than there; and `biomes` is `Vec<f64>` because
`Number.isFinite` does not coerce.

**Milestone 4 done** — rule-driven icon placement, as
`cartalith-assets::placement`: `place_map_icons_ruled` (the reference's
`placeMapIconsRuled`), `icon_slot_for_item` with the `TREE_SLOT`/
`SCATTER_SLOT` legacy fallback maps, and `sprite_draw_rect`. The first real
placement golden-parity surface in this crate — positional and seeded, so it
diffs **exactly** rather than within a tolerance. Still wired to nothing.

**Both of milestone 4's own v1.27 fixes are structurally necessary in Rust,
not JS-only artifacts** — a real difference from milestone 3, where one of
three ported fixes turned out to be structurally unreachable here. (1) The
most-specific-wins priority sort: nothing about ownership or types makes
insertion-order dependence go away, a `Vec` iterates in build order exactly
like a JS array, so the sort is real ported logic. (2) `requireWetland` ANDed
with the biome test rather than replacing it: a straight transcription of the
old "replace" predicate would reproduce the bug faithfully in any language,
since it's an algorithm defect, not a consequence of JS coercion. Proven with
a hand-traceable fixture (`tGap=1` makes the scatter grid's own jitter
degenerate to zero, so `jx=gx,jy=gy` exactly): three cells, wetland+matching
biome / dry+matching biome / wetland+wrong biome, with the least-specific rule
inserted first — the winner comes out `wetland_grass` / `narrow_biome` /
`generic_land` regardless of insertion order, and reversing the whole rule
array doesn't change it.

**Golden-verified against the real reference**, same transient Node `vm`
technique. Broad sweeps over a synthetic 10×8 grid (a circular elevation peak,
a cycling biome pattern, a periodic wetland mask) across six seed/sea/density
configurations match cell-for-cell, key-for-key, and size-for-size to 1e-9 —
including one case exercising every rule family at once (both relief bands,
three different scatter specificities, and the always-empty `ghost_biome`
non-integer-biome probe placing nothing, confirming the `biome[i] as f64`
comparison). 23 new tests (12 unit + 11 golden).

**Milestone 5 done** — the Library model, as `cartalith-assets::library`:
`AssetDB` (frozen-vocabulary bootstrap, custom-slot add/rename/remove, lazy
scatter-rule attach, item store), `AssetCollections`, `run` (the reference's
`AssetValidator.run()`), and the `assetlib/library.json` record shape
(`LibraryFile`/`SlotRecord`/`ItemRecord`, parse + `to_library_json`/
`apply_library_file`). Pure data management, no images — every item's `hash`
is caller-supplied rather than computed from pixels, which is what keeps the
validator's duplicate-image detection fully testable without a decoder.
Still wired to nothing.

**Lines up with `SAVEFILE_COMPAT.md`'s existing "Asset Library payload,
nothing to deserialise into yet" note** — `LibraryFile` is that something
now, field order matching a real reference export exactly; `cartalith-io`
still deserialises nothing, by design, so that document needed no
correction.

**Two real corrections to `ASSET_LIBRARY_SCOPE.md`'s own §4, found by
reading rather than assumed**: (1) per-slot display *names* are not purely
presentational after all — `AssetValidator.run()`'s "Identical images"
warning renders `slot.name`, confirmed by a golden run
(`"Mountain#1 = Hill#1"`, not `mountain#1 = hill#1`), so the 65-entry
`mkSlots` title table is ported as `slot_title`; (2) the Library's own `poi`
vocabulary is **ten** slots (`lake`/`bridge` included), not the eight
`PACK_POI_SLOTS` milestone 1 ported for pack-import validation — both lists
are real and now both exist (`LIBRARY_POI_SLOTS`).

**The id-slugging/uid-collision hardening asked for by name, found and
ported.** `addCustomSlot` returns the *existing* slot on a uid collision
rather than duplicating or overwriting it; `renameCustomSlot` refuses a
colliding rename outright, keeping the old uid. Neither carries a
version-tagged reference comment like v1.27's fixes do — reported as a
finding, not a named historical fix — but both guard a real hazard:
untrusted, free-form user text (a custom slot's id) colliding on one slug.
A companion finding: two of `run`'s six checks ("Duplicate identifier",
"Invalid filename id") are structurally unreachable through the public API
in *both* languages, for a reason that is not "Rust's type system" — the
same shape of surprise as milestone 3's `Object.assign`-aliasing finding.
Ported anyway as real defence-in-depth. `tests/hardening_asset_db.rs`.

**Golden-verified against the real reference**, same transient Node `vm`
technique — twelve constructed library states for `AssetValidator.run()`
(empty, duplicate hashes across two/three slots, the grass-splat hint,
an empty custom slot, a stale collection reference reached the one real
way, a "kitchen sink" pinning warning order) plus five more for
`to_library_json()`'s shape (pack fields, tag-only inclusion for both
custom and frozen slots, exclusion when neither items nor tags are
present, collections round-tripping, the whole-library-empty `None` case).
Every case matched on the first run. 56 new tests (23 unit + 32
golden-parity + 7 hardening).

Two corrections to milestone 6's scope: its "`itemHash` duplicate
detection" is already implemented here (`duplicate_groups`/`slot_has_dupe`)
— milestone 6 only needs to supply a real hash from pixels; its per-item
transform data shape (`ItemTransform`) also already exists, so `fitToBottom`
remains milestone 6's own work but the field it writes does not need
redesigning. Milestone 6 also needs to wire real item restoration into
`apply_library_file`, deliberately left undone here since it needs decoded
pixels.

**Milestone 6 done** — image handling, as `cartalith-assets::raster`. First
milestone that touches pixels, and narrower than its own original
description once milestone 5's corrections above are read literally: the
transform *shape* (`ItemTransform`) and the duplicate-detection *machinery*
(`duplicate_groups`/`slot_has_dupe`) already existed. What was actually
missing, confirmed against the reference rather than assumed: real PNG
decode/encode, a real content hash from decoded pixels, `fitToBottom`'s and
`renderItem`'s transform math applied to actual pixels (not just represented
as a struct), thumbnail/pack-export bake, `finalizePackTexture`'s inverse
means, and wiring decoded items into library restoration.

**Crate work (`image`) plus a thin port, exactly as the scope doc's own
framing said.** `image = "0.25.10"`, `default-features = false`, only the
`png` feature — every asset this crate ever reads or writes is a PNG (packs,
and the project's own `assetlib/img/N.png`), so the rest of `image`'s format
zoo (gif/jpeg/webp/tiff/avif/exr/…) and its rayon/simd extras are dead
weight this crate never calls. Not present anywhere else in the workspace
before this milestone. `decode_png`/`encode_png` wrap `image`'s own
decode/PNG-encode directly; `item_hash`/`render_item` add a thin,
deliberate policy layer (the hash algorithm, the composite geometry) over
`image`'s resize/overlay primitives — the same shape of "crate for the
container, port for the policy" milestone 2 established for `zip`.

**`itemHash`'s real reference algorithm, read rather than assumed**:
`itemHash(img,w,h)` (line 26913) downsamples the source through
`ctx.drawImage(img,0,0,32,32)` on a 32×32 canvas, then runs a stride-7
FNV-1a variant (offset basis `0x811c9dc5`, prime `0x01000193`, 32-bit
wrapping multiply) over the resulting `ImageData`, appending `-{w}x{h}`
(the item's *original* dimensions, not the thumbnail's). Ported verbatim as
arithmetic — but **not** golden-verified against a captured browser hash,
and this is a real, checked decision rather than a gap: `_alExportEntries`
persists `{img,name,t}` per item with **no `hash` field** (line 27890), and
`_alImportProject` **recomputes** `hash:itemHash(img,w,h)` fresh after its
own decode (line 27922) rather than reading one back from a file — so no
process, browser or Rust, ever compares its hash against another process's.
`crate::library::ItemRecord` already reflected this before this milestone
ever named the reason: it shipped in milestone 5 with no `hash` field at
all. On top of that, the reference's own resample kernel is
implementation-defined per the HTML5 Canvas spec, so bit-exact parity was
never achievable even if the format required it — two browsers are not
obliged to agree on it either. `item_hash` is therefore real, deterministic
content hashing (`image`'s `Triangle` filter standing in for the
unspecified browser resample), verified with real unit tests for the one
property that actually matters: same decoded pixels in, same string out,
different pixels or different original dimensions, different string out.

**`finalizePackTexture`'s "inverse means", read literally rather than
assumed to be some reversed baking transform**: it is exactly what it says
— the mean of each of R/G/B across every pixel of a splat-channel texture,
clamped to never read as less than 1 (`Math.max(1,mean)`, so an
almost-black slot cannot blow the reciprocal past 1), then reciprocated.
Ported as `finalize_pack_texture_inv_mean(w,h,rgba) -> [f64;3]`, pure
arithmetic with no DOM dependency at all — so, unlike `item_hash`, this one
**is** golden-verified against the real reference, same transient Node `vm`
technique as every earlier milestone, six fixtures including the `n==0` and
mean-below-1-clamped cases, matched exactly. Used only by the `textures`
(splat-channel) family; `biomes`/`terrains` deliberately skip it (reference
line 12246, already documented in `ASSET_LIBRARY_SCOPE.md` §3) because they
are sampled as true colour, not splat-modulated. `fit_to_bottom` is the
milestone's other DOM-free function and is golden-verified alongside it —
seven fixtures spanning wide/tall/square items, non-1 scale, and pre-existing
pan values, matched exactly including one case with a `f64` fraction
(`106.66666666666666`).

**`render_item` ports the reference's own shared render core**
(`drawItemOnly`/`renderItem`, `ThumbnailRenderer`'s architecture comment:
"shared render core (thumbnails, inspector preview, export bake)") as one
function serving the same three uses here: scale-to-fit-`size` times the
item's own `scale`, centred, offset by `panX`/`panY`, opaque backdrops
pre-filled black before compositing (ground-texture bake) or left
transparent (sprites). The *geometry* — position, size, alpha compositing
via source-over — is exact; only the resampling kernel (`image`'s
`CatmullRom`, standing in for the reference's unspecified
`imageSmoothingQuality:'high'`) is not reference-identical, for the same
underlying reason `item_hash`'s is not. Real unit tests, not golden —
same DOM-dependency reasoning.

**`AssetDB::apply_library_file_with_items`** is the milestone-5-flagged
wrapper: calls `apply_library_file` (pack/collections/meta/rules and slot
creation, unchanged from milestone 5, still covered by its own tests), then
walks the parsed file's records again and, for each item whose PNG bytes the
caller supplies (keyed by `img` index — the caller's job to have read
`assetlib/img/<idx>.png` out of a project `.zip`, `cartalith-io`/save-format
territory, not this crate's), decodes it, computes a real `item_hash`, and
`add_item`s a `LibraryItem` built from the record's own `name`/`t`. A
missing byte entry or a decode failure for one item is skipped silently and
does not fail the rest of the restore — the reference's own
`try{...}catch(_){}` around this exact step (line 27920-27923).

**Scope check against the task's own seven-point list, confirmed accurate
after reading the reference**: decode/encode (crate work, done), per-item
transform math applied to pixels (`fit_to_bottom` mutates the transform;
`render_item` is what actually *applies* scale/pan to pixels — both done),
thumbnail and export bake (`render_item` is the reference's own single
shared function for both, done), `itemHash` duplicate detection (the
pixel-hash `item_hash` now feeds milestone 5's pre-existing
`duplicate_groups`/`slot_has_dupe`, done), `finalizePackTexture`'s inverse
means (done, and confirmed literal — not a reversed bake transform).
Library restoration end-to-end (`apply_library_file_with_items`, done).

Pack-zip-into-Library import (`AssetImporter.importPackZip`, reference line
27067 — decoding a whole external pack's manifest-declared images straight
into `AssetDB`, as opposed to restoring a previously-exported project) was
**deliberately not built this pass** — the task's own seven-point list
names project restoration (`_alImportProject`'s shape), not pack import
(`importPackZip`), and building it without being asked would be scope creep
beyond a narrowly-scoped milestone. It is a real, small remaining gap
(`PackManifest` + `PackEntries` + this milestone's `decode_png`/`item_hash`/
`fit_to_bottom` are already exactly the pieces it would compose) worth
naming for whoever picks up milestone 7 or a later Library-import UI pass —
not a correction to milestone 7's own scope, which is renderer/Godot
integration and does not need it.

15 new tests (10 raster unit + 3 library unit + 2 golden-parity), still
wired to nothing.

**Milestone 7 done (2026-08-17) — renderer + Godot integration, closing Phase 4.**
New `cartalith-godot::pack` module — the first thing in the workspace to
depend on `cartalith-assets` (a new `Cargo.toml` dependency; the crate's own
doc comment said "nothing depends on this yet" until now). Two of the three
named surfaces are real: **sprite compositing** (`drawMapIcons`'
Y-sorted painter's pass, real pack art via a bilinear blit plus a real
per-slot procedural glyph fallback for all ten `PACK_ICON_SLOTS` shapes —
mountain/hill/six tree kinds/cactus/boulder, with "shrub" doubling as the
reference's own documented catch-all for an uncovered custom asset), and
**ground-texture splat** (the six `SPLAT_PAINT_SLOTS` channels, blended into
`land_color` via the exact `materialWeights` fractions and procedural ramp
colours already computed there — no new logic, a read-only consumer of both).

**The third named surface — the two "painted layers" (`_paintedTex`'s
`biomes`/`terrains` families, the Cartography paint-brush biome/terrain
override) — is honestly out of scope this pass**, not glossed over: `pBio`/
`pTer` are indices into `state.cartoPaint.biome`/`.terrain`, sparse arrays a
manual paint-brush tool populates, and this port has never ported that tool
— there is no producer of a painted-cell array anywhere in the workspace,
and building one is a real, separate UI+state effort the milestone's own
"no GUI controls" boundary rules out. `pack.rs`'s own doc comment records
this as a named follow-up for whoever ports the Cartography paint-brush
tool, not a silent gap.

**Real findings, not assumptions**: `state.viz.icons` defaults `false` in
the reference (icons are opt-in, same as every other `state.viz.*`
stretch feature) — so a pack-less *or* icon-toggle-off render was already
bit-identical before this milestone touched anything, and stays so:
`current_scatter_rules` returns `None` (no configured rules) whenever no
pack supplies real icon art, which is `composite_map_icons`'s own early
return. Splat is the opposite shape: `state.viz.splat` defaults `0.7`,
gated *only* by `assetPack.texAny` — real and on by default the instant a
pack with real ground textures loads, no toggle involved. Both are
genuinely additive/opt-in (no JS-parity gate needed, per the task's own
"judge from what you find" instruction) since there is no pack-less
version of "blend in a texture that doesn't exist" to stay bit-identical
with — confirmed by `golden_parity_render.rs` passing unmodified at its
original `1e-4` tolerance (`RenderCtx.splat` stays `None` on that path,
`with_splat` never called).

A real biome raster and wetland mask are derived at render time from
already-generated temperature/rainfall/height fields (`cartalith_civ::
classify_biome`, already golden-verified elsewhere in the workspace, plus a
`buildWetlandMask`-equivalent) — presentation-side computation, no new
world-generation data, same category `material_weights` already is. One
honest simplification: water is always `BIOME_OCEAN`, since this port has
never built the lake/ocean flood-fill classifier `buildBiomeRaster` uses;
none of the ten frozen icon presets target the lake biome index, so this
costs nothing observable.

Real, permanent new API surface: `WorldGen::load_asset_pack(path) -> bool`
(reads a native filesystem path via `cartalith_assets::read_pack`, same
convention as `load_save`) and `WorldGen::has_asset_pack() -> bool`. No
GDScript UI calls either — this port ships no default pack (confirmed:
nothing in `godot-project/` bundles pack art), so both are real, dormant
plumbing for a future importer, exactly as the milestone's own "wire a
temporary load path if none exists" instruction allowed, kept as shipped
code rather than thrown away after verification.

**Verified three ways.** Unit/integration: a new `tests/pack_compositing.rs`
loads the real `reference_pack.zip` fixture milestone 2 verified against the
reference's own exporter (reused, not reinvented) and proves, on a
synthetic world, that (a) real sprite art actually blits (a mountain relief
peak), (b) the procedural glyph fallback actually fires for a biome the
fixture has no art for, and (c) a pack with no icon slots at all places
nothing — the same "keeps `placeMapIcons` on the legacy/no-op path"
condition `current_scatter_rules`'s own doc comment names. Static:
`cargo build -p cartalith-godot`/`--workspace`, `cargo test --workspace`
(zero regressions, `golden_parity_render.rs` unmodified and still passing,
new tests included), `cargo clippy -p cartalith-godot -p cartalith-assets
--all-targets` clean (one small refactor along the way — the rasterizer's
loose `bytes/gw/gh` triples became a `Canvas` struct, both for clippy's
`too_many_arguments` and because it reads better), `godot4 --headless
--quit main.tscn` clean. Real windowed: launched the actual
`Godot_v4.7.1-stable_win64.exe`, generated a real 512² world, called
`load_asset_pack` on the real fixture (temporary `main.gd` debug calls only,
reverted before commit — the shipped diff carries no GDScript changes at
all), and saved the native output `Image` directly to disk to inspect at
full resolution rather than a scaled-down window screenshot. **Confirmed by
actually looking at it**: a sharp-edged, flat-coloured rectangular block
sits on land exactly where a relief-mode mountain would place one — real
pack sprite art, not a procedural blend (which is always noisy/gradient,
never a hard-edged rectangle); a large irregular checkerboard-patterned
region follows real land-material boundaries rather than sitting in a fixed
box — real per-pixel splat sampling, not a sprite; and small soft-edged
translucent blobs appear elsewhere on plain terrain, consistent with the
procedural glyph fallback rendering where the fixture has no matching art.

**Phase 4 is genuinely complete — all seven milestones done.** Checked
honestly against `ASSET_LIBRARY_SCOPE.md` §8's own "done means", which was
written specifically to give this phase an operational finish line beyond
`ROADMAP.md`'s one-sentence description: "a real `.zip` asset pack authored
outside the app can be imported, validated with the reference's own
warnings, and rendered onto the map — sprites for the slots it carries,
procedural art for the slots it does not — with a pack-less render staying
bit-identical to today's." That bar is met. The one explicit carve-out in
that same sentence — "the Library workspace that *authors* such a pack is a
separate, later GUI effort tracked in `GUI_SHELL_SCOPE.md`" — is not part of
Phase 4's own definition of done, so its absence is not a gap in this row;
it is `GUI_SHELL_SCOPE.md`'s own future work, same as the Cartography
paint-brush tool this milestone found and named above.

## GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`, `HARDWARE_ACCELERATION.md`)

**Done, 2026-08-16.** Piloted a standalone `wgpu` compute path (new crate
`cartalith-gpu`, no `gdext` dependency) on one kernel: `cartalith_noise::vnoise`.
Findings:

- **The `wgpu` hardware path itself works cleanly** on this session's real
  hardware (AMD Radeon RX 7800 XT, Vulkan backend, discrete GPU) —
  instance/adapter/device creation, conservative limits, shader compile,
  dispatch, readback all function correctly.
- **This specific formula is not GPU-viable in `f32`** — `hash`'s
  f64-magnitude-dependent rounding (its own doc comment already flagged
  ~2^61 intermediate products, past `f64`'s own exact range) does not
  survive a portable `f32` WGSL port: 100% of cells diverge at 128×128,
  max abs diff `0.93` on a `[0,1]` output. Measured, not assumed.
  `self_test` (the real correctness gate) correctly reports FAIL and the
  CPU fallback is correctly used instead.
- **`f64` in WGSL is a dead end on this toolchain regardless of hardware
  support** — `wgpu::Features::SHADER_F64` is present on this adapter, but
  naga (wgpu 30's WGSL compiler) has no `enable f64;` implementation at
  all. A real, precise finding, not a shrug.
- **Real GPU-vs-CPU timing measured**: GPU loses at 128×128 (dispatch
  overhead dominates, 0.20×) but wins increasingly at scale — 4.46× at
  512×512, 15.65× at 1024×1024, 19.55× at 2048×2048.
- **Verdict**: the `wgpu` path is a real, viable option for *future*
  candidate kernels that don't share `hash`'s f64-precision dependency
  (e.g. presentation-layer work — hillshade/AO synthesis, biome
  classification — pure functions of already-computed fields). Not this
  kernel, not right now, and no wider `HARDWARE_ACCELERATION.md` adoption
  decision has been made — this pilot answers one narrow question, per its
  own scope doc's explicit boundary.

See `CHANGELOG.md`'s "GPU-compute pilot" entry for the full numbers and
reasoning. Nothing outside `GPU_COMPUTE_PILOT_SCOPE.md`'s "In scope" list
was implemented (no capability-tier classifier, no diagnostics panel, no
telemetry system, no tiled compute) — all still deferred exactly as that
document scoped them.

## GPU layer integration (`GPU_LAYER_INTEGRATION_SCOPE.md`)

Follow-up to the pilot above, prompted by the owner's explicit "connect
GPU for each layer" directive (2026-08-16) plus a real architectural
correction: Cartalith generates a **static map from a one-shot batch
simulation**, not a continuously recomputing app — significantly narrows
`HARDWARE_ACCELERATION.md`'s scheduling/priority/thermal sections (see
`GPU_LAYER_INTEGRATION_SCOPE.md`'s own annotation).

**Milestone 1 — GPU-safe noise redesign: done (2026-08-16).** The pilot's
"not viable" verdict on `hash` was specifically about reproducing JS's
exact double-precision rounding, not about GPU noise being impossible.
`cartalith_noise::gpu_hash`/`gpu_vnoise` (PCG3D-based, pure `u32`
wrapping arithmetic, cited: Jarzynski & Olano, JCGT 2020) verified against
their own GPU counterpart (not JS — `DECISIONS.md` §7a) at 512×512: 0
mismatches at `1e-5` tolerance, max diff 1.28e-6. Real timing: 2.85× at
512², 10.39× at 1024², 11.94× at 2048² (the port's real default
resolution). `hash`/`vnoise` themselves untouched — every existing
JS-matching golden test still passes unmodified. See `CHANGELOG.md`'s
"GPU-safe noise redesign" entry for the full record.

**Milestone 2 — domain warp + crustal heterogeneity on GPU: done
(2026-08-16).** `cartalith_noise::gpu_fbm` (6-octave combinator over
`gpu_vnoise`) plus `cartalith-gpu`'s `gpu_warp.wgsl`/
`gpu_heterogeneity.wgsl`. Non-`world` branch only (periodic/`pfbm`
GPU equivalent deferred). `gpu_heterogeneity` (one `gpu_fbm` call/cell)
matches its CPU twin at `1e-5`, 0 mismatches at 512×512 — confirms
`gpu_fbm` itself is clean. `gpu_warp` (chains two nested `gpu_fbm`
evaluations) needed its own, separately-justified `WARP_TOLERANCE=2e-4`
— a real, measured, structural effect (float-scheduling residue from the
first evaluation amplified through the second), not a loosened test.
Real timing: `gpu_warp` up to 80× at 2048² (24 octave-calls/cell — even
better than milestone 1's bare noise, since GPU's fixed dispatch
overhead amortizes further against costlier per-cell work);
`gpu_heterogeneity` up to 16.7×. `compute_warp`/`compute_heterogeneity`
(CPU, JS-matching) untouched, their own golden-parity tests unaffected.
Found (not introduced): `cargo test -p cartalith-gpu` alone can hit a
flaky driver-level crash under parallel GPU-context churn — reliable
with `--test-threads=1` or as part of a full workspace run. See
`CHANGELOG.md`'s "GPU layer integration milestone 2" entry.

**Milestone 3 — the height formula (`compute_height`) on GPU: done
(2026-08-16).** Treats upstream fields (base/stress/flex/hetero/age/
warp/oro) as opaque GPU buffers — plate assignment/stress/flexure/
orogeny's own GPU portability is deliberately NOT this milestone's scope.
Added `cartalith_noise::gpu_ridged` (the noise-combinator gap milestone 2
anticipated) plus `cartalith-gpu`'s `gpu_height.wgsl`/`dispatch_gpu_height`.
Both `ridged=false`/`true` verified against a CPU twin at 512×512: 0
mismatches, max diff `1.19e-7` — essentially `f32` machine epsilon, given
its own tight `HEIGHT_TOLERANCE` (this kernel has one noise call/cell,
`gpu_heterogeneity`'s clean shape, not `gpu_warp`'s compounding one).
`oro`'s absence changes the formula (not an additive no-op like
warp_x/warp_y) — a dedicated regression test proves the branch is
genuinely wired. `init_gpu_with` gained an automatic storage-buffer-limit
derivation from each kernel's own layout (this kernel needs 9, past
`downlevel_defaults()`'s baseline) — self-contained, existing call sites
unaffected. Real timing: 512²/1024²/2048² at 5.17×/8.13×/4.84× (the
1024²→2048² drop reported honestly, not investigated — possibly memory-
bandwidth-bound at 8 input buffers). `compute_height` (CPU) untouched,
its golden-parity tests unaffected. Also fixed a doc-merge artifact in
`GPU_LAYER_INTEGRATION_SCOPE.md` (milestone 2's own completion note had
been misplaced under milestone 3's heading). See `CHANGELOG.md`'s "GPU
layer integration milestone 3" entry for the full record.

**Milestone 4 — `gauss_blur` + `compute_resistance` on GPU: done
(2026-08-16), genuine three-way JS/CPU/GPU parity.** Unlike milestones
1-3 (all noise-driven, all only GPU-vs-CPU-twin verifiable per
`DECISIONS.md` §7c), neither of these touches noise — verified directly
against the real, untouched `cartalith_terrain::gauss_blur`/
`compute_resistance` (`cartalith-terrain` added as a `cartalith-gpu`
dev-dependency, test-only). `gauss_blur`: max observed divergence
`7.15e-7` at 512×512 across three radius/wrap configs (a direct-sum-in-f32
GPU kernel vs. the CPU's running-sum-in-f64 — the real precision-regime
gap turned out negligible for a bounded linear sum, unlike noise's
chaotic compounding). `compute_resistance`: max divergence `5.96e-8`,
essentially `f32` epsilon. New `GpuBlurContext` (two pipelines — `box_h`/
`box_v` — sharing one device, since `gauss_blur`'s 3-pass structure needs
both kernels reading what the other just wrote). `compute_flexure`
(a thin `gauss_blur`-plus-mask-plus-normalize wrapper) checked, not
ported this pass — noted for whoever wires `gauss_blur` into it.

**Real, honestly-reported timing** — not every kernel wins: `gauss_blur`
20.49× at 2048² (a real win), but `compute_resistance` **loses to CPU at
every size tested, including 2048² (0.38×)** — its formula is too trivial
for GPU dispatch overhead to ever amortize, exactly the case
`HARDWARE_ACCELERATION.md` §6 already warns about. Recorded plainly, not
hidden — not every candidate should actually move to GPU even once it's
technically been verified there.

**Milestone 5 — plate assignment (JFA) on GPU: done (2026-08-16), GPU
beats brute-force exactly.** Confirmed the JFA hypothesis: `assign_plates`
is a textbook Jump Flooding Algorithm, but a specific **in-place-mutation**
variant (a cell can see another cell's update from earlier in the *same*
pass, not just the previous pass's frozen state) — a real algorithm
variant, not an implementation detail. `gpu_jfa_plates.wgsl` implements
the standard **double-buffered** JFA instead (the textbook, race-free GPU
formulation) and doesn't attempt to match the CPU's in-place answer
cell-for-cell — verified against **brute-force exact-nearest-plate ground
truth** instead, per the scope doc's own instruction to investigate which
framing fits rather than assume. Result across three configurations
(512×512 at 14/40 plates, 1024×768 at 22 plates): **GPU JFA matched
ground truth exactly, 0 mismatches, every time.** CPU's in-place JFA had a
tiny, consistent, expected approximation error (1-2 cells out of
262k-786k) against the same truth — a known JFA property, not a bug in
either variant. Also investigated `compute_stress`: confirmed genuinely
harder, not a same-shape sibling — its main loop is a *scatter* (writes to
both a cell and its neighbour in one pass), a real cross-thread write
hazard WGSL's core atomics don't cover, needing a gather reformulation
and its own re-verification. Deferred to its own future milestone, not
bundled in.

**Real timing** (128/512/1024/2048, 24 plates): GPU wins even at 128×128
(1.63×) — the first GPU milestone to win at that size, since JFA's
`log2(size)`-pass structure means real compute work happens even on a
small grid. Scaling to 11.50×/18.22×/15.65× at 512²/1024²/2048² (the last
a real, honestly-reported dip, not investigated). See `CHANGELOG.md`'s
"GPU layer integration milestone 5" entry for the full record.

**Milestone 6 (orogeny sub-investigation) — confirmed poor GPU fit
(2026-08-16).** Orogeny's graph-tracing (`trace_boundaries`/
`tag_boundary_types`/`build_orogeny_field`) is sequential graph
traversal, the same poor-fit category as `compute_stress`'s scatter
hazard and Phase 2's Dijkstra/MST road networks — informational finding,
no kernel built.

**Milestone 6 — first real partial-GPU pipeline integration: done
(2026-08-16), the architecturally significant one.** Every prior
milestone (1-5) built a standalone, never-called kernel — generating a
map has been CPU-only this whole time not because GPU didn't work, but
because nothing wired it into `generate_terrain` itself. This milestone
is that wiring: a new opt-in `WorldParams.use_gpu` flag (default
`false`) runs domain warp, crustal heterogeneity, plate assignment, and
the flexure/base-field blur on GPU inside the real pipeline, with
per-stage CPU fallback on any GPU failure (never a panic) and a new
`WorldState.gpu_stages_used` field so callers can tell which path
actually ran. **Headline result: with the flag at its default `false`,
`generate_terrain`'s output is unchanged** — `cargo test --workspace`
100% green, every existing golden-parity test (this pilot's whole
foundation) unmodified. Closed a real gap along the way: milestones
2/4/5's own dispatch functions were private, unreachable outside
`cartalith-gpu` — four new public wrappers fixed that. **Real end-to-end
timing is the honest, sobering number this milestone adds**: each GPU
wrapper creates its own fresh `GpuContext` per call, so at every size
this pilot ships at by default (128×128 through 1024×1024), the
`use_gpu=true` path is *slower* than CPU (up to ~16× at 128×128),
dominated by ~1.3-1.4s of fixed context-creation overhead that only the
largest tested size (2048×2048) outruns, and only by 19%. Context
reuse/caching across the four stages is flagged as the clear next
optimization, not attempted this pass. See `GPU_LAYER_INTEGRATION_
SCOPE.md`'s milestone 6 "Done." section and `CHANGELOG.md`'s "GPU layer
integration milestone 6" entry for the full numbers.

**Milestone 7 — climate's wind/rain loop on GPU: done (2026-08-17), a
real loss even with milestone 8's own fix applied from the start.**
Built `gpu_weather.wgsl` (`evap_main`/`advect_main`/`deposit_main`) using
the shared-`GpuDevice` pattern from day one (milestone 7 landed after 8,
no reason to repeat 6's original per-call-context mistake). Required a
real refactor first: `simulate_weather`'s previously-inline setup/
teardown extracted into new `pub fn build_weather_grid`/`finish_weather_
grid` (`cartalith-climate`) — pure extraction, `golden_parity_weather.rs`
unchanged. **Correctness**: no noise dependency, verified directly
against the real CPU `simulate_weather` at production `iters=70`: max
abs diff `1.79e-7`, essentially f32 epsilon — 70 iterations of gather/
advect/deposit didn't compound meaningfully (bounded arithmetic, unlike
nested noise). **Real timing, the honest finding**: this kernel's
working set is capped at `min(gw,240)` and stops growing with map
resolution past that — unlike every other GPU-wired stage. Measured at
its real production size (240×240, 70 iters, from a real 2048² map):
**GPU 23.8ms vs CPU 22.2ms, 0.93× — GPU loses**, even with milestone 8's
fix. 210 dispatches (70×3) against a 57,600-cell working set is too
little work to amortize even the remaining per-dispatch overhead once
context-creation stops dominating. Joins `compute_resistance` (milestone
4, 0.38×) as a second confirmed "verified on GPU, shouldn't run there"
case — a different structural reason (dispatch-count-dominated, not
formula-triviality-dominated). **Wired anyway** behind `p.use_gpu` for
architectural consistency (`"weather"` joins `gpu_stages_used`), expected
to keep losing regardless of map size. Found and fixed a real pre-
existing bug along the way: `cartalith-civ`/`cartalith-engine`'s two
`examples/timing_bench.rs` (from the CPU-multithreading milestones)
collided at the same output path, breaking `cargo test --workspace` —
renamed the civ one to `civ_timing_bench.rs`. See `GPU_LAYER_INTEGRATION_
SCOPE.md`'s milestone 7 section and `CHANGELOG.md` for the full record.

Per the scope doc's own feasibility table: the remaining graph/sequential
algorithms (water-body priority-flood's depression-fill half, Dijkstra/MST
road networks, orogeny, `compute_stress`'s scatter) remain a poor GPU fit
without real algorithmic redesign. Flow accumulation, the flagship entry on
that list, is no longer among them — see milestone 9 below.

**Milestone 8 — GPU context reuse across `generate_terrain`'s stages:
done (2026-08-17).** Picked up milestone 6's own flagged next
optimization directly. New `cartalith-gpu::GpuDevice` (adapter+device+
queue, no pipeline) + `init_gpu_shared_device()`, built once per
`generate_terrain(use_gpu=true)` call and threaded through all five GPU
call sites (warp, heterogeneity, plate assignment, two `gauss_blur`
calls) via new `_with(gpu: &GpuDevice)` pipeline builders and wrapper
functions, instead of each stage independently paying its own ~1.3-1.4s
adapter/device handshake. Confirmed (not assumed) `wgpu::Device`/`Queue`
are cheap `Clone` handles by reading `wgpu` 30.0.0's own source before
relying on it. Original standalone functions byte-untouched — every
milestone 1-6 test still exercises the identical code path. **CPU path
confirmed unchanged**: `cargo test --workspace` 0 failures, every
golden-parity test unmodified. **Real result: GPU now beats CPU
starting at 1024×1024** (128²: 1.44s→813ms, 512²: 1.46s→689ms, 1024²:
2.32s→1.39s and crosses from a 0.78× loss to a **1.14× win**, 2048²:
6.03s→5.92s at ~0.98× — reported honestly as likely single-run
variance rather than a regression, per the benchmark's own "not
averaged" caveat, not re-run to chase a better number). See
`GPU_LAYER_INTEGRATION_SCOPE.md`'s milestone 8 section and
`CHANGELOG.md`'s own entry for the full record.

**Milestone 9 — flow accumulation on GPU: done (2026-08-17), the first
genuinely sequential algorithm redesigned rather than ported.** The
owner's "do the algorithms for the GPU" directive, aimed at the one row
this document's own feasibility table had deferred longest.
`compute_flow` sorts every cell by descending height then walks that
order — but those are separable: **flow direction** is a pure function of
the height field (never reads `acc`, so the ordering is irrelevant —
embarrassingly parallel), and **accumulation** over the resulting
receiver forest is a subtree sum, which parallelizes by **pointer
doubling** in `ceil(log2(n))` rounds (22 at 2048²) rather than the
thousands a naive fixpoint iteration would need or the global sort the
CPU pays. Qin & Zhan 2012 / the 2016 RUSLE paper /
`HETEROGENEOUS_COMPUTE_RESEARCH.md` §48-49's own decomposition, applied
for real. Accumulation is `atomic<u32>` **fixed point**, not floats:
WGSL has no atomic float add, and a compare-exchange emulation would be
non-deterministic run to run, whereas integer addition is exactly
order-independent *and* bit-reproducible.

**Correctness**: flow directions **0 mismatches out of 262,144** (both
world-wrap modes, two roughness regimes). Accumulation against the real,
untouched `cartalith_hydrology::compute_flow` is **bit-exact for
`use_rain=false`** (the pipeline's first call), and for discharge seeding
diverges only by seed quantization — with the *opposite* shape to the
CPU's error (worst at tiny accumulations, shrinking as accumulation
grows, because the GPU rounds each seed once and is exact thereafter
while the CPU rounds to `f32` on every one of thousands of writes). At
and above `river_flow_thresh`, the only regime anything downstream
distinguishes: **1.3e-4 relative at 512², 3.3e-4 at 1024²**.

**The measured downstream effect is the real headline** — this is the
first GPU kernel here that is not a leaf computation, so the divergence
was traced through to the civilisation layer, holding terrain fixed:
**river network zero difference** (identical river-cell counts, 0
channel-mask cells, 0 channel receivers, 0 Strahler-order cells
differing) and **settlements zero difference** (`find_settlement_seeds`
returns the same count *and the same positions* — 104/104 at 512²,
125/125 at 1024², zero seeds moved; the suitability raster differs only
in its last `f32` digits, max 1.3e-5).

**Real timing**: isolated kernel 0.20× at 128² (GPU loses — the round
count barely falls with grid size, so a small grid pays nearly the same
dispatch count over far less work), 4.6× at 512², 10.4× at 1024², **15.5×
at 2048²** (31.5ms vs 488.9ms). End-to-end `generate_terrain` ratio moves
0.11×→0.16× / 0.76×→0.83× / 1.14×→**1.36×** / 0.98×→**1.74×** across
128²/512²/1024²/2048² — the largest single-milestone shift this effort
has produced, since `compute_flow` is called up to four times per
generation. Wired behind `p.use_gpu` with per-stage CPU fallback,
`"flow"` in `gpu_stages_used`, `compute_flow` itself byte-untouched,
`cargo test --workspace` 0 failures and 0 modified tests.

**Two honest "shouldn't run on GPU" findings** from reading the real
code: `build_water_bodies`' depression-fill half is a global priority
queue whose parallel formulations trade O(longest ascending path)
iterations for parallelism, with no pointer structure to double (its
connected-components half *is* tractable, and its exact CPU answer even
reproducible) — and it costs only ~92ms at 1024², an order of magnitude
below what flow accumulation was costing. `road_dijkstra` should stay on
CPU: its `prev` array literally *is* the road geometry and is
settle-order-dependent on ties (every GPU alternative would move roads),
and it is already called many independent times over a small downsampled
grid at four still-sequential `.iter().map()` call sites — the available
parallelism is across sources on CPU, not within one traversal on GPU.
See `GPU_LAYER_INTEGRATION_SCOPE.md`'s milestone 9 section and
`CHANGELOG.md`'s own entry for the full record.

**Multi-GPU: done (2026-08-20)** — owner instruction, closing
`GUI_GAP_REGISTER.md` **PR-01/PR-02/PR-04/PR-05** and omission **O3**, and
cashing in `CPU_MULTITHREADING_SCOPE.md`'s recorded integrated-GPU idea. New
`cartalith-gpu/src/multi.rs`. **Enumerable on this machine**: 3 physical
GPUs from 6 adapter rows — RX 7800 XT (discrete, Vulkan; also Dx12 and Gl),
integrated Radeon (Vulkan; also Dx12), Microsoft Basic Render Driver
(software, listed but never selectable). The Gl row reports
`vendor = device = 0`, so grouping keys on PCI identity with an unambiguous
name fallback — keying on name alone would have merged two identical cards,
the canonical multi-GPU rig. **Real**: device enumeration + selection
(each device provably opens), `single device`, `split tiles`, a VRAM cap
over a documented working-set upper bound, `CPU tile pass` and `fail with
error`. **Honestly disabled, refused at the API**: `alternate frames`
(§2.5's own note says it only helps the 3D viewport; there is none) and
`reduce working res` (nothing here resamples a stage down and back up).
**Not implementable, and not faked**: §2.5's `71%` live utilisation and its
"VRAM budget default 75 % of the smallest active device" — `wgpu` 30
exposes no system-wide utilisation and no VRAM size on any backend
(`Adapter::limits()` is an API limit: same 2 GB `max_buffer_size` reported
for the 16 GB card and the shared-memory iGPU). What is shown instead is
real: `Device::generate_allocator_report()`, this app's own allocations.
**Split tiles covers exactly one stage** — `gpu_warp`, the only kernel here
reading nothing outside its own cell (blur needs a halo; JFA, flow and
weather all read across the grid) — and the measured verdict is
**1.22-1.54× at 4096², 0.73-0.81× at 2048² and below**, so the shipped
default is `single device`, not §2.5's `split tiles`, with the numbers in
the menu tooltip. Band sizes come from measured per-device throughput
(integrated = 0.17 × discrete here), not a guess. Determinism: bit-exact
band-vs-whole-grid on one device; ~4e-6 relative across two different
devices, `DECISIONS.md` §7a one level finer. `use_gpu = false` untouched.
Still open in §2.5: **PR-03**, CPU worker threads. Full record in
`HARDWARE_ACCELERATION.md`'s 2026-08-20 section and `CHANGELOG.md`.

## Memory optimization (`MEMORY_OPTIMIZATION_SCOPE.md`, done 2026-08-16)

Owner-reported "consumes a ton of memory" on generation, investigated
with real measurement, not assumption. Confirmed dominant contributor:
`ResourcePotentials` (`cartalith-civ`) held six resource fields
(clay/buildstone/flint/obsidian/sulfur/alum, ~96 MB at 2048²) that
nothing in the pipeline reads. Fixed by freeing them immediately after
computation in `compute_civilisation()`. Real before/after at 2048²:
peak 1,445-1,653 MB → 1,434.5-1,501.8 MB, steady-state 689-691 MB →
678.0-679.9 MB, no persistent leak (re-confirmed). A real but modest
win — the bulk of the remaining ~1.1-1.3 GB transient peak above
baseline is `cartalith-terrain`/`-climate`/`-erosion`/`-hydrology`'s own
~96 full-grid allocations, not instrumented stage-by-stage in this
pass; a real candidate for a follow-up if the owner wants the peak
pushed further. Full numbers in `cartalith-native/docs/CHANGELOG.md`.

## CPU multithreading (`CPU_MULTITHREADING_SCOPE.md`, milestone 1 done 2026-08-16)

Owner-reported "doesn't seem to fully use the cpu" (16 logical cores,
generation used effectively one -- confirmed, `rayon` was not a
dependency anywhere in the workspace before this). Unlike GPU work,
needs no `DECISIONS.md` §7a carve-out: parallelizing an existing
per-cell loop preserves golden-parity output exactly, bit-for-bit, not
within a tolerance -- confirmed by every existing test for the touched
functions passing completely unmodified, plus a full `cargo test
--workspace` (0 failures, 0 modified tests).

**Milestone 1 — `cartalith-terrain` (done 2026-08-16).** Added
`rayon = "1"`; parallelized `compute_warp`, `compute_heterogeneity`
(the fbm loop; the trailing reduction stayed sequential, not the
bottleneck), `compute_height`, `compute_resistance`, and `gauss_blur`'s
`box_h`/`box_v`. Real timing (16-core machine, best of 3, seed 12345):
128² 0.0973s→0.0936s (~1.04x), 512² 0.6019s→0.4859s (~1.24x), 1024²
1.8328s→1.3143s (~1.39x), 2048² 7.0670s→5.1071s (~1.38x). Honest,
modest, not near 16x -- Amdahl's law: plate seeding/Lloyd relaxation,
JFA plate assignment, `compute_stress`, `build_age_field`, and all of
climate/erosion/hydrology stay fully sequential this pass and set the
real ceiling measured. Full record and per-function reasoning:
`cartalith-native/docs/CHANGELOG.md`'s "CPU multithreading milestone 1"
entry.

**Milestone 2 — `cartalith-civ` (done 2026-08-17).** Added `rayon` to
`cartalith-civ`; parallelized 16 functions (`build_lithology`,
`build_slope_field`, `build_soil_fertility`, `build_water_access`,
`build_biome_raster`, `build_wetland_mask`, `build_carrying_capacity`,
`build_npp`, `estimate_regional_density_km2`, `build_resource_
potentials`'s 15-field main loop, `apply_resource_scarcity`, `build_
raw_slope_field`, `build_route_corridors`, `build_landmass_quality`'s
final fold, `build_flood_field`, `build_settlement_suitability`,
`build_travel_cost`, `assign_territory`'s inner cell loop). Left
sequential and why: `chamfer_dist`/`jfa_dist` (wavefront/iterative,
not independent), `build_water_bodies`/`label_land_components`/
`build_landmass_quality`'s flood-fill (connected components),
`road_dijkstra`/`build_road_network`/`civ_hierarchical_network_
topology`/`civ_sea_routes`/`civ_consolidate_and_smooth_ways`
(graph/Dijkstra/MST), settlement placement/naming/villages (RNG-order,
not grid-shaped), `fresh_river_order` (delegates to
`cartalith-hydrology`). Golden-parity exact-unchanged: every existing
`cartalith-civ` test passes unmodified, full `cargo test --workspace`
68 suites 0 failures. Real timing (new `cartalith-civ/examples/
civ_timing_bench.rs` -- renamed 2026-08-17 from `timing_bench.rs`,
which collided with `cartalith-engine`'s own example of the same name,
see `CPU_MULTITHREADING_SCOPE.md` -- chaining this crate's own real per-cell pipeline
since `compute_civilisation()` itself is a private `fn` in the
`cdylib`-only `cartalith-godot`, unreachable for direct benchmarking):
128² ~0.99x, 512² ~1.34x, 1024² ~1.52x, 2048² ~1.81x -- better-scaling
than milestone 1's terrain result, since this crate has larger
independent per-cell functions. Combined with milestone 1: a full
`generate_terrain` + civ-layer pass at 2048² goes from ~10.62s
sequential to ~7.07s parallelized. Full record:
`cartalith-native/docs/CHANGELOG.md`'s "CPU multithreading milestone 2"
entry.

**Milestone 3 — `cartalith-climate`/`cartalith-erosion`/`cartalith-
hydrology` (done 2026-08-17).** Read every candidate function fully
before touching it (same discipline as milestones 1-2). Climate: the
deepest pass, most of the crate genuinely parallelizes (`compute_
temperature`, `apply_cryosphere_albedo`, `blur_coarse`, `deflect_flow`,
`build_wind`, `compute_ocean_current`, `ocean_sst_anomaly`, `apply_
ocean_currents`, `apply_climate_moisture_correctors`, `simulate_
weather`'s `iters` loop — parallel within each iteration, sequential
across, confirming `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 7's own
"gather-shaped" finding applies to the CPU path too). Erosion: mixed,
confirmed real hazards — `droplet_kernel` (genuine per-droplet
sequential state) and `stream_power_kernel`'s donor-receiver `iters`
loop (wavefront *within* one iteration) stay fully sequential; `erode_
thermal`/`stream_power_kernel`'s safe pieces (final clamps, receiver
computation, `u_max`/`cc`) parallelized. Hydrology: confirmed mostly
sequential, matching this scope doc's own leading hypothesis — `compute_
flow` (flow accumulation) stays fully sequential exactly as its own
pre-existing doc comment already flagged; the one real win is `build_
channels`'s main per-cell channelization loop. Golden-parity exact-
unchanged across all three crates; full `cargo test --workspace` 0
failures, 0 modified tests. Real timing (`timing_bench`, measured via a
temporary `git worktree` since a concurrent fork's own uncommitted GPU-
weather work lived in the same `cartalith-climate/src/lib.rs` file):
128² ~1.32x, 512² ~1.55x, 1024² ~1.26x, 2048² ~1.09x — unusually
better-scaling at smaller sizes than larger ones for this session's own
results, plausibly climate's coarse weather grid capping the `iters`
loop's own growth while erosion/hydrology's full-resolution passes keep
growing; not chased further. Full record: `CPU_MULTITHREADING_SCOPE.md`'s
own third-pass section and `cartalith-native/docs/CHANGELOG.md`'s "CPU
multithreading milestone 3" entry.

**Remaining, not yet scoped**: the remaining sequential `cartalith-civ`
stages (settlement placement, naming, roads, territory's outer capital
loop, villages) — confirmed genuinely hard (RNG-order/graph-shaped), not
just unattempted. Every crate's own hard-hazard functions (flow
accumulation, priority-flood, scatter-writes, per-particle/per-iteration
wavefronts) are the real remaining ceiling, per this scope doc's own
"Out of scope" section from the first pass.

**Investigated (2026-08-19): owner report "only gpu active and no
parallelisation."** Verified live through the real loaded GDExtension
(not `cargo run`) rather than trusting the milestone numbers above still
held. **Not a bug**: `rayon::current_num_threads()` correctly reports 16
inside the running extension (release and debug, `use_gpu` on or off), and
every crate's `rayon` dependency and `par_iter` call sites are still
present — no thread-pool bug, no regression. Root cause: `use_gpu`
defaults **on** in the shell, and real timing at 2048² shows the timeline
is GPU-heavy (44%) → genuinely 16-thread-Rayon-heavy (35%, real, but no
GPU activity during it so it doesn't visually read as "generation") →
single-threaded (21%, the deliberately-sequential civ stages above). A
casual Task Manager glance during the first or third phase alone produces
exactly the reported symptom even though the real parallel work — over a
third of total wall clock — is genuinely there. No code fix applied (this
is "working as designed"); one unrelated small inefficiency found and
recorded, not fixed (`WorldGen::absorb()` recomputes `build_water_bodies`
a redundant second time, ~440ms at 2048²). Full account:
`CPU_MULTITHREADING_SCOPE.md`'s "Investigated (2026-08-19)" section and
`cartalith-native/docs/CHANGELOG.md`'s matching entry.

## Unified tool plan (`UNIFIED_TOOL_PLAN.md`, milestones A-E2 done 2026-08-18)

The tool system's foundation layer plus **all four** tool groups' engine
halves, complete. **Done: milestones A, B, C, D, E and E2.** No tool is *wired*
yet; the left rail is still honestly inert (DCC shell milestone 1) until
milestone F. Remaining: **F** (shell wiring) and nothing else.

### Milestone E2 — Region select/export's format-and-pixels half (done 2026-08-18)

- **Done — everything milestone E deferred**, tested and unwired:
  - **The tile visual** — `cartalith-terrain/src/tile_render.rs`: `hypso`, the
    `SEA`/`LAND` palettes, the four v1.29 edge extrapolators,
    `renderHeightTileRGBA`, and ECMA's `ToUint8Clamp`.
  - **The raster-to-vector tracer** — `cartalith-spatial/src/geo.rs`:
    `_geoXY`, `_geoTraceMaskRings`, `_geoRingArea`, `_geoPointInRing`,
    `_geoMaskOutlineCoords`, plus `js_to_fixed`.
  - **gzip** — `cartalith-io/src/gzip.rs` (`flate2`).
  - **The `.zip` writer** — `cartalith-assets/src/archive.rs`, generalised:
    `zipStore` is ONE function in the reference with three callers, so
    `write_pack_entries` became an alias for a neutral `zip_store`.
  - **GeoJSON** — `cartalith-engine/src/geojson.rs`: `exportGeoJSON`,
    `_geoTerritoryFeature`, `_geoProvinceFeature`, and a `JSON.stringify`-exact
    writer built on `cartalith-io`'s now-public `js_num`/`json_string`.
  - **The export composition** — `cartalith-engine/src/region_export.rs`:
    `tilePngBytes` (height branch, via `cartalith_assets::raster::encode_png`),
    the gzip/PNG loop, `refineBtn`'s `.zip` assembly, and
    `extract_region_as_world`.
- **The archive conventions matched `cartalith-assets`' exactly** — same
  function — but **one milestone 2 had deliberately skipped is real**:
  `zipStore` stores rather than deflates when deflate does not shrink the
  entry, and a region-export-shaped archive comes back with **three of four
  entries STORED**. Ported now. A STORE-only archive is byte-identical to the
  reference apart from two header fields no reader interprets.
- **Four reference corrections**: `Uint8ClampedArray` rounds ties to even and
  is not a cast; `hypso` extrapolates into **negative** channels below its
  palette; `toFixed` rounds ties to the larger n where Rust rounds to even
  (reachable at `cellKm == 0.0625`); and the tracer's JS `Map` overwrite
  produces a genuinely **unclosed** ring at a checkerboard pinch.
- **`regionNewWorldBtn` is a UI action with a real core.** The button stays
  unported (UI work is on hold); `extract_region_as_world` is the arithmetic
  and the amplification, with the live-world orchestration listed rather than
  half-built.
- **A harness bug that looked like a reference bug.** E2 ran the real
  `exportRegionTiles` (which milestone E could not) and it disagreed on the
  fourth tile — because block #1's deferred boot `generate()` fired during the
  `setTimeout(0)` the export awaits between tiles and overwrote `field`
  mid-loop. Fixed in the harness; all four tiles then match milestone E's
  hashes, **discharging its disclosure**.
- **Verified:** 18 golden-parity + 61 unit tests, **everything bit-exact with
  no tolerance anywhere** (both GeoJSON documents compared as whole strings,
  rasters as FNV-1a-64 over every byte). `Math.sin`/`Math.cos` agree with V8
  across four azimuths. **58 mutations, 54 killed, 4 equivalent-mutant
  survivors** — and the first sweep's ten survivors included **six real fixture
  gaps**, with degenerate ring reachability settled by brute-forcing all 65 536
  masks on a 4x4 grid through the reference's own tracer. `cargo test
  --workspace`: 1150 passing, 0 failures.
- **Not built:** the selection *interaction* (milestone F),
  `renderBiomeTileRGBA`, `burnChannels` (LOD viewer, not this tool),
  `params.json`'s contents (`SAVEFILE_COMPAT.md` is read-only here), and every
  UI surface.

### Milestone E — the Annotation & measure group (done 2026-08-18)

- **Done — all four tools' engine halves**, across six crates, tested and
  unwired:
  - **Label** — `cartalith-civ/src/labels.rs`: `MapLabel`,
    `arc_label_layout`, `label_font_size`/`label_box`, `label_hit_test`,
    `LabelEditSession`, and the resize/rotate/arc handle formulas.
  - **Icon stamp** — `cartalith-assets/src/manual.rs`: `ManualIcon`,
    `place_manual_icon`, `icon_brush_rule`, `icon_brush_stamp`, `icon_box`,
    `icon_hit_test`, `icon_resize_scale`.
  - **Measure** — `cartalith-spatial/src/measure.rs`: `measure`,
    `measure_path`, `cell_km`.
  - **Region select/export (core)** — `cartalith-spatial/src/region.rs`
    (`norm_region`, `tile_dims`, `FloatRegion`),
    `cartalith-terrain/src/amplify.rs` (`amplify_region`, `refine_tile`),
    `cartalith-io/src/tiles.rs` (`pack_height16`/`unpack_height16`,
    `TileManifest`, `manifest_json`),
    `cartalith-engine/src/region_export.rs` (`export_region_tiles`).
- **Placement decided, not defaulted**, on A-D's rule each time: Label to
  `cartalith-civ` (the reference's own `_civ` family, beside the settlements
  and ways this crate owns), Icon stamp to `cartalith-assets` (the manual half
  of the rule-driven placement already there, same `ScatterRule` table),
  Measure and the region rectangle to `cartalith-spatial` (generic machinery),
  the amplification to `cartalith-terrain` (milestone B's subsystem-domain
  category — it is a height formula), the encodings to `cartalith-io` and the
  composition to `cartalith-engine`. `cartalith-engine` gains a
  `cartalith-io` dependency, its first.
- **Region select/export was split, honestly.** `exportRegionTiles` is four
  calls and a loop; everything hard in it is either pure geometry (shipped,
  bit-exact) or a browser API (which cannot be). So **E2** is format-and-pixels
  only: per-tile PNG (`tilePngBytes`), `gzipBytes`, the `.zip` assembly,
  `exportGeoJSON` + its raster-to-vector boundary tracer, and
  `regionNewWorldBtn`'s replace-the-world path. Smaller than the plan feared —
  and done, see the E2 section above.
- **The plan described the wrong icon function.** `_carIconBrushStamp` is a
  dart-throwing blue-noise scatter *brush*, not the single-icon stamp the plan
  calls it; the actual click-to-place path is four lines elsewhere. The brush
  is deliberately unseeded (the reference's own reasoning: a brush stroke is an
  authoring action), so `icon_brush_stamp` takes its RNG as a parameter and the
  harness overrode `Math.random` inside the vm context to match.
- **`amplifyRegion` has a real division by zero** — `outW == 1` with a region
  spanning more than one cell returns an all-NaN tile. Ported as written,
  pinned by a golden, and it forced `js_min`/`js_max` because Rust's
  `f64::min` swallows NaN where JS propagates it.
- **Measure is an addition, flagged as one** (`DECISIONS.md` §7d): the
  reference has no measuring tool, so this module has **no golden-parity test
  and cannot have one**. Its km scale is the same expression
  `civ_smooth_path` uses, compared as raw `f64` bits.
- **Verified:** 49 golden-parity tests + 132 unit tests. Everything exact with
  no tolerance except **two ULPs** in one 36-glyph arc label (`Math.sin`;
  `dy`/`rot` exact at the same glyphs, so `theta` is bit-identical), pinned to
  exactly those two indices. **89 mutations, 86 killed, 3 survivors, all three
  shown equivalent**; the first pass exposed ten real fixture-shape gaps,
  including five brush constants no golden *could* catch because a dart always
  lands on an integer cell. `cargo test --workspace`: 1034 passing, 0 failures.
- **Not built:** every tool's interaction half (milestone F), E2 in full, label
  and icon *rendering* (a `cartalith-godot` change this milestone is scoped out
  of), and persistence of `state.labels`/`state.mapIcons`.

### Milestone D — the Civilization group (done 2026-08-18)

- **Done — all three tools' engine halves**, in a new `cartalith-civ::tools`
  (`crates/cartalith-civ/src/tools.rs`), tested and unwired. Place
  settlement (`civ_drop_place`/`civ_pick_place_at`/`civ_place_pick_weight`),
  Draw route/way (`civ_dijkstra_path`/`civ_join_dijkstra_segs`/
  `civ_commit_way` plus `civ_find_snap_target`/`civ_snap_point`) and
  Territory/faction (`merge_territory_paint`).
- **Placement decided, not defaulted**: `cartalith-civ`, because all three
  are *manual entry points into a pipeline this crate already owns* — the
  same `Vec<NamedSettlement>`, and four routing helpers (`road_dijkstra`,
  `civ_routing_grid`, `civ_apply_settlement_gravity`, `civ_smooth_path`)
  that are **private to this crate** and a separate tools crate could not
  even see.
- **The headline correction: `_civDijkstraPath` is NOT `road_dijkstra`.**
  The plan said the pathing primitive was already ported. It is not:
  `road_dijkstra` is the reference's `roadDijkstra`, the bare single-source
  relaxation kernel (script block 1, ~22 500 lines earlier);
  `_civDijkstraPath` is one of its *callers* and calls it at one line.
  Unported and now ported: three cost grids
  (`_civLandCostGrid`/`_civWaterCostGrid`/`_civMixedCostGrid`, with
  `_CIV_SEA_COST = 0.6` deliberately *below* the flat-land baseline), the
  existing-way ×0.25 discount and its polyline rasterizer
  (`_civWalkWayCells` rasterizes *between* sparse sample points), settlement
  gravity, reconstruction into world coordinates, wrap-aware smoothing with
  a mode-matched validity repair, and the `reachable` flag.
- **This unblocks the Journey Planner.** `_jpRerouteForMode` was
  `JOURNEY_PLANNER_SCOPE.md`'s one remaining blocked function precisely
  because its whole body is `_civDijkstraPath`. That doc is updated; what
  remains there is a three-line transport→domain mapping and a UI action.
- **Territory/faction is a superset, flagged as an addition not parity**
  (`DECISIONS.md` §7d). The reference has only the brush
  (`_civPaintTerritoryAt`) and never had algorithmic territory generation at
  all; this port's `assign_territory` (§7b) is its own design, so the tool
  paints over a base the reference never had. The brush needed **no new
  code**: milestone C's `PaintStamp::ungated` *is* `_civPaintTerritoryAt`,
  exactly as milestone C predicted. `ungated`, because
  `_civPaintTerritoryAt` has no land/water gate — a faction can own coastal
  water.
- **Three more corrections from reading the reference**: `_civCommitRoute`
  sits **eighteen lines above** `_civCommitWay`, looks nearly identical, and
  is a different tool (`'mixed'` into `civJourneys`, not `'land'`/`'water'`
  into `civWays`) — a closer conflation trap than the `_civOpenRouteEditor`
  one the plan names. The unreachable-leg fallback is **not** a straight
  line from start to end: `_civSmoothPath` splits runs at any `|Δx| > GW/2`
  jump *unconditionally*, so the run holding the start is dropped and the
  stub sits at the target end — milestone F's warning must not promise the
  user a line between their waypoints. And `_civDropPlace` runs
  select-near-existing **before** the water refusal, so a settlement whose
  terrain changed under it stays selectable.
- **Two real bugs found in already-shipped, already-golden-verified code**,
  both latent until this milestone's first *wrapped* route fixture, both
  fixed with every pre-existing golden still passing: (1) `civ_smooth_path`
  summed `km` **across run boundaries** — the reference's guard is `if(k>0)`
  per run, so the seam jump a `brks` entry marks is excluded; a world-wrap
  route read 876.8 km against the reference's 136.6 km, one map width per
  seam, affecting `civ_consolidate_and_smooth_ways` and `civ_sea_routes`
  too. (2) **`Math.hypot` is now genuinely test-enforced** — milestone B
  honestly recorded that its fixtures could not distinguish V8's compensated
  version from `sqrt(x²+y²)`; `_civSmoothPath` accumulates `km` in `f64`
  with no rounding step, so one ULP survives
  (`610.6390435628962` vs the reference's `...63`). `cartalith-civ` now has
  its own `js_hypot` across the route-geometry chain only; the crate's other
  `.hypot()` sites are deliberately untouched, being covered by their own
  passing goldens.
- **No `PassBuffer` anywhere, deliberately.** The plan predicted this for
  two of the three tools; it held for all three. One atomic append; the
  waypoint chain *is* Draw way's pass-buffer unit; Territory's staging is
  milestone C's `PaintLayer`.
- **Golden-verified bit-exact, 16 cases, no tolerance anywhere** — `km`
  compared as raw `f64` bit patterns, the territory raster as an FNV-1a-64
  over every byte. Harness: **whole `<script>` blocks, not line slices**
  (#1 2084-14556, #2 14563-26720), asserted by their real
  `<script>`/`</script>` delimiters. The balance/orphan-close checks fired
  twice and were **wrong both times** — nested template literals, then regex
  literals containing a bare `"` — each fixed properly rather than deleted.
  Emptiness assertions and real negative controls throughout (every "should
  not route" case asserts `reachable === false`). The world underneath was
  FNV-checked against this port's own `generate_terrain` pipeline first:
  field, water bodies, biome raster and Strahler order all matched exactly
  in both cases.
- **Verified**: 28 new unit tests (225 total in `cartalith-civ`) + 16 golden
  tests; `cargo build/test/clippy --all-targets` clean on `cartalith-civ`;
  `cargo test --workspace` 842 passing, 0 failures.
- **Not built, deliberately**: the interaction halves — waypoint capture,
  Escape-to-commit, the shared active-faction quick-select, brush-radius and
  way-type pickers, the snap on/off switch — all input routing, milestone F.
  Also `_civCommitRoute`/`civJourneys` (a Journey Planner surface),
  `_civDropPOI` (no POI concept here), `_civConnectPlaceToNetwork`, and
  provinces over a *painted* territory raster.

### Milestone C — the Water & ecology group (done 2026-08-18)

- **Done — River/water's special commit path**, in a new
  `cartalith-engine::sculpt_commit`
  (`crates/cartalith-engine/src/sculpt_commit.rs`): `WaterState`,
  `commit_sculpt_pass`, `SculptCommitSummary`. Plus
  `enforce_river_channels` in `cartalith-hydrology`, three lines from
  `enforce_channel_descent` as in the reference.
- **What the "special commit path" concretely is** (reference 9318-9346): a
  five-step sequence whose **ordering is load-bearing** — bake the whole
  stack → `enforceRiverChannels` re-clamps cells locked by an *earlier*
  commit (**after** the bake, **before** this batch's carving, or a
  Mountains stamp painted over an old river buries it) → per river stamp,
  `enforce_channel_descent` + lock into `river_mask`/`river_floor` → Lake
  **last**, as a `water_only` dry run against the already-final height,
  depositing into `lake_mask` → one `computeFlow`/`refreshClimate`. Steps
  1-4 are ported; step 5 deliberately is not, because it is downstream
  whole-field recompute and milestone A's `StageGraph` exists so it stays
  deferred. That line — 2-4 are *part of the edit*, 5 is *recompute* — is
  the plan's one real ambiguity, now resolved.
- **Done — Biome paint**, in a new `cartalith-spatial::paint`
  (`crates/cartalith-spatial/src/paint.rs`): `PaintStamp` (hard-edged
  categorical disc, `Stamp` with `Cell = u8`) and `PaintLayer` (lazy
  override grid, nearest-neighbour sample, per-cell merge, sparse
  `state.cartoPaint` persistence). **Placement decided, not defaulted**:
  generic machinery, so beside `PassBuffer` — the module never learns what
  a biome is. Milestone D's Territory paint therefore needs no new stamp
  type.
- **Reading the reference corrected the plan three times on paint**: there
  are **three** layers (`paintBiome`/`paintSplat`/`paintTerrain`), not one;
  the override merges by per-cell **replace only at export**, while the
  renderer alpha-blends it at weight **0.60** over the fully shaded colour,
  and **no analysis consumer merges at all** (`buildEcoregions` and every
  Journey Planner `currentCartBiome()` reader take the unpainted output) —
  so the plan's "merge at every `classify_biome` call site" would have added
  behaviour the reference lacks; and the land gate is `wb[i] !== 0`, which
  excludes **lakes as well as ocean**, including above-sea-level ones.
- **Also corrected on rivers**: `half_w` is `max(1, brushSize*0.13)`, the
  brush — *not* `carveRiverValleys`' discharge-derived width; and
  `enforce_channel_descent` walks the stroke's own points and **never
  resamples**, so a 2-point stroke locks 3 cells where a 23-point one locks
  46. That is a real constraint on milestone F: **stroke capture must not
  decimate hard**.
- **A gap this milestone opened and closed**: `build_water_bodies` had
  deliberately omitted `forceLake` because nothing produced a painted-lake
  array. The Lake commit hook is that producer, so `apply_force_lake` now
  ships in `cartalith-civ` — a post-pass, **bit-equivalent** because `force`
  is the reference's last mutation of `out`, leaving every caller's
  signature alone (including `cartalith-godot`'s).
- **One new affordance, flagged as new not parity**: `PaintStamp::mask` is
  `Option` so the mockup's "respect water mask" switch is buildable later;
  the reference always gates, the Cartography constructor requires a mask,
  and the ungated one is separately named (`DECISIONS.md` §7d).
- **Golden-verified bit-exact, 18 cases, first run** (11 water, 7 paint).
  Six slices with block-comment balance **plus start/end boundary**
  assertions. The assertions caught two things: a false positive on a
  one-line function (fixed properly rather than deleted), and — the one
  worth remembering — that the reference's `let`-declared paint globals are
  **lexical bindings, not context properties** in a `vm` script, so host-side
  assignment silently shadowed them and `_paintAt` ran against defaults,
  producing empty output with no error. Same class as Journey Planner
  milestone 5's silently-empty stage list.
- **Disclosed**: `sculptCommit`'s water-hook body is *transcribed*, not
  sliced (the function's own head and tail are DOM and whole-pipeline
  recompute), so lines 9320-9346 are copied verbatim minus those calls.
- **Open, deliberately**: whether an *incremental* terrain commit should
  clear painted overrides under it. The reference only ever had one
  `generate()`, so it has no answer; `PaintLayer::clear` implements the
  faithful floor and names the question. The deciding caller is milestone F.
- **Not built**: stroke/tap capture and the layer/value/radius pickers
  (input routing, milestone F); the `biomes`/`terrains` pack-image decode
  and the 0.60 blend in `land_color`, both `cartalith-godot` changes this
  milestone is scoped out of — though the producer they were waiting on now
  exists.

### Milestone B — the Sculpt-editor terrain port (done 2026-08-18)

- **Done — the whole thirteen-feature landform registry**, in a new
  `cartalith-terrain::sculpt` (`crates/cartalith-terrain/src/sculpt.rs`),
  implementing milestone A's `cartalith_spatial::Stamp`. Covers all four
  Terrain-group tools at once (Raise/lower, Smooth, Flatten/terrace, Stamp),
  since they share one registry. `cartalith-terrain` gains a
  `cartalith-spatial` dependency — the workspace's second.
- **Placement decided, not defaulted**: `cartalith-terrain`, because the
  features are height-field math and that crate already owns the height
  formula; a `cartalith-sculpt` crate would have bought a `Cargo.toml` and
  nothing else, and `cartalith-engine` orchestrates rather than computes.
- **The real registry**: mountains, hills, ridge, plateau, cliff, canyon,
  valley, river, lake, basin, coastline, volcano, freehand (8 sub-modes) —
  in `Object.keys` order, which is **load-bearing** because a stamp's noise
  seed is `(seed ^ ((index+1)*1013)) >>> 0`. Plus 8 presets, 8 globals, 38
  per-feature controls with their real min/max/step/default, and each
  feature's own `edgeChar`/`edgeFreqMul` edge character. Volcano is the one
  feature that sizes itself from its own control, not `brushSize`.
- **Golden-verified bit-exact, 23 cases** — correcting the plan's own
  prediction that only unit-tested algebra was available here. A stroke
  *sequence* is not a reproducible fixture, but a *stamp* is: the reference
  stores one as plain data, so the real `sculptApplyStamp` runs headlessly
  under `vm.runInContext` with no DOM and no `generate()`. Harness slices
  four contiguous line blocks with block-comment balance assertions on each.
- **No tolerance needed** for `Math.pow`/`exp`/`hypot`, unlike this
  workspace's earlier `1e-4` precedent — the `f32` store absorbs the
  last-ULP `f64` disagreement. Measured, not assumed: the same absorption
  means these fixtures do *not* distinguish V8's Kahan `Math.hypot` from
  naive `sqrt(x*x+y*y)`, and `js_hypot`'s doc says so plainly rather than
  claiming a guarantee it does not have.
- **One deliberate divergence**: `sea_level` lives on the stamp, because
  `Stamp::apply` takes only a destination and cannot read a live global the
  way the reference does. `with_sea_level()` is the explicit re-stamp.
- **A limitation carried over faithfully**: no world-mode equirectangular
  wraparound in stroke distance. `SCULPT_EDITOR_INTEGRATION_PLAN.md` §6 left
  this as an open item and the reference shipped without resolving it.
- **Verified**: 43 unit tests + 23 golden tests; `cargo build/test/clippy
  --all-targets` clean on `cartalith-terrain`. `cargo test --workspace
  --exclude cartalith-godot` also clean — the `cartalith-civ` build break the
  milestone-A note below recorded is **gone**; `cartalith-godot` excluded
  only because a running Godot editor held its `.dll`, and `cargo check -p
  cartalith-godot` is clean.
- **Open, deliberately**: the water-commit hooks (milestone C) — though
  `apply_into`'s `water`/`water_only` primitive is ported and golden-verified
  here; the mockup's "respect water mask" gate for Raise/lower (a real new
  feature — the reference's Freehand has no water gate at all); stroke
  capture/simplification and the overlay palette (Godot-side); shell wiring
  (milestone F).

### Milestone A — the `PassBuffer`/staleness core (done 2026-08-18)

- **Done — the `PassBuffer`/staleness core**, tested and unwired.
  `cartalith-spatial::pass` (`Stamp` trait, `PassEntry<S>`, `PassBuffer<S>`,
  `CommitSummary`) and `cartalith-spatial::staleness` (`StageGraph`), plus
  `cartalith-engine::staleness` (`PipelineStage`/`pipeline_stage_graph()`)
  for Cartalith's own stage names and edges. 43 new tests in `-spatial` (67
  total), 5 in `-engine`, clippy clean on both.
- **The reference's Sculpt editor was read directly, not through a summary.**
  Its draft/commit/discard model is real and is the pass buffer's direct
  ancestor, as the plan claimed. The property reading added: a stamp holds
  **no pixel data** — it is a recipe re-evaluated over its own bounding box —
  which is why `Stamp` shipped as a trait rather than a struct, and why this
  milestone is a small type rather than a delta-buffer subsystem.
- **`DirtyTracker` needed no extension**, only composition. Its `mark_dirty`
  already is "my data changed here, here's why, bump the version" — the one
  primitive both editing and recomputation need.
- **Staleness is deferred structurally, not by convention**: `StageGraph` has
  no recompute hook of any kind and every query takes `&self`. It cannot
  recompute. That is the code answer to the measured ~7.07s terrain+civ at
  2048² behind the mockup's "rivers · deferred".
- **First dependent on `cartalith-spatial`** (`cartalith-engine`). That
  crate's "whenever a real large-world need triggers integration" trigger
  turned out to be the tool system, not LOD rendering — see the section
  immediately below, whose "referenced by nothing" line is now history.
- **Open: milestones C-F** — water & ecology (C), civilization (D),
  annotation & measure (E), shell wiring (F). B is done, above.
  Also deliberately open: the field-level undo snapshot at commit time (no
  undo stack exists in this port yet to snapshot into; `commit` returns the
  touched-tile list a tile-diff undo would need), and tile-incremental
  recompute of hydrology/climate/civ (none are tile-scoped today — staleness
  reports which tiles are stale, stages still recompute globally).
- ~~**Note for the next session:** `cargo test --workspace` currently fails
  to build `cartalith-civ`~~ — **resolved**: that sibling fork has landed.
  Milestone B ran `cargo test --workspace --exclude cartalith-godot` clean.

## LOD/tiling base (`LOD_TILING_BASE_SCOPE.md`, done 2026-08-17; integrated 2026-08-18)

Owner directive, directly after `TERRAIN_ARCHITECTURE_RESEARCH.md` was
filed as forward-looking research (not current scope -- most of it
assumes a real-time camera/LOD/streaming/painting engine Cartalith
isn't): "LOD and zoom etc might be out of scope for the base, but
they're still goals in this project. The base should be present before
integration." Given three concrete scope options, the owner chose the
middle one -- foundational data structures now, real and unit-tested,
zero integration into the live pipeline.

New crate `cartalith-spatial` (no `gdext` dependency): `TiledField<T>`
(zero-copy tile/region/row/column views over a flat `Vec<T>`, the same
SoA layout `WorldState`/`CivData` already use; `tile_size` is a
constructor parameter, not hardcoded, since no real workload exists yet
to benchmark against), a packed `QuadTree<T>` (`Vec<Node>`, integer
child indices, generic caller-defined aggregate flags, real
bounds-rejection proven by a visited-node counter -- a 64x64/leaf_max-4
tree queried with a 1x1 region visits `< len()/4` nodes, not a brute-force
full traversal), and a generic `DirtyTracker` (per-tile dirty flag +
monotonic version counter, no Cartalith-specific field-dependency
semantics baked in). `serde` round-trip tested on all three. 24 real
unit tests (not compile-only), `cargo build/test/clippy -p
cartalith-spatial` clean, full workspace `cargo test` clean (one
`cartalith-engine` GPU-determinism test flake reproduced the
already-documented pre-existing GPU-driver flakiness under parallel
scheduling, unrelated -- passed on isolation and on a clean re-run).

**Confirmed nothing else in the workspace references this crate** -- true as
of 2026-08-17, **no longer true**: `UNIFIED_TOOL_PLAN.md` milestone A
(2026-08-18) built `PassBuffer<S>`/`StageGraph` in this crate and made
`cartalith-engine` its first dependent. The trigger this section waited for
turned out to be the DCC tool system, not Phase 3 or LOD. The bet paid off
as argued -- the tool system started from a tested foundation, and
`DirtyTracker` needed no extension whatsoever to serve its first real
caller. Full record: `cartalith-native/docs/CHANGELOG.md`'s "New crate
cartalith-spatial" entry and its "unified tool plan milestone A" entry.

## Province boundary legibility (fixed 2026-08-17)

The province-boundary overlay (`build_province_boundary_texture`, wired
same-day as milestone 16's own follow-up) was flagged as a known
legibility issue: functionally correct data, but a literal 1px-wide line
at full grid resolution became sub-pixel and near-invisible once
downscaled to the viewport. Fixed with symmetric boundary detection plus
a one-cell dilation for a real ~3px stroke and a modest alpha bump
(not to fully opaque). Real screenshot-verified (seed 12345, Classic,
512×512, both territory and province layers on): boundaries now read as
clean, bold lines at normal view, clearly distinct from roads. See
`CHANGELOG.md`'s "Fix: province boundary lines were illegible at normal
zoom" entry.

## App icon (done 2026-08-17)

Owner-supplied icon (`design/app-icon.png`) wired into both platform build
targets: `project.godot`'s `config/icon` (editor/debug-run window icon —
screenshot-confirmed real, not assumed from config alone), Windows export's
`application/icon`/`console_wrapper_icon` (a real multi-resolution `.ico`
generated from the source), and Android's four `launcher_icons/*` fields
(legacy + adaptive foreground/background/monochrome, generated with real
safe-zone margins so launcher masks don't clip the content). Full record in
`CHANGELOG.md`'s "App icon wired for Windows and Android" entry.

## GUI shell + terrain appearance, second pass (done 2026-08-17)

Second workflow re-audit found and fixed a real structural gap: the Layers
panel is now a permanent fifth region (nav / params-or-placeholder / layers
/ viewport / inspector) rather than something the navigator swapped to —
matching the mockup's own always-visible region count. `GUI_SHELL_SCOPE.md`'s
own "second workflow re-audit" section has the full reasoning.

`TERRAIN_APPEARANCE_SCOPE.md` milestone 3 (hydrology-based colour
modulation, research doc §13) also landed: a subtle, flow-accumulation-
driven wetness tint on land colour near rivers/high flow, gated the same
way milestone 2's hillshade/AO were — `js_reference()` stays a true no-op,
`golden_parity_render.rs` unmodified at its original tolerance.

Both verified together: real end-to-end generation (seed 12345, Classic,
2048×2048, 40 settlements) through the restructured shell, full workspace
test suite green, headless load clean.

## Journey Planner Godot boundary (`JOURNEY_PLANNER_SCOPE.md` closing-status steps 1/2/4, done 2026-08-19)

The engine half of this subsystem has been complete since 2026-08-18 (65 of the reference's 74 `jp*` functions, golden-tested in `cartalith-civ`) and **none of it had ever been reachable from Godot** — zero `#[func]`s existed for any of it. That is now closed for the Rust boundary; the GDScript party form and results panel are deliberately still open (see below).

**New `#[func]` surface** (`cartalith-godot/src/lib.rs`, one new `#[godot_api(secondary)]` block, plus two in the existing INFRA block):

| method | what it is |
|---|---|
| `jp_options() -> Dictionary` | every dropdown vocabulary, keyed by the same field names `jp_compute` accepts; `route_cond` nested per travel category; `reference` carries the terrain/biome/category/animal tables a results panel needs. Pure — callable before `generate()`. |
| `jp_default_plan() -> Dictionary` | `JpPlan::default()` flat (28 keys + `party_fields`), so a form seeds itself from the engine instead of restating `_jpEnsurePlan`'s defaults. Pure. |
| `jp_compute(request: Dictionary) -> Dictionary` | `jp_plan` → `jp_verdict` → `jp_confidence`, flattened. `request` = `route` (int index) or `points` (`PackedVector2Array`), plus optional `plan`, `stage_overrides`, `layovers`. |
| `route_count() -> int` | how many routes are committed. |
| `route_get(index) -> Dictionary` | `{points, brks, km, mode, unreachable_legs}` for one committed route. |

**The route-getter gap was real and is now closed.** `route_commit()`/`way_commit()` had been returning an index into a list nothing could read back — the INFRA milestone disclosed that rather than inventing a getter. `route_get`/`route_count` are that getter, and `jp_compute`'s `route` key is its first real consumer (it reads the route's own `f64` grid coordinates, which is why it is preferred over the `f32` `points` round trip).

**`JpWorld` needed no new pipeline state.** Every raster it borrows was already live on `WorldGen`: `field`/`temperature`/`rainfall`/`flow_discharge` from `WorldState`, `water_bodies`/`territory`/`ways`/`settlements` from `CivData`, `peak_m` from `WorldParams`, `flow_thresh` from the same `river_flow_thresh` call `compute_civilisation` makes. Only the three genuinely derived tables are computed at call time, from those same rasters — `build_cart_biome`/`build_cart_terrain` (milestone 5 added both and, exactly as the scope doc predicted, still no pipeline stage calls either) and `jp_road_cells`. No generation stage was added.

**Three inputs are honestly absent rather than faked**, all disclosed in `journey_bridge.rs`'s module doc:

- `ocean_field`/`wind_field` are `None`. This port's climate stage computes the ocean-current field inside `cartalith_climate::ocean_sst_anomaly` and discards it; nothing in `WorldState` retains a `u`/`v` pair at any resolution, so there is no `currentOceanField()`/`currentWindField()` equivalent to pass. `None` is `jp_sea_condition`'s own supported input — a sea leg reads its structural condition and skips the wind/current term. Retaining the coarse fields past generation is real `cartalith-engine` work.
- `road_cells` sees the generated way network only. `jp_road_cells` takes `&[Way]`; hand-drawn ways are `tools::ManualWay`, whose `Ancient` variant `jp_road_cells` has no branch for (the reference's `_jpRoadCells` does, because its one `civWays` array holds both kinds). Widening it is a `cartalith-civ` change against golden-tested code.
- `road_edges` is empty — the reference's second source is `state.roads.edges`, and `build_road_network`'s `RoadEdge` list is not retained by `compute_civilisation`.

`wildlife_forage_mod` is `|_, _| 1.0`, the reference's own answer on a world with no wildlife layer (already disclosed by the scope doc as a quality ceiling, not a gap).

**One reference behaviour preserved rather than "fixed":** `jp_claimed_at` tests `territory[i] >= 0`, and this port's `assign_territory` uses `0` = unowned — so every cell reads as claimed. That is exactly what the reference does (its `civTerritory` is a `Uint8Array`, so `>= 0` is likewise always true). `civ.territory` is passed through unchanged; changing it here would be a silent divergence.

**Tests**: 28 new plain-Rust tests in `journey_bridge.rs` (`cargo test -p cartalith-godot`, no Godot runtime) — form parsing, the flatten/reparse round trip, per-stage overrides, and one recogniser test per option table pinned against the engine's *own* lookup (a dropdown offering a key the engine does not know does not error, it falls through to `?? 1.0` and reports a plausible number from the wrong row). Plus an end-to-end test that the assembled `JourneyWorld` really drives `jp_plan` rather than merely producing non-empty buffers. 153 unit tests pass after a `cargo clean -p cartalith-godot` rebuild; headless Godot 4.7.1 boot is clean and a scripted smoke run planned a real 1157 km, 11-stage, 3-stop journey with verdict and confidence band.

**Was open, now closed** (see the section immediately below, same day): the GDScript party form and results panel (`JOURNEY_PLANNER_SCOPE.md` closing-status steps 3 and 5) shipped first as an `AcceptDialog` window, then the same day were rebuilt again as the in-shell distance-spine view this port actually kept.

## Journey Planner distance-spine takeover (`JOURNEY_PLANNER_SPEC.md`, done 2026-08-19)

Two passes landed the same day. The first built the ~430-line `journey_planner_window.gd` (`extends AcceptDialog`) — a real, working party form + results panel, but a popup modal over the map. The second, this one, replaced it entirely: `DCC_SHELL_SPEC.md` §4.5.4's own addition makes Journey a third INFRA tool (alongside Way/Route) that, when armed, swaps the whole INFRA viewport region — map, both docks, tool options bar — for `journey_planner_view.gd`'s distance-spine layout, rather than drawing an overlay on the map or staying a dialog. `journey_planner_window.gd` is deleted; its field-binding/results-rendering logic was carried forward into the new file, not rewritten from scratch.

**Architecture**: `JourneyPlannerView` builds three region roots once (`_left_panel` into `app.left_dock_body`, `_center_panel` into `app.viewport_content` alongside `app.viewport`, and a `right_dock.gd` delegate via a new `CTX_JOURNEY` context mirroring the existing `CTX_SCULPT` precedent), all hidden by default. Visibility is recomputed off `app.tool_armed` and `app.workspace_changed` together (`app.armed_tool == "journey" && app.active_domain() == "infrastructure"`), so switching domains away and back while Journey stays armed restores the swap instead of leaving stale chrome — verified with a scripted headless run (arm on `infrastructure`, switch to `world` while armed, confirm the view hides, switch back, confirm it reappears).

**What's real**: route map and terrain-profile stage bands are sliced from `route_get()`'s own points using `plan.stages[i].{i0,i1}` (the same index range `jp_plan` derived that stage over) — real geometry, not a decorative curve. The elevation sparkline draws `plan.profile`'s real 0-1 samples (left undone in the AcceptDialog pass as a disclosed gap; closed for real here since rebuilding the view was the right time). Stops-strip x-position is the nearest route point's real cumulative chord length over the route's total length (exact for position purposes since `map_width_km` is uniform across the grid). The stage inspector's 15 override fields and the stage matrix's mode/pace/hours columns write into `jp_compute`'s real `stage_overrides` map. The results panel's Time/Load/Supply reach/Vessels groups all read real `jp_compute` fields.

**What's disclosed rather than faked**: Carriage Auto mode has no Rust port of the reference's own `jpAutoPickTransport` (checked — no such function exists anywhere in `cartalith-civ`/`journey_bridge.rs`); selecting Auto disables editing the animal/vehicle counts and says so, it does not compute a plausible-looking pick. Party presets (`JP_PRESETS`, reference-JS-only) and re-route-for-mode (`_jpRerouteForMode`, same gap) are both present, disabled, with the reason stated. The Cost results group has nothing to show — `jp_journey_plan_dict`'s full field list carries no monetary figures at all. ⇧-drag spine trim is deferred, not faked — `jp_compute` has no request field a trim gesture could feed. Calculation-trace (`⧉`) is a disabled stub, matching every other genuinely-unbuilt-window precedent in this shell.

**One field-count discrepancy resolved**: `JOURNEY_PLANNER_SPEC.md` §5 says "all 26" party fields; the live `jp_default_plan()` call returns 28 real plan fields (already correctly documented two sections up in this file) grouped into the mockup's own four left-dock sections (Traveler, Season & weather, Carriage, Route conditions — the mockup has no fifth "Stops" group in the left dock; Stops is the separate 32px centre strip per §3's own region table). The spec's prose undercounts by 2; the engine-side number was already right.

**Wiring**: `Data ▸ Journey planner… ⇧J` (`menus.gd`, new `ID_JOURNEY_PLANNER`) and the INFRA dock's own Logistics "Open Journey Planner" button (`infrastructure_workspace.gd`, unchanged call site) both arm the tool via `app.open_journey_planner() -> journey_planner_view.open() -> app.arm_tool("journey")`. The mockup's own "rail-foot slot" phrasing is honoured as the tool's context readout (`set_rail_foot("JOURNEY")`, already wired through `DccShell`'s existing shared `rail_foot` `Label`) rather than a second clickable entry point — making only INFRA's foot cell independently clickable would be a shared-base-class change (`dcc_shell.gd`) for a capability the dock button already provides.

**Verified**: headless boot clean (`--headless --path . --quit`, zero errors). A scripted smoke run (generated a small world, committed a real route, armed the tool, confirmed both docks + centre swap and the map hide, confirmed a real `jp_compute` result with 14 derived stages, applied a stage override and confirmed recompute, disarmed and confirmed full restoration, then re-armed on a different domain and confirmed the view stayed hidden until the domain switched back to INFRA) — all passed, script discarded (not committed, matching this port's "no test scaffolding left behind" convention for one-off harnesses).

**Not attempted this pass, disclosed**: light theme (spec §10's own "still to build" list), the 2560 tablet breakpoint, and the blocked-stage inspector's own distinct visual state beyond the block-token colouring already applied throughout.

## "Layers don't work" — LOD tile layer occluded every overlay (owner report, fixed 2026-08-20)

- [x] **Root cause, verified live.** `viewport_host.gd` added `_lod_layer`
      (deep-zoom tiles) to `_camera` *after* `territory_view`, `province_view`
      and `_debug_layer`, so at `modulate.a == 1.0` it drew over all three.
      `_update_lod()` activates whenever the fit scale exceeds
      `LOD_PX_PER_CELL_THRESHOLD` (1.0 px/cell), which a 384x256 grid in a
      ~900 px viewport already does at `_zoom == 1.0` — so the tile layer was
      opaque from the first frame after every generate at the common small
      presets, with no zooming involved. Picking a field view lit the row,
      built a real texture, echoed its id back from `debug_view()`, and
      changed **0** screen pixels.
- [x] **Fixed** by moving `_lod_layer` to sit directly above `map_view` and
      below the three overlays. A refined tile *is* the base map at deep zoom;
      the other three are overlays that happen to be rasters. Incident
      recorded in the node's own doc comment.
- [x] **Hotkey badges no longer land on dead rows.** `layers_popover.gd`
      badged by position across all rows including the 11 permanent
      `GAP_LAYERS`; the seven new Climate views (2026-08-19) pushed Köppen into
      slot 4, so `4` had been a silent no-op since. Badges now skip
      unavailable rows: `1..8` = `off, elevation, temp, rain, wind, ocean,
      plates, bounds`.
- [x] **Measured on the real rendered frame**, windowed 1600x1000, real
      `app.tscn`, real 384x256 world, differing-pixel counts on a 4-px
      sampling grid (~33,000 samples over the map region). Before: `off` vs
      `temp` = 0. After: all 25 available views 32,734-32,750 each; territory
      fill 24,061; province boundaries 2,151; opacity 100% = 32,750 and
      0% = 0.
- [x] `cargo test -p cartalith-godot --release --lib` 227/227. No Rust
      changed — `sample_bridge`'s layer table was cleared by the
      investigation, not modified. Headless boot (`--quit-after 30`) clean.
- [ ] **Open, reported not fixed:** `godot-project/main.tscn` + `main.gd` (the
      superseded pre-DCC shell, 2,358 lines, referenced by nothing under
      `shell/`) are still in the project and are the editor's only entry in
      `.godot/editor/project_metadata.cfg`'s `recent_files`. Running that
      scene (F6) shows a five-checkbox LAYERS dock, no popover, no field
      views — a second, independent way to see "crude and incomplete" where
      layers do not work. `project.godot`'s `run/main_scene` is correctly
      `res://shell/app.tscn`. Deleting the pair is left to the owner: it is a
      2,400-line removal and `main.gd` is cited across the port's docs as the
      source the current shell was ported from.
- [x] **Closed 2026-08-20 — it was a false alarm.** The untracked
      `godot-project/android/build/src/instrumented/assets/project.godot`
      names `res://main.tscn`, but **that file is not ours**: its
      `config/name` is `"Godot App Instrumentation Tests"`, it ships as part
      of Godot's Gradle build template, and its `res://main.tscn` is its own
      1,308-byte test scene sitting beside it. It is inert twice over — the
      Android preset sets `gradle_build/use_gradle_build=false`, so the export
      never enters `android/build/` at all, and `godot-project/android/` is
      `.gitignore`d. The stale-APK half of the worry was real and is fixed by
      the 2026-08-20 re-export; the artifact itself needed no action.
- [x] **Build hazards documented** in `TOOLCHAIN.md` (new section under
      Windows): the editor holds `target/debug/cartalith_godot.dll` open and
      `cargo build -p cartalith-godot` then fails with `Access is denied. (os
      error 5)` while the stale DLL keeps loading (reproduced live against the
      owner's running editor); and everything routine — editor, Play, every
      `--headless` scripted drive — loads the **debug** entry, not release, so
      both profiles need building when a change is meant to reach an export.

## Deep-zoom LOD showed the *Relief* tile renderer over the *Biome* map (owner report, fixed 2026-08-23)

- [x] **Reproduced live before anything was changed.** Same seed, same window,
      same camera, LOD layer shown vs. hidden: shown, a bare green/gold/grey
      hypsometric ramp over flat blue sea; hidden, the full plate — biome
      colour, the river network, hillshade, AO, paper frame, neatlines. The
      owner's "a zoom action exposes the underlying heightmap" is literal.
- [x] **Root cause: a branch this port never had.** The reference chooses the
      LOD tile coloriser by view mode — `_lodBuildTileRGBA`, reference 11148,
      `biome ? renderBiomeTileRGBA : renderHeightTileRGBA`, with `'biome'` the
      app default (reference 2260). `renderHeightTileRGBA` is *Relief* mode.
      Only that half was ported, and `lod_bridge.rs` wired the compositor
      straight to it while the map view is always the biome look.
- [x] **Second divergence: the entry threshold.** `viewport_host.gd` gated on
      `native_px_per_cell * zoom > 1.0` alone, true at the **fit** view for any
      world narrower than the map rect (512 cells in 888 px = 1.73 px/cell), so
      the wrong renderer was live with no zoom at all. The reference also
      requires camera zoom `> LOD_AUTO_SCALE = 2.2` (reference 13952/13986).
      Added as `LOD_AUTO_ZOOM`; both conditions now hold, as there.
- [x] **Fixed by making a tile carry only what the base raster cannot have.**
      New `cartalith-terrain::tile_render::shade_tile` (the `s` multiplier
      `render_height_tile_rgba` applies, on its own — that function itself
      untouched, goldens unchanged); `lod_bridge::synthesize_tile_rgba` now
      encodes `shade_tile(with detail) / shade_tile(no detail)`, and the new
      `shell/lod_tile.gdshader` multiplies it into `map_view`'s own texture
      (`filter_linear`, which also removes the blocky cells M1 existed for).
      Where the amplifier adds nothing the ratio is exactly `1.0` and the map
      is byte-unchanged — pinned by a test, not asserted in prose.
- [x] **CV-VS-01 (`GUI_GAP_REGISTER.md` §14.4) closed in the same pass.**
      `amplify_region` samples endpoints-inclusive; the raster draws texels.
      Passing `bounds.to_float()` straight through stretched `TILE_CELLS`
      cells of screen over `TILE_CELLS - 1` cells of data and offset it half a
      cell, so every tile edge was a real discontinuity. `tile_sample_region`
      fixes the convention; adjacent tiles now sample exactly one texel apart.
      Measured pre-fix in CIVIL at fit zoom: median row discontinuity 2.26,
      spiking to 19.03 and 10.30 on two `TILE_CELLS` row boundaries.
- [x] `cargo test -p cartalith-terrain -p cartalith-godot` 27 suites pass (8
      new); `--headless --path godot-project --quit` clean; clippy no new
      warnings; non-headless before/after at z1.0/2.31/3.51/5.35/8.0 in both
      WORLD and CIVIL.
- [ ] **Open: `renderBiomeTileRGBA` is still unported.** Tile colour comes from
      the coarse raster, so sub-cell *colour* variation (landColorCore's
      slope/curvature rock and scree, its river SDF) is absent — only sub-cell
      *relief* is there. Closing it needs the climate/lithology fields threaded
      into `lod_bridge`, i.e. a `#[func]` signature change in `lib.rs`.
- [ ] **Open: `lib.rs`'s `lod_synthesize_tile` doc comment is now stale** — it
      still says "one synthesized, **coloured** deep-zoom tile". That file was
      owned by a concurrent session and was deliberately not edited.

## Wind / Ocean-currents streak animation (owner report, done 2026-08-23)

- [x] **The reference's `#windFxCanvas` particle overlay ported**
      (`shell/wind_fx_layer.gd`) — 260 wind / 200 ocean particles advected
      along the flow field at the reference's own `0.315` cells/tick, its
      lifetimes, its respawn rules, its colours, its 1-cell stroke. Attached
      once from `layers_popover.gd::_attach_flow_fx`, under `map_overlay.gd`
      so pan/zoom needs no code.
- [x] **`cartalith-climate::current_ocean_field`** — `currentOceanField()`'s
      vector field + ocean mask, extracted from `ocean_sst_anomaly` so the
      anomaly raster and the streaks cannot disagree. Golden parity unchanged
      (`cargo test -p cartalith-climate`, 21 green).
- [x] **Nothing runs while the layer is off.** `_process()` is one
      `debug_view()` read; no field is held, no particles exist, the node is
      `visible = false` so `_draw()` is never reached. Verified: 0.0000
      frame-to-frame difference with the layer off, and a clean restart after
      a toggle cycle and after a regenerate under a live view.
- [x] **Verified non-headlessly** at 1280×800 on an RX 7800 XT — the only way
      a motion feature can be verified. Numbers in the header above and the
      `CHANGELOG.md` entry; harness `_flowfx_shot.tscn`, uncommitted per the
      `_shot.gd` convention.
- [ ] **Still open: the `flowfx:` data channel is a workaround.** The flow
      field reaches GDScript packed into a `build_debug_texture` raster
      (12 bits each for `u`/`v` at a ±8 scale, ocean mask in alpha) because
      `lib.rs`, the crate's sole `godot` boundary, was owner-reserved for
      concurrent work when this landed. The right shape is a `#[func]`
      returning the field; swapping to it changes only
      `wind_fx_layer.gd::_start`/`_decode`. `flow_fx_raster`'s own doc
      comment says so, and
      `flowfx_channel_round_trips_the_flow_vectors` pins the two sides
      together until then.
- [ ] **Still open: streak density/speed are not exposed as controls** — the
      reference's constants, unparameterised. Belongs with `MEMORY.md`'s
      deferred Phase-3 visualisation-controls pass, not a raw slider.

## Sample panel + Layers popover (`DCC_SHELL_SPEC.md` §6/§9, done 2026-08-19)

**All twelve of §6's dashed Sample fields are live, and none of them needed a
byte of new retention.** `right_dock.gd`'s `MISSING_SAMPLE_FIELDS` listed
twelve readouts (slope, aspect, plate + type, boundary + distance, resistance,
lithology, temperature, precipitation, drainage, biome, soil, control) that
read `—` always, each with "no per-cell query" against a `WorldGen` that
exported no field sampler. `sample_bridge.rs` is that sampler. Elevation — the
thirteenth, §6's large accent readout — was dashed too and is now metres above
sea level.

**Nothing was added to `WorldGen`, `WorldState` or `CivData`.** Every reading
is either a raster generation already keeps or is derived from those at the
one queried cell:

| Field | Source | Cost per query |
|---|---|---|
| elevation | `WorldState::field` + `metersPerUnit()`'s own anchoring | O(1) |
| slope, aspect | central difference of `field` at the cell | O(1) |
| plate + type | `plate_id`, oceanic/continental from `crust_field`'s sign | O(1) |
| boundary + type | `boundary_mask` + `boundary_type` | O(1) |
| boundary distance | ring search over `boundary_mask`, capped at 96 cells | O(d²) |
| resistance, temperature, precipitation, drainage | the same-named `WorldState` fields | O(1) |
| river order | `WorldState::stream_order` | O(1) |
| lithology, soil | `build_lithology`/`build_soil_fertility` **called on one-element slices** | O(1) |
| biome | `CivData::water_bodies` + `classify_biome(t, m)` | O(1) |
| control | `CivData::territory` | O(1) |

**One prior comment was wrong and is corrected in place, not deleted
quietly.** The Biome row claimed `explain_settlement()`'s doc comment meant
"retaining the rasters for arbitrary-cell queries would cost hundreds of MB at
production resolutions." That doc comment is about the *suitability* rasters
(coast SDF, river order, travel cost, the weighted terms), which genuinely are
computed and dropped inside `compute_civilisation`. Biome is not one of them:
`build_water_bodies`' classification has been retained on `CivData` since the
Settlement tool needed snap-to-water, and `classify_biome` is a pure
two-argument function over two rasters `WorldState` already holds. Nothing in
`MEMORY_OPTIMIZATION_SCOPE.md`'s budget had to move.

**Lithology and soil are derived without copying a single formula.** Both
`build_lithology` and `build_soil_fertility` are strictly per-cell (the
lithology port's own doc comment: *"Pure, single-pass, no neighbour reads"*),
so they are called on one-element slices — bit-identical to indexing the
full-grid result, with none of their golden-tested branches restated in
`cartalith-godot`. `one_cell_lithology_and_soil_match_the_full_grid` asserts
that equality at every cell of a 16×12 fixture.

**Aspect is new work and says so.** The reference's `aspectFactor` (line 7590)
is a shading scalar — a signed north-south derivative flipped by hemisphere —
not a compass bearing. The Sample panel's Aspect is the standard GIS downslope
azimuth off the same central difference. **No parity claim is made for it**,
and the first implementation was 180° out (it reported the *uphill* bearing);
`aspect_points_downhill` caught that, which is why the test exists.

**New `#[func]` surface** (`lib.rs`, one new `#[godot_api(secondary)]` block):

| method | what it is |
|---|---|
| `sample_cell(gx, gy) -> Dictionary` | every §6 field for one cell in **one** call — `on_cursor_sampled` fires on every mouse-motion event, and sixteen per-field getters would be sixteen boundary crossings per motion. Keys whose backing data genuinely is not there are **omitted, never zero-filled**. `{}` for an out-of-grid cell rather than clamping to an edge and reporting a neighbour's readings. |
| `debug_layers() -> Array` | the popover's grouped menu in the reference's own `LAYER_GROUPS` order, each row carrying `available` and its legend swatches. |
| `build_debug_texture(view) -> ImageTexture` | one field view as a grid-sized RGBA texture. **Nothing is cached** — caching all seventeen would be ~270 MB at 2048² — so re-picking re-derives. |

**Debug views: 18, in the reference's own six groups.** Base (no overlay,
elevation), Climate (temperature, rainfall), Tectonics (plates, boundaries,
tectonic type, stress, crust age, **resistance**), Hydrology (river flow,
Strahler order), Surface (biomes, terrain, lithology, soil fertility,
**slope**, **aspect**), Civilization (political control). Every ramp that
exists in the reference is ported from its own debug-overlay pixel loop
(lines 8470-8530) and palette constants — `tempColor`, `rainColor`,
`divColor`, `hsl`, `hypso`, `LITH_COLS`, `BTYPE_COLS`, `CART_BIOME_COLS`,
`CART_TERRAIN_COLS` — pinned by `ported_palettes_match_the_reference`. The
four bold ones have **no reference counterpart** (the reference's base map
*is* elevation, and it never drew slope, aspect or resistance); their ramps
are this port's own and each row's hint says so.

**The Layers popover is real.** `layers_popover.gd` (new), opened by §9's
layers button. It used to emit a signal that `app.gd` answered by selecting
the Cartography domain — a stand-in for exactly this. Nothing that stand-in
reached is removed: `cartography_workspace.gd`'s Visible-layers toggles and
`ViewportHost.set_layer_visible()` are untouched, still on the rail, and the
popover's footer points at them. The popover carries the grouped picker, the
active view's legend, and the reference's own `#dbgOpacity` slider blending
the field raster over the base map. Like the reference's, it stays open across
picks. A view whose one input this world lacks (Strahler without river
extraction, biomes/terrain/control on a loaded save) comes back
`available: false` and is drawn greyed with the reason in its tooltip, rather
than offered and then silently doing nothing.

**`available` is O(1), deliberately.** The first version answered it by
building each raster and seeing whether it worked, which at 2048² would have
derived seventeen full-grid rasters every time the popover opened.
`layer_available` reads which *inputs* exist instead, and
`available_matches_debug_raster` pins the cheap answer against the expensive
one across both civ/no-civ and both rivers/no-rivers fixtures so they cannot
disagree.

**Nothing was left open for want of retention** — no field required a raster
this pipeline computes and discards, so there is no disclosed cost estimate to
report and no `DECISIONS.md`-level change to raise.

**Tests**: 15 new plain-Rust tests in `sample_bridge.rs` (169 unit tests pass
after `cargo clean -p cartalith-godot` + full rebuild), clippy clean for the
new code. Two headless smoke runs against a real generated world: a sampler
run over six cells plus a found plate-boundary cell, asserting plausible
ranges rather than merely non-crashing (elevation in [0,1], metres in
±12 km, temperature in [−90, 70] °C, precipitation and soil in [0,1], slope
in [0, 90]°, aspect in [0, 360), a named lithology, a real plate id, biome and
control present) and all 18 views drawing at the right size and not as a flat
colour; and a full-app run that generated a 192² world, drove
`on_cursor_sampled` and read every Sample row back live (`-124 m · ocean`,
`Slope 2.1° · n 3.67`, `Aspect SW 233°`, `Plate + type 10 · oceanic`,
`Boundary + distance transform (shear) · on it`, …), confirmed every row
resets to `—` when the cursor leaves the map, built the popover's 19 rows,
picked four views, checked the legend followed, and confirmed the layers
button opens the popover. Headless Godot 4.7.1 boot clean.

**Still open, deliberately**: `DCC_SHELL_SPEC.md` §6's *Layers* right-dock
context (the ordered list with per-layer opacity bars and blend modes, nested
children under Terrain) is a different thing from this canvas popover and is
not built; §7's Layer-properties/ramp-editor panes still have no
`TerrainAppearance` binding behind them, unchanged by this pass.

## Timeline milestone 1 — shared prerequisites (`TIMELINE_SCOPE.md`, done 2026-08-19)

Population-ceiling chain + shared tier tables + stable ids, the dependency
every later Timeline milestone (proximity graph, collapse/recovery stepper,
snapshot data model, Godot boundary, UI) is blocked on. Milestone 2 is done
(below); milestones 3-6 are **not started**.

**New `cartalith-civ::timeline` module**: `civ_subsistence_mode_at`/
`civ_agrarian_density_km2`/`civ_current_agrarian_density` (the land-use-mode
density chain, `agrarianDensityKm2`/`currentAgrarianDensity`),
`civ_catchment_density_mean`/`civ_catchment_pop`/`civ_settlement_population`
(`_civCatchmentDensityMean`/`_civCatchmentPop`/`_civSettlementPopulation`),
`civ_surplus_fraction`/`civ_trade_k` (`_CIV_SURPLUS_FRACTION`/`_CIV_TRADE_K`),
`civ_tier_floor`/`civ_tier_for_population`/`CIV_TIER_ORDER`
(`_CIV_TIER_FLOOR`/`_civTierForPopulation`/`_CIV_TIER_ORDER`, capped at
`Capital` — see below), `RecoveryPhase` (`_CIV_RECOVERY_FRAC`/
`_CIV_RECOVERY_NAME`, unconsumed until `_civApplyRecovery` lands), and
`civ_assign_tid`/`civ_resync_next_tid` (`_civAssignTid`/`_civResyncNextTid`).

**Stable id (`tid`)**: `NamedSettlement`/`Way` (`cartalith-civ/src/lib.rs`)
both gained a `pub tid: u64` field, `0` = unassigned (matches JS `tid==null`).
Every construction site across the workspace updated. Real assignment is
eager, not the reference's lazy first-touch: `compute_civilisation`
(`cartalith-godot/src/lib.rs`) stamps every settlement/way right after
generation, and `civ_tools_bridge::drop_settlement` draws from the same
counter for a hand-placed settlement. The counter (`next_tid: u64`) is a new
field on `CivData` — `cartalith-civ` stays stateless per `ARCHITECTURE.md`.
This is new design (`DECISIONS.md` §7a), not a reference algorithm ported
1:1 — logged in `CHANGELOG.md`.

**Two decisions recorded** (`TIMELINE_SCOPE.md` §9, repeated in
`CHANGELOG.md` per this port's discipline): the tier table caps at
`Capital` (no `Metropolis` variant — `_civSelectMetropolises` is a separate,
still-unported gap); `_civApplyRecovery` (the v0.82 static recovery pass)
stays out of scope, left for a `PHASE2_SCOPE.md` addendum.

**Golden-verified**: `cartalith-civ/tests/golden_parity_settlement_population.rs`,
9 tests / 25 comparisons against real reference numbers (Node `vm`
extraction, transient harness, not checked in) — every function in the
chain, boundary cases on every mode/tier transition, and the documented
`metropolis`→`Capital` divergence at population 150000/5,000,000.

**Verified**: `cargo build -p cartalith-godot` (cdylib) + headless Godot
4.7.1 boot clean. `cargo test -p cartalith-civ -p cartalith-godot`: all
passing (368 across `cartalith-civ`'s lib + integration suites, 201 across
`cartalith-godot`'s). Clippy clean.

## DCC shell: Storage locations, Recent worlds, Data manager window (`DCC_SHELL_SPEC.md` §2.1/§2.4/§2.5/§9, done 2026-08-19)

The owner-supplied DCC shell spec's File/Data/Preferences file-and-folder
browsing menus, real for the first time — `menus.gd`'s own `_live()`/`_todo()`
convention preserved exactly: nothing flipped live without real behaviour
behind it, and every remaining gap keeps a stated reason.

**New persistence layer** (`shell/dcc_settings.gd`, `DccSettings`): the first
thing in this shell that writes to `user://` (grepped `ConfigFile`/`user://`/
`OS.get_user_data_dir` across `shell/` first — nothing existed). One
`ConfigFile` at `user://cartalith_settings.cfg` holds the four storage roots
(§2.1: projects, tile atlas cache, asset packs, exports) and the last-10
recent-projects list. Root defaults come from `OS.get_user_data_dir()`, not
§2.1's own literal `~/Cartalith/Worlds` etc. — that prose is macOS-flavored
and doesn't hold on Windows; read as directive intent (four separate,
sensible, per-purpose roots), not literal paths, and said so in the file's own
header comment.

**File ▸ Storage locations / Change locations… / Show project on disk**, all
real:

- **Storage locations** (File and Preferences ▸ Application both open the
  identical dialog, per §2.5's own "Same modal as File") is a read-only list
  of the four current root paths (`DccApp.open_storage_locations()`).
- **Change locations…** is a modal with one `FileDialog` (`FILE_MODE_OPEN_DIR`)
  per root; each pick writes back to `DccSettings` immediately, no separate
  confirm step. §2.1's "moving the atlas root invalidates the cache" is
  disclosed rather than faked — no tile atlas/cache concept exists in this
  port yet (Preferences ▸ Tiled LOD is itself still `_todo`), so the dialog
  says plainly that there is nothing to invalidate.
- **Show project on disk** reveals `DccApp.current_project_path`'s containing
  folder via `OS.shell_show_in_file_manager` (Godot 4.4+, present in this
  project's 4.7.1) with an `OS.shell_open("file://...")` fallback for an
  older binary. Disabled with a tooltip until a project has been opened this
  session — `current_project_path` is new Godot-side-only state, set by a new
  `DccApp._load_project()` every `open_project_picker()`/
  `open_recent_project()` call funnels through.

**Data ▸ Recent worlds**: a real submenu (§2.1: "last 10 projects, path shown
as secondary text"), rebuilt on every `about_to_popup` since the list changes
between opens (unlike the fixed-content `_quality_popup`/theme submenus this
shell already had). `DccSettings.remember_project()` moves a re-opened path
to the front rather than duplicating it; clicking an entry calls the same
`_load_project()` path `open_project_picker()`'s own file dialog uses.

**The Data manager window (§9) now exists** (`shell/data_manager_window.gd`,
`DataManagerWindow`, an `AcceptDialog` matching `world_data_window.gd`/
`performance_window.gd`'s own convention) — titled `⧉ DATA MANAGER`, subtitle
"import · export · sources · conversion · validation" verbatim, a routes rail
(the five §2.4 groups) and a route pane. `menus.gd`'s five Data-menu group
items now open it scoped to that group's first route instead of being
disabled at the menu level (`DccApp.open_data_manager(group)`). **Two routes
are genuinely real**, reusing rather than reimplementing:

- **Import ▸ World Data (.zip · fields)** opens the exact same `.zip` picker
  File ▸ Open project… already uses.
- **Import ▸ Assets** calls `open_asset_pack_picker()` directly, per §2.4's
  own table describing this item as "routes to the Assets menu."

- **Import ▸ Heightmaps (PNG)** is live as of 2026-08-20: it reads the PNG,
  takes it as the elevation field and infers a tectonic substrate under it
  (`cartalith_engine::import`), closing DM-01 and MS-02.

**Every other route stays a disclosed gap**, each with its own reason string
shown in the pane rather than generic filler: Import Maps (tiles)/GIS
(nothing reads a tile map or GeoJSON *in*; TIFF is parity-absent, since the
reference's own browser decode does not read it either), Export Maps/
GIS/World Data/Assets, Sources (no registry), Conversion (no CRS/format
conversion — the engine has one flat km projection throughout), and
Validation (no warning-collection pass exists — `load_save()` returns a plain
bool). **Export ▸ World Data's reason was verified against the crate directly
this pass, not assumed from an older comment**: `cartalith-io`'s only
`zip::ZipWriter` lives inside its own `#[cfg(test)]` fixture builder
(`build_test_zip`), not production code — still read-only, confirmed by
`grep`, not by trusting the prior "cartalith-io is read-only" note in
`menus.gd`.

**No Rust touched** — a separate concurrent pass was editing
`cartalith-civ`/`cartalith-godot` for a stable-id field at the same time
(`git status` shows `cartalith-civ/src/timeline.rs` modified by that other
work, not this one); everything above is real against the existing bridge
surface (`bridge.load_save`, `open_asset_pack_picker`) with no new `#[func]`
needed.

**Verified**: every new/modified `.gd` file parses under a headless
`--import` rescan (new `class_name` scripts need one before the global class
cache picks them up — the first `--quit` boot failed with "not declared in
the current scope" until this ran, which is itself worth remembering for the
next new `class_name` file). A scripted, discarded smoke run (`_smoke_data_mgr
.gd`/`.tscn`, deleted after) exercised: storage-root read/write round-trip;
`open_storage_locations`/`open_change_locations` opened back-to-back without
the harness itself leaving a stale exclusive dialog behind; recent-projects
dedup (`remember_project` on an already-present path moves it to front,
`item_count` stays correct, asserted); the Data manager window opened on all
five groups and every one of its 15 routes clicked, confirming the
breadcrumb and pane content match `kind` (`live`/`route`/`gap`) for each;
`show_project_on_disk` no-ops cleanly with no project opened. The test
polluted the real `user://cartalith_settings.cfg` with fake recent-project
paths — noticed and deleted afterward so a real session starts clean.
Headless Godot 4.7.1 boot (`--headless --path godot-project --quit`) clean
with the smoke files removed.

## Timeline milestone 2 — proximity graph + Brandes betweenness (`TIMELINE_SCOPE.md`, done 2026-08-19)

Fully self-contained (places array + `cellKm` in, adjacency/betweenness
out) — no dependency on milestone 1 despite sharing `cartalith-civ::
timeline`. Genuinely new Rust: no Brandes betweenness-centrality
implementation existed anywhere in the workspace before this.

**New functions**: `civ_proximity_adjacency` (`_civProximityAdjacency`,
reference lines 24672-24683 — symmetric k-nearest-neighbour graph among
settlement positions in real km, world-wrap aware; takes bare `(x,y)` pairs
rather than a domain struct, matching the crate's existing "just positions"
idiom) and `civ_betweenness_from_adjacency` (`_civBetweennessFromAdjacency`,
24687-24709 — textbook unweighted Brandes 2001, raw/un-normalised, no
divide-by-2 for the undirected graph, matching the reference exactly; the
reference's redundant `n` parameter dropped in favour of `adj.len()`).
World-wrap distance reuses this crate's existing pattern
(`civ_passed_settlements`'s `dx.min(gw-dx)` gated by a caller-supplied
bool), not a newly invented helper. `js_hypot`/`js_min` (`cartalith-jsmath`)
used call-for-call against the reference's `Math.hypot`/`Math.min`.

**Golden-verified**: `cartalith-civ/tests/golden_parity_timeline_graph.rs`,
6 tests against real reference output (Node `vm` extraction, transient
harness) — a 3-node path (also independently hand-derived, betweenness
`[0,2,0]`, confirming the no-divide-by-2 reading), a 5-node chain, a
world-wrap pair (adjacent only with wrap on), a world-wrap 4-cycle (closes
into a cycle only with wrap on, betweenness ties `[1,1,1,1]` vs. the
no-wrap path's `[0,4,4,0]`), and an 8-settlement 512×328/800km fixture at
two `k` values, one of which produces a disconnected graph (Brandes across
multiple components, no cross-component leakage).

**Verified**: `cargo build -p cartalith-godot` (cdylib) + headless Godot
4.7.1 boot clean. `cargo test -p cartalith-civ`: all passing (21 `timeline`
unit tests + 6 new golden tests, 0 regressions). Clippy clean on every line
this milestone added (two pre-existing `needless_range_loop` findings in
unrelated `lib.rs` code, and the one pre-existing `1 * gw` finding
milestone 1 already logged, are untouched by this milestone).

**Out of scope**: the collapse/recovery step functions (milestone 3 — the
real caller of both new functions), snapshot data model (milestone 4),
Godot boundary (milestone 5), UI (milestone 6). Nothing calls either new
function yet.

## Asset library window — closing the GUI gap (`DCC_SHELL_SPEC.md` §2.3/§8, 2026-08-19)

`ASSET_LIBRARY_SCOPE.md` closed Phase 4's engine side 2026-08-17 but carved
the authoring UI out as later work. That window now exists
(`shell/asset_library_window.gd`, `AssetLibraryWindow`) — `Assets ▸
⧉ Asset library` (⇧A) / `▦ Sprite sheet slicer` are `_live` in `menus.gd`.

**Confirmed discrepancy**: §8's own prose says "24 families." The shipped
`cartalith-assets` (`slots.rs`/`library.rs`) has **eight** — `textures`,
`biomes`, `terrains`, `icons`, `settlement`, `trait`, `poi`, `custom` —
already recorded by `ASSET_LIBRARY_SCOPE.md` §1 and re-verified this pass by
a headless smoke run asserting each family's grid populates with the real
frozen slot count (7/15/13/10/9/7/10/0-open, 71 frozen slots total). The
window's family rail lists the real eight, grouped by `Family::is_texture()`
and the `structures.*` trio, not the mockup's uncoded 24.

**Almost everything is a disclosed gap, and the reason is one gap**:
`cartalith-godot/src/lib.rs` exposes exactly two asset `#[func]`s —
`load_asset_pack(path)` and `has_asset_pack()`. No live `AssetDB` crosses the
Godot boundary, so per-slot fill state, thumbnails, variants, tags, scale,
and pack metadata are all disclosed rather than guessed (the slot grid draws
every slot as a checkerboard on principle). Apply to map, Export pack .zip,
batch edit, Validate and Clear library are gaps for the same reason — there
is no in-memory library session for any of them to act on. **Real**: the
family/slot list and metadata (verbatim frozen constants), search/sort,
multi-select, the pack-loaded status line, Import asset pack .zip…, and the
sprite-sheet slicer's image load + grid-overlay math (Godot's own `Image`
loader, no engine call). The slicer's actual slice op is a gap — no
sheet-splitting function exists anywhere in `cartalith-assets`.

No Rust file touched (verified via `git diff --stat -- cartalith-native/
crates`, empty). Headless Godot 4.7.1 boot clean; a scripted, discarded
smoke run opened the window, selected all eight families, exercised
click/⇧-range multi-select, and opened the slicer modal without error.

## Timeline milestone 3 — collapse/recovery step functions (`TIMELINE_SCOPE.md`, done 2026-08-19)

The mechanistic core of the v0.85 stepper, depending on milestones 1 and 2.
Milestones 4-6 (snapshot data model, Godot boundary, UI) are **not started**.

**New in `cartalith-civ::timeline`**: a settlement-only `CollapsePlace` type
(`tid`/`x`/`y`/`kind`/`pop`/`fortified`/`ruins`) decoupled from
`NamedSettlement` the same way milestone 2 decoupled the graph functions from
any domain struct — `NamedSettlement` has no `traits`/`ruins` fields and
never will just for this stepper's sake. `CollapseCharacter`
(`_CIV_COLLAPSE_CHAR_WEIGHTS`/`_CIV_COLLAPSE_MIGRATION_BIAS`, a closed enum
in place of the reference's string-keyed lookup + `mixed` fallback),
`civ_settlement_stress` (`_civSettlementStress`), `civ_mortality_migration_rates`
(`_civMortalityMigrationRates`), `civ_gravity_migrate` (`_civGravityMigrate`,
the up-to-4-pass saturating Zipf/Ravenstein redistribution), `civ_collapse_step`
(`_civCollapseStep`) and `civ_recovery_growth_step` (`_civRecoveryGrowthStep`).

**Because this port's place type is settlements-only**, `civ_collapse_step`/
`civ_recovery_growth_step` skip the reference's `p.category==='settlement'`
filter-and-reassemble dance over a mixed places array — every input entry is
already a settlement. Disclosed as a structural simplification, not a
behavior change (`timeline.rs`'s own top-of-milestone-3 doc comment).

**The reference's `_K`-null fallback branches are dropped**, matching
milestone 1's already-logged precedent for `_civCatchmentPop`'s dead `K`
parameter: `currentCarryingCapacity` is a hoisted top-level function, always
defined in the real app, so `_K?...:...`'s false branch never executes.
This port's step functions always compute the capacity ceiling via
`civ_settlement_population`, with real `dens`/`field` arrays as the caller's
responsibility (same contract milestone 1 already established).

**A real algorithmic surprise, verified against the actual reference lines
rather than trusted from the scope doc's own summary**: `TIMELINE_SCOPE.md`
§5 describes `civ_collapse_step` as "re-derives tiers" without qualifying
direction. Reading the reference directly (line 24826: `demoted =
_CIV_TIER_ORDER.indexOf(newKind) > _CIV_TIER_ORDER.indexOf(p.kind)`, and only
the `demoted` branch ever updates `p.kind`) shows collapse can only ever
**demote**, never promote, even if a step's mortality/migration math somehow
left a settlement's new population high enough to clear a higher tier's
floor. `civ_recovery_growth_step` is the mirror image — promotion-only,
confirmed the same way. Both directions are now a named, tested invariant
(`collapse_step_never_promotes_even_if_population_would_clear_a_higher_floor`/
`recovery_growth_step_never_demotes_even_if_population_would_clear_a_lower_floor`
in `timeline.rs`'s own unit tests), not an assumption.

**`fortified` is sticky, `ruins` is not**: a demoted former exchange-tier
(`City`/`Capital`, capped per milestone 1's metropolis decision) nucleus
gains `ruins=true` and `fortified=true`; recovery's promotion back into an
exchange tier clears `ruins` but never `fortified` — matching the reference,
which never removes a trait once added (only `ruins` — via `delete
p.ruins` — ever clears). Golden-verified both ways: promotion into City
clears ruins while keeping fortified; promotion into the non-exchange Town
tier keeps ruins set.

**Golden-verified**: `cartalith-civ/tests/golden_parity_timeline_collapse.rs`,
9 tests against real reference numbers (Node `vm` extraction slicing the
milestone-1 chain + the whole v0.85 stepper block verbatim, transient
harness, not checked in) — the abandonment floor at pop0=21/22/23 (newPop
19/20/21, `_CIV_ABANDON_FLOOR=20`'s `<` boundary exact); a fortified-vs-
unfortified destination pair at equal distance/headroom receiving exactly a
1.5x ratio (`_CIV_FORTIFIED_BONUS`); the gravity model's multi-pass
saturation actually engaging (a near destination saturates at its headroom,
the remainder re-offered to a farther one) plus a genuine unplaced/diaspora-
loss case; all four characters on one HUB/DENSE/UNDEFENDED/FORTRESS fixture,
both at the raw-stress level (with a caller-supplied baseline proving the
`L` trade-loss term — trade ranks HUB worst, disease inverts the ranking
entirely, the design doc's own central claim) and end-to-end through
`civ_collapse_step` (`failed` counts of 0/1/2/1 across trade/disease/
conflict/mixed on the identical fixture, exact `died`/`migrated`/`unplaced`
numbers and exact surviving tids/populations for each); a recovery step
promoting a `ruins`+`fortified` Town into City (ruins clears, fortified
stays) contrasted with one promoting Village into Town (ruins stays, Town
is not an exchange tier). Plus 7 new unit tests in `timeline.rs` itself
(character weights sum to 1, an unassigned `tid=0` never does a baseline
lookup even with a populated map, the empty-places no-op, both demote-only/
promote-only invariants, gravity migrate's true no-op case).

**Verified**: `cargo build -p cartalith-godot` (cdylib) + headless Godot
4.7.1 boot clean (exit 0, no errors). `cargo test -p cartalith-civ`: all
passing (303 lib tests incl. 28 `timeline` unit tests, 9 new golden tests, 0
regressions). Clippy clean on every line this milestone added (two
`neg_cmp_op_on_partial_ord` findings in `civ_gravity_migrate` deliberately
kept and `#[allow]`ed with a comment — `!(x > y)` is not `x <= y` when `x`
can be NaN, and the reference's own falsy-check semantics require the
NaN-inclusive reading).

**Out of scope**: `_civSimulateTimeline`/`_civRunCollapseSimulation`
(milestone 4's orchestrator/wiring), the snapshot data model (milestone 4),
the Godot boundary (milestone 5), UI (milestone 6). Nothing calls any of
this milestone's functions yet.

## Timeline milestone 4 — snapshot data model + orchestrator (`TIMELINE_SCOPE.md`, done 2026-08-19)

Split exactly along the crate boundary the scope doc predicted: pure
orchestrator/snapshot/diff logic in `cartalith-civ::timeline` (stateless,
takes/returns explicit values), real mutable state (`timeline: Vec<
TimelineSnapshot>`, `year: i64`) plus thin calling methods on `CivData` in
`cartalith-godot/src/lib.rs` — no `#[func]`/`Variant` surface, milestone 5's
job. Depends on milestones 1 (stable `tid`) and 3 (`civ_collapse_step`/
`civ_recovery_growth_step`).

**`civ_assign_tid`/`civ_resync_next_tid` were already half-built by
milestone 1** — checked per the task's own instruction, not assumed. What
milestone 1 couldn't build was `_civResyncNextTid`'s timeline-history half
(reference lines 20569-20571 scan `civTimeline` entries too), since
`TimelineSnapshot` didn't exist yet. This milestone adds a sibling,
`civ_resync_next_tid_with_timeline`, rather than widening the existing
function's signature (real callers/tests already depend on the two-argument
shape). No caller wires it in yet — nothing in this port reloads settlement
lists out from under the counter today.

**Built in `cartalith-civ::timeline`**: `TimelineSnapshot{year,territory,
settlements,ways}` (dense `Vec<i32>` territory, not the reference's sparse
save-format encoding — a disclosed simplification, since this in-memory
struct doesn't share the reference's save-payload-size concern);
`civ_year_diff` (tid-set diff against the chronologically-previous year, no
memoization cache — the reference's own cache needs five separate
invalidation call sites to stay correct); `civ_snapshot_save`/
`civ_snapshot_load` (upsert-and-sort; restore territory only, filling 0
first); `civ_simulate_timeline` (`_civSimulateTimeline`, 24875-24892) — runs
`opts.steps` collapse-or-recovery steps, `baseline_norm_b` captured only at
`t==0` and reused unchanged for every later step, read directly off the
reference's own `if(t===0)` guard rather than assumed.

**Built on `CivData`**: `civ_goto_year`/`civ_add_year`/`civ_remove_year`
(reference lines 20615-20641, minus UI rebuild calls — milestone 6) —
`civ_add_year`'s four real cases (empty-timeline seed, snapshot-current-
year-first, don't-clobber-an-existing-year, carry-forward-from-nearest-
earlier-year) all read directly off the reference, not inferred from the
scope doc's summary; `civ_remove_year` falls back to the earliest remaining
year, or 0 if none remain.

**A deliberate, logged deviation**: `civ_add_year` caps recorded years at
`TIMELINE_MAX_YEARS = 2000` (`TIMELINE_SCOPE.md` §9's own in-flight
decision) — a no-op past the cap, never a data loss, since the active
year's live state is already snapshotted before the check runs.

**Golden-verified against the real reference**
(`cartalith-civ/tests/golden_parity_timeline_orchestrator.rs`, 4 tests): a
Node `vm.runInContext` harness (transient, same convention as milestones
1-3) sliced the population-ceiling chain plus the v0.85 stepper block PLUS
the orchestrator itself (24614-24892). One false start worth recording: the
first harness run produced all-zero stats for every step because
`_civCollapseStep` filters `places` to `p.category==='settlement'` and the
harness fixtures didn't set it — caught by inspecting suspiciously-uniform
zero output, not a passing-but-wrong test. Fixtures: collapse `mixed`
character over 3 steps (step 0's numbers match milestone 3's own single-step
golden exactly; baseline-normB threading proven across steps 1-2); collapse
`trade` character, 2 steps, a different severity; recovery, 2 steps of 50
years each (final pop/kind, 6211/City, match milestone 3's own single-
100-year-step number exactly, confirming step-to-step chaining is
equivalent to one longer step); `opts.steps` omitted clamps to exactly 1
step, matching milestone 3's own `conflict` numbers verbatim.

Plus 6 new unit tests in `timeline.rs` (snapshot save/load semantics;
`TIMELINE_SCOPE.md` §7 success criterion 3's own named case — tid, not
name/position, disambiguates a settlement that disappears from a
same-name/same-position DIFFERENT settlement that appears in its place;
year-diff against the earliest year and an unrecorded year;
`civ_resync_next_tid_with_timeline`) and 8 new unit tests in
`cartalith-godot/src/lib.rs`'s `civ_timeline_tests` module covering success
criterion 2 directly (adding a year never loses the active year's live
edits; `civ_goto_year` never mutates settlements/ways, tested with the live
array deliberately diverged from both recorded snapshots so a restore bug
would be caught; `civ_remove_year`'s fallback behavior).

**Verified**: `cargo build -p cartalith-godot` (cdylib) + headless Godot
4.7.1 boot clean. `cargo test -p cartalith-civ`: 309 lib tests (+6) plus the
new 4-test golden file, 0 regressions. `cargo test -p cartalith-godot`: 178
lib tests (+8), 0 regressions. `cargo clippy -p cartalith-civ -p
cartalith-godot --all-targets`: clean — every warning shown is pre-existing
in files this milestone didn't touch.

**Out of scope**: the Godot boundary (`timeline_bridge.rs`, milestone 5) and
UI playback controls (milestone 6) are untouched; nothing calls
`civ_add_year`/`civ_remove_year`/`civ_year_diff`/`civ_simulate_timeline`
outside this milestone's own tests. Save-format persistence remains
deferred per `TIMELINE_SCOPE.md` §9.

## Timeline milestone 5 — the Godot boundary (`TIMELINE_SCOPE.md`, done 2026-08-19)

New godot-free `cartalith-godot/src/timeline_bridge.rs`
(`journey_bridge.rs`'s exact isolation pattern — no `godot` dependency, its
own `#[cfg(test)]` suite runs under `cargo test -p cartalith-godot --lib`
with no Godot runtime), plus a new `#[godot_api(secondary)]` block in
`lib.rs` owning the thin `Variant` conversion. 7 `#[func]`s on `WorldGen`:
`civ_add_year`/`civ_goto_year`/`civ_remove_year` (thin wrappers over
`CivData`'s milestone-4 methods, no new logic), `get_civ_year`/
`get_civ_timeline_years` (small getters for milestone 6), `civ_year_diff`
(passthrough to `CivData::civ_year_diff`), and `civ_run_collapse_simulation`
— the one real new wiring (`timeline_bridge::run_collapse_simulation`, a
straight port of `_civRunCollapseSimulation`'s impure half, reference lines
24896-24950).

**The warn-before-overwrite case** (reference's blocking `confirm()` dialog,
lines 24910-24911): no prior "confirm before overwrite" precedent existed
anywhere in this port to match (checked: only unrelated `confirm`/`overwrite`
hits across every bridge module). New design — a first call whose simulated
years would land on already-recorded entries returns `{"ok": false,
"needs_confirm": true, "clobber_years": [...]}` and writes nothing; the
caller re-sends the identical request with `confirm_overwrite: true` to
proceed. Same "a response field the caller checks" shape `jp_compute`'s own
`rejected` array already establishes.

**The anchor/carry-forward claim verified against the real reference, not
just the task brief's summary of it** (lines 24915-24925): because the
"before" frame at `start_year` is always written from live state before the
anchor search runs (if none existed there already), the anchor always
resolves to exactly the `start_year` entry — either a pre-existing one (the
one case genuinely distinguishable from "just use the live grid") or the
live snapshot just captured there. An early test draft assumed an
earlier-year anchor was reachable in a single call; it isn't, caught by the
test itself failing rather than assumed correct, and rewritten to the
actually-reachable case.

**A disclosed, out-of-scope gap**: `CollapsePlace` (milestone 3) carries
`fortified`/`ruins`; `TimelineSnapshot` stores `Vec<NamedSettlement>`
(milestone 4), and `NamedSettlement` (pre-Timeline, Phase 2) has neither
field. Extending it would ripple into every other subsystem that constructs
one — out of this milestone's scope ("do NOT touch milestones 1-4's
already-committed functions"). Threaded correctly *within* one simulation
run (the orchestrator chains `CollapsePlace`, never touching
`NamedSettlement` mid-run); lost only in what gets stored for later
scrubbing. Inert today (milestone 6 isn't built) — flagged for whichever
future milestone extends `NamedSettlement`/`TimelineSnapshot`.

**One additive field, not a refactor**: `CivData` gains `dens: Vec<f32>`
(`civ_current_agrarian_density`'s per-cell output), computed once in
`compute_civilisation` from locals (`carrying_cap`/`water_access`/`biome`)
that function already builds — the same reasoning `water_bodies` was
already kept for. Without it, every simulate call would have re-run the
soil/water-access/biome sub-pipeline. Milestones 1-4's own functions
untouched; only `CivData`'s struct (already extended twice before) and
`compute_civilisation` (Phase 2 infrastructure, not a Timeline deliverable)
grew by one more field.

**GDScript**: all 7 methods wired into `engine_bridge.gd` with the standard
`has_method()` guard, ready for milestone 6 — no UI built.

**Verified**: `cargo test -p cartalith-godot --lib`: 189 lib tests (+11), 0
regressions, no Godot runtime for any of them. `cargo test -p cartalith-civ
--lib`: 309 tests, 0 regressions (calls but doesn't modify `cartalith-civ`).
`cargo build -p cartalith-godot` (cdylib) + headless Godot 4.7.1 boot clean.
`godot --headless --check-only --script shell/engine_bridge.gd`: the
GDScript addition parses clean. `cargo clippy -p cartalith-godot
--all-targets`: clean — every warning shown is pre-existing.

**Out of scope**: milestone 6 (UI playback controls) untouched — nothing in
the shell calls any of these 7 methods yet. Save-format persistence remains
deferred per `TIMELINE_SCOPE.md` §9. The `fortified`/`ruins` snapshot gap
above is disclosed, not fixed.

## Two owner-reported rendering bugs: deep-zoom tile drops, settlement-pin fidelity (owner report, done 2026-08-19)

`viewport_host.gd`/`map_overlay.gd`/`tool_overlay.gd` only, GDScript-only —
`git status` re-checked before editing and re-checked after; no Rust file
touched. Both bugs root-caused against a real headless repro before fixing,
not fixed on assumption; both repros were run again after the fix to
confirm the real before/after numbers below.

**Bug 1 — deep-zoom tiles above `MAX_LOD_TILES_PER_UPDATE` (48) were
dropped and never retried.** Confirmed exactly the mechanism the constant's
own doc comment had already flagged as a risk: `_apply_lod_tiles`'s
reconciliation only ever compared the *current* call's (already-trimmed)
`wanted` against `_lod_tiles`, so a tile trimmed out of `wanted` was never
revisited once the camera stopped moving — permanently missing, not merely
delayed. A second defect in the same code: trimming `wanted` itself (not
just the synthesis budget) also freed already-built, still-visible tiles
outside the closest-48 cut. Repro (grid 512, 1920×1080 viewport, default
zoom wants 64 tiles): `_lod_tiles.size()` stuck at 48/64 across five
redundant `_update_lod()` calls, before the fix. Fixed: `_update_lod()` now
trims only the *missing* subset for the per-call synthesis budget, leaving
the full `wanted` set (and so the free/reposition passes) untouched; the
overflow goes into a new `_lod_backlog`, drained `MAX_LOD_TILES_PER_CATCHUP
:= 6` at a time by a new `_process()` override (disables itself once the
backlog empties). Same repro, post-fix: 48 built + 16 backlogged
immediately, draining to 64/0 over 3 simulated `_process()` frames, stable
after a further redundant `_update_lod()` call.

**Second bug found in the same area** (checked for more than the one
hypothesised bug, per instruction): a tile already built at one
`detail_level` was never rebuilt when the camera zoomed further within the
*same* tile index — matched purely by tile-index key, so a 256px tile kept
covering an ever-larger screen rect instead of being replaced by the
512px/1024px tier that zoom level calls for. Fixed with a new
`_lod_tile_detail` dictionary (tile key → the tier it was actually built
at); a mismatch now frees and rebuilds under the same budget a new tile is.

**Ruled out**: `_lod_tile_cells` being fetched once (not once per world) is
harmless — `lod_bridge::TILE_CELLS` is a fixed Rust constant, never tied to
which world is loaded. Tile boundary clipping and stale-tile cleanup on
world reset/resize were re-checked and are correct as originally shipped
(`59700ab`) — not touched.

**Bug 2 — settlement pins had no inverse-camera-zoom compensation, so their
on-screen size grew unboundedly with zoom instead of holding roughly
constant.** Root-caused against the reference's real formula (line
14980-14983: `_civZoomK() = 1/clamp(viewT.scale, 0.35, 5)`), not the port's
own paraphrase of it — the paraphrase was directionally right but
`map_overlay.gd`'s own doc comment had drawn the wrong conclusion from it,
arguing the term could be dropped because "camera zoom is a plain
transform" the port's `_camera.scale` already applies; that transform is
exactly *why* the reference needs the compensating term (so the two
cancel), and this port's pin radius had nothing supplying it. Confirmed
numerically: at zoom 8.0 the unfixed formula would have rendered a pin at
exactly 8x its zoom-1.0 on-screen radius (no zoom term anywhere in the
formula). Fixed: `map_overlay.gd` gained `_camera_zoom`/`set_camera_zoom()`
(triggers `queue_redraw()`, since Godot rescales a `CanvasItem`'s *cached*
draw commands by an ancestor's transform rather than re-running `_draw()`)
and `_civ_zoom_k()` (the reference's own `1/clamp(z, 0.35, 5)`, using the
reference's clamp bounds rather than this port's own wider `ZOOM_MIN`/
`ZOOM_MAX`); `_settlement_pin_radius()` and `_draw()`'s inline `sc` both now
multiply by it; `viewport_host.gd`'s `_zoom_at()`/`reset_view()` push the
live zoom in on every change. Verified numerically post-fix: on-screen
radius at zoom 1.0 and 4.0 (both inside the reference's clamp) measured
identical (4.6286px, ratio 1.0000); at zoom 8.0 (past the reference's own
5.0 ceiling) the ratio from 4.0 was 1.6, not 2.0 — partial, clamped growth
matching the reference's documented intent, not unconstrained growth.

**Same pass**: every settlement/icon/measure-point/handle `draw_circle`
call in `map_overlay.gd`/`tool_overlay.gd` now passes `antialiased = true`
(Godot 4 defaults it `false`) — `draw_arc` calls already had it;
`draw_colored_polygon` has no such parameter.

**Verified**: every modified file parses (`--check-only --script`), a full
headless boot is clean before and after. Two scripted, discarded smoke
scenes (not committed) drove `EngineBridge.generate()` and a manually-
`_ready()`'d `ViewportHost`/`map_overlay.gd` pair directly (outside the
scene tree, so `_process()`/zoom pushes were called explicitly) to produce
every number above. `LOD_TILING_INTEGRATION_SCOPE.md`'s M1 section carries
the same detail.

## DCC shell GUI audit (owner request, done 2026-08-19)

Full pass over every menu/window/workspace file under `godot-project/shell/`
against `DCC_SHELL_SPEC.md`/`DCC_CONTROL_INDEX.md`, looking for the exact
class of bug already found once this session (`f274d13`: two menu items,
`File ▸ Storage locations` and `Change locations…`, opening two dialogs that
showed the same four rows) — redundant surfaces, stale `_todo()` reasons,
dead ends, and cross-file convention drift. `viewport_host.gd`/
`map_overlay.gd` excluded (a concurrent agent's own rendering-bug pass).

**Findings, all fixed:**

1. **Right dock chrome title was hardcoded `"LAYERS"`** regardless of
   context (`dcc_shell.gd`'s `_build_right_dock`) — misleading once every
   context `right_dock.gd` actually dispatches (Sample, Settlement, Route,
   River, Faction, Measure, Region select, Stamp stack, Journey) draws its
   own real section header one scroll-step below it. Fixed: a new
   `DccShell.right_dock_title`/`set_right_dock_title()` pair, kept live by
   `RightDock._rebuild()` the same way `left_dock_title` already tracks the
   active domain.
2. **`Assets ▸ Asset library` / `▦ Sprite sheet slicer`'s menu glyph was
   wrong** — both prefixed with `DccIcons.SYMBOLS["panels"]` (▤, the phone
   app-bar's own Panels button glyph), not §2.3's own `⧉` "opens a dedicated
   window" marker. Fixed to the literal `⧉` text, matching how every window
   this shell opens already marks its own title (`"⧉ ASSET LIBRARY"`,
   `"⧉ DATA MANAGER"`).
3. **`data_manager_window.gd`'s `export_assets` route reason was stale** —
   referenced `Assets ▸ Asset pack ▸ Build ▸ Export pack .zip…` (a submenu
   path that was never built into `menus.gd`) and claimed the asset-library
   window "is not built", which stopped being true the moment
   `asset_library_window.gd` shipped this session. Corrected to name the
   real location: that window's own (also honestly disabled) window-bar
   button.
4. **`PerformanceWindow` was completely unreachable** — built, instantiated
   in `app.gd`, `DccApp.open_performance()` existed, and nothing anywhere
   called it. It was also a bare "being ported" placeholder despite real,
   already-bound data (`EngineBridge.gpu_stages_used()`,
   `quality_tier()`/`quality_tiers()`/`recommended_quality_tier()`,
   `OS.get_static_memory_usage()`) sitting unused. Fixed: real content (GPU
   stages the last generate actually dispatched, current/recommended
   quality tier, working-set memory), wired to a new
   Preferences ▸ Memory ▸ **Working set…** item.
5. **`Data ▸ World data tables…` was a live, enabled menu item opening a
   placeholder** — the one case in this shell where a non-`_todo()`'d item
   delivered nothing, contradicting this shell's own honesty convention
   (`menus.gd`'s header comment: disabled-with-reason, never "enabled and
   silently inert"). `WorldDataWindow` now builds three real, filterable
   tables (Settlements/Provinces/Economy) straight off
   `bridge.settlements()`/`provinces()`/`trade_balances()` — the same real
   data `civilization_workspace.gd`'s own categories already read, just
   capped there at a top-N summary; this window is the uncapped view.
6. **Preferences ▸ Memory's other two items were missing outright** —
   §2.5 names three (Undo history, Working set, Clear caches); only Undo
   history ever made it into the menu, not even as a disabled placeholder
   for the other two. Fixed alongside finding 4: Working set is now live,
   Clear caches is now an honest `_todo()`.

**Verified**: every modified file re-read after editing; a headless Godot
4.7.1 boot (`--headless --path godot-project --quit`) clean, no parse/
registration errors; a scripted drive (temporary, not committed) generated a
small world and exercised all six fixes directly — right-dock title toggles
`SAMPLE → SETTLEMENT → SAMPLE` on selection/deselection, `WorldDataWindow`
built 127/9/128 real rows across its three tabs, `PerformanceWindow` opened
with real content both from its own `open()` and via the new Preferences
menu item, and `DataManagerWindow`'s corrected reason string read back
exactly as written. No Rust touched.

## Right dock: RD-03/RD-06/RD-08/RD-11 (`GUI_GAP_REGISTER.md` §10 ranks 1, 2, 5, done 2026-08-19)

- [x] **RD-03 — Settlement ▸ Economy / Politics / Logistics.** Were three
      permanently-`disabled` buttons (`right_dock.gd:447-449`) with the
      reason "no per-settlement panel exists yet" — accurate when written,
      obsolete once `world_data_window`/`show_faction()`/
      `open_journey_planner()` all shipped. Now live: **Economy** →
      `app.open_world_data("Economy")`, a new `tab` parameter on both
      `DccApp.open_world_data()` and `WorldDataWindow.open()` that selects a
      tab by title, mirroring `DataManagerWindow.open(group)`'s existing
      "scope to X, empty picks the default" shape (`app.gd`,
      `world_data_window.gd`); **Politics** → `show_faction(int(s.get
      ("faction", 0)))`, this same file's own Faction context; **Logistics**
      → `app.open_journey_planner()`, which was already updated ahead of this
      pass (`DCC_SHELL_SPEC.md` §4.5.4, 2026-08-19) to arm the JOURNEY tool
      takeover rather than open a dialog — confirmed, not re-built.
- [x] **RD-06 — Faction ▸ Territory.** Was a permanent "—" with a comment
      explaining the queries now exist but the dock predates them
      (`right_dock.gd:608`). Now reads `bridge.civ_faction_territory_stats
      (faction)` live — `"%d cells · %.0f km² · %d contested"`, the same
      call and format `civilization_workspace.gd`'s own CIVIL ▸ Territory
      tool-options row already uses — with an honest "—" only when the
      faction has committed no territory at all (empty dict, not a zeroed
      one).
- [x] **RD-08 — Faction ▸ Roster.** Read a comma-joined list of province
      names, which said who claims the faction, not anything about the
      faction itself. §6 calls the field a "roster entry" (singular) —
      switched to `bridge.get_factions()`'s real per-faction row: Culture
      (capitalised `culture` key), a Colour row (new `_faction_colour_row()`,
      an 11×11 `ColorRect` + hex label, same swatch shape `layers_popover
      .gd`'s own legend already uses), and Settlements (`settlement_count`).
      Provinces (the count, not names) stayed as its own field.
- [x] **RD-11 — Right dock's collapsed primary readout.** `DccShell
      .set_dock_readout("right", …)` existed and was wired for the left dock
      only (`world_workspace._push_dock_readout()`); `right_dock.gd` never
      called it. New `_push_dock_readout()` / `_dock_readout_text()` at the
      end of every `_rebuild()`, plus a live update inside `on_cursor_sampled`
      so the Sample elevation reading doesn't go stale between rebuilds (the
      same live-in-place pattern the row labels themselves already use). One
      real reading per context that actually exists: elevation (Sample),
      settlement name (Settlement), faction id + culture (Faction), route
      length (Route), chain/region/stamp counts (Measure/Region/Sculpt),
      journey days·km (Journey, via a new `JourneyPlannerView.readout_text()`
      that reads its own private `_last_result` rather than exposing it). No
      "Layers" context exists yet (RD-10, still an omission), so §6's "layer
      dots for Layers" line has nothing to read from.

**Verified**: all three edited files (`right_dock.gd`, `app.gd`,
`world_data_window.gd`) plus the one-method addition to
`journey_planner_view.gd` re-read after editing. Headless Godot 4.7.1 boot
(`--headless --path godot-project --quit`) clean. A scripted drive
(temporary `_rd_audit.gd`, not committed, same pattern `595582d`'s own
changelog entry describes) booted the real `app.tscn` shell, generated a
64×64/200 km world, selected a real settlement, and exercised all four
fixes by calling into the live tree: clicking Economy opened
`world_data_window` already on its Economy tab; clicking Politics switched
the dock to `CTX_FACTION` with the settlement's real faction id and a
`FACTION` chrome title; clicking Logistics set `armed_tool` to `"journey"`;
the Faction field dump read `Culture: Imperial`, `Settlements: 5`,
`Territory: 180 cells · 1758 km² · 0 contested`, `Provinces: 1`, and a
`#E69F00` colour swatch; and the right-dock readout read `"1 · Imperial"` in
Faction context, `"—"` after deselecting back to an empty Sample, and
`"-154 m · ocean"` after one cursor sample. No Rust touched.

## GUI gap register (`GUI_GAP_REGISTER.md`, owner request, done 2026-08-19)

- [x] **Layer 1 — the complete verified catalogue.** All 18 files under
      `godot-project/shell/` + `shell/workspaces/` (13 112 lines) read in
      full. **123 catalogued gap entries** in 15 tables, each with file:line,
      UI label, current disclosed reason, and **whether that reason is still
      accurate**. Raw count of individually disabled controls is ~180.
- [x] **Layer 2 — design coverage**, every entry classified:
      **(A) 17** designed + engine-ready · **(B) 71** designed but
      engine-blocked (each naming the specific missing function/crate/`#[func]`,
      verified by opening the crate; second axis **wrapper 22 / small 21 /
      large 28**) · **(C) 23** undesigned · **(D) 12** deliberate owner
      decision, no design proposed.
- [x] **Layer 3 — comparable-application research** for all 23 (C) entries.
      10 web searches; every attributed claim carries its source URL; every
      entry ends in a proposal precise enough to build from. Three of them
      recommend **cutting** rather than building (Data ▸ Conversion, §5.2's
      Stroke & grid block, Sources' third row).
- [x] **Menu-naming audit** against `DCC_SHELL_SPEC.md`'s own names and the
      comparables. The shipped shell matches the spec exactly (three
      documented divergences, one omission), so every finding is about the
      spec — recommendations only, renaming is an owner decision.
- [x] **Five stale disclosed reasons corrected** (reason text only; no control
      changed state, no behaviour changed; none overlaps `595582d`'s six):
      `right_dock.gd` Faction ▸ Territory, `app.gd`'s CIVIL/INFRA idle
      tool-options text, `journey_planner_view.gd`'s Cost group, `menus.gd`'s
      Tiled LOD tooltip, `world_workspace.gd`'s Finalize tooltip.
- [x] **Verified**: `#[func]` surface re-enumerated across all 15 modules
      (**151 methods**, vs the 38 `DCC_CONTROL_INDEX.md` counted); eight
      classification-changing claims opened line-by-line and cited in the
      register's §12; headless Godot 4.7.1 boot clean after all five edits;
      `git diff` over `cartalith-native/crates/` empty.
- [ ] **Follow-up, recommended first**: `timeline_bar` is drawn **visible and
      empty** in CIVIL and INFRA (`app.gd`'s `_on_workspace_changed`;
      `dcc_shell.gd`'s `_build_timeline()` builds an empty `timeline_row`;
      `TIMELINE_SCOPE.md` §4 explains why milestone 6 built its own panel
      instead). A 70 px empty strip with no disclosure — the one place the
      shell shows a region with nothing in it and says nothing about why. Two
      honest fixes exist (hide it until something fills it, or put a one-line
      `_todo()`-style disclosure in it); the register's (A) item **JP-13**
      fills it outright for INFRA.
- [x] **The (A) list, ranks 1, 2 and 5 — RD-03, RD-06, RD-08, RD-11, done
      2026-08-19.** Settlement ▸ Economy/Politics/Logistics now open their
      real destinations; Faction ▸ Territory/Roster now read
      `civ_faction_territory_stats()`/`get_factions()` live; the right dock's
      collapsed primary readout is wired. See "Right dock: RD-03/RD-06/
      RD-08/RD-11" above.
- [ ] **The (A) list, remaining 13 entries**, in the register's §10 priority
      order: the Journey Planner's timeline band (rank 3); its blocked-stage
      inline resolutions (rank 4); the **light theme** (rank 6,
      `DccTheme.LIGHT` is fully defined — only the build-once stylebox pass
      blocks it, the largest visible change available with no engine work);
      the Window menu's workspace list/open-windows list/dock-width dragging
      (rank 7); and eight more, §10's own table has the full list.
- [ ] **Nine omissions** — designed surfaces absent entirely, so the menus
      cannot teach a reader that the port owes them: `Data ▸ ⧉ Travel
      library… ⇧L`; `Assets ▸ Asset pack ▸`'s whole 24-control submenu;
      `Preferences ▸ Fallback when VRAM full`; `Theme ▸ follow system`;
      `Window ▸` workspace list and open-windows list; the New world dialog's
      project **name** field; the Journey Planner's timeline band and
      blocked-stage resolutions; and the right dock's `Layers` context.

## Known-open items (not owner-blocked, just not done yet)

- **`get_settlements()` carries no `tid`, so the Timeline's ghost/highlight/exist-only filters can't touch the map yet** (`TIMELINE_SCOPE.md` milestone 6, `civilization_workspace.gd`'s Timeline category). `NamedSettlement` gained a stable `tid: u64` field at the Rust level in milestone 1, and `civ_year_diff()` returns real tid-keyed present/removed/added sets, but `get_settlements()` (`lib.rs`) never puts `tid` into the `Dictionary` it hands to Godot, so nothing on the GDScript side can match a drawn pin (`map_overlay.gd`) to a diff entry. The three filter checkboxes read/display real `civ_year_diff()` counts but cannot filter, ghost or highlight individual settlement pins on the map until `get_settlements()` (and/or a per-year snapshot-settlements getter, since `civ_goto_year` deliberately never touches live `settlements`) exposes `tid`. A real, bounded Rust-side fix, not attempted in the GDScript-only milestone that found it.

- Real Fira Sans/Fira Code font files for the UI theme (design-system match found the pairing; sourcing + OFL-license verification deferred).

- **~~The phone UI is physically unusable by finger~~ — SUPERSEDED 2026-08-20.** The §13 phone layout has since shipped and **was verified running on the real OnePlus 6T** (`ANDROID_BUILD_SCOPE.md` §4.2). `project.godot` now sets `display/window/handheld/orientation="sensor"`, `DccShell._compute_layout_mode()` latches `_phone` true on this device (order-independent aspect `1080/2340 = 0.4615` < `_PHONE_ASPECT_MAX 0.6`, so landscape does not defeat it), and the shell builds **phone chrome, not desktop chrome**: app bar, floating domain rail (`WORLD`/`CIVIL`/`CARTO`), `⋯` overflow, bottom tool sheet, gesture inset, and in landscape the `_phone_side_safe` cutout pocket down the left edge. `_phone_scale = 1080/393 = 2.75` puts §13's 44 px minimum target at **~121 physical px**, clearing Android's 94 px (48 dp) floor. The verdict below described **the desktop shell running on a phone**, which is no longer what happens.
  - **The old "do not enable sensor rotation" warning is now wrong and has been removed.** It was correct while the phone layout did not exist; the layout is what it was protecting against, and `"sensor"` is now load-bearing in the opposite direction — a fixed landscape lock would make `DccShell._apply_phone_orientation()`'s landscape treatment unreachable.
  - **Still open, found 2026-08-20:** runtime-built dialogs (`Open project`, `New world`) keep desktop sizing inside the phone shell — ~1020×690 windows with 10-12 px body type — and `Open project` renders **two stacked headers with two close buttons** (the host `Window` title bar plus the dialog content's own branded header). Reported, not fixed; both are §13 layout work.
  - **Still unseen:** the *portrait* composition, which is §13's primary design. `adb` cannot force it — Godot's `"sensor"` sets `SCREEN_ORIENTATION_SENSOR`, which follows the accelerometer and overrides `settings put system user_rotation`. Physically rotating the phone is the only way.
  - The measurements below are retained as **historical spec input** for the desktop-shell-on-phone case, not as a current description:
  - **The failure is purely physical scale.** The panel is 403×410 dpi and Godot renders at native resolution with no content scaling. In its landscape configuration the display reports density 314 dpi, putting Android's 48 dp minimum touch target at **94 physical pixels**. Actual sizes: menu bar 34 px (2.15 mm, 36% of minimum), workspace tabs 30 px (32%), tool options bar 34 px (36%), **left tool rail 44 px wide with ~35 px pitch (2.78 mm / 2.2 mm, 47%)**, Layers rows 32 px (34%), status bar 26 px (28%), **menu/dropdown popup rows ~22 px (1.39 mm, 23%)**, **slider grabbers ~12 px (0.76 mm, 13%)**. Body text is 10-13 px against a 24 px (12 sp) minimum, i.e. 0.45-0.8 mm cap heights versus the ~1.5 mm a normal eye resolves at 40 cm.
  - **A fingertip contact patch is 110-160 physical pixels here.** One touch spans the menu bar plus the workspace tabs plus the tool options bar, or five consecutive dropdown rows, or three Layers checkboxes.
  - **Event routing is sound** — every tap, swipe and popup in the pass behaved correctly. The pass drove them with `adb shell input tap`, a zero-area synthetic pointer, so it proves the interaction model works and proves nothing about fingers. Verdict: drivable with a stylus or fingernail, effectively undrivable by fingertip, unreadable at arm's length in the dock/status bar/tool options bar.
  - **Correction the milestone will need**: its own "44-52 px targets" must be read as *density-independent* pixels (~86-102 physical px on this device), not raw Godot pixels. At raw pixels the new layout would be no better than the current one.
  - Worst regions in order: left tool rail, menu/dropdown popups, status bar. Best behaved: the dialogs (40 px buttons, internal scrolling).

- **The Android debug `.so` size — mostly resolved, with a residue.** The dedicated `[profile.android-dev]` (`debug = "line-tables-only"`) now exists, so the *mandatory* hand `llvm-strip` is gone: 400 MB → **156 MB**, and a 207 MB APK that installs and runs fine. It is still not the 18 MB a full `--strip-debug` gives, because line tables for a workspace this size are themselves large. Deliberately left there (2026-08-20): adding `strip = "debuginfo"` would recover the size but delete the file-and-line information the profile exists to preserve, leaving `debug = "line-tables-only"` as dead config. If size ever becomes the binding constraint, drop `debug` and set `strip = "debuginfo"` **together**, and say in the comment that backtraces lose file/line. **Also fixed the same day:** `cartalith.gdextension`'s `android.debug.arm64` pointed at `target/aarch64-linux-android/debug/`, which the `android-dev` profile never writes — the 2026-08-18 pass had been hand-copying the `.so` there, which is exactly how a stale library ships. The manifest now points at `android-dev/`. And `Cargo.toml`'s own usage line did not run (`--profile` must follow `build`, not precede it).

- **The New world dialog's default resolution (2048×1311, 2.68 M cells) costs ~875 MB peak on a real phone**, with no progress indication. Re-measured 2026-08-20: peak **899,089 KB (878 MB)** against 2026-08-18's 894,968 KB (874 MB) — **flat**, so everything landed between the two passes cost essentially nothing at the transient peak. Steady-state did grow, 500-538 MB → **662,793 KB (647 MB, +23%)**, which is the phone shell's second chrome tree plus resident new windows; not a leak (held across seven consecutive samples). Wall-clock improved to ~16-18 s from ~31 s. It completes and renders correctly and nothing kills it, but 878 MB is still a large fraction of a mid-range Android per-app budget. Worth revisiting before Android is treated as a supported target rather than a verified one.

## Settlement placement fix, click/zoom investigation, richer pin rendering (owner report, 2026-08-19)

Full detail in `CHANGELOG.md`'s matching entry. Summary:

- [x] **Placement misfire — real root cause found and fixed.** The
      reference has a second, later snap step (`_civSnapToWaterEdge`,
      v1.36/v1.39 — a bounded, tolerance-gated nudge onto the nearest real
      water edge) that milestone 8 deliberately never ported. Ported now as
      `place_settlements_with_water_edge_snap` (`cartalith-civ`), 8 new
      golden-parity unit tests extracted directly from the reference. The
      `coastal` flag now recomputes on final post-snap geometry (a
      disclosed, zero-cost improvement over the reference's own pre-snap
      ordering).
- [x] **"Not visible at all" — investigated live, confirmed FALSE.** A
      headless drive against a real 200-settlement world found every pin
      well-formed (non-zero radius, real on-screen position, valid faction
      colour). What was real: only 2.5% flagged coastal (pre-fix), and
      every settlement always drew at full size regardless of zoom (fixed
      below).
- [x] **Zoom-dependent settlement tiering ported** (`CIV_LOD_PLACE`,
      owner: "a zoom dependent second layer of settlements... quite nice to
      have in the html version"). New `SETTLEMENT_LOD`/
      `_settlement_below_lod()` in `map_overlay.gd`: capital/city always
      full-size, town/village/hamlet gate on raw camera zoom (0.4/0.7/1.4),
      a small faction-tinted dot below threshold rather than hiding
      outright — the same importance-tiered LOD reveal OpenStreetMap Carto
      and the Mapbox/MapLibre style spec both use (cited in the CHANGELOG
      entry). Hit-test radius stays in sync automatically
      (`_settlement_pin_radius` is the one shared source both `_draw()` and
      `_hit_test_settlement` read).
- [x] **Richer pin rendering** — soft drop-shadow (legibility against pale
      biome colours) + a real-data-grounded coastal "harbour" badge
      (`get_settlements()`'s own `coastal` field). No real per-slot
      settlement texture is exposed to GDScript yet (`PACK_SETTLEMENT_SLOTS`
      names the vocabulary; no `#[func]` returns the art), so this is
      enhanced vector, not asset-pack art — checked honestly, not assumed.
- [x] **Settlement-click "pop-up" — confirmed working as designed.** It is
      `_draw_hover_card()` (real name/kind/population data), showing
      because the mouse is already over the pin at click-time; the right
      dock is populated separately via `settlement_selected`. No stray
      dialog exists. No change made.
- [ ] **Still open: the `cartalith-godot` bridge call site.** `lib.rs`
      (~line 639) still calls the old `place_settlements` (kept, unchanged,
      for exactly this reason) — a concurrent Travel Library `#[func]`
      pass had that file (and `journey_bridge.rs`) dirty for this entire
      pass, so it could not be touched. One-line swap to
      `place_settlements_with_water_edge_snap` once that file is clean
      again, threading `flood`/`ws.flow_discharge`/`flow_thresh`/
      `map_width_km` (all already computed at that call site). Until then
      the live game runs the pre-fix placement path.

## Owner-only items

- None currently open. Criterion 4 (real Android device build/install/launch/golden-path) was fully closed 2026-08-17 once the owner unlocked the connected phone mid-session, and **re-verified end to end on 2026-08-18** against everything landed since — see `ANDROID_BUILD_SCOPE.md`.
- This session has real Windows desktop + `godot4` CLI access + real Android device access, which closes most of what earlier sessions couldn't do themselves.
