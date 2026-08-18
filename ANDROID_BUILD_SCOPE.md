# Real Android device pass: what was required, how far it got

Prompted by the owner connecting a real device this session (OnePlus 6T,
Android 14, USB debugging authorized). `docs/STATUS.md`'s MVP criterion 4
("Android `.apk` builds + owner has installed/run") had this recorded:
*"Install+run on real hardware is not reachable from this environment —
investigated via emulator, root-caused as a SwiftShader/emulator
limitation, not our code... softened by the 2026-08-16 `/goal`, no longer
a hard requirement."* With a real device now connected, that constraint no
longer applies — this pass re-attempted it for real, following this
project's own `cartalith-porting-discipline` rule to flag what can't be
verified rather than claim false success.

## What was actually required (none of it was missing)

`TOOLCHAIN.md`'s own framing called Android "the single highest-risk item
in the toolchain" and told Phase 0 to prove it before investing further.
Checked every piece fresh rather than trusting that framing as still true:

- `rustup target list --installed` — `aarch64-linux-android` already
  installed.
- `cargo-ndk` — already installed (`4.1.2`).
- NDK — already installed at
  `AppData\Local\Android\Sdk\ndk\29.0.14206865`, `ANDROID_NDK_HOME` set.
- `godot4` on `PATH` (a WinGet-installed shim resolving to the real
  `Godot_v4.7.1-stable_win64.exe` on the Desktop) — confirmed via
  PowerShell, **not visible from Git Bash's `PATH`**, a real environment
  quirk worth recording: this pass's `godot4 --headless` invocations all
  had to go through the `PowerShell` tool, not `Bash`.
- `cartalith.gdextension` already had correct `android.debug.arm64` /
  `android.release.arm64` library paths pointing at
  `target/aarch64-linux-android/{debug,release}/libcartalith_godot.so`.
- `export_presets.cfg` already had a correctly configured `"Android"`
  preset (`arch/arm64=true`, `package/signed=true`, unique name
  `org.cartalith.walkingskeleton`).
- A previously-built `builds/android/Cartalith.apk` already existed
  (dated 2026-08-15 21:32) — but stale relative to everything landed
  2026-08-16 (Phase 2 milestones through 15, GPU integration, the memory
  fix, CPU multithreading), and built from a **debug**-only `.so` with no
  release counterpart.

**Conclusion: the Phase 0 risk `TOOLCHAIN.md` flagged never materialized.**
Every piece of the Android toolchain gdext/Godot/cargo-ndk needed was
already correctly installed and wired from earlier in this project's
history. This pass's real work was producing a *current* build and testing
it on real hardware, not toolchain setup.

## Build (done)

1. `cargo ndk -t arm64-v8a build --release -p cartalith-godot` —
   compiled clean, 2m38s, produced a current
   `target/aarch64-linux-android/release/libcartalith_godot.so` (14.5MB).
2. `godot4 --headless --export-release "Android" builds/android/Cartalith.apk`
   — **failed**: `Code Signing: Could not find release keystore, unable to
   export.` Expected, not a bug — `TOOLCHAIN.md`'s own Android section
   already says *"No keystore yet. Debug signing is enough to sideload"*;
   no release keystore has ever been created for this project, and
   creating one wasn't in scope for a sideload test.
3. Rebuilt the **debug** `.so` (`cargo ndk -t arm64-v8a build -p
   cartalith-godot`, no `--release`) so the debug library was equally
   current, then `godot4 --headless --export-debug "Android"
   builds/android/Cartalith.apk` — succeeded, signed with Godot's own
   auto-generated debug keystore.

## Install and launch (done)

`adb install -r` succeeded first try. `adb shell am force-stop` then
`monkey -p org.cartalith.walkingskeleton -c android.intent.category.LAUNCHER
1` launched it.

**Real logcat confirms a genuine successful engine start on real
hardware** — this is the actual headline finding, not a guess:

```
Godot Engine v4.7.1.stable.official.a13da4feb
renderingDevice: opengl3 (ProjectSettings)
renderer: gl_compatibility (ProjectSettings)
OpenGL API OpenGL ES 3.2 V@0502.0 ... Using Device: Qualcomm - Adreno (TM) 630
```

The GDExtension loaded (`libcartalith_godot.so` via `nativeloader`), the
Godot native layer initialized (`Godot native layer initialization
completed: true`), and a real OpenGL ES 3.2 context was created against
the device's actual Adreno 630 GPU. No crash, no ANR in this window, no
`gdext`-related error anywhere in the process's logcat.

`adb shell dumpsys meminfo org.cartalith.walkingskeleton` immediately
after launch (process alive, activity recorded `visible=true`):

| Metric | Value |
|---|---|
| PSS Total | 151,982 KB (~148 MB) |
| Private Dirty | 78,244 KB (~76 MB) |
| Native Heap (Private Dirty) | 51,392 KB |
| RSS Total | 261,932 KB (~256 MB) |

This is **launch/idle** memory only — see the blocker below for why
generation-time numbers weren't reachable this pass.

## Golden path, driven for real once the owner unlocked the phone (2026-08-17)

The owner unlocked the device mid-session. Re-checked immediately:
`adb devices` still showed it, and a fresh `adb exec-out screencap`
came back a real 1.26MB image (vs. the earlier blanked 15KB ones) —
`dumpsys window` confirmed `isKeyguardShowing=false`.

The app (still the same process from the earlier launch — it had stayed
alive backgrounded this whole time) was foregrounded via `adb shell am
start`, and the screenshot showed it was **already displaying a fully
rendered world** — real biome/hillshade terrain, rivers, settlements
(faction-coloured markers sized by tier), and the real road network, at
the UI's own default parameters (512×512, seed 12345, 800km, Classic, 40
settlements). This confirms the on-device renderer itself works
correctly — this wasn't left generating blank/broken while backgrounded.

To get a real, freshly-triggered generation with memory sampled through
it (not just a static already-rendered result), tapped the **Generate**
button (`adb shell input tap`, coordinates mapped from the screenshot)
and sampled `adb shell dumpsys meminfo` every ~1s through the run:

| Sample | t (approx) | PSS Total | Native Heap (private dirty) |
|---|---|---|---|
| 1 (right after tap) | +0s | 251,519 KB | 89,520 KB |
| 2 | +1s | 257,166 KB | 96,872 KB |
| 3 | +2s | 269,070 KB | 108,796 KB |
| 6 | +5s | 270,822 KB | 110,548 KB |
| 7 | +6s | 276,018 KB | 115,744 KB |
| **8 (peak)** | **+7s** | **283,326 KB** | **123,052 KB** |
| steady-state (settled, 4 consecutive samples) | +9-12s | ~271,290 KB | — |

**Peak PSS during generation: ~283,326 KB (~277 MB).** Steady-state
after completion settled to ~271,290 KB (~265 MB) and held flat across
four consecutive samples — no runaway growth. For comparison, this
project's own Windows measurement at 2048×2048 found a ~1,434-1,502 MB
peak (`MEMORY_OPTIMIZATION_SCOPE.md`) — this Android run was at the much
smaller default 512×512, so the two aren't directly comparable
size-for-size, but the *shape* (a real transient peak above a lower
steady-state, no leak) matches.

A second screenshot taken right as memory plateaued showed the
**identical map** (same seed, same terrain/settlements/roads, pixel-for-
pixel as far as visual inspection can tell) — exactly the deterministic
behavior a same-seed regeneration should produce, real confirmation the
full pipeline (terrain → climate → erosion → hydrology → Phase 2 civ →
render) ran to completion on-device and re-rendered correctly, not just
redrew stale state.

**No ANR, no crash, no hang.** `adb logcat` for the full window around
the tap showed no `ANR`/`FATAL`/`crash`/`Not responding` lines from this
app (only unrelated system noise — WiFi/location-permission chatter from
other processes). Generation completed in roughly 7-9 seconds wall-clock
at this size on the OnePlus 6T's mobile CPU — slower than this session's
own desktop timing-bench numbers for 512×512 (sub-second on the
16-thread Windows machine per `CPU_MULTITHREADING_SCOPE.md`), which is
expected: a phone SoC has far fewer, far slower cores, and this is the
full pipeline (including the not-yet-multithreaded Phase 2 civ layer),
not just the Rayon-parallelized terrain stage that benchmark measured.

**Golden path confirmed, real device, real numbers.** This closes out
the remaining half of MVP criterion 4.

## The real blocker (resolved above): a genuinely secured lock screen, not a code problem

A screenshot taken ~5s after launch (`adb exec-out screencap`) came back
solid black. Investigated rather than assumed a render failure:

- `dumpsys power` / `dumpsys deviceidle`: `mScreenOn=false`,
  `mScreenLocked=true` at screenshot time.
- Godot's own logcat shows why: `OnPause` then `OnStop` fired ~140ms
  after `OnResume` — the activity was backgrounded by the OS almost
  immediately, then `BufferQueueProducer: ... BufferQueue has been
  abandoned` / `eglSwapBuffers failed: EGL_BAD_SURFACE` — the rendering
  surface was torn down mid-init because the screen locked out from under
  it.
- Woke the screen (`input keyevent KEYCODE_WAKEUP` — confirmed
  `mWakefulness=Awake`, `mScreenOn=true`) and attempted to dismiss the
  keyguard (`wm dismiss-keyguard`, plus a manual swipe gesture). Keyguard
  stayed up (`isKeyguardShowing=true` before and after both attempts).
- `adb shell locksettings get-disabled` returned `false` — **confirmed
  this device has a real, enabled lock credential** (PIN/pattern/
  biometric), not a bare swipe lock. `wm dismiss-keyguard` only works
  against "None"/"Swipe" security; it silently no-ops against a real
  credential, which is exactly what was observed. A follow-up screenshot
  attempt returned byte-identical output to the first — Android
  intentionally blanks `screencap` output while a secure keyguard is
  active, a real OS security behavior, not a tooling bug.

**This is a physical-access requirement, not a toolchain or code gap.**
Nothing in `adb`'s non-root capabilities can dismiss a real lock
credential, and guessing or brute-forcing one was never appropriate to
attempt. Confirming the actual golden path — tapping Generate, watching
it render, capturing on-device memory *during* generation, checking for
an ANR under Android's stricter watchdog — needs the phone physically
unlocked (by the owner) while a session drives it, or the owner running
the already-installed APK by hand.

## Done means (fully reached)

| Item | Status |
|---|---|
| Android toolchain (NDK/cargo-ndk/gdext/Godot export) actually works | **Confirmed**, first real end-to-end proof this project has had |
| Current `.apk` built from today's code, not a stale one | **Done** — installed build reflects all of 2026-08-16's landed work |
| Installs on real hardware | **Done** |
| Launches, GDExtension loads, engine initializes, real GPU context created | **Confirmed via logcat** |
| Golden path exercised (tap Generate, confirm render) | **Done (2026-08-17)** — same-seed regeneration reproduced the identical rendered world |
| On-device memory during generation | **Done** — peak ~283,326 KB PSS (~277 MB) at 512×512, steady-state ~271,290 KB, no leak observed |
| ANR/responsiveness check under load | **Done** — no ANR/crash/hang, ~7-9s wall-clock at 512×512 |

MVP criterion 4 ("Android `.apk` builds + owner has installed/run") is
now **fully closed**, real hardware, real numbers, both halves (build+
install and actually running the golden path).

---

# Second real-device pass (2026-08-18): everything landed since, re-verified

Same phone (OnePlus 6T `ONEPLUS_A6013`, `9608b26b`, Android 14, USB debugging
authorized), same method as the 2026-08-17 pass above so the numbers compare
directly. Reason for the pass: the first one verified a build from
2026-08-16's code. Since then the GUI was replaced twice (panel-browser shell,
then the DCC editor shell, plus a declutter pass), 57 generation controls plus
a File ▸ New world dialog were added, `gw`/`gh` became independent (non-square
maps), four crates were added (`cartalith-spatial`, `-assets`, `-urban`, plus
tool-system code), and terrain appearance milestones 2-5 added real per-pixel
CPU work to `render.rs`. **None of it had been on hardware.**

This was a verification pass, not feature work. Nothing was fixed because
nothing crashed.

## 1. Build and install — works, with one new required step

- `cargo ndk -t arm64-v8a build -p cartalith-godot` — **clean, exit 0.** The
  grown workspace compiles for `aarch64-linux-android` with no new breakage;
  `cartalith-assets`, `-gpu`, `-engine`, `-civ` and `-godot` all rebuilt this
  run, the rest came from cache. 35 s incremental.
- **New finding: the debug `.so` is now 400,480,048 bytes (400 MB).** That is
  debuginfo, not code — `[profile.dev]` sets `opt-level = 1` but leaves
  `debug = true`, and the workspace is now large enough for that to matter.
  Godot's Android exporter stores `.so` files uncompressed, so this produces
  an APK that is slow to build, slow to `adb install`, and pointlessly large.
  Stripped with the NDK's own
  `llvm-strip --strip-debug target/aarch64-linux-android/debug/libcartalith_godot.so`
  → **18,372,760 bytes (18 MB)**, a 22x reduction with zero behaviour change.
  This step did not exist in the 2026-08-17 pass and is now effectively
  required. It is *not* a code fix and nothing was committed for it; if it
  becomes annoying, the real fix is `debug = "line-tables-only"` (or
  `strip = "debuginfo"`) on a dedicated Android profile.
- `godot4 --headless --export-debug "Android" builds/android/Cartalith.apk` —
  **succeeded**, 68,328,426 bytes, signed with Godot's auto-generated debug
  keystore. Still no release keystore (unchanged from the last pass, still not
  in scope). The export log confirms the **real app icons now ship**:
  `icon.webp` / `icon_background.webp` / `icon_foreground.webp` /
  `icon_monochrome.webp` across every mipmap density plus the adaptive
  `themed_icon.xml`.
- `adb install -r` — **Success**, first try.

## 2. Launch and golden path — both work on device

`adb logcat` on a cleared buffer, real hardware:

```
Godot Engine v4.7.1.stable.official
renderer: gl_compatibility · OpenGL ES 3.2 · Adreno (TM) 630
nativeloader: ... lib/arm64 ... libcartalith_godot.so
```

No `FATAL`, no `ANR`, no `lowmemorykiller`, nothing in the `crash` buffer for
this package across the whole session and all six generations.

The full golden path was driven by touch (`adb shell input tap` / `swipe`),
never by keyboard or mouse:

File ▸ New world → the setup dialog → resolution + aspect dropdowns →
**Generate** → map renders → OK → Layers panel toggles (Territory faction
fill, Province boundaries — both drew correctly) → tap a settlement →
Properties populated (`Arcjunjunlucforum (Capital)`, population 19661,
faction 1) **including the WHY HERE causal-chain explainer** (`strong fresh
water (0.96) → strong gentle terrain (0.98) → strong terrain form (0.90)`,
`Despite: weak flood risk (0.06)`, `Suitability 0.76`) → tool rail (tapping
the terrain tool switched the tool options bar and status bar to `RAISE /
LOWER` and lit the rail icon amber) → View ▸ Performance readout → Generate ▸
Climate, whose sliders **drag correctly by touch** (Gravity 1.00g → 1.80g on
a swipe; reset afterwards via `Reset this stage`).

Performance readout, on device: 60 FPS, `Adreno (TM) 630 (Qualcomm)`, 8 CPU
threads, static memory 52.26 MiB, video memory 42.73 MiB, and **"0 of 6
eligible stages ran on GPU — the whole pipeline ran on CPU, as configured"**
— the GPU-compute path is correctly inert on Android, as
`GPU_LAYER_INTEGRATION_SCOPE.md`'s current milestone intends.

Generation runs on a background `Thread` (`main.gd`), so the UI held 60 FPS
through every run including the 31-second one. No ANR anywhere.

## 3. Memory — measured, and it has grown materially

`adb shell dumpsys meminfo org.cartalith.walkingskeleton`, `TOTAL PSS`,
sampled continuously (~0.17 s/sample) across each generation — the same
metric and method as the 2026-08-17 pass.

| Run | Grid | Cells | Peak PSS | Steady PSS | Wall-clock |
|---|---|---|---|---|---|
| **2026-08-17 baseline** | 512x512 | 262 k | **283,326 KB (277 MB)** | 271,290 KB | ~7-9 s |
| this pass | 512x512 | 262 k | **395,756 KB (387 MB)** | 316,200 KB (309 MB) | ~4.5 s |
| this pass | 512x256, Whole world 2:1 | 131 k | 362,137 KB (354 MB) | 307,200 KB | ~3.2 s |
| this pass | 512x910, 9:16 tall portrait | 466 k | >=477,340 KB (466 MB) [1] | 333,950 KB | ~8-9 s |
| this pass | **2048x1311 (the app's own default)** | 2.68 M | **894,968 KB (874 MB)** | 538,300 → 500,040 KB | **~31 s** |
| this pass | 512x512 repeat, after the 2048² world | 262 k | — | **309,200 KB** | ~4.5 s |

[1] the portrait peak fell in a gap between two sampling loops; the true peak
is that figure or a little above it. Reported as a floor, not a point
estimate.

**Like-for-like at 512x512, peak PSS grew ~40% (283 → 396 MB) and steady-state
~17% (271 → 316 MB).** That is the honest on-device cost of everything landed
since: four new crates in the binary, terrain appearance milestones 2-5's
per-pixel work in `render.rs`, and the DCC shell's own node tree. It is real
and it is not a leak.

**No leak.** The last row is the proof: after the 2.68 M-cell world left the
process sitting at ~500 MB, regenerating at 512x512 returned steady-state to
309,200 KB — *marginally below* the first 512x512 run in the same session. The
big world's memory is fully released when a smaller one replaces it.

**The app's own default resolution is the interesting number.** File ▸ New
world opens at 2048x1311 (1.5625:1 reference region frame, 2.68 M cells), and
on this phone that costs **874 MB peak and 31 seconds**. It completed, it
rendered correctly, and nothing killed it — but 874 MB is a large fraction of
a mid-range Android device's per-app budget, and 31 s of silent work with no
progress indication is a poor first experience. Worth knowing before anyone
treats Android as a supported target rather than a verified one.

## 4. Timing

512x512 came out **faster** than the 2026-08-17 baseline (~4.5 s vs ~7-9 s),
sampled the same way from the same debug (`opt-level = 1`) profile. Read that
as "not slower", not as a claimed speedup: both figures are inferred from the
shape of the memory trace rather than an instrumented timer, and the
CPU-multithreading work sits between the two passes. Timing scales roughly
with cell count: 131 k → 3.2 s, 262 k → 4.5 s, 466 k → 8-9 s, 2.68 M → 31 s.

## 5. Non-square maps on device — all four shapes work

Every shape generated, rendered, and reported itself correctly:

| Shape | Result |
|---|---|
| 512x512, 1:1 square, Region | correct; 800 x 800 km |
| 512x256, 2:1 equirectangular, **Whole world** | correct; 800 x 400 km. Whole world correctly **pins** the aspect to 2:1 and **disables** the Aspect control with its own explanatory note |
| 512x910, 9:16 tall portrait, Region | correct; 800 x 1422 km |
| 2048x1311, 1.5625:1, Region | correct; 800 x 512 km |

The viewport aspect-fits each one (square and portrait fit to height with the
plate border intact, 2:1 fills the width), and the header and status bar
report the right cells and kilometres in every case. **The aspect work holds
up on device.** No bug found here.

## 6. The phone UI — structurally intact, physically unusable by finger

This is the honest-negative half of the pass, and it is more nuanced than
"the desktop layout is cramped".

### What does not break

**The app is orientation-locked to landscape**, so the phone hands the DCC
shell a **2340x1080** surface — *wider* than the 1920x1080 it was designed and
verified at, and exactly the same height. Consequently:

- All six regions are present and correctly proportioned. Nothing reflows,
  nothing is clipped, no region collapses.
- The right dock keeps its full 296 px and stays legible in structure.
- **Every runtime-built dialog fits inside the 1080 height and scrolls
  internally.** The New world setup dialog, `Generate — Climate` (the longest
  one, 12+ sliders across three sections), `Generate — Settlements`, and the
  Performance readout were all opened on device and all fit. **The 1080p
  dialog overflow a sibling fork reported is NOT reproduced here.**
- The viewport is ~1700x990 px of map at 403 dpi, and the Phase 3 atlas look
  (paper ground, plate border, hillshade, hydrology tint, geology) is
  genuinely beautiful at that pixel density. The map is the one part of this
  app the phone flatters.

**This depends entirely on Godot's default.** `project.godot` has no
`[display]` section, so `display/window/handheld/orientation` takes its
landscape default. **Do not unlock orientation or set it to portrait before
the responsive milestone ships** — a 1080x2340 portrait surface would give the
296 px dock plus the 44 px rail 31% of the width and stack 154 px of
horizontal chrome, which is precisely the case `UI_SHELL_DESIGN.md`'s deferred
393x852 phone layout exists to solve.

### What does break: absolute pixel sizes against a 403 dpi panel

The shell sizes everything in absolute pixels. The panel is 403 x 410 dpi —
about 2.5x a desktop monitor. Godot renders at native resolution with no
content scaling, so every control is ~2.5x physically smaller than the design
intends. In its landscape configuration the display reports density 314 dpi
(scale 1.9625), so **Android's 48 dp minimum touch target is 94 physical
pixels here.**

| Element | Size in `main.tscn` | Physical | vs the 94 px minimum |
|---|---|---|---|
| Menu bar | 34 px, font 13 | 2.15 mm | 36% |
| Workspace tabs | 30 px, font 11 | 1.90 mm | 32% |
| Tool options bar | 34 px, font 11 | 2.15 mm | 36% |
| **Left tool rail** | **44 px wide**, ~16 px glyphs, ~35 px pitch | **2.78 mm wide, 2.2 mm pitch** | **47%** |
| Layers rows | 32 px | 2.02 mm | 34% |
| Status bar | 26 px, font 10 | 1.64 mm | 28% |
| Menu / dropdown popup rows | ~22 px pitch | 1.39 mm | 23% |
| Slider grabber | ~12 px | **0.76 mm** | 13% |
| Dock body text | 10-12 px | 0.63-0.76 mm em | ~half the 12 sp (24 px) minimum |

A fingertip contact patch is 7-10 mm — **110-160 physical pixels**. One touch
therefore covers, simultaneously: the menu bar *and* the workspace tab row
*and* the tool options bar; or five consecutive dropdown rows; or three
Layers checkboxes; or a whole slider row plus its neighbours.

### The verdict, stated precisely

**Every interaction in this pass succeeded — and that is not evidence a person
can perform them.** `adb shell input tap` injects a zero-area point at an
exact pixel computed from a screenshot. It is a synthetic pointer, not a
finger. What the pass actually proves is that the *event routing* is sound on
Android: taps hit the right controls, swipes drive sliders, popups open and
dismiss, focus behaves. The interaction model works; the target geometry does
not.

- **Usable with a stylus or a fingernail**, with care and squinting.
- **Effectively undrivable by a fingertip.** Nothing in the chrome meets even
  a quarter of the platform minimum except the amber Generate button and the
  40 px dialog buttons.
- **Below the threshold of readability at arm's length** for the right dock,
  status bar and tool options bar: 0.45-0.8 mm cap heights against the ~1.5 mm
  a normal eye resolves at 40 cm.

Worst regions, in order: **the left tool rail** (2.78 mm column, 2.2 mm pitch
— the single smallest interactive region), **menu and dropdown popups**
(1.39 mm rows, the hardest thing in the app to hit correctly), and **the
status bar** (1.64 mm, 10 px type — decorative at this density; its tool hint
and world descriptor cannot be read). Best behaved: the dialogs, whose 40 px
buttons and internal scrolling are the only part of the chrome that survives
contact with a phone.

### Recorded as an open item, deliberately not fixed

`DCC_SHELL_SCOPE.md` and `UI_SHELL_DESIGN.md` both scope a real 393x852 phone
layout — bottom tool bar, bottom-sheet tool options, full-height panel sheets,
44-52 px targets — and both explicitly defer it. Building any of it as a side
effect of a verification pass would leave exactly the half-migrated state this
project has avoided throughout, so **nothing was changed.** The measurements
above are the specification input for whoever picks that milestone up: the
gap is uniformly 2-4x on touch targets and ~2x on type, and the deferred
design's own 44-52 px figures must be read as *density-independent* pixels
(~86-102 physical px here), not as raw Godot pixels — at raw pixels the new
layout would be no better than the current one.

## 7. Device state touched

`svc power stayon usb` and `screen_off_timeout` were raised for the session so
the phone would not re-lock mid-run; both were restored afterwards
(`stayon false`, timeout 120000 ms). Nothing else on the device was changed.

## Done means (this pass)

| Item | Result |
|---|---|
| Grown Rust workspace still builds for `aarch64-linux-android` | **Yes**, clean |
| APK still exports and installs | **Yes** — 68 MB, after a newly-required `llvm-strip` of the 400 MB debug `.so` |
| Launch + GDExtension + GL ES 3.2 context | **Yes**, unchanged |
| Golden path drivable on device | **Yes**, end to end by touch, including overlays, settlement selection, the WHY HERE explainer, the tool rail and the Performance readout |
| Memory vs. ~283 MB baseline | **Grown: 396 MB peak at 512x512 (+40%), 316 MB steady (+17%)**; 874 MB peak at the app's 2048x1311 default |
| Leak | **None** — big-world memory fully released on regenerate |
| Generation time on device | 3.2 s (131 k cells) → 4.5 s (262 k) → 8-9 s (466 k) → **31 s (2.68 M, the default)** |
| Non-square maps | **All four shapes correct**, including Whole world 2:1 pinning |
| Phone UI | **Structurally intact, physically unusable by finger** — see §6; open item, not fixed |
| Crashes / ANRs / OOM kills | **None** |
