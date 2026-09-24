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
- **§6** — conformance: what MUST be written, what MAY be, minimal writer and
  reader. **§6.4a is the damage ladder** — what a missing, mistyped or
  out-of-range value costs, and it is what §7-§13 mean whenever they call
  something damaged.
- **§7-§13** — entry-by-entry specification.
- **§14** — JSON conventions. **Non-optional reading**; §14.2 describes a bug
  that has already cost this project one shipped subsystem.
- **§15** — the legacy flat layout, read-only.
- **§16** — what is deliberately not stored, and why.
- **§17** — notes specific to this port's own implementation (non-normative).
- **§18** — why the container is deflate, measured. Non-normative, but it is
  the reason §3.3 says what it says. Its two levers were open owner decisions
  until 2026-09-23 (Ruling AJ): the shuffle was taken (§8.2, measured on
  shipping in §18.6) and quantisation was declined.

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

### 1.2 What the first independent implementation found, 2026-08-26

The claim at the top of this document — implementable from this text alone —
was tested rather than asserted: a second implementation of the *reader* was
built in the HTML app from this specification, without access to the port's
code. It mostly worked. Every place it could not is a defect in this document,
and each one is fixed in place rather than appended:

| Where | What was wrong |
|---|---|
| §6.4a, §7 | §7's blanket "refuse if any MUST member is missing" and §6.4's much narrower list contradicted each other, with no rule for the gap between them. Now one damage ladder and a per-member table. |
| §8.1, §15.2 | The flat layout's 16-bit height fallback was not documented anywhere. |
| §8.1 | `territory.i32` is wider than the flat layout's own 8-bit store, with no stated obligation either way. |
| §9.1 | `capital` and `suitability` were specified as though every implementation had them. |
| §9.2 | `color` was described as derived from the faction index. It is not, and a reader that regenerated it recoloured the world. |
| §9.4 | Provinces may legitimately be re-derived rather than stored; the spec read as though storing them were required. |
| §10.2 | `<year>` had no grammar, so `+7`, `007` and `7` were all arguably the same entry. |
| §11.3 | A zero-sized region marquee had no defined meaning; two implementations chose differently. |
| §13.3 | **The worst of them.** "The link store, verbatim" was the whole specification of the format's outward-facing half — the half the owner most wants in the HTML app — and the implementer had to read this port's Rust to learn its shape. Now specified field by field. |
| §14.4 | `sea_lane` collides with the flat layout's own `sea-lane` by punctuation alone, in the one application this format was written to be implemented in. |
| §15.1 | This document asserted that a flat archive carries no entities, no history and no annotations. It carries all three, nested inside `params.json`. A reader that believed this sentence silently discarded every settlement, faction, road and label in the file. |

The general lesson, recorded because it will apply again: **the sections that
were thinnest were the ones furthest from this port's own code**. §8 and §14
are exact because they were written while implementing them; §13.3 was a
pointer at a Rust type, and §15's closing sentence was a guess nobody had
checked against the reference. A specification's gaps are where its author
already knew the answer.

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

### 3.0 The file extension — a convention, deliberately not a requirement

**Added 2026-09-07.** This section constrains the container, the entry names,
the compression methods and zip64. **It has never constrained the archive’s own
filename, and it still does not** — the only extensions specified anywhere in
this document are ENTRY extensions (`.f32`, `.u8`, `.i32`, `.bin`, §8).

**The convention this port writes is `.ctl`** (owner decision, 2026-09-07),
and the reason is a real defect rather than branding: a picker filtering on
`.zip` offers every archive on the device, so a user was shown a 2024
`Werk.zip` and then told *"missing zip entry: params.json"* after choosing it.
A distinct extension declines it before the reader has to.

**A conforming reader MUST NOT require it.** The extension carries no meaning
the container does not already carry, every archive written before that date
is `.zip`, and the interoperability writer of §1.1 knows nothing about it.
**Read by content, filter by extension only where a human is choosing a file.**
This port keeps `["ctl", "zip"]` in `OpenProjectDialog.PROJECT_EXTENSIONS` for
exactly that reason.

**No `format_version` bump.** Nothing about the bytes changed, so an archive
written today is byte-identical to one written yesterday under the other name,
and renaming a file moves it between the two conventions with no conversion.


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
{ "format": "cartalith-project", "format_version": 2 }
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
    rules. Versions 1 and 2 are defined; nothing is older than 1.
- A writer MUST write both members.

**Version 2 is the version this document defines, and §8.2 is the whole of
what separates it from version 1.** Version 1 said `1` from 2026-08-25 to
2026-09-23; version 2 (owner Ruling AJ, 2026-09-23) adds the byte-plane
shuffle, which stores a 4-byte raster's bytes reordered under a **different
entry name**. Every other rule in this document is the same for both, and a
version-2 reader reads a version-1 archive exactly as it always did — §8.2 says
why that is guaranteed by the entry names rather than hoped for from this
member.

**A version-1 reader does not reach the "greater" rule above with a shuffled
archive, and that is deliberate.** It looks for `rasters/heightmap.f32`, does
not find it, and refuses the archive under §6.4. That refusal is the fail-loud
marker: the alternative — a reader that warned "newer version" and then put a
typed-array view over shuffled bytes — would draw plausible-looking noise,
which is the one failure this format is built to avoid (§18.4).

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
                                      (a 4-byte raster may instead be stored
                                      as <name>.shuffled.f32 / .shuffled.i32
                                      — §8.2; the slot is the same slot)
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
  journeys.json               MAY
  landmarks.json              MAY
  conflicts.json              MAY

history/                              recorded past states — see §10
  timeline.json               MAY
  territory/<year>.i32        MAY

annotations/                          marks on the sheet — see §11
  labels.json                 MAY
  icons.json                  MAY
  regions.json                MAY
  measurements.json           MAY

library/                              setting-level definitions — see §12
  assets.json                 MAY
  travel.json                 MAY
  settlement_types.json       MAY

drafts/                               uncommitted edits — see §12
  paint.json                  MAY
  sculpt.json                 MAY

cartography/                          derived pixels, cached — see §16.1
  tiles/index.json            MAY     what the tiles were made from
  tiles/<z>/<col>/<row>.u8    MAY     one deep-zoom tile, one byte per pixel
```

**Five of these rows read `reserved` until 2026-09-03 and were wrong.**
`entities/journeys.json`, `library/assets.json`, `library/travel.json`,
`drafts/paint.json` and `drafts/sculpt.json` have all been written since
2026-08-31 and all restore. `entities/landmarks.json` was missing from this
table entirely while being a registered slot with a live writer. A table that
calls a written slot reserved is not a harmless lag: `reserved` invites the
second implementation to claim the name for something else, which is the exact
collision §5 exists to prevent.

**`library/settlement_types.json` was missing from this table until
2026-09-24** while being a registered slot (`cartalith-io`'s `DOCUMENT_SLOTS`)
with a live writer: the shell's `SettlementTypeStore`, written and restored by
`app.gd` since 2026-09-21. It is caller-owned in this port; §12 carries its
shape.

**`cartography/` is `MAY`, and since `8cade54` (2026-09-22) a shipping code
path writes it — but nothing reads it back.** This paragraph said no shipping
path reached it until 2026-09-24. The *Save project as…* dialog's LOD-tiles
checkbox sets `app.gd`'s `_include_lod_tiles_pref`, and all three save paths —
Save as, plain Save, and the save offered before a destructive action — pass
it to `project_save_with_documents`'s `include_lod_tiles`, which fills
`write_project`'s `lod_tiles`. Off by default (ruling 28), so an archive
carries `cartography/` entries only when the user ticked it. `cartalith-io`'s
reader parses the pyramid back into `ProjectData::lod_tiles`, but
`project_open` does not consume it — only tests do — so today the stored
pyramid is written and never used on load. It is listed here rather than as
`reserved` because the name is claimed, the shape is normative, and a second
implementation writing it would be conforming.

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

**`cartography/` — "is it a picture the archive can throw away and redraw?"**

**This paragraph read "There is no `atlas/`, no `cartography/`, and no
`tiles/`" until owner ruling 28 (2026-09-06), and that is no longer true.**
The owner's original example of the failure — *"not atlas and cartography and
both storing map tiles"* — was resolved by deleting the concept from the
archive; the ruling put one shape of it back, optionally. What survives
unchanged is the half the owner's instruction was actually about: **there is
still exactly one home.** There is no `atlas/`, and `cartography/` holds
derived pixels and nothing else — no geometry, no settings, nothing a user
typed. A payload belongs here only if deleting the whole directory costs the
project nothing but time.

That test is what keeps this from becoming the second tile folder. `rasters/`
holds one value per grid cell and is the world; `cartography/` holds a
rendering of `rasters/` at a zoom level and is a cache. See §16.1 for the
optionality, the size, and the staleness rule.

---

## 6. Conformance

### 6.1 A minimal conforming writer

Write a zip containing exactly two entries:

1. `project.json` with `format`, `format_version` and a `world` object giving
   `grid_width`, `grid_height`, `wrap_x`, `map_width_km`, `sea_level` and
   `seed` (§7).
2. `rasters/heightmap.f32` — `grid_width × grid_height` little-endian 32-bit
   floats, row-major (§8).

That archive is valid, at `format_version` `1` or `2` — the shuffle of §8.2 is
what a writer SHOULD do, not what it must, so a minimal writer can still skip
it. Every other entry is optional enrichment. A writer that
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
- If `rasters/heightmap.f32` is missing — **under both of its names**, §8.2 —
  or its length is not `grid_width × grid_height × 4`, the reader MUST refuse
  the archive.
- For **every other** entry: a reader MUST NOT fail the archive. It MUST skip
  the damaged entry, continue, and report what it skipped. A corrupt
  `annotations/labels.json` must not cost the user their world.

Only the four bullets above can refuse an archive. Nothing in §7-§13 adds a
fifth: those sections say what a *value* means, and a value that is missing,
mistyped or out of range is resolved by §6.4a rather than by discarding the
project.

### 6.4a The damage ladder — decided 2026-08-26

**The problem this decides.** The first independent implementation of this
format read §7's closing sentence ("a reader MUST refuse the archive if any
MUST member is missing or is not of the stated type") as the general rule and
§6.4 as a special case of it. Read that way, a writer that forgets one boolean
in `project.json` produces an archive no conforming reader will open, and a
single out-of-range integer somewhere in `entities/settlements.json` has no
stated cost at all. Both readings were available in the text, which means the
text was wrong rather than the reader.

**The decision: damage is contained at the smallest enclosing scope that still
has a defined meaning, and it climbs only when there is none.** Four rungs,
innermost first:

1. **A value.** A member that is absent, of the wrong type, or outside
   §14.1's range is *damaged*. If this section or §7-§13 states a
   substitution for it, the reader MUST apply that substitution and report it.
   §9.1's "clamp an out-of-range faction to `0`" and §9.1's "an unrecognised
   `kind` MUST be read as `town`" are two of these.
2. **An array element.** A damaged value with no stated substitution costs its
   *element*, and nothing more: the reader MUST skip that element, keep the
   rest of the array, and report the skip (§14.3). One settlement with a
   corrupt `id` costs one settlement.
3. **A document.** A damaged value with no stated substitution at the *top
   level* of a document — not inside one of its arrays — costs that document:
   the reader MUST skip the whole entry as §6.4's fourth bullet describes, and
   report it. A `history/timeline.json` whose `years` member is a string
   carries no recoverable snapshots.
4. **The archive.** Only §6.4's own four bullets reach this rung.

**Why containment rather than strictness.** A save file is the user's work.
Refusing to open one is the most expensive thing a reader can do, and it is
only justified where continuing would mean *inventing* the world rather than
reporting a gap — which is exactly what `project.json` and the heightmap are,
and exactly what a label, a road or a faction colour is not. The reciprocal
obligation is that every substitution above is **reported**, never silent: a
reader that repairs quietly trains the user to trust a file that has already
lost something.

`project.json` is governed by §7's own table, which now states the outcome for
each member individually rather than by a blanket rule.

### 6.5 Documents an implementation does not model

§5's slot list is longer than any one implementation's set of internal types,
and permanently so: `library/` and `drafts/` hold payloads whose owner is the
*application* rather than the map engine (§12), and `entities/journeys.json`
was specified (§9.6) before anything could write it. An implementation
therefore partitions §9-§13's documents in two, and the partition is a property
of that implementation rather than of the format:

- **Modelled** documents are ones it parses into its own types, keeps as those
  types, and writes back out of them. `entities/settlements.json` is modelled
  by anything that draws settlements.
- **Carried** documents are ones it stores and returns without understanding.

Both halves are ordinary documents in the file. Nothing in §5 or §9-§13 marks
which is which, and a second implementation MUST NOT infer one from the other's
choice: a slot one implementation models is a slot another carries.

**A carried document MUST be exposed as JSON text, and MUST NOT be exposed as
the implementation's decoded value.** This is the only rule in this section and
it is not a matter of taste. JSON has one number type (§14.1) and in JavaScript
and GDScript it is a double, so decoding a document and re-encoding it rewrites
every integer in it — and every integer in a document the implementation does
not model is one it cannot repair, because it does not know which members were
integers. Carrying the text carries the whole document, including the parts the
format has not specified yet.

The text a reader returns MUST be the document as stored, byte for byte, except
that a leading byte-order mark MUST be removed (§14). In particular the reader
MUST NOT re-order object members, MUST NOT change whitespace, and MUST NOT
apply §14.2's coercion to it — §14.2 governs values a reader *interprets*, and
a carried document is by definition one it does not.

Symmetrically, a writer accepting a carried document MUST accept it as text and
MUST write those bytes unchanged. A writer that cannot write a document
unchanged — because it is not valid JSON, or carries a byte-order mark — MUST
refuse it rather than repair it: an edit the caller did not ask for breaks the
same promise as a decode.

Two obligations follow, and are the reason this section is under Conformance:

- An implementation SHOULD refuse to *return* a modelled document through the
  carried-document channel. Returning one invites the host application to edit
  and re-supply it, after which the archive has two sources of truth for the
  same concept and no rule about which wins. The engine's own accessors are the
  single source; the channel is for everything else.
- A reader that returns carried documents SHOULD also report **which** carried
  documents the archive contained, so that a host application enumerates them
  rather than guessing slot names. The set of slots returned is itself an
  adequate report; a separate list is not required and is a second copy of the
  same fact.

This section is what makes §6.2's "without data loss" reachable for documents.
It does not reach *entries*: an entry the reader does not recognise at all is
still governed by §6.3, and retaining those bytes is a separate obligation.

---

## 7. `project.json`

Required. UTF-8 JSON. The manifest, the version marker, and the identity of
the grid every raster is measured against.

```json
{
  "format": "cartalith-project",
  "format_version": 2,
  "generator": "cartalith-native 0.1.0",
  "created": "2026-08-25T14:03:11Z",
  "world": {
    "grid_width": 512,
    "grid_height": 512,
    "wrap_x": false,
    "map_width_km": 800.0,
    "sea_level": 0.42,
    "seed": 24601,
    "origin": "gen"
  }
}
```

| Member | Type | Required | Meaning |
|---|---|---|---|
| `format` | string | MUST | Always `"cartalith-project"`. §4. |
| `format_version` | integer | MUST | `2` (`1` before 2026-09-23). §4. |
| `generator` | string | SHOULD | Free-form name and version of the writing program. Provenance only; no reader may branch on it. |
| `created` | string | MAY | RFC 3339 UTC timestamp. Provenance only. |
| `world.grid_width` | integer ≥ 1 | MUST | `GW`. Cells across. |
| `world.grid_height` | integer ≥ 1 | MUST | `GH`. Cells down. |
| `world.wrap_x` | boolean | MUST | Whether the grid wraps in longitude — a whole planet rather than a region. Corresponds to the HTML app's `state.world`. Renamed here because a member called `world` inside an object called `world` is not a name. |
| `world.map_width_km` | number > 0 | MUST | Real-world width of the map. **Height is derived, not stored**: cells are square in kilometres, so height is `map_width_km × grid_height / grid_width`. A stored height that disagreed would contradict every distance, gradient and route length in the archive. |
| `world.sea_level` | number in `[0,1]` | MUST | The **effective** threshold against the heightmap's own `[0,1]` range. A cell is land where `heightmap[i] >= sea_level`. If the generator re-anchored sea level from a world-structure archetype, this is the re-anchored value, not the user's input — the user's input belongs in `params.json`. |
| `world.seed` | integer | MUST | The generation seed. Range: §14.1. |
| `world.origin` | string | MAY | **How the height field was produced**, and the one member here that is not a generation input. `"gen"` — produced by the generator from the tuple above; `"import"` — inverted from an imported image, so the tuple does *not* determine it; `"region"` — resampled out of another world's marquee, inheriting that world's `seed`. Other values are permitted and §14.3 governs them: an unrecognised origin is carried, not folded into a known one. **Absent is not `"gen"`** — see the reader table below. |
| `world.name` | string | MAY | The world's own generated display name (`cartalith_civ::naming::world_name`, seeded from `world.seed` — deterministic, but **carries no parity contract**: this concept does not exist in the reference HTML at all, unlike every other member in this table). A writer with none MUST omit the member rather than invent one — the same discipline `world.origin` already follows. Closes `OUTSTANDING_WORK.md`'s "File ▸ Recent worlds leaves show a filename where the canvas shows the world" row (name half; the seed half closed earlier, 2026-09-13). |

**What a reader does when a MUST member is missing or mistyped.** Each MUST
above binds the *writer* unconditionally; the reader's obligation differs per
member, because "refuse" is only right where continuing would mean inventing
the world (§6.4a). `generator` and `created` are provenance and are never
fatal — a reader ignores a missing or malformed one.

| Member | Missing, mistyped, or out of §14.1's range |
|---|---|
| `format` | Refuse (§4). |
| `format_version` | Refuse. Without it the reader cannot say which rules apply. |
| `world.grid_width`, `world.grid_height` | Refuse. Zero or negative is refused too. No raster in the archive can be validated without them, so a wrong guess reads every grid at the wrong stride. |
| `world.map_width_km` | Refuse. Every distance, gradient and `length_km` in the archive is measured against it; substituting a default silently rescales the world. |
| `world.sea_level` | Refuse. It is the coastline. A default would silently redraw it. |
| `world.seed` | Refuse. It is what makes the world regenerable, and a substituted seed produces a *different* world that claims to be this one. |
| `world.wrap_x` | **Read as `false` and report it.** |
| `world.origin` | **Read as unknown.** Never fatal, and never substituted: a reader MUST NOT report a missing `origin` as `"gen"`, because the two are different facts and an archive that re-saves the substituted value has invented a provenance the file never carried. Mistyped (a non-string) is the same case as missing. |
| `world.name` | **Read as unknown (absent or empty).** Never fatal, never substituted — the same "MAY, additive, no `format_version` bump" discipline as `world.origin`. An archive written before this member existed still opens; the shell falls back to the save's filename, exactly as it did before this member was added. |

`wrap_x` is the single exception, and the reason is worth stating rather than
leaving as an oddity: it is the only member here whose absence has a defined
reading that cannot corrupt anything invisibly. A wrapping world read as
non-wrapping is wrong *at one visible seam*, which a person sees and can
correct; a substituted sea level or map width is wrong everywhere, uniformly,
and looks entirely plausible. Refusing an otherwise complete archive over one
missing boolean costs the user their project to protect them from a visible
defect — which is the wrong trade, and the trade §6.4a exists to stop making.

A writer MUST still write `wrap_x`. The leniency is the reader's, and a reader
that exercises it MUST say so.

**`world.origin` is lenient for the opposite reason**, and it is worth stating
because the temptation is to give it a default. It is MAY rather than MUST
because every archive written before it existed — including every genuine
`Cartalith Gen1` export, which will never have one — is legitimately silent
about it, and there is no way to recover the answer from the file. A writer
that knows the provenance MUST write it; **a writer that does not MUST omit
the member rather than write `"gen"`**, since a substituted value is
indistinguishable from a recorded one and the next re-save then carries the
invention forward as fact.

What a *consumer* does with the unknown case is its own business, and this
port's is recorded rather than implied: `cartalith-godot` treats an absent
origin as `"gen"` **for the atlas cache key alone** (`bake_bridge::
origin_for_key`), because an archive from before this member restores every
other element of that key exactly and a distinct fourth value would change the
key of every such project on reopen and orphan tiles its owner already baked.
It does not write that substitution back: the loaded value stays absent, so a
re-save omits the member as the rule above requires.

**No `format_version` bump for this member.** It is additive and MAY: a
version-1 reader meets it and ignores it under §14.3, and a reader that knows
it meets an archive without it and reads the absence as the answer. Bumping
would make every archive this port writes warn in a build that would have
handled it correctly, which is a cost paid for no protection. A member that
changed how an existing one is *read* would be the other case.

---

## 8. `rasters/` — the grid payloads

Every entry under `rasters/` **decodes to** a bare little-endian binary dump.
No header, no length prefix, no padding, no alignment guarantee. The entry's
uncompressed length is exactly `GW × GH × element_size` bytes.

**This paragraph said every entry *is* a bare little-endian binary dump, and
since `format_version` 2 (owner Ruling AJ, 2026-09-23) that is no longer the
whole truth — a deliberate departure, disclosed here rather than left for a
typed-array view to discover.** A 4-byte raster may now be stored with its
bytes *reordered* by the byte-plane shuffle of §8.2, under a different entry
name. The reorder is lossless and exactly reversible: every byte of the dump
below is still in the entry, and a reader that undoes it gets the identical
dump, bit for bit. What the departure costs is the one convenience the old
promise was for — a JavaScript reader can no longer put a `Float32Array`
straight onto a *shuffled* entry's bytes; it undoes the shuffle first, which is
a loop of four lines. Everything below this paragraph — byte order, index
formula, length validation — describes the decoded dump, and applies unchanged
to both encodings.

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

**`territory.i32` is deliberately wider than any implementation needs, and one
existing implementation is narrower than it.** A faction id per cell fits in a
byte today — the flat layout's own territory store is 8-bit, so an archive
converted from it can never carry an id above 255 — and the tree stores it as
`i32` anyway, because the element width is part of the entry name and cannot be
widened later without a new name and a `format_version` bump. Capping the
world's faction count at 255 forever, to save three bytes per cell in a payload
that compresses to nothing (§18.1: `strahler_order` is 28 KiB at 4096²), is not
a trade worth making.

The obligations that follow are the reader's:

- Values MUST be `0` or a positive faction id. A **negative** value is damaged;
  a reader MUST read it as `0` and report it (§6.4a rung 1).
- A reader whose own territory store is narrower than 32 bits MUST NOT truncate
  a value it cannot hold. It MUST report the raster as unreadable and continue
  without territory — a truncated id is a cell silently reassigned to a
  different faction, which is indistinguishable from the author having drawn it
  that way.

The same reasoning applies to `provinces.i32`.

There is **no `rasters/heightmap_rg16.bin`**, and no other second copy of a
raster in a different precision. The flat layout has one (§15) and it does not
survive the mapping into the tree: it is a 16-bit *quantisation* of the same
heightmap, so an archive holding both would hold two disagreeing elevations
with no rule about which is authoritative, and §18.4 records quantisation as a
lossy lever this format has deliberately not pulled. **The owner declined it
again on 2026-09-23 (Ruling AJ)** in the same ruling that adopted §8.2: the
shuffle is lossless and quantisation is not, and that difference is the whole
reason one was taken and the other was not.

### 8.2 The byte-plane shuffle — `format_version` 2, owner Ruling AJ, 2026-09-23

**What it is.** For a raster of `N = GW × GH` elements of 4 bytes each, take
the §8 dump — element `i` occupying bytes `4i … 4i+3`, little-endian — and
write instead every element's byte 0, then every element's byte 1, then byte
2, then byte 3:

```
shuffled[k × N + i] = dump[4i + k]        for k in 0..4, i in 0..N
dump[4i + k]        = shuffled[k × N + i] (the inverse, which a reader applies)
```

The entry is still exactly `N × 4` bytes, still compressed with method 8
(§3.3), and still validated against `GW × GH × 4` (§8) — the length check
happens before the inverse and is identical for both encodings. This is the
HDF5/Blosc "shuffle" filter, and it is the transform §18.1 measured.

**Why it is worth a version.** The low mantissa bytes of an IEEE-754 grid are
close to random, and interleaving them with the smooth high bytes hides the
high bytes' redundancy from deflate. Grouped by plane, the exponent and high
mantissa bytes form long runs deflate can use. §18.1 measured the whole
archive 27-36% smaller with deflate unchanged, and faster to write; §18.6 is
the measurement taken when it shipped.

**Which entries.** Every `rasters/*.f32` and `rasters/*.i32` — every
registered raster whose element is 4 bytes. `.u8` rasters have one plane and
nothing to reorder, so they have no shuffled form. `history/territory/<year>.i32`
is **not** shuffled and keeps §10.2's single spelling; neither is anything
under `cartography/`.

**The marker is the entry name, and that is what makes it fail loudly.** A
shuffled raster is stored under its slot's name with `.shuffled` inserted
before the extension:

| Slot | Shuffled entry |
|---|---|
| `rasters/heightmap.f32` | `rasters/heightmap.shuffled.f32` |
| `rasters/territory.i32` | `rasters/territory.shuffled.i32` |
| … every other 4-byte slot of §8.1 | the same rule |

The extension still names the element type (§5.1): the element is still `f32`
or `i32`, and the reorder is a property of how its bytes are laid out, which is
the thing the new name component carries. §18.4 recorded exactly this
difficulty — *"making it fail loudly instead would mean distinct entry names"*
— as the reason the shuffle was not adopted without a ruling; an infix in the
stem is the distinct name, and it leaves the extension's meaning alone.

A name rather than a member of `project.json` because the two hazards are not
symmetric:

- **An old reader meets a shuffled archive.** It looks up
  `rasters/heightmap.f32`, finds nothing, and refuses the archive (§6.4). It
  cannot read shuffled bytes as a dump, because it never finds them under a
  name it reads as a dump. A marker inside `project.json` would have protected
  only readers that checked it — and §4's own rule tells an older reader to
  *read* a newer version, not refuse it.
- **A new reader meets an old archive.** It un-shuffles an entry **if and only
  if its name says it is shuffled**. `rasters/heightmap.f32` is read as a plain
  dump in every version, so no archive written before this change — and no
  version-2 archive whose writer skipped the shuffle — can be un-shuffled by
  mistake. The decision is carried by the bytes' own name, not by a version
  number that could disagree with them.

**Obligations.**

- A version-2 writer SHOULD write every 4-byte raster shuffled. It MAY write
  one plain under its §8.1 name instead; a reader must accept either.
- A writer MUST NOT write both names for one slot.
- A reader MUST accept both names for every 4-byte slot, MUST apply the inverse
  to the shuffled one and MUST NOT apply it to the plain one.
- A reader that finds **both** names for one slot MUST read the shuffled one
  and MUST report that it ignored the other — two copies of one grid is the
  duplication §8.1 already refuses for the height fallback, and choosing
  silently would hide which one the author's file actually meant.
- An archive written at `format_version` 1 that somehow carries a
  `.shuffled.` name is outside anything any writer has produced; a reader MAY
  treat it by the rules above.

**What it deliberately does not do.** It is not a new compression method
(§3.3 is unchanged: store and deflate only, and the container still opens in
any zip tool). It is not quantisation (§8.1's last paragraph). It does not
touch any JSON document, `preview.png`, a history raster or a tile.

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
| `settlements[].capital` | boolean | MUST | Whether this is its faction's seat. **Independent of `kind`** — see below. Absent: read as `kind == "capital"`. |
| `settlements[].coastal` | boolean | MUST | Whether the settlement is a port. Computed from its final position. Absent: `false`, and a reader MAY recompute it from `x`/`y`, `heightmap` and `sea_level`. |
| `settlements[].suitability` | number `[0,1]` | SHOULD | The placement score. Display and diagnostics only; nothing downstream may branch on it. Absent: `0`. |
| `settlements[].village_seeded` | boolean | MAY | `true` if added by the optional village-seeding pass rather than by primary placement. Matters because villages are not road-network nodes: a network rebuild that fed them back in would restructure the world. Absent: `false`. |

**`capital` and `kind` are two facts, and an implementation may hold only
one.** `kind` is a *size tier*, `capital` is a *political role*, and they are
independent in principle: a faction whose seat is a modest town has
`kind: "town"` with `capital: true`, and a large world can hold several
`kind: "metropolis"` settlements of which one is the seat. An implementation
that has no separate role — that represents "is the seat" by putting `capital`
in the tier vocabulary, as the flat layout's own settlement records do — is a
legitimate reader of this format, and the round trip is defined in both
directions:

- **Reading**, with only `kind`: take `capital` as `kind == "capital"` when the
  member is absent, and prefer the stored `capital` when it is present. A
  stored `capital: false` on a `kind: "capital"` settlement is not a
  contradiction to repair; it is a larger model saying something this reader
  cannot represent, and it MUST NOT be overwritten on write-back (§14.3).
- **Writing**, with only `kind`: write both members — `kind` as stored, and
  `capital` as the boolean the tier implies. Omitting `capital` is permitted
  by the fallback above but loses the distinction for every reader that has it.

`suitability` likewise has no equivalent in every implementation. It is a
diagnostic over generation inputs the archive deliberately does not store
(§16.2), so a reader that never had one writes it absent rather than inventing
a score, and a reader that reads `0` MUST treat that as "not recorded" rather
than as "unsuitable site".
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
| `color` | array of 3 integers `[0,255]` | MUST | The faction's base palette colour, **as stored**. Not derivable — see below. |
| `user_color` | array of 3 integers, or `null` | MUST | The author's chosen identity colour, or `null` for "use the stored `color`". A separate member rather than an overwrite of `color`, so that clearing the override restores the base colour rather than losing it. |
| `tariffs` | object, exporting-faction id (as a decimal string key) → number in `(0, 1]` | MAY | Trade tariffs this faction levies **as importer** on goods from each named faction (IN-13, Ruling AE, added 2026-09-23) — the fraction of each crossing trade flow's volume the levy removes; `1` is an embargo. Directional: `A.tariffs[B]` and `B.tariffs[A]` are independent. A writer MUST omit the member when the faction levies nothing, so a file with no tariffs is byte-identical to one written before the member existed. A reader MUST treat an absent member as "no tariffs" and MUST drop, not clamp, an entry that is on faction `0` (Unclaimed levies nothing), names the faction itself, names an id with no row, or has a rate outside `(0, 1]`. |

**`color` is stored because it is not a function of the index.** An earlier
revision of this document described it as "the palette colour derived from the
faction index", which invited a reader to drop the member and regenerate it —
and that is wrong. A real faction palette is a short hand-picked list followed
by a generated tail: the first several colours are chosen by eye to be
distinguishable and pleasant, and only indices past the end of that list fall
back to a deterministic rule. Two implementations agree on the generated tail
and will not agree on the hand-picked head unless one of them copies the
other's literal table — so a reader that regenerates gets a *different colour
for the same faction*, which is a change to how every territory boundary in
the world reads.

Therefore:

- A writer MUST write `color` for every faction, including index `0`.
- A reader MUST use the stored `color` and MUST NOT regenerate it from the
  index, even when it has a palette rule of its own.
- A reader that meets a faction with no `color` (an archive from a writer that
  believed the earlier wording) MAY fill it from its own palette, and MUST
  report that it did — the colour it produces is its own, not the author's.

The palette rule an implementation applies **when creating a new faction** is
that implementation's business and is not part of this format. The format's
only claim about `color` is that whatever was chosen is written down.

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

**Provinces are the one entity an implementation may legitimately decline to
read.** In some models a province is not authored at all: it is a partition
re-derived on demand from territory and the settlements that seed it, which
means a stored province is redundant the moment either changes, and an
implementation that persisted one would be choosing between a stale answer and
its own fresh one on every load. Such an implementation MAY ignore
`entities/provinces.json` and `rasters/provinces.i32` entirely and rebuild both
from `rasters/territory.i32` and `entities/settlements.json`.

Two obligations remain if it does:

- It MUST NOT delete what it did not read. §6.2 applies unchanged — a
  re-deriving reader that writes the archive back either re-emits the stored
  province document untouched or refuses to overwrite. A province carries a
  `name`, and a name is authored data no derivation can reproduce.
- It MUST NOT write a province document of its own derivation into an archive
  whose provinces it ignored on the way in, which would replace the author's
  names with generated ones.

The slot exists so that an implementation which *does* author provinces has one
home for them. Nothing in the format requires an implementation to have the
concept.

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

### 9.6 `entities/journeys.json`

**Written since 2026-08-31**; this heading said "reserved" and the paragraph
under it said "not written by any implementation today". **Engine-owned in
this port since SP-1** (`STORY_PLANNING_SCOPE.md`, `750fe79`, 2026-09-21): the
payload is `InfraTools::journeys`, a typed `Journey` entity, which
`project_bridge.rs` writes and restores itself and lists in
`ENGINE_OWNED_SLOTS`, so the shell may not write the slot. Until 2026-09-24
this paragraph still said the writer was the shell and that the engine modelled
no journey. Written only when at least one journey exists. The shape:

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

This is still a slot §6.5 serves: an implementation whose journey planner
lives in its user interface rather than in its map engine may **carry** this
document rather than model it, and §6.5's text rule is then the whole of what
it has to get right. (This port did exactly that until SP-1.)


### 9.7 `entities/conflicts.json`

**Written since 2026-09-23** (`STORY_PLANNING_SCOPE.md` SP-4). Engine-owned
in this port: the store is the engine's, and the shell may not write the slot.
Written only when at least one conflict exists.

```json
{
  "next_id": 6,
  "conflicts": [
    { "id": 3, "name": "Siege of Kessra", "kind": "siege",
      "start_year": 212, "end_year": 214, "sides": [2, 1],
      "outcome": "Relieved in the third spring",
      "points": [[40.5, 17.25]],
      "anchor": { "kind": "province", "tid": 9, "at": [41.0, 18.0] } },
    { "id": 5, "name": "Northern front", "kind": "front",
      "start_year": 300, "sides": [3, 4], "outcome": "",
      "points": [[1.0, 2.0], [3.5, 2.5], [6.0, 1.0]] }
  ]
}
```

`kind` is one of `front`, `arrow` (lines, two points or more), `siege`,
`battle` (one point). `sides` are faction indices as in §9.2. `points` are
grid cells **as drawn**, not as currently displayed: when `anchor` is present
the display is `points` translated by the anchor's current position minus
`anchor.at`, so the shape moves with the place it is attached to. `anchor.tid`
is a settlement `tid` (§9.1); for `"province"` it is the `tid` of the
province's seed settlement, because a province `id` (§9.4) is re-issued by
every province pass. `end_year` absent = ongoing; `anchor` absent =
unattached. A row with an unknown `kind` is skipped, the rest of the document
opens; an anchor that resolves to nothing is kept and the shape drawn at
`points` as stored.

### 9.8 `entities/landmarks.json`

**Written since 2026-09-06; specified here 2026-09-24.** Owner ruling 10
(`LARGE_ITEM_RULINGS.md`, 2026-09-06) decided landmarks **persist** in this
slot and asked for an entry here; the slot shipped with its writer and this
section did not exist until now. Engine-owned in this port: the payload is
`WorldGen::landmark_store`, which `project_bridge.rs` writes and restores
(`LandmarksDoc`) and lists in `ENGINE_OWNED_SLOTS`. No `format_version` change
came with it — it is a new optional slot, which §6.3's unknown-entry rule
already lets an older reader carry.

```json
{
  "settings": { "caps": { "peak": 12 }, "armed": { "peak": true },
                "crowding": 1.0, "class_radius_km": [200.0, 34.0, 10.0, 6.0],
                "cross_type_competition": true },
  "results": {
    "landmarks": [ { "kind": "peak", "x": 40, "y": 17, "elevation_m": 3120.0,
                      "score": 0.91, "importance": 0.74, "causal": ["..."] } ],
    "funnels": [ { "kind": "peak", "candidates": 310, "rejected_constraint": 0,
                    "rejected_score": 250, "rejected_spacing": 40,
                    "rejected_cap": 8, "cap": 12, "placed": 12, "limit": "at_cap" } ],
    "seconds": 0.42
  }
}
```

(Values illustrative; the member names are `LandmarkSettingsDto`,
`LandmarkRunDto`, `LandmarkDto` and `LandmarkFunnelDto`.)

- **`settings` is written on every save**, because
  `LandmarkSettings::default()` is a full per-kind table, not "nothing" — absent
  and at-defaults are the same state.
- **`results` is the last run, written only when there is one.** Absent means
  no run is retained (never run, or invalidated since) — not "it ran and placed
  nothing". Ruling 10's reason for persisting it: the run's state cannot be
  recomputed.
- `x`/`y` are grid cells. On open, a landmark row with an unknown `kind`, an
  off-grid cell or a non-finite number is skipped and reported by count; the
  rest restore.
- The run's per-kind **reject list is deliberately not stored** — the funnel
  counts keep the true totals; see `LandmarkRunDto`'s doc for the size
  measurement behind that.
- **Stored twice when the POI icon pass has run**: each landmark it placed as a
  glyph is also a `poi` row with `"origin": "generated"` in
  `annotations/icons.json` (§11.2, §17).

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
the corresponding `year` value from `history/timeline.json`. Content and
validation are exactly §8's: `GW × GH` little-endian signed 32-bit integers,
row-major, faction id per cell, `0` = unowned. **Always the plain dump** —
§8.2's shuffle does not reach this directory, so a history entry has no
second spelling to go with the one grammar below.

**`<year>` has exactly one spelling.** It is the year's *canonical decimal*
form, and nothing else:

```
<year> ::= "0" | ["-"] ("1".."9") {"0".."9"}
```

That is: an optional leading `-` for a negative year, then digits with no
leading zero, and `0` written as the single character `0`. A leading `+`, a
leading zero (`007`), a decimal point, an exponent (`4.0e2`), a thousands
separator and surrounding whitespace are all **invalid names**, and a reader
MUST treat an invalid name as an unrecognised entry (§6.3) — ignore it, and
report it.

This is a rule about *names*, not about numbers, which is why §14.2's
"`1`, `1.0` and `1e0` are the same value" does not reach it: an entry name is
matched, not parsed, and two spellings of the same year are two entries
claiming one snapshot with no rule about which wins. Requiring one spelling
means the match is a string comparison in any language, and a year present in
`timeline.json` maps to at most one entry.

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

`family` is one of `settlement`, `feature`, `poi`, `custom`, `seamarks`.
`slot` names the symbol within that family. `set` is the custom set name and
is non-`null` only when `family` is `custom`. `scale` is a per-instance size
multiplier, `1.0` for a plain click placement.

**`origin`** — optional, added 2026-09-06 under owner ruling 14. One of
`manual` or `generated`. **It is omitted when the icon is hand-placed**, so an
existing document re-serialises byte-identically and no project is silently
rewritten on open (§6.2). Therefore:

- **absent means `manual`** — a document written before the member existed
  predates generated icons having anywhere to be recorded, so hand-placed is
  the only honest reading and never "unknown";
- **`generated` marks a row the automatic placement owns**, and is the marker a
  regenerate needs to replace only its own rows;
- **an `origin` that is neither takes `family`'s answer below and costs its
  row** — a row a reader cannot classify cannot be routed by the regenerate the
  field exists for. Case-sensitive: `"Manual"` is unrecognised. A reader MUST keep an icon
whose `slot` it cannot resolve, and render a placeholder — dropping it
silently loses the author's work over an art-pack mismatch.

**`seamarks` was missing from that list until 2026-09-06, and this port
writes it.** It is the port's own fifth family (owner ruling 2026-09-02), so
the reference writes no icon with it and a reference export can carry none —
but a project saved by this build can, and under the `family` rule below a
reader working from the old four-name list would have dropped every sea mark
in it.

**`family` takes the opposite answer to `slot`, and both are §6.4a.** An
unresolvable `slot` has a stated substitution (the placeholder above) and is
rung 1; a `family` outside the list has none — there is nothing to place the
icon in and every consumer indexes by it — so it is rung 2 and costs its
element. **This build skips that icon, keeps the rest of the array, and reports
the skip**, and MUST NOT read it as `feature` or as any other family:
guessing puts the icon under art the author never chose, which is §6.4a's
"inventing the world" rather than reporting a gap.

> **OPEN — this describes behaviour, and was briefly written here as a `MUST`.
> Downgraded 2026-09-06, unresolved, pending the owner.** §13.3.5 answers the
> *same-shaped* question the opposite way for vault links: *"A reader that cannot
> use a link keeps it, reports it, and writes it back byte-equivalent."* An icon
> row is not information-free either — `x`, `y`, the family text, `slot`, `set`
> and `scale` all survive an unknown `family` and could be held inert exactly as
> `LinkKind::Unknown` holds a kind. **The concrete cost of the skip:** a later
> build adds a sixth manual family, a user saves in it, opens in this build and
> saves — every icon of that family is gone permanently. Two lanes of one batch
> reached opposite dispositions on this, and a cross-lane verification pass
> caught it; **no shipped behaviour changed either way**, so what is at stake is
> whether the format blesses a lossy round trip. That is an owner call. An absent `family` member
is the same case, since it resolves to no family either.

`scale` is rung 1 with a stated substitution: a reader MUST read an absent,
zero or negative `scale` as `1.0`, the value a plain click placement writes.

Array order is draw order.

### 11.3 `annotations/regions.json`

```json
{ "region": { "x": 10, "y": 20, "w": 128, "h": 96 } }
```

The current region-of-interest marquee, in whole cells, or `region: null` for
none. `x`/`y` is the top-left cell, `w`/`h` the extent in cells; the marquee
covers the cells `x … x + w − 1` and `y … y + h − 1`.

A reader MUST clamp it to the grid rather than reject it: `x` and `y` into
`[0, GW − 1]` / `[0, GH − 1]`, then `w` and `h` down to what remains inside the
grid from that corner. A marquee saved against a larger world is recoverable
that way; rejecting it would lose a selection over an arithmetic detail.

**A `w` or `h` of `0`, or negative, means no region.** After clamping, a reader
MUST treat a marquee with `w < 1` or `h < 1` as though `region` were `null`,
and MUST NOT widen it to a minimum of one cell. An empty marquee selects
nothing, which is precisely what "no region" means, and inventing a one-cell
selection the author never made would hand a subsequent crop or export a
region of its own choosing. Writers SHOULD write `region: null` rather than a
zero-sized rectangle.

### 11.4 `annotations/measurements.json`

```json
{
  "gw": 2048, "gh": 1024,
  "measurements": [
    { "mode": "distance", "value": 120.25, "unit": "km",
      "points": [[10.5, 4.0], [88.0, 12.25]] }
  ]
}
```

Readings the author chose to keep. Registered 2026-09-03; like
`entities/journeys.json` (§9.6) this is a slot an implementation whose ruler
lives in its user interface **carries** rather than models, and §6.5's text
rule is then the whole of what it has to get right.

`mode` names what was measured — `distance`, `bearing`, `area`, `radius`,
`section` or `vertical` in this port. `points` are the fractional grid cells
that were clicked, in click order, and are the substance of the entry: the
reading can be taken again from them, and nothing else in the document can.

`value` and `unit` are the single number the reading came down to, and they
are **optional together**. `unit` is one of `km`, `km²` as `km2`, `m` or
`deg`. A writer that has no single number MUST omit both rather than write a
zero, because `0.0 km` is a measurement and "no reading" is not. A reader MUST
tolerate a `unit` it does not recognise by showing the value with the
producer's own unit string rather than assuming kilometres.

**Lengths and areas are canonical km and km², never a display unit.** This is
the same rule §13.1 states for `mapWidthKm` and the reference app states for
itself (*"units: display-only. Canonical storage stays km"*): an
implementation that offers miles converts at the readout, and a file whose
numbers moved with a preference would be unreadable by anything but the
session that wrote it.

**`gw`/`gh` are the grid the points were clicked on**, with the same names,
the same meaning and the same obligation as `drafts/paint.json`'s (§12): a
point is a grid-cell coordinate, so a reader MUST refuse a document whose
`gw`/`gh` are not the world's rather than showing a reading over ground it was
never taken on. Refusing means declining to *display* it; a reader MUST NOT
drop the document from an archive it rewrites (§6.2).

---

## 12. `library/` and `drafts/` — the five caller-owned slots

**This section said "reserved… not written today", and stopped being true on
2026-08-31.** All four original slots are written, and since 2026-09-03 all four
restore; `library/settlement_types.json` joined them on 2026-09-21 and was
added here on 2026-09-24.
They share one property that sets them apart from every other slot, and it is
the reason they are described together: each is a document a *caller* decides
about — save a copy without my drafts, import a library from another project —
rather than one the engine emits on every save. A conforming reader MAY ignore
any of them; none is required to display a world.

| Slot | Carries | Notes |
|---|---|---|
| `library/assets.json` | asset-pack info, collections, custom slots, per-slot metadata and scatter rules | **The item images are not in it.** The record carries each item's image *index*; the bytes those indices point at have no channel in this format. A restore therefore rebuilds every slot and zero items, and a reader MUST report that rather than presenting an empty library as a complete one. |
| `library/travel.json` | every **custom** animal, vehicle, vessel and party preset | Stock entries are deliberately absent: they are read-only by construction and rebuilt identically on every launch, so storing them would store one build's constants. A restore replaces the custom half and leaves stock alone. |
| `library/settlement_types.json` | the user's settlement types: `{"types": [{"id", "name", "kind", "specialisation", "traits": [...], "walls", "age_mode", "age_years", "name_pool", "usage_count"}], "faction_defaults": {"<faction index>": "<type id>"}}` | Written by the shell's `SettlementTypeStore.document()`, only when there is at least one type or faction default. An **absent** slot means "no types" and clears the outgoing project's on open; so does a document that does not parse. A type row with no `id` is skipped. |
| `drafts/paint.json` | the three committed paint layers, each a sparse `[index, value, …]` pair list, plus `gw`/`gh` | An index means nothing without the grid: a reader MUST refuse a document whose `gw`/`gh` are not the world's, because a layer decoded against a different grid is a scrambled picture, not a smaller one. |
| `drafts/sculpt.json` | the uncommitted stamp stack as *recipes* (feature key, seed, stroke points, globals, that feature's controls, `hidden`), plus the armed feature and next stroke's seed | Recipes, not height deltas — that is what makes a draft non-destructive. Same grid rule as `paint.json`, for the same reason: a stroke point is a grid-cell coordinate. |

See §16.3 for what a restored sculpt draft still cannot do, and §16.5 for why
the binary half of the asset library stayed out.

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

A record of which external Markdown notes are attached to which entities in
this project. It is at the root rather than under `annotations/` because it
points *outward*, at files the archive does not contain, which is a different
kind of thing from a mark drawn on the map.

```json
{
  "version": 1,
  "vaults": [
    { "id": "vault_9f2c1a04bb37de51", "display_name": "Elaris" }
  ],
  "links": [
    {
      "link_id": "link_5c8e11a0d3f47b62",
      "entity_kind": "settlement",
      "entity_id": 42,
      "entity_label": "Nareth",
      "vault_id": "vault_9f2c1a04bb37de51",
      "relative_path": "Locations/Nareth.md",
      "selection": { "type": "heading", "value": "The Old Quarter" },
      "source_modified": 1787605785,
      "source_hash": "3b1f9c2a77e04d18",
      "imported_text": "## The Old Quarter\n\nNarrow streets.\n",
      "edited_text": null,
      "imported_data": {
        "frontmatter": { "type": "town", "population": "8420" },
        "fields": { "Founded": "412", "Size / Population": "8,420" }
      }
    }
  ]
}
```

#### 13.3.1 The document

| Member | Type | Required | Meaning |
|---|---|---|---|
| `version` | integer ≥ 1 | MUST | The **link store's own** version, `1` today. Independent of `format_version` (§4): the two version different things, and a reader MUST NOT assume they move together. A reader that meets a higher value SHOULD read what it recognises, apply §14.3, and warn. |
| `vaults` | array | SHOULD | Every vault this project references. Absent or empty is valid — a project with links but no vault entry is damaged filing, not a parse error; see §13.3.4. |
| `links` | array | SHOULD | Every entity-to-note relationship. Absent or empty is valid and is the normal state of a project nobody has linked yet. |
| `snapshots` | object, string → string | MAY | Generated map snapshots of an entity, keyed `<entity key>\|<radius>` (e.g. `settlement:42\|local`; `cartalith-vault`'s `snapshot_key`), each valued with the image's path **relative to the vault root**. Keyed by entity, not by link: a snapshot exists whether or not a note is attached. Omitted, not written empty, when there are none; an older document without it is valid. Added to this table 2026-09-24 — `LinkStore::snapshots` had been written without being specified here. |

A writer MAY omit `vault.json` entirely when `vaults`, `links` and
`snapshots` are all empty (`LinkStore::is_empty`, the gate this port's writer
uses). It MUST NOT omit it merely because `links` is empty: a project whose
only vault state is a snapshot would lose it.

#### 13.3.2 `vaults[]` — a vault as the *project* knows it

| Member | Type | Required | Meaning |
|---|---|---|---|
| `id` | string, non-empty | MUST | Opaque, stable, unique within `vaults`. Referenced by `links[].vault_id`. A reader MUST NOT parse it. |
| `display_name` | string | MUST | What to call this vault to a person. May be empty. |

**There is deliberately no path here, and that is the point of the whole
document.** Where a vault lives is a property of *this machine*, not of the
project: the same project opened on a laptop and a tablet finds the same notes
at two different paths, and a stored path would be wrong on one of them the
first time it travelled. Binding an id to a location is the application's own
local state, stored outside the archive. A project whose vault is not bound on
this device is a normal, nameable condition (§13.3.5), not an error.

The consequence for id assignment: a writer SHOULD derive `id` from something
both devices can compute independently — the display name is the usual choice —
so that the same logical vault lands on the same id without either device
having seen the other's project file. A writer MUST NOT use a random or
clock-derived id, which makes the two devices' links permanently disjoint.

#### 13.3.3 `links[]` — one entity-to-note relationship

| Member | Type | Required | Meaning |
|---|---|---|---|
| `link_id` | string, non-empty | MUST | Opaque, unique within `links`. A reader MUST NOT parse it. If two links share one, the reader MUST keep the first and report the rest (§3's duplicate rule, applied inside a document). |
| `entity_kind` | string | MUST | One of `settlement`, `province`, `continent`, `faction`, `culture`, `landmark`. §13.3.4 fixes what each `entity_id` means. An unrecognised value MUST NOT drop the link — see §13.3.5. |
| `entity_id` | integer | MUST | The entity's own id, subject to §14.1. |
| `entity_label` | string | MUST | The entity's name **at the time the link was made**. May be empty. It is never used to resolve — §13.3.5. |
| `vault_id` | string | MUST | Matches a `vaults[].id`. |
| `relative_path` | string, non-empty | MUST | The note's path **relative to the vault root**, forward slashes, no leading `/`, no `.` or `..` segments. Case is preserved as written; a reader on a case-insensitive filesystem MAY match case-insensitively but MUST write back what it read. |
| `selection` | object | MUST | Which part of the note the link points at. §13.3.6. |
| `source_modified` | integer ≥ 0 | MAY | Seconds since the Unix epoch: the note's modification time as of the last import. `0` or absent means "not recorded". |
| `source_hash` | string | MAY | A digest of the imported bytes, as text. Compared **only for equality**, never interpreted; no algorithm is specified. Empty or absent means "not recorded". A writer that changes algorithm MUST clear every stored hash rather than leave two incomparable kinds in one document. |
| `imported_text` | string or `null` | MAY | What was read from the note. Absent or `null` means **linked but never imported** — a real state, distinct from "imported an empty file". |
| `edited_text` | string or `null` | MAY | The project-side working copy, present only once it differs from `imported_text`. Absent or `null` means "no local edit". |
| `imported_data` | object | MAY | The note's *structured* content. §13.3.7. |

Array order carries no meaning; a reader SHOULD preserve it anyway, so that a
load-and-save cycle produces a diffable file.

**No status member exists, and none may be added.** Whether a link is
connected, stale, cached, missing, unbound or locally edited is *derived* at
read time by comparing what is stored here against what the vault currently
holds. A stored status would be a second source of truth, stale the moment
anything outside the archive changed, and it would be believed. §13.3.5 gives
the derivation.

#### 13.3.4 What `entity_id` means, per kind

The link resolves against exactly one thing, and which one depends on
`entity_kind`:

| `entity_kind` | `entity_id` is |
|---|---|
| `settlement` | a `settlements[].id` from `entities/settlements.json` (§9.1) — an id, not an array index |
| `province` | a `provinces[].id` from `entities/provinces.json` (§9.4) |
| `continent` | a `continents[].id` from `entities/continents.json` (§9.5) |
| `faction` | a `factions[].id` from `entities/factions.json` (§9.2), which equals its array index |
| `culture` | a **0-based index into the implementation's own culture vocabulary** — see below |
| `landmark` | a **derived integer, not an id the generator issued** — see below |

`landmark` (added 2026-09-06) is the second odd one, and it is odd in the
opposite direction to `culture`: there is no id to carry. A generated landmark
is identified by its **kind and its cell** — this port composes
`"<kind>@<x>,<y>"`, e.g. `waterfall@120,64` — and the sequential number the
generator hands each placement is a position in one run's own result list that
moves the moment a cap moves or a type is switched off. Storing that number
would silently re-point every note.

So `entity_id` for a landmark is **the low 52 bits of a hash of that string**,
and the properties a reader needs are these:

- It is derived, so it is reproducible: a reader that can compute the same
  hash over the same key resolves the link without the archive carrying a
  registry. This port uses FNV-1a 64 over the UTF-8 key, written as 16
  lowercase hex digits, of which the **first 13** become the id. An
  implementation that hashes differently MUST treat these links as
  unresolvable (§13.3.5) rather than binding them to whatever it computes.
- It is **not injective**. A reader MUST resolve by recomputing the key of
  every landmark it has and comparing, and MUST treat *two or more* matches as
  unresolvable rather than choosing one.
- It respects §14.1 by construction: 52 bits is one bit under the 2^53
  ceiling.

`culture` is the odd one in the other direction and is worth naming as such.
A culture is not
generated with the world; it is a fixed row in a vocabulary the implementation
ships, which is why it is the only kind here whose id survives regenerating the
world *and* a save/load. That stability is a feature — a person's essay on a
people stays attached to that people — and it is also the one kind whose ids
two implementations must agree on out of band, because the archive does not
carry the vocabulary. An implementation whose culture vocabulary differs MUST
treat these links as unresolvable (§13.3.5) rather than binding them to
whatever sits at that index.

The other four ids — settlement, province, continent, faction — are **only as
stable as the entity**. Provinces may be
re-derived (§9.4) and renumbered; continents are ranked by size and renumber
when terrain edits merge or split a landmass; a faction's id is its row and
rows above a removed faction shift down. A landmark sits between the two
extremes: its derived id is stable while the feature keeps its kind and its
cell — across a re-run of the generation pass at the same seed, across a cap
change and across a type being switched off — and moves when the feature does.
This is not a defect in the format —
it is why `entity_label` is stored, and why §13.3.5 says what it says.

#### 13.3.5 Resolution, and a link whose target is gone

**Resolution is by `(entity_kind, entity_id)` and by nothing else.** A reader
MUST NOT fall back to matching `entity_label` against entity names. An earlier
revision of this document said it should; that was wrong and is corrected here.
Names are not unique, they are user-editable, and two settlements called
"Nareth" are exactly the case where a name-based fallback silently attaches a
person's notes to the wrong place — the one outcome worse than telling them the
link needs re-binding.

`entity_label` exists so that a reader can *say what was lost*: "this note was
linked to **Nareth**". It is what a person needs in order to re-bind the link
by hand, and it is never machine-resolved.

A link can fail to resolve in four independent ways, and a reader MUST keep the
link in all four, MUST report it, and MUST write it back unchanged:

| Condition | What it means |
|---|---|
| `vault_id` names no `vaults[]` entry, or that vault is not bound on this device | **unbound** — the project knows the vault; this machine does not |
| `(entity_kind, entity_id)` resolves to no entity, or `entity_kind` is a value this reader does not know | **unresolved** — show `entity_label` and offer a re-bind |
| The vault is bound but `relative_path` is not there | **missing** if `imported_text` is absent, **cached** if it is present |
| `source_hash` (or, if either side has none, `source_modified`) differs from the file as it is now | **stale** — the copy held here is older than the note |

Otherwise, if `edited_text` is present and differs from `imported_text`, the
link has **local changes**; if not, it is **connected**. Where both a hash and a
timestamp are available the **hash decides**: a file touched by a sync client
has a new modification time and identical bytes, and calling that stale trains
a person to ignore the warning.

**Nothing above is a reason to drop a link.** An unresolvable link is the
user's filing, and losing it quietly is the failure mode §14.2 already cost
this project once — an entire feature's data, on every launch, for its whole
shipped lifetime. A reader that cannot use a link keeps it, reports it, and
writes it back byte-equivalent.

#### 13.3.6 `selection` — which part of the note

A tagged object. Its `type` member names the shape of the rest:

| `type` | Other members | Meaning |
|---|---|---|
| `whole_document` | none | The link is to the entire note. |
| `heading` | `value`: string, MUST | The link is to the section under the heading whose text is `value` — the text alone, without the leading `#` characters or their count. |

`heading` is resolved by **matching the heading's text** in the note as it
currently reads, not by an offset into the file, so that an author editing the
paragraphs above a heading does not silently re-point the link at a different
section. If the heading is no longer present, the link is stale rather than
broken: the reader keeps it, reports it, and MAY fall back to showing whatever
`imported_text` holds.

An unrecognised `type` MUST be read as `whole_document` and reported. It MUST
NOT cost the link, and it MUST NOT cost the document: a `selection` shape added
by a newer writer is exactly the case §6.3 and §14.3 exist for, and refusing
the whole store over one would discard every link in the project to protect
against a narrower selection than the reader expected.

#### 13.3.7 `imported_data` — the note's structured content

Two maps taken from the same read as `imported_text`, both optional, each
absent or empty when the note has nothing of that kind:

| Member | Type | Meaning |
|---|---|---|
| `frontmatter` | object, string → string | Flat scalar keys from the note's leading metadata block. |
| `fields` | object, string → string | Labelled values from the note's own body template — the `**Name:** value` convention. |

They are **two maps, not one merged map**, because a note's metadata block and
its body can legitimately disagree (`type: town` in one, `Type: City` in the
other). Merging them needs a precedence rule this format has no business
inventing; the consumer decides, and to decide it must be able to see where a
value came from.

**Every value is a string, and that is a decision, not an oversight.** A note
whose metadata says `population: 8420` is stored here as the five characters
`"8420"`, never as the number 8420. Two reasons, and this project has already
paid for the first:

1. **It cannot be corrupted by a layer that floats numbers.** §14.2 records
   what happened when it could: a round trip through a component with one
   number type turned two integers into `1.0` and `1787605785.0`, the strict
   parser on the other side refused the document, and every link every user had
   ever made was discarded on each launch. JavaScript has exactly the same
   single number type, and an integer above 2^53 cannot round-trip through it
   at all (§14.1). A map of strings has no such failure mode.
2. **It is what the note said.** `8,420`, `~8000` and `8420` are three
   different things a person wrote. Parsing them into one number is the format
   deciding what the author meant, and it is not recoverable afterwards.
   Whoever consumes a value decides how to read it; the copy preserves it.

A reader MUST NOT coerce these values to numbers on load, and §14.2's
integer-shaped-float rule explicitly does **not** apply inside these two maps —
it governs values the format specifies as integers, and every value here is
specified as text.

Freshness is the link's, not the map's: `imported_data` is captured in the same
read as `imported_text`, under the same `source_hash`, so §13.3.5's status
answers "is this copy current" for both halves together. A reader MUST NOT add
a per-map timestamp; two freshness ideas that can disagree is a state nobody
can explain to a user.

#### 13.3.8 Ids and numbers

Every integer in this document — `version`, `entity_id`, `source_modified` — is
subject to §14.1's range rule and §14.2's integer-shaped-float rule. `link_id`,
`vault_id` and `source_hash` are **strings and MUST remain strings** even when
their content looks numeric; they are opaque tokens, and §14.1's closing
paragraph ("there are no string-encoded integers anywhere in this format") is
about integers stored as text, not about opaque identifiers that happen to be
hexadecimal.

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
- A reader MUST treat an out-of-range integer as a **damaged value**, and
  §6.4a decides what that costs: the value's own stated substitution if it has
  one, otherwise the array element it sits in, otherwise the document. An
  out-of-range `settlements[].id` costs one settlement, not the archive. Only
  `project.json` (§7) can reach the archive.

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

Vocabulary members (`kind`, `class`, `mode`, `family`, `size_mode`, `type`,
`entity_kind`, culture and government keys) are lowercase ASCII identifiers
with `_` as the separator. Each section states the closed set and what an
unrecognised value reads as. A reader MUST NOT crash on an unrecognised
vocabulary value.

**`sea_lane`, with an underscore — and the one collision this causes.** The
separator rule is applied without exception, including to the way kind
`sea_lane` in §9.3. The flat legacy layout's own way vocabulary spells the same
concept **`sea-lane`, with a hyphen**, and has done in every archive it has
ever written. The two are the same kind and MUST be translated at the boundary:
a reader normalising a flat archive into this format's vocabulary rewrites
`sea-lane` to `sea_lane`, and an interoperability writer (§1.1) rewrites it
back.

This is worth stating rather than leaving to be rediscovered because it is the
only place where this format's vocabulary collides with an existing key in the
very application the format was written to be implemented in — so it is the
one rename that a second implementer *will* meet, and will meet as a silently
unrecognised value (read as `track`, §9.3) rather than as an error. Nothing
else in §9-§13's vocabulary differs from the flat layout by punctuation alone.

The rule is not relaxed for it. A single-exception vocabulary is a vocabulary
every future member has to be checked against by hand, and one boundary rename
in one direction is cheaper than that, permanently.

Free text (`name`, `history`, label text) is arbitrary UTF-8 and may be empty.
Empty is a real value and MUST NOT be replaced by a placeholder on load.

---

## 15. The legacy flat layout — read-only

Every `Cartalith Gen1` export up to and including v2.10 uses this layout.
Readers MUST accept it (§1). **No writer produces it** except the explicitly
labelled interoperability export of §1.1.

Eight entries at the archive root, no directories:

| Entry | Contents |
|---|---|
| `params.json` | `{ "v": <number>, "GW": <int>, "GH": <int>, "state": { … } }`, plus an optional top-level `"origin"` — §7's `world.origin`, written by §1.1's interoperability export when it knows one. Top level, not inside `state`: `state` is merged wholesale into the reference app's own live state (§15.4), which is not a place to put a member of this format's invention. A reference export never carries it. |
| `heightmap.f32` | `GW × GH` little-endian f32, row-major |
| `heightmap_rg16.bin` | The **same** heightmap, 16-bit-packed. §15.2. |
| `temperature.f32` | as above, degrees Celsius |
| `rainfall.f32` | as above |
| `volcanic_field.f32` | as above |
| `impact_field.f32` | as above |
| `strahler_order.bin` | `GW × GH` unsigned bytes. Note the extension is `.bin`, not `.u8`. |

A real export carries a great deal more than these — a baked atlas, `map.png`,
a README, biome and lithology rasters, resource potentials, Köppen rasters,
wildlife regions, a tidal-range field, feature and settlement-seed documents
and an asset-library payload. §6.3's unknown-entry rule is what makes that
normal rather than corruption, and it has been in force since the first reader
existed. Of the extras, only `heightmap_rg16.bin` is read back by the flat
layout's own loader; the rest are written for downstream consumers and
recomputed on load.

**Mapping the flat layout onto the tree.** A reader that normalises everything
to the tree's model reads:

| Tree | Flat source |
|---|---|
| `world.grid_width` / `grid_height` | `params.json` → `GW` / `GH` |
| `world.seed` | `params.json` → `state.tect.seed` |
| `world.map_width_km` | `params.json` → `state.mapWidthKm` |
| `world.sea_level` | `params.json` → `state.seaLevel` |
| `world.wrap_x` | `params.json` → `state.world`, defaulting to `false` |
| `world.origin` | `params.json` → `origin` (top level), **absent when the file has none** — which is every export the HTML app itself has written |
| `params.json` → `reference` | the whole `state` object |
| `params.json` → `cartalith` | `state.cartalith` if present, else absent |
| `rasters/*.f32` | the same-named root entries |
| `rasters/heightmap.f32` | `heightmap.f32`, else `heightmap_rg16.bin` (§15.2) |
| `rasters/strahler_order.u8` | `strahler_order.bin` |
| `entities/settlements.json` | `params.json` → `state.places` |
| `entities/factions.json` | `params.json` → `state.civ.factionNames`, `.factionCulture`, `.factionReligion`, `.factionGovernment`, `.factionAgTech` — five parallel arrays indexed by faction, **plus a colour the file does not contain** (§15.3) |
| `entities/ways.json` | `params.json` → `state.civ.ways` (and `state.roads`, the older auto-network) |
| `entities/journeys.json` | `params.json` → `state.civ.journeys` |
| `rasters/territory.i32` | `params.json` → `state.civ.territory`, a **sparse** `[index, faction, index, faction, …]` array; cells not listed are `0` (§15.3) |
| `history/timeline.json` + `history/territory/<year>.i32` | `params.json` → `state.civ.timeline` and `state.civ.year` |
| `annotations/labels.json` | `params.json` → `state.labels` |
| `annotations/icons.json` | `params.json` → `state.mapIcons` |
| `appearance.json` | `params.json` → `state.viz` and the other presentation blocks |

### 15.1 The flat layout is not terrain-only, and assuming it is loses a project

**A flat archive does carry entities, history and annotations.** They are not
separate entries — they are nested inside `params.json`'s `state` object, which
is why an implementation that reads the flat layout as "seven rasters and a
parameter block" silently discards every settlement, faction, road, journey,
territory claim, recorded year, label and icon in the file. That is the single
most expensive mistake available when implementing §15, and this document
previously invited it by asserting the opposite.

The shapes under `state` are the flat layout's **own** vocabulary, not this
format's. Member names differ (`pts` for `points`, `km` for `length_km`, `pop`
for `population`, `brks` for `breaks`, `aIdx`/`bIdx` for `from`/`to`,
`villageAddon` for `village_seeded`, `type` for a way's `kind`), the way-kind
vocabulary differs by punctuation (§14.4), and territory is stored sparsely
rather than as a grid. A normalising reader translates; it MUST NOT assume a
member of the same meaning carries the same name.

Three consequences a second implementer should have in hand before starting:

- **`state.places` is not settlements-only.** The same array holds
  non-settlement points of interest, distinguished by `kind`. A reader
  normalising into `entities/settlements.json` MUST filter by §9.1's closed
  `kind` set and MUST NOT invent a settlement from a point that is not one.
- **A settlement's political role is in `kind`.** There is no separate
  `capital` member; §9.1 states the translation in both directions.
- **Provinces are absent by design.** The flat layout deliberately does not
  persist them, re-deriving them on demand instead — which is the behaviour
  §9.4 now permits explicitly rather than treating as a gap.

### 15.2 `heightmap_rg16.bin` — the flat layout's portable height fallback

Written by every flat export **alongside** `heightmap.f32`, and read only when
`heightmap.f32` is absent.

- **Length:** `GW × GH × 4` bytes — four bytes per cell, not two.
- **Per cell**, in order: the high byte of a 16-bit value, its low byte, then
  `0` and `255`. The last two bytes are padding to an RGBA pixel and carry no
  information; the packing exists so the field can be dropped into an image
  without re-encoding.
- **Decoding:** `height = ((byte0 << 8) | byte1) / 65535`, giving the same
  `[0,1]` normalised elevation `heightmap.f32` carries.
- **Encoding** is `round(clamp(height, 0, 1) × 65535)`, so a value is accurate
  to about 1.5 × 10⁻⁵ — lossy, and not bit-identical to the `.f32` beside it.

A reader normalising a flat archive MUST prefer `heightmap.f32` and MUST use
`heightmap_rg16.bin` only when the `.f32` is absent or unreadable, reporting
that it did — the elevations it produces are quantised, and a parity comparison
against a bit-exact engine will fail on them for that reason alone.

It has **no equivalent in the tree** and MUST NOT be written into one: §8.1
says why, and §18.4 records quantisation as a measured, deliberately unpulled
lever rather than an oversight.

### 15.3 Two things the flat layout cannot carry, and one that is not what it looks like

- **Faction colours are not in the file.** The flat layout stores faction
  names and attributes but no colour, regenerating each from a palette on load.
  A reader normalising into `entities/factions.json` therefore has no authored
  colour to copy and MUST supply one from its own palette and report that it
  did — see §9.2, which is the reason `color` is a stored member of the tree
  rather than a derived one.
- **Territory ids are 8-bit.** The flat layout's territory store holds one
  unsigned byte per cell, so a converted archive can carry faction ids `0`-`255`
  and no more. §8.1 states the reciprocal obligation for a narrow reader meeting
  a wide `territory.i32`.
- **`v` in the flat `params.json` is provenance, not a format selector.** The
  HTML app's own loader never branches on it — every compatibility shim it has
  tests for a missing *key* instead. A reader MUST NOT branch on it either, and
  MUST use §4's test.

### 15.4 The flat layout's shallow-merge hazard

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

### 16.1 The tile pyramid — **optional since owner ruling 28**; the baked atlas, still not stored

**This subsection said "not in the archive at all" and no longer does.** Owner,
2026-09-06: *"the LOD tiles should be stored in the save, or at least optional
to include."* What changed is the decision; the three reasons behind it were
re-checked one at a time rather than dropped, and two of them are still true
and are now **mechanisms in the format** instead of arguments against it.

1. **It is derived** — unchanged, and it is what makes the rest safe. Every
   tile is a rendering of `rasters/`, so a reader may drop the whole directory
   at any time and lose nothing but the time to re-synthesize.
2. **It is enormous** — unchanged, and **measured** rather than asserted (see
   §18.5). This is why the slot is optional, why the depth is the writer's
   choice, and why a writer is expected to show the size before it is paid.
3. **It goes stale invisibly** — *"a reader cannot cheaply tell which is
   older"*. **It can now**: `cartography/tiles/index.json` carries a
   `source_key` computed from the heightmap the archive was written with, and a
   reader recomputes it from the heightmap it just read. On a mismatch the
   tiles are **dropped, not drawn** — the ruling's own instruction, and the
   only behaviour that is safe when the alternative is a stale relief drawn
   over a re-sculpted world.

#### `cartography/tiles/index.json`

Written by whoever writes the tiles; **absent when there are no tiles**, which
is the normal case. All four members are REQUIRED when it is present, and a
reader that cannot find or parse it MUST drop every tile under
`cartography/tiles/` rather than guess.

```json
{
  "source_key": "8b17c0e5d3a94f22",
  "producer":   "cartalith-lod/v1;px=256;zb=4;mid=128;gain=256;...",
  "tile_w":     256,
  "tile_h":     164
}
```

- **`source_key`** — an opaque hex digest of the world the tiles were made
  from. This document does not standardise the hash, because both sides of the
  comparison are written by the same implementation and a second implementation
  is free to use its own; what it standardises is the **rule**: the key MUST
  depend on every world-derived input the tile synthesizer reads, and a reader
  MUST recompute it and drop the tiles on any mismatch. This port's inputs are
  the heightmap, `grid_width`, `grid_height`, `seed` and `sea_level`, and its
  digest is FNV-1a-64. `map_width_km` and `wrap_x` are deliberately **not**
  inputs — no part of the synthesis reads them, and hashing them would discard
  a valid cache on a metadata edit.
- **`producer`** — an opaque string naming the writer and its own constants
  (tile size, encoding, detail schedule). A reader MUST drop the tiles unless
  it produced this exact string itself. The key above covers the *world*; this
  covers the *renderer*, and neither substitutes for the other.
- **`tile_w`, `tile_h`** — one tile's pixel dimensions, the same for every
  level (a level changes a tile's **footprint**, not its pixel count). Every
  tile entry MUST be exactly `tile_w * tile_h * 3` bytes (RGB — see below); a
  shorter one is a truncated tile and MUST be skipped, exactly as §8's rasters
  are.

#### `cartography/tiles/<z>/<col>/<row>.u8`

One tile: **three `u8` bytes per pixel, R/G/B, row-major**, no header and no
length prefix — the same bare-dump rule §8 gives the rasters, and the same
reason the extension names the *element* type (one byte) rather than the pixel
width. `z` is the pyramid level, `col`/`row` the tile's index within that
level's `2^z × 2^z` grid. Leading zeros are not permitted in any of the three,
so one tile has exactly one name.

**This was one channel — a relief-detail shade multiplier the renderer applied
over the base raster — until `LOD_DETAIL_SCOPE.md` LOD-D2 (2026-09-21), and is
a colour picture now** (`renderBiomeTileRGBA`'s own output, alpha dropped since
it is always `255`): a deeper level shows real material boundaries, river bands
and crest strokes the base raster does not have, which a scalar shade multiplier
structurally could not. `producer` is the field that tells the two shapes apart
without a second extension — a reader MUST refuse a stored pyramid whose
`producer` string it does not recognise exactly (this section's own MUST,
above) rather than guess which layout it is from the byte count alone.

A producer whose tiles are some *other* encoding (not this RGB layout) must not
squeeze them in under this extension either: it needs its own extension under
the same prefix, and §6.3's unknown-entry rule carries anything unrecognised
through untouched. `cartography/tiles/0/0/0.png` is already exactly that case —
it is the entry §6.2's own round-trip fixture uses — and remains a foreign
entry. (The reference's *baked atlas* is also PNG, but under a path of its own
outside the archive entirely; it is a different payload, not this one in
another format.)

#### The baked atlas is still not stored

Unchanged, and for reason 2 taken to its conclusion: the atlas is the *whole
map* baked to a chosen depth, ahead of time, as colour PNGs, and it belongs
outside the project file — keyed so that reopening the same world finds it.
`project.json` carries `world.seed`, `world.origin` and the grid dimensions,
which is everything that cache key needs.

### 16.2 Settlement placement inputs

The suitability rasters, coast distance fields and travel-cost surfaces a
settlement was placed from are not stored. They are inputs consumed during
generation, they are large, and `entities/settlements.json` already holds the
result. Storing both would be the same overlap in another costume.

The per-settlement "why here?" explanation is likewise not stored: it is a
diagnostic over rasters that no longer exist, and reconstructing one from
stored data would be inventing it rather than recalling it.

### 16.3 `drafts/` — **written and restored since 2026-09-03**

**This subsection said "reserved, empty today" and no longer applies.** Its
reasoning was that "those editors can only be constructed over a live
generated world — they hold buffers keyed to substrate the archive does not
carry", so persisting a draft "would produce a file that opens with a draft
nothing can commit". Both halves turned out to be narrower than stated:

- The Paint editor's only world-keyed input is the land-only gate's water-body
  classification, and the archive carries what computes it —
  `rasters/heightmap.f32` and `rasters/rainfall.f32`, both required by §5.1.
- The Sculpt editor's are the river lock masks, and *having none* is a legal
  state already reached by any world generated with river carving off, not a
  missing value being stood in for.

`drafts/paint.json` and `drafts/sculpt.json` were written from 2026-08-31 and
restored from 2026-09-03; between those dates every painted cell and every
sculpt stamp was in the archive and applied to nothing on open. §12 is the
normative shape of both.

The one part of the old reasoning that survives is the last clause, and it is
narrower than "nothing can commit": a *sculpt* draft restored into a project
opened from disk is visible and editable but **cannot be committed**, because
baking it needs the generated world's substrate. Recalling a draft and baking
it are different questions — the same distinction §16.2 draws for settlement
placement.

### 16.4 The reserved rasters

`biome`, `lithology`, `koppen` and `wildlife` are computed by the generator and
discarded after use in the current implementation; there is no live value to
write. They are named in §8.1 so the second implementation does not choose
`rasters/biomes.u8` or `climate/koppen.u8` instead.

### 16.5 `library/` — the **binary** half, still not stored

**This subsection said "reserved, empty today" and that is no longer the
whole truth.** Both library documents are written and restored (§12). What is
still deliberately absent is the part the old reasoning was actually about:
the asset library's **art**. `library/assets.json` carries each item's image
*index*, name and transform; the pixels those indices address are not in the
archive, because embedding them is its own design question and answering it
badly would put images somewhere the tree would later have to move them out
of. A restored library therefore comes back as slot definitions with zero
items, and a reader that does not say so is presenting an empty library as a
complete one.

The travel library has no binary payload and is stored in full.

### 16.6 Ephemeral tool state

An in-progress measurement chain, a half-drawn polyline, the currently armed
brush. These are the state of a gesture, not of a project. Nothing that would
be discarded by clicking elsewhere is stored.

### 16.7 The edit history — and the save time, which is the file's own

The history ledger (`WorldGen::undo_ledger`) and the height-undo snapshots
behind it are session state. **Opening an archive clears them outright**:
`load_save` records a floor entry, and a floor entry's first act is to empty
the ledger, so no row written before a save can be read back after one. Storing
the rows would therefore be storing names for steps nothing could revert to —
the snapshots are 4 bytes per cell each and are not in the archive either.

**Nor is a "saved at" stamp**, and that is worth stating because it looks like
an obvious addition. The right dock's `COMMITTED` rule (`GUI_GAP_REGISTER.md`
ED-02) needs two things when a project is reopened: *where* the boundary sits,
and *when* the file was written. The first is the floor row the load has just
created — the world on screen **is** the file, so the rule sits under that row
and every later edit lands above it. The second is the archive's own
filesystem mtime, read by `undo::file_written_at_ms`.

Both answers exist for **every archive any version of this port ever wrote**,
including a `Cartalith Gen1` browser export. A stamp inside the file would have
bought nothing the mtime does not, and would have had to be spelled *unknown*
for the entire installed base. Verified against two archives written before the
boundary existed — a committed reference export
(`crates/cartalith-io/tests/fixtures/real_export_seed24601.zip`, flat layout,
21 days old) and this port's own tree writer from an earlier session — both of
which place the rule and report their true age.

A writer that wants the save time *inside* the archive already has it: the zip
container stamps every entry, and this port's writer populates them from the
clock rather than leaving the 1980 epoch — `project.json`, `params.json` and
each raster in an archive written 2026-09-06 14:39 local all read
`(2026, 9, 6, 12, 39, 28)` UTC. Adding a JSON key for it would be a second,
weaker answer to a question two existing mechanisms already answer.

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
- **§16.1's `cartography/tiles/` is the one slot whose index this crate writes
  itself**, rather than accepting from a caller. `ProjectWrite::lod_tiles` is
  `Option<LodTiles>` and `None` by construction; `write_project` computes
  `source_key` from the heightmap it is writing and `read_project` recomputes
  it from the heightmap it just read, dropping the tiles and warning on a
  mismatch. A caller cannot skip that check, which is the point — the
  alternative was a rule each caller had to remember. The producer half
  (`cartalith-godot`'s `lod_bridge::synthesize_pyramid_masks`,
  `tile_producer_id`, `pyramid_mask_bytes`) is the caller's, because the
  constants a `producer` string has to name live there.
- ~~**`annotations/icons.json` has no `origin` member**~~ — **IT DOES, as of
  2026-09-06. The bullet below is kept because its migration rule is now the
  shipped behaviour and was written before the member existed, which is why it
  did not have to be invented twice.** What is stale in it is only the tense and
  the final paragraph: `ManualIcon` now carries `IconOrigin { Manual, Generated }`
  defaulting to `Manual`, `IconDto.origin` is an optional string
  (`skip_serializing_if`, so **`Manual` is written as an ABSENT member** and an
  existing document re-serialises byte-identically), and `icon_from_dto`
  resolves absent → `Manual`, an unrecognised value → the row is dropped.
  Owner ruling 14 (2026-09-06) said it would: *"ONE LAYER, TWO ORIGINS. One collection with
  an `origin` field. The renderer draws one layer; M6 spacing sees everything,
  so generation cannot place a landmark on top of a hand-placed icon; a
  regenerate replaces only the generated ones."* Recorded here **before** the
  member exists so the migration rule is not invented twice: when it lands,
  **an icon with no `origin` is hand-placed**, because a document written
  without the member predates the generated pass having anywhere to write.
  Absent must read as `manual`, never as unknown, and an `origin` value that
  is neither `manual` nor `generated` takes `family`'s answer above rather
  than being defaulted — a row a reader cannot classify cannot be routed by
  the regenerate the field exists for.

  ~~The member is not here yet because the *format* is not what blocks it…~~
  **Superseded 2026-09-06.** That paragraph was right when written — the blocker
  was that `ManualIcon` recorded no provenance — and the field landed once a lane
  was granted `cartalith-assets`. **What has NOT changed:** a project that has run
  the POI automatic placement still stores each landmark **twice**, once in
  `entities/landmarks.json`'s `results` and once as a `poi` row here. `origin`
  now makes the two **separable** — a reader can tell which rows generation owns
  — and nothing de-duplicates them **in the archive**. On screen they no longer
  draw twice: since `af0129c` (2026-09-06) `map_overlay.gd` suppresses a
  generated `poi` glyph whose cell a landmark ring already marks
  (`_icon_shadowed_by_ring`). This bullet said both passes still drew until
  2026-09-24.
- §6.5's partition is `project_bridge.rs`'s `ENGINE_OWNED_SLOTS` — the
  fourteen documents this port models — against the six of `cartalith-io`'s
  twenty `DOCUMENT_SLOTS` it carries (`annotations/measurements.json`,
  `library/settlement_types.json`, `library/assets.json`,
  `library/travel.json`, `drafts/paint.json`, `drafts/sculpt.json`). Counted
  2026-09-24 from the two lists; this said eleven and six, with
  `entities/journeys.json` among the carried ones, until then —
  `entities/landmarks.json`, `entities/journeys.json` (SP-1) and
  `entities/conflicts.json` (SP-4) have all joined the engine-owned side. `caller_slot_refusal` is the one
  place that decides, and both directions of the channel go through it:
  `project_save_with_documents` refuses a modelled slot on the way in and
  `project_read_document` refuses one on the way out. `project_document_slots`
  and `project_engine_owned_slots` publish the two lists to GDScript.
- §6.5's text rule is why `ProjectData` keeps each document **twice**:
  `documents` holds the parsed, §14.2-coerced `serde_json::Value` the schemas
  in `project_bridge.rs` consume, and `document_text` holds the archive's own
  bytes with only a BOM stripped. Re-serializing a `Value` is not the same
  text — `serde_json`'s object map is a `BTreeMap`, so it sorts members, and it
  drops whitespace and re-emits the coercion. `project_open`'s `documents`
  return and `read_document` both come from `document_text`; the vault, which
  this port *models*, deliberately still goes through the coerced `Value`.
- Two readers, deliberately. `project_open` returns every carried document the
  archive held, which is the right shape when the caller is opening the project
  anyway; its keys are §6.5's "which documents did it contain" report.
  `project_read_document(path, slot)` answers the same question about a file
  the caller does **not** want to open — a shell reloading one of its own
  documents should not replace the world as a side effect (this example was
  saved journeys, which are engine-owned since SP-1 and so refused here) — and `cartalith-io`'s
  `read_document` gives it that without decoding a single raster.
- The writer's refusal of a document it would have to edit (§6.5's last
  paragraph) falls out of `write_project`'s existing validation: it parses each
  document to check it is JSON and writes the original bytes, so a BOM or a
  syntax error fails the save rather than being silently repaired.
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
- **§8.2's shuffle** is `project.rs`'s `write_planes` / `Raster::from_planes`,
  with `raster_entry_name` deciding the stored name and `SHUFFLED_INFIX` the
  infix. The reader tries the shuffled name first and the plain one second,
  and applies the inverse **only** to what it found under the shuffled name.
  The writer streams one plane at a time, so a shuffled raster never exists
  as a second whole copy in memory; the reader decodes the planes straight
  into the element vector. `history/territory/<year>.i32` goes through
  `write_dump`, the plain §8 encoding. The shell's gallery thumbnail
  (`open_project_dialog.gd`'s `_render_thumbnail`) reads the heightmap entry
  directly with `ZIPReader` rather than through the engine, so it carries the
  same two-name rule and gathers only the cells it samples. Backward
  compatibility is pinned by `a_version_1_archive_reads_exactly_as_it_always_did`
  over `crates/cartalith-io/tests/fixtures/project_v1_7x5.ctl`, an archive the
  unmodified version-1 writer produced before the shuffle existed; the stored
  layout — not just the round trip — by
  `a_shuffled_raster_round_trips_bit_for_bit_and_is_stored_as_planes`.
- Round-trip coverage lives in `crates/cartalith-io/src/project.rs`'s own test
  module, `crates/cartalith-godot/src/project_bridge.rs`'s,
  `crates/cartalith-godot/tests/project_round_trip.rs`, and — for §6.5 —
  `crates/cartalith-godot/tests/project_document_channel.rs`, which saves an
  `entities/journeys.json` carrying an id at §14.1's ceiling and free text with
  escaped quotes and four non-Latin scripts in it, reopens the archive, and
  asserts the text comes back byte for byte. Its fixture is pretty-printed with
  its members deliberately out of alphabetical order, and it asserts that
  re-serializing the parsed value would *not* reproduce it: without that, the
  byte-identity assertion could pass by coincidence.

**Four divergences from the rules §6.4a, §10.2, §13.3.6 and §7 now state
explicitly**, found on 2026-08-26 when this document was corrected against its
first independent implementation. Recorded here because a known divergence is
cheaper than a rediscovered one; none is fixed in this document, which owns the
format rather than the code.

- **`world.seed` is narrowed to 32 bits on load.** §7 and §14.1 allow a seed
  anywhere in ±(2^53 − 1); the reader converts it to a 32-bit signed integer,
  which saturates rather than failing. A conforming archive with a large seed
  therefore loads as a *different world* with no report. §14.1's own rule for
  this is "treat an out-of-range integer as damaged" — the range in question
  being the reader's, not the format's.
- **`history/territory/<year>.i32` names are parsed, not matched.** §10.2 now
  requires the canonical decimal spelling and makes anything else an
  unrecognised entry. The reader parses the stem as an integer instead, so
  `+7`, `007` and `7` all resolve to year 7 and the last one read wins
  silently.
- **An unrecognised `selection.type` costs the whole vault document.** §13.3.6
  requires it to cost at most the link, and states the substitution
  (`whole_document`, reported). The reader deserialises `vault.json` as a whole
  and discards the store when any link fails to parse — which is §14.2's own
  failure shape, one layer up.
- **An absent `capital` or `color` defaults silently instead of being
  repaired and reported.** Every member of the entity documents is read with a
  type default, so a settlement with no `capital` reads as `false` rather than
  as §9.1's `kind == "capital"`, and a faction with no `color` reads as black
  rather than taking §9.2's palette fallback with a report. Type defaults are
  the right *shape* of leniency — they are what makes §6.4a rung 1 cheap — but
  they are only correct where the default is the stated substitution, and for
  these two it is not.

**The §6.2 gap is closed for the tree layout, 2026-09-03.** §6.2 asks a reader
that writes an archive back to either re-emit what it did not understand or
refuse to overwrite. Both halves now take the first option:

- **Registered documents this build does not model.** §6.5's verbatim text
  goes out through `project_open` and comes back in through
  `project_save_with_documents` unchanged, so a shell that hands back what it
  was given round-trips `library/travel.json` losslessly through a build that
  has never heard of a travel preset. That is the mechanism §6.2 asks for; it
  still requires the shell to *use* it.
- **Unrecognised entries.** This bullet used to say "are not", and that was
  the honest state until the census grew bytes. `read_project` now returns
  `ProjectData::foreign` — every entry it did not consume, keyed by name,
  **with its raw bytes** — `project_open` holds it on `WorldGen::
  carried_foreign` for the life of the open project, and
  `project_save_with_documents` hands it back to `ProjectWrite::foreign`,
  which `write_project` re-emits verbatim. A payload written by a newer build
  survives being opened and re-saved by an older one. The keys are still the
  census, so anything that only wanted the names kept working.

  Three details are load-bearing rather than incidental:

  - **A name the writer produces itself wins**, and the carried copy is
    skipped — not a preference: `ZipWriter` refuses a duplicate entry name,
    so without the skip the *save* fails rather than the stale copy losing.
  - **The bytes are project-scoped.** They are cleared wherever a world ends
    (`release_world`, `load_save`), because grafting one project's foreign
    entries onto the next is the same defect the landmark settings and the
    vault links were each fixed for.
  - **The flat layout is deliberately excluded**, and this is the one place
    §6.2 is knowingly not met. A flat archive re-saved comes out as a *tree*,
    so carrying its entries would mix one layout's payloads into the other's
    namespace. The shell says so instead: opening a flat archive reports that
    it was read as the older format and that saving converts it.

  **No `format_version` bump.** Nothing about the format changed — this is a
  reader/writer obligation §6.2 already stated, met at last.

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
| **deflate — what was written until 2026-09-23 (`format_version` 1)** | **2.3 MiB** | **24.9 MiB** | **152.7 MiB** | **3.04 s** | **0.43 s** |
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

### 18.4 Two levers that were owner decisions — both decided 2026-09-23 (Ruling AJ)

**The byte-plane shuffle — ADOPTED, owner Ruling AJ, 2026-09-23; now §8.2.**
The paragraph below is the case as it was put to the owner, kept because the
hazard it names is the one §8.2's design answers:

> It is the only measured change worth having, it keeps method 8 so every zip
> reader still opens the container, and it is about ten lines in either
> language. What it costs is §8's promise that a raster entry is a bare
> little-endian dump a JavaScript reader can put a typed-array view straight
> onto. That promise would have to be replaced by a `format_version` bump and
> an explicit marker — and the hazard is that a reader which ignored the
> marker would not fail, it would read *plausible-looking noise*, which is the
> one failure mode this format is built to avoid. Making it fail loudly instead
> would mean distinct entry names, and §8's rule that the extension names the
> element type has no room for a second axis. Not adopted here; it needs an
> owner decision, and it is worth putting to one.

What shipped takes that paragraph's own way out: distinct entry names, with the
second axis carried by an infix in the **stem** (`heightmap.shuffled.f32`)
rather than by the extension, which still names the element. The `format_version`
bump is there too, but it is the declaration of a revision, not the guard; the
guard is that no reader of any version finds shuffled bytes under a name it
reads as a dump (§8.2).

**Quantisation — DECLINED, owner Ruling AJ, 2026-09-23.** Put in the same
ruling as the shuffle and not taken; the case as it stood: Storing the heightmap as `u16` with a scale and an offset,
GeoTIFF-style, halves the payload before compression and compresses far better
than f32 afterwards — the largest available win by some distance. It is
**lossy**. `PARITY_TESTING.md` and `DECISIONS.md` §7a make bit-exact raster
values a property this port tests against the reference engine, and a save
that returns a different float than it was given would break that on the load
path as well as the save path. Costed and deliberately not built; moving that
bar is an owner decision.

### 18.5 What a stored tile pyramid costs — measured, 2026-09-06

**Non-normative**, and the evidence behind §16.1's "it is enormous" and behind
this port's decision to ship the slot **off by default**. Owner ruling 28 asks
for exactly this: *"measure a real pyramid before writing a default into the
UI."*

Three real generated worlds at **2048 × 1311** (seeds 1337 / 987654 / 24601),
every tile produced by the shipping synthesizer and written into a real
deflate archive, then read back for its **compressed** size — not a codec
measured in isolation. Reproduce with
`cargo test -p cartalith-godot --release --lib -- --ignored --nocapture
measure_a_stored_pyramid` and the `CARTALITH_LOD_*` environment overrides its
own doc comment lists.

| Levels | Tiles | Raw (1 B/px) | **Stored** | vs. the 24.9 MiB archive at this grid (§18.1) |
|---|---|---|---|---|
| 0..=3 | 85 | 3.40 MiB | 0.16 .. 0.18 MiB | under 1% |
| 0..=4 | 341 | 13.65 MiB | 0.81 .. 0.89 MiB | 3.3% .. 3.6% |
| 0..=5 | 1 365 | 54.65 MiB | **4.40 .. 5.12 MiB** | 18% .. 21% |
| 0..=6 | 5 461 | 218.65 MiB | **21.87 .. 27.75 MiB** | **88% .. 111%** |

Only the 0..=5 and 0..=6 stored figures are printed as totals by the harness;
the two shallower rows are sums of its per-level lines, so they carry that
rounding.

**Four findings, and each changes something.**

1. **A six-level pyramid roughly doubles the file.** §18.1's *"any change that
   does not move the three float grids is optimising nothing"* is still true of
   every *other* slot; this is the one that outgrows them. Each further level
   is ~5× the last — level 7 extrapolates past 100 MiB — so no fixed depth
   covers the zoom range the compositor can actually reach: `level_for_zoom`
   returns up to 10, and a 2048-wide world at this port's `ZOOM_MAX_FLOOR = 64`
   asks for level 8 at 32 screen pixels per grid cell, deeper on a wider
   viewport. **Storing "the pyramid" is not an option at any depth; storing a
   bounded prefix of it is.**
2. **The 4/3 rule holds for raw bytes and not for stored ones.** Ruling 28's
   *"a pyramid is ~4/3 of its base level"* is exactly right uncompressed
   (218.65 / 164.00 = 1.333). Compressed it is **1.23-1.26×** across the three
   worlds, because the deeper levels carry more sub-cell detail and compress
   less well — the base level alone is 80-82% of the stored total, not 75%.
3. **Deflate does far better on a bare mask than PNG does.** The same tiles as
   RGB PNGs, stored uncompressed in the container, are **65.18 / 69.92 / 81.10
   MiB** at levels 0..=6 — **2.9-3.0× larger** than the same tiles as `.u8`. A
   PNG carries its own deflate, so the container never sees the
   three-identical-channels redundancy, and per-tile streams cannot share a
   dictionary. This is why the format stores a single-channel bare dump rather
   than an image.
4. **The compression ratio is depth-dependent, so a single number would be a
   model.** Levels 0..=5 deflate to 8.1-9.4% of raw; 0..=6 to 10.0-12.7%. A
   writer showing an estimate should quote the raw figure (exact, and free —
   it is `sum(4^z) × tile_w × tile_h`) with the measured band, not a fixed
   ratio.

### 18.6 The shuffle as shipped — measured, 2026-09-23

**Non-normative.** §18.1 was measured on 2026-08-25 by a harness that is not
in this repository, so its figures are *cited* here, not re-asserted. These are
new: the same shape of world (the shell's own generator defaults, seed 24601,
no civilisation layer), written by this port's real `write_project` before the
change (at `064b724`, `format_version` 1) and after it (`format_version` 2),
each archive reopened and its heightmap compared bit for bit before its size
was recorded. Reproduce with
`CARTALITH_SAVE_SIZES=512x512,2048x1311,4096x4096 cargo test -p cartalith-godot --release --test project_round_trip -- --ignored --nocapture measure_a_real_save`.

| Grid | v1 archive | **v2 archive** | smaller by | v1 write, median (min..max, 5 runs) | **v2 write** |
|---|---|---|---|---|---|
| 512² | 2.30 MiB | **1.72 MiB** | 25.2% | 0.040 s (0.039..0.040) | **0.030 s** (0.030..0.031) |
| 2048×1311 | 24.74 MiB | **16.53 MiB** | 33.2% | 0.416 s (0.415..0.422) | **0.307 s** (0.304..0.327) |
| 4096² | 151.41 MiB | **96.42 MiB** | 36.3% | 2.790 s (2.553..2.825) | **1.770 s** (1.751..1.782) |

Per entry at 4096², v1 → v2: `heightmap` 53.73 → 34.05 MiB, `temperature`
54.40 → 34.58 MiB, `rainfall` 42.45 → 27.05 MiB, `volcanic_field` 0.55 →
0.46 MiB, `impact_field` 0.10 → 0.11 MiB; `strahler_order.u8` is untouched
at 0.16 MiB.

**What this confirms and where it differs.** §18.1's 33% at 2048×1311 and 36%
at 4096² reproduce to within a point. At 512² it measured 27% and this
measures 25.2%; §18.1's own rows for that size are rounded to 0.1 MiB (2.3 and
1.7), which by themselves allow anything from 22% to 30%, so the difference is inside that
rounding and no cause is claimed for it. The write is faster at every size, as
§18.1 found, by 25-37%. Read time was not measured this time. `impact_field`
grows by 0.01 MiB: a near-empty field has no run for the shuffle to lengthen,
and that is the one entry it costs anything on.

The before and after runs were separate invocations minutes apart on the same
machine, each run alone; they are not interleaved, so the timing columns are a
direction and a magnitude, not a controlled comparison.
