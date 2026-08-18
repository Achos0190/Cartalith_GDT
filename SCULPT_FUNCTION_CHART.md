# Sculpt — what the HTML does, and how the native version should do it

> Owner's request, 2026-08-18: *"see the sculpt function in the HTML version.
> It's under Generation → Sculpt, chart the functionalities and as stated we
> don't want copies from the javascript version. We want similar functioning in
> the new version."*
>
> Charted from three sources read directly, not from summaries:
> `reference/Cartalith Gen1 v2.10.html` lines 1780–1856 (the panel markup) and
> ~8837–9470 (the behaviour); `cartalith-terrain/src/sculpt.rs` (the
> golden-verified Rust port of that behaviour); and `DCC_SHELL_SPEC.md` §5.2
> (what the new design asks for). Where they disagree, the disagreement is the
> finding and is called out rather than smoothed over.

## The distinction this document runs on

`DECISIONS.md` §7a already draws it: **golden parity binds the maths, not the
markup.** The falloff curve, the noise families, the per-feature amplitude
formulas and the stamp/commit model are the product — those must behave the
same, and they already do, test-enforced. The DOM plumbing that carried them —
integer sliders with scale factors, emoji labels, one scrolling column, an
`enable` gate, a `<details>` accordion — is an artefact of being a single HTML
file. None of it is worth reproducing, and §9 below lists each artefact and what
replaces it.

## 1 · Where the panel lives

| | HTML v2.10 | Native DCC shell |
|---|---|---|
| Reached by | Generate tab → **Sculpt** sub-tab (`#genSculpt`) | WORLD domain on the rail → left dock header switch **GENERATION PIPELINE \| SCULPT** (§5) |
| Shape | One scrolling column, six stacked `.sec` blocks | Three regions: left dock (feature · preset · parameters · brush), tool options bar (the values changed most often, §4), right dock (stamp stack, §6) |
| Enable gate | none — *"being on this tab is the deliberate action… nothing here is destructive until Commit"* | same reasoning, same absence of a gate |
| Locked when | world finalized (`#sculptFinalizedNote`) | Finalize locks stages 01–10 **and** Sculpt (§5.1); the 3D viewport stays available |

The split is the only structural change, and it follows the shell's own rule:
frequently-changed values go horizontal in the tool options bar, structure stays
in the docks. Nothing is dropped.

## 2 · The thirteen features

Ranges and defaults below are the engine's, read out of `sculpt.rs`. The design
spec's §5.2 table matches them exactly — that agreement was the point of the
2026-08-18 design revision, and it is now verified in both directions.

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

Three properties of this table are load-bearing and must survive the port:

- **Radial vs. path.** Lake and Volcano measure distance from the stroke's
  centroid; the other eleven measure signed distance to the polyline, so a
  stroke can meander. Meander itself (`ctx.meander(amp)`) is a sinusoidal
  centreline offset used by River, Canyon and Valley.
- **Per-feature edge character.** `edgeChar` / `edgeFreqMul` domain-warp each
  stamp's *coverage mask*, not its height — so a coastline frays and a mountain
  ridgeline stays tight. One edge treatment for all thirteen would flatten the
  whole library into one look.
- **Registry order is a seed input.** `FEATURE_KEYS`'s index feeds each stamp's
  noise seed (`(seed ^ ((i+1)*1013)) >>> 0`). Reordering the list changes every
  stamp's output, so the UI may re-*group* features visually but must not
  renumber them.

### Freehand's eight sub-modes

Shown only when Freehand is selected (`#sculptModeSeg`).

| Sub-mode | Follows |
|---|---|
| Raise · Lower · Smooth | the drag |
| Cliff · Ridge · Canyon | the drag's **direction** |
| Mesa · Volcano | a single tap (a one-point stroke degenerates to radial distance) |

## 3 · The eight presets

One click seeds a feature's parameters; **it never paints** — the user still
draws the stroke. Verified rather than assumed: each of the eight overrides
exactly one global (`noiseScale`) plus its own feature's parameters.

| Preset | Feature | | Preset | Feature |
|---|---|---|---|---|
| Rolling Hills | Hills | | Volcanic Isle | Volcano |
| Alps | Mountains | | Mesa | Plateau |
| Rockies | Mountains | | Karst | Hills |
| Badlands | Canyon | | Glacial Valley | Valley |

## 4 · The shared brush & noise block

Applies to every feature. Ranges agree between HTML, engine and spec; **five of
the eight defaults do not** — see below.

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

**The five bolded defaults are an open decision.** The engine's column is
`SCULPT_GLOBAL_DEF` from the reference, verbatim and test-pinned. The spec's
column is the design team's chosen starting point. Neither is wrong; they must
not silently diverge. The recommendation is to take the design's defaults, since
a starting brush of 32 cells is small on a 2048² map, and to record the change
so nobody later "fixes" it back — but that is the owner's call, and the code
should read whichever from one place, never from two.

Three noise families back every feature: `sculptFbm`, `sculptRidged`,
`sculptBillow` — all reading the same four globals above. That sharing is what
makes the block coherent rather than eight unrelated dials.

## 5 · The stamp stack

Each finished stroke becomes a live procedural object
(`{type, seed, pts, globals, featureParams, hidden}`) pushed onto a
session-scoped stack. The reference's own comment on that code:
*"nothing here touches `field` or triggers any recompute."*

| Operation | HTML | Native |
|---|---|---|
| List | `#sculptStampList`, newest first, with count tag | Right dock, §6's Stamp-stack context: index, visibility, type, parameter summary |
| Select | click a row | same; selecting re-populates the parameter block for re-tuning |
| Hide / show | `#sculptHideBtn` | same |
| Reorder | Move up / Move down | same — order is bake order, so it is meaningful |
| Delete | `#sculptDeleteBtn` | same |
| Re-tune | edit the selected stamp's parameters, live | same |

## 6 · Undo is two-tier, and stays two-tier

This is the part most likely to be got wrong by rebuilding from the panel
markup alone, so it is stated explicitly.

| Tier | Scope | Records | Cap |
|---|---|---|---|
| Draft undo | the stamp **list** | add · delete · reorder · hide | 30 snapshots |
| Field undo | the **heightfield** | exactly one snapshot, at Commit | the global undo depth |

Continuously dragging a selected stamp's slider does **not** push draft history —
the reference calls this *"a reasonable, common undo granularity"* and it is
right. And `UI_SHELL_DESIGN.md`'s own rule — *"undo granularity is one committed
pass, not one stroke"* — is precisely tier two, verified against real code
rather than invented.

## 7 · Commit and discard

**Discard** drops the draft with a confirmation and touches nothing else.
Identical in both versions.

**Commit** bakes the whole stack in stack order, in one pass. What happens next
is the one place the native version deliberately diverges:

| | HTML v2.10 | Native |
|---|---|---|
| Bake | whole stack, one pass, in order | same |
| River channels | one `enforceRiverChannels()`, plus one `enforceChannelDescent()` per river stamp — carving through rises so the river reaches its outlet, and **locking** those cells so later erosion cannot refill them | same |
| Lake water | one deposit into `lakeMask` | same |
| Flow | one `computeFlow(true)` **immediately** | **deferred** — affected tiles marked stale |
| Climate | one `refreshClimate()` **immediately** | **deferred** — affected tiles marked stale |
| Undo | one `pushUndo()` | same |
| Render | one `renderNow()` | same |

**Why the divergence.** The eager form was measured at **~7 s per stroke at
2048²** and rejected in tool-system milestone C. At the reference's working
resolutions it is affordable; at ours it makes the tool unusable. `PassBuffer`'s
structurally-deferred staleness exists for exactly this, and the status bar
already reports what is stale. This is a `DECISIONS.md` §7a divergence —
principled equivalence, same result on demand, different scheduling — not a
behaviour change.

**`DCC_SHELL_SPEC.md` §5.2 is out of date on this one line.** It still says
Commit *"re-runs erosion, hydrology and climate once."* The engine is right and
the prose is stale; it is flagged in the spec's import header and should be
corrected at the design end rather than the engine end.

## 8 · Water, and what constrains what

There is **no** generic "respect water mask" flag, and adding one would be
inventing a feature. The constraint is per-feature instead:

- **River** and **Lake** are the only features that *write* water state
  (`riverMask` / `riverFloor` / `lakeMask`) on commit.
- Every other feature may freely raise or lower over water. Coastline is defined
  entirely in terms of pulling terrain toward sea level, so gating it on water
  would break it.
- The **categorical paint** tools (biome / terrain / splat) are the ones with a
  hard land-only gate — a different tool family, covered in `STRANDED_TOOLS.md`.

## 9 · What we deliberately do not copy

Each row is a JavaScript- or DOM-specific artefact, not a feature.

| HTML artefact | Why it exists there | Native replacement |
|---|---|---|
| Integer sliders with scale factors — hardness `0–100`, intensity `0–150`, persistence `20–90`, lacunarity `140–320`, edge `0–100` | DOM `<input type=range>` was easier to keep integral | Real float ranges, one source of truth read from the Rust registry — the same reasoning `params.rs` already applies to the 58 generation parameters, so GDScript hardcodes no range, step or label |
| Emoji feature icons (⛰️ 🌋 💧 …) | free glyphs in a single file | §12's thirteen drawn terrain cross-sections, 1.2 px stroke, `currentColor`, already built in `shell/dcc_icons.gd` |
| One scrolling column with a `<details>` accordion | one file, one column | Five-level disclosure across left dock, tool options bar and right dock (§4, §5, §6) |
| `sculptRenderOverlay`'s translucent outline / hatch, explicitly *"a deliberately simpler indicator than a full live-recolor"* | live recolour was too slow in JS | **Open question.** The cost that forced it is a JS cost. A real draft preview is affordable in Rust and is the better tool — but it is a change, so it should be a decision, not a drift |
| `_sculptNavPanLoop` / relocated joystick knob | single-finger drag is captured as a stroke, so panning needed somewhere else to live | Same *problem* on the §13 phone layout, so some equivalent is owed — but it is input routing, not a tool definition, and belongs with the touch work |
| Per-stroke `enforceChannelDescent` scheduling | see §7 | deferred staleness |

## 10 · What the design asks for that neither the HTML nor the engine has

Three blocks in `DCC_SHELL_SPEC.md` §5.2 have no counterpart in v2.10 and no
implementation in `sculpt.rs`. They are new design, not port work, and each is a
real piece of engine effort:

| Spec block | What it asks for | Engine reality |
|---|---|---|
| **Brush shape** | eight built-in shapes (circle, directional, spatter, spiral, dots, cloud, checker, hatch) · Import brush… (greyscale height stamp, alpha respected) · Operation override (subtract / multiply / min / max) · Falloff (smooth / linear / sharp / constant / custom) · Rotation 0–360° · Spacing 0–1 · Mirror | The engine has **one** falloff — `smoothstep` — and **no** brush shape at all. Coverage is distance-to-stroke, full stop. Operation is fixed per feature (`add` or `set`) |
| **Stroke & grid** | Add point · Duplicate · Rotate · Scale · Tilt · Push · Pull · Align, editing the selected stamp's control points | A stamp stores its `pts`, and nothing edits them after the stroke ends |
| **Actions** | Flip X · Flip Y · Rot Left · Rot Right · Flatten selection | none |

None of the three is unreasonable — a brush-shape system in particular is what
would take this from "the reference's sculpt editor" to a DCC-grade one. But
they are additions, and they should be scoped and scheduled as additions rather
than discovered mid-implementation. Until they are, those rows ship disabled
with a tooltip naming what is missing, per the shell's standing honesty rule.

## 11 · State of the port

| Layer | State |
|---|---|
| Sculpt maths, all 13 features, 8 presets, 8 globals, 8 sub-modes | **Built and golden-verified** — `cartalith-terrain/src/sculpt.rs` |
| `PassBuffer` draft/commit/staleness model | **Built** — `cartalith-spatial/src/pass.rs` |
| Commit with river channels, lakes, deferred flow/climate | **Built** — `cartalith-engine/src/sculpt_commit.rs` |
| GDExtension binding | **In progress** — this is `UNIFIED_TOOL_PLAN.md` milestone F and the only thing between the engine and the panel |
| DCC panel | blocked on the binding |
| Brush shape · Stroke & grid · Actions (§10) | **not started, not scoped** |
