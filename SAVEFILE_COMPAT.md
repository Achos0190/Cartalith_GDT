# The Cartalith project archive — format specification

**Status: normative.** This document defines a file format that two
independent programs implement — the Rust/Godot port in this repository, and
(intended) the `Cartalith Gen1` HTML app. It is written to be implementable
from this text alone, in any language, with no access to either codebase.

It supersedes the earlier version of this file, which was an *observational*
note about what the HTML app's `exportZip()` happened to produce.

- **§1-§3** — what changed, why, and the container.
- **§4** — how a reader tells the two layouts apart. Read this before anything else.
- **§5** — the tree, with the reasoning for every boundary.
- **§6** — conformance: what MUST be written, what MAY be, minimal writer and reader.
- **§7-§13** — entry-by-entry specification.
- **§14** — JSON conventions. **Non-optional reading**; §14.2 describes a bug
  that has already cost this project one shipped subsystem.
- **§15** — the legacy flat layout, read-only.
- **§16** — what is deliberately not stored, and why.
- **§17** — notes specific to this port's own implementation (non-normative).
- **§18** — why the container is deflate, measured. Non-normative, but it is
  the reason §3.3 says what it says, and it carries two levers that are open
  owner decisions rather than settled ones.

The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are used in the
RFC 2119 sense.

---

## 1. Owner decision, 2026-08-25

Two directions, both recorded verbatim because both are load-bearing and
neither is this document's own judgment.

> "The save zip should have all project files and the folder structure should
> be a clean and clear tree without semantic overlap (not atlas and cartography
> and both storing map tiles)"

> "Agreed, importing and reading should work from the old and new format, and
> saving/exporting should strictly be the new format. (document this properly
> as I'd like to upgrade the html version to include some of the new
> functionality."

So:

- **Readers MUST accept both layouts** — the flat legacy layout (§15) that
  every `Cartalith Gen1` export up to v2.10 produces, and the tree (§5).
- **Writers MUST produce only the tree.** No writer emits the flat layout as
  its project format.
- **This document is the specification the HTML app will be upgraded
  against.** That is why it names no Rust type, no function, and no file in
  this repository outside §17.

`DECISIONS.md` §7h records this as an owner decision with the same quotes, so
a later reader can tell a settled decision from a revisitable one.

### 1.1 One interoperability writer survives, and is not this format

A writer MAY additionally offer an **export** in the flat legacy layout, for
the express purpose of handing a file to an unmodified pre-upgrade
`Cartalith Gen1` build. That is an export path, not a save path: it is lossy
by construction (it can carry no part of §9-§13), and it MUST be presented to
the user as an export rather than as saving their project.

---

## 2. Why a tree at all

The flat layout has seven entries at the archive root and no directories. It
was adequate while an archive held nothing but terrain. It cannot hold a
project: settlements, factions, territory, roads, labels, icons, recorded
history, links out to a notes vault and the user's own drawn annotations are
all real content, and dropping them all at the root produces exactly the
collision the owner's own example names — two subsystems each believing they
own "the map tiles", each writing them under a name of its own.

The tree's organising rule, which every boundary in §5 follows, is:

> **Group by what the data *is*, not by which subsystem produced it.**

The test for any new payload is "what kind of thing is this?", never "who made
it?". Producer-based folders are how `atlas/` and `cartography/` both end up
holding tiles.

---

## 3. The container

A Cartalith project archive is a **standard PKZIP file**. Nothing about it is
custom.

| Property | Requirement |
|---|---|
| Signatures | Standard: local header `PK\x03\x04`, central directory `PK\x01\x02`, end of central directory `PK\x05\x06` |
| Compression | Method **0 (store)** or method **8 (deflate)** only. Writers SHOULD use deflate. Readers MUST support both. See §3.3. |
| Checksums | Standard CRC-32 per entry |
| Encryption | MUST NOT be used |
| Entry names | UTF-8, forward slashes as separators, no leading `/`, no `.` or `..` segments, no backslashes |
| Directory entries | Writers SHOULD NOT emit explicit directory entries. Readers MUST tolerate them and MUST ignore them. |
| Case | Entry names are **case-sensitive**. Every name in this specification is lowercase except `README.md`. |
| Duplicate names | Writers MUST NOT emit two entries with the same name. If a reader encounters duplicates it MUST use the first and ignore the rest. |
| Zip64 | Writers SHOULD keep an archive below 4 GiB and 65 535 entries so that zip64 is never required. Readers that intend to open large worlds MUST support zip64 — a single 8192×8192 32-bit raster is 256 MiB before compression, and six of them plus recorded history can cross the limit. |

Both `store` and `deflate` are what a browser can produce: the reference's own
`zipStore()` writes raw deflate through `CompressionStream('deflate-raw')`, and
every mainstream JavaScript zip library reads and writes both. Nothing in this
format needs anything else.

### 3.1 Entry order

Entry order is **not significant**. Readers MUST NOT depend on it; they MUST
locate entries by name.

Writers SHOULD write `project.json` first, so that a partially transferred
archive is diagnosable, and SHOULD otherwise write in the order §5 lists.

### 3.2 Writing MUST be atomic from the user's point of view

A writer MUST NOT leave a previously good archive replaced by a partial one.
Build the archive fully (in memory, or in a temporary file beside the target)
and move it into place only once it is complete. A save that fails must leave
the previous save intact. This is the one behaviour a save command cannot get
wrong.

### 3.3 Compression methods, and the one a reader cannot decode

**A writer MUST use only method 0 (store) and method 8 (deflate).** No other
method may appear in a conforming archive — in particular not method 93
(Zstandard, standardised by APPNOTE 6.3.8), nor 12 (bzip2), 14 (LZMA) or 95
(XZ). All four are legal PKZIP; none of them is decodable in a browser
without shipping a decoder alongside the page. §18 records what was measured
before this rule was written, and why the obvious workaround — deflate for the
small documents, something denser for the big rasters — does not work.

**A reader MUST support method 0 and method 8. Nothing more is required.** A
reader MAY support more; because no conforming writer emits more, supporting
more changes nothing about the format and is never a substitute for the rule
above.

**An entry whose compression method the reader cannot decode is intact and
unreadable, which is not the same thing as absent. A reader MUST NOT conflate
the two.** Having distinguished them, §6.4 decides the rest: refuse the
archive when the entry is `project.json` or `rasters/heightmap.f32`, and
otherwise skip it and report it, naming both the entry and the method number
so the user can be told what is actually wrong.

A reader whose zip library refuses the whole archive the moment it meets an
unknown method is **also conforming** — both known JavaScript readers do
exactly that (§18), and the archive was invalid anyway. What is not conforming
is reporting the entry as one that was never written: §6.2's obligation turns
on exactly that distinction, because an entry reported as absent is an entry
the next save drops without telling anyone.

---

## 4. Telling the two layouts apart

**The test is the presence of the entry `project.json` at the archive root.**

| `project.json` present | Layout |
|---|---|
| yes | The tree (§5). Read `project.json` first; it names the format version. |
| no | The flat legacy layout (§15). |

That is the whole test. It is one name lookup in the central directory, it
requires no heuristics over entry names, and it cannot be confused by an
archive that contains extra files. A reader MUST use this test and MUST NOT
guess from the presence or absence of any other entry.

`project.json` carries the version:

```json
{ "format": "cartalith-project", "format_version": 1 }
```

- A reader MUST check `format`. If it is present and is not the string
  `"cartalith-project"`, the archive is not a Cartalith project and the reader
  MUST refuse it rather than guess.
- A reader MUST check `format_version`. It is an integer.
  - Equal to a version the reader knows: read normally.
  - **Greater** than any version the reader knows: the reader SHOULD still read
    it, applying §14.3's unknown-member rule, and SHOULD warn the user that the
    archive was written by a newer program and that parts of it were not
    understood. It MUST NOT silently discard the file.
  - Less than the reader's own version: read according to that older version's
    rules. Version 1 is the only version defined; there is nothing older.
- A writer MUST write both members.

**Version 1 is the version this document defines.** A future version number
will be accompanied by a revision of this document describing what changed;
until then, every conforming archive says `1`.

---

## 5. The tree

Reserved entries are named here so that a second implementation does not
invent a competing location for them. A reserved entry is **not written by any
current implementation**; §16 says why for each.

```
project.json                  MUST   manifest, version marker, world identity
params.json                   SHOULD  the generation parameters
README.md                     MAY     plain-text description of the archive
preview.png                   MAY     a small thumbnail of the map
appearance.json               MAY     presentation settings
vault.json                    MAY     links out to an external Markdown vault

rasters/                              one value per grid cell — see §8
  heightmap.f32               MUST
  temperature.f32             SHOULD
  rainfall.f32                SHOULD
  volcanic_field.f32          MAY
  impact_field.f32            MAY
  strahler_order.u8           MAY
  territory.i32               MAY
  provinces.i32               MAY
  water_bodies.u8             MAY
  agrarian_density.f32        MAY
  biome.u8                    reserved
  lithology.u8                reserved
  koppen.u8                   reserved
  wildlife.u8                 reserved

entities/                             discrete, id-bearing things — see §9
  settlements.json            MAY
  factions.json               MAY
  ways.json                   MAY
  provinces.json              MAY
  continents.json             MAY
  journeys.json               reserved

history/                              recorded past states — see §10
  timeline.json               MAY
  territory/<year>.i32        MAY

annotations/                          marks on the sheet — see §11
  labels.json                 MAY
  icons.json                  MAY
  regions.json                MAY

library/                              setting-level definitions — see §12
  assets.json                 reserved
  travel.json                 reserved

drafts/                               uncommitted edits — see §16.3
  paint.json                  reserved
  sculpt.json                 reserved
```

### 5.1 Why each boundary falls where it does

Each rule is a *test* a second implementer can apply to a payload this
document does not yet mention.

**`rasters/` — "does it have one value per grid cell?"**
Everything with one value per cell of the `GW × GH` grid lives here, whatever
computed it. Elevation, climate, hydrology, biome, territory and the water-body
classification are all the same *kind* of thing — a grid — and a reader that
wants "every raster" must be able to enumerate one directory.

The folder is deliberately **flat**. Nesting rasters by producing subsystem
(`rasters/climate/`, `rasters/civ/`) reintroduces exactly the overlap the
owner objected to: `water_bodies` is derived by the hydrology pass and consumed
by the civilisation pass, `territory` is civ output that the renderer treats as
cartography, and any producer-based split forces an arbitrary answer. Fifteen
files in one flat directory is clear. Fifteen files across four directories,
half of which could defensibly hold any given one, is not.

The file extension names the **element type**, not a format: `.f32`, `.u8`,
`.i32`. This is the one place the format uses an extension to carry meaning,
and it does so because a reader must know the element width before it can read
a single byte.

**`entities/` — "can something else point at it by id?"**
A settlement, a faction, a province, a continent, a road, a journey. Each has a
stable id; a note in the vault, a journey's route or a province's capital
reference can name one. If a payload is referenceable, it is an entity.

**`history/` — "is this a past state rather than the present one?"**
A recorded year holds its own settlements, roads and territory. Those are the
same *shapes* as `entities/` and `rasters/` hold, and putting them in the same
places would mean two settlements with the same id in one archive meaning
different things at different times. Separating present from past is what makes
`entities/settlements.json` unambiguously "the settlements that exist now".

**`annotations/` — "does anything downstream read it?"**
A label, an icon, a selected region. Deleting every one of them changes nothing
any other subsystem computes; they exist for the reader of the map. If a
payload has no model semantics, it is an annotation, not an entity.

**`library/` — "does it survive regenerating the world?"**
An asset library and a travel library describe the *setting* — what a cart is,
what a mountain icon looks like — not this particular world. They outlive any
one generation. Nothing else in the archive does.

**`drafts/` — "is it uncommitted?"**
In-progress, revertible edit state. Separated from everything else because a
reader that does not implement editing can skip the entire directory and lose
nothing.

**Root-level single files.** `appearance.json` and `vault.json` are single
documents with no family, and a directory holding one file is noise. If either
grows a second sibling it becomes a directory, and that will be a
`format_version` change.

**There is no `atlas/`, no `cartography/`, and no `tiles/`.** The owner's own
example of the failure is resolved by deleting the concept rather than by
picking a winner between two homes for it — see §16.1.

---

## 6. Conformance

### 6.1 A minimal conforming writer

Write a zip containing exactly two entries:

1. `project.json` with `format`, `format_version` and a `world` object giving
   `grid_width`, `grid_height`, `wrap_x`, `map_width_km`, `sea_level` and
   `seed` (§7).
2. `rasters/heightmap.f32` — `grid_width × grid_height` little-endian 32-bit
   floats, row-major (§8).

That archive is valid. Every other entry is optional enrichment. A writer that
also emits `rasters/temperature.f32` and `rasters/rainfall.f32` produces
something a renderer can colour without inventing a climate, which is why those
two are SHOULD rather than MAY.

### 6.2 A minimal conforming reader, without data loss

"Without data loss" means: **anything the reader does not understand survives a
load-and-save cycle unchanged.** That is the property that lets two
implementations evolve independently, and it is the reason for §14.3.

A reader that intends to write the archive back MUST therefore either

- retain the raw bytes of every entry it did not consume and re-emit them
  unchanged, **or**
- refuse to overwrite an archive it did not fully understand, and require the
  user to save to a new file.

A read-only reader (a viewer, an importer) has no such obligation and MAY
simply ignore what it does not know.

### 6.3 The unknown-entry rule

**A reader MUST ignore entries it does not recognise. It MUST NOT treat an
unrecognised entry as corruption and MUST NOT fail the archive because of
one.**

This rule predates the tree — the flat layout already required it, because a
real HTML-app export always carried more than any one reader wanted — and it is
what makes a two-implementation format survivable. It is the mechanism by which
either implementation can add a payload without breaking the other, and by
which the reserved slots in §5 can be filled later by whichever implementation
gets there first.

The same rule applies **inside** JSON documents, at every level: see §14.3.

### 6.4 A recognised entry that is damaged

Distinct from an unrecognised entry, and handled differently:

- If `project.json` is missing, the archive is the flat layout (§4).
- If `project.json` is present but unparseable, or `format` is wrong, the
  reader MUST refuse the archive.
- If `rasters/heightmap.f32` is missing or its length is not
  `grid_width × grid_height × 4`, the reader MUST refuse the archive.
- For **every other** entry: a reader MUST NOT fail the archive. It MUST skip
  the damaged entry, continue, and report what it skipped. A corrupt
  `annotations/labels.json` must not cost the user their world.

---

## 7. `project.json`

Required. UTF-8 JSON. The manifest, the version marker, and the identity of
the grid every raster is measured against.

```json
{
  "format": "cartalith-project",
  "format_version": 1,
  "generator": "cartalith-native 0.1.0",
  "created": "2026-08-25T14:03:11Z",
  "world": {
    "grid_width": 512,
    "grid_height": 512,
    "wrap_x": false,
    "map_width_km": 800.0,
    "sea_level": 0.42,
    "seed": 24601
  }
}
```

| Member | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | MUST | Always `"cartalith-project"`. §4. |
| `format_version` | integer | MUST | `1`. §4. |
| `generator` | string | SHOULD | Free-form name and version of the writing program. Provenance only; no reader may branch on it. |
| `created` | string | MAY | RFC 3339 UTC timestamp. Provenance only. |
| `world.grid_width` | integer ≥ 1 | MUST | `GW`. Cells across. |
| `world.grid_height` | integer ≥ 1 | MUST | `GH`. Cells down. |
| `world.wrap_x` | boolean | MUST | Whether the grid wraps in longitude — a whole planet rather than a region. Corresponds to the HTML app's `state.world`. Renamed here because a member called `world` inside an object called `world` is not a name. |
| `world.map_width_km` | number > 0 | MUST | Real-world width of the map. **Height is derived, not stored**: cells are square in kilometres, so height is `map_width_km × grid_height / grid_width`. A stored height that disagreed would contradict every distance, gradient and route length in the archive. |
| `world.sea_level` | number in `[0,1]` | MUST | The **effective** threshold against the heightmap's own `[0,1]` range. A cell is land where `heightmap[i] >= sea_level`. If the generator re-anchored sea level from a world-structure archetype, this is the re-anchored value, not the user's input — the user's input belongs in `params.json`. |
| `world.seed` | integer | MUST | The generation seed. Range: §14.1. |

A reader MUST refuse the archive if any MUST member is missing or is not of
the stated type.

---

## 8. `rasters/` — the grid payloads

Every entry under `rasters/` is a **bare little-endian binary dump**. No
header, no length prefix, no padding, no alignment guarantee. The entry's
uncompressed length is exactly `GW × GH × element_size` bytes.

| Extension | Element | Size | JavaScript view |
|---|---|---|---|
| `.f32` | IEEE-754 binary32 | 4 | `Float32Array` |
| `.i32` | two's-complement signed | 4 | `Int32Array` |
| `.u8` | unsigned | 1 | `Uint8Array` |

**Byte order is little-endian, always**, for every multi-byte element. This is
the native order of every platform either implementation targets, so a
JavaScript reader may use a typed-array view directly; a reader on a big-endian
platform MUST byte-swap.

**Index formula.** The value for the cell at column `x` (`0 ≤ x < GW`) and row
`y` (`0 ≤ y < GH`) is at element index

```
i = y * GW + x
```

Row-major, origin at the top-left, `x` increasing right, `y` increasing down.
There is no other layout and no per-raster variation.

**Length validation is mandatory.** A raster entry carries no length of its
own, so a truncated one is not a parse error — it is a silently truncated
world. A reader MUST compare the entry's uncompressed length against
`GW × GH × element_size` and MUST reject any raster whose length disagrees
(refusing the archive for `heightmap`, skipping with a report for any other —
§6.4). A writer MUST NOT emit a raster of any other length.

### 8.1 The defined rasters

| Path | Element | Required | Meaning | If absent |
|---|---|---|---|---|
| `rasters/heightmap.f32` | f32 | MUST | Normalised elevation. `[0,1]` after the generator's own stretch; compare against `world.sea_level`. | Refuse the archive. |
| `rasters/temperature.f32` | f32 | SHOULD | Degrees Celsius. | Treat as absent, not as zero. A reader MUST report that the project carries no temperature rather than render a frozen world. |
| `rasters/rainfall.f32` | f32 | SHOULD | Precipitation, generator units. | As `temperature`. |
| `rasters/volcanic_field.f32` | f32 | MAY | Byproduct of the volcanism stamping pass. | All zero. Zero is the true value for "no volcanism", so this substitution is honest. |
| `rasters/impact_field.f32` | f32 | MAY | Byproduct of the crater stamping pass. | All zero, as above. |
| `rasters/strahler_order.u8` | u8 | MAY | Strahler stream order. `0` = not a channel. Orders above 255 saturate at 255. | All zero — "no channels". |
| `rasters/territory.i32` | i32 | MAY | Owning faction id per cell. `0` = unowned (water, or unreachable). Ids index `entities/factions.json`. | No territory. |
| `rasters/provinces.i32` | i32 | MAY | Province id per cell. `0` = no province. Ids match `entities/provinces.json`. | No provinces. |
| `rasters/water_bodies.u8` | u8 | MAY | `0` = land, `1` = ocean, `2` = lake. | Absent. A reader that needs it MUST recompute it from `heightmap` and `sea_level` rather than assume land. |
| `rasters/agrarian_density.f32` | f32 | MAY | Carrying-capacity density used by population simulation. | Absent. |

`biome.u8`, `lithology.u8`, `koppen.u8` and `wildlife.u8` are **reserved**:
named so that no implementation invents a second location for them, not
written by any implementation today (§16.4).

---

## 9. `entities/` — the world's discrete things

All UTF-8 JSON. All optional. Read §14 before implementing any of them.

Every entity carries an integer `id` that other parts of the archive reference.
Ids are stable across a regeneration of the same world and MUST satisfy §14.1's
range rule. `0` is the "unassigned" sentinel and MUST NOT be used as a real id
for a settlement or a way; for a **faction** id `0` is meaningful and means
"Unclaimed" (§9.2).

### 9.1 `entities/settlements.json`

```json
{
  "next_id": 43,
  "settlements": [
    {
      "id": 12,
      "x": 104,
      "y": 57,
      "name": "Sevjuniana",
      "population": 41230,
      "faction": 3,
      "kind": "city",
      "capital": true,
      "coastal": false,
      "suitability": 0.7341,
      "village_seeded": false,
      "trade": { "exports": ["grain", "timber"], "imports": ["iron"] },
      "extras": {
        "specialisation": "port",
        "traits": ["walled", "university"],
        "history": "Founded after the second flood.",
        "age": 320,
        "walls": true
      }
    }
  ]
}
```

| Member | Type | Required | Meaning |
|---|---|---|---|
| `next_id` | integer | SHOULD | The next id an editor should hand out. A reader MUST raise it to `max(id) + 1` if the stored value is lower — otherwise a newly placed settlement collides with an existing one. Absent: derive it the same way. |
| `settlements[].id` | integer ≥ 1 | MUST | Stable id. Unique within the array. |
| `settlements[].x`, `.y` | integer | MUST | Grid cell, `0 ≤ x < GW`, `0 ≤ y < GH`. A reader MUST discard a settlement outside the grid. |
| `settlements[].name` | string | MUST | May be empty. |
| `settlements[].population` | integer ≥ 0 | MUST | |
| `settlements[].faction` | integer ≥ 0 | MUST | Index into `entities/factions.json`. `0` = Unclaimed. A reader MUST clamp an out-of-range faction to `0` rather than drop the settlement. |
| `settlements[].kind` | string | MUST | One of `metropolis`, `capital`, `city`, `town`, `village`, `hamlet`. An unrecognised value MUST be read as `town`, and reported. |
| `settlements[].capital` | boolean | MUST | Whether this is its faction's seat. Independent of `kind`. |
| `settlements[].coastal` | boolean | MUST | Whether the settlement is a port. Computed from its final position. |
| `settlements[].suitability` | number `[0,1]` | SHOULD | The placement score. Display and diagnostics only. Absent: `0`. |
| `settlements[].village_seeded` | boolean | MAY | `true` if added by the optional village-seeding pass rather than by primary placement. Matters because villages are not road-network nodes: a network rebuild that fed them back in would restructure the world. Absent: `false`. |
| `settlements[].trade` | object | MAY | `exports` and `imports`, each an array of resource key strings. Unknown keys MUST be ignored. Absent: no trade profile. |
| `settlements[].extras` | object | MAY | Author-editable fields with no generator equivalent. All members optional. `age` is an integer or `null` (`null` = infer from population). `walls` is a boolean or `null` (`null` = automatic). `traits` is an array of strings in insertion order; order is significant and MUST be preserved. |

### 9.2 `entities/factions.json`

```json
{
  "factions": [
    { "id": 0, "name": "Unclaimed", "culture": "", "religion": "none",
      "government": "none", "ag_tech": "traditionalAgrarian",
      "color": [128, 128, 128], "user_color": null },
    { "id": 1, "name": "Verath", "culture": "highland", "religion": "none",
      "government": "monarchy", "ag_tech": "traditionalAgrarian",
      "color": [214, 39, 40], "user_color": [30, 120, 200] }
  ]
}
```

The array is **dense and index-addressed**: `factions[i].id` MUST equal `i`.
Index `0` is always "Unclaimed" and MUST be present; it is the value
`territory` uses for unowned cells and settlements use for no faction. A reader
MUST NOT allow index `0` to be removed.

| Member | Type | Required | Meaning |
|---|---|---|---|
| `id` | integer ≥ 0 | MUST | Equals the array index. |
| `name` | string | MUST | |
| `culture`, `religion`, `government`, `ag_tech` | string | MUST | Vocabulary keys. An unrecognised key MUST be preserved on write-back and MAY be shown to the user as-is; a reader MUST NOT substitute a default silently. |
| `color` | array of 3 integers `[0,255]` | MUST | The palette colour derived from the faction index. |
| `user_color` | array of 3 integers, or `null` | MUST | The author's chosen identity colour, or `null` for "use the palette rule". A separate member rather than an overwrite of `color`, so that clearing the override restores the palette colour rather than losing it. |

### 9.3 `entities/ways.json`

**One home for every linear route in the project.** Roads, sea lanes and
hand-drawn ways and routes are all polylines over the same grid, and a reader
that wants "every way" must not have to know which tool drew each one. This is
the boundary the owner's "not atlas and cartography" instruction is most
directly about.

```json
{
  "roads":     [ { "id": 7, "name": "Verath Road", "points": [[10.0,4.0],[11.5,5.25]],
                   "breaks": [], "length_km": 12.5, "class": "highway",
                   "from": 0, "to": 4, "hidden": false } ],
  "sea_lanes": [ { "name": "Northern Passage", "points": [[3.0,9.0]], "breaks": [],
                   "length_km": 402.0 } ],
  "manual":    [ { "name": "", "points": [[1.0,1.0]], "breaks": [], "length_km": 3.0,
                   "kind": "track", "sea": false, "hidden": false } ],
  "routes":    [ { "name": "", "points": [[1.0,1.0]], "breaks": [], "length_km": 3.0,
                   "mode": "land", "unreachable_legs": 0 } ]
}
```

All four arrays are optional; an absent array means none of that kind.

Common members:

| Member | Type | Meaning |
|---|---|---|
| `points` | array of `[x, y]` number pairs | Grid coordinates, **fractional** — a smoothed polyline does not pass through cell centres. Not integers. |
| `breaks` | array of integers | Indices into `points` at which the polyline is **discontinuous**. A renderer starts a new sub-path at each listed index. Empty for a single unbroken line. Every value MUST be a valid index into `points`; a reader MUST drop out-of-range values. |
| `length_km` | number | Total length along the polyline in kilometres. Derived, and stored because recomputing it needs `map_width_km` and the same distance convention. A reader MAY recompute it. |
| `name` | string | May be empty. An empty name is a real resting state, not a placeholder: a display fallback such as "Journey 3" is computed at draw time by the caller and MUST NOT be stored, because a stored one survives a deletion and mislabels its neighbour. |
| `hidden` | boolean | Whether the way is suppressed from rendering. |

`roads[]` additionally carries `id` (stable, §9), `class` (one of `highway`,
`regional`, `road`, `track` — unrecognised reads as `track`), and `from`/`to`,
which are **indices into `entities/settlements.json`'s array**, not settlement
ids. A reader MUST drop a road whose `from` or `to` is out of range.

`manual[]` carries `kind` (one of `road`, `track`, `sea_lane`, `ancient`) and
`sea` (whether it crosses water). `routes[]` carries `mode` (one of `land`,
`water`, `mixed`) and `unreachable_legs`, the count of legs the router could
not connect — a real, non-error outcome that the UI reports.

### 9.4 `entities/provinces.json`

```json
{ "provinces": [ { "id": 1, "faction": 3, "name": "Upper Verath",
                   "capital_settlement_index": 4 } ] }
```

`id` matches the values in `rasters/provinces.i32`. `capital_settlement_index`
is an **index into `entities/settlements.json`'s array**, not a settlement id;
a reader MUST drop a province whose index is out of range.

### 9.5 `entities/continents.json`

```json
{ "continents": [ { "id": 1, "name": "Aurelia", "cells": 104320,
                    "min_x": 3, "min_y": 8, "max_x": 402, "max_y": 300,
                    "cx": 210.4, "cy": 150.9, "faction": 3 } ] }
```

Metadata only — there is deliberately no per-continent raster, because a
labelled landmass raster costs `GW × GH × 4` bytes to store a value derivable
from `heightmap` and `sea_level`. `id` is a 1-based rank by cell count,
largest first. `min_*`/`max_*` are **inclusive** cell bounds. `cx`/`cy` is the
cell-space centroid. `faction` is whichever faction holds the most cells here,
or `0`.

### 9.6 `entities/journeys.json` — reserved

Not written by any implementation today. The slot and its shape are specified
here so that the two implementations do not diverge when it lands:

```json
{
  "next_id": 3,
  "journeys": [
    { "id": 1, "name": "The salt road", "party_preset": "merchant_caravan",
      "route": { "points": [[10.0,4.0]], "breaks": [], "length_km": 120.0,
                 "mode": "land" },
      "start_year": 412 }
  ]
}
```

A journey is a *party travelling*, and references a route rather than being
one — which is why it is its own entity and not a fifth array in
`entities/ways.json`. `party_preset` names an entry in `library/travel.json`;
a reader MUST tolerate a name that resolves to nothing and MUST show the
journey rather than drop it.

---

## 10. `history/` — recorded past states

### 10.1 `history/timeline.json`

```json
{
  "year": 412,
  "years": [
    { "year": 0,   "settlements": [ /* §9.1 settlement objects */ ],
                   "ways":        [ /* §9.3 road objects */ ] },
    { "year": 120, "settlements": [], "ways": [] }
  ]
}
```

| Member | Type | Required | Meaning |
|---|---|---|---|
| `year` | integer | MUST | The **currently selected** year cursor. `0` if no year has been recorded. |
| `years` | array | MUST | Recorded snapshots. MUST be sorted ascending by `year`; a reader MUST sort them if they are not. `year` values MUST be unique. |
| `years[].year` | integer | MUST | |
| `years[].settlements` | array | MUST | The settlements as they were, using §9.1's object shape exactly. Ids are the same stable ids, which is what makes "the same settlement, renamed" distinguishable from "a different settlement". |
| `years[].ways` | array | MUST | The roads as they were, using §9.3's `roads[]` shape. |

A snapshot is a **frozen copy**, not a reference: editing a settlement today
must not rewrite history.

Writers SHOULD bound the number of recorded years. This port caps it at 2000;
the cap is a writer policy, not part of the format, and readers MUST accept any
number.

### 10.2 `history/territory/<year>.i32`

One entry per recorded year that has a territory snapshot, where `<year>` is
the decimal `year` value of the corresponding entry in `history/timeline.json`
(negative years are written with a leading `-`). Content and validation are
exactly §8's: `GW × GH` little-endian signed 32-bit integers, row-major, faction
id per cell, `0` = unowned.

**Why this is not inside `timeline.json`.** A territory raster for a 512×512
world is 262 144 values. As a JSON array that is roughly 600 kB of text per
recorded year, parsed by a JSON parser; as a binary entry it is 1 MiB that
deflates to a few kilobytes, read by a typed-array view. At 4096² the JSON form
is tens of megabytes per year and is not viable in a browser. It is a raster,
so it is stored the way rasters are stored — and it is a *past* raster, so it
is not in `rasters/`.

A year listed in `timeline.json` with no corresponding territory entry is
valid; it means that snapshot recorded no territory. A territory entry whose
year is not listed in `timeline.json` MUST be ignored.

---

## 11. `annotations/` — marks on the sheet

### 11.1 `annotations/labels.json`

```json
{
  "labels": [
    { "x": 104.5, "y": 57.25, "name": "The Broken Coast",
      "angle": 0.0, "arc": 0.0, "size": 16.0,
      "font": null, "color": null, "size_mode": "zoom" }
  ]
}
```

`x`/`y` are fractional grid coordinates. `angle` is the baseline rotation in
radians. `arc` bends the baseline; `0` is straight. `size` is the type size.
`font` and `color` are `null` for "use the renderer's default" — a reader MUST
NOT substitute a concrete default on load, because doing so would freeze
today's default into the file. `color`, when present, is a CSS colour string.
`size_mode` is `zoom` (constant on-screen size, the default) or `fixed` (grows
with the terrain); an unrecognised value reads as `zoom`.

Array order is significant — it is draw order.

### 11.2 `annotations/icons.json`

```json
{
  "icons": [
    { "x": 40.0, "y": 12.0, "family": "feature", "slot": "mountain",
      "set": null, "scale": 1.0 }
  ]
}
```

`family` is one of `settlement`, `feature`, `poi`, `custom`. `slot` names the
symbol within that family. `set` is the custom set name and is non-`null` only
when `family` is `custom`. `scale` is a per-instance size multiplier, `1.0`
for a plain click placement. A reader MUST keep an icon whose `slot` it cannot
resolve, and render a placeholder — dropping it silently loses the author's
work over an art-pack mismatch.

Array order is draw order.

### 11.3 `annotations/regions.json`

```json
{ "region": { "x": 10, "y": 20, "w": 128, "h": 96 } }
```

The current region-of-interest marquee, in whole cells, or `region: null` for
none. A reader MUST clamp it to the grid rather than reject it.

---

## 12. `library/` — reserved

`library/assets.json` and `library/travel.json` are reserved for the asset
library and travel library. They are not written today; see §16.5.

---

## 13. Root-level single documents

### 13.1 `params.json` — generation parameters

```json
{
  "cartalith": { "tect.seed": 24601, "climate.lat_n": 55.0, "use_gpu": false },
  "reference": { "tect": { "seed": 24601, "plates": 9 }, "seaLevel": 0.42 }
}
```

Two views of the same settings, both optional, neither authoritative over the
other for a reader that understands only one of them:

- **`cartalith`** — a flat map of dotted parameter keys to values. This is the
  vocabulary the native port's own generator reads back. It is flat and dotted
  precisely so that adding a parameter is adding a key, never restructuring a
  document.
- **`reference`** — the same settings under the HTML app's own nested names,
  for an implementation that speaks that vocabulary. It carries settings the
  `cartalith` view has no equivalent for and vice versa; neither is a superset.

A reader MUST read whichever view it understands and MUST preserve the other
verbatim on write-back (§6.2). A reader that understands neither MUST still
open the archive: `project.json` alone carries everything needed to *display* a
world, and `params.json` is only needed to *regenerate* one.

**`params.json` does not repeat the grid.** `GW`, `GH`, `sea_level`,
`map_width_km`, `wrap_x` and `seed` live in `project.json` and only there. The
flat legacy layout stored them in both places; that is the kind of duplication
this tree exists to remove.

### 13.2 `appearance.json` — presentation

```json
{
  "quality": "quality",
  "look": "vibrant",
  "territory_opacity": 0.32,
  "overrides": { "sun_azimuth": 315.0 },
  "ramp": null,
  "npr": {}
}
```

Everything here affects only how the world is drawn. A reader MUST be able to
ignore this file entirely and still render the world with its own defaults; a
reader that applies it MUST NOT let anything in it change a generated value.
All members optional. `overrides` is a flat map of appearance keys to numbers,
layered over whatever base the reader uses. Unknown keys are ignored (§14.3).

### 13.3 `vault.json` — links out to a Markdown vault

The link store, verbatim: a record of which external Markdown notes are
attached to which entities in this project. It is at the root rather than under
`annotations/` because it points *outward*, at files the archive does not
contain, which is a different kind of thing from a mark drawn on the map.

Every link names both the entity's id and the entity's **name at the time the
link was made**. A reader MUST use the id first and fall back to the name, and
MUST report rather than silently drop a link that resolves to neither — an
unresolvable link is the user's filing, and losing it quietly is the failure
mode §14.2 already cost this project once.

Ids inside this document are subject to §14.1.

### 13.4 `README.md` — for a human with a zip tool

Optional, plain UTF-8 Markdown. A short description of what the archive is and
what its directories hold. No program reads it; no program may depend on it.

### 13.5 `preview.png` — thumbnail

Optional PNG, of a size suitable for a file browser or a project picker
(this port writes at most 512 px on the long edge). **It is a thumbnail, not
map data**, and is named `preview.png` rather than the legacy `map.png` so that
no reader mistakes it for the latter. A reader MUST NOT derive anything from
it. A writer MAY omit it; an archive is not less valid without one.

---

## 14. JSON conventions

Every `.json` entry in the archive is **UTF-8, without a byte-order mark**. A
reader MUST tolerate a BOM by skipping it. Writers SHOULD pretty-print with
two-space indentation; readers MUST NOT depend on whitespace.

### 14.1 Numbers, and the 2^53 rule

**JSON has one number type, and in JavaScript it is a double.** Integers with
magnitude above 2^53 − 1 (`Number.MAX_SAFE_INTEGER`, 9 007 199 254 740 991)
cannot be represented exactly, and a JavaScript implementation of this format
would corrupt them silently.

Therefore:

- **Every integer in this format — every id, index, count, year, population and
  seed — MUST lie in the range −(2^53 − 1) … 2^53 − 1.**
- A writer MUST NOT emit an integer outside that range. A writer whose internal
  counter could exceed it MUST fail the save rather than write a value it
  cannot promise to read back.
- A reader MUST treat an out-of-range integer as a damaged value (§6.4).

This is a deliberate constraint on the format rather than a warning to
implementers, because a warning does not survive contact with a second
implementation. There are no string-encoded integers anywhere in this format
and none should be added: keeping every id inside the safe range is cheaper
than a representation that half the implementations will forget to decode.

Non-integer numbers are ordinary doubles. `NaN` and infinities are not valid
JSON and MUST NOT be written; a writer that holds one MUST write `null` and a
reader MUST treat `null` in a numeric position as absent.

### 14.2 Integers that arrive as floats — the KV-04 rule

**A reader MUST accept an integer-valued number in any JSON numeric form.**
`1`, `1.0` and `1e0` are the same value and all three MUST be read as the
integer `1` wherever an integer is specified. A reader MUST reject only a
number with a genuine fractional part where an integer is required.

This is not pedantry. It is the exact bug that cost this project a shipped
subsystem, registered as **KV-04** in `GUI_GAP_REGISTER.md` §49 and fixed on
2026-08-25. The Markdown Vault's link store was written to disk by a component
that re-parsed correct JSON through a layer with only one number type; two
integer fields came back as `1.0` and `1787605785.0`; the strict parser on the
other side refused the document; and **every link every user had ever made was
silently discarded on each launch**, for the whole shipped lifetime of the
feature. Bisection showed the round trip failed on two fields, not one, and
that the producing side had been correct all along.

Any language whose JSON layer types numbers as floats — JavaScript and GDScript
both do — will re-emit integers this way. Strictness here buys nothing and
costs data.

The reciprocal obligation: **a writer SHOULD emit integers without a decimal
point** where its language permits, so that a strict reader on the other side
is never tested.

### 14.3 Unknown members survive

§6.3's unknown-entry rule applies inside documents too, at every level of
nesting:

- A reader MUST ignore object members it does not recognise, and MUST NOT fail
  a document because of one.
- A reader that writes the archive back MUST preserve unrecognised members
  (§6.2). Round-tripping a project through an older implementation must not
  strip what a newer one wrote.
- An array element that is malformed MUST be skipped, not made to fail its
  whole array. A single bad label costs one label.

### 14.4 Strings

Vocabulary members (`kind`, `class`, `mode`, `family`, `size_mode`, culture and
government keys) are lowercase ASCII identifiers with `_` as the separator.
Each section states the closed set and what an unrecognised value reads as.
A reader MUST NOT crash on an unrecognised vocabulary value.

Free text (`name`, `history`, label text) is arbitrary UTF-8 and may be empty.
Empty is a real value and MUST NOT be replaced by a placeholder on load.

---

## 15. The legacy flat layout — read-only

Every `Cartalith Gen1` export up to and including v2.10 uses this layout.
Readers MUST accept it (§1). **No writer produces it** except the explicitly
labelled interoperability export of §1.1.

Seven entries at the archive root, no directories:

| Entry | Contents |
|---|---|
| `params.json` | `{ "v": <number>, "GW": <int>, "GH": <int>, "state": { … } }` |
| `heightmap.f32` | `GW × GH` little-endian f32, row-major |
| `temperature.f32` | as above, degrees Celsius |
| `rainfall.f32` | as above |
| `volcanic_field.f32` | as above |
| `impact_field.f32` | as above |
| `strahler_order.bin` | `GW × GH` unsigned bytes. Note the extension is `.bin`, not `.u8`. |

A real export carries more than these — a baked atlas, `map.png`, a README,
biome and lithology rasters, resource potentials, Köppen rasters, wildlife
regions and an asset-library payload. §6.3's unknown-entry rule is what makes
that normal rather than corruption, and it has been in force since the first
reader existed.

**Mapping the flat layout onto the tree.** A reader that normalises everything
to the tree's model reads:

| Tree | Flat source |
|---|---|
| `world.grid_width` / `grid_height` | `params.json` → `GW` / `GH` |
| `world.seed` | `params.json` → `state.tect.seed` |
| `world.map_width_km` | `params.json` → `state.mapWidthKm` |
| `world.sea_level` | `params.json` → `state.seaLevel` |
| `world.wrap_x` | `params.json` → `state.world`, defaulting to `false` |
| `params.json` → `reference` | the whole `state` object |
| `params.json` → `cartalith` | `state.cartalith` if present, else absent |
| `rasters/*.f32` | the same-named root entries |
| `rasters/strahler_order.u8` | `strahler_order.bin` |

Everything else in the tree has no flat equivalent: a flat archive carries no
entities, no history and no annotations this reader can restore.

`v` in the flat `params.json` is **provenance, not a format selector**. The
HTML app's own loader never branches on it — every compatibility shim it has
tests for a missing *key* instead. A reader MUST NOT branch on it either, and
MUST use §4's test.

### 15.1 The flat layout's shallow-merge hazard

Relevant to anyone writing the interoperability export of §1.1, and to nobody
else. The HTML app's loader merges the saved `state` into its own live state
**shallowly**. Any nested block the file contains therefore *replaces* the
app's whole default block rather than merging into it. A file that writes
`tect: { seed: N }` alone leaves the app with an undefined plate count, drift,
warp and blur radius.

An interoperability writer must therefore write each nested block **complete or
absent**. The blocks the app re-shims over a default literal (`climate`,
`stream`, `velo`, `glacial`, `coastal`, `planet`, `planet.tides`,
`world_structure`, `viz`) are safe to write partially; `tect`, `volc`, `crater`
and `erosion` are not.

The tree layout has no such hazard: `params.json`'s two views (§13.1) are each
read whole, and neither is merged into a live object.

---

## 16. Deliberately not stored

Each of these is a decision with a reason, recorded so that a second
implementation does not "fix" it by adding a folder.

### 16.1 The baked atlas and the tile pyramid — not in the archive at all

This is the owner's own example of the problem ("not atlas and cartography and
both storing map tiles"), and it is resolved by **deleting the concept from the
archive** rather than by choosing between two homes for it.

Three reasons, all sufficient alone:

1. **It is derived.** Every tile is a rendering of `rasters/`. Nothing in a
   pyramid cannot be rebuilt from what the archive already holds.
2. **It is enormous.** A full pyramid over a large world is orders of magnitude
   larger than the rasters it was derived from — the archive would be dominated
   by data that is not the project.
3. **It goes stale invisibly.** An archive holding both an edited heightmap and
   a pyramid baked before the edit is internally inconsistent, and a reader
   cannot cheaply tell which is older.

A pyramid is a **cache**, and belongs where caches belong: outside the project
file, keyed so that reopening the same world finds it. `project.json` carries
`world.seed` and the grid dimensions, which is everything a cache key needs.

### 16.2 Settlement placement inputs

The suitability rasters, coast distance fields and travel-cost surfaces a
settlement was placed from are not stored. They are inputs consumed during
generation, they are large, and `entities/settlements.json` already holds the
result. Storing both would be the same overlap in another costume.

The per-settlement "why here?" explanation is likewise not stored: it is a
diagnostic over rasters that no longer exist, and reconstructing one from
stored data would be inventing it rather than recalling it.

### 16.3 `drafts/` — reserved, empty today

Uncommitted paint and sculpt edits. Reserved rather than written, because in
the current implementation those editors can only be constructed over a live
generated world — they hold buffers keyed to substrate the archive does not
carry. Persisting a draft that cannot be reconstituted would produce a file
that opens with a draft nothing can commit. The slot is named so that neither
implementation invents a different location when this becomes possible.

### 16.4 The reserved rasters

`biome`, `lithology`, `koppen` and `wildlife` are computed by the generator and
discarded after use in the current implementation; there is no live value to
write. They are named in §8.1 so the second implementation does not choose
`rasters/biomes.u8` or `climate/koppen.u8` instead.

### 16.5 `library/` — reserved, empty today

The asset library and travel library are real, live data in this port, and they
survive a regeneration — which is exactly why they are *setting*-level and not
world-level. They are reserved rather than written in version 1 because both
carry binary payloads (art) whose embedding is its own design question, and
answering it badly would put images in a place the tree would later have to
move them out of.

### 16.6 Ephemeral tool state

An in-progress measurement chain, a half-drawn polyline, the currently armed
brush. These are the state of a gesture, not of a project. Nothing that would
be discarded by clicking elsewhere is stored.

---

## 17. Notes for this repository's implementation

**Non-normative.** Nothing in this section constrains a second implementation.

- The reader and writer are `cartalith-io`'s `project` module
  (`crates/cartalith-io/src/project.rs`); the flat layout's reader remains in
  the same crate's `lib.rs` and its interoperability writer in `save.rs`.
- `cartalith-io` deliberately holds **no schema for §9-§13's documents**. It
  owns the container, the tree's slot registry, the raster encoding and §14's
  number handling; each document's shape is owned by the crate that owns the
  data. A document reaches the archive as JSON text against a registered slot
  name, and an unregistered slot is a write error — that registry is what keeps
  §5's "one concept, one home" a property of the code rather than of good
  intentions.
- §14.2's rule is implemented once, centrally, as a pass over every parsed
  document that rewrites integral-valued floats to integers before any schema
  sees them. Doing it per-field would mean remembering it per-field, and KV-04
  is what forgetting once costs.
- The Godot-facing surface is `crates/cartalith-godot/src/project_bridge.rs`.
  GDScript-owned payloads reach the archive through a document channel rather
  than through a schema in Rust, so a payload the shell owns needs no engine
  change to be persisted.
- `strahler_order` is 8-bit in the archive and wider in memory; it saturates at
  255 on the way out, matching the reference exporter's own `o > 255 ? 255 : o`.
- **This build decodes far more compression methods than the format uses.**
  The `zip` crate is taken with its default features, which bring in
  Zstandard (93), bzip2 (12), LZMA (14), XZ (95), PPMd (98) and Deflate64 (9)
  decoders. That is incidental, not a promise: §3.3 is what a second
  implementation must satisfy, and this port still writes only method 8 (and
  method 0 for `preview.png`, which is already-compressed PNG). The methods
  it genuinely cannot decode are the legacy PKZIP ones (1-6), which is why
  §3.3's own round-trip test uses method 1.
- Round-trip coverage lives in `crates/cartalith-io/src/project.rs`'s own test
  module, `crates/cartalith-godot/src/project_bridge.rs`'s, and
  `crates/cartalith-godot/tests/project_round_trip.rs`.

**One conformance gap, disclosed rather than hidden.** §6.2 asks a reader that
writes an archive back to either re-emit entries it did not understand or
refuse to overwrite. This implementation does **neither** yet: it reports them.
`read_project` returns `foreign_entries`, the names of every entry it did not
consume, and `project_open` hands that list to the shell so a Save command can
warn before it drops them. That is weaker than §6.2 requires and is the honest
state today; retaining the bytes is real work through every layer between the
reader and the save button, and no implementation writes a foreign entry yet.
When the HTML app starts writing payloads this port does not model, closing
this is the first thing that has to happen.

---

## 18. Why the container is deflate — measured, 2026-08-25

**Non-normative.** This section is the evidence behind §3.3, and it exists
because the question it answers ("a dedicated application does not need the
browser's codec — so what should it use?") has a counter-intuitive answer that
is expensive to re-derive and easy to get wrong from first principles.

### 18.1 What was measured

Three real generated worlds (seed 24601, no civilisation layer), written as
the entries §5 and §8 define, each archive round-tripped and compared byte for
byte before its size was recorded. "shuffle" is an HDF5/Blosc-style byte-plane
transform applied to the 4-byte rasters before compression — all the byte-0s,
then all the byte-1s, and so on — which is lossless and exactly reversible.

Whole-archive size:

| Variant | 512² | 2048×1311 | 4096² | write @4096² | read @4096² |
|---|---|---|---|---|---|
| stored (raw payload) | 5.3 MiB | 53.8 MiB | 336.0 MiB | 0.09 s | 0.03 s |
| **deflate — what is written today** | **2.3 MiB** | **24.9 MiB** | **152.7 MiB** | **3.04 s** | **0.43 s** |
| deflate, level 9 | 2.3 MiB | 24.5 MiB | 148.7 MiB | 4.17 s | 0.54 s |
| zstd, level 3 | 2.3 MiB | 24.8 MiB | 151.8 MiB | 0.44 s | 0.18 s |
| zstd, level 9 | 2.3 MiB | 24.6 MiB | 148.4 MiB | 1.59 s | 0.21 s |
| shuffle + deflate | 1.7 MiB | 16.7 MiB | 97.2 MiB | 1.92 s | 0.34 s |
| shuffle + zstd, level 3 | 1.7 MiB | 16.5 MiB | 95.4 MiB | 0.52 s | 0.24 s |
| shuffle + zstd, level 9 | 1.7 MiB | 15.6 MiB | 85.2 MiB | 1.57 s | 0.25 s |
| shuffle + zstd, level 19 | 1.7 MiB | 14.9 MiB | 79.4 MiB | 19.82 s | 0.29 s |

Per entry at 4096², against today's deflate, the whole difference is in three
files: `heightmap.f32` 53.6 → 33.9 MiB, `temperature.f32` 54.5 → 34.6 MiB and
`rainfall.f32` 42.2 → 26.7 MiB under the shuffle. `volcanic_field` and
`impact_field` are already near-empty (1.6 MiB and 0.7 MiB), `strahler_order`
is 28 KiB, and every JSON document in the archive together is under 5 KiB.
**Any change that does not move the three float grids is optimising nothing.**

### 18.2 Three findings

1. **Changing the codec changes almost nothing.** Deflate → Zstandard moves
   the archive by under 3% at every size and every level short of 19. Both are
   LZ77 plus entropy coding, and the low mantissa bytes of an IEEE-754 grid
   are close to random to either of them. Deflate at level 9 and Zopfli (a
   much slower deflate encoder: 1.2% better than shuffle+deflate at 4096², for
   54× the write time) confirm the same ceiling from the other direction.
2. **Rearranging the bytes before compressing does change it**, by 27% at
   512², 33% at 2048×1311 and 36% at 4096² — with deflate, unchanged. It also
   makes the *write faster* (3.04 s → 1.92 s at 4096²), because there are far
   fewer literals left to encode.
3. **Zstandard's real gain is time, not size**: 3.04 s → 0.44 s to write and
   0.43 s → 0.18 s to read, at 4096². If saving a large world ever feels slow,
   that is the lever — and it is a lever with a compatibility price, below.

### 18.3 Why per-entry method mixing does not rescue Zstandard

The tempting reconciliation is to keep `project.json` and `params.json` on
deflate — so that any reader, browser included, can at least open the manifest
and say what the file is — and use method 93 only for the big rasters a
browser was never going to want. **It does not work.** Both known JavaScript
readers reject the *whole archive*, eagerly, at open time, on the first entry
whose method they do not know:

- The reference app's own `unzipAny` walks the central directory and does
  `else throw new Error('unsupported zip method '+method+' for '+name)` inside
  that loop, before any entry's data is returned to the caller.
- JSZip throws `Corrupted zip : compression <n> unknown (inner file : <name>)`
  from `readLocalPart`, which runs for every entry during `loadAsync` —
  not lazily when the entry's content is requested.

So a single method-93 entry costs the browser the manifest too, and the
graceful-degradation story the mixing idea depends on never happens. Combined
with finding 1 — under 3% for the whole exercise — method 93 is refused
outright by §3.3 rather than made conditional.

### 18.4 Two levers that are owner decisions, not this document's

**The byte-plane shuffle.** It is the only measured change worth having, it
keeps method 8 so every zip reader still opens the container, and it is about
ten lines in either language. What it costs is §8's promise that a raster
entry is a bare little-endian dump a JavaScript reader can put a typed-array
view straight onto. That promise would have to be replaced by a
`format_version` bump and an explicit marker — and the hazard is that a reader
which ignored the marker would not fail, it would read *plausible-looking
noise*, which is the one failure mode this format is built to avoid. Making it
fail loudly instead would mean distinct entry names, and §8's rule that the
extension names the element type has no room for a second axis. Not adopted
here; it needs an owner decision, and it is worth putting to one.

**Quantisation.** Storing the heightmap as `u16` with a scale and an offset,
GeoTIFF-style, halves the payload before compression and compresses far better
than f32 afterwards — the largest available win by some distance. It is
**lossy**. `PARITY_TESTING.md` and `DECISIONS.md` §7a make bit-exact raster
values a property this port tests against the reference engine, and a save
that returns a different float than it was given would break that on the load
path as well as the save path. Costed and deliberately not built; moving that
bar is an owner decision.
