# UI shell design — DCC-style editor

> **Imported verbatim from the owner's Claude Design project "UI mockups
> planning", sync 2026-08-18T21:40Z.** It replaced the 2026-08-17 version: the
> design team rewrote it as a pure rationale document and moved all
> control-by-control detail into `DCC_SHELL_SPEC.md` (also imported, repo root).
> The body below is untouched; this box is the port's only annotation.
>
> **What this revision changed**: the menu bar became program-scope only with
> **seven** menus, and Generate / Simulate / Render / View became
> **workspaces on a domain rail** rather than menus. `DCC_SHELL_SCOPE.md`
> records what that meant for the code already built, and carries the owner's
> standing rule for resolving design conflicts ("the newer canvas wins").
>
> **Superseded since import** — the body is kept verbatim, so read these
> against it. Each is disclosed in full where named:
>
> | Body says | Now | Where |
> |---|---|---|
> | Five domains — World, Civilization, Infrastructure, Cartography, Render | **Three** — WORLD / CIVIL / CARTO (owner, 2026-08-20: INFRA absorbed by CIVIL, RENDER by CARTO) | `DCC_SHELL_SPEC.md` top notices and §3 |
> | The dependency-ordered pipeline as the left dock's navigation | Menu structure v3 (2026-08-24): each rail is a flat accordion of subject categories; the numbered stages survive as L3 sections and as pipeline status, not as navigation | `DCC_SHELL_SPEC.md` top notices, §3, §5 |
> | Android phone at 393×852, 44 px keep-clear with 108 px centre lane and scrim, 44 px rail column, 26 px gesture inset | Owner ruling 2026-08-25: the phone follows the **412 dp** canvas. A newer phone spec, `design/dcc-environment-2026-08-31/spec/06-phone.md`, has since arrived and outranks it under the same rule | `DCC_SHELL_SPEC.md` §13; `DCC_SHELL_SCOPE.md` "Which canvas wins" |
> | The reference mockup has nine screens | It has **ten**; the tenth is the `Phone inset rules` card | `DCC_SHELL_SPEC.md`'s screen table |
>
> Path note: the design team writes to a `docs/`-rooted convention. In this
> repository `docs/` holds the *source project's* documentation
> (`docs/README.md` records which is which). `UNIFIED_TOOL_PLAN.md` below means
> the port's own, at the root; `GENERATOR_PARAMETERS.md`,
> `BIOME_AND_VISUALS_PLAN.md`, `ATLAS_ARCHITECTURE.md` and
> `SCULPT_EDITOR_INTEGRATION_PLAN.md` are the source project's, under `docs/`
> (this port's parameter reference is `GENERATION_PARAMETERS.md` at the root).
> `MENU_STRUCTURE.md` exists in neither place; the disclosure tree it names is
> `design/Cartalith Menu Structure v2.dc.html`, superseded by `v3.dc.html`.

Why the editor is arranged the way it is. Supersedes the HTML app's single
scrolling control column: Cartalith is a **map editor with a toolchain**, in the
lineage of Nortantis, terrain editors, image editors and 3D DCC applications,
rather than a form with a preview attached.

Control-by-control detail lives in **`DCC_SHELL_SPEC.md`** — every menu item,
every button, every range, and the v2.10 element each one replaces. This
document is the rule set that spec obeys; where the two disagree, the spec is
newer.

Reference mockup: `Cartalith DCC Shell.dc.html` in the Omelette project
*UI mockups planning* — nine screens covering the default shell (dark and
light), Generate → World, Generate → Sculpt, Cartography → Style, the Asset
library and Data manager windows, tablet, and phone.

## The governing split

| Region | Owns | Never holds |
|---|---|---|
| **Top menu bar** | program functions — files and save locations, data import/export, the asset manager, graphics/performance preferences, window layout | anything you use while your hand is on the map |
| **Domain rail** | which workspace is active — World, Civilization, Infrastructure, Cartography, Render | values, lists, one-shot commands |
| **Tool options bar** | the active tool or workspace's frequently-changed values, horizontally, plus its commit/discard | anything belonging to a different tool |
| **Left dock** | the active workspace's own structure — the generation pipeline, the sculpt tool set, the layer list | program settings |
| **Right dock** | Layers, Properties, Sample, stamp stack, selection inspectors | tool invocation |
| **Viewport** | the map, the brush cursor, the layers button, scale bar, projection/zoom readout, cursor coordinates | chrome that could live in a dock |
| **Timeline bar** | simulation transport and simulation-layer toggles | anything not time-based |
| **Status bar** | pass state, staleness, autosave, tile cache, the active tool's modifier hints | controls |

The load-bearing rule: **the top bar is about the program, the map is about the
world.** A control that changes the world belongs to a workspace; a control that
changes the program belongs to a menu. `UNIFIED_TOOL_PLAN.md` decides what a tool
*is*; this document decides where it appears.

## Consequences of that rule

Seven menus — File, Edit, Assets, Data, Preferences, Window, Help — and no
Generate, Simulate, Render or View menu: those are workspaces, reached from the
domain rail. Conversely GPU acceleration, multi-GPU dispatch, render quality,
lighting defaults, tiled LOD and the atlas cache are *program* settings and live
under Preferences, not beside the terrain sliders they used to sit next to.

Anything with a browsable body of content or a multi-field job gets its own
window rather than a dropdown, marked `⧉` in the menu: the Asset library and the
Data manager. The dropdown that opens a window is a shortcut into it, never a
second implementation of it.

## Disclosure grammar

Five levels, no deeper. `MENU_STRUCTURE.md` and the mockup's structure sheet
carry the full tree.

| Level | Form | Rule |
|---|---|---|
| L1 | domain | Owns a workspace, never a mode |
| L2 | ▾ category | One open at a time; state persists per domain |
| L3 | § section | Always expanded — a titled band of rows |
| L4 | › group | One pass or one tool; its action button sits inside it |
| L5 | + advanced | Expert dials only, closed by default, defaults already correct |

A sixth level means the L2 category is wrong and should be split. A group gated
by a checkbox renders at L4 and is hidden, not disabled, when off.

## Dependency order beats menu order

The generation pipeline is sorted by what informs what — Planet → Extent & scale
→ World structure → Tectonics → Volcanism → Erosion → Hydrology → Climate →
Ecology → Resources — and each stage states what it needs and what it produces.
Editing a stage marks everything downstream stale rather than silently
invalidating it. This is the one place in the UI where order carries meaning, so
it is never re-sorted alphabetically or by frequency of use.

## Non-destructive by default

Sculpting, painting and styling all produce drafts. Strokes become live
procedural stamps; style edits change only what is drawn. Nothing reaches the
real heightfield until an explicit Commit, and no presentation control ever marks
a generation stage stale. Finalizing a world locks generation and sculpting while
leaving the 3D viewport and cartography available.

## Drawn, not borrowed

No emoji. Every glyph is a bespoke 1.2 px line drawing in `currentColor`, so it
inherits the accent when active and the light theme when switched. The thirteen
sculpt features are drawn as terrain cross-sections and are the only icons that
carry meaning rather than decoration — see `DCC_SHELL_SPEC.md` §12.

## Touch targets

Windows is pointer-first: 32 px controls, 26 px status bar.

**Tablet (2560×1600 landscape)** — full desktop parity. Menu bar, tool options
bar, domain rail and docks all scale to 44–52 px targets; docks widen to 400 px
so two-column readouts survive the larger type. Nothing is dropped or tucked
into a sheet.

**Android phone (393×852 portrait)** — reorganises rather than truncates. The
map draws edge-to-edge behind every inset; the top 44 px is keep-clear for a
notch or punch-hole with a 108 px centre lane reserved and a gradient scrim
instead of an opaque bar; the app bar below it is the first row allowed to hold
controls; the domain rail is a 44 px column; tool options become a bottom sheet
and docks become full-height sheets, one at a time, with all five disclosure
levels intact; the bottom 26 px gesture inset holds no targets. In landscape the
cutout moves to a side edge and the same reserve applies horizontally.

Minimum target 44 px, measured inside the safe area, with no exceptions.
