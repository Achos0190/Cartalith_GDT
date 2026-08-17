# DCC shell: supersedes GUI_SHELL_SCOPE.md's target, real milestone plan

Owner-supplied design import (2026-08-17, same Claude Design project as
before — "UI mockups planning," via `claude_design` MCP). Owner's own words,
verbatim: *"to be certain this, the dcc shell, is the design that should be
followed religiously and needs to fully replace the current gui."* Unambiguous:
this is not an alternative direction to weigh, it is the new target, in full.

Design assets imported verbatim: `UI_SHELL_DESIGN.md` (repo root — the rule
set), `design/Cartalith DCC Shell.dc.html` (the mockup: 1920×1080 desktop,
2560×1600 tablet, 393×852 Android phone), `design/Cartalith Menu Structure
v2.dc.html` (the five-level disclosure grammar this shell's menus use). The
design team's own `github.md` (not imported as a repo file, its content noted
here) says explicitly: *"Cartalith GUI.dc.html (earlier panel-based
directions, superseded)"* — confirming the navigator/panel shell built this
session (`GUI_SHELL_SCOPE.md`, commits `5d44c6b` through `2dee8fc`, including
the Fable-5 ultracode declutter pass) is now superseded in full, not merely
extended.

## What actually changes, structurally

The prior shell was a **panel browser**: a left navigator whose subjects swap
a parameter panel and an inspector around a static viewport, closely modeled
on the reference HTML's own scrolling-form layout. This is a **DCC editor**:
a persistent menu bar (program-level only), workspace tabs (what the old
navigator's groups became — WORLD/CIVILIZATION/INFRASTRUCTURE/CARTOGRAPHY/
RENDER, now a single row instead of a permanent column), a **left tool rail**
of map-editing tools, a **tool options bar** for the active tool's live
parameters, a right dock (Layers/Properties/Sample/History/Assets), the
viewport, and a status bar. `UI_SHELL_DESIGN.md`'s own table is the governing
split; read it before touching layout code.

**The load-bearing new idea is the tool rail plus its editing model** — Select/
inspect, Pan, Point sample, Raise/lower, Smooth, Flatten/terrace, Stamp,
River/water, Biome paint, Place settlement, Draw route/way, Territory/faction,
Label, Icon stamp, Measure, Region select/export. A tool's stroke goes into a
**pass buffer**, visible immediately; **Commit pass** writes it to the field,
**Discard** drops the buffer; downstream stages are marked **stale** rather
than recomputed mid-stroke.

## Why this splits into two genuinely different tracks

**Track 1 — shell restructure.** Layout, navigation, menu contents, visual
language. This is real UI/UX work of the same shape every prior GUI pass this
session has done: re-parent every currently-working control into its new
home, keep the golden path intact, verify with real screenshots. Buildable
now, independent of Track 2.

**Track 2 — the tool system.** Interactive brush-based editing of a
*generated* world (raise/lower terrain, paint biomes, draw rivers, place
settlements by hand) with a pass-buffer/commit/discard/staleness model is
**new engine capability that does not exist anywhere in this port today.**
This is not a UI gap the way an inert menu item is — the engine is a one-shot
static generator (`HARDWARE_ACCELERATION.md`'s own correction, reaffirmed
throughout this session). Two real things already exist that this would
build on:

- **`cartalith-spatial`** (`LOD_TILING_BASE_SCOPE.md`) — `TiledField<T>`,
  `QuadTree<T>`, and critically `DirtyTracker` (per-tile dirty flag + version
  counter), built standalone and unintegrated *specifically* "for whenever...
  a real large-world need actually triggers... integration." A real
  interactive-editing pass buffer with staleness propagation is exactly that
  trigger. This is the first concrete reason to integrate it since it was
  built.
- **The reference app's own Sculpt editor** — `MVP_SCOPE.md`'s "Out of scope"
  table lists it (`block 1`) but **nothing in this project has investigated
  it yet.** It is the one place in the whole codebase (JS or Rust) where
  "paint terrain directly" already has a real, shipped implementation to
  read from, rather than being invented fresh. Investigate before designing
  the pass-buffer/commit/discard model from scratch — the reference may
  already have solved parts of this (brush falloff, undo granularity, what a
  "stroke" actually mutates) with real, load-bearing decisions worth reusing.

`UI_SHELL_DESIGN.md` itself names the missing piece directly: *"`docs/
UNIFIED_TOOL_PLAN.md` decides what a tool *is*; this document decides where
it appears."* That document does not exist yet, in this repo or the design
project. Writing it — grounded in the reference's real Sculpt editor, not
invented — is the real prerequisite for Track 2, the same way `JOURNEY_
PLANNER_SCOPE.md`/`ASSET_LIBRARY_SCOPE.md` had to exist before those efforts
could be milestoned honestly.

## A concurrent, now-partially-stale piece of work

`GUI_FEATURE_PARITY_SCOPE.md` was commissioned this session (before this
import) to audit the *previous* shell's inert menus against the reference
app and comparable tools. Its category-1/2 findings (real engine backing
that exists vs. genuinely missing) remain valid — those are about the Rust
engine, not shell layout. Its category-3 findings (which mockup-invented
controls are worth building) may not map cleanly onto the new eight-menu/
tool-rail structure. Whoever picks up that document next should cross-check
its recommendations against `UI_SHELL_DESIGN.md` rather than assume they
transfer unchanged.

## Milestone plan

**Milestone 1 (dispatched)** — shell restructure, desktop 1920×1080, dark
theme only (matching the existing shell's own sequencing: light theme/
responsive breakpoints were already deferred once, stay deferred here too).
Build the six regions per `UI_SHELL_DESIGN.md`'s table. Re-parent every
currently-real control (generation params, sea level, world shape, the four
experimental flags, load-save, credits, all map-overlay toggles, the
causal-chain Inspector data) into its new home — into menu items where the
control is a program-level action, into the right dock's Properties/Sample
where it's a live value. The left tool rail's ~17 tools are built and
visible, honestly inert (no pass-buffer/commit/discard exists yet) — same
"shell now, wire later" discipline `GUI_SHELL_SCOPE.md` milestone 1 already
established successfully. Workspace tabs replace the old navigator groups.
Menu bar replaces the old top-bar's 7-menu set with the new 8-menu set
(File/Edit/Generate/Simulate/Render/Assets/View/Help) per `UI_SHELL_DESIGN.md`
§"Top menu bar" — note this is a real content change, not just a rename
(Edit and Help are new; Project/World/Map are restructured into File/Generate/
Render).

**Milestone 2 (dispatched, parallel, no code)** — write `UNIFIED_TOOL_PLAN.md`
for real: investigate the reference's Sculpt editor (`reference/Cartalith
Gen1 v2.10.html`, grep `reference/FUNCTION_INDEX.md`), determine what each of
the ~17 tool-rail tools actually needs to mutate at the engine level, which
already have a real reference implementation to port (Sculpt-editor-backed
tools) versus which are genuinely new interaction the reference never had
(likely: Draw route/way and Territory/faction as *manual* tools, since this
port's own road/territory generation is algorithmic, not hand-drawn — verify
rather than assume), and what the pass-buffer/commit/discard/staleness model
requires from `cartalith-spatial`'s existing `DirtyTracker`/`TiledField`.
This document is planning only — it does not implement the tool system, it
scopes it honestly, the same way `JOURNEY_PLANNER_SCOPE.md` scoped Journey
Planner into six real milestones rather than attempting it whole.

**Milestone 3+ (not yet dispatched)** — the tool system itself, milestoned by
whatever Milestone 2 finds. Expect this to be large — potentially comparable
to Journey Planner or the Asset Library in scope, since it is genuinely new
engine capability, not a port of already-computed data.

## Hard constraint, unchanged from every GUI pass this session

The real, working golden path must not regress at any point: seed/resolution/
map width/sea level/world shape must reach generation; every real map-overlay
toggle, the causal-chain Inspector, load-save, and credits must keep working.
A beautiful new shell that breaks generation is a regression, not a redesign.

## Verification

Same bar as every prior shell milestone: `cargo build`/`cargo test
--workspace` (0 regressions), `godot4 --headless --quit` clean load, and real
windowed-app screenshot verification end-to-end through the new shell —
compared against `design/Cartalith DCC Shell.dc.html`'s own 1920×1080
reference for structural and visual fidelity.

## Done means (milestone 1)

The desktop dark-theme DCC shell exists as real Godot scenes/Control nodes
matching `UI_SHELL_DESIGN.md`'s six-region structure and the mockup's visual
language, every currently-real feature is reachable through it, the tool
rail is present and honest about what doesn't work yet, and the golden path
is screenshot-verified unbroken. Milestone 2's `UNIFIED_TOOL_PLAN.md` gives
whoever picks up milestone 3 a real, scoped target instead of a green field.

## Milestone 1: done (2026-08-18)

Dispatched, then interrupted mid-flight by an account-level API error with
no recoverable transcript — real, uncommitted work was left sitting in the
working tree (`main.gd`/`main.tscn`/`map_overlay.gd`). This entry is from
the pass that picked it back up: first assessed what was actually there
(`git diff` against all three files, full reads, not an assumption in either
direction), found it **substantially complete and structurally sound**, then
verified and finished it rather than discarding real work per this project's
own reversible-action discipline.

**State found**: all six `UI_SHELL_DESIGN.md` regions built as real Control
nodes; the 8-menu bar's real content change (not a rename) done correctly;
every currently-real control re-parented (generation params into a File >
New World dialog, load-save, credits, the four experimental flags +
villages, all three map-overlay toggles now independent per
`GUI_FEATURE_PARITY_SCOPE.md` item #9, the causal-chain inspector moved to
click-to-pin per item #10, asset-pack import wired per item #1); the
16-tool rail across 5 groups, honestly inert, present with real tooltips
naming what doesn't work; workspace tabs restyling tool-rail emphasis
without touching the viewport, exactly as specified. `cargo build -p
cartalith-godot` and `cargo test --workspace` both passed clean on first
attempt against the prior fork's GDScript — zero corrections needed to any
Rust-facing call.

**What this pass completed/fixed**: one real gap — the status bar's own
"active tool's modifier hints" slot (`StatusHintLabel`) had no
`unique_name_in_owner` and was never written by `main.gd`, so it stayed
"no active tool" even with a tool selected, while the Tool Options Bar
correctly showed the tool's name. Two chrome regions disagreeing about the
same state is exactly the kind of inconsistency this milestone's own bar
exists to catch. Fixed: `_on_tool_selected` now sets it honestly
(`"RAISE / LOWER selected -- no pass-buffer/commit/discard yet"`).

**Judgment call — tool-options-bar/status-bar honesty**: the prior fork had
already made the right call, worth recording since this doc explicitly asks
for it to be judged carefully. The Tool Options Bar shows no fabricated live
parameters — just the tool's name plus one hint line naming the real reason
nothing is live yet. A version reading "RAISE / LOWER · commit pass" with no
working pass-buffer behind it would have been actively misleading, worse
than an honest inert placeholder, per this document's own framing of the
risk. What shipped avoids that trap consistently across every new menu item
and dock control, not just the tool rail.

**Known pre-existing issue, not fixed (out of this milestone's scope)**:
unchecked `CheckBox` nodes in the right dock render with no visible glyph
against `theme/dark_theme.tres` — the theme sets `checkbox_unchecked_color`
but doesn't populate Godot's separate `CheckBox` icon theme items. Confirmed
functional regardless (toggling still works, screenshot-verified); this
predates the milestone (the theme resource isn't part of this diff), so
it's recorded here rather than patched as scope creep.

**Verification**: `cargo build -p cartalith-godot`/`cargo test --workspace`
0 regressions. `godot4 --headless --quit main.tscn` clean load, no script/
parse errors. Real windowed-app screenshot verification end-to-end
(`PrintWindow`/`mouse_event`/`SetCursorPos` automation, maximize/restore
focus-forcing trick, this session's established technique): New World
dialog → Generate (seed 12345, 2K, 800 km, Classic, sea level 42%) →
2048×2048/40-settlement world rendered correctly with terrain/settlements/
roads/sea routes; Territory (faction fill) and Province boundaries toggles
both confirmed rendering correctly and independently of the other three
layers; settlement hover (on-canvas card + Sample dock) and click-to-pin
(Properties dock's full "WHY HERE?" causal chain, persists across
subsequent layer-toggle clicks) both confirmed; File > Open project (.zip)
opened the real save dialog rooted at the project directory and cancelled
cleanly without disturbing the generated world; Help > Credits opened with
its full academic-principles text; tool-rail selection and workspace-tab
switching both confirmed structurally correct against `UI_SHELL_DESIGN.md`'s
own rules (tab switching restyles tool-rail emphasis only, never the
viewport or the already-selected tool's highlight). Full record:
`cartalith-native/docs/CHANGELOG.md`'s "DCC shell milestone 1" entry,
`cartalith-native/docs/STATUS.md`'s own DCC shell section.

Milestone 2 (`UNIFIED_TOOL_PLAN.md`) and milestone 3+ remain not yet
dispatched, unchanged by this pass.
