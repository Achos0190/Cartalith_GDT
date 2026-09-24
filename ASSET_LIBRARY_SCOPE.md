# ASSET_LIBRARY_SCOPE.md — Phase 4: the asset pack format, library and slicer

**What this is:** the definition of Phase 4 — eight porting milestones (§6,
§11), the asset library window and its binding surface (§9, §10) — together
with the pack format as the reference really writes it and the design reasoning
each pass found. **What it is not:** status. Where any of it stands is
`cartalith-native/docs/STATUS.md`'s "Phase 4 — Asset Library" group (AL-1…AL-10).

Everything here was read from `reference/Cartalith Gen1 v2.10.html` (every line
number below resolves there), not from the two older design documents in
`docs/`; where those and the reference disagree, the reference wins.
`ROADMAP.md`'s Phase 4 is one sentence — "Block 3, the sprite and texture pack
system" — and this document is the investigation that sentence deferred.

**Three owner rulings bind this subsystem** (`LARGE_ITEM_RULINGS.md`):

- **Ruling F (2026-09-07): packs keep `.zip` and their format stays identical
  to the HTML app's.** A pack written here must stay readable by the HTML app
  and vice versa, so no entry name, layout or metadata may drift toward this
  port's conventions — and `SAVEFILE_COMPAT.md`, which governs project archives,
  does not apply to a pack.
- **The pack-import warning is owner-ruled text** (2026-09-03, then Ruling W,
  2026-09-21): a golden re-baseline of one string, taken knowingly (§6,
  milestone 7).
- **Ruling AP (2026-09-23): asset library images are embedded in the project
  save** (§2).

---

## 1. What an "asset" actually is

**Not** an arbitrary named image with free-form metadata. An asset is **one PNG
bound to one slot in a frozen, ordered vocabulary the engine already knows how
to draw**, plus an optional per-slot metadata record and an optional per-item
display transform.

The reference has eight families, seven of them closed vocabularies (lines
12029-12052, mirrored by the Asset Library's own `FAMILIES` table at line
26784). This port adds a ninth, `seamarks`. `cartalith_assets::Family::ALL`
holds all nine, in export and display order.

| Family | Manifest section | Slots | Bake | Anchor | Consumed by (reference) |
|---|---|---|---|---|---|
| Splat channels | `textures` | 7 | 512², opaque, seamless | tiled | `surfaceColor`'s splat blend + the parchment overlay |
| Biome ground | `biomes` | 15 | 512², opaque, seamless | tiled | painted Cartography biome layer (`_paintedTex`) |
| Terrain ground | `terrains` | 13 | 512², opaque, seamless | tiled | painted Cartography terrain layer |
| Feature icons | `icons` | 10 | 256², RGBA | **bottom** | `placeMapIcons` → `drawMapIcons` |
| Sea marks | `seamarks` | 8 | 256², RGBA | centre | *no reference counterpart* — added by owner ruling (2026-08-31, "CARTO ▸ Icons: generated placement") |
| Settlement pins | `structures.settlement` | 9 | 256², RGBA | centre | civ layer `_structSprite` |
| Settlement traits | `structures.trait` | 7 | 256², RGBA | centre | `_traitSprite` — imported since v1.28, never drawn by the reference |
| POI markers | `structures.poi` | 8 | 256², RGBA | centre | `_customSprite`/`_featureSprite` |
| Custom icons | `custom` | **open** | 256², RGBA | centre | manual icon brush + rule-driven scatter |

**The sea-mark family** (`PACK_SEAMARK_SLOTS`) exists so the design's fourth
placement family, `SEA MARKS`, and its *snap sea marks to coast* rule are
literal rather than mapped onto three families. Its slot *count* is the
design's (`'SEA MARKS':[6,8]` in `cartalith-dcc-parts.js`); the eight *names*
are this port's, all shoreline things. It is **centre**-anchored by choice:
half its slots (buoy, reef, shoal, whirlpool) float and have no base to stand
on. It sits after `icons` in `Family::ALL`, so no existing pack's file order
moves. The HTML app's reader never reads a section it does not name (§2), so a
pack carrying `seamarks` still loads there, with that section unread. Frozen
from its introduction on the same terms as the ported lists.

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
  as a complete one. This is what makes the subsystem optional rather than a
  dependency.

**Two POI vocabularies, both real.** The Asset Library's `poi` family has
**ten** slots (`LIBRARY_POI_SLOTS`, from the Library's own `FAMILIES`), but the
pack *import* vocabulary has **eight** (`PACK_POI_SLOTS`): `lake` and `bridge`
have no engine POI kind to attach to, so they can be authored and exported but
never load. The reference documents this at line 12033 and leaves it; this port
reproduces both lists rather than "fixing" one.

## 2. What an "asset pack" is as a format

A real, versioned serialization format, verified against a pack the reference
itself exported (milestone 2).

```
mypack.zip
├── pack.json      # manifest, schema 1 or 2 (or pack.csv; JSON wins if both)
├── textures/  biomes/  terrains/      # one PNG per slot
├── icons/  seamarks/  structures/{settlement,trait,poi}/   # slot_01.png, slot_02.png, …
└── custom/<setId>/                    # slot_01.png …
```

- **A plain PKZIP**, written by the same `zipStore()` (line 12009) the world
  save uses and read by `unzipAny()` via the central directory. The export
  policy the reference applies on top — which entries are stored, the
  timestamps, the entry order — is ported behaviour; milestone 2 lists it.
- **The manifest is the source of truth, not the folder layout.** Paths are
  ZIP-root-relative and may be anything; the directory names above are only
  what the exporter writes (`Family::dir`, `Family::asset_path`).
- **Schema 2 is a strict superset of schema 1.** A schema-1 consumer reads a
  schema-2 pack by ignoring what it does not know. **Nothing is rejected:** an
  unknown slot inside a known section is dropped *with a warning*, and a
  top-level section the reader does not know is not read at all (the reference
  builds its result from the sections it names, lines 12113-12170). Parsing can
  fail only on a missing or malformed manifest, never on its content.
- **`pack.csv` is a real second input format:** `parsePackCsv` (line 12093)
  ships. It is header/CRLF/blank-tolerant, carries a `variant` ordering column,
  and — unlike the JSON path — drops unknown slots *silently*. It predates
  `structures`/`custom` and cannot express them, nor a pack name, author or
  licence.
- **Warnings are ordered data.** `parsePackManifest` (line 12113) emits
  per-slot missing-file and unknown-slot warnings in a traversal order that
  partly follows the author's own key order (JavaScript iterates string keys by
  insertion). A UI reports the count beside the import summary and proceeds.

**The editable Library travels inside a project save — a second, different
serialization.** The reference's `_alExportEntries`/`_alImportProject` (lines
27879/27900) write `assetlib/library.json` plus `assetlib/img/N.png` into the
project `.zip`: per-slot metadata, tags, collections, per-item transforms and
scatter rules, rather than a baked pack. The two formats are not
interchangeable. This port's project archive carries the same record
(`AssetDB::to_library_json`, field order matching a captured reference export)
at `library/assets.json` (`project_bridge.rs`, `SLOT_ASSETS`). **Ruling AP
(2026-09-23): the item images are embedded in the save file too**; the entry
layout, and how a pre-change archive loads, are `SAVEFILE_COMPAT.md` §16.5's to
specify. The restore reports how many items it rebuilt (`project_bridge.rs`),
so an archive without the images comes back as slots, metadata and rules with
zero items, never as an apparently complete library.

A deliberate non-format, so nobody looks for it: the live `assetPack` global
is **never serialized into `params.json`** (the reference's transient-UI
invariant 6). The Library payload is the one persisted asset store.

## 3. How assets are actually used

The reference renderer really draws pack sprites onto the map, and has for many
versions; the vector glyphs are the *fallback*, not the other way round.

1. **`placeMapIcons(fld, biome, W, H, opts)`** decides *where* glyphs go —
   pure, reading only its arguments plus `hash()`. Two engines behind one entry
   point: a **legacy** path with the biome→slot mapping hard-coded, and
   **`placeMapIconsRuled`** (v1.26, line 7194), which makes the mapping *data*
   — a `ScatterRule` per asset. Passing no rules keeps the legacy path
   bit-identical, which is what lets a pack-less map stay unchanged.
2. **`iconSlotForItem`** (line 7294) resolves a placed item to a slot key — the
   one place the flat vocabulary and the open custom one are unified
   (`custom::<set>::<slot>`).
3. **`iconVariantsFor` + `pickWeightedVariant`** choose the variant
   (position-hash, optionally weighted by the asset's own rule).
4. **`drawMapIcons`** composites one Y-sorted painter's pass over every icon,
   `spriteDrawRect(x,y,s,base,sw,sh)` (line 12173) giving bottom-centre
   placement scaled to `base = max(3.5, W/110)` and the sprite's own aspect
   ratio. No pack art for a slot ⇒ `drawIconGlyph` draws the procedural
   version.

**Ground textures take a different route, and the asymmetry is easy to get
wrong.** `finalizePackTexture` (line 12196) stores a per-channel inverse mean so
splatting modulates a procedural material ramp by `texel/mean`; `biomes` and
`terrains` deliberately **skip** that and are sampled as true colour
(`_paintedTex`, line 12187), because dividing out a tile's absolute hue is
right for splat and wrong for paint (the reference's own comment, line 12246).

The reference's consumers are the terrain renderer, the civ layer's
settlement/POI drawing and the Cartography manual-icon brush; its urban
morphology (block 4) consumes no pack.

**In this port**, a loaded pack reaches the live map through:

| Section | Path | Symbols |
|---|---|---|
| `textures` | splat blend, inverse means baked once at load | `render::SplatTextures` via `RenderCtx::with_splat`, strength `0.7` |
| `biomes`, `terrains` | painted-cell blend, **true colour**, at the reference's own `0.60` weight and position | `pack::decode_ground_family` → `render::GroundTiles` via `RenderCtx::with_ground_tiles`, sampled by `render::painted_tex`. `GroundTile` has no inverse-mean field, so the asymmetry above cannot be broken silently |
| `icons` | scatter placement and the Y-sorted composite | `pack::composite_map_icons` |
| `structures.trait` | settlement trait badges — **beyond the reference**, which imports trait art and never draws it | `pack::resolve_trait_badges`, through `WorldGen::civ_trait_badge_row`, which the shell installs as the overlay's resolver (`viewport_host.gd::refresh_settlement_traits`). `pack::composite_trait_badges` exists and is tested (`tests/pack_trait_badges.rs`) but has no caller in the live path *(corrected 2026-09-24)* |

The import warning names every section with no compositor on the live map, and
under **Ruling W** it names all of them (`structures.settlement`,
`structures.poi`, `custom`, `seamarks`); drawing those is separate work the
ruling does not authorise. With a pack loaded the raster is quantised before
the icon composite, which draws in bytes, and the colour space is applied after
it (Ruling AN).

## 4. Portable vs. UI-only — the honest split

**Portable pure logic (~600-800 reference lines):**

- Pack manifest: `parsePackCsv`, `parsePackManifest`, `packSummary`, the
  `PACK_*_SLOTS` vocabularies, `PackManifestBuilder`'s manifest half, `slugId`.
- **The archive export policy** — STORE the PNGs, freeze the timestamps, write
  `pack.json` last, deflate only when that helps, never normalise a name on
  read. The container is crate work; this policy is ported behaviour that a
  plain `zip` call gets wrong by default (the timestamp actively so). About
  60 lines of policy over a crate.
- Library model: `AssetDB`'s slot registry and custom-slot add/rename/remove
  (id slugging, uid collision handling, collection cascade), `AssetCollections`,
  `defaultMeta`, `AssetValidator.run()`'s rules — and the `mkSlots` title
  table, which looks presentational and is not: the validator's "Identical
  images" warning prints `slot.name`, not `slot.id` (milestone 5).
- Scatter rules: `defaultScatterRule`, `SCATTER_RULE_PRESETS`,
  `presetScatterRule`, `normalizeScatterRule` (with its v1.27 hardening against
  untrusted project input), `scatterRuleKey`, `currentScatterRules`,
  `autopopulateScatterRules`, `pickWeightedVariant`, and `pickIconVariant`,
  which is three lines and cannot be separated from it.
- Placement/geometry: `placeMapIconsRuled`, `spriteDrawRect`, `iconSlotForItem`
  with its `TREE_SLOT`/`SCATTER_SLOT` maps, `finalizePackTexture`'s arithmetic.
- The slicer's pure core: cell rectangles from cols/rows/spacing/interior-line
  fractions, the crop rounding, the chroma key's colour-distance test, the
  blank-cell test (§11).

**UI/DOM-coupled (~900 of block 3's ~1,439 lines):** `AssetBrowserUI`,
`InspectorUI`, `ImageEditor`, the `SpriteSheetImporter` **modal** (~408, the
largest module in the block, almost entirely pointer and canvas interaction
around that small core), the `AssetLibrary` page controller,
`renderPackInspector`, `toast`, `UIState`'s `localStorage`, drag-and-drop
intake, preview backdrops. Godot owns presentation (`ARCHITECTURE.md`); these
were rebuilt as GDScript, not ported (§9-§11).

**Platform work, not a port:** image decode (`decodePackImage`,
`AssetImporter.decodeBytes`), thumbnail and export rasterisation
(`ThumbnailRenderer`, `encodeItemPng`), the ZIP container. In Rust these are
the `image` and `zip` crates plus Godot's `Image`/`ImageTexture` —
`PROVENANCE.md`'s "take a crate for anything downstream of the pixels".

## 5. How big Phase 4 is

- Block 3 (the Asset Library page): lines 26723-28161, ~1,439 lines.
- Block 1's asset regions: scatter rules and icon placement/drawing
  (~6895-7420), pack parse/load/inspector (~12028-12330) — ~800 lines.
- Block 2's consumers (`_structSprite`, `_traitSprite`, `_customSprite`,
  `_featureSprite`, `_carIconBrush*`, the icon gallery/editor): several hundred
  more, mostly UI.

About 2,250 lines against the Journey Planner's ~3,100 — but where the Journey
Planner was ~70 functions of dense portable modelling, Phase 4 is 600-800 lines
of portable logic inside 1,000+ lines of editor UI, plus image/ZIP work that is
crate integration. **A real sub-phase, not a milestone:** anyone estimating it
from `ROADMAP.md`'s one sentence is wrong by an order of magnitude.

## 6. Milestone breakdown

"Golden" below always means the same technique, and a milestone says where it
does not apply: a transient Node `vm.runInContext` harness lifts the named
reference functions out of the frozen HTML **by line range**, runs them on
fixtures, and the Rust tests assert that run's output verbatim. Fixtures target
what a rewrite gets *plausibly* wrong, not the happy path.

### Milestone 1 — pack manifest model, parsing, validation, serialization

New crate **`cartalith-assets`**, no `gdext` — deliberately the piece with no
images, no archive, no renderer and no UI, which every later milestone is
defined against. It began with no dependency on another Cartalith crate; it has
since taken `cartalith-noise` (milestone 3's `hash`) and `cartalith-jsmath`.

- `slots.rs` — the frozen vocabularies verbatim, and a `Family` enum carrying
  each family's manifest section, export directory, bake size, opacity, anchor
  and multi-variant flag (the reference's `FAMILIES` metadata), plus
  `Family::asset_path` (the exporter's path convention) and `slug_id`.
- `manifest.rs` — `RawManifest` (as authored, key order preserved) and
  `PackManifest` (validated); `parse_pack_csv`, `parse_pack_manifest`,
  `parse_pack_entries` (the `parsePackManifest(zip)` equivalent, over entry
  names so the model needs no archive), `pack_summary`, `to_raw`/`to_pack_json`
  for schema-2 export, `referenced_files`, and a `PackError` whose `NoManifest`
  message is the reference's own string.
- `ordered_map.rs` — a small insertion-ordered map, and not incidental: warning
  order is a function of how the author wrote the pack. `BTreeMap` would sort
  it away; `serde_json`'s `preserve_order` would have leaked into `cartalith-io`
  through workspace feature unification.

Golden fixtures: a missing texture file; an unknown texture slot; an unknown
biome slot that is really a *terrain* slot; one missing icon variant (slot
survives) against all variants missing (slot dropped whole); a bare string
standing in for a one-element variant list; an unknown settlement slot; a
missing custom-set variant; CSV variant ordering as a *stable* sort with
unnumbered rows last; JSON winning over CSV; an empty-string path counting as
missing; and the exact wording and order of every resulting warning.

`packSummary`'s trailing "*N* custom icon(s)" counts custom **slots**, not
variants — a two-variant lighthouse reads "1 custom icon". It looks like a bug
and is the reference's behaviour.

### Milestone 2 — pack ZIP read/write

`unzipAny`/`zipStore` in Rust terms: read a real pack into
`parse_pack_entries`, and write one back. **`cartalith-assets`, module
`archive`, behind an on-by-default `zip` feature**, decided by reading
`cartalith-io` first:

- **There is nothing to share.** `cartalith-io`'s zip handling is
  `ZipArchive::new`, `by_name` and `read_to_end`: the `zip` crate *is* the
  shared helper. "Packs use the same `zipStore()` the world save uses" is true,
  and precisely because it is true it implies a shared crate, not shared code.
- **The dependency would point the wrong way.** Packs in `cartalith-io` would
  make every consumer of the world-save loader drag in the asset vocabulary;
  packs are the optional subsystem.
- **The feature keeps milestone 1's promise literally true.**
  `default-features = false` gives back the archive-free model, and it is
  tested that way (`cargo test -p cartalith-assets --no-default-features`).

**The reference's archive behaviour, ported rather than left to `zip`'s
defaults** (`archive::zip_store`, which the region-tile export writes through
too):

- **`.png` entries are STORED, never deflated** (case-insensitive) — a PNG is
  already DEFLATE-compressed.
- **Anything else is deflated only if that makes it smaller**, else STORED,
  decided by compressing once with `flate2`, the encoder `zip` itself uses.
  Milestone 2 first read this as a browser concern and left it out;
  `UNIFIED_TOOL_PLAN.md` milestone E2 found the region export hits it on its
  first entry, so it is ported.
- **Timestamps are frozen at 1980-01-01 00:00:00** — `zipStore` hardcodes the
  DOS date word to `0x0021` and the time word to `0`, which makes exports
  byte-reproducible. `zip`'s default is the wall clock, so the port sets it.
- **`pack.json` is written last**, after every image, as the exporter does.
- **Names are read verbatim** — no wrapping-folder stripping, no backslash
  rewriting. Zipping a *folder* rather than its *contents* therefore yields a
  manifest at `MyPack/pack.json` that is not found — a real, reported failure
  (the reference's own message says "try re-zipping the folder…"), preserved
  rather than papered over.
- **The writer emits no directory entries**; on read, a directory entry some
  other tool wrote is kept as a zero-byte member, because `unzipAny` walks the
  central directory. Harmless: no manifest path ends in `/`.
- **An unrecognised compression method is an error** worded as the reference
  words it (`unsupported zip method 93 for pack.json`), not a skipped entry.

Not ported: `unzipStore`, `unzipAny`'s fallback for an archive with no readable
central directory, which answers `null` for every deflated entry — a browser
defence against a truncated `ArrayBuffer`. `zip::ZipArchive` requires the
central directory and errors cleanly without it, which is the better answer.

**Verified against a pack the reference itself exported, both directions.** The
harness runs the reference's own `PackManifestBuilder.build()` (line 26964) over
its own `FAMILIES`/`AssetDB` vocabulary and its own `zipStore()` headlessly.
Only two things in that run are not reference code, and the test file says so:
`renderToBlob` (a canvas rasteriser, replaced by a real PNG encoder at each
family's bake size) and three stubbed DOM inputs. The output is checked in as
`tests/fixtures/reference_pack.zip` (18 entries, 21 KB) with that run's
`unzipAny`/`parsePackManifest`/`packSummary` capture.

- **Read:** entries match `unzipAny` name for name and CRC for CRC;
  `parse_pack_entries` reproduces the summary and warning; `to_pack_json()`
  reproduces the exporter's `pack.json` **text byte for byte**.
- **Write:** `write_pack` reproduces entry order, per-entry method, CRC-32,
  uncompressed size and the 1980 timestamps, and the reference's own
  `unzipAny` + `parsePackManifest` read the result back identically. The two
  archives differ by 2 bytes, the first being the version-needed-to-extract
  field. Byte equality is not the bar: the one deflated entry is compressed by
  `miniz_oxide` here and the browser's zlib there, and two conforming encoders
  need not agree on a bit stream.

### Milestone 3 — scatter rules

`cartalith-assets`, module `scatter`: `ScatterRule` + `ScatterMode`, `Default`
(`defaultScatterRule`), `preset_scatter_rule` (the ten `SCATTER_RULE_PRESETS`),
`normalize_scatter_rule`, `scatter_rule_key`, `current_scatter_rules`,
`autopopulate_scatter_rules`, `pick_weighted_variant` and `pick_icon_variant`.
`pick_icon_variant` is `hash`, which is why this crate depends on
`cartalith-noise`: re-implementing that hash locally would have lost the two JS
float subtleties its doc comment carries.

**The v1.27 hardening, re-derived for Rust rather than transcribed.** Rules are
read out of `library.json` inside a *user-supplied project archive*, so every
field reaching the normalizer is untrusted. v1.26 merged with `+x||fallback`,
which lost a legitimate `0` and let `NaN` through. `tests/hardening_v1_27.rs`
has one test per fix, each reproducing the *downstream* arithmetic from
`placeMapIconsRuled` so the test shows the failure it prevents:

1. **`NaN` density scatters on every cell — by the opposite IEEE rule.** JS's
   `keep >= Math.min(1, density)` gets `NaN`, so nothing is rejected. Rust's
   `f64::min` absorbs `NaN` to `1.0`, but `keep` is a hash in `[0,1)`, so
   `keep >= 1.0` is false and the corrupt rule still carpets the map. Rejecting
   non-finite input at the boundary closes both.
2. **`NaN` spacing collapses an O(1) neighbour test to O(n²)** —
   `Math.ceil(W/NaN)||1` gives a 1×1 bucket grid. Rust's NaN-absorbing
   `f64::max` would rescue the derived path *by accident*; the explicit
   `is_finite` check stays, because an implicit dependency on an IEEE corner is
   what the fix exists to remove. The fix is two-sided — reject at the
   boundary, and guard the computed value for callers that bypass it
   (`ScatterRule::spacing_cells(map_width)`, which also reproduces a density of
   exactly `0` deriving spacing as if it were `1`, and the 3-cell floor).
3. **The `Object.assign` aliasing bug is structurally unreachable** — not
   because of ownership, but because the defaults (an owned `ScatterRule` with
   `f64` fields) and the untrusted input (a `serde_json::Value`) are different
   *types*: a `"x"` cannot be stored in the field it would corrupt. No
   defensive code was written. The test pins the reference's own probe
   (`{minSize:"x", maxSize:2}` gives the preset's `0.55` and a surviving `2`)
   so a future "merge" helper fails loudly.

**`ScatterRule` implements `Serialize` and deliberately not `Deserialize`**, so
`normalize_scatter_rule` is the only way in; untrusted input is typed as
`&serde_json::Value` for the same reason.

**`biomes` is `Vec<f64>`, not integers.** The reference filters the list with
`Number.isFinite`, which does not coerce: a `"4"` is dropped, a hand-edited
`5.5` is **kept** and never matches. Truncating to `i32` would make it start
matching, and would rewrite the author's file on the next round trip.

Golden: `pick_weighted_variant` is hash-driven, so it diffs **exactly**,
including the degenerate weightings that must fall through to
`pickIconVariant`'s untouched v1.25 hash. One fixture caught a real bug on the
first run: **`density`'s fallback is not symmetric with the other fields.** The
reference merges first and *then* runs `num(out.density,0,3,1)`, so an absent
`density` keeps the slot preset's value (`cactus` stays 0.35) while a
*rejected* one lands on a literal `1`. Nothing but a golden run would have
found that.

### Milestone 4 — rule-driven icon placement

`cartalith-assets`, module `placement`: `place_map_icons_ruled`
(`placeMapIconsRuled`, line 7194), `icon_slot_for_item` with the
`TREE_SLOT`/`SCATTER_SLOT` legacy maps (lines 7289-7290, `iconSlotForItem`
7294), and `sprite_draw_rect` (12173). Positional and seeded, so it diffs
exactly rather than within a tolerance.

**The legacy (non-ruled) `placeMapIcons` body is out of scope, on purpose.** The
reference enters `placeMapIconsRuled` only when `opts.rules` is non-empty, and
`current_scatter_rules` already reproduces the empty-table condition under which
it falls through. `icon_slot_for_item` is ported in full, legacy branches
included, because a legacy-shaped item and a ruled one must agree on it.

**Both v1.27 fixes here transfer to Rust**, because both are logic defects, not
JS-coercion artefacts:

1. **Most-specific-first priority** (lines 7250-7259, ported as
   `specificity`). Before v1.27 a contested cell went to whichever rule the
   array listed first — which meant the order the user added assets to the
   Library. A `Vec` is as insertion-ordered as a JS array.
2. **`requireWetland` ANDed with the biome test, not substituted for it**
   (line 7273). v1.26 let it *replace* `biomeOk`, silently discarding a rule's
   biome restriction whenever wetland was also required.

Proven by a hand-traceable fixture: a 3×1 grid, `sea=-1`, `tGap=1` — so
`(hash(...)*1)|0` is always `0` and the jitter degenerates to `jx=gx, jy=gy`
(checked against the real `hash`). Three cells (wetland+grass, dry+grass,
wetland+shrub) and three rules inserted **least-specific first** resolve to
`wetland_grass` / `narrow_biome` / `generic_land` across three seeds, unchanged
when the rule array is reversed; the third cell is fix 2's proof. The golden
run — a synthetic 10×8 grid through an eight-rule table across six
sea/seed/density configurations, matched cell-for-cell to 1e-9 — includes a
`ghost_biome` rule with `biomes:[5.5]` placing nothing anywhere, confirming the
`f64` comparison above.

### Milestone 5 — the Library model

`cartalith-assets`, module `library`: `AssetDB` (slot registry, item store,
`add_custom_slot`/`rename_custom_slot`/`remove_custom_slot`, lazy `slot_rules`,
`clear`, `duplicate_groups`/`slot_has_dupe`), `AssetCollections`, `run` (the
reference's `AssetValidator.run()`), `ItemTransform`, and the `library.json`
record shape: `LibraryFile`/`SlotRecord`/`ItemRecord`, `parse_library_json`,
`AssetDB::to_library_json`/`apply_library_file`. Pure data management —
`LibraryItem.hash` is caller-supplied, which keeps the validator's duplicate
detection golden-testable without decoding a PNG. Reuses milestone 1's
`Family`/`slug_id` and milestone 3's rule functions rather than re-deriving
them.

**The record shape**: `{version, kind, pack: {name, author, license},
collections: {name -> [uid]}, slots: [{fam, id, name, meta, items: [{img, name,
t}], set?, rules?}]}`, field order matching a captured `_alExportEntries()`
export. No `hash` field — milestone 6 says why.

Findings that shape the model:

- **Per-slot display names are load-bearing.** `AssetValidator.run()`'s
  "Identical images" warning renders `slot.name`, not `slot.id` — golden:
  `"Identical images: Mountain#1 = Hill#1"`. So the `mkSlots` title table is
  ported (`slot_title`).
- **A custom slot keeps both its raw set name and its slug.** Watching the real
  exporter: the file path uses the slug (`custom/naval/lighthouse_01.png`) and
  the manifest key is the author's text (`"custom": {"Naval": …}`). Losing
  either makes a round trip lossy; `AssetDB.addCustomSlot` carries both
  (`slot.set`, `slot.setId`).
- **Id-slug and uid-collision hardening, in the reference's own code but with
  no version tag.** `addCustomSlot` returns the *existing* slot on a uid
  collision; `renameCustomSlot` refuses a colliding rename and keeps the old
  uid. Ported faithfully and pinned in `tests/hardening_asset_db.rs` —
  free-form user text slugging to a collision is a real hazard for content
  editable outside the app.
- **Two of `run`'s six checks are unreachable through the public API, in both
  languages** ("Duplicate identifier", "Invalid filename id"); ported anyway as
  defence in depth. "Collection references a missing asset" is reachable only
  through `AssetCollections::from_map`'s deliberately unchecked assignment
  (mirroring `_alImportProject`'s `AssetCollections.map=lib.collections||{}`),
  because `remove_custom_slot` cleans membership up first.

Golden: the real `AssetDB`/`AssetCollections`/`AssetValidator`/`_alExportEntries`
on constructed library states — empty, duplicates across two and three slots,
the grass-splat hint present and absent, an empty custom slot, a stale
collection reference reached the only real way, and a combined case pinning
warning *order* — and `to_library_json()`'s shape across its inclusion rules
(a tagged-but-empty custom slot included by `fam.custom`, a tagged-but-empty
frozen slot by its tags, a frozen slot with neither excluded, the
whole-library-empty `None`).

`apply_library_file` restores everything a parsed file carries **except
items** — pack info, collections (unvalidated, per the finding above), per-slot
metadata and scatter rules (normalised during parsing, since the rule key is
computable from a record's own `fam`/`id`/`set`). Items need pixels, which is
milestone 6.

### Milestone 6 — image handling

`cartalith-assets`, module `raster` (not feature-gated: no consumer needs an
image-free build). Crate work plus a thin port: `image`, `default-features =
false`, `features = ["png"]` — every asset this crate reads or writes is a PNG.

- `decode_png`/`encode_png`; `item_hash` (content hash from decoded pixels);
  `fit_to_bottom` (writes the transform); `render_item`, the reference's own
  shared render core (`drawItemOnly`/`renderItem` — per `ThumbnailRenderer`'s
  own comment, "shared render core (thumbnails, inspector preview, export
  bake)"), serving all three uses; `finalize_pack_texture_inv_mean`.
- `AssetDB::apply_library_file_with_items`: `apply_library_file`, then, for
  each item whose PNG bytes the caller supplies (keyed by `img` index —
  reading them out of an archive is the caller's job), decode, hash and
  `add_item`. A missing or undecodable item is skipped without failing the
  rest, as the reference's own `try{…}catch(_){}` does (lines 27920-27923).

**`itemHash` cannot usefully be golden-verified, for two independent
reasons.** Its algorithm (line 26913) is ported verbatim as arithmetic — a
`drawImage` downsample to 32×32, then a stride-7 FNV-1a variant (offset basis
`0x811c9dc5`, prime `0x01000193`, 32-bit wrapping multiply), suffixed
`-{w}x{h}`. But (1) **the hash is never serialized**: `_alExportEntries` writes
`{img,name,t}` (line 27890) and `_alImportProject` recomputes it after its own
decode (line 27922), so no process ever compares its hash with another's; and
(2) **it could not match anyway**: the Canvas spec leaves `drawImage`'s resample
kernel implementation-defined, so two *browsers* need not agree. "Matches
itself" is the only coherent bar — same decoded pixels in, same string out,
everywhere this binary runs (`image`'s `Triangle` filter stands in for the
unspecified resample). `render_item`'s geometry is exact; its resampling
(`CatmullRom`, for `imageSmoothingQuality:'high'`) is not reference-identical
for the same reason.

**Why only two functions here are golden.** The Node `vm` sandbox has no
`document`, canvas, `Image` or `createImageBitmap`, so `itemHash`,
`drawItemOnly`/`renderItem`, `encodeItemPng`, `decodeBytes` and
`decodePackImage` cannot run there. `finalizePackTexture` (the per-channel mean
across every pixel, clamped with `Math.max(1,mean)` so a near-black slot cannot
blow the reciprocal past 1, then reciprocated) and `fitToBottom` touch no DOM
API, and both are golden-verified; the rest are unit-tested, and
`src/raster.rs`'s module docs say so.

**A named non-port: pack import into the Library.** `AssetImporter.importPackZip`
(line 27067) decodes an external pack's manifest-declared images straight into
`AssetDB`, as distinct from restoring a project (`_alImportProject`, covered
above). This port's Assets ▸ Import pack loads a pack into the *renderer*
(`WorldGen::load_asset_pack`, from `app.gd`), not into the Library. Every piece
the equivalent would compose exists — `PackManifest`, `PackEntries`,
`decode_png`/`item_hash`/`fit_to_bottom`.

### Milestone 7 — renderer + Godot integration

`cartalith-godot`, module `pack` — the first consumer of `cartalith-assets`.
`WorldGen::load_asset_pack(path) -> bool` (a native path, the `load_save`
convention) and `has_asset_pack()` load and report the pack; the library
window's Apply to map loads one from memory instead (§10). **No default pack
ships** — nothing under `godot-project/` bundles pack art; the fixture every
test loads is `cartalith-assets/tests/fixtures/reference_pack.zip`.

- **Sprite compositing** (`composite_map_icons`, `drawMapIcons`'s Y-sorted
  pass): builds a rule table from the loaded manifest
  (`autopopulate_scatter_rules`); derives a `BIOME_INDEX` raster and a wetland
  mask from the already-generated height/temperature/rainfall fields
  (presentation-side, no new world data — `cartalith_civ::classify_biome` plus a
  `buildWetlandMask` equivalent); calls `place_map_icons_ruled`; composites each
  placed icon as a bilinear-sampled blit where the pack has art, or as the
  per-slot procedural glyph (`draw_icon_glyph`, all ten `PACK_ICON_SLOTS`
  shapes, "shrub" doubling as the reference's catch-all for an uncovered custom
  asset).
- **Ground-texture splat** (`land_color`'s splat branch in `render.rs`): the
  six `SPLAT_PAINT_SLOTS` channels, decoded and inverse-mean-baked at load,
  blended per cell using the exact `materialWeights` fractions and each
  material's own procedural ramp colour — a read-only consumer of both.
- **Painted ground tiles** (`biomes`/`terrains`): `decode_ground_family` fills
  `LoadedPack::biomes`/`::terrains` as positional `Vec<Option<GroundTile>>`
  tables, blended by the painted-cell path in §3. Reachable only through a
  painted cell — the paint tool's committed layers (`paint_bridge.rs`'s
  `PaintEditor`, driven from the WORLD dock), the port's counterpart of the
  reference's Cartography paint brush.

**Gating.** The reference gates icons behind `state.viz.icons` (default off)
and splat behind `assetPack.texAny` at `state.viz.splat` `0.7`. This port has
no icon toggle: `composite_map_icons` is a no-op whenever the loaded pack
yields no scatter-rule table (`current_scatter_rules` returns `None`), which is
also what keeps a pack-less render bit-identical. `golden_parity_render.rs`
passes unmodified at its original `1e-4` tolerance, since
`RenderCtx::with_splat` is never called on that path.

**The pack-import warning is owner-ruled text, not the reference's.** The
reference's `parsePackManifest` warns *"N pack section(s) not yet used by the
live map (…)"* and names `biomes`/`terrains` even though `_paintedTex` draws
them — its own comment at line 12164 calls that texturing "still follow-up
work". The port's list names the sections *this* port leaves undrawn instead:
the 2026-09-03 ruling authorised the first re-baseline of that string (and the
three fixtures pinning it), the 2026-09-04 audit re-derived the true unused
set, and Ruling W (2026-09-21) names all four remaining sections. The divergence
from the reference is permanent and disclosed; `DECISIONS.md` §7a is what it
overrides.

**Verification.** `cartalith-godot/tests/pack_compositing.rs` loads the real
fixture and proves, on a small synthetic world: sprite art blits where a relief
mountain places one; the procedural fallback fires for a biome the fixture has
no art for; a pack with no icon slots places nothing. `tests/paint_blend.rs`
carries the fixture's own `biomes/jungle.png` and `terrains/paved.png` end to
end. Windowed, on a real 512² world, the saved native `Image` showed a
hard-edged flat block where a mountain places (pack art — a procedural blend is
never hard-edged), an irregular checkerboard following land-material boundaries
(per-pixel splat), and soft translucent blobs elsewhere (the glyph fallback).

Deliberately not built: an `image` dependency in `cartalith-godot` for the
sprite resample (the icons are small; a hand-written bilinear sampler suffices),
and two decorative glyph variants (the arid jagged hill, the cold-mountain snow
cap), which the reference itself calls "procedural-fallback variety only" on
top of the base silhouette that is ported.

## 7. Out of scope for milestones 1-7

- **The Asset Library page UI and the sprite-sheet slicer's canvas
  interaction.** Presentation belongs to Godot; both were rebuilt later as
  GDScript over new bindings (§9-§11), and the slicer's pure core became
  milestone 8 (§11). `design/cartalith-menu-structure.md` §6 names the control
  inventory.
- **Authoring-side conveniences** the reference itself calls authoring-only:
  the standalone `asset_pack_compiler.html`, per-cell naming UI, the preview
  backdrop swatches.

## 8. Done means

A real `.zip` asset pack authored outside the app can be imported, validated
with the reference's own warnings, and rendered onto the map — sprites for the
slots it carries, procedural art for the slots it does not — while a render
with no pack stays bit-identical to the render before this phase existed. The
Library workspace that *authors*
such a pack is the separate GUI effort of §9-§11 (built in the DCC shell; the
panel-browser `GUI_SHELL_SCOPE.md` it was once filed under is superseded).

## 9. The GUI window (2026-08-19)

*A snapshot of one pass. The gaps it found are closed by §10 and §11.*

This pass built `DCC_SHELL_SPEC.md` §8's Asset library window
(`shell/asset_library_window.gd`, `AssetLibraryWindow`) and turned
`Assets ▸ ⧉ Asset library` and `▦ Sprite sheet slicer` from `_todo` into
`_live` in `menus.gd`.

**The durable finding: the design's families are not the engine's.** §8's prose
describes "24 families… Settlements, Terrain, Cartography, plus Collections";
`cartalith-assets` then had the reference's eight (§1; the sea-mark family came
later), confirmed by opening each in a headless run and counting its real slots
(`poi` shows the Library's ten, not the pack-import eight). The 24-family rail is the mockup's finer subdivision,
which no Rust type draws, so the window groups the engine's families the way the
crate does (`Family::is_texture()`, the `structures.*` trio) rather than
inventing a grouping to reach 24 — `GUI_GAP_REGISTER.md` AS-16, an owner
decision.

What the pass could not show was everything behind the two asset `#[func]`s
that existed (`load_asset_pack`, `has_asset_pack`): no `AssetDB` crossed the
boundary, so fill state, thumbnails, variants, tags and pack metadata were
disclosed as gaps rather than guessed (every slot drawn as a checkerboard, never
as "empty" or "filled"), and the slicer could preview a grid but not slice.

## 10. The library's binding surface (2026-08-20)

`cartalith-godot/src/asset_bridge.rs`: `AssetLibrarySession` wraps a live
`AssetDB` plus a decoded-pixel store kept index-parallel to its `store[uid]`
(`AssetDB` itself holds no pixels, by design, and every operation below needs
them). `WorldGen` holds one as `asset_library`, which survives a regenerate the
way `travel_library` does — an authored library describes the setting, not one
generation's output.

**The `as_*` `#[func]`s** (a `#[godot_api(secondary)]` block in `lib.rs`;
eighteen when this section was written — it said "twenty" — and more since):

- import (`as_import_item`, `as_add_custom_slot`); per-slot fill state
  (`as_family_slots`); inspector queries (`as_slot_summary`, `as_item_summary`);
- baked thumbnails (`as_thumbnail_png`, through `render_item`, the reference's
  own shared core); pack metadata (`as_pack_info`, `as_set_pack_info`);
- per-item transform (`as_set_item_transform`, `as_reset_item_transform`,
  writing `LibraryItem::transform` — AS-07); collections (`as_collections` —
  AS-12);
- removal (`as_remove_item`, `as_clear_library`); validation (`as_validate` →
  `library::run`);
- export (`as_export_pack_bytes`: bake every item, build a schema-2 manifest,
  `archive::write_pack`); apply to map (`as_apply_to_map` — the reference's own
  `applyToMap()`: build the pack in memory and load it into the renderer, no
  file round trip);
- five batch operations (`as_batch_tag`/`_collect`/`_rename`/`_duplicate`/
  `_delete`), each read off the reference's `alBatch*` handlers (~lines
  28045-28090) rather than guessed from the button labels. **Batch Rename is
  split exactly as the reference splits it:** a custom slot is renamed for
  real (`AssetDB::rename_custom_slot`); a frozen slot renames its *item
  variants* in place (`AssetDB::item_mut`), because frozen slot names are the
  constant `slot_title`;
- the slicer (§11).

`engine_bridge.gd` wraps each in a `has_method`-guarded forwarder (the `tl_*`
convention); the window drives real fill state, thumbnails, the inspector,
import into the focused slot, batch operations through a small text-prompt
dialog, Validate, Clear, Export and Apply. `menus.gd` gained `Assets ▸ Asset
pack ▸` (`DCC_CONTROL_INDEX.md` §2.3.1's omission O2): stats, metadata and the
Build group call the engine directly; Edit and Batch open the window, because
both need slot or selection context only the grid provides.

**Design choices recorded against the register:** the "Unassigned imports" rail
bucket (AS-12) is a reserved custom-slot set (`UNASSIGNED_SET`,
`asset_library_window.gd`), not a slot-less concept the engine does not have.
Declined because the engine has no counterpart (`STATUS.md`): AS-14, a
user-picked active variant (variant choice is weighted and seeded); AS-15, a
per-slot anchor (`Anchor` is a *family* property); AS-16, the 24-family rail
(§9).

**Verified** by a headless `--script` drive (`WorldGen.new()`, no scene tree)
through the whole authoring cycle: import into a frozen slot with real fill
state and a real thumbnail; a custom-slot import; batch tag; batch duplicate,
which correctly tripped an "Identical images" warning; pack metadata round
trip; validate; export; a disk round trip through `load_asset_pack` on a second
`WorldGen`; apply to map; batch delete (a frozen slot is emptied, not
removed); clear.

## 11. Milestone 8 — the sprite-sheet slicer (2026-08-20)

The slicer half of the owner's report that "the asset slicer and management
system lacks the functionality the html had" (§10 was the management half);
`GUI_GAP_REGISTER.md` AS-09/AS-10/AS-11.

### What the reference actually does

`SpriteSheetImporter` (lines **27465-27870**, the whole object literal —
`#alSlicerBtn` opens it at 28038), read directly. It disagrees with
`DCC_SHELL_SPEC.md` §8's control list, which the Godot modal was first built
to:

| §8's control | The reference |
|---|---|
| Columns / Rows | `#alSlCols`/`#alSlRows`, `clampInt(v,1,128)` (line 27580) |
| Spacing | `#alSlSpacing` — a **half-gutter on interior edges only**, not a pitch (line 27596) |
| Margin | no such control; a *draggable* `gridRect`, of which a uniform margin is one case |
| Skip empty cells | `#alSlSkip` — `isBlank`, alpha **> 8** (line 27768) |
| Trim transparent edges | **does not exist.** The second pixel toggle is background → transparent, a chroma key (`applyChroma`, line 27603) |

**The half-gutter is the finding that matters.** `computeCells` starts each cell
at its division line moved in by `spacing/2` *unless it is the first*
column/row, and ends it at the next line moved back by `spacing/2` *unless it is
the last* — so outer cells come out `spacing/2` wider than interior ones. The
classic equal-cell formula (`cell = (span − 2·margin − (n−1)·gutter)/n`) does
not reproduce it, and that formula is what the Godot overlay carried before
this milestone: the preview drew a grid the slice would not have followed.
Golden fixture `6x4 with spacing 8` over 3072 px pins it — 508 px outer, 504 px
interior, where the equal-cell formula says 505.33 for all six.

### The port

`cartalith-assets/src/slicer.rs`: `compute_cells` (the grid, over line
*fractions*), `crop_cell` + `cell_source_rect` (`cropCell`'s
`Math.max(0,Math.round(x))` / `Math.max(1,Math.round(w))` rounding and its
clipped 1:1 blit — a source rect hanging off the sheet lands transparent, as
`ctx.drawImage` does), `apply_chroma`, `is_blank`, `slice_sheet`, `count_cells`,
`sheet_base_name` (`name.replace(/\.[^.]+$/,'')`), and the two default naming
conventions (`base_r{R}c{C}` for a slot target, `cell N` per cell).

`asset_bridge.rs` holds a `LoadedSheet` on the session, so the modal's live
readout re-runs the real detection pass on every spinbox change without
re-sending a multi-megabyte PNG; `load_sheet`/`clear_sheet`/`slice_preview`/
`apply_slice` and a `SliceTarget` enum. `import_item`'s construction half is
shared as `insert_decoded`, not duplicated. Bound as `as_load_sheet`,
`as_clear_sheet`, `as_slice_preview`, `as_slice_apply`.

**Two port-side additions, beyond the reference:**

1. **`trim_transparent_edges`**, which §8 asks for and the reference lacks. It
   uses the reference's own `BLANK_ALPHA_THRESHOLD` (alpha > 8), so it can
   never disagree with `is_blank` about what counts as content; it has no
   golden fixtures because there is nothing to be golden against, and nothing
   ported depends on it. The reference's real second toggle, chroma keying, is
   wired too.
2. **`SliceTarget::Family`**, §8's "Assign to family" + "Fill from
   first-empty/overwrite", composed from reference primitives: one cell per
   slot in frozen vocabulary order, via `add_item` — no new arithmetic, nothing
   golden-covered changed. The reference's own three targets (a flat slot
   dropdown with three special entries) are ported exactly, including
   `store[uid]=[item]`'s replace-and-stop for a single-image family.

### The Godot side

`asset_library_window.gd`'s slicer modal: the `N cells detected · M non-empty`
readout is `as_slice_preview`'s real detection pass, and the grid overlay draws
**engine-computed** cell spans (`CellGrid::column_spans`/`row_spans`, handed
over as `PackedFloat64Array`s plus the blank-cell indices) rather than
recomputing the half-gutter in GDScript — the drift the "no numbers in GDScript"
rule exists to prevent. Slicing is non-destructive: the sheet stays loaded for a
re-slice with different settings, and closing the modal drops it.

**Canvas interaction** (AS-17): `SheetPreview` has wheel zoom, middle-drag pan,
click-to-select a cell and a draggable margin handle, plus per-interior-line
dragging (`SliceGrid::with_lines`, `slicer::move_line`; bound as
`as_slicer_move_line` and `as_uniform_lines`) and cell-scoped slicing
(`asset_bridge::SliceParams::only_cell`). Writing `compute_cells` against line
fractions rather than `cols`/`rows` is what let the hand-placed lines land
without touching the golden-verified arithmetic.

### Verified

- `tests/golden_parity_slicer.rs`: `computeCells`, `cropCell` rounding,
  `isBlank` and `applyChroma` fixtures from a Node `vm` harness that lifts lines
  27465-27870, **asserts both ends of the range and the presence of all four
  functions before evaluating anything**, and whose fixture tables assert their
  own size and total cell count (the "watch for silently-empty golden output"
  rule).
- **Seven mutations, seven killed**, each re-run against a fresh build: alpha
  threshold 8→7; chroma `<=`→`<`; interior half-gutter → full gutter; gutter
  applied to outer edges too; `index = row*cols+col` → `col*rows+row`; crop
  extent floor 1→0; count clamp 128→256.
- A headless drive through the real gdext boundary on a built 64×32 PNG: load
  (and a garbage-bytes rejection); detection (`total=2 non_empty=1 blank=[1]`,
  spans `[0,32]/[32,64]`); the half-gutter across the boundary (outer 6.67 px
  against interior 2.67 px at 6 columns, spacing 8); a too-dense grid reporting
  itself; an impossible margin refused; a family slice landing in
  `settlement:hamlet` with a real baked thumbnail; a second slice off the same
  sheet (non-destructive); trim cropping a 64×32 cell to 32×32; chroma keying
  everything out and being **refused** rather than silently adding nothing;
  four malformed targets each returning an error; preview-after-clear erroring
  rather than crashing.
