# Unified tool plan: what a tool *is*

> **This document defines what a tool is, what each one needs from the engine,
> and the A-F milestones that follow from that. It does not track them.** Which
> milestones have run lives in **`cartalith-native/docs/STATUS.md`**, the only
> place progress is recorded. The dated "as built" sections below are the record
> of the passes that ran, and the tool-by-tool audit is a 2026-08-18 reading of
> the reference and of this port — read either for *what was found and decided*,
> never for where the work stands.

`UI_SHELL_DESIGN.md`'s own line names this document directly: *"`docs/
UNIFIED_TOOL_PLAN.md` decides what a tool *is*; this document decides where
it appears."* The design team's `github.md` places it at `docs/
UNIFIED_TOOL_PLAN.md`, but this repo keeps scope/decision documents at the
root rather than under a `docs/` folder (`cartalith-native/docs/` is reserved
for the living `CHANGELOG.md`/`STATUS.md`) — the same path discrepancy
`UI_SHELL_DESIGN.md` itself already flagged and resolved for its own file.
This document resolves it the identical way: it lives at the repo root, as
`UNIFIED_TOOL_PLAN.md`.

Written per `DCC_SHELL_SCOPE.md` milestone 2 — investigation and scoping
only, no code. It answers, tool by tool, for the left rail's fifteen tools
(`UI_SHELL_DESIGN.md` "Left tool rail"): what the tool does, whether its
underlying data operation already exists in this Rust port, what its real
parameter set is, and what its pass-buffer/staleness behaviour is. It closes
with a milestone breakdown for Track 2 (`DCC_SHELL_SCOPE.md`'s "the tool
system itself").

## Headline findings, before the detail

1. **The reference has a real, shipped Sculpt editor** (`SCULPT_FEATURES`,
   `sculptStamps[]`, `sculptCommit`/`sculptDiscard`, reference lines
   8837-9470-ish) that already solves brush falloff, noise-modulated
   strokes, a draft/commit/discard model, and undo granularity — for the
   **Terrain** tool group specifically. This is real prior art to port, not
   a brush model to invent.
2. **The reference's own draft/commit/discard model is the direct ancestor
   of `UI_SHELL_DESIGN.md`'s pass buffer** — `sculptStamps[]` (an
   uncommitted, session-scoped list of pending stamp objects) is visible
   immediately (redrawn every stroke via `sculptRenderOverlay`), commits by
   baking the whole stack into `field` in one pass plus **one**
   `computeFlow(true)`/`refreshClimate()` (not a recompute per stroke), and
   discards by dropping the list untouched. `UNIFIED_TOOL_PLAN`'s pass
   buffer is this same idea, generalized past terrain.
3. **`cartalith-spatial`'s `DirtyTracker` is necessary but not sufficient.**
   It gives per-tile dirty flag + monotonic version + a reason string — real
   staleness bookkeeping. It holds **no data**: no uncommitted-edit storage,
   no preview compositing, no discard. A real pass buffer needs a new type
   built alongside it (see "The shared editing model" below), the same way
   the reference needed `sculptStamps[]` *in addition to* its plain
   field-level `undoStack`.
4. **Every terrain tool group (2) has real backing.** Water & ecology (3) is
   split: River/water has real backing (the same Sculpt editor), Biome paint
   has real backing of a *different kind* — an override layer
   (`paintBiome`), not a mutation of the classifier. Civilization (4) is a
   mix: Place settlement and Draw route/way have real reference precedent as
   *manual* tools (verified, not assumed — see below), separate from this
   port's own algorithmic settlement placement and road network; Territory/
   faction has real reference precedent (`_civPaintTerritoryAt`) but the
   port's own `assign_territory` (algorithmic, reference never had this) is
   what it would actually paint over. Annotation & measure (5) is the most
   mixed group: Label and Region select/export have real, fairly rich
   reference precedent; Icon stamp has *partial* backing (rule-driven
   autoplacement exists, manual single-icon placement does not); Measure has
   **zero** reference precedent — genuinely new, and trivial.
5. **Full-pipeline recompute after every edit is not interactively viable.**
   Real measured timing (`CPU_MULTITHREADING_SCOPE.md`): `cartalith-terrain`
   alone at 2048×2048 is ~5.1s parallelized; terrain + the civ per-cell
   layer alone is ~7.07s parallelized, *before* climate/erosion/hydrology or
   the sequential civ stages (settlement placement, naming, roads,
   territory) are added in. The mockup's own "rivers · deferred" status-bar
   text is not a stylistic choice, it is the only viable design — staleness
   must stay deferred/lazy, exactly as `sculptCommit` itself only recomputes
   flow and climate **once per commit**, never mid-stroke, never eagerly
   cascading through settlements/roads/territory.

## The reference's Sculpt editor, read for real

`MVP_SCOPE.md`'s "Out of scope" table lists "Sculpt editor | block 1" as its
only mention anywhere in this project. Read directly (reference lines
~8837-9470), it turns out to be a substantial, late-stage (v1.15+), well-
engineered feature — not a stub.

**What it lets a user do.** Thirteen registered "features"
(`SCULPT_FEATURES`, reference line 8891), each a named landform stamp with
its own control set, applied along a captured pointer stroke (or a single
tap for radial features): Mountains, Hills, Ridge, Plateau, Cliff/
Escarpment, Canyon, Valley, River, Lake, Basin, Coastline, Volcano, and
Freehand (a catch-all with eight sub-modes: raise, lower, smooth, cliff,
ridge, canyon, mesa, volcano). Eight one-click presets (`SCULPT_PRESETS`,
e.g. "Alps", "Badlands", "Volcanic Isle") seed a feature's parameters before
the user paints the stroke. This is raise/lower at minimum, plus a full
landform-stamp library — directly answering the investigation brief's
question.

**The brush model, concretely:**
- **Falloff**: `smoothstep(0, 1, (R - dist) / feather)` from the stroke
  polyline (or the radial centre for radial features), where
  `feather = max(featherFloor, R * (1 - hardness))` — `hardness` is a 0..1
  global (`SCULPT_GLOBAL_DEF.hardness = 0.5`) that narrows the falloff band
  as it rises toward 1 (harder edge), the exact "gauss-like smooth falloff"
  the mockup's Properties panel names (`falloff: smooth (gauss)`).
- **Intensity** scales the per-pixel coverage weight before it's applied
  (`k = cov * P.intensity`), separate from hardness — coverage shape versus
  effect strength are independently tunable, matching the mockup's separate
  `hardness`/`intensity` sliders exactly.
- **Noise-modulated strokes, yes**: three noise families backing every
  feature's `apply()` — `sculptFbm`/`sculptRidged`/`sculptBillow`, each an
  octave/persistence/lacunarity FBM variant reading `noiseScale`/`octaves`/
  `persistence`/`lacunarity` off the same global param object the mockup's
  Properties panel shows (`noise scale`, `octaves · persistence`). A
  separate **domain-warp edge noise** (`edgeNoise`, `edgeChar`/`edgeFreqMul`
  per feature) ragged-warps each stamp's *coverage mask* itself — coastlines
  and lakes get a low-frequency ragged edge, mountain ridgelines a tight
  high-frequency one, matching each landform's real character rather than
  one edge treatment for all thirteen.
- **Radial vs. path-based**: `feat.radial` (true for Lake, Volcano) samples
  distance from a stroke-averaged centre; everything else samples signed
  distance-to-polyline (`sculptNearestOnStroke`) so strokes can meander
  (`ctx.meander(amp)`, a sinusoidal centerline offset used by River/Canyon/
  Valley).

**Draft/commit/discard, already real — this is the pass buffer's ancestor.**
Painting never touches `field`. Each finished stroke becomes a stamp object
(`{type, seed, pts, g, f, hidden}`) pushed onto `sculptStamps[]`, a session-
scoped array (comment, reference line 9089: *"nothing here touches `field`
or triggers any recompute"*). `sculptRenderOverlay` redraws every stamp's
footprint (translucent outline/hatch, not full live-recolour — the file's
own comment explicitly calls this "a deliberately simpler indicator than a
full live-recolor... the real height/material colouring only appears after
Commit"). `sculptCommit()` (line 9317) bakes the **whole stack**, in stack
order, into `field` in one pass, then runs exactly **one**
`enforceRiverChannels()`, one river-specific `enforceChannelDescent()` pass
per river stamp (carving through any rises so the river reaches its
outlet and *locking* the carved cells so later erosion can't refill them —
same precedent `carveRiverValleys()` already uses), one lake→`lakeMask`
deposit, one `computeFlow(true)`, one `refreshClimate()`, one `pushUndo()`,
one `renderNow()`. `sculptDiscard()` (line 9353) drops `sculptStamps` with a
confirm dialog and touches nothing else.

**Undo/redo is two-tier, not one.** A *draft-scoped* undo/redo
(`sculptHistory`/`sculptRedoStack`, JSON snapshots of `sculptStamps`, capped
at 30) tracks structural edits to the stamp list (add/delete/reorder/hide)
while still drafting — continuous slider re-tuning of a selected stamp does
**not** push history, the file's own comment calling this "a reasonable,
common undo granularity." Separately, the field-level `undoStack` only ever
records **one** snapshot at Commit (`pushUndo()` inside `sculptCommit`) —
never per stroke. `UI_SHELL_DESIGN.md`'s own rule — *"Undo granularity is
one committed pass, not one stroke"* — is exactly this, verified against
real code, not invented.

**Water mask / constraints**: no separate "respect water mask" flag exists
for the generic terrain features — the constraint is feature-specific
instead. River and Lake are the two features that *write* water state
(`riverMask`/`riverFloor`/`lakeMask`) on commit; every other feature can
freely raise/lower over water (a Coastline stamp, for instance, is defined
entirely in terms of pulling terrain toward `seaLevel`). The categorical
paint tools (`_paintAt`, biome/terrain/splat painting — see "Biome paint"
below) are the ones with a hard water gate, land-only by construction
(`if(wb[i]!==0) continue`).

**Camera/pan interaction while painting** is also solved for touch (a
relocated joystick, `_sculptNavPanLoop`/`_sculptNavSetKnob`) since a
single-finger drag is captured as a stroke — worth remembering when this
port's own touch/Android target (`UI_SHELL_DESIGN.md`'s phone layout) gets
here, but out of scope for this document's tool-by-tool plan; it's an input-
routing detail, not a tool definition.

## The shared editing model: pass buffer, commit, discard, staleness

Every tool in every group shares one mechanism (`UI_SHELL_DESIGN.md`
"Editing model"). This section specifies it once, concretely, grounded in
both the reference's real precedent above and `cartalith-spatial`'s real
current code.

### What `DirtyTracker` provides today

Read directly (`cartalith-native/crates/cartalith-spatial/src/lib.rs`,
lines 500-560): `DirtyTracker` is a flat `Vec<TileStatus>`, one entry per
tile, each holding `{dirty: bool, reason: Option<String>, version: u64}`.
`mark_dirty` sets the flag, records a reason, and — the one piece of real
semantics — **only `mark_dirty` bumps `version`**; `clear_dirty` resets the
flag without bumping it, because clearing isn't itself a change to the
tile's data. `dirty_tiles()` iterates the dirty set. That's the whole
surface. Its own doc comment is explicit about scope: *"No Cartalith-
specific field-dependency semantics... this stays a generic caller-supplied
reason string rather than a set of Cartalith field names baked into a
library crate."* It was built standalone, unintegrated, "for whenever... a
real large-world need actually triggers... integration" — this is that
trigger, but it answers only the bookkeeping half of the problem.

**What it does not provide, and a real pass buffer needs:** any storage for
an *uncommitted* edit. `TiledField<T>` holds exactly one array — the live,
committed data (`whole()`/`whole_mut()`/`region_mut()` all read/write that
one array directly). There is no scratch copy, no "this tile has a pending
edit visible in the viewport but not yet in the field" state, and therefore
no discard: discarding today would mean "don't call `mark_dirty`," but
nothing was ever written anywhere to discard, because nothing exists to
hold a draft.

### The new type this needs — modelled directly on `sculptStamps[]`

Not a `DirtyTracker` extension; a new, small type that *uses* `DirtyTracker`
and `TiledField` rather than replacing either:

```
struct PassBuffer<Stamp> {
    stamps: Vec<Stamp>,          // sculptStamps[] equivalent — the append-only
                                  // draft, one entry per finished stroke/action
    touched_tiles: HashSet<usize>,  // union of every stamp's affected tile set,
                                  // maintained incrementally as stamps are pushed
}
```

- **Preview** ("visible immediately"): re-render `touched_tiles` each frame
  by compositing `stamps` over a *read* of the committed `TiledField` data —
  never write the field. This is exactly `sculptApplyStamp`'s own documented
  design (reference line 9033's comment: *"writes directly into caller-
  supplied H/W arrays (never `field`/module globals) so both the draft
  preview (a scratch buffer) and commit (field itself) reuse the identical
  code path"*) — one apply function, two destinations, chosen by the caller.
  Godot-side this is a scratch texture region reflecting only
  `touched_tiles`, not a full-map re-render.
- **Commit**: apply every stamp in `stamps`, in order, into the real
  `TiledField`'s data (the same function, field-as-destination), then
  `DirtyTracker::mark_dirty` every tile in `touched_tiles` with a reason
  (`"raise_lower_committed"`, etc.) — one version bump per tile, matching
  "one committed pass" semantics regardless of how many strokes it took.
  Clear `stamps`/`touched_tiles`.
- **Discard**: clear `stamps`/`touched_tiles`. Nothing was ever written to
  `TiledField`, so nothing needs undoing there — a strict improvement over
  the reference's own field-level approach, which never needed a per-stroke
  undo precisely because of this same non-destructive-until-commit design.
- **Undo granularity**: one `TiledField` snapshot (or a tile-diff) taken at
  commit time, not per stroke — same two-tier model as the reference
  (`sculptHistory` for draft-scoped structural edits vs. one `pushUndo()` at
  commit). The draft-scoped undo/redo (reordering/deleting an uncommitted
  stamp before committing) is a real, separate small stack of `Vec<Stamp>`
  snapshots, same shape as `sculptSnapshot()`/`sculptPushHistory()`.

This is genuinely new code (`cartalith-spatial` doesn't have it, the
reference's version is untyped JS objects), but it is a *small*, well-
specified type once written down this way — not an open design question.

### Staleness: what actually needs to re-run, and why it must stay deferred

The causal chain (`VISION.md`'s own table, confirmed against real crates):
`cartalith-terrain` (height) → `cartalith-hydrology` (flow, rivers) →
`cartalith-climate` → `cartalith-civ` (biome classification, soil fertility/
NPP/carrying capacity, settlement suitability, roads, territory). Real
measured cost at 2048×2048 (`CPU_MULTITHREADING_SCOPE.md`'s benchmark table,
Rayon-parallelized): `cartalith-terrain` alone ~5.1s; adding the civ per-
cell layer (`build_resource_potentials`'s 15 fields,
`build_settlement_suitability`) brings terrain+civ to ~7.07s — and that
total explicitly **excludes** climate/erosion/hydrology and civ's own
sequential stages (settlement placement, naming, road Dijkstra, territory's
capital loop), which are additional real time on top. A brush stroke firing
that whole chain per frame, or even per stroke, is not viable at any
resolution this engine targets.

Concretely, per terrain-group tool: raising/lowering terrain invalidates
flow accumulation and river network (`cartalith-hydrology`) directly, which
invalidates biome classification and settlement suitability
(`cartalith-civ`) transitively, which invalidates roads/territory
transitively again. The mockup's own status line — *"downstream update:
rivers · deferred"* — names exactly the first of these and marks it
deferred, not recomputed; this is the right call, not an artifact of the
mockup being unfinished. The concrete rule this plan adopts:

- **On commit**, mark the affected tiles dirty in the height `DirtyTracker`
  with a reason identifying the causal edge crossed (`"height_edited"`).
  Do **not** cascade the mark automatically through hydrology/climate/civ at
  commit time — that would require knowing the full downstream tile
  footprint of a flow-accumulation change, itself an expensive query.
- **Downstream stages carry their own `DirtyTracker`**, each checking its
  own upstream's version counter lazily, the same pattern
  `DirtyTracker::version` already supports (a stage compares the version it
  last computed against against the upstream tracker's current version for
  the tiles it depends on). This is deferred/lazy by construction — a stage
  only recomputes when something actually asks for its current value
  (opening the Hydrology parameter dialog reports staleness per `UI_SHELL_
  DESIGN.md`'s Generate menu row; switching to a workspace tab that displays
  biome color needs current biome; committing a save does not need every
  stage current, only whichever ones the save format requires).
  `cartalith-spatial`'s existing single-reason-string `DirtyTracker` is
  actually well-suited to this — the "no baked-in Cartalith field names"
  design the crate's own comment defends turns out to be right, since each
  stage owns its own tracker instance rather than one shared field-name
  enum.
- **A stage recomputes only its own dirty tiles plus their required
  neighbourhood**, not the whole map — this is real future work
  (`cartalith-hydrology`/`cartalith-civ` are not currently tile-incremental;
  they operate on the whole field). Milestone 3+ below scopes this
  explicitly rather than assuming it falls out for free.

## Tool-by-tool

*Each tool below carries an **Engine backing** finding — what this port had, or
did not have, for it. Those findings are a **2026-08-18 reading** of the
workspace, and they are what the A-F milestones were sized from; they are not a
description of the crates as they stand now. Milestones A-E were written to
close exactly the gaps this section names.*

### Group 1 — Navigate & inspect

**Select/inspect (`V`)** and **Pan (`H`)** are pure viewport-navigation
tools with no field mutation and no engine backing question — they are
Godot `Camera2D`/input-routing work, already how the existing map
navigation behaves today. No pass buffer, no staleness. Not scoped further
here; they belong to Track 1 (shell restructure), not Track 2.

**Point sample (`I`)** has real, strong backing: `VISION.md`'s "WHY HERE?"
causal-chain Inspector (`STATUS.md`, done 2026-08-17) already reads real
computed values at a settlement under the cursor — height, flow, biome,
suitability, the works, sourced from `cartalith-terrain`/`cartalith-
hydrology`/`cartalith-civ` directly. The reference's own `_civInfoAt`
(line 20436) is the same idea generalized to *any* point, not just a
settlement. **Real parameter set**: none to speak of — it's a read, not an
edit. **Pass buffer**: none — a query has no draft to commit or discard.
**Engine backing**: exists (the Inspector's data sources are all real,
golden-verified functions); what's missing is only wiring the existing
Inspector to fire at an arbitrary clicked point instead of only a
settlement hover, a small Godot-side change once the shell restructure
lands the tool rail for real.

### Group 2 — Terrain

All four tools share one engine primitive: the Sculpt editor's stamp
registry and stamp/pass-buffer model, described in full above. What differs
per tool is which `SCULPT_FEATURES` entries (or entry) it exposes and how
its options bar is laid out.

**Raise/lower (`B`)**. Maps to `SCULPT_FEATURES.freehand` with `subMode`
fixed to `raise`/`lower` (a per-pixel add of `±amount`, coverage-weighted).
**Real parameter set**, grounded in the mockup's own Properties panel and
tool options bar (`design/Cartalith DCC Shell.dc.html` lines 59-140):
`hardness` (0..1, mockup shows 0.35), `intensity` (mockup shows "+120 m" —
i.e. this is where the reference's abstract 0..1 `intensity` gets a real-
world unit via `mapWidthKm`/height-range conversion, a Godot-side
presentation concern, not an engine one), `falloff` (fixed "smooth (gauss)"
— the smoothstep shape above, not user-selectable in the mockup),
`noise scale` (mockup: "0.8 km" — same unit-conversion point),
`octaves`/`persistence` (mockup: "4 · 0.5", direct read of
`SCULPT_GLOBAL_DEF.octaves`/`.persistence`), a raise/lower/smooth mode
selector (tool options bar), and a `respect water mask` toggle (mockup:
"on"). That last one is the one field with **no direct reference
equivalent** — the reference's Freehand raise/lower has no water gate at
all (see "Water mask / constraints" above); this port would need to add it
as a real new gate (skip cells where a water-body classification is
nonzero), a small, well-understood addition, not a redesign. The mockup's
`affect layer: bedrock + sediment` field has **no engine backing at all** —
this port's height field is a single `f32` array with no bedrock/sediment
split anywhere in `cartalith-terrain`/`cartalith-erosion` (confirmed by
grep — `sediment`/`bedrock` appear only in erosion's internal deposition-
transport comments, never as a separate persisted layer). Treat this
mockup field as aspirational, not a milestone commitment, unless a real
two-layer height model becomes a requirement elsewhere. **Pass buffer**:
per the shared model above. **Staleness**: height edited → hydrology/
climate/biome/settlements/roads/territory all deferred-stale.

**Smooth (`S`)**. Maps to `SCULPT_FEATURES.freehand`'s `smooth` sub-mode —
the one feature that bypasses the generic per-pixel `apply()` path entirely
and instead runs a dedicated 4-neighbour blur over a **stable pre-loop
snapshot** of the field (reference line 9041's comment: *"the generic per-
pixel-independent apply() path can't read stable neighbour state mid-scan,
so this is the one feature that bypasses feat.apply()"*). This is a real,
specific implementation detail worth preserving exactly — a naive port that
reads/writes the live buffer mid-scan would produce direction-dependent
smoothing artifacts the reference deliberately avoids. **Parameters**:
`amount` (0.02-0.3, reference default 0.12), `hardness` (shared global).
**Engine backing**: same as Raise/lower — needs the pass-buffer preview
path but the blur math itself is a direct, small port.

**Flatten/terrace (`F`)**. Maps to `SCULPT_FEATURES.plateau` — "Terraced
FBM mesa. Sets a flat top; terraces quantize the surface. **Never lowers
existing terrain**" (reference's own hint string, line 8917) — this is
exactly "flatten/terrace" as a concept, not an approximation of it.
Parameters: `plateauHeight`/`Rise` (0.03-0.45, default 0.26), `terraces`
(1-8 steps, default 4), `plateauFreq`/detail frequency (0.4-3, default
1.1). The monotonic "never lowers" behaviour comes from `c.mode='set';
c.val=Math.max(c.h0, level)` — a `set`-to-max, not an `add` — worth
preserving as the defining trait of this tool versus Raise/lower.

**Stamp (landform library)**. Maps to the *rest* of `SCULPT_FEATURES`
(Mountains, Hills, Ridge, Cliff, Canyon, Valley, Coastline, Volcano, Basin —
nine entries once River/Lake are pulled into the Water & ecology group
below) plus the eight `SCULPT_PRESETS`. This is the tool rail's most
direct, richest reference port: a picker (landform type × preset) feeding
the same stamp/pass-buffer pipeline every other terrain tool uses, differing
only in which `apply()` body and control set is active. **Engine backing**:
real and complete for all nine remaining entries — every `apply()` body is
a small, pure, portable function (`sculptApplyStamp`'s per-pixel loop, the
per-feature closures in `SCULPT_FEATURES`) with no DOM/JS-runtime
dependency; this is a large but mechanical port, not a design problem.

### Group 3 — Water & ecology

**River/water (`R`)**. Maps to `SCULPT_FEATURES.river` and `.lake`, whose
commit behaviour is *not* a generic field bake — it is genuinely special-
cased in `sculptCommit()`: River stamps additionally run
`enforceChannelDescent` (carve-through + lock, reusing the same precedent
`carveRiverValleys()` established) and write `riverMask`/`riverFloor`; Lake
stamps run a **`waterOnly` dry-run pass** *after* the main bake (so the
already-final, post-bake height is what gets tested against the lake's
water surface, avoiding double-carving the bowl — reference line 9074's
comment is explicit about why this ordering matters) and deposit into
`lakeMask`, the same array `depositWater()`/`buildWaterBodies`'s
`forceLake` path already reads. **Parameters**: River — `riverWidth`
(2-26px, default 7), `riverDepth` (0.02-0.22, default 0.09),
`riverMeander` (0-0.6, default 0.28), `branchNoise` (0-1, default 0.5).
Lake — `lakeDepth` (0.03-0.3, default 0.13), `lakeShore` (0.05-0.6, default
0.25). **Engine backing**: the underlying `field`/`riverMask`/`lakeMask`/
`enforceChannelDescent`/`computeFlow` primitives all have real ported
Rust equivalents in `cartalith-hydrology`/`cartalith-erosion` (this port's
own hydrology pipeline already produces and consumes exactly these
structures for the algorithmic river network) — what's new is only the
manual-stamp-into-the-same-structures path, not the structures themselves.
**Staleness**: the one tool the mockup explicitly names as deferred
downstream ("rivers · deferred") — because a river edit's own commit
already runs `computeFlow`/`refreshClimate` once, but does **not** cascade
into biome/settlement-suitability/roads/territory, which stay stale exactly
like a terrain edit's does.

**Biome paint (`P`)**. This is the one tool in this group with a materially
different data shape from the Sculpt editor's stamp model, and the
investigation brief's own hypothesis about it (mutating a generated result
being different from the pure function that produces it) is confirmed by
reading the reference directly. `classify_biome` (`cartalith-civ/src/
lib.rs:558`, golden-verified) is a pure `(temperature, moisture) -> biome`
function with no notion of a paintable override — porting it doesn't give
you a paintable layer, because there is nothing in its signature to hold a
manual edit. The reference's real answer is a **separate override array**:
`paintBiome` (reference line 4764, lazily-allocated `Uint8Array(GW*GH)`,
distinct from any classifier output), written by `_paintAt` (line 4783, a
hard-edged circular disc — "categorical data has no half-painted state, so
unlike sculpt()/brushHeight there's no soft falloff here" — reference's own
comment), and merged at read time with `mb[i] = paintBiome[i]` wherever
`paintBiome[i]` is nonzero (line 4779, `_carRefreshIconAndPaintPickers`'s
sibling bake code) — i.e. **the painted layer takes precedence over the
computed classification, cell by cell, only where painted**. This is the
real, grounded design for this port too: a `Vec<u8>` (or `Option<u8>` per
cell) override array, same shape as `classify_biome`'s output, checked
first at every render/query site that currently calls `classify_biome`
directly. Confirmed further by the reference's own invalidation rule
(`paintBiome=null` on terrain rebuild, line 3353's comment: *"hand-painted
Cartography overrides don't survive a terrain rebuild"*) — painted overrides
are downstream of terrain generation and get cleared by a fresh `generate()`,
though not necessarily by every individual terrain-tool commit (this needs
a real decision at implementation time: clear on full regenerate only, or
also on any height edit that changes the underlying temperature/moisture
inputs at that cell — the reference only ever had one "generate," this port
now has incremental terrain edits it didn't).

**Parameters**: brush radius (`_paintRadius`, reference default 6 cells),
a value picker populated from `CART_BIOMES` (13 land biomes, water excluded
— "the brush never touches water"), an erase toggle. Land-only gate
(`wb[i]!==0` excludes both ocean and lake) is hard, not a "respect water
mask" option — worth keeping as a hard constraint rather than a toggle,
matching the reference exactly. **Engine backing**: needs extension — the
override-array type and its merge-at-read-site integration are genuinely
new Rust code, though small and precisely specified by the reference.
**Pass buffer**: same shared model, but the "stamp" content here is disc-
paint deltas to the override array rather than a height-field stamp — the
`PassBuffer<Stamp>` type above is generic enough to hold either.
**Staleness**: painting biome does *not* mark height/hydrology/climate
dirty (it's downstream, read-only of those); it does mark anything reading
biome downstream (settlement suitability partially depends on soil/
biome-adjacent fields — check `build_settlement_suitability`'s actual input
list before wiring this specific edge, not assumed here).

### Group 4 — Civilization

**Place settlement**. Real reference precedent: `_civDropPlace` (line
16051) — click near an existing place selects/inspects it instead
(weighted-nearest pick, `_civPlacePickWeight`/`_civPlacePickVisible`,
prominence-scaled so a small close pin doesn't out-compete a bigger distant
one); otherwise, gated on land (`field[i] >= sea` and water-body
classification zero), it pushes a new `{x,y,name:'',kind:'town',
faction:_civActiveFaction,pop:1000,traits:[]}` onto `state.places` and opens
an editor for name/kind/faction/population/traits/history. **Engine
backing**: this port's `SettlementPlacement`/`NamedSettlement` structs
(`cartalith-civ/src/lib.rs:2411`/`3089`) already carry exactly this shape
(`x, y, suit, faction, capital, kind, coastal` / `+name, pop`) — what's
missing is not a new data model, it's the *manual insertion path*: a
function that appends a caller-placed settlement into the same list
`place_settlements` (the algorithmic placer) produces, so every downstream
consumer (naming, roads, territory) treats a hand-placed settlement
identically to a generated one. This is a genuinely new but small
Rust-side function once framed this way — not a new subsystem.
**Parameters**: kind (settlement tier), faction (from the same "active
faction" quick-select the mockup's Territory tool would also use — these
two tools should share state), initial population (reference default
1000, pre-`_civBasePopForKind`/naming pass — a hand-placed settlement could
instead run through `civ_settle_name`/`civ_base_pop_for_kind` immediately
for a properly-named, tier-populated result rather than the reference's
raw placeholder). **Pass buffer**: arguably unnecessary — placing a
settlement is a discrete, already-atomic action (append one struct), not a
brush stroke needing preview-before-commit; a "pass" here could simply be
zero-or-more placements before an explicit commit, mirroring the group's
shared UI language even though the underlying operation doesn't strictly
need staging. **Staleness**: marks settlement-suitability-adjacent and
road/territory stages dirty (a new settlement changes both).

**Draw route/way**. The scope doc's own hypothesis — *"manual tools, since
this port's own road/territory generation is algorithmic, not hand-drawn"*
— is **confirmed for this tool specifically**, and more precisely than
assumed. There are *two* reference route-drawing systems, easy to conflate:
`_civOpenRouteEditor` (line 20406) is the **Journey Planner's** route
editor (logistics planning over an *existing* route/journey object,
`civJourneys[]`) — not this tool. The real match is `_civTool==='draw_way'`
(reference lines 26071-26107): the user clicks waypoints
(`_civWayWaypoints.push(...)`, snapped to nearby settlements/POIs/other ways
via `_civSnapPoint`/`_civFindSnapTarget`, v1.52's real snap-to-target
feature), and on commit (`_civCommitWay`, Escape key), consecutive
waypoints are joined by **the same terrain-cost Dijkstra pathfinding the
algorithmic road network uses** (`_civJoinDijkstraSegs` →
`_civDijkstraPath`, land or water mode by way type), producing a real path
through real terrain cost between user-chosen anchor points — not a raw
polyline. The result is pushed to `civWays[]` tagged `manual:true`,
distinct from the algorithmic `civRoads`. **Real engine backing**: the
pathing primitive is not new — `road_dijkstra` (`cartalith-civ/src/
lib.rs:3269`) is exactly this same Dijkstra-over-terrain-cost, already
ported and golden-verified for `build_road_network`. What's genuinely new
is: (a) a waypoint-collection interaction (click, snap-to-target, commit on
Escape/tool-switch), (b) a `ManualWay` struct (`pts, km, type, sea: bool,
manual: true`) distinct from `RoadEdge`, and (c) the unreachable-leg
fallback-to-straight-line-and-warn behaviour the reference added in v1.99
after finding the naive Dijkstra fallback could silently cross terrain a
given way type should avoid. **Parameters**: way type (road/sea-lane/other,
driving land-vs-water Dijkstra mode). **Pass buffer**: the whole in-
progress waypoint chain is itself the natural pass-buffer unit — "commit"
already means exactly what `_civCommitWay` does. **Staleness**: a manual
way affects trade-corridor/route-corridor-adjacent civ outputs
(`build_route_corridors`) but not terrain/hydrology/climate at all — a
narrower staleness footprint than any terrain-group tool.

**Territory/faction**. Real reference precedent: `_civPaintTerritoryAt`
(line 15964) — a disc brush writing `_civActiveFaction` directly into
`civTerritory[y*GW+x]` for every cell within radius `_civTerRadius`, no
falloff (categorical, like Biome paint), no land/water gate visible in the
function itself. This is confirmed as the reference's **only** way
territory is ever set — `PHASE2_SCOPE.md`'s own milestone-9 investigation
(already on record in this repo, re-checked here rather than re-derived)
found `getCivTerritory()` only lazily zero-allocates the array and never
computes ownership; the sole writers are this paint function and a save/
load deserializer. But this port does **not** have that gap — `DECISIONS.md`
§7b's `assign_territory` (`cartalith-civ/src/lib.rs:4009`) is a real,
already-shipped algorithmic territory generator (cost-distance Voronoi from
capitals, population-weighted), built for a different original reason
(the reference had no algorithmic path at all) but producing the identical
per-cell shape (`Vec<i32>` faction id, 0 = unowned) `civTerritory` itself
used. So the real design here is: **paint as an override on top of an
already-computed `assign_territory` result**, the same override-layer
pattern as Biome paint, not a from-scratch paintable field the way the
reference's version necessarily was (it had nothing to override). **Engine
backing**: exists for the base layer (`assign_territory`), needs a small
override/merge mechanism for the paint itself — smaller than Biome paint's
equivalent work since the base algorithm and its output shape are already
real and tested. **Parameters**: active faction (shared quick-select with
Place settlement), brush radius. **Pass buffer**: disc-paint deltas, same
generic stamp shape as Biome paint. **Staleness**: territory is a leaf of
the causal chain in this port (nothing currently reads it further
downstream except province sub-partitioning, `_civGenerateProvinces`'s
Rust equivalent, itself not yet wired) — painting territory has the
narrowest staleness footprint of any tool in this group.

### Group 5 — Annotation & measure

**Label (`T`)**. Real, fairly rich reference precedent:
`_civSelectLabel`/`_civConfirmLabel`/`_civCancelLabel` (lines 15356-15367)
plus `drawArcLabel`/`_civSectorLabel` (lines 15244/16509) — placed,
**curved/arc** map labels (name text following a great-circle-like arc
across a region, the atlas-cartography convention for a region/country
name), with editable `name`, `angle`, `arc` (curvature), `size`, `font`,
`color`, `sizeMode` (zoom-relative vs. fixed). Selecting a label snapshots
its editable fields once per edit session so Cancel (`✗`) reverts cleanly
to session-start state, while dragging to reposition commits immediately —
position and styling have deliberately different commit semantics in the
reference, worth preserving. **Engine backing**: **none** — no label
placement/storage/rendering exists anywhere in this Rust port today
(confirmed by grep; the only `label` hits in `cartalith-civ` are unrelated
struct fields on logistics types like `AnimalStats`/`LoadPenalty`). This is
genuinely new interaction *and* new data model — a `Label{x,y,name,angle,
arc,size,font,color,sizeMode}` struct with no Rust-side precedent, though
the reference gives a complete, concrete spec to port rather than invent.
**Pass buffer**: arguably unnecessary for the same reason as Place
settlement — placing/editing one label is a discrete action with its own
confirm/cancel, not a brush stroke.

**Icon stamp**. Partial backing, and the two halves are genuinely
different operations, same distinction the investigation brief drew for
Biome paint. This port already has real, recently-shipped **rule-driven**
icon *auto*-placement (Phase 4 milestone 4, per this repo's own commit
history): `place_map_icons_ruled` (`cartalith-assets/src/placement.rs:201`)
and the scatter-rule system (`cartalith-assets/src/scatter.rs` —
`preset_scatter_rule`, `normalize_scatter_rule`, `pick_icon_variant`,
`autopopulate_scatter_rules`), composited via `composite_map_icons`
(`cartalith-godot/src/pack.rs:388`). The reference's own equivalent —
`_carIconBrushRule`/`_carIconBrushStamp` (lines 15046/15051) — likewise
splits into a **rule** mode (auto-populate by scatter rule) and a **stamp**
mode (place one icon by hand at a clicked point). The rule half is real and
ported; the **manual single-icon-by-click half is not** — same shape gap as
Biome paint (a pure/generated result vs. a hand-authored override), except
here the "override" is additive (drop one more icon) rather than a per-cell
replace. **Real parameter set**: icon family/slot (from the existing
`icon_slot_for_item`/asset-pack icon taxonomy already in `cartalith-
assets`), variant (`pick_icon_variant`'s existing seeded-variant logic could
directly serve a manually-placed icon too, for visual consistency with
auto-placed ones of the same slot). **Engine backing**: needs extension —
a `ManualIcon{x,y,slot,variant}` list, stored and composited alongside (not
instead of) the rule-driven set. **Pass buffer**: same "discrete action,
staging optional" character as Label/Place settlement.

**Measure (`M`)**. **Zero reference precedent** — grepped broadly
(`ruler`, `measureDist`, `distanceTool`) and found nothing; the only
related reference function is `updateScaleBar` (line 14024), a passive
scale-bar readout, not an interactive measuring tool. This is genuinely new
interaction with no engine backing question at all, because it needs
essentially none: a two-click (or drag) distance readout using the exact
same grid-cells-to-km conversion already used throughout the reference and
this port (`state.mapWidthKm`/`GW`, the same ratio `_civDijkstraPath`'s own
`km` computation and the manual-way straight-line fallback both already
use) — a Godot-side-only feature, arguably the smallest single item on the
whole rail. Optionally: a path-length variant that walks the same
terrain-cost Dijkstra `road_dijkstra` already provides, for "real travel
distance" versus straight-line — a nice-to-have, not required for a first
version. **No pass buffer, no staleness** — purely a query.

**Region select/export**. Real, substantial reference precedent:
`regionSel`/`regionDrag` (line 9583) is a genuine interactive drag-to-select
rectangle tool (`normRegion(x0,y0,x1,y1,...)` on drag-end), feeding two real
downstream operations: `exportRegionTiles` (line 11891 — tiled, gzip-
optional, 16-bit-packed-height region export with a schema-2 manifest,
explicitly headless-testable except the PNG step) and a second real
feature this tool's name doesn't obviously suggest but the reference wires
to the same selection — **region amplification** (`amplifyRegion`, referenced
at line 13212 — resolution upsampling of a selected sub-region into a
higher-detail standalone map, adjusting `mapWidthKm` proportionally). Also
real: `exportGeoJSON` (line 12576) and `drawExportTileGrid` (line 9602, the
visual grid overlay showing tile boundaries before export). **Engine
backing**: **none of `exportRegionTiles`/`exportGeoJSON`/`amplifyRegion`/
`tileDims`/`packHeight16` have been ported to Rust yet** (confirmed by
grep across the workspace) — this is a real, sizeable, entirely unstarted
export subsystem, not a small gap. The *selection* interaction itself
(drag-rect, `normRegion`, re-clamp-on-resolution-change) is simple and
Godot-native; the *export* functions behind it are a genuine porting
project comparable in shape (though not necessarily size) to
`ASSET_LIBRARY_SCOPE.md`'s pack read/write work. **Parameters**: tile grid
(cols/rows), tile size, gzip on/off, for tiled export; target resolution
for amplify. **Pass buffer**: not applicable in the stroke sense — a
region selection is itself the "draft" (visible as an overlay, adjustable
by dragging its handles before committing to Export/Amplify), so the
existing pattern already matches the pass-buffer *shape* without needing
the height-stamp machinery at all.

## Honest milestone breakdown

This is real, substantial scope — `DCC_SHELL_SCOPE.md`'s own expectation
("potentially comparable to Journey Planner or the Asset Library") holds up
under this investigation. Sequencing follows dependency, not tool-rail
order: the shared pass-buffer/staleness mechanism has to exist before any
tool can be more than a UI mock, and the Sculpt-editor terrain port is both
the largest single chunk and the one with the most complete reference
answer, so it anchors the plan.

**Milestone A — `PassBuffer`/staleness core (`cartalith-spatial` +
`cartalith-engine`).** The new `PassBuffer<Stamp>` type from "The shared
editing model" above: stamp storage, touched-tile tracking, preview-via-
scratch-composite, commit-via-real-write, discard. Per-stage `DirtyTracker`
instances wired along the real dependency chain (height → hydrology →
climate → civ), each doing lazy version comparison against its upstream,
no eager cascading. No UI. Verifiable headlessly: commit/discard round-trip
tests, staleness-propagation tests against a small synthetic field. This
is the one milestone every other milestone depends on.

**Milestone B — Terrain group, the Sculpt-editor port.** The largest
single chunk: all thirteen `SCULPT_FEATURES` (`apply()` bodies are small,
pure, individually portable — parallelizable across sub-agents/sessions by
feature if useful), the three noise families, the stamp bbox/coverage/
domain-warp pipeline (`sculptStampBBox`/`sculptApplyStamp`), the eight
presets, wired to Milestone A's `PassBuffer`. Raise/lower, Smooth, Flatten/
terrace, and Stamp (landform library) all ship together here since they
share one underlying registry — splitting them further would just be UI
sequencing, not real engine boundaries. Golden-verification note: this is
new-to-the-port *interactive* behavior with no golden JS-array trace to
diff against stroke-by-stroke (per-user-stroke sequences aren't
reproducible test fixtures the way a deterministic generation pass is) —
verify per-feature `apply()` math against the reference's own formulas
(direct algebraic port, checkable by unit test at fixed inputs) rather than
attempting stroke-sequence parity.

**Milestone C — Water & ecology group.** River/water's special commit path
(`enforceChannelDescent` reuse, the lake `waterOnly` dry-run ordering) on
top of Milestone B's stamp pipeline — real but genuinely more delicate than
a generic terrain feature, kept as its own milestone rather than folded
into B. Biome paint's override-array type and its merge-at-read-site
integration into every current `classify_biome` call site (an audit of
those call sites is real work here — confirm the full list before wiring,
don't assume it's only render-time).

**Milestone D — Civilization group.** Place settlement's manual-insertion
path into the existing `SettlementPlacement`/`NamedSettlement` pipeline.
Draw route/way's waypoint-collection + snap-to-target interaction, reusing
`road_dijkstra` as-is, plus the new `ManualWay` type and the unreachable-
leg warning behaviour. Territory/faction's override-on-`assign_territory`
mechanism, the smallest item in this group since its base algorithm is
already real. These three are more independent of each other than the
terrain group's four are (different data structures, different downstream
consumers), so this milestone could split into three parallel sub-efforts
if useful.

**Milestone E — Annotation & measure group.** *Split in the building, and the
split is part of the definition: Region select/export's compute and encoding
core belongs to E, and its PNG/gzip/`.zip`/GeoJSON half became **milestone
E2** — the boundary and the reasoning are under "Milestone E2 as built"
below.* Label's new struct + arc-text
rendering (the one genuinely new *rendering* problem in this whole plan —
curved text along an arbitrary arc is not something any current Godot
scene in this port does). Icon stamp's manual-placement addition beside
the existing rule-driven system. Measure, trivially. Region select/
export's `exportRegionTiles`/`exportGeoJSON`/`amplifyRegion`/`packHeight16`
port — flagged explicitly as its own real sub-effort, unstarted, comparable
in shape to a small Asset Library-style milestone on its own; consider
splitting it out to its own scope document if Milestone E's other three
items land first and this one is still open (same "don't understate it"
instruction this document was asked to honor).

**Milestone F — Shell wiring.** Connect every tool built in B-E to the
actual tool rail/tool options bar/Properties panel Godot scenes Track 1
built (inert per `DCC_SHELL_SCOPE.md` milestone 1), replacing "honestly
inert" with real behavior tool by tool. Status-bar staleness readout
(`"rivers · deferred"` and its siblings) reading Milestone A's per-stage
`DirtyTracker`s. This is where Track 1 and Track 2 actually merge.

**Not in any milestone above, deliberately deferred**: incremental
(tile-scoped, not whole-field) recomputation of hydrology/climate/civ
stages — Milestone A's staleness model marks work as needed but every
stage still recomputes globally when asked, because none of
`cartalith-hydrology`/`cartalith-climate`/`cartalith-civ` are tile-
incremental today (a much larger, separate re-architecture, out of scope
for "does the tool system work" and only worth taking on if lazy-whole-
recompute proves too slow in practice once Milestone F is real and
measurable end to end).

## Milestone A as built (2026-08-18)

Shipped tested and unwired, the same "ship the primitive ahead of the
orchestration" precedent Phase 2 and the Journey Planner both used. No tool
exists yet; this is the mechanism B-F share.

**Where it landed, and why.**

- `cartalith-spatial/src/pass.rs` — `Stamp` (trait), `PassEntry<S>`,
  `PassBuffer<S>`, `CommitSummary`.
- `cartalith-spatial/src/staleness.rs` — `StageGraph`, `StageId`,
  `Staleness`.
- `cartalith-engine/src/staleness.rs` — `PipelineStage` and
  `pipeline_stage_graph()`: Cartalith's own stage names and edges.

The split follows `cartalith-spatial`'s own precedent rather than the
convenience of one file. That crate's `DirtyTracker` doc comment explicitly
refuses to bake Cartalith field names into a library crate, and `QuadTree`
takes caller-defined aggregate flags for the same reason; the stage *names
and edges* are pipeline knowledge, so they live with the orchestrator that
owns pipeline order. `cartalith-engine` gaining a `cartalith-spatial`
dependency is the first one in the workspace — `LOD_TILING_BASE_SCOPE.md`'s
"whenever a real large-world need actually triggers integration" turned out
to be the tool system, not LOD rendering, and that document's "Done" section
is updated to say so.

**What a `Stamp` actually is, and what this port made of it.** Read directly,
the reference's stamp object is `{type, seed, pts, g:{...}, f:{...}, hidden,
_cx, _cy}` — feature key, seed, the captured stroke polyline in grid
coordinates, two flat parameter bags (the eight global brush/noise keys and
the per-feature control values), a hide flag, and a cached centroid for
radial features. The load-bearing property, which the plan above did not
state explicitly: **a stamp stores no pixel data at all.** It is a *recipe*,
re-evaluated over its own padded bounding box every time it is drawn or
baked. That is exactly why the draft can be kept as plain object state,
JSON-snapshotted for undo, reordered, and thrown away for free.

So milestone A ships `Stamp` as a **trait**, not a struct — `bounds()` and
`apply(&self, dst, width, height)`. The recipe is Cartalith-terrain-specific
and belongs with the feature registry milestone B ports; the stack semantics
around it are generic. A biome-paint disc, a territory-paint disc and a
13-feature landform stamp can all implement it (`type Cell` covers `f32`
height and `u8` categorical override layers alike) without the library crate
learning what a biome is. `hidden` moved off the recipe onto `PassEntry`,
because hiding is a *stack* edit — one of the four structural edits the
reference's own draft undo tracks (add, delete, reorder, hide) — not a
property of the recipe.

`Stamp::apply` writing into a caller-supplied destination is a direct port of
the reference's own contract, quoted verbatim in the module docs:
`sculptApplyStamp` *"writes directly into caller-supplied H/W arrays (never
`field`/module globals) so both the draft preview (a scratch buffer) and
commit (field itself) reuse the identical code path."* One apply, two
destinations. A test asserts preview and commit produce identical results —
the test that would catch them drifting.

**How preview avoids mutating.** `preview_into(base: &[Cell], scratch: &mut
[Cell])`. `base` is a shared reference, so the non-destructive guarantee is
the borrow checker's rather than a convention — no stamp implementation can
violate it. `touched_tiles()`/`touched_bounds()` give the renderer its upload
scope. The whole-base copy is deliberately the simple, obviously-correct
primitive.

*Corrected 2026-09-04.* This paragraph used to end "a touched-region-only
refresh is left to the caller, since no renderer is wired yet to say what
shape it wants." Two renderers have been wired since 2026-08-18
(`build_sculpt_preview_texture`, `build_paint_preview_texture`), and the
bounded refresh is no longer left to the caller: `preview_touched_into(base,
scratch) -> Option<Region>` composites the same stack inside
`touched_bounds()` only, returns the window, and touches nothing outside it.
`build_paint_preview_patch` is its first consumer. Its `None` means *"the
draft touched nothing"* — which is neither "nothing to draw" nor "everything
is dirty", and is the one thing about this API a caller must not get
wrong.

**Corrections this pass made to the plan above.**

1. **`DirtyTracker` needed no extension at all.** The plan's conclusion
   ("necessary but not sufficient — holds no data") is confirmed, but the
   remedy is pure composition: not one method was added or changed.
   `mark_dirty` already *is* "my data changed at this tile, here is why, bump
   the version", which is the single primitive both editing and recomputation
   need. `PassBuffer` uses it at commit; each `StageGraph` stage owns one.
2. **Staleness needs two rules, not one.** The plan describes a stage
   "comparing the version it last computed against the upstream tracker's
   current version". That alone is *not* transitive: a height edit bumps only
   height's version, so climate — comparing against hydrology, whose version
   did not move — would report itself current. The built graph adds rule 2:
   a stage is also stale if an upstream is *itself* stale, evaluated
   recursively at query time. That keeps deferral intact (nothing is pushed
   downstream at commit time; computing a flow change's downstream tile
   footprint is exactly the expensive query the plan rightly refuses to run)
   while making civ correctly stale after a terrain edit. A dirty-flag-only
   design also gets this wrong in the other direction, and there is a test
   for it: recomputing civ over a still-stale hydrology does **not** settle
   civ.
3. **Deferral is structural, not conventional.** `StageGraph` has no
   recompute hook of any kind — no closure, no callback, no trait object it
   could invoke — and every query takes `&self`. It is not merely that it
   *doesn't* recompute; it *cannot*. Work happens only when a caller runs a
   stage itself and says so via `mark_recomputed_tiles`.
4. **The real chain has more edges than the plan's spine.** Verified against
   the real signature, not assumed: `build_settlement_suitability` takes
   `field` (height) and `slope_n` directly, alongside the climate-derived
   soil/water/carrying-capacity inputs. So civ depends on height and
   hydrology *directly*, not only through climate, and `pipeline_stage_graph`
   encodes that. With transitive staleness this does not change *whether* civ
   is stale after a height edit — the spine alone gets that right — but it is
   what a future tile-incremental recompute would have to honour, and it
   makes the graph an honest description rather than a simplified one.
   Erosion is deliberately absent: it is genuinely two-way-coupled with
   climate (`ARCHITECTURE.md`'s known acyclicity pressure point,
   `evolveCoupled()`), a cycle cannot be expressed here by construction, and
   inventing an edge direction before a tool makes the question concrete
   would be guessing.
5. **The dirty flag and staleness are separate concerns.** A stage's
   `DirtyTracker` flag means "this stage's data changed and the presentation
   layer has not re-read it" — a re-upload marker, cleared by `acknowledge`.
   Staleness is computed purely from version counters. Acknowledging never
   changes whether anything is stale, and there is a test pinning that.

**Also built, straight from the reference rather than from the plan:** the
draft-scoped undo/redo stack (`sculptHistory`/`sculptRedoStack`, capped at
`SCULPT_HIST_MAX = 30`, ported as `HISTORY_MAX`), covering all four
structural edits and clearing the redo branch on any new edit exactly as the
reference does. Stack **order** is load-bearing (stamps read the destination
they write, so they compose), which is why `move_up`/`move_down` exist and
why a test uses a set-to-constant stamp to prove reordering changes the
result — an add-only stamp would have hidden it.

**Verified.** 43 new unit tests in `cartalith-spatial` (67 total, up from 24)
and 5 in `cartalith-engine`. The behaviours that matter, each tested for
real: a stroke previews without mutating the field; preview and commit agree
exactly; commit applies the whole stack in order and empties the draft;
discard leaves the field **bit-identical** (compared as raw bit patterns, so
a `-0.0`/`0.0` difference would still fail); one commit bumps each touched
tile's version exactly once however many strokes touched it (the "undo
granularity is one committed pass" rule, enforced in code rather than left to
callers); discarded passes never bump a version at all across repeated
commit/discard cycles; staleness marks exactly the right downstream stages at
exactly the right tiles and never recomputes. `cargo build/test/clippy
--all-targets` clean on both crates.

**Not built, deliberately.** The tools themselves (B-E) and shell wiring (F).
Also: the field-level undo snapshot taken at commit time. The plan lists it
under the shared model, but there is nothing to snapshot *into* yet — no
undo stack exists anywhere in this port — and inventing one before milestone
B has a real committed edit to undo would be guessing at its granularity.
`PassBuffer::commit` returns the exact touched-tile list a tile-diff undo
would need, so the seam is left open rather than filled speculatively.

## Milestone B as built (2026-08-18)

The Terrain group's engine half: the whole thirteen-feature Sculpt registry,
ported and golden-verified, wired to milestone A's `PassBuffer` and to
nothing else. Same "primitive ahead of orchestration" precedent — no Godot
scene, `main.gd` or `cartalith-godot` file was touched.

**Where it landed, and why.**

- `cartalith-terrain/src/sculpt.rs` — the whole port (registry, noise,
  geometry, stamp, `Stamp` impl, 43 unit tests).
- `cartalith-terrain/tests/golden_parity_sculpt.rs` — 23 golden tests.

Not a new crate, and not `cartalith-engine`. Milestone A's split (generic
machinery → `cartalith-spatial`, pipeline knowledge → `cartalith-engine`)
leaves a third category this belongs to: **subsystem-domain math**. All
thirteen features are height-field formulas; `ARCHITECTURE.md`'s "one crate
per subsystem" already names `cartalith-terrain` as the crate that owns the
height formula, and the reference itself keeps `SCULPT_FEATURES` in script
block 1 beside tectonics rather than anywhere near its UI. A
`cartalith-sculpt` crate would have bought a `Cargo.toml` and nothing else —
no second consumer, no independent test boundary (the tests need
`cartalith-noise`, which terrain already depends on). `cartalith-engine`
would be wrong for the mirror-image reason to milestone A's: this is
computation, and *"`cartalith-engine` orchestrates; it does not compute"*.
`cartalith-terrain` gains a `cartalith-spatial` dependency, the workspace's
second (milestone A's `cartalith-engine` edge was the first).

**The real feature registry, as it turned out.** The plan's list above is
confirmed exactly — thirteen entries in `Object.keys` order (mountains,
hills, ridge, plateau, cliff, canyon, valley, river, lake, basin, coastline,
volcano, freehand), eight presets, eight Freehand sub-modes, eight shared
globals. Three things reading it added:

1. **The registry's *order* is load-bearing.** A stamp's effective noise
   seed is `(stamp.seed ^ ((index + 1) * 1013)) >>> 0`, where `index` is the
   feature's position in `Object.keys(SCULPT_FEATURES)`. Reordering the list
   silently re-randomises every stamp in the file. `FEATURE_KEYS` carries
   that warning and a test pins the order.
2. **`edgeChar`/`edgeFreqMul` are per-feature registry data, not derived.**
   Thirteen hand-tuned pairs (Coastline 1.5/0.55, ragged and slow; Mountains
   1.4/1.5, tight and fast; River 0.4/0.8, nearly clean because meander
   already supplies its shape). Ported as data.
3. **Volcano is the one feature that does not use `brushSize`.**
   `sculptStampRadius` special-cases it to its own `volcRadius` control,
   because its cone profile is defined in terms of that radius. Everything
   else — including Lake, the other radial feature — uses the brush.

**How the brush model actually works**, now that it is real code:

- **Coverage** is `smoothstep(0, 1, (R - dist) / feather)` with
  `feather = max(floor, R * (1 - hardness))`. The mockup's "falloff: smooth
  (gauss)" is this smoothstep, and it genuinely is not user-selectable —
  there is one falloff shape in the whole registry.
- **`hardness` shapes, `intensity` scales.** Separate multipliers on
  purpose: coverage decides *where*, `k = cov * intensity` decides *how
  much*. The mockup's two sliders are the two real parameters.
- **Two noise passes, not one.** The domain warp displaces the *sample
  position* (`qx, qy`) before coverage is measured, so the silhouette moves;
  then a second, 3.4× higher-frequency term roughens `cov` itself, but only
  where `cov < 1`, so the interior stays solid while only the rim breaks up.
  Both use `seed + 2100`; the feature bodies' own `fbm`/`ridged`/`billow`
  use `seed`/`seed + 700`/`seed + 1400`.
- **`mode` is `add` or `set`, and which one is a feature's defining trait.**
  `add` → `h0 + k*val`; `set` → `h0 + k*(val - h0)`, a coverage-weighted
  lerp. Plateau being `set`-to-`max(h0, level)` is exactly why it never
  lowers terrain and is a flatten/terrace tool rather than another raise
  brush.

**Corrections and additions this pass made to the plan above.**

1. **The plan's golden-verification note was too pessimistic, and this
   milestone's headline result is the correction.** It says: *"this is
   new-to-the-port interactive behavior with no golden JS-array trace to
   diff against ... verify per-feature `apply()` math against the
   reference's own formulas (direct algebraic port, checkable by unit test
   at fixed inputs) rather than attempting stroke-sequence parity."* That
   conflates two things. A *stroke sequence* is indeed not a reproducible
   fixture — but a *stamp* is, and the reference stores one as plain data
   (`{type, seed, pts, g, f}`). Constructing that object directly and
   calling the real `sculptApplyStamp` under Node needs no pointer events,
   no DOM and no `generate()` run, because the reference itself marks this
   block *"pure, DOM-free core"*. So milestone B got real golden-parity
   after all: **23 cases, every one bit-exact**, not unit-tested algebra.
2. **The plan says "the three noise families"; there are three FBM families
   plus a fourth noise consumer.** `sculptFbm`/`sculptRidged`/`sculptBillow`
   are the three, but the edge warp and the rim-detail term are separate
   uses of `sculptFbm` at their own seed and frequency, and porting them as
   "part of the features" would have missed them — they live in
   `sculptApplyStamp`, not in any `apply()` body.
3. **Smooth also ignores the `waterOnly` flag.** The plan correctly flags
   the pre-loop snapshot. It does not mention that the smooth branch
   `return`s before the water-only check, so a smooth stamp would write
   height even on a water-only pass. Unreachable in practice (only Lake
   stamps are ever passed `waterOnly`), ported as-is rather than "fixed",
   and documented at the site.
4. **`sculptStampBBox` and `sculptApplyStamp` disagree about `feather`,
   deliberately kept.** The bbox computes `max(2, rad*(1-hardness))` for
   every feature; `apply` uses `max(1.5, R*(1-hardness))` for non-radial
   ones. The bbox's floor is the larger, so the box always covers what
   `apply` writes — the inconsistency is harmless, and "fixing" it would
   change which tiles a stamp reports as touched. Ported verbatim.
5. **One deliberate divergence, forced by milestone A's trait signature.**
   The reference reads `state.seaLevel` *live* at apply time, so moving the
   sea-level slider re-renders existing Plateau/Coastline drafts.
   `Stamp::apply` takes only a destination, so `sea_level` lives on the
   stamp, with `with_sea_level()` as the explicit re-stamp. Same result, an
   explicit step instead of an implicit global read. Only two of the
   thirteen features read it.
6. **`Math.hypot` is not `sqrt(x*x+y*y)`** — V8 divides by the larger
   magnitude and Kahan-compensates. Ported as V8 computes it. Measured
   honestly afterwards: swapping in the naive form still passes all 23
   golden cases, because the `f32` store absorbs the difference. Kept for
   fidelity, documented as *not* test-enforced, with the real risk named
   (`nearest_on_stroke` picks a segment by `dist < best`, so an ULP can flip
   the sign of `sd`, which Cliff and Canyon read directly).
7. **`Math.pow`/`Math.exp` needed no tolerance here.** `CHANGELOG.md`
   records a prior `1e-4` allowance for them; these stamps use both and
   still diff bit-exactly, because every value is rounded to `f32` at
   exactly the point the JS `Float32Array` assignment rounds it. (The one
   razor-thin thing: the *test fixture's own* base field must be built in
   `f64` and rounded once at the store. Building it in `f32` shifts the
   field by an ULP and fails all 23 cases — which is how tight this is.)
8. **A known limitation carried over faithfully, not introduced.**
   `docs/SCULPT_EDITOR_INTEGRATION_PLAN.md` §6 left an open item: does the
   stroke-distance code handle world-mode equirectangular wraparound (a
   stroke crossing the antimeridian)? Reading the shipped
   `sculptNearestOnStroke` answers it — **no**, there is no wrap handling;
   the reference shipped without resolving its own open item. This port
   matches. Worth revisiting when world-mode sculpting becomes real, but
   inventing wrap behaviour the reference never had would break parity for
   the common case to fix one nobody has hit.

**Verified.** 43 unit tests in `cartalith-terrain::sculpt` plus 23
golden-parity tests, all bit-exact against the real reference under a Node
`vm.runInContext` harness (four contiguous line slices — 2292-2293,
7568-7569, 8304, 8821-9081 — each with a block-comment balance assertion and
a top-level-boundary check, the technique Journey Planner milestone 4
established; it earns its keep here because the 8821-9081 block both opens
and closes inside a long `/* ... */`, so an off-by-one at either end would
have silently swallowed code rather than thrown). Cases: the twelve
non-Freehand features, Freehand's eight sub-modes, the "Alps" preset, Lake's
commit-time `waterOnly` dry run, and a cross-check that no two features
produce the same field at the same seed (which a harness carrying the same
copy-paste error would not catch). `cargo build/test/clippy --all-targets`
clean on `cartalith-terrain`.

**Not built, deliberately.** `sculptCommit`'s water hooks
(`enforceRiverChannels`, `enforceChannelDescent` + `riverMask`/`riverFloor`
locking, the lake→`lakeMask` deposit) are milestone C — though
`apply_into`'s `water`/`water_only` parameters, the primitive those hooks
consume, are ported and golden-verified here, because they are one branch
inside the function this milestone owns and splitting them out would have
meant porting `sculptApplyStamp` twice. Also not built: the "respect water
mask" gate the mockup shows for Raise/lower (the reference's Freehand has no
water gate at all — a real new feature, not a port), stroke capture and
simplification (`rdpSimplify`/`catmullRomSample` are input routing,
Godot-side), the `SCULPT_COLORS` overlay palette, and all shell wiring
(milestone F).

## Milestone C as built (2026-08-18)

The Water & ecology group's engine half: River/Lake's commit hooks and the
Cartography paint brush, both golden-verified, both wired to milestone A's
`PassBuffer` and to nothing else. Same "primitive ahead of orchestration"
precedent as A and B — no Godot scene, `main.gd` or `cartalith-godot` file
was touched.

**Where it landed, and why.**

- `cartalith-spatial/src/paint.rs` — `PaintStamp`, `PaintLayer` (21 unit
  tests).
- `cartalith-spatial/tests/golden_parity_paint.rs` — 7 golden tests.
- `cartalith-hydrology/src/lib.rs` — `enforce_river_channels`.
- `cartalith-engine/src/sculpt_commit.rs` — `WaterState`,
  `commit_sculpt_pass`, `SculptCommitSummary` (10 unit tests).
- `cartalith-engine/tests/golden_parity_sculpt_water.rs` — 11 golden tests.
- `cartalith-civ/src/lib.rs` — `apply_force_lake` (3 unit tests), closing a
  gap this milestone itself opened; see below.

Three placements, each following the A/B precedent rather than defaulting:

1. **The paint brush is generic machinery, so it is in `cartalith-spatial`.**
   A hard-edged categorical disc over a `u8` grid, gated by a caller-supplied
   exclusion mask, contains no Cartalith semantics — the module never learns
   what a biome is, only that `0` means unpainted. Milestone A's `pass.rs`
   module doc had already anticipated exactly this type (*"a biome-paint
   disc, a territory-paint disc, and a 13-feature landform stamp can all
   implement it"*), and the palettes it indexes stay in `cartalith-civ` where
   Journey Planner milestone 5 ported them. This also means milestone D's
   Territory paint needs no new stamp type at all.
2. **The water commit path is orchestration, so it is in `cartalith-engine`.**
   It composes three crates' primitives (`PassBuffer` from spatial,
   `SculptStamp` from terrain, `enforce_channel_descent` from hydrology) and
   computes nothing new. *"`cartalith-engine` orchestrates; it does not
   compute"* points here, and it is the only crate that already depends on
   all three.
3. **`enforce_river_channels` is hydrology-domain, so it is in
   `cartalith-hydrology`** — three lines from `enforce_channel_descent` in
   this port, as it is three lines from it in the reference.

### What River/water's "special commit path" actually is

The plan named it and left it there. Read directly (reference lines
9318-9346), it is a fixed five-step sequence, and **every step's ordering is
load-bearing**:

1. **Bake the whole stack** — every feature, not just the water ones. This is
   `PassBuffer::commit` unchanged; nothing in milestone C reimplements
   baking, ordering, hidden-stamp skipping or tile marking.
2. **`enforceRiverChannels()`** — re-clamp cells locked by an *earlier*
   commit (or by generation's own `carve_river_valleys`) back to their
   recorded floor. **After the bake, before this batch's carving**, and the
   reference's comment says exactly why: a non-river stamp *"can raise
   terrain over an already-locked river channel ... re-clamp locked cells
   back to their floor before this batch's own river hook carves+locks any
   NEW cells."* Run it before the bake instead and a Mountains stamp painted
   across an old river buries it. This is the step a naive port drops, so it
   has two tests: one that the re-clamp holds, and one proving the same stamp
   *does* raise those cells when no lock is recorded — otherwise the first
   test would pass against a no-op.
3. **Per river stamp, in stack order**: `enforce_channel_descent` over the
   stamp's own stroke, then lock every carved cell into
   `river_mask`/`river_floor`.
4. **Lake, last, as a `water_only` dry run** against the already-final
   height, depositing into `lake_mask`.
5. **One `computeFlow(true)`, one `refreshClimate()`** — *not* ported, by
   design. See point 3 below.

**Corrections and additions reading the reference made to the plan.**

1. **`half_w` is the brush, not the discharge.** The plan says River's commit
   reuses *"the same precedent `carveRiverValleys()` established"*, which is
   true of the carve-and-lock *mechanism* but not of its width.
   `carveRiverValleys` derives `halfW` from Strahler order and a real-km
   scale (`(0.8+0.5*(o-1))*widthK`, capped); `sculptCommit` uses
   `max(1, brushSize*0.13)`. That is the right difference, not an
   inconsistency — a hand-painted river has no drainage area to derive a
   width from — but porting the generated formula here would have silently
   changed every hand-painted river's channel.
2. **`enforceChannelDescent` walks the stroke's own points and never
   resamples.** This is the finding that changed the test fixtures. A
   two-point stroke carves at exactly two sites and locks **3 cells**; the
   same stroke as 23 points two cells apart locks **46**. The reference gets
   away with it because a captured pointer polyline is already dense and
   `rdpSimplify` only removes near-collinear points. It is a real constraint
   on milestone F: **stroke capture must not decimate hard**, or a river will
   carve visibly but lock almost nothing, and later erosion will refill it.
   Testing only the coarse stroke would have exercised the lock barely at
   all, so both fixtures ship.
3. **Steps 2-4 are part of the edit; step 5 is downstream recompute.** The
   plan groups all five as "the commit path" and separately says staleness
   must stay deferred, without saying where the line falls. It falls here:
   2-4 write `field`/`river_mask`/`river_floor`/`lake_mask` — the state a
   commit *produces* — and are local to the stamps' own footprints. Flow and
   climate are whole-field recomputes of downstream stages, and milestone A's
   `StageGraph` exists precisely so they do not run at commit time. So
   `commit_sculpt_pass` runs 1-4 and marks tiles, and never calls
   `compute_flow`.
4. **A draft with no water stamps is bit-identical to a plain commit**, and
   there is a test comparing raw `f32` bit patterns rather than trusting it.
   Callers therefore never have to choose between two commit functions.

### How Biome paint's override layer works — and three things the plan had wrong

The plan's core claim is right: it is a separate override array, not a
mutation of the classifier, `0` = unpainted, and the land-only gate is hard
rather than a toggle. Reading `_paintAt` and its consumers corrected three
things around it, one of them materially.

1. **There are three paint layers, not one.** The plan describes only
   `paintBiome`. The reference ships `paintBiome`, `paintSplat` (asset-pack
   ground textures) and `paintTerrain` (`CART_TERRAINS`, the "surface
   underfoot" palette) as three peer `Uint8Array(GW*GH)` arrays driven by one
   brush and switched by `_paintLayer`. They differ only in which palette the
   value indexes, so one `PaintStamp` serves all three and `PaintLayer` is
   instantiated per layer. Getting this wrong would have built a
   biome-shaped type that the terrain and splat layers then could not use.
2. **The merge is two different operations at two different altitudes, and
   the plan describes the rarer one.** The plan says painted values win
   per-cell and asks for *"an audit of every current `classify_biome` call
   site ... don't assume it's only render-time."* The audit's answer:
   - **Per-cell replace** (`mb[i] = paintBiome[i]`) happens in exactly **one**
     place — the Cartalith editor export (line 12435), which copies
     `buildCartBiome()`/`buildCartTerrain()` and overwrites painted cells
     before encoding. That is `PaintLayer::merge_over`.
   - **The renderer does not replace anything.** `landColorCore` (lines
     7898-7900) takes `pBio`/`pTer`/`pSplat` as three extra arguments and
     alpha-blends the painted index's palette colour over the *fully shaded*
     procedural colour at weight **0.60**, deliberately *"not a rewrite of
     the `materialWeights` mix ... so hillshade/AO/crest/splat/haze still
     show through and painted cells don't read as flat pasted stickers."*
   - **No analysis consumer merges at all.** `buildEcoregions` and every
     Journey Planner `currentCartBiome()`/`currentCartTerrain()` reader take
     the unpainted classifier output, checked at each site rather than
     assumed. Painted overrides are presentation and export in the reference,
     never an input to simulation.

   So the plan's phrasing — merge at *"every render/query site that currently
   calls `classify_biome`"* — would have changed behaviour the reference does
   not have. The rasters those overrides sit on are the ones Journey Planner
   milestone 5 ported (`build_cart_biome`/`build_cart_terrain`, 1-based
   `Vec<u8>` output), which is exactly `PaintLayer`'s shape, so the two fit
   without an adapter.
3. **The gate is `wb[i] !== 0`, not `wb[i] === 1`.** The reference's own
   comment insists this *"excludes BOTH ocean(1) and lake(2), never a bare
   `field[i] < sea` check, which misses above-sea-level lakes."* A port
   gating on ocean alone would pass every ocean test and silently paint over
   lakes, so the golden fixture classifies its water band as **2**.

**Also built, straight from the reference rather than the plan:**
`_paintSampleAt`'s deliberately **nearest-neighbour** sampling (bilinear
*"would blend two unrelated palette entries into a meaningless third index"*),
`getPaintLayer`'s lazy allocation with the v0.148 length guard that a
resolution change must reallocate, and `state.cartoPaint`'s sparse
`[index, value, ...]` persistence including its own drop-out-of-range rule.

**One deliberate new affordance, flagged as new.** `PaintStamp::mask` is
`Option`, and `None` means "no gate". The reference always gates — this
exists because `UI_SHELL_DESIGN.md`'s tool options bar shows a *"respect
water mask"* switch the reference has no equivalent for, the same
mockup-vs-reference gap milestone B recorded for Freehand raise/lower.
Leaving the gate optional makes that switch buildable without a redesign.
The Cartography constructor `PaintStamp::new` requires a mask; the ungated
one is a separate, separately-named constructor, so parity is the default
and the addition is opt-in (`DECISIONS.md` §7d).

**One open question left open, deliberately.** The reference clears painted
overrides on terrain rebuild (`paintBiome=null`, *"hand-painted Cartography
overrides don't survive a terrain rebuild"*). It only ever had one
`generate()`; this port now has *incremental* terrain edits, and whether a
Sculpt commit that changes the temperature/moisture inputs under a painted
cell should also clear that cell has no reference answer. `PaintLayer::clear`
implements the reference-faithful floor (clear on full regenerate) and its
doc names the question. The deciding caller is the shell, which does not
exist yet; inventing a policy for it now would be guessing.

### A gap this milestone opened and then closed

`cartalith_civ::build_water_bodies` had **deliberately omitted** the
reference's `forceLake` option, with a stated and, at the time, correct
reason: *"no painting UI exists in this port, so it would be an
always-false input with no caller ever setting it."* Milestone C is the
producer that reasoning was waiting for — the Lake stamp's commit hook
writes `lake_mask`, and `forceLake` is its only consumer, the thing that
makes a painted lake classify as a lake even when its basin is above sea
level or too arid to pool. Without it, `lake_mask` would have been dead
output.

It ships as `apply_force_lake`, a post-pass rather than a new parameter, and
that is bit-equivalent rather than an approximation: in the reference,
`force` is applied after the depression-pooling pass and is the **last**
mutation of `out` (the only statement after it writes the independent
`fillOut` raster). The post-pass form also leaves `build_water_bodies`'
signature and all of its callers alone — one of which is in
`cartalith-godot`, which this milestone must not touch.

### Verified

**18 golden-parity tests, every one bit-exact on the first run** — 11 for the
water commit path, 7 for the paint brush — against the real reference under a
Node `vm.runInContext` harness. Six contiguous line slices (2292-2293,
7568-7569, 8304, **8725-8745**, 8821-9081, **4758-4795**), each with a
block-comment balance assertion and start- *and* end-of-slice top-level
boundary checks. Both new slices sit hard against comment boundaries:
`enforceChannelDescent` is preceded by a four-line block comment and
`enforceRiverChannels` is followed by one, and the paint block has a comment
opening on the line above it and another on the line below.

The assertions caught two things, in the two different ways they can:

- The **end-of-slice** check threw on `hash/vnoise` — a one-line function
  whose closing brace is not at column 0. A false positive, but the class of
  thing it exists to surface, and fixing it properly (strip trailing line
  comments, require a closing brace, semicolon or comment terminator) rather
  than deleting the check kept it useful for the two genuinely tight new
  slices.
- A failure the balance check is **not** designed to catch, and which
  produced *silently empty* output rather than an error: the reference
  declares `paintBiome`/`_paintLayer`/`_paintValue`/`_paintRadius` with
  `let`, which in a `vm` script are lexical bindings, **not** properties of
  the context object. Setting `ctx._paintRadius` from the host created a
  shadow the reference code never read, so `_paintAt` ran against defaults.
  Everything now drives `_paintAt` from inside the context. This is the same
  class as Journey Planner milestone 5's silently-empty stage list: the
  harness lied quietly rather than throwing.

`sculptCommit`'s water-hook body is **transcribed, not sliced** — the
function opens with `_sculptEditorActive()` and closes with `computeFlow`/
`refreshClimate`/`renderNow`/`sculptSyncUI`, all DOM or whole-pipeline
recompute. Lines 9320-9346 are copied verbatim with `sculptStamps` as a
parameter and those calls dropped. Disclosed rather than implied, because a
transcription is weaker evidence than a slice.

**Milestone B's two fixture findings held.** The base field is built in `f64`
and rounded once at the `f32` store. **No tolerance was needed anywhere** —
heights compare as raw `f32` bit patterns folded FNV-1a-64 over all 4096
cells, and the paint layers are integers. One cross-check worth naming: the
`hidden_river_is_skipped` case reproduces milestone B's own `mountains`
golden hash **exactly**, which is independent evidence that the water hooks
are genuinely inert rather than merely usually harmless.

Unit tests: 21 new in `cartalith-spatial` (88 total, up from 67), 10 new in
`cartalith-engine`, 2 new in `cartalith-hydrology`, 3 new in `cartalith-civ`
(197 total).

**Not built, deliberately.** The tools' *interaction* halves — stroke and tap
capture, the layer/value/radius pickers, the active-layer switch — are input
routing and belong to milestone F. Also not built: the `biomes`/`terrains`
pack-image decode, which `cartalith-godot/src/pack.rs` skipped because
*"there is no producer of a painted-cell array anywhere in this workspace"*
and which named "a future milestone that ports the Cartography paint-brush
tool" as the place to resume — that producer now exists, but the consumer is
a `cartalith-godot` render change this milestone is scoped out of. The 0.60
painted-colour blend in `land_color` is the same case. And automatic river
tooling is untouched: this milestone adds the *manual* stamp path into the
same structures, exactly as the plan said.

## Milestone D as built (2026-08-18)

The Civilization group's engine half: Place settlement's manual-insertion
path, Draw route/way's whole pathfinder and snap interaction, and Territory/
faction's override mechanism — all three golden-verified against the real
reference, all three wired to nothing. Same "primitive ahead of
orchestration" precedent as A-C: no Godot scene, `main.gd`, `main.tscn`,
`render.rs` or any `cartalith-godot` file was touched.

**Where it landed, and why.**

- `cartalith-civ/src/tools.rs` — the whole milestone (28 unit tests).
- `cartalith-civ/tests/golden_parity_civ_tools.rs` — 16 golden tests.
- `cartalith-civ/src/lib.rs` — a widened `TerrainValid` (was a `bool`),
  `js_hypot`, and a real bug fix in `civ_smooth_path` (both below).

One file, one crate, and deliberately so. Milestone A's split (generic →
`cartalith-spatial`, pipeline → `cartalith-engine`) and milestone B's third
category (subsystem-domain math → the owning crate) put all three tools in
`cartalith-civ`, and not merely by elimination: **every one of them is a
manual entry point into a pipeline this crate already owns.** Manual
settlement insertion appends into the same `Vec<NamedSettlement>`
`place_settlements`/`name_and_populate_settlements` produce, so naming,
roads and territory cannot tell a hand-placed settlement from a generated
one. Manual ways reuse `road_dijkstra`, `civ_routing_grid`,
`civ_apply_settlement_gravity` and `civ_smooth_path` — four helpers that are
**private to this crate**, which a `cartalith-civ-tools` crate could not
even see. Territory paint merges over `assign_territory`'s own output.
`cartalith-engine` is wrong for milestone B's reason (*"`cartalith-engine`
orchestrates; it does not compute"*); `cartalith-spatial` is wrong for
milestone C's (a Dijkstra that knows about factions, settlement tiers and
sea lanes is not generic machinery).

**Milestone C's prediction held exactly, and it is why this milestone is
tiny in one place and large in another.** Milestone C wrote: *"this also
means milestone D's Territory paint needs no new stamp type at all."* True —
`PaintStamp::ungated` **is** `_civPaintTerritoryAt`, cell for cell, because
`_paintAt`'s own comment calls itself *"a direct lift of
`_civPaintTerritoryAt`'s geometry"* and milestone C ported `_paintAt`. So
the entire new surface territory painting needs is a five-line
`merge_territory_paint`, golden-verified by hashing the whole raster against
the reference's own `civTerritory`. Draw route/way, by contrast, turned out
to be the largest single item in this group by a wide margin — the next
section is the headline correction.

### The headline correction: `_civDijkstraPath` is not `road_dijkstra`

The plan says, of Draw route/way: *"the pathing primitive is not new —
`road_dijkstra` (`cartalith-civ/src/lib.rs:3269`) is exactly this same
Dijkstra-over-terrain-cost, already ported and golden-verified for
`build_road_network`. What's genuinely new is: (a) a waypoint-collection
interaction, (b) a `ManualWay` struct ... and (c) the unreachable-leg
fallback-to-straight-line-and-warn behaviour."*

Checked against the reference, as the brief asked: **that is wrong, and the
gap is most of the tool.** This port's `road_dijkstra` is the reference's
`roadDijkstra` (line 3275) — the bare single-source relaxation kernel over a
caller-supplied cost array. `_civDijkstraPath` (line 25957) is one of its
*callers*, and calls it at exactly one line. Everything that makes a route a
route lives in the wrapper, and none of it was ported:

1. **Three cost grids, not one.** `_civLandCostGrid` (21035: slope cost with
   *all* water impassable — sea via `buildTravelCost`, above-sea lakes via
   the water-body overlay, because a bare `field < sea` check misses those),
   `_civWaterCostGrid` (21051: the mirror image — any water flat-cost 1, land
   impassable, lakes deliberately included unlike `_civMstRoutes`' ocean-only
   grid), and `_civMixedCostGrid` (21090: land+water so a route crosses open
   water when that is genuinely cheaper — slope × biome friction × the shared
   navigable-river discount, with `_CIV_SEA_COST = 0.6` *below* the flat-land
   baseline, v0.94's deliberate correction).
2. **The existing-way discount** (`_civMarkWaysOnGrid`/
   `_civMarkWayNeighborhood`/`_civWalkWayCells`, 21757/21752/21766, and
   `_CIV_EXISTING_WAY_DISCOUNT = 0.25`). `_civWalkWayCells` in particular is
   not "iterate `pts`": it *rasterizes the segments between* the sparse
   smoothed sample points, because `pts` alone is gappy on long straights and
   routers used to ignore half a road.
3. **Settlement gravity** — already ported
   (`civ_apply_settlement_gravity`), but never before called from a manual
   tool.
4. **Path reconstruction into world coordinates**, `((rx+0.5)/sc,
   (ry+0.5)/sc)` per routing cell, with the caller's own exact endpoints
   restored at full precision.
5. **Wrap-aware smoothing with a *mode-matched* validity repair pass**,
   which needed two `_civTerrainValidTest` modes this crate did not have.
6. **The `reachable` flag** (v1.47) — the only way a caller can tell a real
   path from the synthesized fallback.

The port is `civ_dijkstra_path`; the plan's (a)-(c) are all real, and all the
small part. Two of the reference's own signals make the distinction visible
in hindsight: `roadDijkstra` sits in script block 1 beside `buildTravelCost`,
~22 500 lines before `_civDijkstraPath`, and `_civDijkstraPath`'s own header
says it *"mirrors buildRoadsOp"* — it mirrors a *caller*, not the kernel.

**This unblocks the Journey Planner.** `JOURNEY_PLANNER_SCOPE.md`'s closeout
recorded `_jpRerouteForMode` as the subsystem's one still-blocked function,
because *"its whole body is `_civDijkstraPath`, the Route tool's unported
multi-modal pathfinder."* That pathfinder now exists, with all three of its
domains and its `reachable` flag — precisely what `_jpRerouteForMode` checks,
since it *"never silently accepts `_civDijkstraPath`'s straight-line fallback
as if it were a real path."* `JOURNEY_PLANNER_SCOPE.md` is updated. What is
left there is `_jpModeForRoute`'s three-line transport→domain mapping and the
re-route action itself, which is UI.

### What the reference corrected, tool by tool

**Place settlement.**

1. **The gate order is load-bearing and is not the obvious one.** Bounds,
   then **select-near-existing**, *then* the water refusal. A settlement whose
   terrain later changed under it is still selectable — the reference's own
   v1.86 comment worries about exactly that. A port that checked water first
   would make such a settlement unclickable.
2. **`_civPlacePickWeight` is a prominence weighting, not nearest-pixel**
   (v1.88): the winner minimises `d² / weight²` with `weight = 4 + rank`,
   mirroring `drawCivLayer`'s own pin-size formula, so a big city beats a
   small hamlet that is slightly closer. The absolute pick radius is unchanged
   by it. This port models five of the reference's ten settlement classes and
   has no POI concept, so the two special-kind ranks and the POI's flat weight
   of 5 are absent rather than approximated.
3. **Three fields the reference's place object does not carry.** Its place is
   `{x, y, name:'', kind:'town', faction, pop:1000, traits:[]}`; this port's
   `SettlementPlacement` stores `suit`, `capital` and `coastal`, which the
   reference recomputes on demand. `capital` follows from `kind`; `coastal`
   uses the same `civ_is_coastal(.., ocean_only = true)` call and the same
   `max(6, gw/60)` radius `place_settlements` uses, so a hand-placed port is
   coastal on the same test a generated one is; `suit` is the caller's, which
   may sample its own suitability raster.
4. **Name and population stay the reference's placeholder**, `""` and `1000`
   — deliberately **not** `civ_base_pop_for_kind`. The plan floated running a
   hand-placed settlement through `civ_settle_name`/`civ_base_pop_for_kind`
   immediately. Both are public and a shell may do exactly that, but doing it
   *inside* `civ_drop_place` would consume draws from `civ_name_rng`'s stream
   out of band and silently rename every subsequently generated settlement. It
   has to be the caller's explicit step.

**Draw route/way.**

1. **The plan's conflation warning is confirmed — and there is a second,
   closer trap it does not name.** The plan flags `_civOpenRouteEditor` (the
   Journey Planner's editor over an *existing* journey); the real match is
   `_civTool === 'draw_way'` → `_civCommitWay`. Correct. But
   **`_civCommitRoute` sits eighteen lines above `_civCommitWay` in the same
   file**, looks nearly identical, and is a different tool: it routes
   `'mixed'` and pushes to `civJourneys`, while `_civCommitWay` routes
   `'water'` for sea lanes and `'land'` for everything else and pushes to
   `civWays`. Porting the wrong one would let a hand-drawn road cut across a
   bay *and* file the result in the Journey Planner's list.
2. **The unreachable fallback is *not* a straight line from start to end.**
   The v1.99 comment says *"a straight line was drawn there instead"*, and the
   plan repeats it. What actually happens: the `{pts: fp, ...}` straight-line
   branch only runs when `_civSmoothPath` returns `null`, and for a distant
   unreachable target it does not. The reconstruction produces
   `[start, targetCell, end]`; `_civSmoothPath` splits runs at any
   `|Δx| > GW/2` jump — **unconditionally, world mode or not** — the
   start→target-cell hop *is* such a jump, and the run holding the start has
   length 1 and is dropped entirely. The drawn stub therefore sits at the
   **target** end and the start point is absent. Golden-verified, and pinned
   by a test asserting the start really is missing so nobody later "fixes" the
   port into disagreeing with the reference. It also means the shell's warning
   must not promise the user a line between their two waypoints.
3. **`_civTerrainValidTest` needed all four of its modes.** This crate had
   two (`'land'` with no exception, `'ocean'`), inlined as an `is_sea: bool`
   on `civ_smooth_path`. `civ_dijkstra_path` needs `'water'` (ocean *or*
   lake), `'land'` **with the v1.99 sea-lane ferry exception**, and
   `undefined` (mixed repairs nothing, having no forbidden terrain). The ferry
   exception is not decoration: land mode's own discount block is *the only
   place in the whole reference where a land-mode `Infinity` cell becomes
   finite*, so without the matching exception the repair pass would drag a
   legitimate ferry leg back onto dry land. There is a golden test for exactly
   that, and its negative half (the same pair is unreachable with no lane).
4. **`_civWalkWayCells` is used twice with a deliberate asymmetry.**
   `_civMarkWaysOnGrid` skips `w.hidden`; the sea-lane-cell collection inside
   `_civTerrainValidTest` does not. Ported as written, with the asymmetry
   commented at the site rather than smoothed over.
5. **`state.roads.edges` has no equivalent here.** `_civDijkstraPath` also
   discounts that array — `buildRoadsOp`'s legacy Edit-tab output, which this
   port never had. A caller's generated `Way`s go through the same `ways`
   slice instead, reaching the same cells by a different route. Named rather
   than silently dropped.
6. **Rebuilding the cost grid per leg is kept.** `civ_join_dijkstra_segs`
   calls `civ_dijkstra_path` once per leg, so an n-waypoint way builds n−1
   grids — the reference's own behaviour. Hoisting would be a real
   optimisation *and* a real divergence, since the way discount and settlement
   gravity both mutate the grid in place.

**Territory/faction — flagged as an addition, not parity.**

The reference has a territory *paint brush* and nothing else.
`PHASE2_SCOPE.md`'s milestone-9 investigation (re-checked here, not
re-derived) found `getCivTerritory()` only lazily zero-allocates the array,
and its sole writers are `_civPaintTerritoryAt` and a save/load deserializer
— **the reference never had algorithmic territory generation at all**. This
port does (`assign_territory`, `DECISIONS.md` §7b, its own design). So this
tool paints over a computed base the reference never had: a superset under
`DECISIONS.md` §7d, recorded as an addition. The *brush* is a faithful port;
what it composites onto is new. Two specifics:

- **`ungated`, not `new`.** `_civPaintTerritoryAt` has **no land/water
  gate**, unlike `_paintAt`'s hard `wb[i] !== 0` refusal — a faction can own
  coastal water and lake surface. Milestone C's `PaintStamp::ungated`
  constructor, added there for the mockup's *"respect water mask"* switch,
  turns out to be the reference-faithful choice here.
- **Faction ids widen at the merge.** `civTerritory` is a `Uint8Array` in the
  reference; `assign_territory` returns `Vec<i32>`. A `u8` override layer
  therefore covers every faction the reference could express, and the widening
  happens at `merge_territory_paint` and nowhere else.

### Two bugs found in already-shipped, already-golden-verified code

Both were latent because no prior fixture in this crate had a **wrapped**
route, and both are fixed with every pre-existing golden test still passing.

1. **`civ_smooth_path` summed `km` across run boundaries.** The reference
   guards the accumulation with `if(k > 0)` — `k` being the index *within the
   current run* — so the jump from one run to the next, exactly the seam a
   `brks` entry marks, is deliberately excluded. The port used "if anything
   has been pushed", which is the same thing for a single-run path and wrong
   for every multi-run one. Milestone D's case 1 is the first wrapped fixture
   this function has ever had: it reported 876.8 km for a route the reference
   measures at 136.6 km — one whole map width, added per seam crossing. Every
   consumer of a wrapped way's length (`civ_consolidate_and_smooth_ways`,
   `civ_sea_routes`, and now manual ways) was affected.
2. **`Math.hypot` is now genuinely test-enforced.** Milestone B ported V8's
   compensated `Math.hypot` and recorded, honestly, that its own fixtures
   could not distinguish it from `sqrt(x²+y²)` — every case still passed with
   the naive form, because an `f32` store absorbed the difference. Milestone D
   found the fixture that can. `_civSmoothPath` accumulates `km` in `f64`
   across dozens of segments with no rounding step anywhere, so one ULP
   survives to the result: case 1's unreachable land route is
   `610.6390435628962` with Rust's `f64::hypot` and `610.6390435628963` — the
   reference's own value — with V8's. `cartalith-civ` now has its own
   `js_hypot` (identical to `cartalith-terrain`'s) applied across the
   route-geometry chain: `civ_rdp_simplify`, `civ_catmull_rom_sample`,
   `civ_smooth_path`, and `civ_dijkstra_path`'s fallback. The crate's other
   `.hypot()` sites (slope gradients, the Journey Planner's wrap-aware leg
   lengths) are deliberately **not** changed: they are covered by their own
   passing goldens, and editing verified code on an unmeasured hunch is the
   thing this project's discipline exists to prevent. Worth a sweep the day a
   fixture distinguishes them there too.

### No `PassBuffer` anywhere in this milestone, deliberately

The plan predicted this for two of the three tools; it held for all three.
Place settlement is *"a discrete, already-atomic action (append one struct),
not a brush stroke needing preview-before-commit"*. Draw route/way's
*"in-progress waypoint chain is itself the natural pass-buffer unit"*, and
`civ_commit_way` takes that chain as a plain slice — the reference's
`_civWayWaypoints`, exactly. Territory paint's staging is `PaintLayer`, which
milestone C already built; adding `PassBuffer<PaintStamp>` on top would have
been a second staging mechanism over the same data with nobody asking for it.

### Verified

**16 golden-parity tests, every one bit-exact** — including `km` compared as
raw `f64` bit patterns rather than with a tolerance, and the territory raster
compared as an FNV-1a-64 over all its bytes. **No tolerance was needed
anywhere.**

The harness is a Node `vm.runInContext` run of **whole `<script>` blocks, not
line slices**: blocks #1 (2084-14556) and #2 (14563-26720), the same
boundaries `golden_parity_hierarchical_network.rs` documents. That is a
materially stronger boundary guarantee than milestones B/C's contiguous
slices — the delimiters are the real `<script>`/`</script>` tags, and the
harness asserts exactly that (the line before each slice *is* `<script>`, the
line after *is* `</script>`) rather than inferring a top-level boundary from
indentation.

The block-comment balance assertion and Urban M2's orphan-close counter ran
anyway, and **both times they fired they were wrong** — which is how a check
of this kind proves it is looking at all:

- Two false orphan `*/` in block 2, both real comments. Cause: a crude "scan
  to the next quote of the same kind" string skipper desynchronises on
  **nested template literals**, of which this reference has many. Fixed with a
  real template-literal stack, not by deleting the check.
- Then an unbalanced-backtick report. Cause: **regex literals containing a
  bare `"`** (`k.replace(/"/g, '&quot;')`), read as a string opener. Fixed
  with a regex-literal skipper.

**Emptiness assertions, because three subsystems have now been bitten by
silently-empty output that passed every structural check.** Before any golden
value was written down, the extraction asserted: every "should route" path has
≥ 2 points and `km > 0`; every "should not route" path reports
`reachable === false` (real negative controls, not absent assertions); the
territory brush painted a nonzero cell count; the drop tool appended exactly
one place; the unreachable commit produced a non-empty warning. All are
re-asserted on the Rust side.

**The world under the tools is bit-identical, checked before trusting
anything.** The harness's `field`, water-body classification, biome raster and
Strahler river order were FNV-1a-64'd over their raw bytes and compared
against this port's own `generate_terrain` + `build_water_bodies` +
`build_biome_raster` + `fresh_river_order`. All four hashes matched exactly in
both cases, as did the land/ocean/lake cell counts. Both fixtures contain real
ocean, real land and at least one real lake, so the ocean-vs-lake distinction
every water gate turns on is genuinely exercised — case 1 has 42 lake cells.

Case 0 (`gw=24 gh=18 seed=24601 world=false`): a western landmass, an ocean,
an eastern strip — so a land route between two western points is real, a land
route east is genuinely unreachable, and `mixed` connects them by crossing
water. Case 1 (`gw=20 gh=16 seed=314159 world=true`) wraps, and its ocean is
connected *only through the seam*, so both its land and its water route come
back carrying a `brks` entry — the wrap-aware smoothing path that found bug 1
above, and which no fixture in this crate had ever reached.

Six presentation-only functions (`_civRenderPlaceEditor`, `_civRenderWayList`,
`_civRenderJourneyList`, `_civUpdatePlannerPanel`, `drawCivLayerAuto`,
`renderNow`) are neutralised **inside** the context by reassigning their
bindings. Disclosed because it modifies the reference environment — but note
what it is not: no tool body is transcribed or edited (unlike milestone C's
`sculptCommit`, which had to be), and none of the six touches routing,
placement or paint state. `_civRenderPlaceEditor` reaches `_umSiteProfile`,
which lives in script block #3 and is deliberately not loaded.

Everything is driven from **inside** the context — milestone C's lesson.
`civWays`, `_civActiveFaction`, `_civTerRadius`, `_civWayWaypoints` and
`civTerritory` are all `let` declarations, which in a `vm` script are lexical
bindings rather than context properties; setting them from the host would have
created shadows the reference never reads, and the failure mode would again
have been silently-empty output.

Unit tests: 28 new in `cartalith-civ` (225 total, up from 197). `cargo build`
/ `cargo test` / `cargo clippy --all-targets` clean on `cartalith-civ` (two
pre-existing `needless_range_loop` warnings predate this work and are
untouched). `cargo test --workspace`: 842 passing, 0 failures.

### Not built, deliberately

The tools' **interaction** halves — waypoint capture and the Escape/commit
keybinding, the active-faction quick-select the plan notes Place settlement
and Territory should share, the brush-radius and way-type pickers, the
snap-on/off switch (`state.viz.snapWays`) — are input routing and belong to
milestone F. `civ_zoom_pick_r` is exposed so the shell supplies its own zoom
rather than the engine inventing a view model.

Also not built: `_civCommitRoute` and `civJourneys` (the general Route tool —
a Journey Planner surface, not one of this group's three), `_civDropPOI` (this
port has no POI concept), `_civConnectPlaceToNetwork` (the "connect a
hand-placed settlement to the network" spur — a real function with a real port
ahead of it, but not one of the rail's fifteen tools), and
`_civGenerateProvinces` consuming a *painted* territory raster.

## Milestone E as built (2026-08-18)

The Annotation & measure group's engine half — Label, Icon stamp, Measure, and
the compute/encoding core of Region select/export — all golden-verified, all
wired to nothing. Same "primitive ahead of orchestration" precedent as A-D: no
Godot scene, `main.gd`, `main.tscn`, `render.rs` or any `cartalith-godot` file
was touched, and `cartalith-urban` (a sibling's milestone 4) was left alone.

**Where it landed, and why.**

- `cartalith-civ/src/labels.rs` — Label (35 unit tests).
- `cartalith-civ/tests/golden_parity_labels.rs` — 21 golden tests.
- `cartalith-assets/src/manual.rs` — Icon stamp (29 unit tests).
- `cartalith-assets/tests/golden_parity_manual_icons.rs` — 7 golden tests.
- `cartalith-spatial/src/measure.rs` — Measure (12 unit tests; **no golden
  test, and there cannot be one** — see below).
- `cartalith-spatial/src/region.rs` — `norm_region`, `tile_dims`,
  `FloatRegion` (13 unit tests).
- `cartalith-spatial/tests/golden_parity_region.rs` — 2 golden tests.
- `cartalith-terrain/src/amplify.rs` — `amplify_region`, `refine_tile`
  (16 unit tests).
- `cartalith-terrain/tests/golden_parity_amplify.rs` — 11 golden tests.
- `cartalith-io/src/tiles.rs` — `pack_height16`/`unpack_height16`,
  `TileManifest`, `manifest_json` (20 unit tests).
- `cartalith-io/tests/golden_parity_tiles.rs` — 8 golden tests.
- `cartalith-engine/src/region_export.rs` — `export_region_tiles`
  (7 unit tests).
- `cartalith-engine/tests/golden_parity_region_export.rs` — 1 golden test.

Six placements, each argued from A-D's rule (generic machinery →
`cartalith-spatial`, pipeline knowledge → `cartalith-engine`, subsystem-domain
math → the owning crate) rather than by convenience:

1. **Label is `cartalith-civ`.** The reference's own `_civ`-prefixed family:
   `state.labels` sits beside `state.places`, and labels draw in
   `drawCivLayer` beside settlements, ways and territory — all of which this
   crate already owns after milestone D. Nothing about a label is generic (its
   box is sized from *this map's* zoom-relative icon scale) and there is no
   second consumer that would justify a crate, milestone B's argument against
   `cartalith-sculpt`.
2. **Icon stamp is `cartalith-assets`.** It is the manual half of a
   rule-driven system that crate already owns: a manual icon addresses the same
   slot vocabulary, and `icon_brush_rule` reads the very same `ScatterRule`
   table `place_map_icons_ruled` does.
3. **Measure is `cartalith-spatial`.** A wrap-aware distance over a grid with a
   km scale is generic machinery, and it is an *addition* — see below.
4. **The region rectangle is `cartalith-spatial`**, reusing the `Region` that
   crate already had (`norm_region` needed no new type). `FloatRegion` is new
   and load-bearing: `refine_tile`'s sub-bounds are `region.w / cols`, which is
   generally not an integer, and rounding them would break the exact seam
   agreement the whole tiling model rests on.
5. **`amplify_region`/`refine_tile` are `cartalith-terrain`**, milestone B's
   third category — a height-field upsample plus `fbm`/`ridged` detail tapered
   by relief and faded below sea level is a height formula start to finish.
6. **The encodings are `cartalith-io`** (this crate owns what a Cartalith file
   looks like on disk) and **the composition is `cartalith-engine`** (*"it
   orchestrates; it does not compute"*, milestone C's placement for
   `commit_sculpt_pass`). `cartalith-engine` gains a `cartalith-io`
   dependency, its first.

### Region select/export did need a split — and here is the boundary

The plan flagged this as *"a real, sizeable, entirely unstarted export
subsystem"* and asked, if it stayed large, to *"split it out honestly."*
Reading it, the split is real but it does **not** fall where the tool's name
suggests. `exportRegionTiles`' body is four calls and a loop; everything hard
in it is either pure geometry (which ships here) or a browser API (which
cannot). So milestone E ships the **compute and encoding core** and defers a
**milestone E2** that is entirely *format and pixels*:

| Shipped in E | Deferred to E2 |
|---|---|
| `normRegion`, `tileDims` | per-tile PNG (`tilePngBytes`, an `OffscreenCanvas` hypsometric-tint + hillshade pass) |
| `amplifyRegion`, `refineTile` | `gzipBytes` (`CompressionStream`) |
| `packHeight16`, `unpackHeight16` | the `.zip` assembly (`zipStore`) |
| `buildTileManifest` + byte-exact JSON | `exportGeoJSON` (12576) and its `_geoXY`/`_geoTerritoryFeature`/`_geoProvinceFeature` raster-to-vector boundary tracer |
| `exportRegionTiles`' own assembly, minus the two browser steps | `regionNewWorldBtn`'s replace-the-world path (`allocate`/`refreshClimate`/civ clear — orchestration over a live `WorldState`, not geometry) |

That is the honest boundary, and it is a *smaller* E2 than the plan feared:
the geometry it was worried about is done and bit-exact, and what is left is a
PNG encoder (`cartalith-assets` already carries `image` with the `png` codec),
a gzip crate, and a GeoJSON serializer — none of which needs the reference to
be re-read for its *math*, only for its *shape*. `burnChannels` (10373) is
deliberately in neither: it belongs to the LOD viewer, not to this tool.

The **selection interaction** — `regionSel`/`regionDrag`, the dashed overlay,
`drawExportTileGrid` — is pointer routing and belongs to milestone F, exactly
like every other tool's interaction half.

### What the reference corrected, tool by tool

**Icon stamp — the plan describes the wrong function.**

The plan says `_carIconBrushStamp` (15051) is *"stamp mode (place one icon by
hand at a clicked point)"*. It is not. There are **three** placement paths in
the reference, not two:

1. Rule-driven autoplacement (`placeMapIconsRuled`) — already ported.
2. **Click-to-place one icon** — the `_iconPlaceMode` branch of the click
   handler (9776-9784). *That* is the "place one icon by hand" path, it is
   four lines, and it is `place_manual_icon`.
3. **A dart-throwing scatter brush** — `_carIconBrushStamp`, which paints a
   blue-noise *stand* of icons under a radius as the pointer drags, with a
   rejection radius tested against both the icons already on the map and the
   ones this stamp is placing. This is by far the larger of the two manual
   paths and the plan does not describe it at all.

And it is **deliberately non-deterministic**, which changes how it can be
verified. The reference's own comment: *"Unlike the procedural scatterer this
uses `Math.random`, not `hash()`: a brush stroke is an authoring ACTION whose
result is persisted in `state.mapIcons` — re-painting the same spot should add
new icons, not deterministically reproduce the previous ones."* So
`icon_brush_stamp` takes its randomness as a parameter (`&mut dyn FnMut() ->
f64`), the harness overrode `Math.random` **inside the vm context** with a
32-bit LCG, and this port drives the identical stream. Because the RNG is
consumed three times per accepted dart and twice per rejected one, matching all
49 placed icons across ten runs pins the exact sequence of accept/reject
decisions, not merely the outcome — one extra or missing draw anywhere
desynchronises every later dart.

Two smaller corrections: the click path has **no sea-level gate** (only the
brush does), ported as written rather than "fixed"; and `icon.fam`'s
`'feature'` maps to the *pack* family `icons`, a rename between two taxonomies
that a shared enum would have had to answer to twice, so `ManualIconFamily` is
its own type with a `pack_family()` bridge.

**Label — the plan's "arc-text layout" is real, and it splits cleanly at text
measurement.**

`drawArcLabel` (15244) is a Canvas function, but only two of its inputs come
from the canvas: `measureText(text).width` and the per-`char` advances. Those
are properties of the loaded font, not of the geometry, so `arc_label_layout`
takes them as parameters and returns one `{dx, dy, rot}` per glyph in the
label's own frame. The renderer applies them. Nothing in `cartalith-civ`
touches a canvas, and the crate stays free of Godot exactly as
`ARCHITECTURE.md` requires.

Reading it added four things the plan does not have:

1. **`total_w` and the per-char widths are separately load-bearing.** The
   function reads the measured total once for the centring offset and the
   per-char widths inside the loop, and in a real font those disagree because
   of kerning. A port that summed the char widths would drift on any kerned
   string. The harness's stub metrics therefore make them *deliberately*
   unequal, so the fixture can see the difference.
2. **`sizePx` is truncated for the font string but not for the geometry.**
   `${sizePx|0}px` sets the measuring font; the untruncated value feeds the
   arc-radius floor `max(sizePx*1.2, …)` and the halo `max(1, sizePx*0.16)`.
   Ported as written.
3. **The `|arc| < 0.01` straight branch is not an optimisation.** Below it the
   derived radius `total_w / (2.2*|a|)` diverges, so the arc is meaningless
   rather than merely subtle.
4. **The two commit semantics the plan noticed are exact, and asymmetric.**
   `_civSelectLabel`'s own comment: the snapshot is taken *"once per edit
   session (re-clicking/dragging an ALREADY-selected label does not retake the
   snapshot)"*, and *"x,y are deliberately excluded — dragging to reposition
   commits immediately"*. `LabelEditSession` implements both, and the golden
   pins that a cancel reverts the name and **does not** revert the position.

The three on-canvas handle formulas (resize, rotate, arc) are **transcribed,
not sliced** — they are inline in a `pointermove` listener, not callable
functions, so there is nothing to call. Disclosed here rather than implied, the
same disclosure milestone C made about `sculptCommit`'s body.

**Measure — confirmed as an addition, flagged as one.**

The plan's *"zero reference precedent"* holds: re-grepping finds only
`updateScaleBar` (14024), a passive scale bar. So there is **no golden-parity
test for `cartalith-spatial::measure` and there cannot be one**; it is
unit-tested against its own contract and recorded as new under
`DECISIONS.md` §7d rather than presented as parity. What it *is* faithful to
is the km scale, which is not invented: `hypot(dx,dy) * map_width_km / gw` is
the same expression `civ_smooth_path`'s golden-verified `km` accumulation,
`civ_catchment_radius_cells`' `cell_km` and `_geoCellKm` all use, and a test
compares the two as raw `f64` bit patterns. The plan's optional terrain-cost
variant is **not** built; if it ever is it belongs in `cartalith-civ` beside
`civ_dijkstra_path`, not here.

**Region export — a real division by zero, ported rather than fixed.**

`amplifyRegion`'s coordinate mapping is
`cy = rh > 1 ? ry + (oy/(outH-1))*(rh-1) : ry`. With `outH == 1` **and**
`rh > 1` that is `0/0`, and the entire output comes back `NaN` — verified
against the reference, not inferred, and pinned by a golden case. No shipped
caller reaches it because `tileDims` floors both edges at 2px. Ported as
written (`DECISIONS.md`: the reference's behaviour is the specification), with
a companion fixture where the region *also* collapses to one cell and the
result is legitimately finite — the pair is what distinguishes `rh > 1` from
`rh >= 1`.

This also forced `js_min`/`js_max` in `amplify_region`: `Math.min(1, NaN)` is
`NaN` in JS, while Rust's `f64::min` **returns the other operand** and would
have silently turned an all-NaN tile into a plausible-looking one.

**The manifest JSON is written by hand, on purpose.** `serde_json` renders an
`f64` of `16.0` as `16.0`; `JSON.stringify` renders it as `16`. The manifest's
`coarse` bounds are `bounds.w / cols` — an integer for most tile grids and a
fraction for the rest — and a schema-2 manifest is a file other tools read. So
`manifest_json` formats numbers the way `Number.prototype.toString` does, and
one golden case uses `cols = 7` over `bounds.w = 30` specifically so every
`coarse.x`/`coarse.w` in it is a long fraction.

### Verified

**49 golden-parity tests**, plus 132 unit tests across the six modules.

Harness: a Node `vm.runInContext` run of **whole `<script>` blocks** — #1
(2084-14556) and #2 (14563-26720) — with the harness asserting that the line
before each slice *is* `<script>` and the line after *is* `</script>`, the
stronger boundary guarantee milestone D established. Everything is driven from
**inside** the context (milestone C's lesson about `let` bindings not being
context properties); `renderNow`, `drawCivLayerAuto`, `_civRenderLabelEditor`,
`_civRenderLabelList`, `_carRenderIconList` and `_carSelectIcon` are
neutralised inside the context and none of them touches layout, placement or
routing state.

**Two environment modifications, both disclosed rather than implied:**

1. `drawArcLabel` and `_civLabelBox` take a Canvas 2D context, so the harness
   supplies a stub that records `translate`/`rotate`/`strokeText` and answers
   `measureText` from a fixed formula. No function body is transcribed or
   edited — the layout arithmetic under test runs inside the real
   `drawArcLabel`; the stub supplies only what a font supplies and receives
   only what a transform is.
2. `Math.random` is replaced, inside the context, with a seeded LCG, because
   `_carIconBrushStamp` is deliberately unseeded (above).

**The balance check fired twice, and was wrong both times** — which is how a
check of this kind proves it is looking, exactly as in milestone D:

- An orphan `*/` in block #2, inside a real comment. Cause: milestone D's
  template-literal stack still mishandled a `}` closing an *object or arrow
  body* inside a `${ }` substitution, which ends the substitution early and
  desynchronises everything after it. Fixed with a brace-depth-anchored
  substitution stack, not by deleting the check.
- Then six orphan `*/` in block #1, at lines like `c0.waveStr/Math.max(...)`.
  Cause: the regex-literal skipper's "does a value precede this `/`?" test
  matched a **single** identifier character, so any multi-character identifier
  read as "no value precedes" and the divide was consumed as a regex. Fixed.

The documented **apostrophe-in-prose** blind spot appeared as a *symptom* of
the first of those rather than as a cause: the desynchronised scan re-entered
code inside a block comment and read `stage's water` as a string opener.

**Emptiness and shape assertions, before any golden was written down**, because
four subsystems have now been bitten by silently-empty output that passed every
structural check: all 13 rectangles non-empty; every non-degenerate
amplification non-constant and inside `[0,1]`, the collapsed-region run
constant *and finite*, and the `outW == 1` run entirely NaN; the four
`refineTile` tiles agreeing on their shared edge with delta exactly 0; at least
one straight-branch and one arc-branch label layout (119 arc glyphs across 11
cases); label hit-testing producing hits, misses **and** a topmost-wins
overlap, with all five armed handle kinds reachable; cancel reverting the name
but not the position; 49 brushed icons across ten runs with **two runs
legitimately empty** (a real negative control), every icon in bounds and on
land; and both accepted and rejected click placements. All are re-asserted on
the Rust side.

**The fixture is synthetic, and both sides hash it first.** Unlike milestone D
(which reproduced a real `generate_terrain` world), the height field here is
built from **pure arithmetic** — no `sin`/`cos`/`exp` — so V8's libm and Rust's
cannot disagree about the *input* before the function under test runs. It
carries a deliberately **quantised** `% 11` term (urban M3's lesson) and both
land and water in quantity (370 / 1166 cells), so the brush's sea-level gate is
genuinely exercised. Both sides FNV-1a-64 its raw `f32` bytes and every golden
file asserts that hash before trusting anything else.

**One non-bit-exact result in the whole milestone, measured rather than
assumed.** Case 9 of the arc layout (a 36-glyph label) matches on 106 of its
108 values; two are **one ULP** away, both `dx`, both from `r * sin(theta)` —
`dy` and `rot` agree exactly at those same glyphs, so `theta` itself is
bit-identical and the divergence is purely V8's `Math.sin` against Rust's.
Every other arc case is exact. This is the project's second such allowance
after `CHANGELOG.md`'s `1e-4` for `Math.pow`/`Math.exp`, and far tighter: one
ULP is ~1.4e-16 relative, a sub-picometre glyph offset. It is safe here in a
way it would not have been in milestone D, where an ULP could flip a
`dist < best` segment pick and with it the *sign* of a signed distance —
nothing branches on a glyph position; it goes straight into a canvas transform.
The test pins the exact divergence (every value within 1 ULP, and *exactly two*
values not bit-identical, at exactly those two indices) so it cannot quietly
grow. Everything else in the milestone — heights as raw `f32` bit patterns,
packed bytes, manifest JSON strings, box geometry, handle math, brushed icon
positions and scales — compares **exactly, with no tolerance anywhere**.

### Mutation testing

**89 mutations across the six modules — magic numbers, comparators, orderings,
early exits, sign flips and family switches. 86 killed, 3 survivors, all three
demonstrated equivalent mutants.** 81 were killed by a golden-parity test, 5 by
a unit test.

Both of milestone 3's documented false-survivor traps were guarded explicitly:
every mutation `touch`es its file and the runner **asserts cargo actually
recompiled** the crate (a missing `Compiling <crate>` line is reported as
BROKEN, not as a survivor), and every mutation is applied by a `sed` address
that **skips comment lines**, with a pre-check that the needle occurs on at
least one non-comment line. One mutation was reported BROKEN for a needle that
did not match and was re-run with a corrected pattern rather than dropped.

**The first pass found five real fixture-shape gaps**, and they were fixed by
adding differently-*shaped* fixtures rather than by weakening the mutations:

1. `norm_region`'s `ceil` on the extent survived, because the only fractional
   drag in the set was small enough that the 8-cell minimum masked the
   difference. Added a *larger* fractional drag.
2. `norm_region`'s JS-falsy-zero rule (`minW || 8`) survived, because no
   fixture passed an explicit minimum of 0 or 1. Added both — the pair is what
   pins the rule rather than an off-by-one.
3. `tile_dims`' `aspect >= 1` survived, because at any sane tile size the two
   branches compute the same pair. Added an aspect-1 case with a tile size
   *below* the 2px floor, where they diverge.
4. The `rh > 1` / `rw > 1` degenerate guards survived, because no fixture
   collapsed the region *and* the output together. Added one.
5. The label hit test's `side / 2.0` survived, because the only miss in the
   probe table was far outside every box. Added two probes straddling a box
   edge by one cell.

**And five brush constants survived in a way no golden could have fixed.**
`ICON_BRUSH_MIN_DENSITY`, `ICON_BRUSH_MIN_SPACING`, `ICON_BRUSH_MAX_DARTS`, the
`3.0` spacing constant and the `x 2` dart oversample all reach the same answer
on a small saturated disc. Two structural reasons, both worth recording:

- A dart always lands on an **integer** cell, so two darts can only ever be an
  integer distance apart — and no integer separation lies between 2.9 and 3.0.
  A dart-versus-dart fixture *structurally cannot* see the spacing constant.
  The fixture that can seeds an existing icon at a **fractional** position.
- `max(1.2, 3/sqrt(d))` reaches its floor only above `d = 6.25`, which the
  reference's own 0..1 density slider cannot reach. Inside the shipped
  parameter range that floor is unobservable, so it is driven out of range by a
  direct test instead.

Those five are now killed by scripted-RNG unit tests that observe each constant
on its own (a counted dart budget; an accept/reject at a known fractional
separation) rather than through a statistical outcome. Two further goldens were
added for the same reason — a large zero-density brush and an unsaturated
five-tap drag — and killed the spacing constant and the oversample directly.

**The three survivors, each shown equivalent rather than merely unexplained:**

1. `amplify_region`'s `base < sea` becoming `base <= sea`. At `base == sea` the
   taken branch computes `max(0, (sea-base)/0.06)` = `max(0, 0)` = 0, which is
   the `else` branch's value. The two are the same function.
2. `norm_region`'s `x + w > gw` becoming `>=` (and the `y`/`h` mirror). At
   equality the clamp body computes `x = max(0, gw-w)` = `x` and
   `w = min(w, gw-x)` = `w` — a no-op.
3. Same for the `js_round` half-up rule inside `region.rs`: `tile_dims` is its
   only caller and feeds it strictly positive values, where JS's half-up and
   Rust's half-away-from-zero agree. (The *other* `js_round`, in `manual.rs`,
   does see negatives, and is killed by a scripted-RNG test that lands a dart
   at exactly `-0.5`.)

`cargo build` / `cargo test` / `cargo clippy --all-targets` clean on all six
crates' new code. `cargo test --workspace`: **1034 passing, 0 failures.**
`cargo build --workspace` hit the known `cartalith_godot.dll — Access is
denied` transient (a Godot editor holding the DLL) and was run as
`--exclude cartalith-godot` plus `cargo check -p cartalith-godot`, both clean.

### Not built, deliberately

The tools' **interaction** halves — label drag/rotate/arc capture and the
`prompt` for a new name, the icon gallery arm/disarm and brush on/off, the
measure tool's two-click capture, and the region drag-rectangle with its dashed
overlay and `drawExportTileGrid` — are input routing and belong to milestone F.
So does the `_civLabelPointerHandled` guard that stops a synthesized trailing
click misfiring into "add a new label", and the LOD gate (`!_lodOn`) that
disables label and icon editing while the tiled viewer is on.

Also not built: **milestone E2** in full (the table above), the reference's
`_carDrawMapIcon`/`drawArcLabel` *rendering* (a `cartalith-godot` change this
milestone is scoped out of), `_carIconTypeList`'s glyph fallbacks, the label
and icon **list panels**, and persistence of `state.labels`/`state.mapIcons`
into the save format — `SAVEFILE_COMPAT.md` is read-only in this port and
adding a writer is its own decision.

## Milestone E2 as built (2026-08-18)

The deferred half of Region select/export — *format and pixels*, exactly as
milestone E scoped it — plus `exportGeoJSON` and the non-UI core of
`regionNewWorldBtn`. All golden-verified, all wired to nothing. Same
"primitive ahead of orchestration" precedent as A-E: no Godot scene, `main.gd`,
`main.tscn`, `render.rs` or any `cartalith-godot` file was touched, and the
sibling forks in `cartalith-urban` and `cartalith-civ` were left alone.

Milestone E's assessment — *"a **smaller** E2 than the plan feared"* — held.
Every item on E's deferred list is done, and the only thing that grew was the
verification, not the code.

**Where it landed, and why.**

| piece | home | tests |
|---|---|---|
| `hypso`, `SEA`/`LAND`, `edgeL/R/U/D`, `renderHeightTileRGBA`, `ToUint8Clamp` | `cartalith-terrain/src/tile_render.rs` | 13 unit + 3 golden |
| `_geoXY`, `_geoTraceMaskRings`, `_geoRingArea`, `_geoPointInRing`, `_geoMaskOutlineCoords`, `toFixed` | `cartalith-spatial/src/geo.rs` | 15 unit + 8 golden |
| `gzipBytes`/`gunzipBytes` | `cartalith-io/src/gzip.rs` | 6 unit |
| `zipStore` (generalised) | `cartalith-assets/src/archive.rs` | 5 golden |
| `exportGeoJSON`, `_geoTerritoryFeature`, `_geoProvinceFeature`, the `JSON.stringify` writer | `cartalith-engine/src/geojson.rs` | 9 unit + 2 golden |
| `tilePngBytes`, the gzip/PNG loop, `refineBtn`'s `.zip` assembly, `regionNewWorldBtn`'s core | `cartalith-engine/src/region_export.rs` | 18 unit + 3 golden |

Four placements, each argued from A-E's rule rather than by convenience:

1. **The tile visual is `cartalith-terrain`.** `renderHeightTileRGBA` is a pure
   function of a height tile plus three scalars; its tint is a height ramp and
   its shade is the same normal-from-height formula `shadeFactor` uses. That is
   milestone B's "subsystem-domain math" category and milestone E's own reason
   for putting `amplify_region`/`refine_tile` here — the next step of the same
   pipeline, in the same crate. It touches no canvas and no encoder.
2. **The raster→vector tracer is `cartalith-spatial`.** Every function in it
   operates on *a binary mask over a grid* plus *a km scale* and knows nothing
   about what the mask means — the reference proves the point itself by calling
   one shared `_geoMaskOutlineCoords` from both the territory and the province
   exporter. Same rule that put `norm_region`/`tile_dims`/`measure` there.
3. **gzip is `cartalith-io`**, beside `pack_height16`, which produces the bytes
   being compressed: this crate owns what a Cartalith file looks like on disk.
4. **The two compositions are `cartalith-engine`** — *"it orchestrates; it does
   not compute"*.

### The zip and PNG conventions: reused, and one of them corrected

The brief asked whether the region export shares `cartalith-assets`' archive
conventions. **It does — they are literally the same function.** The reference
has exactly one zip writer, `zipStore` (12009), with three callers: the
asset-pack exporter, the project `.zip` export, and the region export. So
rather than write a second one, `cartalith-assets::archive` grew a neutral
`zip_store`/`zip_store_bytes`, `write_pack_entries` became a one-line alias for
it, and `cartalith-engine` gained a `cartalith-assets` dependency (which it
needed anyway, for `raster::encode_png` — the PNG encoder Phase 4 already built
on the `image` crate with `default-features = false` and `png` only). Both of
milestone 2's recorded conventions carried over unchanged: **`.png` entries are
STORED**, and **every timestamp is frozen at 1980-01-01**.

**One convention milestone 2 had deliberately not ported turned out to be
reachable, and is ported now.** `zipStore` falls back to STORE whenever DEFLATE
does not actually make the entry *smaller*; milestone 2 read that as a
browser-side size concern and skipped it. Running the reference's own
`zipStore` on a four-entry archive shaped like a region export shows **three of
the four entries come back STORED** — the `.png`, a 7-byte `params.json` whose
deflate header costs more than it saves, and an incompressible blob. Only the
height tile deflates. So `deflate_helps` now measures first and chooses second.
`cartalith-assets`' own tests were unaffected (a real `pack.json` still
deflates), and the milestone-2 note in `archive.rs` is updated rather than left
contradicting the code.

**How close the zip bytes get.** Closer than "not comparable". For a STORE-only
archive the two writers produce the **same 172 bytes** apart from two fields no
reader interprets: the version-needed/made-by word (`zip` writes `1.0` for a
stored entry, the reference hardcodes `2.0`) and the external file attributes
(`zip` stamps unix `0644`, the reference writes `0`). The golden normalises
exactly those and then demands every remaining byte match, which is a stronger
claim than a structural walk and would fail loudly if a third difference
appeared.

**What genuinely cannot match, stated once.** Deflated zip entries, gzip
streams and PNG payloads are all produced by `miniz_oxide` here and by the
browser's zlib/PNG encoder there; two conforming encoders need not agree on a
bit stream. So the *decisions* are golden-verified (method per entry, names,
manifest fields), the *pixels* are golden-verified byte for byte before
encoding, and the containers are verified by round trip in both directions.
Reproducibility survives: gzip's MTIME is pinned to `0` and the zip's
timestamps to 1980, so the same export twice is the same bytes.

### What the reference corrected, function by function

**`tilePngBytes` has two renderers, and only one of them is in scope.** It
picks `renderBiomeTileRGBA` over `renderHeightTileRGBA` when
`state.mode === 'biome'`, and that renderer samples the whole climate stack
(temperature, moisture, flow, aspect, curvature, splat, lakes, AO, SVF, cast
shadows) off the coarse grid. That is a Phase 3 rendering concern, not an
export one. The height renderer — the reference's own default *and* its own
fallback — ships here; the biome branch is disclosed as not ported rather than
approximated.

**`Uint8ClampedArray` is not a cast.** `out[p] = c[0]*s` stores a float into a
clamped byte array, and ECMA's `ToUint8Clamp` rounds **ties to even** after
clamping and mapping NaN to `0`. `c[0]*s` is fractional almost everywhere, so
`as u8` (truncation) would be wrong in roughly half of all pixels.

**`hypso` extrapolates past its own palette, into negative channels.** The
depth ramp `d = (sea - v)/sea` is not clamped, so at `sea = 0.3` a `v` of
`-0.1` returns `[-0.67, -10.67, -16.67]`. Verified against the reference, not
inferred, and pinned by a golden. It is harmless only because the clamped store
catches it — which is a second reason `u8_clamped` cannot be shortcut.

**`Number.prototype.toFixed` does not round like Rust.** ECMA picks *"the
larger n"* on a decimal tie; `format!("{:.3}")` picks the even one. That is
reachable rather than theoretical: an 800 km map on a 12 800-cell grid has
`cellKm == 0.0625`, so the first cell's easting is an exact tie at three
decimals — JS says `0.063`, Rust says `0.062`. `js_to_fixed` implements the
spec rule over the exact decimal expansion.

**The tracer's JS `Map` semantics are observable, not incidental.** Ring
discovery follows *insertion* order, and the checkerboard pinch the reference
says it *"doesn't disambiguate"* works by one cell's edge **overwriting**
another's at the same key. What that looks like from outside is an **unclosed
ring** — the walk stops on a mid-ring visited key — and `_geoRingArea`'s
`i < len-1` then silently omits the closing segment. Reproduced exactly,
including the unclosedness, with its own named test.

**`exportGeoJSON` needed its own JSON writer.** `serde_json` renders an
integral `f64` as `16.0` where `JSON.stringify` renders `16` — milestone E's
reason for hand-writing `manifest_json`, and the same reason here, except that
this document is compared to the reference *as a whole string*.
`cartalith-io`'s `js_num`/`json_string` are now `pub` and reused rather than
copied a third time (and `json_string` gained the two short control escapes
`QuoteJSONString` uses, since a place name is arbitrary user text).

**`regionNewWorldBtn` is a UI action with a real computational core.** The
button itself is an interaction, so it belongs to milestone F like every other
tool's interaction half, and E2 leaves it alone. What it *computes* before it
starts mutating anything does not:
`tileDims(sel, 1, 1, ts)` for the new grid, `max(1, mapWidthKm * sel.w / GW)`
against the **old** `GW` for the new scale, and `amplifyRegion` for the field.
That is `extract_region_as_world`. The rest — `allocate()`, `refreshClimate()`,
clearing places/ways/journeys/territory/provinces/labels/icons, the `confirm()`
and the `_setupOpen('calibrate')` handoff — is orchestration over a live world
the shell owns, and is listed in the function's own doc comment rather than
half-built. Two reference notes are kept because they are decisions, not
details: it deliberately does **not** normalise (the data is already meaningful
elevation in the parent's `[0,1]` space), and clearing the civ layer is the
honest answer rather than a subtly-wrong coordinate remap.

### The harness bug that looked exactly like a reference bug

Milestone E disclosed that it never invoked `exportRegionTiles` itself. E2
could: Node has `CompressionStream`, and `tilePngBytes` finds no
`OffscreenCanvas` and returns `null`, which is precisely the headless behaviour
the reference documents. So the real function ran end to end with `wantGzip`
on — and disagreed with milestone E on the **fourth tile only**.

It was the harness. With the DOM stubbed, block #1's boot code schedules a
deferred first `generate()` on a timer, and the reference's `microtask()` is
literally `setTimeout(r, 0)` — which `exportRegionTiles` awaits between tiles.
The boot work fired between tile 3 and tile 4 and overwrote `field` mid-loop.
`amplifyRegion` called twice in a row is bit-identical; the harness was not.
Fixed by making `requestAnimationFrame` inert and draining pending macrotasks
before installing any fixture, after which all four tiles match milestone E's
recorded hashes exactly. That **discharges milestone E's disclosure**: the
assembly matches, not just its four primitives.

Recorded at length because "the reference is non-deterministic" is a conclusion
worth being slow to reach.

### Verification

- **18 golden-parity tests + 61 unit tests**, everything bit-exact with no
  tolerance anywhere: `hypso` compared as raw `f64` bit patterns, six rasters
  as FNV-1a-64 over every byte plus their first and last twelve, both GeoJSON
  documents as whole strings (2136 and 924 characters), and a STORE-only zip
  as bytes.
- **The trig agrees.** `renderHeightTileRGBA` calls `Math.sin`/`Math.cos` on
  the sun azimuth, and the byte-exact match holds across four azimuths (0, 45,
  200, 315) — so this is not one lucky argument. Worth noting given Phase 5
  milestone 5 found `f64::exp` diverging from V8's on 20 721 of 240 000
  arguments.
- `cargo build --workspace`, `cargo test --workspace` (**1150 passing, 0
  failures**) and `cargo clippy --all-targets` on all five touched crates: all
  clean. The `cartalith_godot.dll` access-denied transient did not appear this
  run.

### Mutation testing: 58 mutations, 54 killed, 4 survivors

And, exactly as in milestone E, **the first sweep was the useful one**: it
started at 47/10, and **six of those ten survivors were real fixture gaps, not
equivalent mutants.** Each one is a constant no golden *could* have caught with
the fixtures as first written:

1. **`_geoXY`'s three decimals** — every coordinate in the 12x9 fixture is a
   whole number of kilometres or a clean `.5`, so `toFixed(3)` and `toFixed(2)`
   agree on all of them. Closed with a second fixture at `cellKm = 0.390625`.
2. **The tracer's `ring.length >= 4` filter, in both directions.** Whether a
   length-3 or length-4 ring is even *reachable* was settled by brute force
   rather than argued: all 65 536 masks on a 4x4 grid were run through the
   reference's own tracer, and length-4 rings occur for **1 695** of them and
   length-3 rings (which the filter drops) for **8 760**. Both real, both now
   fixtured from the reference's own examples.
3. **The shell/hole split's `area > 0`.** The same sweep found rings of area
   **exactly zero** — and the reference files them as *holes*. Closed with the
   mask that produces one.
4. **`v < sea` in the shading branch**, because no pixel in six rasters sat
   exactly at sea level. Closed with two more rasters that do.
5. **`strahlerOrder`'s spelling**, because the GeoJSON golden's world traced no
   river. Closed with a second real `exportGeoJSON` run on a 24x18 bowl that
   produces two order-2 channels.

The transferable lesson repeats milestone E's: *a fixture sampled on round
numbers cannot see a rounding rule, and a fixture built from tidy rectangles
cannot see a degenerate-geometry branch.* Where reachability was genuinely in
question, brute force answered it in seconds and beat any amount of reasoning.

The four remaining survivors are equivalent mutants, with the algebra:

1. **`sarea < best_area` → `<=`** (smallest enclosing shell). To differ, two
   *distinct* shells must both contain the hole's first vertex and have equal
   `|area|`. Boundary-traced shells do not cross, so if two both contain a
   point one strictly contains the other — and a strictly-contained ring has
   strictly smaller area. Equal areas therefore force the same ring.
2. **`d < 0.5` → `d <= 0.5`** (the sea ramp split). At `d == 0.5` the first
   branch is `mix(SEA[2], SEA[1], 0.5/0.5) = mix(…, 1) = SEA[1]` and the second
   is `mix(SEA[1], SEA[0], (0.5-0.5)/0.5) = mix(…, 0) = SEA[1]`. Identical for
   every input, not merely for the fixtures.
3. **V8's compensated `Math.hypot` → naive `sqrt(x²+y²+z²)`.** The two differ
   by at most an ULP or two in `il`, which scales a unit normal, then a Lambert
   term, then a colour, and is finally stored through `u8_clamped` — an 8-bit
   quantiser. A ≤2-ULP change can only move the byte if the product lands
   within ~1e-15 of an exact `.5` tie. Milestone B recorded the same survivor
   for the same reason (an `f32` store absorbed it there) and milestone D found
   the fixture that *can* distinguish the two — an unrounded `f64` kilometre
   accumulation — and there is no such accumulation here. The compensated form
   is kept regardless: it is what the reference computes, and the equivalence
   is a property of the output quantiser, not of the function.
4. **`tile_dims(sel, 1, 1, ts)` → `(sel, 2, 2, ts)`.** `tile_dims` reads only
   `aspect = (w/cols)/(h/rows)`; with `cols == rows` the two divisions cancel
   exactly, for any selection. The paired asymmetric control `(2, 1)` **is**
   killed, so the fixture does detect a wrong tile grid — only the exactly
   cancelling mutation survives.

Every survivor was re-run in isolation afterwards, because a stale binary
reports a healthy `N passed`; all four survive on their own too.

### Not built, deliberately

- **The selection interaction** — `regionSel`/`regionDrag`, the dashed overlay,
  `drawExportTileGrid` — is pointer routing and belongs to milestone F, like
  every other tool's interaction half.
- **`renderBiomeTileRGBA`** (the biome branch of `tilePngBytes`), for the
  reason above.
- **`burnChannels`** (10373), which milestone E already placed in neither half:
  it belongs to the LOD viewer, not to this tool.
- **`params.json`'s contents.** `zip_region_export` takes the bytes as a
  parameter because `serializeState()` is a *save writer*, and
  `SAVEFILE_COMPAT.md` is explicitly read-only in this port; adding a writer is
  its own decision.
- **Every UI surface**: `regionNewWorldBtn` itself, `refineBtn`, the download,
  the progress label and the `confirm()` — every tool's interaction half is
  milestone F's, not E2's.

E2 is where the Region select/export tool's engine half ends; **milestone F**,
the shell wiring, is where that engine half and Track 1's shell meet — see
"Milestone F as built" below for how, and how completely: this sentence was
still the last one in the file as of 2026-08-31, which is exactly the
staleness `OUTSTANDING_WORK.md` §1 flagged.

## Milestone F as built (2026-09-01)

No pass wrote this section while the work happened, which is the entire
reason `OUTSTANDING_WORK.md` §1 flagged the gap and this pass exists. The
work itself is not new: it shipped across roughly a dozen commits between
2026-08-18 (`7f5e54c` "Sculpt bindings, the ten-stage pipeline, and the right
dock"; `611c5fa` "Bind the six remaining tool engines to Godot") and
2026-08-25 (`789626d`, a Paint commit/discard gating fix), with the dispatch
substrate and the CARTO/WORLD domain wiring landing 2026-08-19 (`2729734`,
`5bffabd`, `ecd113a`, `82243b8`), a visual sweep on 2026-08-20 (`cd29266`),
and the six-mode Measurement toolbar plus the SG-01/SG-03 staleness restore
on 2026-08-24 (`7ffb59a`, `a0ce1f0`, `1099ca1`, `b5b83f0` — the last
restoring a feature an unrelated commit, `cebd466`, had silently deleted the
same day). `STRANDED_TOOLS.md`'s own 2026-08-19 resolution note caught the
very start of this ("Sculpt now does [have a binding]... which is the
template for the rest") and was never revisited once the rest actually
shipped. This section is that revisit, verified against the working tree
rather than against either document's memory of itself.

**The claim in one line: all sixteen tools `STRANDED_TOOLS.md` catalogued
are closed** — thirteen with a real `cartalith-godot` binding behind a real
shell control, three that correctly need neither — **with one tool the
design added that this port still declines to bind, by a Milestone D
decision that predates this milestone, not an oversight, and one small,
honestly-drawn loose end on Region select's corner handles.** The rest of
this section is the enumeration that backs that sentence.

### The dispatch substrate

One mechanism arms and routes every tool, in `shell/app.gd`: a single
`ButtonGroup` (`tool_group`, `app.gd:84`) shared across every domain's TOOLS
block, an `armed_tool` string defaulting to `"inspect"`, and five
`id -> Callable` dictionaries — `_click_handlers`, `_drag_handlers`,
`_release_handlers`, `_escape_handlers`, `_backspace_handlers`
(`app.gd:92-106`) — that a workspace populates through
`register_tool_click_handler` and its four siblings (`app.gd:137-150`).
`_on_map_clicked`/`_on_map_dragged`/`_on_map_released` (`app.gd:152-162`)
look up whichever entry matches `armed_tool` and call it, or do nothing for
an id nobody registered — which is Select/inspect's entire mechanism: it
registers no handler at all, because `_wire_selection()`'s unconditional
cursor-sample forwarding already is its behaviour, and Pan needs none
either, being "always available as a modifier" on the camera itself
(`global_tools.gd`'s own comment, line 10-12).
`arm_tool()` (`app.gd:108-135`) is the one place a tool becomes active, and it is deliberately
workspace-agnostic — no domain file can see another domain's tool, which is
what let five files (`world_workspace.gd`, `civilization_workspace.gd` for
Settlement/Territory and, since the 2026-08-20 domain merge, Way/Route
alongside it, `infrastructure_workspace.gd`, `cartography_workspace.gd`,
`global_tools.gd`) each register their own tools with no shared switch
statement anywhere.

Counted directly in the tree: **ten tool ids are registered** — `icon`,
`label`, `measure`, `paint`, `region`, `route`, `sculpt`, `settlement`,
`territory`, `way`. That is fewer than the sixteen `STRANDED_TOOLS.md`
catalogued only because five of Sculpt's rows (Raise/lower, Smooth,
Flatten/terrace, Stamp, River/water) share one registry-backed id exactly as
Milestone B's own plan predicted ("splitting them further would just be UI
sequencing, not real engine boundaries"), and because Draw route/way was
always two reference tools (`draw_way`/`route`) and stays split into two ids.

### Tool by tool, against `STRANDED_TOOLS.md`'s own sixteen rows

| # | Tool | Tool id | Arms it | Stores the draft | Renders it |
|---|---|---|---|---|---|
| 1 | Select / inspect (`V`) | *(default; no handler registered)* | `armed_tool`'s own default | n/a | selection is the shell's own — unchanged |
| 2 | Pan (`H`) | *(none; camera modifier)* | always active | n/a | `viewport_host.gd` camera — unchanged |
| 3 | Point sample (`I`) | *(none; a readout)* | n/a | n/a | right dock Sample context — unchanged, still correctly not a tool |
| 4-8 | Raise/lower, Smooth, Flatten/terrace, Stamp, River/water | `sculpt` | `world_workspace.gd:355-358` | `sculpt: Option<SculptEditor>` (`lib.rs:1909`) | `tool_overlay.gd` path preview + brush ring; `build_sculpt_preview_texture` composited live |
| 9 | Biome paint | `paint` | `world_workspace.gd:359-361` | `paint: Option<PaintEditor>` (`lib.rs:1941`) | brush ring; `build_paint_preview_texture` (visibility fixed `1099ca1`; Commit/Discard gating fixed `789626d`) |
| 10 | Place settlement | `settlement` | `civilization_workspace.gd:545` | `civ_tools: Option<CivTools>` (`lib.rs:1930`) | Settlement inspector + map pin |
| 11 | Draw route / way | `way` + `route` | `infrastructure_workspace.gd:266-267` | `infra: Option<InfraTools>` (`lib.rs:1967`) | `tool_overlay.gd` path preview while drafting; Route/Way inspector once committed (`right_dock.gd:332`, `show_route`) |
| 12 | Territory / faction paint | `territory` | `civilization_workspace.gd:546` | `civ_tools` (shared with row 10) | `territory_texture()` wash (`engine_bridge.gd:655`) + brush ring |
| 13 | Label | `label` | `cartography_workspace.gd:556-558` | `labels: Option<LabelBridge>` (`lib.rs:1953`) | `map_overlay.gd::_draw_labels` once placed; `tool_overlay.gd` handles (resize/rotate/arc) while selected |
| 14 | Icon stamp | `icon` | `cartography_workspace.gd:552-554` | `icons: Option<IconEditor>` (`lib.rs:1919`) | `map_overlay.gd` icon draw once placed; `tool_overlay.gd` resize handle (added `a0ce1f0`) |
| 15 | Measure | `measure` | `global_tools.gd:104-107` | `infra` (shared with row 11) | `tool_overlay.gd` ruler / ring / A-B labels; right dock, one context per mode |
| 16 | Region select / export | `region` | `global_tools.gd:110-112` | `infra` (shared with row 11) | `tool_overlay.gd` dashed marquee + corner handles; right dock Region summary (`right_dock.gd:1422-1447`) |

Rows 1-3 are unchanged from `STRANDED_TOOLS.md`'s own read and correctly so
— none of the three ever needed a `cartalith-godot` binding, and none has
grown one. Rows 4-16 are the thirteen that did, and every one now has both
halves: an engine call and a shell control that reaches it.

### Rows 15 and 16 grew past what was proposed

`STRANDED_TOOLS.md`'s own "What I would expect on the UI" table sketched
Measure as one ruler mode and Region select as a marquee feeding an
existing export route. What shipped is larger than either sketch:

- **Measure is six modes**, not one: Distance, Bearing, Area, Radius,
  Cross-section and Δ vertical (`global_tools.gd:50-63`), each reading one
  of eight bound `#[func]`s (`measure_begin`/`add_point`/`result`/`clear`/
  `section`/`area`/`radius`/`vertical`, `engine_bridge.gd:2333-2383`) and
  each rendered by the same `tool_overlay.gd` primitives (a ring for
  Radius, A/B end labels for Cross-section, a closed polygon for Area)
  rather than one drawing routine per mode.
- **Region select is a closed loop, not half of one.** The proposal's own
  gap — "bounds as a typed field, not a marquee dragged on the map" — is
  gone: `global_tools.gd:306-324` drags a rect into `region_set`, the right
  dock's `_build_region` (`right_dock.gd:1422-1447`) reads it back through
  `region_get` and shows extent, cell count and a per-LOD tile estimate
  (the design's own §4.5.1 spec, quoted in that function's doc comment:
  "Extent in both units, cell count, tile estimate per LOD, and Send to
  Data > Export"), and its own `Send to Data ▸ Export` action opens the
  Data manager straight onto the export pane that already called
  `region_export_tiles` (`data_manager_window.gd:1411, 1820, 1831, 1923`).
  Marquee, readout and export route are one path now, not two.

### The one tool the design added that this port still declines to bind: POI

`STRANDED_TOOLS.md`'s own resolution note records that the design revision
added a seventeenth control past its original sixteen: POI (§4.5.3, key
`P`, `_civDropPOI`). It has no row in the table above because it has no
tool id, and that is a decision, not a gap this milestone left open — and
not even this milestone's decision. `cartalith-civ/src/tools.rs`'s doc
comment for `civ_place_pick_weight` says outright, in Milestone D, before
Milestone F existed to wire anything: *"[the reference's] POI branch (a
flat weight of 5) is likewise absent because this port has no POI concept"*
(`tools.rs:137-138`). `cartalith-godot/src/civ_tools_bridge.rs`'s own module
doc — which self-identifies as Milestone F's own work (`civ_tools_bridge.rs:1-2`:
*"the CIVIL tool group's Godot-facing bridge state —
`UNIFIED_TOOL_PLAN.md` milestone F"*) — confirms it rather than papering
over it: *"POI is not a ported concept... `_civDropPOI` has no Rust
counterpart anywhere in the workspace. This module therefore binds
Settlement and Territory only"* (`civ_tools_bridge.rs:20-26`).
`civilization_workspace.gd:531-537` carries the shell-side consequence
forward correctly: no fifth button is drawn next to Settlement/Territory/
Way/Route, because *"[a]rming a button with no engine behind it would be
the fake control this port's own discipline... exists to avoid... omitted
rather than built disabled or wired to a stub."* That is this project's own
standing rule working as intended, not an item for anyone's backlog.

### The staleness readout — SG-01, SG-02, SG-03

Milestone F's other stated half: *"Status-bar staleness readout... reading
Milestone A's per-stage `DirtyTracker`s."* Built, and — per the commit log
— briefly un-built by an unrelated change before being restored the same
day:

- `app.gd:789-796`, `_setup_staleness()`, starts a one-second `Timer` onto
  `refresh_staleness()` rather than wiring a signal into every one of the
  half-dozen `#[func]`s that can dirty something (`app.gd:756-762` gives
  the reasoning: six couplings for a readout that is a plain query).
- `refresh_staleness()` (`app.gd:819-841`) reads `bridge.stale_stages()`
  (`engine_bridge.gd:2042` → `lib.rs:3549`, which reads `self.stages`, a
  `cartalith_spatial::staleness::StageGraph`, `staleness.rs:86` —
  Milestone A's own type, unmodified), names the stale stages and the
  most-upstream reason, and writes the shell's `stale` status slot. This is
  SG-01.
- SG-02 is the civ-side half of the same reading: a sculpt or paint commit
  settles hydrology and climate but leaves civ stale until
  `recompute_civilisation` catches up, which `stale_stages()`'s own doc
  comment (`lib.rs:3540-3547`) names as the one source the graph itself
  cannot represent, reported through a dedicated `civ_dirty` flag instead.
- SG-03 is the per-*parameter* half: moving a generation dial marks the
  correct pipeline stage stale too, not only a tool commit — `set_params`'s
  own doc comment (`lib.rs:2682`) says it "marks the staleness graph... for
  the 25 keys that have a live-apply path," and `params.rs:706` names which
  node of `pipeline_stage_graph()` each one invalidates.
- The explicit "recompute now" affordance F's own definition implies also
  shipped: a `Recompute` button (`app.gd:787, 843-859`) calling
  `recompute_stale_stages()`, visible only when the bound binary actually
  has that method — the same degrade-rather-than-crash discipline every
  wrapper in `engine_bridge.gd` uses.

### One honest residual

Region select's corner handles are drawn — `tool_overlay.gd:221-223`,
*"handles resize it... drawn even though resize-by-drag isn't wired yet, so
the affordance reads correctly once it is"* — and grabbing one does
nothing: no `region_resize` `#[func]` exists anywhere in
`cartalith-godot` (checked; `infra_tools_bridge.rs` and `lib.rs` between
them have `region_set`/`_get`/`_clear`/`_export_tiles` and no fifth verb).
This is not a promise this milestone broke: neither Milestone E's nor E2's
own "as built" sections above ever scoped a resize verb for Region, only
the drag-to-draw marquee and its dashed overlay, which this milestone did
build. Label and Icon, by contrast, both got a real resize verb
(`label_resize_size`, `icon_resize`) because their own reference precedent
had one. Redrawing a region today means dragging a fresh marquee from
empty, which reaches every other step of the loop correctly — only the
handle-grab shortcut the overlay's own comment anticipates is unbuilt.
Small, and exactly as far as "genuinely incomplete" goes for this milestone.

### Verified

Not re-run this pass — this is a documentation closeout, not a code change,
so nothing here required a rebuild — but counted directly against the
working tree: **131 `#[test]` functions** across the six bridge files this
milestone's tools are bound through (`sculpt_bridge.rs` 12, `paint_bridge.rs`
25, `label_bridge.rs` 23, `icon_bridge.rs` 28, `infra_tools_bridge.rs` 22,
`civ_tools_bridge.rs` 21), on top of the 29 in `cartalith-civ/src/tools.rs`
and 26 in `cartalith-spatial/src/pass.rs` that Milestones A and D's own
sections above already counted for the pure-engine halves. The `#[func]`
wrapper counts cited above are a direct count of `engine_bridge.gd`'s own
per-source-file blocks between lines 1844 and 2506, each a 1:1 wrapper over
one bound `#[func]` by the file's own stated contract (`:1350-1366`): 34 for
Sculpt (`:1367-1571`, matching `STRANDED_TOOLS.md`'s own 2026-08-19 count of
Sculpt's binding exactly), 13 for Icon (`:1844-1922`), 10 for Paint
(`:2201-2258`), 4 for Way (`:2259-2281`) and 8 for Route (`:2282-2331`), 8
for Measure (`:2332-2383`), 4 for Region (`:2384-2405`), 17 for Label
(`:2414-2506`).

### Not built, deliberately — carried forward, not new to this pass

Everything Milestones D, E and E2 already named as deferred to F and listed
under their own "Not built, deliberately" headings above is now either
built (the interaction halves: waypoint capture, the Escape/commit
keybindings, the drag-rectangle with its dashed overlay, the two-click
measure capture, label drag/rotate/arc, icon arm/disarm/resize) or still
correctly absent for the reason those sections already gave: `params.json`'s
contents (needs a save *writer*, and `SAVEFILE_COMPAT.md` is read-only by
design), persistence of labels/icons into the save format (same reason),
and `_civGenerateProvinces` consuming a painted territory raster (Milestone
D's own item, unrelated to shell wiring). None of these are Milestone F's
to close, and this pass did not touch any of them.
