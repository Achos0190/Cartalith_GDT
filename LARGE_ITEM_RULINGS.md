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

*Superseded 2026-09-23 by Ruling AO:* on regenerate a journey's route is
**re-snapped** onto the new terrain, not invalidated (`journey_progress::resnap_journey`;
AO's addendum gives the policy for a stop that no longer exists). AO re-ruled
this without naming ruling 9; this note is the link.

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
*Written 2026-09-24 as `DECISIONS.md` §7q.*

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

*Superseded 2026-09-23:* the owner ruled to **ratify the existing split** after
all (`OUTSTANDING_WORK.md`'s closed "Consolidate landmarks into `cartalith-civ`"
row) — landmark synthesis stays in `cartalith-civ` (`landmark.rs`), the
analytical field library stays in `cartalith_terrain::analysis`. Ruling AP
confirmed no crate split. The consolidation above was never carried out.

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

---

## 2026-09-07 — three rulings from the GUI parity work

All three were put to the owner with the measurement that raised them, and all
three were answered the same day. **Two are code; the third deliberately is
not.**

### Ruling A — Android file-picking goes through SAF

**The question.** Opening a project `.zip` on Android lands in the user’s real
Documents and lists **zero files** — a 3.5 MB `Werk.zip` sits there, invisible.
The two available mechanisms are exactly complementary, measured with **zero
permissions**: the in-shell `DirAccess` browser lists directories fine and
**cannot see a file another uid wrote**; SAF
(`DisplayServer.file_dialog_show`, `FEATURE_NATIVE_DIALOG_FILE = true`) browses
the whole device but its **directory** URI is unusable by `DirAccess` (err 31),
so it cannot back a storage root.

**Ruled: route file-picking to SAF.** `choose_file()` — Open project, import
asset pack, choose sprite sheet — goes through Android’s own picker.
**Folder-picking KEEPS the in-shell browser** it was just given, which is what
closed the owner’s original complaint. Two mechanisms, split by mode, each on
the side it actually works.

**What this obliges.** `OPEN_FILE` returns a `content://` document URI.
**`FileAccess` and `ZIPReader` both accept it — measured, err=0 and an exact
byte length** — but the CALLERS do not know that: `app.gd`'s
`_on_open_zip()`/`_on_import_pack()` and `asset_library_window.gd` treat the
return as a filesystem path. **Audit every caller before shipping it**, and
expect anything doing `get_base_dir()`, `path_join()` or a `dir_exists` check on
the result to need a branch.

**Explicitly NOT ruled: `MANAGE_EXTERNAL_STORAGE` is not being added.** It was
offered and declined. It would let one browser serve both modes, and it needs
`export_presets.cfg` (guarded), a manual "All files access" grant the user
currently **cannot** give because the toggle is greyed out, and a Play Store
justification. **Do not re-propose it as a shortcut.**

### Ruling B — the selected Preferences chip keeps Medium and gains the accent

**The question.** The owner asked for the selected option to be **bold**. What
shipped is IBM Plex Mono **Medium (500)** — measured at **+3.3% ink**, real and
repeatable and subtle. True Bold (700) means a fourth face.

**Ruled: keep Medium, and add the accent ink to the selected chip.**

**The reason is layout, and it is the whole point of the ruling.** Medium’s
advance is **identical to Regular at 93.00**, so nothing re-flows. A Bold face
has a wider advance and would re-flow **every chip row** — and chip rows on the
phone are exactly where this project’s clipping defects have been appearing
(`CREATE WORLD` at 7 of 46 dp painted, the SAVE foot’s primary button
off-screen). **Colour costs no layout at all**, so it buys legibility for free.

**This supersedes the literal reading of the owner’s own earlier wording**
(*"make the text of the selected option bold"*) — by the owner, with the
trade-off in front of them. **Record it at the call site** or a later
conformance pass will read the accent as drift and remove it.

### Ruling C — the invisible OFF switch track is a CANVAS defect, not a shell one

**The question.** On the light palette an OFF switch is a lone dark knob: its
track is `#f4f2ee` against a `#fbfaf7` ground, **1.03:1**, so the track is
invisible and the control does not read as a switch. **`ENV:1354` specifies
exactly this**, so changing the shell would be a deviation from the reference.

**Ruled: take it back to the designer. Fix the REFERENCE, not the shell.**

**So this is the one ruling that produces no code change, and that is
deliberate.** Parity with the canvases is the standing definition of done for
GUI work; patching the shell around a canvas defect would put the two out of
step and guarantee a later conformance pass reverts it. **Same principle that
settled the radius question:** one source of truth, and when it is wrong you
change it there.

**Until the canvas is corrected the shell keeps drawing `ENV:1354` as written**,
and `_cklight_probe` continues to pin it — so the current behaviour cannot drift
while the question is open. **Do not "fix" this in `dcc_widgets.gd`.**

---

## 2026-09-07 (late) — the two tablet exceptions, so "100%" has a meaning

**Context.** The owner checked the tablet and found it still shows the PC
layout. `TABLET_UI_SPEC.md` had parked adoption behind *"an owner decision that
has not been made"* — a decision that had in fact been given the same day. Of
the canvas’s items, §4.2 named exactly **two as not adoptable**, and those two
decide what "match 100%" can mean on tablet. Both are now ruled.

### Ruling D — `Run stage NN` is DRAWN, and runs the whole pipeline

**The question.** The canvas draws a run button per pipeline stage. **The
engine has no partial recompute** — one `generate()` resolves all ten stages
together, and they share state.

**Ruled: draw the buttons as the canvas draws them, wired to a FULL run**, with
that stage’s result brought into view. Visually 100%, no engine work.

**The obligation this creates, and it is the whole reason the ruling is written
down: the affordance implies a capability the engine does not have.** Each
button MUST carry a tooltip saying it recomputes the whole pipeline — stages are
not independent. **A control that silently does something larger than its label
says is the defect this project keeps finding**, so this one says it out loud.

**Explicitly NOT ruled: partial recompute is not being built.** It was offered
and declined as Phase-scale — it needs a per-stage dependency graph, cache
invalidation, and golden re-verification per stage, and would want its own scope
document. **Do not re-propose it as part of a tablet pass.**

### Ruling E — the tablet shows the ENGINE’s stage names, not the canvas’s

**The question.** The canvas names its ten stages differently from
`progress.rs::STAGE_NAMES`, which `_assert_stage_names()` actively guards
against drift.

**Ruled: keep the engine’s names.** A recorded, deliberate departure from the
canvas’s labels.

**Why this is the conservative answer rather than the lazy one.** The guard
stays meaningful, the golden fixtures are untouched, and — the part that decides
it — **a user reading a progress bar sees the same words the logs and the error
messages use.** A display-name mapping would have kept the guard working on ids
while making the UI and the logs disagree, which is a debugging cost paid every
time something goes wrong, forever, to win a resemblance.

**Renaming the engine’s stages was also declined:** it touches `progress.rs`,
the guard, every golden fixture carrying a stage name, and the documents quoting
them — **and it changes strings the HTML reference produced, which is the parity
baseline this whole port is measured against.**

**So "100% on tablet" now means: every layout and style, with these two named
exceptions, both recorded at their call sites.**

---

## 2026-09-07 (late) — Ruling F: asset packs keep `.zip` AND their format

**Owner: *"For assetpacks let’s keep the zip and formatting as is (identical to
the html)."*** Given immediately after the project extension became `.ctl`, and
it settles the obvious follow-on question before anyone asks it.

**Two separate things are ruled here, and the second is the one that binds:**

1. **The extension stays `.zip`.** Asset packs are a different artefact from a
   project save. `.ctl` was adopted so the project picker stops offering
   archives the reader will refuse; a pack picker has no such problem.
2. **The FORMAT stays identical to the HTML app’s.** This is the constraint,
   not the filename. **A pack written here must remain readable by the HTML
   app and vice versa**, so nothing about the entry names, the layout or the
   metadata may drift toward this port’s conventions.

**What this forbids, stated because a tidy-minded pass would otherwise do it:**
renaming pack exports to `.ctl` "for consistency"; folding pack entries into
the project archive’s naming scheme; or applying `SAVEFILE_COMPAT.md`’s rules to
a pack — **that document governs project archives and says so; a pack is not
one.**

**Already true in the tree, and that is why this ruling costs nothing.** The
`.ctl` change deliberately split the ten GDScript sites into two populations
and left five alone: asset packs (`app.gd:3338`,
`asset_library_window.gd:1282`/`:3213`/`:3226`), the atlas cache
(`menus.gd:4174`/`:4201`) and tile export (`data_manager_window.gd:3152`).
**The ruling confirms a decision rather than reversing one — which is the good
case, and worth recording precisely because nothing has to change.**

**The atlas cache and tile exports are NOT covered by this ruling.** They kept
`.zip` on the same reasoning — different artefacts — but the owner spoke only of
asset packs, and their format is this port’s own rather than the HTML app’s. Do
not extend the interoperability half to them without asking.

---

## 2026-09-08 — Ruling G, made and WITHDRAWN the same hour

### Ruling G — **WITHDRAWN the same hour it was made.** `action()` does NOT take `2px 12px`

> **WITHDRAWN 2026-09-08, refuted by the batch’s own verifier and then
> confirmed by a fourth independent census. The ruling below is WRONG and is
> kept only because how it was wrong is worth more than the ruling was.**
>
> **The census had the right arithmetic and the wrong population.** The
> canvas *does* disambiguate the action button — the thing this ruling and
> the backlog row both said it does not. It tags every radius-8 node with a
> height role variable, and that role **is** the population:
>
> | Role | Resolves to | What it is | N |
> |---|---|---|---|
> | `--btnH` | 28px | **the action button** | 14 |
> | `--ctl` | 24px | the small inline chip | 15 |
> | `--tool` | 30px | the tool bar | 1 |
>
> **All eight `2px 12px` nodes are `--ctl` (7) or `--tool` (1). Not one is
> `--btnH`.** And `role_px("btn_pad_x"/"btn_pad_y")` feeds exactly `action()`
> and `modal_button()`, both `--btnH`-class — so the ruling moved the inline
> chip’s padding into the button’s slot.
>
> **What `--btnH` actually draws:** `4px 14px` ×3, `4px 15px` ×3,
> `4px 13px` ×3, `6px 18px` ×2, `0 14px` ×2, `4px 12px` ×1 — **y=4 on 10 of
> 14, x=14 on 5 of 14**, the only x with a plurality.
>
> **So the shipped code was already half right and the ruling made it worse.**
> `action()`’s own literal is `10`/`4`; **y=4 is the canvas figure**. Only x
> is open, 10 against a plurality of 14 — and that is the half with the named
> regression history, so it stays guarded and unapplied.
>
> **The gate worked and was overridden.** The brief said a re-count
> disagreeing with 8 of 18 must stop and report. The lane recounted, got 47
> nodes rather than 18, reasoned that *"the rule protects the winner, not the
> literal fraction"*, and implemented. **The winner was itself wrong under the
> population the key actually serves, which is precisely what the gate
> existed to catch.** A stop-and-report gate that is reasoned past is not a
> gate.
>
> **Reverted in the tree**: `btn_pad_x`/`btn_pad_y` are back at the shipped
> `[11, 18]`/`[3, 9]`, and the correct census is written into the comment
> above them. **Four wrong figures have now passed through this one row**
> — `4px 12px`, `3px 11px`, `2px 12px`, and 18-as-the-population — which is
> the argument for counting the population before the plurality.

**Not an owner ruling.** Made by the main loop under the owner’s standing rule
that **the canvases in `design/mcp-2026-09-07/` are the authority for GUI**,
and recorded here so it is not re-litigated in the next brief. **Reverse it
freely** — it is a judgement on a genuinely ambiguous canvas, not a fact.

**Why the row stalled.** It had been re-opened at the canvas on 2026-09-07 and
**both figures previously in play turned out to be wrong.** `4px 12px` occurs
four times in the PC canvas and **on none of the controls carrying**
**`border-radius:8px`** — the shape the shell’s buttons use.
`DccTheme.ROLE["btn_pad_x"]`’s own comment claimed `3px 11px`, but **every**
instance of that in the canvas carries `border-radius:999px`: it is a **pill**
(the add/subtract mode chips, the breakpoint targets), not the action button.
**A false claim sitting in shipping code**, corrected in place with a census.

**What blocked it was that the canvas draws no single control unambiguously**
**"the action button."** The census of what it gives a radius-8 control:

| Padding | Count | Share |
|---|---|---|
| **`2px 12px`** | **8** | **8 of 18** |
| `2px 9px` | 5 | |
| `3px 10px` | 2 | |
| `0 14px` | 2 | |
| `2px 13px` | 1 | |

**The ruling: take `2px 12px`.** It is the canvas’s commonest treatment for
**exactly the shape the shell’s buttons already use**, which is the narrowest
defensible reading of "match the canvas" when the canvas does not name the
control. The alternative — leaving the shipped `10/4`, which matches **no**
census entry — is the one option the owner’s rule clearly excludes.

**It is conditional on the count, and the lane was told so.** If a re-count
disagrees with 8 of 18, the ruling rests on a wrong number and stops. **A
ruling built on a figure that has already been wrong twice in this row gets to
be provisional.**

**This change has a named regression history and is guarded because of it.**
x 10 → 12 widens and y 4 → 2 shrinks; the same class of change shipped **a
265 px tool bar** and **DS-03’s eight over-wide minimums**. `_ds03fit_probe`
and `_ds03shot_probe` run **before and after**, and a regression in either is a
stop-and-report rather than something to tune around. **A laid-out size is not
a minimum** — the rule that class of bug keeps teaching.

---

## 2026-09-12 — urban generation, fortification, and the LOD zoom target

Given through `AskUserQuestion` on 2026-09-12, after the main loop compared the
owner’s two reference images against the code. **Both images are vendored** at
`design/owner-references-2026-09-12/` — `urban-town-plan-walled-market-town.jpg`
and `lod-zoom-target-aletsch-sentinel2.jpg` — so a lane opens the reference
itself rather than a description of it. The option labels quoted below are the
exact labels the owner selected.

### Ruling H — urban generation moves toward the town plan by all three routes, and a re-baseline is authorised

**The owner selected all three options offered:** *"Renderer first
(Recommended)"*, *"New culture profile"* and *"Change the ported algorithm"*. The
third was offered with the description *"Alter grow/fortify/blocks themselves so
every town changes. Departs from the reference and needs a golden
re-baseline."* **So this is the owner ruling that the handoff’s "a golden
re-baseline needs an owner ruling" requires — scoped to urban generation
(`cartalith-urban`) and to nothing else.**

**The target, as drawn in the plan:** a near-circular curtain with evenly spaced
round towers, and gates only where the approach roads cross it; a citadel
straddling the wall; a central market plaza with a well; radial arterials cut by
a few curving cross-streets into legible wedge blocks; lots along each block’s
frontage with open garden courtyards behind; green commons inside the wall beside
the citadel; suburbs strung along the roads outside the gates, thinning with
distance, with a clear gap left against the wall wherever there is no gate; and
strip farmland in parcels of differing orientation, scattered with trees.

**What the code already has, verified 2026-09-12:** a hull-traced curtain with
gates (`fortify.rs::build_wall`); a bastioned star fort behind `opts.fortified`
(`apply_star_fort`); the plaza and radial streets; epoch growth with wall
generations and an extramural share (`growth.rs`); blocks and parcels; a
courtyard building grammar (`districts.rs`); strip and ring fields; and garden
trees, orchards and wells generated by `hinterland.rs::build_details` — **which
`shell/urban_layout_draw.gd` does not draw at all.** Exactly two culture profiles
exist, `medieval` and `venus`.

**Sequencing is the main loop’s, not the ruling’s**, and follows the option
labels. **Renderer first** — draw what the model already generates; no parity
risk. **Then a third culture profile** — additive: the goldens exercise
`medieval` and `venus` by name, and nothing found pins the count at two. **Then
the algorithm changes**, each recorded as a **deliberate departure from the
reference** — what changed, why, and which golden moved — per
`cartalith-porting-discipline`, and never as a widened tolerance. A re-baseline
under this ruling still re-derives the golden it replaces and names it in the
commit.

**All of it sits behind the GUI.** The owner’s instruction of the same day was to
take the GUI rows first, starting with the tablet interface, which has not been
overhauled yet. None of Rulings H-K reorders that.

### Ruling I — a citadel, and star forts that actually generate

Selected: *"Build a citadel"* and *"Enable star forts"*. **The citadel is new
engine work**: a fortified enclosure attached to the circuit — the plan’s sits
astride the wall on the north-east, with its own towers and a single large
building inside. Today only a *"Castle keep"* civic **building** style exists
(`amenities`), not an enclosure.

**Star forts turned out to be reachable already, and the question that produced
this ruling said otherwise.** It told the owner star forts never appear because
settlements carry no `fortified` trait. **Re-opened at the symbols, that is
false:** `civ_settlement_toggle_trait` (the Place Editor’s trait chips, through
`civ_roster_bridge.rs::toggle_trait`) writes the trait, `urban_bridge.rs` reads it
into the town’s `fortified` request, and `generate.rs` grants the fort when the
town is walled, at least `FORT_MIN` in population and on the `organic` gate
scheme. **What is actually missing is the drawing:**
`urban_layout_draw.gd::_draw_wall` branches on `curtain`, `palisade` and `ditch`
only, and its own comment says the `bastioned` branch was left out because *"this
port’s settlements carry no traits"* — a reason that went stale when trait
toggling shipped. So a settlement marked Fortified can generate a star fort that
is drawn as a plain curtain. **The ruling stands; the work it implies is smaller
than the question made it look.**

### Ruling J — a menu to set a settlement’s city type and regenerate that settlement alone

The owner’s own words, typed into the fortification question: *"Let’s create a
menu to modify city types and be able to regenerate a specific settlement at
will"*. **New GUI capability, verified absent:** `city_viewer_window.gd` carries no
culture, city-type or regenerate control, and `urban_bridge.rs::urban_layouts`
takes settlement indices and nothing that selects a profile. **No canvas draws
it**, so it needs a design derived from the DCC canvases’ own vocabulary (owner
ruling, 2026-08-25) before it is built — and as GUI work it falls under the
standing definition of done.

### Ruling K — the LOD zoom target is the Aletsch image, top-down; sharpness first, then ice

Selected: *"Top-down equivalent (Recommended)"* and *"Sharpness first, then ice
(Recommended)"*. **3D stays parked** (2026-08-31): the target is the image’s
relief, snow, ice and vegetation as read from above, not its oblique camera.
**The order is ruled.** First port `renderBiomeTileRGBA`, so LOD tiles carry
colour rather than a shade ratio — without it no zoom level can add detail. Then
draw ice from fields that already exist: `glacial_kernel`’s carved troughs, the
`passes.glacial_snowline` parameter, and the snow and rock material fractions.
**A glaciation model — ice extent, flow, moraine stripes — was offered and not
chosen.** The owner-supplied research `docs/research/lod extra info.md` is the
design input for both steps.

---

## 2026-09-13 — Ruling L: the PC left rail, re-sorted by the owner

**The owner re-sorted every category and control on the PC left rail**, starting from the code-derived
inventory of 2026-09-12. Vendored verbatim at `design/owner-references-2026-09-12/left_rail_tree_resorted.md` — **the tree in that file is the
specification; this entry records only its standing rules and the owner’s resolved decisions.**

**Grouping rules (the owner’s, abridged):** WORLD ▸ PIPELINE holds what the seed-driven pipeline reads
or writes, in stage order; WORLD ▸ SCULPT holds everything done by hand on the map surface (height
molding and biome painting), and **the mode pill is the only gate**; CIVIL is anything about people;
CARTO is how the map is drawn, and **every visibility toggle lives in Layers**; each tab’s Tools row
lists only tools that can be armed there.

**Resolved decisions, as the owner wrote them:**
1. Droplet Erode, Carve fjords and Center landmasses stay in Hydrology — on-demand passes grouped with
   the domain they act on, not with Sculpt.
2. **Biome paint lives in WORLD ▸ Sculpt ▸ Biomes (left dock).** The right-dock copy is a duplicate, to
   retire once Sculpt ▸ Biomes is live; Biome paint (B) in the Tools row stays the way to arm it.
3. **Resources is removed** — its values are calculated, not set; stage 10 stays a read-only row in
   Pipeline status.
4. **Linked vault notes live under CIVIL, per entity** — never a standalone category; Continents’ notes
   move from WORLD ▸ World data to Territories ▸ Linked notes. **Standing rule for any future note link.**

**It supersedes one earlier instruction, in mechanism not intent.** 2026-09-07: sculpt tools appear only
when the sculpt menu is accessed, implemented as `_sculpt_body.visible = armed_tool == "sculpt"`. The
re-sort makes **SCULPT mode itself** that gate and has picking a feature arm the tool, so the
armed-tool gate goes.

**Two things a builder must know, checked by the main loop when this was recorded:**
- The tree carries the inventory’s *"(disabled: no binding)"* notes on **Erode (droplet)** and **Count
  painted lakes as water**. **Those notes are wrong** — both guards ask the live `WorldGen`, both methods
  are exported `#[func]`s, and both buttons are enabled. Carry the controls over; drop the notes.
- **Decision 1 keeps Center landmasses in Hydrology, while the tree places it in Generate ▸ Run.**
  **Owner, 2026-09-13: "Tree is leading"** — where the notes and the tree disagree, the tree wins;
  Center landmasses goes to Generate ▸ Run.

---

## 2026-09-13 — Ruling M: Reference Map Reconstruction Mode (owner answers to the research's eleven questions)

**The research is `REFERENCE_MAP_RECONSTRUCTION_RESEARCH.md`** (its questions are §5). The owner's answers, verbatim where given, and what each changes in the proposed design:

1. **Plate-edge roughness — "Tectonic alpha might be it."** **Checked at the symbol, 2026-09-13: it is not an edge control.** `tect.alpha` ("Tectonic α", 0–1.2) is used only in `cartalith_terrain::compute_height`, as `0.5 + α·(0.40·base_field + 0.50·stress) + …` — it scales how strongly plate base and boundary stress raise **height**. Plate edges in the generator are shaped only by warp (`compute_warp`, large-scale bends), Lloyd relaxation and plate count. **Answer 2 makes this moot:** with drawn boundary lines, roughening means displacing the owner's own lines — the generator's warp formula (amplitude) plus a finer detail octave from the same noise. **Confirmed by the owner the same day ("yes"):** edge roughness on drawn lines is **Edge warp** (the generator's `compute_warp` formula, amplitude) **plus Edge detail** (a finer octave from the same noise functions); α stays a height parameter.
2. **How plates are drawn — "Draw boundary lines."** Not sites or painted regions. The user draws boundary polylines over the reference; plates are the regions those lines enclose (closed against the map edge), and the roughening in 1 displaces the lines. Design consequence: the constrained assignment becomes a line-bounded region fill (labelling cells between drawn lines), not a site-seeded Voronoi; lines that do not close a region are reported, never silently joined.
3. **Do drawn plates change height — "No, is only to inform the resources generation step."** Apply rebuilds only the plate-derived substrate the resources step reads, and never touches height, hydrology or climate. **RM-11 ("grow relief from plates") is dropped.** **What "the resources step" reads from plates, checked at the symbols 2026-09-13:** `cartalith_civ::build_resource_potentials` takes `boundary_type` (subduction and ocean–ocean arc cells seed the copper distance field), `shear_field`, `age` and `volcanic`, plus lithology; `cartalith_civ::build_lithology` takes `age`, `volc`, `crust` and `resist`. So Apply must rebuild exactly `plate_id` → boundary mask and **type**, **shear**, stress, **crust**, **age**, **resistance** and **volcanic** from the drawn lines, then mark civ stale so resources recompute — and nothing else.
4. **Plate crust and drift — "As proposed":** inferred from the sculpt (`classify_plate_crust`, `infer_plate_velocities`), with optional per-plate overrides.
5. **Reference image storage — "embed in the save file."** The original image bytes go into the project zip (`annotations/reference.<ext>`), with a size warning.
6. **Registration — "Scale rotation and offset only."** No perspective or rubber-sheet warping.
7. **Placement and canvas — "Not yet, will be the first question when we start this."** No canvas is commissioned now; the first step of the build is the placement/canvas question to the owner.
8. **Editing opened projects — "Automatic, sculpting should always be available."** An opened project becomes editable on open, without a "Continue editing" action: RM-0 promotes a loaded world automatically (rebuilding the substrate, keeping the saved climate). Its open-time cost must be measured and stated; if it is large, the rebuild may run lazily on first edit, but the user never has to ask for it.
9. **Detection scope — "yes, what else would be possible?"** v1 is land/water and rivers. Possible later classes, recorded for the owner: mountain/hill symbols (hachures, icons) as uplift hints; forest and biome colour regions (seeding biome paint); roads and borders as lines; lakes as a separate water class; shallow-sea shading as shelf depth; settlement markers; painted hillshade as a rough height guess; labels by OCR (the least robust).
10. **Formats — yes:** PNG/JPEG/WebP decoded by Godot, PNG also through Rust; source cap 4096 px on phone, 8192 px on desktop.
11. **Export — yes:** the reference image is never drawn into any export.

**Not scheduled.** This ruling records the design decisions; build rows wait until the work is started, whose first step is question 7.

## 2026-09-20 — Ruling N: settlement river/coastal binding uses real geometry, not a proxy

**The finding is `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` EF-7.** The owner, discussing that document's design: *"a settlement should be properly rendered on a coast and along/around a river."* A targeted investigation (read-only, checked at the symbol, not assumed) confirmed this is real and precisely locatable, in both `Cartalith Gen1 v2.11.html` and this port, byte-for-byte the same design:

- **Siting** (`cartalith-civ`'s `build_settlement_suitability`'s river term, `civ_is_coastal`) decides "has river"/"has coast" from per-cell statistical proxies — flow accumulation, Strahler order sampled at the settlement's own cell, distance to the nearest ocean cell — with no reference to any real, connected waterway or coastline. A cell can score full river marks from locally-high flow that belongs to a disconnected channel.
- **Rendering** (`cartalith-urban`'s `build_site`) does pick one real traced river polyline for local layout, but its binding test is `riverPath` truthiness — true for an empty or one-point path. A **known, deliberately-reproduced** bug (`URBAN_MORPHOLOGY_SCOPE.md` milestone 5, finding 3, golden-pinned as `pathOfOne`/`pathEmpty`), carried for parity, never recognised as something to fix.
- No scope document anywhere names this as a defect to close — the existing paper trail treats the proxy design as parity to preserve.

**Owner ruling, 2026-09-20 — the large option, with one explicit constraint:**

> "From the description I'd say Large. Keep in mind that this should also always be in proximity of the best settlement locations as per the layer for it."

Siting itself changes, not just the downstream render binding: a settlement only scores as river/coastal when a real, connected waterway or coastline genuinely reaches it. **The existing suitability ranking stays the authority for where settlements go overall** (food, defensibility, resources, and the rest) — the fix changes what the river/coastal *term inside that ranking* measures, from a raw proxy value to real-geometry proximity. It is not a separate gate that bypasses or overrides the ranking.

**What this authorises, concretely** (full detail in EF-7):
- The suitability river term becomes distance-to-nearest-point-on-a-real-traced-river-polyline (`cartalith_hydrology::trace_river_polylines`'s existing whole-world output — no new field needed for this half), not a raw flow/order sample.
- The coastal term becomes the equivalent check against a real traced coastline — **blocked on EF-6** (coastline vectorisation, not yet built). The river half has no such dependency and can proceed first.
- `build_site`'s `riverPath`-truthiness bug is fixed in the same pass, same family of defect: require a minimum real length before a site draws river-bound. `pathOfOne`/`pathEmpty` are the golden fixtures this deliberately re-baselines.

**Explicit deviation, disclosed per `DECISIONS.md`'s own rule, not assumed correct.** This re-baselines `build_settlement_suitability`'s and `civ_is_coastal`'s golden tests and ripples into anything downstream of settlement placement — economy, urban layout, faction territory. That blast radius is why this got a design pass and a recorded ruling before any build, rather than being built straight from the owner's word.

**Not scheduled as a build yet.** The river half is unblocked; the coastal half waits on EF-6. Neither has a build row in `OUTSTANDING_WORK.md` as of this ruling.

*Stale, noted 2026-09-24: both halves have since been built.* The river half is `649897f` (the suitability river term reads real traced river geometry, and `build_site`'s `riverPath` truthiness fix went in with it, re-baselining `pathOfOne`/`pathEmpty`). The coastal half is `9ad4399` (`build_coast_reach`, proximity to EF-6's traced coastline, is the suitability coastal term). **`civ_is_coastal` was deliberately left as a proxy.** It is the port and sea-lane eligibility gate, not the suitability term, and `9ad4399` scoped Ruling N to suitability only, so the "re-baselines `civ_is_coastal`'s golden tests" sentence above did not happen as written. Its east-west wrap on non-wrapping maps was a separate defect. Ruling AR fixed it (`c4435ee`: the `world` argument to `civ_is_coastal`). Status lives in `cartalith-native/docs/STATUS.md`.

## 2026-09-21 — Ruling O: v2.69's tile-refinement sea-level clamp lands

**The finding is `OUTSTANDING_WORK.md`'s v2.69 row, checked at the symbol and measured, not merely "confirmed inherited."** `amplify_region`'s underwater term already tapers going down into water and measures symmetric (0.32%/0.32%) at that layer alone — fine. One layer up, `add_zoom_detail` (the deep-zoom octave-stacking pass) gives a water cell zero extra octaves (`if base < sea { continue; }`) while a land cell gets up to six *unclamped* extra octaves with no `[0,1]` clamp on the write-back. Measured on a synthetic coastal gradient: 0.00% land→sea at z=2 rising to 0.17% at z≥4, and 0.00% sea→land at every level tested — strictly one-directional, and more extreme than the HTML's own asymmetry because this port's water cells are fully exempt rather than merely tapered.

**Owner ruling, 2026-09-21: land the fix.** Both functions clamp the excursion toward sea level at half the remaining headroom, per `RC_ENGINE_CHANGES.md`'s own v2.69 entry — fixing only one left a third of the drift in the HTML's own report, so both move together. `cartalith-terrain/tests/golden_parity_amplify.rs` currently pins the asymmetric behaviour byte-exact against the frozen v2.11 reference; that golden is authorised to move, re-derived rather than a tolerance widened, with the change recorded at the symbol per `cartalith-porting-discipline`. This departs from the frozen reference's own generated pixels near every coastline — a real, disclosed parity deviation, not a silent one.

## 2026-09-21 — Ruling P: CA-19, the biome colour table becomes user-editable

**The finding is `OUTSTANDING_WORK.md`'s CA-19 row.** The biome colour table is buildable as a writable, user-facing control today; the only reason it wasn't already exposed is `DECISIONS.md` §7a's general protection against moving a golden without a ruling.

**Owner ruling, 2026-09-21: authorise it.** The fixed palette becomes user-editable; whatever golden currently pins the fixed colours as a constant is re-baselined to treat the table as data rather than a literal, with the change recorded at the symbol.

## 2026-09-21 — Ruling Q: water bodies reclassify by hydrological topology, not size

**The finding is `HYDROLOGY_CLASSIFICATION_RESEARCH.md`, imported verbatim 2026-09-08, and `OUTSTANDING_WORK.md`'s own row on it.** `build_water_bodies` calls the *largest* below-sea connected component the ocean and every other one a lake — size-primary, relative rather than an absolute threshold, but size all the same, with no connectivity, basin-topology, connection-type or map-boundary state anywhere in it. Three failure modes follow directly: a world with little ocean and one huge inland basin makes the lake the ocean (largest wins); a genuinely marine body truncated by the map edge that is not the largest component becomes a lake (the research's `MAP_BOUNDED`/`UNRESOLVED` case, unrepresented here); and a Caspian-shaped case (a saline, connectivity-isolated sea) is unrepresentable in either direction, since salinity is not a stored property and `endorheic` is not a state.

**The additive first step — deriving `ocean_connected`/`map_boundary_contact`/`basin_type` beside the existing classification, changing nothing it classifies — was already built and verified 2026-09-21 (commit `c6de2a2`), byte-identical, no golden moved.** `inflow_count`/`outflow_count`/`salinity` were deliberately not attempted there; they need a river/flow network the topology function doesn't carry.

**Owner ruling, 2026-09-21: authorise the full reclassification, not just the additive metadata.** `build_water_bodies` moves to a topology-primary rule per the research. This re-baselines `build_water_bodies_largest_below_sea_component_is_ocean` and ripples into `build_biome_raster`, route costing (`RouteContext::water_bodies`), lake labelling (`lake_features`, keyed on `== 2`) and landmark placement — every downstream consumer of `WaterBodies::classification`. Each moved golden is recorded at the symbol, what changed and why, per `cartalith-porting-discipline` — not a tolerance widened.

## 2026-09-21 — Ruling R: IN-13 trade flows use per-faction currencies with exchange rates

**The finding is `OUTSTANDING_WORK.md`'s IN-13 row.** Trade flows (who trades with whom, prices, tariffs, caravans as entities) cannot be built without first deciding what "currency" means in this world — `TradeBalance` already names *what* moves, never *who* holds it or in what unit.

**Owner ruling, 2026-09-21: each faction has its own currency, with an exchange rate between any two.** Cross-faction trade needs a conversion step at the point of exchange, not a single universal unit of account. This is a design decision at the start of a large, unbuilt subsystem — no golden exists yet to move, and the build itself is not scheduled by this ruling; it only settles the question that was blocking a design.

*Status note, 2026-09-24:* this ruling is **still unbuilt**. What shipped instead was Ruling AB's single world-wide scarcity price index, with Ruling AE's tariffs (`bbc255f`). No `currency` or `exchange_rate` exists in any crate: the only matches are doc comments saying that a value is *not* a currency. Ruling AR (2026-09-24) confirms that R stands and is not superseded, and that per-faction currencies are built on top of AB's pricing.

*Stale, noted 2026-09-27: built as Ruling AU shaped it* -- a user-set name, symbol and rate per faction on the roster (`civ_roster_bridge.rs::FactionCurrency`), conversion only in `cartalith_civ::currency`, saved in `entities/factions.json` (`SAVEFILE_COMPAT.md` §9.2). Where it stands is `STATUS.md`'s to say.

## 2026-09-21 — Ruling S: v2.71 half 2, woodland as a spatial area — declined

**The finding is `OUTSTANDING_WORK.md`'s v2.71-half-2 row (`RC_ENGINE_CHANGES.md` §8.2).** This port has zero spatial representation for woodland anywhere — only an abstract `woodland_ha` hectare figure feeding trade/fuel economics. The RC spec states two behavioural rules (arable wins in a conflict; no street-crossing guard, unlike farmland) but names no siting algorithm, polygon shape or density constants, and `Cartalith_RC` was not reachable from this session's filesystem to check the real `buildFarmland` v2.71 code. The row asked the owner to choose between waiting for the real source or building a non-ported placeholder now.

**Owner ruling, 2026-09-21: disregard woodland as a spatial area, for now.** *"I'm not satisfied with it in the HTML version."* Not a resourcing deferral — the owner has looked at how the reference itself handles woodland and does not want it ported as-is, faithfully or not. Moved to `OUTSTANDING_WORK.md` §5 (declined and shelved). Reversible by a word, but a future re-proposal should design something the owner finds satisfying rather than default back to a faithful port of the reference's own woodland placement.

## 2026-09-21 — Ruling T: hydrology's world-wrap case gets its own rule, size-primary

**The finding is `OUTSTANDING_WORK.md`'s own follow-up row to Ruling Q**, flagged by the agent that built Ruling Q rather than decided silently: the topology-primary boundary-touching-is-ocean rule was applied uniformly to `world=true` (toroidal) maps, where the X edge is not a real edge at all — `HYDROLOGY_CLASSIFICATION_RESEARCH.md` never addresses wrapped-world topology, so this was a judgment call, not something the research dictated. Confirmed to move real output: every golden file's `case_1_world_wrap` fixture moved under the uniform rule.

**Owner ruling, 2026-09-21: special-case `world=true`.** A wrapped map keeps the old size-primary (largest-below-sea-component-is-ocean) rule; only bounded (non-wrapped) maps use Ruling Q's boundary-touching rule. Needs a build: `build_water_bodies` branches on `world`, and `case_1_world_wrap`'s golden fixture in `golden_parity_waterbodies.rs` (and any of the twelve other re-baselined files whose fixtures are wrapped-world) reverts toward its pre-Ruling-Q values for the wrapped case specifically — re-derive, don't guess, per the standing discipline.

## 2026-09-21 — Ruling U: Refine detail moves to Preferences ▸ Tiles & LOD

**The finding is `OUTSTANDING_WORK.md`'s §3.1 row.** A 2026-09-05 ruling put "Refine detail for the current view" on the WORLD rail beside Bake & finalize; the 2026-09-07 design canvas doesn't draw it there at all, and its GENERATE panel explicitly routes atlas/LOD work to Preferences ▸ Tiles & LOD instead. The canvas postdates the ruling.

**Owner ruling, 2026-09-21: follow the canvas.** Move the control from the WORLD tool-options bar (`app.gd::_tool_options_generate()`) to Preferences ▸ Tiles & LOD. This is the standing "an owner decision is newer than any canvas" rule read the other way around — here the canvas is newer than the ruling it revisits, so the canvas wins.

## 2026-09-21 — Ruling V: GeoJSON import creates an unknown faction rather than remapping or dropping it

**The finding is `OUTSTANDING_WORK.md`'s `FUNCTIONAL_CONTRACT.md` DM-03 row.** The GeoJSON parser is built and hardened (`cartalith_io::parse_geojson`, verified against a 25-case table plus a 20 000-level nest); applying an imported document stalled on one open question: what happens when an imported feature names a faction this world does not have.

**Owner ruling, 2026-09-21: create it.** An unknown faction name in an imported document becomes a new faction in this world, preserving the import's own intent rather than remapping by fuzzy name match or silently dropping authorship into "unclaimed." Still owed alongside the apply logic itself: the Data-manager Import route in GDScript, which doesn't exist yet.

## 2026-09-21 — Ruling W: the pack-import unused-section warning names all four undrawn sections

**The finding is `OUTSTANDING_WORK.md`'s `FUNCTIONAL_CONTRACT.md` cap. 6 row**, re-derived 2026-09-13: the pack-import warning names only `trait` as unused, but `structures.settlement`, `structures.poi`, `custom` and `seamarks` art are every bit as undrawn — none has a compositor reaching the live map. `manifest.rs`'s own comment above `unused.push("trait")` said widening the list was a behaviour decision the owner had not made, and is golden-pinned (`golden_parity_pack_manifest.rs`, `golden_parity_pack_zip.rs`, plus a test asserting the OLD behaviour, `settlement_and_poi_are_not_named_by_the_unused_warning`, which this ruling makes false by name).

**Owner ruling, 2026-09-21: name all four.** Honest disclosure now; drawing any of the four into the live map composite is separate, unauthorised future work. Needs a build: extend the `unused` push-list in `manifest.rs`, invert or remove the now-false test, and re-derive whatever golden fixture pins the warning string's exact wording — the standard re-baseline discipline, not a silent string change.

## 2026-09-21 — Ruling X: `Data ▸ Export ▸ Maps ▸ tiles` stays in the export menu

**The finding is `OUTSTANDING_WORK.md`'s own row, deliberately not ruled on by the 2026-09-07 menu audit that found it.** Ruling 29 (*"the tiled output should only live in the save menu. It has no merit in the export menu"*) scopes its own body to the LOD pyramid and the proposed Build Manager. `Data ▸ Export ▸ Maps ▸ tiles` is a different artefact — a region-marquee PNG tile grid (`PANE_PURPOSE.export_maps`), not the LOD pyramid — and the row's badge, tooltip and code all describe that different thing.

**Owner ruling, 2026-09-21: Ruling 29 does not cover this row. Leave it in the export menu.** No code change — this closes the ambiguity, not a build.

## 2026-09-21 — Ruling Y: build GPU device reuse across generations, with explicit device-loss handling

**The finding is `OUTSTANDING_WORK.md`'s own row, re-scoped by measurement 2026-09-03.** Six pipeline builds total 2.60 ms against a device handshake of several hundred milliseconds — the device, not the pipelines, is where the reuse value is. Holding a `wgpu::Device` alive between `generate_terrain` calls changes lifetime and failure semantics around Wgpu's `lost` flag, which this project has already measured losing on `forward_plus`/Vulkan.

**Owner ruling, 2026-09-21: build it, with a real recovery path for device loss** — not a feature that assumes the device never dies. The `lost` flag must be handled explicitly (re-acquire the device and retry, or fall back to CPU for that generation, rather than an unhandled panic/hang) since this project has already measured hitting it.

## 2026-09-21 — Ruling Z: shrink the phone MAP tab's half-open detent to fit its real content

**The finding is `OUTSTANDING_WORK.md`'s own row, reported and deliberately left unchanged 2026-09-06.** The half-open detent height (`PHONE_DETENT_HALF_FRAC = 0.46`) was transcribed faithfully from the prototype's own `Math.round(fh*0.46)`. The sheet it opens is ~92% blank regardless of world state (933 of 1 003 rows blank with no world, 912 WITH one) — not an empty-state bug, genuinely more vertical space than the real content needs.

**Owner ruling, 2026-09-21: shrink the detent to fit the real content.** A deliberate deviation from the transcribed prototype value, authorised because the prototype's own proportion does not describe this content's actual size. Needs a build: re-derive the half-open fraction (or switch to a content-sized detent if this shell has that mechanism elsewhere) rather than the flat transcribed 0.46.

## 2026-09-23 — Ruling AA: the urban-algorithm town-plan row's radial arterials extend the radial branch, not organic `grow`

**The finding is `OUTSTANDING_WORK.md`'s "change the ported urban algorithm toward the owner's town plan" row (Ruling H), candidate 1.** A scoping pass found the wedge-block geometry the owner's reference image shows (radial arterials cut by a few curving cross-streets) already exists on `cartalith-urban`'s Venus/`"radial"` branch (`radial.rs::build_radial_streets`, sharing `blocks.rs`'s bisector platting with the medieval branch) — but radial towns never call `grow` and are excluded from the wall-lots/faubourg suburb machinery (`generate.rs`: gated on `profile.planning != "radial"`), so the owner's combined image (wedges + suburbs + wall gaps together) needed a choice: extend the radial branch to carry that machinery, or reshape organic `grow`'s own macro-structure instead.

**Owner ruling, 2026-09-23: extend the radial branch.** Port/open the suburb-thinning and wall-gap machinery (already built and verified for the organic branch — commits `5d780dc`/`21c6949`) onto `"radial"` towns, rather than reshaping `grow`'s macro-structure. Lower blast radius: touches the 4 of 29 orchestration golden cases that are Venus/radial, not the 25 organic-branch cases a `grow` reshape would put at risk.

## 2026-09-23 — Ruling AB: IN-13 trade prices are scarcity-derived from `TradeBalance`

**The finding is `OUTSTANDING_WORK.md`'s IN-13 row's remaining "prices, tariffs" piece.** No price exists anywhere in the engine today (`TradeFlow::volume` is a physical demand quantity, not monetary); the obvious candidate input is `TradeBalance`'s already-computed per-good surplus/deficit, but using it as the price basis is a design call, not a code-derivable default — the alternative was a new fixed per-good table, which the reference has none of to port (values would be invented).

**Owner ruling, 2026-09-23: derive price from `TradeBalance`'s existing surplus/deficit (scarcity-based).** No new authored data. This still leaves open how the derivation itself is shaped (the specific curve from surplus/deficit to a price number) and the three other IN-13 blockers unresolved (tariff-rate source — reuse `civ_faction_relations` or a new field; confirming a faction-aware match doesn't move the existing single-faction/no-tariff probe output; caravan entity semantics) — this ruling settles only the price-basis question.

*Status note, 2026-09-24:* the price is built (`bbc255f`, each `TradeFlow` carries `price`). The other blockers were ruled by AE (tariffs), AF and AP (caravans) and AQ (sea lanes).

## 2026-09-23 — Ruling AC: a citadel enclosure is sited on a settlement's size tier, not faction-seat status

**The finding is `OUTSTANDING_WORK.md`'s "a citadel enclosure straddling the town wall" row (Ruling I), whose own text named "settle the design first: which settlements get one" as a separate blocker independent of the GUI-sequencing gate (which was lifted 2026-09-22).**

**Owner ruling, 2026-09-23: largest settlements only, by size tier — not gated on faction-seat status.** Matches how the existing rare bastioned-wall star fort is already gated by size/class. Still open: whether the citadel's area counts inside the wall circuit for growth purposes (the row's own second named question), and the citadel's own build (nothing like it exists yet — `fortify.rs` has no castle-enclosure construction, only the curtain/gates/spurs/star fort).

*Stale, noted 2026-09-24: the citadel is built.* It is `cartalith_urban::citadel::build_citadel`, gated in `generate.rs` on `CITADEL_MIN_POP` (this ruling's size tier), in `9467a23`. Whether the citadel's area counts toward growth is still unruled.

## 2026-09-23 — Ruling AD: courtyard-perimeter blocks are gated by outermost ring, distance from market

**The finding is the urban-algorithm town-plan row's candidate 4, "perimeter lots with open courtyards held at high density"** — real, un-shipped work (courtyard buildings exist today only as per-parcel typology on strip lots, not the ring-around-a-shared-courtyard block subdivision the owner's image describes), blocked only on what "perimeter" means structurally: the outermost ring by distance from the market/town centre, a district flag, or a rules threshold.

**Owner ruling, 2026-09-23: the outermost ring by distance from market.** Matches how other density gradients already keyed on market distance work in this engine (e.g. Clark's demand-decay gradient in `growth.rs`). Not yet built — this ruling settles only the gating definition.

*Stale, noted 2026-09-24: built* as `cartalith_urban::courtyard::build_courtyard_rings`, which `generate.rs` calls with `anchors.market` (`ae6a8c8`).

## 2026-09-23 — Ruling AE: IN-13 tariffs get their own relationship field, not `civ_faction_relations`

**The finding is IN-13 trade flows' second remaining sub-question**: whether a cross-faction tariff rate reuses `civ_faction_relations`'s existing pairwise score, or needs a dedicated field — `civ_faction_relations`'s own module doc is explicit that it deliberately is not diplomacy/treaties/vassalage, so reusing it for tariffs would repurpose a value past its stated scope.

**Owner ruling, 2026-09-23: a new relationship field**, not a reuse of `civ_faction_relations`. Not yet built — this settles the source, not the field's own shape or default values. Two of IN-13's four original sub-questions remain: confirming the faction-aware trade match doesn't move the existing single-faction/no-tariff probe output (a verification constraint, not an owner decision), and caravan semantics (settled below, Ruling AF).

*Stale, noted 2026-09-24: built.* `cartalith_civ::trade::Tariff` and `tariff_rate` shipped with AB's pricing in `bbc255f`.

## 2026-09-23 — Ruling AF: one caravan is one aggregate shipment per way

**The finding is IN-13 trade flows' fourth sub-question**: what a "caravan" entity represents — one `TradeFlow` row, an aggregate per way, or a manually-initiated shipment — which decides whether caravans are a visualization of the existing stateless trade match or a real simulated, persisted thing with its own movement/consumption rules against the Timeline's year cursor.

**Owner ruling, 2026-09-23: one aggregate shipment per way** — a periodic bundle of all matched trade volume currently routed over one way, not one entity per individual flow and not a manual player action. Closest in shape to `cartalith_civ::travel_library::Journey` (route + start_year + a persisted DTO), fewer entities than a per-flow model, easier to keep in sync with `trade_flows()`'s own deliberately stateless match. Not yet built — the exact tick/refresh cadence against the trade match and the Timeline's year cursor is still an implementation detail for whoever builds it.

*Superseded in part, noted 2026-09-24.* **Ruling AP** (2026-09-23) kept "one per way" and replaced the persisted, `Journey`-shaped entity with a **derived view**: one row per loaded way, rebuilt from the live trade match, nothing saved. So the cadence question above no longer arises. The land half is built as that view (`da51a57`: `TradeNetwork::way_goods`, and `civ_trade_flows`'s `caravans` array). Ruling AQ gave sea lanes one caravan each; that half is not built.

## 2026-09-23 — Ruling AH: keep 2048×1311 as the Android resolution ceiling

**The finding is `MEMORY_OPTIMIZATION_SCOPE.md` §8's own row**: 4096px needs 2.41 GiB and 8192px needs 9.65 GiB, so 2048×1311 is the last preset that fits Android's real memory budget; the doc twice declined to change `RESOLUTION_PRESETS` unilaterally.

**Owner ruling, 2026-09-23: keep 2048×1311 as the Android ceiling.** Higher presets stay desktop-only where memory allows. Documentation-only — no code change needed.

## 2026-09-23 — Ruling AI: fix military manpower's era-fixed standing-army ratio, and make it vary by agricultural/industrial development too

**The finding is `MILITARY_MANPOWER_SCOPE.md`'s finding 2**: on sparse worlds (33 settlements), standing armies read 0.19–1.20% of population against a 1–2.5% target band, traced to the model sitting at Imperial Rome's fixed ratio regardless of era — the era table's own "standing" column was meant to vary, and never has. Fixing it moves outputs already validated against the owner's worked example.

**Owner ruling, 2026-09-23: fix it, and go further — the ratio should also vary by how agricultural vs. industrialised a nation is**, in the owner's own words: *"a heavily industrialised nation needs less manpower to foot a larger army than an agricultural nation that is dependant on individual farmers without machines."* **This maps directly onto an axis the engine already has**: `roster::AG_TECH_LEVELS` (6 tiers, `farmers_per_urbanite` derived from a real historical series — England's agricultural-labour-share data, CAMPOP/Broadberry & Gardner 2013 — already a per-faction field, `FactionEntry::ag_tech`, defaulting to `"traditionalAgrarian"`) — no new classifier needed, reuse it. Re-validate the fix against the owner's original worked example within a stated tolerance, not just against the sparse-world band.

**Correction, 2026-09-23, same day — this ruling's own premise was false.** A dispatched build read `manpower.rs` fresh (last touched `45630cc`, 2026-09-06, untouched by anything today) and found there is no era-fixed ratio: `standing = total × (1−α) × ecological_factor × fiscal_extraction_efficiency / SOLDIER_UPKEEP`, where `α = f/(1+f)` already comes from `AG_TECH_LEVELS.farmers_per_urbanite` — the exact mechanism this ruling asked for **already exists and is strong**, measured at an equal population/land/institutions: 1 090 (subsistence) → 2 219 (traditional) → 24 617 (industrial) standing army — **11.09× over traditional, 22.6× over subsistence** (arithmetic corrected 2026-09-23: this read "1 090 → 24 617, an 11.09× spread", but 24 617 / 1 090 = 22.6×). `ERA_BANDS` is read only for the pass/fail *verdict*, never as an input to the headcount — `era_for` is an *output*, which is what "era should not be the driver" actually asked for. "Imperial Rome's ratio" in finding 2 described a measured *result* at one ag-tech level, not a hardcoded constant — the only literal ratio in the chain is `SOLDIER_UPKEEP = 3.0` (Roman legionary pay against a subsistence wage, per its own doc, unrelated to era). No behaviour was changed; one mutation-tested unit test was added pinning the 11.09× spread. Re-measured with today's date (post Ruling 11's ecological-ceiling raise and today's government-default wiring): the sparse 33-settlement world now reads 3 of 6 factions within band (was 1 of 6 in the stale finding-2 text), and a 108-sample sweep across seeds/world-sizes found most of the remaining "below band" cases trace to a *different*, already-known effect (finding 3's map-scale/ecological-factor interaction: 33 of 36 below-band cases on an 800 km world vs. 11 of 36 on a 2000 km one) — not era, not industrialization. **Options for a fresh ruling**: (a) accept the ag-tech rule as already satisfied, correct finding 2's stale figures; (b) take on the map-scale normalization of land-capacity÷population separately (its own ruling, not this one); (c) rule on whether the era table's own band shape (non-monotone in α — the Iron Age band sits *above* the High-medieval one) should be reproduced by re-fitting `SOLDIER_UPKEEP` to vary with α, which would re-baseline the owner's original Kingdom A/B worked example. **`MILITARY_MANPOWER_SCOPE.md` finding 2 and this ruling's own "never has" wording are both stale** and need correcting once the owner picks an option.

**Owner ruling, 2026-09-23, same day: all three.** (a) The ag-tech scaling is accepted as already satisfying the original ask — that piece of finding 2 closes. (b) Take on the map-scale normalization of land-capacity÷population as a real follow-up build. (c) Re-fit `SOLDIER_UPKEEP` to vary with the ag-tech share (α), reproducing the era table's own non-monotone band shape, accepting that this re-baselines the original Kingdom A/B worked example — re-derive it, don't just widen a tolerance around the old numbers.

## 2026-09-23 — Ruling AJ: add the save-format byte-plane shuffle; leave lossy u16 quantization alone

**The finding is `STATUS.md`'s save-compression row**: a byte-plane shuffle would make saves 27–36% smaller and write faster (lossless), but needs a `format_version` bump and ends the save format's current "bare dump, no transformation" promise (`SAVEFILE_COMPAT.md` §8). A separate, lossy option — quantizing saved rasters to `u16` — was also on the table, currently barred by this project's own parity-testing rules.

**Owner ruling, 2026-09-23: add the byte-plane shuffle. Do not add u16 quantization** (not asked for, stays barred). `SAVEFILE_COMPAT.md` §8's bare-dump promise needs correcting to record the shuffle as a deliberate, disclosed departure, the same way every other save-format decision in that document is recorded.

**Built same day, commit `027248c`.** `PROJECT_FORMAT_VERSION` 1→2; fail-loud by entry name (`rasters/heightmap.shuffled.f32`), not version number, so a pre-change reader refuses the archive under §6.4 rather than misreading shuffled bytes. Real backward-compat proof: a genuine v1 archive generated with the unmodified pre-ruling writer reads bit-identically and re-saves cleanly to v2. Measured 25.2–36.3% smaller across three grid sizes, writes faster too. `SAVEFILE_COMPAT.md` §8/§18 corrected in place.

## 2026-09-23 — Ruling AG: unify label glyph layout on `map_overlay.gd`'s own (more recent) font-size model

**The finding is the `label_glyph_layout` row** (`UNWIRED_FUNCTIONS.md`): two competing label-layout implementations exist — the engine's `label_glyph_layout`/`arc_label_layout` (Rust, `cartalith-civ/src/labels.rs`, introduced `29d0f50`/`611c5fa`, 2026-08-18) and `map_overlay.gd`'s own GDScript re-implementation with a different font-size model (introduced `fd9de7c`, 2026-09-01, **two weeks later**). `label_box_at`/`label_handles` already size off the engine's (older) font model, while the drawn glyphs size off the GDScript file's own (newer) model — a real mismatch between hit-testing/box placement and what's actually drawn.

**Owner ruling, 2026-09-23: unify the models under the most recent one.** Checked at the symbol before recording this: `map_overlay.gd`'s own font-size model is the more recent of the two (2026-09-01 vs. 2026-08-18), so unification means bringing `label_box_at`/`label_handles` onto `map_overlay.gd`'s own model — not switching label drawing to call the engine's `label_glyph_layout` as originally proposed (that would have reverted to the OLDER model). Whether the engine's `label_glyph_layout`/`arc_label_layout` should also be updated to match (so the two stay unified going forward rather than just patched once) is an implementation call for whoever builds this, not decided here. Not yet built.

## 2026-09-23 — Ruling AK: the general tool row (Inspect/Measure/Region-select/domain tools) moves to a new always-visible top bar, overriding the current design canvas

**The finding, disclosed honestly rather than hidden**: the owner sent three screenshots of the running app (WORLD, CIVIL, CARTO) showing the "TOOLS block" — Inspect, Measure, Region select, Pan (a disabled legend), plus per-domain tools (WORLD: Biome paint, Sculpt-mode only; CIVIL: Settlement/Territory/Way/Route; CARTO: Icon/Label) — as a horizontal `HFlowContainer` row inside each domain's own left-dock panel, and asked for it to move into "a horizontal toolbar that sits on top, following the dcc design from claude design." **A dispatched build checked the actual current canvas before building anything, and it does not agree**: `design/dcc-environment-2026-08-31/Cartalith DCC Environment.dc.html` (`ENV:316-327`, `ENV:1828-1829`; confirmed against `spec/04-left-dock.md` §2.4 too) draws these tools as text pills with key-letter hints, pinned in the **left dock**, not a top bar. The only top bar any canvas draws is the existing Sculpt/Paint/Measure toolbar (`Cartalith Paint Toolbar.dc.html`, already built as `tool_bar.gd`'s `DccToolBar`), and it appears only while one of those three specific tools is armed — it was never meant to carry general tools like Inspect/Measure/Region-select.

**Owner ruling, 2026-09-23: build it as instructed anyway** — a direct owner decision outranks a design canvas, this project's own standing rule (`DCC_SHELL_SPEC.md`'s own precedent, cited elsewhere in this file). Three follow-up questions the build surfaced, all confirmed the same day: **keep the new bar as built** (icons, not the canvas's text-pill style; always visible, not conditional on an armed tool the way the canvas's own top row is). The phone is deliberately unchanged (no top bar in the phone spec, `06-phone.md` §6.3) — not raised as a question, disclosed here since it's a real, if minor, platform divergence from this ruling's own desktop/tablet scope.

**This is a real, disclosed departure from the current canvas**, recorded per this project's own rule that an owner decision does not silently override a design document — the canvas itself should be corrected or annotated to match, the same way other owner-overrides-canvas moments in this file are recorded, so a future pass doesn't get re-confused by the mismatch.

## 2026-09-23 — Ruling AL: a generated landmark draws its own per-kind glyph inside its class ring, and can be clicked to inspect

**The finding is the owner's report, verbatim:** *"currently all landmarks are rendered with the same circle and not the associated icon from the generative settings. Nor can I click to inspect them on the map."* Both halves were true at the symbol. `map_overlay.gd::_draw_landmark_ring` drew an open ring (radius = class, size within class = importance) and nothing else, although `shell/dcc_icons.gd` already carried a glyph for every one of the 49 engine kinds (`landmark_glyph()` resolves all 49; checked against `cartalith_civ::landmark::kinds()`) and nothing in `map_overlay.gd` called it. And `_gui_input` hit-tested settlements only, so a landmark had no click target at all.

**What is KEPT from ruling 14 (2026-09-06), and why it is still right.** The ring stays. Its radius still encodes CLASS (`LANDMARK_CLASS_RADIUS`) and its size within the class still encodes `importance` (±25%), which were the two readable fields ruling 14 chose the ring over a flat per-kind glyph to keep. The class colours are unchanged (cool neutral for physical classes, the civ accent for cultural). The "one layer, the ring wins over a generated POI glyph at the same cell" de-dup (`_icon_shadowed_by_ring`) is untouched and its three probes pass unchanged (`_iconmerge_probe`, `_ringdedup_probe`, `_vfy_iconmerge_probe`).

**What is ADDED.** Inside each ring, `DccIcons.get_icon(DccIcons.landmark_glyph(kind))`, tinted the ring's own class colour, over a translucent dark plate (`LANDMARK_PLATE`, alpha 0.62 — translucent for ruling 14's other reason: a landmark annotates the terrain, it does not sit on top of it). A 16-unit glyph is illegible inside a 4.5 px ring, so every class radius is multiplied by `LANDMARK_MARK_SCALE` = 2.0, which preserves the class RATIOS (continental is still 2x local; the largest local is still smaller than the smallest continental). The glyph is rasterised at the smallest bucket (`LANDMARK_GLYPH_RASTERS`) at or above its on-screen size, read from `get_screen_transform()`, so it stays crisp under camera zoom without caching one texture per continuous size. **A kind with no glyph** (none exists today; the engine's type list is data and can grow ahead of the glyph table) draws the pre-AL ring alone — never a wrong or blank icon.

**One ordering change, forced by a measurement.** Landmarks now draw BEFORE hand-placed icons, not after: with a plate and a glyph, a ring drawn last covered an authored icon on the same cell (`_vfy_iconmerge_probe` measured the authored POI at 0 pixels; it was 32 before AL and is 45 after the reorder). Authored content wins the pixel; the ring still shows around it.

**Click to inspect.** `map_overlay.gd` gained `landmark_hovered`/`landmark_selected`, fired from `_gui_input` exactly where the settlement pair fires (mouse and touch-hold paths both), re-emitted by `viewport_host.gd` and fanned out by `app.gd::_wire_selection()` to any workspace with `on_landmark_selected`. The right dock has a new `CTX_LANDMARK` context showing every field `bridge.landmarks()` returns — kind (the engine's own label), class, elevation (m), importance and suitability (%), cell, and the causal chain as a numbered list, which is the generation's own reasoning and so what "inspect" means here. Hovering draws the same on-canvas card a settlement hover does. The context is dropped on regenerate, world load and a landmark re-run, since the row is a snapshot of a replaced list.

**Settlement vs landmark under one pointer: the NEARER CENTRE wins, an exact tie goes to the settlement** (`_pick_mark`). Not "settlement always wins" — a town inside a ~20 px ring would make the landmark unclickable from most of its own mark; not "landmark always wins" — a small pin inside a big ring would lose its clicks, a regression. Right-click is unchanged (settlement hit only).

**Disclosed cost:** the marks are twice their pre-AL size, so a dense world reads busier at fit zoom (321 landmarks on the probe's pinned 800 km world). `LANDMARK_MARK_SCALE` is the single knob if the owner wants them smaller; below ~1.5 the local-class glyphs stop being legible. The selected landmark is not highlighted on the map (a settlement pin is not either).

**Verified** by `cartalith-native/godot-project/_lmglyph_probe.gd` (windowed): synthetic pixel checks (different kinds differ, a no-glyph kind puts no ink inside its ring) and a live shell on a pinned world with a real `landmark_run()` — a click on a landmark puts every field of `bridge.landmarks()[i]` in the dock, and settlement / empty-map clicks behave as before. GDScript only; no Rust touched.

## 2026-09-23 — Ruling AM: a settlement's authored history events are a `chronos` block in its own vault note, in the Chronos Timeline plugin's exact syntax

**The question.** `STORY_PLANNING_SCOPE.md` SP-3 wants authored events interleaved by date with a settlement's simulated history, and §3 already ruled they live in the Vault rather than in a new store. But `cartalith-vault::block` holds one opaque free-text body per entity with no dated-entry schema, so *some* markup for "a dated entry" had to be chosen before a typed accessor could exist (the SP-3 scoping pass, `OUTSTANDING_WORK.md` §2.3).

**Owner ruling, 2026-09-23:** authored events go in the settlement's existing vault note as a fenced ` ```chronos ` code block, following the **Chronos Timeline** Obsidian plugin's syntax **exactly** — `- [Date~Date] #Color {Group Name} Event Name | Description`, with `@` periods, `*` points and `=` markers, dates `YYYY[-MM-DD[Thh:mm:ss]]`, negative years valid. **The reason is interoperability, and it is deliberate:** the same note opened in real Obsidian with that plugin installed draws a working interactive timeline with no Cartalith involvement. A private variant of the syntax would forfeit that for nothing, so none is accepted or invented.

**Two consequences, both built into the reader (`cartalith_vault::chronos`):**

- **Where in the note: anywhere, not inside the machine block.** §3's original wording ("inside the machine block") is superseded: the `CARTALITH:BEGIN/END` block is Cartalith-owned and replaced wholesale on every block write (`block::upsert`), so an author's event list inside it would be overwritten by the next export. The block is user content and lives outside it, like any other prose. `STORY_PLANNING_SCOPE.md` §3 corrected to match.
- **Year precision only.** This port's clock is the Timeline's signed `i64` year and `STORY_PLANNING_SCOPE.md` §5 forbids a finer parallel clock, so `[1879-03-14]` is read as 1879; month/day/time are validated loosely and dropped. The note keeps them, so Obsidian still sees them. *Superseded 2026-09-23 by Ruling AO's addendum:* month and day are now kept (`chronos::Event`'s `start_md`/`end_md: Option<MonthDay>`, a fixed 365-day calendar, `MONTH_DAYS`); only time of day is still dropped.

**Read-only, and nothing fails.** Cartalith reads the block and never writes one; the author edits the note in Obsidian or in the Vault window's existing editor. A line that is not valid Chronos is reported (count plus the line text) and skipped, never fatal — one typo must not blank a settlement's history. `> ORDERBY`/`> DEFAULTVIEW` flags and `#` comments are view directives, not events, and are passed over silently.

**Addendum, 2026-09-23 — the write path (owner: *"even when using input fields it should add the syntax accordingly"*).** The "Read-only" paragraph above is superseded for *adding* events; existing lines are still edited in the note itself. The Settlement Editor's Political history tab now has an **Add event** form (year, optional end year, name, optional description, colour and group) that composes the Chronos line and writes it into the note, so the author never types the syntax:

- `cartalith_vault::chronos::to_line` is `parse_line`'s inverse, refusing any value that would not read back identically (`every_optional_combination_round_trips_through_the_real_parser`, all 16 optional-part combinations, plus the owner's cheatsheet lines as literal text). `chronos::append_line` puts the line at the end of the note's last `chronos` block, or appends a new block at the end of the note — **never** into a block inside the machine block. Appended, not sorted: Chronos does not assume source order (`> ORDERBY`) and the reader sorts by year itself.
- `VaultSession::add_chronos_event` writes the free-text body under the same `expect_hash` / `Error::SourceChanged` guard as `write_file` — **not** `write_block` — and re-syncs any link that was in step with the note, so the author's own add does not show as Stale.
- `#[func] vault_add_chronos_event(rel, expect_hash, event)` returns `{ok, error, hash, conflict}`; `place_editor_window.gd::_build_add_event_form` / `_submit_event` draw the form and tell the author when the note changed elsewhere (nothing written; the list is re-read so the retry is against what they now see).
- Only the plain `-` event kind is authored from the UI (SP-3's need). The serializer writes all four kinds because the grammar is shared; the UI does not offer periods, points or markers.
- Evidence: the unit tests in `chronos::tests` and `tests::an_authored_event_is_written_as_chronos_and_a_concurrent_edit_refuses_it`; windowed probe `_pechronos_add_probe.gd`, which types into the real fields and presses the real button.

## 2026-09-23 — Ruling AN: build toward HDR/wide-gamut output — the three finishing stages become one pass with one quantisation

**The question** (`OUTSTANDING_WORK.md` §20, carried in §3.1 as blocked): is §20's high-precision display pipeline about fixing a measured precision defect, or about building toward HDR/wide-gamut output? `TERRAIN_APPEARANCE_SCOPE.md` milestones 5 and 6 had rejected it on the first reading, because no defect was measured: clipping *fell* from 0.78% to 0.68%.

**Owner ruling, 2026-09-23: build toward the HDR/wide-gamut future, as the staged version the row itself scoped.** That means removing the `u8` round trips *between* the three whole-raster correction stages (`apply_local_contrast`, `apply_color_grade`, `apply_color_space`) and quantising once at the end. HDR output itself is **not** part of this ruling: Godot's texture stays `RGB8`, PNG/BigTIFF stay 8-bit, and no export default moves. A 16-bit or float encoder is separate future work.

**What was built, and one deviation from the row's wording.** The row said to widen "the intermediate buffer" to `f32`. What shipped is `render::finish_rgb`/`finish_raster`: all three stages **fused per pixel in `f64`**, with one quantiser. Once local contrast's two blurs exist, each stage is a function of its own pixel, so a buffer buys nothing a register doesn't. It would also cost 12 B/px on top of `export_raster.rs`'s *measured* 23 B/px `PEAK_BYTES_PER_PIXEL` gate, and that measurement is what keeps a 32K export from aborting the process. Local contrast is now split into a measurement (`local_contrast_rows` → `LocalContrast::delta`) and an application (inside the fused pass). The single quantiser is the seam a future higher-precision encoder replaces; `render::finish_pixel_continuous` exposes the unquantised value.

**Kept deliberately:**
- Each stage still clamps to `0..=255`. That is range, not precision, and dropping it is the HDR step itself.
- The quantiser is the one the old last stage used: truncation, except where the colour space re-encoded the pixel, which rounds (`apply_color_space`'s own documented reason).
- The three `apply_*` functions remain as the fused pass with two stages switched off. They are byte-identical to the passes they replaced; the old-chain hashes were reproduced exactly on `43a2f76`.
- Exports stay in the working space (sRGB). `export_raster.rs`'s 2026-09-06 decision is untouched: `finish_raster` is called with `ColorSpace::Srgb` there, and `_exportraster_probe.gd` §15 still passes byte-identical.

**The four call sites:**
- Screen: `lib.rs::build_color_texture`.
- Layer and whole-raster PNG export: `export_raster.rs`, `export_raster_png` and `export_layers`.
- Banded E1/E2 export: `render::bake_export_band`. It now corrects only the band's own rows and no longer spends work on apron rows it discards.
- LOD tiles: `render::render_biome_tile_rgba`. Its inline per-pixel local contrast became the `delta` closure of the same fused pass.

Two stay partially chained:
- `export_snapshot_png` runs only the grade, so there was nothing to fuse; that is unchanged.
- With an asset pack loaded, the screen quantises once before the icon composite (which draws in bytes) and applies the colour space after it.

**The re-baseline, measured:**
- **At the shipped default there is none.** The grade is at rest and the space is sRGB, so only one stage runs and there is no intermediate quantisation to remove. `tests/color_space.rs`'s `FINISHED_RENDER_FNV1A` = `0x6154_1058_49e7_10d6` did **not** move. That is the control.
- **Where two or three stages run, pixels move up by one to three levels and never down.** Each difference is a fraction the old intermediate truncation threw away.
- **Synthetic fixture** (128×79), fused vs the old chain:

  | Case | Pixels differing | Worst move |
  |---|---|---|
  | Default + Display P3 | 5 012 / 10 112 | 1 |
  | Antique sRGB | 5 139 | 2 |
  | Antique P3 | 9 318 | 2 |
  | Strong grade | 5 276 | 2 |

- **Mean |error| against the continuous value**, fused vs old: 0.275 vs 0.478, 0.478 vs 0.828, 0.254 vs 0.838 and 0.454 vs 0.832. So the new output is **42–70% closer to the true colour**, not merely different.
- **New pin:** `ANTIQUE_P3_FNV1A` = `0xf96d_1e67_6c25_daae`, previously `0x6c83_b198_b39e_4d68`.
- **One more digest re-derived:** `tests/layer_stack.rs`'s Antique screen digest, `0xae24_83aa_9cb4_63bf` → `0x4427_8798_5bad_05ae`. It moved for the same reason: it is the only look there with a grade that is not at rest. The default, Vibrant and `js_reference` screen digests and all four bake digests are unchanged.
- **Real 512² world, seed 12345:**

  | Case | Pixels moved | Deltas |
  |---|---|---|
  | Default sRGB | 0% | — |
  | Default P3 | 68.8% | 0..1 |
  | Antique sRGB | 70.9% | 0..2 |
  | Antique P3 | 97.3% | 0..3 (3 channels at 3) |

- **Live shell (`_finishpass_probe.gd`, windowed, HEAD build vs this build):**

  | Output | Pixels moved | Deltas |
  |---|---|---|
  | Default screen | 0 | — |
  | Default 2K export | 0 | — |
  | Antique+P3 screen | 95.8% | 0..3 |
  | Antique 2K export | 69.3% | 0..2 |

  Side by side and at 60× difference gain, the pictures are indistinguishable and the difference image has no structure.

**No JS reference exists for this**: the reference renders through 8-bit canvas `getImageData`. The bar was therefore "equivalent or better, measured", not byte identity.

## 2026-09-23 — Ruling AO: SP-2/SP-4's three open questions, ruled

`STORY_PLANNING_SCOPE.md` §6 named three unresolved questions gating SP-2 and SP-4. The owner ruled all three directly, in response to being asked exactly what was still needed:

**Grain (SP-2): a real calendar date, not a fraction of a year — and it reuses Chronos's own date, not a second system.** The reference has no defined calendar object, but it consistently treats a year as 365 days with no month structure — `restCadence`/layover fields cap at 365, the food-demand formula multiplies by `*365*2` (`Cartalith Gen1 v2.11.html:6622`), and the Journey Planner already computes trip length in real days (`jpFmtDays`, `totalDays`) walking a season cycle by day-offset (`jpSeasonAt(startSeason, dayOffset)`, `:19313`). **Corrected same day**: Ruling AM already committed this port to writing dates in the Chronos plugin's own `YYYY-MM-DDThh:mm:ss` syntax for authored events, and Chronos's format carries a real year/month/day (`chronos::Event.start`/`.end` are years only today — §5's "no finer clock" rule for THAT feature — but the on-disk syntax and `chronos::parse`'s own grammar already accept month and day; only the reader currently discards them). SP-2 needs day-level positioning, so it should use that same year/month/day rather than inventing a parallel `(year, day_of_year)` representation the rest of the app doesn't share. **The calendar itself still needs a real, disclosed choice**: the reference's own 365-day-year assumption argues for a plain 12-month calendar with no leap years (month lengths summing to 365 — the simplest scheme that stays compatible with Chronos's `MM`/`DD` ranges and this port's existing day-count math), but no specific month-length table exists anywhere in this project yet. Whoever builds SP-2 picks one, states it plainly, and should extend `chronos::Event`/`chronos::parse`'s month/day handling (currently dropped) rather than duplicate date math elsewhere.

**Regenerate semantics (SP-2): re-snap onto the new terrain.** A journey's route polyline is recomputed against the regenerated world rather than kept as-is with a staleness mark or invalidated outright. This is real, disclosed scope beyond the minimum: a re-snap needs to handle a stop point that no longer exists (a settlement that moved or was removed) and a route that is no longer geometrically valid — whoever builds this should name the edge-case behaviour explicitly rather than assume the happy path.

**Attachment model (SP-4): a conflict can reference a settlement or province.** Not free-geometry-only — a drawn conflict (front, siege, battle marker) can optionally link to a real settlement/province, so moving the place moves the conflict and "what conflicts happened here" becomes a real query. This is what unblocks Landmark M9's cultural-interpretation work downstream. Real added scope over the free-geometry-only option: the attachment needs a real reference (by `tid` or similar stable id, not by name or position), and behaviour on the referenced entity's deletion/regenerate needs deciding by whoever builds it.

Not yet built. See `OUTSTANDING_WORK.md`'s SP-2/SP-4 rows for status.

*Stale, noted 2026-09-24:* the two addenda below record SP-2 and SP-4 as built. Status is in `cartalith-native/docs/STATUS.md`, not here.

**Addendum, 2026-09-23 — SP-2 built; the three points this ruling left to the builder, as decided.** Implementing this ruling's own open points, not a new owner decision.

- **The calendar: `cartalith_vault::chronos::MONTH_DAYS` = `[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]`**, `DAYS_PER_YEAR` = 365. The Gregorian lengths with February fixed at 28 — no leap year, ever. It was chosen over an invented scheme because it keeps Chronos's own `MM`/`DD` meaning what an Obsidian user expects for 365 of 366 days. The owner's EBSS vault was checked first: it writes bare years and one `YYYY-MM` (`[4349-09]`, `Timeline Summary.md`) and names no months or month lengths, so there was no in-world calendar to match. The only date Chronos can write that this calendar cannot hold is `-02-29`. It reads as its bare year, the way every month/day read before this change. It is not refused.
- **Where the date lives.** `chronos::Event` gains `start_md`/`end_md: Option<MonthDay>`. They are absent unless the source wrote a month this calendar has. With both absent, `to_line` writes byte-for-byte what it wrote before; the 16-combination round-trip test is unchanged and still passes. The calendar maths is `MonthDay::{new, day_of_year, from_day_of_year}`, `day_number` and `date_of_day_number`, all beside the parser, so there is one date representation. The cursor's day is `WorldGen::civ_day` (0..364), beside `CivData::year` and not a second clock: `civ_goto_year` still moves only the year, and only journeys read the day. A saved journey departs on **1 January of `Journey::start_year`**, because SP-1's persisted entity carries a year and nothing finer. A full departure date is a format change left for later.
- **Rest days are spread evenly** over the journey (`journey_progress::JourneyTimeline::at`), not taken as a whole stop every Nth day. Arrival is derived as departure + `ceil(total_days)`, where `total_days` is `jp_plan`'s own figure (travel + rest + layovers).
- **Re-snap policy** (`journey_progress::resnap_journey`, `story_bridge.rs`). The route snapshot keeps only its two endpoints as stops; `_jpRerouteForMode` re-paths the same two. Each endpoint is bound to the nearest old settlement within the planner's own stop radius (`jp_stop_radius_cells`), matched by **`tid` and name together**, because a fresh generate re-issues `tid`s from 1.
  - **(a) The stop no longer exists** (no new settlement has that `tid` and name): **the journey is dropped** and reported by name in `journey_resnap_report()` and the log. A route to a place that no longer exists would still look planned.
  - **(b) The route is no longer valid** (an unreachable leg under the journey's mode): **kept and flagged**, with the straight-line fallback and `unreachable_legs`. This matches how `route_commit` treats a freshly drawn route. The planner then judges it; usually that is a blocked stage, which draws no marker.
  - An endpoint that was not a settlement keeps its cell, scaled by any grid-size change.

### Addendum, 2026-09-23 — SP-4's open points, decided by the builder

The ruling left the attachment's deletion/regenerate behaviour to whoever built SP-4. These are the calls made (`cartalith_civ::conflict`, `cartalith-godot/src/conflict_bridge.rs`), each stated so it can be overruled:

- **A province is referenced by its seed settlement's `tid`, not by `Province::id`.** `civ_generate_provinces` numbers provinces positionally (faction order, then settlement order) on every run, so an id survives a recompute only by coincidence; the seed's `tid` survives it by construction. A province whose seed stops being a seed stops resolving. (`ConflictAnchor::Province(u64)`.) Provinces are a live entity — `get_provinces()`, `civ.province_list`, the vault's province links — so both halves of the ruling's "settlement or province" are built.
- **Moving the place moves the conflict by translation.** The shape is stored as drawn plus the anchor's position when attached (`Conflict::anchor_at`); it is drawn translated by however far the anchor has since moved. Attaching and detaching never move the drawing. No settlement-move operation exists in this port today, so this is exercised by unit test only.
- **The referenced place is deleted** (`civ_delete_settlement`, or a province seed lost to a recompute): **the anchor is kept, not cleared**; the shape draws where it was attached, at half alpha, and the right dock says "missing". If the `tid` resolves again, it re-attaches. Clearing it would lose the author's link silently; re-binding it would be worse.
- **Auto-populate / Clear places** (`civ_populate`, `civ_clear_places`) reissue every `tid` on the same grid: conflicts are **kept, every anchor detached first** (shape baked where drawn), because a kept `tid` would re-bind to an unrelated new town.
- **A new world** (generate, import, recentre) **or opening a project** empties the store, as it empties knowledge links and landmarks — `tid`s restart and the grid may differ. This is deliberately *not* SP-2's re-snap: a conflict has no route to re-solve, and there is no stable cross-world identity to re-snap onto.
- **Outcome is free text**, not an enum: a closed outcome vocabulary is the first step of the resolution model `STORY_PLANNING_SCOPE.md` §5 rules out.
- **Years are plain `i64`**, the Timeline cursor's unit: SP-4's only question of time is "active in the cursor's year", which needs no day grain.
- **Persistence: `entities/conflicts.json`, engine-owned** (`SAVEFILE_COMPAT.md` §9.7), the landmarks slot's arrangement — the store is engine state GDScript has no view of.
- **The manpower read is the world as it stands, not the conflict's year.** `conflict_sides_manpower` returns exactly CIVIL ▸ Military's figures (`manpower_by_faction`). A year-scoped read would need the manpower pass rebuilt over a `TimelineSnapshot` (settlement populations and ways per year); not done, and said so in the dock.

## 2026-09-23 — Ruling AP: fourteen open items ruled in one sitting, from the owner-decision backlog compiled the same day

The backlog compiled earlier the same day (`OUTSTANDING_WORK.md`'s cross-referenced decision clusters) was put to the owner as fourteen `AskUserQuestion` prompts, each with a recommended option. Recorded here rather than left scattered across rows, so a later reader finds the ruling in one place per this file's own convention.

**Which source-engine line this port follows: the DCC line, not the v2.22 mainline.** The reference forked at v2.22; every engine change from v2.25 on exists only on that branch, and `RC_ENGINE_CHANGES.md` already catalogues v2.11→v2.73 against it. This authorizes re-freezing the reference snapshot against the DCC line and resuming porting from that document's own change list — `OUTSTANDING_WORK.md` §2.8's re-freeze row is no longer blocked on "which line," only on the mechanical work of choosing an exact target version and re-generating the frozen snapshot + function index.

**Landmark state persists across save/reload.** Reverses the assumption every landmark row to date has shipped under ("regenerate on load, no save-tree slot"). Real work: a save-format addition (an `entities/landmarks.json`-shaped slot, following the conflicts/journeys precedent), plus deciding what "state" actually means per landmark (a name the player gave it, at minimum — `research §25`'s discovered/named/monumentalized states are the fuller shape, not yet scoped in detail).

**Province-level territory assignment: not now.** The lasso stays faction-only; no work authorized here.

**The 16K/32K single-image export: resume it.** Un-shelves `EXPORT_SCOPE.md`'s batches B–D. Still gated on the two sub-questions the backlog named (five scope questions — content types, label/road/river detail, settlement filters, UI-freeze tolerance; and the codec/size tradeoff, where nothing makes a 32K file small and the one compressed option is blocked by licensing) — **not answered by this ruling**, raise them as their own batch when this is picked up rather than guessing.

**Multi-ridge mountain ranges ("the Himalaya problem"): scheduled.** The spec contradiction blocking it (`RC_ENGINE_CHANGES.md` §6b.2 vs §9) was resolved 2026-09-21; this ruling is the scheduling authorization that row's own text said it still needed. Large (multi-sheet orogenic belt geometry) — a multi-batch feature, not a single dispatch.

**Snow gets an aspect (slope-direction) term.** Reverses LOD-D4's declined golden re-baseline. A real, visible change to the main map's `materialWeights` snow term, not just LOD tiles — every existing snowy mountain's rendered look changes. Scheduled, not yet built.

*Stale, noted 2026-09-24:* built in `19c38d9`, and verified partial. It shipped as `TerrainAppearance::snow_aspect_c` (2.0 °C shipped; 0.0 under `js_reference()`) and `snow_aspect_shift`. The Rust render hashes were re-baselined, and the JS-parity goldens did not move. LOD-D4's bar 1b is still not met. Status is in `STATUS.md`.

**Asset library images: embedded in the save file.** Real save-format work — the library currently persists item definitions with no image payload at all.

**Crater model constants: kept as they are.** No work authorized — this closes the question rather than opening a task. The port's independently-derived constants stay; re-deriving to match the reference exactly was declined.

**IN-13 caravans: the derived view.** One row per way with active trade load, nothing persisted, rebuilt from the live trade match every time — not a saved entity with its own refresh rule. **Not yet resolved by this ruling**: whether a sea trade route counts as a "way" for this purpose or needs its own entity — Ruling AF's own text flagged this as "one more question, not yet asked"; it is asked now and **still open**, raise it before building the sea-lane case. *(Answered 2026-09-24 by Ruling AQ: one caravan per sea lane. The land half was built as this derived view in `da51a57`.)*

**"Fixed size" map labels: fix them to actually stay fixed.** Confirms the already-flagged bug is a real defect, not intended behavior.

**Landmarks become vault-linkable.** A new `cartalith_vault::EntityKind` variant, consistent with settlements and provinces. Identity is expected to be as weak as those two (derived, not stable across a regenerate) — that weakness is accepted, not a blocker.

**The parity contract (`DECISIONS.md` §7a/§7d) does not apply to landmarks.** All landmark work is confirmed divergence-by-addition, tested for internal correctness and determinism rather than against a reference that never existed for this subsystem. Closes the question this file's landmark rulings (AL, and M9's own row) had been implicitly assuming but never formally settled.

**The landmark viewshed budget: the conservative default already shipped stays.** No work authorized — M7's existing default is ratified as-is rather than replaced.

**Landmark code stays in `cartalith-civ`.** No crate split authorized.

*Correction, 2026-09-23, same evening — five of the landmark items above re-asked questions already ruled.* Persistence (ruling 10), vault-linkability (ruling 13), the parity contract (ruling 12), the viewshed budget (ruling 16) and crate placement (ruling 24, since superseded by the owner's ratify-the-split ruling) were all settled on 2026-09-06; today's answers match them, so nothing conflicts. Two things this ruling called new work **already exist**: the `entities/landmarks.json` save slot (`project_bridge.rs`'s `SLOT_LANDMARKS`, holding settings and the last run's placements) and `cartalith_vault::EntityKind::Landmark` (`45630cc`). What genuinely remains is per-landmark authored state (a player-given name first) and a shell entry point for linking a landmark to a note. Cause: the backlog these questions came from was compiled from `LANDMARK_GENERATION_SCOPE.md` §4, which still listed them as open (fixed in `d1b331c`); `MISTAKES.md` now carries the rule — search this file before putting a question to the owner.

**Not asked, disclosed here as builder's calls instead** (genuinely low-stakes, no other work depends on the answer): the ice/snow memory-budget doc ambiguity (`≤10 MiB` — read as the binary/1024²-based convention, the stricter reading) and map-thumbnail hillshade (left flat, as already decided — nobody has asked for it since). GPU memory pooling's VRAM-retention question stays open with its own filed reopen trigger (`OUTSTANDING_WORK.md` §2.6); not urgent enough to ask about today.

## 2026-09-24 — Ruling AQ: four open items ruled, from questions raised by the day's investigations

Asked through the session's question tool; the owner picked each answer. The questions arose from work the same day, each with its evidence recorded in `OUTSTANDING_WORK.md`.

**Sea level becomes a live control.** The reference's Sea level slider moves the live world's sea level and re-derives the coastline, lakes, biomes and the civ layer's water-dependent inputs without regenerating (v2.10 12720, `bind('sea')`). This port applied it only on the next Generate (`set_sea_level` writes `params.sea_level`). **Make it live, as the reference does.** Ruling 17 lands with it: a sculpt draft is re-stamped through `SculptStamp::with_sea_level` as the slider moves. Owned by `OUTSTANDING_WORK.md` §2.5, "Sea level is not a live control".

**Sea trade gets caravans: one per sea lane.** Settles the question Ruling AP left open. Each seaborne trade route between two ports is a lane with its own caravan row, derived from the same live trade match and never saved — the same derived view as the land caravans (`da51a57`). Owned by the IN-13 piece 4 row, §2.3.

**Every tooltip that names developer code is rewritten, not only default panels.** The four visible labels were already fixed (`3a12d64`); the tooltip sweep (~795 at the 2026-09-23 count) covers all of them, with the code names kept as `##` comments beside the string. Owned by §2.10's "Developer code names are shown to users" row.

**SP-3 gets the combined strip.** The owner's 2026-08-25 fork asked for both tracks on one strip; it was built as four separate lists on one tab. Build one strip with ownership, population/tier, authored events and journeys passing interleaved by date, keeping the lists as the detail beneath it. Owned by a new §2.3 row.

## 2026-09-24 — Ruling AR: four items the civilisation alignment audit raised

Asked through the session's question tool after the 2026-09-24 code-versus-documentation audit; the owner picked each answer.

**A reopened project must support every civilisation readout, by saving the missing rasters.** Today `project_open` restores the civ layer but `load_save` marks the world `WorldSource::Loaded`, and journey planning, trade flows, military, town layouts and the faction economy all refuse a Loaded world — while telling the user the save "carries no civilisation layer", which is false for a project. **Extend the project format to store the flow/river/channel rasters those readouts need**, so they work on open without recomputing. A format change: it needs a `SAVEFILE_COMPAT.md` entry, and older projects without the rasters must still open (and say what they cannot do).

**A saved journey keeps its full plan.** `entities/journeys.json` stores the whole plan — all plan fields, stage overrides, layovers, animal choices and trim — not only the 20-field party preset, so a reopened journey plans exactly as it did and SP-2 markers and SP-3 pass dates stop being planned from defaults.

**Per-faction currencies stand (Ruling R is kept).** Ruling AB's world-wide scarcity price index was built instead of R; R is not superseded. Build per-faction currencies with exchange rates on top of AB's scarcity pricing.

**Fix coastal detection's east-west wrap on non-wrapping maps.** `civ_is_coastal` wraps x even when the map is not a world, so an edge settlement can count as coastal because of water on the opposite edge. Wrap only on world maps; the affected goldens are a disclosed re-baseline under this ruling.

## 2026-09-24 — Ruling AS: four items the engine alignment audit and the sculpt cleanup raised

Asked through the session's question tool; the owner picked each answer.

**Port the reference's World Structure orogeny derivation.** With World Structure on, this port fixes `OrogenyParams { fold_k: 0.16, trench_k: 1.0, fault_block_k: 0.0 }`; the reference derives `foldIntensity = 0.6 + tectonicEnergy` and `trenchDepth = 0.7 + 0.8·oceanDepth` and carries `faultBlock 0.6` (v2.10 2265, 2536-2538), so horst-and-graben terrain never forms here. Port the derivation; World-Structure and archetype worlds change, a disclosed re-baseline under this ruling.

**Turn the ocean-grain lattice fix on in the shipped look.** `TerrainAppearance::sea_grain_warp` stays 0.0 on the reference-parity path (`js_reference()` must pin it explicitly first, since it inherits the field through `..default()`) and becomes non-zero in `default()`. Only the Rust render hashes move, disclosed. The framing of the fix as a "divergence" is scrubbed per §7p.

**The phone keeps 4K and 8K in New World, with a memory warning.** Ruling AH's 2048 ceiling stands as the recommendation; the phone offers the higher presets but warns about memory before generating rather than hiding them.

**A sculpt commit clears painted override cells it covers.** Today painted biome/terrain cells survive a sculpt commit (and are dropped on regenerate and load). A sculpt changes the ground, so the paint under a committed stamp is cleared.

## 2026-09-24 — Ruling AT: Go to year, the viewshed refine, and splat under a sculpt

Asked through the session's question tool; the owner picked each answer.

**Go to year: a recorded year's snapshot becomes the new territory base.**
Going to a recorded year makes its snapshot the paint base and clears the paint
layer and any pending stroke, so the next stroke, subtract or recompute edits
that year rather than erasing it. **Going to an unrecorded year leaves
territory, base and paint untouched.** That departs from the reference's
`terr.fill(0)` on a year with no snapshot, deliberately, so scrubbing the
timeline strip can no longer erase unsaved territory. The strip's comments and
its board-D readout ("territory holds at …") then become true and stay.

**Ruling 16's "recompute and refine" covers the current view only**, beside
Preferences ▸ Tiles & LOD's existing *Refine detail for the current view*.
**A refined viewshed result is saved with the project** in
`entities/landmarks.json`; a regenerate or sculpt that changes the terrain
marks it stale and falls back to the coarse result.

**A sculpt commit clears splat (ground-texture) paint under its stroke as well
as biome and terrain paint**, as shipped in `5639e3d`: a sculpt changes the
ground.

## 2026-09-24 — Ruling AU: tile shading, glaciation, currencies, legacy archives

Asked through the session's question tool; the owner picked each answer.

**Deep-zoom tiles keep the reference tile's shading** (`LOD_DETAIL_SCOPE.md`
owner question 1). The shipped app does not switch tiles to match this port's
screen; the gap at LOD entry stays as measured, and no change is made.

**New worlds turn the glacial erosion pass on** (`LOD_DETAIL_SCOPE.md` owner
question 3). It is an app-default divergence: `cartalith_godot::params::defaults()`
enables it, `WorldParams::defaults()` and every golden stay as they are. Cold
mountains gain carved troughs and ice by default; generation takes longer, and
the cost is to be measured and disclosed.

**Per-faction exchange rates are user-set** (Ruling R, kept by AR). Each faction
carries its own currency; the user types its rate in the faction roster and the
engine only converts. No rate is derived from the economy.

**Legacy flat `.zip` archives import their settlements, labels and icons**,
mapped onto today's settlements, annotations and icons, with a report of
anything that did not map.

## 2026-09-24 — Ruling AV: Peak viewshed, the Android vault picker, Military wording, deep-zoom info views

Asked through the session's question tool; the owner picked each answer.

**Peak landmarks score the land they overlook.** `pool_peak` gains a
visible-land term from the existing coarse viewshed, so `peak` joins the kinds
whose `needs_viewshed` is true. Landmarks carry no parity contract
(`DECISIONS.md` §7q); the standing bar applies.

**The Android vault folder is picked through a small Android plugin**: a `.aar`
that opens the system folder picker (`ACTION_OPEN_DOCUMENT_TREE`), takes a
persisted grant, and hands the tree URI to `vault_connect_saf`.

**CIVIL ▸ Military says the campaign items were declined, in one line**: war
campaigns over time were considered and declined; the conflict overlay shows
each side's manpower. No "needs a decision".

**Build LOD-D7: info and debug views get deep-zoom tiles** like the terrain
view (`LOD_DETAIL_SCOPE.md` owner question 6).

## 2026-09-24 — Ruling AW: war campaigns over time are not declined

**Supersedes Ruling AV's Military wording and `MILITARY_MANPOWER_SCOPE.md` §4's
decline of campaigns, change over time, and per-settlement garrisons.** The
owner, in their own words: war campaigns over time "should not be declined".
Asked what to build, the owner answered: **"Territory over time and siege lines
drawn in their own Conflict layer under Carto."** Asked about garrisons: **yes,
derived.**

- **Territory over time.** A war's effect is shown as territory changing across
  the Timeline's year cursor: the claims a conflict moves between recorded
  years, read through the existing timeline snapshots (Ruling AT's base model).
- **Siege lines in their own Conflict layer under CARTO.** A new map layer,
  listed with the other CARTO layers, draws conflicts: siege lines around a
  besieged settlement and the fronts between the sides, following the year
  cursor. It is its own layer, not part of the SP-4 conflict overlay in CIVIL.
- **Per-settlement garrisons, derived.** Each faction's standing army is split
  across its settlements by a stated rule (walls, capital, border exposure),
  shown per settlement and usable as a conflict's starting forces.

What the owner did not ask for is not implied: no unit-by-unit movement and no
combat rule that resolves battles on its own. Settle any such step with the
owner before building it.

## 2026-09-26 — Ruling AX: the map-context forks (`MAP_CONTEXT_SCOPE.md` §10)

Asked through the session's question tool the day the owner scheduled the map
context work; the owner picked the recommended answer to each.

**F1 — phone long-press: drop the sample pin *and* open the verbs, at 480 ms.**
This settles the collision between `06-phone.md` §7 (pin, 480 ms) and the
shipped `map_overlay.gd` (`_TOUCH_HOLD_MS := 500`, menu). CM-5 is unblocked.

**F2 — build the phone's thumb fan, after CM-5.** CM-6 is unblocked, ordered
after CM-5.

**F3 — the ring's cardinals are global: Inspect / Measure / Undo / Region**,
with each domain's four tool families on the diagonals (§5.1).

**F4 — a settlement right-clicked in CARTO shows one "…in CIVIL ›" row**, not
CIVIL's verbs inline.

F5 (ring customisation and the tablet edge sliders) was not asked; the scope's
recommendation, *later*, stands until the owner says otherwise.

## 2026-09-28 — Ruling AY: migrate old conflict anchors on load

Asked through the session's question tool after `36312e2` moved conflict
anchors from the cell corner to the cell centre. The owner chose the
recommendation: **when a project saved before that fix is opened, shift each
stored conflict anchor (`ConflictAnchorDto::at`) by half a cell on both axes**,
so old conflicts draw exactly where they did, now centred on the pin. Keyed on
the save's format (an additive marker the new writer sets), tested from a real
pre-fix archive.

## 2026-09-28 — Ruling AZ: the owner-blocked rows, answered in one sitting

Asked through the session's question tool, in four rounds, to clear every row
the backlog tiering had marked as waiting on the owner.

**Garrisons.** The border-exposure weight becomes a **user setting**, not a fixed
value (the owner's own answer, replacing the scale of 1). A faction with no
settlement marked Capital gets **no capital weight anywhere** — the rule matches
what the dock shows.

**Exit buttons.** Apply the proposed rule everywhere: **Close** for windows that
only show or apply changes live; **OK / Cancel** only where changes are held
until confirmed.

**LOD tile sawtooth.** Not accepted as within tolerance: **keep fixing** the last
configuration (1.67 against the bar).

**Invisible label fix.** The owner had not (or cannot confirm they had) run
CARTO ▸ Generate labels, so the fix was likely working as designed: close the
defect, and make the need to generate labels clearer on screen.

**Mouse clipped after setting a siege status.** Close as not reproducible; the
owner will re-report with exact steps if it recurs.

**Screens with no design** (phone error and storage-full states, the Data
manager in the Environment prototype, phone sheets covering the tab bar, a
touch hint for map drag, two export panes too wide for a phone): **derive them
from the DCC canvases' own vocabulary**, per the standing rule, build them, and
show the owner screenshots to approve.

**Village map style.** Softer: **more colour bands**, matching the other presets.

**Performance and platform.** Build now: **phone memory reduction** (the 878 MB
peak for a default world, R1), **a smaller Android APK** (strip the remaining
debug symbols), and **a GPU memory pool** kept between generations. Performance
telemetry stays parked.

**16K/32K export (E4 and the codec).**
- Overlays in v1: **settlements, labels, roads and ways, icons and borders**.
- Settlements: **the user picks a minimum tier** (default towns).
- Rivers: **the vector line**, not the baked ink.
- Town plans: **drawn above a size** readable at the export's resolution.
- **No UI freezing** — the per-band 1-2 s freeze is not acceptable for v1;
  render off the main thread with progress.
- Codec: **offer JPEG alongside PNG** (amends ruling 26, which set PNG only);
  the dialog still shows the expected size.

**Research tracks to start:** the **3D viewport** (its three commissioned
questions), **importance-driven refinement (EF-9)**, and **GPU stream-power
erosion** (needs a scope first).

**The v2.73 footpath class:** the owner will supply the v2.73 (or newer DCC)
source file; the re-freeze and port wait on that file arriving. *Same day, the owner added: "Version 2.71 seems to be latest, 2.73 doesn't seem to exist." So the v2.73 footpath row, and the v2.72/v2.73 entries in `RC_ENGINE_CHANGES.md`, are unverified until the source folder is checked.*
**Resolved by Ruling BQ, 2026-09-28, later the same day: confirmed — v2.72 and
v2.73 do not exist, v2.71 is the latest. The footpath row is closed as having
no source, not left waiting on a file that will not arrive.**

## 2026-09-28 — Ruling BA: CM-7's two engine rows

Asked through the session's question tool when CM-7 stopped at them; the owner
chose the recommendation for both.

**Deleting a way from the context card is undoable**: it records an undo step
like other edits, so Edit ▸ Undo restores the way.

**A picked river is "the branch to its mouth"**: the stretch clicked plus
everything downstream to where it meets the sea or a lake. That line is what
the card names and what *Trace downstream* and *Show catchment* act on.

## 2026-09-29 — Ruling BB: labels do not regenerate automatically after a sculpt

Asked through the session's question tool after `09858bd` added the CARTO ▸
Labels cue. The owner chose **"No, keep the cue"**: after a sculpt commit the
generated labels stay where they were until the user presses Generate labels,
and the Labels panel's "The terrain has changed since labels were last
generated" cue is the signal. `engine_bridge.gd`'s note that the re-run is "a UI
decision, not a binding decision" is now answered.

## 2026-09-29 — Ruling BC: the tool ring matches the mockup's style and shape, in the theme's colours

Owner, after seeing the tablet ring screenshots: *"The ring BTW doesn't look like
1-to-1 the mockup"*, then: *"It's the style and shape, that the ring follows the
color scheme of the set theme is correct."*

So `design/map-context-2026-09-25/` (`Main`, `Tablet`, `Phone`) is the reference
for the ring's **style and shape**, 1-to-1:
- the scrim behind an open ring and the disc behind the slots;
- slot size, with the label inside the slot;
- the hover wedge toward the hovered direction;
- the 46 px centre button, and the accent `‹ BACK` with its glow at a sub-ring's
  centre;
- the caption pill;
- the sub-ring layout;
- the phone thumb fan's equivalents.

Its **colours do not carry over**. The mockup's dark hex values are one theme's
rendering; the shipped ring takes the active theme's tokens (`DccTheme`), and
that is correct as built. Where the mockup's contrast refinement (its README,
"What the 2026-09-25 refinement changed") sets a contrast relationship, such as
disc opacity or disabled-label contrast, keep the relationship in the theme's
tokens.

## 2026-09-29 — Ruling BD: the river carve is fixed at its source, and the valley is shaded along the smooth line

Asked through the session's question tool after `_riverzoom_probe` measured the
causes (commit `19d3ba8`).

**RV-1: fix it, and re-baseline.** The carve (`enforce_channel_descent`) must
stop leaving junction pits that classify as lakes. Measured: 11-18% of traced
river cells are lake-class, and about half of those "lakes" are 1-3 cells. The
generated rivers and lakes change in every world. The golden values are
re-recorded, and each old → new is disclosed in the same commit.

**RV-3: re-cut the valley on the line.** Keep the carve for hydrology, but shade
the terrain from a smooth valley cut along the vector river. Today the stepped
carve shows as dark grooves, and about 30% of them have no river drawn on
them. How terrain looks near rivers changes; the render and tile golden
values are re-recorded and disclosed.

Both follow DECISIONS §7p: the port's improved behaviour wins over matching the
legacy HTML.

## 2026-09-27 — Ruling BE: one vault window, with a real Markdown editor

Owner, after the vault showcase screenshots: *"can we consolidate screenshot 2
and 3 into a single window and give the editor basic word editor functions,
bold, understriped, text size, header, like a functional markdown editor."*

**One window.** Screenshot 2 was browse and edit (`open_browse`). Screenshot 3
was the entity-scoped view (`open_for`). They become one layout:
- The tree and search on the left. Attached notes carry a link marker.
- On the right, an **Attached to** block listing each link: entity name, kind
  and faction, plus Show on map, Open place editor, Attach and Detach. A
  frontmatter `tid` that resolves to a real entity is shown as "by frontmatter,
  not attached", with an Attach action.
- Below it, the note pane. It is a read-only preview until **Edit** switches
  that same pane to the editor. There is no separate form underneath.
- Opening from an entity preselects that entity's notes.
- Screenshot 3's advanced actions move into a collapsed **More** section: map
  snapshot, Cartalith block, field fill, and write confirmations.

**The editor** is a functional Markdown editor:
- A toolbar with Bold, Italic, Underline, Strikethrough, a heading/size picker
  (Normal, H1, H2, H3), bullet and numbered lists, quote, code, and a
  note link (`[[…]]`).
- Ctrl+B, Ctrl+I and Ctrl+U shortcuts. A button wraps the selection, or inserts
  the markup at the cursor.
- A Write / Preview toggle.
- Themed; it fills the pane.
- Save keeps the existing hash guard, which refuses when the file changed on
  disk.
- On the phone, the toolbar is a scrollable row above the keyboard.

**Defaults taken, stated to the owner:**
- Markdown has no underline, so Underline writes `<u>…</u>`, which Obsidian
  renders.
- Markdown stores no arbitrary font size, so "text size" is the heading level.

Build it from the DCC vocabulary and the approved Vault Browser mockup
(`design/vault-browser-2026-09-21/`); screenshots go to the owner for approval.

## 2026-09-27 — Ruling BF: vault templates and Markdown follow Obsidian exactly

Owner: *"I do have a templates folder in my obsidian vault. Maybe we should
implement the same structure as obsidian to use templates. (same for how
bold/links/etc work in markdown. I'd like to keep that syntax aligned. Same as
for example with the chronos timeline plugging and formatting we did before."*

This is the same rule as Ruling AM (Chronos): **Obsidian's own syntax and
structure, exactly, with no Cartalith variant.** A note written or created in
Cartalith must read identically in Obsidian, and the reverse.

**Templates** (Obsidian core Templates plugin):
- **The template folder** is read from `.obsidian/templates.json` (`folder`),
  then from the Templater plugin's `templates_folder` if that plugin is
  present. `template::discover`'s "the path contains *template*" rule is
  kept only as the fallback when neither setting exists.
- **Placeholders:** `{{title}}`, `{{date}}`, `{{time}}`, `{{date:FORMAT}}` and
  `{{time:FORMAT}}` are filled.
  - Formats use Moment.js tokens; the defaults come from `templates.json`'s
    `dateFormat`/`timeFormat`, else Obsidian's own defaults (`YYYY-MM-DD`,
    `HH:mm`).
  - `{{date}}` means **today's real date**, as in Obsidian.
  - The existing `{{…_Name}}` tokens (`template::fill_title`) keep working.
  - This reverses `fill_title`'s deliberate "leave `{{title}}` and `{{date}}`
    alone" — Obsidian fills them.
- **Templater's `<% … %>`** is JavaScript only Templater can run. It is left
  verbatim, never executed or imitated.
- **The editor** gains **Insert template** at the caret, as Obsidian's command
  does.

**Markdown** — the editor's toolbar and Preview match Obsidian's syntax:
- `**bold**`, `*italic*`, `~~strike~~`, `==highlight==`;
- `[[note]]`, `[[note|alias]]`, `[[note#heading]]`, `![[embed]]`;
- `#tag`, `- [ ]` tasks, `> [!note]` callouts, headings, lists, quotes, code.

Underline stays `<u>…</u>`, since Obsidian has no native underline. Ctrl+E
toggles Write/Preview, as Obsidian's reading-view shortcut does.

## 2026-09-27 — Ruling BG: landslip and pinnacle landforms

Asked by the owner with a photograph of the Trotternish ridge and the Old Man of
Storr (Isle of Skye), and whether the generator could make it. It can already
make the glacial lochs, the moorland and the stepped basalt escarpments. It
cannot make the landslip (rotational slump blocks tilted back beneath a
ragged scarp) or the pinnacles, which are far below one cell. The owner:
*"I want the landslip and pinnacle forms."*

**Landslips** are a generation process. They act on steep escarpments where
a strong caprock lies over a weak substrate. Each one:
- cuts a ragged arcuate scarp back into the plateau edge;
- rotates and back-tilts the detached blocks downslope;
- leaves hummocky ground whose hollows fill with small lochans.

There is also a sculpt stamp to place a landslip by hand. The process moves
generated output, so it re-baselines with every old → new golden value
disclosed.

**Pinnacles** exist at two scales:
- as placed, named landmarks with their own marker, on slipped blocks and
  sea cliffs;
- as spires that appear in the LOD tile detail at deep zoom (ties to EF-9).

Sea stacks use the same rule on coastal cliffs.

It is scoped first: rock-strength source, slump model with its literature,
a measurement proving it fires only where geology supports it, and the
milestones.

## 2026-09-27 — Ruling BH: geology first; every landform process reads it

Owner: *"I believe it's important for landforms and terrain to be informed by
the geology layer and age layers and lithology layer. Can we order the
generator in such a way all this informs the geological features and how strong
they are carved and created by the simulator."*

Where the code stood when this was ruled (checked 2026-09-27):
- `build_age_field` derives rock age from the plates.
- `compute_resistance` sets resistance to 0.6 × crust type + 0.4 × age.
- Resistance drives **only** `stream_power_kernel` (with
  `recompute_resistance_after_erosion`).
- Lithology is a derived label that informs no shaping process, except the
  optional `dynamic_lithology`, which is off by default.
- Thermal, glacial, coastal, the river carve and volcanism do not read
  rock.

The ruling:
1. **Lithology is a real model computed before any erosion.** Each cell gets a
   rock type from its plate setting, age, volcanism and deposition. Each type
   carries properties: strength, erodibility, jointing and layering,
   solubility and permeability.
2. **Layering is explicit.** Caprock over a weak substrate is recorded as such.
3. **Every shaping process reads it:**
   - stream-power incision (already partly);
   - the thermal talus angle;
   - glacial erodibility;
   - coastal cliff retreat;
   - karst in soluble rock;
   - landslip susceptibility;
   - where pinnacles and stacks survive.
   Resistant rock stands as ridges, scarps and cliffs; weak rock becomes
   lowland.
4. **Stages are reordered** so geology is complete before any process that
   reads it. `EROSION_GEOLOGICAL_TIME_SCOPE.md`'s geological time is the
   clock they run on. *Correction, same day, from `GEOLOGY_FIRST_SCOPE.md`: that scope has no clock (the owner declined one on 2026-09-02), so BH builds none; see the scope's owner question 4.* *Superseded the same day by Ruling BJ: the owner chose to add a simple clock (the scope's GF-7).*

It moves every generated world, so it re-baselines with every old → new
disclosed (DECISIONS §7p). It is scoped first. **Ruling BG (landslips and
pinnacles) builds on this model**, so the order is: BH scope, BH build, BG.

## 2026-09-27 — Ruling BI: map style presets and new drawing techniques

Asked after `MAP_STYLE_RESEARCH.md` (`6f8b303`).

**Build all eight researched presets:**
- Atlas, Imhof relief and Blueprint, all buildable from today's knobs;
- Ink wash and Woodcut, which need small renderer additions;
- Vintage atlas;
- Nautical and Night, which need new ramps.

Each follows the research document's knob values. They join
`render_workspace.gd`'s `STYLE_PRESETS` and take their colours from the
preset itself.

**Schedule all four new techniques**, each scoped first, since each is a
new rendering method:
- true Tanaka illuminated contours;
- Raisz physiographic pictograms;
- mappa-mundi figurative icons;
- Ordnance Survey-style symbology.

Where the research flagged the existing `hachure` field's drawing as
unverified, verify it before any preset relies on it.

## 2026-09-27 — Ruling BJ: the geology-first scope's owner questions

Answers to four questions in `GEOLOGY_FIRST_SCOPE.md`. Its other questions keep
their defaults.

- **Rock types: 11** (the scope's table).
- **Painting lithology: yes, in this build.** Hand-painting rock types (like
  biome painting) is part of BH, not a later GF-8. Erosion responds to the
  painted rock.
- **Coastal pass: on in the app at GF-4**, when cliff retreat starts
  reading rock, so hard headlands and soft bays follow the geology.
- **Time: add a simple clock.** Scope a geological-time parameter that scales
  how far each process runs. This reverses the "no clock" default. The
  2026-09-02 decline covered the erosion-time scope's larger design; the
  owner now wants a simple one.

## 2026-09-27 — Ruling BK: code is annotated so a human can audit it

Owner: *"I'd also like the code to be properly annotated so it can be easily
audited by a human. Take this with you for work to be done and all the code
that is done we'll do later."*

**Standing, from now on:** every change leaves its code auditable.
- Every function, module and type has a doc comment saying what it does, why
  it exists, and what it must never do.
- Every non-obvious constant says where its value comes from: a measurement
  (with the command or probe that measured it), a citation, the reference
  (with the v2.11 function), or a labelled judgement.
- Every non-obvious branch, workaround or deliberate divergence says why,
  and names the ruling or commit behind it.
- Every test says what it protects and what would make it fail.
- Comments describe the code as it is. Prose that a change makes stale is
  fixed in the same change (MISTAKES: "Change behaviour → hunt the prose").

This is part of "done" for every lane, and the main loop checks it when
verifying.

**Existing code** gets a separate annotation-audit pass later. It is a
backlog row, not in scope for current lanes, except for code a lane already
touches.

## 2026-09-27 — Ruling BL: Night's rivers stay dimmer

With rivers now drawn through the style (the per-style rivers lane), Night's
rivers pass through the preset's −0.35 exposure grade and read dimmer than
the old overlay's bright cyan. The owner was asked whether to keep them
luminous (draw them after the grade) or leave them as part of the image:
*"Stay dimmer."* Night's rivers go through the grade like the rest of the map.

## 2026-09-27 — Ruling BM: relief tracks rock by construction and by uplift-driven erosion

GF-2, GF-3 and GF-7 (`22ec647`, `ec4e078`, `5875636`) built rock-aware stream
power, a threshold hillslope and a geological clock, all behind
`geology_processes`, which is off in the app. None made relief track rock
(B1/B2/B4, `GEOLOGY_FIRST_SCOPE.md` §5.6–§5.8). The light pass lowers land by
1–7 m, rebound returns 78–93 % of that, and the carve sets channel depth.
With no ongoing uplift there is nothing for differential erosion to act
against.

Asked how relief should come to track rock, the owner chose **both**:
1. **Rock-aware construction.** Rock strength enters how the heightmap is
   built, before erosion, for the big shapes: hard caps held up as plateaus
   and scarps, weak rock lowered.
2. **Uplift-driven erosion.** An ongoing uplift field, derived from the
   existing tectonics, lets differential erosion wear weak rock faster and
   leave hard rock standing, for the detail.

Scope both before building. They re-baseline app worlds when switched on,
with disclosure. The clock (GF-7) should then act on uplift plus erosion, as
the scope originally intended.

## 2026-09-27 — Ruling BN: the Ruling BM scope's owner questions

Asked after `GEOLOGY_FIRST_SCOPE.md` was amended for Ruling BM (`14c53a4`):
- **Q14, coastline: yes.** Rock-aware construction may move the coastline.
  Weak coastal rock can be lowered below sea level into new bays and inlets,
  and hard headlands stay. This reverses the scope's default ("land stays
  land") and must still make no inland pits.
- **Q12, uplift sources: all of them.** Convergence, flexure and the
  continental background, **plus rift shoulders (uplift) and basin
  subsidence**. This reverses the scope's narrower default.
- **Q18, order: relief first** (GF-10 → GF-13), then GF-4, GF-5, GF-6 and
  GF-8, with GF-9 last.

The other questions (Q13, Q15, Q16, Q17, Q19) keep the scope's defaults.

## 2026-09-27 — Ruling BO: overnight standing answers

Given before the owner stepped away, so work can continue without them:
- **Geology relief (GF-10 → GF-13):** if every measurement bar passes,
  **switch `geology_processes` (and the relief model) ON in the app**, and
  disclose the world change in the commit. If any bar fails, leave it off.
- **UI designs derived from the DCC vocabulary** (for example, the planner
  sheet covering the tab bar, the map-drag hint, phone failure states):
  **build them, commit, and leave screenshots for the owner to review
  later.**
- **Forced lakes** (the lake tool): **draw them.** A forced lake is real water
  on the map, and labels and rivers respect it.
- **The stale checkout** (`cartalith_wt_route`, in the user folder): if its
  uncommitted edits have already landed, delete it; otherwise report it.
  *Done the same day: its `way_render_geometry` / `render_brks` edit had
  already landed in `cartalith-godot/src/lib.rs`, so the checkout was
  removed.*

## 2026-09-28 — Ruling BP: a freeze during generation is acceptable; zoom and sharpen must be fast

Owner, verbatim: *"A freeze during generation/sim isn't an issue even the html
had loadscreen messages. As long as zooming and sharpening is fast"*.

- A multi-second main-thread stall while generating, simulating or doing a
  full recolour (`build_color_texture`: 2.6 s at 2048², 48.6 s at 8192²,
  measured 2026-09-28) is **accepted**. It is not a defect to engineer away.
- The 16.7 ms per-frame bar applies to **interactive** paths: zoom notches,
  pan, and deep-zoom tile arrival ("sharpening"). Work there is judged
  against it.
- Same day, owner: *"But if it can be sped up I'm not against it"*. So
  speeding up generation or the full recolour is welcome when it is cheap
  and changes no output (byte-identical), but it ranks below the interactive
  bar. No generation work needs to be moved off the main thread for its own sake.

## 2026-09-28 — Ruling BQ: v2.72 and v2.73 do not exist

Owner, verbatim: *"v2.72/73 this version doesn't exist. I was mistaken."*
Confirms the same-day note already on record (§ the v2.73 footpath row, above):
*"Version 2.71 seems to be latest, 2.73 doesn't seem to exist."* **v2.71 is the
latest real source version.** The v2.72/v2.73 entries in `RC_ENGINE_CHANGES.md`
§8 were recorded by `6aa1ff3` (2026-09-20) from an unverified source and reached
this branch in the merge `24384a4`.

- `RC_ENGINE_CHANGES.md` keeps both entries as history, each marked at its
  heading as not a real source version, and its span/count figures corrected
  (v2.11 → v2.71; 47 items, not 50).
- Per `DECISIONS.md` §7p, what this port already built from those entries stays
  built — the raster river display-area bar and the village-green plaza `kind`
  are real, tested improvements. Their provenance is re-labelled everywhere as
  "a port improvement; its spec was recorded as RC v2.72/v2.73, which does not
  exist," in `STATUS.md` and in the code comments that cited them.
- The v2.73-half-2 footpath class was never built and has no source to port
  from; its `OUTSTANDING_WORK.md` row is closed rather than left blocked. The
  idea can be re-raised as a port-original feature if the owner wants one.
- `OUTSTANDING_WORK.md`, `STATUS.md` and `FUNCTIONAL_CONTRACT.md` had each
  carried a "v2.72"/"v2.73" count or provenance claim; all are corrected in the
  same pass as this ruling.

## 2026-09-28 — Ruling BR: show the original raster under a painted layer, on an opacity slider

Owner, verbatim: *"when painting layers like terrain/biome is to have the
original terrain/biome raster on an opacity slider so you can see where the
current ones sit."*

- While a paint tool is armed on a raster layer (terrain/height, biome, and
  any other paintable raster), the shell can show that layer's **original**
  (as generated, before any hand painting) raster together with the current
  one, with an **opacity slider** controlling the original.
- It is a viewing aid only. It never changes the data, the save or the
  export.
- Routed as a backlog row in `OUTSTANDING_WORK.md`.

## 2026-09-28 — Ruling BS: journey supply = foraging plus suggested resupply stops; the all-or-nothing block goes

The owner noted that the planner often marks journeys blocked by cargo and supply
constraints, and chose to combine two proposals: live foraging and resupply
stops. The owner's answers, verbatim: *"Stops should be suggested, Trade is
possible, the old rule should go."*

- **Foraging first.** Per leg, what the land provides (forage, grazing,
  hunting, water sources, from biome × season × wildlife) is subtracted from
  the party's need. Only the remaining gap is carried.
- **Suggested resupply stops.** Walking the route, where the carried gap would
  exceed carrying capacity, the planner **suggests** a stop at the last
  feasible point before it: settlements restock everything (trade is possible
  there), rivers, lakes and oases restock water, and good pasture restocks
  fodder. The user accepts or declines each one. An accepted stop becomes a
  stage with its own time and cost, on the map and in the log.
- **The old rule goes.** The reference's hard block on carry capacity is
  retired (DECISIONS §7p). A journey is blocked only when a single stretch
  between two possible resupply points cannot be carried even after
  foraging, and the message names that stretch.

## 2026-09-28 — Ruling BT: roads are routed on a finer grid on large worlds

Diagnosis (commit `63bedc8d`): on the owner's world (2048×1311, seed 246371),
20 of 236 ways stop 59-585 cells short of their own settlement. The routing
grid is capped at `RW = min(GW, 384)`, as in the reference. Corridor
consolidation claims shared cells busiest-edge-first, and the reference's
deliberately bounded endpoint snap (`_snapT2`) cannot reach back to the
settlement. This is inherited JS behaviour. It becomes visible only above the
384-cell cap, which the reference's canvas never exercised.

Two fixes were offered: extend each short way back along its own corridor, or
route on a finer grid at large world sizes. Owner, verbatim: *"The alternative
is what i prefer, document it"*.

- The routing grid scales with world size instead of stopping at 384. The
  cap is raised or tied to the world grid, so a 2048-wide world routes near
  full resolution. It is a deliberate improvement on the reference
  (DECISIONS §7p).
- Worlds at or below 384 cells route exactly as before, so no golden moves.
  That is a hard check.
- Generation may get slower (Ruling BP accepts a freeze during generation).
  Measure the cost at 2048 and 8192 and report it. If it is severe, bring
  back a bounded option rather than silently capping.
- Done means every way's endpoints reach its own settlements on the owner's
  world. `_roadsettle_probe` checks each way against its own A→B
  settlements, not the nearest endpoint.

## 2026-09-28 — Ruling BU: rivers become water in the map itself, not a line layer on top

Owner, verbatim: *"the rivers already on the map should be more profound
themselves, like the new lines we have created. But this time in the map
itself, no longer the current depressions."*

- **Direction, not a build order.** Rivers stop being a stroke drawn over a
  finished raster (RV-2/RV-5's current shape) and become membership in the
  map's own per-pixel colour field, merged seamlessly with RV-4's sea/lake
  shoreline field. RV-3's shaded valley groove — the "current depressions"
  named above — is retired for any cell the new river-water field covers.
- **Scoped in full** as `RIVERS_IN_MAP_SCOPE.md`, milestones RIM-1 through
  RIM-7, marked FIRST NEXT in `OUTSTANDING_WORK.md` §2.5. Nothing is built
  by this ruling; it authorizes the direction and the scope document that
  follows it, per this file's own convention of recording the reasoning
  rather than re-asking the owner later.
- **River generation is untouched.** This is a rendering-layer change only,
  in the same class as RV-2 through RV-5 before it — no generation golden
  moves.
- Supersedes the "zoom-sensitive rivers" research row's standalone framing
  (folded into RIM-5) and §13's "rivers stay a separate system from the
  terrain colour raster" framing in `TERRAIN_APPEARANCE_SCOPE.md`, both
  noted at their own documents.

## 2026-10-05 — Ruling BV: river density no longer eases the channel threshold on slope, on the integrated path

Owner, on being shown the coastal river combs and three options: *"Follow up with 1"* (option 1 of three; the other two, draw-side comb thinning and both together, are not built).

- On the integrated-drainage path only, channel initiation uses `thresh/density` with no slope factor whenever `river_density != 1`. The reference path and density 1 are unchanged; no golden moves.
- Reason and numbers: `DECISIONS.md` §7p's 2026-10-05 paragraph and `STATUS.md`.
- Combs are reduced, not eliminated; draw-side thinning (option 2) stays an open, owner-gated row in `OUTSTANDING_WORK.md`.

## 2026-10-10 — Ruling BW: four owner calls on labels, import, river combs and old saves

Owner, answering four questions put by the coordinating session:

1. **Region labels: offer both, not one default.** The owner asked that the user can choose between fixed (constant on-screen size) and scaling (grows with the map) for region labels. Shell default stays as shipped (fixed) unless the size-mode control already offers both, which is checked, not assumed.
2. **Legacy HTML import: migrate the label `sizeMode` on import** so an imported label behaves as its author intended (the reference's `fixed` means grows-with-terrain; the shell's Fixed means constant on screen). Native saves are unaffected.
3. **River combs: build option 2, draw-side thinning** (spacing about L/2 after Hovius 1996, with an order or discharge cap so a real lighter parallel river is not hidden; delta fans re-verified). No change to river density or thresholds (Ruling BV).
4. **Old saves: turn `integrate_drainage` on when loading a project saved before it existed**, accepting that those worlds' rivers visibly change on load and need a re-baseline of any affected check.

## 2026-10-10 — Ruling BX: hatching fix, preset styling and the editing-tools scope

Owner, answering the coordinating session's questions:

1. **Diamond-lattice / diagonal hatching in the base texture: authorised, candidate (a)** - relax sub-threshold 1-cell grooves after the light stream-power pass, scoped to that pass's output. A re-baseline of generation-path world hashes and cell-colour hashes is authorised (RV-1 pin precedent); parity tests against the JS reference are not to be moved.
2. **Style presets also restyle** icon glyph inks, journey-marker labels, and pin and faction colours. The coordinator's caution stands as a design constraint, not a refusal: faction colours identify ownership, so a restyle must keep factions distinguishable (a contrast guard), and pins keep their meaning.
3. **Android build:** made after the running agents land (colour confirm flow, ramp bar, continent-label fade, legacy import), not after the queue.
4. **Editing tools (the 2026-10-10 PhotoCraft/MapTiler row), scope:** (C) adjustment stacks are PER LAYER (terrain, water, labels/overlays), not one whole-map stack; (D) the brush engine applies to biome/colour paint, height sculpt, and river/road drawing; (E) colour management covers exported PNGs with an embedded profile, a print (CMYK) gamut warning in the picker, wide-gamut/HDR display, and importing images with profiles; (F) a Look control with hue, saturation and brightness sliders plus dark/light/pastel variants of each preset.

## 2026-10-10 — Ruling BY: geology construction alone, and one shared look-undo history

Owner, answering the coordinating session's questions after GF-10 measured NO-GO (0 of 5 seeds on the pre-registered rule; `GEOLOGY_FIRST_SCOPE.md` §5.10):

1. **Ship rock-aware height construction alone (GF-11); do not build the uplift-driven erosion stage (GF-12).** The prototype showed construction makes real scarps and keeps B8 on 4 of 5 seeds, and that the uplift stage is what erases them and adds lakes and cost (B9 2.65x). GF-11 therefore needs its own bars, written before the measurement; GF-12 stays unbuilt. Landslip landforms (Ruling BG) may build on GF-11.
2. **One shared undo/redo history for look edits** (ramp stops, biome colours, style presets and later adjustment stacks), one entry per gesture, separate from the heightmap undo unless unifying them is trivial. The ramp bar and the colour dialog already produce one `final` callback per gesture.

3. **(2026-10-10, after GF-11's first measurement) Existing lakes are exempt from B14.** `GEOLOGY_FIRST_SCOPE.md` §4.14 (a depression on the structural surface keeps its depth) and B14 (every cell newly below sea is ocean) contradicted each other; the owner ruled that an existing lake bed may go below sea level as a lake, and B14 judges only land that was not already a depression. The GF-11 retry must register its amended rule BEFORE measuring and disclose that it was written after the first result.

4. **(2026-10-10, after the GF-11 retry) The exemption extends to G3.** Lake cells in existing depressions of the structural surface are counted and reported but not judged by G3 (lake share on river paths and the 1-3-cell lake count), consistent with item 3. The rule is amended again BEFORE the next measurement and the amendment is disclosed as made after seeing results. Seed 24601's 6.75 % lake share was above-sea pools in kept depressions, not below-sea lake beds.

## 2026-10-10 — Ruling BZ: editing-tools order, hatching candidate (b), generate freeze, v2.71 file

Owner, answering the coordinating session's questions:

1. **Editing tools build order: D, C, E, F** (brush engine, per-layer adjustment stacks, colour management, Look control with variants).
2. **Base-texture hatching: try candidate (b)**, shading from a lightly low-passed height (as RV-3's `valley_shade_field` does). One `cell_color` hash re-baseline is authorised (RV-1 pin precedent); `golden_parity_*` tests against the JS reference are not to be moved. Candidate (a) failed and was reverted (STATUS.md, 2026-10-10).
3. **Generate freeze: move water bodies and valley shading to a worker thread** to cut the main-thread block (about 1.5 s at 2048, about 5.8 s at 4096), after a cache-key audit; the window stays responsive with a progress indicator.
4. **The v2.71 DCC line file is not available.** The re-freeze (`OUTSTANDING_WORK.md` section 2.8) stays blocked; the port keeps following the DCC line from `RC_ENGINE_CHANGES.md` and the specs as they stand.

5. **(2026-10-10, answered earlier the same day, recorded late) Three small calls.** (a) EF-9.1: **keep the 5-term importance fit** (pooled 2.74x gain, accepted by the pre-declared rule at a +0.003 margin) rather than ship slope alone (2.42x). (b) **Colour relief (`ramp_strength`) keeps its shipped default of 0**, with the hint under the ramp rows saying why ramp edits are invisible at 0; no look change. (c) **The 4 px screen floor on map handle radii stays** (visibility over exact hit-circle accuracy when zoomed far out).

## 2026-10-11 — Ruling CA: GF-11 cost bar, lake fix to be shown first, label culling, river comb factor

Owner, answering the coordinating session's questions:

1. **GF-11 cost: accept about 1.2x.** The G5 construction-cost bar is relaxed from 1.20x to 1.25x (measured 1.211 and 1.187). This unblocks the rest of the geology-first line (GF-2/3, GF-4 to 6, GF-8/9, GF-13) and the landslip landforms that take their inputs from it.
2. **Lake fix: show the difference between candidate 1 (basin-area floor) and candidate 2 (per-basin water balance) before choosing.** Nothing is built until the owner has seen the comparison. (`LAKE_FILL_RESEARCH.md` shortlist; the basin id and outlet data exist, `WaterBodies::basin` and `basin_outlet`.)
3. **Generated labels: keep exact culling** (no overlaps; fewer names at the shipped size; zooming in reveals more). Not relaxed, and no regenerate-on-resize.
4. **River combs: thin more strongly than the current factor** (S = 0.5 x length, cap 24 cells). The new strength is not fixed by the owner: the coordinating session proposes S = 1.0 x length with a cap of 32 cells, to be shown on a few seeds before it ships. Ruling BV still applies: no river density or threshold change.

## 2026-10-11 — Ruling CB: the editing-tools owner questions, rivers in exports, label weight, FH-R2

Owner, answering the coordinating session's questions (the numbering is `EDITING_TOOLS_SCOPE.md` section 11 where it applies):

1. **PNG tagging (scope question 3): yes, tag every exported PNG as sRGB by default.** File bytes change, pixels do not; Display P3 stays an explicit export option; the heightmap PNG stays untagged (ET-E1).
2. **Style variants (scope question 5): dark, light and pastel are generated by rule** over each of the 17 presets, each of the 51 renders checked by the contrast guard, with hand-tuned overrides only where a guard fails (ET-F2).
3. **Exports match the screen for rivers:** exports carry the floodplain tint and the bank line as the on-screen look does (closes the rivers-in-map owner nod).
4. **Thin-glyph label weight: a heavier weight on Woodcut, Ink and Vintage**, as a per-preset label weight so the lettering stays readable.
5. **HDR (scope question 1): ship wide-gamut now and defer HDR display and HDR export until asked, AND investigate why Forward+ lost the device on the owner's RX 7800 XT** before any HDR decision (ET-E4 splits into the wide-gamut part, which proceeds, and a Forward+ investigation).
6. **CMYK print-gamut warning (scope question 2): the owner's own ICC profile plus a labelled generic fallback**, CIEDE2000 with the threshold measured from the profile's noise (ET-E3).
7. **Roads (scope question 6): add freehand (unrouted) roads** as well as least-cost routing (ET-D4 gains a freehand mode, an explicit toggle; the routed mode stays the default).
8. **FH-R2 (religion expansion S1 to S7 plus the S3 ruling): scheduled after editing-tools milestone F.**
9. **Paint undo (scope question 4)** was built at its default, one undo step per stroke, and is not contradicted by any answer.

## 2026-10-11 — Ruling CC: lake fix is a basin-area floor that scales with world size

Owner, after seeing the measured comparison of candidate 1 (basin-area floor) and candidate 2 (per-basin water balance) in `LAKE_FILL_RESEARCH.md`:

1. **Build candidate 1, the basin-area floor, at a 4-cell floor** at the reference size (the harness worlds, 2048 x 1311 cells). Candidate 2 (water balance) is parked until single basin ids, a temperature-based evaporation term and river/lake consistency exist.
2. **The floor must scale properly with world size.** The owner did not answer whether Ruling BY binds a lake fix with either offered option; the answer given was that the floor has to scale to the world. Read as: a fixed cell count is wrong, so the floor is expressed so that it means the same on a small grid and a large one (the coordinating session's reading: 4 cells at 2048 x 1311, scaled by the grid's cell count relative to that, never below 1 cell; a labelled judgement, to be checked against the harness worlds). Ruling BY is read as limiting only the GF-11 bars, not a lake fix; lakes the owner or the paint tool forced (`apply_force_lake`) are never removed.

**Addendum (2026-10-11, to Ruling CA item 4).** Strength chosen after measurement: comb spacing S = 2.0 x stream length, capped at 32 cells (both labelled judgements; Hovius 1996 supports only the order of the ratio, about 0.5). Measured over 16 (factor, cap) pairs on four worlds: the cap never binds, nothing real is lost at any pair, and factors above 2 to 4 add only short stubs. On the 512 x 328 / 40 075 km world 52 strands are removed (28 before) and streaks with a parallel neighbour fall from 54 to 39, but the interior hatching there is untouched: it is out of the comb rule's reach and would need a new draw-side rule that changes `COMB_DISCHARGE_FRAC` (a river threshold, Ruling BV), so it needs an owner ruling.

## 2026-10-11 — Ruling CD: journey planner conformance (rivers, slope, way types)

Owner, after `PLANNER_AUDIT_2026-10-11.md`:

1. **River travel: build it.** Real river channels become navigable for River Transport, with direction from the flow (the routing surface) and speed scaled by discharge and order: faster downstream, slower upstream. Port-only change per DECISIONS 7p; the planner river-stage golden may be re-baselined for it, disclosed as made after seeing the result, and no other golden moves without its own ruling.
2. **Land speed: slope only.** Slope affects planned land speed on and off roads. River crossings and fords/bridges stay verdict text (not in planned time). Moves the planner land goldens: re-baseline disclosed the same way.
3. **Way types differ, from one shared table.** Road, Track, Ancient and sea lane each get their own cost discount in the Route tool's router and their own planner row, read from a single shared table so the router and the planner cannot drift.
4. **Small port-side fixes the audit found, no ruling needed beyond 7p:** the sea condition must be computed for the vessel a stage actually uses; a hidden generated way must not stay credited through its edges; the Route tool's way list must include `civ.road_edges` so the router and planner see the same network.

**Addendum (2026-10-11, to Ruling CD).** 5. **Track gets its own slower planner row, and the reference-verified planner goldens that move because of it are re-baselined** (owner, answering the coordinating session's question after batch 1: the candidate was `Dirt Track / None / Wild`, which moved the M5/M6/SP-2 plans from 41.318 to 43.203 days). The re-baseline is disclosed as made after seeing the result; every moved value is listed old and new in `STATUS.md`; no other golden moves without its own ruling.

## 2026-10-12 — Ruling CE: the New World seed preview is built, with an approximation disclaimer

Owner, after reading `NEW_WORLD_PREVIEW_PROPOSAL.md` and seeing its mockup: **"Let's put the new screen in with a disclaimer that the preview is an approximation."**

1. **Build milestones NWP-1 to NWP-4** of the proposal: the engine's own early stages at 320 px with the two cell-unit settings rescaled to the target resolution (recommendation (a)), a `preview_land_mask` binding, and the new two-column New World dialog with the grey-on-blue preview, the seed dice and lock, the four-thumbnail history and the plates control.
2. **A visible disclaimer is part of the screen**, on the preview itself, in plain words: it is an approximation of the plate layout and the land/ocean split, not the finished map (it will not show fine coastline, small islands, lakes or relief).
3. **The proposal's recommended defaults stand** for its open questions unless the owner says otherwise: Create uses the width-scaled plate count (so the preview shows the plates the world will have; 800 km stays at 14, today's default), "36 for world" means the 40,075 km width, log-linear scaling, the seed lock lasts for the session, four history thumbnails (session only), grey #8f9295 on blue #2b4d6b, and under an archetype the derived plate count is shown read-only.

## 2026-10-12 — Ruling CF: the open-rulings round (chroma, Night, press ICC, planner goldens, FH-R2, river gates, output moves)

Owner, answering the coordinating session's prompt for rulings, in the owner's selections:

1. **ET-F1 chroma bar: the exact-grey Rust fix**, not a restated tolerance. Greys must stay exactly neutral through the Look path. It may move look outputs; re-baseline them and disclose it (DECISIONS §7p).
2. **Night (Dark): lift the darkest surfaces slightly** for legibility.
3. **Press `.icc` loader: deferred.** Own synthetic ICC v4 profile plus the generic fallback stand until a real press profile is available to test.
4. **Planner leftovers (seam-run endpoints, off-map km): fix and re-baseline** the pinned goldens that move.
5. **FH-R2: proceed with a plan first.** Draft the research read (S1-S7 location to be found or asked) and the re-baseline plan for owner sign-off before any code.
6. **Order-versus-discharge river gate and River Transport discharge gate: measure first**, report numbers, change nothing yet. Ruling BV (no river density/threshold changes) stands meanwhile.
7. **Parcel-water re-baseline and authored-battle spacing: do both, with re-baseline** and disclosure per DECISIONS §7p.
8. **Delta look and interior hatching on 512x328: deferred**; Ruling BV stands.

**Addendum (2026-10-12, to Ruling CF item 6).** The measurement (`RIVER_GATE_MEASUREMENTS.md`, "Chain breaks") found a defect: `cartalith_hydrology::build_channels_core` steers each channel cell to an aspect-steered receiver that is not the D8 receiver the discharge flows to, so 3,759 order-3+ chains break across 37 worlds. **Owner ruled: option F1, the full fix** — a receiver guard in `build_channels_core` (when the steered receiver carries less discharge than the cell itself, use the D8 receiver), on the integrated-drainage path so the reference-path goldens stay put; one network for generation, drawing and planning. It moves app-world rivers and everything downstream: look at frames first for a staircase look, then re-baseline and disclose per DECISIONS §7p. The gate thresholds themselves stay as they are (Ruling BV); re-run the gate measurements after the fix before any gate decision (owner questions 9-11 take their defaults: keep the draw-side bridge and re-measure loose ends; count a river leaving the map border as reaching an outlet in the measurement only).

## 2026-10-12 — Ruling CG: GF-13 stays off, FH-R2 R2-0/R2-1 approved, planner slope gains Tobler asymmetry

Owner, answering the coordinating session's prompt after GF-13's assessment (`GEOLOGY_FIRST_SCOPE.md` §7 "GF-13 under Ruling BY", commit `0e2ea5ff`) and the planner slope fix (`c0f41f98`):

1. **GF-13: keep `relief_model` off.** The revised B4 holds on 3 of 5 seeds (12345 fails on share, 314159 on ratio); per GF-13's rule and Ruling BO a failed bar means leave it off. App worlds are unchanged.
2. **Investigate the deep-zoom staircase** (one-cell staircase on lowered edges in constructed worlds; arm A has none) and fix it behind `relief_model`, then re-measure, before any further switch-on decision.
3. **FH-R2: approve Q0 and Q2** of `FH_R2_PLAN.md` — import the research doc into the repo, and keep the new religion/government archetypes out of the default pick so S1 moves nothing. Start R2-0 and R2-1.
4. **Planner slope: add Tobler asymmetry** (downhill faster than uphill, so A→B and B→A plan differently). Slope does NOT additionally stack on the Hills / Mountain Trails terrain rows (the owner chose the asymmetry option only). The port-only planner goldens that move are re-baselined and disclosed per DECISIONS §7p; reference-verified goldens (slope off) must not move.
   **Addendum, owner clarified 2026-10-12:** keep the existing slope on Hills / Mountain Trails; asymmetry only. (The sentence above reads as if slope stops stacking on those rows; the first build, `833665d5`, did that and was corrected the same day.)

**Addendum 2 (2026-10-12, to Ruling CG item 2).** The follow-up (`5587de37`, `GEOLOGY_FIRST_SCOPE.md` §7 "Follow-up: what the remaining artefacts are") found the remaining deep-zoom stair is the lithology colour sampled at the nearest cell (`RenderCtx::litho_at_f`), shared with app worlds. **Owner: build the lithology blend as its own renderer row** (blend the lithology colour terms of each jittered corner's rock by the bilinear weights), as a divergence from the reference's nearest read under DECISIONS §7p: measure, look at frames, re-baseline and disclose any pinned export/snapshot/deep-zoom pixels that move, and confirm the fit-view map and `js_reference` goldens stay. Follow-up owner questions 5 (flat-resolution gradient), 6 (3-cell feather) and 7 (check the deep carve in GF-4..GF-9 measurements) take their defaults: no, no, yes.

## 2026-10-12 — Ruling CH: GF-4 bars and glacial wiring

Owner, answering the coordinating session after `GEOLOGY_FIRST_SCOPE.md` §7 "GF-4 measured" (commit `5627735d`):

1. **B7 is revised to leave out the tidal-marsh noise** (owner chose to revise rather than keep the bar). It is disclosed as a change made after seeing the result: B7 measured 1.27-1.49x with the marsh and 1.62-2.17x without it, and passes on only 2 of 4 measurable seeds even then. The revision must be written into the scope with that disclosure and must not be presented as a pre-registered bar. The coastal flip in the app (Ruling BJ) is not approved by this; B8 (small-lake rise on 2 of 5 seeds) still stands as a failing bar and the small lakes are to be investigated (the scope default) before any flip.
2. **Wire the glacial rock kernel** (`glacial_kernel_rock`) into every processes-on world, with the `replica_reproduces_generate_terrain` test in `gf0_geology_harness.rs` changed in the same commit and a disclosed re-baseline per DECISIONS §7p. App worlds do not change (processes remain off there).

**Addendum (2026-10-13, to Ruling CH).** The small-lake rise (B8) was found (`456ca989`, `GEOLOGY_FIRST_SCOPE.md` §7 "GF-4 under Ruling CH") to be the tidal marsh: its noise is centred on zero, so it lowers about half the flat cells in its band and digs 1-3-cell hollows that count as lakes. **Owner: make the marsh accretion-only (`max(0, bump)`) in BOTH the reference coastal kernel and the rock kernel**, fixing it at the cause; the app-path reference marsh golden is re-recorded with disclosure (DECISIONS §7p, a deliberate change away from legacy, Ruling AP/BN stance) and frames are looked at. **And investigate the second small-lake cause** on processes-on seed 24601 (169 small lakes with the marsh off against a control of 49) before any switch-on of the processes. The coastal flip in the app (Ruling BJ) remains unapproved until B7/B8 are re-measured.

**Addendum 2 (2026-10-13, to Ruling CH).** The second small-lake cause (`49c55056`, `GEOLOGY_FIRST_SCOPE.md` §7 "GF-4: accretion-only marsh and the second lake cause") is the coastal pass's estuary cut, which lowers every high-discharge cell up to `estuary_depth` (about 552 m above sea at the default 0.08) and leaves closed hollows. **Owner: cap the cut so a cell is never lowered below its downstream neighbour, in both coastal kernels**, measured first, with the coastal goldens re-recorded with disclosure (DECISIONS §7p) and frames read. **B7 (revised) still failing on seeds 24601 and 71077345 is accepted; the coastal flip in the app stays off.**

## 2026-10-17 — Ruling CI: the photoreal target render and the zoom-gains-detail requirement

Owner, after `RENDER_TARGET_GAP_ASSESSMENT_2026-10-17.md` (commit `adb12bf5`) against the second target render (`design/vision-render-2026-10-17/target_map_render.webp`, VISION.md "Second target"):

1. **Zoom that gains detail at every zoom level is a hard feature** (standing; Ruling K / LOD_DETAIL_SCOPE.md). The assessment measured detail per pixel falling at every step past about x4 (median mean gradient 2.48 → 1.10 → 0.77 from x4 to x64; unique colours 42k → 25k → 4.6k). **Approved: re-key `add_zoom_detail`'s frequency to kilometres and change its octave decay** (LOD-D5's recorded open question), as a golden re-baseline done behind the app-side `params::defaults()` divergence so the JS-parity (`js_reference`) goldens stay put (DECISIONS §7p, `MISTAKES.md` "Change generated output"): measure zoom ladders before and after, disclose each re-baselined app pixel pin old → new, read frames at x4 / x16 / x64.
2. **Add the target as a new named look ("Satellite relief")**, with no frame or paper, a navy deep ramp and a turquoise shelf, and a **coast-distance shelf/halo term in the sea colour** (0 in `js_reference()`, on in the new look). **Natural Vibrant stays the default** until the new look passes its own zoom ladder.
3. **Investigate mountain relief first** (the highest ground reads as a snow blob with no ridges; the height field itself may be the limit — EF refinement / relief gain / ridge shading at x4–x64), then forest canopy, then the square mottle (`bio_jitter` on/off test). Assessment questions 5 (overlay defaults) and 6 (world shape) take their recommended defaults: leave the default; world shape is a World Structure archetype question, not rendering.

**Addendum (2026-10-18, to Ruling CI).** After the zoom-detail re-key (`1fc9208f`: mean gradient at x64 0.41 → 2.89, the km law on in the app look, `js_reference` unchanged) and `MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md` (`36930b48`):
1. **Real mountain ranges in the height field: not yet.** Parked as a row (the investigation's H1/H2/H3 measurement spike and the multi-sheet belt); the owner wants to look at the new zoom detail and the Satellite relief look first.
2. **Snow shedding by slope and slope rock go into the new "Satellite relief" look ONLY** (the investigation's R1/R2), alongside the navy deep ramp, turquoise shelf and the coast-distance shelf/halo term of Ruling CI item 2. Natural Vibrant and the reference rule stay as they are.

## 2026-10-18 — Ruling CJ: FH-R2 Q1 and Q4 approved

Owner, answering the coordinating session after R2-0/R2-1's vocabulary half landed (`f69b9515`): **approve `FH_R2_PLAN.md` Q1 and Q4 at the plan's defaults.** Q1: the religion axes are approved and the existing eight are classified as the plan proposes, with `flame_creed` a dualist, exclusive-leaning religion (distinct from `one_god`). Q4: S2's three terms (retention split, axis-derived competition, trade-diaspora influx) and M2's bars as written in the plan's §6 are approved, including **replacing `belief_has_no_syncretic_equilibrium`**. Build R2-1's `ReligionProfile` table and R2-2 in that order, measurement-first (M0 baseline hashes already recorded; M2 bars gate it). **Q9 (governments) stays out of FH-R2** and needs its own row and ruling; if governments are ever built, exclude the new forms from `civ_default_government` first. The default pick still cycles only the original eight religions (Q2).

## 2026-10-18 — Ruling CK: the story menus proposal approved at its defaults

Owner, after seeing the mock (`design/story-menus-2026-10-17/mockup.html`) and `STORY_MENUS_PROPOSAL.md`: **approve Q1–Q7 at their recommended defaults.** Q1: add an eighth top-level menu, `Story`, against `DCC_SHELL_SPEC.md` §2's seven (with the rail section and the card verbs). Q2: on tablet, merge Data's rows into the existing tablet `World` menu below its pipeline rows. Q3: the renamed Data menu is called `World`. Q4: move export to `File ▸ Export ▸` (imports stay; a signpost row remains in World). Q5: Session mode is a toggle in `Window` that also seeds a layout. Q6: STORY is a node-list section of the rail, not a fourth domain (Ruling L item 4 forbids a standalone notes category). Q7: `Add event` stays limited to the four conflict kinds until a free-event store exists. Milestones SM-1 to SM-7 are scheduled in that order; probes that pin menu contents, rail nodes or card labels are updated in the same batches; `ROADMAP.md` Phase 6 is the home.

## 2026-10-18 — Ruling CL: EF-9 ships ON by default

Owner, after the EF-9.4 continuous effective-level rebuild (`273bc220`; `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.5): **ship EF-9 (`LodBudget::ef9_refine`) ON by default in the app.** Measured on vs off: I4 seam 18/18 vs 17/18, pops 0 vs 13, D3 8/12 vs 8/12 (the failing runs are today's display path in both states), payoff 33/34 views, about +0.9 ms per frame and +2.4 ms per camera step (medians). Done as a tier default with the toggle still available (`viewport_host.gd::set_ef9_refine`); it is a deliberate app-side change (DECISIONS §7p): the viewport moves, so pinned LOD/viewport pins are re-baselined with disclosure, and `js_reference` and every grid digest stay byte-identical. The cheapest tier may keep it off if its frame-time budget needs it.

## 2026-10-18 — Ruling CM: terrain-streak spike outcome

Owner, after `TERRAIN_STREAK_SPIKE_2026-10-18.md` (`df64f48c`): the radial streaks on 71077345's dome come from the volcano edifice model (`EDIFICE_GULLY_SPACING_CELLS`, the §7l-ii ruling), not from the river carve or the flow routing; the earlier "diagonal fine structure" figures were a measurement artefact. **Owner: (1) re-scope the dome streaks as an edifice and zoom-law row** (gully spacing keyed to kilometres, sub-cell gullies at zoom, fix the pits the gullies leave; measure first, no change yet); **(2) retire the sub-cell carve (b) and the Rho8 routing (c) variants** (scrub the superseded approach, do not dual-path — DECISIONS §7p / the engine-improvement standing rule); **(3) ALSO ship variant (a), the smooth (B-spline) rain up-sample**, switched on in the app-side `params::defaults()` (0/off in `js_reference()`), as a deliberate app-side change with a disclosed re-baseline: it removes the square crease lattice from the rain field and moves every app world's rain-derived values. **(4)** Locate 314159's remaining streaks with the river layer hidden before any further generation change (the straight vertical river strokes are the comb already in the backlog).

## 2026-10-18 — Ruling CN: a non-destructive colour-paint tool in CARTO ▸ Colours

Owner, 2026-10-18, answering where painting belongs: *"The carto function has a colours tab. That's also where we need to have the terrain paint tools and colour selection to choose a colour for the terrain."* Read as a **presentation tool**, separate from WORLD paint (Biome, Terrain, Splat and Rock paint edit world data and stay in WORLD per `DCC_SHELL_SPEC.md` §4.5.2): **build a non-destructive "Paint colour" tool in `CARTO ▸ Colours`** — a brush plus a colour picker (with the print-gamut check the colour picker already carries), strokes stored in map kilometres so they gain detail at every zoom level (the hard requirement of 2026-10-17), composited consistently in the screen, deep-zoom tiles and export, saved with the project as an additive optional slot, with its own undo, hideable and clearable, and never touching world data or generation. Blend modes (replace / tint / multiply), behaviour across a regenerate (by position, in km) and the storage format are the scope's design questions, with defaults. The owner chose the layer itself (recommended option), not only shortcuts to WORLD paint; shortcut rows from CARTO into the WORLD paint tools remain a separate, optional nicety.

## 2026-10-18 — Ruling CO: volcano pit fill (c-late) ships; edifice follow-ups take their defaults

Owner, after `EDIFICE_AND_STREAKS_2026-10-18.md` (`a0d0d1af`): the volcano pits come from the edifice gully term (4 cells: about 1.56 km at 800 km, about 5 km at 3000 km; at Earth width every volcano is clamped to a 2-cell radius). **Owner: ship variant (c-late), the pit fill on the FINAL field** (1-3-cell lakes on volcanoes from a median of 41 to 10, outlet agreement 99.93 %, frames indistinguishable from today at every zoom): off in `js_reference`/parity, ON in the app-side `params::defaults()`, as a deliberate app-side change with a disclosed re-baseline (DECISIONS §7p). The doc's other questions **take their defaults**: retire variant (a) as built in favour of a design row for type-dependent gully spacing and a gully depth capped by the cone; give tile-time furrows (b) a believability pass, then wire them into the deep-zoom tiles (zoom requirement); fold the 314159 streak row into the river-comb row (the streaks are the river comb: height north-south excess 1.48 falls to 0.21 with the carve off; no streak at any zoom with the river layer hidden); file the coarse-scale 2-cell radius floor as its own row.

## 2026-10-18 — Ruling CP: GF-9 waits for a switch-on of the geology processes (route A, later)

Owner, after `GEOLOGY_FIRST_SCOPE.md` §7 "GF-9 opened at its symbols" (`21dfc52e`): GF-9 is the last geology step and assumes the app ships the geology processes ON, which Ruling CG keeps off (GF-13's revised B4 holds on 2 of 5 seeds; the coastal B7 fails on 2 seeds). **Owner: wait, then switch on and take route A.** GF-9 stays parked; nothing is built now and the processes stay OFF in the app. A later switch-on is a deliberate, disclosed re-baseline of every app world (DECISIONS §7p) decided by the owner, after which route A deletes the old/new switches and removes `tect.dynamic_lithology`. Until then: G9-2 (build the civ projection early behind its own off switch) and G9-3 (remove `dynamic_lithology` before the switch-on) take their defaults (no). GF-6's leftover (RV-3 valley shade does not read the carve width `ω`) and GF-8's leftover (Rock draft layers not saved in `drafts/paint.json`) remain open rows.

## 2026-10-18 — Ruling CQ: asset pack refinement tools approved at the proposal's defaults

Owner, after `ASSET_PACK_TOOLS_PROPOSAL.md` (`c2e7eeb7`): **approve Q1–Q10 at the recommended defaults.** Q1: *Import pack* also fills the library as the reference's `importPackZip` does, and a separate *Open pack in library…* row is kept. Q2: draw settlement, POI, sea-mark and custom-family art on the map, behind a switch that is on in the app. Q3: move feature icons to a sprite pass that stays sharp at deep zoom, after AP-5's measurement, with export following the screen (an engine change: its pixel pins are re-baselined with disclosure, `js_reference` untouched). Q4: harmonise a pack to the Look by baking pixels. Q5: per-sprite pivot is baked as pan, no pack format change. Q6: no size-per-kilometre yet. Q7: refine tools write a new variant and never replace the original. Q8: keep the blank-licence → "CC0" export bytes but confirm before exporting with a blank licence. Q9: tags and licences stay in the project, not in `pack.json`. Q10: the loose-folder import adapter first. **Build AP-1 first**, then the first cut (pack report, seam checker, near-duplicate finder, batch fit/trim/pad, variant generation), all within Ruling F (pack format unchanged). The proposal's hard zoom requirement applies: sprites must keep quality at ×16 and ×64.

## 2026-10-18 — Ruling CR: line-art conversion method and AP-10

Owner, 2026-10-18, on the asset pack tools: *"be conscious about what might be needed to convert line art to proper assets. In the html version the lines weren't solid enough and often it became a blobby mess."* After `LINEART_METHODS_RESEARCH_2026-10-18.md` and `ASSET_PACK_TOOLS_PROPOSAL.md` §9 (`909d08bc`): **approve the research's ranked pipeline and new milestone AP-10 (line-art conversion as a refine tool), with the recommended defaults** — Q-L1 solid lines at ×1 (a 1 px minimum stroke width) with a per-variant line-weight control (0.5–1.5 px, default 1.0), Q-L2 Otsu after paper flattening (not Sauvola), Q-L3 rebuild the signed distance field on load rather than save it, Q-L4 keep 256² pack sprites, Q-L5 vector-trace SVG export later, Q-L6 an optional "even line weight" verb (off by default). The pipeline: clean the ink at import at full source resolution (local paper flattening, threshold, despeckle, pinhole fill, scans-only 1 px gap closing), store a signed distance field of the cleaned mask (at least 2 texels across the thinnest stroke), render it with a 1 px edge ramp and 1 px minimum stroke width, one ink colour from the Look palette baked into the variant (Ruling CQ Q4), export keeps the PNG pack format. It writes a NEW variant, never replacing the original (Ruling CQ Q7). Fixing the blit (`blit_sprite`: one bilinear sample while shrinking 5–9×) and the bake (`render_item`: CatmullRom on straight alpha) is a separate, cheaper change behind a switch. Potrace is GPL and must not be linked. All measurements so far are on synthetic samples: real scans are to be tested in AP-10.

## Ruling CS — painted icon edges, colour-paint default blend and layers, Fit all, 3D approved (owner, 2026-10-19)

Four answers to the questions raised by AP-5, CP-2/CP-3, AP-7 and `3D_VIEWPORT_SCOPE.md`:

1. **Painted feature icons stay faithful** when magnified (AP-5 Q-P1): no alpha sharpening, so soft art and drop shadows survive. Line-art icons already meet the ≤ 1 px bar.
2. **Colour-paint default blend is Colour** (the owner's answer was *"1, but use paint kayersy"* — read as: option 1, **Colour**, **and organise paint into layers**). Interpretation recorded here; the layers part is to be confirmed against the first build: named paint layers with order, visibility and per-layer clear, each holding marks, composited in order before the light. Lava defaults stay as shipped until the owner rules otherwise.
3. **AP-7 Fit all stays placement-only** (no new variant; Undo restores placements).
4. **The 3D vertex-mesh viewport (`3D_VIEWPORT_SCOPE.md`) is approved** and 3D-0 (the measurement gate) is scheduled. `DECISIONS.md` §4 ("2D only for v1") gets a superseding entry beside it.

## Ruling CT — 3D vertical exaggeration (owner, 2026-10-19)

After 3D-0 (`791ad1d1`, GO for 3D-1) the owner chose, from the true-scale vs 3.4× captures at ×16/×64: **the 3D view defaults to true scale, 1.0× vertical exaggeration** (*"The first truescale 1 @ 16x"*), not 2D's 3.4×, which draws needles. This is 3D_VIEWPORT_SCOPE.md Q4. The ruling does not yet cover a user slider; 3D-1 may add one only if it costs nothing.

## Ruling CU — mountain ranges un-parked; Satellite relief land tone pass (owner, 2026-10-19)

The owner, seeing Satellite relief near-identical to Natural Vibrant on land and unlike the target render, said *"unpark and continue"* to the assistant's plan: **(1)** un-park the mountain-range row (Ruling CI's addendum had parked it as *"Not yet"*; `MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md` H1, ridged / dissected orogen height field, `relief_model` under world structure, the volcano clip), and **(2)** build a land tone pass for the Satellite relief look only (tonal range, saturation, relief and edge contrast measured against `design/vision-render-2026-10-17/target_map_render.webp`; switch-guarded, other looks byte-identical). Both are judged at ×1/×4/×16/×64. Order: tone pass first (cheap, measurable), then the mountain height-field work under the investigation's §4 defaults, each re-opened at its symbols before building. One agent at a time (owner, 2026-10-19).

## Ruling CV — mountain ranges stay on at 24 dissection iterations (owner, 2026-10-19)

After `782ec8c7` (ridged, dissected orogen ranges; one of three bars, the channel fraction, fails and the measure itself falls when ranges are dissected): **keep `tect.orogen_relief` on** in `params::defaults()`, and **accept 24 dissection iterations** (about +3.4 s per 2048×1311 world generation). The multi-sheet belt stays its own `OUTSTANDING_WORK.md` §2.9 row. Also the master Landmarks switch hides hand-placed POI icons (commit `4be27175`) and the landmark zoom law's numbers stand.

## Ruling CW — volcano appearance approved at the research defaults; order of the remaining work; two agents (owner, 2026-10-24)

The owner directed: *"Do volcanoes first, the 6T isn't connected, after the textures from then on whatever is still outstanding."* Answers to the follow-up: **all nine questions in `VOLCANO_APPEARANCE_RESEARCH.md` §8 at the recommended defaults (all yes)** — a render-only classification first; a per-volcano age in years on a log scale up to 3 million years; active/dormant/extinct derived from age; Satellite relief first with every other look byte-identical; a subtle capped daytime glow on active craters; volcano age overriding the slope-rock warmth rule; the shape corrections (they move the height field, an app-only divergence) only after measuring final heights; new types in the research's order (caldera with lake and resurgent dome, lava dome, maar, flood basalt); a tephra/tuff rock palette slot. **Order: volcano appearance (VA-0…), then the forest texture for flat lowland in Natural Vibrant, then whatever remains outstanding.** The 6T is not connected, so device passes (3D-0(c), CP-4, E5, AP-8 on Android) stay open and are not attempted. **Agent cap: two at a time** (owner, 2026-10-24; supersedes the one-at-a-time instruction of 2026-10-19).

## Ruling CX — volcano morphometry on, ages re-spread, and the crater-lake rules (owner, 2026-10-24)

After VA-5 (`a45b4e6d`, built behind `volc.morphometry`, off at both boundaries) and the VA-1 report's finding that at the default `volc.age` almost no volcano is extinct on the approved age map, the owner decided:

1. **Turn `volc.morphometry` on in the app**: `params::defaults()` on; `WorldParams::defaults()` and `js_reference()` stay off. A disclosed `DECISIONS.md` §7p re-baseline of every app world's volcanoes. A save without the key reloads as saved (off).
2. **Re-spread volcano ages** so a world has a realistic mix of young, dormant, old-green and extinct volcanoes: map the age draw onto the log age scale so that about 500 ka falls at the dormant boundary, as the research's §6.1 justified. This changes only the age table and the look, **not the height field or any random draw order**. The hotspot chains keep their extinct members. Save compatibility: the optional `volcanoes` key already saved must still load, and an old table reloads as saved.
3. **Fix `center_landmasses`**, which shifts every world grid except the geology column (so a centred world's rock is misregistered): re-open it at the symbol, write a failing test first, fix it, and shift the volcano table consistently.
4. **Try a grid-resolution-aware cinder-cone law.** The 0.39 km grid and erosion hold cinder height-to-width at 0.06-0.12 against the cited 0.15-0.21. Measure first on the five census worlds (71077345, 24601, 314159, 483920, 246371). Keep the law only if it measurably improves toward the cited ratio without breaking the rivers, lakes, settlements and watersheds bars.
5. **The acid crater-lake colour**: turquoise or emerald for the crater lakes of active and dormant volcanoes in Satellite relief, per the research (§3.5, §6.2).
6. **(Owner addition, the same day.) A young volcano with magma or lava in its crater must not be filled with a lake.** An active volcano (age below the active threshold in the volcano table) keeps a dry crater showing the lava and the glow. A dormant one may hold an acid (turquoise or emerald) crater lake. An extinct one may hold a normal lake. Switch-guarded like the other volcano behaviour. It is a generation-side water change, so it is measured on the water bars: ocean cells on river paths, 1-3-cell lakes within 1.25x, outlet agreement. Any golden or pin that moves gets a RE-BASELINED comment and a `DECISIONS.md` §7p line.

Each item is judged at x1/x4/x16/x64 with before/after frames, and committed separately. **Not in this ruling:** VA-6 (new volcano kinds).

## Ruling CY — places of worship drawn, then the rite switched on; FH-R2 Q6 and Q7 confirmed as built (owner, 2026-10-24)

After FH-R2 R2-4 stopped at its switch (`49be2c5d`: the app draws no place of worship, so a new rite would only reshape an undrawn gap in the houses; `STATUS.md`'s R2-4 entry), the owner decided:

1. **Draw places of worship in town views, then switch the rite table on** (R2-4 / S3a). Draw every town's place of worship in its rite's building form in the City Viewer and anywhere towns are drawn, for every town including unaffiliated ones (a settlement with no faith takes the culture's default form). Then pass each settlement's faith into town layout, so the rite table applies: **temple** for `sun_cult`, `sky_pantheon`, `god_king`; **shrine** for `earth_mother`, `sea_lords`, `old_gods`; **mosque** for `revealed_law`; **church** for `one_god`. The other seven religions keep today's towns until S4 gives them forms of their own. The app today passes no faith to town layout and the shell never receives the town buildings (`Town::churches`); both are what this changes. Measured on the same bars as the R2-4 report: site count, centres and plots unchanged; every house that moves is explained by the new building. Any golden or pin that moves gets a RE-BASELINED comment and a `DECISIONS.md` §7p line; old saves reload as saved. **Not in this ruling:** S3b / R2-5 (plural sites, minority quarters, deity siting) and governments (G1, Q9).
2. **FH-R2's defaults are confirmed as built:** Q6, a custom religion is its base profile plus overrides, applying in its holders' lands only (R2-3, `STATUS.md`); Q7, every faction starts on `plural`, with the lose/win rates tolerant x0.5/x0.5, exclusive x2/x0.25 and persecuting x4/x0.1 (R2-6, `STATUS.md`).

## Ruling CZ — governments (G1) approved at every recommended default (owner, 2026-10-25)

After `GOVERNMENTS_SCOPE.md` (`a2567163`), the owner **approved all nine of its §9 questions at the recommended defaults**:

1. **Q1:** append the six core forms (`merchant_republic`, `city_league`, `sacral_monarchy`, `holy_order`, `feudal_realm`, `nomad_khanate`) after `city_state`, in the scope's §2 order.
2. **Q2:** accept the 12 authored, uncalibrated manpower rows of §3.1-§3.2; each row's doc comment says "authored ordering" and names its anchor or labels its judgement.
3. **Q3:** build the optional pair (`stratocracy`, `tributary_hegemony`) as G1-2, both at `CITIZEN_SHARE` 0.30, so §4.1 does not arise.
4. **Q4:** freeze both derivations (`CITIZEN_MODERNISATION`'s identity and `SOLDIER_UPKEEP_BY_BRACKET`'s geometric mean) to the first nine forms, the reference's, with the 0.30 floor pinned as a literal. New forms are priced by the calibration, not into it.
5. **Q5:** build the khanate's levy knob (`GOVERNMENT_LEVY_BONUS`, one row) as G1-5, its value set by §3.3's rule and pinned with its derivation.
6. **Q6:** keep every new form out of `civ_default_government` (`CIV_GOVERNMENTS_REFERENCE_LEN = 9`), as Ruling CJ already required. G1-6 is not proposed and not built.
7. **Q7:** no relabel of `theocracy`; the hierocracy reading goes in its doc comment only.
8. **Q8:** no coupling of government to the economy, relations, religion or the timeline in G1; the picker hints say what is not modelled.
9. **Q9:** group the picker by axis, with the stored vocabulary order unchanged.

**Build order:** G1-0 (measurement column on the 20 FH-R2 harness worlds, with a positive control), G1-1 (the guards first, then the six core forms), G1-2, G1-3 (hub UI, including the unknown-key display that must never read "None / Unclaimed"), G1-4 (save compatibility, no `format_version` bump), G1-5, each with the scope's numeric bars and committed separately. Zero movement of generated output is the bar throughout; a pin that moves without the scope saying it should is a failure, not a re-baseline.

## Ruling DA — terrain-following lava flows, step by step (owner, 2026-10-25)

The owner asked: *"Will the revised volcano system also go with lava flows? Like in Iceland and Hawaii and such, not only at waterline but also on land"*, and queued it: *"Que it after the current volcano work"* (`OUTSTANDING_WORK.md` row "Terrain-following lava flows"). With the Ruling CX items landed, the coordinating session briefed the build in the row's own order, each step measured at x1/x4/x16/x64 before the next:

1. **Render-only terrain-following flows** for each young or active vent (the volcano table's age and activity, the edifice's position): traced downhill on the world's own height field with lateral spread and branching, young-lava colour and a capped daytime glow while active, cooling to dark basalt, then weathering and vegetation by each flow's own eruption age; distinct from rivers; consistent in the screen, the deep-zoom tiles (kilometre-keyed) and the export; Satellite relief first, every other look byte-identical; the length law from the cited literature, judgements labelled; flow state reproducible from the seed and the volcano table (no new random draw, old saves reload as saved).
2. **Fissure flows and lava fields** on the same logic (Iceland), only if step 1 measures well and reads believably.
3. **A lava delta or bench** where a flow meets the sea only after measuring, since it moves the land mask: behind a switch, generation-side, on the water and settlement bars; otherwise stop and report the numbers and the decision needed.

Decisions taken inside that brief (labelled judgements, open to the owner): the flows are drawn for **active and dormant** edifices only (an extinct one's flows are older than 500 kyr, the age the look already greens or weathers away); the length law is a log-uniform draw per flow from bands bracketing cited flows (shield 8-50 km: Kilauea 2018 about 13 km, Mauna Loa 1984 25 km, 1859 51 km, USGS; basaltic stratocone 4-20 km; andesitic stratocone 1.5-8 km; scoria cone 1-8 km, the last three judgements), since the engine knows neither effusion rate (Walker 1973) nor volume (Malin 1980); the seven radial colour ribbons `volcano_look` drew inside each cone are **retired** in favour of the traced flows (DECISIONS §7p: scrubbed, not kept beside); a Satellite relief look saved before the term (no `lava_flows` key) reopens with the flows off, as it was saved.

**Outcome of step 1 (2026-10-25):** built (`STATUS.md`, Ruling DA step 1). Steps 2 and 3 are not built; the decision each needs is in the `OUTSTANDING_WORK.md` row. The brief's other item, the caldera lake cut by a straight edge on 71077345, is real (H2's soft ceiling flattening a tall cone's high side) and is **not fixed**: a cone-scaling trial failed the archipelago's settlement bar and did not remove the edge; the decision it needs is its own `OUTSTANDING_WORK.md` row.

## Ruling DB — fissure fields, a lava bench and the higher world ceiling (owner, 2026-10-25)

The owner answered the three decisions Ruling DA left open (`OUTSTANDING_WORK.md` rows "Terrain-following lava flows: steps 2 and 3" and "Volcano summits flattened at the soft ceiling"), to be built in the order 3, 1, 2, each measured:

1. **Fissure fields:** derive the rift direction at generation from the plate boundary and record it in the volcano table (an optional save key; old saves reload as saved), then render fissure fields and spatter lines in Satellite relief.
2. **Lava delta: yes**, a small bench at the sea may add land; measure it against the settlement-within-3-cells and outlet-within-5-cells bars and the lake and coast metrics on five worlds before keeping it; cap the bench to labelled-judgement sizes (Kilauea 2018, about 1.5 km² of new land, USGS); if the bars fail, keep it behind a switch and report.
3. **Summit ceiling edge on 71077345: "Higher world ceiling"** (every world re-baselined, disclosed). Built first; if raising the ceiling fails a bar, stop and report rather than tune. Sea-level behaviour to be checked (24601 sets its sea level from the height histogram; the earlier attempt moved coasts).

Decisions taken inside the build (labelled judgements, open to the owner): the higher top is relative to the sea, `1 + (1 - sea)` in field units, i.e. twice `peak_m` in metres on every world (a fixed 2.0 let the Hawaii-like seed 8 reach about 26 600 m); the World Structure sea level is solved on the field as the old top left it, so no coast moves through the solve; the heightmap exports keep their 0..1 encoding and clip summits above 1 (a known limit, not changed).

**Outcome of item 3 (2026-10-25):** built and on in the app (`STATUS.md`, Ruling DB item 3); every bar held.

**Outcome of item 1 (2026-10-25):** built (`STATUS.md`, Ruling DB item 1): the trend is recorded for every rift edifice (record-only, no generated value moves), and fissure fields are drawn in Satellite relief under the existing `lava_flows` look term. Judgements inside it (open to the owner): the fissure and sheet bands in `lava_flows`' module doc, an older second fissure beside the newest, and a shield's rift-zone vents taking the recorded trend.

**Outcome of item 2 (2026-10-25):** built and on in the app (`STATUS.md`, Ruling DB item 2): every bar held on the five census worlds, so no switch-off was needed. Judgements inside it (open to the owner): flows under 1 000 years build benches, water up to 50 m deep, 5 m above the sea, a smooth seaward lobe; benches are rare and small on the census worlds because most young flows meet deep water.

## Ruling DC — the optional LLM assistant: yes, with prose and commands, remote and local, phone and desktop, each behind its own setting (owner, 2026-10-25)

The owner answered the first three questions of `LLM_INTEGRATION_RESEARCH.md` §7 (the research follows his question *"what if we added LLM integration to Cartalith with an API; call a local model or whatever the user has; it has to work with an API key"*). Verbatim: **(a)** scope = prose **and** map commands, both; **(b)** providers = remote **and** local; **(c)** platforms = phone as well as desktop; and **"all three, but guarded by settings"**.

Read together: Q1 is yes (the feature is wanted); Q2 is both prose/Vault assist and natural-language commands; Q3 is both hosted and local providers. Each capability sits **behind its own setting**, and the **whole feature is off by default** (this is also the research's recommended Q6 default, but Q6 itself is not yet ruled). Generation stays model-free: the model never sits on the seed-deterministic generation path.

Consequences, as labelled judgements the owner may overrule:

1. **Command actions need a grouped-undo milestone first.** The research's own condition (§6 "Deferred", Risk 5, Q10): per-tool undo means a model-emitted batch is not one-step undoable today. Commands are therefore limited to view/layer/parameter actions (LLM-5) until a grouped undo exists; edit actions (sculpt/paint) remain a separate owner ruling (Q10 open). Every action is a proposal the user confirms, executed by the existing undoable setters.
2. **Phone is in scope, not deferred.** Android needs the `INTERNET` permission declared in the export preset (none exists today), a cleartext/LAN policy for local servers, and **Android Keystore** key storage (a plugin); LLM-8 is promoted from "later" to a v1 deliverable. Its unknowns (TLS roots, Keystore plugin, the open SAF vault milestone) stay the largest risks.
3. **Per-capability settings:** at least prose/Vault assist, commands, remote providers, local providers, each independently switchable, all off until the user turns them on; the consent screen names what is sent. With everything off, a full session opens zero sockets (the LLM-2 bar).
4. Both an OpenAI-compatible adapter (covers local and hosted) and the Anthropic adapter are in v1 (LLM-1, LLM-4).

**Still open:** Q4 (transport; default hybrid), Q5 (key storage with no OS store), Q6 (default state; the "off by default" above is the owner's wording but the consent-screen form is not ruled), Q7 (Vault writes marked as model-generated), Q8 (cost caps), Q9 (answered in substance by (c): Android is in v1; its sequencing relative to desktop is not ruled) and Q10 (edit actions). No scope document exists yet; the `OUTSTANDING_WORK.md` row records the next step.

## Ruling DD — the LLM assistant's remaining questions: hybrid transport, no plaintext key, consent screen, click-only Vault prose, no enforced cap, desktop-then-phone, edit actions behind grouped undo (owner, 2026-10-25)

The owner answered the questions Ruling DC left open (`LLM_INTEGRATION_RESEARCH.md` §7), as recorded by the main loop:

- **Q4 transport = Hybrid.** A Rust protocol crate (`cartalith-llm`: requests, SSE parsing, validation, redaction, mock-transport tests) over Godot's `HTTPClient` for the wire.
- **Q5 key storage where no OS credential store exists = environment variable or refuse.** No plaintext key file.
- **Q6 default = off by default, plus one consent screen naming what is sent.**
- **Q7 Vault writes = model prose is written to the Vault only on a user click, and marked model-generated.**
- **Q8 cost caps = NO enforced cap.** The owner chose "No cap". A token counter is still shown.
- **Q9 phone sequencing:** answered by Ruling DC (phone is in v1). Within v1, the research default stands unless the owner says otherwise: desktop first, then phone.
- **Q10 = the model MAY run edit actions (sculpt/paint)** once a grouped undo exists and is tested.

Consequences, as labelled judgements the owner may overrule:

1. **Grouped undo is a hard prerequisite for any edit action, and for commands generally.** No model-proposed action that changes the map runs before a single-step grouped undo exists and its test is green. Edit actions are still proposals the user confirms.
2. **Settings guard each capability.** Prose/Vault assist, commands, view/layer/parameter actions, edit actions, remote providers and local providers are each their own switch, all off until turned on; edit actions are a separate switch from commands.
3. **Q8 deviates from the research's recommendation, by owner choice.** `LLM_INTEGRATION_RESEARCH.md` §5.3 and Q8 recommended a default per-session token cap. The owner ruled no enforced cap. The deviation is recorded here rather than reverted: no cap is enforced by default, but a visible running token counter, `max_tokens` on every request and no automatic retry beyond one remain, since they limit a single runaway request, not spend. A cap setting may exist as an opt-in, off by default. The risk (an unbounded bill on a hosted key) is the user's and the consent screen must say so.
4. **No plaintext key anywhere.** Where neither Credential Manager nor Android Keystore is reachable, the key comes from an environment variable or the feature refuses to call a remote provider. This overrides the research's "last resort plain file".
5. **Consent form:** one screen, shown before the first call, naming what is sent (the prompt and only the sources the user opted in), to which endpoint, and the cost-cap deviation above.
6. `LLM_ASSISTANT_SCOPE.md` is the resulting scope document.

## Ruling DE — the ecological army term: C1 at gamma 0 plus C2 (owner, 2026-10-25)

After `ECOLOGICAL_ARMY_TERM_SCOPE.md` (`2789cf8a`), the owner approved the scope's recommended candidates, **C1 with gamma = 0 plus C2 (beta 0.5, capped at 0.90 of the levy), "as recommended"**. Q1 and Q2 are those words; Q3 to Q6 are read as the scope's recommended defaults, which is what "as recommended" names:

1. **Q1:** C1 at gamma = 0. The standing army's budget reads the faction's ecological factor `E` only where the land falls short (`E` ≤ 1, linear, unchanged by control flow); above 1, abundant land adds no revenue (Domar 1970).
2. **Q2:** yes, C2: a one-sided forage term on the field army, `field = levy · min(0.90, (0.34 + 0.20·L) · E^0.5)` for `E` > 1 only, so geography above 1 still moves a military output where the literature puts it (supply on the march). beta 0.5 and the 0.90 cap are labelled judgements the owner accepted.
3. **Q3:** keep `E` ≤ 1 linear.
4. **Q4:** keep `standing < field < levy` as an invariant asserted on the harness, empty factions excluded and counted, on the pre-industrial rows.
5. **Q5:** accept the §6 re-baseline under `DECISIONS.md` §7p (`G1_0` manpower, Kingdom B, the discrimination test rewritten), no switch for the old budget, old values beside new.
6. **Q6:** the industrial levy (§3 point 4) is its own row, not built here.

**Build order:** EA-0 (the diagnostic harness, committed first), EA-1 (the term), EA-2 (re-baseline, probes and prose), each with the scope's numeric bars; a bar that fails stops the build for a report rather than a tuning.

## Ruling DF — a local LLM provider on the same machine scales Cartalith back to the lowest-power GPU (owner, 2026-10-25)

Owner, on the LLM assistant (Rulings DC, DD): when a local provider is selected, show a warning that GPU resources should be changed; when the LLM software runs on the **same machine**, scale Cartalith back to the least powerful GPU (usually the CPU's integrated graphics) so the dedicated card is left to the LLM. He first said it need not be automatic if it cannot be checked, then: *"I think it should be easy to check though. It's visible in the ip address if it's on the local network or on the same machine. So maybe automatically switching should hinge on that with a user override."*

The rule, with its labelled judgements (the owner may overrule):

1. **Same-machine detection reads the base-URL host only.** `localhost`, `127.0.0.0/8` (so `127.0.0.1`), `::1` and `::ffff:127.0.0.1` count as the same machine. Private LAN ranges (`10/8`, `172.16/12`, `192.168/16`, link-local `169.254/16` and `fe80::/10`) and every remote host do **not**: the LLM runs on another computer, so nothing is switched.
2. **Behaviour.** Same machine: the automatic low-power-GPU preference applies while that provider is the active one. LAN or remote: no switch. The **warning is always shown for a local preset** and states what was detected and what will happen.
3. **User override**, a setting with three values: **Automatic** (default, rule 1), **Always low-power GPU**, **Never change**.
4. **Edge cases are covered by the override and a visible "detected as" line**, not by guessing: Docker and WSL (a container's `localhost` may or may not be the host's GPU), SSH tunnels and port forwards (a `localhost` URL whose server is elsewhere), a hostname that resolves to localhost, `host.docker.internal`. Hostnames other than `localhost` are **not resolved** (no DNS call, no network) and are classed "unknown, no switch".
5. **What the app can switch is limited, and the rule does not promise more.** See `LLM_ASSISTANT_SCOPE.md` s.5a: the compute path (wgpu) can be pointed at another adapter without a restart; the Godot display adapter is not known to be switchable from inside the running app, so for it the rule is the warning and the "detected as" line.
6. **Address classification is a pure function tested with no network** (new bar, milestone LLM-2), covering IPv4, IPv6, bracketed forms, hostnames, ports and a refused userinfo.

## Ruling DG — VA-6 new volcano kinds, measure first, build only the kinds whose bars hold (owner brief, 2026-10-25)

The coordinating session briefed VA-6 of `VOLCANO_APPEARANCE_RESEARCH.md` (caldera with lake and resurgent dome, lava dome, maar, flood basalt, and the owner's supervolcano note) **measure-first**: measure on five worlds (71077345, 24601, 314159, the Hawaii-like seed 8 and 483920; 246371 was measured too) what each kind changes at generation, against Ruling CX's bars (ocean on river paths 0, 1-3-cell lakes within x0.9-1.2, outlets within 5 cells at least 99.5 %, settlements within 3 cells at least 0.85, the sea level unchanged by construction as under `tect.high_ceiling`), with placement derived from the existing volcano table under the owner's three notes filed under the VA-6 row (the co-occurrence table; the binding rule that placement follows tectonics, magma supply and geological history and never spreads kinds for variety; supervolcanoes possible but very rare, zero or one per world, usually none, an old quiet caldera with an ash apron). Then build **only** the kinds whose bars hold, behind switches off in `WorldParams::defaults` and on in `params::defaults` as disclosed re-baselines; report a failed kind with numbers rather than tune it; write the owner's questions where the net is a design question.

**Outcome (2026-10-25):** the **lava dome** is built and on in the app (`volc.kind_dome`); the caldera rule, the supervolcano and the flood basalt failed bars; the maar held its bars but did not read on the map and is built off at both boundaries. Numbers in `STATUS.md`'s VA-6 entry; the questions for the owner in the VA-6 row of `OUTSTANDING_WORK.md`.

## Ruling DH — the custom quick bar: every recommended default approved, with the Assistant chip's condition recorded (owner, 2026-10-25)

Owner, on `CUSTOM_QUICK_BAR_SCOPE.md`'s Q1-Q11, approved the scope's recommended defaults. Recorded as answered:

1. **Q1. One global pin list** (not one per domain). Per-domain visibility is CQ-8.
2. **Q2. Pins sit at the palette bar's right end**, not in the contextual options bar.
3. **Q3. The ring dock, plus sharing a diagonal's sub-ring; no replacement of a built-in slot.** This lifts `MAP_CONTEXT_SCOPE.md` fork F5 (Ruling AX, "ring customisation: later") for those two things only; replacing a diagonal stays declined (CQ-8).
4. **Q4. The bar ships empty**, with the `＋` chip.
5. **Q5. Caps: 24 bar pins, 4 dock pins (plus the Assistant chip), 9 children per shared sub-ring.** 9 is the geometric ceiling in `radial_ring.gd`; 24 and 4 are labelled judgements.
6. **Q6. The Assistant chip appears automatically when the assistant is enabled AND a provider is configured, with a user switch to hide it; this condition is recorded as a ruling (this one).** The condition, exactly: *shown iff the assistant is enabled, a provider is configured, and the user has not turned the chip off.* The five states and the gate reason strings are `CUSTOM_QUICK_BAR_SCOPE.md` section 5, built in CQ-7 behind LLM-3's panel. It changes only by a later ruling.
7. **Q7. Per-device storage only**; no sync between desktop and phone.
8. **Q8. Toggle commands (check and radio rows) are not in v1** (CQ-8).
9. **Q9. Phone: a chip row in the MAP sheet and a section in MORE.**
10. **Q10. Tools are not pinnable in v1.**
11. **Q11. The Customise window and the chip only**, unless Godot's `PopupMenu` can report a right-click; CQ-3 decides that with a windowed check (CQ-0's headless finding is in the scope, section 11).

Consequences: milestones CQ-0 to CQ-7 are unblocked in order (CQ-7 still gated on LLM-3); a pin is stored per device and never in a project, export or archive (CQ-1's leak bar); `MAP_CONTEXT_SCOPE.md` F5 and the "later" sentences listed in the scope's risk 9 are fixed when CQ-4 lands.

## Ruling DI — the assistant's parameter-driving questions P1-P5: every recommended default approved (owner, 2026-10-25)

Owner, on `LLM_ASSISTANT_SCOPE.md` s.5b's owner questions P1-P5, approved the recommended defaults. Recorded as answered:

1. **P1. Reject an out-of-range or fractional model number; no clamping.** The proposal validator stays stricter than `params::set`, which clamps and rounds for a human's slider. After validation passes, any `clamped` or `rejected` entry from `set_params` is a bug and rolls the group back.
2. **P2. The model may flip boolean switches**, tagged "advanced" where a ruling is involved. The six excluded keys stay out.
3. **P3. Parameters and summary statistics are their own context-source checkbox**, off until ticked.
4. **P4. Two-step catalogue only**: no one-step "send the whole catalogue", even for a large-context hosted model, so there is one behaviour to test.
5. **P5. Never send map images for now.** Deferred; it is a privacy and cost decision and would need a later ruling.

Consequences: LLM-6a is unblocked with these as fixed inputs; the validator (`cartalith-llm` `catalogue.rs`) rejects rather than coerces; the context-source checkbox in LLM-3's panel gains a parameters-and-statistics entry; no image path is built; the scope's P1-P5 are marked answered and the `OUTSTANDING_WORK.md` LLM row carries the answers.

## Ruling DJ — VA-6's five questions answered: calderas as a disclosed re-baseline, maars relaxed, the supervolcano exempted from the bars, the flood basalt render-only, domes unchanged (owner, 2026-10-25)

The owner's answers to Ruling DG's five questions (the VA-6 row of `OUTSTANDING_WORK.md`), relayed by the coordinating session:

1. **Q1 calderas: accept the age rule as a disclosed re-baseline** that moves settlements, outlets and lakes near old stratocones. On in `params::defaults`, off in `WorldParams::defaults` and `js_reference`; old saves reopen as saved (optional save key); `DECISIONS.md` §7p. Still re-measure every Ruling CX bar on six or more worlds and report the exact moves (settlements within 3 cells, outlets within 5 cells, small lakes), keep the sea level identical by construction, and **do not silently exceed sensible sizes: report the diameters**.
2. **Q2 maars: relax the site rule and allow a small maar lake to stand** under the lake-area floor (check the 4-cell floor); measure; on in the app if the bars hold, else report numbers.
3. **Q3 supervolcano: exempt this one rare feature from the bars and build it**: zero or one per world, usually none, an old quiet caldera with an ash apron, not an active vent, placed by the tectonic rule already prototyped; report its local effects on the few worlds that have one.
4. **Q4 flood basalt: a render-only texture** (trap terrain and steps, a dark basalt plateau) that does **not** move the height field; Satellite relief only.
5. **Q5 domes: keep the faithful cited sizes** (sub-pixel at fit view, visible from x4); no change.

Binding, restated: placement follows tectonics, magma supply and history, never even spreading. Zoom detail at x1/x4/x16/x64 judged on frames looked at. If a kind fails a bar beyond what is accepted here, report numbers rather than tune.

**Outcome (2026-10-25):** all four built and on in the app; the measured moves, the caldera diameters (four above the cited 24 km), the supervolcano's local effects, the maar exemption and the frames are in `STATUS.md`'s VA-6 entry.
