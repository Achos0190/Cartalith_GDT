# Status

**Cartalith native port — the milestone ledger.** Rewritten from scratch
2026-08-31 against the working tree, replacing an 8 122-line narrative that had
become a second changelog and mis-stamped itself 2026-08-25.

> **Before editing code, run `MISTAKES.md`'s preflight table** at the repository
> root — it is keyed to what you are about to do, and is preemptive by design (owner
> instruction, 2026-09-03). This file says what state the project is in;
> `MISTAKES.md` says what has gone wrong reaching that state and what rule
> prevents each recurrence. Two of its entries exist because *this* file
> carried a false claim — it named three deleted probes as "present and
> uncalled", and it asserted landmark generation was unbuilt on the day a
> 3 730-line implementation of it shipped.

---

## Orientation — read this screen, then stop if that is all you need

**Phase.** Phases 0, 1, 2, 4 and 5 are complete. **Phase 3 (rendering) is the
only phase with milestone work outstanding.** Phase 5’s milestones closed on
2026-09-03 and this line said *in progress* until 2026-09-12 — see below.

- **Phase 3** — the 2D half is done (`TERRAIN_APPEARANCE_SCOPE.md` milestones
  1-6, all six verified below). **The 3D drape does not exist**: zero
  `MeshInstance3D` / `Node3D` / `Camera3D` occurrences anywhere under
  `godot-project/shell/` or in any `.tscn`; the only real scene is
  `shell/app.tscn`, a `Control` tree. 3D is **parked by the owner, 2026-08-31**,
  the same day the commissioned research landed —
  `cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md`, 1 530 lines, complete
  with its recommendation made and its own *Status: parked* heading listing
  three unanswered questions. `DECISIONS.md` §4 continues to stand and **no 3D
  work of any kind is scheduled**.
- **Phase 5** — **milestones complete, verified 2026-09-03 in `9e79e52`, and this
  file did not record it for nine days.** Until 2026-09-12 this paragraph said
  milestone 16 was still open and that a verification pass over milestones 9, 10
  and 13 — ported by agents that died before reporting — was *in flight*. Both
  had been settled by three adversarially verified batches (16-18): milestone 16
  had already shipped in `cff1edc`, with its golden **re-derived independently
  from the frozen reference** (`node tools/um_capture.js`, 29 cases, byte-identical);
  **12 of 13 stage modules proven mutation-covered**; and milestone 17’s five
  `_um*` adapters survive mutation.

  **What remains is defects, not milestones**, which is why the table marks it
  **done\***. That commit does not name the one stage module left unproven, and
  nothing this sweep found says whether it is 9, 10 or 13. The drawing defect
  this paragraph used to name — the block-ground fill in `draw_layout` failing
  triangulation at deep zoom — **was fixed 2026-09-13** (`773262e`, parcel fills
  `de30275`): `urban_layout_draw.gd::_fill_ground_polygon` fans a polygon the
  ear-clipper rejects into triangles instead of skipping it, and the
  `OUTSTANDING_WORK.md` row is struck. *Corrected 2026-09-24 (alignment audit
  Part 1 C23): this paragraph still called it open eleven days later.*

| Phase | `ROADMAP.md` says | This file says | The one thing to know |
|---|---|---|---|
| **0** — walking skeleton | done | **done\*** | The `.exe` and `.apk` exist and the extension loads. `export_presets.cfg` defines two targets, Windows Desktop and Android, which is what `ROADMAP.md` now says; WASM is uncommitted |
| **1** — terrain MVP | done | **done** | All seven `MVP_SCOPE.md` criteria; the ocean-current stretch goal shipped too and was never recorded either way |
| **2** — civilisation layer | done | **done** | All 21 milestones, plus the Journey Planner sub-phase engine-complete at 66 of 74 `jp*` functions |
| **3** — rendering and 3D | partial | **partial** | 2D done, 3D absent and parked |
| **4** — Asset Library | done | **done** | Eight milestones — the slicer landed 2026-08-20 (`ROADMAP.md` no longer carries a count) |
| **5** — urban morphology | milestones closed, defects remain (2026-09-13) | **done\*** | Every milestone has code; 16 and 17 closed under adversarial verification 2026-09-03 (`9e79e52`), 12 of 13 stage modules mutation-covered. Open work is defects |
| *not a phase* — LOD and large worlds | the base and integration scopes, then `LOD_DETAIL_SCOPE.md` (2026-09-13) | **built and shipping; LOD-D0 to D6 closed 2026-09-21** | A tiled deep-zoom pyramid is on screen; the chunk atlas is baked and stored but nothing draws from it (LODI-M3; *corrected 2026-09-24: this said the persistent atlas is on screen*). **Since LOD-D1/D2 a deep tile is real colour**: `render_biome_tile_rgba` is the port of `renderBiomeTileRGBA`, byte-identical to the reference in `golden_parity_tile_biome.rs` (`f6d1bd5`), and `lod_bridge::synthesize_tile_rgba` feeds it to the tile shader (`9d2a800`). Corrected 2026-09-23: this cell said the port was missing and tiles carried only a shade ratio, which stopped being true when LOD-D1 closed. Several D-milestone acceptance bars were closed as measured and not met. See the *LOD detail* group below. Owner-supplied direction arrived 2026-09-12 as `docs/research/lod extra info.md`; `LOD_DETAIL_SCOPE.md` turns it into milestones LOD-D0 to D6 (plus an optional D7), and `ROADMAP.md`’s LOD section points there (2026-09-13) |

**Every "done" above means "done against `reference/Cartalith Gen1 v2.10.html`",
and the source has moved twelve mainline versions past it.** Measured
2026-09-17 in the working copy: the source repo holds **164** `Cartalith Gen1
v*.html` (newest **v2.22**) plus a second line of **49** DCC files (newest
**v2.71**), and it **forked at v2.22** — every engine change from v2.25 on
exists only on the DCC line. This does not un-do a milestone; a phase verified
against v2.10 is still verified against v2.10. It does mean **no row above can
be read as "matches the source today"**. *(Noted 2026-09-24: `OUTSTANDING_WORK.md`
§2.8's re-freeze row carries a different measurement of the same folder, also
dated 2026-09-17 — 44 DCC-line files, newest v2.66 — against this paragraph's
49/v2.71. The source repository is outside this one, so neither figure was re-measured here; treat both as dated.)*

**`RC_ENGINE_CHANGES.md` is the full porting spec for v2.11 → v2.73** (function
and constant named per change, why each number is that number, which harness
verified it) — read the change there, not here; this file only tracks whether
each has been ported. Nine of the interval's changes are deliberate upstream
re-baselines a golden fixture taken against v2.10 will fail *correctly*: the
eight **v2.48, v2.49, v2.50, v2.51, v2.57, v2.59, v2.60 and v2.61** move `field`
itself, and **v2.55** moves every LOD tile and baked atlas chunk (never
`field`). *(Corrected 2026-09-24, alignment audit Part 2 E3: this said "Seven"
over a list of eight plus v2.55.)* The single highest-leverage one is **v2.57**
(`RC_ENGINE_CHANGES.md` §6i): it retunes the plate-base blur radius
(`PLATE_BASE_BLUR_K` 0.35 → 0.18), measured as the single highest-leverage
constant in the height formula — the coastline is the level set of a blur of a
piecewise-constant plate Voronoi map, and the un-retuned partition reproduced
the land mask at only IoU 0.813. **v2.57 is ported as of `a74b35c`
(2026-09-24)**: `cartalith_engine::PLATE_BASE_BLUR_K` (0.18) behind
`TectonicParams::narrow_plate_base_blur`, chosen by `plate_base_blur_r` — on in
the shipped app (`cartalith_godot::params::defaults`), off on the parity path
(`PLATE_BASE_BLUR_K_V2_10`, 0.35), so no golden moved.

**Which of the v2.11 → v2.73 interval is already ported was surveyed on
2026-09-21** (corrected 2026-09-23: this paragraph said it was "not established
anywhere" for two days after the survey ran). The counts, the method and which
of its claims were and were not spot-checked are in `OUTSTANDING_WORK.md` §2.9's
first row. Read them there; they are not copied here, so they cannot go stale
here. Two limits to know: the survey predates later ports on the same interval
(depression-filled flow routing, §6g, landed 2026-09-22 in `76f64bc` —
`build_routing_surface`, `DECISIONS.md` §7o), and it is a survey rather than a
per-change ledger, so this file still carries no per-change status for the span.
**Which line to follow is settled:** `LARGE_ITEM_RULINGS.md` Ruling AP
(2026-09-23) chose the DCC line, so the re-freeze (`OUTSTANDING_WORK.md` §2.8)
now waits only on picking an exact DCC version and regenerating the snapshot and
its index.

**What landed most recently** is in *The last seven days*, below — a fixed
snapshot here goes stale fast (this paragraph used to carry one and it read as
current for ten days after it stopped being updated), so this section points
at the dated log instead of keeping its own copy.

**What's next** is `OUTSTANDING_WORK.md`'s full backlog; see *What is left*,
below, for the current count and the top blockers. (One item that used to be
named here by name is stale: urban morphology's milestones 8-17 are all `done`
in the Phase 5 ledger below (re-checked 2026-09-24; this said one was `partial`
and one `ready`), not "nothing started" —
that milestone group finished after this paragraph was last written and was
never updated to match.)

---

## What this file is, and what the other files are not

- **This is the only place progress is recorded.** Owner decision,
  2026-08-31. If a status is not in this file, it is not tracked. `CLAUDE.md`
  and `README.md` both name this file authoritative; that is now literally
  true rather than aspirational.
- **Scope documents define milestones. They do not track them.** Read a scope
  document for what a milestone *is*, what it beat, and why it is shaped that
  way. Do not read one for whether it is done — every status column and
  progress claim in them is being removed in favour of a pointer here. Until
  that pass finishes, treat any status sentence in a scope document as
  historical.
- **`cartalith-native/docs/CHANGELOG.md` is retired.** Frozen and marked, not
  deleted: 29 534 lines of per-milestone narrative that git messages do not
  carry, kept as history. It stopped being maintained on 2026-08-26 and
  stopped being a source of state on 2026-08-31. A grep for `2026-08-3`
  across it returns **zero matches** while `git log` shows eleven commits on
  2026-08-30/31, so it was already five days behind before it was retired.
- **Every status below carries its own evidence**, named as a symbol or a
  file. No row rests on another document's claim. Where a milestone cannot be
  checked from code — a device measurement, an owner action, a research
  finding — the row says so instead of asserting a status.

### Status vocabulary

| Value | Means |
|---|---|
| **done** | The named symbols exist in the working tree and, where the milestone required it, are reachable from a caller |
| **done\*** | The code half is verified; the remainder is an owner action or a device measurement this file cannot check. The row says which |
| **partial** | Some of the milestone's own "done means" is met and some is not. The row says which half |
| **not started** | Verified absent. The row names what was searched for |
| **blocked** | Not started, and something concrete stops it. The blocker is named |
| **declined** | Deliberately not built, with the reason recorded in code or in a ruling. Not a gap |
| **shelved** | Built or buildable, and stopped by the owner. **No row carries this today.** `EXPORT_SCOPE.md`'s five rows did until the owner un-shelved them (ruling 15, 2026-09-06); Ruling AP (2026-09-23) resumed the remaining batches |
| **unverified** | The deliverable is not a code artefact. The row says what it would take to check |

A few rows carry a **qualified** status — *done, superseded*; *done, evidence
re-pointed*; *done, answered negatively*; *done, no consumer*. That is
deliberate: those milestones are built and something about them is not what the
defining document expects, and flattening them to a bare `done` would lose the
part worth knowing. The qualifier is always explained in the same row.

**Verification method.** Every row was checked by opening the symbol in the
working tree on 2026-08-31, not by reading another document. Line numbers drift
and are given only where a symbol name is not enough; prefer the symbol.

---

## The last seven days

> **Stale — this section stops on 2026-09-20. Read `git log` for anything
> later** (marked 2026-09-24, alignment audit Part 1 C28 / Part 2 C2). It was
> not caught up here because the gap is too large to re-check claim by claim:
> `git log --format=%ad --date=short | sort | uniq -c` (author dates) counts
> **149** commits on 2026-09-21, **22** on 09-22, **107** on 09-23 and **52**
> on 09-24 (to `919bce1`, when this note was written). The status changes those days made are recorded in
> the ledger rows themselves, each dated. What follows is kept as written.

Dated, because this is what a returning session needs and it is exactly what
went missing from the old file. Commits are from `git log`; each claim below was
re-checked against the tree rather than copied from the commit message.

### 2026-09-20 (night)

A second batch landed the same day, same method (3 builders at the standing cap, Opus 5 max for the two engine lanes, Sonnet 5 for GUI, each independently verified):

- **Ruling N, river half, built and verified** (`649897f`). `build_settlement_suitability`’s river term now reads proximity to a real traced river polyline instead of a raw flow/order sample. Deliberate golden re-baseline, three settlement golden suites re-pinned. The verifier caught and corrected one shipped overclaim: connectivity here is enforced structurally by the order>=2 threshold, not by the polyline itself — true for rivers, not for coastlines, which is exactly why EF-6 matters.
- **EF-6 built and verified** (`630c0dd`). Coastline and fault-line tracing (one shared contour primitive) plus ridge tracing (a second, TPI-based one) — zero prior vectorization of anything but rivers, checked by grep. Unblocks Ruling N’s coastal half. Verified independently: 0 missing/0 spurious contour crossings, every ridge point above land mean+1SD.
- **PLAN’s phone-sheet subtitle fixed** (`554d953`). Hooked to `_apply_result()`, where every recompute path converges. A residual, pre-existing, out-of-scope gap disclosed and filed: the header’s TITLE half still doesn’t refresh on stage isolate.

Three tracks landed in one session, run in parallel per the owner’s instruction (1 GUI builder, 2 engine builders at Opus 5 max effort):

- **CIVIL half of Ruling L closed** (`5f839d7`). Both confirmed behaviour bugs and all three layout defects fixed, re-verified on all five form factors (271 HEAD diffs before, 0 after). Owner call C1 (entry-lit mismatch) left open. WORLD and CARTO not started.
- **The IME row closed** (`f2b0e33`). `DccWidgets.phone_present()` — the ~15-caller shared phone-dialog routine — now clears the on-screen keyboard; a stale duplicate formula in the conformance grader, found by the verifier, fixed in the same commit.
- **EF-0 and EF-1 built** (`2373c08`), engine-only, no Godot bridge yet: a queryable multi-resolution elevation primitive (EF-0) and boundary-seeded local hydrology re-accumulation for real river tributaries (EF-1) — the first two pieces of `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`. An adversarial verifier corrected two overclaims (EF-1’s "consistent by construction" language; an understated exit-and-re-enter bound) and found one real integration hazard for later (`carve_rivers=true`, the default, makes `WorldState::field`/`flow_discharge` mutually inconsistent) — all three corrected in the code’s own doc comments and the design document rather than left as an unqualified success. 289 lib tests plus a new 7-test suite, 0 failed; no golden-parity test moved.
- **Ruling N recorded** (`eea35e1`): the owner’s complaint that "a settlement should be properly rendered on a coast and along/around a river" is a real, named defect — settlement siting decides river/coastal status from per-cell proxies with no reference to a real connected waterway, byte-for-byte the same in the legacy HTML and this port. Owner ruling: fix siting itself, staying a term inside the existing suitability ranking. River half unblocked and filed; coastal half blocked on EF-6, filed not built.

### 2026-09-12

Work resumed after the weekly limit reset (the batch lost to the limit on
2026-09-08 had failed at dispatch, so it was re-dispatched unchanged, then
exited mid-batch and resumed from cache). Most of the day went into
correcting status documents that had gone stale while the tree moved under
them — **the costliest was Phase 5**: this file's own Orientation/phase table
and `CLAUDE.md`'s Contents table both still called urban morphology *in
progress* nine days after `9e79e52` closed its last two milestones under
adversarial verification. Both corrected. Also stale: the Orientation's "what
landed most recently" still led with 2026-09-02 and called it *uncommitted*
ten days after `4ec07f5`/`45b368d` committed it; `OUTSTANDING_WORK.md`'s
headline said 100 against a live 118; `SESSION_HANDOFF.md`'s budget line had
run out and reset; the LOD phase-table row said *built and shipping* with no
note that a deeper level can't be sharper. One known defect
(`draw_layout`'s block-ground triangulation failure, from the 2026-09-08 roof
fix) had no backlog row and was filed.

**Four owner rulings, H-K** (`LARGE_ITEM_RULINGS.md`, against reference
images at `design/owner-references-2026-09-12/`) set direction for urban
generation and the LOD zoom: a walled market-town plan (draw what the model
already generates, add a culture profile, and an **authorised golden
re-baseline scoped to `cartalith-urban`**), a citadel and star forts, a
per-settlement city-type/regenerate menu, and LOD tiles carrying colour
before ice — all behind the GUI rows, tablet first. Seven rows filed,
backlog 119 → 126. **One ruling was asked on a false premise and caught
before recording**: star forts were reported blocked on settlements carrying
no `fortified` trait, but the trait path is live end to end — only the
renderer's bastioned branch was missing, on a comment whose reason had gone
stale. Ruling I was recorded with the verified chain; the mistake is in
`MISTAKES.md`.

A phone-sheet clipping fix (pinning the GENERATE sheet's PIPELINE/SCULPT
segment above its scroll) was built, then refuted — the canvas keeps that
segment inside the scrolling body, under a header the shell never built, so
at peek the shell leaked half a clipped row instead — and reverted, patch
kept. Its citation sweep found 51 stale canvas citations, not 49.

**The on-glass method's blanket claim was split**: a desktop probe cannot
prove a finger reaches a control, but the same clipping defect reproduces on
the desktop composition within **3 px** of the device, so **desktop layout
measurements are trustworthy** — recorded on the METHOD row.

Owner-supplied research arrived, `docs/research/lod extra info.md` on
scale-dependent terrain detail (left in the *source* project's `docs/` tree,
where the owner put it, though it targets this port). **Mapped against the
code before filing**: its crate names are not this workspace's; the pyramid,
a live level ladder, and §16's macro/meso/micro weights and multi-scale
shading already exist; the gap it describes is the one already on the
backlog — LOD tiles carry a shade ratio rather than colour.

**2026-09-13.** The owner re-sorted the PC left rail (**Ruling L**); five
batches followed. First batch shipped two of Ruling L's three lanes: a typed
seed now reaches the built world, including the rolled-while-hidden case
that sank the previous attempt; re-entering a domain no longer drops the
armed tool's options row; the journey planner no longer swallows CIVIL
navigation while armed (the old code had swallowed all 14 categories). Three
concurrent batches (five builders, file- and measurement-disjoint, on the
owner's instruction) were each refuted in part and reverted — patches kept
in `.claude/resume-2026-09-12/`: a tablet rail/menu fix shipped inside an
undeclared landscape-dock change that turned three committed probes red; a
seed fix broke rolled seeds; a re-arm lane shipped past its own gate and
swallowed navigation again. Findings kept: Android BACK never commits a
`SpinBox`; the journey planner is the only view with the re-arm pattern; and
re-entering a domain overwrites any armed tool's options row (pre-existing).
Zoom did not reproduce on desktop.

Two further tablet/phone batches landed and held two items for fix-up (a
phone GENERATE sheet header with unscaled text; a failed-open error hidden
on phone): the tablet now fits its own frame (options row scrolls only on
overflow; portrait 800×1280 and 1024×768 fit) and draws the canvas's ☰ File
World Data menu bar with every accelerator; the phone bottom nav draws the
canvas glyphs. Two more batches then closed both held items and more: the
phone GENERATE sheet shows the canvas's header at peek; a refused project
open no longer strands the user (dialog/picker stay open, phone shows a
warning banner); 8 of 9 WORLD categories draw 232 px in tablet portrait; the
Window menu's dock switches read real state; native saves show their seed;
parcel ground fills stopped vanishing at deep zoom. A parallel small/medium
run added round towers on curtain walls, a pan-mode drag cursor, a fitted
tablet export pane, and a drop-count probe for reopened saves; a
district-based roof tint and a vector river overlay were built, refuted
(district is not wall containment; the overlay drew over debug views with no
off switch) and reverted before commit.

**Evening**: `LOD_DETAIL_SCOPE.md` turned Ruling K and the owner's LOD
research into milestones LOD-D0…D6, now rows in `OUTSTANDING_WORK.md`;
`REFERENCE_MAP_RECONSTRUCTION_RESEARCH.md` and Ruling M recorded the owner's
design answers for sculpting over a loaded map image (designed, not
scheduled); a signed release APK from `01e4faa` went to the 6T. Ruling L's
CIVIL half is **held, not committed** — its adversarial panel confirmed two
behaviour bugs (an armed Way tool's Commit/Discard row lost on some rail
presses and jumps).
### 2026-09-09 – 2026-09-11

**Nothing happened, and that is the whole entry.** The weekly usage limit was
reached partway through a batch on 2026-09-08 and reset on 2026-09-12. **No
commits, no lanes, no tree changes** — `git log` shows the jump directly from
2026-09-08 to 2026-09-12.

**Written down because an undated gap reads as missing records.** The
2026-09-03 – 2026-09-06 block above is the opposite case: work happened there
and was never written up. **This one is a genuine quiet period**, and a session
reading `git log` should not go looking for the batch that filled it.

**One batch was lost rather than half-finished**, which matters for trusting
the tree: all three of its agents failed at dispatch, so the working tree was
clean at `1bc24f6` and nothing partial had to be unwound. It was re-dispatched
unchanged on 2026-09-12.
### 2026-09-08

Cheapest-first at the owner's direction, lanes allowed a smaller model.
Backlog **126 → 119**.

**Finding of the day was about proof, not code.** A lane converted the right
dock's readouts to the user's unit preference correctly (Sample Position,
route Length, river Catchment, Ecoregion/Territory area, every Measure
field), holding elevation, bearing, Centroid (grid cells), Discharge (a
rate) and Productivity/Ruggedness byte-identical as negative controls — but
its own probe couldn't police it: the three Position checks tested for a
`mi` suffix, a `km` suffix and inequality, **all three satisfied by
relabelling alone** (a mutant that kept the km value and appended `mi`
passed every one). A3b/A3c now rebuild the expected number from
`DccUnits.to_unit()` at probe time instead of a typed constant; the same
mutant now fails A3b. The conversion was right; only the proof was weak.

Four smaller findings the same day: **Falloff** was reported unbuilt by a
stale note even though `Falloff`'s `coverage()` is consumed at three sites
in `SculptStamp::apply_into`, registered as `GLOBAL_RANGES` row 9, and
already a live dropdown — the fourth stale "unbuilt" reason this tree has
shipped. Three rows (`shortcuts_dialog.gd:417`'s unmappable-key guard, the
commit-message debt at `MISTAKES.md:132`, the Android `logcat` correction)
closed for the cost of a grep each — already finished, just never struck
from their numbered section (the counter reads sections, not strikethrough).
`ANDROID_BUILD_SCOPE.md:1272` still contains the string it corrects, inside
the blockquote doing the correcting — a grep for a stale string is not a
test for a stale claim. And the units sweep's *"Discharge 4,200"* beside
*"Catchment 3 500 km²"* turned out to be `right_dock.gd::_thousands()`
disagreeing with the canvas's own spaced-thousands convention, pre-existing
and just newly visible.

**Later the same day, the GUI rows, and three corrections to my own work.**
Roofs stopped failing to draw: at deep zoom Godot logged triangulation
failures and skipped buildings silently, but the geometry was never bad —
the same 5 009 footprints are non-degenerate in layout metres and at
fit-to-box scale, and the 2.7% that fail do so only through the deep-zoom
transform, every one with a post-transform bounding-box diagonal under
0.09 px (the renderer's own guard). Proved by whole-frame md5 (identical
before/after, errors 151 → 0, frames not blank at 24.6% ink), then re-tested
with the fix reverted. Thirteen fills across 9 files had never drawn a
pixel: a `Button` with `flat = true` silently voids every stylebox override,
`hover` as well as `normal` — verified on interior modal colour with the
border ring excluded so ink can't contaminate the sample, mutation-tested.
A ruling of mine (the action button takes `2px 12px`) was refuted and
reverted the same hour: the canvas disambiguates by height role, not by
frequency — `--btnH` (28px) is the action button at N=14, `--ctl` (24px) the
inline chip at N=15, and all eight `2px 12px` nodes are `--ctl`/`--tool`, not
one `--btnH`. Withdrawn as Ruling G. A lane separately reasoned past the
brief's explicit stop-and-report gate and implemented anyway — a gate that
can be reasoned past is not a gate.

**The on-device method ran for the first time and works.** `adb exec-out
screencap` on the attached handset found the GENERATE sheet's chip row
occluded by the nav bar at its collapsed detent (three pixel rows of a
~20 px label). It also produced two phantom defects and one dead end, none
filed: a downscaled view twice showed text the full-resolution crop didn't
contain, and an apparently-unresponsive MORE button turned out to be a
full-screen panel that had opened over the nav bar. Three of four first
readings were wrong and the fourth incomplete — the honest summary of a
first pass on glass.

### 2026-09-07

A full day on the GUI, driven by the owner's standing priority and by
defects found on the handset: seven multi-agent batches, then a long
main-loop stretch once the weekly budget ran low. Backlog moved
113 → 146 → **132** — it rose as the parity audit converted "nobody has
checked this" into rows, then fell as those rows closed; read a rise here as
*more known*, not *more broken*.

**The three canvases arrived and became the definition of done.** The owner
supplied PC, Tablet and Android through the `claude_design` MCP, imported
verbatim to `design/mcp-2026-09-07/` (a `.dc.html` renders standalone only in
**headed** Chrome). `TABLET_UI_SPEC.md` and `ANDROID_UI_SPEC.md` were written
from them, to the owner's *"All designs layouts and styles should match
100%"*, with the before/mid/after checks waived at parity.

**What the owner reported on glass, and what each turned out to be** — every
one had a green desktop probe behind it, and each probe was right about what
it measured: the journey planner's entire control surface hung in a phone
sheet built `visible = false` while `_left_panel.visible` stayed `true`, so
every probe passed measuring a panel nobody could see (fixed at `open()`,
the one function all seven entry points converge on); the sculpt drawer's
grab row measured 19.84 dp against a 44 dp floor, not gesture arbitration as
first hypothesised (`_detent_probe` had passed by pressing the handle's
exact centre); Preferences now stamps the selected value on 10 of 15 desktop
parent rows and the phone chip takes Medium plus the accent ink; the file
browser couldn't reach the owner's files on either platform (PC had no `..`
row or drive list; Android landed in the app sandbox) — both fixed, Android
now through SAF, and `Werk.zip` opens; and the picker tiles, previously a
colour gradient, now render the archive's own heightmap with no save-format
change needed (`SAVEFILE_COMPAT.md` already makes heightmap and sea level
MUST).

A regression shipped and its own probe asserted it: `e830112` right-aligned
`number()`'s field citing the canvas's `ENV:351` (a 52 px readout span) where
the real field is `SIZE_EXPAND_FILL` at 388 px, moving a 36 px number 343 px
from its label — and the same commit's `_inputfill_probe:165` asserted that
alignment, green on the regression it existed to catch. Both corrected.

All sixteen non-conforming dialogs now reach the phone, converted in the
main loop via the three-call protocol (`phone_window` at build, `phone_fit`
after the body, `phone_present` instead of `popup_centered`). A release APK
(`8bcb0dce…`) went to the D: drive, hash-verified at the destination and on
the handset, booting with 0 script errors. Every batch had an adversarial
verifier except one (verifier died on a session limit, debt discharged by
the next batch's verifier); the cargo floor held all day at **157 result
lines / 3 253 passed / 0 failed / 28 ignored**.

### 2026-09-03 – 2026-09-06

**Not written up here, and that is a gap rather than a quiet period** — read
`git log` for these four days. They are named so a returning session does not
read the jump from 09-02 to 09-07 as nothing having happened.

### 2026-09-02

Four parallel workflows (33 agents), every claim re-verified against the code
by an agent that didn't make it and re-run once more by hand: `cargo test -p
cartalith-civ -p cartalith-terrain -p cartalith-engine -p cartalith-godot`
aggregates **1 543 passed, 0 failed, 21 ignored** against a freshly-built dll,
all seven touched `.gd` files `--headless --check-only` clean. All uncommitted
when written; committed `4ec07f5`/`45b368d`.

**The landmark ("point of interest") pass, reported broken by the owner**
("seems to make the program freeze and doesn't render on the map") — two
symptoms, three causes. (1) `landmark_run()` ran synchronously on Godot's main
thread: `_poifreeze_probe.tscn` measured **0 main-loop frames served** during a
1 224.9 ms pass, against 255 for a `generate()` doing four times the work on a
`Thread`. (2) Nothing pushed placements to the map: `MapOverlay._landmarks`'
only writer is `ViewportHost.refresh_annotations()`, which
`civilization_workspace.gd::_lm_run()` never called, so a regenerate also left
world A's rings drawn over world B. (3) The one nobody predicted: a `#[func]`
that builds a `Dictionary` cannot be called from a worker thread — without the
`experimental-threads` feature (`cartalith-godot/Cargo.toml` pins `godot =
"0.5.5"` with only `features = ["api-4-7"]`), every `Dictionary`/`Array`/
`GString` op routes through `ensure_main_thread()`, and `generate_sized` had
only ever been thread-safe because it takes and returns primitives. **The
reusable lesson: a worker-thread `#[func]` must be primitives-in,
primitives-out.** Fixed accordingly: `landmark_run` now returns `bool` with
the reason in a `String`, `landmark_last_run()` builds the reply dict on the
main thread, and `engine_bridge.gd` reuses `generate()`'s `Thread` →
`call_deferred` → signal pattern with a new `landmark_finished` signal.
Separately, `box_h`/`box_v` and `sep_min_max` (`cartalith-terrain/src/analysis.rs`)
were parallelised over output rows (`par_chunks_mut(gw)`, bit-identical, not a
float reordering) — measured at the shipping 2048×1311 default: **4.14 s →
0.39-0.86 s**, off the main thread. `_poifreeze_probe.tscn` is the committed
regression check.

**Vulkan, DirectX and `RenderingDevice` all answered "no", the first by
measurement** — driving the committed `_shot.tscn` harness on the owner's RX
7800 XT (driver 26.7.1, Godot 4.7.1) via launch flags, no file edited to
produce the table: `gl_compatibility` boots and generates clean;
`forward_plus`/vulkan loses the device during generate **3 of 3**
(`VK_ERROR_DEVICE_LOST`); `forward_plus`/d3d12 segfaults
(`DXGI_ERROR_DEVICE_REMOVED`); boot is clean on all of them, it's the
*generate* that kills the device. `RenderingDevice` is separately disqualified
(null under both `gl_compatibility` and `--headless`, which would delete the
68 `cartalith-gpu` tests with no replacement). DirectX needs no work —
`COMPUTE_BACKENDS` already unions `DX12` and `backend_rank`'s Vulkan-first
order restates wgpu-core 30's own HAL registration order. 178 lines appended
to `3D_TERRAIN_RENDER_RESEARCH.md`, **3D left parked**.

**Four defects found while looking for something else**, each verified, none
fixed except the last: `engine_bridge.gd` forces `param_set("use_gpu", true)`
at boot over a `false` default whose own comment says the GPU path produces a
different world — **the shipped app does not generate the world the 88
`golden_parity_*.rs` files verify** (a product-default call, left to the
owner); `multi.rs`'s `is_software` doc comment claims a software rasterizer is
"never selected by default", but `force_fallback_adapter: false` only declines
to *restrict to* fallback adapters and filters nothing, so a box with no
working hardware adapter silently runs on Microsoft Basic Render Driver; no
`log` backend is registered anywhere, so wgpu's logging is a no-op and the
Android "zero wgpu lines in logcat" PASS condition **cannot fail** even though
wgpu/wgpu-hal/ash are compiled into the shipped arm64 `.so` with GPU forced on
at boot; and the route-map cutout placed LOD tiles half a world cell off the
colour they multiply (its probe checked UVs lay in `0…1` but never that a
tile's footprint agreed with where the sprite was placed — ranges are not
registration) — **fixed**, with two smaller defects beside it.

**Nine backlog rows closed** (`OUTSTANDING_WORK.md` §2.3/§2.6), each
golden-verified: Rayon across `road_dijkstra`'s three independent source maps
(ordering proved by mutation — reversing collection order fails four golden
tests); R8 (~45 MiB, by probe reduction and early release — the scope
document's prescribed "chunk it" mechanism turned out impossible, since Prim
reads an arbitrary source's result until the pass ends); R7 (`want_prev`,
10.24 MiB); R5 (`jfa_dist` → i32/i32/u32, 32.2 MiB, bit-identity proved by
mutation); R4 (`plate_id` → `u16`, 15.36 MiB); `_civPlaceSmelting` and
`_civSaltAccess` ported with a new `golden_parity_smelting_salt` suite; the
food-shed readout surfaced in the place editor's Trade tab; and the Nortantis
disclosure added to `credits.gd`.

**Documented-but-false claims corrected in place**: `DECISIONS.md` §7i,
`JOURNEY_PLANNER_SCOPE.md`'s 2026-08-19 update, `world_workspace.gd`'s "58
parameters" (really 81), the `paint_set_brush` doc comment, `roster.rs`'s
food-shed self-claim. Also: the GPU determinism flake was filed as
blocked-on-owner in four places, but `803b725` (2026-08-25) had already
replaced the `assert_eq!` with a 1e-6 worst-element tolerance; and a
`gl_compatibility` rationale does exist, in
`.claude/skills/godot-shell/SKILL.md`, though four of five investigators
reported it as never recorded.

### 2026-09-01

Eight `OUTSTANDING_WORK.md` §1 items closed across three re-verified passes
(each re-run `cargo test`/`cargo check`/`cargo clippy` clean and every touched
`.gd` file `--headless --check-only` clean, with `cargo build -p
cartalith-godot` re-run each pass to dodge the stale-dll hazard). Full detail
is in each subsystem's ledger row below, not repeated here: `UNWIRED_FUNCTIONS.md`
re-cut from 75 open rows to 21 (dangerous class 25 → 0, closed in two more
sub-passes the same day); `UNIFIED_TOOL_PLAN.md`'s "Milestone F as built"
section (Tool system ledger); Vault §14 Compare (MV-5); route corridors/travel
cost as a selectable analysis field (GFP-4); landmark `resource_extraction_site`
went buildable, 14 of 49 kinds (LM-8); `civ_food_shed` built and reached Godot
(EC-3); WORLD/CIVIL left-dock restyle and GUI replacement stage 4 (RP-S4);
paint brush falloff shipped, closing the tool system's last dangerous-class
rows, recorded as a deliberate reference divergence at `DECISIONS.md` §7k; a
tectonics World-Structure override-disclosure bug the owner found by manual
testing was fixed (`world_workspace.gd`'s `WS_OVERRIDDEN_KEYS`/
`_refresh_ws_override_rows`, dimming and disabling the three parameter rows
`deriveFromWorldStructure()` silently overrides); and all six of
`OUTSTANDING_WORK.md` §2.3's journey/route rows closed, three of them the
Journey Planner quality ceilings JP-QC2/QC3/QC4 above, the fourth
`DECISIONS.md` §7i's swamp/ford-cost terms (`civ_swamp_penalty`/
`civ_river_crossing_cost`, `cartalith-civ/src/lib.rs:5583`/`:5599`) wired into
`way_commit`, `route_commit` and `jp_reroute` so the formula can't drift
between the auto-populate road builder and the manual tools. All uncommitted
when written; committed by `4ec07f5` the next day.

### 2026-08-31

- **GUI replacement stages 1 and 2** (`c03b43c`). Stage 1: the new token
  system — `dcc_theme.gd`'s `sunken` re-based `#101112` → `#191c1e`, new
  `accent_ink` and `accent_wash_2`, and a fourth density set (`LAPTOP`, with
  `W_LAPTOP_MAX` and `is_laptop()`). Stage 2: the rail fold — `DOMAINS` holds
  world / civilization / cartography; `RAIL_NODES` holds 3 heads + 10 nodes,
  counted in the file. Guard probe `godot-project/_railfold_probe.gd` asserts
  node→category reachability and that every category appears in exactly one
  node's `owns`.
- **`UNWIRED_FUNCTIONS.md` re-cut** (`5543ef3`) against the new shell: **77
  open rows** (17 trivial · 25 small · 17 medium · 18 large), plus a 25-item
  dangerous class. The interesting finding is not stale wiring but **nine
  stale *reasons*** — a control disabled with a tooltip citing a binding that
  exists and is being called every tick. `audit_wiring.py` structurally cannot
  see these, because every `#[func]` involved *is* called and it is the prose
  that lies.
- **Owner rulings on all eighteen Large rows** (`LARGE_ITEM_RULINGS.md`,
  **untracked**). Fourteen to build, two to schedule separately, one deferred
  after research, one authorisation withdrawn as unnecessary. Two override
  standing rules: paint-brush falloff is a **deliberate divergence from the
  reference** and must be recorded in `DECISIONS.md` when it lands; colour
  management was ordered built with the golden-re-baseline cost stated and
  accepted.
- **Owner ruling: INFRA is absorbed by CIVIL, RENDER by CARTO** (`fbfcae2`) —
  the five→three domain fold that stage 2 then implemented.
- **The design files are whole** (`660cbef`). The desktop prototype had arrived
  truncated at exactly 262 144 bytes; the re-export split the heavy method
  bodies into `cartalith-dcc-parts.js` (54 059 bytes) behind `window.CDCC`.
  `Cartalith DCC Environment.dc.html` is now 239 712 bytes and ends properly.
  The same commit corrected a tablet threshold that had shipped wrong.
- **`DESIGN_HANDOFF.md`** (`f40969d`) and **49 landmark glyphs** (`a585ab1`).

### 2026-08-30

- **Landmark generation, end to end** (`36a9311`, `f084650`, `42263a4`,
  `ae62adf`, `a6feec3`, `c495821`). The owner's research imported, an inventory
  that found half of it already built, then the build:
  `cartalith-terrain/src/analysis.rs` (M1's field library — `slope`, `aspect`,
  `curvature`, `tpi`/`tpi_multiscale`, `local_relief`, `ruggedness`,
  `normalise`), `cartalith-civ/src/landmark.rs` (49 kind specs, `generate()`,
  `Landmark` carrying its causal chain, `LandmarkFunnel`, `LandmarkStore`),
  `landmark_bridge.rs`, ten `#[func]`s, the map ring layer, and the
  CIVIL ▸ Landmarks panel.
- **The gap register was itself stale** (`f184d69`): 12 of 44 rows wrong —
  eleven had shipped, one described a real gap inaccurately.
- **Six spec items that had no build** (`7b367b7`) — project picker, staged
  ten-stage generator readout, app-bar search, undo chip, two persisted coach
  marks — and the borrow panic one of them exposed: seven `viewport_host.gd`
  functions guarded `has_world`, which is false only until the *first* world,
  so a re-generate could reach a `#[func]` on a mutably-borrowed object.
- **Tablet targets 260 → 1** (`f129495`), and the 16:9 tablet that was getting
  the phone GUI. `phone_fit()` had no tablet sibling, leaving `DccTheme.ROLE`
  read by nothing.
- **Test harnesses committed** (`e1f18ca`, F8) — the ~70 probe scenes this
  repository cites as evidence, finally in the repository.
- Rivers gained a real width scaled to map extent (`58dd5b2`); centre landmasses
  had been deleting every river in the world, permanently (`d738c51`).

### 2026-08-29

- **Religion diffusion scoped** (`94b0f65`) from an owner-supplied paper, and
  **culture and religion as traits** (`c3ceb83`) —
  `crates/cartalith-civ/src/belief.rs`, 945 lines. See the caveat in the
  ledger: it has **zero consumers** anywhere in the workspace today.
- A canvas design pass whose own audit refuted all three of its designs
  (`2735fb7`) — recorded rather than shipped.
- LOD debug overlay (`c72a5b4`); the 16:9-tablet finding (`745932d`).

### 2026-08-26 and earlier that week

- Route planner fixed — the pathfinder was right, the planner never asked it
  (`5a40805`); pass-aware routes and a per-stage picker (`d029e13`).
- Wildlife forage reaches the planner behind a fingerprint cache (`af0485f`).
- The `erode()` op assembled, and a panic it was hiding (`00a9a3d`).
- Vault search and "confirm always" (`42f3acc`); project documents return what
  was stored rather than a re-serialisation (`afc2d57`).
- File ▸ Open had been reading the project tree as flat, silently dropping the
  civilisation layer (`82b49ad`).

**Update 2026-09-02:** All work described in this section is committed as of `4ec07f5`
(97 files, +31 990/−358). The working tree carries only one modified tracked file
and two untracked probe scenes; every status change recorded here is against the
committed tree.

---

## What is left

*Stale, noted 2026-09-24:* `OUTSTANDING_WORK.md`'s *The count, honestly* has moved past this — its latest entry reads **81** (after the alignment audit's §2.11 rows and the fixes that followed). Read that entry, not this paragraph, for the live figure. **Recounted 2026-09-23: 69 open items.** These are the unstruck table rows in
`OUTSTANDING_WORK.md` §1-§4, counted by script over the file. The newest entry
in that file's *The count, honestly* gives the method, and explains why the
figure is not the 71 its previous entry carried and what this pass closed and
filed. Everything below in this paragraph is history. **Recounted 2026-09-20 (night): 122 items.** Was 124 after Ruling N/EF-3/EF-6/EF-9 were filed (120 after the `main` merge; before that 110, then 115, then 120 — the fuller chain is above). Ruling N’s river half, EF-6 and the PLAN subtitle row closed (-3); the coastal-binding row updated in place, unblocked now that EF-6 landed; one new small row filed for a disclosed residual (the header’s TITLE half still goes stale on stage isolate) (+1). Run
`scratchpad/count_outstanding.py` rather than trusting this paragraph — it
counts rows in the NUMBERED sections and skips the archive sections, which are
deliberately unnumbered. **The counts below were 155 (3/99/33/20) and stood for
six days and 58 commits**, which is the regression `CLAUDE.md` names: a status
recorded anywhere but here goes stale here.

**§4 is now empty.** The twenty open owner decisions were all answered — the
last three on 2026-09-07 (`LARGE_ITEM_RULINGS.md`: Android file-picking to SAF;
the Preferences chip keeps Medium and gains the accent; the invisible OFF switch
track is a canvas defect and the shell must NOT be patched around it).

The full list is **`OUTSTANDING_WORK.md`** (assembled 2026-08-31; recount
2026-09-01 morning after eight `§1` items closed or narrowed; a same-day
second pass then closed two more — Paint brush falloff and GUI replacement
stage 4; a same-day third pass then closed all six of §2.3's journey/route
cluster rows; verifying that pass then found and closed one more, a §3.2 row
citing the same stale claim. **155 items across 24 subsystems**, every row
naming the document that owns it). Its counts, carried here so this file
answers "what is left" without a second read:

| | Count | Where |
|---|---:|---|
| In flight — code exists, uncommitted or partial | 2 | `OUTSTANDING_WORK.md` §1 |
| Ready to start — nothing blocks them | 106 | §2 |
| Blocked — a named blocker | 24 | §3 |
| Open owner decisions — not work yet | 0 | §4 |
| Declined / shelved — kept so nobody re-proposes them | 23 entries, 3 groups | §5 |

**The table above is itself stale** (155 was the count on 2026-08-31/09-01;
see the recount at the top of this section for the current figure and
`OUTSTANDING_WORK.md` for the live breakdown) — kept rather than deleted
because the two caveats below still apply to any count this document reports.

Two caveats that document states about itself, repeated because they change how
the number should be read: it counts **rows, not effort** (urban milestone 10
and "delete three probe files" are both one row), and it counts
`UNWIRED_FUNCTIONS.md`'s genuinely-open rows as **a single row** of the 3 in
flight above, because that document is a live backlog with a `file:line` per
row and forking it would guarantee the two drift. That row carries **21** open
rows as of the 2026-09-01 third pass (22 after the second pass, 23 after the
morning re-cut, 75 before it). Counted individually the true figure is nearer
177, not 155.

### The open owner decisions

**None of the six below is open now** (re-checked 2026-09-23). All were
answered, most of them twice: rulings 10, 12, 13 and 16 on 2026-09-06, then
Rulings AO and AP on 2026-09-23. The table is kept because its right-hand
column records what each answer does and does not build. The questions still
open after Ruling AP live in their own `OUTSTANDING_WORK.md` rows, not here:
the export's E4 scope questions and its 32K size/codec trade-off, and IN-13's
sea-lane question. This line used to read *"Twenty, in full in
`OUTSTANDING_WORK.md` §4"*, and §4 has recorded zero since 2026-09-06.

**That list was incomplete** (corrected 2026-09-24, alignment audit Part 2
C6; IN-13's sea-lane question has since been answered by Ruling AQ). Still
unruled, per `ALIGNMENT_AUDIT.md`'s owner-question lists re-read against
Rulings AQ, AR and AS: `LOD_DETAIL_SCOPE.md`'s six owner questions (no ruling
in `LARGE_ITEM_RULINGS.md` answers any of them; question 6 gates LOD-D7),
LOD-D5's `add_zoom_detail` octave-decay re-baseline, LOD-D1's two surfaced
decisions (tile vs map shading; the 40 ms synthesis
budget); whether v2.25's `tileShadeExag` is ported or declined; the MV-4
folder-picking mechanism; importing legacy flat `.zip` settlements/labels;
ruling 16's refine action's scope and the Peak viewshed term; the Military
*Not built* wording; the citadel's area in growth (Ruling AC left it open);
authored-battle spacing; erosion §8 Q1 and Q3-Q5. Ruling AS (2026-09-24)
answered three the audit listed as open — the orogeny derivation, the
`sea_grain_warp` default and the phone's 4K/8K presets — and added a fourth
(a sculpt commit clears the paint it covers). `3e6e0e1` recorded them and
changed no code. **Two are now built and verified (2026-09-24):** the
orogeny derivation (`cartalith-engine::world_structure_orogeny_ks`, pinned by
`cartalith-engine/tests/world_structure_orogeny.rs`) and the `sea_grain_warp`
default (`1.0` in `TerrainAppearance::default()`, `0.0` pinned in
`js_reference()`; `tests/color_space.rs::the_ocean_lattice_fix_is_on_in_the_shipped_look_only`).
The phone presets and the sculpt/paint clear are ruled and not built. None of the LOD
questions has a row in `OUTSTANDING_WORK.md` §3.1.

| # | Question | Gates |
|---|---|---|
| 1 | ~~**What is a conflict attached to**~~ — **answered 2026-09-23, Ruling AO** (a settlement or province, by `tid`); built as SP-4 | Story planning SP-4, and through it landmark M9. The highest-leverage unanswered question in the project: two documents' largest remaining milestones sit behind one unasked question |
| 2 | ~~**The viewshed cost budget**~~ — **answered 2026-09-06 (ruling 16: cheap and coarse, plus a manual refine) and 2026-09-23 (Ruling AP: the conservative default M7 shipped stays; no work)** | Landmark M7, done 2026-09-21 on that default (LM-7) |
| 3 | ~~**Regenerate semantics for a journey's route polyline**~~ — **answered 2026-09-23, Ruling AO: re-snapped**; built with SP-2 (policy in the ruling's addendum) | Story planning SP-2 — no longer gated |
| 4 | ~~**Does the landmark set live in the save tree, or regenerate on load?**~~ — **answered 2026-09-06 (ruling 10: PERSIST) and re-affirmed 2026-09-23 by Ruling AP** | **Built for the run**: `entities/landmarks.json` carries the landmark settings and the last run's results (`project_bridge.rs::LandmarksDoc`). **Not built**: any per-landmark authored or temporal state (a player-given name, research §25's states), which neither `Landmark` nor `LandmarkDto` has. That is LM-9's remaining temporal half. *Corrected 2026-09-23: this row was still posed as open, and Ruling AP's own text calls the save slot new work, although it has existed since 2026-09-06* |
| 5 | ~~**Does a landmark become a `cartalith_vault::EntityKind`?**~~ — **answered 2026-09-06 (ruling 13: YES) and re-affirmed 2026-09-23 by Ruling AP** | **Built**: `cartalith_vault::EntityKind::Landmark`, addressed by `landmark_entity_id` over `Landmark::key()` (`45630cc`, 2026-09-06). *Corrected 2026-09-23: this row said `links.rs` had no landmark variant* |
| 6 | ~~**Does `DECISIONS.md` §7a/§7d's parity contract apply to landmarks at all?**~~ — **answered 2026-09-06 (ruling 12: EXEMPT) and re-affirmed 2026-09-23 by Ruling AP**; all landmark work is divergence-by-addition, tested for internal correctness and determinism | Nothing to build. This confirms the assumption `landmark.rs` was built under |

**Answered on 2026-08-31 by `LARGE_ITEM_RULINGS.md`**, and therefore no longer
open despite `UNWIRED_FUNCTIONS.md` still listing them as such: icon placement
families (a fourth `SEA MARKS` family is created rather than mapped onto three),
paint falloff (bind it, as a recorded divergence — **built and verified
2026-09-01**, no longer merely answered), and the save-slot contract (a
fifth slot is scheduled, so the list is a live contract rather than residue).

---

## Milestone ledger

One row per milestone, grouped by subsystem, each group naming the scope
document that **defines** those milestones. The scope document defines; this
table tracks.

### Phase 1 — Terrain MVP · `MVP_SCOPE.md`

Nine rows: seven success criteria, one in-scope stretch goal, and the
out-of-scope table read against the code.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| MVP-1 | Criterion 1 — height / temperature / rainfall / flow match golden data at a fixed seed | done | `cartalith-terrain/tests/golden_parity_height.rs`; `cartalith-climate/tests/golden_parity_temperature.rs` and `golden_parity_weather.rs`; `cartalith-hydrology/tests/golden_parity_flow.rs`; `cartalith-engine/tests/golden_parity_pipeline.rs` for the assembled run. 94 `golden_parity_*.rs` files exist across the workspace (`find crates -name "golden_parity_*.rs" | wc -l`, 2026-09-24; was 88) |
| MVP-2 | Criterion 2 — the world renders as a recognisable 2D map | done | `cartalith-godot/src/render.rs::cell_color` / `material_weights`, pinned by `cartalith-godot/tests/golden_parity_render.rs`; drawn through `shell/viewport_host.gd` |
| MVP-3 | Criterion 3 — builds as a Windows `.exe` **and the owner has run it** | done\* | `godot-project/builds/windows/Cartalith.exe` + `Cartalith.pck` + `cartalith_godot.dll`; `export_presets.cfg` carries the `Windows Desktop` preset. The build half is verified. "The owner has run it" is not checkable from code — `DECISIONS.md` §5 says so explicitly |
| MVP-4 | Criterion 4 — builds as an Android `.apk` **and the owner has installed and run it** | done\* | `godot-project/builds/android/Cartalith.apk` plus nine later named builds (perf, mem, phonefix, ph412, dcc830, lm, dashA, devtest, release); the `Android` preset. Same limit as MVP-3 |
| MVP-5 | Criterion 5 — map width visibly scales feature size, as a consequence of parity | done | `cartalith_terrain::terrain_detail_k`, `river_coarse_ease`; `cartalith_hydrology::river_width_scale_k`, `river_flow_thresh`. Exposed at creation time by `shell/new_world_dialog.gd`'s "Map width & resolution" section |
| MVP-6 | Criterion 6 — a changelog entry records what was ported, verified and deferred | done, superseded | `docs/CHANGELOG.md` exists at 29 534 lines. **Retired 2026-08-31**; this file replaces its status role. The criterion was met at the time and is no longer the mechanism |
| MVP-7 | Criterion 7 — opens a real HTML-app `.zip` and renders that save's terrain | done | `cartalith_io::load_save`, with `cartalith-io/tests/golden_parity_real_export.rs` asserting against `fixtures/real_export_seed24601.zip` **and** an independent capture (`real_export_seed24601_captured.json`) rather than round-tripping the loader against itself — exactly what the criterion demanded |
| MVP-S6 | In-scope item 6's stretch goal — ocean-current terrain coupling | done | `cartalith_climate::{compute_ocean_current, current_ocean_field, deflect_flow, apply_ocean_currents, ocean_sst_anomaly}`, with `golden_parity_ocean_current.rs`, `golden_parity_deflect_flow.rs` and two regression tests. The scope document asked for the outcome to be recorded either way and **never recorded either**; it is recorded here |
| MVP-OOS | The "Out of scope" table — sculpt editor, 3D view, LOD pyramid, NPR styles, multi-resolution baking/atlas | 4 of 5 shipped | Sculpt — `cartalith-terrain/src/sculpt.rs` + `cartalith-godot/src/sculpt_bridge.rs` + `engine_bridge.gd`'s `get_sculpt_features`. LOD pyramid — see LODB/LODI below. NPR — `render.rs`'s watercolor/stipple fields with `tests/golden_parity_npr.rs` and `engine_bridge.gd`'s `npr_api`. Baking/atlas — `cartalith-engine/src/bake.rs` + `cartalith-io/src/atlas.rs` + `bake_bridge.rs`. **Only the 3D terrain view is genuinely still out** |

**Group total: 9 — 9 done** (2 of them `done*`).

### Phase 2 — Civilisation layer · `PHASE2_SCOPE.md`

Twenty-one milestones plus the superseded first milestone 9. All shipped and,
except where noted, reachable from the shell. `compute_civilisation()` in
`cartalith-godot/src/lib.rs` assembles the layer; `shell/engine_bridge.gd`
surfaces settlements / roads / sea_routes / provinces / trade_balances /
factions; `shell/viewport_host.gd` draws it.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| P2-01 | 1 — affordance fields (lithology, soil fertility, water access) | done | `cartalith_civ::{build_lithology, build_soil_fertility, build_water_access}`; `tests/golden_parity_affordance.rs` |
| P2-02 | 2 — water-body classification (`buildWaterBodies`) | done | `cartalith_civ::build_water_bodies`, consumed by `build_biome_raster` and by the settlement snap path (`civ_snap_land` takes `wb` + `lake_fill`) |
| P2-03 | 3 — biome classification | done | `cartalith_civ::classify_biome`, `build_biome_raster`; `tests/golden_parity_biome.rs` |
| P2-04 | 4 — carrying capacity, NPP, population density | done | `cartalith_civ::{build_carrying_capacity, build_npp, estimate_regional_density_km2}`; `tests/golden_parity_carrying_capacity.rs` |
| P2-05 | 5 — resource potentials (15 fields) | done | `cartalith_civ::build_resource_potentials`; `tests/golden_parity_resource_potentials.rs`. The predicted `WorldState` retention fix is real — `boundary_type`/`shear_field` are on `WorldState` |
| P2-06 | 6 — route corridors, landmass quality, coast SDF | done | `cartalith_civ::{build_route_corridors, build_landmass_quality, build_coast_sdf}` |
| P2-07 | 7 — settlement suitability / seed-finding | done | `cartalith_civ::{build_settlement_suitability, find_settlement_seeds, fresh_river_order, build_flood_field}` |
| P2-08 | 8 — settlement placement + faction assignment | done | `cartalith_civ::label_land_components`, the crate-private `civ_snap_land` / `civ_snap_coast` / `civ_is_coastal` (which since `c4435ee`, 2026-09-24, wraps x only on world maps — Ruling AR; before, a settlement on a non-wrapping map's edge could count as coastal from water on the opposite edge), `assign_landmass_factions`, and the public `place_settlements_with_water_edge_snap`. Unit tests pin the snap quirks. **`_civIterativeAutoWorld`'s centrality → tier loop added 2026-09-23** (it had never been ported; the one network build before it matched the reference's *first* pass, not its last): `cartalith_civ::{civ_iterative_network, civ_centrality_tier_feedback, civ_network_betweenness, civ_network_way_pairs, CIV_AUTO_WORLD_PASSES}`, called from `compute_civilisation` on the auto-populate path (one pass on the SG-02 keep path). `tests/golden_parity_centrality_feedback.rs` (20 cases, generated by `tools/civ_tier_feedback_capture.js` from the reference's own source text), `tests/iterative_network.rs`. Moves `kind` only — `capital` (the territory seat) stays as placement set it. **`wantCounts` (fixed per-tier counts) added 2026-09-23:** `cartalith_civ::{place_settlements_with_counts, civ_quota_tier, civ_want_counts_seed_params}`, read from `CivParams::want_counts()` (`civ.fixed_counts` + `civ.n_capital`…`civ.n_hamlet`) by `compute_civilisation` — `max_places` = total, the quota classifier replaces the rank cascade, thresh 0.35 and a total-derived `suppR` replace the two placement dials, one network pass, metropolis skipped. New World ▸ Generation carries the six controls (`new_world_dialog.gd::_build_fixed_counts`). `tests/golden_parity_want_counts.rs` (26 cases from the reference's own sliced text, `tools/civ_want_counts_capture.js`), `tests/iterative_network.rs` (real worlds), `params_mapping.rs::fixed_counts_are_off_unless_flagged_and_non_empty`, `_wantcounts_probe.tscn`. Flag off is byte-identical (probe hashes on 3 seeds match the pre-change DLL). Deviation: an all-zero request falls back to automatic where the reference alerts and places nothing. Pending independent verification. |
| P2-09i | 9 (first, renumbered) — investigation: territory/provinces is a dead end in the reference | unverified | A research finding about the reference HTML, not a deliverable. Superseded in-document by milestone 10; the 2026-08-19 correction notice is part of the record. Nothing in code to check |
| P2-09 | 9 — settlement population + naming | done | `cartalith_civ::{civ_default_culture, civ_name_rng, civ_settle_name, civ_base_pop_for_kind, name_and_populate_settlements}`, over `cartalith_rng::Mulberry32`. **Names follow the roster's faction culture since `c4435ee` (2026-09-24)**: the naming callers pass the roster's culture column through `civ_faction_culture`, which falls back to `civ_default_culture` (`CIV_CULTURES[faction % 7]`) only where no roster entry exists, as the reference's `civFactionCulture[faction]` does; before, an edited faction culture never reached a name (alignment audit Part 1 A2) |
| P2-10 | 10 — territory (cost-distance Voronoi from capitals, `DECISIONS.md` §7b) | done | `cartalith_civ::assign_territory`, reusing `road_dijkstra` and `build_travel_cost`. `engine_bridge.gd` exposes a territory texture; `viewport_host.gd` draws it beside `province_view` |
| P2-11 | 11 — road network (`buildTravelCost` / `roadDijkstra` / `buildRoadNetwork`) | done | `cartalith_civ::{build_travel_cost, build_road_network}` and crate-private `road_dijkstra`, with unit tests `road_dijkstra_flat_grid_diagonal_uses_sqrt2` and `road_dijkstra_impassable_water_stays_unreachable` |
| P2-12 | 12 — civ auto-populate road network (`_civHierarchicalNetwork`) | done | `cartalith_civ::civ_hierarchical_network_topology`; `tests/golden_parity_hierarchical_network.rs` |
| P2-13 | 13 — sea routes (`_civMstRoutes`) | done | `cartalith_civ::civ_sea_routes` → `WorldGen::get_sea_routes` → `engine_bridge.gd::sea_routes()` → **drawn** by `godot-project/map_overlay.gd` (`_sea_routes`, fed from `viewport_host.gd`'s `_bridge.sea_routes()`; toggled by `set_show_sea_routes`) — *re-pointed 2026-09-24: this cited `viewport_host.gd:1152`* — and in `civilization_workspace.gd`; also consumed by `place_search.gd` and `infrastructure_workspace.gd`. *The scope document's "not yet wired into rendering" is stale.* **Current/wind-costed as of 2026-09-01** — see JP-QC3 below |
| P2-14 | 14 — corridor consolidation + path smoothing | done | `cartalith_civ::civ_consolidate_and_smooth_ways`; `tests/golden_parity_road_consolidation.rs`. Output reaches the map through `get_roads` / `engine_bridge.gd`'s `roads()` |
| P2-15 | 15 — village seeding (`_civSeedVillages`) | done | `cartalith_civ::civ_seed_villages`; the toggle the milestone flagged as UI-less now exists — `new_world_dialog.gd`'s header names village seeding among the settings it owns; **and each village's road**: `_civConnectVillageAddons` (the dirt track the reference draws to every addon village, missed until the owner's 2026-09-23 report) is `cartalith_civ::tools::civ_connect_village_addons` over the new multi-source `road_dijkstra_multi`, wired into `compute_civilisation`; `tests/golden_parity_village_connect.rs`, `godot-project/_villageroads_probe.tscn` |
| P2-16 | 16 — provinces (`_civGenerateProvinces`) | done | `cartalith_civ::civ_generate_provinces`; `WorldGen::get_provinces` and `build_province_boundary_texture`; the overlay is assigned to `viewport_host.gd`'s `province_view` every redraw, and provinces feed `world_data_window.gd`. *Corrected 2026-09-24: this said provinces also feed `place_search.gd`; that file's header declines to index them (a province's position is its capital's, already a settlement row).* **Provinces follow painted territory since `c4435ee` (2026-09-24)**: `cartalith-godot`'s `civ_reprovince` re-runs `civ_generate_provinces` over `civ.territory` after `CivTools::rebase` merges the paint (and after a GeoJSON territory import), as the reference's `_civGenerateProvinces` reads the painted `civTerritory`. *The scope document's "deliberately not wired, no new `TextureRect`" is stale* |
| P2-17 | 17 — economy investigated, first slice ported | done | `cartalith_civ::civ_resource_trade_balance` plus `civ_world_mean_resources` / `civ_catchment_km2` / `civ_place_resource_context`; `WorldGen.trade_balances` → `civ_trade_bridge.rs` → `engine_bridge.gd` → `world_data_window.gd`. *The scope document contains both "not yet wired anywhere" and its own same-day retraction; only the second is true* |
| P2-18 | 18 — culture beyond naming (`_civCultureTerrainFit`) | done | `cartalith_civ::civ_culture_terrain_fit`, called inside a `#[func]` off a live `civ_faction_aggregates` result, surfaced in `shell/faction_roster_window.gd` (its header names the verdict explicitly) |
| P2-19 | 19 — Journey Planner milestone 1 (physical primitives + seasonal/closure logic) | done | `jp_fatigue` / `jp_load_penalty` / `jp_surface_gain` / `jp_can_use_wheels` and `jp_season_at` / `jp_rest_days` / `jp_seasonal_closure` / `jp_sea_closure`, imported by `cartalith-godot/src/journey_bridge.rs`, backing `shell/journey_planner_view.gd` (3 165 lines, with a real reroute control). *"Not wired to any caller … unstarted future work" is stale on both halves* |
| P2-20 | 20 — `_civFactionAggregates` | done | `cartalith_civ::civ_faction_aggregates`, called from `cartalith-godot/src/lib.rs` and `civ_military_bridge.rs`; `tests/golden_parity_faction_aggregates.rs`; reaches the UI through `faction_roster_window.gd`. *"No `#[func]`, no GDScript; all UI work is on hold" is stale twice over — the hold was lifted the same day it was called* |
| P2-21 | 21 — `_civSelectMetropolises` + `_civApplyRecovery` | done | `cartalith_civ::civ_select_metropolises` and `cartalith_civ::timeline::civ_apply_recovery`; `tests/golden_parity_metropolis_recovery.rs`. Surfaced in File ▸ New world ▸ Generation via `new_world_dialog.gd`'s `set_metropolis_enabled` / `set_recovery_phase`. **Roads survive recovery since `c4435ee` (2026-09-24)**: `cartalith-godot`'s `remap_after_recovery` / `remap_endpoints` re-point way endpoints onto the re-indexed settlement list and drop ways touching an abandoned site; before, a non-Stable recovery phase left every way's endpoint indices stale (alignment audit Part 1 A3) |

**Group total: 22 — 21 done, 1 unverified.**

### Journey Planner · `JOURNEY_PLANNER_SCOPE.md`

Engine-complete at 66 of the reference's 74 `jp*` functions (6 UI-only, 2 JS
idioms with no Rust function to write). Six engine milestones, five integration
steps, and four declared quality ceilings. **All four quality ceilings are now
closed** (JP-QC1 below, 2026-08-23; JP-QC2/QC3/QC4, 2026-09-01) — every place
this port's Journey Planner answer used to be deliberately the reference's own
answer on a world missing a layer now has that layer.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| JP-1 | Physical-modeling primitives + seasonal/closure cluster | done | `jp_fatigue`, `jp_load_penalty`, `jp_surface_gain`, `jp_can_use_wheels`, `jp_season_at`, `jp_rest_days`, `jp_seasonal_closure`, `jp_sea_closure` — 92 `pub fn jp_*` (`grep -c "^pub fn jp_"`, 2026-09-24; was 90) in `cartalith-civ/src/lib.rs` |
| JP-2 | Transport mode selection | done | `jp_auto_pick_transport`, `jp_vessel_matrix`, `jp_best_animal_for_context`, `jp_pick_species_for_route`; reachable through `jp_compute`'s `auto_carriage` key |
| JP-3 | Physical travel cost | done | `jp_calc_land_ex` (and the `jp_calc_land` wrapper), `jp_journey_cost`, `jp_calc_water_ex` via `JpVesselResolver` |
| JP-4 | Consumption / resupply | done | `jp_capacity_ex` (seasonal physiology, draft shortfall, saddlebag mass), `jp_foraging`, `jp_assess_resupply` |
| JP-5 | Route / stage derivation | done | `jp_ensure_plan`, `jp_plan`; `JpStageOverride` / `JpLeg` consumed by `journey_bridge.rs` |
| JP-6 | Verdict / reporting | done | `jp_verdict`, `jp_confidence`, both flattened by `cartalith-godot`'s `jp_compute` |
| JP-INT-1 | Integration 1 — a route to plan (waypoint capture + route readback) | done | `WorldGen::route_count` / `route_get`; `route_begin` / `route_append_stop` / `route_commit` in `infra_tools_bridge.rs`; caller `shell/journey_planner_view.gd` |
| JP-INT-2 | Integration 2 — a `JpWorld` assembled from live state | done | `journey_bridge.rs::JourneyWorld`, building `cart_biome` / `cart_terrain` / `jp_road_cells` at call time from existing rasters |
| JP-INT-3 | Integration 3 — the party form | done | `shell/journey_planner_view.gd` — the in-shell distance-spine takeover (28 plan fields, per-stage overrides, Travel Library pickers); replaced the deleted `journey_planner_window.gd` dialog |
| JP-INT-4 | Integration 4 — `#[func]`s over the boundary | done | `jp_options`, `jp_default_plan`, `jp_compute`, `jp_reroute`, `jp_plan_for_route` |
| JP-INT-5 | Integration 5 — the presentation the port left out | done | `journey_planner_view.gd` — calculation trace group (`∏ factor == daily_km`), stops strip, elevation/segment drawing, vessel matrix, campaign-duration advisory |
| JP-REROUTE | `_jpRerouteForMode` — the one remaining engine gap | done | `cartalith_civ::jp_reroute_for_mode`, exposed as `WorldGen::jp_reroute` with `jp_reroute_mode(transport, force_mode)` sizing `RouteInputs` |
| JP-06/08 | Named journeys persisting across save/load | done | The named journey is SP-1's engine `Journey`, persisted in the engine-owned `entities/journeys.json` slot (`project_bridge.rs::SLOT_JOURNEYS`) and drawn on the map after a reopen. *Corrected 2026-09-23:* this row cited `journey_planner_view.gd::journeys_document()` / `restore_journeys_document()`, which have been stubs since SP-1 (`journeys_document()` returns `""`; the restore only clears). Since `a64ffad` (2026-09-24) the planner's own Journeys list is rebuilt from them on project open (`journey_planner_view.gd::_restore_from_engine`, proven by `_jprestore_probe.gd`); per-stage overrides, layovers, animal choices and trim were never persisted and restore at their defaults — Ruling AR (2026-09-24) rules that the full plan be saved; not built |
| JP-CONF | Route-planner conformance re-check — `_jpEnsurePlan` reaching the shell | done | `cartalith_civ::civ_path_water_frac`; `WorldGen::jp_plan_for_route`; probe `godot-project/_routeplanner_probe.gd` |
| JP-QC1 | Quality ceiling — wildlife richness feeding `jp_foraging` | done | `cartalith-civ/src/wildlife.rs` ports `buildTRI` / `guildTrophic` / `build_ecoregions` / `region_richness` / `assign_wildlife` in full, with `golden_parity_wildlife.rs`; `cartalith-godot/src/lib.rs` passes a real `forage_mod` from `sample_bridge::WildlifeCache`. *(The scope document and `journey_bridge.rs`'s module doc no longer call it unported — corrected 2026-09-23)* |
| JP-QC2 | Quality ceiling — ocean-current / wind coarse fields reaching the sea-lane router and `jp_sea_condition` | done | **Built 2026-09-01; the "blocked" framing this row used to carry was itself wrong** — no `WorldState` retention was ever needed. `cartalith_climate::current_wind_field`/`current_ocean_field` already existed as callable `pub fn`s, already used (deliberately uncached) by the Wind/Ocean-currents debug views' `sample_bridge::flow_fx_raster`. `coarse_ocean_wind_fields` (`cartalith-godot/src/lib.rs`) mirrors that same recipe; both `jp_plan`-driving `#[func]`s — `jp_plan_for_route` and `jp_compute` — now pass `Some(&JpCoarseField)` for `ocean_field`/`wind_field`, not `None`. The `cartalith-civ` consumer side (`jp_sea_condition`, `JpWorld::ocean_field`/`wind_field`) was already complete and already golden-tested (`m5_sea_condition_reads_the_real_wind_and_current_and_zeroes_an_oared_hull`) — the whole gap was two `None`s at the Godot boundary *(Line citations dropped 2026-09-24 — they had drifted by 1 500-2 000 lines; the symbols are the citation.)* |
| JP-QC3 | Quality ceiling — `_civSeaTimeEdgeCost` (current/wind-costed sea lanes) | done | **Built 2026-09-01, reversing the prior decline.** `civ_sea_time_edge_cost` (`cartalith-civ/src/lib.rs`) plus `CIV_LANE_REF_VESSEL`/`CIV_LANE_CURRENT_W`/`CIV_LANE_TACK_FLOOR` (reference lines 21197/21198/21203). `road_dijkstra` gained an additive `edge_cost: Option<&dyn Fn(usize,usize,isize,isize)->f64>` parameter — every pre-existing call site still passes `None`, bit-identical. `civ_sea_routes` takes `ocean_f`/`wind_f` and reproduces the reference's own passability wrap: an edge with either endpoint impassable in the land/lake cost grid never reaches the costed callback, so a strong current cannot make water sailable. Wired via `coarse_ocean_wind_fields` from both `compute_civilisation` callers, `absorb` (`cartalith-godot/src/lib.rs`) and `recompute_civilisation`. Golden-tested: `civ_sea_time_edge_cost_is_none_without_any_field_and_penalises_a_current_aligned_edge`, `civ_sea_routes_still_connects_ports_with_or_without_current_and_wind_fields` *(Line citations dropped 2026-09-24 — they had drifted by 1 500-2 000 lines; the symbols are the citation.)* |
| JP-QC4 | Quality ceiling — `jp_road_cells` seeing hand-drawn (manual) ways | done | **Built 2026-09-01.** `jp_road_cells` (`cartalith-civ/src/lib.rs`) now takes a `manual_ways: &[tools::ManualWay]` parameter and applies the reference's `'ancient' -> ["Dirt Track","Deteriorated"]` mapping plus the manual Road/Track tuple, filtering `SeaLane`/hidden. Wired at both Godot call sites (inside `jp_plan_for_route` and `jp_compute`, `cartalith-godot/src/lib.rs`) via `self.infra.as_ref().map_or(&[], \|t\| &t.ways)`. Golden-tested: `jp_road_cells_reads_hand_drawn_ways_including_ancient` *(Line citations dropped 2026-09-24 — they had drifted by 1 500-2 000 lines; the symbols are the citation.)* |
| JP-PARTY | Widen `JpParty` from four fixed species to a generic animal map | declined | `jp_capacity_ex`'s sums are pinned to `JP_ANIMAL_KEYS` and read `jp_seasonal_animal` / `jp_desert_animal_mod` per species; the shipped alternative is `travel_library.rs`'s "substitutes for" path. Declined in the document and matched by the code |

**Group total: 19 — 18 done, 1 declined.**

### Travel Library · `TRAVEL_LIBRARY_SPEC.md`

**Added 2026-09-24** (alignment audit Part 1 E36: the spec had no group here).
The spec carries no milestone numbering; the IDs below follow its sections so
the rows can be referred to. Each was opened at its symbol.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| TLS-1 | §2 — placement: its own window, tabbed by definition type | done | `menus.gd::_data` — `_live(p, "Travel library…", ID_TRAVEL_LIBRARY, KEY_MASK_SHIFT \| KEY_L)`, a single row rather than the spec's submenu (the spec's own port note records the choice); `shell/travel_library_window.gd`'s four tabs `animal` / `vehicle` / `vessel` / `preset` |
| TLS-2 | §3 — the four definition types and their fields | done | `WorldGen::{tl_list, tl_get, tl_add_blank, tl_edit, tl_reset_to_stock, tl_capture_preset_from_plan}` over `cartalith_civ::travel_library` |
| TLS-3 | §4 — validation (ok / incomplete / conflicting) and usage | done | `travel_library::ValidationState::{Ok, Incomplete, Conflicting}`, surfaced per row by `tl_list` and by `travel_library_window.gd::_build_validation_banners`. Known wording defect: the banner's "re-plans N saved journeys" is also shown for vehicles and slotless animals, which no `Journey` references (`tl_list`'s doc: `usage_journeys` stays 0 for them) — alignment audit Part 1 B15 |
| TLS-4 | §6.2 — a definition reaches a computed journey | done | The spec's §6.4 records the 2026-08-20 measurement that a custom animal re-plans a journey; `JpParty` stays four fixed species with substitution (JP-PARTY above, declined) |
| TLS-5 | §2's *Import definitions .csv* | done | `c1e0a2a` (2026-09-24): `travel_library_window.gd::import_csv` — row 1 names `tl_get` field keys, each later row becomes an entry through `tl_add_blank` + `tl_edit` and is removed again if refused; probe `godot-project/_tlcsv_probe.gd` |

**Group total: 5 — 5 done.**

### Economy and trade · `ECONOMY_SCOPE.md`

This document carries no milestone numbering — its work sits in prose sections
and a "real next milestones" list. The IDs below are assigned here so the rows
can be referred to.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| EC-1 | Pass 1 — `civ_resource_trade_balance` + catchment resource context, wired end to end | done | `cartalith_civ::{civ_resource_trade_balance, civ_world_mean_resources, civ_catchment_km2, civ_place_resource_context}`, called per settlement in `compute_civilisation`, exposed as `get_trade_balances()`, wrapped by `engine_bridge.gd::trade_balances()` and read by `civilization_workspace.gd` and `world_data_window.gd` (*corrected 2026-09-24: this said three workspaces; `grep -rn "trade_balances()"` over `shell/` finds these two readers*) |
| EC-2 | `_civPlaceSmelting` | done | `trade.rs` defines `civ_place_smelting` with a three-case golden suite; committed in `4ec07f5` |
| EC-3 | Food-surplus cluster — `_civFoodShed` / `_civPlaceFoodSurplus` / `_civPlaceCatchmentCeiling` / `_civCatchmentPop` | done | **Crate-complete AND Godot-wired as of 2026-09-01 (second pass), with the UI display gap now closed.** All four are real: `civ_food_shed` (`trade.rs`, a direct port of `_civFoodShed`/`_civFoodConnected`/`_civRoadConnected`/`_civFoodMode`/`_civFoodDeliverable`/`_civGoodReach`, distinct from `trade.rs`'s separate, pre-existing 15-good trade match, which excludes `food`); `civ_place_food_surplus`/`food_surplus_ratio` (unit-tested this pass); `civ_catchment_pop` (`timeline.rs`). The chain: `civ_trade_bridge.rs`'s private `food_shed_rows()` builds one shared `RoadComponents` and calls `civ_food_shed` once per settlement, resolving `farmers_per_urbanite` through `civ_ag_tech_by_key`; the `#[func] civ_food_shed` reads it out; `engine_bridge.gd::civ_food_shed()` and `trade_store.gd` (`_food_shed` cache, `food_shed()`/`food_shed_for(index)`, populated by `refresh()` alongside `civ_trade_flows`) complete the chain to the shell. The existing "Match trade flows" button (`infrastructure_workspace.gd`, `_match_trade_flows`) already triggers it. **The UI gap is closed:** `place_editor_window.gd` now reads `TradeStore.food_shed_for(_index)` in the Trade tab. Verified: `cargo test -p cartalith-godot --lib` 409/409 after a fresh build, `cargo check --workspace` clean (2026-09-01). *Line citations dropped 2026-09-24 — every one had drifted; the symbols are the citation* |
| EC-4 | `_civFactionAggregates` itself | done | `cartalith_civ::civ_faction_aggregates` with three live callers; `civ_ocean_dist_field` and the `CIV_TAX_RATE` / `CIV_PRIMARY_SPECIALISATION` tables ported alongside |
| EC-5 | The Journey Planner as the economy layer's consumer | done | Tracked above as JP-1…JP-INT-5 |
| EC-6 | `civ_culture_terrain_fit` genuinely callable | done | Called inside a `#[func]` returning key/value/world_mean/ratio/verdict per faction; consumed by `faction_roster_window.gd`'s "Territory fit" panel. *The scope document says "still deliberately not exposed … all UI work is on hold" — the hold was lifted the same day it was called* |
| EC-7 | `_civSaltAccess` | done | `trade.rs` defines `civ_salt_access` with a three-case golden suite; committed in `4ec07f5` |
| EC-8 | `_civFactionAggregates`' resource- and density-fed half, surfaced as a readout | partial | *Corrected 2026-09-24 (alignment audit Part 1 C19): this row said `blocked` on a memory decision; the blocker was worked around in `45b368d`.* **Built:** `WorldGen::civ_faction_economy` (`cartalith-godot/src/lib.rs`) feeds the aggregate both rasters — resources rebuilt on demand, density from the retained `CivData::dens` — and returns territory, pop, food capacity/surplus, the fifteen resource means, strategic, exports/imports; wrapped by `engine_bridge.gd::civ_faction_economy` and drawn by `civilization_workspace.gd::_fill_faction_economy` (Economy ▸ By faction). **Not met:** it returns empty for a reopened project (it requires `WorldSource::Generated`; Ruling AR's raster save is not built), tax income is drawn nowhere, and the Faction Roster — where the old on-screen note lived — does not show it (`GUI_GAP_REGISTER.md` FR-03) |
| EC-9 | Military manpower as the economy layer's first real consumer | done | `cartalith-civ/src/manpower.rs` reads `civ_current_agrarian_density`, `civ_faction_aggregates`, `civ_catchment_pop`'s tiers and `RoadComponents`/place navigability; surfaced in `civilization_workspace.gd`'s "Military" category |
| EC-10 | IN-13 — trade flows between settlements (`GUI_GAP_REGISTER.md` §42/§43; Rulings AB, AE, AF, AP) — **row added 2026-09-23; there was none** | partial — 3 of 4 pieces built | **Match and network flow** shipped on 2026-08-25 under `GUI_GAP_REGISTER.md` §42: `cartalith_civ::trade::trade_flows`, routed by the private `WayRouter` in `trade.rs`, probe `godot-project/_in13_probe.gd`. **Scarcity prices and tariffs** landed in `bbc255f` (2026-09-23): each flow carries `price` (Ruling AB); there is a directional `Tariff` (Ruling AE) on the importer's roster row (`civ_roster_bridge.rs` `tariffs`); and `civ_trade_bridge.rs` exposes `civ_set_trade_tariff` / `civ_trade_tariff`. **Caravans on land and river ways are built** (`da51a57`, 2026-09-24): Ruling AP's derived view, one per way with trade load, nothing persisted — `TradeNetwork::way_goods`, `civ_trade_flows`' `caravans`, the CIVIL ▸ Trade Caravans group, probe `_caravan_probe.gd`. The sea-lane half is **ruled and not built**: Ruling AQ (2026-09-24) answered the question Ruling AP left open — one caravan per sea lane, the same derived view (*corrected 2026-09-24: this said the question was still open*); `OUTSTANDING_WORK.md` §2.3 has the row. Tariffs have no GUI. Ruling R's per-faction currencies (re-affirmed by Ruling AR) are not built |

**Group total: 10 — 8 done, 2 partial.** EC-10 added 2026-09-23; EC-8 moved blocked → partial 2026-09-24.

### Military manpower · `MILITARY_MANPOWER_SCOPE.md`

The scope document carries the owner's supplied specification verbatim, because
the reference has no model to check it against — so parity is not the bar here
and the era table is.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| MM-1 | The four outputs (standing / field / emergency armies, sustainable war duration) from five variables | done | `cartalith-civ/src/manpower.rs` — `Manpower::standing_army` / `field_army` / `emergency_mobilization`, with `Drivers` (`food_surplus_per_farmer`) and `government_extraction`; reachable via `cartalith-godot/src/civ_military_bridge.rs` |
| MM-2 | The era as an output, with the table's bands as an on-screen sanity check | done | `manpower.rs::EraBand { standing: (f64,f64), … }` with per-era rows; verdicts rendered by `civilization_workspace.gd::_manpower_tooltip` |
| MM-3 | §1a/§2.6 — citizen / free population as the band's denominator (owner ruling, 2026-08-25) | done | `manpower.rs` states the ruling in source; `citizen_share(key)`, `CITIZEN_MODERNISATION`, `Manpower::citizen_population`. The document's claim that "the four outputs do not move" is matched by the code: the fraction is a denominator only |
| MM-4 | CIVIL ▸ Military panel, including the "Not built" disclosure in the same words | done | `civilization_workspace.gd` — `_military_body = DccWidgets.category(self, "Military", categories)`, `_fill_manpower(parent, factions)`, per-faction manpower dicts read at five sites |
| MM-5 | CV-25's fortification axis and `power.military` left as they are | declined | `cartalith-civ/src/military.rs` untouched by the manpower pass; the golden-verified `0.45·normPop + 0.35·fortifiedFraction + 0.20·capitalTierNorm` composite still feeds `civ_faction_aggregates`. Reason recorded in `civ_military_bridge.rs`'s module doc |
| MM-6 | Per-settlement garrisons | declined | `manpower.rs` produces per-faction headcounts only — no settlement-keyed output type exists. Disclosed on screen in CIVIL ▸ Military ▸ Not built |
| MM-7 | Campaigns, unit movement, combat resolution | declined | No combat or campaign type anywhere in `cartalith-civ`; still true after SP-4: `cartalith_civ::conflict` (2026-09-23) annotates a drawn conflict with authored free-text outcome and reads manpower, and resolves nothing (`STORY_PLANNING_SCOPE.md` §5) |
| MM-8 | Change over time (manpower across the year cursor) | declined | `manpower.rs` takes no year argument and `TimelineSnapshot` carries no manpower field; the model reads the world as it stands, as §4 states |
| MM-F2 | Finding 2 — the standing column; owner ruling AI (c), 2026-09-23: soldier upkeep per agricultural-labour bracket, derived from the era table | done | Committed `dda315e` (2026-09-23; corrected the same day from "done (not yet committed)"). `manpower.rs::SOLDIER_UPKEEP_BY_BRACKET` / `soldier_upkeep` / `alpha_bracket` (shared with `era_for`), replacing the flat `SOLDIER_UPKEEP = 3.0`. Re-derived from `ERA_BANDS` + `era_for` + `GOVERNMENT_EXTRACTION` + `CITIZEN_SHARE` by `soldier_upkeep_is_derived_from_the_era_table`; the Iron-Age-above-High-medieval pair pinned by `a_median_polity_of_each_bracket_lands_in_its_own_band`. **Re-baselines the worked example's standing army** (A 5 846 → 9 661, B 19 067 → 25 750; levy and field unchanged), accepted by the owner. Open: Kingdom B's standing now exceeds its own 365-day rung by 6.7 % (`the_force_ladder_decreases_with_duration`) |
| MM-F3 | Finding 3's residue — `ecological_factor` tracked map area; owner ruling AI (b), 2026-09-23: normalise land per person to the world's own | done | Committed `dda315e` (2026-09-23; corrected the same day from "done (not yet committed)"). `manpower.rs::civ_military_manpower_world` / `world_land_reference`, called by `civ_military_bridge.rs::manpower_rows`. Pinned by `map_scale_does_not_move_the_ecological_factor` (land ×6.25 → identical outputs). Measured on `_mpscale_probe.tscn`: standing below-band 21/33/11 of 36 on the 1 200/800/2 000 km shapes before, 17/20/17 after (b)+(c). Open: the 0.25 floor now binds on 36 of 108 faction-samples |

**Group total: 10 — 6 done, 4 declined.**

### Story planning · `STORY_PLANNING_SCOPE.md`

One subsystem over the Timeline's year cursor. It carries the owner's three
2026-08-25 forks; owner decisions 1 and 3 above, which gated it, were both answered 2026-09-23 by Ruling AO.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| SP-1 | The Journey entity | done | **Re-checked at the symbols 2026-09-23 (SP-2's build); the old row here was stale.** It said `grep "pub struct Journey"` found nothing and that journeys were a GDScript dictionary. Both are now false. `cartalith_civ::travel_library::{Journey, JourneyRoute}` exist. The store is `InfraTools::journeys` (`journey_save`/`journey_delete`/`journey_get`), exposed as the `#[func]`s `journey_save`/`journey_list`/`journey_get`/`journey_delete`, and populated by the live shell: `journey_planner_view.gd::_save_journey` calls `bridge.journey_save` with a preset captured from the form. The entity persists as `entities/journeys.json` (`project_bridge.rs` `journey_to_dto`/`dto_to_journey`, test `journeys_round_trip`). Preset usage is real: `preset_usage_in_journeys` feeds `tl_list`'s rows in `lib.rs`. `animal_usage_in_journeys` is still `0`, correctly: a `Journey` names a preset, not an animal. **Not part of SP-1, and still session-only:** the planner's richer per-journey working state (`stage_overrides`/`layovers`/`animal_entries`/`trim` in `_journeys`) |
| SP-2 | Journey progression over the cursor | done | **Built 2026-09-23 against the engine `Journey`, which is the live one (SP-1 above).** The date: `cartalith_vault::chronos` gains a 365-day calendar (`MONTH_DAYS`, `MonthDay`, `day_number`) and `Event.start_md`/`end_md`; the year-only path is byte-identical. The cursor gains a day (`WorldGen::civ_day`, `get_civ_date`/`civ_set_day_of_year`). The maths is `cartalith_civ::journey_progress::JourneyTimeline::{from_plan, at}` over `jp_plan_full`'s own plan, with supply from the shared `jp_leg_supply`; `sp2_progression_reads_the_planners_own_golden_journey` pins it to the m5 golden. Live: `story_bridge.rs::journey_positions`; `map_overlay.gd` markers carry a food-left and arrival readout; a day slider sits in the timeline strip (row 3b). Regenerate re-snap: `resnap_journey` / `adopt_resnapped` / `resnap_carried_journeys`. Windowed probe `_sp2journey_probe.gd`. **Left:** departure is always 1 January of `start_year`, because the entity has no finer start and widening it is a format change. *Corrected 2026-09-24:* this said there was no timeline cache and quoted `journey_positions` at 48–54 ms per call at 2048×1311; that was the figure `b5aff17` (2026-09-23) fixed — `story_bridge.rs::JourneyPlanCache` keys each saved journey's plan on a content hash of every input `plan_saved_journeys` hands `jp_plan_full`, so a warm call re-plans nothing (the commit reports ~12×; not re-measured here). The planner's own Journeys list still clears on a regenerate (`journey_planner_view.gd` connects `generation_finished` to `clear_journeys`) while the engine journey survives; on project open it is rebuilt (`_restore_from_engine`, `a64ffad`) |
| SP-3 | The settlement timeline strip | partial | *Moved done → partial 2026-09-24 (alignment audit Part 1 C25): the `ruins`/`fortified` piece does not survive a save — `TimelineSnapshot::collapse_flags` is in memory only (its own doc: `project_bridge`'s `TimelineYearDto` does not carry it), so every reopened year reads as not recorded; and the authored-events list is not read-only as this row said — `place_editor_window.gd::_build_add_event_form` writes a Chronos line into the note.* All four pieces were built 2026-09-23. Faction ownership periods (`civ_settlement_ownership_periods`, `_build_political`); population/tier per recorded year (`civ_settlement_population_trajectory`); **authored events** — a ` ```chronos ` block in the settlement's vault note, Chronos Timeline syntax exactly (Ruling AM), parsed by `cartalith_vault::chronos::parse`, read by `VaultSession::entity_chronos`, exposed as `vault_entity_chronos`, drawn by `place_editor_window.gd::_build_authored_events`, with an *Add event* form beneath it (`_build_add_event_form`) that appends a Chronos line to the note; **journeys passing** — `civ_passed_settlements_at`/`JourneyTimeline::elapsed_at_point`/`civ_settlement_journey_passes(tid)`, a "Journeys passing" section on the Political History tab, probe `_sp3journeypass_probe.gd`; **`ruins`/`fortified` per year** — `TimelineSnapshot::collapse_flags: BTreeMap<tid, CollapseFlags>`, a side table (not new `NamedSettlement` fields — measured blast radius, see `OUTSTANDING_WORK.md` §2.3's SP-3 row), written only by `run_collapse_simulation`, a fourth trajectory column, probe `_pestatus_probe.gd` — **not persisted** (see above). Four separate lists/columns on one tab, not one interleaved strip — Ruling AQ (2026-09-24) rules the combined strip be built; not built |
| SP-4 | The conflict overlay | done | 2026-09-23. `cartalith_civ::conflict` (`Conflict`, `ConflictKind` front/arrow/siege/battle, `ConflictAnchor` settlement-or-province keyed on `tid`, `ConflictStore`, `side_manpower`), `cartalith-godot/src/conflict_bridge.rs` (`conflict_add/update/delete/list/get`, `conflicts_attached_to`, `conflict_sides_manpower`), `WorldGen::conflicts`, saved in engine-owned `entities/conflicts.json` (`SAVEFILE_COMPAT.md` §9.7). Shell: CIVIL Conflict tool (Way/Route vocabulary), right-dock `CTX_CONFLICT` form, CIVIL ▸ Military ▸ Conflicts list, `map_overlay.gd::_draw_conflict`. Windowed probe `_sp4conflict_probe.gd` 48/48. **Manpower is the world now, not the conflict's year** (disclosed in the dock); year-scoped read, map-click selection and re-drawing a committed shape not built. Lifecycle decisions: Ruling AO addendum |
| SP-5 | The planning aid, joined up | done | 2026-09-23, same day its blocker (two of SP-1…SP-4) went stale. Both `STORY_PLANNING_SCOPE.md` §4 joins built: `WorldGen::conflicts_touching_settlement(tid)` (`anchors_touching` — this settlement's own conflicts plus its province's, read live) feeding a "Conflicts here" section on the Political History tab; each dated journey-pass row annotated when its year falls inside one of those conflicts' active range (no new Rust needed — both values already had year precision). Probe `_sp5joined_probe.gd` confirmed to pin the change (24/24 current, 11/24 pre-change). |

**Group total: 5 — 4 done, 1 partial.** Story planning was built in one session 2026-09-23; SP-3 moved to partial 2026-09-24 because its collapse flags are not saved.

### Markdown Vault · `MARKDOWN_VAULT_SCOPE.md`

Seven milestones (0-6). The entity audit that opened this work found that
continents did not exist as entities; milestone 0 created them.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| MV-0 | 0 — the addressable continent | done | `cartalith_civ::Continent` and `civ_continents(...)` with its own `civ_continent_name_rng` stream; exposed as `WorldGen::get_continents()`; read by `engine_bridge.gd` and turned into a vault-linkable row by `civilization_workspace.gd::_knowledge_row(cg, "continent", …)` |
| MV-1 | 1 — link, read, section-aware write-back | done | Crate `cartalith-vault` (`backlinks.rs`, `block.rs`, `export.rs`, `links.rs`, `markdown.rs`, `provider.rs`, `template.rs`); `cartalith-godot/src/vault_bridge.rs` carries 53 `#[func]`s (`grep -cE "^\s*#\[func\]"`, 2026-09-24; was 47), plus `vault_saf.rs`'s one; panels in `shell/vault_window.gd` (`_build_connection`, `_build_attach`, `_build_links`, `_build_reader`) and persistence in `shell/vault_store.gd` |
| MV-2 | 2 — the map snapshot (§21, §22) | done | `cartalith-vault/src/export.rs` implements the map snapshot; committed in `4ec07f5` |
| MV-3 | 3 — project-scoped links (§26) | done | **The defining document files this as *blocked*; the blocker has lifted.** The save format carries a civ layer: `project_bridge.rs` defines `SLOT_VAULT = "vault.json"`, writes `self.vault.store.to_json()` into the project's documents and restores it via `LinkStore::from_json`, and the same tree carries `entities/settlements.json`. The shell half is now wired: `shell/vault_store.gd` and `vault_bridge.rs` register the project-scoped `vault.json` slot; committed in `4ec07f5` |
| MV-4 | 4 — the Android provider (§6) | partial | *Corrected 2026-09-24 (alignment audit Part 1 C20): this row said `not started` and searched only `provider.rs`.* **Built (`cff1edc`, 2026-09-02):** `cartalith-godot/src/vault_saf.rs::SafVaultProvider`, a `VaultProvider` that delegates every operation to a GDScript dispatcher `Callable` (Rust has no JNI path to `content://` URIs), connected by the `#[func]` `vault_connect_saf` and wrapped by `engine_bridge.gd::vault_connect_saf`; `_vaultsaf_probe.gd` drives it with a stand-in dispatcher. **Not built:** the Android half — no shell `_saf_dispatch` handler, no caller of `vault_connect_saf` outside the wrapper and the probe, no tree-URI picking or persisted grant — and nothing is device-verified |
| MV-5 | 5 — the conflict UI (§14's *Compare*) | done | Built 2026-09-01: `vault_window.gd`'s `_compare_link()`/`_compare_dialog()`/`_lcs_diff()`/`_build_diff_rows()` — an O(n·m) LCS diff between the on-disk file and the working copy's own preview, deliberately calling `vault_read_file`/`vault_preview_section_write` rather than `vault_reload_link`, so opening Compare cannot itself clear a Stale status. §14's three-way prompt (Reload source / Keep current / Compare…) is now complete. Dynamically verified end to end (edit externally → Stale → Compare shows the real diff without clearing Stale → Reload clears it) |
| MV-6 | 6 — search, the note as data, culture, and "confirm always" | done | **All four panel pieces are built**: search (`vault_window.gd::_build_search()` over `engine_bridge.gd`'s `vault_search`), the "note says" readout (`_build_note_data()` / `_build_entity_data()` over `vault_file_data` / `vault_link_data`), and the three don't-ask-again checkboxes (`_build_write_prefs()` over `vault_write_prefs()`). **The culture picker is built too** (corrected 2026-09-23: this cell said it was missing). `civilization_workspace.gd` calls `_knowledge_row(sec, "culture", …)` over `get_cultures()`, in the tree since 2026-09-01 (`fd9de7c`, per `git log -S`). The "record defect" this cell pointed to, a shell string claiming `get_cultures()` did not exist, was closed on 2026-08-25 (see *One shell string that used to lie to the user* below) |

**Group total: 7 — 6 done, 1 partial.** MV-6 corrected from partial to done 2026-09-23; MV-4 from not started to partial 2026-09-24.

~~MV-3 is the row to watch … It is not-started, not blocked~~ — **superseded
2026-09-06.** MV-3 **shipped 2026-09-02** in `4ec07f5`, and this table's own MV-3
row has said `done` since. This paragraph was left behind when the row was
updated, so the table and the prose beneath it contradicted each other for four
days. `MARKDOWN_VAULT_SCOPE.md` was itself corrected on this point on 2026-09-04
(`52666b9`). ~~**Still true and still worth doing:** `vault_store.gd` and
`vault_bridge.rs` recite the retired blocker in source.~~ *Stale, re-checked
2026-09-24:* both files now quote that blocker only as a corrected claim (their
headers say it was false since 2026-08-25). **One copy survives:**
`cartalith-vault/src/links.rs`'s identity table still says a settlement's `tid`
is not stable across save/load because "civ is not saved" — false since the
project tree (`DECISIONS.md` §7h) persists `entities/settlements.json`. That is
a code comment, recorded in `ALIGNMENT_AUDIT.md` Part 1 C26 and routed from `OUTSTANDING_WORK.md` §2.11.

### Landmark generation · `LANDMARK_GENERATION_SCOPE.md`

Nine milestones. Seven of the nine are done and two are partial (LM-5, LM-9),
as of 2026-09-23. *(Corrected 2026-09-24: this paragraph quoted the scope
document's §0 "No code was written for this pass" and §3 "Nothing below is
started" as still standing there; the cleanup removed both, and a grep of
`LANDMARK_GENERATION_SCOPE.md` for either finds nothing.)*

`crates/cartalith-civ/src/landmark.rs` is **8 138 lines** (`wc -l`,
2026-09-23; it was 3 730 at first ship): 49 kind specs (`LandmarkKindSpec { key:`
lines), of which **27 are `buildable: true`** and 22 carry a `not_built:` reason
(recounted 2026-09-24, unchanged). **The `needs_viewshed` flag and the viewshed
reads do not match** (re-checked 2026-09-24, alignment audit Part 1 C24; this
said "6 carry `needs_viewshed: true`, and since LM-7 there is a real viewshed
behind them"). Six kinds carry the flag — `peak`, `volcanic_feature`, `fort`,
`watchtower`, `border_marker`, `sacred_mountain` (unbuilt). Six kinds read
`Derived::vis` — `fort`, `watchtower`, `fortified_pass`, `fortified_crossing`
(all through `pool_military`), `volcanic_feature` (`pool_volcanic`) and
`border_marker` (`pool_border_marker`). So **`peak` is flagged and reads no
viewshed**, and **`fortified_pass` / `fortified_crossing` read it unflagged**.
(Corrected 2026-09-23: this paragraph still read 14 buildable, 35 blocked and
"no implementation behind" the viewshed flag.)

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| LM-1 | M1 — extract the analytical field library (Category A) | done | `cartalith-terrain/src/analysis.rs`, 1 056 lines (`wc -l`, 2026-09-24; was 725 when this row was written; `visibility` has since been added) — `slope`, `aspect` (the gap M1 existed to close), `curvature` / `curvature_at`, `tpi` / `tpi_multiscale`, `local_relief`, `ruggedness`, `normalise` — all `Vec<f32>` and resolution-scaled, with the §31 Category A notice in the module doc. **The consolidation half was declined in source** with the reason given: `build_ao` stays bit-identical because `DECISIONS.md` §7a protects rendered output. Reachable: `landmark.rs` calls `analysis::tpi` |
| LM-2 | M2 — hydrological candidates: waterfall, ford, confluence | done | `kinds()` marks `waterfall`, `ford`, `river_confluence` buildable, plus `spring`, `lake`, `gorge`, `cliff`, `harbour`; `LandmarkInputs` takes flow/channel/recv/order/water. Fixtures shaped to reach the code: `each_waterfall_constraint_is_load_bearing`, `dropping_the_strahler_field_still_places_confluences` |
| LM-3 | M3 — mountain-pass candidates | done | `LandmarkInputs::corridors` — "`build_route_corridors` … Mountain passes read this and nothing else can substitute for it"; `mountain_pass` buildable; the §8 `S_pass` weights are named constants. The 2D saddle half was **declined in source**: `saddle`'s `not_built` reads "A saddle with connectivity is a mountain pass, which is generated; a saddle without it is a shape, not a landmark" |
| LM-4 | M4 — peak / ridge / prominence candidates, generalised to 2D | done | `peak` and `ridge` buildable; the pass reads M1's extracted field through `Ctx::tpi_broad` built from `analysis::tpi` at the broad-scale radius; the two-cone fixture drives the spacing tests |
| LM-5 | M5 — resource- and settlement-linked candidates | partial | **The resource half is built and reachable**: `mine` and `quarry` buildable, driven by `MINE_RESOURCES` (8 keys) and `QUARRY_RESOURCES` (4 keys) over `LandmarkInputs::resources`, assembled by `WorldGen::landmark_resource_pairs`. **The accessibility half is still absent for mine and quarry**: their "settlement access" term is `Ctx::influence`, straight-line Euclidean gravity over a wrap-corrected distance, so a resource cell with real road access scores the same as one with none (re-read 2026-09-23). **Corrected 2026-09-23:** this cell also said `LandmarkInputs` had no roads/ways field and that `market_site`, `trade_depot` and `caravan_station` were `not_built`. `LandmarkInputs::ways` has existed since 2026-09-02 (`45b368d`), and all three, with `bridge_site` and `road_junction`, are `buildable: true` in `kinds()` |
| LM-6 | M6 — spatial competition / Poisson-disc filtering | done | `landmark.rs::Buckets` / `Buckets::new(gw, gh, world, max_radius)`, used with a shared cross-type field. **Bridson (2007) was deliberately declined**, with the argument and the 0.866·r² vs π·r² packing measurement in the doc comment. Mutation/boundary tests present: `spacing_rejects_the_weaker_of_two_candidates_inside_one_radius`, `at_cap_and_spacing_are_different_answers`, `crowding_higher_packs_tighter`, `a_zero_or_nan_crowding_does_not_take_the_map_with_it`, `cross_type_competition_changes_the_answer`, `a_placed_landmark_is_never_inside_its_own_exclusion_radius` |
| LM-7 | M7 — viewshed (the expensive one, entirely new) | done | 2026-09-21, `222189f`: a real line-of-sight primitive, `cartalith_terrain::analysis::visibility` with `ViewObserver`, sized by a disclosed conservative default rather than an owner number. **Ruling AP (2026-09-23) ratified that default.** *Not built (re-checked 2026-09-24):* ruling 16's manual "recompute and refine" action — no refine path exists in `landmark.rs`, `landmark_bridge.rs` or the shell; ruling 16 itself left open whether it is view-scoped and whether it persists. It unblocked `fort`, `watchtower`, `fortified_pass`, `fortified_crossing` and `volcanic_feature`; `border_marker` followed on 2026-09-21. `sacred_mountain` stays blocked on §26's cultural-meaning input, not on the viewshed. *Corrected 2026-09-23: this row said "blocked, zero line-of-sight code" for two days after M7 landed* |
| LM-8 | M8 — Category C suitability synthesis + the Landmark object model | done | `landmark.rs::Landmark { id, kind, class, x, y, elevation, score, importance, causal, seed }` — §22's object model including the causal chain and §27's `seed_L`; `pub fn generate(...)` runs §30's twelve steps; the Category C weight/threshold block has every weight a named commented constant. Reachable end to end: `landmark_bridge.rs` + `landmark_kinds()`, `landmark_settings()`, `landmark_run()`, `landmarks()`, `landmark_funnels()`, `landmark_headroom()` → `engine_bridge.gd` → CIVIL ▸ Landmarks (`civilization_workspace.gd::_build_landmarks`) and CARTO ▸ Assets & landmarks (`cartography_workspace.gd`). Edge-case bar met by `degenerate_grids_do_not_panic` and `a_wrongly_sized_optional_input_degrades_rather_than_panicking` |
| LM-9 | M9 — cultural interpretation and temporal state (research §24-26) | partial | **The wiring itself is built 2026-09-23**: `LandmarkInputs::battles` takes every drawn `ConflictKind::Battle` (SP-4), anchor-resolved to its position now, and `battlefield` is buildable — the engine's 27th kind, checked (not assumed) to be reachable at the placement pass, probe `_lm9battle_probe.gd`. Checked, and still blocked for a different reason each: `battlefield_historic` would double-record a cell `battlefield` already covers unless a present-year read separates past from current battles (open question 1, persistence); `destroyed_fortress` still has no "destruction" to read, since `Conflict::outcome` stays free text by SP-4 §5's own deliberate no-resolution-model rule. **No owner decision gates the rest of M9 any more** (corrected 2026-09-23). Ruling AP re-affirmed three questions that had already been ruled on 2026-09-06 and partly built. Persistence is ruling 10: `entities/landmarks.json` has carried the settings and the last run's results since then (`project_bridge.rs::LandmarksDoc`). Vault linking is ruling 13: `cartalith_vault::EntityKind::Landmark`, addressed by `landmark_entity_id` over `Landmark::key()`, `45630cc`. The parity exemption is ruling 12 — *which also required a `DECISIONS.md` §7-series note; there is none (`DECISIONS.md` has no occurrence of "landmark", checked 2026-09-24)*. This cell used to say `EntityKind` had no `Landmark` variant, and it has had one since 2026-09-06. **What remains is build work**: per-landmark authored/temporal state (at minimum a name the player gave it, with research §25's discovered/named/monumentalized as the fuller shape — neither `Landmark` nor `LandmarkDto` has such a field), a present-year read for `battlefield_historic`, the civilisation-traits input §26 requires for the other Cultural-family kinds (`shrine`'s `not_built`: *"That needs the civilisation's own traits as an input, which this pass does not take"*), and a shell entry point for linking a landmark to a note (`vault_landmark_entity_id` has no shell caller; the right dock's `CTX_LANDMARK` context, added under Ruling AL, is now the surface it can hang off) |

**Group total: 9 — 7 done, 2 partial (LM-5, LM-9).** LM-9 moved from blocked to partial 2026-09-23 (its conflict wiring is built). LM-7 corrected from blocked to done the same day (it landed 2026-09-21).
**Residual inside M8:** 22 of 49 declared kinds still ship `buildable: false`
(was 35 at the 2026-09-01 count this paragraph continues with), each with its
reason in source. `resource_extraction_site` went
buildable 2026-09-01 — it reads the three resource-potential fields (`timber`,
`sulfur`, `alum`) that Mine's and Quarry's own resource lists don't, through
their identical, already-validated detector, so it claims no cell either of
them already does; the fixture test
`resource_extraction_site_reads_a_disjoint_resource_set_from_mine_and_quarry`
proves the disjointness rather than asserting it. The other 35 reasons were
individually re-verified against the code the same pass, not just re-read; six
were rewritten for precision (`volcanic_feature`, `rock_formation`,
`glacial_feature`, `salt_works`, `ruin`, `abandoned_settlement`) with no change
to their blocked conclusion. That is one row in `OUTSTANDING_WORK.md` §1 and is
the largest landmark work remaining after M7.

### Religion diffusion · `RELIGION_DIFFUSION_SCOPE.md`

Seven milestones from an owner-supplied paper, scoped 2026-08-29. **Milestone
1 (RD-1) is done**, on top of a foundation the scope document does not number
(RD-0), which landed the same day. Milestones 2-7 are not started. Corrected
2026-09-23: this line said none was started, while RD-0 and RD-1 below both
read `done`. `belief.rs::belief_step`, `belief_seed` and
`SettlementReligionState` are all present.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| RD-0 | Foundation — culture and religion as quantitative traits, and the compatibility relation | done, **consumed** | `cartalith-civ/src/belief.rs`, **2 220 lines** (was 945 at the 2026-09-03 correction below — the file has grown substantially since; **re-corrected 2026-09-23** by a religion-expansion research pass, not re-derived from scratch) — `culture_domain`, `ReligionDomain`, `CIV_RELIGION_DOMAIN`, `religion_domain`, `CompatBasis`, `Compat`, `compat`, `compat_value`, `NEUTRAL_COMPAT`, `COMPAT_WEIGHTS`. Its module doc says it is "the foundation both milestone 1 and milestone 3 need, and nothing above it". **Corrected 2026-09-03:** that claim was false. `grep 'belief::'` across `crates/` excluding the file itself returns **15 hits**, all in `cartalith-godot/src/lib.rs` (`belief_seed`, `belief_links_from_ways`, `BeliefNetwork::build`, `belief_step`, `BELIEF_STEP_RATE`). It has consumers and it is bound |
| RD-1 | 1 — MVP: network exposure and conversion, read-only | **done — re-corrected 2026-09-23, the 2026-09-03 correction itself went stale** | `SettlementReligionState` is a real, built type (`share: [f64; CIV_RELIGION_COUNT]`, one per settlement, held on `CivData::belief`), not a name-only placeholder — confirmed at the symbol by an independent research pass, not re-derived from the 2026-09-03 note's own claim. `belief_step` is a three-term logistic (exposure, compatibility, conformity frequency); `belief_seed` seeds every settlement wholly into its faction's religion; `get_settlements()` emits `religion`/`adherents`. **What is genuinely still absent, per the same 2026-09-23 pass**: the retention split (§22/RD-2), competition (§18/RD-6), missionary/institutional terms, and sea-lane exposure — see `RELIGION_DIFFUSION_SCOPE.md` for the current, real remainder. That document embeds its owner-supplied research paper; there is no separate research file to cite. *Corrected 2026-09-23: this cell pointed to "the research doc it cites", which does not exist* |
| RD-2 | 2 — institutional presence and retention split (§11, §22) | not started | No `Inst_{i,R}`, no clergy count, no `P_retain` |
| RD-3 | 3 — religion trait vectors, authored (§6-§13) | not started | `COMPAT_WEIGHTS` is `[None, Some(1.0), None, None, None]` — one of five components populated. Primarily a content pass |
| RD-4 | 4 — prestige and success bias (§15-16) | not started | Needs `EliteAdherents` / `RulerReligion` / `MerchantStatus` terms this port does not compute |
| RD-5 | 5 — political modifiers (§25) | not started | Also the milestone where §4's unresolved fork must be decided: is a state religion the hand-set flag or a derived plurality? |
| RD-6 | 6 — competition (§18) and vertical/horizontal/oblique weighting (§23) | not started | — |
| RD-7 | 7 — sensitivity-analysis tooling (§31) | not started | Dev-facing |

**Group total: 8 — 2 done, 6 not started.** (Corrected 2026-09-23 from "1 done (unconsumed), 7 not started": RD-1 was already `done`, and RD-0 is consumed.)
The religion *screens* shipped in a 2026-09-03 batch. The gap-register group's GGR-RELIG row records the correction (this line used to call them blocked).

### Timeline · `TIMELINE_SCOPE.md`

Six milestones, all built 2026-08-19.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| TL-1 | 1 — shared prerequisites: population-ceiling chain + stable ids | done | `timeline.rs::{civ_subsistence_mode_at, civ_agrarian_density_km2, civ_current_agrarian_density, civ_settlement_population, civ_tier_floor, civ_tier_for_population, civ_assign_tid, civ_resync_next_tid}` |
| TL-2 | 2 — proximity graph + Brandes betweenness centrality | done | `timeline.rs::civ_proximity_adjacency` and `civ_betweenness_from_adjacency`, the latter documented as Brandes (2001), un-normalised, one BFS per source |
| TL-3 | 3 — the collapse and recovery step functions | done | `timeline.rs::{civ_collapse_step, civ_recovery_growth_step, civ_apply_recovery, civ_settlement_stress, civ_mortality_migration_rates, civ_gravity_migrate}` with `CollapseStepResult` / `RecoveryStepResult` |
| TL-4 | 4 — snapshot data model + orchestrator | done | `timeline.rs::{TimelineSnapshot, YearDiff, civ_year_diff, civ_snapshot_save, civ_snapshot_load, civ_simulate_timeline}` with `SimulateMode` / `SimulateTimelineOpts` |
| TL-5 | 5 — the Godot boundary | done | `cartalith-godot/src/timeline_bridge.rs` (`CollapseSimRequest`, `CollapseSimReport`, `run_collapse_simulation`) and the `#[func]`s `civ_add_year`, `civ_goto_year`, `civ_year_diff`, `civ_run_collapse_simulation` |
| TL-6 | 6 — UI playback controls | partial | `shell/workspaces/civilization_workspace.gd`'s **Timeline** category (`_build_timeline`, `DccWidgets.category(self, "Timeline", …)`; its own header cites `TIMELINE_SCOPE.md` milestone 6): years pill row + Add year, the 1200 ms playback transport, the collapse/recovery form with its overwrite confirmation, and the "Exist only" filter over `civ_year_diff().present` (`_tl_apply_filters`). **Not met:** the milestone's *three* filters — "Ghost removed" and "Highlight new" are drawn but do nothing (`_build_timeline_filters`: per-pin fade/halo is unbuilt, and `civ_year_diff()` returns tid sets only, not the removed settlements' positions). *Corrected 2026-09-24 (alignment audit Part 1 C25/B15): this row said `done` and placed the controls "under Politics"; the category has been called Timeline since Ruling L* |

**Group total: 6 — 5 done, 1 partial.** TL-6 moved done → partial 2026-09-24.
*Corrected 2026-09-23:* this said CV-24 and ED-02 were open owner decisions.
ED-02 (the undo-history panel) is built — `Edit ▸ Undo history…`
(`menus.gd`, `GUI_GAP_REGISTER.md` §42). CV-24 (the year scrubber as program
scope) waits on a design, not an owner decision. And TL-6's criterion 5 is
only partly met: the *Ghost removed* and *Highlight new* toggles change nothing
(`civilization_workspace.gd::_build_timeline_filters`, `GUI_GAP_REGISTER.md`
CV-03) — routed in `OUTSTANDING_WORK.md` §2.3.

### Phase 3 — 2D terrain appearance · `TERRAIN_APPEARANCE_SCOPE.md`

Six milestones plus one follow-up, and §20's staged half (Ruling AN). All built. The 3D half of Phase 3 is not in
this document and does not exist — see *Orientation*.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| TA-1 | 1 — audit + `TerrainAppearance` abstraction + one real editable ramp | done | `render.rs::TerrainAppearance`, owned by `RenderCtx`; the field-name→range table drives `WorldGen::set_appearance`, which `app.gd` calls at startup with `DccSettings.lighting_defaults()`, and `dcc_settings.gd::appearance_defaults()` persists. *"Not yet wired to any UI/`#[func]`" is stale* |
| TA-2 | 2 — multidirectional hillshade + ambient occlusion | done | `TerrainAppearance::{sun_alt_deg, relief_lights, ao_strength, ao_radius_frac}`; `render.rs::build_ao`. Exposed through the `set_appearance` key table |
| TA-3 | 3 — hydrology-based colour tint | done | `TerrainAppearance::{hydro_wet_strength, hydro_wet_radius_frac}` (defaults 0.38 / 0.006), computed by `render.rs::build_hydro_wetness`, tunable via the `"hydro_wet_strength" => "Wetness"` row |
| TA-4 | 4 — the atlas look: paper ground, forest stippling, plate border | done | `TerrainAppearance::{paper_strength, paper_tint, paper_grain, paper_mottle, paper_wash, stipple_strength, stipple_scale_frac, border}`; `render.rs::paper_tone` and `apply_border` |
| TA-4F | 4 follow-up — the overlays learn about the frame | done | `WorldGen::get_border_inset_frac` (wrapped as `engine_bridge.gd::border_inset_frac`; *re-pointed 2026-09-24 — the `#[func]` is `get_border_inset_frac`*) is consumed by every overlay draw call — `viewport_host.gd` and `civilization_workspace.gd` both pass `_bridge.border_inset_frac()` alongside the road/sea-route geometry |
| TA-5 | 5 — geological material exposure + local contrast | done | `TerrainAppearance::{litho_exposure, local_contrast, local_contrast_radius_frac, local_contrast_knee}`; `render.rs::apply_local_contrast`, a rayon-parallel neighbourhood pass over final colour |
| TA-6 | 6 — the GPU question answered by measurement; §29 quality tiers | done | Parallel pass: `use rayon::prelude::*` in `render.rs` with `par_chunks_mut` at three sites and a `rayon::join`. Tiers: `enum QualityTier`, `TerrainAppearance::for_tier`, `recommended_quality_tier` (with an explicit Android downgrade); `WorldGen::{get_quality_tier, set_quality_tier, list_quality_tiers, get_recommended_quality_tier}`; `engine_bridge.gd` and the tier picker in `menus.gd`; `tests/appearance_tiers.rs` |
| TA-20s | §20's staged half: the finishing stages as one pass with one quantisation (owner ruling, `LARGE_ITEM_RULINGS.md` Ruling AN, 2026-09-23) | done, pending independent verification | `render.rs::finish_rgb`/`finish_raster`/`local_contrast_rows`; called from `lib.rs::build_color_texture`, `export_raster.rs` (`export_raster_png`, layers), `render::bake_export_band`, `render::render_biome_tile_rgba`. `tests/color_space.rs::fusing_the_finishing_passes_is_a_bounded_rounding_change` and `ANTIQUE_P3_FNV1A`; the default `FINISHED_RENDER_FNV1A` was unchanged by this milestone (both hashes were later re-baselined by the snow aspect term, `19c38d9`, 2026-09-24 — see LOD-D4 — and again by Ruling AS's ocean lattice fix, same day). Probe: `godot-project/_finishpass_probe.gd` (windowed). §20's output half (HDR/tone mapping, a non-8-bit encoder) is **not built** |

**Group total: 8 — 8 done.**
The GUI for all of this is `shell/workspaces/render_workspace.gd` (2 140 lines at `919bce1`, counted 2026-09-24; this said 1 055),
composed into CARTO — see GFP-5.

### Sculpt live · `SCULPT_LIVE_SCOPE.md`

Five milestones (L0-L4). The sculpt **editor** shipped as tool-plan milestone B;
"live" — bounded preview during a stroke — did not.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| SL-0 | L0 — measure, before deciding anything else | done | *Corrected 2026-09-24 (alignment audit Part 2 C3 / Part 3): this row said `not started`, "no instrumented harness", for five weeks after one landed.* `cartalith-godot/tests/sculpt_live_l0_bench.rs::l0_measure` (`611c5fa`, 2026-08-18) times the stamp, the whole-grid precomputes, the per-pixel loop, the commit's steps, `compute_flow` (CPU and GPU) and climate refresh; `SCULPT_LIVE_SCOPE.md` §8 records its findings. **Caveat the scope states:** the preview now runs more whole-grid work than L0 measured (`GridPrecompute::build`'s later additions, `with_map_scale`'s river SDF and water bodies), so L0 should be re-run before L1 is sized |
| SL-1 | L1 — bounded live preview | not started | `render.rs::build_ao` still takes `(field, gw, gh, sea_level, world, a)` — **no window parameter**, and neither do `smooth_sea_h` or `build_hydro_wetness`. `PassBuffer::touched_bounds()` exists in `cartalith-spatial`, and `cartalith-godot/src/lib.rs` refers to it in the subjunctive ("would give the rectangle a bounded …"), i.e. it is not called for this |
| SL-2 | L2 — live water, at proxy resolution | not started | *Updated 2026-09-24:* it said "blocked by definition on L0's numbers"; those numbers exist (SL-0), and `SCULPT_LIVE_SCOPE.md` §8 reads them as favouring the full-resolution GPU accumulation route over a proxy. Nothing of L2 is built; river/lake reclassification on top of accumulation was not measured |
| SL-3 | L3 — downstream (erosion, climate, biomes, civ): proxied, not live | declined | A recommendation, not a limitation to engineer away: the crates operate on the whole field, and erosion and climate are global equilibria — a locally-recomputed result is a different answer, not a preview. What ships instead is the staleness readout (tool-plan milestone F / GFP-3) |
| SL-4 | L4 — the three §5.2 blocks with no engine | not started | Independently scoped: neither v2.10 nor `sculpt.rs` has them |

**Group total: 5 — 1 done, 3 not started, 1 declined.** SL-0 moved not started → done 2026-09-24.

### Phase 4 — Asset Library · `ASSET_LIBRARY_SCOPE.md`

Eight milestones plus §9's GUI window and §10's `#[func]` surface. Complete.
Note that `ROADMAP.md` says "all seven milestones" — the eighth (the
sprite-sheet slicer) landed 2026-08-20 and the count was never updated.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| AL-1 | 1 — pack manifest model, parsing, validation, serialization | done | `cartalith-assets/src/manifest.rs` — `PackManifest`, `parse_pack_csv`, `parse_pack_manifest`, `parse_pack_entries`; `tests/golden_parity_pack_manifest.rs` |
| AL-2 | 2 — pack ZIP read/write | done | `cartalith-assets/src/archive.rs` — `read_pack_entries`, `read_pack`, `write_pack_entries`, `write_pack`; `tests/golden_parity_pack_zip.rs`. Reachable via `WorldGen::load_asset_pack` and menus.gd's "Import asset pack .zip…" |
| AL-3 | 3 — scatter rules | done | `cartalith-assets/src/scatter.rs` — `preset_scatter_rule`, `normalize_scatter_rule`, `pick_icon_variant`, `autopopulate_scatter_rules`, `scatter_rule_key`; `tests/golden_parity_scatter_rules.rs` |
| AL-4 | 4 — rule-driven icon placement | done | `cartalith-assets/src/placement.rs` — `place_map_icons_ruled`, `icon_slot_for_item`, `sprite_draw_rect`; `tests/golden_parity_placement.rs`; called for real from `cartalith-godot/src/pack.rs` |
| AL-5 | 5 — the Library model (`AssetDB`, collections, validator, `library.json`) | done | `cartalith-assets/src/library.rs` — `AssetDB`, `AssetCollections`, `rename_custom_slot`, `to_library_json`, `parse_library_json`, the validator run; `tests/golden_parity_library.rs` and `hardening_asset_db.rs` |
| AL-6 | 6 — image handling | done | `cartalith-assets/src/raster.rs` — `decode_png`, `encode_png`, `encode_png_rgb8`, `encode_png_luma16`; `tests/golden_parity_raster.rs` |
| AL-7 | 7 — renderer + Godot integration | done | `cartalith-godot/src/pack.rs` — `load_pack_from_bytes`, `composite_map_icons`; `WorldGen::icon_list`; `tests/pack_compositing.rs` |
| AL-8 | 8 / §11 — the sprite-sheet slicer | done | `cartalith-assets/src/slicer.rs::slice_sheet` with `tests/golden_parity_slicer.rs`; driven from `asset_bridge.rs`'s `load_sheet` / `slice_preview` / `apply_slice`, and `slice_params_from`. Reached from `menus.gd`'s "⧉ Sprite sheet slicer (▦)" |
| AL-9 | §9 — the Asset library GUI window | done | `shell/asset_library_window.gd`, 189 132 bytes (`wc -c`, 2026-09-24; was 160 685), reached from `menus.gd`'s `_live(p, "⧉ Asset library", ID_ASSET_LIBRARY, KEY_MASK_SHIFT \| KEY_A)`. The family rail matches `cartalith-assets`' own `slots.rs` grouping — **nine** families there (`Family::ALL: [Family; 9]`, `SeaMark` added by the 2026-09-02 ruling; this said eight). *§9's body enumerates eight gaps that §10 and §11 later close; both readings stand in the file and §9 is only true as of 2026-08-19* |
| AL-10 | §10 — the `AssetDB` `#[func]` surface (the `as_*` methods: eighteen at `8506f13`, 27 in `cartalith-godot/src/lib.rs` by 2026-09-23 — this row said "twenty") | done | `cartalith-godot/src/asset_bridge.rs` — `AssetLibrarySession` with `import_item`, `add_custom_slot`, `remove_item`, `validate`, `thumbnail_png`, batch tag/collect/rename/duplicate/delete, `export_pack_bytes`; held as `WorldGen::asset_library` and surviving re-generate |

**Group total: 10 — 10 done.**
Three items are **declined because the engine has no counterpart** and should
not be re-proposed: AS-14 (user-picked active variant — variant choice is
weighted and seeded), AS-15 (per-slot Anchor — `Anchor` is a *family* property),
AS-16 (the 24-family rail — owner decision, disclosed in the window's header).
### Phase 5 — Urban morphology · `URBAN_MORPHOLOGY_SCOPE.md`

Twenty rows (milestones 1-17, plus 8a and 17a which shipped out of order, plus 17a's golden verification UM-17A-G). *(Corrected 2026-09-24: this said nineteen; the table has twenty.)*
*(Corrected 2026-09-23: this line called Phase 5 "the largest block of unbuilt
work in the project", which stopped being true when its milestones closed —
the group's own rows are the answer.)*

*Stale, kept as the 2026-08 baseline:* `crates/cartalith-urban/src/lib.rs` now
declares 21 `pub mod` lines, `generate` among them (2026-09-24). The paragraph
below describes the crate before milestones 8-16. The single decisive check was: `crates/cartalith-urban/src/lib.rs` declares exactly
ten `pub mod` lines — `astar`, `blocks`, `geom`, `graph`, `growth`, `plaza`,
`rng`, `routes`, `rules`, `site`. There is **no** fortification, districts,
amenities, water-infrastructure, hinterland or `generate()`-orchestration
module. Every "not started" row below rests on that list plus a named
corroborating comment.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| UM-1 | 1 — RNG substreams + geometry kernel | done | `cartalith-urban/src/rng.rs` (`Substream`, `fnv1a`, `stream`) and `geom.rs` (`Vec2`, `js_hypot`, `js_exp`, `js_sin`, `js_cos`, `js_log`, `js_round`, `js_min`, `js_max`), both re-exported |
| UM-2 | 2 — planar street graph | done | `graph.rs` (`Graph`, `Node`, `Edge`, `Face` + uniform-grid index and planar face extraction), with `graph/tests.rs` and `graph/tests/golden.rs` |
| UM-3 | 3 — A* over the cost raster | done | `astar.rs`, re-exported as `astar`, with `astar/tests.rs` and `astar/tests/golden.rs` |
| UM-4 | 4 — generation rules + culture profiles | done | `rules.rs` — `CULTURE_PROFILES`, `DEFAULT_RULES`, `MEDIEVAL`, `VENUS`, `MetaRules`, `ParcelRules`, `StreetRules`, `SettlementRules`, `apply_plot_chaos`, `apply_wildness`, `resolve_profile`, `resolve_rules`; `rules/tests.rs` + `golden.rs` |
| UM-5 | 5 — the site model | done | `site.rs` — `build_site`, `Site`, `SiteOpts`, `WaterCtx`, `TerrainCtx`, `Harbour`, `Hill`, `Economy`, `shore_from_mask`, `terrain_suitability`; `site/tests/golden.rs` is 4 502 lines. Called from `cartalith-urban/src/generate.rs` (which `urban_adapter.rs::run_layout` calls) — *re-pointed 2026-09-24; this said `urban_adapter.rs` calls it directly* |
| UM-6 | 6 — anchors and primary routes | done | `routes.rs` — `place_anchors`, `build_primaries`, `build_primaries_from_paths`, `Anchors`, `Route`; called from `generate.rs` (*re-pointed 2026-09-24 from `urban_adapter.rs`*) |
| UM-7 | 7 — organic growth | done | `growth.rs` — `grow`, `GrowOpts`, `Occupancy`, `WallBuilder` / `RecordingWallBuilder`, `WallState`, `WallGeneration`, `supersede_wall`, `estimate_carrying_capacity`, `logistic_ramp`, `ring_crossings`, `dist_to_line`; `growth/tests/golden.rs` is 2 159 lines |
| UM-8A | 8a — the plaza (`buildPlaza`) | done | `plaza.rs::build_plaza` with `plaza/tests.rs` + `golden.rs`; called from `generate.rs` (*re-pointed 2026-09-24 from `urban_adapter.rs`*) |
| UM-8 | 8 — radial (Venus) streets, waterway | done | `radial.rs` — `build_radial_streets`, `build_waterway`; committed in `4ec07f5` |
| UM-9 | 9 — water infrastructure (`buildHarbour`, `addRiverBridges`, `detectRiverCrossings`) | done | `water.rs` — 716 lines (`wc -l`, 2026-09-24); committed in `4ec07f5` |
| UM-10 | 10 — fortification (`buildWall`, `applyStarFort`, `townBank`, `builtMassHull` …) | done | `fortify.rs` — 1 163 lines (`wc -l`, 2026-09-24); committed in `4ec07f5` |
| UM-11 | 11 — graph cleanup passes (`pruneLargest`, `removeWaterCrossings`, `privatizeAlleys`, `lanePass` …) | done | `cleanup.rs` — 637 lines (`wc -l`, 2026-09-24); committed in `4ec07f5` |
| UM-12 | 12 — blocks and parcels | done | `blocks.rs` — `build_blocks`, `build_parcels`, `Block`, `Parcel`, with `blocks/tests.rs` and `blocks/tests/golden.rs`; called from `generate.rs` (*re-pointed 2026-09-24 from `urban_adapter.rs`*) and drawn by `shell/urban_layout_draw.gd` |
| UM-13 | 13 — districts and buildings | done | `districts.rs` — 1 458 lines (`wc -l`, 2026-09-24); committed in `4ec07f5` |
| UM-14 | 14 — amenities (markets, civic hall, games) | done | `amenities.rs` — 725 lines (`wc -l`, 2026-09-24); committed in `4ec07f5` |
| UM-15 | 15 — hinterland, decay, details, metrics | done | `hinterland.rs` — 1 051 lines (`wc -l`, 2026-09-24) with passing golden; committed in `4ec07f5` |
| UM-16 | 16 — `generate()` orchestration + `hashModel` | done | `cartalith-urban/src/generate.rs`, added in `cff1edc` (2026-09-02), golden-verified by `generate/tests/golden.rs`; `run_layout` calls `generate()`. *Corrected 2026-09-24: this row read "ready" for three weeks after it shipped* |
| UM-17A | 17a — the adapter and the first consumer | done | `urban_adapter.rs::{um_place_context, run_layout, settlement_layout, settlement_layout_with}` → `cartalith-godot/src/urban_bridge.rs::urban_layouts` (which calls `settlement_layout_with`, carrying a per-settlement rule set — *added 2026-09-24*) → `engine_bridge.gd` → `city_viewer_window.gd` and `viewport_host.gd` (map deep-zoom town layer) |
| UM-17 | 17 — the civ adapter (20 pure `_um*` functions) | done | *Corrected 2026-09-24:* all 20 are accounted for — 18 ported (the 16 distinct functions in `urban_adapter.rs`, where `um_place_context_with` is a variant, plus `um_wall_spec`/`um_infer_walls` in `military.rs`), and the port table at the head of `urban_adapter.rs` records `_umPt` as not applicable (a JS array/object normaliser) and `_umCacheKey` as out of scope by the scope document. The earlier text follows. **16 of 20 ported** (`grep -c "^pub fn um_" crates/cartalith-civ/src/urban_adapter.rs` = 16), verified against the port table at the head of `urban_adapter.rs`: `um_site_box_km`, `um_water_near_km`, `um_water_reach_km`, `um_site_kind_from_terrain`, `um_infer_age`, `um_ray_box_exit`, `um_way_bearing_from`, `um_route_ends`, `um_primary_paths`, `um_terrain_orient`, `um_water_ctx`, `um_terrain_ctx`, `um_place_context` (the last "minus four fields"). `um_wall_spec` / `um_infer_walls` live in `military.rs`. **The three formerly skipped landed in `cff1edc`** — `um_harbour_scale:372`, `um_site_profile:1240`, `um_ore_bearing:1504` — so this row's "three are deliberately skipped pending later milestones" is history as of 2026-09-02. Five cache/draw helpers are out of scope for every milestone by design |
| UM-17A-G | 17a — golden-verify the block-2 `_um*` adapter | done | **2026-09-02.** The recorded blocker — *"needs a block-2 capture harness that can run `_um*` inside the host's full civ scope; the existing harness slices block 4 only"* — was **wrong, not merely stale**: `cartalith-native/tools/um_block2_capture.js` drives the unmodified reference under Node (v24.19.0) and `crates/cartalith-civ/tests/golden_parity_urban_adapter.rs` holds 9 tests over the extracted fixtures. Mutation matrix **22/22 killed**; an independent verifier confirmed the fixtures are genuinely reference-extracted, not replayed from the port. **Two real port bugs found that 11 synthetic-field unit tests had not**: `slope_at` used `f64::hypot` where the reference uses `Math.hypot` (the V8-libm divergence `geom::js_hypot` exists for), and `um_site_profile` clamped the resource-context centre. A third defect was in the fixture itself and was caught before being committed as truth |

**Group total: 20 — 20 done** (corrected 2026-09-24 from "17 done, 1 partial, 1 ready", and the same day from 19 to 20: the table carries twenty rows).

**Ruling-driven urban work after the milestones — no milestone rows, listed so
it is tracked here** (added 2026-09-24, alignment audit Part 1 C23; each named
symbol opened). None of it is a scope-document milestone, so none of it is
counted in the group total above.

- **Ruling H** (move toward the owner's town plan): wall lots and the faubourg,
  `cartalith_urban::wallside` / `generate.rs::build_wall_lots`; perimeter
  courtyard blocks (sited by **Ruling AD**), `courtyard::build_courtyard_rings`,
  called from `generate.rs` with `anchors.market` on the organic plan
  (`ae6a8c8`).
- **Ruling AA** (radial towns get the suburb machinery): `generate.rs` runs
  `build_wall_lots` on both planning branches (`ff8c525`).
- **Ruling I / AC** (a citadel, sited by size tier): `citadel::build_citadel`,
  gated in `generate.rs` on `CITADEL_MIN_POP` and a non-radial plan
  (`9467a23`). Unruled: whether the citadel's area counts toward growth.
- **Ruling J** (set a settlement's city type and regenerate it alone): City
  Viewer's *Town plan* section (`city_viewer_window.gd`, over
  `urban_bridge.rs::urban_town_plan_options` / `apply_urban_rules_preset`),
  `ca25ee1`.

*Resolved 2026-09-24 by the scope's cleanup (`c4c930c`):* the defining
document's two count defects are gone -- it now says 27 `_um*` functions in
total and 20 in the adapter's scope, and the quoted "not a dependency of
`cartalith-godot`" sentence was removed. The layering stands:
`urban_bridge.rs` reaches `cartalith-urban` through `cartalith_civ::urban_adapter`,
and `cartalith-urban` depends only on `cartalith-rng` and `cartalith-jsmath`.

### Tool system · `UNIFIED_TOOL_PLAN.md`

Seven rows (A-F plus E2). **All complete**, and as of 2026-09-01 the plan's own
last line says so too, rather than still calling milestone F the only work
left.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| UTP-A | A — `PassBuffer` / staleness core | done | `cartalith-spatial/src/pass.rs::PassBuffer<S>`; `lib.rs::DirtyTracker` and `StageGraph`. Reached from the shell: `app.gd::_setup_staleness()` starts a 1 s poll onto `refresh_staleness()`, which writes the `stale` status slot from `stale_stages()` and mounts a `Recompute` action into `status_row` |
| UTP-B | B — Terrain group, the Sculpt-editor port | done | `cartalith-terrain/src/sculpt.rs` carries the 13-entry `SCULPT_FEATURES` transcription (registry order documented as load-bearing) and the stamp bbox/apply pipeline; `cartalith-godot/src/sculpt_bridge.rs` plus the `sculpt_set_feature` / `sculpt_begin_stroke` / `sculpt_add_point` / `sculpt_end_stroke` / `sculpt_commit` `#[func]`s. Reachable: `world_workspace.gd`'s `_sculpt_click` / `_sculpt_drag` / `_sculpt_release` |
| UTP-C | C — Water & ecology group (lake `water_only` commit path, biome paint override) | done | `cartalith-engine/src/sculpt_commit.rs` step (4) applies Lake stamps a second time with `water_only = true` against the final height; `sculpt.rs::SculptStamp::apply_into(.., water_only: bool)` (*re-pointed 2026-09-24: this cited `apply_stamp`, which does not exist*) with the unit test `the_lake_water_only_pass_never_touches_the_height_field`. Biome paint: `cartalith-godot/src/paint_bridge.rs`, registered under tool id `"paint"` |
| UTP-D | D — Civilization group (place settlement, draw way/route, territory override) | done | `cartalith-civ/src/tools.rs` — `ManualWay`, `civ_place_pick_radius`, `civ_place_pick_weight`; `infra_tools_bridge.rs`'s `way_begin` / `way_append_point` / `way_commit` / `way_discard`; `civ_tools_bridge.rs::paint_at`. Reachable: `civilization_workspace.gd` registers `"settlement"` + `"territory"`; `infrastructure_workspace.gd` registers `"way"` and `"route"`. **Territory lasso added 2026-09-23 (owner request):** `"territory_lasso"` — a clicked polygon rasterised by `civ_tools_bridge::polygon_cell_mask` and staged by `CivTools::paint_polygon` (`#[func] civ_territory_paint_polygon`) into the same `territory_draft` the brush uses, as one masked `PaintStamp`; faction-level only (provinces have no hand-editable state — `OUTSTANDING_WORK.md` §2.3). `lasso_*` tests, `_terrlasso_probe.tscn`. Pending independent verification |
| UTP-E | E — Annotation & measure group (label arc text, icon stamp, measure, region core) | done | `cartalith-civ/src/labels.rs` — `MapLabel`, `label_box`, `label_hit_test`, `label_arc_value`, `label_rotate_deg`; `cartalith-assets/src/manual.rs::IconBox` + `icon_bridge.rs::icon_handle`; `infra_tools_bridge.rs`'s `measure_begin` / `measure_add_point` / `measure_legs` / `region_set` / `region_tile_estimate`. Reachable: `cartography_workspace.gd` registers `"icon"` and `"label"`; `global_tools.gd` registers `"measure"` and `"region"`; `shell/tool_overlay.gd` draws the marquee, ruler path and brush ring |
| UTP-E2 | E2 — Region select/export encoding half (PNG/gzip/`.zip`/GeoJSON) | done | `cartalith-engine/src/region_export.rs::zip_region_export` (documented as `#refineBtn`'s handler minus the download); `cartalith-engine/src/geojson.rs::export_geojson` with its own golden tests; `cartalith-io/src/{tiles.rs,gzip.rs,atlas.rs}`. Boundary + consumer: `geojson_bridge.rs`, called from `shell/data_manager_window.gd`'s `export_gis` row |
| UTP-F | F — shell wiring (every B-E tool onto the rail / tool options bar / dock, plus the status-bar staleness readout) | done | `shell/app.gd` holds the dispatch substrate (`_click_handlers` / `_drag_handlers` / `_release_handlers` / `_escape_handlers` / `_backspace_handlers`, `_on_map_clicked` / `_on_map_dragged` / `_on_map_released`, `arm_tool`, one shared `tool_group: ButtonGroup`). **Ten tool ids are registered** — counted in the tree: `icon`, `label`, `measure`, `paint`, `region`, `route`, `sculpt`, `settlement`, `territory`, `way`. `shell/engine_bridge.gd` opens a block literally titled `-- Milestone F tool bindings --` with one guarded wrapper per bound `#[func]`. The staleness readout F asked for exists (`app.gd::_setup_staleness` / `refresh_staleness` + the `Recompute` action). Commit affordance: `tool_bar.gd`'s `Commit` chip → `bridge.sculpt_commit("sculpt")` |

**Group total: 7 — 7 done.**

**Closed out 2026-09-01.** `UNIFIED_TOOL_PLAN.md` now carries a verified
"Milestone F as built" section (its own last line rewritten to point at it
instead of trailing off) enumerating all sixteen `STRANDED_TOOLS.md` tools
against the code: thirteen bound, three correctly needing no binding (Select,
Pan and one more), one declined by a predating Milestone D decision (POI —
`tools.rs:137-138`), and one small, honestly-drawn loose end — Region select's
corner-handle resize (`region_resize`) has no `#[func]` behind it and was never
actually scoped by any A-E2 milestone, so it is future work, not a broken
promise. `STRANDED_TOOLS.md`'s own stale "44 methods… not one wired" claim is
annotated false in place, dated, rather than silently rewritten.

### GPU compute pilot · `GPU_COMPUTE_PILOT_SCOPE.md`

Six "done means" criteria. All met. The document gained a resolution section,
*What the pilot found (2026-08-16)*, in `0d8a547` on 2026-09-23. Until then
this paragraph correctly said it had none, and the only place the pilot was
called done was the opening line of `GPU_LAYER_INTEGRATION_SCOPE.md`.

| ID | Criterion | Status | Evidence |
|---|---|---|---|
| PILOT-1 | Minimal wgpu hardware path: Instance/Adapter/Device + §9 self-test | done | `cartalith-gpu/src/lib.rs` — `GpuContext`, `init_gpu()`, `GpuInitError`, `self_test()` (an 8×8 known-input GPU-vs-CPU gate), test `gpu_context_creates_on_this_hardware` |
| PILOT-2 | One compute kernel: vnoise in WGSL at a real field size | done | `shaders/vnoise.wgsl` and `vnoise_f64.wgsl`; `dispatch_gpu()`; CPU reference `vnoise_grid_cpu()` |
| PILOT-3 | CPU-parity test at an explicit, documented tolerance | done, **answered negatively** | `F32_TOLERANCE = 1e-4` with a doc comment naming `cartalith_noise::hash`'s ~2^61 middle product as the reason, and the test that carries the finding, `f32_hash_diverges_from_cpu_reference`. **The criterion asked the kernel to match the golden-verified CPU output; the code's answer is that the JS-matching `hash` is not f32-portable.** That finding is the pilot's whole value and it is written down only in the *other* document — recorded here so it stops being |
| PILOT-4 | CPU fallback path exercised by a real test | done | `vnoise_grid(ctx: Option<&GpuContext>, ...)` gates GPU behind `self_test`; tests `gpu_fallback_path_matches_cpu_reference` and `cpu_path_is_deterministic`; `ComputePath::{Gpu,Cpu}` reports which ran |
| PILOT-5 | Real measured GPU-vs-CPU numbers at several field sizes | done | `measured_gpu_vs_cpu_timing`; `VnoiseResult` carries `gpu_dispatch_and_readback` / `cpu_duration` |
| PILOT-6 | Lives in its own crate with no gdext dependency | done | `cartalith-gpu/Cargo.toml` `[dependencies]` = `cartalith-noise`, `wgpu` 30, `pollster`, `bytemuck` only; no godot/gdext entry. Test-only dev-deps on `cartalith-terrain` / `-climate` / `-hydrology` |

**Group total: 6 — 6 done.**
`init_gpu_f64`, the pilot's one undisposed residue, was deleted on 2026-09-06
under ruling 22 (see the GPU layer integration group below). Its shader source
stays as the pilot's finding.

### GPU layer integration · `GPU_LAYER_INTEGRATION_SCOPE.md`

Nine milestones, two named deferrals, and two lettered milestones — **E**
(erosion's per-cell parts) and **M** (multi-GPU) — which the scope document's
milestone table has carried since `0d8a547` (2026-09-23). *Corrected
2026-09-24 (alignment audit Part 2 C7): this said one shipped subsystem was not
mentioned at all, and GLI-M / GLI-E below said the document had no milestone
for them.*

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| GLI-1 | GPU-safe noise redesign (`gpu_hash` / `gpu_vnoise`) | done | `cartalith-noise/src/lib.rs` — `gpu_hash`, `gpu_vnoise` (single-round PCG3D), alongside the untouched JS-matching `hash` / `vnoise`. `shaders/gpu_noise.wgsl`; `GPU_SAFE_NOISE_TOLERANCE = 1e-5`; tests `gpu_safe_noise_matches_cpu_reference_at_real_field_size`, `gpu_safe_noise_self_test_passes` |
| GLI-2 | Domain warp + crustal heterogeneity on GPU | done | `shaders/gpu_warp.wgsl`, `gpu_heterogeneity.wgsl`; `dispatch_gpu_warp`, `dispatch_gpu_heterogeneity`; `cartalith_noise::gpu_fbm`; `WARP_TOLERANCE = 2e-4` with its own justification comment. CPU `compute_warp` untouched |
| GLI-3 | `compute_height` as a standalone GPU kernel | done | `shaders/gpu_height.wgsl`; `dispatch_gpu_height`; `HEIGHT_TOLERANCE = GPU_SAFE_NOISE_TOLERANCE`; `cartalith_noise::gpu_ridged`; tests `gpu_height_matches_cpu_reference_at_real_field_size` and `gpu_height_has_oro_true_changes_the_formula` |
| GLI-4 | `gauss_blur` + `compute_resistance` on GPU (three-way JS/CPU/GPU parity) | done | `shaders/gpu_gauss_blur.wgsl`, `gpu_resistance.wgsl`; `dispatch_gpu_gauss_blur`, `dispatch_gpu_resistance`; `BLUR_TOLERANCE = 2e-6`, `RESISTANCE_TOLERANCE = 5e-7`; both tests run against the **real** `cartalith-terrain` functions via a dev-dependency, not a GPU twin |
| GLI-5 | Plate assignment (JFA) on GPU | done | `shaders/gpu_jfa_plates.wgsl`; `dispatch_gpu_assign_plates`; `brute_force_nearest_plate` as ground truth; test `gpu_jfa_plates_vs_cpu_jfa_vs_brute_force_ground_truth`. CPU `assign_plates` unchanged |
| GLI-6a | Orogeny graph-tracing on GPU — investigated, judged a poor fit | declined | No orogeny/`trace_boundaries` kernel exists: one doc-comment mention in `cartalith-gpu/src/lib.rs` and no code. `trace_boundaries` remains CPU-only |
| GLI-6b | First real partial-GPU pipeline integration (`use_gpu` flag, per-stage fallback) | done | `cartalith-engine/src/lib.rs` — `WorldParams::use_gpu`, `defaults()` sets it false, `WorldState::gpu_stages_used`, per-stage branches for warp, plate_id, flexure, base_field and heterogeneity, each `match … { Some(..) => push stage name, None => CPU function }`. Determinism and CPU-path-unchanged tests present. **The milestone's own "out of scope: UI exposure of the `use_gpu` flag" is now false**: `engine_bridge.gd` does `param_set("use_gpu", true)` in `_ready()`, and `menus.gd` adds a checked `Preferences ▸ GPU acceleration` row with `GPU_TOGGLE_TIP` (which is §7c's required "this may produce a different world" messaging, verbatim in the tree) and a live backend readout. The engine default is still `false`, so both statements are locally true and only the conclusion is stale |
| GLI-7 | Climate wind/rain loop on GPU | done | `shaders/gpu_weather.wgsl` (evap/advect/deposit entry points); `GpuWeatherContext`, `init_gpu_weather_with`, `dispatch_gpu_weather`, `simulate_weather_loop_gpu_with`. The required refactor landed: `cartalith_climate::build_weather_grid` / `finish_weather_grid` are public and `simulate_weather` calls them. Wired at two sites in `cartalith-engine` (including the post-carve recompute); test `gpu_weather_loop_matches_real_cpu_simulate_weather` |
| GLI-8 | GPU context reuse across `generate_terrain`'s stages | done | `GpuDevice`, `init_gpu_shared_device()`, the `init_gpu_{warp,heterogeneity,jfa_plates,gauss_blur}_with` family and the `*_grid_gpu_with` wrappers. `cartalith-engine` gets its device set from `init_gpu_device_set()` and passes `set.primary()` to every stage. *Updated 2026-09-24:* that call no longer opens a device per generate — under Ruling Y, `multi.rs::init_gpu_device_set` serves a cached set (`DEVICE_CACHE`) while its preference key matches and no device is lost, test `init_gpu_device_set_reuses_a_healthy_device_and_drops_a_lost_one` |
| GLI-9 | Flow accumulation on GPU — the first sequential algorithm redesigned | done | `shaders/gpu_flow.wgsl`; `GpuFlowContext`, `init_gpu_flow_with`, `dispatch_gpu_flow`, `GpuFlowResult`; `FLOW_TOLERANCE = 1e-3` and `FLOW_ANY_CELL_TOLERANCE = 5e-3`. Wired once per generate with a `flow_on_gpu` closure used at all four `compute_flow` call sites. Tests `gpu_flow_matches_real_cpu_compute_flow`, `gpu_flow_is_bit_reproducible`, `gpu_flow_downstream_river_network_divergence`; example `examples/flow_downstream_settlements.rs` |
| GLI-P2 | Phase 2 per-cell affordance fields on GPU (`OUTSTANDING_WORK.md` §2.6) | done — 2 of 4 wired, 2 unwired on measurement | `cartalith-gpu/src/affordance.rs`, `shaders/gpu_{biome,carrying,resources,suitability}.wgsl`. **Wired** in `cartalith-godot::compute_civilisation` under `use_gpu`: `resource_potentials_grid_gpu_with` (per-cell kernel only; `cartalith_civ::{resource_copper_dist, resource_flow_max, finish_resource_potentials}` stay CPU) and `settlement_suitability_grid_gpu_with`, reported as `resource_potentials`/`settlement_suitability` in `get_gpu_stages_used`. **Built, not called by production**: `biome_raster_grid_gpu_with` (bit-identical) and `carrying_capacity_grid_gpu_with` — both measured slower than the CPU at every size. `CARRYING_TOLERANCE`, `RESOURCES_TOLERANCE`, `SUITABILITY_TOLERANCE`; test `tests/affordance.rs`; example `affordance_gpu_compare`; probe `godot-project/_civgpu_probe.tscn` |
| GLI-D1 | Deferred at m5 — `compute_stress` gather reformulation | done | 2026-09-23. `shaders/gpu_stress.wgsl`; `stress_gather_grid_gpu_with`, `StressGather`/`StressParams`/`StressPair` in `cartalith-gpu/src/lib.rs`; `cartalith-engine`'s `compute_stress_gpu`, wired at the existing `use_gpu` call site, reported as `"stress"` in `gpu_stages_used`. Every boundary edge's contribution depends only on its plate pair, so the host tabulates it once per pair and each GPU cell gathers its up to 6 edges (4-neighbour plus world-wrap), writing only itself — no atomics; the boundary-type comparison is exact via a rounding-direction bit shipped with the f32 magnitude. `cartalith_terrain::compute_stress` had `stress_edge`/`normalize_by_abs_max` pulled out (arithmetic unchanged, `golden_parity_stress.rs` passes unmodified) so both paths share one formula. `STRESS_GPU_TOL = 2e-6` (measured: worst 5.36e-7 across 128²–2048², 6.3–6.7x at 2048²), 18/19 mutants killed (1 equivalent), probe `_stressgpu_probe.gd`. `cargo test --workspace --no-fail-fast` 3723 → 3725. `OUTSTANDING_WORK.md` §2.6's own row has the full account |
| GLI-D2 | Deferred at m2 — world-wrap support for the milestone 1-5 kernels | done | 2026-09-21, `023c904`. `cartalith-noise` gained `gpu_pvnoise` / `gpu_pfbm`, periodic siblings of `gpu_vnoise` / `gpu_fbm`, with matching WGSL behind a `world` flag in both shaders, so warp and heterogeneity now dispatch on the GPU under `world=true`. The old `if p.use_gpu && !world` gate survives only as history in a doc comment in `cartalith-engine/src/lib.rs`. *Corrected 2026-09-23: this row said "not started" for two days after it landed.* **Plate assignment wrapped only from `a74b35c` (2026-09-24)**: until then this row overclaimed — `023c904` covered warp and heterogeneity, while the GPU JFA plate kernel ran on world maps without wrap and cut every row at the seam (alignment audit Part 2 A1/C). `shaders/gpu_jfa_plates.wgsl` now takes a `world` flag (x-neighbours wrap, distances fold to the nearest copy); non-world plate ids are byte-identical |
| GLI-E | Erosion's per-cell parts on GPU (`OUTSTANDING_WORK.md` §2.6) — the scope document's milestone **E** (row added here 2026-09-23) | done — thermal only; stream-power declined on measurement | 2026-09-23, `08020ee`. `shaders/gpu_thermal.wgsl` and `cartalith_gpu::thermal_grid_gpu_with`, the gather form of `erode_thermal`'s scatter (no atomics). Wired in `cartalith_engine::erode_op` (the Erode button's op, not a `generate_terrain` stage) behind the same `use_gpu` / `gpu_allowed_for_grid` gate as the other GPU stages, with CPU `erode_thermal` as fallback. `ErodeSummary::thermal_on_gpu` reports which path ran. `THERMAL_GPU_TOL = 1e-6` (`erode_op.rs` tests); probe `_thermalgpu_probe.gd`. **Stream-power is not ported, deliberately**: its per-cell phases measured 1.1-1.8% of the kernel, and the rest is serial by construction. `OUTSTANDING_WORK.md` §2.6's row keeps it open as a scoping question |
| GLI-M | Multi-GPU device set, VRAM budgeting and split-tiles warp — the scope document's milestone **M** | done | `cartalith-gpu/src/multi.rs`, 1 653 lines (`wc -l`, 2026-09-24; was 1 291): `MultiGpuMode`, `VramFallback`, `GpuPreferences`, `enumerate_devices()`, `vram_verdict()`, `gpu_allowed_for_grid()`, `device_supports_grid()`, `GpuDeviceSet`, `init_gpu_device_set()`, `split_rows()`, `set_weights()`; `warp_grid_gpu_split` / `warp_band_gpu_with` in `src/lib.rs`. Reached from `cartalith-engine` **before every other GPU stage**, gating the whole GPU path on a VRAM verdict, and from the shell: `menus.gd::_build_gpu_mode_menu()` plus the `gpu_vram_budget_gb` / `gpu_set_vram_fallback` / `gpu_vram_estimate` `#[func]`s. `AlternateFrames` and `ReduceWorkingRes` are deliberately unimplemented variants whose `is_implemented()` returns false |

**Group total: 15 — 14 done, 1 declined.** GLI-D1 moved not-started → done
2026-09-23; GLI-D2 was corrected to done the same day (it landed 2026-09-21);
GLI-E was added the same day (it landed 2026-09-23 and had no row).

**The seven zero-caller public `cartalith-gpu` functions are deleted**
(corrected 2026-09-23: this paragraph still called them live and blocked on an
owner decision). `init_gpu_f64` went on 2026-09-06 under `LARGE_ITEM_RULINGS.md`
ruling 22; its shader source stays, and a doc comment in
`cartalith-gpu/src/lib.rs` records why. The other six (`heterogeneity_grid_gpu`,
`gauss_blur_grid_gpu`, `assign_plates_grid_gpu`, `flow_accumulation_gpu_with`,
`gpu_resistance_grid_cpu`, `warp_grid_gpu`) went in `46aff27` (2026-09-21). A
grep for `fn <name>` across `crates/` finds none of the seven, 2026-09-23.

### CPU multithreading · `CPU_MULTITHREADING_SCOPE.md`

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| CPU-1 | Pass 1 — `cartalith-terrain` (warp, heterogeneity, height, resistance, `gauss_blur`) | done | `rayon = "1"` in `Cargo.toml`; all five parallelised — `compute_warp` (`par_chunks_mut`/`zip`), `box_h`, `box_v` (column-major scratch), `compute_heterogeneity`, `compute_resistance` (`par_iter_mut`), `compute_height` |
| CPU-2 | Pass 2 — `cartalith-civ` | done | Spot-verified in place: `build_lithology`, `build_slope_field`, `build_soil_fertility`, `build_water_access`, `build_biome_raster`, `build_wetland_mask`, `build_carrying_capacity`, `build_npp`, `apply_resource_scarcity` (incl. `par_sort_unstable_by`), `build_resource_potentials`, `build_route_corridors`, `build_settlement_suitability`, `build_travel_cost`, `assign_territory`'s inner loop. **The named-sequential set is genuinely sequential**: `jfa_dist` and `road_dijkstra` carry no rayon |
| CPU-3 | Pass 3 — `cartalith-climate` / `-erosion` / `-hydrology` | done | Hydrology's two wins are exactly the two claimed — `compute_flow`'s rain rescale `acc.par_iter_mut()` and `build_channels`' triple `par_chunks_mut`. The confirmed-unsafe cases stayed sequential: `cartalith-erosion`'s `droplet_kernel` contains zero rayon calls |
| CPU-4 | 2026-08-19 investigation: "only GPU active, no parallelisation" — working as designed | done | Both structural claims re-verified: `rayon = "1"` is present in `-terrain`, `-civ`, `-climate`, `-erosion`, `-hydrology`, `-godot` (and now `-engine`); `compute_height` and `compute_resistance` are called unconditionally from `cartalith-engine`, outside every `if p.use_gpu` branch, so the CPU+Rayon phase still runs on every generate. The timing tables themselves are device measurements — **not checkable from code** |
| CPU-5 | 2026-08-25 ponytail pass — duplicate `build_water_bodies` / `build_slope_field` removed, LOD tile passes parallelised | done | *At the time,* `build_water_bodies` had exactly one call site in `cartalith-godot/src/lib.rs`, with `CivData::water_bodies` holding the classification for `absorb`/PaintEditor and a doc comment saying why. **No longer (re-checked 2026-09-24):** `lib.rs` now calls it from four functions — `compute_civilisation`, `build_color_texture`, `get_rivers`, `build_sculpt_preview_texture` — and `export_raster.rs`, `export_session.rs` and `infra_tools_bridge.rs` call it too; the cost of the repeated classification is unmeasured and no row tracks it; `build_slope_field` likewise. Sixth crate confirmed: `cartalith-terrain/src/amplify.rs`'s `par_chunks_mut` in `amplify_region` and `add_zoom_detail`, and `tile_render.rs`'s `shade_tile` |
| CPU-6 | Integrated-GPU / multi-adapter idea — recorded, explicitly not scoped | **built elsewhere** | See GLI-M. `multi.rs::enumerate_devices()` walks every adapter, `GpuDeviceSet` opens more than one, `MultiGpuMode::SplitTiles` partitions the warp grid across them, reachable at `Preferences ▸ GPU`. *Corrected 2026-09-24:* this row quoted the document as saying the integrated GPU "is never enumerated or used at all" and called it built "contrary to this document"; those sentences are gone — the document's section is now headed *"Separate idea, recorded here and built elsewhere"* and points at `multi.rs` |
| CPU-7 | Remaining hard-hazard functions (CPU flow accumulation, priority-flood, scatter-writes, per-droplet state) | declined | Confirmed still sequential by reading each: `compute_flow` (only its rain rescale is parallel), `build_water_bodies`' priority-flood (`MinHeap::with_capacity(n)`), `chamfer_dist` / `jfa_dist`, `road_dijkstra`, `droplet_kernel`. The document's own rule — genuine cross-cell state, not "hasn't been tried" — holds in the code |

**Group total: 7 — 5 done, 1 built elsewhere (CPU-6), 1 declined.** *(Corrected 2026-09-24: this said 6 done, while the Ledger totals have always counted CPU-6's qualified status separately.)*

**Not a milestone, and no row until now** (added 2026-09-24): a configurable CPU worker pool is built — `cartalith_engine::{ensure_thread_pool, set_configured_thread_count}` (test `tests/thread_pool_setter_honesty.rs`), set from `menus.gd::_build_cpu_threads_menu`. `OUTSTANDING_WORK.md` §5's entry declining a bounded thread pool now carries a 2026-09-24 correction saying it was built.

**Census drift, re-measured 2026-08-31** (pattern:
`par_iter|par_chunks|par_bridge|par_sort|rayon::join|collect_into_vec|par_extend`
over each crate's `src/`): terrain **11**, civ **30**, climate **60**, erosion
**15**, hydrology **4**, godot **16**, engine **1**. The document's 2026-08-19
census read 8 / 25 / 44 / 13 / 5. **Hydrology went down**, which is exactly the
direction that census was written to detect — the 2026-08-25 ponytail pass
removed duplicated work rather than adding parallelism.

### Memory optimisation · `MEMORY_OPTIMIZATION_SCOPE.md`

Fifteen rows: five landed passes and the ranked R1-R8 list the 2026-08-25 audit
produced. **R1-R5, R7 and R8 have landed; R6 has not** (corrected 2026-09-23;
this line said R4-R8 had not landed, and R4, R5, R7 and R8 had been in the tree
since `4ec07f5` on 2026-09-02).

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| MEM-1 | Instrument, confirm the dominant cost, fix (2026-08-16): six unused resource fields | done | `compute_civilisation` sets `resources.clay/buildstone/flint/obsidian/sulfur/alum = Vec::new()` — since `7228bb4` (2026-08-17) after settlement placement rather than immediately after `build_resource_potentials` returns, because the economy wiring needs all fifteen keys until then (the comment at the free says so; *re-pointed 2026-09-24*). `ResourcePotentials` still computes all 15, as the fix intended |
| MEM-2 | Tracked budget line item: the global undo stack (2026-08-23) | done | `cartalith-godot/src/undo.rs` — `MAX_STEPS = 5`, `DEFAULT_BUDGET_BYTES = 256 * 1024 * 1024`, constructor takes the budget, module doc states the whichever-binds-first rule. UI: `menus.gd`'s `Undo history` submenu with `Clear undo history now` and a live-cost tooltip |
| MEM-3 | Android budget measured by category + kept instrumentation (2026-08-25) | done\* | Instrumentation is real: ruling 19 deleted `shell/performance_window.gd` (`06c8963`), and `menus.gd::_refresh_working_set_row` now carries `RENDER_VIDEO_MEM_USED` / `RENDER_TEXTURE_MEM_USED` / `RENDER_BUFFER_MEM_USED` on the Working set row beside `OS.get_static_memory_usage()`. The PSS/category tables are handset measurements — **not checkable from code** |
| MEM-4 | Generation peak measured field by field, ranked list R1-R8 (2026-08-25) | done | This milestone produced an audit, not production code; its conclusions are the R-rows below. **The three probes it was built on are deleted as of 2026-09-03** — `cartalith-civ/examples/_peakaudit_peak.rs`, `_peakaudit_block.rs`, `_peakaudit_hash.rs` — which `MEMORY_OPTIMIZATION_SCOPE.md` scheduled for exactly this point ("named for deletion when the audit closes"). *This cell read "the probes … are present and uncalled, exactly as recorded" until they were removed, and a verifier caught it the same day; a deleted file named as present is the defect this file exists to prevent.* |
| MEM-5 | Overlay lever 1 — collapse the dash loop into one `draw_multiline` | declined | Measured a no-op and reverted: `godot-project/map_overlay.gd` still emits per-dash lines, and the evidence probe was kept — `_dashbatch_probe.gd` / `.tscn` exist. The document says the probe is retained "as the reason not to try it again"; that is what the tree shows |
| MEM-6 | Overlay lever 2 — bound the overlay by zoom (`_run_offscreen`) | done | `map_overlay.gd` — `_visible_local_rect()` inverts the canvas transform, `_run_offscreen(pts, k, pad)` rejects a run whose bounds miss it, cached once per `_draw()` and applied at the way draw site. Pixel-identity probe kept: `_cull_probe.gd` / `.tscn` |
| MEM-7 | R1 — free the previous world before generating the next | done | `cartalith-godot/src/lib.rs::release_world(&mut self)`, called from `generate_sized` and — the audit's own correction — from `generate_world_structure_sized`. Both call sites sit below their function's refusal checks, as the safety argument requires |
| MEM-8 | R2 — delete four dead resident grids | done | `WorldState` no longer declares `flexure_field`, `heterogeneity_field` or `flow_area` — they are locals inside `generate_terrain`. `ChannelResult::slope` was **deliberately not deleted** and is released instead (`ch.slope = Vec::new()`), because `golden_parity_river.rs` asserts it in all three cases. *Resolved 2026-09-24 check: §6's R2 table now carries both columns, and its "Where the audit was wrong" section records that it once said "nobody, anywhere"* |
| MEM-9 | R3 — block `build_resource_potentials`' `per_cell` buffer | done | `const RESOURCE_BLOCK: usize = 1 << 18;` with `per_cell: Vec<[f32; 15]>` allocated at `RESOURCE_BLOCK.min(n)`, a `while block_start < n` loop, `collect_into_vec(&mut per_cell)` and a per-block sequential scatter |
| MEM-10 | R4 — `plate_id: Vec<usize>` → `Vec<u16>` | done | `WorldState::plate_id: Vec<u16>` (`cartalith-engine/src/lib.rs`) and `assign_plates(...) -> Vec<u16>` (`cartalith-terrain`). Landed in `4ec07f5` (2026-09-02, per `git log -S`) |
| MEM-11 | R5 — `jfa_dist`'s three scratch grids to i32/i32/u32 | done | `jfa_dist` (`cartalith-civ/src/lib.rs`) now carries an `R5` comment and `i32`/`i32`/`u32` scratch, argued **bit-identical**, with a `u32` headroom assertion. Landed in `4ec07f5` |
| MEM-12 | R6 — the two `with_capacity(n)` heap reservations | not started | Unchanged, re-checked 2026-09-23: `MinHeap::with_capacity(n)` in `build_water_bodies` and `DijkstraHeap::with_capacity(n)` in `road_dijkstra`. `MEMORY_OPTIMIZATION_SCOPE.md` ranks it low on purpose (on Android an untouched reservation is address space, not resident pages), and `OUTSTANDING_WORK.md` §5 lists it as declined as low-value. Neither is a ruling or a code note, which this file's `declined` requires, so the row stays `not started` |
| MEM-13 | R7 — `road_dijkstra`'s discarded `prev` | done | `road_dijkstra(..., want_prev: bool)` with an R7 doc comment; the sweep that only read `dist` passes `want_prev: false`, and a test asserts `prev` is not written. Landed in `4ec07f5` |
| MEM-14 | R8 — chunk `civ_hierarchical_network_topology`'s parallel Dijkstras | done | `civ_hierarchical_network_topology` carries R8 comments: each settlement's `dist` is kept as a per-settlement probe row rather than a whole grid, and `res1` is released after pass 1. Landed in `4ec07f5` |
| MEM-15 | Per-segment overlay culling (still open after `_run_offscreen`) | done | `map_overlay.gd::_segment_chains` returns the maximal runs of on-screen segments, so a long way crossing the window is no longer dashed in full. Probe `godot-project/_segcull_probe.gd`. Landed `af28882` (2026-09-05) |

**Group total: 15 — 13 done, 1 not started, 1 declined.** Corrected
2026-09-23: MEM-10, -11, -13, -14 and -15 each said "not started" for about
three weeks after they landed; each was re-opened at its symbol for this
correction.

§6's walk-down table projects 618.28 → 469.56 MiB for all eight R-changes.
R1-R5, R7 and R8 have landed; R6 has not. No after-figure for the landed set
has been re-measured here. The last measured point this file carries is
R1-R3's 518.92 MiB.
### LOD and tiling · `LOD_TILING_BASE_SCOPE.md` + `LOD_TILING_INTEGRATION_SCOPE.md`

`ROADMAP.md` files this under "Not a phase"; that section *originally ended*
*"revisit when a concrete need appears rather than building it speculatively"*,
and now says so and points at the scope documents (corrected here 2026-09-24:
this said ROADMAP "still says" it). **The deep-zoom pyramid was built, is
wired, and is on screen. The atlas is written and not read back** — see LODI-M3.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| LODB-1 | Base — new crate `cartalith-spatial`: `TiledField`, packed `QuadTree`, `DirtyTracker`, serde round-trip | done, **two of its three structures since retired** | Built as specified. On 2026-09-22 `TiledField<T>` and `QuadTree<T>` were **retired** (`5c99cc9`: a full-workspace grep found no real caller for either, and their 16 tests went with them). **`DirtyTracker` is kept** because it has real callers, and is still in `cartalith-spatial/src/lib.rs` with caller-supplied reason strings and a per-tile `version`. Corrected 2026-09-23: this row still cited `TiledField` and `QuadTree` as present. The crate's own test count moved with the retirement and is not re-quoted here |
| LODB-2 | Integration — the tool system picked the base up (2026-08-18) | done | `cartalith-spatial/src/pass.rs::PassBuffer<S>` and `staleness.rs::StageGraph`, built on `DirtyTracker` (`PassBuffer` does its own tile arithmetic; `TiledField` is retired, see LODB-1). **Now depended on by five external crates, not one**: `cartalith-civ`, `-engine`, `-godot`, `-io`, `-terrain` each list it. The crate has also grown modules the document does not mention: `contour`, `geo`, `measure`, `paint`, `pyramid`, `region` |
| LODI-M0 | Integration M0 — confirm Z1 needs nothing once the camera lands | done\* | Verification, not new work; the confirming pass is a device measurement — **not checkable from code** |
| LODI-M1 | Integration M1 — a minimal interactive Z2: tile the deep-zoom case only | done | `cartalith-spatial/src/pyramid.rs` (`pyramid_dims`, `pyramid_tile_bounds`, `pyramid_level_for_zoom`, `tiles_in_view`); `cartalith-terrain/src/amplify.rs` (`amplify_region`, `add_zoom_detail`) and `tile_render.rs::shade_tile` (*since LOD-D1/D2 off the deep-zoom path: its only callers are `cartalith-engine/examples/compute_config_bench.rs` and `tile_render.rs`'s own tests, re-checked 2026-09-24; tiles now go through `render_biome_tile_rgba`*); `cartalith-godot/src/lod_bridge.rs` (1 846 lines, `wc -l` 2026-09-24; was 783); `engine_bridge.gd`'s `lod_level_for_zoom` and `lod_synthesize_tile`; the deep-zoom tile scheduler in `shell/viewport_host.gd` with `shell/lod_tile.gdshader`. The 2026-08-19 bug-fix pass (dropped tiles never reconsidered once the camera stopped) is recorded in the scope document and its fix is in the scheduler |
| LODI-M2 | Integration M2 — nothing; the Data manager export panel, not a new milestone | declined | Declared not a milestone by the document itself. The export panel exists (`shell/data_manager_window.gd`) |
| LODI-M3 | Integration M3 — atlas cache (Z5) | done, no consumer | `cartalith-io/src/atlas.rs` — `AtlasStore`, `encode_chunk`, `put`, `put_meta`; plus `cartalith-engine/src/bake.rs`. Deferred in the plan until M1 shipped and was kept; both conditions were met. **Nothing reads the atlas back** (added 2026-09-24, alignment audit Part 2 C5): the reader `WorldGen::atlas_tile_png` is wrapped by `engine_bridge.gd` and called by no shell file (only `_bake_probe.gd`); `atlas_is_covered`'s own doc says it waits on that reader; the deep-zoom layer builds every tile from `lod_synthesize_tile`; `menus.gd`'s atlas menu header says the atlas is "still write-only". No backlog row tracks a reader |

**Group total: 6 — 5 done, 1 declined.**
Shell surface: `Preferences ▸ Tiles & LOD` ships LOD levels 0-8 and
auto/manual, both landed 2026-08-30 (`5f11b27`, `3338a79`).

### LOD detail · `LOD_DETAIL_SCOPE.md`

**Added 2026-09-23.** This group was missing, although `OUTSTANDING_WORK.md`
closed LOD-D0 to D6 on 2026-09-21. Each row's evidence is the commit and symbol
that milestone's closed backlog row cites, re-opened at the symbol for this
entry. Several milestones were closed with acceptance bars measured and **not**
met. By this file's vocabulary those are `partial`, and the rows say which bars.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| LOD-D0 | Zoom-sweep harness (measurement only, no pixel change) | done | `cartalith-godot/src/lod_sweep.rs` (metric definitions and unit tests) and the windowed `godot-project/_lodsweep_probe.gd`. `11936cb` |
| LOD-D1 | Port `renderBiomeTileRGBA` as a pure engine function | done | `render.rs::render_biome_tile_rgba`, golden-tested against the reference in `tests/golden_parity_tile_biome.rs` (worst delta 0). `f6d1bd5`. Two decisions it surfaced stay with the owner: the reference's tile and map shading disagree by construction, and single-thread synthesis measured above the scope's 40 ms budget |
| LOD-D2 | Colour tiles on screen, and the sharpness bar | partial — built; 3 of 6 D0 bars met | `lod_bridge::synthesize_tile_rgba` calls `render_biome_tile_rgba`, and `shell/lod_tile.gdshader` samples the colour directly. `9d2a800`. Not met at close: zoom-40 detail on the 2048 world (also failing before this milestone), the LOD-entry `mean \|ΔL*\|` bar at both sizes, and the seam ratio at 512 |
| LOD-D3 | Continuous transitions: parent fallback and a colour-space morph | partial — 2 of 4 bars met | `lod_bridge::morph_for_zoom`. `b6cc014`. Met: zero pops. Not met: worst level-boundary `T_i ≤ 1.5×` (1 of 12 still over), zero holes, and the seam ratio (unmoved) |
| LOD-D4 | Ice and snow from fields that already exist | partial — 1 of 4 bars met | `TerrainAppearance::ice_strength` and `render.rs::apply_ice_cover`. `02f6d51`. **The aspect term Ruling AP (2026-09-23) authorised is built** (`19c38d9`, 2026-09-24; *corrected the same day: this row said not built*): `TerrainAppearance::snow_aspect_c` (2.0 °C shipped, 0.0 under `js_reference()`, so no JS-parity golden moved; the Rust render hashes in `tests/color_space.rs` and `tests/layer_stack.rs` were re-baselined) and `render.rs::snow_aspect_shift`, fed to `material_weights` as a snow temperature shift, with `tile_snow_facing` on LOD tiles. **The snow-versus-aspect bar (1b) is still not met** after it, per the commit's own reading (not re-measured here), so the count stays 1 of 4. `snow_aspect_c` has no GUI control |
| LOD-D5 | Scale-aware shading weights, and hydrology that resolves | partial — 2 of 3 bars met | `TerrainAppearance::detail_scale_strength`. `c685930`. The unmet bar (detail per pixel non-decreasing from zoom 4 to 40) needs `add_zoom_detail`'s octave decay changed, a golden re-baseline recorded in `amplify.rs` as awaiting an owner ruling |
| LOD-D6 | Tile synthesis off the main thread, profiled per device | done\* — desktop bars met, phone bars unmeasured | `lod_worker.rs::LodSnapshot` and Rust-side `rayon` workers. `c74a150`. The phone frame-time and thermal bars need a handset and are not checkable from code |
| LOD-D7 | *(optional)* Debug and info views in tiles (`renderAffordanceTileRGBA`) | blocked — owner question 6 | Not ported: a grep of `crates/` and `godot-project/shell/` for `affordance_tile` / `renderAffordanceTile` finds nothing, 2026-09-23. The scope builds it **only if the owner wants it** |

**Group total: 8 — 2 done, 1 done\*, 4 partial, 1 blocked.**

### Save file and project archive · `SAVEFILE_COMPAT.md`

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| SF-1 | Reading the HTML app's `.zip` | done | `cartalith_io::load_save`; `tests/golden_parity_real_export.rs`. Same as MVP-7 |
| SF-2 | Writing a save (`ROADMAP.md`'s first "option kept open", closed 2026-08-23) | done | `cartalith_io::write_save` in `crates/cartalith-io/src/save.rs`, `WorldGen::save_project`, and the golden test `tests/golden_parity_save_writer.rs`. File ▸ Save / Save as… / Autosave / Revert / Close project are real controls |
| SF-3 | The project archive as a tree, carrying the whole project | done | *Recounted 2026-09-24 (alignment audit Part 1 C22; this said fifteen slots and that journeys were caller-owned).* `cartalith-godot/src/project_bridge.rs` defines **18** `SLOT_*` constants — `entities/{settlements,factions,ways,provinces,continents,landmarks,journeys,conflicts}.json`, `history/timeline.json`, `annotations/{labels,icons,regions}.json`, `appearance.json`, `vault.json`, `drafts/{paint,sculpt}.json`, `library/{assets,travel}.json` — of which **14** are in its `ENGINE_OWNED_SLOTS`, journeys among them (engine state since SP-1, JP-06/08). `cartalith_io::DOCUMENT_SLOTS` registers **20**: those 18 plus the shell's own `annotations/measurements.json` and `library/settlement_types.json` (`app.gd`), which travel through `project_save_with_documents`. Beside the slots: `rasters/`, `history/territory/<year>.i32` and the optional stored LOD pyramid under `cartography/tiles/` (`cartalith_io::project::LOD_TILE_PREFIX`) |
| SF-4 | `state.erosion` written to saves | declined | Only 2 of 16 keys are modelled by the reference's `loadZip()`, so it is deliberately not written rather than written partially. The limitation is disclosed in `SAVEFILE_COMPAT.md`'s own "Writing a save" section |
| SF-5 | Save compression — the byte-plane shuffle (27-36 % smaller, writes faster) | done | **Built 2026-09-23 under owner Ruling AJ** (`LARGE_ITEM_RULINGS.md`). `format_version` 1 → 2 (`cartalith_io::PROJECT_FORMAT_VERSION`); every 4-byte `rasters/` entry is written byte-plane shuffled under `<name>.shuffled.f32/.i32` (`project.rs`'s `write_planes`, `Raster::from_planes`, `raster_entry_name`, `SHUFFLED_INFIX`). **The fail-loud marker is the entry name**: a pre-shuffle reader finds no `rasters/heightmap.f32` and refuses rather than reading noise, and this reader un-shuffles only what it finds under the shuffled name, so every v1 save reads unchanged (`a_version_1_archive_reads_exactly_as_it_always_did`, over a fixture the unmodified v1 writer produced). `SAVEFILE_COMPAT.md` §8/§8.2 record the departure from the bare-dump promise; §18.6 measures it on a real save: 2.30 → 1.72 MiB at 512² (25.2%), 24.74 → 16.53 MiB at 2048×1311 (33.2%), 151.41 → 96.42 MiB at 4096² (36.3%), and the write 25-37% faster. `history/territory/<year>.i32` is not shuffled |
| SF-6 | Save compression — quantising saved rasters to `u16` | declined | **Owner Ruling AJ, 2026-09-23: not added.** Lossy; `PARITY_TESTING.md` and `DECISIONS.md` §7a bar it, and the ruling left that bar where it was. `SAVEFILE_COMPAT.md` §8.1 and §18.4 record the decision |

**Group total: 6 — 4 done, 2 declined.**
~~Four save slots are written but not yet read back by a caller; a fifth (saved
measurements) was scheduled by the 2026-08-31 rulings.~~ *Stale, re-checked
2026-09-24:* the draft and library slots are read back
(`project_bridge.rs` parses `SLOT_PAINT` / `SLOT_SCULPT` on open; `app.gd`
restores the two library slots through `asset_library_restore_document` /
`travel_library_restore_document`), and the fifth slot exists —
`annotations/measurements.json`, restored by `app.gd`. **What is written and
never read by the product is the stored LOD pyramid**, `cartography/tiles/**`:
`cartalith-io` loads and validates it into `lod_tiles` (dropping it when its
`source_key` does not match the world), and in `cartalith-godot` only the
save path writes that field and only `lod_worker.rs`'s tests read it.

### Android build and device · `ANDROID_BUILD_SCOPE.md`

Sixteen rows (*corrected 2026-09-24; this said twelve*). Most of this document's content is device measurement, which this
file cannot verify; the rows below separate the durable code artefacts from the
handset numbers.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| AND-1 | First real-device pass (2026-08-17): toolchain, build, install, launch, golden path | done\* | Toolchain wiring is verifiable: `godot-project/cartalith.gdextension` declares `android.debug.arm64` / `android.release.arm64`; `export_presets.cfg` carries the `Android` preset with `package/signed=true`; `[profile.android-dev]` exists in `cartalith-native/Cargo.toml`. The device run, logcat and meminfo numbers are handset measurements — **not checkable from code** |
| AND-2 | Second real-device pass (2026-08-18): re-verify the grown workspace | done\* | The durable artefact is the strip problem it hit, made permanent as `[profile.android-dev]` with a comment naming the 400 MB → 18 MB hand-strip and `debug = "line-tables-only"`. Memory/timing tables — **not checkable from code** |
| AND-3 | Third pass (2026-08-20): two config defects fixed, §13 phone layout first run on glass | done | Both fixes are in the tree. `project.godot`'s `[display]` section is written with `;` comments carrying the warning; `cartalith.gdextension`'s `android.debug.arm64` points at `target/aarch64-linux-android/android-dev/` — the directory the documented command actually writes — with a `;` comment block naming the exact refresh command per entry |
| AND-4a | Fourth pass defect 1 — portrait (orientation was a string, Godot 4 wants int) | done | `project.godot` `window/handheld/orientation=6` (SCREEN_SENSOR), under a `;`-commented block explaining that locking the OS orientation would make `_apply_phone_orientation()` unreachable |
| AND-4b | Fourth pass defect 2 — Open project: duplicated header and desktop sizing | done | `shell/app.gd` documents the phone treatment ("borderless, a content-scaled …") for PH-12; `borderless = true` appears on the window classes it applies to |
| AND-4c | Fourth pass defect 3 — light theme available everywhere | done | `shell/dcc_theme.gd` carries a full light half derived from the canvas's own `themeStr`, with per-token light readings documented in place |
| AND-4d | Fourth pass defect 4 — bottom sheet buttons scaled at the choke point | done | `shell/dcc_shell.gd::set_tool_options()` calls `phone_fit(tool_options_row, _phone_scale, true)` when `_phone`, then defers `phone_insets_changed` — the single choke point every workspace's tool row passes through |
| AND-4e | Fourth pass item 5 — the phone overflow menu | done | `shell/phone_menu.gd` exists (49 257 bytes, `class_name PhoneMenu`); its header names §5's four faults as what it replaces, and `dcc_shell.gd` declares `var _phone_menu: PhoneMenu ## L2-L5. Replaces the old _phone_overflow`. It re-presents `menus.gd`'s real `PopupMenu`s through `activate_item()` as a five-level drill-down rather than reparenting the desktop bar. *The document's "Done means" table still reads "Overflow menu — **Diagnosed, not fixed** (§5), by instruction", and §5 is still written as an open design brief. The fix landed under a different scope document and this one was never told* |
| AND-5 | Pinch-to-zoom pass (2026-08-24) | done | `project.godot` `pointing/android/enable_pan_and_scale_gestures=true` — the single setting the pass identified |
| AND-6 | Device pass — civ / urban / render windows on the phone (2026-08-24) | unverified | A device-driving pass; its findings live in `GUI_GAP_REGISTER.md` §22 PH-01 rather than in code this document owns |
| AND-7 | APK staleness: the silent `has_method` guard now speaks | done | `shell/engine_bridge.gd::_has(method: String) -> bool` with a `push_warning(` and once-per-name suppression, plus `missing_bindings() -> PackedStringArray` and a summary warn. The `.gdextension` refresh-command comment block (per-entry, `;`-commented, with the "`#` is parsed as DATA" note) is in `godot-project/cartalith.gdextension` |
| AND-8 | Device pass 2026-08-25 (§46/§47/§48 + ponytail LOD) and the SurfaceFlinger frame-time method | unverified | A measurement pass; the reusable output is the method write-up. **The build it verified is superseded** — `target/aarch64-linux-android/android-dev/libcartalith_godot.so` and `builds/android/Cartalith-lm.apk` are both dated 2026-08-30 |
| AND-9 | Positive control that `push_warning` reaches Android logcat | done | Measured 2026-09-07 (`88bf297`) through the app's own UI on a release build: `E/godot WARNING …` followed by `at: push_warning (core/variant/variant_utility.cpp:…)`, with three negative controls — the third being the same marker string appearing earlier as `W/FilesystemDirectoryAccess` with **no** `push_warning` line, so "the marker is in the log" does not pass on its own. Corrected 2026-09-23: this row read "not started" for 16 days after it landed |
| AND-10 | Landscape / rotation driven over adb | done | `project.godot` sets `orientation=6` (SCREEN_SENSOR), which respects the rotation lock, so `settings put system user_rotation` does nothing — but `adb shell wm user-rotation lock 1` does rotate with auto-rotate off, and the fourth pass (2026-08-20) captured landscape that way. Release with `wm user-rotation free` and restore `accelerometer_rotation` to `1`. The method is `ANDROID_BUILD_SCOPE.md` §2.1. Corrected 2026-09-23: this row read "blocked, needs the owner to rotate the handset" although the route had already been used |
| AND-11 | Release keystore / signed release export | declined | **Ruling 23**: store distribution and signing are a deliberate non-goal; stay on debug signing, the repo holds no secret. `--export-release` failing at signing is expected, and the unsigned APK it leaves is the good one. How the release APKs were signed is now recorded: the 2026-09-07 drop signed that unsigned APK with Godot's debug keystore via `apksigner` (`CN=Godot`, ~57 MB) — `ANDROID_BUILD_SCOPE.md` §1.3. Corrected 2026-09-23 from "not started" |
| AND-12 | APK cruft — development probe/shot scenes ship inside the APK; release profile unstripped | declined | **Half of this no longer holds.** The `_*` probe and shot scenes are excluded from the export since `686cd2a` (2026-09-03): `export_presets.cfg`'s Android preset has `exclude_filter="addons/godotsteam/*,addons/godot_ai/*,_*"`. They still sit in `godot-project/` for development; they do not ship. What remains is the owner's call: `cartalith-native/Cargo.toml` has no `[profile.release]` section, so the release `.so` is not stripped |

**Group total: 16 — 12 done (2 of them `done*`), 2 unverified, 2 declined.** *Recounted from the rows 2026-09-24; this said "10 done, 2 unverified, 2 not started, 1 blocked, 1 declined", which neither matched the rows nor summed to 16.*

### DCC shell · `DCC_SHELL_SCOPE.md`

**Six** rows — the header said five while the table listed six and the group
total below it said six; corrected 2026-09-06. All complete — but **milestones 1
and 2 cite files that no longer exist**, which makes them unverifiable at face value even though the capability
survived.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| DCC-1 | GUI milestone 1 — the six-region DCC shell (menu bar, workspace tabs/rail, tool options bar, tool rail, viewport, right dock, status bar) | done, **evidence re-pointed** | `project.godot` `run/main_scene="res://shell/app.tscn"`. `shell/dcc_shell.gd` declares and builds `menu_bar_row`, `tool_options_row`, `left_dock`/`left_dock_body`, `right_dock`/`right_dock_body`, `timeline_row`, `status_row`, plus `_build_menu_bar()` and `_build_rail()`. **The document's evidence is written entirely against `main.gd`/`main.tscn`/`map_overlay.gd` and against a 7→8 menu change producing File/Edit/Generate/Simulate/Render/Assets/View/Help. Neither `main.gd` nor `main.tscn` exists, and the menu bar is now seven menus with no Generate, Simulate, Render or View** (`menus.gd` adds File, Edit, Assets, Data, Preferences, Window, Help) |
| DCC-2 | GUI milestone 2 — the Generate menu's six live stage parameter dialogs (57 controls) | done, **re-homed** | There is no Generate menu and no `main.gd`. The parameter surface moved into the WORLD workspace's ten-stage dock: `shell/workspaces/world_workspace.gd` carries the ten-stage table and builds live parameter rows per stage from the engine's own table (`_build_param_row`, `_build_erosion_passes`, `_build_droplet_erosion`); `cartalith-godot/src/params.rs` is the flat dotted-key API those rows read. The milestone's "no per-stage staleness indicator" decision is superseded by the real staleness slot |
| DCC-3 | GUI milestone 3 — the World Setup dialog (map size, resolution, dimensions, aspect) | done | `shell/new_world_dialog.gd` builds the Extent section, the "Map width & resolution" section (Map width preset / Resolution / Aspect rows) and the derived `Grid` / `Extent` / `Cell size` / `Aspect` readout. `func request() -> Dictionary` is what `app.gd` hands to `bridge.import_heightmap(...)` and to generation |
| DCC-T2 | Tool-track milestone 2 — write `UNIFIED_TOOL_PLAN.md` for real (planning only) | unverified | The deliverable is the document itself. It exists at the repository root — 1 538 lines after the 2026-09-24 condensing pass (`wc -l`; this said 2 268) — with the reference's Sculpt editor read out, the pass-buffer model, the tool-by-tool table and the A-F breakdown |
| DCC-T3 | Tool-track milestone 3+ — the tool system itself | done | *The document says "Milestone 3+ (not yet dispatched)", repeated at the end of the milestone-1 entry. It was dispatched and completed* — this is UTP-A…UTP-F above, all in the tree |
| DCC-P412 | The 412 dp phone migration (geometry from `Cartalith Android Phone.dc.html`, content from Menu Structure v3) | done | `dcc_theme.gd`'s `PHONE_REF_SHORT` and the phone density set; `dcc_shell.gd::_build_phone_menu_bar()` and `_phone_menu: PhoneMenu`; the ☰ side drawer is gone in favour of the domain drill. Probe present: `godot-project/_ph412_probe.gd` |

**Group total: 6 — 5 done, 1 unverified.**

### GUI feature parity · `GUI_FEATURE_PARITY_SCOPE.md`

Eight milestones. **Seven are done and one, GFP-2, is partial.** GFP-2's
largest item (`PopupMenu` styling) was solved by another route, and its
tooltip and scrollbar chrome are still Godot stock: a grep of
`godot-project/shell/` for `TooltipPanel`, `VScrollBar` and `HScrollBar` finds
nothing, 2026-09-23. (Corrected 2026-09-23: this line used to call the eighth
milestone "superseded by another route", which contradicted the table and the
group total below.) The document still reads as an open plan and should be
closed out.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| GFP-1 | 1 — Category 1 sweep (10 rows of real-backing-needs-wiring) | done | Row 5 (faction culture-terrain-fit): `WorldGen::civ_faction_terrain_fits()` over `civ_faction_aggregates` + `civ_culture_terrain_fit`; `engine_bridge.gd` wraps it; `faction_roster_window.gd::_build_terrain_fit()` is the panel. Row 7 (GPU toggle): `engine_bridge.gd` sets `param_set("use_gpu", true)` as the shell default with `Preferences ▸ GPU acceleration` able to turn it off, and `gpu_stages_used()` over `WorldGen::get_gpu_stages_used`. Rows 1/9/10 are in the DCC shell (asset import, per-layer toggles in `layers_popover.gd`, click-to-pin in `right_dock.gd`). *The outcome table still records rows 5 and 7 as open, which is harder to spot than a plain to-do because it is presented as a closed milestone's result* |
| GFP-2 | 2 — Category 4 visual-consistency sweep (`PopupMenu` / `Tooltip` / `ScrollBar` entries in `dark_theme.tres`) | partial | `theme/dark_theme.tres` contains **no** `PopupMenu`, `TooltipPanel`, `TooltipLabel`, `VScrollBar` or `HScrollBar` entry (grep → zero hits). **PopupMenu was solved by another route** — programmatically, via `DccWidgets.style_popup()` reached through `DccShell.style_popup()` — so the milestone's largest item is closed elsewhere. **Tooltip chrome and scrollbar grabbers are still Godot stock**; the grabber overrides in `dcc_widgets.gd` are `HSlider`, not `ScrollBar`. The milestone as written targets a resource the DCC shell no longer drives its chrome from |
| GFP-3 | 3 — stale-field tracking | done | `cartalith-spatial`'s `StageGraph`/`DirtyTracker` behind `WorldGen::stale_stages()`; `app.gd::_setup_staleness()` (1 s poll, `Recompute` action in `status_row`) and the corrected header comment recording that the *tools*, not the dials, are what leave staleness behind |
| GFP-4 | 4 — Category 2 small items (heightmap import, GeoJSON export, CPU/memory readout, route-corridor/travel-cost fields) | done | Heightmap import: `WorldGen::import_heightmap` reached from `app.gd` and offered as a live row in `data_manager_window.gd`. GeoJSON export: `geojson_bridge.rs` over `cartalith_engine::geojson::export_geojson`, surfaced as `data_manager_window.gd`'s `export_gis`. Readout: the Working set row (`menus.gd::_refresh_working_set_row`) and the menu bar’s `top_mem` slot in `app.gd`; `performance_window.gd` was deleted under ruling 19 (`06c8963`). Travel cost (per-settlement string): `journey_bridge.rs` and `journey_planner_view.gd`. **Fully closed 2026-09-01**: route corridors / travel cost as a *selectable analysis field* shipped — `sample_bridge.rs`'s `LAYER_GROUPS`/`legend()`/`debug_raster()` all carry `corridor` and `travel_cost` ids now, each with a dedicated fixture test proving the ramp is actually reached (`corridor_view_reaches_a_real_pass_and_marks_water_distinctly`, `travel_cost_view_spans_its_ramp_and_marks_water_impassable`), reachable through the existing Layers popover with no GDScript change needed |
| GFP-5 | 5 — Terrain appearance GUI | done | `shell/workspaces/render_workspace.gd` (2 140 lines at `919bce1`, counted 2026-09-24; this said 1 055) builds the appearance groups against `WorldGen::{get_appearance, set_appearance, list_appearance_tunables, reset_appearance}`, owns the ramp editor and preset table, and is composed into CARTO rather than owning a rail button |
| GFP-6 | 6 — Faction roster + `_civFactionAggregates` GUI | done | `shell/faction_roster_window.gd` (`class_name FactionRosterWindow`, 36 821 bytes) reads `bridge.get_factions()` and `bridge.civ_faction_terrain_fits()`, with `_build_terrain_fit()`. Engine side: `civ_roster_bridge.rs` and `civ_military_bridge.rs` |
| GFP-7 | 7 — Category 3 build-recommended remainder (layer opacity, measurement tool, quality tiers) | done | Layer opacity: `shell/layers_popover.gd` driving `host.set_debug_opacity`. Measurement: `global_tools.gd` registers `"measure"`; `measure_bridge.rs` + `infra_tools_bridge.rs::measure_legs`; `tool_overlay.gd` draws the ruler chain with area/radius modes. Quality tiers: `engine_bridge.gd`'s `quality_tier()` / `set_quality_tier()` / `quality_tiers()` |
| GFP-8 | 8 — large Category 2 items (Journey Planner GUI, Asset Library UI, tile/LOD viewport) | done | Journey Planner: `shell/journey_planner_view.gd`, 153 772 bytes. Asset Library: `shell/asset_library_window.gd`, 160 685 bytes. Tile/LOD viewport: `viewport_host.gd`'s `_build_lod_tile()`, `_lod_layer`, `_lod_backlog`, `_lod_debug_layer` with `shell/lod_tile.gdshader`, over `lod_bridge.rs`; probes `_tiledlod_probe.gd` and `_lodlevels_probe.gd` exist |

**Group total: 8 — 7 done, 1 partial.**
GFP-4 closed in full 2026-09-01. **One thing survives as open work from this
document**: GFP-2's tooltip/scrollbar chrome. Every other milestone, including
the never-attempted per-stage slider audit this document names in its own
closing paragraph as future work rather than a milestone, is either done or
correctly out of scope — this document should be closed out.

### GUI replacement, 2026-08-31 · `design/dcc-environment-2026-08-31/spec/00-REPLACEMENT-PLAN.md`

Eight rows: the §0 blocker and seven stages. **Stages 1, 2 and 4 landed; 3, 5,
6 and 7 are unblocked and unstarted.**

| ID | Stage | Status | Evidence |
|---|---|---|---|
| RP-0 | §0 blocker — the desktop prototype arrived truncated at 262 144 bytes, 84 UNSPECIFIED items | done | `design/dcc-environment-2026-08-31/Cartalith DCC Environment.dc.html` is now **239 712 bytes** and ends properly with `</script></body></html>`; the heavy method bodies moved to `cartalith-dcc-parts.js` (54 059 bytes) behind `window.CDCC`; `statusMid` appears in the prototype. Commit `660cbef` records the split. **§5.1 and §5.3 still ask the owner to re-export the file and to say what `statusMid` shows; both were answered the same day and neither was struck** |
| RP-S1 | 1 — tokens: new `--ins`/`--wash` values, new `--accInk` and `--wash2`, four density sets incl. LAPTOP 1366 | done | `dcc_theme.gd`'s `"sunken": Color("#191c1e")` with the header recording the `#101112 -> #191c1e` change; `accent_ink` and `accent_wash_2` documented as new; the fourth density set (`W_LAPTOP_MAX`, `const LAPTOP`) with `is_laptop()`. Landed in `c03b43c` |
| RP-S2 | 2 — the rail, five domains to three, plus the node tree and a mode-carrying `select_domain_category` | done | `dcc_shell.gd`'s `const DOMAINS` holds exactly world / civilization / cartography; `const RAIL_NODES` holds **3 `kind: head` rows and 10 `kind: node` rows** (counted in the file) with `mode`, `category` and `owns` keys, matching the plan's table. Guard probe `godot-project/_railfold_probe.gd` (427 lines) asserts node→category reachability and that every category appears in exactly one node's `owns`. Landed in `c03b43c` |
| RP-S3 | 3 — menus, restyled to the new tokens | not started | `shell/menus.gd` is not in `c03b43c`'s file list, and its current working-tree diff is `UNWIRED_FUNCTIONS.md` wiring work, not a restyle. Note that the token re-base propagates automatically wherever menus read `DccTheme`, so part of this stage may already be moot. `_cmdindex_probe.gd` is present as the guard the plan names |
| RP-S4 | 4 — left dock, mode by mode, against `spec/04-left-dock.md` | done, §6d restyled not embedded | CARTO's four destinations (including the new LABELS and ICONS panels) were built out of order during stage 2 (`cartography_workspace.gd`, +357 lines in `c03b43c`). **2026-09-01 morning: WORLD's two modes and CIVIL's Landmarks and Factions & settlements categories** were checked against the spec and the live engine, then restyled into conformance — a deliberate *restyle, not rebuild*: `dcc_shell.gd`'s `RAIL_NODES` header's shipped rule ("each domain's dock is ONE accordion of every category that domain owns … a node click *opens* its category, it never hides a sibling") was kept over the prototype's `ldPipe`/`ldSculpt`/`ldCarto`/`ldLabels` mode gates, which would strand the 33-category rail-fold contract `RP-S2` committed to. Landed: a real `F` shortcut for the Freehand feature chip (`world_workspace.gd`), an accordion-floor fallback so closing CIVIL's open category never leaves none open (`civilization_workspace.gd::_lm_enforce_floor`), Landmarks as CIVIL's default-open category matching `Default civCat = 'landmarks'`, and inspect-arm-on-settlement-select. **2026-09-01 second pass: CIVIL's Ways & routes and Journey planner closed**, both in `infrastructure_workspace.gd`. §6c: a real `ROUTES` teaser list under the Network group (`_build_routes_teaser`/`_refresh_routes_teaser`/`_routes_teaser_row`, `:862-934`) — one row per committed route (glyph, name, nearest settlement at each end by Euclidean distance, km), clicking opens the Journey Planner through the one shared `app.open_journey_planner()` entry point; kept in step with the editable routes list via a hook in `_refresh_manual_routes()` (`:1008-1020`). §6d: **deliberately not embedded** — `_fill_logistics()`'s doc comment (`:1213-1234`) reasons that the full TRAVELER/SEASON/CARRIAGE/ROUTE/STOPS accordion lives in `journey_planner_view.gd`'s private fields with no exposed accessor, so embedding it here would either bind to nothing or reach into another file's state with no shared contract; the dock instead names the five parameter groups honestly and opens the same shared planner. Verified independently: `_railfold_probe.tscn` **PASS**; `_deadwire_probe.tscn` **DONE fail=0**, `Workspace[civilization/Routes & ways]` and `Workspace[civilization/Travel]` both `0 UNWIRED, 0 dead-silent, 0 gated`; `--headless --check-only` clean. **One disclosed, non-blocking gap:** neither the ROUTES row nor the "Open Journey Planner" button can preselect which route the planner opens to — both tooltips say so; the planner always opens to its own default (route #1 or the most recently saved journey). Fixing that needs a small addition to `journey_planner_view.gd`'s `open()`, not attempted this pass |
| RP-S5 | 5 — right dock, tool options, status, timeline, viewport furniture | not started | **The plan marks this "*Blocked* until the desktop file is re-exported"; that blocker was cleared on 2026-08-31 by RP-0.** It is unblocked and simply not started: `shell/right_dock.gd` (1 969 lines) and `shell/tool_bar.gd` (606 lines) carry no new-spec contexts, `tool_bar.gd` is absent from `c03b43c`, and the `--tbH` 34→40 change is not in `dcc_theme.gd`'s shipped tool-options height. `statusMid` on the shell side is the pre-existing generation-stage readout, not the design's |
| RP-S6 | 6 — phone, from the complete `spec/06-phone.md` | not started | The phone shell in the tree is the 2026-08-25 412 dp migration (DCC-P412), built from the older `Cartalith Android Phone.dc.html`; `06-phone.md` (90 257 bytes) is a newer authority that has not been read into code. `shell/phone_menu.gd` is absent from `c03b43c`; `dcc_shell.gd`'s phone half still carries the 2026-08-25 constants |
| RP-S7 | 7 — the nine windows, restyled to the new tokens | not started | `faction_roster_window.gd`, `city_viewer_window.gd`, `place_editor_window.gd`, `world_data_window.gd`, `performance_window.gd`, `vault_window.gd`, `travel_library_window.gd`, `data_manager_window.gd`, `asset_library_window.gd` are all untouched by `c03b43c` (`performance_window.gd` has since been deleted under ruling 19, `06c8963`) except a 5-line change to `place_editor_window.gd`, and carry no new-token pass. The plan itself notes the prototypes do not specify these windows — they keep their implementations and are re-pointed at the new frame |

**Group total: 8 — 4 done, 4 not started.**

Two further defects in this plan, recorded rather than carried: **§2's line
counts are wrong by up to 41 %** (`dcc_shell.gd` listed at 4 339 and measuring
6 113; `dcc_theme.gd` 739 → 1 188; `menus.gd` 2 758 → 3 438; `right_dock.gd`
1 759 → 1 969; `app.gd` 1 859 → 2 521; `cartography_workspace.gd` 1 192 →
1 549), and the table is presented as a **sizing input for stages that have not
run yet**, so it under-states them. Some of that drift is stages 1-2's own work
landing after the plan was written.

### Superseded desktop shell · `GUI_SHELL_SCOPE.md`

Four rows. **History only.** The shell this document built no longer exists;
status is `declined` in the sense of *superseded and removed*, not of work
outstanding.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| GSS-1 | 1 — desktop panel-browser shell (top bar, 4-group navigator, layer panel, mode bar, right inspector, bottom bar) | declined (superseded) | `godot-project/main.gd` and `main.tscn` are absent; `project.godot` boots `res://shell/app.tscn`; the navigator/mode-bar structure was replaced by the DCC rail (`dcc_shell.gd::_build_rail`) and workspaces |
| GSS-2 | Cleanup pass — eliminate top-bar / navigator duplication (Map ▸ Layers removed) | declined (superseded) | Not checkable against today's tree — the pass edited `main.gd`'s `_build_menus()` and `_on_map_menu_id`, neither of which exists. There is no Map menu; layers live in `shell/layers_popover.gd` |
| GSS-3 | Second workflow re-audit against the GUI mockup | declined (superseded) | It re-read `main.gd`/`main.tscn` against `design/Cartalith GUI.dc.html`; all three are superseded (the design sources are now `design/dcc-environment-2026-08-31/`) |
| GSS-4 | GUI decluttering pass — target information architecture (`NAV_GROUPS`: WORLD / CIVILIZATION / CARTOGRAPHY / EXPLORE) | declined (superseded) | The IA that shipped is three `DOMAINS` (WORLD/CIVIL/CARTO) over a ten-node rail tree, by owner ruling 2026-08-20 and rebuilt 2026-08-31. **No `NAV_GROUPS`, `NAV_SUBJECT_HINTS` or EXPLORE group exists anywhere in `godot-project/`** |

**Group total: 4 — 4 declined (superseded).**
This document's header **no longer** asserts *"UI work is now on hold entirely
(owner, 2026-08-18)"*; that hold was lifted later the same day. Corrected
2026-09-23: this paragraph said the header still carried the sentence, but it
was removed on 2026-09-01 in `fd9de7c`. Checked with `git show`: one
occurrence of "on hold" in `GUI_SHELL_SCOPE.md` at `fd9de7c^`, none at `fd9de7c`,
and none today. The header now opens with a **SUPERSEDED** notice pointing
here.

### Export · `EXPORT_SCOPE.md`

Five milestones. **Shelved by the owner 2026-08-25, un-shelved by ruling 15
on 2026-09-06, and resumed by Ruling AP on 2026-09-23** (corrected 2026-09-23:
this section said all five were still shelved and unbuilt, which had been false
since E1 landed on 2026-09-22). Re-verified at the symbols 2026-09-23.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| EXP-E1 | E1 — the banded terrain renderer (`ExportBandPlan`, `apply_local_contrast` / `build_grade_influence` splits, `tests/export_bands.rs`) | done | `render.rs` — `ExportBandPlan`, `bake_export_band`; `crates/cartalith-godot/tests/export_bands.rs` present. `export_raster.rs`'s `BAKE_WIDTHS` is now `[2048, 4096, 8192, 16384, 32768]`. Landed `de3c95e` (2026-09-22) |
| EXP-E2 | E2 — the streaming writer (PNG first, BigTIFF second, band-in / file-out) | done | `crates/cartalith-godot/src/export_stream.rs` pulls bands from `bake_export_band` and writes each before dropping it; PNG and BigTIFF (`tiff` crate). Landed `fc7db2b` (2026-09-23) |
| EXP-E3 | E3 — the options struct (one dictionary: width, format, style override, content set, settlement tier) | done | `crates/cartalith-godot/src/export_options.rs`; `#[func]`s `export_image` and `export_image_estimate` in `export_raster.rs`. Landed `ce2c71d` (2026-09-23). No shell caller yet (that is E5) |
| EXP-E4 | E4 — the overlay session (cross-frame begin / band / composite / write / finish) | partial | **Batch A of four landed** (`34db87f`, 2026-09-23): `export_session.rs` (`ExportSnapshot`, `ExportSessionCore`) and five `#[func]`s in `export_raster.rs` — `export_session_begin` / `_submit_tile` / `_finish` / `_abort` / `_state` (recounted 2026-09-24; this named three) — Rust core only. **Batches B-D are not built**, and no `.gd` file under `godot-project/shell/` calls any `export_session_*` (grep, 2026-09-23). Still **the one milestone with no reference behaviour to port against**. **Two owner questions remain open under Ruling AP**: E4's five scope questions (overlay content, label/road/river detail, which settlements, UI-freeze tolerance, river stroke vs bake) and the 32K size/codec trade-off. The second conflicts with ruling 26 (2026-09-06: PNG, "even if size balloons"), and a 32K PNG measures 213.9 MB (`EXPORT_SCOPE.md`). `OUTSTANDING_WORK.md` flags that for the owner |
| EXP-E5 | E5 — the export dialog | not started | No `.gd` file under `godot-project/shell/` calls `export_image` or `export_session_*` (grep, 2026-09-23). `data_manager_window.gd` still drives the older `export_raster_png` path at 2K-32K (ruling 15) |

**Group total: 5 — 3 done, 1 partial, 1 not started.**
What is left, and what remains open for the owner, is `OUTSTANDING_WORK.md`
§2.3's export row. Codec survey conclusion, kept because it is expensive to
redo: WebP is eliminated at 16 383 px and JPEG XL at its AGPL encoder.

### Gap register · `GUI_GAP_REGISTER.md`

**Read this document as history.** Its ID total was re-counted three times
(123 → 215 → 300) and its A/B/C/D open/closed split was never re-derived once;
a class marker survives on only 54 of 215 rows, so the register cannot say how
many of its own IDs are open. `UNWIRED_FUNCTIONS.md` is the live successor,
re-cut 2026-08-31 against the three-domain shell.

| ID | Section | Status | Evidence |
|---|---|---|---|
| GGR-10 | §10 — the actionable (A) list, twelve ranked rows | done | Sampled rows check out: light theme (PR-13/14) → `dcc_theme.gd`'s `const LIGHT` and `var pal: Dictionary = DARK if _dark else LIGHT`; RD-06/08 → `faction_roster_window.gd`'s `bridge.get_factions()`; JP-13/14 → `journey_planner_view.gd`; CA-05 icon resize → `icon_bridge.rs::icon_handle` + `cartography_workspace.gd`'s `"icon"` drag handler. SH-01 is recorded done-then-withdrawn, which the rail's own header confirms ("Removed 2026-08-24 with the expansion itself") |
| GGR-49 | §49 — headed "REGISTERED, NOT FIXED"; **all three findings are now fixed** | done | KV-04: `shell/vault_store.gd::save_from()` now parses only to validate and splices the engine's own string into the document unchanged (`"store": state`), with a comment naming `entity_id` / `source_modified` and the silent-loss failure — §49's own "stop re-parsing" fix. WW-16: `world_workspace.gd` replaces the stage-06 `gap` string with a note dated "Corrected 2026-08-30 … it was stale on six of its seven claims", and `_build_droplet_erosion(...)` gates on `bridge._has("erode_op")`, i.e. a live `#[func]`. CV-12: `civilization_workspace.gd`'s button is still correctly disabled and its tooltip was rewritten ("the previous wording ended 'the crate has no consumer at all', which was false") to name the real per-line blocker — urban milestones 9, 10 and 13. *Superseded, re-checked 2026-09-24:* CV-12 is no longer a disabled button — `civilization_workspace.gd::_build_settlement_diagnostics` draws a per-settlement card list from `engine_bridge.gd::settlement_diagnostics` (specialisation, fortification rung, river classification, harbour eligibility), with the reference's bridge/ford/harbour-validity line disclosed as absent |
| GGR-58 | §58 / PH-28 — a 16:9 tablet is classified as a phone and its controls are clipped off the screen | done | §58 says "**Not applied here** … this pass has a measurement, not a mandate". **It was applied**: `dcc_shell.gd` now reads `_phone = _touch and (short_side / long_side) < _PHONE_ASPECT_MAX and not _is_tablet_sized(short_side)`, with `const _TABLET_MIN_DP := 900.0` and `_is_tablet_sized()` returning `short_side_px / (dpi / 160.0) >= _TABLET_MIN_DP`. §58 proposed ~600 dp (Android's `sw600dp`); commit `660cbef` records the owner ruling **900 dp** instead, on the 48 dp rail + 400 dp dock chrome-floor arithmetic. Both the status and the number in §58 are wrong |
| GGR-53 | §53 — four registered-not-fixed phone items (`⌕` and `⋮` not in the app bar, bottom-nav/sheet colour literals off by 2/255, tool-options sheet still resident, phone pill upper-cases once) | partial | **The `⌕` half is unblocked and its stated reason is stale**: §53 says "`⌕` has no destination — `menus.gd`'s Edit ▸ Find on map… is a `_todo()` row". It is now `_live(p, "Find on map…", ID_FIND_ON_MAP, KEY_MASK_CTRL \| KEY_F)` backed by `shell/place_search.gd`. The `⋮` overflow still exists as floating phone furniture (`_phone_overflow_pop`), the two colour deltas are unchanged, and `dcc_shell.gd` still calls `phone_fit(tool_options_row, …)` on a resident row rather than presenting a sheet |
| GGR-50 | §50 — six registered-not-fixed items from the OnePlus 6T pass | partial | **The memory item is disclosed, not fixed**: the Working set row (`menus.gd::_refresh_working_set_row`, which replaced `performance_window.gd` under ruling 19) still reports `OS.get_static_memory_usage()`, which excludes the Rust allocations and the ~544 MB of Gfx dev that `dumpsys meminfo` counts, with a note naming the source and §50's own handset figure (0.2 GB on screen vs 818 MB PSS). The other five (Label ellipsis hole, DS-12 duplicate class token, navpad hover tint, stock focused pane-switcher chip, two `✕` on the L2 sheet header) were not re-verified individually this pass |
| GGR-51 | §51 — the registered-not-fixed menu-conformance set | partial | Mixed, and worth splitting. **Settled by a newer authority**: the tool-options bar height, now `--tbH` 40/56 in the 2026-08-31 spec, superseding §51's "third owner decision". **Disclosed rather than fixed**: `tool_bar.gd` builds the freehand row from `bridge.get_sculpt_freehand_modes()` live and adds the note "The canvas's Flatten, Noise and Mask have no engine mode at all; these eight are `FreehandMode`'s own list, read live." **Unchanged**: the badge right-column (a real Godot `PopupMenu` limitation), the tablet dock contents (that is DS-03), and the sculpt feature-name gap (an engine gap). Two rows §51 listed were already proven stale by §53's own probe |
| GGR-DS03 | DS-03 — the tablet interior is desktop-sized; the tablet artboard is a content decision, not a scaling layer | blocked | An owner **content** decision — *which controls leave the tablet* — not answerable from the canvas. Under it sits a real architectural blocker: `dcc_theme.gd`'s `const TABLET := {36: 52, 34: 52, 30: 44, …}` is keyed by the bare desktop integer, and the artboard maps one desktop figure to two tablet figures in at least five places, so a value-keyed table cannot express it; §57 also refuted the obvious role-keyed placement. One mechanism claim in §51 has drifted: `phone_fit()` no longer opens `if not _phone: return` — the gate moved to its callers — but the **verdict** DS-03 rests on is still correct |
| GGR-DS13 | DS-13 — the phone viewport control column (navpad, zoom, layers FAB) does not match the canvas | not started | §57's pass produced a design and its own audit refuted it on four high-severity counts. Two re-confirmed here: `viewport_host.gd`'s left-button branch is `elif mb.button_index == MOUSE_BUTTON_LEFT and _pan_mode:` with no armed-tool condition (so `tool_pan` is navigation after all), and `_build_navpad()` opens `if not _touch: return`, so touch tablets get the same pills. `menus.gd` has no View menu, confirming that zoom's stated destination does not exist. **Nothing was built and the register says so** |
| GGR-RELIG | The religion-diffusion screens (§57) — designed, audited, refuted as premature | **stale — corrected 2026-09-23, a 2026-09-03 batch already shipped the data path this row says is missing** | This row's own "no data path" claim was already false when written: `get_settlements()` emits `religion` and `adherents` per settlement (confirmed at the symbol, `RD-1` above), and a 2026-09-03 batch shipped the religion screens against that data — `OUTSTANDING_WORK.md`'s own record of that batch is the correction this row should have carried since. Left as a defect-in-the-record example rather than silently deleted, per this file's own discipline of naming what it got wrong |
| GGR-05 | §5 — omissions: designed, not present, not even as a disabled item | unverified | Not re-verified item by item. §37, §39, §42 and §45 close large parts of it and §5 was never rewritten to reflect that, so its current accuracy is unknown. Confirming each absence requires the canvas alongside the shell — **flagged rather than guessed** |
| GGR-BULK | §15-§56 — the fixed/closed body of the register (RF-01, SB-01, BK-01/02, IN-10/11/12, CA-13, SH-01, MT-01, RD-01/01b/02, MR-01…03, TO-01/02, CV-20/23/25/26, MN-09, SH-15, KV-01…03, FR-02, PE-01, SH-11, WW-13, IN-13, VA-01, ED-02, MN-10, RL-01, CA-20, RF-02…05, FI-04, PH-12…PH-27, HD-01…04, DS-01…14, MEM-01…04, FX-01…03, BI-01) | done, **spot-verified only** | Every spot check held: SH-01's withdrawal is recorded in `dcc_shell.gd`; MEM-02's culling fix and the LOD backlog are in `viewport_host.gd`; the light theme is `dcc_theme.gd`; the 412 dp phone set is `dcc_shell.gd`'s phone half plus `_ph412_probe.gd`. The named probe harnesses are all present in `godot-project/` (`_rf01_probe`, `_backnav_probe`, `_in13_probe`, `_phonechrome_probe`, `_tabletparity_probe`, `_hidpi_probe`, `_flowzoom_probe`, `_cull_probe`, `_menuconf_probe`, `_railalign_probe`). **Not individually re-derived** |

**Group total: 11 — 3 done, 3 partial, 1 blocked, 1 not started, 1 unverified,
1 done-spot-verified, 1 stale — corrected (GGR-RELIG).** *(Recounted from the
rows 2026-09-24: this said 2 blocked and omitted GGR-RELIG.)*

### Options kept open · `ROADMAP.md`

Neither is work until someone commits to it. Both are owner decisions (18 and
19 in the list above).

| ID | Item | Status | Evidence |
|---|---|---|---|
| OPT-STORE | Store distribution (`DECISIONS.md` §6) | not started, **but the tree already carries a Steam SDK** | `godot-project/addons/godotsteam/` is vendored and `steam_api64.dll` + `libgodotsteam.*.dll` ship in `builds/windows/`. **No shell script, workspace script or `project.godot` line references Steam**, and `export_presets.cfg` excludes `addons/godotsteam/*` from the Android build. Vendored, not wired — and no scope document mentions the SDK is in the tree at all |
| OPT-WASM | A WASM target sharing `cartalith-engine` (`DECISIONS.md` §2) | not started | No wasm export preset in `export_presets.cfg`; no `wasm32` target configuration anywhere in `cartalith-native`. Confirmed absent. Note that `ROADMAP.md` Phase 0 says the skeleton "builds and runs on all three targets" while `export_presets.cfg` defines exactly two — Windows Desktop and Android — and the third target `DECISIONS.md` §2 names is this uncommitted WASM one. **`ROADMAP.md` contradicts itself within one page** |

**Group total: 2 — 2 not started.**

---

## Ledger totals

**283 milestone rows across 31 subsystem groups**, recounted 2026-09-24 from
the tables above (278 across 30 on 2026-09-23; the new group is *Travel
Library*, five rows). Method: one script run classifying each row by the
leading word of its Status cell, with `done*` counted inside `done` — the same
script reproduces the 2026-09-23 figures exactly when run over that version of
this file. Shares are rounded and do not sum to 100.

| Status | Count | Share |
|---|---:|---:|
| **done** (221, of which 7 are `done*`) | 221 | 78 % |
| **not started** | 18 | 6 % |
| **declined** (deliberate, with the reason in code or a ruling) | 18 | 6 % |
| **partial** | 16 | 6 % |
| **unverified** (not a code artefact) | 5 | 2 % |
| **blocked** (a named blocker) | 2 | 1 % |
| **other qualified statuses**, one each: MVP-OOS "4 of 5 shipped", CPU-6 "built elsewhere", GGR-RELIG "stale — corrected" | 3 | 1 % |
| **shelved** | 0 | — |

**What moved on 2026-09-23, beyond the day's builds.** A reconciliation pass
corrected rows that had stayed wrong after their code landed:
- GLI-D2, MEM-10, MEM-11, MEM-13, MEM-14 and MEM-15, from not started to done.
- LM-7, from blocked to done.
- MV-6, from partial to done.
- EXP-E1 to E3 to done, EXP-E4 to partial and EXP-E5 to not started. All five
  had read shelved.
- RD-1 was already done, and the group header now says so.
- Later the same evening, from the Android documents' cleanup: AND-9 from not
  started to done (`88bf297`), AND-10 from blocked to done (the adb rotation
  route was already in use), and AND-11 from not started to declined (Ruling 23).
  AND-12's note was corrected: the probe scenes stopped shipping in `686cd2a`.
- 2026-09-24, from the urban scope cleanup: UM-16 from ready to done (`cff1edc`)
  and UM-17 from partial to done (all 20 adapter functions accounted for).

The same pass added three rows the ledger lacked: GLI-E (thermal erosion on
the GPU), EC-10 (IN-13 trade) and the eight-row *LOD detail* group.

**What moved on 2026-09-24, from the alignment audit** (`ALIGNMENT_AUDIT.md`;
each row re-opened at its symbol):
- SL-0, not started → done (the L0 harness has existed since `611c5fa`).
- MV-4, not started → partial (`vault_saf.rs`, `cff1edc`).
- EC-8, blocked → partial (`civ_faction_economy`, `45b368d`).
- SP-3 and TL-6, done → partial (collapse flags unsaved; two of the three
  timeline filters inert).
- CPU-6's qualifier, "built, contrary to this document" → "built elsewhere"
  (the document's contrary sentences are gone).
- Added the five-row *Travel Library* group, all done.
- Group totals corrected where they did not match their own rows: Android
  (16 rows; the total listed not-started and blocked rows that do not exist)
  and Phase 5 (20 rows, not 19).

**Where the 18 not-started rows are** (recounted 2026-09-24).

| Subsystem | Not started | Note |
|---|---:|---|
| Religion diffusion | 6 | RD-2…RD-7; the foundation and milestone 1 are built |
| Sculpt live | 3 | L1, L2, L4; L0 is done (2026-09-24 correction) and L3 is declined by design |
| GUI replacement | 4 | Stages 3, 5, 6, 7 |
| Options kept open | 2 | Store distribution and WASM; neither is work until someone commits to it |
| Memory optimisation | 1 | MEM-12 (R6), ranked low on purpose |
| Export | 1 | EXP-E5, the dialog |
| Gap register | 1 | GGR-DS13 |

**Where the 2 blocked rows are.** LOD-D7 (owner question 6, an optional
milestone) and GGR-DS03 (an owner content decision). EC-8 left this list
2026-09-24: its memory blocker was worked around, and it is now partial. The paragraph that used to stand here named thirteen blocked rows,
most of them behind owner questions answered on 2026-09-06 or by Rulings AI,
AJ, AO and AP on 2026-09-23. It had been flagged as "stale past the point of
layered corrections" and is replaced rather than corrected again.

**Read the `done` figure carefully.** 78 % of rows done is not 78 % of the
project done — rows are not effort. Urban milestone 10 is one row and nine
reference functions; "R7 — `road_dijkstra`'s discarded `prev`" is also one row.
`OUTSTANDING_WORK.md` sizes every outstanding item; this table counts them.

**No test run backs this table**, with one dated exception: JP-QC2/QC3/QC4's
flip to `done` above was checked against a real `cargo check --workspace`,
`cargo test -p cartalith-civ` and `cargo test -p cartalith-godot` (both all
targets, zero failures) run in the same pass that recorded them, 2026-09-01.
Every other row's `done` means the named symbols exist in the working tree
and, where the milestone required reachability, a caller was opened.
`PARITY_TESTING.md`'s
golden suites are the actual correctness bar and there are **94 (recounted 2026-09-24; was 88)
`golden_parity_*.rs` files** in the workspace; whether they currently pass is
not recorded here. One test used to be known intermittent:
`generate_terrain_gpu_path_is_deterministic_and_valid` in
`cartalith-engine/src/lib.rs` failed roughly one run in three under
full-workspace parallel load, by about 1 ulp. It now compares the worst
per-element deviation against `GPU_DETERMINISM_TOL = 1e-6`, `DECISIONS.md` §7a's
principled-equivalence bar, instead of a whole-field `assert_eq!` (`803b725`,
2026-08-25). `OUTSTANDING_WORK.md` confirmed that on 2026-09-22 and closed the
row. Corrected 2026-09-23: this paragraph still called it an open owner
decision.

---

## Claims this file deliberately does not make

Recorded so nobody mistakes silence for absence, and so the next pass does not
try to "fix" these by asserting them.

- **That the owner has run any build.** MVP criteria 3 and 4 both require it and
  `DECISIONS.md` §5 says a session cannot certify it. The `.exe` and `.apk`
  exist; that is all this file can see.
- **Any device measurement.** Every PSS figure, frame time, thermal observation
  and logcat result in `ANDROID_BUILD_SCOPE.md`, `MEMORY_OPTIMIZATION_SCOPE.md`
  §7-§8 and `PERFORMANCE_BENCHMARKS.md` is a handset reading. The
  *instrumentation* is verified present; the numbers are not re-derivable here.
- **That any golden test passes today.** See above. A stale binary reports a
  healthy `N passed`, which is this project's own recorded hazard.
- **`GUI_GAP_REGISTER.md`'s open/closed split.** Its A/B/C/D class markers
  survive on 54 of 215 rows and recovering each dropped letter is "a judgment
  per row, not arithmetic" — declined by three consecutive audit passes.
  GGR-BULK above is spot-verified, not re-derived.
- **`UNWIRED_FUNCTIONS.md`'s 23 rows individually** (75 at the 2026-08-31 cut,
  before the 2026-09-01 re-cut closed 52 of them). They are one row in
  `OUTSTANDING_WORK.md` and are not enumerated here, because that document is
  the live backlog with a `file:line` per row and forking it would guarantee
  drift.
- **The state of the uncommitted working tree.** 126 files and 16 488
  insertions sit uncommitted (re-measured 2026-09-01, was 30 files / 6 871
  insertions on 2026-08-31). Every status above is against the **committed**
  tree unless
  the row says otherwise. When that work lands, the rows it touches need
  re-verification, not a copy of its commit message.

---

## Known defects in the project record

These cost a future session either re-derived work or a wrong plan, so they are
recorded here rather than left for the next audit to rediscover. Each is
verified in the working tree on 2026-08-31. The full set is
`OUTSTANDING_WORK.md` §6; this is the subset that would mislead someone reading
*this* file's neighbours.

### Documents that assert a thing does not exist on a day it does

This is the defect class that caused this rewrite. ~~Five instances survive~~
— **none survives, re-checked 2026-09-24** (alignment audit Part 1 C26): each
quoted phrase below was grepped for in its document at `HEAD` and none is there
any more (`ROADMAP.md` "no code written" and "Revisit when a concrete need";
`LANDMARK_GENERATION_SCOPE.md`'s §0/§3 sentences; `CPU_MULTITHREADING_SCOPE.md`
"never enumerated or used at all"; `GPU_LAYER_INTEGRATION_SCOPE.md` "stays off
by default and unexposed"). The table is kept as the record of what they said.

| Document | What it still says | What the code says |
|---|---|---|
| `ROADMAP.md`, "Options kept open" | Landmark generation was imported and cataloged 2026-08-30 with "**no code written**" | `landmark.rs` is 3 730 lines with `generate()`, ten `#[func]`s, a `landmark_store` field on `WorldGen`, 49 glyphs and a CIVIL ▸ Landmarks panel — all landed the same day |
| `LANDMARK_GENERATION_SCOPE.md` §0, §3 | "**No code was written for this pass.**" / "**Nothing below is started.**" | Seven of nine milestones are done and two are partial as of 2026-09-23 (see the landmark group) |
| `ROADMAP.md`, "Not a phase: LOD" | "Revisit when a concrete need appears rather than building it speculatively" | A tiled deep-zoom pyramid with a persistent chunk atlas is shipping and is on screen — `pyramid.rs`, `atlas.rs`, `lod_bridge.rs` (783 lines), the `viewport_host.gd` scheduler |
| `CPU_MULTITHREADING_SCOPE.md` | "`cartalith-gpu` currently only ever requests a single `PowerPreference::HighPerformance` adapter … the integrated GPU is never enumerated or used at all, for anything" | `multi.rs::enumerate_devices()` walks every adapter and `GpuDeviceSet` opens more than one |
| `GPU_LAYER_INTEGRATION_SCOPE.md` m6 | `use_gpu` "stays off by default and unexposed in the UI"; "generating a new map today still runs on CPU by construction" | `engine_bridge.gd` does `param_set("use_gpu", true)` in `_ready()`; `Preferences ▸ GPU acceleration` ships with `GPU_TOGGLE_TIP` |

### The stale UI hold — the third and fourth copies

The UI hold called by the owner on **2026-08-18 was lifted later the same day**.
`CLAUDE.md` carried the stale version until 2026-08-23, when `PARITY_AUDIT.md`
caught it. **Both later copies are gone** (corrected 2026-09-23: this section
said they were still in the tree). `GUI_SHELL_SCOPE.md`'s header ("UI work is
now on hold entirely") and `UNIFIED_TOOL_PLAN.md`'s "all UI work is on hold"
were both removed on 2026-09-01 in `fd9de7c`, verified by `git show` before and
after that commit. `STRANDED_TOOLS.md` never carried the phrase
(`git log -S "on hold"` on it is empty). Three scope-document milestone entries
were recorded as resting their "not wired" verdict on the hold (PHASE2 m20,
ECONOMY's `civ_culture_terrain_fit`, TERRAIN_APPEARANCE m1); all three are wrong
in code, and whether their own prose was corrected was not re-checked in this
pass.

### One shell string that used to lie to the user

1. ~~`civilization_workspace.gd`'s Culture category said there was no
   `get_cultures()` binding.~~ **Fixed, closed 2026-08-25** — verified against
   the working tree 2026-09-01: `_cultures()` at
   `civilization_workspace.gd:1681` calls `bridge.get_cultures()`
   (`cartalith-godot/src/lib.rs:11995`) when present, and the empty-list note
   at `:1704-1708` only fires when `bridge.has_method("get_cultures")` is
   false — a genuine stale-DLL fallback, not a false claim about a missing
   binding. This section previously cited `lib.rs:11711`, which is unrelated
   code inside a different function's error branch; that citation was never
   checked before being written, which is the exact failure this section
   exists to catch, reproduced inside itself.
2. ~~**`cartalith-godot/src/vault_bridge.rs`** still asserts that "`cartalith-io`'s
   save format … carries no civ data at all".~~ **Fixed — re-checked
   2026-09-24:** `vault_bridge.rs`'s module doc now quotes that sentence only
   under "What used to stand here, and why it was wrong". One copy of the
   same false premise survives in `cartalith-vault/src/links.rs`'s identity
   table ("civ is not saved"); see the Markdown Vault group. The original
   entry follows. That describes the retired
   `.zip` path, not the current project tree, which carries
   `entities/settlements.json` and `vault.json` side by side. **Half-fixed**:
   `shell/vault_store.gd` carries its own correction, dated 2026-09-01,
   verified against `project_bridge.rs`'s real `vault.json` round trip rather
   than taken on the file's own word; `vault_bridge.rs`'s copy of the same
   claim (its module doc, near its `#[func]` list) was reported but is not
   this file's to fix.

`UNWIRED_FUNCTIONS.md`'s 2026-08-31 cut found **nine** of this class and named
the tooling limit: `audit_wiring.py` finds unwired *bindings* and structurally
cannot see a stale *reason*, because every function involved is called and it is
the prose that lies.

### Documents that contradict themselves within one file

A reader who lands mid-document gets a false answer with **no signal to keep
reading**. Moving status out of scope documents only fixes this if the stale
half is deleted rather than left standing as history.

*Re-checked 2026-09-24 (alignment audit Part 1 C26): every entry below is now
resolved in its document at `HEAD`* — `PHASE2_SCOPE.md` no longer carries "Not
yet wired anywhere"; `ECONOMY_SCOPE.md` no longer files anything "not
started"; `MEMORY_OPTIMIZATION_SCOPE.md` §6's table now carries both columns
and its "Where the audit was wrong" section says so; and a grep of
`GPU_LAYER_INTEGRATION_SCOPE.md` finds no "four functions" or "dead" passage.
Kept as the record.

- **`PHASE2_SCOPE.md` m17** — "Not yet wired anywhere — no real caller exists",
  then "Resolved same day" four lines down. Both sentences stand.
- ~~**`ASSET_LIBRARY_SCOPE.md` §9** — enumerates eight gaps that §10 and §11 later
  close. Nothing marks §9 as a 2026-08-19 snapshot.~~ Fixed 2026-09-23: §9 is
  now labelled a dated snapshot and the closed gaps are written as the built
  design.
- **`MEMORY_OPTIMIZATION_SCOPE.md` §6** — the R2 table says
  `ChannelResult::slope` is read by "nobody, anywhere"; the later "Where the
  audit was wrong" section retracts it (`golden_parity_river.rs` asserts it
  three times) and the table was never edited.
- **`ECONOMY_SCOPE.md`** — files the food-surplus cluster under "not started"
  and later says "`_civFoodShed` **is** ported now". The earlier claim was never
  retracted. This document has no milestone numbering at all, which is why two
  of its four items could rot unnoticed.
- **`GPU_LAYER_INTEGRATION_SCOPE.md`** — milestone 6's prose describes the
  pipeline as calling four functions that milestone 8 records as dead.

### Counts that disagree with themselves

Small, but this is the document set that exists because countable claims drift.

*Re-checked 2026-09-24:* `ROADMAP.md` no longer says "all seven milestones" or
"all three targets"; `LOD_TILING_BASE_SCOPE.md` no longer says "24 unit
tests"; `ANDROID_BUILD_SCOPE.md` no longer says "~100" probe scenes;
`LARGE_ITEM_RULINGS.md` now says 1 530 lines and records that it read 1 486;
`CPU_MULTITHREADING_SCOPE.md` now disclaims its crate census. The probe count
has also moved: `ls godot-project/_*.gd | wc -l` gives **482**, not 84. The
entries below are kept as the record.

- `ROADMAP.md` Phase 4 says "all seven milestones"; the **eighth** (the
  sprite-sheet slicer) landed 2026-08-20.
- `ROADMAP.md` Phase 0 says the skeleton "builds and runs on all three targets";
  `export_presets.cfg` defines **two**, and the same file calls the third
  (WASM) uncommitted.
- ~~`URBAN_MORPHOLOGY_SCOPE.md` gives the `_um*` denominator as **20** in one
  place and **28** in another.~~ Resolved 2026-09-24 (`c4c930c`): 27 in total,
  20 in scope, everywhere.
- `UNWIRED_FUNCTIONS.md`'s 2026-08-31 headline **77** double-counted two rows
  its own "fixed during the audit" section closed; 75 were genuinely open at
  that cut, and its Large section heading read "(16)" where the intro said 18.
  **Historical**: the 2026-09-01 re-cut was written from scratch against the
  tree rather than patched, closed 52 of those 75 rows, and carries
  internally-consistent counts throughout (18 Large in both the heading and
  the running total; see the Tool system and GUI feature parity rows above for
  two of the closures) — 23 rows remain open.
- `LOD_TILING_BASE_SCOPE.md` records 24 unit tests and one dependent crate;
  there were **144** tests across eight modules and **five** external dependents
  on 2026-08-31, before `TiledField`/`QuadTree` and their 16 tests were retired
  on 2026-09-22 (`5c99cc9`).
- `CPU_MULTITHREADING_SCOPE.md`'s rayon census is four crates stale — see the
  re-measured figures in that group.
- `ANDROID_BUILD_SCOPE.md` says "~100" probe scenes; a 2026-08-25 audit counted
  76; **84** `_*.gd` files sit in `godot-project/`'s root today.
- `LARGE_ITEM_RULINGS.md` says the 3D research "stands complete at 1 486 lines";
  it is longer. Drift inside a single day.

### The reference freeze has drifted

*Corrected 2026-09-24 (alignment audit Part 2 D4): this section said
`reference/` holds only v2.10. It holds **both** frozen snapshots — `Cartalith
Gen1 v2.10.html` indexed by `FUNCTION_INDEX.md`, and `Cartalith Gen1
v2.11.html` indexed by `FUNCTION_INDEX_v2.11.md` (the v2.11 freeze,
`45b368d`) — checked by listing `reference/`.* `CLAUDE.md` no longer asserts
that the frozen snapshot is the source's latest. The drift that remains is
the source's, not the folder's: the source has moved past v2.11 onto the DCC
line (Ruling AP), so the re-freeze to an exact DCC version and its regenerated
index is **real outstanding work**, listed in `OUTSTANDING_WORK.md` §2.8.

### `CHANGELOG.md`'s last five days

Its last heading is `## 2026-08-26 (12)` and a grep for `2026-08-3` returns
**zero matches**, while `git log` shows eleven commits on 2026-08-30/31. Missing
entirely: landmark generation end to end, the 49 glyphs, `DESIGN_HANDOFF.md`,
the prototype import, the GUI replacement spec, the INFRA→CIVIL / RENDER→CARTO
ruling, stages 1-2, and the unwired re-cut. This is **why the file was retired**
rather than a reason to catch it up: git carries the commits, and this file
carries the state.

---

## Maintaining this file

- **Update it in the same change that changes its answer.** A milestone moving
  from `not started` to `done` is one row edit here plus the code; a status
  sentence added to a scope document instead is a regression.
- **Re-verify, never copy.** The row format exists to force this: a status with
  no symbol beside it is not a status. If you cannot open the symbol, the row
  says `unverified` and why.
- **Prefer a symbol to a line number.** Line numbers drift within a day; this
  project has the receipts.
- **Do not let this file become a changelog again.** The previous one reached
  8 122 lines by appending a narrative section per pass. *The last seven days*
  is a rolling window, not an archive — when a week ages out, its rows should
  already be reflected in the ledger and the prose should go. `git log` is the
  narrative record; `CHANGELOG.md` is the frozen one.
- **Counts in this file are dated and reproducible.** Where a count appears, the
  method is named so the next reader can re-run it rather than trust it.

---

*Rewritten 2026-08-31 against the working tree at `5543ef3` plus 30 uncommitted
files. Supersedes the 8 122-line narrative that preceded it, which is recoverable
from git history and is not worth recovering — its content is either in
`CHANGELOG.md`, in `git log`, or re-verified above.*
