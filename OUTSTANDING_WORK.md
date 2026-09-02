# Outstanding work

> **Relationship to `cartalith-native/docs/STATUS.md`** (added 2026-08-31, when
> STATUS.md became the single source of truth for progress)
>
> **This file answers "what is left, and where do I go to do it?"** It is a
> *routed backlog*: every row names the document that owns the work, carries a
> size, and sits in one of six buckets — in flight, ready, blocked, open
> decision, declined, or a defect in the project record.
>
> **`STATUS.md` answers "what state is this in?"** — verified against the code,
> per milestone. **Where the two disagree about a status, `STATUS.md` wins**,
> and the disagreement is a bug in this file to fix rather than a fact to
> reconcile.
>
> The division is deliberate: a ledger of what exists and a queue of what is
> next are different shapes, and merging them is how the last `STATUS.md` grew
> to 8 122 lines. Keep it that way — **this file must not acquire a "done"
> column.** A row that is finished is deleted from here and recorded there.
>
> *One correction to the paragraph below, which was written hours before the
> rewrite it describes: `STATUS.md` is no longer 8 122 lines and no longer
> disclaims its own purpose. It was rewritten from scratch on 2026-08-31 against
> the working tree and is **1 157 lines**. The rest of the reasoning stands —
> the twenty-odd scope documents still answer the question only for themselves,
> which is why this file exists.*

The single list of what is left, assembled 2026-08-31 from every scope document
in this repository plus a code check of the claims that mattered most. It exists
because the question "what is actually left?" was costing a session most of its
budget: `STATUS.md` was 8 122 lines with four lines over 15 000 characters and
said in its own header that it no longer met its purpose, `CHANGELOG.md` stopped
five days short of the working tree, and the twenty-odd scope documents each
answer the question only for themselves.

**This document is a router, not a replacement.** Every row names the document
that owns it. Where a row's owning document disagrees with the code, the code
won and §6 records the disagreement — those defects in the project record are
worth more than any single milestone row below, and a reader with ten minutes
should read §6 before §2.

## The count, honestly

**128 outstanding items across 24 subsystems** — **re-derived by counting table
rows mechanically, 2026-09-02 (third pass)**, after an earlier pass left four
different totals in this file at once (a headline of 142, a table summing to 143,
and a report claiming 145). That is §6.8's own "counts that disagree with
themselves", reintroduced; the fix is the count, and the lesson is that the
arithmetic here is not safe to delegate.

The figure is `§1 + §2 + §3 + §4`, with §5's declined entries deliberately
outside it. A five-lane verification wave closed five rows on 2026-09-02 and
they are deleted: CARTO ▸ Labels, all three steps (§2.2, built and verified
against the code); the reference re-freeze to v2.11 (§2.8, done — `reference/`
now holds v2.11 and a regenerated `FUNCTION_INDEX_v2.11.md`); and
`_civPlaceSmelting`, `_civSaltAccess`, and `_civFactionAggregates`'s resource-
and density-fed half (all three §2.3, all found already built). One row was
added — whether the committed v2.11 is `Cartalith_RC`'s actual live head,
unresolved and unverifiable from this machine (§3.3). **A sixth row went in the
same pass**: *Label collision culling*'s §3.2 entry, one half of a
duplicate-classification defect this section has carried for days. It was filed
as blocked on the labelling pass; that pass landed in this wave, so the row was
not merely misclassified any more, it was false. The §2.2 half stands. Net,
taking 133 → **128**.
*Earlier the same day, a separate pass closed three rows taking 136 → 133: the
urban **17a caveat** (§2.1, which is now empty), and
`TERRAIN_APPEARANCE_SCOPE.md` **§16** and **§19** (§2.5).*

| | Count | Meaning |
|---|---:|---|
| In flight | 3 | Code exists, committed but partial (§1) |
| Ready to start | 72 | Nothing blocks them; someone has to pick them up (§2) |
| Blocked | 34 | A named blocker, listed in §3 |
| Open decisions | 19 | Not work yet — the owner owes an answer first (§4) |
| Declined / shelved | 25 entries | §5, kept so nobody re-proposes them |

Of the 35 blocked, **14 are blocked on an owner decision and nothing else** —
that is the largest single category of stalled work in the project, and §4 is
the shortest path to unsticking it. That 14 is checkable and checks out: §3.1
holds exactly 14 rows.

**Every count above was re-derived by counting table rows mechanically,
2026-09-02,** and the per-section figures are: §1 **3**; §2 0+16+5+6+11+22+9+3 =
**72**; §3 14+10+11 = **35**; §4 **19**. *The previous version of this paragraph
gave §2 as 98 and §3 as 33 while the table beside it said 80 and 34 — the
document reproducing, in its own count section, the exact defect §6.8 exists to
record. Both are now derived by the same script that produced the table, so they
cannot disagree.*

One thing does not reconcile, and it is a classification defect rather than an
arithmetic one: **one item is still listed twice, once as ready and once as
blocked.** *Saved measurements + CSV* appears in §2.2 **and** in §3.2, and a row
cannot be both, so the count of unique items is **127**. Left as found: deciding
which half to drop is a judgement about sequencing, not a text fix.

*This paragraph named **two** such duplicates until 2026-09-02. The other,
**Label collision culling**, is resolved rather than reclassified: its §3.2 entry
said it was blocked on the labelling pass, that pass landed in this wave, and a
blocked-row whose blocker has shipped is false rather than merely misfiled. It is
deleted and counted in the headline above.*

Four caveats on that number, stated rather than buried:

1. **It counts rows, not effort.** Urban milestone 10 is one row and ~407
   reference lines; "delete three probe files" is also one row. Sizes are on
   every row for this reason. The **109** rows that carry a size (everything
   except §4's 19 decisions — and 128 − 19 = 109, so the split is checkable
   against the headline) divide **36 large, 44 medium, 29 small**, re-derived
   2026-09-02 by the same script that counts the rows. *This caveat has now been
   overstated twice: it read "142 rows, 42/56/44" until 2026-09-01 and "134 rows,
   40/54/40" until today, both times because the sizes were counted by hand
   separately from the rows.*
2. **The `UNWIRED_FUNCTIONS.md` backlog is one row of the 3 "in flight" above,
   not many** — that document is itself a live backlog with a `file:line` per
   row, and re-counting it here would guarantee the two drift (this
   corrects an earlier version of this caveat, which pointed at "the 106
   ready" — the row has only ever lived in §1). It carries **21** open rows
   as of the 2026-09-01 third pass (22 after the second pass, 23 after the
   morning re-cut, 75 before it), **re-verified unchanged at 21 by a second
   full re-cut on 2026-09-02**. Counted individually the true total is
   **148** — this one row swapped for its 21 (128 − 1 + 21). (The figures here
   were "177, not 155" until 2026-09-01, "173" until 2026-09-02, and "153"
   against the 133 headline earlier the same day; each was arithmetic against a
   headline that has since moved, which is why the working is shown.)
3. **Six surveyors returned 487 rows; roughly 300 were `done` or `declined`,**
   and the rest deduplicated heavily — the urban milestones, the landmark
   viewshed and the vault's §26 each arrived from two or three surveys
   independently. The compression is real, not a sampling gap.
4. **Nobody ran the test suite.** "Done" for `UNIFIED_TOOL_PLAN.md` milestones
   A–E means the named crate modules and bridges exist and the commit reported
   green, not that `cargo test` passed this pass. §7 says what else is
   uncovered.
## The three that matter

If you stop reading here:

1. **Urban morphology milestone 16** — and only 16, as of 2026-09-02. Milestones
   8-15 shipped in `4ec07f5` (6 077 lines of module source, 7 251 test lines);
   the three `_um*` adapters (`_umHarbourScale`, `_umSiteProfile`,
   `_umOreBearing`) and their downstream wiring landed in `cff1edc`; and the
   **block-2 capture harness this entry named as remaining is built and its
   fixtures are golden-verified** (§2.1, which closed the row as *wrong* rather
   than stale, and found two real port bugs doing it). What remains is milestone
   16 alone — `generate()` orchestration + `hashModel` — which is blocked by
   definition on the stages it hashes and therefore sits in §3.2, not §2.
2. **The GUI/shell replacement, stages 3, 5, 6 and 7** — `00-REPLACEMENT-PLAN.md`
   still opens with a truncated-prototype blocker that was resolved the same
   day (`BUILD_ANSWERS.md` §1). Stages 1, 2 and (as of 2026-09-01, second
   pass) 4 have landed; **stages 3, 5, 6 and 7 are unblocked and unstarted**,
   and anyone reading only the plan will believe stage 5 is still blocked.
3. **The project record itself** — largely **actioned on 2026-08-31**:
   `CHANGELOG.md` is now retired (frozen and marked, not backfilled — the 51
   commits since `bcabd5a` stay in `git log`), and `STATUS.md` was rewritten
   from scratch against the working tree. What §6 records below is therefore
   history plus whatever has not yet been swept; re-verify a §6 row against the
   file it names before acting on it.
   **The "commit the two untracked documents" item this entry used to lead
   with is done** (corrected 2026-09-01): `LARGE_ITEM_RULINGS.md` and
   `cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md` are both in `HEAD` —
   `git cat-file -e HEAD:<path>` succeeds for each, and so does the same check
   on this file. They landed in `fd9de7c`, along with 235 others. Nothing on
   this list is lost by a clean checkout any more; what remains here is
   whatever §6 records that a later pass has not yet swept.

---

## 1. In flight right now

Code exists for these; they are neither startable nor blocked, they are
half-landed. Each needs finishing. (The "and, in most cases, committing" this
sentence carried until 2026-09-01 is gone — see the correction below.)

**2026-09-01: eight agents worked this section's original eight rows in
parallel; each claim was independently re-verified against the code (compiled,
tested, or parse-checked — not re-read from the report) before being recorded
here.** Four rows closed outright — Milestone F's closeout, the `statusMid`
composite, Vault §14 Compare, and route corridors/travel cost as an analysis
field — and were deleted, their status moved to `STATUS.md`. The other four
were rewritten to describe what actually remained, which in three cases was
substantially narrower than what they said that morning.

**Same-day second pass: three more agents dispatched on three of those four
residuals, independently re-verified against the code (not the reports)
before being recorded here** — `cargo test -p cartalith-spatial --lib`
148/148, `--test golden_parity_paint` 7/7, `cargo test -p cartalith-godot
--lib` 409/409 and `cargo test -p cartalith-civ --lib` 513/513 all re-run
clean after a fresh `cargo build -p cartalith-godot` (the dll was stale
against exactly the files this pass touched), `cargo check --workspace`
clean, every touched `.gd` file `--headless --check-only` clean, and both
`_railfold_probe.tscn` and `_deadwire_probe.tscn` re-run passing. **GUI
replacement stage 4 closed outright** and is deleted below, its status moved
to `STATUS.md` (RP-S4). **Economy milestone 2 narrowed further** — Godot
wiring is now real; what remains is rewritten below. **Paint brush falloff**
was not its own §1 row (it lived inside the `UNWIRED_FUNCTIONS.md` backlog
row and, individually, in §2.2) but closes outright too; both are updated
below.

**Correction (2026-09-01, later the same day): "nothing in this document is
committed" is no longer true, and every "still uncommitted" qualifier below
is history.** Commit `fd9de7c` — *"Three rounds finishing in-flight work,
then two bugs found by hand"* — landed **237 files, 90 718 insertions**,
which is all three of the passes described above plus the documents they
wrote. `git status --short` now shows one modified tracked file (an
unrelated in-flight `journey_planner_view.gd` change) and two untracked
probe scenes. The re-verification those qualifiers asked for once the tree
committed is therefore **owed now**, not later — that is the live half of
the claim, and it is what §7's "the uncommitted working tree" bullet has
been reduced to.

| Item | Owns it | Size | Where it stands / next step |
|---|---|---|---|
| **The `UNWIRED_FUNCTIONS.md` backlog** — 21 open rows (1 small · 3 medium · 17 large), down from 75, plus **2** dangerous-class entries (1 real, 1 documented non-defect kept for contrast) | `UNWIRED_FUNCTIONS.md` | large | **Re-cut again 2026-09-02** — all 21 rows re-opened at their cited symbol and independently re-verified against `cff1edc`; **0 closed, the count holds at 21**, which is itself the finding. One **new dangerous-class entry**: the Settlement diagnostics overlay's tooltip is now a *false reason* — it disables the control citing a blocker that no longer holds, the class `tools/audit_wiring.py` structurally cannot see because the binding *is* called and it is the prose that lies. The right-dock "follow the armed tool" Medium row narrowed by four newly-landed tool-driven contexts. Re-cut from scratch 2026-09-01 morning, not patched: all 75 previously-open rows re-opened at their cited symbol and independently re-verified; 52 closed that pass (17 of 17 trivial, 24 of 25 small, 13 of 17 medium). **Same-day second pass**: one more of the 18 Large rows — Paint brush falloff, the row the morning cut named highest-severity — independently re-verified as built and closed, taking Large to 17 open (of `LARGE_ITEM_RULINGS.md`'s eighteen 2026-08-31 **build** rulings, tracked individually in §2.2 below) and the dangerous class from 3 entries to 1 (the 2 genuinely-dangerous Paint entries close; 1 documented non-defect remains, kept for contrast). **Same-day third pass**: one more Medium row — "Manual road tool / `road_edges` never retained" — independently re-verified as already false (`CivData::road_edges` genuinely retains `civ_hierarchical_network_topology`'s output) and closed alongside the wider journey/route cluster in §2.3. **Committed in `fd9de7c`** (this row said "still uncommitted" until 2026-09-01); the re-verification of every closed row that the commit was the precondition for is now due |
| **Landmark M8 residual** — 30 of 50 declared kinds still ship `buildable:false` (was 35) | `LANDMARK_GENERATION_SCOPE.md` | large | **Twenty** generate today (`landmark.rs::kinds()`, each unbuilt kind carrying a `not_built:` reason). *Two counting errors in this row are corrected 2026-09-02 against the code, not against the previous report: the denominator is **50**, never 49 (`grep -c "KindSpec {"`), and the buildable count at `HEAD` was **15**, so "fourteen" was low by one.* **2026-09-02: the five way-graph kinds landed** — `market_site`, `trade_depot`, `caravan_station`, `bridge_site`, `road_junction` — taking buildable 15 → 20. The `LandmarkInputs::ways` thread a prior lane left dead is now read end to end, from the `WayGrid` through five detectors to the gdext caller; each kind was verified placing on a real `generate_terrain` world rather than by flipping its flag. `JUNCTION_MIN_WAYS` was corrected 3 → 2 in the same pass, its inherited rationale shown false. `resource_extraction_site` went buildable 2026-09-01 — it reads the three resource-potential fields (`timber`, `sulfur`, `alum`) that Mine and Quarry's own resource lists don't, through their identical already-validated detector, so it claims no cell either of them already does. The other 30 reasons were individually re-verified against the code this pass, not just re-read; six were rewritten for precision (`volcanic_feature`, `rock_formation`, `glacial_feature`, `salt_works`, `ruin`, `abandoned_settlement`) with no change to their blocked conclusion. Six still need M7's viewshed; several need §13's route load; the military family is downstream of Fort |
| **Economy milestone 2** — the food-surplus cluster | `ECONOMY_SCOPE.md` | small | **Crate-complete and Godot-wired as of 2026-09-01 second pass; the remaining gap is a UI surface, not a binding or a port.** `civ_trade_bridge.rs`'s `food_shed_rows()` builds one shared `RoadComponents`, resolves each settlement's `farmers_per_urbanite` via the `civ_ag_tech_by_key` route the manpower model already uses, and calls `civ_food_shed` once per settlement; the `#[func] civ_food_shed` reads it out; `engine_bridge.gd` and `trade_store.gd` (caching alongside `civ_trade_flows`) complete the chain, triggered by the existing "Match trade flows" button — no new UI entry point was needed for the data to compute and cache. **What remains:** no dock or window calls `TradeStore.food_shed_for(index)` — confirmed by direct search, `place_editor_window.gd:385` still reads only `navigability`; the natural landing spot is right beside it, in the Trade tab. Two small residuals disclosed but not fixed this pass: `food_shed_rows()` recomputes `lithology`/`soil` per call rather than reading a `CivData` field (an efficiency nicety for whoever next touches `lib.rs`'s `compute_civilisation`, not a correctness gap); and a stale self-claim in the crate's own docs — **half of that second residual is now closed** (2026-09-01): `roster.rs`'s module doc did assert *"nobody at the `cartalith-godot` boundary calls `civ_food_shed`"* and has been corrected against `civ_trade_bridge.rs::food_shed_rows`, while the `trade.rs` half was re-checked line by line and **no such claim is there** — the citation was wrong, not the file |

---

## 2. Committed and scheduled, not started

Nothing blocks these. They are ordered largest-first within each group, and the
groups are ordered by how much of the remaining project they represent.

### 2.1 Urban morphology — what remains

Phase 5. Milestones 8-15 are **built and committed** in `4ec07f5`. Milestone 16
(`generate()` + `hashModel`) is the only remaining milestone, and it is **not in
this section** — it is blocked by definition and sits in §3.2.

**This section is empty as of 2026-09-02, and the row it held closed as *wrong*
rather than merely stale.** The 17a caveat — golden-verify the block-2 `_um*`
adapter — recorded its blocker as *"needs a block-2 capture harness that can run
`_um*` inside the host's full civ scope; the existing harness slices block 4
only"*. That premise is disproved by a running counter-example:
`cartalith-native/tools/um_block2_capture.js` drives the unmodified reference
under Node and `crates/cartalith-civ/tests/golden_parity_urban_adapter.rs` now
holds the extracted fixtures. Mutation matrix **22/22 killed**, and an
independent verifier confirmed the fixtures are genuinely reference-extracted
rather than replayed from the Rust port.

**The pass found two real port bugs the 11 synthetic-field unit tests could not
see** — which is the argument for golden-verification, stated concretely:

- `slope_at` used `f64::hypot` where the reference uses `Math.hypot`. This is
  the exact V8-libm divergence `CLAUDE.md` warns about and `geom::js_hypot`
  exists for.
- `um_site_profile` clamped the resource-context centre where the reference does
  not.

A third defect was in the fixture itself (`background_pots` never mirrored the
harness's one iron deposit), caught before it could be committed as truth.

*The "built and uncommitted — `git log 4ec07f5..HEAD` returns nothing" note this
section carried is history: `cff1edc` landed the downstream wiring.*

> **Before executing any ruling that says "add `cartalith-urban` as a dependency
> of `cartalith-godot`": the substance is already done.** `urban_bridge.rs`
> ("the first consumer `cartalith-urban` has ever had") reaches the crate
> through `cartalith_civ::urban_adapter`, which is the layering
> `cartalith-civ/Cargo.toml:18-22` explicitly defends. The "zero consumers"
> sentence in `URBAN_MORPHOLOGY_SCOPE.md:1761-1766` is a **quotation of what
> `PARITY_AUDIT.md` §3.4 found before milestone 17a**, and the same paragraph
> describes closing it. What is missing is the generator stages above, not the
> Cargo edge; adding a direct edge would buy nothing and violate the layering.

### 2.2 The owner's 2026-08-31 Large rulings

Eighteen rows, all ruled **build** on 2026-08-31 in `LARGE_ITEM_RULINGS.md`.
**Sixteen remain not started; two have closed and are deleted from the table
below.** Paint brush falloff closed 2026-09-01, second pass, verified against
the code (`paint.rs`, `paint_bridge.rs`, the two touched `.gd` files,
`DECISIONS.md` §7k; full evidence in `UNWIRED_FUNCTIONS.md`'s Large section
and `STATUS.md`). CARTO ▸ Labels closed 2026-09-02, all three steps verified
against the code: `LabelClass` (5 variants) and `label_class` on `LabelDto`
(`crates/cartalith-godot/src/project_bridge.rs`, `#[serde(default)]` so an
older archive still opens), `labels::label_candidates`/`generate_labels`
(`crates/cartalith-civ/src/labels.rs`) over five sources kept beside the
hand-placed list, and `LABEL_TYPOGRAPHY_DEFAULTS` transcribed from
`parts.js:363` (26/2.5/.28, 18/2/.20, 13/1.5/.06, 15/1.5/.14 italic,
11/1.2/.06), drawn by `map_overlay.gd::_draw_labels`. The Water class had no
entity at all; `labels::lake_features()` fills it. **One correction on the
way out**: the ruling's own "halo and tracking do not exist in the engine
today" was half true — tracking, yes; halo was not, since
`labels::arc_label_line_width` already existed, golden-pinned; what was
missing was a halo any label class could actually *set*, which
`LabelTypography::halo_px` now is. Two of the sixteen still carry costs the
build must honour, and one more is excluded pending an owner answer (§3.1).
*`LARGE_ITEM_RULINGS.md` was untracked when this section was written; it is
tracked in `HEAD` as of `fd9de7c` — see §6.1.*

| Item | Size | Note |
|---|---|---|
| Civilisation authoring operations — five re-entrant civ stages plus a civ PARAMS group | large | **Built 2026-09-02** — `enum CivRebuild` (`Downstream`/`Routes`/`Replace`), `fn civ_rebuild` and a civ PARAMS group all landed; `recompute_civilisation` is now a one-line delegate to `civ_rebuild(CivRebuild::Downstream)`. **A live bug survives and is the real remaining work.** `CivRebuild::Routes` (reached from `civ_auto_routes`, the Generate Roads button) restores only `ways`/`road_edges`/`sea_routes`/`next_tid` from the fresh recompute and discards the rest of it (`fresh = old`), leaving `territory`, `provinces`, `province_list`, `trade_balances`, `explanations` and `dens` stale — then the function unconditionally sets `self.civ_dirty = false` and calls `self.stages.mark_recomputed(PipelineStage::Civ.id(), "civ_recomputed")` under a comment reading "everything derived from the settlement list has just been re-derived from the settlement list", which is false for this branch. Before this landed, that unconditional tail only ever ran for `Downstream`. Reachable from `civ_drop_settlement`, `civ_edit_settlement` and `civ_delete_settlement` (`crates/cartalith-godot/src/lib.rs`), all of which set `civ_dirty = true`; `self.civ_dirty` is read in exactly one place, `stale_stages()` (`:3932`), which drives SG-02's "Recompute now" indicator — so clicking Generate Roads after a hand-edit silently clears the stale indicator while territory/trade/provinces stay wrong until a full recompute. Fix: gate the unconditional tail on `mode`, or give `Routes` its own narrower staleness clear |
| CARTO ▸ Icons — generated placement plus a fourth sea-marks asset family | large | New slots, new glyphs, a coastline snap test. Answers owner question 4 |
| Label collision culling (measure-and-suppress) | large | Explicitly *not* standalone — build inside the labelling pass. Culling a set nothing generates is half a feature |
| Cut · Copy · Paste · Select all | large | Selection sets → clipboard → commands, in that order. Step one — replacing the three unrelated single-`i64` selections — is independently valuable |
| The river entity — a `get_rivers()` binding plus viewport river hit-testing | large | **Built 2026-09-02** — `get_rivers(min_order)` and `river_at(gx, gy, radius_cells, min_order)` (viewport hit-testing) both ship (`crates/cartalith-godot/src/lib.rs:5852`, `:5879`), and `right_dock.gd`'s River context is reachable (`_on_map_clicked_river` → `river_at()`). `river_entities()` reuses `split_river_polylines(&trace_river_polylines(..))`, byte-identical to `geojson_bridge.rs:96`. Two real parity bugs were found and fixed in the pass that built it: a `mag` lower clamp the reference does not have (now `js_min(1.0, (f/thresh).ln()/lmax)`, matching `Math.min(1,Math.log(f/thresh)/lmax)` with no `.max(0)`), and two `f64::hypot` → `js_hypot` corrections. **Three things survive, unfixed.** A third live `f64::hypot` at `crates/cartalith-hydrology/src/lib.rs:843`, inside `enforce_channel_descent` (`enforceChannelDescent()`, reference HTML 8725-8737, `Math.hypot` at line 8733) — measured 108 of 400 integer offset pairs differ, and it is worse than the two fixed instances because here `d` feeds `t = d/half_w` and `target = floor + (fld[i]-floor)*t*t`, written straight into the terrain field, not just a branch test. Four survived mutants in `channel_disc` (`:938`): reverting `js_hypot(gx, gy)` back to `gx.hypot(gy)` scores green because the test measuring the JS/Rust divergence never asserts the call site actually uses it, and the other three are `channel_lmax`'s `0.05` → `0.06`, `half_w`'s floor `0.5` → `0.4`, and `slope_fac`'s `5.0` → `6.0`. And a doc comment at `:903-904` cites "reference HTML lines 4534-4540" for `channel_disc`'s geometry, while the `mag` line the nearby "not floored at zero" fix note is about is reference line **4532** (`reference/Cartalith Gen1 v2.10.html:4532`), outside that cited range |
| Settlement diagnostics overlay | large | **Corrected 2026-09-02 — this cell asserted the opposite of the ruling it cites.** It read *"carries a scoped authorisation to edit `crates/cartalith-godot/Cargo.toml`"*. `LARGE_ITEM_RULINGS.md` says in bold: **"The scoped `Cargo.toml` authorisation is therefore withdrawn as unnecessary — not exercised."** No Cargo edit is needed and none should be made: `urban_bridge.rs:44` already reaches the crate via `cartalith_civ::urban_adapter`, and `cartalith-civ/Cargo.toml:22` records that indirection as **deliberate layering**, kept so `cartalith-urban`'s only dependency stays `cartalith-rng`. A direct edge buys nothing and breaks a defended decision — this is §6.9's stale-quotation defect reaching a second file. What the owner *did* choose stands: surface urban data now, and **every field with no data is dashed with its reason, never left blank**. **A second correction, same day**: the disabled control's own tooltip (`civilization_workspace.gd:1586`) still says "Blocked on urban milestones 9, 10 and 13" — stale now, since all three shipped (§2.1: milestones 8-15 in `4ec07f5`, the `_um*` adapters and their wiring in `cff1edc`). `um_site_profile`, `um_harbour_scale` and `um_ore_bearing` are ported pure functions in `crates/cartalith-civ/src/urban_adapter.rs`. The real gap is narrower: no `#[func]` in `cartalith-godot` exposes any of the three cheaply — `urban_layouts()` is the only urban binding that crosses, and it runs the reference's whole `generate()` per settlement (site, streets, blocks, lots, buildings, walls, farmland) to build a City Viewer town, not this control's three-line fact card. Next step is a lightweight `#[func]` over the already-ported pure functions, not waiting on milestones that are already built |
| Landmark funnel — a crowding parameter plus rejected-candidate coordinates and an overlay layer | large | `landmark_funnels()` returns eight scalars and no coordinates, so the dictionary grows |
| Colour management — a colour space on the render target, threaded to the texture | large | **The owner overrode the recommendation to defer.** Stated and accepted cost: every golden-parity fixture is sRGB. Must ship behind a default that leaves sRGB byte-identical, or be re-baselined deliberately and said so |
| Rebindable keyboard shortcuts — a per-context binding table in `DccSettings` with conflict detection | large | The read-only list already ships; what is missing is rebinding, applied over the menu accelerators at build time |
| Units (km / mi / nautical miles) — one formatter ahead of five hard-coded call sites | large | **Built 2026-09-02.** `DccUnits` (`godot-project/shell/dcc_units.gd`) formats km/mi/nmi (`KM_PER_MI := 1.609344`, `KM_PER_NMI := 1.852`, the owner's added nautical miles); `DccSettings.units_mode()`/`set_units_mode()` persist the choice; `menus.gd`'s Preferences ▸ Units `_todo` is now a live three-way radio; the `phone_menu.gd:84-85` promise ("units km/mi wired") is real. **What remains is narrower than "five call sites".** `menus.gd`'s own note on the row (`:1911-1933`) re-grepped its old five-site claim rather than trusting it, and found it wrong: two sites are wired (`viewport_host.gd`'s scale bar and cursor-coordinate readout, both through `DccUnits.format_adaptive()`/`format_thousands()`), two are real and still hard-coded `"%.1f km"`/`"%.0f km"` literals in `right_dock.gd` (Measure's running total/per-segment lengths, e.g. `:1586`, `:1615`, `:1627`; and Region select's km column, e.g. `:2952`), and the fifth (Sculpt's `#sBrushKm` hint) was never built at all — `world_workspace.gd`'s `_build_brush_globals()` shows brush size in px only, so there is nothing there to convert. Two real sites left, not five |
| Saved measurements + CSV, as a fifth caller-owned save slot | large | Deliberately *not* a second persistence mechanism — rides the `project_bridge.rs` slot work |
| CPU worker threads — a configurable Rayon pool at engine init plus a `#[func]` to read and set it | large | `ThreadPoolBuilder` has no call site today |
| Report an issue → a local save-diagnostic-report action | large | No endpoint required: dump generation info, missing bindings, project format version, GPU state and the last error to a file the user attaches themselves |
| The manual-icon tool (arming, rendering, persistence) | large | Scheduled separately, as `UNIFIED_TOOL_PLAN.md` milestone E's own pass, not as GUI work. The persistence half rides the caller-owned save slots |
| Region ▸ New world from selection — the orchestration around `extract_region_as_world` | large | A scoped parity pass. The resample is built and tested; allocate / clear warp fields / invalidate caches / refresh climate / empty the civ layer is new `WorldGen` state and must not be folded into GUI work |
| Decoding pack biomes and terrains | medium | **Has no row in any size table anywhere.** It surfaces only inside a trivial doc-fix row that says "a separate Medium-sized job — the doc fix must not wait on it" |

### 2.3 Civilisation, economy and journeys

| Item | Owns it | Size | Next step |
|---|---|---|---|
| Story planning **SP-3** — the settlement timeline strip (simulated history + authored vault events + journey passes) | `STORY_PLANNING_SCOPE.md` | large | No per-settlement history accessor in `timeline.rs`; `civilization_workspace.gd:1633` is the world-level strip, not a per-settlement one |
| Story planning **SP-4** — the conflict overlay in CIVIL, reading real manpower figures | `STORY_PLANNING_SCOPE.md` | large | Blocks landmark M9. Its attachment model is undecided (§4) |
| **CV-23** — historical territorial occupation over time | `STATUS.md` | large | Timeline work, not territory work |
| **VA-01** — the vault scan *index* (not the scan) | `STATUS.md` | medium | |
| The `wantCounts` / user-fixed-tier-count branch of `_civIterativeAutoWorld` | `PHASE2_SCOPE.md` m8 | small | Deferred at the time as "separate future work"; `cartalith-godot/src/lib.rs:911` records its absence |

### 2.4 Vault, project archive and save format

| Item | Owns it | Size | Next step |
|---|---|---|---|
| Vault **milestone 3** — project-scoped links (§26), inside the save rather than `user://` | `MARKDOWN_VAULT_SCOPE.md` | medium | **The doc's "blocked" status is stale.** The blocker lifted 2026-08-25 with the §7h project tree, and `cartalith-io/src/project.rs:292` already registers a `vault.json` slot. The move has not happened: `vault_store.gd:36` is still `user://markdown_vault.json` |
| Vault **milestone 2** — the map snapshot (§21, §22) at immediate/local/regional radii | `MARKDOWN_VAULT_SCOPE.md` | medium | Its own record: "blocked on nothing — `export_raster.rs` already crops" |
| Project archive remainder — project-layer panels, the `library/` and `drafts/` slots, a `preview.png` producer, foreign-entry preservation | `STATUS.md`, `SAVEFILE_COMPAT.md` §17 | medium | Nothing draws any of it; `preview.png` has a writer and no producer; foreign entries are reported rather than preserved |
| `drafts/paint.json` and `drafts/sculpt.json` are declared slots nothing writes | `UNWIRED_FUNCTIONS.md` | medium | `project_bridge.rs:1784/2059` documents and asserts the encoding; `app.gd:1851` writes only `entities/journeys.json` |
| `library/assets.json` and `library/travel.json` are declared slots nothing writes | `UNWIRED_FUNCTIONS.md` | medium | Both restore; `ops_bridge.rs:33` records the write blocker |
| Story planning **SP-1** — the `Journey` entity proper | `STORY_PLANNING_SCOPE.md` | medium | Half met, and the half that landed was built outside this document's plan: journeys persist as GDScript-owned state (`journey_planner_view.gd:3125` → `entities/journeys.json`). Not met: no `Journey` type in `cartalith-civ`, and the doc's own acceptance test still fails — `travel_bridge.rs:252` returns a hardcoded `0` |

### 2.5 Rendering, terrain appearance and export-adjacent

| Item | Owns it | Size | Next step |
|---|---|---|---|
| The stage-by-stage `WorldParams`-field audit against every stage-01…11 slider | `GUI_FEATURE_PARITY_SCOPE.md` | large | The document's own closing "honest size statement": the Generate pipeline's ~60-80 individual stage sliders, "none of which are individually scoped anywhere yet". No such audit document exists |
| §20 — the high-precision display pipeline | `TERRAIN_APPEARANCE_SCOPE.md` | medium | `render.rs` still composites into a `u8` RGB buffer (`apply_local_contrast(… rgb: &mut [u8] …)`, `:3646`) |
| Geology microtexture / dune ripples | `FUNCTIONAL_CONTRACT.md` cap. 6 | medium | On `render.rs`'s own "deliberately excludes" list (`:16`) |
| Sky-view-factor and cast-shadow fields, and their toggles | `FUNCTIONAL_CONTRACT.md` cap. 6 | medium | `render.rs:17`. AO itself has shipped, so the summary row overstates the gap |
| SDF coast/river/biome tinting and the vector river overlay | `FUNCTIONAL_CONTRACT.md` cap. 6 | medium | `render.rs:17-21` — depends on subsystems the renderer's own doc says are not built |
| GeoJSON **import** | `FUNCTIONAL_CONTRACT.md` DM-03 | medium | Export shipped 2026-08-24; import was explicitly out of scope then |
| Slippy-map tile addressing (XYZ/TMS/WMTS, a zoom ladder, retina variants) | `FUNCTIONAL_CONTRACT.md` cap. 6/9 | medium | Tile *export* exists; addressing is the remainder |
| `rockSlope` refinement and wetness darkening | `render.rs:15` | small | **Registered nowhere else.** Two reference viz features on the renderer's own exclusion list that `FUNCTIONAL_CONTRACT.md`'s absent-entirely list does not name at all |
| The ocean value-noise lattice blockiness (`seaColorCore`'s `n_low`) | `TERRAIN_APPEARANCE_SCOPE.md` | small | Found by looking during milestone 6 and deliberately not fixed; present in the `js_reference` dump too |
| **CA-05** — the icon tool has no on-canvas resize handles (labels do) | `FUNCTIONAL_CONTRACT.md` | small | `icon_bridge.rs` has none, `label_bridge.rs` does |
| Hand-lettered settlement glyphs, the fourth atlas element | `TERRAIN_APPEARANCE_SCOPE.md` | small | Ambiguous: `map_overlay.gd:48-53` draws a per-tier glyph set ported from the reference's own table, so markers exist but nothing calligraphic does. Whether the item is met depends which sense of "glyph" was meant |

### 2.6 GPU, threading and memory

| Item | Owns it | Size | Next step |
|---|---|---|---|
| `compute_stress` gather reformulation on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Deferred at milestones 5, 6 and 9 in turn. Needs a scatter→gather rewrite plus its own float-equivalence re-verification |
| Erosion's per-cell parts (thermal, stream-power) on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Feasibility table rates it "Good"; no erosion shader among the ten `.wgsl` files |
| Phase 2 per-cell affordance fields on GPU (biome, carrying capacity, resource potentials, settlement suitability) | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | "Directly comparable to climate/erosion's per-cell case" |
| Water-body priority-flood (`build_water_bodies`) on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Half tractable, half genuinely hard — the above-sea depression fill is a global priority queue, and parallel Planchon-Darboux is a research task. ~92 ms at 1024² |
| Rendering / colour synthesis on GPU (`render.rs`) | `GPU_LAYER_INTEGRATION_SCOPE.md` | medium | The feasibility table calls it "best fit, no golden-parity tension at all", and the pilot named it the natural next target. Distinct from §21's beachhead argument in §5 |
| `cartalith-godot`'s own sequential orchestration | `CPU_MULTITHREADING_SCOPE.md` | medium | Named explicitly as untouched, and as "the real ceiling left" alongside the hard-hazard functions |
| World-wrap support for the milestone 1-5 kernels (warp, heterogeneity) | `GPU_LAYER_INTEGRATION_SCOPE.md` | medium | Both stages fall back to CPU whenever `world=true` (`cartalith-engine/src/lib.rs:778`, `:894`) |
| Full `ComputeTier` capability classifier | `GPU_COMPUTE_PILOT_SCOPE.md` §4 | medium | `crates/cartalith-gpu/src` contains only `lib.rs` and `multi.rs`; grep for `ComputeTier` returns nothing |
| Performance telemetry system | `GPU_COMPUTE_PILOT_SCOPE.md` §24 | medium | Deferred until more than one workload needs monitoring; nine kernels exist now |
| GPU memory pooling across persistent fields | `GPU_COMPUTE_PILOT_SCOPE.md` §14 | medium | |
| Hardware diagnostics panel (§23) | `GPU_COMPUTE_PILOT_SCOPE.md` | medium | Partly delivered by the multi-GPU work (`performance_window.gd:78`, `menus.gd:1663`); no §23 panel as specified |
| Tiled / chunked GPU compute (§18) | `GPU_COMPUTE_PILOT_SCOPE.md` | large | Partly unblocked by the LOD pyramid. `multi.rs` ships a band split covering exactly one kernel (`gpu_warp`), 1.22-1.54× at 4096² and a loss at 2048² and below |
| Per-segment culling for one long way whose bounding box crosses the window | `MEMORY_OPTIMIZATION_SCOPE.md`, `GUI_GAP_REGISTER.md` §54 | medium | The zoom-bound overlay lever shipped (-87.5% gfx dev); this residue did not |
| Previews re-upload the whole texture — `touched_tiles`/`touched_bounds` unused | `UNWIRED_FUNCTIONS.md` | medium | Producer at `cartalith-spatial/src/pass.rs:193/199`; zero consumers |
| Integrate `QuadTree` and `TiledField` into a real caller, or retire them | `LOD_TILING_BASE_SCOPE.md` | medium | **Two of the crate's three data structures are unconsumed** three weeks and six dependent crates later — every external reference is a doc comment, and `lod_bridge.rs:54-63` argues at length why using `QuadTree` there "would be strictly worse than not using it". `DirtyTracker` does have real callers. Also leaves the deferred `tile_size` benchmark with no workload |
| Rayon across `road_dijkstra`'s independent sources — **residue only** | `GPU_LAYER_INTEGRATION_SCOPE.md` m9 | small | **Corrected 2026-09-02: the three call sites this row named are done.** It claimed *"all three call sites are still plain `.iter().map()`"*; at `cartalith-civ/src/lib.rs:5984`, `:6067` and `:7815` all three are now `rp.par_iter()`, each carrying a comment explaining that `par_iter().collect()` over the indexed source list preserves order so Prim's `best[i] < bd` tie-breaks — and therefore the goldens — stay identical. Landed with the R7/R8 memory work. **What is actually left is one site the row never named**: `build_road_network` (`:5515`) still runs `for place in places` sequentially. A second site, `:6358`, is deliberately sequential and must stay so — its own comment records that the running per-cell min is *meant* to compare across capitals in order |
| Per-pipeline caching across repeated `generate_terrain` calls | `GPU_LAYER_INTEGRATION_SCOPE.md` | small | Milestone 8 shares the device within one call only |
| Average the GPU-vs-CPU benchmark over multiple runs | `GPU_LAYER_INTEGRATION_SCOPE.md` | small | The 2048² ratio moved 1.19×→0.98× with no code change; single-run variance is currently indistinguishable from a result |
| Investigate the `gpu_height` throughput drop from 1024² (8.13×) to 2048² (4.84×) | `GPU_LAYER_INTEGRATION_SCOPE.md` | small | A plausible cause (memory-bandwidth-bound at 9 buffers) is stated and untested |
| Decide `gpu_compute_height`'s status — built, verified, and **never called** | `GPU_LAYER_INTEGRATION_SCOPE.md` m3 | small | There is no `if p.use_gpu` branch around `compute_height`. Resistance's non-wiring is a documented decision (0.38×); height's is not explained anywhere. Either a real gap or an undocumented decision — the docs do not say which |
| Hardware capability cache (§30) | `GPU_COMPUTE_PILOT_SCOPE.md` | small | Deferred as "nothing expensive enough to cache"; milestone 8 later measured the adapter/device handshake at ~1.3-1.4 s, so the premise weakened |
| Delete the three `_peakaudit_*` probes | `MEMORY_OPTIMIZATION_SCOPE.md` | small | None called, none a test, all three named for deletion when the audit closes |

### 2.7 Android and on-device verification

No Android pass has run since 2026-08-25. All six items below are live.

| Item | Owns it | Size | Next step |
|---|---|---|---|
| Six features never driven on device since the 2026-08-24 USB disconnect — paint visibility, save/undo, the debug views, GeoJSON export, hand-drawn ways, civ-recompute | `ANDROID_BUILD_SCOPE.md` | medium | Recorded as *unverified on device*, not as verified. The 2026-08-25 pass drove a different list and did not pick these up |
| **PH-16** — the Journey Planner centre panel is 1 434 px (61% of the phone screen) of nothing, with the map hidden behind it | `STATUS.md` | medium | "The worst thing on the phone." Registered, not fixed |
| **PH-15** and the phone residue — scroll flick activates the row it starts on; label clipping without ellipsis; DS-12 prints the class twice; a stuck hover pill; a stock-Godot focused tab | `STATUS.md` | medium | The Memory row under-reporting, listed with these, has since been fixed |
| The default 2048×1311 new world costs ~878 MB peak on the phone | `STATUS.md` | medium | The "no progress indication" half is stale — a staged 10-stage readout ships off `cartalith-engine::progress`. The memory cost stands |
| Prove `push_warning` reaches Android's `logcat` (a positive control) | `ANDROID_BUILD_SCOPE.md` | small | Owed by two consecutive passes; the second explicitly declined it, noting the alternative "rests on an argument, not a measurement" |
| The left-panel sheet retains its scroll offset across close/reopen and will not scroll back up | `ANDROID_BUILD_SCOPE.md` | small | Six swipe attempts at three x positions failed. Not investigated |
| Exercise **R1**'s Godot-side hunk inside a running Godot process on the handset | `MEMORY_OPTIMIZATION_SCOPE.md` | small | The case for R1 is four arguments, not a screenshot |
| Bottom-docked controls do not ride above the IME | `UNWIRED_FUNCTIONS.md` | small | Zero `get_virtual_keyboard_height` hits in `shell/`, re-verified 2026-08-31 |
| The Android debug `.so` residue — 156 MB, 207 MB APK | `STATUS.md` | small | Reduced from 400 MB; still not the 18 MB a full strip gives. See §5 for why it stays |

### 2.8 Discipline debts

Small, cheap, and each one the kind of thing that silently invalidates a later
measurement.

| Item | Owns it | Size | Next step |
|---|---|---|---|
| **21 menu commands still unavailable**, each carrying a stated reason | `STATUS.md` | medium | 356 total entries, 21 unavailable (was 245 / 24) |
| Copy in the two upstream owner notes the research briefs cross-reference (`Gravity influence.md`, `Weather Model.md`) | `PROVENANCE.md` | small | They live only in the upstream `Cartalith_RC` / `Cartalith-Gen1` repositories. The alternative the doc itself allows is keeping the paragraph so the dangling reference is a known one |
| Five "left undetermined" questions from the unwired re-cut — light-theme inertness of the CARTO panels, the phone measure strip / label bar / way card, the 44 vs 48 dp target sweep, whether `sculpt_stroke_point` can reject an appended point, landscape composition beyond the sheet handle, and whether any `_todo` reason cites a `PARITY_AUDIT.md` section number that has moved | `UNWIRED_FUNCTIONS.md` | small | Three of the six need a handset or a light-theme capture, not a read |

---

## 3. Blocked, with the blocker named

A row is here only if something concrete stops it. Where the blocker is an
owner answer, the question itself is in §4.

### 3.1 Blocked on an owner decision

| Item | Owns it | Size | Blocker |
|---|---|---|---|
| **Landmark M7 — viewshed / line-of-sight** | `LANDMARK_GENERATION_SCOPE.md` | large | Open question 5: the accuracy/cost budget (observer count, radius cap, grid resolution). §5 states the complexity honestly and deliberately does not choose a number. **Gates six of the 49 landmark kinds**, and `needs_viewshed` already ships as a declared flag with no implementation behind it. 8192² is 67 M cells, so it is not a naive all-pairs proposition |
| **IN-13 — trade flows**: who trades with whom (bipartite match, network flow), prices, tariffs, caravans as entities | `STATUS.md` | large | Needs a decision about what a currency is in this world. `TradeBalance` names *what*, never *who* |
| **DS-03 — the tablet interior is not a scaled desktop** | `GUI_GAP_REGISTER.md` §57 | large | A content decision — *which controls leave the tablet* — before a styling one. ~55 paired elements measure ×1.00 to ×2.06 with no centre, and roughly 30% of the desktop's content is deleted. Also architecturally blocked: `DccTheme.TABLET`'s key space is exhausted, one desktop integer mapping to two tablet figures in at least five verified places, and §57 refuted the obvious placement for a role-keyed resolver |
| **The right dock does not follow the armed tool** (`rdExtraMode` ladder, nine contexts) | `UNWIRED_FUNCTIONS.md` | medium | Owner question 1. Deliberately excluded from the 2026-08-31 build in flight. Merging naively makes the dock flip away from a selected settlement the moment a tool arms |
| Resolution-range policy — 4096 needs 2.41 GiB and 8192 needs 9.65 GiB, so 2048×1311 is the last Android-viable preset | `MEMORY_OPTIMIZATION_SCOPE.md` §8 | small | A product decision. The doc twice refuses to change `RESOLUTION_PRESETS` unilaterally, and now has the numbers to support whichever way it goes |
| Save compression — the byte-plane shuffle (27-36% smaller, writes faster) | `STATUS.md` | medium | Needs a `format_version` bump and a fail-loud marker; **it ends `SAVEFILE_COMPAT.md` §8's bare-dump promise** |
| Save compression — quantising saved rasters to `u16` | `STATUS.md` | medium | Lossy. `PARITY_TESTING.md` and `DECISIONS.md` §7a bar it without a ruling |
| **CA-19** — a writable biome colour table | `STATUS.md`, `PARITY_AUDIT.md` | medium | Buildable today, but **costs a golden re-baseline** that `DECISIONS.md` §7a protects |
| **CV-24 / ED-02** — the year scrubber as program scope; the undo-history panel | `STATUS.md` | medium | Both want a decision, not wiring. `TIMELINE_SCOPE.md` §4's standing instruction is to design the panel first rather than guess the region |
| Delete the seven uncalled `cartalith-gpu` public functions (~70 lines) | `GPU_LAYER_INTEGRATION_SCOPE.md` | small | The ponytail pass declined to delete public API on its own authority. Verified today: `heterogeneity_grid_gpu`, `gauss_blur_grid_gpu`, `assign_plates_grid_gpu`, `flow_accumulation_gpu_with`, `gpu_resistance_grid_cpu` and `init_gpu_f64` have zero callers; `warp_grid_gpu`'s only external hit is a doc comment. `init_gpu_f64` is separately owner question 8 |
| The flaky GPU determinism test `generate_terrain_gpu_path_is_deterministic_and_valid` | `STATUS.md` F1 | small | Fails ~1 run in 3 under full-workspace parallel load, by ~1 ulp. The decision is whether an `assert_eq!` on a whole f32 field is the right bar for a path §7a holds only to principled equivalence |
| Military manpower **finding 2** — standing armies land at Imperial Rome's ratio, not the era table's standing column | `MILITARY_MANPOWER_SCOPE.md` | medium | Correcting it means recalibrating outputs currently validated against the owner's worked example. Reported, not tuned |
| Stop shipping the ~218 `_*_probe` / `_*_shot` development scenes inside the APK | `ANDROID_BUILD_SCOPE.md` | small | The owner's call — the pass ran under a standing instruction not to touch `export_presets.cfg` or `Cargo.toml`. `exclude_filter` does not mention them |
| Shrink `STATUS.md` | `STATUS.md` own header | medium | An editorial decision for the owner, declined twice by audit passes as correctly out of their remit. Still not made — but **the size that motivated it is gone**: this cell said "8 122 lines with four lines over 15 000 characters" until 2026-09-01, contradicting this document's own header three paragraphs in. `wc -l` gives **1 445** today (1 157 at the 2026-08-31 rewrite, so it is growing again). The decision is open; the emergency is not |

### 3.2 Blocked on other work in this list

| Item | Owns it | Size | Blocker |
|---|---|---|---|
| **Landmark M9** — cultural interpretation and temporal state | `LANDMARK_GENERATION_SCOPE.md` | large | `STORY_PLANNING_SCOPE.md` **SP-4**, which is not started and whose attachment model is undecided, plus open questions 1-2. **Two documents' largest remaining milestones sit behind one unasked question** |
| Urban **milestone 16** — `generate()` orchestration + `hashModel`, the whole-subsystem golden | `URBAN_MORPHOLOGY_SCOPE.md` | medium | Blocked by definition on milestones 8-15: `hashModel` can only be compared once every stage it hashes exists. Milestone 12 already had to dump state directly for want of it |
| Urban **milestone 17**'s remaining five `_um*` — `_umWallSpec`, `_umInferWalls`, `_umHarbourScale`, `_umSiteProfile`, `_umOreBearing` | `URBAN_MORPHOLOGY_SCOPE.md` | medium | Each one's only consumer is milestone 9, 10, 13 or 15. Two data gaps compound it: settlements carry no `specialisation` and no `traits`, so the honest fallbacks are `economy: null` / `fortified: false` |
| Story planning **SP-2** — journey progression over the cursor | `STORY_PLANNING_SCOPE.md` | large | §6's regenerate-semantics question explicitly gates it: whether a journey's route polyline is invalidated, re-snapped, or kept with a staleness mark "needs a ruling before SP-2 ships". The grain question (real date vs fraction of a year) is also unresolved |
| Story planning **SP-5** — the planning aid, joined up | `STORY_PLANNING_SCOPE.md` | medium | Deliberately last: worth nothing until at least two of SP-1…SP-4 exist. Only SP-1 is partly real |
| Saved measurements + CSV | `LARGE_ITEM_RULINGS.md` | large | Rides the `project_bridge.rs` caller-owned slot read/write work being built for the other four slots |
| **CA-03 / CA-04 / RD-10** — per-layer blend mode and layer reorder | `FUNCTIONAL_CONTRACT.md`, `GUI_FEATURE_PARITY_SCOPE.md` 7b | large | `render.rs` bakes the three overlay categories into one per-pixel pass. Needs the single colour pass to become separable outputs — an architecture change, not a control. Opacity shipped; this did not, and the precondition has not moved |
| Religion-diffusion screens | `GUI_GAP_REGISTER.md` §57 | large | `cartalith-civ::belief` and its Godot bridge do not exist. `get_settlements()` emits no religion field and no adherent counts. Recorded lesson: the surfaces cannot be designed before the engine half exists |
| The **GUI_GAP_REGISTER §3** A/B/C/D open/closed split, never re-derived | `GUI_GAP_REGISTER.md` | medium | Recovering each dropped class letter is "a judgment per row, not arithmetic" — declined by three consecutive audit passes. The register cannot currently say how many of its 300 IDs are open. `UNWIRED_FUNCTIONS.md` is the live successor; read the register as history |

### 3.3 Blocked on hardware, or on a design that does not exist

| Item | Owns it | Size | Blocker |
|---|---|---|---|
| The phone overflow menu — re-present the seven desktop menus as a touch-sized drill-down | `ANDROID_BUILD_SCOPE.md` §5 | large | A mobile menu design is being produced separately; the pass was instructed to diagnose only. Four compounding causes including 15 hover-opened submenus and ~12 physical-px rows |
| **BUILD_ANSWERS §3** — the Data-manager window and 13 of 24 asset families are absent from the new Environment prototype | `design/…/BUILD_ANSWERS.md` | medium | Awaiting a decision: build them against the older `Cartalith DCC Shell.dc.html` canvas, or have the design project add the window to the Environment file. **A standing offer to supply it exists.** Easy to lose, because it sits in an answers file rather than in the plan |
| **BUILD_ANSWERS §4** — phone generation-failure and storage-full states are undesigned; content descriptions and dynamic type are absent; the 48 dp target sweep is partly done | `design/…/BUILD_ANSWERS.md` | small | The design does not exist; the design project has offered to produce it on request. Feeds stage 6 |
| **DS-13** — the phone viewport control column (zoom/pan/navpad) redesign | `GUI_GAP_REGISTER.md` §57 | medium | Three registered, nothing built — and four high-severity refutations of the proposed design, including three colour equalities that were arithmetically false |
| Observe the §13 phone **landscape** composition on the device | `ANDROID_BUILD_SCOPE.md` | small | `adb` cannot force it: Godot's `orientation="sensor"` sets `SCREEN_ORIENTATION_SENSOR`, which follows the accelerometer and overrides `settings put system user_rotation`. **Needs the owner to physically rotate the handset.** Every measurement in §50 is portrait |
| **§47** — hi-DPI blur confirmed only to `_phone_scale` 2.748, not the owner's 3.664 | `STATUS.md` | small | Needs the OnePlus 12, which this project has not had on the bench. §47 is confirmed *in kind* and not at that scale |
| **GPU §21** — thermal / mobile-adaptive GPU scheduling | `GPU_COMPUTE_PILOT_SCOPE.md` | medium | **No Android GPU compute path exists to adapt.** The handset runs the CPU pipeline entirely; the device passes treat "zero `wgpu` lines in logcat" as a *pass* condition. Both `project.godot` renderer keys are `gl_compatibility` |
| Owner question 3 — should the WORLD left-dock A/B switch come back? | `UNWIRED_FUNCTIONS.md` | small | Doubly blocked: an owner call, *and* the captions and gate (`ldSwitch`/`ldSwA`/`ldSwB`) are in the truncated tail of `02-rail-and-domains.md` §8, so there is no label to build it with |
| The 3D research's three commissioned questions (`gl_compatibility` rationale; wgpu/Godot GPU coexistence; what a raised device floor buys) | `3D_TERRAIN_RENDER_RESEARCH.md` | medium | Parked with the 3D viewport. Question 2 is named the highest-value unanswered question and gates `RenderingDevice`, compute shaders and GPU-driven culling. Resuming is cheap — the research is complete at 1 530 lines |
| Vault **milestone 4** — device pass verifying the Android SAF provider (folder picker, persisted grant, revocation) | `MARKDOWN_VAULT_SCOPE.md` | large | Needs a real Android device |
| Whether the root `Cartalith Gen1 v2.11.html` (now also mirrored into `reference/`) is `Cartalith_RC`'s live head, or a copy that repository has since moved past | `CLAUDE.md`, `REFERENCE_DRIFT_v2.10_to_v2.11.md` | small | `Cartalith_RC` is not present on this machine and is not a remote of this repository, so it cannot be checked without opening it — do not assert either way without doing so. Left open by the 2026-09-02 re-freeze pass, which froze `reference/` to v2.11 without resolving it |

---

## 4. Open decisions the owner still owes

Not work. Each of these has to be answered before the row it gates becomes a
task. Ordered by how much they unblock.

| # | Question | Owns it | Gates |
|---|---|---|---|
| 1 | **What is a conflict attached to** — free geometry, or a reference to a settlement/province? | `STORY_PLANNING_SCOPE.md` §6 Q2 | SP-4, and through it landmark M9. The single highest-leverage unanswered question in the project |
| 2 | **The viewshed cost budget** — observer count, radius cap, grid resolution | `LANDMARK_GENERATION_SCOPE.md` OQ 5 | Landmark M7 and six landmark kinds |
| 3 | **Regenerate semantics for a journey's route polyline** — invalidated, re-snapped, or kept with a staleness mark? | `STORY_PLANNING_SCOPE.md` §6 Q3 | SP-2, explicitly |
| 4 | **Does the landmark set live in the save tree (`entities/landmarks.json`) or regenerate on load?** | `LANDMARK_GENERATION_SCOPE.md` OQ 1 | The record's shape — research §25's state transitions cannot be recomputed. Storage is in memory today and the save format is untouched |
| 5 | **Does a landmark become a `cartalith_vault::EntityKind`?** | `LANDMARK_GENERATION_SCOPE.md` OQ 2 | A `Landmark` template exists in `design/vault-templates/` and is recognised by `template.rs:155`, but `links.rs:81-84` has no variant. `MARKDOWN_VAULT_SCOPE.md` §4's identity-strength table needs a new row first |
| 6 | **Does `DECISIONS.md` §7a/§7d's parity contract apply to landmarks at all?** | `LANDMARK_GENERATION_SCOPE.md` OQ 3 | `FUNCTION_INDEX.md` returns nothing for "landmark", so there is nothing to match. `landmark.rs` was built assuming divergence-by-addition; no ruling is recorded |
| 7 | **Does `rdExtraMode()` replace the right dock's ten selection contexts, or sit beside them?** | `UNWIRED_FUNCTIONS.md` Q1 | The `rdExtraMode` medium row (§3.1) |
| 8 | **What should `statusMid`'s `repaint NN ms` measure** — frame time, texture-upload time, or `_refresh_map()` wall time? | `UNWIRED_FUNCTIONS.md` Q2 | One field of a composite that otherwise shipped |
| 9 | **Should a committed sculpt stamp re-evaluate when sea level moves?** | `UNWIRED_FUNCTIONS.md` Q6 | Also a parity question: the reference re-reads `state.seaLevel` live, this port snapshots. `sculpt.rs:1076 with_sea_level` exists and nothing calls it |
| 10 | **Should generated worlds be denser relative to carrying capacity?** (`civ_settlement_population`'s surplus fractions) | `ECONOMY_SCOPE.md`; also `MILITARY_MANPOWER_SCOPE.md` finding 3 | Raised independently by two documents. `ecological_factor` saturates at its 2.0 ceiling on 5 of 6 real factions, which is the symptom |
| 11 | **Is `init_gpu_f64` kept or deleted?** | `UNWIRED_FUNCTIONS.md` Q8 | Part of the seven-dead-functions row. The pilot recorded no disposition for its own residue — `GPU_COMPUTE_PILOT_SCOPE.md` has no `f64`/`SHADER_F64` mention at all |
| 12 | **Is the phone app bar's ☰ / ▤ pair now stale?** | `UNWIRED_FUNCTIONS.md` Q9 | Scopes stage 3 of the shell rebuild. The 2026-08-31 Android canvas's app bar is [world pill] · ⌕ · ⋮ |
| 13 | **`--good` and `--accH`** — declared and never used | `UNWIRED_FUNCTIONS.md` Q10 | The prototype records both as declared-and-never-used itself, so a shell with no consumer may be fidelity rather than a gap |
| 14 | Where do landmarks live in the crate graph? | `LANDMARK_GENERATION_SCOPE.md` OQ 4 | **Answered de facto, never formally**: the code landed in `cartalith-civ/src/landmark.rs` and `cartalith-terrain/src/analysis.rs` rather than a new crate. §4.4 called it "a real architectural fork, not just a filing question" and the fork is not recorded as decided |
| 15 | How does a generated landmark relate to the manual icon tool (`annotations/icons.json`)? | `LANDMARK_GENERATION_SCOPE.md` OQ 6 | Affects the save format, the renderer, and M6's spacing inputs. Partially touched — 49 glyphs were drawn — but a rendering vocabulary is not the decision |
| 16 | Should the 16K/32K export be un-shelved? | `EXPORT_SCOPE.md` | See §5 for the four things an un-shelve costs |
| 17 | Store distribution and signing (`DECISIONS.md` §6) | `ROADMAP.md` | "Things the architecture permits and nobody has committed to." Not work until someone commits. `export_presets.cfg` has only Windows Desktop and Android |
| 18 | A WASM target sharing `cartalith-engine` (`DECISIONS.md` §2) | `ROADMAP.md` | Same status. Zero `wasm32` hits in any `Cargo.toml` |
| 19 | Should `STATUS.md` be shrunk, and how? | `STATUS.md` header | Listed in §3.1 as a blocker too, because it taxes every session that follows `CLAUDE.md` literally |

**One question left this section on 2026-09-01 and is not renumbered into
it.** *"Which unit systems are offered? (**PR-15**)"* — formerly #16, owned by
`GUI_GAP_REGISTER.md` §10 — is **answered**: `LARGE_ITEM_RULINGS.md` rules it
*"Build, and add nautical miles"*, naming km / mi / nautical miles outright.
An answered question is not an open decision, so it is deleted here rather
than carried with a note, per this document's own no-"done"-column rule. The
work it gated is the Units row in §2.2, which now carries the residual
`_todo` at `menus.gd`. Questions 17-20 shifted up by one; a citation of
"question 16" written before this date means Units, not the export shelf.

---

## 5. Declined and shelved, and why

Kept so nobody re-proposes them. Nothing here is a gap.

**Owner-parked or shelved, reversible by a word**

- **The 3D viewport, and all 3D work.** Parked 2026-08-31, the same day the
  research landed: *"On part of the 3D let's keep that for later at this
  moment, it will be implemented later on."* `DECISIONS.md` §4 continues to
  stand. The research is complete and parked at 1 530 lines. The two menu rows
  and the phone 2D/3D FAB stay drawn and disclosed; the FAB's toast becomes
  honest only when the Small relief-exaggeration row lands. This is why
  `ROADMAP.md` Phase 3's "3D drape" is **not** listed as outstanding above.
- **16K/32K single-image export, E1-E5.** Shelved 2026-08-25 at the owner's
  request. Un-shelving costs four things in order: (a) lifting the shelf;
  (b) reversing the documented "rendered once, tiled and single are the same
  pixels" decision in `export_raster.rs` — a `DECISIONS.md`-grade change whose
  "same pixels, no seams" guarantee **has already been earned** by E1's
  byte-identity tests, which were built, proven at five band heights, and then
  deliberately reverted; (c) a ruling on the codec/size trade, since §6.3 is
  blunt that at 32K no codec makes this small (500 MB - 1 GB lossless, the one
  lossy option ruled out by AGPL licensing on `jxl-encoder`, WebP eliminated
  at 16 383 px); and (d) accepting that E4 — overlays into a `SubViewport`
  across frames under a synthetic per-band camera — is new work with **no
  reference behaviour to port against**, because the reference's own bake
  draws terrain and nothing else.

**Declined on measurement or architecture**

- **§21, the GPU rendering path for appearance.** Milestone 6 measured the case
  at ~5% of a generate+render and declined to start; a second renderer would
  diverge from the golden-verified one under `DECISIONS.md` §7c. Treat this as
  a decision to confirm, not a task to schedule.
- **Tile-scoped (incremental) recomputation of hydrology/climate/civ stages.**
  A separate re-architecture, only worth taking if lazy whole-recompute proves
  too slow — and it has not: 76.5 ms @512², 188.9 ms @2048², 18.8× cheaper
  than the generation it replaces.
- **Per-stage re-execution of the ten-stage pipeline.** The capability exists in
  neither this engine nor the reference app; verified by Playwright against the
  real reference (WW-11).
- **Orogeny graph-tracing and Dijkstra/MST road networks on GPU.** The first
  needs genuine algorithmic redesign; the second was confirmed as
  should-stay-on-CPU because predecessor ties are settle-order-dependent and
  roads would visibly move.
- **The hard-hazard sequential functions** — CPU flow accumulation,
  priority-flood, `compute_stress`'s scatter, `erode_thermal`'s delta scatter,
  `droplet_kernel`, the stream-power main loop, orogeny tracing,
  `road_dijkstra`'s traversal. Confirmed unsafe per function with the hazard
  named, not assumed. **Three separate documents defer the same four
  algorithms for the identical reason**; they are the shared ceiling on both
  the GPU and the Rayon efforts, and none has an owner.
- **Landmark M1's consolidation** — consolidate the three duplicate slope/TPI/curvature copies onto the canonical field. M1's own "Done when" demands `build_ao`'s output be proven **bit-identical** before and after refactoring. `DECISIONS.md` §7a protects the rendered output, and `cartalith-terrain/src/analysis.rs` module doc explains the reasoning: refactoring `build_ao` would put a golden-protected render path at risk to share four lines of box blur. Declined rather than scheduled.
- **R6** — stop reserving grid-sized capacity in the two heaps (42.96 + 32.2 MiB). Declined as low-value, with its own note already recording that the saving is small on Android.
- **A bounded thread pool** — declined as "this port has no interactive editing
  mid-generation to protect against". The Sculpt/paint tool system has since
  landed, so the premise is worth re-checking.
- **`ComputeBackend` trait abstraction** — "premature with one kernel". Nine
  kernels exist now, so the stated reason has partly expired.
- **Overlay lever 1** (collapse the dash loop into one `draw_multiline`) —
  measured a no-op to the digit, verified pixel-identical, reverted rather than
  shipped. `_dashbatch_probe` is kept as the reason not to retry.
- **hi-DPI mitigations** — font oversampling 1 152 KB, icon re-rasterisation
  424 KB. "There is no trade-off here to make."
- **The Android `.so` at 156-171 MB** with `debug = "line-tables-only"` rather
  than the 18 MB a full strip gives. If size ever becomes the binding
  constraint, drop `debug` and set `strip = "debuginfo"` together.

**Declined because the engine has no counterpart**

- **AS-14** user-picked "active variant" (variant choice is weighted and seeded);
  **AS-15** per-slot Anchor (`Anchor` is a *family* property); **AS-16** the
  24-family rail (owner decision, disclosed in the window's header).
- **Vault §35 criteria 6-7** — POIs and "regions" as entity kinds. Recorded as
  unsatisfiable rather than faked.
- **Vault §11 TextRange/MarkdownBlock selections** — a correctness decision: a
  byte offset stops pointing at the right paragraph. **§19** continent field on
  a settlement's export block — `civ_continents` deliberately keeps no raster
  (268 MB at the 8192² ceiling). **Two-way sync, `obsidian://` links, the Data
  manager vault block** — §33's explicit V1 non-goal. **Setext headings** — ATX
  only, because that is what all four of the owner's real templates use.
  **Feeding the imported note copy back into world state** — §36 forbids a
  second source of truth.
- **Journey Planner**: six DOM render functions (Godot's job), `_jpLayovers` and
  `_jpSettlements` as Rust functions, and the `JpParty` widening — re-examined
  and deliberately declined, because `TRAVEL_LIBRARY_SPEC.md` §3.1 carries no
  seasonal-physiology or desert fields.
- **Military**: per-settlement garrisons (a placement rule nothing implies),
  campaigns / unit movement / combat (each needs a clock, a map objective and
  an opposed force), change over time, and leaving `power.military` as the
  reference's golden-verified composite.
- **Urban**: `_umDrawLayout`/`_umDrawLayoutPreview`/`_umLayoutAlpha` and the
  block-1 LOD hook (canvas rendering is Godot's job — built as GDScript
  instead); the `_umModelCache` LRU and one-per-frame `setTimeout` queue (a
  workaround for the browser's single thread; this port has real threads); the
  17 removed culture profiles (only `medieval` and `venus` are live);
  `buildGridStreets` and the palimpsest mode (removed upstream, no live caller).
- **Asset library authoring-side conveniences** the reference itself calls
  authoring-only.
- **`state.erosion` is not written to saves** — only 2 of 16 keys are modelled
  by `loadZip()`, so it is deliberately not written rather than written
  partially.
- **Warfare, Narrative/Scenario, year-by-year historical playback, and a
  coordinate system / projection.** The first three need a product decision
  nobody has made; projection is declined outright because Cartalith's world is
  a flat, non-georeferenced procedural grid with no real-world CRS.
- **`DECISIONS.md` §7b's simulated historical territorial expansion** —
  considered and deferred, not rejected: revisit only if the static weighted-
  Voronoi result feels wrong once actually seen, not preemptively.
- **The Data manager's five silent navigation rows** — re-checked and left
  alone twice; each opens a pane that explains itself.

---

## 6. Contradictions in the project record

These are defects in the record, not milestones, and they are worth more than
any single row above: each one costs a future session either re-derived work or
a wrong plan. They are ordered by what they cost.

### 6.1 Two documents exist only in the working tree — ~~open~~ **closed 2026-09-01**

**This defect is fixed, and the paragraph below is kept only because §6 is a
record of what the project record got wrong.** `LARGE_ITEM_RULINGS.md` and
`cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md` are both in `HEAD` —
verified with `git cat-file -e HEAD:<path>` on each, not by reading a
document — having landed in `fd9de7c` with 235 other files. No clean checkout
loses either. The stale wording survived in three places at once (here, "The
three that matter" #3, and §2.2's footnote), which is itself the pattern this
section exists to name: one fact asserted in three files ages in three
places.

*What it said, for the record:* `git status` → `?? LARGE_ITEM_RULINGS.md` and
`?? cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md`. The first carries the
owner's rulings on all eighteen Large rows, **including two scoped
authorisations that override standing rules** — editing
`crates/cartalith-godot/Cargo.toml`, and diverging from the reference on paint
falloff — plus the accepted cost on colour management. The second is 1 530
lines of commissioned research. A clean checkout loses both. *This is the
cheapest thing on this page to fix.*

### 6.2 `CHANGELOG.md` is five days behind the repository

Last heading: `## 2026-08-26 (12)`. A grep for `2026-08-3` returns **zero
matches**, while `git log` shows eleven commits dated 2026-08-30/31. Missing
entirely: landmark generation end to end (`a6feec3`), the 49 landmark glyphs,
`DESIGN_HANDOFF.md`, the prototype import, the GUI replacement spec, the
INFRA→CIVIL / RENDER→CARTO ruling, stages 1-2 (`c03b43c`), and the unwired
re-cut (`5543ef3`). A grep for "landmark" across all 29 534 lines returns two
unrelated hits. `CLAUDE.md` tells every session this file records what actually
landed; for the last week it does not, and anyone reconstructing state from it
will conclude the GUI replacement has not begun.

### 6.3 `STATUS.md` contradicts the code it summarises, on the same day

Its newest section header (`:133`, dated 2026-08-30) reads *"Landmark
generation catalogued, **nothing built** — no viewshed, no Poisson-disc…"*.
Landmark generation shipped that same day:
`cartalith-civ/src/landmark.rs` is **3 730 lines** with 49 kind specs, a
`Landmark` struct carrying `causal_chain`, and `pub fn generate`; plus
`landmark_bridge.rs`, 49 glyphs, `_landmark_probe.gd`, and a CIVIL ▸ Landmarks
panel. Thirteen of the 49 kinds generate today. The genuinely-absent parts are
narrower than the header: viewshed, persistence, and the vault entity kind.

Three further `STATUS.md` defects:

- Its `Last updated:` line (`:330`) says **2026-08-25** while sections dated
  2026-08-30 sit above it, and there is **no 2026-08-31 section at all** — so
  the largest structural change since the shell was written (the five→three
  rail fold) is invisible in the authoritative living status.
- It reports the unwired backlog as *"44 → 21 open"*; the 2026-08-31 re-cut
  says **77**. A session trusting `CLAUDE.md`'s "authoritative status is
  `STATUS.md`" gets a number 56 rows low.
- It has no section for `MILITARY_MANPOWER_SCOPE.md` (built 2026-08-25) or for
  `ECONOMY_SCOPE.md` at all.

### 6.4 Six scope documents are stale in the same direction

Every one understates progress. The pattern is consistent and worth naming: the
gap registers get re-verified, the scope documents do not.

| Document | What it still says | What is true |
|---|---|---|
| `TERRAIN_APPEARANCE_SCOPE.md:1113` "Still open" | Nine items, incl. "the GUI editing panel (all UI work on hold)", "§17 colour vibrancy", and milestone 1's elevation-ramp question | **Three of nine are stale.** The UI hold lifted 2026-08-18 and the panel shipped as `render_workspace.gd`; §17 shipped as the "Colour grade" and "Grade field influence" groups; the ramp question is answered in code both ways (`lib.rs:1873`, `list_ramp_presets`). §19 is half-done rather than open |
| `ASSET_LIBRARY_SCOPE.md:1068`, `:1179` | AS-07, AS-12 and AS-17 are "still honestly a gap" | All three closed **2026-08-23** (`as_set_item_transform` at `lib.rs:11060`, `as_collections`, `SliceGrid::move_line`) and are recorded closed in `GUI_GAP_REGISTER.md` |
| `UNIFIED_TOOL_PLAN.md:2265`, `:2268`; `STRANDED_TOOLS.md:34` | "all UI work is on hold"; milestone F is the only work left and is unwired | **Resolved 2026-09-01, not just historical.** The hold-lifted claim itself was already stale — `CLAUDE.md` records having already corrected this exact class of error once — and the second half is now fixed rather than merely diagnosed: `UNIFIED_TOOL_PLAN.md` carries a verified "Milestone F as built" section, and `STRANDED_TOOLS.md`'s "44 methods... not one wired" claim is annotated false in place. See `STATUS.md`'s Tool system row |
| `MARKDOWN_VAULT_SCOPE.md:247`, `:259`, `:439` | Milestone 3 "blocked"; milestone 6 "engine half done", "the UI half is not built" | Milestone 3's blocker lifted 2026-08-25 (it is not-started, not blocked). Milestone 6's UI landed 2026-08-26 — `vault_window.gd` 641 → 1 140 lines, with `_build_search` and the "confirm always" checkbox |
| `GUI_FEATURE_PARITY_SCOPE.md` status box | Twelve items open, incl. heightmap import, GeoJSON export, the appearance GUI, the faction roster, the Journey Planner GUI, the Asset Library UI, the LOD viewport, light theme, opacity, measurement, quality tiers, PopupMenu theming | **Fully discharged as of 2026-09-01.** Every milestone item now exists in code — the one survivor, route corridors/travel cost as a selectable analysis field, shipped this pass (`sample_bridge.rs`'s `corridor`/`travel_cost` ids, tested). Only the never-attempted per-stage slider audit the document names in its own closing paragraph remains, and that was never a milestone item. The document should be closed out |
| `DCC_SHELL_SCOPE.md` | "Milestone 2 and milestone 3+ remain not yet dispatched"; "still deferred: light theme, responsive breakpoints, all tool functionality" | Both dispatched and completed the same day the sentence was written; all three deferrals closed. It is a 2026-08-18 snapshot wearing a milestone-plan title |
| `design/…/00-REPLACEMENT-PLAN.md` §0 | Opens with "the desktop prototype we received is truncated"; stages 5 and parts of 2 blocked | The split re-export landed the same day (Environment 239 712 B + `cartalith-dcc-parts.js`), `BUILD_ANSWERS.md` §1 confirms everything is present, and stage 2 completed. **Stage 5 is not blocked; it is not started** |

### 6.5 `FUNCTIONAL_CONTRACT.md` disagrees with itself in four places

Its bodies were not updated when its summary table and absent-list were. The
document explains why — it is a summary no feature commit is obliged to touch —
and it has now gone stale three times in eight days, with its own header
recording corrections on 2026-08-23, -24 and -25.

- Capability 3's body says slider-triggered live re-tuning is absent; `:578` and
  `:644-651` both record **SG-03 closed 2026-08-24**, citing `set_params`' own
  doc comment.
- Capability 6's body says the atlas/tile cache and the bake lock "remain
  unbuilt"; the absent-list strikes that bullet as landed
  (`cartalith_engine::bake::AtlasStore`).
- Capability 6 lists AO toggles as absent; AO shipped and `render.rs:1515`
  exposes `"ao_strength"`.
- Capability 13's body says urban milestones 8-17 "remain entirely unbuilt";
  8a and 12 landed 2026-08-24.

### 6.6 The reference freeze has actually drifted — the re-freeze itself closed 2026-09-02, the stale sentence survives

`FUNCTIONAL_CONTRACT.md:16-24` asserts the frozen v2.10 is the live repository's
latest and there is *"no re-freeze question to raise"*. That sentence is still
there, unedited — scope documents were deliberately not mass-edited in the
re-freeze pass — and it is still wrong.

**What is fixed:** `reference/` now holds `Cartalith Gen1 v2.11.html` (2 374 691
bytes) alongside v2.10 (untouched, byte-unmodified), `reference/FUNCTION_INDEX_v2.11.md`
was generated mechanically, and `REFERENCE_DRIFT_v2.10_to_v2.11.md` records the
drift. This was §2.8's row; it is done and deleted from that list, so
`FUNCTIONAL_CONTRACT.md`'s capability tags now have a v2.11 index to be checked
against, even though nothing has re-checked them yet.

**What is not fixed:** `FUNCTIONAL_CONTRACT.md`'s own sentence — a documentation
defect independent of the index it was excused by. And a question the re-freeze
raised rather than closed: whether the root `Cartalith Gen1 v2.11.html` is
`Cartalith_RC`'s actual live head, or a copy that repository has since moved
past, is unverified and unverifiable from this machine — see §3.3.

### 6.7 Five documents claim a blocker that has already lifted

Each of these will cause someone to skip real, startable work:

**A fifth instance was found and fixed 2026-09-02, and it is the most expensive
one yet recorded — it cost a dispatched wave.** Three live test headers stated
that golden fixtures *"could not be extracted because the environment has no JS
runtime"*. `node` is v24.19.0 and has been available for weeks; the retired
`CHANGELOG.md` even notes at its line 2000 that *"Node is installed now"*. Worse,
everything the claim gated had **already shipped on 2026-08-15**:
`stamp_volcanoes_provinces` was golden-verified in `713e0b1`, and `555b753` /
`3fd2fef` flipped `volc.provinces`, `terrain_wind_deflection` and `currents` to
`true`. Nothing in this port was ever off-by-default because of it; only the
prose lagged, for eighteen days. `tools/jsruntime_probe.js` now settles it two
ways — it proves the runtime executes the frozen reference *and* that the
committed fixtures are genuinely its output rather than the Rust port's, with
4/4 mutants killed. Three further stale claims in the same family
(`cartalith-climate/src/lib.rs`, `golden_parity_weather.rs`, and
`cartalith-engine/src/lib.rs`'s own `WorldParams::defaults` comment) are
recorded but **not yet swept**.

| Claim | Reality |
|---|---|
| `journey_bridge.rs:70` and `JOURNEY_PLANNER_SCOPE.md`: the ecoregion/species-richness subsystem "is unported and on no milestone anywhere" | Ported 2026-08-23 (`b7a46a7`) — `wildlife.rs:367` `build_ecoregions`, `:550` `region_richness`, `:588` `assign_wildlife`. The remaining work is wiring, and smaller than either document says |
| `ANDROID_BUILD_SCOPE.md`: the ~19 MB of `godotsteam`/`godot_ai` addons is "flagged, not fixed", including in a Done-means table row | Fixed 2026-08-20 in `d044af9` (`export_presets.cfg:56 exclude_filter`), with no `CHANGELOG` entry — so the fix is invisible to the docs and the doc actively misreports it |
| `GPU_LAYER_INTEGRATION_SCOPE.md` m6: `use_gpu` is deliberately "unexposed in the UI until a real UI/UX pass adds the §7c messaging" | The shell exposes it (`menus.gd:2792`) **and defaults it on at boot** (`engine_bridge.gd:170`). The engine default is still `false`, so both statements are locally true and the conclusion is stale. §7c's messaging requirement *was* met (`menus.gd:1629-1632`), but nowhere in that document |
| `CPU_MULTITHREADING_SCOPE.md`: using the integrated GPU alongside the dedicated one is a "separate, lower-priority idea recorded, not scoped" | Shipped 2026-08-20 as `crates/cartalith-gpu/src/multi.rs` — enumeration, selection, split tiles and a VRAM cap |

### 6.8 Counts that disagree with themselves

Small, but this is the document set that exists because countable claims drift.

- `ROADMAP.md` Phase 4 says "all seven milestones"; `ASSET_LIBRARY_SCOPE.md`
  §11 records an **eighth** (the sprite-sheet slicer, 2026-08-20). The count is
  stale low, not the work.
- `ROADMAP.md`'s "Not a phase: LOD and large worlds" still says *"revisit when
  a concrete need appears rather than building it speculatively"*, while
  `STATUS.md` lists shipped "LOD levels 0-8, Tiled LOD auto/manual" and
  `LOD_TILING_BASE_SCOPE.md` exists.
- `URBAN_MORPHOLOGY_SCOPE.md` gives the `_um*` adapter's denominator as **20**
  at `:2098` and **28** at `:1770`. The 20-item list is the one that enumerates
  names, so it is the checkable one: 13 ported, 5 blocked on milestones
  9/10/13/15, `_umPt` typed away, `_umCacheKey` out of scope.
- `UNWIRED_FUNCTIONS.md`'s headline **77** double-counted two rows its own
  "fixed during the audit" section closed (State religion,
  `_refresh_phone_bar_lit()`); 75 were genuinely open at the 2026-08-31 cut.
  Its Large section heading read "(16)" where the intro said 18. **Both
  historical as of the 2026-09-01 re-cut**, which was written from scratch
  against the tree rather than patched, and carries internally-consistent
  counts (18 Large in both the heading and the running total) — see
  `STATUS.md`.
- `LARGE_ITEM_RULINGS.md` says the 3D research "stands complete at 1 486
  lines"; `wc -l` gives **1 530** — drift inside the same day.
- `LARGE_ITEM_RULINGS.md` answered owner questions 4, 5 and (by implication) 7;
  the 2026-08-31 cut of `UNWIRED_FUNCTIONS.md` still listed all ten as open.
  **Fixed in the 2026-09-01 re-cut**, which marks 4, 5 and 7 "Answered" by
  name (7 only partly executed: the fifth save slot the ruling called for is
  still unbuilt) and leaves 1, 2, 3, 6, 8, 9 and 10 genuinely open.

### 6.9 One claim that would misdirect a ruling

`URBAN_MORPHOLOGY_SCOPE.md:1761-1766` contains the sentence *"the crate is not a
dependency of `cartalith-godot`"*. It is a **quotation of what `PARITY_AUDIT.md`
§3.4 found before milestone 17a**, and the same paragraph goes on to describe
closing it. Read out of context it will produce a ruling to add a Cargo edge
that would buy nothing and violate the layering `cartalith-civ/Cargo.toml:18-22`
defends. The full correction is in §2.1.

---

## 7. What this document does not cover

- **Test status.** No surveyor ran `cargo test`. Every "done" and every
  "golden-verified" here is the owning document's claim carried forward, plus a
  structural code check that the named module or binding exists. The known
  intermittent failure (`generate_terrain_gpu_path_is_deterministic_and_valid`)
  is listed in §3.1 as a decision, not a result.
- **The re-verification that commit was the precondition for.** This bullet
  used to read *"the uncommitted working tree… **126 tracked files** now
  differ from `HEAD` (16 488 insertions, 10 499 deletions)… Every such row
  needs re-verification once that work commits."* **That work committed**
  (`fd9de7c`, 237 files / 90 718 insertions), so the precondition is met and
  what is left is the debt, not the tree: `git diff --shortstat` re-run
  2026-09-01 gives **1 file changed** (an in-flight `journey_planner_view.gd`),
  and the only untracked paths are two `_routecutout_probe.*` scenes and a
  `tools/__pycache__/`. Nothing in this document has re-verified a closed row
  against the committed tree yet.
- **`UNWIRED_FUNCTIONS.md`'s 21 rows individually** (22 after the same-day
  second pass, 23 after the 2026-08-31 cut's morning pass, 75 before it).
  They are one row here because that document is the live backlog with its
  own `file:line` per row, and forking it would guarantee the two drift.
- **`GUI_GAP_REGISTER.md` as a working list.** Its ID total was re-counted three
  times (123 → 215 → 300) and its A/B/C/D open/closed split was never
  re-derived once; a class marker survives on only 54 of 215 rows. Read it as
  history. `UNWIRED_FUNCTIONS.md` is the successor, re-cut 2026-08-31 against
  the three-domain shell.
- ~~**The stray root files.**~~ `518.86`, `518.92` and `66.0` — accidental
  shell-redirect artefacts from the memory-measurement work — **are gone**
  (`ls` finds none of the three, 2026-09-01). Kept as a struck line rather
  than deleted so nobody re-investigates the same three filenames.
