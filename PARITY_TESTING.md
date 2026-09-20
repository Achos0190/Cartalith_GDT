# Parity testing

The port must reproduce the JS engine's numbers at fixed seeds, not merely look
similar (`DECISIONS.md` §7). This is how — **for the CPU reference pipeline**,
which this document still governs in full.

**GPU/optimized paths are a documented exception, not silently unmeasured**
(`DECISIONS.md` §7a, added 2026-08-16). Where JS-array diffing is genuinely
impractical — the GPU-compute pilot's `hash()` case, where JS's own
double-precision rounding at ~2^61 has no `f32`/WGSL equivalent — the bar
shifts to: same underlying academic principle (cite it, the way
`PROVENANCE.md` already does), and an equal-or-better visual/qualitative
result, judged by looking at it, not by an assertion. This is a narrower
carve-out than it sounds: it applies when a real, understood, reported
reason blocks numeric comparison, not as a default excuse to skip testing
because writing a golden test is more work. If a GPU/optimized path *can*
be tested against the CPU reference at a real tolerance, do that — this
exception exists for the `hash()`-shaped case, not as blanket permission.

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

## Deliberate re-baselines are decisions, not failures (`DECISIONS.md` §7n)

A golden fixture is captured once, against a frozen reference (root
`CLAUDE.md`'s Constraints section). The source engine keeps moving after that
freeze, and `RC_ENGINE_CHANGES.md` documents versions where the owner changed
`field` — the actual generated output, not a rendering detail — on purpose.
Its own words: *"a world generated from the same seed does not come back the
same, which is a decision to carry across deliberately, not a regression to
chase"* (§8.1). A golden test has exactly one failure mode, red, so today
nothing tells a porter *"the source engine re-baselined on purpose and this
port has not caught up"* apart from *"something broke."* This section is the
mechanism that tells them apart. It does not carry any upstream change into
the port by itself, and it makes no claim about which of them already are —
that survey is `OUTSTANDING_WORK.md` §2.9, open.

**The mechanism has two parts, and a re-baseline that has actually reached a
test needs both.**

1. **A named list — the register below.** Before treating a golden failure
   as a regression, check whether the stage it touches is covered by a row
   here. If it is, and the port has not yet ported that change, the failure
   is expected, not a defect — leave the golden alone and port the change
   deliberately (or don't, and say so) rather than chasing the red test.
2. **A marker at the assertion.** The moment a golden test is updated to
   assert output that *reflects* one of these changes — a deliberate
   re-anchor, not merely a gap the port hasn't closed — put a comment
   immediately above the assertion:
   ```
   // RE-BASELINE: v2.XX — RC_ENGINE_CHANGES.md §Y; DECISIONS.md §7n
   ```
   That is what makes the test self-explaining at the point of failure to
   whoever hits it next, without depending on them having read this file
   first.

A golden failure covered by neither — no row here for its version range, no
marker at the assertion — stays a regression until shown otherwise. Red still
defaults to broken.

**Verify a re-baseline claim by isolation, not by the version label.**
`RC_ENGINE_CHANGES.md` proves each of v2.59, v2.60 and v2.61 is caused by its
own named change — and nothing else — by forcing the changed flag equal on
both sides of a diff and getting byte-identical output. Do the same before
adding a row: a version bump can carry more than one change, and the label
alone does not prove which part moved the golden.

### Register: known re-baselines carried from the source engine

First instances, reopened at `RC_ENGINE_CHANGES.md` §8.1 and checked against
its own text 2026-09-20 — that section's own count is *"Eight are deliberate
re-baselines"* (v2.48, v2.50, v2.51, v2.57, v2.59, v2.60, v2.61, and v2.49
above 12 800 km), confirmed here rather than copied from a backlog row.

| Version | What moves | `RC_ENGINE_CHANGES.md` | `DECISIONS.md` |
|---|---|---|---|
| v2.48 | Plate-age distance transform becomes exact and wrap-aware (was a 3×3 chamfer) — moves `ageField`-derived roughness and `resistanceField` | §8.1 (v2.48 row) | §7n |
| v2.50 | Sub-cell crater/volcano amplitude is area-ratio-corrected against the drawn-radius floor, so a floored footprint stops inflating its depth/height budget | §8.1 (v2.50 row) | §7n |
| v2.51 | Crater depth is keyed to real diameter (Pike 1977), not to radius in grid cells | §8.1 (v2.51 row) | §7n |
| v2.57 | `PLATE_BASE_BLUR_K` 0.35 → 0.18 — the coastline stops being the level set of the raw plate polygon | §6i; §8.1 (v2.57 row) | §7n |
| v2.59 | Depression-filled routing (integrated drainage) becomes the default | §6k; §8.1 (v2.59 row) | §7n |
| v2.60 | §6l step 2c's sculpt-derived finishing descent pass (`enforceChannelDescent`/`CHANNEL_DESCENT_CENTRE_HALFW`) is the half that moves `field` — the drawn chains' climb 12.55%→4.33% at the cost of moving 0.74% of the map. (The render stamp's width floor and 4-connectivity, also §6l, are a rendering fix and move no generated value — not part of this register on their own.) | §6l; §6m.4; §8.1 (v2.60 row) | §7n |
| v2.61 | River paint moves to true-coverage alpha with banks; a river-fed pit is gated as a lake by flow, not by local rainfall; v2.60's step 2c digging pass is reverted | §6m (§6m.3–§6m.4); §8.1 (v2.61 row) | §7n |
| v2.49, above `mapWidthKm` 12 800 | The river width scale factor's shared floor stops responding to real km past this extent; `RIVER_WIDTH_MIN_K` becomes its own constant | §8.1 (v2.49 row); §7.11 | §7n |

**Explicitly not in this register:** v2.58 (river selection gains a scale
term). `RC_ENGINE_CHANGES.md` states `hash_gen1.js` vs v2.57 is all identical
there — it moves the rendered river overlay, never a generated value — so it
is a rendering contract (§6j), not a re-baseline. **Corrected 2026-09-20:**
it still changes the visible map at every scale, and `RC_ENGINE_CHANGES.md`
§8.1 files it under the changes that affect generated output — so it is not
that golden testing cannot see it at all, only that **no value a golden
fixture currently asserts can fail on it**, since none of them capture the
rendered river overlay.

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
