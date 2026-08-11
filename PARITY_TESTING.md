# Parity testing

The port must reproduce the JS engine's numbers at fixed seeds, not merely look
similar (`DECISIONS.md` §7). This is how.

## Tolerance, not bit-identity

`tests/perf/hash_gen1.js` asserts exact FNV-hash identity between HTML versions,
which works because both sides run identical JavaScript doing identical operations
in identical order. **That standard does not survive a language change.** Rust
float arithmetic, LLVM's scheduling and vectorisation, and JS float semantics
diverge on the same formula, and the pipeline is a long chain — substrate →
height → erosion → hydrology — where per-step differences compound.

Set the tolerance carefully. Too loose stops catching bugs; too tight fails on
harmless noise and teaches whoever maintains this to ignore red tests.

## Extracting golden data

1. Use the newest `Cartalith Gen1 v*.html` (root `CLAUDE.md`'s file table names
   it) as reference.
2. Reuse the harness pattern in `tests/perf/hash_gen1.js` and `tests/stub_head.js`
   — they already run the engine headlessly and read out `field`, `tempField`,
   `rainField`, and `flowField` after `generate()`.
3. Cover at least three configurations: the app's own default (reference seed,
   512px, 800 km), one small map width, and one world-scale width. Scale extremes
   are where v1.60, v2.05, and v2.07 all found real bugs.
4. Dump raw float arrays in the `.f32` format `exportZip()` already writes, rather
   than inventing one. Store them in the new repository under
   `cartalith-engine/tests/golden/`. Commit at the smallest resolution that still
   exercises the pipeline — 512px checks in comfortably, 2048px does not.

## What tolerance means

**Per field, not one global number.** Height sits in `[0,1]`, temperature spans
roughly −40 to 40 °C, and flow covers orders of magnitude between a dry cell and a
river mouth. Give each its own tolerance and record the reasoning, as this project
does for every other constant.

**Aggregates alongside per-cell checks.** Also assert distribution properties —
mean, land fraction, min and max. The v1.25 sea-level fix and v1.34's food-shed
calibration both turned on measuring the real distribution rather than an assumed
one, and both are the kind of bug a per-cell check misses or over-flags.

**A red test means re-read the JS, not widen the tolerance.** Loosening until
green defeats the discipline. If a genuine, understood difference exists, document
it and adjust deliberately — the same way the HTML CHANGELOG discloses every
deliberate re-baseline.

## Port the RNG first, and the noise second

The JS engine uses **`mulberry32`**, and almost everything derives from it: plate
placement, noise seeding, feature placement. A different PRNG — even a better one
— makes every downstream comparison fail for reasons unrelated to whether the port
is correct.

Port `mulberry32` and test it alone before anything depends on it. Then do the
same for `hash`, `vnoise`, `fbm`, and `ridged`, testing each against input/output
pairs pulled from the JS engine.

This is the single most expensive mistake available here, and the cheapest to
avoid.

## Test structure

```
cartalith-<stage>/tests/golden_parity.rs
```

One test per pipeline stage — substrate, height, climate, erosion, hydrology —
each feeding that stage a known-good input from the *previous* stage rather than
re-running the whole pipeline. A failure then names the stage that broke.

Add one end-to-end test over the configurations above.

Keep every one runnable under plain `cargo test` with no Godot present. That is
exactly why no engine crate may depend on `gdext` (`ARCHITECTURE.md`).
