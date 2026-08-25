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
