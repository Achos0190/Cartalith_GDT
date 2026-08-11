# Solar Energy Budget — grounding ocean heating in planet parameters (v1.85)

## Owner report

> "For the climate modeling, now that water and wind are influenced by topology I want to make sure
> this all interconnect to map rendering aswell. (so rainfall etc should be informed by it) we also
> have sliders for gravity, axial tilt and how long days are on the world all these things inform how
> much energy a sun sets in a world and how much it keeps (eg. The heating if the ocean and flow of it
> are influenced) let's always assume a sun like star and quantify this in a simple way. All I want is
> that the heating of the ocean and resulting ocean currents and subsequent wind are all based in
> grounded values."

Two claims to check, one to build:

1. **Does the terrain-coupled wind/current work (v1.77–v1.82) actually reach map rendering, or is it
   an isolated debug-view effect?** — measurement, no code expected.
2. **Are `state.planet`'s gravity/axial-tilt/rotation sliders wired into how much solar energy the
   world receives and keeps, specifically the ocean-heating magnitude that drives currents and wind?**
   — a real gap, confirmed by direct code inspection.

## Part 1 — climate → rendering: already connected, confirmed by direct inspection

`refreshClimate()` (the master climate pipeline) calls, in order: `computeTemperature()` →
`simulateWeather(state.climate.wIters)` → `applyClimateMoistureCorrectors()` → (if `currents`)
`applyOceanCurrents()` → (if `seasons`) `computeSeasons()`.

The critical fact is that `simulateWeather` — the pass that actually produces `rainField`, the one
consumers render — calls `oceanSSTAnomaly()` **internally, before building its own wind field**:

```js
if(c.currents){ const an=oceanSSTAnomaly(WW,WH,wrapX,step);
  for(let i=0;i<N;i++) if(eh[i]<sea){ tc[i]+=an[i]; sstEvap[i]=0.2+0.8*clamp01((tc[i]+2)/30); } }
...
const wx=new Float32Array(N), wy=new Float32Array(N); buildWind(wx,wy,WW,WH,step,tc,decl||0,{elev:eh});
```

`oceanSSTAnomaly` itself calls `computeOceanCurrent` (the terrain-coupled, Ekman-rotated 2-D current
field with the v1.82 heat-driven western/eastern-boundary bend). So the real loop is:

```
terrain → computeOceanCurrent (currents) → oceanSSTAnomaly (SST) → tc (sea temp fed into wind)
        → buildWind (pressure gradient responds to tc) → moisture advection → rainField
```

This is `docs/research/system-coupling-audit.md`'s documented "Loop 2," shipped since v0.067 and
directly exercised by the v1.77–v1.82 terrain-coupling work in this same session (v1.82's own
smoke assertions already measure that `currents=true` vs `currents=false` produces a measurably
different mean-absolute SST anomaly and a different cold/warm-upwelling split on a real world).
**No code change was needed for this part** — it was already true and is now additionally reinforced
by Part 2 below, which makes the *magnitude* entering that same loop planet-parameter-aware instead
of a flat user slider.

## Part 2 — the real gap: insolation magnitude was independent of gravity/tilt/rotation

`state.planet = {g, rotationHours, axialTiltDeg, radiusRel}` already drives real physics elsewhere:

| Consumer | Formula | Since |
|---|---|---|
| `circulationCells()` | `N_c = round(3·√(Ω·R/√g))`, Ω=24/rotationHours | v0.6x (Weather v2) |
| `buildWind`'s Coriolis term | `omega=24/rotationHours` | v0.6x |
| `computeTemperature`'s altitude term | `lapseRate·g` | G1 (gravity-influence.md) |
| seasonal declination shift | `±axialTiltDeg` | W3 (v0.93 `computeSeasons`) |

But the **base equator–pole temperature gradient itself** — `tSea = poleTemp + (equatorTemp−poleTemp)
· max(0,cos(lat))`, independently reimplemented at **six call sites** (`computeTemperature`,
`oceanSSTAnomaly`, `simulateWeather`, `computeTempInto`, plus the `currentWindField`/`currentOceanField`
debug-view previews) and mirrored a seventh time in the GPU shader (`uEqT`/`uPoT` uniforms) — read
`equatorTemp`/`poleTemp` as flat user sliders, completely unconnected to gravity, tilt, or rotation.
This is exactly the shape the value that actually sets the ocean's SST baseline in `oceanSSTAnomaly`
and `simulateWeather`, so it is the correct, narrow target: "how much energy the sun sets and how much
the world keeps" is, in this codebase's own existing model, the equator–pole insolation *contrast* —
not total planetary energy budget in any absolute sense (this tool has no atmosphere-composition or
albedo-integral model to make that quantity meaningful).

## Design: two grounded multipliers on the contrast term, gravity deliberately left alone

Both multipliers are normalized to **exactly 1.0 at this file's own Earth defaults** (g=1,
rotationHours=24, axialTiltDeg=23.4°) — the CLAUDE.md invariant ("Earth defaults reproduce the
previous version bit-exactly") is satisfied by construction, not by a special-cased branch, and is
verified directly (`node tests/perf/hash_gen1.js v1.84 v1.85` → ALL IDENTICAL at defaults across every
scenario the harness runs).

### Axial tilt → `insolationContrastK()`

Grounded in the second-order (Legendre P₂) energy-balance-model approximation of annual-mean
insolation vs. latitude and obliquity (North & Coakley 1979, *"Differences between seasonal and mean
annual energy balance model calculations of climate and climate sensitivity"* — the standard EBM
textbook form). The equator–pole **contrast coefficient**

```
s2(ε) = 3·sin²ε − 2
```

is negative for Earth-like tilt (equator warmer — today's shape), **crosses zero at the critical
obliquity ε = arccos(1/√3) ≈ 54.7356°**, and is positive beyond it. That crossing is not an invented
number — it is the real, documented obliquity past which a planet's *annual-mean* insolation gradient
reverses and the poles receive more sun over a year than the equator (Rose, Cronin & Bitz 2017, *ApJ*
846:28, *"Ice Caps and Ice Belts: The Effects of Obliquity on Ice-Albedo Feedback"* — their s₂₀
coefficient's zero-crossing; the same result is discussed in the exoplanet-habitability literature,
e.g. Williams & Kasting 1997, for high-obliquity worlds).

```js
insolationContrastK(tiltDeg) = s2(tiltDeg) / s2(23.4°)
```

The file's own tilt slider (`id="ptilt"`) caps at 45°, short of the 54.7356° reversal — so across the
UI's entire reachable range this is a real, meaningful, monotonic, correctly-signed adjustment
(K≈1.31 at 0° tilt, K≈0.33 at the 45° cap), never the degenerate/reversed case, though the underlying
formula handles it correctly if the slider range is ever widened.

### Rotation (day length) → `rotationContrastK()`

A slower rotator has a weaker Coriolis constraint on its circulation, which permits more direct
(less zonally-banded) meridional overturning and therefore transports heat poleward more efficiently
— flattening the gradient. This is the qualitative, repeatedly-reported direction for slow planetary
rotators in circulation-model studies (e.g. Merlis & Schneider 2010 on the heat-transport efficiency
of weakly-rotating atmospheres; the same direction motivates `circulationCells()`'s own
"slow rotators collapse to one giant Hadley cell" behavior, already shipped).

```js
rotationContrastK(rotationHours) = (24/rotationHours)^0.25
```

Reuses the **same** Ω=24/rotationHours already driving `circulationCells()`'s cell count and
`buildWind`'s Coriolis term — not a new, unrelated constant. The exponent (0.25) is a disclosed,
order-of-magnitude choice, not independently fitted to any dataset or paper — the same class of
disclosed tuning constant as this file's `JP_MOUNT_SADDLEBAG_FRAC`, `FEATURE_RADIUS_MAX_FRAC`, etc.
Across the day-length slider's range (6–96 h): K≈1.41 at 6 h, K≈0.71 at 96 h.

### Gravity: deliberately given no additional term here

Gravity's established, defensible roles in this pipeline are already implemented — lapse rate Γ~g
(G1, `computeTemperature`'s altitude cooling term) and circulation-cell count/Coriolis strength via
`circulationCells()`'s `1/√g` term. A **further**, *direct* gravity→ocean-heating-magnitude link would
need an atmosphere-composition/pressure/greenhouse model this tool does not have to be more than a
guess — there is no equally simple, equally citable closed form the way the P₂ obliquity term or the
Ω-based rotation term have. Rather than pad the feature with an invented formula to check a box, this
is left alone and disclosed as a deliberate scope cut. Gravity's role in "how much energy the world
keeps" is, honestly, already fully expressed by the two pre-existing channels above.

### Wiring: one function, seven call sites redirected

```js
function climEffectiveEquatorTemp(){
  const c=state.climate;
  return c.poleTemp + (c.equatorTemp-c.poleTemp)*insolationContrastK()*rotationContrastK();
}
```

`poleTemp` stays the fixed anchor (unchanged from today's formula); only the *spread* above it is
scaled — exactly the term the existing formula already multiplies by `shape(lat)`. This is a disclosed
simplification, not a global-mean-preserving reformulation: at the critical 54.7356° tilt the contrast
collapses to zero and the whole world reads at `poleTemp` uniformly, rather than at some blended mean.
Building a mean-preserving version would require also re-deriving `poleTemp` itself — a larger,
unrequested redesign not needed to satisfy "quantify this in a simple way."

Every one of the six duplicate `tSea`/`tSeaAt` formulas (`computeTemperature`, `oceanSSTAnomaly`,
`simulateWeather`, `computeTempInto`, `currentWindField`, `currentOceanField`) and the GPU shader's
`uEqT` uniform now read `climEffectiveEquatorTemp()` instead of raw `state.climate.equatorTemp` — one
source of truth, avoiding this file's own repeatedly-diagnosed "two functions answering one question"
drift (the GPU shader itself needed no GLSL change: it already computes `tSea=uPoT+(uEqT-uPoT)*cos(lat)`
from its uniforms, so passing an already-contrast-scaled `uEqT` produces the identical grounded result
with a one-line JS change).

## Measured, live, end-to-end (not just correct in isolation)

A generated 256×160 region (seed 12345) at Earth defaults vs. varied tilt/rotation, `refreshClimate()`
re-run each time (`tests/perf` probe methodology, not committed — ad hoc per this file's own
"measure before calibrating" discipline):

| Config | mean tempField (°C) | equator-band (<15° lat) mean (°C) |
|---|---|---|
| defaults (23.4°, 24h) | 14.9 | 26.9 |
| tilt 0° | 29.2 | 43.6 |
| tilt 45° (slider max) | −16.2 | −9.6 |
| rotation 96h (slow) | 1.4 | 11.0 |
| rotation 6h (fast) | 34.1 | 49.2 |

All four non-default configurations diverge from the default in the predicted direction and by a
magnitude consistent with the pure-function `insolationContrastK`/`rotationContrastK` calculation done
in isolation — confirming the wiring is genuinely live through `generate()`/`refreshClimate()`, not
just correct on paper. `currents=true` vs `currents=false` at max tilt still shows the expected small
SST-anomaly perturbation on top of the (now much larger) grounded base field, confirming the existing
ocean-current toggle continues to function as a secondary term riding on the new grounded baseline.

## What this does NOT do (disclosed scope)

- No new stellar-type slider — a Sun-like (G-type) star is assumed throughout, per the owner's own
  framing ("let's always assume a sun like star").
- No absolute top-of-atmosphere energy-budget quantity (W/m², total absorbed power) is computed or
  exposed anywhere — the tool has no albedo-integral or atmosphere-composition model to make one
  physically meaningful. The grounding is specifically the **equator-pole insolation contrast**, the
  one quantity this codebase's existing temperature model actually uses.
- Gravity is not given a new direct term (see above) — a disclosed scope cut, not an oversight.
- Bit-identical at Earth defaults only; away from defaults this is a genuine, deliberate re-baseline of
  temperature/rainfall/ocean-current magnitude for any world with a non-23.4° tilt or non-24h day —
  the same class of disclosed, measured re-baseline as v1.36/v1.39/v1.46/v1.60's placement/relief
  fixes.
