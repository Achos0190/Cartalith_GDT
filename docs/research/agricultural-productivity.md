# Agricultural productivity as a slider, not a constant

## 1. The question

`FARMERS_PER_URBANITE = 9` (v1.34) is a single flat number: every world, at every tech level,
needs nine farmers to free up enough surplus for one non-farming town dweller (~90% of the
population on the land, urbanisation pinned at 10–15%). The owner's observation: a civilisation
that has "mastered things like the plow" and sits "roughly on a level of industrial production,
even so barely" should not still be capped at that ratio — 9:1 is a *pre-improvement* agrarian
number, and the whole point of the plow/rotation/steam/fertilizer lineage is that it moves.

This is correct. The real historical curve is not a step function between "medieval" and
"industrial" — it is a centuries-long slide, and by the time a society is genuinely mastering the
plow (heavy iron ploughs, systematic multi-field rotation, selective breeding) it has typically
already moved well off the 9:1 baseline, long before the first steam engine turns.

## 2. The historical curve

The cleanest long-run series is the share of the labour force in agriculture — England has the
best-attested numbers because its agricultural transformation is unusually early and unusually
well documented (Broadberry & Gardner 2013; corroborated by the Cambridge Group for the History of
Population and Social Structure, CAMPOP). Converting that share `a` to the game's own ratio
(`farmers : urbanite = a / (1-a)`) gives a directly usable ladder:

| Era | Ag. labour share | Ratio (farmers : 1 urbanite) | What changed |
|---|---|---|---|
| Pre-improvement agrarian (ancient/early medieval, ard plow, two-field, no selective breeding) | ~90% | **9 : 1** *(current baseline)* | Where `FARMERS_PER_URBANITE=9` actually sits — closer to a generic "traditional agrarian society" textbook figure than to late-medieval England specifically |
| Late medieval England, heavy plow + 3-field rotation, horse collar (the game's own cited yield source, 1250–1450) | ~75–80% | ~3.5–4 : 1 | Manorial-account yields already 470–1000 kg/ha, but labour share had not yet moved much |
| Agricultural Revolution, "mastered the plow" (Norfolk 4-course rotation, drainage, enclosure, selective breeding, iron plow) — England ~1700 | **47%** (1701) | **~0.9 : 1** | Roughly one farmer's surplus supports one additional non-farmer |
| Same, maturing — England ~1760 | 43% | ~0.75 : 1 | Plateaued for ~50 years (Overton's "Agricultural Revolution" window) |
| **Early/"barely" industrial** — steam threshing, mechanical reaper, first chemical (guano/superphosphate) fertilizer, pre-railway/pre-mass-import — England ~1800 | 35% | **~0.54 : 1** | This is the rung that matches the owner's description |
| Mature steam-era, post-1846 Corn Law repeal (imports now a real factor — see caveat below) | 28% (1851) | ~0.39 : 1 | |
| Full mechanized/chemical (post Haber–Bosch, tractors) | 11% (1911) → <2% today | ~0.12 : 1 → ~0.02 : 1 | Off the top of any pre-modern setting's plausible range |

Wheat yield tells the same story from the other side (1 bu/acre ≈ 67.3 kg/ha):

| Era | Yield | kg/ha |
|---|---|---|
| Medieval (matches the game's existing `GRAIN_YIELD_MIN/MAX_KG_HA=470–1000`) | 5–10 bu/acre | ~340–670 |
| ~1700 (post-rotation) | 19–20 bu/acre | ~1280–1345 |
| ~1800 | +6 bu/acre over 1700 | ~1750 |
| ~1860 (steam era) | +3 bu/acre over 1800 | ~1950 |

So going from "medieval" to "barely industrial" is roughly a **2.5–3× yield increase** *and* a
**roughly 10–15× swing in the farmers:urbanite ratio** — the ratio moves far more than yield alone,
because the Agricultural Revolution's other lever (animal traction, enclosure-driven labour
efficiency, fewer man-hours per hectare) is doing at least as much work as the yield gain itself.

## 3. A finding worth flagging: the current 9:1 mixes two different reference points

The engine's `GRAIN_YIELD_MIN/MAX_KG_HA` is explicitly sourced "England 1250–1450" — a relatively
productive, already-plow-equipped medieval England. But `FARMERS_PER_URBANITE=9` (~90% agrarian)
matches a *more generic, less advanced* agrarian-society figure — closer to ancient/early-medieval
Europe, or non-English medieval Europe, which lagged England's own precocious trajectory by
centuries (CAMPOP: "the shift out of agriculture in England began 200–300 years earlier than
elsewhere"). Attested English labour share for 1300–1600 is closer to **~75%** (ratio ~3:1), not
90%. This isn't necessarily wrong — a conservative pre-improvement baseline is a defensible
*default* — but it means the current single constant quietly straddles two eras rather than
representing either cleanly. Worth being explicit about when this becomes a tunable axis instead
of a hardcoded assumption.

## 4. One lever is enough for the game, not two

Real history moves both **yield/hectare** and **labour share** together, and they compound. The
engine only has one free knob that reaches the urbanisation ceiling (`FOOD_BASE_SURPLUS_RATIO`,
derived from `FARMERS_PER_URBANITE`) — `grainYieldKgHa(soil)` is a *relative* fertility curve, not
an absolute one, and rescaling it independently would require re-validating soil, carrying
capacity, and every downstream consumer of `currentAgrarianDensity`/`_civFoodShed` against a new
baseline (exactly the kind of "assumed distribution" trap v1.31/v1.34/v1.25 each got bitten by).
The clean, low-risk lever is **`FARMERS_PER_URBANITE` alone**, moved to a per-world "Agricultural
Technology" setting — it already carries the compounded historical effect (both yield *and* labour
efficiency are baked into the labour-share numbers above), it's the single quantity every
downstream function already reads through `foodSurplusRatio`, and it needs no change to
`grainYieldKgHa`, `currentSoilReference`, or the carrying-capacity chain.

## 5. Proposed calibration ladder

A discrete named ladder (matching the existing World Structure archetype convention — named
presets, each setting a continuous underlying value that stays editable) rather than a raw
0–1 slider with no historical anchor:

| Preset | `FARMERS_PER_URBANITE` | Ag. share | Flavor text |
|---|---|---|---|
| Subsistence | 19 | ~95% | Hoe/digging-stick agriculture, no plow |
| Traditional Agrarian *(current default)* | 9 | ~90% | Ard plow, minimal rotation — the existing baseline, unchanged for old saves |
| Advanced Agrarian | 4 | ~80% | Heavy plow, 3-field rotation, horse collar (late medieval) |
| Improved Agrarian — "mastered the plow" | 1 | ~50% | Multi-course rotation, drainage, enclosure, selective breeding |
| Early Industrial — "barely industrial" | 0.45 | ~31% | Steam threshing/reaper, first chemical fertilizer, pre-mass-import |
| Industrial | 0.15 | ~13% | Mechanization, synthetic nitrogen, rail/steamship grain trade |

The owner's stated setting ("mastered the plow" + "barely industrial") sits at **Improved
Agrarian → Early Industrial**, i.e. `FARMERS_PER_URBANITE` somewhere in the **0.45–1** range — a
9–20× shift from the current default, which matches the intuition that prompted this research.

## 6. Caveats

- **England's later 19th-century numbers are partly trade, not just productivity.** Post-1846 Corn
  Law repeal, Britain imported large volumes of American/Russian wheat — its 1851/1911 agricultural
  share understates what a *closed* local hinterland (which is what Cartalith's food-shed model is)
  could actually feed itself. The 1700–1800 band is cleaner for this purpose: Corn Laws were still
  protectionist, so that fall in agricultural share is overwhelmingly a genuine own-soil
  productivity story, which is why the proposed "barely industrial" rung is anchored there rather
  than at the 1851/1911 figures.
- This is a **labour-share ratio**, not a literal farmer headcount model — Cartalith doesn't track
  individual farmers, so "ratio" here means exactly what `FARMERS_PER_URBANITE` already means in
  the shipped code: the surplus fraction a hinterland cell can export before its own farmers eat it.
- Numbers are England-specific because that's where the attested series exist; presented as an
  illustrative curve, not a claim that every fantasy world's agricultural history must mirror
  England's.

## 7. Sources

- Broadberry, S. & Gardner, L. (2013), cited via [Our World in Data — Employment in
  Agriculture](https://ourworldindata.org/employment-in-agriculture): England agricultural labour
  share 47% (1701), 43% (1761), 28% (1851), 11% (1911).
- Cambridge Group for the History of Population and Social Structure (CAMPOP), ["When did Britain
  industrialise?"](https://www.campop.geog.cam.ac.uk/blog/2025/01/02/when-did-england-industrialise/)
  and the underlying *When did Britain industrialise? The sectoral distribution of the labour force
  and labour productivity in Britain, 1381–1851* (Ec. Hist. Review): ~75% agrarian 1300–1600, ≤53%
  of the male labour force by the 1750s, England "least agricultural economy in Europe" at 35% by
  1800, 42% national average 1813–1820.
- [British Agricultural Revolution](https://en.wikipedia.org/wiki/British_Agricultural_Revolution)
  and *Sources of the Change in English Wheat Yields, 1700–1860* (Overton) via ResearchGate: wheat
  yields ~5–10 bu/acre medieval, ~19–20 bu/acre by 1700, +6 bu/acre 1700–1800, +3 bu/acre
  1800–1860.
- Existing in-file citation: `GRAIN_YIELD_MIN/MAX_KG_HA=470–1000` (England 1250–1450, manorial
  accounts) and `FARMERS_PER_URBANITE=9` (v1.34, `docs/research/food-logistics.md` §6) — the
  baseline this document extends, not replaces.
