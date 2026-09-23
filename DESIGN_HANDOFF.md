# Designing a new GUI for Cartalith — the handoff

Everything a designer needs to produce a GUI for this app that can actually be
built, written for someone with no prior exposure to the codebase.

**Read this before drawing anything.** The single most expensive failure mode
here is a beautiful design that cannot be implemented, or that is implemented
and then silently disagrees with the engine. Both have happened; the rules below
are what stopped them.

---

## 1 · What the product is

A desktop-and-Android worldbuilding tool. It generates a world — terrain,
hydrology, climate, ecology, resources — then a civilisation on top of it:
settlements, factions, territory, roads, sea routes, trade, a timeline. The user
inspects, edits and styles that world, plans journeys across it, and exports it.

It is a **DCC application** in the sense Blender or Houdini are: a persistent
frame of docks and bars around a viewport, one armed tool at a time, and deep
parameter trees. It is not a document editor and not a game.

There is a **desktop shell, a tablet shell and a phone shell**: one design
system — the same tokens, commands and disclosure depth — composed three ways.
See §5.

---

## 2 · What already exists (do not redesign it blind)

Newest first. The canvases are interactive prototypes, not static screens.

| Where | What |
|---|---|
| `design/mcp-2026-09-07/` | **The owner's three named form-factor canvases**, imported 2026-09-07: `Cartalith DCC Environment.dc.html` (PC, plus tablet frames), `Cartalith Tablet.dc.html` and `Cartalith Android.dc.html`. `CAPTURE.md` there explains how to render them standalone |
| `design/Cartalith-Android-2026-09-07.dc.html` | The phone canvas `ANDROID_UI_SPEC.md` names as its authority |
| `design/dcc-environment-2026-08-31/` | The 2026-08-31 replacement prototype (*"Replace the current GUI, do not upgrade"*) that `dcc_theme.gd`'s tokens were re-based onto, with `BUILD_ANSWERS.md` |
| `design/*.dc.html` | The earlier DCC canvases: shell, menu structure v2/v3, journey planner, measurement and paint toolbars, GUI |
| `design/android-2026-08-30/`, `design/phone-redesign/` | Earlier phone designs, superseded |
| `design/landmark-generation/` | The landmark generator panel design |
| `design/landmark-icons/` | The 49-glyph landmark icon set |
| `DCC_SHELL_SPEC.md` | **The control-by-control specification** (repository root) |
| `TABLET_UI_SPEC.md` | The tablet canvas, inventoried against the shipped shell |
| `ANDROID_UI_SPEC.md` | The phone's per-screen inventory and locked decisions |

**Standing rule, from the owner:** *when two design canvases disagree, the newer
one wins; where none exists, derive from the DCC canvases' own vocabulary.* And
an owner decision is newer than any canvas — `Data ▸ Conversion` is still drawn
in a canvas and was removed by decision, so the canvas is the stale party there.

---

## 3 · The design tokens, resolved

Lifted from `cartalith-native/godot-project/shell/dcc_theme.gd`, which is the
source of truth: **if this section and that file disagree, the file wins** —
this section drifted from it once already, after the 2026-08-31 re-base.
Regions are separated by hairlines, never by fills.

**Radius is 0 for regions, panels and rules.** Action buttons are the
exception: **8 px on desktop and phone, 12 px on tablet** (`ROLE`'s
`btn_radius`), because both current canvases draw them rounded — the older
radius-0-everywhere rule had been abandoned by the design while the shell was
still implementing it. Phone pills are fully rounded (24 px on a 48 dp target).

### Dark (the default)

| Token | Value | Used for |
|---|---|---|
| `bg` | `#0d0e0f` | Application ground, viewport letterbox |
| `panel` | `#121314` | Docks, menu bar, tool options bar |
| `panel_alt` | `#111210` | Rows sitting a shade back from panel |
| `raised` | `#17191a` | Menus, popovers, modals — anything floating |
| `sunken` | `#191c1e` | Input wells, list bodies, the open menu-bar title — a shade *above* `panel` |
| `line` | `rgba(255,255,255,.10)` | **Region** separators |
| `line_soft` | `rgba(255,255,255,.07)` | Quieter dividers |
| `border` | `rgba(255,255,255,.16)` | **Control** outlines — chips, buttons, wells |
| `text_bright` | `#e8ebec` | Headers, active rows, the wordmark |
| `text` | `#c8cbcd` | Body |
| `text_secondary` | `#a9adb0` | Menu-bar items, parameter-row labels |
| `text_dim` | `#8d9296` | Secondary values |
| `text_faint` | `#6f7478` | Units, hints |
| `text_ghost` | `#5f6468` | Disabled |
| `accent` | `#e0a34a` | |
| `accent_hover` | `#f0bd72` | |
| `accent_dim` | `#a4650f` | |
| `accent_ink` | `#141005` | Type on a filled accent surface |
| `accent_wash` | `rgba(224,163,74,.09)` | Hover and list-row selection |
| `accent_wash_2` | `rgba(224,163,74,.16)` | Segment and toggle **on**-state — armed, as against merely current |
| `stale` | `#b9a878` | "downstream is stale" marks |
| `good` · `block` · `water` · `warn` | `#6fae7d` · `#c96a5a` · `#6a9bc4` · `#e0a840` | Semantic verdicts and water |

`raised` is now *darker* than `sunken`; the pair is no longer a ramp and must
not be read as one.

**`line` at .10 and `border` at .16 are different values and the difference is
load-bearing.** Drawing both at .10 is why the shell's chips once read as
suggestions rather than as edges.

### Light

`bg #f4f2ee` · `panel #fbfaf7` · `panel_alt #eeece7` · `raised #fbfaf7` ·
`sunken #eceae4`, inks inverted, `accent #a4650f`. **Type on a filled accent
surface is `accent_ink`, and it flips with the theme** (`#f7f4ee` in light) —
never near-black on light amber. `#ffffff` appears in neither theme.

### Type

UI in Helvetica Neue / system sans. **All numeric readouts, codes, shortcuts and
section labels in IBM Plex Mono**, letter-spacing .12–.26 em.

Sizes: menu 12 · menu item 11 · body 12 · small 11 · tiny 10 · micro 9 ·
section header 9 (tracked wide) · mono readout 11 · modal title 16 · **hero 26**
(30 on tablet; one big accent readout per context — the sample panel's
elevation).

Tablet type is 13–14, phone 13.

---

## 4 · Frame geometry

The frame's regions. Dock widths are user-draggable within min/max. Desktop
and tablet figures are `dcc_theme.gd`'s (`H_*`, `W_*`, `ROLE`).

| Region | Desktop | Tablet | Phone |
|---|---|---|---|
| Menu bar | 36 | 52 | — (app bar 56) |
| Tool options bar | 40 | 56 | bottom sheet |
| Domain rail | 40 | 48 | 44 column |
| Left dock | 372 (300–520) | 400 | full-height sheet |
| Viewport | fills | fills | fills, edge-to-edge |
| Right dock | 304 (260–460) | 400 | full-height sheet |
| Timeline bar | 70 | 88 | 52 |
| Status bar | 26 | 36 | 22 |

Two more densities override the dock widths only: **laptop** (a pointer
window narrower than 1920 — the canvas's `LAPTOP 1366` frame) at 330 / 280,
and **tablet portrait** at 232 / 232. The tablet canvas draws landscape docks
at 320; the shell holds 400 until the dock content reflows to fit, because at
320 most of it measured wider than the dock (`dcc_theme.gd`'s `W_DOCK_TABLET`
comment has the measurement).

Phone extras: top safe area 28; bottom nav 64; gesture inset 20 with a 112×4
handle; reference short side **412 dp**. A 108 px camera-cutout reserve applies
on the **side** edge in landscape; portrait reserves no centre lane. These are
the 412 canvas's figures. The 2026-09-07 phone canvas draws different chrome
(no fixed app-bar height, an 84 px four-cell nav, a landscape rail), tabulated
in `dcc_theme.gd`'s phone block — check `STATUS.md` before assuming either set
is final.

The viewport never scrolls; docks scroll independently. Only one modal at a
time. Menus overlay the tool options bar — they never push layout.

---

## 5 · Three compositions of one design

- **Tablet follows its own canvas, and keeps every desktop command.** The
  August directive was *"keep the tablet version as close as possible to the
  windows gui"*. On 2026-09-07 the owner ruled that the tablet canvas is to be
  matched — *"All designs layouts and styles should match 100%"* — so the
  tablet's menu bar is its canvas's own `☰ · File · World · Data`, not the
  desktop's seven. What is preserved is reach: `_tabletparity_probe.gd`
  asserts that every command title reachable on desktop is reachable on
  tablet, by title and occurrence count.
- **Phone reorganises rather than truncates.** Docks become full-height sheets,
  tool options becomes a bottom sheet, and **all five disclosure levels survive
  inside them**. Bottom bar is four task tabs: `MAP · GENERATE · PLAN · MORE`.

**Minimum touch target 44 px, measured inside the safe area, no exceptions.**
Android's own minimum is 48 dp and the two specs are on record as disagreeing;
where they do, take the larger.

Phone/tablet is decided on a real device by **short side ≥ 900 dp** — *not*
by aspect ratio, and deliberately *not* Android's usual `sw600dp`. Aspect alone
classified every 16:9 tablet as a phone. 900 is an owner ruling (2026-08-31):
the desktop-parity chrome needs 448 dp (rail + dock) before any map, and 900 is
where the map becomes the larger pane again (`dcc_shell.gd`, `_TABLET_MIN_DP`).

---

## 6 · Iconography — the rule that catches most designs

**No emoji anywhere in the product.** Not in menus, not on buttons, not as
status marks.

Every glyph is a bespoke inline SVG on a **16×16 viewBox**, rendered at 12 px in
panels and 14–17 px on canvas buttons: `fill:none`, `stroke:currentColor`,
`stroke-width:1.2`, round caps and joins. **One weight only**, so a glyph never
reads bolder than the hairlines around it. No fills except a 0.7 px dot where a
mark must survive at 12 px. Nothing inside a glyph smaller than 1 px at render
size.

Text symbols stay text — `▾ ▸ ‹ › ⌄ ● ○ ☑ ☐ ✓ ✕ ＋ ⌫ ↶ ↷ ▶ ⏸ ☰ ▤ ⋯ 🔒` — since
they are typographic and inherit type metrics.

There are **89 drawn glyphs already**, in `shell/dcc_icons.gd`'s `PATHS`: 13
sculpt features (terrain cross-sections that read as one family), 15
tool-palette glyphs, 8 utility glyphs, 5 domain marks, 2 phone-nav marks and 46
landmark glyphs — which, with 3 sculpt glyphs reused, cover all 49 landmark
types. Reuse before drawing.

---

## 7 · The structure a design has to fit

**Seven menus, program scope only**: File · Edit · Assets · Data · Preferences ·
Window · Help. World generation, simulation, rendering and map styling are
**workspaces reached through the domain rail, never menu items**.

**Three domains** on the rail, each swapping both docks and the tool options
bar: `WORLD · CIVIL · CARTO`, over a ten-node rail tree. There were five until
the owner merged them (2026-08-20): INFRA's roads, rivers, ports, trade and
logistics live under CIVIL, and RENDER's terrain appearance under CARTO. The
viewport, camera, selection and armed tool all persist across a domain switch.

**One tool is armed at a time, globally.** Arming never changes the workspace and
switching workspace never disarms — they are orthogonal, which is why the tool
palette is a block at the head of every left dock and not a mode.

**Five disclosure levels, never six.** L1 domain → L2 category → L3 section →
L4 group → L5 advanced fold. A design that needs a sixth level needs a different
structure.

---

## 8 · What you can build with

The shell is Godot 4.7 with a hand-built widget layer. Everything a design uses
must map onto `shell/dcc_widgets.gd`'s factories:

`category` · `stage_category` · `section` · `group` · `advanced` (the
disclosure levels) · `slider` (with a separate on-release for expensive writes)
· `toggle` · `choice` (dropdown) · `number` · `chip` · `segment` (segmented
control) · `action` (button) · `tool_button` · `tools_block` · `tool_strip` ·
`note` (prose) · `stale_mark` · `box` · `well` (input) · `text_button` · `band`
· `pad` · the modal family (`modal_card`, `modal_prose`, `modal_stat`,
`modal_list`, `modal_actions`, `modal_button`, `confirm`, `prompt` and
siblings) · plus touch and phone variants `touch_slider`, `phone_pill`,
`phone_slider`, `phone_head`, `phone_window`. The file is the list; this is a
summary of it.

If a design needs something not in that list, **say so explicitly in the
handoff** — a new widget is real work and worth knowing about before build, not
after.

**What does not exist and cannot be drawn as though it does:** a 3D viewport
(deferred by ruling, research first) and the MSAA/anisotropy settings that
would hang off it, and viewshed/visibility analysis.

**What exists only in part — draw it at its real extent:** multi-selection
covers icons, labels and sculpt stamps, not settlements; cut/copy/paste covers
icons and labels; colour management offers sRGB and Display P3; units are
display-only (km, mi, nautical mi), with everything stored in km. All four
were once on the list above as *absent* and turned out to be built — check the
code, or `UNWIRED_FUNCTIONS.md`, before drawing any control as missing.

---

## 9 · The rules that were learned expensively

Each came from a real defect. They are cheap to state and were costly to find.

1. **A control with nothing behind it is a defect.** If a design draws something
   the engine cannot do, it must be drawn *disabled with its reason visible*, not
   drawn live. The repo maintains `UNWIRED_FUNCTIONS.md` — a standing table of
   every presented-but-unbuilt function with a proposal — precisely so this stays
   honest.
2. **A stale reason is the same defect wearing a disguise.** Several rows sat
   disabled for a week against bindings that existed and were being called. The
   wiring audit cannot see this: every function *is* called, and the tooltip is
   what lies.
3. **Disclosure belongs where the user meets it**, not in a footnote. Six
   landmark types depend on an analysis the engine does not compute; the row
   says `[no viewshed]` on the row itself.
4. **Fewer, honest controls beat more.** ~50 per-type advanced folds of inert
   sliders is worse than one sentence saying the per-type thresholds are not
   exposed.
5. **Say what limited a result, in one word.** The landmark panel's cap slider
   shows the cap, what was actually placed, and *which of five things stopped
   it* — because "you asked for fewer" and "these were not good enough" are
   different sentences.
6. **Design for the phone by opening it on a phone.** Three defects in one day
   were invisible on a desktop-sized window: a picker column at a third width, a
   header subtitle running off the screen, and a headroom estimate off by 12×.

---

## 10 · What to hand back

A design lands cleanly here when it comes with:

- **Artboards per breakpoint** where they differ — desktop 1920 (and laptop
  1366 where the docks narrow), tablet 2560 landscape and portrait, phone
  412 dp. Do not hand phone-only or desktop-only for a region that exists
  on both.
- **Every state drawn**, not just the happy one: empty, loading, disabled-with-
  reason, error, and at-limit. The disabled state is the one most often missing
  and the one this codebase most needs.
- **Real values, not lorem.** Use the app's own vocabulary — real settlement
  names, real parameter labels, real units. Where a value is unknown, mark it
  visibly as a placeholder rather than inventing one.
- **A control inventory**: every control drawn, what it binds to, and — the
  important column — whether that binding exists today. That table is what makes
  a design buildable in one pass instead of three.
- **The reasoning**, briefly, for anything that departs from an existing canvas.
  The newer canvas wins by default, so a departure needs to say why.

---

## 11 · Where the code is

Repository: <https://github.com/Achos0190/Cartalith_GDT>
Branch: `claude/cartalith-rust-godot-setup-lhgtgh`

| Path | What |
|---|---|
| `cartalith-native/godot-project/shell/` | The whole GUI |
| `shell/dcc_theme.gd` | Every token, size and role |
| `shell/dcc_widgets.gd` | Every widget factory |
| `shell/dcc_icons.gd` | Every drawn glyph |
| `shell/menus.gd` | The seven menus |
| `shell/workspaces/` | The workspace classes behind the three domains (INFRA and RENDER survive as classes composed into CIVIL and CARTO) |
| `cartalith-native/crates/` | The 16 Rust crates behind it |
| `UNWIRED_FUNCTIONS.md` | What is presented but not built, with proposals |
| `DCC_CONTROL_INDEX.md` | Every control, indexed |
| `GUI_GAP_REGISTER.md` | Every disconnected control found, and its history |
