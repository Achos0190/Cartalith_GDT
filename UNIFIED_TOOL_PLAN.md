# Unified tool plan: what a tool *is*

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
`cartalith-engine`). DONE 2026-08-18 — see "Milestone A as built" below.**
The new `PassBuffer<Stamp>` type from "The shared
editing model" above: stamp storage, touched-tile tracking, preview-via-
scratch-composite, commit-via-real-write, discard. Per-stage `DirtyTracker`
instances wired along the real dependency chain (height → hydrology →
climate → civ), each doing lazy version comparison against its upstream,
no eager cascading. No UI. Verifiable headlessly: commit/discard round-trip
tests, staleness-propagation tests against a small synthetic field. This
is the one milestone every other milestone depends on.

**Milestone B — Terrain group, the Sculpt-editor port. DONE 2026-08-18 —
see "Milestone B as built" below.** The largest
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

**Milestone E — Annotation & measure group.** Label's new struct + arc-text
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
primitive; a touched-region-only refresh is left to the caller, since no
renderer is wired yet to say what shape it wants.

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
