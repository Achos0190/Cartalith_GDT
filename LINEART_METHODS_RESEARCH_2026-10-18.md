# Line art to proper assets: methods research (2026-10-18)

**Research, not a scope. It carries no status column**; status lives in
`cartalith-native/docs/STATUS.md`. The recommendation and the milestone it
proposes (AP-10) are in `ASSET_PACK_TOOLS_PROPOSAL.md` §9. This file holds
the data.

**The owner's request (2026-10-18):** *"from the earlier shared art craft
github maybe you can find better methods to use for creating asset packs and
refining images for the asset packs"*, and *"be conscious about what might be
needed to convert LINE ART to proper assets. In the html version the lines
weren't solid enough and often it became a blobby mess."*

**Which repository.** "Art craft" is PhotoCraft,
`github.com/storytold/photocraft`. Its `NOTICE` names the ArtCraft Team as
the copyright holder, and it is the same repository `EDITING_TOOLS_SCOPE.md`
cites 21 times. Nothing points to a different repository.

---

## 1. Why line art goes wrong today (read at the symbols)

A pack sprite goes through four steps before it reaches the screen. Each step
was read in the code on 2026-10-18:

1. **Background removal.** `cartalith_assets::slicer::apply_chroma` is the
   reference's `applyChroma` (v2.11 line 28251). It keys out every pixel
   within `tol` of one sampled paper colour. Alpha is either 0 or 255. Pixels
   outside the tolerance keep their RGB at full opacity, and that includes
   the light anti-aliased edge of every stroke and all paper that is darker
   than the sample.
2. **The bake.** `raster::render_item` resizes to the family's 256² with
   `FilterType::CatmullRom` on straight (not premultiplied) RGBA. The
   reference does the same with `drawImage` at `imageSmoothingQuality:'high'`
   (`drawItemOnly`, v2.11 line 27403).
3. **The blit.** `pack::composite_map_icons` calls `Canvas::blit_sprite`,
   which does **one bilinear sample per destination pixel**
   (`sample_bilinear`) into the grid-resolution RGB8 buffer. The rectangle
   comes from `placement::sprite_draw_rect`: `dh = base × 2.2 × s` with
   `base = max(gw/110, 3.5)`. On a 2048-wide grid with `s = 1` that is
   **40.96 px tall**, and the default rule's `s` range of 0.7–1.2
   (`scatter.rs` defaults) gives 28.7–49.2 px. A 256² sprite is therefore
   minified 5–9×. Bilinear point sampling at that ratio reads only about one
   source pixel in six to eighty, so a 1–2 px stroke is hit or missed by
   chance. That is where lines break up.
4. **Zoom.** The base map is drawn by `viewport_host.gd::_raster` with
   `TEXTURE_FILTER_NEAREST`, so ×16 turns each icon pixel into a 16 px block.
   No deep-zoom tile path draws icons (`ASSET_PACK_TOOLS_PROPOSAL.md` §1.3).

**What the lab measured for each step** (a scratch replica of steps 1–4 in
Python, on the samples in §2, at ×1; `iso.py`'s numbers):

- **Ink kept (ink mass ÷ truth):** 0.31–0.45 for today's chain.
- **Replacing only the point-sampled blit** with a premultiplied area filter
  of the same 256² bake raises that to 0.55–0.81.
- **Filtering the 1024² original** instead of the 256² bake raises it to
  0.90–1.01 on the three clean samples, but only 0.60 on the scan, where the
  key's lost ink also counts.
- **The paper halo on the scan** stays at 19–22 % of the frame in all three
  arms, so it is caused by the chroma key, not by the filter.

So the "not solid enough" lines have two named causes, the point-sampled blit
and the straight-alpha 256² bake. The "blobby mess" has two more: opaque paper
left behind by a single-colour key, and nearest-neighbour magnification of a
40 px raster.

---

## 2. Test set

Four synthetic samples were drawn as vector strokes in normalised
coordinates, so the truth can be rendered exactly at any output size. Each
was perturbed by a smoothed random walk so it reads as hand-drawn, and each
was rasterised to a 1024² "scan", which is what an author would import. No
third-party art was used. The seeds and code are in
`cartalith-native/docs/evidence/lineart-2026-10-18/lineart_lab.py`.

| Sample | What it tests | Source condition |
|---|---|---|
| `pine` | 19 thin strokes, 6 px at 1024 (0.24 px at map scale) | black on white, saved as JPEG at quality 80 (an AI-style export) |
| `castle` | outlines, crenellations, small windows (enclosed holes), a solid filled door, and a dashed line of 4.9 px strokes | clean PNG |
| `mountain` | ridge line plus 14 hatch strokes 4 px wide, about 20 px apart | clean PNG |
| `scan` | the castle as a bad scan: yellowed paper, a 28 % illumination fall-off, grain (σ 6), 60 dust specks, ink starvation (strokes thinned to 35 % in places), a 0.9 px scanner blur, JPEG at quality 35 | the "blobby mess" input |

**Output scales.** The sprite is 40 px tall on the map (×1, the 2048-grid
`dh` above), then 160, 640 and 2560 px (×4, ×16, ×64).

**The target** is the vector truth at that size with every stroke floored to
1.0 output pixel. That is the owner's "solid lines": the art as drawn, but no
line thinner than one pixel.

---

## 3. Metrics (all produced by the lab today)

Each method's output is composited over a map green. Effective ink is how far
a pixel moves from the green towards the ink colour, on a 0–1 scale.

- **Components** and **holes:** 8-connected parts of the ink at alpha > 0.5,
  and enclosed paper regions. Fewer holes than the target means gaps were
  filled, which is a blob merge. More components means lines broke.
- **Solidity:** the mean effective ink over the target's ink pixels. 1.0 is
  fully solid.
- **Stroke width p5 / median:** `2·edt − 1` at the medial ridge, in output
  pixels.
- **Edge width:** the number of pixels with 0.1 < alpha < 0.9, divided by the
  boundary length. This is the transition width in pixels; about 1 means a
  crisp anti-aliased edge.
- **Halo:** the share of the frame that the sprite makes *lighter* than the
  ground by more than 20 luma, which is paper left opaque.
- **Ink mass:** total effective ink divided by the un-floored truth's
  coverage. 1.0 is a faithful tone; above 1 is darkening.

---

## 4. Results

### 4.1 Background removal and binarisation (1024² source, before any resize)

IoU and component count against the truth at alpha > 0.5. The truth has 7
components on the castle and the scan.

| Method | pine | castle | mountain | **scan** |
|---|---|---|---|---|
| chroma key, tol 40 (**today**) | 0.874 | 0.918 | 0.876 | **0.267, 577 comp.** |
| Otsu on raw luminance (Otsu 1979) | 0.980 | 1.000 | 1.000 | 0.952, 62 comp. |
| Bradley, flattened paper (Bradley & Roth 2007) | 0.923 | 0.849 | 0.927 | 0.824, 66 comp. |
| Sauvola, flattened paper (Sauvola & Pietikäinen 2000) | 0.925 | 0.753 | 0.926 | 0.778, 70 comp. |
| **flatten → Otsu → despeckle (< 24 px) → pinhole fill (< 12 px)** | 0.980 | 1.000 | 1.000 | **0.961, 7 comp.** |
| same, plus closing r = 1 | 0.978 | 1.000 | 0.999 | 0.961, 7 comp. |
| same, plus closing r = 2 | 0.976 | 1.000 | 0.994, **13 comp. (a hatch merge)** | 0.960, 7 comp. |

**Flattening** means dividing luminance by a local paper estimate: a maximum
filter, then a mean filter, over a window of 15 % of the sprite.

**Finding 1 — the paper window must be wider than the largest solid ink
area.** The first run used a 25 px window. It read the castle's 123 px solid
door as paper and hollowed it out; a draft figure showed it and was not kept. The 15 % window fixed it: castle IoU went from 0.807 to 1.000.

**Finding 2 — local thresholding was worse than flattening plus Otsu on every
sample.** Once the illumination is flattened, Sauvola and Bradley only erode
the anti-aliased edge. They are kept as a fallback for a strongly uneven page
and are not the default. This is a measurement on four synthetic samples;
real scans may differ (owner question L2).

**Finding 3 — closing with r = 2 merged two hatch lines** on the mountain.
Closing stays at r ≤ 1 and is off for clean art.

### 4.2 Sprite methods at four zooms

Measured; the full table is `results.json`. Target values follow each slash.

| Method | ×1 solidity | ×1 ink mass | ×16 / ×64 components | ×16 / ×64 holes | ×16 edge | ×64 edge | halo |
|---|---|---|---|---|---|---|---|
| **today** (key → 256² CatmullRom → point blit → nearest zoom) | 0.09–0.15 | 0.31–0.45 | pine 15/1, castle 4/7, mtn 16/14, scan 4/7 | castle **0/14**, scan **0/15** (every window filled) | 5.8–19.6 px | 22–77 px | scan **22 %** |
| coverage raster (local colour-to-alpha → area filter of 1024²) | 0.20–0.35 (faint) | 0.99–1.23 | exact on clean art; **scan 55/7 and 61/7** (noise kept) | exact | 0.70–1.77 | 2.48–6.63 | 0 |
| **SDF + stroke floor** (cleaned mask → SDF 128² for ×1 and 512² for ×4 and up) | **0.72–0.75** | 2.7–4.4 (darker) | **exact on all four** | castle 14/14, scan 14/15, others exact | **0.67–0.79** | **0.55–0.75** | 0 |
| medial-axis field (centreline distance + radius), 256² | 0.62–0.73 | — | mountain **164/14 and 168/14**; the solid door is reconstructed wrongly on the castle and the scan | castle 14/14 | 1.18–2.10 | 1.11–5.58 | 0 |

**SDF at ×4** (512²): components pine 1/6, castle 7/11, mountain 13/13 and
scan 7/9. Holes: castle 14/14, scan 14/14, mountain 2/0, pine 0/0. Edges are
0.88–1.35 px. ×4 has fewer components than the target and never fewer holes
than it, so strokes that touch were bridged and no gap was filled.

**SDF storage resolution** (×64, castle / mountain, components against 7/14):

| Stored size | castle | mountain |
|---|---|---|
| 64² | 9 | 84 |
| 128² | 15 | 98 |
| 256² | 7 | 15 |
| 512² | 7 | 14 |

**Finding 4 — a single-channel SDF cannot represent a stroke narrower than
about two texels.** The mountain's 4 px hatching is 1 texel at 256² and
breaks there. It is 2 texels at 512² and holds. The rule is therefore that
the stored SDF has at least two texels across the asset's thinnest stroke,
measured at import. See `sdf_resolution_x64_*.png`.

**Finding 5 — sample an SDF at no more than about 4 texels per output
pixel.** The 512² SDF read at ×1 (12.8 texels per pixel) aliases, with edges
of 4.6–8.6 px. The 128² level at ×1 (3.2 per pixel) gives 1.42–1.83. So the
SDF needs mip levels that are *built from the mask*, not averaged from the
level above.

**Finding 6 — the stroke floor has a cost at map scale.** The floor is a
uniform offset, `δ = max(0, (1 − w_p2) / 2)` output pixels, where `w_p2` is
the asset's 2nd-percentile stroke width; it was 0.40–0.44 px at ×1. It makes
every line solid (p5 width 1.00 px) but darkens the tone 2.7–4.4×.

- Hatching spaced closer than about 2 output pixels is closed into a solid
  dark band at ×1. This is visible in `grid_mountain.png`. No method can
  resolve 1 px-spaced hatching at 40 px; the only choice is between a faint
  tone and a solid one.
- A **gap-aware floor** was tried: a second channel holds the local paper-gap
  width, and the offset is capped so it never closes a gap. In this form it
  **failed**: ×1 components went to 12–29 against 1–8, and holes to 0. Its
  gap estimate is wrong near corners. It is recorded, not recommended.
- The 10th-percentile floor of the first run missed the castle's dashed line
  at ×4 (3 components against 11). The 2nd percentile caught it (7 against
  11).

**Finding 7 — re-stroking along the skeleton loses to the plain SDF.** Two
things go wrong:

- The nearest-centreline field is discontinuous where the centrelines meet,
  and interpolation breaks the strokes there: 164 components against 14 on
  the mountain at ×16.
- Filled areas reconstruct wrongly: the door in `grid_castle.png` and `grid_scan.png` shows it.

It survives only as an optional "even line weight" edit at source resolution
(Zhang & Suen 1984 thinning). It is not a storage format.

**File size** (PNG, best compression):

| Stored as | Range |
|---|---|
| today's 256² RGBA bake | 15.9–51.8 KB |
| L8 SDF 512² | 10.8–21.0 KB |
| L8 SDF 128² | 3.1–3.9 KB |
| 1024² coverage (L8) | 8.1–156 KB (the scan keeps its grain) |

**Runtime.** Python 3.11 with numpy and scipy, one process, while another
agent was compiling Rust. These are orders of magnitude only, not
benchmarks:

- cleaning a 1024² sprite: tens of ms;
- building a 128² SDF (an exact EDT of a 2048² supersample): about 0.5 s;
- rendering a 2560² frame by bilinear SDF lookup in Python: 0.1–0.25 s.

In the app the lookup is one texture fetch and a clamp per fragment in a
canvas shader. That was not measured.

### 4.3 What was not tested

- **Bitmap-to-Bézier tracing** (potrace-like) was not run. Potrace is GPL-2.0+
  (Selinger), so it cannot be linked into this MIT/Apache-2.0 workspace.
  PhotoCraft's `trace.rs` is a permissively licensed alternative: marching
  squares at level 0.5, then Schneider's least-squares Bézier fit. Tracing
  would remove the SDF's resolution limit, but only after a curve-fitting
  tolerance has been chosen. That makes it a candidate for an SVG export, not
  for the map path.
- **MSDF** (Chlumský 2015; `msdfgen` is MIT) keeps sharp corners. The
  single-channel SDF rounds corners by about one texel, which is 5 px at ×64
  from 512². The crenellations in `grid_castle.png` show the rounding is
  slight. MSDF was not measured.
- **Matting of coloured, non-line art.** PhotoCraft has it as `matting.rs`, a
  guided filter followed by colour decontamination. It is out of this test's
  scope. Line art is one ink colour, and that colour is replaced (§5), so
  there is nothing to decontaminate.
- **Real scans.** Every sample is synthetic.

---

## 5. Colour and halo

- **Today's chain leaves 19–22 % of the scan's frame as opaque paper.** The
  chroma key's tolerance cannot follow an illumination gradient.
- **Every mask-based method leaves zero halo** (§4.2), because no paper RGB
  survives. Alpha comes from the ink mask, and RGB is one ink colour.
- **The ink colour can be set from the Look's palette** and baked into the
  new variant's pixels, which is Ruling CQ Q4. That is both the "colourise
  ink to the palette" step and the paper removal: paper has no colour left
  to carry.
- **Blending stays in sRGB, as Godot's 2D canvas does.** Blending in linear
  light makes thin dark lines on a light ground look thinner. FreeType's
  stem darkening exists to compensate for exactly that. The stroke floor
  above is this project's equivalent of stem darkening.

---

## 6. PhotoCraft against Cartalith

**Licence.** PhotoCraft's code is "MIT OR Apache-2.0" (`LICENSE-MIT`,
`LICENSE-APACHE`, `NOTICE`, workspace `Cargo.toml`). Cartalith's workspace is
also `MIT OR Apache-2.0` (`cartalith-native/Cargo.toml`). Code may therefore
be reused **with the copyright and licence notice kept**.

Some material is excluded or under other terms:

- The ArtCraft name and logos are trademarks and are **not** covered by that
  licence (`NOTICE`).
- The bundled Inter and JetBrains Mono fonts are OFL-1.1.
- The bundled icons are Lucide, under ISC.
- The face-detection weights are BSD-3-Clause.

`ATTRIBUTION.md` lists each of these. No art is proposed for reuse.
`EDITING_TOOLS_SCOPE.md` §9 item 5 ("PhotoCraft is read as documentation
only") still stands as this project's choice: the methods below are cited as
designs, and any code taken must keep its notice.

The repository was read as a shallow clone of commit `9185b64b`
(2026-10-18) in a scratch folder, which was deleted afterwards. The symbols
below are from that commit.

| Need | PhotoCraft (file, symbol) | What it does | Cartalith today |
|---|---|---|---|
| Ink from paper | `crates/algo/src/color_to_alpha.rs`: `opacity`, `remap` | GIMP-style unmixing against a key colour: `a = max_i` of the per-channel distance ratio, then transparency/opacity thresholds. Exact round trip | `slicer::apply_chroma`, a binary key. The lab's `c2a_alpha` applies PhotoCraft's formula to a **local** paper estimate |
| Threshold | `crates/compose/src/adjust.rs`: `Adjustment::Threshold`, `threshold_white` (a global level); Otsu in `crates/algo/src/scancrop.rs` | a global threshold; Otsu with close/open and 8-connected components for scan cropping | nothing in `cartalith-assets`. `adjust.rs` (ET-C1) has Levels, Curves, OkLCh HSL and a gradient map |
| Morphology, despeckle | `crates/algo/src/other.rs` (Minimum, Maximum); `noise.rs` (`median`, Dust & Scratches); `scancrop.rs` (close/open, component filter) | grey-level min/max, and median with a threshold | none |
| Exact distance transform | `crates/algo/src/selection/distance.rs` (Felzenszwalb & Huttenlocher 2012); `crates/compose/src/effects.rs` (a 1D F&H pass, a 5×5 chamfer, and `edge_offset`: **the sub-pixel signed distance of an anti-aliased pixel to its edge**) | an exact EDT; layer-style strokes and glows built from distance | `cartalith_civ::build_coast_sdf`, a **jump flood**, which is exact only one seed away (`MISTAKES.md`, "Write an oracle") |
| Bitmap → path | `crates/vector/src/trace.rs`: `trace_mask`, `contours`, `fit_trace` | marching squares at 0.5, corners at > 60°, then the Schneider 1990 Bézier fit | none |
| Path → pixels | `crates/vector/src/raster.rs` (exact-area scanline coverage); `stroke.rs` | analytic anti-aliasing with no supersampling | `pack.rs` `Canvas::fill_triangle`, a pixel-centre point test (aliased) |
| Resampling | `crates/algo/src/resample.rs`: `Resample` (Nearest, Bilinear, Photoshop-measured Bicubic, Smoother, Sharper, Automatic, Lanczos, Preserve Details) | **premultiplies before filtering** (`premultiply` / `unpremultiply`); widens the filter when reducing, and pre-averages below ¼ | `render_item` uses CatmullRom on straight alpha; `sample_bilinear` is a point sample on straight alpha |
| Edge refinement | `crates/algo/src/matting.rs` (guided filter, then decontaminate colours) | soft edges, and fringe colours replaced by nearby opaque colour | none |
| Denoise, sharpen | `denoise.rs` (guided filter); `sharpen.rs` (unsharp mask, Smart Sharpen, High Pass) | | none for assets |
| Adjustment stack | `crates/compose/src/adjust.rs` (in `compose/src/lib.rs`'s layer stack) | Photoshop-style adjustment layers | `cartalith-godot/src/adjust.rs` (ET-C1/C2) |
| Brush and colour management | the `paint` and `cms` crates | already read for ET-D and ET-E | `EDITING_TOOLS_SCOPE.md` |

**Searched for and not found** (`grep -rIi` over `crates/`):

- Sauvola, Bradley and Niblack (the only threshold-method hits were Otsu in
  `scancrop.rs` and `segment/subject.rs`);
- skeletonisation or thinning;
- SDF or MSDF *output* as an asset format;
- potrace.

**PhotoCraft is an image editor and has no asset or sprite pipeline.** What
it offers this work is building blocks: colour-to-alpha, an exact EDT, an
anti-aliased signed-distance seed, premultiplied resampling, an exact-area
rasteriser and a tracer.

---

## 7. Sources

Pages fetched or searched today:

1. C. Green, *Improved Alpha-Tested Magnification for Vector Textures and
   Special Effects*, SIGGRAPH 2007 course (Valve). One-channel distance
   fields thresholded at render time.
   https://cdn.akamai.steamstatic.com/apps/valve/2007/SIGGRAPH2007_AlphaTestedMagnification.pdf
   (located by search; the search summary was read, the PDF was not opened).
2. V. Chlumský, *Shape Decomposition for Multi-channel Distance Fields*
   (thesis, 2015), and `msdfgen`, MIT. https://github.com/Chlumsky/msdfgen
   (search summary).
3. P. Selinger, *Potrace*, GPL-2.0-or-later. https://potrace.sourceforge.net/README
   (search summary).
4. FreeType, stem darkening (`ft2-properties`): emboldening at small sizes to
   offset gamma-correct thinning.
   https://freetype.org/freetype2/docs/reference/ft2-properties.html (search
   summary).
5. PhotoCraft at `9185b64b`, https://github.com/storytold/photocraft (cloned
   and read at the symbols in §6).

Bibliographic references, not fetched today; these are the methods the lab
implemented:

6. N. Otsu, *A Threshold Selection Method from Gray-Level Histograms*, IEEE
   Trans. SMC 9(1), 1979.
7. J. Sauvola, M. Pietikäinen, *Adaptive document image binarization*,
   Pattern Recognition 33(2), 2000.
8. D. Bradley, G. Roth, *Adaptive Thresholding Using the Integral Image*,
   J. Graphics Tools 12(2), 2007.
9. P. Felzenszwalb, D. Huttenlocher, *Distance Transforms of Sampled
   Functions*, Theory of Computing 8, 2012.
10. T. Y. Zhang, C. Y. Suen, *A fast parallel algorithm for thinning digital
    patterns*, CACM 27(3), 1984.

---

## 8. Evidence

All of it is in `cartalith-native/docs/evidence/lineart-2026-10-18/`, and
every image was opened after it was written.

| File | Shows |
|---|---|
| `sources.png` | the four 1024² inputs |
| `grid_pine.png`, `grid_castle.png`, `grid_mountain.png`, `grid_scan.png` | rows: today, the coverage raster, SDF + floor, the medial-axis field. Columns: ×1 (40 px, shown 5×), ×4, and 200 px crops at ×16 and ×64, over map green |
| `binarise_scan.png` | 300 px crops of the scan: the input, today's key, raw Otsu, Sauvola, and flatten + Otsu + despeckle + close |
| `sdf_resolution_x64_castle.png`, `sdf_resolution_x64_mountain.png` | the SDF stored at 64², 128² and 256², read at ×64 |
| `results.json` | every number above |
| `lineart_lab.py`, `lineart_figs.py` | the generator and the figures. Run with `py -3.11 -I lineart_lab.py <out>`, then `py -3.11 -I lineart_figs.py <out> <figdir>`; it needs numpy, scipy and Pillow. It is deterministic: fixed seeds, and no timing enters a figure |

The §1 attribution arms (`iso.py`) were a scratch script that reused
`lineart_lab.py`'s functions. It was deleted with the scratch folder, and its
three numbers per sample are quoted in §1.
