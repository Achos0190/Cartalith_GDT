# Rivers in the map itself — scope document

Filed 2026-09-28, from the owner's river proposal (`OUTSTANDING_WORK.md`'s
count paragraph, item 63). This document defines the milestones; it carries
no status column — see `cartalith-native/docs/STATUS.md` for that, per
`CLAUDE.md`'s "One place holds state" rule.

## The owner's request, verbatim

> the rivers already on the map should be more profound themselves, like the
> new lines we have created. But this time in the map itself, no longer the
> current depressions.

Read together with the reference images the owner supplied (not in this
repository — described below from the ask itself and from Ruling BI/BL's
prior record of the owner's stated preferences), the request is: stop
drawing rivers as a stroke laid over a finished map, and instead make a
river's channel genuinely be water **in the map's own colour field** — the
same way the coastline and lake shores already are, since RV-4 gave those a
per-pixel, antialiased sub-cell edge. "No longer the current depressions"
targets RV-3's shaded valley groove: the owner wants the river to read as
water widening downstream, not as a shaded ditch with a line drawn over it.

## The four reference images (description, since the images are not checked in)

The owner did not attach files to this repository; the description below is
built from the request's own wording and is a design target to verify
against, not a citable source. Four images were referenced as showing:

1. **A river channel painted in the sea's own colour**, narrow at a
   mountain source and widening smoothly toward a broad river mouth — the
   channel reads as the same substance as the ocean, not a different-coloured
   line crossing dry land.
2. **Soft, feathered banks** with an optional thin, darker outline drawn in
   the active style's ink colour — visible in ink/parchment/atlas-like
   styles, absent or very light in cleaner styles.
3. **Lakes and rivers sharing one fill**, merging into the sea at the mouth
   with no seam, colour step or drawn boundary between river, lake and
   ocean.
4. **Lighter or greener floodplain tints** flanking the channel instead of a
   dark shaded trench, plus small fan-shaped deltas where a large river
   meets open water.

## Current state, read at its symbols

Rivers today are a **vector stroke drawn on top of a finished raster**, not
water in the map's own colour field. In order:

- **RV-2 — smooth centrelines.** `cartalith-godot/src/river_stroke.rs`
  builds a per-point width/colour/cut model from the traced network
  (`river_half_width_profile`, `settle_join_widths`, `draw_ranks`,
  `stroke_pieces`, `coast_end`). `lib.rs::WorldGen::river_draws` (line
  ~10602) turns that into `RiverDraw`s coloured from `RIVER_ORDER_RGB`
  (`lib.rs`, a discrete Strahler-order palette, light headwater to dark
  trunk). This is real centreline geometry — exactly what the new milestone
  below reuses as its input skeleton — but it is emitted as a stroke to
  render, not as membership in the map's water classification.
- **RV-3 — the shading groove, already softened.** `valley_shade.rs`'s
  `valley_shade_field` (its own module doc: *"the carve filled back in, then
  a smooth valley cut along each drawn line, as deep as the carve was
  there"*) is a **rendering-only** height field the three render paths shade
  instead of the true hydrological carve. It already answers half the
  owner's "no longer the current depressions" complaint — the dark, stepped
  grooves from the raw D8 carve are gone — but the result is still a shaded
  depression, water-coloured nowhere; the actual blue pixels still come from
  the RV-2 stroke drawn on top of that shading.
- **RV-4 — the per-pixel shoreline, built for sea and lakes only.**
  `render::shore_field`/`shore_field_forced` (`render.rs` ~10740) compute a
  signed per-cell distance from the water classification
  (`shore_depth`/`shore_depth_forced`), which `shell/map_shore.gdshader` and
  `shell/shore_field.gdshaderinc` contour at zero to draw an antialiased,
  sub-cell coast and lake edge. **Rivers are not inputs to this field at
  all** — `shore_field`'s classification comes from `build_water_bodies`
  (ocean/lake only), so a river channel narrower than one cell, which is
  most of the network, has no representation in it. `river_under_water.gdshader`
  exists only to let the *stroke* hide correctly under RV-4's now-smooth sea
  and lake edge at a mouth, confirming rivers are still the layer being
  hidden by water, not water themselves.
- **RV-5 — the export path, still a second pass over a finished bake.**
  `render.rs::paint_vector_rivers` (~8592) is explicit in its own doc
  comment: it takes `bake_rect`'s finished output and "re-renders" every
  pixel a stroke reached, composited inside `land_color` — "the slot a
  deep-zoom tile composites its own layer in." Its own reasoning for being a
  *second* pass rather than a `river` argument to `bake_rect` is that the
  local-contrast measurement runs on the river-free map first; a river in
  that measurement would ring every course with a halo. That reasoning holds
  today because rivers are a paint-over-the-bake step, not part of the
  colour field being contrast-measured — exactly what this scope proposes to
  change.
- **The map-overlay river layer.** `godot-project/map_overlay.gd`'s
  `_draw_rivers_into` (~5242) draws `WorldGen::river_view_mesh`/
  `river_view_mesh_runs` — a tapered, joined stroke mesh in screen pixels,
  textured with `WorldGen::river_color_texture` — as its own `CanvasItem`,
  above the base map texture. This is the fit-zoom/interactive path's
  equivalent of RV-5's export-time paint-over: a separate draw call laid on
  top, not part of the map's own colour texture.
- **River colour and style hooks.** `render::river_style_color` (`render.rs`
  ~8458) already exists as exactly the kind of per-style hook the new
  milestones need: it takes a straight RGBA river colour and moves it toward
  `TerrainAppearance::river_ink` (an ink colour, by `river_ink` blend
  strength) and scales alpha by `river_opacity` — the mechanism Ruling BI's
  per-preset river treatment and Ruling BL's dimmer Night rivers already run
  through. `RIVER_ORDER_RGB` (`lib.rs` ~4241) is the Strahler-order palette
  it starts from. Both are reusable as-is; neither assumes the stroke
  rendering path.

**In short: the network's geometry, per-point width, join rules, colour
model and per-style hooks are all already built (RV-2's real work). What is
missing is exactly what the owner is asking for — membership of that
geometry in the map's own per-pixel water field, the way RV-4 already gave
the coast and lake shore.**

## Milestones

Numbered `RIM-` to avoid colliding with `REFERENCE_MAP_RECONSTRUCTION_RESEARCH.md`'s
own `RM-*` numbering.

- **RIM-1 — a per-pixel river distance field, merged with RV-4's shore
  field.** Build a signed distance-to-water field from the RV-2 centrelines
  and their per-point widths (`river_half_width_profile`), sampled the same
  way `shore_depth` samples the coast/lake classification, and combine it
  with `shore_field`'s existing sea/lake distance (minimum of the two
  signed distances, so whichever water is closer wins — this is exactly how
  a river merges into a lake or the sea with no seam). Contour it at zero
  through the same shader machinery RV-4 already built
  (`map_shore.gdshader`, `shore_field.gdshaderinc`), so rivers gain
  antialiased sub-cell edges for free rather than a second AA technique.
  This is the milestone that answers reference image 1 and 3.
- **RIM-2 — an optional per-style bank outline.** A thin darker outline at
  the zero-crossing, in the active style's ink colour, gated per preset the
  same way `river_style_color`/Ruling BI's per-preset river treatment
  already gate colour and opacity — on for ink/parchment/atlas-like
  presets, off or very light for cleaner ones. Reference image 2.
- **RIM-3 — no groove; a floodplain tint band.** The renderer's shading
  stops reading `valley_shade_field`'s carve-derived depression for any
  cell RIM-1 now classifies as river water (water cells already skip it, by
  `valley_shade.rs`'s own "water is never touched" rule — this extends that
  rule to the new river-water class rather than changing the module). A
  lighter/greener tint band is added just outside the river's edge, keyed
  off RIM-1's distance field the way a shoreline's own biome band already
  keys off `shore_depth`. Reference image 4's floodplain half.
- **RIM-4 — deltas and lakes-on-course.** Where a large river's mouth
  discharge exceeds a measured threshold, widen the distance field into a
  small fan shape rather than a single terminal width step (deltas,
  reference image 4's second half). Where a traced run passes through a
  lake mid-course (the "beads of square lakes" RV-2's own status notes left
  to RV-4/RIM), the merged distance field already handles it by construction
  once RIM-1 lands, since the river and lake share one signed field — this
  item is the verification pass confirming no seam remains at a mid-course
  lake, not new geometry.
- **RIM-5 — small streams faded out when zoomed out.** Below a stream-order
  or discharge threshold that scales with km-per-pixel (the same shape as
  `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`'s EF-1/EF-9 scale-dependent
  detail and `LOD_DETAIL_SCOPE.md`'s per-tile river SDF threshold), a
  headwater's contribution to the distance field fades rather than popping.
  This folds in the "zoom-sensitive rivers" backlog row (`OUTSTANDING_WORK.md`,
  owner 2026-09-27) rather than leaving it a separate research task — see
  "Supersession" below.
- **RIM-6 — retire the main-river overlay lines, except in line-art
  styles.** Once RIM-1 ships, `map_overlay.gd::_draw_rivers_into` and
  `render::paint_vector_rivers` stop drawing the stroke into the default
  render paths — the river is already water in the base colour field, so
  drawing a second stroke on top would double it. **Exception:** a future
  line-art or blueprint-style preset that wants a schematic line rather than
  painted water keeps the stroke path available, gated per-preset the same
  way RIM-2's outline is. **Hit-testing is kept regardless of what is
  drawn**: `_river_pick_radius_cells`, `right_dock.gd`'s river hover/click,
  `get_rivers`, and `MAP_CONTEXT_SCOPE.md` §11's river identity/trace/catchment
  picks all key off the RV-2 centreline data, not off the drawn stroke —
  none of that changes.
- **RIM-7 — export and tile parity, one shared path.** `paint_vector_rivers`
  (RV-5) and the LOD tile rasterizer both fold RIM-1's field into their own
  colour computation the same way `bake_rect`/`render_biome_tile_rgba`
  already fold in the shore field, rather than keeping the current
  paint-over-a-finished-bake shape. One shared function decides "is this
  pixel river water, and by how much," called from the screen texture, the
  export bake and the LOD tiles — the same discipline RV-5's own doc comment
  already states ("cannot place a pixel differently") for the shore/bake
  relationship.

## Acceptance bars

- **Per-pixel water coverage versus the overlay.** On the owner's world
  (2048×1311, seed 246371), measure the count of screen pixels classified as
  river water by RIM-1's field against the count of pixels the current
  `_draw_rivers_into` stroke currently paints, at a fixed zoom; the two
  should agree within the antialiasing band's width (a few pixels), not
  differ by a channel's full width — this catches a distance field that is
  systematically narrower or wider than the stroke it replaces.
- **Seam checks.** Zero visible colour discontinuity at (a) a river-lake
  junction, (b) a river mouth into the sea, (c) two tributaries joining —
  measured the way `_riverzoom_probe.gd` already measures confluence gaps
  (0.0-cell target), extended to sample the merged distance field's value
  either side of each junction rather than the stroke's drawn pixels.
- **Per-zoom performance within Ruling BP's interactive bar.** RIM-1's field
  build must not push a frame with rivers on past Ruling BP's 16.7 ms
  interactive target at any of `_riverzoom_probe.gd`'s existing zoom steps;
  RV-2's own status note already records the rivers-off baseline at 16.7 ms
  and rivers-on median frames at 19-28 ms post-RV-2 — RIM-1 must not regress
  that, and should be measured against it directly, not against a fresh
  baseline.
- **Screenshot checks.** Before/after pairs at fit zoom and at deep zoom (the
  same crops `_riverzoom_probe.gd`/`_rivstyle_probe.gd` already use),
  confirming: no drawn depression/groove beside a river (RIM-3), a visible
  width taper from source to mouth (RIM-1), a merged lake/river/sea fill
  with no seam (RIM-1/RIM-4), and the per-style bank outline present only
  where RIM-2 turns it on.
- **Probes.** Extend `_riverzoom_probe.gd`, `_rivcolour_probe.gd` and
  `_shorestep_probe.gd` (RV-4's own coast-stepping probe) rather than adding
  parallel new ones, since RIM-1 is explicitly built as RV-4's mechanism
  extended to rivers. A new probe leg is warranted only for RIM-2's outline
  and RIM-4's delta fan, which have no existing analogue.

## Risks

- **Performance of a per-pixel distance field at 8192.** RV-4's `shore_field`
  is already one pass with nine neighbour reads per land cell, `rayon`-parallel
  by rows, linear in cells — but it reads a coarse two-state (ocean/lake)
  classification. A river distance field instead samples a dense polyline
  network with per-point widths; a naive nearest-point search per cell is
  not linear in cells, and needs a spatial index (a grid of nearby
  centreline segments, or a signed-distance rasterization pass seeded from
  the stroke geometry itself) to stay in the same cost class as RV-4. This
  is the milestone's main open engineering question and should be measured
  before committing to an approach, per `MISTAKES.md`'s "measure before
  pinning a constant" rule.
- **Double-counting cost with RV-5's existing second pass.** RIM-7 removes
  `paint_vector_rivers`'s reason to exist as a second pass at all (its own
  doc comment's reason for being a second pass — avoiding a contrast-measurement
  halo — stops applying once rivers are part of the colour field the
  contrast measurement reads). Folding it back into `bake_rect` changes what
  that measurement sees; this needs its own before/after local-contrast
  comparison, not an assumption that removing a pass is free.
- **`valley_shade_field`'s "water is never touched" invariant.** RIM-3
  extends that invariant to a new water class (river, not just sea/lake);
  the module's own doc comment states this guarantee is relied on by every
  water test in the three render paths (`RenderCtx::water_height`) — RIM-3
  must not weaken it, only widen which cells qualify.
- **Golden parity.** River *generation* (`compute_flow`, `trace_river_polylines`,
  `carve_channel_network`, `build_water_bodies`) is untouched by every
  milestone above — this is a rendering-layer change only, the same class
  RV-2 through RV-5 already were. No `golden_parity_*` fixture should move.
  If a milestone's implementation touches anything upstream of rendering,
  that is a scope violation, not an incidental extra.

## Owner questions, with defaults

1. **Should RIM-6 remove the stroke draw call entirely for the default
   preset the day RIM-1 ships, or keep both paths side by side for a
   comparison period?** Default: remove it for the default preset once
   RIM-1's coverage acceptance bar passes, per `CLAUDE.md`'s standing
   instruction that a deliberate engine/graphics change away from legacy
   behaviour is correct without a parallel path.
2. **Does the bank outline (RIM-2) default on or off for styles not named
   in the owner's description (Default, Night, Vintage, Blueprint)?**
   Default: off outside ink/parchment/atlas-like presets, matching how
   Ruling BI already scoped other new drawing techniques per-preset.
3. **What Strahler order or discharge threshold gates RIM-4's delta fan?**
   Default: measure the discharge distribution at real river mouths on a
   few worlds first (per `MISTAKES.md`'s mutation-and-measure discipline)
   rather than picking a round number; propose a value back to the owner
   once measured.
4. **Does RIM-5's fade-out threshold apply uniformly, or should it await
   `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`'s EF-1/EF-9 multi-resolution
   work landing first, since both touch scale-dependent river detail?**
   Default: RIM-5 ships independently, using the same km-per-pixel input
   `LOD_DETAIL_SCOPE.md` already plans to expose, and is revisited if EF-1/9
   changes that input's shape later — not blocked on research that has no
   owner-scheduled build date.

## What this scope does not touch

River **generation** — flow accumulation, channel tracing, depression-filled
routing, water-body classification — is untouched by every milestone above.
No generation golden moves. This document is a rendering-layer proposal only,
in the same class as RV-2 through RV-5 before it.

## Supersession

This document folds in and supersedes the following in-flight or planned
work, noted at each source below with today's date:

- `OUTSTANDING_WORK.md`'s "Zoom-sensitive rivers: from a symbolic line to the
  real river" row (owner, 2026-09-27) — its research scope (cartographic
  generalization by zoom, stream-order fade) becomes RIM-5 here, built on
  RIM-1's field rather than on the retired stroke.
- `OUTSTANDING_WORK.md`'s residual "rivers ignore the style preset" work for
  the river-specific half — the per-style colour/outline/opacity hooks
  (`river_style_color`, Ruling BI/BL) are kept and reused by RIM-1/RIM-2
  exactly as built; the row's remaining roads/borders/labels scope is
  unaffected and stays open under its own row.
