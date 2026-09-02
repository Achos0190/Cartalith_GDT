# Decision log

Every major choice, what it beat, and why. Read the reasoning before revisiting
one — each was narrowed from real alternatives, not taken by default.

| # | Decision | Beat |
|---|---|---|
| 1 | Native rewrite | wrapping the HTML app (Tauri/Electron/Capacitor) |
| 2 | Rust engine | C/C++ |
| 3 | Godot shell via `gdext` | pure Rust (bevy, wgpu+winit) |
| 4 | 2D only for v1 | 2D and 3D together |
| 5 | Cloud session builds | local PC, or a hybrid |
| 6 | Personal/hobby bar | store distribution |
| 7 | Golden-value parity | independent correctness |
| 8 | Code in a new repository | code beside these docs |
| 9 | Godot version pinned at setup | a version pinned here |

## 1. Native rewrite, not a wrapped web app

Wrapping would cost almost nothing and reuse the whole tested engine. It was
rejected on the stated goal, not on merit: a wrapper still runs the same
Canvas2D/WebGL JS inside a webview, which is an installable icon rather than
different performance — and Android WebView GPU and touch behaviour is a known
source of quirks this project already works around.

If the goal were "ship an exe and apk quickly," wrapping would be the right call.
It is lower risk and far less work.

## 2. Rust, not C/C++

Performance is a wash — both compile through LLVM, neither is inherently faster
when written competently. Three things decided it:

- **Memory safety where it matters most here.** The engine manipulates large flat
  arrays across many sequential passes, exactly where C/C++ produces silent
  out-of-bounds and aliasing bugs. The HTML CHANGELOG records hitting this class
  repeatedly — edge-clamping errors, off-by-one indexing, antimeridian wraparound,
  NaN propagation — and catching it only through runtime testing, because JS
  offers no compile-time guarantee. Rust catches much of it at compile time, or
  panics at the faulty line rather than returning a wrong answer three passes on.
- **One toolchain, three targets.** `cargo` cross-compiles to Windows, Android
  (`cargo-ndk`), and WebAssembly, so a future browser build could share the engine
  core. C/C++ reaches WASM through Emscripten with clunkier tooling.
- **Mature Godot bindings.** `gdext` gives Rust GDExtension access without a
  hand-written C ABI bridge.

**The tradeoff, stated plainly:** the borrow checker fights the pattern this
generator leans on — passes mutating large shared arrays in place. Expect real
up-front thought about ownership (arena patterns, `&mut [f32]` threaded through
functions, `rayon` where Web Workers were) rather than mechanical translation.
A one-time cost during the port, not an ongoing tax.

## 3. Godot shell, not pure Rust

Godot solves for free what is not this project's differentiator: windowing, input
including touch, a UI system, and one-click export for Windows and Android —
which carries the NDK, Gradle, and signing plumbing a pure-Rust stack would have
to build by hand. The value lives in generation and simulation, which is what
moves to Rust.

## 4. 2D only for v1

3D means a second rendering pipeline — mesh generation, camera, lighting — on top
of porting and verifying a generation engine. Cutting it keeps the first milestone
achievable. `ROADMAP.md` Phase 3 brings it back.

## 5. Cloud session builds; owner verifies on hardware

Godot cross-exports to Windows and Android from Linux given the right SDKs and
templates, so building needs no Windows machine.

What a container cannot do is run the `.exe` on Windows, install and touch-test
the `.apk`, or confirm real GPU behaviour. Build here, verify there — the same
carve-out the HTML project already documents for WebGL, Workers, canvas, and
touch. **Every milestone states which half it achieved.**

## 6. Personal/hobby distribution

Builds the owner runs: a plain `.exe`, a sideloaded `.apk`. No signing
certificates, store listings, or policy compliance yet. This can change later at
no architectural cost.

## 7. Golden-value parity, not independent correctness

The JS engine carries 200+ versions of measured, owner-verified correctness —
scale-invariant terrain detail, river formation, climate coupling, settlement
placement. Re-deriving that by feel would discard it silently.

Parity makes the existing engine ground truth the port is checked against
mechanically. `PARITY_TESTING.md` covers how, and why exact bit-identity — the
standard the HTML project holds itself to JS-to-JS — is not achievable across
languages.

## 7a. Principled equivalence for GPU/optimized paths (owner decision, 2026-08-16)

§7 above still governs the CPU reference pipeline — every subsystem ported
this session (tectonics, climate, erosion, hydrology, rendering, real HTML
export round-trip) stays golden-verified against the JS engine and that
work is not being discarded or devalued.

What changes: for GPU-accelerated or otherwise re-optimized paths, exact or
tolerance-bound numerical matching against JS is **not** a requirement when
it becomes impractical. The GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`,
`cartalith-gpu`) hit exactly this wall — `hash()`'s JS semantics depend on
IEEE-754 *double*-precision rounding at an intermediate magnitude (~2^61)
that exceeds `f32`'s useful range entirely, and WGSL has no working `f64`
support on this toolchain (`naga` doesn't implement `enable f64;`). Owner's
own framing: "rust, godot and wgpu are inherently a different type of code
language" — cross-hardware GPU determinism is a categorically different,
harder problem than the cross-*language*-on-CPU tolerance §7 already
accepted, and insisting on JS-array-diffable output from a GPU path is not
worth blocking real optimization over.

The replacement bar, for any path where JS-parity genuinely can't be
tested 1:1: implement the same **academic principles and generation flow**
the reference embodies (the actual algorithm/model being approximated, not
an arbitrary reinvention — `PROVENANCE.md`'s citation list still describes
*why* a formula looks the way it does even when its exact digits are no
longer being chased), and judge the result by whether it reaches an
**equal-or-better visual/qualitative outcome**, not by array diffing.
"Same seed reproduces the same world" (this port's own determinism
contract) still holds *within* whichever path (CPU or GPU) actually ran —
this is about JS-cross-checkability, not about abandoning determinism
inside the Rust/wgpu implementation itself.

This reopens a redesigned, GPU-native hash/noise function as a real option
— the GPU pilot's "not viable" verdict was specifically about reproducing
JS's exact rounding, not about GPU noise generation being impossible in
principle. A GPU-safe hash is legitimate future work; scope it properly
(same discipline as every other milestone this session — a scope doc, not
an improvised inline rewrite) rather than retrofitting it into whichever
crate happens to be open at the time.

## 7b. Territory/border generation: cost-distance, strength-weighted (owner decision, 2026-08-16)

> **Correction notice (2026-08-19, cross-repo documentation audit).** The
> premise below — "no algorithm for this at all" — is **false**. The
> reference has a real, wired auto-generation function,
> `_civAutoPolity` (reference HTML line 20665, bound to the "Recalculate
> Territories" button at line 26662), which runs `buildTravelCost` plus a
> multi-source binary-heap Dijkstra seeded from **every settlement**
> (diagonal-weighted, capped at `MAX_REACH = GW*0.35`) and writes the
> territory raster through the local alias `terr` — the reason an earlier
> grep for `civTerritory[` write sites (`PHASE2_SCOPE.md` milestone 9)
> missed it. RC's own vendored `docs/research/political-fragmentation.md:48`
> already says so plainly. This does not retract the decision below — the
> port's capital-seeded, population-weighted design is still what's
> implemented, and may still be the better result — but the "nothing to
> golden-test against" premise in this section's own **Verification
> standard** is false: `_civAutoPolity`'s output (all-settlement-seeded,
> unweighted, reach-capped) is a real comparison point the port never had
> before. Whether to adopt it, offer it as a second mode, or leave the
> current design as-is un-reconciled is an open decision for the owner,
> not something this notice resolves.
>
> **Resolved (2026-08-19, owner decision).** The current design stays as
> the only mode. No reconciliation against `_civAutoPolity`, no second
> mode offered. Closed — do not revisit without new owner direction.

The reference has **no algorithm for this at all** — territory ownership
is set only by hand-painting with a brush tool, or restored from a save
file (`_civGenerateProvinces` partitions an *already-painted* territory
raster; nothing computes that raster programmatically). This is genuinely
new design, not a port, and falls under §7a's "principled equivalence"
latitude — there is no JS behaviour to approximate here at all, only
academic grounding to build from.

**Decision**: cost-distance Voronoi from capitals, weighted by faction
strength (capital population) — not straight-line Voronoi. Each land cell
is assigned to whichever capital reaches it at the lowest *effective*
cost, where effective cost is the real terrain travel-cost distance
(`buildTravelCost`/`roadDijkstra`, Phase 2 milestone 11) divided down by a
monotonic function of that capital's settlement population (higher
population → farther effective reach for the same terrain cost). This
produces borders that follow mountain ranges and rivers rather than being
geometrically arbitrary, and ties directly to two things this port already
has real, cited grounding for: multiplicatively-weighted Voronoi
diagrams (standard computational-geometry technique for size-weighted
spatial competition) and Christaller's central place theory (1933,
already cited in `PROVENANCE.md` for the civilisation layer generally —
this is exactly the "settlement hierarchy projects influence proportional
to size" idea Christaller describes, just applied to faction territory
instead of trade-catchment radius).

**Why not plain Voronoi**: straight-line nearest-capital ignores terrain
entirely and reads as artificial — worse than doing nothing, for a port
whose explicit goal (`DECISIONS.md` intro, owner's own framing) is an
equal-or-better visual/qualitative result, not a technically-simplest one.

**Why not simulated historical expansion** (a contested-territory
flood-fill/war-of-conquest model): considered and deferred, not rejected —
real complexity (temporal simulation, conflict resolution, balancing) for
a v1 that a static weighted-Voronoi pass doesn't need. Revisit only if the
static version's results feel wrong once actually seen, not preemptively.

**Verification standard**: per §7a, this can't be golden-tested against
JS (nothing to compare against) — judge it by whether borders look
geographically plausible on real generated worlds (following terrain
features, stronger factions visibly larger) once implemented, the same
"equal-or-better visual result, judged by looking at it" standard §7a
already established.

**Blocked on**: Phase 2 milestone 11 (the travel-cost/Dijkstra
infrastructure this depends on) landing first — see `PHASE2_SCOPE.md`.

## 7c. Consequence of 7a, made explicit: GPU generation produces a different world than CPU, same seed (2026-08-16)

Worth stating plainly once the first real consequence showed up
(`GPU_LAYER_INTEGRATION_SCOPE.md` milestone 1, the GPU-safe noise
redesign): because the GPU noise primitive is a genuinely different hash
function from the CPU/JS-matching one (not a precision-tolerant port of
the same algorithm — that was proven unreachable), **any pipeline stage
that moves to GPU and depends on noise will produce different output than
the CPU path, for the same seed.** Not "close within tolerance" —
different, by design, the same way two different-but-valid noise
functions always diverge from their first call.

This does not break determinism *within* a path: the CPU path is fully
deterministic and JS-matching (unchanged); the GPU path is fully
deterministic given the new noise (same seed always reproduces the same
GPU-generated world). What breaks is the assumption that "seed + GPU
on/off" together fully determine one canonical world — they don't
anymore, once GPU touches anything noise-derived. §7a already anticipated
this ("this is about JS-cross-checkability, not about abandoning
determinism inside the Rust/wgpu implementation itself") but didn't spell
out the concrete UX consequence.

**Implication for later UI/UX work** (per the `cartalith-ui-per-milestone`
process): whenever a GPU-accelerated generation path is actually wired up
and user-facing, the UI needs to communicate this honestly — a GPU toggle
is not merely "faster," it's "a different (still valid, still
deterministic-per-seed) world," the same way the reference's own
archetype/world-structure toggles change generation, not a hidden
performance-only switch. Exact wording/placement is a UI-pass decision
when that toggle actually exists, not decided here — this entry exists so
the fact isn't rediscovered as a surprise, or worse, silently hidden from
the user, later.

## 8. Documentation here, code in a new repository

`Cartalith_RC` has strict conventions of its own — single HTML file, version per
file, a test and hash discipline tied to that file — that a Cargo workspace does
not fit. A separate repository avoids conflicts over `.gitignore` scope, CI
assumptions, and `CLAUDE.md` ownership.

These documents are written to be copied wholesale into that repository as its
seed documentation. Nothing here depends on staying in `Cartalith_RC`.

## 9. Godot version pinned at setup, not here

Install the latest stable Godot 4.x available when the repository is created, and
record the exact version in its own toolchain notes.

This document was written against a knowledge cutoff. A patch version hardcoded
here risks being stale — missing bugfixes, or naming a release no longer
recommended — by the time anyone executes the plan. Same discipline the HTML
project enforces about re-measuring rather than trusting a stale assumption.

## 7d. Behavior is the contract, not implementation (owner decision, 2026-08-17)

Owner's words, verbatim: "everything should be redesigned in rust and be more
effective (so if qgis/mapbox do the same job and have the same visual result
as the html version take the QGIS and mapbox examples and implementations as
leading — this kind of thing wasn't possible in an efficient way at the time
of the html). Other than that all described functions from the old project
should be maintained."

What this refines: §7's golden-parity discipline and §7a's principled-
equivalence carve-out both already distinguish *what a feature produces* from
*how it computes it*. This decision generalizes that distinction to the whole
port: the reference HTML app defines the **feature contract** — every
described function must exist and produce an equivalent result — but its
*implementations* were shaped by a single-file browser app's constraints
(no threads, no GPU compute, no native file I/O, one canvas). Where mature
tools in the same problem domain (QGIS, Mapbox GL, and by extension the
terrain/DCC lineage the shell now follows) solve the same job with the same
visual result more effectively, their approach is **leading**, and the
reference's approach is historical context, not a requirement.

What this does NOT change: the CPU generation pipeline's existing golden
verification stands — those algorithms were ported and verified precisely
because their *outputs* are the contract, and nothing here re-litigates a
verified stage. This decision governs how *unported and future* work is
approached (rendering architecture, tiling, layer compositing, interaction
models, file handling), and how already-ported presentation-layer code may be
*improved past* JS parity — the door §7a already opened for GPU, now open on
principle wherever a better-practice implementation preserves the behavioral
contract.

Practical test for any future pass: "would a user of the HTML app find this
feature present and its result equivalent or better?" If yes, the
implementation is free to differ. If a change would make a described function
absent or visibly worse, it violates the contract regardless of how modern
the replacement is.

## 7e. Settlement coastal flag computed from final geometry, not the reference's own pre-snap order (2026-08-19)

Owner report: settlements flagged coastal/river-adjacent weren't actually
landing on the water. Investigation found the real root cause was a missing
port of the reference's own `_civSnapToWaterEdge` (a bounded, tolerance-gated
water-edge snap the reference added in v1.36/v1.39, after milestone 8's
golden-parity harness had already deliberately scoped it out as DOM-coupled
logic) — now ported as `place_settlements_with_water_edge_snap`
(`cartalith-civ`), golden-verified against the real reference via 8 new unit
tests built from a small hand-copied Node harness, not guessed.

One small, deliberate deviation from the reference's own statement order
found along the way and disclosed here per this section's own pattern: the
reference computes a settlement's `coastal` flag (`_civIsCoastal`) ONCE,
*before* `_civSnapToWaterEdge` runs, and never re-checks it (reference lines
25423 vs 25558) — meaning the reference itself could, in principle, ship a
settlement whose flag no longer matches its final post-snap position, even
though in practice the snap only ever moves a site *closer* to water, so this
has never been observed to matter. This port recomputes `coastal` on the
FINAL position instead, in `place_settlements_with_water_edge_snap`, since
both the position and `civ_is_coastal` already exist at that point and
recomputing costs nothing extra. This is a correctness improvement with no
observed behavioral cost, not a redesign — flagged per this project's own
"never silently change a numerical result" discipline (`README.md`) rather
than left as an unstated difference from the reference's literal ordering.

Also disclosed: the reference's v1.46 landmass-scoped coastal-PREFERENCE
swap (reference line 25447 — redistributes WHICH settlements are flagged
coastal to hit a per-landmass target share, an abundance/distribution
concern) and its crossroads-settlement promotion pass (reference line
~25607) remain unported. Both are real, separate reference features,
confirmed by reading them, not conflated with the geometry bug this entry
fixes — left for a future pass if the owner wants them.

## 7f. The pre-carve `computeFlow(true)` is skipped when carving runs (2026-08-24)

`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §3.2.1 found, and this pass
confirmed by re-reading every statement between the two points, that
`generate_terrain` computes `flow_discharge` immediately before
`carveRiverValleys` and then **unconditionally overwrites it** at that
function's own step (3). The carve block in between reads `field`, `pre`,
`stress.stress_field`, `resistance_field`, `rainfall` and its own locally
computed `flow_for_network` — never `flow_discharge`. On the default path
(`carve_rivers: true`) the first call's result is therefore discarded unread.

**Why the reference does it.** It is a faithful port of the reference's own
order. In JS, `flowField` is a module global that the renderer, every overlay
and the sample readout may read at any moment, and `carveRiverValleys()` is
conceptually a separate op that could be invoked on its own — so the reference
has to leave the global current between the two. In this port it is a function
local with no observer between its two assignments.

**The deviation.** The call is skipped when `p.carve_rivers` is on. It is
**not** skipped when `carve_rivers` is off, because there it is the output —
the skip is conditional by construction, not an unconditional deletion.

**What it is worth**, measured on this machine with
`cargo run --release --example timing_bench -p cartalith-engine`, best of 3
after a warm-up, `WorldParams::defaults`:

| Size | Before | After | Saved |
|---|---|---|---|
| 512² | 0.3641 s | 0.3280 s | 36 ms (9.9 %) |
| 1024² | 1.1396 s | 1.0385 s | 101 ms (8.9 %) |
| 2048² | 4.8275 s | 4.3955 s | **432 ms (8.9 %)** |

One `compute_flow` call at 2048² measures 402 ms on the same machine, which is
what the saving should be and is.

**Why this is not a §7 parity question.** It changes no formula, no constant
and no operation order — the value that survives is the value that always
survived, computed from the same inputs by the same call. The claim is
*bit-identity*, and it is held to that by
`precarve_flow_skip_leaves_generation_bit_identical` (`cartalith-engine`),
which runs the same generation twice — once skipping, once with the
reference's literal call order restored through a private
`force_precarve_flow` escape hatch — and `assert_eq!`s every raster
`WorldState` carries plus `gpu_stages_used`, over four carving fixtures
(both `world` modes, several seeds) and two non-carving ones. `gpu_stages_used`
cannot differ: `flow_on_gpu` returns `Some` iff `gpu_flow` is `Some`, which is
fixed for the whole function, and the two flow calls inside the carve block
push the same `"flow"` string under the same condition.

Disclosed here rather than absorbed because `CLAUDE.md` requires it —
deviating from the reference's own call sequence is a decision even when the
output is provably identical. Same pattern as §7e.

**Also found and deliberately not taken**: §3.4 of the same research notes that
`compute_temperature`'s first call is likewise unread on the default path.
Left alone — it is one O(N) per-cell pass rather than a global sort-and-walk,
and skipping it would mean restructuring `apply_ocean_currents`' `&mut
temperature` argument. Recorded so the next reader knows it was considered.

## 7g. New opt-in coupling between subsystems is an approved pattern (2026-08-24)

`GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md` §4 asked, as its item 5,
whether the owner wants the door open to *adding* new feedback between
subsystems — beyond the reduction work §3.2 and §3.2.4 already cover — the
way the graphics literature has kept moving for a decade (erosion↔vegetation,
cryosphere↔albedo, dynamic lithology; the latter two already named as
documented follow-ups elsewhere in this repo).

**Owner's answer: yes, opt-in additions are fair game**, on the same terms
`ErosionPassParams` already demonstrates as a working pattern in this
codebase (`crates/cartalith-engine`, wired 2026-08-24, `1f7c295`/`8e666ac`):

- **Off by default.** A new coupling must not change any existing golden
  output when its toggle is left at its default.
- **Real physical justification**, not novelty for its own sake — the same
  bar the six existing erosion passes were held to.
- **Affordable because it is optional.** The cost of the *feature existing*
  is roughly the cost of one more conditional branch and one more toggle in
  `WorldParams`; the cost of *running* it is paid only by whoever turns it on.

This is a standing decision for future work in this area, not a one-time
approval — a later PR proposing a new opt-in coupling (e.g. erosion feeding
back into vegetation/biome classification) does not need to re-litigate
whether such additions are welcome, only whether the specific proposal meets
the three bars above. Raised via `PARITY_AUDIT.md` pass 2 (§13, finding F3),
which found this decision existed only in conversation with no durable
record — recorded here so it has one.

## 7h. The save format is a tree, and it is no longer the reference's (owner decision, 2026-08-25)

Two owner statements, in order, both verbatim.

The requirement:

> "The save zip should have all project files and the folder structure should
> be a clean and clear tree without semantic overlap (not atlas and cartography
> and both storing map tiles)"

The compatibility ruling, given after this port raised that a tree is a
deliberate divergence from the reference's `exportZip()` and asked whether both
layouts should be written:

> "Agreed, importing and reading should work from the old and new format, and
> saving/exporting should strictly be the new format. (document this properly
> as I'd like to upgrade the html version to include some of the new
> functionality."

**What is decided.** Readers accept both the flat legacy layout and the new
tree. Writers produce only the tree. `SAVEFILE_COMPAT.md` is rewritten as a
normative, implementation-neutral **specification** rather than an
observational note, because the owner intends to implement it in the HTML app —
so its audience is a second implementer working in JavaScript who cannot read
this workspace's Rust.

**Why this is recorded here.** `CLAUDE.md`'s standing rule is not to deviate
from the reference silently, and this is the largest deviation the port has
made in a format the reference defined. Before the ruling it was a judgment
call this port would have had to justify; after it, it is settled, and the
distinction matters to whoever re-reads this in six months wondering whether it
is revisitable. It is not, absent a new owner decision.

**What it costs, disclosed.** A project saved by this port can no longer be
opened by an unmodified pre-upgrade `Cartalith Gen1` build. The earlier
"judgment call, disclosed" in `SAVEFILE_COMPAT.md` — writing the whole
parameter block under the reference's own nested names so that a save
round-tripped through the HTML app — was worth its cost while it was nearly
free. It stopped being free the moment it collided with the owner's structural
requirement: keeping it would have meant writing every raster twice, once at
the root for the reference and once under `rasters/` for the tree, which is
*literally* the duplication the first quote forbids. The flat writer survives
as an explicitly-labelled interoperability **export**, not as a save path
(`SAVEFILE_COMPAT.md` §1.1).

**Relationship to §7d.** This is §7d applied to file handling, which §7d's own
closing paragraph names as one of the areas it governs: the reference's flat
archive was shaped by a single-file browser app that had one concept to store,
the behavioural contract ("a project survives save and reload") is preserved
and enlarged, and the implementation is free to differ. The §7d test — "would a
user of the HTML app find this feature present and its result equivalent or
better?" — is answered by the format now carrying the civilisation layer,
history and annotations that the flat one dropped on the floor.

**One constraint the format took on from this decision, worth naming.** A
second implementation in JavaScript makes JSON's single number type a format
hazard rather than a language quirk: integers above 2^53 are unrepresentable
there. `SAVEFILE_COMPAT.md` §14.1 therefore *constrains the format* — every id,
index, count and year must fit the safe-integer range — rather than warning
implementers about it. §14.2 additionally requires readers to accept `1.0`
where an integer is specified, which is the durable form of the fix for
`GUI_GAP_REGISTER.md` KV-04, where exactly this class of bug silently discarded
every knowledge link a user had ever made.

## 7i. Routes are pass-aware, using the reference's own corridor detector (owner decision, 2026-08-26)

**The owner's words:** *"routes should be terrain aware. A steep cliff or
mountain or any other feature would probably always have a most passable point
and humans have a tendency to use those points naturally."*

**The obvious candidate, and why it was rejected on evidence.**
`_civEnhancedTravelCost` (reference line 20958) already carries a mountain-pass
test — a cell whose immediate neighbours along one axis are both `0.018`
higher, more than `0.15` above sea level — and cuts that cell's slope penalty
to `0.40×`, with its own comment saying why: *"Natural through-routes;
road-builders exploit them."* It is used by exactly one caller, the automatic
trunk-road network builder, and the manual Route/Way tools' grids
(`_civLandCostGrid`, `_civMixedCostGrid`) have never seen it. Wiring it in was
a two-line change and was the first thing tried.

**It was then measured, and it is dead.** On a ridged-noise fixture it fired on
**20 cells out of 12 288**. On a real generated 512×384 world it reached **zero
of four** long crossings — not "changed them slightly": zero. The reason is
structural, not a tuning miss: it is a *one-cell* test, and generated terrain
is smooth at one-cell scale. A real pass — hundreds of metres of col between
two summits — does not look like a local minimum between two immediate
neighbours. Shipping it would have satisfied the request on paper and changed
nothing on the map, which is the failure mode this port has a working rule
about.

**What is used instead is also the reference's own.** `buildRouteCorridors`
(reference line 5903) answers the same question at a scale that exists: it
looks `gw/64` cells out along four axes, takes the **minimum** of the two
flanking maxima — its own comment: *"a corridor needs a barrier on BOTH sides
of the axis — min, not max. One steep flank is a hillside; two is a pass"* —
and pushes the result through a knee at `0.45` so the field is near-zero almost
everywhere and spikes only at genuine pinch points. It is already ported
(`cartalith_civ::build_route_corridors`) and already carries golden coverage
through settlement suitability, which is the only consumer the reference gives
it. **The whole divergence is that a router now reads the field the reference
computed to describe exactly what a router needs.**

**How it is applied.** `RouteContext::corridors` carries the field;
`civ_pass_relief` turns a corridor value into a multiplier on the land cell's
*slope term*, `1 - 0.60 × corridor`. At full strength that is `0.40` — exactly
`_civEnhancedTravelCost`'s own pass factor, so the magnitude is the
reference's and only the detector changed. It never touches the `1 +`
baseline, so a pass is cheaper to **climb**, never cheaper than flat ground,
which is what stops a chain of cols out-competing a valley floor. `None` is
byte-for-byte the reference, and **every golden fixture passes `None`**, so
those tests keep meaning "matches v2.10" instead of quietly re-baselining onto
this port's own output.

**Measured on the shipped path** (`cartalith-godot/tests/pass_relief_measure.rs`,
seed 24601 at 512×384, 169 558 land cells):

| measurement | value |
|---|---|
| land carrying any corridor value | 30.8% |
| mean corridor over land | 0.064 → a **4%** slope discount on the average cell |
| land above half strength | 1.02% → **30–60%** off, where the pinch points are |
| long crossings whose route changed | 2 of 4 |

That is the shape the term has to have: invisible almost everywhere, decisive
at the ~1% of cells that are real passes. The test asserts loose bounds around
those numbers, so a later terrain-pipeline retune that makes the field dead —
or makes it broad — fails there rather than being noticed by eye.

**Cost.** `build_route_corridors` runs once per `RouteInputs::build`, i.e. once
per way/route commit or re-route, not once per Dijkstra leg. It is suppressed
for `RouteMode::Water` (no slope term to relieve) and for worlds under 128
cells wide, where its own `max(2, gw/64)` reach collapses to two cells and it
would be measuring noise rather than a range.

**Not taken in the same pass:** `_civEnhancedTravelCost`'s other two terms —
the swamp/floodplain penalty and the river ford-vs-bridge cost — are equally
absent from the route grids and equally defensible to add. They need
`flow`/`flow_thresh` plumbed onto `RouteContext`, which this term does not, so
they are named here as the obvious next step rather than bundled in.

> **Correction notice (2026-09-01).** The paragraph immediately above is
> history: **the obvious next step was taken.** Both terms are built and
> wired, and this section's own "not taken" wording is what
> `OUTSTANDING_WORK.md` §2.8 flagged as stale. Verified by opening each
> symbol rather than by re-reading a document:
>
> - `civ_swamp_penalty` and `civ_river_crossing_cost` are private `fn`s in
>   `cartalith-civ/src/lib.rs`, factored out of `civ_enhanced_travel_cost`
>   itself so the auto-network builder and the manual Route/Way tools cannot
>   drift apart — that function calls the same pair. Magnitudes are the
>   reference's: `1.8×` for low-lying land (within `0.06` of sea level) whose
>   flow exceeds `8 × flow_thresh`, and an additive `8 × mag × ford_k` river
>   crossing with `ford_k` `0.35`/`0.75`/`1.00` by Strahler order.
> - The plumbing this section said the corridor term did not need is now
>   there: `RouteContext::flow` and `RouteContext::flow_thresh`
>   (`cartalith-civ/src/tools.rs`), read by `civ_land_cost_grid` **and**
>   `civ_mixed_cost_grid` — so the ford term reaches `RouteMode::Land`,
>   which is why `RouteContext::river_order`'s own doc now names two
>   consumers instead of one.
> - All three `RouteContext` construction sites in
>   `cartalith-godot/src/lib.rs` — `way_commit`, `route_commit` and
>   `jp_reroute` — pass `Some(&ws.flow_discharge)` and a
>   `cartalith_hydrology::river_flow_thresh` value. There is no fourth
>   *production* site; the two others in the crate are test fixtures
>   (`infra_tools_bridge.rs`'s `#[cfg(test)]` helper and
>   `tests/pass_relief_measure.rs`), and both pass `flow: None` on purpose —
>   see the parity note below.
>
> **The parity position is unchanged, and deliberately so.** `flow: None`
> reproduces the reference's own falsy `flow` guard exactly — penalty `1.0`,
> crossing cost `0.0` — so every golden fixture predating the field still
> means "matches v2.10", the same argument `corridors: None` carries above.
> Coverage is the unit test
> `civ_swamp_penalty_and_river_crossing_cost_match_the_reference_formula` in
> `cartalith-civ/src/lib.rs`, which pins both formulas including the
> ford-cheaper-than-bridge ordering and the at-threshold boundary.

**Relationship to §7d.** The §7d test — *"would a user of the HTML app find
this feature present and its result equivalent or better?"* — is answered by
both halves already existing in the HTML app: the detector, and the router that
never asked it anything.

## 7j. The Journey Planner applies its per-stage suggestions, not just shows them (owner decision, 2026-08-26)

**The owner's words:** *"per stage should auto pick either according to terrain
or animals/carriage. Basically it should always pick from technically best and
available per stage. (and scale to group and cargo size)."*

**What the reference does.** It computes exactly this, twice per land stage:
`_jpBestLandTransportForStage` (line 18053, v1.53) measures which land mode is
fastest on that stage's own ground, and `_jpBestPackageForStage` (line 18080,
v1.66) measures which pack species and vehicle that stage's terrain rewards.
v1.66 exists because of an owner report quoted in the reference itself: *"at
the desert transitions they will exchange their mule and cart for camels with
travois and a different supply set-up... For now I cant make any such a
finetunement."* Both functions carry the same stated contract — **"measure,
never silently apply"** — and both are reached only from `_jpRenderResults`,
which renders "⚡ faster mode available" past a **+10%** margin and leaves the
swap to the user. Both were ported in milestones 2/6 with tests, and until now
neither had a production caller.

**The decision.** `jp_auto_stage_picks` applies them, behind `jp_compute`'s
opt-in `auto_stage` key and the party form's own "Re-pack per stage where it
pays" toggle — off by default, because it rewrites per-stage overrides and
doing that unasked the first time a route opens would be acting behind the
user.

**Four rules keep it from being a worse answer than the suggestion it
replaces**, all asserted in
`auto_stage_picks_only_emit_measured_improvements_and_apply_as_overrides`:

1. **The reference's +10% margin is kept.** Its stated reason — *"so a 1%
   numerical wobble never nags the user"* — applies at least as strongly when
   the swap is made rather than shown. A party that re-tacks its whole train
   for a 2% gain is not modelling anything real.
2. **A blocked stage is not skipped, and the margin does not apply to it.**
   There is no percentage between "cannot cross" and "can cross". This is the
   owner's own scenario: a train with carts does not cross Deep Sand *slowly*,
   it is refused outright — so a picker that skipped blocked stages would have
   been useless for precisely the case v1.66 was written for.
3. **Availability is checked, because `jp_calc_land` deliberately does not.**
   `jp_capacity_ex`'s v1.83 branch issues `group_size - declared` mounts to a
   Mounted Rider party, because in the reference a human typed "Mounted Rider"
   into the form and that *is* the declaration. With no human behind it, the
   first run of this picker duly "discovered" that a twelve-person, 900 kg
   merchant caravan travels **39% faster as riders** — by conjuring ten horses
   it does not own and leaving the cargo on the road. `jp_stage_mode_available`
   is this port's own gate and exists for that.
4. **A hand-set per-stage field always wins.** `auto_stage` fills gaps in
   `stage_overrides`; it never overwrites a value the user set.

**Scaling to group and cargo is inherited, not re-implemented.** Every
candidate is measured through the same `jp_calc_land` the stage itself uses,
against that stage's *effective* plan — which already carries group size,
cargo, supply days and the animal counts. A twelve-person caravan and a lone
courier get different answers from the same terrain without this function
knowing anything about either. Animal *counts* are preserved rather than
resized: sizing a train from cargo stays `jp_auto_pick_transport`'s job for the
whole route, which is the reference's own division and its own words.

**All-or-nothing application.** The picks are measured against the stages the
first plan derived, so applying them needs a second `jp_plan_full`. If that
replan derives a different number of stages, nothing is applied and no picks
are reported — rather than overrides landing on the wrong stages.

**Relationship to §7d.** The feature is present in the HTML app as a
recommendation the user must execute by hand, stage by stage, on a form that
had no per-stage vehicle control at all until v1.66 added one. "Equivalent or
better" is met by executing it.

## 7k. Paint brush falloff: a probability-threshold edge, no palette index ever blended (owner ruling, 2026-08-31)

**The owner's ruling** (`LARGE_ITEM_RULINGS.md`, taken by interrogation over
`UNWIRED_FUNCTIONS.md`'s Large section): *"Bind it — add a falloff term to
`PaintStamp`... This is a deliberate divergence from the reference, not a
parity fix... It must be recorded in `DECISIONS.md` as a divergence when it
lands. Also resolves the duplicate: two `Hardness` copies are on screen at
once today, and only one should survive."* This was the highest-severity row
in `UNWIRED_FUNCTIONS.md` and the source of both remaining dangerous-class
entries there.

**What the reference does.** Nothing. `cartalith-spatial/src/paint.rs`
quotes it verbatim: painting is *"a hard disc... unlike `sculpt()`/
`brushHeight` there's no soft falloff here."* `Brush::hardness`/`softness`
(`paint_bridge.rs`) were accepted, clamped and stored since the Paint tool
shipped, but went nowhere — `PaintStamp::apply` never read either one.

**Why this is a divergence and not a parity gap.** There is nothing to
port: the reference brush has no falloff of any shape to reproduce, hard or
soft. `DCC_SHELL_SPEC.md` §4.5.2's tool options row lists the two sliders
anyway, almost certainly carried over from the Sculpt row's own shape rather
than a deliberate reference behaviour this port had simply failed to find.

**The mechanism** (`cartalith_spatial::paint::PaintStamp::with_falloff`,
`paint.rs:180`). The categorical-blending objection the reference's own
comment raises is real and untouched by this: averaging two palette indices
produces a meaningless third index, so every painted cell still carries
exactly one clean index, always, at any hardness or softness. What softens
is the disc's own *edge* — which cells a dab touches at all — never the
*value* a touched cell receives.

- `hardness`/`softness` (`paint.rs:143-144`) are the two `DCC_SHELL_SPEC.md`
  §4.5.2 sliders, verbatim and uncombined by the caller. They combine into
  one *softening* amount inside `PaintStamp` itself,
  `((1.0 - hardness) + softness).clamp(0.0, 1.0)`
  (`PaintStamp::feather_width`, `paint.rs:199`) — moving either slider away
  from "fully hard" (`hardness = 1, softness = 0`) softens the edge a
  little, both pushing the same needle the same way, rather than one being
  forced into the other's exact inverse.
- That amount times the radius is the width, in cells, of a falloff band at
  the disc's own outer rim. Inside `radius - width` every cell paints
  unconditionally — what keeps the centre solid rather than fading it
  uniformly; from there out to `radius` (the disc's existing hard boundary,
  unchanged) the paint probability ramps linearly from 1 to 0
  (`passes_falloff`, `paint.rs:219`).
- The probability is decided against `cell_dither` (`paint.rs:249`): a
  deterministic hash of the cell's own absolute grid position, not a
  per-frame random draw, so repainting the same spot at the same brush
  settings keeps or drops exactly the same cells every time — the brush
  stipples the map, it does not flicker. It is MurmurHash3's public-domain
  `fmix64` finalizer over a salted position key, picked only for a fast,
  good avalanche — **not for JS parity**: there is no reference falloff to
  match, so `cartalith-rust-conventions`' precision-matching rules do not
  govern this function.

**The bit-identity guarantee.** `PaintStamp::new`/`PaintStamp::ungated`
construct with `hardness: 1.0, softness: 0.0`. `(1.0 - 1.0) + 0.0` is `0.0`
with no rounding — IEEE 754 subtraction of two equal finite operands is
exact — so `feather_width()` is exactly `0.0` for every stamp that never
calls `with_falloff`, and `PaintStamp::apply` skips the falloff branch
entirely for it rather than evaluating a probability that always comes out
to 1. This is a **strict superset** of the old behaviour, not a
replacement: `cartalith-civ`'s territory brush (`cartalith-civ/src/
tools.rs:973`, `cartalith-godot/src/civ_tools_bridge.rs:345`) calls
`PaintStamp::ungated` and never `with_falloff`, so it is untouched by this
entry and stays a hard disc forever, by construction rather than by a
separate check. Every pre-existing golden/regression test for the hard-disc
case — `cartalith-spatial/tests/golden_parity_paint.rs`'s 7 cases (checked
against the reference's own `_paintAt`), `cartalith-civ`'s territory-brush
suite, and every hard-disc test already in `paint.rs`'s own module — passed
unchanged; none needed touching.

**Verification.** `cargo test -p cartalith-spatial --lib`: 148 passed, 0
failed, including 4 new tests added with this entry — bit-identity at the
construction default and again with an explicit `with_falloff(1.0, 0.0)`
call at two different radii; a mottled (not merely smaller) edge at
`hardness=0.4`, checked over the full ~1 000-cell annulus rather than a
single ray so the assertion cannot pass by luck; determinism across two
applications of one stamp; softness alone feathering the edge while
hardness stays at `1.0`. `cargo test -p cartalith-spatial --test
golden_parity_paint`: 7 passed, unchanged. `cargo test -p cartalith-godot
--lib`: 409 passed, 0 failed, 6 pre-existing ignores, including 3 new tests
exercising the same two claims through `PaintEditor::stroke_at`/`Brush`'s
real public names rather than `PaintStamp` directly. `cargo test -p
cartalith-civ`: the 513-test lib suite plus every golden-parity suite in
the crate, all passing — the territory brush is provably unaffected.

**The duplicate slider.** `UNWIRED_FUNCTIONS.md` separately flagged
`Hardness` drawn live in two places at once. `world_workspace.gd:2103` (the
WORLD dock's Biome paint panel, which also carries `Softness` at `:2105` and
owns the actual `_paint_brush` dictionary) and `tool_bar.gd`'s unified tool
options bar only mirrored that same dictionary through `_paint_state`/
`_write_paint_state` and never held a value of its own. The dock's copy
survives; the tool bar's was deleted outright — between its `Size` slider
and `Land only` toggle — not hidden or disabled, matching Sculpt's own
precedent of a narrowed subset in the bar against the dock's fuller set
(the bar still has no `Softness` control at all, and none was added: nothing
here was ever duplicated for that field).

**Relationship to §7d.** Moot rather than met. §7d asks whether a divergent
implementation is equivalent-or-better for a reference *feature*; this one
has no reference feature to be equivalent to, so there is nothing it could
regress. Recorded per this section's own pattern (§7e, §7f) because
deviating from "the reference has none at all" is still a decision, not
because §7d's own test applies to it.

## 7l. Crater frequency is an area density, behind an off-by-default flag (2026-09-02)

**The defect.** `stampCraters` (reference line 3569) stamps exactly
`state.crater.count` craters whatever the map represents:
`while(placed<c.count && guard<c.count*40)`. `cellKm=state.mapWidthKm/GW`
scales crater *size* correctly and the *count* not at all. Cartalith's own
width range runs 5 km to 40 000 km, an area ratio of **64 000 000:1**, so the
same slider position is a negligible density on a world and an unrenderably
dense one on a region. Verified in the reference and in
`cartalith_terrain::stamp_craters`, which ports it faithfully.

**Owner ruling, 2026-09-02: break parity here.** Raised as a §7a conflict and
ruled on directly — *"I'm the user telling you to break that on this point on
basis of the new information that I've provided to make the generation more
scientifically accurate."* So the density model is the **shipped default**
(`CraterParams::physical_model: true`), not an opt-in. Setting it `false`
restores the reference's own path byte for byte, and the import/inversion path
keeps it.

**What the goldens did instead of being re-baselined.** Six golden cases run
craters (`golden_parity_volc_craters` x2, `golden_parity_pipeline` x2,
`golden_parity_carve` x2). Their expected values were captured *from the
reference under Node*; regenerating them from this port's own output would turn
a parity test into a self-referential snapshot and silently delete the parity
coverage of every other stage in those pipelines. So each pins
`physical_model = false` instead, with the reason in the file. They remain true
reference parity; the new model has its own tests in
`cartalith_terrain::crater_density_tests`. **No golden data was discarded.**

**The model.** `lambda = R20 * T * A * (20/Dmin)^b * I`, then
`N ~ Poisson(lambda)`, then diameters drawn from a truncated `D^-b` law over
`[Dmin, 400 km]`.

- `R20 = 5.6e-15 km^-2 yr^-1` — Grieve & Shoemaker's rate for `D >= 20 km`.
- `T` — a **geological** surface exposure age, default 100 Myr.
- `A` — the map's real area, so a 5 km region and a 40 000 km world differ by
  the 64 000 000x they actually differ by.
- `Dmin` — **resolution-aware**: `max(1 km, 2 cells)`. This is what makes the
  physical model tractable. At 2048 cells a 40 000 km map has 19.5 km cells, so
  its smallest *resolvable* crater is ~39 km, not 1 km — and under `D^-2` that
  floor removes the overwhelming majority of a population that would otherwise
  number in the hundreds of thousands. Physics and performance agree.
- `I = count/100` — the existing 0-200 slider as an intensity multiplier, so
  the reference's own default of 100 means "physically calibrated".

**The calibration is a coincidence worth stating**: at the app's untouched
default (800 km, 2048x1311) with `T = 100 Myr`, `lambda ~= 92` — against the
reference's hand-tuned default `count` of 100. The physical model lands almost
exactly where the tuned constant already sat. Asserted in
`the_physical_model_lands_near_the_references_own_default`, so the claim fails
loudly if it stops being true.

**Three clocks, and they must not be confused.** The owner's second ruling the
same day: *"the timeline for civilisation is different than a timeline for a
geological scale."* Cartalith now carries three distinct age quantities:

| Quantity | Scale | What it means |
|---|---|---|
| the civilisation Timeline (`TIMELINE_SCOPE.md`, the year cursor) | years - millennia | when things happened to people |
| `CraterParams::surface_age_myr` | 10^4 - 10^9 years | how long this surface has collected impacts |
| `CraterParams::age` | 0-1, unitless | how *worn* each crater looks (morphology) |

Reading the year cursor into crater density would make a civilisation's rise
change the crater count, which is nonsense. They are not points on one axis and
nothing converts between them. `surface_age_myr` is deliberately its own
parameter for this reason.

**The size-frequency law is now built**, which the first cut deliberately left
out: the reference's three flat bands (90% at 0.5-5 km radius, 9% at 5-25, 1%
at 25-200) sampled *uniformly within* each band, which produces far too many
large craters relative to small ones. `crater_diameter_km` replaces them with
the inverse-CDF of a truncated power law. Morphology thresholds (`large`,
`basin`) are keyed off the drawn radius in both modes, so a crater of a given
size still looks the same — only how often each size is drawn changes.

**Sources, checked rather than accepted.** The owner supplied a research note;
these are the citations that survived verification:

- Grieve, R. A. F., & Dence, M. R. (1979). *The terrestrial cratering record:
  II. The crater production rate.* Icarus **38**, 230-242. **Confirmed** —
  volume and pages exact.
- Grieve, R. A. F., & Robertson, P. B. (1979). *The terrestrial cratering
  record: I. Current status of observations.* Icarus **38**. **Confirmed.**
- Grieve, R. A. F. (1984). *The impact cratering rate in recent time.* JGR,
  doi:10.1029/JB089iS02p0B403. **Confirmed.**
- Grieve, R. A. F., & Shoemaker, E. M. (1994), in *Hazards Due to Comets and
  Asteroids*, Univ. Arizona Press, pp. 417-462. **Chapter confirmed; the title
  in the supplied note is wrong** — it is *"The record of past impacts on
  Earth"*. This is the source of **(5.6 ± 2.8) x 10⁻¹⁵ km⁻² yr⁻¹ for D ≥ 20 km**,
  which is confirmed.
- Oetting, A., et al. (2025). *Slopes of Lunar Crater Size-Frequency
  Distributions on Exterior Impact Melt Deposits of Young Craters.* JGR Planets
  **130**, e2024JE008589. **Confirmed** — and it measures a CSFD slope of
  **2.85** for craters ≤10 m, i.e. the slope is *not* −2 across all sizes.
- French, B. M. (1998). *Traces of Catastrophe*, LPI Contribution 954. Cited
  twice in the supplied note's body and **absent from its reference list**.
- Fassett (2016), Hartmann (2008), Cai & Fa (2020), Wünnemann et al. (2010):
  confirmed real.
- **Not found: "Grieve (1981), The record of large scale impact on Earth."**
  The 1981 Grieve paper that exists is Grieve, Robertson & Dence, *Constraints
  on the formation of ring impact structures* — different topic, three authors.
  Treat that citation as unverified.

**Three caveats the supplied note understates, recorded because they bound
what this model may later claim:**

1. **5.6 x 10⁻¹⁵ is the highest of the published estimates**, not a consensus.
   Hughes (1981) gives (2.6 ± 0.9) x 10⁻¹⁵ and Hughes (2000) (3.46 ± 0.30) x
   10⁻¹⁵ km⁻² yr⁻¹ — roughly *half*. The note presents the high end as *the*
   figure without saying so.
2. **The −2 cumulative slope is for D ≳ 20 km and must not be extrapolated
   down.** The note's own table runs it to D = 1 km, which is exactly the
   extrapolation its §4 warns against — and this engine's smallest band is
   radius 0.5 km, i.e. **D = 1 km**, squarely in the invalid zone. Oetting
   (2025)'s 2.85 slope is direct evidence the exponent moves with size.
3. **The note's central formula is dimensionally incomplete.** The rate is per
   *year*; a density needs a surface age `T` (`rho = rate x T`). Its §7
   `lambda = rho_max I^gamma A F_scale` has no age term and never assigns
   `rho_max` a value, so it cannot be evaluated as written. **This is why the
   implementation anchors on the app's own default rather than on the
   terrestrial rate**: the physics fixes the *shape* (density x area, Poisson),
   and the anchor fixes the *scale*, without pretending to a calibration the
   source does not supply. `crater.age` is a morphological degradation term
   (0-1), **not** a surface exposure age in years; conflating the two would be
   a bug.

**Still not built:** a piecewise size-frequency exponent. `CRATER_SFD_EXPONENT`
is a single `2.0` across 1-400 km, which is knowingly outside the range its
evidence covers (caveat 2 above). A piecewise law would be more faithful, but no
source in the supplied research gives a terrestrial exponent for the 1-20 km
band, and inventing one would be worse than extrapolating a measured one. The
constant is named and documented so it can be tuned when a source exists.

### 7l-i. Degradation over geological time (2026-09-02, same authorisation)

§7l built crater **frequency** and **size**; it did not build **morphology with
age**, which is what made its own record incoherent. `stamp_one_crater` scaled
depth by `1 - age*0.8` — linear in a unitless 0-1 term, carrying no length at
all, so a 1 km crater and a 100 km crater of the same age came out equally
fresh. Built now under the same crater authorisation, gated behind the same
`crater.physical_model` flag, so every golden stays bit-identical.

**The physics, and why it is the missing piece.** Crater topography relaxes
diffusively, so relief decays as `exp(-t/t_diff)` with `t_diff ∝ L²/kappa`. The
`L²` is the whole finding: at one diffusivity a 100 km crater's timescale is
**10 000x** a 1 km crater's, so one surface of one age holds fresh large craters
and erased small ones. That is why Earth's crater record is a *preserved subset*
rather than a census — the question §7l left open when it computed a production
rate but no survival.

**The anchor, and what it is not.** `kappa` is **not** measured and no
diffusivity is claimed. It is pinned exactly the way §7l pinned density — one
free constant chosen so the default lands where the tuned default already sat:

> a crater of `CRATER_D_MIN_KM` loses **half** its relief in one
> `CRATER_SURFACE_AGE_MYR`,

giving `tau = ln2 · (T/T_ref) · (D_ref/D)²` and introducing no numeric constant
that §7l had not already named. Averaged over the population the model actually
draws, the default map keeps **0.72** of its crater relief — worn, not erased;
asserted in `the_default_map_keeps_most_of_its_crater_relief` so the claim fails
loudly if it stops being true. The implied `kappa ≈ 7e-3 m² yr⁻¹` does land
inside the order of magnitude usually quoted for terrestrial hillslope
diffusion, which is recorded as a sanity check and nothing more — unlike §7l's
citations, that range has **not** been verified against a paper here.

**Each feature relaxes on its own length, not the crater's.** The four
`CRATER_FEATURE_*` fractions are read off the profile `stamp_one_crater`
already draws (bowl `1.00 D`, rim `0.20 D`, central peak `0.18 D`, basin rings
`0.33 D`), so they carry no calibration of their own — and the rim, a fifth of
the diameter wide, ages **25x** faster than the bowl it encloses. **This is the
visible change**: below about 2 km at the default surface age, craters lose
their rims entirely and become shallow depressions, which under §7l's `D^-2`
population is most of the count. The relief total barely moves (~13% over the
whole 0-4000 Myr range) because the few largest craters carry it and they are
the ones that barely relax.

**`crater.age` multiplies the physical term; it does not feed it.** The two
answer different questions — `crater.age` is an authoring control that shallows
the whole population uniformly regardless of size, `tau` is size-dependent
physics. Folding `age` into the elapsed time was rejected: `age = 0` would then
mean "no degradation at all", so a cosmetic-looking slider would silently switch
off the physical model. Caveat 3 above already warned that conflating those two
would be a bug; a multiplier is what keeps them distinct.

**Two things deliberately left alone, both needing their own ruling.**
`impact_field` is **not** degraded — it marks shocked rock and impact melt for
the lithology stage, which survives the topography, and damping it would move
biomes, settlement placement and roads, outside what §7l authorised. And
degradation is **not** wired to `stream.diffuse_d` (§7m's hillslope
diffusivity), which is the same physics at a different scale: coherent, but it
would make craters change when a user touches an erosion slider.

> **Superseded the same day.** Both were put to the owner and both were ruled
> in — see §7l-ii immediately below, which also disposes of the two volcanism
> flags §7l-i's neighbours were holding open. Nothing above is retracted; the
> reasoning stands, the answer changed.

### 7l-ii. The owner's three rulings of 2026-09-02

Three questions were put to the owner on 2026-09-02 and **all three were
answered yes**. They are recorded together because they landed together, and
implemented so that **any one is revertible without the others** — the volcano
flags are two independent lines in `cartalith_godot::params::defaults`, the
diffusivity coupling is one parameter on `crater_degradation_tau`, and the
`impact_field` damping is one multiplier in `stamp_one_crater`.

The pattern §7l established carries all three: **`WorldParams::defaults` keeps
the reference's behaviour** and `cartalith_godot::params::defaults` — the app
boundary every `WorldGen` routes through — turns divergence on. That is what
lets ~28 golden suites, sixteen of them in `cartalith-civ`, keep meaning "what
the reference does" while the shipped generator diverges.
`exactly_the_ruled_divergences_ship_at_the_app_boundary` now enumerates the
roster and fails if a fourth divergence appears in either function.

#### Ruling 1 — both volcanism flags ship

`volc.exclude_transform` and `volc.edifice_model` were built, tested and left
`false` at both boundaries because §7l's authorisation was *for craters*. They
now default `true` at the app boundary, `false` in `WorldParams::defaults`.

**The exclusion, re-measured with the flag on** (`volcano_transform_boundaries.rs`,
`arc_and_rift_pools_are_polluted_by_transform_cells`, 256x160 over 12 seeds —
26 960 boundary cells, 10 022 of them transform, 37.2%):

| pool | flag off | flag on |
|---|---|---|
| arc (`conv`) | 11 248 cells, **3 863 transform (34.3%)** | 7 385 cells, **0 transform (0.0%)**, 65.7% of the sites survive |
| rift (`div`) | 13 278 cells, **4 292 transform (32.3%)** | 8 986 cells, **0 transform (0.0%)**, 67.7% of the sites survive |

Zero is by construction, not by luck — the filter drops exactly the cells this
crate's own `classify_boundary` types `TRANSFORM`. The figure worth having is
the second one in each cell: **the correction removes about a third of the
candidate sites and leaves two thirds**, on every one of the twelve seeds, so
it corrects placement rather than starving it. `excluding_transform_leaves_both_pools_populated`
asserts the per-seed floor and the measurement test now asserts the aggregate.

**End to end at the shipped defaults** (96x72, seed 2026 — all three *flags*
against the parity baseline; ruling 2 contributes nothing here, since it
changes no default): **6 911 of 6 912 cells differ, mean |Δ| 0.0129,
max |Δ| 0.3416, land 5 578 → 5 589 cells.** Recorded because the first figure
invites the wrong conclusion. Essentially every cell moves — the field is
renormalised and then routed and carved, so any perturbation propagates
globally, the same chaotic sensitivity §7a already relies on. The *magnitude*
is what says this is the same world differently detailed: about 1% of full
scale on average, and a land fraction that moved by 11 cells in 6 912.
`the_shipped_defaults_generate_a_different_world_from_the_parity_baseline`
bounds the magnitude and deliberately does **not** bound the count.

#### Ruling 2 — crater degradation reads the erosion diffusivity

`crater_degradation_tau` carried a private `kappa`. It now takes
`ErosionPassParams::diffuse_d`, because `hillslope_diffuse` (§7m) is the same
physics at a different scale and a world cannot coherently hold two unrelated
diffusivities.

**The calibration is preserved exactly, not approximately.** The formula becomes

```text
tau = ln2 · (d / d_ref) · (T / T_ref) · (D_ref / D)²
```

with `d_ref = CRATER_DEGRADATION_DIFFUSE_D_REF = 0.15`, the reference's own
`state.erosion.diffuseD`. At the default `d` that middle factor is `1.0` **bit
for bit**, and it is written first so the remaining operand order is untouched,
so the whole expression is the anchor it replaces, unchanged. §7l-i's half-life
still holds and is still asserted; `the_default_diffusivity_reproduces_the_old_anchor`
adds a direct `assert_eq!` against the old closed form at four `(D, T)` pairs.
`the_crater_anchor_matches_the_shipped_diffusivity`, in `cartalith-engine`
because the two constants live in crates that cannot see each other, fails if
either side moves.

**It reads the raw `diffuse_d`, not `hillslope_extent_scale`'s corrected
value.** That correction exists to make a one-cell Laplacian mean the same
physical diffusion at any cell size (§7m) — a discretisation fix for the kernel,
not a different `kappa`. `crater_degradation_tau` already works in kilometres
and megayears, so it wants the physical quantity. It also reads it whether or
not the hillslope *pass* is enabled: a diffusivity is a property of the
landscape, not of which buttons the user pressed.

**The coupling is the point, and it is made visible in three places**, because
a coupling nobody can see is worse than no coupling: the function's doc comment,
`stamp_craters`' doc comment, and the GUI control itself, whose label is now
*"Diffusivity D (also weathers craters)"*.

**One contract was narrowed, deliberately.** `ErosionPassParams`' documented
promise is *"off is bit-identical"* — every pass off means every knob inert.
`diffuse_d` is now the exception: under `crater.physical_model` it changes
generated terrain with every pass off, because it is no longer only an erosion
knob. Pinned in both directions by
`the_erosion_diffusivity_reaches_craters_only_under_the_physical_model`, which
asserts it is inert on the reference path — which is what keeps
`erosion_passes_off_leave_generation_bit_identical`, and the sixteen
`cartalith-civ` suites, true.

#### Ruling 3 — `impact_field` degrades, gated behind `crater.physical_model`

`impact_field` marks shocked rock and impact melt for the lithology stage, and
was written pristine no matter how relaxed the crater — a fully degraded crater
still stamped a fresh impact signature.

**The gate is kept, and the reason first recorded here was wrong.** This
paragraph originally claimed that damping `impact_field` "moves lithology, and
through it biomes, carrying capacity, settlement placement, roads and sea
routes: sixteen `cartalith-civ` golden suites… Ungated, sixteen suites fail.
That is measured history, not caution."

**Corrected 2026-09-02, by measurement.** It does not. `impact_field` reaches no
downstream consumer in the civilisation layer at all: `grep -rn impact_field
crates/cartalith-civ/src/` returns **nothing**, `build_lithology` takes no
impact field, and `compute_affordance_fields` passes only `field`, `age_field`,
`volcanic_field`, `crust_field`, `resistance_field` and `rainfall`. Run with the
gate deliberately removed, `cargo test -p cartalith-civ --no-fail-fast` gives
**27 of 27 binaries passing, 0 failures** — while `cartalith-terrain`'s own
`golden_parity_volc_craters` fails 2 of 5, which is what proves the ungate was
actually live.

The claim came from this session's own briefing, which carried the sixteen-suite
figure forward from the crater **height-field** change — where it was real and
measured — and attached it to a different field that does not share those
consumers. Two changes to the same subsystem, one blast radius, applied to the
wrong one.

**The gate stays**, on its true and smaller justification: `impact_field` is
saved (`cartalith-io`) and is pinned by `golden_parity_volc_craters`' two
reference-extracted `expected_impact` arrays. Those are the fixtures the gate
protects. `tau` is `None` whenever `crater.physical_model` is false —
`WorldParams::defaults` — so the factor is exactly `1.0` there and `(1 - t) ·
1.0` is bit-identical to the value it always was.
`the_reference_path_writes_an_undamped_shock_record` asserts the gate directly,
and an ungate mutant is killed by it.

**The physics, stated honestly: the shock record outlives the landform.** A
relaxed crater still has shatter cones and a melt sheet, so the damping must be
*gentler* than the topographic one, not equal to it. The rate is expressed the
only way this file already knows how — a feature length fed through the same
`1/(frac·D)²` law §7l-i's four `CRATER_FEATURE_*` fractions use — so it adds no
second mechanism. `CRATER_FEATURE_SHOCK = 2.0` is the shocked zone's own extent:
a continuous ejecta blanket and its shock aureole reach roughly one crater
*radius* beyond the rim, so the affected patch is about twice the crater's
diameter across, against the bowl's `1.00 D`. The shock timescale is therefore
exactly **4x** the bowl's.

**What is not claimed.** The 2:1 extent is a standard order-of-magnitude figure
for an ejecta blanket; **no shock-annealing rate was measured, fitted, or taken
from a source, and no source in this repository has been checked for one.** What
is defensible is the sign and the shape — monotone decay, on the same diffusive
law, slower than the relief. `the_shock_record_fades_but_outlives_the_landform`
asserts exactly that pair of claims and nothing stronger, and
`the_shock_aureole_relaxes_four_times_slower_than_the_bowl` pins the ratio
against the closed form with the `2.0` written as a literal, so a mutation of
the constant cannot move both sides of the comparison together.

One consequence worth naming: the `max` that resolves overlapping craters now
means "the fresher or larger signature wins" rather than "the one whose centre
is nearer", so a young crater overprints an ancient one. That is the right
behaviour and it was not available before.

## 7m. Hillslope diffusion is corrected for real map extent (2026-09-02)

**The defect.** `hillslope_diffuse` (`cartalith-erosion/src/passes.rs`) computes
`delta = d * (l + r + u + dn - 4h)` — an explicit five-point Laplacian at a grid
spacing of exactly one cell, so `delta = d · dx² · ∇²z_real`. Matching the
physical `dz = D·dt/dx²·∇²z` requires `d` to scale as `1/cell_km²`. It did not:
`cartalith-erosion` contained **zero occurrences of `map_width_km`**, verified,
and the same literal `diffuse_d` applied at every extent.

At 2048 cells a 5 km region has 0.00244 km cells and a 40 000 km world 19.53 km
cells. The ratio of cell **areas** is 8 000² = **64 000 000** — the identical
figure §7l cites for the crater count, one layer up the pipeline.

**Scope, deliberately narrow.** The owner ruled 2026-09-02: *"let's only fix the
hillslope extent blindness."* Two investigations had shown why the larger
geological-clock feature was the wrong first move — at the app default
`stream.uplift = 0.0`, so a duration term wired to erosion today would mean
exactly one thing, a flatter world; and stream power equilibrates (measured in
this engine: per-iteration change falls to 2e-5 by 360 iterations), so more time
stops changing anything. **This is the extent fix alone. No clock.**

**Not a defect everywhere.** `stream_power_kernel`'s extent-blindness is
dimensionally *correct* and was deliberately left alone: it computes
`area^0.5 / l` with both in cells, and `area_cells^0.5 / l_cells =
A_real^0.5 / L_real` — the cell size cancels identically at m=0.5, n=1. A 5 km
region and a 40 000 km world genuinely should incise the same per unit time.
"Fixing" it would introduce a bug.

**The correction is one-sided, and that is the safety story.**
`hillslope_extent_scale(map_width_km, gw, d)` returns `(REF_CELL_KM/cell_km)²`
anchored at 800 km / 2048 — the app's own untouched default, the same
anchor-at-the-default discipline `terrain_detail_k` and `_V3D_RATIO0` use, so
the correction is **exactly 1.0** there and every golden fixture is
bit-identical. Reducing `d` for a coarse map is unconditionally safe. Raising it
is not: at 5 km / 2048 the raw factor is ~25 600x, which would take `d = 0.15`
to 3840 and detonate an explicit FTCS scheme whose stability bound is `d ≤ 0.25`.
Increases are therefore capped at `HILLSLOPE_STABLE_D`.

**That cap is a real ceiling, recorded rather than hidden**: a very fine region
cannot express its full physical diffusion in one pass, and the correct upgrade
is **more passes**, not a larger coefficient — the explicit scheme's own
constraint. Named as the path if it ever matters.

**Parity preserved without touching a single golden.** `d_scale` defaults to
`1.0` at every existing call site, so all five erosion golden suites pass
unedited. One subtlety made this non-obvious: `golden_parity_passes`'
`hillslope_diffuse_case_2` pins the reference at **`d = 0.9`**, already past the
stability wall. A blanket clamp would have broken parity, so the correction may
*reduce* `d` freely but may only *raise* it to the bound, and a `d` the caller
already chose above the bound passes through untouched. Asserted in
`an_already_unstable_d_is_never_reduced_by_the_correction`.

**Reachability.** `passes.hillslope` is `false` by default, so this was a latent
bug, not an active one — and wrong by seven orders of magnitude the moment a
user ticked Hillslope on a region or a world. No new user-facing parameter: the
scale is derived from `map_width_km` and `gw` at the one call site that knows
them.

**Verified:** `cargo test --workspace` 2 626 passed, 0 failed (up from 2 619 —
seven new tests, nothing moved). No golden file edited.

**Two siblings found and deliberately not fixed**, recorded so they are not lost:
`erode_thermal`'s `talus` is a raw normalised height difference across one cell,
so `tan θ = talus · peak_m / cell_km` gives 87° at 5 km, 7.0° at 800 km and
0.14° at 40 000 km against a real scree repose of 30-37°; and the velocity and
glacial kernels were not audited. `erode_thermal` is manual-op only, which is
why it waits.
