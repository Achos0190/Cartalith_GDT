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
- **Rendering** (`cartalith-urban`'s `build_site`) does pick one real traced river polyline for local layout, but its binding test is `riverPath` truthiness — true for an empty or one-point path. A **known, deliberately-reproduced** bug (`URBAN_MORPHOLOGY_SCOPE.md:867-870`, golden-pinned as `pathOfOne`/`pathEmpty`), carried for parity, never recognised as something to fix.
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

## 2026-09-23 — Ruling AC: a citadel enclosure is sited on a settlement's size tier, not faction-seat status

**The finding is `OUTSTANDING_WORK.md`'s "a citadel enclosure straddling the town wall" row (Ruling I), whose own text named "settle the design first: which settlements get one" as a separate blocker independent of the GUI-sequencing gate (which was lifted 2026-09-22).**

**Owner ruling, 2026-09-23: largest settlements only, by size tier — not gated on faction-seat status.** Matches how the existing rare bastioned-wall star fort is already gated by size/class. Still open: whether the citadel's area counts inside the wall circuit for growth purposes (the row's own second named question), and the citadel's own build (nothing like it exists yet — `fortify.rs` has no castle-enclosure construction, only the curtain/gates/spurs/star fort).

## 2026-09-23 — Ruling AD: courtyard-perimeter blocks are gated by outermost ring, distance from market

**The finding is the urban-algorithm town-plan row's candidate 4, "perimeter lots with open courtyards held at high density"** — real, un-shipped work (courtyard buildings exist today only as per-parcel typology on strip lots, not the ring-around-a-shared-courtyard block subdivision the owner's image describes), blocked only on what "perimeter" means structurally: the outermost ring by distance from the market/town centre, a district flag, or a rules threshold.

**Owner ruling, 2026-09-23: the outermost ring by distance from market.** Matches how other density gradients already keyed on market distance work in this engine (e.g. Clark's demand-decay gradient in `growth.rs`). Not yet built — this ruling settles only the gating definition.

## 2026-09-23 — Ruling AE: IN-13 tariffs get their own relationship field, not `civ_faction_relations`

**The finding is IN-13 trade flows' second remaining sub-question**: whether a cross-faction tariff rate reuses `civ_faction_relations`'s existing pairwise score, or needs a dedicated field — `civ_faction_relations`'s own module doc is explicit that it deliberately is not diplomacy/treaties/vassalage, so reusing it for tariffs would repurpose a value past its stated scope.

**Owner ruling, 2026-09-23: a new relationship field**, not a reuse of `civ_faction_relations`. Not yet built — this settles the source, not the field's own shape or default values. Two of IN-13's four original sub-questions remain: confirming the faction-aware trade match doesn't move the existing single-faction/no-tariff probe output (a verification constraint, not an owner decision), and caravan semantics (settled below, Ruling AF).

## 2026-09-23 — Ruling AF: one caravan is one aggregate shipment per way

**The finding is IN-13 trade flows' fourth sub-question**: what a "caravan" entity represents — one `TradeFlow` row, an aggregate per way, or a manually-initiated shipment — which decides whether caravans are a visualization of the existing stateless trade match or a real simulated, persisted thing with its own movement/consumption rules against the Timeline's year cursor.

**Owner ruling, 2026-09-23: one aggregate shipment per way** — a periodic bundle of all matched trade volume currently routed over one way, not one entity per individual flow and not a manual player action. Closest in shape to `cartalith_civ::travel_library::Journey` (route + start_year + a persisted DTO), fewer entities than a per-flow model, easier to keep in sync with `trade_flows()`'s own deliberately stateless match. Not yet built — the exact tick/refresh cadence against the trade match and the Timeline's year cursor is still an implementation detail for whoever builds it.

## 2026-09-23 — Ruling AH: keep 2048×1311 as the Android resolution ceiling

**The finding is `MEMORY_OPTIMIZATION_SCOPE.md` §8's own row**: 4096px needs 2.41 GiB and 8192px needs 9.65 GiB, so 2048×1311 is the last preset that fits Android's real memory budget; the doc twice declined to change `RESOLUTION_PRESETS` unilaterally.

**Owner ruling, 2026-09-23: keep 2048×1311 as the Android ceiling.** Higher presets stay desktop-only where memory allows. Documentation-only — no code change needed.

## 2026-09-23 — Ruling AI: fix military manpower's era-fixed standing-army ratio, and make it vary by agricultural/industrial development too

**The finding is `MILITARY_MANPOWER_SCOPE.md`'s finding 2**: on sparse worlds (33 settlements), standing armies read 0.19–1.20% of population against a 1–2.5% target band, traced to the model sitting at Imperial Rome's fixed ratio regardless of era — the era table's own "standing" column was meant to vary, and never has. Fixing it moves outputs already validated against the owner's worked example.

**Owner ruling, 2026-09-23: fix it, and go further — the ratio should also vary by how agricultural vs. industrialised a nation is**, in the owner's own words: *"a heavily industrialised nation needs less manpower to foot a larger army than an agricultural nation that is dependant on individual farmers without machines."* **This maps directly onto an axis the engine already has**: `roster::AG_TECH_LEVELS` (6 tiers, `farmers_per_urbanite` derived from a real historical series — England's agricultural-labour-share data, CAMPOP/Broadberry & Gardner 2013 — already a per-faction field, `FactionEntry::ag_tech`, defaulting to `"traditionalAgrarian"`) — no new classifier needed, reuse it. Re-validate the fix against the owner's original worked example within a stated tolerance, not just against the sparse-world band.

**Correction, 2026-09-23, same day — this ruling's own premise was false.** A dispatched build read `manpower.rs` fresh (last touched `45630cc`, 2026-09-06, untouched by anything today) and found there is no era-fixed ratio: `standing = total × (1−α) × ecological_factor × fiscal_extraction_efficiency / SOLDIER_UPKEEP`, where `α = f/(1+f)` already comes from `AG_TECH_LEVELS.farmers_per_urbanite` — the exact mechanism this ruling asked for **already exists and is strong**, measured at an equal population/land/institutions: 1 090 (subsistence) → 24 617 (industrial) standing army, an **11.09× spread**. `ERA_BANDS` is read only for the pass/fail *verdict*, never as an input to the headcount — `era_for` is an *output*, which is what "era should not be the driver" actually asked for. "Imperial Rome's ratio" in finding 2 described a measured *result* at one ag-tech level, not a hardcoded constant — the only literal ratio in the chain is `SOLDIER_UPKEEP = 3.0` (Roman legionary pay against a subsistence wage, per its own doc, unrelated to era). No behaviour was changed; one mutation-tested unit test was added pinning the 11.09× spread. Re-measured with today's date (post Ruling 11's ecological-ceiling raise and today's government-default wiring): the sparse 33-settlement world now reads 3 of 6 factions within band (was 1 of 6 in the stale finding-2 text), and a 108-sample sweep across seeds/world-sizes found most of the remaining "below band" cases trace to a *different*, already-known effect (finding 3's map-scale/ecological-factor interaction: 33 of 36 below-band cases on an 800 km world vs. 11 of 36 on a 2000 km one) — not era, not industrialization. **Options for a fresh ruling**: (a) accept the ag-tech rule as already satisfied, correct finding 2's stale figures; (b) take on the map-scale normalization of land-capacity÷population separately (its own ruling, not this one); (c) rule on whether the era table's own band shape (non-monotone in α — the Iron Age band sits *above* the High-medieval one) should be reproduced by re-fitting `SOLDIER_UPKEEP` to vary with α, which would re-baseline the owner's original Kingdom A/B worked example. **`MILITARY_MANPOWER_SCOPE.md` finding 2 and this ruling's own "never has" wording are both stale** and need correcting once the owner picks an option.

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
