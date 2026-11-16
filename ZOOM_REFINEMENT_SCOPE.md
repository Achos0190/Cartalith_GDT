# Zoom refinement: real sub-cell terrain, generated once per tile and cached — scope

**Written 2026-11-16**, from the owner's direction of that day, routed under `OUTSTANDING_WORK.md`'s "World emboss"
row:

> *"the moment each heightmap and subsequent tile has been generated the first time we can just load it in. Or we
> should find another way to render it cheaper after initial generation."*

This document **defines** milestones ZR-0 to ZR-8. It does not track them: status is
`cartalith-native/docs/STATUS.md`'s, and this file has no status column. It changes no product code. Everything
below is **new capability, not a port** (the reference's own per-tile passes are opt-in toggles, §1.6), so under
`DECISIONS.md` §7p it ships behind a switch and its re-baselines need an owner ruling (§8).

**Why it exists.** Ruling FG (`a673a1df`, `LARGE_ITEM_RULINGS.md`) fixed the noise law that adds detail on zoom. Three
of its bars cannot be met by noise, and `STATUS.md`'s FG entry records them as not met:

| FG bar (`WORLD_EMBOSS_FINDINGS_2026-11-15.md` §4) | After FG (`cartalith-native/docs/evidence/world-emboss-fg-2026-11-16/metrics.txt`) | Why noise cannot close it |
|---|---|---|
| **B1** width invariance: `added` on land within 0.7–1.3x of the 1 200 km map at x8 and x32 | maps 5 000 km and wider get 0.35–0.41 of it at x8 | a wide map's terrain is smoother *per cell*; noise bound to that relief stays weaker |
| **B4** a 40 075 km world zoomed to 0.471 / 0.078 km/px vs the 1 200 / 200 km maps at fit: luma gradient within ±30 %, `bind_r` within 0.15 | 0.18–0.23 of the native map's gradient | the native map has real valleys and ridges from orogen + erosion + hydrology at its own cell size |
| **B5** median land gradient non-decreasing from x4 to 0.05 km/px and one step beyond | fails from x4 to x32 | the coarse shading fades faster than the octaves arrive |

The findings say it outright (§2.5): *"A full equivalence, with real dendritic valleys and tributaries at the viewed
scale, is out of reach for any noise law."* This scope is the other half: real landforms, computed per tile from
the parent tile, generated **once** and cached in the atlas store, so the owner's "load it in" holds.

**Standing requirements that bind it:** zoom gains detail at every level (Ruling CI, the hard requirement; judge
every frame at several zooms, never fit view only); GPU/optimised paths are judged by principled equivalence and
visual quality, not array parity; and an engine change away from the legacy HTML is correct when it is the better
engine (owner, 2026-09-23; §7p): the superseded path is scrubbed, not kept as a second path (§4.8).

---

## 1. Inventory, opened at the symbols on `a673a1df` (2026-11-16)

Method: each row was opened in the code in a fresh worktree of `origin/main`, and callers were found with
`grep -rn '<symbol>(' crates --include=*.rs` excluding `/tests/` and comment lines. The documents' own claims were
not taken on trust; where a document and the code disagree, the code is quoted.

### 1.1 The atlas tile store: built, write-only, and not the tile the screen draws

| Piece | Symbol | What the code does |
|---|---|---|
| Store | `cartalith_io::atlas::AtlasStore` (`put`, `get`, `delete`, `keys_for_world`, `world_bytes`, `total_bytes`, `clear_world`, `evict_to`) | Files under a root the shell resolves from `user://`: `<root>/<sanitised world key>/ts<tile size>/LOD<z>/<z>_<col>_<row>.bin` (+ `.png`). `get` writes the access time itself (`touch_access`, because NTFS last-access is off). Holds no state between calls. |
| Shell state | `cartalith_godot::bake_bridge::BakeState` (`root`, `finalized`, `tile_size`), `BakeState::store()` | `None` until `atlas_set_root`; every caller reports "no atlas root" rather than writing to the working directory. |
| World key | `bake_bridge::world_key_signature`, `origin_for_key`, `WorldGen::world_key` | FNV-1a of grid size, seed, map width, sea level, wrap, **origin** (`ORIGIN_GENERATED` / `_IMPORTED` / `_REGION`) and `params::world_key_state`. **Not** a hash of the height field. |
| Chunk format | `cartalith_io::atlas::encode_chunk` → `pack_height16` | **4 bytes per cell, two of them height** (the reference's pre-v2.53 `rg16`). The reference widened this to 24 bit in v2.53 (`RC_ENGINE_CHANGES.md` §6h.1); the port has not. |
| Bake | `WorldGen::{bake_all, bake_visible}` → `WorldGen::run_bake` → `cartalith_engine::bake::{bake_all_tiles, bake_tiles}` | Writes `pyramid_tile` chunks at `bake_bridge::DEFAULT_TILE_SIZE` = **1024 px**, depth capped at `MAX_BAKE_DEPTH` = 6, with `AmplifyOpts::default()` plus seed and sea: the **reference schedule**, not Ruling FG's kilometre law, and no `Roughness`. Synchronous on the main thread (its own doc: *"the shell must show a busy state around it"*). |
| Status / cap | `WorldGen::atlas_status`, `bake_estimate` (19 ms per 1024 px tile, cited in code), `atlas_evict_to` → `AtlasStore::evict_to` | Eviction spans every world in the store, coldest first and deeper first at equal age; it keeps a world's coarsest level (or its warmest single chunk when it has one level). Wired to `Preferences ▸ Tiles & LOD ▸ Atlas cache ▸ Size cap · GB`. |
| Read at draw time | `WorldGen::atlas_tile_png`, `atlas_is_covered` | **Nothing draws from the atlas.** `viewport_host.gd::_build_lod_tile` synthesises every tile; `atlas_tile_png` has no shell caller; `atlas_is_covered` feeds only the chunk-debug overlay. `menus.gd`'s header over `_build_atlas_cache_menu` says why the read cannot be switched on: a baked chunk is the **Relief** colouring at 1024 px, the screen draws the **biome** colouring at 256 px. The reference, by contrast, does read baked heights back into its draw cache (`atlasLoadImg`, v2.71). |
| Sculpt invalidation | `BakeState::check(Mutation::HeightEdit)` in `WorldGen` (sculpt, `erode_bridge.rs`, `region_edit_bridge.rs`) | Only the **finalize lock**: a finalized world refuses height edits. An *un*finalized world can be sculpted after a bake and **its chunks stay under the same key** (the key has no field hash). Stale baked chunks are possible today; they are harmless only because nothing reads them. |

**Verdict: built as a store, partial as a cache.** It has keys, eviction, a cap, export/import and a status
readout. It lacks a reader, the drawn tile's format, a generation-rule version, and edit invalidation. ZR keeps the
store and fills those four gaps (§4.4).

### 1.2 The live LOD pyramid: built, and recomputed on every visit

| Piece | Symbol | What the code does |
|---|---|---|
| Tile height | `cartalith_engine::bake::pyramid_tile_padded_rough` | `refine_tile_padded` (upsample + fbm from the **coarse field**, not from a parent tile), then `add_zoom_detail_km_padded` under FG's law (`opts.km`) or `add_zoom_detail_padded` under the reference schedule. A pure function of world coordinates; the halo is the neighbours' own texels. |
| Tile colour | `lod_bridge::synthesize_tile_rgba_rivers_conformed(_with_parent)` via `LodSnapshot::render_tile_conformed` | Height, then `render::render_biome_tile_rgba` shading; rivers come from the **world network drawn into the tile** (the `rivers` argument), not from per-tile drainage. `TILE_PX` = 256. |
| Depth | `lod_bridge::{BASE_MAX_LEVEL = 10, MAX_LEVEL = 16, DETAIL_FLOOR_KM = 0.015, max_level_for_map}` | Since FG, a 40 075 km / 2048 world reaches level 16. Level 16 is the `f32` position ceiling (`2^24` tile pixels across). |
| Threading | `lod_worker::LodWorker` (`request_conformed`, `generation` counter), `LodSnapshot` | A Rust-side rayon pool; no `Gd` crosses a thread; a world change bumps `generation` and drops stale jobs. |
| Caches | `lod_worker::budget_for_tier` (`cache_tiles` 32 / 32 / 35 / 35 textures), `PLAIN_TILE_CACHE` = 8 | **RAM only.** Panning back within ~35 tiles is free; anything beyond is re-synthesised. Nothing survives a session. |
| Invalidation | `viewport_host.gd::invalidate_lod_tiles` → `_clear_lod_tiles` | Clears **every** live tile (13 call sites in `shell/*.gd`). No dirty rectangle. |
| Cost | FG's entry in `STATUS.md` | 7.2–7.7 ms per tile at level 10, 8.9–9.5 ms at level 16 (cited, not re-measured here). Phone not measured. |

**Verdict: built.** This is the path the FG noise runs on and the fallback ZR shows while a real tile computes.

### 1.3 EF-9 and the morph: built and on by default (Performance excepted)

`cartalith_engine::continuous_lod` (effective level `L(x, zoom)`, `level_weight`), `subdivision::{select_tiles,
conform_plan}`, `elevation::{conform_weight, conform_tile_in_place, conform_colour_in_place,
world_elevation_tile_conformed}`, and `lod_tile.gdshader`'s `morph` / `per_pixel` / `lam_zoom` uniforms. EF-9 ships on
for Balanced, Quality and Ultra (`LodBudget::ef9_refine`, Ruling CL). Two of its pieces are reusable as they stand:
**`conform_weight`** is the edge-taper ZR's seams need (§4.2), and the shader's parent/tile **mix** is the cross-fade
ZR needs when a real tile replaces its noise tile (§4.5).

### 1.4 EF-0 to EF-8

| | What the code has | Verdict |
|---|---|---|
| **EF-0** elevation query | `elevation::{world_elevation_tile, world_sample_elevation, world_amplify_opts}`, `amplify::sample_elevation(_rough)`. The live tile is the same composition (`bake::pyramid_tile`). Seam test `adjacent_tiles_agree_bit_for_bit_on_their_shared_edge`, same-level only. | **built** (noise from the coarse field) |
| **EF-1** tile hydrology | `cartalith_hydrology::tile::{TilePlacement, tile_flow, tile_rivers, tile_channel_thresh}`: fine-resolution D8 routing seeded with the world's discharge as an inflow boundary, in coarse-cell-equivalent units (`1/refine²` per fine cell). Harness `tests/tile_hydrology.rs`. **No non-test caller** (`tile_flow`'s only caller is `tile_rivers`, and `tile_rivers` has none). `TilePlacement` is coarse-cell aligned. | **built, unwired** |
| **EF-2** ridge detail | covered by EF-0 per the research doc; no separate code | n/a |
| **EF-3** tile erosion | `cartalith_erosion::tile::{tile_erode, ring_mask, uniform_area_seed}` over `stream_power_kernel_bounded`, outer ring pinned bit-identical. Harness `crates/cartalith-engine/tests/ef3_tile_erosion.rs` (with `measure_tile_erosion_cost`, `#[ignore]`d). `tile_erode` has **no shipping caller**. | **built, unwired** |
| **EF-4** author edits survive refinement | `DirtyTracker` exists for sculpt drafts (`sculpt_bridge.rs`); no tile-level edit record | **open** |
| **EF-5** cache, don't precompute | the RAM caches of §1.2 only | **open** (this scope) |
| **EF-6** vectors | `cartalith_terrain::vector::{trace_coastline, trace_fault_lines, trace_ridgelines}` at world resolution; called by export, `sample_bridge`, `lib.rs`; never per tile | **built at world scale** |
| **EF-7** settlement binding (Ruling N) | in `cartalith-civ` (`golden_parity_settlement_placement.rs` and others) | **built**; not this scope's |
| **EF-8** tile data structure | no `TileData` type; the tile is a `Vec<f32>` plus RGBA | **open** (§4.4 defines it) |

### 1.5 Local refinement, drainage or erosion that already runs somewhere

- **GF-8's windowed re-erosion runs in the shipped app.** `WorldGen::paint_commit_rock` → `rock_paint::commit` →
  `cartalith_erosion::tile::tile_erode_rock` re-erodes a padded window (`REERODE_PAD_CELLS`) at **world** resolution
  with the ring pinned. It is the one production caller of the tile-erosion machinery. *(The doc comment above
  `tile_erode` still says GF-8 "has no shell caller until GF-8's Rock paint target is built"; that clause is stale.
  Reported here, not fixed: it is product code.)*
- **The manual erosion op** (`cartalith_engine::erode_op`, the reference's droplet `erode()`) runs on the whole field.
- **The stage graph** (`cartalith_engine::staleness::recompute_stale`) re-runs downstream stages after a commit. It
  is the source of the post-sculpt world inputs a tile cache must detect as changed (§4.4).
- **Nothing refines drainage or erosion per LOD tile.** `render::tile_river_seeds` exists but its only caller is a
  test (`#[allow(dead_code)]`, its own doc).

### 1.6 The reference's per-tile passes (v2.71, `reference/Cartalith v2.71 DCC test.html`)

The v2.71 function index is still `OUTSTANDING_WORK.md` §2.8's open step, so these were found by grepping the
function names; **line numbers are v2.71's own** and drift-sensitive, so the names are what to search.

| v2.71 | Line | What it does | In the port |
|---|---|---|---|
| `pyramidTile` | 12980 | `refineTile` → `addZoomDetail` → `featureFieldTile` (v2.52 stamps) → `burnChannels` + `sharpDelta` (under `coarseFlow`) → `featureDetailPass` → `tileErode` (under `microErode`) | the first two only (`bake::pyramid_tile`, whose doc records the rest as "not ported") |
| `burnChannels` | 12622 | **"Burn rivers"**: bilinear coarse flow at each texel, normalised by the world maximum; above `thresh` 0.005 a quadratic channel of width `max(1, widthK·√rel)` (W ∝ Q^0.5, Leopold & Maddock 1953) and depth `burnK·rel`, max-combined, floored at `sea − 0.06`. Seam-safe because it reads only world coordinates. | not ported |
| `sharpDelta` | 12723 | carves coarse-flow local maxima `sharpK` deeper in the delta zone | not ported |
| `tileErode` / `tileMicroErodeKernel` | 12702 / 12663 | **"Micro-erode"**: 400 droplets per tile, edge cells fixed | not ported |
| `featureDetailPass` | 12840 | valleys / fjords / canyons from the coarse order and masks, ramping in from z3 to z6 | not ported |
| `featureFieldTile` | (in `pyramidTile`) | v2.52 sub-cell craters and volcanoes resolved per tile (`RC_ENGINE_CHANGES.md` §8, v2.52 row) | not ported (no Rust file names it) |
| `lodTileOpts` | 13454 | Burn rivers and Micro-erode ride `_lodBurnRivers` / `_lodMicroErode`, **both `false` by default** | — |

Two defects a faithful port would inherit: `tileErode` seeds every tile with the same constant (`opts.seed||12345`
unless a seed is passed, and `pyramidTile` passes the world seed), so **every tile's droplets start at the same
offsets**; and `burnChannels` normalises by the *world's* maximum flow, so a tile's channels depend on the largest
river anywhere. ZR does neither (§4.1).

---

## 2. What "real refinement" means here

A level-`z` tile is **its parent's terrain, upsampled properly, plus the landforms that first become resolvable at
level `z`**: a new band of relief bound to the terrain, channels from drainage re-accumulated at this resolution and
connected to the parent's network, and a light erosion pass that turns the two into valleys. Applied level after
level, the valleys of level `z` are the inputs of `z+1`, so zooming reveals tributaries inside the valleys the
previous level showed, which is what B4 measures and noise cannot produce.

Where it runs: from the first level whose texel is finer than a base cell (on a 40 075 km / 2048 world, level 4,
9.8 km texels) down to a **real-refinement floor** (default: texel ≈ 15–20 m, level 13 there; owner question Q2).
Below the floor the FG noise law continues to `max_level_for_map` exactly as shipped. Above the first refined level
the tile is the EF-0 tile it is today: the world's own generation pipeline already worked those scales.

---

## 3. Measured cost of one refinement step (2026-11-16)

**What was measured.** A scratch bench (`examples/zr_bench.rs` in a throwaway worktree of `a673a1df`, not committed,
deleted afterwards; ZR-0 rebuilds it as a committed harness). It generates the EF-3 harness's own fixture world
(`WorldParams::defaults(256, 192, 20260920)`, 800 km; `generate_terrain`), takes `compute_flow` over the final field
as the coarse flow (EF-3's self-consistent pair), and for three all-land tiles (EF-3's `TILES`) runs one step on a
**288 × 288 padded tile** (256 interior + 16-texel halo; 18 coarse cells at refine 16):

- (a) EF-0 `amplify_region` at the child resolution: today's noise tile, for comparison;
- (b) Catmull-Rom ×2 upsample of a half-resolution parent window (scratch code);
- (c) one gradient-bound value-noise octave at 4 texels (scratch code);
- (d) `tile_flow` (EF-1: fill, route, accumulate with the coarse inflow);
- (e) a `burnChannels` port on the tile's own flow (scratch code);
- (f) `tile_erode` (EF-3) at the world's light-pass iterations (9), and (g) at 3;
- (g2) a Braun–Willett implicit stream-power solve over D8 receivers, 3 passes (scratch code);
- (h) quantise the 256² interior to u16 (per-tile min/max) and gzip four streams;
- (i) write, read back and gunzip one tile file.

Median (min..max) of 15 tiles. PC: this machine, 16 logical cores, release build, run alone. 6T: `cargo ndk -t
arm64-v8a build --release -p cartalith-engine --example zr_bench`, pushed to `/data/local/tmp`, run from `adb shell`,
deleted afterwards; no app data touched. **The 6T figures are noisy**: in two of its four runs another process (the
Google search app) was using 93–113 % CPU, and the single-thread spread is wide because the scheduler moves the
thread between the 2.8 GHz and 1.77 GHz clusters. Read the 6T column as a range, not a point.

| Step, per 288² tile | PC, all cores | PC, 1 thread | 6T, 8 threads (2 runs) | 6T, 1 thread (2 runs) |
|---|---|---|---|---|
| (a) EF-0 noise tile, for comparison | 1.34 (1.08..1.50) | 8.94 (8.92..9.14) | 15.1–15.4 (7.2..37.2) | 65–68 (34..105) |
| (b) bicubic ×2 from parent | 0.87 | 1.25 (0.86..1.46) | 2.7–2.9 | 3.1–3.3 (2.6..24) |
| (c) one detail octave | 0.66 | 0.66 | 3.7–3.9 | 4.0–4.4 (3.4..17) |
| (d) `tile_flow` (EF-1) | 1.05 | 1.03 (1.01..1.08) | 8.8–9.5 | 9.7–11.1 (7.9..35) |
| (e) Burn rivers | 0.63 | 0.63 (0.31..0.87) | 2.4–2.5 | 3.2–3.8 (1.2..4.8) |
| (f) `tile_erode`, 9 iters | 11.06 (9.39..11.60) | 23.85 (22.56..24.56) | 165–178 (74..242) | 121–135 (108..439) |
| (g) `tile_erode`, 3 iters | 8.82 (8.09..9.27) | 18.99 (17.99..19.97) | 119–131 (45..164) | 157–185 (108..334) |
| (g2) Braun–Willett, 3 passes | 4.64 | 4.61 (4.57..4.69) | 46–47 (19..59) | 34–36 (20..55) |
| (h) u16 + gzip, 4 streams | 7.37 | 7.30 (7.00..7.63) | 32–33 (24..38) | 26–28 (23..34) |
| (i) file write + read + gunzip | 0.88 | 0.88 (0.76..1.25) | 1.7–1.9 | 1.4–1.7 (1.4..2.2) |

Milliseconds. Earlier PC runs of the same binary repeated these within their spreads.

**Reading it.**

- **The step without erosion is cheap**: (b)+(c)+(d)+(e) is 3.6 ms on one PC thread and ~20 ms on one 6T thread.
- **EF-3's setup dominates EF-3**, as its own module doc measured (69–74 %): 9 iterations cost 23.9 ms on one
  thread and 3 cost 19.0, so the priority-flood fill and MFD accumulation that run before the first iteration are
  ~17 ms of it. The same routing `tile_flow` does in 1.0 ms. On the 6T, EF-3 is 110–185 ms per tile.
- **A D8 implicit solve over the routing already computed is 4–5x cheaper than EF-3** (4.6 ms vs 19.0 on one PC
  thread; 34–36 vs 157–185 ms on the 6T), most of it three sorts. Whether it carves valleys good enough for B4 is
  not measured here; that is ZR-0's question (Q1).
- **Encoding is not free on a phone**: default-level gzip of four streams costs 26–33 ms on the 6T. Two streams (the
  stored record, §4.4) at a fast level are ZR-0's to measure; the estimate below takes half of (h).

**Composed per-tile estimates** (sums of the rows above, so estimates, not measurements):

| Chain, one thread | PC | 6T |
|---|---|---|
| ZR-light: (b)+(c)+(d)+(e)+(g2)+½(h) | ~12 ms | ~67 ms (min-based ~47) |
| ZR-EF3: (b)+(c)+(d)+(e)+(g)+½(h) | ~26 ms | ~190 ms |
| colouring the tile (unchanged, cited) | 8.9–9.5 ms per tile at level 16 (FG, all cores) | not measured |

**Per view, first visit** (estimate: a 2560 × 1440 view of 256 × 128 tiles is ~143 tiles per level; the 6T's
1080 × 2340 portrait is ~100; PC at ~12 effective threads, 6T at ~4 big cores): ZR-light **~0.14 s per level on the
PC and ~1.7–2.4 s on the 6T**; ZR-EF3 ~0.31 s and ~4.5–6.4 s. A revisit costs only the read (i) plus colouring, or
the read alone with the colour cache (ZR-5).

**Disk.** The stored record (§4.4) measured on these tiles: gzipped u16 height as a residual against the parent's
bicubic prediction **75.7 KB** (69.2..79.1) per 256² tile, u16 log flow **50.3 KB** (47.8..52.5); together 126 KB,
**1.97 bytes per texel**. (Plain u16 height: 124.8 KB; row-delta height: 88.7 KB.) Raw u16 + u16: 256 KB.

For the **40 075 km / 2048 × 1024 world** (tiles 256 × 128 = 32 768 texels → **~63 KB per stored tile**), storing
land and coast tiles only (sea tiles get no refinement, §4.1; land fraction 0.30 is a labelled assumption, Earth's
is 0.29):

| Level | Texel | Tiles (all) | Land tiles | That level | Cumulative |
|---|---|---|---|---|---|
| 4 | 9.8 km | 256 | 76 | 4.7 MB | 7.2 MB |
| 6 | 2.4 km | 4 096 | 1 228 | 76 MB | 102 MB |
| 8 | 612 m | 65 536 | 19 660 | 1.2 GB | 1.6 GB |
| 10 | 153 m | 1.05 M | 314 572 | 19 GB | 25 GB |
| 11 | 76 m | 4.19 M | 1.26 M | 76 GB | 101 GB |
| 12 | 38 m | 16.8 M | 5.03 M | 303 GB | 404 GB |
| **13** | **19 m** | 67.1 M | 20.1 M | **1.2 TB** | **1.6 TB** |
| 14 | 9.6 m | 268 M | 80.5 M | 4.7 TB | 6.3 TB |
| 16 | 2.4 m | 4.29 G | 1.29 G | 76 TB | 101 TB |

So **"generate once, load forever" is a per-visit cache, not a whole-world bake**, below about level 8. What one
user actually touches is small: one dive from level 4 to 13 at one spot is ~10 levels × ~143 tiles ≈ **90 MB**. An
area pre-bake of one 1 200 × 675 km window (0.1 % of the world) down to level 13 is ~90 000 tiles, **~5.4 GB**, about
1.5 min on the PC (ZR-light) and ~25 min on the 6T; to level 12 it is ~1.3 GB.

---

## 4. Design

### 4.1 One refinement step

For a tile `T = (z, col, row)` at a refined level, with `P` its parent at `z − 1`:

1. **Parent window.** The parent's texels under `T` plus a margin, read from the parent and, at a parent edge, its
   neighbours (up to 3 × 3 parents). They are the parents' **stored, quantised** values (step 7), never a fresh f32
   copy, so a child is the same whether its parent came from disk or was just computed.
2. **Upsample ×2.** A separable cubic with no overshoot past the local min/max of its four taps (Catmull-Rom
   clamped, or a monotone cubic; Catmull-Rom alone invents pits and spurs that drainage then routes into). Child
   texel centres sit at parent coordinates `x/2 − 1/4`.
3. **New detail band.** The FG kilometre law's octave(s) whose wavelength first fits this level (`λ` between 2 and
   4 child texels), amplitude from the coarse relief and `Roughness` exactly as `add_zoom_detail_km_padded`
   computes it, sampled in **world texel coordinates** with an integer hash (no transcendental `libm`; §6 R3). It
   reads only world coordinates and the parent, so it is seamless by construction.
4. **Drainage with a halo.** Over the padded tile (interior + halo `H`, default 16 texels): a priority-flood fill
   with an ε gradient (lakes stay where the fill makes them; `LAKE_FILL_RESEARCH.md`), D8 receivers, and
   accumulation seeded on the halo's open boundary with the **parent's stored flow** (bicubic, in physical drained
   area per texel). This is EF-1's inflow condition with the parent in place of the coarse world, so every channel
   the tile draws is the continuation of one its parent already carried, plus tributaries that only resolve here.
   Rain: the coarse `rainfall` sampled at world coordinates (EF-1's header names uniform rain as its approximation;
   this is the obvious next step and costs nothing).
5. **Burn rivers.** The reference's `burnChannels` shape (quadratic cross-section, W ∝ Q^0.5), but on this tile's
   flow in **physical units** (drained area in km², via `km_per_cell` and the level's texel size), with width and
   depth in metres from the existing river law (`cartalith_hydrology::river_half_width_profile`, `channel_disc`),
   not normalised by the world's largest river (§1.6). Floor at `sea − 0.06` as the reference does. `sharpDelta`'s
   delta-zone term is included under the same switch.
6. **Light erosion.** An implicit stream-power solve over step 4's receivers (`K·A^m·S^n`, m = 0.5, n = 1, the
   world's own `StreamPowerParams` scaled by the level's resolution correction, as `tile_erode` scales `k`) for a
   small fixed number of passes; then one thermal/talus pass at the critical slope for the rock
   (`cartalith_erosion::critical_talus`) so ridges do not go sharper than rock stands. Whether this is EF-3's
   `tile_erode` itself or a lighter solver reusing step 4's routing is **Q1**: the two differ 4–5x in cost (§3), and
   one algorithm must be chosen per rule version, for every device (§4.3). Micro-erode droplets are an **option**,
   off by default: their randomness is per tile (seeded from the tile key), and stream power already does what they
   did.
7. **Taper, quantise, emit.** The non-local change (steps 4–6, `Δ = after − before`) is multiplied by
   `elevation::conform_weight(distance to the tile edge, band B)` (B default 16 texels), so it is exactly zero on
   the tile edge and full inside. The flow carried to the children is blended the same way toward the parent's
   upsampled flow. Then height is quantised (per-tile offset + u16 step, or u24; §4.4), and the **quantised** values
   are what the colouriser and the children read.

**Sea tiles** (every texel below sea level in the parent window) are not refined: they are the EF-0 tile, stored as
nothing, which is why the disk table counts land only. **Loaded saves** lack `stress_field`, `resistance_field` and
the other substrate fields (`CivData`'s own doc); erosion then runs with the uniform defaults EF-3's tests use and the
tile record says so (`inputs: "loaded-default"`), never a silent zero.

### 4.2 Determinism and seams

**The tile is a pure function**: `T = F_rules(seed, key, parent window, coarse inputs in T's cone)`. Since each
parent is the same function one level up, by induction **the whole refined pyramid is a pure function of the world**
(its coarse fields, its seed, the rule version). The cache is therefore invisible: a hit and a recompute are the
same bytes, and ZR-1's first bar is exactly that.

What makes it hold in code:

- One implementation per step, called by the worker, the pre-bake and the tests (the `LodSnapshot::render_tile`
  rule: identity by control flow).
- No dependence on thread count: per-tile work is serial or writes disjoint rows; tiles are independent jobs.
- No platform `libm` on the path (integer hashes, `cartalith-jsmath`'s helpers where a transcendental is
  unavoidable), no `f32` reassociation, no FMA contraction (Rust does not contract by default). ZR-1 asserts the
  same tile hash on the PC and the 6T.
- The parent is read in its quantised form (step 1).

**Seams.** The shared edge of two neighbours stays continuous at every level:

1. *Base* (steps 2–3): a function of the parent level and world coordinates. If two parents agree on their shared
   edge, their children's bases agree on theirs, because each child's upsample reads its neighbour parent's texels
   through the window (step 1). Level 3's base is EF-0's tile, which `adjacent_tiles_agree_bit_for_bit_on_their_shared_edge`
   already proves seamless. So, by induction, every level's base is seamless.
2. *Non-local change* (steps 4–6): two neighbours compute drainage and erosion over overlapping halos and get similar
   but not identical answers there. The taper makes the change zero on the edge, so **the edge texels are the base's,
   bit for bit**, and the band blends the two answers smoothly into the base.
3. *Flow* is tapered toward the parent's upsampled flow the same way, so the flow a child hands its own children is
   continuous across every seam too; recursively, it bottoms out in the coarse discharge, which is a world field.

What this costs: within band B a channel is drawn at its parent's resolution, one level softer. ZR-1 measures
whether that reads as a break (bar: channel continuity across seams).

**The cone.** `T` reads coarse inputs only inside its footprint grown by the halos of every ancestor down to the first
refined level. Halos shrink by 2x per level down, so the margin is dominated by the coarsest refined level:
about `2 · (H + 2)` texels of that level, which is ~36 base cells at H = 16. The cone is what makes invalidation exact
(§4.4).

### 4.3 The switch and the rule version (§7p)

- **Switch** `zoom_refine`: `false` in `cartalith_engine::WorldParams::defaults()` (every golden and parity path
  byte-identical by control flow), `true` in `cartalith_godot::params::defaults()` once ZR-7's bars pass, with its
  `PARAMS` and `JS_PATHS` rows (the MISTAKES row for changing generated output). It is a presentation input, so it is
  **not** in `world_key_state`; it is in the refined-tile namespace (§4.4).
- **`ZR_RULES_VERSION`**, a literal beside a committed **golden hash of a fixture tile** at that version. Any change
  to the refinement arithmetic moves the hash, the test goes red, and the fix is to bump both. Mutating the version
  alone must also go red. Old-version tiles are never read; they age out under eviction.
- With `zoom_refine` off, the shipped FG path is untouched: same tiles, same bytes (ZR-7 bar).

### 4.4 Caching in the existing atlas store

**Namespace.** Refined tiles live beside, not inside, the 1024-px bake:
`<root>/<world key>/zr<ZR_RULES_VERSION>-ts256/LOD<z>/<z>_<col>_<row>.zr`. Same `AtlasStore` root and per-world
directory, so `clear_world` (a `remove_dir_all` of that directory) already removes them; a different subtree, so
`keys_for_world`'s `ts<n>` parse skips it and the reference-format export never sees it. **The size cap does not
see them yet:** `world_bytes`, `total_bytes` and `evict_to` all walk `keys_for_world`, so ZR-2 must give them a
second walk over the `zr*` subtree, or refined tiles would grow unbounded under a cap that reports the store as
small. `AtlasStore` gains `put_zr`/`get_zr` rather than a second store.

**Record (`.zr`), little-endian, one file per tile:**

| Field | Size | Note |
|---|---|---|
| magic, `ZR_RULES_VERSION`, `enc` | 8 B | encoding decided by the field, never inferred from bytes (§6h.1's rule) |
| tile key `(z, col, row)`, `w`, `h` | 12 B | so a record is self-describing without a manifest (the existing `get` returns `w: 0, h: 0`) |
| **cone digest** | 16 B | hash of every coarse input cell in the cone (below) |
| `inputs` flags | 2 B | e.g. loaded-save defaults, wrap |
| height offset, step | 8 B | per tile; the step is at most the f32 ULP of the field top (do not exceed the source's precision, §6h.1) |
| height residual vs the parent's bicubic, gzip | ~76 KB at 256², ~38 KB at 256 × 128 (measured, §3) | quantise-before-use (§4.1 step 7) |
| flow, log-u16, gzip | ~50 KB at 256², ~25 KB at 256 × 128 (measured) | the children's inflow |

No RGBA: colour depends on `TerrainAppearance`, which is deliberately outside the world key (`bake_bridge.rs`'s
header). Colour has its own optional cache (ZR-5).

**Keys and validity: a digest, not an invalidation list.** Once per world state, a per-cell input hash grid is built
over every coarse field a refined tile reads (`field`, the coarse flow, `rainfall`, `stress_field`,
`resistance_field`, the `Roughness` inputs, sea level). A tile's **cone digest** is a hash over that grid inside its
cone (§4.2). On a read, the digest is recomputed (tens of microseconds even for a level-4 cone of ~8 000 cells) and
compared; a mismatch is a miss, and the stale file is overwritten by the recompute. This covers:

- **sculpt, carve, paint, erode op, rock paint** (any commit that moves a coarse input), including the far-field
  changes `recompute_stale` makes downstream (a re-routed river moves discharge far from the brush), and nothing
  else: tiles whose cone did not change keep validating;
- **regenerate / parameter change**: the world key changes, the whole namespace goes cold;
- **an import or region resample at an identical parameter tuple**: the origin in the world key separates them.

It does not depend on enumerating callers, which is the failure `MISTAKES.md` records for staleness keys
(*"derive the list from the definition: every argument of the function you guard"*): the digest *is* the list of the
step function's coarse arguments, and ZR-2's bar mutates each one.

As a tidy-up, a commit's changed-cell bounding box (the `DirtyTracker` rectangle already produced) may also delete
the files whose cone intersects it, so dead tiles do not wait for eviction to free their space.

**Eviction.** `AtlasStore::evict_to` gains one rule: **refined tiles go before user bakes** at equal age (they are
recomputable; a bake is something the user asked to keep), and within refined tiles, deeper first at equal age, as
today. The current floor (a world keeps its coarsest level) applies to bakes only. Default cap: the existing
`Size cap · GB` setting, one budget for both (Q3).

**Size per level and in a session**: §3's table (~63 KB per land tile of a 2:1 world; ~90 MB per dive to 19 m texels).

**The 1024-px bake is superseded.** With refined tiles stored in the drawn tile's own format, `bake_all` and
`bake_visible` become "pre-bake refined tiles" (ZR-6), and the reference-format chunk writer is kept only as an
export of the reference's `World/` archive layout if the owner wants it (Q6). That is the §7p rule: scrub the
superseded path rather than dual-path it.

### 4.5 Pipeline: first visit, the morph, and revisits

1. The viewport asks for `T` as today (`LodWorker::request_conformed`). The noise tile (the shipped FG tile) is
   synthesised and drawn immediately, exactly as now.
2. In parallel a **refine job** for `T` is queued on the same pool at lower priority: read `.zr`; on a valid hit,
   decode (~1–2 ms) and colour; on a miss, make sure the ancestors and neighbours in its window are present
   (recursively, coarsest first, cached as they complete), run §4.1, write `.zr`, colour.
3. When the real tile lands it **cross-fades over its noise tile** (a per-tile `real_fade` uniform beside the
   shader's existing `morph`, ~250 ms, a labelled judgement), so no pop. EF-9's conform cross-fade at mixed-level
   edges keeps working because both tiles are drawn through the same shader path.
4. **Priority**: visible tiles at the drawn level first, then their missing ancestors, then the next level's tiles
   under the view (prefetch, bounded), then neighbours; a pan or zoom cancels queued work through the existing
   `generation` rule. Tier budgets gain `refine_in_flight` and `refine_depth` beside `max_in_flight`.
5. **Revisit**: a valid `.zr` is a ~1–2 ms read plus colouring, so a revisit costs about what today's noise tile
   costs, and the refinement itself is never paid twice (the owner's "load it in"). With the colour cache (ZR-5) a
   revisit is a texture decode.
6. **Cold deep jump** (a single jump from fit to x900): the first real tile at depth needs its ancestor chain, ~4–9
   tiles per level for ~10 levels (≈ 60 tiles, ~0.1 s PC, ~1 s 6T ZR-light, estimates); the noise tile holds the view
   meanwhile.

**Pre-bake (optional).** *Refine detail for the current view* and *Bake all* keep their menu rows and the
`bake_estimate` readout, retargeted at refined tiles, run on the worker pool with progress and cancel instead of
blocking the main thread. Area pre-bake: a marquee and a depth; whole world: only down to the depth whose estimate
fits the cap (§3: ~1.6 GB to level 8 on a 40 075 km world). On a phone it runs only while charging (Q5).

### 4.6 Phone

- **What runs.** The same algorithm and rule version (one answer per world on every device; a device that computes
  less draws noise below its depth, never a different refinement).
- **Depth.** Real refinement stops `refine_depth` levels short of the PC's floor: default 2 levels on the 6T
  (texels ~76 m instead of ~19 m on the 40 075 km world) and 1 on the Pad 3, set by ZR-6's in-app measurement.
  Below that, the FG noise as shipped.
- **Cost.** ~1.7–2.4 s of background work per level of a full 6T view, first visit only, with ZR-light (§3); ~4.5–6.4 s
  with EF-3, which is why Q1 matters most on the phone.
- **Disk.** The phone default cap is lower (Q3: 512 MB); refined tiles go first.
- **Fallback.** The Performance tier keeps `zoom_refine` off (as EF-9 is off there, Ruling CL): FG noise only,
  exactly today's phone. A tier whose measured frame budget fails is moved to that fallback, not shipped degraded.

### 4.7 What this does not do

No author edits at a deep level (EF-4: sculpting a level-13 tile directly); a sculpt edits the coarse field and its
cone re-refines. No per-tile EF-6 vectors (coastline/ridge vectors stay at world scale). No change to the world's
generated fields, saves (refined tiles are a cache, not a save member; Q6) or the river network the map draws at
fit. Refined tiles do feed the 3D view (`3D_VIEWPORT_SCOPE.md` takes heights from the 2D tile builder), which is a
consequence, not a milestone here.

### 4.8 What it replaces

The noise-only deep zoom stays as the fallback and the "meanwhile" picture, by design. The reference-format bake
(1024 px, `rg16`, reference schedule, never read) is replaced by ZR's pre-bake (Q6 decides whether its archive layout
survives as an export). The reference's Burn rivers / Micro-erode toggles are not ported as toggles: their effect is
part of every refined tile.

---

## 5. Milestones

Numeric bars are measured on at least three seeds and the FG width sweep (200 / 1 200 / 5 000 / 15 000 / 40 075 km,
1024 × 512, plus the 40 075 km / 2048 world), windowed for anything that rasterises, every frame looked at, at
several zooms (Ruling CI). Timing per `MISTAKES.md`: medians with min..max, harness alone, separate processes.

### ZR-0 · Measure first (no output change)

A committed harness (`#[ignore]`d) that does §3 properly, plus the questions §3 could not answer:

- per-step cost on PC, 6T and Pad 3 **in the app's process** (not `adb shell`), and the colouring cost of one tile
  separated from its height synthesis;
- Q1's comparison: EF-3 `tile_erode` vs the light solver over `tile_flow`'s routing, on the same tiles, measured in
  B4's own terms offline (hillshaded luma gradient and `bind_r` of a refined world tile at 0.471 and 0.078 km/px
  against the native 1 200 and 200 km maps' fit tiles);
- compression at fast and default deflate levels, residual vs row-delta, u16 vs u24 height;
- the cone size and the ancestor count of a cold deep jump.

**Go/no-go:** ZR-1 starts only if the offline refined tile reaches **≥ 0.6 of the native map's luma gradient** at both
km/px (today 0.18–0.23) with **`bind_r` within 0.15**, for at least one solver, at ≤ 30 ms per tile on one PC thread.
Below that, ZR closes with the finding and the owner decides.

### ZR-1 · The refinement step (engine, no display change)

`cartalith_engine::zoom_refine::refine_tile(parent_window, coarse, key, rules) -> RefinedTile`, §4.1–4.2, behind
`zoom_refine`.

- **Determinism:** the same tile hash across 1 / 4 / 16 threads, two processes, PC and 6T.
- **Cache invisibility:** a tile computed from a parent read back from its encoded record is byte-identical to one
  computed from the in-memory parent.
- **Seams:** shared-edge `|Δ| = 0` bit for bit between same-level neighbours at every refined level, across parent
  boundaries included; `lod_sweep::seam_ratio` across seams ≤ its shipped bar.
- **Channel continuity:** ≥ 90 % of channel texels (flow above the tile channel threshold) within 2 texels of a seam
  continue on the other side.
- **Cone exactness:** changing one coarse input cell outside a tile's cone moves 0 bytes of it; inside, ≥ 1.
- **Drainage connects:** every channel texel's D8 path leaves through the tile edge or ends in a filled lake; 0 open
  pits.
- `zoom_refine` off: 0 bytes moved in every golden and in the FG tiles (identity by control flow).

### ZR-2 · Record, store and validity

`.zr` (§4.4), `AtlasStore::{put_zr, get_zr}`, the per-cell input hash grid and cone digest, eviction order.

- Round trip: decode(encode(t)) is the quantised `t`, byte for byte.
- Size: median ≤ 70 KB per 256 × 128 land tile (measured ~63 KB).
- Validity: each coarse input in the step's signature, mutated in turn inside a cone, turns that tile into a miss; a
  sculpt stroke invalidates exactly the tiles whose cone it touches (count against an independent oracle).
- Cap: `world_bytes`, `total_bytes` and `evict_to` count refined tiles (fill a store past the cap with refined tiles only; `evict_to` brings it under).
- Eviction: under a cap, refined tiles go before baked chunks; a world keeps its coarsest baked level; no orphaned
  tiles of an old `ZR_RULES_VERSION` survive a full eviction pass.

### ZR-3 · Background pipeline and the morph

The refine queue on `LodWorker`, priorities and cancellation (§4.5), the `real_fade` cross-fade.

- No main-thread frame over the shipped LOD budget during a refine wave (windowed, PC and 6T).
- Holey frames after the first built frame ≤ the shipped count on the EF-9 zoom-while-pan script.
- No pop: per-frame luma change on a tile at the moment its real tile lands ≤ the D3 level-boundary bar (1.5x the
  median of the ten frames around it).
- First visit, PC: real tiles cover the view ≤ 0.5 s median after the camera stops, per level, at Quality.

### ZR-4 · Cached revisits

- A revisit of a cached region costs ≤ 1.2x today's noise tile per tile (read + decode + colour), measured.
- After a restart, a previously visited region shows real tiles with **0** refinement jobs run (counted).

### ZR-5 · Colour cache (optional, decided by ZR-0's numbers)

RGBA keyed by (world key, rule version, appearance hash), WebP or PNG, evicted before height records.

- Revisit cost ≤ 25 % of colouring, on the 6T; an appearance change misses 100 % of entries and never draws a stale
  colour (flip each appearance input in turn).

### ZR-6 · Pre-bake and the phone pass

Retarget *Refine detail for the current view* and *Bake all* to refined tiles on the worker, with progress, cancel
and the estimate (§4.5); set `refine_depth` per tier from in-app measurements on the 6T and Pad 3.

- `bake_estimate`'s seconds and bytes within ±25 % of the measured bake, PC and 6T.
- 6T: no frame over its tier budget while refining; peak resident memory +≤ 64 MB over the noise-only baseline.
- Performance tier: 0 refine jobs.

### ZR-7 · FG's bars, closed

With `zoom_refine` on, the windowed FG harness (`_fg_probe.gd`, `fg_bars.py`):

- **B1:** `added` on land within **0.7–1.3x** of the 1 200 km map at x8 and x32, every width.
- **B4:** the 40 075 km world at 0.471 and 0.078 km/px against the 1 200 and 200 km maps at fit: land luma-gradient
  energy within **±30 %**, `bind_r` within **0.15**.
- **B5:** median land gradient **non-decreasing** from x4 to 0.05 km/px and one step beyond, on the 40 075 km world.
- **B2, B3, B7, B8** still met; **B6:** the 800 km anchor's Ruling CI ladder within ±10 % of its shipped x4 → x64
  median.
- **Off is the old look:** `zoom_refine` off reproduces the FG metrics table exactly.

Passing ZR-7 is what lets `zoom_refine` default to on (an owner ruling under §7p, with the re-baselined FG metrics
recorded old beside new).

### ZR-8 · Retire the superseded bake

Remove the 1024-px reference-schedule bake path (or keep only its archive export, per Q6), update the Atlas cache
menu's tooltips (they promise no read today; they must now say what is read), and fix the prose each change makes
stale (`menus.gd`'s atlas headers, `engine_bridge.gd`'s `atlas_is_covered` note, `bake.rs`'s "not ported" table,
`tile_erode`'s caller clause).

---

## 6. Risks

- **R1 · The taper reads as a grid.** A 16-texel band at every 256-px tile edge where channels soften. ZR-1's
  continuity bar measures it; the levers are a wider halo (cost grows with `(256 + 2H)²`) or a band scaled with the
  level's channel width.
- **R2 · Cold deep jumps.** The ancestor chain (§4.5 item 6) delays the first real tile at depth; the noise tile
  covers it, but a user jumping straight to x900 waits ~1 s on the 6T for real detail (estimate).
- **R3 · Cross-device determinism.** Any `libm` call, a reassociated sum or a thread-count-dependent reduction makes
  PC and phone disagree. Tiles are per-device caches, so a disagreement is not corruption, but it would break "one
  world everywhere" and ZR-1's cross-device bar.
- **R4 · Disk on phones.** Flash wear and a full disk. The cap, refined-first eviction and the charging-only pre-bake
  bound it; a write failure must degrade to RAM-only, visibly (`atlas_status`), never block drawing.
- **R5 · Inconsistent coarse inputs.** With `carve_rivers` on, `WorldState::field` and `flow_discharge` disagree
  (EF-1's documented hazard). The step must take `compute_flow` over the final field (EF-3's harness does) or the
  inflow condition routes against a topology the discharge never had.
- **R6 · B1 may stay short.** A wide map is smoother per cell, and real erosion carves in proportion to relief and
  drainage. If ZR-0's offline tile closes B4 but not B1, the amplitude of the new detail band (§4.1 step 3) is the
  lever, and that is a noise term again: say so rather than tune it to pass.
- **R7 · The cone grows if the halo grows.** Invalidation stays exact but gets coarser; a sculpt then re-refines a
  wider ring. Measured in ZR-0.
- **R8 · Erosion strength drifts with level.** A per-level resolution correction (`k · refine` in EF-3) that is wrong
  by a constant compounds over ten levels. ZR-1 adds a bar: across levels, the median channel incision per level is
  within 0.5–2x of the level above.
- **R9 · Stale prose.** The atlas UI promises no read today; once it reads, every tooltip and header in §5 ZR-8 is
  false at once.

---

## 7. Owner questions, each with the default this plan assumes

| # | Question | Default |
|---|---|---|
| **Q1** | Light erosion: EF-3's `tile_erode` (built and tested, ~19 ms per tile on one PC thread and 110–185 ms on the 6T), or a lighter implicit solve over the drainage step's own routing (~4.6 ms and ~34 ms)? | **Measure both in ZR-0 and take the cheaper one that passes the go/no-go**; one algorithm for every device |
| **Q2** | Where does real refinement stop and the noise law take over? | At a texel of ~15–20 m (level 13 on a 40 075 km / 2048 world); FG noise from there to level 16 |
| **Q3** | Disk budget for refined tiles? | The existing `Size cap · GB`, shared with bakes, refined tiles evicted first; default 2 GB on desktop, 512 MB on phones |
| **Q4** | Cache colour too (faster revisits, invalidated by every look change)? | Decided by ZR-0's numbers; on for phones if colouring is ≥ 4x a decode there |
| **Q5** | Pre-bake on a phone? | Area pre-bake only, and only while charging; whole-world pre-bake limited to the depth whose estimate fits the cap |
| **Q6** | The reference-format 1024-px bake and its `World/` archive export? | Retire the bake; keep the archive *export* only if you want reference interchange (default: retire both) |
| **Q7** | Do refined tiles travel with a saved project? | No: they are a cache, re-derived on demand; a project stays small |
| **Q8** | Tiers? | Balanced, Quality, Ultra on; Performance off (EF-9's Ruling CL pattern); phones `refine_depth` 2 (6T) / 1 (Pad 3) levels shallower |
| **Q9** | Faithful reference toggles (Burn rivers and Micro-erode as off-by-default switches, with their fixed-seed and world-max defects) or this design's per-tile version always on under `zoom_refine`? | This design, recorded as a §7p divergence |
| **Q10** | Sculpting directly at a deep level (EF-4)? | Out of scope; a sculpt edits the coarse field and its cone re-refines |

---

## 8. ZR-0 results (2026-11-16): no-go

A measurement record, not a status (status is `STATUS.md`'s). Harness: `cartalith-native/crates/cartalith-godot/tests/
zr0_refine.rs` (`#[ignore]`d: `zr0_bars`, `zr0_timing`, `zr0_determinism`). The §4.1 chain is prototyped there and
nowhere else. Evidence, frames, the full table and the verdict: `cartalith-native/docs/evidence/zoom-refine-zr0-2026-11-16/`
(`VERDICT.md`, `metrics.txt`, six sheets).

**Method.**

- The world and the native maps:
  - the 40 075 km / 2048 x 1024 world, seed 1200;
  - the native 1 200 km and 200 km maps, seed 1200, 1024 x 512.
- Each is coloured by the shipped deep-zoom colouriser at 0.471 and 0.078 km/px. The ZR frames use the shell's level and
  morph rule.
- Three places, picked by inputs: a mountain, a plain and FG's pivot.
- `bind_r` uses one definition for every frame. A native map at fit has no on/off pair, so B4's `bind_r` had never been
  measured. The definition is in `metrics.txt`'s header.
- Prototype choices, all labelled in the harness:
  - the global quantum `2^-20`;
  - a burn at the level where the valley width `0.1 * sqrt(A km^2)` spans 2-4 texels, with depth half the local slope
    times the width;
  - uplift off for both solvers;
  - FG's top fade on the band.

**Against the bar.**

- **Land luma gradient (≥ 0.6 of native):**
  - Against the same-terrain native windows it is met everywhere: 0.63-4.37 for Braun-Willett, 0.59-4.31 for EF-3. On
    mountains it overshoots 2.8-4.4x.
  - Against the whole native map at fit it fails at 0.078 on flat ground (0.47-0.49).
- **`bind_r` (within 0.15 of native):** it fails in 5 of 6 cells for both solvers. ZR gives 0.10-0.25 where native gives
  0.47-0.59. Only the 0.471 mountain passes (0.39 vs 0.43).
- **Cost (≤ 30 ms per tile on one PC thread):**

  | Solver | PC, one thread | 6T, one thread |
  |---|---|---|
  | Braun-Willett | 20.4 ms (19.5..21.3): passes | 80.7 ms |
  | EF-3 | 37.0 ms (35.1..38.5): fails | 139.7 ms |

  Colouring one tile costs 61 ms on the PC and 189 ms on the 6T, both on one thread.

**Q1:** Braun-Willett, if anything is built. EF-3 is no better on any measure.

**Why it fails.** The refined tiles are the per-level FG band, and noise dominates them:

- Drainage, the burn and erosion add 4-12 % to the gradient over the steps-2-3-only control.
- The native maps' km-scale ridge and valley systems have no counterpart. Nothing in a per-tile step creates structure
  at 5-50 km from a smooth 19.6 km cell.
- Physical-unit steps fight the renderer's per-map normalisation. One 35° talus pass moved 1.6 M texels on the world
  mountain at level 8, because the FG amplitude implies slopes steeper than 35° there.

This is R6's case in a stronger form. Closing B4 would need a landform process at coarse refined levels, not a light
per-tile pass. **No tuning was done to pass.**

**What holds:**

- **Seams:** 0 edge texels off the base at every level; channel continuity 94.7-98.1 %; seam step ratio 1.06-1.21.
- **Determinism:** identical digests at 1, 4 and 16 threads, and identical on the PC and the OnePlus 6T, for both solvers.
- **Cold jump:** one level-8 tile needs 41 ancestor tiles (levels 1-7).

**Not measured:** compression, the Pad 3, the app's own process, more seeds.

**Decision:** per §5 ZR-0, ZR-1 does not start. ZR closes with this finding, and the owner decides.
