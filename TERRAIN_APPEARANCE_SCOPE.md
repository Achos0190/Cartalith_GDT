# Terrain appearance: milestone plan (Phase 3 begins)

Turns `TERRAIN_APPEARANCE_RESEARCH.md` (owner-supplied, 2026-08-17) into a
real, incremental milestone plan, the same way `GPU_LAYER_INTEGRATION_SCOPE.md`
and `CPU_MULTITHREADING_SCOPE.md` turned their own research/investigation into
staged work. That research doc itself proposes 15 phases (§31); this doc
breaks the *first* few into something one milestone can actually land, per
this project's own "one subsystem at a time" discipline
(`cartalith-porting-discipline` skill).

## Why now

`ROADMAP.md`'s Phase 3 entry: *"Also the natural moment to revisit 2D fidelity
beyond MVP's 'correct and plain': multi-octave grain, hillshade quality, NPR
styles. And the moment to install a UI/UX skill, once the interface outgrows
four controls."* Phase 2's own remaining large piece (Journey Planner, ~70
reference functions, `ECONOMY_SCOPE.md`) is correctly scoped as its own future
sub-phase, not blocking this. Terrain appearance doesn't need Phase 3's 3D
half (`DECISIONS.md` §4, still deferred) — it's a pure extension of the
existing 2D `render.rs` per-pixel pipeline, already real and golden-verified
(`MVP_SCOPE.md` point 10, `STATUS.md` criterion 2).

## Milestone 1 — audit + `TerrainAppearance` abstraction + one real editable ramp (this pass)

Per the research doc's own §1 (audit first), §2 (`TerrainAppearance`
abstraction), and §3 (editable elevation→colour ramp) — CPU-only, no GUI yet,
no GPU yet. Everything else (§4 GUI, §5-6 domain modes/auto-fit, §7 presets,
§8-13 material/climate/slope/curvature/geology/hydrology modulation, §14-19
lighting/AO/detail/colour/contrast/atmosphere, §20-23 display pipeline/GPU/
CPU-fallback/perf, §24-30 GUI layout/preview/serialization/determinism/debug/
quality tiers) stays out of this milestone — real future work, not attempted
here.

1. **Audit** (research §1): read `crates/cartalith-godot/src/render.rs` in
   full. Document, in this file, exactly where elevation/climate/biome/slope/
   hillshade/hydrology currently become colour, and whether that happens
   per-pixel on CPU (expected — no GPU rendering path exists yet per
   `STATUS.md`'s own Phase-3 note on `gl_compatibility` not supporting compute
   dispatch through Godot's own renderer) baked into one raster at generate()
   time.

2. **`TerrainAppearance` abstraction** (research §2): extract the
   elevation→colour logic currently inline in `render.rs` into a structure
   that takes the same physical fields it already reads (elevation, slope,
   biome, hillshade inputs — whatever `render.rs` already consumes, don't add
   new physical fields this pass) and produces colour, **without changing
   the produced pixels** — this is a refactor, not a visual change. Verify via
   this crate's own existing golden-parity render tests
   (`golden_parity_render.rs`) passing byte-identical, unmodified.

3. **One real editable colour ramp** (research §3, narrowed): replace
   whatever currently maps elevation to a colour band (check `render.rs` for
   the real current mechanism — likely a small number of hardcoded threshold/
   colour pairs) with a proper ordered breakpoint list (elevation → colour,
   smooth-interpolated between neighbours) that produces the **same default
   visual output** as today (i.e., encode the current hardcoded bands as the
   default ramp, don't invent new colours) but is now a real data structure
   that could be edited later — not yet exposed to any UI or `#[func]`, this
   milestone is data-structure-only, matching `cartalith-spatial`'s own
   "standalone, unintegrated-with-UI, but real" precedent from earlier this
   session.

## Explicitly out of scope for milestone 1

- Any GUI (research §4/§24-25) — no gradient editor, no controls, nothing
  Godot-scene-visible changes this pass.
- Elevation-domain modes / Auto Fit (§5-6).
- Presets (§7).
- Biome/material/slope/curvature/geological/hydrology modulation beyond
  whatever `render.rs` already does today (§8-13) — this milestone ports the
  *existing* logic into the new abstraction, it does not add new modulation.
- Lighting/AO/multi-scale detail/colour vibrancy/local contrast/atmosphere
  (§14-19) — all real future milestones.
- GPU path (§21) — CPU-only this pass, matching where `render.rs` already is.
- Serialization/presets-as-data, determinism testing beyond the existing
  golden tests, debug visualization modes, quality tiers (§26-29).

## Verification

- `cargo build -p cartalith-godot`, `cargo test -p cartalith-godot` — the
  existing `golden_parity_render.rs` tests must pass **byte-identical,
  unmodified** (this is a pure refactor + data-structure introduction, the
  rendered output must not change at all this pass).
- `cargo clippy -p cartalith-godot --all-targets` clean.
- `godot4 --headless --quit main.tscn` clean load.
- Real windowed-app screenshot: generate the same seed before/after (or just
  after, compared by eye against this session's own prior screenshots of the
  same seed/settings) confirming the map looks identical to before this
  milestone — since the whole point of milestone 1 is "no visible change yet,
  just a sound structure to build on."

## Done means

`render.rs`'s elevation-to-colour logic lives behind a `TerrainAppearance`-
shaped abstraction with a real, data-driven (not hardcoded-inline) colour
ramp, producing pixel-identical output to before this milestone. Milestone 2
(whenever picked up) can then add the GUI/editing/domain-modes/auto-fit
layer on top of a real foundation, the same "base before integration"
sequencing this session already used for `cartalith-spatial`.

## Milestone 1 — done (2026-08-17)

**Audit finding, corrected from this doc's own initial assumption**: there
is no elevation-keyed colour *breakpoint ramp* anywhere in `render.rs`,
despite `TERRAIN_APPEARANCE_RESEARCH.md`'s MapTiler-style mental model.
Colour comes from `material_weights()` — a continuous multi-input blend
over temperature/moisture/slope/relative-elevation/aspect/curvature
producing six material fractions (snow/rock/sand/wetland/canopy/grass),
each contributing colour via a **noise-jittered** 3-stop micro-ramp
(`ramp3`, selected by `tt`, a per-pixel texture-variety value from coherent
noise — not from elevation). Relative elevation (`r`) is one continuous
`smoothstep` input among several, not a lookup axis. So step 3's original
plan ("re-encode the current hardcoded elevation bands as a ramp") doesn't
map onto reality — there are no elevation bands to re-encode. What's real
and now editable: the 25 material/water 3-stop palettes plus the
`exag`/`sun_az_deg`/`bio_blend` shading constants, previously 26 free
module-level consts, now one owned `TerrainAppearance` struct
(`Default` reproduces every value exactly).

**Built**: `TerrainAppearance` struct in `render.rs` (25 named `[Rgb;3]`
palette fields + 3 shading constants), threaded through `grass_col`/
`forest_col`/`sand_col`/`rock_col`/`snow_col`/`wetland_col`/`sea_color_core`/
`land_color`/`sea_shade_from`/`RenderCtx::shade` — all previously reading
bare module consts, now reading `&TerrainAppearance`. `RenderCtx` owns one
`TerrainAppearance` (constructed via `Default` in `RenderCtx::new`) so
`cell_color(&ctx, x, y)`'s public signature — and `RenderCtx::new`'s own —
stayed **completely unchanged**, meaning `golden_parity_render.rs` needed
zero modification. Not yet wired to any UI/`#[func]` — genuinely
standalone-but-real, matching `cartalith-spatial`'s own precedent.

**Verified**: `cargo build -p cartalith-godot` clean. `golden_parity_render.rs`
— both tests (`cell_color_matches_js_surface_and_sea`,
`cell_color_matches_js_world_wrap`) pass byte-identical, file completely
unmodified. `cargo clippy -p cartalith-godot --all-targets` clean (no new
warnings; the one pre-existing `needless_borrow` at `lib.rs:280` is
unrelated). `cargo test --workspace` 0 regressions. `godot4 --headless
--quit main.tscn` clean load. Real windowed-app screenshot (seed 12345,
Classic, 2048², 40 settlements) confirms the map renders correctly —
biome colours, hillshade, settlements, roads, sea routes all visible,
matching this session's prior screenshots of the same settings.

**Next real question for milestone 2** (not attempted here): whether a
literal MapTiler-style elevation-breakpoint ramp should be added as a new,
separate visual layer/mode alongside this material-weighted system, or
whether "editable ramp" for Cartalith should instead mean exposing
`TerrainAppearance`'s own real palettes (the 25 material colours) to a
GUI — a genuine design decision, not a re-encoding exercise, now that the
audit has corrected the original assumption.
