# Editing tools (PhotoCraft- and MapTiler-inspired) — scope document

Filed 2026-10-11, from the owner's request of 2026-10-10 (`OUTSTANDING_WORK.md`
row **"PhotoCraft-inspired editing tools: brush engine, gradient editor, colour
management, adjustment layers; and MapTiler-style style handling"**). This
document **defines** milestones; it carries no status column. Whether any of it
is built is `cartalith-native/docs/STATUS.md`'s answer, per `CLAUDE.md`'s "One
place holds state" rule. Where this document says what exists, it says which
symbol was opened to establish it, on 2026-10-11.

Milestone ids are `ET-` (editing tools). A grep of every `*.md` in the
repository on 2026-10-11 found no other `ET-` numbering.

## 1. What fixes the scope

The scope is not this document's to choose. Three owner rulings set it
(`LARGE_ITEM_RULINGS.md`):

- **Ruling BX item 4 (2026-10-10)**: *"(C) adjustment stacks are PER LAYER
  (terrain, water, labels/overlays), not one whole-map stack; (D) the brush
  engine applies to biome/colour paint, height sculpt, and river/road drawing;
  (E) colour management covers exported PNGs with an embedded profile, a print
  (CMYK) gamut warning in the picker, wide-gamut/HDR display, and importing
  images with profiles; (F) a Look control with hue, saturation and brightness
  sliders plus dark/light/pastel variants of each preset."*
- **Ruling BZ item 1 (2026-10-10)**: *"Editing tools build order: D, C, E, F."*
- **Ruling BZ item 5 (2026-10-10)**, the part that touches this work: (b)
  *"Colour relief (`ramp_strength`) keeps its shipped default of 0"*, with the
  hint under the ramp rows saying why ramp edits are invisible at 0. Milestone
  B's ramp bar therefore edits a ramp that the default look does not show.
  Nothing below changes that default.

Two older rulings bind parts of it:

- **Ruling BY item 2 (2026-10-10)**: one shared undo/redo history for look
  edits, *"ramp stops, biome colours, style presets and later adjustment
  stacks"*, one entry per gesture, separate from the heightmap undo. Milestone
  C must extend that history; it must not grow a second one.
- **The 2026-08-31 interrogation rulings** (top of `LARGE_ITEM_RULINGS.md`):
  *Paint brush falloff* is a deliberate divergence from the reference and must
  be recorded in `DECISIONS.md` (it is, §7k); *Colour management* "touches the
  one surface every golden-parity fixture pins", so it ships behind a default
  that leaves sRGB byte-identical or re-baselines deliberately and says so.
- **Ruling AN (2026-09-23)**: build toward HDR/wide-gamut output; the three
  finishing stages became one pass with one quantiser (`render::finish_rgb`).
  HDR output itself was explicitly **not** part of that ruling.
- **`DECISIONS.md` §7p (2026-09-23)**: a deliberate engine or graphics change
  away from the legacy HTML is the correct behaviour, and the superseded
  approach is scrubbed rather than dual-pathed. Every milestone below is new
  engine or shell work with no reference counterpart; none is a parity port.

## 2. Sources read, and how they were used

**This repository's code** is the primary evidence. Each section names the
symbols it opened.

**External sources**, read on 2026-10-11 as documentation only. No code was
copied, and none may be: PhotoCraft is MIT/Apache-2.0 and egui-based, so its
patterns can be adapted, and any adapted code needs attribution in the style of
an `ATTRIBUTION.md`. Several of these pages were read through a
summarising fetch rather than line by line. A figure taken from one is marked
*(as read via summary)*, and it must be re-read at the source before it is
pinned in code.

| Source | URL | What it was used for |
|---|---|---|
| PhotoCraft repository front page | https://github.com/storytold/photocraft | Licence (MIT or Apache-2.0); crate layout; the "engine first, thin egui UI" split |
| PhotoCraft `crates/paint/src/dynamics.rs` | https://raw.githubusercontent.com/storytold/photocraft/main/crates/paint/src/dynamics.rs | Dynamics inputs (pressure, tilt, rotation, direction, fade, speed). The scalar control law `(min + (1 − min) × control) × (1 − jitter × random)`, clamped to `min`. A seeded RNG keyed by stroke seed, dab index and stream, so that *"feed points in any chunking, get the same dabs"* *(as read via summary)* |
| PhotoCraft `crates/paint/src/brush.rs` | https://raw.githubusercontent.com/storytold/photocraft/main/crates/paint/src/brush.rs | Spacing as a fraction of diameter (default 0.1). Scatter: both axes, count 1–16, count jitter, maximum amount 10.0 (= 1000 %) *(as read via summary)* |
| PhotoCraft `crates/paint/src/symmetry.rs` | https://raw.githubusercontent.com/storytold/photocraft/main/crates/paint/src/symmetry.rs | Symmetry modes vertical, horizontal, dual and diagonal. An axis is a centre plus a rotation. Reflected passes keep pressure and tilt *(as read via summary)* |
| PhotoCraft `ui-egui/src/gradient_ui.rs` | https://raw.githubusercontent.com/storytold/photocraft/main/crates/ui-egui/src/gradient_ui.rs | Already the model for milestone B. Its tear-off distance is 36 pt and its grab zone 8 pt; `ramp_bar.gd` uses its own 24 px `DELETE_DIST`, labelled a judgement |
| PhotoCraft `ui-egui/src/color_picker_ui.rs` | https://raw.githubusercontent.com/storytold/photocraft/main/crates/ui-egui/src/color_picker_ui.rs | Print-gamut warning: an RGB→CMYK→RGB round trip; out of gamut when ΔE > 3.0; a warning triangle plus a clickable "nearest printable colour" swatch *(as read via summary)* |
| PhotoCraft `crates/cms/src/gamut.rs`, `builtin.rs` | https://raw.githubusercontent.com/storytold/photocraft/main/crates/cms/src/gamut.rs and `.../builtin.rs` | The gamut check is a relative-colorimetric round trip to Lab, judged by ΔE76 with a default threshold of 4.0. Soft-proof CLUTs are 33³ (RGB) and 17⁴ (CMYK). The built-in "coated CMYK" profile is *synthetic* (300 % total ink, medium GCR) and CC0, **not** FOGRA or SWOP characterisation data *(as read via summary)*. **Note:** the picker's 3.0 and `gamut.rs`'s 4.0 disagree, so neither is adopted here (§6, ET-E3) |
| MapTiler, "Maps and styles basics" | https://docs.maptiler.com/guides/map-design/maps-and-styles-basics/ | Vocabulary: a *style* is a `style.json`; *layers*; *layer groups ("blocks")*; a *Quick edits* panel for palette and fonts |
| MapTiler, "Quick map editing" | https://docs.maptiler.com/guides/map-design/quick-map-editing/ | Quick edits adjusts colours by a picker **or** by hue, saturation and lightness. The page does not say whether the sliders are global or per group |
| MapTiler, style editor announcement (2023-08) | https://www.maptiler.com/news/2023/08/design-amazing-maps-with-our-new-style-editor/ | *"Using sliders, you can instantly remove all the color from the style or completely change the hue."* *"Selecting groups or individual layers gives you increasing levels of control over their appearance."* |
| MapTiler SDK, map styles | https://docs.maptiler.com/sdk-js/api/map-styles/ | Variants: *"Each reference style offers a range of variants that contain the same level of information and serve the same purpose, but use different color schemes"* (e.g. `BRIGHT.DARK`, `.LIGHT`, `.PASTEL`). An unknown variant falls back to the default style |
| Krita manual, brush tips and brush options | https://docs.krita.org/en/reference_manual/brushes/brush_settings/brush_tips.html, `.../options.html` | Spacing; scatter axes (X along the stroke, Y perpendicular, *"has the most effect"*); sensors drive options. Krita's sensor page returned 404, so the sensor-curve mechanism was **not verified** there |
| Godot 4.7 class reference, `InputEventMouseMotion`, `InputEventScreenDrag` | https://docs.godotengine.org/en/stable/classes/class_inputeventmousemotion.html, `.../class_inputeventscreendrag.html` | Both carry `pressure` (0..1), `tilt` (−1..1 per axis) and `pen_inverted` |
| Godot 4.7 class reference, `ColorPicker` | https://docs.godotengine.org/en/stable/classes/class_colorpicker.html | Modes RGB, HSV, LINEAR and OKHSL. **No CMYK mode** |
| Godot, "HDR output arrives in Godot 4.7" | https://godotengine.org/article/hdr-output-arrives-in-godot-4-7/ | HDR output on **Forward+ and Mobile only, not Compatibility**. Windows needs D3D12. Android is *"planned for a future Godot release"* |
| W3C PNG Specification, Third Edition (Recommendation, 24 June 2025) | https://www.w3.org/TR/png-3/ | Colour-chunk precedence cICP > iCCP > sRGB > cHRM/gAMA. iCCP and sRGB should not both be present. cICP carries BT.2100 PQ/HLG for HDR |
| ICC, specification page | https://www.color.org/icc_specs2.xalter | The current specification is ICC.1:2022 (profile version 4.4.0.0). The PDF itself could not be read here, so rendering-intent wording is cited from the lcms2 and PhotoCraft pages, not from ICC.1 |
| Little CMS | https://www.littlecms.com/color-engine/ | MIT; soft-proofing and gamut checking exist as features. Its API names were **not verified** (the API PDF was not read) |
| Björn Ottosson, "A perceptual color space for image processing" (2020-12-23) | https://bottosson.github.io/posts/oklab/ | Oklab/OkLCh, for hue, saturation and lightness edits that keep perceived lightness and hue independent |
| `png` 0.18.1 and `moxcms` 0.8.1 sources, in this machine's cargo registry | `~/.cargo/registry/src/*/png-0.18.1`, `.../moxcms-0.8.1` | See §6. Read, not run |

**Not read, and labelled wherever used:** the CIEDE2000 formula paper
(Sharma, Wu and Dalal, 2005). It is named as the better ΔE in §6 from general
knowledge only.

## 3. What exists today

### 3.1 Milestones A and B, and the shared look history (commits read with `git show --stat`)

- **A — the colour confirm dialog** (`84bfa153`, 2026-10-10).
  `DccWidgets.confirmable_color(pick, on_revert, on_open, on_commit)`
  (`shell/dcc_widgets.gd`) upgrades a `ColorPickerButton` in place:
  - It remembers the opening colour (`st["old"]`), draws was/now `ColorRect`
    swatches, and adds Cancel and OK.
  - Every close that is not OK reverts. The revert calls `on_revert` explicitly,
    because assigning `ColorPickerButton.color` emits no `color_changed`.
  - It also reverts on `tree_exiting`, because a picker freed by a dock rebuild
    never emits `popup_closed`.
  - `on_open` and `on_commit` are the look-history hooks.

  `STATUS.md` records it wired to the dock stop picker, the CARTO ramp swatches
  and the biome swatches. Faction, label-colour and slicer pickers are not
  wired.
- **B — the interactive ramp bar** (`d938ce5a`). `shell/ramp_bar.gd` (no
  `class_name`, `preload`ed) works as follows:
  - Drag a handle to move a stop, or drag it off the bar to delete it.
  - Click the bar to add a stop, and double-click a handle to open the A popup.
  - Live engine writes are throttled to `LIVE_MS = 33`, and each gesture ends
    in one `final` callback.
  - The bar owns no history. `on_begin` lets the host snapshot before the first
    write.
- **The shared look history** (`590c12c5`, `5081090c`; Ruling BY item 2).
  `shell/look_history.gd`:
  - Entries are `{label, before, after}`, with a cursor and a redo tail.
    `MAX_ENTRIES = 100` (labelled judgement).
  - `snapshot()` reads the parts `ramp`/`mode`, `biomes` and `style`, and
    `apply()` writes them back.
  - `wants_undo()` / `wants_redo()` route Ctrl+Z between this history and the
    heightmap stack, using `height_sig()`.

  **The file's own header lists what it does not cover:** *"Painter/NPR/
  appearance SLIDER edits, the layer stack, colour space, saved-look load, and
  the quality tier."* Milestones C and F add parts to `snapshot()` and
  `apply()`. They do not add a second history.

### 3.2 The brushes (for D)

**Paint** (biome / terrain / splat), `crates/cartalith-godot/src/paint_bridge.rs`:
- `Brush` holds `{value, radius, hardness, softness, erase, land_only}`.
  Defaults are `radius 6.0`, `hardness 1.0`, `softness 0.0` and
  `land_only true`.
- `PAINT_RADIUS_RANGE = (1.0, 40.0)` and `PAINT_TILE_SIZE = 64`.
- `PaintEditor::stroke_at(gx, gy)` pushes **one** `PaintStamp` per call. It
  calls `PaintStamp::new`/`ungated` and then
  `.with_falloff(hardness, softness)`.
- There is no begin/end pair, no spacing, no pressure and no stroke state.

`PaintStamp` (`crates/cartalith-spatial/src/paint.rs`):
- It is a categorical disc. `with_falloff` widens a **probabilistic edge band**
  (`feather_width = ((1 − hardness) + softness).clamp(0, 1) × radius`).
- The decision in the band is `passes_falloff` against `cell_dither`, a
  deterministic `fmix64` hash of the absolute cell position.
- No palette index is ever blended (`DECISIONS.md` §7k).

**The paint dispatch** is `shell/workspaces/world_workspace.gd::_paint_apply_dab`:
- It calls `bridge.paint_set_brush(...)` and then `bridge.paint_stroke_at(gx,
  gy)` once per `map_dragged` signal.
- One motion sample makes one dab, so a fast drag leaves gaps between dabs.
- Measured there 2026-09-04 (a cited figure, not re-measured): a full-grid
  preview rebuild costs **0.73 / 1.48 / 4.55 / 16.80 ms** per dab at
  512/1024/2048/4096². That is why `build_paint_preview_patch()` is used per dab
  and the panel rebuild waits for release.

**Paint undo** — is any reachable? A grep of `lib.rs` and `paint_bridge.rs` for
`.undo()`/`.redo()` found only the sculpt draft's (`lib.rs`
`sculpt_undo`/`sculpt_redo`). The paint commit records a "no snapshot" row in
`undo.rs`. This is a reading, not an absence proof.

**Sculpt** (height, including river carving),
`crates/cartalith-terrain/src/sculpt.rs`:
- **One `SculptStamp` per stroke**, holding `points: Vec<Point>` (a polyline),
  `globals: SculptGlobals`, `params: FeatureParams` and `sea_level`.
- `SculptGlobals`:
  - `brush_size` 6..200 (default 32), `hardness` 0..1 (0.5) and `intensity`
    0..1.5 (1.0);
  - the noise block;
  - `falloff: Falloff`, one of `Smooth` (default, the reference's smoothstep),
    `Linear`, `Sharp` (t³) or `Constant`.
- The coverage weight enters the effect at **two sites**:
  - `k = cov * g.intensity` in the Freehand-Smooth branch;
  - `k: hit.cov * g.intensity` in the general `Ctx` construction.
- Two of thirteen features are `radial: true`: Lake and Volcano.
- `River` is a sculpt feature: *"carves a bed along your stroke, lowers banks,
  auto-fills water on commit"*. That makes river drawing a sculpt stroke.
- `Falloff`'s own doc already scopes the three asks it declined: an elliptical
  tip; **stroke spacing / jitter**, which *"need a stroke accumulator"* and
  change *"one stamp per dab rather than one per stroke"*; and airbrush, which
  is time-dependent.
- `crates/cartalith-terrain/tests/golden_parity_sculpt.rs` exists. `Falloff`'s
  doc says it holds 22 hashes, a count this document did not re-run.

**The sculpt dispatch** is
`world_workspace.gd::_sculpt_click`/`_sculpt_drag`/release:
`sculpt_begin_stroke` → `sculpt_add_point` per motion sample →
`sculpt_end_stroke`.

**Roads** — the Way tool, `crates/cartalith-godot/src/infra_tools_bridge.rs`:
- `InfraTools::way_append_point` appends **clicked** waypoints, and
  `infrastructure_workspace.gd::_way_click` feeds it from `map_clicked`.
- `way_commit` routes every leg by least-cost Dijkstra (`civ_commit_way`).
- The module doc's section "No freehand way mode" records why the DCC spec's
  "freehand / snap / least-cost" dropdown has only one real algorithm.

**Where pointer pressure is lost:** `map_overlay.gd::_gui_input` reads an
`InputEventMouseMotion` and emits `map_dragged.emit(p["gx"], p["gy"])`, which
`viewport_host.gd` relays.
- `mm.pressure`, `mm.tilt` and `mm.pen_inverted` exist on the event (Godot 4.7
  class reference) but are never read.
- Grepping `shell/` for `pressure` found only `menus.gd`'s note that the canvas's
  *Stylus pressure* and *Palm rejection* rows *"have no counterpart anywhere in
  this shell"* (`TABLET_UI_SPEC.md` §2.1 records the same).

**The design that exists:** `design/Cartalith Paint Toolbar.dc.html` draws the
tool options bar with `spacing 18 %`, `max Δ ±420 m`, `pressure on`,
`presets ▾` and `symmetry off`. That artboard is the later bar
(`DCC_SHELL_SCOPE.md`'s canvas-precedence table), so its values are the design
defaults below, by the "newer canvas wins" rule. `SCULPT_LIVE_SCOPE.md` L4 and
`SCULPT_FUNCTION_CHART.md` §10 scope sculpt *spacing* and *mirror* as unbuilt
design. ET-D3 below is the build of those two items for sculpt; see `doc_notes`.

### 3.3 The look pipeline (for C, E and F)

**`render::finish_rgb(rgb, delta, grade, space)`** (`render.rs`):
- It is the fused finishing pass of Ruling AN: local-contrast `delta`, then the
  grade (`GradeAxes` with exposure, contrast, saturation, temperature, shadow
  and highlight tints and gamma), then the display matrix. All of it is `f64`
  per pixel, followed by **one quantiser**.
- That quantiser truncates, and rounds only when the colour space re-encoded.
- `finish_pixel_continuous` exposes the unquantised value.
- The grade gates on `TerrainAppearance::grade_is_identity`, and the space on
  `ColorSpace::Srgb`. Both are early returns, so byte identity holds by
  control flow.
- The grade's per-pixel field weights come from `build_grade_influence`. That is
  one `f32` per pixel, and the buffer is **empty** when every weight is at rest.
  This is the precedent for C's per-pixel layer weight.
- Ruling AN lists four call sites: `lib.rs::build_color_texture` (screen);
  `export_raster.rs` (`export_raster_png`, `export_layers`);
  `render::bake_export_band` (banded export); and
  `render::render_biome_tile_rgba` (LOD tiles).

**`ColorSpace`** has two members, `Srgb` (default) and `DisplayP3`, listed in
`COLOR_SPACES`. `SRGB_TO_P3` has non-negative entries and every row sums to 1,
so it cannot clip and keeps neutrals exact. `apply_color_space`'s doc states
three things:
- the transform reaches **only** the terrain raster, not Godot-drawn overlays;
- the project's `gl_compatibility` renderer does no colour management;
- `forward_plus` *"loses the device on the owner's RX 7800 XT"*
  (`cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md` has the table:
  `VK_ERROR_DEVICE_LOST` and `DXGI_ERROR_DEVICE_REMOVED`).

`project.godot` confirms `config/features=("4.7", "GL Compatibility")` and
`renderer/rendering_method="gl_compatibility"` (also on mobile).

**`LayerStack`** (`render.rs`) holds the terrain raster's three categories,
`RasterLayer::{Terrain, ColourRelief, Hillshade}`:
- Each `LayerEntry` has `{visible, opacity, blend}`. `BlendMode` is one of
  Normal, Multiply, Screen, Overlay or Add.
- It is composited in registers inside `land_color`. `LayerStack::DEFAULT` is
  byte-identical by control flow (`is_default`).
- It is **land only**: `sea_color_core` has its own shade and no ramp.
- It is bound as `WorldGen::{get_layer_stack, set_layer_stack,
  list_blend_modes}` and drawn by `render_workspace.gd`'s layer-stack panel.

This is a *compositing* stack of light and colour categories. It is not an
*adjustment* stack, and C does not replace it.

**Style presets** are `render_workspace.gd::STYLE_PRESETS`, 17 entries counted
2026-10-11, from "Natural Vibrant" to "Tanaka relief":
- Each entry is `[name, look, npr{}, appearance-overrides{}?, ramp?]` and is
  applied by `_apply_preset`.
- Each preset also carries an overlay treatment (roads, sea lanes, labels,
  borders, and since Ruling BX item 2 icons, journey markers, pins and
  factions). `map_overlay.gd::set_style_treatment` applies it.
- Colours are resolved at a few functions: `way_style_for`, `_treated_mix`,
  `_treated_rgb`, `icon_fill_for`, `label_inks`, `journey_inks` and
  `restyle_factions`.
- `_overlaystyle_probe.gd` asserts label contrast ≥ 4.5 and class inks ≥ 3.0
  (WCAG ratio).

Looks are `LOOK_PRESETS` (`render.rs`): "Quality tier", "Natural Vibrant" and
"Antique Parchment".

**Persistence.** `project_bridge.rs::AppearanceDoc` saves `quality`, `look`,
`territory_opacity`, `overrides`, `ramp`, `npr` and an optional `biome_cols`.
`biome_cols` is omitted when it equals the reference table
(`biome_cols_member`), so an untouched project stays byte-identical. The layer
stack and the colour space are not members. The colour space is excluded on
purpose: *"display = app, working space = document"* (`export_raster.rs`
module doc).

**Exports.**
- `export_stream.rs::write_png` (`png` 0.18.1) and
  `cartalith_assets::raster::encode_png_rgb8` (`image` 0.25.10) both write
  **untagged** PNGs.
- `export_raster.rs`'s module doc states the consequence and names the fix:
  *"an ICC-tagged export (write `sRGB`/`Display P3` chunks and let the consumer
  convert), which needs an encoder that takes a profile"*.
- Overlays reach an export through a Godot `SubViewport`, composited by
  `export_session.rs::composite_premul_over`. Any colour change made where
  `map_overlay.gd` resolves an ink therefore reaches the export without further
  work.

**Imports.**
- `cartalith_assets::raster::decode_png` uses `image::load_from_memory_with_format`
  then `to_rgba8()`, so it ignores colour chunks.
- It is reached by asset-pack loading (`cartalith-assets/src/library.rs`) and by
  **heightmap** import (`cartalith-engine/src/import.rs` `decode_heightmap`,
  `import_heightmap`).
- A heightmap is data, not colour. ET-E2 must never colour-manage it.

## 4. Milestone D — the brush engine (first, per Ruling BZ item 1)

**Goal.** Give the three drawing tools one deterministic stroke engine:
- pressure and tilt dynamics;
- spacing;
- symmetry;
- scatter;
- falloff on top of `PaintStamp::with_falloff`.

The tools are biome/colour paint, height sculpt (including the River feature)
and road drawing.

**Shared invariants for every D batch:**
- With every new control at its default, each tool emits **exactly** today's
  stamps. This is identity by control flow (a gate that skips the engine), not
  identity by arithmetic.
- `golden_parity_paint`, `golden_parity_sculpt` and every `golden_parity_*`
  stay unmoved.
- Pressure that the device did not report is **absent**, never `1.0` or `0.0`:
  *"never encode no value as a plausible value"*. *Not verified:* what
  `pressure` a plain mouse reports in Godot 4.7 on Windows. ET-D2 measures it
  before any default is chosen.
- Randomness (scatter, jitter) is a pure function of `(stroke seed, dab index,
  stream)`, as in PhotoCraft's `dynamics.rs`. The same input samples, delivered
  in any chunking, give the same dabs.
- Recording obligation: when D lands, `DECISIONS.md` gains a §7k-adjacent entry
  for the brush engine's divergences: spacing, dynamics and symmetry all have no
  reference counterpart. The 2026-08-31 falloff ruling set this precedent.

### ET-D1 — the stroke engine core (Rust, `godot`-free)

**Design.** Add a new module beside `paint.rs` in `cartalith-spatial`,
`stroke.rs`. It is generic machinery with no Cartalith semantics, which is the
same placement argument `paint.rs`'s module doc makes. It takes a stream of
samples `{x, y, pressure: Option<f32>, tilt: Option<(f32, f32)>, t_ms}` and
emits dabs `{x, y, size_k, flow_k, angle}`.

| Concern | Design |
|---|---|
| **Spacing** | A fraction of the current dab diameter, walked along the polyline with the remainder carried between segments: Krita's and PhotoCraft's model. The default is the Paint Toolbar artboard's `18 %` (design value); PhotoCraft's own default is 0.1. Applies only to dab tools |
| **Smoothing** | Optional "pulled string": the dab follows a point lagging the pointer by a distance. PhotoCraft's default is 100 px *(as read via summary)*; here it is cells, with the default off (judgement) |
| **Dynamics** | Each output (size, flow; angle for future tips) is driven by one sensor (pressure, tilt elevation, speed, fade), through PhotoCraft's law `(min + (1 − min) × control) × (1 − jitter × rnd)` clamped to `min`. A curve editor is **not** built, for the reason `Falloff`'s doc gives (Blender 5 converted hand-drawn curves back to presets) |
| **Scatter** | A perpendicular offset (and an optional both-axes mode) in diameters, with count 1..N and count jitter. N is capped at 16, PhotoCraft's cap *(as read via summary)* |
| **Symmetry** | Vertical, horizontal, dual (point) and N-fold radial about a centre plus an angle. Reflected dabs keep pressure and tilt. Applied **after** dynamics and jitter, so a mirrored dab is the exact mirror of its source |
| **Determinism** | A splitmix/`fmix64`-style counter RNG keyed `(seed, dab_index, stream)`. `cell_dither`'s own `fmix64` is the precedent in the same crate |

**Files.** New: `crates/cartalith-spatial/src/stroke.rs` and its tests. Edit:
`crates/cartalith-spatial/src/lib.rs` (one `mod`/`pub use`).

**Untouched.** `paint.rs`, `pass.rs`, every other crate, the shell.

**Acceptance (unit tests, each mutation-checked):**
- With spacing `s`, consecutive dab centres along a straight synthetic drag are
  `s × diameter` apart to ±1e-9 cells, whatever the sample density.
- Fed one sample every 3 diameters, the engine leaves no gap greater than
  `s × diameter`. Today's one-dab-per-sample dispatch leaves gaps of 3 diameters
  on the same input; assert that as the pre-change contrast.
- The same 200 samples fed as 1, 7 and 200 chunks produce byte-equal dab lists.
- With pressure absent, `size_k` and `flow_k` are exactly `1.0`, whatever the
  pressure sensor's settings.
- Symmetry produces the mirror of every dab to 1e-12 for each mode. N-fold
  radial produces exactly N copies.
- Mutation checks: the spacing remainder carry (drop it, and the spacing test
  goes red); the RNG stream key (swap streams, and the chunking test is
  unaffected while a pinned-literal scatter test goes red); the symmetry
  ordering (apply jitter after the mirror, and the mirror test goes red).

**Verification.** `cargo test --workspace` (never a crate subset). No probe:
nothing here draws.

**Risks.** Floating accumulation over long strokes. Carry the remainder in
`f64` and test a 10 000-sample stroke against the closed form.

**Batch.** One lane (Rust).

**Design points settled while building ET-D1 (2026-10-11), kept here because they bind ET-D2.** (a) `Dual` symmetry is the vertical mirror, the horizontal mirror and their composition (point reflection): four copies, not two. (b) The engine has no end-of-stroke hook: with smoothing on, the dab point stops short of the last pointer position, so ET-D2 must add a `finish()` flush if the stroke's tail has to reach the pointer. (c) The first dab of a stroke has no travel direction; scatter uses the world x-axis there (a labelled judgement). (d) An absent sensor reading skips the whole dynamics law including jitter; jitter alone needs `Sensor::Off`. (e) Limits chosen as labelled judgements in `stroke.rs`: `MIN_SPACING` 0.01, `MAX_SPACING` 10, `MIN_STEP` 1e-3 cells, `MAX_RADIAL` 64, default speed reference 1000 cells/s, fade distance 100 cells, tilt range [-1, 1] (Godot's convention, not verified here).

### ET-D2 — paint on the engine (biome / terrain / splat)

**Design.**
- `paint_bridge.rs` gains `stroke_begin(seed)`, `stroke_sample(...)` and
  `stroke_end()`. Each `stroke_sample` runs D1 and pushes one `PaintStamp` per
  emitted dab. The dab radius is `brush.radius × size_k`, clamped to
  `PAINT_RADIUS_RANGE`.
- **Flow has no meaning for a categorical layer** (§7k: no index is blended).
  So `flow_k` is not wired to paint. Pressure drives size only, plus optionally
  the edge band width (softness).
- The edge band gains a shape: the band's pass probability uses a `Falloff`-like
  shape. Linear is today's ramp, kept as the default, so the
  `with_falloff_at_the_construction_defaults_is_bit_identical_to_the_hard_disc`
  test and `golden_parity_paint` hold.
- `stroke_at` stays as the one-dab path; with spacing off the shell keeps
  calling it.
- A stroke becomes **one** paint undo step: the dabs from `stroke_begin` to
  `stroke_end` group into one draft-history unit. See OQ-4.
- The shell part:
  - `map_overlay.gd` adds a **new** signal,
    `map_stroke_sample(gx, gy, pressure, tilt_x, tilt_y, is_pen)`, beside
    `map_dragged`. It does **not** change `map_dragged`'s signature, whose
    listeners are `app.gd`, `global_tools.gd`, `tool_overlay.gd`,
    `cartography_workspace.gd` and `world_workspace.gd` (a grep for
    `map_dragged`, 2026-10-11).
  - `viewport_host.gd` relays it behind the same `_generating()` gate.
  - `world_workspace.gd`'s paint handlers switch to it.
- `is_pen` is decided from the event: pressure strictly between 0 and 1 seen at
  least once in the stroke, or a non-zero tilt, or `pen_inverted`. This rule is
  a judgement, and it must be calibrated against the measurement in the
  acceptance list.

**Files.**
- `crates/cartalith-godot/src/paint_bridge.rs`
- `crates/cartalith-godot/src/lib.rs` (three `#[func]`s)
- `crates/cartalith-spatial/src/paint.rs` (the band shape)
- `godot-project/map_overlay.gd`
- `godot-project/shell/viewport_host.gd`
- `godot-project/shell/engine_bridge.gd`
- `godot-project/shell/workspaces/world_workspace.gd`

**Untouched.** Sculpt, the Way tool, `cartalith-civ`'s territory brush (it never
calls `with_falloff`, per `paint.rs`'s doc), the render path and the preview
patch path (`build_paint_preview_patch` is reused as is).

**Acceptance:**
- **Measured first:** the `pressure`/`tilt` that a mouse, the touch emulation
  and (if available) a pen report on this machine, in a windowed probe. The
  `is_pen` rule is written only after that measurement, and the measured values
  are quoted in the probe header.
- With spacing off and pressure absent, a scripted drag commits a paint layer
  byte-equal to the same drag through today's `paint_stroke_at` path.
  `golden_parity_paint` is unmoved.
- With spacing 18 %, a scripted fast drag (1 sample per 3 radii) paints a
  connected stroke. The painted-cell connectivity along the path is 1
  component; today's path gives more than 1 on the same input, and the probe
  must fail against the pre-change shell.
- A stroke is one undo step: after a stroke of more than 10 dabs, one undo
  empties the draft.

**Verification.**
- `cargo test --workspace`, plus a parse check of every touched `.gd` and
  `shell/app.gd`.
- A **windowed** probe from a scratch project copy with its own `config/name`
  (no `--headless`: it rasterises; Godot 4.7.1 has no `--user-data-dir`).
- Compare the `.dll` mtime against the `.rs` mtimes and state both.
- The probe reaches the Paint tool through the real tab/sheet path on the phone
  layout as well (`MISTAKES.md` "Add a control to a tool bar...").

**Risks.**
- Dab count rises with spacing, and preview cost is per dab (16.80 ms at 4096²
  for a full rebuild). Batch the dabs of one motion sample into one
  `build_paint_preview_patch` call.
- `map_overlay.gd` is a shared, very large file. Keep the edit to the new signal
  and its emission.

**Batch.** Two lanes, file-disjoint:
- (1) Rust: `paint.rs`, `paint_bridge.rs` and `lib.rs`;
- (2) GDScript: `map_overlay.gd`, `viewport_host.gd`, `engine_bridge.gd` and
  `world_workspace.gd`, against a stub `#[func]` list agreed in the brief.

### ET-D3 — sculpt on the engine (height, including River)

**Design.** Sculpt strokes are continuous polylines, so **spacing does not
apply** to them: one stamp per stroke stays, and `Falloff`'s doc warns that
changing it moves the draft, the stamp-stack UI, undo and the save format.
Dynamics and symmetry do apply:
- **Pressure.** `SculptStamp` gains `pressure: Option<Vec<f32>>`, one per
  `points` entry, smoothed by D1.
  - At the two coverage sites, `k` becomes `cov × intensity × p(s)`, where
    `p(s)` interpolates pressure at the stroke parameter the hit carries
    (`hit.s`, read in the `Ctx` construction). **Confirm `s`'s meaning at
    `cover_at` before building.**
  - When `pressure` is `None` the multiplication is skipped, not performed with
    `1.0`, so `golden_parity_sculpt`'s hashes cannot move by an ulp.
  - Pressure on size (a varying radius along the stroke) is **not** in this
    milestone: radius enters all thirteen feature formulas and `bbox`, which is
    the elliptical-tip cost `Falloff`'s doc names.
- **Symmetry.** At `sculpt_end_stroke`, each mirror of the polyline (D1's
  transform) is pushed as its own `SculptStamp`, so the stamp stack and undo see
  N stamps. The seed derivation per mirror is a judgement (`seed + k`), stated
  in the doc comment.
- **River.** It inherits both. Pressure deepens or shallows the carved bed
  through `k`. The water auto-fill on commit is untouched.
- **Save format.** `pressure` serialises as an optional member, omitted when
  `None`, so an old or untouched draft round-trips byte-identically. This is the
  `biome_cols_member` precedent.
- **Shell.** `world_workspace.gd`'s `_sculpt_click`/`_sculpt_drag` read the new
  signal and pass pressure through a new `sculpt_add_point_p(x, y, p)`.
  `sculpt_add_point` is kept for callers without pressure.

**Files.**
- `crates/cartalith-terrain/src/sculpt.rs`
- `crates/cartalith-godot/src/sculpt_bridge.rs`
- `crates/cartalith-godot/src/lib.rs` (sculpt `#[func]`s)
- sculpt draft (de)serialisation in `project_bridge.rs` (its sculpt section
  only)
- `godot-project/shell/engine_bridge.gd`
- `godot-project/shell/workspaces/world_workspace.gd` (sculpt handlers only)

**Untouched.** The thirteen feature formulas (`eval` arms), `Falloff`, `bbox`,
paint, the Way tool.

**Acceptance:**
- `golden_parity_sculpt` is unmoved; quote its count from the run, not from
  this document.
- Pressure measurably changes the effect: for a Mountains stroke with pressure
  ramping 1 → 0.2, the committed height delta at the stroke's end is ≤ 0.25 of
  its start (judgement bar, fixture-dependent; write the fixture first, then
  the bar, then measure).
- A vertical-mirror stroke produces two stamps whose committed height deltas are
  mirror images to 1e-6. Fixture: a square grid, centre axis.
- A draft with pressure saves and reopens with an equal `pressure` vector. An
  old draft without it reopens byte-identically.
- Mutation: replace the `None` skip with `× 1.0` and confirm the hash test still
  passes. If it does, add the control-flow assertion that the skip branch was
  taken. Remove the interpolation and confirm the ramp test goes red.

**Verification.** `cargo test --workspace`, the parse check, and a windowed
probe drawing a mirrored River stroke from a scratch copy. Compare the `.dll`
and `.rs` mtimes.

**Risks.** `SculptStamp`'s new field reaches `PassBuffer::rewrite_stamps`
(sea-level re-stamping, Ruling AQ), so the re-stamp must carry the pressure
through: a test re-stamps a pressured draft and compares.

**Batch.** Two lanes:
- (1) Rust: `sculpt.rs`, `sculpt_bridge.rs`, `lib.rs` and the `project_bridge.rs`
  sculpt section;
- (2) GDScript: `engine_bridge.gd` and the `world_workspace.gd` sculpt handlers.

`lib.rs` is shared with D2, so D2 and D3 are **not** concurrent batches.

### ET-D4 — road drawing on the engine

**Design.**
- The Way tool gains a **drag-to-draw** gesture. The pointer path runs through
  D1 with spacing in **cells** (not diameters; a road has no tip) and smoothing,
  and each emitted point becomes a waypoint (`way_append_point`).
- Commit still routes every leg by least-cost Dijkstra. The module doc's "no
  freehand way mode" stands unless OQ-6 overturns it.
- Symmetry commits mirrored waypoint lists as separate ways.
- Pressure, tilt and scatter have no meaning for a way (a way has a type, not a
  width per point), so they are not offered. The options row must not draw them
  (no enabled-and-inert control).
- Clicks keep working exactly as today.

**Files.**
- `godot-project/shell/workspaces/infrastructure_workspace.gd`
- `crates/cartalith-godot/src/infra_tools_bridge.rs`, only if symmetry needs an
  engine-side multi-draft commit. Otherwise the shell commits N drafts in
  sequence.

**Untouched.** `civ_commit_way` and its routing, Route, Measure, Region select.

**Acceptance:**
- A scripted drag with spacing *k* cells yields waypoints no further than *k*
  apart along the path, and the committed way passes within one cell of every
  waypoint (snap rules unchanged).
- A click-only way commits byte-identically to today (same `ways` entry).
- A mirrored drag commits two ways whose geometry mirrors within the routing's
  own tolerance. Measure it, then state the bar: least-cost routing over
  asymmetric terrain is **not** expected to mirror exactly, and saying so is
  part of the acceptance.

**Verification.** A windowed probe, the parse check, and `cargo test
--workspace` if Rust moved.

**Risks.**
- Many waypoints make many Dijkstra legs. Measure commit time for a 200-waypoint
  drag (median with min..max, harness run alone) before choosing the default
  spacing.
- Snap-to-place on every dense waypoint may zig-zag. Snap only the first and
  last point of a drag (judgement), and say so in the code.

**Batch.** One lane (GDScript), plus a Rust lane only if the engine-side commit
is needed.

### ET-D5 — the brush controls (tool options bar and dock)

**Design.**
- Add `spacing`, `pressure` (on/off plus target: size / softness for paint,
  strength for sculpt), `symmetry` (mode plus a centre picked on the map, drawn
  by `tool_overlay.gd`) and `scatter` to the tool options rows.
- These follow `design/Cartalith Paint Toolbar.dc.html`'s two-row bar and its
  drawn defaults (`spacing 18 %`, `pressure on`, `symmetry off`).
- `presets ▾` stays out of this milestone (no design for its contents).
- `scatter` has no artboard. Derive its row from the bar's own vocabulary
  (`CLAUDE.md`'s canvas rule), or leave it to the dock.

**Files.** `godot-project/shell/tool_bar.gd`,
`godot-project/shell/workspaces/world_workspace.gd` (tool-option builders),
`godot-project/shell/tool_overlay.gd` (the symmetry axis), and
`godot-project/shell/workspaces/infrastructure_workspace.gd` (Way row).

**Acceptance:**
- Each control changes the engine state it names. Read it back through the
  bridge, then draw a stroke and measure the effect (D2/D3/D4's bars).
- On the phone layout, every new control is reachable through the real tab and
  sheet path with no forced reveal (`MISTAKES.md`, Ruling BR), and meets the
  44 px floor.

**Verification.** A windowed probe at desktop, tablet and phone densities,
pressing edges rather than centres, plus the parse check.

**Batch.** Two lanes, file-disjoint: (1) `tool_bar.gd` + `tool_overlay.gd`; (2)
the two workspace files.

## 5. Milestone C — per-layer, non-destructive adjustment stacks

**Goal (Ruling BX item 4).** One ordered, toggleable adjustment list **per
layer**: terrain, water, and labels/overlays. Each adjustment is
non-destructive (it never writes a field or a committed colour table) and is
undoable through the shared look history.

**Adjustment kinds** (from the row): **Levels** (in-black, in-white, gamma,
out-black, out-white), **Curves** (a fixed small number of control points,
monotone interpolation; the count is a judgement), **Hue/Saturation/Lightness**
in OkLCh (Ottosson 2020; hue rotates without shifting perceived lightness), and
**Gradient map** (luminance → an `ElevationRamp`-shaped stop table, reusing
`ElevationRamp::sample`). Each entry has `{kind, enabled, opacity 0..1,
params}`.

### ET-C1 — the adjustment model and its maths (Rust)

**Design.**
- A new `render`-adjacent module, `crates/cartalith-godot/src/adjust.rs`.
  `render.rs` is 12 325 lines; the `export_raster.rs` module doc argues the
  same split.
- It holds `AdjustStack { entries: Vec<Adjustment> }`, `AdjustStack::apply(c:
  Rgb) -> Rgb` in **continuous 0..255 `f64`, no clamp to bytes inside**, and
  `is_identity()`.
- Its tests compare against literal expected values. A test never asserts a
  constant against itself.

**Files.** New `adjust.rs` and its tests; one `mod` line in `lib.rs`.

**Untouched.** `finish_rgb`, every render path.

**Acceptance:**
- Each kind at rest is the identity, by control flow (`is_identity` true, and
  `apply` skipped by the caller).
- Levels with `(in 0..255, gamma 1, out 0..255)` returns its input bit-exactly.
- HSL hue +360° returns its input within 1e-9.
- A disabled entry contributes nothing, and an entry at opacity 0 likewise.
- Mutation: perturb each formula's one non-trivial constant, and a literal test
  goes red.

**Batch.** One Rust lane.

### ET-C2 — terrain and water stacks inside `finish_rgb`

**Design.**
- `finish_rgb` gains a fourth input: `layers: Option<(&AdjustStack,
  &AdjustStack, &[u8])>`, meaning terrain stack, water stack, and a per-pixel
  **water weight** `0..=255`.
- In `finish_px` the stacks run **after the grade and before the display
  matrix**. The display transform stays last, and still only the single
  quantiser exists: `c' = mix(terrain.apply(c), water.apply(c), w/255)`.
- The weight buffer is **empty** whenever both stacks are identity or the two
  are equal. In that case the per-pixel cost is one length check, the same rule
  `build_grade_influence` follows.
- **Why after the light and grade, not inside `land_color`/`sea_color_core`.**
  `cell_color` quantises to a byte before `finish_rgb`, so adjusting there would
  add a second quantisation, contrary to Ruling AN. It would also act before the
  hillshade multiply, so a user's Levels would be reshaped by lighting, which is
  not what adjusting the look of the water means.
- **Where `w` comes from: each consumer's own water decision, never a second
  classifier** (`MISTAKES.md` "Add a capability…": derive from every consumer
  path).
  - **Screen** (`build_color_texture`): the per-cell water classification the
    texture's own sea/lake branch uses. The antialiased coast is the shader's
    blend of two already-adjusted texels, so no fractional weight is needed.
  - **River colour texture** (`river_color`, *"the map with every river at full
    coverage"*): `w` is the river coverage that texture paints with, maximised
    with cell water.
  - **Export bake and banded export** (`BakeFields::pixel`, `bake_export_band`):
    the branch that chooses sea, lake (`bake_lake_at`) or river paint
    (`river_field::pixel_paint`).
  - **LOD tiles** (`render_biome_tile_rgba`): its own water branch.
- **Memory.** One byte per pixel on top of `export_raster.rs`'s measured 23 B/px
  `PEAK_BYTES_PER_PIXEL`. The gate's measurement must be **redone**, not
  incremented by assumption.

**Files.**
- `crates/cartalith-godot/src/render.rs` (`finish_rgb`, `finish_px`,
  `finish_raster`, the four call sites' weight plumbing)
- `crates/cartalith-godot/src/lib.rs` (`build_color_texture`, the river colour
  texture, `#[func] get/set_adjust_stack(layer)`)
- `crates/cartalith-godot/src/export_raster.rs`
- `crates/cartalith-godot/src/lod_bridge.rs` (tile caller)
- `crates/cartalith-godot/src/river_field.rs` (only if the weight is read
  there)

**Untouched.** `LayerStack` and its blend modes, the grade, `ColorSpace`,
`cell_color`, `land_color`, `sea_color_core`.

**Acceptance:**
- **Byte identity at rest:** `tests/color_space.rs`'s `FINISHED_RENDER_FNV1A`
  and `tests/layer_stack.rs`'s digests are unmoved, and every `golden_parity_*`
  is unmoved.
- A water-only Levels move changes bytes only where `w > 0`. Count the moved
  pixels inside land cells away from the coast band: it must be 0. Count inside
  water: > 0. Do this for **each** of the four consumers (screen, export, banded
  export, LOD tile) plus the river colour texture. A consumer that does not move
  is the `with_lakes` defect again.
- Screen equals export in the working space, byte for byte, with a non-identity
  water stack (extend `_exportraster_probe.gd` §13).
- The cache keys include the stacks: `lod_cache_key` and `river_paint_key`
  change when a stack changes, and a stale LOD tile after a stack edit is a
  failure.

**Verification.**
- `cargo test --workspace`.
- A windowed probe from a scratch copy: before/after captures at a pinned view.
  Assert on a palette-agnostic measure or a forced palette (`MISTAKES.md`
  "Assert on pixels").
- The export probe.
- The `.dll`/`.rs` mtimes stated.

**Risks.**
- The shore and river shader (`map_shore.gdshader`) blends texels and paints
  rivers from `river_color`. A seam at a river bank is possible if the base
  texture and `river_color` disagree on `w`. Acceptance: no colour step greater
  than 1 LSB across a river bank in a capture with a strong water-only
  adjustment (the RIM seam method).
- Cost: measure the stack pass at 2048² (median with min..max, run alone)
  against `finish_rgb` at rest.

**Batch.** Two lanes, file-disjoint:
- (1) `render.rs` and `lod_bridge.rs`;
- (2) `export_raster.rs`, the `lib.rs` texture builders and the `#[func]`s.

They are joined by a signature agreed in the brief. The verifier owns the
cross-consumer byte census.

### ET-C3 — the labels/overlays stack (GDScript)

**Design.**
- The same `AdjustStack` semantics are applied to every Godot-drawn ink at the
  points where `map_overlay.gd` already resolves one: `way_style_for`,
  `_treated_mix`, `_treated_rgb`, `icon_fill_for`, the label ink and halo
  functions, `journey_inks` and `restyle_factions`. The province border
  texture's ink is resolved in `viewport_host.gd`.
- There is one GDScript function, `adjust_overlay_color(c: Color) -> Color`. It
  is a port of `adjust.rs`'s formulas, and a mirror test pins the two to each
  other. That is the `river_field.rs`/`map_shore.gdshader` precedent.
- **Guards kept** (Ruling BX item 2): faction colours stay pairwise
  distinguishable, and label/halo contrast stays ≥ 4.5 (`_overlaystyle_probe.gd`'s
  existing bars). An adjustment that would break a guard is clamped back to the
  guard, and the panel says so.
- Exports inherit the adjustment through the overlay `SubViewport`.

**Files.**
- `godot-project/map_overlay.gd` (the resolution functions only)
- `godot-project/shell/viewport_host.gd` (border ink)
- a new `godot-project/shell/overlay_adjust.gd` (no `class_name`, `preload`ed;
  the `look_history.gd` precedent for avoiding the import pass)

**Untouched.** The Rust renderer and the treatment tables in
`render_workspace.gd`.

**Acceptance:**
- At rest, `style_treatment()` and every resolved ink are unchanged, value for
  value.
- A hue rotation moves road, label and border inks. The guards hold on all 17
  presets.
- The exported PNG's overlay pixels match the screen's (the existing export
  probe's overlay section).
- Mirror test: GDScript and Rust agree to 1/255 on a fixed table of 64 colours
  through each kind.

**Verification.** A windowed probe and the parse check.

**Batch.** One GDScript lane. It may run beside C2's Rust lanes only if the
brief proves file- and measurement-disjointness.

### ET-C4 — the stack panel, history and persistence

**Design.**
- A per-layer panel lives in the CARTO/Render workspace: a layer selector
  (Terrain · Water · Labels & overlays), an ordered list (add, remove, reorder,
  enable, opacity), and per-kind editors. The Curves editor reuses
  `ramp_bar.gd`'s gesture vocabulary.
- **History:** `look_history.gd` gains an `adjust` part in `snapshot()` and
  `apply()`, with one entry per gesture (Ruling BY item 2). A slider drag is one
  entry.
- **Persistence:** `AppearanceDoc` gains an optional `adjust` member, omitted
  when every stack is empty, so an untouched project stays byte-identical. This
  is `biome_cols_member`'s rule.
  - Saved looks (presets) carry the stacks as well; the "Look/preset" format is
    the owner's existing saved-look file.
  - The colour space stays out of both, by the standing "display = app, working
    space = document" rule.

**Files.**
- `godot-project/shell/workspaces/render_workspace.gd`
- `godot-project/shell/look_history.gd`
- `crates/cartalith-godot/src/project_bridge.rs` (`AppearanceDoc`)
- the saved-look reader and writer (locate at its symbol before briefing)

**Acceptance:**
- One drag is one history entry, and undo restores the stack and the map bytes.
- Save then open restores the stacks exactly.
- An untouched project's `appearance.json` is byte-identical before and after
  this milestone. A backward-compatibility test opens a project written before
  it.

**Verification.** A windowed probe (undo through the real accelerator *and* the
menu; `STATUS.md` records the accelerator path as unverified for the look
history), `cargo test --workspace` and the parse check.

**Batch.** Two lanes: (1) Rust `project_bridge.rs` plus the saved-look format;
(2) the GDScript panel plus `look_history.gd`.

## 6. Milestone E — colour management

**Standing constraint.** The default must leave sRGB byte-identical
(2026-08-31 ruling). Embedding a profile changes the **file** bytes of an export
but not its **pixel** bytes. Which of the two the existing export probes compare
must be checked at `_exportraster_probe.gd` before E1's acceptance is written.
OQ-3 asks the owner about the default.

**Library.**
- `moxcms` 0.8.1 is already in `Cargo.lock` as a dependency of `image` 0.25.10.
  It is BSD-3-Clause OR Apache-2.0 and pure Rust.
- Its source (read, not run) has `ColorProfile::new_srgb`, `new_display_p3`,
  `new_display_p3_pq`, `new_bt2020_pq` and `new_from_slice`. It has 8-bit
  transforms, a profile `encode()` (`writer.rs`) and CMYK⇄RGB support. It has
  **no built-in CMYK profile**.
- Using it directly needs a `Cargo.toml` edit in `cartalith-godot` and/or
  `cartalith-assets`, which this document may not make. The coordinator must
  authorise it.
- `png` 0.18.1's encoder (read) writes `sRGB` (`Encoder::set_source_srgb`),
  `gAMA`/`cHRM` and `iCCP` (`Info::icc_profile`).
- Its encoder has no cICP writer: grep found `cICP` only in the decoder and the
  chunk table. The raw `Writer::write_chunk` route is unverified.
- `image` 0.25.10's PNG encoder has `ImageEncoder::set_icc_profile` (read in
  `codecs/png.rs`).

### ET-E1 — tagged PNG exports

**Design.**
- Every PNG export carries colour information:
  - the `sRGB` chunk (rendering intent perceptual, a judgement) on the default
    path;
  - an `iCCP` with `moxcms`'s encoded Display P3 profile when the user exports
    in Display P3.
- That P3 export is a **new export option**, "Export colour space: sRGB /
  Display P3". It is not the monitor setting, because a document is not the
  monitor.
- The P3 export bytes come from the same `SRGB_TO_P3` re-encode the screen uses.
  Then the file and the screen agree on a P3 panel, which is the cost
  `export_raster.rs`'s doc names.
- Both encoders are covered: `export_stream.rs::write_png` and
  `cartalith_assets::raster::encode_png_rgb8`.
- The 16-bit heightmap PNG (`encode_png_luma16`) stays **untagged**: it is data.

**Files.**
- `crates/cartalith-godot/src/export_stream.rs`
- `crates/cartalith-assets/src/raster.rs`
- `crates/cartalith-godot/src/export_raster.rs`
- `crates/cartalith-godot/src/export_options.rs`
- the export dialog GDScript (locate at its symbol)

**Acceptance:**
- A default export decodes to **pixel bytes identical** to today's and contains
  exactly one `sRGB` chunk and no `iCCP`. Parse the chunk list in the test.
- A P3 export contains one `iCCP`, no `sRGB`, and a profile whose primaries,
  parsed back with `moxcms`, are the P3 primaries (`.68/.32 .265/.69 .15/.06`,
  D65) within 1e-4.
- A P3 export's pixels equal the screen's in P3 mode, byte for byte.
- The heightmap PNG has neither chunk.
- Mutation: drop the chunk write, and the chunk test goes red.

**Risks.** Third-party viewers differ. Note this in the dialog's hint, not in
the acceptance.

**Batch.** Two lanes: (1) the two encoders plus `export_raster.rs`; (2) the
export options plus the dialog.

### ET-E2 — importing images with profiles

**Design.**
- Asset-pack images (`decode_png` via `library.rs`) are converted to the sRGB
  working space at decode when they carry `cICP`, `iCCP`, `sRGB` or
  `gAMA`/`cHRM`. The order is the PNG-3 precedence: cICP > iCCP > sRGB >
  cHRM/gAMA.
- The rendering intent is relative colorimetric with black-point compensation (a
  judgement, matching PhotoCraft's gamut path).
- An untagged image is treated as sRGB and left byte-identical.
- **Heightmaps are never converted.** `decode_heightmap`/`import_heightmap` call
  a data decode that ignores colour chunks, and a test asserts that a P3-tagged
  heightmap imports to the same field as its untagged twin.
- If the slicer/asset path has a JPEG route, it follows the same rule; locate it
  at its symbol first.

**Files.**
- `crates/cartalith-assets/src/raster.rs` (a `decode_png_managed` beside
  `decode_png`)
- `crates/cartalith-assets/src/library.rs` (call site)
- `crates/cartalith-engine/src/import.rs` (switch to an explicit data decode;
  behaviour unchanged)

**Acceptance:**
- An untagged PNG decodes byte-identically to today.
- A P3-tagged pure (255, 0, 0) decodes to sRGB red *clipped*, since P3 red is
  outside sRGB. State the clipping policy (relative colorimetric clips) and test
  that it holds.
- The heightmap twin test passes.
- Existing asset-pack goldens are unmoved; asset fixtures are untagged, which
  must be checked first.

**Batch.** One Rust lane.

### ET-E3 — the print (CMYK) gamut warning in the picker

**Design.**
- A `#[func] color_print_gamut(c: Color) -> Dictionary` round-trips sRGB → CMYK
  (relative colorimetric) → sRGB and returns `{delta_e, in_gamut, nearest}`.
- It **omits every key** when no CMYK profile is loaded. It never reports "in
  gamut" for "not checked".
- `confirmable_color`'s was/now row gains a warning glyph plus a clickable
  nearest-printable swatch. That is PhotoCraft's picker pattern; Godot 4.7's
  `ColorPicker` has no CMYK mode, so no CMYK sliders are built.
- **The ΔE threshold is not adopted from PhotoCraft**, whose two files disagree
  (3.0 in the picker, 4.0 in `gamut.rs`, both as read via summary). This
  document recommends CIEDE2000 (not verified here) with the threshold put to
  the owner as part of OQ-2, or measured against the chosen profile's own
  round-trip noise floor on in-gamut colours.
- The profile source is OQ-2.

**Files.**
- a new `crates/cartalith-godot/src/print_gamut.rs`
- `lib.rs` (one `#[func]`)
- `godot-project/shell/dcc_widgets.gd` (`confirmable_color` only)
- `godot-project/shell/engine_bridge.gd`

**Acceptance:**
- Without a profile, the warning is absent: the probe asserts the glyph is not
  drawn, and the bridge dictionary is empty.
- With a profile, a known in-gamut neutral (128, 128, 128) is in gamut, and
  saturated sRGB blue (0, 0, 255) is flagged. Confirm that against the profile
  before pinning; it is expected for coated-paper profiles, not assumed.
- Clicking the swatch sets the picker to `nearest`, which is itself in gamut.
- Mutation: invert the threshold comparison, and both literal tests go red.

**Batch.** Two lanes: Rust (`print_gamut.rs`, `lib.rs`) and GDScript
(`dcc_widgets.gd`, `engine_bridge.gd`).

### ET-E4 — wide-gamut display everywhere, and HDR

**Design, the wide-gamut part (buildable).** `ColorSpace::DisplayP3` reaches the
terrain raster only; overlays and chrome stay sRGB numbers (`apply_color_space`'s
own stated boundary). Extend it to the map overlays: ET-C3's
`overlay_adjust.gd` applies the same `SRGB_TO_P3` encode last when the display
space is P3. The DCC chrome stays sRGB, and the panel says so.

**Design, the HDR part (blocked, not designed further here).**
- Godot 4.7 supports HDR output on Forward+ and Mobile only, **not** on
  Compatibility.
- This project ships `gl_compatibility`. `forward_plus` and `mobile` lose the
  device on the owner's RX 7800 XT (`3D_TERRAIN_RENDER_RESEARCH.md`).
- Android HDR is not in 4.7.
- HDR display is therefore unbuildable without a renderer change that has failed
  on the owner's hardware. OQ-1 asks what to do.
- The engine-side HDR step Ruling AN names (dropping `finish_px`'s per-stage
  clamps and writing a deeper encoder, e.g. 16-bit PNG plus cICP PQ) is
  renderer-independent. It is listed as a possible alternative in OQ-1, not
  scheduled.

**Files (wide-gamut part).** `godot-project/shell/overlay_adjust.gd` (from C3),
`godot-project/map_overlay.gd` (no new resolution points; C3's function is
reused).

**Acceptance.**
- In P3 mode, an overlay ink of sRGB (255, 0, 0) is drawn as the `SRGB_TO_P3`
  encode of it, matching the Rust value to 1/255.
- In sRGB mode nothing moves, value for value.

**Batch.** One GDScript lane. It depends on C3.

## 7. Milestone F — the Look control

**Goal (Ruling BX item 4).** Hue, saturation and brightness sliders, plus dark,
light and pastel variants of each style preset. The model is MapTiler's Quick
edits (sliders over the whole map or a layer group) and its reference-style
variants: *"the same level of information… different color schemes"*.

### ET-F1 — Look sliders as a quick edit over C's stacks

**Design.**
- The three sliders write **one HSL adjustment pinned at the top** of the
  selected groups' stacks: all groups (the default), or Terrain, Water or Labels
  & overlays. That is MapTiler's group selection.
- It is an ordinary C entry, so it is undoable, saved and guarded with no new
  machinery.
- "Brightness" is OkLCh lightness, not `grade_exposure`, so it acts the same on
  every group.
- The existing grade sliders are kept; the Look control is the quick surface
  over them, not a replacement.

**Files.** `godot-project/shell/workspaces/render_workspace.gd` (the control),
`godot-project/shell/look_history.gd` (label only).

**Acceptance:**
- At 0/0/0 the pinned entry is absent: the stack is unchanged, and so are the
  bytes.
- Saturation −100 % gives a capture whose terrain chroma (OkLCh C) is below
  1e-3 on every pixel; labels follow when "all groups" is selected. That is
  MapTiler's *"remove all the color"*.
- The faction and label guards hold.

**Batch.** One GDScript lane.

### ET-F2 — dark, light and pastel variants per preset

**Design.**
- A variant is a **rule**, not a hand-authored 51-row table (17 presets × 3). It
  is a transform applied to a preset's appearance bundle, overlay treatment and
  a pinned C entry:
  - *dark*: paper and ground lightness down, inks inverted toward light, the
    Night family's treatment;
  - *light*: monochrome-leaning, lowered chroma, the MapTiler "light"
    description;
  - *pastel*: chroma down and lightness up, *"tuned-down contrast"* per the
    MapTiler search snippet (that phrase was **not** found on a page fetched
    here).
- There is a per-preset override slot for any preset where a guard fails. See
  OQ-5.
- The UI is three chips under each style tile. An unknown variant falls back to
  the base preset, as MapTiler's SDK does.

**Files.** `godot-project/shell/workspaces/render_workspace.gd`
(`STYLE_PRESETS`, `_apply_preset`), `godot-project/map_overlay.gd` (no new
resolution points).

**Acceptance:**
- For all 17 presets × 3 variants (51 renders, windowed), labels meet ≥ 4.5,
  class inks ≥ 3.0 and factions stay pairwise distinct (`_overlaystyle_probe.gd`'s
  bars, reused).
- The mean OkLCh L of the dark variant is below the base, which is below the
  light variant. Calibrate the metric on two hand-checked presets first and put
  both numbers in its label (`MISTAKES.md` "Invent a screening metric").
- Base presets are unmoved: `_stylepresets_probe.gd` passes unchanged.

**Batch.** One GDScript lane, plus a verifier walking all 51 renders.

### ET-F3 — variants in history, saves and saved looks

**Design.** The preset part of `look_history.gd` records the variant. The project
and the saved look record `{preset, variant}`, omitted when there is no variant.

**Acceptance.** Undo across a variant switch restores the bytes. Save then open
restores the variant. An untouched project stays byte-identical.

**Batch.** Two lanes: Rust persistence and GDScript history.

## 8. Order, batches and gates

- **Order (Ruling BZ item 1):** D (D1 → D2 → D3 → D4 → D5), then C (C1 → C2 →
  C3 → C4), then E (E1, E2, E3, E4; E4 needs C3), then F (F1 → F2 → F3).
- **At most two agents per batch, file-disjoint** (standing instruction). No two
  batches at once, and no dispatch while a verifier runs.
- **Every batch's verifier checks:**
  - the `cargo test --workspace` total, quoted from the run;
  - a parse check of every touched `.gd` plus `shell/app.gd`;
  - a windowed probe from a scratch project copy with its own `config/name`;
  - `.dll`/`.rs` mtimes;
  - that every new probe fails against the pre-change tree;
  - the annotation rule (Ruling BK): every new function, type and constant
    documented, every constant sourced or labelled a judgement, every test
    opening with what it protects.
- **Goldens:** no milestone here moves a `golden_parity_*` test, and none has a
  ruling to. A milestone that finds it must stops and asks.

## 9. Risks across the whole scope

1. **Consumers missed.** C2 and E1/E4 each have four or five render/export
   consumers. The 2026-09-24 `with_lakes` defect (attached to both export paths
   and moved 0 bytes) is the pattern. Every acceptance above is a per-consumer
   byte census.
2. **Pressure from devices that do not report it.** If a mouse reports `0.0`,
   pressure-to-size would shrink every mouse dab to the minimum. ET-D2 measures
   the devices first, and absence stays absence.
3. **`lib.rs` and `map_overlay.gd` are shared, very large files.** Most batches
   touch one of them, so batches must be serial.
4. **Phone reachability.** Every new control must be reached through the real
   phone path (Ruling BR).
5. **Licence hygiene.** PhotoCraft is read as documentation only. Any
   adaptation needs attribution. `moxcms` and `png` are permissive.

## 10. What this scope does not touch

- generation, erosion, hydrology and civ;
- `LayerStack`'s compositing categories and blend modes;
- the grade's existing axes;
- `ramp_strength`'s default of 0 (Ruling BZ item 5);
- the heightmap undo stack;
- sculpt's thirteen feature formulas and `Falloff`;
- `civ_commit_way`'s routing;
- the renderer choice (OQ-1 asks rather than assumes);
- HDR encoders.

## 11. Owner questions, with recommended defaults

Each was searched for in `LARGE_ITEM_RULINGS.md` (terms `freehand`, `CMYK`,
`FOGRA`, `pastel`, `per stroke`, `HDR`, `gamut`) on 2026-10-11. None is already
ruled. The premises were read at the symbols cited in §3.

1. **HDR display.** Godot 4.7 offers HDR only on Forward+/Mobile. This project
   runs Compatibility because Forward+/Mobile lost the device on your RX 7800
   XT, and Android HDR is not in 4.7. Should "wide-gamut/HDR display" ship as
   wide-gamut only (ET-E4), with HDR display deferred until a renderer change
   works on your hardware? Or should the engine-side HDR step (16-bit PNG plus
   cICP export, no display) be built instead?
   **Default:** ship wide-gamut (ET-E4) now; defer HDR display; do not build the
   HDR export until asked.
2. **Which CMYK profile, and which ΔE threshold, for the print warning?** No
   profile is built into `moxcms`. The options:
   - (a) you load a press ICC file (accurate; nothing shown until you do);
   - (b) a synthesised generic coated profile shipped with the app (PhotoCraft
     ships a CC0 one: 300 % total ink, not press-characterised);
   - (c) a standard characterisation such as FOGRA39, whose licensing was not
     verified here.

   **Default:** (a) plus (b) as a labelled fallback ("generic, not a press
   profile"). CIEDE2000 with the threshold measured from the profile's in-gamut
   round-trip noise, proposed back to you.
3. **Should every PNG export carry an `sRGB` tag by default?** This changes
   every export file's bytes but not its pixels, and makes colour-managed
   viewers show it as intended.
   **Default:** yes, sRGB tag on by default; Display P3 export as an explicit
   option.
4. **Paint undo granularity.** Spacing turns one drag into many dabs. Should one
   stroke be one undo step?
   **Default:** yes, one step per stroke, matching Ruling BY's one entry per
   gesture.
5. **Dark/light/pastel variants: generated by rule, or hand-tuned per preset?**
   **Default:** generated by one rule per variant, with hand overrides only
   where a contrast or faction guard fails. That is 51 renders verified, not 51
   bundles authored.
6. **Road brush.** Drag-to-draw roads still route every leg by least-cost, as
   the reference does. The DCC spec's "freehand" routing mode was declined for
   having no reference algorithm, and `DECISIONS.md` §7p now permits new
   behaviour. Do you want freehand (straight, unrouted) roads as well?
   **Default:** no; keep least-cost routing and add only drag-to-draw, spacing
   and symmetry.
