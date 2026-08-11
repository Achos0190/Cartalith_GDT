# Reading the HTML app's `.zip` saves

MVP reads an existing save's terrain data. Writing one is out
(`MVP_SCOPE.md` point 12).

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

## Entries the MVP reads

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

## Deferred

- **Writing a save.** Even a minimal `params.json` plus three `.f32` fields that
  the HTML app could reopen is a reasonable near-term goal — confirm before
  spending time on it.
- **Civ and UI payloads** — settlements, factions, territory, ways, labels, icons,
  Asset Library. There is nothing in the port to deserialise them into yet.
- **Saves written by older HTML versions**, where `loadZip()`'s own compatibility
  shims fill in historical defaults. MVP reads current-version exports.
