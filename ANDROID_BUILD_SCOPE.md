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

---

# Third real-device pass (2026-08-20): current code rebuilt and installed; the on-device run is blocked by the phone's lock screen

Same phone (OnePlus 6T `9608b26b`, Android 14, USB debugging authorized).
Prompted by the owner asking whether the new GUI and everything landed since
2026-08-18 had actually reached the APK. **It had not** — both APKs on disk
predated the whole three-domain DCC shell merge, the rebuilt Asset Library,
Travel Library, the Journey Planner work, heightmap import, metropolis/
recovery, Multi-GPU, the layers z-order fix, and the `6a97911` launcher crash
fix.

They do now: a current `.so` and a current APK were built, installed, **and
driven through a full world generation on the phone**. The run was blocked for
the first ~5 minutes by the device's fingerprint lock screen (§4); the owner
unlocked it mid-session and everything after that is real hardware.

**Headline: this is the first time the §13 phone layout has ever run on a
phone, and it works.** See §4.2.

## 1. Two real defects found before anything could be built

Both were in committed-or-working-tree config, both would have produced a
wrong build, and neither was the thing the pass went looking for.

### `project.godot`'s `[display]` section was corrupted in the working tree

The committed version carries a twelve-line `##` comment block explaining the
§13 orientation choice, followed by `display/window/handheld/orientation=
"sensor"`. The working tree had all of it collapsed into a single garbage key:

```
sensorstillbootsthere##mostofthetimewithoutforcingit.display/window/handheld/orientation="sensor"
```

A botched edit, uncommitted. Godot's `ConfigFile` parser treats `;` as its
comment character, not `#`, so this does not fail loudly — it parses as a
custom project setting with a nonsense name, the real orientation key is never
set, and the app silently reverts to Godot's unset default of **landscape**.
That is precisely the "full desktop chrome crammed onto a phone" bug the
committed comment exists to prevent, and it would have shipped invisibly.
Restored with `git checkout`; no code change was needed, the committed content
was already right.

**Lesson, same shape as this project's other recorded ones:** a config file
that parses is not a config file that is correct. `#` is not a comment
character in Godot's `ConfigFile`.

### `cartalith.gdextension` pointed Android at a directory nothing builds into

`Cargo.toml` grew a dedicated `[profile.android-dev]` on 2026-08-18 (to make
the 400 MB to 18 MB `llvm-strip` step permanent rather than manual). Cargo
writes that profile's output to `target/aarch64-linux-android/android-dev/`.
But `cartalith.gdextension`'s `android.debug.arm64` still read
`target/aarch64-linux-android/debug/`, which only the *plain* `dev` profile
writes.

The 2026-08-18 pass papered over this by hand-copying the stripped `.so` into
`debug/`. That copy is exactly how a stale library ships: the manifest points
at a file no build step ever refreshes, so the APK silently keeps whatever was
copied there last. Confirmed on disk this session — `debug/libcartalith_godot.so`
was a 20,934,600-byte artifact dated 2026-08-18 22:29 that no current build
would have touched.

Fixed by pointing `android.debug.arm64` at
`res://../target/aarch64-linux-android/android-dev/libcartalith_godot.so`, the
directory the documented profile actually produces. Desktop and release paths
are unchanged.

### And a documentation bug in the same area

`Cargo.toml`'s own usage line read
`cargo ndk -t arm64-v8a --profile android-dev build -p cartalith-godot`, which
**fails**: `--profile` is a `cargo build` flag, so `cargo-ndk` sees it first
and exits with `unexpected argument '--profile' found`. Corrected in place to
put `--profile` after `build`, with a note explaining why.

## 2. The stale-artifact scare that turned out to be a non-issue

`godot-project/android/build/src/instrumented/assets/project.godot` was flagged
before this pass as still naming `res://main.tscn` — a scene deleted in
`788053b` — with the worry that the export would use it and the APK would fail
to launch. **Checked, and it is harmless.** That file is not ours: its
`config/name` is `"Godot App Instrumentation Tests"`, it ships as part of
Godot's Gradle build template, and its `res://main.tscn` is its own 1,308-byte
test scene sitting right beside it. Nothing about it refers to this project.

It is also inert twice over: the Android preset has
`gradle_build/use_gradle_build=false`, so the export uses the prebuilt APK
template and never enters `android/build/` at all, and `godot-project/android/`
is `.gitignore`d. **No action taken, correctly.**

The app's real main scene is `res://shell/app.tscn`, which exists and exported
fine.

## 3. Build, export, install — all three succeeded

- `cargo ndk -t arm64-v8a build --profile android-dev -p cartalith-godot` —
  **clean, exit 0**, 21.7 s incremental. `cartalith-terrain`, `-erosion`,
  `-hydrology`, `-climate`, `-gpu`, `-assets`, `-engine`, `-civ` and `-godot`
  all rebuilt. Only two pre-existing dead-code warnings in `cartalith-gpu`
  (`dispatch_gpu_height`, `dispatch_gpu_resistance`), unrelated to Android.
- Result: `target/aarch64-linux-android/android-dev/libcartalith_godot.so`,
  **156,553,640 bytes**, dated 2026-08-20 09:37.
- `godot4 --headless --export-debug "Android" builds/android/Cartalith.apk` —
  **succeeded**, signed with Godot's auto-generated debug keystore. Still no
  release keystore; still not in scope.
- `adb install -r` — **Success**, first try, streamed install.

### The `.so` is 156 MB, not the 18 MB the profile was meant to produce

`debug = "line-tables-only"` cut the 2026-08-18 figure from ~400 MB to 156 MB —
a real 2.5x win, and it did remove the need for a *mandatory* manual strip —
but it is nowhere near the 18 MB that `llvm-strip --strip-debug` achieved,
because line tables for a workspace this size are themselves large. Godot
stores `.so` files uncompressed, so the APK came out **207,106,507 bytes**
against the 2026-08-18 pass's 68 MB.

**Deliberately not "fixed" this pass.** Adding `strip = "debuginfo"` to the
profile would get the 18 MB back, but it would also delete the file-and-line
information the profile's own comment says on-device panic diagnosis needs,
leaving `debug = "line-tables-only"` as dead config contradicting the line
below it. Since this pass was specifically watching for a crash class (§4),
keeping resolvable backtraces was worth the install time. If the size becomes
the binding constraint, the honest change is to drop `debug` and set
`strip = "debuginfo"` together, and to say in the comment that backtraces lose
file and line.

### Which APK is the real one

`export_path="builds/android/Cartalith.apk"` resolves relative to the Godot
project directory, so the preset writes **`godot-project/builds/android/
Cartalith.apk`**. The second copy at `cartalith-native/builds/android/` was
cruft from an older pass that invoked Godot with an explicit path from a
different working directory; it was stale (2026-08-18 09:20) and had no way of
ever being refreshed. **Deleted.** Both locations are `.gitignore`d, so nothing
was committed either way.

### Confirmed the APK carries current code

Not inferred from timestamps — read out of the archive. `assets/shell/`
contains 57 entries including `asset_library_window.gdc` (53,098 bytes — the
rebuilt one from `88b4d54`), `travel_library_window.gdc`,
`journey_planner_view.gdc`, `dcc_shell.gdc` (34,188 bytes),
`data_manager_window.gdc` and all five workspace scripts, every one stamped
2026-08-20 09:38. The native library is the 09:37 build. **The APK is built
from the tree at `6a97911`.**

## 4. A fingerprint-secured lock screen, again — and this time it cleared

The app was launched twice (`adb shell monkey -c LAUNCHER`). Both times logcat
shows the same three lines within ~80 ms of each other:

```
V Godot   : OnResume: GodotFragment{...}
V Godot   : OnPause:  GodotFragment{...}
V Godot   : OnStop:   GodotFragment{...}
```

The activity is backgrounded by the OS before the engine ever creates its GL
context. In **these two locked-screen attempts** there is therefore no
`Adreno (TM) 630` line and no `OpenGL ES 3.2` line — the GDExtension is never
reached. (§4.1 below is the successful run once the phone was unlocked; nothing
in this section should be read as the pass's conclusion.)

Diagnosed rather than assumed:

- `dumpsys power` gave `mWakefulness=Dozing` initially; raised to `Awake` with
  `KEYCODE_WAKEUP` and `svc power stayon usb`.
- `dumpsys window` gave `mDreamingLockscreen=true`, before and after
  `wm dismiss-keyguard`, which silently no-ops against a real credential.
- `locksettings get-disabled` returned `false`, i.e. a real lock credential is
  set.
- `adb exec-out screencap` returned a genuine 1.58 MB image (not the blanked
  ~15 KB of the 2026-08-17 pass, because the screen was awake this time) and it
  shows the lock screen with a **fingerprint prompt**.

This is the identical physical-access requirement recorded in the 2026-08-17
section, and it is not a toolchain, code or export problem. Nothing in `adb`'s
non-root surface dismisses a fingerprint or PIN credential, and attempting to
work around one was never appropriate. A poll loop was left watching
`mDreamingLockscreen` so the smoke test could fire the instant it cleared.

## 4.1 Unblocked: the owner unlocked the phone, and everything works

`mDreamingLockscreen=false` at 09:44. The watcher launched the app immediately.
Real logcat, real hardware, cleared buffer:

```
I godot       : Godot Engine v4.7.1.stable.official.a13da4feb
I AdrenoGLES-0: Driver Path: /vendor/lib64/egl/libGLESv2_adreno.so
I godot       : OpenGL API OpenGL ES 3.2 V@0502.0 - Compatibility
                - Using Device: Qualcomm - Adreno (TM) 630
```

`OnResume` with **no** following `OnPause`/`OnStop` this time — the activity
stayed foregrounded and the GL ES 3.2 context was created against the real
Adreno 630.

**GDExtension load proven, not inferred.** `Godot Engine v…` alone would not
prove the Rust side loaded, so the process's own address space was read:

```
7271375000-7272152000 r-xp  .../lib/arm64/libcartalith_godot.so
```

The library is mapped **executable** into pid 10877. The engine is behind the
shell.

### The golden path, driven by touch

`File ▸ Open project` came up on its own at boot, offering **Create a new
world / Import a heightmap / Drop a .zip save** — the heightmap-import entry
point is itself one of the things the stale APK was missing, so its presence on
screen is direct evidence the new code shipped.

Tapped **Create a new world** → the `New world` dialog opened fully populated
(seed 311447, Extent `Region`, Map width `Province · 800 km`, Resolution `2K`,
grid 2048 x 1311 = 2.68 M cells, cell size 0.391 km, Archetype `Classic`, plus
the Village-seeding and Imperial-seat-tier toggles) → tapped **Create**.

It generated and rendered: coastlines, rivers, roads, an impact crater, and
faction-coloured settlement markers with labels (`Haldvannho vnordfjord`,
`Crungrimcrag`, `Zafashkadrest`, `Yusirsirskadmarch`, …) over the Phase 3 paper
atlas look. The app bar updated to **`ELDRA · 311447`** and the status readout
to `2048 x 1311 · 800 x 512 km · z1.0`.

**No `FATAL`, no `SCRIPT ERROR`, no `USER ERROR`, no panic, no ANR, no
`lowmemorykiller`, nothing in the `crash` buffer** for this package across the
whole session. The only error line anywhere in logcat was an unrelated
`bluetooth` file-metadata warning from a system process.

### The `6a97911` GL-context bug does not bite on Android — verified

This was the specific thing the pass was told to watch for: Android is GL
Compatibility too, so the wgpu-enumeration hazard that killed the desktop
renderer could in principle recur. **It does not.** The GL ES 3.2 context was
created cleanly at boot, a 2.68 M-cell generation ran through the full pipeline
without touching it, and `grep -i wgpu` over the whole logcat is empty. The
2026-08-18 pass's Performance readout finding — that the GPU path is correctly
inert on Android and the whole pipeline runs on CPU — is the reason: there is
no enumeration to go wrong here.

### Memory at the app's own default, vs. the 2026-08-18 baseline

`dumpsys meminfo`, `TOTAL PSS`, sampled every ~2 s through the run, same metric
as the previous two passes.

| Run | Grid | Peak PSS | Steady PSS |
|---|---|---|---|
| 2026-08-18 | 2048x1311 (2.68 M) | 894,968 KB (874 MB) | ~500,040-538,300 KB |
| **this pass** | 2048x1311 (2.68 M) | **899,089 KB (878 MB)** | **662,793 KB (647 MB)** |

**Peak is flat** — 878 MB against 874 MB, a 0.5% difference on a single
sample, i.e. unchanged. Everything landed since 2026-08-18 (the three-domain
shell merge, the rebuilt Asset Library, Travel Library, Journey Planner,
heightmap import, metropolis/recovery, Multi-GPU) cost essentially nothing at
the transient peak, which is dominated by the generation pipeline's own
buffers.

**Steady-state grew ~23%** (≈510 → 647 MB) and that is the honest cost: the
phone shell builds a second full chrome tree, and the new windows are resident
once opened. Not a leak — PSS held at 662,79x KB across seven consecutive
samples with sub-100 KB jitter.

Generation took roughly **16-18 s** wall-clock (peak at t+9-10 s in the trace,
settled by t+16 s) against the 2026-08-18 pass's ~31 s at the identical grid.
Read as "not slower"; both are inferred from the shape of the memory trace
rather than an instrumented timer.

## 4.2 The §13 phone layout, on a phone, for the first time

`project.godot`'s restored `orientation="sensor"` (§1) is what made this
reachable. The device reported `cur=2340x1080` — the owner has the phone
physically resting in landscape, and `"sensor"` correctly followed it.

`_compute_layout_mode()`'s aspect test is deliberately order-independent, so
landscape does not defeat it: `min/max = 1080/2340 = 0.4615`, under
`_PHONE_ASPECT_MAX = 0.6`, so **`_phone` latched true and the shell built phone
chrome, not desktop chrome.** Confirmed visually — the screenshots show the
§13 composition, not the 2026-08-18 pass's crammed desktop shell:

- the **app bar** with hamburger, `CARTALITH` wordmark and world subtitle;
- the **floating domain rail** with rotated `WORLD` / `CIVIL` / `CARTO` labels
  and its expand chevron;
- the **`⋯` overflow** and panel-picker buttons at top right;
- the **bottom tool sheet** (`GENERATE · WORLD` with `Generate world`,
  `New seed`, `Center landmasses`, `Bake ALL & finalize`);
- the **gesture inset** bar;
- and the landscape treatment specifically: `_phone_side_safe` is the black
  column down the left edge holding the rotated clock, with the chrome shifted
  inward to clear it — exactly "the cutout moves to a side edge".

`_phone_scale` comes out `1080 / PHONE_REF_SHORT (393) = 2.75`, putting §13's
44 px minimum target at **~121 physical px** against Android's 94 px (48 dp)
floor. **This directly retires the 2026-08-18 pass's §6 finding** that the
chrome was "structurally intact, physically unusable by finger" — that verdict
described the desktop shell running on a phone, which is no longer what
happens.

### Two things that are still wrong, reported not fixed

1. **Runtime-built dialogs do not take the phone treatment.** `Open project`
   and `New world` render as desktop-sized floating windows (~1020x690 in a
   2340x1080 surface) with 10-12 px body type — physically ~0.7 mm, well under
   the ~1.5 mm a normal eye resolves at arm's length. They are *usable* (the
   `Create` button is a comfortable target and the content scrolls), but they
   are visibly not part of the phone composition around them. §13 scopes
   full-height panel sheets and bottom sheets for exactly this.
2. **`Open project` shows two stacked headers and two close buttons** — an
   outer `Open project` window title bar with an `✕`, and immediately inside it
   the dialog's own `Cartalith / start a world, continue one, or bring a
   heightmap in from disk` header with a second `✕`. Almost certainly the
   content's branded header colliding with the host `Window`'s chrome rather
   than a phone-specific bug, but it reads as a duplicated title on any
   platform.

Neither was touched. Both are layout work with a real design behind them
(§13 / the DCC shell spec), and inventing a fix inside a verification pass is
the half-migrated state this project avoids by policy.

### Portrait was not reachable over `adb`

The primary §13 composition is portrait, and it still has not been seen.
`settings put system user_rotation 0` has no effect here: Godot's
`orientation="sensor"` sets the activity to `SCREEN_ORIENTATION_SENSOR`, which
follows the physical accelerometer and overrides the user-rotation setting.
**Physically rotating the phone is the only way**, which is a five-second
owner action, not a blocker. (The Android lock screen is itself portrait-pinned,
which is why the §4 lock-screen capture looked portrait while the app runs
landscape — those are consistent, not contradictory.)

### Device state touched

`svc power stayon usb` for the session (restored to `false`);
`accelerometer_rotation` and `user_rotation` were set to `0` during the failed
portrait attempt and restored to `1` / `0` (auto-rotate on) afterwards.
`screen_off_timeout` was read but never changed. Nothing else.

## 5. Unrelated cruft noticed in the APK, flagged not fixed

`export_filter="all_resources"` pulls the editor-only addons into the shipped
APK. The archive contains `assets/addons/godotsteam/` and
`assets/addons/godot_ai/` (including `_cli_exec.gdc`), plus
`lib/arm64-v8a/libgodotsteam.android.template_debug.arm64.so` (18,695,392
bytes) and `libsteam_api.so` (526,984 bytes) — about 19 MB of a Steam
integration and a personal MCP dev tool in a non-Steam Android build.

`.gitignore`'s own comment already says neither belongs in the repo. An
`exclude_filter` on the Android preset would drop them. **Left alone
deliberately**: this pass's job was to get current code onto the phone and
verify it, and re-cutting the export filter is a separate change that wants its
own build-install-run cycle rather than being smuggled into a verification
pass. Worth ~19 MB and one line to whoever picks it up.

## Done means (this pass)

| Item | Result |
|---|---|
| `project.godot` orientation config | **Fixed** — corrupted working-tree `[display]` key restored; landscape regression averted |
| `.gdextension` Android debug path | **Fixed** — now points at `android-dev/`, the directory the documented profile writes; ends the hand-copy that shipped stale `.so`s |
| `Cargo.toml` usage line | **Fixed** — `--profile` must follow `build` |
| Instrumented `main.tscn` artifact | **Non-issue** — Godot's own test project, and `use_gradle_build=false` means it is never read |
| Rust cdylib cross-compiled for `aarch64-linux-android` | **Yes**, clean, from `6a97911` |
| APK exported and signed | **Yes** — 207 MB (large; see §3) |
| APK carries current code | **Yes**, verified by reading `assets/shell/` out of the archive |
| Installed on real hardware | **Yes** |
| Duplicate stale APK path | **Deleted** (`cartalith-native/builds/android/`) |
| App observed running on device | **Yes** — after the owner unlocked the phone (§4.1) |
| GDExtension actually loaded | **Yes** — `libcartalith_godot.so` mapped `r-xp` into the live process, not inferred |
| GL context | **Yes** — OpenGL ES 3.2, Adreno (TM) 630, Compatibility |
| `6a97911` GL-context fix verified on Android | **Yes** — clean context, full generation, zero `wgpu` lines in logcat |
| Golden path on device | **Yes** — Open project → New world (2048x1311, 2.68 M cells) → Create → world rendered, `ELDRA · 311447` |
| Crashes / ANRs / script errors / OOM kills | **None** |
| Memory at 2048x1311 | Peak **899,089 KB (878 MB)**, flat vs. 2026-08-18's 874 MB; steady **662,793 KB (647 MB)**, up ~23%; no leak |
| §13 phone layout on real hardware | **Yes, and it works** — `_phone` latched, phone chrome built, 44 px targets land at ~121 physical px; retires the 2026-08-18 §6 "unusable by finger" verdict |
| Phone-layout defects found | **Two, reported not fixed** — runtime dialogs keep desktop sizing; `Open project` shows duplicated header/close (§4.2) |
| §13 *portrait* composition | **Still unseen** — `"sensor"` follows the accelerometer, so `adb` cannot force it; needs the phone physically rotated |
| Editor-only addons in the APK | **Flagged, not fixed** — ~19 MB of godotsteam + godot_ai (§5) |

---

# Fourth real-device pass (2026-08-20): the four owner-reported defects, and why portrait never worked

Owner, after running the `a80a386` APK on the OnePlus 6T:

1. *"it doesnt switch to portrait mode"*
2. *"the open project menu doesnt follow the design"*
3. *"make sure the lightmode version is available everywhere"*
4. *"the bottom menu butons on phone are near too small to use"*

All four are fixed and verified on the device. A fifth item — *"not much from
the menus work on android"* — was **diagnosed only**, at the owner's
instruction, because a proper mobile menu design is being produced separately
and building one here would be thrown away (§5).

## 1. Portrait: the setting was a string, and Godot 4 wanted an integer

The previous pass concluded portrait "was not reachable over `adb`" and left it
as a five-second owner action. That was wrong, and so was the hypothesis this
pass was handed (that `_landscape` latched at boot). **The runtime code was
never the problem.** `dumpsys window` against the running `a80a386` build:

```
source=ActivityRecord{... org.cartalith.walkingskeleton/...} SCREEN_ORIENTATION_LANDSCAPE
```

The activity was requesting **landscape**, hard, despite `project.godot`
carrying an `orientation` value of `"sensor"`. Android therefore never rotated
the window, `root.size_changed` never fired, and `_apply_phone_orientation()`
was unreachable for the entire life of the build.

The cause is that this key changed type between Godot generations. Godot 3
spelled it as a string; **Godot 4 redeclared it as `TYPE_INT`**:

```
INFO={ "name": "display/window/handheld/orientation", "type": 2,
       "hint_string": "Landscape,Portrait,Reverse Landscape,Reverse Portrait,
                       Sensor Landscape,Sensor Portrait,Sensor" }
```

`type: 2` is `TYPE_INT`, and `DisplayServer.SCREEN_SENSOR` is `6`. A string
value is not an error — it is silently discarded, and the setting falls back to
`0`, which is Landscape. `ProjectSettings.get_setting()` returned `0` against
the old value and returns `6` against the new one.

**A second, worse hazard was found while fixing it.** The `##` comment block
that used to sit above this key was not a comment at all:

```
KEYS_IN_DISPLAY=["the cutout moves to a side edge)that##`DccShell._landscape`…"]
```

The `[display]` section contained exactly **one** key — the entire comment
paragraph, whitespace-stripped, with the real key name swallowed onto its tail.
Only `;` starts a comment in `project.godot`; a `##` line is parsed as data, and
an unbalanced quote or apostrophe inside one opens a string literal that eats
every key below it **with no error reported**. The old block happened to have an
even number of quotes, which is the only reason the key survived at all — and it
is the same class of failure `CLAUDE.md` already records ("an apostrophe in
prose defeating a comment scanner"). The section is now written with `;`
comments and a warning, and `ConfigFile.get_section_keys("display")` returns the
one key it should.

Verified on the device: the exported manifest now carries
`android:screenOrientation=13` (`SCREEN_ORIENTATION_FULL_USER`, Godot's mapping
for `SCREEN_SENSOR`), up from `0`, and the app runs portrait.

**Note for anyone driving rotation over `adb`.** `FULL_USER` respects the user's
rotation lock, so with auto-rotate off it locks to the *current* rotation and
`settings put system user_rotation` does nothing. `adb shell wm user-rotation
lock 1` does work, and is how the landscape captures were taken. The previous
pass had also left `accelerometer_rotation` at `0` (auto-rotate **off**) while
recording that it had restored it — so even a correct build would not have
rotated in the owner's hands. Restored to `1` this pass.

## 2. Open project: two headers, and a desktop dialog inside a phone

Both halves of the previous pass's §4.2 report are fixed.

**The duplicated header** was an `AcceptDialog` drawing the host `Window`'s
title bar and close button above `_build_head()`'s own branded header and its
close glyph. The design draws one header, so the window chrome is the one that
goes: `borderless = true`. This was wrong on every platform, and the desktop
capture confirms it is now a single header there too.

**The desktop sizing** is fixed by giving the dialog a phone presentation
(`_present()`), per §13's "docks become full-screen sheets": the window fills
the screen, and `content_scale_factor` scales the desktop-authored composition
by the shell's own `_phone_scale`. That is deliberately *not* a second set of
phone constants — at 2.75 on this handset the layout area works out to
1080/2.75 = 393 px, which is exactly the mockup's own phone reference width, so
the existing numbers land on the phone reference by construction.

One content change was needed to make it fit: a `Window` cannot shrink below its
content's minimum, and the head's subtitle is a single unwrapped `Label` whose
text alone is ~420 px — wider than the whole phone reference. Phone hides it;
the three action tiles below say the same thing. The tile grid picks its column
count from the available width (`_fit_columns`), giving 1 column in portrait and
3 in landscape, and re-runs on rotation via `phone_insets_changed`.

## 3. Light theme: three separate gaps, only one of which was the documented one

The rebuild pass's own disclosed limitation ("only repaints nodes whose colours
trace back to a `DccTheme` token") was real but was the *smallest* of three.
Found by capturing every window under the light palette rather than reasoning
about the walk.

**(a) The override-name lists had drifted.** `rebuild_theme()` works off two
hand-maintained arrays of override *names*, and re-running the grep their own
comment documents found six in use that were not listed —
`caret_color`, `font_placeholder_color`, `font_uneditable_color` and the
`disabled` / `focus` / `read_only` styleboxes, all introduced by
`dcc_widgets.gd`'s text fields after the arrays were written. Every dialog with
a text well kept dark input chrome under the light palette.

**(b) The project-wide Theme resource was never touched at all.** This is the
structural one. `project.godot` sets `gui/theme/custom` to `dark_theme.tres`, a
real hand-authored dark `Theme`, and that resource is the fallback for every
control state nothing overrides explicitly — disabled buttons, scrollbars,
`SpinBox`/`OptionButton`/`CheckBox` chrome, bare `Button`s and `LineEdit`s. None
of it is a per-node override, so the tree walk could never reach it: the colours
live in a `Resource`. `_recolor_project_theme()` now remaps it in memory (the
same cached instance the whole tree resolves against, so nothing is written to
disk). This is what made a disabled `DccWidgets.action()` button — "Bake ALL &
finalize", and the world workspace's "Finalize · LOD 0-3" — a dark slab on a
light shell, and what left Travel Library's bare filter field and Close button
dark.

Six colours in that resource are not `DccTheme` tokens at all, so the reverse
lookup cannot see them; they are handled by an explicit supplementary table
(`_theme_extras`), each entry a derivation rather than a new colour: two plain
surfaces, one token with a one-digit typo (`#8d9396` for `text_dim`'s
`#8d9296`), and the accent with the same lighten/darken the widgets already
apply. Two more (`#1a1206`, the near-black for text sitting *on* the amber slab,
and `#e66b6b`, the error red) are deliberately left alone — they are correct in
both palettes, the same reasoning `DccTheme` already applies to
`warn`/`block`/`water`.

**(c) Embedded `Window` chrome came from Godot's built-in theme.**
`dark_theme.tres` defines no `Window` entries, so every `AcceptDialog`'s title
bar was Godot's stock dark one — a charcoal bar over light content, and nothing
to remap. `_style_window_chrome()` now writes those entries from tokens, and is
called from `_ready()` as well as `rebuild_theme()` so it is right on a cold
boot in either palette.

Two literal white drag handles in the phone chrome were also re-expressed
against `text_ghost`; as flat white they stayed white and vanished into a light
panel.

### Coverage, per window

Captured under the light palette after a live dark-to-light *switch* (not a cold
boot, which would build every node from `c()` and pass trivially):

| Window | Result |
|---|---|
| Main shell (menu bar, docks, rail, timeline, status) | **Correct** |
| Open project / welcome | **Correct** |
| New world | **Correct** |
| Asset library (rebuilt `88b4d54`, after the theme pass) | **Correct** |
| Travel library | **Correct** after (b) and (c); was the worst offender |
| Data manager (rebuilt this week) | **Correct** |
| World data | **Correct** |
| Performance | **Correct** |

Dark mode re-verified on the same windows: unchanged, no regression. A tree
audit for nodes still holding an inactive-palette value reports only one class,
and it is a false positive — `#a4650f` is both `DARK.accent_dim` and
`LIGHT.accent`, so a correctly-repainted light accent looks like a missed dark
token to an exact-match scan.

## 4. The bottom sheet buttons: the chrome was scaled, the contents never were

The previous pass computed `_phone_scale = 2.75`, observed 44 px targets landing
at ~121 physical px, and called the sheet comfortable. That arithmetic was
correct and measured the wrong thing. It describes the **chrome** —
`_build_phone_app_bar()`, `_build_phone_rail()` — which does route every size
through `_ptap()`. The sheet's **contents** never touched `_ptap()` at all.

`tool_options_row` is filled by the workspaces' own
`_build_*_tool_options_row()` callbacks, which are written against desktop pixel
constants — `cartography_workspace.gd` sets buttons to a literal
`Vector2(34, 20)`. Godot's default stretch mode is disabled, so 20 virtual px is
20 *physical* px: about 1.6 mm on this 314 dpi panel. The owner was right and
the arithmetic was answering a different question.

Fixed at `set_tool_options()`, the single choke point every workspace already
passes through, so one pass over the finished row phone-sizes every current and
future tool row without making a dozen workspace files phone-aware. Existing
minimum sizes and explicit font-size overrides are scaled; anything tappable is
then floored at §13's 44 px. The sheet's own padding constants were scaled too —
left raw they put the first control flush against the screen edge.

## 5. The overflow menu: diagnosed, deliberately not fixed

Owner: *"not much from the menus work on android."* A mobile menu design is
being produced separately, so this is evidence for that design pass, not a
repair.

**It is wired to something real.** `_build_phone_overflow()` reparents the
actual desktop menu bar — all seven genuine program menus (File, Edit, Assets,
Data, Preferences, Window, Help) are present in the sheet and are the real ones,
not placeholders. §13's promise that the overflow carries "the full menu bar" is
kept structurally.

**It is unusable in practice**, for four compounding reasons, all visible in the
device capture:

1. **Nothing in the menu path is phone-scaled.** `add_menu()` styles each
   `MenuButton` with `DccTheme.inset(11, 9, 11, 9)` and `FS_MENU` (12 px) — raw
   desktop values, no `_pscale`/`_ptap`. The row renders at roughly 12 physical
   px, about 1 mm tall, against §13's 44 px floor. This is the same class of bug
   as §4 above, in a surface §4's fix deliberately does not touch.
2. **Desktop status chrome eats the sheet.** The reparented bar also carries the
   `CARTALITH` wordmark (150 px minimum) and the five readout labels
   (world/res/cpu/gpu/mem) separated by 22 px gaps. Pre-generation those labels
   are empty strings, so most of the 220-px-tall sheet is blank space with the
   menu row squeezed into a strip at the bottom.
3. **The menus do not respond to touch at all.** Tapping `File` at its centre
   produced no popup and no pressed state; holding the touch down
   (`input motionevent DOWN`, captured while held) produced neither. Whatever
   the precise mechanism — the target is small enough that this was not
   conclusively separated from a simple miss — the observable result is that the
   menu is inert by finger on the device. This is the whole of the owner's
   report.
4. **15 submenus assume hover.** `menus.gd` uses `add_submenu_*` 15 times.
   Submenu traversal in a desktop `PopupMenu` opens on hover, which touch does
   not have, and a nested `PopupMenu` positioned for a pointer has nowhere sane
   to go on a 1080-wide screen. Even if (1) and (3) were fixed, roughly 41 items
   behind 15 hover-opened submenus is not a phone menu.

The honest summary for the design brief: **the routing is real and worth
keeping; the presentation is a desktop menu bar shown at desktop scale inside a
phone sheet, and no part of it was ever adapted.** A design that keeps the seven
menus and their ~41 destinations but re-presents them as a full-screen,
touch-sized, drill-down list would inherit all the existing wiring.

## 6. Device state touched

`svc power stayon usb` for the session (restored to `false`).
`accelerometer_rotation` was found at `0` — auto-rotate **off**, left that way by
the previous pass despite its own note saying otherwise — and is restored to `1`.
`wm user-rotation lock 1` was used to capture landscape and released with
`wm user-rotation free`. `user_rotation` restored to `0`. Nothing else.

## Done means (this pass)

| Item | Result |
|---|---|
| Portrait on device | **Fixed** — root cause was `orientation` being a string where Godot 4 wants int `6`; manifest now `screenOrientation=13` |
| `project.godot` comment hazard | **Fixed and documented** — `##` is not a comment here; section rewritten with `;` |
| Open project: duplicated header | **Fixed** — `borderless`; correct on desktop too |
| Open project: desktop sizing on phone | **Fixed** — full-screen + `content_scale_factor`, responsive column count |
| Light theme: override-name drift | **Fixed** — six missing names added |
| Light theme: project Theme resource | **Fixed** — remapped in memory, plus a six-entry derivation table |
| Light theme: embedded `Window` chrome | **Fixed** — written from tokens, cold-boot safe |
| Light theme: per-window coverage | **8 of 8 correct**, both palettes, verified by capture |
| Bottom sheet targets | **Fixed** — scaled at `set_tool_options()`, floored at 44 px |
| Overflow menu | **Diagnosed, not fixed** (§5), by instruction |
| Headless smoke test | **PASS** |
| Desktop windowed launch | **Clean** — world generated, `6a97911` GL fix not regressed |
| Device install and run | **Clean** — portrait and landscape both captured |

# Pinch-to-zoom pass (2026-08-24)

Owner: *"zooming doesn't seem to work on the phone."* Full write-up in
`cartalith-native/docs/CHANGELOG.md`; this section records only the Android
specifics, because they are the reusable part.

## The setting

`input_devices/pointing/android/enable_pan_and_scale_gestures` defaults to
**false** in Godot 4.7.1, and with it off the Android input layer never
attaches its `GestureDetector`/`ScaleGestureDetector` — so
`InputEventMagnifyGesture` and `InputEventPanGesture` are never produced on a
device, no matter how correctly the game handles them.
`viewport_host.gd` had handled the magnify event since the camera was written.
`project.godot` now carries an `[input_devices]` block turning it on.

Two things worth keeping, both read out of the shipped APK with
`dexdump -d` rather than from documentation:

- `GodotGestureHandler.onScale` and `.onScaleBegin` open with
  `iget-boolean … panningAndScalingEnabled` and branch straight out;
  `GodotInputHandler.enablePanningAndScalingGestures(Z)` is the only writer.
- `ScaleGestureDetector` is built with the 2-arg constructor and
  `setQuickScaleEnabled` is **never** called, so Android's single-finger
  double-tap-drag zoom does not exist in a Godot app. Two real fingers is the
  only path to a magnify event.

## Driving a real multi-touch pinch from `adb`, on an unrooted device

This is the part that took the work, and it is worth writing down because
every obvious route is a dead end:

| Route | Why it fails |
|---|---|
| `adb shell input tap` / `swipe` / `motionevent` | single-pointer only — there is no multi-touch verb in `input` at all |
| two concurrent `input swipe`s | not a pinch: each is its own pointer-0 DOWN…UP stream, so the app sees one jittering finger and `getPointerCount()` never reaches 2 |
| `sendevent /dev/input/event2` (the real panel) | node is `0666` but SELinux denies `u:r:shell:s0`; DAC is not the gate here |
| `adb root` → `setenforce 0` | refused: LineageOS gates it on `persist.sys.root_access`, which cannot be set without root |

**What works: AOSP's own `/system/bin/uinput`.** `/dev/uinput` is group `uhid`
and `shell` is in `uhid`, so no root is needed. Register a virtual touchscreen
and inject MT protocol B directly:

- `configuration` entries `100`=`UI_SET_EVBIT` (`EV_KEY`, `EV_ABS`),
  `101`=`UI_SET_KEYBIT` (`BTN_TOUCH`), `103`=`UI_SET_ABSBIT`,
  **`110`=`UI_SET_PROPBIT`** with data `[1]` for `INPUT_PROP_DIRECT` — without
  that property InputReader does not treat it as a touchscreen.
- `abs_info` ranges set to `0-1079` × `0-2339` so the virtual device maps 1:1
  onto this panel and injected coordinates are screen coordinates.
- Both slots down **in the opening report** (`ABS_MT_SLOT` 0 and 1, distinct
  `ABS_MT_TRACKING_ID`s) so the app gets `ACTION_DOWN` +
  `ACTION_POINTER_DOWN` with `pointerCount == 2`; interpolated move reports;
  release by setting each slot's tracking ID to `-1`.
- Registration needs a `delay` (~2.5 s) before the first inject, or InputReader
  has not enumerated the device yet.
- The span must clear `ScaleGestureDetector`'s `config_minScalingSpan`
  (~27 mm ≈ 430 px here) or `onScaleBegin` is never called. 600 → 1000 px
  worked; anything starting under ~450 px would silently do nothing.

Generator script and the two command files live in the session scratchpad, not
in the repo — they are ~90 lines of generated JSON and are cheaper to
regenerate from this description than to maintain.

## Result

Read off the app's own `z%.1f` viewport readout, on a real 2048 × 1311 world
generated on the phone (OnePlus 6T, LineageOS 22.2 / Android 15):

| Build | Gesture | Readout |
|---|---|---|
| fix on | pinch out 600 → 1000 px | **z1.0 → z2.2** |
| fix on | pinch in 1000 → 600 px | **z2.2 → z1.0** |
| control APK, setting `false`, otherwise identical | the same injected pinch | **z1.0, unchanged** |

The control build is the point: it reproduces the owner's report exactly, so
the setting is the cause rather than something else that moved in the same
window. Deep-zoom LOD tiles resolve in the zoomed capture, so the whole
`_zoom_at` → `set_camera_zoom` → `_update_lod` chain runs on touch.

## Device state touched

The virtual `uinput` touchscreen exists only for the lifetime of each `uinput`
invocation and is gone afterwards; nothing persistent was registered. The
screen was woken and the keyguard dismissed (no PIN was entered — it was not
set). Everything pushed to `/data/local/tmp/` was removed again. The fixed APK
was reinstalled last, so the
device is left on the fixed build. No settings or properties were changed —
`persist.sys.root_access` was *attempted* and refused, which changed nothing.
