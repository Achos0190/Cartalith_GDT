# Provenance: sources, algorithms, formats

The engine's scientific grounding is real and cited. A rewrite that re-derives it
from scratch loses two hundred versions of calibration, so this file records
where the knowledge lives and what the port must carry forward.

Three layers, each handled differently:

1. **Academic grounding** — already credited in the app and derived in
   `docs/research/`. The port inherits it by porting the formulas.
2. **Algorithms** — published methods, independently implemented. Some can be
   swapped for a Rust crate; some cannot, because parity depends on their exact
   behaviour. The table below says which.
3. **File formats** — ZIP, DEFLATE, PNG, GeoJSON. All standard, all replaceable
   by crates.

## 1. Academic grounding — already credited, must travel

**The app ships a credits screen** (`#creditsModal`, "Credits & academic
principles") listing the methods studied and their sources, with the standing
claim that *algorithms were studied, not copied — every solver is an original
implementation*. That claim is the port's inheritance too: porting your own
implementation of a published method carries the same standing.

**The native app needs its own credits screen.** Dropping it in the rewrite would
quietly withdraw attribution the HTML app has always given. Not MVP-blocking —
but it belongs in Phase 1's definition of done, not discovered at release.

**`docs/research/` holds the derivations** — 38 documents, each carrying the
citations and the reasoning for one subsystem. These are not background reading;
they are why constants have the values they do. The ones the terrain MVP depends
on:

| Document | Grounds |
|---|---|
| `scale-invariant-terrain.md` | `terrainDetailK`, `riverFlowThresh`, channel initiation vs. map scale |
| `natural-rivers.md`, `river-overhaul.md`, `multiscale-rivers.md` | drainage networks, Strahler order, stream burning |
| `pipeline-order-audit.md` | why the generate() stages run in the order they do |
| `solar-energy-budget.md`, `gravity-influence.md` | insolation, axial tilt, rotation, lapse rate |
| `weather-model-v2.md`, `system-coupling-audit.md` | wind, moisture advection, climate↔erosion coupling |
| `physical-model-tails.md`, `engine-optimization.md` | model limits and where they were deliberately cut |

**Copy `docs/research/` into the new repository** — at minimum the six above.
A Rust port whose constants have no reachable derivation is a port that will be
"cleaned up" by someone who cannot see why 0.0004 is 0.0004.

Representative citations, so the shape is clear: Braun & Willett (2013) for
stream-power incision; Strahler (1957) and Leopold & Maddock (1953) for drainage
order and hydraulic geometry; Mei et al. (2007) for velocity-field erosion;
Montgomery & Dietrich (1988) for channel initiation; Ekman (1905) with
Sverdrup (1947) and Stommel (1948) for ocean currents; North & Coakley (1979) for
the energy-balance obliquity term; Christaller (1933), Brandes (2001),
Zipf (1946), Verhulst (1838), Tallavaara et al. (2018) for the civilisation
layer; Diocletian's Price Edict and Broadberry & Gardner (2013) for transport and
agricultural ratios. The credits modal and `docs/research/` carry the full list —
this is a sample, not a replacement.

## 2. Algorithms — what may be swapped for a crate, and what may not

The parity discipline (`PARITY_TESTING.md`) decides this, not taste. An algorithm
whose *exact output* feeds the heightmap must be hand-ported; one that only has
to be *correct* can come from a crate.

| Algorithm | Used for | Port as |
|---|---|---|
| **mulberry32** (PRNG) | every seeded decision | **Hand-port.** A different PRNG changes every world. Port and test it before anything else. |
| **Value noise / fBm / ridged** | terrain, warping, detail | **Hand-port.** `noise-rs` implements different hash and lattice functions; its output will not match. |
| **Priority-flood depression fill** (Barnes-style) | `buildWaterBodies`, lake classification | **Hand-port, carefully.** Equal-priority pop order decides the fill tie-break and therefore lake shape. v1.87 documents keeping the exact heap structure for this reason. |
| **JFA Voronoi** | plate assignment | **Hand-port.** Plate boundaries are the base of the whole heightmap. |
| **Chamfer distance transform** | coast/ocean distance fields | **Hand-port.** Cheap, and the approximation's exact values feed placement. |
| **D8 flow routing, Strahler ordering** | drainage network | **Hand-port.** The core hydrology. |
| **Blue-noise / dart-throwing rejection** | settlement and scatter placement | **Hand-port** where seeded and persisted; a crate is fine for anything regenerated freely. |
| **Dijkstra, Prim MST, A\*** | roads, sea lanes, routing | **Crate is fine** (`petgraph`) *if* tie-breaks match; otherwise hand-port. Routing is civ-layer, outside MVP — decide then. |
| **Brandes betweenness** | settlement centrality | **Crate is fine** (`petgraph`). Civ-layer, post-MVP. |
| **Catmull-Rom, RDP simplify** | route smoothing and thinning | **Crate is fine.** Presentation geometry, no parity claim. |
| **FNV-1a** | cache keys, test hashes | **Crate is fine** (`fnv`) — or hand-port, it is six lines. Must match only where golden hashes are compared. |
| **CRC32** | ZIP integrity | **Crate** (`crc32fast`). Standard, no parity surface. |

The rule behind the table: **hand-port anything upstream of the heightmap;
take a crate for anything downstream of the pixels.**

## 3. File formats and libraries

The HTML app has **zero runtime dependencies** — it implements these itself
because a browser page loading from `file://` cannot assume a package manager.
That constraint does not apply to a Rust binary, so the port should use crates
rather than re-implementing a ZIP writer.

| Format | In the HTML app | In the port |
|---|---|---|
| **ZIP (PKZIP)** | hand-written `zipStore` / `unzipAny` | [`zip`](https://docs.rs/zip) crate |
| **DEFLATE** | native `CompressionStream('deflate-raw')`, added v1.90 | via `zip`, or `flate2` |
| **CRC32** | hand-written | `crc32fast`, via `zip` |
| **PNG** | canvas `toBlob` | `image` crate |
| **JSON** | `JSON.stringify` / `parse` | `serde` + `serde_json` |
| **GeoJSON** | hand-built, RFC 7946 shape | `geojson` crate, or hand-built again |
| **`.f32` field dumps** | raw `Float32Array` bytes | plain little-endian reads — see `SAVEFILE_COMPAT.md` |

One caveat carried from `SAVEFILE_COMPAT.md`: the app's GeoJSON export uses
**local planar kilometres, not WGS84**, and says so — a generated world has no
true georeference. Keep that disclosure if the port keeps the export.

## Licence position

The HTML app is original code throughout, implementing published methods. The
port continues that: it re-implements the author's own implementations, in
another language.

Where the port takes a crate instead (the table above), that crate's licence
applies to it. Most of the Rust ecosystem is MIT/Apache-2.0 dual-licensed, which
is compatible with anything reasonable — but run `cargo deny` or `cargo license`
before release rather than assuming. That check belongs in Phase 1's definition
of done alongside the credits screen, since both are release-blocking and neither
is visible until someone looks.

Skills vendored into this folder carry their own licences — see `SKILLS.md` and
each skill's own `ATTRIBUTION.md`.
