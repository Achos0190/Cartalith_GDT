# DCC shell: supersedes GUI_SHELL_SCOPE.md's target, real milestone plan

> **This document defines the DCC shell's milestones, its design rules and the
> decisions behind them. It does not track them.** Where the work stands is in
> **`cartalith-native/docs/STATUS.md`**, which is the only place progress is
> recorded. The dated "as built" sections below are records of the passes that
> ran, not a statement of the shell as it stands today.

> ## 📐 WHICH CANVAS WINS (owner, 2026-08-25) — read this before resolving any design conflict
>
> Owner's words, verbatim: *"In essence always follow the newer design or use
> the design principles from the DCC versions to fill in gaps."*
>
> Two rules, in order:
>
> 1. **The newer canvas wins.** Where two artboards disagree, the later one is
>    authoritative — not the more detailed one, not the one a scope document
>    happens to cite, and not the one already implemented.
> 2. **Where no canvas exists, derive from the DCC canvases' own vocabulary.**
>    Fill the gap by extending what they establish; do not invent a treatment,
>    and do not copy one in from outside this design system. Say in the pass's
>    write-up which surfaces were *designed* rather than *matched*.
>
> A third rule follows from the first and is worth stating because it has
> already caught a pass out: **an owner decision is newer than any canvas.**
> `Data ▸ Conversion` is still drawn in the canvas and was removed by owner
> decision on 2026-08-20; the canvas is stale there, and the shell is right.
>
> ### What this settles, as of 2026-08-25
>
> | Conflict | Resolution under the rule |
> |---|---|
> | Phone geometry: `DCC Shell.dc.html` §13 (393 dp) vs `Cartalith Android Phone.dc.html` (412 dp) | **412 dp** — already ruled separately the same day, and the rule agrees. The migration it implies is described below |
> | Phone bottom-nav **tab set**: the 412 canvas's five pre-v3 domains vs `Cartalith Menu Structure v3.dc.html` | **v3's domain set.** v3 is newer than the phone canvas. So the migration takes 412's *geometry* and v3's *content* — the one place the two authorities split. The tab set is `WORLD · CIVIL · CARTO · PANELS · MORE` in the canvas's glyph-over-caption treatment |
> | Tool options bar: `DCC shell 1920`'s one 34 px row vs `Cartalith Paint Toolbar.dc.html`'s two ruled 38 px rows | **Paint Toolbar** — it is the later artboard and draws a component `DCC shell 1920` does not. Measured 78 px: two 38 px rows with the rule between |
> | Dropdown check marks: canvas's typographic `●`/`○` vs Godot's stock radio icons | **The canvas.** No newer authority disagrees. Godot draws that column from theme *icons*, so the marks have to be drawn as a filled disc and a hairline ring rather than set as type |
> | Menus with no artboard at all (Edit, Window, Help, Preferences' bands, World Data's phone list) | Rule 2 — derive from the drawn menus, and disclose which |
>
> **Newer sources since this table was written.** Rule 1 applies to them too:
> `design/dcc-environment-2026-08-31/` (the replacement plan and its
> `spec/01…06` section specs — `06-phone.md` is a newer phone authority than
> the 412 canvas) and `design/mcp-2026-09-07/` (which governs corner radius,
> by owner ruling 2026-09-07 — `DCC_SHELL_SPEC.md` §11). Where the table above
> names a winner, check it against these before building.
>
> ### The 412 phone migration, 2026-08-25
>
> `DCC_SHELL_SPEC.md` §13's phone column is **superseded, not merely disagreed
> with**, and §13 now opens with the item-by-item table of what the 412 canvas
> changes: a 28 dp status row on a solid ground (no keep-clear reserve,
> scrim or centre lane), 56 dp app bar, 64 dp bottom nav (`14px` glyph over a
> `9.5px` caption), 20 dp gesture inset with a `112×4` handle, and the ☰ side
> drawer replaced by the canvas's `02 Domain` full-screen drill. The shell's
> constant is `DccTheme.PHONE_REF_SHORT`. The migration's record — before/after
> numbers at 1440×3168 and 1080×2400, the pass on the owner's OnePlus 6T, and
> the five surfaces *designed* rather than matched — is `GUI_GAP_REGISTER.md`
> §53; its state is `STATUS.md`'s DCC-P412.
>
> Recorded here rather than only in `GUI_GAP_REGISTER.md` §51 because it
> resolves conflicts that have not happened yet, and every future design pass
> needs it before it starts rather than after it has guessed.

> ## ✅ THE HOLD IS LIFTED — BUILD IT (owner, 2026-08-18, later same day)
>
> Owner's words, verbatim: *"Replace the current GUI and replace it in full by
> the DCC version including all it's wiring and functionality."* — issued
> alongside a design sync (2026-08-18T23:05Z) that made the spec definitive.
>
> This **supersedes the hold** recorded earlier the same day (*"im reworking
> the UI completely, so hold off on that work for now"*). The rework has
> landed; `DCC_SHELL_SPEC.md` is now a 13-section control-by-control spec with
> §5.2 rewritten from v2.10's real `SCULPT_FEATURES` registry and a new §12
> banning emoji in favour of drawn SVG glyphs.
>
> **Scope of the instruction**: full replacement, not an incremental reskin —
> the shell *and* its wiring and functionality. That puts tool-system milestone
> F (shell wiring) inside this work rather than beside it.
>
> `DCC_CONTROL_INDEX.md` — the per-control audit this instruction was issued
> alongside — is described in the next section. Its largest finding, that the
> spec's ten-stage pipeline assumed **per-stage re-execution** the one-shot
> `generate_terrain` cannot offer, was settled the next day: verified against
> the reference under the owner's instruction, no version of the product runs
> a single stage, and the target is the reference's regenerate-on-release
> behaviour (`DCC_SHELL_SPEC.md` header correction #2).
>
> **Still true**: the Android measurements in `ANDROID_BUILD_SCOPE.md` (real
> per-region touch-target percentages, and the correction that the design's
> "44-52 px" must be read as *density-independent* pixels — ~86-102 physical
> on that device) are live input for the phone/tablet breakpoints in
> `DCC_SHELL_SPEC.md` §13.

## The control index comes first (2026-08-18)

Owner's instruction before any implementation: *"Before implementing the GUI I
want you to properly index all functions and buttons in the design and compare
it to the current program and functionalities."* Carried out as
**`DCC_CONTROL_INDEX.md`** (repo root): 452 controls from the spec as first
imported, one row each, with the `#[func]`, parameter key or crate function
behind each, a 2026-08-18 status (**wired** / **backed, unwired** / **engine
gap** / **new**), and the summary the owner asked for — the engine gaps by
size, where spec and engine disagree, which of the old shell's built work the
design deletes or relocates, and the decisions only the owner can make. **Its
status column is a snapshot of that day, not the state of the shell** — that
is `STATUS.md`'s.

Two of its findings bore directly on this document's milestone plan: the
spec's ten-stage pipeline does not partition the way
`GENERATION_PARAMETERS.md`'s eight engine groups do and assumed per-stage
re-execution (settled 2026-08-19, above); and the design had no surface for
the ~11 non-terrain tools whose engine halves `UNIFIED_TOOL_PLAN.md`
milestones C-E own, so milestone F's target changed shape, not just its skin.
The second was closed by the spec's 2026-08-19T00:20Z revision, whose §4.5
Tool palette gives every such tool a surface (the gap `STRANDED_TOOLS.md`
counted).

## The design import this plan answers (2026-08-17)

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

*This section and the milestone plan below were written against the
**2026-08-17** import (eight menus, a left tool rail, five workspace tabs). The
2026-08-18 revision restructured it — seven program-scope menus, a vertical
domain rail, and (2026-08-19) the tools moved into each left dock's TOOLS
block — and the domains later merged to three. The reasoning here still
holds; the region list is `DCC_SHELL_SPEC.md`'s.*

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

*The inventory in this section is the state of the port on **2026-08-18**, and
it is the argument for the two-track split — the tracks are shaped by what
existed and what did not on that day. Read it for the reasoning; the tools it
calls absent are what Track 2's own milestones (`UNIFIED_TOOL_PLAN.md` A-F)
were written to build.*

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

## A concurrent piece of work, and how it was re-pointed (2026-08-18)

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

**Milestone 1** — shell restructure, desktop 1920×1080, dark
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
Render). *The 2026-08-18 revision then replaced that eight-menu set with seven
program-scope menus (File · Edit · Assets · Data · Preferences · Window · Help,
`DCC_SHELL_SPEC.md` §2); Generate, Simulate, Render and View became
workspaces.*

**Milestone 2 (parallel with milestone 1, no code)** — write `UNIFIED_TOOL_PLAN.md`
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

**Milestone 3+** — the tool system itself, milestoned by whatever milestone 2
finds. Expect this to be large — potentially comparable
to Journey Planner or the Asset Library in scope, since it is genuinely new
engine capability, not a port of already-computed data.

**Read the two paragraphs above as the plan, not as open work.** Where
milestone 2 and milestone 3+ stand is `STATUS.md`'s DCC-T2 and DCC-T3 rows
(milestone 3+ *is* `UNIFIED_TOOL_PLAN.md`'s A-F, rows UTP-A…UTP-F). The
deliverable of milestone 2 is `UNIFIED_TOOL_PLAN.md` itself, at the repository
root, with the tool-by-tool table and the A-F breakdown; its "Milestone F as
built (2026-09-01)" section is the tool-by-tool evidence for milestone 3+.
**The two did not happen on the same day** — milestone 2 ran on 2026-08-18,
milestone F closed on 2026-09-01 — and a 2026-09-06 correction here briefly
said otherwise before a verifier caught it. Collapsing two dates into one made
a fortnight of work look instantaneous, which is the kind of tidy-sounding
claim this document exists to stop.

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
reference for structural and visual fidelity — or, where a newer canvas
supersedes it, against that one (the rule at the top of this document).

## Done means (milestone 1)

The desktop dark-theme DCC shell exists as real Godot scenes/Control nodes
matching `UI_SHELL_DESIGN.md`'s six-region structure and the mockup's visual
language, every currently-real feature is reachable through it, the tool
rail is present and honest about what doesn't work yet, and the golden path
is screenshot-verified unbroken. Milestone 2's `UNIFIED_TOOL_PLAN.md` gives
whoever picks up milestone 3 a real, scoped target instead of a green field.

## Milestone 1 as built (2026-08-18)

*The three "as built" sections below are dated records. The files they cite —
`main.gd`, `main.tscn` and the Generate menu — no longer exist: the shell boots
`shell/app.tscn`, and the capabilities were re-homed (the stage dialogs into
the WORLD workspace's dock, `shell/workspaces/world_workspace.gd`; the World
Setup dialog into `shell/new_world_dialog.gd`). `STATUS.md`'s DCC-1 / DCC-2 /
DCC-3 rows carry the re-pointed evidence.*

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
`cartalith-native/docs/CHANGELOG.md`'s "DCC shell milestone 1" entry (retired,
kept as history).

## Milestone 2 (GUI track): the Generate menu's parameter dialogs — as built (2026-08-18)

Note on numbering: "milestone 2" was originally the *planning* task above
(`UNIFIED_TOOL_PLAN.md`, the tool track). This is the second **shell** GUI
milestone and runs on the other track — the two are independent, and the tool
track's numbering is unaffected.

Dispatched from the owner's directive *"make all generation options active in
the current interface so that we have the same functional controls as the
older html version."* This is the GUI half; the engine half landed in
parallel as `GENERATION_PARAMETERS.md` (7 → 58 parameters reachable from
GDScript through a flat dotted-key API). **Their API was the contract, and it
is what this consumes** — this milestone hardcodes no ranges of its own.

**What shipped** (`godot-project/main.gd` only; `main.tscn` untouched, the
dialogs are built at runtime):

- Six live stage dialogs — Tectonics, Volcanism, Erosion, Hydrology, Climate,
  Settlements — **57 controls, all functional end to end**. The other four
  stages in `UI_SHELL_DESIGN.md`'s list (Glacial & coastal, Ecology,
  Infrastructure, Politics) stay visibly present and disabled, each tooltip
  naming the real reason: the engine has no parameters for them, because
  those passes are unported (glacial, coastal) or have no dials in either
  engine (ecology, infrastructure, politics).
- Dialogs, never persistent panels — `UI_SHELL_DESIGN.md`'s governing rule
  for the whole menu bar, held.
- The five-level disclosure grammar from `design/Cartalith Menu Structure
  v2.dc.html`: menu bar (1) → Generate menu (2) → stage dialog (3) → a
  section per `params.rs` group (4) → that section's collapsed **ADVANCED**
  fold (5), holding "only dials whose defaults are already correct".
  Advanced membership follows a rule rather than taste: a parameter is
  Advanced if the reference itself buried it (its `<details class="adv">`
  *Physical coupling fields* block) or if the reference never exposed it at
  all and this port surfaces it as a superset (`DECISIONS.md` §7d).
- Real reset at two granularities: per-dialog *Reset this stage*, and
  Generate → *Reset all generation parameters* delegating to the engine's own
  `reset_params()`.
- Six parameters proxied rather than duplicated: the four experimental flags
  and village seeding already had working controls in File > New World, so
  their stage rows drive those existing `CheckBox` nodes directly. One source
  of truth; verified in the app that toggling village seeding in Generate >
  Settlements flips the New World checkbox too.
- Two parameters deliberately excluded, reasons recorded in code
  (`EXCLUDED_KEYS`): `sea_level` (New World owns it via `set_sea_level()`),
  and `use_gpu` (`GPU_LAYER_INTEGRATION_SCOPE.md`'s current milestone is
  still the GPU-safe noise redesign; per `DECISIONS.md` §7c the GPU path
  produces a *different* world for the same seed, so surfacing the switch now
  would expose an incomplete path).

**Staleness — the decision this milestone was asked to make honestly.**
`UI_SHELL_DESIGN.md` says each Generate stage "reports staleness". Two facts
made a per-stage staleness indicator the wrong thing to build in this pass: no
staleness system existed yet (`UNIFIED_TOOL_PLAN.md` scopes it as milestone A),
and — the load-bearing one — **the engine is a one-shot generator**. `generate_terrain` runs the whole pipeline or none of it, so
there is no per-stage incremental recompute for a stage to be stale
*relative to*. A per-stage "stale" pip would advertise exactly the
incremental pipeline that does not exist, which is worse than showing
nothing.

**Decision: no per-stage staleness indicators.** Instead each dialog carries
an honest regenerate-to-apply affordance — a footer line stating plainly that
the whole world is regenerated and there is no per-stage recompute, a
status-bar note when a parameter has changed since the last generate, and a
*Generate now* button whose own tooltip says it runs the same single full
pass File > New World's Generate runs. When the tool system's real staleness
model lands, this is the natural place to upgrade; until then it claims
nothing the engine cannot do. *(`STATUS.md`'s DCC-2 row records this decision
as superseded by the staleness slot of `UNIFIED_TOOL_PLAN.md` milestone A —
staleness left behind by committed **tool** passes. Dials still regenerate the
whole world, per `DCC_SHELL_SPEC.md` header correction #2.)*

**Real parity gaps found, recorded rather than papered over.** These are
genuine parity information, not wiring gaps — each belongs to a pipeline
stage `cartalith-engine` has not ported, and each is stated in the relevant
dialog's own header text so a user reads it where it matters:

- Droplet hydraulic erosion, hillslope diffusion, velocity (momentum)
  erosion and evolve-and-sediment are all separate *manual* ops in the HTML
  app with no engine equivalent — the Erosion dialog exposes only the
  stream-power carve that generation actually runs, and says so.
- Glacial erosion and the coastal pass are unported outright, so that stage
  has no dialog at all rather than an empty one.
- The graph-driven orogeny switch is omitted by the engine; its three dials
  (fold intensity, trench depth, fault blocks) are hardcoded to the values
  the reference's own defaults produce, so behaviour matches — exposing them
  needs three new fields threaded through `OrogenyParams`.
- Geoid, tides, seasons and Köppen classification are unported (all
  default-off in the reference).
- Min stream order is a render filter, not a generation parameter; it belongs
  with the Render menu's map-mode work.

**Two honest deviations from the reference's own presentation**, both
recorded: value-readout precision is derived from each parameter's step
rather than copying each reference span's `toFixed` (agrees everywhere except
`Uplift spread`, `18.0 px` here vs `18px` there); and `flexure`/`hetero` ship
in the reference with a static HTML slider position that contradicts its own
`state` default — the reference overwrites both in `syncUI` (v2.10 line 12656), so
the `state` default is the real one and is what these dialogs show.

**Verification.** `cargo build -p cartalith-godot` clean; `cargo test
--workspace` 563 tests across 83 binaries, 0 failures, 0 regressions;
`godot4 --headless --quit main.tscn` clean load. Then the load-bearing check
— real 1920×1080 windowed app, seed 12345 / 2048×2048 / 800 km / Classic,
**one parameter changed at a time** so attribution is unambiguous, proving
control → engine → visibly different world across five parameters in five
different structs: `tect.plates` 14→40 (`TectonicParams`); the two climate
temperatures to minimum (`ClimateInputParams` — identical coastlines, fully
glaciated world, the expected terrain/biome decoupling); `volc.count` 20→100
(`VolcanismParams`); `crater.count` 100→200 (`CraterParams`); `river_density`
×1→×3 (`WorldParams`). *Reset this stage* confirmed restoring exact defaults.

**Golden path re-verified, no regressions**: generation end-to-end from both
entry points, all five map-overlay toggles, the causal-chain Inspector on
hover *and* click-to-pin with the pin surviving subsequent layer toggles,
Credits, and File > Open project's dialog.

**Deferred by this pass, unchanged**: light theme, responsive breakpoints, and all
tool functionality (not permanently — see the note at the end of this
document). The pre-existing `dark_theme.tres` unchecked-`CheckBox` glyph issue
recorded under milestone 1 is unchanged and visible in these dialogs too.

## Milestone 3 (GUI track): the World Setup dialog — as built (2026-08-18)

Owner's own request, verbatim: *"maybe we should start thinking about a
proper base setup menu where we can pick map size, resolution, dimensions -
basically expanded from the current html version."* `UI_SHELL_DESIGN.md`
puts "New world" in the File menu and rules that menu items open **dialogs,
never persistent side panels**, so this is File ▸ New world grown from
milestone 1's parameter carry-over into a real world-setup gate.

It is the GUI half of the non-square work commit `22ae75b` landed on the
Rust side (`GENERATION_PARAMETERS.md` "Map dimensions and aspect ratio").
Nothing in Rust changed this pass: the API it needs already existed.

**What the dialog is.** One new section, `MAP SIZE, RESOLUTION &
DIMENSIONS`, prepended to the existing New-world section list (seed/sea
level, world structure and the advanced fold stay exactly where milestone 1
put them). Its rows share one grammar — **label · guided preset · exact
value** — so the pattern is learned once:

| Row | Preset column | Exact column |
|---|---|---|
| Extent | Region / Whole world | — |
| Map width (km) | Local 200 · Province 800 · Region 2 000 · Subcontinent 5 000 · Continent 12 000 · Planet 40 075 · Custom | the reference's own free km entry |
| Resolution (columns) | 512 / 1K / 2K / 4K / 8K / Custom — the reference's own segment | free grid width, 4–8192 |
| Aspect (rows) | 2:1 · 16:9 · 1.5625:1 reference region frame · 4:3 · 1:1 · 3:4 · 9:16 · Custom | free grid height |

Below them a live derived readout — **Grid** (cells and total), **Extent**
(km × km), **Cell size** (km per cell), **Aspect** (ratio and
landscape/portrait/square) — recomputed on every change, so picking 1K in
region mode shows the real 1024 × 512 grid and the real km extent of both
axes before anything is generated.

**How it expands on the reference.** The reference ships a Working-resolution
segment that sets the width only, one free "Map width (km)" number input with
no scale to judge it against, an extent segment, and no aspect control at
all — its `gridH()` hardcodes 2:1 in world mode and 1.5625:1 otherwise. Both
of those reference ratios are here **by name**, so nothing the reference does
became unreachable; the additions are the aspect choice itself, the map-width
scale presets, free entry beside every preset, and the derived readout.
`DECISIONS.md` §7d permits this: behaviour is preserved, the superset is
recorded.

**The three engine rules the design is built around**, taken from
`GENERATION_PARAMETERS.md` rather than re-derived:

1. **Cells are square in kilometres.** Every km↔cell conversion in the
   workspace comes from the single quotient `map_width_km / gw` applied to
   both axes, so map height in km is `width_km × gh / gw` — **derived**.
   There is deliberately no height-in-km control; it is a readout, and the
   dialog's own header text says so rather than leaving the absence to look
   like an oversight.
2. **World mode is physically 2:1.** X wraps 360° of longitude over `gw`, Y
   spans 180° of latitude over `gh`. Choosing Whole world pins the aspect to
   2:1, takes the row count from `WorldGen.reference_grid_height(gw, true)`,
   and disables the aspect and row controls **with the reason stated in prose
   directly above them** — a silently greyed control reads as a bug.
3. **Grid height is a call argument, not a stored parameter**, because it
   reallocates every field in the pipeline. It sits beside seed, map width
   and resolution as an argument to `generate_sized()`.

**Built at runtime from the engine's own metadata**, following the pattern
milestone 2 established: no constant the engine owns is copied into
GDScript. Both reference `gridH` factors (0.5 / 0.64) are asked of
`reference_grid_height()`, extent is stored through `set_params({"world":
…})`, and the post-generation readout reads `get_map_width_km()` /
`get_map_height_km()` back rather than echoing what the dialog asked for — so
a disagreement between the setup readout and the generated world would be
visible instead of assumed away. The two scene-authored controls this section
needs (`%ResolutionInput`, `%WidthInput`) are **re-parented** into the new
rows, not duplicated: one node per value.

**One value, two surfaces, one node.** `world` is a real generation
parameter, so the Generate ▸ Climate dialog legitimately shows it *and* the
setup dialog owns it as a creation-time shape decision. Rather than exclude
it from one side, it became a `PROXY_KEYS` entry onto the Extent control —
the same mechanism milestone 2 already used for the four experimental flags.
Verified live in the app: flipping the Climate dialog's checkbox moves the
Extent selector, writes the engine parameter, disables the aspect control and
re-derives the grid to 2048 × 1024.

**Honest guidance rather than discovery-by-waiting.** Two warnings appear
under the readout when they apply: 4K/8K grids are memory- and time-heavy on
this port's CPU-only pipeline (milestone 1's static hint, now conditional and
covering the row count too), and aspect ratios past ~16:1 are degenerate —
non-crashing, but the coarse weather grid loses almost all resolution across
the short axis and the plate frame swallows a large fraction of the sheet
(the finding the Rust non-square pass recorded).

**One real bug found in the existing dialog**: `%WidthInput`'s `max_value`
was 40 000 km, so the natural "Earth's equator" figure of 40 075 km silently
clamped. Raised to 100 000 with a step of 5 (40 075 is a multiple of it).
Caught by the screenshot verification, not by reading.

**Verification.** `cargo build -p cartalith-godot` clean. `cargo test
--workspace`: **719 tests across 88 binaries, 0 failures, 0 regressions**
(the count grew from milestone 2's 563 because sibling forks added
`cartalith-urban` and a terrain sculpt module; `cartalith-civ` compiled fine
despite the flagged possibility of mid-edit sibling state).
`godot4 --headless --quit main.tscn` loads clean, with warnings identical to
the pre-change baseline (the two RID/ObjectDB lines are pre-existing —
checked by stashing this change and re-running, not assumed).

Then the load-bearing check — real 1920×1080 windowed app, driven through
this dialog, each shape's readout compared against what
`get_map_width_km()`/`get_map_height_km()` reported after generating:

| Shape | Asked | Engine reported | Rendered |
|---|---|---|---|
| 2:1 landscape, Earth-like | 1024 × 512, 2 000 km | 1024 × 512, 2000 × 1000 km | correct 2:1 plate, not stretched |
| 3:4 portrait, Classic | 768 × 1024, 1 500 km | 768 × 1024, 1500 × 2000 km | correct portrait plate, polar snow at the north edge |
| Whole world, Earth-like | 1024 × 512, 40 000 km | 1024 × 512, 40000 × 20000 km | 2:1 with **visible polar caps top and bottom**, sea lanes wrapping |
| 16:9, Archipelago | 640 × 360, 1 200 km | 640 × 360, 1200 × 675 km | correct 16:9 plate |

Every readout matched the engine exactly. `map_overlay.gd` needed no change:
its `_displayed_rect()` already fits with `min(size.x/gw, size.y/gh)`, so
markers, roads and sea lanes land on the right pixels at any aspect.

**Archetype dispatch re-verified** (the `a265b2b` bug, where World Shape
silently never reached generation): Earth-like and Archipelago both
dispatched through `generate_world_structure_sized` and produced their
characteristic worlds with real settlement counts, and the call's `bool`
return is still surfaced as a visible failure rather than swallowed.

**Golden path re-verified, no regressions**: generation from both entry
points, all five overlay toggles (territory fill and province boundaries
included), the causal-chain Inspector on hover **and** click-to-pin — driven
through `map_overlay`'s own real hit test at a settlement's own pixel, not a
hand-made signal emit — all six Generate stage dialogs building, and Credits.

**Deferred by this pass, unchanged**: light theme, responsive breakpoints, and all
tool functionality. Saving a *parameter set* as a named preset document is
the natural follow-up this milestone deliberately does not attempt.

**These three deferrals were not permanent** — each was taken up by a later,
independent pass, not by this one (note added 2026-09-06). Where each lives in
code, and where its state is recorded:

| Deferred here | Code | `STATUS.md` |
|---|---|---|
| Light theme | `dcc_theme.gd`'s `const LIGHT` | AND-4c, GGR-10 |
| Responsive breakpoints | `dcc_theme.gd`'s density sets (`TABLET`, `TABLET_PORTRAIT`, `LAPTOP`, the phone constants beside `PHONE_REF_SHORT`, desktop default) behind `is_touch()` / `is_phone()` / `is_laptop()`; the 412 dp phone migration | RP-S1, DCC-P412 |
| All tool functionality | the tool track, `UNIFIED_TOOL_PLAN.md` milestones A-F | UTP-A…UTP-F, DCC-T3 |
