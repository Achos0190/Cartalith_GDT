# Roadmap

A sketch, deliberately not over-planned. Each phase gets its own scope document —
as `MVP_SCOPE.md` serves Phase 1 — written when it starts, informed by what the
previous phase actually taught.

**This file defines the phases. It does not track them.** Read it for what a
phase *is*, where its boundary sits, and which decisions shaped it. For where
any of it stands today — done, partial, blocked, declined — read
**`cartalith-native/docs/STATUS.md`**, which is the only place progress is
recorded. Where a phase description below was written before the work started
and the work then contradicted it, the correction is recorded in the phase's own
section rather than by quietly rewriting the original text.

## Phase 0 — Walking skeleton

No engine logic. Prove `gdext` + Godot + Rust builds and runs on all three targets
with placeholder content: a triangle, a button, a printed line. Steps in
`TOOLCHAIN.md`.

"All three targets" means the three `DECISIONS.md` §2 names: Windows, Android,
and a WASM build. Only the first two have ever been committed to — WASM is
listed under *Options kept open* below and has no export preset.

**The Android export is the risk.** gdext's own docs call Android support
experimental (`REFERENCES.md`). Surface trouble here immediately rather than
discovering it mid-Phase 1.

## Phase 1 — Terrain MVP

Scope in `MVP_SCOPE.md`, which carries the seven success criteria. The phase is
the full pipeline — tectonics, height, climate, erosion, hydrology —
parity-verified (`PARITY_TESTING.md`), rendered in 2D, reading HTML saves
(`SAVEFILE_COMPAT.md`), and packaged as an `.exe` and an `.apk` confirmed on the
owner's hardware.

Two things belong in the definition of done and are easy to forget: the **credits
screen** and a **licence check** over the crates pulled in (`PROVENANCE.md`).

## Phase 2 — Civilisation layer

Block 2: factions, settlements, territory, roads, provinces, economy. A new
`cartalith-civ` crate depending on `cartalith-engine`'s terrain without modifying
it. `PHASE2_SCOPE.md` carries the milestone definitions.

It will need its own golden data. Settlement suitability is exactly the kind of
subtly-tuned scoring — soil × rainfall optimum, flood penalty, coastal preference
— that a rewrite gets plausibly wrong; read the v1.30 "one function" CHANGELOG
entry before starting.

The Journey Planner is large and largely self-contained. Consider it a sub-phase
rather than bundling it. **That advice was taken**: it became its own scope
document, `JOURNEY_PLANNER_SCOPE.md`, with six engine milestones and five
integration steps. Economy aggregation was likewise split out into
`ECONOMY_SCOPE.md`.

## Phase 3 — Rendering and 3D

Two halves, scoped separately.

**2D fidelity beyond MVP's "correct and plain"** — multi-octave grain, hillshade
quality, NPR styles — is `TERRAIN_APPEARANCE_SCOPE.md`, six milestones plus a
follow-up.

**The 3D drape** deferred in `DECISIONS.md` §4, and with it the point to evaluate
`Terrain3D` and `godot_heightmap_plugin` (`REFERENCES.md`) — as a dependency or as
reference for a Godot-idiomatic clipmap renderer. **The owner parked 3D on
2026-08-31**, the same day the research it commissioned landed:
`cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md` explores the options,
makes a recommendation, and parks itself with three questions unanswered.
`DECISIONS.md` §4 continues to stand.

**The UI/UX half went further than this entry anticipated.** The entry named it
as "the moment to install a UI/UX skill (`SKILLS.md`), once the interface
outgrows four controls". What it became instead is a full DCC-style shell with
its own scope document (`DCC_SHELL_SCOPE.md`): a hold was called on the GUI on
2026-08-18 and **lifted later the same day** by the owner — *"replace the current
GUI and replace it in full … including all it's wiring and functionality"* — and
that replacement has been its own line of work since.

## Phase 4 — Asset Library

Block 3, the sprite and texture pack system. `ASSET_LIBRARY_SCOPE.md` carries the
milestone definitions.

The paragraph this entry originally carried said the phase was "lower priority
unless custom art becomes a near-term goal — and probably better after Phase 3
establishes what art plugs into. Confirm before starting." **The owner's explicit
direction started it**, which satisfied the confirmation; it did not wait for
Phase 3, and the dependency this entry assumed does not exist.

**Two corrections to that paragraph, from reading the real code:** an asset is
not arbitrary art but one PNG bound to a slot in a **frozen, ordered
vocabulary**, and an asset pack is a real shipping serialization format
(PKZIP + `pack.json`/`pack.csv`), not a proposal. The renderer genuinely
draws pack sprites; the vector glyphs are the *fallback*. Also confirmed:
Phase 5's urban morphology does **not** consume asset packs, so the two are
independent.

## Phase 5 — Urban morphology

Block 4, procedural city layouts. Already a self-contained DOM-free engine in the
JS codebase, which suggests it ports cleanly into `cartalith-urban`, depending on
`cartalith-civ` for settlement context. `URBAN_MORPHOLOGY_SCOPE.md` carries the
milestone definitions.

**Started 2026-08-18. Two corrections to the paragraph above, from actually
reading the code — see `URBAN_MORPHOLOGY_SCOPE.md`:**

- "DOM-free" is right (zero hits for any browser API in block 4's whole range),
  but **"ports cleanly" is true of the boundary and false of the effort**: 92
  engine functions / 2,937 lines plus a 28-function / 925-line civ adapter —
  ~3,860 lines, **the largest single unported subsystem** this port found,
  bigger than the Journey Planner and the Asset Library. ~17 milestones, not one
  phase-sized push.
- **"depending on `cartalith-civ`" is wrong for the engine.** `generate(seed,
  opts)` takes only scalars and two plain rasters; no civ types anywhere. The
  civ coupling lives one layer up in block 2's `_um*` adapter. `cartalith-urban`
  depends on `cartalith-rng` alone.

## Not a phase: LOD and large worlds

The tiled-LOD deep-zoom system matters as worlds grow. Godot's terrain plugins may
cover the 3D case; the 2D case may need its own answer if 20,000 km worlds — the
ones v2.05–v2.09 were fixing — turn out to matter natively too.

This section originally ended "revisit when a concrete need appears rather than
building it speculatively." **The owner triggered that revisit on 2026-08-17**,
choosing to lay the foundation without waiting for the need — data structures
only, touching nothing in the live pipeline. It therefore has scope documents of
its own and is no longer an unscheduled option:
`LOD_TILING_BASE_SCOPE.md` for the standalone base, and
`LOD_TILING_INTEGRATION_SCOPE.md` for threading it through the pipeline.

## Options kept open, not scheduled

~~Save-file **writing** (`SAVEFILE_COMPAT.md`)~~ — **no longer open.**
Authorised by the owner 2026-08-23 after five register rows (FI-01, DM-04,
JP-06, JP-08, MEA-07) had queued up behind it. `SAVEFILE_COMPAT.md`'s own
"Writing a save" section carries the format decisions and the one disclosed
limitation (`state.erosion`).

Store distribution (`DECISIONS.md` §6) and a WASM target sharing
`cartalith-engine` (`DECISIONS.md` §2) are things the architecture permits
and nobody has committed to. Raise them rather than assuming they are queued.

~~**Markdown Vault integration**~~ — **no longer unscheduled. Scheduled
2026-08-24 on the owner's own instruction**, naming three entity kinds:
continents, provinces and settlements (explicitly *not* POIs, which stay an
unported concept). `MARKDOWN_VAULT_SCOPE.md` is the scope document this
section demanded, and carries the milestone definitions.

The verification this section asked for was done first and changed the plan:
**continents did not exist**. `generate_continentality_field` is a per-cell
scalar, not an entity — but `build_landmass_quality`'s golden-verified
8-neighbour flood fill has always labelled land components and always threw
the labelling away, so `cartalith_civ::civ_continents` keeps it. Settlements
(`tid`) and provinces (`id`) were real as expected.

POIs and "regions" as §35 criteria 6-7 mean them are **unsatisfiable in this
port** and are recorded as such rather than faked.

**Priority and framing, owner 2026-08-18**: *"Its not a critical part."* It
does not compete with engine or parity work; the owner's 2026-08-24 go-ahead
scheduled it, it did not promote it. The target is a **generic Markdown
vault** — Obsidian is one compatible vault and the owner's own, but nothing
may require it and no Obsidian-specific behaviour belongs in the core. **An
Obsidian plugin is a wish, deferred outright.** Nothing shipped writes an
`obsidian://` link, a wikilink or a block reference.

`DCC_SHELL_SPEC.md` §9's vault block in the Data manager assumes more than
the V1 design allows (`obsidian://` links, note links in exported GeoJSON,
two-way sync — the last an explicit V1 non-goal) and **was deliberately not
touched by this work**; `DCC_CONTROL_INDEX.md` records the conflict and
`MARKDOWN_VAULT_INTEGRATION.md`'s header resolves it.

**Landmark generation** — owner-supplied research imported 2026-08-30
(`LANDMARK_GENERATION_RESEARCH.md`) and cataloged the same day
(`LANDMARK_GENERATION_SCOPE.md`). Placed here rather than as a numbered phase
because it is not one subsystem's work: it composes terrain, hydrology,
civilisation, urban and vault data that already spans Phases 1, 2 and 5, the
same cross-cutting shape the Markdown Vault entry above has — not a new
pipeline stage with its own start/end.

The 2026-08-30 investigation found this closer to buildable than the research's
own "Cartalith already possesses or intends to possess" hedge suggested: a
golden-verified mountain-pass corridor detector (`DECISIONS.md` §7i) and a
population-weighted cost-distance influence field (`DECISIONS.md` §7b)
already existed and mapped closely onto two of the research's own suitability
terms; a TPI-equivalent computation already existed, buried inside the 2D
renderer's ambient-occlusion pass (`cartalith-godot/src/render.rs`) rather
than exposed as reusable data; and 15 mineral resources, soils and lithology
were all real. What that pass found **completely absent** was
viewshed/visibility and any general-purpose Poisson-disc sampler — the two the
research leans on hardest for landmark *significance* rather than mere
placement. `LANDMARK_GENERATION_SCOPE.md` §1 is the full inventory, §3 lays
out nine dependency-ordered milestones, and §4 poses six questions —
persistence, Markdown Vault entity status, whether the golden-parity contract
even applies here, the crate boundary, the viewshed cost budget, and the
manual-icon relationship — raised for an owner ruling.
