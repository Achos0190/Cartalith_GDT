# Terrain appearance — Phase 3's 2D milestones

Turns `TERRAIN_APPEARANCE_RESEARCH.md` (owner-supplied, 2026-08-17) into an
incremental milestone plan, the way `GPU_LAYER_INTEGRATION_SCOPE.md` and
`CPU_MULTITHREADING_SCOPE.md` staged their own research. The research proposes
15 phases (§31); this document breaks the first few into milestones one pass can
land, per the "one subsystem at a time" discipline (`cartalith-porting-discipline`
skill).

**This document defines Phase 3's 2D appearance milestones and records what each
pass found, measured and decided. It does not track them.** Where any of it
stands — done, partial, blocked, declined — is `cartalith-native/docs/STATUS.md`'s
alone. Deep-zoom LOD tiles, which reuse this renderer at tile resolution, are
`LOD_DETAIL_SCOPE.md`'s.

## Why now

`ROADMAP.md`'s Phase 3 entry: *"Also the natural moment to revisit 2D fidelity
beyond MVP's 'correct and plain': multi-octave grain, hillshade quality, NPR
styles. And the moment to install a UI/UX skill, once the interface outgrows
four controls."* Terrain appearance needs none of Phase 3's 3D half
(`DECISIONS.md` §4): it extends the existing 2D `render.rs` per-pixel pipeline,
which was already real and golden-verified (`MVP_SCOPE.md` point 10; `STATUS.md`
row MVP-2). When this was written, Phase 2's remaining large piece (the Journey
Planner) was scoped as its own sub-phase and did not block it.

## Rules every milestone here holds

Stated once here; each milestone below names only what it added.

- **The zero gate — parity by control flow, never by arithmetic.**
  `TerrainAppearance::js_reference()` reproduces the reference renderer
  bit-for-bit, and every stage this document adds **early-returns on its own
  zero** there (a dedicated branch, so parity cannot drift on a float
  reassociation), while `default()` carries the shipped look. Established in
  milestone 2 by `relief_lights <= 1`; extended in 3, 4, 5 and 6.
  `golden_parity_render.rs` was edited exactly once in six milestones — to choose
  which appearance its context is built with — and both of its tests pass at
  their original `1e-4` tolerance with every expected value unchanged.
- **`material_weights` and its palettes were left alone.** The material blend is
  the part golden-verified against the JS engine, and research §32 warns it is
  the easiest place to improve one terrain type while breaking another. Every
  milestone here acted on lighting, a final tonal stage, or a builder input
  instead. **Ruling AP (`LARGE_ITEM_RULINGS.md`, 2026-09-23) authorises the first
  deliberate edit:** giving the snow term an aspect (slope-direction) input — a
  re-baseline of the main map, scheduled, and traced in `LOD_DETAIL_SCOPE.md`'s
  LOD-D4.
- **`js_reference()` is a test path, not a look.** It is what keeps
  `golden_parity_render.rs` honest; no shell control selects it. The shipped
  standard is `default()`. Under `DECISIONS.md` §7p a deliberate rendering
  improvement is the standard on its own terms — the reference's look is not an
  alternative to restore — while the CPU golden verification itself stands.
- **Found by looking, not by the statistics.** Four milestones in a row shipped a
  first version that passed every mechanical check and failed by eye: AO speckle
  (2), an invisible hydrology tint (3), a halftone stipple (4), quilting from
  local contrast (5). Crops are taken at the **maximum-difference window** (a
  256² integral-image search), not a guessed "interesting" spot.
- **Measured against §30's anti-list, across worlds.** Every figure is at the
  app's own 2048² (Wide at 2048×1024 from milestone 5), seed 12345, Classic and
  Archipelago, with the frame band excluded once milestone 4 added one. A result
  that holds on one world only is reported as such.

## Milestone 1 — audit + `TerrainAppearance` abstraction + one real editable ramp

**Defined as** research §1 (audit), §2 (`TerrainAppearance`) and §3 (editable
elevation→colour ramp): CPU-only, no GUI, no GPU, **no change to the produced
pixels**. Everything else (§4 GUI, §5–6 domain modes and auto-fit, §7 presets,
§8–13 modulation, §14–19 lighting/AO/detail/colour/contrast/atmosphere, §20–23
display pipeline/GPU/CPU fallback/performance, §24–30 layout/preview/
serialisation/determinism/debug/quality tiers) was out of scope.

1. **Audit** `render.rs`: where elevation, climate, biome, slope, hillshade and
   hydrology become colour, and confirm it happens per pixel on the CPU, baked
   into one raster at `generate()` (no GPU rendering path existed —
   `gl_compatibility` cannot dispatch compute through Godot's renderer).
2. **Extract** the colour logic into a structure that takes the fields
   `render.rs` already reads and produces colour — a refactor, verified by
   `golden_parity_render.rs` passing byte-identical and unmodified.
3. **One editable ramp**: re-encode the current elevation→colour mapping as an
   ordered breakpoint list producing the same default output, data-structure
   only, following `cartalith-spatial`'s "standalone but real" precedent.

**Done meant** `render.rs`'s colour logic behind a `TerrainAppearance`
abstraction with data-driven palettes, pixel-identical to before. Verification:
`cargo build`/`test`/`clippy -p cartalith-godot`, `godot4 --headless --quit
main.tscn`, and a windowed before/after screenshot.

**What the audit found (2026-08-17) — step 3's premise was wrong.** There is no
elevation-keyed breakpoint ramp in `render.rs`, whatever the research's
MapTiler-style model assumed. Colour comes from `material_weights()`, a
continuous blend over temperature/moisture/slope/relative-elevation/aspect/
curvature producing six material fractions (snow/rock/sand/wetland/canopy/
grass), each coloured through a **noise-jittered** 3-stop micro-ramp (`ramp3`,
selected by `tt`, a per-pixel texture value from coherent noise — not
elevation). Relative elevation is one `smoothstep` input among several, not a
lookup axis, so there were no bands to re-encode. What became editable instead:
the 25 material/water 3-stop palettes and the `exag`/`sun_az_deg`/`bio_blend`
shading constants — 26 free module-level consts, now one owned
`TerrainAppearance` (`Default` reproduces every value exactly).

**Built.** `TerrainAppearance` (25 `[Rgb;3]` palettes + 3 shading constants),
threaded through `grass_col`/`forest_col`/`sand_col`/`rock_col`/`snow_col`/
`wetland_col`/`sea_color_core`/`land_color`/`sea_shade_from`/`RenderCtx::shade`.
`RenderCtx` owns one, built via `Default` in `RenderCtx::new`, so the public
signatures of `cell_color(&ctx, x, y)` and `RenderCtx::new` did not change and
`golden_parity_render.rs` needed no edit. It shipped ahead of any UI or `#[func]`
caller.

**Verified.** Both golden tests (`cell_color_matches_js_surface_and_sea`,
`cell_color_matches_js_world_wrap`) byte-identical with the file unmodified;
`cargo test --workspace` 0 regressions; clippy clean apart from one pre-existing
`needless_borrow` in `lib.rs`; clean headless load; a windowed screenshot (seed
12345, Classic, 2048², 40 settlements) matching prior screenshots.

**The question this left** — whether "editable ramp" should mean a literal
MapTiler-style elevation ramp as a separate mode, or exposing the 25 material
palettes to a GUI — was a design decision, not a re-encoding exercise. It was
later answered both ways (see *What the milestones deferred*).

## Milestone 2 — relief lighting: multidirectional hillshade + ambient occlusion (2026-08-17)

Milestone 1 was deliberately zero-visual-change. This one is the opposite: the
default render should look meaningfully better, judged by looking at it.

**Why these two.** §14 (multidirectional hillshade) and §15 (ambient occlusion)
act on the **lighting term only**, never on the material term — low risk, high
visibility. And they need each other: extra lights reveal landforms whose
ridgelines run parallel to the single NW sun (structurally invisible under one
light), but lifting shadowed slopes *flattens* depth, the classic
multidirectional failure; AO restores that depth from the terrain's own
concavity rather than light direction.

Rejected, with reasons: §10/§11 (slope/curvature modulation) would have meant
editing `material_weights`, and slope and curvature are already inputs there.
§18 (local contrast) needs a neighbourhood pass over final colour — a different
architecture from this per-pixel renderer, and the research's own haloing
warnings make it a milestone, not an add-on.

**Built.** `TerrainAppearance` gained `sun_alt_deg` (hoisted from two hardcoded
`40.0`s), `relief_lights`/`relief_directionality`/`relief_ambient`/
`relief_gain` and `ao_strength`/`ao_radius_frac`, named after §14/§15's own GUI
vocabulary so an editing panel maps onto them directly. `shade` computes the
normal once and dots it against a precomputed weighted light table (6 lights,
weight `((1+cos θ)/2)^p`, the primary still dominant at 43%); `build_ao` is a
two-scale cavity map from the existing separable box blur.

**The AO normalisation is what makes it survive §32.** Each scale is normalised
by its own RMS over *land cells only*, so occlusion is measured against the
world's own relief. A fixed threshold would have given a low-relief world no AO
and crushed an alpine one. It is a pure function of the heightfield, so §27
determinism holds.

**Parity — the gate this document's rule comes from.** `js_reference()`
(`relief_lights: 1`, `ao_strength: 0.0`, the original curve constants)
reproduces the pre-milestone renderer bit-for-bit: `relief_lights <= 1` takes a
dedicated early-return in `shade`, and `ao_strength == 0` skips the AO
precompute, leaving the `1.0` previously hardcoded. `golden_parity_render.rs`
now builds its context through `RenderCtx::with_appearance(..., js_reference())`
— its only edit. This follows `DECISIONS.md` §7a read carefully: §7a's carve-out
covers paths where JS parity is *impractical* (GPU/`f32`/`naga`) and says the CPU
rendering port "stays golden-verified against the JS engine"; a deliberate
visual improvement is not an impractical one, so the reference path stays
tested. It also meets research §1.5 ("preserve the current renderer as a
fallback/reference implementation") literally.

**A/B harness.** `tests/appearance_ab_dump.rs` (`#[ignore]`d; run with
`--ignored`) renders one generated world through both appearances and writes raw
RGB dumps for Classic and Archipelago — research §1.6's deterministic A/B
comparison, needed because UI screenshots cannot isolate the renderer.

**Result.** From both the dump and the windowed app (2048², seed 12345, Classic,
40 settlements): drainage networks, ridge/valley structure and coastal
escarpments become legible where the single-sun render showed a flat tan wash.

| | Classic before | Classic after | Archipelago before | Archipelago after |
|---|---|---|---|---|
| min luma | 39.4 | **39.4** | 31.6 | **31.6** |
| mean luma | 133.3 | 128.8 | 108.7 | 108.0 |

Identical minima: no new darkest pixel, no black valleys (AO darkens concavities
only and is floored at `1 - ao_strength`). Mean luma barely moves, so contrast is
redistributed, not dimmed. Archipelago — the §32 case — gains definition without
being crushed.

**Caught by looking:** a 3× crop showed speckle on flat plains — the fine AO
radius resolved to 1 cell at 512², picking up per-cell height noise (§30's
"random texture noise"). Both radii floored (`r_fine = (r_broad/3).max(2)`).

**Cost: essentially free.** 512² render 45 → 45 ms (Classic), 20 → 19 ms
(Archipelago): the normal is shared across all six lights, and AO is a one-time
O(n) blur plus a lookup.

**Verified.** `cargo test --workspace` 71 suites, 0 failures, 0 modified
expectations; clippy clean for this milestone's files; clean headless load.

## Milestone 3 — hydrology-based colour tint (2026-08-17)

**Why §13.** Chosen over §12 (geological exposure — needed a lithology field
threaded from `WorldState`/`lib.rs`) and §18 (local contrast — still a different
architecture). §13 fits milestone 2's pattern: a new term in the *final tonal
stage* (beside AO and vignette), never touching `material_weights`.

**Built.** `hydro_wet_strength`/`hydro_wet_radius_frac`. `build_hydro_wetness`
log-compresses the `flow` field the way `cell_color`'s TWI term already does,
min-max normalises it (robust across worlds with very different total flow —
`build_ao`'s reasoning), keeps only the top of the range via `smoothstep` (so
hillside sheet-flow doesn't tint the whole map), and blurs it into a soft halo.
It pulls `land_color`'s final tone toward a cool, muted green-grey, deliberately
short of `wetland_temp`'s darkest stop, so it reads as dampness near a channel
rather than a competing material (§13: "do not paint rivers into the terrain
colour raster" — the river overlay stays a separate system).

**Parity:** `hydro_wet_strength: 0.0` in `js_reference()` skips the precompute.

**A real tuning pass.** The first set (`strength 0.20`, `smoothstep 0.72–0.97`,
`radius_frac 0.004`) passed every mechanical check and was shaped like real river
networks in an amplified diff — but at actual strength only 0.4% of pixels
changed, by a mean of 2.5/765: imperceptible. Strengthened to `0.38` /
`0.55–0.88` / `0.006`: 2.19% of Classic's pixels change, and a crop at the
maximum-diff pixel shows a real, deliberately subtle cooling along the valley
floor.

**Across worlds**, the same shape as milestone 2's AO: Classic 2.19% of pixels,
visible at the point of maximum effect; Archipelago 0.75% and essentially
imperceptible — less major flow for the effect to find, not a bug. Luma minimum
identical before/after in both, no banding or haloing.

**Verified.** `cargo test --workspace` 0 regressions; clippy clean for this
milestone's files (pre-existing warnings elsewhere confirmed unrelated by file
and line); clean headless load; one windowed run end to end. The controlled
comparison is the dump harness, extended with an isolation pair (milestone 2's
relief/AO held fixed, only `hydro_wet_strength` toggled), because windowed UI
automation had proved unreliable for multi-shot comparison.

## Milestone 4 — the atlas look: paper ground, forest stippling, plate border (2026-08-17)

`VISION.md`'s sequencing item 2 named four things still ahead: *"the paper/vellum
ground, forest stippling, hand-lettered glyphs and the physical border."* Three
are in `render.rs`'s raster and landed here. Hand-lettered settlement glyphs are
drawn by `godot-project/map_overlay.gd`, not this raster, and were left out.

**Why three at once.** They are independent stages, each gated on its own
parameter and measurable alone:

- **Paper/vellum ground** — the largest tonal shift and the only one touching
  the whole sheet, ocean included. Applied at the top of `cell_color` after both
  land and sea branches: an ocean not on the same paper as the land reads as
  terrain art pasted onto parchment.
- **Forest stippling** — texture over `material_weights`' actual `canopy`
  fraction, not decorative noise wherever the image looks green.
- **Physical plate border** — composites over finished colour and reads no world
  data, so it cannot damage terrain legibility.

**Parity:** `js_reference()` gains `paper_strength`, `stipple_strength` and
`border_width_frac` at `0.0`; `paper_tone` returns before touching a `vnoise`,
the stipple block is inside an `if`, `apply_border` returns its argument.

**Two corrections caught by looking.**

1. *The paper was first a pure hue rotation, and too weak.* The parchment tint is
   divided by its own Rec.709 luma so it warms without dimming (a straight
   multiply by an off-white costs ~10% luma everywhere and would flatten the
   relief milestone 2 bought) — but alone it only rotated hue, leaving a
   digital-looking saturated ocean. What shifted the tonal feel was
   `paper_wash`: a pull toward a paper-coloured grey **of the same luminance**,
   so chroma drops and nothing else does. Pigment in a sheet is never as
   chromatic as an emitted colour; that is the difference between a screen
   render and a printed plate.
2. *The first stipple read as a regular halftone screen.* Value noise on the
   axis-aligned grid at a few cells per mark makes a visible diagonal checker —
   §30's "random texture noise", found in a 6× crop. Fixed by rotating the
   sampling lattice ~34°, domain-warping it with a second coherent field, and
   flooring the mark size at 4 cells; the marks now clump the way drawn
   stippling does. Every stage is a pure function of cell coordinates (§27).

**Measured, terrain only** (40-cell frame band excluded; "base" is milestone 3's
look):

| | Classic base | Classic atlas | Archipelago base | Archipelago atlas |
|---|---|---|---|---|
| interior luma min | 42.4 | 41.0 | 34.6 | 33.8 |
| interior luma mean | 132.8 | **133.0** | 106.3 | **106.2** |
| interior luma sd | 31.32 | **31.89** | 27.66 | **28.30** |
| interior mean chroma | 59.7 | 51.96 | 70.3 | 51.96 |
| any-channel clipping | 0.70% | 0.73% | 0.03% | 0.03% |

Mean luma is unchanged to a fraction of a level (tint and wash are
luminance-preserving by construction), and contrast **rises** slightly. The luma
minimum drops 1.4 and 0.8 levels, entirely from paper grain. Terrain clipping is
unchanged (whole-image clipping rises to 0.87%/0.17% only because the cream
margin is itself bright).

**Across worlds — this time the opposite way round.** Milestones 2 and 3 keyed on
relief and drainage, so they were strong on Classic and faint on Archipelago. The
paper acts on the whole sheet, and Archipelago is mostly ocean, so it loses
**26%** of its chroma against Classic's 13%: its bright cyan sea becomes a muted
teal-grey, the largest single visual change either world saw in this phase. The
two worlds start 18% apart in chroma (59.7 vs 70.3) and land within 0.01 of each
other (51.960 vs 51.963) — not by clamping, since the reduction ratios differ
(0.871 vs 0.739), but because a common printing medium is what converges two
differently coloured subjects. Stippling is the mirror image: 13.9% of Classic's
pixels, 10.8% of Archipelago's, legible only over continuous canopy.

**Cost — not free.** 2048² render 598 → 915 ms (Classic), 295 → 597 ms
(Archipelago): the paper is four extra `vnoise` calls on every pixel, ocean
included. Accepted as a one-shot generate-time cost; the obvious first
optimisation is precomputing the two sheet-scale mottle octaves at coarse
resolution and sampling them bilinearly.

**Found in the real app:** two systems drawing *over* the finished raster knew
nothing about the frame (`lib.rs`'s river channel-mask tint and `map_overlay.gd`'s
markers), so a settlement at the extreme west edge put its marker partly on the
margin. Outside this milestone's `render.rs`-only scope; fixed the same day by
the follow-up below.

**Verified** (for milestone 4 and its follow-up together): `cargo test
--workspace` 383 passed / 0 failed, no expected value modified; clippy clean for
this milestone's files (and four `field_reassign_with_default` warnings the A/B
harness had accumulated cleaned up); clean headless load; windowed runs of both
worlds at 2048², 40 settlements, with frame, parchment and canopy texture reading
correctly under the overlay. The harness gained a `noatlas`/`withatlas` isolation
pair (milestones 2–3 fixed) plus `paperonly`/`stippleonly` dumps, since a
combined image cannot show which stage carries a change.

### Milestone 4 follow-up — the overlays learn about the frame (2026-08-17)

It turned out to be **four** systems, not two: the river tint and the GDScript
markers, plus `build_territory_texture`'s faction wash and
`build_province_boundary_texture`'s line, found while fixing the first two.
Territory was the worst — a semi-transparent fill over every owned cell, so any
faction reaching the sheet edge coloured the bare margin outright.

**Clip, don't inset.** The tempting fix is to give overlays the plate *interior*
as their coordinate space, as a real atlas plate is laid out. That is wrong here,
because `apply_border` **composites over the finished raster's outermost cells**:
the terrain under the margin is *covered*, not moved. Remapping markers inward
would shift every one away from the coastline, river or road it sits on — worse
than the defect. Making insetting correct would mean resampling the world into
the interior inside `render.rs`, moving every measurement above and forcing an
answer on aspect ratio when the frame is a fixed cell count per side. Not a
defect fix. So everything is handled at the neatline, differently per kind:

- **Linear features are clipped** (roads, sea lanes, province boundaries) — a
  road reaching the sheet edge genuinely continues past it.
- **Point symbols are placed or omitted, never sliced.** A settlement whose cell
  is under the frame points at no visible terrain and is not drawn; one whose
  centre is inside keeps its exact position and the clip trims any overhang.
- **Raster tints fade rather than cut.** The river tint and territory wash are
  multiplied by `1 - border_cover`, so they stop exactly where the paper wash
  starts.

**Where the geometry lives.** `render.rs` exports it: `border_width_cells` (`0.0`
when disabled) and `border_cover` (`0.0` in the interior, `smoothstep`-ramped to
`1.0` under the margin); `apply_border` was rewritten onto both rather than
keeping a second copy of `0.014 * gw`. `WorldGen::get_border_inset_frac()`
carries it across gdext as a **fraction of texture width**, which survives
`_displayed_rect()`'s letterbox maths without GDScript knowing the resolution.
`map_overlay.gd` derives `_interior_rect()` from it and scissors its canvas item
to that rect (`RenderingServer.canvas_item_set_clip` +
`canvas_item_set_custom_rect`) — one scissor for circles, arcs, polylines and
dashes rather than four hand-written clippers. `Control` re-sets both on every
`NOTIFICATION_DRAW`, immediately before `_draw()`, so the override lasts one frame
and needs no restore.

**Parity:** with `border_width_frac == 0.0`, `border_cover` is `0.0` everywhere;
each raster call site is written `tinted + (plain - tinted) * cover`, so
`cover == 0.0` restores the old value *bit-exactly*; `_border_frac == 0.0` makes
`_interior_rect()` return `_displayed_rect()` and skips the scissor.

**Measured at the failing case** — the same 2048² / seed 12345 / Classic world
with the fix stashed and applied, cropped 4× at the west edge, counting overlay
ink inside the frame band:

| | marker orange on margin | river-tint cyan on margin |
|---|---|---|
| before | 268 px | 67 px |
| after | **0 px** | **0 px** |

All differences lie in the frame band and within 4 px of the neatline; nothing in
the interior moved. Archipelago (35 settlements, sea routes on) is 0/0 too and
shows both rules at once: a coastal capital just inside is trimmed at the
neatline while its sea lanes are cut there.

## Milestone 5 — geological material exposure + local contrast (2026-08-18)

**Why these two.** §12 and §18 were the two items milestones 3 and 4 deferred,
and the reason for deferring each had gone. Together they attack §30's objective
from opposite directions: §12 puts *more real information* into the image, §18
makes information already there *easier to separate*.

Rejected: §16 (multi-scale detail) — largely delivered by milestone 4's paper
grain and stipple; §17 (colour vibrancy) — milestone 4 had just deliberately
*removed* 13–26% of the chroma; §20 (high-precision/tone-mapping pipeline) — its
payoff is HDR/wide-gamut output nothing consumed (**superseded by Ruling AN**,
see milestone 6); §21 (GPU) — a later milestone; §29 (quality tiers) — not yet
enough stages to tier.

**§12's plumbing, checked before committing.** The brief suggested Journey
Planner milestone 5's `build_cart_terrain`/`CART_TERRAINS` (`dca5954`). It is
the wrong source: a party-movement *surface* vocabulary (Paved Road, Dirt Track,
Open Plains …) derived from inputs `render.rs` already reads — a coarse
re-classification, not new information. The right one is
`cartalith_civ::build_lithology`: seven `LITH_KEYS` rock types from the
*tectonic substrate* (`age_field`, `volcanic_field`, `crust_field`,
`resistance_field`), which `render.rs` could not derive — and `lib.rs` already
called it (inside `compute_civilisation`, for the soil chain), so it was one call
in the file that already made it. Over Classic's land that vocabulary is **shale
45%, metamorphic 33%, basalt 11%, sandstone 7%, limestone 4%, granite 0.4%** —
and granite is what the reference's climate heuristic paints by default. The
renderer was showing one rock for a world that has seven.

**Built — §12, two halves.** Five new rock palettes (`rock_basalt`/
`rock_andesite`/`rock_limestone`/`rock_shale`/`rock_metamorphic`; granite and
sandstone existed), `litho_strength` and `litho_exposure`.
`RenderCtx::with_lithology` is a **builder**, like `with_splat`, so
`golden_parity_render.rs` stays positionally valid.

- `rock_material_col` **blends** the reference's `rock_col` toward the real
  rock's palette: the heuristic still carries surface character (scree really is
  paler than its parent rock), the lithology supplies identity.
- Bedrock **shows through thin soil** in `land_color`, gated on §12's own list —
  slope, vegetation potential (`w.c`), effective moisture — and scaled by the
  cover fraction not already rock or snow, so it is self-limiting and never
  bleeds through an icecap.

The lithology index is sampled through a **coherent positional jitter**
(`RenderCtx::litho_at`, ~10-cell wavelength): `build_lithology` is categorical,
so a contact sampled straight renders as a clean vector line — §30's "artificial
outlines" and "hard biome borders" at once. It is the renderer's own idiom, as
`bio_jitter` does for biome classification.

**Built — §18.** `local_contrast`, `local_contrast_radius_frac`,
`local_contrast_knee` and `apply_local_contrast` — the **first stage that is not
per-pixel**, necessarily: separating neighbouring materials is a statement about
a neighbourhood of the *finished* colour. It runs over the output buffer in
`lib.rs`, after the river tint and before the icon pass; `cell_color` is
untouched. §18's three constraints are met by construction rather than tuning:

- *No haloing* — the response is `d · exp(−(d/knee)²)`, so gain **falls to zero**
  as the luminance difference grows. An unsharp mask's halo is an overshoot
  proportional to edge strength; here the strongest edges (coastline, snowline,
  neatline) get essentially nothing. Checked on a 3× coastline crop: no rim.
- *No edge-detection artefacts* — the correction is **additive and equal on all
  three channels**, a pure luminance nudge; chroma is provably unchanged (51.79 vs
  51.80 below).
- *No excessive sharpening* — the band is a ~20-cell blur at 2048², acting on
  material-sized regions, not a 3×3 kernel.

It fades out under the frame via `border_cover`, so the margin's paper grain is
never amplified.

**Parity:** `js_reference()` gains `litho_strength`, `litho_exposure` and
`local_contrast` at `0.0`; `rock_material_col` returns `rock_col` before touching
a palette, the show-through block is inside an `if`, `apply_local_contrast`
returns before allocating. §12 is also off *by data* on that path, since the
golden test never calls `with_lithology`. One non-`#[ignore]`d test guards what
`render.rs` cannot: it is `#[path]`-included standalone by the golden test, so it
spells the rock order out as `LITHO_PALETTE_ORDER` instead of importing
`LITH_KEYS`, and `appearance_ab_dump.rs` — which sees both crates — asserts they
match.

**Two corrections, by measuring and by looking.**

1. *The geology gate was written in raw slope, and raw slope is
   resolution-dependent.* `slope_at` is a per-**cell** difference, so a mountain
   measures shallower on a finer grid: median land slope over Classic is
   **0.00354 at 512² and 0.00054 at 2048²**, 6.6× apart. The first
   `smoothstep(0.008, 0.050, slope)` gated the stage down to the steepest ~5% of
   land at the resolution the app runs at. Fixed by normalising to `slope * gw`,
   the project's own convention (`cartalith_civ::build_slope_field` stores
   `slopeAt(x,y)*GW`): Classic pixels moved by more than 3 levels went
   **1.17% → 6.61%**. The reference's own `material_weights` normalisers
   (`slope/0.04`, `slope/0.08`) inherit the same dependence and were left alone —
   they are golden-verified.
2. *Local contrast as a plain high-pass amplified the sheet's texture.*
   `luma − blur(luma)` sweeps in everything finer than the radius — milestone 4's
   ~3-cell paper grain and the C¹ seams of the mottle's value-noise lattices — and
   produced a faint rectangular quilting across land and sea. Fixed by making it
   a **band-pass** (subtract a small blur, not the raw image), so the boosted band
   is the material scale; the benefit survived (luma sd 33.10 before the fix,
   33.08 after).

**Measured** ("base" is milestone 4's look):

| | Classic base | Classic m5 | Archipelago base | Archipelago m5 | Wide (2048×1024) base | Wide m5 |
|---|---|---|---|---|---|---|
| interior luma min | 41.0 | 38.7 | 33.8 | 26.9 | 45.4 | 39.4 |
| interior luma mean | 132.75 | **131.60** | 105.98 | **105.31** | 136.98 | **135.23** |
| interior luma sd | 31.94 | **32.85** | 28.34 | **28.98** | 27.28 | **28.80** |
| interior mean chroma | 51.80 | 51.24 | 51.84 | 51.81 | 52.49 | 51.24 |
| any-channel clipping | 0.78% | **0.67%** | 0.04% | 0.04% | 0.00% | 0.00% |

Contrast **rises** in all three worlds while mean luma falls about one level and
clipping *falls* — separation bought from the middle of the range. Chroma moves at
most 1.25 of ~52, all of it geology's (rock palettes are less chromatic than the
tan they replace); local contrast alone (`lconly`) is 51.79 against 51.80. Luma
minimum drops 2–7 levels from local contrast deepening the darkest concavity — at
26.9/255 in the worst case, a deep shadow, not a black valley.

**Which stage carries what** (pixels moved by more than 3 levels per channel):

| | Classic | Archipelago | Wide |
|---|---|---|---|
| geology (§12) | 6.61% | 0.94% | 10.75% |
| local contrast (§18) | 24.90% | 11.69% | 31.52% |
| both | 27.46% | 12.18% | 35.58% |

Within geology the halves split 0.94% (rock palette) to 5.29% (soil show-through)
on Classic — at 2048² the reference's own rock *fraction* is small except near
summits, by the resolution dependence above.

**Across worlds.** Geology is strong on Classic and Wide and nearly absent on
Archipelago (0.94%) — a low-relief fragmented world has little steep, thin-soiled
ground, which is the honest answer, not a knob to force. Local contrast is
substantial on **all three**, because every world has material boundaries, which
is why it was worth pairing with a relief-keyed effect. At 3×: Classic's glacial
valley reads as depth rather than a pale smear, with no rim at the snow/rock
boundary (the strongest edge in the crop); its uplands pick up sandstone warmth
and escarpments read as exposed strata; Archipelago's islands gain limestone/
sandstone patches with ragged contacts and its mid-ocean ridges become legible;
Wide reads correctly at 2:1 and its frame band is **bit-identical** — 0 of
168 896 frame pixels changed, so `border_cover`'s fade is exact.

**Cost.** 2048² render 923 → 1110 ms (Classic, +20%), 607 → 752 ms
(Archipelago), 501 → 599 ms (Wide): three separable box blurs plus one `exp` per
pixel, one extra `vnoise` pair and a palette blend on land, and `build_lithology`
(one neighbour-free `par_iter` pass). Real-app `build_color_texture` end to end:
1442 ms Classic, 1085 ms Archipelago, 761 ms Wide — all one-shot.

**Verified.** `cargo check -p cartalith-godot --all-targets` and `cargo build
--release` clean (the debug cdylib hit the known `Access is denied` lock from a
running editor, so the debug DLL was built and exercised in a detached worktree);
`cargo test --workspace` **572 passed / 0 failed**, no expected value modified;
clippy clean for this crate's files; clean headless load. The real
`build_color_texture` path — which the dump harness does *not* exercise — was
run headless for all three worlds and produced correct PNGs with the river tint,
frame and non-square aspect intact.

## Milestone 6 — the GPU question, answered by measurement; and §29 quality tiers (2026-08-18)

**Why these two.** §21 (GPU) and §29 (quality tiers) were the two remaining items
with a real consumer. The brief said to *verify what is reachable before
committing* to §21, and that verification changed what got built:

- **GPU compute is reachable** — not through Godot's renderer (`gl_compatibility`
  cannot dispatch `RenderingDevice` compute) but through the standalone `wgpu`
  instance `cartalith-gpu` owns. On the session's adapter at 2048²: GPU-safe noise
  **2.8 ms** against 36.8 ms single-thread CPU, domain warp **8.0 ms** against
  794 ms.
- **But the renderer was not GPU-bound; it was single-core-bound.** Five
  milestones had grown `build_color_texture`'s per-pixel loop to ~1 s at 2048² on
  **one thread**, while every engine crate feeding it had been Rayon-parallel
  since `CPU_MULTITHREADING_SCOPE.md` milestones 2–3 — the last serial O(gw·gh)
  loop in the workspace, costing more than any GPU kernel would save.

So this milestone built the parallel CPU path and the tier ladder, and **not** a
WGSL port (arithmetic in *The §21 verdict*).

Rejected: **§17 (colour vibrancy)** — milestone 4 deliberately removed the
chroma, and the tier table below shows chroma is the renderer's most stable
statistic (51.3–52.8 everywhere). **§16 (multi-scale detail)** — still largely
delivered by milestone 4's grain and mottle; an explicit macro/meso/micro set
"would be relabelling, not building" (it shipped anyway — see *What the
milestones deferred*). **§20 (high-precision/tone-mapping pipeline)** — payoff is
HDR/wide-gamut output nothing consumed, and clipping was *falling* (0.78% →
0.68% on Classic), so the problem tone mapping solves was not present.

> **§20: superseded 2026-09-23 by an owner ruling (`LARGE_ITEM_RULINGS.md`
> Ruling AN), not by a new defect.** Both measurements still stand; the owner
> ruled §20 worth building *toward*, not only as a fix for a measured defect —
> the premise both rejections (here and milestone 5's) rested on. The staged
> first step: local contrast, the grade and the colour space run as one per-pixel
> pass with a single quantisation (`render::finish_rgb`), no `u8` round trip
> between them. The shipped default is byte-identical; graded and Display P3
> renders move up 1–3 levels, never down, landing 42–70% closer to the continuous
> value. **The output half is a separate piece of work:** the texture is `RGB8`
> and both export encoders are 8-bit, so tone mapping, a linear working space and
> an HDR output path have nothing to feed yet.

### Built — the parallel appearance pass

`cartalith-godot` gains `rayon = "1"` (the declaration five sibling crates carry;
nothing new in the dependency tree). Three loops became parallel:

- `build_color_texture`'s per-pixel loop in `lib.rs` — `par_chunks_mut(gw*3)` over
  rows, body unchanged including the river tint.
- `apply_local_contrast`'s luma build and correction loop.
- `box_h`, the horizontal half of every separable box blur — also speeding up
  `build_ao`, `build_hydro_wetness` and `smooth_sea_h`. `apply_local_contrast`'s
  two independent `blur_once` calls run under `rayon::join`.

`box_v` was left serial here: it walks columns, so each task would need `&mut` to
a disjoint stride of every row, which rayon cannot express over a flat buffer
without `unsafe`; blur-transpose-blur-transpose doubles memory traffic and
touches `smooth_sea_h`, on the JS-parity path. Flagged rather than reached for.
(The `box_v` parallelised over output rows on 2026-09-02 is a different function,
in `cartalith-terrain/src/analysis.rs`; `render.rs`'s is still a serial column
walk.)

**Bit-identical, proven three ways.** `cell_color` is a pure function of
`(&ctx, x, y)`, rows write disjoint bytes, and no float is reassociated, so §27
holds by construction. Checked by (1) a non-`#[ignore]`d test rendering all four
tiers serially and in parallel and comparing bytes; (2) the A/B harness rendering
each world both ways and `assert_eq!`ing at 2048²; (3) all **48** dumps re-run
after the `box_h` change and diffed: 48 of 48 byte-identical.

| | Classic 2048² | Archipelago 2048² | Wide 2048×1024 |
|---|---|---|---|
| `cell_color` serial | 1040 ms | 665 ms | 583 ms |
| `cell_color` parallel | **125 ms** | **70 ms** | **61 ms** |
| speedup | 8.3× | 9.5× | 9.5× |

End to end in the **real app** (headless Godot, one debug DLL, one generated world
at 2048×1311, `RAYON_NUM_THREADS=1` versus unset — a true A/B in one binary):

| `build_color_texture` | 1 thread | all threads |
|---|---|---|
| performance | 626 ms | **242 ms** |
| balanced | 809 ms | **252 ms** |
| quality | 955 ms | **293 ms** |
| ultra | 1008 ms | **289 ms** |

3.3× at Quality — lower than the harness's 8–9× because the real path also builds
the lithology field, copies into a `PackedByteArray`, constructs the `Image` and
runs the serial `box_v` halves — all excluded from the harness's
`cell_color`-only timing. The 955 ms single-thread figure agrees with
milestone 5's 1442 ms at 2048², which scales to ~924 ms at this resolution.

### Built — §29 quality tiers, designed from a measurement that contradicts §29

`QualityTier` (`Performance`/`Balanced`/`Quality`/`Ultra`) with `name`,
`from_name`, `ALL`, plus `TerrainAppearance::for_tier` and
`recommended_quality_tier()`; across gdext as `get_quality_tier`/
`set_quality_tier`/`list_quality_tiers`/`get_recommended_quality_tier` on
`WorldGen`.

**`Quality` is `TerrainAppearance::default()` returned unchanged**, not a
re-listing of its fields, so the ladder cannot drift from the tuned look even by a
typo — verified by a byte comparison test and by the three 2048² tier dumps, each
byte-identical to that world's existing `after` dump.

**The tier table comes from measured stage costs, and they contradict §29's
recipe.** `cost_table` in `appearance_ab_dump.rs` renders the full default look
with exactly one stage disabled, best of three, at 2048². Marginal cost, largest
first:

| stage | Classic | Archipelago | Wide |
|---|---|---|---|
| local contrast (§18) | 53 ms | 53 ms | 30 ms |
| paper grain + mottle (§16-ish) | ~18 ms | ~18 ms | ~6 ms |
| stipple | 3 ms | 6 ms | 6 ms |
| geology (§12) | ~0 ms | 0 ms | 6 ms |
| hydrology tint (§13) | ~2 ms | ~0 ms | ~0 ms |
| ambient occlusion (§15) | ~0 ms | ~0 ms | ~2 ms |
| relief lights 6→1 (§14) | ~0 ms | ~0 ms | ~0 ms |

The bottom four rows are **at or below the noise floor** of a single-machine
wall-clock measurement — the first single-sample version produced *negative*
marginal costs, which is the measurement saying it is not one. Best-of-three fixed
the sign, not the fact that those stages are free.

§29 prescribes a Performance tier with "basic hillshade, no expensive AO",
assuming raymarched AO and a full shading pass per light. This renderer's AO is
one separable blur computed once, and extra lights are five dot products against
a normal computed anyway; building from §29's text would have surrendered
milestone 2's relief for nothing. So the ladder drops stages in **measured cost
order**:

- **Performance** — no local contrast, paper fibre/mottle, stipple or geology;
  keeps all six lights, AO and the hydrology tint.
- **Balanced** — exactly `Quality` minus the two most expensive stages (local
  contrast, paper mottle). Lightening a 3 ms stage would give up image for no
  time.
- **Quality** — `default()`.
- **Ultra** — ten lights, `ao_strength` 0.32, `local_contrast` 0.62.

**The ladder drops texture, never identity.** Every tier keeps the paper tint, the
wash and the plate frame — what makes the sheet read as an atlas plate
(`VISION.md`) — since those are multiplies and a composite, not per-pixel noise. A
test asserts it.

Ladder cost (parallel, 2048², including local contrast): Classic **74 / 101 / 162
/ 163 ms**, Archipelago **38 / 58 / 127 / 130 ms**, Wide **40 / 53 / 88 / 89 ms**.
Performance is 2.2–3.3× cheaper than Quality; Ultra costs **the same as Quality**,
which is why `recommended_quality_tier()` never proposes it — it is a quality
choice, not a performance tier.

**Policy stayed with the owner.** This pass left `WorldGen` starting at `Quality`
on every device. `recommended_quality_tier()` reads `available_parallelism()`
(capping Android one rung lower) behind a getter that *offers* a tier; applying it
was left to a later decision. The Android device pass's 874 MB / ~31 s at
2048×1311 is the real consumer, and this milestone gives it two independent
levers — a 3.3× parallel render and a 2.4× cheaper tier — without choosing
between them.

**Parity:** `paper_tone`'s fibre and mottle now **each early-return on their own
zero** rather than sharing `paper_strength`'s gate — the milestone-2 rule one
level finer, and what makes a smooth-sheet Performance tier cost nothing instead
of computing four `vnoise` calls and multiplying them by 0. With both on the
arithmetic is unchanged, so `default()` is bit-identical. `js_reference()` needed
**no new fields** (`paper_strength: 0.0` short-circuits ahead of both gates), and
it is not a tier — `for_tier` is never on the parity path.

### The §21 verdict, with the arithmetic

A GPU appearance path would still win on raw kernel time. It is nevertheless **not
the next thing to build**, and the reason is the ratio:

- Before this milestone: ~955 ms of appearance inside a ~6.5 s generate+render at
  2048×1311 — 15%, worth a large, risky port.
- After: **293 ms of ~5.9 s — 5%.** A perfect GPU port taking the render to zero
  would save about 5% of the time to a new world.
- Against that: a WGSL `cell_color` means porting `material_weights`, 25
  palettes, the jittered `ramp3` micro-ramps, ten `vnoise` call sites, the
  lithology jitter and the AO/hydrology tables — in `f32`, since WGSL has no
  `f64` — producing a second renderer that **diverges from the golden-verified CPU
  one** under `DECISIONS.md` §7c and must be kept in step with every future
  appearance milestone. `cartalith-gpu`'s milestone 7 already reported one kernel
  that lost to CPU; this surface is much larger.

If picked up later, the beachhead is **`apply_local_contrast`**, not `cell_color`:
the single largest stage (30–53 ms), a self-contained whole-raster pass reading no
world fields — one upload, one download, no material logic — and its output
feeding a `u8` buffer bounds `f32` divergence by construction.

### Measured against §30's anti-list — all four tiers, three worlds

40-cell frame band excluded. "moved vs Q" is the share of interior pixels
differing from `Quality` by more than 3 levels in any channel.

| world | tier | luma min | luma mean | luma sd | chroma | clip % | moved vs Q |
|---|---|---|---|---|---|---|---|
| Classic | performance | 42.5 | 132.60 | 31.48 | 51.82 | 0.78 | 47.4% |
| Classic | balanced | 41.0 | 132.05 | 31.35 | 51.27 | 0.78 | 31.5% |
| Classic | **quality** | 38.7 | 131.92 | **32.79** | 51.40 | 0.68 | — |
| Classic | ultra | 38.0 | 131.63 | **33.06** | 51.30 | 0.68 | 3.8% |
| Archipelago | performance | 33.9 | 105.81 | 27.73 | 51.83 | 0.04 | 30.2% |
| Archipelago | balanced | 33.8 | 105.76 | 27.76 | 51.80 | 0.04 | 16.1% |
| Archipelago | **quality** | 26.9 | 105.55 | **28.93** | 51.93 | 0.04 | — |
| Archipelago | ultra | 26.8 | 105.51 | **29.01** | 51.92 | 0.04 | 0.2% |
| Wide | performance | 47.1 | 137.56 | 26.82 | 52.84 | 0.00 | 52.7% |
| Wide | balanced | 45.5 | 136.37 | 26.82 | 51.56 | 0.00 | 37.5% |
| Wide | **quality** | 39.4 | 135.61 | **28.60** | 51.47 | 0.00 | — |
| Wide | ultra | 38.4 | 135.21 | **29.13** | 51.33 | 0.00 | 6.2% |

Every tier is a real, visible change (16–53% of pixels), so none is a placebo.
Contrast rises up the ladder in all three worlds and clipping never rises with it
(Classic *falls*, 0.78% → 0.68%). Chroma moves at most 1.5 of ~52 across the
whole ladder — the tiers trade texture and separation, not colour. Luma minimum
falls up the ladder but never below 26.8/255.

One honest non-monotonicity: on Classic, Balanced's `luma sd` (31.35) is slightly
*below* Performance's (31.48). Not a defect — Balanced adds geology, whose rock
palettes are less contrasty than the tan they replace, while still lacking the
local contrast that actually raises `sd`. `sd` is a consequence of the ladder, not
its ordering.

**Crops at 3×, at the maximum-difference window:**

- **Classic, Performance vs Quality.** Performance keeps everything structural —
  glacial tongue, shaded ridge flanks, coastal escarpment, settlement dot — and
  loses the sheet's fibre, the rock-colour variety (the eastern scarp's sandstone
  warmth goes flat tan) and the crisp snow/rock boundary. The right trade for a
  cheap tier, and the crop that justified keeping six lights and AO.
- **Archipelago, Performance vs Quality.** The clearest case for the stipple: one
  smooth green wash against real clumped canopy. Both honest maps; only one an
  atlas plate.
- **Wide, Performance vs Quality.** An impact crater: pale limestone-grey rim with
  a sandstone patch inside against uniform tan. Both correct at 2:1, frame correct
  on all four sides.
- **Quality vs Ultra**, all three worlds: barely separable even at the
  maximum-difference window (0.2–6.2% of pixels); Ultra deepens the strongest
  shadows slightly and nothing else.

**A pre-existing artefact found by looking, and recorded rather than fixed in a
performance milestone.** The full-sheet downsample shows **rectangular blockiness
in the open ocean** — squares about 80 cells across at 2048². It is not from
milestones 4–6: it is present in the `js_reference` dump too, *more* visible
there because milestone 4's wash mutes it. The source is `seaColorCore`'s `n_low`
term, value noise sampled at `25.6/gw` — a lattice about 80 grid cells wide, with
the C¹ seams value noise has at low frequency — so it is inherited from the
reference HTML, and it is on §30's anti-list ("banding that looks artificial").
**The fix exists** as `TerrainAppearance::sea_grain_warp` ("Ocean grain warp",
added 2026-09-03): milestone 4's stipple fix reused — rotate the sampling lattice
and domain-warp it — at `0.0` it takes a dedicated branch that is the reference
expression exactly. Its `default()` value is `0.0`. Being inherited from the
reference is not a reason to keep the artefact: under `DECISIONS.md` §7p a
rendering improvement is the standard on its own terms, and the golden is not in
the way — it is the same gated-stage pattern every milestone here used, with one
extra line: `js_reference()` inherits this field from `default()`, so it must pin
`sea_grain_warp: 0.0` explicitly before `default()` turns it on.

### Verified

`cargo build -p cartalith-godot` clean (debug and release); `cargo test
--workspace` **1156 passed / 0 failed** across 89 suites, no expected value
modified; clippy **zero warnings for this crate**, test targets included; clean
headless load. The real `build_color_texture` path was driven headless at
2048×1311 for all four tiers, one thread and all threads, producing correct PNGs
and exercising all four new `#[func]`s including the unknown-name rejection.

Eight tests in `tests/appearance_tiers.rs` (synthetic 128×79 field, no
generator, 40 ms, so in the ordinary sweep): `quality_tier_is_exactly_the_default_look`,
`render_parallel_matches_serial_bit_for_bit`, `every_tier_renders_a_distinct_image`,
`every_tiered_stage_gate_is_load_bearing`, `every_ultra_tier_knob_is_load_bearing`,
`tier_table_is_monotone_in_cost`, `tier_names_round_trip_and_reject_junk`,
`recommendation_never_proposes_ultra`.

**Mutation-tested.** Forcing `paper_tone`'s mottle branch off was caught by
`every_tiered_stage_gate_is_load_bearing`; collapsing `Balanced` into `Quality` by
`every_tier_renders_a_distinct_image`. A third, sloppier mutation passed — because
the mutation itself was incomplete: **a mutation test only tests what the mutation
actually changed.**

## What the milestones deferred, and where each went

Each milestone recorded what it left for later; the lists overlapped, so they
are merged here. Where an item later got a home, the symbol is named; whether it
is finished is `STATUS.md`'s question, not this document's.

| Deferred item | Deferred by | Where it went |
|---|---|---|
| Milestone 1's elevation-ramp question | 1–6 | Answered both ways: `render::RAMP_PRESETS` (`list_ramp_presets`/`load_ramp_preset`) is the literal preset ramp; `TerrainAppearance`'s palettes, exposed through the "Colour grade" group, are the alternative |
| The atlas look proper (paper ground, stippling, plate border — `VISION.md`) | 2, 3 | Milestone 4 |
| §12 geological exposure | 3, 4 | Milestone 5 |
| §10/§11 slope/curvature modulation | 2 | Rejected in milestone 2 — slope and curvature were already `material_weights` inputs, and editing it was excluded (see *Rules*) |
| §18 local contrast | 2, 3, 4 | Milestone 5 |
| The overlay-over-frame defect | 4 | Milestone 4's follow-up |
| The GUI editing panel | 2–6 | `shell/workspaces/render_workspace.gd` (`GUI_SHELL_SCOPE.md` deferred it; the DCC shell built it) |
| §17 colour vibrancy | 5, 6 | Shipped 2026-08-24 as the "Colour grade" and "Grade field influence" groups in `render_workspace.gd` |
| §16 multi-scale detail as a control set | 5, 6 | Rejected in milestone 6 as "relabelling", shipped anyway: `detail_macro_weight`/`detail_meso_weight`/`detail_micro_weight` with a "Multi-scale detail" group. `LOD_DETAIL_SCOPE.md`'s LOD-D5 makes the weights scale-aware at tile resolution |
| §19 atmospheric/distance effects | 5, 6 | `haze_strength`, `atmo_desaturation`, `atmo_contrast`, under an "Atmosphere" group; the panel's help text owes a further, elevation-keyed haze distinct from these three |
| §20 high-precision display pipeline | 5, 6 | Staged half under Ruling AN (`render::finish_rgb`); the output half (tone mapping, linear working space, an HDR/non-8-bit output) is separate work |
| §21 GPU rendering path | 2–6 | Declined for now by *The §21 verdict*, which names `apply_local_contrast` as the beachhead |
| The ocean value-noise lattice | 6 | `TerrainAppearance::sea_grain_warp`, 2026-09-03 (see milestone 6) |
| Hand-lettered settlement glyphs | 4, 5, 6 | `map_overlay.gd`, not this raster — outside this document |

The 2026-09-06 correction that first recorded four of these as shipped
(`OUTSTANDING_WORK.md` §6.4) also listed the ocean lattice as "genuinely still
open" per `render.rs`'s doc comments — but `sea_grain_warp` had been added three
days earlier. The row above is corrected at the symbol.

<!-- A duplicate, shorter "Milestone 3" section briefly existed here, committed
by a concurrent fork that picked up milestone 3's in-progress render.rs changes
mid-tuning; its numbers (`smoothstep(0.72, 0.97, …)`) were the first-guess
parameters, not the committed `0.55, 0.88`. Removed; the section above is the
accurate record. -->
