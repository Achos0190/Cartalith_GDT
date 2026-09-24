# SCULPT_FUNCTION_CHART.md — the reference's sculpt editor against the port and the DCC design

**What this is:** the HTML's Generation ▸ Sculpt panel, feature by feature,
charted against the Rust port and `DCC_SHELL_SPEC.md` §5.2 — what has to
behave the same, and what the port does differently on purpose. **What it is
not:** status. Where the sculpt work stands is
`cartalith-native/docs/STATUS.md` (UTP-B, UTP-C, UTP-F and SL-0…SL-4). A bare
§ number inside a table below is a `DCC_SHELL_SPEC.md` section.

> Owner's request, 2026-08-18: *"see the sculpt function in the HTML version.
> It's under Generation → Sculpt, chart the functionalities and as stated we
> don't want copies from the javascript version. We want similar functioning in
> the new version."*

Charted from three sources read directly: `reference/Cartalith Gen1 v2.10.html`
1787–1855 (the `#genSculpt` markup) and 8821–~9470 (the behaviour);
`cartalith-terrain/src/sculpt.rs` (the golden-verified port of that
behaviour); and `DCC_SHELL_SPEC.md` §5.2. Where they disagree, the
disagreement is the finding.

## The distinction this document runs on

`DECISIONS.md` §7a: **golden parity binds the maths, not the markup.** The
falloff curve, the noise families, the per-feature amplitude formulas and the
stamp/commit model are the product and must behave the same, test-enforced.
The DOM plumbing that carried them — integer sliders with scale factors, emoji
labels, one scrolling column, a `<details>` accordion — is an artefact of a
single HTML file. §9 lists each artefact and what replaces it.

## 1 · Where the panel lives

| | HTML v2.10 | Native DCC shell |
|---|---|---|
| Reached by | Generate tab → **Sculpt** sub-tab (`#genSculpt`) | WORLD rail → the left dock's **PIPELINE \| SCULPT** mode pill (Ruling 6 restored the switch; Ruling L makes the mode the only gate, and picking a feature arms the `sculpt` tool) |
| Shape | One scrolling column, six stacked `.sec` blocks | Three regions: left dock (feature · preset · parameters · brush), tool options bar (the most-changed values, §4), right dock (stamp stack, §6) |
| Enable gate | none — *"being on this tab is the deliberate action… nothing here is destructive until Commit"* | same reasoning, same absence of a gate |
| Locked when | world finalized (`#sculptFinalizedNote`) | a baked world refuses the commit: `WorldGen::sculpt_commit` returns early when `bake.check(Mutation::HeightEdit)` fails |

The split follows the shell's own rule: frequently-changed values go
horizontal in the tool options bar, structure stays in the docks. Nothing is
dropped.

## 2 · The thirteen features

Ranges and defaults are the engine's, read out of `sculpt.rs` (the `*_CTL`
tables and `Feature::meta`); §5.2's table matches them exactly. "px" is the
reference's label and means **grid cells** at the working resolution
(`SculptGlobals::brush_size`'s doc).

| # | Feature | Interaction | Mode | Parameters (min–max, default) | Character |
|---|---|---|---|---|---|
| 1 | Mountains | stroke | add | Height 0.10–0.55 (0.42) · Peak sharpness 0.6–3.0 (1.5) · Ridge freq 0.6–5.0 (1.6) · Ruggedness 0–1 (0.55) | Ridged multifractal; edge noise tight (1.4 / ×1.5) |
| 2 | Hills | stroke | add | Amplitude 0.02–0.30 (0.11) · Rolling freq 0.5–4.0 (1.4) · Softness 0–1 (0.7) | Smooth FBM; soft edge (0.55 / ×0.9) |
| 3 | Ridge | stroke | add | Height 0.02–0.35 (0.15) · Width frac 0.1–0.6 (0.28) · Detail freq 0.5–4.0 (1.5) | One crest along the stroke, not a mass |
| 4 | Plateau | stroke | **set** | Rise 0.03–0.45 (0.26) · Terraces 1–8 (4) · Detail freq 0.4–3.0 (1.1) | Terraced mesa; **never lowers** existing terrain |
| 5 | Cliff / Escarpment | stroke, **direction-sensitive** | add | Rise 0.05–0.45 (0.22) · Steepness 0.2–1.0 (0.75) | The one hard-edge tool; high side is left of the stroke |
| 6 | Canyon | stroke | add (negative) | Depth 0.03–0.35 (0.18) · Wall steepness 0–1 (0.7) · Meander 0–0.8 (0.35) | Inverted ridged carve |
| 7 | Valley | stroke | add (negative) | Depth 0.03–0.30 (0.14) · Width frac 0.3–1.0 (0.85) · Meander 0–0.8 (0.3) | Broad U-shaped glacial trough |
| 8 | River | stroke | **set** | Width 2–26 px (7) · Depth 0.02–0.22 (0.09) · Meander 0–0.6 (0.28) · Branch noise 0–1 (0.5) | Semi-automatic; **writes water state** on commit |
| 9 | Lake | **radial**, brush = radius | **set** | Depth 0.03–0.30 (0.13) · Shore 0.05–0.6 (0.25) | Radial bowl; **writes water state** on commit |
| 10 | Basin | stroke | add (negative) | Depth 0.02–0.25 (0.1) · Floor rough 0–1 (0.4) | Endorheic sink — no outlet, unlike Lake |
| 11 | Coastline | stroke | **set** | Amount 0.1–1.0 (0.85) · Raggedness 0.4–4.0 (1.6) | Pulls toward sea level; raggedest edge of the thirteen (1.5) |
| 12 | Volcano | **radial**, own radius control | add | Cone height 0.15–0.6 (0.45) · Crater depth 0–0.9 (0.5) · Radius 30–200 px (110) · Flank rough 0–1 (0.6) | Cone + crater, ridged flanks |
| 13 | Freehand | continuous drag or tap | per sub-mode | Amount 0.02–0.30 (0.12) | Catch-all touch-up, eight sub-modes |

Three properties of this table are load-bearing:

- **Radial vs. path.** Lake and Volcano measure distance from the stroke's
  centroid; the other eleven measure signed distance to the polyline, so a
  stroke can meander. Meander (`ctx.meander(amp)`) is a sinusoidal centreline
  offset used by River, Canyon and Valley.
- **Per-feature edge character.** `edgeChar` / `edgeFreqMul` domain-warp each
  stamp's *coverage mask*, not its height — so a coastline frays and a
  mountain ridgeline stays tight. One edge treatment for all thirteen would
  flatten the library into one look.
- **Registry order is a seed input.** `FEATURE_KEYS`'s index feeds each
  stamp's noise seed (`(seed ^ ((i+1)*1013)) >>> 0`). The UI may re-*group*
  features visually but must not renumber them.

### Freehand's eight sub-modes

Shown only when Freehand is selected (`#sculptModeSeg`).

| Sub-mode | Follows |
|---|---|
| Raise · Lower · Smooth | the drag |
| Cliff · Ridge · Canyon | the drag's **direction** |
| Mesa · Volcano | a single tap (a one-point stroke degenerates to radial distance) |

## 3 · The eight presets

One click seeds a feature's parameters; **it never paints** — the user still
draws the stroke. Each of the eight overrides exactly one global
(`noiseScale`) plus its own feature's parameters (checked, `sculpt::Preset`).

| Preset | Feature | | Preset | Feature |
|---|---|---|---|---|
| Rolling Hills | Hills | | Volcanic Isle | Volcano |
| Alps | Mountains | | Mesa | Plateau |
| Rockies | Mountains | | Karst | Hills |
| Badlands | Canyon | | Glacial Valley | Valley |

## 4 · The shared brush & noise block

Applies to every feature. Ranges agree between HTML, engine and spec; five
of the eight defaults do not.

| Control | Range | Engine default | Spec §5.2 default | Notes |
|---|---|---|---|---|
| Brush size | 6–200 px | 32 | **64** | Shows the km equivalent at the working resolution (`#sBrushKm`) |
| Hardness | 0–1 | 0.5 | **0.35** | `feather = max(floor, R × (1 − hardness))` — narrows the falloff band as it rises |
| Intensity | 0–1.5 | 1.0 | 1.00 | `k = cov × intensity`. Coverage *shape* and effect *strength* are independently tunable — that is why both sliders exist |
| Noise scale | 1–20 | 5.0 | **6.0** | |
| Octaves | 1–8 | 5 | 5 | |
| Persistence | 0.20–0.90 | 0.5 | **0.52** | |
| Lacunarity | 1.40–3.20 | 2.0 | 2.00 | |
| Edge noise | 0–1 | 0.55 | **0.45** | Multiplied by each feature's `edgeChar` / `edgeFreqMul` |
| Seed | integer | — | project seed | Dice button randomises |
| Falloff | Smooth · Linear · Sharp · Constant | Smooth | five, incl. Custom | **Port addition** (`sculpt::Falloff`), not in the reference, which hardwires `smoothstep`. Custom is deliberately not built — see `Falloff`'s doc |

**Defaults, settled 2026-08-19: the engine's column wins, for all eight.** The
owner's ruling was that the values are placeholders and one set should be
picked. The engine's set is the pick on a ground that is not taste:
`cartalith-engine/tests/golden_parity_sculpt_water.rs` spreads
`..SculptGlobals::default()`, so those numbers are golden-parity *inputs*.
`sculpt_bridge::global_controls` reads each default live from
`SculptGlobals::default()`, so the control table cannot drift from them.
§5.2's table is what needs correcting, at the design end.

Three noise families back every feature — `sculptFbm`, `sculptRidged`,
`sculptBillow` — all reading the same four noise globals. That sharing is what
makes the block coherent rather than eight unrelated dials.

## 5 · The stamp stack

Each finished stroke becomes a live procedural object
(`{type, seed, pts, globals, featureParams, hidden}`) pushed onto a
session-scoped stack. The reference's own comment (line 9084): *"nothing here
touches `field` or triggers any recompute."*

| Operation | HTML | Native |
|---|---|---|
| List | `#sculptStampList`, newest first, with count tag | Right dock, §6's Stamp-stack context (`right_dock.gd::_build_sculpt`): index, visibility, type, parameter summary |
| Select | click a row | same; selecting re-populates the parameter block for re-tuning. The port adds multi-select (`sculpt_select_set`, `sculpt_select_all_stamps`) |
| Hide / show | `#sculptHideBtn` | same |
| Reorder | Move up / Move down | same — order is bake order, so it is meaningful |
| Delete | `#sculptDeleteBtn` | same |
| Re-tune | edit the selected stamp's parameters, live | same |

## 6 · Undo is two-tier, and stays two-tier

The part most likely to go wrong when rebuilding from the panel markup alone.

| Tier | Scope | Records | Cap |
|---|---|---|---|
| Draft undo | the stamp **list** | add · delete · reorder · hide | 30 snapshots (`SCULPT_HIST_MAX`, ported as `pass::HISTORY_MAX`) |
| Field undo | the **heightfield** | exactly one snapshot, at Commit | the global undo depth — `WorldGen::sculpt_commit` pushes one `undo::HeightUndo` entry, "Sculpt commit" |

Continuously dragging a selected stamp's slider does **not** push draft
history — the reference's comment (line 9298) calls this *"a reasonable, common
undo granularity"*, and it is right. *"Undo granularity is one committed pass,
not one stroke"* is tier two; that rule was quoted from `UI_SHELL_DESIGN.md` as
first imported (`db46908`), and today's `UI_SHELL_DESIGN.md` carries it as
"Non-destructive by default": nothing reaches the real heightfield until an
explicit Commit.

## 7 · Commit and discard

**Discard** drops the draft with a confirmation and touches nothing else.
Identical in both versions.

**Commit** bakes the whole stack in stack order, in one pass, then runs the
same post-edit tail the reference runs:

| | HTML v2.10 (`sculptCommit`, line 9317) | Native (`WorldGen::sculpt_commit`) |
|---|---|---|
| Bake | whole stack, one pass, in order | same — `commit_sculpt_pass` |
| River channels | one `enforceRiverChannels()`, plus one `enforceChannelDescent()` per river stamp — carving through rises so the river reaches its outlet, and **locking** those cells so later erosion cannot refill them | same |
| Lake water | one deposit into `lakeMask` | same |
| Flow and climate | one `computeFlow(true)` + one `refreshClimate()` | same, as one `refresh_climate` (its first step re-derives discharge): Height is marked changed at the touched tiles and `cartalith_engine::staleness::recompute_stale` re-runs hydrology and climate |
| Erosion | not re-run | not re-run — erosion is part of the Height stage (owner decision 2026-08-24, `cartalith-engine/src/staleness.rs` module doc) |
| Carve-time river network (channels, stream order) | not re-derived | not re-derived |
| Settlements, roads, territory | not re-run | not re-run; the port **reports** civ stale in the status bar's `stale` slot until Recompute civilisation |
| Undo | one `pushUndo()` | same |
| Render | one `renderNow()` | caller re-reads `build_color_texture()` |

**A number this chart once got wrong.** Its first draft deferred flow and
climate too, citing *"~7 s per stroke at 2048²"*. That is
`CPU_MULTITHREADING_SCOPE.md`'s **full generation** (`cartalith-terrain`
~5.1 s, ~7.07 s with the civ per-cell layer, climate, erosion and hydrology
excluded) — never a commit. `SCULPT_LIVE_SCOPE.md` L0 measured the whole
reference-shaped commit at ~123 / ~204 / ~564 ms CPU at 512² / 1024² / 2048²
(~94 / ~100 / ~131 ms with GPU flow and weather), and the commit has run the
tail since `8e666ac` (2026-08-24). What stays deferred is the cascade into civ,
for a structural reason: `cartalith-civ` operates on the whole field.

**`DCC_SHELL_SPEC.md` §5.2** says Commit *"re-runs erosion, hydrology and
climate once."* Hydrology and climate: right. Erosion: wrong. The spec's own
header correction #1, which says commit only marks tiles stale, predates
`8e666ac` and is out of date too.

## 8 · Water, and what constrains what

There is **no** generic "respect water mask" flag, and adding one would be
inventing a feature. The constraint is per-feature:

- **River** and **Lake** are the only features that *write* water state
  (`riverMask` / `riverFloor` / `lakeMask`) on commit.
- Every other feature may freely raise or lower over water. Coastline is
  defined entirely as pulling terrain toward sea level, so gating it on water
  would break it.
- The **categorical paint** tools (biome / terrain / splat) are a different
  tool family with a land-only gate — hard in the reference; in the port on by
  default and switchable (`Land only`, backed by `PaintStamp::ungated`). See
  `UNIFIED_TOOL_PLAN.md`, Biome paint.

## 9 · What we deliberately do not copy

Each row is a JavaScript- or DOM-specific artefact, not a feature.

| HTML artefact | Why it exists there | Native replacement |
|---|---|---|
| Integer sliders with scale factors — hardness `0–100`, intensity `0–150`, persistence `20–90`, lacunarity `140–320`, edge `0–100` | DOM `<input type=range>` was easier to keep integral | Real float ranges from one source of truth, the Rust registry (`Feature::meta`, `sculpt_bridge::global_controls`) — the reasoning `params.rs` applies to the generation parameters, so GDScript hardcodes no range, step or label |
| Emoji feature icons (⛰️ 🌋 💧 …) | free glyphs in a single file | §12's thirteen drawn terrain cross-sections, 1.2 px stroke, `currentColor` (`shell/dcc_icons.gd`) |
| One scrolling column with a `<details>` accordion | one file, one column | Disclosure across left dock, tool options bar and right dock (§4, §5, §6) |
| `sculptRenderOverlay`'s translucent outline / hatch, *"a deliberately simpler indicator than a full live-recolor"* (line 9242) | live recolour was too slow in JS | **A real draft preview.** Owner ruling 2026-08-18 (`SCULPT_LIVE_SCOPE.md`): `build_sculpt_preview_texture` returns the drafted colour and hillshade, not an outline |
| `_sculptNavPanLoop` / relocated joystick knob | single-finger drag is captured as a stroke, so panning needed somewhere else | On the phone, `ViewportHost._navpad` and two-finger pan (`viewport_host.gd`'s `InputEventPanGesture` branch); single-finger drag stays the stroke. Input routing, not a tool definition |

## 10 · What the design asks for that the reference does not have

Three blocks in `DCC_SHELL_SPEC.md` §5.2 have no counterpart in v2.10. They are
new design, not port work, scoped as `SCULPT_LIVE_SCOPE.md` L4.

| Spec block | What it asks for | Engine |
|---|---|---|
| **Brush shape** | eight built-in shapes (circle, directional, spatter, spiral, dots, cloud, checker, hatch) · Import brush… (greyscale height stamp) · Operation override (subtract / multiply / min / max) · Falloff (smooth / linear / sharp / constant / custom) · Rotation 0–360° · Spacing 0–1 · Mirror | **Falloff** has four of the five: `sculpt::Falloff` (Smooth, Linear, Sharp, Constant) — a port addition, read at the three coverage sites. The hand-drawn Custom curve is deliberately not built (`Falloff`'s doc gives the evidence). No brush shape, no import, no operation override (operation is fixed per feature, `add` or `set`), no rotation, spacing or mirror: coverage is one distance through one falloff |
| **Stroke & grid** | Add point · Duplicate · Rotate · Scale · Tilt · Push · Pull · Align, editing the selected stamp's control points | A stamp stores its `pts` and nothing edits them after the stroke ends. The port adds capture-time grid snap (`sculpt_set_grid_snap`, `sculpt_bridge::GRID_SNAP_STEPS` = 1/2/4/8 cells) — a different feature from control-point editing |
| **Actions** | Flip X · Flip Y · Rot Left · Rot Right · Flatten selection | None. Multi-stamp selection, which these would act on, exists |

`Falloff`'s doc records why the rest is expensive: `SculptStamp::apply_into` is
a per-pixel-independent function of one distance, and an elliptical tip,
spacing/jitter or an airbrush breaks that across all thirteen formulas. The
design's own prototype mocks these blocks (`04-left-dock.md` §5.5–§5.6); the
shell names them in one "Not built" note
(`world_workspace.gd::_build_sculpt_unbuilt_note`), not as working-looking
buttons.

## 11 · Where the code lives

The layers, by symbol; how far each has got is `STATUS.md`'s.

| Layer | Code |
|---|---|
| Sculpt maths — 13 features, 8 presets, the globals, 8 sub-modes, `Falloff` | `cartalith-terrain/src/sculpt.rs`, golden tests `cartalith-terrain/tests/golden_parity_sculpt.rs` |
| Draft / commit / staleness model | `cartalith-spatial/src/pass.rs` (`PassBuffer`), `cartalith-spatial/src/staleness.rs` (`StageGraph`) |
| Commit with river channels and lakes | `cartalith-engine/src/sculpt_commit.rs` (`commit_sculpt_pass`); post-commit tail `cartalith-engine/src/staleness.rs` (`recompute_stale`) |
| GDExtension binding | `cartalith-godot/src/sculpt_bridge.rs` (`SculptEditor`) and the `sculpt_*` `#[func]`s in `lib.rs` |
| DCC panel | `shell/workspaces/world_workspace.gd` (SCULPT mode), `shell/right_dock.gd::_build_sculpt` (stamp stack), `shell/tool_bar.gd` (Commit chip) |
