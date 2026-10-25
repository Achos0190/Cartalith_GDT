# Governments (G1) — scope

**What this is.** The scope for stage G1 of the 2026-09-23 research document "Religion & Government
Expansion" (`RELIGION_GOVERNMENT_EXPANSION_RESEARCH.md`, imported verbatim under Ruling CG item 3): six core
and two optional government forms, the manpower rows each one needs, what each form changes, the hazards found
at the symbols, the save and UI consequences, a re-baseline plan, milestones with numeric bars, and owner
questions with recommended defaults.

**Why it exists.** `FH_R2_PLAN.md` Q9 put G1 outside FH-R2. Ruling CJ (2026-10-18) confirmed that: *"Q9
(governments) stays out of FH-R2 and needs its own row and ruling; if governments are ever built, exclude the
new forms from `civ_default_government` first."* The owner (2026-10-24/25) asked for governments to be scoped
first and said he would approve before any build. **Approved 2026-10-25 (Ruling CZ, `LARGE_ITEM_RULINGS.md`): every
§9 question at its recommended default.** What has been built since is in `STATUS.md`, not here.

**What it is not.** It is not a status record. Like every scope document, it defines milestones and does not
track them. Progress lives in `cartalith-native/docs/STATUS.md` (`CLAUDE.md`, "One place holds state"). The
backlog row is in `OUTSTANDING_WORK.md` §3.1.

**Method, and what was opened (2026-10-25).** Each claim below was re-opened at its symbol, on the tree at
`86104453`. Every number is one of four kinds: **cited** (its source is named), **computed** (§7.1 gives the
method), a **labelled judgement**, or a **projection** (derived from control flow and not yet run). The word
"measured" is used only for the replica check in §7.1, which was run during this pass.

| Opened | Symbols |
|---|---|
| `cartalith-civ/src/roster.rs` | `CIV_GOVERNMENTS` (and its doc's 2026-09-23 correction), `civ_default_government`, `CIV_RELIGIONS_REFERENCE_LEN`, `civ_default_religion` (the bound it uses), tests `vocabularies_are_the_reference_lengths`, `default_government_cycles_every_real_form`, `default_government_never_returns_none` |
| `cartalith-civ/src/manpower.rs` | module doc; `GOVERNMENT_EXTRACTION`, `government_extraction`, `CITIZEN_SHARE`, `citizen_share`, `CITIZEN_MODERNISATION`, `CITIZEN_FLOOR`, `CITIZEN_CEILING`, `SOLDIER_UPKEEP_BY_BRACKET`, `ALPHA_BRACKETS`, `ERA_BANDS`, `era_for`, `military_drivers`, `civ_military_manpower`, `civ_military_manpower_world`, `world_land_reference`; tests `worked_example_kingdom_a`/`_b`, `the_citizen_ruling_moves_no_headcount`, `citizen_fraction_is_driven_by_government`, `citizen_fraction_widens_as_the_society_leaves_agriculture`, `citizen_share_table_is_the_roster_vocabulary_and_falls_back_safely`, `government_table_is_the_roster_vocabulary_and_falls_back_safely`, `soldier_upkeep_is_derived_from_the_era_table`, `soldier_upkeep_by_alpha_literals`, `a_median_polity_of_each_bracket_lands_in_its_own_band` |
| `cartalith-civ/src/` | `conflict.rs` (`side_manpower`), `garrison.rs` (module doc, `standing`), `timeline.rs` (`YearFactionInstitutions`), `lib.rs` (`civ_faction_aggregates`, `civ_tax_rate`); `relations.rs` has no `government` term (grepped) |
| `cartalith-godot/src/` | `civ_roster_bridge.rs` (`FactionEntry::default_for`, `set_field`'s `"government"` arm, tests `seeded_roster_matches_the_reference_base_table` and the index-8 `find_or_create_by_name` case); `civ_military_bridge.rs` (`resolve_institutions`); `project_bridge.rs` (the `factions.json` read and write of `government`); `vault_bridge.rs` (`vocab_label`); `lib.rs` (`civ_government_vocabulary`, `key_label_array`, the `get_factions` `"government"` key); `fh_r2_baseline.rs` (module doc: columns, worlds, `AT_CURRENT`) |
| Shell | `faction_roster_window.gd` (`_vocab_choice`, the Identity Government row), `right_dock.gd` (Faction card's Government field), `civilization_workspace.gd` (the Military block's "Who the bands are measured against" note), `_manpower_probe.gd` (`GOVS`) |
| Tests directory | `cartalith-civ/tests/golden_parity_roster.rs` and `golden_parity_military.rs`: neither mentions a government (grepped) |
| Documents | `CLAUDE.md`; `MISTAKES.md` preflight table; `FH_R2_PLAN.md` §2.1, §3, §4.1, §9; the research document's Governments sections and its decision 6; `MILITARY_MANPOWER_SCOPE.md` §1, §2.3, §2.6, §3.2-§3.2a; `FACTION_HUB_DESIGN.md` §3.3-§3.5 and §8; `SAVEFILE_COMPAT.md` §9.2 and §10.1; `ECONOMY_SCOPE.md` (its `CIV_GOVERNMENTS` paragraph); `LARGE_ITEM_RULINGS.md` Rulings AI, CF, CG, CJ and CY |

**Not opened:** `campaign.rs`, `conflict_bridge.rs` beyond its `manpower` grep, the Godot widget code behind
`DccWidgets.choice`, and the Faction design canvas. Claims about those are marked as such where they appear.
**Nothing was run** apart from the scratch replica in §7.1: no `cargo` and no Godot, because another lane was
pushing volcano work during this pass. Every "unmoved" below is therefore a projection, and G1-0 and G1-1 exist
to confirm it.

---

## 1. What exists today

**Nine forms.** `CIV_GOVERNMENTS: [(&str, &str); 9]` holds `none`, `chiefdom`, `tribal_confederacy`,
`monarchy`, `oligarchy`, `republic`, `theocracy`, `empire` and `city_state`, in that order. This is the
reference's own list (reference line 14794, cited in the doc). Saves store these keys as strings, so the list
can only be appended to, never reordered.

**One consumer: the manpower model.** `manpower.rs` keys two tables by the same strings:

- `GOVERNMENT_EXTRACTION` (none 0.10 … empire 0.70). This enters `military_drivers`'s `state_capacity`:
  `extraction × (0.55 + 0.45·reach) × (0.70 + 0.60·urban)`, clamped to 0.03-0.95. That one number drives
  `fiscal_extraction_efficiency`, `professionalization`, the levy reach (`LEVY_STATE`), the campaign
  capability, and so the standing army, field army, levy, war duration and **which era row `era_for` picks**
  (it splits rows by `state_capacity`).
- `CITIZEN_SHARE` (none 0.90 … empire 0.30). This enters only `citizen_fraction`:
  `clamp(share + CITIZEN_MODERNISATION × urban, CITIZEN_FLOOR 0.20, CITIZEN_CEILING 0.98)`. At run time that is
  the era-band **verdict's denominator** (`citizen_population`, `standing_citizen_share`,
  `emergency_citizen_share`, `era_*_verdict`) and nothing else. The test `the_citizen_ruling_moves_no_headcount`
  pins this.

**Two derivations read both tables.** Both feed the hazards in §4.

- `CITIZEN_MODERNISATION = 0.68` is documented as `CITIZEN_CEILING − min(CITIZEN_SHARE)` = 0.98 − 0.30 (the
  `empire` row). `citizen_fraction_widens_as_the_society_leaves_agriculture` asserts that identity and asserts
  that every `CITIZEN_SHARE` row reaches the ceiling at the industrial row.
- `SOLDIER_UPKEEP_BY_BRACKET = [1.1200, 0.7983, 1.8153, 2.2214, 1.7696, 1.0714, 1.0714]` (Ruling AI (c)). This
  is a geometric mean over *"every roster government but `none` × capital reach 0 … 1.0"*.
  `soldier_upkeep_is_derived_from_the_era_table` re-derives it from `CIV_GOVERNMENTS`, `GOVERNMENT_EXTRACTION`
  and `CITIZEN_SHARE`, and asserts `govs.len() == 8`.

**The default pick is wired.** `civ_default_government(i)` returns `candidates[(i−1) % 8]` over every non-`none`
key. `FactionEntry::default_for` calls it for every index except 0 (wired 2026-09-23, owner: "Yes, wire it in
now"). A fresh faction's government therefore changes generated output. That is why the research document's
"G1 changes generated output? No, while unwired" is stale (`FH_R2_PLAN.md` §2.1, §4.1).

**What government does not reach today**, grepped across `cartalith-civ`, `cartalith-engine` and
`cartalith-godot`:

- the economy (`civ_faction_aggregates`' `tax_income` is `pop × civ_tax_rate(kind)`, per settlement tier; that
  is reference line 23557, cited in its doc);
- relations (`relations.rs` has no government term);
- the power axes, belief, town layout, the GeoJSON export (`cartalith-engine` has no `government`), and the
  collapse/recovery timeline (`timeline.rs` only *records* the key, in `YearFactionInstitutions`).

**Downstream of the manpower model:** garrisons (`garrison.rs` splits `standing_army`), conflict side readings
(`conflict::side_manpower`), the CIVIL ▸ Military block, and the Faction hub's Military tab.

**No JS-captured golden reads a government.** Neither `golden_parity_roster.rs` nor `golden_parity_military.rs`
mentions one, and `manpower.rs`'s module doc states that the reference has no army-size model.

---

## 2. The forms

Seven of the keys and labels below are the research document's. Its "why distinct" column is quoted where it
is used. The research document also notes that today's list mixes two axes, constitution (rule by one, few or
many) and territorial scale (city-state against empire), and recommends documenting that rather than
restructuring. This scope follows that: the vocabulary is appended, and the UI groups it (§6).

| # | Key | Label | Real-world models (research doc) | Core / optional |
|---|---|---|---|---|
| 9 | `merchant_republic` | Merchant Republic | Venice, Genoa, Novgorod | core |
| 10 | `city_league` | City League | the Hanseatic League, the Lombard League | core |
| 11 | `sacral_monarchy` | Sacral Monarchy | the Caliphate, Byzantium, Pharaonic Egypt | core |
| 12 | `holy_order` | Holy Order | the Teutonic Order, the Hospitallers | core |
| 13 | `feudal_realm` | Feudal Realm | Capetian France | core |
| 14 | `nomad_khanate` | Nomadic Khanate | the Mongols, the Turkic khaganates | core |
| 15 | `stratocracy` | Military Regime | the Mamluk Sultanate, the Tokugawa bakufu | optional (Q3) |
| 16 | `tributary_hegemony` | Tributary Hegemony | the Aztec Triple Alliance | optional (Q3) |

`#` is the index the form would take in `CIV_GOVERNMENTS`, after `city_state` at 8. The core six are appended
first and the optional two after them, so dropping the optional pair never renumbers a core form.

`theocracy` keeps its key and label. The research document re-documents it as *hierocracy* (rule by a
priesthood: the Papal States, the prince-bishoprics), and that goes in its doc comment only (Q7).

---

## 3. The manpower rows: 12 core, 16 with the optional pair

Each form needs exactly two rows, one in `GOVERNMENT_EXTRACTION` and one in `CITIZEN_SHARE`. Both tables'
coverage tests compare their lengths with `CIV_GOVERNMENTS.len()`. **These are authored orderings, not
calibrated values.** The research document says so (*"Only two worked examples exist for calibration … new rows
would be authored orderings"*), and its decision 6 asks the owner to accept that. Every value below is placed
against an existing row. A value that equals an existing row is that row's grounding reused, cited by its
anchor. A value that falls between two rows is a **labelled judgement**, with its reason given.

The specification's own cautions constrain these rows. Quoted from `MILITARY_MANPOWER_SCOPE.md` §1:

- *"Geography, state organization, wealth inequality, military culture and whether soldiers are
  self-supporting can move a society substantially outside"* the era bands.
- *"'warrior societies' should not automatically get huge standing-army modifiers"*.

So no row gives a military-flavoured form extra standing army through the label alone.

### 3.1 `GOVERNMENT_EXTRACTION` (share-of-surplus capacity, 0..1)

Anchor rows today: none 0.10 · chiefdom 0.15 · tribal confederacy 0.20 · monarchy and theocracy 0.45 ·
oligarchy 0.50 · republic and city-state 0.55 · empire 0.70. The table's doc grounds the ordering: *"a chiefdom
redistributes what it can see, a city-state and a republic tax a small area intensively, an empire runs a
professional revenue service"*.

| Key | Value | Kind | Reason |
|---|---|---|---|
| `merchant_republic` | **0.55** | equals `republic`/`city_state` | The intensive small-area tax state. Customs and excise revenue is a monetised economy, and the model already credits that through `state_capacity`'s urbanisation term `(0.70 + 0.60·urban)`. A higher base would count trade twice |
| `city_league` | **0.30** | labelled judgement | *"Confederated cities, weak centre, no common treasury."* The member cities tax well; the league as one faction pools little. It sits above `tribal_confederacy` (0.20, a confederation without urban fiscal machinery) and below `monarchy` (0.45) |
| `sacral_monarchy` | **0.50** | labelled judgement | Byzantium and the Caliphate ran salaried fiscal bureaucracies beyond an ordinary monarchy, but they are not credited with `empire`'s professional revenue service at every scale. It sits between `monarchy` (0.45) and `empire` (0.70), equal to `oligarchy` |
| `holy_order` | **0.50** | equals `oligarchy` | A military-monastic corporation is rule by a corporate few. Anchored to the `oligarchy` row's grounding |
| `feudal_realm` | **0.30** | labelled judgement | *"Decentralised — revenue/troops sit with vassals."* The crown captures less than an administrative monarchy (0.45). Set equal to `city_league`'s 0.30 because both are weak centres over strong parts |
| `nomad_khanate` | **0.25** | labelled judgement | Tribute and booty with a thin bureaucracy. It sits just above `tribal_confederacy` (0.20), because a khanate's decimal command structure reaches further than a kin confederation, and below `feudal_realm` |
| `stratocracy` | **0.60** | labelled judgement | The state exists to pay its soldiers (the Mamluk *iqta'*, the bakufu's stipends). It sits between `city_state` (0.55) and `empire` (0.70) |
| `tributary_hegemony` | **0.40** | labelled judgement | *"Tribute extraction from nominally autonomous subjects."* Tribute is extraction without administration, so it sits below `monarchy` (0.45) and above `feudal_realm` |

### 3.2 `CITIZEN_SHARE` (full-civic-status share before modernisation, 0..1)

Anchor rows today: none and chiefdom 0.90 · tribal confederacy 0.88 · monarchy and theocracy 0.55 (Domesday
England, cited in the doc) · republic 0.50 (Polybius' 225 BC figures) · city-state 0.45 (Athens c. 431 BC) ·
oligarchy 0.40 (*"the least-grounded row in this table"*, by its own doc) · empire 0.30 (pre-Caracalla
citizens, *"a fifth to a third of the empire"*).

**Every new row must be at least 0.30** (§4.1). The reasons below show that the floor follows from what the
row measures, not only from the hazard. The row is the share of the population holding full civic status: the
citizen or free body. It is not the share holding political power.

| Key | Value | Kind | Reason |
|---|---|---|---|
| `merchant_republic` | **0.50** | equals `republic` | The `oligarchy` row's doc says *"Venice's patriciate over a free populace is far higher"* than 0.40. The free body is broad and the franchise narrow, and this table measures the first. Anchored to `republic` |
| `city_league` | **0.50** | equals `republic` | A league of burgher towns: the same free civic body as a merchant republic |
| `sacral_monarchy` | **0.55** | equals `monarchy` | A free peasantry and townsfolk over a servile or tied substrate. Anchored to the `monarchy`/`theocracy` row's Domesday grounding |
| `holy_order` | **0.40** | equals `oligarchy` | A small corporate ruling body over a heavily tied subject peasantry (the Prussian lands). Anchored to `oligarchy`, and like that row, weakly grounded |
| `feudal_realm` | **0.55** | equals `monarchy` | The `monarchy` row's own grounding is Domesday England, which is a feudal society. Reusing it is the honest choice |
| `nomad_khanate` | **0.88** | equals `tribal_confederacy` | Kin-based free horsemen. The table's doc says kin-based polities *"barely distinguish status at all"* |
| `stratocracy` | **0.30** | equals `empire`; floor-bound (§4.1) | The ruling military caste is small: the Tokugawa samurai were around a sixteenth of the population (labelled judgement, not a citation). But the subject population of both models was legally free and not chattel. The row measures the free body, which is not below a slave-holding empire's. A value below 0.30 is a different modelling claim and needs its own ruling (Q3, Q4) |
| `tributary_hegemony` | **0.30** | equals `empire` | Conquered subject peoples outside the ruling people's citizen body. The same grounding as `empire` |

### 3.3 A third knob for the khanate only, optional (G1-5, Q5)

The research document flags `nomad_khanate`: *"Near-universal mobilisation of free horsemen — flagged: the
existing two-knob extraction/citizen-share schema may not express this well."* The code confirms the flag. The
levy reach is `clamp(LEVY_BASE 0.04 + LEVY_STATE 0.30·state + LEVY_LOGISTICS 0.22·logistics, 0, 0.60)`.
Government enters only through `state_capacity`. A low-extraction khanate therefore gets a *low* levy, which is
the opposite of the historical case.

Computed in §7.1: at a traditional-agrarian test faction, the khanate's levy is 3.8 % of its citizens, against a
5-15 % mobilization band. `chiefdom` and `tribal_confederacy` read the same way.

The proposed fix is a third table, `GOVERNMENT_LEVY_BONUS`:

- It adds to the levy reach inside the existing 0.60 cap.
- It has **one row** (`nomad_khanate`). The lookup returns exactly `0.0` for every other key, so every
  existing form is unchanged by control flow, not by arithmetic.
- It touches neither the standing army nor the upkeep derivation. The specification's caution is about
  *standing-army* modifiers. A larger levy with no larger standing army is exactly the shape it describes (*"the
  military is the adult population temporarily switching occupation"*).

The value is set by a stated rule rather than chosen, the same discipline `SOLDIER_UPKEEP_BY_BRACKET` uses. The
rule: a median khanate (reach 0.5, logistics 0.5, ecological 1, on the `traditionalAgrarian` row) lands at the
centre of its own era's mobilization band. §7.1's sweep puts that near **0.25**. That figure is indicative only;
G1-5 derives the value and pins the derivation in a test.

---

## 4. The hazards, and how each is prevented

There are three hazards. The research document named the first; the second and third were found re-opening
the symbols for this scope. Each one is caught by an existing test, so none of them would ship *silently*. The
real risk is the tempting fix: re-deriving the constant so the test passes, which moves every faction's
numbers.

### 4.1 `CITIZEN_MODERNISATION` is derived from the lowest `CITIZEN_SHARE`

**The exact hazard.** `CITIZEN_MODERNISATION` (0.68) is derived as `CITIZEN_CEILING` (0.98) minus the lowest
`CITIZEN_SHARE`, which is empire's 0.30. The derivation's own statement is *"at full industrialisation, civic
status is universal whatever the government is called."* A new form below 0.30 (a stratocracy at, say, 0.25)
makes the lowest share 0.25, and the derivation then demands 0.73.

**What moves if a builder re-derives it** (computed, §7.1):

- **The citizen fraction of every government whose fraction is not already at the ceiling.** At Kingdom A's
  inputs, `monarchy` and `theocracy` go 0.7455 → 0.7599, `republic` 0.6955 → 0.7099, `city_state`
  0.6455 → 0.6599, `oligarchy` 0.5955 → 0.6099 and `empire` 0.4955 → 0.5099. `chiefdom` and
  `tribal_confederacy` are already clamped at 0.98 and do not move.
- **Every standing army.** `SOLDIER_UPKEEP_BY_BRACKET`'s derivation divides by `citizen_fraction`, so
  re-deriving the upkeep after the change moves Kingdom A's standing army 9 661 → 9 800 (+1.4 %) and Kingdom
  B's 25 750 → 26 409 (+2.6 %). The worked-example pins fail.
- **The median-polity band literals.** `a_median_polity_of_each_bracket_lands_in_its_own_band` moves (for
  example, the High-medieval share 1.171 % → 1.152 %).

**What does not move.** The era *row* a polity lands in. `era_for` reads `agricultural_labour_ratio` and
`state_capacity`, never `citizen_fraction`. The earlier description, *"shifts every government's era band"*,
is therefore exact only if "era band" means the band's verdict and its denominator. The era assignment holds.

**What happens if a builder does not re-derive it.** The identity assertion in
`citizen_fraction_widens_as_the_society_leaves_agriculture` fails. If that assertion were simply deleted, the
sub-0.30 form would never reach the ceiling at the industrial row, which breaks the derivation's own statement.

**Prevention**, built in G1-1 before any vocabulary is appended:

1. **Every appended `CITIZEN_SHARE` row is at least 0.30.** §3.2 shows the floor follows from what the row
   measures. A new test asserts, as a **literal** (`MISTAKES.md`: never assert a constant against itself), that
   the minimum over the whole table is `0.30`, and that `CITIZEN_MODERNISATION` is `0.68`. The failure message
   names this section.
2. **The derivation is stated over the reference rows.** The identity test computes `lowest` over
   `CITIZEN_SHARE`'s rows for `CIV_GOVERNMENTS[..CIV_GOVERNMENTS_REFERENCE_LEN]` (§4.3). The "every government
   converges at the industrial row" loop keeps running over *all* rows, so an appended row below the floor
   still fails it.
3. **A sub-0.30 form is a separate ruling** (Q3/Q4), and that ruling is a disclosed re-baseline under
   `DECISIONS.md` §7p with the movements above recorded old → new. The alternative a ruling could choose is a
   per-row modernisation: `share + (CEILING − share)·urban`. That reaches the ceiling for every row with no
   global constant, but it changes every existing government's fraction at every non-industrial row. It is a
   larger re-baseline, not a smaller one.

### 4.2 `SOLDIER_UPKEEP_BY_BRACKET` is a mean over every roster government

**The hazard.** The upkeep table is the geometric mean over *"every roster government but `none`"*. The test
re-derives it by iterating `CIV_GOVERNMENTS` and asserts `govs.len() == 8`. Appending any form fails that
assertion. If the builder lets the derivation run over all the forms and re-records the table, **every
faction's standing army moves, including factions whose government did not change**. Computed (§7.1) with the
§3 rows over 16 forms, the table becomes `[1.1814, 0.8538, 1.9302, 2.2650, 1.7779, 1.0662, 1.0662]`. Kingdom A
goes 9 661 → 9 086 (−5.95 %) and Kingdom B 25 750 → 25 255 (−1.92 %). Garrisons and conflict side readings
follow, because they read `standing_army`.

**Prevention** (G1-1): the derivation set is the reference eight. The test iterates
`CIV_GOVERNMENTS[..CIV_GOVERNMENTS_REFERENCE_LEN]` minus `none`, and keeps its `govs.len() == 8` literal. The
constant's doc comment is changed to say *"every reference government"* and why: the era bands were calibrated
against that population of forms, and an appended form is priced *by* that calibration, not *into* it. Changing
the set is a separately ruled re-baseline.

### 4.3 `civ_default_government` cycles every non-`none` form

**The hazard.** `candidates[(i−1) % candidates.len()]` over every non-`none` key. Appending forms changes the
modulus, and every fresh faction at index 9 or above changes government. Projected from the function body, on
24-faction worlds:

- with the six core forms (14 candidates), **16 of 24 factions move** (indices 9-24);
- with all eight (16 candidates), **8 of 24 move** (indices 9-16).

Indices 1-8 never move. So the app-default 6-faction world (`WorldParams::defaults()`'s `civ.factions`, per
`fh_r2_baseline.rs`'s module doc) is unaffected either way. Larger rosters, GeoJSON-imported factions, and
factions added past index 8 are affected.

**Prevention** (G1-1), mirroring the religion precedent exactly:

- Add `CIV_GOVERNMENTS_REFERENCE_LEN: usize = 9`, beside `CIV_RELIGIONS_REFERENCE_LEN`. Its doc says it must
  never be raised without a separately ruled pass.
- `civ_default_government` draws from `CIV_GOVERNMENTS[..CIV_GOVERNMENTS_REFERENCE_LEN]` minus `none`.
- Add a test, `default_government_never_picks_an_appended_form`, over indices 0..64.

This is what Ruling CJ requires (*"exclude the new forms from `civ_default_government` first"*). The existing
period-8 test stays as written and keeps passing.

### 4.4 Lesser hazards found in passing (routed into G1-1 and G1-3)

- **The shell picker substitutes a plausible value for an unknown key.** `_vocab_choice` selects
  `maxi(0, keys.find(current))`. A key this build does not know displays as index 0, "None / Unclaimed". That
  breaks the no-value rule and `SAVEFILE_COMPAT.md` §9.2 (*"a reader MUST NOT substitute a default silently"*).
  G1 makes this matter, because an older build opening a G1 save would show it. Fix in G1-3: an unknown key
  shows as "Unknown form (`<key>`), from a newer build", which is kept and not selectable as a choice. The
  Identity row's `d.get("government", "monarchy")` default has the same smell; it becomes absent-means-dashed.
- **The right dock shows the key, not the label.** It uses `String(roster.get("government","?")).capitalize()`.
  For the new keys that reads correctly ("Merchant Republic") only by accident. G1-3 reads the vocabulary label.
- **`_manpower_probe.gd`'s `GOVS` lists the eight forms by hand.** It stays the reference eight on purpose, or
  it gains the new forms in G1-3's probe and says so. Either way it must not silently become a partial list.
- **Military prose.** The CIVIL ▸ Military note's examples ("a kin-based chiefdom … a slave-holding empire")
  stay true. Any new sentence about forms must state what the rows are: authored orderings.

---

## 5. What each form changes

**In the faction, economy and era-band computations.** A form changes exactly what its two rows (three for the
khanate under G1-5) feed, and only for a faction **set to it by hand**. With §4.3's guard, no generated faction
ever defaults to one.

| Computation | Through | Changes when a faction takes a new form |
|---|---|---|
| Standing army, professional core | `GOVERNMENT_EXTRACTION` → `state_capacity` → `fiscal_extraction_efficiency`, `professionalization` | yes, that faction only |
| Levy (emergency mobilization) and field army | `state_capacity` → `LEVY_STATE`; `GOVERNMENT_LEVY_BONUS` (khanate, G1-5) | yes, that faction only |
| War duration and the force ladder | `campaign_capability(state_capacity, …)` | yes, that faction only |
| **Era row** (`era_for`) | `state_capacity` splits rows within an α bracket | yes. For example, a `city_league` or `feudal_realm` faction at Kingdom A's inputs reads *Late antique / early medieval* where `monarchy` reads *High medieval* (computed, §7.1) |
| Era-band verdicts and citizen population | `CITIZEN_SHARE` → `citizen_fraction` | yes, that faction only |
| Other factions' numbers | `world_land_reference` sums land and population, not government | **no** |
| Garrisons, conflict side readings | read `standing_army` | yes, that faction's |
| Economy (`civ_faction_aggregates`: tax, sectors) | no government input | **no**. Coupling it is Q8, default no |
| Relations, power axes, belief, town layout, GeoJSON | no government input | **no**. Q8 |
| Timeline year records | `YearFactionInstitutions.government` stores the key verbatim | key recorded; a past year reads its recorded form (`resolve_institutions`) |

**Per form, in words.** Each line is what the rows imply at Kingdom A's inputs (75 % agricultural, reach 0.20,
poor roads). Figures are computed (§7.1), shown only to compare forms, and are not targets.

- **Merchant Republic.** It reads exactly as `republic` (standing 10 584, levy 45 409). It differs in name and
  grouping only until a later economy coupling (Q8). That is disclosed in its picker hint.
- **City League.** A weak centre: standing 8 277, levy 34 939, *Late antique / early medieval*. Lower than
  `monarchy`'s 9 661 and *High medieval*.
- **Sacral Monarchy.** Standing 10 123, a little above `monarchy`, with the same citizen body. Its religious
  coupling (the research document's `god_king`/`revealed_law`) is **not** modelled in G1 (Q8).
- **Holy Order.** It reads as `oligarchy` on both rows (standing 10 123, citizens 59.6 %).
- **Feudal Realm.** The same extraction as `city_league`, with a monarchy's citizen body. Standing 8 277;
  mobilization 4.7 % of citizens, *below* its band, which is reported and not tuned. The specification's
  *"Feudal obligations were often ~2 months"* is a duration claim. No government enters the duration curve
  except through `state_capacity`, so this is disclosed, not modelled.
- **Nomadic Khanate.** Standing 7 816 and a low levy (3.4 % of citizens, below its band) without G1-5. With
  G1-5, the levy rises and the standing army does not change.
- **Military Regime** (optional). Standing 11 046, second only to `empire` (11 968). The citizen body is
  `empire`'s, and standing reads 2.23 % of citizens, *above* the High-medieval band. That is reported, as
  `empire`'s own 2.42 % *above* already is.
- **Tributary Hegemony** (optional). Standing 9 200; `empire`'s citizen body.

---

## 6. UI: the faction hub, on desktop, tablet and phone

**Where it lives.** The Government row on the hub's Identity tab (`FACTION_HUB_DESIGN.md` §3.5:
`civ_government_vocabulary`, `civ_set_faction_field("government")`, E). The right-dock Faction card keeps its
read-only Government field. No new window and no new menu item.

**Vocabulary call.** `civ_government_vocabulary` returns `{key, label}` today. G1-3 adds two members per row,
both additive:

- `group`: one of *Kin-based*, *Monarchic*, *Civic and mercantile*, *Religious*, *Imperial and military*.
- `hint`: one line, the research document's "why distinct" text, plus what the model reads. Examples: "Reads
  as Republic in the manpower model; the trade framing is not modelled yet", or "Mobilises more of its free
  horsemen (G1-5)".

Every row also gains `reference: bool`, true for the reference nine, so the shell can say "not offered as a
default for new factions" without hard-coding a list. Old callers that read only `key`/`label` are unaffected.

**The picker.** A popup choice (`DccWidgets.choice`) with group headers as disabled separator items, in
vocabulary order inside each group. The stored keys and their order never change; only the display groups
them. Below the picker sits the selected form's `hint`, the same pattern as the Ag. technology row's hint.

| Form factor | Rule |
|---|---|
| Desktop (window 960 × 640, min 680 × 440) | The Identity row as today. Popup list items at `role_px("row_min_h")`. 17 entries plus 5 headers fit without a scrollbar at the minimum window height, or the popup scrolls vertically, never horizontally. The hint wraps inside the pane width |
| Tablet | `DccTheme.is_tablet()` metrics (`role_px`, not literals), 44 px `btn_min_h` (`FACTION_HUB_DESIGN.md` §3.3) |
| Phone (393 × 852 dp reference) | Every popup row and the picker itself is at least **44 px** (`PHONE_TAP_MIN`, `dcc_theme.gd`). One scroller per screen. No row may push the Identity pane's minimum width past the screen: the `ScrollContainer` minimum-size fold in `MISTAKES.md` is the known way to break this. The hint uses wrapped `autowrap`, never `clip_text` (`MISTAKES.md`: `clip_text` collapses the minimum width to 1) |

**The Military tab.** On a faction set to a new form, the "Who the bands are measured against" group adds one
line: the form's citizen share, its anchor row ("the share a Republic confers"), and that the row is an
authored ordering. For the khanate without G1-5, it adds "Universal mobilisation of free horsemen is not
modelled yet."

**Unknown keys.** §4.4: an unknown key shows as itself, with its reason, and is never shown as "None /
Unclaimed".

---

## 7. Save format, and re-baseline impact

### 7.1 How the figures above were computed (the replica)

The figures in §3.3, §4 and §5 were computed with a scratch Python replica of `military_drivers`, `era_for`,
`civ_military_manpower` and the `SOLDIER_UPKEEP_BY_BRACKET` derivation rule. It was written for this pass
and kept out of the repository, in the session scratchpad as `gov_replica.py`. **Before any figure was taken
from it, it was checked against the code's own pinned literals.** It re-derived the upkeep table exactly
(`[1.12, 0.7983, 1.8153, 2.2214, 1.7696, 1.0714, 1.0714]`) and reproduced Kingdom A (standing 9 661, levy 41
221, field 15 870, *High medieval*) and Kingdom B (25 750, 98 889, 47 368, *Military-fiscal state*), matching
`the_citizen_ruling_moves_no_headcount`.

It is a replica, not the harness. Its job here is to size the hazards. G1-0 and G1-1 confirm every movement
claim in the real code, on real worlds.

### 7.2 Save format: additive only, and old saves reload as saved

- **Where a government is stored.** `entities/factions.json` (`SAVEFILE_COMPAT.md` §9.2) and
  `history/timeline.json`'s `years[].institutions[].government` (§10.1) store the key as a string. §9.2 already
  says *"An unrecognised key MUST be preserved on write-back"*, and `project_bridge.rs` reads and writes
  `government` verbatim. **No new key, no new file and no `format_version` bump.** G1 adds vocabulary, not
  schema.
- **Old saves reload as saved.** A pre-G1 save holds only the reference nine and reads byte-identically. Its
  generated output is unchanged, because §4.3 keeps fresh factions on the reference eight. G1-4 proves this with
  a **real** pre-G1 archive (`git show <sha>:` of a committed fixture, not a fixture of empty collections:
  `MISTAKES.md`, backward-compatibility row). The archive must open, resolve and re-serialise `factions.json`
  byte-identically.
- **A G1 save opened by an older build** keeps the key on write-back. But that build's manpower reads it
  through the documented fallbacks: `government_extraction` → 0.15 (`chiefdom`) and `citizen_share` → 0.90.
  Its picker shows "None / Unclaimed" (§4.4: that build cannot be fixed). G1-4 writes this into §9.2 as a
  forward-compatibility note, the same way `FH_R2_PLAN.md` §8 disclosed unknown religion keys.
- **Hand-edit validation.** `set_field`'s `"government"` arm accepts any `CIV_GOVERNMENTS` key, so the new
  forms become settable once appended, with no other change.

### 7.3 Which goldens, pins and harness columns would move

**With §4's three guards in place, G1-1 and G1-2 are projected to move zero bytes of generated output, by
control flow.** No fresh faction can take a new form. The upkeep and modernisation derivations read only the
reference rows. No other consumer reads a government. This is a projection until G1-0's columns confirm it.

| Pin, golden or column | Moves? | Kind of change |
|---|---|---|
| JS-captured goldens (`cartalith-civ/tests/golden_parity_*`, urban goldens) | no | none read a government (grepped) |
| `roster.rs` `vocabularies_are_the_reference_lengths`: `CIV_GOVERNMENTS.len() == 9` | yes | deliberate literal update to 15 (core) or 17 (with the optional pair), a `RE-BASELINED` comment, plus a new literal pin `CIV_GOVERNMENTS_REFERENCE_LEN == 9` |
| `GOVERNMENT_EXTRACTION`/`CITIZEN_SHARE` array types `[(&str, f64); 9]` | yes | type length only. The coverage tests compare against `CIV_GOVERNMENTS.len()` and pass once both tables are appended |
| `soldier_upkeep_is_derived_from_the_era_table` (`govs.len() == 8`), `soldier_upkeep_by_alpha_literals` | **no**, by §4.2's bound | without the bound: the whole upkeep table and both worked examples (§4.2 figures) |
| `citizen_fraction_widens_…` identity, `worked_example_kingdom_a`/`_b`, `the_citizen_ruling_moves_no_headcount`, `a_median_polity_of_each_bracket_lands_in_its_own_band` | **no**, by §4.1's floor and bound | without them: §4.1's figures |
| `default_government_cycles_every_real_form`, `default_government_never_returns_none`, `civ_roster_bridge` index-1 (`chiefdom`) and index-8 (`city_state`) pins | **no**, by §4.3's bound | without it: those pins stay, but indices 9 and above move |
| `fh_r2_baseline.rs` roster column (`AT_CURRENT`, hashes every faction's government) | **no**, by §4.3's bound | without it: on the 10 24-faction worlds, 16 of 24 factions (core six) or 8 of 24 (all eight) change government (projected, §4.3). The 10 6-faction worlds cannot move either way |
| A **manpower** column (standing, levy, field, citizen fraction, era, per faction, by `to_bits`) | none exists today | G1-0 adds it. It is the column that catches §4.1 and §4.2, which the roster column cannot |
| `_manpower_probe.gd` | no | its `GOVS` is the reference eight (§4.4) |

**What does move, deliberately, and only on a user's edit.** A faction set by hand to a new form changes its own
manpower, garrisons and conflict readings (§5). That is the feature, not a re-baseline.

---

## 8. Milestones

Each milestone ships alone, after the owner approves it. The app builds and `cargo test --workspace` passes
after each. Every touched `.gd` file plus `shell/app.gd` is parse-checked, because the Rust suite cannot see a
broken shell (`MISTAKES.md`). Every new function, constant and test is annotated for an auditor (Ruling BK):
what it does, why, what it must never do, and the source of each constant.

| ID | What | Bars (numeric, each one measured, not asserted) | Depends on |
|---|---|---|---|
| **G1-0** Measure | A test-only ignored harness, `g1_baseline.rs` beside `fh_r2_baseline.rs`, over the **same 20 worlds**: 5 seeds × {800 km plain, 40 075 km wrapped} × {6, 24 factions}. Columns: roster (government only) and **manpower** per faction (standing, professional core, levy, field, field and levy duration, citizen fraction, era name, both verdicts), each an FNV-1a over `to_bits`/`Debug`, recorded on the pre-G1 tree and pinned as literals. Plus the replica's checks as a unit test: the upkeep table and Kingdom A/B re-derived independently of the constants | (1) 20 of 20 worlds recorded, every column non-empty, with ≥ 6 factions carrying a non-zero standing army on each world (this guards against silently empty output). (2) Positive control: setting faction 1 `monarchy → empire` in a scratch run moves the manpower column on ≥ 19 of 20 worlds (exemptions named), and the roster column on 20 of 20; restored afterwards. (3) The roster column agrees with `fh_r2_baseline`'s on all 20 worlds. (4) Runtime ≤ 2 × the FH-R2 harness's (about 17 s, one run, 2026-10-12, `fh_r2_baseline.rs`), median of 3 runs alone, min..max given | owner approval |
| **G1-1** Guards and the six core forms | `CIV_GOVERNMENTS_REFERENCE_LEN = 9`; bound `civ_default_government`, the upkeep derivation and the modernisation identity to the reference rows (§4.1-§4.3); the 0.30 floor test; append the six core forms (§2) and their 12 rows (§3.1-§3.2) with doc comments naming each anchor or judgement; theocracy's hierocracy note | (1) **0 of 20 worlds move** on either G1-0 column. (2) Mutations, run with the project's Python mutation discipline and restored in a `finally`: each of the 12 new literals → its own pin red (12 of 12 killed); removing the default-pick bound → G1-0's roster column moves on 10 of 10 24-faction worlds; removing the upkeep bound → `soldier_upkeep_is_derived_from_the_era_table` red; setting a new `CITIZEN_SHARE` row to 0.29 → the floor test red. (3) Kingdom A/B literals 9 661 / 25 750 unchanged. (4) Plausibility on the harness worlds, faction 1 forced to each new form in turn (6 × 20 readings): standing < field < levy in 120 of 120; each new form's standing within 0.5×-2× of its anchor row's (§3) on every world. Verdict counts are reported, not barred | G1-0 |
| **G1-2** The optional pair | `stratocracy`, `tributary_hegemony` and their 4 rows, both at `CITIZEN_SHARE` 0.30 | As G1-1: 0 of 20 worlds move; 4 of 4 literal mutants killed; 40 of 40 forced readings ordered | G1-1; Q3 |
| **G1-3** Hub UI | Vocabulary gains `group`, `hint`, `reference`; grouped picker with hint; Military-tab line; unknown-key display (§4.4); right dock reads the label; `_factionhub_probe` extended | (1) Every one of the 15 (or 17) keys selectable and persisted, read back from `get_factions` (15 of 15). (2) Phone at 393 × 852: every picker and popup row ≥ 44 px, measured on the live nodes; Identity pane `get_combined_minimum_size().x` ≤ screen width; 0 horizontal scrollbars. (3) Tablet uses `role_px` (no literal row heights in the diff). (4) An unknown key (`"future_form"`) shows its own name: 0 rows read "None / Unclaimed" for it. (5) Windowed probe for any pixel claim (`MISTAKES.md`: `update()` is a no-op headless). (6) Parse-check every touched `.gd` and `app.gd` | G1-1 |
| **G1-4** Save compatibility | `SAVEFILE_COMPAT.md` §9.2 forward-compatibility note; round-trip tests | (1) A real pre-G1 archive opens and re-serialises `factions.json` byte-identically. (2) A G1 save with each new key in turn round-trips, 15 of 15 (or 17 of 17). (3) A timeline year recording `merchant_republic` reads that form back through `resolve_institutions` | G1-1 |
| **G1-5** Khanate levy (optional) | `GOVERNMENT_LEVY_BONUS` with one row, its value derived by §3.3's rule and pinned with its derivation test | (1) 0 of 20 worlds move. (2) Every key except `nomad_khanate` reads exactly `0.0` (asserted over the whole vocabulary). (3) A median khanate's mobilization lands within ±0.5 percentage points of its era band's centre. (4) Its standing army is identical, to the bit, with and without the bonus | G1-1; Q5 |
| *(not proposed)* G1-6 Defaults | New forms join `civ_default_government` | Would move 8-16 of 24 factions on large worlds (§4.3): an output change needing its own ruling and a `DECISIONS.md` §7p re-baseline. Listed only so the omission is explicit | Q6 |

**Note on G1-1 bar 4's ordering half.** It did not hold as written, and not because of G1's rows: the reference's own forms broke `standing < field < levy` on the same worlds, because the standing budget was linear in the ecological factor. G1-1 asserted a one-directional guard instead (no form breaks an ordering its anchor keeps). Owner Ruling DE (2026-10-25, `ECOLOGICAL_ARMY_TERM_SCOPE.md`) fixed the model, and `g1_forced_forms_are_ordered_and_near_their_anchor` now asserts the ordering on every forced reading as well (160 of 160 G1, 160 of 160 reference, measured). Status is in `STATUS.md`.

Model tier is decided per lane when a milestone is scheduled. G1-1's guards and G1-3's shell work are routine
for a strong builder. A separate verifier should re-run G1-1's mutations.

---

## 9. Owner questions, each with a recommended default

| # | Question (research document decision in brackets) | Recommended default |
|---|---|---|
| **Q1** | Approve the six core forms (keys, labels, append order §2)? | **Yes**, appended after `city_state` in the order in §2 |
| **Q2** | Accept authored, uncalibrated manpower rows, the 12 in §3.1-§3.2, each anchored or labelled? [D6] | **Yes**. Each row's doc comment says "authored ordering" |
| **Q3** | Build the optional pair (`stratocracy`, `tributary_hegemony`)? And at what citizen share for `stratocracy`? [D6's flag] | **Yes, in G1-2, both at 0.30**, so §4.1 does not arise. A stratocracy below 0.30 means a re-baseline under its own ruling |
| **Q4** | Prevent §4.1-§4.2 by freezing both derivations to the reference rows (and the 0.30 floor)? Or re-derive over every form and accept the re-baseline (Kingdom A −5.95 % with all 16 rows; more with a sub-0.30 row)? | **Freeze.** New forms are priced by the calibration, not into it |
| **Q5** | Add the khanate's levy knob (G1-5)? | **Yes**, as its own milestone after G1-1, with its value set by §3.3's rule. Without it the form's levy is the lowest of any state-level form, which mislabels it |
| **Q6** | Keep the new forms out of `civ_default_government`? | **Yes.** Ruling CJ already requires it. No inclusion pass is proposed |
| **Q7** | `theocracy`: relabel as Hierocracy, or note it in the doc only? | **Doc only.** The key and label stay |
| **Q8** | Should a government also reach the economy (tax), relations, religion (`sacral_monarchy` ↔ `god_king`), or the timeline's collapse rules? | **No, none in G1.** Each touches reference-ported, golden-verified code and would be its own ruled row. The UI hints say what is not modelled |
| **Q9** | Group the picker by axis (§6), with the stored order unchanged? | **Yes** |

---

## 10. Risks

- **The tempting fix.** All three hazards fail loudly in existing tests. The danger is a builder who "fixes"
  the test by re-deriving. G1-1's bounds, the floor test and the mutation bars exist to make that impossible to
  do quietly.
- **Labels without behaviour.** `merchant_republic` reads exactly as `republic`, and `holy_order` as
  `oligarchy`, until Q8. This is the same honesty problem custom religions had before R2-3. The hint must say
  so (§6).
- **Two weakly grounded anchors.** `oligarchy`'s row is *"the least-grounded row in this table"* by its own
  doc, and `holy_order` inherits it. That inheritance must be stated, not hidden.
- **The replica is not the code.** It reproduced every pinned literal it was checked against (§7.1). But
  movement claims are confirmed only by G1-0 and G1-1 on real worlds.
- **Cross-lane drift.** `manpower.rs` and `roster.rs` have been edited by other lanes before. Re-open each
  symbol at G1-1's start rather than trusting this document's description of it (`MISTAKES.md`, "re-open a
  backlog row at its symbol").
