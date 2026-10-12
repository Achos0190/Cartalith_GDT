<!--
IMPORT NOTE (added on import; everything below the closing marker is the source, verbatim, not edited)

Source: the owner-requested Claude Docs research document "Religion & Government Expansion",
  https://claude.ai/artifact/X8PudUqgDYC6nMBidaorA7
  (Claude Docs id f3fa7bbe-666a-4228-b525-71241ea43df4, tab "Religion & Government Expansion"),
  revision 6, authored 2026-09-23.
Imported: 2026-10-12, under Ruling CG item 3 (LARGE_ITEM_RULINGS.md), which approved
  FH_R2_PLAN.md question Q0 ("import the research doc into the repo").
Method: the Claude Docs `export` tool, format "markdown" (24 773 bytes, decoded from its base64
  and checked by byte count and UTF-8 validity). Verbatim, not edited. The exporter rendered the
  byline's date chip and mention chip as plain text ("Sep 23, 2026 · @Vincent").

Read with FH_R2_PLAN.md. That plan re-checked this document against the code on 2026-10-12 and
found two of its claims stale: S1's and G1's "changes generated output? No" (the default-pick
functions it calls unwired were wired 2026-09-23), see FH_R2_PLAN.md §2.1 and §4.1. This
document's own status sentences (what is built, what is pending) are dated 2026-09-23 and are
not status: cartalith-native/docs/STATUS.md is (CLAUDE.md, "One place holds state").
-->

# Religion & Government Expansion

Sep 23, 2026 · @Vincent

**Recommendation: build both mechanisms together — intra-religion pantheons, and inter-religion coexistence.** The engine already has the data structure a rich multi-faith model needs: `belief::SettlementReligionState` stores a full per-settlement distribution across every religion, not a single flag. What it lacks is the *dynamics* that let that structure hold more than one faith at once.

- **The container already exists.** `CivData::belief` holds one `SettlementReligionState` per settlement, and `get_settlements()` already sends `religion`/`adherents` to the shell.
- **A naive "just display the existing diffusion state as more temples" approach will not work, and the code proves it.** `belief_step` uses a conformity exponent that pushes every town toward a single faith. A real existing test, `belief_has_no_syncretic_equilibrium`, confirms even a town sitting between two equal rival neighbours ends up over 99% one faith — its own failure message says that if that figure ever drops below 0.99, a real coexistence mechanism has arrived and every surface assuming a single winner needs revisiting.
- **"Many temples" means two different real-world things, and the model needs both:**
  1. *Many gods within one religion* — a Greek city's temples to Athena, Hephaestus and Poseidon inside one civic religion. Needs no new dynamics, just a siting rule keyed on each deity's domain.
  2. *Many religions within one settlement* — Quanzhou, Dura-Europos, Karakorum, Venice. Minorities survive there through birth-based retention, steady trade-diaspora replenishment, and a ruler's toleration policy — all currently missing from the belief model.
- **The existing 8 religions contain almost no exclusive religion.** Six of seven read as additive/polytheist/folk traditions. Exclusive, bounded minorities are exactly what form separate quarters, so the vocabulary is missing the type that produces the picture you described.
- **The structural space itself is not large** — religious-studies typologies land at roughly 14–15 archetypes. The "thousands of gods" diversity is *instance-level* (named deities with domains), which has to be generated per world, not hand-typed as archetypes.
- **Proposal: 7 new religion archetypes (15 total including "none"), a generated pantheon/deity layer on top, 6 core + 2 optional new government forms, and one new per-faction axis (religious policy) deciding whether trade-hub pluralism survives.**

## What exists today in the engine

### Belief layer (`cartalith-civ/src/belief.rs`, 2,220 lines)

- **Compatibility (`compat()`):** five components (L, K, M, S, Q), but only K (terrain-domain match, via `CIV_RELIGION_DOMAIN`, 5 rows) is evaluated. `ancestor_rites`, `flame_creed` and `none` are deliberately unthemed. 31 of the 56 religion-culture pairs return a neutral 0.5.
- **Milestone 1 is fully built:**
  - `SettlementReligionState` holds shares summing to 1, indexed by position in `CIV_RELIGIONS`, slot 0 = unaffiliated.
  - `BeliefNetwork` is road links weighted by population and a carriage-decay curve (220 km cutoff).
  - `belief_step` is a three-term logistic: exposure, compatibility, conformity frequency.
  - `belief_seed` seeds every settlement wholly into its faction's religion.
- **Measured properties already in the file:** a fresh world is all "none" and stays that way (a fixed point); there is no stable mixture — takeover happens in 17–25 years or never.
- **Missing terms, each stated as missing:** competition, retention, missionary/institutional effects, and sea lanes.
- **A useful find:** `trade::TradeFlow` already carries `from`/`to` settlement indices, including sea-mode flows — a maritime exposure term can be built from trade flows without solving a separate sea-route endpoint problem.
- Religion array size derives from `CIV_RELIGIONS.len()`, so appending religions grows every share array automatically — no size-coupling risk.

### Physical faith sites (`cartalith-urban`)

- Site count: `clamp(round(households/500), 1, 4)`, zero for a hamlet or when the rite is "none."
- **All sites in a town share one rite today.** `build_faith_sites` takes one `faith` string; every site gets it — no town has ever had two rites.
- The existing rites (`church`, `temple`, `shrine`, `mosque`, `orthodox`) are architectural forms, not religions.
- **Civ religion never reaches town generation.** `urban_adapter.rs` passes `faith: None`, so every town takes its culture profile's hardcoded default. `RC_ENGINE_CHANGES.md` §6r.4 records this same gap in the source HTML: *"faith is arguably the faction religion's job — that decision is open, in both codebases."*
- Useful placement precedent for a future minority quarter: worship sites are excluded from harbour/craftriver districts and near the quay/river; sites sit ≥180m apart; the first sits 70–230m from the market.

### Governments

- `GOVERNMENT_EXTRACTION`/`CITIZEN_SHARE` in `manpower.rs` are keyed by the `CIV_GOVERNMENTS` strings; tests force both tables to match the vocabulary's length — every new government needs two authored rows.
- **A real hazard:** `CITIZEN_MODERNISATION` derives as `CITIZEN_CEILING − min(CITIZEN_SHARE)`. A new government below the current floor (empire's 0.30) silently shifts every government's era band.
- Only two worked examples exist for calibration (`MILITARY_MANPOWER_SCOPE.md`) — new rows would be authored orderings, not calibrated values.

### Prior art and stale documentation, found in passing

- The source HTML project has nothing on religion/government beyond what's already ported — only §6r.4 (faith option never set) and §6q.5, which lists institutional quarters as *not built*: "cathedral close as a precinct … Jewry by the castle, the foreign-merchant factory at the quay, friaries in the suburbs." That's direct prior art for a minority-faith-quarter design.
- The approved design canvas (`Religion.dc.html`) already has a **TYPE column** (organized/folk/cult/heresy) waiting for real data — the shell currently shows `—` because no such field exists on `CIV_RELIGIONS` yet.
- **`STATUS.md` is stale on religion in three places**, found while researching (not this task's job to fix, flagged for the coordinating session): it claims the belief model "exists only as a name" when it's fully built and wired; it cites a file length out of date; and it calls the religion screens "blocked" when another 2026-09-03 batch already shipped them.
- `roster.rs` currently carries uncommitted work from earlier this session (`civ_default_religion`/`civ_default_government`, built and tested but pending a separate wiring pass) — any new religion/government vocabulary must keep those functions' literal test pins (7 religions, 8 non-none governments) in sync.

## Real-world grounding: the structural axes

A flat list of names hides that religions differ along several independent axes. These are the ones that actually change simulation behaviour.

| Axis | Values | Basis | What it does in the simulation |
| --- | --- | --- | --- |
| **1. Object of worship** | monotheism · henotheism/monolatry · polytheism · dualism · animism · ancestor veneration · non-theistic soteriology · ruler cult | Tylor (animism, 1871); Max Müller (henotheism) | Pantheon size — how many distinct cult sites one religion builds |
| **2. Exclusivity** | exclusive (belonging rules out others) vs. additive/cumulative (worship many; cults stack) | Jan Assmann's *Mosaic distinction* — polytheisms make gods translatable (Melqart = Heracles), which is how additive traditions absorb each other | **The key pluralism axis.** Additive meets additive → absorption/syncretism, low competition. Exclusive meets anything → boundary, quarter, strong competition |
| **3. Membership mode** | birth/ethnic · universal/missionary · initiatory · civic | "World" vs. "ethnic" religion; Weber | Retention (birth religions are sticky), proselytism, whether a minority persists |
| **4. Locus** | household/lineage · place · civic/state · portable | Jonathan Z. Smith, *Here, There, and Anywhere* (2003) | "Anywhere" religions travel with merchants — they're the trade-hub religions |
| **5. Institutional form** | hierarchical clergy · temple priesthood · monastic · congregational · specialist (shaman) · none | Weber/Troeltsch church–sect; Whitehouse's *Modes of Religiosity* (2004) | Site threshold, building grammar, congregation size |
| **6. Great vs. little tradition** | official/elite vs. folk/lived | Redfield (1956); "lived religion" | The same town can hold an official cult and folk practice at once — another source of plural sites |
| **7. Relation to the state** | established-exclusive · established-tolerant · non-established plural · persecuting | Ottoman *millet*; Islamic *dhimma*; Roman toleration; the Venetian ghetto (1516) | **Decides whether hub minorities survive** — a per-faction policy, not a property of the religion |

Also relevant as framing: Jaspers' Axial Age (the turn toward transcendent, universal, portable religions); Stark & Finke's "religious economies" (pluralism in competitive markets). Norenzayan's "big gods" link between moralising high gods and large polities is contested — useful only as a soft default for government coupling, never a rule.

### The trade-hub evidence behind the coexistence design

- **Karakorum** (William of Rubruck, 1254): twelve "idol" temples of different nations, two mosques, one church — under explicit non-establishment.
- **Dura-Europos, 3rd century CE:** a synagogue, a house-church, a Mithraeum, and temples of Bel, Artemis and other Palmyrene/Greek gods in one frontier trading town.
- **Quanzhou, Song–Yuan:** the Qingjing mosque, a Tamil Shaiva temple inscription (1281), Nestorian and Catholic gravestones, and the Cao'an Manichaean shrine — where Mani appears as the "Buddha of Light," syncretism made visible.
- **Ostia:** more than a dozen mithraea plus a synagogue — the portable "anywhere" cults of a port.
- **Mazu (Tianhou) temples:** built wherever Fujianese merchants settled — the diaspora temple as an institution.

### Where gods get sited inside one town (the rule for polytheist siting)

- The Hephaisteion overlooks the Athenian agora's metalworking quarter.
- The Temple of Portunus stands at Rome's river harbour, in the Forum Boarium.
- A poliad deity sits at the market or acropolis; sea gods at harbours and headlands.
- Medieval monotheist towns achieve the same effect with dedications to many different saints — many shrines inside one religion.

## The synthesis: two mechanisms, one container

### Mechanism A: intra-religion multiplicity (pantheons and dedications)

No new dynamics needed. A religion's structural profile says how many distinct cult sites it produces and where each belongs:

- **Polytheist/henotheist:** one site per major deity, sited by domain — poliad/supreme god at market or civic core (today's rule); sea god at the harbour district; craft god in artisan quarters; river god at the ford/bridge; harvest/earth deity toward the gate to the fields. This needs an exception to the current "never on the working waterfront" exclusion — right for a parish church, wrong for a harbour god.
- **Monotheist:** parish churches with different saint dedications (as today), plus one principal church.
- **Animist:** extramural sacred groves and springs.
- **Ancestral:** no public building below a size threshold, then lineage halls.

**Domains must come from the world, not a hand-list** — a sea god only where the town is coastal, a river god named after a river the engine actually generated. Same "never fabricate a verdict without a basis" discipline this project already applies elsewhere.

### Mechanism B: inter-religion coexistence

Three additions to `belief_step`, each already has a slot in the owner-supplied paper this engine implements — none is invention:

1. **Retention split.** Today every religion loses a flat share each year and wins it back through attraction. Change: out-flow becomes `rate × (1 − retention) × share`, with retention derived from the axes (birth > universal-doctrinal > initiatory > folk) — no per-religion magic number. Effect: a birth-membership minority stops bleeding away, which alone breaks the current fixation result for boundary religions.
2. **Competition derived from axes, not a hand-typed matrix.** Additive × additive → weak (absorption); exclusive × proselytising → strong; birth religions → weak toward everyone (they don't proselytise). Same derivation discipline `compat()` already uses.
3. **Trade-diaspora influx.** Each year a settlement receives inflow proportional to trade volume with its partners, weighted by the partners' own religious shares — using `TradeFlow`'s already-present endpoint indices, including sea routes. Retention plus influx together produce a stable minority share roughly equal to influx over out-flow: **hubs hold communities; an isolated village still converges on one faith** — the historically correct contrast.
4. **Toleration.** Covered by the new religious-policy axis (next section) — persecution raises minority out-flow, tolerant regimes lower it.

**Data shape:** keep `share` as the primary-affiliation partition. Don't add a second multi-faith structure — additive participation (a Christian merchant still making offerings at a harbour shrine) is handled on the building side by Mechanism A. A separate co-affiliation vector is a later option, not a prerequisite.

**Site allocation for a plural settlement:**

- Distribute the town's faith sites across religions by adherent share.
- Give a site only above a congregation threshold that depends on institution type — small for congregational/initiatory forms, parish-sized for hierarchical ones.
- Place minority/exclusive sites in quarters — at the quay as a foreign-merchant factory, or by the castle as English Jewries were — following the source project's own §6q.5 prior art.
- The majority faith keeps the current core-siting rule.

**One determinism caution, flagged for whoever builds this:** the belief simulation runs on demand and isn't persisted today. If town layouts read it directly, the same town could render differently depending on whether the player has run the simulation yet — needs a decision (D5 below): run a fixed number of years at generation time, or persist the state.

## Proposed taxonomy

### The existing 8, classified (a draft for your authoring, not a derivation)

The shell's own design doc treats a taxonomy inferred from the eight labels as fabrication unless authored — so treat this table as a starting draft for you to confirm or edit, not a finished classification.

| Key | Object | Exclusivity | Locus | Proposed TYPE | Confidence |
| --- | --- | --- | --- | --- | --- |
| `sun_cult` | henotheism | additive | state | organized | soft — could be monolatry |
| `earth_mother` | goddess-centred polytheism | additive | place | folk | medium |
| `sea_lords` | polytheism | additive | place (coast) | folk | medium |
| `sky_pantheon` | civic pantheon | additive | civic | organized | strong |
| `ancestor_rites` | ancestor veneration | additive | household/lineage | folk | strong |
| `flame_creed` | dualism (fire temples) *or* confessional monotheism | exclusive-leaning | civic | organized | **softest — your call.** "Creed" implies a confession; this is the only existing entry that may already be exclusive |
| `old_gods` | pre-institutional polytheism | additive | place | folk | medium |

**The finding that matters:** at most one of the seven existing religions is exclusive, none is universal-missionary, none is portable/"anywhere," and none is a ruler cult. Mechanism B has nothing to act on until those types exist in the vocabulary.

### 7 new entries (appended, never reordered — saves store keys by string)

| Key | Label | Type | Structure | Real-world model | Terrain verdict | Default-eligible? |
| --- | --- | --- | --- | --- | --- | --- |
| `one_god` | The One God | universal monotheism | exclusive · universal · portable+civic · hierarchical clergy | Latin/Orthodox Christianity | None — universalism isn't landscape-bound | yes |
| `revealed_law` | The Revealed Law | universal monotheism, legal-scriptural | exclusive · universal · portable · congregational-scholarly | Islam | None — "desert breeds monotheism" is a discredited determinism, refused deliberately | yes |
| `covenant_folk` | The Covenant | ethnic monotheism | exclusive · birth, non-proselytising · portable/diaspora | Judaism, Samaritans | None — law and kinship, not landscape | yes, but rare |
| `the_path` | The Middle Path | non-theistic soteriology | exclusive for monks, additive for laity · universal · monastery | Buddhism, Jainism | None — spread along trade routes | yes |
| `mysteries` | The Mysteries | initiatory salvation cult | additive · initiatory · small cult-group | Isis, Mithras, Eleusis | None | **No** — mystery cults were voluntary associations, never establishments; exclude from default-assignment |
| `spirit_ways` | Spirit Ways | animism + shamanism | additive · birth-place · specialist (shaman), no temples | Siberian/steppe practice, early kami worship | None, specifically — bound to every place equally, so any one terrain key would be fabricated | yes |
| `god_king` | The Divine Throne | ruler cult / sacral kingship | additive (a loyalty rite, tolerates the rest) · civic · state priesthood | Pharaonic kingship, the Roman imperial cult, the Sapa Inca | None — political, not terrain | yes; couples naturally to empire/sacral-monarchy governments |

**Considered and rejected:** a Manichaean-type universal dualism (duplicates `one_god` plus whatever you decide for `flame_creed`); a heresy/sect entry (that's an *event* — a schism spawning a new instance — not an archetype, better left for the optional world-instanced-religions stage); a merchant-patron cult like Mazu (that's a deity *domain* inside a pantheon, not a whole religion).

**One optional 8th, recommended against as a fixed entry:** `joined_teachings` (an explicit syncretic umbrella, like China's *sanjiao*). Better modelled as an *outcome* of weak competition between additive religions than as a pickable label — add only if you specifically want a player-selectable state syncretism.

**Why 7, not 3–4 or 30:** each fills an axis combination the current 8 can't express; together they cover every value on axes 1–5. More would only differ in flavour. One real cost: with all 7 new entries deliberately unthemed (no terrain verdict), the compatibility model's neutral-pair share rises from 31/56 to 80/105 — meaning the new religions are indistinguishable to today's `belief_step` except by exposure. **The retention/competition work in Mechanism B isn't optional polish; it's the only way the model can actually tell these religions apart.** The "thousands of named gods" diversity belongs in a later generated-deity layer, not in this archetype list.

### Government forms: 6 core + 2 optional

The current list mixes two axes — constitution (rule by one/few/many) and territorial scale (city-state vs. empire). Recommend documenting that rather than restructuring (saves store keys).

| Key | Label | Real-world models | Why distinct |
| --- | --- | --- | --- |
| `merchant_republic` | Merchant Republic | Venice, Genoa, Novgorod | Trade-hub framing — customs/excise revenue, narrow patriciate over a broad free populace |
| `city_league` | City League | the Hanseatic League, Lombard League | Trade-hub framing — confederated cities, weak centre, no common treasury |
| `sacral_monarchy` | Sacral Monarchy | the Caliphate, Byzantium, Pharaonic Egypt | Ruler *is* the religious head — couples naturally to `god_king`/`revealed_law` |
| `holy_order` | Holy Order | the Teutonic Order, the Hospitallers | A military-monastic corporation rules |
| `feudal_realm` | Feudal Realm | Capetian France | Decentralised — revenue/troops sit with vassals, distinct from administrative monarchy |
| `nomad_khanate` | Nomadic Khanate | the Mongols, Turkic khaganates | Near-universal mobilisation of free horsemen — flagged: the existing two-knob extraction/citizen-share schema may not express this well |
| *optional* `stratocracy` | Military Regime | the Mamluk Sultanate, the Tokugawa bakufu | Its citizen share may fall below today's floor, which re-derives every other government's band — real ruling required |
| *optional* `tributary_hegemony` | Tributary Hegemony | the Aztec Triple Alliance | Tribute extraction from nominally autonomous subjects |

`theocracy` stays, re-documented as *hierocracy* (rule by a priesthood — the Papal States, prince-bishoprics).

### New small axis: religious policy (per faction)

Toleration is a property of the *ruler*, not the religion — historically the variable deciding whether Karakorum or Venice kept its minorities.

- `plural` — non-established, all cults licit (Mongol practice).
- `tolerant_establishment` — a state faith with protected minorities (millet, dhimma).
- `exclusive_establishment` — a state faith; others restricted.
- `persecuting`.

Feeds Mechanism B's retention/conversion terms as a modifier only — never sets shares directly ("the king converts ≠ everyone converts"). Also the natural home for deciding whether a faction's state religion is a hand-set flag or a derived plurality — currently an open fork in the project's own religion-diffusion design doc.

## Cost, staging, and what's needed from you

You can stop after any stage below — each is independently useful.

| Stage | Content | Size | Changes generated output? |
| --- | --- | --- | --- |
| **S1 Taxonomy as data** | Append 7 religions; a `ReligionProfile` table (object/exclusivity/membership/locus/institution/TYPE); fill the design canvas's TYPE column | small–medium | No — defaults stay "none" until wired |
| **S2 Coexistence dynamics** | Axis-derived retention, axis-derived competition, trade-diaspora influx from `TradeFlow` | medium | No — the belief layer is inert until run |
| **S3 Civ→urban faith wiring** | Religion→rite table; multi-site allocation by adherents; minority quarters; deity-by-district siting | medium–large | **Yes** — every town's faith sites change; needs its own ruling |
| **S4 New building forms** | Assembly hall, fire temple, monastery/stupa, cult hall, sacred grove, lineage hall | large | Yes |
| **S5 Pantheons and deities** | Per-world generated deities — domains from real terrain, names from culture pools. **This is the actual "thousands of gods" layer.** | medium | Cosmetic unless dedications drive siting |
| **S6 Religious policy** | The toleration vocabulary; wired into retention/conversion | medium | No, until wired into faction defaults |
| **S7 World-instanced religions** *(optional)* | Per-world religion instances (archetype + origin culture + name); schisms spawn "heresy" instances | large | Yes — touches the reference-ported relations bonus, a real golden-parity risk |
| **G1 Governments** | 6+2 vocabulary entries, 12–16 manpower calibration rows | small–medium | No, while unwired |

**Suggested order:** S1 + G1 together first (pure content, fills the already-approved UI slot, zero output change) — then S2 (the actual answer to "coexist") — then S3 (the visible payoff, needs your sign-off) — then S5/S6, with S4/S7 as budget allows.

### Decisions needed from you

1. **Approve the axes and the classification of the existing 8** — `flame_creed` is the one real judgement call: dualist fire-temple religion, or a confessional monotheism?
2. **Approve the 7 new religions**, and decide on the optional 8th (`joined_teachings`).
3. **A fixed archetype list now, with per-world instances later (S7) — or build instances from the start?**
4. **Authorise the civ→urban faith wiring (S3).** This changes every town's generated output — currently an open question in both this port and the source project.
5. **Belief determinism for town rendering:** run a fixed number of years at generation time, or persist the simulated state?
6. **The government list** — accept authored (not calibrated) manpower rows outside the original specification, and flag on `stratocracy`'s re-derivation hazard.
7. **Add the religious-policy axis?**
8. **Wire the already-built `civ_default_religion`/`civ_default_government` functions in** — without this, everything above stays invisible in a freshly generated world. *(Note: this was asked and answered separately earlier this session — wiring is already in progress.)*
