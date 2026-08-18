# Markdown vault templates — owner-supplied reference

Owner-supplied 2026-08-18 alongside the clarification that the integration
targets a **generic Markdown vault**, not Obsidian
(`MARKDOWN_VAULT_INTEGRATION.md`'s header carries that decision). Extracted
verbatim; these are the owner's real authoring templates, not invented
examples.

```
Landmark template.md
Settlement Template.md
Region Template/
    Regional Overview.md
    Regional Culture.md
    Landmarks/
        Landmark template.md      (same file as the root one)
```

## What these tell us that the spec does not

Four structural facts, each with a consequence for whoever scopes this.

**1. A region is a folder, not a note.** `Region Template/` is a directory
containing `Regional Overview.md`, `Regional Culture.md`, and a `Landmarks/`
subfolder. `MARKDOWN_VAULT_INTEGRATION.md`'s `KnowledgeLink` identifies a
target by `vaultId` + `relativePath` — a *file* path. Linking a Cartalith
region to a vault region therefore means linking to a **directory with a
conventional layout**, or to one note within it, and the model as written
does not distinguish those. Resolve before implementing; do not assume.

**2. Two placeholder conventions coexist in the same corpus.**
`{{Landmark_Name}}` and `{{Region_Name}}` are Templater-style substitutions;
`[Name]`, `[If applicable]`, `[Optional]` are plain prose placeholders a human
overwrites. A template-filling implementation must not treat them alike, and
**neither is Cartalith's to fill** under §23's machine-owned-block rule —
Cartalith owns only its delimited block and everything outside it belongs to
the author.

**3. The entity set matches V1 scope exactly** — Settlement, Landmark
(the spec's "POI"), Region — plus Regional Culture as a companion note to a
region rather than a fourth entity. The naming differs: the spec says POI, the
templates say Landmark. Pick one vocabulary and record it.

**4. Landmark appears twice, byte-identical** — once at the vault root and
once inside `Region Template/Landmarks/`. So the same template is used both
free-standing and region-scoped, which means a landmark's *location in the
vault* carries meaning the note body does not. Any "which region does this
landmark belong to" inference must read the path, not the front matter — and
there is no front matter in any of these templates at all, which is worth
noting given §10 lists YAML front matter among the constructs to support.

## Field-by-field: what Cartalith could actually fill

This is the useful half for §18-19 (the Cartalith feedback block and the
exportable-field registry). Cartalith should offer **only** fields it derives
from real world state, and never touch the authored ones.

### Settlement Template — derivable today

| Template field | Real source | Status |
|---|---|---|
| Type (City-State / City / Town / Village / Waypoint) | `SettlementKind` | Real — but the vocabularies differ; this port has 5 tiers, the template lists 6 labels |
| Size / Population | `get_settlements()` `population` | Real |
| Location — region, landmarks, terrain | territory/province + biome raster + `explain_settlement` | Real |
| Economic Activity — key trades | `get_trade_balances()` (`civ_resource_trade_balance`) | Real, engine-side; exports/imports per settlement |
| Strategic Importance — trade routes, borders | roads (`get_roads`), sea routes, territory adjacency | Real |
| Infrastructure — water sources, roads | water access, road network | Real |
| Notable Features or Landmarks | resource potentials, volcanism, craters | Partial — Cartalith knows the terrain features, not their names |
| Current Status (Active / Abandoned / …) | — | **Authored.** No temporal model exists (`VISION.md`'s open question) |

### Settlement Template — authored, never Cartalith's

Former Names · Era of Establishment · Founding date/myth · Ruling authority ·
Governance · Historical Significance · Culture & Identity · Unique customs ·
Notable People · Known Issues / Threats.

Governance and culture are worth calling out: `ECONOMY_SCOPE.md` found the
reference's own Government/Religion/Ag-tech fields are **UI-only** with no
simulation behind them, and `PHASE2_SCOPE.md` found culture is naming-flavour
data only. So Cartalith has nothing true to say about them and must not
pretend otherwise.

### Regional Overview — derivable today

Biome · Terrain · Climate · Key Resources — all real (biome raster, height/
slope, temperature/rainfall, the 15 resource fields). Natural hazards is
partial: volcanism and flood risk are real, the rest is authored. History and
Political Entities are authored, except that faction/territory ownership is
real.

### Regional Culture — entirely authored

Language, beliefs, social norms, food, arts, festivals. Cartalith has
`civ_culture_terrain_fit` (a terrain-affinity score) and the naming syllable
tables — neither is a cultural description. **Nothing in this template should
be machine-filled.**

## The honest summary

Roughly a third of the Settlement template and half of Regional Overview map
onto real, already-computed state. Everything else is authorial. That ratio is
itself the argument for §23's delimited-block design: the machine block should
be small, clearly fenced, and regenerable, sitting inside a note that is
overwhelmingly the author's.
