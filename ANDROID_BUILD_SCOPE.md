# ANDROID_BUILD_SCOPE.md — the Android build, and the real-device passes

**What this is:** how this port is built, packaged, installed and driven on a
real Android handset — the durable method, the hazards each device pass found,
and the measurements each took, kept with their date and build. **What it is
not:** status. What each pass left, and where Android stands, is
`cartalith-native/docs/STATUS.md`'s "Android build and device" group
(AND-1…AND-12); open device work is routed to this file from
`OUTSTANDING_WORK.md`. The APK drop location is `GUI_GAP_REGISTER.md` §58
("Build and artefact"); signing policy is Ruling 23 in `LARGE_ITEM_RULINGS.md`.

**Every figure in §3 is one handset reading of one build.** And **any Android
memory figure must state its seed** (`MEMORY_OPTIMIZATION_SCOPE.md`): the New
World dialog rerolls the seed on every open, and seed alone spread six
otherwise identical runs over 160 MB (§3.8).

---

## 1. Build and package

### 1.1 Toolchain

Checked fresh on the first pass (2026-08-17) and all already in place:
`rustup` target `aarch64-linux-android`; `cargo-ndk` (4.1.2); the NDK at
`AppData\Local\Android\Sdk\ndk\29.0.14206865` with `ANDROID_NDK_HOME` set;
Godot 4.7.1. The `godot4` WinGet shim is on PowerShell's `PATH` and **not** Git
Bash's, so Godot invocations go through PowerShell or the full path
(`Godot_v4.7.1-stable_win64_console.exe`). `TOOLCHAIN.md` called Android "the
single highest-risk item in the toolchain"; the risk never materialised.

### 1.2 Two native libraries, one refresh command each

`godot-project/cartalith.gdextension` is the authority, in its own `;`
comment block:

| Entry | Directory | Refreshed only by |
|---|---|---|
| `android.debug.arm64` | `target/aarch64-linux-android/android-dev/` | `cargo ndk -t arm64-v8a build --profile android-dev -p cartalith-godot` |
| `android.release.arm64` | `target/aarch64-linux-android/release/` | `cargo ndk -t arm64-v8a build --release -p cartalith-godot` |

`--profile` / `--release` are `cargo build` flags and must follow `build`;
`cargo ndk --profile …` fails with *"unexpected argument"*. **Nothing else
writes either directory, and an export packages whatever `.so` is there** — run
the matching command before the matching export. Both entries have shipped
stale: the debug entry once read `…/debug/`, which only a hand-copy refreshed
(fixed 2026-08-20), and the release entry resolved to an eight-day-old build
because no documented step ran `--release` (fixed 2026-08-24). The orphaned
`…/debug/` hand-copy was deleted.

`[profile.android-dev]` (`cartalith-native/Cargo.toml`) inherits `dev`
(`opt-level = 1`) with `debug = "line-tables-only"`: panic backtraces still
resolve to file and line, which on-device diagnosis needs, without the full
debuginfo that made the plain debug `.so` 400 MB. It is still large — §3.9 has
the sizes — because line tables for this workspace are large and Godot stores
`.so` files uncompressed. Adding `strip = "debuginfo"` would recover the size
and delete the file/line information; if size becomes binding, drop `debug` and
set `strip` together, and say in the comment that backtraces lose file and line.

### 1.3 Export, sign, verify

- **Preset** `"Android"` in `godot-project/export_presets.cfg`: arm64,
  `package/signed=true`, `package/unique_name="org.cartalith.walkingskeleton"`,
  `gradle_build/use_gradle_build=false` (the prebuilt template — so
  `godot-project/android/build/`, including Godot's own instrumentation-test
  `project.godot` that names a `res://main.tscn`, is never read), and
  `exclude_filter="addons/godotsteam/*,addons/godot_ai/*,_*"`: the two
  editor-only addons since `d044af9` (2026-08-20, ~19 MB of Steam SDK and a dev
  tool), the `_*` probe and shot scenes since `686cd2a` (2026-09-03, under the
  owner's scoped authorisation to edit that line and nothing else in the file).
- **The only real APK** is `godot-project/builds/android/Cartalith.apk`:
  `export_path` resolves relative to the Godot project. A stray
  `cartalith-native/builds/android/` copy appeared twice from a wrong working
  directory and was deleted twice. Both paths are `.gitignore`d.
- **Signing — Ruling 23: no release keystore, deliberately.** `--export-release`
  fails at *"Code Signing: Could not find release keystore"*, and **the unsigned
  APK it leaves is the good one.** The 2026-09-07 drop signed that APK with
  Godot's own debug keystore (`apksigner`, verified `CN=Godot`) — ~57 MB, from
  the release `.so`. `--export-debug` signs in one step but packs the
  `android-dev` `.so` and yields a ~207 MB APK.
- **Verify the library inside the APK, not the timestamp**: sha256
  `lib/arm64-v8a/libcartalith_godot.so` out of the archive against the file just
  built, and keep a control — the previous APK should carry a *different* hash.
  A `.so` 25 commits behind its shell shipped through two clean device passes
  (§3.7).
- The launcher activity is `org.cartalith.walkingskeleton/com.godot.game.GodotAppLauncher`.
  `am start` on `.GodotApp` throws `SecurityException` (not exported); resolve it
  with `cmd package resolve-activity --brief` rather than guessing.

### 1.4 Configuration hazards

- **In `project.godot` and `cartalith.gdextension`, only `;` starts a
  comment.** Godot's `ConfigFile` parses a `#` or `##` line as **data**, and an
  unbalanced quote or apostrophe inside one opens a string that silently
  swallows every key below it. Twice this left the `[display]` section holding a
  single garbage key with the real one eaten (§3.3, §3.4).
- **`godot --headless --import` strips `project.godot`'s `;` comments**,
  including the block warning about the rule above. md5 the file before and
  after every Godot invocation and restore it if the comments went.
- **`window/handheld/orientation=6`** (`DisplayServer.SCREEN_SENSOR`) under
  `[display]`. **Godot 4 declares this key `TYPE_INT`**; the Godot 3 string form
  `"sensor"` is not an error — it silently resolves to `0`, Landscape, so the
  exported activity requested `SCREEN_ORIENTATION_LANDSCAPE` and
  `_apply_phone_orientation()` never ran. `6` exports as manifest
  `screenOrientation=13` (`SCREEN_ORIENTATION_FULL_USER`). Sensor rather than a
  portrait lock, because the phone composition has a landscape treatment.
- **No `stretch/mode` key**, deliberately — the shell does its own scaling
  (`project.godot`'s own `;` block explains why a stretch transform would
  multiply it a second time).
- **`pointing/android/enable_pan_and_scale_gestures=true`** under
  `[input_devices]`. The engine default is `false`, and with it off Android
  never attaches its `GestureDetector`/`ScaleGestureDetector`, so
  `InputEventMagnifyGesture` never arrives however well the handler is written
  (§3.5). `emulate_mouse_from_touch` stays at its default `true`, which is why
  one finger delivers both touch and emulated-mouse events
  (`ANDROID_UI_SPEC.md` §1.12, `_family`).

### 1.5 Detecting a stale library at runtime

`engine_bridge.gd` guards every native wrapper with `EngineBridge._has(method)`,
which returns a safe default when the loaded GDExtension lacks the method — so a
shell degrades instead of crashing against an older binary — and
`push_warning()`s **once per method name**:

```
Cartalith: the loaded GDExtension has no WorldGen.<name>(). Whatever needed it
is degraded to a safe default. This almost always means the native library is
older than the shell (a stale libcartalith_godot.so) -- rebuild and re-export
before treating the missing feature as a bug.
```

`missing_bindings()` returns the accumulated set at runtime. **The warning
reaches logcat** — measured 2026-09-07 on a release export, driven through the
app's own UI: `E/godot … WARNING: …` followed by `at: push_warning
(core/variant/variant_utility.cpp:1033)`. That `at:` frame is what distinguishes
it from the same marker string appearing under another tag. **Read logcat for it
before trusting any on-glass result about a native-backed feature**
(`MISTAKES.md`, "Trust an on-glass result"): Android exports that shipped the
`android-dev` `.so` logged it on every cold boot while four batches of on-glass
claims went unexamined. `dcc_shell.gd` carries the shell-side twin — a build id
printed at boot, so two APKs of the same library can be told apart.

---

## 2. Driving and measuring a handset

### 2.1 Getting the app on screen

- **A secure lock screen is a physical-access requirement.** Launched under it,
  the activity logs `OnResume` → `OnPause` → `OnStop` within ~80–140 ms and the EGL
  surface is abandoned (`eglSwapBuffers failed: EGL_BAD_SURFACE`), so the GL
  context and the GDExtension are never reached. `wm dismiss-keyguard` silently
  no-ops against a real credential (`locksettings get-disabled` → `false`), and
  `screencap` is **blanked** while a secure keyguard is up (a ~15 KB image). Wake
  with `KEYCODE_WAKEUP`, hold with `svc power stayon usb`, and ask the owner to
  unlock; a poll on `mDreamingLockscreen` can launch the instant it clears.
  Never attempt to work around a credential.
- **Command-line arguments cannot be injected into a release build.** `am start
  --esa command_line_params "--verbose"` is accepted (*"has extras"*) and arrives
  empty (`D/GodotActivity: … with parameters []`); do not plan a device probe
  around `_cl_` flags — drive it through the UI.
- **Rotation.** `FULL_USER` respects the rotation lock, so `settings put system
  user_rotation` does nothing. `adb shell wm user-rotation lock 1` does rotate
  (the fourth pass captured landscape this way); release it with `wm
  user-rotation free`. Restore `accelerometer_rotation` to `1` afterwards — one
  pass left auto-rotate off while recording that it had restored it, so a
  correct build would not have rotated in the owner's hands.
- **Restore every device setting touched** (`stayon`, `screen_off_timeout`,
  rotation, any density override) and say what was left. A mutation harness
  once restored every source file and left its mutated APK installed on the
  handset; hash the installed `base.apk` (`adb shell md5sum` of `pm path`'s
  result) against the build you meant.

### 2.2 Input

- **`adb shell input tap` is a zero-area synthetic pointer.** It proves event
  routing, never that a finger can hit the target. Size claims come from the
  framebuffer: pixels ÷ px/mm. The 6T is **401.6 ppi = 15.81 px/mm**, from the
  panel's own 6.41 in diagonal — not from the density override, which does not
  describe the glass. And **scan before believing a picture**: a rendered
  screenshot once *appeared* to show a control that a per-row brightness scan
  proved was not in the pixels.
- **Long press:** `input swipe X Y X+1 Y+1 900` — a zero-length swipe with a
  duration.
- **Back.** `DccShell._ready()` sets `quit_on_go_back = false`, and
  `_notification()` handles `NOTIFICATION_WM_GO_BACK_REQUEST` one level at a
  time — dialog, then phone-menu level, then overlay — ending at the same
  save/discard/cancel gate File ▸ Close project uses. Before that (2026-08-24)
  `KEYCODE_BACK` killed the process and the generated world. A drive can still
  reach the exit gate by pressing Back at the viewport; dismiss a sheet with its
  ✕.
- **Injected key events do not reach the shortcut path** — `KEYCODE_M` did not
  arm Measure (2026-08-24). Keyboard shortcuts are not an `adb` route around
  touch navigation.
- **A real two-finger pinch without root — AOSP's `/system/bin/uinput`.** Every
  obvious route fails: `input` has no multi-touch verb; two concurrent `input
  swipe`s are two pointer-0 streams, never `pointerCount == 2`; `sendevent` to
  the panel node (`0666`) is denied by SELinux for `u:r:shell:s0`; `adb root` →
  `setenforce 0` is refused (LineageOS gates it on `persist.sys.root_access`).
  `/dev/uinput` is group `uhid` and `shell` is in `uhid`, so register a virtual
  touchscreen and inject MT protocol B:
  - `configuration` entries `100` = `UI_SET_EVBIT` (`EV_KEY`, `EV_ABS`),
    `101` = `UI_SET_KEYBIT` (`BTN_TOUCH`), `103` = `UI_SET_ABSBIT`, and
    **`110` = `UI_SET_PROPBIT` with data `[1]` (`INPUT_PROP_DIRECT`)** — without
    it InputReader does not treat the device as a touchscreen;
  - `abs_info` ranges `0-1079` × `0-2339`, so injected coordinates are screen
    coordinates on this panel;
  - both slots down **in the opening report** (`ABS_MT_SLOT` 0 and 1, distinct
    `ABS_MT_TRACKING_ID`s) so the app sees `ACTION_DOWN` + `ACTION_POINTER_DOWN`
    with two pointers; interpolated moves; release by setting each slot's
    tracking ID to `-1`;
  - a ~2.5 s `delay` after registration before the first inject, or InputReader
    has not enumerated the device;
  - a span that clears `ScaleGestureDetector`'s `config_minScalingSpan`
    (~27 mm ≈ 430 px here): 600 → 1000 px works; a start under ~450 px silently
    does nothing.

  The generator is cheaper to rebuild from this description than to keep (~90
  lines of generated JSON); the virtual device exists only for each `uinput`
  invocation.
- **Diagnose on the desktop, confirm on glass.** `--force-touch` at a phone
  viewport boots the phone composition with synthesised `device = -1` pointer
  events — what Android's mouse emulation stamps — so hit-testing,
  `mouse_filter` and the hold timer are genuinely exercised, with full console
  output, in seconds. It is **not** the phone (`MISTAKES.md`, "Verify anything
  phone-shaped"): it cannot show an IME covering a form or anything a real
  finger does differently, and the handset alone could not have shown which node
  was eating a tap (§3.6).

### 2.3 Measuring

- **Memory:** `adb shell dumpsys meminfo org.cartalith.walkingskeleton`, the
  `TOTAL PSS:` line — grep that string exactly; a bare `grep TOTAL` also matches
  the summary table and gives a different number. Poll from the host (a process
  cannot see its own allocator), sample continuously through a transient peak,
  and **state the seed**.
- **Frame time:** `dumpsys gfxinfo` is useless here — Godot renders into its own
  `SurfaceView`, not through HWUI, so after the splash `Total frames rendered`
  sticks at 0 and every percentile reads `4950ms`. Use SurfaceFlinger:

  ```
  adb shell dumpsys SurfaceFlinger --list | grep -i cartalith
  adb shell "dumpsys SurfaceFlinger --latency '<the SurfaceView[...GodotAppLauncher](BLAST)#N line>'"
  ```

  Line 1 is the refresh period in ns; each following line is `desired present
  ready`. Take column 2, sort, de-duplicate, difference. The buffer holds ~128
  frames, so dump **immediately** after the gesture, and look at large gaps
  before filtering them — the interesting number is often one long frame.
- **Logcat:** GDScript `print()` arrives as `I/godot` (an earlier note here said
  it never appeared; that was true of an older template and is false now);
  `push_warning` as `E/godot` plus an `at: push_warning` frame (§1.5). **A
  complete successful generation writes zero `godot`-tagged lines**, so a
  silent log is the expected state whether or not anything is wrong. `wgpu`'s
  own diagnostics reach logcat only since `cartalith-godot` registered a `log`
  backend (`install_logger()`, `android_logger`); before that, "zero `wgpu`
  lines" could not fail and proved nothing (§3.3).
- **That the library loaded:** read `/proc/<pid>/maps` for
  `libcartalith_godot.so` mapped `r-xp` — the engine banner alone does not prove
  the Rust side loaded.

---

## 3. The device passes

One OnePlus 6T (`ONEPLUS_A6013`, serial `9608b26b`, 1080 × 2340) throughout —
Android 14 at the first passes, LineageOS 22.2 / Android 15 from 2026-08-24.
Headings are dated; STATUS.md's AND rows cite them.

### 3.1 First pass (2026-08-17) — toolchain, install, golden path

- Release `.so` built clean (2 m 38 s, 14.5 MB); `--export-release` failed at
  signing as expected; the debug `.so` was rebuilt and `--export-debug`
  succeeded. `adb install -r` first try.
- Logcat: Godot 4.7.1, `renderer: gl_compatibility`, OpenGL ES 3.2 on the
  Adreno 630, GDExtension loaded through `nativeloader`, no crash, no ANR.
- Idle meminfo after launch: PSS 151 982 KB, Private Dirty 78 244 KB, Native
  Heap (private dirty) 51 392 KB, RSS 261 932 KB.
- Blocked first by the secure lock screen (§2.1); once the owner unlocked it,
  **Generate** was tapped on the app's defaults (512×512, seed 12345, 800 km,
  Classic, 40 settlements) and memory sampled every ~1 s: PSS 251 519 KB at the
  tap, 257 166 at +1 s, 269 070 at +2 s, 270 822 at +5 s, 276 018 at +6 s, peak
  **283 326 KB at +7 s** (native heap private dirty 89 520 → 123 052 KB), then
  steady ~271 290 KB flat across four samples; ~7–9 s wall-clock. A second
  screenshot showed the identical map — the same-seed determinism a regenerate
  should produce.

### 3.2 Second pass (2026-08-18) — the grown workspace, re-verified

Everything since 2026-08-16 (two GUI replacements, 57 generation controls, the
New World dialog, independent `gw`/`gh`, four crates, terrain appearance
milestones 2–5) had never run on hardware. Nothing crashed, so nothing was
fixed.

- **The debug `.so` had grown to 400 480 048 bytes of debuginfo**;
  `llvm-strip --strip-debug` took it to 18 372 760 (22×, no behaviour change),
  giving a 68 328 426-byte APK. Made permanent as `[profile.android-dev]` (§1.2).
  The export shipped the real adaptive icons.
- Golden path driven by touch end to end — New world, Generate, Layers toggles,
  a settlement's Properties with the WHY HERE causal chain, the tool rail,
  sliders dragging by touch.
- **In-app Performance readout:** 60 FPS, Adreno 630, 8 CPU threads, static
  memory 52.26 MiB, video memory 42.73 MiB, and *"0 of 6 eligible stages ran on
  GPU — the whole pipeline ran on CPU, as configured"*. That was the
  configuration then. **Since, `engine_bridge.gd::_ready()` turns `use_gpu` on
  at boot with no Android gate**, and `wgpu`, `wgpu-hal` and `ash` are compiled
  into the arm64 `.so` (`cartalith-godot`'s `install_logger()` doc comment), so
  "Android runs CPU-only" is not a standing fact; read the backend from logcat.
- Memory and timing: §3.9. Generation ran on a background `Thread`, so the UI
  held 60 FPS through the 31-second default-size run. That 31 s of silent work
  is the "no progress indication" the staged generation readout was built to
  answer (`GenerationProgress`, `EngineBridge.generation_stage`).
- **Non-square maps all correct on device:** 512×512 1:1 (800 × 800 km),
  512×256 Whole world 2:1 (800 × 400 km, aspect pinned and its control disabled
  with a note), 512×910 9:16 (800 × 1422 km), 2048×1311 1.5625:1 (800 × 512 km),
  each aspect-fitted with the plate border intact.
- **The desktop shell on a phone was structurally intact and physically
  unusable by finger.** Orientation-locked to landscape, it got a 2340×1080
  surface and absolute pixel sizes on a ~403 dpi panel; at the landscape density
  (314 dpi, scale 1.9625) Android's 48 dp target is 94 physical px, and the
  chrome reached 13–47 % of it — the slider grabber ~12 px (0.76 mm), the tool
  rail 44 px wide on a 35 px pitch, popup rows ~22 px. This verdict was
  **retired 2026-08-20** when the phone composition first ran on glass (§3.3);
  the full table is in `986fb0c`.

### 3.3 Third pass (2026-08-20) — current code on glass, the phone composition's first run

- **Two config defects fixed before building:** a botched, uncommitted edit had
  collapsed `project.godot`'s `##` comment block and the orientation key into
  one garbage key (restored with `git checkout`; §1.4); and
  `android.debug.arm64` still read `…/debug/`, which the `android-dev` profile
  never writes — repointed (`d09b2d5`). `Cargo.toml`'s own usage line had
  `--profile` before `build`; corrected in place.
- `.so` 156 553 640 bytes; APK 207 106 507 bytes; the APK read back and
  confirmed to carry `6a97911`'s shell scripts.
- Blocked by the lock screen again (a fingerprint prompt), then unlocked by the
  owner; `libcartalith_godot.so` mapped `r-xp` into the live process; a 2048×1311
  world (seed 311447, app bar `ELDRA · 311447`) generated and rendered.
- **The `6a97911` GL-context fix was reported as holding on Android** on the
  strength of a clean GL ES 3.2 context, a full generation and an empty
  `grep -i wgpu`. **The last of those was not evidence**: no crate had
  registered a `log` backend, so no `wgpu` line could have appeared (§2.3).
- **The phone composition ran on a phone for the first time.**
  `_compute_layout_mode()`'s aspect test (`min/max = 0.4615 < _PHONE_ASPECT_MAX =
  0.6`) latched `_phone` in landscape; `_phone_scale` was `1080 / 393 = 2.75`
  (`PHONE_REF_SHORT` has since moved to 412, so the 6T now runs at 2.62). Two
  defects reported, not fixed: runtime dialogs kept desktop sizing, and *Open
  project* drew two headers and two close buttons — both fixed by the fourth
  pass. The pass also concluded portrait was unreachable over `adb`; the
  fourth pass found the real cause.

### 3.4 Fourth pass (2026-08-20) — the four owner-reported defects

Owner, on the `a80a386` APK: *"it doesnt switch to portrait mode"*, *"the open
project menu doesnt follow the design"*, *"make sure the lightmode version is
available everywhere"*, *"the bottom menu butons on phone are near too small to
use"*. All four fixed and verified on the device (`c33ccb6`).

1. **Portrait: the setting was a string, and Godot 4 wanted an integer.**
   `dumpsys window` showed the activity requesting `SCREEN_ORIENTATION_LANDSCAPE`
   hard: `ProjectSettings.get_setting()` returned `0` for `"sensor"` and `6` for
   the int. While fixing it, the `##` block above the key turned out to have
   swallowed it — `get_section_keys("display")` returned one key, the whole
   comment paragraph with the real key name on its tail, surviving only because
   that paragraph happened to hold an even number of quotes. Rewritten with `;`
   comments and a warning; manifest `screenOrientation` 0 → 13 (§1.4).
2. **Open project: two headers and a desktop dialog.** `borderless = true` drops
   the host window's title bar (wrong on every platform, desktop included), and
   `_present()` gives the phone a full-screen presentation with
   `content_scale_factor` set to `_phone_scale` — not a second set of phone
   constants: at 2.75 the layout area was 1080 / 2.75 = 393 px, the phone
   reference by construction. The ~420 px unwrapped subtitle is hidden on phone,
   and `_fit_columns` picks 1 tile column in portrait, 3 in landscape.
3. **Light theme: three gaps.** (a) `rebuild_theme()`'s hand-maintained
   override-name arrays had drifted six names behind `dcc_widgets.gd`'s text
   fields (`caret_color`, `font_placeholder_color`, `font_uneditable_color` and
   the `disabled` / `focus` / `read_only` styleboxes). (b) **The project-wide
   `dark_theme.tres` was never touched** — the fallback for every state nothing
   overrides, and a `Resource`, so no tree walk could reach it;
   `_recolor_project_theme()` remaps it in memory, with `_theme_extras` for the
   six colours that are not tokens. (c) Embedded `Window` chrome came from
   Godot's built-in theme; `_style_window_chrome()` writes it from tokens at
   `_ready()` and on every rebuild. Verified after a live dark → light *switch*
   (a cold boot would pass trivially) across all eight windows then shipped.
4. **Bottom-sheet buttons: the chrome was scaled, the contents never were.** The
   previous pass's "44 px targets land at ~121 physical px" was correct
   arithmetic about the chrome (`_ptap()`); the workspaces' own tool rows used
   desktop constants (`cartography_workspace.gd` set a literal `Vector2(34, 20)`
   — ~1.6 mm). Fixed once, at `DccShell.set_tool_options()`, the choke point
   every workspace's row passes through (`phone_fit()` there).
5. **The overflow menu — diagnosed only, by instruction.** It reparented the
   real desktop menu bar into a phone sheet: raw desktop sizes (~12 physical px
   rows), desktop status chrome eating the sheet, no response to a finger, and
   15 hover-opened submenus. Superseded: `phone_menu.gd` re-presents the menus as
   touch-sized screens (its header names these four faults), and the owner's
   2026-09-05 ruling rebuilt it as five bespoke screens from `06-phone.md` §6.6.

### 3.5 Pinch-to-zoom pass (2026-08-24)

Owner: *"zooming doesn't seem to work on the phone."* `viewport_host.gd` had
handled `InputEventMagnifyGesture` since the camera was written; the events never
arrived, because `enable_pan_and_scale_gestures` defaults to `false` (§1.4).
Read out of the shipped APK with `dexdump -d`:
`GodotGestureHandler.onScale` / `.onScaleBegin` open by testing
`panningAndScalingEnabled` and branch straight out, and
`enablePanningAndScalingGestures(Z)` is its only writer; `ScaleGestureDetector`
is built with the 2-arg constructor and `setQuickScaleEnabled` is **never**
called, so Android's single-finger double-tap-drag zoom does not exist in a
Godot app — two real fingers are the only path. Driven with the `uinput` recipe
(§2.2) on a 2048 × 1311 world, read off the app's `z%.1f` readout:

| Build | Gesture | Readout |
|---|---|---|
| fix on | pinch out 600 → 1000 px | **z1.0 → z2.2** |
| fix on | pinch in 1000 → 600 px | **z2.2 → z1.0** |
| control APK, setting `false`, otherwise identical | the same injected pinch | **z1.0, unchanged** |

The control reproduces the owner's report, so the setting is the cause. Deep-zoom
LOD tiles resolved in the zoomed capture, so `_zoom_at` → `set_camera_zoom` →
`_update_lod` runs on touch.

### 3.6 Civ / urban / render windows pass (2026-08-24)

GDScript only, driven on the existing `android-dev` `.so`: press-and-hold on a
settlement and on open water (the two branches of one context sheet), the place
editor, the City Viewer and its picker, the faction roster, the CARTO left sheet.
**The finding: no GUI input had ever reached `map_overlay.gd` on a phone**
(`GUI_GAP_REGISTER.md` §22, PH-01). Found by moving to a `--force-touch` desktop
run at 393 × 852 and asking `gui_get_hovered_control()` what was under the map
centre — the chrome spacer. That loop (§2.2) is the reusable result. One
observation from the drive, that the left panel sheet kept its scroll offset
across close/reopen and would not scroll back up, did not reproduce in the shell
at either density (`_sheetback_probe.gd`); it stands unconfirmed on glass.

### 3.7 Staleness pass (2026-08-24) — the APK was a day behind, and nothing said so

An audit sha256-compared the APK's `libcartalith_godot.so` with the build on
disk: **byte-identical to a 2026-08-23 14:34 build, with 25 commits landed in
`crates/` since** (`git log`; the commit title says 21). Through two clean
device passes, whole subsystems were inert — the NPR panel, Measure's Area /
Radius / Cross-section, the faction roster, the City Viewer, save, undo, the
erosion parameters, four debug views, GeoJSON export, hand-drawn ways,
civ-recompute — every one working code defeated by an older `.so`, behind 200
silent `has_method()` guards. Fixed by routing all 200 through `_has()` (§1.5),
building the release library for the first time since 2026-08-16 (21 577 640
bytes) and writing each entry's refresh command into `cartalith.gdextension`
(§1.2). The rebuilt debug `.so` (161 004 536 bytes) was confirmed inside the APK
by sha256 (`610125e8…51e7751`). On device: the NPR panel built and applied live,
the erosion-pass parameters and the icon bindings came back, the stage table
resolved — and then the phone dropped off USB, so the remaining items (paint
visibility, save/undo, the debug views, GeoJSON export, hand-drawn ways,
civ-recompute) were recorded as unverified on device rather than as verified;
`OUTSTANDING_WORK.md` routes that row here.

### 3.8 2026-08-25 pass — §46, §47, §48 and the LOD work, first on glass

LineageOS 22.2 / Android 15, physical density 450 with an override of 314,
`_phone_scale` 2.748. `.so` 171 644 632 bytes, verified byte-identical inside
the APK; `project.godot` md5 unchanged across all seven Godot invocations. Full
findings are `GUI_GAP_REGISTER.md` §50.

- **Frame time (SurfaceFlinger, §2.3):** deep-zoom panning median 16.7 / p99
  16.8 / max 16.9 ms, no frame over one vsync; a zoom notch median 16.7 / p99
  100.1 / max 117.0 ms, 4 frames over 33 ms — against
  `PERFORMANCE_BENCHMARKS.md` §5's pre-parallelisation 1.3–1.8 s frozen frame.
- **Generation 25.1 s cold, 24.8 / 25.8 s warm** at 2048×1311 — the first
  instrumented figure, read off the app's own Pass row; every earlier timing
  here was inferred from the shape of a memory trace.
- **Memory: peak 1 033 MB, steady 818 MB — do not quote as a regression.** No
  pass in this chain fixed the seed, and six clean runs of the identical
  procedure on the identical APK measured **869 / 902 / 916 / 937 / 963 / 1 029
  MB** steady — the whole apparent rise, from seed alone (`GUI_GAP_REGISTER.md`
  §52, `MEMORY_OPTIMIZATION_SCOPE.md`). The 2026-08-20 figure was also sampled
  every ~2 s against continuous sampling here. §52 names a likely real mechanism
  (canvas vertex buffers: 290.8 MiB across 311 237 drawn objects). A separate,
  dirty deep-zoom sample — 12 zoom-in notches from a fresh generate — reached
  556 MB of `Gfx dev` and 1 279 MB PSS in under a minute.
- A bare drag did not pan until the hand tool was armed; a 250 ms flick
  activated the row it started on (§50 PH-15, reproduced three times from the
  same coordinate before it was written down).
- Landscape was not captured; §2.1 has the `wm user-rotation` route.

### 3.9 Measurements, collected

**Memory, `TOTAL PSS`** (seed as stated by the pass; "—" where it was not):

| Date | Build | Grid | Seed | Peak | Steady | Sampling |
|---|---|---|---|---|---|---|
| 2026-08-17 | debug | 512×512 | 12345 | 283 326 KB | ~271 290 KB | ~1 s |
| 2026-08-18 | debug, stripped | 512×512 | — | 395 756 KB | 316 200 KB | ~0.17 s |
| 2026-08-18 | " | 512×256 (2:1) | — | 362 137 KB | 307 200 KB | " |
| 2026-08-18 | " | 512×910 (9:16) | — | ≥ 477 340 KB (peak fell between loops — a floor) | 333 950 KB | " |
| 2026-08-18 | " | 2048×1311 | — | 894 968 KB | 538 300 → 500 040 KB | " |
| 2026-08-18 | " | 512×512 after the 2048 world | — | — | 309 200 KB | " |
| 2026-08-20 | `android-dev` | 2048×1311 | 311447 | 899 089 KB | 662 793 KB | ~2 s |
| 2026-08-25 | `android-dev` | 2048×1311 | not fixed | 1 033 MB | 818 MB | continuous — see §3.8 |
| 2026-09-07 | release | 2048×1311 | fixed, stated in the row | 908 / 898 MB | 815 / 791 MB | ~0.42 s, host-polled |

The 2026-08-18 regenerate at 512×512 after the 2048 world settled *below* the
first 512×512 run, so the big world's memory is released — no leak; later passes
found every steady state flat to the kilobyte. The 2026-08-18 pass's
"+40 % peak / +17 % steady since 2026-08-17" rests on unstated seeds and is not
a supportable comparison. The 2026-09-07 row is `OUTSTANDING_WORK.md`'s archived
"Measure performance on a 2K map" row, which states its seeds, reproduces the
878 MB peak within 2.2 %, and finds steady state the half that moved.

**Generation time:**

| Date | Grid | Time | How |
|---|---|---|---|
| 2026-08-17 | 512×512 | ~7–9 s | inferred from the memory trace |
| 2026-08-18 | 131 k / 262 k / 466 k / 2.68 M cells | 3.2 / 4.5 / 8–9 / ~31 s | inferred |
| 2026-08-20 | 2048×1311 | ~16–18 s | inferred |
| 2026-08-25 | 2048×1311 | 25.1 s cold, 24.8 / 25.8 s warm | instrumented (the app's Pass row) |
| 2026-09-07 | 2048×1311 | 22.6–22.9 s `last_generate_ms` | instrumented; includes `absorb()` and a deferred frame — pipeline-to-pipeline against the desktop release build (2.17–2.37 s) is nearer 6–7× than 10× |

**Library and APK sizes:**

| Date | Artefact | Bytes |
|---|---|---|
| 2026-08-17 | release `.so` | ~14.5 MB |
| 2026-08-18 | debug `.so` → `llvm-strip --strip-debug` | 400 480 048 → 18 372 760 |
| 2026-08-18 | APK (stripped debug) | 68 328 426 |
| 2026-08-20 | `android-dev` `.so` / APK | 156 553 640 / 207 106 507 |
| 2026-08-24 | `android-dev` `.so` / release `.so` | 161 004 536 / 21 577 640 |
| 2026-08-25 | `android-dev` `.so` | 171 644 632 |
| 2026-09-07 | release `.so` / debug-keystore-signed release APK | 27 123 240 / 57 610 259 |

### 3.10 After 2026-08-25

Device work since then is recorded in `OUTSTANDING_WORK.md`, in rows routed to
this file — among them the `push_warning` control and the release-signed drop
(both 2026-09-07, §1.3 and §1.5), the 2K performance measurement (§3.9), the
zoom-notch stall and its stutter follow-up, boot-to-welcome time, and idle
redraw measured on a OnePlus 12. Read each there; this file records the method
they used, not their state.
