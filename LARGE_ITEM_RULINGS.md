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

## Owner rulings — 2026-09-05, the five held groups from the menu design-conformance audit

The audit (four Fable 5.1 auditors + an adversarial cross-check, 283 items
enumerated from code) produced 99 deviations. Six were fixed under "fix what
needs no input"; five groups were held because each needed a decision. All five
are now ruled, and **four of the five go toward the drawings**, which is worth
recording: the earlier `Data ▸ Conversion` precedent made the canvas the stale
party, and these do not follow that pattern.

**1. Phone MORE — build the bespoke screens per `06-phone.md` §6.6.** The shell's
re-presentation of the desktop popups is superseded. Five purpose-built screens:
Project, Civilization, Data, Simulation, Preferences. This is a phone-navigation
rewrite, not a conformance fix, and should run as its own multi-batch arc.
**Unresolved and stated rather than guessed:** `phone_menu.gd` names
`docs/ANDROID_UI_SPEC.md` as its authority and that file is not in this
repository — it lives in the owner's design project. The build proceeds from
`06-phone.md`; if the missing spec later contradicts it, `06-phone.md` is what
was ruled on here.

**2. Left dock — restructure to the 12 mode-gated blocks** of
`04-left-dock.md` §3. One body per rail node, gated by mode. **This deliberately
hides controls that are reachable today**, which is the opposite instinct to the
tablet ruling ("keep everything, reflow only") — the two surfaces are being
decided differently on purpose. Architectural; touches all three workspaces.

**3. Δ vertical — keep it live in 2D, and re-present it horizontally.** The
Measurement Toolbar canvas's "vertical tools appear only in 3D relief" is
recorded as superseded: the heightmap carries the data and this port has no 3D
viewport, so gating it there would make the mode unreachable. **Beyond the
options offered, the owner asked for a presentation change**: rotate it so the
readout is horizontal, with **X = distance and Y = height**. That is a
cross-section profile chart, not the current numeric list — scope it as new
drawing work over the same measurements, not as a relayout.

**4. All three structural moves — go to where the newest design puts them.**
Journey planner becomes a CIVIL rail node (v3/RP) rather than a Data menu row;
Atlas "Refine detail for the current view" moves to the WORLD rail beside Bake &
finalize; the Asset-pack submenu flattens from four bands to the newest 9-row
expansion. **A consequence was raised before the ruling and the ruling stands:**
the flat shape has nowhere for `Clear library… destructive`, which the bands
carry. **Do not silently drop it** — find it a home and say where, because losing
a destructive action to a layout change is a capability loss wearing conformance
clothes.

**5. All 37 no-design surfaces — derive from the DCC vocabulary and build them.**
The 2026-08-25 ruling ("where none exists, derive from the DCC canvases' own
vocabulary") is extended to the whole set. Eight of them are not really
undesigned — the DCC Shell canvas draws a bespoke breadcrumb browser whose own
comment says it "replaces the stock OS tree picker", and `browse_dialog.gd`
already exists — so those eight stock `FileDialog`s are derivable work. **The
other 29 have no drawing at all**, so the owner will first see them running
rather than drawn; each should carry, in its own source, which canvas vocabulary
it was derived from.

---

## 2026-09-05 (evening) — the round-2 design canvas

Put to the owner as thirteen artboards at
<https://claude.ai/code/artifact/782c0fd3-b6ce-4320-a491-a8b48d63e9cd>, page 1
carrying the two questions only the owner could settle.

**6. The WORLD left dock — Option B. The A/B mode switch comes back.**
Answered `Option b` against two artboards drawing the same content twice. This
settles **Owner question 3**, which had been open since before 2026-09-05 and was
filed in §3.3 as *doubly* blocked — an owner call **and** captions that no longer
exist in any readable file.

Three consequences travel with it, and they are the cost of the option chosen:

- ~~**The two captions are DERIVED, not read.**~~ **WITHDRAWN 2026-09-06 — this
  consequence was false when written, and following it would have made the tree
  lie.** `ldSwA:'GENERATION PIPELINE',ldSwB:'SCULPT',` is at
  `design/dcc-environment-2026-08-31/Cartalith DCC Environment.dc.html:1940`,
  `ldCollapsedLabel`'s word for mode a at `:1937`, and `04-left-dock.md:138`
  binds segment B's text to `{{ ldSwB }}`. **The captions are QUOTED.** That file
  is whole — 239 712 bytes, 1 994 lines, ending `</script></body></html>` — since
  commit `660cbef` ("Design answered: the files are whole"); `04-left-dock.md`
  §0's truncation note and its §9.1 "Lost to truncation" table were never
  updated. The derivation reasoning is kept in `world_workspace.gd` as
  **corroboration, not provenance**.
- ~~**The pill costs 34 px**, which pushes World data below the fold.~~
  **BOTH HALVES MEASURED WRONG, 2026-09-06.** Read off the drawn nodes at
  1920×1080: the band is **40.0 px**, confirmed by hiding it and watching every
  category move up by exactly 40.0. And it does **not** cause the fold crossing —
  the scroll fold is `y=1054`, the World data header is at `y=1656` with the pill
  and `y=1616` without: a 40 px shift onto a header already ~562 px below the
  fold. **8 of 9 categories are below it either way; only `Generate` clears it,
  and the cause is Generate's expanded body.** The artboard's B1 caption carries
  the same two errors. Related: the shipped comment's reason for the short form —
  that `GENERATION PIPELINE` would not fit — is false too; it measures **185 px
  in a 372 px dock, 187 px spare**. `PIPELINE` stays on the newest-canvas rule,
  not on a width.
- **`WorldDockB.dc.html` must be fixed before it is built.** Its lit half is
  drawn as a filled amber slab (`--acc` / `--accInk`). `dcc_widgets.gd:1194`
  reserves that treatment for the tool bar's three SCULPT / PAINT / MEASURE
  segments *"and nothing else"*, citing `GUI_GAP_REGISTER` §48 (DS-02) — the pass
  that **removed every filled amber slab in the shell**. The lane flagged the
  pill's *radius* as its open question and missed the *fill*, on the single
  element the owner was being asked to approve.

**7. The timeline's pill on-state matches the WORLD dock's.** Owner: *"For the
timeline b as well"* — resolving, in one decision, the split the design critic
asked to be resolved once across **Timeline**, **WorldDockB** and **BiomePaint**.

**Resolved to the WASHED treatment** — `accent_wash_2` fill, `accent` ink,
border — which is what `DccWidgets.segment()` → `set_segment_on()` already
builds and what the approved `Religion.dc.html` uses. This is not a style
preference: the filled alternative is the one DS-02 deleted shell-wide, so
building it would reintroduce something a previous pass deliberately removed.
~~**Recorded as an interpretation** … if the filled look was meant specifically,
this reverses.~~ **HEDGE RETIRED 2026-09-06 — the canvas settles it.**
`ldSwABg` is `var(--wash2)` and `ldSwACol` is `var(--acc)` at
`Cartalith DCC Environment.dc.html:1941-42`, and `tlSpeeds` carries its own
`bg:'var(--wash2)'` for §7. Owner ruling, shipped code and prototype all agree;
`ldSwitch:s.domain==='WORLD'` also settles which domains show the pill.
**One real open question remains, and it is a token question rather than a
correction:** the prototype's `--wash2` is alpha **0.16**, while
`DccWidgets.set_segment_on()` paints `accent_wash` at **0.09**. The operative
instruction — go through `set_segment_on()` — is unchanged; moving the shell to
0.16 would re-base 141+ call sites and is filed in `OUTSTANDING_WORK.md`.

---

## 2026-09-06 — all nineteen open decisions answered

`OUTSTANDING_WORK.md` §4 goes to **zero**. Put to the owner in five rounds, each
option grounded in what the scope documents say and in how comparable tools
(Azgaar's FMG, grand-strategy titles, DCC apps) solve the same problem.

**Five went against the recommendation offered. Each is recorded with its cost;
none of them is agreement.** Q2 was amended rather than chosen.

**8. What is a conflict attached to? → GEOMETRY WITH OPTIONAL REFERENCES.**
Settles what `STORY_PLANNING_SCOPE.md` §6 called the highest-leverage unanswered
question. Unblocks **SP-4** and through it **landmark M9**. Two obligations that
are not optional: a deleted referenced entity leaves the geometry and dashes the
reference through the existing absent-vs-set discipline (omit the key, `has()`,
state the reason) — **never a sentinel id**; and the reverse query *"which
conflicts touch this settlement?"* must be answerable, or the references are
decoration.

**9. Journey route on regenerate? → INVALIDATE.** *(Against the recommendation,
which was to keep it with a staleness mark.)* The cost stands and is recorded: a
regenerate is cheap and frequent here, so this repeatedly discards authored work.
Build the mitigations rather than re-litigating — invalidation must be **loud**,
**re-plan from the same endpoints** must be one click (party, season, carriage
and stages are authored inputs and survive), and the Journey entity itself is not
deleted; only its polyline is invalid.

**10. Landmark persistence? → PERSIST** in `entities/landmarks.json`. Consistent
with the recorded finding that research §25's state transitions cannot be
recomputed. Needs a `SAVEFILE_COMPAT.md` entry and a format-version note.

**11. Settlement density? → RAISE THE `ecological_factor` CEILING.** It saturates
at 2.0 on 5 of 6 real factions, so the ceiling and not the ecology is deciding.
**This ruling IS the golden re-baseline authorisation** — record it as such, since
a lane will otherwise correctly refuse. Re-measure afterwards: if it still pins on
most factions the ceiling was not the binding constraint and the row re-opens.

**12. Parity contract for landmarks? → EXEMPT.** `FUNCTION_INDEX.md` returns
nothing for "landmark"; there is no reference to diff against. **Write it into
`DECISIONS.md` as a §7-series note**, not only here — §7a/§7d is where a lane
looks. The project's standing bar (property tests, mutation-tested constants,
probes on drawn output) applies anyway; it was not separately ruled.

**13. Landmark as a vault `EntityKind`? → YES**, finish the wiring. The template
exists and `template.rs:155` recognises it; `links.rs:81-84` does not resolve it.
The half-wired state was the worst of the three.

**14. Generated landmarks vs the manual icon tool? → ONE LAYER, TWO ORIGINS.**
One collection with an `origin` field. The renderer draws one layer; **M6 spacing
sees everything**, so generation cannot place a landmark on top of a hand-placed
icon; a regenerate replaces only the generated ones. Costs a migration of
`annotations/icons.json`.

**15. 16K/32K export? → UN-SHELVE.** *(Against the recommendation.)* Back on the
critical path with `EXPORT_SCOPE.md` §5's costs unchanged: the **render-once
decision must be reversed** (establish what depends on it first); four measured
gaps close; **the codec question is now live and may need its own decision** —
WebP dies at 16 383 px, JPEG XL dies on its AGPL encoder, and neither survivor is
obvious; and the banded renderer that was prototyped, measured byte-identical and
reverted should be recovered from history rather than rewritten. Note also that
the reference's own bake draws **terrain and nothing else**.

**16. Viewshed budget? → CHEAP AND COARSE, PLUS A MANUAL "recompute and refine".**
*(Amended by the owner; the refine action is theirs.)* Coarse keeps a regenerate
interactive; an explicit action upgrades it. **The shell already has this
vocabulary** — `Refine detail for the current view` on the WORLD tool-options
bar. Follow it rather than inventing a second. Open while building: view-scoped or
whole-world, and whether a refined result persists (interacts with ruling 10).

**17. Sculpt stamp on sea-level move? → RE-READ LIVE**, matching the reference.
*(Against the recommendation.)* This port's snapshot at `sculpt.rs:1076` becomes
the divergence to remove. **The cost stands: one global slider retroactively
changes every committed stamp.** Together with ruling 9 this is one coherent
position — *the world is live and authored artefacts follow it* — and both need
the same thing built: **the change is visible when it happens.**

**18. Shrink `STATUS.md`? → NO.** The tax is the accepted price of one place
holding state. Closes the §4 row **and** the §3.1 blocker pointing at it. Not to
be re-opened as an efficiency idea.

**19. Diagnostics window? → NO WINDOW.** The spec draws exactly two (§8 Asset
library, §9 Data manager) and §2.6's Window menu names those two; §2.5 gives
`Working set` as a read row and `GPU acceleration` as a toggle plus readout.
**`performance_window.gd` folds away** — and its DCC restyle hours earlier is
superseded, not wasted, since the *rows* move. **Inventory what it shows and give
each item a menu-row home or drop it deliberately**, or this quietly loses
capability.

**20. Phone app bar `☰` / `▤`? → STALE.** Adopt `[world pill] · ⌕ · ⋮` per the
2026-08-31 Android canvas; newer-canvas-wins applies. Scopes stage 3 of the shell
rebuild. **Measure that everything the two glyphs reach is still reachable
before deleting either** — the phone MORE rewrite passed exactly that test.

**21. `statusMid`'s `repaint NN ms`? → `_refresh_map()` WALL TIME.** The number a
user can act on and the one measurable honestly without a rendering-server hook.
Quote it as a median with min..max, harness run alone.

**22. Declared-but-unused residue? → DELETE `init_gpu_f64`, KEEP `--good` /
`--accH`.** They look alike and are not: the function is pilot residue with no
caller, the two tokens are declared-and-never-used **in the prototype too**, so
declaring them is fidelity. **Record that reason beside them** or the next
dead-code sweep removes them and is right to by its own rule.

**23. Store distribution and signing? → NOT YET, a deliberate non-goal.** Stay on
debug signing; this repo continues to hold no secret. `--export-release` failing
at signing is **expected and correct** — the unsigned APK it leaves is the good
one. Keep that attached to the APK recipe.

**24. Landmarks in the crate graph? → CONSOLIDATE INTO `cartalith-civ`.**
*(Against the recommendation, which was to ratify the existing split.)* A real
refactor, scheduled as its own row: the terrain-derived half moves in, the
dependency direction wants watching, and **golden tests must not move** — a crate
move that changes a value is a re-baseline and this ruling does not grant one.

**25. WASM target? → NOT YET, a deliberate non-goal.** Recorded as decided rather
than unexamined, because it is not cheap: every crate on the shared path would
have to stay wasm-compatible, forbidding threads, filesystem and some
dependencies in exactly the crates doing the heavy work. Zero `wasm32` hits in
any `Cargo.toml` today, which is correct.

**26. The 16K/32K export codec? → PNG.** Owner, 2026-09-06: *"For the codec in
export use png, even if size balloons. We should just inform the user of the
expected file size."* And, clarifying the deliverable the same day: *"this would
be an user generated monolithic image of the map. No layers, no extensive
information. Just to be used outside of Cartalith in an image viewer."*

The survey had already eliminated the alternatives on their own terms — **WebP
dies at 16 383 px**, below the smaller of the two target sizes, and **JPEG XL
dies on its AGPL encoder**. Size was the only argument left against PNG and the
owner spent it deliberately. The clarification removes the rest of the question:
one flat raster, no sidecar metadata, no layer preservation, no tiling.

**Three consequences follow, and they are build items rather than open
questions.**

**Export RGB, not RGBA — ALREADY TRUE, nothing to change.** Measured
2026-09-06: `render::bake_rect` fills `vec![0u8; w*h*3]`, `encode_png_rgb8` takes
exactly that, and `write_tiles` cuts 3-byte runs. **There is no alpha on this
path to drop.** This paragraph first claimed dropping it "cuts the pre-encode
allocation by a quarter" and quoted a saving of 2.75 → 2.06 GB at 32K; that
saving was never available, because the buffer has always been three-channel.
The instruction stands as a constraint on anything added later, not as work.

**And the memory figures it quoted were the RAW BUFFER, not the peak** — the
error that mattered. `w*h*3` is 2.06 GB at 32 768 × 20 976 and 515 MB at
16 384 × 10 488, but the export also allocates the local-contrast pass's luma
and its blur buffers. **The file's own budget said 15 B/px and was itself too
low** — `blur_once` allocates two buffers, not one, and both live until it
returns. Corrected bound **23 B/px**, against a **measured 21.7 B/px** from
host-polled peak resident memory. **So 32K really costs about 15.8 GB and 16K
about 3.95 GB** — 7.7× what this ruling first said. Anything reasoning about
whether a device can hold an export must use the peak, not the buffer. Establish that the
renderer's output really is opaque everywhere before dropping the channel; if
any pass writes alpha, composite onto the theme's ground rather than keep it.

**Report memory as well as file size, before the run.** The instruction is to
tell the user the expected size, which is only useful *ahead* of a long export —
the Data-manager route pane's ESTIMATE block is the existing shape for that.
Derive bytes-per-pixel from real exports at smaller sizes rather than a generic
PNG rule of thumb: map imagery compresses unusually well, so a textbook figure
will be wrong in the user's favour and still wrong. **Memory is the harder
constraint** — and by more than this ruling first allowed. Against a real peak
of ~15.8 GB at 32K and ~3.95 GB at 16K, a phone that already peaks near 878 MB
on a 2048×1311 world cannot hold **either** new size. **The export refuses
rather than dying mid-run** (built 2026-09-06): a failed `Vec` inside a
GDExtension aborts the process and takes the editor and any unsaved world, so
`refuse_unaffordable` runs before the allocation. **Note it gates every width,
the three that shipped before this ruling included** — a deliberate behaviour
change, since an 8K export on a device with 2 GB free aborted today.

**The FILE is far smaller than §6 guessed, and in the wrong direction.**
Measured across three worlds: **213.9 MB at 32K** (208.3 .. 229.8) and 80.4 MB
at 16K — a 9.6× compression ratio, not the 2-4× §6.3 assumed, so its
"500 MB - 1 GB" is 2.3-4.7× too high. This ruling warned a textbook figure would
be "wrong in the user's favour"; it was wrong **against** the user instead. The
estimate now ships a model fitted to five real exports.

**PNG is scanline-ordered, and that is still the good news — but the prototype
cannot be recovered, and this ruling was wrong to say it could.** It said
*"recover that prototype from history rather than rewriting it"*. **It is not in
history:** the banded renderer was reverted *before* its pass committed.
`git log --all --diff-filter=A -- '*export_bands*'` returns nothing, and
`git log --all -S ExportBandPlan` returns only two commits whose diffs are prose.
**`EXPORT_SCOPE.md` §4.1-§4.3 is the entire surviving artefact**, and any banded
implementation is a rewrite against that description. The design property holds
— a band of rows is what a PNG encoder consumes next, where a tiled format would
not be — so the approach is right and only the "recover it" instruction was
false. **Note also that the existing `tiled` flag is NOT this**: `write_tiles`
slices tiles out of an already-whole raster, so it costs the same peak and buys
nothing at 32K. Check Godot's own
`Image`/`save_png` dimension limits early: if it cannot write a 32K PNG in one
call, the banded path stops being an optimisation and becomes the only route.

Unchanged by this ruling: **the render-once decision still has to be reversed**
(ruling 15's first cost), and what depends on it must be established first.

**27. The timeline stores mutations, not snapshots.** Owner, 2026-09-06: *"for
the timeline we basically only have to track mutations per year. If a position
doesn't change for 50 years that's 50 datapoints we do not need."*

**The current shape is worse than the archive-size discussion suggested, and the
memory cost is the headline.** `TimelineSnapshot` is `{ year, territory:
Vec<i32>, settlements: Vec<NamedSettlement>, ways: Vec<Way> }` — every recorded
year keeps a **full territory raster and a complete copy of every settlement and
way**. `CivData.timeline` is a live `Vec<TimelineSnapshot>`, so this is resident
memory first and archive size second: one snapshot's territory at 2 048 × 1 311
is **10.74 MB in RAM**, and the phone already peaks near 878 MB with no timeline
at all.

**On disk the redundancy is real but milder, and an earlier framing of mine was
wrong.** Deflate compresses each year's entry **independently**, so temporal
redundancy is entirely unexploited — 100 unchanged years cost 100 × ~40 KB, not
~40 KB. The 294× figure measured for a territory raster is *spatial* coherence
within one year and says nothing about the year-over-year case this ruling is
about.

Build notes, not decisions:

- **The unit is a mutation** — a cell whose owner changed, a settlement whose
  fields changed, a way added or removed. Everything else is inherited.
- **Reconstruction must be exact, and that is the property to test:** replay to
  year N and compare against a full snapshot taken the old way. A delta chain
  that drifts is worse than the redundancy it replaces.
- **Keyframes are probably still wanted**, because the timeline has a year cursor
  the user drags and pure deltas make a late year replay from the beginning.
  State the interval and why.
- **This ends `SAVEFILE_COMPAT.md` §10.2's `history/territory/<year>.i32` as the
  on-disk shape**, so it needs a `format_version` bump and §18.4's **fail-loud**
  discipline: a reader that ignores the marker must not read plausible-looking
  noise.
- **Measure resident memory and archive size before and after** rather than
  asserting the win.

**28. LOD tiles are stored in the save, optionally.** Owner, same message: *"the
LOD tiles should be stored in the save, or at least optional to include."*

**The archive can already carry them, which makes this smaller than it looks.**
`cartography/tiles/**` round-trips today as **foreign entries**, with a test
asserting a tile survives open-and-resave byte-identically. This is promotion to
a first-class optional slot, not new plumbing.

Two things it does need:

1. **A producer.** Tiles are synthesised on demand (`lod_bridge::
   synthesize_tile_rgba`) and the atlas cache is explicitly deferred at M3, so
   nothing currently holds a tile set to write.
2. **Invalidation.** Tiles are *derived*, so a stored set is a cache that can go
   stale against a regenerated or sculpted heightmap. `cartalith-spatial`'s
   `StageGraph`/`Staleness` is the existing mechanism. **Silently drawing a stale
   tile over a re-sculpted world is the failure to design against — prefer
   dropping them to drawing them.**

**Optional means the default must be chosen.** Recommended **off, with the size
shown at save time**: a pyramid is ~4/3 of its base level, which makes this the
one slot capable of outgrowing the three float grids that dominate the archive
today. **Measure a real pyramid before writing a default into the UI** — the
previous ruling in this file shipped four unmeasured figures.

**29. Tiled output lives in the save path only, never in Data ▸ Export.** Owner,
2026-09-06: *"the tiled output should only live in the save menu. It has no merit
in the export menu."*

**This resolves PR-10's open question by rejecting its premise rather than
answering it.** The round-3 canvas took Gaea's lesson — that what you build and
how it is written are different objects — and proposed a **Build dialog plus a
persistent Build Manager under `Data ▸ Export`**, with build type, tile grid and
inter-tile blending as a named, saved, reusable definition. The owner's answer is
that there is no such object here: a tile pyramid is **not an export product**,
it is part of what a project stores.

**It agrees exactly with ruling 28, which was decided independently a few hours
earlier**, and the two together now say one thing rather than two: tiles are an
**optional slot inside the project archive** (`cartography/tiles/**`, already
round-tripping as a foreign entry), written when the user saves, defaulting off,
with the size shown at save time. **Ruling 28's "size shown at save time" now has
a home** — the save affordance, not a preferences pane and not an export dialog.

**What this deletes from the design, and it is most of it:**

- no build dialog, no build manager, no saved build definitions;
- **no build type, no tile grid, no blending percentage** — those describe a file
  layout the user chooses, and there is no such choice when the destination is
  the archive's own slot;
- `Preferences ▸ Tiles & LOD` keeps only what changes how the app **draws right
  now**: the atlas-cache size cap (a `_todo` row today), tile size (256/512/1024
  as shipped, not the canvas's four), the memory readouts, and clear/rebuild.

**Do not conflate this with ruling 26.** The 16K/32K PNG is a *monolithic single
image for an external viewer* — one flat raster, deliberately not tiled, and the
owner scoped it that way on the same day. Tiled output going to the save path
takes nothing away from it: they are different artefacts with different
destinations, and the export menu keeps the one it already has.
