# The HTML app's `.zip` save format

MVP read an existing save's terrain data; writing one was out (`MVP_SCOPE.md`
point 12). **A writer exists now** (owner-authorised 2026-08-23) — see
[Writing a save](#writing-a-save) at the end, which also records the three
things the real implementation clarified about the format.

Everything below was verified by reading `exportZip()`, `zipStore()`,
`serializeState()`, and `f32bytes()` in `Cartalith Gen1 v2.10.html`. Re-check
against the live file when implementing — the format has been stable since v1.90,
but this document is a summary, not a schema.

## It is a standard ZIP — use the `zip` crate

`zipStore()` writes genuine PKZIP: local headers (`PK\x03\x04`), a central
directory (`PK\x01\x02`), an end-of-central-directory record (`PK\x05\x06`),
standard CRC32, and standard method codes (`0` STORE, `8` raw DEFLATE, as
`CompressionStream('deflate-raw')` produces). Nothing is custom.

The Rust [`zip`](https://docs.rs/zip) crate should read these directly. Confirm
that early — opening a real export should be one of the first things
`cartalith-io` does, before anything is built on the assumption.

DEFLATE arrived in v1.90; entry names and contents did not change, and pre-v1.90
STORE-only saves still read. The `zip` crate handles both methods by default.

## Entries this port reads — and, since 2026-08-23, writes

| Entry | Contents | Format |
|---|---|---|
| `params.json` | `{ v, GW, GH, state }` — `state` is a deep clone of the whole app state | UTF-8 JSON |
| `heightmap.f32` | heightmap, `GW*GH`, row-major (`y*GW + x`) | raw LE `f32` |
| `temperature.f32` | °C, same layout | raw LE `f32` |
| `rainfall.f32` | same layout | raw LE `f32` |
| `volcanic_field.f32` | volcanism stamp byproduct | raw LE `f32` |
| `impact_field.f32` | crater stamp byproduct | raw LE `f32` |
| `strahler_order.bin` | stream order, `0` = non-channel | raw `u8` |

A real export also carries biome and lithology rasters, resource potentials,
settlement seeds, wildlife regions, Köppen rasters, an optional baked atlas, an
Asset Library payload, `map.png`, and a README — all civ or UI output, all outside
MVP scope.

**Ignore unknown entries; do not error on them.** A real save always contains more
than the reader wants, and that is normal rather than corruption.

## `.f32` is a bare byte dump

`f32bytes(a)` is `new Uint8Array(a.buffer.slice(0, a.length*4))` — the raw bytes of
a `Float32Array`, little-endian IEEE-754 everywhere that matters. So
`heightmap.f32` is exactly `GW*GH*4` bytes: no header, no length prefix, nothing.

In Rust, read it explicitly rather than casting:

```rust
let values: Vec<f32> = bytes
    .chunks_exact(4)
    .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
    .collect();
```

A zero-copy `bytemuck::cast_slice` is tempting, but the `Vec<u8>` the `zip` crate
returns is allocator-aligned, not guaranteed `f32`-aligned. Copy, or check
alignment first.

## `params.json`

`serializeState()` writes `{ v: VERSION, GW, GH, state: <deep clone> }`. The state
object is large — every generation block (`tect`, `volc`, `crater`, `climate`,
`erosion`, `stream`, `glacial`, `coastal`, `planet`, `world_structure`) plus civ
and UI state the MVP has no use for.

Two workable approaches:

1. Parse as `serde_json::Value` and pull only what the terrain pipeline reads —
   `GW`, `GH`, `state.tect.seed`, `state.mapWidthKm`, `state.seaLevel`, and each
   stage's own parameters. Fastest to MVP.
2. Define a struct with `#[serde(default)]` throughout, growing it as the port
   grows. More future-proof; worth switching to once the shape settles.

Either way, read the live `serializeState()` and `state` when implementing. `state`
has grown across 210+ versions and the list above is a sample.

## What "reads a save" means

Pull `GW`, `GH`, seed, map width, and sea level from `params.json`, and the
`heightmap`/`temperature`/`rainfall` arrays directly. No regeneration — the save
already holds the generated fields. Godot then renders them exactly as it renders
a fresh `generate()`.

This is simpler than the generation path, and it doubles as golden data: a real
export is a golden fixture that costs nothing extra once the reader exists
(`PARITY_TESTING.md`).

## Writing a save

`cartalith_io::write_save` (`crates/cartalith-io/src/save.rs`), driven by
`WorldGen::save_project(path)`. It writes exactly the seven entries the table
above lists, in `exportZip()`'s own order, DEFLATE-compressed (method 8, the
reference's own method from v1.90). Nothing else: the atlas, `map.png`, the
README, the biome/lithology/resource rasters and the Asset Library payload are
all things a reader must tolerate and a writer need not produce.

Verified three ways rather than one, because the failure this format makes
easy is a file that opens cleanly and is quietly wrong:

- `crates/cartalith-io/tests/golden_parity_save_writer.rs` re-writes a **real
  HTML-app export** and checks the result against that fixture's independent
  value capture, not against the reader that shares the writer's code.
- `crates/cartalith-godot/tests/save_round_trip.rs` generates a real world,
  saves it, reloads it, and then regenerates from the restored parameters —
  the strongest available statement that nothing generation depends on was
  lost.
- `godot-project/_save_probe.gd` does the same through the real GDExtension,
  decoding `heightmap.f32` in GDScript and comparing it to `sample_cell`.

### Three things the implementation clarified

1. **`loadZip()` merges `state` shallowly.** `Object.assign(state, pk.state)`
   means any nested block a writer emits *replaces* the reference's whole
   default block rather than merging into it. A writer that emitted
   `tect: {seed: N}` alone — the minimum this port's own reader needs — would
   leave the reference app with an undefined plate count, drift, warp and blur
   radius. Every nested block written must therefore be complete, or absent.
2. **`loadZip()` shims most blocks and not all.** `climate`, `stream`, `velo`,
   `glacial`, `coastal`, `planet`, `planet.tides`, `world_structure` and `viz`
   are each `Object.assign`ed over a default literal, so a partial one is
   safe. `tect`, `volc`, `crater` and `erosion` are not. This port covers
   `tect` (bar the four keys `loadZip` backfills individually —
   `tectonicGraph`, `foldIntensity`, `trenchDepth`, `faultBlock`), `volc` and
   `crater` in full. **It does not write `state.erosion` at all**: it models 2
   of that block's 16 keys (`diffuseD`, `diffusePasses`), and writing those
   two would replace the reference's entire droplet-erosion parameter set.
   A save this port writes reopens in the reference app with the reference
   app's own `erosion` defaults — a real, disclosed limitation.
3. **The `v` field is provenance, not a format selector.** `loadZip()` never
   branches on it; every compatibility shim it has tests for a missing *key*.
   This port writes `210`, the frozen reference snapshot it is built against.

### The parameter block, and one judgment call

`params.json`'s `state` carries every generation parameter **twice**:

- at its reference path (`tect.blurR`, `climate.latN`, …), for the HTML app;
- under `state.cartalith`, keyed by this port's own dotted parameter keys.

The second copy is the one this port reads back, and it exists because ten of
this port's parameters have no reference equivalent at all (`use_gpu`, the six
erosion-pass toggles, `passes.sediment_capacity`, `passes.tidal_k`,
`climate.terrain_wind_deflection` — the reference deleted its own
`terrainWind` in v1.78). Without it every one of those would be silently lost
on each save. `state.cartalith` is a key the reference never wrote, so
`Object.assign` carries it straight through and `serializeState()` writes it
back out: **a save can round-trip through the reference app without losing
this port's parameters.** The mapping table and its drift guard live in
`crates/cartalith-godot/src/params.rs`.

**Judgment call, disclosed:** this document's earlier "Deferred" note said to
*confirm before spending time on* HTML-app readability. The reference half was
written anyway, because `state.tect.seed` is mandatory for this port's own
reader and point 1 above then forces `tect` to be complete — and once one
block must carry reference names, the rest cost one table column each. What
was *not* done on that account is `state.erosion` (point 2) and
`world_structure.archetype` (this port stores the archetype's knobs, not its
name, so a reopened save shows `earth`).

### Deliberately not written

- **Civ and UI payloads** — settlements, factions, territory, ways, labels,
  icons, paint and sculpt drafts, the Asset Library. `load_save` clears all of
  them for a loaded world (they need tectonic substrate the format does not
  store), so writing them would produce a file this port cannot read back.
  This is the ceiling `GUI_GAP_REGISTER.md` JP-06/JP-08 and MEA-07 now sit
  against: the *writer* exists, and what remains is a channel for
  GDScript-owned project state to reach `state`.
- **Saves written by older HTML versions**, where `loadZip()`'s own
  compatibility shims fill in historical defaults. This port reads
  current-version exports.
