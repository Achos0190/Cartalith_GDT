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

## Milestone 2 — relief lighting: multidirectional hillshade + ambient occlusion (done 2026-08-17)

Milestone 1 was deliberately zero-visual-change groundwork. This one is the
opposite: the default render should look meaningfully better, judged by
looking at it.

**What was chosen, and why these two.** `TERRAIN_APPEARANCE_RESEARCH.md`
lists 15 phases; this milestone did two of them properly rather than four
badly. §14 (multidirectional hillshade) and §15 (ambient occlusion) were
picked because they share one decisive property: **they act on the lighting
term only, never on the material/colour term.** The material blend
(`material_weights` and the 25 palettes) is the part golden-verified against
the JS engine, and the part §32 warns is easiest to break for one terrain
type while improving another. Leaving it untouched made this a low-risk,
high-visibility change.

The two also complement each other, which is why doing only one would have
been worse than doing neither well:

- Multidirectional lighting reveals landforms whose ridgelines run *parallel*
  to the single NW sun — structurally invisible under one light, no matter
  how the curve is tuned.
- But adding lights lifts shadowed slopes and therefore *flattens* depth,
  the classic multidirectional failure mode. AO restores that depth from the
  terrain's own concavity rather than from light direction.

Rejected for this pass, with reasons: §10/§11 (slope/curvature modulation)
would have meant editing `material_weights`, the golden-verified part —
exactly the §32 risk above, and unnecessary since slope and curvature are
*already* inputs there. §18 (local contrast) needs a neighbourhood pass over
final colour, an architecturally different shape from this per-pixel
renderer, and the research doc's own haloing/edge-artifact warnings make it a
milestone rather than an add-on.

**Built.** `TerrainAppearance` gained `sun_alt_deg` (hoisted from two
separate hardcoded `40.0`s), `relief_lights`/`relief_directionality`/
`relief_ambient`/`relief_gain`, and `ao_strength`/`ao_radius_frac` — named to
match §14/§15's own GUI vocabulary so the deferred editing panel maps onto
them directly. `shade` computes the surface normal once and dots it against a
precomputed weighted light table (6 lights, weight `((1+cos θ)/2)^p`, primary
still dominant at 43%); `build_ao` is a two-scale cavity map built from the
existing separable box blur.

**The AO normalization is the part that makes it survive §32.** Each scale is
normalized by its own RMS over *land cells only*, so occlusion is measured
against the world's own relief statistics. A fixed magnitude threshold would
have given a low-relief world no AO at all and crushed an alpine one — the
exact "flatters one terrain, destroys another" failure §32 names. It is also
a pure function of the heightfield, so §27 determinism holds.

**Golden-parity: kept exact, not re-baselined, not loosened.** New
`TerrainAppearance::js_reference()` (`relief_lights: 1`, `ao_strength: 0.0`,
original curve constants) reproduces the pre-milestone renderer bit-for-bit —
`relief_lights <= 1` takes a dedicated early-return branch in `shade` so JS
parity can never drift on a float reassociation, and `ao_strength == 0` skips
the AO precompute entirely, leaving the `1.0` the code previously hardcoded.
`golden_parity_render.rs` now constructs its context through the new
`RenderCtx::with_appearance(..., js_reference())`: **both tests still pass at
their original `1e-4` tolerance with every expected value unchanged** — the
only edit is which appearance the context is built with.

That choice follows `DECISIONS.md` §7a read carefully rather than loosely.
§7a's carve-out is scoped to paths where JS parity is *impractical*
(GPU/`f32`/`naga`), and it says in as many words that the CPU rendering port
"stays golden-verified against the JS engine and that work is not being
discarded or devalued". A deliberate visual improvement is not an
impractical one, so the reference path stays tested. This also satisfies
research doc §1.5 ("preserve the current renderer as a fallback/reference
implementation") literally rather than in spirit.

**A/B harness** — new `tests/appearance_ab_dump.rs` (`#[ignore]`d; run with
`--ignored`) renders the same generated world through both appearances and
writes raw RGB dumps, covering Classic and Archipelago. This is research doc
§1.6's "establish deterministic A/B comparison rendering", and it exists
because UI screenshots alone can't isolate the renderer from the rest of the
app.

**Verified.** `cargo build -p cartalith-godot` clean; `cargo test
--workspace` 71 suites, 0 failures, 0 modified expectations; `cargo clippy -p
cartalith-godot --all-targets` clean for this milestone's files (the one
remaining warning is the pre-existing `needless_borrow` in `lib.rs`);
`godot4 --headless --quit main.tscn` clean.

Real before/after, both from the deterministic dump and from the real
windowed app (2048², seed 12345, Classic, 40 settlements, same parameters
both runs): drainage networks, ridge/valley structure and coastal
escarpments become legible where the single-sun render showed a flat tan
wash. Against §30's anti-list, measured rather than eyeballed:

| | Classic before | Classic after | Archipelago before | Archipelago after |
|---|---|---|---|---|
| min luma | 39.4 | **39.4** | 31.6 | **31.6** |
| mean luma | 133.3 | 128.8 | 108.7 | 108.0 |

Identical minima confirm no new darkest pixel — no black valleys (AO darkens
concavities only and is floored at `1 - ao_strength`). Mean luma barely
moves, so the change redistributes contrast rather than dimming. The
archipelago case is the important one for §32: the low-relief world gains
definition without being crushed or going monochromatic.

**One real regression caught mid-pass by looking, not by reading.** A 3× zoom
of the dump showed speckle on flat plains: the fine AO radius resolved to 1
cell at 512², close enough to the raw field that the cavity signal picked up
per-cell heightfield noise — "random texture noise" on §30's own anti-list.
Floored both radii (`r_fine = (r_broad/3).max(2)`) and re-verified.

**Cost: essentially free.** 512² render time 45→45 ms (Classic) and 20→19 ms
(Archipelago). The normal is computed once and reused across all six lights,
so multi-light adds only dot products; AO is a one-time O(n) separable blur
plus a per-pixel array lookup.

**Still open for later milestones**: the atlas look proper (paper/vellum
ground, forest stippling, hand-lettered glyphs, physical border — see
`VISION.md`), §10/§11/§18 as reasoned above, the GUI editing panel (deferred
by `GUI_SHELL_SCOPE.md`), the GPU path (§21), and milestone 1's own open
question about whether an elevation-breakpoint ramp should exist as a
separate mode alongside the material system.

## Milestone 3 — hydrology-based colour tint (done 2026-08-17)

**What was chosen, and why.** §13 (hydrology-based colour modulation) —
picked over §12 (geological exposure, which would need a new lithology field
threaded from `WorldState`/`lib.rs`, real plumbing beyond render.rs's own
scope for one milestone) and §18 (local contrast, which needs a two-pass
neighbourhood architecture over the final colour buffer — a genuinely
different shape from this per-pixel renderer, same reason milestone 2
deferred it). §13 fit the established milestone-2 pattern exactly: a new
term added to the *final tonal stage* (alongside AO and vignette), never
touching `material_weights` — the golden-verified part `TERRAIN_APPEARANCE_
RESEARCH.md` §32 warns is easiest to break for one terrain type while
helping another.

**Built.** `TerrainAppearance` gained `hydro_wet_strength`/
`hydro_wet_radius_frac`. `build_hydro_wetness` log-compresses the existing
`flow` field the same way `cell_color`'s own TWI term already does, min-max
normalizes it (so it holds up across worlds with very different total flow —
the same reasoning `build_ao`'s RMS normalization already established),
keeps only the top of that range via `smoothstep` (ordinary hillside
sheet-flow shouldn't tint the whole map), and blurs it into a soft halo. The
result blends `land_color`'s final tone toward a cool, muted green-grey —
deliberately short of `wetland_temp`'s own darkest stop, so it reads as
dampness near a channel, not a second, competing material classification
(§13's own "do not paint rivers into the terrain colour raster" — the actual
river vector overlay stays a separate system, unchanged).

**Golden-parity: same pattern as milestone 2, not re-litigated.**
`hydro_wet_strength: 0.0` in `js_reference()` skips the precompute and
leaves the term a no-op — both `golden_parity_render.rs` tests pass at their
original `1e-4` tolerance, expected values unchanged.

**A real tuning pass, not a first-guess-worked story.** The first parameter
set (`strength 0.20`, `smoothstep 0.72–0.97`, `radius_frac 0.004`) passed
every mechanical check — builds, tests, correctly shaped like real river
networks in an amplified diff — but a side-by-side crop at *actual* strength
showed no perceptible difference at all: only 0.4% of pixels changed, by a
mean of 2.5/765 possible. Caught by looking, not by trusting the diff
statistics. Strengthened to `0.38` / `0.55–0.88` / `0.006` and re-verified
the same way: 2.19% of pixels change in the Classic world, and cropping
exactly at the maximum-diff pixel (not a guessed "looks riverlike" region)
shows a real, if deliberately subtle, cooling along the actual valley floor.

**Honest cross-world result, same shape as milestone 2's own AO finding on
Archipelago.** Classic (real relief, real rivers): clearly visible at the
point of maximum effect, 2.19% of pixels touched. Archipelago (low-relief,
fragmented, less continuous drainage): only 0.75% of pixels touched, and the
effect is essentially imperceptible even at its own strongest pixel — not a
bug, just less major flow accumulation for the effect to find on a world
shaped like that. Both anti-list checks held: luma minimum identical
before/after in both worlds (no darkening beyond the deliberate floor, no
black valleys), no visible banding or haloing in either crop.

**Verified.** `cargo build -p cartalith-godot` clean; `cargo test
--workspace` 0 regressions (full suite, all crates); `cargo clippy -p
cartalith-godot --all-targets` clean for this milestone's files (three
pre-existing warnings elsewhere — two in `cartalith-civ` from concurrent
work, one pre-existing `needless_borrow` in `lib.rs` — confirmed unrelated
by file/line); `godot4 --headless --quit main.tscn` clean. Real windowed-app
run (seed 12345, Classic, 2048², 40 settlements) generated and rendered
correctly end-to-end with no crash or visual corruption. The primary
before/after comparison used the deterministic A/B dump harness
(`appearance_ab_dump.rs`, extended with an isolation pair — milestone 2's
own relief/AO held fixed, only `hydro_wet_strength` toggled — so this
milestone's delta is measured independently of milestone 2's already-
verified one) rather than repeated real-app screenshots, since this
session's own milestone-2 report already found windowed UI automation
unreliable; one real-app run confirmed end-to-end correctness, not a
multi-shot visual comparison.

**Still open**: §12 (geological exposure — needs new `WorldState` plumbing),
§18 (local contrast — needs a two-pass architecture), the atlas look proper,
the GUI editing panel, the GPU path, milestone 1's elevation-ramp question.

<!-- A duplicate, shorter "Milestone 3" section briefly existed here,
committed by a concurrent fork that picked up this milestone's
in-progress render.rs changes from the shared working tree at an
earlier point mid-tuning — its own numbers (`smoothstep(0.72, 0.97, …)`)
described the *first-guess* parameters, not the final tuned ones the
actually-committed code carries (`0.55, 0.88`). Removed rather than left
to drift from the real code; the section above is the accurate record. -->
