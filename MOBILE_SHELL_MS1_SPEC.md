# MOBILE_SHELL_MS1_SPEC.md — MS-1: size classes in dp, live re-classification and UI scale

**What this is.** The build specification for MS-1, the first milestone of `MOBILE_SHELL_PROPOSAL.md` §6. The owner
scheduled it first on 2026-10-31 (Ruling DV, `LARGE_ITEM_RULINGS.md`). It states what the code does today, read at the
symbols named. It says what MS-1 changes, in what order, how each step is proved, and what the owner has to decide.
Written 2026-10-31 against commit `5bd7af8`. Symbols are named instead of line numbers, because line numbers drift.

**What it is not.** A build. The owner's instruction was *"Start the refactor with MS-1, mind this is documentation
only."* No file under `cartalith-native/` was changed. How far MS-1 has got is `cartalith-native/docs/STATUS.md`'s
answer, never this file's. The work is routed from `OUTSTANDING_WORK.md` ("Phone and touch").

**Read this first: the 900 dp line.** The proposal's §3 draws the phone/tablet line at 600 dp of width. An earlier owner
answer, which the proposal did not cite, draws it at **900 dp of shortest width**
(`design/dcc-environment-2026-08-31/BUILD_ANSWERS.md` §2.3, 2026-08-31). None of the proposal's nine questions asked to
move it. The shipped code, in turn, applies that answer only to screens narrower than aspect 0.6 (§2.2). MS-1 is
specified so it can be built without settling this. It keeps today's line exactly, makes it one row of one table, and
asks the owner where the line should end up (MQ-1, §8).

**Superseded on the line itself by Ruling EG** (owner, 2026-11-07, and its 2026-11-08 addendum, `LARGE_ITEM_RULINGS.md`):
with a known density, a touch window under **900 dp of shortest width is a phone at every aspect ratio**, sized by
density, and EG (not MQ-1 (c)'s 600 dp width line) decides phone versus tablet. **Built 2026-11-08**: `DccTheme.classify()`
no longer gates the dp line on aspect 0.6; the aspect line now applies only to the legacy figure (no known density, §3.2),
so every probe without `--density` keeps its class (B10). It moves four of §2.2's rows from tablet to phone (the column
added there). Where the sections below say "today's rule", they describe the rule before EG.

## 0. In short

- MS-1 replaces *"phone or tablet, decided once at boot from the aspect ratio, then scaled by the short side ÷ 412"*
  with one classifier over the window's size in dp, re-run on every resize, and a real dp unit in both touch
  compositions. The PC composition is not touched.
- It moves no device between phone and tablet. Whether and when the line moves is MQ-1. *(Ruling EG moved it on
  2026-11-08: see the note above.)*
- Six stages, each landing on its own: **S0** measure (no product change) · **S1** the classifier (no behaviour change)
  · **S2** the dp unit · **S3** live re-evaluation within a class · **S4** a class change without a restart · **S5** UI
  scale.
- The first step is S0 on the OnePlus 6T. This tree carries three different figures for that phone's density (§2.4),
  and the unit depends on which one is true.
- Acceptance bars B1–B10 are in §5. Owner questions MQ-1–MQ-5 are in §8, **all answered by Ruling DY**
  (2026-11-01).
  - MQ-1: width, landing with MS-7.
  - MQ-2: DPI ÷ 160 if needed.
  - MQ-3: keep as today.
  - MQ-4: no UI scale on the PC.
  - MQ-5: **re-present open windows in place**. This is not the default, and it enlarges S4.

## 1. The milestone

`MOBILE_SHELL_PROPOSAL.md` §6:

| # | Milestone | Bar |
|---|---|---|
| MS-1 | Width classes and dp constants in `DccTheme` / `DccShell`, re-evaluated on resize; UI scale | a probe resizing 360 → 900 → 1 920 dp gets phone → tablet → large without a restart |

Made precise, the bar is B5 (§5). At `--density 2`, one process is resized 720 × 1 560 → 1 800 × 2 400 →
3 840 × 2 160 → 720 × 1 560 px (360 × 780 → 900 × 1 200 → 1 920 × 1 080 → 360 × 780 dp). It reports phone → tablet
(expanded) → tablet (extra-large) → phone, and after each step it matches a cold boot at that size. "Large" in the
proposal's sentence is its extra-large class (from 1 600 dp).

## 2. Today, read at the symbols

All paths are under `cartalith-native/godot-project/shell/`. Every symbol below was opened for this document.

### 2.1 How the shell decides phone or tablet

`DccShell._ready()` sets `_touch`: a touchscreen with the `mobile` feature, or the testing-only `--force-touch` user
argument. It calls `_compute_layout_mode()` once, publishes `DccTheme.set_phone(_phone)` and
`DccTheme.set_phone_scale(_phone_scale)`, and then builds either `_build_phone_shell()` or `_build_desktop_shell()`.
`_compute_layout_mode()` reads `get_viewport_rect().size` and sets:

- `_phone = _touch and (short / long) < _PHONE_ASPECT_MAX and not _is_tablet_sized(short)`, with
  `_PHONE_ASPECT_MAX` = 0.6.
- `_is_tablet_sized(short_px)` = `short_px / (screen_get_dpi() / 160) >= _TABLET_MIN_DP` (900). It answers only when
  `OS.has_feature("mobile")` is true and the DPI reads above 0. Otherwise it returns false and the decision is the
  aspect alone, which is the case in every `--force-touch` probe.
- `_landscape = size.x > size.y`, and `_phone_scale = max(1, short / DccTheme.PHONE_REF_SHORT)`, with
  `PHONE_REF_SHORT` = 412. This is computed for every composition, not only the phone (§2.3).
- `DccTheme.set_narrow(size.x < W_LAPTOP_MAX)` (1 920) and `DccTheme.set_portrait(not _landscape)`. On touch but not
  phone, and on the narrow pointer band, it also sets the two dock widths from `role_px()`.

On resize, `_on_window_resized()` re-runs `_compute_layout_mode()` on the phone only, for its orientation. Off the phone
it calls `_on_tablet_resized()`, which acts only when a tablet flips orientation, so a same-orientation resize keeps the
dock widths a user dragged (WI-04). A pointer window's resize changes nothing. **`_phone` itself is never re-decided.**
A phone stays a phone and a tablet stays a tablet for the life of the process. The comment on `_on_window_resized()`
argues why that was safe on the 6T (`GUI_GAP_REGISTER.md` §56: over 52 cold starts, `root.size_changed` never fired).

### 2.2 The 900 dp line, and what the code applies of it

The owner's answer, verbatim (`BUILD_ANSWERS.md` §2.3, imported 2026-08-31):

> **2.3 Tablet threshold — shortest-width ≥ 900 dp gets the desktop-parity shell.**
> Below 900 dp: the phone shell with touch density. Not Android's 600 dp, because the desktop shell has a
> hard floor: 48 rail + 400 dock = 448 dp of chrome before any map. At 800 dp that leaves a 352 dp map —
> narrower than the dock beside it, which is not the Windows GUI in any useful sense; at 900 dp it leaves
> 452 dp, and the map is the larger pane again. So `TABLET 800` is deliberately phone-shaped, and it is on
> the correct side of the line. The Environment's `TABLET PORTRAIT` frame (1600 dp shortest width) and
> `TABLET 2560` are both above it. Stated in both files' headers.

It is recorded in commit `660cbef`, in `_TABLET_MIN_DP`'s comment and in `STATUS.md` row GGR-58. It is **not** in
`LARGE_ITEM_RULINGS.md`. That is how `MOBILE_SHELL_PROPOSAL.md` §3 came to replace it with a 600 dp width line without
asking. The proposal is corrected in this change, and the miss is recorded in `MISTAKES.md` under "Put a question, or a
factual premise, to the owner", now ×3.

**The code applies the answer only below aspect 0.6.** `_is_tablet_sized()` sits behind `and not`, after the aspect
test. It can turn a narrow screen into a tablet, but it can never turn a squarer one into a phone. At aspect 0.6 or
more (16:10, 4:3, a foldable's inner screen) every touch window is a tablet, whatever its size. The shell records the
case itself: a comment in `_build_desktop_shell()` reports `--force-touch --vp 800x1280` as `phone=false tablet=true`.
Four consequences:

- `TABLET 800`, which §2.3 calls *"deliberately phone-shaped"*, gets the desktop-parity shell.
- In portrait that shell has a 48 dp rail and two docks of 232 dp (`DccTheme.TABLET_PORTRAIT`). By those constants
  `TABLET 800` keeps 800 − 512 = 288 dp of map, narrower than §2.3's own 352 dp example. This is arithmetic, not a
  measurement, and the drawn docks can be wider: `W_DOCK_TABLET`'s header records dock content drawing 331–382 px
  against a declared 320.
- `_TABLET_MIN_DP`'s worked table is false about the code in two rows: `TABLET 800 → phone` and `2560x1600 at
  288 dpi = 889 dp → phone`. Both screens are aspect 0.625, so both are tablets. Ruling BK requires a comment to match
  its code; S1 corrects the table when it moves.
- `STATUS.md` GGR-58's *"It was applied"* is true for 16:9 and narrower screens only. A dated note is added there in
  this change.

Fourteen windows under the three candidate rules. Sizes are in dp, portrait first. The pixel sizes are dp × density.

| Window (dp) | Example | Aspect | Today's code, on a device (before EG) | §2.3 as written | Proposal §3 | Ruling EG, built 2026-11-08 |
|---|---|---|---|---|---|---|
| 360 × 780 | 720 × 1 560 px at 2 | 0.46 | phone | phone | phone | phone |
| 411 × 891 | OnePlus 6T, 1 080 × 2 340 px at 2.625 | 0.46 | phone | phone | phone | phone |
| 891 × 411 | the 6T in landscape | 0.46 | phone, landscape | phone, landscape | phone landscape | phone, landscape |
| 480 × 1 040 | 1 440 × 3 120 px at 3 | 0.46 | phone | phone | phone | phone |
| 673 × 841 | foldable, inner screen | 0.80 | **tablet** | phone | tablet, medium | **phone** |
| 800 × 1 280 | `TABLET 800`, 1 600 × 2 560 px at 2 | 0.625 | **tablet** | phone | tablet, medium | **phone** |
| 864 × 1 536 | 16:9, 1 080 × 1 920 px at 1.25 | 0.5625 | phone | phone | **tablet, expanded** | phone |
| 889 × 1 422 | 1 600 × 2 560 px at 1.8 | 0.625 | **tablet** | phone | tablet, expanded | **phone** |
| 1 194 × 834 | 11-inch class, 2 388 × 1 668 px at 2 | 0.70 | **tablet** | phone | tablet, expanded | **phone** |
| 1 366 × 1 024 | 13-inch class, 2 732 × 2 048 px at 2 | 0.75 | tablet | tablet | tablet, large | tablet |
| 1 600 × 1 000 | 2 560 × 1 600 px at 1.6 | 0.625 | tablet | tablet | extra-large | tablet |
| 1 920 × 1 080 | 4K at 2 | 0.5625 | tablet | tablet | extra-large | tablet |
| 2 560 × 1 440 | 4K at 1.5 | 0.5625 | tablet | tablet | extra-large | tablet |
| 3 840 × 2 160 | 4K at 1 | 0.5625 | tablet | tablet | extra-large | tablet |

The last column is `BUILD_ANSWERS.md` §2.3 as written, which is what Ruling EG adopted: it equals the §2.3 column row for
row. B1 (`_ms1class_probe.gd --table`) pins it with literal rows; on the pre-EG tree 13 of its 81 checks fail.

Five of the fourteen rows differ between at least two rules, and every one of them has a shortest side from 600 to
899 dp. Today's code and §2.3 differ in four of them; today's code and the proposal in one, the 16:9 tablet.

Two further facts bear on the choice. Off a device, today's rule is the aspect alone, so a probe at
`--force-touch --vp 3840x2160` gets the phone composition, although the same panel on a device at density 2 is a
tablet. And a UI scale (S5) shrinks the effective size. At 125 %, a 4K panel at density 2 is 1 536 × 864 dp, which is a
phone under either 900 dp rule. Under the proposal's 600 dp width line it stays a tablet up to 320 %.

### 2.3 What a "pixel" means in each composition today

- **Phone.** Every phone figure is authored in 412-reference units and multiplied by `_phone_scale`. `_pscale()` sizes
  boxes; `_ptap()` sizes tap targets, applying the 44-unit floor in reference units before scaling; `_pfont()` sizes
  type and also multiplies by `_os_text_scale()`. So the phone UI grows with the screen's width instead of holding a
  size in dp. A 44-unit target is 154 px = 51 dp on a 480 dp phone at density 3, and 77 px = 38.5 dp on a 360 dp phone
  at density 2, under the 44 dp floor. On the 6T the factor is 1 080 ÷ 412 = 2.621.
- **Phone windows** use the same factor through `DccWidgets.phone_window()` and `phone_present()`. These set the
  window's own `content_scale_factor`, with mode `CANVAS_ITEMS` and aspect `IGNORE`. `DccWidgets.oversample()`'s header
  records what that cost on this build (Godot 4.7.1; `GUI_GAP_REGISTER.md` HD-01):
  - a content-scaled sub-window does not raise its font raster by itself (`get_oversampling()` reads 1.0 at a factor
    of 3.664);
  - `oversampling_override` does, once the window is in the tree, and a resize resets it to 1.0;
  - an embedded `PopupMenu` inherits its embedder's scale.
- **Tablet.** There is no density term anywhere. `_scaled(px)` returns the `DccTheme.TABLET` entry (menu bar 52, rail
  48, status bar 36, timeline 88 and the rest) or `max(44, round(px × TOUCH_SCALE))`. `role_px()` returns its touch
  column. Both give the same pixels at every density. Those figures are the design canvases' own, drawn in CSS px,
  which is to say dp, so they are correct only on a panel at density 1. On a 4K panel at density 2 the 52 px menu bar
  is 26 dp. `GUI_GAP_REGISTER.md` §58 measured the effect at 3 840 × 2 400: the dock's type is the desktop's 10.5 px
  (DS-03).
- **Desktop.** Pixels, with `DccTheme`'s density sets: the base set, and `LAPTOP` below 1 920 px wide, chosen at boot.
  Out of scope, and unchanged by MS-1.
- **A published value that contradicts its own comment.** `DccTheme.phone_scale()`'s comment says it is 1.0 on desktop
  and tablet. But `_ready()` publishes `_phone_scale` unconditionally, and `_compute_layout_mode()` computes it for
  every composition, so it reads short ÷ 412 everywhere: 2.62 on a 1 920 × 1 080 desktop. That is harmless today only
  because its consumers are phone-only. It has 25 consumer lines in 8 files. This pass opened the gates in five of
  them:
  - `busy_overlay.gd` is gated on `is_phone()`;
  - `journey_planner_view.gd` is gated on `app.is_phone()`;
  - `tool_fan.gd` and `phone_menu.gd` are constructed only by `_build_phone_shell()`;
  - `open_project_dialog.gd`'s line is in its phone branch.

  The other three files (`dcc_widgets.gd`, `new_world_dialog.gd`, `workspaces/world_workspace.gd`) are S2's audit.

### 2.4 The 6T's density: three figures in one tree

| Figure | Where it is | Density | The 6T in dp |
|---|---|---|---|
| 402 dpi | `_TABLET_MIN_DP`'s worked table | 2.5125 | 430 × 931 |
| 384 × 832 dp | `GUI_GAP_REGISTER.md` §58 (*"at the tested densities"*) | 2.8125 (450 dpi) | 384 × 832 |
| 420 dpi | the density bucket commonly published for this model; **not measured in this tree** | 2.625 | 411 × 891 |

After S2 the phone's unit is the density, so this figure decides what the owner's phone looks like afterwards.
Compared with today's 2.621, the 6T's whole UI would change by +0.15 % at 2.625, −4.1 % at 2.5125 and +7.3 % at
2.8125.

A related read: `_os_text_scale()` takes the font scale to be `screen_get_scale() ÷ (screen_get_dpi() ÷ 160)`. If those
two calls report different notions of density on this phone, the quotient is not 1.0 at Android's default font size,
and every phone label carries the difference. That has not been measured. S0 reads both calls at two font sizes.

### 2.5 How probes size the shell

A probe script that reads `--vp WxH` makes a `SubViewport` of that size, hosts the shell in it, and sets the window to
min(W, 1 600) × min(H, 1 000); `_storylists_probe.gd` is one example. 106 of the 597 probe scripts do this, and so does one capture script,
`_pcreach_shot.gd`.
212 `.gd` files mention `--force-touch` (873 lines; 195 of those files are probes), and **none reads a density**: no
`--density` argument exists. So a probe at 1 080 × 2 340 gets a 2.621 phone because of the short-side rule, and one at
393 × 852 gets 1.0. The commands are in §6.

### 2.6 What a dp unit reaches beyond the shell's own chrome

Each of these was opened for this document, and each is an S2 item.

- **Map tiles.** `ViewportHost._update_lod()` turns deep-zoom tiles on from `screen_px_per_cell = native_scale ×
  _zoom`, where `native_scale` comes from the control's size. Under a content scale that becomes dp per cell. Tiles
  would then engage `density` times too late, and the base raster's nearest-neighbour cells would show first. That is
  the blockiness `LOD_PX_PER_CELL_THRESHOLD`'s comment exists to prevent.
- **3D view.** `view3d.gd` hosts it in a `SubViewportContainer` with `stretch = true`. Under a content scale the
  `SubViewport` would be sized in dp and its image magnified.
- **Preview capture.** `DccApp._capture_preview_png()` crops the framebuffer, which is in physical px, by
  `viewport.get_global_rect()`, which is in canvas units.
- **Physical pixels from the platform.** There are 13 `DisplayServer` reads of physical px or density, in 4 files:
  - `dcc_shell.gd`, 7: `get_display_safe_area` 1, `screen_get_dpi` 2, `screen_get_scale` 1, `screen_get_size` 1,
    `virtual_keyboard_get_height` 2;
  - `dcc_widgets.gd`, 1: `virtual_keyboard_get_height`;
  - `popout_geometry.gd`, 3: `screen_get_size` 2, `screen_get_usable_rect` 1;
  - `popout_host.gd`, 2: `screen_get_scale`.
- **Export.** The export overlay renders in its own `SubViewport` of its own size, which a root content scale should not
  reach. That is a bar (B9), not an assumption.

## 3. The design

### 3.1 One classifier

A static, pure function in `DccTheme`, which already holds every other form-factor flag and constant:

```gdscript
static func classify(px: Vector2, density: float, ui_scale: float, touch: bool,
		density_known: bool, current: Dictionary = {}) -> Dictionary
```

- **Inputs.**
  - `px`: the size of the viewport the shell lives in, in **physical px**. That is `Window.size` for the root and
    `SubViewport.size` in a probe. It is never `get_viewport_rect()`, which turns into dp once a content scale is
    applied.
  - `density` and `ui_scale`.
  - `touch`: `_touch`.
  - `density_known`: whether the density is real (a device, or `--density`) or the legacy figure (§3.2).
  - `current`: the current class, for hysteresis.
- **Outputs.**
  - `dp`: the effective size, `px ÷ (density × ui_scale)`.
  - `phone` and `landscape`.
  - `width_class`: `compact` below 600, `medium` 600–839, `expanded` 840–1 199, `large` 1 200–1 599, `xl` from 1 600.
    These are the proposal's §3 names. In MS-1 nothing consumes `width_class` except its probe; MS-7 (the docking rule)
    is the first real consumer.
- **The line is one row of a threshold table.** In MS-1 the row was today's rule exactly (MQ-1's default):
  `phone = touch and aspect < 0.6 and (not density_known or short_dp < 900)`. **Since Ruling EG (built 2026-11-08)
  it is** `phone = touch and (short_dp < 900 if density_known else aspect < 0.6)`: the dp line at every aspect, and the
  aspect line for the legacy figure only. `_PHONE_ASPECT_MAX`, `_TABLET_MIN_DP`
  and their comments move from `DccShell` to `DccTheme` with it, and the worked table is corrected in the move (§2.2).
- **Hysteresis.** A class is left only when the size is past its line by a band: 16 dp for a dp line, 0.01 for the
  aspect line. Both are labelled judgements, to be revisited with S3's numbers.
- **No new `class_name`.** A new one needs `godot --headless --import`, which strips `project.godot`'s `;` comments
  (`MISTAKES.md`).
- `DccShell` is its only production caller. The B1 probe calls it directly, without booting a shell.

### 3.2 Where the density comes from

- **On a device:** the source S0 picks. The target is the platform's own density, which is what Android means by dp and
  which follows the user's Display size setting. If GDScript cannot read it, the fallback is `screen_get_dpi() ÷ 160`,
  as today, with the 6T's resulting change stated (MQ-2).
- **Off a device, with `--density D`:** D, and the dp line applies. This is a new user argument, parsed the way
  `--force-touch` is.
- **Off a device, without it:** the **legacy figure**, max(1, short ÷ 412) below aspect 0.6 and 1.0 otherwise. That is
  exactly what each composition draws at today. The dp line does not apply, also as today. This keeps every existing
  probe's result unchanged until it is migrated (B10).
- **On a pointer build:** 1.0, and nothing reads it.

### 3.3 The unit

Each touch composition takes the unit through the mechanism it is already built around.

- **Phone: its own multiplier, now from density.** `_phone_scale` becomes density × UI scale instead of short ÷ 412.
  `_pscale()`, `_ptap()` and `_pfont()` follow without an edit (142, 26 and 28 call lines), and so does every
  `phone_present()` window (41 call lines). The root window keeps a factor of 1, so nothing is scaled twice. Under
  MQ-3's default, phones narrower than 412 dp stay exactly as today until their surfaces are rebuilt in MS-2 to MS-6:
  `_phone_scale = min(density × ui_scale, short_px ÷ 412)`, still at least 1. On the 6T, if its density is 2.625, that
  is today's 2.621 to the last bit.
- **Tablet: the viewport's content scale.** The window the shell lives in gets `content_scale_factor` = density × UI
  scale. In the product that is the root `Window`; in a probe it is the host `SubViewport`'s `size_2d_override`, with
  stretch. Every tablet figure then reads as dp with no call-site edit: the `TABLET` table, `TOUCH_SCALE`, `role_px()`'s
  touch column, the static widget factories' sizes and the theme's fonts. That is the only practical way the 205
  `role_px()` lines in 24 files, and the factories' literals, get a density at all. `DccTheme.phone_scale()` is
  published as 1.0 off the phone, which makes its comment true once S2 has audited its 25 consumer lines. S0 decides
  the content-scale mode, and whether text needs `oversampling_override` (by HD-01's method). *Decided by S0's desktop
  rows (2026-11-07, `cartalith-native/docs/evidence/ms1-2026-11-07/`): mode `DISABLED`, which scales by the factor
  alone, while `CANVAS_ITEMS` scales against the 1152 × 648 base. No override is needed: the engine raises the root's
  oversampling to the factor by itself.*
- **Desktop: untouched.** A factor of 1, no `--density`, and no UI scale unless MQ-4 says otherwise.

**Why two mechanisms and not one.** One root factor for both was considered. It would give the phone dp too, but every
`phone_present()` window already applies its own factor. The phone would need its 41 present sites and 25 consumer
lines re-pointed for no visible gain, and every raw-pixel draw in the phone chrome would grow, losing the 6T's pixel
identity (B7). A per-call-site density for the tablet was considered the other way round. It would mean auditing 205
`role_px()` lines, the 9 `_scaled()` lines and every literal in the static factories, which is DS-03 restated. So each
composition keeps the mechanism it was built around. S2 records this choice in `DECISIONS.md` when it lands.

### 3.4 Live re-evaluation within a class (S3)

`root.size_changed` starts a one-shot debounce timer of 150 ms, a labelled judgement to be revisited on glass. The timer
is a child of the shell, outside both compositions. When it fires, `classify()` runs.

- **If the class is unchanged,** today's within-class handlers run: the phone's `_apply_phone_orientation()`, and the
  tablet's orientation flip with WI-04's rule unchanged. The unit is re-applied if the density or the UI scale changed,
  and the width class and portrait flags are published.
- **If the class changed,** §3.5 runs.
- Pointer builds keep their early return.

### 3.5 A class change without a restart (S4)

A swap is only safe if it cannot diverge from a cold boot, so boot and swap share one path. `_mount_composition()`
builds the composition, attaches the survivors, and runs everything that `DccShell._ready()` and `DccApp._ready()` now
do after the build. `_unmount_composition()` is its inverse. A swap is: unmount, publish the new flags, mount, restore.

- **Survivors** are detached and re-attached, never freed: `ViewportHost` (`viewport`) and `ResourceOverlay`, which
  `DccApp._ready()` adds to the composition's `viewport_content`; `EngineBridge` and the world it holds; the camera,
  which lives in `ViewportHost`; and the debounce timer.
- **Rebuilt:**
  - Each composition, under one root. The phone's is `_phone_root`. The desktop's is a local `VBoxContainer` that no
    field holds today, so S4 gives it one.
  - The `CommandIndex`, which walks the live menu bar (`MISTAKES.md`, "Move a command off the menu bar").
  - The `node_added` hooks for the new class: `_on_tablet_node_added` on a tablet, `_on_dock_node_added` where both
    docks exist.
  - `tablet_arbitrate()` and `_run_dock_fit()`. `tablet_fit()` floors height only and runs once, from a deferred pass
    (`MISTAKES.md`), so it has to be inside the mount path rather than left to boot.
- **Restored:** the domain and node, the selection and the timeline cursor. Within the new class's rules, the phone
  sheet's detent or the tablet docks' collapsed state. WI-04's dragged widths are kept per class.
- **Quiescence.** A swap waits while anything holds unconfirmed input: an armed tool's draft, a drag, a generation or
  bake behind the busy overlay, or a window with held changes. The status line says it is waiting, and the swap runs
  when the last of them ends. Nothing is discarded to make room for a layout change.
- **Open windows are re-presented in place** (Ruling DY, MQ-5: "Re-present in place"; the default was close and
  rebuild).
  - **When the swap runs,** every open window stays open and moves into the new class's layout. Nothing is freed.
  - **Today a window decides its layout once:** `phone_window()` does it in the window's `setup()`. `phone_present()`
    has 41 call lines and `phone_fit()` 58, in 28 files each.
  - **So each of those windows gets a re-present path,** run by the swap after the mount:
    - it re-applies the window's presentation for the new class;
    - where its `setup()` built different content per class, it rebuilds that content inside the open window;
    - it carries its state across: typed text, the selected tab, the scroll position, the follow or pin state.
  - **The quiescence rule above is unchanged.** A swap still waits for held input. Re-presenting is what happens to
    the windows when the swap does run.
- **S4.0, before any S4 code: the inventory.** Each entry gets a stated action: rebuilt, survives, reset, freed or
  (a window) re-presented.
  **Written 2026-11-07:** `cartalith-native/docs/evidence/ms1-2026-11-07/S4_INVENTORY.md`, with Ruling EG's consequences for S4.
  `MISTAKES.md`'s "Delete a surface" row applies, because a swap deletes one.
  - Every field the two builders and their callees assign. `dcc_shell.gd` alone declares at least 78 fields typed with
    a Node class, and 16 `Dictionary` fields, several of which map keys to nodes. The same for `app.gd`, whose
    `_ready()` is 423 lines long.
  - Every connection made during the build: 14 `node_added`, `size_changed` and `phone_insets_changed` connections in
    9 files.
  - The static registries in `DccTheme` that hold nodes: `_portrait_watchers` and `_elide_labels`.
  - Every window `setup()` that branched on the class, with the state its re-present path must carry (MQ-5).
  - Every piece of code that relies on the class never changing, found by the prose that justifies it. Two are known:
    `DccTheme`'s `_elide_labels`, whose comment says labels are registered only `if is_tablet()` because it is
    *"decided once at boot and never revisited"*; and `radial_ring.gd`'s `_apply_touch_scale()` (*"so there is no
    resize hook to re-run this from"*).

### 3.6 UI scale (S5)

- `DccSettings.ui_scale()` and `set_ui_scale()`, in a new section of `user://cartalith_settings.cfg` beside
  `_SEC_HANDEDNESS`. Values 1.0, 1.25, 1.5 and 2.0; default 1.0.
- `Preferences ▸ UI scale ▸ 100 % · 125 % · 150 % · 200 %`, a radio submenu built the way
  `menus.gd::_build_handedness_submenu()` builds Handedness. The proposal's Touch & gestures panel, where it ends up, is
  later work.
- Touch builds only (MQ-4: the PC gets no UI scale, Ruling DY). On the phone, under MQ-3's default, the row is shown disabled with its reason: phones follow
  Android's Display size until the phone layout reflows below 412 dp.
- Changing it re-runs `classify()`, because a scale can cross a line (§2.2), and re-applies the unit.
- Probes use a `--ui-scale S` user argument, which overrides the setting for the run without writing it.

### 3.7 Probes

`--density D` and `--ui-scale S` are the two new user arguments. The shell applies the unit to whatever viewport it
lives in, so a probe host needs nothing beyond passing them. MS-1 edits no existing probe; they keep the legacy figure.
Every new MS-1 probe passes `--density`. Migrating an old probe means adding `--density` and re-reading its expected
numbers, probe by probe. That is follow-up work, not MS-1.

## 4. Stages

| Stage | Delivers | Changes product behaviour? | Bars |
|---|---|---|---|
| S0 | the measurement sheet (§4.1), filled | no | — |
| S1 | `classify()`, the threshold table, `--density`, the legacy figure | no | B1, B2, B8, B10 |
| S2 | the dp unit: the phone's multiplier, the tablet's content scale, the §2.6 sites | tablets, and phones wider than 412 dp | B3, B7, B8, B9, B10 |
| S3 | debounced re-evaluation within a class | live updates only | B4, B8, B10 |
| S4 | S4.0's inventory, then the mount path and the swap | class changes without a restart | B5, B8, B10 |
| S5 | UI scale | adds a preference | B6, B8, B10 |

Each stage lands on its own and is verified before the next starts. S0 needs the 6T, and so does S2's B7. S1 and S3 do
not.

### 4.1 The S0 measurement sheet

Every result cell reads *not measured* until S0 fills it, so an empty cell can never pass for a measured one.

| What | How | Result |
|---|---|---|
| Android's density and size on the 6T | `adb shell wm density`, `adb shell wm size` | not measured |
| `screen_get_dpi()`, `screen_get_scale()` | printed at boot, at the default font size and at `adb shell settings put system font_scale 1.3` (restored after) | not measured |
| `screen_get_size()`, `window_get_size()`, `get_viewport_rect().size` | printed at boot | not measured |
| the platform density, read from GDScript if 4.7.1's Android export allows it (the candidate is the `AndroidRuntime` singleton with `JavaClassWrapper`; unconfirmed) | printed at boot | not measured |
| `_os_text_scale()` at both font sizes | printed at boot | not measured |
| the exported manifest's `resizeableActivity` and `configChanges`, which decide whether split screen, a foldable or `wm density` resizes the app live or restarts it | `aapt2 dump xmltree <apk> --file AndroidManifest.xml` | not read |
| a cold boot to the first frame on the 6T | median (min..max) of 5, run alone | not measured |
| text at a root factor of 2, in `DISABLED` and in `CANVAS_ITEMS` mode | HD-01's edge test, windowed | not measured |
| whether an embedded `AcceptDialog`, a `PopupMenu` and a `ColorPickerButton`'s popup inherit a root factor of 2 | the drawn size of a 300 px window, windowed | not measured |
| touch and mouse positions in `_gui_input` under a factor of 2 | a tap at a known physical pixel | not measured |
| `view3d.gd`'s `SubViewport` size under a factor of 2 | printed | not measured |
| whether `size_2d_override` with stretch, in a probe, draws what a root factor draws | a framebuffer diff of one reference scene | not measured |
| the baselines: every `--vp` probe's result and class, and windowed captures at 393 × 852 and 1 080 × 2 340 (phone), 800 × 1 280 and 2 560 × 1 600 (tablet), 1 920 × 1 080 and 1 366 × 768 (desktop) | the probe suite and `_shot.gd`, before S1 | not taken |

S0 is done when the sheet is filled in here, or in a dated evidence file under `cartalith-native/docs/evidence/` linked
from here. MQ-2 was answered ahead of time (Ruling DY): if the platform density cannot be read, `screen_get_dpi() ÷ 160`
is accepted, and the measured change is recorded in `DECISIONS.md`.

## 5. Acceptance bars

These rules apply to every bar:

- Pixel bars run **windowed**, never `--headless`: `ImageTexture.update()` is a no-op there and `frame_post_draw` never
  fires. Each pixel bar has a positive control that must move.
- Sizes are read from the **drawn rect against its container's visible rect**, or from the framebuffer, and never from
  `get_global_rect()` alone.
- Every probe's usage header names exactly the arguments it reads.
- A new probe must **fail on the code before its stage**. One that passes there pins nothing (`MISTAKES.md`).

The bars:

- **B1 — the classification table (S1).** A probe calls `DccTheme.classify()` over §2.2's fourteen windows in both
  orientations, at UI scale 1.0 and 1.25. It adds boundary rows: a shortest side of 899, 900 and 901 dp, and an aspect
  of 0.599, 0.600 and 0.601, each approached from both sides. The expected column is written by hand from this
  document's text, never computed by the function under test. Mutation: 900 → 899 and → 901, 0.6 → 0.59 and → 0.61, and
  the 16 dp band → 0 must each turn the table red, under `MISTAKES.md`'s mutation rules (Python, exact literal replace,
  restored in a `finally`).
- **B2 — no behaviour change at S1.** Windowed captures at the six S0 sizes are byte-identical before and after S1.
  Every probe prints the same class as in S0's baseline.
- **B3 — the unit (S2).**
  - Tablet, `--force-touch --vp 2388x1668 --density 2`, read from the framebuffer: the menu bar is 104 px tall (52 dp),
    the rail 96 px wide (48 dp) and the status bar 72 px tall (36 dp), each ±1 px. A 12 dp label passes HD-01's edge
    test with a maximum adjacent-pixel |ΔLum| of at least 0.9. HD-01 measured 0.98 for a native control and 0.27 for a
    magnified raster; 0.9 is a labelled judgement between them. *As built, that figure is taken as a share of the glyph box's own
    luminance range. HD-01's figures were white on black, and the theme's 12 px label reads 0.579 raw even at
    factor 1.* *Since Ruling EG, 2 388 × 1 668 at density 2 (1 194 × 834 dp) is a phone, so the tablet figures are
    measured on the OnePlus Pad 3 (`--vp 3392x2400` and `2400x3392 --density 2`, 1 696 × 1 200 dp), and the 11-inch
    window gets the phone leg: `phone_scale` 2.0 (the density, never 1 668 ÷ 412 = 4.05), a 96 px undo chip.*
  - Phone, `--force-touch --vp 1440x3120 --density 3`: a 48 dp pill is 144 px tall, and every tap target is at least
    132 px on both axes (44 dp). The walk is by target, not by height.
  - Map: the zoom at which `_lod_active` first turns true is the same at `--density 1` and `--density 2`, within one
    zoom step. *Measured 2026-11-07, and it does not hold: step 2 at density 1, step 0 at density 2. The steps compare
    two different layouts. At density 2 the 1 194 dp window keeps a 346 dp map beside two 400 dp docks, so its
    cover-fit zoom (2.44) is already past `LOD_AUTO_ZOOM` (2.2). The LOD threshold itself is now in physical px.*
  - 3D: `view3d.gd`'s `SubViewport.size` equals its rect in physical px at density 2.
  - Preview: `_capture_preview_png()`'s crop equals the map's physical rect at density 2.
- **B4 — live within a class (S3).** Rotating `--vp 1080x2340 --density 2.625` to 2 340 × 1 080 gives the phone's
  landscape layout without a rebuild: the composition root's instance id is unchanged. A tablet dock dragged away from
  its default width (to 320 dp, say, if WI-04's range allows it) keeps that width (±0.5 dp) through a same-orientation
  resize from 2 388 to 2 200 px wide at `--density 2`. *Since Ruling EG that window is a phone; the leg resizes the
  OnePlus Pad 3 from 3 392 to 3 100 px wide at `--density 2` (1 696 → 1 550 dp) instead.*
- **B5 — a class change without a restart (S4).** The §1 sequence. After each step it checks:
  - the class, from `classify()`;
  - the composition: phone ⇔ `_phone_root` is valid; tablet ⇔ both docks are valid;
  - a **structural match with a cold boot** at the same size and density. A dump of the composition tree (path,
    class, visibility, rect to 1 dp) equals a cold boot's dump, with an empty allow-list. Any entry later added to that
    list is justified in this file;
  - the world is untouched: a hash the probe computes from the height field, read through the bridge, is equal before
    and after;
  - the camera's zoom and centre are equal to 1e-6, and the domain and node are unchanged;
  - **open windows are re-presented** (MQ-5). The probe opens three windows before the swap: the faction roster on a
    chosen tab, the Place editor, and the vault with unsaved text. Each stays open across the swap, in both directions.
    - Each one's layout dump equals a fresh open of the same window in the new class.
    - Its state is kept: the tab, the text and the scroll position.

  Over ten round trips, the orphan node count (`Performance.OBJECT_ORPHAN_NODE_COUNT`) and the number of connections on
  `root.size_changed` and on `node_added` return to their values before the first swap. Captured stderr holds no
  `SCRIPT ERROR`.

  Hysteresis, with the band applied on both sides of the line as §3.1 states it: a 900 × 1 600 dp window shrunk to
  890 × 1 582 stays a tablet. At 880 × 1 564 it becomes a phone. Back at 900 × 1 600 it is still a phone, and at
  920 × 1 636 it is a tablet again. One second of oscillation between 896 and 904 dp causes at most one swap.

  Timing: a swap costs less than a cold boot at the same size, both measured as the median (min..max) of five serial
  runs, run alone. If it does not, a restart would be the better design, and this file says so.
- **B6 — UI scale (S5).** `--vp 2388x1668 --density 2 --ui-scale 1.5` gives a 156 px menu bar (52 dp × 2 × 1.5).
  *Since Ruling EG that window is a phone at any UI scale; the leg runs at `--vp 4000x2800 --density 2` (2 000 × 1 400 dp,
  1 333 × 933 dp at 150 %, a tablet either way).*
  Pressing `Preferences ▸ UI scale ▸ 150 %` through the visible menu, not by calling a function, writes 1.5 to the
  settings file and gives the same drawn heights. The setting is restored afterwards. At UI scale 125 %,
  `--vp 3840x2160 --density 2` classifies as MQ-1's rule says (a phone under today's), and that swap meets B5.
- **B7 — on the 6T (S2 onwards).**
  - `_phone_scale` is within S0's stated deviation of 2.621. Under MQ-3's default, with a density of 2.625, it is
    bit-identical.
  - Boot screenshots before and after, two of each, diffed outside the clock, with the installed build's
    `lastUpdateTime` recorded.
  - Rotation works.
  - `adb shell wm size 1600x2560` with `adb shell wm density 320` brings the app up, by a restart or a live resize as S0
    found, in the class MQ-1's rule gives 800 × 1 280 dp. `adb shell wm size reset` and `adb shell wm density reset`
    run afterwards.
- **B8 — every stage.** `cargo test --workspace` is unaffected; no Rust change is planned, and the total is checked
  against the last recorded run. Every touched `.gd` file, plus `shell/app.gd`, is parse-checked after every edit.
- **B9 — the PC is unchanged.** Pointer builds (no `--force-touch`) at 1 920 × 1 080 and 1 366 × 768, windowed:
  framebuffers byte-identical before and after each stage, and the root's `content_scale_factor` reads 1.0. An export
  PNG of a fixed world is byte-identical before and after S2, at density 2 as well as 1.
- **B10 — the existing probes.** After every stage, each of the 107 scripts that read `--vp` (106 probes and `_pcreach_shot.gd`) gives the same pass or fail, and
  prints the same class, as its S0 baseline.

## 6. Files, symbols and counts

Expected to change. The build may find more, and names them in its report. Stages S1 to S3 are drafted as
GDScript in `cartalith-native/drafts/mobile-shell/` (drafts 01 and 02, not parse-checked; its README says how to apply them).

| File | Symbols | Stage |
|---|---|---|
| `shell/dcc_theme.gd` | `classify()` and the threshold table (new); `_PHONE_ASPECT_MAX` and `_TABLET_MIN_DP` (moved in); `set_phone_scale()`'s publication off the phone; the `_elide_labels` comment | S1, S2, S4 |
| `shell/dcc_shell.gd` | `_ready()`, `_compute_layout_mode()`, `_is_tablet_sized()`, `_on_window_resized()`, `_on_tablet_resized()`, `_phone_scale`; new `_density()`, `_apply_unit()`, `_mount_composition()`, `_unmount_composition()` | S1–S5 |
| `shell/app.gd` | `_ready()`, split so its composition-dependent part is in the mount path; `_capture_preview_png()` | S2, S4 |
| `shell/viewport_host.gd` | `_update_lod()` | S2 |
| `shell/view3d.gd` | the `SubViewport`'s size | S2 |
| `shell/dcc_widgets.gd`, `shell/popout_host.gd`, `shell/popout_geometry.gd` | whatever S0 finds: embedded and native windows, physical-pixel reads | S2 |
| `shell/radial_ring.gd` | `_apply_touch_scale()`, if the ring survives a swap | S4 |
| `shell/dcc_settings.gd`, `shell/menus.gd` | UI scale | S5 |
| new probes | `_ms1class_probe` (B1, B4, B5, B6) and `_ms1unit_probe` (B3), each a `.gd` with a `.tscn` | S1–S5 |

Every new function, constant and branch is annotated per Ruling BK. A stage that changes behaviour corrects, in the same
change, the prose that described the old behaviour:

- the "decided once at boot" comments on `_phone`, in `_ready()`, on `_on_window_resized()`, in `DccTheme`'s
  `_elide_labels` and in `radial_ring.gd`, found with
  `grep -rn -i "decided once\|once at boot\|changes at runtime" --include='*.gd' shell/`;
- `_TABLET_MIN_DP`'s worked table;
- `DccTheme.phone_scale()`'s comment.

**The counts in this document**, measured on 2026-10-31 at `5bd7af8`:

| Count | Figure |
|---|---|
| call lines of `_pscale(`, `_ptap(`, `_pfont(`, `_scaled(` | 142, 26, 28 and 9, in 2, 2, 2 and 1 files |
| call lines of `phone_present(`, `phone_fit(`, `tablet_fit(` | 41, 58 and 2, in 28, 28 and 1 files |
| call lines of `role_px(` and `phone_scale()` | 205 in 24 files; 25 in 8 files |
| call lines of `DccTheme.is_phone()`, `is_tablet()`, `is_touch()` | 33 in 18 files; 113 in 27; 30 in 11 |
| scripts reading `--vp` | 107: 106 probes and `_pcreach_shot.gd` |
| `.gd` files mentioning `--force-touch` | 212 (873 lines; 195 of the files are probes) |
| probe scripts | 597 |
| uses of `--density` | 0 |
| `DisplayServer` reads of physical px or density | 13 in 4 files |
| `node_added`, `size_changed` and `phone_insets_changed` connections | 14 in 9 files |
| node-typed fields in `dcc_shell.gd` | at least 78, plus 16 `Dictionary` fields |
| `app.gd`'s `_ready()` | 423 lines |

```sh
# From cartalith-native/godot-project. The filter drops lines whose first non-blank character is '#',
# so doc comments are not counted and a call with a trailing comment is.
cnt() { grep -rn --include=*.gd -F "$1" shell/ | grep -v ":[0-9]*:[[:space:]]*#" | grep -v "func $2"; }
cnt "_pscale(" _pscale | wc -l                    # 142; files: cnt ... | cut -d: -f1 | sort -u | wc -l
cnt "phone_present(" phone_present | wc -l        # 41; the same for every helper in the table
grep -rl --include='*.gd' -F -e '"--vp"' . | wc -l                    # 107; with --include='_*_probe.gd', 106
grep -rl --include='*.gd' -F -e '--force-touch' . | wc -l             # 212 (grep -rn: 873 lines)
grep -rl --include='_*_probe.gd' -F -e '--force-touch' . | wc -l      # 195
ls _*_probe.gd | wc -l                                                # 597
grep -rn --include='*.gd' -F -e '--density' . | wc -l                 # 0
grep -rn --include='*.gd' -oE "DisplayServer\.(screen_get_size|get_display_safe_area|virtual_keyboard_get_height|mouse_get_position|window_get_size|screen_get_dpi|screen_get_scale|screen_get_usable_rect|window_get_position)" shell/   # 13
grep -rn --include='*.gd' -E "node_added\.connect|size_changed\.connect|phone_insets_changed\.connect" shell/ | grep -v ":[0-9]*:[[:space:]]*#"   # 14
grep -oE "^var [A-Za-z_]+ *: *[A-Z][A-Za-z0-9_]*" shell/dcc_shell.gd | sed -E 's/.*: *//' | sort | uniq -c
#   Node classes summed: 78. Tween (not a Node) excluded, CommandIndex not checked,
#   untyped and inferred fields not counted. Dictionary: 16.
awk 'NR>815 && /^func /{print NR; exit}' shell/app.gd   # 1238: _ready() runs from line 815 to 1237
```

## 7. Risks

| # | Risk | Where it bites | What contains it |
|---|---|---|---|
| R1 | something is scaled twice | phone windows, which apply their own factor | the phone keeps a root factor of 1 (§3.3); S0 measures what a tablet's windows inherit |
| R2 | blurry text under a content scale | every tablet label | S0's edge test; `oversampling_override` on the root, re-applied on `size_changed`, since a resize resets it (HD-01) |
| R3 | map tiles engage too late | deep zoom on a dense tablet | `_update_lod()` in physical px; B3 |
| R4 | the 3D view renders at dp resolution | `view3d.gd` | B3 |
| R5 | canvas rects mixed with physical images or platform pixels | `_capture_preview_png()`, the 13 `DisplayServer` reads, native pop-out windows | S2's list; B3 |
| R6 | stale references after a swap | at least 78 fields, 16 dictionaries, two registries | S4.0's inventory; B5's orphan and connection counts |
| R7 | an open window keeps the old class's layout, or loses its state when re-presented | the 28 files that call `phone_present()` | each window's re-present path (Ruling DY, MQ-5), probed in both directions (B5) |
| R8 | a fitter that runs only once, at boot | `tablet_fit()`, `_run_dock_fit()` | both inside the mount path; B5's structural match |
| R9 | code that assumes the class never changes | `_elide_labels`, `radial_ring.gd` | found by its prose (§3.5) and rebuilt by the swap |
| R10 | the owner's 6T changes size | every phone surface | S0 first; MQ-2 if the platform density cannot be read; B7 |
| R11 | small phones overflow | phones narrower than 412 dp | MQ-3's default keeps them as they are today |
| R12 | Android restarts the app instead of resizing it | split screen, foldables, `wm density` | S0 reads the manifest. A restart already lands in the right class, so until the manifest allows resizing, live swaps matter for resizable windows and probes |
| R13 | existing probes change meaning | the 107 scripts that read `--vp` | the legacy figure; B10 |

## 8. Owner questions, with defaults

**All five are answered by Ruling DY** (owner, 2026-11-01, `LARGE_ITEM_RULINGS.md`). Each answer follows its question below.
They were needed in this order: MQ-5 before S4, MQ-4 before S5, and MQ-1 before MS-7. None of them blocked S0–S3.

**MQ-1. Where should the phone/tablet line end up?**

- (a) Today's code: a phone is below aspect 0.6 and under 900 dp on its shortest side. **MS-1 ships this whatever the
  answer is,** so MS-1 moves no device, and B2 and B10 can prove it.
- (b) `BUILD_ANSWERS.md` §2.3 as written: every touch window under 900 dp on its shortest side is a phone. Against
  today it moves four of §2.2's rows from tablet to phone: the foldable, `TABLET 800`, the 889 dp panel and the
  11-inch class. It is one table row and can land straight after S1.
- (c) The proposal's §3: under 600 dp wide is a phone, under 500 dp tall in landscape is a phone landscape, and
  everything else is a tablet. Against today it moves one row: the 16:9 tablet at 864 dp becomes a tablet. It is best
  landed with MS-7's floating docks. Before them, that tablet in portrait would get the desktop shell's 48 dp rail and
  two 232 dp docks, leaving 352 dp of map by the constants — the arrangement §2.3 was written to prevent.
- (d) Something else.

Default: (a) inside MS-1, with the final line decided before MS-7. **Recommendation: (c), landing with MS-7.** The
docking rule removes §2.3's premise, because a docked panel can then never leave the map narrower than 520 dp. And the
mockup's medium and expanded classes were drawn for exactly the 600–1 199 dp range. If the owner wants §2.3 honoured
before MS-7, (b) is one row and can land straight after S1.

> **Answered (Ruling DY): (c), "Width: under 600 dp = phone", landing with MS-7.** MS-1 still ships (a) and moves no
> device. This settles what the top of this file left open, where the line ends up, and supersedes §2.3's 900 dp
> line as the target.
>
> **Superseded by Ruling EG** (2026-11-07; its 2026-11-08 addendum answers `S4_INVENTORY.md` Q-S4-1): option (b),
> §2.3 as written, decides phone versus tablet, and (c)'s width classes stay layout classes inside the tablet shell for
> MS-7. Built 2026-11-08 (§2.2's last column).

**MQ-2. If the 6T's platform density cannot be read from GDScript, accept `screen_get_dpi() ÷ 160`, and with it the
change S0 measures on the 6T?** By §2.4's figures that change lies between −4.1 % and +7.3 %. Default: yes, recorded in
`DECISIONS.md` with the measured figure. The alternative is a small Android plugin that reads the true density. Asked
only if S0 finds no route.

> **Answered ahead of time (Ruling DY): accept DPI ÷ 160.** No plugin. It applies only if S0 finds no way to read the
> platform density.

**MQ-3. Phones narrower than 412 dp, until the phone surfaces are rebuilt (MS-2 to MS-6)?**

- (a) **Default.** Keep them exactly as today. The layout stays 412 units wide and shrinks to fit, so a 360 dp phone
  keeps its 38.5 dp targets. UI scale is disabled on phones.
- (b) Pure dp now. Every target is 44 dp, but layouts authored at 412 may overflow at 360. Each overflow is measured and
  either fixed in MS-1 or listed against the milestone that rebuilds that surface. UI scale works on phones.

The owner's 6T is 411 dp wide, so under (a) it does not change.

> **Answered (Ruling DY): (a), keep as today.**

**MQ-4. UI scale on the PC too?** Default: no. The PC is left as it is (owner, 2026-10-31: *"Leave the pc menu as
is"*), and its own density sets already cover narrow and wide windows.

> **Answered (Ruling DY): no.** The default's reason had become stale: the owner had since asked for the PC shell
> refactor (`PC_SHELL_PROPOSAL.md`). So the question was put again as a fresh choice, and the answer was still no. The
> PC gets no UI scale and keeps its own density sets.

**MQ-5. What happens to open windows when the layout changes class?**

- (a) **Default.** Every window without held input closes when the swap runs, and is rebuilt the next time it opens. A
  window with held input delays the swap until it is confirmed or cancelled.
- (b) Windows stay open and each is re-presented in the new layout. Every window then needs a re-present path, which
  touches 28 files.
- (c) Ask first: a card says the window changed size class and offers to switch now.

> **Answered (Ruling DY): (b), "Re-present in place".** The owner chose against the default. Every open window
> stays open and is re-presented in the new layout, with its state kept. That gives each of the 28 files a re-present
> path. §3.5 says what the path does, S4.0's inventory lists each window's state, and B5 probes three windows in
> both directions. The quiescence rule is unchanged.

## 9. Not in MS-1

- **The PC composition.** That includes the fact that its `LAPTOP` set is chosen once at boot: a pointer window resized
  across 1 920 px keeps its boot-time answer. It is recorded here because S1 touches the same function. It is not
  MS-1's work, and it is not scheduled.
- Floating docks, the 520 dp rule, the ☰ collapse at 1 180 dp and the readouts from 1 920 dp (MS-7).
- The bottom bar and domain tab (MS-2), the domain sheet (MS-3), MENU (MS-4), the tool strip (MS-5) and STORY (MS-6),
  and with them the phone surfaces' reflow below 412 dp.
- The Touch & gestures panel. MS-1 adds one Preferences row.
- Any Rust change. None is expected.

## 10. `MISTAKES.md` rows to carry into every MS-1 brief

| Row | How it applies here |
|---|---|
| Schedule or act on a backlog row | re-open every symbol in §2 before starting |
| Quote a count of call sites, instances or files | §6's figures are dated; re-measure with its commands before quoting them |
| Write a test that pins a constant · Mutation-test | B1's expected column is literal, and mutation is required |
| Change behaviour | §6's list of "decided once" prose |
| Declare the work green · Re-run the parse check after EVERY edit | `cargo test --workspace`, and every touched `.gd` plus `shell/app.gd` parse-checked after each edit |
| Run a pixel probe · Write `--headless` into a brief | B2, B3 and B9 run windowed |
| Assert a control is present, sized or reachable · Ask a lane to walk a touch floor | the drawn rect against its container's visible rect; targets on both axes |
| Floor a control that a rebuild creates | `tablet_fit()` floors height only and runs once, so it belongs in the mount path |
| Delete a surface | a swap deletes a composition: S4.0's inventory comes first |
| Move a command off the menu bar | the `CommandIndex` is rebuilt after a swap |
| Add a probe in the same commit that changes the behaviour | B1's live rows and B5 must fail before their stage |
| Write a probe's usage header | name `--vp`, `--density`, `--ui-scale` and `--force-touch` exactly as the probe reads them |
| Run `godot --headless --import` | no new `class_name` (§3.1) |
| Quote a timing or a benchmark | B5's timing is a median with min..max, run alone |
| Run a probe `--headless` and report its error count | B5 captures stderr for `SCRIPT ERROR` |
| Judge a measurement against a floor | name the density of every measured size |
| Screenshot a running app as evidence | B7: two shots of each, diffed, with `lastUpdateTime` |
| Put a question, or a factual premise, to the owner | MQ-1 quotes §2.3 verbatim, and §2.2 states what the code does with it |

## 11. Corrections made with this document

- `MOBILE_SHELL_PROPOSAL.md` §3 carries a correction: its 600 dp line overturns `BUILD_ANSWERS.md` §2.3 without having
  asked, and the question is now MQ-1. §6's MS-1 row points here.
- `LARGE_ITEM_RULINGS.md`: Ruling DV records this scheduling, and cites §2.3 so the 900 dp answer can be found from
  that file.
- `cartalith-native/docs/STATUS.md`: the mobile shell section records this specification, and that MS-1 has no code.
  Row GGR-58 gets a dated note: the 900 dp answer is applied below aspect 0.6 only.
- `MISTAKES.md`: "Put a question, or a factual premise, to the owner" goes to ×3.
- `OUTSTANDING_WORK.md` and `CLAUDE.md`: their routing rows.

The `.gd` comments that §2.2 and §2.3 found false are not edited here, because this pass changes no code. S1 and S2
correct them when they touch those symbols.
