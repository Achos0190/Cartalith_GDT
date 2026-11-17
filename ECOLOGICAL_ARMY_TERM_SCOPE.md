# Ecological army term — scope

**What this is.** A scope for one defect in the military manpower model (`MILITARY_MANPOWER_SCOPE.md`,
`cartalith-civ/src/manpower.rs`). A faction whose land feeds more people than live on it pays a **standing army
larger than the field army** it can concentrate, so the model's own ordering `standing < field < levy` breaks.
This document reproduces the failing readings, names the factions that fail and says why, reviews how
pre-modern armies were actually bounded by food, foraging and supply, and proposes candidate terms. Each one is
measured on the 20 FH-R2 harness worlds. It also gives the re-baseline plan under `DECISIONS.md` §7p, milestones
with numeric bars, and owner questions with recommended defaults.

**Why it exists.** G1-1's bar 4 (`GOVERNMENTS_SCOPE.md` §8) asked for `standing < field < levy` in every forced
reading. It did not hold, and not because of G1's rows: the reference's own forms break it on the same worlds.
`g1_forced_forms_are_ordered_and_near_their_anchor` (`cartalith-godot/src/g1_baseline.rs`) reports the count.
Its doc names the cause as missing an ecological term in `field = levy x (0.34 + 0.20 x logistics)`, and it
asserts a weaker one-directional bar instead.

**What it is not.** It is not a status record and it carries no status column. Progress lives in
`cartalith-native/docs/STATUS.md`. The backlog row is in `OUTSTANDING_WORK.md` §2.3. The owner's answer to §8 is
Ruling DE (`LARGE_ITEM_RULINGS.md`). The pass that wrote this document made no code change. The diagnostic below ran in a throwaway worktree and was then discarded.

**Method, and what was opened (2026-10-25, tree `61fc1585` = `origin/main`).** Every number is one of four kinds:

- **measured**: produced in this pass by the command in §2;
- **replica**: computed from the measured rows by re-applying a candidate formula to each row's own drivers. This
  is exact, because each candidate changes one product of the drivers the dump records (§2.3);
- **cited**: the source is named in §4;
- **judgement**: labelled as such.

| Opened | Symbols |
|---|---|
| `cartalith-civ/src/manpower.rs` | `military_drivers`, `manpower_with_levy_bonus` (chains 1-3), `civ_military_manpower_world`, `world_land_reference`, `SOLDIER_UPKEEP_BY_BRACKET`, `ALPHA_BRACKETS`, `ECOLOGICAL_FLOOR`/`_CEILING` (and their doc's ruling-11 measurement), `LEVY_BASE`/`_STATE`/`_LOGISTICS`, `GOVERNMENT_LEVY_BONUS`, `FIELD_BASE`/`_LOGISTICS`, `GOVERNMENT_EXTRACTION`, `era_for`, `ERA_BANDS`; tests `kingdom_b`, `worked_example_kingdom_b`, `the_ecological_ceiling_binds_where_the_ruling_put_it`, `the_factor_still_discriminates_above_the_old_ceiling`, `the_citizen_ruling_moves_no_headcount`, `an_independent_replica_reproduces_both_worked_examples` |
| `cartalith-civ/src/roster.rs` | `AG_TECH_LEVELS` (`farmers_per_urbanite` 19, 9, 4, 1, 0.45, 0.15) |
| `cartalith-godot/src/` | `g1_baseline.rs` (whole), `civ_military_bridge.rs` (`MilWorld::manpower_rows_of`'s `ManpowerInput` build, `resolve_institutions`), `fh_r2_baseline.rs` (grepped: no manpower column), `conflict_bridge.rs` (grepped: forwards `standing_army`/`field_army`) |
| Consumers (grepped for `standing_army\|field_army`) | `conflict.rs`, `garrison.rs`, `conflict_bridge.rs`; shell `faction_roster_window.gd`, `right_dock.gd`, `civilization_workspace.gd`; probes `_manpower_probe.gd`, `_ecoclamp_probe.gd`, `_mpscale_probe.gd`, `_mm8garrison_probe.gd`, `_sp4conflict_probe.gd` |
| Documents | `CLAUDE.md`, `MISTAKES.md` preflight, `MILITARY_MANPOWER_SCOPE.md` §1, §2.3-§2.4, §3.2a, §3.3 finding 3; `GOVERNMENTS_SCOPE.md` §8-§9; `LARGE_ITEM_RULINGS.md` Ruling CZ; `DECISIONS.md` §7p |

**Not opened:** `campaign.rs`, `garrison.rs` beyond its module doc, and the probe bodies beyond
`_manpower_probe.gd`'s header and its ordering check. **No probe was run in this pass.**

---

## 1. The formulas, at their symbols

`manpower_with_levy_bonus` computes every output for one faction (`civ_military_manpower_world` first rescales
`land_capacity` by the world's own land per person). Write `E` for `ecological_factor`, `α` for
`agricultural_labour_ratio`, `s` for `state_capacity`, `L` for `logistics_capacity` and `u` for
`soldier_upkeep(α)`. Every output is divided by total population:

```
standing / pop = (1 − α) · E · (0.04 + 0.12·s) / u                         chain 1, fiscal
levy     / pop = 0.25 · clamp(0.04 + 0.30·s + 0.22·L [+ khanate 0.1886], 0, 0.60)   chain 2
field    / pop = levy/pop · (0.34 + 0.20·L)                                 chain 2's tail
```

`E` appears **only in chain 1**. It is the faction's land per head relative to the world's, clamped to
0.25..4.0 (`ECOLOGICAL_FLOOR`/`_CEILING`). `MILITARY_MANPOWER_SCOPE.md` §3.3 finding 3 already established by
control flow that the levy, field, ladder and durations "never touch `ecological`". The standing army is linear in
`E`. The field army is independent of it. So for any fixed `α, s, L` there is a crossover

```
E* = [0.25 · levy_reach · (0.34 + 0.20·L)] / [(1 − α)·(0.04 + 0.12·s) / u]
```

above which `standing > field`, whatever the government.

**Crossover by ag-tech row** (replica, capital reach 0.5, from the formulas above):

| Row (`farmers_per_urbanite`) | α | `u` | E* chiefdom, L 0.45 / 0.70 | E* monarchy | E* empire |
|---|---|---|---|---|---|
| subsistence (19) | 0.95 | 1.1200 | 7.89 / 11.75 | 7.35 / 10.29 | 7.09 / 9.60 |
| **traditionalAgrarian (9)**, every harness world | **0.90** | **0.7983** | **2.81 / 4.17** | **2.61 / 3.64** | **2.52 / 3.39** |
| advancedAgrarian (4) | 0.80 | 1.8153 | 3.18 / 4.70 | 2.95 / 4.09 | 2.84 / 3.81 |
| improvedAgrarian (1) | 0.50 | 2.2214 | 1.54 / 2.25 | 1.42 / 1.93 | 1.37 / 1.80 |
| earlyIndustrial (0.45) | 0.31 | 1.7696 | 0.88 / 1.28 | 0.81 / 1.09 | 0.78 / 1.02 |
| industrial (0.15) | 0.13 | 1.0714 | 0.42 / 0.61 | 0.39 / 0.52 | 0.37 / 0.48 |

Two things follow.

- **The harness worlds sit in the worst pre-modern bracket.** `α` 0.90 is `ALPHA_BRACKETS` bracket 1, whose
  upkeep (0.7983, "land-grant and self-equipped levies") is the lowest in `SOLDIER_UPKEEP_BY_BRACKET`. The
  cheapest soldier is multiplied by a budget up to 4x larger. Below `E*` ≈ 2.5 the ordering holds.
- **On the two industrial rows the crossover is below 1.** The ordering fails *without* any land abundance. That
  is a second, separate defect (§3.4), and no ecological term can fix it.

---

## 2. Reproduction: which factions break the ordering, and why

### 2.1 The committed harness, re-run

Command (alone, worktree `C:/cw/ec1` at `origin/main` `61fc1585`, its own `CARGO_TARGET_DIR`):

```
cargo test -p cartalith-godot --lib g1_baseline -j 2 -- --ignored --nocapture --test-threads 1
```

**Measured** (3 passed, 48.6 s for the three tests including the diagnostic):

- `g1_0_baseline_columns`: the positive control moved manpower on 20 of 20 worlds and governments on 20 of 20.
  Against `G1_0`, 0 of 20 worlds moved, and the roster agreed with `fh_r2_baseline` on 20 of 20.
- `g1_forced_forms_are_ordered_and_near_their_anchor`: **160 forced readings (8 G1 forms x 20 worlds), ordered
  125**. The reference control: **160 readings (8 real forms x 20), ordered 120**.

**Reconciling the brief's "90 of 120".** That was G1-1's first run: six core forms x 20 worlds, before G1-5. The
six core forms now read **95 of 120** ordered. All six failed on the same 5 worlds (30 failures, so 90 ordered),
and G1-5's khanate levy bonus orders the khanate on those 5 (25 failures, so 95). The 8-form count is 7 forms x 5
worlds = 35 failures (so 125). The khanate appears in no failure.

### 2.2 Exactly which readings fail

**Forced readings (faction 1, every form): the same 5 worlds, every form on each.**

| World (seed, km, factions) | faction 1's `E` | Reference forms ordered | G1 forms ordered |
|---|---|---|---|
| 314159, 40 075, 6 | **4.000** | 0 of 8 (standing/field 1.38-1.57) | 1 of 8 (khanate) |
| 314159, 40 075, 24 | **3.288** | 0 of 8 (1.19-1.33) | 1 of 8 |
| 404, 800, 24 | **4.000** | 0 of 8 (1.45-1.62) | 1 of 8 |
| 707, 40 075, 6 | **3.257** | 0 of 8 (1.17-1.32) | 1 of 8 |
| 707, 40 075, 24 | **4.000** | 0 of 8 (1.45-1.62) | 1 of 8 |
| the other 15 worlds | 0.250 … 2.359 | 8 of 8 | 8 of 8 |

So the government barely matters (chiefdom fails with the rest). The split is `E` alone: every failing faction-1
reading has `E` ≥ 3.257, and every ordered one has `E` ≤ 2.359.

**Default governments, every real faction (the world as generated):** **59 of 290** non-empty factions break the
ordering, on **17 of 20** worlds (all but 404/800/6, 707/800/6 and 314159/800/6). By shape:

| Shape | failing / factions |
|---|---|
| 800 km, 6 factions | 2 / 30 |
| 40 075 km, 6 factions | 5 / 30 |
| 800 km, 24 factions | 29 / 120 |
| 40 075 km, 24 factions | 23 / 110 |

**What every one of the 59 has in common (measured):**

- `α` = 0.900 and `u` = 0.7983, as on every harness faction. The generated worlds use one ag-tech row.
- `E` from 2.606 to 4.000, median 4.000. 51 of the 290 factions sit at the 4.0 ceiling.
- Standing/field from 1.01 to 1.62.
- **Every one already reads `above` its era's standing band.** The standing army is 2.2-5.2 % of total
  population against Iron and Bronze Age bands of 1-2.5 % and 0.5-2 % of citizens. By era: Iron Age agrarian 40,
  Bronze Age 15, Classical 4.
- Every government appears: theocracy 11, oligarchy 10, tribal confederacy 8, chiefdom 7, monarchy 7,
  republic 7, city-state 5, empire 4.
- Field < levy holds on all 59. Standing < levy holds on all 610 readings of §2.1-§2.2 (largest standing/levy
  0.698).

**Ten factions are empty, not violators.** Factions 23 and 24 on each 40 075 km 24-faction world have zero
population and read 0 / 0 / 0. `0 < 0` is false, so a bar that does not exclude them counts them as
"unordered". The harness's "22 armed of 24" comments are these factions. Any ordering bar must count them
separately. It must not pass them as ordered and must not fail them (`MISTAKES.md`: never encode "no value" as
a plausible value).

**Why.** The standing army is paid from `non_agricultural x E x extraction`. The non-agricultural population is
the *realised* surplus, but `E` is the land's *potential*. The constants' own doc says so: *"paying an army out
of unfarmed potential is not a fiscal quantity"*. A faction with four times the world's land per head is credited
with four times the revenue per head. Its levy and field army, meanwhile, are the same as on poor land. The
violators are the factions where that credit is largest. Their standing armies are the ones above the era band.
The field army is not too small: it is the standing army that is too large.

### 2.3 The diagnostic (not committed, reproducible)

Two `#[ignore]`d tests were appended to `g1_baseline.rs` in the worktree only, then discarded. EA-0 (§6) commits
their equivalent.

1. **`eco_diag_dump`.** For each of the 20 worlds it prints one row per real faction under its default
   government. It also prints faction 1 forced to each of the 16 forms. Each row carries total population, `α`,
   `E`, extraction, `u`, `s`, `L`, standing, field, levy, citizen fraction, the era and its band (610 rows).
2. **`eco_diag_agtech`.** On the same worlds it forces **every** faction to each of the six `AG_TECH_LEVELS`
   rows in turn (1 800 rows). `E` is world-relative, so it is unchanged when all factions share a row. That makes
   the sweep a clean test of `α` at the measured geography.

The candidates in §5 were scored by re-applying each formula to those rows' own drivers. Each candidate replaces
one factor of `standing` or `field`. That factor is known exactly from the row (`E`, and `field/levy` =
`0.34 + 0.20·L`), so the replica is exact rather than an approximation. EA-1's bar re-measures it on the built
code anyway.

---

## 3. What the problem is, and what it is not

1. **It is a chain-1 problem, not a chain-2 one.** All 59 violators read above their standing band, and none
   reads a field army below anything. The brief's framing ("no ecological term in the field army") names the
   symptom's location, not the cause. A term on the field army (§5, C2) also restores the ordering, but it
   leaves all 59 above their bands.
2. **It is not a G1 problem.** The reference forms fail on the same five worlds. G1's one-directional guard
   (`same_as_anchor`) is correct as written.
3. **The probe has not seen it.** `_manpower_probe.gd` asserts *"standing < field < levy everywhere"* on its own
   generated world. `MILITARY_MANPOWER_SCOPE.md` §3.3 records it passing after ruling 11, when its world's `E`
   ranged 0.438-2.912, below every crossover in §1. One world is one sample (`MISTAKES.md`, "Report a layout
   measurement"). The probe was not run in this pass.
4. **Industrial rows fail for a different reason.** This is replica plus sweep, measured on forced rows that no
   generated world uses today. On `earlyIndustrial` and `industrial`, the ordering fails on 12 and 49 of the
   176 factions with `E` ≤ 1. Standing exceeds the **levy itself** on 75 and 120 of 290. The levy chain is capped
   at `0.60 x 0.25` = 15 % of population, while `ERA_BANDS` gives the industrial eras 15-50 % mobilisation. The
   cause is a missing industrial term in chain 2, not ecology. **It is out of this scope** and is offered as its
   own row (Q6).

---

## 4. How pre-modern armies were bounded by food: the evidence

Tags: **[verified]** means the bibliographic details were confirmed by a web search in this pass.
**[cited]** means the source is named from the literature and its details were not re-fetched in this pass.
Treat [cited] page-level claims as needing a check before they enter a doc comment.

**4.1 Fiscal capacity reads realised surplus, not land.** This bounds the standing army.

- **Domar, E. D. (1970), "The causes of slavery or serfdom: a hypothesis", *Journal of Economic History* 30(1),
  18-32.** [cited] Where free land is abundant relative to labour, land earns no rent. A ruler can extract a
  surplus only by binding labour, not by holding land. More land per head does not raise the extractable surplus.
  This is the strongest argument that `E` above 1 should not scale the budget.
- **Boserup, E. (1965), *The Conditions of Agricultural Growth*, Allen & Unwin.** [cited] Under land abundance
  output is labour-limited. Intensification, and with it surplus per area, comes from population pressure, not
  from spare land.
- **Hopkins, K. (1980), "Taxes and trade in the Roman Empire (200 B.C.-A.D. 400)", *Journal of Roman Studies*
  70, 101-125.** [cited] Roman taxation, and the army it paid, were a small share of a realised output, about a
  tenth. It was not a share of potential.
- The model's own constant doc (`ECOLOGICAL_FLOOR`, ruling 11) already makes this argument against a higher
  ceiling. §5's C1 applies it to the term itself.

**4.2 Food and forage bound concentration.** This bounds the field army.

- **Perjés, G. (1970), "Army provisioning, logistics and strategy in the second half of the 17th century",
  *Acta Historica Academiae Scientiarum Hungaricae* 16** (reprinted in J. Black (ed.), *Warfare in Europe
  1650-1792*, Ashgate 2005). [verified] Large armies could not be fed from what was procurable on the spot, so
  magazines had to be stocked in advance. Contemporary experts (Montecuccoli, Zrínyi) put a practical ceiling
  near 48 000-50 000 men on one concentrated army.
- **Engels, D. W. (1978), *Alexander the Great and the Logistics of the Macedonian Army*, University of
  California Press.** [verified via secondary sources] About 3 lb of grain and half a gallon of water per man per
  day, and far more grain and forage per animal. An army could carry only days of supply, so its size and route
  were set by the harvest and the density of the land it crossed.
- **van Creveld, M. (1977), *Supplying War: Logistics from Wallenstein to Patton*, Cambridge University Press.**
  [cited] Before railways, armies lived off the country while moving. How large a force a region could carry
  depended on its population density and agricultural surplus.
- **Lynn, J. A. (ed.) (1993), *Feeding Mars: Logistics in Western Warfare from the Middle Ages to the Present*,
  Westview.** [cited] The "tyranny of forage": fodder for horses outweighed rations for men and could not be
  carried, so the green forage of the theatre bounded mounted forces.
- **Roth, J. P. (1999), *The Logistics of the Roman Army at War (264 B.C.-A.D. 235)*, Brill**, and **Erdkamp, P.
  (1998), *Hunger and the Sword*, Gieben.** [cited] Roman field forces were sized and timed against the local
  harvest and allied supply. Foraging capacity depended on the region's surplus.
- **Sinor, D. (1972), "Horse and pasture in Inner Asian history", *Oriens Extremus* 19, 171-184.** [verified]
  Pasture bounded the horse herds, and so the steppe armies, that could be sustained outside the steppe. This is
  the case for a symmetric forage term, in which thin land limits concentration.

**4.3 A standing army above a single field army is normal, which bears on the bar itself.**

- **Tacitus, *Annals* 4.5** [cited]: 25 legions in AD 23, dispersed along every frontier, plus auxilia of
  comparable strength. The owner's specification quotes about 250 000 regulars under Tiberius. No single campaign
  army approached that.
- **Parker, G. (1988), *The Military Revolution*, Cambridge University Press.** [cited] Most of an early-modern
  standing army sat in garrisons, not in the field army.

So `standing < field` is **not a historical law** for large states. The model's "field army" is drawn from the
levy alone (chain 2), not from the standing force. Where the ordering fails here, though, the standing army also
fails its own era band (§2.2). That points at chain 1, so the ordering is a useful alarm on this model even
though it is not a universal truth. C3 below takes the other reading.

---

## 5. Candidate terms, measured on the 20 FH-R2 worlds

All figures are **replica** on the measured rows (§2.3), traditionalAgrarian as generated, unless the row says
otherwise. "Moved" counts factions whose standing or field changes. "Verdicts" are standing-band verdicts out
of the 290 non-empty factions: within / above / below.

### C1: the budget reads realised surplus (chain 1)

```
E_budget = E                  if E ≤ 1        (unchanged by control flow)
         = E^γ                if E > 1
military_budget = non_agricultural · E_budget · fiscal_extraction_efficiency
```

Shortage stays linear: land that cannot feed its people does cut the surplus. Abundance gets diminishing returns
(§4.1). **γ = 0** is Domar's reading (abundant land adds nothing to what can be taxed). **γ = 0.5** is a
labelled judgement, halfway on a log scale, that keeps ruling 11's discrimination above 1.

| | ordered (default govs) | verdicts within / above / below | forced G1 (160) | forced reference (160) | moved |
|---|---|---|---|---|---|
| current | 231 / 290 | 55 / 92 / 143 | 125 | 120 | — |
| **C1, γ = 0** | **290 / 290** | **135 / 12 / 143** | **160** | **160** | 114 (every `E` > 1), standing x0.25-1.00 |
| C1, γ = 0.5 | 290 / 290 | 81 / 66 / 143 | 160 | 160 | 114, standing x0.50-1.00 |

**Every ag-tech row (the `eco_diag_agtech` sweep, ordered / 290, within the band):**

| γ | subsistence | traditional | advanced | improved | early industrial | industrial |
|---|---|---|---|---|---|---|
| current (γ = 1) | 290, 202 | 231, 55 | 235, 85 | 201, 70 | 165 | 127 |
| 0.5 | 290, 226 | 290, 81 | 290, 117 | **224**, 105 | 166 | 127 |
| 0.25 | 290, 252 | 290, 106 | 290, 151 | 268, 124 | — | — |
| **0** | **290, 276** | **290, 135** | **290, 170** | **290, 157** | 177 | 127 |

**C1 clears all 610 traditionalAgrarian readings for any γ ≤ 0.60**: the violators need `1 − γ ≥ max
ln(standing/field) / ln E` = 0.398. At γ = 0.65, 609 of 610 clear. γ = 0.5 therefore has a thin margin on
today's worlds and fails 66 readings on `improvedAgrarian`. Only γ = 0 clears all four
pre-industrial rows. **C1 at γ = 0 is the measured best on both bars**, the ordering and the era band, on every
pre-industrial row.

**The cost of γ = 0:** above `E` = 1, geography no longer moves the standing army. That reverses the standing
half of ruling 11's purpose, which `the_factor_still_discriminates_above_the_old_ceiling` protects (2.5 against
3.5 must differ). After ruling AI (b) normalised `E` to the world, `E` > 1 means "more land per head than the
world average", and C1 at γ = 0 says that extra land is not extra revenue. The geography still shows in
`food_surplus_per_farmer`, which reads `E` unchanged. C2 is the way to keep it in the military outputs.

### C2: forage lets abundant land feed a larger concentration (chain 2's tail)

```
field = levy · min(FIELD_CAP, (0.34 + 0.20·L) · max(E, 1)^β),   β = 0.5,  FIELD_CAP = 0.90
```

This puts §4.2 into the field army. `FIELD_CAP` 0.90 (judgement) keeps some of the levy at home, so field <
levy by construction of the cap, not by assertion. One-sided (`max(E, 1)`) so that `E` ≤ 1 factions are
bit-identical by control flow.

| | ordered | verdicts | forced G1 / reference | moved | cap binds |
|---|---|---|---|---|---|
| C2 alone | 290 / 290 (all four pre-industrial rows 290; improved 223) | **unchanged, 55 / 92 / 143** | 160 / 160 | 114, field x1.00-2.00 | 6-9 per row |
| symmetric variant, `E^β` | 290 / 290 | unchanged | 160 / 160 | **290**, field x0.50-2.00 | 6 of 610 |

C2 alone fixes the symptom but **leaves all 59 violators above their standing band**. That is §3's point 1. It
also moves both field durations and the force ladder (their inputs are `field`), which C1 does not.

**C1 (γ = 0) + C2, measured together:** 290 / 290 on subsistence, traditional, advanced and improved; 259 and
131 on the two industrial rows; 114 factions moved; verdicts as C1. This split follows the literature: abundance
does not raise taxes (§4.1) but does feed armies on the march (§4.2). Ruling 11's "geography above 1 must
discriminate" survives in the field army, where §4.2 says it belongs.

### C3: restate the bar, change no formula

`standing < levy` and `field < levy`, with empty factions excluded and counted. **Measured 610 of 610 hold
today. 0 factions move.** This follows §4.3. It leaves the 59 above-band standing armies, and the `ERA_BANDS`
verdicts already report those as findings (*"reported, never enforced"*). It is the honest option if the owner
reads the ordering as an invariant the model never promised.

### Kingdom A and B (the specification's worked examples, `MILITARY_MANPOWER_SCOPE.md` §3.1)

Kingdom A has `E` = 1.000 exactly, so it is **unchanged under every candidate** by control flow (standing 9 661,
levy 41 221, field 15 870). Kingdom B has `E` = 1.05 (land 1 050 000 for a population of 1 000 000, single-faction path):

| | standing (pinned 25 750) | field (pinned 47 368) | specification |
|---|---|---|---|
| C1, γ = 0 | 25 750 / 1.05 ≈ **24 524** | unchanged | ~20 000; 40 000-60 000 |
| C1, γ = 0.5 | 25 750 / √1.05 ≈ **25 129** | unchanged | |
| C2 | unchanged | 47 368 · √1.05 ≈ **48 538** | |

All stay inside the specification's ranges. These are replica; EA-1 measures them.

### What does not move, by control flow (projection, EA-1 confirms)

- `SOLDIER_UPKEEP_BY_BRACKET`'s derivation and `GOVERNMENT_LEVY_BONUS`'s derivation are both taken at `E` = 1, so
  both take the unchanged `E` ≤ 1 branch of C1 and C2 and are untouched.
- `E` itself, `food_surplus_per_farmer`, the levy, the citizen fraction and `era_for`.
- C1 also leaves the field army, durations and ladder unchanged.

---

## 6. Re-baseline plan (`DECISIONS.md` §7p)

The manpower model is port-only: `MILITARY_MANPOWER_SCOPE.md` says the reference has no model, so there is no
JS golden and no parity switch to keep. Under §7p the chosen term **replaces** the linear one. The linear budget
is not kept as an option, and the prose describing it is scrubbed in the same change. **What moves, and the
record each needs:**

| Pin or surface | Moves under C1 / C2 | Record |
|---|---|---|
| `g1_baseline.rs` `G1_0` manpower column | 20 of 20 worlds (each has at least one `E` > 1 faction, measured) | re-recorded under the ruling, old hash beside new, `RE-BASELINED <date> under Ruling <x>` |
| `G1_0` governments column | 0 of 20 (control flow) | none; must stay 0 |
| `worked_example_kingdom_b`, `the_citizen_ruling_moves_no_headcount`, `an_independent_replica_reproduces_both_worked_examples` | B's standing (C1) or field (C2) | literal re-baselined, old beside new; the replica gains the term independently (not by calling the code) |
| `the_factor_still_discriminates_above_the_old_ceiling` | C1 γ = 0: ratio 1.4 → 1.0 (its purpose ends); γ = 0.5: → √1.4 = 1.1832 | γ = 0 + C2: rewritten to assert the **field** ratio √1.4 = 1.1832 (literal), with its doc saying why the standing half moved |
| `the_ecological_ceiling_binds_where_the_ruling_put_it` | no (it pins `E`, not the budget) | none |
| `g1_forced_forms_are_ordered_and_near_their_anchor` | ordered becomes n | tightened to `ordered == n` with empty factions excluded and counted; the one-directional guard stays |
| `_manpower_probe.gd`, `_ecoclamp_probe.gd`, `_mpscale_probe.gd`, `_mm8garrison_probe.gd`, `_sp4conflict_probe.gd` | printed figures for `E` > 1 factions | re-run after `cargo build` (the `.dll`, `MISTAKES.md`); any literal re-recorded with old beside new |
| Garrisons (MM-6), manpower over time (MM-8), conflict readings | follow the standing army | no pin found by grep; MM-8's probe is the check |
| Prose | `manpower.rs` module and constant docs (chain 1, `ECOLOGICAL_FLOOR`'s "linear" paragraph, `MilitaryDrivers::ecological_factor`), `MILITARY_MANPOWER_SCOPE.md` §2.3-§2.4 and §3.3 finding 3, `g1_baseline.rs`'s test doc, `GOVERNMENTS_SCOPE.md` §8 G1-1 bar 4 note, `STATUS.md`, `DECISIONS.md` §7p dated line | same change (`MISTAKES.md`, "Change behaviour") |

`fh_r2_baseline.rs` has no manpower column (grepped) and does not move. No `cartalith-civ/tests/` golden reads
manpower (grepped).

---

## 7. Milestones

Each ships alone, after the owner rules. `cargo test --workspace` (the result-line count checked against the
floor before any total is read) and a parse check of every touched `.gd` plus `shell/app.gd` run after each.
Every new function, constant and test is annotated for an auditor (Ruling BK). Each constant names its source,
and γ, β and `FIELD_CAP` are labelled judgements unless the owner rules a value.

| ID | What | Bars (numeric, each measured) | Depends on |
|---|---|---|---|
| **EA-0** Measure | Commit §2.3's two diagnostics as one ignored harness test in `g1_baseline.rs`. It counts ordered / unordered / **empty** per world and per ag-tech row, and reports per-faction `E`, `α` and standing/field for the unordered | (1) Reproduces §2.2 exactly: 231 of 290 ordered, 59 unordered, 10 empty; forced 125 / 160 and 120 / 160; the 5 named worlds. (2) Ag-tech sweep reproduces §5's current row (290, 231, 235, 201, 165, 127). (3) Positive control: `ECOLOGICAL_CEILING` mutated to 2.0 in a scratch run (Python mutation discipline, restored in a `finally`) moves the unordered count, measured, and restores to 59. (4) 0 of 20 worlds move on `G1_0` | owner ruling |
| **EA-1** The term | C1 at the ruled γ (default 0) in `manpower_with_levy_bonus`, with an `E` ≤ 1 branch that is the old expression by control flow. If Q2 = yes, C2 with `FIELD_CAP` as a named constant | (1) Ordered 290 of 290 (default govs), 160 of 160 forced G1, 160 of 160 forced reference, empty factions 10 excluded and counted. (2) Moved exactly the 114 factions with `E` > 1; the other 176 bit-identical. (3) Kingdom A 9 661 / 41 221 / 15 870 unchanged; Kingdom B at §5's replica value ±1. (4) `soldier_upkeep_is_derived_from_the_era_table` and `the_khanate_levy_bonus_is_derived_from_its_era_band` green with no literal changed. (5) Standing verdicts within ≥ 135 of 290 at γ = 0 (≥ 81 at γ = 0.5). (6) Mutations killed: γ → 1 (harness red), the `E` ≤ 1 branch removed (176 factions move, red), `FIELD_CAP` → 1.0 (field = levy possible, red) | EA-0; Q1, Q2 |
| **EA-2** Re-baseline and prose | §6's table: `G1_0` re-recorded old beside new; pins re-baselined; probes re-run after `cargo build` with `.dll` and `.rs` mtimes stated at the end; prose scrubbed; `STATUS.md` and a `DECISIONS.md` §7p dated line | (1) `G1_0` governments 0 of 20 moved, manpower 20 of 20 re-recorded. (2) Every probe in §6 PASS, run after the rebuild. (3) `git grep` for the old chain-1 wording (`non_agricultural × ecological_factor`, "linear in this factor") returns only historical records | EA-1 |
| EA-3 Industrial levy | §3 point 4: chain 2 has no industrial mobilisation term. Measured first on 2026-10-25 (§10); the owner chose K2 at R = 0.80 (Ruling DL), and what was built and measured is §10.7 | Its own ruling (Q6), then Ruling DL | — |

Model tier is per lane when scheduled. EA-0 and EA-2 are routine for a strong builder. A separate verifier should
re-run EA-1's mutations.

---

## 8. Owner questions, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| **Q1** | Which chain-1 term: C1 at γ = 0 (abundant land adds no revenue; measured best on the ordering and the band, every pre-industrial row), C1 at γ = 0.5 (keeps standing discrimination above 1; 224 of 290 on `improvedAgrarian`), or none (C3)? | **C1, γ = 0** |
| **Q2** | Add C2 (forage term on the field army, one-sided, β = 0.5, `FIELD_CAP` 0.90), so that geography above 1 still moves a military output? | **Yes, with C1 γ = 0.** It keeps ruling 11's purpose in the field army, where §4.2 puts it. Without C1, no: C2 alone leaves the 59 above their band |
| **Q3** | Keep `E` ≤ 1 linear (a short land cuts the budget in proportion)? The symmetric variant (`√E` both sides) raises the 95 factions at the 0.25 floor x2 | **Yes, keep it linear.** Whether the 0.25 floor now decides rather than guards (95 of 290 at it, measured) is ruling 11's question from the other end, and it is a separate one |
| **Q4** | Keep `standing < field < levy` as a model invariant asserted on the harness, with empty factions excluded and counted? Or adopt C3's weaker bar, given §4.3 (a dispersed standing army above one field army is historically normal)? | **Keep it, on the pre-industrial rows only.** With C1 it holds 290 of 290 on all four. The industrial rows fail for §3 point 4's reason |
| **Q5** | Accept the re-baseline in §6 (`G1_0` manpower 20 of 20, Kingdom B, the discrimination test rewritten) under §7p, with no switch for the old budget? | **Yes**, old values beside new |
| **Q6** | File §3 point 4 (on the industrial rows, standing exceeds the levy because chain 2 has no industrial term) as its own row? | **Yes**, as its own scope. Not built here, and no generated world reaches those rows today |

---

## 9. Risks

- **Fitting the exponent to today's worlds.** γ ≤ 0.60 is the measured bound on one ag-tech row. Choosing γ just
  under 0.6 would pass today's 610 readings and fail the next row (§5's sweep). γ = 0 is a rule (Domar), not a
  fit. γ = 0.5 is a judgement and must be labelled as one.
- **Asserting the ordering against itself.** A `field = max(field, standing)` style fix makes the bar
  tautological (`MISTAKES.md`, "never assert a constant against itself") and is not proposed. C2's cap makes
  field < levy structural, so EA-1's bar for it is the mutation (`FIELD_CAP` → 1.0), not the inequality.
- **One sample of geography.** The 20 worlds share one generator, and every faction is on one ag-tech row. The
  sweep in §2.3 varies `α` at fixed geography, not the reverse.
- **Cross-lane drift.** `manpower.rs` and `g1_baseline.rs` were edited three times on 2026-10-25 (G1-1, G1-4,
  G1-5). Re-open every symbol at EA-0's start (`MISTAKES.md`, "re-open a backlog row at its symbol").

---

## 10. The industrial levy (§3 point 4, Ruling DE Q6): measured first (2026-10-25), then ruled (Ruling DL, §10.7)

**What this section is.** The measure-first pass on the `OUTSTANDING_WORK.md` §2.3 row *"Military manpower: no
industrial mobilisation term in the levy chain"*. It reproduces the failure, says exactly why the industrial rows
fail, reviews how industrial mobilisation differed, and scores three candidate terms. **No candidate met the
bars, so nothing was built and nothing was tuned.** The owner questions are in §10.6, and the owner answered them as Ruling DL (§10.7). Status lives in `STATUS.md`.

**Method.** Tree `3da18424` (`origin/main`), worktree `C:/cw/in1`, its own target dir. Re-opened at the symbols:
`manpower.rs` `manpower_with_levy_bonus` (chains 1-2), `military_drivers`, `era_for`, `ERA_BANDS`,
`SOLDIER_UPKEEP_BY_BRACKET`, `ALPHA_BRACKETS`, `MILITARY_AGE_FRACTION`, `LEVY_*`, `FIELD_*`; `g1_baseline.rs`
`ea0_ecological_ordering_diagnostic`, `g1_0_baseline_columns`. **Measured** numbers come from
`cargo test -p cartalith-godot --lib ea0_ -j 2 -- --ignored --nocapture --test-threads 1`, run alone. For the run, a
temporary line (since removed) printed each sweep row's drivers: `α`, `E`, `s`, `L`, citizen fraction, population,
the three headcounts, the era and both verdicts, 1 800 rows in all. **Replica** numbers re-apply a formula to those
rows. The replica reproduces the dumped standing, field and levy to a relative 4.2e-6, which is the dump's
4-decimal rounding.

### 10.1 The baseline moved before this pass began

`ea0_ecological_ordering_diagnostic` and `g1_0_baseline_columns` were both **red on `origin/main`**. The ordering
bars held: 290 / 0 / 10 with default governments, and 160 and 160 forced. What failed was the geography-bound
literals:

- standing verdicts: 132 / 10 / 148, against the pinned 138 / 9 / 143;
- `E > 1`: 109 factions, against 112;
- the cap binding: 21, against 18;
- the sweep: `[290, 290, 290, 290, 264, 129]`, against `[290, 290, 290, 289, 264, 127]`;
- `G1_0`: manpower moved on 20 of 20 worlds, governments on 0 of 20.

**Bisected** on `G1_0`'s manpower column:

| Tree | Worlds moved (of 20) |
|---|---|
| `5cfa2997^` | 0 |
| `7877dfa8` | 0 |
| `5cfa2997` (Ruling DG, lava domes on in the app) | 12 |
| `1153bbee` (Ruling DJ, calderas, maars and the supervolcano on) | 20 |

No manpower formula moved. This pass re-recorded both tests, with the old values beside the new and the bisection
in their docs.

**The `improvedAgrarian` miss (289 of 290) went with it.** The faction (314159/800/24, faction 21, a republic) read
`E` 0.973 and logistics at the 0.150 floor on `50a6fc6d`. On today's worlds it reads `E` 0.294 and is ordered
(standing 116, field 367, levy 991).

Its old cause had nothing to do with industry:

- `α` 0.50 is bracket 3, whose upkeep is 2.2214.
- `E` just under 1 takes neither forage arm.
- At the logistics floor, the field share is 0.37 of a levy at a reach near 0.25. So the field army was about 2.3 %
  of population, against a standing army of about 2.4 %.

That is a chain-1 against chain-2 balance at the logistics floor, in a pre-industrial bracket. Neither an
industrial term (bracket 4 and above, by control flow) nor Ruling DE's terms (`E` > 1 only) can reach it. It may
recur on another world. It is reported as its own class (§10.6 Q3), not fixed.

### 10.2 Why the industrial rows fail (measured)

All 300 default-government rows on the 20 worlds read `α` 0.900 (`traditionalAgrarian`). **No generated world uses
an industrial row**, so the industrial rows appear only in the forced sweep, which forces every faction to one row:

| Row | ordered | fails as standing ≥ field (< levy) | fails as standing ≥ levy | standing verdict within / above / below | mobilization verdict |
|---|---|---|---|---|---|
| `earlyIndustrial` (α 0.31, bracket 4) | **264 / 290** | 26 | 0 | 142 / 10 / 138 | **below on 290** |
| `industrial` (α 0.13, bracket 5) | **129 / 290** | 65 | **96** | 111 / 28 / 151 | **below on 290** |

**The cause is in chain 2.** Chain 1 grows with `1 − α`, and chain 2 does not read `α` at all:

- **Standing** (chain 1), at `α` 0.13: `0.87 · E_b · (0.04 + 0.12 s) / 1.0714` of population, which is 3.2 % + 9.7 % · `s`.
  The largest measured is 12.1 %.
- **Levy** (chain 2): `0.25 · min(0.60, 0.04 + 0.30 s + 0.22 L)`. That is at most **15 % of population, whatever the
  society**.
- **Field**: at most 0.54 of the levy (0.90 with forage).

So once the standing army passes 15 % of population it exceeds the levy itself. It overtakes the field army well
before that.

**A second, structural finding: the `Total industrial mobilization` band cannot be reached by construction.**

- The levy is at most `MILITARY_AGE_FRACTION` × reach, so at most 0.25 × 1.0 of total population.
- The citizen fraction is at most `CITIZEN_CEILING`, 0.98.
- Mobilization is therefore at most 25.5 % of citizens.

That band asks for 30-50 %, and `Railway / industrial mass army` for 20-40 %. No levy term inside the military-age
pool can put a faction within it. Every candidate below leaves the `industrial` row **below on 290 of 290**.

### 10.3 How industrial mobilisation differed: the evidence

Tags as in §4. **[verified]** means confirmed by a web search in this pass. **[cited]** means named from the
literature, with details not re-fetched in this pass.

- **Conscription with a trained reserve reached most of the cohort.**
  - France mobilised 8.3-8.4 million men in 1914-18 [verified] (INED, *"Farewell life, farewell love"*,
    demographic note). That is about 21 % of a 39.6 million population.
  - A table in Brill's encyclopedia of the First World War gives France **81 % of men fit for duty** mobilised
    [verified as quoted; the denominator was not confirmed].
  - Germany's peacetime army was about 0.70-0.84 million. Behind it stood about 3 million trained reservists
    (1.3 million first line, 1.7 million second line), plus the Ersatz reserve [verified, secondary sources;
    their figures disagree].
  - The mechanism is the reserve system. Every conscript served two or three years, then spent most of his military
    age in the reserve and the Landwehr. So the share of the cohort a state can call is set by how universal service
    is, not by how far the state's roads reach.
- **Development drives mobilisation.** Broadberry, S. and Harrison, M. (eds) (2005), *The Economics of World
  War I*, Cambridge University Press [verified]. Their summary argues that mobilisation was explained largely by
  each country's level of development [verified as an abstract-level claim]. That is the case for a term in
  `1 − α` rather than in the government.
- **Railways moved armies to the railhead, not beyond it** [cited]: van Creveld (1977), *Supplying War*, chs. 3-4;
  Showalter, D. (1976), *Railroads and Rifles*, Archon. Railway concentration solved deployment and the feeding of
  a front. Beyond the railhead, supply went back to horse and foot. So a railway term on the field share rests on
  weaker evidence than the reserve term.
- **The field share stayed near half in the railway era.** In 1914, Germany's field armies were 1.6-2.1 million of
  the mobilised total [verified, secondary sources]. That is roughly the 0.34-0.54 this model already gives. It
  argues **against** a railway term that pushes the field share toward `FIELD_CAP`.
- **Industrial output and demography, not food, bound the mass army.** The era table's own constraint column says
  so (*"Industrial logistics"*, *"Industrial capacity / demographics"*) [the owner's specification]. The
  military-age pool stays near a quarter of the population, because the 15-49 male cohort does not grow with
  industry. That is why §10.2's band is out of reach.

### 10.4 Candidate terms (replica on the 1 800 sweep rows)

**The shared gate.** All three candidates use an industrialisation index,
`i = clamp((0.45 − α) / (0.45 − 0.10), 0, 1)`:

- Its endpoints are `ALPHA_BRACKETS[3]` and `ALPHA_BRACKETS[5]`. The linear ramp between them is a labelled
  judgement.
- It reads 0 for `α` ≥ 0.45, 0.40 on `earlyIndustrial` and 0.914 on `industrial`.
- **The term applies only where `alpha_bracket(α) ≥ 4`.** So the four pre-industrial rows, every generated world,
  and Kingdoms A (α 0.75) and B (α 0.55) stay bit-identical by control flow.

Ordered counts are out of 290, on subsistence / traditional / advanced / improved / early industrial / industrial.

| Candidate | Formula | ordered | worst standing/field on `industrial` |
|---|---|---|---|
| current | none | 290 / 290 / 290 / 290 / 264 / **129** | — |
| **K1** conscription reach, R = 0.80 (cited) | `reach' = reach + i·(R − reach)` | 290 / 290 / 290 / 290 / **290** / **274** | 1.345 |
| K1, R = 0.70 / 0.90 / 1.00 | same | … / 290 / 261, 282, 288 | 1.53 / 1.20 / 1.09 |
| **K2** K1 plus railway logistics in the field share, R = 0.80 | adds `fs = 0.34 + 0.20·(L + i·(1 − L))` | 290 / 290 / 290 / 290 / 290 / **287** | 1.137 |
| K2, R = 0.70 / 0.90 / **1.00** | same | … / 290 / 279, 289, **290** | 1.29 / 1.016 / 0.919 |
| K2 with the field share driven to `FIELD_CAP` (0.90), R = 0.80 | `fs' = fs + i·(0.90 − fs)` | … / 290 / 290 | — |
| **K3** reserve multiplier, `Y` = 2.5 / 3.5 / 4.75, capped at R = 0.80 | `levy' = min(pool·R, levy + i·standing·(Y − 1))` | … / 290 / 278 (any `Y`) | — |

**The three failures K2 leaves at R = 0.80 are all empires at state capacity 0.91:** 12345/40075/24 f7,
314159/800/24 f15 and 707/800/24 f23. Their standing armies are 10.8-12.1 % of population (11.0-12.4 % of
citizens), against the band's 5-10 %, so all three read **above** their own standing band. That is a chain-1
excess at the top of the state-capacity range, the same shape as §2.2. A levy term cannot cure it without calling
more of the pool than the cited figure supports.

**Why none is built:**

- **K1** at the cited R fails 16 of 290 on `industrial`.
- **K2** at the cited R fails 3.
- **K2 at R = 1.00** orders all 290, but only by calling the *whole* military-age pool, above the roughly 80 %
  that §10.3 cites, and with a margin of only 0.919. Choosing R to clear the bar is the fit §9 warns against.
- **K2 driven to `FIELD_CAP`** orders all 290, but it contradicts §10.3's evidence that the field share stayed
  near half.
- **K3** fails 12 whatever `Y` is. It also ties the levy to the standing army, so any bar it passes would be half
  tautological (`MISTAKES.md`, "never assert a constant against itself").

The forced-form readings on the industrial rows were not measured, because the default-government bar already
fails.

**No candidate reaches the band.** At R = 0.80, `earlyIndustrial` reads within its mobilisation band on 26 of 290
and below on 264. `industrial` reads below on 290 of 290 under every candidate (§10.2).

### 10.5 What a build would move

**On the 20 harness worlds, nothing, by control flow:** all 300 rows read `α` 0.900 (measured).

**The positive control for that claim is the forced sweep.** K1 and K2 move every non-empty `earlyIndustrial` and
`industrial` row, because `i` > 0 on both.

A build would re-record the `ea0_` sweep literals for the two industrial rows, old beside new. Windowed probes
would move only where a user sets a faction's ag-tech to an industrial row.

### 10.6 Owner questions, each with a recommended default

**Answered by Ruling DL (2026-10-26):** Q1 K2 at R = 0.80, industrial brackets only, the ordering asserted on the pre-industrial rows and reported on the industrial ones; Q2 yes, a measure-first row; Q3 yes, guard the case (prevent, or detect by a test and handle in the model); Q4 restate the bands (not the recommended "accept below"). §10.7 says what was done with each.

| # | Question | Recommended default |
|---|---|---|
| **Q1** | Which industrial levy term, if any? Options: K2 at the cited R = 0.80 (`earlyIndustrial` 290, `industrial` 287 of 290; the 3 misses are above-band empires); K2 at R = 1.00 (290 of 290, but it calls the whole military-age pool); K1 alone (274); or none until a generated world uses an industrial row | **K2 at R = 0.80.** Keep the ordering asserted on the pre-industrial rows only (Ruling DE Q4, as ruled) and *report* it on the industrial rows. File the 3 misses with Q2 |
| **Q2** | At the top of the state-capacity range on the industrial row, chain 1 pays a standing army of 11-12 % of population, above the 5-10 % band. Should chain 1 get its own industrial treatment (as Ruling DE gave `E`), or is that band reading accepted? | **File it as its own row, measured first.** Do not fold it into the levy term |
| **Q3** | The `improvedAgrarian` miss of `50a6fc6d` (logistics floor, `E` just under 1, bracket 3's high upkeep) has gone from today's worlds but can recur. Pin it as a reported class, or add a guard? | **Report only.** `ea0_`'s `EA sweep unordered` print already names any recurrence |
| **Q4** | `Total industrial mobilization` (30-50 %) and `Railway / industrial mass army` (20-40 %) cannot be reached with a military-age pool of 0.25 and a citizen ceiling of 0.98: the most is 25.5 %. Restate those bands (as shares of men, or as totals over a war), widen the pool for total war, or accept "below" as a known reading? | **Accept "below" and document it.** The pool is demography, and France's 8.4 million over four years was about 21 % of the population, inside the model's ceiling. The band appears to use a different denominator, which is the owner's to restate |

### 10.7 Ruling DL: what was built and measured (2026-10-26)

Tree `4bebe582` (`origin/main`), worktree `C:/cw/in2`, its own target dir. Re-opened at the symbols: `manpower.rs` `manpower_with_levy_bonus`, `era_for`, `ERA_BANDS`, `CITIZEN_MODERNISATION`'s doc; `g1_baseline.rs` `ea0_ecological_ordering_diagnostic`, `g1_0_baseline_columns`, `g1_forced_forms_are_ordered_and_near_their_anchor`. **Measured** numbers come from `cargo test -p cartalith-godot --lib g1_baseline -j 2 -- --ignored --nocapture --test-threads 1` (run alone) and the unit tests named below.

**Q1, the term.** `CONSCRIPTION_REACH` (0.80, cited) and `industrialisation(α)`, which is `None` below bracket 4 (no term, by control flow) and otherwise the §10.4 ramp. In chain 2, `reach' = reach + i·(0.80 − reach)`, and the field share reads `L + i·(1 − L)` in place of `L`. Measured:

| | before | after |
|---|---|---|
| sweep ordered: subsistence, traditional, advanced, improved | 290 / 290 / 290 / 290 | 290 / 290 / 290 / 290 (hashes bit-identical) |
| sweep ordered: `earlyIndustrial` / `industrial` | 264 / 129 | **290 / 287** (the replica's figures exactly) |
| default governments; forced G1; forced reference | 290 / 0 / 10; 160; 160 | unchanged |
| `G1_0` (the app's own reading, 20 worlds) | — | governments 0 of 20 moved, manpower 0 of 20 |
| Kingdom A / B | — | unchanged (their worked-example tests pass with no literal changed) |

The positive control is the per-row manpower hash the harness now prints. The four pre-industrial rows' hashes are the ones the tree before the term printed, and the two industrial ones moved.

The three `industrial` exceptions are the empires §10.4 named (12345/40075/24 f7, 314159/800/24 f15, 707/800/24 f23). Each is at state capacity 0.910, with a standing army of 11.0-12.4 % of citizens, so each is **above** its standing band. They are Q2's row.

One existing unit pin moved, as the ruling intends. At Kingdom A's institutions the industrial levy went 50 455 → 186 996, from 1.29x the traditional levy to 4.80x. That is still under the standing army's 8.27x. It is re-baselined, with the old value beside the new, in `standing_army_rises_with_industrialisation_at_equal_population`.

**Q2, the measure-first row (not built).** The row is in `OUTSTANDING_WORK.md` §2.3. What it should measure first:

- Chain 1 on bracket 5 is `(1 − α)·E_b·(0.04 + 0.12 s)/u`, with `u` 1.0714. So at `s` 0.91 the standing army is about 12 % of population, against the `Total industrial mobilization` standing band of 5-10 % (Q4 did not change it).
- The driver-space test below finds 1 003 unordered `industrial` readings: 737 empires and 176 stratocracies among them, all at `E` 0.85-1.45.
- Two readings to tell apart before any term. Does `SOLDIER_UPKEEP_BY_BRACKET`'s bracket-5 value, derived at median state capacity, under-price a conscript army at high state capacity? Or is `EXTRACTION_CEILING` too high for an industrial fiscal base? Either would move standing armies on the industrial rows only.

**Q3, the guard.** It was measured first over the whole input space (`the_ordering_holds_across_the_driver_space_but_one_class`, `manpower.rs`). The grid is six rows x 17 governments x capital reach 0..1 by 0.25 x road density 0..1 by 0.1 x navigable and sea access each off and on x `E` 0.25..4.0 by 0.05: 284 240 readings per row.

- The ordering fails on **0** readings on `subsistence`, `traditionalAgrarian` and `advancedAgrarian`.
- It fails on **853** on `improvedAgrarian`. Every one has standing ≥ field, at logistics below 0.20 (largest 0.195) and `E` 0.80-1.55.

So the `50a6fc6d` case is a class, not one faction: bracket 3's upkeep of 2.2214 against a levy and a field share that both fall with logistics. Its cause is in chain 2. On the harness sweep, the same row's levy reads **below** its own mobilization band on 244 of 290 factions.

A term that forces the ordering there would be a cap on the standing army, which asserts the bar against itself (§9). So none was built. Instead:

- The model classifies every reading (`Manpower::ordering`, `ManpowerOrder`: ordered, standing not below field, field not below levy, empty). It names the case wherever it occurs, as it already names an era band a faction falls outside.
- The driver-space test pins where the case can occur: 853 readings, logistics under 0.20, and among the pre-industrial rows `improvedAgrarian` only. A change that widens it fails.
- The harness asserts that the case never occurs on the 20 generated worlds (Ruling DE Q4). It also checks, on every reading, that the model's classification agrees with the harness's own inequality.

Prevention is an open question (§10.8).

**Q4, the bands.** The table has one denominator, the citizen body. On the industrial rows that is the whole population up to `CITIZEN_CEILING`, because the citizen fraction reaches the ceiling there for every government.

Against it, the owner's figures read as shares of **men**:

- France mobilised about 8.4 million of about 39.6 million: about 21 % of its people and about 43 % of its men.
- Germany mobilised about 13.2 million of about 65 million: about 20 % and about 41 %.

Both cells are restated by **halving** them, since men are about half of a population (a labelled judgement):

| Row | was (of men) | now (of citizens) | anchors |
|---|---|---|---|
| `Railway / industrial mass army` | 20-40 % | **10-20 %** | the same conversion. `era_for` never returns this row (bracket 5 is `Total` or `Modern mechanized`); it is kept so the table stays the owner's |
| `Total industrial mobilization` | 30-50 % | **15-25 %** | France about 21 % and Germany about 20 % of population, 1914-18 |

Measured: `industrial`'s mobilization verdicts went from 0 within / 0 above / 290 below to **290 / 0 / 0**. A median industrial monarchy reads 19.268 % of citizens, inside the band (`the_restated_industrial_bands_are_reachable`).

The shell reads every band through the bridge's `era_mobilization_lo` and `_hi`, so the tooltips print the restated figures with no `.gd` change. No probe was run:

- no generated world uses an industrial row;
- `G1_0` moved on 0 of 20 worlds;
- no probe forces an industrial row (grepped).

**Mutations** were run in Python: exact literal, restored in a `finally`, file hash identical afterwards. Every one was killed:

- `CONSCRIPTION_REACH` 0.80 → 0.70;
- the bracket gate `< 4` → `< 3`;
- the railway field logistics removed;
- the conscription reach removed;
- the ramp's far end `[5]` → `[4]`;
- `Total` 0.15-0.25 → 0.30-0.50;
- `Railway` 0.10-0.20 → 0.20-0.40;
- the classifier's standing arm returning `Ordered`;
- two one-digit harness literals (the industrial ordered count and a sweep hash).

### 10.8 Owner questions left by Ruling DL

| # | Question | Recommended default |
|---|---|---|
| **Q7** | `Early industrial` mobilization (15-30 %) was not restated. If the owner's table uses the male denominator throughout its industrial rows, it would be 7.5-15 % of citizens. Restate it the same way? Its readings under the term today: 26 within, 264 below, of 290 | **Ask before restating.** It is reachable as stated, and halving it would also need the shell's `%.0f` band format widened to show 7.5 |
| **Q8** | Prevent Q3's class rather than name it? The measured cause is that bracket 3's levy reads below its own band (244 of 290). A bracket-3 levy term would move `improvedAgrarian` readings, and Kingdom B | **Measure first, as its own row**, as for Q2 |


### 10.9 The industrial standing army above its band: measured first, nothing built (2026-11-17)

**Task and rule.** `OUTSTANDING_WORK.md` §2.3's row *Military manpower: on the industrial rows, a high-state-capacity standing army above its own band* (Ruling DL Q2: measure first, do not build). This section is the measurement. **No product code changed**; the scratch code is ignored and pins nothing. Tree `7d7cead5` (`origin/main`), worktree `C:/cw/army`, its own target dir. Opened at the symbols first: `manpower.rs` `manpower_with_levy_bonus` (chain 1: `standing = non_agricultural * budget_ecology * fiscal_extraction_efficiency / soldier_upkeep`), `SOLDIER_UPKEEP_BY_BRACKET` (bracket 5 = 1.0714), `soldier_upkeep_is_derived_from_the_era_table`, `EXTRACTION_FLOOR` and `EXTRACTION_CEILING` (0.04 and 0.16), `military_drivers` (state capacity clamped to 0.03..0.95; `citizen_fraction` 0.98 on this row), `era_for`, `the_ordering_holds_across_the_driver_space_but_one_class`; `g1_baseline.rs` `ea0_ecological_ordering_diagnostic`. Every number below was produced this session by the commands under "Evidence"; numbers from elsewhere are cited.

#### 10.9.1 Today's reading (290 factions, every faction forced to `industrial`)

| | at `7d7cead5` | the row's 2026-11-15 text |
|---|---|---|
| `industrial` ordered, of 290 (10 empty factions apart) | **288** | 286 |
| `earlyIndustrial` ordered | 290 | |
| pre-industrial rows | 290 each (the `ea0_` test passes) | |
| `industrial` standing verdicts, within / above / below the era's standing band | **98 / 28 / 164** | |
| median standing, of citizens | **3.53 %** | |
| generated factions (300) whose default ag-tech is on bracket 5 or 6 | **0** (all 300 are bracket 1, `traditionalAgrarian`) | "no generated world uses the row" |

The two leaves (`EA industrial unordered`, `ea0_run.log`):

| leaf | government | E | state capacity | logistics | standing, of citizens | standing / field |
|---|---|---|---|---|---|---|
| 404/800/24 f21 | republic | 1.002 | 0.715 | 0.450 | **10.42 %** (band 5-10 %) | 1.0103 |
| 24601/800/24 f8 | city state | 1.023 | 0.715 | 0.450 | **10.42 %** | 1.0000 |

So the row's older picture (empires at 0.910) is gone. Both leaves are at state capacity 0.715 (0.55 x full reach x 1.30 urbanisation), at `E` just over 1, where the budget factor is already saturated (`E <= 1` arm) and the forage term on the field army (`E^0.5`, `E > 1` only) has barely started. They miss the ordering by 0 to 1 %.

**Over the driver space** (the shipped test's 284 240-reading grid on the `industrial` row, re-run read-only): **1 003 unordered readings**, the pinned figure, so the model has not moved. By government: empire 737, stratocracy 176, republic 30, city state 30, merchant republic 30. They begin at state capacity **0.692**, `E` 0.85-1.45, logistics up to 1.000. The ordering never fails on `field >= levy` (0 readings).

**On the sweep, 28 of 290 `industrial` readings stand above the band and 164 stand below it.** The above-band ones have median state capacity 0.715; the below-band ones have median `E` **0.250**, the ecological floor, where chain 1's budget is linear in `E` (Ruling DE C1 kept the linear arm below 1). Whatever is done to the high-capacity tail, most generated industrial factions have the opposite problem. That is a separate finding (Q10).

#### 10.9.2 Telling apart the two causes §10.7 names

The formula is standing proportional to `fee(s) / upkeep`, with `fee(s) = 0.04 + (ceiling - 0.04) * s`. Field army and levy read neither factor, so an ablation re-prices the standing army **analytically** from the model's own drivers. The control: chain 1 reconstructed from the printed drivers equals the model's standing on all 290 rows (`analyse.py`; a re-price that cannot reproduce the baseline proves nothing). The driver-space version applies the same ratio to the shipped model's readings. No shipped constant was touched.

**Cause (b), "`EXTRACTION_CEILING` is too high for an industrial fiscal base": yes in effect, but it is not "binding".** A lower ceiling on the industrial brackets does clear the grid (with the upkeep re-derived, at 0.08; table below). But the ceiling is never reached: the highest fee on the 290 is **0.1492**, against 0.16, because state capacity tops out at 0.910 on this row (empire, full capital reach; the clamp is 0.95). It acts as a parameter of the fee's slope: the fee spans 0.053-0.149 (2.8x) over the upkeep derivation set's state capacities 0.107-0.910, while the band (5-10 %) is 2x wide.

**Cause (a), "bracket 5's upkeep, derived at median state capacity, under-prices a conscript army at high state capacity": yes in effect, but as a flat level it is not wrong.** An upkeep that rises with the fee (C3) clears the grid. The flat value is centred on its derivation set by construction: the geometric mean over its set (the reference's eight governments but `none` x capital reach 0..1 by 0.1, `E` 1, logistics 0.5) is at the band centre 7.5 % (geometric-mean fee 0.09056, `driver_space_run.log`), and the full grid's median standing is 6.87 %, inside the band. So a *level* correction (C1) has nothing to correct on average and moves the below-band majority further down. What is wrong is that the upkeep is flat over state capacity.

**Isolated ablations** (each alone; `driver_space_run.log` for the grid, `analyse_out.txt` for the sweep). "DS" is the 284 240-reading grid; its verdicts are for era-13 readings only.

| change, alone | sweep unordered | sweep within / above / below | DS unordered | DS median / max standing | DS era-13 within / above / below |
|---|---|---|---|---|---|
| baseline | 2 | 98 / 28 / 164 | 1 003 | 6.87 % / 12.36 % | 202 572 / 20 720 / 59 732 |
| upkeep 1.20 | 0 | 96 / 7 / 187 | 152 | 6.13 % / 11.03 % | 192 612 / 4 428 / 85 984 |
| upkeep 1.30 | 0 | 68 / 7 / 215 | 0 | 5.66 % / 10.18 % | 178 396 / 2 196 / 102 432 |
| upkeep 1.324 (10 % at s 0.91) | 0 | 73 / 1 / 216 | 0 | 5.56 % / 10.00 % | 177 124 / 0 / 105 900 |
| ceiling 0.14 | 0 | 96 / 7 / 187 | 99 | 6.27 % / 10.85 % | 201 104 / 4 428 / 77 492 |
| ceiling 0.131 | not run | not run | 0 | 6.01 % / 10.17 % | 193 488 / 2 196 / 87 340 |
| ceiling 0.128 (10 % at s 0.91) | 0 | 99 / 1 / 190 | 0 | 5.92 % / 9.94 % | 192 040 / 0 / 90 984 |

Where each first clears (isolated, nothing re-derived): the **sweep** needs only upkeep 1.083 (+1.1 %) or ceiling 0.158; the **grid** needs upkeep 1.300 (+21 %; 1.295 leaves 2) or ceiling 0.131 (0.132 leaves 3). Both single changes also move every faction on the row, and both push the below-band count up (164 to 216 on the sweep at the exact upkeep; to 190 at the exact ceiling).

**The finding.** The two readings cannot be told apart by this data, and each alone is sufficient. They are one lever, `fee(s) / upkeep(s)`: the best slope-side change of each kind clears the grid (C2 and C3 below), and their grid results differ by 44 of 284 240 readings. The only reading the data rules out is a *flat level* change of the upkeep (C1), which worsens the band; and a ceiling change alone breaks the derivation test (see next). Isolated, an upkeep change is a level change and a ceiling change is a slope change, but **any ceiling change breaks `soldier_upkeep_is_derived_from_the_era_table` unless the upkeep is re-derived at it** (the rule's share-at-upkeep-1 contains the fee), and re-deriving cancels most of the level effect. With the upkeep re-derived to hold the geometric mean at the band centre, the ceiling must fall to **0.08** (twice the floor) before the grid clears:

| ceiling, upkeep re-derived | upkeep | DS unordered | DS median / max | DS era-13 within / above / below |
|---|---|---|---|---|
| 0.15 | 1.0229 | 803 | 6.88 % / 12.15 % | 202 616 / 20 720 / 59 688 |
| 0.12 | 0.8767 | 312 | 6.94 % / 11.42 % | 226 984 / 7 184 / 48 856 |
| 0.10 | 0.7782 | 85 | 7.01 % / 10.79 % | 243 608 / 4 428 / 34 988 |
| 0.09 | 0.7285 | 18 | 7.05 % / 10.41 % | 246 156 / 2 196 / 34 672 |
| **0.08** | **0.6785** | **0** | 7.10 % / 9.99 % | 248 580 / 0 / 34 444 |
| 0.04 (fee flat in s) | 0.4732 | 0 | 7.50 % / 7.50 % | 249 508 / 0 / 33 516 |

So the attribution: **the leaves belong to the fee's spread over state capacity, which the flat upkeep (a) does not follow and the ceiling (b) sets.** Either reading fixes it; neither is a wrong level, and the ceiling is not reached. The field army reads state capacity only through the levy reach (`LEVY_STATE` 0.30) and never through the fee, so standing rises faster in `s` than field does; the grid's unordered readings begin at `s` 0.692 (measured).

#### 10.9.3 Three candidates, scored

All three are brackets 5 and 6 only (`alpha < 0.25`; `SOLDIER_UPKEEP_BY_BRACKET[6]` repeats `[5]`), so `earlyIndustrial` and every pre-industrial row are bit-identical by control flow, as Ruling DL did. Numbers are from the tables above and `analyse_out.txt`; "sweep" is the 290-faction industrial sweep.

| | C1: upkeep 1.0714 to **1.33** (a level change: band top at the highest reachable capacity) | C2: industrial ceiling **0.08** with the upkeep re-derived to **0.6785** (a slope change, rule-consistent) | C3: upkeep **scaled by the fee**, `u0 * (fee(s) / 0.09056)^0.5` (a slope change; the derivation set's geometric mean preserved exactly) |
|---|---|---|---|
| sweep leaves | 0 (from 2) | 0 | 0 |
| sweep within / above / below (baseline 98 / 28 / 164) | 73 / 1 / 216 | 123 / 1 / 166 | 121 / 1 / 168 |
| sweep median standing (baseline 3.53 %) | 2.85 % | 3.40 % | 3.40 % |
| DS unordered (baseline 1 003) | 0 | 0 | 0 |
| DS median / max standing | 5.53 % / 9.95 % | 7.10 % / 9.99 % | 7.18 % / 9.63 % |
| DS era-13 within / above / below (baseline 202 572 / 20 720 / 59 732) | 174 176 / 0 / 108 848 | 248 580 / 0 / 34 444 | 248 624 / 0 / 34 400 |
| smaller change that fails | 1.295 leaves 2 on the grid | ceiling 0.09 leaves 18 | exponent 0.25 leaves 117; 0.5, 0.75 and 1.0 clear |
| what else moves | `SOLDIER_UPKEEP_BY_BRACKET[5]` and `[6]`; **breaks `soldier_upkeep_is_derived_from_the_era_table`** (its rule centres the mean polity, this centres the top), so the rule must be restated; the values literal test; the industrial hash and verdict counts in `ea0_`; the 1 003 pin in the driver-space test; `standing_army_rises_with_industrialisation_at_equal_population`'s 8.27x ratio | a new bracket-gated ceiling constant read in `military_drivers`; bracket 5's upkeep re-derived (the derivation test needs the ceiling as an input); the same pins as C1 | `military_drivers` gains one expression for `soldier_upkeep` on brackets 5 and 6; the derivation test passes for bracket 5 because the geometric mean is preserved by construction (it should assert that, not the constant); the same pins as C1 |
| `fh_r2` / `g1_baseline` / Kingdom A and B | **unmoved at the default `civ.base_ag_tech` (1)**: 0 of 300 generated factions are on bracket 5 or 6; both worked examples are pre-industrial. A world generated with `civ.base_ag_tech` 4 or 5 (the parameter exists; `lib.rs` `base_ag_tech_seeds_the_generated_roster_and_moves_no_settlement` sets all six levels but reads no manpower, and no baseline sets it) would read the new values | same | same |
| standing-army outputs that move | `standing_army`, `professional_core` (standing x professionalization), the standing shares and verdict. **Not** field, levy or the durations: `campaign_capability` reads state capacity, logistics and professionalization, none of which read the fee or the upkeep | same | same |
| needs a §7p switch | **no.** The manpower model is port-only, (`MILITARY_MANPOWER_SCOPE.md`: the reference has no manpower model, checked by unit tests and a live probe, not golden parity; §6 above and Q5), so under §7p the new reading replaces the old, old value beside new, as Ruling DE Q5 and Ruling DL did | no | no |

A fourth reading was scored and **rejected**: a global `EXTRACTION_CEILING` change. Every row's standing army is `fee(s) / upkeep`, so it moves every faction on all six rows (arithmetic, not run: at `s` 0.5, a ceiling of 0.128 gives a fee of 0.084 against 0.100, a factor 0.84), which re-records `G1_0` on 20 of 20 worlds, both Kingdoms and every pre-industrial hash, for 2 leaves.

Not covered by any candidate: the one `Modern mechanized` reading on the sweep (state capacity at least 0.8 and logistics at least 0.8 lands in the 0.5-3 % standing band, a different table row from the 5-10 % one, so it reads "above" at any level). That is a band-edge discontinuity in `era_for`, not this row (Q15).

**Verdict.** All three clear the sweep and the grid, none touches a world at the default base technology, none needs a switch. They differ in collateral: C1 is the only one that worsens the band (grid below-band 59 732 to 108 848) and the only one that breaks the derivation rule; C2 and C3 both lower the below-band count (to 34 444 and 34 400) and raise the grid median to 7.1-7.2 %. C2 and C3 are the same lever (C3 at exponent 0.5 is the fee to the power 0.5, C2 at ceiling 0.08 is a fee line with a third of the slope; their grid "within" counts differ by 44 of 284 240); C3 is one expression with one labelled exponent, C2 adds a constant and a bracket branch. **Recommended: C3.** **The 0.08 of C2 and the 0.5 of C3 are fitted to the bar, not sourced**: each is the smallest scanned value that clears the grid (C3: between 0.25 and 0.5), so they are labelled judgements whose only support is this measurement. §10.7 Q3 declined a term whose job was to force the ordering; the owner should read Q9 with that in mind. What does carry independent support is the shape (a state that collects more per head pays its soldier more), not the number.

### 10.10 Owner questions from §10.9, each with a recommended default

| # | Question | Recommended default |
|---|---|---|
| **Q9** | Build a fix at all, given 2 leaves of 290, both within 1 % of the ordering, on a row no generated world uses? | **Yes, C3, small.** The cost is one expression and a re-record of the industrial row's pins; no generated output moves. If the owner would rather wait, the row stays measured and costs nothing |
| **Q10** | The industrial sweep's larger problem is the other tail: 164 of 290 standing armies read below the band (median `E` 0.25, the floor, with the budget linear in `E` below 1). Open a separate measure-first row for it? | **Yes, its own row, measured first.** Candidates would be a floor on the budget factor or a sub-linear arm below 1, which also moves the pre-industrial rows and so is a Ruling DE question. Not folded into Q9 |
| **Q11** | Which of §10.7's two readings should the model adopt, given that the data cannot separate them (each alone clears; 44 of 284 240 readings apart)? (a) the upkeep follows the fee (C3), or (b) a lower industrial ceiling with the upkeep re-derived (C2) | **(a), C3.** It keeps the derivation rule's mean exactly and adds no constant; and note the ceiling is never reached (highest fee 0.1492), so (b) is a slope parameter, not a binding limit |
| **Q12** | C3's exponent: 0.5 (a labelled judgement), or another value? 0.25 leaves 117 grid readings; 0.5, 0.75 and 1.0 clear. 1.0 makes the standing army a constant 7.5 % of citizens, which makes the band a definition rather than a result | **0.5**, the smallest round value that clears; not 1.0 |
| **Q13** | Apply to bracket 6 (`alpha < 0.10`, which repeats bracket 5's upkeep) as well | **Yes**, so the existing `[6] == [5]` test keeps its meaning |
| **Q14** | After the fix, assert `standing < field < levy` on all six rows (today the two industrial rows are only reported) | **Yes**: the sweep reads 290 of 290 and the grid 0 of 284 240 on the industrial row (the pre-industrial rows already hold) |
| **Q15** | The `Modern mechanized` standing band (0.5-3 %) sits below the 5-10 % one it follows at `s >= 0.8` and logistics `>= 0.8`. Restate it, or keep the owner's table as written? | **Keep as written** and report it; it is the owner's table (`ERA_BANDS`) and no ordering fails because of it |

### 10.11 Ruling FU item 1: C3 built (2026-11-17)

**Built** (`manpower.rs`): `soldier_upkeep(alpha, fee)` now returns, on `alpha_bracket >= INDUSTRIAL_UPKEEP_FIRST_BRACKET` (5, so brackets 5 and 6, Q13), `SOLDIER_UPKEEP_BY_BRACKET[b] * (fee / INDUSTRIAL_UPKEEP_REFERENCE_FEE)^INDUSTRIAL_UPKEEP_FEE_EXPONENT`, with the reference fee 0.09056 and the exponent 0.5 (Q12). `military_drivers` computes the fee once and passes it. Brackets 0-4 read no fee: by control flow, so `earlyIndustrial` and every pre-industrial row are bit-identical. No §7p switch (port-only, Ruling DE Q5 and DL precedent). **Both constants are fitted to the bar and say so in their doc comments (Ruling BK):** 0.09056 is the measured geometric-mean fee of bracket 5's derivation set, 0.5 the smallest round exponent that clears the grid; neither is sourced. The shape (a state that collects more per head pays its soldier more) is what has independent support.

**Reproduced against §10.9.3's C3 column**, from the model's own readings (not a re-price):

| | §10.9.3 C3 | now |
|---|---|---|
| sweep leaves (industrial unordered, of 290) | 0 | **0** (`ea0_`: 290 ordered on all six rows) |
| sweep within / above / below | 121 / 1 / 168 | **121 / 1 / 168** (pinned in `ea0_`) |
| grid unordered, industrial | 0 | **0** (was 1 003; pinned) |
| grid era-13 within / above / below | 248 624 / 0 / 34 400 | **248 624 / 0 / 34 400** (pinned in the driver-space test) |

**Re-pinned, each with its reason in the test:** the bracket 5/6 value literals now carry the fee (`soldier_upkeep_by_alpha_literals` at the reference fee, unmoved; `industrial_upkeep_follows_the_fee_on_brackets_5_and_6_only` for the term); `standing_army_rises_with_industrialisation_at_equal_population` industrial standing 68 929 -> 71 178 and the ratio 8.265 -> 8.534 (a monarchy's fee 0.0966 gives x1.0326); `a_median_polity_of_each_bracket_lands_in_its_own_band`'s industrial share 7.818 -> 7.658 %; the driver-space pin 1 003 -> 0 (the other five rows unchanged: `[0, 0, 0, 853, 0, 0]`); the `ea0_` industrial ordered count 288 -> 290, now asserted on all six rows, and the industrial manpower hash `0xfd92dc8c3684a768` -> `0xb982324cf21769af` with the other five bit-identical (the positive control). New: `industrial_upkeep_reference_fee_is_the_derivation_sets_geometric_mean` recomputes the reference fee and checks the derivation rule's geometric mean still lands at the band centre, so the term is the rule's own consequence and not a number asserted against itself. **Unmoved, run:** `G1_0`, `fh_r2` and Kingdom A and B (no generated faction is on bracket 5 or 6 at the default `civ.base_ag_tech`).

**Mutants (Python, exact literal, restored in a `finally`), all killed:** exponent 0.5 -> 0.25 and 1.0; reference fee 0.09056 -> 0.0906 and 0.09; gate 5 -> 4 and 5 -> 6; `>=` -> `>`; fee source replaced by `state_capacity`, by `EXTRACTION_FLOOR` and by `EXTRACTION_CEILING`.

#### Evidence

`cartalith-native/docs/evidence/industrial-army-2026-11-17/`: `ea0_run.log` (the existing `ea0_` diagnostic, passing), `ia_dump_run.log` (300 `IA row` and 300 `IA default` lines), `driver_space_run.log` (the grid), `analyse.py` and `analyse_out.txt` (the re-pricing, with its control). Code, ignored and pinning nothing: `g1_baseline.rs` `ia_dump_industrial_rows`; `cartalith-civ/tests/industrial_army_driver_space.rs`. Commands, each run alone:

- `cargo test -p cartalith-godot --lib ea0_ -j 4 -- --ignored --nocapture --test-threads 1`
- `cargo test -p cartalith-godot --lib ia_dump -j 4 -- --ignored --nocapture --test-threads 1`
- `cargo test -p cartalith-civ --test industrial_army_driver_space -- --ignored --nocapture`
- `python -I analyse.py ia_dump_run.log`

Not done: no `cargo test --workspace` (no product code changed; the two scratch tests are `#[ignore]`d); no probe; no mutation test (nothing is pinned).
