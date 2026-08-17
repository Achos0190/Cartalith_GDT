# Asset Library (Phase 4): what it really is, and a milestone plan

`ROADMAP.md`'s Phase 4 is one sentence — "Block 3, the sprite and texture pack
system" — plus a "Confirm before starting" note, which the owner's own
direction to continue "until you've finished phase 4" satisfies. This document
is the investigation that sentence deferred, written the same way
`JOURNEY_PLANNER_SCOPE.md` / `GPU_LAYER_INTEGRATION_SCOPE.md` /
`TERRAIN_APPEARANCE_SCOPE.md` were: read the real reference code first, say
plainly how big the thing is, then break it into milestones that each stand on
their own.

Everything below was verified by reading `reference/Cartalith Gen1 v2.10.html`
directly — not the two design documents in `docs/` that predate the
implementation. Where those documents and the shipped code disagree, the code
wins and the disagreement is called out.

---

## 1. What an "asset" actually is

**Not** an arbitrary named image with free-form metadata. An asset is **one
PNG bound to one slot in a frozen, ordered vocabulary the engine already knows
how to draw**, plus an optional per-slot metadata record and an optional
per-item display transform.

There are eight families, seven of them closed vocabularies (reference lines
12029-12052, mirrored by the Asset Library's own `FAMILIES` table at line
~26781):

| Family | Manifest section | Slots | Bake | Anchor | Consumed by |
|---|---|---|---|---|---|
| Splat channels | `textures` | 7 | 512², opaque, seamless | tiled | `surfaceColor`'s splat blend + the parchment overlay |
| Biome ground | `biomes` | 15 | 512², opaque, seamless | tiled | painted Cartography biome layer (`_paintedTex`) |
| Terrain ground | `terrains` | 13 | 512², opaque, seamless | tiled | painted Cartography terrain layer |
| Feature icons | `icons` | 10 | 256², RGBA | **bottom** | `placeMapIcons` → `drawMapIcons` |
| Settlement pins | `structures.settlement` | 9 | 256², RGBA | centre | civ layer `_structSprite` |
| Settlement traits | `structures.trait` | 7 | 256², RGBA | centre | `_traitSprite` — **imported since v1.28, still not drawn** |
| POI markers | `structures.poi` | 8 | 256², RGBA | centre | `_customSprite`/`_featureSprite` |
| Custom icons | `custom` | **open** | 256², RGBA | centre | manual icon brush + rule-driven scatter |

Three properties matter more than the table:

- **Slots hold 1..N variants.** A ridge of forty peaks must not be forty copies
  of one drawing, so `pickIconVariant(x,y,seed,n)` picks deterministically by
  position hash. Same world ⇒ same icons ⇒ stable re-exports.
- **Order is load-bearing, twice over.** `PACK_BIOME_SLOTS`/`PACK_TERRAIN_SLOTS`
  are index-aligned 1:1 with the frozen `CART_BIOMES`/`CART_TERRAINS` paint
  vocabularies (slot N here is paint index N+1 there), and the structure lists
  mirror `CIV_SETTLEMENT_CLASSES`/`CIV_POI_TYPES`/`CIV_TRAITS` key for key.
  Reordering any list silently re-points every pack ever authored.
- **A missing slot is normal, not an error.** Every slot falls back to
  procedural art independently, so an icons-only or two-file pack is as valid
  as a complete one. This is the property that makes the whole subsystem
  optional rather than a dependency.

A real inconsistency worth carrying forward, found by reading rather than
assumed: the Asset Library's `poi` family has **ten** slots but the pack
*import* vocabulary has **eight** — `lake` and `bridge` have no engine POI kind
to attach to, so they can be authored and exported but never load. The
reference documents this in a comment at line 12033 and shrugs; this port
reproduces the same two lists rather than "fixing" one of them.

## 2. What an "asset pack" is as a format

A real, versioned serialization format — comparable in status to the world
save `SAVEFILE_COMPAT.md` documents, and now equally verified against live
code.

```
mypack.zip
├── pack.json      # manifest, schema 1 or 2 (or pack.csv; JSON wins if both)
├── textures/  biomes/  terrains/      # one PNG per slot
├── icons/  structures/{settlement,trait,poi}/   # slot_01.png, slot_02.png, …
└── custom/<setId>/                    # slot_01.png …
```

- **A plain PKZIP**, written by the same `zipStore()` the world save uses
  (STORE + raw DEFLATE since v1.90) and read by `unzipAny()` via the central
  directory. Nothing custom — the Rust `zip` crate reads it, exactly as
  `cartalith-io` already proved for world saves.
- **The manifest is the source of truth, not the folder layout.** Paths are
  ZIP-root-relative and may be anything; the directory names above are only
  what the exporter happens to write.
- **Schema 2 is a strict superset of schema 1.** A schema-1 consumer reads a
  schema-2 pack by ignoring what it does not know. Unknown keys anywhere are
  dropped *with a warning*, never rejected — so parsing a pack can only fail
  on a missing or malformed manifest, never on its content.
- **`pack.csv` is a real second input format**, not a design-doc suggestion:
  `parsePackCsv` (line 12093) ships. It is header/CRLF/blank-tolerant, carries
  a `variant` ordering column, and — unlike the JSON path — drops unknown slots
  *silently*. It predates `structures`/`custom` and cannot express them, nor a
  pack name/author/licence.
- **Warnings are ordered data.** `parsePackManifest` emits per-slot missing-file
  and unknown-slot warnings in a traversal order that partly follows the
  author's own key order (JavaScript iterates string keys by insertion). A UI
  reports the count next to the import summary and proceeds.

**Packs also travel inside a world save.** `_alExportEntries`/`_alImportProject`
write `assetlib/library.json` + `assetlib/img/N.png` into the project `.zip` —
a *second*, different serialization: the editable Library (per-slot metadata,
tags, collections, per-item transforms, scatter rules) rather than a baked
pack. `SAVEFILE_COMPAT.md` already lists "an Asset Library payload" among the
entries the MVP reader ignores; that entry is this. The two formats are not
interchangeable and both are real.

A deliberate non-format, worth stating so nobody looks for it: the live
`assetPack` global is **never serialized into `params.json`** (the reference's
transient-UI invariant 6). The Library's `assetlib/` payload is the one
persisted asset store.

## 3. How assets are actually used

Yes — the reference renderer really draws pack sprites onto the map, and has
for many versions; the vector glyphs are the *fallback*, not the other way
round.

The path, end to end:

1. **`placeMapIcons(fld, biome, W, H, opts)`** decides *where* glyphs go. Pure
   in the reference's own "amplifyRegion mold" — reads only its arguments plus
   the pure `hash()`. Two engines behind one entry point: a **legacy** path
   with the biome→slot mapping hard-coded, and **`placeMapIconsRuled`** (v1.26)
   which makes that mapping *data* — a `ScatterRule` per asset. Passing no
   rules keeps the legacy path bit-identical, which is what lets a pack-less
   map stay unchanged.
2. **`iconSlotForItem`** resolves a placed item to a slot key — the one place
   the flat vocabulary and the open custom vocabulary are unified
   (`custom::<set>::<slot>`).
3. **`iconVariantsFor` + `pickWeightedVariant`** choose the variant
   (position-hash, optionally weighted by the asset's own rule).
4. **`drawMapIcons`** composites: one Y-sorted painter's pass over every icon,
   `spriteDrawRect(x,y,s,base,sw,sh)` giving bottom-centre placement scaled to
   `base = max(3.5, W/110)` and the sprite's own aspect ratio. No pack art for
   that slot ⇒ `drawIconGlyph` draws the procedural vector version instead.

Ground textures take a different route entirely: `finalizePackTexture` stores a
per-channel inverse mean so splatting modulates a procedural material ramp by
`texel/mean`, while `biomes`/`terrains` deliberately **skip** that step and are
sampled as true colour (`_paintedTex`), because dividing out a tile's absolute
hue is right for splat and wrong for paint. That asymmetry is real, documented
in the reference at line 12246, and easy to get wrong in a port.

Phase 5's urban morphology does **not** consume packs today (checked: block 4
has no `assetPack` reference). The consumers are the terrain renderer, the civ
layer's settlement/POI drawing, and the Cartography manual-icon brush.

## 4. Portable vs. UI-only — the honest split

The same split every Phase 2 milestone investigation drew. Line counts are
measured, not estimated.

**Genuinely portable pure logic (~600-800 lines):**

- Pack manifest: `parsePackCsv`, `parsePackManifest`, `packSummary`, the six
  `PACK_*_SLOTS` vocabularies, `PackManifestBuilder`'s manifest half, `slugId`.
- Library model: `AssetDB`'s slot registry and custom-slot add/rename/remove
  (id slugging, uid collision handling, collection cascade), `AssetCollections`,
  `defaultMeta`, `AssetValidator.run()`'s rule set.
- Scatter rules: `defaultScatterRule`, `SCATTER_RULE_PRESETS`,
  `presetScatterRule`, `normalizeScatterRule` (with its real v1.27 hardening
  against untrusted project input), `scatterRuleKey`, `currentScatterRules`,
  `autopopulateScatterRules`, `pickWeightedVariant`.
- Placement/geometry: `placeMapIconsRuled`, `pickIconVariant`, `spriteDrawRect`,
  `iconSlotForItem`, `finalizePackTexture`'s inverse-mean maths.
- Project persistence: the `assetlib/library.json` record shape.
- Inside the slicer, a small pure core: cell rectangles from
  cols/rows/spacing/interior-line fractions, and the chroma key's Euclidean
  colour-distance test.

**Inherently UI/DOM-coupled (~900+ of block 3's ~1,439 lines):**

`AssetBrowserUI` (rail/grid/toolbar/search/multi-select/batch ops, ~72),
`InspectorUI` (~203), `ImageEditor` (~30), the `SpriteSheetImporter` **modal**
(~408 — by far the largest single module in the block, and almost entirely
drag/canvas/pointer interaction around that small pure core), the
`AssetLibrary` page controller (~225), `renderPackInspector`, `toast`,
`UIState`'s `localStorage`, drag-and-drop intake, preview backdrops.

**Neither — platform work, not a port:** image decode (`decodePackImage`,
`AssetImporter.decodeBytes`), thumbnail/export rasterisation
(`ThumbnailRenderer`, `encodeItemPng`), ZIP read/write. In Rust these are the
`image` and `zip` crates plus Godot's own `Image`/`ImageTexture`, not
hand-ported logic — `PROVENANCE.md`'s "take a crate for anything downstream of
the pixels" rule applies cleanly.

## 5. How big Phase 4 actually is

**Honestly: large — roughly 70% of the Journey Planner by raw size, but with a
much smaller portable core and a much larger UI surface.**

- Block 3 (the Asset Library page): lines 26723-28161, **~1,439 lines**.
- Block 1's asset-related regions: scatter rules + icon placement/drawing
  (~6895-7420) and pack parse/load/inspector (~12028-12330), **~800 lines**.
- Block 2's consumers (`_structSprite`, `_traitSprite`, `_customSprite`,
  `_featureSprite`, `_carIconBrush*`, the icon gallery/editor UI): several
  hundred more, most of it UI.

Total ≈ **2,250+ lines** against the Journey Planner's ~3,100. But where the
Journey Planner was ~70 functions of *dense portable modelling*, Phase 4 is
maybe 600-800 lines of portable logic wrapped in 1,000+ lines of editor UI and
a platform layer of image/ZIP handling that is crate work rather than porting.

So: **it is a real sub-phase, not a milestone.** It does not need its own phase
the way the Journey Planner does, but it is not a one-pass job either, and
anyone estimating it from `ROADMAP.md`'s single sentence will be wrong by an
order of magnitude. Sequenced below in seven milestones.

## 6. Milestone breakdown

### Milestone 1 — pack manifest model, parsing, validation, serialization: done (2026-08-17)

New crate **`cartalith-assets`** (no `gdext`, no dependency on any other
Cartalith crate — the standalone shape `cartalith-spatial` set). Deliberately
the piece with no images, no archive, no renderer and no UI in it, and the
piece every later milestone is defined against.

Shipped:

- `slots.rs` — all seven frozen vocabularies verbatim, plus a `Family` enum
  carrying each family's manifest section, export directory, bake size,
  opacity, anchor and multi-variant flag (the reference's `FAMILIES` metadata),
  `Family::asset_path` (the exporter's own path convention) and `slug_id`.
- `manifest.rs` — `RawManifest` (as authored, key order preserved) and
  `PackManifest` (validated), `parse_pack_csv`, `parse_pack_manifest`,
  `parse_pack_entries` (the `parsePackManifest(zip)` equivalent, taking entry
  names so the crate needs no archive dependency), `pack_summary`, `to_raw` /
  `to_pack_json` for schema-2 export, `referenced_files`, and a `PackError`
  whose `NoManifest` message is the reference's own string.
- `ordered_map.rs` — a small insertion-ordered map. Not incidental:
  the reference's unknown-slot warnings are emitted by iterating the author's
  own objects, and JavaScript iterates string keys in insertion order, so
  warning order is a function of how the pack was written. `BTreeMap` would
  sort it away; serde_json's `preserve_order` feature would have leaked into
  `cartalith-io` through workspace feature unification. ~40 lines instead.

**Golden-verified against the real reference** — a real execution path exists
and was used, not stood in for. A transient Node `vm` harness (same technique
as `cartalith-civ`'s golden tests) extracts `parsePackCsv`/`parsePackManifest`/
`packSummary` plus their six vocabularies from the frozen HTML by line range
and runs them on five fixtures; the expected values in
`tests/golden_parity_pack_manifest.rs` are that run's output verbatim. Every
case matched on the first run.

The fixtures deliberately target what a rewrite gets *plausibly* wrong rather
than the happy path: a missing texture file, an unknown texture slot, an
unknown biome slot that is really a *terrain* slot, one missing icon variant
(slot survives) vs. all variants missing (slot dropped whole), a bare string
standing in for a one-element variant list, an unknown settlement slot, a
missing custom-set variant, CSV variant ordering as a *stable* sort with
unnumbered rows pushed to the end, JSON winning over CSV when both are present,
an empty-string path counting as a missing file, and the exact wording and
ordering of all nine resulting warnings.

28 tests total (18 unit + 9 golden + 1 doctest). **Not wired to anything** —
same "don't wire in what nothing calls" discipline as `cartalith-spatial` and
every unwired Phase 2 primitive.

### Milestone 2 — pack ZIP read/write

`unzipAny`/`zipStore` in Rust terms: read a real `.zip` pack into
`parse_pack_entries`, and write one back. Depends on the `zip` crate, which
`cartalith-io` has already proved against real reference exports. Lands either
in `cartalith-assets` behind a feature or in `cartalith-io` — decide from
ergonomics when it starts, not now. Verification should include a real
round-trip against a pack the reference itself exported.

### Milestone 3 — scatter rules

`ScatterRule` + `defaultScatterRule`/`SCATTER_RULE_PRESETS`/
`presetScatterRule`/`normalizeScatterRule`/`scatterRuleKey`/
`currentScatterRules`/`autopopulateScatterRules`/`pickWeightedVariant`. Pure,
self-contained, and carrying real, documented v1.27 hardening against untrusted
project input (a NaN `density` scattered on *every* cell; a NaN `spacing`
collapsed an O(1) neighbour test to O(n²); an `Object.assign` aliasing bug made
every fallback read the garbage it was supposed to replace). Port those fixes
as fixes, with tests that name them.

### Milestone 4 — rule-driven icon placement

`placeMapIconsRuled` + `iconSlotForItem` + `spriteDrawRect`. Pure, needs
`cartalith-noise`'s `hash` — the first milestone with a cross-crate dependency,
and the first with real golden-parity surface (placement is positional and
seeded, so it diffs exactly). Note the two v1.27 fixes inside it: scatter-rule
priority is sorted most-specific-first so the winner is not insertion-order
dependent, and `requireWetland` is ANDed with the biome test rather than
replacing it. Depends on milestone 3.

### Milestone 5 — the Library model

`AssetDB`'s slot registry, custom-slot add/rename/remove with id slugging and
uid-collision handling, `AssetCollections`, per-slot metadata/tags,
`AssetValidator.run()`, and the `assetlib/library.json` record shape with its
`normalizeScatterRule`-on-load behaviour. Pure data management; no images.
Depends on milestones 1 and 3.

### Milestone 6 — image handling

Decode, per-item transform (scale/pan/`fitToBottom`), thumbnail and export
bake, `itemHash` duplicate detection, `finalizePackTexture`'s inverse means.
Crate work (`image`) plus a thin port, not a hand-port. First milestone that
touches pixels.

### Milestone 7 — renderer + Godot integration

Sprite compositing into the map render (`drawMapIcons`' painter's pass and
per-slot procedural fallback), ground-texture sampling for splat and the two
painted layers, and only then any UI. Genuine Phase 3-adjacent rendering work;
the UI on top of it is a `GUI_SHELL_SCOPE.md` job and should re-run
`ui-ux-pro-max` rather than bolting controls on (`MEMORY.md`'s own standing
note).

## 7. Out of scope for all milestones above

- **The Asset Library page UI itself** — browser rail/grid/toolbar/search/
  multi-select, the inspector, drag-and-drop intake, toasts, `localStorage` UI
  state, preview backdrops. `ARCHITECTURE.md`: Godot owns presentation.
  `design/cartalith-menu-structure.md` §6 already names the real control
  inventory (Library select/tag/collect/rename/duplicate/delete/clear; sprite
  sheets; pack name/author/licence, validate, import, export) — that is a GUI
  milestone, not a port.
- **The sprite-sheet slicer's canvas interaction** — draggable grid rectangle,
  interior line handles, eyedropper, live preview. Its pure core (cell
  rectangles, chroma-key distance) is portable and can ride along with
  milestone 6; the modal is not.
- **Authoring-side conveniences** the reference itself calls authoring-only:
  the standalone `asset_pack_compiler.html`, per-cell naming UI, the preview
  backdrop swatches.
- **Wiring anything into `compute_civilisation()` or the Godot shell** before
  milestone 7 — the same discipline the Journey Planner milestones follow.

## 8. Done means

A real `.zip` asset pack authored outside the app can be imported, validated
with the reference's own warnings, and rendered onto the map — sprites for the
slots it carries, procedural art for the slots it does not — with a pack-less
render staying bit-identical to today's. The Library workspace that *authors*
such a pack is a separate, later GUI effort tracked in `GUI_SHELL_SCOPE.md`.
