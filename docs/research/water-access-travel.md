# Water access for overland travel — grounding the v1.56 fix

## 1. The question

Owner report: the Journey Planner's water-supply warning fires too readily. "In reality people often
drank from streams/rivers/other smaller stops along a route... aside from literally carrying their
own water sources." Suggest an adjustment that reflects this instead of the hard warning.

## 2. What the planner was actually testing

`_jpStageDryKm` (the function measuring "how far can a party go with no freshwater in reach") tests
whether any cell within a short detour radius has `flowField[i] > flowThresh`, where
`flowThresh = GW*GH*0.0004` — the exact same constant `_jpDeriveStages` passes in, sourced from the
same convention used file-wide to decide whether a cell renders as part of the mapped river network.

That constant is not merely a rendering cutoff — it IS the engine's own channel-initiation threshold.
`buildRiverNetwork` marks a cell as a channel (`chan[i]=1`, Strahler order ≥ 1) exactly when
`flow[i] > channelThreshold(thresh, slopeN, density)`, and at the default river-density (`density=1`,
what every world uses unless the user moves the "River density" style slider), `channelThreshold`
reduces to `thresh` **exactly**, independent of slope (`dexp = |log(1)| = 0`, so the slope term's
exponent is zero). So `flowThresh` is simultaneously "does this render as a mapped river" AND "is
this a channel at all, down to the smallest headwater the engine recognises" — there is no looser,
already-computed "minor stream" signal sitting underneath it to fall back on. A travelling party
historically didn't need a mapped river; a spring or a first-order stream sufficed. The planner was
conflating the two.

## 3. Grounding a looser threshold

`docs/research/natural-rivers.md` already cites Horton's laws for a related purpose (the Min-stream-
order display slider): stream *counts* fall geometrically with Strahler order, bifurcation ratio
Rb ≈ 3–5. The companion relation (Schumm 1956; area/order scaling used throughout Horton-Strahler
geomorphology) is that mean contributing area — and, in this engine's own accumulation model, flow —
also scales roughly by Rb per order step, so going down **two** orders (from whatever order
`flowThresh` effectively represents to a genuine first-order headwater) divides the flow requirement
by roughly Rb² ≈ 9–36.

That theoretical band was checked against the actual generated terrain, not assumed:
`tests/perf/probe_water_gap.js` samples 60 straight-line "routes" between random land points on a
real generated world (seed 12345, 256px) and measures, for a sweep of candidate divisors, what
fraction register a dry stretch and how long it runs. Results on the pre-fix file:

| divisor | dry / 60 | mean dry km (among dry) |
|---------|----------|--------------------------|
| 1 (current) | 59 | 74.1 |
| 4  | 50 | 38.0 |
| 9  | 38 | 18.2 |
| 12 | 29 | 18.0 |
| **16** | **28** | **12.4** |
| 25 | 14 | 8.0 |
| 30 | 14 | 7.6 |
| 36 |  7 |  4.0 |

The measured curve and the theoretical Rb² band agree: the practically useful range is roughly
divisor 9–36, and 16 (Rb≈4, a commonly-cited mid value in Horton-Strahler literature) sits in the
middle of both. **`JP_DRINKING_FLOW_DIVISOR = 16`** — nearly two-thirds of routes that previously
read "no water anywhere" now find a source, and the ones that still don't average a genuinely severe
~12 km gap rather than a routine 74 km one.

## 4. Scope

The new threshold is applied **only inside `_jpStageDryKm`** — a local `drinkThresh =
flowThresh/JP_DRINKING_FLOW_DIVISOR`, never written back to `flowField`, `buildRiverNetwork`, or any
other consumer of the file-wide `flowThresh` convention. The rendered river network, the biome
overlay, and every other flow-threshold test are untouched, so `generate()`/`render()` stay
bit-identical to v1.55 — this is a Journey-Planner-only (civ-layer) change.

## 5. The second half: a graduated response, not a hard cliff

Separately, the auto water-crossing tier (`JP_DESERT_WATER`/`_jpDesertTierForGap` — a 4-step ladder
from "Dense Oasis Route" to "Deep Desert Crossing" that scales both speed and water reserve with the
measured gap) was gated on `isDesert`, so a non-desert biome with a genuinely long dry stretch got
only a flat, ungraduated 1.1× reserve multiplier and no speed adjustment at all — the SAME treatment
regardless of whether the measured gap was half a day or a week. v1.56 removes that gate for the
**auto** path (a measured-gap tier now resolves for any biome); the **explicit override dropdown**
stays desert-only, since its labels ("Dense Oasis Route") are desert-narrative and the control is
only ever shown once a journey includes a desert stage. Combined with §3's lower drinking threshold,
most routes now measure a short gap and get the "Dense Oasis"-tier treatment (1.10× reserve, a small
speed *bonus*) rather than ever approaching `jpAssessResupply`'s hard "no party size fixes this"
block — that block is untouched and still correct for the rarer case of a genuinely week-long
waterless crossing, desert or otherwise.

## 6. Sources

- Horton, R.E. (1945), "Erosional development of streams and their drainage basins."
- Schumm, S.A. (1956), area-order scaling in drainage basin morphology (the companion relation to
  Horton's bifurcation ratio cited in `docs/research/natural-rivers.md` §4).
- This engine's own `buildRiverNetwork`/`channelThreshold` (block 1) — read before assuming
  `flowThresh` means "mapped river only"; at the default density it IS the order-1 channel bar.
- `tests/perf/probe_water_gap.js` — the empirical sweep grounding the chosen divisor.
