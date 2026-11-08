# MS-1 measurements, 2026-11-07

The evidence file `MOBILE_SHELL_MS1_SPEC.md` §4.1 asks for. It holds the S0 measurement sheet and the bar
results of each stage built in this pass. Godot 4.7.2 on this Windows desktop (96 dpi, screen scale 1.0). Every
pixel figure comes from a windowed run. Status lives in `cartalith-native/docs/STATUS.md`, not here.

## S0, the measurement sheet

### Rows a desktop can answer: measured

All of these come from `_ms1s0_probe.gd` (`--resolution 1200x800`, windowed). It boots no shell and
builds reference scenes at a root `content_scale_factor` of 2.

| What | Result |
|---|---|
| Text at a root factor of 2, mode `DISABLED` | Visible rect 600 × 400 and final transform 2.000. Godot raised the root's oversampling to 2.000 by itself. On HD-01's edge test the 12 px label reads a max adjacent \|ΔLum\| of **0.925** (1.000 at factor 1). Setting `oversampling_override` = 2 changes nothing (0.925). |
| Text at a root factor of 2, mode `CANVAS_ITEMS` (aspect IGNORE) | **Not usable.** The visible rect is 576 × 324 and the final transform 2.083, because the mode scales against the project's base size (1152 × 648), not by the factor alone. That is the HD-04 effect `project.godot` already warns about. Edge 1.000, oversampling 2.469. |
| Whether an embedded `AcceptDialog`, a `PopupMenu` and a `ColorPickerButton`'s popup inherit a root factor of 2 | **All three inherit it.** Each stays embedded with its own `content_scale_factor` 1.0, and each draws at about twice its declared size in physical px: a 300 × 200 dialog draws as 654 × 476 including its frame, a 78 × 113 menu as 194 × 261, a 306 × 600 picker as 636 × 785. |
| Touch and mouse positions in `_gui_input` under a factor of 2 | A press pushed at physical (400, 300) reaches `_gui_input` at local **(200, 150)**, so the mapping is correct and needs no code. |
| `view3d.gd`'s `SubViewport` under a factor of 2 | A `SubViewportContainer` with `stretch = true` of 400 × 300 canvas units gets a **400 × 300** `SubViewport`. Its physical rect is 800 × 600, so the 3D image is rendered in dp and magnified (R4). S2 has to correct this. |
| Whether `size_2d_override` with stretch, in a probe host, draws what a root factor draws | **Identical.** 0 of 120 000 px differ (max \|ΔLum\| 0.000). Edge 0.984 in both. |

What this decides for S2 (spec §3.3): use the root `Window` in mode `DISABLED` with `content_scale_factor` =
density × UI scale, and in a probe use the host `SubViewport`'s `size_2d_override` with stretch. No
`oversampling_override` is needed, because the engine raises the root's oversampling by itself.
`DccWidgets.oversample()` remains for content-scaled sub-windows (HD-01).

### Rows that need the OnePlus 6T: pending the owner's reconnect

No device was connected (`adb devices` empty), and `adb` was not run. Every cell below reads *not measured*.

| What | Command | What the result changes |
|---|---|---|
| Android's density and size | `adb shell wm density`, `adb shell wm size` | The 6T's true dp. 420 dpi (2.625) leaves the phone bit-identical after S2 (`_phone_scale` = min(2.625, 1080/412 = 2.6214) = 2.6214). 402 dpi (2.5125) shrinks every phone figure by 4.1 %. 450 dpi (2.8125) changes nothing, because MQ-3's cap at short ÷ 412 holds it at 2.6214. |
| `screen_get_dpi()`, `screen_get_scale()` at default font size and at `adb shell settings put system font_scale 1.3` (restore with `adb shell settings put system font_scale 1.0`) | printed at boot (`Cartalith shell build` line onward, `adb logcat -s godot`) | Whether `screen_get_dpi() ÷ 160` equals `wm density` ÷ 160. If not, MQ-2 applies (DPI ÷ 160 accepted, Ruling DY) and the measured change goes into `DECISIONS.md`. Also whether `_os_text_scale()` is 1.0 at the default font size. |
| `screen_get_size()`, `window_get_size()`, `get_viewport_rect().size` | printed at boot | Whether `_host_px()` (`Window.size`) equals the viewport rect at factor 1 on the device, which is the premise of S1's identity there. |
| The platform density from GDScript (`AndroidRuntime` with `JavaClassWrapper`, unconfirmed) | printed at boot | If readable, `_density()` uses it instead of DPI ÷ 160. |
| The manifest's `resizeableActivity` and `configChanges` | `aapt2 dump xmltree <apk> --file AndroidManifest.xml` | Whether split screen or `wm density` resizes the app live (S3/S4 reachable on glass) or restarts it (R12). |
| A cold boot to the first frame | median (min..max) of 5, run alone | B5's timing comparison on glass. |
| B7: boot screenshots before and after S2, two of each, with `lastUpdateTime` | `adb exec-out screencap -p > a.png` ×2, `adb shell dumpsys package org.cartalith.walkingskeleton \| grep lastUpdateTime` (`export_presets.cfg` `package/unique_name`) | Pixel identity of the owner's phone. |
| B7: `adb shell wm size 1600x2560` with `adb shell wm density 320`, then `adb shell wm size reset` and `adb shell wm density reset` | as written | Whether 800 × 1 280 dp comes up a tablet (today's rule: aspect 0.625, so a tablet). |

### S1 against the drafts

S1 is drafts 01 and 02 (`cartalith-native/drafts/mobile-shell/`), transplanted. `check_symbols.sh` was run on `f5637c89` and reported nothing missing and nothing taken. The function bodies are the drafts', apart from two deviations, each made because the draft was wrong against its own spec or the live code:

1. **`classify()` without a current class, for a size of zero.** Here `width_class` is `""`, where draft 01 returns `"xl"`. A guessed class stands in for "not classified", and `MISTAKES.md` forbids encoding absence as a plausible value. Draft 01's own `width_class()` already treats `""` as not classified.
2. **S1's `_compute_layout_mode()` passes no `current`**, where draft 02 passes `_layout`. With `_layout` fed back, the hysteresis band acts from S1 onwards. A phone resized to an aspect between 0.600 and 0.610 would then stay a phone, where today's code makes it a tablet. That breaks the draft's own statement that S1 "gives the same `_phone` ... as today". The band arrives in S3, as the draft's S3 section has it.

Two things the drafts leave to the build are in place:

- `_TABLET_MIN_DP`'s worked table was corrected in the move;
- the comment on `phone_scale()` is marked false until S2.

## S1 results

- **B1.** `_ms1class_probe.gd --table`: PASS, 73 checks. Before S1 it fails, because `classify()` does not exist. All 12 mutants were killed (`mut_b1.py`: exact literal replace, restored in a `finally`, the file's hash checked afterwards): `TABLET_MIN_DP` 899 and 901, `PHONE_ASPECT_MAX` 0.59 and 0.61, `BAND_DP` 0, `BAND_ASPECT` 0, the width-class bounds 601, 841, 1201 and 1601, and the band direction flipped on each line.
- **B2 and B9.** `_ms1ident_probe.gd` takes a windowed capture with every text node transparent. The pristine tree was captured twice at 18 sizes, and both captures were identical at every size. After S1 all 18 are **byte-identical** to the baseline and report the same class:
  - desktop (pointer): 1920 × 1080 and 1366 × 768;
  - `--force-touch`: 360 × 800, 393 × 852, 412 × 915, 540 × 1170, 1080 × 2340, 1440 × 3120, 852 × 393 and 2340 × 1080 (phone), and 600 × 960, 800 × 1280, 1280 × 800, 2388 × 1668, 2560 × 1600, 3392 × 2400 and 2400 × 3392 (tablet);
  - 3840 × 2160, which is a phone today, by the aspect alone off a device.

  The positive control (`--control`, a 4 × 4 block) moves the hash.
- **B10, partly.** 14 probe runs gave the same RESULT line as their baseline, each with 0 `SCRIPT ERROR` lines:
  - `_alignment_probe` on desktop, tablet and phone;
  - `_phonemap_probe` and `_phoneui_probe`, each at 393 × 852, 412 × 915 and 852 × 393;
  - `_quickbar_phone_probe` at 393 × 852 and 852 × 393;
  - `_gsw_probe` (339 checks).

  `_quickbar_phone_probe` at 720 × 1600 and 1080 × 2340 exits 2 ("this probe is for the phone form") both before and after: a `--resolution` taller than this monitor is clamped. The other 93 `--vp` scripts were not re-run.

`DccShell._phone_scale` reads 2.6214 on a 1920 × 1080 desktop and 1.8641 at 1366 × 768. That confirms spec §2.3: `phone_scale()` is published as short ÷ 412 off the phone.

## S2 results (the dp unit)

Built from draft 02's `_phone_scale_for()`, `_apply_unit()` and `_unit_factor()`, with one deviation. At a factor of 1, `_apply_unit()` clears a probe host's `size_2d_override` to (0, 0), the host's default, rather than setting it to the full size. That makes a legacy run the pre-MS-1 viewport by construction. Three sites (spec §2.6) follow:

- `ViewportHost._update_lod()` multiplies by the viewport's final transform;
- `DccApp._capture_preview_png()` maps its crop through that transform;
- `view3d.gd`'s `_fit_physical()` renders 3D at physical resolution.

- **B3, `_ms1unit_probe.gd`.** Sizes were read from the framebuffer by a modulate-to-black difference. Every leg below **fails on the S1 tree**: the menu bar draws 52 px and the undo chip 168 px.
  - **Tablet** (`--vp 2388x1668 --density 2`): menu bar 104 px, rail 96 px, status bar 72 px (each ±1). The 12 px label "CARTALITH" passes the edge test, normalised to its own luminance range (≥ 0.9). This deviates from spec B3's raw 0.9. That figure was HD-01's white-on-black reading, and this label's ink is not white: it reads 0.579 raw even at factor 1. The 3D `SubViewport` equals the view's physical rect. The preview image is the map's physical rect after the thumbnail rule. **PASS, 7 checks.**
  - **OnePlus Pad 3** (`--vp 3392x2400` and `2400x3392`, `--density 2` = 1 696 × 1 200 dp): the same checks, **PASS in both orientations**. `classify()` gives it the tablet form: aspect 0.71, so a tablet under today's rule and under MQ-1 (c). The width class is xl in landscape and large in portrait. Captures `pad3-*.png` here: the map is the larger pane in both orientations, and no dock overflows.
  - **Phone** (`--vp 1440x3120 --density 3`): `phone_scale` 3.0000. The 48-unit undo chip is 144 px. Every visible button is at least 132 px on both axes. **PASS, 3 checks.**
  - **Map: the bar as written does not hold, reported rather than tuned.** On a small generated world at 2388 × 1668, the tile pyramid first engages after 2 zoom steps at `--density 1` and at fit (step 0) at `--density 2`. That is 2 steps apart, against the bar's "within one step". The cause is layout, not the LOD arithmetic. At density 2 the window is 1 194 dp wide, so the two 400 dp docks leave a **346 × 634 dp** map instead of 1 540 × 1 468 px. The cover-fit zoom is therefore 2.44 instead of about 1.27, which is already past `LOD_AUTO_ZOOM` (2.2), a zoom gate in canvas units. At engagement the physical px per cell is 6.97 at density 1 and 3.30 at density 2, both over the 1.0 threshold. **For the owner:** at true dp, an 11-inch-class tablet in landscape keeps a 346 dp map. This is spec §2.2's arithmetic made real, and it waits on MS-7's floating docks.
- **B9.** Pointer builds at 1920 × 1080 and 1366 × 768 are byte-identical to the S0 baseline, and the root factor reads 1.000. An export PNG at density 2 was **not measured**.
- **B10, partly.** All 16 legacy touch captures (no `--density`) are byte-identical to the baseline. The 14 probe runs give the same results. `DccTheme.phone_scale()` now reads 1.0000 off the phone, where it read 2.6214 at 1920 × 1080.
- **B7: pending the 6T.** At its quoted densities: 420 dpi gives `_phone_scale` = 2.6214, unchanged; 402 dpi gives 2.5125, every phone figure −4.1 %; 450 dpi gives 2.6214, held by the cap. At `--density 2.625` the desktop composition reads `phone_scale` 2.6214.
- The regression runs above preceded one line added to `view3d.gd` afterwards: a deferred first `_fit_physical()`, without which the 3D `SubViewport` stayed in dp. That line was verified by the B3 re-run. The S3 run re-runs everything.

## S3 results (live re-evaluation within a class)

These are draft 02's S3 pieces: `_booted`, the 150 ms `_resize_timer`, `_on_window_resized()`, `_on_resize_settled()`, and `_request_class_swap()`, which only logs. One addition: when the class is unchanged, `_on_resize_settled()` publishes the new classification (`_layout`, `DccTheme.set_layout()`), as spec §3.4 says. The draft dropped it. The "decided once" prose on `_phone`, `_ready()`, `_on_window_resized()`, `_on_tablet_resized()` and `_process()` was rewritten. The comments in `DccTheme._elide_labels` and in `radial_ring.gd` stay true until S4 builds a swap.

- **B4, `_ms1class_probe.gd --live`.**
  - **Phone, `--vp 1080x2340 --density 2.625` rotated to 2340 × 1080.** The layout goes landscape and stays a phone. `_phone_root` keeps its instance id, so the composition was not rebuilt. The published width class is expanded, 891 dp. PASS, 4 checks.
  - **Tablet, `--vp 2388x1668 --density 2`.** The left dock was set to 320 dp through `_left_width` and `_apply_dock_widths()`, which is what WI-04's drag writes, not a finger. It reads 320.00 dp both before and after a resize from 2388 to 2200 px. The unit follows the resize: the override goes from (1194, 834) to (1100, 834). PASS, 4 checks.
  - **Before S3:** the phone leg passes, because the old handler already rotated the phone, so that leg guards rather than pins. The tablet leg **fails**: the override stays at (1194, 834).
- **The class question (spec §3.4), `--live swap`, at 900 × 1 600 dp.**
  - At 890 × 1 582 the window stays a tablet and nothing is requested (inside the band).
  - At 880 × 1 564 the shell logs exactly one `size class change requested (phone=true, (880.0, 1564.0) dp); kept the boot class until MS-1 S4` and keeps the tablet composition.
  - Before S3 the leg's checks pass, but no request line is logged.
- **B10, partly, and B9.** All 18 identity captures are byte-identical to the S0 baseline. The 14 probe runs are unchanged. B3's six legs still pass.

## S5 results (UI scale)

There is no draft for S5; it follows spec §3.6. `DccSettings.ui_scale()` and `set_ui_scale()` are stored in a `[ui_scale]` section and accept only 1.0, 1.25, 1.5 and 2.0. `Preferences ▸ UI scale ▸ 100 % · 125 % · 150 % · 200 %` is a radio submenu built like Handedness, with ids 930 to 933. It exists on touch builds only: the PC's Preferences is unchanged (MQ-4), and the phone shows a disabled row with its reason (MQ-3 (a)). The shell reads the setting in `_ui_scale()` on touch builds, and `--ui-scale` overrides it. A choice calls `DccShell.ui_scale_changed()`, which takes the same path as a settled resize.

- **B6.**
  - **`--vp 2388x1668 --density 2`, then 150 % pressed** (`id_pressed` on `UiScaleChoice`, the handler path, not a finger). The settings file reads 1.5 and the menu bar draws **156 px**. Restoring 100 % stores 1.0 and the bar draws 104 px again. PASS, 7 checks. On the S3 tree the leg **fails**: there is no submenu.
  - **`--ui-scale 1.5` on the same tablet.** The menu bar is 156 px, but **the layout does not fit**. That size is 796 × 556 dp, so the rail plus two 400 dp docks leave the map 0 px wide and the status bar draws 57 px against 108. The 3D and preview checks fail with it. **Reported, not tuned:** a UI scale above 100 % on an 11-inch-class tablet waits on MS-7's floating docks, just as density 2 alone leaves a 346 dp map (S2 results above).
  - **At UI scale 125 %, 4K at density 2 is 1 536 × 864 dp.** A cold boot (`--ui-scale 1.25`) gets the **phone** composition with `phone_scale` 2.0, which is MQ-1's current rule. Pressing 125 % on a running 4K tablet logs `size class change requested (phone=true, (1536.0, 864.0) dp)` and keeps the tablet. B6's "that swap meets B5" waits on S4.
- **B9 and B10, partly.** All 18 identity captures are byte-identical to the S0 baseline, and the 14 probe runs are unchanged. B3 and B4 still pass.

## Ruling EG, 2026-11-08: the 900 dp line at every aspect

`DccTheme.classify()` now reads `phone = touch and (short_dp < 900 if density_known else aspect < 0.6)`. With a
known density (a device, or `--density`) the 900 dp shortest-width line alone decides; the aspect line applies
only to the legacy figure, so every run without `--density` keeps its pre-EG class. The phone's unit was already
S2's `min(density x ui_scale, short / 412)` (EG item 2), so nothing else changed. Measured on `996cb307` plus the
change, against a pristine worktree of `996cb307`, both windowed, each user dir wiped.

- **B1.** `_ms1class_probe.gd --table`: PASS, 81 checks. The changed rows are the foldable, `TABLET 800`, the
  889 dp panel and the 11-inch class (both orientations, and the 11-inch at 125 %), now phones; new rows pin the
  line at aspect 0.9 (899 / 900 / 901 dp, both sides) and a 16:10 tablet of 1 600 x 1 000 dp shrunk past the band
  (1 430 x 894 stays a tablet, 1 400 x 875 becomes a phone). On the pre-EG tree **13 of 81 fail**. Mutation (Python,
  exact literal replace, restored in a `finally`, hash checked): 8 of 8 killed, `TABLET_MIN_DP` 899 and 901,
  `PHONE_ASPECT_MAX` 0.59 and 0.61, `BAND_DP` 0, `BAND_ASPECT` 0, the aspect gate put back on the dp line, and
  the legacy branch given the dp line.
- **B2, B9 and B10's class.** All 18 `_ms1ident_probe.gd` captures (the same sizes as S1, no `--density`) are
  byte-identical to the pristine tree and print the same class: legacy runs are untouched by construction.
- **The 11-inch window under EG** (`_ms1unit_probe.gd --vp 2388x1668 --density 2 --leg phone`): the phone
  composition, `phone_scale` **2.0000** (the density; short / 412 would be 4.05), the undo chip 96 px, every
  visible button at least 88 px on both axes (30 walked). On the pre-EG tree: FAIL, 3 of 3 (a tablet).
- **Re-measured at sizes that stay tablets** (the old legs ran at 2 388 x 1 668 @2, now a phone): B3 on the
  OnePlus Pad 3 (3 392 x 2 400 and 2 400 x 3 392 @2) PASS, 7 checks each; B4's tablet leg resized 3 392 -> 3 100
  px @2: the 320 dp dock reads 320.00 before and after, the override follows to (1550, 1200), width class large;
  B6 at 4 000 x 2 800 @2: menu bar 104 px, 156 px at 150 % (1 333 x 933 dp, still a tablet), 104 px restored;
  at 4K @2, 125 % still logs the class-change request at 1 536 x 864 dp.
- **Regression, base against EG:** the other MS-1 legs (phone rotation, the swap request, the 1 440 x 3 120 @3
  phone, `_ms1s0_probe`) and the PC and VE probes at desktop, tablet and phone (`_pc1select`, `_pc4context`,
  `_pc6ask`, `_pc7windows`, `_pc8planner`, `_pc9menus`, `_ve5`: 19 runs) give the same result line on both trees,
  each with 0 `SCRIPT ERROR` lines. None of them passes `--density`, so EG cannot reach them.
- No Rust changed; `cargo test` was not run.

## S4: not built; the S4.0 census

S4 (spec §3.5) needs the full S4.0 inventory first: an action for each item (rebuilt, survives, reset, freed, or re-presented), and per window, the state its re-present path must carry (MQ-5, Ruling DY). That inventory was **not written** in this pass. This is the census it starts from, measured 2026-11-07 on the S5 tree:

| Count | Figure | Method |
|---|---|---|
| node-typed fields in `dcc_shell.gd` | 73 | `grep -cE "^var [A-Za-z_]+ *: *(Control\|Button\|Label\|…)"` over 27 Node classes. The spec's 78 used a different class list, so the two are not comparable. |
| `Dictionary` fields in `dcc_shell.gd` | 17 | the spec counted 16 |
| `node_added`, `size_changed` and `phone_insets_changed` connections | 15 in 10 files: `dcc_shell.gd` 4, `app.gd` 2, `dcc_widgets.gd` 2, and one each in `asset_library_window.gd`, `new_world_dialog.gd`, `open_project_dialog.gd`, `quick_bar_menu_pin.gd`, `quick_bar_window.gd`, `vault_window.gd` and `view3d.gd` (new in S2) | spec §6's grep |
| `phone_present(` call lines | 41 in 30 files | spec §6's `cnt` |
| files calling `phone_window(` | 31 | `grep -rln` |
| static node registries in `DccTheme` | `_portrait_watchers` (Callables) and `_elide_labels` (Labels) | — |
| prose that relies on the class never changing | `DccTheme._elide_labels` and `radial_ring.gd`'s `_apply_touch_scale()`, both rewritten in S5 to say what S4 must do; `dcc_shell.gd::_process()`'s self-disable, rewritten in S3 to say the swap must re-enable it | spec §6's grep, run again |

## Bars: measured and pending

| Bar | State |
|---|---|
| B1 | **met** (S1); re-pinned for Ruling EG (2026-11-08), 81 checks, 8 of 8 mutants killed |
| B2 | **met at 18 sizes** (S1) |
| B3 | **met for tablet, Pad 3 and phone.** The map leg **does not hold as written**: 2 steps apart, a layout effect that waits on MS-7. |
| B4 | **met** (S3) |
| B5 | **pending S4** (not built) |
| B6 | **met through the handler.** The 4K swap waits on S4. |
| B7 | **pending the 6T** |
| B8 | parse checks after every edit, all clean. No Rust changed, so `cargo test` was not run (brief). |
| B9 | **met** for 1920 × 1080 and 1366 × 768 at every stage, with a root factor of 1.000. The export PNG half was **not measured**. |
| B10 | **14 of 107** scripts re-run, plus 16 legacy touch captures. The other 93 were not re-run. |
