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

## Milestone 4 — the atlas look: paper ground, forest stippling, plate border (done 2026-08-17)

`VISION.md`'s sequencing item 2 named four things still ahead after
milestone 3: *"the paper/vellum ground, forest stippling, hand-lettered
glyphs and the physical border."* Three of the four are in `render.rs`'s
raster and landed here. The fourth — hand-lettered settlement glyphs — is
drawn by `godot-project/map_overlay.gd`, not by this raster, and was
deliberately left alone (a concurrent fork owns GDScript this session).

**What was chosen, and why all three rather than two.** Milestones 2 and 3
both used the standard "two done properly rather than four badly". Three
landed here because they are genuinely independent stages, each gated on its
own parameter, each measurable on its own:

- **Paper/vellum ground** — the single biggest tonal shift, and the only one
  that touches the *whole* sheet, ocean included. Applied at the top of
  `cell_color` after both the land and sea branches, deliberately: an ocean
  that isn't on the same paper as the land makes the map read as terrain art
  pasted onto a parchment background.
- **Forest stippling** — the one with real data behind it.
  `material_weights` already computes a `canopy` fraction, so this is
  texture over actual canopy rather than decorative noise laid wherever the
  image happens to look green.
- **Physical plate border** — the cheapest of the three and the only one
  that cannot damage terrain legibility, since it composites over the
  finished colour and reads no world data at all.

None of them touch `material_weights` or the 25 palettes — the same rule
milestones 2 and 3 both held to, and the reason §32's "flatters one terrain,
destroys another" risk stays bounded.

**Golden-parity: the same gating mechanism, extended, not replaced.**
`js_reference()` gains `paper_strength: 0.0`, `stipple_strength: 0.0`,
`border_width_frac: 0.0`, and each of the three stages **early-returns on
its own zero** rather than merely evaluating to an arithmetic no-op —
`paper_tone` returns before touching a single `vnoise`, the stipple block is
inside an `if`, `apply_border` returns its argument. That is the identical
discipline `relief_lights <= 1` established in milestone 2 (a dedicated
branch, so parity can never drift on a float reassociation), and it means
`golden_parity_render.rs` is **still completely unmodified** and both tests
still pass at their original `1e-4` tolerance with every expected value
unchanged. Three milestones in, that file has never been edited except for
which appearance the context is built with.

**Two real corrections caught by looking, not by the numbers.** Milestone 3's
lesson held again, twice:

1. *The paper was originally a pure hue rotation and it was too weak.* The
   parchment tint is divided by its own Rec.709 luma before use, so it warms
   without dimming — necessary (a straight multiply by an off-white costs
   ~10% luma everywhere and would flatten exactly the relief legibility
   milestone 2 bought), but on its own it only rotated hue and left a
   digital-looking saturated ocean. What actually shifted the tonal feel was
   adding `paper_wash`: a pull toward a paper-coloured grey **of the same
   luminance**, so chroma drops and nothing else does. Pigment soaked into a
   sheet is never as chromatic as an emitted colour; that is the whole
   difference between a screen render and a printed plate.
2. *The first stipple read as a regular halftone screen.* Value noise
   sampled on the axis-aligned grid at a few cells per mark produces a
   visible diagonal checker — precisely §30's "random texture noise"
   failure, and the same class of regression as milestone 2's AO speckle,
   found the same way (a 6× crop of the real dump, not a diff statistic).
   Fixed by rotating the sampling lattice ~34°, domain-warping it with a
   second coherent field, and flooring the mark size at 4 cells. The marks
   now clump the way drawn stippling does. Deterministic throughout (§27):
   every stage is a pure function of the cell coordinates.

**Measured against §30's anti-list, terrain only.** All figures at the app's
own 2048², seed 12345, with the 40-cell frame band excluded so the border
doesn't skew the terrain statistics; "base" is milestone 3's look, "atlas"
is this milestone's:

| | Classic base | Classic atlas | Archipelago base | Archipelago atlas |
|---|---|---|---|---|
| interior luma min | 42.4 | 41.0 | 34.6 | 33.8 |
| interior luma mean | 132.8 | **133.0** | 106.3 | **106.2** |
| interior luma sd | 31.32 | **31.89** | 27.66 | **28.30** |
| interior mean chroma | 59.7 | 51.96 | 70.3 | 51.96 |
| any-channel clipping | 0.70% | 0.73% | 0.03% | 0.03% |

Mean luma is unchanged to a fraction of a level in both worlds (the tint and
the wash are luminance-preserving by construction), and contrast **rises**
slightly rather than falling — so nothing was washed out or flattened. The
luma minimum drops 1.4 and 0.8 levels, entirely from the paper grain: no new
black valleys. Clipping is unchanged in the terrain (whole-image clipping
rises to 0.87%/0.17% only because the cream frame margin is itself bright).

**Cross-world honesty — and this time it runs the opposite way to
milestones 2 and 3.** Both of those were strong on mountainous Classic and
nearly invisible on low-relief Archipelago, because they keyed off relief
and drainage. This milestone is the reverse: the paper acts on the entire
sheet, and Archipelago is mostly ocean, so it loses **26%** of its chroma
against Classic's 13%. Its bright cyan sea becomes a muted teal-grey, which
is the largest single visual change either test world has seen in this whole
phase. A detail worth recording: the two worlds start 18% apart in mean
chroma (59.7 vs 70.3) and land within 0.01 of each other (51.960 vs 51.963)
— not by clamping, since the reduction ratios differ (0.871 vs 0.739), but
because a common printing medium is exactly what converges two differently
coloured subjects. Forest stippling is the mirror image: 13.9% of Classic's
pixels touched versus 10.8% of Archipelago's, and it is only really legible
where there is continuous canopy to texture.

**Cost, honestly: this one is not free.** 2048² render time 598 → 915 ms
(Classic) and 295 → 597 ms (Archipelago) — the paper is four extra `vnoise`
calls on every pixel of the map, including the ocean. That is a one-shot
cost at generate time against a pipeline that already takes far longer, so
it was accepted rather than optimized, but it is a real regression from
milestone 2's "essentially free" and the obvious first candidate if the
render ever needs to be fast (the two sheet-scale mottle octaves could be
precomputed at a coarse resolution and bilinearly sampled).

**One real known limitation, found in the real app and not fixed here —
~~open~~ RESOLVED in a follow-up pass the same day (see below).** Two
systems draw *over* the finished raster and know nothing about the frame:
`lib.rs`'s river channel-mask tint, and `map_overlay.gd`'s settlement/road
markers. In both test worlds a settlement sitting at the extreme west edge
puts its marker partly on the plate margin. It is a small, real defect; the
fix (skip the overlay inside the border band) belongs in those two files,
both of which are outside this milestone's `render.rs`-only scope and one of
which a concurrent fork owns this session. Flagged rather than reached for.

### Milestone 4 follow-up — the overlays learn about the frame (done 2026-08-17)

The limitation above, fixed. It turned out to be **four** systems, not two:
the river tint and the GDScript markers as flagged, plus
`build_territory_texture`'s per-faction wash and
`build_province_boundary_texture`'s line, both found while fixing the first
two. Territory is the worst of the four — a solid semi-transparent fill over
every owned cell, so any faction whose land reaches the sheet edge coloured
the bare margin outright, not just at one settlement.

**Inset versus clip, and why insetting is the wrong shape for this frame.**
The tempting fix is to give the overlays the plate *interior* as their
coordinate space, so nothing can land on the margin by construction. That is
how a real atlas plate is laid out — and it is wrong here, because
`apply_border` does not lay the plate out that way. It **composites over the
finished raster's outermost cells**: the terrain under the margin is
*covered*, not moved. Remapping markers into the interior would therefore
shift every one of them away from the coastline, river and road it sits on,
which is a far worse defect than the one being fixed. Making insetting
correct would mean resampling the *world* into the interior inside
`render.rs` — a different, much larger change that would move every
measurement in the table above, and that also has to answer what happens to
the world's aspect ratio when the frame is a fixed cell count on all four
sides. Not a defect fix.

So everything is handled at the neatline instead, and the two overlay kinds
are handled *differently* because they are different objects:

- **Linear features are clipped** (roads, sea lanes, province boundaries). A
  road that reaches the sheet edge genuinely continues past it; cutting it
  at the neatline is exactly what a plate does.
- **Point symbols are placed or omitted, never sliced.** A settlement whose
  cell is under the frame has no visible terrain beneath it at all — its
  marker points at nothing — so it is off-plate and is not drawn. One whose
  centre is inside keeps its exact position and lets the clip trim any
  overhang, which is the actual reported defect (markers landing *partly* on
  the margin).
- **Raster tints fade rather than cut.** The river tint and the territory
  wash are multiplied by `1 - border_cover`, the frame's own soft edge, so
  they stop exactly where the paper wash starts instead of a cell and a half
  early.

**Where the geometry lives.** `render.rs` keeps it, and now exports it:
`border_width_cells` (width in cells, `0.0` when disabled) and
`border_cover` (frame coverage at a cell, `0.0` throughout the interior,
`smoothstep`-ramped to `1.0` under the margin). `apply_border` was rewritten
onto both rather than keeping a second copy of `0.014 * gw`.
`WorldGen::get_border_inset_frac()` carries it across the gdext boundary as
a **fraction of texture width** — a fraction, not a cell count, because
`map_overlay.gd` works in screen pixels against a letterboxed texture and a
fraction survives `_displayed_rect()`'s fit maths without the GDScript side
knowing the resolution. `map_overlay.gd` derives `_interior_rect()` from it
and scissors its canvas item to that rect
(`RenderingServer.canvas_item_set_clip` + `canvas_item_set_custom_rect`) —
one scissor covering circles, arcs, polylines and dashed lines rather than
four hand-written clippers. `Control` re-sets both from its own rect on
every `NOTIFICATION_DRAW`, which fires immediately before `_draw()`, so the
override lasts one frame and needs no restore.

**Same zero-gate discipline, so parity is untouched.** `border_cover` is
`0.0` everywhere when `border_width_frac == 0.0`; every raster call site is
written `tinted + (plain - tinted) * cover` so `cover == 0.0` restores the
old value *bit-exactly* rather than to within an ulp; `_border_frac == 0.0`
makes `_interior_rect()` return `_displayed_rect()` unchanged and skips the
scissor. `golden_parity_render.rs` remains completely unmodified and both
tests still pass at `1e-4` — four milestones in, that file has still never
been edited.

**Measured on the real app, at the specific failing case.** The same
2048²/seed 12345/Classic world generated twice — once with the fix stashed,
once applied — screenshotted and cropped 4× at the west edge, counting
overlay ink inside the frame band:

| | marker orange on margin | river-tint cyan on margin |
|---|---|---|
| before | 268 px | 67 px |
| after | **0 px** | **0 px** |

Differences between the two runs sit entirely in the frame band and within
4 px inside the neatline; nothing in the plate interior moved. Archipelago
(35 settlements, sea routes on) is 0/0 as well, and shows both rules at once
— a coastal capital whose centre is just inside is trimmed cleanly at the
neatline while its sea lanes are cut there.

**Verified.** `cargo build -p cartalith-godot` clean; `cargo test
--workspace` 383 passed / 0 failed, no expected value anywhere modified;
`cargo clippy -p cartalith-godot --all-targets` clean for this milestone's
files (the sole remaining warning in the crate is the pre-existing
`needless_borrow` in `lib.rs`; the `cartalith-gpu`/`cartalith-civ` warnings
are concurrent forks' and were confirmed unrelated by file and line — this
pass also cleaned up four `field_reassign_with_default` warnings the A/B
harness had accumulated); `godot4 --headless --quit main.tscn` clean load.
Real windowed app (2048², seed 12345, 40 settlements) generated and
screenshotted for **both** Classic and Archipelago, and the plate frame,
parchment ground and canopy texture all read correctly at the app's own
display scale with the settlement/road overlay on top. The controlled
before/after is the deterministic dump harness at the same 2048² the app
uses — `appearance_ab_dump.rs` now emits a `noatlas`/`withatlas` isolation
pair (milestones 2 and 3 held fixed) plus `paperonly`/`stippleonly` dumps,
since the three stages are independent and a combined image cannot show
which one is carrying a change.

**Still open**: hand-lettered settlement glyphs (`map_overlay.gd`, not this
raster), §12 (geological exposure — needs new `WorldState` plumbing), §18
(local contrast — needs a two-pass architecture), the GUI editing panel
(`GUI_SHELL_SCOPE.md`), the GPU rendering path (§21), milestone 1's
elevation-ramp question, and the overlay-over-frame defect above.

<!-- A duplicate, shorter "Milestone 3" section briefly existed here,
committed by a concurrent fork that picked up this milestone's
in-progress render.rs changes from the shared working tree at an
earlier point mid-tuning — its own numbers (`smoothstep(0.72, 0.97, …)`)
described the *first-guess* parameters, not the final tuned ones the
actually-committed code carries (`0.55, 0.88`). Removed rather than left
to drift from the real code; the section above is the accurate record. -->
