# 3D viewport — scope (a vertex mesh over the existing raster)

**What this is.** The scope for a navigable 3D view of the map. It defines
milestones 3D-0 to 3D-8 and gives the reasoning for them. It carries **no
status column**: progress lives only in `cartalith-native/docs/STATUS.md`
(`CLAUDE.md`, "One place holds state"). Writing it touched no Rust, no
GDScript, no `project.godot` and no shader.

**Why now.** The owner, 2026-10-18, wanted the 3D mode on the roadmap and
said: *"Maybe 3D should be done with vertices."* 3D had been parked since
2026-08-31 (`LARGE_ITEM_RULINGS.md`, "Deferred, with research first").
Ruling AZ (2026-09-28) started only the *research* track ("Research tracks to
start: the 3D viewport (its three commissioned questions)"). **This document is
the scope that request asks for.** It was written awaiting the owner's
approval (question Q10 in §7). **The owner approved it on 2026-10-19**
(`LARGE_ITEM_RULINGS.md`, Ruling CS item 4), and `DECISIONS.md` §4a records
that beside §4 ("2D only for v1"), which it supersedes for this scope only.
3D-0 was scheduled by the same ruling.

**Inputs read for this scope, and what each contributed:**

- `cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md`. It is 1 708 lines
  today; the "1 530" other documents quote is the 2026-09-01 count. It gave
  the approach survey, the capability-gate framing, the recommendation and
  the 2026-09-01 answers. **Two of its premises are now out of date (§0).**
- `LARGE_ITEM_RULINGS.md`: the 3D rows, Ruling K, Ruling AZ, Ruling CI and
  its addendum, and Ruling CL.
- `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`: §2, §4, EF-0 and §8.5.
- `LOD_DETAIL_SCOPE.md`: the rules, and the bars of D0, D3 and D6.
- `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md`, `TERRAIN_ARCHITECTURE_RESEARCH.md`
  §3–5 and `MAP_STYLE_RESEARCH.md` (the Berann row).
- The code, opened at the symbols named below.
- **New measurements taken for this scope** (§2.4). They came from a
  throwaway Godot 4.7.1 project in the session scratchpad, which was deleted
  afterwards. Nothing was committed.

---

## 0. Summary

1. **"With vertices" means a display mesh, not a vertex world.** The terrain is
   drawn as real triangles. Its vertex heights are read from the same height
   tiles the 2D deep-zoom view colours, and they are morphed by the same
   continuous level `L(x)` (EF-9, §8.5). The simulation stays a raster. This
   is the reading §1 recommends. Storing the world as vertices (a TIN) is also
   considered there, and it would overturn the elevation-field architecture.
2. **wgpu and Godot cannot share a device under the shipping renderer.** They
   stay separate devices, and data crosses through the CPU. That costs the 3D
   viewport nothing: its heights and colours are already CPU products of the
   `lod_worker` pool. Under Forward+ or Mobile a shared device is possible in
   the API, but it is unsafe in practice (§2.1).
3. **`gl_compatibility` stays.** It runs everything this design needs,
   measured today on the dev desktop. Under Compatibility the depth buffer is
   24-bit with no reverse-Z, and the GPU's device class is not reported; the
   design accounts for both (§2.2).
4. **A raised device floor buys headroom, not techniques**, while the renderer
   stays Compatibility. The techniques it would unlock need a renderer switch,
   and that switch is blocked by the unexplained device loss (§2.3).
5. The design is §3, the capability gate is §4, and the milestones are §5.
   3D-0 is a measurement go/no-go.

**Two premises of the 2026-08-31 research that no longer hold:**

- **`QuadTree<T>` is gone.** The research builds its culling and LOD on
  `cartalith_spatial::QuadTree<T>`, but `QuadTree` and `TiledField` were
  retired in `5c99cc9` (2026-09-22; `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`
  §2's correction). This scope builds its quadtree on the pyramid's own
  `ChunkId {z, col, row}` addressing instead. That is what EF-0 and EF-9 were
  built on.
- **The tiles are no longer what the research assumed.** It assumed the drape
  is one grid-resolution colour texture. Since then the deep-zoom tiles carry
  real colour (LOD-D1/D2) with kilometre-keyed detail (`1fc9208f`, Ruling CI).
  They also carry painted rivers (RIM-7) and importance refinement (EF-9, on
  by default since Ruling CL). A 3D view that drapes only the grid-resolution
  texture would break the hard requirement that zoom gains detail at every
  level (Ruling CI item 1).

---

## 1. What "with vertices" means

The owner's sentence has four readings. All four were considered.

| # | Reading | What it is | Verdict |
|---|---|---|---|
| **V1** | **A display mesh from the raster, displaced in the vertex shader** | A small flat patch, one mesh reused for the whole terrain. Each instance's vertices read their height from a height tile in the vertex shader, and the morph happens there too. | **Recommended** |
| V2 | A display mesh built on the CPU | Rust computes every vertex position for each chunk and uploads an `ArrayMesh`. | Kept as a fallback only (§3.2) |
| V3 | **The world stored as vertices (a TIN or adaptive mesh) instead of a raster** | Generation, saving and editing all operate on a triangulation. | **Not recommended.** It would overturn the elevation-field decision; cost below |
| V4 | The 2D map drawn as a mesh | Replace the 2D tile compositor with geometry. | Out of scope. 2D draws texels, and geometry adds nothing to a top-down view |

**Why V1 is the right reading.** The design that follows from it treats
vertices as the way the surface is drawn, and the raster as the way the
surface is known. That split is the one the elevation-field work already
chose ("raster stays the simulation substrate, deliberately not a vertex/TIN
replacement", per `CLAUDE.md`'s row for
`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`). It is also what the 2008–2010
terrain-rendering literature converged on:

- Geometry clipmaps: constant x/y vertex data, with z fetched from an
  elevation texture in the vertex shader (Asirvatham and Hoppe, *GPU Gems 2*
  ch. 2).
- CDLOD: a quadtree of regular grids morphed in the vertex shader (Strugar).

V1 over V2 is a measurement, not a taste. In the scratch probe (§2.4), one
65 × 65 patch mesh reused 1 024 times held video memory at **18.9 MB**. One
CPU-style 2 049 × 2 049 mesh holding similar geometry cost **226.7 MB**. A
CPU-built mesh also has to be rebuilt on every level change and every
regenerate, and the patch never changes.

**What V3 would cost**, stated so that the owner can choose it knowingly:

- **Every generation stage reads and writes a `gw × gh` `Vec<f32>`.** That
  includes tectonics, erosion (`cartalith-erosion`), D8 flow and channels
  (`cartalith_hydrology::compute_flow` / `build_channels`), climate, civ
  cost-distance and the GPU kernels (`cartalith-gpu`). Each would need
  re-deriving on a triangulation. The alternative is to rasterise the TIN
  back for them, and then the TIN is a display format, which is V1 with
  extra steps.
- **The parity contract would go.** `DECISIONS.md` §7's golden-value
  harnesses compare raster arrays against the JS engine. Nothing replaces
  them on a mesh.
- **The save format changes.** `rasters/heightmap.f32` is the canonical field
  on disk (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §2), and
  `SAVEFILE_COMPAT.md` would need a mesh section and a migration.
- **What V3 is usually chosen for is already here.** Its usual benefit is
  adaptive resolution: dense where relief is, sparse on plains. This project
  already gets arbitrary detail on demand from
  `cartalith_engine::elevation::world_sample_elevation` (EF-0) and the tile
  synthesis, without storing it.

**Where "vertices" goes beyond the terrain:** overlays become geometry in 3D
too. Settlements are instanced meshes, labels are billboards, and roads and
borders are draped ribbons (§3.8). Those are vertices as well, and the
owner's sentence fits them.

---

## 2. The three commissioned questions, answered

`3D_TERRAIN_RENDER_RESEARCH.md` §"Answered 2026-09-01" already answered all
three, and those answers stand. This section re-checks them against today's
tree, adds what was measured for this scope, and answers them for the design
in §3 rather than in general. **The coexistence question comes first, as the
brief asks.**

### 2.1 Can `cartalith-gpu`'s wgpu compute and Godot's renderer share a device?

**Under the renderer that ships (`gl_compatibility`): no. They are separate
devices, and data crosses through the CPU.**

- **Godot exposes no GPU device here.** Measured today under
  `gl_compatibility`/`opengl3` on the dev desktop:
  `RenderingServer.get_rendering_device()` returned `null`, and so did
  `create_local_rendering_device()`. The Godot class reference agrees:
  *"RenderingDevice is not available when running in headless mode or when
  using the Compatibility rendering method."* With no `RenderingDevice` there
  is no Vulkan handle to hand to wgpu, and no way to import a wgpu buffer or
  image.
- **wgpu cannot join Godot's GL context either.** wgpu is deliberately kept
  off GL. `cartalith-gpu/src/multi.rs`'s `COMPUTE_BACKENDS` is
  `VULKAN | DX12 | METAL | BROWSER_WEBGPU`, and `compute_instance()` is the
  only instance construction site. This is the `6b2c4d9` fix for wgpu standing
  up a second GL context inside Godot's process (the `6a97911` crash). The
  separation is half of what makes today's arrangement work, not an accident
  to tidy away (research, Question 2). Searched: `wgpu-hal-30.0.0/src/gles/`
  for `pub unsafe fn new_external` and `pub unsafe fn from_raw`; neither
  matched, so the GLES backend offers no adopt-a-context constructor.

**Under Forward+ or Mobile on Vulkan: possible in the API, unsafe in
practice.**

- **Godot hands out the raw handles.** Measured today, `forward_plus/vulkan`
  and `mobile/vulkan`:
  `RenderingDevice.get_driver_resource(DRIVER_RESOURCE_LOGICAL_DEVICE |
  TOPMOST_OBJECT | COMMAND_QUEUE, RID(), 0)` all returned non-zero, and
  `QUEUE_FAMILY` returned 0. Under `forward_plus/d3d12` the device and queue
  handles were non-zero but `TOPMOST_OBJECT` was 0. The 4.7
  `extension_api.json` names the Vulkan aliases
  (`DRIVER_RESOURCE_VULKAN_DEVICE`, `_INSTANCE`, `_QUEUE`,
  `_QUEUE_FAMILY_INDEX`, `_IMAGE`, `_BUFFER`). It also lists
  `RenderingDevice.texture_create_from_extension` (wrap a native image in an
  RD texture) and `RenderingServer.texture_create_from_native_handle`.
- **wgpu 30 can adopt them.** These are in the locked wgpu 30.0.0 source:
  - `wgpu::Instance::from_hal` (`src/api/instance.rs`);
  - `Adapter::create_device_from_hal` (`src/api/adapter.rs`);
  - `wgpu_hal::vulkan::Adapter::device_from_raw` (`src/vulkan/adapter.rs`);
  - `Device::create_texture_from_hal` and `create_buffer_from_hal`
    (`src/api/device.rs`).

  So a wgpu device can wrap Godot's `VkDevice`, and Godot can wrap a
  wgpu-created image.
- **Why that path is unsafe in practice. Each reason is checked, not assumed:**
  1. **`device_from_raw`'s safety contract can't be shown to hold.** It
     requires that *"`enabled_extensions` must be a superset of
     `required_device_extensions()`"* and that the device was created with
     wgpu's `physical_device_features()`. Godot created its device with its
     own extension set, and nothing exposes that set: in 4.7's
     `extension_api.json` the only `RenderingDevice` method with "extension"
     in its name is `texture_create_from_extension`. A violated contract here
     is undefined behaviour, not an error.
  2. **Sharing a queue needs one lock that both sides honour.** The Vulkan
     specification lists *"The `queue` parameter in vkQueueSubmit"* among
     the externally synchronised parameters. The exception is a queue
     created with `VK_DEVICE_QUEUE_CREATE_INTERNALLY_SYNCHRONIZED_BIT_KHR`,
     and Godot gives no sign of using it. Godot submits from its render
     thread. wgpu submits from wherever `Queue::submit`, `write_buffer` or
     `poll` is called, and that includes the generation thread and the
     `lod_worker` pool. Every wgpu submit would have to be marshalled onto
     Godot's render thread (`RenderingServer.call_on_render_thread` exists
     for this), which reshapes the whole GPU crate.
  3. **The renderer switch itself fails on this hardware, cause unknown.**
     Forward+ and Mobile lose the device during a generate
     (`VK_ERROR_DEVICE_LOST` 3 of 3; `DXGI_ERROR_DEVICE_REMOVED` on d3d12;
     research §Question 2, and `STATUS.md`'s 2026-09-01 entry). It recurred in
     probes on 2026-09-07 (`OUTSTANDING_WORK.md`, "The `--generate` leg of the
     capture probes crashes the GPU"). **It is still unattributed.** The
     isolation control was never run: one generate under `forward_plus` with
     `use_gpu=false`, blocked because `engine_bridge.gd::_ready` forces
     `param_set("use_gpu", true)`. Searched: `git log --grep` for
     `forward_plus|device lost|isolation control` returns no later
     measurement. Sharing a device makes two stacks more coupled, not less.
- **Two separate Vulkan devices with shared memory** is a third option. wgpu
  can export memory through `VK_KHR_external_memory_win32`/`_fd`, which
  `wgpu-hal`'s `adapter.rs` enables when available. It still needs Godot on a
  `RenderingDevice` renderer to import the memory, plus semaphores across two
  devices, so it inherits reason 3.
- **Has anyone shipped wgpu on Godot's own `VkDevice`?** No source found.

**What this means for 3D: nothing is lost.** The 3D viewport's inputs are
height and colour tiles. `lod_worker`'s rayon pool already produces them on
the CPU (`LodSnapshot::render_tile`, LOD-D6), and the shell already uploads
them as textures for 2D. A tile is 256² RGBA8, 262 kB (`LodBudget`'s own doc).
A height tile of the same size in `FORMAT_RF` is the same order. The upload
already happens every time 2D draws a tile. **The 3D design in §3 needs no
interop, so the coexistence question does not gate it.** What it does gate is
the deferred **Full-GPU** path (GPU-driven culling, GPU tile synthesis inside
Godot; §2.3), which needs a `RenderingDevice` renderer whatever happens with
wgpu.

### 2.2 Why `gl_compatibility`, and does 3D need anything else?

**The rationale (re-verified today).**

- `HEAD:cartalith-native/godot-project/project.godot` lines 101–102 set
  `renderer/rendering_method` and `.mobile` to `"gl_compatibility"`. The
  working copy currently carries an uncommitted edit by another session, so
  the line numbers were read from `git show HEAD:`.
- The setting was **inherited from Phase 0 (`d1922fe`), with its reasoning
  recorded**. `.claude/skills/godot-shell/SKILL.md`, "Choose the renderer
  before you build the UI", landed in the same commit: Compatibility "reaches
  the most Android hardware and asks the least of it"; it has no compute; and
  the choice should be revisited "if the later 3D phase wants GPU terrain
  work" (research, Question 1).
- **Two findings since then make it the measured choice, not only the
  inherited one:**
  - Forward+ and Mobile lose the device on generate (§2.1, reason 3).
  - HDR output exists only on Forward+ and Mobile, which is why ET-E's HDR
    half is blocked (`EDITING_TOOLS_SCOPE.md`, "Design, the HDR part";
    `STATUS.md` ET-E4).

**What 3D needs, checked under `gl_compatibility` today.** Throwaway project,
Godot 4.7.1, RX 7800 XT, driver 26.7.1, windowed 1600 × 900, vsync off:

| Capability the design uses | Result | How it was checked |
|---|---|---|
| Vertex-texture displacement (VTF) | **works** | A 257² patch with a `textureLod` height read in `vertex()`. Switching `amp` 0 → 4 changed 24 266 sampled pixels (positive control). Same result on Forward+ and Mobile |
| Per-instance height layer from a `Texture2DArray` (`FORMAT_RF`), chosen by `INSTANCE_CUSTOM` in a `MultiMesh` | **works** | `texelFetch(sampler2DArray, ivec3(p, int(INSTANCE_CUSTOM.x)), 0)`. `Texture2DArray.update_layer()` on one layer moved 3 662 sampled pixels |
| `MultiMesh` with per-instance custom data | **works** | One draw call for 1 024 patches (table in §2.4) |
| One shadowed `DirectionalLight3D` | cited, not measured | Godot PR #77496 (research §C) |

**Two Compatibility limits the design must absorb:**

- **Depth is 24-bit with no reverse-Z.** The Godot renderer comparison lists
  Compatibility depth as *"24-bit without reverse Z, medium precision"*, so
  near and far planes are set dynamically (§3.11).
- **The GPU's device class is not reported.** Under `gl_compatibility`,
  `RenderingServer.get_video_adapter_type()` returned **0
  (`DEVICE_TYPE_OTHER`) for a discrete GPU**. Under Vulkan it returned 2
  (discrete). The research's Standard tier keys on a "real device-class
  signal", which Compatibility does not give, so §4's gate measures instead
  of reading a class.

**Verdict: stay on `gl_compatibility`.** This is research §D's Milestone 0,
now on the measurements above. Nothing in 3D-1 to 3D-8 needs another renderer.

### 2.3 What would a raised device floor buy?

**It depends on the renderer.**

**While the renderer is Compatibility, the floor buys headroom and nothing
else.** A faster GPU or more RAM moves a device up §4's tiers: more patches,
more cached tiles, shadows, a higher patch density. It unlocks no technique.
Godot 4.7's own documentation names the Adreno 630 as an example device for
the Mobile renderer (research, Question 3), so the 6T was never the
constraint on renderer choice either.

**What a renderer switch would unlock**, each checked in 4.7's
`extension_api.json` or the renderer docs:

| Capability | Forward+ / Mobile (`RenderingDevice`) | Compatibility |
|---|---|---|
| Compute shaders, so GPU culling and GPU tile synthesis inside Godot | yes (`compute_list_dispatch_indirect` exists) | no |
| Indirect draws, so GPU-driven visibility | yes (`draw_list_draw_indirect`) | no |
| Mesh shaders | **not in Godot 4.7 at all**: no `RenderingDevice` method has "mesh" in its name | no |
| 16-bit storage or arithmetic as a public flag | **no public flag.** `RenderingDevice.Features` lists only MetalFX ×2, buffer device address, 32-bit image atomics, ray query, raytracing pipeline and HDR output | no |
| Reverse-Z depth | yes (the renderer docs say Compatibility lacks it) | no |
| HDR output | yes | no |
| Hardware limit queries (`limit_get`, `has_feature`) | yes; measured: max 2D texture 16 384, compute workgroup x 1 024 | no API; probe instead |

**Texture formats do not move with the floor.** The design uploads
runtime-generated tiles uncompressed (RGBA8, `FORMAT_RF`), so ETC2, ASTC and
BC compression would not help.

**So the floor is not what gates 3D.** The renderer is, and the renderer is
gated by the unattributed device loss. One caution carries over from the
research: the 6T is the only handset on the bench, so raising the minimum
above it makes the minimum tier unverifiable on owned hardware.

### 2.4 Measurements taken for this scope

**Setup.** One machine (Ryzen 7 9800X3D / RX 7800 XT, Godot 4.7.1,
`gl_compatibility`, windowed 1600 × 900, vsync off). Each run is unshaded,
with every vertex in view (worst case) and one 1 024² height texture. Frame
time is the wall-clock delta between `_process` calls over 300 frames, after
60 warm-up frames. **Each configuration ran twice in separate processes**,
and the two figures are given as median (min..max) for each run.

**These are scratch measurements, not this app's**: no world, no colour
drape, no overlays. They size §4's bars and prove nothing about the shipping
shell.

| Workload | Triangles | Draw calls | Frame time, run 1 | Frame time, run 2 | Video memory |
|---|---|---|---|---|---|
| One 257² mesh | 131 072 | 1 | 0.133 ms (0.042..0.401) | 0.138 (0.047..0.333) | 21.9 MB |
| One 1 025² mesh | 2 097 152 | 1 | 0.599 (0.501..0.843) | 0.603 (0.438..0.842) | 70.7 MB |
| One 2 049² mesh | 8 388 608 | 1 | 1.964 (1.076..2.856) | 1.967 (0.846..3.139) | 226.7 MB |
| 256 patches of 65², one `MeshInstance3D` each | 2 097 152 | 256 | 0.973 (0.371..1.917) | 0.971 (0.678..1.278) | 18.8 MB |
| 1 024 patches, separate | 8 388 608 | 1 024 | 2.750 (2.088..3.549) | 2.743 (2.160..3.418) | 18.8 MB |
| 4 096 patches, separate | 33 554 432 | 4 096 | 9.995 (7.482..12.187) | 9.974 (7.616..12.403) | 18.8 MB |
| 256 patches, one `MultiMesh` | 2 097 152 | 1 | 0.921 (0.070..2.251) | 0.913 (0.716..1.131) | 18.8 MB |
| 1 024 patches, one `MultiMesh` | 8 388 608 | 1 | 1.909 (1.692..2.795) | 1.866 (1.702..2.042) | 18.9 MB |
| 4 096 patches, one `MultiMesh` | 33 554 432 | 1 | 5.962 (4.555..7.088) | 5.967 (5.698..6.309) | 19.0 MB |

**What the table supports:**

- **Desktop has an order of magnitude of headroom.** §3.12's Standard
  workload is about 2 M triangles, and it ran in about 1 ms here.
- **At 1 024 patches and above, `MultiMesh` beats one node per patch.** It
  measured 1.9 ms against 2.7 ms at 1 024 patches, and 6.0 ms against 10.0 ms
  at 4 096, so the per-node cost under Compatibility is real.
- **A reused patch keeps memory flat; a whole-world mesh does not.**

**What it does not support:** any statement about the Adreno 630. That is
3D-0(c).

---

## 3. Design

The data path in one line: the existing tile synthesis produces colour and
heights; a quadtree selects tile nodes by `L(x)`; and one patch mesh is drawn
as instances that fetch heights and colour from texture arrays.

### 3.1 Heights come from the same builder as the 2D colour tiles

`lod_bridge::tile_heights` (private today) builds each tile's amplified
heights. Each builder uses the "Ruling CI" kilometre zoom law
(`render::tile_amplify_opts` → `zoom_detail_km_law`), RV-3's valley field, the
water tile and the halo (`render::tile_halo_px`). It then hands them to
`render_biome_tile_rgba_water` to colour.

**The 3D view takes its vertex heights from that same `TileHeights`,** in the
same synthesis that produces the colour tile. One worker job returns
`(rgba, heights)`, which extends `LodSnapshot::render_tile`. That gives three
guarantees:

1. **2D and 3D agree on height by construction.** The surface's shape and the
   shading baked into its colour come from one array.
2. **No second synthesis.** LOD-D6's determinism property, *"a worker-built
   tile is byte-identical to a main-thread one"*, extends to the heights
   unchanged.
3. **The halo carries across tile edges,** so normals computed from it match
   along every tile seam (§3.4).

**Do not use EF-0's `world_elevation_tile` for this, although its name
suggests it.** Commit `1fc9208f`'s message records that "EF-0/EF-9
`world_amplify_opts` keep the reference schedule", so EF-0 tiles lack the
kilometre zoom law the screen tiles have. A 3D surface built from EF-0 would
disagree with the 2D picture at every zoom past ×4. This is the
"capability attached is not the consumer reading it" class (`MISTAKES.md`).

**Units.** The field is normalised `[0, 1]` `f32`. Metres come from the
world's own conversion, the one `WorldGen::measure_vertical` reports with.
Read it from that function at build time rather than restating a formula
here, which would be a second source for one conversion. Vertical
exaggeration is a uniform shader scale taken from `Preferences ▸ Graphics ▸
relief exaggeration` (`menus.gd::_build_relief_exag_menu`). **It never writes
back into `WorldState`** (research §E: save, export and
`get_map_height_km()` read the same field).

### 3.2 Geometry: a CDLOD quadtree on the pyramid's `ChunkId`

- **A node is a pyramid tile** `(z, col, row)`, covering index space
  `[c, c+1]/2^z × [r, r+1]/2^z` at every level (§8.5). The root is z = 0.
  Depth is capped by the tier's `LodBudget::max_level` and `lod_bridge`'s
  `MAX_LEVEL` = 10.
- **Each drawn node is one instance of one shared patch** of `N × N` vertices.
  N is 33, 65 or 129 by tier (§3.12). The patch is a unit square, and the
  instance transform scales it to the node's footprint.
- **The patch is coarser than the tile, on purpose.** A 256-px tile drawn
  with a 65² patch puts a vertex every 4 tile texels. The finer texels still
  reach the screen through per-pixel normals (§3.4) and the colour drape.
  That is the CDLOD/clipmap convention: geometry carries the silhouette and
  textures carry the detail.
- **Draws.** All instances of one level form one `MultiMesh`, so terrain is
  at most `MAX_LEVEL + 1` draw calls. This is a design property, chosen
  because one node per patch cost more under Compatibility in §2.4. Each
  instance's custom data carries its tile layer index, its parent's layer
  index and its index-space footprint.
- **V2 fallback.** If a device fails §4's vertex-texture-fetch control,
  patches are built CPU-side as `ArrayMesh` from the same `TileHeights`, with
  the same selection. It is slower and costs more memory (§2.4). It exists
  only so that a device without VTF lands in Basic rather than Off.

### 3.3 The level is the same `L(x)` as 2D, so detail agrees

**How 2D chooses a level** (EF-9 §8.5):

```text
L(x) = λ_zoom + gain · BOOST_LEVELS · b(x)
λ_zoom = log2(max(1, gw · px_per_cell / TILE_PX))
```

The terms:

- `λ_zoom` is `lod_bridge::continuous_level`.
- `b(x)` is the zoom-independent `BoostField` texture
  (`cartalith_engine::continuous_lod`).
- Each level's weight is `w_k = smoothstep01(L − k + 0.5)`.

In 2D, `px_per_cell` is uniform over the view.

**In 3D, `px_per_cell` varies across the view** with the distance from the
camera:

```text
px_per_cell(x) = cell_m · f_px / max(d_min, |x − camera|)
f_px          = viewport_height_px / (2 · tan(fov_y / 2))
λ_3d(x)       = log2(max(1, gw · px_per_cell(x) / TILE_PX))
L_3d(x)       = λ_3d(x) + gain(λ_3d) · BOOST_LEVELS · b(x)
```

The terms:

- `cell_m` is the coarse cell's size in metres (`cartalith_spatial::cell_km`).
- `d_min` is a clamp so the camera's own position never divides by zero. It is
  a labelled judgement: 1 cell.
- `gain(λ) = smoothstep01(λ − λ_entry)` restates §8.5's `gain(zoom)` in level
  units. `λ_entry` is the level at the 2D `_update_lod()` entry gate.

This is CDLOD's distance-driven level with EF-9's importance boost added,
which is exactly the substitution §8.5 made in the other direction.

**The agreement property, which makes this testable.** Take an orthographic
top-down 3D camera at the 2D view's scale. Its `px_per_cell(x)` is the 2D
`px_per_cell` at every point, so `λ_3d ≡ λ_zoom` and `L_3d ≡ L`. The same
levels and the same weights follow, and so do the same heights and colours.
Acceptance bar **B1** (§5) measures this.

**The morph.** A vertex of a node at level k lies where
`L_3d ∈ [k − 0.5, k + 0.5)`. It takes `w = smoothstep01(L_3d − k + 0.5)` and
does two things:

1. **Moves sideways.** Its x/y slides toward the parent grid's position
   (CDLOD's odd-vertex morph) as `w → 0`.
2. **Blends its height:** `h = mix(h_{k−1}(x′), h_k(x′), w)` at the morphed
   position `x′`. Both are read from resident layers: the own tile and the
   parent tile.

**No crack between nodes of neighbouring levels.** A level-k node meets a
level-(k−1) neighbour only where `L_3d ≤ k − 0.5`, so `w = 0` along that edge.
There its vertices sit on the parent grid with parent heights, matching the
coarser node exactly. This is §8.5's spatial argument ("a `z0+1` tile next to
one that is not wanted has weight 0 along their shared edge"), carried from
colour to geometry.

**The selection rule that makes it hold.** Split a node when `L_3d ≥ k + 0.5`
anywhere in it, and keep neighbours within one level. That is CDLOD's range
condition; Strugar claims "no need for stitching meshes" for it.

### 3.4 Colour drape, lighting and normals

**The drape** is the colour tile from the same job (§3.1), sampled with the
same per-level weights as the heights. It carries:

- every look in `render::LOOK_PRESETS`, including "Satellite relief" with its
  Ruling CI addendum terms (slope snow and rock);
- the painted rivers (RIM-7);
- the lithology and RV-3 shading.

None of it is recomputed for 3D.

**Baked shading versus real light.** 2D tiles bake a top-down hillshade, and a
3D light on top of it would shade the slopes twice. Two drape modes:

- **Map shading (as 2D)**: the baked tile, drawn unlit. This is the Basic
  tier and a selectable mode, and it is what the 2D/3D agreement bar (B1) is
  measured in.
- **Lit (default from Standard up)**: the same tile requested with the
  hillshade layer hidden, lit in 3D. The hillshade layer is `LayerStack`'s
  `RasterLayer::Hillshade` with `visible = false`, the existing layer
  mechanism. The light is one `DirectionalLight3D` whose direction is the
  look's own sun from `TerrainAppearance`.
  - Hiding the layer is a different tile request, so it needs its own cache
    key. The key derivation must include it (`MISTAKES.md`, "write a
    staleness/cache key").

**Normals are per pixel, not per vertex.** They come from finite differences
of the height layer in the fragment shader. That is the Sobel form
`slope_at`/`grad_at` use in Rust, and it reads the tile halo, so normals agree
across tile seams. Per-vertex normals are not built: a 4-texel vertex spacing
would discard the detail the tile carries.

**Ambient occlusion.** `build_ao`'s cavity map is a top-down approximation
(its own doc comment), so it is not used in the lit mode until it has been
looked at from oblique angles (research §C).

### 3.5 Seams and skirts

Crack-freedom is by construction (§3.3). **Skirts are kept as a safety net**:
downward fins along each node's border, as in Ulrich's chunked LOD (described
in Schneider and Westermann 2006).

- **When they matter:** the one frame in which a parent layer is not yet
  resident. Parent fallback (LOD-D3) normally prevents that.
- **Depth:** the node's local relief bound, from §3.13's min/max hierarchy.
- **Cost:** `4N` extra vertices per patch.

### 3.6 Sea and shore

- **Geometry below sea level is flattened to the sea surface by default.** The
  drape already draws depth through the look's sea ramp, including Satellite
  relief's navy-to-turquoise shelf, so true bathymetric relief under an opaque
  sea is wasted triangles.
  - This is the "flatten oceans" parameter `menus.gd`'s "3D viewport
    defaults" row lists (SS2.5).
  - A bathymetry mode is that switch turned off.
- **Shore and lakes use the 2D test.** The shoreline is decided by the same
  height test as the 2D tile's water: `lod_bridge::synthesize_tile_water_mask`
  and the `TileHeights.water` tile. So the coast cannot sit in a different
  place in 3D.
- **Not in scope:** an animated water surface.

### 3.7 Rivers: painted into the colour, not ribbons

- **Rivers stay in the colour** (Ruling BU, `RIVERS_IN_MAP_SCOPE.md`). The
  carved field puts river cells lower, so a painted river on the drape lies
  in its valley in 3D with no extra geometry.
- **Not ribbons.** Ribbon geometry is the research's milestone 6, and it is
  needed only for a look that strokes rivers as vector lines. It is filed as
  an optional follow-up, not in 3D-0 to 3D-8.
- **Not decals.** The river is already in the colour layer.

### 3.8 Overlays

| Overlay | In 3D | Source |
|---|---|---|
| Settlements and icons | Billboarded quads in one `MultiMesh` per icon class, atlas UV in custom data | The same render list `map_overlay.gd` draws in 2D |
| Labels | `Label3D` with `fixed_size`, culled by distance and by the 2D label tiers' minimum zoom, mapped through `px_per_cell(x)` | The 2D label placement |
| Roads, sea lanes, borders, journey routes | Ribbon meshes draped by sampling the displayed height along each polyline, plus a small depth offset | `world_sample_elevation` at the node level, CPU side |
| Painted colour layer (Ruling CN) | In the drape | Composited into the tile like the 2D path |

**Billboards versus decals.** A decal drape (project the 2D overlay layer onto
the terrain) was rejected. The 2D overlays are Godot `_draw()` calls, not a
texture, so there is nothing to project without rasterising every overlay
into tiles.

### 3.9 Camera

- **Three modes:**
  - **Orbit** (default): pivot on the picked ground point.
  - **Fly**: WASD plus mouse on desktop.
  - **Top-down**: orthographic, at the 2D view's scale.
- **Entering 3D starts top-down at the current 2D view, then tilts.** This
  honours Ruling K, which made the Aletsch target top-down.
- **Clamps:**
  - the camera's height above the ground under it, `≥ max(2 cells × 0.1,
    near-clearance)` (a labelled judgement), sampled from the displayed
    surface (§3.10);
  - pitch between 5° and 90° in Orbit;
  - the pivot inside the world rectangle.
- **Phone gestures:** two-finger pan, pinch zoom, two-finger twist rotate and
  two-finger vertical drag tilt. The gestures are arbitrated in the same place
  the slider and scroller arbitration lives (`MISTAKES.md`, gesture hazard
  rows).

### 3.10 Picking: casting a ray onto the drawn surface

Every map tool converts a screen point into map coordinates through the 2D
viewport transform (`viewport_host.gd`, `_map_display_rect()`). In 3D that
step becomes a ray cast onto **the surface as drawn**:

1. **Coarse cull.** Walk a min/max height hierarchy built once per world from
   the field: `4^z` aggregation per level, like `BoostField`'s build. This
   replaces the retired `QuadTree`. It skips nodes the ray cannot touch.
2. **March inside candidate nodes.** Evaluate the height exactly as the
   vertex shader does: same `L_3d`, same levels, same morph, same mix.
3. **Refine by bisection** to below 0.1 cell.

**Heights for picking stay on the CPU.** The worker's height arrays stay
CPU-side in an LRU keyed by `ChunkId`, beside the GPU layer. Reading depth
back from the GPU under Compatibility means a full-frame
`get_texture().get_image()`, which stalls the pipeline.

**What picking feeds:** the context card (`context_broker.gd` /
`context_card.gd`, `MAP_CONTEXT_SCOPE.md`), the measure tools
(`WorldGen.measure_vertical`), the place editor, and later the brushes.

### 3.11 Precision at world scale

**The range.** World width goes up to **100 000 km**: `new_world_dialog.gd`'s
"Width (km)" field allows 1 to 100 000. In metres that is 1e8, where an `f32`
spacing is 2^(26−23) = **8 m**; at 20 000 km it is **2 m**. Godot's
documentation says shaders stay single precision, and `precision=double` needs
a custom engine build. gdext's builds would also need a double feature this
project does not use.

**Camera-relative rendering, from the first milestone.**

- The camera stays at the origin.
- Each instance's transform is `(node_origin − camera_position)`, computed in
  `f64`: GDScript floats are 64-bit, so the subtraction is done before the
  cast to `Vector3`.
- Heights are relative to the node.

Vertex positions near the camera are then small numbers, whatever the world
size. The research's floating-origin section says to build this in from the
start rather than retrofit it.

**Depth (24-bit, no reverse-Z, §2.2).**

- **Near and far planes are dynamic.** Near is
  `clamp(0.05 · height_above_ground, 1 m, 1 km)` and far is the horizon
  distance for that altitude, clamped to the world diagonal. Both are labelled
  judgements to calibrate in 3D-3.
- **Tested** by a z-fighting bar on the coast and ridge crossings (B5).
- **Fallback if it fails:** two cameras (a near and a far depth range).
- **Logarithmic depth written from the fragment shader is the last resort,**
  because writing `DEPTH` disables early-z.

**The world is flat.** The map is a rectangle, so no curvature is drawn.

### 3.12 Memory and vertex budget per tier

**Per resident layer pair:** colour 256² × 4 B = 262 144 B. Height
(256 + 2·pad)² × 4 B; `pad` is `render::tile_halo_px`'s, and is about 270 kB
for a pad of a few texels. Together that is about **0.53 MB**. CPU height
copies for picking add the height half again.

| Tier | Patch | Typical patches drawn | Triangles | Layer capacity | GPU tile memory | Source of each number |
|---|---|---|---|---|---|---|
| Basic | one 257² mesh over a level-z world texture | 1 | 131 k | 1 + 1 | ≈ 1 MB | Research milestone 1; measured 0.13 ms in §2.4 |
| Standard (phone) | 33² | ≤ 256 | ≤ 0.5 M | 64 | ≈ 34 MB | Fits LOD-D6's *"steady memory at most 60 MiB above the recorded steady state"* bar; 64 is a labelled judgement |
| Standard (desktop) | 65² | ≤ 512 | ≤ 4.2 M | 256 | ≈ 136 MB | Labelled judgement; §2.4 measured 1 024 × 65² at 1.9 ms |
| Full | 129² | ≤ 512 | ≤ 17 M | 512 | ≈ 272 MB | Labelled judgement; §2.4 measured 33.5 M triangles at 6.0 ms |

**The 2D tiles give way in 3D.** While 3D is on, 2D's LOD tiles are parked, so
3D's budget replaces 2D's rather than adding to it. The phone's generation
peak (878–908 MB at 2048 × 1311, `LOD_DETAIL_SCOPE.md` rules) must not move:
3D allocates nothing inside `generate`.

### 3.13 Streaming: only the nodes the camera can see

**The selection itself is visibility-driven.** This is the owner's "rendering
only what's visible" (2026-08-31, `LARGE_ITEM_RULINGS.md`):

- A node is considered only if its bounding box (footprint × §3.10's min/max
  height range) intersects the view frustum.
- It is refined only where `L_3d` asks for it.
- Nodes behind the camera or outside the view cost nothing beyond the box
  test.

**Order of work:**

1. **Requests go to `lod_worker`** ordered by projected screen area, then by
   EF-9's importance (Q2 of §8.6 already ordered synthesis by importance).
   They are capped by `LodBudget::max_in_flight`.
2. **Prefetch** asks for the next level where `L_3d` is within 0.25 of a
   boundary (§8.5's `PREFETCH_LEVELS`) and along the camera's motion vector.
3. **Hysteresis and parent fallback.** A layer is evicted only when it has
   been out of the selection for longer than the tier's park time, and a
   parent is never evicted while a child it covers is missing (LOD-D3).
4. **Occlusion** (later, Standard and up): a coarse horizon test from the
   min/max hierarchy, which drops nodes behind a ridge. This is research step
   5; it is not built until 3D-3's measurements show it pays for itself.

### 3.14 Interaction with the rest of the app

- **Mode switch.** 3D is a viewport mode. It is reached from `View ▸ 3D` and
  the phone's existing 2D/3D FAB (`viewport_host.gd`'s FAB anchor note).
- **Rows that become live.** These rows already exist and are disclosed as
  unavailable today:
  - `Preferences ▸ Graphics ▸ 3D viewport defaults` (relief exaggeration,
    detail, light, flatten oceans; `menus.gd`);
  - "Anti-aliasing · anisotropy" (MSAA 3D is supported under Compatibility,
    per the renderer docs);
  - the alternate-frame row.
- **Session mode** (Ruling CK Q5, `STORY_MENUS_PROPOSAL.md`) is a layout
  toggle. 3D does not change it, and a seeded layout may hold the 3D view.
- **Export.**
  - The 16K/32K export stays the top-down 2D bake (`EXPORT_SCOPE.md`).
  - A **3D screenshot** is new: a `SubViewport` rendered at N × the window
    size. Past the device's texture limit it is rendered in tiles by an
    off-axis frustum (`Camera3D.frustum_offset`) and stitched. Under
    Compatibility there is no limit query (§2.3), so tiles of at most
    4 096 px are used (a labelled judgement). It is written as PNG through
    `cartalith-assets`'s `PngColour::Srgb` tag, as other picture exports are.
- **2D features in 3D.** Each is disabled with a stated reason until its
  milestone makes it work. Every unavailable control carries a reason
  (`CommandIndex` convention). The reasons quoted here are the ones the gate
  shows:

| Feature | In 3D | Reason shown |
|---|---|---|
| Context card, measure (distance, vertical, cross-section) | works from 3D-5 | — |
| Sculpt brushes, WORLD paint, CARTO colour paint | disabled until a later row | "Brushes paint on the 2D map; switch to 2D to edit." Brushes need ray-picking under a moving camera with a draft preview, which 3D-5 does not build |
| Lasso, rectangle and region selection | disabled | "Selections are drawn on the 2D map." |
| Label drag, place moving | disabled | "Drag places on the 2D map." |
| LOD debug overlay | replaced by a 3D node-level debug view (3D-3) | — |
| Graticule, scale bar | scale bar hidden (no single scale under perspective); graticule draped (3D-6) | "No single scale in a perspective view." |

---

## 4. The capability gate

The owner's rule: *"if a adreno 630 can't bring enough power 3D rendering
should be disabled based on a machines power"*. The rest of that ruling (a
gate, not a ceiling) is in `LARGE_ITEM_RULINGS.md`. The research's tiers were
keyed to the renderer name. Re-cast as its "Status: parked" note asked, they
key off **measured capability**, so that moving the minimum device is a
policy constant.

### 4.1 Tiers

| Tier | What it requires (all measured by the probe, §4.2) | What it gives |
|---|---|---|
| **3D Off** | Any probe control fails, or Basic's bar fails | 2D only. The 3D entry points stay visible and say why |
| **3D Basic** | VTF control passes (or the V2 fallback works); Basic workload p95 ≤ 33 ms | One displaced mesh, map-shading drape (unlit), orbit camera, picking for the context card |
| **3D Standard** | Basic, plus the `Texture2DArray` per-instance control; Standard workload p95 ≤ 33 ms (phone class) or ≤ 16.7 ms (desktop class); the layer budget allocates without a GPU memory error | §3 in full: quadtree, morph by `L_3d`, lit drape, overlays |
| **3D Full** | Standard, plus the Full workload p95 ≤ 16.7 ms at the window's own resolution | 129² patches, shadows, larger cache and far range. **GPU-driven culling is not here.** It needs a renderer switch (§2.3) |

**Where the bars come from:**

- 33 ms is LOD-D6's phone bar ("p99 frame ≤ 33 ms").
- 16.7 ms is one 60 Hz frame.
- "Phone class" and "desktop class" are read from `OS.get_name()`. The adapter
  type cannot be used: it reads 0 under Compatibility (§2.2).

### 4.2 The runtime probe

- **When it runs:** on first entry to 3D, not at boot. It also re-runs when
  the adapter name, the driver string or the app version changes, and from a
  "Re-test 3D" button.
- **What it does.** It renders into the real window for about 3 s (an
  off-screen render cannot be timed under Compatibility the same way):
  1. **VTF control.** Draw the 257² patch at `amp` 0 and then 4. Some pixels
     must move, as in §2.4.
  2. **Array control.** Update one `Texture2DArray` layer; pixels must move.
  3. **Workloads.** Basic: one 257² mesh. Standard: 256 × 33² (phone) or
     512 × 65² (desktop), one `MultiMesh`. Full: 512 × 129². Each runs
     180 frames after 60 warm-up frames; the probe records the median and the
     p95.
  4. **Memory.** Allocate the tier's layer arrays and confirm with a readback
     of one layer. Record `Performance.RENDER_VIDEO_MEM_USED` as "this app's
     usage", not as a device ceiling (research §"Capability detection").
- **Result.** The highest tier whose controls and bars all pass. It is stored
  in settings with the measured numbers. **It is suggested, never forced, and
  never above what passed.** This follows `recommended_quality_tier()`'s "never
  auto-apply" pattern (research §tiers).

### 4.3 What the gate tells the user

Every state names its measured cause, in the `CommandIndex` "every
unavailable entry carries a reason" shape. The wording is a draft, to be
written with the `design:ux-copy` skill when built:

- *"3D is off on this device: drawing terrain from height textures did not
  work here (renderer: Compatibility, OpenGL ES 3.2)."*
- *"3D is off on this device: a test view of 131 000 triangles took 48 ms per
  frame; 3D needs 33 ms or less."*
- *"3D runs in Basic mode here: the test view for Standard took 41 ms per
  frame. Re-test, or force Standard."*
- *"3D Standard. Full needs 16.7 ms per frame; this device took 22 ms."*

### 4.4 Forcing a higher tier

**"Force tier…" is available at every gate**, as the owner instructed, with
the research's warning wording. The warning names the failure classes this
project has already hit, not only "may be slow": a driver reset
(`VK_ERROR_DEVICE_LOST`), and the AMD Compatibility crash in
`GUI_GAP_REGISTER.md` §14.1.

**A forced tier that fails** (shader compile, a texture allocation, or a frame
over 250 ms) drops one tier and says so. **It never crosses the gdext boundary
as a panic** (`cartalith-rust-conventions`).

### 4.5 The Android floor: what "a base amount of processing power" could mean

The owner's phrase, made measurable. Each of these is a proposed default, and
the owner may move any of them:

| Measure | Basic | Standard | Why this measure |
|---|---|---|---|
| Frame time p95 at the probe workload | ≤ 33 ms | ≤ 33 ms | One 30 Hz frame; LOD-D6's phone bar |
| Vertices processed per second (workload × fps) | ≥ 2 M/s (66 k × 30) | ≥ 8 M/s (256 × 1 089 × 30) | Throughput independent of how the workload is shaped |
| Draw calls per frame for terrain | 1 | ≤ 11 (one `MultiMesh` per level) | Fixed by design, so a check rather than a bar |
| Frame time with N resident chunks: 64 layers sampled | — | p95 ≤ 33 ms | Texture bandwidth, which the Adreno shares with the CPU |
| Memory above 2D steady state | ≤ 8 MB | ≤ 60 MiB | LOD-D6's bar |
| Thermal: 5-minute orbit, frame time at the end against the start | — | ≤ 1.5× | LOD-D6 records thermal behaviour but sets no bar; 1.5× is a labelled judgement |

**Where these can be measured:**

- **The emulator cannot measure them.** `ANDROID_BUILD_SCOPE.md` records no
  Android emulator (grep for `emulator` and `avd`: no hits). An emulator's GPU
  is the host's, translated, so it can only exercise the probe's **code
  paths**: does each branch run, does each disclosure string render. It
  cannot produce a single number for these bars.
- **The OnePlus 6T is where the bars are measured.** It is the only handset on
  the bench, with the Adreno 630, `ANDROID_BUILD_SCOPE.md`.
  - Drive it with `adb exec-out screencap` and `adb shell input` at
    coordinates read off the image.
  - Read logcat for the stale-library warning first (`MISTAKES.md`, rows 155
    and 170).
  - Record the installed build's `lastUpdateTime`.

**The 6T's tier is unknown until 3D-0(c) runs.** Godot's documentation makes
it a Mobile-class device (§2.3), and in the 2D app it holds 60 fps at the
Performance tier (`ANDROID_BUILD_SCOPE.md`, "In-app Performance readout").
Neither is a 3D measurement.

---

## 5. Milestones

**Rules for every milestone:**

- **Each fits one lane plus a verifier** (`LOD_DETAIL_SCOPE.md`'s rule).
- **It moves no golden.** 3D is a pure consumer of verified fields
  (research §E).
- **Its probes run windowed.** Headless has no rasteriser and never fires
  `frame_post_draw`.
- **Its code is annotated per Ruling BK.**
- **Its screenshots are opened and looked at, at several camera heights,**
  before it is called done (`MISTAKES.md`, "Verify a visual change"; the
  2026-10-17 zoom requirement).

### 3D-0 · Measurement spike: go/no-go. No shell change

Run from a scratch `git worktree`, never the live tree. Three parts:

- **(a) The isolation control** that 2026-09-01 left open. One generate under
  `forward_plus` with `use_gpu=false`, made in the worktree's copy of
  `engine_bridge.gd`. It tells whether wgpu causes the device loss. **This
  does not gate 3D.** It decides whether Q2's renderer question can ever
  reopen.
- **(b) A desktop prototype on a real world.**
  - **World:** seed 24601 at 2048 × 1311, plus one 512 × 384 world.
  - **Scene:** a probe scene with a `SubViewport`, a `Camera3D` and one
    `MultiMesh` of 65² patches. It draws the level-z0 tiles of the current
    view, from heights taken through a temporary engine accessor beside
    `tile_heights` (worktree only), with the 2D colour tiles as the drape.
  - **Measured:** frame time (median, min..max, three separate runs),
    triangles, draw calls, GPU and CPU memory, and tile build time per tile.
  - **Captured:** one oblique frame at three camera heights.
- **(c) A Basic workload on the 6T.** The same §2.4 probe packaged as a
  throwaway APK (a separate package name, so the owner's install is
  untouched). It records the Basic and Standard workload numbers against
  §4.5.

**Go if** desktop (b) meets Standard's 16.7 ms p95 with margin **and** the
oblique frames show the tile detail on the surface.

**No-go** (re-scope before 3D-1) if desktop Standard cannot reach 16.7 ms on
this machine.

**The phone result does not decide go or no-go.** It sets the phone's
starting tier (Off and Basic are both legitimate answers under the owner's
ruling).

**Size:** small. **Parity class:** measurement only.

### 3D-1 · The gate and an empty 3D mode

- `View ▸ 3D` and the FAB switch to a `SubViewport` holding a `Camera3D`.
- The §4.2 probe is built, the tier is stored, and every §4.3 string is shown
  with its measured numbers.
- **Built before any terrain**, so "3D unavailable, here's why" works from the
  first day (research §D, Milestone 0).

**Acceptance:**

- The probe's positive controls fail when VTF is stubbed out.
- Each tier's disclosure renders.
- Forcing a tier then failing drops one tier without a crash.
- `command_index` counts the 3D rows with reasons.

### 3D-2 · Basic: one displaced mesh, map-shading drape, orbit, camera-relative from the start

- One 257² patch over a level-z world height texture, with the 2D colour
  texture as the drape. Camera-relative transforms from the start.
- Orbit and top-down cameras, and the ground clamp.

**Acceptance:** B4 (no frame below the ground), B5 (depth), and Basic's frame
bar on desktop.

### 3D-3 · Standard geometry: quadtree, `L_3d` morph, layer arrays, culling

- **Engine:** `LodSnapshot` returns heights beside RGBA from one job (§3.1),
  with the determinism test extended to the heights.
- **Shell:** the `ChunkId` quadtree and its selection rule, the
  `Texture2DArray` layer cache with parent fallback, the CDLOD morph by `L_3d`
  (§3.3), skirts, frustum culling by the min/max hierarchy, prefetch, and a
  node-level debug view.

**Acceptance:** B1, B2, B3, B6, and Standard's frame and memory bars.

### 3D-4 · Lit drape, sea and atmosphere

- The hillshade-off tile request, with its own cache key; per-pixel normals;
  one `DirectionalLight3D` from the look's sun.
- Below-sea flattening with the water-mask shore; `WorldEnvironment` fog.
- A shadowed light at Full only, after a look at the Compatibility shadow path
  (research §C).

**Acceptance:**

- **B7:** normals continuous across tile seams.
- The coastline is in the same place as in 2D (B1 run on the shore pixels).
- Frames read in every look, including Satellite relief.

### 3D-5 · Picking and the read-only tools

Ray-terrain picking (§3.10), the context card, the measure tools, and the
disclosures for disabled 2D features (§3.14).

**Acceptance:** B8, and every disabled control carries its reason.

### 3D-6 · Overlays

Settlement and icon billboards (`MultiMesh`), `Label3D` labels, draped road,
border and route ribbons, and the draped graticule.

**Acceptance:**

- No ribbon is visibly buried or floating.
- Labels follow the 2D tier visibility.
- Frame bar held with overlays on.

### 3D-7 · Device pass and tier calibration

- On the 6T: every §4.5 bar, plus the phone gestures driven on glass, never by
  `pressed.emit()` (`MISTAKES.md` row 155).
- Re-calibrate the tier thresholds from the numbers.
- Record thermal behaviour.

**Acceptance:** the phone reaches the tier the probe chose, and every gesture
works on glass.

### 3D-8 · 3D screenshot export and Session mode

The tiled off-axis screenshot (§3.14) and Session mode layout support.

**Acceptance:** a stitched 3D screenshot shows no seam at tile boundaries;
this is B3's metric run on the stitch lines.

**Deferred, not scheduled:**

- river ribbons for vector-line looks;
- brushes in 3D;
- GPU-driven culling, which needs a renderer switch (§2.3) and a placed-object
  system.

### 5.1 Acceptance bars

**Every threshold here is a proposed default, calibrated in 3D-0 and 3D-3.**
The metrics are LOD-D0's wherever one exists.

| Bar | What | Threshold | How it is measured |
|---|---|---|---|
| **B1** 2D/3D agreement | A top-down orthographic 3D frame at the 2D view's scale, map-shading drape, overlays off, against the 2D frame of the same view | Mean \|ΔL\*\| ≤ 1.0 over map pixels, 99th percentile ≤ 4.0; level index per pixel equal on ≥ 99.9 % | Both frames captured windowed; three seeds; zooms ×4, ×16 and ×64 |
| **B2** no cracks | Clear the frame to magenta, then fly through level changes | **0** magenta pixels inside the terrain silhouette on every frame | Scripted camera path over three seeds |
| **B3** no pops | LOD-D0's `T_i`, warping frames by depth-buffer reprojection | **0** pops; level-boundary `T_i` ≤ 1.5 × the median of neighbouring frames | The D3 bar, on a 3D path |
| **B4** ground clamp | Camera height above the displayed surface | 0 frames below it | Scripted fly path |
| **B5** depth | Z-fighting along coastlines and ridge crossings at the highest and lowest altitudes | 0 flickering pixels between two frames of a static camera | Frame diff, static camera |
| **B6** zoom gains detail | LOD-D0's detail per screen pixel near the camera, at camera heights equivalent to ×4, ×16 and ×64 | ≥ the 2D value at the same `px_per_cell` | The hard requirement of Ruling CI item 1 |
| **B7** seams in shading | Normal difference across a tile seam | ≤ the median difference between neighbouring columns × 1.5 | The LOD-D0 seam-ratio method |
| **B8** picking | Ray hit against the true surface point (rendered depth at the cursor) | ≤ 0.5 screen px reprojection error at 95 % of samples | 1 000 random screen points per view |
| **Frame time** | Per tier, §4.1 | Basic and phone-Standard p95 ≤ 33 ms; desktop Standard and Full p95 ≤ 16.7 ms | Median, min..max and p95; the harness alone |
| **Memory** | 3D steady state above 2D steady state | Phone ≤ 60 MiB; desktop ≤ the tier's §3.12 figure + 10 % | Host-side peak resident (`MISTAKES.md`, memory row) |

---

## 6. Risks

- **The 6T may land in Basic or Off.** That is an acceptable outcome under the
  owner's ruling; it is not a design failure. The risk is a gate that reads it
  wrong, which is why the probe uses the real window and real workloads.
- **The cost of the `z0 + 1` layer carries over.** EF-9's extra layer already
  outgrows the 2D parked-tile cache (§8.5: 54 promoted tiles live against 35
  parked). Perspective views see more ground than top-down, so the layer
  budget in §3.12 may need raising. 3D-3 measures it.
- **Compatibility shadows have documented rough edges** (research §C). Shadows
  are held to Full and to a device look.
- **24-bit depth may not hold at the longest views.** The fallbacks are in
  §3.11.
- **Two worlds of overlay code.** A `_draw()` overlay and its 3D mesh will
  drift unless both read one render list. 3D-6 must consume the same list, not
  a copy (`MISTAKES.md`, "two views of one state will disagree").
- **Exposing a private builder.** `tile_heights` going public is an API change
  in `cartalith-godot`. It stays crate-internal, with the worker its only new
  caller.
- **The device loss stays unexplained.** Nothing here depends on Forward+, but
  if 3D-0(a) shows that the generate trips a GPU reset whoever else is on the
  device, that is a defect worth its own row.

---

## 7. Owner questions, each with the default this scope assumes

**Searched before asking:** `LARGE_ITEM_RULINGS.md`, for `3D`, `vertex`,
`Adreno` and `Aletsch`. Only the 2026-08-31 rows, Ruling K and Ruling AZ
touch 3D, and none of them answers Q1 to Q10.

1. **Q1. Which reading of "with vertices"?**
   - **Default: V1.** The display mesh is displaced from the raster's tiles
     and the simulation stays a raster (§1).
   - V3, a vertex world, would overturn the elevation-field architecture and
     the parity contract.
2. **Q2. The renderer.**
   - **Default: stay on `gl_compatibility`.** Revisit only if 3D-0(a)
     attributes the device loss and it is fixed. GPU-driven culling stays
     deferred until then.
3. **Q3. Lit or map-shaded by default** (from Standard up).
   - **Default: lit**, with the 2D hillshade suppressed and "Map shading (as
     2D)" one toggle away.
   - Basic is always map-shaded.
4. **Q4. Vertical exaggeration.**
   - **Default:** the existing `Preferences ▸ Graphics ▸ relief exaggeration`
     value, with a live slider in 3D. The measure tools always report true
     values.
5. **Q5. The phone.**
   - **Default:** the gate decides. Expect Basic until 3D-7 measures.
   - "Off on every phone" is the other legitimate choice.
6. **Q6. Overlays in the first release.**
   - **Default:** settlements, icons and labels in 3D-6. Roads, borders and
     routes come in the same milestone, after them.
7. **Q7. Editing in 3D.**
   - **Default:** read-only tools only (context card, measure). Brushes and
     selections are disabled with a reason and filed as a later row.
8. **Q8. Rivers.**
   - **Default:** painted into the drape (Ruling BU). Ribbons only for a
     vector-line look, later.
9. **Q9. The entry camera.**
   - **Default:** top-down at the current 2D view, then tilt (Ruling K's
     top-down target). Orbit is the default mode after entry.
10. **Q10. Approving this scope, and `DECISIONS.md` §4.**
    - **Default:** on approval, record a new `DECISIONS.md` entry. It would say
      that §4's timing reason is satisfied, that its cost reason stands, and
      that it is paid by this staged, gated plan. 3D-0 is then scheduled.
    - Until approval, nothing is built and §4 stands.

---

## 8. 3D-0 measurements (2026-10-19) and the go/no-go for 3D-1

This section records what 3D-0 measured and what it means for the design.
It is design input, like §2.4. Whether 3D-0 is done is `STATUS.md`'s answer.
Raw output, the probe, the shader and the worktree-only accessor are in
`cartalith-native/docs/evidence/3d0-2026-10-19/` (`results.txt`,
`td0_probe.gd.txt`, `td0_terrain.gdshader.txt`, `td0_accessor.patch`). The
captures there are numbered 01 to 09.

**Setup.** The dev desktop as in §2.4, but on Godot **4.7.2** (§2.4 used
4.7.1). The build was a worktree of `7025fdba` plus the accessor, as a
release DLL. A scratch `gl_compatibility` project, windowed 1600 × 900 with
vsync off. Seed 24601, 1 200 km wide, at 2048 × 1311 and 512 × 384, in the
"Natural Vibrant" look. Each view draws the level `z0` that 2D draws at the
same `px_per_cell` (`lod_level_for_zoom`), as the 256 tiles nearest the
pivot in front of the camera. They are one `MultiMesh` of 65² patches, with
heights and colour from two `Texture2DArray`s filled by one `tile_heights`
call per tile (§3.1). A sunk backdrop at `z0 − 4` makes a second draw.
Camera-relative throughout. The exaggeration is 3.4 (the relief default)
unless stated.

### 8.1 (b) Desktop prototype: the numbers

Frame time is the median (min..max) of 300 frames, and p95. There were three
separate processes per world, and the spread across them is given.

| Workload | Triangles | Draws | Median, 3 runs | p95, 3 runs | GPU texture memory |
|---|---|---|---|---|---|
| ×4 view, `z0` = 5 (2048) | 2.13 M | 2 | 0.782–0.785 ms | 0.855–0.859 ms | 137.6 MB |
| ×16 view, `z0` = 7 (2048) | 2.36 M | 2 | 0.980–1.061 ms | 1.057–1.116 ms | 150.9 MB |
| ×64 view, `z0` = 9 (2048) | 4.19 M | 2 | 1.525–1.575 ms | 1.582–1.666 ms | 250.7 MB (512 layers with the backdrop) |
| **Standard (desktop), 512 × 65², one `MultiMesh`** (2048) | 4.19 M | 1 | 1.315–1.333 ms | **1.390–1.404 ms** | 249.6 MB |
| Standard, the same (512 × 384) | 4.19 M | 1 | 1.303–1.327 ms | 1.370–1.402 ms | 292.4 MB |

- **Tile build** (colour and heights together, one tile, main thread,
  release): median 7.13–7.15 ms on 2048, from 1 572–2 244 tiles per run
  (min 1.67, max 34.91). On 512 × 384 it was 8.33–8.92 ms. A cold view of 256
  tiles is about 1.8 s of synthesis, so §3.13's streaming and parent
  fallback are needed. A synchronous fill is not an option.
- **Memory.** Texture memory per layer pair is about 0.5 MB, which agrees
  with §3.12's 0.53 MB. Host working set was 594–628 MB on the 2048 world,
  with the generate included, and the peak was 782–884 MB.
- **VTF on real data.** Exaggeration 0 against 3.4 moved 191 509–359 943
  sampled pixels per view, so the vertex-texture fetch works with the
  shipped tiles.
- **No holes.** 0 sky pixels in the lower 40 % of every frame. That is a hole
  indicator, not B2.

### 8.2 Detail at every zoom level (the hard requirement)

**At every camera height, the level 2D draws carries more detail than a
coarser one.** The metric is the mean luminance gradient per screen pixel in
the near field. At each camera, the drawn level `z0` is compared with the same
camera drawing tiles three levels coarser:

| World | ×4 | ×16 | ×64 |
|---|---|---|---|
| 2048 × 1311 (exaggeration 3.4) | 0.878 vs 0.580 (1.51×) | 3.756 vs 0.714 (5.26×) | 7.545 vs 1.301 (5.80×) |
| 2048 × 1311 (exaggeration 1.0) | 1.43× | 4.42× | 6.13× |
| 512 × 384 (exaggeration 3.4) | 6.320 vs 0.471 (13.4×) | 4.383 vs 0.684 (6.41×) | 1.496 vs 0.692 (2.16×) |

- **On 2048, the near-field detail rises at every step from ×4 to ×64.**
- **On 512 it falls from ×4 to ×64**, although at each height the drawn
  level still beats the coarse one. In the ×64 frame (capture 06) the
  near field is steep spires, and the drape is stretched down their faces.
  This is texture stretch on steep geometry, which is §8.3's finding seen from
  the drape's side. The three views also frame different ground, so this is
  not B6. B6 (3D detail ≥ 2D at the same `px_per_cell`) is still 3D-3's bar.

### 8.3 Finding: the tile heights are shading detail, and are too rough as geometry

The deep-zoom tile heights were tuned to be shaded in 2D. Displaced as
geometry they read as needles (captures 02, 03 and 08, coastal escarpments
first). True-scale land slope between neighbouring texels, from the tiles
each view drew (2048 world):

| View (texel spacing) | p50 | p90 | p99 | Over 60° |
|---|---|---|---|---|
| ×4, `z5` (about 147 m) | 1.6° | 8.6° | 19.1° | 0.015 % |
| ×16, `z7` (about 37 m) | 12.2° | 38.6° | 70.6° | 2.2 % |
| ×64, `z9` (about 9 m) | 25.1° | 59.0° | 80.1° | 9.5 % |

**What it means for 3D-3:**

- A lowland plain whose median slope is 25° at 9 m spacing is not ground
  that reads as real. At exaggeration 3.4 it turns to spires everywhere; at
  1.0 it is rugged, and the coastal bluffs still spike (captures 07 and 08).
- §3.2 already says geometry carries the silhouette and textures carry the
  detail. 3D-3 needs that made concrete, and it is not decided here. The
  candidates are:
  - (a) displace a low-passed height, one or two levels coarser than the
    drape, and take the full tile into per-pixel normals (§3.4);
  - (b) a geometry amplitude law separate from the shading one.
- Either choice changes no 2D pixel, because 3D only reads the tiles.
- This is also owner question Q4: 3.4 is the 2D hillshade default, and as a
  3D vertical scale it produces needles.

### 8.4 (a) The isolation control

The shell's `_shot.tscn -- --generate` was run (512 × 384, seed 483920) under
`forward_plus`/Vulkan at 1280 × 800. It ran from a scratch copy whose
`engine_bridge.gd` boot line was switched by an environment variable:

- `use_gpu = false`: 3 of 3 clean, with no wgpu adapter in the log.
- `use_gpu = true`: 3 of 3 clean. The wgpu Vulkan adapter came up in each
  run, and there was no `DEVICE_LOST`.

**The 2026-09-01 device loss (3 of 3 then, on Godot 4.7.1 with the same
driver, 26.7.1) does not reproduce on 4.7.2.** So the control cannot blame
wgpu or clear it, because there is no failure left to attribute. Q2's
renderer question may reopen on evidence, but the cause was never found.
Two things changed between the runs: Godot 4.7.1 → 4.7.2, and the shell. A
renderer switch would need its own soak test first. None of 3D-1 to 3D-8
needs it.

### 8.5 (c) The 6T

**Not run.** `adb devices` listed no device on 2026-10-19, so no throwaway
APK was built. The phone's starting tier is still unknown. This does not gate
go or no-go (§5, 3D-0).

### 8.6 Go/no-go for 3D-1

**GO.** The bar is "desktop (b) meets Standard's 16.7 ms p95 with margin
**and** the oblique frames show the tile detail on the surface".

- **Frame time:** Standard measured p95 1.370–1.404 ms in six runs, a 12×
  margin. Every view workload was at most 1.83 ms p95.
- **Detail:** shown at ×4, ×16 and ×64 on both worlds (§8.2), with
  captures 01 to 06 opened and looked at.

**Carried into later milestones, not into 3D-1:**

- §8.3's geometry roughness (3D-3, with Q4);
- the 6T numbers (3D-0(c), owner hardware);
- the cold-fill cost of about 7 ms per tile (3D-3's streaming).

---

## Sources

**Project, opened at the symbol:**

- `cartalith-gpu/src/multi.rs` (`COMPUTE_BACKENDS`, `compute_instance`);
  `cartalith-godot/src/lod_bridge.rs` (`TILE_PX`, `MAX_LEVEL`,
  `continuous_level`, `tile_heights`, `synthesize_tile_water_mask`);
  `lod_worker.rs` (`LodBudget`, `LodSnapshot::render_tile`); `render.rs`
  (`zoom_detail_km_law`, `LOOK_PRESETS`, `render_biome_tile_rgba_water`);
  `cartalith-engine/src/elevation.rs` (`world_elevation_tile`,
  `world_sample_elevation`); `continuous_lod.rs` (`BoostField`); `lib.rs`
  (`measure_vertical`, `get_map_height_km`).
- `godot-project/shell/menus.gd` (the "3D viewport defaults" and
  "Anti-aliasing · anisotropy" rows), `engine_bridge.gd` (`use_gpu`),
  `new_world_dialog.gd` (Width 1–100 000 km).
- `git show HEAD:…/project.godot`.

**Local dependency sources (read in `~/.cargo/registry`):**

- `wgpu-30.0.0`: `Instance::from_hal`, `Adapter::create_device_from_hal`,
  `Device::create_texture_from_hal` / `create_buffer_from_hal`.
- `wgpu-hal-30.0.0`: `vulkan::Adapter::device_from_raw` and its safety
  contract, `required_device_extensions`, and the optional `external_memory_*`
  extensions.
- `gdextension-api-0.5.1/src/4.7/extension_api.json`: `RenderingDevice`'s
  `DriverResource` enum, `Features` enum, `texture_create_from_extension`,
  `draw_list_draw_indirect` and `compute_list_dispatch_indirect`;
  `RenderingServer.call_on_render_thread`, `texture_create_from_native_handle`
  and `get_video_adapter_*`.

**External:**

- Vulkan specification, *Fundamentals* — threading behaviour (externally
  synchronised parameters, `vkQueueSubmit`'s `queue`):
  <https://docs.vulkan.org/spec/latest/chapters/fundamentals.html>
- Godot 4 class reference, `RenderingDevice` — not available under headless
  or Compatibility:
  <https://docs.godotengine.org/en/stable/classes/class_renderingdevice.html>
- Godot 4 documentation, *Overview of renderers* — Compatibility has no
  compute, 24-bit depth without reverse Z, MSAA 3D supported, no HDR output:
  <https://docs.godotengine.org/en/stable/tutorials/rendering/renderers.html>
- Godot 4 documentation, *Large world coordinates* — shaders stay single
  precision; `precision=double` needs a custom build:
  <https://docs.godotengine.org/en/stable/tutorials/physics/large_world_coordinates.html>
- A. Asirvatham and H. Hoppe, *Terrain Rendering Using GPU-Based Geometry
  Clipmaps*, GPU Gems 2 ch. 2 — vertex-texture-fetched elevation, transition
  morph:
  <https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-2-terrain-rendering-using-gpu-based-geometry>
- F. Strugar, *CDLOD* (paper, data and source) — a quadtree of regular grids,
  no stitching: <https://github.com/fstrugar/CDLOD>
- T. Ulrich, *Rendering Massive Terrains using Chunked Level of Detail
  Control*, SIGGRAPH 2002 course notes:
  <https://api.crossref.org/works/10.1145%2F2077378.2077415>. Skirts as
  described in J. Schneider and R. Westermann, *GPU-Friendly High-Quality
  Terrain Rendering* (WSCG 2006):
  <https://www.cs.cit.tum.de/fileadmin/w00cfj/cg/Research/Publications/2006/GPU-Friendly_High-Quality_Terrain_Rendering/wscg06.pdf>.
  The year of Ulrich's work is inconsistent across sources (2000 against
  2002), noted rather than resolved.
- Godot PR #77496 (3D shadows in Compatibility) — cited from
  `3D_TERRAIN_RENDER_RESEARCH.md` §C, not re-fetched.
