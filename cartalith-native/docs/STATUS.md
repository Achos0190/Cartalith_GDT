# Status quicklist

A living checklist, not a narrative — read this first, before `CHANGELOG.md`,
to know what's done vs. open without re-reading the whole history each
session. Update it in the same commit as whatever changes its answer.
`CHANGELOG.md` stays the detailed record of *how*; this is only *what/done?*.

Last updated: 2026-08-16 (post Phase 2 milestone 6: settlement-suitability prerequisites).

## MVP_SCOPE.md — "done means all seven"

| # | Criterion | Status |
|---|---|---|
| 1 | Height/temp/rain/flow match golden data | **Done.** Every pipeline stage golden-verified bit-exact/tight-tolerance against the real JS engine: tectonics/orogeny (graph-driven T1-T5), volcanism+provinces, climate (temp/wind/rain), ocean currents, terrain wind deflection, erosion, hydrology, world-structure archetypes, full carve pipeline. Nothing left pinned to a stale default. |
| — | UI/UX (not one of the seven, but part of the `/goal` "feature and graphic parity" directive) | **Reskinned 2026-08-16, then re-themed same day per explicit owner feedback.** First pass: `ui-ux-pro-max` dark-dashboard design system, grouped World Parameters/World Structure/Advanced cards, visible keyboard-focus states. Owner preferred the reference HTML's own look, so the palette was swapped to a literal port of the reference's real `:root[data-theme="light"]` parchment theme (`#efe7d6`/`#fbf5e9`/`#b07f3f` accent) — not a fresh design-system search, the actual CSS values from `Cartalith Gen1 v2.10.html` line 271. Confirmed by real-window screenshot that the map's own pixels are untouched by the theme swap (JS/Rust colour ramps, not CSS/Theme — same guarantee the reference's own code comment makes). Deferred: real Fira font files (license-unverified, kept Godot's default font), and `MVP_SCOPE.md` point 9 (sea level) still isn't user-adjustable from Godot — needs a new `#[func]` binding, not done this pass. See `CHANGELOG.md`'s UI reskin and "real Windows hands-on verification" entries. |
| 2 | Recognisable 2D map render | **Done (2026-08-16).** Replaced the placeholder elevation-only tint with the reference's real default-settings biome/hillshade renderer (`crates/cartalith-godot/src/render.rs`, new): `materialWeights` (snow/rock/sand/wetland/canopy/grass), the six climate-selected colour ramps, multi-scale hillshade, `bioBlend` desaturation, edge haze, and `seaColorCore` (smoothed-bathymetry depth/temperature banding — confirmed this is JS's real default, not a stretch feature). Two real bugs caught by golden verification, not by read-through: a missing final `ao*vignette` multiply (~40% too bright at corners) and sea colour needing the smoothed, not raw, depth field. Golden-verified against two real `generate()` runs at `1e-4` tolerance (`golden_parity_render.rs`). Deliberately excludes every `state.viz.*`-gated stretch feature (splat texturing, geology, NPR "Painter" styles, AO/SVF/shadow, SDF tinting) — all off at JS's own defaults; that's genuine Phase 3 scope, see below. |
| 3 | Windows `.exe` builds + owner has run it | **Done (2026-08-16).** Ran the actual windowed MVP UI (not `--headless`) on this session's real Windows desktop: launched, screenshotted via `PrintWindow`, drove real synthetic mouse clicks at real screen coordinates. Confirmed generation end-to-end (real biome-coloured map, correct status label) under the new light theme. Caught two real bugs this way that no amount of code review had surfaced: the World-Structure dropdown rendered blank (malformed hand-authored `.tscn` item properties; GDScript's negative-index fallback meant it may have silently been generating with the `Rift` archetype instead of `Classic` this whole time), and the window title was still "walking skeleton". Both fixed and re-verified by the same screenshot method. See `CHANGELOG.md`'s "real Windows hands-on verification" entry. |
| 4 | Android `.apk` builds + owner has installed/run | **Apk builds and packages, confirmed.** Install+run on *real hardware* is not reachable from this environment — investigated via emulator, root-caused as a SwiftShader/emulator limitation, not our code (see `CHANGELOG.md`'s Android emulator entries). Per the `/goal` set 2026-08-16, this is **no longer a hard requirement** — testing via Godot editor/headless and local Android Studio is sufficient for now. |
| 5 | Map width scales feature size | **Done** — a consequence of criterion 1's parity, verified via the world-structure archetype port. |
| 6 | Changelog entry per milestone | **Ongoing** — `CHANGELOG.md` has an entry for every milestone so far; keep this up. |
| 7 | Opens a real HTML-app `.zip`, renders it, checked against the HTML app's own output | **Done (2026-08-16).** `cartalith-io::load_save` verified bit-exact against a real export produced by running the actual, unmodified reference engine (not just its own synthetic round-trip tests): `crates/cartalith-io/tests/golden_parity_real_export.rs` against `crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`. See `CHANGELOG.md`'s "cartalith-io verified against a real HTML-app export" entry for the harness technique (including a genuine `generate()`-name-collision gotcha found along the way). |

## ROADMAP.md phases

| Phase | Status |
|---|---|
| 0 — Walking skeleton | **Done.** Triangle/button/`ping()` confirmed on Windows and Android (build+package; Android run-on-device is the one open half, see criterion 4 above). |
| 1 — Terrain MVP | **7/7.** Criteria 1/2/3/5/6/7 done, criterion 4 softened by the 2026-08-16 `/goal` (no longer a hard requirement) and otherwise blocked purely on owner phone access. Two "easy to forget" Phase-1 closeout items from `ROADMAP.md` are still **not started**: a credits screen, and a licence audit of the crates pulled in (`PROVENANCE.md`). |
| 2 — Civilisation layer | **Started 2026-08-16, milestones 1–5 of an unknown-but-large number done.** `PHASE2_SCOPE.md` traced `currentSettlementSuitability`'s (the "v1.30 one function" `ROADMAP.md` names) real dependency chain first and found it several milestones away — matching the reference's own v0.104 history, which drew the same boundary. **Milestone 1** (`cartalith-civ` crate, zero `gdext` dep): lithology classification (bit-exact), soil fertility, water access (both `1e-4` tolerance). Found and fixed a real gap: `WorldState` never retained the raw per-cell plate-crust array past `generate_terrain`; added as `WorldState.crust_field`. **Milestone 2**: `build_water_bodies` (ocean/lake/land classification) — a connected-components flood fill plus a priority-flood depression fill using a hand-ported min-heap (`PROVENANCE.md` already flagged this exact algorithm's tie-break sensitivity; index-for-index/comparison-for-comparison port, not swapped for a stdlib heap). Found and root-caused a real harness bug during extraction (not a fixture mismatch): the reference's own `state` literal defaults `tect.seed` to `Math.random()`-derived at script load — the harness had been setting a nonexistent top-level `state.seed`, leaving generation genuinely nondeterministic across runs until fixed to set `state.tect.seed`. **Milestone 3**: biome classification (`classifyBiome`/`buildBiomeRaster`, 12 climate categories + water-body overrides) — bit-exact, both cases passed first attempt; harness called the reference's own `buildBiomeRaster()` directly rather than reimplementing its composition, and cross-checked against milestone 2's already-verified field/count data two independent ways. **Milestone 4**: `buildCarryingCapacity`/`buildNPP`/`estimateRegionalDensityKm2` (`1e-4` tolerance, both cases passed first attempt) — `buildResourcePotentials` split out into its own milestone 5 after checking its real size (~108 lines, 9 resource-type rules). Caught a real short-circuit-vs-blend gotcha porting the biome-residual gate (`bK&&biome` requires *both* truthy, not just arithmetic that happens to match at the default). Harness needed an explicit `allocate()` call before `generate()` (bypassing the UI means nothing else sizes `field`/`GW`/`GH` first) — found via a `RangeError`, not assumed. This harness's `field[0..5]` cross-validated against *three* independently-built extractions across three different milestones/sessions (`golden_parity_carve.rs`, the `cartalith-io` real-export fixture, and this one) — strong evidence the extraction technique itself is sound. **Milestone 5**: `buildResourcePotentials` (all 15 fields — copper/tin/iron/gold/salt/timber/lead/silver/clay/buildstone/flint/obsidian/gems/sulfur/alum — `1e-4` tolerance, both cases first attempt). Needed the same `WorldState`-retention fix milestone 4 predicted: added `boundary_type`/`shear_field` (from `cartalith-terrain`'s `StressResult`, previously computed but discarded past `generate_terrain`), matching `crust_field`'s own milestone-1 precedent. Verified the production scarcity default with a dedicated test, not by inspection: the original six resources are genuinely unthinned by default, only the nine v1.31 additions are. **Milestone 6**: `buildRouteCorridors`/`buildLandmassQuality`/`buildCoastSDF` — the last three affordance fields settlement suitability's real `ctx` needs. Found and root-caused another real harness bug before trusting the data (not just "close enough"): first extraction reproduced `field[0..5]` ~1e-5 off the trusted fixture — root cause was `golden_parity_carve.rs`'s fixture using `w_iters=12` (a speed override) not the real default `70`; fixed by matching it exactly. Three real porting subtleties caught: `currentSlopeField()` is raw/unscaled, distinct from milestone 1's own `build_slope_field` (which is pre-multiplied by `GW`) — added `build_raw_slope_field` rather than reusing the wrong one; `buildLandmassQuality`'s flood fill is 8-neighbour, deliberately different from milestone 2's 4-neighbour water-body fill; `buildCoastSDF` always runs via the true-Euclidean Jump Flooding Algorithm in production (`{euclid:true}`), not the simpler chamfer fallback, so `jfaDist` was ported for real. Added a third, larger fixture (48×40) specifically because both established small fixtures genuinely produce zero nonzero corridor cells from the real reference engine (confirmed real — sparse-by-design, not a bug — but an all-zero test wouldn't have caught an inverted min/max in the flanking-barrier logic). All three cases passed first attempt, `1e-4` tolerance. **Milestone 7 finding, corrected from an initial wrong claim**: Strahler-order machinery (`strahler_from_receivers`) is already ported and `WorldState.stream_order` is already populated — whether it's a semantic match for `buildRiverNetwork`'s own independent channelization (settlement suitability's real `riverOrder` input) is the open, unverified question, not "nothing exists." **Still not reachable**: settlement suitability/seed-finding itself (pending milestone 7's river-network-order resolution); factions, territory, provinces, culture, economy, roads, the Journey Planner (all untouched, block-2 proper). `buildCartBiome` (a separate, denser 15-category editor-bridge biome-paint layer) confirmed out of scope — no consumer exists in this port. See `CHANGELOG.md`'s "Phase 2 milestone 1/2/3/4/5/6" entries and `PHASE2_SCOPE.md` for the living milestone list. |
| 3 — Rendering and 3D | Not formally started. Two things to remember when it does: **(a)** criterion 2's renderer (above) ports the reference's *default-settings* material model only — real biome colours, real hillshade — explicitly excluding every `state.viz.*`-gated stretch feature (splat texturing, geology microtexture, NPR "Painter" styles, AO/SVF/shadows, multi-sun, SDF coast/river/biome tinting). Wiring any of those in is genuine Phase 3 work. **(b)** When that work lands, re-invoke `ui-ux-pro-max` for the UI side rather than bolting raw sliders onto the newly-exposed params — keep it consistent with the 2026-08-16 light parchment theme (ported from the reference's own `:root[data-theme="light"]`), not the earlier dark-dashboard match that theme replaced. **(c)** GPU compute *via Godot's own renderer* was researched 2026-08-16 (prompted by `godot-demo-projects/compute/heightmap`) and found not applicable *through that path*: `project.godot` uses the `gl_compatibility` renderer, which doesn't support `RenderingDevice` compute dispatch at all (engine-level constraint, already documented in `.claude/skills/godot-shell/SKILL.md`). That finding does **not** apply to a *standalone* `wgpu` instance created directly by Rust code — see the GPU-compute pilot section below, which tested exactly that and found the hardware path itself viable (the renderer choice is irrelevant to a `wgpu` instance that never touches Godot's own rendering pipeline). If Phase 3 revisits Godot's own renderer for other reasons (3D terrain drape, particles), GPU-accelerated presentation-layer work *through Godot* becomes reachable as a further, separate option — not before, and not for core generation (which must stay CPU-Rust for golden-parity reproducibility regardless of renderer). |
| 4 — Asset Library | Not started. |
| 5 — Urban morphology | Not started. |

## GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`, `HARDWARE_ACCELERATION.md`)

**Done, 2026-08-16.** Piloted a standalone `wgpu` compute path (new crate
`cartalith-gpu`, no `gdext` dependency) on one kernel: `cartalith_noise::vnoise`.
Findings:

- **The `wgpu` hardware path itself works cleanly** on this session's real
  hardware (AMD Radeon RX 7800 XT, Vulkan backend, discrete GPU) —
  instance/adapter/device creation, conservative limits, shader compile,
  dispatch, readback all function correctly.
- **This specific formula is not GPU-viable in `f32`** — `hash`'s
  f64-magnitude-dependent rounding (its own doc comment already flagged
  ~2^61 intermediate products, past `f64`'s own exact range) does not
  survive a portable `f32` WGSL port: 100% of cells diverge at 128×128,
  max abs diff `0.93` on a `[0,1]` output. Measured, not assumed.
  `self_test` (the real correctness gate) correctly reports FAIL and the
  CPU fallback is correctly used instead.
- **`f64` in WGSL is a dead end on this toolchain regardless of hardware
  support** — `wgpu::Features::SHADER_F64` is present on this adapter, but
  naga (wgpu 30's WGSL compiler) has no `enable f64;` implementation at
  all. A real, precise finding, not a shrug.
- **Real GPU-vs-CPU timing measured**: GPU loses at 128×128 (dispatch
  overhead dominates, 0.20×) but wins increasingly at scale — 4.46× at
  512×512, 15.65× at 1024×1024, 19.55× at 2048×2048.
- **Verdict**: the `wgpu` path is a real, viable option for *future*
  candidate kernels that don't share `hash`'s f64-precision dependency
  (e.g. presentation-layer work — hillshade/AO synthesis, biome
  classification — pure functions of already-computed fields). Not this
  kernel, not right now, and no wider `HARDWARE_ACCELERATION.md` adoption
  decision has been made — this pilot answers one narrow question, per its
  own scope doc's explicit boundary.

See `CHANGELOG.md`'s "GPU-compute pilot" entry for the full numbers and
reasoning. Nothing outside `GPU_COMPUTE_PILOT_SCOPE.md`'s "In scope" list
was implemented (no capability-tier classifier, no diagnostics panel, no
telemetry system, no tiled compute) — all still deferred exactly as that
document scoped them.

## Known-open items (not owner-blocked, just not done yet)

- Credits screen (Phase 1 closeout, `ROADMAP.md`).
- Crate licence audit (Phase 1 closeout, `PROVENANCE.md`).
- Real Fira Sans/Fira Code font files for the UI theme (design-system match found the pairing; sourcing + OFL-license verification deferred).
- Sea level as a user-adjustable Godot control (`MVP_SCOPE.md` point 9 — real terrain scope, just not wired to a `#[func]` yet).

## Owner-only items

- Criterion 4's full sense (installed and run on the owner's *actual phone*) — softened by the 2026-08-16 `/goal`, no longer blocking.
- Nothing else currently requires the owner specifically; this session has real Windows desktop + `godot4` CLI access, which closes most of what earlier sessions couldn't do themselves.
