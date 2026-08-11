# Sculpt Editor Integration Plan

## Goal

Promote today's hand-drawn terrain tools — currently a "Manual Terrain" accordion buried in
Generate → World, sitting alongside "Geology," "Hydrology," "Climate," "Ecology" — into a
full-fledged, stamp-based, non-destructive landscape editor, by porting the architecture proven
in `fractal-geology/Fractal Geology Painter v0.1.html` (11 feature brushes, per-feature fractal
edge character, a stamp stack, a cheap live-preview compositor) into Cartalith's engine (script
block 1), while satisfying three hard requirements from the brief:

1. **Full, rich menu** — not one accordion among several, its own editor surface.
2. **Zoom and brush size stay visually and behaviorally consistent** — the brush footprint must
   read as the same real-world size at any zoom level, in every view mode (including LOD).
3. **Painted geology feeds the resource/affordance system (lithology → soil → water access →
   resource potentials → carrying capacity → settlement suitability) — but only on commit.**
   While painting/tuning, nothing downstream recomputes; the expensive full-grid pipeline runs
   once, deliberately, when the user says so.

This document is a plan, not a diff. Two decisions below needed sign-off before any of the
phased work (P0+) starts — **both are now decided** (see "Design decision 1/2" below and the
summary at the end): a 4th Generate sub-tab, and a session-scoped commit whose final step is a
full re-render so the visible map/debug view never goes stale after committing.

## Current state (verified in `Cartalith Gen1 v1.06.html`, not assumed)

**It isn't literally under "Geology."** The accordion is titled **"Manual Terrain"** (line 1005),
a *sibling* of the "Geology" accordion (line 762, which actually hosts Source/Planet/Scale/World
Structure/Tectonics/Volcanism) — both live inside `#genWorld` (Generate → World). Functionally
your instinct is right either way: it's one collapsible section among many, not its own surface.

**Two independent, overlapping brush systems exist today, both destructive:**

| | Plotline feature brush | Direct-paint brush |
|---|---|---|
| Entry point | `applyFeatureAlongCurve()` (line 6775) | `sculpt()` (~6698) / `brushHeight()` (line 8356) |
| Features/modes | 7: mountainRange, hills, ridge, plateau, escarpment, canyon, river | 9: raise, lower, smooth, cliff, ridge, canyon, mesa, volcano, water |
| Interaction | draw a guide stroke (RDP-simplified, Catmull-Rom smoothed), then **Apply** | live drag-paint |
| Writes to | `field` directly, in place | `field` directly, in place (except `water` — see below) |
| Post-stroke recompute | **Full-map** `computeFlow(true); refreshClimate()` (line 10160–10162) | `endPaintStroke()` → `recomputeClimate()` → **temperature only** (line 4260) |
| Undo | shared 5-slot full-`Float32Array` snapshot stack (`pushUndo`/`undoStack`, line 7028) | same |

The asymmetry in the middle row is real and already inconsistent: painting a mountain range with
the guide tool already triggers a full climate+flow recompute per stroke (expensive, but at least
correct); direct-paint sculpting does not touch rain/flow/lithology/resources **at all** until
something else happens to call `computeFlow()`. Neither is "on commit" — both are "on every
stroke, differently."

**The affordance/resource stack already exists and is exactly the right commit hook.**
`buildLithology → buildSoilFertility / buildWaterAccess → buildResourcePotentials →
buildCarryingCapacity → buildSettlementSuitability` (lines 4700–4895) are pure functions, lazily
cached (`_lithField/_soilField/_waterField/_resourcePots/_carryCapField/_settleSuitField`), and —
confirmed by direct inspection — **correctly invalidated by `generate()` (2646) and
`computeFlow()` (4031)**. So "recompute resources on commit" is not new machinery: it's *already
what happens* whenever `computeFlow(true)` runs. The gap is only that direct-paint sculpting never
calls it, and the guide-brush calls it on every single stroke instead of once at commit.

**No sculpt-overlay/delta layer exists**, despite `docs/UNIFIED_TOOL_PLAN.md` describing one
(`sculpt_delta.f32`, a `[Bake & regenerate] / [Regenerate, keep overlays] / [Cancel]` dialog). That
document predates the actual merge architecture (4 sequential script blocks, not a `Gen.*`
namespace) and was superseded. Today, sculpting a mountain by hand and then clicking "Generate
world" silently discards it — `_imported` (the only guard on `confirmRegenerate()`, line 10007)
is never set by sculpting. This plan's Draft/Committed split (below) is the first real
implementation of that superseded idea, built to match what actually shipped.

**Water already has a non-destructive precedent worth reusing, not reinventing.**
`depositWater(gx,gy)` (line 6656) — the existing "Water" direct-paint mode — floods the brush
radius up to the clicked terrain height into `lakeMask` (a `Uint8Array`, line 4682), **without
touching `field` or sea level**, and is explicitly documented as "cleared on the next generate()."
`buildWaterBodies`'s `forceLake` option (line 4673) makes `lakeMask` cells always classify as
lake regardless of connectivity. Separately, the existing plotline `river` feature already locks
carved cells into `riverMask`/`riverFloor` (line 10158) to protect them from erosion refill. **Both
of these are exactly the mechanisms the new Lake and River stamps should extend**, not replace —
see §5.

**Brush-size-vs-zoom already works for the common case, with one real gap.** Off-LOD, the brush
cursor (`renderBrushCursor`, line 7104) is drawn onto `#polyOverlay`, a sibling canvas inside the
same `.canvas-stack` that `viewT`'s CSS `transform: translate()/scale()` applies to — so the
cursor **already** visually scales with zoom, free, no extra code. In LOD (tiled) mode, though,
`renderBrushCursor` **bails out entirely** (`if(_lodOn ...) return;`, line 7107) — no cursor is
drawn at all — even though the underlying edit machinery (`editTileAt`, line 8397) already
rescales the *effect* radius correctly per tile resolution. That's a real, fixable gap, not a
design choice.

## Design decision 1 — where does the new editor live? **DECIDED: a 4th Generate sub-tab.**

Alongside World / Civilization / Cartography —
`data-gsub="sculpt"`, label "Sculpt" (reviving the name the pre-merge standalone engine's own
history used for this before two IA passes folded it into an accordion).

The alternative — a new top-level tab alongside Generate/Explore — is mechanically no harder
(both are additive edits to `#tabBar`/`#genSubBar` and their click handlers), but
`tests/perf/smoke_gen1.js` line 1562 asserts `tabCount === 2 && tabsOnly2 === true`, and CLAUDE.md
+ the v0.65 CHANGELOG entry both frame the 2-position top bar as a deliberate, hard-won IA
decision (Assets/Export were *demoted out* of being tabs specifically to preserve it). A 4th
Generate sub-tab is additive to an array-based assertion (`subTabs`, smoke_gen1.js line 426) and
doesn't touch that invariant, while still delivering everything "full-fledged" actually requires:
its own toolbar, its own canvas-interaction mode, its own inspector panel — a genuine sibling of
World/Civilization/Cartography, not a `<details>` accordion buried inside one of them.

`_manualTerrainActive()` (line 6737, currently gated to `_genSubTab==='world'`) is replaced by an
equivalent `_sculptEditorActive()` gated to `_genSubTab==='sculpt'`.

## Design decision 2 — what does "commit" actually do to history? **DECIDED: A, with a condition.**

**A. Session-scoped stamp stack.** The stamp stack (per §3) is a cheap, fully non-destructive
*staging area* for the current sculpting session — unlimited undo/redo, reorder, re-tune, hide,
delete, all essentially free because it's JSON snapshots of a lightweight object list, not the
heightmap. **Commit** bakes the whole stack into `field` in one pass, runs `computeFlow(true);
refreshClimate()` once, pushes **one** snapshot onto the existing `pushUndo()`/`undoStack`, and
clears the stamp stack. A single Ctrl+Z after commit reverts the whole batch cleanly (consistent
with how every other field-mutating action in the engine — Generate, erosion passes, the plotline
brush — already uses `pushUndo`). Once committed, a stamp is no longer independently editable;
adjusting a committed mountain range means painting a new stamp on top of it, the same way a
second erosion pass builds on the first rather than reopening it.

(Option B — fully persistent stamp history matching the PoC exactly, where `field` is always "base
+ full replay of every stamp ever placed" — was rejected: it doesn't scale to Cartalith's grid
sizes, and `field` is *also* mutated by things that aren't stamps at all (tectonics, erosion,
imported heightmaps), so "field is a pure function of the stamp stack" stops being true the moment
any of those run.)

**Condition (user-specified): commit must end with the underlying layers actually rendered, not
just recomputed in the background.** Concretely, the commit sequence's last step is a full
`renderNow()` (the same call the existing plotline Apply handler already makes after its own
`computeFlow(true); refreshClimate()`, line 10162 — not new machinery, just making sure the new
Commit path does it too). This guarantees:
- The main terrain view redraws with the committed geometry/materials immediately — no stale
  frame between "I clicked Commit" and "the map looks right."
- **If the user is currently on one of the affordance/resource debug views** (Lith / Soil / Water
  / Resources / Carry Cap / Settlement) **when they commit, that view redraws with post-commit
  data as part of the same render** — because `renderNow()` is exactly what triggers each view's
  lazy `current*()` builder (§ "Resource/affordance system"), and those builders now recompute
  from the freshly-committed `field`/`rainField`/`tempField`/`flowField` since `computeFlow()`
  already nulled their caches. No second action, no manual refresh, no toggling the view off and
  back on to force a rebuild.
- Views the user is *not* currently looking at stay lazily built (rebuilt whenever next opened),
  per the engine's existing performance-conscious design (CHANGELOG frames these as deliberately
  "debug-view + export only... built lazily," e.g. `docs/AFFORDANCE_FIELD_PLAN.md`'s whole
  premise) — this condition is a correctness guarantee for whatever is currently rendered, not a
  mandate to eagerly rebuild every possible layer regardless of visibility, which would waste
  cycles for no visible benefit.
- This becomes a named acceptance test in §10 (Verification), not just an implied side effect.

This reuses the existing undo idiom instead of inventing project-lifetime stamp persistence, and
matches the brief's "updating to layers only happens when committing the terrain" literally: draft
work is free, reversible, and invisible to every downstream system; committing is the one
deliberate, coarser step — after which the user immediately *sees* the result, exactly like every
other terrain-shaping stage in the engine already behaves.

## 3. Layer model

```
┌─────────────────────────────────────────────────────────────────────┐
│  DRAFT (new)                                                        │
│  sculptStamps[]  — { id, type, seed, g:{...}, f:{...}, pts, hidden }│
│  previewField    — Float32Array, = committed field + Σ(visible      │
│                     stamps' deltas), rebuilt ONLY within each       │
│                     stamp's bbox (the PoC's dirty-rect compositor)  │
│  stampHistory    — JSON snapshots of sculptStamps[] (cheap, deep    │
│                     undo/redo while painting/tuning)                │
│  Nothing here touches `field`, `rainField`, `tempField`, lithology, │
│  resources, or triggers any recompute.                              │
└─────────────────────────────────────────────────────────┬───────────┘
                                                            │ COMMIT
                                                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│  COMMITTED (existing engine state)                                  │
│  field — bake every stamp in stack order (ported applyStamp)        │
│  computeFlow(true)  → clears _riverNet/_waterBody/_cartBiome/…       │
│  refreshClimate()   → temperature + weather + moisture + currents   │
│  (this ALREADY nulls _lithField/_soilField/_waterField/_resourcePots│
│   /_carryCapField/_settleSuitField — confirmed at lines 2646, 4031) │
│  pushUndo() — ONE snapshot for the whole batch                      │
│  renderNow() — the visible map AND any active affordance/resource   │
│    debug view redraw with post-commit data, same call, same frame   │
│  sculptStamps[] cleared; stampHistory cleared                       │
└───────────────────────────────────────────────────────────────────┘
```

The commit step is small precisely because it reuses existing, already-correct invalidation —
the plan's job is making sure it *always* runs (once, on commit), not building new invalidation.
The final `renderNow()` is not optional polish — it's the mechanism that satisfies "layers get
rendered on commit," and it's the same call the existing plotline Apply handler already makes.

## 4. Feature registry — consolidating three overlapping lists into one

| Existing plotline (7) | Existing direct-paint (9) | PoC (11) | Unified registry entry |
|---|---|---|---|
| mountainRange | — | mountains | **Mountains** (ridged multifractal, PoC's peak-sharpness/ruggedness controls) |
| hills | — | hills | **Hills** (smooth FBM, PoC's softness control) |
| ridge | ridge | — | **Ridge** (keep as its own entry — a linear crest, distinct from a mountain mass) |
| plateau | mesa | plateau | **Plateau** (PoC's terraced FBM, `Math.max` never-lowers semantics kept) |
| escarpment | cliff | cliff | **Cliff / Escarpment** (merge — both are a side-signed step across the stroke normal; PoC's low-frequency "fault trace" edge warp is a strict improvement on both) |
| canyon | canyon | canyon | **Canyon** |
| — | — | valley | **Valley** (new — PoC's broad U-shaped trough, distinct from Canyon) |
| river | — | river | **River** — commit locks carved cells into **existing** `riverMask`/`riverFloor` (line 10158 precedent), reusing `enforceChannelDescent` |
| — | — | lake | **Lake** — commit extends **existing** `lakeMask`/`depositWater`/`forceLake` (line 6656 precedent) with the PoC's radial bowl + fractal shoreline, instead of a flat flood |
| — | — | basin | **Basin** (new — endorheic sink, no outlet, distinct from Lake which always fills to a level) |
| — | — | coastline | **Coastline** (new) |
| — | volcano | volcano | **Volcano** (PoC's cone+crater+flank-rough, richer than the existing conical stamp) |
| — | raise/lower/smooth | — | **Freehand** (kept as its own registry entry, not retired — a genuinely different interaction: continuous drag, no preset landform, for quick touch-ups. `mesa`/`volcano`'s STAMP_BRUSHES one-dab-per-tap behavior and `cliff`/`ridge`/`canyon`'s STROKE_BRUSHES direction-from-drag behavior fold into Freehand's sub-modes.) |

Net: **13 registry entries**, each with the PoC's per-feature `controls[]` (dynamic parameter
panel) and per-feature `edgeChar`/`edgeFreqMul` fractal edge character (§6), replacing both the
7-feature plotline set and the 9-mode direct-paint set. All 8 PoC presets (Rolling Hills, Alps,
Rockies, Badlands, Volcanic Isle, Mesa, Karst, Glacial Valley) port as one-click parameter seeds.

**Not retired, kept as a fallback:** the existing plotline/direct-paint code paths stay reachable
only if something outside the new tab still depends on them (to be confirmed in P0 — a quick
grep found no other call sites besides the UI handlers themselves, so full retirement is likely
clean, but this gets verified before deleting anything, not assumed).

## 5. Zoom / pointer sizing

1. **Off-LOD:** already correct (CSS-transform sibling-canvas trick) — no change needed.
2. **LOD mode:** replace `renderBrushCursor`'s early bail (line 7107) with an LOD-aware draw that
   converts the world-space radius to the current tile's screen scale using the **same math
   `editTileAt` already uses** (`state.radius * p.td.w / p.b.w`, line 8397) — so the visual
   cursor and the actual applied effect finally agree in LOD mode, matching what already works
   off-LOD.
3. **Live real-world readout:** next to the brush-size slider, show `radius × cellKm` (reusing the
   `cellKm = state.mapWidthKm / GW` idiom already used ad hoc at ~30 call sites) so "relative
   size" isn't just visual — the user sees the actual km footprint at any zoom.
4. **Pointer→grid conversion:** audit all painting pointer handlers to route through the existing
   `evtToGridLOD()` (line 7057) uniformly, so painting behaves identically whether the user is
   zoomed via `viewT.scale` or via LOD's own `_lodZoom`. Today the direct-paint and guide-stroke
   handlers call `evtToGrid` at a few sites that predate `evtToGridLOD` — P0 confirms and fixes
   any that aren't LOD-safe yet.

## 6. Fractal edge character (ported from the PoC, §"per-feature edge character")

Each registry entry keeps its own `edgeChar`/`edgeFreqMul` multiplier on the domain-warped mask
boundary (coastlines and lakes ragged and low-frequency; mountain ridgelines tight and
high-frequency; rivers and valleys comparatively clean since their shape already comes from the
meander parameter; Cliff/Escarpment wanders like a fault trace). This is a straight port of the
PoC's `stampBBox`/`applyStamp` warp logic (`fractal-geology/Fractal Geology Painter v0.1.html`,
the `edgeChar`/`edgeFreqMul`/mottled-transition-band code) — see that file's README for the
worked rationale per landform. One adaptation: Cartalith's `field` uses `state.seaLevel` (not a
flat 0.42) and needs the noise-scale/octave/persistence/lacunarity parameters to reuse the
engine's own `fbm`/`ridged` primitives rather than the PoC's standalone `makeNoise`, so seeds stay
consistent with the rest of the terrain (no second unrelated noise library).

**Open item for P0:** does the existing curve-distance code (`applyFeatureAlongCurve`'s nearest-
point search) already account for world-mode equirectangular wraparound (painting a feature that
crosses the antimeridian)? Not verified yet — needs a direct check before the ported compositor
assumes an answer either way, since world mode wrap (CLAUDE.md invariant 9) is a hard constraint
elsewhere in the engine.

## 7. Water — extend existing mechanisms, don't add a parallel one

**Hard requirement (owner, 2026-07-19): river and lake water render with the same treatment as
sea water, with one exception — standing water in a lake or river is not required to sit at
`state.seaLevel`, exactly like real hydrology.** This turned out to already be the engine's design
principle since v0.103, not something to newly build — the job here is wiring painted stamps into
the existing correct system, not inventing elevation-aware water rendering:

- **Lakes already render above sea level, and already reuse the sea's own color function.**
  `buildWaterBodies` (line 4636) classifies every cell `0`=land / `1`=ocean / `2`=lake, and does so
  three ways: (a) below-sea depressions not connected to the largest (ocean) component → inland-sea
  lake; (b) **a priority-flood fill auto-detects natural above-sea depression lakes** — any pooled,
  well-watered depression (`depth>lakeDepth && rain[i]>=lakeRain`) becomes class 2 with zero user
  input, a real hydrological simulation already running today; (c) `opts.forceLake` (fed by
  `lakeMask`, the existing "Water" brush's deposit) always overrides to class 2, for a user-placed
  lake regardless of what the natural test would say. The renderer (`renderNow`, line 6499) checks
  this **before** the normal sea/land split: `if(_lakeWB[i]===2 && vw>=state.seaLevel){ …
  lakeColor(x,y,i) … }` — and `lakeColor()` (line 6253) **literally calls `seaColorCore()`** (the
  same function `seaColor()` itself calls), just at a fixed shallow depth and shifted slightly
  brighter/greener — i.e. "the same treatment as the sea," by construction, not by convention. A
  lake that dips below `state.seaLevel` doesn't need this branch at all — it already passes the
  ordinary `isWater()` test and renders via `seaColor()` directly.
  **Lake stamps, on commit:** carve the radial bowl (fractal shoreline warp per §6) into `field`,
  then deposit the affected cells into `lakeMask` (the same array `depositWater()`, line 6656,
  already writes) so `buildWaterBodies`'s `forceLake` path always classifies them as lake — whether
  the bowl's floor ends up above or below `state.seaLevel`. No new field; reuses the existing
  invalidation already correctly wired at every `lakeMask`-writing site.
- **Rivers already render at any elevation, by construction — a mountain stream costs nothing
  extra.** The river overlay (`surfaceColor`, line ~6151) blends `waterShade(bed, depth, sed, Kd)`
  (line 6286, a Beer–Lambert bed-transmission model — the same physically-motivated "semi-
  transparent water over a surface" family as the sea/lake shaders, just modeling depth-visibility
  instead of a fixed palette) over the land color wherever `_riverNet.intensity[i]>0` — purely a
  function of the traced drainage network, never of absolute height versus `state.seaLevel`. **River
  stamps, on commit:** call the existing `enforceChannelDescent` + lock carved cells into
  `riverMask`/`riverFloor` — literally the same call the plotline river feature already makes (line
  10158) — then let `computeFlow(true)` retrace `_riverNet` through them, same as today. No new
  field, no elevation handling to add: the network overlay already doesn't care.
- **Basin/Coastline/Canyon/Valley** don't need a water flag at all — they're pure elevation
  shaping; whether a cell ends up underwater falls out of `isWater(v) = v<state.seaLevel` (line
  6342) or the natural above-sea lake detection above, same as any other terrain.

## 8. UI / menu content (the "full and rich" ask)

New `#genSculpt` panel (sibling of `#genWorld`/`#genCiv`/`#genCarto`), built from Cartalith's
existing idioms — not the PoC's raw HTML/CSS, so it matches the rest of the app:

- **Feature palette** — `.seg`-style button grid (13 entries, icon+label), built the same way
  `seg('brushSeg','brush','brush')` (line 9995) already wires buttons to state today.
- **Presets row** — 8 buttons, one-click parameter seed (port PoC's `PRESETS` table).
- **Global brush/noise** accordion — brush size (+ live km readout), hardness, intensity, noise
  scale, octaves, persistence, lacunarity, **edge noise** (new), seed + 🎲, using the engine's
  existing `bind()`/`v()`/`lab()` slider-wiring helpers (CLAUDE.md invariant 7).
- **Per-feature dynamic controls** — built from each registry entry's `controls[]`, same pattern
  as the PoC's `buildFeatureControls()`.
- **Stamp stack panel** — list of this-session stamps: select / reorder / hide / delete, port of
  the PoC's `#stampList`.
- **Commit / Discard draft** buttons (replacing today's implicit "it's already been written to
  `field`" model) — Commit runs §3's bake sequence; Discard clears the draft stack with no field
  changes at all.
- **Draft-vs-committed visual distinction** — since Cartalith's main view is already full-color
  terrain (unlike the PoC's grayscale-first default), add a subtle staged-edit indicator (outline
  or hatch over cells with a pending, uncommitted delta) rather than porting the PoC's 5 view-mode
  buttons, which would duplicate views the main app already has (biome/relief/debug).

## 9. Phased implementation

- **P0 — SHIPPED, `Cartalith Gen1 v1.07.html`.** Per CLAUDE.md's "new version = new file, never
  edit old versions in place" rule, this landed as a new version, not an in-place edit of v1.06.
  Ported into script block 1 (right after `catmullRomSample`, before
  `drawRiverWays` — the file's existing "pure curve/geometry primitives" neighborhood): 11 features
  (`SCULPT_FEATURES`), the dirty-rect compositor (`sculptApplyStamp`/`sculptStampBBox`), stroke
  geometry (`sculptNearestOnStroke`), and three new parametrized noise wrappers
  (`sculptFbm`/`sculptRidged`/`sculptBillow`). Two decisions made *during* porting, worth recording:
  - **Naming: flat, prefixed top-level functions/consts** (`SCULPT_FEATURES`, `sculptApplyStamp`,
    …), matching this codebase's actual convention for registry-style globals in script block 1
    (`CIV_FACTIONS`, `BIOME_KEYS`, `buildResourcePotentials`) — **not** the block-4 IIFE-namespace
    pattern (`UME`), which exists specifically for block 4's cross-block isolation, a concern that
    doesn't apply here (a direct collision check against all PoC-side names came back clean).
  - **Noise: new parametrized wrappers on `vnoise()`, not a reuse of the existing `fbm()`/
    `ridged()`.** The plan originally said "reuse the engine's own primitives" — turned out to be
    half right: `fbm()`/`ridged()` hardcode 6 octaves/0.5 persistence/2.0 lacunarity (only
    `ridgedFbm()` parametrizes octave count), but every sculpt feature needs all three as
    independent sliders. `sculptFbm`/`sculptRidged`/`sculptBillow` are built on the engine's own
    `vnoise()`/`hash()` (so painted terrain still shares its base noise primitive with tectonics/
    erosion) but are new functions, and deliberately keep the PoC's own range convention
    (`sculptFbm` ~[-1,1], not `fbm()`'s [0,1]) because every ported feature formula was tuned
    against that range — changing it would silently alter results already visually verified in the
    PoC. `smoothstep(a,b,x)`/`lerp`/`clamp01` (this file's own existing globals) ARE reused as-is.
  - **World-wrap open item — resolved.** Read `applyFeatureAlongCurve` in full: it has no
    equirectangular seam-wrap handling at all (plain 2D Euclidean bboxes/distances, no `state.world`
    branch). `sculptApplyStamp` matches that — parity, not a regression. A future pass could add
    wraparound to both uniformly; out of scope here.
  - Verified: `tests/run.sh` — **1013 passed, 0 failed** (923 pre-existing + 90 new, covering noise
    range/determinism, all 11 features' finiteness/`[0,1]`-bounds/mask-locality/bit-reproducibility/
    raise-lower-water semantics, geometry, per-feature edge character, and the world-wrap parity
    check above). `tests/run_um.sh` **831/831** (block 4 untouched). `node tests/perf/hash_gen1.js
    "Cartalith Gen1 v1.06.html" "Cartalith Gen1 v1.07.html"` — **ALL IDENTICAL** across the
    default/geoid/waves/ao/icons config matrix, proving cross-version neutrality (nothing wired to
    any UI yet, so this had to be a no-op — confirmed, not assumed). `smoke_gen1.js` **179/179**,
    unchanged.
- **P1 — Tab shell.** `#genSculpt` + `data-gsub="sculpt"`, wired into `_genSubTab` show/hide, empty
  panel. Confirm `tests/perf/smoke_gen1.js`'s 2-tab-bar assertion is untouched and its `subTabs`
  array assertion is additive. No functionality yet.
- **P2 — Draft layer.** Stamp stack + `previewField` compositor + feature palette + presets +
  per-feature controls + stamp-stack panel + cheap stamp-history undo/redo. `field` is never
  touched. `_sculptEditorActive()` gates pointer handling exactly like `_manualTerrainActive()`
  does today.
- **P3 — Commit path.** Bake stack → `field`; `computeFlow(true); refreshClimate()`; one
  `pushUndo()`; **`renderNow()`**; clear draft. Acceptance test: with a Resources/Carry Cap/
  Settlement debug view already active, paint a mountain range and Commit — copper/iron potential
  and settlement suitability must visibly update in that same view, same frame, no manual
  refresh. This is the acceptance test for both "feeds resources on commit" and the "layers get
  rendered on commit" condition together — they're one requirement, not two.
- **P4 — Zoom/pointer polish.** Fix `renderBrushCursor`'s LOD bail; live km readout;
  `evtToGridLOD` audit.
- **P5 — Water integration.** River→`riverMask`/`riverFloor`; Lake→enriched `lakeMask`/
  `depositWater`.
- **P6 — Deprecate the old accordion.** Remove "Manual Terrain," `applyFeatureAlongCurve` call
  sites, old `brushHeight`/`sculpt()` mode chain — *after* confirming (P0's grep) nothing else
  depends on them. Freehand raise/lower/smooth survives as a registry entry (§4), not a
  regression.
- **P7 — Docs/tests.** CHANGELOG entry, ROADMAP update, `tests/run.sh` assertion additions,
  `tests/perf/smoke_gen1.js` additions for the new tab + commit flow.

## 10. Verification

- Headless: extend `tests/run.sh`'s suite with the ported feature registry (determinism,
  `[0,1]` bounds, mask locality, raise/lower/water semantics — the PoC's own corpus is the
  starting point) plus new assertions for the commit sequence (`computeFlow`/`refreshClimate`
  actually called once per commit, never mid-draft; affordance caches null after commit, non-null
  and unchanged during draft).
- Browser (`tests/perf/smoke_gen1.js` additions): tab mechanics, paint → draft (verify `field`
  unchanged, and that an already-open Resources/Carry Cap/Settlement debug view does NOT change
  during draft) → commit (verify `field` changed, `renderNow()` ran, and — with that debug view
  still open from before the commit — its rendered pixels/legend values changed in the same pass)
  → undo (verify one Ctrl+Z fully reverts both `field` and the debug view's rendered data) →
  LOD-mode cursor visibility → zoom-relative brush-circle size at a few `viewT.scale`/LOD-zoom
  levels.
- Cross-version neutrality (CLAUDE.md's standing rule): with the new tab never opened / no stamps
  committed, `generate()` and rendering stay bit-identical to v1.06 at defaults — same FNV-hash
  discipline as every other opt-in feature in this codebase.

## Decisions (locked)

1. **Tab placement — DECIDED: new Generate sub-tab "Sculpt"** (4th branch alongside World /
   Civilization / Cartography). Preserves the tested 2-position top-level phase-switch invariant.
2. **Commit model — DECIDED: session-scoped stamp stack**, condition attached: commit's last step
   is a full `renderNow()`, so the visible map and any currently-open affordance/resource debug
   view show post-commit data in the same render pass — never merely "correct next time it's
   opened." Folded into §3's layer-model diagram, P3's acceptance test, and §10's verification
   plan above.
3. **Still open, lower stakes, deferred to P6 itself:** whether to fully retire the
   plotline/direct-paint code paths outright, or keep them dormant behind a flag for one release
   as a safety net. Recommendation stays full retirement once P0's dependency check comes back
   clean — revisit only if that check finds a surprise dependency.

P0 (port the pure core, headless-test it in isolation, no UI wiring) is next.
