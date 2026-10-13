# High-resolution image export: findings, constraints and milestones E1-E5

> **Shelved at the owner's request 2026-08-25**, the day it was raised
> (*"Let's shelve the 16k export and higher for the moment."*); **un-shelved
> 2026-09-06 by ruling 15**, against the standing recommendation; and
> **resumed again 2026-09-23 by Ruling AP**, which un-shelves E4's batches B–D
> (§7). The findings below were written under the hold, so anything phrased
> as "if this is ever un-shelved" describes a condition that has happened.
>
> **Both of Ruling AP's sub-questions are answered** (Ruling AZ, 2026-09-28,
> `LARGE_ITEM_RULINGS.md`; *corrected 2026-10-13: this header still called
> them open*). E4's five: overlays in v1 are settlements, labels, roads and
> ways, icons and borders; the user picks a minimum settlement tier (default
> towns); rivers draw the vector line, not the baked ink; town plans draw above
> a size readable at the export's resolution; and **no UI freeze** -- render off
> the main thread with progress. The codec: **JPEG alongside PNG**, amending
> ruling 26 (PNG only); the dialog still shows the expected size. The text
> below that predates the answers is kept as the record of what was asked.

> **The owner scoped the deliverable on 2026-09-06, and it is narrower than
> this document assumes throughout:** *"a user generated monolithic image of
> the map. No layers, no extensive information. Just to be used outside of
> Cartalith in an image viewer."* One flat raster — no sidecar metadata, no
> layer preservation, no tiling. Read §6's codec comparison as history; what
> remains live there is the size arithmetic.
>
> **The pass that wrote this document left the tree as it found it.**
> Everything it described as "prototyped" was written, run, and then
> reverted, and the shipped 2K/4K/8K export was untouched. Test suite before
> and after: **139 binaries, 2 254 passed, 0 failed, 8 ignored.**
>
> **2026-09-06 changed the tree.** `BAKE_WIDTHS` is now five rungs, the peak
> estimate was corrected from 15 to 23 B/px, `export_raster_estimate` returns a
> measured `file_bytes`, and every export is gated on a real memory budget. The
> shipped 2K/4K/8K behaviour is unchanged except that it now refuses rather than
> aborts on a device that cannot hold it. §2.1, §2.2 and §6.3 carry the
> measurements; §3 enumerates what still depends on the render-once decision,
> which was **not** reversed; §4 records that the banded prototype is not
> recoverable from git.
>
> **RGB, ruling 26's other build item, needed no change: the path was already
> three-channel.** `render::bake_rect` fills `vec![0u8; w * h * 3]`,
> `cartalith_assets::raster::encode_png_rgb8` takes exactly that, and
> `write_tiles` cuts three-byte runs out of it. There is no alpha anywhere on
> this path to drop, so ruling 26's *"cuts the pre-encode allocation by a
> quarter"* was already banked — the 2.06 GB / 515 MB figures it quotes are the
> raster the code has been allocating all along, not a saving still to be made.
>
> **This document defines what a 16K/32K export is — the findings, the
> constraints, the codec survey and the five milestones §7 sets out. It does
> not track them.** Anything about where work stands belongs to
> **`cartalith-native/docs/STATUS.md`**, which is the only place progress is
> recorded.

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

### 2.1 The width ceiling — **closed 2026-09-06**

```rust
const BAKE_WIDTHS: [i64; 5] = [2048, 4096, 8192, 16384, 32768];
```

`export_raster_png` refuses anything not in that array rather than rounding —
deliberately, and the doc comment says why. **8192 was the ceiling until ruling
15; it is now 32768**, and both new rungs were run end to end before being
offered rather than merely permitted. Three worlds each, `2048 × 1311` grid,
`_exportbig_probe.gd`:

| width | file, median of 3 | wall | peak resident |
|---|---|---|---|
| 8 192 | 28.1 MB | 3.9 s | 1 377 MB |
| 16 384 | 80.4 MB | 15.7 s | 4 041 – 4 169 MB |
| 32 768 | 213.9 MB | 69.2 s | 15 187 – 15 349 MB |

The ceiling is now a **memory budget rather than a constant** — see §2.2.

### 2.2 The whole raster is held in RAM — **and 15 B/px was too low by half**

```rust
const PEAK_BYTES_PER_PIXEL: u64 = 3 + 4 + 16;   // 23, was 3 + 12
```

The old figure counted `apply_local_contrast`'s luma plus *one* buffer per blur.
`render::blur_once` allocates **two** — `b` for `box_h`'s output and `out` for
`box_v`'s, both live until it returns — and `apply_local_contrast` runs two of
them inside one `rayon::join`, which may execute them concurrently. 3 (RGB8
raster) + 4 (`luma`) + 16 (two blurs × two `f32` buffers) = **23**.

**Measured, not merely re-derived.** Peak resident set polled by the host across
single-export runs gives an incremental slope of **21.7 B/px** over both large
intervals — `(4169 − 1377) MB ÷ 128.9 MP = 21.66` for 8K→16K and
`(15349 − 4169) MB ÷ 515.5 MP = 21.69` for 16K→32K. A working set undercounts,
so 23 is the bound the gate budgets against.

Height comes from `render::bake_dims` = `round(W · GH / GW)`, so at the app's
own 2048 × 1311 grid:

| width | height | pixels | raw RGB8 | **peak at 23 B/px** | measured peak resident |
|---|---|---|---|---|---|
| 2 048 | 1 311 | 2.7 M | 8 MB | ~62 MB | 762 MB total, export delta below the noise |
| 4 096 | 2 622 | 10.7 M | 32 MB | ~247 MB | 780 MB total |
| 8 192 | 5 244 | 43.0 M | 129 MB | **~988 MB** | 1 377 MB total |
| 16 384 | 10 488 | 171.8 M | 515 MB | **~3.95 GB** | 4 041 – 4 169 MB total |
| 32 768 | 20 976 | 687.4 M | 2.06 GB | **~15.8 GB** | 15 187 – 15 349 MB total |

(The "total" column is the whole process, which already holds ~765 MB for a
world at that grid before the export starts.)

**This was reached by raising a constant, and it should not have been reachable
that way alone.** 16K and 32K both complete on a 31 GB desktop; neither is
possible on the phone, which peaks near 878 MB on a 2048 × 1311 world. So the
ladder was raised **together with a refusal**: `export_raster.rs::
refuse_unaffordable` compares the peak against `OS.get_memory_info()`'s
`available` and returns `{ok: false, error}` rather than attempting it, because
a `Vec` allocation that cannot be served **aborts** the process — inside a
GDExtension that costs the editor and any unsaved world. Where the platform
reports no budget at all (Godot fills unavailable entries with `-1`), anything
above the pre-ruling `UNGATED_MAX_WIDTH = 8192` is refused rather than risked.

§7's **E1 is what removes the ceiling instead of gating it** — the banded
renderer of §4 — and E2's streaming writer is what turns that into a lower
peak. The banded path's peak, as measured in E2's commit (`fc7db2b`): **~1.33
GB for a 32K PNG** against the monolithic ~15.2–15.3 GB, tracking band height
at ~21.9 bytes per rendered-band pixel. Where E1 and E2 stand is `STATUS.md`'s.

### 2.3 Content is terrain only

`export_raster_png` goes through `render::bake_rect` — biome, terrain, splat
paint, and the river-channel tint. Settlements, routes, ways, labels, territory
and manual icons all live in `map_overlay.gd` and reach no export path except
E4's overlay session (§5). E3's `export_image` parses an overlay in its content
set and **refuses** it until that session draws it, rather than writing a
terrain-only file that looks as if it honoured the request.

### 2.4 No style or layer choice

The export renders with whatever `TerrainAppearance` is current
(`WorldGen::appearance()`). The style surface that *would* have
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
appearance changed if the export failed halfway. E3's override is structural
rather than behavioural: `WorldGen::appearance_rebased` takes `&self`, so it
cannot write the live appearance at all.

---

## 3. The documented decision that would have to be reversed

`export_raster_png`'s own doc comment (`export_raster.rs`, "# Tiled and single
are the same pixels") records a real decision with a real reason:

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

### What actually depends on it, enumerated 2026-09-06

Ruling 15 says the reversal is the first cost and that what depends on it must
be established first. Established by reading the code rather than the note, and
**not reversed** — this pass raised the ceiling by gating memory (§2.2), which
buys the two sizes on machines that can hold them without touching the
guarantee:

1. **`write_tiles` is the only consumer of "one finished raster".** It takes
   `rgb: &mut [u8]` spanning the whole image and copies row runs out of it; it
   has exactly one call site, `export_raster_png`'s `tiled` branch. Its
   guarantee is not just prose — `_exportraster_probe.gd` §8 samples 3 000
   pixels of `tile_0_0.png` against the same region of the single-file 4K export
   and asserts **zero** differ.
2. **`render::apply_local_contrast` is the only neighbourhood stage**, which
   §4.1 already establishes and §4.2 already solves with an apron.
3. **`render::build_grade_influence(ctx, w, h)`** allocates the full `w × h` and
   lifts cells by `oy · gh / h`, so a band must be handed the full `h` and its
   own row offset — §4.1's "yes with care", restated here because it is a
   *dependency* on the whole-image shape and not only a porting note.
4. **`render::bake_rect`'s band-safety is already shipped, not merely claimed.**
   `export_snapshot_png` calls it as
   `bake_rect(ctx, &bf, chan, out_w, out_h, x0, y0, w, h)` — a sub-rectangle of
   a *virtual* image far larger than anything allocated — and that is the
   Markdown Vault's map snapshot, in use. So the riskiest-sounding line of §4.1
   is the one with a production caller.
5. **The memory estimate is keyed to the monolithic peak.**
   `PEAK_BYTES_PER_PIXEL`, `refuse_unaffordable` and `export_raster_estimate`'s
   `peak_bytes` all describe holding the whole raster at once. A banded path
   makes the peak a function of band height, so **E1 has to move those three
   together** or the app will refuse exports it could serve.

Nothing else in the tree reads the whole-raster property.

### How the milestones answered it

**Not by reversing the decision in place.** The banded path is a second
entry point, `export_image` (taking E3's options), and `export_raster_png`
stays monolithic and render-once, with its note still true of it; the "same
pixels, no seams" guarantee the note protects is carried for the banded path
by E1's byte-identity tests (§4.3). Item 5's three figures moved
the way it said they must: `export_raster_estimate` reports the banded plan
beside the monolithic peak (`bands`, `band_rows`, `apron_rows`,
`band_peak_bytes`, `band_affordable`).

**The two paths therefore refuse differently, and the owner ruled that
correct (2026-09-23, recorded on `OUTSTANDING_WORK.md`'s export row).** On a
platform that reports no free memory, `export_image` allows 16K/32K — its band
peak never exceeds the 8K monolithic figure — while `export_raster_png` still
refuses anything above 8K, because it is not banded and its real cost at 32K
is ~15 GB. The first framing of that question proposed relaxing the old path
to match; that would have let a real ~15 GB allocation proceed blind.

---

## 4. The banded renderer — the reverted prototype, and E1's design

Built and run during the pass that wrote this document, then reverted with
everything else. Recorded here in full because the verification is the
expensive part and the result was better than expected.

> **The prototype was never in git, so this section is the whole artefact.**
> Ruling 15 said the prototype *"should be recovered from history rather than
> rewritten"*. It was reverted **before** its pass committed, so no commit
> ever contained it: `git log --all --diff-filter=A -- '*export_bands*'`
> returned nothing on 2026-09-06, and `git log --all -S ExportBandPlan` then
> returned only two prose commits — `76f5e6b` (whose `--stat` is `CLAUDE.md |
> 1 +` and `EXPORT_SCOPE.md | 376 ++++`) and `fd9de7c`, the backlog row.
> **§4.1–§4.3 were the whole surviving artefact, and E1 is a rewrite against
> them, not a `git checkout`** (`MISTAKES.md`: prove an artefact is in history
> before saying "recover it from history").

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

**One limit on that argument, disclosed in E1's commit (`de3c95e`):** it
covers `box_h`'s luma sums but not strictly `box_v`'s sums of averages at
32K's 657-row window, which need ~55 bits against `f64`'s 53. The identity is
measured exact at test size and not proven exact at that extreme; a byte would
flip only if a 1-ulp difference also flipped a `u8` rounding.

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

*(Everything in this subsection describes the reverted prototype. E1's own
test file has the same name,
`cartalith-native/crates/cartalith-godot/tests/export_bands.rs`, and these
properties are its bar in §7.)*

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
*(2026-09-24: built since — the second option. `cartalith-godot/src/export_session.rs`'s
`ExportSnapshot` owns what a band reads, assembled from the LOD worker's
`SnapshotInputs`, in E4 Batch A, `34db87f`, 2026-09-23. Progress is `STATUS.md`'s.)*

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

### E4 as scoped, 2026-09-23

The overlay session was scoped by measurement before any of it was built,
because a naive port of option (a) hits five things (recorded on
`OUTSTANDING_WORK.md`'s export row, `9aa879e`):

1. **A 32 768-px-wide `SubViewport` returns a null image** on this Godot
   version, so a full-width 32K band cannot be one viewport: the overlay
   tiles on both axes. Android's limit is unmeasured and likely lower.
2. **`SubViewport` readback is premultiplied alpha** (a 50%-white rect reads
   back `[128, 128, 128, 128]`), so the straight-alpha formula would darken
   every antialiased edge. `export_session.rs::premul_over` /
   `composite_premul_over` carry the right one.
3. **The export's pixel→grid mapping is corner-aligned** (`bake_rect`'s
   `sx = (gw−1)/(W−1)`) while the live overlay's is texel-centred, so a naive
   "control size W, zoom 1" camera compresses the overlay by ≈8 px at 32K's
   edges. The fix is a derived per-axis affine on a parent-node camera — not
   `SubViewport.canvas_transform`, which the urban-layout culler reads
   around.
4. **Symbol size means three different things inside `map_overlay.gd`**: pins
   scale with the fitted map width, way and label widths are constant screen
   pixels, and glyph rasterisation caps at 256 px. **Owner answer,
   2026-09-23: uniform magnification** — symbols keep their proportions on a
   giant export, like a real poster. That is the bigger lift: new
   symbol-scale drawing code in `map_overlay.gd`, not the screen-pixel-constant
   shortcut recommended for v1.
5. **Copying overlay state is a silent-divergence risk**: one missed setter
   makes the export differ from the screen with nothing failing. The setter
   list is derived from every real caller, and parity is proved by diffing the
   live and export overlays under an identical camera.

**Sequenced as four batches, each verified on its own:** A, the Rust session
core — `ExportSnapshot` (the world as it was when the user pressed the
button, owned, so a sculpt or regenerate mid-session cannot change the
export), `ExportSessionCore` (tiles must cover every band exactly once, in
order), `export_stream.rs::BandSink`, and the `export_session_*` `#[func]`s;
B, one-tile registration; C, many tiles and many bands end to end; D, wiring,
real-size measurement and documentation. Point 4 was the question blocking B.
**Ruling AP resumed B–D; Ruling AZ answered the five scope questions (top of
this document).**

### E4 batch B as built, 2026-10-13

Where it stands is `STATUS.md`'s; this records the design the batch chose.

- **A second overlay, not the live one.** `map_overlay.gd::copy_state_from`
  copies every script variable of the live overlay except a named list of
  render-cache and input bookkeeping (`EXPORT_STATE_SKIP`), deep-copying
  arrays and dictionaries -- point 5's silent-divergence risk answered by
  copying the definition rather than enumerating setters. A probe draws the
  live overlay and a copy under one camera: **0 differing pixels** of 26 056
  covered.
- **Laid out at export size, fit view.** The copy's local size is the grid at
  `(W-1)/(gw-1)` px per cell; a parent `Node2D` applies the per-axis
  corner-aligned affine of point 3 plus the tile offset. Measured: two edge
  capitals land 0.47 and 0.24 px from `bake_rect`'s pixel at 8192, where the
  naive camera would be ~11 px off.
- **Uniform magnification (point 4).** `prepare_export` sets camera zoom 1 and
  `_sym_gain` = export rect width / live rect width, which multiplies what the
  live overlay holds in constant screen pixels (ways via `_way_scale`, "fixed"
  labels and the label clamps, icons, landmark rings); pins, "zoom" labels and
  town plans already scale with the rect. Measured: the same region at 8192
  covers 3.885x what it covers at 4096 (gain^2 = 4.001); withholding the gain,
  2.972x.
- **Content.** The export's tier replaces the settlement LOD (no dots; addon
  villages only at `"all"`); ways/routes/labels/icons off clear their data;
  hover, selection, journeys, conflicts, the timeline's ghosts and the
  overlay's own rivers are always cleared (the terrain band paints the vector
  rivers, RV-5).
- **One tile end to end.** At 2048 px (one band) the tile goes through the
  batch-A session: every pixel the overlay leaves clear is byte-identical to
  the terrain-only `export_image`, every opaque one is the overlay's colour.

**Left to C and D, found while building B:** a tile's culled layers are built
for its own view, so a label or pin crossing a tile seam must be checked; the
copy has no route to the engine for town layouts it has not cached (the live
overlay's request signal is not wired); the territory wash ("borders") is an
engine texture, not this overlay, so it needs its own export path; the
landmark glyph textures top out at `LANDMARK_GLYPH_RASTERS`' largest size;
`TILE_MAX = 4096` is a judgement still to be measured; and the Rust parser's
default tier for `settlements: true` is `"all"` where Ruling AZ's dialog
default is towns (E5's to pass). *(2026-10-13, batches C and D below: the
seam check, the town layouts, the borders path and the `TILE_MAX` measurement
are done; the glyph cap and the parser default are not.)*

### E4 batches C and D as built, 2026-10-13

Where it stands is `STATUS.md`'s; this records the design and what was
measured. All figures are one run on the dev desktop (RX 7800 XT,
gl_compatibility, windowed) unless a count is given.

- **The driver.** `ExportOverlay.run(bridge, path, opts, live, borders,
  tile_max)` (`shell/export_overlay.gd`) takes one export through the session:
  begin, build the overlay copy, fetch the town plans, then per band poll until
  its terrain is ready, draw and submit its tiles (`band_tiles`, at most
  `tile_max` a side), and close the file. It emits `progress(fraction, text)`,
  `cancel()` aborts it and removes the file, and every engine call goes through
  new gated `EngineBridge.export_session_*` wrappers, which answer `busy`
  while a generation holds the engine; the driver waits that out (the session
  and the copy own their state). Nothing in the shell calls `run` yet: the
  dialog is E5.
- **Off the main thread (Ruling AZ).** Three things moved in `export_session.rs`:
  the band terrain renders on a feed thread from `begin` (`BandFeed`, a
  rendezvous channel, so at most one finished band waits), polled by the new
  `export_session_poll`; the snapshot build itself runs first on that thread
  (it was **695 ms** of main-thread time inside `export_session_begin` on a 2048
  x 1311 world; after, 7.4 and 12.6 ms); and the file is closed by a closer thread
  through the new `export_session_close` (`export_session_finish` waited
  **~270 ms** for the writer's last band at 4096 px; it stays, blocking, for
  callers off the main thread). `begin` refuses while a previous session's
  feed or closer is still running (`STOPPING`, `CLOSING`), and the session's
  `Drop` joins its threads: an export cancelled a few seconds before quitting
  crashed Godot at exit (2 of 2 runs), because the feed thread was still running
  library code as the library unloaded; with the join, 0 of 2.
- **Tile seams.** Two view-dependent behaviours of the copy were removed. (1)
  The borders: a magnified `TextureRect` decides a pixel on a texel edge by
  interpolated UVs, so one tile and twelve tiles of the same 4096 px export
  disagreed on two columns (35 px, up to 103 levels); the export draws them
  with `BORDER_SHADER`, which picks the cell from the pixel's global integer
  coordinate, and measures **0 of 262 144 pixels** off the grid. (2) The
  neatline clip: the overlay's own scissor sits at a fractional pixel the
  renderer rounds per tile (a 13 px column, up to 83 levels); the copy is
  clipped by a parent `Control` at whole export pixels (`clip_pixels`, within
  half a pixel of the live edge). Town plans in an export reveal by tier, not
  by view, and draw within a whole site box of a tile, not half
  (`_urban_plan_export`; not mutation-tested).
- **What a seam measures now** (`_exportoverlay_probe.gd` §7-§8, seed 9137,
  384 x 288): a 1024 px region at 8192 with every overlay and the borders,
  one tile against six stitched: **130 px differ, by at most 1 level** (8 652
  inked px lie on the seams). The same region with the tile origin moved by
  (37, 23) and no seam at all: 1 088 px, at most 1 level -- the same
  antialiasing class, so it is not a seam property. Town plans cut through a
  town's centre: **1 pixel differs** (137 levels, 11 px from the seam, no
  neighbour) -- suspected a polygon edge on a pixel centre, but the origin
  control showed 0, so that cause is not demonstrated.
  Whole files, one 4096 px tile against twelve 1000 px tiles: **926 of
  12.6 M px differ, by at most 2 levels.** The probe's bar: AA-class pixels
  inside `_segcull_probe`'s tolerance, and at most `MAX_ISOLATED_TIES` (4,
  labelled judgement) larger ones, each alone.
- **Full size** (`_exportfull_probe.gd`, seed 20260906, 2048 x 1311, towns,
  ways, labels, icons, borders; peak polled from the host):

  | width | bands | wall | file | frames | median | longest | > 50 ms | peak resident (at start) |
  |---|---|---|---|---|---|---|---|---|
  | 16 384 | 5 | 17.4 s | 68.6 MB | 900 | 17.5 ms | 110.7 ms | 17 | 2 453 MB (1 516) |
  | 32 768 | 33 | 112.2 s | 170.1 MB | 5 803 | 18.5 ms | 110.7 ms | 14 | 2 430 MB (1 514) |

  The estimate's band peak is 942 MB at both widths; the measured rise above
  the start is ~916-937 MB. No frame reached 111 ms; the 50-110 ms ones are
  the tile composite (up to 53 ms for a 9.4 MP tile at 16K) and tile draws --
  hitches, not a freeze, and unmeasured on a phone. In the 32K file the rows
  at the 32 band boundaries step by a mean 0.25 levels against 0.29 for all
  other rows (p99 3.15), and the columns at the 4096 px tile seams by 0.27
  against 0.30: no seam step. Read by eye: the downscaled 32K and two 1:1
  crops, one across a tile seam through a label and one across a band
  boundary through a pin.
- **`TILE_MAX`** stays 4096: 8192 px square tiles read back in 84-123 ms and a
  16384 px one in 276-282 ms on this machine, so 4096 is below the desktop
  ceiling; Android's is unmeasured.
- **Town plans: owner question, default chosen.** The export draws a town as
  its plan from a 75 px site box (`EXPORT_URBAN_MIN_BOX_PX`: the box at which
  the live crossfade has finished, measured 31-75 px; the live floor of 16 px
  was recorded as a smudge). On these 1 200-2 400 km worlds no width reaches
  it -- the box is 46 px at 32K on 1 200 km -- so every town stays a pin and a
  name. A smaller world, or a lower threshold, would show plans; the probe
  checks the plan path by narrowing the copy's map width.

**Still not built** *(as of batch D; see "E5 first slice" below for what has
since landed)*: the E5 dialog and its menu entry; the Rust parser's
`settlements: true` default (`"all"`) against Ruling AZ's towns; the landmark
glyph raster cap (unmeasured at export scale); the borders are nearest-cell
blocks at export scale, as on screen at zoom; Android tile and frame
measurements.

### E5 first slice as built, 2026-10-13

Where it stands is `STATUS.md`'s; this records the design the slice chose.

- **The parser default, fixed at its cause.** `export_options.rs::
  settlements_from` (new, pure, the dictionary reader's one call) maps
  `settlements: true` with no tier to `SettlementTier::DEFAULT` = `Town`
  (Ruling AZ), not `All`; `export_overlay.gd::overlay_content` mirrors it.
  Re-baselined deliberately: `(settlements: true, no tier)` moved `"all"` ->
  `"town"`; `tests/export_options.rs::settlements_true_without_a_tier_draws_towns`
  asserts the literal name, and mutating `DEFAULT` back to `All` turns it red.
- **The dialog.** `shell/export_dialog.gd`, one script with inline phone and
  desktop branches, opened by `Data ▸ Export ▸ Export map image…`
  (`menus.gd::ID_EXPORT_MAP_IMAGE`, `DccApp.open_export_dialog`). Sizes are
  the engine's ladder (`export_raster_widths`), each priced by
  `export_image_estimate`: dimensions and the fitted PNG size; a size whose
  largest band (`band_peak_bytes`, 23 B/px) exceeds reported free memory is
  disabled with both figures, and nothing is refused where the platform
  reports none. The 16K and 32K wall times and file sizes above are shown as
  "measured once on a desktop", never as a prediction. Content: settlements
  on at towns, rivers, ways, labels, icons and borders on, journey routes off
  (labelled judgement: AZ's v1 list on). Colour space sRGB by default, Display
  P3 as the explicit option (Ruling CB item 1). The destination is the shell's
  save picker under the `exports` root. Export runs `ExportOverlay.run` with a
  progress bar and Cancel; closing the window cancels; everything is disabled,
  with the reason, while a generation runs.
- **Not in this slice:** JPEG (Ruling AZ; the streaming writer has PNG and
  BigTIFF only), a style or look override control (E3 carries `style`), a
  file-exists confirmation, and the menu row's phone/command-index check
  beyond the Data popup it lives in.

---

## 6. Codecs

**Read this section for what it is.** The Rust-side capability claims below were
verified by reading the crates' own source in the local registry. The
suitability judgements about JPEG's linework damage and AVIF's encode cost are
**analysis supplied by the coordinator**, recorded as such and not verified
here. **The pass that wrote this section took no file-size or encode-time
measurements** — the work was shelved first. §6.3's measurements were taken
later, on 2026-09-06, and §6.1's BigTIFF recipe was corrected by E2 (below).

### 6.1 The streaming constraint decides more than the compression ratio

At 32K the raster cannot be assembled in memory, so **the encoder must accept
the image incrementally**. That eliminates more candidates than quality does:

| format | max side | streams from bands? | verdict |
|---|---|---|---|
| **PNG** | 2³¹−1 | **yes** — `png::Encoder::write_header()` → `Writer::stream_writer()` gives an `io::Write` that takes rows | **keep as the default.** Lossless, universal, and `png` 0.18.1 is *already resolved in `Cargo.lock`* through `image`'s png feature; naming it directly adds no new package. `image`'s own `PngEncoder` takes a whole buffer, which is why the `png` crate has to be named. |
| **BigTIFF + Deflate** | 2³²−1 | **yes** — but **not** by the route first recorded here (`TiffEncoder::new_big()`, then `ImageEncoder::rows_per_strip()` / `next_strip_sample_count()` / `write_strip()`). **In `tiff` 0.11.3 that route writes a corrupt file under compression** (found by E2, `fc7db2b`): the compressor is installed only by the whole-image `ImageEncoder::write_data`, so `write_strip` alone emits raw strips under a `Compression = Deflate` tag, which `tiff`'s own decoder and libtiff/Pillow both reject. The working route predicts and compresses each strip with the crate's public `Deflate` compressor and writes it through `DirectoryEncoder` (`export_stream.rs::write_bigtiff`) | **the large-format option.** Verified present in `tiff` 0.11.3 (**MIT**, pure Rust, so the Android cross-build is unaffected). `Compression::Deflate(DeflateLevel)` behind the default `deflate` feature, plus `Predictor::Horizontal`, which is what makes Deflate competitive with PNG's own filtered Deflate on smooth gradients. |
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
right form and reduces the addition to `tiff` + `quick-error` — `flate2` and
`half` are already in the tree. That was a prediction when written; E2 took
exactly that form and `Cargo.lock` gained exactly those two packages
(`fc7db2b`), which confirms it. `zstd` is available as a feature and should **not** be
taken: it pulls a C library into a build that cross-compiles to Android, to save
perhaps 10 % over Deflate on a format fewer tools read.

A `BandWriter` over both formats was written (thread-owned encoder plus a
depth-1 `SyncSender`, which sidesteps `StreamWriter`'s self-referential borrow
and gives back-pressure for free) — **but it was never compiled or run.** Treat
it as a design sketch, not as verified code. Its counterpart in the tree is
`export_stream.rs::BandSink`: `write_bands` on its own thread behind a
`sync_channel(1)`, the same shape.

### 6.3 The thing the owner should hear plainly

**At 32K, no codec makes this small — but PNG makes it much smaller than this
section guessed.** The paragraph below was written from a rule of thumb and is
kept for the record; **the measurement replaces it.**

> ~~Lossless RGB at 32 768 × 20 976 is ~2.06 GB raw; a good lossless codec on
> terrain art might reach 2-4×, so 500 MB - 1 GB is the realistic landing
> zone.~~

**Measured 2026-09-06, three worlds, `2048 × 1311` grid, default appearance:
32 768 × 20 976 lands at 213.9 MB (208.3 .. 229.8), i.e. a 9.6× ratio, not
2-4×.** The guess is **2.3–4.7× too high**. Ruling 26's own warning — *"a
textbook figure will be wrong in the user's favour and still wrong"* — held,
though it erred in the other direction: the estimate was wrong *against* the
user, and a size the owner would have rejected on a 1 GB projection is actually
a 214 MB file.

| width | upsample | bytes/px, median (min .. max) | file, median |
|---|---|---|---|
| 2 048 | 1× | 1.1641 (1.1363 .. 1.1860) | 3.13 MB |
| 4 096 | 2× | 0.8646 (0.8582 .. 0.8933) | 9.29 MB |
| 8 192 | 4× | 0.6544 (0.6512 .. 0.6905) | 28.11 MB |
| 16 384 | 8× | 0.4678 (0.4619 .. 0.5018) | 80.38 MB |
| 32 768 | 16× | 0.3112 (0.3030 .. 0.3343) | 213.93 MB |

Bytes/px falls because the information in the picture is bounded by the **grid**,
not by the export width; everything past `gw` is interpolation, which a PNG
filter predicts almost for free. `export_raster_estimate` now returns
`file_bytes` from a two-constant power law fitted to those five rows
(`FILE_BYTES_AT_GRID_WIDTH` / `FILE_BYTES_UPSAMPLE_DECAY`, residuals +3.1 /
+0.2 / −4.4 / −3.5 / +4.8 %, inside the ±7.3 % world-to-world spread).

Lossy JPEG XL at visually-lossless quality could plausibly reach 100-200 MB,
but that is a quality trade on his linework, it is his call, and — see above —
the encoder licensing rules it out for this workspace anyway. **Ruling 26
settled it as PNG regardless, and at 214 MB the size argument that ruling spent
turns out to have been much smaller than either party thought.**

Whatever ships, `export_raster_estimate()` should report **the estimated output
file size per format** alongside the peak memory, clearly labelled an estimate,
so a long export is chosen against a number rather than discovered. With two
formats in E3's options, that is one model: `file_bytes` is given for PNG and
omitted for BigTIFF, for which nothing has been measured
(`export_image_estimate`'s doc comment).

---

## 7. Milestones

Written as a sequence so the overlay decision and the UI pass stay separately
reviewable. Where each stands is `cartalith-native/docs/STATUS.md`'s.

- **E1 — the banded terrain renderer.** `ExportBandPlan` + a band entry point in
  `render.rs`, the `apply_local_contrast` / `build_grade_influence` splits of
  §4.1, and `tests/export_bands.rs`. *Prototyped and green during the pass that
  wrote this document; §4.3 is exactly what was proven, and the bar.* The
  shipped 2K/4K/8K path must come out as one band with a zero apron, i.e.
  unchanged.
- **E2 — the streaming writer.** PNG first, BigTIFF second, band-in / file-out,
  with round-trip tests that decode with a different decoder than the encoder and
  compare every byte at several band geometries including a short final band.
- **E3 — the options struct.** One dictionary across the boundary, not fifteen
  `#[func]` parameters: width, format, style override (look / ramp / preset file
  / tunables, layered over `appearance()` without mutating it), and the content
  set including the explicit settlement tier of §5. `export_raster_estimate`
  extended to report dimensions, band count, peak bytes **and** estimated file
  size. `format` carries `png` (the default, ruling 26) and `bigtiff`, E2's
  second writer; the file-size estimate exists for PNG only (§6.3).
- **E4 — the overlay session.** The cross-frame `begin` / band / composite /
  `write` / `finish` contract of §5, with `map_overlay.gd` drawing into a
  `SubViewport` at export scale under a synthetic per-band camera. This is the
  one milestone with no reference behaviour to port against. Sequenced as
  batches A–D (§5, "E4 as scoped"); Ruling AP resumed B–D and Ruling AZ
  answered their five scope questions (top of this document).
- **E5 — the export dialog.** Owned by the UI pass, against E3's contract.

## 8. What must not be broken

- The shipped `export_raster_png(path, width, tiled)` at every `BAKE_WIDTHS`
  rung (2K/4K/8K, and 16K/32K since ruling 15), including the tiled layout and
  its `index.json` and its memory refusal (§3). It is in use.
- `export_layer_previews` and `export_channel_atlas`, which share the file and
  the `export_render` helper but none of this problem.
- The reference HTML, which is read-only.
