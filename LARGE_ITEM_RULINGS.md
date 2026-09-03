# Owner rulings on the eighteen Large rows

> **Relationship to `cartalith-native/docs/STATUS.md`** (added 2026-08-31)
>
> **Nothing in this file is status, and it should never be read as any.** These
> are *decisions* — what the owner chose, what each choice commits to, and what
> cost was stated and accepted. A decision is true from the moment it is taken
> and does not go stale as code lands; that is exactly why it lives in its own
> document rather than in a ledger that gets rewritten.
>
> - **Is it built?** → `cartalith-native/docs/STATUS.md`, the single source of
>   truth for progress.
> - **Is it queued, and what blocks it?** → `OUTSTANDING_WORK.md`.
> - **Which control is it, and what is behind it today?** →
>   `UNWIRED_FUNCTIONS.md`, whose Large section this file rules on.
> - **Was it decided, and what did that commit to?** → here.
>
> Two rulings below carry obligations that outlive them and are easy to lose:
> **paint-brush falloff must be recorded in `DECISIONS.md` as a deliberate
> divergence from the reference when it lands**, and **colour management touches
> the one surface every golden-parity fixture pins**, so it ships behind a
> default that leaves sRGB byte-identical or it re-baselines deliberately and
> says so. Cite a ruling from here rather than re-asking the owner.

Taken 2026-08-31, by interrogation over `UNWIRED_FUNCTIONS.md`'s Large section
(sixteen rows with proposals, plus the two recorded without one because they
needed a decision rather than a design). Recorded here so the build can cite a
ruling instead of re-asking.

**These are decisions, not schedules.** Nothing below is in flight; the work in
flight on this date is the 59 trivial/small/medium rows.

---

## Build

| Row | Ruling | What that commits to |
|---|---|---|
| **CARTO ▸ Labels: the whole panel** | **All three steps, in order** | (1) a `label_class` field on `MapLabel`; (2) a generated labelling pass emitting per-class placements — this is what makes the drawn-count column real; (3) a per-class typography record carrying size/halo/tracking. Note `halo` and `tracking` do not exist anywhere in the engine today, so step 3 creates them. |
| **Label collision culling** | **Build with the labelling pass** | Measure-and-suppress rides in the same pass that places labels. Explicitly *not* a standalone job — culling a set nothing generates is half a feature. Unblocks icon placement rule 1. |
| **CARTO ▸ Icons: generated placement** | **Build, and add a sea-marks asset family** | A generated placement pass, *plus* a fourth family in `cartalith-assets` so `SEA MARKS` and the *snap sea marks to coast* rule become real. **This answers owner question 4**: the design's four placement families become literal rather than mapped onto the engine's three. Carries new slots, new glyphs and a coastline snap test. |
| **The river entity** | **One binding plus viewport hit-testing** | `get_rivers()` returning polylines with `id, name, length_km, source_elev, discharge, catchment_km2, tributaries, navigable`, and river hit-testing so the dock context becomes reachable. Closes the seven dashed fields, the three Actions and CARTO's rivers-as-ways prose together. |
| **Civilisation authoring operations** | **Re-entrant civ stages plus a civ parameter group** | Expose the civ pipeline's own stages as five re-entrant `#[func]`s over an *existing* world, plus a civ `PARAMS` group. Turns CIVIL from a generate-time output into an editable layer. The single largest CIVIL gap, taken whole rather than split. |
| **Settlement diagnostics overlay** | **Surface the data now** (ruling stands; its stated premise was wrong — see below) | ~~Add `cartalith-urban` as a dependency of `cartalith-godot`.~~ **No Cargo edit is needed and none should be made.** The option was put to the owner citing `UNWIRED_FUNCTIONS.md`'s line that *"the crate is not even a dependency of `cartalith-godot`"*. That sentence is a quotation of a pre-milestone-17a finding and is stale. Verified 2026-08-31: `cartalith-godot/src/urban_bridge.rs:44` already does `use cartalith_civ::urban_adapter::{…}` and calls `urban_adapter::settlement_layout` at `:238`; `cartalith-civ/Cargo.toml:22` carries `cartalith-urban`, and the comment there records the indirection as **deliberate layering**, kept so `cartalith-urban`'s only dependency stays `cartalith-rng`. A direct edge would buy nothing and break a defended decision. **The scoped `Cargo.toml` authorisation is therefore withdrawn as unnecessary — not exercised.** What the owner actually chose — surface urban data now rather than defer to `URBAN_MORPHOLOGY_SCOPE.md` — stands, and is reachable through `urban_adapter` today. The accepted risk is unchanged: milestones 8–16 are largely unbuilt, so every field without data must be dashed with its reason, never left blank. |
| **Landmark funnel** | **Both halves** | A crowding parameter on the placement pass *and* a rejected-candidate coordinate list plus a new overlay layer to draw it. `landmark_funnels()` returns eight scalars today and carries no coordinates, so the dict grows. |
| **Cut · Copy · Paste · Select all** | **Selection sets → clipboard → commands** | In that order. Step one — a selection *set* per entity kind, replacing the three unrelated single-`i64` selections — is independently valuable and pays for itself even if the clipboard never lands. |
| **`Units` (km / mi)** | **Build, and add nautical miles** | One formatter ahead of all five hard-coded call sites, plus the settings key, plus a third unit. Nautical earns its place because the app has sea routes and navigable rivers. Closes the written promise at `phone_menu.gd:84-85`. |
| **Rebindable keyboard shortcuts** | **Per-context table with conflict detection** | A binding table in `DccSettings`, applied over the menu accelerators at build time. Per-context, not flat — the same key means different things with a tool armed. |
| **Saved measurements + CSV** | **Fold into the caller-owned save slots** | A measurement store as a save slot, riding the `project_bridge.rs` read/write section being built on this date for the other four slots. Deliberately *not* a second persistence mechanism. |
| **Colour management** | **Build it** | A colour space on the render target, threaded through to the texture. Owner overrode the recommendation to defer. **The cost that was stated and accepted:** every golden-parity fixture is sRGB, so this touches the one surface the parity harnesses pin. Do it behind a default that leaves sRGB byte-identical, or re-baseline deliberately and say so. |
| **Paint brush falloff** | **Bind it — add a falloff term to `PaintStamp`** | The highest-severity row in the document, now ruled. **This is a deliberate divergence from the reference**, not a parity fix: `cartalith-spatial/src/paint.rs` quotes the reference verbatim — *"a hard disc… unlike `sculpt()`/`brushHeight` there's no soft falloff here"*. It must be recorded in `DECISIONS.md` as a divergence when it lands. Also resolves the duplicate: two `Hardness` copies are on screen at once today, and only one should survive. |
| **CPU worker threads** | **Build it — a configurable pool** | Call `ThreadPoolBuilder` at engine init with a count from settings; expose a `#[func]` to read and set it. Rationale accepted: on the 6T, saturating all eight cores stutters the UI thread and thermally throttles mid-generation. |
| **`Report an issue`** | **Replace with a local diagnostic dump** | Rename to a *save diagnostic report* action writing generation info, missing bindings, project format version, GPU state and the last error to a file the user attaches themselves. No endpoint required. Three of those five readouts are being built on this date as trivial rows. |

## Schedule separately

| Row | Ruling |
|---|---|
| **The manual-icon tool** | **Milestone E of `UNIFIED_TOOL_PLAN.md`**, as its own scoped pass. Arming, rendering and persistence are three distinct gaps; the persistence half depends on the caller-owned save slots being writable. |
| **`Region ▸ New world from selection`** | **A scoped parity pass.** The resample is built and tested; the orchestration — allocate, clear warp fields, invalidate caches, refresh climate, empty the civ layer — is new `WorldGen` state and must not be folded into GUI work. |

## Deferred, with research first

| Row | Ruling |
|---|---|
| **The 3D viewport** | **Deferred — but explore options first.** Owner, verbatim: *"For 3d I'd like to first explore options, so defer for now. If you can put Sonnet or another smal agent on it to do some research on how to render the terrain as detailed as you can. Since we're in a game engine anyway…"* A research pass was dispatched the same day; its output is `cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md`. `DECISIONS.md` §4 stands until that is read. The two menu rows and the phone 2D/3D FAB stay drawn and disclosed; the FAB's toast becomes honest when `Preferences ▸ Graphics ▸ relief exaggeration` lands as a Small row. |

**Parked later the same day.** Owner: *"On part of the 3D let's keep that for
later at this moment, it will be implemented later on."* The research document
stands complete at **1 530 lines** (counted 2026-09-01; this line read "1 486"
when written, and drifted inside a single day) with its recommendation made;
three commissioned
questions were never answered and are listed under its own *Status: parked*
heading, so resuming is cheap. `DECISIONS.md` §4 continues to stand. **No 3D
work of any kind is scheduled.**

### Two corrections to that research brief, same day

The first brief was written with mobile as a hard design constraint. The owner
reversed both halves of that framing:

1. **The Adreno 630 is a capability *gate*, not a *ceiling*.** Verbatim: *"It
   shouldn't just take the adreno 630 as limit, it might be that a base amount
   of processing power is needed and if a adreno 630 can't bring enough power 3D
   rendering should be disabled based on a machines power."* So the target
   render is designed first, the hardware it requires is established second, and
   insufficient hardware becomes a **runtime capability gate with named tiers** —
   not a reason to down-scope the approach. A gated-off 3D viewport must
   disclose *why*, per this repository's standing rule that a control with
   nothing behind it says so; a silently missing menu item is the wrong shape.
2. **Visibility-driven streaming is the central question, above the geometry
   survey.** Verbatim: *"the research should find viable methods of rendering
   large worlds without taking the whole whole world to be rendered at once.
   Other large mmo's seem to have methods of rendering o ly what's visible
   instead of the whole world. Research all the existing methods and find the
   best ones."* The brief now covers frustum and occlusion culling (including
   what Godot 4.7 ships natively), chunked streaming with hysteresis, the LOD
   seam problem, CDLOD/clipmaps/virtualized geometry, sparse virtual texturing,
   impostors, GPU-driven culling into indirect draws, floating-origin precision
   at world scale, and the documented streaming architectures of large-world
   MMOs and open-world titles — ranked for *this* application, where the source
   is a generated heightfield plus classification layers rather than authored
   art.

---

## Consequences for the open owner questions

- **Question 4** (*how do four icon placement families map onto three asset
  families?*) is **answered**: they do not map — a fourth family is created.
- **Question 5** (*paint falloff: bind it, or delete all three sliders?*) is
  **answered**: bind it, as a recorded divergence.
- **Question 7** (*are the four unwritten save slots deliberate or an
  oversight?*) is answered by implication — a fifth slot is now scheduled for
  saved measurements, so the list is a live contract rather than residue.
- Questions **1, 2, 3, 6, 8, 9, 10** remain open. Question 1 still blocks the
  right dock's `rdExtraMode()` Medium row, which is why that row was excluded
  from the build in flight.

---

# Owner rulings on the four GUI blockers — 2026-09-03

A second round, asked and answered on 2026-09-03 when the owner reprioritised
GUI work. Four rows sat in `OUTSTANDING_WORK.md` §3.1 *blocked on an owner
decision and nothing else*; three are now unblocked and one is reclassified.

The owner also set the GUI order these serve: **the §3.2 rows blocked on other
work first, then the unblocked rows, then the rows blocked on a design that does
not exist.**

| Item | Ruling | What it means for the build |
|---|---|---|
| **DS-03 — the tablet interior is not a scaled desktop** | **Keep everything; reflow only** | The tablet gets the **full desktop inventory**, in a denser or scrolling layout. No control is removed, so the ~30% of desktop content currently deleted comes back. This is a *content* answer and it retires the per-control question entirely: there is no "which controls leave" list to build, because none leave. The styling problem is not solved by it — `DccTheme.TABLET`'s key space is still exhausted, with one desktop integer mapping to two tablet figures in at least five verified places, and §57 refuted the obvious placement for a role-keyed resolver. **That architectural half remains, and is now the whole of DS-03.** |
| **The right dock does not follow the armed tool** (`rdExtraMode`, nine contexts) | **Selection wins; the tool appends a section** | The dock keeps showing the selected entity; an armed tool adds its own section *below* rather than replacing the view. This is the answer to **owner question 1**, which `LARGE_ITEM_RULINGS.md` above records as still blocking this row. The naive merge — flipping the dock away from a selected settlement the moment a tool arms — is explicitly rejected. Nothing is yanked away mid-edit, so no "is editing" signal is needed. |
| **CV-24 / ED-02** — the year scrubber as program scope; the undo-history panel | **Both wait for a design pass** | `TIMELINE_SCOPE.md` §4's standing instruction — design the panel first rather than guess its region — is upheld rather than overridden. **These move from §3.1 (blocked on an owner decision) to §3.3 (blocked on a design that does not exist).** They are not closed and not startable; the ruling is that guessing is worse than waiting. |
| Stop shipping the ~218 `_*_probe` / `_*_shot` scenes inside the APK | **Exclude them — a scoped authorisation to edit `export_presets.cfg`** | **This overrides a standing prohibition and the override is deliberately narrow.** Agents are otherwise forbidden to touch `export_presets.cfg`; this authorises adding the probe/shot patterns to `exclude_filter` and nothing else. It does not authorise any other change to that file, and it does not extend to `Cargo.toml`, `.gitignore` or `project.godot`, which remain off limits. Note the precedent worth honouring: the Settlement diagnostics row carried a scoped `Cargo.toml` authorisation that was later **withdrawn as unnecessary** once the code was read — so verify the exclusion is actually needed before exercising it. |

## Consequences for the open owner questions

- **Question 1** (*does `rdExtraMode()` replace the right dock's ten selection
  contexts, or sit beside them?*) is **answered**: it sits beside them. The
  selection keeps the dock; the tool appends.
- Questions **2, 3, 6, 8, 9, 10** remain open. Question 3 (the WORLD left-dock
  A/B switch) was **not** put to the owner in this round on purpose: it is
  doubly blocked, and the captions and gate live in the truncated tail of
  `02-rail-and-domains.md` §8, so there is no label to build the control with
  even once the call is made. Asking it would have produced an answer that
  still could not be executed.

## Owner ruling — 2026-09-03, the pack-import warning

**A golden re-baseline is authorised, and it is the first one this project has
taken.** Every agent brief in this effort carries the line *"a golden re-baseline
needs an owner ruling — you do not have one."* This is that ruling, and its scope
is deliberately narrow.

**Authorised:** edit the warning string in `cartalith-assets/src/manifest.rs`
(`"N pack section(s) not yet used by the live map (…)"`), and re-capture the three
fixtures that pin it — `golden_parity_pack_manifest.rs:131`, `:292`, and
`tests/fixtures/reference_pack_captured.json:320`.

**Not authorised by this ruling:** any other golden, any other function, or a
re-capture of `parsePackManifest`'s other outputs. One string, three fixtures.

**The cost, stated because it is permanent.** The port now diverges from
`Cartalith Gen1 v2.11.html`'s own `parsePackManifest` output on this string. Every
future parity comparison of that function carries the divergence, so it must be
disclosed there rather than rediscovered as a failure. `DECISIONS.md` §7a is the
protection this overrides; the owner overrode it knowingly, choosing one source of
truth over a port-side filter.

**Why the alternative was rejected:** annotating port-side would have kept parity
intact but left a known-false string in the tree, corrected only at the point of
display — two places to keep in step instead of one.

## Owner ruling — 2026-09-03, trait sprites

`trait` is the one clause of that warning that is **true**: `asset_bridge.rs`
round-trips `manifest.structures.traits` and `pack.rs` composites no trait sprite.
Scheduled as an ordinary backlog row rather than folded into the warning fix.

## Owner rulings — 2026-09-04

**1. The pack warning: re-derive the whole list before ruling again.** The
2026-09-03 trait ruling rested on the premise that `trait` was the warning's one
true clause. That premise did not survive being re-opened — `composite_map_icons`
(`pack.rs:470`) draws settlement and poi sprites too. Rather than widen the
re-baseline on a second premise that might also be wrong, the owner asked for the
measurement first: **for every section name the warning can emit, establish
whether `pack.rs` actually composites it, and report the true unused set.** No
further edit to the string until that lands. Audit-only; it closes nothing by
itself.

*Why this matters beyond the row:* two premises in a row failed on contact here.
The measurement is the cheap way to stop ruling on the third.

**2. "Selection wins, the tool appends" DOES extend to the Journey planner.**
`rdMode4()` (rule 8) is the last built context that replaces the selection; it
becomes an appended section like every other. **Carry the conversion hazard
across with it:** rule 1's conversion silently took Commit/Discard away from a
live uncommitted draft, because `_tool_section()` answers with one id whose
`match` reached the ordinary tools before the draft clause. Every transition
INTO the converted state has to be enumerated and proved, not just the disarm
path. Batch 22's Lane A was told rule 8 was out of scope (this ruling did not
exist when it was dispatched) — it is batch 23's.

**3. The flat export keeps `world_origin`.** It is this port's own
interoperability surface, not Gen1's, and a reader that does not know the key
ignores it. Provenance stays in both the project save and the flat export.
Closes the question Lane C raised rather than decided.
