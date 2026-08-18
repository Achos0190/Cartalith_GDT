# LOD/tiling integration — scope

> Owner, mid-session (2026-08-18), while a coordinating session builds real
> camera zoom/pan interaction directly on the viewport: *"Don't forget to wire
> zoom/pan LOD tiling etc either."*

That sentence names two things and this document is about the second one only.
Real camera zoom/pan on `ViewportHost`/`map_overlay.gd` is the coordinating
session's work, not this document's — it is not re-litigated or re-planned
here. What this document scopes is the other half of the sentence: does
"zoom/pan" also need *tiling*, and if so, which kind, and how much.

## This is a boundary being moved on purpose, not a bug being fixed

`LOD_TILING_BASE_SCOPE.md` recorded a deliberate decision on 2026-08-17, given
three concrete options (data structures only / + dirty-region scaffolding /
start threading tiles through the live pipeline now): the owner chose the
narrowest of the three. `cartalith-spatial`'s `TiledField<T>`/`QuadTree<T>`/
`DirtyTracker` were built standalone, "no camera, no quadtree-driven
rendering, no clipmaps, no GPU residency, no interactive painting," and the
document closes by saying integration should happen "whenever Phase 3
(3D, `ROADMAP.md`) or a real large-world need starts actual integration" —
quoting `ROADMAP.md`'s own "Not a phase: LOD and large worlds" section:
*"Revisit when a concrete need appears rather than building it
speculatively."*

That trigger already fired once, and not the way anyone expected. The base
scope's own "Integrated (2026-08-18)" section records that `cartalith-spatial`
gained its first real dependent — `cartalith-engine`, via `PassBuffer`/
`StageGraph` — through the DCC tool system's milestone A, not through LOD or
rendering. `TiledField`/`QuadTree`/`DirtyTracker` still have **zero**
consumers in any rendering or camera-facing code. The owner's remark this
session is a second, independent knock on the same door: a real request,
arriving through the "revisit when a concrete need appears" clause exactly as
designed, not a course-correction of the 2026-08-17 decision. This document
takes it as seriously as that clause intends — by checking, with real numbers,
whether a concrete need has actually appeared, rather than assuming the
question answers itself just because it was asked.

`TERRAIN_ARCHITECTURE_RESEARCH.md`'s own framing note is worth re-reading here
because part of it is now visibly aging in real time, exactly as this
project's own "expect these documents to age" rule anticipates: it says
Cartalith is "a one-shot batch generator: click Generate, get a static 2D map
image. No camera, no continuous render loop, no interactive terrain editing."
The sculpt system (`SCULPT_LIVE_SCOPE.md`) already made the third clause
false, and the coordinating session's own work is making the first two false
as this document is being written. The research document's own caveat about
itself — "revisit when a concrete need appears" — is doing exactly its job.

## What "LOD tiling" actually has to mean here, precisely

Everyone who has used the term "Tiled LOD" about this project — the owner's
remark, `DCC_SHELL_SPEC.md`'s Preferences panel, and the reference HTML's own
`_lodOn` feature — is pointing at some mix of five genuinely different things.
Separating them is most of this document's work, the same way separating
L1/L2/L3 was most of `SCULPT_LIVE_SCOPE.md`'s.

| Tier | What it actually is | Needs from `cartalith-spatial` | Status today |
|---|---|---|---|
| **Z1 · Basic zoom/pan** | The *same* single whole-grid raster, viewed through a camera transform at different scale/offset — what any `Camera2D`/viewport-transform setup gives for free once it exists | **None** — this is a GPU compositing operation over one already-uploaded texture, not a data-structure question | Coordinating session's work, landing now |
| **Z2 · Deep-zoom detail synthesis** | Showing texture *finer than the base grid can*, once the camera zooms in past roughly one screen pixel per grid cell — the reference's own `_lodOn`/`drawLODView()`, and what `DCC_SHELL_SPEC.md`'s "Tiled LOD: auto on zoom" actually means | A tiled compositor + a synthesis pass (fractal/noise amplification), not raw storage tiling | Math ported (`amplify_region`/`refine_tile`), **no interactive viewer exists** |
| **Z3 · Streaming/scale tiling** | Splitting the *base* raster itself into tiles because rendering or holding it whole is the bottleneck at very large resolutions | `TiledField`/`QuadTree` exactly as built | **Not triggered** by this port's real resolutions — see §1 below |
| **Z4 · Tile-pyramid export** | Writing a Leaflet/GIS-style `{z}/{x}/{y}` tile set to disk for external consumption — an output format, not a live view | None — a different subsystem (`cartalith-io`/`cartalith-engine::region_export`) | **Engine- and Godot-binding-complete**, golden-tested; only a GDScript UI panel is missing, and that panel is under the current UI hold |
| **Z5 · Atlas / bake cache** | Persisting a Z2 deep-zoom bake to disk so re-opening a project has instant deep zoom, mirroring the reference's IndexedDB atlas | `serde` round-trip already exists on the base types | **Zero engine equivalent of any kind** |

The owner's one sentence — "zoom/pan LOD tiling" — reads most naturally as Z1
+ Z3, because that is the shape "pan around a big map, and tile it so that's
fast" takes in most rendering literature (and in
`TERRAIN_ARCHITECTURE_RESEARCH.md`'s own 9-phase roadmap, which is about
exactly that shape, for a real-time 3D camera this product does not have yet).
But the concrete features already named elsewhere in this project under the
same words — `DCC_SHELL_SPEC.md`'s Preferences panel, the reference's own
`_lodOn` — are Z2/Z4/Z5, not Z3. That mismatch is the headline finding this
document has to argue for, not assert: **Z3 is not what this port's own
"Tiled LOD" language has ever meant**, and it is not what the real numbers
say is needed. What real usage will actually surface a gap for is Z2.

## 1 · The assumption this scope would otherwise rest on, checked against real numbers

The unexamined version of this task is "an 8192² map is huge, so it probably
needs tiling." That deserves the same treatment `SCULPT_LIVE_SCOPE.md`'s L0
gave the "~7s/stroke" figure it was nearly built on: check it before building
anything on top of it.

**Texture size, computed directly.** `new_world_dialog.gd`'s own
`RESOLUTION_PRESETS` (512/1024/2048/4096/8192, `GRID_MAX := 8192`) are this
port's real target range. `build_color_texture()` (`lib.rs:1574`) allocates
one `Format::RGB8` buffer at `gw*gh*3` bytes; the three overlay builders
(`build_territory_texture`, `build_province_boundary_texture`,
`build_paint_preview_texture`, all `Format::RGBA8`) allocate `gw*gh*4`:

| Resolution | RGB8 (colour texture) | RGBA8 (each overlay) |
|---:|---:|---:|
| 512² | 0.75 MiB | 1 MiB |
| 1024² | 3 MiB | 4 MiB |
| 2048² | 12 MiB | 16 MiB |
| 4096² | 48 MiB | 64 MiB |
| 8192² | **192 MiB** | **256 MiB** |

Even the worst realistic case — colour + territory + province + paint-preview
all GPU-resident simultaneously at the 8192 ceiling — is under 1 GiB
(192+256+256+256 = 960 MiB), and `ViewportHost._ready()` (`viewport_host.gd:
58-65`) starts `territory_view`/`province_view` hidden, so that worst case is
not the default. This is smaller than a single uncompressed 4K video frame
buffer times a handful, on a project whose own measured *field-data* memory
already dwarfs it (below).

**Godot's own ceiling, checked rather than assumed.** Godot enforces a hard
16384-per-dimension texture limit regardless of renderer, and real GPUs
(desktop and flagship mobile alike) commonly support that or more; this
port's own renderer is GL Compatibility / OpenGL ES 3.2
(`project.godot:40-41`), and `ANDROID_BUILD_SCOPE.md` already has a real
device (a Qualcomm Adreno 630) driving that exact backend successfully.
8192 is exactly half of Godot's own ceiling per axis. `Image::create_from_data`
takes a flat `PackedByteArray`; 192 MiB is not close to any known limit on
that type. **The one real open question**: `ANDROID_BUILD_SCOPE.md`'s own
device pass only exercised the UI's default 512×512 (`ANDROID_BUILD_SCOPE.md`
line 111-113) — nobody has actually generated at 8192 on that device or any
other Android hardware, so "flagship mobile GPUs handle 8192 comfortably" is
inference from GPU generation and documented ES 3.2 minimums, not a verified
number the way the 512×512 golden path is. Budget-tier Android GPUs are the
one place community reporting suggests a real 4096 ceiling can bite. Worth a
real device check before shipping 8192 as a supported preset on Android, but
that is a `ANDROID_BUILD_SCOPE.md`-shaped verification task, not a tiling
question — tiling would not fix a hardware ceiling, it would just make the
symptom appear later.

**Render cost, from the numbers that already exist rather than a new bench.**
No existing benchmark measures `build_color_texture` or
`build_sculpt_preview_texture` above 2048² — `sculpt_live_l0_bench.rs`'s own
methodology stops there deliberately ("this project's standing benchmark
sizes"), and `CPU_MULTITHREADING_SCOPE.md` never measures rendering at all
(it is generation-pipeline only). What does exist is `SCULPT_LIVE_SCOPE.md`'s
own L0 table, and its scaling is close enough to ideal `O(gw·gh)` (~4.2-4.7×
per doubling on the constructor, ~3.8-3.9× on the per-pixel loop) that
extrapolating two more doublings is a reasonable estimate, clearly flagged as
one:

| Stage | 2048² (measured) | 4096² (extrapolated ×~4.2) | 8192² (extrapolated ×~4.2 again) |
|---|---:|---:|---:|
| `with_appearance` ctor | 296.03 ms | ~1.24 s | ~5.2 s |
| per-pixel colour loop | 100.93 ms | ~0.39 s | ~1.5 s |
| local contrast (Quality tier) | ~30-53 ms | ~0.16 s | ~0.6 s |
| **`build_color_texture`, estimated total** | ~427-450 ms | **~1.8 s** | **~7 s** |

Two things matter about that ~7s figure, not just its size. First, it is a
**one-time cost paid on generation, on sculpt commit, or on a quality-tier
change** — not a per-frame or per-pan/zoom cost. Once `ImageTexture::
create_from_image` has run once, panning and zooming that texture via Z1 is
GPU compositing over an already-resident texture and costs nothing extra
regardless of grid size. Second, ~7s at the resolution ceiling is a real
number worth someone owning, but it is a rendering-cost question
(`GPU_LAYER_INTEGRATION_SCOPE.md`'s shape — no GPU path exists for
`smooth_sea_h`/`build_ao`/`build_hydro_wetness`, confirmed by L0's own table
header) not a tiling question. Tiling the render would not make the total
work smaller; it would only change when each piece of it happens.

**Memory, from a real measured baseline.** `MEMORY_OPTIMIZATION_SCOPE.md`
measured real process memory at 2048² (~689-691 MB steady-state, `Get-Process`
sampling, not estimated) and extrapolated "roughly ×4 at 4096² (~1.6 GB),
×16 at 8192² (~6+ GB)." That number is dominated by field data (`WorldState`/
`CivData`'s many `gw*gh` `Vec<f32>`/`Vec<i32>` arrays), not by the ≤1 GiB of
2D raster textures computed above. This matters for scoping: if the real
memory pressure at large resolutions is the *field* data, tiling the
*render* doesn't touch the actual cost driver — and multi-resolution
generation (fields computed cheaper than 1:1 with the height grid) is exactly
what `LOD_TILING_BASE_SCOPE.md`'s own "out of scope" section already ruled
out as "a pipeline-wide numerical-parity change... not a free architectural
win." That ruling still holds and this document does not reopen it.

**Conclusion of this section, stated plainly**: nothing measured or computed
here supports Z3 (streaming/tiling the base raster because it is itself the
bottleneck) at 512-8192. The single-raster approach is cheap enough, at every
resolution this port targets, that basic zoom/pan (Z1) is not blocked on any
`cartalith-spatial` integration at all. If this conclusion should ever
change, the trigger is a resolution ceiling well past 8192 — the source
project's own `docs/WORLD_REGIONAL_TILING_PLAN.md` names 16384 as its own
aspirational target, but that is the *source project's* documentation, not a
commitment this port has made (`CLAUDE.md`'s own naming-hazard note) — or a
GPU/multi-device dispatch feature (`DCC_CONTROL_INDEX.md` §2.5's "Multi-GPU
mode: split tiles" row, itself flagged "owner decision before any scoping").
Neither is true today.

## 2 · How the reference itself — and the DCC design copying its language — actually solve this

The most relevant "comparable tool" here is not an external DCC app; it is
Cartalith's own JS predecessor, whose exact feature this project's "Tiled
LOD" language was named after. `docs/research/save-export-architecture-audit.md`
(the source project's own architecture audit, read here only as historical
fact about what was built, not as this port's plan) found that the
reference's "Tiles & LOD" accordion is actually **three unrelated features**
bundled under one label:

| Sub-feature (reference) | Trigger | What it produces | Maps to |
|---|---|---|---|
| Live preview (`_lodOn`, `drawLODView()`) | Zooming in past a threshold | A rendering *mode* — nothing persisted | **Z2** |
| Atlas (IndexedDB bake) | Manual "Bake" / "Bake ALL levels" | Chunk PNGs cached across sessions, keyed by world | **Z5** |
| Export tile grid (`exportRegionTiles`) | Manual "Refine & export" on a selection | A standalone downloadable `.zip` | **Z4** |

`docs/HANDOFF.md` records exactly why the live-preview half (Z2) exists: a
real owner complaint — *"There is still a certain pixilated quality to the
map when we zoom. The graphics should be finer than that"* — on a
20,000km/2048px world (9.77 km/cell), fixed by making `addZoomDetail`'s
synthetic-noise frequency actually scale with `cellKm` instead of defaulting
to `1.0`. The document's own framing is precise: *"the LOD viewer's whole job
is showing texture finer than the base grid can."* That is a content-synthesis
problem — adding plausible detail beyond what the simulation actually
computed — not a streaming/scale problem. It is also exactly the failure mode
this port's own viewport is set up to reproduce the moment deep zoom becomes
possible: `viewport_host.gd`'s `_raster()` sets
`CanvasItem.TEXTURE_FILTER_NEAREST` (`viewport_host.gd:152`), so zooming a
Z1-only viewport in past one screen pixel per grid cell will show visibly
blocky single-cell squares, not a graceful degrade — the reference's exact
complaint, on this port's exact rendering path, waiting for someone to zoom
in far enough to trigger it. Given that most of this port's worlds are
described in hundreds to tens of thousands of km at a few thousand cells
(the same "0.39 km/cell at 800km/2048px" scale the reference's own defaults
use), a user inspecting a single settlement at street-adjacent zoom will hit
this quickly. **Z2, not Z3, is the real gap real usage will surface once Z1
lands.**

`DCC_SHELL_SPEC.md` copies this three-part bundle's language and its
confusions near-verbatim into Preferences ▸ Tiles & LOD (§5's own table,
`DCC_SHELL_SPEC.md:229-232`): "Tiled LOD: auto on zoom / manual" (Z2),
"Atlas cache: size cap + Clear" (Z5), "Chunk debug overlay" (a debug view of
whichever of Z2/Z3 turns out to be built), plus the dock foot's own
`Finalize · LOD 0-3 · 85 tiles` counter (`DCC_SHELL_SPEC.md:451`) and §9's
Data manager "Export ▸ Maps ▸ Leaflet tile pyramid route" (`DCC_SHELL_SPEC.
md:89`, detailed at lines 644-664 — TILES/PROJECTION/LAYERS/OUTPUT/ESTIMATE,
a standard XYZ/TMS tile-pyramid export, i.e. Z4). None of Preferences' four
rows and none of the Data manager route are wired to any UI today — see the
catalogue below — but three of the four rows name a feature (Z2, Z5, and the
debug overlay of whichever is real) this document can now describe precisely
instead of leaving as a vague "LOD" placeholder.

## 3 · What already exists, per tier

- **Z1**: nothing needed from this crate; the coordinating session's camera
  work is sufficient by construction (§1).
- **Z2**: the *synthesis math* already exists and is already ported —
  `cartalith-terrain::amplify`'s `amplify_region`/`refine_tile`
  (`UNIFIED_TOOL_PLAN.md` milestone E's own table, `cartalith-terrain/src/
  amplify.rs`, 16 unit + 11 golden tests) is the direct Rust port of the
  reference's `amplifyRegion`/`refineTile`, and it is already reachable
  end-to-end via `region_export_tiles`'s `AmplifyOpts` (`lib.rs:3714-3720`).
  What does **not** exist: any interactive, camera-driven caller of it — no
  `#[func]` exposes amplify/refine as a standalone per-tile synthesis call
  outside the export bundle, and no Godot-side tiled compositor or
  quadtree-driven viewport exists at all. `TiledField`/`QuadTree` are exactly
  the shape a Z2 compositor would want (a scratch buffer for synthesized
  tiles, a spatial index for "which synthesized tiles does the current
  viewport rect touch"), and neither has ever been asked that question.
- **Z3**: `TiledField`/`QuadTree` exist and are real (`LOD_TILING_BASE_SCOPE.
  md`, "Done 2026-08-17"), but §1 above found no real trigger for using them
  this way.
- **Z4**: complete, further than either of the other two. `region_set`/
  `region_get`/`region_export_tiles` are real `#[func]` methods (`lib.rs:
  3611-3732`) over `cartalith_engine::region_export::{export_region_tiles,
  zip_region_export}`, themselves built on the milestone-E2 pipeline
  (`cartalith-spatial::region`'s `norm_region`/`tile_dims`/`FloatRegion`,
  `cartalith-terrain::amplify`, `cartalith-terrain::tile_render`,
  `cartalith-io::tiles`' `pack_height16`/`TileManifest`) — 18 golden-parity +
  61 unit tests, "everything bit-exact with no tolerance anywhere"
  (`cartalith-native/docs/STATUS.md`'s milestone E2 entry). The **only**
  missing piece is the Data manager UI panel that would call
  `region_export_tiles` — `menus.gd:153` says so directly ("The window does
  not exist yet") — and that panel is squarely inside the current UI hold
  (`CLAUDE.md`: "All UI work is on hold... it includes the tool system's
  milestone F"). There is no cartalith-spatial/LOD-tiling engineering work
  left to do here; there is a UI panel waiting behind a different, already-
  recorded moratorium.
- **Z5**: nothing. `DCC_CONTROL_INDEX.md` §2.5's own audit is exact: "No
  atlas cache exists in any form." `serde` round-tripping already exists on
  every `cartalith-spatial` type (`LOD_TILING_BASE_SCOPE.md` scope item 4),
  which is necessary for a future on-disk cache but is not itself one — there
  is no file format, no eviction policy, no cache-key scheme (world+seed+
  tile+level), nothing.

One more piece worth naming precisely because its name is easy to
mis-read: `infra_tools_bridge.rs`'s `REGION_LOD_GRIDS` (`("low",1,1),
("medium",2,2),("high",4,4)`, `infra_tools_bridge.rs:424`) is the *only*
thing called "LOD" anywhere in the current Rust codebase. It is not a
rendering level of detail — it is three preset tile-grid densities the
region-export estimate UI would show ("this selection is ~4 tiles at low,
~16 at high"), run through the same `tile_dims` the real export uses so the
estimate can never disagree with what exporting actually produces. It has
nothing to do with Z1-Z3 and should not be extended to mean something it
doesn't.

## Catalogue: the DCC design's Tiles & LOD promise vs. what exists

Same table shape `STRANDED_TOOLS.md` used for the tool palette gap.

| DCC promise | Spec ref | v2.10 id | Maps to tier | Engine reality | Status |
|---|---|---|---|---|---|
| Preferences ▸ Tiled LOD (`auto on zoom` / `manual`) | §5.1/§2.5 | `#lodAutoChk` | Z2 | `amplify_region`/`refine_tile` ported and reachable via export only; no interactive viewer, no standalone `#[func]` | **engine gap — the real one** |
| Preferences ▸ Tile size · LOD levels (256/512/1024; levels 0-8) | §5.1/§2.5 | `#lodMaxLevel` | Z2/Z3 params | `TiledField::tile_size` is a free constructor param; `region_export` carries its own independent `tile_size` | not wired to anything — no Z2 viewer exists to parametrize yet |
| Preferences ▸ Atlas cache (size cap + Clear) | §5.1/§2.5 | `#lodBakeBtn`, `#lodClearAtlasBtn` | Z5 | none | **engine gap**, correctly deferred (§3) |
| Preferences ▸ Chunk debug overlay (`off/grid/colours`) | §5.1/§2.5 | `#lodDbgSeg` | debug view of Z2/Z3 | none | correctly gated — "needs the tiling to be real first" (`DCC_CONTROL_INDEX.md:225`) |
| Dock foot `Finalize · LOD 0-3 · 85 tiles` | §5.1 | — | Z2/Z3 readout | none | cosmetic placeholder until Z2 exists |
| Data manager ▸ Export ▸ Maps ▸ Leaflet tile pyramid | §9 | — | Z4 | **complete** (`region_export_tiles`, golden-tested) | UI panel only, behind the milestone-F UI hold — **not an engine gap** |

Four rows are real gaps. One row (the export route) is not a gap at all —
it is finished work waiting for a UI hold to lift, and this document is
careful not to invent it a second time as if it needed re-scoping.

## Milestones

Sequenced, and honestly smaller than the DCC design's full Preferences ▸
Tiles & LOD panel implies — that panel promises five controls' worth of
polish (auto-vs-manual toggle, a size/levels picker, a cache budget, a debug
overlay) for a feature (Z2) that does not exist in any interactive form yet.
Building the panel before the feature is the "half-migrated, adds complexity
without payoff" trap `LOD_TILING_BASE_SCOPE.md`'s own "why standalone, not
wired in" section already named.

**M0 · Confirm Z1 needs nothing, once the camera lands (verification, not
new work).** After the coordinating session's zoom/pan ships, confirm panning
and zooming the existing single-raster `TextureRect` is smooth at every
resolution preset, on both the desktop target and — since §1 flagged this as
the one real unverified number — a real Android device at something past the
512×512 the existing golden path exercised. If this holds (§1's numbers say
it should), Z1 is done and needs nothing further from this document.

**M1 · A minimal interactive Z2: tile the deep-zoom case only, not the whole
map.** The first genuinely new milestone. Scope: once the camera's zoom
exceeds roughly one screen pixel per grid cell, switch the affected screen
region from sampling the base raster to compositing `amplify_region`/
`refine_tile`-synthesized tiles for just the visible rect — using
`TiledField` as the synthesized-tile scratch buffer and `QuadTree::
query_region` to resolve which tiles the current viewport touches. Needs:
one new `#[func]` exposing amplify/refine synthesis for an arbitrary tile
request (today it only exists bundled inside `region_export_tiles`'s export
path), and the Godot-side compositor itself. Explicitly **not** in this
milestone: an atlas cache (Z5 — nothing to persist a bake into yet), a
UI toggle for auto/manual (`#lodAutoChk` — pick one behavior, ship it,
revisit if it's wrong), or a chunk debug overlay (needs Z2 to exist first,
per the catalogue above).

**M2 · Nothing — the Data manager export panel, not a new milestone.** Z4 is
done. When the UI hold lifts and milestone F resumes, the Data manager's
Leaflet tile-pyramid route is a GDScript panel calling three already-tested
`#[func]` methods, not new engine or `cartalith-spatial` work. Recorded here
only so nobody re-derives it as if it were still open.

**M3 · Atlas cache (Z5), deferred until M1 ships and is kept.** Persisting a
Z2 bake needs a real file-format and eviction-policy decision this document
does not make, because building storage for a feature that doesn't exist yet
is exactly the dead-weight risk `LOD_TILING_BASE_SCOPE.md` already argued
against once. Revisit once M1 is real and its own usage shows whether
re-synthesizing on every zoom is actually a cost worth caching against, or
cheap enough (GPU-resident, not disk-resident) that a cache buys nothing.

## Out of scope — the owner's own chosen boundary, restated for this pass

- **Z3, streaming/tiling the base raster itself.** Not triggered by any
  number in §1 at this port's real 512-8192 range. Revisit only if the
  resolution ceiling itself moves well past 8192, or if the Preferences
  panel's "Multi-GPU mode: split tiles" row (`DCC_CONTROL_INDEX.md` §2.5,
  its own "owner decision before any scoping") becomes real.
- **Multi-resolution generation** (fields cheaper than the height grid).
  `LOD_TILING_BASE_SCOPE.md`'s own boundary, unchanged: "a pipeline-wide
  numerical-parity change... not a free architectural win." Nothing in this
  document's real-numbers section changes that.
- **A GPU compute path for `build_color_texture`/`with_appearance`.** The
  extrapolated ~7s at 8192 (§1) is real and worth someone's attention, but
  it is `GPU_LAYER_INTEGRATION_SCOPE.md`'s shape of question (the render
  itself), not a tiling question — tiling the render would relocate that
  cost, not remove it.
- **The Data manager UI panel for Z4.** Engine-complete; blocked on the
  standing UI hold, not on anything this document could scope differently.
- **Atlas cache (Z5) before M1 exists.** See M3 above.
- **Anything resembling `TERRAIN_ARCHITECTURE_RESEARCH.md`'s full 9-phase
  roadmap** — clipmaps, out-of-core GPU-residency paging, a real-time
  camera-navigable 3D terrain engine. That document's own framing note
  already says this is Phase-3-or-later territory; nothing in the owner's
  "zoom/pan LOD tiling" remark asks for a 3D engine, and this document does
  not manufacture one.

## Sequencing

M0 (verify, gated on the coordinating session) → M1 (the one real new
milestone — a minimal interactive Z2) → M3 (deferred, gated on M1 landing
and proving itself worth caching). M2 is not sequenced because it is not
this document's work — it happens whenever the standing UI hold lifts,
independent of anything above. Z3 stays out of scope by the numbers in §1,
not by assertion.
