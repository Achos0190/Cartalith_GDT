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

### Milestone 2 — pack ZIP read/write: done (2026-08-17)

`unzipAny`/`zipStore` in Rust terms: read a real `.zip` pack into
`parse_pack_entries`, and write one back.

**Placement, decided by reading rather than by the coin-toss this section
originally left open: `cartalith-assets`, module `archive`, behind an
on-by-default `zip` feature.** The open question was whether to put it in
`cartalith-io` instead, or extract a shared helper. Reading `cartalith-io`
first settled it:

- **There is nothing to share.** `cartalith-io`'s entire "zip handling" is
  `ZipArchive::new`, `by_name`, `read_to_end` and a `MissingEntry` error
  variant — the `zip` crate *is* the shared helper the reference's own
  `unzipAny`/`zipStore` pair was. A common wrapper over that would be a
  wrapper around a wrapper; milestone 1's "packs use the same `zipStore()`
  the world save uses" finding is true and, precisely because it is true,
  implies **no shared code**, only a shared crate.
- **`cartalith-io` writes nothing, on purpose.** `MVP_SCOPE.md` point 12 and
  `SAVEFILE_COMPAT.md`'s own "Deferred" section make it reading-only.
  A pack *writer* there would break that crate's stated boundary, and it is
  the writer where the reference's real quirks live.
- **The dependency would point the wrong way.** Putting packs in
  `cartalith-io` makes it depend on `cartalith-assets`, so every consumer of
  the world-save loader drags in the asset vocabulary. Packs are the optional
  subsystem; the save loader is not.
- **The feature keeps milestone 1's promise literally true.**
  `default-features = false` gives back exactly the archive-free manifest
  model, and it is tested that way (`cargo test -p cartalith-assets
  --no-default-features`) rather than merely asserted.

**Reference quirks found and preserved** (the zip layer has its own, as
expected):

- **`.png` entries are STORED, never deflated** — a PNG is already internally
  DEFLATE-compressed, so re-compressing it is wasted CPU for no gain. The
  reference says so in its own comment; the port applies the same rule by
  filename extension, case-insensitively.
- **Timestamps are frozen at 1980-01-01 00:00:00.** `zipStore` hardcodes the
  DOS date word to `0x0021` and the time word to `0`. That makes exports
  byte-reproducible, and it is *not* what the `zip` crate does by default (it
  uses the wall clock), so the port sets it explicitly.
- **`pack.json` is written last**, after every image — the exporter appends it
  once its family walk is done. Not semantically load-bearing, but it is what
  a reference-written pack looks like.
- **Names are read verbatim.** No wrapping-folder stripping, no backslash
  rewriting. This is why zipping the *folder* instead of its *contents* yields
  a pack whose manifest is at `MyPack/pack.json` and is therefore not found —
  a real, reported failure (the reference's own error message says "try
  re-zipping the folder…"), preserved rather than papered over.
- **Directory entries are kept** as zero-byte members, because `unzipAny`
  walks the central directory and stores what it finds. Harmless: no manifest
  path ends in `/`.
- **An unrecognised compression method is an error**, worded exactly as the
  reference words it (`unsupported zip method 93 for pack.json`), not a
  silently skipped entry.

Two deliberate non-ports, stated rather than smuggled: `zipStore`'s extra
"…and only if the compressed bytes actually came out smaller" fallback (a
browser-side size/`CompressionStream`-availability concern that no reader can
observe), and `unzipStore`, which is `unzipAny`'s fallback for an archive with
no readable central directory and answers `null` for every deflated entry —
a browser-quirk defence, not a format variant. `zip::ZipArchive` requires the
central directory and errors cleanly without it, which is the better answer.

**Verified against a pack the reference itself exported, in both directions.**
The harness runs the reference's *own* `PackManifestBuilder.build()` (line
26964) over its *own* `FAMILIES`/`AssetDB` vocabulary and its *own*
`zipStore()` (line 12009) headlessly under Node's `vm.runInContext`, all
lifted verbatim by line range from the frozen HTML. Only two things in that
run are not reference code, and the test file says so: `renderToBlob` is a
canvas rasteriser, replaced by a real PNG encoder emitting genuine PNGs at each
family's own bake size, and the three DOM inputs `E('alPackName'|…)` are
stubbed with real values. Everything else — filenames, entry order, which
entries are stored vs. deflated, the frozen timestamps, the manifest's exact
JSON text, every CRC-32 — is the reference's own output, checked in as
`tests/fixtures/reference_pack.zip` (18 entries, 21 KB) alongside that run's
`unzipAny`/`parsePackManifest`/`packSummary` capture.

- **Read**: this port's entries match the reference's `unzipAny` output name
  for name and CRC for CRC; its `parse_pack_entries` reproduces the summary and
  the one warning; and `to_pack_json()` reproduces the exporter's `pack.json`
  **text byte for byte**.
- **Write**: `write_pack` reproduces the reference archive's entry order,
  per-entry method, CRC-32, uncompressed size and 1980 timestamps; and the
  bytes were fed back through the reference's own `unzipAny` +
  `parsePackManifest`, which read all 18 entries with identical payloads,
  an identical `pack.json`, and an identical summary and warning list. The two
  archives differ by 2 bytes in total; the first differing byte is the
  version-needed-to-extract field. Exact byte equality is not achievable and
  is not the bar — the single deflated entry is compressed by `miniz_oxide`
  here and by the browser's zlib there, and two conforming encoders need not
  agree on a bit stream.

14 new tests (4 golden-parity + 10 unit). Still wired to nothing.

**Corrections to this document that the milestone surfaced:**

- §4 files "ZIP read/write" under *"Neither — platform work, not a port"*.
  That is three-quarters right and one-quarter wrong: the *container* is pure
  crate work, but the reference's **export policy** — STORE the PNGs, freeze
  the timestamps, write `pack.json` last, never normalise a name on read — is
  real ported behaviour that a plain `zip` call gets wrong by default, in the
  timestamp's case actively so. Roughly 60 lines of policy over a crate, not
  zero.
- Milestone 5 (the Library model) must keep **both** the raw set name and its
  slug on a custom slot. Confirmed by watching the real exporter run: the file
  path uses the slug (`custom/naval/lighthouse_01.png`) while the manifest key
  is the author's own text (`"custom": {"Naval": …}`). Losing either makes a
  round-trip lossy, and `AssetDB.addCustomSlot` really does carry both
  (`slot.set` and `slot.setId`).
- `packSummary`'s trailing "*N* custom icon(s)" counts custom **slots**, not
  variants — a two-variant lighthouse reads as "1 custom icon". Already
  matched by milestone 1's port; noted here because it looks like a bug and is
  not.

### Milestone 3 — scatter rules: done (2026-08-17)

`cartalith-assets`, module `scatter`: `ScatterRule` + `ScatterMode`,
`Default` (`defaultScatterRule`), `preset_scatter_rule` (the ten
`SCATTER_RULE_PRESETS` inline), `normalize_scatter_rule`, `scatter_rule_key`,
`current_scatter_rules`, `autopopulate_scatter_rules`,
`pick_weighted_variant` and `pick_icon_variant`. Pure and self-contained, and
still wired to nothing.

**The v1.27 hardening, ported as fixes rather than transcribed.** Rules are
read out of `assetlib/library.json` inside a *user-supplied project `.zip`*, so
every field reaching the normalizer is untrusted. v1.26 merged it with the
`+x||fallback` idiom, which lost a legitimate `0` (falsy) and let a `NaN`
propagate instead of rejecting it. A Rust port has different natural failure
modes, so each of the three named failures was re-derived here rather than
guarded by reflex — `tests/hardening_v1_27.rs` has one test per fix, each
reproducing the *downstream* arithmetic inline (four lines, lifted from
`placeMapIconsRuled`) so the test shows the failure it prevents rather than
asserting a value:

1. **`NaN` density scattered on every cell — still a real hazard, by the
   opposite IEEE rule.** The JS predicate is `keep >= Math.min(1, density)`
   and `Math.min(1, NaN)` is `NaN`, so nothing is ever rejected. Rust's
   `f64::min` *absorbs* NaN (`f64::min(1.0, NAN) == 1.0`) — but `keep` is a
   hash in `[0,1]`, so `keep >= 1.0` is false anyway and the corrupt rule
   still carpets the map. Same catastrophe, opposite mechanism; rejecting
   non-finite input at the boundary closes both.
2. **`NaN` spacing collapsing an O(1) neighbour test to O(n²) — real, and
   `f64::max` would have masked it.** `Math.ceil(W/NaN)||1` gives a 1×1 bucket
   grid, so `fits()` degenerates from a nine-bucket lookup into a scan over
   every icon placed so far. Rust's NaN-absorbing `f64::max` would rescue the
   derived-spacing path *by accident*; the explicit `is_finite` check is kept,
   because an implicit dependency on an IEEE corner is exactly what this fix
   existed to remove — and fix 1 above shows how little that intuition can be
   trusted.
3. **The `Object.assign` aliasing bug — structurally unreachable, and not for
   the reason one would guess.** It is not "Rust's ownership rules": the bug
   needs the defaults and the untrusted input to inhabit *one mutable object*,
   and here they are different **types** — `base` is an owned `ScatterRule`
   with an `f64` field, the input is a `serde_json::Value`. There is no
   merge-in-place operation to get wrong because a `"x"` can never be stored
   in the field it would have to corrupt. **No defensive code was written for
   it.** The test pins the reference's own probe case (`{minSize:"x",
   maxSize:2}` must give the preset's `0.55` and a surviving `2`) so a future
   refactor toward a "merge" helper fails loudly, and adds a
   nothing-poisons-anything sweep over an all-garbage record.

A fourth guarantee this port has and the reference cannot: `ScatterRule`
implements `Serialize` but **deliberately not `Deserialize`**. The hardening
is not bypassable by a future caller reaching for `serde_json::from_str` —
`normalize_scatter_rule` is the only door in. Untrusted input is typed as
`&serde_json::Value` for the same reason.

**Golden-verified against the real reference**, the same transient Node `vm`
technique as milestones 1-2: all nine functions plus `hash` lifted out of the
frozen HTML by line range and run on the fixtures. `pick_weighted_variant` is
deterministic-hash-driven, so it diffs **exactly** — an 11-case × 36-position
sweep matched index for index, including the three degenerate weightings that
must fall through to `pickIconVariant`'s untouched v1.25 hash. 37
`normalize_scatter_rule` fixtures cover the JavaScript idioms a rewrite gets
plausibly wrong, and one did catch a real bug on the first run: **`density`'s
fallback is not symmetric with the other numeric fields.** The reference
merges first and *then* runs `num(out.density,0,3,1)`, so an absent `density`
keeps the slot preset's own value (`cactus` stays 0.35) while a *rejected* one
lands on a literal `1`. Every other numeric field falls back to the preset in
both cases. Nothing but a golden run would have found that.

24 new tests (11 golden + 4 hardening + 9 unit).

**Corrections to milestone 4, which depends on this one:**

- **Milestone 4 is not "the first milestone with a cross-crate dependency" —
  milestone 3 is.** `pickWeightedVariant` falls through to `pickIconVariant`,
  which is `hash`, so `cartalith-assets` already depends on `cartalith-noise`.
  Reimplementing that hash locally to preserve milestone 1's "no dependency on
  any other Cartalith crate" property would have been the worse trade by a
  wide margin (`cartalith-noise`'s `hash` carries two hard-won JS float
  subtleties in its own doc comment).
- **`pickIconVariant` shipped here, not in milestone 4.** §4 files it under
  "Placement/geometry"; it is three lines and `pickWeightedVariant` cannot be
  golden-tested without it.
- **`spaceOf`'s half of v1.27 fix 2 shipped here too**, as
  `ScatterRule::spacing_cells(map_width)`. The fix is two-sided — reject at
  the boundary, and guard the *computed* value for callers that bypass the
  boundary — and leaving half of a named fix to a later milestone would have
  made it untestable here. Milestone 4's `placeMapIconsRuled` calls the
  method. It reproduces two reference quirks: a density of exactly `0` derives
  spacing as if it were `1` (`+0||1`), and the floor of 3 cells.
- **Milestone 4's own two v1.27 fixes are confirmed still its own** — the
  most-specific-first priority sort and the `requireWetland` AND both live
  inside `placeMapIconsRuled`'s scatter branch (reference lines 7258-7271), not
  in the rule model.
- **`biomes` is `Vec<f64>`, so milestone 4's `biomeOk` compares against
  `biome[i] as f64`.** Not an aesthetic choice: the reference filters the list
  with `Number.isFinite`, which does not coerce, so a `"4"` is dropped while a
  hand-edited `5.5` is **kept** and simply never matches. Truncating to `i32`
  would make it start matching, and would rewrite the author's file on the
  next `library.json` round trip.
- Milestone 4 also needs the legacy `TREE_SLOT`/`SCATTER_SLOT` kind→slot maps
  (reference lines 7281-7283) for `iconSlotForItem`'s non-ruled branch. Small,
  but not currently named anywhere in this document.

### Milestone 4 — rule-driven icon placement: done (2026-08-17)

`cartalith-assets`, new module `placement`: `place_map_icons_ruled`
(`placeMapIconsRuled`, reference line 7194), `icon_slot_for_item` with the
`TREE_SLOT`/`SCATTER_SLOT` legacy fallback maps (7289-7300), and
`sprite_draw_rect` (12173). Pure, and the first milestone in this crate with
real golden-parity *placement* surface: positional and seeded, so it diffs
**exactly** rather than within a tolerance. Still wired to nothing.

**The legacy (non-ruled) `placeMapIcons` body is out of scope, on purpose.**
The reference only enters `placeMapIconsRuled` when `opts.rules` is
non-empty; its own hard-coded v1.25 biome-switch body is untouched code this
milestone's scope never named, and `current_scatter_rules` (milestone 3)
already reproduces the empty-table condition under which the reference falls
through to it. `icon_slot_for_item` is still ported in full — including its
legacy `cat`/`kind` branches and the `TREE_SLOT`/`SCATTER_SLOT` maps
milestone 3's corrections flagged as this milestone's remaining work — since
it is the one function a legacy-shaped item and a ruled item would both have
to agree with, even though this crate's own placement engine never produces
the former.

**Both named v1.27 fixes, checked with the same scrutiny milestone 3 applied
to its three (one of which it found structurally unreachable in Rust) — and
both of these transfer, because both are real logic defects, not JS-coercion
artifacts:**

1. **Most-specific-first priority sort** (reference lines 7250-7259, ported as
   `specificity`). Before v1.27 a contested cell's winner was whichever rule
   the caller's array happened to list first — which, since the table comes
   from iterating an object, meant "whichever order the user added assets to
   the Library in." **Structurally necessary in Rust too**: nothing about
   ownership or types removes insertion-order dependence from a `Vec` any
   more than from a JS array — ordering was always a real, ported
   `sort_by_key`, not a JS artifact.
2. **`requireWetland` ANDed with the biome test, not substituted for it**
   (reference line 7273). v1.26's scatter branch let `requireWetland`
   *replace* `biomeOk` outright, silently discarding a rule's biome
   restriction whenever wetland was also required. **Structurally necessary
   in Rust too**: an algorithm/predicate defect, not a consequence of JS
   type coercion or `Object.assign` aliasing (the two mechanisms behind two
   of `scatter.rs`'s three v1.27 fixes) — a straight transcription of the
   old "replace" logic reproduces the bug in any language.

Proven with a hand-traceable fixture rather than left to a broad sweep's
chance coverage: a 3x1 grid, `sea=-1` (every cell counts as land), `tGap=1`.
The last choice is the trick — `hash(*)` is always in `[0,1)`, so
`(hash(gx,gy,seed)*1)|0` is always `0`, meaning the scatter grid's own jitter
degenerates to zero and `jx=gx, jy=gy` exactly for every cell (checked
against the real reference `hash`, not assumed). Three cells — wetland+grass,
dry+grass, wetland+shrub — and three rules (`wetland_grass`: wetland AND
grass; `narrow_biome`: grass only; `generic_land`: any land) inserted
**least-specific first** resolve to `wetland_grass` / `narrow_biome` /
`generic_land` across three seeds, unchanged when the whole rule array is
reversed. The third cell is fix 2's own proof: it is wetland (would have
satisfied the pre-v1.27 OR/replace semantics) but the wrong biome, so
`wetland_grass` is correctly rejected and the cell falls through.

**Golden-verified against the real reference**, the same transient Node `vm`
technique as milestones 1-3. A synthetic 10x8 grid (single circular
elevation peak, biome cycling through `(x*3+y*5)%14`, wetland mask on
`(x+y)%4==0`) run through an eight-rule table across six sea/seed/density
configurations matches cell-for-cell, key-for-key, and size-for-size to
1e-9 — including one configuration that exercises every rule family at
once (both relief bands sharing one bucket grid, including an unbounded
`elevMin:null` relief rule; three different scatter specificities winning
different cells; and a `ghost_biome` rule with `biomes:[5.5]` placing
**nothing**, anywhere, confirming `biomeOk`'s `biome[i] as f64` cast: a
non-integer rule biome is finite so nothing rejects it at the normalizer
boundary, but it simply never equals an integer `BIOME_INDEX`).

23 new tests (12 unit + 11 golden).

**Corrections to milestones 5-7 found on this read: none.** `TREE_SLOT`/
`SCATTER_SLOT` were already flagged by milestone 3's own corrections as this
milestone's remaining, previously-unnamed work, and that is exactly where
they landed — no further scope drift found.

### Milestone 5 — the Library model: done (2026-08-17)

`cartalith-assets`, new module `library`: `AssetDB` (slot registry, item
store, `add_custom_slot`/`rename_custom_slot`/`remove_custom_slot`,
`slot_rules` lazy attach, `clear`), `AssetCollections`, `run` (the reference's
`AssetValidator.run()`), and the `assetlib/library.json` record shape
(`LibraryFile`/`SlotRecord`/`ItemRecord`, `parse_library_json`,
`AssetDB::to_library_json`/`apply_library_file`). Pure data management; no
images — `LibraryItem.hash` is always caller-supplied (a test fixture today,
milestone 6's real `itemHash`-equivalent later), which is what keeps the
validator's duplicate-image detection fully implementable and
golden-testable without decoding a single PNG. Depends on milestones 1 and 3,
confirmed: `library` reuses `Family`/`slug_id` (1) and `ScatterRule`/
`normalize_scatter_rule`/`scatter_rule_key`/`preset_scatter_rule` (3)
directly rather than re-deriving any of them.

**The `library.json` record shape, and how it lines up with
`SAVEFILE_COMPAT.md`'s existing cross-reference.** `SAVEFILE_COMPAT.md`
already lists "an Asset Library payload" among the entries its MVP reader
ignores, with the note "there is nothing in the port to deserialise them into
yet." `LibraryFile` is that something, now real: `{version, kind, pack:
{name, author, license}, collections: {name -> [uid]}, slots: [{fam, id,
name, meta, items: [{img, name, t}], set?, rules?}]}` — field order matching
a real `_alExportEntries()` export exactly (verified against a captured
reference run, below). `SAVEFILE_COMPAT.md`'s own note stands as written and
needed no correction: `cartalith-io` still deserialises nothing here (this
crate has no dependency on it, and the reverse would be the wrong direction
per milestone 2's own reasoning), so the MVP reader's ignore-list is still
accurate. What changed is only that a real, tested Rust shape for that
payload now exists in `cartalith-assets`, for milestone 6/7 (or a future
`cartalith-io` extension) to read into rather than design from scratch.

**A correction to this document itself, found by reading rather than
assumed: per-slot display *names* are not purely presentational after all.**
§4 filed `mkSlots`'s `name`/`desc`/`code` columns as UI-only text, and that
holds for `desc`/`code` (genuinely never read outside the browser UI) — but
`AssetValidator.run()`'s "Identical images" warning renders `slot.name`, not
`slot.id` (`SLOT_REG[e.uid].slot.name+'#'+(e.idx+1)`), confirmed by a golden
run: `"Identical images: Mountain#1 = Hill#1"`, not `mountain#1 = hill#1`.
This milestone therefore ports the `mkSlots` title table too (`slot_title`,
65 entries across six frozen families), the one piece of "presentational"
data that turned out to be load-bearing for golden parity.

**A second correction, also found by reading rather than assumed: the
Library's own `poi` vocabulary is ten slots, not the eight
[`PACK_POI_SLOTS`] milestone 1 ported.** `AssetDB` bootstraps a family from
the Asset Library's own `FAMILIES` table, not from the pack-import
vocabulary `parsePackManifest` validates against — and `FAMILIES[...].slots`
for `poi` carries `lake`/`bridge` in addition to the eight `PACK_POI_SLOTS`
already document as the pack-import subset. Both lists are real and now both
exist (`LIBRARY_POI_SLOTS`, ten; `PACK_POI_SLOTS`, eight, unchanged) rather
than one being "fixed" to match the other — reproducing the same
`lake`/`bridge`-import-but-never-load inconsistency §1 already named.

**The id-slugging and uid-collision hardening asked for by name, checked for
rather than assumed absent.** `addCustomSlot`/`renameCustomSlot` carry real
defensive logic in the reference's own code — `addCustomSlot` returns the
*existing* slot on a uid collision rather than creating a second one or
overwriting the first (`const existing=...find(...); if(existing) return
existing;`), and `renameCustomSlot` refuses a colliding rename outright,
keeping the *old* uid rather than clobbering the rename target
(`if(SLOT_REG[nuid]) return uid;`). Unlike v1.27's fixes, **neither carries a
version-tagged comment** — there is no `/* vX.YY fix */` marker to point at,
so this is reported as a finding rather than a named historical fix. Both
are ported faithfully and pinned with tests explaining the *why* (untrusted,
free-form user text slugging to a collision is a real hazard for content
editable outside the app, not a hypothetical) in `tests/hardening_asset_db.rs`.

That same file also documents a companion finding: two of `run`'s six checks
— "Duplicate identifier" and "Invalid filename id" — are **structurally
unreachable through this module's own public API, in both languages**, for
a reason that is not "Rust's type system" (the same shape of surprise
milestone 3's fix #3 found for the `Object.assign` aliasing bug). Ported
anyway, faithfully, as real defence-in-depth. A third check, "Collection
references a missing asset," is reachable but *only* via
`AssetCollections::from_map`'s deliberately unchecked assignment (mirroring
`AssetCollections.map=lib.collections||{}` in `_alImportProject`) —
`remove_custom_slot` already cleans up membership before the validator could
ever see a stale reference through ordinary editing.

**`AssetValidator.run()` golden-verified against the real reference** — the
scope document's own suggestion that it is "a strong golden-verification
candidate" held up. A transient Node `vm` harness (same technique as
milestones 1-4) ran the real `AssetDB`/`AssetCollections`/`AssetValidator`/
`_alExportEntries` on twelve constructed library states — empty, one item,
duplicate hashes across two and three slots, the grass-splat hint present
and absent, an empty custom slot, a stale collection reference reached the
only real way, and a "kitchen sink" combining several warnings at once, to
pin the reference's exact warning *order*. Every case matched on the first
run; `to_library_json()`'s shape was checked the same way across five more
scenarios (pack fields, a bare frozen slot, a tagged-but-empty custom slot
included by `fam.custom`, a tagged-but-empty frozen slot included by its
tags, a frozen slot with neither excluded entirely, collections
round-tripping verbatim, and the whole-library-empty `None` case).

**Deliberately not restored by this milestone**: `apply_library_file`
restores everything a parsed `LibraryFile` carries *except* items —
pack info, collections (unvalidated, per the finding above), and per-slot
metadata/scatter rules (`normalizeScatterRule`-on-load, applied eagerly
during parsing rather than at apply time, since the rule key is fully
computable from a record's own `fam`/`id`/`set` without touching the live
registry). `SlotRecord.items` carries everything a real reader has *except*
pixels — `img` index, name, transform — for milestone 6 to pair with decoded
`assetlib/img/<idx>.png` bytes and a real `itemHash`.

56 new tests (23 unit including `slot_title` completeness and bootstrap
invariants + 32 golden-parity + 7 hardening, some overlap between the golden
and hardening files by design — the same scenario pinned once for "matches
the reference" and once for "and here is why it matters").

**Corrections to milestones 6-7's scope, both real and both small:**

- Milestone 6's "`itemHash` duplicate detection" is **already implemented**
  here (`duplicate_groups`/`slot_has_dupe`), just missing the one piece that
  needs pixels: computing the hash itself. Milestone 6 should call
  `AssetDB::add_item`/`slot_meta_mut` with a real `itemHash`-equivalent
  string, not reimplement duplicate grouping.
- Milestone 6's "per-item transform (scale/pan/`fitToBottom`)" already has
  its data shape here (`ItemTransform`); `fitToBottom` is a real
  pixel-dimension computation milestone 6 still owns, but the field it
  writes into (and its `library.json` round trip) does not need
  re-designing.
- Milestone 6 needs to wire real item restoration into
  `AssetDB::apply_library_file` (or a milestone-6-owned wrapper around it):
  decode `assetlib/img/<SlotRecord.items[].img>.png`, compute its hash, and
  call `AssetDB::add_item` with a [`LibraryItem`] built from
  `ItemRecord::name`/`t`. This milestone deliberately stops one call short of
  that so it never touches a pixel.

### Milestone 6 — image handling: done (2026-08-17)

`cartalith-assets`, new module `raster` (not gated behind a feature — unlike
`archive`'s `zip` feature, no consumer in this crate needs an image-free
build, and `image`'s `default-features = false` + `png`-only already keeps
its own dependency footprint small). First milestone that touches pixels.

**Narrower than this section's own original description, confirmed by
reading rather than assumed — milestone 5's own corrections (above) called
this exactly, before this milestone ever ran**: the transform *shape*
(`ItemTransform`) and the duplicate-detection *machinery*
(`duplicate_groups`/`slot_has_dupe`) already existed. What milestone 6
actually shipped: real PNG decode/encode (`decode_png`/`encode_png`), a real
content hash from decoded pixels (`item_hash`), the transform math itself
applied to pixels rather than merely represented (`fit_to_bottom` mutates the
transform; `render_item` is what actually composites scale/pan onto a
canvas), thumbnail and pack-export bake (`render_item` again — the
reference's own single shared function for both, per `ThumbnailRenderer`'s
own architecture comment), `finalizePackTexture`'s inverse means
(`finalize_pack_texture_inv_mean`), and wiring decoded items into library
restoration (`AssetDB::apply_library_file_with_items`).

**Crate work (`image`) plus a thin port, exactly as this section's original
framing said** — not a hand-port, matching how milestone 2 already reused
`zip` rather than reimplementing archive handling
(`PROVENANCE.md`'s "take a crate for anything downstream of the pixels").
`image = "0.25.10"`, `default-features = false`, `features = ["png"]`: every
asset this crate ever reads or writes is a PNG (every pack entry, every
`assetlib/img/N.png` project entry), so `image`'s gif/jpeg/webp/tiff/avif/exr
codecs and its rayon/simd extras are dead weight this crate never calls. Not
present anywhere else in the workspace before this milestone.

**`itemHash` — read the real reference algorithm, then a real, checked
compatibility decision, not an assumption.** `itemHash(img,w,h)` (line
26913) downsamples through `ctx.drawImage(img,0,0,32,32)` on a canvas, then
runs a stride-7 FNV-1a variant (offset basis `0x811c9dc5`, prime
`0x01000193`, 32-bit wrapping multiply) over the resulting pixels, appending
`-{w}x{h}` (the item's original dimensions). The hash constants and stride
are ported verbatim as arithmetic. **It is not, and cannot usefully be,
golden-verified against a captured browser hash**, for two independent
reasons both found by reading rather than assumed:

1. **The hash is never serialized, on either side of this format.**
   `_alExportEntries` writes `{img,name,t}` per item (line 27890) — no
   `hash` field — and `_alImportProject` **recomputes**
   `hash:itemHash(img,w,h)` fresh after its own decode (line 27922) rather
   than reading one back from a file. No process, browser or Rust, ever
   compares its own hash against another process's; each computes one from
   its own decode, for its own runtime's own duplicate detection.
   `crate::library::ItemRecord` already reflected this before this
   milestone ever named the reason — it shipped in milestone 5 with no
   `hash` field at all.
2. **It could not match even if the format required it to.** The
   downsample runs through `ctx.drawImage`'s resample, whose exact kernel
   the HTML5 Canvas spec leaves implementation-defined — two *browsers* are
   not obliged to produce the same 32×32 pixels for the same source image,
   so "matches the reference" was never a coherent bar for this function,
   only "matches itself" is.

`item_hash` is therefore real, deterministic content hashing (`image`'s
`Triangle` filter standing in for the browser's unspecified resample),
verified with real unit tests for the property that actually matters: same
decoded pixels in, same string out, on every run, on every platform this
binary runs on; different pixels or different original dimensions, a
different string out.

**`finalizePackTexture`'s "inverse means" — read literally, and the
literal reading holds.** It is not a reversed baking transform: it is the
mean of each of R/G/B across every pixel of a texture, clamped so it never
reads as less than 1 (`Math.max(1,mean)`, so an almost-black slot cannot
blow the reciprocal past 1), then reciprocated. Ported as
`finalize_pack_texture_inv_mean(w,h,rgba) -> [f64;3]`, pure arithmetic with
no DOM dependency — unlike `item_hash`, this one **is** golden-verified
against the real reference (same transient Node `vm` technique as every
earlier milestone), six fixtures matched exactly including the `n==0` and
mean-below-1-clamped cases. `fit_to_bottom` is the milestone's other DOM-free
function and is golden-verified alongside it, seven fixtures spanning
wide/tall/square items, non-1 scale, and pre-existing pan values.

**`render_item` ports the reference's own shared render core**
(`drawItemOnly`/`renderItem`; `ThumbnailRenderer`'s own architecture
comment: "shared render core (thumbnails, inspector preview, export
bake)") as one function serving the same three uses here. The *geometry* —
position, size, alpha compositing via source-over — is exact; only the
resampling kernel (`image`'s `CatmullRom`, standing in for the reference's
unspecified `imageSmoothingQuality:'high'`) is not reference-identical, for
the same underlying reason `item_hash`'s is not.

**Why these five functions split real unit tests vs. golden-parity tests**:
every prior milestone's golden tests lift real reference functions into a
headless Node `vm.runInContext` sandbox. That sandbox has no `document`, no
`HTMLCanvasElement`, no `CanvasRenderingContext2D`, and no `Image`/
`createImageBitmap` — so `itemHash`, `drawItemOnly`/`renderItem`,
`encodeItemPng`, `decodeBytes`, `decodePackImage` simply cannot execute
there. `finalizePackTexture` and `fitToBottom` are the only two functions in
this milestone's scope that touch no DOM API at all, so those two, and only
those two, are golden-verified; everything else is real unit tests,
documented as such in `src/raster.rs`'s own module docs.

**`AssetDB::apply_library_file_with_items`** is the milestone-5-flagged
wrapper (its own note: "wire real item restoration into
`AssetDB::apply_library_file` (or a milestone-6-owned wrapper around it)").
Calls `apply_library_file` first (pack/collections/meta/rules and slot
creation — unchanged from milestone 5, still covered by its own tests), then
walks the parsed file's records again and, for each item whose PNG bytes the
caller supplies (keyed by `img` index — reading `assetlib/img/<idx>.png` out
of a project `.zip` is the caller's job, `cartalith-io`/save-format
territory, not this crate's), decodes it, computes a real `item_hash`, and
calls `AssetDB::add_item` with a `LibraryItem` built from the record's own
`name`/`t`. A missing byte entry or a decode failure for one item is skipped
silently and does not fail the rest of the restore — the reference's own
`try{...}catch(_){}` around this exact step (line 27920-27923).

**A real, deliberate non-port worth naming, found while checking the milestone's
own scope against the reference rather than assumed complete**:
`AssetImporter.importPackZip` (reference line 27067) — decoding a whole
*external pack's* manifest-declared images straight into `AssetDB`, as
distinct from restoring a previously-exported *project*
(`_alImportProject`, which `apply_library_file_with_items` above covers).
The task driving this milestone named project restoration by its real
reference function (`_alImportProject`'s shape); it did not name pack
import. Building `importPackZip`'s equivalent without being asked would be
scope creep this crate's own "narrower than its own original description"
finding argues directly against. It is a real, small remaining gap — every
piece it would compose already exists (`PackManifest` from milestone 1,
`PackEntries` from milestone 2, `decode_png`/`item_hash`/`fit_to_bottom`
from this one) — worth naming for whoever next touches pack import into the
Library, but it is not a correction to milestone 7's scope below, which is
renderer/Godot integration and does not need it.

15 new tests (10 raster unit + 3 library unit + 2 golden-parity). Still
wired to nothing.

**Corrections to milestone 7's scope: none found.** Milestone 7 was already
scoped as renderer + Godot integration plus only-then UI, with the sprite-
sheet slicer's canvas interaction and the Library page UI itself both
already named out of scope in §7 below. Reading this milestone's own real
implementation surface (decode/encode/hash/transform/bake all now real, in
`cartalith-assets`, no `gdext` dependency) confirms milestone 7's
boundary is exactly where the scope doc already drew it: sprite compositing
into the map render and ground-texture sampling are real rendering work in
`cartalith-godot`/`render.rs`, and nothing this milestone shipped changes
that surface's shape.

### Milestone 7 — renderer + Godot integration: done (2026-08-17)

`cartalith-godot`, new module `pack` — the first thing in the workspace to
depend on `cartalith-assets` (its own doc comment said "nothing depends on
this yet" until now). Two of this milestone's own three named surfaces are
real:

- **Sprite compositing** (`composite_map_icons`, `drawMapIcons`'s own
  Y-sorted painter's pass): builds a scatter-rule table from a loaded pack's
  manifest (`autopopulate_scatter_rules`), derives a `BIOME_INDEX` raster and
  a wetland mask from the already-generated height/temperature/rainfall
  fields (presentation-side computation, no new world-generation data —
  `cartalith_civ::classify_biome`, already golden-verified elsewhere, plus a
  `buildWetlandMask`-equivalent), calls `place_map_icons_ruled`, then
  composites each placed icon: a real bilinear-sampled blit
  (`sprite_draw_rect`'s destination geometry) where the pack has art for
  that slot, a real per-slot procedural glyph fallback (`draw_icon_glyph`,
  all ten `PACK_ICON_SLOTS` shapes — mountain/hill/six tree kinds/cactus/
  boulder, "shrub" doubling as the reference's own documented catch-all for
  an uncovered custom asset) otherwise.
- **Ground-texture splat** (`land_color`'s new branch, `render.rs`): the six
  `SPLAT_PAINT_SLOTS` channels, decoded and inverse-mean-baked at load time
  (`finalize_pack_texture_inv_mean`, milestone 6's own function — wired to
  something real for the first time), blended per-cell using the *exact*
  `materialWeights` fractions and each material's own procedural ramp colour
  `land_color` already computes — no new logic, splat is a read-only
  consumer of both.

**The third named surface — ground-texture sampling for the two "painted
layers" — is deliberately not implemented this pass, and this is a real
scope finding, not an oversight.** Read literally (reference lines
7898-7900, 12187-12196): `pBio`/`pTer` are per-cell indices into
`state.cartoPaint.biome`/`.terrain`, sparse arrays a manual Cartography
paint-brush tool populates (`paintBiome`/`paintSplat`/`paintTerrain` module
globals). This port has never ported that tool — there is no producer of a
painted-cell array anywhere in the workspace, and building one from scratch
is itself a real, separate UI+state effort this milestone's own "no GUI
controls" boundary rules out (a paint tool has no meaning without a brush UI
to drive it). Unlike splat (gated only by `assetPack.texAny`, on by default
the instant a pack loads) and icons (gated by `state.viz.icons`, off by
default regardless), the painted layers are gated by a *third* piece of
state this port simply has no producer for — so `LoadedPack` parses
`.biomes`/`.terrains` from the manifest (for a correct warning count) but
never decodes or rasterises them. Named here as the natural remaining item
for whoever next ports the Cartography paint-brush tool, per the terrain-
appearance research vocabulary this document's own §1 table already used.

**Two real defaults confirmed by reading the reference, not assumed**:
`state.viz.icons` defaults `false` (icons are an opt-in `state.viz.*`
stretch feature like every other one `render.rs`'s own doc comment already
excludes — a pack-less *or* icon-toggle-off render was always bit-identical,
and `current_scatter_rules` returning `None` whenever no pack supplies real
icon art is `composite_map_icons`'s own early return, reproducing exactly
that no-op). `state.viz.splat` defaults **`0.7`** — the opposite shape,
gated only by `assetPack.texAny`, real and active the instant a pack with
real ground textures loads, no toggle at all. Both are genuinely additive/
opt-in rather than JS-parity-gated stretch features (per this milestone's
own "judge from what you find" instruction) — there is no pack-less version
of "blend in a texture that doesn't exist" to stay bit-identical with.
`golden_parity_render.rs` passes unmodified at its original `1e-4`
tolerance either way, since `RenderCtx.splat` stays `None` on that path
(`with_splat` is a builder method, never called by the test).

**This port confirmed to ship no default asset pack** — nothing in
`godot-project/` bundles pack art — so real sprite/splat compositing has
nothing to composite in the common case, exactly as this milestone's own
scope anticipated. Real, permanent new plumbing was added for it rather than
a throwaway stand-in: `WorldGen::load_asset_pack(path) -> bool` (a native
filesystem path, same convention as `load_save`) and
`WorldGen::has_asset_pack() -> bool`, both real `#[func]` API surface with
no GDScript UI call site anywhere — dormant, real code for a future importer
or `GUI_SHELL_SCOPE.md` pass to call, not a GUI control in itself.

**Verified three ways.** A new `cartalith-godot/tests/pack_compositing.rs`
loads the real `reference_pack.zip` fixture milestone 2 golden-verified
against the reference's own exporter (reused rather than inventing a new
fixture, per this milestone's own instruction) and proves, on a small
synthetic world: real sprite art blits where a relief-mode mountain places
one; the procedural glyph fallback fires for a biome region the fixture has
no art for at all; and a pack whose manifest has no icon slots places
nothing — the same "keeps `placeMapIcons` on its legacy/no-op path"
condition `current_scatter_rules`'s own doc comment names as what keeps a
pack-less render bit-identical. Static: `cargo build -p cartalith-godot`/
`--workspace`, `cargo test --workspace` (zero regressions,
`golden_parity_render.rs` unmodified), `cargo clippy -p cartalith-godot -p
cartalith-assets --all-targets` clean (the rasterizer's loose `bytes/gw/gh`
argument triples became a small `Canvas` struct along the way, both for
clippy's `too_many_arguments` and because it reads better), `godot4
--headless --quit main.tscn` clean. Real windowed: launched the actual
`Godot_v4.7.1-stable_win64.exe`, generated a real 512² world, called
`load_asset_pack` against the real fixture (temporary `main.gd` debug calls
only, reverted before commit — the shipped diff carries no GDScript
changes), and saved the native `Image` output directly to disk for
full-resolution inspection rather than a scaled-down window screenshot.
**Confirmed by actually looking at it**: a sharp-edged, flat-coloured
rectangular block sits on land exactly where a relief-mode mountain would
place one (real pack sprite art — a procedural blend is always noisy/
gradient, never a hard-edged rectangle); a large irregular checkerboard
region follows real land-material boundaries rather than sitting in a fixed
box (real per-pixel splat sampling, not a sprite); small soft-edged
translucent blobs appear elsewhere on plain terrain (the procedural glyph
fallback, where the fixture has no matching art).

3 new tests (real integration tests against the real fixture pack, not
unit tests standing in for one). Not gold-plated beyond what was asked: the
sprite resample is a hand-written bilinear sampler rather than a new `image`
crate dependency in `cartalith-godot` (the icons involved are small; a
manual sampler is the smaller, sufficient tool); the procedural glyph
fallback drops two purely-decorative reference variants (the arid jagged
hill outline, the cold-mountain snow-cap) since the reference itself
describes them as "procedural-fallback variety only" on top of an
unconditional base silhouette, which is what's ported.

**Phase 4 is genuinely complete.** Checked honestly against §8's own "done
means" below, written specifically to give this phase an operational finish
line beyond `ROADMAP.md`'s one-sentence description — that bar is met. The
Library-authoring workspace is that same sentence's own explicit carve-out,
tracked separately in `GUI_SHELL_SCOPE.md`, not part of this phase's
definition of done.

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

## 9. The GUI window now exists (2026-08-19), and what it found

`DCC_SHELL_SPEC.md` §8's Asset library window is now built
(`cartalith-native/godot-project/shell/asset_library_window.gd`,
`AssetLibraryWindow`) — `Assets ▸ ⧉ Asset library` / `▦ Sprite sheet slicer`
are `_live` in `menus.gd`, no longer `_todo`. This section is that pass's own
honest close-out, in the same voice as §§1-8 above, not a rewrite of them.

**A real discrepancy confirmed against the live engine, not the mockup**: §8's
own prose describes "24 families... Settlements, Terrain, Cartography, plus
Collections." `cartalith-assets` ships **eight**, exactly as §1 above already
said ("eight families, seven of them closed vocabularies") — re-verified this
pass by reading `slots.rs`/`library.rs` directly and by a headless smoke run
that opened every one of the eight and confirmed each grid populates with the
real frozen slot count (textures 7, biomes 15, terrains 13, icons 10,
settlement 9, trait 7, poi 10 — the Library's own 10-slot `poi` list, not the
8-slot pack-import one — custom 0/open). The 24-family, four-group rail is the
mockup's own finer subdivision; nothing in the shipped crate draws that line,
so the window's family rail groups the real eight the way the crate itself
groups them (`Family::is_texture()`, the `structures.*` trio) rather than
inventing a fifth grouping to hit 24.

**What the window can honestly show, and what it can't, comes down to one
gap**: `cartalith-godot/src/lib.rs` exposes exactly two asset-related
`#[func]`s -- `load_asset_pack(path)` and `has_asset_pack()`. There is no live
`AssetDB` on the Godot side of the boundary, so per-slot fill state,
thumbnails, item variants, tags, scale, and pack metadata (name/author/
license) are all disclosed gaps in the window, not guessed values -- the slot
grid shows every slot as a checkerboard on principle, never as "empty" or
"filled," because the engine genuinely cannot say which from here. Apply to
map / Export pack .zip / batch edit / Validate / Clear library are gaps for
the same reason: there is no in-memory library-editing session anywhere in
this workspace for any of them to act on. The sprite-sheet slicer modal's
image load, dimension readout, and columns/rows/margin/spacing grid overlay
are real (Godot's own `Image` loader plus arithmetic); the slice operation
itself is a gap -- `cartalith-assets::raster` decodes/encodes whole PNGs with
no sheet-splitting function anywhere in the crate.

None of this needed a new `#[func]` or touched any Rust file. Closing the gap
above -- a `#[func]` surface for `AssetDB` query/mutation -- is real, scoped
future work, not filed here as a blocker to §8's "done means."
