# FH-R2 — custom religions with real attributes: research read and re-baseline plan

**A plan for owner sign-off, written 2026-10-12 under Ruling CF item 5 ("FH-R2: proceed with a plan first").
No code has been written for it.** This document carries no status column; whether any of it is built is
`cartalith-native/docs/STATUS.md`'s answer, never this file's (`CLAUDE.md`, "One place holds state").

Every number below names its source: a symbol opened on 2026-10-12, the research document, or a **labelled
judgement** (Ruling BK). Every code citation names a symbol; line numbers are given only as a search aid and
drift (`MISTAKES.md`, "Re-resolve a citation late in a long pass").

---

## 1. What FH-R2 is

| Source | What it says |
|---|---|
| `FACTION_HUB_DESIGN.md` §6, phase table, row **R2 — real attributes** | Custom religions carry axes (exclusivity, locus, institutional form) that drive coexistence, retention and the siting rule. Engine change: `belief.rs` dynamics, `SettlementReligionState` use, civ→urban faith wiring (S3). **Hard dependency on S1-S7 and the S3 ruling; changes generated output, so it needs a ruling and a golden/pin re-baseline plan.** "R2 is not designed here." |
| `FACTION_HUB_DESIGN.md` §7, milestone row **FH-R2** | "blocked on the researched expansion S1-S7 and the S3 ruling … changes generated output" |
| `LARGE_ITEM_RULINGS.md` **Ruling CB item 8** (2026-10-11) | "FH-R2 (religion expansion S1 to S7 plus the S3 ruling): scheduled after editing-tools milestone F." |
| `LARGE_ITEM_RULINGS.md` **Ruling CF item 5** (2026-10-12) | "FH-R2: proceed with a plan first. Draft the research read (S1-S7 location to be found or asked) and the re-baseline plan for owner sign-off before any code." |
| `OUTSTANDING_WORK.md`, the FH-R2 row (section 2.11 area) | Routes here. |

What FH-R1 already shipped (the base R2 builds on), opened at its symbols: `cartalith-civ/src/religion_library.rs`
(`ReligionLibrary`, `ReligionDef {id, name, base_key, colour, notes}`, `ReligionLibrary::engine_key`). Its module
doc's "What it must never do" list says R1 must **never carry real attributes** and that every engine consumer
(`belief_seed`, `compat`, the relations faith term in `relations.rs`, the religious power axis
`faction_has_religion`) sees only a built-in key. R2 is exactly the change that lifts that rule, so that module
doc is one of the prose sites R2 must rewrite.

## 2. Where S1-S7 live — found

**S1-S7 are the stages of the owner-requested research document "Religion & Government Expansion"**, dated
2026-09-23, published as a Claude Docs document:
<https://claude.ai/artifact/X8PudUqgDYC6nMBidaorA7> (Claude Docs id `f3fa7bbe-666a-4228-b525-71241ea43df4`).
The only in-repository pointer to it is `OUTSTANDING_WORK.md`'s 2026-09-23 narrative entry ("Researched and
published, not built … staged S1–S7/G1 with 8 owner decisions gating the largest pieces (S3, civ→urban faith
wiring …)"). It was read in full for this plan on 2026-10-12 (revision 6).

**Naming hazard.** Two other "S1-S7" sets exist in this tree and are **not** this one:
`docs/research/settlement-generation-audit.md` / `docs/HANDOFF.md` (the source project's v1.17 settlement
refactor, S1 Site Profile … S7 Diagnostic overlays, shipped) and `STATUS.md`'s `RP-S1`…`RP-S7` (the GUI
replacement stages). A brief that says "S3" must say "religion-expansion S3".

**Recommendation for the owner (question Q0 below):** import the research document verbatim into the
repository as `RELIGION_GOVERNMENT_EXPANSION_RESEARCH.md`, as was done for `HYDROLOGY_CLASSIFICATION_RESEARCH.md`
and `LANDMARK_GENERATION_RESEARCH.md`, so the spec this plan depends on cannot drift or vanish outside git.

### 2.1 The stages, as the research document defines them

| Stage | Content (research doc's words, condensed) | Research doc's "changes generated output?" | **Re-checked 2026-10-12 against the code** |
|---|---|---|---|
| **S1** Taxonomy as data | Append 7 religions (`one_god`, `revealed_law`, `covenant_folk`, `the_path`, `mysteries`, `spirit_ways`, `god_king`); a `ReligionProfile` table (object, exclusivity, membership, locus, institution, TYPE); fill the design canvas's TYPE column | "No — defaults stay `none` until wired" | **Stale. Yes, it does** — see §4.1. The defaults were wired the same day (`civ_default_religion`), so appending to `CIV_RELIGIONS` changes fresh factions' religions |
| **S2** Coexistence dynamics | Axis-derived retention, axis-derived competition, trade-diaspora influx from `TradeFlow` | "No — the belief layer is inert until run" | Holds: `belief_is_absent_until_it_is_run_and_never_defaults` (`cartalith-godot/src/lib.rs` tests) pins that `compute_civilisation` does not build the layer; it is built only by `civ_belief_run`. It **does** change what every belief run produces (§4.2) |
| **S3** Civ→urban faith wiring | Religion→rite table; multi-site allocation by adherents; minority quarters; deity-by-district siting | "Yes — every town's faith sites change; needs its own ruling" | Holds. `urban_adapter.rs` passes `faith: None` today (beside `civic_style: None`), so every town takes its culture profile's default rite |
| **S4** New building forms | Assembly hall, fire temple, monastery/stupa, cult hall, sacred grove, lineage hall | Yes | Holds (new `cartalith-urban` geometry) |
| **S5** Pantheons and deities | Per-world generated deities, domains from real terrain, names from culture pools | "Cosmetic unless dedications drive siting" | Holds |
| **S6** Religious policy | Per-faction toleration axis (`plural`, `tolerant_establishment`, `exclusive_establishment`, `persecuting`) wired into retention/conversion | "No, until wired into faction defaults" | Holds only if the per-faction default is `plural`-equivalent to today; any non-neutral default moves belief runs |
| **S7** World-instanced religions (optional) | Per-world instances (archetype + origin culture + name); schisms spawn heresy instances | "Yes — touches the reference-ported relations bonus" | Holds: `relations.rs`'s `religion_term` (±0.20, key equality) |
| **G1** Governments | 6 core + 2 optional forms; 12-16 manpower calibration rows | "No, while unwired" | **Stale. Yes, it does** — `civ_default_government` is wired and cycles the non-`none` list (§4.1) |

The research document's eight decisions (D1-D8) are carried into §9 as owner questions.

### 2.2 What FH-R2 needs of S1-S7

FH-R2 ("custom religions carry axes that drive coexistence, retention and the siting rule") is, concretely:

- **S1** (the axis vocabulary a custom religion is given values on — R2's data model), and
- **S2** (the dynamics that read those axes — without it, axes are decoration), and
- **S3** (the siting rule — "drive … the siting rule"), and
- **S6** if toleration is to be per faction (the research doc's own finding that toleration is a ruler's property).

S4, S5, S7 and G1 are **not** required by FH-R2 and are proposed as separate later rows (Q3).

## 3. The code it would touch — opened 2026-10-12

| Symbol | File | What it is today | What FH-R2 does to it |
|---|---|---|---|
| `CIV_RELIGIONS: [(&str,&str); 8]` | `cartalith-civ/src/roster.rs` | The reference's 8 keys; doc: "never rename or reorder" (saves store keys by string) | S1 appends 7 (append only) |
| `CIV_GOVERNMENTS: [(&str,&str); 9]` | same | 9 forms, read by manpower | G1 only (not FH-R2) |
| `civ_default_religion(culture_key, faction_index)` | same | Picks the best `compat` value among non-`none` keys; **for unthemed cultures (`common`, `imperial`) cycles a tie of all 7 by `faction_index % tied.len()`** | Its tie set would grow unless new archetypes are excluded (Q2) |
| `civ_default_government(faction_index)` | same | `candidates[(faction_index-1) % 8]` | G1 only |
| `CIV_RELIGION_COUNT = CIV_RELIGIONS.len()` | `cartalith-civ/src/belief.rs` | Sizes every `share` array — appending grows them automatically (research doc's finding, confirmed) | none |
| `CIV_RELIGION_DOMAIN: [ReligionDomain; 5]`, `compat`, `compat_value`, `NEUTRAL_COMPAT` | `belief.rs` | Only the terrain-domain component K is evaluated; the doc discloses "31 of the 56 pairs return the stated neutral" | S1's 7 unthemed entries raise that to 80 of 105 (research doc's figure; 7×7 new + 31 = 80, 15×7 = 105). The disclosure prose must be updated |
| `belief_step(states, net, culture_of, rate)` | `belief.rs` | Three-term logistic (exposure, compat, conformity with `BELIEF_CONFORMITY_K`), one net `rate`; doc: "Conversion and retention are one number here, on purpose … Milestone 2 is the split" | S2: retention split, axis-derived competition, trade influx — new terms |
| `belief_seed(faction_of, faction_religion)` | `belief.rs` | Seeds every settlement wholly into its faction religion | none, but its doc is **stale** (below) |
| `SettlementReligionState { share }`, `BeliefNetwork` | `belief.rs` | Per-settlement distribution; road links only, 220 km cutoff (research doc) | S2 adds a trade-flow input (`trade::TradeFlow`, which carries endpoint indices including sea mode — research doc's finding, not re-opened here) |
| `CivData::civ_belief_run(years)` / `#[func] civ_belief_run` | `cartalith-godot/src/lib.rs` | On demand only; **not persisted** — `project_bridge.rs` restores `belief: Vec::new()` ("Not persisted: `CivData::belief`'s own doc states the decision") | S3 needs a decision here (Q5) |
| `religion_term` in the relations build | `cartalith-civ/src/relations.rs` | ±0.20 on key equality, `none` on either side is 0 | Moves wherever a faction's default religion moves (§4.1); S7 only beyond that |
| `faction_has_religion` → `religious` power axis | `cartalith-civ/src/lib.rs` (`civ_faction_aggregates`) | **Boolean** gate only | Unmoved by any key change while defaults never return `none` (`civ_default_religion`'s own test) |
| `urban_adapter.rs` place-options build (`faith: None`) | `cartalith-civ/src/urban_adapter.rs` | The town takes `profile.defaultFaith` | S3: supply a rite (and, in S3b, an allocation) |
| `build_faith_sites(seed, lots, buildings, anchors, count, faith: &str, site, harbour)`, `faith_name` | `cartalith-urban/src/districts.rs` | One rite string for every site; rites `church/temple/shrine/mosque/orthodox`; harbour/craftriver/quay excluded | S3b: a plural allocation and the deity-domain exception; must keep the single-rite path byte-identical (§5) |
| `ReligionDef`, `ReligionLibrary::engine_key` | `cartalith-civ/src/religion_library.rs` | `base_key` only | R2 proper: a custom religion's axis values (override of, or instead of, its base's profile) |
| `religions[].base_key` | `SAVEFILE_COMPAT.md` §9.9 | "MUST: one of the eight built-in religion keys" | A schema addition for axis fields (additive) and "eight" becomes "a built-in key" |

**Stale prose found in passing (not fixed here — this is a docs-only plan; route to the build's first lane):**
`belief_seed`'s doc comment ("`civ_roster_bridge::FactionEntry::default_for` sets `religion: \"none\"` for
**every** faction … the diffusion layer does nothing until a faction is given a religion by hand") is false
since 2026-09-23: `default_for` calls `civ_default_religion`, and the test
`the_shipped_roster_now_seeds_real_faith_from_its_wired_default` (`cartalith-godot/src/lib.rs`) pins the
opposite. Likewise the `civ_belief_run` `#[func]` doc's `reason` text ("Every faction's religion is set to
None") now describes only a hand-edited roster.

## 4. Exactly which outputs and pins would move

Method: the symbols in §3 were opened and the test pins below grepped by literal on 2026-10-12. **Nothing was
run** (no cargo, no Godot — another lane owned the build). Every "unmoved" below is a prediction to be confirmed
by M0/M1's hashes, not a measurement.

### 4.1 S1 (and G1) move fresh-world output through the wired defaults

`civ_default_religion` cycles the full non-`none` tie for unthemed cultures. Appending 7 unthemed religions (all
return `NEUTRAL_COMPAT` 0.5, as the research doc states) grows that tie from 7 to 14. Consequences, derived from
the function body:

- Faction indices 0-6 of a `common`/`imperial` culture keep their religion (the new keys are appended, so
  `tied[i]` is unchanged for `i < 7`).
- **Faction indices ≥ 7 of an unthemed culture change religion** (`i % 14` ≠ `i % 7`). Themed cultures are
  unaffected (their best value 1.0 is unique).
- That moves, for those factions: the relations `religion_term` (and so stances), the belief seed, the GeoJSON
  export's `religion` property (`cartalith-engine/src/geojson.rs`, `civFactionReligion[fid]`), and every shell
  surface reading `get_factions().religion`. The religious power axis does not move (boolean gate).
- If `mysteries` is excluded from defaults as the research doc recommends, the tie is 13, and indices ≥ 7 move
  the same way.

**Recommended (Q2):** S1 excludes the new archetypes from `civ_default_religion`'s candidate set until a later,
separately ruled pass, so **S1 changes no generated output by control flow** and is purely additive vocabulary.
That restores the research doc's own premise for S1.

G1 has the same shape (`civ_default_government`'s `% 8`), plus the `CITIZEN_MODERNISATION` hazard the research
doc names. G1 is out of FH-R2's scope and gets its own row.

### 4.2 Pins and goldens, by stage

| Stage | Pin or golden (crate, test or symbol) | Expected movement |
|---|---|---|
| S1 | `roster.rs` tests: `assert_eq!(CIV_RELIGIONS.len(), 8)` | Moves to 15 — a deliberate literal update |
| S1 | `belief.rs` tests: `assert_eq!(CIV_RELIGION_COUNT, 8, …)` | Moves to 15 |
| S1 | `belief.rs`: `CIV_RELIGION_DOMAIN.len() == 5` (and its bijection tests) | Unmoved — new entries are deliberately unthemed |
| S1 | `roster.rs` `civ_default_religion` tests (`("common", 0) == ("common", 7)`, the 7-distinct-picks test, the never-`none` test) | Unmoved **if** Q2's exclusion is taken; the first fails otherwise |
| S1 | `cartalith-godot/src/lib.rs` `the_shipped_roster_now_seeds_real_faith_from_its_wired_default` (`earth_mother` at index 1) | Unmoved either way (index 1 < 7) |
| S1 | `cartalith-godot/src/civ_roster_bridge.rs` default-religion tests | Unmoved with Q2's exclusion; re-check every literal otherwise |
| S1 | `SAVEFILE_COMPAT.md` §9.9 `base_key` "one of the eight" | Prose update; a v-old reader meeting a new key reads it as an unknown religion (`religion_library`'s documented pass-through) — a forward-compat disclosure, not a golden |
| S1 | Shell probes reading religion vocabulary: `_religionpanel_probe`, `_religionui_probe`, `_vfy_religion_probe`, `_relig0905_probe`, `_faithfaction_probe`, `_factionhub_probe`, `_lbrelig_probe`, `_rcustomdisplay_probe` (from `grep -l religion godot-project/_*probe.gd`, 20 files in all) | Any probe that counts picker rows moves; each must be re-run windowed, baseline first |
| S2 | `belief.rs` `belief_has_no_syncretic_equilibrium` | **Designed to fail** when coexistence arrives (its own message: "if this ever drops below 0.99, a coexistence mechanism has arrived and every surface that assumed a single winner needs revisiting"). Replace it with the coexistence bars in §6 — not delete it silently; the replacement keeps the isolated-village half of the claim |
| S2 | `belief.rs` determinism and order-independence tests (`the_same_world_produces_the_same_adherence_bit_for_bit`, `a_step_does_not_depend_on_settlement_order`) | Must stay green unchanged (they are properties, not values) |
| S2 | Any `belief.rs` test pinning a takeover time ("17-25 years or never", the research doc's reading of the file) | Moves; list every one old→new |
| S2 | Shell probes `_belief_probe`, `_shares_probe`, `_faithmeasure_probe`, `_faithdenom_probe`, `_faithcard_probe` | Values move; single-winner assumptions in surfaces (the test's own warning) must be hunted |
| S3 | `cartalith-urban/src/districts/tests/golden.rs`, `cartalith-urban/src/generate/tests/golden.rs` (reference-captured; cases pass their own `o_faith`, e.g. `faithNoneBasilica`) | **Must stay byte-identical** — S3 changes only what the civ adapter passes in; the urban crate's single-rite path is the JS-golden-verified one (DECISIONS §7p "What this does not change") |
| S3 | `cartalith-civ/tests/golden_parity_urban_adapter.rs` (generated by `tools/um_block2_capture.js`) | Predicted unmoved: its header lists `_umPlaceContext` outputs, and the adapter's own comment says faith is "not a `_umPlaceContext` field". **Confirm by hash in M0/M3** — if it captures town output end to end, it moves and is a re-baseline |
| S3 | Every generated town with a non-`none` faction religion whose rite differs from its culture profile's `defaultFaith` | Moves (sites' rite, building plan, provenance text) — this is the visible payoff and the reason S3 needs its own ruling |
| S3b | Town generation that reads belief shares | Moves only after Q5 is decided |
| S6 | Belief runs for any faction whose default policy is not neutral | Moves |

## 5. Re-baseline plan (DECISIONS §7p)

§7p's own test decides which side each change is on: FH-R2 **changes what a described feature produces**
(which faiths a town holds, where its sites stand), so per §7p's "What this does not change" paragraph it
"still needs its own ruling". It is not covered by §7p alone. Ruling CF item 5 asks for this plan; the
re-baselines below each need the owner's sign-off on this plan (or a later ruling) before they land, and
`MISTAKES.md` "Change generated output" applies: **a golden re-baseline needs an owner ruling.**

Rules for every re-baseline in this work, following the CD and CF precedents in `LARGE_ITEM_RULINGS.md`:

1. **Identity by control flow first.** Each stage is shaped so that the old inputs take the old code path:
   S1 with Q2's exclusion; S2 terms that are exactly zero for a world with no influx and default policy
   (verified by hash, not algebra); S3 with `faith: None` still meaning "profile default" for an unaffiliated
   faction and the reference-captured urban goldens untouched.
2. **Disclose after seeing the result.** Every moved value listed old→new in `STATUS.md`'s entry for the batch,
   as Ruling CD's addendum required.
3. **Scrub, don't dual-path** (§7p point 2). When S2 lands, the single-net-rate model is removed, not kept as a
   mode; `belief_step`'s "three terms, not nine" and "conversion and retention are one number" doc sections
   are rewritten in the same change; `religion_library.rs`'s "never carry real attributes" rule is rewritten
   when R2 lands. The urban crate's single-rite `build_faith_sites` is **not** a superseded path — it is the
   reference-verified case of the plural allocation (allocation of one rite), so it stays as that case.
4. **No re-baseline outside the ruled stage.** A golden that moves in a stage where §4.2 predicted it would not
   is a finding to report, not something to re-baseline in passing.

## 6. Measurement-first protocol, with numeric bars

All thresholds below are **labelled judgements** unless a source is named; the owner may move any of them.
Worlds: the seeds the planner and lake work already use where possible, plus at least three seeds per claim
(`MISTAKES.md`, "Report a layout measurement": one world is one sample).

**M0 — baseline, before any code.**
- Record `cargo test --workspace` result-line count and totals, and hash: every golden file listed in §4.2;
  for 5 seeds × {800 km, 40 075 km}: per-faction religion and government, the relations matrix, the GeoJSON
  bytes, per-town faith-site list (rite, count, centroid), and a 100-year belief run's share vectors.
- Record the shell probes in §4.2 (windowed, scratch copy) with their pre-existing failures as the baseline.

**M1 — S1 (taxonomy as data).** Bar: **0 bytes moved** in every M0 hash (with Q2's exclusion). Any move is a
failure, not a re-baseline. Literal pins updated: 8→15 only.

**M2 — S2 (coexistence) — measure before calibrating.** First add the terms with weight 0 and confirm 0 bytes
moved. Then, for the calibrated weights, the bars:
- *Hubs hold minorities:* among settlements in the top decile of trade volume, **at least 50%** hold two or
  more faiths each at ≥ 5% share after 200 simulated years (labelled judgement; the research doc's
  Karakorum/Quanzhou/Dura-Europos evidence supports "hubs hold communities", not a number).
- *Isolated villages still converge:* among settlements with road degree ≤ 1 and zero trade flow, **at least
  90%** hold one faith at ≥ 95% after 200 years (keeps the half of `belief_has_no_syncretic_equilibrium`'s claim
  that is historically right).
- *The analytic check:* for a two-node fixture with constant influx `I` and out-flow rate `o`, the steady
  minority share is within **±10%** of `I / (I + o)` (the research doc's "stable minority share roughly equal to
  influx over out-flow").
- *Distribution and determinism:* shares sum to 1 within `1e-9`; bit-identical re-runs; mirrored chains agree
  within `1e-12` (the existing tests' tolerances).
- *Cost:* a 100-year `civ_belief_run` on the largest harness world, median with min..max over 5 runs, run alone,
  **no more than 2× today's median** (labelled judgement; the trade term adds one pass over `TradeFlow` per
  step).
- Mutation-test each new constant (retention order, competition weights, influx coefficient): each must turn a
  test red.

**M3 — S3a (rite from faction religion only).** Bars: urban reference goldens **byte-identical**; an unaffiliated
(`none`) faction's towns byte-identical to M0; every other town's sites differ only in rite and its
consequences, never in count (count is still `clamp(round(households/500), 1, 4)`, research doc), checked over
all towns of 5 seeds.

**M4 — S3b (plural allocation, quarters, deity domains).** Bars: sites allocated by adherent share with the
institution-dependent threshold; no minority site inside the harbour/craftriver/quay exclusion **except** a
deity whose domain is sea/river (the research doc's stated exception); ≥ 180 m site spacing preserved (the
research doc's reading of `build_faith_sites`); a before/after image of three towns at every zoom touched,
**opened and looked at** (`MISTAKES.md`, "Verify a visual change before committing").

**M5 — R2 proper (custom religion axes).** Bar: a custom religion whose axis values equal its base's profile is
**byte-identical** to its base in every M0 hash; changing one axis moves only the terms that read that axis
(change each input, confirm the dependent reacts — `MISTAKES.md`, "Add a capability").

## 7. Milestones

Each is one verified batch; per the standing cap, one builder lane plus a verifier, Opus for any lane that
changes generated output (the `FACTION_HUB_DESIGN.md` tier rule for FH-5/FH-R1).

| # | Milestone | Needs | Output change |
|---|---|---|---|
| **R2-0** | Import the research doc into the repo (Q0); fix the stale `belief_seed` / `civ_belief_run` prose; record M0 | Q0 | none |
| **R2-1** | S1: append 7 religions, `ReligionProfile` table (axes 1-5 + TYPE), the existing 8 classified as the owner approves (Q1), Religion screen TYPE column filled, `SAVEFILE_COMPAT.md` §9.9 updated | Q1, Q2 | none (M1) |
| **R2-2** | S2: retention split, axis-derived competition, trade-diaspora influx; replace `belief_has_no_syncretic_equilibrium`; surfaces that assumed a single winner revisited | Q4 | belief runs (M2) |
| **R2-3** | R2 proper: `ReligionDef` gains axis values (default = base's profile), library editor fields, save schema addition | Q6 | none for unedited religions (M5) |
| **R2-4** | S3a: religion→rite table; `urban_adapter.rs` passes the faction's rite | S3 ruling (Q3) | every affiliated town's rite (M3) |
| **R2-5** | S3b: plural site allocation, minority quarters, deity-by-district siting | Q3, Q5 | town layouts (M4) |
| **R2-6** | S6: religious-policy axis per faction, as a retention/conversion modifier only | Q7 | belief runs only |

S4, S5, S7 and G1 are outside FH-R2 and are proposed as their own `OUTSTANDING_WORK.md` rows if the owner wants
them.

## 8. Risks

- **The research document's "no output change" for S1 and G1 is stale** (§4.1). Taking it at its word would
  ship an unruled output change.
- **Single-winner assumptions in surfaces.** The syncretic test's own message says every surface that assumed
  one faith needs revisiting when S2 lands; the Religion screen, the faith card and settlement inspectors are
  the candidates (probe list in §4.2).
- **Determinism of town layouts (D5).** Belief is not persisted and not run at generation; if S3b reads shares,
  the same town could render differently before and after a user runs the simulation. Q5 must be answered
  before R2-5.
- **Neutral-pair dilution.** 80 of 105 pairs neutral after S1 (research doc's figure): without S2 the new
  religions differ only by exposure, so S1 alone gives the user labels with no behaviour — the UI must say so
  (as R1's "behaves as its base" line does).
- **Save forward-compatibility.** An older build opening a world with a new key or new `religions.json` fields
  reads an unknown religion as no faith (`religion_library`'s pass-through) — disclose in `SAVEFILE_COMPAT.md`.
- **Reference-golden contamination.** S3b touches `cartalith-urban`, which holds JS-captured goldens; the
  single-rite path must remain the exact reference path (M3/M4 bars).
- **Calibration without data.** S2's weights are authored orderings (birth > universal > initiatory > folk,
  research doc), not fitted values; each must be labelled as a judgement in its doc comment (Ruling BK).

## 9. Owner questions, each with a recommended default

| # | Question (research doc decision in brackets) | Recommended default |
|---|---|---|
| **Q0** | Import "Religion & Government Expansion" into the repository as `RELIGION_GOVERNMENT_EXPANSION_RESEARCH.md`, verbatim? | **Yes** — the spec should live in git |
| **Q1** | Approve the axes and the classification of the existing 8; is `flame_creed` a dualist fire-temple religion or a confessional monotheism? [D1] | Approve the axes; `flame_creed` = **dualist, exclusive-leaning** (the research doc's "softest" call; keeps it distinct from `one_god`) |
| **Q2** | Approve the 7 new religions; add `joined_teachings`? Should the new archetypes be eligible for fresh-faction defaults? [D2, D8] | Approve the 7; **no** `joined_teachings`; **exclude all new archetypes from `civ_default_religion`** in R2-1 so S1 moves no output; revisit defaults in a separately ruled pass |
| **Q3** | Authorise S3, the civ→urban faith wiring that changes every affiliated town's generated output, with the re-baseline in §5? [D4] | **Yes, in two steps**: S3a (rite only) first, S3b after Q5 |
| **Q4** | Approve S2's three terms (retention, competition, trade influx) and M2's bars, including replacing `belief_has_no_syncretic_equilibrium`? | **Yes**, bars as written in §6 |
| **Q5** | Belief determinism for town rendering: run a fixed number of years at generation time, or persist the simulated state? [D5] | **Persist**, and have S3b read the persisted state, falling back to the faction religion alone when none exists (never a fabricated share). A fixed generation-time run adds cost to every generate (the 4096-wide generate already blocks ~5.8 s, `FACTION_HUB_DESIGN.md` §8) |
| **Q6** | For a custom religion: free axis values, or base profile plus overrides? | **Base profile plus overrides**, so an unedited custom religion stays byte-identical to its base |
| **Q7** | Add the per-faction religious-policy axis (S6) as part of FH-R2? [D7] | **Yes, after S2**, with every faction's default policy carrying an exactly neutral modifier (1.0 on retention and conversion), so adding the axis moves nothing until a policy is changed |
| **Q8** | Fixed archetype list now, world-instanced religions (S7) later? [D3] | **Fixed list now**; S7 its own row |
| **Q9** | G1 governments and their authored manpower rows, including the stratocracy re-derivation hazard [D6] | **Out of FH-R2**; own row and ruling; if built, also exclude new forms from `civ_default_government` first |

## 10. What was opened for this plan

`MISTAKES.md` (preflight table); `FACTION_HUB_DESIGN.md` §6-§9; `LARGE_ITEM_RULINGS.md` Rulings CB and CF;
`OUTSTANDING_WORK.md` (FH-R2 row and the 2026-09-23 religion entry); `RELIGION_DIFFUSION_SCOPE.md` §3-§4;
`DECISIONS.md` §7p; the research document above. Code: `roster.rs` (`CIV_RELIGIONS`, `CIV_GOVERNMENTS`,
`civ_default_religion`, `civ_default_government`, their tests), `belief.rs` (`belief_step`, `belief_seed`,
`CIV_RELIGION_COUNT`, `belief_has_no_syncretic_equilibrium`), `religion_library.rs` (module doc),
`relations.rs` (module doc, `religion_term`), `cartalith-civ/src/lib.rs` (`faction_has_religion`, the religious
axis), `urban_adapter.rs` (`faith: None`), `districts.rs` (`build_faith_sites`, `faith_name`),
`cartalith-godot/src/lib.rs` (`civ_belief_run`, its tests), `project_bridge.rs` (belief not persisted),
the headers of `golden_parity_urban_adapter.rs` and `cartalith-urban/src/generate/tests/golden.rs`. Searches
stayed inside the repository. **Not opened:** `trade::TradeFlow`, `manpower.rs` beyond its table pins, and the
Religion design canvas — their claims above are the research document's, marked as such.
