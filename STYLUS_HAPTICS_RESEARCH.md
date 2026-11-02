# Stylus and haptics on the OnePlus Pad 3, for Cartalith's Godot shell

**Agent-produced 2026-11-01** (finished in the small hours of 2026-11-02 UTC), at maximum depth, at the owner's
request. **Research, not a scope.** It has no milestones of record and no status column. §8 proposes a build plan for
the owner to schedule or not; how far anything has got is `cartalith-native/docs/STATUS.md`'s answer, never this file's.
A second research copy's supplement was checked claim by claim by a verifier and merged on 2026-11-02. What came from
it, and what was refused and why, is listed at the end (*Supplement merge, 2026-11-02*).

**The owner's words (2026-11-01):** *"For the gui, I'd like you also to look into the resolution of the OnePlus tab 3
and the functionality of the Stylus for the OnePlus tab3. I want that stylus functionality to be implemented. Maybe
there are other online sources (krita for example) and other applications/tablets that work and implement a stylus
well. I want this stylus that is accompanied with the OnePlus tan3 and it's haptics to he fully utilised. For terrain
coloring like that you could "feel" the relief of the terrain/mountains. Do a deep dive in the technical research and
available sources. Run a research agent at Max effort for this."*

**The owner's scope correction, the same day:** *"Let's be clear this research agent is eventually for the godot
version of Cartalith."* So the target is the Godot 4 shell and the Rust crates in `cartalith-native/`, built for
Android. The web (§5) is a short note, and the HTML working copy gets one paragraph (§5.3).

**How to read the tags.** Every claim carries one.
- **[verified: …]** — a primary source opened in this session: a vendor page, official documentation, source code,
  a paper's own text, or this repository's code at a named symbol.
- **[secondary: …]** — a review, forum post, news piece or another document in this repository that reports it.
- **[unconfirmed]** — a search excerpt, a retailer listing, or a page that could not be opened.
- **[estimate]** and **[judgement]** — this document's own arithmetic or design choice, labelled as such.
- A source whose own authors say it was written with an AI (a README disclosure, or AI co-author lines on its commits)
  is **not evidence**. It is named only to say so, and a claim resting on it alone is **[unconfirmed: AI-written]**.
  The same holds for a GitHub issue its author withdrew as AI-written (§4.3).

Source IDs (S1, A6, G1 …) resolve in the **Sources** list at the end. Code is named by symbol, not by line number,
because line numbers in this tree drift within a day (`MISTAKES.md`).

---

## Findings summary

1. **The device is almost certainly the OnePlus Pad 3** (model OPD2415: the prefix of its reported firmware build
   [secondary: S16], and the model LineageOS lists for it [verified: L1]). Its panel is 13.2 inches, 3392 × 2400
   pixels, 315 PPI, 7:5, LCD (LTPS), up to 144 Hz adaptive, touch sampling *"Up to 540 Hz / Default 120 Hz"*
   [verified: S1]. It launched on Android 15 with OxygenOS 15 [verified: S1]; OxygenOS 16 on Android 16 reached it
   from November 2025, India first [secondary: S16]. A newer **OnePlus Pad 4** (April 2026, *"currently limited to
   India"*) has a 13.2-inch *"3.4K"* 144 Hz panel and a different pen, the Stylo Pro [secondary: S15]; that the panel
   is the same 3392 × 2400 rests on search excerpts [unconfirmed: S13]. Which one the owner holds is owner question Q1.
2. **Its default display density is not published.** So its size in dp is a table, not a number (§1.3). Under Ruling
   DY's MQ-1 it is a **tablet in both orientations at any density below 640 dpi**. Whether its bar shows all eight menus
   depends on the density: landscape reaches 1 180 dp at or below 459 dpi, portrait only at or below 325 dpi
   [estimate: §1.3 arithmetic]. One `adb shell wm density` settles it.
3. **The pen is the OnePlus Stylo 2**, sold separately [verified: S2]. OnePlus publishes *"16,000 levels of pressure"*,
   *"Ultra-low latency"* and *"Haptic feedback"* [verified: S2], and says it *"replicates the familiar feel of writing on
   paper"* with *"subtle vibration feedback"* [verified: S3]. Reviewers report tilt, hover, magnetic charging and
   pairing on the tablet's edge, and **no barrel button**: a touch-sensitive double-tap area instead [secondary: S4,
   S5, S6, S8]. Pressure levels aside, OnePlus publishes no hover distance, report rate, latency figure or protocol.
   - **Hover is in range or out of range only.** LineageOS's copy of the Pad 3's touch driver gives the pen's distance
     axis a range of 0 to 1 [verified: L6].
   - **Whether the double-tap area sends apps a key is unknown.** A forum report has a OnePlus pen, not shown to be the
     Stylo 2, sending its side button to every app as `KEYCODE_F21` on ColorOS 16 [secondary: S20]. Godot would
     deliver such a key as `KEY_UNKNOWN` (§2.2, §4.2).
4. **The pen has its own vibration motor, and Cartalith cannot drive it.** Every review that tested which apps get the
   pen's haptics and gestures found them only in OnePlus's own apps (Notes) [secondary: S5, S6, S7]. A GitHub issue,
   citing an XDA thread, guesses that the pen is reached through an OPPO/OnePlus SDK class,
   `com.oplus.ocs.ipemanager.sdk.PencilManagerUnitClient`, that checks the calling app's signature; the same issue says
   an unpatched third-party notes app (StarNote) does get the haptics [unconfirmed: S12]. No public SDK was found.
   So the relief Cartalith can make felt today comes from **the tablet's own vibrator**, if it has one. OnePlus does
   not publish whether it does, or its type [verified: S1 lists none]; §8's SH-0 measures it.
   - **It may have none.** LineageOS's build for the Pad 3 ships no vibrator service, where the OnePlus 13 on the same
     common tree adds one [verified: L2, L4]. A device tree describes a ROM, not the hardware, and some signs point
     the other way (§1.4).
   - **The first SH-0 command settles it.** `adb shell cmd vibrator_manager list` prints *"No vibrator found"* when
     there is none [verified: A26]. Then no body-vibration cue in §7 can be felt on the Pad 3, and its relief is sound
     and sight (§7.7). The vibration design still serves the OnePlus 6T and any other device with a motor.
   - LineageOS's copy of the Pad 3's touch driver has an on/off command for the pen's vibration [verified: L6], but
     no ordinary app can reach it, and the stock pen service is restricted (§2.3).
5. **Every haptic the shell ships is silent on Android today.** `DccShell._haptic()` calls `Input.vibrate_handheld()`
   [verified: `shell/dcc_shell.gd::_haptic`]. It has 9 code call sites, using four of its six kinds (`sample`,
   `tool_arm`, `detent`, `back`), counted 2026-11-01 with `grep -rn '_haptic("' --include=*.gd
   cartalith-native/godot-project | grep -v _probe | grep -v ':[0-9]*:[[:space:]]*#' | wc -l`. Godot 4.7.2 documents that this *"requires enabling the VIBRATE
   permission in the export preset. Otherwise, vibrate_handheld will have no effect"* [verified: G2]. The Android preset
   never enables it: its only permission keys are `permissions/custom_permissions=PackedStringArray()` and
   `permissions/internet=true` [verified: `godot-project/export_presets.cfg`, `grep -n "permissions/"`]. Godot's
   `GodotNativeBridge.vibrate` catches the resulting `SecurityException` and logs *"VIBRATE permission not found"*
   [verified: G1]. This is read from the preset and the engine source; one `adb logcat` on the device confirms it
   (§8.2). `PORT_ONLY_FEATURES.md` lists *"An installable APK with haptic feedback"* as a port feature
   [verified: that file's Android row]; it is probably false today.
6. **What Godot 4.7.2 passes to scripts from an Android pen** [verified: G1, read at tag `4.7.2-stable`]:
   - The pen arrives as **mouse events** (`InputEventMouseButton`/`InputEventMouseMotion`), not touch events.
   - **Pressure** comes from `getPressure()`. **Tilt** is `sin(AXIS_TILT)` times `(−sin, cos)` of the orientation, a
     vector in −1..1.
   - **The eraser is never reported**: nothing on Android sets `pen_inverted`.
   - **Hover** arrives as a plain `InputEventMouseMotion`: position only. It has no pressure, no tilt, no distance, and
     enter, move and exit look the same.
   - **Stylus buttons are dropped**: a press with the side button held arrives as button index `NONE` with an empty
     mask. Issue #124191 reported this on 4.7.2, but its author withdrew it the same day as AI-written
     [secondary, withdrawn: G5]; the claim rests on the source.
   - **Batched samples are dropped**: only the newest point of each Android batch reaches Godot.
   - **The moves inside the touch slop are dropped** unless `input_devices/pointing/android/disable_scroll_deadzone`
     is on, because they never reach the engine: the first motion Godot sees is already past the slop, which is 8 dp
     by AOSP's default and can be changed by a device overlay [verified: A25].
   - **Godot then merges motion events once per frame**, and the merged event keeps the first sample's pressure and
     tilt. Cartalith never turns this off (`Input.use_accumulated_input` is not set anywhere in the shell).
   - **Keys above F12 arrive as `KEY_UNKNOWN`**, and so do the stylus key codes: Godot's Android key table stops at
     F12 [verified: G6].
   - **A multi-touch event gives every pointer pointer 0's tilt**, and the emulated mouse follows the first touch. So
     a palm that lands before the pen can steer the stroke, or turn it into a pan or a zoom, unless Android cancels
     the palm first (§4.2) [verified: G1, G6; Android's part: A30].
7. **Cartalith's own pen plumbing is real and partly built.**
   - `map_overlay.gd` emits `map_stroke_sample` with pressure and tilt, or `null` when no pen is detected.
   - Pressure scales the sculpt stamp's strength and WORLD paint's dab size (`WorldGen.sculpt_add_point_p`,
     `paint_stroke_sample`; dab size = `pressure_min` 0.2 + 0.8 × p).
   - Two Pressure toggles exist.
   - Tilt reaches the paint engine, but no option turns its tilt dynamic on.
   - **Colour paint ignores pressure.**
   - **A brush ring already follows the cursor** for sculpt, WORLD paint, colour paint and territory
     (`shell/tool_overlay.gd::set_brush_cursor`, fed by `cursor_sampled`, which `map_overlay.gd::_gui_input` emits for
     every mouse motion, button held or not). Godot delivers pen hover as mouse motion (point 6), so the ring should
     already follow a hovering pen; that is read from the code, not seen on a device. It never hides when the pen
     leaves, and draws no `pressure_min` ring.
   - **Missing:** a pen-only mode, a palm rule and any stylus setting (all [verified: `shell/menus.gd`'s Stylus
     pressure / Palm rejection note; `dcc_settings.gd`, grep]).
8. **Two routes reach Android's full haptic API without a Gradle build.** GDScript can use Godot's built-in
   `AndroidRuntime` singleton with `JavaClassWrapper`. The shell already does this once, in
   `shell/llm_net_hint.gd::open_app_settings`. Rust can use raw JNI, which `llm_keystore_android.rs` already does.
   Either reaches `Vibrator`, `VibrationEffect.Composition` (API 30/31), the envelope builders (API 36) and
   `performHapticFeedback` (API 3; it needs no permission) [verified: A6–A12, G3]. **Intercepting the input Godot
   drops is different.** That needs Java code listening on the UI thread: a Kotlin plugin, which needs the Gradle build
   the project does not use [verified: G4; `ANDROID_BUILD_SCOPE.md` §1.3], or a fix upstream in Godot. A
   `JavaClassWrapper.create_proxy` listener would run GDScript on the UI thread, which §4.4 judges unsafe [judgement].
9. **The limits that shape the design:**

   | Limit | Value | Source |
   |---|---|---|
   | Touch-feedback latency | keep it between 5 and 50 ms; quality drops between 70 and 100 ms | [verified: R1] |
   | Gap between two primitives | at least 50 ms to be felt as two | [verified: A17] |
   | Intensity steps | differ by a ratio of 1.4 or more, e.g. 0.5, 0.7 and 1.0 | [verified: A17] |
   | Pulse trains on a 10-inch tablet's motor | 2.5–10 Hz read as dashed or dotted, 25–50 Hz as solid (the commonest answer, 41.58 %), 100 Hz and above not as a line at all (61.75 %) | [verified: R2] |
   | Primitive durations | TICK 5 ms; CLICK and LOW_TICK 12 ms (targets) | [verified: A23] |
   | Ringing | an LRA rings 20–50 ms after a 20 ms input | [verified: A16] |
   | Touch latency on this tablet, one review | 41.9 ms *"total touch latency"* with the WALT Latency Timer; finger or pen, refresh rate and sample count not stated. If the pen's latency is similar, it uses most of the 50 ms budget (§7.9) | [secondary: S17]; the consequence [judgement] |

10. **A concrete relief design** (§7). It is built on the evidence that texture vibration frequency is scan speed over
    spatial period, and that, for fine textures, roughness tracks vibration power [verified: R3]:
    - **Contours:** tick on the same contour lines the renderer draws, at its own interval
      (`render.rs::apply_npr`).
    - **Index lines** are stronger by the renderer's own factor of 1.4.
    - **Slope** sets the strength, in three steps.
    - **Ruggedness** (`cartalith_terrain::analysis::ruggedness`) sets a texture. As implemented it also rises with
      slope (§7.4's note).
    - **Ridges and valleys** pulse where the height under the pen stops rising or stops falling.
    - **Coast and water** get their own cue, and open water is quiet.
    - **A scheduler** keeps 50 ms gaps and switches to texture when crossings come too fast. A fallback ladder covers
      devices without primitives or amplitude control.
    - **Without a body vibrator** the ladder's last rung is the whole relief on that device: an optional sound per
      contour and a brightened line (§7.7). Which rung the Pad 3 is on is SH-0's first measurement.
11. **A build plan for the Godot shell** (§8): thirteen milestones, SH-0 to SH-12, of which the last three are
    optional or blocked. Each says whether it is a project setting, GDScript only, a Rust bridge change, or an Android
    plugin. SH-0 (measure the device) and SH-1 (turn on the VIBRATE permission) come first and are small. SH-0's first
    command decides how much of §7 the Pad 3 can use (§8.1, *If SH-0 finds no vibrator on the Pad 3*).

---

## 1. The device

### 1.1 Which tablet "OnePlus tab 3" means

- **OnePlus Pad 3** is the product the words most likely mean. OnePlus sells it under that name, with a specification
  page and a product page [verified: S1, S2]. Retailers give its model number as OPD2415 [unconfirmed: S13], LineageOS's
  device list gives the same [verified: L1], and the firmware build reported for its Android 16 update is
  `OPD2415_16.0.0.211 (EX01)` [secondary: S16].
- **Other tablets the words could mean:**
  - **OnePlus Pad 4** (launched 30 April 2026, *"currently limited to India"*): a 13.2-inch *"3.4K"* panel at 144 Hz;
    its pen is the **OnePlus Stylo Pro**, with *"up to 16,000 pressure points"* and *"swappable tips"*
    [secondary: S15]. That the panel is the Pad 3's 3392 × 2400, and the phrase "dual tips", come from search excerpts
    [unconfirmed: S13].
  - **OnePlus Pad 2** (2024) and **OnePlus Pad Lite**.
  - **OnePlus Pad 2 Pro** (OPD2413), launched in China in May 2025, with a pen sold as *"the OnePlus Smart Stylus Pen
    Pro"* [secondary: S19]. LineageOS builds it and the Pad 3 from one device tree, `erhai` [verified: L1]. So a report
    about the Pad 2 Pro may hold for the Pad 3's hardware, but not necessarily for its pen or its software [judgement].
- Everything below assumes the Pad 3 with the Stylo 2. If the owner holds a Pad 4, §1.2–§1.3 hold for its panel, but
  §2 is about the wrong pen (Q1).

### 1.2 Display and system

| Property | Value | Source |
|---|---|---|
| Size | *"13.2 in, 33.53cm"* | [verified: S1] |
| Resolution | *"3392\*2400"* | [verified: S1] |
| Aspect | *"7:5 Ratio"* (3392 ÷ 2400 = 1.413) | [verified: S1]; ratio [estimate] |
| Pixel density | *"PPI 315"* (pixel pitch 25.4 ÷ 315 = 0.0806 mm; active area 273.5 × 193.5 mm) | [verified: S1]; mm [estimate] |
| Panel | *"LCD (LTPS)"* | [verified: S1] |
| Colour depth | *"12bit (8bit hardware +4bit FRC)"*; footnote: *"this display is not native 12-bit hardware"* | [verified: S1, S2] |
| Colour gamut | *"96% NTSC / DCI-P3 coverage 98% / DeltaE≈0.7"* | [verified: S1] |
| Refresh | *"Up to 144 Hz, adaptive refresh"*; footnote: *"only available on compatible and whitelisted apps"* | [verified: S1, S2] |
| Touch sampling | *"Up to 540 Hz / Default 120 Hz"* | [verified: S1] |
| Touch controller | Novatek NT36536: the touch firmware files are `FW_NT36536_HUAXING.img` and two variants | [verified: L2, `proprietary-files.txt`; L6 holds the driver] |
| Panel mounting | natively portrait, mounted landscape: `ro.surface_flinger.primary_display_orientation=ORIENTATION_270`. Apps see rotated coordinates; raw `getevent` coordinates (§8.2) are the panel's own [judgement] | [verified: L2, `properties/vendor.prop`] |
| Refresh while a pen is connected | LineageOS's own pen service caps the display at 120 Hz while a Bluetooth pen with an OPPO or Maxeye vendor id is connected (`config_penSupportedRefreshRate` = 120.0, commented *"Max supported refresh rate when using pen"*). Whether stock OxygenOS does the same is not established | [verified: L2, L7 `PenService.updateRefreshRateCap`]; stock [unconfirmed] |
| Brightness | *"600nits (HBM 900 nits)"* | [verified: S1] |
| Chipset | Snapdragon 8 Elite, Adreno 830 | [verified: S1] |
| Battery | *"12.140mAh"* on the spec page; *"about 12,140mAh (typical)"* in the product page's footnote | [verified: S1, S2] |
| Weight, thickness | 675 g, 5.97 mm | [verified: S1] |
| OS at launch | *"Android 15 / Oxygen OS 15"* | [verified: S1] |
| OS now | OxygenOS 16 on Android 16, rolling out from November 2025 (build `OPD2415_16.0.0.211 (EX01)`, India first); a rollout pause after boot-loop reports was mentioned in one source | [secondary: S16]; the pause [unconfirmed: S13] |
| Haptic motor | **not published**: the specification table has no haptics or vibration line. LineageOS's build for it ships no vibrator service; whether the hardware has a motor is open (§1.4) | [verified: S1, every group printed; L2] |
| Amplitude control, composition primitives | **not published**; measure with `dumpsys vibrator_manager` (§8.2) | — |

**What reviews report.** These are other people's figures, cited, not measured here:
- AndroPlus: *"The total touch latency measured with the WALT Latency Timer was 41.9ms"*, and *"the Movement Rate was
  around 290Hz on average for both single and multi-touch"*, a different metric from the 540 Hz figure. Refresh *"can
  be set for individual apps, with choices of 90Hz, 120Hz, and 144Hz"*. The review does not say whether a finger or the
  pen was used, at what refresh rate, or over how many samples [secondary: S17]. The page itself was opened, so the
  290 Hz figure no longer rests on a search excerpt.
- Notebookcheck: the touchscreen *"is scanned for touches up to 120 times per second"*, which matches the
  specification's default, and the maximum brightness is *"just under 700 cd/m²"* [secondary: S18].
- Parka Blogs: *"the 144Hz refresh rate actually does not improve or affect the latency"* of the pen [secondary: S4].

**How Godot renders it.** `project.godot` sets no stretch mode and `window/handheld/orientation=6` (sensor)
[verified: `project.godot`, `[display]`]. So the Godot viewport is the full panel, 3392 × 2400 physical pixels
(8.1 MP), in either orientation, under `screen/immersive_mode=true` [verified: `export_presets.cfg`].

### 1.3 Density, dp size and Cartalith's layout class

The default density (`ro.sf.lcd_density`, read by `adb shell wm density`) **is not published anywhere this session
could find** [unconfirmed: searches for OPD2415 and Pad 3 density returned nothing]. The arithmetic, for the densities
Android devices commonly use (dp = px × 160 ÷ density) [estimate]:

| Density (dpi) | Scale | Landscape (dp) | Portrait (dp) |
|---|---|---|---|
| 315 (the panel's physical PPI) | 1.97 | 1 722.9 × 1 219.0 | 1 219.0 × 1 722.9 |
| 320 | 2.00 | 1 696.0 × 1 200.0 | 1 200.0 × 1 696.0 |
| 360 | 2.25 | 1 507.6 × 1 066.7 | 1 066.7 × 1 507.6 |
| 400 | 2.50 | 1 356.8 × 960.0 | 960.0 × 1 356.8 |
| 420 | 2.625 | 1 292.2 × 914.3 | 914.3 × 1 292.2 |
| 440 | 2.75 | 1 233.5 × 872.7 | 872.7 × 1 233.5 |
| 480 | 3.00 | 1 130.7 × 800.0 | 800.0 × 1 130.7 |

**Which layout class it gets** [estimate, from the rules as written]:
- **Ruling DY, MQ-1** (*"under 600 dp wide is a phone; under 500 dp tall in landscape is a phone landscape; everything
  else is a tablet"*, `LARGE_ITEM_RULINGS.md`): the portrait width, 2 400 px, falls below 600 dp only above 640 dpi.
  The landscape height falls below 500 dp only above 768 dpi. **So it is a tablet, in both orientations, at any
  realistic density.**
- **The shipped classifier** (`shell/dcc_shell.gd::_compute_layout_mode`) calls a screen a phone only when
  `short ÷ long < _PHONE_ASPECT_MAX` (0.6). The Pad 3's ratio is 2400 ÷ 3392 = 0.708, so it is a tablet there too
  [verified: the constant and the function].
- **The 1 180 dp narrow-bar line** (`MOBILE_SHELL_PROPOSAL.md` §2.2):
  - Landscape is 1 180 dp or wider at or below 459.9 dpi, so it shows the eight menus whole.
  - Portrait is that wide only at or below 325.4 dpi. Above that it shows `☰ · File · World · Records · Story`.
- **The width bands of `MOBILE_SHELL_PROPOSAL.md` §3:**
  - Landscape is *large* (1 200–1 599) for densities from about 340 to 452 dpi.
  - Portrait is *expanded* (840–1 199) for densities from about 320 to 457 dpi.
- **The 520 dp docking rule:** in landscape at 400 dpi, a 1 356.8 dp window minus the two docks (340 + 312) and a
  64 dp rail leaves about 640 dp of map. That is above 520, so both docks can stay docked [estimate; the rail width
  for the large band is not stated in §3].

### 1.4 Does the Pad 3 have a vibration motor?

OnePlus does not say (§1.2). One command settles it: `adb shell cmd vibrator_manager list` prints one id per vibrator,
or *"No vibrator found"* when there is none [verified: A26, `runListVibrators`, the same on AOSP `main`,
`android15-release` and `android16-release`]. Until it is run, the evidence is circumstantial and points both ways.

**What LineageOS's device trees show** [verified: L1–L5; a case-insensitive grep over each whole tree for "vibrat",
"haptic", "richtap" and "awinic"]:
- LineageOS's build for the Pad 3 (`erhai`, which also serves the Chinese Pad 2 Pro) ships no vibrator service, no
  vibrator files and no vibrator configuration. The grep's one hit, `odm/lib64/libvibrationmgr.so`, sits in
  `proprietary-files.txt`'s `# Camera` block.
- The Pad 2's tree (`caihong`) has no hit at all, on either branch [verified: L3].
- The OnePlus 13 (`dodge`) inherits the same `sm8750-common` tree as the Pad 3, and adds its vibrator service in its own
  `device.mk` (`# Vibrator`, `vendor.qti.hardware.vibrator.service.oplus`). So the Pad 3's lack of one is a choice made
  per device, not something the common tree left out.
- On the older chip family, `sm8650-common` wraps its vibrator block in `ifneq ($(TARGET_IS_TABLET),true)`. That block
  only copies `excluded-input-devices.xml`; the vibrator service itself is added per device, as in the OnePlus 12's
  `device.mk`. The same guard sits on thirteen other blocks: Context Hub, Doze, fingerprint, GPS, IPACM, IR, NFC,
  overlays, SecureElement, the proximity sensor, telephony, LineageOS's touch service and the network manifest. Some
  are hardware a Wi-Fi tablet lacks; some are not [verified: L5, the heading above each guard].

**What they cannot show** [judgement]:
- A device tree says what a ROM builds, not what the hardware holds. LineageOS's device-support requirements name no
  vibrator [verified: L8, grep for "vibrat" and "haptic" finds nothing], so an official build may leave a working
  motor unused without breaking a rule.
- They say nothing about stock OxygenOS, which the owner runs.

**Signs pointing the other way:**
- The `sm8750-common` tree that the Pad 3 inherits loads `qcom-hv-haptics.ko` and `oplus_haptic.ko` [verified: L5,
  `modules.load`]. That tree also serves the OnePlus 13, so this shows the kernel modules are there, not a motor.
- An AI-written project's `getevent -lp` dump of the Pad 2 lists a `qcom-hv-haptics` force-feedback device
  [unconfirmed: AI-written, S21].
- A PhoneArena search excerpt says the Pad 2's haptic feedback *"wasn't a highlight"* [unconfirmed: search excerpt; the
  page returned HTTP 403].

**What follows** [judgement]. The design keeps both branches open until SH-0 runs. If the list prints *"No vibrator
found"*, no body-vibration cue in §7 can be felt on the Pad 3, whatever the permission: `Input.vibrate_handheld()` has
nothing to drive. The Pad 3's relief is then sound and sight (§7.7, rung 4). The pen's own motor is the only actuator
left, and no ordinary app can reach it (§2.3; Q10, Q13). The vibration design still serves the OnePlus 6T and any
other device with a motor (§8.1).

---

## 2. The stylus

### 2.1 Identity, pairing and charging

| Property | Finding | Tag |
|---|---|---|
| Name | **OnePlus Stylo 2**, listed as the Pad 3's pen, *"sold separately"* | [verified: S2] |
| Model number | OPN2402 (some listings give OPK2402) | [unconfirmed: retailer listings, S13] |
| Price | USD 99; HotHardware lists *"OnePlus Stylo 2 ($100)"* | [secondary: S4, S5, S9; S8] |
| Same pen as the Pad 2's | *"The Stylo 2 is identical to the one that shipped with last year's OnePlus Pad 2"* | [secondary: S8] |
| Charging and pairing | *"The pen can attach to the side of the tablet for charging and Bluetooth pairing"*; *"charges by attaching magnetically to the top edge of the tablet"*. LineageOS's Pad 3 files include a wireless-pen charger's firmware, `cps8601_firmware.bin` | [secondary: S4; S7]; [verified: L2] |
| Battery | *"almost always full"* while attached; no figure published | [secondary: S4] |
| Protocol | **not published.** Android Central found that USI pens *"that work with the Pixel Tablet and Chromebooks"* did not work on the Pad 2, and that the Stylo 2 was the only pen that did. LineageOS's own pen service talks to OnePlus pens through a vendor Bluetooth LE service, and finds a connected one among Android's input devices by an OPPO (0x22D9) or Maxeye (0x330A) vendor id; so on that build the pen is also a Bluetooth input device | [secondary: S10]; [verified: L7, `PenService`, `PenProtocol`] |
| Works on other tablets | OnePlus pads only, per listings. Parka Blogs: *"This tablet is actually a rebrand of the Oppo Pad 4 Pro, and the OnePlus Stylo 2 pen is a rebrand of the Oppo Pencil 2 Pro"* | [unconfirmed: S13]; [secondary: S4] |

### 2.2 Capabilities

| Capability | Finding | Tag |
|---|---|---|
| Pressure | *"16,000 levels of pressure"* (up from 4 096 on the first Stylo). LineageOS's copy of the Pad 3's touch driver reports pressure, tilt and hover from the touch controller. A LineageOS commit says newer OnePlus pens (it names the Stylo Pro) also need data relayed from the pen's Bluetooth service to the touch driver; that matters if the owner holds a Pad 4 (Q1) | [verified: S2, S3]; [secondary: S11]; the driver [verified: L6]; the relay [unconfirmed: L7, commit `6b36209`, marked `Assisted-by: LLM`] |
| Pressure quality | *"Initial activation force of the pen is low enough"*; *"the pen has difficulty detecting minimal changes in pressure when drawing with minimal pressure"*; *"There's a tendency to draw thicker lines"* | [secondary: S4] |
| Pressure curve | OnePlus offers no pressure adjustment, *"so you'll have to rely on your drawing app"* | [secondary: S5] |
| Tilt | supported: *"detects tilt"*; works in Krita and Concepts on the Pad 2, but *"does not work consistently with Infinite Painter and Concepts"* on the Pad 3 | [secondary: S6; S5; S4] |
| Hover | *"There's still the hover feature though"*. **In range or out of range only**: LineageOS's copy of the touch driver gives the pen's `ABS_DISTANCE` a range of 0 to 1, and reports the pen tool while it touches or is in range. No hover height is published | [secondary: S5]; [verified: L6, `touchpanel_common_driver.c`] |
| Report rate | "240 Hz Sampling Frequency" appears in a retailer's product title | [unconfirmed: S13] |
| Latency | *"Ultra-low latency"* (OnePlus, no number); 0 ms and 2 ms figures appear only in listings. A reviewer: *"There's still noticeable latency"*, *"the amount of latency will vary depending on the app"* | [verified: S2]; [unconfirmed: S13]; [secondary: S4] |
| Buttons | **no physical barrel button.** *"double tap to switch tools (instead of a button)"*; *"a touch-sensitive area on the side of the stylus to switch tools"*. Whether that area sends apps a key is open (below the table) | [secondary: S8, S6] |
| Gestures | *"Page turn"*, *"Laser pointer"*, *"smooth touch control"* for presentations | [verified: S3] |
| Eraser end | **none found.** No source read mentions one. Reviewers describe double-tap tool switching instead; that it switches to the eraser in Notes comes from forum search excerpts | [secondary: S8, S6]; [unconfirmed: S14] |
| Palm rejection | *"supports … palm rejection"*, but *"Palm rejection does not work well"* (Pad 2) and *"occasional issues with palm rejection. Resting my palm on the display may activate double-tap undo"* (Pad 3) | [secondary: S5, S4] |
| Line quality | *"slight wobble or jitter with diagonal lines that are drawn slowly"* | [secondary: S4] |

**The double-tap area and keys.**
- A Krita Artists thread (started 2026-03-25) reports a pen named only *"OnePlus Stylo"* on a *"OnePlus Pad 2 Pro (or
  OnePlus Pad 3, they are the same)"* under *"ColorOS 16 based on Android 16.0"*. Android's Switch Control showed its
  side button as *"KEYCODE_F21"*, and the button can be held (*"It holds."*) [secondary: S20]. The forum closed the
  thread automatically on 2026-03-31; its author did not withdraw it.
- **That pen is not shown to be the Stylo 2.** The Pad 2 Pro launched with *"the OnePlus Smart Stylus Pen Pro"*
  [secondary: S19], and the Stylo 2 has a touch area, not a button that is held [secondary: S8, S6].
- Two painting apps changed for that report. Drawpile ships a setting, *"Translate F21 key to middle click (OnePlus)"*
  [verified: K14]. Krita's Qt fork *"adds handling for F13 to F24 keys"* and *"a flag to translate F21 to middle
  clicks"*, and Krita turned the translation on unconditionally; both changes merged on 2026-03-27 [verified: K15].
- An AI-written reverse-engineering project says that on a Pad 2 the Stylo 2 sends `KEY_F20` for a double tap only in
  its presentation mode, and in normal mode only a Bluetooth notification [unconfirmed: AI-written, S21].
- **So whether the Stylo 2's double tap produces any key event on the Pad 3 is unknown.** SH-0 measures it with
  `getevent -lt`, with the pen's presentation mode off and then on (§8.2). Godot would deliver such a key as
  `KEY_UNKNOWN` (§4.2).

### 2.3 Its haptics, and who can use them

- **The pen has its own motor.** OnePlus: *"Near-instant response and subtle vibration feedback elevate your
  experience"* [verified: S3]. A news report: *"a linear motor that makes the tip vibrate"* [secondary: S11].
  - In Notes, *"Every stroke of the stylus has haptic feedback and a pencil on paper scratching noise from the
    tablet's speakers"* [secondary: S7].
- **Reviewers found it only in OnePlus's own apps.** Each of these reviews says so:
  - *"These pen features only work with OnePlus apps and do not work with third party apps"* [secondary: S5].
  - *"The touch gestures and vibrations also only work on the preinstalled Notes app"* [secondary: S6].
  - *"I only noticed this in the OnePlus notes app, and not in other apps"* [secondary: S7].
- **How the restriction might work.** A GitHub issue about a patched third-party notes app, StarNote (dated
  2026-09-26), cites an XDA thread. Its author writes that a class
  `com.oplus.ocs.ipemanager.sdk.PencilManagerUnitClient` *"checks the signature of the apps before communicating with
  the stylus"*, and calls this speculation [unconfirmed: S12]. The XDA thread itself refused to load (HTTP 403).
  - The same report says that the unpatched app gets the haptics, so some third-party apps are allowed. OnePlus may
    whitelist partners. That is not established.
- **How the pen's motor is driven.** Two leads. Neither is open to an ordinary app [judgement]:
  - **A tablet-side switch.** LineageOS's copy of the Pad 3's touch driver has an on/off command for the pen's
    vibration: `nvt_enable_pen_vibrator` sends `EVENTBUFFER_EXT_PEN_VIBRATOR_ON` (0x16) for the value 3 and `_OFF`
    (0x17) for 2, under `MODE_PEN_CTL` [verified: L6, `NT36536_noflash`]. It is a kernel driver path, not an app
    interface.
  - **A Bluetooth setting.** An AI-written project says the pen makes its writing texture itself once a texture type
    and strength are set over Bluetooth; that the pen forgets the setting on every Bluetooth disconnect; that a
    separate command plays one short pulse; and that writing the touch-driver command above had *"no effect we could
    feel"* on a Pad 2 [unconfirmed: AI-written, S21]. It also says the stock pen service, *"Smart accessories"*
    (`com.oplus.ipemanager`), guards its haptic interface with a OnePlus permission, which fits S12 and the reviews
    above [unconfirmed: AI-written, S21]. Only the Bluetooth command channel appears in code outside that project:
    LineageOS's pen service uses the same vendor service and characteristic (`18092dbc-…`, `18093046-…`)
    [verified: L7 code; its commits are marked `Assisted-by: LLM`].
  - **A direct Bluetooth route** from Cartalith would need `BLUETOOTH_CONNECT`, a *"dangerous"* runtime permission
    since API level 31 [verified: A28]. The same AI-written account warns that the pen's firmware-update services sit
    beside the command channel [unconfirmed: AI-written, S21], and such a route could clash with the system's own
    connection to the pen [judgement]. Q13 asks the owner; the default is no.
  - Even if a route opened, that account offers a fixed writing texture and one-shot pulses. A texture that follows
    the terrain would be one Bluetooth command per change, with unpublished latency [judgement, on an unconfirmed
    premise].
- **The Android input path itself offers nothing for this.** `InputDevice.getVibratorManager()` (API 31) exposes a
  vibrator on an input device [verified: A4]. Whether the pen registers one is unknown, but SH-0 tests it cheaply.
  Android's `cmd vibrator_manager list` prints device vibrators only: *"This does not include any connected input
  device"* [verified: A26].
- **Consequence for Cartalith.** Until OnePlus offers access (Q10), the relief has to come from the **tablet's own
  vibrator**. It is felt through the hand that holds or steadies the tablet, and through the drawing hand's palm resting
  on the glass. **The Pad 3 may have none** (§1.4). Whether it has one, and how strong it is, are SH-0's first two
  measurements. If it has none, the relief falls back to sound and sight (§7.7).

### 2.4 Platform comparison: what other pens put in apps' hands

| Pen | In-pen haptics for third-party apps | Hover | Buttons and gestures for apps | Source |
|---|---|---|---|---|
| OnePlus Stylo 2 | no public API found; reviewers saw it only in OnePlus apps; one report has an unpatched third-party notes app receiving it | yes (in range or out only) | double-tap area: OnePlus apps only, per reviews. Another OnePlus pen's side button reaches every app as the key F21 on ColorOS 16; whether the Stylo 2's double tap sends any key is unmeasured (§2.2) | [secondary: S5, S6, S7, S20]; [unconfirmed: S12]; hover [verified: L6] |
| Apple Pencil Pro | yes: `UICanvasFeedbackGenerator` (iPadOS 17.5): *"When using Apple Pencil Pro with a compatible iPad, this type of feedback can provide a tactile response"*, e.g. *"an object snapping to a guide"* | yes: `UIHoverGestureRecognizer.zOffset` (16.1) | squeeze (`UIPencilInteraction.Squeeze`, 17.5); barrel roll (`UITouch.rollAngle`, 17.5) | [verified: K8] |
| Windows haptic pens | yes: `SimpleHapticsController` with inking waveforms (*"InkContinuous … Required: Yes"*, Pencil/Brush/Marker optional) and interaction waveforms (Click, Hover, Success…) | yes | barrel button passes "unprocessed input" to the app | [verified: K9, K10] |
| Samsung S Pen | not found | yes (Air view; not researched further, and not in K11) | Air actions: the S Pen Framework *"converts them to KeyEvents"* for apps that declare them (click, double click, swipes, circles) | [verified: K11]; hover [unconfirmed] |

---

## 3. Android APIs for each capability

API levels are from each member's *"Added in API level N"* line on developer.android.com, read 2026-11-01
[verified: A1–A14].

### 3.1 Input

| Capability | API | Level | Notes |
|---|---|---|---|
| Tool type | `MotionEvent.TOOL_TYPE_STYLUS`, `TOOL_TYPE_ERASER` | 14 | eraser = *"an eraser or a stylus being used in an inverted posture"* |
| Pressure | `getPressure()` / `AXIS_PRESSURE` | 1 / 12 | *"normalized to a range from 0 (no pressure at all) to 1 (normal pressure), although values higher than 1 may be generated"* |
| Tilt | `AXIS_TILT` | 14 | radians; *"0 … perpendicular to the surface, and PI/2 … flat"* |
| Orientation | `AXIS_ORIENTATION`, `getOrientation()` | 12 / 9 | for a stylus *"from -PI radians to PI radians, where 0 is pointing up"* |
| Hover distance | `AXIS_DISTANCE` | 14 | *"don't rely on precise values for app-critical functionality"* (stylus guide). In LineageOS's copy of the Pad 3's touch driver it is only 0 or 1 (§2.2) [verified: L6] |
| Hover | `ACTION_HOVER_MOVE` / `ENTER` / `EXIT`; `View.onHoverEvent` | 12 / 14 / 14 | Godot's render views receive them in `onGenericMotionEvent` [verified: G1, `GodotGLRenderView`, `GodotVulkanRenderView`] |
| Barrel buttons | `BUTTON_STYLUS_PRIMARY` (32), `BUTTON_STYLUS_SECONDARY` (64); `ACTION_BUTTON_PRESS`/`RELEASE` | 23 | button actions go to `onGenericMotionEvent`, not `onTouchEvent` |
| Stylus key codes | `KEYCODE_STYLUS_BUTTON_PRIMARY` … `_TAIL` (308–311) | 34 | the tail button *"does not usually correspond to the function of an eraser"* |
| Function keys F13–F24 | `KEYCODE_F13` … `KEYCODE_F24` (326–337; F21 is 334) | 36 | level and values [verified: A5, read 2026-11-02]. A OnePlus pen's side button arrives as `KEYCODE_F21` [secondary: S20]. Godot 4.7.2 maps none of these, nor the stylus key codes (§4.2) [verified: G6] |
| Batched samples | `getHistorySize()`, `getHistoricalX(int,int)`, `getHistoricalEventTime(int)` | 1 / 5 / 1 | *"motion events with ACTION_MOVE may batch together multiple movement samples"* |
| Unbuffered dispatch | `View.requestUnbufferedDispatch(MotionEvent)`; `(int source)` | 21; 30 | *"just requesting unbuffered dispatch on most MotionEvent streams will not improve your input latency. Side effects include: increased latency, jittery scrolls and inability to take advantage of system resampling"* |
| Palm cancellation | `ACTION_CANCEL`; `FLAG_CANCELED`, *"only set for events with ACTION_POINTER_UP and ACTION_CANCEL"* | 1; 33 | FLAG_CANCELED *"is typically set when the user was accidentally touching the screen, such as by gripping the device, or placing the palm on the screen"* |
| Prediction | platform `MotionPredictor` (`record`, `predict`, `isPredictionAvailable`) | 34 | androidx `MotionEventPredictor` in `androidx.input:input-motionprediction` 1.0.0 (stable, 2025-11-19) [verified: A21] |
| Front-buffer ink | `GLFrontBufferedRenderer`, `CanvasFrontBufferedRenderer` in `androidx.graphics:graphics-core` (stable 1.0.4; 1.1.0-beta01, page updated 2026-10-07) | library min API 29 | *"not designed to be used for refreshing the entire screen"* [verified: A15, A20] |
| Ink library | `androidx.ink` 1.0.0 (2025-12-17); 1.1.0-alpha10 (2026-10-21) | — | [verified: A22] |
| Handwriting | `View.setAutoHandwritingEnabled`, `InputMethodManager.startStylusHandwriting` | 33 | for text fields; not relevant to a map |
| Input-device vibrator | `InputDevice.getVibratorManager()` | 31 | for devices such as gamepads [verified: A4] |
| Listener before the view | `View.OnTouchListener.onTouch` | 1 | *"This allows listeners to get a chance to respond before the target view"* [verified: A3] |

### 3.2 Haptics

| API | Level | What it does, in the documentation's words |
|---|---|---|
| `Vibrator.vibrate(long)`, `createOneShot(ms, amplitude)` | 1; 26 | the **legacy** path. *"Avoid using the legacy one-shot vibrations … These vibrations might feel buzzy"* [verified: A16] |
| `createWaveform(timings, amplitudes, repeat)` | 26 | step waveform; *"Non-zero amplitudes are rounded up to 100% on devices lacking amplitude control"* [verified: A17; the sentence is not on A8] |
| `Vibrator.hasAmplitudeControl()` | 26 | *"True if the hardware can control the amplitude"* |
| `createPredefined(EFFECT_CLICK / TICK / HEAVY_CLICK / DOUBLE_CLICK)` | 29 | effects with a platform fallback |
| `VibrationEffect.Composition`, `startComposition()`, `addPrimitive(id, scale, delay)` | 30 | *"If a composition contains even one unsupported primitive, the entire vibration fails to play"* [verified: A17] |
| `PRIMITIVE_CLICK`, `TICK`, `QUICK_RISE`, `SLOW_RISE`, `QUICK_FALL` | 30 | TICK: *"a light crisp sensation intended to be used repetitively for dynamic feedback"* |
| `PRIMITIVE_THUD`, `SPIN`, `LOW_TICK` | 31 | LOW_TICK: *"very short low frequency effect … intended to be used repetitively"* |
| `areAllPrimitivesSupported`, `arePrimitivesSupported` | 30 | *"If a primitive is not supported by the device, then no vibration will occur"* |
| `getPrimitiveDurations(int...)` | 31 | duration in ms, *"zero for the unsupported ones"* |
| `VibratorManager`, `getDefaultVibrator()`, `vibrate(CombinedVibration)` | 31 | — |
| `Vibrator.vibrate(effect, VibrationAttributes)` | 33 | *"Requires Manifest.permission.VIBRATE"*; *"The app should be in the foreground"* |
| `getResonantFrequency()`, `getQFactor()` | **34** | NaN when unknown |
| `addPrimitive(id, scale, delay, delayType)`, `DELAY_TYPE_PAUSE`, `DELAY_TYPE_RELATIVE_START_OFFSET` | 36 | exact start offsets |
| `BasicEnvelopeBuilder` (intensity + sharpness) | 36 | *"designed to provide a consistent haptic perception across devices"*; guarantees *"sensitivity levels (SL) above 10 dB SL"* [verified: A10] |
| `WaveformEnvelopeBuilder` (amplitude + Hz) | 36 | *"do not support automatic haptics fallback. If the device does not support PWLE effects or the requested frequencies, the vibration will not play"* [verified: A10] |
| `areEnvelopeEffectsSupported()`, `getEnvelopeEffectInfo()`, `getFrequencyProfile()` | 36 | — |
| `createRepeatingEffect(effect)` | 36 | — |
| `VibrationEffect.Builder`, `VibrationEffect.Preset` (`PRESET_CLICK`, `PRESET_TICK`, `PRESET_LOW_TICK`) | **"Added in version 37.2"** on the reference page; the guide says *"Starting in Android 16 (26Q4)"* | *"built-in, framework-level automatic fallback"*. The two pages disagree on the version label; either way it is newer than the Pad 3's launch OS [verified: A11, A17] |
| `View.performHapticFeedback(constant)` | 3 | *"doesn't require any special permissions"*; *"honours the user's system setting for enabling touch feedback"* [verified: A19] |

**`HapticFeedbackConstants`, every feedback constant on the page** [verified: A12]. The page also lists two flags for
`performHapticFeedback(int, int)`, `FLAG_IGNORE_VIEW_SETTING` (level 3) and `FLAG_IGNORE_GLOBAL_SETTING` (level 3,
deprecated in 33), left out here:

| Level | Constants |
|---|---|
| 3 | `LONG_PRESS` |
| 5 | `VIRTUAL_KEY` |
| 8 | `KEYBOARD_TAP` |
| 21 | `CLOCK_TICK` |
| 23 | `CONTEXT_CLICK` |
| 27 | `KEYBOARD_PRESS`, `KEYBOARD_RELEASE`, `VIRTUAL_KEY_RELEASE`, `TEXT_HANDLE_MOVE` |
| 30 | `CONFIRM`, `REJECT`, `GESTURE_START`, `GESTURE_END` |
| 34 | `SEGMENT_TICK`, `SEGMENT_FREQUENT_TICK`, `DRAG_START`, `TOGGLE_ON`, `TOGGLE_OFF`, `GESTURE_THRESHOLD_ACTIVATE`, `GESTURE_THRESHOLD_DEACTIVATE`, `NO_HAPTICS` |

- **Texture constants do exist.** AOSP's haptics UX guide calls `CLOCK_TICK` (level 21) and `TEXT_HANDLE_MOVE`
  (level 27) *"haptic constants that are designed for virtual haptic textures"*, which *"must be subtle to provide the
  feeling of movement through repeated cues"* [verified: A29]. The two *segment* constants (level 34) are the other
  candidates.
- `SEGMENT_FREQUENT_TICK` is *"expected to be very soft, so as not to be uncomfortable when performed a lot in quick
  succession. If the device can't make a suitably soft vibration, then it may not make any vibration"*. It fits
  contour ticks.

### 3.3 Vibrator rate, latency and ringing

| Quantity | Value | Tag |
|---|---|---|
| Primitive targets | CLICK 12 ms (limit < 30), TICK 5 ms (< 20), LOW_TICK 12 ms (< 30), QUICK_FALL 100 ms, QUICK_RISE 150 ms, SPIN 150 ms, THUD 300 ms, SLOW_RISE 500 ms (each ± 20 ms) | [verified: A23] |
| Peak acceleration targets | CLICK 2 G, TICK 1 G, LOW_TICK 0.25 G | [verified: A23] |
| Frequency targets | CLICK *"Roughly at resonant frequency"*; TICK at *"2x resonant frequency"* (limit < 500 Hz); LOW_TICK at *"2/3 resonant frequency"* (limit < 100 Hz) | [verified: A23] |
| A single custom pulse | *"20 ms is the recommended maximum threshold … longer than 20 ms isn't recommended because it's perceived as a buzzy vibration"* | [verified: A24] |
| Ringing | *"the actuator may continue to ring for another 20 to 50 milliseconds after a 20-millisecond input"*; a measured step shows *"visible ringing that lasts at least 50ms"* | [verified: A16, A18] |
| Gap between primitives | *"Use delays of 50 ms or longer for discernible gaps"* | [verified: A17] |
| Calibrating ticks | *"calibrate a single tick or low tick primitive with a larger composition, for example, 100 consecutive ticks"* | [verified: A23] |
| LRA resonance | *"200 to 300 Hz range"* on some, *"around 150 Hz"* on others | [verified: A18] |
| Retrigger behaviour | whether a new `vibrate()` cancels a running one is **not documented** on the pages read | [unconfirmed]; measure in SH-0 |
| End-to-end vibrator latency on the Pad 3 | **not published** | measure (§8.2) |

---

## 4. Godot 4

### 4.1 The engine version

- The project declares `config/features=PackedStringArray("4.7", "GL Compatibility")` and `gl_compatibility` on
  mobile [verified: `project.godot`].
- `TOOLCHAIN.md` says the build machine has Godot 4.7.2 with 4.7.2 export templates (as of 2026-10-26)
  [verified: `TOOLCHAIN.md`].
- This section reads the **`4.7.2-stable`** tag, commit `ed1daf0`, through a sparse clone of `godotengine/godot`
  [verified: G1].

### 4.2 What the Android port passes to scripts

All read in G1.

| Path | What happens | Symbol |
|---|---|---|
| Which pens become mouse events | `isMouseEvent()` returns true for `TOOL_TYPE_MOUSE`, `TOOL_TYPE_STYLUS`, `TOOL_TYPE_ERASER`, using **pointer 0's** tool type only | `GodotInputHandler.isMouseEvent`, `getEventToolType` |
| Pressure | `event.getPressure()` (pointer 0) | `GodotInputHandler.handleMouseEvent` |
| Tilt | `x = −sin(orientation) × sin(AXIS_TILT)`, `y = cos(orientation) × sin(AXIS_TILT)` | `getEventTiltX`, `getEventTiltY` |
| `pen_inverted` (eraser) | never set on Android: no `set_pen_inverted` call exists under `platform/android` | `grep -rn set_pen_inverted platform/android` → nothing |
| Hover | `ACTION_HOVER_*` become an `InputEventMouseMotion` with position and relative only: no pressure, tilt, distance or button mask; enter and exit are not distinguished | `AndroidInputHandler::process_mouse_event` (hover branch) |
| Moving with the tip down | `InputEventMouseMotion` with `set_pressure`, `set_tilt`, `set_button_mask` | same, `AMOTION_EVENT_ACTION_MOVE` |
| Buttons | mapped: PRIMARY, SECONDARY, TERTIARY, BACK, FORWARD. **`BUTTON_STYLUS_PRIMARY`/`SECONDARY` are not mapped**. A mask that is not 0 is kept, so the press has button index `NONE` | `_android_button_mask_to_godot_button_mask`, `_button_index_from_mask` |
| Keys above F12, and the stylus key codes | the key table stops at `AKEYCODE_F12` and has no `AKEYCODE_STYLUS_*` entry. `godot_code_from_android_code` returns `Key::UNKNOWN` for any code not in it. `process_key_event` still emits the `InputEventKey` (its only early returns are for UTF-16 surrogates), so a script sees a press and a release with `keycode` and `physical_keycode` both `KEY_UNKNOWN`. Every key that is not a gamepad's is forwarded. Godot `master` (`95c0fa5`) has the same table [verified: G6] | `android_godot_code_pairs` in `android_keys_utils.h`; `godot_code_from_android_code`; `AndroidInputHandler::process_key_event`; `fix_keycode`; `GodotInputHandler.onKeyDown`/`onKeyUp` |
| Batched samples | only `getX()`, `getY()` and `getPressure()`, the newest point; no `getHistorical*` call exists in the Java input layer | `grep -rn getHistorical platform/android/java/lib/src` → nothing |
| Moves inside the touch slop | `onTouchEvent` returns early for every `ACTION_MOVE` the gesture handler did not take. `GestureDetector.onScroll` takes none until the move exceeds the touch slop (`TOUCH_SLOP = 8` dp, AOSP's default, which a device overlay, `config_viewConfigurationTouchSlop`, can change). `disable_scroll_deadzone` makes `onActionMove` dispatch them at once. Its doc warns of *"unintended scrolling from slight finger movements"* | `GodotInputHandler.onTouchEvent`, `GodotGestureHandler.onActionMove`; [verified: A25, G2] |
| Stylus quick-scale | `scaleGestureDetector.setStylusScaleEnabled(true)`: with the stylus button held, a drag scales. With `enable_pan_and_scale_gestures` on (Cartalith has it on), the scale becomes `InputEventMagnifyGesture` | `GodotInputHandler` constructor; `ScaleGestureDetector` (AOSP) [verified: A25] |
| Long press | `GestureDetector` long-press is off unless `enable_long_press_as_right_click`; Cartalith leaves it off and times its own holds | `Godot.kt` setup; `project.godot` |
| Pen and finger together | a finger joining a pen gesture is `ACTION_POINTER_DOWN` on the mouse path, which `handleMouseEvent` ignores. A pen joining a finger gesture makes the pen a touch index. Both cases need the pen and the finger in one Android event, that is, from one input device [judgement]; the next three rows say what follows | `handleMouseEvent` switch; `handleTouchEvent` |
| Tilt in a multi-touch event | every pointer gets pointer 0's tilt. `setTouchEvent` fills each pointer's tilt from `getEventTiltX(event)` and `getEventTiltY(event)`, which read the orientation and tilt with no pointer index, and `process_touch_event` passes it into each touch-drag event. So a pen that is touch index 1 behind a palm carries the palm's tilt [verified: G1, G6] | `InputEventRunnable.setTouchEvent`; `getEventTiltX`, `getEventTiltY` |
| Which touch the emulated mouse follows | the first touch pressed, until it lifts (`mouse_from_touch_index`). With a palm down first, the palm drives the emulated mouse that `map_overlay.gd` reads, and a pen that became a touch index does not [verified: G1] | `Input::_parse_input_event_impl` (ScreenTouch branch) |
| A pen stroke beside a resting palm | with `enable_pan_and_scale_gestures` on (Cartalith's setting), `onScroll` sends a scroll with two or more pointers and no drag in progress to `handlePanEvent`, which becomes an `InputEventPanGesture`; the scale detector can emit `InputEventMagnifyGesture`; `viewport_host.gd` handles both. So such a stroke can pan or zoom the map [verified: G6; `project.godot`; `viewport_host.gd`]. Whether it can happen depends on Android. AOSP's default rejects touches from other input devices while a stylus is down or hovering, unless one device reports pen and finger together [verified: A30]. LineageOS's copy of the Pad 3's touch driver registers the pen as its own input device [verified: L6]. What OxygenOS does is SH-0's palm probe | `GodotGestureHandler.onScroll`; `viewport_host.gd`'s gesture branches; `PreferStylusOverTouchBlocker` (AOSP) |
| Finger pressure | a finger drag's `ScreenDrag.pressure` is the finger's `getPressure(i)`, and touch-to-mouse emulation copies it into the emulated motion (device −1) | `InputEventRunnable.setTouchEvent`; `Input::_parse_input_event_impl` (ScreenDrag branch) |
| Accumulation | by default one motion event per frame: *"set Input.use_accumulated_input to false to make events emitted as often as possible"*. The merge keeps the **older** event's pressure and tilt and takes the newer position | `InputEventMouseMotion::accumulate`; `Input::parse_input_event` [verified: G1, G2] |
| `vibrate_handheld` | `Input.vibrate_handheld(duration_ms = 500, amplitude = -1.0)`; Java calls `VibrationEffect.createOneShot(ms, amplitude)` with amplitude `CLAMP(int(a × 255), 1, 255)`, after `requestPermission("VIBRATE")` | `GodotJavaWrapper::vibrate`, `GodotNativeBridge.vibrate` |
| Without the permission | `requestPermission` asks only for *dangerous* permissions, so for VIBRATE it returns true, and the `vibrate` call throws a `SecurityException` that is caught and logged | `PermissionsUtil.requestPermissions`; `GodotNativeBridge.vibrate` |

**What the input layer never reads.** In `GodotInputHandler.java`, `GodotGestureHandler.kt`, `InputEventRunnable.java`
and `GodotGLRenderView.java` at `4.7.2-stable`, `FLAG_CANCELED`, `getFlags`, `TOOL_TYPE_PALM`, `AXIS_DISTANCE`,
`getHistorical` and `requestUnbufferedDispatch` occur 0 times each, and `getToolType` once, in `getEventToolType`
[verified: G1, G6; counted 2026-11-02 with `cat <the four files> | grep -c <term>`].

**Version history** [verified: G5, GitHub search API, 2026-11-01]:
- Pressure and tilt on Android arrived in **#80644** (merged 2023-09-17, milestone 4.2).
- The `amplitude` argument arrived in **#91143** (merged 2024-05-02, milestone 4.3).
- Calling `vibrate_handheld` without the permission crashed in 4.4 (**#103496**); **#107385** (milestone 4.5) catches
  the exception instead.
- The editor's long-press haptic used `HapticFeedbackConstants.LONG_PRESS` in **#117198** (merged 2026-03-14,
  milestone 4.7).

### 4.3 Known issues and proposals

All from G5 (titles, states and dates as the search API returned them on 2026-11-01; the closing comments of
#124191–#124193 were read on the issue pages by the fact-check, 2026-11-02 UTC).

| Item | State | What it says |
|---|---|---|
| #124191 *Android: stylus side button is reported as MouseButton NONE (index 0) with an empty button mask* | **withdrawn**: closed 2026-10-05 by its own author, the day it was opened, after *"Used an ai to write this. closing for now and will use my own words later."*; a maintainer then added `archived`; no fix | 4.7.2 via Pixelorama, XPPen pad. It matches the source above, which is the evidence; the report itself is not [secondary, withdrawn] |
| #124192 *dragging with the stylus side button held sometimes becomes a magnify gesture with no motion events* | withdrawn the same way, same day | *"In my measurements this happened in 18 of 66 strokes"*, with pan-and-scale enabled; the author had *"not yet run [the minimal project] as a separate app on the device"*. Treat the figure as unconfirmed |
| #124193 *releasing the stylus side button during a drag ends the drag, and the rest of the stroke is lost* | withdrawn the same way, same day | — |
| #84020 *Stylus pressure not recognised on MIUI* | open since 2023-10-26 | pressure 0.0 on a Xiaomi pad |
| #85990 *Pen tilt angle is not in radians* | open | notes Android returns the sine of the angle (as the source shows) |
| #75903 *Pen/stylus input is not responsive enough for hand writing* | open | Windows; fast strokes lose points |
| #78059 *Android editor with a Samsung S Pen … "hold to right click" … scrolling stuck … then scrolls when hovering* | open | S Pen hover interacting with long-press |
| #122989 *Add `vibrate_on_long_press` to `Control` node* | open PR, milestone 4.x | — |
| proposals #6659 *Implement stylus button support*; #4702 *eraser-end detection*; #12981 *pen barrel roll axis* | open | none is an Android haptics proposal; no proposal for `VibrationEffect.Composition` was found |

### 4.4 What each capability needs in Godot

| Capability | Today's engine (GDScript, settings) | JavaClassWrapper or Rust JNI (no Gradle) | Kotlin v2 plugin (needs Gradle) or upstream Godot |
|---|---|---|---|
| Pressure, tilt | ✓ (with the caveats above) | — | — |
| All batched samples | ✗ | ✗ (needs a Java listener on the UI thread) | ✓ |
| Moves inside the slop | ✓ with `disable_scroll_deadzone=true` | — | — |
| Hover position | ✓ | — | — |
| Hover distance, enter/exit | ✗ | ✗ | ✓ for enter and exit; the distance is only 0 or 1 in this tablet's driver (§2.2) |
| A key from the pen (F13–F24 or a stylus key code), if the Stylo 2 sends one (§2.2) | arrives, but only as `KEY_UNKNOWN`; a script could treat an unknown key while the pen is near as the pen's [judgement] | ✗ | ✓ exactly, or an upstream key mapping |
| Eraser end | ✗ (no `pen_inverted` on Android) | ✗ | ✓ |
| Stylus buttons | ✗ (index NONE; and the Stylo 2 has none) | ✗ | ✓ |
| `FLAG_CANCELED` palm flags | ✗ | ✗ | ✓ |
| Unbuffered dispatch | ✗ | possible on the `SurfaceView`, from the UI thread; not recommended (§3.1) | ✓ |
| Prediction | ✗ (Godot never exposes `MotionEvent`s) | ✗ | ✓ |
| Front-buffered wet ink | ✗ | ✗ | ✓, as an overlay view; a large build |
| One-shot vibration with amplitude | ✓ `vibrate_handheld` (+ VIBRATE permission) | ✓ | ✓ |
| Primitives, envelopes, capability queries | ✗ | ✓ | ✓ |
| `performHapticFeedback` constants | ✗ | ✓ (a `View` is needed, e.g. the Activity's decor view, called on the UI thread through `runOnUiThread`; untested [judgement]) | ✓ |

**Why the input rows need Java.**
- `JavaClassWrapper.create_proxy()` can implement a Java interface in GDScript [verified: G3]. But the proxy calls
  straight into Godot on whichever Java thread invokes it. `AndroidRuntimePlugin.createProxyFromGodotObjectID` →
  `Callable.call` → `nativeCallObject` is synchronous [verified: G1, `AndroidRuntimePlugin.kt`,
  `variant/Callable.kt`].
- A touch listener on the render view would therefore run GDScript on Android's UI thread, beside Godot's own loop.
  That is not safe [judgement; untested].
- A Kotlin plugin's listener can copy the event and hand it over with `emitSignal`, which posts through
  `godot.runOnRenderThread` [verified: G1, `GodotPlugin.emitSignal`].
- The render view is reachable from a plugin: `Godot.renderView` is public and `GodotRenderView.getView()` returns the
  `SurfaceView` [verified: G1].
- `GodotPlugin` itself has no input hooks [verified: G1, its method list].
- The v2 plugin docs: *"v2 Android plugin requires the use of the Gradle build process"* [verified: G4]. The preset has
  `gradle_build/use_gradle_build=false` [verified: `export_presets.cfg`].

**Precedents in this repository:**
- `shell/llm_net_hint.gd::open_app_settings` already uses `Engine.get_singleton("AndroidRuntime")` and
  `JavaClassWrapper` (*"Godot 4.4+"*) [verified].
- `crates/cartalith-godot/src/llm_keystore_android.rs` is, in `ANDROID_BUILD_SCOPE.md`'s words, *"raw JNI from Rust
  (`jni-sys`, already in `Cargo.lock`), not a `.aar`"*; its own module doc says the same (*"WHY JNI FROM RUST, NOT A
  `.aar` PLUGIN"*) [verified: both].
- Ruling AV (2026-09-24) approved *"a small Android plugin: a `.aar`"* for the vault folder picker
  [verified: `LARGE_ITEM_RULINGS.md`]. No `.aar` and no `godot-project/android/` tree exists today
  [verified: `ls`, grep].

---

## 5. The web, as a reference only

### 5.1 Which pen axes exist (W3C Pointer Events)

Pointer Events Level 3 became a *"W3C Recommendation 30 June 2026"* [verified: W1]. Its pen model is a useful checklist
of axes, and it matches Android's:

| Axis | Definition (quoted) | Android |
|---|---|---|
| `pressure` | *"in the range of [0,1] … For hardware and platforms that do not support pressure, the value MUST be 0.5 when in the active buttons state and 0 otherwise"* | `AXIS_PRESSURE` |
| `tiltX`, `tiltY` | degrees in [−90, 90]; *"A positive tiltX is to the right"*, *"A positive tiltY is towards the user"* | `AXIS_TILT` + `AXIS_ORIENTATION` |
| `twist` | *"clockwise rotation (in degrees, in the range of [0,359]) … around its own major axis"* | none: no `AXIS_` constant for it [verified: A1, the full list of `AXIS_` constants]; cf. Apple's `rollAngle` |
| `tangentialPressure` | barrel pressure, [−1, 1] | none [verified: A1, same list] |
| eraser button | `buttons` bit 32 | `TOOL_TYPE_ERASER` |
| `getCoalescedEvents()`, `getPredictedEvents()` | the batched samples and a prediction | `getHistorical*`, `MotionPredictor` |

The web also illustrates the "no value" rule this project keeps. The spec reports `0.5` when no pressure exists, a
plausible value that is not a reading. Cartalith's `map_stroke_sample` passes `null` instead [verified:
`map_overlay.gd`, the signal's doc comment].

### 5.2 Web haptics

- The Vibration API takes on/off patterns only, with no amplitude. `Input.vibrate_handheld`'s own doc says *"For Web,
  the amplitude cannot be changed"* [verified: G2].
- Microsoft's Edge explainer for `navigator.playHaptics(effect, intensity)` and a CSS `@haptic` rule offers four
  effects (`hint`, `edge`, `tick`, `align`) and requires sticky user activation. It is marked *"Archived"*
  [verified: W2].
- Search excerpts say the CSS Working Group accepted haptics as a work item in March 2026 [unconfirmed: W3]. The
  explainer itself names its current venue as the W3C Web Incubator Community Group, at `github.com/wicg/web-haptics`
  [verified: W2].
- A browser can reach none of Android's primitives, envelopes or capability queries, and no pen-tip haptics.

### 5.3 The HTML working copy (one paragraph)

`html/Cartalith v2.73 DCC test.html` has a touch layer that only listens for `pointerType === 'touch'`. Counts were
taken with `grep -c`:
- `pointerType`: 6 lines. Five are the touch layer (`_tchReplay`, `_tchMultiStart`, `_tchDown`, `_tchMove`,
  `_tchUp`). The sixth sets the asset-library slicer canvas's grid-handle tolerance, 26 px for touch and 11 otherwise
  [verified: re-run 2026-11-02 on the working tree, enclosing function named for each line].
- `.pressure`, `tiltX`, `altitudeAngle`, `getCoalescedEvents`, `getPredictedEvents`, `navigator.vibrate`: 0 each.

So it has no pen path and no haptics. A pen reaches the base `view` handlers as if it were a mouse [verified: the
handlers, read 2026-11-02; no device run]:
- Outside the Tiled LOD viewer, the sculpt brush ring (`sculptRenderCursor`) follows a hovering pen.
- An eraser contact (`button` 5) sculpts like the tip outside LOD. Under LOD the sculpt path needs `e.button === 0`, so
  an eraser contact pans. Paint's `pointerdown` has no button test, so an eraser paints. CIVIL's tools return unless
  `e.button === 0`.
- Sculpt points carry no pressure: `_sculptPts` holds `{x, y}` only.
- Chrome on Android reports an eraser as `pointerType` "pen", with the eraser bit (32) set in `buttons` and `button` 5.
  It passes tilt and twist only for a stylus tool, so an eraser's tilt reads 0 [verified: W4]. `navigator.vibrate`
  needs a user gesture from Chrome 60, and Firefox for Android returns `true` without vibrating [verified: W5], so a
  `true` does not prove a motor.

Its changelog says so for haptics and *Pen draws, finger pans* [verified: `html/CHANGELOG_v2.73.md`]. These would
carry over:
- the Pointer Events pen axes;
- pen-only drawing by `pointerType === 'pen'`, with the palm rule;
- the relief computation in §7.4, which is plain arithmetic on the height grid;
- on/off pulse trains through `navigator.vibrate` on Chrome for Android.

Nothing else in this document applies to it.

---

## 6. How the best apps do it

### 6.1 Krita

- **The global pressure curve.** *"Input Pressure Global Curve : This is the global curve setting that your tablet will
  use in Krita. The settings here will make your tablet feel soft or hard globally"* [verified: K1, manual 5.3.0].
- **Per-brush sensors** drive size, rotation, opacity and more. The manual lists sixteen: Pressure, PressureIn,
  X-tilt, Y-tilt, Tilt-direction, Tilt-elevation (*"0° is the stylus horizontal, 90° is the stylus vertical"*), Speed,
  Drawing Angle, Rotation, Distance, Time, Fuzzy (Dab), Fuzzy Stroke, Fade, Perspective and Tangential Pressure.
  Two examples of use:
  - *"if you have trouble drawing fine lines, try to use a concave curve set to pressure"*.
  - For left-handed users the Tablet Settings carry a *"Pen Tilt Direction Offset"* (5.3).
  ([verified: K3, K1])
- **Smoothing** has five modes: none, basic, weighted, stabilizer and pixel. The stabilizer *"averages all inputs …
  draw a circle around your cursor"*, and since 5.3 uses *"Sample Count at Max Speed"* and *"at Min Speed"*. *"Stabilize
  sensors"* smooths pressure, speed and tilt too [verified: K2].
- **The Tablet Tester** draws mouse events in red and tablet events in blue, and logs every event: *"If you have no blue
  line … Krita cannot access your tablet"* [verified: K1].
- **Android-only workarounds** (5.2.15):
  - *"Treat page up/down keys as right/middle mouse buttons (workaround for Xiaomi styluses)"*.
  - *"Disregard tablet position history (workaround for jagged lines with Xiaomi styluses)"*.
  ([verified: K1])
- **Android hover and buttons.** Krita carries its own Qt patch, *"Android: Properly handle Tablet events"* (2020). Its
  message: *"Prior to this Hover events from stylus/tablet were treated as Mouse events and primary and secondary stylus
  buttons weren't handled either"*.
  - It maps `BUTTON_STYLUS_PRIMARY` to the middle button and `SECONDARY` to the right.
  - It computes tilt in degrees as `toDegrees(−sin(orientation) × AXIS_TILT)`: linear in the angle, where Godot uses
    its sine.
  - ([verified: K5]) KDE bug 429940, where pressure was *"completely ignored"* on ChromeOS, was fixed by that patch on
    2021-01-19 [verified: K4].
- **Touch painting and stray strokes.** *"Auto (the default) will automatically detect whether you have a stylus
  available and enable or disable touch painting accordingly"*; Auto was added and made the default in 5.2.13
  [verified: K13]. Long-press context menus are *"enabled by default on Android"* (5.2.14) [verified: K13]. Merge
  request !2680's stray-stroke guard is in `MAP_INTERACTION_RESEARCH.md` §4 [secondary: K12]; Ruling EC adopted the
  guard.
- **Brush outline.** *"Tilt Outline"* gives *"a circular outline with a tilt-indicator"*. *"Use effective outline
  size"* keeps the outline at *"the maximum possible brush diameter"* [verified: K13]. Both bear on SH-6's hover ring.
- **The OnePlus key.** Krita's Qt fork added handling for F13 to F24 and a flag that turns F21 into a middle click, and
  Krita turned the translation on unconditionally (§2.2) [verified: K15].

### 6.2 Other apps and platforms

| App or platform | What matters here | Tag |
|---|---|---|
| Infinite Painter | Stylus settings appear *"only when the application detects that you are using a pressure-sensitive stylus"*. The finger gets *"a separate function"*. Front and back button actions, with a warning that *"secondary buttons in styluses may send non-standard codes"*. *"Global dynamics curves"* calibrate *"pressure, velocity and tilt"*. Version 7.1 supports Google's *"Low Latency Drawing technology"*, with the settings *"Disabled, Active, Fastest"* | [verified: K6] |
| Procreate | Global *"Stabilization"* sliders and a pressure curve under Actions ▸ Prefs ▸ Pressure and Smoothing; *"We generally don't recommend adjusting your global pressure settings unless you need to"*. Its graph shows *"The pressure of your pencil input on the X axis"* and *"The effect of that pressure on your brushes on the Y axis"*, and *"adjusting the curve can have unwanted results"* | [verified: K7] |
| Drawpile | Ships a setting, *"Translate F21 key to middle click (OnePlus)"*. Its developer: *"The F13 to F24 keys aren't handled by the underlying Qt framework"*. The F-Droid notes for 2.3.1-beta.1 list *"stylus support and workarounds for Wacom MovinkPad, OnePlus and Xiaomi devices"* | [verified: K14]; the developer's post [secondary: S20] |
| Apple UIKit | `coalescedTouches(for:)`: *"Some devices gather touch data at rates up to 240 Hz"*. `predictedTouches(for:)` exists *"to minimize the apparent latency"*. Pencil force is *"measured along the axis of the pencil"*. Hover `zOffset` is used *"to generate a preview that indicates where a hovering Apple Pencil touches down"* | [verified: K8] |
| Windows Ink | *"By default, the InkCanvas supports ink input only from a pen"*. The eraser tip or erase button gives an erase stroke. Barrel-button input can be passed to the app unprocessed. Wet ink is drawn *"on a low-latency background thread"* | [verified: K10] |
| Windows pen haptics | Continuous inking waveforms play *"while the pen contacts the screen"*. Interaction waveforms play for UI actions. *"Always stop haptic feedback when the pointer exits the element"* | [verified: K9] |
| Clip Studio Paint, ibis Paint, Concepts | finger and palm policies are summarised in `MAP_INTERACTION_RESEARCH.md` §4. The Parka Blogs reviewer *"adjusted the pressure curve"* in Clip Studio Paint, on the Pad 2, and *"can draw the thin and thick lines more easily"* with the Stylo 2 | [secondary: K12; S5] |
| ArcGIS Field Maps, QField | a stylus locks the markup tool and *"your fingers can still be used to pan, zoom"*; QField's freehand mode turns on when a stylus hovers | [secondary: K12] |
| Wacom guidance | not opened this session | — |

**Map and terrain apps that make relief felt.** None was found that renders terrain relief as vibration on a consumer
tablet. That is a negative search result, so it is unconfirmed. The nearest work is research on vibrating lines and
maps for blind users (§7.2).

### 6.3 The practices that matter for Cartalith

1. **One global pressure curve, plus calibration** (Krita, Infinite Painter, Procreate). The Stylo 2 needs one, because
   its light-pressure response is weak [secondary: S4, S5]. Offer soft, linear and firm presets and a
   *"press as hard as you comfortably draw"* calibration. Do not build a free curve editor.
   `EDITING_TOOLS_SCOPE.md` ET-D1 already rejected one, citing Blender's move back to presets
   [verified: that table's Dynamics row].
2. **Smoothing is a per-tool option.** Cartalith's stroke engine already has a "pulled string" smoother
   (`stroke.rs`, `smoothing` cells) [verified: `paint_stroke_begin`'s option table].
3. **The pen draws and fingers navigate**, once a pen has been seen (Infinite Painter, Field Maps, Windows Ink,
   Krita's touch-painting Auto).
4. **Palm handling.**
   - Ignore touches while a pen is down and for a short while after.
   - Honour `ACTION_CANCEL`.
   - Never let a resting palm undo: the Stylo 2 reviewer's complaint was exactly a palm triggering *"double-tap undo"*
     [secondary: S4].
5. **Hover shows the brush footprint** (Apple's zOffset preview, Android's stylus guide: *"Stylus hover can be used to
   preview the size of the brush"*) [verified: K8, A15].
6. **Prediction draws the cursor ahead, never the committed stroke** (Apple's predicted touches are *"additional,
   temporary input"*; W3C's are "speculative" draw-ahead) [verified: K8, W1].
7. **A tester screen that shows raw events** (Krita) is how stylus problems are diagnosed. Krita's two Xiaomi
   workarounds, and Drawpile's and Krita's OnePlus F21 translation (§6.2), show that per-device quirks are normal.
8. **Haptics that mean something.**
   - Apple fires canvas haptics on alignment, as in snapping to a gridline.
   - Windows plays continuous ink textures and stops them when the pen leaves.
   - Android says frequent events must be *"very subtle"*.
   - All three say the same thing: tie each haptic to a geometric event the user can see, and keep the frequent ones
     soft [verified: K8, K9, A16].

---

## 7. Feeling the relief

### 7.1 What this hardware can make felt

| Channel | On the Pad 3 + Stylo 2 | On the OnePlus 6T |
|---|---|---|
| Pen-tip vibration | exists, but no access (§2.3) | no pen |
| Device body vibration | **unknown whether the Pad 3 has a vibrator, and it may have none**: LineageOS's build for it ships no vibrator service (§1.4). SH-0's `cmd vibrator_manager list` settles it, then the type. If there is one, a 675 g tablet resting on a desk will damp it; it is best felt through a supporting hand or the drawing palm [judgement] | a phone vibration motor, whose type this tree does not record; its HAL capabilities under the handset's LineageOS 22.2 / Android 15 build are unmeasured [verified: `ANDROID_BUILD_SCOPE.md` for the OS] |
| Sound | 8 speakers [verified: S1]; an optional click per contour is an accessibility fallback, not haptics. One review: *"the back starts to vibrate when you turn it up from around the 5th level"* [secondary: S17]; whether a quiet sound can be felt as well as heard is untested [judgement] | speaker |
| Sight | the contour lines, the hover ring and the cursor are already visible | same |

### 7.2 What the research says

- **Texture is felt as vibration whose frequency is speed over spacing.** Bensmaïa and Hollins recorded skin vibrations
  from textured surfaces. They found the peak frequency *"coincided with the natural frequency"*. That frequency *"is
  proportional to belt speed and inversely related to spatial period"*: F = v ÷ P.
  - But **roughness tracks intensity, not frequency**, for the fine surfaces the paper studies (it proposes this for
    *"a fine surface (spatial period<200 μm)"*). *"doubling the scanning velocity does not result in the substantial
    decrease in roughness predicted by a frequency theory … the effects of speed on roughness match those of speed on
    power"* [verified: R3, full text].
  - **For the design:** a rate that follows pen speed over feature spacing, and an *intensity* that follows roughness.
- **Tool-mediated texture rendering is driven by the user's speed and force.** When a hand-held probe moves over a
  surface, *"a vibration signal is induced in that probe that is a function of the user's motion as well as the
  properties of the material"*, and modelling that relationship is *"a popular method for creating virtual textures
  in hand-held probes"*; Heravi et al. cite Culbertson et al.'s and Romano and Kuchenbecker's models for it (R5 among
  them) and train on the Penn Haptic Texture Toolkit's data. Their own renderer plays texture only above *"a speed threshold of 5 millimeter per
  second"* [verified: R4].
  - R5's virtual textures *"accurately capture and recreate the roughness of real textures"*, played through *"a
    Wacom tablet and a stylus augmented with a Haptuator"* [verified: R5 abstract, PubMed 25248220].
- **Speed changes felt roughness through a rigid probe too.** Lederman, Klatzky, Hamilton and Ramsay varied the speed at
  which textured surfaces were explored with a rigid probe: *"Substantial effects of speed were obtained"*
  [verified: R13, abstract]. A pen is such a probe [judgement].
- **A pen that recreates writing on paper.** RealPen is *"an augmented stylus for capacitive tablet screens that
  recreates the physical sensation of writing on paper"*, by *"re-generating the friction-induced oscillation and
  sound of a real writing tool in contact with paper"*, with *"sound feedback aligned with the writing pressure and
  speed"* [verified: R12, abstract]. It is the closest pen precedent. It pairs touch with sound, the fallback the Pad 3
  may need (§1.4).
- **Lines and maps drawn in vibration, for blind and low-vision users.** Tennison, Uesbeck, Giudice, Stefik, Smith and
  Gorlewicz (ACM TAP 17(2), 2020) recruited 26 blind or visually impaired participants and analysed 17 (seven pilot
  participants excluded, two removed for missing data), on a *"10.0-inch Samsung Galaxy Tab S tablet"*. They used
  8 mm wide lines and on/off pulse trains at 2.5–500 Hz [verified: R2].
  - Lines at 2.5 Hz were most often called **dashed** (65.79 %), 5–10 Hz **dotted** (55.03 %), 25–50 Hz **solid**
    (41.58 %).
  - At 100 Hz and above, participants felt no feedback or did not read it as *"a representation of a line"*
    (61.75 %, the two answers pooled).
  - The motor's own operating frequency was *"around 200 Hz"*. Pulse trains above 100 Hz were *"sporadic"*.
  - Sadia et al. (2021) cite Tennison and Gorlewicz (2016): shapes *"with up to four edges"* were identified when
    rendered by vibrotactile actuators. For their own electrovibration edges they used a width of 10 mm, *"chosen based
    on the guidelines provided by Palani et al. (2018b) for vibrotactile rendering of virtual lines"* [verified: R11].
  - Giudice et al. (ASSETS '12) built a vibro-audio interface on a Samsung Galaxy tablet. Search excerpts credit it
    with an 8.9 mm best line width for tracing, but others credit 8.89 mm to a 2014 Palani and Giudice paper, whose
    PDF did not load [unconfirmed: R16].
- **Klatzky et al. (2014)** note that a vibrating screen excites the whole fingertip, so there is no gradient across
  the skin to signal leaving a line, and that a vibrated line must be wider than an embossed one [unconfirmed: R6, the
  PDF would not load; from a search excerpt].
  - The consequence for a pen is a [judgement]: the pen tip is a point and the body vibration is global, so relief
    must be coded **in time** (ticks as the pen moves), not in space.
- **Latency.** Kaaresoja, Brewster and Lantz (ACM TAP 11(2), 2014): *"tactile feedback latency should be between 5 and
  50 ms"*. The point of subjective simultaneity was 5 ms for tactile. *"The perceived quality dropped significantly
  between latency conditions 70 and 100 ms"* [verified: R1, abstract].
- **Detection thresholds.** Android's actuator primer cites Bolanowski et al. (1988) for the detection threshold, and
  its basic envelopes keep every vibration at least 10 dB above it [verified: A18]. Bensmaïa and Hollins add that the
  Pacinian channel *"is almost 16 times more sensitive at 216 Hz than it is at 108 Hz"* [verified: R3].
- **Interactive vibrating maps.** TouchOver map (Poppinga et al., MobileHCI '11, 545–550) is an early example of a phone
  map that answers touch with vibration and sound [unconfirmed: R8, bibliographic record only]. Walker and Salisbury's
  *Large haptic topographic maps* (I3D 2003) is a force-feedback precedent [unconfirmed: R15, bibliographic record
  only].
- **Surface haptics** (electrovibration, ultrasonic friction) change friction under the finger. Kim, Israr and
  Poupyrev (UIST '13) rendered 3D features such as bumps on touch screens this way [unconfirmed: R14, from a search
  excerpt]; Sadia et al. cite such work for rendering *"3D geometrical shapes in the form of height maps"*
  [verified: R11]. The Pad 3 has neither [judgement], so they are out of scope.

### 7.3 Perceptual and hardware limits used below

| Limit | Value used | Source |
|---|---|---|
| Latency from pen move to tick | median ≤ 50 ms; never beyond 70 ms | [verified: R1, for feedback to a press on a virtual button]; applying it to a moving pen is a [judgement] |
| Discrete ticks | gap ≥ 50 ms, so at most 16 ticks/s with a 12 ms primitive; 15/s is used | [verified: A17]; 16/s [estimate: 1000 ÷ (50 + 12) = 16.1, rounded down]; 15/s, one under it [judgement] |
| Gap detection at the fingertip | **not used.** With a lab tactor on the index fingertip, *"the estimated T gap threshold was 5.0 ms (SE: 0.37 ms)"*. Android says *"a 5 to 10 ms gap between two primitives is too short to be detectable"* on a device; a phone motor's 20–50 ms of ringing is the likely reason, so the 50 ms gap above stands | [verified: R10; A17; A16]; the reason [judgement] |
| Pulse train felt as separate marks | ≤ 10 Hz (dotted or dashed) | [verified: R2] |
| Pulse train felt as a continuous line | 25–50 Hz (read as solid by 41.58 %) | [verified: R2] |
| Intensity steps | 3 levels, scale 0.5 / 0.7 / 1.0 (ratio ≥ 1.4) | [verified: A17]. A NASA review agrees: *"only three widely spaced intensity values that would probably be usable"* (Geldard 1960), and an intensity step is noticed at *"5% to 30% with an average around 16%"* (Craig 1972), each as cited by R9 [verified: R9] |
| Vibration frequency | peak sensitivity *"in the range of 150 to 300 Hz"*, lowest thresholds at the fingertips; on the finger *"only three to five rates might be distinguished over the range of 2 to 300 Hz (or 5 to 8 steps if intensity is added as a redundant cue)"*. So the texture carries roughness mostly in its intensity, as R3 found, and its rate only coarsely | [verified: R9, the last citing Sherrick 1985]; the design use [judgement] |
| Short pulse length | ≤ 20 ms | [verified: A24] |
| Tick and low-tick durations | 5 ms and 12 ms targets; read the device's own with `getPrimitiveDurations` | [verified: A23, A9] |
| Below this speed, no texture | 5 mm/s, the threshold R4's own renderer used, not a measured perceptual limit | [verified: R4] |
| An example motor, for scale only | Precision Microdrives C10-100, a 10 mm coin LRA, not the Pad 3's or the 6T's part: *"Typical operating current"* 67 mA, resonant frequency 175 Hz, typical lag 11 ms, rise 37 ms, stop 177 ms. A motor takes time to start and longer to stop, and §7.9's latency bar includes that time [judgement] | [verified: H1] |

### 7.4 The mapping

**The principle is "feel what you see"** [judgement]. Each cue sits on something the map already draws, computed the
same way the renderer computes it.

| Terrain quantity | Haptic cue | Computed from (engine symbol) | When |
|---|---|---|---|
| **Contour crossing** | one tick per line crossed | interval `iv` = `contour_m ÷ peak_m` (peak 4 000 m when `peak_m` is 0), held between 0.001 and 0.2, or 0.05 when `contour_m < 5`; relief `r = (h − sea) ÷ (1 − sea)`, as `render.rs::apply_npr`'s contour block and the `r_frac` passed to `land_color` compute them. A crossing is a change of `floor(r ÷ iv)` | paint, sculpt, hover |
| **Index contour** (every 5th) | the next strength step up (0.7 → 1.0) | `apply_npr`'s own rule: index line when `round(r ÷ iv) % 5 == 0`, drawn 1.4× stronger. That is the same 1.4 ratio Android recommends [verified: both] | same |
| **Up or down** | rising crossings: `TICK`; falling: `LOW_TICK` (lower frequency: AOSP targets TICK at twice the resonant frequency and LOW_TICK at two-thirds of it [verified: A23]) | sign of `Δr` along the stroke | same [judgement] |
| **Slope** | the strength of each tick: below 5° → 0.5, 5–15° → 0.7, above 15° → 1.0 | `sample_bridge::slope_at` (the gradient) or `cartalith_terrain::analysis::slope`, turned to degrees as `sample_cell`'s `slope_deg` does | same; thresholds [judgement] |
| **Ruggedness** | a texture: an envelope's intensity ∝ TRI, its pulse rate = pen speed ÷ feature spacing on screen (F = v ÷ P, R3) | `cartalith_terrain::analysis::ruggedness`'s formula (mean absolute height difference to 8 neighbours, resolution-scaled). That function builds a whole grid; the bridge would apply the same formula at the sample point from 9 cells, so no new grid is kept [judgement] | paint, sculpt; not hover |
| **Ridge crest** | one `CLICK` at 1.0 (`THUD` 0.5 where supported, for a crest above an index interval) | the stroke's own height profile: `dr/ds` changes from positive to negative after rising at least ½ `iv` | paint, sculpt, hover [judgement] |
| **Valley floor** | one `LOW_TICK` at 0.7 | `dr/ds` changes from negative to positive after falling at least ½ `iv` | same |
| **River** | one `QUICK_FALL` at 0.5 | `WorldState.river_mask` (or `stream_order`) changes under the pen | paint, sculpt [judgement] |
| **Coast, land → water** | `QUICK_FALL` at 0.7 (fallback: two `LOW_TICK`s 60 ms apart) | `drawn_water_classification()` changes from land to ocean or lake, the same classification the map draws | all |
| **Coast, water → land** | `QUICK_RISE` at 0.7 (fallback: two `TICK`s) | the reverse | all |
| **Open water** | silence | contours are land-only in the renderer (`apply_npr` is skipped on water) | all |

**Ruggedness is not independent of slope** [estimate: the formula re-derived on uniform planes]. On a plane whose
gradient is g, `ruggedness`'s mean of the eight absolute differences comes to 0.71 g to 0.79 g, depending on the
gradient's direction, while `slope` is g, both times `gw`. So the texture row would also play on smooth steep ground,
which the slope row already codes. The function's own doc comment says a uniform slope has *"a small ruggedness"*;
the arithmetic says otherwise.

**Why the crest and valley come from the stroke's profile, not from a ridge map** [judgement]:
- It needs no new field.
- It fires exactly where the pen passes over the top, whatever the angle.
- It cannot fire on flat ground.

The engine's ridge tracer (`cartalith_terrain::vector::trace_ridgelines`) stays a later option for "you crossed a
named range's spine".

**Pressure does not change the haptics** [judgement]. The pen is already pressing; scaling the haptic by pressure
would make light sketching feel nothing.

### 7.5 How the zoom changes the rate

The crossing rate is (pen speed) × (map metres per screen millimetre) × tan(slope) ÷ (contour interval). The Pad 3 is
273.5 mm wide [estimate: §1.2]. These examples assume a 2 000 km-wide world and a 50 mm/s pen; all the inputs are
assumptions [estimate]:

| Zoom | Slope | Interval | Metres of rise per screen mm | Crossings per second |
|---|---|---|---|---|
| fit (1×) | 2° | 50 m | 255 | 255 |
| 16× | 30° | 50 m | 264 | 158 (pen at 30 mm/s) |
| 64× | 10° | 50 m | 20 | 20 |
| 64× | 10° | 100 m | 20 | 10 |
| 256× | 10° | 50 m | 5 | 5 |

So at the fit zoom a world map's contours come too fast to feel one by one; they merge visually too. **The haptic
interval therefore scales by zoom** [judgement]. Start at the map's own interval. If the expected crossing rate at the
current pen speed exceeds 10 per second, step it to 2×, 5× (the index lines), 10×, 25× … until it does not. Above the
top step, play texture only. The visible contour interval is never changed.

### 7.6 Computing it from the engine

**The Rust bridge** (new file `crates/cartalith-godot/src/haptic_bridge.rs`, one `#[func]` on `WorldGen`) [judgement]:
- **Call:** `haptic_segment(x0, y0, x1, y1, iv, sea) -> PackedFloat32Array`, in grid coordinates. It returns a short
  event list: kind, position along the segment (0..1) and strength, plus the end point's slope and ruggedness.
- **Sampling:** it walks the segment at half-cell steps with a bilinear height sampler over `WorldState.field`. The
  renderer already has one: the private free function `render.rs::sample_arr` (it *"Never wraps, even in world
  mode"*) behind `RenderCtx::h_f` [verified]. A wrapping world needs the coordinate wrapped first, as
  `RenderCtx::grad_at_f` does. The walk caps at 64 samples, so a long segment at low zoom stays cheap.
- **Water:** it reads `WorldState.sea_level` and `drawn_water_classification()`.
- **Why Rust:** one boundary crossing per motion event. `sample_cell` would be one call per point, returning a
  dictionary of about sixteen fields; its own doc says that dictionary exists so the Sample panel does not make sixteen
  round trips per motion event [verified: `sample_cell` doc].
- **Deep zoom:** at deep zoom the drawn relief comes from LOD tiles with synthesised detail
  (`info_tile.rs`, `lod_bridge::tile_heights`). The first version samples the base field. Tile heights are a later
  step, so that the feel matches the deep-zoom picture [judgement; listed in §9].

**The GDScript side** (new `shell/haptic_relief.gd`, owned by `app.gd`) [judgement]:
- **Pen and hover samples:**
  - Drawing samples come from `map_stroke_sample` (with the pen check fixed, SH-2).
  - Hover samples come from `cursor_sampled`, while the button mask is empty and a drawing tool is armed.
- **Pen speed:** taken in screen millimetres from consecutive positions and timestamps. Pixels become millimetres with
  the platform's `xdpi` read through `JavaClassWrapper` (`DisplayMetrics`), or the published 315 PPI as a fallback.
- **The interval:** `iv` and `peak_m` are read from the live appearance, the same `Npr` record `render.rs` uses, so the
  ticks land on the drawn lines.
- **Scheduling:** events go to a scheduler (§7.7), which plays them through a backend `shell/haptic_out.gd`.
  - The backend uses `JavaClassWrapper` with the capability ladder below.
  - It falls back to `Input.vibrate_handheld()` when `AndroidRuntime` is missing.
  - A Rust JNI backend, modelled on `llm_keystore_android.rs`, is the second choice if SH-7 measures the GDScript JNI
    cost as too high.
- **Sculpt and paint:** sculpt strokes are captured as a polyline and stamped at the end, so the field under the pen
  does not change during a stroke. Paint changes classes, not heights. So both feel the existing terrain.
  - Colour paint (`shell/colour_paint_tool.gd`) never alters heights.

### 7.7 Scheduling, fallbacks and degradation

**The scheduler** [judgement, built on §7.3]:
- **Gaps and rate:** at least 50 ms between discrete primitives (configurable 40–80 ms), at most 15 per second.
- **Priority** when events compete: coast > crest or valley > river > index contour > contour > texture.
- **Texture mode:**
  - It starts when discrete demand exceeds 10 per second for more than 150 ms.
  - It plays 100 ms segments: an envelope, a waveform with amplitudes, or a pulse train between 25 and 50 Hz. The
    intensity follows the crossing density and the ruggedness.
  - Segments are renewed every 100 ms. Each starts and ends at zero amplitude: *"Start and end a waveform at zero
    amplitude whenever possible"* [verified: A17].
- **Stop rules:**
  - Stop within 50 ms when pen speed drops below 5 mm/s [verified: R4 for the threshold].
  - Stop when no hover event has arrived for 100 ms. Godot does not report hover exit (§4.2).
  - Stop on stroke end.
  - Always call `cancel()` when the pen leaves: Windows' rule, *"always call StopFeedback explicitly"* [verified: K9].
- **Call rate:** at most 20 vibrator calls per second, whatever the mode.

**The capability ladder**, probed once at start-up and shown in the tester (SH-9):

| Rung | Test | Ticks | Texture |
|---|---|---|---|
| 1 | `areAllPrimitivesSupported(TICK, LOW_TICK, CLICK)` (API 31+: `PRIMITIVE_LOW_TICK` is level 31) | `Composition` primitives with scales 0.5 / 0.7 / 1.0 | `BasicEnvelopeBuilder` if `areEnvelopeEffectsSupported()` (API 36); else rung 2's waveform |
| 1b | `performHapticFeedback(SEGMENT_FREQUENT_TICK / SEGMENT_TICK)` (API 34); `CLOCK_TICK` (API 21) is the older texture constant (§3.2) | ordinary / index contour; no permission; honours the system touch-feedback setting | — |
| 2 | `hasAmplitudeControl()` | `createWaveform([0, 12], [0, a])` with a = 128 / 180 / 255 | `createWaveform` with an amplitude ramp |
| 3 | on/off only | fixed 10–12 ms pulses; strength coded by doubling (two pulses 60 ms apart = strong) | a 25 Hz on/off train |
| 4 | `hasVibrator()` is false, or the user turned vibration off. On the Pad 3 this may be the only rung (§1.4) | optional audio click per contour, and a brief brightening of the crossed line | — |

**How it degrades:**
- **On the Pad 3 without a vibrator** (§1.4): rung 4 only. The relief is the optional click per contour (Q11) and the
  brightened line. The pen's own motor stays out of reach (§2.3; Q10, Q13).
- **On the OnePlus 6T**, the same ladder applies. Its rung is unknown until SH-0 runs on it.
- **The desktop** has no vibrator path. The shell's `OS.has_feature("mobile")` guard stays.
- **A browser** gets rung 3 at best (§5.3).

### 7.8 Settings, battery and accessibility

**Settings** [judgement]. A *Pen & haptics* group sits beside the mockup's Touch & gestures rows. The mockup already
draws *"Haptic tick on holds and snaps"* and *"Pen draws, finger pans"*. No haptics or pen setting exists in
`dcc_settings.gd` today [verified: grep].

| Setting | Values | Default |
|---|---|---|
| Feel the relief | off / on | Q2 |
| Strength | low / medium / high (global multiplier 0.5 / 0.7 / 1.0) | medium |
| Contour interval for feel | follow the map's contours / ×2 / ×5 / custom metres | follow the map |
| Tools | colour paint, WORLD paint, sculpt, hover (each a toggle) | Q3, Q5 |
| Water and coast cues | on / off | on |
| Texture | on / off | on |
| Sound when no vibration | on / off | off (Q11) |

**Battery.**
- Android's power profile has no vibrator entry. The AOSP `power_profile.xml` and its documentation list
  `battery.capacity`, `screen.on` and others, but none for the vibrator [verified: A27]. So battery statistics cannot
  attribute vibration by themselves; the battery bar is measured as an on/off difference (§7.9).
- An estimate to size it [estimate]. Assume ticks at 10/s × 12 ms, a 12 % duty cycle. At an assumed 50–150 mA motor
  current, that is 6–18 mAh per hour of continuous drawing: 0.05–0.15 % of 12 140 mAh. The current is not published;
  only the arithmetic is offered. For scale, the example motor in §7.3 draws a typical 67 mA [verified: H1]; at the
  same 12 % duty that is about 8 mAh per hour, 0.07 % of 12 140 mAh [estimate].
- The larger cost may be the CPU work and the extra motion events once accumulation is off. SH-2 measures both.
- Under Android's battery saver, drop texture mode and keep contour ticks [judgement].

**Accessibility.**
- The cues are redundant: the contour lines, the coast and the hover ring are already drawn [judgement].
- Haptics must never be the only carrier of information. This follows Android's guidance to co-design visual, audio
  and haptic effects [verified: A16].
- AOSP's shell help says that for the shell's own test vibrations the *"touch feedback strength user setting will be
  used to scale amplitude"* [verified: A26]. Which system intensity setting scales Cartalith's own vibrations depends
  on how they are played, and is not established here [unconfirmed]. Cartalith's strength setting multiplies on top of
  whichever applies.

### 7.9 Acceptance bars

| Bar | Target | How to measure |
|---|---|---|
| Software latency, motion event → vibrator call | median ≤ 15 ms, max ≤ 30 ms over 500 events | timestamps in the scheduler (`Time.get_ticks_usec()` at event arrival and before the call), on the device, printed as median (min..max) |
| End-to-end latency, pen crossing a drawn contour → felt tick | median ≤ 50 ms, every sample ≤ 70 ms [R1] | a 240 fps slow-motion video showing the pen tip, the contour line and a small square the app flashes on the frame it calls the vibrator. Motor onset can be read from a second phone's accelerometer logger placed on the tablet. Report the median with min..max of at least 30 crossings |
| Discrete tick rate | never more than 15/s; no two primitives closer than 50 ms | the scheduler's log of 60 s of fast strokes over mountains |
| Crossings felt (discrete regime) | ≥ 95 % of crossings scheduled within 50 ms, **and** ≥ 90 % agreement in a blind count | software: the log of crossings against calls. Human: 10 slow strokes over a test profile with 6 known crossings, eyes closed; count ticks against the truth |
| Coast and crest cues | 10 of 10 coast crossings and 10 of 10 crests on a test island produce their cue | the log, plus the owner's count |
| Silence | 0 vibrator calls over water, with "Feel the relief" off, or with the system's touch feedback off | the log |
| Battery | ≤ 1 % of battery per hour of continuous drawing attributable to relief | 30 min scripted strokes with relief on, then off, from the same charge level. The difference in `dumpsys battery` charge counter, or the platform's battery usage screen |
| Frame cost | relief adds ≤ 1 ms median to the frame on the Pad 3 | Godot's frame-time monitor, relief on versus off, three runs each (median with min..max) |

**The end-to-end bar may be out of reach on this tablet.** One review reports 41.9 ms of *"total touch latency"* on
the Pad 3, without saying whether a finger or the pen was used [secondary: S17]. If the pen's input latency is near
that, a 50 ms median leaves under 10 ms for the scheduler, the vibrator call and the motor's own start [estimate:
50 − 41.9 = 8.1 ms]. So SH-0 measures the pen's input latency first (§8.2). The bar is then read again with that
number in hand; until then it stands as written [judgement]. The vibration bars apply only to a device that has a
vibrator (§1.4).

---

## 8. A build plan for the Godot shell

### 8.1 Milestones

"Needs" uses five kinds of change:
- **Config**: a project setting or the export preset.
- **GDScript**: shell scripts only.
- **JCW**: GDScript with `JavaClassWrapper`, no Gradle.
- **Rust**: a `cartalith-godot` bridge change.
- **Plugin**: Kotlin plus the Gradle build.

Every milestone keeps the project's standing rules:
- annotate the code (Ruling BK);
- never encode "no value" as a plausible value;
- parse-check every touched `.gd` and `shell/app.gd`;
- `cargo test --workspace` for any Rust;
- verify on the device: `--force-touch` on a desktop is not the tablet (`MISTAKES.md`).

| ID | Milestone | Needs | Numeric bars |
|---|---|---|---|
| **SH-0** | **Measure the device** (§8.2). First, whether there is a body vibrator (`cmd vibrator_manager list`): the answer decides how much of §7 the Pad 3 can use. Then density, size, Android level, pen axes, ranges and rates, hover, the pen's input latency, any key from the pen's double tap, what a resting palm does, and the vibrator's capabilities; the same on the 6T. Write a device sheet under `cartalith-native/docs/evidence/stylus-haptics-<date>/` | nothing built | every row of §8.2 filled or marked "not reported by the device" |
| **SH-1** | **Make the shipped haptics play**: `permissions/vibrate=true` in the Android preset. The preset is edited only under the owner's scoped authorisation; `ANDROID_BUILD_SCOPE.md` §1.3 records that precedent for `exclude_filter` | Config | logcat shows 0 *"VIBRATE permission not found"* lines over 20 holds; `dumpsys vibrator_manager`'s history lists Cartalith's vibrations |
| **SH-2** | **Input hygiene**: `disable_scroll_deadzone=true`; `Input.use_accumulated_input = false` while a pen stroke or hover feel is active. Tell pen from finger by event class and device: an Android pen is an `InputEventMouse*` with device ≥ 0 (Godot's Android handler never sets the device, so it stays at the default 0 [verified: G1, `input_event.h`]); a finger is an emulated mouse with device −1 or a `ScreenTouch`. That fixes a likely `_stroke_pen_seen` misfire: ET-D2 measured that touch emulation passes the drag's own pressure through, and on Android that is the finger's `getPressure(i)` (§4.2), so a finger reading strictly between 0 and 1 would latch as a pen. The Pad 3's finger values are unmeasured (§9). Check the rule against a docked keyboard's touchpad, a third pointing device (LineageOS's Pad 3 files include `pogo_touchpad.idc` [verified: L2]) [judgement]. A pen that became a touch index carries pointer 0's tilt, not its own (§4.2) | Config + GDScript | the first sample arrives within 1 dp of the press; samples reaching GDScript ≥ 90 % of the display rate while drawing; 0 of 50 finger strokes classed as pen; 0 of 50 pen strokes classed as finger |
| **SH-3** | **Pen draws, finger pans** (the mockup's row, `MOBILE_SHELL_PROPOSAL.md` §4; Ruling EC). Once a pen is seen: fingers navigate only; the pen acts at once. The palm rule ignores touches during pen contact and for 300 ms after, and while the pen hovers. ✋ is hidden in this mode (§2.5). Also covers a palm landing first. The pen may then become a second touch, the palm may drive the emulated mouse, and the stroke may pan or zoom the map through Godot's gestures (§4.2), unless Android cancels the palm first; SH-0's palm probe says which | GDScript | 0 dabs from 30 palm-rest trials; pinch and pan work with the pen lifted; 0 strokes lost to a palm, and 0 pans or zooms, in 30 trials of palm first, then pen |
| **SH-4** | **Pressure curve and calibration**: soft / linear / firm (γ 0.6 / 1.0 / 1.6) and a "comfortable maximum" scale. Applied in `world_workspace.gd` before `sculpt_add_point_p` and `paint_stroke_sample`; the engine's law is unchanged | GDScript | unit table of literals: firm maps 0.5 to 0.33 (0.5^1.6 = 0.33, [estimate]); a mutated γ fails the test |
| **SH-5** | **Tilt to the brush**: expose `tilt_size` / `tilt_min` in `paint_stroke_options`, driving `Sensor::TiltElevation`; pressure-to-radius for colour paint, if the owner wants it (Q12) | Rust (+ GDScript toggles) | a literal-pinned dab size at tilt sin 60° = 0.866; `golden_parity_paint` unmoved with tilt off |
| **SH-6** | **Hover cursor and footprint**: extend the brush ring that already follows the cursor at the current radius (`tool_overlay.gd::set_brush_cursor`) with an inner ring at `pressure_min`, and hide it 150 ms after the last hover event (today it stays where the pen left) | GDScript | ring on screen in the frame after the first hover event; gone ≤ 150 ms after the pen leaves |
| **SH-7** | **Haptic backend**: `haptic_out.gd` with the capability probe and play API (§7.7). Move the `_HAPTIC_MS` kinds to primitives or constants where supported, with the ms table as the fallback (Q6) | JCW (Rust JNI if too slow) | JNI call cost median ≤ 0.5 ms on the Pad 3 [judgement]; the probe's capability readout matches `dumpsys vibrator_manager` |
| **SH-8** | **Haptic relief**: `haptic_bridge.rs` + `haptic_relief.gd` (§7.4–§7.7) | Rust + GDScript + JCW | every row of §7.9 |
| **SH-9** | **Stylus tester and calibration screen**, in the spirit of Krita's tablet tester: a live stream of rate (Hz), pressure histogram, tilt vector, hover, device id and button mask, key events (`keycode`, `physical_keycode`), and pan or magnify gestures; the pressure-curve preview; one button per haptic pattern; the capability sheet | GDScript + JCW | rates within 10 % of `getevent`'s; every capability field matches dumpsys |
| **SH-10** | *(optional)* **Raw input plugin**: a Kotlin v2 plugin listening on `godot.renderView.view` for batched samples, `AXIS_DISTANCE` (only in or out of range on this tablet, §2.2), `TOOL_TYPE_ERASER`, stylus buttons, keys above F12 and `FLAG_CANCELED`, plus `MotionEventPredictor` for the cursor only; signals to GDScript. **Or** upstream Godot fixes for the same gaps (§4.3 issues) | Plugin (Gradle) or upstream | samples reaching GDScript = Android's samples (getevent) ± 5 %; 0 eraser contacts drawn as ink |
| **SH-11** | *(optional, not recommended before SH-2's numbers)* front-buffered wet-ink overlay for the paint preview | Plugin | stroke-preview latency halved against SH-2's measured median |
| **SH-12** | *(blocked)* **pen-tip haptics** through OnePlus's pen SDK, only if OnePlus grants access (Q10). An unofficial Bluetooth route is Q13, default no. On a Pad 3 without a body vibrator this is the only route to a felt relief (§1.4) | Plugin + vendor SDK | — |

**Order** [judgement]: SH-0 → SH-1 → SH-2 → SH-9 (the tester is the measuring tool for the rest) → SH-3 → SH-4 →
SH-6 → SH-7 → SH-8 → SH-5. SH-10–SH-12 come after an owner decision.

**If SH-0 finds no vibrator on the Pad 3** (the list prints *"No vibrator found"*, §1.4) [judgement]:
- SH-1 still ships. It makes the shell's haptics play on the 6T and on any device with a motor; on the Pad 3 there is
  nothing for it to drive.
- SH-7's probe reports rung 4 on the Pad 3, and SH-8's relief there is sound and sight (§7.7): the click per contour
  (Q11) and the brightened line. The vibration parts of SH-7 and SH-8 are built and measured on the 6T, and the
  vibration bars of §7.9 are measured there.
- The pen's motor becomes the only actuator on the Pad 3. It needs OnePlus (Q10, SH-12) or an unofficial Bluetooth
  route (Q13, default no).
- Q2's recommended default already waits for SH-0.

The open CM-4 row in `OUTSTANDING_WORK.md` (*"CM-4 device check: the pen barrel button and haptics on the owner's
tablet"*) is answered in part by this research:
- The Stylo 2 has no barrel button [secondary: S8, S6].
- Godot 4.7.2 would report one as button index `NONE`, not `MOUSE_BUTTON_RIGHT` [verified: G1; the withdrawn
  #124191 says the same].
- Its double-tap area may send a key (§2.2). If it does, Godot reports the key as `KEY_UNKNOWN` (§4.2). SH-0's
  `getevent -lt`, with the pen's presentation mode off and then on, settles it.
- The haptics are silent until SH-1, and the Pad 3's body cannot play them at all if it has no vibrator (§1.4).

### 8.2 Measurement plan on the real tablet

Run each over `adb` from the PC, with the APK installed. Every command below is standard Android tooling. The two
`cmd vibrator_manager` forms are quoted from AOSP's shell help [verified: A26]. Output formats vary by OEM.

| What | Command | What to record |
|---|---|---|
| **Body vibrator present? Run this first** | `adb shell cmd vibrator_manager list` | one id per line, or *"No vibrator found"* when there is none [verified: A26]; input-device vibrators are excluded by design. If none, §8.1's *If SH-0 finds no vibrator* applies, and the vibrator rows below are run on the 6T |
| Model, build | `adb shell getprop ro.product.model`, `getprop ro.build.version.release`, `getprop ro.build.version.sdk`, `getprop ro.build.display.id` | model, Android version, API level, OxygenOS build |
| Density, size | `adb shell wm density`, `adb shell wm size`, `adb shell getprop ro.sf.lcd_density` | physical and override density; pixels; then §1.3's row |
| Input devices | `adb shell dumpsys input` | the pen's device name, sources (`STYLUS`, `BLUETOOTH_STYLUS`?), and the ranges of `PRESSURE`, `TILT`, `DISTANCE`, `ORIENTATION`; whether the pen and the fingers are separate devices (LineageOS's driver registers the pen as its own, `…_pen` [verified: L6]); whether the pen also appears as a Bluetooth input device (§2.1) |
| Raw pen events | `adb shell getevent -lt` while drawing and hovering | `ABS_PRESSURE`/`ABS_MT_PRESSURE` range, `ABS_TILT_X/Y`, `ABS_DISTANCE`, `BTN_TOOL_PEN`/`BTN_TOOL_RUBBER`, `BTN_STYLUS*`; the report rate in Hz (events ÷ seconds); whether hover produces events |
| Pen axes and ranges | `adb shell getevent -lp` | the pen device's `ABS_PRESSURE`, `ABS_TILT_X/Y` and `ABS_DISTANCE` ranges. A 0–1 distance means hover is in or out of range only, as in LineageOS's driver (§2.2). Coordinates are the panel's own, natively portrait (§1.2) |
| Pen double tap | `adb shell getevent -lt` while double-tapping the pen's touch area, first with the pen's presentation mode off, then on | any `KEY_*` line and which device sent it (another OnePlus pen sends F21, §2.2). No line at all is also an answer |
| Pen input latency | a 240 fps video of fast pen strokes in SH-9's tester, or any app that draws at once: the gap between the tip and the end of the drawn line, divided by the pen's speed | median (min..max) of at least 10 strokes. It includes drawing time, so it bounds the input latency from above. Compare with the review's 41.9 ms touch figure (§1.2) |
| Vibrator capabilities | `adb shell dumpsys vibrator_manager` | `capabilities`, `supportedEffects`, `supportedPrimitives`, `compositionSizeMax`, `primitiveDelayMax`, `q-factor`, `frequencyProfile`, envelope limits (the keys `VibratorInfo.dump` prints [verified: A26]); recent vibration history |
| Feel the primitives | `adb shell cmd vibrator_manager synced -f primitives 7` (TICK), `… 8` (LOW_TICK), `… 1` (CLICK) | felt or not, on the desk and in the hand |
| Gaps | `… synced -f primitives 7 -w 30 7`, then `-w 50` and `-w 80` (two TICKs in one composition) | whether two ticks are felt at each gap |
| Retrigger | two separate `… synced -f -B primitives 1` calls in quick succession (`-B` runs in the background) | whether the second call cuts the first short |
| Haptics silent today | `adb logcat \| grep -i "VIBRATE permission"` during holds on the map | the *"VIBRATE permission not found"* line, before and after SH-1 |
| Battery | `adb shell dumpsys battery` (charge counter) before and after a 30 min scripted session, relief on and off | mAh per hour difference |
| Godot's view of the pen | SH-9's tester; until it exists, a probe scene printing every `InputEvent` class, device, pressure, tilt and button mask, every `InputEventKey` (`keycode`, `physical_keycode`, `pressed`), and every `InputEventPanGesture` and `InputEventMagnifyGesture` | the hover events, eraser, button behaviour, any key from the double tap (expected as `KEY_UNKNOWN`, §4.2) and finger pressure values on this tablet |
| Palm first, then pen | the same probe, while drawing with a palm already resting on the glass; 30 trials | whether Android cancels the palm (AOSP's default, §4.2), whether the pen arrives as a mouse or as a touch index, and whether any stroke pans or zooms |

A browser test page offered with the supplement was not adopted: it shows Chrome's view, not Godot's, and its move
logging is broken (*Supplement merge, 2026-11-02*).

**Notes from this tree's own device work** [verified: `ANDROID_BUILD_SCOPE.md`]:
- On the 6T's Android 15 build, *"SurfaceFlinger `--latency` returns nothing … and `gfxinfo` counts the HWUI view, not
  Godot's GL surface"*. Latency must be measured from video or in-app timestamps, not from those tools.
- Probe APKs must not be launched through command-line arguments.

### 8.3 Owner questions, with recommended defaults

**Answered 2026-11-02 by Ruling ED** (`LARGE_ITEM_RULINGS.md`): Q1 "Pad 3 + Stylo 2"; Q7 "Yes, enable it" (the
Android preset now carries `permissions/vibrate=true`, not yet seen on a device); Q9 *"Whichever gives the best and
most extensive feature set"*, read as accepting the Gradle build and the Kotlin input plugin (SH-10); Q13 "Yes, try
it", as a read-only discovery experiment first, against the recommendation. The other nine keep their defaults until
the owner says otherwise. The table below is kept as the questions were asked.

| # | Question | Recommended default |
|---|---|---|
| Q1 | Which tablet and pen do you have: OnePlus Pad 3 with the Stylo 2, or the newer Pad 4 with the Stylo Pro? | Pad 3 + Stylo 2 |
| Q2 | Should "Feel the relief" be on by default? | Off until SH-0 shows the Pad 3's vibrator is worth using; then on for drawing tools. The Pad 3 may have no vibrator (§1.4); then it stays off there, and Q11 applies |
| Q3 | Which tools get relief? | Colour paint, WORLD paint, sculpt. Not Way, Territory or Region |
| Q4 | Which contour interval drives the ticks? | The map's own interval (`contour_m`, or 5 % of relief when unset), felt even when the lines are not drawn, stepped up by zoom (§7.5) |
| Q5 | Should hovering feel the relief, as a preview? | Yes: contours and coast only, at the lowest strength; no texture |
| Q6 | Should the shipped one-shot table (`_HAPTIC_MS`, from `BUILD_ANSWERS.md` §4) move to Android's primitives and constants where the device supports them, with the same meanings? | Yes, with the ms table kept as the fallback |
| Q7 | May the Android export preset enable the VIBRATE permission (a scoped edit)? | Yes |
| Q8 | Should a still pen held for 500 ms open the cartwheel and card, as a finger does? | No while *Pen draws, finger pans* is on: the finger hold opens them |
| Q9 | Accept a Gradle build for a Kotlin input plugin (SH-10), pursue upstream Godot fixes, or neither for now? | Neither for now; decide after SH-0–SH-9 |
| Q10 | Ask OnePlus for access to the pen's haptics? | Optional; low odds. If SH-0 finds no body vibrator, this is the only sanctioned route to a felt relief on the Pad 3 (§1.4) |
| Q11 | Offer a sound per contour when the device cannot vibrate? | Offered in settings, off by default. On a Pad 3 without a vibrator it is the whole relief besides the drawn lines (§1.4); RealPen pairs a pen's feel with sound [verified: R12] |
| Q12 | Should colour paint's stroke width follow pen pressure? | No for now: colour paint marks have one radius in km, and variable width is a bigger change to its data |
| Q13 | May Cartalith talk to the pen's own motor over Bluetooth, bypassing OnePlus's service? **The premise is unconfirmed**: it comes from an AI-written project, which says the pen takes a texture setting and a one-shot pulse command (§2.3). The route would need `BLUETOOTH_CONNECT`, a dangerous runtime permission; by the same account it sits beside the pen's firmware-update services; it could clash with the system's own connection; and it would give a fixed texture and one-shot pulses, not a texture that follows the terrain | No |

---

## 9. Limits of this research

- **No device was used.** Every statement about Cartalith's Android behaviour comes from reading the shell, the engine
  bridge and Godot 4.7.2's source, not from running it.
- **Not established** (each is a row of §8.2):
  - the Pad 3's density;
  - whether it has a vibrator at all (LineageOS's build ships none, §1.4), and if so its type, amplitude control and
    primitives;
  - the height at which the Stylo 2's hover starts (the driver reports only in range or not, §2.2), its report rate,
    its input latency (one review gives 41.9 ms of touch latency, method unstated), its protocol vendor, and whether
    it has an eraser end;
  - whether the Stylo 2's double tap produces a key event, and which (§2.2);
  - whether the Pad 3 reports the pen and the fingers as one input device (LineageOS's driver: two), and whether
    OxygenOS cancels a resting palm when the pen comes near, as AOSP does by default (§4.2);
  - whether stock OxygenOS caps the refresh rate at 120 Hz while the pen is connected, as LineageOS's build does
    (§1.2);
  - whether OxygenOS flags palms with `FLAG_CANCELED`;
  - the finger pressure values the Pad 3 reports;
  - whether a new vibration cancels a running one;
  - the cost of a `JavaClassWrapper` call;
  - the motor's current draw.
- **Not opened:**
  - the XDA thread about the Stylo 2's haptics (HTTP 403);
  - Klatzky et al. 2014 (HTTP 503, then a certificate error);
  - the ACM pages (HTTP 403; the Tennison paper was read from NSF PAR instead), among them those of Kim et al. 2013 and
    Giudice et al. 2012;
  - the Palani and Giudice 2014 PDF on spatial.umaine.edu (a TLS failure, then HTTP 503);
  - the PhoneArena page on the Pad 2's haptics (HTTP 403);
  - the OnePlus community threads (rendered by script);
  - Choi and Kuchenbecker 2013, TouchOver map and Walker and Salisbury 2003 (bibliographic records only);
  - Wacom's guidance.
- **AI-written sources, named and not used as evidence.** A Pad 2 pen-protocol project (S21) and a Chrome hover repro
  (*Supplement merge, 2026-11-02*) say they were written with an AI. Every claim that rests on them alone is
  [unconfirmed]. LineageOS's pen-relay commits are marked `Assisted-by: LLM`, so their messages are [unconfirmed]
  too; the code itself is cited as code.
- **Unconfirmed device claims.** The Pad 4's exact resolution, the Stylo 2's model number, 240 Hz and its latency
  figures rest on search excerpts or listings. The Pad 3's model number rests on S16 and LineageOS's device list (L1),
  not on OnePlus. The Pad 3's OxygenOS 16 update and
  the Pad 4's launch rest on one news piece each (S16, S15).
- **Unconfirmed absence.** No map app with haptic relief was found; that is a negative search result.
- **The design in §7 is a judgement built on the cited limits.** Its thresholds (slope bins, 150 ms, 300 ms palm
  window, ½ `iv` hysteresis, the interval steps) are starting values to tune on the device. They are not measurements.
- **Deep zoom.** The relief samples the base height field. At deep zoom the drawn relief includes synthesised detail
  from the LOD tiles, so the feel and the picture can differ until a later step samples tile heights.
- **Side finding, not acted on.** `crates/cartalith-godot/src/vault_saf.rs` says in its module doc *"There is no JNI
  binding anywhere in this workspace"*. `llm_keystore_android.rs` uses raw JNI, so that sentence is stale. It is
  reported here, not edited.
- **Side finding, not acted on: stale pen prose.** `TABLET_UI_SPEC.md` §4.2 says *"nothing in the shell reads pen
  pressure"*, and `EDITING_TOOLS_SCOPE.md` §3.2 says `mm.pressure`, `mm.tilt` and `mm.pen_inverted` *"are never
  read"*. Both predate ET-D2, ET-D3 and ET-D5: `map_overlay.gd::_emit_stroke_sample` reads all three, and
  `world_workspace.gd::_sculpt_take_pressure` exists [verified: both symbols, 2026-11-02]. Reported here, not edited
  (Ruling BK).
- **Side finding, not acted on: `STATUS.md`'s CM-4 row contradicts itself.** Its title says *"pen barrel button
  verified"*; its body says *"Pen barrel button: wired but not device-verified"*, and assumes that a barrel button
  *"already arrives as `MOUSE_BUTTON_RIGHT`"*. Godot 4.7.2 reports a stylus button as index `NONE` (§4.2), and the
  Stylo 2 has no barrel button (§2.2). Reported here, not edited.

---

## Sources

Accessed 2026-11-01 unless noted. Downloads were kept in the session's scratch folder, outside the repository.

**Device and pen**
- **S1** OnePlus Pad 3 specifications (global), https://www.oneplus.com/global/oneplus-pad-3/specs: every display,
  battery, OS and dimension figure in §1.2; the embedded specification JSON was printed in full.
- **S2** OnePlus Pad 3 product page, https://www.oneplus.com/global/oneplus-pad-3: Stylo 2 *"16,000 levels of
  pressure / Ultra-low latency / Haptic feedback / Laser pointer with page turn functions"*; footnotes on 12-bit, 144
  Hz and battery; *"sold separately"*.
- **S3** OnePlus Pad 2 product page, https://www.oneplus.com/global/oneplus-pad-2: *"subtle vibration feedback"*,
  *"Page turn / Laser pointer"*.
- **S4** Parka Blogs, *Artist Review: OnePlus Pad 3 with OnePlus Stylo 2*, 2025-07-19,
  https://www.parkablogs.com/content/oneplus-pad-3-2025-artist-review.
- **S5** Parka Blogs, *Artist Review: OnePlus Pad 2 with OnePlus Stylo 2*, 2024-08-15,
  https://www.parkablogs.com/pictures/oneplus-pad-2-artist-review.
- **S6** MyNextTablet, *OnePlus Pad 3 Review*, 2025-07-02, https://mynexttablet.com/oneplus-pad-3-review/.
- **S7** The Gadgeteer, *OnePlus Pad 3 review*, 2025-06-30,
  https://the-gadgeteer.com/2025/06/30/oneplus-pad-3-review-this-android-tablet-gives-you-a-lot-of-bang-for-your-buck/.
- **S8** HotHardware, *OnePlus Pad 3 Review*, https://hothardware.com/reviews/oneplus-pad-3-hands-on.
- **S9** Tom's Guide, *I've reviewed the OnePlus Pad 3*, 2025-06-05,
  https://www.tomsguide.com/computing/android-tablets/oneplus-pad-3.
- **S10** Android Central, *Does the OnePlus Pad 2 support stylus input?*,
  https://www.androidcentral.com/tablets/does-the-oneplus-pad-2-support-stylus-input.
- **S11** The Business Standard, *OnePlus Pad 2: Simulating 'writing on paper' with a vibrating stylus*, 2024-07-27,
  https://www.tbsnews.net/tech/oneplus-pad-2-simulating-writing-paper-vibrating-stylus-904331.
- **S12** GitHub issue AmpleReVanced/revanced-patches#180 (2026-09-26, about a patched StarNote build), citing an XDA
  thread (https://xdaforums.com/t/magisk-module-app-oneplus-stylo-2-global-haptic-feedback.4758335/, not opened:
  HTTP 403).
- **S13** Search-result summaries, not opened as pages:
  - the Pad 3's OxygenOS 16 rollout (croma.com, smartprix.com, gizmochina.com, tuttoandroid.net);
  - the Pad 4's launch (smartprix.com, gsmarena.com, mysmartprice.com);
  - Stylo 2 retailer listings (amazon.co.uk, croma.com, cellularkenya.co.ke, ennap.com);
  - the 290 Hz movement-rate figure (androplus.jp);
  - Pad 3 density searches (none found).
- **S14** OnePlus Community forum posts on the Stylo 2's double-tap (search excerpts only).
- **S15** Android Authority, *OnePlus Pad 4 launched*, 2026-04-30,
  https://www.androidauthority.com/oneplus-pad-4-launched-3662236/: *"a 13.2-inch display"*, *"3.4K resolution and a
  144Hz refresh rate"*, the *"Stylo Pro"* with *"up to 16,000 pressure points"* and *"swappable tips"*, *"currently
  limited to India"*. Opened by the fact-check, 2026-11-02 UTC.
- **S16** Android Headlines, *OnePlus Pad 3 & Pad 2 Are Receiving Android 16 with OxygenOS 16*, 2025-11-10,
  https://www.androidheadlines.com/?p=1248073: Pad 3 build *"OPD2415_16.0.0.211 (EX01)"*, rollout *"hitting in
  India"*. Opened by the fact-check, 2026-11-02 UTC.
- **S17** AndroPlus, *OnePlus Pad 3 review* (English edition, dated 2025/7/20),
  https://androplus.jp/en/entry/oneplus-pad-3-tablet-review/: the WALT touch latency, the 290 Hz movement rate, the
  per-app refresh choices and the back vibrating at volume. Read through WebFetch by the verifier and by this merge;
  `curl` received a Cloudflare HTTP 403 page.
- **S18** Notebookcheck, *OnePlus Pad 3 tablet review*,
  https://www.notebookcheck.net/Top-tablet-finally-available-worldwide-OnePlus-Pad-3-tablet-review.1097828.0.html:
  brightness and touch scanning. It says nothing about vibration.
- **S19** The Tech Outlook, *OnePlus Pad 2 Pro (OPD2413) launched in China…*, 2025-05-14,
  https://www.thetechoutlook.com/new-release/gadgets-release/oneplus-pad-2-pro-opd2413-launched-in-china-with-13-2-inch-3-4k-display-and-snapdragon-8-elite-soc/:
  *"the OnePlus Smart Stylus Pen Pro priced at ¥499"*. Opened by this merge, 2026-11-02.
- **S20** Krita Artists forum, *"My stylus's side button is not recognised by Krita"*, https://krita-artists.org/t/176767
  (read as JSON): started 2026-03-25, closed automatically by the forum on 2026-03-31, not withdrawn. Posts 1, 7 and 8
  are cited.
- **S21** ardyln/oneplus-pad2-stylo2, https://github.com/ardyln/oneplus-pad2-stylo2: README, `docs/PROTOCOL.md`,
  `docs/RESEARCH.md`, `docs/STYLO2.md`, `tools/research/getevent-lp-caihong.txt`. **AI-written by its own account**:
  the README says *"Most of the code and docs were written with an AI assistant (Claude), under the author's
  direction"*, and all 60 of its commits carry a Claude co-author line. Not evidence; every claim resting on it alone is
  [unconfirmed: AI-written].

**LineageOS** (each read at the commit named, 2026-11-02; a device tree describes LineageOS's build, not the hardware,
and not stock OxygenOS)
- **L1** `LineageOS/lineage_wiki` at `bf49224`: `_data/devices/erhai_variant1.yml` (*'Pad 2 Pro'*, OPD2413) and
  `erhai_variant2.yml` (*'Pad 3'*, OPD2415), both `tree: android_device_oneplus_erhai`.
- **L2** `LineageOS/android_device_oneplus_erhai`, branch `lineage-24.0` at `3005381`: `device.mk`
  (`TARGET_IS_TABLET := true`, the `OplusPen` package, inherits `sm8750-common`), `proprietary-files.txt`,
  `properties/vendor.prop`, `overlay/OPlusPenResTarget/res/values/config.xml`.
- **L3** `LineageOS/android_device_oneplus_caihong` (the Pad 2), branches `lineage-24.0` and `lineage-23.2`, both at
  `04eea88`.
- **L4** `LineageOS/android_device_oneplus_dodge` (OnePlus 13, `lineage-24.0`, `769d8bf`) and
  `android_device_oneplus_waffle` (OnePlus 12, `lineage-24.0`, `bcaf432`): the `# Vibrator` blocks of `device.mk`.
- **L5** `LineageOS/android_device_oneplus_sm8750-common` (`lineage-24.0`, `96d2864`): `common.mk`, `modules.load`;
  `android_device_oneplus_sm8650-common` (`lineage-23.2` and `lineage-24.0`, `e84caeb`): `common.mk`'s
  `TARGET_IS_TABLET` guards.
- **L6** `LineageOS/android_kernel_oneplus_sm8750-modules` (`lineage-24.0`, `e05c031`):
  `oplus/kernel/touchpanel/oplus_touchscreen_v2/touchpanel_common_driver.c` (the pen input device, its axes and its
  name) and `Novatek/NT36536_noflash/nvt_drivers_nt36536_noflash.c` and `.h` (`nvt_enable_pen_vibrator`).
- **L7** `LineageOS/android_hardware_oplus` (`lineage-24.0`, `eaa32c0`): `packages/Pen` (`PenService.kt`,
  `PenProtocol.kt`, `res/values/config.xml`). The commits that add the pressure relay and the refresh-rate cap
  (`6b36209`, `930a82c`, `38c7a90`) are marked `Assisted-by: LLM`, so their messages are [unconfirmed]; the code is
  cited as code.
- **L8** `LineageOS/charter` (`1e4201d`): `device-support-requirements.md`; a grep for "vibrat" and "haptic" finds
  nothing.

**Android**
- **A1** `MotionEvent` reference, https://developer.android.com/reference/android/view/MotionEvent.
- **A2** `View` reference, https://developer.android.com/reference/android/view/View.
- **A3** `View.OnTouchListener` reference, https://developer.android.com/reference/android/view/View.OnTouchListener.
- **A4** `InputDevice` reference, https://developer.android.com/reference/android/view/InputDevice.
- **A5** `KeyEvent` reference, https://developer.android.com/reference/android/view/KeyEvent (`KEYCODE_F13` to
  `KEYCODE_F24`, *"Added in API level 36"*, read again 2026-11-02).
- **A6** `Vibrator` reference, https://developer.android.com/reference/android/os/Vibrator.
- **A7** `VibratorManager` reference, https://developer.android.com/reference/android/os/VibratorManager.
- **A8** `VibrationEffect` reference, https://developer.android.com/reference/android/os/VibrationEffect.
- **A9** `VibrationEffect.Composition` reference,
  https://developer.android.com/reference/android/os/VibrationEffect.Composition.
- **A10** `VibrationEffect.BasicEnvelopeBuilder` and `.WaveformEnvelopeBuilder` references (same base URL).
- **A11** `VibrationEffect.Builder` and `.Preset` references (same base URL).
- **A12** `HapticFeedbackConstants` reference, https://developer.android.com/reference/android/view/HapticFeedbackConstants.
- **A13** `MotionPredictor` reference, https://developer.android.com/reference/android/view/MotionPredictor.
- **A14** `InputMethodManager` reference, https://developer.android.com/reference/android/view/inputmethod/InputMethodManager.
- **A15** *Advanced stylus features*,
  https://developer.android.com/develop/ui/views/touch-and-input/stylus-input/advanced-stylus-features.
- **A16** *Haptics design principles*, https://developer.android.com/develop/ui/views/haptics/haptics-principles
  (updated 2026-10-26).
- **A17** *Create custom haptic effects*, https://developer.android.com/develop/ui/views/haptics/custom-haptic-effects.
- **A18** *Vibration actuators primer*, https://developer.android.com/develop/ui/views/haptics/actuators.
- **A19** *Haptics APIs*, https://developer.android.com/develop/ui/views/haptics/haptics-apis.
- **A20** androidx.graphics releases, https://developer.android.com/jetpack/androidx/releases/graphics.
- **A21** androidx.input releases, https://developer.android.com/jetpack/androidx/releases/input.
- **A22** androidx.ink releases, https://developer.android.com/jetpack/androidx/releases/ink.
- **A23** AOSP, *Implement constants and primitives*,
  https://source.android.com/docs/core/interaction/haptics/haptics-constants-primitives.
- **A24** AOSP, *Assess hardware*, https://source.android.com/docs/core/interaction/haptics/hardware.
- **A25** AOSP source (branch `main`): `core/java/android/view/GestureDetector.java`, `ViewConfiguration.java`
  (`TOUCH_SLOP = 8`, `DOUBLE_TAP_TIMEOUT = 300`, `DEFAULT_LONG_PRESS_TIMEOUT = 400`) and `ScaleGestureDetector.java`
  (`setStylusScaleEnabled`), https://android.googlesource.com/platform/frameworks/base.
- **A26** AOSP source (`main`): `services/core/java/com/android/server/vibrator/VibratorManagerService.java` (the
  `cmd vibrator_manager` help text) and `core/java/android/os/VibratorInfo.java` (`dump`). `runListVibrators` in the
  same file prints *"No vibrator found"* when there is none; read on `main`, `android15-release` and
  `android16-release`, 2026-11-02.
- **A27** AOSP, *Power values*, https://source.android.com/docs/core/power/values, and
  `core/res/res/xml/power_profile.xml` (`main`): no vibrator entry.
- **A28** `Manifest.permission` reference, https://developer.android.com/reference/android/Manifest.permission:
  `BLUETOOTH_CONNECT`, *"Added in API level 31"*, *"Protection level: dangerous"*.
- **A29** AOSP, *Haptics UX design*, https://source.android.com/docs/core/interaction/haptics/haptics-ux-design:
  `CLOCK_TICK` and `TEXT_HANDLE_MOVE` as constants *"designed for virtual haptic textures"*.
- **A30** AOSP source (`main`): `services/inputflinger/PreferStylusOverTouchBlocker.h` and `.cpp` (*"When stylus is
  down, all touch is ignored"*; touch is also rejected while a stylus hovers unless a flag disables it; a device that
  reports stylus and touch together is left alone) and `UnwantedInteractionBlocker.cpp` (with multi-device input on,
  the blocker is skipped, and *"hovering stylus rejects touch"* is that mode's default).

**Godot**
- **G1** Godot source at tag `4.7.2-stable` (commit `ed1daf0`), https://github.com/godotengine/godot:
  - `platform/android/java/lib/src/main/java/org/godotengine/godot/input/GodotInputHandler.java`,
    `GodotGestureHandler.kt`, `InputEventRunnable.java`;
  - `…/nativeapi/GodotNativeBridge.kt`, `…/utils/PermissionsUtil.java`, `…/Godot.kt`;
  - `…/plugin/GodotPlugin.java`, `AndroidRuntimePlugin.kt`, `…/variant/Callable.kt`;
  - `platform/android/android_input_handler.cpp`, `java_godot_wrapper.cpp`, `export/export_plugin.cpp`;
  - `core/input/input.cpp`, `core/input/input_event.cpp`, `core/input/input_event.h`.
- **G2** Godot class docs at the same tag: `doc/classes/Input.xml` (`vibrate_handheld`), `InputEventMouseMotion.xml`,
  `InputEventScreenDrag.xml`, `ProjectSettings.xml` (`input_devices/pointing/android/*`).
- **G3** Godot docs, *Integrating with Android APIs* (JavaClassWrapper, AndroidRuntime),
  https://docs.godotengine.org/en/stable/tutorials/platform/android/javaclasswrapper_and_androidruntimeplugin.html.
- **G4** Godot docs, *Godot Android plugins*,
  https://docs.godotengine.org/en/stable/tutorials/platform/android/android_plugin.html.
- **G5** GitHub search API for `repo:godotengine/godot` and `repo:godotengine/godot-proposals` (stylus, pen, tilt,
  hover, vibrate, haptic): issue and PR numbers, titles, states, dates and milestones as returned on 2026-11-01. The
  fact-check re-opened every issue and PR named in §4.2–§4.3 on github.com; #124191–#124193 were withdrawn by their
  author as AI-written.
- **G6** Godot source at `4.7.2-stable` (`ed1daf0`), read by the supplement and its verifier, 2026-11-02:
  `platform/android/android_keys_utils.h` and `.cpp` (`android_godot_code_pairs`, `godot_code_from_android_code`),
  `core/os/keyboard.cpp` (`fix_keycode`), `platform/android/android_input_handler.cpp` (`process_key_event`,
  `process_touch_event`), `GodotGestureHandler.kt` (`onScroll`), `InputEventRunnable.java` (`setTouchEvent`),
  `GodotGLRenderView.java`; and `android_keys_utils.h` on `master` at `95c0fa5` (the same table).

**Apps and platforms**
- **K1** Krita manual 5.3.0, *Tablet Settings*, https://docs.krita.org/en/reference_manual/preferences/tablet_settings.html.
- **K2** Krita manual, *Freehand Brush Tool*, https://docs.krita.org/en/reference_manual/tools/freehand_brush.html.
- **K3** Krita manual, *Options* and *Tablet sensors*,
  https://docs.krita.org/en/reference_manual/brushes/brush_settings/options.html and
  https://docs.krita.org/en/reference_manual/brushes/brush_settings/tablet_sensors.html.
- **K4** KDE bug 429940, https://bugs.kde.org/show_bug.cgi?id=429940.
- **K5** Krita, `3rdparty/ext_qt/0102-Android-Properly-handle-Tablet-events.patch` at commit `6fe29b52`,
  https://invent.kde.org/graphics/krita.
- **K6** Infinite Painter, *Settings*, https://docs.infinitestudio.art/painter/technical/settings/.
- **K7** Procreate, *Apple Pencil, Pressure and Smoothing*,
  https://help.procreate.com/articles/tIdR13-apple-pencil-pressure-and-smoothing.
- **K8** Apple developer documentation (JSON): `UICanvasFeedbackGenerator`, `UITouch.rollAngle`, `altitudeAngle`,
  `force`, `maximumPossibleForce`, `UIPencilInteraction.Squeeze`, `UIHoverGestureRecognizer.zOffset`,
  `UIEvent.coalescedTouches(for:)`, `predictedTouches(for:)`, https://developer.apple.com/documentation/uikit.
- **K9** Microsoft Learn, *Pen haptic feedback in Windows apps* (updated 2026-09-21),
  https://learn.microsoft.com/en-us/windows/apps/develop/input/pen-haptics.
- **K10** Microsoft Learn, *Pen interactions and Windows Ink in Windows apps*,
  https://learn.microsoft.com/en-us/windows/apps/design/input/pen-and-stylus-interactions.
- **K11** Samsung, *Air Actions* (S Pen Remote SDK), https://developer.samsung.com/galaxy-spen-remote/air-actions.html.
- **K12** This repository's `MAP_INTERACTION_RESEARCH.md` (2026-11-01), §3–§4.
- **K13** Krita manual, *General Settings*, https://docs.krita.org/en/reference_manual/preferences/general_settings.html:
  touch painting's *Auto* (5.2.13), long-press context menus (5.2.14), *Tilt Outline*, *Use effective outline size*.
- **K14** Drawpile: source on `main` at `2c2a616`, `src/desktop/dialogs/settingsdialog/tablet.cpp` (*"Translate F21 key
  to middle click (OnePlus)"*) and `metadata/en-US/changelogs/2030101001.txt`; F-Droid listing
  https://f-droid.org/en/packages/net.drawpile/ (version 2.3.1-beta.1).
- **K15** KDE GitLab: `szaman/qtbase` !14 *"Android OnePlus tablet handling"* (target `krita/5.15`) and `graphics/krita`
  !2726 *"[android] Enable OnePlus middle click workaround"* (target `master`), both merged 2026-03-27 by `dkazakov`;
  read through the GitLab API.

**Web**
- **W1** W3C, *Pointer Events Level 3*, Recommendation 30 June 2026, https://www.w3.org/TR/pointerevents3/.
- **W2** Microsoft Edge explainer, *Haptics* (archived),
  https://microsoftedge.github.io/MSEdgeExplainers/Haptics/explainer.html.
- **W3** Search excerpts on the CSS Working Group's haptics issue (w3c/csswg-drafts#13728) and a WICG repository.
- **W4** Chromium source on `main` (2026-11-02): `third_party/blink/renderer/core/events/pointer_event_factory.cc`,
  `ui/events/blink/blink_event_util.cc`, `input_event.mojom` (`kEraser` = 5), `web_pointer_properties.h`
  (`kEraser = 1 << 5`).
- **W5** MDN browser-compat-data on `main` at `0efedff8c8b6`: `api/PointerEvent.json`, `api/Navigator.json`,
  `api/Element.json`, `api/HTMLCanvasElement.json`.

**Research**
- **R1** Kaaresoja, T., Brewster, S., Lantz, V. (2014). Towards the temporally perfect virtual button: touch-feedback
  simultaneity and perceived quality in mobile touchscreen press interactions. *ACM Transactions on Applied Perception*
  11(2), 9. doi:10.1145/2611387. Abstract at https://eprints.gla.ac.uk/104653.
- **R2** Tennison, J. L., Uesbeck, P. M., Giudice, N. A., Stefik, A., Smith, D. W., Gorlewicz, J. L. (2020).
  Establishing vibration-based tactile line profiles for use in multimodal graphics. *ACM TAP* 17(2), 7.
  doi:10.1145/3383457. Full text at https://par.nsf.gov/servlets/purl/10204748.
- **R3** Bensmaïa, S. J., Hollins, M. (2003). The vibrations of texture. *Somatosensory & Motor Research* 20(1), 33–43.
  Full text at https://pmc.ncbi.nlm.nih.gov/articles/PMC2074877/.
- **R4** Heravi, N., Culbertson, H., Okamura, A. M., Bohg, J. (2024). Development and evaluation of a learning-based
  model for real-time haptic texture rendering. *IEEE Transactions on Haptics*; arXiv 2212.13332 (full text).
- **R5** Culbertson, H., Unwin, J., Kuchenbecker, K. J. (2014). Modeling and rendering realistic textures from
  unconstrained tool-surface interactions. *IEEE Transactions on Haptics* 7(3), 381–393. doi:10.1109/TOH.2014.2316797.
  Abstract: PubMed 25248220, https://pubmed.ncbi.nlm.nih.gov/25248220/ (the record was read by the fact-check through
  NCBI E-utilities, 2026-11-02 UTC; full text not opened).
- **R6** Klatzky, R. L., Giudice, N. A., Bennett, C. R., Loomis, J. M. (2014). Touch-screen technology for the dynamic
  display of 2D spatial information without vision: promise and progress. *Multisensory Research* 27(5–6), 359–378.
  doi:10.1163/22134808-00002447 (record from Crossref; the paper not opened; its content from a search excerpt only).
- **R7** Choi, S., Kuchenbecker, K. J. (2013). Vibrotactile display: perception, technology, and applications.
  *Proceedings of the IEEE* 101(9), 2093–2104 (bibliographic record only).
- **R8** Poppinga, B., Magnusson, C., Pielot, M., Rassmus-Gröhn, K. (2011). TouchOver map: audio-tactile exploration of
  interactive maps. *MobileHCI '11*, 545–550 (bibliographic record only).
- **R9** Wenzel, E. M., Godfroy-Cooper, M. (2021). *The Role of Tactile Cueing in Multimodal Displays: Application in
  Complex Task Environments for Space Exploration*. NASA/TM-20210017508, April 2021,
  https://ntrs.nasa.gov/api/citations/20210017508/downloads/NASA:TM20210017508.pdf (full text extracted). A literature
  review: the studies it cites (Geldard 1960, Craig 1972, Sherrick 1985, Stevens 1968) are as cited by it.
- **R10** Villalonga, M. B., Sekuler, R. (2023). Keep your finger on the pulse: Better rate perception and gap detection
  with vibrotactile compared to visual stimuli. *Attention, Perception, & Psychophysics* 85(6), 2004–2017.
  doi:10.3758/s13414-023-02736-y; full text at https://pmc.ncbi.nlm.nih.gov/articles/PMC10545646/ (read through NCBI
  E-utilities). Stimuli: 50 ms pulses from a C-2 tactor on the left index fingertip.
- **R11** Sadia, B., Sadic, A., Ayyildiz, M., Basdogan, C. (2021). Exploration Strategies for Tactile Graphics Displayed
  by Electrovibration on a Touchscreen. arXiv 2112.13459v1 (full text).
- **R12** Cho, Y., Bianchi, A., Marquardt, N., Bianchi-Berthouze, N. (2016). RealPen: Providing Realism in Handwriting
  Tasks on Touch Surfaces using Auditory-Tactile Feedback. *UIST '16*. doi:10.1145/2984511.2984550; abstract at
  https://arxiv.org/abs/1803.02307.
- **R13** Lederman, S. J., Klatzky, R. L., Hamilton, C. L., Ramsay, G. I. (1999). Perceiving Surface Roughness via a
  Rigid Probe: Effects of Exploration Speed and Mode of Touch. *Haptics-e* 1(1). Abstract at
  https://digital.lib.washington.edu/researchworks/items/5122cd00-0d35-45f0-b379-19141419b89f/full.
- **R14** Kim, S.-C., Israr, A., Poupyrev, I. (2013). Tactile rendering of 3D features on touch surfaces. *UIST '13*,
  531–538. doi:10.1145/2501988.2502020 (Crossref record, which holds no abstract; the ACM page returned HTTP 403; its
  content here is from a search excerpt).
- **R15** Walker, S. P., Salisbury, J. K. (2003). Large haptic topographic maps. *Proceedings of the 2003 symposium on
  Interactive 3D graphics*, 83–92. doi:10.1145/641480.641499 (bibliographic record only).
- **R16** Giudice, N. A., Palani, H. P., Brenner, E., Kramer, K. M. (2012). Learning non-visual graphical information
  using a touch-based vibro-audio interface. *ASSETS '12*, 103–110. doi:10.1145/2384916.2384935 (record only; its
  content here is from search excerpts).

**Other hardware**
- **H1** Precision Microdrives, *C10-100 10 mm coin LRA* product page,
  https://precisionmicrodrives.com/products/fb7e332c-199f-4922-ac22-857688f22351/c10-100-10mm-coin-lra: typical
  operating current, resonant frequency, lag, rise and stop times. For scale only.

**This repository** (read at the symbols named in the text):
- `cartalith-native/godot-project/map_overlay.gd` (`map_stroke_sample`, `_emit_stroke_sample`, `_stroke_pen_seen`,
  `_gui_input`, `_TOUCH_SLOP`).
- `shell/dcc_shell.gd` (`_haptic`, `_HAPTIC_MS`, `_compute_layout_mode`, `_is_tablet_sized`, `_PHONE_ASPECT_MAX`).
- `shell/workspaces/world_workspace.gd` (`_sculpt_take_pressure`, `_on_map_stroke_sample`, `set_sculpt_pressure_on`).
- `shell/tool_bar.gd` (`paint_pressure_on`); `shell/engine_bridge.gd` (`sculpt_add_point_p`, `paint_stroke_sample`).
- `shell/colour_paint_tool.gd` (`_on_press`, `_on_drag`, `on_cursor_sampled`); `shell/llm_net_hint.gd`
  (`open_app_settings`); `shell/menus.gd` (the Stylus pressure / Palm rejection note); `shell/tool_overlay.gd`
  (`set_brush_cursor`) and `world_workspace.gd::on_cursor_sampled` (added by the fact-check).
- `project.godot`; `export_presets.cfg`.
- `crates/cartalith-godot/src/lib.rs` (`paint_stroke_begin`, `paint_stroke_sample`, `sculpt_add_point_p`,
  `sample_cell`, `drawn_water_classification`); `render.rs` (`apply_npr`, `Npr::contour_m`, `land_color`'s `r_frac`,
  `sample_arr`, `RenderCtx::h_f`, `RenderCtx::grad_at_f`);
  `sample_bridge.rs` (`slope_at`, `aspect_of_gradient`); `llm_keystore_android.rs`; `vault_saf.rs`.
- `crates/cartalith-spatial/src/stroke.rs` (`Sample`, `Sensor::TiltElevation`).
- `crates/cartalith-terrain/src/analysis.rs` (`ruggedness`, `tpi`, `curvature`); `vector.rs` (`trace_ridgelines`).
- `crates/cartalith-engine/src/lib.rs` (`WorldState::field`, `sea_level`, `river_mask`, `stream_order`).
- `MOBILE_SHELL_PROPOSAL.md` §2.2, §2.5, §3, §4; `MOBILE_SHELL_MS1_SPEC.md`; `LARGE_ITEM_RULINGS.md` (Rulings AV, DY,
  EB, EC).
- `EDITING_TOOLS_SCOPE.md` §3.2, ET-D1–ET-D3; `MAP_CONTEXT_SCOPE.md` §7; `ANDROID_BUILD_SCOPE.md` §1.3 and the Android
  device notes.
- `TABLET_UI_SPEC.md`; `PORT_ONLY_FEATURES.md`; `OUTSTANDING_WORK.md` (the CM-4 device-check row); `TOOLCHAIN.md`.
- Added by the supplement merge: `shell/viewport_host.gd` (its `InputEventPanGesture` and `InputEventMagnifyGesture`
  branches); `project.godot` (`enable_pan_and_scale_gestures`); `world_workspace.gd::_sculpt_take_pressure`;
  `map_overlay.gd::_emit_stroke_sample`; `cartalith-native/docs/STATUS.md` (the CM-4 row); `TABLET_UI_SPEC.md` §4.2;
  `EDITING_TOOLS_SCOPE.md` §3.2; `LARGE_ITEM_RULINGS.md` (searched for stylus, haptic, vibrat, bluetooth and barrel
  before Q13 was written: one hit, Ruling DU's Q5, about a stylus setting off two-finger taps); the HTML working copy's
  `_tch*` functions, `sculptPointerDown`, `sculptRenderCursor` and the asset-library slicer's `pointerdown`.
- `design/mobile-shell-2026-10-31/Shell.dc.html` (the Touch & gestures rows); `html/Cartalith v2.73 DCC test.html`;
  `html/CHANGELOG_v2.73.md`.

---

## Fact-check, 2026-11-01

An adversarial check by a second agent, run in the early hours of 2026-11-02 UTC. It re-opened the sources behind
every number, specification, API name or level, Godot file, issue and app behaviour: A1–A24 and A27 (each member's
"Added in" line), AOSP `main` (A25–A27), a fresh sparse clone of Godot at `4.7.2-stable` (`git ls-remote` gives
`ed1daf0bf001…`), every Godot issue, PR and proposal named, S1–S12, K1–K11, W1, W2 and R1–R5. It re-read the
repository claims at their symbols, re-derived the dp table, §1.3's thresholds, the §7.5 table and the battery
arithmetic, and re-ran the call-site count (9, with the same command). The corrections, each now in the text:

1. **A hover cursor is not missing** (Finding 7, SH-6). `tool_overlay.gd::set_brush_cursor` draws a brush ring at the
   cursor for sculpt, paint, colour paint and territory, on `cursor_sampled`, which `map_overlay.gd::_gui_input`
   emits for every `InputEventMouseMotion` whatever the buttons. Pen hover arrives as mouse motion (§4.2), so the
   ring should follow a hovering pen; that is code reading, not a device test. The old line cited "§2.3 symbols",
   which §2.3 does not hold.
2. **#124191–#124193 are withdrawn** (Finding 6, §4.3, §8.1). Their author closed each the day it was opened, after
   *"Used an ai to write this. closing for now and will use my own words later."*; #124192's *"18 of 66 strokes"*
   comes with *"I have not yet run it as a separate app on the device"*. The `NONE` claim stands on the source alone.
3. **Tennison et al. analysed 17 participants, not 26** (§7.2): 26 were recruited, 7 pilots excluded, 2 removed (R2,
   §4.1.1).
4. **"Above 100 Hz as choppy"** (Finding 9) is not R2's finding: participants felt no feedback or no line (61.75 %,
   pooled); "sporadic" describes the vibrometer signal.
5. **1000 ÷ 62 = 16.1 rounds down to 16, not 15** (§7.3). The 15/s cap is kept as a labelled margin.
6. **The pressure curve was adjusted in Clip Studio Paint only** (§6.2; S5, test #6), not in Infinite Painter; K12 does
   not mention it.
7. **HotHardware lists the pen at $100** (§2.1, S8), not $99.
8. **The OPPO relation is reported** (§2.1): S4, already cited, says the Stylo 2 is *"a rebrand of the Oppo Pencil 2
   Pro"* and the Pad 3 *"a rebrand of the Oppo Pad 4 Pro"*.
9. **Search excerpts were tagged secondary** (Finding 1, §1.1, §1.2, §2.2). By this document's own rules S13 and S14 are
   [unconfirmed]. The OxygenOS 16 rollout and the OPD2415 build now rest on S16 and the Pad 4 on S15, both opened and
   both secondary. The Pad 4's exact 3392 × 2400, "dual tips", the rollout pause, the listings' model number and the
   eraser-in-Notes claim stay [unconfirmed].
10. **Who gets the pen haptics** (Finding 4, §2.3, §2.4): S12 says the unpatched third-party app does, so "OnePlus apps
    only" is softened. S12's class name is its author's guess, citing XDA, not a quote from it.
11. **R5 opened** (§7.2): it was tagged secondary "via search"; its PubMed abstract is now quoted. R4 cites Culbertson
    et al.'s and Romano and Kuchenbecker's models (its references 7, 8, 28) and trains on Penn Haptic Texture Toolkit
    data (reference 6); "the toolkit's models among them" was a gloss.
12. **Scopes made explicit** (§7.2, §7.3): R3 proposes the power rule for *"a fine surface (spatial period<200 μm)"*; R1
    measured feedback to a press on a virtual button; 5 mm/s is a setting of R4's renderer, not a perceptual limit.
13. **Ruggedness tracks slope** (§7.4, new note; Finding 10 points to it). On a uniform plane `ruggedness` is 0.71–0.79
    × `slope`, re-derived in `check/tri_vs_slope.py` in the session's scratch folder, outside the repository. So texture
    would also play on smooth steep ground.
14. **Android details** (§3, §7.7): `FLAG_CANCELED` is *"only set for events with ACTION_POINTER_UP and ACTION_CANCEL"*;
    the `HapticFeedbackConstants` list said "every constant" but left out the page's two `FLAG_IGNORE_*` flags; the
    `createWaveform` sentence is on A17, not A8; rung 1 needs API 31, because `PRIMITIVE_LOW_TICK` is level 31.
15. **Touch slop** (Finding 6, §4.2): 8 dp is AOSP's default `TOUCH_SLOP`, and `config_viewConfigurationTouchSlop` can
    change it per device; what is lost is the moves inside it.
16. **Intensity scaling** (§7.8): the quoted AOSP help line is about `cmd vibrator_manager`'s own vibrations; which
    setting scales an app's vibration was not established.
17. **ET-D2 did not warn of a finger misfire** (SH-2). It measured that touch emulation passes the drag's pressure
    through; the misfire is this document's inference and is unmeasured on the Pad 3.
18. **Smaller fixes**: "That is not safe" (§4.4) is now tagged [judgement; untested], and Finding 8 names that route
    and its tag; the `llm_keystore_android.rs`
    quote is `ANDROID_BUILD_SCOPE.md`'s wording, not the module doc's; W2 verifies the WICG repository (§5.2); the tilt
    row separates Pad 2 from Pad 3 (§2.2); the S Pen's hover is not in K11 (§2.4); R6 carries its full record.

**Checked and held**: the S1–S3 figures and quotes; the review quotes; every other API level in §3; the A16–A24 quotes
and primitive targets; the A25 constants; the A26 help text and `VibratorInfo.dump` keys; A27; every Godot source claim
in §4.2 at the tag, including that VIBRATE reaches the manifest only from the preset (`permissions/vibrate` or
`custom_permissions`) and that the app template declares no permission; the other issues, PRs and
proposals; K1–K11; W1; R1–R4's quotes; the dp and §7.5 arithmetic; every repository symbol in §7.4–§7.6.

**Not checked**: anything that needs the device (all of §8.2, and whether the ring follows a hovering pen on the Pad 3);
the built APK's manifest (none in this checkout); the XDA thread, not attempted; R6–R8 beyond their Crossref records;
search excerpts other than the two news pieces opened (S15, S16).

---

## Supplement merge, 2026-11-02

A second copy of this research lane ran at the same time and wrote a supplement, `STYLUS_HAPTICS_SUPPLEMENT.md`, which
stays in the session's scratch folder, outside the repository. A verifier re-opened the source of every factual claim
in it and gave each a verdict. This merge took only claims the verifier confirmed or downgraded, with the verifier's
tags, and re-opened the load-bearing ones itself: the forum thread (as JSON); the LineageOS trees and kernel driver at
the commits in L1–L8; AOSP's `runListVibrators`, `View.onHoverEvent` and stylus blocker; the Godot files of G6; the
AndroPlus review, The Tech Outlook and the Lederman record (through WebFetch); the verifier's saved copies of the
pages behind every new quotation; both AI-written repositories' READMEs and commit lines; and the HTML working copy's
handlers. The fact-check above is unchanged.

**Merged as confirmed:**
- AndroPlus's touch latency, movement rate and refresh choices, replacing the 290 Hz search excerpt (§1.2, §7.9)
  [secondary: S17]. The verifier's caveat holds: one reviewer's figures, method unstated, cited, not measured.
- Notebookcheck's brightness and touch-scan figures (§1.2) [secondary: S18].
- The forum report of `KEYCODE_F21`, and Drawpile's and Krita's F21 workarounds (§2.2, §6) [secondary: S20;
  verified: K14, K15]. The supplement said Krita !2726's merge was not shown; it merged on 2026-03-27.
- Godot 4.7.2's key table and its `KEY_UNKNOWN` path, the tilt copied from pointer 0, and the emulated mouse following
  the first touch (§4.2) [verified: G6]; Android's `KEYCODE_F13`–`F24`, level 36 (§3.1) [verified: A5].
- The LineageOS facts: no vibrator in the Pad 3's or the Pad 2's tree, the OnePlus 13's per-device vibrator, the older
  common tree's tablet guard (§1.4), the touch controller, the pen charger, the panel mounting and the keyboard
  touchpad (§1.2, §2.1, SH-2) [verified: L1–L5].
- The AOSP primitive frequency targets (§3.3) [verified: A23], and the UX guide's texture constants, which correct this
  document's old line *"No texture constant exists"* (§3.2) [verified: A29].
- The NASA review, Villalonga and Sekuler, RealPen, Lederman et al., Sadia et al. and the C10-100 datasheet (§7.2,
  §7.3, §7.8) [verified: R9–R13, H1].
- From §6's apps: Krita's *Auto* touch painting from its own manual (upgrading the secondary K12), *Tilt Outline* and
  effective outline size; Procreate's curve axes; Infinite Painter's low-latency settings [verified: K13, K7, K6].
- The HTML facts (§5.3), with the verifier's two corrections: the sixth `pointerType` line is the asset-library
  slicer's, not the touch layer's; and under the Tiled LOD viewer an eraser contact pans instead of sculpting. Chrome's
  eraser mapping and browser support [verified: W4, W5].
- The `getevent -lp` row, the double-tap `getevent -lt` row with the pen's presentation mode off and on, and the palm
  probe with pan and magnify gestures logged (§8.2); the last two carry the verifier's additions.
- The stale pen prose in `TABLET_UI_SPEC.md` §4.2 and `EDITING_TOOLS_SCOPE.md` §3.2, reported in §9.

**Merged with the verifier's downgraded tag:**
- *"The Stylo 2's double tap reaches third-party apps as a function key"* became an open question (§2.2). The report's
  pen is named only "OnePlus Stylo", has a side button that holds, and ran on a Chinese Pad 2 Pro under ColorOS 16;
  the Pad 2 Pro launched with a different pen [secondary: S19].
- *"Strong evidence, not proof, that the Pad 3 has no motor"* became *unresolved; it may have none* (§1.4), with what a
  device tree cannot show and the signs pointing the other way.
- *"The pen can only give a constant texture and one-shot pulses"* stays a judgement on an unconfirmed premise, and the
  touch driver's tablet-side on/off command is added (§2.3) [verified: L6].
- *"Pressure, tilt and hover come from the touch screen"* holds for the touch driver; a LineageOS commit says newer
  pens also relay data over Bluetooth [unconfirmed] (§2.2).
- The palm-first case now includes pan and zoom gestures and AOSP's default of rejecting a palm (§4.2, SH-3).
- Walker and Salisbury 2003 to [unconfirmed: record only]; Giudice et al. 2012 stays [unconfirmed], its doubtful
  attribution stated (§7.2).

**Upgraded by the verifier:** hover in or out of range only, from a judgement to [verified: L6];
`BLUETOOTH_CONNECT` as a dangerous permission from API 31, from [unconfirmed] to [verified: A28]; `erhai` as the
Pad 3 [verified: L1].

**Found by the verifier, re-checked here, merged:** LineageOS's own pen service and its 120 Hz cap while a pen is
connected (§1.2, §2.1) [verified: L2, L7]; the pen as its own input device in the touch driver (§4.2) [verified: L6];
AOSP's stylus-over-touch blocker [verified: A30]; *"No vibrator found"* (§1.4) [verified: A26]; `STATUS.md`'s
self-contradicting CM-4 row (§9).

**Refused, and why:**
- **Claims resting only on two AI-written sources.** The pen-protocol repository (S21) says in its README that most of
  its code and docs were written with an AI, and all 60 of its commits carry a Claude co-author line. The Chrome hover
  repro, https://github.com/SonyStone/stylus-hover-repro, has one commit, with the same line. The supplement had tagged
  both [secondary]. S21 appears only as [unconfirmed: AI-written]; the repro is not used.
- **"Empty output means no vibrator."** Refuted: the command prints *"No vibrator found"* [verified: A26].
- **"The Pad 2 logs double taps as `KEY_F20`; single taps, holds and slides send nothing."** Refuted as written. The
  source records `KEY_F20` only in its presentation mode, and "send nothing" describes the Bluetooth notification in
  normal mode; the supplement fused the two modes. §2.2 reports what the source says, tagged AI-written.
- **"Android's own guidance agrees" with the 5 ms gap threshold.** Refuted: a 5 ms threshold on a lab tactor and
  Android's *"a 5 to 10 ms gap … is too short to be detectable"* disagree (§7.3).
- **Sadia et al.'s "10 mm, chosen from vibrotactile guidelines".** A misquote; §7.2 uses the paper's own words.
- **"The GL view never calls `setClickable`, so hover reaches Godot."** The mechanism is refuted: AOSP's
  `View.onHoverEvent` passes hover to `onGenericMotionEvent` for clickable views too. The conclusion was already in
  §3.1, so nothing was added.
- **Duplicates**, not added again: Infinite Painter's non-standard-codes warning and the Windows Ink row (§6.2), the
  50 ms gap and three-levels rules (§3.3, §7.3), the Xiaomi workarounds (§6.1), the "no barrel button" premise for
  CM-4 (§8.1), and the supplement's Q15, which is Q11. Its vibrator-list row duplicated §8.2's, with the wrong
  expected output; §8.2's row now comes first and gives the real one.
- **The supplement's Q13, "map the pen's double tap to an action"**: not added. Its premise, that the Stylo 2 sends a
  key, is unverified; it waits until SH-0 records one. The supplement's Q14, an unofficial Bluetooth route to the pen,
  is this document's Q13, with its premise marked unconfirmed and the default "No".
- **The supplement's battery arithmetic for the 6T** (20 ticks/s, a 3 700 mAh battery): not used. 20/s exceeds this
  document's 15/s cap, and the capacity was not verified. §7.8 applies the datasheet current to this document's own
  duty cycle instead.
- **The browser test page** (`pen_haptics_test.html`): not copied into the repository. The verifier found it safe (no
  network calls besides the page, no storage, no `eval`) and loading without errors in headless Chromium, but not
  sound as it stands. Move lines are logged only when the log's length is a multiple of 8: 0 of 40 moves were logged,
  then all 40 once the 400-line buffer was full, so the coalesced and predicted counts almost never show, and a full
  buffer rewrites 400 lines per event, which can lower the rate it measures. A throwing `setPointerCapture` skips the
  down line and the rate reset. One rate counter is reset by any `pointerdown`, a palm's included. A `true` from
  `navigator.vibrate` does not show a motor. And it shows Chrome's view, not Godot's, so it could not replace SH-0's
  `adb` rows or SH-9's tester.

**Process notes** (no action taken): the two lanes shared one scratch folder, and the supplement's lane overwrote two
of the other lane's scratch files (`scripts/apilevels.py` and `dl/aosp/haptics-ux-design.html`), which the file times
bear out. The supplement reports that the permission system denied its lane a `curl` of OnePlus's specification pages
under a browser user agent, and that it was not retried; that cannot be checked from here. AndroPlus refused `curl`
(HTTP 403) and was read through WebFetch; PhoneArena (HTTP 403) and spatial.umaine.edu (a TLS failure, then HTTP 503)
were not read. Files in the scratch folder from an earlier, unknown
merge run (`merge/dl/`, `merge/testpage/`) were not used by the verifier or by this merge.
