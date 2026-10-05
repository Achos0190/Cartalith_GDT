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
2026-09-20 in the working copy: the source repo holds **164** `Cartalith Gen1
v*.html` (newest **v2.22**) plus a second line of **51** DCC files (newest
**v2.73**), and it **forked at v2.22** — every engine change from v2.25 on
exists only on the DCC line. This does not un-do a milestone; a phase verified
against v2.10 is still verified against v2.10. It does mean **no row above can
be read as "matches the source today"**. *(Noted 2026-09-24: `OUTSTANDING_WORK.md`
§2.8's re-freeze row carries a different measurement of the same folder, also
dated 2026-09-17 — 44 DCC-line files, newest v2.66 — against this paragraph's
49/v2.71. The source repository is outside this one, so neither figure was re-measured here; treat both as dated.)*

**`RC_ENGINE_CHANGES.md` is the full porting spec for v2.11 → v2.71** (function
and constant named per change, why each number is that number, which harness
verified it) — read the change there, not here; this file only tracks whether
each has been ported. *(Corrected 2026-09-28: this said "→ v2.73." Owner,
2026-09-28: v2.72 and v2.73 do not exist; v2.71 is the latest. The two entries
stay in `RC_ENGINE_CHANGES.md` §8 as history, marked not-real-source-versions;
what this port built from them stands as a port-original improvement — see the
v2.72/v2.73 paragraphs below.)* Nine of the interval's changes are deliberate upstream
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

**Which of the v2.11 → v2.71 interval is already ported was surveyed on
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

**"v2.73" is not a real source version.** Owner, 2026-09-28: v2.72 and v2.73 do
not exist; v2.71 is the latest. Recorded by `6aa1ff3` (2026-09-20) from an
unverified source. What this port built from the recorded spec below — the
village-green plaza `kind` — is a port improvement; its spec was recorded as RC
v2.72/v2.73, which does not exist. The footpath half described below was never
built (no source to port from); the idea can be re-raised as a port-original
feature if the owner wants it, rather than as a port of a nonexistent version.

**"v2.73" was recorded as the newest, and not simulation** — `hash_gen1.js` vs v2.72 ALL IDENTICAL, and the
generated LAYOUT hash is identical too. It adds a village green as a distinct plaza KIND and a
footpath class, and **both halves carry a finding a port should have before it writes either.**
(1) **A green is a plaza with a different purpose, not a smaller one.** The plaza builder already
cuts a widened bay off the principal street — the geometry of a market place and of a village green
alike — and what separates them is **market right, not size**: a chartered town's plaza is
commercial and a market CROSS stands in it as the legal marker of the right to trade, while a
village's green is common land and carries no cross because there is no right to mark. So the
change is a `kind` field and a branch on it — **the same three street calls, in the same order, at
the same widths**, which is what keeps blocks, parcels and buildings bit-identical — and the
threshold is the civic-building pass's own chartered-town line reused, not a second number for the
same distinction. (2) **The footpath had a free source that never runs, and only measuring it caught
that.** The alley-privatisation pass models a through-alley taken into the adjoining plots: the edge
must leave the STREET graph while the foot traffic survives, which is what a snicket is, so
recording the killed line costs four lines. It is also **unreachable on the profile the app
generates** — that pass opens `if(!bias) return;` and the default rules set `deadEndBias` to 0,
with only the medina family's 0.16 floor ever setting it: **0 paths across six populations on the
default profile.** Shipping that half alone would have been a feature that computes correctly and
shows nothing. The always-on source invents nothing either — it connects features the engine
already places (church, wells, the pond) to streets that already exist. **A path may not run
through a house, and the first cut did**: 11.4% of sampled path length fell inside a building
footprint at pop 12 000, fixed by a seven-sample rejection, 0.0% after. See
`RC_ENGINE_CHANGES.md` §8.2.

**"v2.72" is likewise not a real source version** (owner, 2026-09-28; see the
note above the v2.73 paragraph) — recorded as the version before it, and the one
to read before trusting any river harness. What this port built from the
recorded spec below — the raster river renderer's display-side area bar — is a
port improvement; its spec was recorded as RC v2.72, which does not exist. It
moves no generated value — `hash_gen1.js` vs v2.71 ALL IDENTICAL — and it changes the drawn map
materially at large extents, which is why the spec files it in §8.1 beside v2.58 rather than with
the shell rows. Two independent defects, one screenshot of a 40 000 km world, and **both landed on
the wrong one of two renderers.** (1) **The straight lines across the map are rivers.** The receiver
tree wraps in X in world mode, so consecutive points of one stem sit at opposite edges and the tile
renderer stamps a band along every segment — one wrapped step paints a river clean across the map.
The HTML has had a splitter for exactly this since v1.29 and had applied it at **three of four
sites**; the fourth is the geometry the RASTER path draws from, **and the raster path is the one
that draws at the default**, so the renderer that splits is the one nobody was looking at. The
sharpest part: v2.58 unwrapped the same stem's LENGTH in that very loop while still handing the
drawer the wrapped points — **fixing a measurement of a quantity is not fixing the thing the
quantity describes**, which is why this read as fixed for fourteen versions. (2) **A detection ease
is not a display threshold** — the third consumer of one constant answering three questions, now
written up as `RC_ENGINE_CHANGES.md` §7.13. **Why every existing harness was blind, which a port's
test plan should copy**: region mode cannot wrap and its ease is 1, so both fixes are no-ops there
*by construction* — every river probe runs region mode and the hash battery never sets world mode.
A field hash would not have caught it either. See `RC_ENGINE_CHANGES.md` §8.1 and §7.13.

**v2.71 is the version before those two, and is likewise not simulation** — `hash_gen1.js` vs v2.70 ALL IDENTICAL — but its first
half is a rule a port inherits whether or not it copies the feature. The owner asked whether a new
guidance layer was needed to keep a settlement's drawing off the water; it was not. The adapter has
always built a 22 m mask of the real sea, lakes and river band, and the engine's own `isWater`
predicate reads it — it simply never reached the RENDERER, because the model record handed to the
drawing code is deliberately function-free and the mask was not among the fields copied. **The trap
is where a port will hit it too**: on the real-map-water path the site builder sets its water
polygon EMPTY on purpose (the map already paints the sea beneath the town), so a clip keyed on that
polygon passes every synthetic fixture and does nothing live — measured, 7 of 39 real towns carry an
empty one. Carry the mask instead, and where a town carries both, the mask must win. The measurement
method is the other reusable part: two cheaper metrics both lied (overdraw as a share of a 211 000-px
sea reads 0.14% and looks like antialiasing; a palette match misses an antialiased street edge
entirely), and the honest test renders the town, renders it again with the settlement layer stripped,
and diffs inside the water. See `RC_ENGINE_CHANGES.md` §8.2.

**v2.70 is the version before that, and is also not simulation** — a flat limited-palette map style, opt-in, `hash_gen1.js`
vs v2.69 ALL IDENTICAL. It is worth a line for its SHAPE: the HTML has exactly one land-colour
function and one water-colour function, each called by the main per-pixel loop, the LOD tile
renderer and the flat bake, so a whole new map style costs one flag and one step in each chain and
every surface picks it up — including the export, which is how an exported image matches the screen
with no second code path. A port whose colour logic is duplicated per renderer pays for each style
N times. See `RC_ENGINE_CHANGES.md` §8.2.

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

### 2026-10-04

- **Rivers painted into the map (`RIVERS_IN_MAP_SCOPE.md` RIM-1, RIM-3, RIM-6, default render path) — built, verified
  by an independent adversarial pass (PASS).** `river_field.rs` builds a per-pixel river distance field from the
  RV-2 centrelines; `map_shore.gdshader` paints it with the stroke's own coverage law, composed under the shore's water
  so a river meets a lake or the sea with no hole; `valley_shade_field` takes `recut: bool` and skips the river recut
  when `rivers_as_water` is on (no groove) and a floodplain tint sits beside each river; `map_overlay.gd` no longer
  draws the stroke when the river is painted (hit-testing untouched). Painted vs stroke coverage on seed 246371:
  ratio 1.02-1.05, IoU 0.90-0.96; GPU 0.27 ms painted vs 0.34 ms stroke at 2048x1311; the field costs 86 MB and 96 ms
  there. Applies only when `WorldGen::rivers_painted()` (grid within `river_field::MAX_TEXELS`; 8192² keeps the stroke).
  Deep-zoom tiles (z > 2.5) and export still draw their own strokes (RIM-7 not built). **Not built:** the rest of RIM-7 (RIM-2, RIM-4 and RIM-5 are built, and RIM-4's fans now draw in tiles and export, below). No generation golden moved (`cargo test --workspace`: 4 657 passed, 0 failed, 54 ignored).
- **Paint "Original" overlay (Ruling BR) — built on desktop for Biome and Terrain, GDScript only
  (`shell/paint_original.gd`, `ViewportHost.set_paint_original`); verified PARTIAL.** `_paintorig_probe.tscn` passes on
  desktop (display changes with the slider, world data unchanged). The "original" is the engine's unpainted
  `bclass`/`cterrain` classification re-derived from the *current* height, so a committed Sculpt edit moves it; a true
  pre-sculpt snapshot needs Rust. Splat has no generated raster. **The phone Paint bar is unreachable** (arming Paint
  selects WORLD, whose GENERATE sheet hides the tool-options row) — pre-existing; fixed the same day, see the next entry.
- **Ruling BK annotation pass: `cartalith-civ/src` complete** (comments only, 10 files; census `items_undoc` 940 → 277,
  `tests_unprotected` 876 → 193, `consts_undoc` 159 → 47, all remaining in `tests/` and `examples/`;
  `cargo test -p cartalith-civ --lib` 828/0/1). Verified comment-only by an independent checker.
- **Ruling BK annotation pass: `cartalith-civ/tests` and `examples` done, three generated files left** (comments
  only, 31 files, +1560/-10 comment lines; comment-strip-and-compare against HEAD SAME on all 31; census `tests/`
  `items_undoc` 261 -> 43 of 379, `tests_unprotected` 193 -> 30 of 200, `consts_undoc` 47 -> 1 of 52; `examples/`
  16 -> 0). The remaining 43/30/1 sit in `golden_parity_centrality_feedback.rs`, `golden_parity_urban_adapter.rs`
  and `golden_parity_want_counts.rs`, which are generated: a hand comment is lost on regeneration, so the fix
  belongs in their generators. Protects lines were derived by reading, with no mutation runs. Independent verifier
  found a false clamp claim and four citation slips, all fixed; `cargo test -p cartalith-civ` all targets 0 failed.

- **Four failing phone probes triaged - all pass again, verified PASS (probe files only, no product defect).** `_nwsize`, `_nwcard`, `_sheetgrab`, `_vfy_gesture`, plus `_detent` (same cause as `_sheetgrab`), run windowed at 1080x2340 from scratch project copies, 5 of 5 runs each at HEAD and after (HEAD: 1, 2, 3, 1, 3 failures; after: 0). Seven failures, classified at the symbol: four stale preconditions (b) and three intentional changes that made an assertion obsolete (c). (b) `_nwsize`: the first `GENERATE` label is the sheet header title since `773262e5`, not the nav cell, so the tap never raised the sheet; the probe now selects the nav cell by ancestry. (b) `_sheetgrab`/`_detent`: while no world exists the embedded `PhoneProjectPicker` is an exclusive full-screen window and swallowed every pushed event; the probes now hide it. (b) `_vfy_gesture` gate 1: 69 of the 76 MenuButtons are Godot ColorPicker internals inside hidden PopupPanels (`021d955f`); the probe now filters on that ancestor and asserts the 7 menu-bar buttons. (c) `_nwcard`: Map width, Width (km) and Archetype were lifted onto the phone card by `77f91942` (owner's 2026-09-07 request), so the probe now asserts they are on it. (c) `_sheetgrab`: the grab row is the whole 66 dp header block since `773262e5` (measured 65.61 dp), so the probe asserts >= 44 dp and within 1 dp of the peek. Mutations each turn the right probe red (grab filter IGNORE, peek lift removed, MenuButton RELEASE, size section hidden, picker-hide removed, archetype section hidden). `cargo test --workspace`: 4685 passed, 0 failed. **Not verified:** other phone densities for `_vfy_gesture`, any real-device pass, and the roughly 100 other probes that mention the same widgets (a sibling probe may share the picker-not-hidden precondition).
- **SpinBox touch arbitration, and the two stale probes - built, verified PASS (GDScript only).** `DccWidgets.PgSpin` (an inner class, no `class_name`) and `touch_spinbox()`, attached from `DccShell._touch_arbitrate`'s SpinBox branch, so phone docks, phone windows and tablet are covered (29 SpinBoxes measured after PLAN is entered; the earlier "18" was stale). A vertical swipe on a SpinBox's arrow strip scrolls the container and leaves the value alone; a deliberate tap on the upper or lower half steps by one step, applied on release (no auto-repeat on a held arrow on touch); typing is unchanged; desktop SpinBoxes stay stock (0 scripts, mouse leg passes). Slop is 8 dp x unit with no default on purpose. The six failures: `_rangeswipe` inset 3/8/18 were a real defect (the strip was unarbitrated: a swipe moved a seed 243025 to 243034); `_rangeswipe`'s slider leg was a stale precondition (every left-dock category starts collapsed, the probe now opens one through its real header); `_gestclass`' two seed-tap failures were a stale precondition (an earlier leg had scrolled the card away, the probe now scrolls the field into view and checks the hit-test). `_gestclass` fail 2 -> 0, `_rangeswipe` fail 4 -> 0 (5 of 5 runs each, verifier-reproduced), desktop `_rangeswipe` 0; the new swipe/tap/typing legs fail 22 checks on HEAD code and with the hook disabled. **Not verified:** real-device touch, the tablet shell by a probe of its own. Four other probes failed identically before and after; see the next entry.
- **Phone Paint reachable by thumb — built, verified PASS (GDScript only).** The GENERATE sheet's mode segment is now PIPELINE / SCULPT / PAINT; the PAINT column carries Target field, Size, the Ruling BR Original slider, Erase, Land only, Class, ARM PAINT and Discard/Commit. `_phonepaint_probe.tscn` drives it with real taps and drags at `--vp 1080x2340 --force-touch`. Second pass, same day, verified PARTIAL (findings fixed): the phone Paint column now carries Hardness and Softness (0..1, writing the shared `_paint_brush`; the desktop tool bar has none for Paint by the 2026-08-31 ruling, only the WORLD dock does), the map chip reads `PAINT · DRAFT` while Paint is armed (`DccShell._refresh_viewport_context`, refreshed on `tool_armed`), and the Original slider is inert with a reason once another tool is armed (`PaintOriginal.set_percent` still the only writer). `_phonepaint_probe` 179 checks, 7 of 8 windowed runs green (the miss is the known slider-fill flake); five mutations each turn it red. Not covered: a device pass; the chip still reads `SCULPT · DRAFT` when nothing sculpt or paint is armed. `_gestclass` (fail 2) and `_rangeswipe` (fail 4) fail identically at HEAD and with these edits: diagnosed as SpinBox touch arbitration on Godot 4.7.1, not fixed (own row in `OUTSTANDING_WORK.md`).
- **RIM-7 remainder: the painted river on deep-zoom tiles and in export - built, verified PARTIAL (findings fixed).** For a look with `smooth_shores` and `rivers_as_water`, tiles above zoom 2.2 and every export (whole, banded, overlay snapshot) now paint the river by the screen's own law instead of stroking it. One gate, `river_field::painted`, decides all three paths (`river_delta::fans_drawn` calls it too); `river_field::eval_window` is the shared per-texel decision (the screen texture's output is bit-identical to before), `pixel_paint` carries the shader's law (coverage x order-1 RIM-5 alpha x frame weight, floodplain weight, bank), the colour comes from the cached `WorldGen::river_colour_field`, and the floodplain tint and bank line are drawn after the finishing pass (`render::river_post_rgb`, `RiverPost::apply`). The stored-pyramid fingerprint gets `PAINTED_RIVERS_TAG`. Looks that stroke, the Rivers layer off and grids over the field's budget (8192) still stroke, exactly as before. Measured: screen path and every OFF/stroke frame pixel-identical to HEAD (seeds 246371 and 483920, verifier-reproduced); z2.1 to z2.3 water area ratio 0.97/1.01/0.88 (HEAD 0.89/1.01/0.83); export at 4 px/cell matches the tile view within 0.25 cell; painted tile build 8-9 ms against 24-27 ms for the stroke; pan frame time unchanged (vsync-bound 16.7 ms); 16K (5 bands) and 32K (33 bands) banded exports show no seam spikes. `cargo test --workspace`: 4697 passed, 0 failed (+12); the verifier's own mutations killed every new gate and constant. Closed with it: the `u64::MAX` fingerprint separator now has a test. **Not verified / open:** the export's own-style behaviour has no unit test (five mutants survive; a probe-only check shows it correct); 8192 end to end (the probe ran 45 minutes without a frame); a mid-session style change and the tile cache after a river-switch toggle were read statically only; real preset click-through. **Visible change to tell the owner:** exports of painting looks now carry the floodplain tint and bank outline, which they never drew before.
- **RIM-7, deltas part (fans on the deep-zoom tiles and in export) - built, verified PASS.** RIM-4's fans now draw above the tile switch (camera zoom 2.2, `LOD_AUTO_ZOOM`, not 2.5) and in every export, before the network, by the same stroke law (`river_stroke::rasterize_with`; `SnapshotInputs::river_fans`; `WorldGen::export_river_fans`), behind one gate `river_delta::fans_drawn` shared with the screen path. The tile cache key carries the switch (`dl`). Owner world, 4 mouths: ON-vs-OFF from 0 px (HEAD) to 4 195-7 439 px per frame at z2.3-8, exports 1 207-5 218 px; OFF pixel-identical to HEAD (24/24 frames, 8/8 exports); frame cost and tile build unchanged. `cargo test --workspace`: 4 685 passed, 0 failed, 54 ignored. **Still open:** the fans read as thin near-straight hairlines, fainter than the trunk, so they are present but not a convincing delta (a look decision that moves RIM-4's pinned constants: branches that re-split, curved or wider branches, a delta-plain tint, a narrowing trunk); the order-versus-discharge gate (owner); two unit-test gaps (`export_fans_ignore_style`, the fingerprint separator word); only seed 246371 probed, no 8192 window run, no banded 16K/32K export run, no Android. The rest of RIM-7 (painted rivers themselves in tiles and export) is unchanged.
- **RIM-4 (river deltas at large mouths) - built for the painted screen path, verified PARTIAL, findings fixed or disclosed.** `river_delta.rs`: a mouth whose own Strahler order is >= 3 AND whose discharge is >= 2.0e-4 of the grid's cells (the owner world's own mouth p90, a labelled judgement; measured on seeds 246371 2048x1311 / 483920 / 912345: 2203 / 171 / 69 mouths, 6.3% / 5.8% / 13.0% of order >= 3) gets a fan of 2 (order 3) or 4 (order >= 4) straight distributaries from an apex upstream of the mouth, dropped if they meet no water. Owner world: 62 eligible, 57 fans, 131 branches, 0.7 ms to derive. Off switch `WorldGen::set_river_deltas(false)`, default ON; OFF is pixel-identical to HEAD (12/12 frames, 0 px). Frame cost unchanged (vsync-bound 16.67 ms ON = OFF = HEAD). Part C: two rivers through Lake Octvaliana at z2-4 show no seam or bead. **Caveats:** the fan is faint and straight at fit to z2.1 (not a convincing delta yet); it is absent at z >= 2.5 and in export (RIM-7); the order >= 3 gate skips 155 mouths that clear the discharge bar, 3 of the top 5 by discharge being order 1 (owner call: gate on order, discharge, or both); 12 of 46 mutants survive (the fan's colour-field fill is drawn twice so dropping one pass is covered by the other, fan-vs-network order, the fan pad, apex-wet, a few boundary literals). The probe's 'PROBE-FAIL: 4 free ends stop short of adjacent water' also fails with the HEAD DLL (builder's reading; not re-run by the verifier). `cargo test --workspace`: 4 678 passed, 0 failed, 54 ignored.
- **RIM-2 (optional bank outline) - built for the painted screen path, verified PARTIAL, findings fixed.** A 1 px antialiased line in the preset's own ungraded river ink on both banks, clamped inside the field's valid band, scaled by the RIM-5 headwater fade, gated per preset by `river_bank` (default 0 = off; on for Antique 0.40, Ink 0.60, Atlas 0.25, Ink wash 0.35, Woodcut 0.70, Vintage atlas 0.30). Constants `BANK_WIDTH_PX` 1.0 and `BANK_BAND_MARGIN_CELLS` 1.0 are labelled judgement. Shader and screen path only: tiles and export are RIM-7's, no generation golden moved (`cargo test --workspace` 4 667/0/54, +7 `river_field` tests). **Off is not strictly byte-identical to HEAD:** 12 of 12 bank-preset OFF frames and 15 of 20 shipped-off frames are exact; in the other 5, one pixel is off by 1 LSB (cause unproven, probably driver rounding after the shader recompile), and the comments now say so. Frame cost unmeasurable (Ink, 16.7 ms vsync-bound, GPU +0.00-0.008 ms). Mutation survivors disclosed: the inward-clamp `0.5`, the shader `be` offset and the `bank` key in `river_paint_params` (probe-covered only). Not checked: confluence crops, other GPUs, other world sizes. **Not built:** RIM-4, RIM-7.

- **RIM-5 (headwater fade by zoom) — built for the painted screen path, verified PARTIAL.** Order-1 streams fade by a smoothstep on screen pixels per cell (gone at 0.2, full at 0.8; `river_stroke.rs::o1_distance_fade`, mirrored in `river_field.rs::coverage` and `map_shore.gdshader`). Tiles, export and the stroke fallback are untouched (z4 and z16 tiles byte-identical). The thresholds are labelled judgement: the opening view is a cover fit near 0.65 px per cell, where the fade barely acts, so it shows once zoomed out; other world and window sizes were not measured. The flat tributary stub at z16 pre-exists (checked against 11a3dab6). `cargo test --workspace`: 4 660 passed, 0 failed, 54 ignored.

### 2026-09-28

- **Developer notes in the UI still naming internal symbols — closed, pending
  independent verification.** The remaining spots the row named — the
  export-summary "how" fields and the re-run-validators tooltip/label in
  `shell/data_manager_window.gd` — now read in plain words (e.g. "Settlements
  placed in this world" rather than `EngineBridge.settlements()`), with the
  symbol (`EngineBridge.*`, `AssetLibrarySession::validate()`,
  `validate_animal`/`validate_vehicle`/`validate_vessel`/`validate_party_preset`)
  moved to a `##` comment above the statement, never inside a string. Grepped
  to confirm no symbol remains in a string literal; the file parse-checks
  clean under Godot 4.7.1 `--headless --check-only`.

- **"Roads do not meet their settlements" — built (Ruling BT and its extend-back follow-up), verified by the main loop 2026-09-28 (Quinmarcauriana x16 before/after opened; the lane's counts civ 1019/0/11 and godot 1659/0/34; three goldens moved with owner-approved disclosures).**
  Two changes, both owner rulings dated 2026-09-28. **(1) `RW_CAP`:**
  `civ_routing_grid`'s cap (`cartalith-civ/src/lib.rs`) moved from
  `gw.min(384)` to `gw.min(RW_CAP)`, `RW_CAP = 1024` (measured, not
  guessed — see below). Worlds at or below the old 384-cell cap route
  byte-identically (`sc == 1.0` either way). **(2) Extend-back:** a new pass
  in `civ_consolidate_and_smooth_ways` splices a stranded run back along its
  OWN edge's own routed `path` toward its own settlement (never the other
  one, never touching another way) when its visible end lands more than
  ~1.5 cells short — closing the corridor-consolidation claim-order gap the
  re-diagnosis below found, which turned out NOT to be downsampling-specific
  (`golden_parity_road_consolidation.rs`'s case1 fixture, 16×12, `sc == 1.0`,
  already had a way 4.3 cells short before this pass existed).
  Measured via `_roadsettle_probe` on the owner's exact world (seed 246371,
  2048×1311, metropolis on, villages on, river_density 1.55, geology_model
  on): 384 cap (before) — 14.2s generation, 20/236 ways >2 cells from their
  own settlement. `RW_CAP=1024` alone — 18.1s, 15/244 short (full resolution
  2048 tried too: 32.9s, still 15/246 short, no further gain, so 1024 was
  kept). `RW_CAP=1024` + extend-back — **18.7s, 0/244 short**.
  Two goldens moved by the `RW_CAP`/extend-back change and were re-baselined
  with a dated disclosure comment at each: `downsampled_routing_grid_batch_
  of_six` (`golden_parity_village_connect.rs`, world "downsampled" is
  480×200 — above the old cap, a deliberate divergence from the frozen JS
  reference per `DECISIONS.md` §7p, not a bug) and `road_consolidation_
  case_1_k5_corridor_sharing` (`golden_parity_road_consolidation.rs`, the
  16×12 fixture above). A third test, `cartalith-godot`'s
  `recovery_keeps_every_road_on_the_settlements_it_joined`, asserted
  "rebuild draws strictly more than filtering" as a proxy for "filtering
  alone leaves a stranded-corridor gap a rebuild fixes" — extend-back closes
  that gap on both sides, so the lengths came out exactly equal and the
  proxy assertion broke correctly; rewritten (owner-authorized) to check
  what its name actually says (every surviving way's endpoints reach their
  own settlement in both networks, `assert_endpoints_reach_settlements`) and
  `rebuilt >= filtered` rather than `>`.
  A new test protects extend-back directly: `extend_back_reaches_a_
  settlement_whose_own_cells_a_busier_edge_already_claimed`
  (`golden_parity_road_consolidation.rs`) — a synthetic K3 fixture where a
  busier edge's claim eats 3 cells into a quieter edge's own near-settlement
  path, and the quieter edge's way must still reach its settlement exactly.
  `cargo test -p cartalith-civ` (both `--release` and the default profile):
  **1019 passed, 0 failed, 11 ignored** (1018 prior baseline + this new
  test). `cargo test -p cartalith-godot`: **1659 passed, 0 failed, 34
  ignored**. See `OUTSTANDING_WORK.md`'s "The count, honestly" for the
  closed row.

- **"Roads do not meet their settlements" — re-diagnosed, pending independent verification.**
  The earlier half-cell-convention hypothesis in this row was wrong (a prior
  probe measured 0.00 px because "nearest endpoint to any settlement" can't
  see a settlement whose OTHER ways miss it). A per-way check (each way's
  declared endpoints against its own `a_idx`/`b_idx` settlement) on the
  owner's exact world (seed 246371, 2048×1311, metropolis on, river_density
  1.55, geology_model on) found 20 of 236 ways with one endpoint 59-585
  cells from its own settlement. Root cause traced to the symbol:
  `civ_consolidate_and_smooth_ways`'s corridor-consolidation claim logic
  (`cartalith-civ/src/lib.rs` ~8479-8644), matching the reference's own
  `_civHierarchicalNetwork` (`Cartalith Gen1 v2.11.html` ~22175-22227)
  **exactly**, including the v1.02 second-pass endpoint snap
  (`_snapT2`/`snap_t2`) already correctly ported. The reference's own
  comment documents this snap as deliberately bounded so it "can never
  reach toward a NEIGHBOURING place" — accepting that a deep shared
  corridor still visibly misses. The reference caps its routing grid at
  the same `RW=min(GW,384)` this port uses, so this is inherited JS
  behaviour made newly visible at world sizes (2048×1311) the reference's
  browser canvas never exercised, not a port-introduced regression. **No
  code was changed**: no existing golden exercises `gw>384`
  (`golden_parity_road_consolidation.rs`/`golden_parity_hierarchical_
  network.rs` use gw=14/16; `iterative_network.rs`'s gw=512 case is
  `#[ignore]`), so a threshold change would move none of them, but it would
  still be a deliberate deviation from the reference's own documented
  algorithm — `DECISIONS.md`'s "do not deviate silently" rule requires an
  owner ruling first, not a silent patch. `cargo test -p cartalith-civ`
  unchanged at 1013 passed / 0 failed / 11 ignored. See
  `OUTSTANDING_WORK.md`'s row for the full account and next step.

- **Developer code names in the Data Manager's gap-register notes — narrowed, verified by the main loop 2026-09-28 (edited string literals grepped: no ## and no identifiers inside them).**
  `data_manager_window.gd`'s `val_check` reason and its five `CHECKS_*_NOTE`
  consts, plus `world_workspace.gd`'s micro-erode LOD note, no longer name
  `EngineBridge.last_open_warnings`, `app.gd`, `viewport_host.gd::move_view_to`,
  `AssetLibrarySession::validate()`, `travel_library.rs`, `as_pack_info().name`,
  `poly_self_intersects`, `cartalith-urban`, `cartalith-spatial/src/geo.rs`,
  `golden_parity_geo.rs`, `ensure_ccw`, `blocks.rs`, `inset_poly` or
  `pyramid_tile` inside the user-visible string — each symbol moved to a `##`
  comment on the line above. Both files parse clean under `--headless
  --check-only`. Still open, same file, a different section: the export-summary
  "how" fields (~lines 1666-1697) and a validators tooltip/label (~2456/2511)
  still name `EngineBridge.*` and `AssetLibrarySession::validate()`
  (`OUTSTANDING_WORK.md` §2, "Developer notes shown in the UI still name
  internal symbols").

- **Ruling BS (journey planner: foraging plus suggested resupply stops; the
  carry-capacity block retired) built — verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree, civ and godot; desktop suggested-stops screenshot opened).**
  `cartalith-civ`: `jp_calc_land_ex` already carried only the net-of-foraging
  gap (confirmed at the symbol); the pre-loop block now reads cargo alone
  (Haste keeps the gross test), a full supply interval that does not fit is
  shortened to what does (`carry_days`/`carry_capped` on `JpLandCalc`,
  "Binding: capacity"), and the surviving post-loop block names the waterless
  stretch or the cargo. New `jp_resupply_walk` walks the route and suggests
  settlement stops (1 d, day-wage cost) or returns a `JpSupplyBlock` naming the
  uncarriable stretch; `JpPlan::accepted_resupply`, `JpJourneyPlan::
  {resupply_suggestions, supply_block, resupply_stop_days}`; verdict, cost and
  confidence read them. Water and pasture stops are **not** suggested: the model
  already refills water at every drinking-water point and nets grazing off
  fodder, so neither lengthens the range (stated in code and UI). Saved with the
  journey: `Journey::resupply_stops`, `entities/journeys.json` member
  `resupply_stops` (default-empty, skipped when empty — pre-ruling files re-save
  byte-identical, tested). Bridge: `jp_compute` `resupply_accepted`,
  `journey_set_resupply`, `journey_get.resupply_stops`. UI: "Resupply stops"
  results group (accept/decline/remove), route-map squares, timeline segment,
  time row. Six goldens moved by design (dated disclosures at each). `cargo test
  -p cartalith-civ -p cartalith-godot`: 2 671 → 2 677 passed, 0 failed, 45
  ignored; 14/14 mutants killed. Probe `godot-project/_jpresupply_probe.gd`
  (windowed, desktop and `--vp 1080x2340 --force-touch`): a party the old
  engine refused (gross 2.38x) gets 2 suggested stops, accepting both takes it
  from severe to strained at 84 calendar days.

- **Ruling BK annotation-audit census done — verified by the main loop
  2026-09-28 (test and "protects" totals re-counted by grep).** `ANNOTATION_AUDIT.md` (repository root, new) measures every
  crate under `cartalith-native/crates/` plus `godot-project/shell/**/*.gd`
  with a reproducible line-scan script (reproduced in full in the doc): 43%
  of Rust `fn`/`struct`/`enum`/`trait`/`mod` items across 312 860 LOC have no
  preceding doc comment; 4 154 of 4 158 `#[test]` fns have no "protects"
  comment nearby; 662 of 1 277 numeric consts have no adjacent provenance
  comment (documented there as a systematic undercount for multi-line array
  constants); 1 167 of 3 129 GDScript `func`s (37%, 101 811 LOC) have no
  preceding comment. This is a census only — no code or comments were
  changed. `OUTSTANDING_WORK.md` §2.8's Ruling BK row now points at the
  document and proposes a ranked, per-crate batching order (highest-risk
  first: `cartalith-jsmath`, `cartalith-rng`+`cartalith-noise`,
  `cartalith-engine`, ... GDScript shell last). **Not yet re-run by a second
  pass; the method's own blind spots are stated in the document rather than
  hidden.**

- **Ruling BK annotation pass, batch 1 (`cartalith-jsmath`) done — verified by the main loop 2026-09-28 (diff re-read as comment-only; 17/17 re-run).** Comments only, zero behaviour change: every
  `fn`/`struct`/`mod` in `cartalith-jsmath/src/lib.rs` and `libm.rs` now has a
  doc comment (several already did; the census's heuristic missed some because
  a plain `//` comment sat between the doc block and the item, breaking
  rustdoc-style adjacency — fixed by folding those into the doc block rather
  than duplicating it), every `#[test]` carries a `// Protects: ...` line, and
  every numeric FDLIBM constant (`js_exp`, `kernel_sin`, `kernel_cos`,
  `rem_pio2`, `js_log`, `js_log10`, `js_acos`, the `atan` module) carries a
  provenance comment naming its fdlibm source file and routine. The crate's
  own census re-run: items_undoc 22→0, tests_unprotected 17→0, consts_undoc
  68→0 (LOC 2065→2170, all added lines comments). `cargo test -p
  cartalith-jsmath`: 17 passed / 0 failed, unchanged from before the pass. A
  scratch checker asserting every changed line is a comment, blank, or a
  trailing-comment addition to unchanged code passed. No comment was found
  wrong against its code in this batch. Batch 2 (`cartalith-rng` +
  `cartalith-noise`, per `ANNOTATION_AUDIT.md`'s ordering) is next.

- **Ruling BK annotation pass, batch 2 (`cartalith-rng` + `cartalith-noise`)
  done — verified by the main loop 2026-09-28 (diff re-read as comment-only; 20/20 re-run).** Comments only, zero behaviour
  change: every `fn`/`struct`/`mod` across both crates' `src/lib.rs` and
  `tests/*.rs` now has a doc comment (`Mulberry32`, its test module, the
  golden `Case`/`HashCase`/`NoiseCase` fixtures, and the small handful of test
  fns the census still flagged after batch 1's `Protects:`-in-body pattern
  wasn't itself picked up as documentation), and every `#[test]` carries a
  `// Protects: ...` line. Both crates' constants were already fully
  provenanced (0 unprovenanced in `ANNOTATION_AUDIT.md`'s original count), so
  no constant comments were added. Census re-run: `cartalith-rng`
  items_undoc 6→0, tests_unprotected 4→0 (LOC 225→255); `cartalith-noise`
  items_undoc 17→0, tests_unprotected 16→0 (LOC 1209→1291); all added lines
  comments (checked with a script asserting every changed line is a comment,
  blank, or a trailing-comment addition to unchanged code — passed for both
  crates). `cargo test -p cartalith-rng -p cartalith-noise`: 20 passed / 0
  failed (13+2+1 noise, 3+1 rng), identical pass counts before and after.
  No comment was found wrong against its code in this batch. Batch 3
  (`cartalith-engine`, per `ANNOTATION_AUDIT.md`'s ordering) is next.

- **Ruling BK annotation pass, batch 3 (`cartalith-engine`, first half by
  module) — verified by the main loop 2026-09-28 (diff re-read as comment-only; engine 214/0/8 matches HEAD).** Comments only, zero
  behaviour change. Scope: `src/slippy_export.rs`, `src/progress.rs`,
  `src/center.rs`, `src/geo_clock.rs`, `src/channel_atlas.rs`,
  `src/sculpt_commit.rs`, `src/import.rs` (2 432 LOC), plus their two
  associated golden-parity test files `tests/geology_gf7.rs` and
  `tests/golden_parity_sculpt_water.rs` (635 LOC) — chosen as the
  highest-risk (golden-parity-tied) modules of the crate's first ~2 800 LOC
  half, since `geo_clock.rs`/`sculpt_commit.rs`/`geojson.rs`-adjacent code
  ties directly to `tests/geology_gf7.rs`, `golden_parity_sculpt_water.rs`
  and `golden_parity_geojson.rs`; `geojson.rs` itself and its golden test
  were left for batch 4 to keep this batch near budget. Every `fn`/
  `struct`/`mod` in these nine files now has a doc comment, every `#[test]`
  carries a `// Protects: ...` line (placed as the first line inside the
  function body, per batch 1/2's own pattern — a line between `#[test]` and
  `fn` is outside the census script's scan window and does not count), and
  every numeric constant without a doc comment already covering it got a
  provenance line. `geo_clock.rs`'s constants were already provenanced
  (citing `GEOLOGY_FIRST_SCOPE.md` §4.12, pre-existing, not re-verified
  against the scope document by this pass) and needed no new comment; the
  test-fixture constants added this pass (`geology_gf7.rs`'s `GW`/`GH`,
  `golden_parity_sculpt_water.rs`'s `GW`/`GH`/`SEA`/`SEED`/`ZERO6`/`ZEROM`)
  are cross-checked against sibling fixture files in the same crate instead
  of a scope citation, stated as such rather than invented. Census
  re-run, scoped to these nine files: items_undoc 66→0, tests_unprotected
  57→0, consts_undoc 21→0 (LOC 2872→3229, all added lines comments).
  Crate-wide re-run: `cartalith-engine` items_undoc 233→167,
  tests_unprotected 221→164, consts_undoc 51→30 (LOC 14344→14701 non-blank).
  `cargo test -p cartalith-engine`: 210 passed / 0 failed / 7 ignored
  (18 result lines), identical before and after. A script asserting every
  changed line across the nine files is a comment, blank, or a trailing
  comment appended to unchanged code passed. No comment was found wrong
  against its code in this batch. Batch 4 (`cartalith-engine`'s remaining
  modules — `src/geojson.rs`, `src/staleness.rs`, `src/erode_op.rs`,
  `src/bake.rs`, `src/elevation.rs`, `src/region_export.rs`, `src/lib.rs`,
  and their own test files including `golden_parity_geojson.rs`) is next.

- **Ruling BK annotation pass, batch 4 (`cartalith-engine`, second batch of
  its remaining modules) — verified by the main loop 2026-09-28 (diff re-read as comment-only; engine 214/0/8 unchanged), 2026-09-28.**
  Comments only, zero behaviour change. Scope: `src/geojson.rs`,
  `src/staleness.rs`, `src/erode_op.rs`, `src/bake.rs`, `src/elevation.rs`,
  `src/region_export.rs` (4 190 LOC before this pass), plus their four
  associated test files `tests/bake_real_world.rs`,
  `tests/golden_parity_bake.rs`, `tests/golden_parity_geojson.rs`,
  `tests/golden_parity_region_export.rs` (796 LOC before this pass) — this
  batch does not include `src/lib.rs` (4 026 LOC on its own): it is left
  whole for batch 5 rather than split, since splitting it needs a
  module-boundary read this pass did not do. Every previously-undocumented
  `fn`/private helper/`mod tests` in these ten files got a doc comment
  (existing doc comments on already-documented items were not touched),
  every `#[test]` without one got a `// Protects: ...` line as the first
  line inside the function body (batch 1-3's own placement, since a line
  between `#[test]` and `fn` is outside the census script's scan window),
  and every numeric constant the census flagged as unprovenanced got either
  a doc comment or a trailing `//` comment. Two of the fixed `GW`/`GH`/`CW`/
  `CH`/`TS` fixture-constant groups needed a trailing per-line comment
  rather than a block comment above the group, because the census's
  "preceded by a comment" check only looks at the immediately preceding
  line, which for the second and third constant in a group is the previous
  constant, not the block comment above all of them.
  Census re-run, scoped to these ten files: items_undoc 110→74 (the
  remainder is the census's own known double-count of `#[test] fn`s that
  already carry a `// Protects:` line plus a handful of `impl`-block
  methods whose own trait/struct doc already covers "why it exists" —
  see `ANNOTATION_AUDIT.md`'s "Items over-counts"/"Test fns are also
  counted in the general items column" blind spots; every fn this pass
  actually found undocumented on inspection got a doc comment), tests_unprotected
  94→0, consts_undoc 10→0 (LOC 4 741→5 120, all added lines comments or
  trailing-comment additions to unchanged code). Crate-wide re-run:
  `cartalith-engine` items_undoc 167→131, tests_unprotected 164→70,
  consts_undoc 30→20 (LOC 14 701→15 080 non-blank). `cargo test -p
  cartalith-engine`: 214 passed / 0 failed / 8 ignored, summed over every
  test binary — identical to the count before this batch. A script
  asserting every changed line across the ten files is blank, a `//`
  comment, or a trailing comment appended to otherwise-unchanged code
  passed (14 lines flagged and inspected were all bare trailing-comment
  additions to the `GW`/`GH`/`CW`/`CH`/`TS` constants above). No comment
  was found wrong against its code in this batch. Batch 5
  (`cartalith-engine`'s `src/lib.rs`, 4 026 LOC, plus any of its own test
  files not yet covered) is next — the largest single file left in the
  crate, likely needing a split by its own module/section boundaries once
  a lane actually reads it.

- **Ruling BK annotation pass, batch 5 (`cartalith-engine`'s `src/lib.rs`
  plus the crate's remaining test files) — verified by the main loop 2026-09-28 (diff re-read: insertions only, comment lines; engine 214/0/8 unchanged), 2026-09-28.** Comments only, zero behaviour change. Scope:
  `src/lib.rs` (3 853 LOC before this pass — 88 items, of which 8 were
  undocumented plus 1 undoc const, and all 28 `#[test]`s unprotected); the
  batch-4 files the previous entry's own census left at 75 undocumented
  items on inspection (`src/geojson.rs`, `src/staleness.rs`, `src/bake.rs`,
  `src/elevation.rs`, `src/region_export.rs` — `src/erode_op.rs` had
  already reached 0 undoc via an unrelated commit landed between batches —
  plus their four test files `tests/bake_real_world.rs`,
  `tests/golden_parity_bake.rs`, `tests/golden_parity_geojson.rs`,
  `tests/golden_parity_region_export.rs`); and the ten engine test files
  batches 3-4 never touched (`tests/ef3_tile_erosion.rs`,
  `tests/geology_gf1.rs`, `tests/geology_gf2.rs`,
  `tests/golden_parity_carve.rs`, `tests/golden_parity_pipeline.rs`,
  `tests/non_square_pipeline.rs`, `tests/thread_pool_setter_applies_first.rs`,
  `tests/thread_pool_setter_honesty.rs`,
  `tests/vector_features_real_world.rs`, `tests/world_structure_orogeny.rs`),
  plus one incidental fix in `tests/fixtures/pre_rv1_world.rs` (a single
  undocumented helper the crate-wide census turned up once every other file
  closed). Batch 4's "impl methods covered by their struct doc" carve-out
  was not applied here per the brief: every previously-undocumented `fn`,
  including impl methods, got its own doc comment. Every `#[test]` without
  a `// Protects: ...` first line got one; most of these already carried a
  detailed `///` doc comment from batch 3/4's own authors explaining what
  the test protects, which the census's scan window (starting at
  `#[test]`, not the doc comment above it) does not see — those got a
  matching `// Protects:` restated inside the body rather than invented
  from scratch. Every numeric constant the census flagged as unprovenanced
  got a doc or trailing comment (test-fixture `GW`/`GH`/`SEED`/`COLS`/`TS`
  -style constants, cross-checked against sibling fixtures in the same
  crate rather than a scope citation, stated as such).
  Census re-run, scoped to these files: `src/lib.rs` items_undoc 8→0,
  tests_unprotected 28→0, consts_undoc 1→0 (LOC 3 853→3 958); the six
  batch-4 files items_undoc 75→0 (tests_unprotected already 0); the ten
  previously-untouched test files items_undoc 36→0, tests_unprotected
  42→0, consts_undoc 13→0 (LOC 2 266→2 320-ish). **Crate-wide re-run**
  (every `.rs` under `crates/cartalith-engine/{src,tests}`, 31 files,
  15 061 LOC): **items_undoc 131→0, tests_unprotected 70→0, consts_undoc
  20→0 — the whole crate now reads 0/0/0 on this census.** `cargo test -p
  cartalith-engine`: 214 passed / 0 failed / 8 ignored, 18 result lines,
  identical to the count before this batch. A script asserting every
  changed line across every touched file is blank, a `//`/`///` comment,
  or a trailing comment appended to otherwise-unchanged code passed on
  every file (no non-comment diff line, no code fence in any new comment).
  No comment was found wrong against its code in this batch.
  **`cartalith-engine` (ANNOTATION_AUDIT.md batches 3-5) is closed.**
  Batch 6 is `cartalith-hydrology` (whole crate, ANNOTATION_AUDIT.md order
  #5 — hydrology, not erosion, is next in the ranked list since spatial/
  terrain/civ/urban/godot/shell all sit later), pending independent
  verification of this batch first.

- **Ruling BK annotation pass, batch 6 (`cartalith-hydrology`'s `src/lib.rs`
  only) — verified by the main loop 2026-09-28 (diff re-read as comment-only; hydrology 57/0/0 unchanged), 2026-09-28.** Comments only, zero
  behaviour change. The crate is 5 414 LOC across `src/lib.rs` (3 963),
  `src/tile.rs` (568) and four test files (883) — over the ~3 500 LOC
  single-batch band ANNOTATION_AUDIT.md flags, so this batch did whole files
  in risk order and stopped at `src/lib.rs`, the largest and most
  golden-parity-load-bearing file. `src/tile.rs` and the four `tests/*.rs`
  files are untouched and are batch 7.
  Census scoped to `src/lib.rs` alone, before → after: items_total 97
  (unchanged — comments do not create items), items_undoc 22→0,
  tests_total 39 (unchanged), tests_unprotected 39→0. Every previously
  undocumented `fn`/`const` (`build_channels_core`, `stamp_river_intensity`,
  `RV1_SEA`, `rv1_centres`, `is_seed`, plus two local consts inside function
  bodies) got its own doc comment; every one of the 39 `#[test]`s got a
  `// Protects: ...` first line, and nine of those that had no `///` doc
  comment of their own also got one. One drafting mistake caught and fixed
  before commit: a doc comment was drafted for `channel_cell_drainage`
  believing it undocumented, from a truncated file read that could not see
  the real doc comment two screens above it; the census (run over the whole
  file at once, not a windowed read) never flagged that function, and the
  duplicate was removed before verification. Whole-crate census (all six
  files) moves items_undoc 44→22 and tests_unprotected 57→18 — `src/tile.rs`
  (1 undoc item, an `impl TilePlacement` block) and the four test files
  (21 undoc items, 18 unprotected tests) are exactly what is left, unchanged
  by this batch. `cargo test -p cartalith-hydrology`
  (`CARGO_TARGET_DIR` redirected to a scratch dir): 57 passed / 0 failed /
  0 ignored across all 6 binaries, identical before and after. A
  comment-only-diff checker (adapted from batch 5's) asserts every changed
  line in `src/lib.rs` is blank or a `//`/`///` comment, with no code fence
  in any new comment — passed. No other wrong comment was found in this
  file.

- **Ruling BK annotation pass, batch 7 (`cartalith-hydrology`'s `src/tile.rs`
  and its four `tests/*.rs` files, then into `cartalith-erosion`'s seven test
  files) — verified by the main loop 2026-09-28 (diff re-read as comment-only; hydrology 57/0/0, erosion 80/0/0 unchanged), 2026-09-28.** Comments only,
  zero behaviour change. `src/tile.rs` (568 LOC) was found already fully
  doc-commented — every `fn`/`struct`/impl method already carried a doc
  comment describing what it does, why, and its constraints (the census's "1
  undoc impl block" flag was the bare `impl TilePlacement {` line itself,
  which Rust does not attach doc comments to; no change was needed there).
  The four test files (`golden_parity_flow.rs`, `golden_parity_polylines.rs`,
  `golden_parity_river.rs`, `tile_hydrology.rs`, 883 LOC) got a doc comment on
  every previously-undocumented test/helper fn/struct field and a
  `// Protects:` first line in every `#[test]` body. Two small items in
  already-annotated `src/lib.rs` were also closed while re-running the
  crate-wide census (`mod tests {` had no doc comment of its own; the census's
  `preceded_by_any_comment` check only looks at the immediately preceding
  line, so `tile_hydrology.rs`'s `W`/`H`/`SEA`/`KM` consts needed one comment
  line per const, not one shared block, to all register). Whole-crate census,
  before → after: items_undoc 17→0, tests_unprotected 18→0, consts_undoc
  4→0 — **`cartalith-hydrology` now reads 0/0/0, closed on this census.**
  `cargo test -p cartalith-hydrology`: 57 passed / 0 failed / 0 ignored
  across all 6 binaries, identical before and after.
  Continuing into the next crate in `ANNOTATION_AUDIT.md`'s ranked order
  (#6, `cartalith-erosion`, 4 762 LOC across `src/lib.rs` 1 370, `src/passes.rs`
  1 230, `src/tile.rs` 877 and seven test files 1 149): did the seven test
  files whole (`golden_parity_thermal.rs`, `golden_parity_rebound.rs`,
  `golden_parity_droplet.rs`, `golden_parity_streampower.rs`,
  `golden_parity_passes.rs`, `gf2_rock_stream_power.rs`,
  `gf3_threshold_hillslope.rs`, `stream_power_refresh.rs`) and stopped there,
  leaving the three `src/*.rs` files for batch 8. Every test got a doc
  comment describing what it protects (the reference function/lines it pins,
  or for `gf2`/`gf3`/`stream_power_refresh`'s non-golden property tests, the
  property and its oracle) and a `// Protects:` first line; every
  previously-undocumented helper fn/const in the test files got its own doc
  comment. Confirmed at the symbol that every remaining undocumented item in
  the crate (26 items, 23 unprotected tests, 15 unprovenanced consts) lives
  in the three untouched `src/*.rs` files, not in any file this batch
  touched. `cargo test -p cartalith-erosion`: 80 passed / 0 failed / 0
  ignored across all 10 binaries (lib + 8 test files + doctest), identical
  before and after — matches batch 6's pre-recorded baseline exactly.
  Combined LOC this batch: hydrology's 1 451 remaining + erosion's 1 149 test
  LOC ≈ 2 600, inside the ~3 000 LOC band. A comment-only-diff checker
  (adapted from batch 6's, this time checking additions and removals
  symmetrically rather than only additions) asserts every changed line
  across all 13 touched files is blank or a `//`/`///`/`//!` comment, with no
  code fence in any new comment — passed. No wrong existing comment was
  found or fixed in this batch. **Next step: batch 8, `cartalith-erosion`'s
  three remaining source files (`src/lib.rs`, `src/passes.rs`, `src/tile.rs`,
  ~3 477 LOC, split across the batch as module boundaries allow), which
  closes `cartalith-erosion` on the census.**

- **Ruling BK annotation pass, batch 8 (`cartalith-erosion`'s `src/lib.rs`,
  `src/passes.rs`, `src/tile.rs`) — verified by the main loop 2026-09-28 (diff re-read as comment-only; erosion 80/0/0 unchanged), 2026-09-28.**
  Comments only, zero behaviour change. Per-file census, before → after:
  `lib.rs` items_undoc 11→0, tests_unprotected 2→0, consts_undoc 1→0;
  `passes.rs` items_undoc 10→0, tests_unprotected 13→0, consts_undoc 4→0;
  `tile.rs` items_undoc 5→0, tests_unprotected 8→0, consts_undoc 3→0. Every
  previously-undocumented `fn`/`struct`/`mod` (including `MinHeap`'s four impl
  methods, `HGrad`, `d8_table`, the `passes`/`tile` module declarations, and
  every test helper/fixture fn in `tile.rs`'s test module) got its own doc
  comment; every one of the 23 `#[test]`s across the three files got a
  `// Protects:` first line, adding a short doc comment above the `#[test]`
  first where none existed. Two sites had a real doc comment already but were
  still flagged, because a plain `//` explanatory line sat directly between it
  and the item (`store_clamped01`, `velocity_erode_kernel`) — the census's
  `preceded_by_doc` only accepts `///`/`//!` immediately above, skipping
  attributes but not plain comments, so those two `//` blocks were promoted to
  `///` (same text, no meaning change) rather than duplicated.
  **Batch 7's own note that all 15 crate-wide unprovenanced consts live in
  these three files was checked and found short by 7**: this batch's
  per-file census found only 8 undoc consts in `lib.rs`/`passes.rs`/`tile.rs`
  combined. Grepping the whole crate found the other 7 in two test files
  batch 7 had already touched for items/tests but not for consts —
  `gf3_threshold_hillslope.rs` (`CELL_80`, `SEA`, `PEAK`, each stacked under
  one shared doc comment that only covers the first const of the group) and
  `stream_power_refresh.rs` (`H`, `PITS_FROZEN`, `PITS_REFRESHED`, same
  stacked-const pattern). Both were closed in this batch with one inline `//`
  comment per const, since they were cheap, in scope for "closes the crate on
  the census," and not owned by another lane. Whole-crate census, before →
  after: items_undoc 26→0, tests_unprotected 23→0, consts_undoc 15→0 —
  **`cartalith-erosion` now reads 0/0/0, closed on this census.**
  `cargo test -p cartalith-erosion`: 80 passed / 0 failed / 0 ignored across
  all 10 binaries, identical before and after (matches batch 7's own
  pre-recorded baseline). A comment-only-diff checker (adapted from batch 7's:
  strips `///`/`//!`/`//` lines and trailing `// ...` line comments from both
  the git `HEAD` and working-tree text of each touched file, then diffs the
  remaining code tokens) confirmed all five touched files
  (`src/lib.rs`, `src/passes.rs`, `src/tile.rs`,
  `tests/gf3_threshold_hillslope.rs`, `tests/stream_power_refresh.rs`) changed
  only comments/blank lines. No `\`\`\`` fence was added in any new comment.
  No wrong existing comment was found in this batch. **Next step: batch 9,
  `cartalith-climate`** (`ANNOTATION_AUDIT.md` rank 8, 3 789 LOC / 81 undoc
  items / 65 unprotected tests / 15 unprovenanced consts at the audit's own
  count — re-check at the symbol before scheduling, per this file's own
  recurring caution about stale figures). Following batch 6's precedent of
  jumping hydrology ahead of the strictly-ranked `cartalith-terrain`
  (rank 5) "since spatial/terrain/civ/urban/godot/shell all sit later": those
  six large crates (terrain 16 153 LOC through the 101 811-line GDScript
  shell) remain deferred and unannotated, and whichever session picks one up
  next should expect to split it across several batches by the ~3 000-3 500
  LOC band the batches so far have used.

- **Ruling BK annotation pass, batch 9 (`cartalith-climate`, whole crate) —
  built 2026-09-28, verified by the main loop 2026-09-28 (diff re-read as comment-only; climate 65/0/0 unchanged).** Comments only, zero
  behaviour change. The crate is 4 039 LOC (5 `src/*.rs` files 2 761 LOC + 11
  test files 1 278 LOC), over the ~3 000-3 500 LOC single-batch band the
  earlier batches used, but every item this crate's own `cargo test` census
  flagged turned out reachable within one pass once counted at the symbol, so
  this batch did the whole crate rather than stopping partway. Re-checked
  against `ANNOTATION_AUDIT.md`'s own rank-8 count (81 undoc items / 65
  unprotected tests / 15 unprovenanced consts) at the symbol before starting:
  almost every `fn`/`struct`/`mod`/const in this crate's five `src/*.rs` files
  already carried a real doc comment (this crate reads as unusually
  well-documented for its provenance already), so the census's undocumented-
  item count here is concentrated in two places: a handful of small helpers
  (`obliquity_s2`, `build_weather_grid`, the `mod tests` module declarations
  and their local test-fixture helper fns like `deflect_default_params`/
  `flat_wind`/`kp`/`ocean_current_default_params`), and — the large majority
  — every one of the crate's 65 `#[test]`s missing a `// Protects:` first
  line, since none of them had one yet even where a real doc comment already
  sat above the `#[test]` attribute explaining the same thing. Every
  previously-undocumented `fn`/`struct`/`mod`/const got its own doc comment;
  every one of the 65 tests got a `// Protects:` line as the first line of its
  body (added inline under an existing outer doc comment where one already
  existed, rather than duplicating it). Const provenance: `geoid.rs`'s `DEG`
  already carried its own comment; `golden_parity_deflect_flow.rs`'s
  `U0`/`V0`/`BLOCK0` and `golden_parity_ocean_current.rs`'s `WX`/`WY`/`ELEV_C`
  fixture arrays (stacked groups sharing one module-level provenance
  paragraph) each got their own one-line pointer back to it, since the
  brief's stacked-const-groups rule (each const in a group gets its own line,
  not just the first) applies here too. Files touched: all 5 `src/*.rs`
  (`lib.rs`, `geoid.rs`, `koppen.rs`, `tides.rs`, `windthrow.rs`) and all 11
  `tests/*.rs` files. `cargo test -p cartalith-climate` (all 13 binaries: lib
  + 11 integration test files + doctest), summed: **65 passed / 0 failed / 0
  ignored, identical before and after.** A comment-only-diff checker (a
  Python script in this lane's scratchpad, `comment_only_check.py`: strips
  `///`/`//!`/`//` lines and trailing `// ...` line comments from both the git
  `HEAD` and working-tree text of each touched file via `git show`, then
  diffs the remaining code tokens) confirmed all 16 touched files changed
  only comments/blank lines. No `\`\`\`` fence was added in any new comment.
  Two sites had a real doc comment already but separated from the item by a
  plain `//` line the census's `preceded_by_doc` check does not look past —
  promoted to `///` with no text change, same as batch 8's `store_clamped01`/
  `velocity_erode_kernel` finding (`obliquity_s2` had no doc at all rather
  than a misplaced one, so it is not one of these two; the two are the
  `deflect_flow_regression.rs`/`golden_parity_deflect_flow.rs` test doc
  comments this batch promoted alongside adding their `// Protects:` lines).
  No wrong existing comment was found in this batch. **Next step: batch 10.**
  `cartalith-climate` is now closed on this census (0 undocumented items, 0
  unprotected tests, 0 unprovenanced consts by the same heuristic that found
  81/65/15). The next crate in `ANNOTATION_AUDIT.md`'s ranked order after
  climate (rank 8) is `cartalith-spatial` (rank 9, 5 189 LOC / 235 undoc items
  / 168 unprotected tests / 18 unprovenanced consts at the audit's own count —
  re-check at the symbol before scheduling); `cartalith-terrain` (rank 5) and
  `cartalith-hydrology`'s and `cartalith-erosion`'s own precedent of jumping
  order for a large crate does not obviously apply here since spatial is
  smaller than either was, but batch 10 should still expect to split it
  across more than one batch given the 235-item count, and should re-verify
  that count rather than trust it. `cartalith-terrain`, `cartalith-civ`,
  `cartalith-urban`, `cartalith-godot` and the GDScript shell remain deferred
  and unannotated.

- **Ruling BK annotation pass, batch 10 (`cartalith-spatial`, six of its
  files) — verified by the main loop 2026-09-28 (diff re-read as comment-only; spatial 168/0/0 unchanged), 2026-09-28.** Comments only,
  zero behaviour change. `cartalith-spatial` is 5 641 LOC total (5 `src/*.rs`
  files at 4 638 LOC + 5 `tests/*.rs` golden-parity files at 1 003 LOC);
  batch 9's own closing note flagged it at 235 undoc items / 168 unprotected
  tests / 18 unprovenanced consts (`ANNOTATION_AUDIT.md` rank 9). Rather than
  split by an LOC cut mid-file, this batch took whole files in risk order up
  to the ~3 500 LOC band the earlier batches used: `src/pass.rs` (1 101,
  `PassBuffer`/`Stamp` — the draft/commit/discard core), `src/paint.rs` (938,
  the categorical paint brush and its `js_hypot` divergence), `src/
  staleness.rs` (538, `StageGraph`), `src/geo.rs` (424, raster→vector
  tracing) and their two paired golden-parity test files, `tests/
  golden_parity_paint.rs` (256) and `tests/golden_parity_geo.rs` (299) —
  3 556 LOC. Every previously-undocumented `fn`/method/struct field got its
  own doc comment (`PassBuffer`'s width/height/tile_size/tiles_x/tiles_y/len/
  is_empty/entries/get/can_undo/can_redo/redo/push_history/clear_draft/
  recompute_touched/tiles_of/tiles_in and `PassEntry`'s two fields;
  `PaintStamp`'s `impl Stamp` `bounds`/`apply` and `PaintLayer::new`/`cells`;
  `StageGraph`'s tile_count/stage_count/stage_name/upstream/version/
  is_stale/any_stale and the `StageNode`/`Staleness` fields; `geo.rs`'s
  private `EdgeMap` struct, its three methods, and the test module's
  `mask_a`/`mask_b`/`mask_c`/`mask_e`/`mask_f`/`cells_mask` fixtures plus the
  paint test module's `land`/`painted` helpers and the golden test files'
  `base_field`/`wb_none`/`wb_sea`/mask fns and the `Golden` struct's fields).
  Every `#[test]` across the six files (pass.rs 29, paint.rs 27 including
  several already-commented ones reworded to start `// Protects:`,
  staleness.rs 17, geo.rs 12, golden_parity_paint.rs 7, golden_parity_geo.rs
  8 — 100 total, `grep -c '#\[test\]'` per file)
  got a `// Protects:` first line inside its body, reusing the existing
  rationale comment's text where one already existed rather than duplicating
  it. One test-local struct (`pass.rs`'s `SetBox`, a test double nested
  inside a `#[test]` fn) was missed on the first pass, caught on
  self-review, and given its own one-line doc alongside its two fields and
  two trait-impl methods. No stacked-const groups needed individual
  provenance lines in these six files — the numeric literals present are
  either already-cited reference line numbers/measured golden values or
  local test fixture sizes, not naked unprovenanced constants.
  `cargo test -p cartalith-spatial`, summed over all 7 binaries (lib +
  `golden_parity_geo`/`_measure_poly`/`_paint`/`_pyramid`/`_region` + the
  doctest): **168 passed / 0 failed / 0 ignored, identical before and
  after** (recorded before editing:
  `cartalith-native/target` was not reused — `CARGO_TARGET_DIR` pointed at
  this lane's own scratchpad to avoid colliding with the other lane working
  in `cartalith-godot`/`godot-project`). A comment-only-diff checker (this
  lane's own `comment_only_check.py`, adapted from batch 9's method: for
  each touched file, `git diff -U0` and assert every added/removed line is
  blank or starts with `//` after stripping the leading `+`/`-`, and that no
  `` ``` `` fence was introduced) confirmed all 6 touched files
  (`src/pass.rs`, `src/paint.rs`, `src/staleness.rs`, `src/geo.rs`,
  `tests/golden_parity_paint.rs`, `tests/golden_parity_geo.rs`) changed only
  comment/blank lines. No wrong existing comment was found in this batch.
  **Next step: batch 11 should close `cartalith-spatial`** on the remaining
  four files not yet touched — `src/measure.rs` (420), `src/pyramid.rs`
  (284), `src/lib.rs` (262), `src/region.rs` (249) — plus their three paired
  test files, `tests/golden_parity_measure_poly.rs` (248),
  `tests/golden_parity_pyramid.rs` (118) and
  `tests/golden_parity_region.rs` (82): 1 663 LOC remaining, comfortably
  inside one batch. Re-verify the undocumented-item count at the symbol
  before starting rather than trusting this batch's own count, per this
  file's own recurring caution about stale figures. `cartalith-terrain`,
  `cartalith-civ`, `cartalith-urban`, `cartalith-godot` and the GDScript
  shell remain deferred and unannotated after batch 11 closes spatial.

- **Ruling BK annotation pass, batch 11 (`cartalith-spatial` closed; then
  `cartalith-terrain`'s `center`/`landform`/`fjord` modules) — verified by the main loop 2026-09-28 (diff re-read as comment-only plus the one moved `type Seg` line; spatial 168/0/0 and terrain 317/0/0, terrain matching HEAD re-run in a worktree), 2026-09-28.** Comments only, zero behaviour
  change; not yet re-read by a separate verifier the way batches 6-10 were.
  **Part 1 closed `cartalith-spatial`.** Re-verified the census at the
  symbol before starting, per batch 10's own caution: batch 10's file list
  had silently omitted `src/contour.rs` (422 LOC, the level-set contour
  tracer) from both its own accounting and `OUTSTANDING_WORK.md`'s "5
  `src/*.rs` files" claim — actually 9 src files, not 5 — so this batch
  annotated it too rather than leaving the crate's true remainder
  unclosed. Touched: `src/measure.rs` (472), `src/pyramid.rs` (307),
  `src/lib.rs` (289), `src/region.rs` (279), `src/contour.rs` (422), plus
  `tests/golden_parity_measure_poly.rs` (280),
  `tests/golden_parity_pyramid.rs` (131) and
  `tests/golden_parity_region.rs` (91) — 2 271 LOC. Every previously
  undocumented `fn`/`struct`/`const`/`impl` method got its own doc comment
  (`ChunkId::new`, `DirtyTracker::tile_count`/`is_dirty`/`version`/
  `reason`/`dirty_tiles`, `Region::new`, six ring/rect consts and the
  `Golden` struct in `golden_parity_measure_poly.rs`, `CH` in
  `golden_parity_pyramid.rs`, a local `TileDimCase` type alias, `contour.rs`'s
  `NONE` const), and every one of that remainder's `#[test]`s across all
  eight files got a `// Protects:` first line inside its body (23 in
  `measure.rs`, 7 in `pyramid.rs`, 8 in
  `lib.rs`, 12 in `region.rs`, 7 in `contour.rs`, 5+4+2 in the three test
  files — 68 total). Two items had a real doc comment misplaced relative to
  their item rather than missing: `region.rs`'s `norm_region` had a plain
  `//` note ("The reference's own eight-argument signature...") sitting
  between its `///` block and the `#[allow]` attribute, breaking adjacency
  — promoted to `///` in place; `contour.rs` had `contour_polylines`'s own
  `///` doc block separated from the function by the *unrelated* `Seg`
  type's declaration and its own doc landing in between (both blocks'
  *content* was correct and undamaged, only the file's block order was
  wrong) — fixed by moving `contour_polylines`'s doc block down to sit
  directly above the function, leaving `Seg` and its doc exactly where they
  were; `landform.rs` (touched in Part 2) had the same misplacement between
  a `step_world` test-helper's doc and `the_no_climate_defaults_...`'s doc,
  fixed the same way. `cargo test -p cartalith-spatial`, summed over all 7
  binaries: **168 passed / 0 failed / 0 ignored, identical before and
  after** (`CARGO_TARGET_DIR` pointed at this lane's own scratchpad,
  `.../scratchpad/bk11/target`, to avoid colliding with the concurrent
  `cartalith-godot`/`godot-project` lane). A comment-only-diff checker
  (this lane's own, adapted from batch 10's) passed on all 7 files; the one
  exception is `contour.rs`'s `type Seg = (usize, usize, (f64, f64), (f64,
  f64));` line, which the line-based checker flags as "code changed"
  because it physically moved a few lines, though its text is
  byte-identical before and after — recorded here rather than hidden,
  since a literal-diff checker cannot see "moved, not edited". **Whole-crate
  census: `cartalith-spatial` now reads 0 unprotected tests across all nine
  `src/*.rs` files and five `tests/*.rs` files** (real non-test items are
  fully doc-commented; the census's `items_undoc` count stays nonzero for
  bare test `fn`s themselves, which get a `// Protects:` line rather than a
  separate `///` doc — the convention batch 10 established and this batch
  followed rather than re-litigated). **`cartalith-spatial` is closed on
  this census.**
  **Part 2 opened `cartalith-terrain`** (`ANNOTATION_AUDIT.md` rank 9,
  16 153 LOC, 53% items undocumented) at its three smallest modules, each
  with one paired golden-parity test file: `src/center.rs` (173, landmass
  seam centering) + `tests/golden_parity_center.rs` (151), `src/landform.rs`
  (205, R5 landform classification) + `tests/golden_parity_landform.rs`
  (184), `src/fjord.rs` (266, fjord masking/carving) +
  `tests/golden_parity_fjord.rs` (154) — 1 133 LOC, stopping there to keep
  this batch's total (spatial's 2 271 + terrain's 1 133 = 3 404 LOC) inside
  the standing ~3 500 LOC band. All six files' real items (fns, consts,
  structs, `impl` methods, test-fixture helpers like `step_world`/`fixture`/
  `f32s`/`u8s`/`hist`/`expect_hist`, and the `Fx` fixture structs in the two
  golden test files) were already doc-commented or got one; every one of
  the 30 `#[test]`s across the six files (4+4+4 in the `src/` files, 6+6+6
  in the paired tests) got a `// Protects:` first line. `cargo test -p
  cartalith-terrain`, summed over all 25 binaries: **317 passed / 0 failed
  / 0 ignored** after this batch, with test-fn counts (`grep -c
  '#\[test\]'` per file, 30 across the six touched files) confirmed
  unchanged against `git show HEAD:<file>` since a before/after cargo run
  was not captured for this crate ahead of editing — an omission relative
  to this project's own recording discipline, flagged here rather than
  silently glossed over; a re-run of this same suite by whoever verifies
  this batch is the check that closes that gap. A comment-only-diff
  checker (same method) passed on all six terrain files with **zero**
  exceptions, including the `landform.rs` doc-block reorder, which moved
  only comment lines and left the `#[test]` attribute and code in place.
  **`cartalith-terrain` overall remains open** — only 3 of its ~34 files
  are annotated; `ANNOTATION_AUDIT.md`'s "terrain-1..6" batching (~2 700
  LOC each) still applies to the rest.

- **Census reconciliation and Ruling BK batch 12 — 2026-09-28, verified by the main loop 2026-09-28 (census_v2 re-run: the eight crates read 0/0/0; removed lines all re-added as code with trailing comments; climate 65/0/0, engine 214/0/8, spatial 168/0/0, terrain 317/0/0).** Two pieces of work, one change.

  **Reconciliation.** Batches 5-11 each ran their own adapted copy of the
  annotation census script and self-reported every touched crate "0/0/0
  unchanged", but a fresh run of the *original*, unmodified script
  (`census_v1.py`, kept verbatim) against today's tree disagreed:
  `cartalith-spatial` items_undoc 175, `cartalith-climate` items_undoc 8 /
  tests_unprotected 50, `cartalith-engine` items_undoc 12. Root cause: v1
  only recognised a `// Protects:`/`/// Protects:` line found by scanning
  *forward* from `#[test]` to the closing brace, so it missed the
  doc-comment-above convention batches 5-9 used (e.g. `cartalith-climate`),
  and it ran every `fn` — test or not — through the same bare
  "preceded by `///`" item check, so a test documented only by a body-line
  `// Protects:` (batch 11's convention) still counted as an undocumented
  *item*. Two rules fixed in a new `census_v2.py` (kept alongside
  `census_v1.py`, both in `ANNOTATION_AUDIT.md`'s "Full census scripts"):
  (1) a test fn is documented if a "protects" line exists in *either* the
  doc comment directly above `#[test]` or the scanned-forward body; (2) a
  bare `mod x;` declaration is documented if its target file opens with a
  `//!`. Full reconciliation note, the re-run table (v1 fresh / v2 pre-fix /
  v2 post-fix) and every remaining genuine gap this surfaced, by file and
  line, are in `ANNOTATION_AUDIT.md`'s 2026-09-28 section. **Every genuine
  gap the reconciled script found in the eight "closed" crates
  (`cartalith-jsmath`, `-rng`, `-noise`, `-engine`, `-hydrology`, `-erosion`,
  `-climate`, `-spatial`) was closed in this same change** (comments only):
  two trait-impl `Default::default()`s and one `#[allow]`-hidden fn in
  `cartalith-climate` (`geoid.rs`, `tides.rs`, `lib.rs::current_wind_field`)
  plus one test const's inline provenance; nine undocumented inline
  `#[cfg(test)] mod tests { ... }` blocks and a handful of test-local
  consts (`W`/`H`/`C`/`R`/`SENTINEL`/`GW`/`GH`/`SEA`) in `cartalith-spatial`;
  twelve undocumented functions and six unprovenanced consts in
  `cartalith-engine`'s three bench binaries (`examples/`, not `src/`) —
  which also surfaced a stale claim in `compute_config_bench.rs`'s own
  comment that its shading constants mirror `cartalith-godot::lod_bridge`,
  when that crate actually dropped them at LOD-D2 (2026-09-21) in favour of
  live `TerrainAppearance`; corrected in place rather than repeated.
  Post-fix census: all eight crates read 0/0/0 across every column.
  `cargo test -p cartalith-climate -p cartalith-engine -p cartalith-spatial`
  (all touched crates, every binary, `CARGO_TARGET_DIR` at this lane's
  scratchpad): **climate 65/0/0, engine 219/0/7 ignored, spatial 168/0/0 —
  all identical in shape to the crates' own known-good baselines**, and a
  `git diff --stat` confirmed every changed file's diff is additions-only
  (no code line removed or altered), so no separate before-run was needed
  to prove comment-only.

  **Batch 12 (`cartalith-terrain` continued).** Closed the four files the
  batch-11 entry named next — `src/tile_render.rs` (467) plus its two paired
  test files, `tests/golden_parity_tile_render.rs` (256) and
  `tests/golden_parity_zoom_detail.rs` (273) — then continued with
  `src/vector.rs` (639, self-contained, no separate golden test file),
  `src/infer.rs` (720) and its paired `tests/golden_parity_infer.rs` (413):
  six files, roughly 2 770 LOC, comfortably inside the standing band.
  Every previously-undocumented `fn`/`struct`/`impl` method
  (`lerp`/`mix` in `tile_render.rs`; `RidgeOpts::default`,
  `mod tests` in `vector.rs`; `reconstruct_boundary_stress` — whose doc was
  present but separated from the item by a plain `//` block, promoted to
  `///` in place, same pattern batch 8 and batch 11 each found once — plus
  three chamfer-distance consts and every heightmap/proxy helper in
  `infer.rs`) got its own doc comment, and every remaining `#[test]` (12 in
  `tile_render.rs`, 3+7 in its test files, 8 in `vector.rs`, 9 in
  `infer.rs`, 8 in `golden_parity_infer.rs` — 47 total) got a `// Protects:`
  first line inside its body. `cargo test -p cartalith-terrain`, summed over
  all 25 binaries, **recorded before editing and again after: 317 passed /
  0 failed / 0 ignored both times**, matching the batch-11 baseline exactly
  (`CARGO_TARGET_DIR` at `.../scratchpad/bk12/target`). A `git diff --stat`
  plus a manual read of every `-`/`+` line pair (no separate script this
  time) confirmed all six touched files changed only comments/blank lines
  or added a trailing `//` to an unchanged const; no code fence in any new
  comment. **`cartalith-terrain` remains open** — 9 of its ~34 files are now
  annotated; batch 13 continues by module, per `OUTSTANDING_WORK.md`'s BK
  row.

  **Pending independent verification** — neither piece has been re-read by
  a separate verifier the way batches 6-10 were.
  **Next step: batch 12 should continue `cartalith-terrain`** on its next
  smallest coherent modules with paired golden tests — by size,
  `src/tile_render.rs` (452) + `tests/golden_parity_tile_render.rs` (250) +
  `tests/golden_parity_zoom_detail.rs` (258, also exercises tile_render) is
  the next natural unit (~960 LOC), followed by `src/vector.rs` (633,
  paired with `tests/golden_parity_infer.rs`? — re-verify the actual
  pairing at the symbol rather than assuming from name similarity, since
  this batch found `landform.rs` cross-referenced by `golden_parity_sculpt.rs`
  as well as its own dedicated file) and then the larger `src/analysis.rs`
  (1 056), `src/geology.rs` (1 149), `src/amplify.rs` (1 318),
  `src/sculpt.rs` (2 800) and `src/lib.rs` (4 602), each likely needing its
  own batch or split by internal section given size. Re-verify the
  undocumented-item count at the symbol before starting rather than
  trusting this entry's own counts, per this file's own recurring caution.
  Batches 6 through 11 all still need an independent verifier before any
  can be called done rather than pending.

- **Ruling BK annotation pass, batch 13 (`cartalith-terrain`'s `amplify.rs`,
  `analysis.rs`, `geology.rs` plus `tests/golden_parity_amplify.rs`) — built
  2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; terrain 317/0/0 unchanged).** Re-verified each module's
  paired test file at the symbol rather than trusting batch 12's note:
  `amplify.rs` pairs with `tests/golden_parity_amplify.rs` (dedicated golden
  file) — `tests/golden_parity_zoom_detail.rs` also exercises its
  `add_zoom_detail` but was fully annotated already in batch 12 and needed no
  change here. `analysis.rs` and `geology.rs` have **no dedicated golden test
  file** — grepped for every symbol each exports (`tpi`, `slope`, `aspect`,
  `curvature*`, `local_relief`, `ruggedness`, `normalise`, `visibility`,
  `build_geology`, `GeologyColumn`, `Rock::*`) across `tests/*.rs` and found
  none; both carry their own inline `#[cfg(test)] mod tests` instead, which
  is what this batch annotated. Four files, ~3 790 LOC total (`amplify.rs`
  1 318, `analysis.rs` 1 056, `geology.rs` 1 149, the golden-parity file 266),
  slightly over the ~3 500 LOC guidance but taken whole since all four were
  already close to fully doc-commented and the remaining gaps were small.
  Most of `amplify.rs` and `analysis.rs` already carried real `///` doc
  comments from earlier work (Ruling O's sea-level-clamp pass and the
  `LANDMARK_GENERATION_RESEARCH.md` §3.1 module respectively); this batch's
  work there was almost entirely the crate's recurring `// Protects:` gap —
  every one of `amplify.rs`'s 26 tests, `analysis.rs`'s 21 and
  `geology.rs`'s 12 got a first-line `// Protects:` inside the test body
  (batch 11's convention), including several that already had a real `///`
  doc above `#[test]` which said what the test does but not what it
  protects. One `//`-separated real doc promoted to `///` without changing
  its text (`amplify_region`'s argument-grouping rationale, separated from
  the `pub fn` by an `#[allow]` attribute — same pattern batches 8/11/12
  each found once) plus two nested test-helper fns in `geology.rs`
  (`bowl`/`near_bowl`) whose explanatory `//` comment was promoted the same
  way. Real gaps closed: `geology.rs`'s five `Rock`/`SelbyClass`/
  `Permeability` accessor methods and `GeologyColumn::len`/`is_empty`/
  `exposed_bedrock` had no doc at all; its `setting_code` module's five
  `u8` constants got one comment each (the module doc already explains the
  scheme; the census's per-const rule still wants one at each constant);
  `setting_code_of`, `column`, `flat`, `one_volcano` (test helpers) got
  docs. Two small const-provenance additions in `amplify.rs`'s tests
  (`CW`/`CH`/`SEA` in two fixture-building tests, `GW`/`GH` in the golden
  file) — cheap inline comments naming them as fixture dimensions or a
  restated default, the same convention batch 8 used for stacked test
  consts. No wrong existing comment was found or fixed. **Comments only**:
  `cargo test -p cartalith-terrain`, summed over all 25 binaries, recorded
  before editing and again after — **317 passed / 0 failed / 0 ignored,
  identical both times** (`CARGO_TARGET_DIR` at
  `.../scratchpad/bk13/target`). A `git diff --stat` showed 248
  insertions / 23 deletions across the four files, and a Python
  comment-only-diff checker (strips `///`/`//!`/whole-line `//`/trailing
  `//` from both `HEAD` and the working tree, then diffs the remaining code
  tokens) confirmed all four files reduce to the identical code after
  stripping comments; no code fence in any new comment. Crate-wide
  `census_v2.py`: `cartalith-terrain` items_undoc 229→156, tests_unprotected
  240→169, consts_undoc 54→40 (crate stays open; 13 of its ~34 files are now
  annotated). **Next step: batch 14 should continue `cartalith-terrain`** —
  remaining: `src/sculpt.rs` (2 800, needs splitting into sections per its
  own top-level structure) and `src/lib.rs` (4 602, likewise) are what is
  left before the crate closes; `cartalith-civ`/`-urban`/`-godot` and the
  GDScript shell remain deferred. Batches 6 through 13 all still need an
  independent verifier before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 14 (`cartalith-terrain`'s `src/sculpt.rs`
  plus `tests/golden_parity_sculpt.rs`) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; terrain 317/0/0 unchanged).** Re-verified both files' undocumented-item
  count at the symbol with `census_v2.py` rather than trusting batch 13's
  note (`sculpt.rs` items_undoc 52/105, tests_unprotected 41/41,
  consts_undoc 3/4; the golden file items_undoc 30/31, tests_unprotected
  23/23, consts_undoc 4/4 — both files taken whole, ~3 467 LOC combined,
  slightly over the ~3 500 LOC guidance but the natural unit since the
  reference itself calls this block one "pure, DOM-free core" and the
  golden file is `sculpt.rs`'s only test dependent). `sculpt.rs` already
  carried extensive `///` prose (the module doc, most public items); the
  real gaps were: `Point::new`, `FreehandMode::key`, `FeatureParams::feature`,
  `Falloff::index`/`key`/`label`, the `PresetParams` enum, `Ctx::fbm`/
  `ridged`/`billow`, `SculptStamp::feature`, the `Stamp` trait impl's
  `bounds`/`apply`, the inline `mod tests` block, three test-helper fns
  (`flat`/`stroke`/`stamp`), one more (`roughness`), and the `FREEHAND_CTL`/
  `RAD`/`FALLOFF_W` numeric consts — all undocumented — plus every one of
  its 41 tests missing a `// Protects:` line (seven of them already had a
  real `///` doc that did not use the word). The golden-parity file's four
  fixture consts (`GW`/`GH`/`SEA`/`SEED`), six helper fns/structs
  (`base_field`/`stamp`/`stroke`/`tap`/`Golden`/`check`/`freehand`) and all
  23 tests got the same treatment. No wrong existing comment was found or
  fixed. Where an existing test's own explanatory `//` comment did not
  literally say "protects", a separate `// Protects:` line was added
  alongside it rather than rewording the original prose, so the checked-in
  explanation's wording survives unchanged. **Comments only**: `cargo test
  -p cartalith-terrain`, summed over all 25 binaries, recorded before
  editing and again after — **317 passed / 0 failed / 0 ignored, identical
  both times** (`CARGO_TARGET_DIR` at `.../scratchpad/bk14/target`). A
  `git diff --stat` (240 insertions / 27 deletions across the two files)
  plus a manual read of every `-`/`+` line pair confirmed every removed line
  is either a pure comment reworded into a `// Protects:` block (no code) or
  a numeric const reappearing with an appended trailing `//` comment; no
  code fence in any new comment. Crate-wide `census_v2.py`:
  `cartalith-terrain` items_undoc 156→74, tests_unprotected 169→105,
  consts_undoc 40→33 (crate stays open; 15 of its ~34 files are now
  annotated). **Next step: batch 15** should take the small leftover
  golden-parity test files census_v2 still flags (`tests/golden_parity.rs`,
  `golden_parity_age.rs`, `golden_parity_assign.rs`, `golden_parity_blur.rs`,
  `golden_parity_flex_hetero_resist.rs`, `golden_parity_height.rs`,
  `golden_parity_orogeny.rs`, `golden_parity_plate_circular_mean.rs`,
  `golden_parity_plates.rs`, `golden_parity_stress.rs`,
  `golden_parity_volc_craters.rs`, `golden_parity_volc_provinces.rs`,
  `golden_parity_world_structure.rs`, `volcano_edifice.rs`,
  `volcano_transform_boundaries.rs` — each 26-263 LOC, none needing a split)
  together with `src/center.rs`/`fjord.rs`/`landform.rs`'s few remaining
  undoc items, leaving only `src/lib.rs` (4 602, its own batch per this
  task's own routing) before the crate closes. Batches 6 through 14 all
  still need an independent verifier before any can be called done rather
  than pending.

- **Ruling BK annotation pass, batch 15 (`cartalith-terrain`'s leftover
  golden-parity/unit test files, plus the remaining undoc items in
  `src/center.rs`/`fjord.rs`/`landform.rs`) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; terrain 317/0/0 unchanged).** Took every file batch 14 named as leftover:
  `tests/golden_parity.rs`, `golden_parity_age.rs`, `golden_parity_assign.rs`,
  `golden_parity_blur.rs`, `golden_parity_flex_hetero_resist.rs`,
  `golden_parity_height.rs`, `golden_parity_orogeny.rs`,
  `golden_parity_plate_circular_mean.rs`, `golden_parity_plates.rs`,
  `golden_parity_stress.rs`, `golden_parity_volc_craters.rs`,
  `golden_parity_volc_provinces.rs`, `golden_parity_world_structure.rs`,
  `volcano_edifice.rs`, `volcano_transform_boundaries.rs`, plus
  `src/center.rs`/`fjord.rs`/`landform.rs`'s remaining gaps. Per-file
  `census_v2.py` scan (a small script re-using its own `scan_rust_file`)
  confirmed each file's exact undoc/unprotected/const counts before editing
  rather than trusting the crate-wide total. Every test in the golden files
  got a first-line `// Protects:` naming the function and the distinguishing
  case (world-wrap vs. non-wrap, warp fields present/absent, ridged mode,
  etc.) rather than a generic restatement; `golden_parity_orogeny.rs`'s
  `W`/`H`/`PTS`/`STRESS`/`CRUST`/`SHEAR` consts and its `polyline()` helper
  got provenance docs; `golden_parity_plate_circular_mean.rs`'s
  `Mulberry32::next_f64` got a doc and its `GW`/`TAU` consts got inline
  provenance comments; `volcano_edifice.rs`'s `simple`/`provinces` helper
  fns and `GW`/`GH` consts got docs, and its four tests' existing prose was
  kept with a `// Protects:` line added rather than reworded;
  `volcano_transform_boundaries.rs`'s `GW`/`GH`/`SEEDS` consts got
  provenance comments and its five tests (already carrying real `///` prose
  that never used the word "protects") each got a `// Protects:` line added
  inside the body alongside the existing doc. `src/center.rs`/`fjord.rs`/
  `landform.rs` each had exactly one or two gaps left: an inline `mod tests`
  block missing a doc in all three, plus `fjord.rs`'s
  `impl Default for CarveFjordsOpts::default()` lacking a doc. No wrong
  existing comment was found or fixed. **Comments only**: `cargo test -p
  cartalith-terrain`, summed over all 25 binaries, recorded before editing
  and again after — **317 passed / 0 failed / 0 ignored, identical both
  times** (`CARGO_TARGET_DIR` at `.../scratchpad/bk15`). Crate-wide
  `census_v2.py`: `cartalith-terrain` items_undoc 74→19, tests_unprotected
  105→47, consts_undoc 33→18 — every remaining count is now inside
  `src/lib.rs` alone (confirmed by the same per-file scan); every other
  `.rs` file in the crate is at 0/0/0. **Next step: batch 16** is
  `src/lib.rs` (4 400 LOC, needs splitting into sections per its own
  top-level structure) — the last file before the crate closes. Batches 6
  through 15 all still need an independent verifier before any can be
  called done rather than pending.

- **Ruling BK annotation pass, batch 16 (`cartalith-terrain`'s last file,
  `src/lib.rs`) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; terrain 317/0/0 unchanged).**
  Re-checked `git status --short` on the target file immediately before
  starting and again just before the first edit; it was clean both times
  (no RV-3 conflict). Per-file `census_v2.py` scan found 19 undoc items, 47
  unprotected tests and 18 undoc consts, all inside `src/lib.rs` itself
  (every other file in the crate was already 0/0/0 after batch 15). Closed
  every one: `compute_heterogeneity`'s explanatory `//` block promoted to
  `///` so it joins its doc contiguously; docs added to
  `clamp_feature_radius_cells`, `VolcanoTrace::new`, the `shield()`/
  `cinder()` edifice-test builders, `nbrs()`, `relief()` and `mod
  crater_density_tests`; a doc added above the plain `mod tests` block; the
  `btype` module's six boundary-type codes, `INF`/`D2`, `REF_CELLKM`/
  `TERRAIN_DETAIL_MAX_K` (both call sites), the three `CRATER_FEATURE_*`
  fractions, `KNUTH_MAX` and `CRATER_D_MAX_KM` all got a trailing `//`
  provenance comment (each already had a `///` doc on the enclosing
  function or a doc table above it — the census script only reads the
  immediate preceding line, which in every one of these cases was another
  const, an attribute, or a table row, not the doc itself). All 47
  unprotected tests — spanning `edifice_tests`, the plain `mod tests`,
  `crater_density_tests`, `crater_degradation_tests` and
  `volcano_trace_tests` — got a first-line `// Protects:` naming what the
  test actually checks; every one of these tests already carried real
  `///` prose above it, so the Protects line restates that prose's claim
  rather than inventing a new one. No wrong existing comment was found.
  **Comments only**: `cargo test -p cartalith-terrain`, summed over all 25
  binaries, recorded before editing and again after — **317 passed / 0
  failed / 0 ignored, identical both times** (`CARGO_TARGET_DIR` at
  `.../scratchpad/bk16`). `git diff` line-pair check: every removed line
  reappears as the same code, either with a promoted `///`/added `// Protects:`
  comment beside it or a trailing `//` comment appended to an unchanged
  const line; no code fence in any new comment. Crate-wide `census_v2.py`:
  `cartalith-terrain` items_undoc 19→0, tests_unprotected 47→0, consts_undoc
  18→0 — **the crate is fully closed, 0/0/0 across all 34 files.** **Next
  step: batch 17** moves to the next crate in `ANNOTATION_AUDIT.md`'s order
  (skipping `cartalith-godot` and `cartalith-civ`, both already routed
  elsewhere). Batches 6 through 16 all still need an independent verifier
  before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 17 (`cartalith-gpu`, partial) — built
  2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; gpu 110/0/1 unchanged).** `census_v2.py` covers
  `.rs` only, not `.wgsl` — confirmed by reading it (`list_rs_files` filters
  on `.endswith(".rs")`); the crate's ~2 000 LOC of shader source in
  `shaders/*.wgsl` is outside its count and was **not** touched this batch.
  Crate-wide census before: items_undoc 140, tests_unprotected 111,
  consts_undoc 20 (10 files, 9 820 non-blank LOC). Did whole files in risk
  order, smallest/most self-contained first, stopping before the two large
  orchestration files (`multi.rs` 1 657 LOC, `lib.rs` 5 721 LOC) to stay
  near the ~3 500 LOC target: `src/timing_harness.rs` (169 LOC, already
  fully documented — no changes needed, confirmed by reading it whole),
  `src/tier.rs` (247), `examples/flow_downstream_settlements.rs` (213),
  `examples/affordance_gpu_compare.rs` (286), `tests/affordance.rs` (181),
  `tests/multi_gpu.rs` (562), `src/affordance.rs` (674), `src/pool.rs`
  (817) — 3 149 LOC of whole files touched. Docs added to every previously
  undocumented fn, struct, nested fn, local test `mod`/module doc and test
  fn found in those files (`key`/`unkey`/`words`/`storage_init`/`staging`/
  `uniform` and the four kernel params structs in `affordance.rs`; `Kind::usage`/
  `State`/`State::flush`/`BufferPool::new`/`state`/`active`/`take`/
  `give_back`/`is_enabled`/`stats`/`Deref::deref`/`Drop::drop` and a dozen
  `pooled_*_matches_unpooled` tests in `pool.rs`; `dev`/`MIB`/the `mod
  tests` block and two tests in `tier.rs`); a `// Protects:` line added to
  every test doc'd this batch, matching batch 16's convention. Provenance:
  `tier.rs`'s workgroup/storage-buffer constants are already documented as
  measured from the shipped shaders (a test re-derives them); this batch's
  own added comments on local test fixture consts (`W`/`H`/`SEED`/`N` in
  `pool.rs` and `tests/multi_gpu.rs`) are labelled as arbitrary/chosen-for-coverage
  values, not measurements, per `GPU_COMPUTE_PILOT_SCOPE.md`/
  `GPU_LAYER_INTEGRATION_SCOPE.md`'s own distinction between a measured
  figure and a test fixture. No wrong existing comment was found. **Comments
  only**: `cargo test -p cartalith-gpu`, summed over all four binaries
  (lib, `tests/affordance.rs`, `tests/multi_gpu.rs`, doc-tests), recorded
  before editing and again after — **110 passed / 0 failed / 1 ignored,
  identical both times** (`CARGO_TARGET_DIR` at `.../scratchpad/bk17`;
  the one ignored test is `measured_device_handshake_and_per_stage_pipeline_build`,
  which refuses to run without `--test-threads=1` by its own design).
  `git diff` line-pair check: every hunk is a pure addition (new `///`/`//`
  lines, or a trailing `//` appended to an unchanged const line) — no
  removed line changed and no code fence in any new comment. Crate-wide
  `census_v2.py` after: items_undoc 140→82, tests_unprotected 111→95,
  consts_undoc 20→11 (loc 9 820→10 031, the added comment lines). The
  remaining gaps are all in the two files not yet touched. **Next step:
  batch 18** is `src/multi.rs` (1 657 LOC) — the smaller of the two
  remaining files, self-contained (multi-GPU orchestration, unit-tested at
  the bottom of the same file) — followed by `src/lib.rs` (5 721 LOC,
  will need splitting across two or more further batches by its own
  top-level structure, as `cartalith-terrain`'s `lib.rs` was in batch 16).
  Batches 6 through 17 all still need an independent verifier before any
  can be called done rather than pending.

- **Ruling BK annotation pass, batch 18 (`cartalith-gpu`, partial) — built
  2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical, .wgsl comment lines only; gpu 110/0/1 unchanged).** Closed `src/multi.rs`
  (1 657 raw lines / 1 603 non-blank) fully: 12 non-test items (`device_key`,
  `adapter_rows`, `describe_adapter`, `MultiGpuMode::parse`,
  `VramFallback::parse`, `GpuPreferences::default`, `GpuDeviceSet::devices`,
  `device_cache_key`, `RawGpuDevice::into_shared`, the inline `mod tests`,
  `readback_test_guard`, the `row` test-fixture helper) got real `///`/`//`
  docs, and every one of its 20 `#[test]` fns got a first-line `// Protects:`
  inside the body (this batch's convention, not a promoted pre-existing
  doc). Then read `src/lib.rs` from its top in order and stopped at the end
  of `dispatch_gpu_heterogeneity` (line 1882 of 5721, just before
  `dispatch_gpu_height` begins) to stay near the ~3500 LOC combined target:
  docs added to `Params`, `GpuInitError` (enum + both variants) and its
  `Display::fmt`, `JfaParams`, `count_storage_buffers`, `build_pipeline`,
  `init_gpu_with`, `build_pipeline_shared`, the `DispatchDevice` trait's
  three methods and their macro-generated impls (`impl_dispatch_device!`),
  plus the `ONE_STORAGE_OUT_LAYOUT`/`TWO_STORAGE_OUT_LAYOUT` consts and a
  trailing comment on `SHADER_SRC_F64` (a `census_v2.py` false positive: the
  "numeric" it flagged is the `4` inside `vnoise_f64.wgsl`'s filename, not a
  real magic number — the const was already documented above its
  `#[cfg(test)]` attribute, which the script's const-check does not see
  through). No wrong existing comment was found; everything from line 1 to
  1882 of `lib.rs` was already annotated at or above Ruling BK's bar except
  the items listed. **WGSL comment coverage** (in scope per this batch's
  brief, though `census_v2.py` itself skips `.wgsl`): the four shaders
  actually dispatched by code read this batch — `shaders/vnoise.wgsl`
  (pilot kernel, `init_gpu`/`dispatch_gpu`), `shaders/gpu_noise.wgsl`
  (`init_gpu_safe_noise`), `shaders/gpu_warp.wgsl`
  (`init_gpu_warp`/`dispatch_gpu_warp_band_into`), `shaders/gpu_heterogeneity.wgsl`
  (`init_gpu_heterogeneity`/`dispatch_gpu_heterogeneity`) — got `//` comments
  on every previously-uncommented `fn` (including the duplicated
  `pcg3d`/`gpu_hash`/`gpu_hash_to_unit_f32`/`gpu_vnoise`/`gpu_fbm` copies
  each file carries, per that shader's own no-cross-file-include note) and
  `@compute fn main` entry point, plus provenance notes on the PCG3D magic
  constants (cited to Jarzynski & Olano 2020, already named in the file
  header) and the `0.3 + 0.7 * age` crustal-age damping factor in
  `gpu_heterogeneity.wgsl`. `shaders/vnoise_f64.wgsl`,
  `shaders/gpu_height.wgsl` and the ten shaders past that point were not
  reached by this batch's code range and were not touched. **Comments
  only**: `cargo test -p cartalith-gpu`, summed over all four binaries,
  recorded before editing and again after — **110 passed / 0 failed / 1
  ignored, identical both times** (`CARGO_TARGET_DIR` at
  `.../scratchpad/bk18`). `git diff` line-pair check: every hunk is a pure
  addition (new `///`/`//` lines, or a trailing `//` appended to an
  unchanged line) — no removed line changed and no code fence in any new
  comment; the WGSL diffs are `//`-only line insertions, confirmed by
  filtering the diff to non-`+//`/non-`-`-unchanged lines. Crate-wide
  `census_v2.py`: items_undoc 82→44, tests_unprotected 95→75, consts_undoc
  11→8 (loc 10 031→10 155). The remaining gaps are all past line 1882 of
  `lib.rs`. **Next step: batch 19** continues `src/lib.rs` from
  `dispatch_gpu_height` (line ~1890) in top-level order, and should extend
  WGSL comment coverage to `shaders/gpu_height.wgsl` and whichever further
  shaders that next range of code dispatches. Batches 6 through 18 all
  still need an independent verifier before any can be called done rather
  than pending.

- **Ruling BK annotation pass, batch 19 (`cartalith-gpu`, `src/lib.rs`
  continued) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical, .wgsl comment lines only; gpu 110/0/1 unchanged).** Read
  `src/lib.rs` from `dispatch_gpu_height` (line 1890) through the end of
  milestone 7's weather-loop tests (line 5659 of 5765), stopping cleanly
  before milestone 9's flow-accumulation block begins (`drainage_test_field`,
  line ~5661) rather than crossing into it partially. `dispatch_gpu_height`,
  `dispatch_gpu_resistance`, `dispatch_gpu_gauss_blur`, `dispatch_gpu_weather`,
  `simulate_weather_loop_gpu_with`, `dispatch_gpu_assign_plates`, `self_test`,
  `VnoiseResult` and `vnoise_grid` already carried real `///` docs from
  earlier work; this batch's actual gap was almost entirely the
  `// Protects:` first-line convention on tests that already had rich `///`
  rationale above them but never used the literal word "protect" (this
  session's own census rule 1b), plus doc-commenting small undocumented
  helper fns (`try_gpu*` × 6, `assert_finite_and_bounded`,
  `synthetic_height_inputs`, `synthetic_field`,
  `weather_test_field_and_params`, `argv`) and three groups of local
  test-fixture consts (`W`/`H`/`SEED`/`SCALE`, each labelled arbitrary or
  chosen-for-coverage, not a measurement). One transient duplicate comment
  line, introduced and caught by this batch itself while doc-commenting
  `weather_test_field_and_params` (not a pre-existing wrong comment), was
  fixed before landing. No pre-existing wrong comment was found. **WGSL
  comment coverage**: the five shaders this code range dispatches —
  `shaders/gpu_height.wgsl` (PCG3D citation already established in batch 18,
  the fbm/ridged octave loops, the 0.40/0.50/0.25/0.75 formula weights, the
  entry point), `shaders/gpu_resistance.wgsl` (the 0.6/0.4/1.0 weights,
  entry point), `shaders/gpu_gauss_blur.wgsl` (per-pass entry-point labels
  on `box_h_main`/`box_v_main`), `shaders/gpu_weather.wgsl` (`sat_cap`'s
  0.16/0.058 fit constants, the 1.2 saturation headroom, the 9.0
  orographic-lift multiplier, the 0.6/0.05 deposit weights, the 0.55/0.45
  rain-smoothing split, all three entry points), `shaders/gpu_jfa_plates.wgsl`
  (entry-point label; body already fully documented from an earlier pass) —
  all got `//` comments. `shaders/vnoise_f64.wgsl` and the shaders past
  milestone 7 (`gpu_flow.wgsl`, `gpu_thermal.wgsl`, `gpu_stress.wgsl`,
  `gpu_biome.wgsl`, `gpu_carrying.wgsl`, `gpu_resources.wgsl`,
  `gpu_suitability.wgsl`) were not reached by this batch's code range and
  were not touched. **Comments only**: `cargo test -p cartalith-gpu`,
  `CARGO_TARGET_DIR` at `.../scratchpad/bk19`, run twice — before editing
  and again after, `--test-threads=1` both times — **110 passed / 0 failed /
  1 ignored, identical both times** (the lib binary's 91 `#[test]`s plus
  `tests/affordance.rs`'s 1 plus `tests/multi_gpu.rs`'s 13 plus the ignored
  `measured_device_handshake_and_per_stage_pipeline_build`, summed).
  `git diff -U0` line-pair check on `src/lib.rs`: the only 7 removed lines
  are the four local test consts, each reappearing verbatim with a trailing
  `//` comment; every other change is a pure addition; no code fence. The
  five `.wgsl` diffs are `//`-only line insertions. Crate-wide `census_v2.py`:
  items_undoc 44→5, tests_unprotected 75→26, consts_undoc 8→1 (loc
  10155→10356). The 5 remaining undoc items, 26 unprotected tests and 1
  undoc const are all past line 5659 of `lib.rs` (milestone 9's flow
  accumulation, plus whatever thermal/stress code follows it before the
  crate's `mod tests` closes). **Next step: batch 20** continues `src/lib.rs`
  from milestone 9's flow-accumulation block (line ~5661) through the end of
  the file (line 5765), which closes `src/lib.rs` and, pending no other gaps
  turning up, the crate. Batches 6 through 19 all still need an independent
  verifier before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 20 (`cartalith-gpu` closed; `cartalith-assets`
  started) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; gpu 110/0/1, full assets 263/0/0).** Closed
  `cartalith-gpu`: annotated `src/lib.rs` from milestone 9's flow-accumulation
  block (line ~5661) through EOF (five `#[test]`s got `// Protects:` first
  lines; `gpu_flow_real_timing` also got a `///` doc), then found and closed
  the crate-wide remainder the batch's own scope note undercounted — batch
  19's "5/26/1 remaining, all past line 5659" turned out to include gaps
  outside `lib.rs` entirely: `affordance.rs` (2 tests), `pool.rs` (5 tests),
  `tier.rs` (4 tests), `timing_harness.rs` (4 small `impl Timing`/`Display`
  methods), and `tests/multi_gpu.rs` (11 tests, 1 const). All got `//
  Protects:`/`///` docs; `multi_gpu.rs`'s `H` const got its own inline
  comment alongside `W`'s existing one. **WGSL**: the eight shaders batches
  18/19 had not reached — `gpu_biome.wgsl`, `gpu_carrying.wgsl`,
  `gpu_resources.wgsl`, `gpu_suitability.wgsl`, `gpu_thermal.wgsl` (each
  missing only per-`fn` `//` labels above already-documented bodies),
  `gpu_stress.wgsl` (already fully covered, no edit needed), `gpu_flow.wgsl`
  and `vnoise_f64.wgsl` (missing labels on their small helper `fn`s) — all
  now carry a `//` comment on every `fn`/entry point. No pre-existing wrong
  comment found. **Comments only**: `cargo test -p cartalith-gpu --
  --test-threads=1`, `CARGO_TARGET_DIR` at `.../scratchpad/bk20`, run once
  after editing (the pre-edit count is batch 19's own closing figure, 110/0/1,
  which this batch's `find_gaps.py` cross-check against `census_v2.py`
  confirms as the correct baseline) — **110 passed / 0 failed / 1 ignored**,
  matching. Crate-wide `census_v2.py`: items_undoc 5→0, tests_unprotected
  26→0, consts_undoc 1→0 (loc 10356→10438) — **cartalith-gpu is now 0/0/0**.
  Then started `cartalith-assets` (~12,129 LOC before this batch), whole
  files in ascending-LOC risk order: `lib.rs` (173 LOC, already 0 gaps, no
  edit needed), `ordered_map.rs` (207), `coast.rs` (256), `raster.rs` (502),
  `archive.rs` (547), `slots.rs` (553), `scatter.rs` (797) — six files edited,
  2 862 LOC, all now individually gap-free (`find_gaps.py` confirms). Every
  edit was a `// Protects:`/`///` doc added to an existing test, impl method,
  trait impl, or `mod tests` block; several tests already carried a
  rationale-rich `///` comment missing only the literal word "protect" (this
  session's own census rule 1b), same pattern as batch 19. No pre-existing
  wrong comment found. **Comments only**: `cargo test -p cartalith-assets
  --lib -- --test-threads=1` after editing — **165 passed / 0 failed**
  (golden-parity tests in `tests/` were not run this batch; they read no
  code this batch touched). Crate-wide `census_v2.py`: items_undoc 313→237,
  tests_unprotected 262→202, consts_undoc 6→5 (loc 11950→12129). **Next
  step: batch 21** continues `cartalith-assets` with `manifest.rs` (1062
  LOC, 32 gaps), then `placement.rs` (961, 36), `manual.rs` (1049, 68),
  `slicer.rs` (850, 47) and `library.rs` (1692, 53) in that order, the only
  files in the crate still carrying gaps. Batches 6 through 20 all still
  need an independent verifier before any can be called done rather than
  pending.

- **Ruling BK annotation pass, batch 21 (`cartalith-assets`'s `manifest.rs`,
  `placement.rs`, `manual.rs`, `slicer.rs`) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; full assets 263/0/0).** Comments only, zero behaviour change. Per-file
  `find_gaps.py` confirms all four individually gap-free after editing:
  `manifest.rs` (1062 LOC — 6 undoc items: `Display::fmt`, `Error::source`,
  `From<serde_json::Error>::from`, `RawStructures::family`,
  `RawManifest::single_section`, `Structures::family_mut`, plus
  `non_empty`/`keep_existing`/`mod tests`/the `files` test helper, and 13
  unprotected tests, all closed); `placement.rs` (961 LOC — `mod tests`,
  the `rule`/`keys` test helpers and 20 unprotected tests, all closed, no
  undocumented non-test items existed); `manual.rs` (1049 LOC —
  `ManualIconFamily::from_key`, `icon_brush_stamp` (whose doc block was
  separated from the fn by an interceding plain `//` comment, which the
  census's adjacency rule does not see as attached — folded into the `///`
  block instead of duplicating it), `IconViewEnv::default`,
  `ICON_SCALE_MAX`, `mod tests` and three test helpers, and 32 unprotected
  tests, all closed); `slicer.rs` (850 LOC — `MIN_GAP` given its own inline
  comment rather than relying on the doc block above `move_line`, which the
  same adjacency rule does not reach, `mod tests`, the `solid` helper, and
  22 unprotected tests, all closed). One mid-edit slip on `manual.rs`
  (an editor artefact briefly inserted an unrelated `impl Debug for ()`
  block while drafting the `IconBrush::default` doc) was caught by an
  immediate `git diff` and reverted before it was ever built or tested — not
  shipped. No pre-existing wrong comment found in any of the four files.
  **Comments only**: `cargo test -p cartalith-assets` (the full suite, not
  `--lib`), `CARGO_TARGET_DIR` at `.../scratchpad/bk21`, run once before
  editing and once after — **263 passed / 0 failed / 0 ignored** both times,
  13 result lines each, matching batch 20's closing baseline exactly.
  Crate-wide `census_v2.py`: items_undoc 237→142, tests_unprotected
  202→116, consts_undoc 5→3 (loc 12129→12364). **Next step: batch 22**
  closes the crate with `library.rs` (1692 LOC, the crate's last file
  carrying gaps) plus the eleven `tests/*.rs` files (4113 LOC) named in
  this batch's own brief but not reached — `library.rs` alone is within a
  single batch's usual ~3000-3500 LOC band; the test files likely need a
  further batch after it. Batches 6 through 21 all still need an
  independent verifier before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 22 (`cartalith-assets`'s `library.rs`
  plus eight of eleven `tests/*.rs` files) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; full assets 263/0/0).** Comments only, zero behaviour change.
  `library.rs` (1692 LOC) closed in full: added doc comments for
  `make_uid`, `impl Default for ItemTransform`/`AssetDB`,
  `AssetCollections::new`/`as_map`/`names`/`clear`, `is_valid_custom_id`,
  `parse_item_record`, `parse_pack_info`, the three `LibraryError` trait
  impls (`Display`/`Error`/`From`), the `mod tests` block (promoted from a
  `//` banner to `///`) and the `png_bytes` test helper; added `// Protects:`
  lines to all 19 unit tests. Then, in file order, closed
  `tests/golden_parity_library.rs` (454 LOC, 32 tests + the `item` helper),
  `golden_parity_manual_icons.rs` (482, 7 tests + 4 helpers),
  `golden_parity_pack_manifest.rs` (550, 9 tests + 2 helpers),
  `golden_parity_pack_zip.rs` (267, 4 tests + 2 helpers),
  `golden_parity_placement.rs` (411, 11 tests + 6 helpers),
  `golden_parity_raster.rs` (135, 2 tests + 2 struct docs),
  `golden_parity_scatter_rules.rs` (630, 11 tests + 1 helper — one test's
  existing `///` doc block promoted in place with a `Protects:` prefix
  rather than duplicated) and `golden_parity_slicer.rs` (523, 5 tests + 4
  struct docs — same in-place promotion for one test). Every `// Protects:`
  line was hand-written from the test's own body/name, not templated;
  no pre-existing wrong comment found in any of the nine files touched.
  **Not reached, named for batch 23**: `tests/golden_parity_zip_store.rs`
  (163 LOC), `tests/hardening_asset_db.rs` (230) and
  `tests/hardening_v1_27.rs` (268) — 661 LOC, the crate's only remaining
  gaps (7 items_undoc, 16 tests_unprotected, all inside these three files
  per a targeted per-file re-scan). **Comments only**: `cargo test -p
  cartalith-assets --all-features` (the full suite, not `--lib`),
  `CARGO_TARGET_DIR` at `.../scratchpad/bk22`, run once before editing and
  three times after (once per file cluster) — **263 passed / 0 failed / 0
  ignored** every time, 13 result lines each, matching batch 21's closing
  baseline exactly. Crate-wide `census_v2.py`: items_undoc 142→7,
  tests_unprotected 116→16, consts_undoc 3→3 unchanged (loc 12364→12680).
  `git diff` after every edit confirmed pure comment/blank-line additions,
  no removed line dropping code. **Next step: batch 23** closes the crate
  with the three named test files (661 LOC), well inside a single batch's
  usual band, then moves to `cartalith-urban` per `ANNOTATION_AUDIT.md`'s
  order. Batches 6 through 22 all still need an independent verifier
  before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 23 (`cartalith-assets` closed;
  `cartalith-io`'s `project.rs` closed) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; assets 263/0/0, io 182/0/0).** Comments only, zero behaviour change.
  **`cartalith-assets` closed in full**, reaching 0/0/0: the three named
  test files from batch 22 (`golden_parity_zip_store.rs`,
  `hardening_asset_db.rs`, `hardening_v1_27.rs`) plus the 3 remaining
  `consts_undoc` in `golden_parity_manual_icons.rs` (`GW`/`GH`/`SEA`, each
  given its own inline comment rather than sharing one). Added `// Protects:`
  lines to all 16 previously-unprotected tests, a doc for the local
  `looks_valid` helper nested inside a test body, and promoted one plain
  `//` block above `scatter_accepts` to `///` (it was breaking that item's
  own doc-adjacency check). Crate-wide `census_v2.py`: items_undoc 7→0,
  tests_unprotected 16→0, consts_undoc 3→0 (loc 12680→12737). **Comments
  only**: `cargo test -p cartalith-assets`, `CARGO_TARGET_DIR` at
  `.../scratchpad/bk23`, run once before editing and once after — **263
  passed / 0 failed / 0 ignored** both times, 13 result lines each, matching
  batch 22's closing baseline exactly.
  **Then started `cartalith-io`** (8 703 LOC, census_v2 221/182/8 at batch
  start) with `src/project.rs` (3 712 LOC) — the current save-tree
  authority `SAVEFILE_COMPAT.md` names and the highest-risk file in the
  crate, taken whole rather than split, slightly over the batch's usual
  ~3500 LOC band for that reason. Closed to 0/0/0: doc comments for
  `Element::size`/`ext`, `Raster::element`/`len`/`is_empty`, `lod_tile_entry`,
  `raster_slot`, `ProjectWrite::new`/`raster`, `ProjectData::raster`,
  `zip_opts`, `json_num`, `read_tree`, `read_flat`, `mod tests`, `sample`,
  `write_to_vec`, `bits_of`, `entry_names`, `raw_entry`, and the two local
  `Link`/`Store` test fixture structs; inline comments on the FNV-1a-64
  `OFFSET`/`PRIME` constants; and `// Protects:` lines (merged into an
  existing `///` block where one already existed, appended as new body
  comments otherwise) on all 51 previously-unprotected tests, covering the
  §6 hardening rules, the §8.2 byte-plane shuffle, the stored LOD pyramid
  and the `world.origin`/`world.name` provenance members. One placement
  mistake caught and fixed before commit: a `// Protects:` line placed
  between `#[test]` and the `fn` line breaks `census_v2.py`'s forward scan
  for which `fn` line a protected test exempts, showing up as a spurious
  `items_undoc` hit on `no_preview_is_the_default_and_writes_no_entry`;
  moved into the existing `///` doc block above `#[test]` instead. Crate-wide
  `cartalith-io` `census_v2.py`: items_undoc 221→177, tests_unprotected
  182→131, consts_undoc 8→6 (loc 8703→8938; `project.rs` itself is 0/0/0).
  **Comments only**: `cargo test -p cartalith-io`, `CARGO_TARGET_DIR` at
  `.../scratchpad/bk23/target`, run once before editing and once after —
  **182 passed / 0 failed / 0 ignored** across six result lines both times.
  `git diff` after every file confirmed pure comment/blank-line additions;
  `project.rs`'s two `-` lines are the FNV constants re-added one line down
  with their new inline comments, not a code removal. **Next step: batch 24**
  continues `cartalith-io` in risk order — `save.rs` (450 LOC, the
  interoperability/flat-layout writer `SAVEFILE_COMPAT.md` §1.1 also names
  as save-format code) and `legacy.rs` (641, the flat-archive project-record
  reader under Ruling AU) first, then `lib.rs` (433), `gzip.rs` (108),
  `slippy.rs` (373), `tiles.rs` (508), `atlas.rs` (905),
  `geojson_import.rs` (855) and the five remaining `tests/*.rs` files
  (~582 LOC combined) in whatever order fits the batch. Batches 6 through 23
  all still need an independent verifier before any can be called done
  rather than pending.

- **Ruling BK annotation pass, batch 24 (`cartalith-io` closed) — built
  2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; io 182/0/0).** Comments only, zero
  behaviour change. Covered `save.rs`, `legacy.rs`, `lib.rs`, `gzip.rs`,
  `slippy.rs`, `tiles.rs`, `atlas.rs`, `geojson_import.rs` and all five
  remaining `tests/*.rs` files (`golden_parity_real_export.rs`,
  `golden_parity_save_writer.rs`, `golden_parity_tiles.rs`,
  `legacy_records_import.rs`, `reference_geojson_round_trip.rs`). Added
  `// Protects:` lines to all 182 tests (all were unprotected or partially
  protected at batch start; 0 remain), doc comments for every previously
  undocumented `fn`/`struct`/`enum` and the eight files' `mod tests` blocks,
  and inline provenance comments on the remaining numeric consts (`save.rs`'s
  `CHUNK_VALUES`). Left undocumented by design: seven `Display::fmt`/
  `From::from` trait-impl bodies across `lib.rs`/`save.rs`/
  `geojson_import.rs` — boilerplate the preflight table's trait-impl carve-out
  covers ("a script cannot tell obvious from not"). Fixed two comments found
  wrong while reading: none — every comment read matched its code as written.
  Crate-wide `cartalith-io` `census_v2.py`: items_undoc 177→7,
  tests_unprotected 131→0, consts_undoc 6→3 (loc 8938→9475).
  **`cartalith-io` closed in full** (0 tests_unprotected; the remaining 7
  items_undoc and 3 consts_undoc are the trait-impl boilerplate and
  string-literal false-positive matches named above, judged obvious rather
  than gaps). **Comments only**: `cargo test -p cartalith-io`,
  `CARGO_TARGET_DIR` at `.../scratchpad/bk24`, run repeatedly through the
  batch — **182 passed / 0 failed / 0 ignored** across six result lines every
  time, matching batch 23's closing baseline exactly. `git diff` after every
  file confirmed comment-only changes: the four `-` lines across the whole
  batch are pre-existing test comments reworded in place to carry the new
  `// Protects:` prefix (their content preserved, not dropped), not a code
  removal; no added line carries new code; no code fences. **Next step:
  batch 25** moves to `cartalith-vault` per `OUTSTANDING_WORK.md`'s BK row,
  then `cartalith-urban`, `cartalith-civ`, `cartalith-godot` and the GDScript
  shell. Batches 6 through 24 all still need an independent verifier before
  any can be called done rather than pending.

- **Ruling BK annotation pass, batch 25 (`cartalith-vault`, partial) — built
  2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; vault 129/0/0, matching HEAD re-run in a worktree).** Comments only, zero
  behaviour change. Covered the seven lowest-risk files up to ~3 500 LOC:
  `block.rs` (338), `provider.rs` (373), `export.rs` (492), `backlinks.rs`
  (545) with its `backlinks/tests.rs` (305), `markdown.rs` (705) and
  `chronos.rs` (761) — 3 519 LOC in total. Added `// Protects:` lines to
  every test in those seven files (all were either unprotected or had a doc
  block that did not use the word "protect"), doc comments for every
  previously undocumented `fn` (trait methods on `VaultProvider`, `FsVault`
  inherent methods, `BacklinkIndex`'s `new`/`from_json`/`to_json`/
  `is_built`/`note_count`/`link_count`/`entity_block_count`, `field()`,
  `Kind::as_str`, a few private helpers), and `mod tests`/helper-fn doc
  comments in each file. Promoted one `//`-only comment to a real `///` doc
  (`backlinks.rs`'s `token_bits`). Fixed comments found wrong while reading:
  none — every comment read matched its code as written. Not reached this
  batch: `template.rs` (1085), `links.rs` (1534) and `lib.rs` (2069) — about
  4 688 LOC, **next step: batch 26**, in that risk order (`template.rs`
  first — Obsidian template placeholder/Moment-token substitution; `links.rs`
  next — the link store; `lib.rs` last — the crate's own `VaultSession`
  orchestration and its test module `tests::…`, which this batch's `cargo
  test` output shows already carries roughly two dozen scenario tests of its
  own).
  Crate-wide `cartalith-vault` `census_v2.py`: items_undoc 203→118,
  tests_unprotected 129→58, consts_undoc 1→1 unchanged (the one remaining
  undocumented numeric const has not yet been reached — it may live in one
  of the three files left for batch 26; not yet verified which). loc
  7 621→7 909. **Comments only**: `cargo test -p cartalith-vault`,
  `CARGO_TARGET_DIR` at `.../scratchpad/bk25` — **129 passed / 0 failed / 0
  ignored**, matching the pre-batch total exactly (this batch did not record
  a pre-edit baseline run before starting, contrary to the brief; the
  post-edit run's 129 is cross-checked against `git diff`, which shows only
  comment insertions and one comment promotion, so no test could have been
  added or removed by this batch's edits). `git diff` after every file
  confirmed comment-only changes: the three `-` lines in the whole batch are
  `backlinks.rs`'s pre-existing `token_bits` `//` comment, removed and
  re-added one line up as `///`, its wording unchanged; no other line was
  removed; no added line carries new code; no code fences.

- **Ruling BK annotation pass, batch 26 (`cartalith-vault` closed) — built
  2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; vault 129/0/0).** Comments only, zero
  behaviour change. Recorded a baseline `cargo test -p cartalith-vault`
  (`CARGO_TARGET_DIR` at `.../scratchpad/bk26`) before any edit: **129
  passed / 0 failed / 0 ignored**, matching batch 25's claimed total.
  Closed the three files batch 25 left open — `template.rs` (1088 LOC after),
  `links.rs` (1521 LOC after) and `lib.rs` (1981 LOC after) — then found
  `census_v2.py` still flagged small residual gaps in the five files batch 25
  called closed and closed those too: `provider.rs` (`VaultError`'s own doc,
  `Display::fmt`, `From<io::Error>`, and every delegating method on `impl
  VaultProvider for FsVault`), `block.rs` (`BlockError`'s own doc,
  `Display::fmt`, `ends_with_blank_line`, `strip_one_break`,
  `unescape_attr`), `backlinks.rs` (one undocumented numeric const, `PAD` in
  `excerpt` — the one batch 25 could not locate — given a provenance comment
  as a labelled judgement call, not a citation; and a `mod tests;`
  declaration the census script's module-doc resolver cannot see past,
  because `backlinks.rs` is a file-module with a same-named submodule
  directory rather than a `mod.rs` — given its own `///` line rather than
  relying on the target file's `//!`), `chronos.rs` (`to_line`'s nested
  `clean` helper) and `markdown.rs` (`Display for SectionError`'s `fmt`).
  Added `///`/`// Protects:` docs to every previously undocumented `fn`,
  `impl` method, `enum`, `struct` and inline `mod tests` this batch touched;
  every test in `template.rs`, `links.rs` and `lib.rs` now carries a
  `Protects:`-worded line, either a new one or an existing doc block
  prefixed with the word (`preceding_comment_block_has_protects`/
  `test_fn_window_and_body_protects` both require the literal word, and most
  of `lib.rs`'s tests already had a real doc block that just never said
  "protects"). No comment was found to contradict its code; none were
  rewritten for correctness, only added.
  Crate-wide `cartalith-vault` `census_v2.py`, before this batch:
  items_undoc 118, tests_unprotected 58, consts_undoc 1, loc 7 909. After:
  **items_undoc 0, tests_unprotected 0, consts_undoc 0**, loc 8 180 — the
  crate is closed on the census. **Comments only**, confirmed per file with
  `git diff` after every edit (no removed line failed to reappear as the
  same code one line away; no added line carries new code; no fenced code
  blocks) and by the post-edit test run matching the recorded baseline
  exactly: `cargo test -p cartalith-vault`, same `CARGO_TARGET_DIR` —
  **129 passed / 0 failed / 0 ignored**, byte-identical to the pre-edit
  count. **Next step: batch 27** moves to `cartalith-urban` per
  `OUTSTANDING_WORK.md`'s BK row. Batches 6 through 26 all still need an
  independent verifier before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 27 (`cartalith-urban`, opened at its
  milestone-1-3 files) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; urban 349/0/0)
  (comments only; urban 349/0/0 unchanged).** Recorded a pre-edit baseline
  (`cargo test -p cartalith-urban`, `CARGO_TARGET_DIR` at
  `.../scratchpad/bk27`): **349 passed / 0 failed / 0 ignored**.
  `cartalith-urban` is 58 files, ~52 800 LOC (census_v2 376/349/63) — the
  crate's submodule-style files (`graph.rs` + `graph/tests.rs` +
  `graph/tests/golden.rs`, etc.) carry far more LOC in their paired test files
  than the top-level `.rs` file alone (`site.rs` is 827 LOC but its
  `tests/golden.rs` alone is 4 502), which changes the risk-order LOC budget
  from what a bare directory listing of `src/` suggests. Followed
  `URBAN_MORPHOLOGY_SCOPE.md`'s milestone order literally: milestone 1
  (`rng.rs`, `geom.rs` — the RNG substreams and geometry kernel), milestone 2
  (`graph.rs` + `graph/tests.rs` + `graph/tests/golden.rs` — the planar street
  graph), milestone 3 (`astar.rs` + `astar/tests.rs` +
  `astar/tests/golden.rs` — A* over the cost raster) — 8 files, 3 105 LOC,
  stopping before milestone 4 (`rules.rs`, 1 508 LOC with its own tests) to
  stay near the ~3 500 LOC band. Most items were already doc-commented from
  the port's own working discipline; real gaps closed: `geom.rs`'s
  `Add`/`Sub`/`Mul::{add,sub,mul}` trait methods, six of its test-fixture
  helpers (`pts`/`flat`/`square`/`cw_square`/`tri`/`l_shape`), `graph.rs`'s
  `Default::default`, a local `HalfEdge` struct, `graph/tests.rs`'s
  `scenario_ops`/`run`, `astar/tests.rs`'s `mk`/`run`. Six bare
  `mod tests;`/`mod golden;` declarations (`rng.rs` has none — its test module
  is inline) each got an explicit `///` line rather than relying on the
  census's module-doc resolver, which cannot see past a file-module with a
  same-named submodule directory — the same quirk batch 26 found in
  `cartalith-vault`'s `backlinks.rs`. Two local numeric consts
  (`graph.rs`'s `attach_point`'s `SNAP`/`ESNAP`) got trailing `//` provenance
  comments citing the reference line their doc-comment paragraph already
  named. One `//`-block promoted to `///` (`graph.rs`'s "Nine arguments..."
  rationale above `add_street`, wording unchanged). All 33 tests across the
  four files (`rng.rs` 5, `geom.rs` 13, `graph/tests.rs` 8, `astar/tests.rs`
  7) given a `// Protects:` first line, almost always alongside an existing
  real rationale comment rather than replacing it. No wrong existing comment
  was found while reading. `cargo test -p cartalith-urban`, same
  `CARGO_TARGET_DIR`, run again after every file: **349 passed / 0 failed / 0
  ignored**, byte-identical to the pre-edit baseline. `cargo build -p
  cartalith-urban` clean. `git diff` confirmed comment-only per file: every
  removed line reappears as the same code one line away (the SNAP/ESNAP pair
  gains a trailing comment; the "Nine arguments" block's `//` become `///`,
  text unchanged); no added line carries new code; no fenced code blocks.
  Crate-wide `census_v2.py`: items_undoc 376→322, tests_unprotected
  349→316, consts_undoc 63→61 (loc 52 831→52 957); a per-file re-check
  confirms all eight touched files are individually 0/0/0 on the census.
  **Next step: batch 28** continues `cartalith-urban` at milestone 4
  (`rules.rs` + `rules/tests.rs` + `rules/tests/golden.rs`) and milestone 5
  (`site.rs` + `site/tests.rs` + `site/tests/golden.rs`, ~6 019 LOC —
  likely its own batch given size alone), then milestones 6-17 in
  `URBAN_MORPHOLOGY_SCOPE.md`'s order, and finally the non-reference-port
  modules `citadel.rs`/`wallside.rs`/`courtyard.rs`, before moving to
  `cartalith-civ`, `cartalith-godot` and the GDScript shell. Batches 6
  through 27 all still need an independent verifier before any can be called
  done rather than pending.

- **Ruling BK annotation pass, batch 28 (`cartalith-urban`, milestones 4-7)
  — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked, additions only; urban 349/0/0) (comments only;
  urban 349/0/0 unchanged).** Recorded a pre-edit baseline
  (`cargo test -p cartalith-urban`, `CARGO_TARGET_DIR` at
  `.../scratchpad/bk28`): **349 passed / 0 failed / 0 ignored**. Followed
  `URBAN_MORPHOLOGY_SCOPE.md`'s milestone order: milestone 4 (`rules.rs` +
  `rules/tests.rs` + `rules/tests/golden.rs` — generation rules and culture
  profiles), milestone 5 (`site.rs` + `site/tests.rs` — the site model),
  milestone 6 (`routes.rs` + `routes/tests.rs` — anchors and primary
  routes), milestone 7 (`growth.rs` + `growth/tests.rs` +
  `growth/tests/golden.rs` — organic growth). `rules.rs` and `site.rs`
  themselves were already almost fully doc-commented from the port's own
  working discipline (2 and 3 census gaps respectively); the real gaps were
  concentrated in the paired `tests.rs` files, most as missing
  `// Protects:` lines on tests that already carried a real rationale
  comment above them (`rules/tests.rs` 13 tests, `site/tests.rs` 12,
  `routes/tests.rs` 6, `growth/tests.rs` 22). Six bare `mod tests;`/`mod
  golden;` declarations each got an explicit `///`/`//` line rather than
  relying on the census's module-doc resolver, which cannot see past a
  file-module with a same-named submodule directory (batches 26 and 27's
  quirk, again). Local numeric consts got provenance comments: `rules.rs`'s
  `assign_onto` macro method; `routes.rs`'s five `PROV_*` string constants
  (each needed its own directly preceding `//` line, not one shared block
  comment, since the census only looks at the single nearest line above);
  `growth.rs`'s `logistic_ramp`'s `K` and `estimate_carrying_capacity`'s
  `N`; `growth/tests/golden.rs`'s three `LOGISTIC_RAMP_BULK_*` constants
  (same one-line-per-const fix). One citation was corrected while writing a
  doc for `growth/tests.rs`'s `wall_state_from`: drafted citing a
  nonexistent `golden::Wall0Spec`, checked against the actual golden.rs and
  corrected to `golden::WallSpec` (`c.wall0: Option<&'static WallSpec>`)
  before landing. No other wrong existing comment was found while reading.
  `cargo test -p cartalith-urban`, same `CARGO_TARGET_DIR`, run again after
  all four milestones: **349 passed / 0 failed / 0 ignored**, byte-identical
  to the pre-edit baseline. `git diff --stat` confirmed comment-only across
  all ten touched files: 248 insertions, 0 deletions — every change is an
  added `///`/`//` line, no removed or altered code line anywhere; no fenced
  code blocks. Crate-wide `census_v2.py`: items_undoc 322→265,
  tests_unprotected 316→261, consts_undoc 61→50 (loc 52 957→53 205; 123
  gaps closed against a ~150-gap budget) — a per-file re-check confirms all
  files through milestone 7 are individually 0/0/0 on the census. **Stopped
  at the milestone 7/8 boundary** (LOC-read budget: growth's paired files
  alone are ~3 900 LOC, pushing the four milestones' combined read close to
  the 8 000 LOC cap). **Next step: batch 29** continues `cartalith-urban` at
  milestone 8 (`radial.rs` + `radial/tests.rs` + `radial/tests/golden.rs` —
  radial (Venus) streets and waterway) and milestone 8a (`plaza.rs` +
  `plaza/tests.rs` — the plaza); per-file gap counts at the time of this
  batch were `radial.rs` 1, `radial/tests.rs` 17, `radial/tests/golden.rs`
  2, `plaza.rs` 0, `plaza/tests.rs` 16 — re-check before relying on them, as
  this file itself warns. Batches 6 through 28 all still need an
  independent verifier before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 29 (`cartalith-urban`, milestones 8-8a
  plus part of 10) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked, additions only; urban 349/0/0; fortify CAP citation checked at v2.11 line 30386)
  (comments only; urban 349/0/0 unchanged).** Recorded a pre-edit baseline
  (`cargo test -p cartalith-urban`, `CARGO_TARGET_DIR` at
  `.../scratchpad/bk29`): **349 passed / 0 failed / 0 ignored**. Milestone 8
  (`radial.rs`/`radial/tests.rs`/`radial/tests/golden.rs`) and milestone 8a
  (`plaza.rs`/`plaza/tests.rs`) were already fully doc-commented in the main
  files; the gaps were `// Protects:` lines missing from 13 radial tests and
  13 plaza tests (most already carried a real rationale comment above them),
  a `mod tests;` line in each main file needing its own `///` (the census's
  module-doc resolver still cannot see past a file-module with a same-named
  submodule directory), two helper fns (`eq_bits`/`prov_count` in
  `radial/tests.rs`) and two `PROV_*` consts in `radial/tests/golden.rs`
  needing their own preceding line rather than the shared block comment
  above the first of the pair. Milestone 9 (`water.rs`/`water/tests.rs`)
  was likewise already fully commented in the main file; closed its `mod
  tests;` line, 12 `// Protects:` lines, three helper fns
  (`eq_f`/`eq_pt`/`eq_pts`), the `setup`/`apply_extra` builders, two golden
  entry fns and the `WM`/`HM` pair. Milestone 10's main file (`fortify.rs`)
  needed five numeric consts cited against the reference (`CAP` line 30386,
  `GAP_R` line 30513, `SB`/`FH` line 30620, `DITCH_W`/`COVERED_W`/`GLACIS_W`
  line 30640) and one trait-impl method doc
  (`FortificationBuilder::build_wall`, confirmed to delegate straight to the
  free fn of the same name). No wrong existing comment was found while
  reading. `cargo test -p cartalith-urban`, same `CARGO_TARGET_DIR`, run
  again after every file: **349 passed / 0 failed / 0 ignored**,
  byte-identical to the pre-edit baseline. `git diff --stat` confirmed
  comment-only across all eight touched files: 179 insertions, 0 deletions.
  Crate-wide `census_v2.py`: items_undoc 265→247, tests_unprotected
  261→223, consts_undoc 50→39 (67 gaps closed against the ~150-gap batch
  budget). **Stopped at `fortify.rs` done, `fortify/tests.rs` and
  `fortify/tests/golden.rs` not started** — `fortify/tests.rs` alone is
  1 954 LOC carrying 110 of the remaining gaps (49 unprotected tests, ~29
  undocumented helper fns, 2 consts) and `fortify/tests/golden.rs` another 6,
  and reading it in full would have pushed this batch past both the
  LOC-read and gap-count budgets at once; the 49 tests need their
  `// Protects:` lines written from their own bodies rather than
  templated, which milestone 10's remaining budget did not allow. **Next
  step: batch 30** finishes milestone 10 (`fortify/tests.rs` +
  `fortify/tests/golden.rs`), then continues in `URBAN_MORPHOLOGY_SCOPE.md`'s
  order: cleanup, blocks, districts, amenities, hinterland, generate, then
  the non-reference modules citadel, wallside, courtyard. Batches 6 through
  29 all still need an independent verifier before any can be called done
  rather than pending.

- **Ruling BK annotation pass, batch 30 (`cartalith-urban`, milestone 10
  finished, plus cleanup/blocks/amenities/hinterland/citadel/wallside/
  courtyard partial) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked, additions only; urban 349/0/0)
  (comments only; urban 349/0/0 unchanged).** Recorded a pre-edit baseline
  (`cargo test -p cartalith-urban`, `CARGO_TARGET_DIR` at
  `.../scratchpad/bk30`): **349 passed / 0 failed / 0 ignored**. Finished
  milestone 10: `fortify/tests.rs` (1 954 LOC, 110 gaps by the per-file
  census — 49 `// Protects:` lines written from each test's own body, 10
  undocumented helper fns (`eq_bits`, `check_poly`, `terrain_raster`,
  `water_ctx`, `terrain_ctx`, `check_fort`, `landlocked`, `extent`,
  `deflected`, `has_near`), a `mod golden;` needing its own `///` line (the
  census's module-doc resolver cannot see `fortify/tests/golden.rs` from
  `fortify/tests.rs`) and the `MW`/`MH` raster-dimension consts) and
  `fortify/tests/golden.rs` (6 gaps — five undocumented generated structs
  `BastionSpec`/`FortSpec`/`DensifyCase`/`NearestCase`/`CornerCase` and the
  `ACOS_HASH` const, which sits under a block comment that documents its
  sibling `ACOS_N` but not itself). Two self-inflicted duplicate-doc slips
  (`densify_input`, `rect_town` — both already carried a real `///` from an
  earlier pass, misread as gaps) were caught by `git diff`/re-read and
  reverted before landing. Then continued in `URBAN_MORPHOLOGY_SCOPE.md`'s
  order: `cleanup.rs`, `blocks.rs` and `amenities.rs` were already
  fully doc-commented in their own bodies — each closed only a `mod tests;`
  line needing its own `///` (same file-module/same-named-submodule-
  directory blind spot as batches 26-29). `wallside.rs` closed three
  undocumented fns (`Arc::new`, `Arc::len`, `bbox`) plus its own `mod
  tests;` line. `hinterland.rs` closed `Detail::point` plus its `mod
  tests;` line. `citadel.rs` closed `crosses_rect` plus its `mod tests;`
  line. `courtyard.rs` closed the `FRONTAGE_CLAMP` const, which sat under no
  comment of its own between two consts that each carry one (`FRONTAGE`
  above it, `MIN_LOT_AREA` below). `generate.rs` was checked and is already
  0/0/0. `districts.rs` was surveyed but **not touched**: its 23 remaining
  gaps are almost entirely (20 of 23) `PROV_*` string constants whose value
  already embeds a milestone/ruling citation in prose (e.g. `PROV_MARKET`
  cites M-NET-10 inline) but which the census still wants a preceding `//`
  line for, plus its own `mod tests;` line and two helper fns (`cand`,
  `retag`) — left for the next batch rather than rushed at the end of this
  one's budget. No wrong existing comment was found while reading; no
  behaviour changed. `cargo test -p cartalith-urban`, same
  `CARGO_TARGET_DIR`, run again after every file: **349 passed / 0 failed /
  0 ignored**, byte-identical to the pre-edit baseline. `git diff --stat`
  confirmed comment-only across all nine touched files: 180 insertions, 0
  deletions. Crate-wide `census_v2.py`: items_undoc 247→174,
  tests_unprotected 223→174, consts_undoc 39→35 (126 gaps closed against
  the ~150-gap batch budget; ~6 240 LOC read/edited across all files this
  batch touched or surveyed, against the ~8 000 LOC cap). **Stopped after
  `courtyard.rs`, with `districts.rs` surveyed but not started.** Next
  step: batch 31 opens `districts.rs` at its 23 remaining gaps (each
  `PROV_*` const needs its own one-line `// see <ruling/milestone>` or
  equivalent directly above it, per batch 29's precedent for `routes.rs`'s
  `PROV_*` set), then whatever of the milestone-defined module list
  (`URBAN_MORPHOLOGY_SCOPE.md`) remains after that within budget. Batches 6
  through 30 all still need an independent verifier before any can be
  called done rather than pending.

- **Ruling BK annotation pass, batch 31 (`cartalith-urban`, `districts.rs`
  closed; `citadel.rs`/`citadel/tests.rs`, `courtyard.rs`, `generate.rs`,
  `hinterland.rs`, three golden.rs files, `examples/town_json.rs`,
  `blocks/tests.rs`, `generate/tests.rs`, `districts/tests.rs` and
  `amenities/tests.rs` all closed) — built 2026-09-28, verified by the main loop 2026-09-28 (diff checked: comments plus one attribute reflowed token-identically; urban 349/0/0) (comments only; urban 349/0/0 unchanged).** Recorded a
  pre-edit baseline (`cargo test -p cartalith-urban`, `CARGO_TARGET_DIR` at
  `.../scratchpad/bk31`): **349 passed / 0 failed / 0 ignored**. Finished
  `districts.rs`'s 23 gaps left over from batch 30: 20 `PROV_*` consts each
  got a one-line `//` comment naming which branch writes it, `mod tests;`
  got its own `///` line, and the two helper fns `cand`/`retag` (already
  carrying a `//` block, not a `///` one, so the census's `preceded_by_doc`
  could not see it) were promoted to `///`. Then closed every other file
  `census_v2.py` still flagged in the crate except the three largest test
  files: `citadel.rs` (`mod tests;`), `citadel/tests.rs` (the `town` helper,
  three `// Protects:` lines, the `SEED` const), `courtyard.rs` (`mod
  tests;`), `generate.rs` (`mod tests;`), `hinterland.rs` (`mod tests;`),
  `blocks/tests/golden.rs`/`amenities/tests/golden.rs`/`districts/tests/
  golden.rs` (nine undocumented generated structs), `examples/town_json.rs`
  (three fns), `blocks/tests.rs` (a multi-line `#[allow(...)]` attribute
  collapsed to one line so the census's doc-scanner could see the `///`
  already above `grid`, two undocumented dump helpers, six `// Protects:`
  lines and two consts), `generate/tests.rs` (`mod golden;`, four helper
  fns, two consts, thirteen `// Protects:` lines — the whole-subsystem
  golden plus every unit-level pin), `districts/tests.rs` (eleven helper
  fns, one const, thirteen `// Protects:` lines) and `amenities/tests.rs`
  (seven helper/method fns, three consts, twenty `// Protects:` lines
  across markets/civic/games/log10). No wrong existing comment was found
  while reading; no behaviour changed. `cargo test -p cartalith-urban`, same
  `CARGO_TARGET_DIR`, run again after `blocks/tests.rs` and at the end:
  **349 passed / 0 failed / 0 ignored**, byte-identical to the pre-edit
  baseline. `git diff --stat` confirmed comment-only across all fifteen
  touched files: 316 insertions, 8 deletions (the 8 are the collapsed
  attribute line and two const/comment merges, each a comment-line
  reduction, not a code change). Crate-wide `census_v2.py`: items_undoc
  174→99, tests_unprotected 174→103, consts_undoc 35→5 — the crate stood at
  383 total gaps after batch 30 and stands at 207 now, so 176 gaps closed
  against the ~200-gap batch budget; roughly 6 900 LOC read/edited across
  the fourteen files this batch touched, against the ~10 000 LOC cap.
  **Stopped after `amenities/tests.rs`**, with three files
  left unclosed: `hinterland/tests.rs` (81 gaps), `cleanup/tests.rs` (70
  gaps) and `wallside/tests.rs` (56 gaps) — 207 gaps remaining, which would
  not fit this batch's budget alongside what was already done. Next step:
  batch 32 opens `hinterland/tests.rs` (largest remaining file) to close
  `cartalith-urban`, then moves to `cartalith-civ`, then `cartalith-godot`,
  then the GDScript shell (per the owner's routed order). Batches 6 through
  31 all still need an independent verifier before any can be called done
  rather than pending.

- **Ruling BK annotation pass, batch 32 (`cartalith-urban` closed) — built
  2026-09-28, verified by the main loop 2026-09-28 (diff checked code-identical; urban 349/0/0) (comments only; urban
  349/0/0 unchanged).** Recorded a pre-edit baseline (`cargo test -p
  cartalith-urban`, `CARGO_TARGET_DIR` at `.../scratchpad/bk32`): **349
  passed / 0 failed / 0 ignored**. Closed the three files batch 31 left:
  `hinterland/tests.rs` (81 gaps — 35 `// Protects:` lines, seven
  undocumented helper fns/structs (`eq_bits`, `Fx`, `fixture`,
  `detail_hash`, `kind_counts`, `idx_hash`, `anchors_at`), and four consts
  (`HM`, `YOFF`, `JIT`, a test-local `MAX_SEG`) that sat under a doc meant
  for a sibling const rather than their own), `cleanup/tests.rs` (70 gaps —
  42 `// Protects:` lines and 13 undocumented helper fns (`grid`, `rect`,
  `alive`, `scenario`, `lane_added`, `water_scenario`, `priv_scenario`,
  `priv_big`, `ring_poly`, `fort_fixture`, `fort_golden`, `dry_site`,
  `lane_scenario`)) and `wallside/tests.rs` (56 gaps — 25 `// Protects:`
  lines, 10 undocumented helper fns (`landlocked`, `ctx`, `q`, `faub`,
  `inside`, plus five already-covered by context) and one const (`BIG`)).
  Every `census_v2.py` gap in the crate was confirmed closed with a
  per-file detail script (built on `census_v2`'s own `scan_rust_file`
  helpers) before moving on, rather than assumed from the count alone. No
  wrong existing comment was found while reading; no behaviour changed.
  `cargo test -p cartalith-urban`, same `CARGO_TARGET_DIR`, run again at
  the end: **349 passed / 0 failed / 0 ignored**, byte-identical to the
  pre-edit baseline. `git diff --stat` confirmed comment-only across all
  three touched files: 296 insertions net (`hinterland/tests.rs` had 7
  lines replaced by 9 comment+doc lines that carry the same code, `cleanup/
  tests.rs` and `wallside/tests.rs` are pure additions with zero
  deletions). Crate-wide `census_v2.py`: items_undoc 99→0, tests_unprotected
  103→0, consts_undoc 5→0 — **cartalith-urban now reads 0/0/0**, closing
  the 207 gaps left after batch 31 within its ~250-gap budget; items_total,
  tests_total and consts_with_numeric (942/349/143) are unchanged before
  and after, confirming no item was added or removed by the comment pass.
  Roughly 4 300 LOC read/edited across the three files, against the
  ~6 000 LOC cap. **`cartalith-urban` is closed for Ruling BK.** Next step:
  batch 33 opens `cartalith-civ` (~70 000 LOC, 1 076/1 023/162 at census
  time when last measured) — per `OUTSTANDING_WORK.md`'s routed order, file
  order by risk: golden/fixture files first (largest gap density, lowest
  risk of a wrong comment since the golden values pin exact behaviour),
  then the crate's core logic files, then its own `tests.rs` trees last
  (largest raw gap count but most mechanical — mostly `// Protects:` lines
  once the core is understood). Batches 6 through 32 all still need an
  independent verifier before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 33 (`cartalith-civ` started, verified by the main loop 2026-09-28 (diff checked, additions only; civ 1013/0/11)) — built 2026-09-28 (comments only; cartalith-civ
  1013/0/11 unchanged).** Recorded a pre-edit baseline (`cargo test -p
  cartalith-civ`, `CARGO_TARGET_DIR` at `.../scratchpad/bk33`): **1013 passed
  / 0 failed / 11 ignored**, summed over 36 binaries. Wrote the crate's
  per-file gap table (`find_gaps.py` against `cartalith-civ`) and a
  risk-ordered file plan into `OUTSTANDING_WORK.md`'s BK row: golden-parity
  and engine-logic files first, `lib.rs` (657 gaps, 24 584 LOC) and
  `landmark.rs` (169 gaps, 8 328 LOC) deferred whole to a later batch since
  either alone would consume the batch budget, `PHASE2_SCOPE.md`'s military/
  manpower/relations/naming cluster taken first, then `ECONOMY_SCOPE.md`'s
  currency. Closed, in order: `currency.rs` (9 gaps), `military.rs` (18) —
  its module doc already covered every item so only test `// Protects:`
  lines and two helper-fn docs were needed, `garrison.rs` (18), `conflict.rs`
  (26), `campaign.rs` (20, plus naming two previously-undocumented test
  consts `GW`/`GH`), `roster.rs` (16) + `tests/golden_parity_roster.rs` (3),
  `relations.rs` (23), `naming.rs` (22) + `tests/golden_parity_settlement_
  naming.rs` (5). No wrong existing comment was found while reading; no
  behaviour changed. `cargo test -p cartalith-civ`, same `CARGO_TARGET_DIR`,
  run again at the end: **1013 passed / 0 failed / 11 ignored**,
  byte-identical to the pre-edit baseline. `git diff` on every touched file
  confirmed additions only (zero deletions) throughout. Crate-wide
  `census_v2.py`: items_undoc 1076→988, tests_unprotected 1023→953,
  consts_undoc 162→160 — 160 gaps closed against the ~200-gap batch budget
  (`find_gaps.py`'s raw count: 2261→2101); roughly 3 800 LOC read/edited
  across the nine files this batch touched, against the ~10 000 LOC cap.
  **Stopped after `tests/golden_parity_settlement_naming.rs`**, comfortably
  inside budget rather than pushing to the limit. Next step: batch 34
  continues `cartalith-civ` down the risk-ordered plan — `journey_progress.rs`
  (`JOURNEY_PLANNER_SCOPE.md`, 23 gaps), `wildlife.rs` (22), then the larger
  files (`urban_adapter.rs`/`urban_adapter/tests.rs`, `manpower.rs`,
  `belief.rs`, `trade.rs`/`trade/tests.rs`, `tools.rs`, `travel_library.rs`,
  `labels.rs`, `timeline.rs` per `TIMELINE_SCOPE.md`) before finally
  `landmark.rs` and `lib.rs`, each as its own dedicated batch or split by
  risk-ordered section given their size. Batches 6 through 33 all still need
  an independent verifier before any can be called done rather than pending.

- **Ruling BK annotation pass, batch 34 (`cartalith-civ` continued) — built
  2026-09-28, comments only, verified by the main loop 2026-09-28 (diff checked code-identical; civ 1013/0/11).** Recorded a
  pre-edit baseline (`cargo test -p cartalith-civ`, `CARGO_TARGET_DIR` at
  `.../scratchpad/bk34`): **1013 passed / 0 failed / 11 ignored**, summed
  over 36 binaries — matches the STATUS.md figure batch 33 left. Closed, in
  order: `journey_progress.rs` (23 gaps), `wildlife.rs` (22), `urban_adapter.rs`
  (2 -- the `idx` helper and a `mod tests;` declaration the census script
  cannot resolve to its actual `urban_adapter/tests.rs` target, given its own
  preceding doc line instead so the census reads it correctly), `manpower.rs`
  (36 -- 29 `// Protects:` lines plus 6 item docs plus the `mod tests` block
  doc, mostly folding "Protects:" into existing rich prose doc comments
  rather than duplicating them), `belief.rs` (43 -- two item docs, one
  provenance comment on a test-local `NEVER` sentinel constant, two module
  docs for `mod tests`/`mod diffusion_tests`, and 31 `// Protects:`/`///
  Protects:` lines, several folded into existing prose the same way). No
  wrong existing comment was found while reading; no behaviour changed.
  `cargo test -p cartalith-civ`, same `CARGO_TARGET_DIR`, run again at the
  end: **1013 passed / 0 failed / 11 ignored**, byte-identical to the
  pre-edit baseline. `git diff` on every touched file confirmed the checker's
  rule (every removed line reappears as the same code, no added line carries
  new code) -- `grep`-verified directly for `manpower.rs` and `belief.rs`
  (no non-comment/non-attribute line in the diff) and visually for the other
  two. Crate-wide `census_v2.py`: items_undoc 988→940, tests_unprotected
  953→876, consts_undoc 160→159 -- 126 gaps closed, under the ~250-gap batch
  budget; roughly 7 700 LOC read/edited across the five files this batch
  touched (journey_progress.rs, wildlife.rs, urban_adapter.rs, manpower.rs,
  belief.rs), under the ~10 000 LOC cap. **Stopped after `belief.rs`**,
  deliberately short of both budgets rather than pushing into `trade.rs` with
  little margin left; the `urban_adapter/tests.rs` and `trade/tests.rs`
  companion test files the batch's own plan named were not reached and stay
  open. Next step: batch 35 continues `cartalith-civ` down the risk-ordered
  plan -- `trade.rs` + `trade/tests.rs` (13 + 103 gaps, `trade/tests.rs` is a
  separate module file the same shape as `urban_adapter/tests.rs`), then
  `urban_adapter/tests.rs` (107 gaps, left from batch 34's step), `tools.rs`,
  `travel_library.rs`, `labels.rs`, `timeline.rs` per `TIMELINE_SCOPE.md`,
  before finally `landmark.rs` and `lib.rs`, each as its own dedicated batch
  or split by risk-ordered section given their size. Batches 6 through 34
  all still need an independent verifier before any can be called done
  rather than pending.

- **"Settlements draw their own water on the map" (owner report) — built,
  verified by the main loop 2026-09-28 (before/after overlay screenshots opened: full-view water fill gone; diff limited to the draw_water gate).** `map_overlay.gd`'s only main-map call to
  `UrbanLayoutDraw.draw_layout()` now passes a new `draw_water = false`
  argument; `draw_layout()` gates its `water_poly` fill, its coastal-site
  river-line fallback, and `_draw_water_mask()`'s `water_mask_runs` clip
  behind that flag (default `true`, so the City Viewer, the right-dock
  thumbnail and every probe caller are unaffected). Reasoning: the main map's
  own base raster already shows this same water, at the map's real
  hydrology, so the layout's second, unconnected fill only duplicated it — the
  owner's own "either not needed or below sea level" framing. Whether a
  settlement's water should instead connect to the map's real rivers is
  explicitly deferred, per the owner's own report, and untouched here.
  Verified with a new windowed probe, `_settlewater_probe.tscn` (a coastal
  settlement, seed 24601): the pre-fix path (`draw_water = true`, called
  directly) painted 6 561 px inside a 40 px box around the town; the real
  main-map path (`draw_water = false`) painted 17 px there (settlement
  chrome only, e.g. the pin) — before/after screenshots
  `_settlewater_before.png` / `_settlewater_after.png`. `_mapdata_probe`
  re-run windowed, unrelated: 43/43 PASS.

### 2026-09-27

- **Two `OUTSTANDING_WORK.md` rows closed, verified by the main loop 2026-09-27 (`_panemin` fail=0 at 500x1080 touch re-run; `_exportpanes_gis.png` inspected).**
  **(1) "Two handset route panes still overflow — and it is the BODY, not the
  footer" (re-routed to `DataManagerWindow`).** Re-measured before touching
  anything (`_panemin_probe.gd --route export_maps/export_gis --verbose
  --force-touch --vp 500x1080`): `export_maps`' Scheme row (`_row()`, Tiles
  column) demands 386 px against 376 px of room, driven by its 120 px fixed
  label plus a 256 px segment group; `export_gis`' EXTENT row (`_pattern_row()`)
  demands 476 px, driven by a 316 px unwrapped hint label beside a 74 px fixed
  label — both bodies, confirmed against the window's own client rect, not the
  footer (last batch's footer wrap). Fix, on phone only: `_row()` and
  `_pattern_row()` (`data_manager_window.gd`) stack their label above the
  control instead of beside it at a fixed width, the same trade
  `_build_tile_export_pane()`'s two-column collapse already makes at this
  density; the EXTENT hint label gets `autowrap_mode` + `SIZE_EXPAND_FILL`
  (mirroring `_pattern_heading()`'s existing purpose-line idiom — a wrap
  without the expand flag was tried and rejected by an earlier pass, per
  `MISTAKES.md`'s clip_text-collapse trap). Ruling AZ (design from DCC
  vocabulary, screenshots for approval) — both panes screenshotted at
  1080×2340 (`_exportpanes_probe.gd`) and opened; no overflow, every control
  reachable, `INCLUDE` chips and the EXTENT hint wrap cleanly. `_panemin_probe.gd`
  extended with a per-control `_check_controls_in_bounds()` leg (every visible
  control's global rect against both the window body and the app viewport);
  run before the fix it failed as expected (`export_gis`: 30/53 controls
  outside the window body, worst 82 px over; `export_maps`' aggregate
  `contents_min` also over at 430 vs 412, though its per-control leg happened
  to pass — the 18 px gap there never reached a real rendered control). After
  the fix: `panemin` `fail=0 skipped=33` at both `--vp 500x1080` and `--vp
  1080x2340` (both `--force-touch`), and `fail=0 skipped=15` unchanged at
  desktop `--vp 1152x648`. **(2) "Developer code names are shown to users",
  the 4 remaining tooltip hits** the 2026-09-27 tooltip sweep (below) could
  not reach because this file set was held by the per-style rivers lane, now
  released: `territory_influence()`/Dijkstra and `LabelTypography`/
  `set_field()`/`map_overlay.gd::_label_font_for()` in
  `cartography_workspace.gd`; `water_anim_layer.gd` in `render_workspace.gd`;
  `cartalith_engine::geojson` in `data_manager_window.gd`. Each rewritten in
  plain words with its provenance kept as a `##` comment above the line (the
  `ad883b5` pattern) — no logic/signature changes. `_codenames_probe.gd`
  re-run windowed: the tooltip pass now flags exactly one remaining distinct
  hit, `data_manager_window.gd`'s `val_check` route ("There is a warning
  collection…", naming `project_open()`) — not one of the four this batch was
  scoped to, still inside a file the probe's own `SKIPPED_SOURCES` list
  carries, left as the next tooltip-sweep row's finding rather than folded in
  here. The visible-label pass is unchanged at its one known remaining hit
  (`world_workspace.gd`'s `cartalith-spatial` note, out of scope — belongs to
  the other lane, per this batch's brief). Regression: `_ctxphone_probe -- --vp
  1080x2340 --force-touch` 56/56, `_ctxring_probe` 132/132, both unchanged.
  `--check-only --script` clean on `data_manager_window.gd`,
  `workspaces/cartography_workspace.gd`, `workspaces/render_workspace.gd` and
  `_panemin_probe.gd`. **Not yet independently re-verified by the main loop or
  a second session** — mark this row resolved only after that pass.

- **`OUTSTANDING_WORK.md`'s "Developer code names are shown to users" row,
  tooltip sweep — done\*, verified by the main loop 2026-09-27 (`--check-only` on touched files, format-placeholder counts compared per file, the right_dock diff read).** Owner Ruling AQ
  (`LARGE_ITEM_RULINGS.md`, 2026-09-24): rewrite every developer code name
  (`.gd`/`.rs` file, `snake_case()` call, `cartalith_*`/`cartalith-*` crate)
  shown to a user. The visible-label half closed 2026-09-24 (`3a12d64`); this
  batch is the tooltip half. Re-counted with a reproducible script
  (`grep`-based scan of string literals containing `.gd`/`.rs`/`::`/
  `cartalith_`/`cartalith-`/snake_case( across the 32 `shell/*.gd` files that
  mention "tooltip", excluding the five files owned by the concurrent
  per-style-rivers lane) rather than trusting the row's own "~795" figure:
  201 candidate string literals before, of which roughly 110 were genuine
  `tooltip_text` sinks (direct assignment, `DccWidgets` helper tooltip
  parameters, `set_item_tooltip`/`_todo` on menus, or a `"tip"`/`"why"` dict
  key consumed by one) across `right_dock.gd`, `menus.gd`, `phone_menu.gd`,
  `place_editor_window.gd`, `asset_library_window.gd`, `app.gd`,
  `dcc_shell.gd`, `layers_popover.gd`, `tool_bar.gd`, `journey_planner_view.gd`,
  `new_world_dialog.gd`, `shortcuts_dialog.gd`,
  `workspaces/{civilization,infrastructure,world}_workspace.gd`; the rest were
  `DccWidgets.note`/`modal_foot`/`DccShell.set_status` (visible text, not a
  tooltip) or `res://`/`user://` load paths (not user-facing at all), left
  untouched. Each genuine tooltip keeps its code provenance as a `##` comment
  above the line and is rewritten in plain user language; `%s`/`%d`
  placeholders and GDScript escaping preserved; no logic/signature changes.
  `_codenames_probe.gd` extended with a second, windowed pass that walks
  `tooltip_text` on every live Control (not just visible ones) plus every
  `PopupMenu` item's tooltip, across WORLD/CIVIL/CARTO's default panels, the
  menu bar and the Shortcuts dialog. Re-run after the sweep: the visible-label
  pass shows 1 remaining hit (`workspaces/world_workspace.gd`'s "LOD terrain
  data" `DccWidgets.note`, naming `cartalith-spatial` — a visible Label, not a
  tooltip, so out of this batch's own brief; left for the next visible-text
  pass). The tooltip pass shows 4 distinct remaining hits, all traced by hand
  to the five files this batch could not touch (`workspaces/
  cartography_workspace.gd` ×2, `workspaces/render_workspace.gd`,
  `data_manager_window.gd`) — `engine_bridge.gd` and `viewport_host.gd` carry
  no remaining hits. `--check-only` clean on all 15 touched `.gd` files plus
  `shell/app.gd`; `_ctxring_probe` 132/132 and `_ctxphone_probe -- --vp
  1080x2340 --force-touch` 56/56, both re-run after the sweep, no regression.
  Marked verified by the main loop 2026-09-27 (`--check-only` on touched files, format-placeholder counts compared per file, the right_dock diff read) because this batch was built by
  four parallel agents against a shared classification method the main loop
  wrote but did not itself re-derive line-by-line for every one of the ~110
  edits — the counts and skip list above are the main loop's own re-run, the
  individual rewrites are not.

- **Owner Ruling BF: vault templates and Markdown follow Obsidian exactly. Built
  2026-09-27, verified by the main loop 2026-09-27 (cartalith-vault 129/0, `_mdedit` 58/0, `_vaultbf` 28/0, `_vaultbe` 50/0 re-run; preview screenshot inspected).** `cartalith-vault`
  `template.rs`: `Config::read` takes the template folder from
  `.obsidian/templates.json` (`folder`), else Templater's
  `.obsidian/plugins/templater-obsidian/data.json` (`templates_folder`), else
  the old "path contains *template*" rule (`Source::Fallback`); absent,
  malformed or blank JSON is "not configured". `discover(files, &cfg)` offers
  only the folder's files. `fill_title` became `template::fill`: Obsidian's
  `{{title}}` (the note's basename), `{{date}}`/`{{time}}` (today, in the
  vault's `dateFormat`/`timeFormat`, else `YYYY-MM-DD`/`HH:mm`) and
  `{{date:FORMAT}}`/`{{time:FORMAT}}` through `format_moment` (Moment's
  tokenizer order, English locale incl. `L`/`LL`/`LT`…), plus the owner's
  `{{…Name}}`/`[Name]`; Templater `<% %>` is copied verbatim. The clock is a
  seam (`template::DateTime`); the bridge passes Godot's local time.
  `template::insert` is Obsidian's Insert template: body at the caret,
  properties merged (lists unioned, scalars take the template's value, new keys
  appended). Bridge: `vault_template_source`, `vault_insert_template`; the
  vault window's create section says where templates came from. Editor
  (`markdown_editor.gd`): Highlight `==…==`, Task `- [ ] `, Callout
  `> [!note]`, Tag `#`, Insert template (picker), Ctrl+E Write/Preview;
  Preview renders highlight, `[[note|alias]]`, `[[note#heading]]` (as "note >
  heading"), `![[embed]]` (a labelled placeholder, not the embed), `#tags`,
  tasks and callouts (one colour for every type). Also fixed: both editor
  pickers opened offset by the vault window's position (`anchor_rect`).
  Evidence: `cartalith-vault` 129 unit tests; mutation 20/20 Rust, 10/10
  `_mdedit`, 1/1 picker anchor killed; `_mdedit` 58/0; new windowed
  `_vaultbf_probe` 28/0 desktop, 26/0 at 1080x2340 touch; `_vaultbe` 50/0 and
  49/0; the other `_vault*` probes pass; `cargo test --workspace` 3998 passed,
  0 failed.

- **Owner Ruling BE: one vault window, with a Markdown editor. Built; verified by the
  main loop (`_mdedit` 37/0, `_vaultbe` 50/0, `_vault` passes; screenshots inspected and
  sent to the owner).** Once a vault is bound, `vault_window.gd`'s
  `open_browse()` and `open_for(kind, id, label)` draw the same layout
  (`_build_browse`). On the left are the tree and its search: typing filters
  by name, Enter searches inside notes, and attached notes carry an accent-dot
  marker. On the right comes **Attached to** (`_build_attached_to`). It lists
  every link on the note with the entity's name, kind and faction, plus Show on
  map, Open place editor and Detach. A frontmatter `type`/`tid` that resolves
  but is not linked is shown as "by frontmatter, not attached", with an Attach
  button. There is an "Attach to…" picker for any settlement, province,
  continent or faction. Below that, the note is shown as chips,
  outline/excerpt and the rendered Markdown. **Open to edit** turns the same
  pane into `shell/markdown_editor.gd`. Its toolbar has B/I/U(`<u>`)/S,
  Normal/H1/H2/H3, lists, quote, code, a code block and a `[[…]]` picker. It
  also has Ctrl+B/I/U, Write/Preview, and Save through `vault_write_file`'s
  hash guard, which refuses a file changed on disk. Closing with unsaved edits
  asks first. The old form's advanced sections now sit in a collapsed
  **More**: working copy and section insert, map snapshot, the Cartalith block
  and its removal, field fill, write confirmations, index and Disconnect. All
  five guarded writes are still reachable and still stop at their preview.
  `open_for` preselects the entity's first note, or shows its attach flow.
  Narrower than 640 px, the window folds to one pane.
  - **Evidence:** `_mdedit_probe.gd` (headless, 37 exact-string checks) went
    red on 3 of 3 transform mutants and was restored byte-identical.
    `_vaultbe_probe.tscn` (windowed) ran 50 checks at 1600x1000 and 49 at
    1080x2340 `--force-touch`, all green. All 14 existing vault probes are
    green. `_vaulttree_probe` had been red at HEAD before this change.
  - **Not yet re-run by a separate verifying pass.**
- **Ruling BH scoped: `GEOLOGY_FIRST_SCOPE.md` written. Nothing in it is
  built, and GF-0 to GF-7 are all unstarted.** It is a documentation change
  only, and no code moved. Checked at the symbols, it records three facts:
  - `resistance_field` gives stream power a `K` factor of 0.65–1.0. That is
    at most 1.36× on continental land, derived by arithmetic from
    `compute_resistance` and `stream_power_kernel_bounded`.
  - `build_lithology` classifies sediment from the post-erosion surface and
    present rainfall, so no shaping process could read it causally.
  - `EROSION_GEOLOGICAL_TIME_SCOPE.md` has no clock for the ruling to run on.

  *Superseded the same day by the two entries below*: the milestones are now
  GF-0 to GF-9, and GF-0 is built.
- **`GEOLOGY_FIRST_SCOPE.md` amended for Ruling BJ.** It is a documentation
  change. The milestones are renumbered:
  - GF-0 to GF-6 are unchanged;
  - **GF-7 is the geological clock** (§4.12): one dimensionless parameter,
    `geological age ×1.00`, that scales pass and iteration counts. It is
    linear for stream power, the threshold hillslope and karst; saturating
    for coastal and glacial; and logarithmic for a new weathered mantle;
  - **GF-8 is lithology painting** (§4.13): a Rock target in Biome paint,
    baked commit, a field-and-column undo step, and a differential re-erosion
    through `tile_erode`;
  - **GF-9 is the re-baseline and scrub**, formerly GF-7.

  The save format is specified as a separate `geology` set (§2.6). GF-4 turns
  the coastal pass on in the app. §9 marks Q1, Q2, Q4 and Q7 as answered, and
  adds Q11. **GF-1 to GF-9 are unstarted.**
- **GF-0 built: the geology-first measurement harness**
  (`crates/cartalith-godot/tests/gf0_geology_harness.rs`). Verified by the main loop 2026-09-27 (controls 3/0; `gf0_bars` re-run in release, 800 km values match to 4 dp); was pending independent
  verification. It changes no generated output: it is a test target, plus a
  test-only `cartalith-erosion` dev-dependency of `cartalith-godot`.
  - **The fast controls** run in every workspace test:
    `positive_control_b1_b2_b3_see_a_built_in_effect`,
    `negative_control_a_permuted_rock_map_reads_no_effect` and
    `metric_helpers_hold_on_hand_checked_inputs`.
  - **The measurement** is `--ignored`: `gf0_bars` (15 worlds, 168 s in a
    release build) and `gf0_b9_cost`. The commands are in the scope's §5.4.
  - **It is arm 1 only**, with `resistance_field` quartiles standing in for
    rock strength, and labelled as a stand-in. Arm 2 needs GF-1's column.
  - **Baseline at 800 km, five seeds:**
    - B1 ρ −0.49 to −0.33;
    - B2 0.14 to 0.43;
    - B3 0.52 to 1.40, and undefined on 2 seeds, where weak-rock mean depth is
      net raising;
    - B6 1.32 to 6.58;
    - B7 0.13 to 2.10, on an extra arm with coastal on;
    - B8: 0 ocean cells on river paths on all 15 worlds;
    - B9: `generate_terrain` median 3.115 s (3.089..3.124), with an
      independent re-run of 3.095 s (3.068..3.140);
    - B10: deterministic on all five seeds.
  - **Not measurable yet:** B4 and B5, and B10's rock-type counts, each with
    its reason.
  - **Negative control:** |B1| ≤ 0.001 on all 15 worlds.
  - **Mutation testing:** 3 of 3 metric mutants turned the fast controls red:
    the strong/weak split, the permutation and the ranking. The source was
    restored hash-identical afterwards.
  - **Findings recorded in §5.4.**
    - A rock-blind pipeline reads far from 1 on B6 and B7, so both bars now
      also require a multiple of the measured control.
    - Above 40°, land is 0.13–0.37 % at 800 km, 14–18 % at 80 km, and 0 % at
      8 000 km.
  - **Workspace suite:** 4 004 passed before the change, and 4 007 after it,
    the three new controls. 0 failed and 42 ignored before; 44 ignored after,
    the two measurements. 179 result lines before, 180 after.

  Next: GF-1, the lithology model and column.
- **GF-1 built: the lithology model and column, read by no process.
  verified by the main loop 2026-09-27 (geology_gf1 5/0, terrain geology 13/0, gf0 controls 3/0, params_mapping 34/0, sample 39/0 re-run; staged tree built alone in a worktree).**
  - **What:** `cartalith-terrain/src/geology.rs` holds the 11-type rock table
    (`ROCK_PROPS`) and the derivation (`build_geology`) from pre-erosion causes
    only: plate crust, a labelled boundary distance (wraps in x), structural
    lows, latitude, facies noise and the volcanic setting. The setting is no
    longer discarded: `stamp_volcanoes_*_traced` record the winning edifice per
    cell. `generate_terrain` runs the new geology stage after sea level and
    stores `WorldState::geology` (the column, or `GeologyAbsent` with a reason
    for imports, restored saves and the switch off). The switch,
    `WorldParams::geology_model`, is off in `WorldParams::defaults` and on in
    `params::defaults()`; it has a `PARAMS` and `JS_PATHS` row, no GUI control,
    and a save without the key reloads with it off. The Sample dock gains Rock
    (surface), Beneath (contact depth in metres), Strength, Soluble ·
    permeable and Regolith rows, each dashed with the engine's reason when
    there is no value.
  - **Bit-identity:** `geology_gf1.rs` asserts every pre-existing array
    bit-identical with the switch on and off (4 worlds, plus the simple
    volcanism path), and that the column does not move when erosion or
    climate does. `gf0_bars`' §5.4 lines are byte-identical before and after
    on all 15 worlds. `golden_parity_pipeline.rs` is untouched.
  - **First values (scope §5.5):** B10 passes on all five seeds at 800 km,
    9–10 rock types and a two-layer share of 0.14–0.28 of land. B4's control
    arm reads 0.035–0.243 at 800 km, and its input-selected twin cannot be
    measured with this derivation. The control arm already reads B3 at
    7.9–108, above the treatment bar.
  - **Sources:** Hoek–Brown `mi` verified in Marinos & Hoek (2000) Table 2,
    with two corrections to the scope. The Selby bands, the Freeze & Cherry
    ordering and the angle of repose are cited, not verified. Every other
    value is a labelled judgement.
  - **Mutation testing:** 191 mutants over the table and the classifier; 185
    were killed at first, and the 6 survivors were killed after new
    assertions. Sources were restored hash-identical.
  - **Memory:** the generation peak at 2048 × 1311 rises from 486.50 to
    513.94 MiB (+27.44 MiB). `WorldState` grows by 11.0 B/cell.
  - **Not built, against the scope's GF-1 list:** the §2.6 save set (moved to
    GF-8), the single-layer import column, and the lithology map view.
  - **Probe:** `_gf1sample_probe.tscn` (windowed): 27 checks green, with a
    screenshot.
- **GF-2 mechanism built, gated off in the app pending bars — pending
  independent verification.** Stream power and rebound read rock and
  deposition becomes regolith, behind a new switch,
  `WorldParams::geology_processes`, which is **off in `WorldParams::defaults()`
  and in `params::defaults()`** (coordinator decision 2026-09-27: B8's
  small-lake rise is the regression RV-1 fixed, and B1/B2 show no gain). The
  app keeps GF-1's inert column; its default world hashes bit-identical to
  HEAD `2cf0143` on two seeds at 2048 × 1311. GF-3 and GF-7 re-measure B1, B2
  and B8 with the switch on before it can ship. See
  `GEOLOGY_FIRST_SCOPE.md` §5.6.
  - **What:** `cartalith_erosion::stream_power_kernel_rock` (κ of the exposed
    rock to the power `c = tect.resist`, re-read every iteration, which is the
    contact switch), `lift_column` (rebound lifts the contact) and
    `account_regolith` (net gains become regolith, lowering strips it first,
    applied by the caller as §4.9 specifies). `generate_terrain` routes every
    stream-power call, rebound, the sediment routing and the glacial and carve
    lowering through one gate that needs both `geology_model` and
    `geology_processes`. The switch has a `PARAMS`/`JS_PATHS` row and reloads
    off from a save without the key. The harness and the GF-2 tests turn it
    on explicitly; the control arm is the app's own world.
  - **Bars at 800 km, five seeds, switch on (control → treatment):** B3 passes, full
    (2.02–2.80 × control) and on the stream-power call alone (1.97–2.17 ×).
    B9 passes (ratio of medians 1.038 and 1.014 on two runs; brackets overlap,
    so no difference is established). B10 passes. **B1 and B2 fail on all five**
    (Δρ −0.009 to +0.002; B2 moves under 1.3 %): the treatment changes heights
    by metres (1st–99th percentile −10 to +15 m), mostly along channels, which
    no relief statistic at 390 m cells sees. **B8 fails on 2 of 5**: 1–3-cell
    lakes 1.28× and 1.26× against a 1.25× bar; lake share and ocean-on-path
    hold everywhere, and B8 passes on all ten 80 km and 8 000 km worlds.
  - **Design disclosed:** a first build wrote regolith inside the kernel's
    iterations and doubled the small-lake count; it was replaced by the
    scope's own caller-side rule, with no constant tuned.
  - **Goldens:** none moved and none re-recorded; app worlds do not move
    either. `pre_bh_world` (§6.2) is a GF-9 action and is not built.
  - **Tests:** `cartalith-erosion/tests/gf2_rock_stream_power.rs` (7, literal
    values) and `cartalith-engine/tests/geology_gf2.rs` (7, including exact
    replays of the light pass, the carve, the glacial strip and the sediment
    routing), plus a `params_mapping` test for the switch's save rule. 21
    mutants over the new code and the switch, all killed (three after replay
    tests were added for first-round survivors).
  - **Probe:** `_gf2relief_shot.tscn` (windowed), before and after at 800 km on
    seeds 483920 and 314159, taken with the switch on (before the gate). The
    broad relief does not visibly change; the differences are in which
    channel gullies are drawn and how deep.
- **GF-3 built: the threshold hillslope stage, behind the same gate and still
  off in the app — verified by the main loop 2026-09-27 (gf3_threshold_hillslope 11/0, golden_parity_thermal 2/0, geology_gf2 7/0, gf0 controls 4/0 re-run; the staged tree built alone in a worktree).** It is
  `cartalith_erosion::threshold_hillslope`: `erode_thermal`'s rule with a
  per-cell threshold, `tan(θc)·cell_m·(1 − sea)/peak_m`, taken from the exposed
  rock and re-read every pass. It runs 8 passes (`THRESHOLD_HILLSLOPE_PASSES`)
  once in the light pass, after stream power and rebound and before the
  trace. The net change becomes regolith. It runs only with `geology_model`
  **and** `geology_processes` on, and `geology_processes` stays off in
  `params::defaults()`. The app-default and parity worlds hash bit-identical
  to HEAD `22ec647`: 48 of 48 arrays, two seeds. `golden_parity_thermal.rs`
  is untouched. See `GEOLOGY_FIRST_SCOPE.md` §5.7.
  - **Bars with the switch on (800 km, five seeds):**
    - **B8 now passes on all 15 worlds.** GF-2's two failures, 1.28× and
      1.26×, are now 1.09× and 1.11×.
    - **B1, B2 and B4 fail on every seed.** B1 and B2 read the same to four
      decimals at 0, 8, 16 and 32 passes, because the stage only lowers
      slopes above `θc`, and at 800 km almost none are.
    - B3 and B10 pass.
    - **B9 passes with a real cost:** ratio of medians 1.106 and 1.107, with
      non-overlapping brackets. 16 passes read 1.202 and 1.132, which is why
      the count is 8.
    - Screenshots with processes on: 671 and 725 pixels change by more than 8
      levels. Relief does not visibly track rock more than under GF-2.
  - **Tests:** `cartalith-erosion/tests/gf3_threshold_hillslope.rs` (11
    tests, literal values), three engine unit tests, and `geology_gf2.rs`'s
    light-pass replay extended to the stage. 20 mutants, run in a scratch
    copy: all 20 killed, four of them after tests were added.
- **GF-7 built: the geological clock, behind the same gate and inert in the
  app — verified by the main loop 2026-09-27 (geology_gf7 4/0, geo_clock 6/0, params_mapping 36/0, gf0 controls 4/0 re-run; world_workspace.gd parse-checked; staged tree built alone in a worktree). The stage-06 row is not yet seen in a running shell.** It is
  `cartalith_engine::geo_clock`, with `WorldParams::geo_age` (τ, 0.25–4, 1.0 at
  both boundaries) and a `geo.age` `PARAMS`/`JS_PATHS` row; a save without the
  key reloads at 1.0. It scales pass and iteration counts only: linear for
  stream power (light pass, evolve, sediment fill) and the threshold
  hillslope; saturating (`k = 0.5`) for glacial and coastal. It acts only with
  `geology_model` **and** `geology_processes` on and τ ≠ 1; otherwise every
  call site runs today's expression. The weathered-mantle law is built and
  tested but **not wired**: §4.12 anchors it at 2 m at τ = 1, which contradicts
  its own τ = 1 identity, so that needs an owner call. There is no karst hook
  before GF-5. The app-default, processes-on-at-τ-1, parity and
  every-clocked-site worlds hash bit-identical to HEAD `ec4e078`: 102 of 102
  arrays, two seeds. See `GEOLOGY_FIRST_SCOPE.md` §5.8.
  - **Sweep with the switch on (800 km, five seeds, τ 0.5 / 1 / 2 / 4):**
    - **No τ passes B1, B2 or B4.** B1 and B2 move by at most 0.013 and 0.002
      across τ, and B4 does not rise with τ.
    - **B8 fails at every τ above 1**: at τ = 2 the 1–3-cell lakes are
      1.9–2.7× the control's, and at τ = 4 they are 4.2–7.7×. It passes at 0.5
      and 1.
    - B3 and B10 pass at τ ≥ 1. Glacial and coastal output rise with τ and
      saturate. Incision over a fixed channel set does not rise.
    - **Recommendation:** the default stays 1.0, and `geology_processes`
      should stay off. The limit is that the kernel alone lowers land by
      metres even at 36 iterations, and isostatic rebound returns 78–93 % of
      that.
    - **B9:** 1.08× at τ = 1, 1.32× at τ = 2 and 1.82× at τ = 4. The cost is
      disclosed, not gated.
    - **Screenshots** (τ 1 against 4, windowed, two seeds): channels, small
      lakes, volcano flanks and coastlines change. **Relief does not visibly
      track rock.**
  - **UI:** stage 06 gets `geological age ×` at the top of its erosion group,
    with a readout of the effective counts from `WorldGen.geo_clock_readout`
    (`erode_bridge.rs`). While the processes are off the row is dimmed and
    non-editable, with its reason. `DCC_CONTROL_INDEX.md` has its row. Parse-checked; **not yet seen rendered in the shell** (§5.8).
  - **Tests:** 6 `geo_clock` unit tests with literal laws and counts; 1 engine
    unit test; `tests/geology_gf7.rs` (4 tests, including one that every
    clocked call site runs the clock's count); `params_mapping.rs` and
    `erode_bridge.rs` additions. Mutation testing ran in a scratch copy:
    31 of 31 mutants killed, one of them after a test was added.
- **Ruling BM scoped, nothing built (2026-09-27):** `GEOLOGY_FIRST_SCOPE.md` amended with rock-aware construction (§4.14), uplift-driven erosion (§4.15), revised B1/B2/B4 (§5.2, §5.9) and milestones GF-10…GF-13; next is GF-10, a harness-only prototype measurement.
- **GF-10 measured: NO-GO. verified by the main loop 2026-09-27 (harness fast tests 7/0 re-run); NO-GO (2026-09-27).**
  The Ruling BM prototype is harness code only: a new "GF-10" section of
  `tests/gf0_geology_harness.rs`, with no production change.
  `GEOLOGY_FIRST_SCOPE.md` §5.10 has the tables. Ruling BN's Q12 (rift
  shoulders and subsidence) and Q14 (a moving coast) are written into §4.14,
  §4.15 and B14.
  - **The verdict against the pre-registered rule:** at 800 km, 0 of 5 seeds.
    B1's margin is met on 0 of 5, the B4 twin's on 0 of 5 and B2's on 5 of 5,
    and B8 fails for arm C on 4 of 5. No setting of the grid meets B1's
    margin on any seed.
  - **Also failing:** B15 (the stage clamps 2–72 cells at 1.0), and B9 (a
    projected 2.65–2.73 × the app's generation time, against ≤ 1.20).
  - **What does work:** construction alone makes scarps (a breach-line slope
    ratio of 1.5–11.9) and keeps B8 on 4 of 5 seeds. The stage erases the
    scarps and adds small lakes.
  - **§5.8's τ > 1 small lakes: the cause is found.** It is the routing,
    frozen for the call's iterations. Refreshing it every iteration removes
    88 % or more of the excess. Regolith accounting and the carve are ruled
    out.
  - **Tests:** 3 new fast tests (the replay is bit-identical to
    `generate_terrain`; B13; B14). 10 of 11 mutants killed, the survivor
    explained in §5.10.
  - `cargo test --workspace --no-fail-fast`, run in a `git archive` copy of
    HEAD `7ac41a4` plus the harness:
    - **before:** 186 result lines, 4 100 passed, 1 failed, 49 ignored. The
      failure was the copy lacking `reference/`, which was then added;
    - **after:** 186 lines, 4 104 passed, 0 failed, 52 ignored.
  - The scope goes back to the owner. `OUTSTANDING_WORK.md` is not touched.
- **Stream-power routing refresh measured: it changes app worlds, so it is
  NOT adopted — owner call. verified by the main loop 2026-09-28 (stream_power_refresh 5/0 re-run) (2026-09-28).**
  `GEOLOGY_FIRST_SCOPE.md` §5.11 has the tables.
  - **Built:** `cartalith_erosion::stream_power_kernel_refreshed` and
    `stream_power_kernel_rock_refreshed`. They rebuild routing every `k`
    iterations, from the ported setup moved into `stream_power_routing`.
    There is no production caller.
  - **Effect on app worlds** (5 seeds, 800 km, 9 iterations): every interval
    below 9 changes 96.8–99.1 % of cells, and small lakes move either way by
    seed. Ocean on path stays 0, and relief and slope fall slightly.
  - **Cost:** 1.14–2.30 × generation.
  - **At τ = 4 on the gated path:** every-iteration refresh reproduces
    §5.10's 62 / 99 / 43 / 53 / 88 small lakes.
  - **Recommendation:** don't adopt it for the app. Wire it to the gated path
    only if a long-run path is ever built.
  - **Identity:** whole-world hashes are identical before and after, 15 of
    15 (app, app with processes, `WorldParams::defaults`; 5 seeds each).
  - **Tests:** 5 fast tests, and 7 of 7 mutants killed.
  - `cargo test --workspace --no-fail-fast` in a `git archive` copy of HEAD
    `f607262`:
    - **before:** 186 result lines, 4 336 passed / 0 failed / 52 ignored;
    - **after:** 187 lines, 4 341 / 0 / 54.
  - `OUTSTANDING_WORK.md` is not touched.
- **Code and doc drift found by the geology scope, fixed — pending independent
  verification.** `OUTSTANDING_WORK.md` §2.13's "Code and doc drift found by
  the geology scope" row, all five items: (1) `sample_bridge::CellSample`'s
  `stress`/`resistance`/`drainage` are `Option<f64>`, omitted (never `0.0`)
  when the backing `WorldState` field is genuinely short; the lithology build
  now reads `resistance_field` with `.get(i)` instead of indexing it directly,
  removing the panic risk. `lib.rs`'s `sample_cell` omits the same three keys
  and pairs an omission with a `*_reason` key (`right_dock.gd` dashes with it,
  mirroring the rock rows). (2) `EdificeModel`'s doc comment in
  `cartalith-terrain/src/lib.rs` corrected: the engine crate's own default
  stays off (golden-pinned), but `cartalith_godot::params::defaults()` turns it
  on — "default-off at both boundaries" was wrong. (3)
  `EROSION_GEOLOGICAL_TIME_SCOPE.md` §1 corrected with a dated note: glacial
  erosion has been an app-default divergence since Ruling AU (2026-09-24), not
  "none" of the second-block passes. (4) `README.md`'s scope list gained the
  eight `*_SCOPE.md` files it was missing (`EROSION_GEOLOGICAL_TIME`, `EXPORT`,
  `GUI_SHELL`, `MILITARY_MANPOWER`, `RELIGION_DIFFUSION`, `SCULPT_LIVE`,
  `STORY_PLANNING`, `TIMELINE`) — every `*_SCOPE.md` at the repository root now
  appears there. (5) The nine misfiled `OUTSTANDING_WORK.md` rows (river
  styling, zoom-sensitive rivers, the four drawing techniques, geology first,
  this drift row, landslip/pinnacle) moved out of §2.4 (Vault) unchanged: six
  rendering/drawing-technique rows into §2.5, and geology-first/this row/
  landslip-pinnacle into a new §2.13 "Terrain generation" — no row text, no
  headline count, no closure.
  - **Tests:** four new Rust tests in `sample_bridge.rs`
    (`short_substrate_fields_read_as_absent_not_zero`,
    `each_short_substrate_field_is_independently_absent`,
    `full_length_substrate_fields_read_as_present`, and the existing
    `civ_sourced_fields_are_absent_without_a_civ_layer`). `cargo test
    --workspace --no-fail-fast`: 181 result lines, 4037 passed, 0 failed
    (re-run from a clean full workspace build after every edit in this
    entry).
  - **Probe:** `_dashreason_probe.tscn` (new, windowed): 11/11 green — a
    positive control on a real generated world (Resistance/Drainage read real
    numbers), the reasoned-dash path driven directly against
    `right_dock.gd::_sample_field_text` with synthetic dictionaries, and a
    zero-is-not-absent control (a real `0.0` reading is never mistaken for a
    dash).
  - **Not independently verified yet** — this entry records what was built
    and measured; a second pass should re-open each item at its symbol before
    this line is trusted as fact.

- **Phone left/right dock sheet scroll, checked with a real touch drag —
  verified by the main loop 2026-09-27 (`_phonesheets` 0 failures re-run at 1080x2340 touch).** `OUTSTANDING_WORK.md`'s row was last
  checked with `_sheetscroll_probe.gd`, which drives `--resolution` rather
  than the `--vp` SubViewport convention every other phone probe uses, so it
  never actually reached phone mode (its own output printed `phone=false`)
  and it set `scroll_vertical` directly rather than dragging, so it never
  exercised the touch path the "will not scroll back up" half of the row is
  about. New `_phonesheets_probe.gd`/`.tscn` boots the real phone shell via
  `--vp 1080x2340 --force-touch`, drags the left and right dock sheets down
  and up through the SubViewport's own hit-test (mouse-emulated touch, the
  `_rangeswipe_probe.gd` pattern), and reads `scroll_vertical` live off the
  `ScrollContainer` at each step. **11/11 checks pass against HEAD, no code
  change needed** — `dcc_shell.gd::_reset_dock_scroll` (called from
  `_set_sheet_open`) already zeroes the offset on every reopen, and a touch
  drag can always bring it back to 0 within an open sheet. Mutation-tested:
  commenting out `_reset_dock_scroll`'s body reproduces the old "retains
  offset" defect (2 checks go red); restoring it is green again.
  Setting `PHONE_SCROLL_DEADZONE` far above its authored `10` reproduces a
  "will not scroll back up" defect (3 checks go red, including the drag-down
  check going false-negative); restoring `10` is green again. Screenshots at
  `left_scrolled_before_close.png` / `left_reopened.png` (and the `right_*`
  pair) in the session scratchpad show the sheet blank-scrolled, then back at
  its `WORLD` header on reopen. `_ctxphone_probe` re-run for regression:
  56/56. **The two handset route panes that still overflow**
  (`export_maps`/`export_gis` bodies in `data_manager_window.gd`,
  `ANDROID_UI_SPEC.md` row) were **not touched**: that file is the other
  lane's, mid-edit for rivers, and the row's own body lives nowhere else —
  left for that lane or a follow-up pass with the file free.

### 2026-09-25

- **`MAP_CONTEXT_SCOPE.md` written, as a proposal and not a schedule.** It
  covers the right-click context card, an 8-slot tool ring per domain, and
  their tablet and phone forms. It adds seven milestone rows (*Map context*,
  below), and none of them is built. **It records one collision nobody had
  written down.** `design/dcc-environment-2026-08-31/spec/06-phone.md` §7
  makes the phone's long-press *"sample terrain → pin + chip"* (480 ms). The
  shipped `map_overlay.gd` (`_TOUCH_HOLD_MS := 500`, PH-02) makes the same
  gesture open the context menu. RP-S6 will hit this the day it starts. The
  scope's fork F1 proposes the pin *and* the verbs.
- **An interactive prototype of that proposal** is in
  `design/map-context-2026-09-25/`: three Design-canvas frames (desktop, tablet,
  phone) that the owner reviewed and asked to refine for contrast. **It is a
  mock over a hand-drawn map and moves no ledger row.** CM-1…CM-7 remain as
  below, because nothing under `godot-project/` changed.

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
budget); ~~whether v2.25's `tileShadeExag` is ported or declined~~ (ported
2026-09-24 under Ruling AP, *verified 2026-09-24* — see the
LOD-D2 row); the MV-4
folder-picking mechanism; ~~importing legacy flat `.zip` settlements/labels~~
(ruled by Ruling AU and built 2026-09-24, pending independent verification —
see SF-7);
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
**The other two are built and verified (2026-09-24):**
the phone's memory confirmation above 2048 × 1311 cells
(`new_world_dialog.gd::_on_create` / `_confirm_phone_memory`; probe
`godot-project/_nwmem_probe.gd`, run once with `--force-touch` and once
without) and the sculpt/paint clear (`SculptStamp::footprint` →
`SculptEditor::footprint` → `PaintEditor::clear_cells_under`, called from
`WorldGen::sculpt_commit`; `sculpt_bridge.rs::sculpt_footprint_clears_paint_under_the_stamp_and_keeps_paint_outside`,
`sculpt.rs::footprint_covers_every_cell_apply_changes_and_is_not_the_bbox`).
**The memory question now covers every route that generates at the dialog's
size (2026-09-24, verified):** Create, the
tool-options Generate (`app.gd::_run_pipeline`), WORLD ▸ Generate
(`world_workspace.gd::_regenerate_now`) and the heightmap import
(`app.gd::_import_heightmap_at`, gated on the picture's working grid) all pass
`NewWorldDialog.generate_checked()` / `confirm_phone_memory_then()`;
`_nwmem_probe.gd` drives each route at 2048 and 4096, phone and desktop. None of the LOD
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
| P2-16 | 16 — provinces (`_civGenerateProvinces`) | done | `cartalith_civ::civ_generate_provinces`; `WorldGen::get_provinces` and `build_province_boundary_texture`; the overlay is assigned to `viewport_host.gd`'s `province_view` every redraw, and provinces feed `world_data_window.gd`. *Corrected 2026-09-24: this said provinces also feed `place_search.gd`; that file's header declines to index them (a province's position is its capital's, already a settlement row).* **Provinces follow painted territory since `c4435ee` (2026-09-24)**: `cartalith-godot`'s `civ_reprovince` re-runs `civ_generate_provinces` over `civ.territory` after `CivTools::rebase` merges the paint (and after a GeoJSON territory import), as the reference's `_civGenerateProvinces` reads the painted `civTerritory`. **2026-09-24, verified:** Generate Roads (`CivRebuild::Routes`) no longer re-bases the paint onto its already-painted pre-run territory (`lib.rs::civ_rebase_territory_paint` returns early when the mode did not re-derive the layer), so a later subtract restores the computed owner; and a GeoJSON territory import is written into the Territory paint layer (`geojson_apply.rs`, `CivTools::territory_paint`) rather than `civ.territory`, so a recompute keeps it. Tests: `civ_merge_tests::a_subtract_after_any_rebuild_restores_the_computed_owner_not_the_paint`, `geojson_apply::tests::imported_territory_survives_a_recompute_and_paint_composes_on_top`; probe `_geoterr_probe.gd`. *The scope document's "deliberately not wired, no new `TextureRect`" is stale* |
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
| JP-CLAIM | `jp_claimed_at` reads the live claim grid's `0` as unclaimed (`OUTSTANDING_WORK.md` §2.11) | done — verified 2026-09-26 | **2026-09-26.** The reference tests `civTerritory[i]>=0` on a `Uint8Array` (`_jpClaimedAt`, v2.11 18850; `_civAutoPolity` writes `0` for unowned), so every sampled point read as claimed: `claimedFrac` 1 on every stage including open sea, tolls ("one levy per political frontier") always 0, and the claimed-land infrastructure floor applied everywhere. The port copied `>= 0`; this port's grid is also `0` = unowned. The milestone-5 golden had been captured on a fixture whose unclaimed cells were `-1`, so it verified the discriminating behaviour while the live grid never exhibited it. Now `> 0`; the fixture is on the live `0` convention and **no golden value moved**. Live journey output does move: sea stages and unclaimed land report `claimed_frac` < 1, tolls count real frontiers, the infra floor applies only to claimed land. Tests `jp_claimed_at_reads_zero_as_unclaimed`, `m5_derive_stages_matches_the_reference_stage_for_stage` (now red under `>= 0`), `journey_bridge::the_assembled_world_actually_drives_jp_plan`, and on a real generated world `substrate_tests::the_live_claim_grid_reads_unowned_cells_as_unclaimed` |
| JP-PARTY | Widen `JpParty` from four fixed species to a generic animal map | declined | `jp_capacity_ex`'s sums are pinned to `JP_ANIMAL_KEYS` and read `jp_seasonal_animal` / `jp_desert_animal_mod` per species; the shipped alternative is `travel_library.rs`'s "substitutes for" path. Declined in the document and matched by the code |

**Group total: 20 — 19 done (JP-CLAIM verified 2026-09-26), 1 declined.**

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
| EC-8 | `_civFactionAggregates`' resource- and density-fed half, surfaced as a readout | partial | *Corrected 2026-09-24 (alignment audit Part 1 C19): this row said `blocked` on a memory decision; the blocker was worked around in `45b368d`.* **Built:** `WorldGen::civ_faction_economy` (`cartalith-godot/src/lib.rs`) feeds the aggregate both rasters — resources rebuilt on demand, density from the retained `CivData::dens` — and returns territory, pop, food capacity/surplus, the fifteen resource means, strategic, exports/imports; wrapped by `engine_bridge.gd::civ_faction_economy` and drawn by `civilization_workspace.gd::_fill_faction_economy` (Economy ▸ By faction). **Not met:** tax income is drawn nowhere, and the Faction Roster — where the old on-screen note lived — does not show it (`GUI_GAP_REGISTER.md` FR-03). *A reopened project answers since 2026-09-24 (Ruling AR's world substrate, `SAVEFILE_COMPAT.md` §8.3; verified 2026-09-24): `_jprestore_probe.gd` step 4 compares it byte-for-byte against the generated world; a project saved before then still returns empty* |
| EC-9 | Military manpower as the economy layer's first real consumer | done | `cartalith-civ/src/manpower.rs` reads `civ_current_agrarian_density`, `civ_faction_aggregates`, `civ_catchment_pop`'s tiers and `RoadComponents`/place navigability; surfaced in `civilization_workspace.gd`'s "Military" category |
| EC-10 | IN-13 — trade flows between settlements (`GUI_GAP_REGISTER.md` §42/§43; Rulings AB, AE, AF, AP) — **row added 2026-09-23; there was none** | partial — 3 of 4 pieces built | **Match and network flow** shipped on 2026-08-25 under `GUI_GAP_REGISTER.md` §42: `cartalith_civ::trade::trade_flows`, routed by the private `WayRouter` in `trade.rs`, probe `godot-project/_in13_probe.gd`. **Scarcity prices and tariffs** landed in `bbc255f` (2026-09-23): each flow carries `price` (Ruling AB); there is a directional `Tariff` (Ruling AE) on the importer's roster row (`civ_roster_bridge.rs` `tariffs`); and `civ_trade_bridge.rs` exposes `civ_set_trade_tariff` / `civ_trade_tariff`. **Caravans on land and river ways are built** (`da51a57`, 2026-09-24): Ruling AP's derived view, one per way with trade load, nothing persisted — `TradeNetwork::way_goods`, `civ_trade_flows`' `caravans`, the CIVIL ▸ Trade Caravans group, probe `_caravan_probe.gd`. The sea-lane half is **ruled and not built**: Ruling AQ (2026-09-24) answered the question Ruling AP left open — one caravan per sea lane, the same derived view (*corrected 2026-09-24: this said the question was still open*); `OUTSTANDING_WORK.md` §2.3 has the row. **A tariff control now exists (verified by the main loop 2026-09-27 (`_tariff_probe` re-run, 0 failures at desktop and phone size), 2026-09-27):** `OUTSTANDING_WORK.md`'s "Tariffs have no control" row -- the engine/save half (`Tariff`, `set_tariff`, `civ_set_trade_tariff`/`civ_trade_tariff`, the `factions.json` round trip) was already built and unchanged by this pass. What was missing was the control: `faction_roster_window.gd::_build_tariffs` draws one row per other faction on the *importer's* own inspector pane (a tariff is directional and stored on the importer's roster row, so that pane is where `FactionEntry::tariffs` already lives) -- a percentage field, blank for "no tariff" (the engine's own encoding, never a fake `0`), wired through new `engine_bridge.gd` wrappers `civ_set_trade_tariff`/`civ_trade_tariff`. Editing one emits a new `tariff_changed` signal (kept separate from `roster_changed`, which `_on_roster_changed`'s own comment documents as re-labelling only); `civilization_workspace.gd::_on_tariff_changed` re-runs `TradeStore.refresh` when a match is already held, so CIVIL ▸ Economy ▸ Trade flows ▸ Busiest partners reflects the new rate without a manual re-match. No Rust changed; no save-format change. Verified windowed at desktop (1920×1080) and phone (1080×2340, `--force-touch`) by `godot-project/_tariff_probe.gd`: 18/18 checks pass at both sizes, including the busiest cross-faction partner row shrinking to exactly 0.75× on a 25% tariff, the directional reverse pair staying untaxed, `abc`/`200` refused with the field reverted, and clearing the field restoring the flow exactly. `cargo test --workspace --no-fail-fast`: 177 result lines, 3957 passed, 0 failed (unchanged by this pass, since no Rust was touched). **Per-faction currencies (Ruling R, kept by AR, shaped by AU) are built, verified 2026-09-28 (2026-09-27):** each faction carries a user-set name, symbol and rate against the world price index (`civ_roster_bridge.rs::FactionCurrency`, every member `None` until typed, shown as a marked default); conversion only, in `cartalith_civ::currency` (`to_currency`/`to_index`/`exchange`, an invalid rate refused); saved as `factions.json`'s optional `currency` member (`SAVEFILE_COMPAT.md` §9.2); edited in the Faction roster's Currency section; CIVIL ▸ Economy ▸ Trade flows' Busiest partners show each value in the importer's currency. Display only -- no flow reads a rate. Probe `godot-project/_currency_probe.gd` |

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
| MM-6 | Per-settlement garrisons | done 2026-09-26, **border-exposure scale and no-capital-fallback fix 2026-09-28 (Ruling AZ) — verified 2026-09-29** | Rule in `MILITARY_MANPOWER_SCOPE.md` §5.6: weight = pop × (1 + 0.35/0.45·walled + 0.20/0.45·capital·tier/5) × (1 + exposure_scale × border exposure), largest-remainder split of the rounded standing army. `cartalith-civ/src/garrison.rs::civ_garrisons` (9 unit tests with literals, including several `exposure_scale` values and the no-marked-capital case; hand mutation-tested: ignoring the setting, and flipping `+`→`*` on the exposure term, both caught). Ruling AZ: `exposure_scale` is now a user setting (`GarrisonInput::exposure_scale`, default `EXPOSURE_SCALE` = 1.0, CIVIL ▸ Military ▸ Garrisons slider 0..5), and `is_capital` reads `SettlementPlacement::capital` (the real per-settlement seat flag) through a pure, unit-tested `garrison_is_capital` predicate in `civ_military_bridge.rs` — **no longer** `FactionAggregates::capital`'s highest-pop fallback, so a faction with no marked capital gets the capital weight nowhere. **Folded into `CivParams` (2026-09-29, Rust tests verified by the main loop 2026-09-29; the Godot probes are the lane's own, since the main loop could not rebuild the DLL (held by a stuck Godot process)), closing `OUTSTANDING_WORK.md` §2.11's "Fold the garrison exposure scale into `WorldParams`" row.** `cartalith_engine::CivParams::garrison_exposure_scale` (default `1.0`, a literal duplicate of `cartalith_civ::garrison::EXPOSURE_SCALE` — `cartalith-engine` sits below `cartalith-civ` and cannot import it; guarded against drift by `cartalith-godot`'s `garrison_exposure_scale_default_matches_civ_constant`), a new `civ.garrison_exposure_scale` row in `cartalith_godot::params::PARAMS`/`JS_PATHS` (no reference control; `params::invalidates` special-cases this one `civ.*` key to `None`, since unlike every sibling row `compute_civilisation` never reads it — `civ_military_bridge.rs`'s garrison computation reads it fresh every time, uncached). The `#[func]` getter/setter (`civ_military_bridge.rs`) and the slider's range (0..5, step 0.1) are unchanged; they now read/write `self.params.civ.garrison_exposure_scale`. Saved through `params::save_state` like every other parameter, not `factions.json`, which now carries **only** a read-only compatibility member: `FactionsDoc::garrison_exposure_scale` is never written again (`#[serde(skip_serializing)]`, unconditional), but a project saved between 382945a (first shipped the setting) and this fold-in still opens with its value — `project_bridge.rs::resolve_garrison_exposure_scale`, called from `project_open`, reads the legacy member only when the archive's `params.json` `cartalith` block carries no `civ.garrison_exposure_scale` key at all; the params copy wins whenever both are present (asserted, not merely reasoned: `resolve_garrison_exposure_scale_prefers_the_params_copy_when_both_exist`). The 382945a-format fixture is a **real archive**, not hand-built JSON: generated from a worktree at `382945a` (`sample_civ()`, scale `2.5`, a 4×3 grid), checked in at `cartalith-godot/tests/fixtures/project_382945a_garrison_2_5.ctl`, and read back by `a_project_saved_by_382945a_opens_with_its_legacy_garrison_value`. New tests: `garrison_exposure_scale_round_trips_through_params_json`, `legacy_garrison_exposure_scale_member_is_never_written`, the two above, `garrison_exposure_scale_default_matches_civ_constant`, `the_civ_group_is_a_real_contiguous_group_of_fourteen` (13 → 14 rows), `every_civ_row_marks_civ_stale_and_nothing_else` (now excludes the one key with no live-apply path). Readout only, no golden moved: `exactly_the_ruled_divergences_ship_at_the_app_boundary` still finds exactly six divergences, none of them this field (both `params::defaults()` and `WorldParams::defaults()` carry the same `1.0`). Bridge `civ_military_bridge.rs`: settlement rows' `garrison` keys in `civ_military_summary[_at]`, `civ_settlement_garrison(tid, year)`; siege rows carry `garrison` (`campaign_bridge.rs`); siege ring deliberately not scaled (§5.1). Shown in CIVIL ▸ Military ▸ Garrisons, the right dock's Settlement section, the place editor's Classification section, and the dock's conflict view. Windowed probe `godot-project/_mm8garrison_probe.tscn` re-run clean after the fold-in: `RESULT PASS (0 failed)`, 34 checks — the STATUS row previously read "40/40"; that figure could not be reproduced against this probe's own committed `_check` calls either before or after this change and is recorded here as stale rather than repeated. `cargo test --workspace --no-fail-fast` (same batch, together with the undo fixes below): 3953 passed, 0 failed, 42 ignored |
| MM-7 | Campaigns, unit movement, combat resolution | done in Ruling AW's form (territory over time and siege lines in CARTO ▸ Conflict), verified 2026-09-24; unit movement and combat resolution not built — not asked for, and the ruling says to settle either with the owner first | Scoped in `MILITARY_MANPOWER_SCOPE.md` §5 (2026-09-24). `cartalith-civ/src/campaign.rs::campaigns_at` reads, for one year, every active conflict's siege ring (`Siege`-kind only; radius `siege_line_radius_cells`, Alesia's 11-Roman-mile contravallation, floored at 1 cell), front (4-neighbour edges between two of its sides in the recorded snapshot in force, `year_in_force`) and changed-hands cells (side → other side since the conflict's start); absent, not empty, where unreadable. 8 unit tests with literals, 9 of 9 mutants killed. Bridge `campaign_bridge.rs::conflict_campaigns(year)`; layer row `conflict` in `cartography_workspace.gd::LIVE_LAYERS` (so also in the Layers popover), `viewport_host.gd::refresh_campaigns`, `map_overlay.gd::_draw_campaign`. Still no combat, movement or outcome type anywhere in `cartalith-civ`: the layer only reads authored conflicts and recorded territory. CIVIL ▸ Military's `70adb7a` "considered and declined" line is replaced by a Campaigns note pointing at CARTO ▸ Conflict (`_fill_military`). Windowed probe `godot-project/_awconflict_probe.tscn`: ALL PASS, layer on/off moves 342 px at an in-conflict year and 0 px outside it — 2026-09-24 |
| MM-8 | Change over time (manpower across the year cursor) | done 2026-09-26 — **verified 2026-09-26**; ag-tech/government not recorded per year (follow-up) | Recomputed from the snapshot in force, not stored (`MILITARY_MANPOWER_SCOPE.md` §5.7): settlements, ways and territory are the record's; dens/terrain live (year-invariant); roster and place overrides live and labelled so. `civ_military_bridge.rs::civ_military_summary_at(year)` / `reading_at` / `CivView`; `civ_year_in_force`. CIVIL ▸ Military refills when the year in force changes (`_on_military_cursor`). Probe `_mm8garrison_probe.tscn`: faction 1 standing 4 741 at y0 vs 5 292 at y0+10, unrecorded years read the year in force, a pre-record year reads `none_in_force`; bridge mutant (recorded→live view) killed |
| MM-F2 | Finding 2 — the standing column; owner ruling AI (c), 2026-09-23: soldier upkeep per agricultural-labour bracket, derived from the era table | done | Committed `dda315e` (2026-09-23; corrected the same day from "done (not yet committed)"). `manpower.rs::SOLDIER_UPKEEP_BY_BRACKET` / `soldier_upkeep` / `alpha_bracket` (shared with `era_for`), replacing the flat `SOLDIER_UPKEEP = 3.0`. Re-derived from `ERA_BANDS` + `era_for` + `GOVERNMENT_EXTRACTION` + `CITIZEN_SHARE` by `soldier_upkeep_is_derived_from_the_era_table`; the Iron-Age-above-High-medieval pair pinned by `a_median_polity_of_each_bracket_lands_in_its_own_band`. **Re-baselines the worked example's standing army** (A 5 846 → 9 661, B 19 067 → 25 750; levy and field unchanged), accepted by the owner. Open: Kingdom B's standing now exceeds its own 365-day rung by 6.7 % (`the_force_ladder_decreases_with_duration`) |
| MM-F3 | Finding 3's residue — `ecological_factor` tracked map area; owner ruling AI (b), 2026-09-23: normalise land per person to the world's own | done | Committed `dda315e` (2026-09-23; corrected the same day from "done (not yet committed)"). `manpower.rs::civ_military_manpower_world` / `world_land_reference`, called by `civ_military_bridge.rs::manpower_rows`. Pinned by `map_scale_does_not_move_the_ecological_factor` (land ×6.25 → identical outputs). Measured on `_mpscale_probe.tscn`: standing below-band 21/33/11 of 36 on the 1 200/800/2 000 km shapes before, 17/20/17 after (b)+(c). Open: the 0.25 floor now binds on 36 of 108 faction-samples |

**Group total: 10 — 9 done (MM-7 verified 2026-09-24; MM-6 and MM-8 2026-09-26, verified 2026-09-26), 1 declined.** Corrected 2026-09-24: this line still said "6 done, 4 declined" after Ruling AW reopened MM-6/7/8.

### Story planning · `STORY_PLANNING_SCOPE.md`

One subsystem over the Timeline's year cursor. It carries the owner's three
2026-08-25 forks; owner decisions 1 and 3 above, which gated it, were both answered 2026-09-23 by Ruling AO.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| SP-1 | The Journey entity | done | **Re-checked at the symbols 2026-09-23 (SP-2's build); the old row here was stale.** It said `grep "pub struct Journey"` found nothing and that journeys were a GDScript dictionary. Both are now false. `cartalith_civ::travel_library::{Journey, JourneyRoute}` exist. The store is `InfraTools::journeys` (`journey_save`/`journey_delete`/`journey_get`), exposed as the `#[func]`s `journey_save`/`journey_list`/`journey_get`/`journey_delete`, and populated by the live shell: `journey_planner_view.gd::_save_journey` calls `bridge.journey_save` with a preset captured from the form. The entity persists as `entities/journeys.json` (`project_bridge.rs` `journey_to_dto`/`dto_to_journey`, test `journeys_round_trip`). Preset usage is real: `preset_usage_in_journeys` feeds `tl_list`'s rows in `lib.rs`. `animal_usage_in_journeys` is still `0`, correctly: a `Journey` names a preset, not an animal. **Not part of SP-1, and still session-only:** the planner's richer per-journey working state (`stage_overrides`/`layovers`/`animal_entries`/`trim` in `_journeys`) |
| SP-2 | Journey progression over the cursor | done | **Built 2026-09-23 against the engine `Journey`, which is the live one (SP-1 above).** The date: `cartalith_vault::chronos` gains a 365-day calendar (`MONTH_DAYS`, `MonthDay`, `day_number`) and `Event.start_md`/`end_md`; the year-only path is byte-identical. The cursor gains a day (`WorldGen::civ_day`, `get_civ_date`/`civ_set_day_of_year`). The maths is `cartalith_civ::journey_progress::JourneyTimeline::{from_plan, at}` over `jp_plan_full`'s own plan, with supply from the shared `jp_leg_supply`; `sp2_progression_reads_the_planners_own_golden_journey` pins it to the m5 golden. Live: `story_bridge.rs::journey_positions`; `map_overlay.gd` markers carry a food-left and arrival readout; a day slider sits in the timeline strip (row 3b). Regenerate re-snap: `resnap_journey` / `adopt_resnapped` / `resnap_carried_journeys`. Windowed probe `_sp2journey_probe.gd`. **Left:** departure is always 1 January of `start_year`, because the entity has no finer start and widening it is a format change. *Corrected 2026-09-24:* this said there was no timeline cache and quoted `journey_positions` at 48–54 ms per call at 2048×1311; that was the figure `b5aff17` (2026-09-23) fixed — `story_bridge.rs::JourneyPlanCache` keys each saved journey's plan on a content hash of every input `plan_saved_journeys` hands `jp_plan_full`, so a warm call re-plans nothing (the commit reports ~12×; not re-measured here). The planner's own Journeys list still clears on a regenerate (`journey_planner_view.gd` connects `generation_finished` to `clear_journeys`) while the engine journey survives; on project open it is rebuilt (`_restore_from_engine`, `a64ffad`) |
| SP-3 | The settlement timeline strip | partial | *Moved done → partial 2026-09-24 (alignment audit Part 1 C25): the `ruins`/`fortified` piece does not survive a save — `TimelineSnapshot::collapse_flags` is in memory only (its own doc: `project_bridge`'s `TimelineYearDto` does not carry it), so every reopened year reads as not recorded; and the authored-events list is not read-only as this row said — `place_editor_window.gd::_build_add_event_form` writes a Chronos line into the note.* All four pieces were built 2026-09-23. Faction ownership periods (`civ_settlement_ownership_periods`, `_build_political`); population/tier per recorded year (`civ_settlement_population_trajectory`); **authored events** — a ` ```chronos ` block in the settlement's vault note, Chronos Timeline syntax exactly (Ruling AM), parsed by `cartalith_vault::chronos::parse`, read by `VaultSession::entity_chronos`, exposed as `vault_entity_chronos`, drawn by `place_editor_window.gd::_build_authored_events`, with an *Add event* form beneath it (`_build_add_event_form`) that appends a Chronos line to the note; **journeys passing** — `civ_passed_settlements_at`/`JourneyTimeline::elapsed_at_point`/`civ_settlement_journey_passes(tid)`, a "Journeys passing" section on the Political History tab, probe `_sp3journeypass_probe.gd`; **`ruins`/`fortified` per year** — `TimelineSnapshot::collapse_flags: BTreeMap<tid, CollapseFlags>`, a side table (not new `NamedSettlement` fields — measured blast radius, see `OUTSTANDING_WORK.md` §2.3's SP-3 row), written only by `run_collapse_simulation`, a fourth trajectory column, probe `_pestatus_probe.gd` — **not persisted** (see above). Four separate lists/columns on one tab, not one interleaved strip — Ruling AQ (2026-09-24) rules the combined strip be built; not built |
| SP-4 | The conflict overlay | done | 2026-09-23. `cartalith_civ::conflict` (`Conflict`, `ConflictKind` front/arrow/siege/battle, `ConflictAnchor` settlement-or-province keyed on `tid`, `ConflictStore`, `side_manpower`), `cartalith-godot/src/conflict_bridge.rs` (`conflict_add/update/delete/list/get`, `conflicts_attached_to`, `conflict_sides_manpower`), `WorldGen::conflicts`, saved in engine-owned `entities/conflicts.json` (`SAVEFILE_COMPAT.md` §9.7). Shell: CIVIL Conflict tool (Way/Route vocabulary), right-dock `CTX_CONFLICT` form, CIVIL ▸ Military ▸ Conflicts list, `map_overlay.gd::_draw_conflict`. Windowed probe `_sp4conflict_probe.gd` 48/48. **Manpower is the world now, not the conflict's year** (disclosed in the dock); year-scoped read, map-click selection and re-drawing a committed shape not built. Lifecycle decisions: Ruling AO addendum. **OUTSTANDING_WORK.md §2.11, "Anchored conflict marks sit half a cell off their settlement" — fixed, verified 2026-09-28.** `conflict_bridge.rs::anchor_pos` returned a settlement's raw cell index (the corner) rather than its centre (`+0.5`); `map_overlay.gd`'s settlement pins draw at `_cell_to_screen`'s `cell + 0.5`, while a conflict's points go through `_point_to_screen` (no added offset), so a settlement-coordinate treated as a conflict point without the `+0.5` sits half a cell up-left of the pin. Fixed at the one place both SP-4's marker and CARTO ▸ Conflict's siege ring resolve an anchor (`anchor_pos`, also `landmark.rs::BattleMark`'s anchor input via `lib.rs`). New unit tests `conflict_bridge::tests::{a_settlement_anchor_resolves_to_the_cell_centre_not_the_corner, a_province_anchor_resolves_through_its_seed_settlements_centre}` (literal `(4.0, 0.0)` → `(4.5, 0.5)`); mutation-tested red without the `+0.5`. `_awconflict_probe.gd` updated (its siege's authored point and its "centred on the town" literal `(cx, cy)` → `(cx+0.5, cy+0.5)`) and extended with a windowed screen-space check that the siege ring's centre and the settlement pin's centre coincide (measured: both at the same viewport pixel). `_sp4conflict_probe.gd` needed no literal change — its own "attaching did not move the drawing" check draws on user-clicked points, not the settlement's raw coordinate. **OUTSTANDING_WORK.md §2.11, "Migrate pre-fix conflict anchors on load" (Ruling AY, 2026-09-28) — built, verified 2026-09-28.** `entities/conflicts.json` now carries an additive `anchor_convention` member, written `"centre"` by every writer from this change on (`project_bridge.rs::ConflictsDoc`); a document with no such member (`serde`'s default gives `""`, distinct from `"centre"`) is a pre-fix archive. The loader (`project_bridge.rs::conflicts_from_doc_migrated`, called from `project_open`'s restore pass, extracted so it is unit-testable on its own) shifts every such document's anchored conflicts' `anchor_at` by `+0.5, +0.5` before use — the same corner-to-centre correction `36312e2` applied to `anchor_pos` itself — so `resolved_points`'s `anchor_now - anchor_at` delta lands the shape exactly where the pre-fix build drew it. A free (unanchored) conflict has no `anchor_at` in either convention and is untouched; the marker is checked, never assumed. Four new unit tests in `project_bridge.rs`: `a_pre_fix_archive_migrates_its_stored_anchor_to_the_centre_convention` (a **real** `entities/conflicts.json`, captured verbatim via `cargo test -- --nocapture` from a worktree built at `36312e2^`, one commit before the fix — literal `anchor.at: [7,4]` (corner) → migrated `(7.5, 4.5)` (centre), and its `resolved_points` against a moved settlement equals `(10.0, 6.0)`, the literal the pre-fix build's own `resolved_points` produced for the identical scenario in that same worktree run); `a_conflict_saved_by_the_fixed_build_reopens_with_no_double_shift` (round trip: `anchor_convention: "centre"` present, no second shift); `a_free_unanchored_conflict_never_moves_on_migration`; and the existing `a_conflict_survives_save_load_reopen` updated for the new struct member. Mutation-tested: dropping the `+0.5` shift and dropping the marker check (`migrate_pre_fix_anchor = true` unconditionally) each turned a different one of these tests red. `cargo test --workspace --no-fail-fast` run after the change (see this task's report for the exact count) — **verified 2026-09-28**, per this task's build-lane brief; not yet probed in Godot (no `.gd` files touched, per CM-7's ownership of the workspace layer during this batch) |
| SP-5 | The planning aid, joined up | done | 2026-09-23, same day its blocker (two of SP-1…SP-4) went stale. Both `STORY_PLANNING_SCOPE.md` §4 joins built: `WorldGen::conflicts_touching_settlement(tid)` (`anchors_touching` — this settlement's own conflicts plus its province's, read live) feeding a "Conflicts here" section on the Political History tab; each dated journey-pass row annotated when its year falls inside one of those conflicts' active range (no new Rust needed — both values already had year precision). Probe `_sp5joined_probe.gd` confirmed to pin the change (24/24 current, 11/24 pre-change). |

**Group total: 5 — 4 done, 1 partial.** Story planning was built in one session 2026-09-23; SP-3 moved to partial 2026-09-24 because its collapse flags are not saved.

### Markdown Vault · `MARKDOWN_VAULT_SCOPE.md`

Seven milestones (0-6). The entity audit that opened this work found that
continents did not exist as entities; milestone 0 created them.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| MV-0 | 0 — the addressable continent | done | `cartalith_civ::Continent` and `civ_continents(...)` with its own `civ_continent_name_rng` stream; exposed as `WorldGen::get_continents()`; read by `engine_bridge.gd` and turned into a vault-linkable row by `civilization_workspace.gd::_knowledge_row(cg, "continent", …)`. **Roster culture (2026-09-24, verified):** `civ_continents_with_cultures` names a landmass in the roster culture of the faction holding most of it (ties to the lowest id; unowned land keeps `civ_default_culture(1)`); an unedited roster names byte-identically. `compute_civilisation` passes `faction_cultures` to it, so the app uses it |
| MV-1 | 1 — link, read, section-aware write-back | done | Crate `cartalith-vault` (`backlinks.rs`, `block.rs`, `export.rs`, `links.rs`, `markdown.rs`, `provider.rs`, `template.rs`); `cartalith-godot/src/vault_bridge.rs` carries 55 `#[func]`s (`grep -cE "^\s*#\[func\]"`, 2026-09-26; was 53), plus `vault_saf.rs`'s one; panels in `shell/vault_window.gd` (`_build_connection`, `_build_attach`, `_build_links`, `_build_reader`) and persistence in `shell/vault_store.gd`. **2026-09-26, verified 2026-09-27** — closed `OUTSTANDING_WORK.md`'s "Record approved Vault Browser mockup + close its remaining gaps": the mockup is filed at `design/vault-browser-2026-09-21/Cartalith Vault Browser.dc.html` and cited from `MARKDOWN_VAULT_SCOPE.md` §"Browse" alongside the live artifact link it already carried. The two named engine gaps are closed with real data, not invented: `cartalith_vault::VaultSession::file_backlinks`/`file_mentions` (path-keyed, no entity needed) wrap `Backlinks::backlinks_to`/`mention_candidates`, exposed as `vault_bridge.rs::vault_file_backlinks`/`vault_file_mentions` and `engine_bridge.gd`'s wrappers of the same names; drawn in `vault_window.gd::_build_browse_backlinks`. "Centre on map" reuses the existing `app.viewport.move_view_to` call every other "Focus/Centre" button in this shell already has (`place_editor_window.gd`, `faction_roster_window.gd`) — a browsed file has no entity scope, so `_build_browse_centre` resolves a position from the note's own frontmatter (`type: settlement`, `tid: N`) against `bridge.settlements()`, and draws the button disabled with the specific reason when the frontmatter names no settlement or that tid is not in the current world. Also closed, re-measured against the code (the row's own six "remaining gaps against the mockup" were re-opened at the symbol rather than trusted, per `MISTAKES.md`'s preflight rule — its "already done" note names only the frame/split/`max_size`/"Close" work from `3736fe7`, not these six): **excerpt raw `## ` markers** — `_first_lines` now strips a real ATX marker (1-6 `#` then a space) per line before joining (`_strip_heading_marker`); **"Open to edit" as a primary button** — `_build_note_editor` takes `as_primary`, true only from `_build_browse_preview`, drawing a filled `DccWidgets.action` labelled "Open to edit" there while `_build_attach`'s own call keeps the quieter text button unchanged. Rust unit test `a_browsed_file_finds_its_backlinks_and_mentions_by_path_alone`; windowed probe `_vaultbrowse_probe.gd` (positive/negative controls for backlinks, mentions, Centre on map, the heading-marker strip and the button label); `_vault_probe.gd`/`_vaultunlink_probe.gd` re-run clean. **Still open, re-measured and left alone this pass** (structural layout changes, judged too large a risk to improvise without a design pass): search sitting at the head of the 300 px tree column rather than as its own full-width section above the split; the connection note staying two lines (`"✓ Connected — %s\n%s"`) rather than the mockup's one-line title-bar strip; outline and excerpt drawn stacked rather than side by side. Preview text size was re-measured and found already at the mockup's own 9 px (`DccWidgets.note`'s `FS_MICRO`) — the row's "12-13.5 px" figure did not match the code read this pass, so that cell is stale, not a live gap. **2026-09-27, later — this "stale" call was itself wrong** (verified by the main loop 2026-09-27 (`_vaultlayout` re-run, all passed at both sizes)): the mockup's own CSS (`Cartalith Vault Browser.dc.html`, the outline row's `font:12px` and the excerpt's `font:13.5px/1.7 'Inter'`) does read 12 / 13.5 px, and the live outline/excerpt rows were drawing at `DccWidgets.note`'s `FS_MICRO` (9 px), not 12. Fixed at the two `note()` call sites in `vault_window.gd::_build_browse_preview` (outline heading rows and the excerpt body) with an explicit `add_theme_font_size_override("font_size", DccTheme.FS_BODY)` — 12, the closest existing `DccTheme` token to both mockup figures, the same one `DccWidgets.modal_prose()` already reads prose in elsewhere; no raw literal added. `_vaultlayout_probe.gd` extended with a fourth check block that reads `get_theme_font_size()` on the live outline/excerpt Label, not the constant, at both 1280x800 and 1080x2340 `--force-touch` — passes at both densities; `_vaultbrowsegaps_probe` re-run clean. Screenshots saved and reviewed at both sizes. **2026-09-27, later; verified by the main loop (`_vaultlayout` all passed at 1280x800 and 1080x2340 touch, `_vaultbrowsegaps` still passes, desktop screenshot inspected)** — a second lane closed the three layout gaps this row left open: search now sits at the head of the tree column (`_build_browse_search`, filtering `_build_browse_tree` live via `_refresh_browse_tree`, keystroke by keystroke, without freeing the field itself); the connection note is a one-line header when bound and browsing (`_build_connection_header`: `name · N notes`, "Change vault…", "read-only preview · open to edit" — two lines on phone, where the one-liner measured off the right edge of a 1080 px viewport); and outline/excerpt are drawn side by side on desktop and stacked on phone (`VaultOutlineCol`/`VaultExcerptCol` under an `HBoxContainer`/`VBoxContainer` swap, the same idiom `culture_profiles_window.gd` uses for its own fold). New windowed probe `_vaultlayout_probe.gd`/`.tscn`, run at 1280x800 and at 1080x2340 `--force-touch`: all checks pass at both densities, and both saved framebuffer PNGs were reviewed against the mockup. Not yet re-run by a separate verifying pass **2026-09-27, owner Ruling BE — verified by the main loop:** browse and the entity view are now one layout, and the note pane has a Markdown editor. `_build_attach`, `_build_note_editor` and `_build_file_picker`, named in this row, are removed. See *The last seven days*, 2026-09-27. **2026-09-27, verified by the main loop 2026-09-27 (`_vaultbe` 55/0 re-run):** two of the three "Vault editor polish" items left by the BE/BF builds (`OUTSTANDING_WORK.md`) are fixed. The browse preview's excerpt (`VaultExcerptCol`) no longer repeats the rendered note's own opening: it now only draws when the file read failed (nothing to render then), since a successful read shows the full note a few rows below and the excerpt was a verbatim duplicate of its first lines — the mockup's outline column is unaffected. `markdown_editor.gd`'s U and S toolbar chips render their glyph underlined/struck through (`_style_glyph_chip`, a `RichTextLabel` overlay reading `[u]U[/u]`/`[s]S[/s]`, `mouse_filter = MOUSE_FILTER_IGNORE` so the click, tooltip and hit size stay the underlying `Button`'s own). `_vaultbe_probe.gd` extended with three checks (excerpt non-duplication, U underline, S strikethrough), shown failing before the change and passing after, at 1600x1000 and at 1080x2340 `--force-touch`; `_vaulttree_probe.gd`'s own now-stale excerpt assertion updated to match; `_mdedit_probe.gd` re-run clean (58/58, unaffected — headless, no rendering). Screenshots reviewed at both sizes. The third item, the phone toolbar against the on-screen keyboard, needs the owner's device and is untouched. |
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
(recounted 2026-09-24, unchanged). **The `needs_viewshed` flag matches the
viewshed reads since 2026-09-24** (it did not before: alignment audit Part 1
C24 found `peak` and the unbuilt `sacred_mountain` flagged, and the two
fortified kinds unflagged). The flagged kinds are exactly those whose pool
function reads `Derived::vis` — `fort`, `watchtower`, `fortified_pass`,
`fortified_crossing` (all through `pool_military`), `volcanic_feature`
(`pool_volcanic`), `border_marker` (`pool_border_marker`) and, **since Ruling
AV (2026-09-24, verified 2026-09-24), `peak`**: `pool_peak`
carries a fourth term, "land in view" (`PEAK_TERMS`, weight 0.20, the three
terrain terms scaled by 0.8), omitted when no observer exists — seven, pinned
as a literal set, and against `Needs::of`, by
`the_kind_table_matches_the_research_and_the_design`, and the term by
`a_peak_that_overlooks_more_land_outranks_an_equal_one_that_overlooks_less`.
CIVIL ▸ Landmarks and the phone's family sheet draw a `[viewshed]` tag from the
flag (`_landmark_probe` reads 7 live).
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
| TL-5 | 5 — the Godot boundary | done | `cartalith-godot/src/timeline_bridge.rs` (`CollapseSimRequest`, `CollapseSimReport`, `run_collapse_simulation`) and the `#[func]`s `civ_add_year`, `civ_goto_year`, `civ_year_diff`, `civ_run_collapse_simulation`. **Go to year follows Ruling AT** (2026-09-24, verified 2026-09-24): a recorded year's snapshot becomes the Territory tool's base with paint and any pending stroke dropped (`lib.rs::civ_year_loaded`, run after goto, add, remove and the collapse sim); an unrecorded year moves only the cursor — a deliberate departure from the reference's `terr.fill(0)`. `get_civ_territory_year` names the year the claims came from, which the strip's "territory holds at …" now reads. Tests `go_to_a_recorded_year_rebases_the_territory_tool_on_its_snapshot`, `go_to_an_unrecorded_year_leaves_territory_base_and_paint_untouched`; probe `_terrbool_probe.gd` §3 |
| TL-6 | 6 — UI playback controls | done, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the way highlight and ghost crops opened) | `shell/workspaces/civilization_workspace.gd`'s **Timeline** category (`_build_timeline`, `DccWidgets.category(self, "Timeline", …)`; its own header cites `TIMELINE_SCOPE.md` milestone 6): years pill row + Add year, the 1200 ms playback transport, the collapse/recovery form with its overwrite confirmation, and the "Exist only" filter over `civ_year_diff().present` (`_tl_apply_filters`). **"Ghost removed" and "Highlight new" built 2026-09-28, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the desktop and phone halo/ghost crops opened)** (CV-03): both toggles enabled in `_build_timeline_filters` and pushed by `_tl_push_marks` (from `_refresh_civ_data`, `_rebuild_timeline` and `timeline_changed`); `map_overlay.gd::set_timeline_marks` draws a `good`-token halo on live pins whose tid is in `civ_year_diff().added`, and a faded `text_ghost` pin for each removed settlement at its previous-year cell, read by the new read-only `#[func] civ_year_diff_removed` (`timeline_bridge::civ_year_diff_removed_settlements`; 3 tests, 7/7 mutants killed). Probe `_tlpins_probe.gd` (windowed, real world, years 0 and 10): 17/17 on desktop 1600×1000 and phone 1080×2340 `--force-touch` — each mark moves pixels when on and restores them exactly when off. **Ways and the strip scrub built 2026-09-28, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the way highlight and ghost crops opened):** `get_roads()` now carries a generated way's `tid` (omitted for hand-drawn ways, which no snapshot records); *Exist only* filters ways as well as pins (`_tl_apply_filters`, v2.11 line 15982); *Highlight new* strokes a 0.6-alpha `good` halo under each added way (15989); *Ghost removed* draws each removed way faded and dashed in `text_ghost` at its previous-year geometry, via the new read-only `#[func] civ_year_diff_removed_ways` (`timeline_bridge::civ_year_diff_removed_ways`, which never returns a `hidden` way — a stated divergence from 16026). `timeline_changed` now calls `_tl_on_cursor_moved`, so a strip scrub re-applies *Exist only* (it did not before). 5 timeline_bridge ghost tests, 8/8 mutants killed; `_tlpins_probe.gd` 36/36 on desktop and phone. Negative control: with the old `timeline_changed` hookup restored in a scratch copy, all 4 checks after the strip scrub fail. The probe drives add-year and the strip scrub, not a collapse run; `_petimeline_probe.gd` covers that path. *Corrected 2026-09-24 (alignment audit Part 1 C25/B15): this row said `done` and placed the controls "under Politics"; the category has been called Timeline since Ruling L* |

**Group total: 6 — 6 done.** TL-6 moved done → partial 2026-09-24, and partial → done 2026-09-28 (verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the way highlight and ghost crops opened)) once ways and the strip scrub were marked.
*Corrected 2026-09-23:* this said CV-24 and ED-02 were open owner decisions.
ED-02 (the undo-history panel) is built — `Edit ▸ Undo history…`
(`menus.gd`, `GUI_GAP_REGISTER.md` §42). CV-24 (the year scrubber as program
scope) waits on a design, not an owner decision. TL-6's criterion 5: the
*Ghost removed* and *Highlight new* toggles were inert until 2026-09-28 and
now draw per-pin marks for settlements (verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the desktop and phone halo/ghost crops opened)
and, later the same day, for ways, with *Exist only* re-applied on a strip
scrub (verified by the main loop 2026-09-28) — see the TL-6 row; routed in
`OUTSTANDING_WORK.md` §2.3.

**Way tids survive a recompute — built 2026-09-28, pending independent
verification.** `OUTSTANDING_WORK.md` §2.3's row "`recompute_civilisation`
re-issues every way's tid" was real: `compute_civilisation` rebuilt every way
with tid 0 and `civ_assign_tid` issued each a fresh id, so an unchanged
recompute between two recorded years marked all 254 ways of the probe world as
removed *and* added. Fix: `timeline::{WayIdentity, civ_way_identities,
civ_way_tid_index, civ_inherit_way_tids}` key a way by its endpoint
settlements' tids (unordered) plus its ordinal within that pair, and
`KeptCiv::way_tids` hands the old network's tids to matching rebuilt ways
before the counter runs (Recompute and Generate roads; Auto-populate is
unchanged). The reference churns the same way (`_civAutoRoutes`, v2.11 21857,
rebuilds `civWays` untid'd), so this improves on it (§7p). No save-format
change; no golden moved. Tests: 3 in `timeline.rs`, 1 in `lib.rs`
`civ_pipeline_tests` over a real world (unchanged recompute keeps every tid and
diffs empty; removing and restoring a settlement still removes and adds its
ways); 8/8 mutants killed. `cargo test -p cartalith-civ -p cartalith-godot`
2532 → 2536 passed, 0 failed, 71 result lines both. `_tlpins_probe.gd` gains an
RC leg: 50/50 on desktop and phone; against the pre-fix DLL, 6 RC checks fail
(254 added, 254 removed, halos and ghosts on screen).

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

**Village map style band count, verified 2026-09-28.**
`OUTSTANDING_WORK.md` §2.11's "harsh (yellow/purple/teal)" row: Ruling AZ
(`LARGE_ITEM_RULINGS.md`, 2026-09-28) — "more colour bands, softer, matching
the other presets." `render.rs::quantize_flat_palette`'s `BANDS` moved
`3.0 → 5.0` (four → six flat levels per channel). Only [`Npr::village`]'s
recipe ever sets it, and only "Village" among `render_workspace.gd`'s seven
`STYLE_PRESETS` sets that flag (checked by grep), so no other preset's output
moved and `BANDS` stayed a single constant rather than becoming a per-preset
parameter. Chosen by measurement, not taste alone:
`godot-project/_villagebands_probe.gd` (fixed seed 20260927, 512×328, sRGB,
windowed) put Village's harsh-transition fraction (adjacent-pixel Euclidean
RGB distance > 90 of 441.7) at 0.044551 with `BANDS = 3`, against the other
six presets' own range on the same world, [0.029251, 0.032794]; `BANDS = 5`
is the smallest band count whose figure (0.031397) lands inside that range
(`BANDS = 7` gives 0.029340, also inside but not smaller). `tests/village_bands.rs`
pins the new level spacing (literal `51.0 = 255.0/5.0`) and pins two
non-village preset recipes as unmoved; mutation-tested once (`BANDS → 4.0`
fails `village_quantises_to_six_levels_of_51`, killed). No golden or Rust
render hash reads `Npr::village` (checked by grep over `tests/*.rs`), so
nothing needed re-baselining.

**Ruling BI — the eight researched map style presets, built; verified by the
main loop (`ruling_bi_style_presets` 6/0, tile-biome golden 12/0, `_stylepresets` 0 failures; contact sheet inspected).** `LARGE_ITEM_RULINGS.md` Ruling BI (2026-09-27),
routing `MAP_STYLE_RESEARCH.md` §4: Atlas, Imhof relief, Blueprint, Ink wash,
Woodcut, Vintage atlas, Nautical and Night all joined
`render_workspace.gd`'s `STYLE_PRESETS` (now 15 tiles, up from 7), each an
absolute bundle exactly like the existing six — a look name, an NPR
dictionary, an appearance-override dictionary and (new, a 5th element) a
named elevation ramp, loaded through `EngineBridge.load_ramp_preset()` and
reset to `"Earth"` by any preset that names none.

*Hachure verified first, per the research's own question.* `render.rs`'s
`hachure` `Npr` branch (`apply_npr`, the "D-hachure" stage) already draws
real Lehmann-style hachures: strokes run along the local gradient direction,
in rows spaced perpendicular to it, with stroke frequency and darkness both
scaled by slope steepness. It is class (b)/tunable as shipped, not a stub —
none of the eight presets needed it promoted to a new technique, and none of
the eight uses it (the research's own recipes did not call for it).

*Renderer additions, all opt-in at `0.0`/absent.* Four new `RAMP_PRESETS`
rows (`Blueprint`, `Ink wash`, `Night`, `Vintage atlas` — land-only,
pre-lighting tints, the same mechanism `Atlas`/`Imhof` already used at
`ramp_strength: 0.0`); one new mechanism, `TerrainAppearance::
sea_ramp_strength` (default `0.0` in both `default()` and `js_reference()`)
blending `SEA_RAMP_NAUTICAL`'s depth-banded stops into `sea_color_core` for
"Nautical". `cargo test --workspace --no-fail-fast`: 3998 passed / 0 failed
before this work's own new test file, 4004 passed / 0 failed after (the six
new tests in `crates/cartalith-godot/tests/ruling_bi_style_presets.rs`) —
no existing count moved, so no `js_reference()`/golden output moved either,
asserted directly by `sea_ramp_strength_is_zero_in_default_and_js_reference`
and `sea_ramp_strength_at_zero_moves_nothing`. Two mutations were run against
the new tests (Python exact-replace, run alone, restore, `git diff --stat`
confirmed only the intended 100 lines survived): gating `sea_ramp_strength`'s
`if` to `false` was caught by `sea_ramp_strength_above_zero_moves_the_sea`;
collapsing `Night`'s four ramp stops to one colour was caught by
`ruling_bi_ramp_presets_are_registered`'s own stop-distinctness assertion
(the render-diff test alone did not catch it — recorded in the test file's
own comment).

*Probe:* `godot-project/_stylepresets_probe.gd`/`.tscn` (windowed), on the
same generated world `_presetgal_probe.gd` uses. Applies each of the 15
tiles through its own `button`'s `pressed` signal (the real click path),
reads the live look/NPR/appearance/ramp back from the bridge and asserts
each against the preset's own definition, screenshots the viewport, asserts
each preset's frame differs from the previous one by more than 1% of RGB
bytes (actual range measured: 10.4%-23.6%), and builds one contact sheet of
all 15. Two full runs, 0 failures each (the first run's contact sheet had a
harmless `Image.blit_rect` format-mismatch bug, fixed before the pixels were
judged).

*Visual read, judged against the intended style, this machine, this world
(seed 483920):* Atlas and Vintage atlas both read convincingly as their
target (bright reference-atlas and warm mid-century wall-map respectively).
Woodcut and Night are the strongest of the eight — visible cross-hatch
texture on the former, a genuinely legible dark-mode map on the latter.
Blueprint's first build was wrong: `biome_sat: -1.0` was applied by
`land_color` **after** the ramp mix (`ramp_strength`'s blend runs, then
`biome_sat`'s desaturation runs over the already-blue result), so the land
rendered grey rather than Prussian blue — found from the probe's own
screenshot, fixed by dropping `biome_sat` from the preset (the ramp's opaque
stops already replace the material colour at `ramp_strength: 1.0`, so it was
never needed), re-verified visually. Imhof relief is real but subtle next to
Atlas at this preset's `ramp_strength: 0.5` — the warm/cool read is present,
not dramatic. Ink wash reads as intended for land (warm ink-grey) but lakes
stay their own blue, since the land ramp does not touch water — an accepted
limitation, not a bug. Nautical's depth-banded sea ramp is verified correct
by the Rust tests and by the live appearance readback (`sea_ramp_strength:
0.7` applied), but this probe's crop shows inland lakes rather than open
ocean, so the characteristic multi-band offshore read is not clearly visible
in the saved screenshot — worth a second pass with an ocean-framed crop
before calling the visual side fully checked.

*Deferred, per Ruling BI's own instruction:* the four new drawing
techniques (Tanaka illuminated contours, Raisz physiographic pictograms,
mappa-mundi figurative icons, Ordnance Survey symbology) are scheduled, not
built — each needs its own scoping pass first. Vintage atlas's ramp
substitutes for re-pitching the four base material ramps (`grass_temp` etc.
via a new look), which would have been a larger addition than this ruling's
"small renderer addition" scope; the ramp mechanism reaches a materially
similar picture more cheaply. Files: `crates/cartalith-godot/src/render.rs`,
`crates/cartalith-godot/tests/ruling_bi_style_presets.rs`,
`godot-project/shell/workspaces/render_workspace.gd`,
`godot-project/_stylepresets_probe.gd`, `godot-project/_stylepresets_probe.tscn`.

**Style preset follow-ups (2026-09-27), verified by the main loop 2026-09-27 (`_ctxring` 114/114 re-run).**
Two items closed from the row above.

*Ring density.* `cartography_workspace.gd::ring_slots()`'s CARTO ▸ Style▸
sub-ring built one wedge per `STYLE_PRESETS` entry (grown to 15 by Ruling
BI), and `radial_ring.gd`'s sub-ring divides 360° over however many it is
given — 15 wedges at `SUB_RING_RADIUS`/`SSLOT` (Ruling BC's 84px/56px) crowd
past the geometry's own ~9-slot non-overlap ceiling
(`2*84*sin(pi/n) >= 56` fails past n≈9), measured directly off the live
drawn rects at 112/114 `_ctxring_probe.gd` checks (both new legs red, 14
pairwise overlaps logged). Capped to the mockup's own six
(`design/map-context-2026-09-25/Main.dc.html`'s `STYLES` array — literally
`STYLE_PRESETS`' first six rows, unchanged) plus a seventh "All styles..."
wedge (`app.select_domain_category("cartography", "Style")`) for the eight
Ruling BI additions, rather than an invented "most-used" ranking this build
has no telemetry to support. `_ctxring_probe.gd` 114/114 after (fail-first
confirmed: 112/114 before), `_ctxtablet_probe.gd` 67/67 unaffected. Files:
`godot-project/shell/workspaces/cartography_workspace.gd`,
`godot-project/_ctxring_probe.gd`.

*Nautical, ocean-framed.* The row above's own "worth a second pass" note
closed: `_stylepresets_probe.gd` gained `--focus auto`/`--fzoom` (a coarse
`bridge.sample_cell()` scan for an interior, margin-clear ocean cell farthest
from any sampled non-ocean one, re-applied every preset iteration since
`_apply_preset()`'s own `_refresh_map()` → `ViewportHost.refresh()` →
`reset_view()` snaps the camera back each click). Pixel-sampled
(`ocean4_preset_13_nautical.png` vs. `ocean4_preset_01_default.png`, same
framing, seed 483920): Nautical shows a clear green-toward-navy transition
within ~60px of the coastline (e.g. `(101,128,98)` at the shore edge fading
to a stable `(44,71,88)` further out) matching `SEA_RAMP_NAUTICAL`'s own
`0.00 (130,178,122)` → `0.15 (36,78,138)` stops, visibly distinct from
Default's more cyan-toned water at the same pixels — the depth-banded tint
is real and rendering, not a `render.rs::sea_color_core` defect. This
particular crop (and a zoomed-out re-check, `ocean5_preset_13_nautical.png`)
never reaches the ramp's pale-blue/abyss stops (`0.45`/`1.00`): this seed's
visible sea near the scanned point is a strait/inlet rather than open
ocean far from every coast, so "depth" (distance-from-shore-derived, not
literal bathymetry) saturates around the low-mid stops in every crop tried
here. Not re-verified against a world/seed with genuinely open, far-offshore
water. Files: `godot-project/_stylepresets_probe.gd`.

**Cel / Toon style preset — built 2026-09-27, pending independent
verification.** `OUTSTANDING_WORK.md`'s "Cel / Toon" row (owner: *"cel
shading for a bit of a more stylized look 'cartoonish'"*). A new stage, not a
rework of the Painter `D-cel` (`Npr::cel`), which posterises the finished
colour and is pinned by `golden_parity_npr.rs`; its slider is relabelled
"Posterize" so the two are not one name. Two new appearance tunables in
`render.rs`: `toon_strength` bands the **light** — `toon_band` cuts the
combined hillshade into 4 flat steps (`TOON_BANDS`) with a `smoothstep`
terminator 0.03 shade units wide (`TOON_EDGE`), the ladder anchored on the rig's
own flat-ground shade (`toon_flat_shade`, so flat ground keeps its exact light
and exactly 4 levels exist on a full ramp at any sun or under multi-sun) — and
flattens the albedo (`ramp3` position to its middle stop, grain faded, biome
jitter faded, and the six-material blend sharpened by `toon_sharpen_weights`,
power 8); `toon_outline` draws a slate keyline (`TOON_INK`) on the land side of
coasts and lake shores within radius 2 (`toon_outline_cover`, a neighbourhood
water test in `cell_color`, `BakeFields::pixel` and the tile renderer — cells
on screen/export, tile pixels at deep zoom). The ink/`D-ink` stage was not
reused for the outline: it is curvature×slope with an fbm wobble, so it does
not see coasts at all. No separate ridge line: ridges read through the hard
light terminator. New preset "Cel / Toon" (last in `render_workspace.gd`
`STYLE_PRESETS`, gallery only; ring unchanged): the Natural Vibrant look,
`biome_sat` 0.45, `bio_blend` 1, every texture/gradient stage zeroed, bright
blue river (`river_ink` 0.75 toward (40,150,235), width 1.15). `default()`,
`js_reference()` and both named looks are bit-identical on the grid, export and
tile paths: 12 FNV-1a digests measured before the change and pinned
(`tests/cel_toon.rs`, 18 tests); no other preset sets a `toon_*` key. Mutation:
31 mutants, all killed (6 only after tests were added for them). Windowed
(`_stylepresets_probe.gd --cel-only --grid 1024x656`, seed 483920, DLL
`2a2e96de1f8d9411`): flat-land mottle (median 3×3 luma sd) 0.229 vs Default
2.617; grey-light land 99.5% in two levels (this seed's relief only reaches
flat and one shadow step at grid resolution), deep-zoom tile 4 levels holding
94% of its land vs Default's 209 distinct values; full preset loop 0 failures.
**Known:** biome edges now follow the climate raster exactly, which exposes a
~200-cell straight vertical forest edge on this seed that Default's noise
hides; the fit-view river overlay draws white under every preset, Default
included — not this change. Files: `crates/cartalith-godot/src/render.rs`,
`crates/cartalith-godot/tests/cel_toon.rs`,
`godot-project/shell/workspaces/render_workspace.gd`,
`godot-project/_stylepresets_probe.gd`.

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

**`OUTSTANDING_WORK.md` "Make the need to run CARTO ▸ Generate labels visible
on screen" (Ruling AZ), 2026-09-28, verified 2026-09-29.** The
owner's report ("a label fix looked invisible") traced to a documented, already
-registered gap: `engine_bridge.gd`'s own forwarder-audit comment on
`labels_clear_generated` names `sculpt_commit()` as the one path that leaves
CARTO's generated labelling pass stale — it emits `sculpt_draft_changed`, never
`generation_finished`/`world_loaded`, so `cartography_workspace.gd::
_regenerate_labels()` (already auto-run on world build/change and on a class
dial's release) does not re-run, and the map keeps drawing labels placed
against the pre-commit height field with nothing on screen saying so. That
comment explicitly left the re-run itself unwired ("a UI decision, not a
binding decision"); this closes the UI half only, deliberately not the
auto-rerun. `cartography_workspace.gd`'s Labels panel now shows a cue plus its
own "Generate labels" button in two cases, each gated on `bridge.has_world` so
it never appears before a world exists (the engine's own refusal sentence
already covers that case) and never conflates a genuinely empty run with an
absent one — `labels_generate()` answers `ok: false` only when no world has
ever existed, and `ok: true` with five zeroed rows for a world with nothing to
name (`label_bridge/generate.rs`'s own doc comment calls that "a different and
more useful answer than a refusal"), so the cue reads `_label_gen_ran`
(mirroring that distinction), never a count: (1) never generated for this
world, and (2) generated, but a sculpt committed since (a new
`_label_terrain_stale` flag, set by a `bridge.sculpt_draft_changed` listener
and cleared by `_regenerate_labels()` itself). New windowed probe
`godot-project/_labelgencue_probe.gd`/`.tscn` (18/18): the cue hidden before
any world exists, a positive control (a fresh generate's automatic
`_regenerate_labels()` run leaves it hidden), both cue states reachable and
correctly worded, its own "Generate labels" button proven wired (pressing it
clears either state), and the real, reachable trigger — emitting
`sculpt_draft_changed` — shown to raise the stale cue and the button to clear
it. One mutation (inverting the never-generated branch's condition) killed —
6 of 18 checks failed as expected, file hash-verified restored. Regression:
`_ctxcard_probe.tscn` 115/115 unchanged (its CARTO "Add label here…" row is
untouched). `godot --headless --check-only`, from the `godot-project` root,
clean on `cartography_workspace.gd`, `shell/app.gd` and the new probe. **Not
touched**: the "Layers popover" candidate location (`layers_popover.gd`) —
its `LIVE_LAYERS` toggle list carries no "labels" row to attach a cue to, so
the message lives only where the Labels controls themselves are, per the row's
own first candidate.

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
| GLI-E | Erosion's per-cell parts on GPU (`OUTSTANDING_WORK.md` §2.6) — the scope document's milestone **E** (row added here 2026-09-23) | done — thermal only; stream-power declined on measurement, then reopened by Ruling AZ (2026-09-28) as its own track: `GPU_STREAM_POWER_SCOPE.md`, milestones SP-G0…SP-G6, **none started** (scoped 2026-09-29) | 2026-09-23, `08020ee`. `shaders/gpu_thermal.wgsl` and `cartalith_gpu::thermal_grid_gpu_with`, the gather form of `erode_thermal`'s scatter (no atomics). Wired in `cartalith_engine::erode_op` (the Erode button's op, not a `generate_terrain` stage) behind the same `use_gpu` / `gpu_allowed_for_grid` gate as the other GPU stages, with CPU `erode_thermal` as fallback. `ErodeSummary::thermal_on_gpu` reports which path ran. `THERMAL_GPU_TOL = 1e-6` (`erode_op.rs` tests); probe `_thermalgpu_probe.gd`. **Stream-power is not ported, deliberately**: its per-cell phases measured 1.1-1.8% of the kernel, and the rest is serial by construction. `OUTSTANDING_WORK.md` §2.6's row keeps it open as a scoping question |
| GLI-M | Multi-GPU device set, VRAM budgeting and split-tiles warp — the scope document's milestone **M** | done | `cartalith-gpu/src/multi.rs`, 1 653 lines (`wc -l`, 2026-09-24; was 1 291): `MultiGpuMode`, `VramFallback`, `GpuPreferences`, `enumerate_devices()`, `vram_verdict()`, `gpu_allowed_for_grid()`, `device_supports_grid()`, `GpuDeviceSet`, `init_gpu_device_set()`, `split_rows()`, `set_weights()`; `warp_grid_gpu_split` / `warp_band_gpu_with` in `src/lib.rs`. Reached from `cartalith-engine` **before every other GPU stage**, gating the whole GPU path on a VRAM verdict, and from the shell: `menus.gd::_build_gpu_mode_menu()` plus the `gpu_vram_budget_gb` / `gpu_set_vram_fallback` / `gpu_vram_estimate` `#[func]`s. `AlternateFrames` and `ReduceWorkingRes` are deliberately unimplemented variants whose `is_implemented()` returns false |
| GLI-POOL | VRAM buffer pool kept between generations (`HARDWARE_ACCELERATION.md` §14; `OUTSTANDING_WORK.md` §2.6's pooling row; Ruling AZ) | done — **verified 2026-09-29** | 2026-09-27. `cartalith-gpu/src/pool.rs`: `BufferPool` (keyed by exact size + canonical `Kind`), `BufferPoolStats`, `pool_retention_cap_bytes`, `GpuDevice::buffer_pool()`; every `dispatch_gpu_*` in `lib.rs` and `affordance.rs` acquires through it. Retention: current grid only (`begin_grid` flushes on a cell-count change), released with the device (the pool lives on the shared `Arc`), nothing retained on a lost device, capped at `GPU_GRID_BUFFERS` grids, lowered to `budget - working set` when `vram_budget_bytes` is set. Cleared on reuse, derived from the shaders: flow `delta`, weather `rain`, resources' absent optional planes. Tests: 14 in `pool::tests` (per-kernel pooled-vs-unpooled equivalence over two different inputs plus a poisoned-pool pass, the cap, grid change/disable/loss); mutating any of the three clears turns its test red. Measured run alone (`examples/gpu_pool_bench.rs`, RX 7800 XT, medians of 10): allocations per generation 104-106 buffers -> 2; time unchanged at 1024² and 2048² (inside noise; the 1024² sign flipped between two runs), **-2.6% / -2.7% at 4096²** in two runs (10219 ms (10198..10230) vs 10490 ms (10450..10538)). Also: `read_back` now discards a readback from a device that reported loss mid-dispatch, and the engine's GPU plate-id filter rejects ids past the plate count |

**Group total: 16 — 15 done (GLI-POOL verified 2026-09-29), 1 declined.** GLI-POOL added 2026-09-27. GLI-D1 moved not-started → done
2026-09-23; GLI-D2 was corrected to done the same day (it landed 2026-09-21);
GLI-E was added the same day (it landed 2026-09-23 and had no row).

**The erode-recompute determinism defect (`GPU_STREAM_POWER_SCOPE.md` §7) —
fixed 2026-09-29; verified by the main loop (engine 187/0/8, the staleness
tests and the rewritten recovery test re-run green); the workspace ran
3 956/0/42 in the lane.** Root cause, confirmed at
`cartalith_engine::refresh_climate`: it routed `flow_discharge` with the
`rainfall` it was handed (the previous run's), so after an undo that restores
`ws.field` alone the same op recomputed different drainage. Reproduced without
Godot at 64×40: drainage differed on 2 560 of 2 560 cells. The fix runs the
weather first and routes flow with this surface's uncorrected rainfall
(§7p divergence from the reference's `computeFlow(true); refreshClimate();`).
Tests in `staleness.rs`: `erode_undo_erode_recomputes_bit_identical_drainage_and_climate`,
`a_passes_zero_erode_leaves_a_shipped_default_world_bit_identical`,
`refresh_climate_ignores_the_values_it_is_about_to_overwrite`. All three go red
with the fix reverted. `WorldParams::defaults` generation, and so every golden,
is unchanged by control flow. The shipped default (`passes.glacial`) ends in
`refresh_climate`, so its drainage and rainfall do move. That turned
`cartalith-godot`'s `civ_pipeline_tests::recovery_keeps_every_road_on_the_settlements_it_joined`
red (phase 2: 1376.6 vs 1340.2 km). **Settled as a world-dependent premise, not a
recovery defect.** The new world's phase II abandons one network node: a
non-coastal Hamlet (pop 134, placed by `place_settlements`, not an addon village)
with roads to three towns. `civ_apply_recovery` drops any place that is neither
urban nor a port when its scaled population falls under 18. That is the
reference's `_civApplyRecovery` rule. A 12-seed survey at this size found this in
5 of 12 phase-II worlds. The test now states the rule on what is actually
abandoned: no network node abandoned ⇒ rebuilt km == filtered km, else rebuilt ≥
filtered. It runs over two asserted fixtures (seed 12345, and seed 2, where
phase II abandons none). Three mutants of `remap_after_recovery` were all killed:
filter instead of rebuild, drop a way, duplicate a way. The last is caught only
by the seed-2 exact branch.

Still open:
- **Still owed: a windowed Godot probe.**
- **The carve block keeps the reference order on purpose.** With every erosion
  pass off (`WorldParams::defaults`), generation's final climate is still the
  carve block's inline tail in `generate_terrain`. That tail routes discharge
  with the priming rainfall. It is golden-pinned, so it stays. The first
  recompute after such a generation therefore differs from the stored values.
- **Droplet erosion reads stale rainfall after an undo.** `erode_op` spawns
  droplets through `ws.rainfall`, and an undo restores only the height field.
  So an erode with droplets on, run after an undo and before a recompute, reads
  the rainfall of the undone surface. Nothing recomputes stale stages before
  the op.

  **2026-09-29, verified by the main loop** (the bridge test re-run green; the
  main loop's own mutation of the `recompute_stale` line in
  `run_erode_with_recompute` failed it, file hash restored). Fixed at the call site.
  `cartalith_godot::erode_bridge::WorldGen::erode_op`'s recompute-then-op
  sequence was pulled out into a plain function,
  `erode_bridge::run_erode_with_recompute(stages, p, ws, opts)`, which
  `erode_op` now calls; it runs `cartalith_engine::staleness::recompute_stale`
  **before** `cartalith_engine::erode_op::erode_op`, not only after it —
  flushing whatever a previous op (`undo_last`/`undo_revert_to`, which
  restore `ws.field` alone and mark `Height` stale with nothing recomputed)
  left pending, so the droplets spawn through *this* surface's rainfall
  rather than the undone one's. `recompute_stale` only does work when
  something is actually stale, so an already-current world pays nothing
  extra. The extraction exists so the fix is exercised through the **real**
  function, not a hand-rolled copy of its shape — an earlier draft of this
  fix had a test that called `recompute_stale` itself rather than the fixed
  code, which a reviewing pass caught as "never assert a constant against
  itself" in another form; that test is gone.
  Reproduced and fixed without Godot: `cartalith-godot`'s
  `erode_bridge::tests::erode_undo_erode_with_droplets_on_through_the_real_call_path`
  drives `run_erode_with_recompute` directly (generate with the shipped
  defaults, `passes.glacial` on; erode with droplets on; restore the
  pre-erode field and mark `Height` stale via `mark_changed_tiles` — the
  exact shape `undo_last` leaves; erode again the same way) and asserts
  elevation, `flow_discharge`, `rainfall` and `temperature` are all
  bit-identical across the two erodes. Mutation-verified: removing the
  leading `recompute_stale` from `run_erode_with_recompute` itself (not a
  test helper) makes the test go red; restoring it and hash-checking the
  file against `git diff` confirms the restore was clean.
  `cargo test --workspace --no-fail-fast`: 3957/0/42, unchanged in total (one
  test moved from `cartalith-engine` to `cartalith-godot`, where the real
  call path lives).

  **The windowed Godot probe could not be run this pass.** `target/debug/
  cartalith_godot.dll` was rebuilt clean (`cargo build -p cartalith-godot`)
  once the DLL lock cleared. A probe was written,
  `godot-project/_dropletundo_probe.gd` (+ `.tscn`) — real shell path
  (`WorldGen.erode_op({})` with the reference defaults, droplets on,
  through `_app.undo_last()`), `use_gpu` forced off, three erode/undo
  cycles off the same starting field (A vs B is the fix under test, B vs C
  a CPU-vs-CPU control), asserting `sample_cell`'s elevation/drainage/
  precipitation are bit-identical across all three. Launching Godot to run
  it was refused by this session's own permission layer (an "interfere with
  workloads" classifier, distinct from the DLL lock), on the reasoning that
  another lane may have a Godot window open. Not retried past that refusal,
  per this task's own rule against working around a permission denial. The
  probe is written and ready; it has not been run.
  **Run 2026-09-27 by the main loop, with the owner's go-ahead: PASS, exit 0.**
  The DLL was rebuilt 09:18 from committed code. On a real 12 288-cell world, A vs B
  differ on 0 cells in elevation, drainage and precipitation; the B vs C
  CPU-vs-CPU control also differs on 0. Every erode reported
  `climate_coupled: true`.

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
| MEM-16 | Ruling AZ's phone-memory reduction (2026-09-27): release the civ pass's rasters after their last reader | done — **verified 2026-09-29** | `compute_civilisation` (`cartalith-godot/src/lib.rs`): `drop(lithology)` after the resource kernel, `drop(wetland)` after carrying capacity, `suit`/`wb.fill_level` freed after placement unless `seeding_villages`, `coast_sdf` built after the six-grid free beside the explanations, `suitability_ctx!` rebuilding the context there, and the explanation-only rasters dropped before `assign_territory`. Measured on the desktop, `cartalith-civ/examples/_memlane_peak.rs`: 2048 × 1311 peak **519.42 → 486.50 MiB** (202.9 → 190.0 B/cell); 4096 × 2622 2 028.48 → 1 937.46. The phone figure is **projected, not measured**. Bit-identical: 588/588 `CivData` fingerprints match across 42 configurations against `HEAD` `09858bd`, with a positive control. `new_world_dialog.gd::PEAK_BYTES_PER_CELL` is 241.3 → 190.0, and `_nwmem_probe.gd` literals now read 1.90 GiB |

**Group total: 16 — 14 done (MEM-16 verified 2026-09-29), 1 not started, 1 declined.** Corrected
2026-09-23: MEM-10, -11, -13, -14 and -15 each said "not started" for about
three weeks after they landed; each was re-opened at its symbol for this
correction.

§6's walk-down table projects 618.28 → 469.56 MiB for all eight R-changes.
R1-R5, R7 and R8 have landed; R6 has not. Re-measured 2026-09-27, on the
desktop with the pipeline allocator probe: the landed set stood at **519.42 MiB**
at 2048 × 1311, not at the walk-down's 469.56, because the civ pass has since
grown. MEM-16 takes that to **486.50 MiB (190.0 B/cell)**, and the phone figure
is projected rather than measured. Three stages now tie at 484-487 MiB, which
is the floor for a change that moves no value. What lies below that floor, with
sizes, is in `MEMORY_OPTIMIZATION_SCOPE.md`'s *The 2026-09-27 pass*.
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
| LOD-D2 | Colour tiles on screen, and the sharpness bar | partial — built; 3 of 6 D0 bars met | `lod_bridge::synthesize_tile_rgba` calls `render_biome_tile_rgba`, and `shell/lod_tile.gdshader` samples the colour directly. `9d2a800`. Not met at close: zoom-40 detail on the 2048 world (also failing before this milestone), the LOD-entry `mean \|ΔL*\|` bar at both sizes, and the seam ratio at 512. **2026-09-24, verified:** v2.25's `tileShadeExag` is ported (`render::tile_shade_exag`, on in `default()` via `TerrainAppearance::tile_shade_exag_scaled`, off in `js_reference()` so the v2.11 golden is byte-identical). `tests/tile_shade_exag.rs` measures a deep tile's relief against the map's, normalised to LOD entry: before 0.569 / 0.300 / 0.155 at 2 / 4 / 8 px per cell, after 1.095 / 1.139 / 1.169. At LOD entry the tile already carries 2.20x the map's relief on that fixture — departure 2, untouched by this; no D0 bar was re-run. **2026-09-27, seam-ratio bar (Ruling AZ), independently verified:** (the verifier rebuilt into its own target and re-ran the six 512 configs plus 483920/2048/zoompan, all ≤ 1.5, worst 1.388 on 24601/512/pan, where the harness picked a different pivot cell; HEAD's 483920/512/zoompan re-read 1.46/1.52/1.54; the planted seam still trips the bar, 1.02→25.8; frames show the doubled-blend band on the boundary before and gone after) met on all 18 sweep configurations. Two causes, each measured: (1) the colouriser's meso/macro/crest stencils clamped at the tile edge, so neighbouring tiles shaded their shared edge sample differently (removing the meso term alone took 483920/512 pan 1.52→1.20); fixed with a halo (`render::render_biome_tile_rgba_padded`, `render::tile_halo_px`, `bake::pyramid_tile_padded`, `amplify::amplify_region_padded`), shared-edge delta 2.82→0.00 levels in `lod_bridge::tests::neighbouring_tiles_colour_their_shared_edge_sample_alike`; (2) `viewport_host.gd::_lod_tile_rect` now lays a tile over its exact sample span (with `lod_tile.gdshader` pulling UVs in half a texel) instead of a one-texel overlap that mid-morph was alpha-blended twice. The harness's `OVERLAY_LAYERS` also lacked `rivers` and `conflict`; the last over-bar reading (71077345/512 pan, 1.55) was a river stroke's edge one pixel from a tile boundary, not a tile seam. Medians, overlays hidden, before → after: worst 1.5957 → 1.3801, 3 → 0 of 18 over 1.5. JS goldens unmoved; tile pixels within `tile_halo_px` of an edge change |
| LOD-D3 | Continuous transitions: parent fallback and a colour-space morph | partial — 3 of 4 bars met | `lod_bridge::morph_for_zoom`. `b6cc014`. Met: zero pops. Not met: worst level-boundary `T_i ≤ 1.5×` (1 of 12 still over) and zero holes. **2026-09-27, independently verified:** the seam ratio is now ≤ 1.5 on all 18 D0 configurations with the child and parent levels on screen together (see LOD-D2's row), so the count moves from 2 of 4 to 3 of 4 |
| LOD-D4 | Ice and snow from fields that already exist | partial — 1 of 4 bars met | `TerrainAppearance::ice_strength` and `render.rs::apply_ice_cover`. `02f6d51`. **The aspect term Ruling AP (2026-09-23) authorised is built** (`19c38d9`, 2026-09-24; *corrected the same day: this row said not built*): `TerrainAppearance::snow_aspect_c` (2.0 °C shipped, 0.0 under `js_reference()`, so no JS-parity golden moved; the Rust render hashes in `tests/color_space.rs` and `tests/layer_stack.rs` were re-baselined) and `render.rs::snow_aspect_shift`, fed to `material_weights` as a snow temperature shift, with `tile_snow_facing` on LOD tiles. **The snow-versus-aspect bar (1b) is still not met** after it, per the commit's own reading (not re-measured here), so the count stays 1 of 4. `snow_aspect_c` has no GUI control. **Owner question 3 answered by Ruling AU, 2026-09-24 (verified 2026-09-26):** new worlds run the glacial pass (`cartalith_godot::params::defaults()`, the sixth ruled divergence; `WorldParams::defaults` and goldens unchanged). Measured the same day (`tests/lod_d4_ice_and_snow.rs::measure_the_glacial_default`, 2048×1311, CPU path, no other build running): generation median 2.60 s (2.58..2.64) → 3.28 s (3.22..3.30) over 5 alternating runs, re-run 2.59 s (2.57..2.66) → 3.25 s (3.23..3.26). Ice already drew on a default world without the pass (glacier potential gates on the snowline setting, temperature and flow, not on the pass); cells at potential ≥ 0.5 on seeds 24601 / 1337 / 987654 went 29 270 / 696 / 2 257 → 28 153 / 682 / 1 124 with it on. The pass's deepest cut at shipped settings was 0.0007 of normalised height (about 5 m), so the troughs it adds are shallow |
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
| SF-6 | A reopened project is the saved world — the world substrate (owner Ruling AR, 2026-09-24) | done (verified 2026-09-24) | **Built 2026-09-24.** `cartalith-godot/src/substrate.rs` writes every `WorldState` grid the core rasters do not carry — flow, plate id, boundary mask/type, stress, shear, age, resistance, crust, the channel receiver/mask/intensity, the carve lock — as fourteen registered `rasters/` slots (`cartalith_io::SUBSTRATE_RASTERS`) under a `project.json` `substrate` member (`SUBSTRATE_MEMBER`); stream order is rebuilt from `strahler_order.u8`. `project_open` rebuilds the `WorldState` and installs it as `WorldSource::Generated` (`WorldGen::install_reopened_world`), plus the Paint/Sculpt editors a generate gives; it reports `substrate: "complete" / "absent" / "incomplete"`. An archive without it opens `Loaded` as before and every refusal says the save lacks its hydrology and tectonic rasters (`substrate::NEEDS_SUBSTRATE`, `WorldGen::full_world_refusal`) instead of "no civilisation layer". `format_version` stays 2 (`SAVEFILE_COMPAT.md` §8.3). Costs +16.43 MiB at 2048×1311 (16.42 → 32.85 MiB, `measure_a_real_save`). Tests: `project_bridge::substrate_tests` (6, incl. a HEAD-written pre-substrate fixture) and two in `cartalith-io`; probes `_jprestore_probe.gd` (jp_compute, faction economy, military, trade flows, urban layouts and a Sample reading byte-identical after reopen) and `_lodseed_probe.gd` case D. Also fixed: `cartalith-io` parses JSON floats correctly rounded (`float_roundtrip`) — way lengths came back one ulp off. **Since built (2026-09-24, verified):** `CivData::road_edges` is saved as `entities/ways.json`'s `road_edges` (§9.3; absent = none, so earlier projects reopen as before) and restored exactly (`substrate_tests::a_reopened_project_restores_road_edges_exactly`; 71 of 4 449 road cells on the test world come only from it, and a journey over them plans differently without it — `a_reopened_project_plans_over_the_saved_road_edges`); `project_open` builds the Territory tool (`CivTools`) over the restored claim grid, so territory paint and GeoJSON border import work on a reopened project (`_reopentools_probe.gd`); Ruling AT's held year is saved as `history/timeline.json`'s `territory_year` (§10.1); `_jprestore_probe.gd` now also compares the SP-2 journey markers and the landmark set after reopen. **Also 2026-09-24, verified:** an archive without `rasters/territory.i32` reopens with no claim grid (`CivData::territory` empty, §8.1's "No territory") and a `project_open` warning, instead of an all-unowned grid that a re-save then wrote back; the Territory tool's base is "nothing claimed" (`project_bridge.rs::civ_tools_for_reopen`), and the readers that indexed the grid take a partial one as no territory (`civ_reprovince`, `civ_reset_territory_paint`, `CivData::civ_goto_year`, `contested_cell_count`, `civ_faction_territory_stats`, `build_territory_texture`, `sample_refs`, `cartalith_civ::jp_claimed_at`). The manual-placement name stream is saved as `entities/settlements.json`'s `name_stream` (§9.1) and resumed on reopen, so reopened drops no longer repeat pre-save names. `civ_territory_commit` returns whether it committed and `engine_bridge.gd` marks dirty only then, after the engine call. Tests `substrate_tests::{an_archive_without_its_claim_grid_reopens_with_no_territory_and_says_so, a_reopened_project_resumes_the_manual_name_stream}`, `civ_merge_tests::{an_unrestored_claim_grid_is_no_territory_on_every_path, go_to_a_recorded_year_fills_an_unrestored_claim_grid}`; probes `_terrmissing_probe.gd`, `_namestream_probe.gd`, `_terrbool_probe.gd` §2b. **Also 2026-09-26, verified 2026-09-26:** an archive without `rasters/provinces.i32` reopens with the province raster unknown (`CivData::provinces` empty) and a `project_open` warning, not an all-zero grid; a re-save leaves it absent, `build_province_boundary_texture` draws nothing instead of indexing it, and the next territory commit rebuilds it. With no claim grid, every claim-derived readout now reports absent rather than zero: `get_factions()` omits `claimed_cells`; `civ_faction_economy`, `civ_military_summary(_at)`, `civ_faction_relations` and `civ_faction_terrain_fits` rows carry `absent` = `"no_claim_grid"` and omit what the claims feed (territory km², food, resources, exports/imports; `overall` and the manpower model with its land capacity; border, trade, rivalry, value and stance; the terrain mix); `civ_settlement_garrison` says `no_claim_grid`; `manpower_by_faction` returns nothing; `civ_clear_territory` returns `{}` (count unknown) and leaves a whole empty grid; `civ_regional_population` omits `claimed`. The Faction Roster dashes each such figure with the reason. **Also 2026-09-26, verified 2026-09-26:** `civilization_workspace.gd`'s own panels dash the same way rather than default to 0 -- `_fill_faction_economy` (Economy ▸ By faction), `_fill_relationships` (value/stance/border_cells), `garrison_row()`'s new `no_claim_grid` arm, and `_clear_territory_now()`'s status line (`r.has("cleared_cells")`, not `r.get(..., 0)`) all read `absent` first (`OUTSTANDING_WORK.md` §2.11, "CIVIL still prints zeros for keys the engine now omits"). Tests `substrate_tests::an_archive_without_its_province_raster_reopens_with_none_and_says_so`, `claim_grid_tests::an_unknown_claim_grid_counts_as_unknown_not_zero`; probe `_claimsabsent_probe.gd` (54 checks, windowed, on a generated world saved with its substrate and reopened with each raster stripped -- now also reading the CIVIL panels directly, not just the Faction Roster) |
| SF-6 | Save compression — quantising saved rasters to `u16` | declined | **Owner Ruling AJ, 2026-09-23: not added.** Lossy; `PARITY_TESTING.md` and `DECISIONS.md` §7a bar it, and the ruling left that bar where it was. `SAVEFILE_COMPAT.md` §8.1 and §18.4 record the decision |

| SF-7 | A legacy flat `.zip` imports its settlements, labels and icons (owner Ruling AU, 2026-09-24) | done (verified 2026-09-24) | **Built 2026-09-24.** `cartalith-io/src/legacy.rs::read_legacy` reads `state.places` (the six tiers; the reference's four extra classes as `town`), the faction roster, `state.civ.territory`, `state.labels` and `state.mapIcons`, and reports what did not map into `ProjectData::warnings`; `cartalith-godot/src/legacy_import.rs` installs them from `WorldGen::load_save` over a world that stays `Loaded`. Mapping table and the not-imported list: `SAVEFILE_COMPAT.md` §15.5. Tests: `cartalith-io/tests/legacy_records_import.rs` (3) against the empty real export and `legacy_records_seed24601.zip`, a real v2.11 export written by `tools/legacy_records_capture.js`; `legacy.rs` unit tests (4); `legacy_import.rs` tests (4). Probe: `godot-project/_legacyimport_probe.gd` (windowed, through `app._load_project`). **Not imported:** ways, journeys, recorded years, points of interest, painted cartography cells. **Refusal wording, 2026-09-24 (verified 2026-09-26):** a legacy import's substrate-needing readouts refuse with `substrate::LEGACY_NEEDS_SUBSTRATE`, which says what is unavailable and why and does not advise the regenerate that would discard the import (`release_world` drops civ, labels and icons; nothing carries them across). A tree project keeps `NEEDS_SUBSTRATE`, which now also says what a regenerate costs. `WorldGen::loaded_legacy_zip`, set from `ProjectData::legacy` in `load_save`, picks between them through `substrate::needs_substrate`; Force lake's refusal follows it too. Probes: `_legacyimport_probe.gd` and `_jprestore_probe.gd` pin each wording |

**Group total: 7 — 5 done (1 pending independent verification), 2 declined.**
~~Four save slots are written but not yet read back by a caller; a fifth (saved
measurements) was scheduled by the 2026-08-31 rulings.~~ *Stale, re-checked
2026-09-24:* the draft and library slots are read back
(`project_bridge.rs` parses `SLOT_PAINT` / `SLOT_SCULPT` on open; `app.gd`
restores the two library slots through `asset_library_restore_document` /
`travel_library_restore_document`), and the fifth slot exists —
`annotations/measurements.json`, restored by `app.gd`. ~~**What is written and
never read by the product is the stored LOD pyramid**~~ — *read back since
2026-09-24, verified:* `WorldGen::project_open` holds
`ProjectData::lod_tiles` in `LodWorker`, and `LodWorker::tile`/`request` serve
a held tile instead of synthesising it only when its producer string equals
the live snapshot's (`LodSnapshot::producer_id`: `tile_producer_id` plus a
digest of every other tile input, the colour space included). Pyramids saved
before that date have no digest and are never seeded. ~~**A freshly generated
world saved with tiles and reopened is not seeded**~~ — *seeded since owner
Ruling AR's world substrate (2026-09-24, verified 2026-09-24):*
the reopened world used to have no flow or lithology (75 481 of 174 080 sample
pixels differed); a project now stores the substrate (`SAVEFILE_COMPAT.md`
§8.3) and reopens as the complete world with the Paint/Sculpt editors a
generate gives it, so `_lodseed_probe.gd` case D serves 4 of 4 sample tiles,
synthesises 0, and a no-seed session draws the stored pixels exactly (0 of
174 080 differ). A world saved *from a reopened session* is seeded: 341 of 341 tiles served, 0 synthesised, all
byte-identical to synthesis. *2026-09-24, verified:*
**re-saving a seeded project writes the held tiles back instead of drawing them
again** — the save path goes through `LodWorker::pyramid_masks`, which reuses
each held mask whose producer is still the live snapshot's and synthesises only
the rest. `lod_worker::tests::a_re_save_after_a_seeded_open_synthesises_nothing`
(21 of 21 reused, 0 synthesised, tiles byte-identical; a stale-producer control
synthesises all 21) and `_lodseed_probe.gd` section F (341 served, 0
synthesised, 341 of 341 stored entries byte-identical after the re-save).

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
| AND-13 | The Android debug `.so` residue — build the remaining strip (**Owner Ruling AZ, 2026-09-28**) | done — **verified 2026-09-28** | `cartalith-native/Cargo.toml`'s `[profile.android-dev]` now sets `strip = "debuginfo"` and drops `debug`, in place of `debug = "line-tables-only"` alone. Measured 2026-09-27: `android-dev` `.so` 199 937 792 → 35 464 168 bytes (5.6x); `--export-debug` APK came to 68 134 680 bytes; the `.so` inside the exported APK sha256-matches the file `cargo ndk` built. The release recipe was re-run unaffected: release `.so` 29 660 088 bytes, signed release APK (`CN=Godot`) 60 511 545 bytes — in the recipe's usual ~57-60 MB range. **The trade-off, paid as named in `ANDROID_BUILD_SCOPE.md` §1.2 before this ruling: a panic or native crash now names a function but no longer resolves to a source file and line on-device** — that capability existed only from 2026-08-18 to 2026-09-27 and is gone from the shipped `.so`. The desktop dev DLL is unaffected: `cargo build -p cartalith-godot` found it already up to date at 29 582 336 bytes both before and after the `Cargo.toml` edit, confirming `[profile.dev]` itself was untouched |

**Group total: 17 — 13 done (2 of them `done*`, 1 of them verified 2026-09-28), 2 unverified, 2 declined.** *Recounted from the rows 2026-09-27 to add AND-13; this previously said 16.*

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

**2026-09-28, Ruling AQ's last two developer code names — pending independent
verification.** `OUTSTANDING_WORK.md`'s row closed: `data_manager_window.gd`'s
`val_check` route reason no longer says "project_open()" to the user, and
`world_workspace.gd`'s LOD note no longer says "cartalith-spatial" to the
user; both now read in plain terms with the code name kept beside them as a
`##` comment, the same pattern the earlier sweep used. Both files parse-check
clean under `godot --headless --check-only`, and `_codenames_probe.gd` re-ran
with no failures. No behaviour changed.

**2026-09-28, Ruling BO's two phone touch-design rows — pending independent
verification, screenshots await owner review.** `OUTSTANDING_WORK.md`'s "the
planner fix lands the user on a sheet that covers the tab bar" was found
**already fixed** at its symbol: `dcc_shell.gd::_apply_phone_orientation()`
(landed `9dd41c6c`, 2026-09-08) already sets `left_dock`/`right_dock`'s
`offset_bottom` to clear the phone bottom bar, and `_navbarfix_probe.gd` (live
`godot-project/_navbarfix_probe.gd`) passes today, 0 failures, against both
dock sheets; `_leftsheet_shot_probe.gd` (new, this pass) confirms the same
through the real PLAN-tab entry point (`app.gd::open_journey_planner()`).
**No code changed for this row** — the backlog row is stale, not the shell.
The second row, "a drag on the map does nothing until the hand tool is armed,
with no on-screen cue", got a new **third coach-mark toast**,
`pan_drag_hint`, added to `dcc_shell.gd`'s existing `_COACH_MARKS` /
`_show_next_coach_mark()` framework (the same primitive the bottom-bar and
sheet-handle first-run hints already use) — text: "A one-finger drag only
moves the map while Pan (✋) is armed.", centred (`near = null`) rather than
anchored to the navpad ✋, since that button is a private local of
`viewport_host.gd`, a file this pass was not permitted to edit.

**First screenshot round was illegible** (owner-caller review, same day): the
toast read faint with a map label drawn through it. Investigated rather than
assumed — `_show_phone_toast()`'s own style was never translucent
(`DccTheme.panel("raised")` carries no alpha; `text_bright`-on-`raised` is
18:1 in this world's light theme). The actual cause was **the screenshot,
not the toast**: it was captured mid fade-in (`modulate.a` between 0 and 1,
which fades background and text together and lets the map show through). Two
things came out of the fix: `_show_phone_toast()` now also sets
`wrap.z_index = 50`, defensively (tree order already put every toast above
the map by sibling order — `_phone_root`'s FIRST child is the map viewport —
so this was never a live ordering bug, only cheap insurance against a future
map-layer `z_index` change); and `_pandraghint_probe.gd` now waits for full
opacity before sampling anything, and adds two objective checks per toast —
text-vs-background contrast (WCAG relative-luminance formula, >= 4.5:1;
measured 18.00:1 for all three marks) and a background-purity sample (the
rendered pixel must resemble a theme `raised` token, not whatever is behind
the toast). **A real, pre-existing, unrelated finding surfaced along the
way**: at natural cold boot, the FIRST coach mark (`bottombar_tabs`, not
touched by either pass) fires before any world exists, while
`phone_project_picker.gd`'s full-screen picker is still the only thing on
screen — so on every fresh launch that toast paints, correctly, underneath an
already-open modal. Not a regression from this row and not fixed here; noted
for whoever next touches `_maybe_show_coach_marks()`'s call site.

**Pending independent verification, 2026-09-28**: the cold-boot race in the
paragraph above is fixed. `dcc_shell.gd::_maybe_show_coach_marks()` no longer
starts the sequence unconditionally one frame after boot; it now defers to a
new `_start_or_defer_coach_marks()`, which reads `bridge.has_world`
(reflectively) on that same deferred frame — the real precondition
`app.gd::_ready()` itself branches on to decide whether `phone_project_picker`
will open at all — and, if a world does not yet exist, holds the sequence in
a new `_coach_marks_wait_for_picker` flag rather than checking the picker's
own `visible` (which is not yet meaningful at that point; `open()` has not
run). `phone_project_picker.gd::setup()` now forwards its own `Window.
visibility_changed` to a new `dcc_shell.gd::_on_phone_entry_screen_visibility()`,
which starts the sequence the moment that dialog reports closed. **New probe**
`_coachcoldboot_probe.gd`/`.tscn` (`--vp 1080x2340 --force-touch`) drives a
genuine cold boot end to end: shown failing first against the pre-fix code
(`no coach-mark toast exists anywhere while the picker is open` — red, because
the old code created and started `bottombar_tabs` immediately), green after
(0 failures) — a real world generation dismisses the picker
(`bridge.generation_finished`) and the toast is confirmed to appear only then,
both moments screenshotted and looked at directly. Re-run clean alongside it:
`_pandraghint_probe.gd` (18/18) and `_navbarfix_probe.gd` (unmodified, still
green). Not yet inspected by the owner or a separate verifier.

**Pending independent verification, 2026-09-28**: CM-2 follow-ups item (2)
above ("the phone peek-sheet pause is wired but has no probe leg") is closed.
New leg **PP** in `_ctxphone_probe.gd`, built the same way `_ctxring_probe.gd`'s
own leg T already covers the desktop ring/card (`1ca95c5`): starts a live
2-year `CivilizationWorkspace` timeline playing, opens the phone's peek sheet
over an empty cell, and reads the live `Timer.paused`/`time_left` off the real
`_tl_play_timer` node — paused and held (not ticking) while the sheet is open,
resumed at the same point in its interval on scrim-dismiss, with `_tl_playing`
untouched throughout. Full `_ctxphone_probe.gd` re-run: 66/66 green (up from
the prior 56/56 baseline; the new leg adds 10 checks, all passing). Item (1)
of that same row (the per-cell controlling-faction read) still needs Rust and
was left alone. Not yet inspected by the owner or a separate verifier.

Both new probes pass (`_navbarfix_probe.gd` unmodified and still green;
`_ctxphone_probe.gd` re-run clean, 56/56; `_pandraghint_probe.gd` 18/18,
including the new contrast/purity checks for all three marks). Screenshots
(re-captured at full opacity): `pan_drag_hint.png`,
`coach_bottombar_tabs.png`, `coach_sheet_handle.png`, and
`planner_sheet_tabbar.png`, left for the owner under this session's
scratchpad — not yet committed or reviewed.

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

### Map context · `MAP_CONTEXT_SCOPE.md`

**Proposed 2026-09-25; scheduled by the owner 2026-09-26** as
`OUTSTANDING_WORK.md` §2.12. Since CM-1 (2026-09-26, pending independent
verification) a right-click or PH-02 touch hold emits `map_overlay.gd`'s
`context_requested` with every pick under the pointer, and
`shell/context_broker.gd` asks each workspace and `GlobalTools` for rows.
Since CM-2 (2026-09-26, verified 2026-09-26) desktop and tablet
present them in `shell/context_card.gd`, and the right button opens it on
release, not press; the phone still gets CX-01's styled `PopupMenu` as its L4
sheet, with the same rows as under CM-1. The touch hold timing is unchanged on
the phone (`_TOUCH_HOLD_MS` 500; CM-5 moves it). Since CM-4 (2026-09-26,
pending independent verification) the tablet form (`DccTheme.is_tablet()`)
branches at that same 500 ms deadline: the ring and card open together and
the finger stays tracked instead of being swallowed, so a slide can select a
slot. The phone branch below that check is untouched.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| CM-1 | The request and the broker (multi-hit `context_requested`, providers per workspace) | done, verified 2026-09-26 | `map_overlay.gd::hits_at` / `request_at` / `context_requested` (settlement and landmark picks in GDScript; labels and icons through new read-only engine picks `label_pick_all` / `icon_pick_all`, `label_bridge/pick.rs` and `icon_bridge/pick.rs`, which select nothing, unlike `label_hit_test` / `icon_hit_test`). `shell/context_broker.gd` (`build_request`, `collect`, `merge`, `present`). `context_actions` on `GlobalTools` (empty), `WorldWorkspace` (empty), `CivilizationWorkspace` (CX-01's five rows, same text, order, enabled state and `_on_ctx_id` arms), `CartographyWorkspace` (Ruling AX F4's one "Settlement actions in CIVIL ›" row). `map_right_clicked` kept as a shim with no shell consumer. Probe `godot-project/_ctxbroker_probe.gd` (windowed): 32/32, and its A–F rows match the HEAD `fb5611f` run except CARTO's new row; three merge mutants all killed. §4.3's other WORLD/CARTO **B** rows were not built by CM-1: they needed the card (CM-2, row below, which builds those a shell function already backs) |
| CM-2 | The context card (sectioned, disabled-with-reason, replaces the `PopupMenu`) | done 2026-09-26, verified 2026-09-26 | `shell/context_card.gd` (a `PopupPanel`): §4.1's header readout (hit name · class, then `sample_cell` elevation · biome and cell → km), *Select — N objects here ▸* (the nearest 8; a pick re-resolves through `context_broker.gd::reselect`), sections in fixed order, danger rows last behind a rule, disabled rows with their reason drawn under them, inline param rows, one submenu level with ‹ BACK, keyboard (Up/Down/Enter/Left/Right/Backspace/Esc) and type-to-filter. `context_broker.gd::present` picks the card unless `form` is phone. `map_overlay.gd`: RMB now opens on release (under 8 px travel, `_RMB_CLICK_SLOP`; a drag opens nothing, for CM-3). New rows, card only: `GlobalTools` (Centre view here, Measure from here, Copy coordinate ▸), WORLD (sculpt and paint Draft rows, Tool rows for brush size / paint radius / erase, the biome eyedropper, Cross-section from here), CARTO (label Edit text / Delete, icon Delete, Label this, Add label here, Stamp armed icon here with its reason, View field ▸, Style preset ▸, Start export region here). Not built: the Menu key / Shift+F10 opener, the timeline pause under an open card (§9.4), CIVIL's §4.3 additions (its provider was not this lane's file). Probe `godot-project/_ctxcard_probe.gd`, windowed: 115/115 at desktop 1600x900 and at tablet 2560x1600 `--force-touch`; four mutants killed. `_ctxbroker_probe.gd` now reads the card: 32/32, and its CM-1 `ROW` lines are identical to the pre-CM-2 run |
| CM-3 | The desktop ring (RMB-drag marking, RMB-hold, Q) | done 2026-09-26, verified 2026-09-26 | New `shell/radial_ring.gd` (a hand-drawn `Control`, `faction_banner.gd`'s own precedent): the full §6 gesture state machine (no-wait flick, the 150 ms flick-visibility delay, the 300 ms still-hold that opens the ring and the card together, Q's sticky tap vs. aimed hold, one sub-ring, the 24 px dead zone, on-screen edge clamping). `shell/context_broker.gd` gains the ring driver (`ring_press`/`ring_pointer`/`ring_release`/`ring_click`/`ring_is_open`/`ring_key_press`/`ring_key_release`/`ring_close`/`ring_collect`/`ring_domain_req`), installed on `map_overlay.gd` as five `Callable`s (`set_ring_callbacks`, the same seam `set_context_pick_resolver` already uses) so that file stays ignorant of the tool/ring system, exactly as it already is of `EngineBridge`. `GlobalTools.ring_cardinals` (N Inspect / E Measure▸ / S Undo / W Region, per Ruling AX F3) and each workspace's own `ring_slots(req)` (`world_workspace.gd`'s Uplift/Carve/Freehand/Paint over `FEATURE_KEYS`, regrouped by key per `SCULPT_FUNCTION_CHART.md` §2; `civilization_workspace.gd`'s Settle/Territory/Way/Route; `cartography_workspace.gd`'s Label/Icon/View/Style) — every leaf calls the exact function its own dock control already calls (`_on_feature_button_armed`, `_on_paint_layer_changed`, `set_measure_mode`, `_arm_icon_from_ui`, `_infra._set_way_type`, …), no new verb. `app.gd` wires Q press/release and Esc-closes-the-ring in `_unhandled_key_input`. **Two flagged discrepancies, built against the real shell rather than the stale scope text**: `MAP_CONTEXT_SCOPE.md` §5.1's Way row ("road · track · trail · bridge") copies `DCC_SHELL_SPEC.md` §4.5.4, which `infrastructure_workspace.gd`'s own comment already calls wrong against the tested `parse_way_type` enum — built against the real `WAY_DRAW_TYPES` (road/track/sea_lane/ancient) instead; Icon▸'s "Custom" family has no arming index in `icon_arm()`'s numeric API (`ICON_FAMILIES`'s own comment) — built with the 3 families the shell can actually arm. Probe `godot-project/_ctxring_probe.gd` (windowed): 45/45 — the no-wait flick (ring never drawn) and the held-drag path across all 8 directions × 3 domains, a `children` slot's sub-ring picking the right feature key (not just the right tool), a short click still opening the card and never the ring, a still hold opening both together and releasing sticky rather than picking, Q tap (sticky) vs. Q hold (aimed pick), the armed slot's own `armed` flag, the ring's rect on screen at all four edges, and text-contrast on the ring's own ground in both forced palettes. One mutant (the sticky/close branch) killed. `_ctxcard_probe.gd` re-run clean, 115/115 — no CM-2 regression |
| CM-4 | Tablet: hold → ring + card with slide-to-select; pen barrel button verified | **verified 2026-09-26**, 2026-09-26 | `map_overlay.gd`: on `DccTheme.is_tablet()`, the 500 ms still-hold (`_TOUCH_HOLD_MS`, §7.1's own number — unchanged, not re-derived) now opens the ring and the card together (`_touch_ring_active`) instead of swallowing the lift; the finger stays tracked (motion already forwarded to `_ring_pointer_cb` unconditionally) so a slide reaches a slot, and the lift is forwarded to the same `_ring_release_cb` the desktop RMB-hold path uses (slide-to-select / dead-zone-sticky / dismiss, whichever `radial_ring.gd::release()` decides) — never a drag's end. `map_dragged` is suppressed while `_touch_ring_active`, or the armed tool would also see the slide. The phone branch (`_touch_swallow_up`) is byte-for-byte unchanged; CM-5 owns it. `shell/context_broker.gd::ring_touch_open()` opens the ring full-visible immediately (`arm(..., true)`, the same no-flick-delay branch Q's press uses — the wait already happened in `map_overlay.gd`) and opens the card through the same `resolve()` every other presenter uses. Haptics: `app._haptic("sample")` on the hold firing, `app._haptic("tool_arm")` on every hover transition (new `radial_ring.gd` signal `hover_entered`, top ring and sub-ring alike) — both route through `DccShell._haptic()`, the existing haptic table (already shipping `detent`/`back`); its own `OS.has_feature("mobile")` guard makes both a no-op on this desktop build, so **not observed by any probe here, and not claimed as device-verified**. Handedness: new `DccSettings.dominant_hand()`/`set_dominant_hand()` ("left"/"right", default "right", its own `[handedness]` section) — read by `context_broker.gd::present()` to pick which side `context_card.gd` docks on (new `open()` param `dock_side`, new `_target_rect()` branch) for the tablet's touch-hold gesture only; every other presentation is unchanged. **No Preferences menu row writes it yet** — this is the store CM-4's own scope asked for ("a handedness preference, stored"), not a UI toggle; open item for whoever wires `Preferences ▸`. Edge flipping: the ring's existing `_clamp_centre()` (CM-3) already keeps it on screen; the card's `_target_rect()` now tries the handedness-preferred side first and falls back to the other side, same as before, if that side does not fit. **Pen barrel button: wired but not device-verified.** No new code needed — a stylus's barrel button already arrives as `MOUSE_BUTTON_RIGHT` through the same `_gui_input` branch CM-3's desktop RMB ring uses (`map_overlay.gd`'s RMB handling is unconditional on device), so IF Godot reports it that way on the owner's tablet, the pen already gets the full desktop table (§6, flick included) with no further work; this session cannot touch the hardware to confirm it does. New probe `godot-project/_ctxtablet_probe.gd` (windowed, `--vp 2560x1600 --force-touch`): 24/24 — a still hold opens both together with nothing armed by itself; a dead-zone lift leaves both open sticky; a slide onto W (Region, global cardinal) and lift arms it and closes the ring; handedness flips which side of the touch-point anchor the card docks on (measured off `panel_rect()`); the ring's and the card's own rects stay on screen at all four *plate* edges (a touch at the bare *viewport* edge lands on letterboxed paper with no grid cell under it, so the card correctly never opens there — not a bug, verified by leg E's own first assertion). Two mutants killed (the tablet-gate `if`, the dominant-hand dock choice); a third (dropping the hover-transition guard around the haptic signal) survives this probe by design, since haptics are unobserved here — disclosed, not hidden. Regression: `_ctxring_probe.gd` 45/45 (desktop, unchanged) and `_ctxcard_probe.gd` 115/115 at both 1600x900 and 2560x1600 `--force-touch` (tablet) |
| CM-5 | Phone noun surface: long-press → pin → peek card | **done 2026-09-26, verified 2026-09-27** | `map_overlay.gd`: a separate phone-only hold constant, `_PHONE_TOUCH_HOLD_MS := 480` (the tablet's `_TOUCH_HOLD_MS := 500` is untouched — `_process()` picks between the two by `DccTheme.is_tablet()`). The phone's hold branch now drops a **sample pin** (`_sample_pin`, grid space; there was none in this shell before CM-5 — `global_tools.gd`'s own "Not here" note said so — so this is new state, not a build-on) and fires `sample_pin_dropped` (wired in `app.gd` to `_haptic("sample")`, the same pulse CM-4's `ring_touch_open()` fires for the tablet) in the same frame `context_requested` opens the sheet. The pin is drawn every frame (`_draw_sample_pin()`, topmost layer) and is **draggable**: a press within `_SAMPLE_PIN_HIT_RADIUS` of it claims the gesture (`_pin_drag_active`) instead of arming the ordinary hold sequence, and its release re-resolves through the SAME `context_requested` → `context_broker.gd::resolve()` path a fresh gesture uses — no second wiring. New `shell/phone_menu.gd::peek_card()` (peek 66 dp chip row / half full sectioned 46 dp row list, a grab-handle drag between the two, modelled on `dcc_shell.gd`'s own three-detent tool sheet but with two detents and a content-independent 60% cap, since a `ScrollContainer` does not fold its child's minimum size upward the way that measurement would need). New `dcc_shell.gd::phone_present_peek_card()` (mirrors `phone_present_popup()`; skips `_close_all_phone_overlays()` on a re-resolve so a drag's update lands at whichever detent the sheet already had, not reset to peek). `context_broker.gd::_present_phone()` tries the peek/half sheet first, with the old `PopupMenu` kept as a fallback for a harness with no phone chrome. **`global_tools.gd`'s phone gate removed**: `Centre view here` / `Measure from here` (plain rows, no `children`/`param`) now reach the phone too, since a real card-shaped presenter exists to draw them; `Copy coordinate ▸` still does not (it carries `children`, which `_present_phone()`'s existing filter still strips). **CM-5 residuals closed (2026-09-27, verified by the main loop 2026-09-27 (`_ctxphone` 56/56 re-run at 1080x2340 touch))** -- the `OUTSTANDING_WORK.md` row left by `ba01574`: §8.1.4's multi-hit "Select ▸" chip is now built (`phone_menu.gd`'s `_peek_select_mode` / `_open_peek_select` / `_build_peek_select_row` / `_build_peek_back_row` -- a peek chip that forces the `half` detent and swaps the full row list for one row per hit, capped at `context_card.gd::SELECT_CAP` and read off that script rather than re-declared; picking one calls `_peek_reselect` (`context_broker.gd::reselect`), which re-resolves the request to that hit through the SAME `peek_card()` a fresh drop uses). `peek_card()`'s own header now falls back to the picked hit's name when no action sets one, so a reselect to an unheadered row (a label, an icon) still names its object rather than reading "Here". `world_workspace.gd` and `cartography_workspace.gd`'s blanket `req["form"] == "phone"` gates (stale since CM-5 gave the phone a real row-drawing presenter -- `global_tools.gd`'s own gate had already been dropped for the same reason) are removed; the rows that still do not reach the phone (WORLD's Tool `param` rows, CARTO's `View field ▸` / `Style preset ▸`) are excluded by `context_broker.gd::_present_phone()`'s existing `children`/`param` filter alone, each with its reason recorded at the removed gate's old site. `right_dock.gd::on_settlement_selected` now also calls `_show_on_phone()` on a pick (`19d3ba8`'s own mechanism, extended from landmark/icon to settlement), so a settlement tap opens the phone's right sheet instead of changing the dock's context invisibly. `_ctxphone_probe.gd` gained legs SEL / PW / PC / STL, each shown to fail against the pre-residual code before this change (SEL: no Select chip drawn, `peek_in_select()` did not exist yet; PW / PC: read live via `context_broker.gd::collect()` / `last_actions` against a real `ContextCard` opened for comparison, never a constant) -- 56/56 with the residual legs included. Regression re-run clean: `_ctxring_probe.gd` 45/45 (desktop, unchanged). New probe `godot-project/_ctxphone_probe.gd` (windowed, `--vp 1080x2340 --force-touch`): 31/31 — a 400 ms hold does nothing, 480 ms+ drops the pin and opens the peek sheet with its header and chips (CIVIL domain, a settlement fixture), a grab-drag reaches `half` and back, dragging the pin onto empty ground re-resolves the header and chips at the SAME detent, both the peek and the half rects stay on screen (measured px and dp), and a scrim tap (`go_back()`) dismisses the sheet and clears the pin. One mutation (disabling the drag-release re-resolve emit) killed: leg D goes red exactly as expected, everything else stays green. Regression, re-run clean: `_ctxtablet_probe.gd` 24/24 (the tablet's 500 ms path, untouched), `_ctxcard_probe.gd` 115/115 at both 1600x900 and 2560x1600 `--force-touch`, `_ctxbroker_probe.gd` 32/32, `_ctxring_probe.gd` 45/45 |
| CM-6 | Phone thumb fan | **done 2026-09-27, verified 2026-09-27** | New `shell/tool_fan.gd`: an always-shown pill (`_Pill`, built into `_phone_content_gap`, the same clear-of-tool-sheet-and-nav bound `_build_phone_undo_chip()` uses) above the phone's tool sheet. Tap toggles Inspect ↔ the previously-armed tool. Press-and-slide-up (260 ms still-hold, or 12 dp of travel before that timer fires — the canvas's own `pillDown`/`pillMove`) opens a half-fan, drawn by a second, top-level `_Overlay` control parented onto `app` (`radial_ring.gd`'s own ordering trick). Geometry is `design/map-context-2026-09-25/Phone.dc.html`'s own `FAN` table, not the scope's rounded figures (the newer canvas wins): outer arc radius 150, W/NW/N/NE/E at 180/135/90/45/0°; inner arc radius 84, SW/S/SE at 150/90/30°, "distance stands in for the south half" (S sits on N's own ray, just nearer). **The 8 slots are CM-3's own ring data, not a second table**: `ContextBroker.ring_collect(ring_domain_req())`, called fresh on every open, so a slot with `children` (Measure ▸) opens the SAME sub-fan grammar `RadialRing` uses for its sub-ring, and a release with no hover goes sticky rather than closing (`RadialRing.release()`'s own rule, reimplemented for the half-fan's different geometry). One correctness fix over the canvas's own prototype: its `onUp()` tests `if(fan.hover)`, falsy for index 0 in JS, so a drag-release could never pick a sub-fan's first item there — this file tests the index against `-1` instead, the way `RadialRing` already does for its sub-ring. **No collision with CM-5's 480 ms map hold**: the pill is its own `Control`, consuming every event that lands on it, physically separate from `map_overlay.gd`'s surface, with different timing (260 ms vs 480 ms), a different pre-empting slop, and a tap behaviour CM-5's hold does not have. New probe `godot-project/_ctxfan_probe.gd` (windowed, `--vp 1080x2340 --force-touch`): 30/30 — a quick tap never opens the fan and toggles Inspect; a 350 ms still hold opens it; 20 dp of motion opens it early; dragging onto N/W (Inspect/Region, plain leaves) arms them through the real `app.armed_tool`; dragging onto E (Measure, which carries `children`) opens its own sub-fan instead of arming anything, and a second selection there (via the sticky-tap path) arms Measure through the child's own `set_measure_mode()`; a release with no hover goes sticky rather than closing; a sticky tap on a slot arms it; a sticky tap outside every slot dismisses the fan without touching the armed tool; the fan's own bounding rect stays inside the phone viewport (measured px and dp). One mutation (`FAN_HOLD_MS` 260 → 60) killed: leg O goes red exactly as expected, everything else stays green. Regression, re-run clean: `_ctxphone_probe.gd` 31/31 (CM-5, untouched), `_ctxtablet_probe.gd` 24/24, `_ctxring_probe.gd` 45/45. **Not built**: §9.4's "the world pauses under an open surface" rule — grepped and confirmed absent from CM-1 through CM-5 as well (no signal either `PhoneMenu` or `DccShell` fires for a sheet/menu open or close), so this milestone does not add a first implementation of it unasked; carried forward as an open item across the whole CM group, not a CM-6 regression |
| CM-7 | New picks and verbs (landmark, route, stamp; way and river are engine work) | **built 2026-09-28, verified 2026-09-28** (first half verified 2026-09-28) | **Second half (2026-09-28, Ruling BA) -- the two E rows, built:** **Way** -- `WorldGen::way_pick` / `way_get` / `way_delete` in the new `cartalith-godot/src/context_pick_bridge.rs` over all three way stores (`CivData::ways`, `CivData::sea_routes`, `InfraTools::ways`; the reference's one `civWays`, whose way list deletes any of them with `splice`, v2.11 17688). Hit-tested against the drawn curve (`way_render_polyline`), breaks honoured. In `hits[]` via `context_broker.gd::engine_picks` (CIVIL only), so Select ▸ lists it; card rows Inspect / Delete, and a **generated** way's Delete says Generate roads brings it back. A settlement pin takes precedence over the roads ending on it (the way stays reachable through Select ▸) -- found by `_ctxcard`/`_ctxbroker` leg A. **Undoable** (Ruling BA): new `undo::EntryKind::WayDelete` ledger row; the deleted way and its index are held in `context_pick_bridge::WayUndo` and re-inserted by `undo_last()` when that row is newer than the newest height snapshot (`next_undo_step`/`choose_undo`); `can_undo`/`undo_label`/`undo_ledger`/`undo_revert_to`/`undo_stats` (`way_depth`) all see both kinds. An entry is live only while its store's fingerprint chain is intact, so a Generate roads / regenerate / Clear ways in between leaves the row drawn but not reversible, with its reason. Not redoable (the redo tail holds height fields only); a way delete drops the redo tail like any new operation. **River** -- `river_pick` resolves **the branch to its mouth**: nearest channel cell (`stream_order >= 1`) within the pick radius, then channel receivers downstream, continued on the flow-accumulation tree where the channel stops on land, to the first sea or lake cell (else reported as ending inland / at the map edge). Rivers are unnamed, so the hit is described: `River (order N, X km to the sea)`. WORLD rows Inspect river (right dock's River context), Trace downstream, Show catchment (`river_catchment`: every cell whose walk down the same combined tree passes the clicked cell -- D8 alone gave <=3-cell catchments for 7/31/1 of 80 order>=2 cells on three worlds), Clear river highlight; overlay layers in `map_overlay.gd` (trace over the marks, under labels; catchment under the civil layer). Tests: 17 unit tests in `context_pick_bridge::tests` + 2 in `undo::ledger_tests`, literals; 15 Rust mutants, all killed (one, `fp-len`, survived until the addition of `the_fingerprint_sees_a_rerouted_interior`), 4 probe-level mutants (undo index, undo branch, trace draw, catchment cell) all killed. `_ctxpicks_probe.gd` 122/122 windowed, palette forced: generated and hand-drawn way deleted and restored by Edit ▸ Undo with identical points/breaks/type/km/tid/name; revert-to on a way row; Select ▸ narrowing a settlement card to its road; trace ink at 36/36 of the branch's on-screen points (labels hidden; 20/36 with labels drawn over it), 0 before, 0/16 on the river above the click; catchment shading on 37/40 masked samples, 7/5459 unmasked. Regression: `_ctxcard` 115/115 desktop and tablet, `_ctxbroker` 33/33, `_ctxring` 45/45, `_ctxphone` 31/31, `_ctxfan` 30/30, `_awconflict` PASS, `_sp4conflict` PASS. `cargo test --workspace --no-fail-fast` 3942 / 0 / 42. **First half (2026-09-27):** §9.2's cheapest three rows landed: **landmark** (Inspect / Why here? — the pick itself needed nothing new, since CM-1's `hits_at` already carries a `"landmark"` kind; the two rows read `landmarks()`/`landmark_funnels()`, both already **B**, into the status line rather than a second floating popover, CX-01's own precedent), **route** (Open in Journey Planner / Delete — new pick, point-to-polyline distance over `route_get(i).points` run locally in `civilization_workspace.gd::_route_hit`, since `map_overlay.gd::hits_at` is this batch's conflict lane's file and not touched; shared `Workspace.point_to_polyline_distance` static helper, `Geometry2D.get_closest_point_to_segment` per segment), and **sculpt stamp** (Select / Hide / Show / Move up / Move down / Delete — **E-small** closed: new `#[func] fn sculpt_stamp_points(index) -> PackedVector2Array` in `cartalith-godot/src/lib.rs`, the bound accessor `sculpt_list_stamps`'s own doc comment said was missing; `world_workspace.gd::_stamp_hit` buffers each stamp's polyline by its own captured `globals.brush_size / 2`, newest-first so the first hit is the topmost, matching the stack's own read order; every row prefers `right_dock_ctrl`'s own `_on_stamp_*` handler over calling `bridge` directly, so a hide/reorder/delete from the card and from the dock's stack list never disagree). Also closed the **CM-2 residual** row (this task's own brief, four CIVIL rows CM-2 left as optional): settlement-class Tool param (steps `KIND_ORDER`, mirrors `_tool_options_settlement`'s own Class choice), Open city layout… (`app.open_city_viewer`), Start way here / Start route here (arms the tool then places the first point through `_infra`'s own `_way_click`/`_route_click`, exactly `_settlement_click`'s own two-step shape), and Commit territory / Discard territory draft (shown whenever Territory is armed — no bound territory-draft cell count exists to gate on the way Sculpt/Paint's own draft rows do, so this follows the tool-options row's own always-shown rule instead, stated rather than silently matched). **Stopped before the two E rows** (way list/delete; river identity with trace downstream/catchment) **exactly where the brief said stopping was expected**: a "river identity" (which stem, for `CTX_RIVER`) and whether "delete a way" needs undo are both real design questions the scope doesn't settle, not implementation gaps this pass could route around. New unit tests (`cartalith-godot/src/lib.rs::stamp_points_grid_tests`, three, godot-free — `SculptEditor`/`PassBuffer` construct without a `WorldGen`): a three-point stroke's literal coordinates round-trip; an out-of-range index is empty, not a panic; a one-point tap returns one point, not empty (distinguishing "no stamp" from "a tap"). One mutation (`e.stamp.points.clone()` → `Vec::new()`) killed, Python-scripted, restored and hash-verified. New windowed probe `godot-project/_ctxpicks_probe.gd` (`_ctxpicks_probe.tscn`, no extra flags): 37/37 — landmark Inspect/Why-here write the expected text into the hint status line; route Open-in-Planner switches to the CIVIL planner mode on the clicked route and Delete drops `route_count()` by one; stamp Select/Hide/Delete each change the real engine state the row claims (`sculpt_get_selected_stamp`, the `hidden` flag round-tripping through a reopened card, `sculpt_stamp_count()`); the settlement-class param's `+` button steps `_settlement_kind`; Open city layout opens `app.city_viewer_window`; Start way/route here each place exactly one point on a fresh draft; Commit territory raises `civ_faction_territory_stats()`'s `claimed_cells`; CX-01's three original rows still land unchanged. **Two real bugs found and fixed by that probe, not just its own fixtures**: `_route_hit` read `app.viewport.label_px_per_cell()`, which does not exist on `ViewportHost` (the method is on `app.viewport.overlay`, the `MapOverlay`) — silently fell back to the no-viewport tolerance every real run, not just headless ones; and the route row's own name fell back to `"Journey %d"` through `Dictionary.get()`, which never fires because `route_get` always carries a `name` key (blank, not absent) — an unnamed route's row read `Delete ""`. Both fixed in `civilization_workspace.gd`. Regression, re-run clean after both fixes: `_ctxcard_probe.gd` 115/115 at both 1600×900 and 2560×1600 `--force-touch` (its own CX-01 literals updated for the three additive rows — this milestone's own regression test, not a new one), `_ctxbroker_probe.gd` 32/32 (same literal update), `_ctxring_probe.gd` 45/45, `_ctxphone_probe.gd` 31/31, `_ctxfan_probe.gd` 30/30, `_ctxtablet_probe.gd` 24/24. `cargo test --workspace --no-fail-fast`: 3907 passed, 0 failed, 42 ignored. `godot --headless --check-only` clean on every `.gd` touched plus `shell/app.gd`. **Not this pass's**: `map_overlay.gd` itself (conflict lane's file) — no edit was needed for any of the three built picks, since landmark reused CM-1's existing hit and route/stamp both run their own local pick over `req.gx`/`req.gy` rather than through `hits_at` |

**Group total: 7 — CM-1/CM-2/CM-3/CM-4/CM-5/CM-6 done (CM-5 verified 2026-09-27, CM-6 verified 2026-09-27), CM-7 built (first half verified 2026-09-28; second half -- the two **E** rows, way delete with undo and river branch/catchment per Ruling BA -- built 2026-09-28, verified 2026-09-28).**

**`OUTSTANDING_WORK.md` row "CM-2 residuals: the keyboard opener, pausing the timeline, and CIVIL's extra card rows" — built 2026-09-27, verified by the main loop 2026-09-27 (`_ctxring` 132/132 and `_ctxcard` 115/115 re-run).** Re-opened at the row's own three cited symbols first: CIVIL's landmark/route/way/settlement-class/Territory-draft/Open-city-layout/Start-way-route rows named in the row's own text turned out already built (`45c8733`/`3bfbe0f`, CM-7) — only the vault-note row (§4.3 CIVIL Info, "Open vault note / Attach vault note…") and `context_actions`'s stale `PopupMenu` doc comment were still open there, both closed this pass. **Keyboard opener** (§4.2, §6): `app.gd::_unhandled_key_input` gains `KEY_MENU` / `Shift+F10` (checked against every other branch there and against no shortcut clash), routed through a new `_open_context_card_from_keyboard()` that builds the same request an RMB release would (`map_overlay.gd::request_at`, unedited) at the cursor's own last-tracked position, falling back to the overlay's own centre when the cursor is off the plate. **Timeline pause** (§9.4, "the world pauses under an open surface... resumes on close"): new `app.gd::pause_for_context_surface()`/`resume_from_context_surface()`, ref-counted (the ring and card can be open together, §6/§7.1's hold rows) and fanned out by capability to every workspace with the method, never `CivilizationWorkspace` by name. `radial_ring.gd::_active`'s five write sites are now routed through one new `_set_active()` so every open/close transition is covered without re-deriving them per call site. `context_card.gd` connects the `Window`-native `about_to_popup`/`popup_hide` signals. `civilization_workspace.gd` gains `pause_playback_for_context()`/`resume_playback_for_context()`, holding `_tl_play_timer.paused` (never `stop()`, which resets `time_left`) so playback resumes at the same point in its 1200 ms interval, guarded by `_tl_paused_by_context` so a resume never un-pauses a timer this workspace stopped for its own reason. Also wired into the phone's peek/half sheet (`phone_menu.gd`'s three `_peek_open` transition sites, reached through `has_method`/`call` since `_shell` is typed `DccShell`, which does not declare these methods) since §9.4 names the peek sheet explicitly — **not covered by a dedicated `_ctxphone_probe.gd` leg this pass**, only by the existing 56/56 regression re-run (green, no new assertion). The phone thumb fan (§9.4's fourth surface) is not built in this tree (`grep` found no `tool_fan`-style file beyond CM-6's `tool_fan.gd`, which is CM-6's own half-fan, not this row's target — F2 remains unbuilt), so nothing there to wire. **`civ.territory` row genuinely not built**: §4.3's "Territory under cell → Open faction in roster · Claim for this faction" needs the per-cell controlling-faction read the scope itself already flags missing (`sample_cell` does not return it, and `civ_territory_influence()` is aggregate, not per-cell) — the row's own GATING condition ("territory under cell") is unreadable without that engine call, not just one action on it, so nothing was added rather than a row shown unconditionally disabled against a condition that cannot be evaluated. Reported as an open **E** gap, not built. `_ctxring_probe.gd` gained legs **M** (Menu key at the cursor; Shift+F10 with the cursor off the map, both asserted against `context_broker.gd::last_request["screen_pos"]`, never pixels) and **T** (a live 2-year timeline started playing, `Timer.time_left` read before/after a Q-tap-opened sticky ring and after an RMB-opened card, both asserted `paused`/`time_left` against the live `Timer` node) — shown failing first (both legs' checks red, `civ == null` short-circuiting the rest of leg T) against the pre-change code, green (132/132) after. `_ctxbroker_probe.gd` leg A and `_ctxcard_probe.gd` leg A (CIVIL's own regression tests) updated for the added vault-note row and re-run green (33/33, 115/115 desktop and tablet). `_ctxphone_probe.gd` re-run green, 56/56, no regression. `godot --headless --check-only` clean on every `.gd` touched (`app.gd`, `context_card.gd`, `radial_ring.gd`, `civilization_workspace.gd`, `phone_menu.gd`, `_ctxring_probe.gd`, `_ctxbroker_probe.gd`, `_ctxcard_probe.gd`). No `.rs` file touched, `cargo test --workspace` not re-run for that reason. **Not yet inspected by the owner or a separate verifier.**

**`OUTSTANDING_WORK.md` §2.11's "Two undo quirks outside the way work" — fixed 2026-09-29, Rust tests verified by the main loop 2026-09-29; the Godot probes are the lane's own, since the main loop could not rebuild the DLL (held by a stuck Godot process).** Both found 2026-09-28 building `3bfbe0f`, both pre-existing, neither a CM-7/Ruling BA regression.

**CM-4's own two residuals — the touch-sized ring and the Preferences handedness row — closed 2026-09-27; verified by the main loop (`_ctxtablet` 28/28 and `_ctxring` 46/46 re-run, the card-clear screenshot inspected).** `shell/radial_ring.gd`: new `_apply_touch_scale()`, run once from `setup()` (the same "decided once at boot" shape `DccShell._compute_layout_mode()` already uses for `_phone`/`_phone_scale`). On `DccTheme.is_tablet()` it turns `RING_RADIUS`/`SLOT_SIZE`/`DEAD_ZONE`/`SUB_RING_RADIUS`/`SUB_DEAD_ZONE` into live `_r_ring`/`_r_slot`/`_r_dead`/`_r_sub_ring`/`_r_sub_dead` fields that every drawing and hit-test function now reads instead of the bare consts. `DccTheme.TOUCH_SCALE` (1.53, this shell's one documented general-purpose touch multiplier) is reused rather than a new ring-only factor — plain multiplication only reaches 91.8 px off the desktop 60, short of §7.2's own named 96 dp target, so `_r_ring` alone additionally floors at 96.0, the same "scale, then never let a named figure fall short" shape `DccShell._ptap()` already uses. Desktop is untouched (`is_tablet()` false, the scale step no-ops). `shell/menus.gd`: new `Preferences ▸ Handedness` (`_build_handedness_submenu`, IDs 717/718 — the two free slots above `ID_VAULT_BROWSE`'s 716), a plain Left/Right radio submenu built the same shape as the neighbouring Theme/Units ones, unconditional in `_preferences()` so both the desktop menu bar and the tablet's `☰ ▸ Preferences` reach it; writes through the existing `DccSettings.set_dominant_hand()`. **The stored default, unchanged and now reachable rather than assumed**: `dominant_hand()` reads `"right"` when nothing has ever been written — a fresh install or a build predating this row both read as right-handed, not as an absent/blank state. `debug_state()` gained `radius`/`slot`/`dead`/`sub_radius`/`sub_dead`, the LIVE geometry, so a probe measures the shipped node rather than re-declaring the constant it is supposed to check. `_ctxtablet_probe.gd`: new leg **T** (`live_radius >= 96.0` and `live_radius > RING_RADIUS`, both against `debug_state()`), leg E's own `pad` now reads the same live fields instead of the desktop consts it had silently under-sized itself with, and leg D now saves one framebuffer PNG per hand (`_ctxtablet_ring_right.png`, `_ctxtablet_ring_left.png`, `godot-project/`) alongside its existing dock-side assertions — the only "ring edge flipping/handedness" surface that exists is the card's dock side (`context_broker.gd::present()`), since §7.2 names only screen-edge clamping for the ring itself; there is no separate ring-side handedness branch to test beyond what leg E already covers. Measured (`--vp 2560x1600 --force-touch`): live tablet ring radius **96.00 px** (floor engaged, since 60×1.53 = 91.8); desktop ring radius unchanged at 60 px.

**Ruling BC (`LARGE_ITEM_RULINGS.md`, 2026-09-29): the ring's style and shape now match the mockup 1-to-1, in the theme's own colours — verified by the main loop 2026-09-27 (`_ctxring` 68/68, `_ctxtablet` 44/44, `_ctxfan` 49/49 re-run; light and dark screenshots inspected).** `shell/radial_ring.gd` gained the pieces the owner's screenshot review found missing: a 30%-black scrim behind an open ring, a `DccTheme.c("panel")`-toned disc drawn BEHIND the slots (88% opaque main / 94% sub, mockup's own figures, never the mockup's hex), an annulus hover wedge toward the hovered slot (`accent_wash_2`), a real CLOSE/BACK centre button with a glow ring when a sub-ring is open, and glyph+label moved INSIDE each slot circle (both main and sub) instead of hanging below it. `shell/tool_fan.gd` (the phone thumb fan) got the same label-inside-slot move plus the sub-fan's own accent-filled, glowing BACK button with its label.

**Coordinator correction, same day: "the shape is not 1-to-1 yet."** The first pass matched the mockup's own *ratios* onto this shell's OLD, independently-tuned 60 px ring, which put every main-ring label straddling its own slot's border. Fixed by taking the mockup's ABSOLUTE desktop figures literally -- `RING_RADIUS` 92, `SLOT_SIZE` 60, `DEAD_ZONE` 22, `SUB_RING_RADIUS` 84, `SUB_DEAD_ZONE` 32 (`design/map-context-2026-09-25/Main.dc.html`'s own `RR/SLOT/DEAD/SUBR/SUBDEAD`) -- so `SUB_SLOT_RATIO` (56/60) and `CENTRE_RATIO` (46/60) now resolve to the mockup's own literal 56 and 46 rather than approximating them; `_apply_touch_scale()`'s `maxf(96.0, ...)` floor is unchanged and now inert in practice (92 x 1.53 = 140.76 already clears it). Every label (main and sub, both rings, the phone fan) now fits through a new wrap/shrink function, `_draw_fitted_slot()` (`radial_ring.gd`, duplicated into `tool_fan.gd`'s own `_Overlay` class rather than shared, same shape as every other geometry const there): the mockup's own 9 px mono first, a wrap onto a second line at a space or `/` near the middle (`"Cliff / Escarpment"` -> `"Cliff /"` / `"Escarpment"`) when that's too wide for the circle's own chord at that height, and a shrink down to 7 px if even the wrapped line still doesn't fit. A second, related defect the coordinator's screenshots caught along the way: the main ring's own slots stayed at full opacity behind an open sub-ring (the mockup dims every slot but the sub's own parent to 30%) -- fixed in `_draw_ring()`'s new `sub_dir` parameter, which is also what let a dimmed-but-still-legible main-ring label register as a sub-slot's own label overflowing its circle before the fix. The tablet's own sub-ring screenshot pair had the same "hover wedge, not an opened sub-ring" gap the coordinator flagged for it (no `release()` was ever sent, so `_open_sub()` never ran) -- both the screenshot capture and the new tablet leg **L** now aim at the LIVE `debug_state()` radius and release there, same as `_held_drag()`'s own desktop path.

Probe counts after the correction: `_ctxring_probe.gd` (updated geometry consts, leg **G** unchanged, new leg **L** -- 15 checks, main ring x8 + sub-ring x6 + one precondition, each measured from the live drawn pixels): **68/68**. `_ctxtablet_probe.gd` (fixed geometry consts, fixed sub-ring screenshot gesture, new leg **L** -- 8 main + up to 6 sub): **44/44**. `_ctxfan_probe.gd` (new leg **L**, main x8 + sub x6, using a scrim-prediction technique rather than a hover diff, since a sticky sub-fan has no live hover state to diff against): **49/49**. All three re-run together in one clean pass after a transient, unrelated crash in `map_overlay.gd::_draw_rivers` (the concurrent rivers/lakes lane's own in-progress work, calling a not-yet-rebuilt Rust binding) resolved on its own. Screenshots (desktop dark/light, desktop dark/light with a sub-ring open, tablet right/left-handed, tablet sub-ring dark/light, phone sub-fan dark/light) re-inspected by the agent after the correction; still not yet inspected by the owner or a separate verifier.

**Coordinator review, same day, caught a second defect from the two screenshots above: the card overlapped the ring's own slots.** `context_card.gd::_target_rect()`'s dock offset was a bare 10/6 px, right only for the plain-click case with no ring open (CM-2) — CM-3's desktop RMB-hold and CM-4's tablet touch-hold both open the ring and the card at the SAME anchor, and nothing had ever widened that offset to clear the ring's own footprint. Confirmed present at BOTH densities before fixing (temporarily forced `ring_clear` to 0.0 and re-ran both probes: `_ctxtablet_probe.gd` 26/28 RED, `_ctxring_probe.gd` 45/46 RED, both failing only the new overlap leg) — so the 96 dp ring made an existing desktop defect easier to see, not a new one. Fix: `context_broker.gd::present()` now reads the open ring's own LIVE `debug_state()` (`radius`/`slot`) into a new `ring_clear` passed through `context_card.gd::open()`'s new parameter (default 0.0, so the plain-click path is byte-for-byte unchanged) into `_target_rect()`'s offset (`10.0 + ring_clear` in place of the bare `10.0`) — `radius + slot * 0.5` is the same "farthest any slot's own bounding square reaches from centre" term `radial_ring.gd::_clamp_centre()` already computes into its own `pad`, reused rather than a new formula. Widening only the X offset is sufficient and proven so in the comment at the call site: once the card's entire x-range sits outside `anchor.x ± ring_clear`, every point in the card is farther than `ring_clear` from the anchor for any y, so the existing y offset needed no change. New leg **O** in both probes (`_card_overlaps_ring()`, built from the same live `debug_state()` slot geometry, never a re-declared constant) asserts the card's `panel_rect()` intersects none of the ring's 8 live slot rects. Re-run after the fix: `_ctxtablet_probe.gd` **28/28 GREEN**, `_ctxring_probe.gd` **46/46 GREEN** — both PNGs (`_ctxtablet_ring_right.png`, `_ctxtablet_ring_left.png`) regenerated and inspected: the card now sits fully clear of the ring on both hands, and the ring's own caption/slots are unobstructed. `cargo test --workspace --no-fail-fast`: unaffected (no `.rs` file touched by either pass) — last measured 3957 passed, 0 failed.

**Follow-up (`OUTSTANDING_WORK.md` row "the context card can overlap an open sub-ring"), verified by the main loop 2026-09-27 (`_ctxring` 82/82 and `_ctxtablet` 56/56 re-run; sub-ring screenshot inspected).** The fix above only ever widened the card's dock offset for the TOP-level ring, centred at the same anchor as the card. A sub-ring re-centres on its own parent slot (`radial_ring.gd::_open_sub()`), up to the ring's own radius away from that anchor — reachable past the card's own clearance in a different direction. `context_broker.gd` gained `_ring_clearance()` (reads `debug_state()`'s `sub_centre`/`sub_radius`/`sub_slot_size` alongside the existing `radius`/`slot`, zero when no sub-ring is open) and `_sync_card_ring_clear()`, called from every ring path that can call `radial_ring.gd::_open_sub()` (`ring_release()`, `ring_click()`, `ring_key_release()` — the only entry points, since a sub-ring never closes without the whole ring closing). `context_card.gd::open()` gained `sub_x`/`sub_clear` parameters and a new public `set_ring_clear()` that re-clamps an already-open card in place (a translate, not a rebuild) the moment a sub-ring opens beside it, covering both orders (opened after the card, and the card opened with a sub-ring already up). `_target_rect()`'s obstacle reasoning is generalised from one anchor-centred square to the union of both obstacles' x-ranges — the same "x-range alone clears the shape for any y" proof, now applied to two circles instead of one. Confirmed present before the fix: a new leg **O** follow-up in both probes (`_card_overlaps_sub()`, off the sub-ring's own live slot rects) went RED on the pre-fix code — `_ctxring_probe.gd` 81/82 (only the new sub-overlap check failing) and `_ctxtablet_probe.gd` 54/56 (the new check, plus leg **L**'s own sub slot #4 "Cliff / Escarpment" failing once its card-closed workaround was removed, both at the SAME card corner). After the fix: `_ctxring_probe.gd` **82/82 GREEN** (new leg O exercises both WORLD children slots, NW=Uplift and E=Measure), `_ctxtablet_probe.gd` **56/56 GREEN** (new leg O exercises both hands). The tablet leg **L** workaround ("card closed before sampling," dating to the first sub-ring pass) is removed — the card now stays open through both label-sampling gestures and still passes, including the previously-hidden sub #4 case. Screenshots regenerated and inspected: desktop (`_ctxring_sub_NW.png`, `_ctxring_sub_E.png`) and tablet (`_ctxtablet_ring_sub_dark.png`, `_ctxtablet_ring_sub_light.png`) all show the card fully clear of the open sub-ring's own slot circles, both palettes. No `.rs` file touched (the concurrent rivers/lakes lane owns `lib.rs`/`cartalith-hydrology` this session) — `cargo test --workspace` was not re-run for that reason, to avoid contending with that lane's own DLL rebuild; `godot --headless --check-only` clean on every `.gd` touched (`context_broker.gd`, `context_card.gd`, `_ctxring_probe.gd`, `_ctxtablet_probe.gd`) plus `app.gd`. Inspected by the agent that made the fix, not yet by a separate verifier or the owner.

(a) `lib.rs::undo_revert_to` never marked the height stage stale after a multi-step revert — `undo_last`'s own doc comment says why that matters ("Undo was the only height mutator in the crate that wrote `ws.field` and left the staleness graph saying nothing had changed"), and `undo_revert_to` had the identical gap, just never closed alongside it. Fixed by adding the same `self.stages.mark_changed_tiles(PipelineStage::Height.id(), 0..n, "undo_revert_to")` call `undo_last()` already makes, once per successful revert rather than once per popped step (a multi-step revert-to restores one arbitrary earlier field, same as `undo_last`'s own single pop). Before the fix: `stale_stages()` reported nothing after a `undo_revert_to()` that popped real height snapshots. After: `hydrology`/`climate`/`civ` all report stale with `reason == "undo_revert_to"`. Verified with a new windowed probe, `godot-project/_undofixes_probe.tscn` (`-- --vp 1080x2340 --force-touch`; also runs headless as `_undofixlegA_probe.tscn` for leg A alone) — two height edits committed, the graph settled clean, revert to the first edit's own ledger row (which pops both, landing at the pre-edit floor — the floor row itself is never a valid `undo_revert_to` target, since `EntryKind::Floor` is never "live"), then `stale_stages()` read before and after: `GREEN, 11/11` combined (7/7 for leg A alone, headless).

(b) The phone Undo chip (`dcc_shell.gd::_do_phone_undo()`) called `bridge.undo_last()` directly instead of the wrapped `DccApp.undo_last()` (`app.gd`) the menu bar's `↶` square and Ctrl+Z use — so it reverted the height field but skipped that wrapper's repaint (`viewport.map_view.texture = bridge.color_texture()`, `invalidate_lod_tiles()`), its status line and its History-dock refresh. Fixed by reading the toast label with the pure preview query `bridge.undo_label()` *before* the call (since `DccApp.undo_last()` returns nothing to build "Undid: %s" from), then dispatching through `self.undo_last()` (`if has_method("undo_last"): call("undo_last")`, the same base/subclass guard `_menu_bar_undo()` already uses) rather than the raw bridge call; the redundant `notify_ways_changed()` call this chip used to make for a "civ" kind is removed, since `DccApp.undo_last()` already makes it. Verified in the same probe's leg B: a fresh height edit committed, the viewport's drawn texture read before/after `_do_phone_undo()`, and compared against `bridge.color_texture()`'s own bytes read immediately after (the oracle) — POSITIVE CONTROL (the drawn texture actually changed, 147 456 bytes either side) and correctness (the after-texture is byte-identical to the oracle, not merely different).

`cargo test --workspace --no-fail-fast` (same batch as the MM-6 fold-in above): **3953 passed, 0 failed, 42 ignored.** `godot --headless --check-only` clean on `dcc_shell.gd`, `app.gd` and every `.gd` probe touched. Mutation intuition, not a scripted mutant: reverting either fix (dropping the `mark_changed_tiles` call, or reverting the phone chip to call `bridge.undo_last()` directly) reproduces exactly the failure the probe's own "the bug" comments describe, since that is the pre-fix code this batch replaced.

**`OUTSTANDING_WORK.md` row "CM-3 residuals: one-gesture sub-ring picks, the Custom icon family, and the scope's stale Way row" — closed, verified by the main loop 2026-09-27 (`_ctxring` 112/112, `_ctxtablet` 67/67, icon tests 81/0 re-run).** All three parts, against the real shell at the symbol, not the row's own text:

1. **Continuous nested flick** (`shell/radial_ring.gd`). `_update_hover()` gained the mockup's own `ringHover` gate, `dist > RR + 18` (new `PUSH_PAST := 18.0`, touch-scaled into `_r_push_past` alongside every other ring figure): dragging PAST a `▸` slot — not just onto it — now calls `_open_sub()` directly out of `pointer()`, mid-drag, with the RMB (or the tablet's tracked finger) still down, instead of only ever opening on `release()`. `release()`/`click_at()` are otherwise untouched, so a sub-ring opened this way still picks on hover+release and still goes sticky on a dead-zone release exactly as before — one state machine, reached from a second place, not a second one. `context_broker.gd::ring_pointer()` now also calls `_sync_card_ring_clear()` (previously only `ring_release()`/`ring_click()`/`ring_key_release()` did, since only they could reach `_open_sub()`) — the card can no longer sit stale beside a top-level ring's footprint once a drag alone re-centres it onto a sub-ring while the card is already open (§6's still-hold path). Verified on both the desktop RMB drag and the tablet's touch-hold-then-slide (the SAME `radial_ring.gd::pointer()`/`release()` code, reached through `map_overlay.gd`'s `_touch_ring_active` branch, which forwards every touch move to the same `_ring_pointer_cb`) — confirmed at the symbol, not assumed from the shared file.
2. **The Custom icon family** (`shell/workspaces/cartography_workspace.gd`; `cartalith-godot/src/icon_bridge.rs`, `lib.rs`). New `IconEditor::arm_custom(slot, set, scale, rotation, jitter)` builds the `ArmedIcon{family: Custom, slot, set: Some(set)}` the numeric `arm()` path's own doc comment says it cannot address; new `#[func]`s `icon_arm_custom` (validated against the LOADED pack's own `manifest.custom_paths(set, slot)`, not the editing-session library) and `icon_custom_slots` (every `(set, slot)` the loaded pack actually carries at least one image for). The Icon▸ sub-ring's fourth entry, Custom, arms the first such pair on click, the same "click arms this family's own first slot" shape the other three entries already have; disabled with `"no custom icons imported"` when the pack has none (§4.2's disabled-with-reason rule). New `_icon_custom_armed` field on the workspace makes `_arm_icon_from_ui()` re-arm the SAME custom icon after a tool re-arm or a world regenerate, the same persistence the numeric family/variant already got — without it, `_on_any_tool_armed`'s own `_arm_icon_from_ui()` call would have silently clobbered a ring-armed Custom icon back onto the last numeric family the instant `app.arm_tool("icon")` fired its `tool_armed` signal (found and fixed in this pass, not shipped and found later).
3. **The stale Way row** (`MAP_CONTEXT_SCOPE.md` §5.1; `civilization_workspace.gd`'s own comment). The Way▸ sub-ring itself was already correct — `_ring_way_group()` already read `infra_tools_bridge::parse_way_type`'s real, tested vocabulary (`infrastructure_workspace.gd::WAY_DRAW_TYPES`: road / track / sea_lane / ancient) — a prior pass had found and flagged the mismatch but, correctly, would not silently edit a scope document it did not own. §5.1's row now reads **Road · Track · Sea lane · Ancient**; the flagging comment in `civilization_workspace.gd` is updated to say so rather than still describing the row as stale.

Probes: `_ctxring_probe.gd` gained legs **K**/**K2** (one continuous RMB drag through Uplift's sub-ring onto Mountains, released once — armed tool and live feature both asserted; a second continuous drag released in the sub-ring's own dead zone stays open and sticky, unchanged), **V** (CARTO's View and Style diagonals picked through a held-drag + sticky click — `app.viewport.debug_view()` and `bridge.look()` read back as the oracle), **U** (Icon▸ Custom: disabled-with-reason before any custom icon exists; a real slot imported and `as_apply_to_map`'d; the entry becomes enabled and picks through the ring into `icon_armed()` reading `family: "custom"` with a real, non-empty `slot`/`set`), and **W** (the Way▸ sub-ring's own labels read exactly `["Road","Track","Sea lane","Ancient"]`, and picking Sea lane through the ring sets `_infra._way_type == "sea_lane"`). **112/112 GREEN** (up from 46 before this batch). `_ctxtablet_probe.gd` gained the same **K**/**K2** pair on the touch-hold-then-slide gesture (a new `_at_to_local()` helper was needed here — this file's other legs compute their touch points from the probe's own local `centre` var and never needed it, but a sub-ring's `debug_state()` position is in the transformed "at" space, which this tablet-sized viewport's own transform is NOT the identity of, unlike the desktop probe's smaller one; caught by leg K itself landing on sub-item #2 instead of #0 before the conversion was added). **67/67 GREEN** (up from 56).

**Fail-first, shown, not just claimed.** Leg K/K2 (both probes): toggled `_update_hover()`'s new gate to `if false and …`, re-ran — desktop **106/112** (6 of the new K/K2 checks RED, all and only the ones the continuous-flick change is meant to fix), tablet **59/67** (8 of the new K/K2 checks RED) — then restored and re-ran both back to green. Leg U (desktop only): toggled `cartography_workspace.gd`'s `icon_custom_slots()` read to a forced `[]`, re-ran — **108/112** (4 RED: the Custom entry never enables, the ring pick never arms `icon`/`custom`, no real `slot`/`set`) — then restored and re-ran back to green. Leg V and leg W exercise code that was already correct (View/Style were already wired; the Way vocabulary was already the real one), so they have no pre-change failing state to demonstrate — they are coverage additions, stated as such rather than forced red.

`cargo test --workspace`: **all crates green**, no test count regression (this pass's two `#[func]`s and one `IconEditor` method added no new `#[cfg(test)]` cases of their own — the existing `icon_bridge` unit suite already exercises `arm`/`resolve_variant`, and `arm_custom`'s only new branch, the empty-`slot`/`set` reject, mirrors `arm`'s own already-tested shape closely enough that a dedicated case was judged redundant; flagged here rather than silently assumed covered). `godot --headless --check-only` clean on `radial_ring.gd`, `context_broker.gd`, `cartography_workspace.gd`, `civilization_workspace.gd`, `engine_bridge.gd`, `app.gd`, `_ctxring_probe.gd`, `_ctxtablet_probe.gd`. Screenshots (`_ctxring_dark.png`, `_ctxring_sub_dark.png` and the tablet's own sub-ring pair) regenerated and inspected by the agent: the Uplift sub-ring opens correctly re-centred with its accent BACK button, and the tablet's touch-scaled sub-ring sits beside the context card with no overlap. **Not yet inspected by the owner or a separate verifier.**

**Owner report 2026-09-27, "the POI markers aren't clickable for more information in the right pane" — fixed 2026-09-27, verified by the main loop 2026-09-27 (`_poiclick` 12/12 desktop and 13/13 phone, `_lakelabel` 7/7, the crescent-lake test and the `cartalith-godot` lib suite 762/0 re-run).** Measured first with real input pushed into the SubViewport (`godot-project/_poiclick_probe.gd`, windowed, seed 552017, 512x384, landmark pass + all four automatic-placement families): a landmark ring already opened the Landmark context on desktop, but (a) every drawn **icon glyph** — PLACES squares, TREES triangles, SEA MARKS targets, POI diamonds whose landmark is gone, hand-placed icons — had no left-click target at all (the click fell through to the river pick, dock context `river`), and (b) on the **phone** a landmark tap set the right dock's context and put nothing on screen, because the phone's right dock is the closed right sheet. Now: `map_overlay.gd::_hit_test_icon` (radius from the new shared `_icon_radius`, the same one `_draw_icon_glyph` draws with; refuses hidden and ring-shadowed icons exactly as `_draw_annotation_marks` does) joins `_pick_mark` (`{s, l, i}`, nearest centre, ties settlement → landmark → icon) and a new `icon_selected` signal, forwarded by `viewport_host.gd` and `app.gd::_wire_selection`; `right_dock.gd` gains `CTX_ICON` (`on_icon_selected`, `_build_icon`: Kind, Family, Placed by, Cell, Scale, Elevation, Biome, and "Placed for" — the settlement or landmark at the icon's cell with a Show button, or a note saying why there is none); `right_dock.gd::_show_on_phone()` opens the right sheet through the new `dcc_shell.gd::phone_show_right_sheet()` after a landmark or icon tap. On the phone, "right pane" = that right sheet. Probe: desktop 1600x1000 **12/12**, phone `--vp 1080x2340 --force-touch` **13/13**; the pane's text is checked against `bridge.landmarks()` / `bridge.icon_get()`. Mutants: dropping the icon arm of `_pick_mark` and dropping `_show_on_phone()` from `on_landmark_selected` both go red. Screenshots `poiclick_{desktop,phone}_*.png` in the lane's scratch folder, inspected by the lane only.

**Owner report 2026-09-27, "sometimes lakes get labelled when there isn't a lake visible on the map" — two causes fixed 2026-09-27, verified by the main loop 2026-09-27 (`_poiclick` 12/12 desktop and 13/13 phone, `_lakelabel` 7/7, the crescent-lake test and the `cartalith-godot` lib suite 762/0 re-run). No `buildWaterBodies` golden moved.** (1) `label_bridge/generate.rs::labels_generate` named lakes off `CivData::water_bodies`, a copy taken in `compute_civilisation` and refreshed by nothing short of `recompute_civilisation`, while the map redraws lakes from the live heightfield on every texture build. So after a sculpt or erode, pressing Generate labels (the cue Ruling BB relies on) named the pre-edit lakes. This is **not** Ruling BB's expected staleness: the stale input survived the user's own regenerate. Both now read the new `lib.rs::WorldGen::drawn_water_classification()`, which `build_color_texture` also uses, so a label cannot name a lake the texture does not draw; a loaded save, which has no civ layer, now gets lake labels too. Probe `godot-project/_lakelabel_probe.gd` (headless; digs a pit in dry ground, commits, regenerates labels, expects the new lake named) **7/7**; with the old civ-copy line restored and the DLL rebuilt it fails P6. (2) `cartalith_civ::labels::lake_label_anchors`: a crescent or branched lake's centroid lies on land. Measured 2 of 93 named lakes on 16 worlds (`WorldParams::defaults(512, 320, seed)`, eight seeds, bounded and wrapped). Such a label now snaps to the lake's own cell nearest the centroid, and a compact lake keeps its centroid exactly. Test `a_crescent_lake_is_labelled_on_its_own_water_not_its_centroid`; both of its mutants are killed. `cargo test --workspace --no-fail-fast`: **3958 passed, 0 failed, 42 ignored**.

**"Four map-data defects found while fixing lake labels and POI clicks", items 1-3, and Ruling BO "draw forced lakes" -- built 2026-09-28, verified by the main loop 2026-09-28 (`_mapdata` 26/26 re-run; forced-lake z16 screenshot opened; the staged tree built and tested alone in a worktree).** (1) Sample's `water` key (`lib.rs::sample_cell`) and the context card's river terminus (`context_pick_bridge.rs::river_pick`) read `CivData::water_bodies`, which a sculpt does not refresh; both now read `WorldGen::drawn_water_classification()`, now cached under `drawn_water_key` (world epoch, Height/Climate versions summed over every tile, grid, sea level, wrap, forced-lake epoch) and refreshed free by `build_color_texture`. `biome` in Sample still reads the civ copy (not in scope). (2) `apply_force_lake` wrote only the civ copy, so a forced lake was never drawn. The mask now lives on `WorldGen::forced_lakes` and `drawn_water_bodies` applies it (`render::apply_forced_lakes`, the reference's `forceLake` post-pass), so the map texture, `river_water_mask`, the sculpt preview, the PNG export, the export snapshot, the LOD tiles (`SnapshotInputs::forced_lakes` -> `TileFields::with_forced_lakes`, fingerprinted), the lake labels and the river cut all see it; `forced_lakes_epoch` is in `lod_cache_key` and `river_network_key`. The smooth shore (`render::shore_field_forced`/`shore_depth_forced`, `FORCED_SHORE_DEPTH`) runs a forced shore halfway between its cells and the land and leaves natural shores untouched. `recompute_civilisation` re-applies the mask (`compute_civilisation`'s new `forced_lakes` argument), so it is no longer lost there. Saved as the new MAY raster `rasters/forced_lakes.u8` (`cartalith-io::RASTER_SLOTS`, `SAVEFILE_COMPAT.md` §8.1), absent when nothing is forced; archives from before carry the forcing only in `water_bodies.u8`, where it cannot be told from a stale cell, so they are not migrated. The shell's handler (`world_workspace.gd::_on_force_lake`) now repaints the map and invalidates the tiles. Worlds with no forced lake are byte-identical (`None` is a no-op everywhere; no golden moved). (3) All five generated label classes were anchored at the cell index while `_draw_labels` draws a point; `cartalith_civ::labels::label_candidates` now adds `CELL_CENTRE` (0.5) once. Saved labels are hand-placed, already in the point frame, so none are migrated; the flat-layout importer (`legacy_import.rs`) now adds the half cell the reference draws (`_civLabelBox`). A project saved from a flat import before today keeps those labels half a cell off, indistinguishable from hand-placed ones (`SAVEFILE_COMPAT.md` §11.1). Not fixed, found: a hand-placed label's hit box and handles (`label_bridge::shell_label_box`, `labels::label_box_at`) still add the reference's half cell, so they sit half a cell down-right of the drawn label. Probe `godot-project/_mapdata_probe.gd` (windowed, seed 483920, 512x384): **26/26** on the fix (DLL rebuilt after the last `.rs` edit); the same probe against HEAD built in a worktree fails 10 (L2 labels 20.99 px off at z16, L3, S2 Sample "land" on a drawn lake, F4/F6/F9/F11/F12/F15/F17: the forced lake not drawn at fit or z16, not labelled, not drawn after a reopen). Rust: 9 new tests plus two updated label literals (the latter fail on HEAD code: (10.0, 20.0) vs (10.5, 20.5)); 11 mutants in a scratch worktree, 11 killed. `cargo test --workspace --no-fail-fast`: before 4336 passed / 0 failed / 52 ignored (186 lines); after 4431 / 0 / 54 (187 lines; the rest of the growth is a concurrent writer's uncommitted `cartalith-erosion` and `gf0_geology_harness.rs` work in the same tree).

**"Map-data residuals" (the row `fd54736` left) -- items 1-3 built 2026-09-28; verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the lane's `_mapdata` 40/40).** (1) `label_bridge.rs::shell_label_box`'s `px`/`py` no longer add `lb.x + 0.5`/`lb.y + 0.5`; they are `lb.x`/`lb.y` exactly, matching `map_overlay.gd::_draw_labels`'s own `_point_to_screen(Vector2(lb["x"], lb["y"]), rect)`, which has no offset of its own (`fd54736` already put `CELL_CENTRE` into a generated/legacy label's `x`/`y` once, at creation, and a hand-placed one is created from the click's own continuous position — neither needed a second half cell downstream). The hit box, and every handle `handle_circles` derives from it, now sit exactly on the drawn glyph. `cartography_workspace.gd::_label_side_from_handles` carried the same `+0.5` independently (compensating for the old box) and is fixed too, reading the box centre from `label_get()`'s own `x`/`y` instead of the OTHER drag math's `_label_drag_cx`/`_cy` (which keeps its own, unrelated `+0.5` -- `label_resize_size`/`label_rotate_deg`/`label_arc_value` add it internally, ported from the reference, independent of `shell_label_box`). (2) `lib.rs::sample_cell`'s `biome` key now reads the same live `drawn_water_classification` `water` reads (ocean/lake override `classify_biome`, matching `build_biome_raster`'s own precedence: `cartalith_civ::lib.rs`'s `build_biome_raster_water_overrides_climate`), falling back to the civ-layer `s.biome` only when no live classification is available (new helper `sample_biome_word`, mirroring `sample_water_word`'s own reason to be a free function). (3) `infra_tools_bridge.rs::RouteInputs::build` takes a new `forced_lakes: Option<&[u8]>` parameter (every one of its four callers now passes `self.forced_lakes.as_deref()`: `way_commit`, `route_commit`, `jp_reroute` in `lib.rs`, and `story_bridge.rs::resnap_carried_journeys`'s `ctx_for`), applying `crate::apply_forced_lakes` to its freshly-built water classification the same way `WorldGen::drawn_water_bodies` does -- before this, a Land-mode way or route, and the `Mixed`-mode biome raster routing reads, saw a forced lake as ordinary dry land, since `RouteInputs::build`'s own `build_water_bodies` call never knew about the forcing. `apply_forced_lakes` is a no-op with no mask, so a world with no forced lake computes exactly what it did before -- no golden moved. **Evidence:** three new Rust unit tests (`label_bridge::tests::shell_label_box_centre_has_no_added_half_cell`; `lib.rs::forced_lake_tests::the_sample_biome_word_lets_live_water_override_climate`; `infra_tools_bridge::tests::route_inputs_build_applies_a_forced_lake_mask`, which builds a real fixture `WorldState` via `cartalith_engine::generate_terrain` and confirms a forced cell routes as `2` while `None` reproduces the unforced build exactly). `_mapdata_probe.gd` gained legs H (item 1: a click just inside the drawn box's own top-left corner hits the label at z16 -- H5/H6 -- and the reconstructed pre-fix box, centred the old extra half cell down-right, is shown analytically missing that same point) and R (item 3: a Land-mode road stamped straight across a forced lake's centre detours around it -- R7, `closest approach to spot=10.0 cells` -- screenshot `R_routing_detour_wide.png` shows the road bowing around the lake instead of crossing it). `_mapdata` **40/40** (was 26 before this batch; H and R run before `_leg_forced()`'s own `load_save()`, which drops `labels`/`infra`/`civ` without regenerating them). Screenshots opened and inspected: `H_label_hitbox_z16.png` (the glyph "...ell..." of "Whitfell Cairn" centred in the crop, confirming the hit-box anchor and the drawn glyph are the same point) and `R_routing_detour_wide.png` (the committed road's thin line visibly bows around the forced lake's north edge rather than crossing it). `cargo test --workspace --no-fail-fast`, run in a separate worktree at this batch's own base commit (`692fc8d`) for "before": **4431 passed / 0 failed / 54 ignored (187 lines)**; "after", in this working tree: **4434 passed / 0 failed / 54 ignored (187 lines)** -- exactly the three new tests, nothing else moved. **Anomaly, not this batch's:** at the time this line was written, three files this batch never touched (`export_raster.rs`, `lod_worker.rs`, `render.rs`) were mid-edit by an unidentified third writer in the same working tree and left `cartalith-godot` failing to build (`E0061`/`E0432`/`E0593`/`E0599`); this batch's own files (`lib.rs`, `label_bridge.rs`, `infra_tools_bridge.rs`, `story_bridge.rs`, `cartography_workspace.gd`, `_mapdata_probe.gd`) build and pass on their own, verified before that third edit appeared -- the main loop should check who owns those three files before landing anything else here.
**"Map-data residuals" item 4, `sdf_biomes` ignores forced lakes -- built 2026-09-28, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the B_sdfbiome screenshots opened).** Confirmed real first: the grid's `sdf_biomes` leg (`RenderCtx::with_map_scale` and `GridPrecompute::build`) rebuilt `build_water_bodies` without Ruling BO's mask, so a forced lake was classified as the land biome under it and got no band. Now one helper, `render.rs::grid_biome_boundary_dist`, applies `apply_forced_lakes` before `build_biome_raster`; `RenderCtx::with_map_scale_forced(km, mask)` and `GridPrecompute::build_forced(..., mask)` take it, and the old names delegate with `None`. Callers passing the mask: `lib.rs::build_color_texture` and the sculpt preview (`forced_lake_mask()`), `export_raster.rs::export_render_with`, `export_session.rs::ExportSnapshot::build` and `lod_worker.rs::LodSnapshot::build` (`SnapshotInputs::forced_lakes`, already in the fingerprint). Deliberately untouched: the LOD tile's per-tile raster (`render_biome_tile_rgba`), which follows the reference in treating only below-sea cells as water and so bands no lake, natural or forced; the band on screen is the base raster's (fit zoom). **Evidence:** `render.rs::shore_tests::the_biome_band_without_a_forced_lake_is_unchanged` (bowl with a natural lake: `with_map_scale`, `with_map_scale_forced(None)`, a wrong-length mask and both `GridPrecompute` entry points bit-identical to the pre-change pipeline written out from the `cartalith_civ` calls) and `a_forced_lake_gets_a_biome_band_at_its_edge` (dry plateau, 5x5 forced block: unforced has no boundary anywhere; forced has distance 0 at x 9/10/14/15, 2 at the centre, 3 at x 6 on row 12, and the cached path matches). Mutants in a scratch copy: mask dropped in the helper, in `build_forced`, in `with_map_scale_forced` -- 3/3 killed; `with_map_scale` given a wrong-length mask survives, equivalent by `apply_forced_lakes`' length check. `cargo test -p cartalith-godot --no-fail-fast`: before 1455 passed / 0 failed / 34 ignored (35 lines, HEAD `render.rs`), after 1511 / 0 / 34 (35 lines; the render tests are compiled into 28 binaries, 2 each). `_mapdata_probe.gd` leg B (inside F, fit zoom, base raster): mean colour move of the land ring round the forced lake with `sdf_biomes` off vs on, 0.00 before the press and 18.35 after (B2, margin `WATER_MOVES`); `_mapdata` **43/43** against a scratch build; against the mutant with the mask dropped, 0.00 -> 0.00 and B2 FAILS (42/43). Screenshots opened: `B_sdfbiome_forced_off_fit.png` / `B_sdfbiome_forced_on_fit.png` -- with the band on, a speckled ecotone halo rings the forced lake that the off capture lacks.

**Note, 2026-09-28 (documentation only, no status changed by this note): the owner filed a further river request — rivers painted into the map itself as water, not a line layer, "no longer the current depressions" — scoped as `RIVERS_IN_MAP_SCOPE.md` and marked FIRST NEXT in `OUTSTANDING_WORK.md` §2.5. RV-1 through RV-5 below are unaffected by this note; their own verified/built markers stand as recorded. The new scope's RIM-1 builds on RV-2's centreline geometry and RV-4's `shore_field`, and RIM-7 folds RV-5's `paint_vector_rivers` into a single shared colour path instead of a second pass.**

**Owner report 2026-09-27, "check the rivers and lakes on different zoom levels", plus the owner's follow-up about pixelated, partial river lines — investigated, not built; a design is with the coordinator for a ruling.** `godot-project/_riverzoom_probe.gd` (windowed; seeds 483920, 24601 and 71077345 at 1024x656; zoom x1 to x240) measured:
- 11–18% of traced river cells are classified as lake, and about half of those "lakes" are 1–3 cells long.
- 65–101 drawn rivers per world are cut into 3 or more pieces by the lake mask.
- The elevation carve is stepped: p90 0.5 cells and max 2.5 cells off the smoothed line, and it is dug for about 30% more runs than are drawn.
- At fit zoom, headwaters are drawn sub-pixel.
- Export still stamps raster rivers, against Ruling AZ.

**RV-2, smooth river strokes — built 2026-09-27; verified by the main loop 2026-09-27 (river_stroke 12/0, cartalith-hydrology 32/0, cartalith-godot lib 774/0, `_rivlake` and `_riverstroke` PASS; before/after sheets inspected at ×1/×8/×32).** RV-4, RV-5, RV-1 and RV-3 are not touched. What changed:
- **Width is per point.** `cartalith_hydrology::river_half_width_profile` applies `channel_disc`'s existing width law at every traced cell. It keeps a running maximum from the head, so the width never narrows downstream. There is no new width constant. `get_rivers` emits `widths` (full width in cells) per render point. `width_cells` stays as the River dock's mouth reading.
- **Joins.** `river_stroke::settle_join_widths` caps a tributary at its trunk's width where it joins. A run bridged onto another run's head (a continuation) raises that run instead. `river_stroke::draw_ranks` draws every tributary before the run it joins; this is emitted as `draw_rank`. A first build sorted by `own_order`, and the probe refuted it: an order-3 run can end on an order-1 run.
- **Cuts.** The per-point `lake_mask` and its cutting loop in `map_overlay.gd::_draw_rivers` are removed. `river_stroke::stroke_pieces` cuts a stroke only where the *traced* run crosses drawn water, once per crossing, on the shoreline. The cut points are exact, found by bisection. The result is emitted as `pieces`. Ocean (class 1) cuts as well as lakes. This is a deliberate departure from the reference's lakes-only `splitRiverPolylines` rule (§7p): the old stroke was drawn across carved sea inlets.
- **Coast ends.** `river_stroke::coast_end` carries a mouth on dry land to the shore. For the sea this is the field's sea-level crossing; for a lake it is the cell edge.
- **Drawing** (superseded 2026-09-27; see "Rivers through the style" below). `WorldGen.river_strokes_mesh` built every stroke in one native call, as a tapered triangle strip. `_draw_rivers` handed it to `RenderingServer.canvas_item_add_triangle_array`. Strokes are never narrower than 1 px (`MIN_STROKE_PX`). The stroke is solid to its edge, with a 1 px fringe outside. A first build centred the fringe on the edge, and the ×1 PNGs showed dark trunks washed pale.

`_riverzoom_probe.gd` was extended so one probe reads both data shapes. Its new options are `--targets`, `--stats-only` and `--zooms`; it now has a no-pixels positive control and stroke statistics. It was run on HEAD, built from a `git archive` copy in scratch, and on this build. Same 3 seeds, before → after:
- **Rivers in 3+ pieces:** 45/71/65 → 11/18/16. The probe's original lake-mask count gave 65/101/77.
- **Breaks with water past them:** 47 of 214, 101/365 and 85/348 → 89/92, 110/111 and 111/114.
- **Width narrowing steps downstream:** 0 in both. Before, every run had one width. After, 27/40/29 runs taper; the rest sit on `channel_disc`'s 0.5-cell half-width floor at this map scale.
- **Confluences:** gap 0.0 cells in both. Tributary wider than trunk at the join: 1/0/3 → 0/0/0. Trunk drawn over the tributary's end: 14/313, 20/362 and 14/252 → 300/303, 354/360 and 251/253. Continuations that narrow: 9/4/1 → 0/0/0.
- **Free ends beside water that stop short:** 229/258/234 → 77/76/110. The remaining gap has a median of 0.23–0.36 cells; these are mostly sea crossings inside the land cell, as the per-cell metric counts them.
- **×1 diff-mask components:** 664/571/546/804/501/552 → 292/239/220/336/248/221 for the six targets.
- **Median frame at the opening view with rivers on:** 56/67/42 ms → 23/28/19 ms. The rivers-off baseline is 16.7 ms. This is one run each, and the figures are vsync-quantised.

`_rivlake_probe.gd` and `_riverstroke_probe.gd` were ported off `lake_mask`/`width_cells`, and both pass. `_lodsweep_probe` was run with rivers hidden: all 18 seam medians are ≤ 1.40, and there was 1 pop in 2736 frames (483920/512 pan). The pop cannot come from this change, because rivers are hidden in that probe.

PNGs, before/after, looked at by the lane:
- At ×1, headwaters are continuous 1 px lines instead of dotted ones.
- At ×8 and ×32, a river meeting a bead of one-cell lakes stops at the first lake and resumes after the last. Before, it broke into stubs between them.
- At ×240, strokes are smooth.

What remains is not RV-2's to fix:
- The beads of square lakes themselves (RV-4, RV-1/3).
- A LOD-tile hairline visible in the rivers-off frame.

Rust: 13 new tests. 16 mutants (3 in `river_half_width_profile`, 13 in `river_stroke.rs`), all killed after two tests were added for the first-round survivors. `cargo test --workspace --no-fail-fast`: **3971 passed, 0 failed, 42 ignored**. No JS golden moved.

**River mouths reach the water, and water is drawn above rivers — built 2026-09-27, verified by the main loop 2026-09-27 (lake and ocean mouth comparisons opened; the staged tree built and tested alone in a worktree)** (owner: *"When rivers end into the ocean or lake they should be drawn a bit longer to make sure they actually end in the ocean/lake. And the ocean texture should be drawn above the river graphic."*).
- **Every end that meets water is carried into it** (`river_stroke::extend_shore_ends`, `shore_reach`, `ShoreReach::extra`). That covers mouths, lake inlets, lake outlets inside a run, and, new, a run whose head is the dry cell below a lake (`river_draws`, `coast_end` on the reversed run). The margin is not a number. The three drawing paths put the shoreline at three sub-cell places: the base view draws whole cells, a tile's sea is the bilinear field below sea level, and a tile's lake uses its own band rule. All three draw water wherever all four surrounding cell centres are water. The end is marched along its own last step until a cap of half-extent (half-width + fringe) lies wholly in that region. The cap is taken in the raster's own cells, so each raster carries the end as far as its own width needs (`for_each_span`). The march stops at a lake's far shore. One labelled bound applies: incidence up to tan φ = 2. Past that, or in a lake too small, the end takes the widest fit found.
- **Water above rivers.** Tiles already coloured water pixels as water. Their alpha-coverage pass now also skips water, so the overshoot cannot seam the water mid-morph (`render_biome_tile_rgba_rivers`). The base view draws its stroke into its own child canvas item with `shell/river_under_water.gdshader`, which discards on `WorldGen::river_water_mask` sampled nearest. The shader reads the mask through `map_overlay.gd::_map_texture_rect`, Godot's own truncated TextureRect fit. The overlay's `_displayed_rect()` is up to a pixel off the drawn map; that gap is measured and applies to every overlay, and it is not fixed here.
- **Measured, 1024×656, before → after:**
  - Free ends short of adjacent water, 3 seeds (`_riverzoom_probe`): 76 / 80 / 100 → 0 / 1 / 0. The one left (seed 24601, (374.5, 407.5), water only diagonal, 0.71 cells) has no established cause.
  - Rivers in 3+ pieces stay 2 / 0 / 0, and ocean on path stays 0.
  - `_rivstyle_probe` section M (screen pixels), Default and Ink:
    - M1, river pixels on open water at fit: HEAD 5 / 1 → 0 / 0.
    - M2, stroke on the last land pixel before the water at fit: HEAD 74/81 and 73/81 → 62/62 on both.
    - M3, river pixels on open water at z16, 8 mouths: 0 → 0. This leg already held on HEAD, because tiles skipped water.
  - The whole `_rivstyle` probe has 0 failures. `_rivlake` passes.
- **Not measured on screen: reach at z16.** Three attempts were each refuted by their own trace: shallow sea is coloured like beach, the tile's lake band is not the cell grid, and a `toon_outline` rebuild moved open-water pixels.
- **Looked at:** z16 crops show the HEAD sliver of land gone at a sea mouth and a lake inlet. The end is cut by the tile's ragged shoreline, notched at one mouth. At fit, the mouth's edge is the coast's own pixel stair, about 1.4 px, because the map draws its water as nearest-filtered cells.
- **Rust:** 8 new tests, plus shore-flag assertions in an existing one. Mutation testing ran in a scratch copy: 24 mutants, all killed. 6 of them survived the first round and were killed after tests were added. `cargo test --workspace --no-fail-fast`: 4109 passed / 0 failed / 49 ignored.

**RV-4: smooth sub-cell shorelines, sea and lakes, below and above the deep-zoom switch — built 2026-09-28, verified by the main loop 2026-09-28 (shore_tests 8/0, tile-biome 20/0 re-run; lake z16 and sea fit comparisons opened; the staged tree built and tested alone in a worktree)** (the "Coastlines and lake shores are pixel-stepped" row and RV-4 of "RV-4, RV-5"; RV-5, export, not started).
- **The field.** `render::shore_field` (per cell, half-float `RH` texture `WorldGen::shore_field_texture`, built in `build_color_texture` from the same `drawn_water_bodies()` call its colours use): the water surface minus the ground, the smaller of two margins (`shore_margins`) — depth below `sea_level` or below a lake's `fill − LAKE_DEPTH`, and the rainfall past `LAKE_RAIN` for an above-sea lake — clamped so every water cell is `≥ SHORE_EPS` and every land cell `≤ −SHORE_EPS`. So the sea's zero line is the height field's own coastline, a lake shore is where the ground rises through its surface or the country turns too dry for it, and classification cannot change (every cell centre keeps its class; four water centres are always water, the `river_stroke` mouth region). `cartalith_civ::LAKE_DEPTH`/`LAKE_RAIN` are the classifier's existing literals, named; no golden moved.
- **Base map (fit, ×1.4).** `shell/map_shore.gdshader` on `viewport_host.gd::map_view`: pixels between a water and a land cell centre are split along the field's bilinear zero line and antialiased, each side taking its nearest texel; every other pixel is the nearest texel as before, so styles/paper/grade are untouched (probe leg I: 0 off-band pixels move, all 6 runs). `river_under_water.gdshader` reads the same field through `shore_field.gdshaderinc` and fades by the water's coverage; the nearest mask is its fallback.
- **Tiles (z16).** `render::is_lake_pixel`'s band, when `TerrainAppearance::smooth_shores` (true in `default()`, false in `js_reference()`), is water where the same field is positive — one outline from fit to the deepest tile. The tile's own `ht` detail was tried in the rule first and dropped: it made lake shores ragged pixel by pixel and let two lake mouths show river through (M3 4–7 px). The tile sea is unchanged.
- **Measured, 1024×656, 3 seeds, Default and Ink, `_shorestep_probe` (windowed; HEAD in a worktree):** fit/×1.4 shoreline crossings inside a texel (a cell-edge coast is 0 by construction) HEAD 0.000 everywhere → 0.21–0.27 at fit, 0.41–0.52 at ×1.4; antialiased edge pixels HEAD ≤ 0.008 → 0.46–0.56; z16 lake stair fraction (chance 0.22) HEAD 0.325 / 0.246 / 0.313 → 0.171 / 0.108 / 0.170 (Default), sea control unchanged 0.123 / 0.096 / 0.104. HEAD 18 / 16 / 18 failures → 0 / 0 / 0.
- **River mouths:** `_rivstyle` M: M1 0/0, M2 36/36 (fewer judged ends than HEAD's 62: the smooth coast leaves fewer exact-land pixels on the walk), M3 0 except Ink ocean mouth 7 at 6 px, which HEAD shows identically (pre-existing). `_rivlake` PASS. `_riverzoom` stats: 3+ pieces 2/0/0, ocean on path 0, short ends 0/1/0 (unchanged).
- **Known, not handled:** a one-cell lake draws as a rounded blob inside its cell, so a stroke end falling back to `REGION_DRAWN` can stop a fraction of a cell short of it (`river_stroke.rs` "River mouths" note); colours baked per cell next to the coast (Ink's dark shore texels) still trace the old cell line inside the land; land itself stays nearest-filtered at fit.
- **Rust:** 8 new tests (`render::shore_tests`); 19 mutants in a scratch copy, 17 killed, 2 survivors equivalent and documented at their symbols (`SHORE_FAR_LAND`'s sign, the water half of the clamp). `tests/golden_parity_tile_biome.rs::the_lake_branch_draws_a_lake_and_follows_the_terrain` now pins `smooth_shores: false`: its leg (3) asserts the v1.05 band's use of the tile's own height, which the smooth band deliberately drops; no golden value changed. `cargo test --workspace --no-fail-fast`: before 4109 passed / 0 failed / 49 ignored (186 result lines); after 4336 / 0 / 52 (186 lines; the rest of the growth is the concurrent GF-10 lane's tests).

**RV-5: exports draw the screen's vector rivers and smooth water — built 2026-09-28, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the 16K mouth and lake crops opened)** (Ruling AZ, *"the vector line, not the baked ink"*; the RV-5 half of "RV-4, RV-5").
- **Rivers.** Every export (`export_image` banded, `export_raster_png`, `export_snapshot_png`, `export_layer_previews`, the overlay session's `ExportSnapshot`) gets `render::ExportRivers` from one assembly (`export_raster.rs::export_render_with` / `with_export_rivers`): a generated world's network (`WorldGen::export_river_geometry`, the cached `river_geometry_any`) rasterized per export rectangle by `river_stroke::rasterize` through `RasterMap::tile` — the tiles' rasterizer, the `river_px_width` seam at the export's own pixels per cell, `river_style_color` — and composited inside `land_color` by `render::paint_vector_rivers`, after local contrast is measured on the river-free terrain (`render::bake_and_finish`; banded: per band, band rows only).
- **Water.** `BakeFields::with_shore_field` attaches the screen's `shore_field_forced` (forced lakes included); `BakeFields::pixel_at` decides water by `map_shore.gdshader`'s rule (bilinear field, coverage `0.5 + s/fwidth` at the export's pixel step) and antialiases the shore. Sea vs lake: the pixel's own bilinear height below sea level is sea, else lake if any wet corner is above sea, else sea at the nearest wet corner. The shader's nearest-texel kind was built first and drew cell stair steps at the lake/sea boundary in the 16K crop; replaced. `smooth_shores` false (`js_reference()`) and every golden keep the old cell rule by control flow.
- **Retired (§7p):** `render::RiverInk` (its `Stamped` arm and the enum), `WorldGen::river_ink()` and `screen_river_ink()`, `lod_worker::OwnedInk::Stamped`, and `tests/bake_raster.rs::a_partial_stamp_tints_less_than_a_full_flag`. **Kept:** a loaded save's one-cell `strahler_order` flag (`render::save_flag_at`, `WorldGen::save_river_flag`) on screen, tiles and export — such a save has no traced network, so there is no vector to draw. The engine still computes and saves `ChannelResult::intensity` (`cartalith-engine`, `substrate.rs`); nothing in the port draws it now. No JS golden moved.
- **Parity (matched scale, 2 px/cell, `export_raster.rs::rv5_parity_tests`, screen transcribed from the two shaders), default / antique+ink:** centreline mean |Δ| 0.18 / 0.65 levels (no-river export 55.1 / 23.9); shoreline water coverage error 0.000 / 0.000 (cell rule 0.328 / 0.331); lake-shore colour 3.68 / 4.19 (cell rule 14.7 / 13.4); open water 3.85 / 3.88. Sea-shore colour 18.1 / 19.3 is printed, not bounded (pre-RV-5 19.7 / 20.8): the screen shows nearest-cell texels where the export evaluates the material per pixel.
- **Memory and time, banded `export_image`, seed 483920 at 2048×1311, `_rv5export_probe.gd` + host poll of the Godot child's PeakWorkingSet64, one run each unless stated:** 16K 1 651 MB / 52.1 and 52.9 s → 1 680 and 1 679 MB / 54.0 and 54.3 s (+~29 MB, +~2 s; the per-band river layer and the repaint of river pixels). 32K 1 653 MB / 410.7 s → 1 682 MB / 340.4 s (time is noisy on this shared machine — the other lane was building; no slowdown is established). 16K with `rivers: false`: 1 671 MB / 52.4 s.
- **Looked at (16K crops, before → after):** river mouth: stepped raster ink and a blue stamped line inside the channel → smooth channel, water drawn over the river, no stair steps on the banks; lake: stepped cell shore and stamped ink lines drawn across the lake → smooth antialiased shore, no river on the water. Two short, pale, straight strokes stop at the lake shore in the lake crop: they are rivers (absent from the same export with `rivers: false`, where those pixels are land), drawn in RV-2's light headwater colour, and they read straight because their traced geometry is straight there. The coast beside the mouth keeps its pale shallow-sea band, as before.
- **Rust:** new tests `export_raster::rv5_parity_tests` (1), `export_session::…a_snapshot_draws_the_vector_rivers_and_smooth_shores…`, `tests/export_bands.rs::vector_rivers_band_identically`; 12 mutants in a scratch copy, 12 killed (one survivor, "all-water pixel drawn as land", killed after the open-water leg was added). `cargo test --workspace --no-fail-fast`: before 4434 / 0 / 54 ignored → after 4436 / 0 / 54 (the other lane's tree ran concurrently). GDScript parse-checked: `data_manager_window.gd`, `_exportraster_probe.gd`, `_riverstroke_probe.gd`, `_rv5export_probe.gd`, `app.gd`.

**RV-3: the valley is shaded along the drawn river line, not the stepped carve (Ruling BD) — built 2026-09-28, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the 24601 x8 before/after opened).** Measured first, on the tree before this change (`_riverzoom_probe`, seeds 483920 / 24601 / 71077345, 1024×656): the problem was still there after RV-1/4/5. Runs traced (and carved) vs drawn 1637/1219, 1670/1245, 1201/908; traced cell to drawn line median 0.16/0.14/0.15 cells, p90 1.19/0.57/0.58; screenshots at ×2/×8/×32 show dark stair-stepped grooves beside drawn rivers and parallel grooves with no river.
- **What changed.** `valley_shade::valley_shade_field` (new `crates/cartalith-godot/src/valley_shade.rs`) builds a *shading-only* height: the carve (`WorldState::river_mask`) filled back in by harmonic relaxation, then a valley cut along each drawn line (`river_geometry_any`, RV-2's widths), as deep as the carve was there (deepest carve within half-width + 1.5 cells, smoothed ±1.5 cells along the line), with a smooth `(1−q²)²` shoulder. Water cells and every land cell 8-adjacent to water keep the true height, and no land is cut below `sea + CARVE_LAND_MARGIN`, so every per-cell and bilinear sea test is unchanged. `WorldGen::valley_shade_field` (cached under `river_network_key_str`) hands it to the three render paths only: `build_color_texture`, `lod_snapshot_inputs` (new `SnapshotInputs::shade`, in the stored-pyramid fingerprint) and `export_render_with` / `export_snapshot`. Water is still decided from the world's field: new `RenderCtx::water_height` / `with_water_height` / `with_precomputed_water_height` and `GridPrecompute::build_split` keep water bodies (tiles), `smooth_sea_h`/sea shade, coast distances and `sdf_biomes` on the true height. New `TerrainAppearance::smooth_valleys` (true in `default()`, false in `js_reference()`). Hydrology, the carve, saves and Sample are untouched.
- **No engine, hydrology, render or tile golden moved.** No golden fixture goes through `WorldGen`, and `js_reference()` turns the valley off. `_riverzoom_probe` grid statistics (carve drift, channel lakes, map-wide lakes, stroke pieces/joins/ends) are byte-identical before and after on all 3 seeds.
- **Measured** (`_rv3shade_probe.gd`, new, windowed; rivers off, labels cleared; zooms 2–32, `trunk` and `undrawn` targets reused via `--targets`; groove = black top-hat > 8 levels at 3 px/cell). Pooled dark fraction, before → after: *off the drawn line on the carve* 0.146 → 0.092, 0.141 → 0.119, 0.269 → 0.161; control far from any river 0.092, 0.076, 0.085 (unchanged); so the excess over control +0.054 → −0.000, +0.064 → +0.042, +0.184 → +0.076. Positive control, *on the drawn line*: 0.251 → 0.307, 0.186 → 0.204, 0.334 → 0.406 (the valley is kept and now sits under the stroke).
- **Looked at** (scratch `rv3/sh_{before,after}/`, `rv3/rz_{before,after}/`): 24601 ×8 and ×32, 71077345 ×2 (base map) and ×4: the stepped grooves become smooth valleys following each curve; doubled parallel grooves beside a trunk are gone; coasts, lakes and sea colours look unchanged.
- **Known, not handled:** the last carved cell beside water keeps its carve (the frozen ring); drawn rivers whose carve was shallow or kept-lake get no valley (as before); a sculpted channel that no drawn river follows is filled in by the shading (its height is unchanged). The shade field's build cost is not measured; it is cached per network key.
- **Rust:** 12 new tests (`valley_shade::tests` 9, `render::valley_split_tests` 3). Mutation in a scratch copy: 22 mutants, 20 killed. 4 survived round one because the water fixture's trench was below sea level; the fixture was rebuilt and all 4 are killed. 2 are equivalent and documented at their symbols (the fill's bank start, and its never-below-carve guard). `cargo test --workspace --no-fail-fast`: before 4507 / 0 / 54 ignored (187 result lines), after 4600 / 0 / 54 (187 lines; most of the growth is concurrent lanes' tests). GDScript parse-checked: `_rv3shade_probe.gd`, `shell/app.gd`. The pre-existing `Gd<T>::bind() failed, already bound` panic spam from `WorldGen::lod_worker_stats` appears in the probe logs on both DLLs; it is not this change's.

**RV-3 follow-ups: sculpted channels keep their groove, the valley-shade cost measured, the step beside water documented — built 2026-09-28, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the x8 sculpted-channel before/after opened)** (`OUTSTANDING_WORK.md` row "RV-3 follow-ups: a sculpted channel with no drawn river is filled in the shading; ...").
- **(1) Sculpted channels.** The lock mask now records who dug a cell: `WorldGen::sculpt_commit` stores cells a Sculpt commit locked (or re-locked with a new floor) as `valley_shade::LOCK_SCULPT` (2) via `mark_sculpt_locks`; the generation carve's cells stay `LOCK_CARVE` (1). `valley_shade_field` fills only `LOCK_CARVE` cells, and fills them from `max(field, river_floor)`, so anything that lowered a carved cell below its floor since (a sculpt, an erosion pass) stays in the shading. Every lock reader tests `!= 0`; the value is saved with the mask, and `SAVEFILE_COMPAT.md`'s `river_mask.u8` row now says so (old saves hold only 1s, which is what they were). On a world nothing has edited, the result is unchanged (test `a_floor_equal_to_the_field_changes_nothing`, bit-equal).
- **Proved on screen** (`_rv3cost_probe.gd --mode sculpt`, new, windowed; seed 24601, 1024×656; a 40-cell River stroke at (208, 328), default brush, 4570 cells locked, no drawn river within 3 cells, height at its middle 0.809 → 0.685). Along-averaged luma cross-section, offsets −12..+12 cells: after, the north wall drops from 175.5 (−6) to 106.4 (−3) at ×4 and to 99.9 at ×8; before, the same rows read 176.8 → 162.3, a gentle ramp over a filled mound. Screenshots opened: scratch `rv3b/out_before/s24601_channel_z{04.0,08.0}.png` show the locked half of the channel filled into a grey plateau; `rv3b/out_after/` show the whole trough with its walls. (The probe's range statistic does not separate the two, 67.8/68.9 before vs 72.4/78.7 after, because the stamp's lowered banks outside the lock have walls in both; the cross-section rows are the reading.)
- **(2) Cost** (`_rv3cost_probe.gd --mode cost`, new; the shell's own `bridge.generate` with its defaults, seed 24601, square grids; `WorldGen::valley_shade_stats`, a new diagnostic `#[func]`; this machine, one Godot and no build running while the reps were timed; median (min..max) over reps of a freehand sculpt commit followed by the shell's repaint). `core` is `valley_shade::valley_shade_field` alone; `total` adds building the drawn network and water classification on cold caches, which the repaint needs anyway. **It always runs on the main thread, inside `build_color_texture`.**

  | Grid | core before | core after | total after | whole repaint (context) |
  |---|---|---|---|---|
  | 2048² (n 5) | 137.0 ms (135.9..139.4) | 106.7 ms (106.4..106.9) | 696.8 ms | 2645 ms |
  | 8192² (n 3) | 1261.7 ms (1229.1..1302.4) | 711.3 ms (706.3..731.6) | 10867 ms | 48590 ms |

  First build after generation: 140 → 107 ms (2048²), 3397 → 713 ms (8192²; the before reading may have overlapped the start of a compile in this lane, so it is not a clean figure). **It still breaks PERFORMANCE_BENCHMARKS.md's bar** (main-thread work inside a 60 Hz frame, 16.7 ms), as does the repaint it sits in, by far more: the valley core is 4 % of the 2048² repaint and 1.5 % of the 8192² one. Done here: the fill relaxes a compact array of the carved cells only (neighbours pre-resolved), the water ring is tested per visited cell instead of built as a grid-sized mask, and the re-cut gathers its cut per touched cell instead of cloning the field. Moving the valley alone off the main thread would not remove the stall, because the synchronous repaint needs its result; that is the repaint's problem, and this lane has not filed it.
- **(3) The step beside water: documented, not fixed**, at `valley_shade::is_frozen`. Filling that cell would raise a land corner of a bilinear cell with a water corner, and `render::land_color`'s per-pixel sea test still reads the shaded height, so the drawn shoreline would move at every mouth and inlet. The clean fix routes every per-pixel sea test to `RenderCtx::water_height` in all three render paths.
- **Rust:** 8 new tests in `valley_shade::tests`, each with `// Protects:`. Mutation in a scratch copy: 21 mutants, 17 killed in round one. Two new tests (`the_recut_under_a_sculpt_is_as_deep_as_the_carve_not_the_sculpt`, `a_cell_below_sea_level_freezes_its_ring_even_when_not_classified_water`, fixture rebuilt once) killed 2 more. 3 are equivalent and documented at their symbols: the fill's bank start and its never-below-surface guard (as RV-3 found), and the alternating sweep order. `cargo test -p cartalith-godot` (scratch target): before 1616 passed / 0 failed / 34 ignored (35 result lines), after 1624 / 0 / 34. No golden moved: no golden fixture reaches `valley_shade` or the relabelling in `lib.rs`.
- **Not covered:** a project saved before this change reads every lock as the carve's when reopened, so a channel sculpted before 2026-09-28 is still filled until it is sculpted again.

**The repaint row: zoom/sharpen measured, repaint sped up (byte-identical), the carved cell beside water filled — built 2026-09-28, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; mouth before/after diffed: change confined to the mouths)** (`OUTSTANDING_WORK.md` row "The colour-texture repaint blocks the main thread ...; and the carved cell beside water keeps its step"; Ruling BP changed Part A mid-lane: a freeze on generation/simulation is acceptable, zoom and sharpen must be fast, cheap output-identical speed-ups welcome). All timings: this machine (16 threads), windowed Godot 4.7.1, DLL built into the lane's scratch target, one Godot probe at a time (the owner's two idle Godot processes open), no build running.
- **An async (off-main-thread) repaint was built, then reverted per Ruling BP** (no trace left in the tree). Its measurements, for the record: at 2048² main-thread share 14.5 ms (14.3..14.9, n 3), landing frame 90 ms; at 8192² 272 ms (269..278, n 3), landing frame 893 ms — the texture upload itself cannot fit 16.7 ms at 8192².
- **Zoom / sharpen / pan (the paths that must be fast)** — `_zoomcost_probe.gd` (new), seed 24601, 2048×1311, vsync off, 24 wheel notches (×1.15) from fit, HEAD DLL. **No `build_color_texture` or other repaint runs on these paths**: the `zoom_step` call itself is 0.2–0.9 ms; tiles arrive from the worker (LOD-D6). With the app's default overlays on, every notch's first frame is 41–84 ms and pan at ×16 is 36 ms median (32..51, n 120), 144 frames over 16.7 ms; with every overlay hidden, 5.5–40 ms and pan 2.1 ms. The cost is `map_overlay.gd::_draw()` re-running on every camera move (`_update_lod()` queues it): per-layer redraw (`--layer-costs`) at fit — rivers 38.2 ms (`river_view_mesh`, 3.6 M indices; 27 ms measured alone), roads 30.2, sea_routes 9.5, settlements 3.7, none 1.2; at ×16 — roads 25.1, settlements 23.1, sea_routes 9.5, rivers 2.1. **Not fixed here** (a GDScript overlay change, outside this lane's row): filed for the main loop to route.
- **Repaint speed-up, output byte-identical.** (1) `build_color_texture` builds the drawn water bodies first and fills `drawn_water_cache` before asking for the valley, so the valley and the river network no longer run a second identical priority flood after an edit. (2) New `RenderCtx::with_appearance_split` builds the context in one `GridPrecompute::build_split` pass instead of `with_appearance(..).with_water_height(..)`, which built the bathymetry blur from the shaded field and threw it away. `_rv3cost_probe.gd --mode cost` (now also hashes the four textures per rep): 2048² **2663 ms (2642..2674, n 5) → 2038 ms (2028..2085, n 5)** with the speed-up alone (HEAD + the two changes, built in a scratch worktree), every rep's map/river/mask/shore sha256 identical to HEAD's; 8192² 48.6 s (RV-3 follow-ups' figure; this lane's re-run 53.6 s, 48.9..55.7, n 3) → **37.6 s (37.6..37.8, n 3)** on the full tree. The rest of the 2048² repaint: pixels ~540 ms, precompute ~465, river field ~196, river pixels ~140, valley core ~107, finish ~85.
- **Part B: every per-pixel sea test reads the world's height, and the ring is unfrozen.** `render.rs`: `cell_color_river` (sea branch, waves), `toon_water_cell`/`_f`, `hillshade_raster`, `build_grade_influence_cells`, `BakeFields::pixel_at` (cell rule), `shore_cover`'s sea-or-lake kind, `water_cover`, `bake_rect`'s ink guard, `is_lake_pixel`'s smooth band and flat-lake test all read `RenderCtx::water_height`. Tiles: new `render_biome_tile_rgba_water` takes a second height tile amplified from the world's field and every tile water test reads it (`tile_water_class`); `lod_bridge::tile_heights` builds it only when `render::tile_water_differs` (a changed cell beside water in the tile's footprint), so other tiles amplify once as before. `valley_shade::is_frozen` now freezes water only. **Shoreline unmoved:** `lod_bridge::tests::a_generated_world_draws_the_same_water_with_the_valley_field` (160×112 generated world, the unfrozen valley changes land beside water) — every tile of levels 0–3 (mask and water-pixel RGBA) and the export's cell-rule cover, smooth-shore cover and full-water colours identical to the world-height render; the screen's per-cell test cannot move (no cell changes side). `_repaintasync`/`_rv3cost` hashes: the water mask and shore field textures identical to HEAD on all 5 reps; the map and river textures change (the valley). **Looked at** (`_mouthstep_probe.gd`, new; seed 24601 1024×656, three largest sea mouths, ×8 and ×32, rivers off and on): the grey carve notch at the mouth cell is gone, the coastline and water pixels match HEAD; diffs are confined to land near the mouths.
- **Rust:** new tests `render::valley_split_tests::the_split_constructor_is_the_two_step_context`, `lod_bridge::tests::{the_water_mask_is_the_same_whatever_the_shaded_field_holds, only_a_tile_whose_coast_the_valley_changed_gets_a_water_tile, a_generated_world_draws_the_same_water_with_the_valley_field}`, `valley_shade::tests::the_carved_cell_at_the_mouth_loses_its_step`; two ring tests rewritten for the new rule. Mutation (scratch copy, 17 mutants): 12 killed. Survivors: the screen's per-cell test reading the shaded height (equivalent — no cell changes side); the export's sea-or-lake kind and the flat-lake test (not reached by the fixtures); the tile river layer's land-only mask (the tests synthesize without rivers); a tile sea-threshold change applied to both arms (equivalent for a truth-vs-split comparison). `cargo test -p cartalith-godot` (scratch target): before 1624 / 0 / 34 ignored, after 1656 / 0 / 34 (the growth includes the concurrent BK/io lanes' tests). No golden moved. GDScript parse-checked: `_rv3cost_probe.gd`, `_zoomcost_probe.gd`, `_mouthstep_probe.gd`, `shell/app.gd`. `cargo test --workspace` not run by this lane.

**Overlay layer cache: zoom and pan stop re-running every overlay layer (`OUTSTANDING_WORK.md` "Zoom and pan stutter", Ruling BP) — built 2026-09-28, verified by the main loop 2026-09-28 (timings re-summarised from the lane's log; x16 before/after compared; the staged tree built and tested alone in a worktree). Pan and the notch first frame now hold 16.7 ms, and frames over 16.7 ms during a notch are about half HEAD's (timings below).**
- **Before (reproduced on the `298c0cfc` tree)**: `_zoomcost_probe --layer-costs` (seed 24601, 2048×1311, vsync off, Ryzen 7 9800X3D / RX 7800 XT, GL Compatibility, one Godot running). Redraw at fit: rivers 38.1 ms (23.6 of it `river_view_mesh`), roads 31.1 (5.4 of it per-point conversion, 12.8 per-dash `draw_line` commands), sea routes 9.4. At ×16: roads 24.9 (12.2 converting the points of runs that were then culled), settlements 22.8.
- **Change** (`map_overlay.gd` "Layer cache" block). Every layer draws into its own child canvas item, in the old draw order (rivers < catchment < sea routes < roads < mid < settlements < top). `_draw()` is now the content path: every cached layer is rebuilt in the same frame, so an edit, toggle or hover is never late. A camera move calls the new `view_changed()` (`viewport_host.gd::_update_lod`, `set_camera_zoom`, `set_lod_active`). Only catchment and mid redraw per move. The five cached layers (rivers, sea routes, roads, settlements, top) rebuild only when their key changes (zoom, size, content, the deep-zoom switch, the revealed-town set), or when the view leaves the guard rect they were culled to (the view plus 15% each side; a background rebuild starts 10% from its edge). A zoom rebuild runs in the background into a hidden second set of items and swaps in when complete; until then the old set shows, scaled by the camera — the one visible transient. Also: a whole-run cull on a cached grid-space box before any point is converted; `_stroke_points` as one `Transform2D * PackedVector2Array`; each dashed run as one `draw_multiline`; urban layouts split into `_urban_plan_update` / `_draw_urban_layouts`.
- **Second pass (same day), after the first measured MORE frames over 16.7 ms during notches than HEAD (72 → 290 desktop, 87 → 222 phone form).** Per-frame chunk traces (scratch-only instrumentation) found five causes, each fixed:
  - **Chunks too coarse and unbalanced.** One road chunk was 16 ms. Now rivers 24, roads 24 and sea 4 chunks (the ways balanced on point count, `_weighted_bounds`), settlements 16, top 12 (labels balanced on estimated glyph-raster cost, `_label_chunk_bounds`).
  - **The settlement plan (up to 6.7 ms) ran inside one chunk.** It now runs in 6 ordered steps (`_settlement_plan_begin` / `_settlement_plan_step`).
  - **The top layer was redrawn on every move** (4–27 ms of each notch's first frame at zoom ≥ 8: glyph rasterisation at each new zoom). It is now cached, and labels that cannot reach the guard are skipped.
  - **The budget was fixed and used a layer-average estimate.** It is now AIMD on the measured frame time, less the camera move's own work that frame. Each chunk is placed on its own last cost, ratio-corrected and floored (`LAYER_FRAME_TARGET_US` block).
  - **The 50% guard itself made every settled frame slower**: the renderer submits every command of a chunk whose bounds touch the screen. Idle frame at zoom 4.32: HEAD 8.4 ms, guard 0.5 → 17.6, 0.25 → 10.6, 0.15 → 9.6, 0.05 → 8.6. Now 0.15.
- **Rust:** `WorldGen::river_view_mesh_runs` / `river_run_count` (a run range of the base-view stroke, so rivers rebuild in chunks), through a pure `river_stroke::view_mesh` over `for_each_span_in`. `engine_bridge.gd` wrappers: `river_run_count` answers -1 on an older binary, and the overlay then draws the stroke whole. Tests `river_stroke::tests::{chunked_view_mesh_is_the_whole_mesh_in_order, a_chunk_draws_only_its_own_runs}` (`// Protects:`). `cargo test -p cartalith-godot` (scratch target): before 1656 / 0 / 34 ignored, after 1658 / 0 / 34 (no Rust change in the second pass). Mutation testing not run. A scratch-only print confirmed the chunked path runs on the probe world (1667 runs).
- **Pixels** (`_zoomcost_probe --shots`, new: every layer on, fit / ×4 / ×16 / ×32 / ×16 after the 120-frame pan; HEAD overlay against this one, same DLL, map rect only; re-taken after the second pass with the same results). Desktop: 187–270 px differ (≤ 0.033%), max delta 8–32, ≤ 35 px over 10; after the pan, 1973 px (0.24%), max 33, 9 over 10. Phone: below the status-bar clock (which also differs between two HEAD runs), 172–587 px differ, ≤ 52 over 10, max 51. Crops opened at the worst spots show single-pixel antialiasing specks along dashed roads, from the batched dashes and the native point transform; nothing else is visibly different. Two HEAD runs are byte-identical on desktop.
- **Probes on the final overlay** (windowed, scratch copy + scratch DLL). `_mapdata_probe` 43/43; `_rivstyle_probe` 0 failures (HEAD the same); `_tlpins_probe` 50/50 on desktop and phone; `_ctxring_probe` 132/132 on desktop. After the first pass, the first run of `_tlpins` and `_ctxring` failed once (`_tlpins` H-/G+/G- "changed=6400"); it did not recur in 3 later runs, nor on HEAD in 3, nor on the final overlay — recorded, not explained. `_ctxring_probe --vp 1080x2340` fails 37 checks on HEAD and on this change, the same set (the probe has no `--force-touch` phone form). The shutdown "RID/texture leaked" errors match HEAD on the same `--notches 2` run (35 CanvasItem / 35 material / 36 texture), so they predate this change.
- **Timings, HEAD overlay against the final one** (2026-09-28 11:23–11:33, `_zoomcost_probe`, same scratch DLL, vsync off, one Godot at a time, 24 notches and 120 pan frames per run). A process check before every run found no cargo, rustc or other Godot running; the fourth rep also checked after. The new phone-form run 1 is EXCLUDED: a `cargo` build started during it (seen at the next check; its numbers were 157 notch frames over and pan 16.5 ms median). Log: the lane's scratch `ovl2/timings2.log`. The probe counts a notch sharp only when the overlay's background rebuild has also finished (`pending_builds()`), and times the frames after a pan until it settles.

  | | runs | notch first frame, median / p95 / max | notch frames > 16.7 ms, per run | pan at ×16, median / p95 / max | pan + settle frames > 16.7 ms |
  |---|---|---|---|---|---|
  | desktop, HEAD | 4 | 55.0 / 88.3 / 125.9 ms | 24, 24, 24, 24 | 35.1 / 52.6 / 76.9 ms | 480 of 480 |
  | desktop, new | 4 | 11.8 / 17.1 / 18.1 ms | 15, 13, 11, 13 | 6.2 / 9.5 / 13.2 ms | 0 |
  | phone form, HEAD | 4 | 49.9 / 82.2 / 96.9 ms | 32, 26, 26, 27 | 39.0 / 45.5 / 58.6 ms | 480 of 480 |
  | phone form, new | 3 | 9.7 / 23.0 / 41.9 ms | 15, 10, 17 | 8.4 / 12.8 / 17.1 ms | 1 |

- **Against Ruling BP's 16.7 ms.** Pan passes. The notch first frame's median is ~12 ms against a settled idle frame of ~9 ms, so what is left of it is the map's own render; the overlay adds ~2–3 ms. Notch frames over 16.7 ms are about half HEAD's per run. **Not zero, and why:**
  - One label's glyph rasterisation at a new raster size (up to ~11 ms for a big region name) cannot be split without changing how the label draws.
  - Around zoom 3–6, frames with no overlay work at all measure 17–25 ms (the tile path, unchanged here).
  - Phone-form numbers are the desktop GPU at 1080×2340, not a handset.

**Zoom residual: label glyphs pre-warmed off the main thread, build budget held under the bar — built 2026-09-28, verified by the main loop 2026-09-28 (the worker thread was removed after a thread-safety review; timings re-read from ovl2/timings5.log; 0 px differ). Notch frames over 16.7 ms: desktop 58 → 31 over 4 runs, phone form 62 → 43, against `7063f4d7`. Pixels are byte-identical to it.**
- **Attribution** (scratch-only per-frame trace: overlay chunk costs, `viewport_host.gd::_process` time and tiles installed, the viewport's measured render time; seed 24601, 24 notches). The notch frames still over 16.7 ms on `7063f4d7` fall into three groups:
  - **Label re-raster:** a top-layer chunk holding one big region or continent name at a new raster size, 1–5.7 ms for that label alone. Proved by a control: with 40 frames between notches and the pre-warm below, no label drew in over 0.8 ms in 20 notches; without the pre-warm, 32 frames had such a label.
  - **Build overshoot:** frames of 17–19 ms where the budget placed settlement or road chunks on top of a 12–14 ms base. The cause is the AIMD target of 17.5 ms, which sits above the bar.
  - **One first-notch frame of ~27 ms** whose cost is in neither the overlay nor the tile path (a 1 ms river chunk in it; unexplained, one per run).
- **The tile path at zoom 3–6 is not ours to cut.** `_process`' tile installation measured 0.4–1.3 ms a frame. The 17–25 ms "no overlay work" frames the previous entry blamed on it were the first pass's 50% guard, which the second pass already shrank (idle frame 17.6 → 9.6 ms at ×4.3). Nothing changed there.
- **Change** (`map_overlay.gd` only):
  - **Label pre-warm:** `_warm_step`, `_list_warm_queue` and `_list_label_glyphs`. When the zoom changes, the glyphs the top layer will need are listed (~1 ms). That covers the new zoom and one wheel notch in and out, per font and cache size (fill and outline). They are then rendered into the font's own cache through the `TextServer`, a few per frame on the main thread, in what the frame's build budget leaves and never in a frame the camera moved. The draw that follows finds them cached.
  - **Main thread, not a worker (coordinator, 2026-09-28).** The first version of this change rendered on a `WorkerThreadPool` task. It was moved to the main thread while an owner-reported new-project stall was under investigation: nothing documents rendering into a `FontFile` cache from another thread while the main thread draws with or replaces that font. Whether the worker version caused the stall is not established here. A scratch stress run of the main-thread version passed and exited 0: 4 new worlds generated with zooming during each generation, then 20 back-to-back notches in and 20 out per world, with the labels re-pushed every third notch.
  - **Build budget:** the target is now vsync-aware (`_frame_target_us`): 16.0 ms without vsync, 17.5 ms with it, where the wall time is the display interval.
- **Pixels:** `_zoomcost_probe --shots` against `7063f4d7`, same DLL, for both versions: 0 px differ at fit / ×4 / ×16 / ×32 / after the pan, on desktop and on the phone form below the status-bar clock.
- **Probes:** `_mapdata_probe` 43/43; `_rivstyle_probe` 0 failures; `_tlpins_probe` 50/50 on desktop and phone; `_ctxring_probe` 132/132 on desktop. No Rust change.
- **Timings of the main-thread version** (2026-09-28 12:39–12:49, `_zoomcost_probe`, same scratch DLL, vsync off, 4 runs each, 24 notches and 120 pan frames per run). No cargo, rustc or other Godot was running before any run (the script waited out another lane's Godot runs several times), and no cargo or rustc after any. Another Godot during a run would not have been seen by the after-check. Log: the lane's scratch `ovl2/timings5.log`. The worker version measured 51 → 25 desktop and 57 → 39 phone form (`timings4.log`).

  | | notch first frame, median / p95 / max | notch frames > 16.7 ms, per run | pan at ×16, median / p95 / max | pan + settle frames > 16.7 ms |
  |---|---|---|---|---|
  | desktop, `7063f4d7` | 11.7 / 17.0 / 22.1 ms | 12, 15, 17, 14 | 6.3 / 9.6 / 14.3 ms | 0 |
  | desktop, new | 11.4 / 17.2 / 20.9 ms | 8, 11, 5, 7 | 6.2 / 9.4 / 12.1 ms | 0 |
  | phone form, `7063f4d7` | 9.7 / 23.6 / 42.5 ms | 12, 19, 15, 16 | 8.3 / 12.7 / 17.3 ms | 1 |
  | phone form, new | 9.6 / 27.4 / 42.1 ms | 12, 9, 11, 11 | 8.3 / 12.5 / 17.9 ms | 1 |

- **Still over:** the first-notch frame above, and a few build frames at 17–18 ms. On the phone form, a few notches' first frames are 20–46 ms on both trees. The pre-warm needs roughly a frame or two to land, so a notch arriving one frame after the last (the probe's back-to-back pattern) can still pay part of a re-raster. Phone-form numbers are the desktop GPU at 1080×2340, not a handset.

**Owner report "starting a new project causes the app to crash/stall" (2026-09-28) — investigated, one defect fixed; verified by the main loop 2026-09-28 (gate diff read; the lane's _newproj_probe PASS with 0 WorldGen panics in the real project on the rebuilt DLL).** New probe `_newproj_probe.gd` drives the desktop flow: welcome → `New world…` → Create → `generation_finished` → screenshot, and optionally a second project over the open one (`--repeat`) with mouse motion, clicks or wheel notches over the map during each generate (`--hover`/`--click`/`--zoom`). All runs windowed, in a scratch copy of the project.
- **Not reproduced as a crash or a permanent hang.** Every run exited 0 with the map drawn: the fresh HEAD DLL and the stale 02:07 one, and each with the `map_overlay.gd` of `e76b7e4a`, of the working tree at 12:35 (the worker-thread glyph pre-warm), and of `5d193bb4`. The stale DLL only degrades: one missing-binding warning (`river_run_count`), no error. So a stale DLL is not the cause. The shared `target/debug/cartalith_godot.dll` was rebuilt from HEAD anyway (13:15), since it lacked today's `#[func]`s.
- **The owner's actual bug, found after clarification ("in a FRESH instance the setup menu doesn't load and the program seems to freeze") — fixed; verified by the main loop 2026-09-28 (size-floor diff read; the lane's real-click cold-start probe FAIL before, PASS after).** The New World dialog opened as a **1 × 1 exclusive modal**: no form and no Create button, with the shell blocked behind it. Cause: the owner's `user://cartalith_settings.cfg` held `new_world_dialog@desktop=Vector2i(1, 1)`. `DccWidgets._desktop_window_size` restored it, because that dialog sets no `min_size`, and hiding the dialog then saved 1 × 1 back. The earlier legs missed it because `pressed.emit()` on the OK button ignores geometry. Fix (`dcc_widgets.gd`): new `_window_size_ok` with `WINDOW_SIZE_FLOOR` (200 × 150, a labelled judgement), and a saved size under the floor or under `min_size` is neither restored nor saved. How the first 1 × 1 was written is not known. New leg `_newproj_probe --real-click 1`: a cold start, then mouse clicks at the drawn `New world` and `Create`. Before the fix, the dialog was 1 × 1 and Create started no generate (FAIL). After the fix: the dialog is 620 × 612 with the full form (screenshot opened), the generate runs, `RESULT PASS`, exit 0, 0 panics, and the settings entry now reads `Vector2i(620, 612)`. Not driven: `File ▸ New` (it calls the same `app.open_new_world()`, so the same fix covers it) and the phone picker (its path uses `phone_present()` and does not use this restore).
- **Fixed: map input during a generate reached the borrowed engine.** A second project started over an open world, with the mouse over the map, panicked `Gd<T>::bind() failed, already bound; T = WorldGen` on every motion or click. The failing reads were `sample_cell` and `get_settlements` (right dock Sample), `river_at` (a click), and `sculpt_stamp_count` (the dock rebuild). With `--hover 1 --click 1 --repeat 2` this gave 168 panics on HEAD with either DLL, and 0 after the fix. Fix: `viewport_host.gd::_generating()` gates every relayed map signal (select, hover, sample, click, drag, release, right-click, context) while `EngineBridge.generating`. That is the same borrow `_engine_readable()` already guards for the viewport's own reads. The first project is unaffected: `has_world` is false then, and the listeners take their no-world branch.
- **Measured, not changed: the main thread blocks after every generate** (the `generation_finished` handlers). The block is 2.4–2.7 s at 2048 wide, 9.5–11 s at 4096, and 35 s at 8192 (8192: 125 s of generation, then 21 s in the viewport's refresh lambda and 11 s in `CivilizationWorkspace`). At 4K and above, Windows shows it as "Not responding". Ruling BP accepts a freeze during generation, so this is reported here, not cut.
- **Open:** Windows logged hangs (event 1002, "closed after it stopped interacting") of `Godot_v4.7.1-stable_win64.exe` at 12:10 (started 12:09:27) and 12:19 (started 12:10:05, about 9.5 min). Whether those were the owner's runs, and which world size they used, is not known from here. The worker-thread pre-warm that was in the working tree then is gone at `5d193bb4`. Proof run in the real project with the rebuilt DLL: `_newproj_probe -- --hover 1 --click 1 --repeat 2` gave `RESULT PASS` and exit 0, with 0 panics, 0 missing-binding warnings and the map on screen. No Rust changed.

**Tile seam bar on 71077345/512 pan: passes again, no code change -- re-measured 2026-09-28, verified by the main loop (log re-read).** The row filed after RV-1 (seam ratio 1.92 against <= 1.5) was re-measured on the current tree with `_lodsweep_probe`: seam-ratio median 1.114 (0.943..4.587, n=120) rivers hidden, 1.106 with `--rivers-on`; PROBE-RESULT: PASS. The fix landed somewhere in RV-4/RV-5's shore and river rework; not bisected. Open: one frame reads 4.59 alongside one 6664-px hole frame and 1 pop, which looks like a tile-arrival transient (not captured). Logs: the lane's scratch `seam/out0`, `out1`. The other 17 D0 configurations were not re-run.

**River colour along one course — built 2026-09-28, verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the seed-1 continuation crops opened)** (OUTSTANDING_WORK.md row "Rivers change colour abruptly mid-course, and one ×1 trunk vanishes into a sea-classed valley").
- **(2), the sea-classed valley: already gone, no change made.** `_riverzoom_probe --seeds 1 --zooms 1,4` on the pre-change tree: `ocean_cells_on_path` 0 over 1045 drawn rivers; the ×1 and ×4 trunk frames show the trunk running on into its lake, with no pale band. RV-1 had fixed it, as the row's strike-through says.
- **(1) was real, and had two causes.** Both are in `lib.rs::WorldGen::river_draws`. First, the per-point colour was the nearest traced cell's `RIVER_ORDER_RGB` entry, so it jumped one palette step (26–46 levels) between two render points where the order rose. Second, a `river_draw_plan` continuation (a run carried past a land pit onto the next run's head) restarted at the head run's own order. It began pale, and if its own order was 1 it was also drawn at the order-1 de-emphasis alpha (0.4).
- **Fix.** The colour is now read at a blended order: `river_stroke::blended_orders`, the trailing mean of the traced order over `colour_ramp_cells(gw)` = max(gw/128, 4) cells (a labelled judgement). The palette is then read between entries (`palette_at`). `carry_orders` raises a continuation's points to the order the incoming run ended in, and gives each continuation group one `own_order`, so the group shares one de-emphasis. `incoming_orders` starts the blend at the incoming run's end colour. Confluences are untouched: a tributary still meets its trunk in its own colour, and colour still only darkens downstream. `get_rivers()`' `orders` stay the traced cell's own orders; `own_order` is now the group's. Screen, tiles and export all read `colors` and `own_order`, so all three change. No JS golden and no water classification changed.
- **Measured** (`_rivcolour_probe.gd`, new, windowed; 1024×656; seeds 483920 / 24601 / 1; before = the tree before this change, DLL built into scratch):
  - Steps of 15 levels or more between consecutive render points: 405 / 438 / 343 before, 1 / 0 / 0 after.
  - Continuations whose colour steps: 100 / 105 / 79 before, 0 / 2 / 1 after.
  - Continuations whose alpha steps: 132 / 144 / 105 before, 0 / 0 / 0 after.
  - Positive control, runs whose colour changes along their course: 348 / 386 / 300 before, 278 / 312 / 239 after.
  - Screen walk at ×4 along the continuation targets, largest step between 1-cell stroke samples: 2 / 43 / 52 before, 8 / 19 / 7 after (ground control 5 / 13 / 10). The walk skips samples near river ends, so it cannot read the order-step targets, which sit at confluences. Its ×1 readings are noisy on 1–2 px strokes.
- **Looked at:** `scratchpad/final_{before,after}/s*_z004.0_crop3x.png`. Seed 1: a trunk that was drawn dark, then translucent grey, then pale across a bridge is one continuous blue course after. Seed 24601: the confluence step now blends over several cells. Seed 483920: the grey 0.4-alpha segment on the Province course is gone.
- **Rust:** 6 new tests in `river_stroke::tests`. Mutation testing in a scratch copy: 23 mutants, 22 killed. Two survived the first round; one was killed by a new assertion. The other, `ramp <= 0` → `< 0`, is equivalent and documented at its symbol. The `lib.rs` wiring cannot be reached from `cargo test` (`WorldGen`); the probe covers it. `cargo test -p cartalith-godot`: 1511 passed / 0 failed / 34 ignored before, 1517 / 0 / 34 after. `_rivstyle_probe`: 1 failure (M3 Ink ocean mouth 7, 6 px), identical on the before DLL, so it predates this change.

**White rivers at fit zoom, fixed 2026-09-27; verified by the main loop** (`_rivstyle` 0 failures including the new screen-pixel section S, `_stylepresets` 0 failures, `_riverzoom` ×1 on 1024: 2/0 rivers in 3+ pieces with 0 ocean on path; the repainted Default screenshot inspected). Cause: `map_overlay.gd::_draw_rivers` recorded only the river colour texture's RID in the canvas item. The next `build_color_texture` freed that texture, and the renderer drew the dangling RID as its default white until something redrew the overlay, which about a dozen shell repaint paths never did. Fix: `engine_bridge.color_texture()` emits `color_texture_rebuilt`, which the overlay connects to `queue_redraw`, and the overlay holds `_river_tex` so the recorded RID cannot dangle. Section S of `_rivstyle_probe` reads SCREEN pixels at fit and ×1.4 for Default, Blueprint, Ink and Night, after the preset is applied and after an app-style repaint, against a control measured in the same run: HEAD 14 failures, fixed 0. `_riverstroke_probe` still crashes with an access violation at exit after printing ALL PASS. That crash predates this fix (it was seen at `2cf0143`) and is filed as its own row.

**Probe exit crash (access violation after the verdict), fixed 2026-09-28; verified by the main loop 2026-09-28 (the staged tree built and tested alone in a worktree; the pool shutdown code read).** Cause: the deep-zoom tile pool (`lod_worker::pool`, a rayon pool in a `static`) was never stopped, so a probe that quits with tiles in flight left its threads running `LodSnapshot::render_tile` while Godot unloaded the DLL. Bisected in a scratch copy of `_riverstroke_probe`: quitting after section C, or after D but before its final rivers-layer flip (which re-queues the view), or after that flip once the pyramid settled, all exit 0; quitting straight after the flip exits 139 (5 of 5 full runs on the pre-fix DLL). Fix: the pool keeps its threads' `JoinHandle`s (`spawn_handler`), and `lod_worker::shutdown_pool`, called from `CartalithExtension::on_stage_deinit(InitStage::Scene)` in `lib.rs`, drops the pool and joins every thread before unload. After: `_riverstroke_probe` ALL PASS and exit 0, 3 of 3 runs. Unchanged before/after, same pre- and post-fix DLLs, windowed: `_glaciallodkey`, `_lodlevels`, `_tiledlod`, `_stylepresets`, `_sculptlodcache` exit 0 on both; `_rivstyle` exits 1 on both with the same failure (`M3 Ink: no river pixel on open water at z16`), not this change's. Not reproduced on this tree, before or after the fix: `_riverzoom_probe` (full sweep and `--stats-only`, exit 0) and `--headless --import` (exit 0). `cargo test -p cartalith-godot`: 1517 passed, 0 failed, 34 ignored after. The pre-fix copy scored 1516 passed and 1 failed; the failure was `golden_parity_tile_biome`'s reference-path test, which cannot find `reference/` from the scratch copy. Not covered: rayon's global pool cannot be joined. Its callers are synchronous `par_iter`s, so it has no work in flight at quit.

**`_rivstyle_probe` M3 "Ink: no river pixel on open water at z16", fixed 2026-09-28 — verified by the main loop 2026-09-28 (probe diff read; the lane's mutant, ocean under a stroke drawn as land, fails the fixed leg).** Neither a stale expectation nor an Ink regression: a probe measurement defect. The leg asserts the current behaviour (owner, 2026-09-27: *"the ocean texture should be drawn above the river graphic"*) — NO river pixel on open water — so it was kept as is. Cause: `_mouth_deep`'s control (the largest change on pixels far from every river) was read on every second pixel of every second row, while the leg judges every pixel. On Ink mouth 7 all six flagged pixels were far pixels in open sea, ~30 cells from the mouth, changed by 9 (3 per channel) in the tile rebuild's contour-shaped noise, against a sampled control of 8. The largest change on near open-water pixels in that view was 5 (per-pixel histogram and diff image, scratch diagnostic). Fix: the control is read on every pixel (`_rivstyle_probe.gd::_mouth_deep`, comment cites this row). Full probe windowed on a scratch DLL of the current tree: before 1 failure, after 0 failures, exit 0. Screenshots opened: the Ink mouth 7 and lake mouth 2 z16 crops show strokes reaching the shore and none on the water. Mutation (scratch workspace, ocean pixels under a river stroke drawn as land): the fixed leg fails on both presets, 6 of 8 mouths each, 30-886 px. No Rust changed.

**Rivers through the style (owner, 2026-09-27: *"the only issue I have with the rivers: they're drawn on top of the style"*; *"can we put the rivers into the map again with the same vector approach?"*) — built 2026-09-27, verified by the main loop 2026-09-27 (`_rivstyle` 0 failures, `_riverstroke` ALL PASS, `_rivlake` PASS re-run; the staged tree built and tested alone in a worktree without the GF-2 lane's engine files).** RV-2's strokes keep their vector shape and now take every preset's colour, Painter styles, paper and grade. What changed:
- **One geometry.** `lib.rs::WorldGen::river_draws` builds every run's drawn stroke: render points, pieces, per-point widths, colours, Strahler order and discharge, and draw rank. `get_rivers()` marshals it. `river_geometry()` is the cached, owned copy in draw order. Its key is the world epoch; the Height, Climate and Hydrology stage versions summed over every tile; the grid; sea level; wrap; and map width (`river_network_key`).
- **One width seam.** Width and visibility go through `river_stroke::river_px_width(point {width, order, own_order, discharge}, px_per_cell, preset)`. Today its body is RV-2's rule: the reference's order-1 de-emphasis (v2.11 9577-9578) keyed on raster density, and the 1 px floor. The zoom-sensitive-rivers lane replaces the body.
- **Deep-zoom tiles** rasterize the strokes at the tile's own resolution. `river_stroke::rasterize` fills a `render::RiverLayer` (premultiplied, sparse 32-px blocks, top-left rule, composited in draw order), and `render::land_color` composites it before the Painter block. The tile carries river coverage in its alpha, and `lod_tile.gdshader` draws river pixels from the tile alone. Culling grows each segment by its width and fringe, so strokes don't seam at tile edges.
- **Base view (below the deep-zoom switch).** The map texture holds no river. `build_color_texture` also builds `river_color_texture`: the same map with every river composited at full coverage, in its styled colour, through `land_color` and every stage after it. `rasterize_colour_field` builds this over a band `colour_field_pad_cells` wider than the river (two passes, so each river keeps its own colour on its own cells). `map_overlay.gd::_draw_rivers` draws RV-2's stroke in screen pixels (`WorldGen::river_view_mesh`, through the same seam at the screen's density), textured with that texture. So the shape and antialiasing are the vector's, at screen resolution, and the colour is the style's. The overlay draws nothing while tiles are up, under a field view, or with the layer off.
- **A one-texel-per-cell raster in the base texture was built first and refused.** Nearest-filtered below the switch, it was stair-stepped, which was the coordinator's blocker.
- **Where the river sits.** It is composited into the lit colour just before the Painter block (the paint brush's slot), so sepia, crosshatch, stipple, risograph and the village quantiser all act on it. `river_through` at 0 lays it over the Painter styles instead. Paper, frame, local contrast (measured from the river-free terrain), the grade and the colour space follow. Terrain AO is not applied under the water. Sea and lake pixels are untouched. The export, `js_reference` and golden paths attach no layer, so exports still stamp `river_ink`. No JS golden moved.
- **Per-preset treatment.** Seven new tunables: `river_width`, `river_opacity`, `river_ink` (toward `river_ink_r/g/b`), and `river_through`. They are grouped as a "Rivers" group in the render workspace. 10 of the 15 `STYLE_PRESETS` carry a `river_*` bundle.
- **Layer switch.** It sets the engine flag, redraws the overlay and rebuilds the tiles; `lod_cache_key` carries the flag. The base texture is not re-rendered. `river_strokes_mesh`, `_show_rivers` and the overlay's `_rivers` copy are gone, and `refresh()` no longer marshals `get_rivers(1)`.

Measured windowed on this machine, with the DLL rebuilt after the last `.rs` change:
- **`_riverzoom_probe`, ×1 and ×1.4, seed 483920, same targets, old overlay (HEAD from `git archive`) → now.** Diff-mask components:

  | Grid | ×1 | ×1.4 |
  |---|---|---|
  | 384×288 | 101 → 102 | 73 → 75 |
  | 1024×656 | 235 → 243 | 174 → 176 |
  | 2048×1312 | 186 → 195 | 163 → 176 |

  With a full-contrast black ink (`--appearance river_ink=1,...`) the same frames read:

  | Grid | ×1 | ×1.4 |
  |---|---|---|
  | 384×288 | 100 | 74 |
  | 1024×656 | 237 | 175 |
  | 2048×1312 | 181 | 162 |

  So the geometry is within ±2 of the overlay's (and below it at 2048). The excess in the default look is the paper-toned headwater colour falling under the probe's 24-level diff threshold, not breaks. Screenshots show smooth, continuous strokes on both grids.
- **Continuity metrics.** Rivers in 3+ pieces **2/0/0**. Ocean on path **0/0/0**. Opening-view frame with rivers on: 16.6 ms (vsync) at 1024 against the overlay's 21.9 ms, and 32.1 ms against 33.9 ms at 2048.
- **`_rivstyle_probe.gd` (new): 0 failures.**
  - The river colour texture carries the rivers on 9 presets, and the map texture is byte-identical with the layer off.
  - Blueprint is white (luma 237), Woodcut black (11), Ink 53, Ink wash grey (chroma 4). Vintage atlas is muted (15 against 96 chroma). Antique is sepia (b−r −13). Night is blue (b−r 75, +47 luma over the ground).
  - z16 tiles match the texture at the same cell within 0.3–1.7 levels.
  - A sculpt far from tile 0 moves the river network key, and all 80 fresh river cells are drawn.
- **Costs**, median (min..max) of 5:

  | | 1024×656 | 2048×1312 |
  |---|---|---|
  | `color_texture()` (map + river colour texture) | 294 ms (292..301) | 1182 ms (1164..1203) |
  | Earlier rivers-off build, same machine | 224–226 ms | 925–927 ms |
  | Colour field | 7.1 MB in blocks, 146k px | 20.4 MB in blocks, 519k px |
  | Base-view mesh per overlay redraw | 9.3 ms (422k triangles) | 19.2 ms (837k triangles) |

  From the earlier build, the river texture therefore adds about +70 ms at 1024 and +256 ms at 2048. It also adds one more `gw×gh` RGB texture (8 MB at 2048). The switch itself is under 0.1 ms plus the tile rebuild.
- **`_riverstroke_probe`: ALL PASS.**
  - Width bar, restated from first principles. The chord is the ground width W plus a raster constant c: the fringe and bilinear reconstruction, sized in *tile* pixels. At zooms exactly 2× apart the pyramid moves one level and the magnification is identical (measured 1.181 at both). So w2 − w1 = W2 − W1 is the bar; the ratio (2W+c)/(W+c) is 2 only if c = 0. Measured: difference 7.0 px against 7.00. c = 3.0 px, inside its bound of 0..4·mag = 4.7.
  - Colour bar: the centre pixel against the river colour texture at the same point must be within the bank's tile-vs-map drift (the control) + 4 levels. Measured 2.8 against 1.3 + 4 and 2.8 against 2.0 + 4.
  - Unbroken: 809/809 and 397/397.
- **`_riverstroke_probe` leg D, re-derived after the verifier's run** (19.5 here, 20.5 there, against the old bar 0.3 × 67 = 20.1).
  - Stable, not flaky: three runs on the same DLL all read 19.5.
  - Not ink: HEAD's own tile renderer, on the same world (HEAD plus the GF-2 lane's uncommitted engine files, from `git archive`), reads 19.0 with no river anywhere in its tiles. The world moved under GF-2, and its chosen trunk sits in a valley whose own blue shift is ~19 levels (the near-channel wetness tint and wet ground).
  - The old bar assumed that valley share was small. The new bar measures it on the one raster that holds no river by construction, the map texture, at the same ground points: 21.4. OFF must read ≤ that + 4: 19.5 passes.
  - Positive control added: the same measure with the layer ON reads 136.5, far above.
  - Leg D now waits for the pyramid (`lod_pending() == 0`) instead of 90 frames.
- **`_riverstroke_probe` continuity.** On the new world the chosen trunk runs into the plate frame. The neatline correctly hides it there, and samples under the frame read as breaks: 1199 of 1342 here, and 1200 of 1442 for HEAD's overlay on the same frame. The check now keeps to the neatline's interior: 1177 of 1177 and 762 of 762. All pass, three runs.
- **`_rivlake_probe`: PASS.** Its "> 2 levels" tolerance is replaced by a control: the same state re-rendered after a round trip of the switch. Both switch and control read 0.0 levels over the lake once each frame waits for the pyramid; the earlier 1 level was an unsettled frame. The river reads 113 levels.
- **Seam 1.90 on 71077345/pan is not this change.** `_lodsweep_probe` (512×384, pan, rivers hidden unless noted):
  - `git archive` builds of HEAD and 8c2b11d (pre-GF-1): both 1.92.
  - 5665137 (pre-RV-1): **1.06**. dd836f6 (RV-1): **1.92**.
  - This tree, rivers on: 1.89.
  - So RV-1's carve, which changes that world, moved it, not GF-1. It is reported, not fixed here.
- **`_stylepresets_probe`**: 0 failures, 15 presets. Before and after sheets were compared.

Known costs:
- **Night is dimmer** than the old overlay's cyan: its river goes through the preset's −0.35 exposure. That is ruled, not open: Night's rivers go through the grade (Ruling BL, `LARGE_ITEM_RULINGS.md`).
- **Contrast in the default look.** The default look's headwaters are paper-toned, so they are lower-contrast than the overlay's raw palette.
- **Stored pyramids.** A stored LOD pyramid drops the tile alpha.
- **Narrow views.** A fit view narrower than 800 px on a very large grid lets a 1 px stroke reach past the colour band. There it samples terrain and reads thinner.
- **Exports** still stamp `river_ink`.

Probes and shell:
- `_owner5_probe --rivdbg` was removed: it rewrote the overlay's river list and read its retired one-width helpers.
- `_rivervec_shot` was updated.
- `_riverzoom_probe` gained `--appearance` and fractional-zoom file names.
- `_lodsweep_probe` gained `--rivers-on`.

`cargo test --workspace --no-fail-fast`: **4040 passed, 0 failed, 44 ignored** over 181 result lines. The baseline at the start of this lane was 4016/4/44 over 180 lines; its 4 failures were the concurrent GF-1 lane's then-uncommitted tests. Mutants:
- 18 of 20 were killed in round one, and the two survivors were killed after a test was added.
- 9 of 9 were killed on the colour field.
- One equivalent mutant survives: the seam's `None` split, unreachable while the seam never declines a point.

**RV-1, the carve no longer leaves pits (Ruling BD) — built 2026-09-27, verified by the main loop 2026-09-27 (workspace 3998/0/42; the recovery test's +50 km margin replaced by a strict inequality plus a searched exact-branch seed, and its duplicate/drop mutants re-killed).** `generate_terrain` no longer runs `enforce_channel_descent` once per traced run. It runs `cartalith_hydrology::carve_channel_network` once over the whole network. `enforce_channel_descent` remains the reference port and the Sculpt River stamp's carve (`sculpt_commit.rs`), and generation no longer calls it. A scratch harness isolated four mechanisms on the probe's worlds, and each has a fixture:
- **Diagonal steps (the largest).** Runs are D8, but `build_water_bodies` uses 4-connected components and a 4-connected flood. A trench cell reached only diagonally was sealed on four sides. Below a half-width of 1 cell, which is the norm above 800 km, every diagonal step did this. Each diagonal step now also cuts the lower of its two orthogonal connectors to the downstream floor.
- **Confluences.** The trunk is carved first. A tributary that arrived with a lower floor then sank the confluence below everything downstream. Floors are now settled over the network before anything is cut: last-traced runs first, and a confluence takes the minimum of its inflows.
- **Runs that stop short.** A channel ends where the next cell fails the slope-area test, usually on a depression's flat lip. The run's accumulated `drop` had already cut its last reach below that uncarved receiver. This left 10–30-cell channel-shaped "lakes". No run is now floored below the level it drains to: its receiver's terrain, its confluence's settled floor, or the surface of any lake it crosses downstream.
- **Cutting land under sea level.** The floor limit was `sea - 0.06` everywhere, so long lowland trunks cut below sea level, and at the coast the ocean flooded the valley. No cut now takes a land cell below `sea + CARVE_LAND_MARGIN` (1e-4).

Lakes that runs cross are now kept. The integrated-drainage routing surface (`lake_surface`) marks every depression deeper than `CARVE_KEEP_LAKE_DEPTH`. This is `build_water_bodies`' own 0.004. The run's floor there is the lake surface, and no disc is cut there. The reference's carve trenched these outlets.

`_riverzoom_probe` (extended with map-wide lake totals, a channel-shaped/basin split and ocean-on-path). It was run windowed, from `git archive` copies of `1750821` (before) and the same tree plus this change (after), with each DLL built from its copy. Seeds 483920 / 24601 / 71077345 at 1024×656, before → after:
- **Traced river cells that are lake:** 18.3% / 11.9% / 11.4% → 10.8% / 2.3% / 0.8%. What remains on 483920 is trunks crossing its two interior seas, which are 14 650 and 41 688 cells.
- **Lake runs of 1–3 cells on channels:** 149 / 103 / 114 → 58 / 7 / 13.
- **1–3-cell lakes, map-wide:** 346 / 607 / 534 → 29 / 26 / 16.
- **Ocean cells on channels:** 191 / 129 / 0 → 0 / 0 / 0.
- **Channel-shaped lakes of 10+ cells:** 28 / 35 / 23 → 5 / 2 / 0.
- **Basins of 100+ cells survive:**
  - 483920: [14557, 41799] → [466, 14650, 41688]
  - 24601: [101, 134, 147, 294, 410] → [150, 234, 308, 319, 1382, 1544, 1971]
  - 71077345: [141, 202, 223, 2851] → [140, 202, 231, 257, 2817]
- **Total lakes:** 434 / 727 / 625 → 54 / 54 / 32. Total area: 59 412 / 4 856 / 5 607 → 58 193 / 6 539 / 4 005. The area lost is the artefacts above.
- **Rivers in 3+ pieces:** 11 / 18 / 16 → 2 / 0 / 0.

Seed 1 was run as well. Lake cells on the path went from 20.4% to 5.4%, and channel-shaped lakes from 23 (6 311 cells) to 1 (11 cells). A comb of lake strips at ×8, where the trunk vanished, is now continuous rivers. An 11 596-cell basin that the old carve half-drained is now one lake.

PNGs looked at by the lane:
- At ×8 and ×32, stepped teal lake strips along rivers are gone, and the strokes run through.
- At ×1 on 483920, the dark one-cell specks are gone.
- On 71077345 at ×32, the trunk is continuous.
- Parallel straight runs across flat basins remain. They are a routing-surface artefact, not RV-1's.

Goldens:
- **Re-recorded, as the ruling says.**
  - `golden_parity_carve.rs`: the carved world is now pinned as exact FNV hashes of this port's output. 14×11: field `840b6f763f0a08d5` → `dc93cab994c3d56a` (47/154 cells, max |Δ| 0.064); river_mask 58 → 54 cells. 16×12 wrap: field `bf4fa3d78f04cc70` → `e60433adbe7b4830` (37/192, max 0.067); river_mask 54 → 50. temperature, rainfall and flow_discharge hashes also moved; the old and new values are in the file.
  - `world_structure_orogeny.rs`: `0x916a11930abef69e` → `0x82a370d51c2cb23a`, and `0x34ab5acc582af633` → `0x93ed9ddf927390ad`. Only the carve moved them.
- **Not re-recorded: pinned instead.** 16 civ JS-parity suites under `cartalith-civ/tests/golden_parity_*.rs` are pinned to the reference's world: affordance, biome, carrying capacity, civ tools, faction aggregates, hierarchical network, resource potentials, road consolidation, road network, sea routes, settlement naming, placement, prereqs and suitability, smelting/salt, and water bodies.
  - They test civ ports against JS captures taken on the reference's world. Re-recording them from the new world would make each a snapshot of itself, which is the outcome this file already refused for craters (§7l).
  - `cartalith-engine/tests/fixtures/pre_rv1_world.rs` (+ `pre_rv1_worlds.bin`, 164 KB) holds the six arrays the carve changes: field, temperature, rainfall, flow_discharge, river_mask and river_floor. They were captured from `1750821` for all 7 configurations those suites generate. Every other `WorldState` field was hash-compared and is identical.
  - `pin()` asserts the world's pre-carve `stream_order` hash, and `golden_parity_carve.rs` asserts the capture against the reference's own arrays, within its old tolerance.
  - Without the pin, 27 tests in 14 of these files failed, on the lane's first build of this change. That run was not repeated on the final build.
- **Flagged, not changed:** `cartalith-godot` `civ_pipeline_tests::recovery_keeps_every_road_on_the_settlements_it_joined`. Its premise `rebuilt > filtered + 50 km` was measured on its one world. Before: 5 abandoned nodes, 1124.7 → 1242.9 km (+118.2). After: 4 nodes, 1412.4 → 1454.4 km (+42.0). The invariant "rebuild never draws less" still holds. Lowering the margin would loosen a test, so it is left red for a decision.

Rust:
- 12 new tests: 7 `carve_network_*` in `cartalith-hydrology`, 4 in `cartalith-civ/tests/carve_leaves_no_pits.rs` (the real classifier), and the capture check. Each fixture has a positive control: the old per-run carve must show the lake or pit.
- 12 mutants on `carve_channel_network`, all killed. They were run by Python exact-replace, restored in `finally`, the file hash-checked and no residue left. A 13th mutant was equivalent: the centreline land clamp duplicated the cut's. It was removed rather than kept.
- `cargo test --workspace --no-fail-fast`: **3997 passed, 1 failed (the flagged test), 42 ignored.**

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
| GGR-CA19 | CA-19 — the biome colour table: a picker, and every reader honouring the override (Ruling P) | done (verified 2026-09-24) | **Built:** CARTO ▸ Colours ▸ Biome colours draws one `ColorPickerButton` per class, a per-row Reset and Reset all (`render_workspace.gd::_build_biome_colours`, over `engine_bridge.gd`'s `biome_colors_api` wrappers); the map's paint blend already read the override (`WorldGen::appearance()`). `sample_bridge::legend_with` / `debug_raster_with` and `PaintEditor::preview_full_with` / `preview_patch_with` take the live table (`biome_legend_and_raster_follow_an_edited_table`, `biome_previews_take_the_table_they_are_handed`). **Wired (`30025bf`):** `debug_layers`, `build_debug_texture`, `build_paint_preview_texture` and `build_paint_preview_patch` pass the live table, so the legend, the Biomes field and the paint preview follow an edit; `_biomecol_probe.gd` passes on desktop and phone. **Persisted (2026-09-24, verified):** `appearance.json`'s optional `biome_cols` member (`SAVEFILE_COMPAT.md` §13.2; `project_bridge.rs::biome_cols_member` / `biome_overrides_from_member`), omitted when the table is the reference one; an edit marks the project dirty (`engine_bridge.gd`). Tests `project_bridge::biome_cols_tests` (4, from a HEAD-written `appearance.json`) and `_biomesave_probe.gd`. *2026-09-24, verified:* opening a flat or legacy archive no longer inherits the previous session's edits — `WorldGen::load_save` resets the members `project_open` restores wholesale from `appearance.json` (overrides, ramp, NPR block to the session default, biome table), and that restore still wins; `_lookreset_probe.gd` (30 checks; 9 fail against the pre-change DLL) |
| GGR-53 | §53 — four registered-not-fixed phone items (`⌕` and `⋮` not in the app bar, bottom-nav/sheet colour literals off by 2/255, tool-options sheet still resident, phone pill upper-cases once) | partial | **The `⌕` half is unblocked and its stated reason is stale**: §53 says "`⌕` has no destination — `menus.gd`'s Edit ▸ Find on map… is a `_todo()` row". It is now `_live(p, "Find on map…", ID_FIND_ON_MAP, KEY_MASK_CTRL \| KEY_F)` backed by `shell/place_search.gd`. The `⋮` overflow still exists as floating phone furniture (`_phone_overflow_pop`), the two colour deltas are unchanged, and `dcc_shell.gd` still calls `phone_fit(tool_options_row, …)` on a resident row rather than presenting a sheet |
| GGR-50 | §50 — six registered-not-fixed items from the OnePlus 6T pass | partial | **The memory item is disclosed, not fixed**: the Working set row (`menus.gd::_refresh_working_set_row`, which replaced `performance_window.gd` under ruling 19) still reports `OS.get_static_memory_usage()`, which excludes the Rust allocations and the ~544 MB of Gfx dev that `dumpsys meminfo` counts, with a note naming the source and §50's own handset figure (0.2 GB on screen vs 818 MB PSS). The other five (Label ellipsis hole, DS-12 duplicate class token, navpad hover tint, stock focused pane-switcher chip, two `✕` on the L2 sheet header) were not re-verified individually this pass |
| GGR-51 | §51 — the registered-not-fixed menu-conformance set | partial | Mixed, and worth splitting. **Settled by a newer authority**: the tool-options bar height, now `--tbH` 40/56 in the 2026-08-31 spec, superseding §51's "third owner decision". **Disclosed rather than fixed**: `tool_bar.gd` builds the freehand row from `bridge.get_sculpt_freehand_modes()` live and adds the note "The canvas's Flatten, Noise and Mask have no engine mode at all; these eight are `FreehandMode`'s own list, read live." **Unchanged**: the badge right-column (a real Godot `PopupMenu` limitation), the tablet dock contents (that is DS-03), and the sculpt feature-name gap (an engine gap). Two rows §51 listed were already proven stale by §53's own probe |
| GGR-DS03 | DS-03 — the tablet interior is desktop-sized; the tablet artboard is a content decision, not a scaling layer | blocked | An owner **content** decision — *which controls leave the tablet* — not answerable from the canvas. Under it sits a real architectural blocker: `dcc_theme.gd`'s `const TABLET := {36: 52, 34: 52, 30: 44, …}` is keyed by the bare desktop integer, and the artboard maps one desktop figure to two tablet figures in at least five places, so a value-keyed table cannot express it; §57 also refuted the obvious role-keyed placement. One mechanism claim in §51 has drifted: `phone_fit()` no longer opens `if not _phone: return` — the gate moved to its callers — but the **verdict** DS-03 rests on is still correct |
| GGR-DS13 | DS-13 — the phone viewport control column (navpad, zoom, layers FAB) does not match the canvas | not started | §57's pass produced a design and its own audit refuted it on four high-severity counts. Two re-confirmed here: `viewport_host.gd`'s left-button branch is `elif mb.button_index == MOUSE_BUTTON_LEFT and _pan_mode:` with no armed-tool condition (so `tool_pan` is navigation after all), and `_build_navpad()` opens `if not _touch: return`, so touch tablets get the same pills. `menus.gd` has no View menu, confirming that zoom's stated destination does not exist. **Nothing was built and the register says so** |
| GGR-RELIG | The religion-diffusion screens (§57) — designed, audited, refuted as premature | **stale — corrected 2026-09-23, a 2026-09-03 batch already shipped the data path this row says is missing** | This row's own "no data path" claim was already false when written: `get_settlements()` emits `religion` and `adherents` per settlement (confirmed at the symbol, `RD-1` above), and a 2026-09-03 batch shipped the religion screens against that data — `OUTSTANDING_WORK.md`'s own record of that batch is the correction this row should have carried since. Left as a defect-in-the-record example rather than silently deleted, per this file's own discipline of naming what it got wrong |
| GGR-05 | §5 — omissions: designed, not present, not even as a disabled item | unverified | Not re-verified item by item. §37, §39, §42 and §45 close large parts of it and §5 was never rewritten to reflect that, so its current accuracy is unknown. Confirming each absence requires the canvas alongside the shell — **flagged rather than guessed** |
| GGR-EXIT | `OUTSTANDING_WORK.md` §2.11's "Exit buttons are inconsistent across windows" — **built, verified 2026-09-28** | done (verified 2026-09-28) | **Ruling AZ** (`LARGE_ITEM_RULINGS.md`, 2026-09-28) answered the row's own owner-sign-off blocker: Close for a window that only shows things or applies changes live, OK/Cancel only where changes are held until a confirm. Applied through one helper, `dcc_widgets.gd::phone_window()` — `ok_button_text = "Close"` used to run only on its phone branch; hoisted above the branch so every plain `AcceptDialog` that calls it gets Close on desktop and tablet too, while every held-change caller (`new_world_dialog.gd`'s "Create", the vault's "Write to Markdown", `menus.gd`'s "Save"/"Replace"/"Forget") already overrides it afterward and is unaffected. Fixes the row's own three named windows — `place_editor_window.gd`, `faction_roster_window.gd`, `settlement_types_window.gd` — plus two more of the same latent bug class in `app.gd` (`open_storage_locations()`, `open_about()`), found by the same inventory and fixed for free by the shared helper without editing `app.gd`. Verified by a new windowed probe, `_exitbuttons_probe.gd`/`.tscn`, that opens every `AcceptDialog`/`ConfirmationDialog` this pass could reach (excluding the CM-7-owned files: `map_overlay.gd`, `shell/context_*`, `shell/workspaces/*.gd`, `shell/engine_bridge.gd`, `shell/app.gd` itself, `shell/right_dock.gd`) and reads each one's real exit-control text, with a positive control (a hand-built `AcceptDialog` that skips `phone_window()`, still stock "OK") proving the checker is not vacuous. Mutation-tested: reverting the hoist makes the probe fail on exactly the five windows that depend on it and no others. `_vaultbrowse_probe.gd` and `_currency_probe.gd` re-run clean, unchanged (neither asserts on an OK/Close label this batch touched). Marked verified 2026-09-28 per this file's own discipline, not because a check failed |
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

**293 milestone rows across 32 subsystem groups**, recounted 2026-09-26 after merging `main` (the new group is *Map context*, `MAP_CONTEXT_SCOPE.md`, 7 rows; Ruling AW moved MM-6/7/8 off declined); before that, 283 across 31, recounted 2026-09-24 from
the tables above (278 across 30 on 2026-09-23; the new group is *Travel
Library*, five rows). Method: one script run classifying each row by the
leading word of its Status cell, with `done*` counted inside `done` — the same
script reproduces the 2026-09-23 figures exactly when run over that version of
this file. Shares are rounded and do not sum to 100.

| Status | Count | Share |
|---|---:|---:|
| **done** (225, of which 7 are `done*`) | 225 | 77 % |
| **not started** | 25 | 9 % |
| **declined** (deliberate, with the reason in code or a ruling) | 15 | 5 % |
| **partial** | 16 | 5 % |
| **unverified** (not a code artefact) | 5 | 2 % |
| **blocked** (a named blocker) | 4 | 1 % |
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

**Where the 18 not-started rows were** (recounted 2026-09-24; *the 2026-09-26 total is 25 — add the Map context proposal's five unblocked rows and MM-6/MM-8, reopened by Ruling AW*).

| Subsystem | Not started | Note |
|---|---:|---|
| Religion diffusion | 6 | RD-2…RD-7; the foundation and milestone 1 are built |
| Sculpt live | 3 | L1, L2, L4; L0 is done (2026-09-24 correction) and L3 is declined by design |
| GUI replacement | 4 | Stages 3, 5, 6, 7 |
| Options kept open | 2 | Store distribution and WASM; neither is work until someone commits to it |
| Memory optimisation | 1 | MEM-12 (R6), ranked low on purpose |
| Export | 1 | EXP-E5, the dialog |
| Gap register | 1 | GGR-DS13 |

**Where the 2 blocked rows were** (*2026-09-26: 4, adding the Map context proposal's CM-5 and CM-6, blocked on its forks F1 and F2*). LOD-D7 (owner question 6, an optional
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
