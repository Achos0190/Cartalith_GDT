# `drafts/stylus-input/` — the stylus **input** work (standard permissions)

**GDScript drafts that Godot never loads** (they sit outside `godot-project/`),
written for a later PC session with a Godot binary to paste into the shell. They
are **not parse-checked** here: this session has no Godot. `check_symbols.sh`
greps the shipped tree for every shipped symbol the drafts call, which is the
only verification available without the editor.

## Scope, and what is parked

Owner, 2026-11-02: *"Let's park the haptics feedback for now. And continue
working on what sits within the permissions for stylo input."*

- **Off (Ruling EE, 2026-11-02 — *"standard stylus input with pressure and touch
  is what it is"*):** the haptic feedback (SH-7, SH-8, SH-12), the Q13 Bluetooth
  experiment, the root/Magisk route, and the SH-10 Kotlin plugin (no Gradle). `STYLUS_HAPTICS_RESEARCH.md` and `OUTSTANDING_WORK.md`
  carry that. SH-1 (the VIBRATE permission) already landed in `export_presets.cfg`
  and is not touched here.
- **Here:** the stylus **input** milestones, which are all ordinary pointer
  input — pressure, tilt, hover, pen-vs-finger — and ask for **no special
  permission** and change **no generated output**. From
  `STYLUS_HAPTICS_RESEARCH.md` §8.1: SH-2, SH-3, SH-4, SH-5 (GDScript part),
  SH-6, SH-9.

## What ships already (read at the symbols, 2026-11-02)

The pen is **not** unplumbed. An Android pen arrives as an emulated mouse, and
the shell already reads it:

- `map_overlay.gd::_emit_stroke_sample` reads `mm.pressure`, `mm.tilt`,
  `mm.pen_inverted` off the `InputEventMouseMotion` and emits `map_stroke_sample`
  with a pen/not-pen flag (the ET-D2 rule, `_stroke_pen_seen`).
- `world_workspace.gd` feeds pressure to `sculpt_add_point_p`; a Sculpt pressure
  toggle (`_sculpt_pressure_on`, `set_sculpt_pressure_on`) can turn it off.
- `tool_overlay.gd::set_brush_cursor` draws a brush ring that follows the cursor
  (so it already follows a hovering pen — Godot delivers hover as plain motion).

So the input work **extends** this; it does not start from nothing. The gaps the
research named: colour paint ignores pressure, there is no pressure curve, tilt
reaches the paint engine with no control, there is no hover inner ring, no
pen-only/finger-pans mode with a palm rule, and no stylus setting or tester.

## The drafts

| File | Milestone | What it is | Needs | Status |
|---|---|---|---|---|
| `00_stylus_tester.gd` | **SH-9** | A self-contained tester/calibration `Control`: live pressure, tilt, tilt angle, eraser flag, hover, buttons, report rate, device id, and the observed pressure span. Pure pointer input; no engine, no settings write | `DccTheme.c()` (optional styling); a menu row to open it | **written** |
| `01_dcc_settings_stylus.gd` | **SH-2/4/5** | A `DccSettings` `_SEC_STYLUS` section: pressure curve (soft/linear/firm), comfortable maximum, *Pen draws, finger pans*, tilt-to-size. Per-install, never a world's | pastes by the handedness accessors | **written** |
| `10_pressure_curve.gd` | **SH-4** | One helper `_apply_pen_pressure` plus two one-line wraps at the sculpt and paint pressure chokepoints. GDScript only; identity at the default settings, so no golden can move | `01`; `_sculpt_take_pressure`, `_paint_drag` | **written** |
| `11_hover_inner_ring.gd` | **SH-6** | One optional field + one arc on `tool_overlay.gd`'s brush ring: a dimmer inner ring at the minimum-pressure radius, so a hovering pen previews its footprint span. Additive; default 0 = today's single ring | `set_brush_cursor`, `MEASURE_COLOR`, `cell_px` | **written** |

**Planned next, in this folder (not yet drafted):**

- `02_input_hygiene.gd` — **SH-2.** `project.godot` keys
  `input_devices/pointing/android/enable_long_press_as_right_click` and
  `.../emulate_touch_from_mouse` as the research names, plus
  `disable_scroll_deadzone=true` and `Input.use_accumulated_input = false` while a
  pen stroke is live (set and restored in `map_overlay.gd`). The
  finger-mistaken-for-a-pen refinement that needs a real `TOOL_TYPE` is left to
  SH-10 (the plugin) and is called out, not faked.
- `20_pen_draws_finger_pans.gd` — **SH-3.** The mockup's *Pen draws, finger pans*
  with a palm rule, in `map_overlay.gd`'s input routing. Ties to **Ruling EC**
  (✋ only with drawing tools; one finger pans with tap-only tools). The most
  delicate draft; written last and against the real `_gui_input` branches.

SH-5's Rust half (a `paint_stroke_options` field) is its own change, not a draft
here: it needs the workspace build and a golden-parity pass. **SH-10 (the Kotlin
input plugin) is OFF** — Ruling EE (2026-11-02) narrows the scope to standard
input only (pressure and touch), so nothing here needs a Gradle build or reaches
past what Godot delivers.

## Order

`00` stands alone. `01` (settings) comes before `10`/`20`, which read it. `02`,
`10`, `11` are independent of each other. `20` lands last and after Ruling EC's
map-input work, because it rewrites input routing.

## How to verify (for the PC session)

1. `bash check_symbols.sh` — every shipped symbol the drafts call must be found.
2. Paste each draft per its header's REPLACE/EDIT notes; `godot --headless
   --check-only` the files it touches, plus `shell/app.gd`.
3. **On the device** (no pen exists on the desktop host): open the tester on the
   Pad 3, write and hover with the Stylo 2, and read the live values. That is
   also SH-0's "what does the pen report" measurement without adb
   (`STYLUS_HAPTICS_RESEARCH.md` §8.2).
4. A mouse on the desktop is the control: pressure reads a flat 1.0, tilt (0,0),
   no hover — which proves the readout works even with no pen.

## Rules followed

Standard pointer input only; no permission; no engine-output change (so no
golden risk); every function annotated (Ruling BK); no value is ever drawn as a
plausible default (absent pressure reads `—`, never `0`).
