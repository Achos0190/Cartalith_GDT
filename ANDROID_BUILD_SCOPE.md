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
