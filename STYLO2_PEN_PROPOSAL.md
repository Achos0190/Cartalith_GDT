# The OnePlus Stylo 2 in Cartalith: a proposal, and a request to revisit Ruling EE

**A proposal of 2026-11-09, agent-produced at the owner's request** (2026-11-08/09): form a theory on how to use and
implement the OnePlus Stylo 2, the OnePlus Pad 3's pen, in Cartalith, taking reference from other graphical and
map-making software. **Documentation only.** Nothing here is built, no device was touched and nothing was installed.
It has no status column; how far anything has got is `cartalith-native/docs/STATUS.md`'s answer, never this file's.

**What it asks.** Ruling EE (2026-11-02) set the stylus to *standard input only* — *"no haptics, no root, no plugin,
no pen SDK"*. The owner's own integration pack (2026-11-08/09) now shows that an ordinary, non-root app can receive
the pen's **double tap** and its **side slide**, and read its **battery**, over Bluetooth LE. Those are not reachable
under EE. This proposal therefore asks the owner **to revisit EE for one narrow part**: a Kotlin Android plugin that
listens to the pen over BLE. It says what changes if the owner agrees (§6–§8), and what stays possible, and is
recommended, if the owner does not (§9). **The haptic wall is not part of the request**: firing the pen's motor stays
off in every option here (§2.3).

**How to read the tags** (the convention of `STYLUS_HAPTICS_RESEARCH.md`):
- **[verified: …]** — a primary source opened in this session, or this repository's code at a named symbol.
- **[secondary: …]** — a review, forum post or third-party article.
- **[vendor claim]** — a vendor's marketing or the pack's own statement about the owner's device.
- **[unconfirmed]** — a search excerpt, a page that would not open, or a claim nobody here has measured.
- **[judgement]** — this document's own design choice.
- **The pack** is the owner's integration pack, copied verbatim into
  `cartalith-native/docs/evidence/stylus-2026-11-08/cartalith-pen/` (`README.md`, `protocol.md`, `Stylo2Pen.kt`,
  commits `f0c61697`, `67704643`). Its fourth file, `stylus-input.md`, is listed but **not yet supplied**. The pack's
  findings come from the owner's reverse-engineering of his own devices. They are **owner-measured, not independently
  reproduced here**, and are tagged **[pack]**. A document's claim about itself is a claim (`MISTAKES.md`); §2.4 lists
  where the pack and this repository's earlier research disagree.

---

## 1. Summary and top recommendations

1. **Measure before deciding (SP-0, allowed under EE as it stands).** One `adb shell getevent -lt` while double-tapping
   and sliding on the pen, with its presentation mode off and on, says whether either gesture already reaches Android as
   a key. If it does, Cartalith can use it with **no plugin and no Bluetooth**, inside EE (§9.2). This is SH-0's
   unrun double-tap row (`STYLUS_HAPTICS_RESEARCH.md` §8.2), so it is not new work.
2. **If the owner revisits EE, build one small, read-only BLE plugin** (§6): a Kotlin Android v2 plugin wrapping a
   corrected `Stylo2Pen.kt`, a GDScript facade, **off by default**, asking for `BLUETOOTH_CONNECT` only when the user
   switches it on, and absent without error on desktop, the 6T and any non-OnePlus device. Its writes are limited to an
   **allow-list of three query commands** (battery, firmware, name). It never sends the enable, vibration-type or
   vibration-start commands and never touches anything else on the pen.
3. **Double tap: "Switch to eraser" by default for drawing tools, user-selectable, never Undo by default** (§4.1).
   This follows the convention every pen platform read here shares (Apple, Procreate, Huawei), and the one Stylo 2
   review that says a resting palm *"may activate double-tap undo"* in OnePlus's own app [secondary: S4 of the earlier
   research].
4. **Slide: brush size while a drawing tool is armed, nothing otherwise by default; zoom and the timeline year as
   options** (§4.2). It is the pen's analogue of Wacom's Touch Ring, whose Photoshop default is brush size.
5. **Battery: a row in the Stylus settings and the tester, always; a status chip only at or below 20 %** (§4.3), like
   Samsung's S Pen alert.
6. **Do not offer the vibration-feel setting** (§4.4). It configures the pen's writing feedback, which reviewers found
   plays only in OnePlus's own apps, so in Cartalith it would change how the pen feels *elsewhere* and nothing here.
7. **Every pen action is a pointer into the existing command paths** — `app.gd::undo_last`, `arm_tool`, the brush
   steppers, `viewport_host.gd`'s zoom step, `dcc_shell.gd::tl_set_year` — never a second implementation (the rule of
   `CUSTOM_QUICK_BAR_SCOPE.md`'s `pin_key`).

---

## 2. What the pen offers

### 2.1 Two channels

| Capability | Channel | Reachable by an ordinary app? | Reachable in Cartalith today (Godot 4.7.2, EE) | Tag |
|---|---|---|---|---|
| Position, pressure, tilt | the touch controller → Android `MotionEvent` | yes | **yes**, as an emulated mouse with pressure and tilt | [pack]; [verified: `STYLUS_HAPTICS_RESEARCH.md` §4.2, Godot source] |
| Hover | `MotionEvent` hover | yes | position only; in or out of range | same |
| Barrel button | the pack: *"a standard Android key/`MotionEvent` button"* | yes, if it exists | **no**: Godot maps a stylus button to index `NONE` — and see §2.4 | [pack]; [verified: §4.2 of the research] |
| Battery %, firmware, name | BLE command `2C 08 / 06 / 26` on characteristic `58480001-…` | yes, with `BLUETOOTH_CONNECT` | no | [pack] |
| **Double tap** | BLE notification `01` on `180938ac-…` | yes, with `BLUETOOTH_CONNECT` | **no**, unless SP-0 finds it also sends a key (§9.2) | [pack] |
| **Side slide** ("film") | BLE Nordic-UART text `film <pos>`, `0x05`…`0xFF`, direction by delta | yes, with `BLUETOOTH_CONNECT` | no | [pack] |
| Charge events | BLE NUS text `chg evt …` | yes | no | [pack] |
| Configure writing-feedback vibration | BLE `2C 94 02 <type> <level+1>` | yes — configures only | no | [pack] |
| **Fire the pen's motor** | the touch controller (`OPLUS_COMPONENT_SAFE`, signature-level) | **no**; only with an Oplus-issued OCS `IPE_PENCIL_CLIENT` authCode | no | [pack]; agrees with the earlier research §2.3 |

### 2.2 The pen's two new gestures, as an input device

- **Double tap**: a discrete event, one byte. No position, no count beyond "a double tap happened" [pack].
- **Slide**: a position on a capacitive strip, about 250 distinct values (`0x05`–`0xFF`), reported as text. Direction
  is the change between reports. **Not published or measured here:** the report rate, the latency over BLE, whether
  lifting the finger sends anything, and whether a stationary finger jitters [unconfirmed]. A BLE link's connection
  interval puts a floor under the latency (typically 7.5 ms to tens of ms, set by the central) [judgement; measured in
  SP-3].
- **Neither gesture has a position on the screen.** So both are *modal* inputs, like a keyboard shortcut: what they do
  depends on what is armed, not where the pen is.

### 2.3 The haptic wall, and why this proposal does not go near it

The pack says it plainly: *"there is no command an unprivileged app can send to make the motor buzz — we verified
this exhaustively on-device"*; `2C 92` is *"accepted, no buzz"*, *"including a fully-privileged rooted session"*
[pack]. This agrees with the earlier research (reviewers found the pen's haptics only in OnePlus's own apps; the
licensed SDK route; §2.3 there) and with Ruling ED's refinement and Ruling EE's closure.

So:
- **Out of every option here:** sending `2C 92` or `2C 94`, any signature, key or permission workaround, root, and any
  pursuit of an authCode by means other than asking Oplus (owner question P7, default no).
- **The relief-you-can-feel goal is unchanged by this proposal**: on the Pad 3, which has no body vibrator (Ruling ED
  update), it remains sound and sight (`STYLUS_HAPTICS_RESEARCH.md` §7.7).
- **The pack's legal note** cites the EU Software Directive 2009/24/EC Art. 6 and the Dutch Auteurswet Art. 45m for
  interoperability analysis of the owner's own devices, and says the pack *"does not include or enable any signature
  forgery, key extraction, or circumvention of a protection measure"* [pack]. This proposal is not legal advice. It
  notes only that the read-only BLE use proposed here sends nothing to the pen beyond queries, and that shipping an app
  that depends on an **undocumented** vendor protocol to other people is a different question from analysing one's own
  device (owner question P9).

### 2.4 Where the pack and the earlier research disagree, or the pack disagrees with itself

| # | Point | The pack | Elsewhere | What settles it |
|---|---|---|---|---|
| 1 | Barrel button | *"arrives as a standard Android key/`MotionEvent` button"* (`protocol.md`) | The Stylo 2 has **no physical barrel button**, a touch-sensitive double-tap area instead [secondary: reviews S6, S8 of the earlier research]; and Godot 4.7.2 reports any stylus button as index `NONE` [verified: research §4.2]. `STATUS.md`'s CM-4 row was corrected for exactly this | SP-0's `getevent -lt`. Until then this proposal assumes no barrel button |
| 2 | Slide range | `0x05` one end … `0xFF` other end (`protocol.md`) | `Stylo2Pen.kt`'s doc: *"0x00..0xFF"* | the code should clamp to the measured range (SP-3) |
| 3 | Slide encoding | ASCII `"film <pos>"` | the Kotlin parses `<pos>` as **hex** (`toIntOrNull(16)`) | one captured line (SP-3) |
| 4 | Double tap and keys | a BLE notification | an AI-written project (not evidence) claims a key `F20` only in the pen's presentation mode; another OnePlus pen sends `F21` to every app on ColorOS 16 [secondary: S20]; Krita and Drawpile translate F21 [verified: K14, K15 of the earlier research] | SP-0 |
| 5 | Battery notify | *"Battery change — battery notify char — byte"* | no UUID given, and the Kotlin does not subscribe to it | SP-3: query on a timer until the characteristic is named |
| 6 | `stylus-input.md` | listed as the standard-input reference | not yet supplied | when it arrives, compare with `drafts/stylus-input/` |

The pack's characteristic `18093046-…` is the one LineageOS's own pen service uses [verified there: L7 of the earlier
research, code marked `Assisted-by: LLM`], which is partial, independent support for the GATT layout.

### 2.5 Review of `Stylo2Pen.kt` (read in full; the evidence copy is not edited)

A plugin must not ship it as is. Each item is read from the file [verified: `Stylo2Pen.kt`]; the Android rules are
cited in §6.4.

1. **Four CCCD writes back to back.** `onServicesDiscovered` loops `enableNotify` over four characteristics, and each
   calls `writeDescriptor` at once. Android's GATT client runs one operation at a time, so writes issued before the
   previous `onDescriptorWrite` are refused — typically only the first subscription takes. **The double tap and the
   slide may then never arrive.** Fix: a one-at-a-time operation queue.
2. **`onConnected` fires before any subscription is confirmed.** It should fire after the last `onDescriptorWrite`.
3. **No `close()` on an unexpected disconnect.** `STATE_DISCONNECTED` only calls `onDisconnected`; the `BluetoothGatt`
   leaks, and Android limits the number of GATT clients per app process. Close, then reconnect with back-off.
4. **No status handling.** A non-success status on connect (the commonly reported `133`) is ignored, and
   `writeCharacteristic`'s return code (an `int` from API 33) is discarded.
5. **`bondedPen()` matches any bonded device whose name contains "Stylo"**, and reading `name` itself needs
   `BLUETOOTH_CONNECT` from API 31. Match the exact name, then remember the address the user confirmed.
6. **Every command is writable.** `writeCmd` is private, but `setVibrationType` is public. The plugin should expose
   only the allow-listed queries (§6.3).
7. **Callbacks run on a Binder thread**, as its own doc says. They must be copied and posted to Godot through
   `emitSignal`, never call into GDScript directly (§6.2).

---

## 3. How other software uses pen side inputs

Collected 2026-11-09 by a web pass and from `STYLUS_HAPTICS_RESEARCH.md` §2.4 and §6. Defaults vary by OS version and
by page; where two sources disagree, both are given.

### 3.1 Drawing and painting software, and pen platforms

| App or platform | Double tap / squeeze | Barrel or side button | Slider, ring or strip | Battery | Source |
|---|---|---|---|---|---|
| **Apple Pencil 2 / Pro (system)** | Double tap: switch to eraser, switch to last used, show colour palette, show ink attributes, off; a toggle *"Allow Double Tap Only with Hover"*. Squeeze (Pro): tool palette and the same list, plus a Shortcut; its sensitivity can be lowered *"if it triggers by accident"* | — (no button) | — | not established | Apple's developer doc: apps should *"perform the preferred action to provide a consistent user experience across apps"*, and *"If the preferred action doesn't make sense in your app, consider giving people a way to choose a custom action"* [verified: developer.apple.com/documentation/applepencil/handling-double-taps-from-apple-pencil]; the option lists [secondary: macrumors.com Pencil Pro guide] |
| **Procreate** | Double tap: the iPadOS default *"Switch Between Current Tool and Eraser"*, or in-app: Drawing Assist, Eyedropper, QuickShape, QuickMenu, Full Screen, **Clear Layer**, Copy-Paste menu. Squeeze: QuickShape, QuickMenu, Layer Select, Eyedropper; *"If all squeeze gestures are toggled off in Procreate, squeeze will default to the iPadOS settings"* | — | — | — | [verified: help.procreate.com, Handbook ▸ Pencil] |
| **Krita** | — | Tablet settings offer *"Use Mouse Events for Right and Middle clicks"* for tablets that report no side buttons; on Android, *"Treat page up/down keys as right/middle mouse buttons"* (Xiaomi). The pop-up palette opens with a right click. Its Qt fork translates OnePlus's `F21` key to a middle click, on unconditionally | — | — | [verified: docs.krita.org tablet settings and pop-up palette pages; K15 of the earlier research]; default button bindings [unconfirmed] |
| **Clip Studio Paint** | Pencil double tap: set only in iPadOS's settings; squeeze defaults to CSP's Quick Access palette | Galaxy S Pen: *"Clip Studio Paint commands can be executed by pressing the pen button on the S Pen"*, assigned in CSP's Air Action Settings | — | — | [verified: tips.clip-studio.com article 555]; the double-tap FAQ [unconfirmed: search excerpt] |
| **Infinite Painter** | Touch double-tap and long-press are separate "Gestures" | A "Stylus" section *"sets functions for the finger and the stylus buttons"*, shown only once a pressure-sensitive stylus is detected; it warns that secondary buttons *"may send non-standard codes"* | — | — | [verified: docs.infinitestudio.art settings]; the warning [verified: K6 of the earlier research] |
| **Concepts** | "customisable gestures"; options not read | Finger Action can be set to pan | — | — | [unconfirmed: search excerpt]; finger action [verified: `MAP_INTERACTION_RESEARCH.md` §4] |
| **ArtRage** | not documented in what was found | per-stylus tool settings; pressure, tilt, barrel rotation | — | — | [unconfirmed: search excerpt] |
| **Samsung S Pen (Air actions)** | Button double press and gestures, **per app** ("App actions") and system-wide ("Anywhere actions"); the S Pen Framework converts them to `KeyEvent`s for apps that declare them | press and hold opens Camera by default | — | **shown in the Air command menu; a low-battery notice at about 20 %** | [verified: K11 of the earlier research for the framework]; the rest [unconfirmed: samsung.com support excerpts] |
| **Wacom (driver)** | — | Pro Pen 3D: upper button right click, middle Pan/Zoom; Pan/Scroll moves the canvas while the cursor stays put | **Touch Ring, Photoshop default: brush size**; zoom and scroll elsewhere; per-application tabs | the driver's own UI | [unconfirmed: 101.wacom.com and support.wacom.com excerpts] |
| **Microsoft Surface Pen / Slim Pen 2** | Top button: click, double-click and hold launch apps (defaults differ across Microsoft's pages) | barrel button = right click | — | **Surface app, Pen tab** | [unconfirmed: support.microsoft.com excerpts]; haptic waveforms [verified: K9 of the earlier research] |
| **Huawei M-Pencil** | Double tap default *"switch between brush and eraser"*; colour palette; off. In Huawei's own GoPaint the quick switch *"is disabled by default"* | — | — | PC Manager (MateBook) | [unconfirmed: consumer.huawei.com excerpt] |
| **OnePlus Stylo 2 (stock)** | Double tap switches tools **in OnePlus Notes only**, per reviews; a resting palm *"may activate double-tap undo"* | none | the side strip: no review read describes a third-party use | OnePlus's own settings | [secondary: S4, S6, S8 of the earlier research] |

**No pen with a capacitive side slider was found in third-party drawing apps**; the Wacom Touch Ring is the closest
analogue [unconfirmed: negative search].

### 3.2 Map-making and GIS software

| App | Pen use | Side inputs; brush size, zoom or timeline on the pen | Source |
|---|---|---|---|
| **ArcGIS Field Maps** (iOS, Android) | Markup *"with either your finger or a stylus"*. The first stylus tap or drag makes the markup type *"stylus-locked"*: *"every tap or drag with the stylus will add markup"*, while fingers still pan and zoom; a finger needs a double tap on the type to lock it | none documented | [verified: doc.arcgis.com/en/field-maps/ios/use-maps/markup.htm and the Android page]; the absence [unconfirmed] |
| **QField** (QGIS mobile) | *"Draw lines and polygons using a stylus or finger input in freehand digitizing mode"*; freehand turns on when a stylus or mouse hovers, with an editable line or polygon layer selected; the Reshape Eraser takes stylus or finger | none documented | [verified: docs.qfield.org/how-to/data-collection/digitize/]; users report the hover icon missing on a Galaxy Tab S10 FE [secondary: community.qfield.org/t/1796] |
| **Campaign Cartographer 3+** | the pen works as a mouse; a moderator: CC3+ *"does not understand touch input"*; a pressure tablet *"doesn't work particularly well"* | a user suggests a pen side button for the middle-mouse quick pan | [secondary: forum.profantasy.com comments 33042, 28393] |
| **Wonderdraft, Dungeondraft, Inkarnate** | no page on stylus, pressure or tablet behaviour was found | — | [unconfirmed: negative searches; do not assume either way] |
| **Avenza Maps** | Draw and Measure: lines, areas, circles, point by point; no stylus mention seen | — | [unconfirmed: the support page returned HTTP 403] |
| **QGIS desktop on tablets** | not researched | — | — |

**So no map-making or GIS app read here maps a pen's side input to anything.** What they share is the split this
project already adopted: the pen draws, the finger navigates (Field Maps' stylus lock, QField's hover activation;
Ruling EC; `MAP_INTERACTION_RESEARCH.md` pattern E). Cartalith's pen side inputs would be borrowed from drawing apps.

### 3.3 Conventions that bind this proposal

1. **The double-tap default is "current tool ↔ eraser"** (Apple, Procreate, Huawei; OnePlus Notes switches tools).
   Undo is not among Apple's system options nor Procreate's in-app list; it appears in forum threads. Nothing read
   publishes a rule against undo on a double tap. Apple's published mitigation for misfires is hover-gating
   (*"Allow Double Tap Only with Hover"*). The Stylo 2's own palm complaint is exactly an undo that misfired
   [secondary].
2. **The user chooses, and "Off" is always an option** (Apple, Procreate, Huawei, Samsung).
3. **A strip or ring is continuous: brush size first, then zoom or scroll** (Wacom). None read maps undo or a
   destructive action to a slide.
4. **Per-application or per-context mapping exists** (Samsung's App actions, Wacom's per-application tabs), but the
   apps themselves mostly offer one global mapping (Procreate) [judgement from the rows above].
5. **The settings appear only once the hardware is seen** (Infinite Painter's Stylus section; Krita's touch-painting
   Auto) [verified: K6, K13 of the earlier research].
6. **Battery lives in the OS or a companion app, with one low-battery alert** (Samsung, Surface, Huawei).
7. **Map apps use the pen to separate drawing from navigating, not for shortcuts** (§3.2): the side inputs are a
   drawing-app idea that a map-maker can borrow, not a map-app convention.

---

## 4. Use cases, ranked for a map-making app

Ranked by value to Cartalith's drawing tools (`app.gd::tool_kind()`'s `TOOL_DRAWING`: sculpt, WORLD paint, colour
paint, Region, Territory, Lasso, Way) against cost and misfire risk [judgement].

| Rank | Input | Use | Why this rank |
|---|---|---|---|
| 1 | Double tap | **Toggle the eraser / lower mode of the armed drawing tool** | the cross-app convention (§3.3.1); undoable by the tool's own undo; no misfire is destructive |
| 2 | Slide | **Brush size of the armed drawing tool** | the one control every drawing session adjusts most; the Touch Ring precedent; not a world edit, so nothing to undo |
| 3 | Battery | **Settings row and a low-battery chip** | cheap once connected; avoids a dead pen mid-session |
| 4 | Double tap | Switch to the last tool | Apple's second option; useful between Way and Inspect |
| 5 | Slide | Zoom, with no drawing tool armed | a natural strip use; but pinch and the −|+ rocker already exist (Ruling EB item 2) |
| 6 | Slide | Scrub the timeline year (CIVIL) | distinctive for Cartalith; but a ~250-step strip over a range of centuries is coarse and a slip moves the whole map's state; offered, not default |
| 7 | Double tap | Undo | offered, never default (§3.3.1); a misfire on a stroke is recoverable by Redo |
| 8 | Double tap | Open the tool fan or the context card at the pen | needs the pen's hover position at the moment of the tap; plausible, unmeasured |
| — | — | Configure vibration feel | **not offered** (§4.4) |
| — | — | Fire a haptic on contour crossings | **impossible** (§2.3) |

### 4.1 Double tap, in detail

- **Default:** with a drawing tool armed, toggle that tool's erase or lower mode; with any other tool, nothing [judgement].
  - Sculpt (Freehand): Raise ↔ Lower, the mode segment `world_workspace.gd` already shows (*"Raise/Lower/Smooth"*).
    WORLD paint: paint ↔ erase (`world.paint_erase`). Colour paint has no erase mode (no `erase` in
    `colour_paint_tool.gd`, grep 2026-11-09), nor do Way, Territory and Region: there the tap does nothing and the
    status line says why (never a silent no-op).
- **Options:** Off · Eraser toggle · Last tool · Undo · Open the tool fan · any pinnable command (`CUSTOM_QUICK_BAR_SCOPE.md`'s
  `pin_key`, so the pen reuses the quick bar's picker and runs the same handle a menu click runs).
- **One mapping per tool kind**, not per tool: drawing, placing and none (`TOOL_DRAWING`, `TOOL_PLACING`, `TOOL_NONE`).
  Three rows in settings cover every tool, and a new tool inherits its kind's mapping [judgement].
- **Misfire guard:** ignore a double tap that arrives while the tip is on the glass, and for 150 ms after it lifts
  [judgement; SP-4 measures whether that is needed].

### 4.2 Slide, in detail

- **Default:** with a drawing tool armed, brush size through the tool's own stepper (`world_workspace.gd::_step_brush_size`,
  `_step_paint_radius`, and colour paint's radius); otherwise nothing.
- **Mapping:** relative, not absolute — each step of `Δpos` moves the value by one stepper increment, so lifting and
  re-placing the finger anywhere on the strip continues from the current value, as a scroll wheel does [judgement]. A
  dead band of 2 positions absorbs jitter until SP-5 measures it.
- **Options:** Off · Brush size · Zoom (`viewport_host.gd`'s zoom about the screen centre, `ZOOM_BUTTON_STEP` 1.35 per N
  positions) · Timeline year (`dcc_shell.gd::tl_set_year`, the one year writer `PC_SHELL_PROPOSAL.md` names) · Brush
  strength.
- **Feedback:** the brush ring (`tool_overlay.gd::set_brush_cursor`) redraws at the new size at the pen's hover
  position, or the screen centre if the pen is out of range; the year strip moves for a year scrub.

### 4.3 Battery

- **Always:** a row in the Stylus settings section and in the tester (SH-9's `00_stylus_tester.gd` gains a "Pen link"
  block): battery %, firmware, name, connection state, last event. When not connected, the row reads "—" with the
  reason ("Pen link is off", "Not paired", "Permission denied"), never `0 %`.
- **Status chip:** only at or below 20 %, once per session, dismissible [judgement, after Samsung's 20 % notice].
- **Polling:** query on connect and every 10 minutes, until SP-3 names the battery notify characteristic (§2.4 #5).

### 4.4 Why the vibration-feel setting is not offered

`setVibrationType` configures the pen's own **writing feedback** [pack]. Reviewers found that feedback only in
OnePlus's own apps [secondary: S5, S6, S7 of the earlier research]. If Cartalith wrote it, the user's pen would feel
different in OnePlus Notes and nowhere in Cartalith. It is also a write to a vendor setting with no documented reset.
**Default: not built.** If SP-0 finds the writing feedback does play while drawing in Cartalith (it has never been
tested), owner question P6 reopens it.

---

## 5. Where pen actions land in the shell

| Pen action | Lands at | Already shipped? |
|---|---|---|
| Eraser toggle | the armed tool's own erase/lower control: `world.paint_erase` row, sculpt's Raise/Lower segment | yes, as controls [verified: `world_workspace.gd`] |
| Last tool | `app.gd::arm_tool(id)`, the single arming entry | yes [verified] |
| Undo | `app.gd::undo_last()`, *"The single Undo entry point (Edit row, toolbar square, phone chip, Ctrl+Z)"* | yes [verified] |
| Any command | `CommandIndex` row by `pin_key` (`CUSTOM_QUICK_BAR_SCOPE.md`, milestone CQ-0) | the index ships (`shell/command_index.gd`) [verified] |
| Tool fan | MS-5's fan and strip (`MOBILE_SHELL_PROPOSAL.md` §2.5) | the strip's first step ships (`a2200e5b`) |
| Brush size | `world_workspace.gd::_step_brush_size`, `_step_paint_radius` | yes [verified] |
| Zoom | `viewport_host.gd`, `ZOOM_BUTTON_STEP` | yes [verified] |
| Year | `dcc_shell.gd::tl_set_year` | yes [verified] |
| Which kind is armed | `app.gd::tool_kind()`, `viewport_host.gd::tool_kind()` | yes [verified] |

So the plugin adds **inputs**, not features: no action listed needs new engine code, and none moves generated output.

---

## 6. Architecture, if the owner revisits EE

### 6.1 Shape

```
Stylo 2 ──BLE──> CartalithPen (Kotlin v2 plugin, .aar)          Android only
                   - corrected Stylo2Pen: op queue, allow-list, reconnect
                   - signals: pen_state, pen_double_tap, pen_slide, pen_info
                 ──emitSignal (posted to Godot's thread)──>
               PenLink (GDScript facade, shell/pen_link.gd)       every platform
                   - Engine.has_singleton("CartalithPen") ? live : absent
                   - settings, permission flow, misfire guard, slide accumulator
                 ──> PenActions: arm_tool / undo_last / steppers / tl_set_year / pin_key
```

- **Why a plugin, and not the no-Gradle routes this repository already uses.** GDScript's `JavaClassWrapper.create_proxy`
  implements Java **interfaces** and calls GDScript synchronously on the calling thread (`STYLUS_HAPTICS_RESEARCH.md`
  §4.4). `BluetoothGattCallback` is an abstract **class**, so it cannot be proxied, and its callbacks arrive on a
  Binder thread. Rust's raw JNI (`llm_keystore_android.rs`'s route) cannot define a Java subclass either: Android's
  JNI *"`DefineClass` is not implemented. Android does not use Java bytecodes or class files"* [verified:
  developer.android.com/training/articles/perf-jni]. So BLE callbacks need compiled Kotlin or Java in the APK, which
  is an Android plugin, and **a v2 plugin needs the Gradle build** (§6.4). [judgement on the conclusion]
- **The facade owns all policy.** The plugin knows bytes and the pen; it knows nothing about tools. All mapping,
  settings and guards are GDScript, so desktop and probe scenes can drive `PenLink` with fake events.

### 6.2 The plugin

- **Package:** `org.cartalith.pen`, one class `CartalithPenPlugin : GodotPlugin`, with the corrected `Stylo2Pen` beside it.
  Source in a new `cartalith-native/android/pen-plugin/` Gradle module; the built `.aar` and a small export addon in
  `godot-project/addons/cartalith_pen/` (§6.4).
- **Methods (`@UsedByGodot`):** `is_supported() -> bool` (API ≥ 33, BLE present, a bonded device named exactly
  "OnePlus Stylo 2"); `start() -> bool`; `stop()`; `query_battery()`; `query_info()`.
- **Signals:** `pen_state(state: String)` (`off`, `no_permission`, `not_paired`, `connecting`, `connected`, `lost`);
  `pen_double_tap()`; `pen_slide(pos: int)`; `pen_info(battery: int, firmware: String, name: String)`, emitted with
  only the fields known, never a placeholder value.
- **Allow-list (enforced in Kotlin, mutation-tested):** writes only `2C 08 01 01` (battery), `2C 06 01 05` (firmware),
  `2C 26 01 0B` (name). Never `2C 90` (pen enable/disable), `2C 92` (start vibration), `2C 94` (vibration type), and
  never any characteristic outside `58480001-…` and the four CCCDs. Never a firmware-update service.
- **Threading:** every GATT callback copies its bytes and calls `emitSignal`, which Godot posts to its own thread
  (`GodotPlugin.emitSignal`, read at 4.7.2 in the earlier research §4.4). No GDScript runs on the Binder thread.
- **Lifecycle:** connect on `start()` and on app resume if enabled; disconnect and `close()` on pause, `stop()`, and
  every unexpected disconnect; reconnect with back-off 1 s, 2 s, 5 s, then every 30 s while enabled.

### 6.3 The GDScript facade

- `shell/pen_link.gd`, owned by the app like `CommandIndex`. On a platform without the singleton, every method is a
  no-op and `state()` returns `"absent"`; the Stylus settings show the Pen link rows greyed with *"Needs a OnePlus
  Stylo 2 on Android"*. Nothing errors on desktop, the 6T, or another tablet.
- **Settings** in the Stylus section drafted in `drafts/stylus-input/01_dcc_settings_stylus.gd` (per install, never in a
  world save): `pen_link_on` (default **false**), `double_tap_drawing` / `_placing` / `_none`, `slide_drawing` /
  `_placing` / `_none`, `low_battery_chip`.
- **The slide accumulator:** keeps the last position and time; a report more than 250 ms after the previous one starts a
  new contact (no delta applied) [judgement; SP-5 measures it].

### 6.4 Build, permission and SDK facts

| Fact | Consequence here | Source |
|---|---|---|
| A v2 plugin *"is an Android library with a dependency on the Godot Android library"*, with an init class extending `GodotPlugin`; GDScript-callable methods carry `@UsedByGodot`; it registers through manifest meta-data `org.godotengine.plugin.v2.<Name>` | one `.aar`, one init class | [verified: docs.godotengine.org/en/stable/tutorials/platform/android/android_plugin.html]; the meta-data key [secondary: translated doc excerpts] |
| It ships to the editor *"as an `EditorExportPlugin`"* under `addons/<name>/`, using `_get_android_libraries` (the `.aar` paths), `_get_android_dependencies` and the `_get_android_manifest_*` hooks; `_supports_platform` returns true for Android | `addons/cartalith_pen/`; the manifest hook adds the permission and `<uses-feature android:name="android.hardware.bluetooth_le" android:required="false"/>` so devices without BLE are not excluded | same page [verified]; the `uses-feature` line [judgement] |
| *"v2 Android plugin requires the use of the Gradle build process"* | §6.5 | same page [verified]; already recorded in the earlier research (G4) |
| Signals: *"the plugin can use the GodotPlugin::emitSignal(...)"*, declared in `getPluginSignals()`; at `4.7.2-stable` `emitSignal` posts through `godot.runOnRenderThread` | callbacks are thread-safe once copied | the doc [verified]; the thread rule [verified at 4.7.2 by `STYLUS_HAPTICS_RESEARCH.md` §4.4, G1] |
| `JavaClassWrapper` is *"only available in Android builds"*; its proxies are for Java interfaces (`create_sam_callback` needs a single-abstract-method interface) | no proxy for the abstract `BluetoothGattCallback` (§6.1) | [verified: docs.godotengine.org/en/stable/classes/class_javaclasswrapper.html] |
| `OS.request_permission(name)` *"Returns `true` if the permission has already been granted"*; `OS.request_permissions()` *"Requests dangerous permissions"*; `OS.get_granted_permissions()`; the result arrives as `MainLoop.on_request_permissions_result` | the permission flow in §7's table, on `get_tree()` | [verified: docs.godotengine.org class_os and class_mainloop] |
| The preset's per-permission keys are generated from a built-in list that holds `BLUETOOTH`, `BLUETOOTH_ADMIN`, `BLUETOOTH_PRIVILEGED`; `BLUETOOTH_CONNECT` was not found in it | declare it from the plugin's own manifest (preferred), or `permissions/custom_permissions` | [verified in part: godot `platform/android/export/export_plugin.cpp`, master; the rest of the file unread, so the absence is unconfirmed] |
| *"The `BLUETOOTH_ADVERTISE`, `BLUETOOTH_CONNECT`, and `BLUETOOTH_SCAN` permissions are runtime permissions"*; `BLUETOOTH_CONNECT` is for an app that *"communicates with already-paired Bluetooth devices"*; `BLUETOOTH_SCAN` only to look for devices | **only `BLUETOOTH_CONNECT`**: the pen is bonded by OnePlus's own pairing, so Cartalith never scans and needs no location permission | [verified: developer.android.com/develop/connectivity/bluetooth/bt-permissions] |
| `BluetoothGatt.writeCharacteristic(ch, byte[], int)`, `writeDescriptor(d, byte[])` and `BluetoothGattCallback.onCharacteristicChanged(gatt, ch, byte[])` are *"Added in API level 33"*; the older forms are deprecated in 33 | the pack's client is API 33+; `is_supported()` is false below 33 instead of raising the app's minSdk | [verified: developer.android.com/reference/android/bluetooth/BluetoothGatt and BluetoothGattCallback] |
| Godot's Android template: `minSdk 24`, `targetSdk 36` (on `master`); the preset's `gradle_build/min_sdk` is empty here, so the default applies | the plugin's own `minSdk` must be ≤ the app's (24) for the manifest merge, with runtime checks for 31 and 33 | [verified: godot `platform/android/java/app/config.gradle`, master, not the 4.7.2 tag — SP-2 reads the tag]; the preset [verified: `export_presets.cfg`, `gradle_build/min_sdk=""`] |
| Android BLE runs *"one operation at a time, with each new command issued only after the previous one succeeds or fails"*; *"When multiple BLE operations are issued back-to-back, it's common to see only the first one succeed"* | the queue of §2.5 item 1 | [verified: punchthrough.com/android-ble-operation-queue/]; status 133's disconnect-close-reconnect remedy [secondary: forum results, not opened] |
| A GDExtension `.so` (Rust gdext) cannot add Java classes; Android's JNI does not implement `DefineClass` | Rust is not a route for the callbacks | [verified: developer.android.com/training/articles/perf-jni] |

The Pad 3 runs Android 15 or 16 (API 35/36) [verified: S1 of the earlier research; secondary for 16], so the API-33
floor costs nothing on the device this is for.

### 6.5 What the switch to the Gradle build costs this project

- The export preset changes `gradle_build/use_gradle_build=false` to `true`, and the Android build template must be
  installed in the project. That is an edit to `export_presets.cfg` under the owner's scoped authorisation (the
  precedent `ANDROID_BUILD_SCOPE.md` §1.3 records), and it changes the APK recipe (`GUI_GAP_REGISTER.md`'s build and
  drop notes).
- It also unblocks what the Gradle build was the obstacle for: Ruling AV's vault-folder `.aar`, and SH-10 if the owner
  ever wanted it. **This proposal asks for neither**; it only notes that the cost would be paid once.
- `project.godot` is not touched by any of this [judgement; SP-2 checks it].

---

## 7. Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| **BLE instability**: status 133, dropped subscriptions, stale clients | high without care | the op queue, `close()` on every disconnect, back-off (§2.5, §6.2); SP-7's one-hour bar |
| **Coexistence with OnePlus's own pen service** (`com.oplus.ipemanager` holds its own connection) | unknown | Android lets several apps be GATT clients of one link; subscriptions are per link, so Cartalith never *disables* a notification on exit [judgement]. SP-7 checks OnePlus Notes' double tap still works after a Cartalith session |
| **The double tap fires two actions** (OnePlus's system action and Cartalith's) | unknown | SP-4 checks; if both fire, the setting's help text says so and the owner decides (P3) |
| **Permission UX**: `BLUETOOTH_CONNECT` shows a "Nearby devices" prompt | certain once enabled | asked only when the user turns Pen link on, with one line of why first; on denial the toggle returns to off with the reason; on "don't ask again" the row offers *Open app settings* (`llm_net_hint.gd::open_app_settings`, the shipped JNI call) |
| **Firmware variance**: an OxygenOS or pen update moves a UUID or the frame | medium over years | match by UUID, not handle; an unknown frame is logged and dropped, never guessed; the tester shows the firmware string; the feature degrades to "absent", never to wrong actions |
| **A misfired double tap** (palm, grip) | medium | Eraser toggle as default, not Undo; the 150 ms tip guard; Off is one tap away |
| **minSdk**: the corrected client uses API-33 GATT calls | certain | `is_supported()` false below 33; the app's own minSdk is not raised (§6.4) |
| **Distribution**: an undocumented protocol in a shipped app | owner's call | P9 |
| **Scope creep toward the motor** | the earlier history shows it | the allow-list is code, tested; §2.3 is closed in every option |

---

## 8. Milestones (if EE is revisited; SP-0 and SP-1 are inside EE either way)

Every milestone keeps the standing rules: annotate for a human auditor (Ruling BK); never encode "no value" as a
plausible value; parse-check every touched `.gd` plus `shell/app.gd`; `cargo test --workspace` for any Rust (none is
planned); verify on the device — a desktop with `--force-touch` is not the tablet (`MISTAKES.md`).

| ID | Milestone | Needs | Numeric bars |
|---|---|---|---|
| **SP-0** | **Measure the pen's gestures with stock tools** (SH-0's open row): `getevent -lt` while double-tapping and sliding, presentation mode off then on; `dumpsys input` (is the pen an input device, does it report a battery); feel whether the writing feedback ever plays while drawing in Cartalith. Device sheet under `cartalith-native/docs/evidence/stylus-<date>/` | nothing built; **inside EE** | every row filled or marked "not reported by the device"; 20 double taps and 10 full slides logged per mode |
| **SP-1** | **The key path** (only if SP-0 finds a key, §9.2): an unknown key from the pen device, within 2 s of pen activity, runs the double-tap mapping | GDScript; **inside EE** | 20/20 double taps act; 0 actions from 20 presses of each key on a connected keyboard |
| **SP-2** | **Plugin skeleton and the Gradle switch**: the `.aar` loads, `PenLink` sees the singleton on the Pad 3 and reports `absent` on desktop and the 6T | Kotlin + Gradle + preset (owner-authorised) | APK builds and launches; 0 Bluetooth permission prompts and 0 GATT calls in logcat with Pen link off over a 10-minute session; cold start within 5 % of the previous APK (median of 5, min..max) |
| **SP-3** | **Connect, query, subscribe** with the corrected client | Kotlin | connected within 3 s of switching on, 10/10; battery equals OnePlus settings' figure ±1 %, 5/5; all four subscriptions confirmed, 10/10 connects; reconnect within 5 s after detaching and reattaching the pen, 10/10; a logged write audit shows only the three allow-listed frames |
| **SP-4** | **Double tap → action** with the three per-kind rows | GDScript | 50/50 taps delivered; tap-to-action latency median ≤ 150 ms (240 fps video, min..max of 10); 0 actions in 30 trials of drawing with a resting palm |
| **SP-5** | **Slide → brush size, zoom, year** | GDScript | a full slide covers the brush range in ≤ 2 passes; a still finger for 10 s moves the value 0 steps; report rate measured and stated (median, min..max); direction correct 20/20 |
| **SP-6** | **Settings, tester block, low-battery chip** | GDScript | every Pen link row renders "—" with a reason when absent, on desktop, the 6T and the Pad 3 with the pen detached; the chip appears once at ≤ 20 % and never above |
| **SP-7** | **Coexistence and stability** | measurement | a 60-minute drawing session with 0 crashes and 0 stuck reconnect loops; OnePlus Notes' double tap still works after Cartalith exits, 5/5; the pen's settings in OnePlus's app unchanged before and after |

**Order:** SP-0 → (SP-1 if a key exists) → owner decision on EE → SP-2 → SP-3 → SP-4 → SP-6 → SP-5 → SP-7. Each bar is
measured on the Pad 3; desktop runs only prove absence handling.

---

## 9. If Ruling EE stands

### 9.1 What does not change

- The standard-input work goes ahead as drafted: SH-2, SH-3, SH-4, SH-5, SH-6 and SH-9 in
  `cartalith-native/drafts/stylus-input/` (being built now; this proposal adds nothing to them and changes none).
- No plugin, no Gradle, no `BLUETOOTH_CONNECT`, no BLE. The pen's double tap, slide and battery stay unused.

### 9.2 What is still possible inside EE

1. **SP-0** — measuring with stock tools is research, not a route around anything.
2. **A key from the pen, if it sends one.** Godot 4.7.2 forwards every non-gamepad key, and keys above F12 arrive as
   `KEY_UNKNOWN` [verified: research §4.2]. If SP-0 shows the double tap (or the slide's ends) produce a key, a GDScript
   rule can treat an unknown key from the pen's input device, shortly after pen activity, as the pen's gesture (SP-1).
   Drawpile and Krita did exactly this for OnePlus's `F21` [verified: K14, K15]. It is the engine's own input, so it
   reads as within EE's letter; that reading is the owner's to confirm (P2).
3. **The same actions by other means:** the two-finger tap is undo (Ruling DU Q5); the quick bar can pin Eraser and
   Last tool (`CUSTOM_QUICK_BAR_SCOPE.md`); the brush steppers sit in the tool options bar. A one-tap eraser pin is
   most of the double tap's value.
4. **Battery:** OnePlus's own settings show it; Cartalith shows nothing.

---

## 10. Owner questions, with recommended defaults

| # | Question | Recommended default |
|---|---|---|
| P1 | **Revisit Ruling EE** to allow one read-only BLE plugin for the pen's double tap, slide and battery (no haptics, no writes beyond three queries)? | **Not yet: run SP-0 first.** If SP-0 finds no key from the pen, then yes, for this narrow plugin only |
| P2 | Is SP-1 (reading a key the pen sends, through Godot's own input) within EE? | Yes: it is input the engine already delivers |
| P3 | Double tap's default with a drawing tool armed? | Eraser / lower toggle; Off, Last tool, Undo, Tool fan and any pinnable command as options |
| P4 | The slide's default? | Brush size with a drawing tool armed; nothing otherwise; Zoom and Timeline year as options |
| P5 | Battery display? | Settings and tester row always; a chip once per session at ≤ 20 % |
| P6 | Offer the pen's vibration-feel configuration? | No (§4.4), unless SP-0 finds the writing feedback plays in Cartalith |
| P7 | Apply to Oplus for an OCS `IPE_PENCIL_CLIENT` authCode to fire the pen's motor? | No (Ruling EE); revisit only if the felt relief becomes a goal again |
| P8 | One mapping per tool kind, or per tool? | Per kind (drawing, placing, none) |
| P9 | Ship Pen link in builds given to others, or only in the owner's own APKs? | Owner's own builds, labelled experimental, until SP-7 passes and the owner has considered the protocol's undocumented status |
| P10 | Accept the Gradle build switch, if P1 is yes? | Yes; it is the only route for BLE callbacks (§6.1) |
| P11 | Wait for the pack's `stylus-input.md` before SP-2? | No; fold it into the drafts when it arrives |
