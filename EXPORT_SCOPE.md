# High-resolution image export — findings, **shelved 2026-08-25**

> **Shelved at the owner's request, 2026-08-25**, the same day it was raised:
> *"Let's shelve the 16k export and higher for the moment."* Nothing in this
> document is scheduled work. It exists so that the day it is un-shelved, the
> expensive parts do not have to be established a second time.
>
> **The shipped 2K/4K/8K export was left untouched.** Everything described below
> as "prototyped" was written, run, and then reverted; the pass left the working
> tree exactly as it found it. Test suite before and after: **139 binaries,
> 2 254 passed, 0 failed, 8 ignored.**
>
> **This document defines what a 16K/32K export would be — the findings, the
> constraints, the codec survey and the five milestones §7 sets out. It does not
> track them.** The shelving is an owner decision and stays; anything about where
> work stands belongs to **`cartalith-native/docs/STATUS.md`**, which is the only
> place progress is recorded.

## What was asked for

Owner, 2026-08-25:

> "I'd like an option to export the image at a higher res. Like for example I
> want a png at 16k or even 32k. With this export function should also be
> options on what to include on the to be generated image, settlements, routes
> what layers and styles (most commonly the standard style from the carto
> layer, but a user might want their own."

Clarified the same day, and the clarification matters because it removes a
whole axis:

> "Let's be clear the export would just be one image. Not LOD not zoomable.
> Just a single picture."

So: **one flat file**, opened in Photoshop or sent to a printer. Not a tile
pyramid, not a manifest. The existing `export_raster_png(path, width, tiled =
true)` tiled mode is a *file layout* that already ships and should keep
working, but it is not the feature and must never become the only route to a
large export.

And, on the file itself:

> "So maybe we should use a proper image codec to keep a image of that size and
> resolution and sharp and low in size. Can you find a suitable codec?"

---

## 1. The single most valuable finding: the reference's bake draws terrain only

**Established by reading `reference/Cartalith Gen1 v2.10.html` directly**, not
inferred.

`bakeSingle(W, onP)` (line 11975) and `bakeTiled(W, onP)` (line 11982) differ
only in which rectangles they ask for. Both walk output pixels and call exactly
one function per pixel:

```js
const c = bakePixel(x*sx, gy), p = (yy*w+x)*4;
d[p]=c[0]; d[p+1]=c[1]; d[p+2]=c[2]; d[p+3]=255;
```

`bakePixel(gx, gy)` (line 11931) is the material path and nothing else: height,
temperature, rainfall, the three prologue fields `buildGridFields` precomputes,
sea colour, lake colour, `landColorCore`, crest, and the coast/river SDF bands.
It touches **no** settlement, route, way, label, territory or icon data. There
is no second pass over the canvas afterwards — `bakeSingle` returns
`oc.toBlob(...)` immediately after its strip loop, and `bakeTiled` pushes each
tile's bytes straight into its entry list.

`exportZip` (12465-12466) writes `map.png` from `bakeSingle` and `tiles/` from
`bakeTiled`, so **the archive's picture is terrain-only too**.

### What that decides

The overlay half of the owner's request is **new work, not a parity port**.
There is no golden target for "a settlement drawn into an exported image",
because the reference has never drawn one. The two honest options are:

- **(a) Render the overlays in Godot at export scale and composite**, keeping
  one source of truth for how a settlement, a route and a label look — the same
  `map_overlay.gd` that draws them on screen.
- **(b) Reimplement the overlay drawing in Rust**, which duplicates
  `map_overlay.gd` and will drift from what is on screen the first time either
  side is touched.

**(a) is right**, and this pass found a concrete constraint that shapes how it
has to be built (§5).

---

## 2. The four gaps, as measured

All four are in
`cartalith-native/crates/cartalith-godot/src/export_raster.rs`.

### 2.1 The width ceiling

```rust
const BAKE_WIDTHS: [i64; 3] = [2048, 4096, 8192];   // line 42
```

`export_raster_png` refuses anything not in that array rather than rounding —
deliberately, and the doc comment says why. 8192 is the ceiling.

### 2.2 The whole raster is held in RAM

```rust
const PEAK_BYTES_PER_PIXEL: u64 = 3 + 12;   // line 51
```

3 bytes for the RGB8 raster, plus 12 for `apply_local_contrast`'s luma and its
two `f32` blur buffers. Height comes from `render::bake_dims` =
`round(W · GH / GW)`, so at the app's own 2048 × 1311 grid:

| width | height | pixels | raw RGB8 | **peak at 15 B/px** |
|---|---|---|---|---|
| 2 048 | 1 311 | 2.7 M | 8 MB | ~40 MB |
| 4 096 | 2 622 | 10.7 M | 32 MB | ~161 MB |
| 8 192 | 5 244 | 43.0 M | 129 MB | **~645 MB** |
| 16 384 | 10 488 | 171.8 M | 515 MB | **~2.6 GB** |
| 32 768 | 20 976 | 687.4 M | 2.06 GB | **~10.3 GB** |

**This is not reachable by raising a constant.** 16K is already beyond what a
32-bit-ish allocation budget should be asked for on a phone, and 32K is beyond
most desktops.

### 2.3 Content is terrain only

`export_raster_png` goes through `render::bake_rect` — biome, terrain, splat
paint, and the river-channel tint. Settlements, routes, ways, labels, territory
and manual icons all live in `map_overlay.gd` and reach no export path.

### 2.4 No style or layer choice

The export renders with whatever `TerrainAppearance` is current
(`WorldGen::appearance()`, `lib.rs:3759`). The style surface that *would* have
to be selectable already exists and is well-shaped for it:

| surface | where | count |
|---|---|---|
| named looks | `render::LOOK_PRESETS` | 3 (`Quality tier`, `Natural Vibrant`, `Antique Parchment`) |
| elevation ramps | `render::RAMP_PRESETS` | 9 |
| numeric tunables | `TerrainAppearance::TUNABLE` + `TUNABLE_LIGHTS` | the `list_appearance_tunables()` table |
| a whole saved look | `save_appearance_preset` / `load_appearance_preset` | JSON, `format: "cartalith-appearance"` |

So "the standard style from the carto layer, but a user might want their own"
already has a serialisation format and a loader. An export-time override should
be a *layer over* `appearance()` that does not mutate the live one — not a
`set_look` / export / `set_look` back dance, which would leave the user's own
appearance changed if the export failed halfway.

---

## 3. The documented decision that would have to be reversed

`export_raster_png`'s own doc comment (`export_raster.rs`, around line 174)
records a real decision with a real reason:

> **Tiled and single are the same pixels.** The raster is rendered **once**
> either way and only the file layout differs, so ticking `bakeTiles` cannot
> change what the map looks like. That is a deliberate departure from the
> reference, which re-renders per tile because a browser canvas has a hard area
> cap (~16.7 MP on iOS Safari, which its own `canvasWorks` probe exists to
> detect) — a constraint no native build has. Rendering once is also strictly
> less work and removes any chance of a seam.

That was correct at 8192 × 5244. It is **exactly** what blocks 32 768 × 20 976.
Reversing it is a considered change to a documented decision, on the same
footing as `DECISIONS.md` §7a-§7h, not a bug fix — and whoever reverses it owes
the guarantee the note was protecting, in a form at least as strong: **the same
pixels, with no seams.**

---

## 4. The banded renderer — prototyped, verified, and reverted

Built and run during this pass, then reverted with everything else. Recorded
here in full because the verification is the expensive part and the result was
better than expected.

### 4.1 Which stages are band-safe, and the one that is not

| stage | band-safe? |
|---|---|
| `render::bake_rect` | **yes, already.** Every output pixel is a pure function of its own `(x, y)` through `sx`/`sy` derived from the *full* dimensions — which is precisely why it takes `out_w`/`out_h` separately from the rectangle it fills. |
| `render::apply_color_grade` | **yes**, per-pixel. |
| `render::build_grade_influence` | **yes with care**: it lifts a `gw × gh` cell field into output space by `oy · gh / h`, so a band must be given the **full** `h` and its own row offset, never its own height. Splits cleanly into a cell half (once per export) and a row half (per band). |
| `render::apply_local_contrast` | **no.** It is the only neighbourhood stage in the renderer, and it is the whole problem. |

`apply_local_contrast`'s radius is
`round(gw · local_contrast_radius_frac)` clamped to `[3, gh/4]`, with
`local_contrast_radius_frac` defaulting to **0.010** — so **~328 rows** at
32 768 wide. Its blur is `blur_once` = one `box_h` then one `box_v`, each at
that radius, so the vertical dependency is exactly ±`rad` rows.

### 4.2 Why an apron is bit-identical rather than merely close

A band that renders `rad` extra rows above and below itself — **clipped at the
image edges, where the full pass clamps too** — sees exactly the window the
whole-raster pass sees. The one thing that could break that is `box_v`'s
*incremental* accumulator (`acc += src[in] − src[out]`), which in general makes
a partial sum depend on where the walk started.

It does not break here, and the reason is worth keeping:

> Every luma is `0.2126·r + 0.7152·g + 0.0722·b` over integer channels, cast to
> `f32`. The smallest value that can occur is `0.0722` (`b = 1`), whose `f32`
> ulp is `2⁻²⁷`; the largest is `255`. So every luma is an integer multiple of
> `2⁻²⁷` below `2⁸`, and a window sum of a few thousand of them needs well under
> `f64`'s 53 bits of mantissa. Every partial sum, every difference and every
> accumulator state is therefore **exact**, and the incremental walk equals the
> direct window sum no matter which row it started from.

Two further details a re-implementation must not miss:

- `apply_local_contrast`'s `border_cover(a, x, y, gw, gh)` call must be given
  the band's **image-space** row and the **full** height. Passing the band's own
  coordinates would draw a plate neatline across every band boundary.
- The radius must be derived from the full `(w, h)`, not the band's. A band that
  computed its own would boost harder band by band, and every seam would show as
  a step in local contrast.

### 4.3 What was actually measured

A `tests/export_bands.rs` was written against `src/render.rs` (the same
`#[path]` trick `tests/bake_raster.rs` already uses, so it runs under plain
`cargo test` with no Godot present). Six tests, all passing:

- **A banded render is byte-identical to a monolithic one.** A 61 × 43 fixture
  rendered at 512 px wide, once as a single band and then at 271 / 128 / 64 /
  37 / 5 rows per band. **Zero differing bytes** in every case — not "a handful
  off by one level", zero.
- **The same with the river-channel tint, `world = true` wrap and hachure on.**
  Zero differing bytes at 97 / 48 / 16 rows per band.
- **No step at a band boundary.** The mean absolute row-to-row delta across each
  boundary in the banded image matched the same rows of the monolithic image to
  within `1e-12`, over ≥ 4 boundaries.
- **The identity is not vacuous**: separate assertions that `local_contrast > 0`,
  that the plate frame is on, that the grade influence is non-empty, that the
  apron is non-zero at the test's own size, and that turning local contrast off
  changes the picture.
- **Plan arithmetic**: every row covered exactly once at 2K/4K/8K/16K/32K, and a
  budget equal to the whole raster yields exactly **one band with a zero apron**
  — i.e. the shipped 2K/4K/8K path stays the monolithic computation, not a
  banded approximation of it.

This is the answer to "how do you prove it at 32K, where there is nothing to
compare against": the band plan derives every decision from the full `(w, h)`,
so it is width-independent by construction, and the identity is then measured at
a width that *can* be run both ways.

---

## 5. The overlay constraint this pass found

`map_overlay.gd` draws in camera space: `_crisp_begin()` sets
`draw_set_transform(…, Vector2(1/zoom, 1/zoom))` and returns `k`, so every
coordinate and every font size inside it is in **screen** pixels. An export
therefore needs a **synthetic camera per band**, not the live one.

The blocker for the obvious shape of option (a): a `SubViewport` does not
produce a texture until a frame has been drawn, and GDScript reaches that with
`await RenderingServer.frame_post_draw`. **A single synchronous `#[func]` that
calls back into GDScript through a `Callable` cannot await**, so the export
cannot be one Rust call with an overlay callback. It has to be a **session**
driven from GDScript across frames:

```
begin(opts) → per band: render terrain → draw overlay → composite → write → finish
```

which in turn means the Rust side must either rebuild its `RenderCtx` per band
(the honest simple option — `RenderCtx::with_appearance`'s precomputes are
grid-resolution, so this is seconds added to an export that is minutes long) or
hold the heavy precomputes as owned parts. This was reached but not built.

### The settlement-LOD question, now sharper

`map_overlay.gd`'s `SETTLEMENT_LOD` gates town/village/hamlet on camera zoom
(`town: 0.4`, `village: 0.7`, `hamlet: 1.4`; capital/city/metropolis always
drawn), with `VILLAGE_ADDON_LOD = 2.4` hiding addon villages outright below it,
and the thresholds measured against `_lod_zoom_base()` — a value derived from
the displayed rect, not a constant.

**A flat image has no zoom.** There is no tier to inherit, and "whatever the
camera was doing" is not an answer at 32K. So the options struct must state what
the export includes — an explicit "down to this tier", or simply all of them —
rather than mirroring the viewport. That is squarely part of the owner's
"options on what to include".

---

## 6. Codecs

**Read this section for what it is.** The Rust-side capability claims below were
verified by reading the crates' own source in the local registry. The
suitability judgements about JPEG's linework damage and AVIF's encode cost are
**analysis supplied by the coordinator**, recorded as such and not verified
here. **No file-size or encode-time measurements were taken** — the export runs
that would have produced them were never made, because the work was shelved
first.

### 6.1 The streaming constraint decides more than the compression ratio

At 32K the raster cannot be assembled in memory, so **the encoder must accept
the image incrementally**. That eliminates more candidates than quality does:

| format | max side | streams from bands? | verdict |
|---|---|---|---|
| **PNG** | 2³¹−1 | **yes** — `png::Encoder::write_header()` → `Writer::stream_writer()` gives an `io::Write` that takes rows | **keep as the default.** Lossless, universal, and `png` 0.18.1 is *already resolved in `Cargo.lock`* through `image`'s png feature; naming it directly adds no new package. `image`'s own `PngEncoder` takes a whole buffer, which is why the `png` crate has to be named. |
| **BigTIFF + Deflate** | 2³²−1 | **yes** — `TiffEncoder::new_big()`, then `ImageEncoder::rows_per_strip()` / `next_strip_sample_count()` / `write_strip()` | **the large-format option.** Verified present in `tiff` 0.11.3 (**MIT**, pure Rust, so the Android cross-build is unaffected). `Compression::Deflate(DeflateLevel)` behind the default `deflate` feature, plus `Predictor::Horizontal`, which is what makes Deflate competitive with PNG's own filtered Deflate on smooth gradients. |
| WebP | **16 383** | — | **impossible, not merely unsuitable.** WebP is VP8-bitstream-compatible and encodes its dimensions in 14 bits, so 16 383 is a hard format maximum. That fails 32 768 outright and fails **16 384 by one pixel**. Confirmed against Google's WebP FAQ and RFC 9649. Written down because it is otherwise the obvious-looking choice and will be suggested again. |
| **JPEG XL** | 2³⁰ | no streaming API | **eliminated on licensing, before the FFI question arises.** The one usable pure-Rust encoder, `jxl-encoder` 0.3.1 (2026-07-11), is **AGPL-3.0-only or commercial** — which this workspace's `MIT OR Apache-2.0` cannot take. `jxl-oxide` is decoder-only; `zune-jpegxl` describes itself as a small proof-of-concept encoder. The libjxl-FFI-on-Android question was never reached. |
| AVIF | large | no | coordinator's analysis: encoding ~688 megapixels would be extraordinarily slow and large-image decoder support is uneven. Not investigated further. |
| JPEG | 65 535 | in principle | fits, but coordinator's analysis is that chroma subsampling and DCT ringing land on exactly the thin coloured linework a map is full of — routes, borders, labels, settlement pins. Terrain gradients would survive; the linework would not. |

### 6.2 The dependency cost, if BigTIFF is taken

`tiff = "0.11"` at **default** features resolved cleanly against this workspace
and added **six** packages: `tiff`, `fax`, `zune-jpeg`, `zune-core`, `weezl`,
`quick-error`. Four of those (`fax`, `zune-jpeg`, `zune-core`, `weezl`) are
decode-side codecs an export path never reads, and `tiff`'s default feature set
is what drags them in. `default-features = false, features = ["deflate"]` is the
right form and should reduce the addition to `tiff` + `quick-error` — `flate2`
and `half` are already in the tree — **but that trimmed resolution was not
re-run before the work was shelved, so treat the reduced figure as expected
rather than confirmed.** `zstd` is available as a feature and should **not** be
taken: it pulls a C library into a build that cross-compiles to Android, to save
perhaps 10 % over Deflate on a format fewer tools read.

A `BandWriter` over both formats was written (thread-owned encoder plus a
depth-1 `SyncSender`, which sidesteps `StreamWriter`'s self-referential borrow
and gives back-pressure for free) — **but it was never compiled or run.** Treat
it as a design sketch, not as verified code.

### 6.3 The thing the owner should hear plainly

**At 32K, no codec makes this small.** Lossless RGB at 32 768 × 20 976 is
~2.06 GB raw; a good lossless codec on terrain art might reach 2-4×, so
500 MB - 1 GB is the realistic landing zone. Lossy JPEG XL at visually-lossless
quality could plausibly reach 100-200 MB, but that is a quality trade on his
linework, it is his call, and — see above — the encoder licensing rules it out
for this workspace anyway.

Whatever ships, `export_raster_estimate()` should report **the estimated output
file size per format** alongside the peak memory, clearly labelled an estimate,
so a forty-minute export is chosen against a number rather than discovered.

---

## 7. Milestones, if this is un-shelved

Written as a sequence so the overlay decision and the UI pass stay separately
reviewable.

- **E1 — the banded terrain renderer.** `ExportBandPlan` + a band entry point in
  `render.rs`, the `apply_local_contrast` / `build_grade_influence` splits of
  §4.1, and `tests/export_bands.rs`. *Prototyped and green during this pass; see
  §4.3 for exactly what was proven.* The shipped 2K/4K/8K path must come out as
  one band with a zero apron, i.e. unchanged.
- **E2 — the streaming writer.** PNG first, BigTIFF second, band-in / file-out,
  with round-trip tests that decode with a different decoder than the encoder and
  compare every byte at several band geometries including a short final band.
- **E3 — the options struct.** One dictionary across the boundary, not fifteen
  `#[func]` parameters: width, format, style override (look / ramp / preset file
  / tunables, layered over `appearance()` without mutating it), and the content
  set including the explicit settlement tier of §5. `export_raster_estimate`
  extended to report dimensions, band count, peak bytes **and** estimated file
  size per format.
- **E4 — the overlay session.** The cross-frame `begin` / band / composite /
  `write` / `finish` contract of §5, with `map_overlay.gd` drawing into a
  `SubViewport` at export scale under a synthetic per-band camera. This is the
  one milestone with no reference behaviour to port against.
- **E5 — the export dialog.** Owned by the UI pass, against E3's contract.

## 8. What must not be broken

- The shipped `export_raster_png(path, width, tiled)` at 2K/4K/8K, including the
  tiled layout and its `index.json`. It is in use.
- `export_layer_previews` and `export_channel_atlas`, which share the file and
  the `export_render` helper but none of this problem.
- The reference HTML, which is read-only.
