# LOD/tiling integration — scope

> Owner, mid-session (2026-08-18), while a coordinating session builds real
> camera zoom/pan interaction directly on the viewport: *"Don't forget to wire
> zoom/pan LOD tiling etc either."*

That sentence names two things and this document is about the second one only.
Real camera zoom/pan on `ViewportHost`/`map_overlay.gd` was the coordinating
session's work and is not re-planned here. What this document scopes is the
other half: does "zoom/pan" also need *tiling*, and if so, which kind, and how
much.

**This document defines tiers Z1–Z5 and milestones M0–M3 and records the
reasoning behind them. It does not track them** — status is
`cartalith-native/docs/STATUS.md`'s. Its sections describing "what exists"
describe the tree **as scoped, on 2026-08-18**, because that baseline is what the
milestones were cut against. `LOD_DETAIL_SCOPE.md` (LOD-D0…D7) extends M1's
deep-zoom tiles toward the owner's Aletsch target and does not re-scope anything
here.

## This is a boundary being moved on purpose, not a bug being fixed

`LOD_TILING_BASE_SCOPE.md` recorded a deliberate decision on 2026-08-17: given
three options (data structures only / + dirty-region scaffolding / start
threading tiles through the live pipeline now), the owner chose the narrowest,
building `cartalith-spatial` standalone with "no camera, no quadtree-driven
rendering, no clipmaps, no GPU residency, no interactive painting", to be
integrated when "a concrete need appears" (`ROADMAP.md`'s "Not a phase: LOD and
large worlds").

That trigger had already fired once, and not the way anyone expected: the base
scope's *What integration found (2026-08-18)* section records that
`cartalith-spatial` gained its first real dependent — `cartalith-engine`, via
`PassBuffer`/`StageGraph` — through the DCC tool system's milestone A, not
through LOD or rendering. None of its types had any consumer in rendering or
camera code. The owner's remark is a second, independent knock on the same
door, arriving through the "revisit when a concrete need appears" clause
exactly as designed. This document takes it as seriously as that clause
intends: by checking, with real numbers, whether a concrete need has appeared,
rather than assuming the question answers itself because it was asked.

`TERRAIN_ARCHITECTURE_RESEARCH.md`'s framing note was already aging when this
was written: it describes Cartalith as "a one-shot batch generator … No camera,
no continuous render loop, no interactive terrain editing." The sculpt system
(`SCULPT_LIVE_SCOPE.md`) had made the third clause false, and the camera work
was making the first two false.

## What "LOD tiling" actually has to mean here

Everyone who has used "Tiled LOD" about this project — the owner's remark,
`DCC_SHELL_SPEC.md`'s Preferences panel, the reference HTML's `_lodOn` — is
pointing at some mix of five different things. Separating them is most of this
document's work, as separating L1/L2/L3 was most of `SCULPT_LIVE_SCOPE.md`'s.

| Tier | What it actually is | Needs from `cartalith-spatial` | Scoped as |
|---|---|---|---|
| **Z1 · Basic zoom/pan** | The *same* single whole-grid raster, viewed through a camera transform — what any `Camera2D`/viewport-transform setup gives once it exists | **None** — GPU compositing over one already-uploaded texture | **M0** (verify only) |
| **Z2 · Deep-zoom detail synthesis** | Texture *finer than the base grid*, once the camera passes roughly one screen pixel per grid cell — the reference's `_lodOn`/`drawLODView()`, and what `DCC_SHELL_SPEC.md`'s "Tiled LOD: auto on zoom" means | A tiled compositor plus a synthesis pass (fractal/noise amplification), not raw storage tiling | **M1** — the one real new milestone; extended by `LOD_DETAIL_SCOPE.md` |
| **Z3 · Streaming/scale tiling** | Splitting the *base* raster into tiles because holding or rendering it whole is the bottleneck | The (since retired) `TiledField`/`QuadTree` | **Out of scope** by §1's numbers |
| **Z4 · Tile-pyramid export** | Writing a Leaflet/GIS-style `{z}/{x}/{y}` tile set to disk — an output format, not a live view | None — `cartalith-io` / `cartalith-engine::region_export` | **M2** — engine-complete at scoping; a UI panel only |
| **Z5 · Atlas / bake cache** | Persisting a Z2 bake so re-opening a project has instant deep zoom, mirroring the reference's IndexedDB atlas | Serialisation of whatever it stores | **M3**, deferred until M1 proves itself |

The owner's sentence reads most naturally as Z1 + Z3, because that is the shape
"pan a big map, tiled so it's fast" takes in most rendering literature and in
`TERRAIN_ARCHITECTURE_RESEARCH.md`'s own 9-phase roadmap (built for a real-time
3D camera this product does not have). But the features already named elsewhere
under the same words — `DCC_SHELL_SPEC.md`'s Preferences panel, the reference's
`_lodOn` — are Z2/Z4/Z5. That mismatch is the headline finding, argued below
rather than asserted: **Z3 is not what this port's "Tiled LOD" language has ever
meant, and it is not what the numbers say is needed. Z2 is the gap real usage
surfaces.**

## 1 · The assumption this scope would otherwise rest on, checked against real numbers

The unexamined version of this task is "an 8192² map is huge, so it probably
needs tiling." That deserves the treatment `SCULPT_LIVE_SCOPE.md`'s L0 gave the
"~7 s/stroke" figure it was nearly built on: check it first.

**Texture size, computed directly.** `new_world_dialog.gd`'s `RESOLUTION_PRESETS`
(512/1024/2048/4096/8192, `GRID_MAX := 8192`) are this port's target range.
`WorldGen::build_color_texture` allocates one `Format::RGB8` buffer at `gw*gh*3`
bytes; the three overlay builders (`build_territory_texture`,
`build_province_boundary_texture`, `build_paint_preview_texture`, all
`Format::RGBA8`) allocate `gw*gh*4`:

| Resolution | RGB8 (colour texture) | RGBA8 (each overlay) |
|---:|---:|---:|
| 512² | 0.75 MiB | 1 MiB |
| 1024² | 3 MiB | 4 MiB |
| 2048² | 12 MiB | 16 MiB |
| 4096² | 48 MiB | 64 MiB |
| 8192² | **192 MiB** | **256 MiB** |

Even the worst realistic case — colour, territory, province and paint-preview
all GPU-resident at the 8192 ceiling — is under 1 GiB (192+256+256+256 =
960 MiB), and `ViewportHost._ready()` started `territory_view`/`province_view`
hidden, so that worst case is not the default. The project's own measured
*field-data* memory dwarfs it (below).

**Godot's ceiling, checked rather than assumed.** Godot enforces a hard 16384
per-dimension texture limit regardless of renderer, and desktop and flagship
mobile GPUs commonly support that or more. This port renders through GL
Compatibility / OpenGL ES 3.2 (`project.godot`'s `renderer/rendering_method`),
and `ANDROID_BUILD_SCOPE.md` has a real device (an Adreno 630) driving that
backend. 8192 is half Godot's ceiling per axis, and 192 MiB is not close to any
known limit on `Image::create_from_data`'s `PackedByteArray`. **The one open
question at scoping:** the Android device pass had only exercised the default
512×512, so "flagship mobile GPUs handle 8192 comfortably" was inference from GPU
generation and ES 3.2 minimums, not a measurement; budget Android GPUs are where
community reports put a real 4096 ceiling. That is an `ANDROID_BUILD_SCOPE.md`
verification, not a tiling question — tiling would not fix a hardware ceiling,
only make the symptom appear later. M0 carries it.

**Render cost, from numbers that already existed.** No benchmark measured
`build_color_texture` or `build_sculpt_preview_texture` above 2048² —
`sculpt_live_l0_bench.rs` stops there deliberately ("this project's standing
benchmark sizes"), and `CPU_MULTITHREADING_SCOPE.md` covers generation only.
`SCULPT_LIVE_SCOPE.md`'s L0 table scales close to ideal `O(gw·gh)` (~4.2–4.7×
per doubling on the constructor, ~3.8–3.9× on the per-pixel loop), so two more
doublings is a reasonable **estimate, flagged as one**:

| Stage | 2048² (measured) | 4096² (extrapolated ×~4.2) | 8192² (extrapolated ×~4.2 again) |
|---|---:|---:|---:|
| `with_appearance` ctor | 296.03 ms | ~1.24 s | ~5.2 s |
| per-pixel colour loop | 100.93 ms | ~0.39 s | ~1.5 s |
| local contrast (Quality tier) | ~30-53 ms | ~0.16 s | ~0.6 s |
| **`build_color_texture`, estimated total** | ~427-450 ms | **~1.8 s** | **~7 s** |

Two things matter about ~7 s besides its size. It is a **one-time cost** paid on
generation, sculpt commit or a quality-tier change — once
`ImageTexture::create_from_image` has run, Z1 pan/zoom is GPU compositing over a
resident texture at any grid size. And it is a rendering-cost question
(`GPU_LAYER_INTEGRATION_SCOPE.md`'s shape — no GPU path existed for
`smooth_sea_h`/`build_ao`/`build_hydro_wetness`, per L0's table header), not a
tiling one: tiling the render would change *when* the work happens, not how much
of it there is.

**Memory, from a measured baseline.** `MEMORY_OPTIMIZATION_SCOPE.md` measured
~689–691 MB steady-state at 2048² (`Get-Process` sampling) and extrapolated
"roughly ×4 at 4096² (~1.6 GB), ×16 at 8192² (~6+ GB)". That is dominated by
field data (`WorldState`/`CivData`'s many `gw*gh` arrays), not by the ≤1 GiB of
raster textures above. So tiling the *render* does not touch the real cost
driver, and multi-resolution *generation* is exactly what
`LOD_TILING_BASE_SCOPE.md` ruled out as "a pipeline-wide numerical-parity
change … not a free architectural win". That ruling stands.

**Conclusion.** Nothing measured or computed here supports Z3 at 512–8192. The
single-raster approach is cheap enough at every targeted resolution that Z1 is
not blocked on any `cartalith-spatial` integration. The triggers that would
change this: a resolution ceiling well past 8192 (the *source* project's
`docs/WORLD_REGIONAL_TILING_PLAN.md` names 16384 as its aspiration, but that is
not a commitment of this port — `CLAUDE.md`'s naming-hazard note), or a
multi-GPU dispatch feature (`DCC_CONTROL_INDEX.md` §2.5's "Multi-GPU mode: split
tiles", itself marked "owner decision before any scoping").

## 2 · How the reference — and the DCC design copying its language — solve this

The most relevant comparable tool is Cartalith's own JS predecessor, whose
feature this project's "Tiled LOD" language was named after.
`docs/research/save-export-architecture-audit.md` (the source project's audit,
read here only as fact about what was built) found the reference's "Tiles &
LOD" accordion is **three unrelated features** under one label:

| Sub-feature (reference) | Trigger | What it produces | Maps to |
|---|---|---|---|
| Live preview (`_lodOn`, `drawLODView()`) | Zooming in past a threshold | A rendering *mode* — nothing persisted | **Z2** |
| Atlas (IndexedDB bake) | Manual "Bake" / "Bake ALL levels" | Chunk PNGs cached across sessions, keyed by world | **Z5** |
| Export tile grid (`exportRegionTiles`) | Manual "Refine & export" on a selection | A standalone downloadable `.zip` | **Z4** |

`docs/HANDOFF.md` records why Z2 exists: a real owner complaint — *"There is
still a certain pixilated quality to the map when we zoom. The graphics should
be finer than that"* — on a 20 000 km / 2048 px world (9.77 km/cell), fixed by
making `addZoomDetail`'s noise frequency scale with `cellKm` instead of
defaulting to `1.0`. In its words, *"the LOD viewer's whole job is showing
texture finer than the base grid can."* That is content synthesis, not
streaming. It is also exactly what this port's viewport reproduces the moment
deep zoom is possible: `viewport_host.gd::_raster()` sets
`CanvasItem.TEXTURE_FILTER_NEAREST`, so zooming a Z1-only viewport past one
screen pixel per cell shows blocky single-cell squares. At the scales this
port's worlds use (the reference default is 0.39 km/cell at 800 km / 2048 px), a
user inspecting one settlement hits it quickly. **Z2, not Z3, is the gap real
usage surfaces once Z1 lands.**

`DCC_SHELL_SPEC.md` copies the three-part bundle's language and its confusions
into Preferences ▸ Tiles & LOD (§5's table): "Tiled LOD: auto on zoom / manual"
(Z2), "Atlas cache: size cap + Clear" (Z5), "Chunk debug overlay" (a debug view
of whichever of Z2/Z3 is built), plus the dock foot's
`Finalize · LOD 0-3 · 85 tiles` counter and §9's Data manager "Export ▸ Maps ▸
Leaflet tile pyramid" route (TILES/PROJECTION/LAYERS/OUTPUT/ESTIMATE — a
standard XYZ/TMS export, i.e. Z4). The catalogue below maps each onto a tier.

## 3 · The starting point, per tier (2026-08-18)

- **Z1** — nothing needed from this crate; the camera work is sufficient by
  construction (§1).
- **Z2** — the *synthesis math* was already ported: `cartalith-terrain::amplify`'s
  `amplify_region`/`refine_tile` (`UNIFIED_TOOL_PLAN.md` milestone E, 16 unit +
  11 golden tests at the time), the direct Rust port of `amplifyRegion`/
  `refineTile`, reachable end to end through `region_export_tiles`'s
  `AmplifyOpts`. **What did not exist:** any camera-driven caller — no `#[func]`
  exposing per-tile synthesis outside the export bundle, and no Godot-side tiled
  compositor.
- **Z3** — `TiledField`/`QuadTree` existed (`LOD_TILING_BASE_SCOPE.md`), but §1
  found no trigger for using them this way. Both were retired on 2026-09-22 with
  no caller.
- **Z4** — the furthest along of all five. `region_set`/`region_get`/
  `region_export_tiles` were real `#[func]`s over
  `cartalith_engine::region_export::{export_region_tiles, zip_region_export}`,
  built on the milestone-E2 pipeline (`cartalith-spatial::region`'s
  `norm_region`/`tile_dims`/`FloatRegion`, `cartalith-terrain::amplify`,
  `cartalith-terrain::tile_render`, `cartalith-io::tiles`' `pack_height16`/
  `TileManifest`) — 18 golden-parity + 61 unit tests, "everything bit-exact with
  no tolerance anywhere". **The only missing piece was the Data manager panel**
  calling `region_export_tiles` — GDScript work, not LOD-tiling engineering.
- **Z5** — nothing. `DCC_CONTROL_INDEX.md` §2.5: "No atlas cache exists in any
  form." No file format, eviction policy or cache-key scheme
  (world + seed + tile + level).

**A name that is easy to misread:** `infra_tools_bridge.rs`'s `REGION_LOD_GRIDS`
(`("low",1,1), ("medium",2,2), ("high",4,4)`) was, at scoping, the only thing
called "LOD" in the Rust codebase, and it is **not a rendering level of
detail** — it is three preset tile-grid densities for the region-export estimate
("~4 tiles at low, ~16 at high"), run through the same `tile_dims` the real
export uses so the estimate cannot disagree with the export. It has nothing to
do with Z1–Z3 and should not be extended to mean something it doesn't.

## Catalogue: the DCC design's Tiles & LOD promise, mapped to tiers

Same table shape `STRANDED_TOOLS.md` used for the tool palette. "At scoping" is
the 2026-08-18 engine state the milestones were cut against.

| DCC promise | Spec ref | v2.10 id | Tier | At scoping | Scoped as |
|---|---|---|---|---|---|
| Preferences ▸ Tiled LOD (`auto on zoom` / `manual`) | §5.1/§2.5 | `#lodAutoChk` | Z2 | synthesis ported, reachable only via export; no viewer | **M1 — the real engine gap** |
| Preferences ▸ Tile size · LOD levels (256/512/1024; levels 0-8) | §5.1/§2.5 | `#lodMaxLevel` | Z2/Z3 params | `region_export` carried its own independent `tile_size` | parameters of M1; nothing to wire until M1 exists |
| Preferences ▸ Atlas cache (size cap + Clear) | §5.1/§2.5 | `#lodBakeBtn`, `#lodClearAtlasBtn` | Z5 | none | **M3**, deferred |
| Preferences ▸ Chunk debug overlay (`off/grid/colours`) | §5.1/§2.5 | `#lodDbgSeg` | debug view of Z2 | none | after M1 — "needs the tiling to be real first" (`DCC_CONTROL_INDEX.md` §2.5) |
| Dock foot `Finalize · LOD 0-3 · 85 tiles` | §5.1 | — | Z2 readout | none | cosmetic until M1 exists |
| Data manager ▸ Export ▸ Maps ▸ Leaflet tile pyramid | §9 | — | Z4 | **complete** (`region_export_tiles`, golden-tested) | **M2 — a UI panel, not an engine gap** |

Four rows were real gaps. The export route was not: it was finished engine work
waiting for a panel, and this document deliberately does not invent it a second
time.

## Milestones

Sequenced, and smaller than the DCC design's full Preferences ▸ Tiles & LOD
panel implies — that panel promises five controls' worth of polish for a
feature (Z2) that did not yet exist interactively. Building the panel before the
feature is the "half-migrated, adds complexity without payoff" trap
`LOD_TILING_BASE_SCOPE.md`'s *Why standalone, not wired in* already named.

**M0 · Confirm Z1 needs nothing, once the camera lands (verification, not new
work).** After the camera's zoom/pan ships, confirm panning and zooming the
single-raster `TextureRect` is smooth at every resolution preset, on the desktop
target and — since §1 flagged it as the one unverified number — a real Android
device at something past 512×512. If it holds, Z1 needs nothing further from
this document.

**M1 · A minimal interactive Z2: tile the deep-zoom case only, not the whole
map.** The one genuinely new milestone. Once zoom exceeds roughly one screen
pixel per grid cell, the visible region switches from sampling the base raster
to compositing `amplify_region`/`refine_tile`-synthesised tiles for just the
visible rect. Needs one new `#[func]` for per-tile synthesis (it existed only
inside `region_export_tiles`) and the Godot-side compositor. **Not in M1:** an
atlas cache (Z5 — nothing to persist yet), an auto/manual toggle (`#lodAutoChk` —
pick one behaviour, ship it, revisit if wrong), or a chunk debug overlay (needs
Z2 first).

*As scoped*, M1 would use `TiledField` as the tile scratch buffer and
`QuadTree::query_region` to resolve visible tiles. **It was built without
either**, and the reason is recorded at `cartalith_godot::lod_bridge`'s module
doc (*"Why not `TiledField`/`QuadTree`"*): the compositor resolves visible chunks
by `cartalith_spatial::pyramid` index arithmetic (`tiles_in_view`,
`pyramid_level_for_zoom`), where a quadtree would first cost an O(field) min/max
scan. The fixed 64-cell tile grid M1 first shipped on was replaced by that
pyramid on 2026-08-24, with the level carried in each tile's key. What a tile
*contains* is `LOD_DETAIL_SCOPE.md`'s subject.

**Lessons from M1's first build (2026-08-19 bug-fix pass, after `59700ab`)** —
two correctness bugs in the shipped compositor, each a rule for any later tile
scheduler:

- **A per-call synthesis cap must have a backlog, or it drops tiles for good.**
  `_update_lod()` capped synthesis at `MAX_LOD_TILES_PER_UPDATE := 48` per call
  (the cap's own doc comment already said a real fix was out of scope), and
  `_apply_lod_tiles` only compared the current call's `wanted` against
  `_lod_tiles` — so a dropped tile was never reconsidered once the camera
  stopped: permanently missing texture, not merely delayed. Reproduced headless
  (grid 512, 1920×1080 viewport, 64 tiles wanted): `built` stuck at 48/64 across
  five redundant `_update_lod()` calls. Fixed with a bounded `_lod_backlog`
  drained by `_process()` at `MAX_LOD_TILES_PER_CATCHUP := 6` per idle frame,
  replaced wholesale on every `_update_lod()`; same repro afterwards: 48 built +
  16 backlogged, draining to 64/0 over 3 frames.
- **A tile must be keyed by the detail it was built at.** A tile built at one
  detail tier was never rebuilt when the camera zoomed further within the same
  tile index. The first fix tracked the tier in a parallel `_lod_tile_detail`
  dictionary; since the pyramid (2026-08-24) the level is part of the key and
  that dictionary is retired.

Neither is Z5: both are the compositor correctly finishing work it starts, not
persisting it across sessions.

**M2 · Nothing — the Data manager export panel, not a new milestone.** Z4 was
engine-complete at scoping. The Data manager's Leaflet tile-pyramid route is a
GDScript panel over three already-tested `#[func]`s, and belongs to the GUI work
(`DCC_SHELL_SCOPE.md`), not to `cartalith-spatial`. Recorded so nobody
re-derives it as open engine work.

**M3 · Atlas cache (Z5), deferred until M1 ships and is kept.** Persisting a Z2
bake needs a file-format and eviction-policy decision this document does not
make, because building storage for a feature that does not exist yet is the
dead-weight risk `LOD_TILING_BASE_SCOPE.md` already argued against. Revisit once
M1 is real and its usage shows whether re-synthesising on every zoom is worth
caching against, or cheap enough (GPU-resident, not disk-resident) that a cache
buys nothing.

## Out of scope — the owner's chosen boundary, restated for this pass

- **Z3, streaming/tiling the base raster.** Not triggered by any number in §1 at
  512–8192. Revisit only if the resolution ceiling moves well past 8192, or the
  "Multi-GPU mode: split tiles" row (`DCC_CONTROL_INDEX.md` §2.5, "owner decision
  before any scoping") becomes real. The `TiledField`/`QuadTree` it would have
  used were retired on 2026-09-22 with no caller (`LOD_TILING_BASE_SCOPE.md`), so
  Z3 would start by recovering them from git history.
- **Multi-resolution generation** (fields cheaper than the height grid).
  `LOD_TILING_BASE_SCOPE.md`'s boundary, unchanged.
- **A GPU compute path for `build_color_texture`/`with_appearance`.** The
  extrapolated ~7 s at 8192 (§1) is real and worth someone's attention, but it is
  `GPU_LAYER_INTEGRATION_SCOPE.md`'s question — tiling would relocate that cost,
  not remove it.
- **The Data manager UI panel for Z4.** GUI work, not this document's (M2).
- **Atlas cache (Z5) before M1 exists.** See M3.
- **Anything resembling `TERRAIN_ARCHITECTURE_RESEARCH.md`'s full 9-phase
  roadmap** — clipmaps, out-of-core GPU-residency paging, a camera-navigable 3D
  terrain engine. That document itself calls this Phase-3-or-later territory, and
  nothing in the owner's remark asks for a 3D engine. (`LOD_DETAIL_SCOPE.md`'s
  owner question 5 asks whether its LOD-D3 colour-space morph, a 2D shader blend
  with no geometry or paging, falls inside this bullet.)

## Sequencing

M0 (verify, gated on the camera work) → M1 (the one real new milestone — a
minimal interactive Z2) → M3 (deferred, gated on M1 landing and proving itself
worth caching). M2 is not sequenced because it is GUI work, independent of
everything above. Z3 stays out of scope by the numbers in §1, not by assertion.
