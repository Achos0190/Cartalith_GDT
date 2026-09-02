# Religion diffusion — a quantitative model, scoped from an owner-supplied paper

Owner-supplied design input, 2026-08-29: a research paper, *"Quantitative
Modelling of Religious Diffusion, Adoption, and Retention — A Mathematical
Framework for Cartalith."* Preserved verbatim in §1 for the same reason
`MILITARY_MANPOWER_SCOPE.md` preserves its owner-supplied specification
verbatim: **it is the only ground truth there is.** Nothing in the reference
implements this, so there is no source code to check a paraphrase against —
only the paper itself.

This document does not commit to building the whole thing. It exists to turn
a 35-section research paper into a bounded first slice, and to record the one
decision that has to be made before any of it, in the open, rather than
inferred later from a diff.

Four sections: §0 how this relates to what exists today and why it is new
scope, not a port; §1 the paper as supplied; §2 what the paper's abstractions
map onto in this port's actual data model, on inspection; §3 the milestones,
starting from the smallest slice that is still the paper's real architecture
rather than a toy.

---

## 0 · Relationship to what exists today, and why this is new scope

**Today, religion is one hand-picked categorical value per faction, and
nothing else.** `cartalith_civ::roster::CIV_RELIGIONS` is an 8-entry fixed
vocabulary (`sun_cult`, `earth_mother`, `sea_lords`, `sky_pantheon`,
`ancestor_rites`, `flame_creed`, `old_gods`, plus `none`), reference line
~14780. It is set by the player through the Faction Inspector's Religion
dropdown (`_civFeRel`, reference line 16254/16302) and by nothing else — no
generation pass, no simulation step, ever writes it. `roster.rs`'s own doc
comment on `CIV_RELIGIONS` states the reference's own history plainly: *"the
reference scoped FMG's full spatial religion-spread model down to exactly
this list, on purpose."* That sentence is not this port's inference — it is
the reference's own v1.10 changelog entry (line 14772): *"borrow-list #4,
'religions are a similar spread-model layer if wanted': scoped down to a
per-FACTION categorical 'state religion' attribute — FMG's religions are a
substantially larger [feature]."* And the reference's own scope-declaration
comment (line 23197) names *"per-settlement religious diffusion"* explicitly
among the things *"the simulation genuinely doesn't model."*

**The one place religion currently has any behavioural effect at all** is
`cartalith_civ::relations`: same-faith factions get a `+0.20` relations bonus,
different faiths a `−0.20` penalty, and `none` on either side is silence, not
division (`relations.rs:33`, `religion_term`). That is the entire footprint —
one scalar, symmetric, static, computed from two categorical labels.

So: the paper describes exactly the spread-model layer the reference author
looked at and explicitly declined. Building it is **new scope beyond the
reference, not a gap in the port** — the same category as `ECONOMY_SCOPE.md`
and `MILITARY_MANPOWER_SCOPE.md`, both of which found reference-absent
territory and built new subsystems on top of it deliberately, recording why
in the scope document itself rather than in `DECISIONS.md` (neither of those
two has a `DECISIONS.md` entry; the scope document *is* the record, and this
one follows that precedent).

---

## 1 · The owner's specification, reproduced as supplied

> # Quantitative Modelling of Religious Diffusion, Adoption, and Retention
>
> ## A Mathematical Framework for Cartalith
>
> **Status:** Research and model-design paper
> **Application:** Cartalith cultural, demographic, settlement-network and
> historical simulation
> **Domain:** Cultural evolution, religious studies, network science,
> mathematical sociology, population dynamics
>
> ### Abstract
>
> The geographical spread of religion is not adequately represented by a
> simple distance-based diffusion model. Religious systems propagate through
> human populations, social relationships, transportation networks, political
> structures and institutions. A population may encounter a religion through
> trade or migration without adopting it; conversely, a religion already
> established in a settlement may spread rapidly through family, prestige,
> conformity, institutional presence or political patronage.
>
> This paper develops a quantitative framework for modelling religious
> diffusion in Cartalith. The proposed model combines four established
> mathematical approaches: weighted network diffusion, cultural-transmission
> theory, frequency-dependent and prestige-biased transmission, and
> population-dynamic models of religious conversion and belief loss.
>
> The central distinction is between exposure, adoption, retention, and
> institutional reproduction. Existing Cartalith settlement, road, river,
> maritime, trade and migration networks provide the connectivity graph
> through which religious exposure propagates. Religious and cultural
> characteristics then determine how exposure translates into conversion.
>
> The model deliberately avoids assigning religions a single universal
> "attractiveness" score. Instead, religions are represented through
> quantitative traits whose effects depend on the receiving population and
> its environment.
>
> ### 1. Research Question
>
> The primary question is: *"How can the spread, adoption, persistence and
> decline of religions be represented mathematically as a function of human
> connectivity, cultural transmission, religious characteristics, social
> structure and political conditions?"*
>
> A secondary question is: *"Which characteristics of a religion can
> reasonably be represented as quantitative parameters without reducing
> religion to an arbitrary 'appeal' statistic?"*
>
> The literature suggests that religious systems possess characteristics
> affecting their ability to gain and retain adherents, while transmission is
> also strongly influenced by social context and cultural transmission
> biases. Watts describes religious systems as evolutionary systems whose
> traditions are inherited, change over time, and differ in their ability to
> gain and retain members.
>
> ### 2. Fundamental Model Distinction
>
> The system should distinguish four processes: **EXPOSURE → ADOPTION /
> CONVERSION → RETENTION → REPRODUCTION / TRANSMISSION**. These processes are
> not interchangeable.
>
> A religion can have: high exposure but low conversion; low exposure but
> high conversion; high conversion but poor retention; high retention but
> weak external transmission; strong institutional reproduction; strong
> political support but weak popular adherence.
>
> Therefore: Exposure ≠ Conversion ≠ Retention. This distinction is central
> to the model.
>
> ### 3. Religious Diffusion as a Network Process
>
> Cartalith already possesses the fundamental structure required for the
> geographical component: settlements connected through roads, rivers,
> maritime routes, trade and migration.
>
> Represent settlements as nodes G = (V, E) where V = settlements/population
> centres, E = connections between settlements. Each edge has a weight
> G_ij = f(d_ij, r_ij, t_ij, m_ij, p_ij, q_ij) where d = geographic distance,
> r = road connectivity, t = travel accessibility, m = maritime/river
> connectivity, p = political accessibility, q = commercial connectivity. The
> resulting value represents effective human connectivity, not simply
> geographic proximity — a distant major port can have substantially more
> contact with another settlement than a nearby isolated village.
>
> Mathematical cultural-transmission research supports representing
> populations as interconnected nodes with weighted transmission
> relationships. Network structure and centrality can influence the rate at
> which cultural variants spread.
>
> ### 4. Religious Exposure
>
> Let R_{i,r} represent the number of adherents of religion r in settlement
> i. Religious exposure arriving at settlement i can initially be represented
> as E_{i,r} = Σ_j G_ij R_{j,r}. Additional direct transmission mechanisms
> can be added: E_{i,r} = Σ_j G_ij R_{j,r} + M_{i,r} + I_{i,r} + X_{i,r},
> where M = missionary exposure, I = institutional exposure, X = other
> direct communication/travel exposure. This is preferable to creating a
> religious "radius."
>
> ### 5. Six Principal Diffusion Mechanisms
>
> **5.1 Human Movement** — migration, resettlement, diaspora formation,
> pilgrimage, population displacement. The religion moves because its
> adherents move.
>
> **5.2 Trade and Exchange** — merchants, trade routes, caravan networks,
> ports, commercial colonies. Trade creates repeated contact between
> populations.
>
> **5.3 Social and Kinship Transmission** — family, marriage, friendship,
> neighbourhood networks, household transmission. Primarily a local cultural
> mechanism.
>
> **5.4 Missionary and Institutional Transmission** — missionaries,
> priests/clergy, temples, churches, monasteries, schools, religious
> specialists. Intentional and organized transmission.
>
> **5.5 Political Transmission** — ruler adoption, elite conversion,
> patronage, state religion, legal privilege, discrimination, coercion,
> conquest. Political power changes the incentives and constraints
> surrounding religious behaviour. Conquest should not itself be treated as
> equivalent to conversion — it changes the political environment through
> which subsequent conversion mechanisms operate.
>
> **5.6 Information and Cultural Transmission** — scripture, oral tradition,
> preaching, translation, literacy, religious art, symbolic communication.
> Permits religious ideas to propagate without requiring permanent migration
> of religious specialists.
>
> ### 6. Religion as a Quantitative Trait Vector
>
> There is no established universal scalar describing the "attractiveness"
> of a religion. Instead, represent a religion as a vector of characteristics
> **R** = (C_comp, C_ritual, C_commit, C_inst, C_coh, C_pros, C_trans,
> C_mem) where C_comp = comprehensibility, C_ritual = ritual intensity,
> C_commit = commitment cost, C_inst = institutional capacity, C_coh =
> cohesion potential, C_pros = proselytization tendency, C_trans =
> transmission fidelity, C_mem = memorability. These are model parameters,
> not claims that these quantities have universal empirical values.
>
> ### 7. Comprehensibility
>
> A religion with highly complex doctrine may require greater exposure or
> institutional support before it can be reproduced accurately. Represent
> C_comp ∈ [0,1] where larger values represent greater ease of
> comprehension. Potential determinants: number of core propositions,
> doctrinal complexity, narrative structure, required specialist knowledge,
> linguistic complexity. Primarily affects transmission fidelity, not
> conversion directly.
>
> ### 8. Memorability
>
> A religious system may contain memorable narratives, distinctive symbols,
> repeated phrases, ritual structures, easily recognized practices. Represent
> C_mem ∈ [0,1]. A simplified transmission model: T_f = C_mem · C_comp,
> where T_f is transmission fidelity. Not an established empirical universal
> law — a modelling abstraction based on cultural-transmission theory.
>
> ### 9. Cultural Compatibility
>
> Compatibility should not be stored purely as a property of the religion —
> it is a relationship Compat(R, C) where R = religion, C = receiving
> culture. A religion could have Compat(R, C1) = 0.85 and Compat(R, C2) =
> 0.30. Potential components: Compat = w1·L + w2·K + w3·M + w4·S + w5·Q where
> L = linguistic compatibility, K = cosmological compatibility, M =
> moral/social compatibility, S = ritual compatibility, Q = compatibility
> with existing social institutions. One of the most important interaction
> terms in the system.
>
> ### 10. Ritual Intensity and Commitment Cost
>
> Religious practices impose different levels of participation: fasting,
> dietary restrictions, daily prayer, weekly worship, pilgrimage, initiation,
> distinctive clothing, sacrifice, communal labour. Represent C_ritual ∈
> [0,1] and C_join ∈ [0,1] (cost of adopting the religion) — these should
> not be treated as the same variable. A religion can have High JoinCost +
> High Retention, because demanding rituals may simultaneously discourage
> marginal converts while strengthening group cohesion. The mathematical
> model should therefore treat conversion cost and retention benefit
> separately.
>
> ### 11. Institutional Capacity
>
> Potential components: clergy, temples/churches, monasteries, schools,
> scriptures, administrative hierarchy, financial resources, missionary
> organizations. Represent intrinsic capacity as InstCap_R ∈ [0,1] but local
> institutional presence separately as Inst_{i,R}. Thus InstCap_R ≠
> Inst_{i,R} — a highly organized religion with no institution in a
> settlement still has low local institutional influence.
>
> ### 12. Social Cohesion
>
> Religious participation can create shared identity, communal ritual,
> mutual aid, collective obligations, marriage networks, charity, common
> festivals. Represent Coh_R ∈ [0,1]. Should primarily affect retention and
> secondary transmission, not direct conversion attractiveness.
>
> ### 13. Proselytization Capacity
>
> Some religions actively seek converts; others are primarily transmitted
> through birth, kinship or cultural inheritance. Represent Pros_R ∈ [0,1].
> This modifies missionary activity: M_{i,R} = Pros_R · Clergy_{i,R} ·
> Resources_{i,R} · Access_i. Proselytization becomes a capacity rather than
> a guaranteed conversion rate.
>
> ### 14. Frequency-Dependent Transmission
>
> Let p_R = N_R / N be the proportion of the population adhering to religion
> R. Under positive frequency-dependent transmission: T_R ∝ p_R^k where k >
> 1 represents conformity. This produces a feedback effect: more adherents →
> more social exposure → more conformity → more adoption → more adherents.
> Cultural-evolution research identifies conformity as a major transmission
> bias and shows that frequency-dependent transmission can generate rapid
> changes in cultural prevalence and S-shaped diffusion curves.
>
> ### 15. Prestige Bias
>
> People may preferentially adopt cultural traits associated with
> prestigious individuals — a separate mechanism from conformity. Conformity:
> P(copy R) ∝ Frequency(R). Prestige: P(copy R) ∝ Prestige(R). Recent
> mathematical work explicitly models the interaction between success bias
> and prestige bias and shows prestige can accelerate cultural dynamics
> through a rich-get-richer mechanism. Henrich & Gil-White's theoretical work
> provides the foundational distinction between prestige and dominance:
> prestige is voluntarily conferred deference, rather than coercive
> authority. For Cartalith: Prestige_{i,R} = f(RulerReligion, EliteAdherents,
> MerchantStatus, MilitarySuccess, ScholarStatus), calculated from the
> actual society.
>
> ### 16. Success Bias
>
> A population may copy cultural practices associated with people perceived
> as successful — differs from prestige: SuccessBias_R ≠ PrestigeBias_R. For
> example, a foreign population may adopt the religion of a prosperous
> merchant community because that community is perceived as economically
> successful. A mathematical model can represent role-model influence as
> G_ij ∝ β(A_ij) / Σ_k β(A_ik), where β represents the bias assigned to the
> perceived success of model j. Mathematical cultural-evolution work has
> explicitly developed stochastic models combining success and prestige
> biases.
>
> ### 17. Social Network Reinforcement
>
> Religious conversion should not be treated as simple one-contact contagion.
> Let E_{i,R} represent accumulated exposure. Conversion probability can be
> represented by a nonlinear function P_conv = 1 − e^(−kE), where k controls
> responsiveness to exposure — repeated exposure produces diminishing returns
> rather than unlimited linear growth. A threshold model can also be used:
> P_conv = σ(k(E − θ)) where θ = exposure threshold, k = steepness, σ =
> logistic function. Useful for religions requiring substantial social
> reinforcement before adoption.
>
> ### 18. Religious Competition
>
> Multiple religions can coexist within a settlement. Let R_1, R_2, …, R_n
> represent competing religious populations. The adoption probability of
> religion R should depend partly on the presence of competing systems, e.g.
> Competition_R = Σ_{q≠R} w_Rq · p_q, where w_Rq represents how strongly
> religion q competes with religion R. Allows mutually compatible religions,
> rival religions, syncretic systems, exclusive religions, persecuted
> religions to behave differently.
>
> ### 19. A General Conversion Function
>
> P_conv(R, i) = σ(β0 + βE·E + βC·Compat + βU·U + βS·Social + βP·P + βI·Inst
> + βF·Freq − βK·Cost − βX·Competition), where E = religious exposure,
> Compat = religion–culture compatibility, U = perceived utility, Social =
> social reinforcement, P = prestige/success influence, Inst = local
> institutional presence, Freq = frequency/conformity effect, Cost =
> conversion cost, Competition = strength of competing religions, σ =
> logistic function σ(x) = 1/(1+e^−x). This guarantees 0 < P_conv < 1.
>
> ### 20. Population Dynamics
>
> Let N_i be the population of settlement i, R_{i,r} the population
> following religion r. Then ΔR_{i,r} = Conversions − Defections +
> BirthTransmission + MigrationIn − MigrationOut. A simplified continuous
> representation: dR_{i,r}/dt = C_{i,r} − D_{i,r} + B_{i,r} + M^in_{i,r} −
> M^out_{i,r}. This allows demographic processes to remain separate from
> conversion.
>
> ### 21. Religious Loss and Deconversion
>
> The 2010 mathematical model of religious diversification explicitly models
> transmission and loss of religious ideas, including frequency-dependent
> loss of belief. A simple loss model: D_{i,r} = γ_r · R_{i,r}, where γ_r is
> the baseline loss rate. A richer model: D_{i,r} = R_{i,r} · (γ0 + γC(1 −
> Coh) + γI(1 − Inst) + γP·Pressure + γX·Competition). Allows religious
> communities to decline when institutions disappear, social cohesion
> weakens, political conditions become hostile, or competing religions
> become stronger.
>
> ### 22. Retention
>
> Conversion and retention should be modelled separately. Define P_retain(R,
> i) as the probability an adherent remains within religion R. A possible
> model: P_retain = σ(α0 + αC·Coh + αI·Inst + αF·Freq + αK·Kin − αP·Pressure
> − αX·Competition). This produces a crucial distinction: Easy to convert ≠
> Easy to retain, and Difficult to convert ≠ Weak religion. A religion may
> have high entry barriers but extremely strong retention.
>
> ### 23. Vertical, Horizontal and Oblique Transmission
>
> Cultural evolutionary theory distinguishes multiple transmission pathways:
> vertical (parent → child, T_V), horizontal (peer → peer, T_H), oblique
> (adult/leader/institution → unrelated individual, T_O). Total transmission
> rate: T_R = w_V·T_V + w_H·T_H + w_O·T_O. The relative weights vary by
> society — isolated agrarian society: w_V > w_H > w_O; cosmopolitan port:
> w_H, w_O > w_V; missionary state: w_O ≫ w_H. A particularly useful
> abstraction for Cartalith.
>
> ### 24. Religion–Culture Interaction Matrix
>
> Rather than assigning a religion a universal compatibility value,
> construct a matrix C_{R,C} where rows represent religions and columns
> represent cultures (each cell a compatibility value). Provides a
> quantitative mechanism for syncretism, resistance, rapid adoption,
> cultural friction, partial adoption.
>
> ### 25. Political Environment
>
> Political conditions should modify, rather than replace, cultural
> transmission. Define PoliticalSupport_{i,R} ∈ [−1, 1] where −1 = severe
> persecution, 0 = neutral, +1 = strong state support. Political effects can
> modify P_conv and P_retain but should not directly set religious
> population. This prevents "king converts → everyone converts immediately"
> and instead produces "king converts → political environment changes →
> institutions/incentives/penalties change → exposure and conversion
> probabilities change → population gradually changes."
>
> ### 26. Network Centrality
>
> Settlement centrality should influence religious diffusion: degree
> centrality, betweenness centrality, closeness centrality, weighted trade
> centrality, maritime centrality. For settlement i, Centrality_i can modify
> exposure: E_{i,R} = Centrality_i · Σ_j G_ij R_{j,R}. A major port,
> crossroads, pilgrimage centre or imperial capital becomes a natural
> religious diffusion hub — preferable to imposing arbitrary map-based
> religious spread radii.
>
> ### 27. Emergent Religious Geography
>
> The model should not directly draw religious boundaries. Instead: World
> Geography → Settlement Network → Human Movement → Religious Exposure →
> Cultural Interaction → Conversion → Demographic Change. Religious
> geography becomes an emergent property of the simulation — one of the
> principal advantages of the network approach.
>
> ### 28–29. A Complete Model and Proposed Data Model
>
> The complete system: World → Settlement Network → {Roads, Trade,
> Migration} → Exposure → Cultural Context (Compatibility, Language,
> Kinship, Social structure) → Conversion Model (Conformity, Prestige,
> Utility) → Adherents → {Family, Institutions, Mission} → Retention →
> Secondary Diffusion.
>
> Proposed data model — a **Religion** carries identity; doctrine
> (complexity, memorability, exclusivity); ritual (intensity, participation
> cost); social (cohesion, kinship emphasis); institution (organizational
> capacity, clergy requirement); transmission (proselytization, missionary
> capacity, transmission fidelity); behavioural (conformity sensitivity,
> prestige sensitivity, retention). A **Culture** carries language,
> cosmology, social structure, kinship structure, ritual system, authority
> structure, economic structure, openness to external culture. A
> **ReligionCultureRelation** carries linguistic/cosmological/ritual/social
> compatibility, conversion cost, syncretism potential, resistance. A
> **SettlementReligionState** carries adherent population, institutional
> presence, clergy population, missionary presence, prestige, political
> support, exposure, conversion rate, retention rate, competition. This
> prevents intrinsic religious properties from being confused with local
> conditions.
>
> ### 30–31. Calibration and Sensitivity Analysis
>
> The model should not initially assign real-world numerical values to
> abstract properties. Use dimensionless normalized parameters 0 ≤ x ≤ 1,
> calibrated through historical case studies, empirical demographic data
> where available, sensitivity analysis, comparative simulations, known
> historical outcomes. For each parameter x, measure S_x = ∂Outcome/∂x (or a
> finite-difference approximation) to determine which parameters actually
> matter — preferable to assigning arbitrary importance weights permanently.
>
> ### 32. Important Modelling Principle
>
> The model should never assume ReligionQuality → Conversion. Instead:
> ReligionTraits × Culture × Network × SocialStructure × PoliticalEnvironment
> → Conversion. The key conceptual safeguard against creating an artificial
> "best religion."
>
> ### 33. Academic Foundations
>
> Cultural evolutionary theory (conformity, prestige, success bias,
> frequency-dependent and vertical/horizontal/oblique transmission);
> mathematical prestige/success-bias models (fixation probability and
> fixation time); mathematical models of religion (transmission and loss of
> belief, frequency-dependent effects); religious conformity and conversion
> (overlapping-generations models, religious↔secular transitions); cultural
> macroevolution of religion (religious systems as evolutionary systems).
>
> ### 34–35. Recommended Architecture and Final Model
>
> Hybrid architecture: Network Diffusion + Cultural Transmission +
> Population Dynamics + Political Modifiers, i.e. P_conversion =
> f(NetworkExposure, CulturalCompatibility, SocialReinforcement,
> TransmissionBias, InstitutionalPresence, PoliticalEnvironment,
> ConversionCost, Competition), followed by Population_{t+1} = Population_t +
> Births + Conversions + MigrationIn − Deaths − Deconversions −
> MigrationOut, with Religious Institutions = f(AdherentPopulation,
> Resources, PoliticalSupport, OrganizationalCapacity) feeding back into
> exposure and retention.
>
> The complete causal loop: Connectivity → Exposure → Conversion →
> Population → Institutions → Transmission → Retention → Further Diffusion,
> modified by Culture + Social Structure + Religion Traits + Prestige +
> Conformity + Utility + Political Environment. Religious geography is not
> painted onto the world — it emerges from it.
>
> ### References
>
> Bisin, Topa & Verdier (2007), *Dynamic Models of Religious Conformity and
> Conversion*, European Economic Review 51(5). Baldini et al. (2010), *A
> Model for the Evolutionary Diversification of Religions*, Journal of
> Theoretical Biology 267(4). Henrich & Gil-White (2001), *The Evolution of
> Prestige*, Evolution and Human Behavior 22(3). Mesoudi (2017), *Cultural
> Evolution: A Review of Some Recent Findings*. Tehrani (ed., 2023), *The
> Oxford Handbook of Cultural Evolution*, ch. 45 (Watts, *The Cultural
> Macroevolution of Religion*). *Prestige Bias in Cultural Evolutionary
> Dynamics* (2024), Royal Society Open Science. *Biases in Cultural
> Transmission Shape the Turnover of Popular Traits* (2014), Evolution and
> Human Behavior 35(3). *A Two-Level Mutation-Selection Model of Cultural
> Evolution and Diversity* (2010), Journal of Theoretical Biology 267(2).

---

## 2 · What the paper's abstractions map onto, on inspection

Read against this port's actual data model rather than assumed compatible.

**§3's connectivity graph already exists, three times over — reuse it,
don't rebuild it.** `WayRouter` already holds a per-source Dijkstra with a
`prev_way` cache over the generated road/sea-lane network (cited unwired at
`RD-02` in `PARITY_AUDIT.md` §20 — this would finally give it a second
consumer); `TradeFlow.from`/`.to` already carries commercial connectivity;
`civ_hierarchical_network_topology` already carries the road hierarchy §3
wants as `r_ij`. Building a second graph would duplicate exactly the
structure `PARITY_AUDIT.md` has repeatedly flagged as "already built, just
unwired" for other subsystems. The paper's own §26 point — centrality makes
ports and crossroads natural hubs "without imposing arbitrary map-based
spread radii" — is already true of this network for the same reason it is
true of trade.

**§20's population dynamics has a real tick to run on.** Cartalith is not a
continuously-simulated world — `dR/dt` has no clock to integrate against by
default. But `TIMELINE_SCOPE.md`'s subsystem is exactly that clock:
`TimelineSnapshot { year, territory, settlements, ways }`, stable `tid`s
across regeneration, and a year cursor the Story-planning layer already
reads and writes (`STORY_PLANNING_SCOPE.md`). A diffusion step belongs here
— one recompute per simulated year, driven the same way `recompute_
civilisation` already is (on-demand, not a background loop), not a new
timing mechanism.

**§9's `Compat(R, C)` is not new — a version of it is already ported and
golden-tested**, one layer up. `relations.rs`' `culture_term` and
`religion_term` are already exactly "does this categorical trait match
across a pair," just applied to two factions instead of a religion and a
culture. The paper's compatibility matrix (§24) generalises this from a
symmetric same/different bonus to an asymmetric, weighted, five-component
score — real new work, but grounded in a pattern this port already has
tests for, not built from nothing.

**§6's trait vector and §29's `Religion`/`Culture`/`ReligionCultureRelation`
structs are the one place the paper needs data the reference has never had
an opinion about at all.** `CIV_RELIGIONS` is eight *names*, nothing more —
no comprehensibility, ritual intensity, institutional capacity or any other
trait exists for any of them today. Every number in §6-§18 is new-authored
content, not extracted from anywhere, and per §30 that is by design: the
paper itself says these start as calibrated dimensionless parameters, not
derived facts. This is the part that most needs an owner pass before it is
built, because whoever picks the Sun Cult's `C_ritual` is making a creative
decision about the setting, not a technical one.

**§4/§20's `R_{i,r}` — a religious population *per settlement* — is the
central gap.** Nothing in this port or the reference tracks population by
religion at any granularity finer than "this faction's one state religion."
Settlements have `pop` (total) and nothing that subdivides it. This is not
a missing accessor the way `RD-01`/`RD-02` are; it is a genuinely new field
that has to be initialized, persisted (a new save-format document slot, per
`SAVEFILE_COMPAT.md` §6.5's "documents an implementation does not model"
pattern — or, if it should be engine-owned and golden-tested like the rest
of `cartalith-civ`, a new tree-format array), and reconciled with the
existing hand-set `civFactionReligion` flag every faction already has. §5
below is where that reconciliation gets decided rather than assumed.

**§11's institutional capacity has a real anchor already**: settlement
`kind` already includes `monastery` (reference: `p.kind==='monastery'`,
counted into "Religious sites" on the Faction overview today). A settlement-
level `Inst_{i,R}` accumulator is a natural extension of a POI kind that
already exists and is already religion-coded, not an invention.

**§25's political modifiers have real machinery to attach to.** Faction
territory control, `civ_faction_aggregates`' power axes (including the
existing `religious` axis), and the relations model are all live. "Ruler
converts → political environment changes → conversion probabilities
change" is expressible as a `PoliticalSupport_{i,R}` term derived from
whether `i`'s controlling faction's `civFactionReligion` matches `R` — which
also means this term is the first place the new per-settlement model and
the old per-faction flag talk to each other.

**What has no anchor at all**: prestige (§15) needs a notion of "elite
adherents" and "scholar status" this port doesn't compute; missionary
agents (§5.4) as *mobile* entities are a bigger step than a static
`Pros_R × Clergy_{i,R}` term — the paper's own §13 keeps it a capacity, not
an agent, which is the right scope for a first pass. Both are named in the
milestone list below rather than designed here.

**Performance has a known shape and a known fix.** `jp_compute`'s wildlife
modifier (`PARITY_AUDIT.md` §23 F12) needed a content-fingerprint cache
before it could be wired at all — a naive per-keystroke recompute measured
~9× slower. A per-settlement, per-religion exposure sum over the road/trade
graph is the same shape of problem and should expect the same treatment
before any UI calls it on a hot path.

---

## 3 · Milestones

Ordered so each one is independently useful and independently golden-
testable, and so the paper's own central distinction — exposure ≠
conversion ≠ retention (§2 of the paper) — is not collapsed away for
convenience in the very first milestone that claims to implement it.

1. **MVP: network exposure and conversion, read-only.** New
   `SettlementReligionState` (adherent population per religion, one
   settlement) seeded at settlement creation from its founding faction's
   `civFactionReligion` — the only sourceless-population question the MVP
   has to answer, and the least invasive answer, since it makes the new
   layer agree with the old flag on day one instead of contradicting it.
   Exposure via §4's first term only (`Σ_j G_ij R_j`) over the existing
   route/trade graph (§2 above) — defer missionary/institutional/direct
   terms. Conversion via §19's logistic form collapsed to three terms:
   exposure, the existing `culture_term`-shaped compatibility, and §14's
   frequency/conformity feedback. One diffusion step per Timeline year.
   Output is additive and read-only: a new settlement-inspector panel
   showing the religious breakdown; `civFactionReligion` and
   `relations.rs`' `religion_term` are untouched. Retention is *not*
   separated from conversion yet — a single net rate is applied, an
   explicit, disclosed simplification of the paper's own §2 distinction,
   acceptable only because this milestone exists to prove the network-
   diffusion mechanic before spending complexity budget on the split.
2. **Institutional presence and retention split (§11, §22).** `Inst_{i,R}`
   from monastery-kind POIs and a real clergy count; `P_retain` as its own
   function so a religion can be hard to convert to and easy to keep, or
   the reverse, per the paper's own point in §22.
3. **Religion trait vectors, authored (§6-§13).** The one milestone that is
   primarily content, not code — an owner-facing pass to give the existing
   8 religions real `C_comp`/`C_ritual`/`C_inst`/etc. values, plus the
   `Compat(R,C)` matrix (§9, §24) against the existing culture list.
4. **Prestige and success bias (§15-16).** Needs `EliteAdherents`/
   `RulerReligion`/`MerchantStatus` terms this port doesn't compute yet —
   scoped separately because each is its own small investigation.
5. **Political modifiers (§25).** `PoliticalSupport_{i,R}` from territory
   control vs. `civFactionReligion` — also the milestone that decides
   whether a state religion becomes a *derived* plurality over its
   settlements' populations rather than a hand-set flag (§5 below), since
   that is the natural point where the two models would otherwise
   contradict each other.
6. **Competition (§18) and vertical/horizontal/oblique weighting (§23).**
7. **Sensitivity-analysis tooling (§31).** Dev-facing, not player-facing —
   a way to answer "does this parameter matter" the way this port already
   answers it for pass-relief and wildlife-forage (measured, not reasoned).

## 4 · The one fork this scope document does not resolve

Once §1's per-settlement population model exists, is a faction's "state
religion" still the player's hand-set `civFactionReligion` flag (cosmetic,
authoritative for `relations.rs`), or does it become the *derived*
plurality religion among the faction's settlements (simulated,
authoritative once the simulation exists)? The MVP in milestone 1 ships
with the flag unchanged specifically so this fork does not block a first
working slice — but it is a real design decision, not a technical one, and
milestone 5 is where it has to be made rather than assumed.
