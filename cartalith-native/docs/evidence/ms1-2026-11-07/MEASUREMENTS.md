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
