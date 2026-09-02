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

There is a **desktop shell, a tablet shell and a phone shell**, and they are not
three designs — see §5.

---

## 2 · What already exists (do not redesign it blind)

| Where | What |
|---|---|
| `design/*.dc.html` | The DCC canvases: shell, menu structure v2/v3, journey planner, measurement and paint toolbars, GUI |
| `design/android-2026-08-30/Cartalith Android.dc.html` | The current phone design — an interactive prototype, not static screens |
| `design/phone-redesign/` | An earlier phone exploration, superseded |
| `design/landmark-generation/` | The landmark generator panel design |
| `design/landmark-icons/` | The 49-glyph icon set |
| `DCC_SHELL_SPEC.md` | **The control-by-control specification.** Lives in the owner's Claude Design project, not this repo |
| `ANDROID_UI_SPEC.md` | The phone's locked decisions. Same place |

**Standing rule, from the owner:** *when two design canvases disagree, the newer
one wins; where none exists, derive from the DCC canvases' own vocabulary.* And
an owner decision is newer than any canvas — `Data ▸ Conversion` is still drawn
in a canvas and was removed by decision, so the canvas is the stale party there.

---

## 3 · The design tokens, resolved

Lifted from `cartalith-native/godot-project/shell/dcc_theme.gd`. These are the
real values, not approximations. **Radius is 0 everywhere on desktop and
tablet.** Regions are separated by hairlines, never by fills.

### Dark (the default)

| Token | Value | Used for |
|---|---|---|
| `bg` | `#0d0e0f` | Application ground, viewport letterbox |
| `panel` | `#121314` | Docks, menu bar, tool options bar |
| `panel_alt` | `#111210` | Rows sitting a shade back from panel |
| `raised` | `#17191a` | Menus, popovers, modals — anything floating |
| `sunken` | `#101112` | Input wells, list bodies |
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
| `accent_wash` | `rgba(224,163,74,.08)` | Active menu / tool background |
| `stale` | `#b9a878` | "downstream is stale" marks |

**`line` at .10 and `border` at .16 are different values and the difference is
load-bearing.** Drawing both at .10 is why the shell's chips once read as
suggestions rather than as edges.

### Light

`bg #f4f2ee` · `panel #f2f0ec` · `panel_alt #eeece7` · `raised #fbfaf7` ·
`sunken #e7e5e0`, inks inverted, `accent #a4650f`. **Filled accent surfaces
carry reversed paper-coloured type in both themes** — never near-black on light
amber.

### Type

UI in Helvetica Neue / system sans. **All numeric readouts, codes, shortcuts and
section labels in IBM Plex Mono**, letter-spacing .12–.26 em.

Sizes: menu 12 · menu item 11 · body 12 · small 11 · tiny 10 · micro 9 ·
section header 9 (tracked wide) · mono readout 11 · modal title 16 · **hero 26**
(one big accent readout per context — the sample panel's elevation).

Tablet type is 13–14, phone 13.

---

## 4 · Frame geometry

Six regions in DOM order. Dock widths are user-draggable within min/max.

| Region | Desktop | Tablet | Phone |
|---|---|---|---|
| Menu bar | 34 | 52 | — (app bar 56) |
| Tool options bar | 34 | 52 | bottom sheet |
| Domain rail | 40 | 48 | 44 column |
| Left dock | 372 (300–520) | 400 | full-height sheet |
| Viewport | fills | fills | fills, edge-to-edge |
| Right dock | 300 (260–460) | 400 | full-height sheet |
| Timeline bar | 70 | 88 | 52 |
| Status bar | 26 | 36 | 22 |

Phone extras: top safe area 28 with a **108 px centre lane reserved** for a
punch-hole (nothing is centred there); bottom nav 64; gesture inset 20 with a
112×4 handle; reference short side **412 dp**.

The viewport never scrolls; docks scroll independently. Only one modal at a
time. Menus overlay the tool options bar — they never push layout.

---

## 5 · The three shells are one design, deliberately

- **Tablet keeps full desktop parity** — same regions, same seven menus, same
  disclosure depth. Only the numbers grow. This is an explicit owner directive:
  *"keep the tablet version as close as possible to the windows gui"*. It is
  measured: a probe asserts the same menus in the same order and **223 reachable
  menu rows on both**.
- **Phone reorganises rather than truncates.** Docks become full-height sheets,
  tool options becomes a bottom sheet, and **all five disclosure levels survive
  inside them**. Bottom bar is four task tabs: `MAP · GENERATE · PLAN · MORE`.

**Minimum touch target 44 px, measured inside the safe area, no exceptions.**
Android's own minimum is 48 dp and the two specs are on record as disagreeing;
where they do, take the larger.

Phone/tablet is decided by Android's `sw600dp` breakpoint — short side ≥ 600
density-independent pixels — *not* by aspect ratio. Aspect alone classified every
16:9 tablet as a phone.

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

There are **~100 drawn glyphs already**, in `shell/dcc_icons.gd`: 13 sculpt
features (terrain cross-sections that read as one family), 12 tool-palette
glyphs, 5 domain marks, and the 49 landmark types. Reuse before drawing.

---

## 7 · The structure a design has to fit

**Seven menus, program scope only**: File · Edit · Assets · Data · Preferences ·
Window · Help. World generation, simulation, rendering and map styling are
**workspaces reached through the domain rail, never menu items**.

**Five domains** on the rail, each swapping both docks and the tool options bar:
`WORLD · CIVIL · INFRA · CARTO · RENDER`. The viewport, camera, selection and
armed tool all persist across a domain switch.

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

`category` · `section` · `group` · `advanced` (the five levels) · `slider` (with
a separate on-release for expensive writes) · `toggle` · `choice` (dropdown) ·
`number` · `chip` · `segment` (segmented control) · `action` (button) ·
`tool_button` · `note` (prose) · `well` (input) · `text_button` · `band` ·
`modal_button` · `pad` · plus phone variants `phone_pill`, `phone_slider`,
`phone_head`, `phone_window`.

If a design needs something not in that list, **say so explicitly in the
handoff** — a new widget is real work and worth knowing about before build, not
after.

**What does not exist and cannot be drawn as though it does:** a 3D viewport
(deferred), MSAA/anisotropy settings, colour management beyond sRGB, a clipboard
model for cut/copy/paste, multi-selection of any entity, unit switching (km
only), and viewshed/visibility analysis.

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

- **Artboards per breakpoint** where they differ — desktop 1920, tablet 2560,
  phone 412 dp. Do not hand phone-only or desktop-only for a region that exists
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
| `shell/workspaces/` | The five domains |
| `cartalith-native/crates/` | The 16 Rust crates behind it |
| `UNWIRED_FUNCTIONS.md` | What is presented but not built, with proposals |
| `DCC_CONTROL_INDEX.md` | Every control, indexed |
| `GUI_GAP_REGISTER.md` | Every disconnected control found, and its history |
