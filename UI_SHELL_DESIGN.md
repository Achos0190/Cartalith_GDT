# UI shell design — DCC-style editor

> **Imported verbatim from the owner's Claude Design project "UI mockups
> planning", sync 2026-08-18T21:40Z.** This **replaces** the 2026-08-17
> version that was previously here: the design team rewrote it as a pure
> rationale document, moving all control-by-control detail into the new
> `DCC_SHELL_SPEC.md` (also imported, repo root).
>
> **The shell changed structurally in this revision** — see `DCC_SHELL_SCOPE.md`
> for what that means for the code already built. In short: the menu bar is now
> program-scope only with **seven** menus, and Generate / Simulate / Render /
> View became **workspaces on a domain rail** rather than menus.
>
> Path note: the design team writes to a `docs/`-rooted convention
> (`docs/UI_SHELL_DESIGN.md`). This repository keeps its scope and design
> documents at the root — `docs/` here holds the *source project's* own
> documentation (see `docs/README.md`). References below to
> `UNIFIED_TOOL_PLAN.md`, `GENERATOR_PARAMETERS.md`, `MENU_STRUCTURE.md`,
> `BIOME_AND_VISUALS_PLAN.md`, `ATLAS_ARCHITECTURE.md` and
> `SCULPT_EDITOR_INTEGRATION_PLAN.md` follow that convention; in this repo the
> port's own equivalents are `UNIFIED_TOOL_PLAN.md` and
> `GENERATION_PARAMETERS.md` at the root, while the source project's versions
> live under `docs/`. **They are different documents with the same names** —
> `docs/README.md` records which is which.

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
