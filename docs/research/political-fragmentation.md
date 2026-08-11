# Political fragmentation on a single landmass

Grounding note for v1.58. Owner report: auto-populate assigns one faction per connected landmass
(`contFaction`, keyed on landmass component id), so a world with few large landmasses reads as
"settlements cluster onto one faction" — a faction with no landmass of its own gets zero
settlements regardless of how many factions exist. Owner's own framing of the fix: "if there is
only 1 continent it should lead to a division of the continent, based on geography and industrial
prowess... I think those are easy denominators to use."

## Why a single landmass plausibly holds more than one polity

Political unification of an entire landmass under one state is the historical exception, not the
rule. Most of history's large landmasses hosted several simultaneous, often rival, polities:

- **Warring States China** (5th–3rd c. BCE) — seven major states on one continuous landmass, each
  centred on its own river basin / agricultural core, separated by mountain ranges and the
  logistics of the day.
- **The Indian subcontinent** — essentially never unified for long outside empire-building peaks;
  the Deccan Plateau, the Gangetic plain, and the coastal strips each repeatedly hosted independent
  kingdoms, split by the Vindhya and Western/Eastern Ghats ranges.
- **Classical Greece** — dozens of independent city-states on one peninsula, fragmented as much by
  mountainous terrain (poor overland connectivity) as by choice.
- **The Holy Roman Empire / early modern Italy** — a patchwork of hundreds of small states on
  contiguous land, where terrain, river lines, and the uneven distribution of trade wealth (not
  political theory) did most of the work of drawing borders that persisted for centuries.

Two factors recur across all of these as the practical determinants of *where* new independent
centres of power emerge and *where* the border between them settles:

1. **Geography** — mountain ranges, major rivers, and sheer distance raise the cost of projecting
   power, so a state's effective reach is bounded well short of "the whole landmass." Borders
   overwhelmingly settle along the *expensive* terrain between two cores (a watershed ridge, a
   river line), not through the middle of good farmland.
2. **Economic base** — an independent polity needs a resource base large enough to support a
   court, an army, and trade — a fertile river valley, a mining district, a natural harbour. Poor,
   sparse land rarely sustains an independent state; it gets absorbed by whichever neighbour is
   strongest, which is exactly the "marginal land doesn't get its own polity" pattern.

## What the engine already has, reused rather than re-derived

- **Suitability already is "geography + economic base" in one number.** `buildSettlementSuitability`
  (v1.30, "one function") sums carrying capacity, freshwater, slope, elevation band, soil×rain,
  buildability (core terms) plus coast/river/lake/mineral opportunity terms — i.e. it is already the
  file's own unified answer to "how good is this land, including its resources." Reusing it as the
  capacity signal here is the same "don't build a second signal that answers the same question"
  discipline v1.30/v1.33/v1.35/v1.37 all independently arrived at — a fresh "industrial prowess"
  field would just be suitability with extra steps and a second place to drift out of sync.
- **The border-drawing is already geography-aware and already exists.** `_civAutoPolity`
  ("Recalculate Territories") floods outward from every settlement's own faction through
  `buildTravelCost` — a slope-squared cost field, so steep terrain (mountain ranges) is expensive to
  cross and cheap valleys are not. Two rival capitals seeded on opposite sides of a range will,
  under this existing flood-fill, naturally meet and stop somewhere near the ridge — the real-world
  pattern above — with **zero changes needed** to that function. The only gap was ever upstream: no
  settlement on a single landmass could get any faction but the landmass's one assigned id, so there
  was never more than one flood source to begin with.
- **Farthest-point/blue-noise spacing for placing several "seeds" that shouldn't crowd each other is
  not a new technique in this file** — v1.26's asset-scatter engine already does exactly this
  (`spacing` rejection over a shared bucket grid) for placing multiple instances of a thing without
  clustering. Reused here for capital placement: a rival capital needs to be both economically
  strong (suitability-ranked) *and* far enough from an existing capital to be a genuinely separate
  centre, which is a direct restatement of "geography and industrial prowess" as the two seeding
  criteria.

## The one genuinely new piece: how many polities does a landmass earn

Given `factionCount` factions total and `L` landmasses with candidate settlements, today's code
gives each landmass exactly one faction id, cycling if `factionCount <= L` and simply wasting the
remainder if `factionCount > L` (ids `L+1..factionCount` never appear anywhere — true whether there
is 1 landmass or several; a 1-continent world is just the extreme case, not a special one).

`factionCount - L` "spare" seats (only ever > 0 when there is real unused capacity) are apportioned
across landmasses by **highest averages** (Jefferson/Webster-family apportionment, the same family
of method used for real seat allocation, e.g. the US House of Representatives): repeatedly hand the
next seat to whichever landmass currently has the highest `capacity / (seats+1)`, where `capacity`
is the landmass's summed settlement suitability. This is a real, named, deterministic algorithm
rather than an invented threshold, and it has the property the owner asked for directly: a bigger
and/or richer landmass earns more seats, in proportion, and a small or poor one stays at its
guaranteed floor of one.

**Every existing landmass keeps its exact primary faction id** (same cycling order as the current
code) — only the ids that were never used before (`L+1..factionCount`) get handed out as extra
seats. So when `factionCount <= L` (today's common multi-continent case, which the owner explicitly
called "okay in the base"), the apportionment loop never fires and the output is byte-identical to
the current code — this is a strict generalisation, not a special-cased rewrite.
