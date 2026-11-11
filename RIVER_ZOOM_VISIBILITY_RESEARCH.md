# River zoom visibility — research and a proposed rule

Agent-produced 2026-11-11, for the `OUTSTANDING_WORK.md` §2.5 row *"Zoom-dependent
river drawing (owner request, 2026-11-09)"*. Documentation only: no code was
changed and nothing was built or measured here. **Research, not a scope**; it
carries no status column (`cartalith-native/docs/STATUS.md` does).

> Owner, 2026-11-09: *"smaller estuaries, rivers or channels won't be drawn until
> the appropriate zoom has been reached. There should be research done how other
> map tools handle this."*

**How to read the evidence.** Web claims are tagged **[fetched]** (the page was
opened this session and the figure read from it), **[search]** (a search-result
summary; the page itself was not opened) or **[unconfirmed]** (recalled, or no
source found; do not build on it). Claims about this tree were read at the
symbol named; no document's claim about itself was trusted. Every number in §3
is a cited constant, a figure derived from one, or a **labelled judgement**
that RZ-0 must measure before it is pinned (`MISTAKES.md`: never assert a
constant against itself; "measured" only for numbers just produced).

---

## 0. Summary

1. **Every mature tool gates waterways by class or importance per zoom, and
   fades or tapers rather than pops.** Class is the cheap proxy
   (OSM Carto, OpenMapTiles); the generalisation literature ranks by
   upstream drainage area and keeps a count that falls with the square root
   of the scale ratio (Töpfer). **Order is the weaker key** — the NHD
   pruning work uses drainage area precisely because order is not stored and
   is distorted in braided and large rivers.
2. **Cartalith already has half of this.** RIM-5 fades *order-1* streams
   below 0.8 screen pixels per cell. That keys on Strahler order, and this
   tree's own comments record that order and discharge disagree: the owner
   world's biggest single outlet (1.5 % of its grid) is an order-1 run
   (`COMB_DISCHARGE_FRAC`'s doc, `cartalith-hydrology/src/lib.rs`). So the
   current fade would thin that river with the headwater trickles when the
   map is zoomed out (inferred from the code; not rendered here).
3. **Recommended rule (§3):** key visibility on **discharge, expressed as a
   length-weighted rank `u`** per world, and keep a fraction
   `f(ppc) = min(1, (ppc / P_FULL)^0.5)` of the drawn network length (Töpfer's
   square-root law, in pixels-per-cell instead of map scale), largest
   discharge first, with a half-open fade band on `u` so nothing pops. The
   rank is baked once per world into the field's B channel; the per-pixel
   test is one `smoothstep` in the shader that already runs RIM-5. It changes
   drawing only, never the generated network.

---

## 1. How other tools do it

### 1.1 Slippy-map tile styles

- **OpenStreetMap Carto** [fetched, `style/water.mss`, master]. Waterway lines
  start at **z8** as "low-zoom water lines" with widths z8 0.7, z9 1.2, z10 1.5,
  z11 1.5; **river** proper from **z12** (width z12 2.3, z13 3, z14 4.5, z15 6,
  z16 8, z17 10, z18 12), **canal** from z12 (z12 1.12 … z18 5.6), **stream**
  from z12 permanent and z13 intermittent (z12 0.8, z13 1.4, z14 2 … z18 4),
  **drain/ditch** from z12/z13, labels river z13, stream/ditch z15; intermittent
  is a dash (4,3), at low zoom 8,4. Lesson: class gates *appearance*, width grows
  roughly geometrically with zoom (about 1.3 to 1.5 times per level at z12 to
  z14, then slower), and the thinnest class starts about 4 zoom levels after the
  river class.
- **OpenMapTiles `waterway`** [fetched, schema page]. Five classes (`stream`,
  `river`, `canal`, `drain`, `ditch`). Natural Earth rivers cover z3 to z8 and
  OSM waterways z9 and up; **up to z11 only `river` appears, `canal` is added at
  z12, no class filter from z13**; at low zoom unnamed or very short linestrings
  are dropped. Lesson: class tiers plus a length filter (a crude importance
  measure), each tier opening at a fixed zoom.
- **Mapbox Streets v8** [fetched]. `waterway` has a minimum zoom of 7; classes
  river ("from the Amazon to small creeks a few meters wide"), canal, stream
  ("no wider than 1-2 meters"), stream_intermittent, drain, ditch. Larger rivers
  are also polygons in the `water` layer, and the docs say line styling "should
  be biased toward the smaller end of the scales": the line is the thin
  symbol and the polygon is the real river. Per-class zoom is not documented
  there. (The style spec's `interpolate` on `zoom` is the usual width-by-zoom
  mechanism; the exponent most styles use is [unconfirmed].)
- **Google Maps** [unconfirmed]. Proprietary; no published per-zoom rule found.
  Observed behaviour (rivers by size, labels later than lines, thin lines that
  widen) is consistent with the above but is not evidence.
- **Natural Earth** [fetched, 10m rivers page]. `scalerank` "ranked by relative
  importance"; the basic set is for general maps "up to about 1:30,000,000", the
  North America and Europe supplements (scale ranks 9 to 12, about four times as
  many features) for river-focused maps at roughly 1:5 million to 1:20 million.
  A per-feature `min_zoom` attribute exists in the dataset's newer releases
  [unconfirmed: the page read did not mention it]. Lesson: a hand-assigned
  integer importance, compared with a per-scale cut.
- **Esri ArcGIS** [fetched, Pro help]. Visibility is a layer's min and max scale;
  everything draws at all scales by default, and Esri advises giving two
  adjacent layers the same boundary scale so detail layers do not overlap at a
  switch. Lesson: a hard cut at a scale, with the overlap caveat. Nothing
  river-specific.
- **USGS NHD / NHDPlus** [search]. The generalisation work (Stanislawski and
  Buttenfield) prunes by **upstream drainage area (UDA)** and reach code, not
  order, "until the subbasin channel density meets a user-specified threshold",
  re-run per target scale; one example uses a 1.5 % UDA cut to mimic a 1:1 million
  flowline network, another selects as primary stem paths all channels that drain
  20 % or more of the subbasin. The stated reason order is avoided: it was "not
  explicitly coded in the NHD", and NHDPlus V1 orders were "unpleasingly large"
  in big rivers and braided areas. The NHD display thresholds themselves (as the
  National Map renders them) were **not found** [unconfirmed]; the
  pruning papers' exact formulas were not opened.

### 1.2 Generalisation theory

- **Töpfer's radical law** (Töpfer and Pillewizer 1966) [search: ISPRS Wilmer 2010;
  arXiv 1102.1561]: `n_f = n_a · sqrt(M_a / M_f)`, the number of objects kept at
  the derived scale from the source count, `M` the scale denominators. It says
  **how many** to keep, not **which**. Applied to hydrography by Stanislawski
  (2009) and found to track the percentage of channel length displayed, but the
  same literature reports problems pruning 100K to 500K, and Austrian maps
  vary from the law, so it is a first guess, not an exact constant.
  Variants use network length instead of object count [search].
- **Strahler and Horton orders** (Horton 1945; Strahler 1957) [search, via the
  NHD papers]: the conventional way to pick primary stems. Bifurcation ratios of
  about 3 to 5 (so each order is a few times fewer and longer than the one
  below) are the textbook range [unconfirmed here, recalled]; a "stream" can be
  defined by width at the mouth or by extent on the map [search]. **Shreve
  magnitude** (count of upstream sources) behaves like discharge and is the
  order-like quantity closest to UDA [recalled, unconfirmed].
- **swisstopo** [unconfirmed]. The searches returned swisstopo's role only in
  road-network generalisation research and a CARGEN river-selection paper
  (length and density thresholds, 1:5,000 to 1:25,000, [search]); no swisstopo
  river-selection rule was found. Do not cite swisstopo for a number.
- **Pruning keeps the network connected** [search, Stanislawski]: whole reaches are
  removed, "preserving topologic integrity of the flow network". The
  recommended rule below has the same property by construction (§3.2).

### 1.3 Fantasy and worldbuilding tools

- **Azgaar's Fantasy Map Generator** [search]. The generator computes river
  flux from precipitation and terrain; an older CSS sample sets rivers at a
  fixed stroke of 0.4 px, and version 1.65's changelog lists "refactoring river
  rendering". Whether the current build gates rivers by flux per zoom was not
  confirmed; its published river width law is [unconfirmed]. It is a vector
  SVG map with a default maximum zoom of 20 (the sources disagree: 20 vs 40).
- **Wonderdraft, Inkarnate, Dungeondraft** [unconfirmed]. These are
  hand-drawn: rivers are authored paths or brush strokes, so no per-zoom
  hydrological rule was found for any of them and none is claimed. (The
  owner's request is therefore closer to GIS tile styling than to what these
  tools do.)

### 1.4 How they avoid popping, and what they say about estuaries

- **Width by zoom, not a switch.** OSM Carto's low-zoom widths 0.7 to 1.5 px
  across z8 to z11 and the river widths from z12 are a ramp, so a feature
  never appears at full weight; the *opening* of a class is still a step at
  one integer zoom (a real pop that tile styles accept) [fetched].
- **Alpha fade over a zoom band** is the usual remedy where a class must open
  mid-zoom; Esri's advice about equal boundary scales [fetched] is the same
  problem seen from the layer side.
- **Line to polygon handover.** Mapbox keeps the line "biased toward the smaller
  end" because large rivers are also polygons [fetched]; the line is a symbol
  for a river too thin to see as an area. Cartalith's painted river is already
  the "area" form (RIM-1), with a 1 px floor.
- **Labels open later than lines** (OSM Carto: river lines z12, labels z13;
  streams z12/z13 lines, z15 labels) [fetched]. Cartalith has no river label
  today as far as the overlay search found (§2.4); a label rule would follow
  the line rule, not lead it.
- **Estuaries, deltas, braided channels.** None of OSM Carto, OpenMapTiles or
  Mapbox has an estuary or delta class in the waterway layer [fetched]: a wide
  tidal mouth is a **water polygon** (`water`/`natural=water`) drawn at the
  zoom of its area, not a waterway line. The generalisation papers treat
  braided networks as a special case that the stem-path method reduces to a
  primary channel [search, Buttenfield et al. 2013's UDA example]. Neither a
  braided-channel nor an estuary-funnel rule from a mapping product was found
  [unconfirmed]; §3.6 therefore proposes the narrowest thing consistent with
  what Cartalith already draws.

---

## 2. What Cartalith has today (read at the symbols)

Opened for this document: `river_stroke.rs` (`river_px_width`, `RiverPoint`,
`DrawnRun`, `o1_deemphasis`, `o1_distance_fade`, `O1_GONE_PPC`,
`O1_FADE_FULL_PPC`, `MIN_STROKE_PX`, `EDGE_FRINGE_PX`), `river_field.rs`
(module doc, `coverage`, `edge_px`, `pixel_paint`, `painted`, `CHANNELS`,
`MAX_TEXELS`), `river_delta.rs` (module doc), `lod_bridge.rs`
(`continuous_level`, `morph_for_zoom`), `cartalith-hydrology/src/lib.rs`
(`channel_disc`, `river_width_scale_k`, `river_draw_plan`,
`COMB_DISCHARGE_FRAC`), `map_shore.gdshader` (header and RIM-5 uniforms),
`map_overlay.gd::_draw_rivers_into`, and the documents `RIVERS_IN_MAP_SCOPE.md`,
`RIVER_GATE_MEASUREMENTS.md`, `LOD_DETAIL_SCOPE.md`, `3D_VIEWPORT_SCOPE.md` §3.3,
`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.

### 2.1 The data

A drawn river is a `DrawnRun`: per point a `width` in grid cells, an `order`
(Strahler of the nearest traced cell), a `discharge` (`flow_discharge` at that
cell, NaN for "no reading") and the run's `own_order`. `RiverPoint` carries
`width_cells`, `order`, `own_order`, `discharge` to the one width/visibility
seam, **`river_px_width`**, whose own doc says it is "the one seam that decides
how wide, how opaque and whether a river point is drawn", returns `None` for
"not drawn at this scale", and **today never returns `None` and reads neither
`order` nor `discharge`**. The seam was left for exactly this work.

The width law is `channel_disc`: half-width `(0.6 + 3·mag² + 0.45·(o−1)) ·
slope_fac · width_k` cells, clamped to `[0.5, 9·width_k]`, with
`mag = min(1, ln(Q/T)/lmax)`; `width_k = river_width_scale_k(map_km) =
clamp(800/map_km, 1/16, 16)` (it shrinks on a wide world). So a river's **width
in cells is a symbolic law, not a physical channel width**, and the slope
factor makes it non-monotone in discharge.

### 2.2 How a river reaches the pixels (three paths, one law)

- **Screen.** For a look with `smooth_shores` and `rivers_as_water`
  (`river_field::painted`), `river_field::build` makes an RGBA half-float
  distance field at `field_scale(gw,gh)` texels per cell (`MAX_TEXELS` =
  12 582 912 texels, i.e. about 100 MB at 8 bytes). Channels: `R` distance to the
  nearest centreline, `G` that segment's half-width, `B` the **order-1 weight**
  (1 where the run's own order is 1), `A` the plate-frame weight. The shader
  `map_shore.gdshader` evaluates `coverage` per pixel from a `ppc` uniform; the
  law is mirrored in Rust (`river_field::coverage`, tests hold them equal). A
  look that does not paint draws the vector stroke (`map_overlay.gd::_draw_rivers_into`
  from `river_view_mesh`/`river_view_mesh_runs`), which never has a RIM-5 fade
  (RIM-5 is "in the painted path alone").
- **Deep-zoom tiles and export (RIM-7).** `river_field::PaintSource::raster`
  runs the same per-texel evaluator and `pixel_paint` at the tile's or export
  block's own pixels, with that raster's `ppc`
  (`lod_bridge.rs` takes `paint: Option<&PaintSource>`; `export_raster.rs`
  `paint_under`). So density, hence visibility, is a function of the raster's
  own pixels-per-cell.
- **Delta fans** (`river_delta::delta_fans`): extra runs for a mouth whose
  `own_order >= MIN_ORDER` (3) and whose discharge is at least
  `MIN_DISCHARGE_FRAC` (2e-4) of the grid's cells; the same segment list feeds
  all three paths; one gate (`fans_drawn`) so they cannot disagree. Gate
  numbers and the owner's 2026-11-01 "keep both" are in
  `RIVER_GATE_MEASUREMENTS.md` (Ruling EA item 9).

### 2.3 What zoom rules exist already

- **`o1_deemphasis(zk)`**: order-1 streams drawn at 0.55 width and 0.4 alpha at
  the opening view, reaching full by `zk = 8` pixels per cell. The numbers are the
  *reference's* own (v2.11 lines 9577-9578, per the function's doc), so the
  number 8 is a cited constant.
- **RIM-5 `o1_distance_fade(ppc)`**: `smoothstep(0.2, 0.8, ppc)` on the order-1
  share's alpha; both ends **labelled judgement**, tuned on the owner's
  2048x1311 world (`_riverzoom_probe.gd --rim5`, 2026-10-04): the contain fit
  there is 0.46 pixels per cell, the cover-fit opening view 0.65, the camera's
  minimum zoom (`ZOOM_MIN` 0.4) 0.19. Other sizes were not measured.
- **`MIN_STROKE_PX` = 1** and `EDGE_FRINGE_PX` = 1: no drawn channel is narrower
  than one pixel (the owner's RV-2 bar; a sub-pixel antialiased stroke read
  dotted).
- **Tile level**: `continuous_level(ppc, gw) = log2(max(1, gw·ppc/TILE_PX))` and
  `morph_for_zoom` cross-fade tile levels with `smoothstep01(L − k + 0.5)`; EF-9
  adds an importance boost to that `L(x)` (`3D_VIEWPORT_SCOPE.md` §3.3 restates
  it). **No river term is in `L(x)`**; the boost's river input
  (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.1) is channel order from
  `strahler_from_receivers`.
- **What does not exist**: EF-1's tile hydrology (`cartalith_hydrology::tile`)
  has "No non-test caller" (that document's §8.1), so zooming never adds channels
  the coarse network lacks. Everything finer than the network is the same
  network drawn wider.

### 2.4 Facts that shape the rule

1. **Network is fixed; zoom reveals, it does not create.** At every zoom the
   candidate set is the one generated network. "Zoom gains detail" (the owner's
   hard requirement, memory rule 2026-10-17; `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md`
   judged rivers at x1/x4/x16/x64 and found them "wide, uniform, pale strokes"
   from x16) must therefore be met by a visible-length that grows with zoom
   and by width and tributary structure, not by new geometry.
2. **Order and discharge disagree.** `RIVER_GATE_MEASUREMENTS.md` (Gate 1):
   on the owner world 152 mouths clear the discharge bar but are order 1 (median
   18.7 x T, 52 % order-1), against 22 that are order >= 3 only; order and
   discharge are accumulated on different trees (the `River::discharge` doc it
   cites). The comb-thinning constant's doc states the owner world's biggest
   outlet (1.5 % of its grid) is an order-1 run. **Order cannot be the key.**
3. **Width is a poor key.** `hw·ppc` at fit is about 0.5 to 1.5 cells times
   0.46 = under 1 pixel for nearly the whole network (so a "hide below 1 px
   width" rule hides everything at fit, which the floor then un-hides); and the
   slope factor decouples it from discharge. Width is an *output* of the law.
4. **Discharge is scale-free once normalised.** `Q/T` (T = `river_flow_thresh`)
   is how Gate 2 and the delta gate already speak ("multiples of T"); the
   grid-share form (`MIN_DISCHARGE_FRAC`) moves with world size, per Gate 1's
   own finding that its meaning "moves with world size and km".
5. **The field's four channels are full** (`CHANNELS = 4`, one RGBA16F texture,
   about 100 MB at the texel cap). A new per-segment datum must reuse a channel
   or cost a second texture. The B channel (order-1 weight, interpolated so that
   a tributary blends its trunk's rule) is the natural one to repurpose.
6. **River labels**: the overlay search found no river-name draw in
   `map_overlay.gd` (matches were the "river trace" and selection layers only).
   Not exhaustively searched, so treat "no river labels today" as likely, not
   proven.
7. **Picking** (`_river_pick_radius_cells`, `right_dock.gd`'s river hover,
   `get_rivers`, `MAP_CONTEXT_SCOPE.md` §11 river picks) reads the RV-2
   centreline, not the drawn stroke, by `RIVERS_IN_MAP_SCOPE.md` RIM-6's explicit
   rule. A hidden river would otherwise stay clickable.

---

## 3. A proposed rule for Cartalith

### 3.1 The quantity: discharge as a length-weighted rank `u`

Per world, once, when the river geometry is built (not per frame):

1. For every drawn segment take the discharge `Q`, made **non-decreasing
   downstream** along its run by a running maximum from the head (the same
   idiom `river_half_width_profile` uses for width; a lake bridge or a draw-plan
   extension cannot lower it). A NaN discharge stays "no reading" and takes the
   previous reading, never a plausible zero (`MISTAKES.md`: never encode "no
   value" as a plausible value); a run with no reading at all is drawn at
   `u = 0` (always visible), because hiding a river for lack of data is the
   worse failure.
2. Sort segments by `Q` descending and let `u` be the **cumulative drawn length
   of segments with discharge at least this segment's, divided by the total
   drawn length**, in `(0, 1]`. The biggest river has `u` near 0; the least
   segment `u = 1`. (Length-weighted, because Töpfer's variants and the NHD
   pruning both target a *length or density* share, and a per-segment count would
   let a swarm of 1-cell stubs dominate; the delta fans take the `u` of their
   apex's parent, §3.6.)
3. Store `u` as the **B channel**, replacing the order-1 weight. Bilinear
   filtering blends `u` across a confluence so a tributary and its trunk blend
   their rules instead of punching a hole (the property the order-1 weight was
   designed for). Half-float resolution (11 mantissa bits) is about 5e-4 over
   `0..1`, well below the fade width in §3.3.

Why rank and not `log2(Q/T)`: a rank makes the threshold a **share of the
network**, independent of world size, km per cell, river density
(`river_density` 0.30 to 3.0 moves T) and look; the shader needs no per-world
constant. A per-world sort is `O(n log n)` over the drawn segments (tens of
thousands on the owner world; to be counted in RZ-0), run only when the
geometry is rebuilt, which already happens on regeneration, river-density
change, a look change that repaints, or a sculpt.

### 3.2 The threshold: the share of the network drawn at a given density

```text
f(ppc) = min(1, (ppc / P_FULL)^p)        p = 0.5,  P_FULL = 8
alpha_v(u, ppc) = 1 − smoothstep(f, f + Δ, u)        Δ = 0.12
```

- `ppc` is the raster's screen pixels per grid cell (the same input as RIM-5,
  `LOD_DETAIL_SCOPE.md` D5 and `continuous_level`; **never a km figure**, which
  is a generation parameter).
- `p = 0.5` is **Töpfer's exponent** (`n_f = n_a sqrt(M_a/M_f)`): scale
  denominator `M` is inversely proportional to `ppc`, so the kept share is
  `sqrt(ppc/P_FULL)`. Cited [search]; its applicability to *this* network's
  length shares is a hypothesis that RZ-0 tests (the literature itself finds
  deviations).
- `P_FULL = 8` is the reference's `zk = 8`, "the LOD cap", at which the
  order-1 de-emphasis ends (`river_stroke.rs`, `O1_FADE_ZOOM`; reference v2.11
  line 9577 per its doc): the point where the old design already considered
  every stream fully "the subject". Cited, not tuned. It also places the
  progression across the owner's four judgement levels (below).
- `Δ = 0.12` of the rank is a **labelled judgement**: wide enough that a
  zoom step of about one octave never moves a stroke from fully visible to
  invisible (u of a given stroke is fixed; `f` moves by ×1.41 per ppc octave,
  so at `f` near 0.5 it moves about 0.2; the band is the cross-fade, it must be
  looked at, not assumed). RZ-1 measures the largest visible-length change
  between zoom frames.
- **Monotone and connected by construction.** `Q` is non-decreasing downstream,
  so `u` is non-increasing downstream; with `alpha_v` non-increasing in `u`, a
  river is visible from its mouth upstream to a point that moves up the
  valley as you zoom in. A stroke can lose its head, never its middle: no
  fragment floats, the topological property the pruning literature insists on.
- **Zoom gains detail at every level.** `f` is strictly increasing in `ppc`
  until `P_FULL`, so on the owner world (contain fit 0.46 ppc; so
  `ppc = 0.46 · zoom`, derived arithmetic from RIM-5's measured fit, not
  re-measured): x1 `f = 0.24`, x4 (ppc 1.84) `f = 0.48`, x16 (7.4) `f = 0.96`,
  x64 (29.4) `f = 1`. The rule adds length at x1 to x16 and the widths
  keep growing (`hw · ppc`) above that, so x64 is wider, not equal. **Above
  P_FULL the rule is the identity**, so the 1 px floor and the existing
  width law, which already grow without bound, carry "more detail" there.
  Note the opening (cover-fit) view is 0.65 ppc, `f = 0.29`; RZ-0 must show
  that this is not a map with almost no rivers (a pass bar below), because
  `P_FULL = 8` is the least-evidenced number here and the knob to turn first.
- **The floor is never violated**: a visible stroke is at least 1 px
  (`MIN_STROKE_PX`); `alpha_v` fades alpha only, never narrows below the floor.
  Alpha-fading a full-weight line reads as a lighter line; it is thinning to a
  fraction of a pixel that read dotted (the RV-2 finding).

### 3.3 No popping

Three layers, none of which is a step:

1. `alpha_v` is a `smoothstep` over a rank band (above): continuous in `ppc`
   and in `u`, zero slope at both ends.
2. The screen shader takes **continuous camera `ppc`**, as RIM-5 does; nothing
   is quantised to a tile level.
3. The tile and export rasters (§3.7) bake at their own `ppc`; the screen
   composites level tiles with the existing `morph_for_zoom` weights, which
   already cross-fade levels.

A width taper on the fade (a fading stroke also thins toward the 1 px floor)
is **not** proposed: it adds nothing the floor and alpha do not and risks the
dotted read. Labels, if rivers ever get them, open later than lines (§1.4),
at `f` at or above 0.5, as a labelled judgement.

### 3.4 What replaces what

- `o1_distance_fade` (RIM-5) and the B-channel order-1 weight are
  **retired**, replaced by `alpha_v` keyed on `u`. The reference's
  order-1 opening look (`o1_deemphasis`'s 0.55 width, 0.4 alpha) is the
  piece an owner may want to keep; proposed default: **retire its alpha half
  and keep a width de-emphasis** keyed to `u` (`wm = 0.55 + 0.45·(1 − smoothstep(0.5, 1.0, u))` at
  `ppc` 1, converging to 1 at `ppc` 8) so the opening look stays close to
  today's. The 0.5 to 1.0 band is a labelled judgement; RZ-2 compares frames.
  This is a deliberate engine/graphics change away from the reference, which the
  standing rule (memory: engine improvement priority) permits without a
  parallel path, but **it moves painted pixels**, so every pixel-asserting
  probe and look screenshot moves and needs the owner's sign-off
  (`MISTAKES.md`: pixel thresholds are palette-bound; a generated-output
  change needs a ruling, here drawing only).
- The floodplain tint's `(1 − o1)` factor becomes the visibility alpha times
  `(1 − smoothstep(0.5, 1.0, u))`: a hidden river leaves no tint, and the
  headwater class stays tint-free as before (both ends labelled judgement).
- The RIM-2 bank outline already takes the order-1 alpha ("RIM-5's fade
  removes the outline with the stream"); it takes `alpha_v` instead, same
  mechanism.
- The vector-stroke fallback (a look without `rivers_as_water`, or a grid over
  the field's texel budget) gets the same rule through `river_px_width`
  returning `None` (the seam's own contract) or `alpha_v` in its alpha, so
  that a look does not draw a different set of rivers from another
  (`river_field::painted`: "must never be answered differently by the three
  paths"). Today's fallback has no fade at all.

### 3.5 What it must never do

- **Never change the generated network.** No flow, trace, channel, `get_rivers`,
  planner navigability (`cartalith_civ::fresh_river_channels`, order 3) or
  save content may read or write the visibility value; no `golden_parity_*`
  fixture moves; the world hash is identical with the rule on and off.
  (`RIVERS_IN_MAP_SCOPE.md`: touching anything upstream of rendering is a
  scope violation.)
- Never let a stroke be narrower than 1 px or leave a gap mid-course (§3.2).
- Never answer the rule differently on screen, in tiles and in export at equal
  `ppc` (RIM-7's discipline; one function, one test per pair).
- Never read a missing discharge as zero (§3.1).
- Never hide a river that has a delta fan or reaches a visible mouth while its
  mouth is faded in (the mouth has the largest `Q`, so `u` is least there;
  asserted by a test over several worlds, RZ-2).

### 3.6 Estuaries and deltas

- **There is no estuary model to hide.** The width law caps at `9·width_k` and
  the mouth is carried into the sea by `ShoreReach`/`coast_end`; the only
  estuary-like drawn feature is the RIM-4 delta fan. (`estuaryDepth` in the
  engine is a coastal-erosion parameter of the terrain, not a drawn channel.)
  Proposal: add none; if the owner wants a tidal funnel it is a separate feature
  (OSM-style it would be a polygon, §1.4).
- **Fans**: their apex river has `own_order >= 3` and `Q >= 2e-4` of the grid, so
  it sits at small `u` and is visible long before the fan matters. Fans take
  the **apex's `u`** (so a fan never shows without its river) and in addition
  fade in by **their own on-screen reach**: `alpha_fan = smoothstep(6, 16,
  fan_reach_px)`, `fan_reach_px` the fan's maximum branch length times `ppc`.
  The 6 and 16 pixels are a **labelled judgement** (a fan of branches shorter
  than a few pixels is a smudge, not a delta); RZ-5 measures the reach
  distribution on the Part A worlds of `river_delta.rs`.
- **Braided channels and distributary networks** are not generated, so they are
  not a visibility case. Gate 1's result stands: keep today's fan gate
  (Ruling EA item 9, "Keep both").

### 3.7 Consistency across screen, tiles and export

- **One function** in `river_stroke.rs` next to `river_px_width`:
  `river_visibility(u, ppc) -> f32`, pure. The GLSL copy in `map_shore.gdshader`
  is held equal by a mirror test exactly as `the_shader_mirrors_*` do for
  `o1_distance_fade` now.
- **Screen**: continuous camera `ppc` as a uniform, the field texture unchanged
  between zoom steps (no rebuild on zoom; the field is built per world and look,
  not per view).
- **Tiles**: `PaintSource::raster` evaluates at the tile's pixels; it takes
  the tile level's `ppc`. Within one level the visibility is frozen while the
  camera `ppc` changes by up to one octave (`f` moves about 1.41x), so a stale tile
  would hold a coarser set than the screen asks for. Two ways: (a) accept it
  and let `morph_for_zoom`'s cross-fade hide the change at level boundaries,
  measuring that the step between adjacent frames stays under the bar in RZ-4; or
  (b) draw the river as the screen shader's own layer over the tiles with the
  camera `ppc`, which would remove the question but is a larger change. **Default: (a),
  decided by RZ-4's measurement.** Calling this out because it is the one place
  a naive implementation pops.
- **Export** (`export_raster.rs`, whole, banded, overlay snapshot): the export's
  own `ppc`. A whole-map export at the grid's resolution (1 px per cell) would
  draw `f = 0.35`; one at 8 px per cell and above draws everything. That is the
  Esri meaning of scale-dependent drawing (a map at a scale shows what that scale
  shows) and is **owner question Q4**.

### 3.8 Interaction with RIM, the colour field, and picking

- **RIM**: nothing is added to the field's size: B changes meaning (order-1 weight
  to `u`), the four channels and `MAX_TEXELS` are unchanged. The field is
  already a per-world product; the one new build step is the sort in §3.1.
  A river that fades also loses its floodplain and bank (§3.4), and the colour
  field (`river_color_texture`, the "map with every river at full coverage")
  needs no change: visibility multiplies coverage in the shader and in
  `pixel_paint`, it is not baked into the colour.
- **Interaction with RIM-5's constants**: `O1_GONE_PPC`/`O1_FADE_FULL_PPC` become
  dead and are removed in the same change (the engine-priority rule: scrub the
  superseded approach, no dual path), together with every comment that names
  them (`river_stroke.rs`, `river_field.rs` module docs, the shader header,
  `RIVERS_IN_MAP_SCOPE.md` RIM-5's text, `STATUS.md`'s RIM-5 entry): the prose
  that described the old behaviour (`MISTAKES.md`: change behaviour, hunt the
  prose).
- **Hit-testing**: default is that a river is pickable and hover-highlighted only
  where `alpha_v` is at least 0.5 for its `u` at the current `ppc`, so the pick
  agrees with what is drawn. That changes RIM-6's "hit-testing is kept
  regardless of what is drawn" for the zoomed-out case and is **owner
  question Q5**; the Rust side exposes `river_visible_at(run, ppc)` and the
  GDScript pick calls it.
- **Colour paint (Ruling CN)**: its marks composite after the river (the
  river is part of the map's colour before paint); visibility does not interact.

### 3.9 Performance

- Per-pixel cost: one `smoothstep` and a multiply where RIM-5 already does one;
  no new texture fetch; **no field rebuild on zoom** (the rule is a function of
  a uniform). Bar below.
- Build cost: one sort of `n` segments and a prefix sum (§3.1), once per
  geometry build. RZ-0 counts `n` on the owner world and measures the sort.
- Tiles and export: one more multiply in `pixel_paint`; tiles bake at their own
  level so no tile is re-baked on zoom within a level (§3.7).
- Ruling BP's interactive bar (16.7 ms) and the rivers-on baseline of 19 to
  28 ms median that `RIVERS_IN_MAP_SCOPE.md` quotes for RV-2 are the bars to
  hold: the change must not regress them. (That baseline is another
  session's figure; RZ-0 re-measures it.)

---

## 4. Milestones

Numbered `RZ-`. Every bar is a proposal; RZ-0 may move numbers and says so.

- **RZ-0 — measure before pinning (no code in the app).** An `#[ignore]`d
  harness beside `river_gate_measurement.rs` (same twelve seeds and shapes,
  plus the owner world 246371 at 2048x1311, density 1.55), run alone in release,
  median with min..max (`MISTAKES.md`). Report: (i) `n` drawn segments and total
  length per world; (ii) the share of total length that is order 1, and what
  share of the **top-1 % and top-10 % discharge length** is order 1 (the
  prediction of §2.4 item 2 is that it is not zero); (iii) for each of
  `f` = 0.1, 0.24, 0.5, 0.96, the **number of connected components and the
  visible length** of the kept network under rank-by-`u`, under order-only
  (>= 2, >= 3), and under `Q/T` thresholds, so the three keys are compared on
  one table; (iv) `f(0.46)`, `f(0.65)`, the opening view, in drawn pixels;
  (v) the sort's cost. **Pass bars:** the harness runs and prints non-empty
  tables (`MISTAKES.md`: assert non-emptiness, silently-empty golden output); the
  rank rule keeps the top-10 % discharge outlets visible at `f = 0.24` on every
  world (no top-decile outlet lost, **the bar the order key fails**); and
  `P_FULL` is confirmed or moved with the table beside it. Output: a section
  appended to this file.
- **RZ-1 — the function and its mirrors.** `river_visibility(u, ppc)`,
  the shader copy, `river_field::coverage` taking it. Bars: continuity (|Δalpha|
  per 1 % ppc step under 0.05 over `ppc` 0.1 to 40); `f` strictly increasing
  below `P_FULL` and exactly 1 at and above it; alpha 1 for `u = 1` at
  `f = 1`; mutation of `p`, `P_FULL` and `Δ` (a test asserting literals,
  not the constants against themselves) goes red; the mirror test holds
  shader and Rust equal. No generation golden moves.
- **RZ-2 — the rank into the field; retire RIM-5.** B carries `u`; deltas
  take the apex's `u`; floodplain and bank take `alpha_v`. Bars: **world hash
  and every `golden_parity_*` unchanged**; `u` non-increasing downstream on
  every run of every test world; zero broken-middle strokes
  (counted by the connected-component check on the drawn set at 8 `ppc`
  values: components at `f` never more than at `f = 1` plus a tolerance of the
  fan count); a delta-bearing mouth is always visible whenever its fan is.
  Probe update: `_riverzoom_probe.gd` (extends its `--rim5` leg), `_rivcolour_probe.gd`;
  both run to the end and exit non-zero on failure (the 2026-10-18 exit-code
  fix).
- **RZ-3 — the zoom ladder, judged by eye and by count.** At x1, x4, x16 and
  x64 on the owner world and two other seeds, in two looks (default painted and
  one ink look) and Night: frame pairs before/after and a table of visible drawn
  length and tributary count. **Bars (the owner's hard requirement):** drawn
  length at x4 at least 1.5 times x1 and at x16 at least 1.3 times x4
  (non-decreasing at every level, strictly increasing to `P_FULL`); at x16 and
  above no previously visible river is gone; at x1 the top-decile discharge
  rivers are all drawn at full alpha; no frame shows an isolated fragment of
  fewer than 3 cells. The owner reviews the pairs: **a measurement cannot
  grade "reads as a map" and nothing here claims it does.**
- **RZ-4 — tiles and export.** `PaintSource::raster` and `pixel_paint` take the
  rule; export uses its own `ppc`. Bars: for a swept `ppc`, screen texture,
  tile and export agree on `alpha_v` at the same `ppc` to the half-float
  tolerance; in a continuous zoom of 2 octaves across a tile-level switch
  (including `LOD_AUTO_ZOOM` 2.2, about 1.0 ppc on the owner world) the
  **visible-length change between adjacent frames stays at or below 3 % of total
  drawn length** (labelled judgement; the "pops stay at zero" bar of
  `LOD_DETAIL_SCOPE.md` D5 is the intent); if it does not, switch to option (b)
  of §3.7. Export bars: a 16K export draws `f = 1` (no river lost against
  today at equal `ppc`); a grid-resolution export draws `f(1)`.
- **RZ-5 — fans.** Fan fade by reach (§3.6). Bars: no fan is drawn whose longest
  branch is under 6 px; a fan reaches full alpha at 16 px; `river_delta` tests
  unchanged for the off switch (`set_river_deltas`): byte-identical output
  with fans off, as its module doc already guarantees.
- **RZ-6 — picks.** `river_visible_at` and the GDScript pick/hover (per Q5).
  Bars: a probe asserts that every river the picker returns at `ppc` has
  `alpha_v >= 0.5` there, and that none drawn at `alpha_v >= 0.5` is
  unpickable; hit-radius behaviour above `P_FULL` identical to today.
- **RZ-7 — performance and close-out.** Bars: median frame with rivers on at
  each of the probe's zooms (x1 to x240, `_zooms`) within the run-to-run spread
  of the pre-change build on the same machine, and under Ruling BP's 16.7 ms
  where today's passes; the sort under 50 ms on the owner world at build
  (labelled judgement, RZ-0 may move it); `cargo test --workspace`, parse-check
  of every touched `.gd` plus `shell/app.gd`, and the `STATUS.md` entry in the
  same change. Remove the dead RIM-5 constants and their prose.

---

## 5. Owner questions, with defaults

1. **Which quantity drives visibility?** Default: **discharge, as a length-weighted
   rank** (§3.1), because order ranks the owner world's biggest outlet with the
   trickles. Alternatives: order (rejected: §2.4 item 2), on-screen width
   (rejected: §2.4 item 3).
2. **How sparse may the zoomed-out map be?** `f(0.65)` is about 0.29 at the opening
   view with `P_FULL = 8`. Default: accept `P_FULL = 8` (the reference's own cap)
   subject to RZ-0's table; if the opening view looks too bare, lower `P_FULL`
   to 4 (`f` 0.40 at the opening view, full at x8.7) before changing `p`.
3. **Is the Töpfer exponent 0.5 right?** Default yes (cited, and consistent with
   what the generalisation literature found, with exceptions). An owner may prefer
   a steeper curve (more rivers earlier); RZ-0's table is the evidence.
4. **Do exports follow the rule at their own density?** Default **yes**: a map
   shows what its scale shows. The alternative is "an export always draws the
   full network" (the legacy HTML's export shows every river); say so if the
   grid-resolution export should keep today's look.
5. **Should a hidden river be unpickable?** Default **yes** (pick what you see);
   this narrows RIM-6's "hit-testing is kept regardless of what is drawn".
6. **Retire the order-1 opening look's alpha, keep a width de-emphasis?**
   Default yes (§3.4). Alternative: keep the whole reference look and apply the
   rank rule on top, at the cost of two overlapping rules.
7. **Delta fans fade in by on-screen reach (6 to 16 px)?** Default yes; the
   alternative is "a fan follows its river's alpha only".
8. **A tidal estuary funnel or braided channels?** Default **no**: not generated,
   so nothing to hide; if wanted, a separate feature with its own scope
   (a polygon, as OSM does).
9. **Does the vector-stroke fallback follow the same rule?** Default yes, so
   looks cannot differ in which rivers they show.
10. **River labels** (if the owner wants them): lines first, labels at `f` at or
    above 0.5. Default: out of scope here.

---

## Sources

Fetched this session: OpenStreetMap Carto `style/water.mss`
(<https://raw.githubusercontent.com/gravitystorm/openstreetmap-carto/master/style/water.mss>);
OpenMapTiles schema, `waterway` (<https://openmaptiles.org/schema/#waterway>);
Mapbox Streets v8 reference
(<https://docs.mapbox.com/data/tilesets/reference/mapbox-streets-v8/>);
Natural Earth 10m rivers and lake centerlines
(<https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-rivers-lake-centerlines/>).
From search summaries only (pages not opened): Töpfer's radical law and hydrography
applications (<https://www.isprs.org/proceedings/xxxviii/part4/files/Wilmer.pdf>,
<https://arxiv.org/pdf/1102.1561>); Stanislawski, "Feature pruning by upstream
drainage area to support automated generalization of the United States
National Hydrography Dataset"
(<https://www.usgs.gov/publications/feature-pruning-upstream-drainage-area-support-automated-generalization-united-states>;
DOI 10.1016/j.compenvurbsys.2009.07.004, to be checked before any figure is
reused); the ICA generalisation workshop papers
(<https://kartographie.geo.tu-dresden.de/downloads/ica-gen/workshop2010/genemr2010_submission_11.pdf>,
<https://kartographie.geo.tu-dresden.de/downloads/ica-gen/workshop2013/genemappro2013_submission_15.pdf>);
ArcGIS Pro, display layers at certain scales
(<https://doc.esri.com/en/arcgis-pro/latest/help/mapping/layer-properties/display-layers-at-certain-scales.html>);
Azgaar's Fantasy Map Generator
(<https://github.com/Azgaar/Fantasy-Map-Generator>, and the CSS gist in the
search results).
Not found: Google Maps, Wonderdraft, Inkarnate, Dungeondraft per-zoom river rules;
swisstopo river selection rules; NHD display thresholds as rendered.
