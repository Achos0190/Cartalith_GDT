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

78 outstanding items — 2026-09-23, net −1: slippy-map tile addressing (XYZ/TMS/WMTS, a zoom ladder,
retina variants) closed, verified independently. A thin naming layer over the existing golden-verified
LOD pyramid (`cartalith-io::slippy` + `cartalith-engine::slippy_export`), wired into the Data manager's
Export ▸ Maps pane. `cargo test --workspace --no-fail-fast` reproduced exactly (3542→3558, the +16 being
16 new Rust tests, zero movement elsewhere). The agent's own windowed probe (40/0) was not independently
re-run — a concurrently-running batch agent held the only available Godot instance throughout
verification; full symbol-level diff review and the Rust test suite were judged sufficient. Commit
`a3bca18`.
78 outstanding items — 2026-09-23, unchanged in count (correction to an already-open row, not a close):
the LOD-tile seam-ratio investigation was re-run against current HEAD rather than trusted from its
2026-09-21 numbers, since the river rendering has changed twice more since then. Only 1 of 18 swept
configs now exceeds LOD-D2's ≤1.5 bar (was 4 of 18) — but the dominant cause was misattributed: not the
two fixes this row previously credited, but a separate, larger change (`d657091`) that retired the raster
river bake for every generated world, making those two fixes' own path dead code for this scenario. The
remaining outlier's river-contribution was tested (hiding the overlay drops its median but trades it for
two new overflows plus a 225.14 max-ratio blow-up elsewhere, exposing a pre-existing near-zero-denominator
instability in `seam_ratio`'s neighbour-mean guard) rather than assumed, and correctly left unbuilt — a
metric-definition change on a mutation-tested acceptance bar is not a unilateral fix. No code changed
(diagnostic edit reverted, confirmed clean).
78 outstanding items — 2026-09-23, unchanged in count (real scope found before scheduling, not a build):
§20's "high-precision display pipeline" row was carried as a plain medium-sized next-step; a scoping-only
pass (deliberately not a build) found the owning scope doc has already rejected building it twice on
record, since its own measurements show the defect §20 would fix is not currently present. Moved from §2.5
to §3.1 (blocked on an owner decision — whether the real goal is a currently-unmeasured precision fix or
HDR/wide-gamut output whose reachability under this port's actual `gl_compatibility` renderer is
unconfirmed), with the real blast radius (all four rendering pipelines, hardcoded 8-bit Godot/PNG/TIFF
paths, a golden hash needing re-pinning) recorded so a future build doesn't have to re-derive it.
79 outstanding items — 2026-09-23, unchanged in count (partial progress on an already-open row, batches
B-D and E5 remain): E4 of the 16K/32K export — Batch A landed, the overlay session's Rust core with no
GDScript wiring yet. `BandSink` (a writer thread over the existing banded writer), `ExportSnapshot`/
`ExportSessionCore` (a `Send+Sync`, Godot-free world snapshot and state machine so editing the world mid-
session cannot change the export), the premultiplied-alpha compositing formula confirmed against a real
readback, and five new `#[func]`s. `export_image`'s pre-render checks factored into a shared helper used by
both it and the new session-open call, behaviour-preserving. A concurrent-agent `git stash` briefly swept
this batch's uncommitted hunks together with a second, unrelated batch's; the pop restored both,
independently confirmed by symbol-level diff review rather than trusting either agent's own report.
`cargo test --workspace --no-fail-fast` reproduced 3542/0/37. Commit `34db87f`.
79 outstanding items — 2026-09-23, unchanged in count (amendment to an already-closed row, not new work):
the vector river overlay's RDP tolerance and colour, both revised on a fresh owner ruling that superseded
the row's own prior decisions. `RIVER_RDP_EPS_CELLS` 0.75 → 0.5 — the owner's first-requested `0.3` was
measured and rejected before implementing it blind (it made rivers MORE wiggly on this port's D8 lattice,
7.00°/cell → 14.63°/cell, confirmed by an overlay screenshot); `0.5` was the owner's own middle ground once
shown the measurement. Colour: the lake-matched tint (`lake_color_at`, removed) replaced with a discrete
Strahler-order palette sampled per render point. `cargo test --workspace --no-fail-fast` reproduced
3542/0/37, run twice identically. Commit `34db87f`.
79 outstanding items — 2026-09-23, unchanged in count (real fix landed, row stays open pending device
confirmation): the app no longer redraws at 60fps with nothing changing — root-caused to no low-processor
mode being set, fixed with one gated line in app.gd, desktop-measured (0 draws/s at rest, ~1% of a core
against ~8%/~46% before) with 6 existing UPDATE_ALWAYS probes confirmed unaffected. The original defect was
measured ON THE DEVICE; this fix is desktop-proxy only, so the row stays open until the same SurfaceFlinger
measurement is repeated on real hardware. Commit `1176acb`.
79 outstanding items — 2026-09-23, unchanged in count (scoping, not a build): E4 of the 16K/32K export
(the overlay session) scoped into 4 batches after real measurement surfaced genuine risks a naive port
would have hit (a hard SubViewport width limit, premultiplied-alpha readback, corner-vs-texel-centred
registration, split symbol-scale semantics). Owner ruled on the one question blocking batch B — uniform
magnification, the bigger-lift option. Five more questions gate batches B-D, not yet asked. Nothing built.
**79 outstanding items** — 2026-09-23, net −1: the landmark-consolidation row (ruling 24) closed on a fresh
owner ruling — the original 2026-09-06 ruling was made on a summary that turned out to be wrong (the
"split" it asked to consolidate was never actually landmark-only code); re-measured and put back to the
owner, who ratified the existing split as correct. Documentation-only, no code moved.
80 outstanding items — 2026-09-23, unchanged in count, one row corrected not closed: the "project archive
remainder" row's own framing ("nothing draws any of it") was stale — `drafts/` and both `library/` JSON
documents have been written and restored since early September. Re-scoped to the one genuinely open piece,
`library/`'s binary art payload (§16.5), explicitly named a design-decision gate (storage format not yet
settled) rather than a pure build task, so not scheduled to an agent.
80 outstanding items — 2026-09-23, unchanged in count (partial progress on an already-open row, E4/E5
remain): E3 of the 16K/32K export's options struct landed — one `ExportOptions` dictionary replaces
fifteen `#[func]` parameters, with a style-override layer verified structurally non-mutating (`&self`).
`cargo test --workspace --no-fail-fast` reproduced 3525/0/37. **A real owner-call surfaced, not decided
here**: the new `export_image` path now allows 16K/32K on a platform reporting no free memory (band peak
never exceeds the existing 8K monolithic figure), while `export_raster_png` still refuses there — the two
paths' refusal policy has diverged and needs a ruling. Commit `ce2c71d`.
## The count, honestly

**80 outstanding items** — 2026-09-23, unchanged in count (partial progress on an already-open row, E3/E4
remain): E2 of the 16K/32K export's banded writer landed — bands now stream to disk instead of the whole
raster living in memory, 32K PNG peaking at ~1.33 GB against ~15.2–15.3 GB monolithic, verified with an
independent decoder at every band geometry and cross-checked externally against Pillow/libtiff. Also caught
and worked around a real defect in `EXPORT_SCOPE.md` §6.1's own documented BigTIFF recipe (it writes a
corrupt file in the actual `tiff` crate). `cargo test --workspace --no-fail-fast` reproduced 3516/0/37
exactly. Commit `fc7db2b`.
**80 outstanding items** — 2026-09-23, net −1: `build_parcels`' frontage-grant retry loop closed, verified
independently. Ported the reference's own `PARCEL_GRANT_MAX_SPIN = 4096` (measured against 60 real towns,
worst unbounded run 159 367 spins against the reference's own 172 644); the previously-hanging cases
(pop 7 000 at variance 0.15, ~43s; the shipped "Planned Grid" preset's pop 4 000 at variance 0.10, >100s)
now finish in ~100ms. Golden re-baseline pre-authorized and disclosed old→new (20/29 generate cases, 22/36
district cases, both scoped to parcel-derived fields only). `cargo test --workspace --no-fail-fast`
reproduced 3510/0/36 exactly. Commit `86ccf73`.
**81 outstanding items** — 2026-09-23, net −1: the town's own garden clutter (trees, orchards, wells,
market cross, economy props) now crosses the bridge and draws, and the intramural/extramural roof tint is
fixed for real with a per-building wall-containment test instead of the district proxy that was reverted
2026-09-13. Verified independently (`cargo test -p cartalith-civ`/`-p cartalith-godot` 734/0/1 and 633/0/8;
a windowed probe re-run confirms exact roof-hue values; real before/after screenshots). Commit `362cde9`.
82 outstanding items — 2026-09-23, net −1: pack trait art released to the live map, verified
independently (`cargo test --workspace --no-fail-fast` reproduced 3507/0/36, a new windowed probe confirms
the resolver moves real pixels through the real `EngineBridge` path). Commit `7d22e96`. Also landed, no
count change (amends the already-closed wall-hugging row, `5d780dc`): the faubourg is now a wedge-shaped
cluster rather than a single line, per the owner's follow-up after seeing the committed result, measured
directly against the reference image. Commit `21c6949`.
83 outstanding items — 2026-09-22, net −1: depression-filled flow routing closed, verified
independently. A real, standard-algorithm implementation (Barnes Priority-Flood+ε — the reference's own
fix postdates every HTML snapshot in this repository, so there was no source to port line-by-line; see
`DECISIONS.md` §7o), gated off in the golden baseline and on at the app boundary, matching
`crater.physical_model`'s precedent. Measured on four real worlds, closing the reference's own
"highest-priority row" (§6g). `cargo test --workspace --no-fail-fast` reproduced 3507/0/36, zero golden
movement. Commit `76f64bc`.
**84 outstanding items** — 2026-09-22, unchanged in count, net zero: the "third culture profile" row
closed (−1), shipped as a new Rules preset (`market_town`) rather than a `CultureProfile`, since every knob
the owner's town plan needs lives on `Rules` and a new profile id would have silently lost the tiltyard.
Delivers the row's actual goal (legible blocks, ~40-50% fewer at large populations) while honestly not
reproducing the plan's radial/suburb structure. Commit `d287922`. **Found as a side effect and filed
separately (+1)**: a real, reproducible hang in shipped `build_parcels` (the "Planned Grid" preset can fail
to finish a pop-4,000 town in 90+ seconds) that corrects an earlier survey's claim this pattern didn't
exist in the Rust port — it does, one level deeper than that survey's grep looked. Not fixed (needs a
golden re-baseline), filed for scheduling.
**84 outstanding items** — 2026-09-22, unchanged in count, net effect of four moves: two owner-directed,
owner-reported features (river rendering promoted to a live smooth vector stroke with the texture bake
removed, commit `d657091`; lots backing onto the wall inside plus a poor faubourg outside, commit `5d780dc`)
were filed and closed in the same pass — real root-cause fixes with golden re-baselines disclosed, no net
count change since neither was a tracked backlog row before this pass. The font-size-sweep row closed on an
explicit owner ruling ("drop it — canvases are the standing definition of done"), net −1. The river work's
own investigation surfaced a real new open item, filed net +1: depression-filled flow routing
(`compute_flow` has none — the reference's own "highest-priority row," RC_ENGINE_CHANGES.md §6g), owner-
authorized to build but not yet scheduled to an agent. −1 (font sweep) + 1 (depression-fill) = 0.
**84 outstanding items** — 2026-09-22, net −1: Rulings 28/29's LOD-tile save option (optional, default
off, size shown at save time) closed, verified independently — real binding, a real producer reusing the
live interactive tile renderer, save-dialog checkbox with a live byte estimate, plus a real pre-existing
byte-format bug (`LOD_TILE_CHANNELS`, the length guards still assumed 1 byte/pixel against real RGB tiles)
found and fixed because it blocked the round-trip. `cargo test -p cartalith-io`/`-p cartalith-godot` and a
Godot-level probe reproduced exactly. Commit `8cade54`.
85 outstanding items — 2026-09-22, unchanged in count, one row corrected not closed: the `wantCounts`
fixed-tier-count row (§2.3) was resized small → medium/large after an agent read the real reference before
building and found a 5-field UI, a stateful quota-consumption classifier, and an iterative promote/demote
loop this port has never built at all — not a small parameter thread. Zero files touched, nothing built,
disclosed rather than a simplified invention shipped in its place.
**85 outstanding items** — 2026-09-22, net −2: two small rows closed, independently verified. (1) The
star-fort bastioned wall trace — `urban_bridge.rs` now emits `fort_trace`/`fort_ravelins`, a new
`_draw_bastioned_wall` ports the reference's branch verbatim; a live dict diff and a windowed pixel probe
with a positive control confirm it, viewed directly (plain curtain → real angular trace with ravelins).
Its "sequenced after the GUI rows" gate (owner, 2026-09-12) was explicitly lifted this session now that the
Settlement Editor GUI rebuild closed 2026-09-21. Commit `c6dc8bf`. (2) The flaky GPU determinism test —
found ALREADY FIXED a month ago (commit `803b725`, re-opened at the symbol per `MISTAKES.md` rather than
assumed stale); zero files changed, confirmed via empty `git diff` against HEAD and a negative-control test
proving the existing tolerance isn't vacuous. `cargo test --workspace --no-fail-fast` reproduced 3485/0/34
for both, independently re-run by the coordinating session.
87 outstanding items — 2026-09-22, unchanged in count: a real, owner-reported bug (the Measure tool's
"Plan a journey" button opening the planner blank instead of committing the measured path as a route) was
filed and closed in the same pass — `right_dock.gd::_on_plan_journey()` now commits the measured chain
through the same route-commit path the INFRA Route tool uses and seeds the planner with it, confirmed with
a 13/13-assertion probe re-run independently (commit `32c6c71`). No net count change — this was never a
tracked backlog row before this pass.
87 outstanding items — 2026-09-22, unchanged in count: a real, owner-reported bug ("Browse & edit a
note…" doing nothing when clicked) was filed and closed in the same pass — root-caused to an unbounded
range check in `_data()`'s `id_pressed` handler silently swallowing `ID_VAULT_BROWSE`'s id, confirmed with
a positive control on pre-fix code, fixed, and independently re-verified (commit `a7c3a84`; windowed probe
re-run personally, all four activation cases pass). No net count change — this was never a tracked
backlog row before this pass.
87 outstanding items — 2026-09-22, after two concurrently-dispatched agents landed and were
independently verified in the same pass: (1) `QuadTree`/`TiledField` retired from `cartalith-spatial`
(`DirtyTracker` kept, has real callers) — a genuine closure, net −1, commit `5c99cc9`. (2) E1 (the banded
terrain renderer) landed against the 16K/32K export row — a partial closure, no count change since the
row stays open for E2/E3/E4, commit `de3c95e`. Both independently verified: symbols confirmed present/
absent as claimed, `cargo test --workspace --no-fail-fast` re-run separately for each commit's own file
scope (164/3485/0/34 both times, zero movement between them), `git status` confirmed each commit's exact
file set. One process note disclosed in the QuadTree row: that agent briefly swept the E1 agent's
in-progress `render.rs` into a `git stash` establishing its own baseline — caught, the stash popped
cleanly, `render.rs` confirmed intact before either commit landed. Net: 88 − 1 (QuadTree) + 0 (E1,
annotation only) = 87.
88 outstanding items — 2026-09-22, unchanged in count: the Data ▸ vault menu fold (owner-reported
duplication — four vault rows down to two, plus a three-week-old landing bug actually fixed, not just
hidden) was filed and closed in the same pass — verified after the fact since the agent that built it hit
the account's weekly rate limit mid-verification and committed/pushed on its own (disclosed in the row
itself, not hidden). Independently re-verified regardless: full diff read, `cargo test --workspace
--no-fail-fast` reproduced exactly (163/3496/0/34), `_v3menu_probe.gd` re-run directly by the coordinating
session confirming the exact claimed behavior. No net count change since it was never a tracked backlog
row before this pass. Commit `7f000ef`.
88 outstanding items — 2026-09-21, unchanged in count: the vault browser (folder tree + structured
content preview, owner-requested and approved from a Design Artifact mockup the same day) was filed and
closed in the same pass — reproduced the owner's own ask precisely ("explore the filetree/vault... know
what file I need for what information"), built, and independently verified (commit `ddaa0b1`; full diff
read, all bridge call names confirmed at the symbol, `cargo test --workspace --no-fail-fast` reproduced
exactly, `godot --headless --check-only` parses clean). No net count change since it was never a tracked
backlog row before this pass. A real, disclosed gap (no file-path-keyed backlinks wrapper) is left for a
future pass, not invented around.
88 outstanding items — 2026-09-21, after Settlement Editor Batch F (the Generation rules window, the
real engine signature change) landed and was independently verified — closing the entire six-batch
Settlement Editor plan (`lazy-riding-piglet.md`). Commit `5fa0011`; the load-bearing golden-parity proof
(every one of 163 test-result lines diffed individually before/after the signature change, nothing moved
except the two new tests) re-confirmed by an independent `cargo test --workspace --no-fail-fast` run,
matching the report's own numbers exactly (163/3496/0/34). Net: 89 − 1 = 88.
89 outstanding items — 2026-09-21, unchanged in count: Settlement Editor Batch E (the sixth and last
tab, a readout over `urban_layouts()`) landed and was independently verified (commit `3a9c5cc`; the
report's key-presence claim re-confirmed directly via grep, full diff read, `cargo test --workspace
--no-fail-fast` reproduced exactly). The row stays open — Batch F (the real engine change) is the last
one remaining.
89 outstanding items — 2026-09-21, unchanged in count: Settlement Editor Batch D (the Settlement types
library — new window, new static store, new caller-owned document slot, drop-tool wiring) landed and was
independently verified (commit `933f164`; document-slot classification confirmed at the symbol, all
touched files parse clean, `cargo test --workspace --no-fail-fast` reproduced exactly). The row stays
open — Batches E/F remain.
89 outstanding items — 2026-09-21, unchanged in count: Settlement Editor Batch C (real Political
history tab, one new additive Rust function) landed and was independently verified (commit `86f3ecd`;
full diff read for all four files, `cargo test --workspace --no-fail-fast` reproduced exactly at
163/3494/0/34, +4 matching the new tests). The row stays open — Batches D/E/F remain.
89 outstanding items — 2026-09-21, unchanged in count: Settlement Editor Batch B (real Timeline tab)
landed and was independently verified (commit `29de0a5`; full diff read, six `DccWidgets` signatures
confirmed, `cargo test --workspace --no-fail-fast` reproduced the exact pre-existing baseline). The row
stays open — Batches C/D/E/F remain.
89 outstanding items — 2026-09-21, net +1: two more findings from the same OnePlus 12 device test.
(1) Filed and closed in the same pass, no net change: settlement pin rings ballooning into a screen-filling
blob at deep zoom (reported as "labels don't fade") — reproduced, root-caused (bare local-unit stroke
widths never scaled by `sc`, unlike `radius`), fixed and independently verified (commit `1050cf0`;
before/after screenshots viewed directly). (2) Filed, not closed, +1: the LOD-tile zoom "sawtooth" report —
investigated and measurably improved by the two river-rendering fixes already landed this session (seam
ratio median 1.8527 → 1.4533 on the project's own harness), independently re-measured by this session and
confirmed genuinely better but not uniformly under LOD-D2's own `≤1.5` bar across every seed/config (several
still exceed it) — left open as its own row since the underlying metric isn't fully closed even though the
visual complaint very likely is. Net: 88 + 1 (new, open) + 0 (filed-and-closed) = 89.
88 outstanding items — 2026-09-21, unchanged in count: a real-device regression (rivers rendering
red-brown on LOD tiles, reported from a live OnePlus 12 APK test) was filed and closed in the same pass —
reproduced, root-caused (`channel_tint`'s `[0,1]`-vs-`[0,255]` unit mismatch at one LOD-D1 call site),
fixed, and independently verified (commit `42754bc`; before/after screenshots viewed directly, full
workspace test suite reproduced at the exact pre-existing baseline). No net count change since it was
never a tracked backlog row before this pass. Two more owner-reported regressions from the same device
test (a LOD-tile-zoom sawtooth/seam effect, and labels not fading at max zoom obscuring settlements) are
under active investigation, not yet closed.
88 outstanding items — 2026-09-21, bookkeeping only, no work completed: the two Settlement Editor
rows (desktop + touch) merged into one after resolving with the owner that touch is one `.gd` file with
responsive branches, not a separate build — the same total scope, now tracked as one row instead of two.
Batch A of that row's six-batch plan (`lazy-riding-piglet.md`) landed and was independently verified in
the same pass (commit `76117f4`; full diff read, parse-check clean, `git status` confirms scope, no engine
call changed) — the row stays open, five batches remain. Net: 89 − 1 (merge) = 88.
89 outstanding items — 2026-09-21, after §6l (river rasterization breaks on a diagonal receiver
step) landed and was independently verified (commit `f8d8bcd`; full diff read, `cargo test --workspace
--no-fail-fast` reproduced exactly at 163/3490/0/34, `git status` confirms only `cartalith-hydrology/
src/lib.rs` changed, no golden-parity value moved). Confirmed real by direct code read before dispatching
the fix, not assumed from the §2.9 survey. Net: 90 − 1 = 89.
90 outstanding items — 2026-09-21, unchanged in count: §2.9's "establish which of the specified
changes are already ported" survey completed (7 parallel read-only passes, no files modified) — of ~31
core-simulation items plus surveyable §8.1/§8.2 rows, 12 PORTED, 8 PARTIAL, 26 NOT PORTED, 7 UNCLEAR. Two
of the highest-stakes findings spot-checked directly by this session and confirmed at the symbol
(`enforce_channel_descent` genuinely live and contradicting the spec's own "should not implement"
instruction; `compute_flow` genuinely lacks priority-flood depression-fill routing, the spec's own
"most consequential" finding). One sub-claim (`PLATE_BASE_BLUR_K` hardcoded at 0.35) could not be
confirmed at the symbol and is flagged, not trusted or silently struck. A survey generates no closures
by itself — this unblocks scheduling §2.9's real items, not a batch in itself.
90 outstanding items — 2026-09-21, unchanged in count: the `preview.png` producer half of the
"Project archive remainder" row landed and was independently verified (commit `cf1eeec`; full diff read,
`cargo test --workspace --no-fail-fast` reproduced exactly at 163/3488/0/34, both touched `.gd` files
parse clean, the windowed capture probe's own review PNG viewed directly showing a real rendered map
frame). The row itself stays open — project-layer panels, the `library/`/`drafts/` slots and foreign-entry
preservation are still unbuilt — only its `preview.png` sub-item closed, annotated in place rather than
counted as a separate closure.
90 outstanding items — 2026-09-21, after the pipeline stage-row label alignment fix landed and was
independently verified (commit `c75cf5c`; diff read in full, before/after screenshots viewed directly
confirming the wrapped state label no longer floats above its row's number/name columns, `git status`
confirmed only the one tracked file changed). Net: 91 − 1 = 90.
91 outstanding items — 2026-09-21, after world-wrap support for the GPU warp/heterogeneity kernels
landed and was independently verified (commit `023c904`; full diff read against every claim in the
report, `cargo test --workspace --no-fail-fast` re-run matching exactly at 163/3487/0/34, no golden-parity
file touched, `project.godot`/every `.gd` file confirmed unchanged). The report corrected its own brief's
premise along the way: the CPU-shape GPU twins were never "already correct" for `world=true` — the real
CPU reference calls a genuinely different periodic noise algorithm (`pfbm`/`pvnoise`), which this batch
built as `gpu_pvnoise`/`gpu_pfbm`. Net: 92 − 1 = 91.
92 outstanding items — 2026-09-21, after Ruling Y (GPU device reuse with real loss handling, measured
on this session's own hardware) landed and was independently verified (commit `b6f2816`; both device-loss
tests re-run on the same real GPU). The Markdown Vault reader (owner-requested the same day) also landed
and was independently verified (commit `7749763`) — filed and closed in the same pass, no net count
change from filing it. Net: 93 − 1 = 92.
93 outstanding items — 2026-09-21, after Ruling Z (the phone MAP tab detent is content-sized, not
flat 0.46) landed and was independently verified (commit `b1a8dd5`; `_detent_probe.gd --force-touch`
reproduced PASS, 0 failures). Two agents (vault reader, GPU device reuse) still running. Net: 94 − 1 = 93.
94 outstanding items — 2026-09-21, after the vector river overlay landed and was independently
verified (commits `750fe79` + `347db6e`; full workspace test count unchanged at 3476/0/33, confirming
the GDScript-only closing batch touched no Rust test surface). All three 2026-09-13 reversion reasons
fixed this time, with real screenshot evidence for each. Net: 95 − 1 = 94.
95 outstanding items — 2026-09-21, after SP-1 (the Journey entity, `STORY_PLANNING_SCOPE.md`'s
keystone milestone) landed and was independently verified (commit `750fe79`; 163/3476/0/33 reproduced).
The vector river overlay agent is still running. Net: 96 − 1 = 95.
96 outstanding items — 2026-09-21, after three more owner rulings (X, Y, Z): Ruling X closes the
Ruling-29-scope ambiguity outright (the export-menu tiles row stays, no build owed) — net −1. Ruling Y
(build GPU device reuse, with explicit device-loss handling) and Ruling Z (shrink the phone MAP tab's
half-open detent to fit its real content) are ruled and queued for the next batch, not yet built — no
count change from the ruling itself. Net: 97 − 1 = 96.
97 outstanding items — 2026-09-21, after Ruling W (pack-import warning names all four undrawn
sections) landed and was independently verified (commit `ca77aea`; 163/3469/0/33 reproduced). Two of the
three next batch's agents (vector river overlay, SP-1 Journey entity) still running. Net: 98 − 1 = 97.
98 outstanding items — 2026-09-21, after Ruling V (GeoJSON import creates an unknown faction, both
the Rust apply logic and the GDScript Data-manager route) landed and was independently verified (commit
`d79d776`; 163/3467/0/33 reproduced, all 5 load-bearing tests confirmed present and passing). This closes
the Rulings T/U/V batch — Ruling W (pack-import warning naming) is next, queued since it needs a golden
re-baseline a test currently asserts the opposite of. Net: 99 − 1 = 98.
99 outstanding items — 2026-09-21, after Ruling T (wrapped worlds keep size-primary water-body
classification) landed and was independently verified (commit `e99c6ac`; 163/3467/0/33 reproduced).
Net: 100 − 1 = 99.
100 outstanding items — 2026-09-21, after Ruling U (Refine detail moves to Preferences ▸ Tiles & LOD)
landed and was independently verified (commit `6d1c639`). Net: 101 − 1 = 100.
101 outstanding items — 2026-09-21, after v2.72 (raster river fixes) and v2.73 half 1 (village-green
plaza kind) both landed and were independently verified together (commits `fcaafea`, `68e5656`; full
workspace 163/3450/0/33 reproduced after both landed). v2.72's fix 2 (antimeridian wrap) was found
genuinely inapplicable to this port's disc-stamp rendering technique — proven with a real oracle test,
not assumed — so only fix 1 needed landing. The pre-existing "vector river overlay" row was updated in
place (not counted as new) to note it's now the direct route to the owner's "flowing, smooth" river bar;
a duplicate copy of that same row, filed twice by mistake earlier the same day, was merged back into
one. Net: 103 − 2 (closed) = 101.
101 outstanding items — 2026-09-21, after fixing `landmark.rs`'s stale doc-comment counts directly
(commit `c7710c5`, doc comments only). Net: 104 − 1 = 103.
104 outstanding items — 2026-09-21, after border marker landmarks landed and were independently
verified (commit `92cd8d0`): `LandmarkInputs::territory` wired from `assign_territory`'s real per-cell
faction raster, hard-gated on the existing viewshed primitive, a dedicated negative test proving the
claimed/unclaimed edge does not count as a border. Buildable 25 → 26, blocked 24 → 23 (M8 residual row
stays open — `sacred_mountain` and others remain). One new small row filed for a stale doc-comment count
the agent found in passing but correctly left alone (out of scope). Net: 103 + 1 = 104.
103 outstanding items — 2026-09-21, unchanged in count: the pipeline stage 09/10 wording fix landed
and was independently verified (commit `84d016c`), closing that row; a genuine pre-existing layout
quirk (a wrapped state label drawing above its own row's number/name columns) was found in passing
while verifying it and filed as its own small row rather than silently noted. Net: 103 − 1 + 1 = 103.
103 outstanding items — 2026-09-21, after verifying the Android adaptive icon fix directly on the
6T (device `9608b26b`, still connected): `adb shell screencap` on the home screen shows a real
compass-rose emblem with contrast, not the blank grey plate the owner originally reported. Fixed
2026-09-03, never confirmed on device until now. Net: 104 − 1 = 103.
104 outstanding items — 2026-09-21, unchanged in count: the owner supplied the real
`Cartalith_v2.71_DCC_test.html` and the §6b.2/§9 orogeny contradiction resolved against it (§6b.2 was
right, both of §9's orogeny bullets were false — `RC_ENGINE_CHANGES.md` §9 corrected in place). The row
stays open (the underlying multi-ridge belt is still unscheduled, large) — only the "maybe never
shipped" blocker is removed, so no count change.
104 outstanding items — 2026-09-21, after the hydrology topology reclassification (Ruling Q) landed
and was independently verified (commit `cc834d2`): `build_water_bodies` moves from size-primary to
boundary-contact-primary ocean/lake selection, 13 golden files re-baselined with every value disclosed
old → new, full workspace test count reproduced at 163/3440/0/33. One new small item filed alongside it,
not silently folded in: the wrapped-world (`world=true`) case was a real judgment call the research
doesn't dictate, confirmed to move real output, and needs its own owner ruling if it matters. Net:
104 − 1 (closed) + 1 (new, flagged) = 104.
104 outstanding items — 2026-09-21, after zoom-dependent label density landed and was independently
verified (commit `d213a41`): extends the existing `SETTLEMENT_LOD` pattern to the generic label layer,
found and fixed a real settlement-label duplication bug along the way, no golden moved. Net: 105 − 1 = 104.
105 outstanding items — 2026-09-21, after the river-ink-over-water fix landed and was independently
verified (commit `b04a41b`, root cause confirmed against the real v2.11 reference: this port composited
ink onto a resolved pixel colour with no memory of whether it was land or water, a straight bug with no
golden to move) and the owner declined v2.71 half 2 (woodland as a spatial area — Ruling S, *"I'm not
satisfied with it in the HTML version"* — moved to §5, no longer counted). Net: 107 − 1 (closed) − 1
(declined, moved out of the count) = 105.
107 outstanding items — 2026-09-21, after CA-19 (Ruling P, the writable biome colour table) landed
and was independently verified, commit `cc0f561`: `TerrainAppearance::biome_cols` plus a `WorldGen`
override table, `swatch_color` kept pinned by delegating to a new `swatch_color_with` sibling so no
golden actually moved. `cartalith-godot` re-run in isolation: 874 passed / 0 failed. Net: 108 − 1 = 107.
108 outstanding items — 2026-09-21, after a PC screenshot review (owner request) surfaced three new
findings, all filed at their symbol rather than assumed: river ink painting onto lake/ocean water
unmasked (`stamp_river_intensity`/`channel_tint`, queued behind CA-19's hold on `render.rs`), pipeline
stage rows 09/10 reading as "didn't run" when biome/resource generation genuinely runs every call (a
wording bug, not an engine gap — `progress.rs`'s own doc says so), and zoom-dependent label density (the
settlement-pin `SETTLEMENT_LOD` pattern already exists and already cites OSM Carto/MapLibre precedent,
but the generic label layer never got it — an agent is building the extension now). Net: 105 + 3 = 108.
105 outstanding items — 2026-09-21, after landing v2.69's sea-level clamp fix (Ruling O), verified
and committed in `c37488a`: `clamp_toward_sea` added at all 4 sites in `amplify.rs`, golden hashes
re-derived across 4 test files in 2 crates, full workspace test count reproduced independently at
163/3435/0/33, one mutation-test spot-check run and cleanly reverted. No new row opened; the row closes
outright since Ruling O already authorized the fix and the fix matched the ruling exactly. Net:
106 − 1 = 105. 106 outstanding items — 2026-09-21, after two real 6T device passes closed both remaining rows: the
zoom-clamp row (confirmed not a bug, aimed at real terrain per its own stated next step — badge and
pixels stop together at the cap, the opposite of the filed defect) and the boot-time-variance row
(the slow ~15.4 s boot is real and reproduces every time, but the original 6–15.7 s VARIANCE does not —
15 cold starts landed in a tight band; a real, unattributed 9.4 s main-thread stall was found and filed
as its own row). Net: 107 − 2 (closed) + 1 (filed) = 106. 107 immediately before, after landing labels' fixed-size default and a real font-family
picker (commit `f826932`) and filing one new row for a load-bearing finding made along the way: labels
carry no `civ_zoom_k()`-style term at all, so `size_mode: "fixed"` does not actually counteract live
camera zoom — confirmed at the symbol and independently reproduced, `_labelblur_probe.gd`'s own
pre-existing assertion already said as much. Net: 106 + 1 = 107. 106 immediately before, unchanged in
count: four owner rulings recorded (`LARGE_ITEM_RULINGS.md`
O/P/Q/R) unblocking v2.69's sea-level clamp, CA-19's writable biome colour table, the full hydrology
topology reclassification, and IN-13's per-faction-currency model — no row opened or closed by the
rulings themselves, only unblocked; each row's own text updated to point at its ruling. 106 immediately
before, unchanged in count: the zoom-notch device pass on the 6T closed
the row this whole session's LOD-D6 work was waiting on (frozen frame gone, ~13× reduction, 66.77 ms
vs. an 868-885 ms baseline, independently cross-checked against the agent's raw SurfaceFlinger log) and
filed one new row for a genuinely different defect the same pass found (real but smaller stutter on
later/land-centred notches, not a freeze). Net: 106 − 1 + 1 = 106. 106 immediately before, owner
instruction: two new GUI rows filed, `Cartalith Settlement
Editor.dc.html` and its touch variant (a faction/settlement editor for urban morphology, owner-supplied
design canvases in project `067f80e7-dbb7-4492-8e69-96aaa8050a4d`), both blocked on `/design-login`
until the design MCP is authorized. Net: 104 + 2 = 106. Also: the 6T reconnected mid-session: a fresh
release build reflecting every change landed today (LOD-D6, Ruling L, the WorldState-Arc chain) is
built, signed and installed on-device; the zoom-notch row (device-pass-gated all session) marked in
flight, no count change yet pending its real measurement. 104 immediately before, after closing the round-3 GUI surfaces row's last open piece
(rail subtitles, commit `837cbfb`) — unblocked by Ruling L's completion, re-verified genuinely
unaffected by it, with one unrelated stale probe literal fixed along the way. 105 immediately before,
after closing `LithoSource` and `SaveFields`'s Arc follow-ons
(commit `e376cda`), the last two rows from the `WorldState`-Arc memory-optimization chain LOD-D6
kicked off. `LithoSource` turned out to be four fields, not three — the earlier row's count traced to
a miswritten commit message, corrected at the symbol before closing. Both independently re-verified:
combined tree builds clean, `cargo test --workspace` reproduces 163/3432/0/33 exactly. No new rows
filed this batch — both follow-ons were fully scoped by the prior batch's own flags. Net: 107 − 2 = 105.
107 immediately before, after a 3-agent batch closed three small/medium rows and
filed two: the `_v3menu_probe.gd` Data-menu lookup (fixed with a structural `MenuButton`-text match,
replacing a submenu-row text match that had already broken once) and `_railfold_probe.gd`'s
Labels/Icons section (a stale probe, not a bug — real functionality shipped under the owner's
2026-09-02/09-03 rulings three weeks before anyone re-checked the assertions) both independently
re-verified to a full `PASS`; and `WorldState`'s four `Vec<f32>` LOD-snapshot fields moved to
`Arc<Vec<f32>>`, dropping a 43 MB per-snapshot clone to a refcount bump, blast radius smaller than
feared (5 of 6 reading crates needed zero edits), golden tests byte-identical, `cargo test --workspace`
independently reproduced exactly (163/3431/0/33) after clearing a stale `.dll` file lock. Filed two
follow-ons the closing agent flagged, same shape, not yet built: `LithoSource`'s three deep-cloned
fields (another ~43 MB), and a loaded save's `SaveFields` copies. Net: 108 − 3 (closed) + 2 (filed) =
107. 108 immediately before, after closing `LARGE_ITEM_RULINGS.md`'s Ruling L in full
(commit `04b3b27`; the WORLD and CARTO halves, completing the CIVIL half closed 2026-09-20) and
filing two small pre-existing probe defects found along the way, unrelated to the rail re-sort
itself: `_railfold_probe.gd`'s Labels/Icons placement-rule controls asserted inert when they are now
live, and `_v3menu_probe.gd`'s Data-menu-popup lookup keying off text that moved to a rail node under
Ruling L's own earlier CIVIL half. Net: 107 − 1 (Ruling L closed) + 2 (follow-ups filed) = 108. 107
immediately before, unchanged in count through a 3-agent batch that filed and
closed the same three rows: CV-23 (found already built — a fourth false alarm, this one on the
coordinating session's own pre-dispatch grep, corrected and folded into `MISTAKES.md`), Landmark M7's
viewshed (real new engine work, unblocking 5 of 7 gated landmark kinds with a disclosed conservative
sizing default rather than an owner number waited on), and a landmark-result staleness UI indicator
(real new work, deriving its visual vocabulary from the shell's own existing pattern). The last two
shared `landmark.rs` between two concurrently-running agents and briefly left `HEAD` unbuildable
between two commits landing the split — caught and fixed within the same batch, `MISTAKES.md` updated
with the process lesson. 107 immediately before, unchanged in count through a batch that filed two rows and
closed the same two: dispatching agents on `UNWIRED_FUNCTIONS.md`'s two apparently-real remaining
gaps (paint-preview shell wiring, the Cut/Copy/Paste clipboard) found both already shipped days
earlier (`45df3019` 2026-09-04, `686cd2a` 2026-09-03) — both independently re-verified before either
row was closed, so no duplicate work landed. A third audit, of `LANDMARK_GENERATION_SCOPE.md`'s 29
blocked landmark kinds for the identical defect shape, found none stale (a genuine negative result,
`cargo test` unchanged at 3417/0) and corrected a grep-artifact count (50/30 → the real 49/29) inside
that row without opening or closing it. 107 immediately before, after a 3-agent batch closed four small LOD-D6-follow-up rows
and filed one new one: `_loddbg_probe.gd`'s stale row count (`7a26d46`, verified two ways — headless
partial run and a full windowed `PASS`); the `lod_cache_key`/`glacial_snowline` mutation-testing gap,
closed with a new probe that killed the named mutant for real (`f0a1166`, independently re-run to the
same pivot cell and diff figures) and which surfaced one new small over-invalidation row, filed not
fixed; and both LOD tile-cache-staleness rows (`GUI_GAP_REGISTER.md`'s and this file's own duplicate),
closed together by invalidating the tile pyramid at all 11 sculpt/paint/undo/redo/revert commit sites
that wrote `map_view.texture` directly (`92d1edc`, independently re-run to the same 15-section `PASS`).
Net: 110 − 4 (closed) + 1 (over-invalidation row filed) = 107. 110 immediately before, after closing
LOD-D6 (`c74a150`; off-main-thread tile synthesis, desktop bars met — 199.50ms → 0.98ms at LOD entry,
memory bounded by assertion — phone bars honestly unmeasured, no device reachable) and filing two
small follow-ups it flagged: `Arc`-ing `WorldState`'s four `Vec<f32>` fields (the real further memory
lever, deliberately not taken inside D6) and `_loddbg_probe.gd`'s stale row-count assertion
(pre-existing, unrelated to D6). 109 immediately before that, after closing LOD-D5 (`c685930`;
scale-aware shading and hydrology, 2 of 3 bars met, 1 needing an owner ruling on `add_zoom_detail`'s
octave decay — see its row). 110 immediately before that, after closing the seven uncalled `cartalith-gpu` public
functions row (`46aff27`; six deleted, one already gone, net −75 lines). 111 immediately before,
after closing LOD-D4 (`02f6d51`; ice and snow from existing
fields, 1 of 4 bars met, one explicitly needing an owner ruling to fix, one barely measurable at this
zoom) and filing one new small follow-up (`lod_cache_key`'s untestable mutant survivor — net zero
change). 111 immediately before, after closing three rows found stale on re-verification: the
tablet CheckBox-growth row (its blocking probe limitation was already fixed 2026-09-07 and never
re-checked; two agents independently confirmed it live this session), VA-01 (a real backlink index
already exists), and `GUI_GAP_REGISTER.md` §11's CIVIL/INFRA `timeline_bar` bullet (that strip is now
fully built). No code changed for any of the three. 113 immediately before, after closing LOD-D3 (`b6cc014`; engine-side CDLOD morph and
parent-level fallback, 2 of 4 D0 acceptance bars met and 2 disclosed not met, including that D2's
open seam-ratio item is explicitly NOT closed here — see its row). 114 immediately before, after
closing v2.70 (`7adb600`; the "Village map" flat-palette
style, one flag through the existing NPR chain — one constant disclosed as this port's own choice,
not the verified reference value, since `Cartalith_RC` wasn't reachable). 115 immediately before,
after closing three rows in one pass: the 28-vs-20 `_um*`
function-count contradiction (`58d9003`, re-derived by grepping the frozen reference directly: 27
functions, never actually in tension), and two PROVENANCE.md rows found already satisfied on
re-verification (Nortantis credits disclosure live since `fb9c5b8`; the upstream-notes row already
met by its own stated fallback paragraph). 118 immediately before, after closing LOD-D2 (`9d2a800`; real RGB tiles reach the
screen through a real cache, 3 of 6 D0 acceptance bars pass and 3 disclosed failing rather than
claimed — see its row). 119 immediately before, after closing LOD-D1 (`f6d1bd5`; byte-identical golden parity
against the real reference, two owner decisions surfaced but not blocking — see its row) and the
v2.71 water-clip half (`0bf3152`; a net-zero row split, the woodland half stays open separately). The
Culture profiles window (`5f111a7`, closing `GUI_GAP_REGISTER.md` CV-02) closed a gap tracked only in
that register, not counted here. 120 immediately before this pass, after closing LOD-D0 (`11936cb`;
baseline recorded over 6 worlds, and its findings change how D2/D3 should be graded — see its row).
121 immediately before
that, after closing the Recent-worlds name row (`5e25c40`; both
halves now — the "ELDRA implies a generator already exists" premise was checked and found false
before anything was built). 122 immediately before that, after filing six new §2.9 rows for the
newly-pulled-in RC_ENGINE_CHANGES.md content (v2.69, v2.70, v2.71, v2.72 [held, river rendering],
v2.73, and the orogeny/"Himalaya problem" spec contradiction that needs resolving at the source
before it can even be scheduled) — 116 immediately before that. That was 2026-09-20, after closing the PLAN header
stale-on-isolate row (`f493b61`); 117 before that, after closing EF-3 (`794802d`; erosion-consistent
tile detail, verified with a conditional/honest result — see its row); 118 before that. Previously 120 in one run of
the counting script, after `main` was merged into this branch. **The two lines of work had diverged since 2026-09-01** and each held rows the other did not: this side’s 115, plus §2.9’s three source-engine rows, the re-opened reference re-freeze (measured twelve versions behind on 2026-09-17, not one) and the Nortantis credits row. Where both sides edited a row, the newer measurement won: the menu-command row stays closed here, because this side re-cut it with a probe on 2026-09-13.

**The headline read 100 until today, from 2026-09-06**, after sweeping 27
closed rows out of the numbered sections in two passes. **It had gone stale by
18 while the rows beneath it were kept current** — 118 live when found; the
same day filed one unrowed defect, then the seven rows owner Rulings H-K opened , two from that evening’s batch and two from a left-rail inventory — less one row found already built that evening, plus one pre-existing bug a verifier found; on 2026-09-13 the owner’s left-rail re-sort opened one row and absorbed two, and the first three-builder batch closed three and filed two; the second closed three; the third closed two and filed two; a concurrent menus run filed one; the fourth closed five and filed three; the last session’s parallel small and medium runs closed thirteen and filed one; a seven-lane run closed three and filed one; `LOD_DETAIL_SCOPE.md` filed LOD-D0 to D6 as seven rows, absorbing two. The parity audit and the first on-glass pass opened
rows faster than batches closed them. **Read a rise as more known, not more
broken**, and re-derive this figure rather than trusting it: a headline a file
states about itself is a claim, and this was the stalest line in the file.

**The headline had been climbing while work was being finished, and that was a
bookkeeping fault rather than a real one.** A closure was filed as a struck row
and left in place, and the counter counts *rows in numbered sections* — so
closing four items and opening none still moved the number up. It read 114
with **87** genuinely open.

**It took two passes to find them all**, and the second is the instructive
one: the first swept rows struck in their *title*, and four more were closed
only in their **notes** cell (`~~Open~~ **CLOSED …**`) — including a feature
that had shipped days earlier and was still being counted as work. The closed rows now live under **Closed rows, kept for the
evidence in them**, which is deliberately unnumbered so the counter skips it.

**They were swept, not deleted.** Each carries a measurement, a refuted premise
or a reason something was declined — several were re-opened during this session
and found stale before being closed properly, which is exactly what those
paragraphs are for.

The earlier discipline still stands: this total comes from **one run of the
counting script**, after a pass once left four different totals in this file at
once (a headline of 142, a table summing to 143, a report claiming 145). That is
§6.8's own "counts that disagree with themselves"; the arithmetic here is not
safe to delegate.

The figure is `§1 + §2 + §3 + §4`, with §5's declined entries deliberately
outside it.

**2026-09-05, thirty-second batch — 100 → 98.** The phone MORE tab becomes
§6.6's five bespoke screens, the browse dialog stops overflowing itself, and both
left-dock follow-ups close. **16 of 17 verdicts confirmed.** `cargo test
--workspace` holds at **3 142** — correct for GDScript-only lanes.

*The phone rewrite's real deliverable was the reachability list, and it is
empty.* Five bespoke screens cannot carry every row seven desktop popups did —
so the lane dumped the ground truth first (7 menus, every non-separator item,
recursively, from the live `MenuBar`), then rendered each screen and read the
drawn strings back. **All 7 menus reachable; every top-level row of File, Data
and Preferences drawn; `missing:` empty.** Only routes changed. Edit and Window
have no row in §6.6's table at all and went to an appended *Not on the MORE
list* band rather than being dropped.

*And the screens act on the real shell rather than copying it.* Each row resolves
to a real `PopupMenu` item by menu name and item id, firing through the existing
`_activate()` — no handler or menu id reimplemented — so a row added to
`menus.gd` appears on the phone with no edit here. Verified live: the Year
slider moves the real cursor, a Speed chip writes the real setting, a Units chip
moves `DccSettings.units_mode()` km→mi, all four CIVIL tools arm.

*Where the port has no engine, the absence is drawn rather than invented.* The
POI row carries `civilization_workspace.gd`'s own reason and arms nothing; §6.6's
CONVERSION band is not built, because the owner removed `Data ▸ Conversion` on
2026-08-20 and the canvas is the stale party there.

*Twelfth consecutive batch with a false clause in prose written the same pass* —
this one a probe's own usage header documenting `--resolution`, a flag it never
reads (it reads `-- --vp WxH`). Following it would have measured **the same box
three times and called it three densities**.

**2026-09-05, thirty-first batch — 99 → 100.** The left dock is restructured to
`04-left-dock.md` §3, the New world phone card is built, and the cold-start
screen is the canvas's centred column. Two findings filed, so the count rises.
`cargo test --workspace` holds at **3 142** — correct for GDScript-only lanes.

*The consequence the owner accepted was bought for almost nothing, and that is
now measured.* The ruling was accepted against my summary "one body per rail
node, gated by mode", which was wrong. §3 gates **one node of ten**. Measured
after the build: WORLD·a renders 9 of 9, WORLD·b renders 1; **CIVIL renders 15
in all four modes, CARTO 10 in all four**. The only thing that leaves the default
view is eight WORLD categories, and only while Sculpt is armed — each with
**four routes back**, all exercised. **34 categories render before and after; 8
of them in one fewer mode; 0 controls without a route.**

*A category that no rail node owned.* `mode_for_category("civilization",
"Religion")` returned `""`, so opening Religion left the rail lighting whichever
node was last active — you could reach it and the rail lied about where you
were. Now owned by the `factions` node.

*Lane B declined the NAME field, and was right to.* The brief said to establish
what a name would **do** first; there is nowhere to put one, so a control that
collects a string nothing stores was not added.

*Eleventh consecutive batch with a false clause in prose written the same pass* —
three this time, all corrected: a mode-switch claimed "derived, not hardcoded"
when only its visibility is; a `open_welcome()` doc still describing the gallery
the same commit replaced; and a tooltip pointing at a menu row (`World ▸ Generate
▸ New seed`) that does not exist in `menus.gd` or the command index.

**2026-09-05, thirtieth batch — 98 → 99.** The owner's three structural moves
ship, eight of the 37 no-design surfaces close, and the vault snapshot panel
tells the three states apart. Three findings filed, so the count rises. `cargo
test --workspace` holds at **3 142**, 0 failed — correct for GDScript-only lanes.

*The moves did the regression the brief warned them not to, and the partitioning
is why.* Moving `Journey planner` to the CIVIL rail and `Refine detail` to the
WORLD bar took both **out of the searchable command index** — a verifier measured
**0 title matches out of 361 rows** for each — and left `⇧J` in neither
`Help ▸ Keyboard shortcuts` nor `UNLISTED`, so it became unrebindable and
undiscoverable. `_add_menu_commands()` walks the live `MenuBar`, so a command
that stops being a menu row stops being searchable **silently, with nothing
failing**. The lanes disclosed it and could not fix it: `command_index.gd` and
`shortcuts_dialog.gd` were reserved from every lane by the main loop. Fixed here,
and `EXTRAS` now carries the rule in its own doc — **any future move off the menu
bar owes this table a row in the same change**.

*A defect found on the way, reachable before this pass and unrelated to it.*
Switching CIVIL rail nodes with the planner up re-showed the civ dock **underneath
a still-visible planner** — two left docks at once — because `_hide()` is what
restores the workspace panel and only a domain change was calling it.

*The binding consequence held.* `Clear library… destructive` survived the
flattening with a home and a stated reason, as the ruling required.

**2026-09-05, twenty-ninth batch — 100 → 98**, after the owner's five rulings
took the count from 96 to 100 by turning one held row into five real ones.
`cargo test --workspace` **3 135 → 3 142**, 0 failed.

*The pass sent to retire single-sample timings wrote another one.* Its own new
doc claimed the GPU height kernel wins **1.06× (1.00..1.08×) at 2048², saving
2.2 ms**. A verifier re-ran the same test serially three times, medians of five
each: **1.00× (0.98..1.04), 1.00× (0.97..1.03), 1.02× (0.96..1.03)** — the
claimed median outside all three brackets, and the "saving" spanning +0.10 to
−0.17 ms, i.e. changing sign. **Withdrawn**: at 2048² no difference is
established. A median of five on a noisy device is still one sample of the
median, and eight more bare point estimates were caught in the same prose.

*Two guards that close a shape this project keeps meeting.* A deleted snapshot
PNG still had its Map field offered and wrote `![](…)` into the user's note
pointing at nothing — now filtered, with **three states kept apart** (never
generated / present-or-unknowable / deleted), the last flagged by a key that is
**omitted rather than `false`**, because no value means unknown. And the
`vault.json` write gate finally has a test **at its call site**: reverting to the
pre-`52666b9` `!store.links.is_empty()` turns the probe red on **snapshots AND
vaults** — the superseded gate lost a vaults-only store too, wider than the row
said.

*Tenth consecutive batch with a false clause in prose written the same pass.*
Including, in the timing harness's own module doc, "None was a measurement" —
the wrong lesson: every one of those figures **was** a measurement, of a machine
under contention or of a single sample, quoted as a property of the code.

**2026-09-05, twenty-eighth batch — 97 → 96.** Vault milestone 2 closes as
already done; both right-dock refresh gaps close; one §20 gap filed. `cargo test
--workspace` **3 133 → 3 135**, 0 failed.

*Sixth false-premise row in nine batches, and guarding it was still worth it.*
Milestone 2 shipped in `4ec07f5`, four commits before HEAD, and the one defect it
left — the save gate — was fixed in `52666b9` two commits later. Both ancestors of
HEAD. But **§21's three radii were pinned by nothing**: with the new test skipped,
`10→12`, `50→60`, `250→200` and immediate/regional **swapped** all SURVIVED the
suite. A snapshot in every note could have silently covered a different area with
the tests green. Now an `assert_eq!` against the literal table plus a
strictly-increasing check, so "Immediate" can never crop wider than "Regional".

*A backward-compatibility test that proved nothing.* The whole legacy check was
two empty arrays. Replaced with a real pre-snapshot `vault.json` — one vault, one
heading link with imported text, no `snapshots` member — asserting it opens,
resolves, reports `Connected`, returns `snapshot() == None` (**absent, not an
empty path**), and re-serialises byte-identically, so no old project gains a
spurious `"snapshots": {}` on first open.

*Lane B found a third bug on its own.* `sculpt_draft_changed` was emitted
**before** the engine call, so a synchronous listener re-read the count from
before the change — measured at 0 stamps on the stroke that created the draft's
first.

*Ninth consecutive batch with a false clause in prose written the same pass*,
three this time: "three references" that are five, "both listeners" when one is
`CONNECT_DEFERRED` and never sees the emit, and an emit count of nine against ten
(that last only in the report, never shipped). The two in the tree are corrected.

**2026-09-05, the menu design-conformance audit and its first fixes — 96 → 97.**
Owner instruction: the GUI verification runs **before** the rest of the list.
Four Fable 5.1 auditors at Ultracode plus an adversarial cross-check, then six
fixes on Opus 5. `cargo test --workspace` holds at **3 133**, 0 failed — correct
for GDScript-only work.

**283 menu items, enumerated from the code rather than from the designs** —
125 conforms, 99 deviates, 37 no-design, 17 design-stale, 5 unreachable. The
enumeration direction was the point: walking the designs and ticking items off
can only find what the designs already list, and 37 surfaces have no design at
all.

*The cross-check earned its place, and its verdict was `partly-unsound`.* Seven
of 42 verdicts refuted, two materially: tool shortcuts V/M/R/B/F/L/I were
reported **unreachable** when `dcc_widgets.gd:948` binds them — a fix lane would
have worked against a defect that does not exist — and a phone-tab count of five
was wrong where two other auditors had four.

*Six fixes shipped, all confirmed.* Autosave defaulted **off** and to **10 min**,
a value **not on its own ladder**, so a fresh install opened the interval submenu
with every radio row unchecked; the settlement panel printed a faction **number**
where the design says the owning **name**; the paint row had Commit and no
Discard; the collapsed left dock showed WORLD's word in every domain; the
collapsed timeline strip measured **17 px** against a 24/34 spec. Migration was
decided deliberately — only the **absent** key moves, an explicit choice is never
rewritten.

*A refutation that prevented damage.* The lane proposed removing
`ROLE["h_timeline"]` and `H_TIMELINE` as dead; `_roleresolve_probe.gd`, a
committed file, reads both. It raised this rather than acting, and the verifier
confirmed the constants are live.

*Eighth consecutive batch with a false clause in prose written the same pass* —
a doc naming `_refresh_stage_rows()`, which exists nowhere (the caller is
`_paint_stage_rows()`), and a dash reason saying "generate a world first"
rendered on a fully generated world holding six factions. Both corrected.

**2026-09-04, twenty-seventh batch — 96 → 95.** Pack trait art is built end to
end and parked behind one line; the group-header row closes on a measurement.
`cargo test --workspace` **3 129 → 3 133**, 0 failed. Every verdict confirmed.

*The row that was filed as app-wide turned out to be two headers.* The batch that
found the `group()` header problem filed it with its blast radius unmeasured. It
is **2 headers over their floor out of 59 distinct headers across 141 surface
states** — and the change was taken anyway only because reverting all 57 live
headers to `AUTOWRAP_OFF` and diffing the framebuffer measured **0 px** of
difference, i.e. the fix costs nothing rather than being merely worth it. The
verifier widened the relationship check past the lane's own: **544 assertions
over 55 headers, 0 failures**, with headers identified structurally rather than
by label.

*The trait-art work is finished and deliberately not switched on.* Everything on
both sides exists — the resolver returns a whole pin's row from Rust so no
geometry is re-derived in GDScript, and the no-art path is byte-identical to the
committed file across a full frame. The single remaining line is **withheld on
purpose**: installing it puts trait art on the live map, which falsifies the
pack-import warning's `trait` clause, and removing that clause moves a golden
literal only the owner can authorise.

*Seventh consecutive batch with a false clause in a lane's own new prose.* Three
of them this time, all naming the same wrong precondition — "until a pack is
imported", when the real gate is that no resolver is installed. A world can hold
a pack full of trait art and still draw discs.

**2026-09-04, twenty-sixth batch — 98 → 96.** Trait badges reach the map and
labels now clear them; the GeoJSON parser is built and stops at the ruling it
needs. `cargo test --workspace` **3 087 → 3 129**, 0 failed. Every verifier
verdict confirmed.

*The label-clearance fix was falsified before it was believed.* Per-pixel
intersection of a settlement name's own ink with the badge row, three fixtures,
before → after: **70 → 0**, **236 → 0**, **75 → 0** px. The verifier re-measured
against the actual committed file rather than the lane's `_PreChange` subclass,
and confirmed the no-trait path byte-identical across the full 2 400×1 200 frame.

*And the port had not "fallen behind the reference" — it had dropped a parameter
present in the very lines it cites.* `lblCandidates` carries `drop` at both
v2.10:15716 and v2.11:16199; HEAD's port took five parameters where the
reference takes seven.

*A premise in the batch brief was wrong, and the lane caught it at the symbol.*
`composite_trait_badges` is a plain `pub fn` taking a raster buffer and a Rust
struct — **GDScript structurally cannot call it**, so the pack-art half was
never closable from `map_overlay.gd`. Four Rust doc comments asserting "the
caller is GDScript" were false and are corrected. What shipped instead is the
reference's own no-art branch, which is this port's only state for every pack.

*Sixth consecutive batch with a false clause in a lane's own new prose*, plus a
botched line-wrap that left five stray tabs mid-expression — it parsed, because
tabs are whitespace, and only a verifier reading the bytes found it.

**2026-09-04, twenty-fifth batch — 97 → 98.** The bounded paint upload closes
end to end; the trait-sprite Rust half closes and the row re-scopes to the one
GDScript caller it still needs; two cross-lane findings filed. `cargo test
--workspace` **3 075 → 3 087**, 0 failed.

*The win survived contact, and is quoted with its spread rather than as a ratio.*
Per dab in the shell, before → after: 512² **1.44 → 0.85 ms**, 1024²
**4.44 → 1.12 ms**, 2048² (the shipped default) **16.51 → 1.85 ms**. Sixteen
milliseconds per pointer-move sample is a dropped frame at 2K, and it is gone.
Proven byte-identical to the full re-upload after every one of 20 dabs, 8 of 8
mutants killed, and the verifier re-derived it on its own world with its own
brush path. The four boundary states are kept distinct — including a world
regenerated mid-stroke, where the new window would have *fitted* the stale
mirror and was refused anyway.

*Fifth consecutive batch with a false clause in a lane's own new prose, and this
time two.* One attributed the fallback dab to the "after" column's maximum
(2.55 ms) when the fallback measures **15.3 ms** — a cause asserted without
measuring the cause. The other said the sculpt and paint preview rasters share a
format; they do not (`RGB8` at `lib.rs:8334`, `RGBA8` at `:9284`), which makes
the opt-in flag load-bearing for a *stronger* reason than the sentence gave.
Both corrected, along with a `blit_sprite` doc that called the function
bottom-anchored three lines above the centre-anchored caller that had just
landed.

*Worth knowing for every future probe:* `ImageTexture.update()` is a **no-op
under `--headless`** — reproduced independently. A pixel probe against a texture
updated that way must run windowed or it proves nothing.

**2026-09-04, twenty-fourth batch — holds at 97.** The Journey panel width
closes; the bounded paint upload's Rust half closes and the row re-scopes to the
shell wiring. `cargo test --workspace` **3 068 → 3 075**, 0 failed.

*The row's own diagnosis was wrong, and it was written by the main loop.* It said
"the panel is dropdowns whose minimum is their widest item". There is **no
dropdown anywhere** in `build_results()` or its fourteen helpers — every dropdown
in the planner lives in the left dock. The real causes were an `HBoxContainer` of
action buttons demanding the **sum** of their sentence-length labels (209 + 202 +
8 = 419, +22 padding = **441**), and unbounded `_kv_row()` label/value pairs. Fix
was 12 lines: flow instead of box, and expand-plus-autowrap on whichever side is
naturally wider. **441 → 258 px body, 456 → 280 dock, identical on three seeds.**

*The open question from last batch is settled by measurement rather than left as
inference.* At `HEAD~1` the dock measured 441 / 441 / 441; at `HEAD` the same.
**Pre-existing, not caused by rule 8's append** — and the verifier re-derived it
on three seeds of its own choosing.

*A verifier stopped a false rule entering `MISTAKES.md`.* A lane reported that
the non-console Godot binary "writes nothing to a redirected stdout"; measured,
it writes 602 bytes. Recording that would have made a standing rule out of a
wrong observation.

*Fourth consecutive batch in which a lane shipped a false clause inside its own
newly-written prose* — this time "absolute figures move a few percent between
runs, the ratios do not", refuted by re-running the bench. Only the **byte**
ratio is stable, because it is `grid / window` arithmetic rather than a timing.

**2026-09-04, twenty-third batch — holds at 97.** First batch at two build
lanes. Rule 8 closes: the Journey planner appends instead of replacing, per the
owner's 2026-09-04 ruling. The pack-section re-derivation lands as the audit it
was asked to be. `cargo test --workspace` unchanged at **3 068**, 0 failed —
correct for a GDScript lane plus an audit.

*The hazard was carried across, and it was not where the ruling described it.*
The ruling, the row and `right_dock.gd`'s own doc all describe a conversion as
three artefacts — a `CTX_` constant, a `CTX_TITLES` row, a `_dispatch()` arm.
There was a **fourth**, in a different file: `journey_planner_view.gd::
build_results()` opened by clearing `right_dock_body` **itself** — harmless while
the planner replaced the dock, destructive the instant it appends, because the
selection lives in that same container. Mutation-verified: re-inserting only that
teardown turns the probe red at exactly the right check.

*A third premise about the pack warning failed, which is why the owner asked for
the measurement.* The backlog row said `composite_map_icons` draws settlement and
poi. It does not — it composites the `icons` family and nothing else, so those
two are undrawn as well. **True unused set: `seamarks`, `settlement`, `trait`,
`poi`, `custom`**; composited: `textures`, `biomes`, `terrains`, `icons`. Only
one section is emittable by the warning today.

*A verifier caught a single-sample measurement.* Lane A reported "no dock
overflow" from one world whose plan was empty; three worlds measure the results
panel at 351 / 385 / 441 px against a 280 px dock. Filed above, with what is
measured kept apart from what is inferred.

**2026-09-04, twenty-second batch — holds at 97.** Vault milestone 3 closes as
already done (shipped 2026-09-02 in `4ec07f5`; `STATUS.md:846` already said so);
right-dock rule 7 is built; the preview row is re-scoped by measurement rather
than reworded. `cargo test --workspace` **3 037 → 3 068**, 0 failed.

*Two counts in the record were wrong in the same direction — too high.* The
right-dock ladder has **8 reachable rules, not nine**: the ninth (`ROUTE`) is a
dead entry the spec itself flags, since `rdMode4()` returns `way` for both tools.
Of the eight, only **rule 7** was genuinely unbuilt; rule 5's remaining half asks
for a settlement inspector this dock already draws.

*A real data-loss bug, found by a verifier and fixed here.* `vault.json` was
written only when `links` was non-empty — one member of a three-member store — so
a project with a connected vault and a map snapshot but no knowledge links wrote
**no document at all** and lost the snapshot on save. The predicate that answers
for all three (`LinkStore::is_empty()`) was built and mutation-tested in the same
batch and simply was not wired. Now wired; the call-site guard is filed above,
because `project_save_with_documents` takes gdext types and no Rust test reaches it.

*The paint preview's decline was measured false.* Its prose called the saving
"negligible"; a full-grid rebuild costs **0.73 / 1.48 / 4.55 / 16.80 ms** per dab
at 512/1024/2048/4096 squared and re-uploads **1 MB to 64 MB** each time, while
`touched_bounds` covers **1.80%** of the grid at 2048. A second comment asserting
the preview *is* cheap per dab was corrected with those figures. The sculpt half's
decline, by contrast, still holds and is owned elsewhere.

**2026-09-04, twenty-first batch — 99 → 96.** DS-03 closes, the pack-manifest
re-baseline closes, the religion share export closes. `cargo test --workspace`
**3 024 → 3 037**, 0 failed.

*DS-03's premise was false and the lane found something worse.* The row (and my
brief) said the tablet deletes ~30% of desktop content. **The deletion set is
zero** — 52 `is_tablet()`/`_touch` branches in `shell/`, every one reflows, none
deletes, confirmed by an identical per-class control census at both densities
across all 10 (domain, mode) pairs. The "~30%" is a property of the *artboard*
(`GUI_GAP_REGISTER.md:9529`, tablet 2560 vs desktop 1920), never of this shell.
**What was really losing content was horizontal overflow through a
`SCROLL_MODE_DISABLED` axis** — the MISTAKES trap, third occurrence. Measured at
tablet: CARTO ▸ Labels forced the 400 px left dock to **1 589 px**, eating 1 189 px
of map; CIVIL ▸ Factions to 555; Landmarks to 417; with a world, WORLD ▸ Generate
to 783; and a fifth panel the lane's own report missed, CIVIL ▸ Infra at 597.
**Desktop was broken the same way** (472 and 1 212 px inside a 372 px slot), so
two desktop panels deliberately move — both from rendering outside their slot to
fitting it.

*The project's first authorised golden re-baseline landed, and held its scope.*
Exactly one string and three fixtures moved; a workspace-wide diff of every
golden and fixture confirms nothing else did. The permanent divergence is
disclosed at five sites a future parity run will actually meet, and all three
fixture sites are load-bearing — re-adding `unused.push("biomes")` is killed.

*The ruling's own premise did not survive being re-opened, and the lane did not
widen anyway.* `trait` is **not** the only true clause: `composite_map_icons`
draws settlement and poi sprites too. Lane B pinned the decision with a test
whose doc says changing it **is** the disclosure, and raised the question rather
than acting on it. That is the right call and it is an open owner question.

**2026-09-03, twentieth batch — holds at 99.** The save-format provenance gap
closes; the religion screens close and are replaced by the narrower engine gap
they exposed; two rows are added from verifier findings. `cargo test --workspace`
**3 015 → 3 024**, 0 failed.

*The batch's most valuable finding was a sentence that had been true of nothing.*
The religion roll-up read `Sun Cult — 9 816 people (8.0%), leads 20 settlements`.
Measured over a real world: **158 of 173 settlements have population 0** (village
add-ons, faithful to the reference), so their adherent dictionaries arrive empty
while their plurality is real. The 9 816 people were minorities inside the 15
populated towns Sun Cult does **not** lead, and all 20 it leads hold nobody —
**the two halves of one sentence shared no settlement.** Alongside it, 43
settlements the roll-up counted for a faith had rows that named no faith at all.
Both fixed, and the hover card now says *why* a pin has no share — proved by a
**framebuffer difference** (25 391 of 2 160 000 bytes, card 276 px vs 157 px,
the populated card byte-identical between arms), not by reading the scene graph.

*A verifier refuted the owner's own ruling being met.* Rule 1's conversion made
`_tool_section()` answer with exactly one id, and its `match` reached
paint/territory/label/icon **before** the draft clause — so arming any of those
four took Commit, Discard, Undo and Redo away from an uncommitted sculpt draft,
and Paint drew *its* Commit in the same slot. That is "nothing is yanked away"
breaking where it matters most. Fixed and measured: all four now read
`stack=true`. A second, distinct gap the same probe exposed is filed above.

**2026-09-03, nineteenth batch — 101 → 97.** Four rows close: the clipboard model
and all four Edit commands (step one had closed 10/10 a batch earlier); the APK
probe-scene exclusion; the coordinate-units probe; and §2.1's last delivery gap.
`cargo test --workspace` holds at 3 015, 0 failed.

*A fourth consecutive batch found a row describing work that was already built.*
DS-03's resolver — `DccTheme.ROLE`, `role_px()`, `is_tablet()` — has existed since
2026-08-31 with **87 live call sites across eight shell files**, and `is_tablet()`
already avoids the predicate `GUI_GAP_REGISTER.md` §57 refuted. The row called it
unstarted for three days. The lane guarded it instead (7/7 mutants killed) and
found a real defect while doing so: `ROLE["h_rail_head"]` read `[29, 34]` against
both the canvas and the shipped shell, now `[30, 44]`. **The reflow half is what
remains**, and it is blocked on nothing.

*The owner's scoped `export_presets.cfg` authorisation was needed and was
exercised* — exactly one line, only the `exclude_filter` key of the Android
preset: `"addons/godotsteam/*,addons/godot_ai/*"` gains `,_*`. Measured rather
than assumed: an export pack now stores 147 files, **0** of them beginning
`res://_`. (The main loop reported mid-batch that `_*` was already present. That
was a **torn read** of a file the lane was concurrently editing — see MISTAKES.md.)

*The main loop edited this file while a verifier was running* — the APK row below
— and the verifier caught the mid-run change as an unexplained diff. Recorded in
MISTAKES.md: a doc the main loop owns is still part of the verifier's baseline.

**2026-09-03, eighteenth batch — 101 → 100.** **Urban morphology is finished.**
Milestones 16 and 17 both closed, and **neither was open**: 16 shipped in
`cff1edc` a day before the row said it "remains … blocked by definition", and
17's stated blocker — "settlements carry no `specialisation` and no `traits`" —
was falsified **six minutes after it was written** and stood for eleven days. One
genuine gap surfaced, was filed in §2.1, and closed the next batch
(`settlement_layout` → `_with`).
Lane C corrected nine false shell claims. `cargo test --workspace` **3 010 →
3 015**, 0 failed.

*The main loop got this batch's premise wrong and the lane caught it.* The brief
asserted "crates/cartalith-urban has no `tests/` directory at all, so the
whole-subsystem golden genuinely does not exist" — the premise was true and the
inference false: this crate puts fixtures at `src/<module>/tests/golden.rs` by
convention, and the milestone-16 golden is 3 139 lines of it. **Absence of a
directory is not absence of a test.** The lane re-derived the golden from the
frozen reference anyway and proved it byte-identical, which is why the wrong
premise cost nothing.

**2026-09-03, seventeenth batch — 103 → 101.** Three §2.5 rendering rows closed
(geology microtexture / dune ripples; sky-view-factor and cast-shadow; SDF coast,
river and biome tinting), one new small row filed for the leg that is genuinely
unbuilt (the vector river overlay). `cargo test --workspace` **2 992 → 3 010**,
0 failed, 25 ignored; byte-identical at the default, no golden re-baseline.

*This batch is the clearest evidence yet for the preflight table's first row.
**Two of the three closed rows named a blocker that was false, and the renderer's
own module doc was the source of it** — each row cited `render.rs`'s "deliberately
excludes" list as evidence, in the file that had already implemented them. A third
lane re-opened six audit rows at their symbols and found all six still open, which
is the same discipline returning the opposite answer: re-opening is not a formality
that always closes something.*

*The audit lane also found **three new false claims of the most expensive kind** —
prose asserting a whole Rust module does not exist. `world_workspace.gd:159` said
Köppen classification is "not ported" (`cartalith-climate/src/koppen.rs` is
golden-tested and drives a live layer); `performance_window.gd:140` said no
per-device GPU enumeration exists (`multi.rs:378`); `civilization_workspace.gd:5405`
said cartalith-civ has no faction relation (`relations.rs` exists to create that
edge, and a surface 330 lines above the note already draws it). `git log -S` dates
all three as false for **fourteen to sixteen days**. They sit in panels that
otherwise work, so no disabled-control sweep reaches them — see MISTAKES.md.*

*Two of this document's own prior claims were retracted by the same lane: a
"CORRECTION" that asserted the opposite of the source comment whose line range it
cited, and a provenance exoneration refuted by `git log -S` on the same note.*

**2026-09-03, sixteenth batch — held at 103.** PH-15, the navpad hover tint and
the label-clipping residue all closed; PH-16 closed **in the state the panel
owns** and re-filed for the state it does not; two rows added.

*The lane's most valuable move was one nobody asked for: **it added a control
state.** The register had measured exactly one — planner open, no world — and read
the result as this panel's defect. Measured against planner *closed*, opening the
planner **removes 447 blank rows** (1 494 → 1 047). So most of that band is the
app with no world loaded, and filling it would have been decoration over a world
that does not exist. A number with nothing to compare against cannot say whose
defect it is.*

*The band the panel does own was real: `_RouteMapView._draw()` returned at
`pts.size() < 2` while `map_texture` already held the world render, and the
comment beside it asserted there was no texture to show — both halves false.
With a world and no route, **253 rows → 98**.*

*A defect the register never caught: `_route_map_wrap` laid out **1 437 px wide
on a 1 080 px screen**, because a `ScrollContainer` with an axis DISABLED folds
its child's minimum size into its own and the overflow propagates past
`PRESET_FULL_RECT` with no scrollbar to reveal it. Now 1 080.*

*And a self-inflicted regression it caught by re-measuring rather than assuming:
`clip_text` plus ellipsis collapses a Label's minimum width to **1**, so beside a
`SIZE_EXPAND_FILL` sibling the text vanished — removing a real line of text and
**raising** the blank-row count 1 047 → 1 072. Both are new `MISTAKES.md` rows.*

*The baseline itself had to be rebuilt first: the previous run's `blank_rows=0`
was 0 by construction, because this machine boots `mode="light"`. The probe now
forces dark **and refuses to run otherwise** — the mechanism the verification
brief demanded without supplying.*

**2026-09-03, fifteenth batch — 104 → 103.** The right dock now **appends** as the
owner ruled, and the unavailable-command row is re-cut against a measurement
rather than a memory.

*The dock lane's own probe went red on its first run and found a bug the fix
created: **the Paint section outlived its own tool**, because nothing called
`leave_paint_context()` when another tool armed inside WORLD — harmless while the
dock was a whole-panel takeover, a stale panel under a live selection once it
appends. It also caught the second-order hazard, that `armed_tool` survives a
domain switch, and gated Paint on WORLD for the reason the old code had.*

*The sharpest finding is about a **test**, not the code: `_rightdock5_probe.gd`
was green while pinning the design the ruling rejects — six checks asserted
`_context == "paint"` and friends, and a seventh was literally
`_check("...", true, ...)`, an unconditional pass. A probe can enforce the wrong
design as confidently as the right one.*

*Menu commands: the row claimed **21 unavailable of 356**; the probe measures
**374 total, 15 unavailable**. Two of the sixteen reasons were false — one a
description of what the command does, standing where its justification should be,
and `command_index.gd` reads exactly that field as the reason.*

*Both agent failures this batch were **infrastructure** — a `server_error` and a
`529 Overloaded` — not code. The phone lane was relaunched on a stronger model
and resumes rather than restarts; its predecessor's output was parse-clean and
test-green in the tree.*

*Four defects in the verification brief, one of them serious: it demanded a
dark-theme pixel count while naming **no mechanism for getting dark**, and this
machine boots light — reproducing, inside a brief that cites the rule, the exact
trap that rule exists to prevent. `MISTAKES.md` carries it as **citing a rule in
a brief is not satisfying it**.*

**2026-09-03, fourteenth batch — 103 → 104, and the verification was the batch's
real output.** Shell stages 5 and 6 landed what the design supplies; the count
went **up** because measuring properly turned two "done" claims into open rows.

*Stage 5's lane behaved well where it mattered: it **declined** `mapCursor`,
`layersBtnBg/Col` and the tool-options bar because §0 lists those bindings as
absent from the delivered prototype, and confirmed `statusMid` prints no invented
number. All 19 of §0's missing bindings were found in the re-export.*

*Five refutations, four fixed here. **The right dock replaces rather than
appends** — measured in a booted app, `settlement name SURVIVED=false` on arming
a tool — which is the naive merge the owner's ruling explicitly rejects, and a
lane had signed it off as satisfied. Now its own row. **The scale bar lost AA**:
`_chrome()` moved it from `text_faint` to `text_dim` while giving it no scrim, so
its background is the map — 3.14:1 over a white map on dark, 4.11:1 over a black
map on light, both from above the line to below it. The nine ratios the lane did
compute were all correct and none of them was this pair. **`vpContext` appended
`EDITED`/`RESOLVED` to every domain** where `ENV:1889` gives the verdict to WORLD
alone. And a **false rationale had shipped into source**: "leaving the map live
… buys back the one thing turning it off broke" — flipping that flag changes 0
of 288 000 pixels, because the panel is the opaque cover.*

*PH-16's own "blank_rows=0" proof was **0 by construction**: the probe borrowed
the register's dark-theme `>23` threshold and ran it on a light capture where
every background pixel is 251. Re-run in dark: **1 069 of 2 400 rows** blank
against the register's original 1 434 — reduced, not gone. The row now carries
that number.*

*The verifier found **three defects in the verification brief**, all correct —
including `git diff 0bba2f9 HEAD` where `0bba2f9` **is** HEAD, written while
anticipating the previous batch's version of the same error. That makes four
unfalsifiable checks shipped in briefs; `MISTAKES.md` now carries the rule as
**ask what result would refute the claim, then check the instruction can produce
it**, plus three new preflight rows the verifier proposed.*

**2026-09-03, thirteenth batch — shell stages 3 and 7, and the religion screens.**
Stage 3 (menus) came back **already done**: all 29 `PopupMenu.new()` sites route
through `DccWidgets.style_popup()`, which reads the tokens, so stage 1's re-base
*was* the restyle — zero edits, `_cmdindex_probe` PASS at 374 entries unchanged.
Stage 7 restyled the nine windows, and the religion screens shipped over the
belief engine that landed the same day.

*The stage-7 lane found **four defects the 2026-08-31 token re-base had caused
and nothing had checked**: an asset-library checkerboard whose contrast fell from
(7,8,8) to (2,3,4) — in the exact pair a comment two lines above recommended — a
trait-chip hover that became a darkening where it had been a lift, a drag preview
invisible on light because `raised` and `panel` had become byte-identical, and a
verdict green left as a raw literal at **1.96:1** on the light panel. A re-base is
verified against its sources; the properties that matter are the differences
between values, and no test covers those. New `MISTAKES.md` entry.*

*It also found the plan's own numbers disagree: §2's "nine windows" and
`STATUS.md` RP-S7's nine are **not the same nine**. RP-S7's list was used.*

*The verifier found three defects in **the verification brief itself**, all
correct. Its `project.godot` check stopped being evidence the moment the main
loop committed mid-verification — a clean tree makes `git diff` empty for every
file — re-checked properly as `git diff 8382744 HEAD` (unchanged, 75 comment
lines intact). Its probe-guard check could not discriminate, because another
lane's `menus.gd` rewiring landed before the baseline was taken. And its "count
settlements shown a default religion" is always 0 by construction, since
`religion` and `adherents` are emitted together for every settlement. Two are
now `MISTAKES.md` preflight rows.*

**2026-09-03, twelfth batch — headline holds at 103.** The layer-stack UI closed
(section 7's row list in CARTO, RD-10's right-dock Layers section **appended**
rather than replacing the selection, per the owner's dock ruling, plus WCAG 2.2
SC 2.5.7 Move up / Move down beside the drag). **PH-16 narrowed rather than
closed**, and one row was added, so the count did not move.

*PH-16's cause was real and is fixed: `journey_planner_view.gd` pre-scaled six
heights by `phone_scale()` **and** let the shared `phone_fit()` walk multiply the
same subtree again, so every pre-scaled row rendered at `phone_scale()`². But a
verifier measured the panel still reporting `(1080, 2400)` — a full-screen
takeover — with `_show()` still switching the viewport off. The register's
complaint is a pixel one; the fix so far is geometric.*

*Three defects the verifier found in the layer UI, two fixed here. **Reset to
quality tier never re-synced the Layers panel**, so the engine returned to the
default order while the panel kept drawing the user's arrangement — fixed. **The
Colour relief row is live over a layer that draws nothing** at the shipped
default, because `ramp_strength` is `0.0` and `composite` skips it: disclosed in
the left dock, and left as its own row because the honest end state is a
judgement about the default, not a patch. A comment's `phone_scale` arithmetic
was two rebases stale (2.748 where the probe prints 2.621) and is corrected.*

*The verifier also refuted **the verification brief itself**: "measure PH-16 at
393×852" cannot discriminate, because `phone_scale()` is exactly 1.0 at that
size. The lane's choice of 1080×2400 was correct and the brief had called it an
evasion. `MISTAKES.md`'s orchestration entry is now ×4 and carries the rule: check
your own verification instruction is discriminating before demanding a lane
satisfy it — a test condition that cannot fail is worse than none, because it
looks like rigour.*

**2026-09-03, eleventh batch — the first under the owner's new GUI order, which
puts the §3.2 rows blocked on *other work* first.** Both preconditions are built,
so **two rows moved §3.2 → §2.2** and only their UI halves remain. The headline
holds at 103; nothing closed, two things became startable.

- **The layer stack.** CA-04's stated reason was **wrong about the pixels**: it
  said `render.rs` bakes the categories into one pass needing an architecture
  change, but both composites already existed inside `land_color` — colour relief
  as a normal-over lerp, hillshade as a multiply — with the operator and slot in
  *source* rather than in *data*. The fix was register-composited, costing no
  allocation; N buffers were measured at **368.8 MiB** at the 8192 export ceiling
  and rejected. Byte-identity at the default is by control flow, proved with **8
  FNV digests taken before the change** and unchanged after.
- **Belief.** The row claimed `cartalith-civ::belief` does not exist. It was
  already 945 lines — compatibility tables with no diffusion, no callers, no
  bridge. That is the **twelfth** row this week that did not survive being
  re-opened at its symbol.

*`export_raster.rs` needed no change, and that was structural rather than lucky —
the stack lives on `TerrainAppearance`, which all three raster consumers already
fetch. The lane proved it with a test named for last week's failure,
`every_stack_control_moves_both_consumer_paths`, whose doc says it measures
rather than assumes **because `with_ground_tiles` did not**, moved no pixel at the
default, and left the suite green while every exported PNG diverged from the map.
That is `MISTAKES.md` being applied before the mistake rather than after it.*

*The verifier found the same class a third time anyway, in the other lane: the
belief staleness key covered `belief_seed`'s second argument and not its first,
so reassigning a settlement to a faction of another faith left it showing the old
religion while the guard reported itself current — **and that was the fix for the
identical miss on the religion column**. The key is now derived from the
function's signature, `culture` is documented as deliberately uncovered with the
reason, and `MISTAKES.md`'s entry is widened to "covering some inputs of a thing,
not all of them ×3".*

**2026-09-03, tenth batch, taking 111 → 103** — nine rows closed, the largest
single drop yet, and most of it came from *measuring* rather than building.

- **`gpu_compute_height` is an undocumented decision, and the right one.** The
  blocker was never written down: `HEIGHT_LAYOUT` binds **9 storage buffers**
  against `REUSED_STAGE_MAX_STORAGE_BUFFERS = 8`, so there was never a device it
  could be built on. Worse, its recorded 5.17×/8.13×/4.84× speedups were against a
  *single-threaded f32 twin*; against the production `compute_height`, which is
  f64 and already `par_chunks_mut`, it wins **2.13× at 1024² and 1.15× at 2048²**
  — about 5 ms, against a handshake two orders of magnitude larger. Not wired, and
  now documented with what would overturn it.
- **The `gpu_height` throughput drop is upload-bound**, hypothesis tested and
  confirmed against a cross-kernel control: the two narrow kernels get *cheaper*
  per cell from 1024² to 2048² while only the nine-buffer bind group turns around.
- **`build_road_network` is parallelised — and has no production caller.** All six
  call sites are tests; the shipped path is `civ_road_network`, already parallel.
  Said out loud rather than filed as a speed win it is not.
- The `_peakaudit_*` probes are deleted, and the ocean lattice and hand-lettered
  glyph rows resolved.

*The verifier's finding is the one worth remembering: **the lane that closed the
benchmark-averaging row then committed that exact defect three times**, writing
single-sample figures into two doc comments and a scope document as measured fact.
None reproduced — 416 ms re-measured at 730, a "5× spread" at 1.4%, a "halving" at
1.35×. All three are now ranges or directions, and the residue is its own row.*

*Two claims were false rather than imprecise and are fixed: `render.rs`'s module
doc still listed `rockSlope` refinement and wetness darkening as **excluded** in
the same file that had just implemented them — and that doc is
`OUTSTANDING_WORK.md`'s own cited location for the row — and `STATUS.md` named
three deleted probes as "present and uncalled". A dangling
`examples/_peakaudit_peak.rs` citation in `landmark.rs` went with them.*

**2026-09-03 — four owner rulings on the GUI blockers, taking 112 → 111 and moving
four rows without closing any work.** The owner reprioritised GUI and answered the
four rows sitting in §3.1 *blocked on an owner decision and nothing else*. Full
text in `LARGE_ITEM_RULINGS.md`'s second section.

- **DS-03: keep everything, reflow only.** The tablet gets the full desktop
  inventory; nothing is removed. That retires the *content* question outright —
  there is no "which controls leave" list to build — and leaves the row as its
  architectural half alone: `DccTheme.TABLET`'s exhausted key space. → §2.2.
- **`rdExtraMode`: selection wins, the tool appends.** This answers
  `UNWIRED_FUNCTIONS.md`'s open question 1, which is deleted from §4 — the only
  reason the headline moved. → §2.2.
- **The APK probe scenes: excluded, under a scoped authorisation** to edit
  `export_presets.cfg`'s `exclude_filter` **and nothing else in that file**.
  `Cargo.toml`, `.gitignore` and `project.godot` remain off limits. → §2.2.
- **CV-24 / ED-02: both wait for a design pass.** `TIMELINE_SCOPE.md` §4's
  instruction to design the panel before guessing its region is upheld rather than
  overridden, so these move §3.1 → **§3.3**: still blocked, on a design rather than
  on the owner. Not closed and not startable.

*Owner question 3 — the WORLD left-dock A/B switch — was deliberately **not**
asked. It is doubly blocked: its captions and gate live in the truncated tail of
`02-rail-and-domains.md` §8, so there is no label to build the control with even
once the call is made, and an answer would not have been executable.*

**The owner's GUI order, standing:** the §3.2 rows blocked on other work first,
then the unblocked rows, then the rows blocked on a design that does not exist.

**2026-09-03, ninth batch, taking 115 → 112.** Four rows closed, and three of the
four were **already built** — the manual-icon tool's arming, rendering and
persistence all resolve at their symbols (`icon_arm` → `IconEditor::arm`;
`viewport_host.gd` drawing `icon_list()`; `SLOT_ICONS` round-tripping 53 icons
with per-instance scale intact), and so does **CA-05**, whose row said the icon
tool has no on-canvas resize handles while `#[func] icon_handles`
(`lib.rs:7759`) has a live caller at `cartography_workspace.gd:966`. The one
genuinely open gap was the density brush, which is built. The layer live-sync
signal and the `LIVE_LAYERS` assertion were likewise already done.

*The pack-import warning was **reclassified rather than fixed**, and the lane was
right to decline the brief's instruction: that string is pinned by golden fixtures
captured from the reference under Node, so editing it is a re-baseline
`DECISIONS.md` §7a protects. It also found the urgency lower than stated —
`PackManifest::warnings` reaches no user, so the false claim is false in code and
not on screen.*

*The verifier refuted four claims. **Four brush constants and the brush seed were
pinned by nothing** — the clamp test compared each constant against itself, the
same shape that let `MIN_REGION_WORLD_AXIS` survive `4 → 3`. Now pinned to the
reference's literal slider attributes (`#carIconBrushR` at 1656, `#carIconBrushD`
at 1657 over the `/100` at 13515) and all four die under mutation. **My own first
replacement for the seed test repeated the defect** — two fresh editors agree
whatever the seed is — and needed the literal added before the mutant died. A
`km²` readout in `world_workspace.gd` was still going through `_thousands` beside
the converted siblings, which is the half-fix `format_area`'s own doc names; and a
spliced sentence in new prose was corrected.*

*One row added: the Coordinate-system units fix is real but nothing would catch
its removal.*

**2026-09-03, eighth batch, taking 116 → 115.** **Saved measurements + CSV** ships
as the *fifth caller* of the slot path rather than a second mechanism, which the
ruling required: one line in `DOCUMENT_SLOTS` was the whole Rust change, because
the channel built last batch was shaped to take it. The document carries `gw`/`gh`
and **refuses on a grid mismatch, then clears** — staleness-marking was rejected
and the reason recorded, that a marked reading stays readable and plausible while
the points under it name different ground. The CSV is canonical km, verified by
re-exporting under mi and comparing bytes. **Pack biome/terrain decoding** landed
with the default output unchanged.

*Two verifier refutations, both real, both fixed here. The measurements **reader**
had a data-loss defect the write side hid: `float(<null>)` is a GDScript runtime
error rather than a conversion, so one null value aborted the whole reader, the
caller took the `ok == false` branch and cleared the in-memory list, and the user
lost every healthy reading beside it — silently, with no reason line. This build
writes that null itself for a NaN. Guarded, and the probe now proves it: reverting
the guard turns "3 of 3 survive" into "0 of 3". And pack decoding wired
`with_ground_tiles` into the on-screen builder only, so with a pack applied and
cells painted the map blended the pack tile while **every exported PNG blended the
flat swatch** — a new divergence, and one the reference does not have
(`landColorCore` reads the same `assetPack` global at 8168, 11730 and 11969).
Attaching it moves no pixel at the default, which is exactly why no golden caught
it.*

*One row added: the pack-import warning still names biomes and terrains as unused
sections, about two families the map now composites.*

**2026-09-03, seventh batch, taking 120 → 116 — and the duplicate is finally gone.**
Four rows closed. The **vault leak**: `import_heightmap` called `absorb` without
`release_world`, so links and snapshots taken against the old world survived into
the imported one. The clear moved **into `absorb`** rather than to the call site —
`absorb` is the funnel all four generated-world paths share and the function that
replaces `self.civ` wholesale, which is what makes every `entity_id` in the store
meaningless; a call-site fix leaves the fifth path free to repeat it. Proven by
revert: deleting the two lines fails 2 assertions, **on the import arm only**.
And the **four caller-owned save slots** now round-trip, which closes the last
duplicate-classification defect this section has carried: *Saved measurements +
CSV* was filed both as ready (§2.2) and as blocked on this work (§3.2), and the
blocker has landed, so the §3.2 half is deleted.

*The atlas row was narrowed rather than closed, on the lane's own honest finding:
the live collision is fixed and was measured real first — three different worlds
at one parameter tuple all hashed to `beffe825` — but the save format records no
provenance, so the discriminator cannot survive a reopen. Stated as a format gap
rather than worked around.*

*Two verifier refutations, both the "a fix nothing asserts is not a fix" class,
both closed here. `WorldGen::carried_foreign` was pinned by nothing — replacing
`std::mem::take` with `Default::default()` left the whole Rust suite green, and no
unit test can reach it because `WorldGen` is a cdylib `GodotClass`; `_savetree_probe.gd`
now drives a real open→re-save with a non-UTF-8 foreign payload and the mutant dies
("foreign entry was DROPPED by the re-save"). And `is_own_entry` has **seven**
branches where its fixture reached four — `params.json`, `README.md` and
`preview.png` each survived the entire workspace suite. Now table-driven over all
seven plus eleven near misses; all three mutants die.*

**2026-09-03, sixth batch, taking 119 → 120 — the count went UP, and that is the
batch's most useful output.** Two rows closed: **Units** (the Measure panel, the
Region-select extent, and — after a verifier caught the same function half
converted — the `radius`, `section` and `area` arms beside them, the last needing
a new `DccUnits.format_area`, since 100 km² is 38.6 mi² and not 62.1) and
**Region ▸ New world from selection** (`region_as_new_world` plus the `WorldGen`
state work the ruling insisted must not be folded into GUI work).

*Three rows were added because the region lane enumerated `WorldGen`'s fields
against `absorb` and `close_world()` instead of trusting the five the ruling
named, and found two pre-existing defects it correctly escalated rather than
silently repairing: `import_heightmap` leaks vault links and snapshots across a
world replacement, and the atlas `world_key` hashes parameters but never how the
field was produced, so a resampled, an imported and a generated world can collide
in one namespace. A verifier added the third. **A backlog that only ever shrinks
is not being read carefully**, and this is what looking properly costs.*

*The Layers row was narrowed rather than closed on the same evidence: the lane
claimed a build-time read covers a later cross-panel click, but `_register_workspaces()`
builds all five workspaces eagerly at launch, so CARTO's checkboxes exist before any
click can happen. And `MIN_REGION_WORLD_AXIS = 4` survived mutation at `4 → 3`,
because its test compared the constant against itself — now pinned to
`generate_sized`'s own `grid_w.max(4)`, and the mutant dies.*

**2026-09-03, fifth pass — two rows closed by re-reading, not by building, taking
121 → 119.** Both said "Built" and stayed open on a residue that later waves had
already fixed, so the *rows* were stale rather than the work. **Civilisation
authoring**: the `CivRebuild::Routes` unconditional tail is gated —
`civ_settle_staleness` clears `civ_dirty` only when `civ_merge` reports the layer
really was re-derived, with a test per mode. **The river entity**: the third
`f64::hypot` at `enforce_channel_descent` is `js_hypot`, the doc citation is
corrected to reference 4532-4537 *with the correction shown*, and all four
surviving mutants now have pinning tests — including
`enforce_channel_descent_carves_the_v8_hypot_disc`, which asserts the call site
the earlier divergence test only measured.

*This is the fourth time this file has carried a row whose blocker had already
lifted. Re-open a "Built … but" row at its cited symbol before scheduling work
against it.*

**2026-09-03, fourth batch, taking 123 → 121.** **Colour management** ships behind an
sRGB default that is byte-identical *by control flow*, not by arithmetic —
`ColorSpace::Srgb => return` fires before a single byte is read, so no matrix
constant, transfer function or rounding rule can move the shipped image. Proven
twice rather than asserted: an FNV hash of the finished render captured **before**
the feature was written and re-run unchanged after, and a real 2048×1312 round
trip measuring `0.000000 %` of bytes moved. The owner's stated cost was therefore
never paid — no golden was re-baselined. **Rebindable shortcuts** ship per-context
with same-context conflicts surfaced, cross-context collisions correctly *not*
flagged, live reapplication and a reset path.

*Two defects the verifier found, both fixed here. `File ▸ Close project` runs
`close_world()`, which does `world_gen = WorldGen.new()` and re-inits
`color_space` to sRGB — so the picker was left reading Display P3 over an sRGB
engine. The comment above that control asserted the opposite ("opening a project
therefore cannot leave this row stale, because nothing underneath it moved"):
nothing in the **document** moves, which is true and is not the question, because
the **engine** moved. And `menus.gd`'s Colour management `_todo` still read "the
renderer is sRGB-only end to end", false in every clause within the same session
— now a signpost to the real control rather than a `_todo`, which would have made
`command_index.gd` count a shipped feature as missing.*

*Both lanes also mis-attributed their test-count deltas, in opposite directions,
while both absolute figures were right. Noted because the absolute number is the
one this project checks; the deltas were never load-bearing.*

**Earlier the same day, taking 124 → 123.** The two stranded funnel chips are
wired (*Show rejected* draws the real capped list and says so in its label;
*Raise crowding to × N* computes the figure, snaps it to the dial's step, and
refuses off-dial answers by printing them rather than clamping), and **Report an
issue** is now a local diagnostic dump with no endpoint, as ruled.

*The chip lane found something larger than its own task: **the crowding
direction was inverted in three places** in `civilization_workspace.gd`. The
engine divides — `LandmarkSettings::radius_km` is `base / crowding_in_force()`,
pinned by `landmark.rs::crowding_higher_packs_tighter` — while the panel
multiplied. The two agreed at the default × 1.00 and nowhere else, so at × 2.00
the panel read "keeps 68 km clear" where the pass kept 17, and the note printed
directly above the chips told the user to move the dial the wrong way.*

*The verifier then refuted two claims, both fixed here rather than deferred. A
replacement comment asserted "neither chip is ever enabled onto a no-op";
measured, the one kind that reached `ok` placed its promised candidate exactly
as computed and that candidate's own new ring rejected another, for a net gain
of zero — the control's "floor" wording was honest, the comment's was not. And
the diagnostic report printed `VRAM budget: 0.0 GB` where `0` is the sentinel
for **no cap** and is the shipping default, so every stock-install report would
have told its reader the GPU path was budget-refused. **That is the fourth
instance in two days of encoding "no value" as a plausible value**, after
`needs_crowding`, `harbour_scale` and `wall_spec`.*

**Earlier the same day, taking 126 → 124.** Three rows closed, each
verified independently rather than on its lane's word: **CPU worker threads**
(the setter that returned `true` while changing nothing now returns the truth —
root cause was `ACTIVE_THREADS` never being written by Rayon's *implicit* global
init, so the honesty check was reading a counter only the explicit path touched;
the settings value is wired and restored first in `_ready()`, and `menus.gd`'s
`_todo` is a real menu), **Settlement diagnostics** (0 bare zeros over a real
world, 20 fields dashed with their reason, and an audit that found **five more**
fields defaulting a value and printing it as though measured), and the
**Landmark funnel** (both halves, with rejection *reasons* rather than bare
coordinates). One row replaced them: the two funnel chips left stranded on
reasons that the funnel work itself made false.

*A boundary defect was fixed here too, and it is the same defect three times in
one day: `rejects.rs` marshalled `Option<f64>::None` as `0.0`, and `0.0` is a
plausible Crowding, so 44 of 614 spacing rows read as genuine measurements. The
key is now absent and callers use `has()` — the idiom the diagnostics card
settled on for `harbour_scale` and `wall_spec` the same day.*

**Earlier the same day, taking 128 → 126.** Two rows deleted, both verified 10/10 by an
independent adversarial pass rather than on their lane's word: **CARTO ▸ Icons**
(§2.2 — a sea-marks asset family, a generated placement pass, and the coastline
snap test the ruling names, reusing the label culler rather than growing a
second one) and **Label collision culling** (§2.2 — now genuinely wired, see
below). Three more rows were **narrowed, not closed, because their verifiers
refuted them**: CPU worker threads (a setter that returns `true` while changing
nothing), Settlement diagnostics (19 of 203 cards print an undashed `order 0`,
breaking the owner's one binding condition), and Cut · Copy · Paste (step one
only, as the ruling sequences it).

*That wave also caught a regression this file's own headline had no way to see:
commit `0f0fe55` used `_label_cull` twice in `cartography_workspace.gd` and
declared it nowhere, so `class_name CartographyWorkspace` failed to register and
`shell/app.gd` — the application root — would not compile. **The Godot shell was
unbootable while `cargo test` reported 2 821 green.** Fixed, and both files now
parse clean. A separate latent hazard was closed at the same time: the label
comparator used `partial_cmp(..).unwrap_or(Equal)`, which is intransitive on a
NaN weight and makes Rust's sort panic — a panic that crosses the gdext boundary
takes the whole Godot process down. Now `total_cmp`, pinned by a test that
panics if the line is reverted.*

An earlier five-lane wave closed five rows on 2026-09-02 and
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
| Ready to start | 54 | Nothing blocks them; someone has to pick them up (§2) |
| Blocked | 28 | A named blocker, listed in §3 |
| Open decisions | 18 | Not work yet — the owner owes an answer first (§4) |
| Declined / shelved | 25 entries | §5, kept so nobody re-proposes them |

Of the 30 blocked, **10 are blocked on an owner decision and nothing else** —
still the largest single category of stalled work, and §4 remains the shortest
path to unsticking it. That 10 is checkable and checks out: §3.1 holds exactly 10
rows. *It was 14 until 2026-09-03, when four GUI blockers were put to the owner
and answered — three became startable and one moved to §3.3, blocked on a design
rather than on a decision.*

**Every count above was re-derived by counting table rows mechanically,
2026-09-02,** and the per-section figures are: §1 **3**; §2 0+16+5+6+11+22+9+3 =
**72**; §3 14+10+11 = **35**; §4 **19**. *The previous version of this paragraph
gave §2 as 98 and §3 as 33 while the table beside it said 80 and 34 — the
document reproducing, in its own count section, the exact defect §6.8 exists to
record. Both are now derived by the same script that produced the table, so they
cannot disagree.*

**Nothing is now listed twice.** This section carried a classification defect for
days — a row cannot be both ready and blocked, and two were: *Saved measurements +
CSV* and *Label collision culling* each appeared in §2.2 **and** §3.2. Both are
resolved rather than reclassified, and in the same way: their blockers shipped, so
the §3.2 halves became false rather than merely misfiled. Culling went on
2026-09-03 when the labelling pass landed; measurements went the same day when the
four caller-owned save slots it was waiting on began to round-trip. The unique
count and the headline are therefore the same number, **103**, for the first time
since this file was written.

Five caveats on that number, stated rather than buried:

1. **It counts rows, not effort.** Urban milestone 10 is one row and ~407
   reference lines; "delete three probe files" is also one row. Sizes are on
   every row for this reason. The **85** rows that carry a size (everything
   except §4's 18 decisions — and 103 − 18 = 85, so the split is checkable
   against the headline) divide **20 large, 41 medium, 24 small**, re-derived
   2026-09-02 by the same script that counts the rows. *This caveat has now been
   overstated twice: it read "142 rows, 42/56/44" until 2026-09-01 and "134 rows,
   40/54/40" until today, both times because the sizes were counted by hand
   separately from the rows.*
2. **§2.9's three rows hide a survey, not an estimate.** `RC_ENGINE_CHANGES.md`
   specifies **46 distinct engine items** across v2.11–v2.71: 27 through v2.52,
   §6h's three (the 24-bit height word, the relief-gate floor and the
   local-contrast rebase), v2.57's plate-base blur, v2.58's river selection,
   v2.59's drainage default plus its ruling on Strahler order (§6k/§7.12), and
   v2.60's river-continuity fix (§6l), v2.61's water-paint/lake-gate pass (§6m)
   v2.62's navigable-river routing/flow-direction pass (§6n) and v2.67's ward-driven plot grain plus the corner-only parcel water test it exposed (§6s), v2.68's never-rendered farmland fringe (§8.2 — render-only, but a port inherits the invisible-detail-kind defect) v2.69's sea-level clamp on tile refinement (§8.1 — a port that writes its own LOD needs both of its rules before it does) v2.70's style-chain shape (§8.2 — one colour function per surface class is what makes a new map style cost one flag instead of N) and v2.71's water clip (§8.2 — the settlement's water mask exists and never reached the renderer, and the polygon a synthetic fixture clips against is EMPTY by design on the real path) — the last of
   which is the first place §6k's "key the threshold on catchment AREA"
   recommendation is actually taken, for a NEW consumer, leaving the three
   existing `order>=3` consumers alone.
   **That arithmetic closes; the figure read 31 before 2026-09-17 and did not** —
   v2.57 had been added to the total without being added to the breakdown. How
   many are already ported is not established, so they are deliberately NOT
   expanded into 33 rows here. Expanding them before the survey would inflate this count with
   work that may already be done — the opposite error to the one that left them
   uncounted until 2026-09-17.
3. **The `UNWIRED_FUNCTIONS.md` backlog is one row of the 3 "in flight" above,
   not many** — that document is itself a live backlog with a `file:line` per
   row, and re-counting it here would guarantee the two drift (this
   corrects an earlier version of this caveat, which pointed at "the 106
   ready" — the row has only ever lived in §1). It carries **21** open rows
   as of the 2026-09-01 third pass (22 after the second pass, 23 after the
   morning re-cut, 75 before it), **re-verified unchanged at 21 by a second
   full re-cut on 2026-09-02**. Counted individually the true total is
   **123** — this one row swapped for its 21 (103 − 1 + 21). (The figures here
   were "177, not 155" until 2026-09-01, "173" until 2026-09-02, and "153"
   against the 133 headline earlier the same day; each was arithmetic against a
   headline that has since moved, which is why the working is shown.)
4. **Six surveyors returned 487 rows; roughly 300 were `done` or `declined`,**
   and the rest deduplicated heavily — the urban milestones, the landmark
   viewshed and the vault's §26 each arrived from two or three surveys
   independently. The compression is real, not a sampling gap.
5. **Nobody ran the test suite.** "Done" for `UNIFIED_TOOL_PLAN.md` milestones
   A–E means the named crate modules and bridges exist and the commit reported
   green, not that `cargo test` passed this pass. §7 says what else is
   uncovered.
## The three that matter

If you stop reading here:

1. ~~**Urban morphology milestone 16**~~ — **closed 2026-09-03. Urban morphology
   has nothing outstanding.** Milestones 8-15 shipped in `4ec07f5`; the three
   `_um*` adapters and their wiring landed in `cff1edc`; and **milestone 16
   itself shipped in `cff1edc` too** — `generate.rs`, `generate/tests.rs`,
   `generate/tests/golden.rs` and `tools/um_capture.js` all enter the tree in
   that commit (`git log --diff-filter=A`). This entry claimed 16 "remains …
   blocked by definition" for a day after it had already shipped. The golden was
   independently re-derived: `node tools/um_capture.js` reproduces
   `generate/tests/golden.rs` **byte-identically** (md5
   `cf6487380773a5e13c1fdf2c5d54ff94`, 29 cases) from the frozen reference, and
   12 of the 13 stage modules are proven mutation-covered by
   `whole_subsystem_matches_reference`. **Measured limit, not a defect:**
   `hash_model` hashes five loops (edges, nodes, blocks, parcels, buildings) and
   none is an amenity, so amenity *placement* is covered by count and presence
   but not position — `MARGIN 25.0 → 200.0` survives. `rules.rs` is the one
   stage module with no mutation coverage.
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
| **The `UNWIRED_FUNCTIONS.md` backlog** — 2 open rows (1 small · 0 medium · 1 large, deferred by ruling — 0 actionable), 0 open dangerous-class | `UNWIRED_FUNCTIONS.md` | large | **Corrected 2026-09-21 from the code, twice over — two of the three items this row dispatched agents to "build" turned out already built, both caught before landing a duplicate implementation, not after.** **Paint preview patches: CLOSED**, already wired since `45df3019` (2026-09-04) — `UNWIRED_FUNCTIONS.md`'s "no shell caller yet" claim was stale, this row repeated it, an agent found the truth before writing new code, re-verified independently (`_paintpatch_probe.gd` headless, `PASS 0 failures`). **Cut/Copy/Paste: CLOSED**, same shape — the clipboard shipped `686cd2a` (2026-09-03), hours after `UNWIRED_FUNCTIONS.md`'s row was last written; the dispatched agent found it live, built nothing new in the shell, and instead closed the one real gap (the probe's icon leg had never run, misreading "no asset pack ships" as "no pack reachable"), re-verified independently (`_clipboard_probe.gd`, 25 new assertions, `PASS`). **statusMid** is still ~90% built, `repaint NN ms` still blocked on Owner question 2. **The 3D viewport** stays open by ruling (deferred, research first) — correctly parked, not actionable. **`label_glyph_layout`** stays open as a genuine design question (rerouting would silently change today's rendering — an arc+tracking contract gap), not a stale claim, and is the only item left in this whole document that is both open and not blocked. **Dangerous class: 0 open, 9 closed** — re-verified at the symbol, eight directly (`2c825ed`), the ninth by the false string's total absence from the tree. Stale source comments: 0 open / 6 closed. Large-tier header corrected 3→1 across two passes (Region ▸ New world already `CLOSED`; Cut/Copy/Paste now closed too). **This backlog is, as of this pass, functionally exhausted of buildable work** — what remains is one design question and one deferred-by-ruling row. |
| ~~**Wire the already-built paint preview patch into its shell caller**~~ — **CLOSED 2026-09-21 (found already done, not built this pass)** | `UNWIRED_FUNCTIONS.md` (Medium) | medium | **This row was itself a stale-document defect, the same class `UNWIRED_FUNCTIONS.md`'s dangerous-class table exists to catch — filed above from that document's own stale "No shell caller yet" claim, without checking the code first.** An agent dispatched to build the wiring found it already existed: `engine_bridge.gd:3071` wraps `build_paint_preview_patch`, `viewport_host.gd:2295`'s `set_preview_patch()` composites the bounded patch (`Image.blit_rect`+`ImageTexture.update()`, chosen so erase can replace pixels rather than blend), and `world_workspace.gd:3089`'s `_paint_show_preview()` calls it, falling back to a full re-upload only when genuinely needed (no base yet, format mismatch, mid-stroke resize, out-of-bounds window). All landed 2026-09-04, commit `45df3019`, same day as the patch API itself — the row simply was never updated after. **Independently re-verified, not trusted**: re-ran `_paintpatch_probe.gd` headless myself — `PROBE PASS (0 failures)`, matching the agent's own report; code at every cited symbol confirmed by direct read; `project.godot` and the live settings file confirmed unchanged. `UNWIRED_FUNCTIONS.md`'s row corrected in the same pass. |
| ~~**Build the Cut/Copy/Paste clipboard (icons and labels)**~~ — **CLOSED 2026-09-21 (found already done, not built this pass)** | `UNWIRED_FUNCTIONS.md` (Large) | large | The clipboard shipped `686cd2a`, 2026-09-03 — `menus.gd`'s `_cut_selection`/`_copy_selection`/`_paste_clipboard`/`_select_all`, a GDScript-side session-lived buffer holding the engine's own `icon_get()`/`label_get()` records (survives a world change by re-creating entities, not by referencing stale indices), a 4-grid-cell paste offset clamped into bounds, and the "scoped to the active layer" dispatch — all landed together, including the probe. The dispatched agent found this before writing new code and instead closed the real gap: the probe's icon leg had never run (misread "no pack ships" as "no pack reachable"). Extended with 25 assertions covering icon copy/cut/paste; independently re-run, `PASS`. Icons and labels only, by the shipped code's own stated scope — sculpt stamps and settlements are named out of scope (no read-back / no selection set). |
| **Landmark M8 residual** — 23 of 49 declared kinds still ship `buildable:false` (was 24, before border marker landed; was 29, before M7's viewshed unblocked 5 more — see M7's own row) | `LANDMARK_GENERATION_SCOPE.md` | large | **Twenty-six** generate today (`landmark.rs::kinds()`, each unbuilt kind carrying a `not_built:` reason). **2026-09-21: border marker landed** (commit `92cd8d0`), taking buildable 25 → 26 (blocked 24 → 23) — `LandmarkInputs::territory` wired from `assign_territory`'s real per-cell faction raster, hard-gated on the same viewshed primitive Fort/Watchtower use, verified with a dedicated negative test proving the claimed/unclaimed edge does NOT count as a border. **Earlier the same day: M7's viewshed unblocked 5** — `fort`, `watchtower`, `fortified_pass`, `fortified_crossing`, `volcanic_feature` — taking buildable 20 → 25 (blocked 29 → 24). Of the six `needs_viewshed` kinds, only `sacred_mountain` now stays blocked on a real remaining gap (no cultural-meaning input, `LANDMARK_GENERATION_SCOPE.md` §26 forbids hardcoding one). **Counting artifact found and corrected 2026-09-21** (before M7 landed): `grep -c "KindSpec {"` and `grep -c "buildable: false"` each over-count by one — the former also matches `pub struct LandmarkKindSpec {`, the latter also matches the field's own doc comment. Real denominator confirmed by direct read: **49** declared kinds. **All 29 then-blocked kinds independently re-audited 2026-09-21, at their cited symbol, for the exact defect this session found repeatedly the same day (a reason naming a missing capability that had since shipped): zero found stale beyond what M7 itself closed.** Two genuine near-misses investigated and refuted: `route_sediment`'s delta-building (gated behind the optional interactive Erode op, no queryable per-cell record) for `delta`; the cirque classifier (a deliberate 2026-09-01 judgment, re-confirmed still true) for `glacial_feature`. **2026-09-02: the five way-graph kinds landed** — `market_site`, `trade_depot`, `caravan_station`, `bridge_site`, `road_junction` — taking buildable 15 → 20. `resource_extraction_site` went buildable 2026-09-01. Several still need §13's route load or `STORY_PLANNING_SCOPE.md` SP-4 (both confirmed still open 2026-09-21); the remainder of the military family is downstream of Fort, now built. **Stale doc comments found in passing, not yet fixed**: `landmark.rs`'s own top-of-file comment and the `not_built` field's doc both cite older buildable/blocked counts predating M7 — filed as its own small row below. |
| ~~**`landmark.rs`'s own doc comments cite stale buildable/blocked counts**~~ — **CLOSED 2026-09-21 (verified)** | `cartalith-civ/src/landmark.rs` | small | Fixed directly by the coordinating session, commit `c7710c5`: the top-of-file comment now states the real 26-buildable breakdown and names `sacred_mountain` as the one §9.3 viewshed kind still blocked (on §26's cultural-meaning gap, not the viewshed); the `not_built` field's own history-of-counts comment gained its two most recent links (24 after M7, 23 after border marker) rather than being overwritten. Doc comments only, `cargo check -p cartalith-civ` clean. |
| ~~**Zoom-dependent label density**~~ — **CLOSED 2026-09-21 (verified)** | `map_overlay.gd`, `cartalith-civ/src/labels.rs` | medium | **Landed exactly as scoped, commit `d213a41`.** `LABEL_CLASS_LOD` (per-class minimum raw-zoom threshold, modeled on `SETTLEMENT_LOD`'s own OSM-Carto-precedented bands) plus `LABEL_WEIGHT_LOD_SPREAD` (a log-scale within-class importance bias off `MapLabel::weight`) gate `_draw_labels()` and `_seed_label_occupancy()` together, mirroring `_settlement_hidden()`'s existing contract. `weight` needed threading through the Rust bridge (`labels.rs`, `lib.rs`, `project_bridge.rs`) — it existed only inside `label_candidates()`'s one-time generation-time sort and never reached the renderer. **Real finding that reshaped the design**: 158 of 186 generated labels on the default world duplicated the settlement pin's own already-correctly-gated inline name at the same point — giving the generic layer's settlement class its own LOD schedule risked contradicting the pin's tier-accurate one, so that class is gated always-hidden there and the pin stays the single source of truth for settlement names. No golden moved. **Independently re-verified**: `cargo test -p cartalith-civ --lib labels` (81/0/1) and the full `cartalith-godot` suite (874/0/19) re-run myself after a `git add -p` split of `lib.rs` from the concurrent river-ink fix's own hunks in the same file; before/after screenshots at fit-to-window and 4x zoom confirm the same dramatic decluttering the agent reported, and the 4x shot confirms this compounds cleanly with the river-ink fix on the same lake. `project.godot` and the live settings file confirmed unchanged. |

---

## 2. Committed and scheduled, not started

Nothing blocks these. They are ordered largest-first within each group, and the
groups are ordered by how much of the remaining project they represent.

### 2.1 Urban morphology — what remains

Phase 5. Milestones 8-15 are **built and committed** in `4ec07f5`; **milestone 16
shipped in `cff1edc`** and milestone 17's five `_um*` are all built and golden-
covered (both verified 2026-09-03, batch 18 — see §3.2's closure note). **Every
urban milestone is now built, and the last delivery gap closed 2026-09-03**
(batch 19): `urban_layouts` now calls `settlement_layout_with`, so a per-settlement
wall/age override reaches the layout. Proven by delivery, not plumbing — an
independent probe measured `umWalls=off` taking the wall ring from 41 points to
absent, edges 1 060 → 1 336 and parcels 3 296 → 3 850, with `auto` restoring a
byte-identical signature. **It was empty until 2026-09-12**, when the owner’s Rulings H-K (`LARGE_ITEM_RULINGS.md`) opened the rows below.

| Item | Owns it | Size | Next step |
|---|---|---|---|
| ~~**Draw what the town model already generates: trees, gardens, orchards and wells; towers on the wall; inside versus outside the wall**~~ — **CLOSED 2026-09-23 (verified), all three parts done** | `URBAN_MORPHOLOGY_SCOPE.md` | medium | **(b)** towers: already shipped 2026-09-13, untouched. **(a)** clutter: `urban_adapter.rs::run_layout` no longer filters `Town::details` to field/pasture — it splits farmland from everything else, so any future detail kind crosses the bridge automatically rather than needing a hand-list. Per `RC_ENGINE_CHANGES.md`'s own rule ("assert every detail kind the generator can emit is reachable, derived from a real town"), swept 10 real walled towns plus specialisation/wall-state variants rather than guessing: well 877, cross 40, bollard 62, tree 30 594, fence 8 118, crane 21, dryingrack 42, logboom 17, spoilheap 73 — every observed kind now has a draw branch in `urban_layout_draw.gd`. Only `waterway` never appears live (`run_layout` always passes `culture: None`, medieval never sets it) — disclosed, not fixed. **(c)** intramural tint, done for real this time: a per-building footprint-centroid-in-ring test (`cartalith_urban::geom::point_in_poly`, the crate's own existing primitive, not a new one), independent of district — measured 0 disagreements against Godot's own independent `Geometry2D.is_point_in_polygon` on every non-straddling building, against the old district-proxy's 906–2 097-wrong-per-town failure rate the 2026-09-13 attempt measured. `building_intramural: Option<Vec<bool>>`, absent (not all-false) on an unwalled town. **Verified independently**: full diff read; `cargo test -p cartalith-civ --lib`/`-p cartalith-godot --lib` reproduced 734/0/1 and 633/0/8 exactly (full-workspace run was blocked by an unrelated concurrent agent's in-progress `blocks.rs` edit at verification time — scoped to the two crates this batch actually touches instead); both new tests re-run in isolation; real windowed before/after screenshots confirm wells/crosses/trees rendering and the roof-hue split; `_towertint_probe.gd` re-run independently, ALL PASS, hue values matching exactly (0.12 intramural, 0.058 extramural unchanged); `godot --headless --check-only` clean; `project.godot` unchanged. Commit `362cde9`. |
| ~~**A settlement marked Fortified can generate a star fort that is drawn as a stone curtain around its gorge — the bridge never passes the fort trace**~~ — **CLOSED 2026-09-22 (verified) — GUI-sequencing gate lifted by the owner** | `URBAN_MORPHOLOGY_SCOPE.md` | small | **The "sequenced after the GUI rows, tablet first" gate (owner, 2026-09-12) was explicitly lifted 2026-09-22** now that the Settlement Editor GUI rebuild fully closed 2026-09-21. `urban_bridge.rs::layout_dict` now emits `fort_trace`/`fort_ravelins`, gated exactly as the reference's own guard (`style==='bastioned'&&fort&&fort.trace.length>2`), absent (not empty) otherwise. New `urban_layout_draw.gd::_draw_bastioned_wall` ports `_umDrawLayout`'s bastioned branch verbatim (v2.11 ~23348-23350): strokes the trace at `max(1.6,5.5*mScale)` then each ravelin at `max(1.0,3*mScale)`, no gate markers, matching the reference exactly. `_draw_wall` now branches to it before the curtain/gate code runs, so a bastioned town no longer draws the gorge as a stone curtain with gate dots. The stale *"settlements carry no traits"* comment is corrected. **Verified independently**: full diff read (additive only, nested strictly inside `style == "bastioned"`, structurally unreachable for any other style); a live dict diff shows exactly 2 new keys added on toggle, every other key and every non-bastioned dict byte-identical; a windowed pixel probe with a positive control (confirms real ink is found before trusting a miss) shows the before/after PNGs — a plain rounded curtain wall becomes a real angular bastioned trace with triangular ravelins, viewed directly, not just asserted; `cargo test --workspace --no-fail-fast` reproduced independently, 0 failures on every run (a ±1 passed-count flake isolated to the untouched `cartalith_engine` crate, confirmed stable across 3 isolated re-runs, both by the agent and independently by the coordinating session). Commit `c6dc8bf`. |
| ~~**A third culture profile, shaped on the owner's town plan**~~ — **CLOSED 2026-09-22 (verified) — shipped as a Rules preset, not a CultureProfile** | `URBAN_MORPHOLOGY_SCOPE.md` | medium | **Scope corrected once the code was read**: every knob the plan needs lives on `Rules` (`StreetRules`/`ParcelRules`/`SettlementRules`), not `CultureProfile`, and a new profile id would silently lose the tiltyard (`amenities.rs::games_spec` matches `"medieval"` by name) while nothing in the app chooses a culture at all (`urban_adapter::run_layout` hard-codes `None`). Shipped as a new named Rules preset, `MARKET_TOWN_RULES` ("market_town"), selectable from the Generation rules window — the one place a preset is actually live-selectable today. Seven fields changed from `DEFAULT_RULES` (wider parallel street spacing 24→50m is the load-bearing one, plus longer segments, fewer pierces, gentler branch jitter, less plot-depth variance, a lower subdivision cap, one wall generation), each reasoned against `design/owner-references-2026-09-12/urban-town-plan-walled-market-town.jpg`. **Delivers the row's stated goal** (legible blocks instead of a shattered lane mesh, ~40–50% fewer blocks at pop 7 000/19 000 across 12 seeds × 3 sizes) but honestly does not reproduce the plan's radial arterials, suburbs, citadel or fields — that's the separate, larger "change the algorithm" row. Byte-identical on every existing golden (additive, no existing call path changed). **Verified independently**: `cargo test --workspace --no-fail-fast` reproduced 3507/0/36 exactly; the new market-town tests re-run in isolation; a windowed probe re-run independently reproduced the exact claimed edge counts (1121→821); real generated-town screenshots compared directly against the reference image confirm the block-grain match; `godot --headless --check-only` clean; `project.godot` unchanged. Commit `d287922`. **A real, reproducible hang bug in existing shipped code was found as a side effect — see the new row below, not fixed here.** |
| ~~**`build_parcels`'s frontage-width loop can hang**~~ — **CLOSED 2026-09-23 (verified)** | `cartalith-urban/src/blocks.rs`; `RC_ENGINE_CHANGES.md` §6r.5 | medium | **Fixed by porting the reference's own `PARCEL_GRANT_MAX_SPIN = 4096`** (§6r.5's cited value): counts consecutive frontage-grant passes that place nothing, resets on any real placement, stops granting on that edge once hit — the existing post-loop stretch widens already-placed grants to fill it, no new fallback invented. **Bound chosen by measurement**: across 60 real towns, the worst unbounded run was 159 367 spins (reference's own figure: 172 644); 16 384 and 65 536 also terminate every case at the same cost, so the reference's own 4096 was kept. The 21 of 60 towns that never reach the cap are byte-identical under it (hash, count, every parcel's geometry) — proving the bound doesn't touch normal generation. **Before/after timing**: the previously-hanging cases (pop 7 000 at variance 0.15, ~43s; the shipped "Planned Grid" preset's pop 4 000 at variance 0.10, >100s on every seed tested) now both finish in ~100ms. **Golden re-baseline, pre-authorized**: 20 of 29 `generate/golden.rs` cases move (67 of 31 845 parcels by count, 0.21%, close to the reference's own 0.51% for its equivalent fix) — only parcel-derived fields, nothing about streets/blocks/plaza/walls/forts/markets/bridges; 9 cases including all 6 Venus plans are byte-identical, the control. `districts/golden.rs` (calls `build_parcels` directly, needed the same re-baseline though outside the row's named scope): 22 of 36 scenarios move, only churchyard parcel IDs renumber — the worship site itself is bit-identical on every one. A new test hangs (not fails) if the cap is ever removed, a deliberate choice so a regression is loud. **Verified independently**: full diff read; `cargo test --workspace --no-fail-fast` reproduced 3510/0/36 exactly; golden diffs confirmed at exactly 20 hash changes (generate) and 22 parcel_count changes (districts), matching the disclosed counts precisely. Commit `86ccf73`. |
| **Change the ported urban algorithm toward the owner’s town plan — golden re-baseline authorised** | `URBAN_MORPHOLOGY_SCOPE.md` | large | **Ruling H, 2026-09-12**: the owner selected *"Change the ported algorithm"*, offered as departing from the reference and needing a golden re-baseline. **Scoped to `cartalith-urban` only.** **Do this only for what the profile row above cannot reach.** Candidates from the plan: legible wedge blocks at size — radial arterials cut by a few curving cross-streets, not a lane mesh; suburbs strung along the roads outside the gates, thinning with distance; a gap kept against the wall where there is no gate; and perimeter lots with open courtyards held at high density. **Each change is a deliberate departure and is recorded as one** — what changed, why, which golden moved — with the golden re-derived rather than a tolerance widened (`cartalith-porting-discipline`). The orchestration golden is `generate/tests/golden.rs`, reproduced byte-identically from the frozen reference on 2026-09-03 (`9e79e52`, 29 cases). Target: `design/owner-references-2026-09-12/urban-town-plan-walled-market-town.jpg`. **Sequenced after the GUI rows, tablet first** (owner, 2026-09-12) |
| **A citadel enclosure straddling the town wall** | `URBAN_MORPHOLOGY_SCOPE.md` | large | **Ruling I, 2026-09-12 — new engine work.** The plan’s citadel sits astride the curtain on the north-east: its own walled enclosure with towers, one large building and an open court, reached from inside the town. **Nothing like it exists**: `fortify.rs` builds the curtain, gates, spurs and the star fort, and the only castle in the crate is the *"Castle keep"* civic **building** style in `amenities`. **Settle the design first**: which settlements get one (size, class, a faction seat, high ground from the site model’s relief), and whether its area counts inside the circuit for growth. It owns a golden once built, under Ruling H’s re-baseline scope. Target: `design/owner-references-2026-09-12/urban-town-plan-walled-market-town.jpg`. **Sequenced after the GUI rows, tablet first** (owner, 2026-09-12) |
| **A menu to set a settlement’s city type and regenerate that one settlement** | `URBAN_MORPHOLOGY_SCOPE.md` | large | **Ruling J, 2026-09-12, in the owner’s words:** *"Let’s create a menu to modify city types and be able to regenerate a specific settlement at will."* **Verified absent:** `city_viewer_window.gd` carries no culture, city-type or regenerate control. **The plumbing is half there**: `urban_layouts` already calls `settlement_layout_with`, so a per-settlement **wall and age** override reaches the layout — proven by delivery on 2026-09-03, when `umWalls=off` removed the ring and moved the parcel count. A culture/profile override and a single-settlement regenerate are the missing halves. **No canvas draws this control**, so it needs a design from the DCC canvases’ own vocabulary (owner ruling, 2026-08-25) before a build, and as GUI work it falls under the standing definition of done. **It is also the tool for judging Rulings H and I**, which argues for building it before the algorithm work it would be used to evaluate |

**The section's earlier row closed as *wrong* rather than merely stale.** The 17a caveat — golden-verify the block-2 `_um*`
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

### 2.3 Civilisation, economy and journeys

| Item | Owns it | Size | Next step |
|---|---|---|---|
| **The raw ecological ratio tracks world SIZE, not only ecology** | `MILITARY_MANPOWER_SCOPE.md` §3.3 | medium | **Disclosed by the ruling-11 lane rather than tuned around, and it is the honest limit of that ruling.** Measured at a fixed faction count: median raw `land_capacity / total_pop` is **0.39 on a 512×384 / 800 km world against 5.04 on a 768×576 / 2 000 km one**. So part of the upper tail the new 4.0 ceiling now admits is **map scale, not fertility**. Normalising it means changing how `land_capacity` or `nucleated_pop` are computed, which is a different question and a larger one — raising the ceiling was the right fix for the symptom ruled on, and this is what it does not reach |
| ~~**Consolidate landmarks into `cartalith-civ`** (ruling 24)~~ — **CLOSED 2026-09-23 (owner ruling: ratify the existing split)** | `ARCHITECTURE.md` | medium | **Owner ruled 2026-09-06 to consolidate, against the original recommendation to ratify the existing split — that ruling was made on a summary that was wrong.** Measured 2026-09-06: the "landmark logic split across two crates" claim was false. The terrain half, `cartalith-terrain/src/analysis.rs`, is general terrain analysis (`local_relief`, `tpi_multiscale`, `slope`, `normalise`) that landmarks *consume*, not landmark-only — `cartalith-godot/src/sample_bridge.rs:154` imports it as user-facing analysis fields in their own right. Usage outside its own crate: 18 in `landmark.rs`, 7 in `landmark_timing.rs`, 4 in `sample_bridge.rs`, 1 in `cartalith-civ/src/lib.rs`. Consolidating would have moved a terrain primitive into the civilisation crate and inverted the dependency the split exists to keep. **Owner ruling 2026-09-23: ratify the existing split as correct** — no refactor. Documentation-only closure, no code change. |
| **Un-shelve the 16K/32K export** (ruling 15) — **THE LADDER SHIPS; E1, E2 AND E3 OF THE BANDED WRITER NOW TOO** | `EXPORT_SCOPE.md` | medium | **Reduced from large 2026-09-06: the path already survived both new sizes, so most of what this row assumed was work turned out to be measurement.** `BAKE_WIDTHS` is now `[2048, 4096, 8192, 16384, 32768]`, and **both new rungs were run end to end** — 16K at 80.4 MB / 15.7 s / 4 169 MB peak, 32K at 213.9 MB / 69.2 s / 15 349 MB peak, reproduced independently by the verifier. `refuse_unaffordable` gates every width against `OS.get_memory_info()` before allocating. **E1 (the banded terrain renderer) CLOSED 2026-09-22 (verified)**: `ExportBandPlan`/`ExportBand`/`bake_export_band` rebuild §4's exact proven-and-reverted design — width-independent band planning (a budget covering the whole raster yields exactly one band with a zero apron, so the shipped 2K/4K/8K path is untouched, still monolithic), the `apply_local_contrast`/`build_grade_influence` band splits with both of §4.2's named traps (band-local `border_cover`, band-local radius) avoided. All five of §4.3's required tests pass with real measured numbers — zero differing bytes at every band size tested including a wider-radius case, no boundary step, non-vacuous negative controls (forcing the apron to 0 diverges by 42 120 bytes, proving the apron is what makes the identity hold), exact plan arithmetic 2K-32K. Four mutation tests each caught a real defect. One honest caveat disclosed: §4.2's f64-exactness proof is measured exact at test size but not strictly proven at 32K's extreme window (~55 bits needed against f64's 53) — a byte would only flip on a coincident u8-rounding edge. Verified independently: `ExportBandPlan`/`bake_export_band` confirmed present at the symbol, `cargo test --workspace --no-fail-fast` re-run (164/3485/0/34, matching the agent's own math exactly against the concurrently-landed QuadTree retirement). Commit `de3c95e`. **E2 (the streaming writer) CLOSED 2026-09-23 (verified)**: new `export_stream.rs` pulls each band from `bake_export_band` lazily, in order, and writes it through a `BufWriter` before dropping it — the finished raster is never held whole. PNG and BigTIFF both land. **Measured**: 32K PNG peaks at ~1.33 GB against the monolithic path's ~15.2–15.3 GB, tracking band height at ~21.9 bytes/rendered-band-pixel. **Round-trip verified exactly to the milestone's own stated bar** — decoded with a reader sharing no code with either encoder, every byte, several band geometries including a short final band; cross-checked externally against Pillow/libtiff at the full 32768×20976 scale, 0 differing pixels. **Two real findings, not guessed**: (1) `EXPORT_SCOPE.md` §6.1's own documented BigTIFF recipe (`new_big()`→`rows_per_strip()`→`write_strip()`) writes a **corrupt file** in `tiff` 0.11.3 — only the whole-image path actually enables compression; worked around by applying the predictor and the crate's public `Deflate` compressor directly through `DirectoryEncoder`. §6.1 needs correcting separately, not done here. (2) the shipped PNG export has always used `Fast` compression, not `Balanced` as an earlier doc comment assumed — caught because a first attempt at `Balanced` came out 17% smaller than the shipped path; the streaming writer now matches `Fast` so `export_raster.rs`'s fitted size/timing model still describes it. **Not wired into the shipped export menu yet** (that's E3) — a new `#[func]` would be one line. `tiff` 0.11.3 added as a new dependency, confirmed MIT-licensed at the registry cache, `default-features=false` keeping only `deflate`; `Cargo.lock` gained exactly two packages (`tiff`, `quick-error`), confirming §6.2's predicted trimmed resolution. **Verified independently**: full diff read including the BigTIFF workaround itself; `tiff`'s MIT license confirmed directly; `cargo test --workspace --no-fail-fast` reproduced 3516/0/37 exactly, purely additive. Commit `fc7db2b`. **E3 (the options struct) CLOSED 2026-09-23 (verified)**: one `ExportOptions {width, format, style, content}` dictionary (`export_options.rs`) replaces the "fifteen `#[func]` parameters" §7 named — unknown keys/looks/ramps/tunables/formats, a look+preset together, a missing preset file, or a tier with settlements off are all refused, not ignored. **`style` layers over `appearance()` without mutating session state structurally, not just behaviorally**: `appearance_rebased(&self, …)` takes `&self`, so it cannot write to `WorldGen` at all — verified independently at the symbol. `content` is §5's explicit settlement tier; overlays beyond terrain+rivers need E4 (doesn't exist) and are refused outright, never faked. Two new `#[func]`s (`export_image`, `export_image_estimate`), not wired into any menu — that's E5. `export_raster_estimate` gains real `bands`/`band_rows`/`apron_rows`/`band_peak_bytes`/`band_affordable` figures, checked against hand-worked-out numbers in advance: 16K is 5 bands/2294 rows/apron 164, 32K is 33 bands/655 rows/apron 328, both peaking at exactly the existing 8K monolithic figure. **Divergent refusal policy flagged and RULED ON 2026-09-23**: on a platform reporting no free memory, the new `export_image` path allows 16K/32K (its real banded cost never exceeds the 8K monolithic figure) while `export_raster_png` still refuses anything above 8K there. **Owner: keep `export_raster_png` as-is** — its caution is correct, not a bug, because it is NOT banded and its real cost at 32K genuinely is ~15 GB; the coordinating session's own first framing of this question was imprecise (it proposed "relaxing" the old path to match, which would have let a real ~15 GB allocation proceed on a platform that couldn't report free memory — caught and corrected before any code changed). No code change needed; the two paths' different thresholds correctly reflect their different real costs. **Verified independently**: full diff read including confirming the `&self` non-mutation guarantee at the symbol; `cargo test --workspace --no-fail-fast` reproduced 3525/0/37 exactly, purely additive (9 new tests); no bug found in or contract changed for E1/E2. Commit `ce2c71d`. **E4 (the overlay session) SCOPED 2026-09-23, not built** — real measurement, not a guess, found genuine risks a naive port would have hit: (1) a 32768px-wide `SubViewport` returns a null image on this Godot version — a full-width 32K band cannot be one viewport, the overlay must tile on both axes (Android's limit is unmeasured and likely lower). (2) `SubViewport` readback is **premultiplied** alpha, not straight — compositing with the straight-alpha formula would darken every antialiased edge; measured exactly (`[128,128,128,128]` for a 50%-white rect). (3) the export's pixel→grid mapping is corner-aligned (`bake_rect`'s own `sx=(gw-1)/(W-1)`) while the live overlay's is texel-centred — a naive "control size W, zoom 1" camera would compress the overlay by ≈8px at 32K's edges; the real fix is a derived per-axis affine on a parent-node camera (not `SubViewport.canvas_transform`, which the urban-layout culler reads around). (4) symbol-size semantics are split three ways inside `map_overlay.gd` (pins scale with fitted map width, way/label widths are constant screen-px, glyph rasterisation caps at 256px) — genuinely gates the whole approach, filed as an owner question and answered 2026-09-23: **uniform magnification** (symbols stay proportionally sized on a giant export, like a real poster) — the bigger-lift option, needs new symbol-scale drawing code in `map_overlay.gd`, not the recommended v1 screen-pixel-constant shortcut. (5) state-copy is a silent-divergence risk — missing one setter makes the export differ from the screen with nothing failing; the plan requires deriving the full setter list from every real caller and proving parity by diffing the live and export overlays under an identical camera, not just eyeballing it. **Sequenced into 4 independently-verified batches** (Rust session core → one-tile registration → many-tiles/many-bands end to end → wiring+real-size measurement+docs), matching this session's own batch discipline for large, architecturally novel work. **Five more owner questions remain, gating batches B-D** (v1 overlay content scope, LOD rule for labels/ways/urban layouts, whether rivers draw the vector stroke or the baked ink when overlays are on, which settlements/filters apply, whether a 1-2s per-band UI freeze is acceptable for v1) — not yet asked. **Stale items found in passing, not fixed**: `STATUS.md`'s `EXP-E4` row still says "shelved/not started" (un-shelved 2026-09-06); `map_overlay.gd`'s header comment describes an obsolete scene relationship. **E4 Batch A landed 2026-09-23, commit `34db87f`** (Rust core, no GDScript wiring yet): `BandSink` (`export_stream.rs`, a writer thread over a `sync_channel(1)` feeding the existing `write_bands`); `ExportSnapshot`/`ExportSessionCore` (new `export_session.rs`, `Send+Sync` and Godot-free, matching `LodSnapshot`'s own shape, state machine `Idle → Open → Finished|Aborted`); `composite_premul_over` (the premultiplied-alpha formula a `SubViewport`'s `transparent_bg` readback needs, confirmed against a real `[128,128,128,128]` readback of a 50%-white rect); five new `#[func]`s (`export_session_begin/_submit_tile/_finish/_abort/_state`); `export_image`'s pre-render checks factored into a shared `prepare_export`, used by both it and `export_session_begin` (behaviour-preserving, not a new check). Session output proven immune to world regeneration mid-session (renders from its own snapshot) via `_exportsession_probe.gd/.tscn`. A concurrent-agent `git stash` swept this batch's uncommitted `lib.rs` hunks together with the river-colour batch's; the pop restored both, independently confirmed by symbol-level diff review, not just the agents' own reports. `cargo test --workspace --no-fail-fast`: 3542/0/37. **Still remaining: E4 batches B (one-tile registration), C (many-tiles/bands end-to-end) and D (wiring, measurement, docs) — no GDScript caller of the new session `#[func]`s exists yet — and E5 (the dialog)**. **The render-once decision was NOT reversed**; what depends on it is enumerated in §3 |
| ~~**Nothing tells the user a landmark result predates their icons**~~ — **CLOSED 2026-09-21 (verified)** | `LANDMARK_GENERATION_SCOPE.md` | small | `LandmarkStore` gains `icon_placed_since_run` + `mark_icon_committed()`, cleared by both `run()` and `invalidate()`, wired from the two real commit paths (`icon_place`, and `icon_brush_stamp` gated on `placed > 0`, never a drag sample that adds nothing). `stale_stages()` gains a `"landmarks"` key, same shape as the existing `civ_dirty` special case. The UI badge reuses the shell's own existing staleness vocabulary exactly (`civilization_workspace.gd`'s § Recompute block) rather than inventing new chrome — a plain-prose note above "Run landmark pass", polled by a `Timer` since the commit happens in a different workspace than the badge lives in. New windowed probe `_lmstale_probe.gd`/`.tscn` drives the real pipeline end to end; independently re-run by the coordinating session, `PASS`, matching the agent's report exactly. The engine-side flag (`landmark.rs`) was committed together with the concurrently-running M7 viewshed work, which shared that file — see `MISTAKES.md`'s new preflight row for the brief broken-HEAD window that caused. Commits `9eaba18` (UI/lib.rs/probe) + `222189f` (the `landmark.rs` field). |
| ~~**The "under 11 px" font sweep measures against a threshold no canvas states**~~ — **CLOSED 2026-09-22 (owner decision: working as designed)** | `design/mcp-2026-09-07/Cartalith Android.dc.html` | small | **Re-framed 2026-09-07 after checking where the 11 px came from, and the answer changes what this row is.** **The finding itself is sound:** a fixed `fs=10.0` label — *"no committed route selected"*, `journey_planner_view.gd:1509`/`:4467` — measures under 11 px on **15 of 23 screens at 1440×3168 and 19 of 23 at 1080×2400**. Both densities, because a fixed size scales with nothing. **But 11 px is not a design floor. It is a SURVEY threshold** from `GUI_GAP_REGISTER.md:7322`, a phone-composition audit counting small text. Nothing in any canvas or ruling states it, and the canvases themselves draw smaller than that everywhere (9.5/10/9/8.5 px, repeatedly). `FS_TINY := 10` is canvas-conformant. **Owner ruling 2026-09-22: drop it — the canvases are the standing definition of done and draw this small on purpose.** No device check needed, no code change made. |
| Story planning **SP-3** — the settlement timeline strip (simulated history + authored vault events + journey passes) | `STORY_PLANNING_SCOPE.md` | large | No per-settlement history accessor in `timeline.rs`; `civilization_workspace.gd:1633` is the world-level strip, not a per-settlement one |
| Story planning **SP-4** — the conflict overlay in CIVIL, reading real manpower figures | `STORY_PLANNING_SCOPE.md` | large | Blocks landmark M9. Its attachment model is undecided (§4) |
| ~~**CV-23**~~ — **CLOSED 2026-09-21 (found already done, not built this pass)** — historical territorial occupation over time | `STATUS.md` | large | **Fourth false alarm this session, and this one is on the coordinating session, not on a stale document.** The pre-dispatch check grepped `HISTORY_TERRITORY_PREFIX` — the *constant* — and found it unused; the real wiring runs through the **field** `history_territory`, a narrower absence proof than it looked like (folded into `MISTAKES.md`'s "report that a feature does not exist" row as a fresh instance). The whole engine-side capability shipped under **owner ruling 27** (`LARGE_ITEM_RULINGS.md`, "The timeline stores mutations, not snapshots"), commit `48a6b5d`: `civ_snapshot_save` builds a `TerritoryFrame::{Key,Delta}` per recorded year (`TERRITORY_KEYFRAME_INTERVAL = 64` bounding scrub cost, not compression), the archive writes/reads it at the reserved `history/territory/<year>.i32` slot exactly matching `SAVEFILE_COMPAT.md` §10.2, and `civ_territory_at(timeline, year)` is the queryable read path (returns `None`, not an empty raster, for a broken chain — deliberate). The dispatched agent found this before writing new engine code and instead closed the one real gap: existing test coverage built `TerritoryFrame` entries by hand, so no delta chain had ever crossed the archive boundary or been contrasted against live state. New test `a_recorded_year_comes_back_holding_what_was_true_then_not_what_is_live_now` (`project_bridge.rs`) records two distinct historical years plus a third, different live state, round-trips through save/load, and asserts each year reads back what was true *then* — a reader that just returned live state would pass every weaker shape of this test. Mutation-tested, both an off-by-one reader and a dropped writer call killed. Independently re-verified: `48a6b5d` confirmed an ancestor of HEAD, every cited symbol confirmed by direct read, new test re-run in isolation against the current tree — passes, crate builds clean alongside two concurrent lanes' in-progress edits. **One thing genuinely still open, disclosed in the code already** (`project_bridge.rs:1797`): ruling 27 moved the in-memory shape to deltas but the archive still writes one full raster per recorded year — deliberate, no format bump owed, out of this row's scope. Commit `184e419`. |
| ~~**VA-01** — the vault scan *index* (not the scan)~~ — **CLOSED 2026-09-21 (verified, already done)** | `STATUS.md` | medium | Re-opened at the symbol: `vault_store.gd:110-210` has a real, working backlink index — `INDEX_PATH := "user://markdown_vault_index.json"`, `vault_restore_backlink_index()`, `vault_backlink_index_json()` — matching `STATUS.md`'s own GGR-BULK line, which already lists VA-01 as done. The row itself was the stale party. |
| The `wantCounts` / user-fixed-tier-count branch of `_civIterativeAutoWorld` — **RESIZED small → medium/large 2026-09-22, real scope found before building** | `PHASE2_SCOPE.md` m8 | medium/large | **Owner authorized building this even without a design canvas; the agent read the real reference (`Cartalith Gen1 v2.11.html:25854-26373`) before writing code and stopped rather than invent a simplified version.** Real reference shape, confirmed at the source and cross-checked against `FUNCTION_INDEX_v2.11.md:192`: **five independent per-tier number inputs** (`civNCap`/`civNCity`/`civNTown`/`civNVil`/`civNHam`), not a checkbox+single field. `wantCounts` threads through five distinct points, not one parameter: `maxPlaces`, a `wantTotal`-dependent `suppR` formula (not a constant swap), `thresh`, a **stateful quota-consumption tier classifier** keyed on `capitalOf[rank]` plus quota-decrement order (replacing this port's existing rank-cascade classifier, not parameterizing it), and `if(wantCounts) break` inside the reference's iterative centrality→tier promote/demote loop. **A second, larger finding**: that promote/demote loop **does not exist in this port at all** — `lib.rs` calls `civ_hierarchical_network_topology` exactly once, unconditionally, so the current default path is a single-pass equivalent of the reference's *last* pass with no promote/demote ever having run. `PHASE2_SCOPE.md` m8's own "what shipped" list doesn't mention this loop, so it's a pre-existing gap, not something this task broke. **Not built** — this is a multi-function change spanning at least `cartalith-godot` and `cartalith-civ` plus new 5-field Godot UI, correctly out of scope for a small single-pass agent. Zero files touched (verified: no diff beyond a pre-existing unrelated `M` on `lib.rs`). Next step, if scheduled: decide whether to also build the never-shipped promote/demote loop first (the non-wantCounts default path's own faithfulness depends on it) or scope wantCounts around it. |
| ~~**Measure tool's "Plan a journey" opened the planner blank, never mutated the measured path into a route**~~ — **CLOSED 2026-09-22 (verified), owner-reported the same session** | `right_dock.gd`, `journey_planner_view.gd`, `app.gd` | small | **Owner: "when using the measure tool there is an option to plan a journey. But this opens only the journey planner. It doesn't mutate the measure to a journey path. (it should do just that in that case!)"** Confirmed real: the button called `app.open_journey_planner()` directly, discarding whatever was on `GlobalTools.measure_points()`. Fix: `right_dock.gd::_on_plan_journey()` commits the measured chain as a real route through the same three calls the INFRA Route tool's own ✓ Commit button uses (`route_begin("mixed")` / `route_append_stop` per point / `route_commit()` — real least-cost Dijkstra, not an arbitrary-polyline route type; mode hardcoded to `"mixed"` matching that tool and the reference's own `_civCommitRoute`, which has no mode choice either), repaints the map (`set_manual_routes`, the same one-liner `_commit_route()` uses), then hands the committed index to a new `app.open_journey_planner_with_route()` → `journey_planner_view.gd::open_with_route()`, which runs `open()` unchanged then re-seeds `_route_index` and clears stage overrides/layovers/trim exactly like the existing route-picker callback does for a genuinely new selection. Falls back to the pre-fix blank open for <2 measured points or no generated world — unregressed. GDScript-only, no Rust touched. **Scope note disclosed by the building agent, not hidden**: the newly-committed route isn't pushed into `infrastructure_workspace.gd`'s own "Hand-drawn routes" panel list — that panel isn't reachable from `right_dock.gd` without a larger wiring change, and it rebuilds itself fresh on next show regardless, so nothing goes stale. Verified independently: full diff read, every called symbol (`GlobalTools.measure_points`, `bridge.route_begin`/`route_append_stop`/`route_commit`, `viewport.overlay.set_manual_routes`, `viewport.manual_routes`) confirmed real by grep rather than trusted from the report, `godot --headless --check-only` parses clean on all three touched files, `_measurejourney_probe.tscn` re-run headless directly by the coordinating session — 13/13 assertions pass (3-point measure commits exactly one new route, the planner's own `_route_index` names it, the solved route holds all 3 clicked stops in order, mode=mixed, and the plain Logistics/⇧J open path still commits nothing and opens normally). Commit `32c6c71`. |

### 2.4 Vault, project archive and save format

| Item | Owns it | Size | Next step |
|---|---|---|---|
| ~~**Vault browser: a folder tree + visual content preview**~~ — **CLOSED 2026-09-21 (verified), owner-requested the same day** | `MARKDOWN_VAULT_SCOPE.md` | medium | **Owner: "explore the filetree/vault and have a visual representation of the files' contents... I need to specifically know what file I need for what information."** Scoped as a lighter pass, not a Markdown renderer — reviewed and approved from a Design Artifact mockup before building. Replaces `open_browse()`'s flat dropdown with a real folder tree (Godot's `Tree` control — the first use of it anywhere in this shell; built client-side from `vault_list_files()`'s flat paths split on `/`, no new bridge call) and a structured preview (frontmatter chips, heading outline, a short excerpt with frontmatter stripped) with "Open to edit" reusing the existing hash-guarded raw editor verbatim. Phone folds to a segmented FILES/PREVIEW switcher matching `culture_profiles_window.gd`'s own pattern. Scoped entirely to the standalone browse entry point — the entity-scoped Attach flow is untouched and probe-confirmed unregressed. **A real, named gap disclosed rather than invented around**: backlinks/unlinked mentions aren't shown in the preview — `vault_entity_backlinks`/`vault_entity_mentions` are entity-keyed only, and the real path-keyed engine function (`cartalith_vault::backlinks::Backlinks::backlinks_to`) has no `#[func]` wrapper exposing it to GDScript yet. No new Rust needed otherwise. Verified independently: full diff read, all four bridge call names and `DccWidgets.chip()`'s inert-`Callable` safety confirmed at the symbol, `cargo test --workspace --no-fail-fast` reproduced exactly (163/3496/0/34, zero movement), `godot --headless --check-only` parses clean, `git status` confirms only the one file changed. Commit `ddaa0b1`. |
| ~~**Data ▸ vault menu: four rows folded to two, three-week-old landing bug fixed**~~ — **CLOSED 2026-09-22 (verified), owner-reported the same session** | `MARKDOWN_VAULT_SCOPE.md`; `menus.gd`, `vault_window.gd` | small | **Owner: "The whole markdown vault there seems to exist a couple of times."** Confirmed real: `_build_vault_rows` built four Data-menu rows about the vault — three ("Markdown vault ▸ Connect · Browse · Links", "Create a note from a template…", "Vault index…") all called the same unscoped `open_vault_overview()`, a defect the function's own doc comment had disclosed for three weeks ("the two lower rows land on the panel's top rather than on the section their tooltip names"); the fourth was the new Browse & edit row from the vault-browser row above. **Owner ruling: fold to two rows** — "Markdown vault…" and "Browse & edit a note…" — with the template-creation and index/backlinks destinations reached from inside the panel instead of promised by separate menu rows that never actually landed on them. Fix: `vault_window.gd::_build_overview()` now calls `_build_index()` (so the index is the first thing under Search regardless of entry point) then a new `_build_pick_entity()` (settlement-name search plus full province/continent lists, reusing `civilization_workspace.gd::_fill_knowledge()`'s own dictionary shape) — picking any entity re-enters `open_for()` scoped, which is what makes `_build_create()`/`_build_attach()` actually build. Nothing the two removed rows promised is gone; both are now genuinely reachable, closing the landing bug rather than just hiding it behind fewer menu rows. **Process note, disclosed rather than hidden**: the agent that built this hit the account's weekly rate limit mid-verification and, deviating from this session's established discipline, committed and pushed on its own before reporting — commit `7f000ef` reached the remote without the usual pre-commit independent check. **Verified after the fact by the coordinating session, not trusted from the commit message**: full diff read for all four touched files; `cargo test --workspace --no-fail-fast` re-run independently, reproduced exactly (163/3496/0/34, zero movement); all three touched `.gd` files parse clean; `_v3menu_probe.gd` re-run independently (not just cited) — confirms "Data ▸ exactly two vault rows", "Markdown vault panel opens on Index and the entity picker, unscoped", and "Picking 'Sevjuniana Province'... landed on its own Create-from-template and Attach sections", `V3 RESULT PASS (0 failures)`. |
| ~~**"Browse & edit a note…" did nothing when clicked — an unbounded range check ate its id**~~ — **CLOSED 2026-09-22 (verified), owner-reported the same session** | `menus.gd` | small | **Owner: "The markdown reader/editor doesn't seem to be finished. There is the option but it doesn't open."** Root-caused precisely, not guessed: `_data()`'s `id_pressed` handler opened with `if id >= ID_DATA_ROUTE_FIRST (400): ... return` — an open-ended check claiming every id ≥ 400 as a Data-manager route regardless of whether it was one. `ID_VAULT_BROWSE = 716` fell into that branch as "route 316 of 15," failed the size check, and returned before ever reaching the `match` statement with its own real `ID_VAULT_BROWSE` arm — **the row had done nothing since it shipped.** Confirmed with a positive control on pre-fix code via both a manual `id_pressed.emit()` and `PopupMenu`'s own real `activate_item_by_event()` engine path (ruling out a probe-methodology artifact); `ID_VAULT` (45, below 400) was unaffected, which is why the vault-menu-fold's own verification probe never caught this — it checked the row exists with the right tooltip, never that clicking it actually did anything. Fix: bound the check on both ends (`i >= 0 and i < size()`) rather than renumbering 716, so any future Data row id ≥ 400 stays safe automatically; corrected the false doc comment that claimed the route run was "allocated above every other id in this file." Verified independently: full diff read, windowed probe re-run personally — all four activation cases pass (both rows, both activation methods) plus both route-boundary rows (400, 414) still correct, `godot --headless --check-only` parses clean, `git status` confirms this was the only file this batch touched. Commit `a7c3a84`. |
| Project archive remainder — the `library/` art binary payload | `STATUS.md`, `SAVEFILE_COMPAT.md` §16.5 | medium | **Re-opened at the symbol 2026-09-23 — the row's own "nothing draws any of it" and "`drafts/`/`library/` slots" framing is STALE and corrected here, not assumed.** `drafts/` (`SAVEFILE_COMPAT.md` §16.3) has been **written and restored since 2026-09-03** — closed in substance, wrongly still named here as open. `library/`'s two JSON documents (`assets.json`, `travel.json`) are **also already written and restored** (§12) — the earlier two corrections below (foreign-entry preservation, `preview.png`) already established this pattern of stale framing in this row. **What is genuinely still open, per §16.5's own text**: `library/assets.json` carries each item's image *index*, name and transform, but **the pixels those indices address are not in the archive** — embedding them is explicitly named as "its own design question", not yet answered (inline in the zip vs. an external reference vs. another scheme), so a restored library comes back as slot definitions with zero items. **This is a design-decision gate, not a pure build task** — needs the storage-format question settled (likely an owner call, given the archive-format precedent this project treats as decision-worthy) before scheduling. The travel library has no binary payload and is already stored in full. *(Two earlier stale claims this row made, corrected 2026-09-21 and retained for the record: (1) foreign-entry preservation was never actually missing — `cartalith-io::project.rs`'s `ProjectData::foreign`/`ProjectWrite::foreign` carry an unrecognised entry byte-for-byte, verbatim, always winning over a slot a later build registers, threaded through the Godot bridge and surfaced to GDScript as `foreign_entries`. (2) the `preview.png` producer half closed 2026-09-21 — `DccApp._capture_preview_png()` captures and PNG-encodes the map viewport at save time, commit `cf1eeec`, verified independently at the time.)* |
| ~~**Markdown Vault reader: raw-text preview and edit, browsable without attaching**~~ — **CLOSED 2026-09-21 (verified), owner-requested the same day** | `MARKDOWN_VAULT_SCOPE.md` | medium | The file browser (Attach a note) only showed structured frontmatter/fields, never raw prose, never editing — owner wanted both, plus availability without first attaching to an entity. Commit `7749763`. `VaultSession::read_for_edit`/`write_file` (the same `Error::SourceChanged` hash-guard `write_section`/`write_block` already use); `vault_window.gd` factored into three shared helpers (file picker, frontmatter/fields reader, raw editor) reused by both Attach and a new standalone "Browse & edit a note…" window (`VaultWindow`'s own `_browse_only` mode, not a second scene). Per the owner's own ruling, editing updates just the `.md` file — frontmatter/machine blocks re-parse fresh on next read, no dual-write. **Independently re-verified**: `cargo test -p cartalith-vault --lib` (92/0, including the new round-trip/refusal test), full workspace suite (163/3479/0/34, matching the concurrent GPU batch's own figure for the combined tree); parse-checked all five touched `.gd` files; viewed the standalone window's own screenshot directly. `project.godot` and the live settings file confirmed unchanged. |
| ~~**Story planning SP-1** — the `Journey` entity proper~~ — **CLOSED 2026-09-21 (verified)** | `STORY_PLANNING_SCOPE.md` | medium | Commit `750fe79`. `cartalith_civ::travel_library::{Journey, JourneyRoute}`: id, name, `PartyPreset` reference, a route **geometry snapshot** (not an index — routes have no stable id, confirmed by measurement), `start_year` from the existing `civ.year` cursor. Persistence moved to `ENGINE_OWNED_SLOTS` (`entities/journeys.json`, matching `SAVEFILE_COMPAT.md` §9.6's shape, specified 2026-08-31 before anything conformed to it). Travel Library's hardcoded `0` is now a real `preset_usage_in_journeys()` count. GDScript bridged, not fully migrated, disclosed rather than silent: the Journey Planner's own richer session state (stage overrides, layovers, animal entries) stays local and doesn't round-trip through the archive — not part of SP-1's contract, and it never meaningfully survived a regenerate before either. Backward compatibility proven: a pre-SP-1 archive's old journeys shape is skipped, not fatal. **Independently re-verified**: `cargo test --workspace --no-fail-fast` reproduced 163/3476/0/33 exactly; confirmed the types and all three load-bearing tests (round-trip, backward-compat skip, usage positive/control) at their cited symbols; parse-checked all four touched `.gd` files myself; confirmed via `git status` this batch touched none of the concurrent river-overlay lane's files. `project.godot` and the live settings file confirmed unchanged. |

### 2.5 Rendering, terrain appearance and export-adjacent

| Item | Owns it | Size | Next step |
|---|---|---|---|
| ~~**Settlement pin rings balloon into a screen-filling blob at deep zoom, reported as "labels don't fade"**~~ — **CLOSED 2026-09-21 (verified), found on the same OnePlus 12 device test** | `godot-project/map_overlay.gd` | small | **Reproduced first, not assumed from the report's own framing.** `_pinlabelovr_probe.gd`'s true pixel-proximity check confirms the NAME label was never the defect (placed off the pin by construction, zero overlap at any zoom) — the pin's own outline/capital/faith rings and coastal-badge outline used bare local-unit stroke widths never multiplied by `sc`, unlike `radius` itself. Past the real deep-zoom ceiling (160–240x on an 800–1200 km world), `radius` correctly shrinks toward zero in local units (holding constant on screen via `_civ_zoom_k()`) while the stroke widths stayed fixed — so the stroke became the dominant shape, measured at z=240 as a ~600-screen-pixel blob in faction colour swallowing the pin and the settlement under it. Fix: `* sc` at all five call sites, matching every other quantity in this draw loop. Hover-state's own `radius += 1.5` bump is the same defect class, out of scope (doesn't apply on the touch-only device that reported this), disclosed not silently fixed. Verified independently: diff read in full, `godot --headless --check-only` parses clean, before/after screenshots viewed directly at z=64 (before: a massive blob swallowing the label; after: a small correct marker, label clean above it). Commit `1050cf0`. |
| **Investigated 2026-09-21, re-investigated 2026-09-23, not independently closed: LOD-tile zoom "sawtooth" reported on the same device** — **now only 1 of 18 configs exceeds the bar, and the real cause was misattributed** | `cartalith-hydrology/src/lib.rs`, `cartalith-godot/src/render.rs`, `cartalith-godot/src/export_raster.rs`, `godot-project/_lodsweep_probe.gd` | small | **Re-measured against current HEAD (`179a69d`) rather than trusted from the 2026-09-21 numbers, since the river rendering has changed twice more since then** (`d657091` promoted the vector overlay and retired the raster river bake for a generated world; `34db87f` today moved `RIVER_RDP_EPS_CELLS` 0.75→0.5 and replaced the river colour scheme). **Only 1 of 18 sweep-run medians now exceeds LOD-D2's `≤1.5` bar** (pan, seed 483920, grid 2048×1311, median 1.6682, worst frame 3.24) — down from 4 of 18. **The dominant cause is not the two fixes this row previously credited (`f8d8bcd`, `42754bc`)** — those patched a raster river-bake path that `d657091` made dead for every generated world (`export_raster.rs::screen_river_ink` returns `None` for `WorldSource::Generated`, confirmed at the symbol): rivers now draw only as a vector overlay, so a raster-bake fix from before that change can no longer be doing the work this row credited it with. **The remaining outlier's cause was tested, not theorized**: hiding the rivers overlay for that one config drops its median 1.6682→1.2303 (confirming rivers are a real partial contributor), but a full 18-config re-run with rivers hidden trades it for two *new* overflows (1.2826→1.5136, 1.2719→1.5533) plus a max-ratio blow-up from 13.80 to 225.14 on a third — a pre-existing near-zero-denominator instability in `lod_sweep.rs::seam_ratio`'s neighbour-mean guard (guards `mean ≤ 0.0`, not "near zero"), only exposed once rivers stop supplying ambient contrast in some flat/coastal columns. **No code changed** — the diagnostic edit to `_lodsweep_probe.gd` was reverted, confirmed clean (`git status` empty on the investigation's full scope). **Correctly left open rather than built past**: hardening `seam_ratio`'s denominator guard is a metric-definition change on a mutation-tested (9/9 survivors), owner-relevant acceptance bar, not a harness-hygiene tweak — out of scope for a unilateral fix. Whether 1.67 at one config is within LOD-D2's "already-accepted tolerance" remains the same owner call this row has flagged since 2026-09-21, now narrowed to one isolated config rather than a systemic defect. `cargo test --workspace --no-fail-fast`: 3558/0/37 before and after (the investigation's own baseline briefly read 3542 before the concurrent tile-addressing batch's tests landed in the shared tree — both numbers independently reproduced, no movement attributable to this row). |
| ~~**Rivers render red-brown on LOD tiles instead of blue**~~ — **CLOSED 2026-09-21 (verified), found on a real OnePlus 12 device** | `cartalith-godot/src/render.rs` | small | **Owner-reported from a real APK build (commit `8e71a01`, this morning's LOD-D1–D6 batch).** Reproduced first (windowed probe, `_riverlod_probe.gd`), then root-caused precisely: `channel_tint()`'s formula (`(g*0.5+0.3).min(1.0)`, `(b*0.5+0.45).min(1.0)`) is written for Rgb normalised to `[0,1]` — its one pre-existing correct caller, `bake_rect` (exports/screen), converts to bytes only after calling it. `render_biome_tile_rgba` (LOD-D1's new tile colour pipeline) called `channel_tint` directly on its own byte-scale `[0,255]` triple — on that scale the `.min(1.0)` clamps collapsed green/blue to near-black on any bright pixel while red stayed near half value, pulling every inked river pixel toward dark red-brown. Fix: scale to `[0,1]` before the call, scale back after — `bake_rect`'s own round trip, reused not reinvented. One call site, ~20 lines. No golden re-baseline needed (the ink/`channel_tint` mechanism is port-only, no JS reference to diverge from); main map and exports were already correct and are untouched. Verified independently: diff read in full, `channel_tint`/`bake_rect`'s actual `[0,1]` contract confirmed at the symbol, before/after screenshots viewed directly (dark red-brown streaks → correct blue rivers/lake), `cargo test --workspace --no-fail-fast` reproduced the exact pre-existing baseline (163/3490/0/34, zero regressions). Commit `42754bc`. |
| The stage-by-stage `WorldParams`-field audit against every stage-01…11 slider | `GUI_FEATURE_PARITY_SCOPE.md` | large | The document's own closing "honest size statement": the Generate pipeline's ~60-80 individual stage sliders, "none of which are individually scoped anywhere yet". No such audit document exists |
| ~~**Promote the vector river overlay to the actual rendering; ditch the texture bake**~~ — **CLOSED 2026-09-22 (verified), owner-directed the same session** | `FUNCTIONAL_CONTRACT.md` cap. 6 | medium | **Owner, after seeing the still-baked, still-pixelated rivers live: "keep the smoothline and use that to render the river... ditch the texture bake," then "the line should get the same look as a lake, and its width scale with the zoom," then, after a screenshot, "make sure rivers don't become interrupted lines... neither tons of parallel rivers," then "a bit more catmull rom smoothing and the coloration should follow lakes."** Three layered passes landed as two commits (`d657091` river-rendering rework, `c6dc8bf`-adjacent probe fixes folded in): (1) `screen_river_ink` returns `None` for a generated world, so `build_color_texture`/LOD tiles no longer bake rivers at all; `_draw_rivers` is on by default, colored via `lake_color_at`, widened in real grid units from the existing `channel_disc` law. (2) `river_draw_plan` (`cartalith-hydrology`) hides near-duplicate parallel runs (heaviest by flow kept) and bridges land-pit dead-ends to a nearby drawn run within one D8 step — 510 real junctions meet at an exact 0.0-cell gap, 344→0 unbridged ends on the shell's default world; a second defect fixed alongside it (`half_width_cells` was reading a tributary's trunk-mouth cell, so every tributary drew at its trunk's width). (3) `river_render_polyline` cuts each run at pinned cells (traced mouths, bridge targets) and RDP-simplifies each piece before Catmull-Rom, smoothing corners while junctions still meet at exactly 0.0 cells; `get_rivers` gained a per-point `colors` array so tint varies along a river's length like a lake's surface. **A real root cause identified and explicitly NOT fixed here, queued separately (see below)**: the remaining fragmentation traces to this port's `compute_flow` having no depression-filled routing — `RC_ENGINE_CHANGES.md`'s own **highest-priority row**, §6g. **Disclosed, not fixed: raster exports (`export_raster_png`, `bake_export_band`, snapshots) still read the old `river_ink()` and show the pixelated bake** — the live view and an export now intentionally diverge until a separate export-time draw pass exists. Verified independently at every layer: full diffs read; `cargo test --workspace --no-fail-fast` reproduced 3494/0/34 exactly; real windowed before/after screenshots at matching crops confirm the staircase gone, the parallel band gone, fragments joined, and color varying along a river crossing a temperature gradient; `godot --headless --check-only` clean; `project.godot` unchanged. Commit `d657091`. **Amended 2026-09-23 — the owner revised both of this row's own decisions, superseding "the line should get the same look as a lake" and its `0.75`-cell tolerance.** Colour: `lake_color_at`'s lake-matched tint is replaced by a discrete Strahler-order palette (`RIVER_ORDER_RGB`/`river_order_color`, light headwater to dark trunk, sampled per render point since order rises along a main stem); `lake_color_at` itself and the single-swatch `color` field are removed (their only callers were this loop). `RIVER_RDP_EPS_CELLS`: `0.75` → `0.5`. The owner's first-requested literal, `0.3` (their "Rivers on a Grid" demo's own slider minimum), was measured and rejected before implementing it blind: on this port's D8 lattice a cell's offset from a chord is an integer over the chord length, and the smallest non-collinear offset is `1/√13 ≈ 0.277` — at `0.3`, measured turning angle went UP (7.00°/cell at 0.75 → 14.63°/cell), rivers reading MORE wiggly, not smoother; `0.5` sits above every staircase family up to the 1:2 case (`1/√5 ≈ 0.447`) and was the owner's own chosen middle ground once shown the measurement. Commit `34db87f`. `cargo test --workspace --no-fail-fast`: 3542/0/37, re-run twice identically. |
| ~~**Port depression-filled flow routing**~~ — **CLOSED 2026-09-22 (verified)** | `RC_ENGINE_CHANGES.md` §6g, §6k; `cartalith-hydrology/src/lib.rs`; `DECISIONS.md` §7o | large | **Not a literal JS port** — the reference's own fix (v2.41) postdates every HTML snapshot in this repository, so there was no source to diff against. Built the standard published algorithm the spec names instead: Barnes/Lehman/Mulla 2014 Priority-Flood+ε (`build_routing_surface`: a dual pit/open-queue flood fill from the map's real outlets, each raised cell tilted to `f32::next_up()` of its source so no flat ever forms), validated against the spec's own disclosed before/after numbers rather than built to reproduce them exactly — full reasoning in `DECISIONS.md` §7o. **Gated exactly like `crater.physical_model`**: off in `WorldParams::defaults()` (the golden baseline, ~30 suites, all captured against a reference with no fill), on at the app boundary (every world the shipped app generates uses it), off whenever a save doesn't carry the key. **Measured on four real worlds**: interior-pit land 58.6–77.8% → 0.0% on every one (spec: 68.5%→0.0%); sea-draining land ×1.21–×3.48 (spec ×1.63); longest main stem ×1.22–×2.83 (spec ×1.80); biggest mouth catchment ×9.3–×30.3 (spec ×18.3). Cost: ~104ms routing pass at 2048×1312; full generation 2.31s→2.61s. **Disclosed, not silently absorbed**: the heightfield itself moves under the flag (carving follows the routed network) — expected, matches the source's own v2.59 shape; under `world=true`, y-edge seeding sends far more land's drainage off the pole edges than before (0.7%→51.7% on one wrapped world), left for the owner if it matters in practice; `_riverconnect_probe.gd`'s check A now fails on exactly 1 of 1840 runs on the shell's test world — a single isolated case, not a systemic regression. **No golden moved** — every golden uses the off-by-default baseline; six `golden_parity_*` test files updated for one added argument each (confirmed at 1-2 line diffs, signature-only). 12/12 mutation tests killed. **Verified independently**: full diff read including the flood-fill implementation itself (the pit/open dual-queue structure and `next_up()` tilt match the published algorithm's shape exactly); `cargo test --workspace --no-fail-fast` reproduced 3507/0/36 exactly, zero movement; gating defaults confirmed at both symbols; `_riverconnect_probe.gd` re-run independently, reproducing the exact disclosed numbers. Commit `76f64bc`. |
| ~~**A settlement built onto the wall from the inside; a poor district against it from the outside**~~ — **CLOSED 2026-09-22 (verified), owner-directed the same session** | `URBAN_MORPHOLOGY_SCOPE.md` | medium | **Owner: "it's not uncommon for a settlement to be built unto the wall from the inside and have a poor district built up against the wall from the outside. Should add that in."** The reference has no matching concept under any plausible name — genuinely new work, authorized as a deliberate departure under the owner's standing **Ruling H** ("change the ported algorithm toward the owner's town plan, golden re-baseline authorised"). Measured before building: only 0.2–1.6% of a wall's inner face had a building against it, 0.0% outside. New `cartalith-urban::wallside::build_wall_lots` (curtain/palisade walls, organic-plan towns): intramural lots run from the wall's inner face to the nearest street within 52 m; 1–3 extramural **faubourg** runs per town follow the wall for 10–20% of its length from a land gate, small lean-to lots (~6.5 m frontage), exempted from the rampart-clearance sweep. Unconditional default behaviour, matching Ruling H's other direct algorithm changes. **Golden re-baseline**: exactly the 14 medieval curtain-walled cases of 29 moved, each re-derived from the crate's own output and disclosed old→new; 15 cases (8 bastioned, 4 Venus, 3 wall-less) correctly unmoved as the control. Head count rises 5–29% on moved cases, stated rather than netted off. **Verified independently**: full diff read; `cargo test --workspace --no-fail-fast` reproduced 3494/0/34 exactly; `golden.rs` diff confirmed exactly 14 hash lines changed on each side; real windowed screenshots (whole-town, wall-lots-stripped, faubourg close-up) confirm buildings along both wall faces; the windowed `_wallside_probe.tscn` re-run independently reproduced the agent's own per-town figures exactly (205 faubourg lots, 104 wall-backed, a 303/309 flip-test positive against a 0/4560 interior control). Commit `5d780dc`. **Refined 2026-09-23, owner: "The shanty shouldn't just be a small line against the wall it should be a cluster against the wall, much like the reference image."** The faubourg is now a wedge-shaped cluster (`FAUBOURG_ROWS` 3–5 rows deep at the gate end, thinning to one row along the wall; `LANE` gaps between rows, some built back to back; `ALLEY` cross-gaps), measured directly against the reference image's own quarter depth and raggedness rather than invented. Each cluster's first row draws from the exact original stream and comes out byte-identical to the single-row version, so the change is purely additive; a second stream draws the rows behind it. Same 14 curtain-walled cases moved again (control cases still untouched), re-derived and disclosed. A real defect found and fixed (rows at exactly the 6 m minimum depth failing the build check by a floating-point hair, `ROW_DEPTH_FLOOR` fixes it for new rows; three pre-existing first-row cases with the identical defect from `5d780dc` are disclosed, not fixed, since fixing them would move the first row this batch's own correctness property requires to stay exact). **Verified independently**: `cargo test --workspace --no-fail-fast` reproduced 3507/0/36 exactly; `golden.rs` diff confirmed exactly 14 hash lines changed; the extended `_wallside_probe.tscn` re-run independently — the pre-fix DLL fails all 4 new distance-band/lane checks, the new DLL passes all of them with lots spanning 5 distance bands and real lanes/back-to-back rows present; screenshots confirm a real multi-row wedge, honestly disclosed as not matching the reference's edge-to-edge block density. Commit `21c6949`. |
| ~~**The vector river overlay**~~ — **CLOSED 2026-09-21 (verified) — delivers the "flowing, smooth" bar** | `FUNCTIONAL_CONTRACT.md` cap. 6 | small | Commits `750fe79` (Rust half — see attribution note below) + `347db6e` (GDScript half). All three 2026-09-13 reversion reasons fixed, not just re-tried: debug-view gating via a new `MapOverlay.set_debug_active()` propagated from `ViewportHost.set_debug_layer()`; a real "Rivers (vector, smoothed)" `LIVE_LAYERS` row, default off, wired into both existing layer-toggle surfaces for free; a new `WorldGen::suppress_river_tint` bool read only inside `build_color_texture()`, deliberately not folded into `river_ink()` itself since that also feeds every PNG export path. Smoothing reuses existing golden-tested geometry (`way_render_geometry`/`civ_catmull_rom_sample`, already used for roads/sea-lanes) — no new spline code. No golden moved. **Attribution note**: the Rust half was swept into commit `750fe79` ("SP-1: the Journey entity") by a concurrent agent's `git add` on files it also legitimately touched — confirmed byte-for-byte present and correct there, nothing lost, purely a commit-message attribution gap; the real reasoning and verification for the Rust half lives in `347db6e`'s own commit message. **Independently re-verified**: full `cargo test --workspace --no-fail-fast` unchanged at 3476/0/33 after landing; parse-checked all three touched `.gd` files; re-ran `_verify_layers_probe.gd` myself, ALL PASS including the new "rivers" row (9/9); viewed all three screenshot pairs directly — (a) before shows the same blocky raster staircase, (b) after shows a visibly thinner, smoother curve with no doubling, (c) an active debug view shows zero river ink anywhere. `project.godot` and the live settings file confirmed unchanged. |
| ~~**Re-derive which pack sections the live map actually composites**~~ — **CLOSED 2026-09-21 (verified) — Ruling W built** | `FUNCTIONAL_CONTRACT.md` cap. 6 | small | Commit `ca77aea`. `unused.push()` in `manifest.rs` extended to all four remaining undrawn sections (`structures.settlement`, `structures.poi`, `custom`, `seamarks`), fixed order. The test that pinned the old behaviour (`settlement_and_poi_are_not_named_by_the_unused_warning`) inverted and renamed per its own doc comment's instruction that this is the test to change on a widening ruling. Two golden fixtures moved, both disclosed old→new (1 section → 4). **Independently re-verified**: `cargo test -p cartalith-assets` confirmed all five load-bearing tests present and passing by name; full `cargo test --workspace --no-fail-fast` reproduced 163/3469/0/33 exactly; read the `unused.push()` diff directly. `project.godot` confirmed unchanged. |
| ~~**Pack trait art is built end to end and gated on the owner's pack-warning ruling**~~ — **CLOSED 2026-09-23 (verified), owner authorized release** | `FUNCTIONAL_CONTRACT.md` cap. 6 | small | **Built 2026-09-04, released 2026-09-23.** `viewport_host.gd::refresh_settlement_traits()`'s one guarded line now installs the resolver (`overlay.set_trait_art_resolver(Callable(_bridge, "civ_trait_badge_row"))`); `engine_bridge.gd` gained the missing forwarder (this shell's rule: call sites address `bridge.<name>()`, never `bridge.world_gen.<name>()`). `manifest.rs` drops the `"trait"` arm from the unused-pack-sections warning — trait art now genuinely reaches the map, so a pack that only supplies it is no longer "unused". Four golden literals re-derived by actually running `parse_pack_manifest`/`read_pack` on the unchanged fixtures, never hand-edited. **Verified independently**: full diff read; `cargo test --workspace --no-fail-fast` reproduced 3507/0/36 exactly; a new windowed probe (`_traitresolver_wiring_probe.gd`, exercising the real path through `EngineBridge` rather than installing the resolver by hand on a raw `WorldGen` the way the two pre-existing trait-art probes do) confirms installing the resolver moves 115 real pixels in the badge window versus none installed; the pre-existing `_traitart_probe`/`_vfy_traitart_probe` were re-run, their only failures traced to stale multi-day-old baselines predating this change (not regressions), re-baselined and re-run clean; `godot --headless --check-only` clean. Commit `7d22e96`. |
| ~~**GeoJSON import**~~ — **CLOSED 2026-09-21 (verified) — Ruling V built, both halves** | `FUNCTIONAL_CONTRACT.md` DM-03 | medium | Commit `d79d776`. `geojson_apply::apply_geojson` walks a parsed document by `properties.layer`: settlement (through `civ_drop_place`'s own gates, using real imported name/pop/kind), territory (rasterised cell-by-cell via `point_in_ring`, even-odd with holes). `poi`/`way`/`river`/`province`/unlabelled features are counted and reported, not silently dropped, each for a disclosed reason. `FactionRoster::find_or_create_by_name`: exact match after trimming, never fuzzy/case-folded; blank/"Unclaimed" resolves to faction 0 and creates nothing. **The GDScript Data-manager Import route was fully built too, not left as a stub**: `data_manager_window.gd`'s Import ▸ GIS/GeoJSON row moved from a gap to live, wired through the shell's existing `DccBrowseDialog` (desktop + Android SAF). No golden moved — new code, never called from any generation path. **Independently re-verified**: `cargo test --workspace --no-fail-fast` reproduced 163/3467/0/33; confirmed all 5 load-bearing new tests actually appear and pass in the run's own log, not just claimed; read `find_or_create_by_name` directly and confirmed the exact-match claim; parse-checked both touched `.gd` files myself. `project.godot` and the live settings file confirmed unchanged. |
| ~~**Slippy-map tile addressing (XYZ/TMS/WMTS, a zoom ladder, retina variants)**~~ — **CLOSED 2026-09-23 (verified)** | `FUNCTIONAL_CONTRACT.md` cap. 6/9 | medium (confirmed right-sized) | **`FUNCTIONAL_CONTRACT.md` capabilities 6/9 tag this §7d "modernize" — no reference function to port.** The pyramid already existed (`cartalith_spatial::pyramid`, `2^z × 2^z` tiles, golden-verified), so addressing is genuinely a thin naming layer over it: an XYZ level `z` **is** atlas level `z`, nothing re-derives which tiles exist. New `cartalith-io::slippy` (pure functions: `TileScheme::parse` rejects rather than defaults on a typo; `tile_path` for XYZ `{z}/{x}/{y}`, TMS with the row flipped about the level, WMTS `{set}/{TileMatrix}/{TileRow}/{TileCol}`, panicking rather than wrapping on an out-of-level address; `zoom_ladder`; `slippy_manifest` writing TileJSON 3.0.0 for XYZ/TMS and a non-TileJSON-claiming WMTS variant, deliberately no `bounds`/`center`/CRS since the world has no georeference). New `cartalith-engine::slippy_export::export_slippy_tiles` — composition only, every tile is the bake's own `pyramid_tile` synthesis (never the atlas cache, since a cached PNG's sun/exaggeration isn't in the world key and could be stale), `rayon`-parallel within a level. New `#[func] slippy_export_tiles` wired into the Data manager's Export ▸ Maps pane (previously-disabled XYZ/TMS/WMTS segments enabled; "Tile grid" swaps to "Zoom range" in pyramid mode; a retina toggle; several canvas-drawn items correctly left disabled with their reason — see open questions). **Verified independently**: full diff read on both new Rust modules and the `lib.rs`/`engine_bridge.gd` wiring, all well-tested (16 new Rust tests covering the TMS-flip involution, WMTS row-before-column, retina-as-same-ground-at-2x, and a same-export-same-bytes determinism check) and following the shell's own `bridge.<name>()` wrapper convention; `cargo test --workspace --no-fail-fast` reproduced exactly (3542→3558, the +16 being these new tests, zero movement elsewhere). The agent's own windowed `_slippy_probe.tscn` run (40/0, real archive byte-identity across schemes at the same address, positive control, a real pane export writing 342 entries in ~1s) was not independently re-run by the coordinating session — a concurrently-running agent (the LOD seam-ratio investigation, same batch) held the only available windowed Godot instance throughout verification, and re-running would have meant contending for the same project/DLL lock; the Rust-level tests and full symbol-level diff review were judged sufficient given the specificity of the agent's own reported numbers. **Open questions disclosed, not guessed past**: no CRS is asserted (a real WMTS Capabilities document needs a `SupportedCRS` this planar world doesn't have — EPSG:3857/4326 stay disabled pending an owner decision on whether a generated world claims one); tiles preserve the world's aspect rather than forcing square (a 2:1 world gives 2:1 tiles); a non-square world's retina short edge can be ±1px off exact-double from independent per-density rounding (documented, probe allows it); canvas items beyond addressing itself (leaflet-preview.html, style.json, MBTiles/folder packaging, all-ocean-tile skipping, a marquee-scoped pyramid, reusing baked atlas tiles, depth beyond 6, the phone data-tiles screen) are deliberately not built, each disabled in the pane with its own reason; peak memory at depth 6 with retina (10 922 tiles, whole archive built in memory) was not measured. Commit `a3bca18`. |
| ~~**LOD-D0 · Zoom-sweep harness**~~ — **CLOSED 2026-09-21 (verified) — baseline recorded, changes how D2/D3 should be graded** | `LOD_DETAIL_SCOPE.md` | small | `cartalith-godot/src/lod_sweep.rs` (metric definitions, 21 unit tests, mutation-tested 9/9) + `_lodsweep_probe.gd/.tscn` (windowed harness driving the real `_zoom_at`/pan/`_update_lod` paths). Baseline over 6 worlds (3 seeds × 2 grids), 2724 judged frames: **0 pops** but worst level-boundary `T_i` up to **1.61×** (D3's ≤1.5× bar breached in 2/12 — pops alone don't discriminate on today's build, grade D3 on both criteria); seam ratio peaks **15.51** against a 1.5 bar; holes to 22 940 px/frame; detail-per-pixel at zoom 40 spans **7.2%–116%** of zoom-1 across worlds (D2's "≥50%" needs a fixed pinned view, not a world-dependent one — the scope's own "~8%" figure was one world's worst case); synthesis alone 3.4–8.8 ms/tile (well under D1's 40 ms bar); no 0.87s stall reproduces on desktop (that's the 6T's own row, unaffected). Aletsch/D4 finding: the view chooser lands on ground uniformly above the snowline in 4 of 6 worlds — D4 needs a view selection that straddles it, not the coldest-highest-cell heuristic. Two bugs the agent found and fixed in its own harness before trusting it: a degenerate "0.0% snow transition" reading for views with no snowline at all (corrected 4/6 readings); a seam-plant that shifted every tile in lockstep, creating no seam to detect. **Also fixed in this commit**: a broken intermediate HEAD (`mod lod_sweep;` landed in a concurrent commit without its file — this harness's own commit supplies it). Commit `11936cb`. |
| ~~**LOD-D1 · Port `renderBiomeTileRGBA` as a pure engine function**~~ (Ruling K, step 1a) — **CLOSED 2026-09-21 (verified) — byte-identical to the reference, two decisions owed** | `LOD_DETAIL_SCOPE.md` | large | `cartalith-godot/src/render.rs`, golden-tested against the real v2.11 engine run under Node (not guessed): **worst delta 0, mean 0.000000** on both golden cases — exact, not merely within the scope's 1e-4 tolerance. Two real bugs found and fixed: an index-underflow panic on a tile narrower than the meso step; two weak mutation survivors (`LAKE_MEMBERSHIP_MIN` structurally inert at 1 px/cell, a crest check with no oracle) closed with new coverage. Mutation 12/12. **Two decisions this row surfaces rather than resolves, neither blocking D2**: (1) the reference's own tile and its own map colour formulas disagree by construction (`shadeFactor2` divides by sample step, the tile's meso block doesn't) — ported literally per this port's own "the reference's errors are part of the contract" rule, measured gap 1.67 mean L* levels at LOD entry (inside D2's ≤2.0 bar); screen-matching instead is a two-line change if wanted. (2) single-thread tile synthesis measures **51.46 ms against the scope's 40 ms budget** — the budget didn't price in this port's own paper-texture/stipple/contrast tail (absent from the reference's tile); 16-core measures 5.92 ms. Three ways out named at the ignored timing test's own doc comment (gate the texture pass off for tiles, adopt the all-cores figure as the real budget, or let D6's off-main-thread synthesis absorb it) — **owner call, not resolved here.** Commit `f6d1bd5`. Unblocks D2 (*"the bridge returns RGB tiles, the shader draws them"*), deliberately not started in this pass. |
| ~~**LOD-D2 · Colour tiles on screen, and the sharpness bar**~~ (Ruling K, step 1b) — **CLOSED 2026-09-21 (verified) — 3 of 6 D0 acceptance bars pass, 3 disclosed failing** | `LOD_DETAIL_SCOPE.md` | medium | `lod_bridge::synthesize_tile_rgba` now calls LOD-D1's `render_biome_tile_rgba` directly (its old `(field, gw, gh, ...)` signature removed — no longer possible to amplify one world and colour another); `lod_tile.gdshader` samples `TEXTURE` directly instead of a scalar ratio. **A real caching architecture, not just "cache TileFields" as the scope's one line said**: a `RenderCtx` alone measured 201.5 ms to build at 2048×1311 (80× a single tile's own ~6 ms), so both it and `TileFields` became `Cow`-backed and cached on `WorldGen`, keyed on world/pack/paint epochs plus the appearance's serde fingerprint — measured hit at 9.9–14.6 ms/tile against ~480 ms uncached. Every golden (including D1's) passes unchanged; workspace 161/3371/0 (+2 tests over D1's baseline), re-verified independently. **Measured before/after on D0's own harness, honestly, not cherry-picked**: detail-per-pixel ≥50% passes at zoom 16 both grid sizes, fails at zoom 40 on the 2048 world **identically before and after (38.1%→38.7%) — pre-existing, not this milestone's fault**. LOD-entry `mean |ΔL*| ≤2.0` **fails both sizes** (2.44 @512, 2.03 @2048) — root cause identified (the tile's meso shading isn't divided by its sample step, LOD-D1's own flagged owner decision): tested and reverted a fix that closes it at 2048 (→1.71) but does nothing at 512. Seam ratio `≤1.5` passes at 2048 but **fails and worsened at 512** (1.57→1.85 median, 3.5→19.6 worst) — the engine-level seam check still passes, so this is screen-side (D3's territory), not tile content. Also disclosed: worst camera step 132→225 ms at 2048×1311 (all one-time cache build, medians unchanged; D6 candidate), ~88 MB retained once LOD engages, no `lod_clear_cache` wired yet. Commit `9d2a800`. Unblocks D3. |
| ~~**LOD-D3 · Continuous transitions: parent fallback and a colour-space morph**~~ — **CLOSED 2026-09-21 (verified) — 2 of 4 D0 acceptance bars met, 2 disclosed not met** | `LOD_DETAIL_SCOPE.md` | medium | `lod_bridge::morph_for_zoom` (mutation-tested, 3/3 killed) shares `level_for_zoom`'s own expression minus rounding, so the fade and the level switch cannot disagree about the boundary — engine-side, one implementation. **Met**: zero pops (already 0 before, not earned by this milestone). **Not met**: worst level-boundary `T_i ≤1.5×` (2/12 failing boundaries → 1/12); zero holes (large movement at 2048×1311 — peak hole px 22940/19750/8030 → 0/1900/1900 across three seeds, root cause measured exactly: the pyramid samples `[0,gw−1]` while the map raster covers `[0,gw]`, so the outermost half coarse-cell is structurally outside every tile — unchanged at 512×384); seam ratio `≤1.5` **not moved at all** (same 3/18 runs fail before and after) — **D2's open seam item is explicitly NOT closed by this milestone**. Two real bugs the agent's own new probe caught after the code was written, both fixed: a parent tile's texel-sized inset overhung its children by half a child-texel (seam ratio 3.47→1.23 once laid out by the child's texel instead); the parent-ready flag raised one call too late froze the outgoing level's own tiles for a frame (the milestone briefly making its own target metric *worse*, 4.4619 vs 4.2543 pre-D3, until fixed to 4.2272). Side effect: D2's open LOD-entry `mean|ΔL*|` item improved on all 6 worlds (now passing 4/6, was 3/6) as a byproduct of the entry fade — not the reverted meso-normaliser fix, which stays untouched and open. Flagged, not fixed: the removed layer tween drops an incidental free redraw of the debug-overlay subtree on level-cross; `_loddbg_probe` hasn't been re-run against this change. Commit `b6cc014`. |
| ~~**`lod_cache_key`'s `glacial_snowline` term has no test coverage — its own mutant survives**~~ — **CLOSED 2026-09-21 (verified)** | `LOD_DETAIL_SCOPE.md` (LOD-D4 follow-up) | small | New windowed probe `_glaciallodkey_probe.gd`/`.tscn`: renders the same tile at two snowlines with ice on (asserts a real pixel region changed, biggest delta on cold land) and again with ice off (asserts byte-identity). **Independently re-run, not just trusted**: 13/13 passed against HEAD, reproducing the same pivot cell `(492,360)` and same diff figures (`differing=2640`, `max_diff=118`) on a second run; applying the named mutation flipped exactly the two ice-ON assertions to `differing=0`/`max_diff=0` (stale cached `TileFields` reused), reverting restored 13/13. **Finding, not fixed**: `glacial_snowline` folds into the cache key unconditionally, while the render path (`render.rs`'s `glacier_on`/`cryo` gates) only reads it when `ice_strength > 0.0` — so a snowline tweak with ice off still busts the cache for a rebuild whose result is structurally guaranteed identical. Over-invalidation, not a correctness bug; see the new row below. Commit `f0a1166`. |
| ~~**`lod_cache_key` over-invalidates on `glacial_snowline` when ice is off**~~ — **CLOSED 2026-09-21 (verified)** | `LOD_DETAIL_SCOPE.md` (LOD-D4/glacial-probe follow-up) | small | The term now folds in a constant `0` when `ice_strength <= 0.0` instead of the real value — confirmed safe by re-reading `render.rs`'s two gated call sites directly (the only readers of `tf.glacier`/`tf.cryo`) rather than assuming. No re-enable staleness risk: `ice_strength` is itself already covered by `appearance_fingerprint`, so flipping it back on changes the key on its own. Added `LodWorker::generation()` (existing internal counter, newly exposed) so the extended probe can assert "not rebuilt", not just "pixels happened to match". Independently re-run by the coordinating session: `_glaciallodkey_probe.gd`, 16/16 checks pass, matching the agent's report exactly — HIT 7.86 ms, MISS (genuine rebuild) 39.32 ms, FIX (this path) 7.89 ms, landing within noise of a cache hit and ~5× faster than a rebuild. Commit `095c7f4`. |
| ~~**LOD-D5 · Scale-aware shading weights, and hydrology that resolves**~~ — **CLOSED 2026-09-21 (verified) — 2 of 3 bars met, 1 structurally unattainable without an owner ruling** | `LOD_DETAIL_SCOPE.md` | medium | One new tunable, `TerrainAppearance::detail_scale_strength`, gating four continuous stages (shading weights shift macro→meso/micro with zoom; micro shade cross-fades to the tile's own relief; crest radius scales into ground units via a new `build_crest` stencil step; river SDF threshold scales by `cells_per_px^(2k)`), each the identity at `0.0` and at 1 tile px/coarse cell. **Finding**: AO needed no work — `ao_radius_frac` was already ground-scale, so half the scope's own "crest and AO in ground units" bullet was already true. Bars 1 (channel components rise with zoom) and 3 (zero pops) **met**. Bar 2 (detail/px non-decreasing 4→40) **NOT met, root cause measured**: `add_zoom_detail` decays each octave 0.6× while a level down halves ground/pixel, so non-decreasing needs changing that decay — this milestone's own stated non-goal and a golden re-baseline, recorded as an owner-ruling note in `amplify.rs`, not fixed. **Two regressions disclosed, not buried**: D2's zoom-40 detail bar goes from comfortably-met (56.7%) to exactly-at-the-bar (50.7%), isolated to the ground-unit crest (the old reading was inflated by a one-pixel-wide crest stroke, not real detail); D2's own open seam-ratio bar worsens further (1.7383→1.8361) with the obvious cause (crest edge skirt) tested and **refuted**, left explicitly unattributed rather than assigned. A mutation-testing harness bug (a positional filter silently ran zero tests while cargo exited 0) caught and fixed before reporting; real result 9/9 killed. Commit `c685930`. |
| ~~**LOD-D6 · Tile synthesis off the main thread, profiled per device**~~ — **CLOSED 2026-09-21 (verified) — desktop bars met, phone bars honestly unmeasured** | `LOD_DETAIL_SCOPE.md` | medium | Real Rust-side `rayon` threading (not `WorkerThreadPool` — `WorldGen` holds a `RefCell` and `Gd<...>` handles, neither `Sync`), a `Send+Sync` owned `LodSnapshot` behind an `Arc` whose `render_tile()` is the *only* colouring function called by both the main thread and every worker job (determinism structural, not coincidental), a generation counter that drops stale-landed tiles, per-tier tile budgets, and a tile LRU cache. **Measured headline**: main-thread cost of one LOD-entry `_update_lod()`, controlled A/B in one probe run — **199.50 ms (synchronous, pre-D6) → 0.98 ms (worker)**, replicated across three runs. This is the exact stall LOD-D3 flagged as a D6 candidate. Memory delta bounded by assertion (62.09–62.88 MB per tier against a 60 MiB budget); the two cache-size drafts that would have blown that bar were caught by the assertion, not trusted from arithmetic. **The phone bars (p99≤33ms/worst≤100ms frame time; thermal/battery over a 5-minute pan) are honestly reported as NOT measured — no handset reachable from this session.** `GUI_GAP_REGISTER.md`/§2.7's zoom-notch row (the 0.87s frozen frame on the 6T) is **NOT** closed by this milestone and **stays open** until a real device pass confirms D6 actually fixes it — the agent explicitly declined to claim that figure without hardware. Commit `c74a150`. |
| ~~**`WorldState`'s four `Vec<f32>` fields should be `Arc`, not cloned, for LOD-D6's snapshot**~~ — **CLOSED 2026-09-21 (verified)** | `LOD_DETAIL_SCOPE.md` (LOD-D6 follow-up) | medium | `field`/`temperature`/`rainfall`/`flow_discharge` are now `Arc<Vec<f32>>`. Blast radius smaller than feared: 5 of 6 crates reading `WorldState` (`-civ`, `-hydrology`, `-terrain`, `-spatial`, `-gpu`) needed **zero edits** — `Deref` carried every read — confirmed by the coordinating session (`git diff --stat` on all five: empty). 19 files touched across `cartalith-engine`/`cartalith-godot`/two `cartalith-civ` test fixtures; no `.gd` file needed changing. Mutation goes through `Arc::make_mut` at the 15 real production write sites (`erode_op`, `center_landmasses`, `recompute_stale`, sculpt-commit/undo/redo) — copies one grid only when a live LOD snapshot still holds it, against every-snapshot-pays-for-all-four before. Golden tests byte-identical (91 binaries, 516 cases, 0 failed, before and after). `cargo test --workspace`: 163/3431/0/33, independently reproduced exactly by the coordinating session after clearing a stale `.dll` file lock (four leftover windowed Godot probe processes from this session's own verification runs) and rebuilding clean; the `retained_bytes` mutation-test re-run in isolation, passes. **The in-app `retained_bytes` figure was NOT re-measured** — the agent couldn't rebuild the locked `.dll` to reprobe, and correctly declined to write an unverified number; the milestone's old `SNAPSHOT_2048X1311` budget constant is kept as a documented upper bound, not lowered. Two follow-ons flagged, not taken — see the two new rows below. Commit `e289c71`. |
| ~~**`LithoSource` still deep-clones three fields for an LOD snapshot**~~ — **CLOSED 2026-09-21 (verified) — it was four fields, not three; the earlier row miscounted** | `LOD_DETAIL_SCOPE.md` (WorldState-Arc follow-up) | medium | **All four** (`age_field`/`volcanic_field`/`crust_field`/`resistance_field`) are now `Arc<Vec<f32>>` on `WorldState`, confirmed at the symbol — the "three" in this row's own earlier text traced back to `e289c71`'s commit *message* misquoting its own doc comment, which already correctly named four. Mutation is narrow (4 sites, all in `center.rs::center_landmasses`); 5 of 6 reading crates needed zero edits, same shape as `WorldState`'s first four fields. New test fills a real gap — no existing fixture ever exercised the `build_lithology` branch with `litho: Some(..)` at all. Measured (same 2048×1311 grid, host-polled peak working set): 89.8 MB four deep clones → 46.9 MB four `Arc` clones, Δ42.99 MB — min-to-min delta matches `e289c71`'s own figure for the first four fields exactly. Independently re-verified: combined tree builds clean, `cargo test --workspace` reproduces 163/3432/0/33 exactly, new test re-run in isolation passes. Commit `e376cda`. |
| ~~**A loaded save still copies its three fields via `cartalith_io::SaveFields`**~~ — **CLOSED 2026-09-21 (verified) — genuinely separate storage from `WorldState`, needed its own fix, not already solved** | `LOD_DETAIL_SCOPE.md` / `SAVEFILE_COMPAT.md` (WorldState-Arc follow-up) | small | `heightmap`/`temperature`/`rainfall` on `SaveFields` are now `Arc<Vec<f32>>`; `volcanic_field`/`impact_field`/`strahler_order` deliberately left plain (a loaded save's `litho` is always `None`, and `strahler_order` is always copied into an owned buffer regardless — Arc-wrapping them would buy nothing). `lod_snapshot_inputs`'s `Loaded` arm goes from a fresh-copy-wrapped-in-`Arc::new` to a plain `Arc` clone; `save_project`'s own write path picked up the same win as a bonus. Memory impact derived (~32.2 MB, three grids vs. `WorldState`'s four) rather than independently re-measured, disclosed as such. Independently re-verified alongside the `LithoSource` row above (same combined commit, same test run). Commit `e376cda`. |
| ~~**`_loddbg_probe.gd` asserts a stale row count (3, real is 5)**~~ — **CLOSED 2026-09-21 (verified)** | `LOD_DETAIL_SCOPE.md` (LOD-D6 follow-up) | small | `_build_lod_debug_submenu` builds five rows — Grid, Colors, Labels, `add_separator()` (which genuinely increments `item_count` — a `PopupMenu` separator is a real, if uninteractable, row) and "Show tile borders on the map" — confirmed against `_refresh_lod_debug_menu`'s own index math, which already locates the border row at index 4, past the separator at index 3. Probe now asserts the literal 5 with a comment explaining the separator. Verified headless (runs clean through to the known section-4c null-texture hang point) and windowed to a full `PASS`, exit code 0. Settings file hash unchanged. Commit `7a26d46`. |
| ~~**EF-3 · Erosion-consistent mountain detail**~~ — **CLOSED 2026-09-20 (verified, conditional result disclosed)** | `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` | large | `stream_power_kernel_bounded` (`cartalith-erosion/src/lib.rs`) adds a pinned base-level ring and an `area_seed` inflow to the existing stream-power kernel for a tile-bounded re-run; `tile_erode` (`src/tile.rs`, new) wraps it with a `k·refine` correction. Mechanism independently verified sound, 13/13 mutation survivors killed; `stream_power_kernel` delegates to it with `(None, None)`, so whole-world golden fixtures are unchanged. **The acceptance test was first circular** (measured against the tile's own eroded output) — repaired to measure against a drainage network traced once from the pre-erosion field and held fixed. **The repaired result is conditional, not universal**: at `deposit = 0.0` EF-3 raises transverse-incision/log-area correlation by +0.058 to +0.087 (6/6 cases); at `deposit = 0.3`, this engine's own shipped default, the same measure *falls* by −0.050 to −0.148 (6/6 cases) — traced to deposition refilling a valley floor to its own start-of-iteration height each iteration when `uplift = 0`. **Not an EF-3 defect**: the same kernel run over the whole world, no tile involved, does the same thing on the same fixed network (+0.036 / −0.055) — EF-3 faithfully reproduces the existing engine kernel's own behaviour at its shipped parameters. Commit `794802d`. |
| **EF-9 · Importance-driven refinement** | `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` | medium | **Not yet asked about the owner** (split out 2026-09-20 from what became EF-7/Ruling N). A subdivision rule — a river corridor or settlement refines sooner than open plain — over inputs that already exist (`slope`/`curvature`/`tpi`, channel order, `boundary_type`/`volcanic_field`, settlement/road positions). The least de-risked piece of the design; proposed as a target once EF-0/EF-1 exist to measure against, which they now do. See `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` for the design and `LARGE_ITEM_RULINGS.md` Ruling N for the owner’s authorisation. |
| ~~**River ink paints onto lake and ocean water, unmasked**~~ — **CLOSED 2026-09-21 (verified)** | `cartalith-hydrology/src/lib.rs`, `cartalith-godot/src/render.rs` | medium | **Root cause confirmed against the real v2.11 reference, not guessed.** The reference's river-ink blend lives entirely inside `surfaceColor` (the LAND colour function); the main pixel dispatch is `isWater(vw) ? seaColor : surfaceColor` (8588), and `buildWaterBodies` classifies a lake exactly like the ocean, so a water pixel structurally never reaches the ink blend at all — this port composited the ink onto an already-resolved pixel colour with no memory of which branch produced it. **This was a straight, undisclosed port bug, not a reference-faithful defect** — no golden re-baseline authorization was needed. `chan[]` itself confirmed NOT the cause: `build_channels_with_threshold` already skips any cell below sea level by construction, the same threshold `build_water_bodies` classifies with — the extensive in-lake ink was pure disc bleed from `stamp_river_intensity`'s own up-to-9-cell-wide stamp, painted with no water check downstream. Three consumer paths gated on land together (the same screen/export drift discipline `RiverInk`'s own doc comment already names, `58dd5b2`): `build_color_texture` (screen), `bake_rect` (export), and the LOD deep-zoom tile path. No golden hash moved — no JS-reference golden-parity test touches this path; two Rust-internal self-consistency tests in `bake_raster.rs` updated to match. **Independently re-verified**: `cargo test -p cartalith-hydrology -p cartalith-godot --no-fail-fast` re-run in isolation, 907 passed / 0 failed / 19 ignored across 32 result lines, exactly matching the agent's report; before/after screenshots at the same "Lake Valarcca" view confirm clean open water after, with ink surviving only right at a real river mouth. Commit `b04a41b`. **A concurrent label-density agent was mid-edit on the same `lib.rs`** (an unrelated `label_dict` "weight" key) — this commit stages only the river-ink fix's own hunks via `git add -p`, verified by hunk count before committing; the label hunk was left unstaged for its own commit. |
| ~~**Pipeline stage rows 09/10 read as "didn't run" when they did**~~ — **CLOSED 2026-09-21 (verified)** | `cartalith-native/godot-project/shell/workspaces/world_workspace.gd`, `cartalith-engine/src/progress.rs` | small | **Landed exactly as scoped, commit `84d016c`.** `"(no engine work this run -- see gap note above)"` → `"(no dials of its own -- see gap note above)"` — reuses stage 8's own `"gap"` field vocabulary in the same `STAGES` table verbatim, so the inline note and the fuller note it points to now agree. Four characters shorter than the old string, so a neighbouring doc comment's 545px width figure still bounds it (disclosed as unremeasured, not re-asserted as a fresh "measured" number). No probe asserted the old string verbatim (grepped). Text only — `progress.rs`, generation behaviour and every golden untouched. **Independently re-verified**: re-ran the headless parse check on the file myself, clean; screenshot evidence (windowed, seed 483920) confirms both rows render the new text intact, no truncation. |
| ~~A 3-line-wrapped pipeline-stage-row state label draws visually above its own row's number/name columns~~ — **CLOSED 2026-09-21 (verified)** | `cartalith-native/godot-project/shell/workspaces/world_workspace.gd` | small | `_build_generate_head`'s four per-row labels (number, dot, name, state) had no explicit `size_flags_vertical`, so the `HBoxContainer`'s default cross-axis stretch let a 3-line-wrapped state label (stages 9/10's finished-state string, narrow dock) drift its own first line above the row's fixed-size columns. Fix: `SIZE_SHRINK_BEGIN` on all four labels plus `VERTICAL_ALIGNMENT_TOP` on the state label, so every column anchors to the row's top edge and the row grows downward instead. Verified independently: diff read in full, before/after screenshots viewed directly (before: wrapped text floats above "09 Ecology & biomes"; after: both start at the same top line), `git status` confirms only this one tracked file changed. Commit `c75cf5c`. |

**Closed from this table 2026-09-03** (batch 17, verified): *Geology microtexture /
dune ripples* and *Sky-view-factor and cast-shadow fields* — `tests/geology_micro_and_sky_fields.rs`,
8 tests; *SDF coast tinting, river bands and biome blend* — `tests/sdf_river_and_biome.rs`,
10 tests, wired into all three `with_appearance` consumers (`lib.rs:6561`, `:8045`,
`export_raster.rs:100`). Byte-identical at the default: `color_space.rs`'s
`FINISHED_RENDER_FNV1A = 0x6154_1058_49e7_10d6` is unmodified and still passes, so
no golden re-baseline was taken. **Two of the three named a blocker that was false**
— `render.rs`'s own "deliberately excludes" list was each row's cited evidence, and
it described the file that had already implemented them.

### 2.6 GPU, threading and memory

| Item | Owns it | Size | Next step |
|---|---|---|---|
| `compute_stress` gather reformulation on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Deferred at milestones 5, 6 and 9 in turn. Needs a scatter→gather rewrite plus its own float-equivalence re-verification |
| Erosion's per-cell parts (thermal, stream-power) on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Feasibility table rates it "Good"; no erosion shader among the ten `.wgsl` files |
| Phase 2 per-cell affordance fields on GPU (biome, carrying capacity, resource potentials, settlement suitability) | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | "Directly comparable to climate/erosion's per-cell case" |
| Water-body priority-flood (`build_water_bodies`) on GPU | `GPU_LAYER_INTEGRATION_SCOPE.md` | large | Half tractable, half genuinely hard — the above-sea depression fill is a global priority queue, and parallel Planchon-Darboux is a research task. ~92 ms at 1024² |
| Rendering / colour synthesis on GPU (`render.rs`) | `GPU_LAYER_INTEGRATION_SCOPE.md` | medium | The feasibility table calls it "best fit, no golden-parity tension at all", and the pilot named it the natural next target. Distinct from §21's beachhead argument in §5 |
| `cartalith-godot`'s own sequential orchestration | `CPU_MULTITHREADING_SCOPE.md` | medium | Named explicitly as untouched, and as "the real ceiling left" alongside the hard-hazard functions |
| ~~World-wrap support for the milestone 1-5 kernels (warp, heterogeneity)~~ — **CLOSED 2026-09-21 (verified)** | `GPU_LAYER_INTEGRATION_SCOPE.md` | medium | Both stages now dispatch on GPU under `world=true`. The real CPU reference (`compute_warp`/`compute_heterogeneity` in `cartalith-terrain`) calls `pfbm`/`pvnoise` under `world=true` — a genuinely different periodic noise algorithm, not a coordinate-wrap trick — so this added `gpu_pvnoise`/`gpu_pfbm` (periodic siblings of `gpu_vnoise`/`gpu_fbm`) to `cartalith-noise`, matching WGSL functions in both shaders behind a `world` flag (warp reuses its `_pad2` slot; heterogeneity stays 16-byte aligned), and threaded `world`/`p_x` through every dispatch function and both CPU-shape reference twins. No golden-parity risk (`WorldParams::defaults()` ships `use_gpu: false`). Verified independently: full diff read against every claim, `cargo test --workspace --no-fail-fast` re-run (163/3487/0/34, matching exactly), `project.godot`/every `.gd` file confirmed untouched. Commit `023c904`. |
| Full `ComputeTier` capability classifier | `GPU_COMPUTE_PILOT_SCOPE.md` §4 | medium | `crates/cartalith-gpu/src` contains only `lib.rs` and `multi.rs`; grep for `ComputeTier` returns nothing |
| Performance telemetry system | `GPU_COMPUTE_PILOT_SCOPE.md` §24 | medium | Deferred until more than one workload needs monitoring; nine kernels exist now |
| GPU memory pooling across persistent fields | `GPU_COMPUTE_PILOT_SCOPE.md` §14 | medium |  |
| Tiled / chunked GPU compute (§18) | `GPU_COMPUTE_PILOT_SCOPE.md` | large | Partly unblocked by the LOD pyramid. `multi.rs` ships a band split covering exactly one kernel (`gpu_warp`), 1.22-1.54× at 4096² and a loss at 2048² and below |
| ~~**Integrate `QuadTree` and `TiledField` into a real caller, or retire them**~~ — **CLOSED 2026-09-22 (verified) — both retired, `DirtyTracker` kept** | `LOD_TILING_BASE_SCOPE.md` | medium | Confirmed unused by a full-workspace grep before deleting anything — every external hit was a doc comment, zero real code usages, not a close call for either. `TiledField<T>` (+`RegionView`/`RegionViewMut`): every real consumer needed only dimensions, not an owned `Vec<T>` — `PassBuffer` does its own tile maths, `lod_bridge::tile_bounds` borrows `gw`/`gh` directly; using it on a live field would mean cloning up to 192 MiB to reach arithmetic that never reads the data. `QuadTree<T>` (+`Node`/`NO_CHILD`): the one real candidate consumer (`landmark.rs::sep_min_max`, parallel and golden-pinned) would be slower and riskier with a range-query quadtree; the tier these were shaped for (Z3) is out of scope per `LOD_TILING_INTEGRATION_SCOPE.md`'s own numbers. `DirtyTracker` kept — real callers confirmed in `sculpt_commit.rs` and three bridge files. The deferred `tile_size` benchmark never existed (prose only). 16 tests deleted with the structures; everything else is comment-only (cross-file doc links re-pointed, two scope docs corrected from "deferred" to "retired"). Verified independently: residual grep confirms zero code references remain, `cargo test --workspace --no-fail-fast` re-run in this exact file scope (164/3485/0/34, zero movement from the concurrently-landed E1 commit), `godot --headless --check-only` clean, `project.godot` unchanged, `git status` confirms exactly these twelve files. **Process note**: the agent briefly swept another session's in-progress `render.rs` into a `git stash` while establishing a clippy baseline — a new instance of `MISTAKES.md`'s shared-file class (never stash in the shared tree, use a worktree); the stash popped cleanly and `render.rs` was independently confirmed intact before commit. Commit `5c99cc9`. |
| ~~**GPU device reuse across generations**~~ — **CLOSED 2026-09-21 (verified) — Ruling Y built** | `GPU_LAYER_INTEGRATION_SCOPE.md` | medium | Commit `b6f2816`. A process-wide `DEVICE_CACHE` in `cartalith-gpu::multi`, keyed on the `GpuPreferences` fields that decide adapter selection; `GpuDevice`/`GpuDeviceSet` clones are `Arc`-backed handles onto the same device, not new ones. `wgpu::Device::set_device_lost_callback` registered on every successful `request_device`, writing the same shared flag the existing reactive `read_back` detection already used. Two-tiered recovery: within one generation, loss falls back to CPU per-stage via the existing tested `device_is_unusable` guard; across generations, the next call sees the lost flag as a cache miss and opens fresh. Mid-dispatch retry considered and rejected — new plumbing, same partial-result hazard the ruling guards against. **Real measurement, this session's own hardware** (AMD Radeon RX 7800 XT, Vulkan): three successive `generate_terrain` calls 269.4 / 23.0 / 22.4 ms — ~11.7x on repeat. Two real-hardware device-loss tests (not mocked): one destroys a real device and confirms the callback fires; one proves cache-reuse-then-fresh-on-loss via `Arc::ptr_eq`. **Independently re-verified**: re-ran both device-loss tests myself on the same hardware, both passed, confirmed the same GPU and fallback message in the log; full workspace suite reproduced 3479/0/34 unchanged. `project.godot` and the live settings file confirmed unchanged. |

### 2.7 Android and on-device verification

No Android pass has run since 2026-08-25. All six items below are live.

| Item | Owns it | Size | Next step |
|---|---|---|---|
| ~~**PLAN’s header TITLE half (“PLAN · STAGE n”) goes stale on stage isolate**~~ — **CLOSED 2026-09-20 (verified)** | `ANDROID_UI_SPEC.md` | small | `_on_stage_clicked()` (`journey_planner_view.gd`) now calls `app._refresh_phone_sheet_header()`, same guard and shape as the sibling subtitle fix (`554d953`), which had named this exact gap as its own residual. Verified with a positive control: a windowed probe (`_stagetitle_probe.gd/.tscn`) reproduced the bug on HEAD (title stuck at "PLAN" after isolating a stage, exit 1) before the fix, and passed after (title reads "PLAN · STAGE 1", reverts to "PLAN" on un-isolate, exit 0). The sibling subtitle probe re-run unmodified, no regression. Commit `f493b61`. |
| ~~**The Android adaptive icon had no background layer**~~ — **CLOSED 2026-09-21 (verified on the 6T)** | `ANDROID_BUILD_SCOPE.md` | small | **Owner-reported 2026-09-03 ("on the 6t the icon is a dull weird grey scale"); root-caused and fixed the same day, unverified on device until now.** `icons/android_adaptive_background_432.png` was an 804-byte fully-transparent blank; now opaque `rgb(0,24,48)`. Second change: `cartalith icon2.png` dilated (MaxFilter 19) into the monochrome layer for **17.05%** ink, inside Android's themed-icon band and fully inside both safe circles. **Verified directly**: `adb shell screencap` on the 6T's home screen (device `9608b26b`, still connected from earlier this session) shows the Cartalith icon as a clear compass-rose emblem with real detail and contrast on the launcher's themed-icon grey plate — not the blank/washed-out grey the owner reported. Viewed through the system's Material You themed-icon rendering (monochrome layer only, launcher-tinted circle), which is the same rendering path that originally exposed the bug, so this is the correct state to check. |
| Phone MORE — §6.6's sub-screens: **nine built, two declined with reasons, one blocked** | `DESIGN_HANDOFF.md` | small | **2026-09-06. Rows are real, checked against live data rather than against the screens' own prose**, and reproduced by the verifier digit for digit: travel draws four chips over four different row sets (animal 37, vehicle 29, vessel 53, preset 17), `landmark_kinds()` is 49, `lm-fam` gives 37 rows for `physical`, and after a real pass `caps total 384 · last run placed 254`. **Absence is drawn as absence** — with no world the same screen says the estimate needs one, and an empty slot says *"No art in this slot yet"* rather than inventing a size. The probe carries a control (an unknown screen id draws only its placeholder) so a lost match arm cannot pass vacuously. **The resolve-by-menu-name+id property was proved by scratch edit**, not by reading: a row injected into `menus.gd::_help()` was enumerated and required to be drawn with `phone_menu.gd` untouched. **`data-tiles` is genuinely blocked** — `wmts|tms|slippy|z/x/y|zoom_level` returns nothing across the workspace, so there is no engine behind it. **`data-io` was declined as a duplicate**: `_fill_data()` already covers all 14 `DataManagerWindow.ROUTES` one level up |
| ~~**Three round-3 GUI surfaces built: rail subtitles, Find-on-map scopes, seeded layouts, the Checks route**~~ — **CLOSED 2026-09-21 (verified) — all four surfaces confirmed, post-Ruling-L** | `DESIGN_HANDOFF.md` | medium | **Built 2026-09-06 from `design/round3-corrected/`, all four verified.** **Rail:** three subtitles added verbatim from `DOMAINS[i].subtitle`, wrapping not clipping (CIVIL is **71** characters, not the artboard's 70) — and the blocking check passes on the strongest evidence available: `const RAIL_NODES` and `const DOMAINS` are **byte-identical** to `da57ca4`, so no node was replaced by a label. **Find on map:** five scope prefixes matched as a whole token (`lb` cannot be reached by `l`), the `.` scope reaching `CommandIndex` — whose **first shipping consumer in the whole shell this is** — client-side band headers proved not to re-rank (drawn order byte-identical to `search()`'s), and a count. `l landmarks` ships **disabled with `place_search.gd`'s own reason as its tooltip and its empty-state text**, so a scope that would always return nothing explains itself. **Layouts:** four task layouts seeded once, proved across two processes with a genuinely deleted config — a forgotten seed stays forgotten. **Checks:** the route over the five real validators, `LOCATE` present and disabled, no `FIX SELECTED` **RE-VERIFIED 2026-09-13 except the rail subtitles:** `_railfind_probe` PASS (pointer and touch; renaming a scope makes it FAIL), `_seedlayout_probe` phases 1 and 2 PASS, `_dm10_probe` OK, `_placesearch_probe` PASS. **Rail subtitles re-verified 2026-09-21, now that Ruling L (all three tabs) is complete**: genuinely unaffected — Ruling L restructured `RAIL_NODES` categories/modes, not `DOMAINS` subtitles, confirmed by `git log` showing no diff hunk ever touches the subtitle values. CIVIL's subtitle is still the longest at 71 chars against the design's stated 70, still wraps to 3 lines rather than clipping, still the tallest of the three, at both the pointer (200 px) and tablet (264 px) rail-column widths. One unrelated stale probe literal found and fixed along the way: `_railfind_probe.gd`'s exact-ten-keys check still expected the retired `cartography/terrain`, updated to `cartography/layers` per Ruling L's CARTO re-sort. Independently re-verified by the coordinating session: both probe legs (pointer, tablet) `PROBE PASS: 0 failure(s)`, reproducing every figure exactly. Commit `837cbfb`. |
| ~~**Implement `Cartalith Settlement Editor.dc.html` + Touch variant** — new GUI: redesign the existing place editor as a tab-strip window~~ — **CLOSED 2026-09-21 (verified) — all six batches landed** | owner-supplied design canvas, project `067f80e7-dbb7-4492-8e69-96aaa8050a4d`; plan `lazy-riding-piglet.md` | large | **Merged into one row 2026-09-21**: touch resolved with the owner as one `.gd` file with responsive branches (matching every other window in this shell — `faction_roster_window.gd`/`culture_profiles_window.gd`'s established convention), not a separate build, so the two design docs are one implementation target. **Scope corrected once the code was read**: `place_editor_window.gd` (909 lines, pre-existing) already implemented almost everything artboards 1a/1b/1e redesign — this is a restructure + four genuinely new pieces (Timeline sim UI, Political history, a Settlement-types library, Layout/Generation-rules with one real engine signature change), not a build from scratch. Full plan at `C:\Users\Vincent\.claude\plans\lazy-riding-piglet.md`, six batches (A–F), each independently verified and committed before the next starts, per this session's standing discipline. **Batch A CLOSED 2026-09-21 (verified)**: tab-strip restructure (Overview/Economy & notables/Timeline-placeholder/Political-placeholder/Vault notes), every existing `_build_*` function's logic reused verbatim, tab switching reuses the PE-01 focus-commit fix rather than a new mechanism, one segmented-button row for both desktop and phone (not `TabContainer`). No engine call changed. Verified independently: full diff read line-for-line, `DccTheme.empty()`/`panel()` signatures confirmed to match, `godot --headless --check-only` parses clean, `git status` confirms only this one file changed, `project.godot` unchanged. Commit `76117f4`. **Batch B CLOSED 2026-09-21 (verified)**: real Timeline tab — recorded-years chip scrubber, Add/Go to/Remove year, `civ_year_diff` readout (reports the bridge's real `present`/`added`/`removed` counts, not the canvas's invented "N changed" figure — the bridge has no such key), the real collapse/recovery simulator wired to `civ_run_collapse_simulation` with a genuine overwrite-confirmation gate (`needs_confirm`/`clobber_years`) before a confirmed re-send, and a run report reading real returned numbers. Authored events stay dashed with the canvas's own stated reason (no per-event store exists). GDScript-only — every bridge call already existed with a thin `engine_bridge.gd` wrapper, confirmed by reading the actual signatures rather than assumed. Verified independently: full diff read line-for-line, six `DccWidgets` API signatures confirmed to match their actual declarations, `get_civ_timeline_years`/`get_civ_year` confirmed to exist, `godot --headless --check-only` parses clean, `cargo test --workspace --no-fail-fast` reproduced the exact pre-existing baseline (163/3490/0/34, zero movement), `git status` confirms only this one file changed. Commit `29de0a5`. **Batch C CLOSED 2026-09-21 (verified)**: real Political history tab. New additive Rust —
`cartalith_civ::timeline::civ_settlement_ownership_periods` derives contiguous per-settlement ownership
spans by walking every recorded `TimelineSnapshot`, reading `NamedSettlement::placement::faction` directly
(kept in sync with the territory raster at every write site that can move one without the other). A year
the settlement is absent from is a gap, not an extension — caught as a real bug in the agent's own first
draft and fixed. Genuinely per-settlement (checked against the timeline's 2000-year cap, not assumed
cheap like `TradeStore` would require). Four new unit tests with exact literal span assertions. GDScript:
a stacked ownership bar + period list, both read-only per the canvas's own framing; manual entries stay
dashed with the canvas's stated reason. Verified independently: full diff read for all four touched files,
`cargo test --workspace --no-fail-fast` reproduced exactly (163/3494/0/34, +4 matching the new tests
exactly), both `.gd` files parse clean, `git status` confirms scope, `project.godot` unchanged. Commit
`86f3ecd`. **Batch D CLOSED 2026-09-21 (verified)**: the Settlement types library. New window
(`settlement_types_window.gd`) over a new static store (`settlement_type_store.gd`, `TradeStore`'s own
shape): named field bundles (kind, specialisation, traits, walls, age policy) plus a per-faction default
column, reusing the place editor's own toggle-chip and vocabulary patterns. Storage: a new caller-owned
document slot, `library/settlement_types.json` — registered in `DOCUMENT_SLOTS`, deliberately absent from
`ENGINE_OWNED_SLOTS`, confirmed at the symbol (not assumed). Wiring: the settlement-drop tool applies a
faction's default bundle via the same `civ_edit_settlement`/`civ_settlement_toggle_trait` calls the place
editor already uses — a no-op, today's behaviour unchanged, when no default is set. Placement rules and
name-pool override beyond Inherit stay dashed per the canvas's own stated reasons. Verified independently:
document-slot classification confirmed at the symbol, all four touched/new `.gd` files parse clean,
`cargo test --workspace --no-fail-fast` reproduced exactly (163/3494/0/34, unchanged from Batch C as
expected), `project.godot` unchanged, `git status` confirms scope. Commit `933f164`. **Batch E CLOSED
2026-09-21 (verified)**: the sixth and last tab, a readout over `urban_layouts()` — fact chips, a
street-class breakdown, the stages-that-ran checklist, substrate flags, and a pointer to Population/Age on
Overview rather than a duplicate view. **Corrects a stale claim in the design canvas itself**: 1g's intro
says the bridge "emits no key at all for buildings, districts, amenities or the wall circuit" — verified
false at the symbol (`urban_bridge.rs::layout_dict`); `buildings`/`building_district`/`markets`/`walls`/
`wall_ring` are all present today. The genuinely-absent pair is narrower: civic hall/places of worship,
and hinterland clutter beyond field/pasture. Verified independently: the key-presence claim re-confirmed
directly via grep (not trusted from the report), full diff read, `godot --headless --check-only` parses
clean, `cargo test --workspace --no-fail-fast` reproduced exactly (163/3494/0/34, unchanged as expected),
`git status` confirms scope. Commit `3a9c5cc`. **Batch F CLOSED 2026-09-21 (verified) — the real engine
change, and the last of the six.** `cartalith_civ::urban_adapter::run_layout` gains `rules: Option<&Rules>`;
the no-override entry point `settlement_layout` forwards `None` unconditionally, so every existing golden
is untouched code. `Rules::to_patch()` populates every field as `Some`, reaching `GenOpts::rules` through
the existing `RulesPatch` partial-merge mechanism as a wholesale overwrite. **Golden-parity proof, the
load-bearing property this batch depended on**: full suite run before (`git stash`) and after, every one
of 163 result lines diffed individually — 3494 → 3496 (+2, the two new tests), nothing else moved. A new
reachability test proves a non-default `Rules` value genuinely changes a real layout (edges 327→312,
street_len 10898→11718). Bridge: `WorldGen.urban_rules`, world-level, in-memory only (disclosed, no
save/reload persistence — per-faction scope left as a later owner decision). New window
`generation_rules_window.gd`: presets, two convenience sliders reusing the real, already golden-tested
`apply_wildness`/`apply_plot_chaos` (not reimplemented in GDScript), street/parcel parameter tables, a
dashed wall-generation block per the canvas's own stated reason. Batch E's disabled placeholder now opens
this window for real. **Independently re-verified by the coordinating session, not trusted from the
report**: full diff read for the signature change/`Rules` struct/`WorldGen` field/`to_patch`'s exhaustive
field population; `cargo test --workspace --no-fail-fast` re-run and reproduced exactly (163/3496/0/34,
matching the report's own numbers precisely); all five touched/new `.gd` files parse clean; `git status`
confirms scope. Commit `5fa0011`. **All six batches of `lazy-riding-piglet.md` are now landed and
independently verified — this row is closed.** |
| **Three-platform design-conformance verification — owner request, 2026-09-05** | `DESIGN_HANDOFF.md` | large | **Trigger: when ALL GUI work is done, not before.** Owner instruction, verbatim: *"When all GUI work is completed I want you to use fable 5.1 to verify tablet (simulated), pc and on the connected phone by adb. All windows, panels functions should be in-line for 100% to the design."* Three targets, and they are **not** interchangeable: **tablet simulated** (the `tabL`/`tabP` frames — 2560×1600 and 1600×2560, the full 17-token touch density override), **PC** (`w1920`, and `w1366` which carries its own 3-token override), and **a real handset over `adb`**. **Model: Fable 5.1.** Scope is every window, panel and function, at 100% conformance — wider than the menu audit below, which walked menus only. **Two things to establish before dispatching, not during:** **a device IS attached** — checked at the moment this row was written rather than assumed: `adb devices` → `9608b26b  device`, 2026-09-05. That matters because the last recorded USB session was **2026-08-24** and six features have been carried as *unverified on device* ever since, so the row was first drafted saying an unattached phone was the likely state; measuring took ten seconds and it was wrong. **Re-check at dispatch anyway** — a handset is unplugged between sessions. And confirm that the APK on the handset is the build under test — a shipped APK once carried a `.so` 25 commits stale, so the row below's `.so` check is a precondition for this one, not a parallel task. **Headless cannot stand in for any of the three**: `ImageTexture.update()` is a no-op under `--headless`, so anything pixel-shaped runs windowed |
| ~~**`File ▸ Recent worlds` leaves show a filename where the canvas shows the world**~~ — **CLOSED 2026-09-21 (verified) — both halves now** | `DCC_SHELL_SPEC.md` §2.1 | small | Seed half closed 2026-09-13 (batch wf53). **Name half closed this pass — and the premise this row carried ("ELDRA" implied a name generator already existed) was checked and found FALSE first**: "ELDRA" was a hardcoded literal in `app.gd`, not a generated name. Built: `cartalith_civ::naming::world_name(seed)` (reuses existing culture/syllable pools, a fresh non-parity-bound RNG stream since this concept has no reference equivalent); `cartalith-io::SaveParams::name`, additive (`world.origin`'s own precedent, no `format_version` bump), documented in `SAVEFILE_COMPAT.md`; wired through `WorldGen::absorb`/`get_world_name()` into Recent worlds, welcome tiles, gallery tiles and the phone picker, all with filename fallback. Two bugs caught and fixed in passing, not introduced by this change: the status pill never updated when opening a project from Recent worlds; a `split(" · ")[0]`-unconditional parse in `phone_menu.gd` that this change's own bare-seed fallback would have made misread a seed as a name. 11 new tests (4 `cartalith-civ`, 7 `cartalith-io`) plus a real headless probe round-tripping a generated name through both save formats and confirming a pre-existing archive with no `world.name` still opens with an empty name, never an invented one. Commits `5e25c40`, `c16151f`. |
| ~~**Ruling 29 versus the `Data ▸ Export ▸ Maps ▸ tiles` row**~~ — **CLOSED 2026-09-21 (verified) — Ruling X: does not cover it, stays in Export** | `EXPORT_SCOPE.md` | small | Ruling 29 (*"the tiled output should only live in the save menu"*) scopes its own body to the LOD pyramid and the proposed Build Manager. `Data ▸ Export ▸ Maps ▸ tiles` is a different artefact — a region-marquee PNG grid (`PANE_PURPOSE.export_maps`), not the LOD pyramid. **Owner ruling: Ruling 29 does not cover this row; leave it in the export menu.** No code change — this closed the ambiguity, not a build. |
| ~~**Rulings 28/29’s "size shown at save time" has no home in the shell**~~ — **CLOSED 2026-09-22 (verified), owner authorized building it undesigned** | `SAVEFILE_COMPAT.md` | small | **Owner explicitly authorized building this without a design canvas.** New `WorldGen::lod_save_pyramid_estimate_bytes()` binds the pre-existing `pyramid_mask_bytes` estimator; a new real producer, `LodSnapshot::render_pyramid_masks()`, loops the SAME `render_tile` the live interactive LOD path uses (no second `RenderCtx` to keep in sync); `project_save_with_documents` gains `include_lod_tiles` (default false); the save dialog gets a plain checkbox showing the live byte estimate, sticky for the session; the disk-space pre-check now accounts for it. `SAVE_PYRAMID_MAX_LEVEL=4` chosen from a real measurement (0..=4 deflates to 18.1 MiB/1.45s; 0..=6 would be 131 MiB/33s). **A real bug found and fixed mid-task, not scope creep — it blocked the feature**: `project.rs`'s per-tile byte-length guards still assumed one byte per pixel, the pre-LOD-D2 shade-ratio format; real tiles have been RGB (3 bytes/pixel) since LOD-D2 shipped 2026-09-21, and nothing had exercised a real producer against the guard until this task did (`RasterLength { expected: 48896, got: 146688 }`, exactly 3x). Fixed with a named `LOD_TILE_CHANNELS=3` constant, both read/write guards corrected, stale byte-count comments in `project.rs`/`lod_bridge.rs`/`SAVEFILE_COMPAT.md` §16.1 corrected (the real archive sizes are 5-6x larger than what those comments claimed, predating this feature). **Verified independently**: full diff read; `cargo test -p cartalith-io` (135/0) and `-p cartalith-godot` (632/0/8) reproduced exactly, including both new round-trip tests in isolation; the Godot-level probe (`_lodsavepyramid_probe.gd`) re-run directly, byte-for-byte matching (estimate 44,520,960 == real written tile bytes, 341 tiles, clean reopen, a pre-existing untiled archive still opens with zero warnings); `godot --headless --check-only` clean on all three touched `.gd` files; `project.godot` confirmed unchanged. Two pre-existing, unrelated test failures in `cartalith-urban`/`cartalith-civ` (from the concurrent wall-hugging-buildings workstream, still in progress) were investigated and confirmed pre-existing by stashing this batch's own files and reproducing identically. Commit `8cade54`. |
| **THE PHONE SHELL MUST MATCH `Cartalith Android.dc.html` 100% FAITHFULLY — owner, 2026-09-07** | `ANDROID_UI_SPEC.md` | large | **Owner: *"The app only faintly resembles this version … where it should resemble it 100% faithfully."* This supersedes the two rows below it — they are symptoms of this.** **The gap is measured, not impressionistic.** The canvas declares **26 repeated collections** and **69 interaction handlers**: `genGroups` `stageRows` `progLog` `resGroups` for generation; `brushFields` `sculptFeatures` `sculptPresets` for sculpt; `layerGroups` `mapTools` `ramps` `stylePresets` `iconVars` for cartography; `partyGroups` for the planner; plus `inspRows` `histRows` `searchRows` `moreRows` `modalExtents` `pickerWorlds` `ovFields` `simSpeeds` `verActs` `toasts` `gridCells` `tabs`. Its generation handlers alone are `hGenGroup` `hGenSeg` `hGenStep` `hGenTog` `hGenerate` `hCancelGen` — **groups, segments, steppers and toggles**, which is exactly the control set the owner could not find. The shipped phone lifts a **one-row tool-options strip**. **It is also an interaction spec, not just a layout:** the header reads *"interactive — drag · pinch · rotate · long-press · edge-swipe"*, and it carries a three-detent sheet model (peek / half / full, 12 mentions of `detent`). **This is a rebuild, not a patch, and it needs a per-screen inventory before any lane touches code** — the round-3 surfaces each had one and those went well; the ones briefed from a summary went badly. **Two standing rules apply and must be said out loud:** an owner decision is newer than any canvas, so where this canvas and a later ruling disagree the ruling wins (`Data ▸ Conversion`, rulings 26/28/29); and **every "Exists today" claim gets opened at the symbol** — seven such claims failed against this code in one week |
| **`ColorPickerButton` ×8 and 123-155 other controls eat the drag by `MOUSE_FILTER_STOP`** | `ANDROID_UI_SPEC.md` | medium | **The second mechanism, named and deliberately not fixed by the gesture passes — recorded so it is not rediscovered as a new bug.** A `STOP` control ends the event walk, so the `ScrollContainer` above it never sees the press and cannot arm its drag. **`ColorPickerButton` is not the touch-DOWN class** (`action_mode=1`, measured by instantiation on 4.7.1) **but it does eat the drag** — the verifier’s rig reports `swipe_scrolls=false`. **The population, with its states named, because the batch’s own governing rule was applied to the dropdown number and not to this one:** across the probe’s eight states at 1080×2340 it runs **123 / 123 / 123 / 124 / 145 / 148 / 148 / 155**, not the flat 123 the report and the comment both carried. Composition at boot: `ColorRect` 94, `PanelContainer` 10, `ColorPickerButton` 8, `Control` 7, `Button` 3, `Label` 1. **Most are decorative and harmless; the question is which of them sit over a scroller a user needs** — that triage is the work, not a blanket conversion to `PASS`, which would be a change of a different kind |
| **Point Godot’s remote debugger at the ANDROID build and read the node tree off the device** | `ANDROID_BUILD_SCOPE.md` | medium | **Owner suggestion, 2026-09-07, and it targets this project’s most expensive recurring gap.** Every phone probe we have walks `get_tree().root` **in a desktop window booted with `--force-touch`**. That reports a healthy shell while the real handset is broken — it did so for five sessions, and the owner found two defects in minutes that a whole session of probes had passed. **The census, the hit-test and the reachability columns are all measured on a machine that is not the target.** **What to try:** Godot 4.x supports remote debugging into a running export. If the Android build can be attached to, the same walk that produces `_gestclass_probe`’s census would run **against the device’s own tree** — real DPI, real `content_scale_factor`, real touch driver, real `emulate_mouse_from_touch`. **That converts our strongest instrument from a simulation into a measurement.** **Open questions to answer before building on it:** whether the export preset must enable remote debug (`export_presets.cfg` is read-only here — an owner decision if it must change); whether the debugger can drive input or only observe (observation alone is still worth it — it would have caught the boot-state defect); and whether it works over USB or needs the network. **Does NOT replace `screencap` + `input tap`** — a node tree still cannot prove a finger reaches a control, which is the distinction this project keeps having to re-learn |
| **THE TABLET CANVAS HAS NEVER BEEN IMPLEMENTED AGAINST — it is a DIFFERENT SHELL, not a scaled one** | `design/mcp-2026-09-07/Cartalith Tablet.dc.html` | large | **The largest conformance gap in the project, found 2026-09-07 by photographing all three canvases against the shipped app, and rated BLOCKING by the re-check. Both fix lanes declined it as out of their file grant, so it is entirely unaddressed.** **The shipped tablet is the DESKTOP shell × `TOUCH_SCALE 1.53`.** The canvas is a different shell: a **horizontal `--railH:56` rail under the menu bar** where the shell draws a **vertical 48 px left strip**; a menu bar of an **overflow square + 3 menus** where the shell draws **7**; and **`scrPlanner` as a third top-level screen**. Visible at a glance in `captures/pairs/P10_tab_1600_shell.png`. **Docks are 232 dp portrait / 320 dp landscape against a flat 400 — and no portrait/landscape split exists anywhere in the GDScript** (`grep` for `tablet_portrait`/`TABLET_PORTRAIT` returns nothing). **12 of 19 palette tokens differ**, re-parsed by the re-check: `--hair #232628`, `--div #1e2123`, `--bor #2c3033` are **opaque hex** where `DccTheme.DARK` uses white-alpha, so there is no alpha to reconcile; `--sur`, `--pan`, `--ink`, `--dis`, `--ins`, `--accInk`, `--good`, `--warn` all differ; `--map`/`--tst` have no shell token at all. Metrics confirmed: `railW 52`, `sbH 28`, `row 48`/`rowD 44`, `ldW/rdW 232|320`, `railH 56`, `rCtl 12`, `rPan 16`. **This needs an owner decision before it is built**, because adopting it changes the tablet from "the desktop shell, larger" to its own shell — and `dcc_theme.gd` now records the measured token table so the decision can be made from facts |
| **The tablet palette is a STRUCTURAL change, not three numbers — and one contrast pair drops below 3:1** | `TABLET_UI_SPEC.md` | medium | **Two independent 17-token derivations agree on 16 of 17 rows and on every qualitative conclusion; the verifier found the one disagreement and the inventory lane was wrong on it.** Truth: **17 shared, 5 exact, 12 differ, 2 one-sided** for DARK (`--wash2` is a DIFFERING token — `DccTheme.DARK:198` has `accent_wash_2` at `.16` against the tablet’s `.15` — not the "no token" the lane filed, which makes both its counts off by one and its correction of my figure itself wrong). **The five that agree are the whole text ramp plus the accent** (`--body`, `--sec`, `--dim`, `--faint`, `--acc`), so the tablet canvas is not a different palette so much as a different GROUND. **The hairlines are the structural part, and it is confirmed by composition rather than by inspection:** `DccTheme`’s `line` is white at α .10, which resolves to `#2a2b2c` over panel and `#252627` over bg, while the tablet’s `--hair` is `#232628` **everywhere**. **No alpha reproduces both** — adopting it is a change to how DARK is built, not a re-tune of three values. **Computed consequence, and it is the reason this needs care:** ghost ink on panel goes **3.11:1 → 2.64:1**, crossing below 3:1. Everything else moves less than 0.5 (body 11.41→1113, accent on ground 8.75→8.62), and light reversed ink IMPROVES 4.30→4.72 |
| **§11: the fills are restored; the 51 stale citations are what is left** | `DCC_SHELL_SPEC.md` §11 (**repository root, not `docs/`**) | medium | **Owner, 2026-09-07, naming the mechanism: outdated documents prevented the new style taking hold. Swept 2026-09-07 and they are right, but the decisive document is LIVE, not stale.** **`docs/DCC_SHELL_SPEC.md` §11 — current in the owner’s design project — ends: *"No fills on panels: regions are separated by hairlines only. Radius 0 everywhere."*** That is what `asset_library_window.gd:889` cites as *"§11: no fills, radius 0"*, and the shell has implemented it faithfully. **HALF of it the shell got wrong, and that half is a plain bug:** §11 restricts fills to **panels**, while the SAME document requires them on interactive elements — §10: the layers popover’s active row is *"filled accent with reversed type"*; §7: the active ramp’s *"row filled"*; and §11’s own type rule, *"Filled accent surfaces carry reversed paper-coloured type"*, which presupposes filled surfaces exist. **"No fills on panels" became "no fills anywhere" somewhere in the port.** **The OTHER half is a real conflict between two live owner documents and must not be resolved silently:** §11 says **radius 0 everywhere**, unqualified; the current canvases draw **81 `border-radius:999px` pills and 76 `8px` corners** in the PC file alone, and their buttons are `border-radius:8px`. **RULED by the owner, 2026-09-07: *"The radius should follow the newest designs."* So:** the canvases are newer and are what the owner has pointed at all session, and an owner decision outranks a spec — so the canvases win and **§11 should be CORRECTED rather than left contradicting them**, or the next port re-infects itself from the same sentence. **Also swept, and separate: 49 citations of SUPERSEDED canvases remain in shipping code** — `DCC Shell.dc.html` ×20 across 12 files, `Android Phone.dc.html` ×17 across 8, `Menu Structure v2/v3` ×11, `Journey Planner DCC` ×1 — against 181 citations of the current ones (`ENV:` 156, `AND:` 19, `TAB:` 6). The tail is small but it is exactly the mechanism the owner named **Applied:** buttons now ship `ROLE["btn_radius"] = [8, 12]`, both figures canvas literals — and the lane corrected the brief on the way: **the tablet canvas contains no `8px` radius at all**, its buttons are `border-radius:var(--rCtl)` with `--rCtl:12px` used 32 times. **What remains is the document:** §11 still reads *"Radius 0 everywhere"* in the owner’s live design project, and every local citation of it (e.g. `asset_library_window.gd:889`) still quotes the superseded rule. **Correcting §11 itself is a write to the owner’s design project and was NOT done unasked** **PART 1 DONE 2026-09-08 — the document no longer contradicts the canvases.** §11’s closing sentence read *"No fills on panels: regions are separated by hairlines only. Radius 0 everywhere."* It now reads **"No fills on panels"** followed by a block recording **both defects it caused**, so the next port cannot re-infect itself from the same sentence: that the panel restriction was read as *no fills anywhere* while §10, §7 and §11’s own type rule all require fills on interactive elements; and that **radius 0 is superseded by the owner’s 2026-09-07 ruling**, with the canvases named as the authority (**`design/mcp-2026-09-07/`**). **The row’s own citation was wrong and is fixed here**: the file is at the repository **root**, not `docs/` — a path that would have sent a lane looking in the source project’s documentation. **WHAT IS LEFT, and why it was NOT swept:** (a) the *"no fills anywhere"* bug in the shell, which is a behaviour change across interactive elements and needs its own before/after; (b) the **49 superseded-canvas citations**. **(b) is deliberately not a rename.** The four shell comments that cite §11 — `dcc_theme.gd:992` and `:1460`, `dcc_shell.gd:4189`, `dcc_widgets.gd:345` — **already reason about this exact tension** (*"is a rule about the desktop artboards"*), so each citation needs a judgement about what the current canvas says on that point. A blind sweep would replace considered reasoning with a wrong pointer **PART 2 DONE 2026-09-08 — 13 fill bugs across 9 files, and the root cause is one Godot rule.** **A `Button` with `flat = true` silently voids EVERY stylebox override, `hover` as well as `normal`** — proved on an isolated probe both ways, and independently by the verifier (identical override: `flat=true` renders the bare mode colour, `flat=false` renders the fill). So a fill that was fully coded, and read correctly in review, **had never drawn a pixel**. **Verified with a stronger discriminator than the lane used**: modal colour of the interior with a 3 px ring excluded, so border and ink cannot contaminate the sample — faction roster selected `#ede5da` over **85 %** of the interior vs `#f4f2ee` unselected; Travel Library active tab `#f3ede2` at 87 % vs `#fbfaf7`. **Both are FILL differences, not ink.** **Mutation kills it**: reverting one `flat=false` makes the probe report exactly the pre-fix failure — *ink-only* — then restored, md5 intact. **No panel gained a fill**, which is the half of §11 that is correct: zero `panel` overrides, zero `PanelContainer` touched, all 13 sites are `Button`s. **`_ds03fit_probe` passes WITH the changes** (0 over-wide leaves, 10/10 pairs, both densities), so the new styleboxes cost no layout. **Two honest narrowings.** The *"a real fill sat dead behind `flat`"* summary over-generalises: at **four** sites no fill existed at all and the treatment is NEW and derived (`_sculpt_stamp_row`, `_stops_row`, the stage-matrix cell, and the hover/pressed halves of the rail row) — the per-row evidence says so plainly, only the top line was wrong. And **9 of 13 cited line numbers are stale by 4-80 lines**, though every named SYMBOL is correct. **§7’s literal named-ramp popover was declined, with a reason**: it does not exist as a row UI, §7’s own correction folds it into a dropdown (`render_workspace.gd`), and the canvas’s popover markup is annotated *"not wired"* in its own text — building one is new feature work, not a dropped fill. The surviving analog, the CARTO ramp Stop-editor row, was fixed and pixel-verified instead. **All ten touched files re-parse clean from the project root.** **STILL OPEN: the 49 superseded-canvas citations**, deliberately not swept for the reason already recorded — the four shell comments citing §11 reason about the tension themselves **UPDATE 2026-09-12 — the count is 51, not 49, and the sweep stopped at its gate.** Re-derived by a verifier: **51 lines in 20 files** — `DCC Shell` 22 in 12, `Android Phone` 17 in 8, `Menu Structure v` 11 in 8, `Journey Planner DCC` 1 in 1. A lane examined 5 files and 12 citations, found three places where the current canvas says something different, and **stopped before any edit**, as its gate required. **Checked at the canvas by the main loop, only ONE of the three is shell drift:** (a) **Refine detail** — the word appears in none of the three current canvases, yet the shell builds the button; filed in §3.1 as an owner question. (b) **Phone tabs** — the shell already builds the canvas’s four function-keyed tabs; only a comment is stale, plus two icons (row in §2.7). (c) **"The only rounded surface"** — `pill()`’s comment is false against a canvas that rounds dozens of surfaces, but the shell already rounds phone surfaces (the sheet at `_pscale(22)`, 23 `pill()` call sites). **So resume the sweep with category 2 split** into *the shell disagrees with the canvas* and *only the comment does*: the first is a parity row, the second is an edit **2026-09-13: a lane’s summary blockquote was refuted and reverted** (its counts mixed grep patterns and probe files — shipping-code citations at HEAD are 233/52/22 by either pattern, 214/35/7 by prefix). A verifier re-opened 26 of the 52 citations and their ABSENT/HIST/RESOLVED verdicts hold (app.gd:2562, asset_library_window.gd:20, browse_dialog.gd:5, data_manager_window.gd:17, dcc_theme.gd:431/1201/1225/1501, dcc_widgets.gd:1313, dcc_shell.gd:526/685/2972/7260, viewport_host.gd:982, dcc_icons.gd:130, phone_menu.gd:4, travel_library_window.gd:5); the edit itself is still to do **2026-09-13, second attempt stopped and its conclusion refuted.** The verifier established what the edit actually is: (1) **the remaining edit is the supersession note** in `DCC_SHELL_SPEC.md` after the "Reference mockup" table — its line still names `Cartalith DCC Shell.dc.html` with no note that the 2026-09-07 canvases supersede it; (2) **six citations are parity differences, not stale pointers** — `app.gd` "Open project dialog 1920", `asset_library_window.gd`, `browse_dialog.gd`, `data_manager_window.gd`, `dcc_theme.gd` (Theme’s dialog), `dcc_widgets.gd` — their screens are drawn in the old DCC Shell canvas and absent from the current `Cartalith DCC Environment.dc.html` (0 hits for asset library, data manager, select folder); list them as parity rows rather than rewriting them into agreement; (3) the Journey Planner citation IS superseded (all three 2026-09-07 canvases carry planner content). **Counts:** state the counting method — the 233/52/22 and 214/35/7 figures do not reproduce under any tried pattern (colon, backtick, word-boundary, any-canvas; with or without probes) |
| **The app never idles: it redraws at 60 fps with nothing changing, and that is the power draw** — **FIXED 2026-09-23, DESKTOP-VERIFIED, NOT YET RE-CONFIRMED ON THE DEVICE** | `ANDROID_BUILD_SCOPE.md` | medium | **Owner, 2026-09-07, on the OnePlus 12: *"There is something that keeps the cpu/gpu active after the initial rendering."* GUI priority explicitly lifted 2026-09-23, cleared to pick up.** **Root cause, measured not assumed**: neither a stray `SubViewport` left at `UPDATE_ALWAYS` (the shipped shell creates none) nor an unconditional per-frame redraw call (every real per-frame `_process` in the shell already only redraws on an actual change) — `project.godot` simply sets no low-processor mode, so Godot's main loop renders every iteration regardless; confirmed by instrumentation showing every one of 294 iterations drew a frame before the fix. **The fix**: `app.gd::_ready()` now sets `OS.low_processor_usage_mode = true`, gated on the app owning the real window (`get_parent() == get_tree().root`) rather than set project-wide — **this gating is load-bearing, not cosmetic**: forcing the mode on unconditionally was tested directly and makes a probe-hosted app (~140 probes mount it in a `SubViewport` pinned to `UPDATE_ALWAYS`) draw 0 frames regardless of that setting, which would stall or stale every probe awaiting `frame_post_draw`/`get_texture()`. **Desktop-measured**: at rest, 0 draws/s on both the picker and the map (CPU ~1–1.6% of a core, against ~8% and ~46% before); animated layers (wind, water) hold their full ~58fps rate throughout; positive controls confirm a real change still draws. Six pre-existing `UPDATE_ALWAYS`-dependent probes re-run and confirmed passing. **Verified independently**: full diff read (the gate condition confirmed exactly); a new windowed probe re-run directly, reproducing the reported numbers closely; `cargo test --workspace --no-fail-fast` unaffected (no Rust touched); `project.godot` confirmed unchanged. Commit `1176acb`. **Genuinely still open**: the original defect was measured ON THE DEVICE (SurfaceFlinger `--latency`, real OnePlus 12); the fix is desktop-proxy-measured only — the natural next step, not yet done, is repeating that same on-device measurement against a rebuilt APK to confirm the real hardware improvement, not just the desktop proxy. |
| ~~**Input controls shrank the desktop rows, and on tablet the CheckBox GREW 4 px**~~ — **CLOSED 2026-09-21 (verified, evidentiary gap closed, no code needed)** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | **The blocking probe limitation this row named was already fixed 2026-09-07 (`7ba28f0`, the same day, 1h28m after the change this row is about) and simply never re-verified.** `_ds03shot_probe.gd` has carried `--vp`/`--comp` flags and an intrinsic composition classifier since that commit — independently confirmed by two agents this session, run live rather than assumed: `--comp tablet` genuinely reaches tablet composition (`intrinsic=tablet`), and every sampled `CheckBox` measures `size=(40.0, 44.0)` against the desktop/pointer post-reflow `(30.0, 24.0)` — the row's disclosed 36→40 growth, now backed by a real dump instead of an assertion. No source changed. |
| **The map thumbnails are hypsometric only — no hillshade, and the reason is arithmetic** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | **Declined deliberately by the THUMBNAILS lane 2026-09-07 with the reasoning at `_hypso()`, and it is worth keeping because a later pass will otherwise "add the missing half" and get it wrong.** `render_height_tile_rgba`’s `exag` (3.4) **scales a difference between ADJACENT cells.** A 96-wide sample of a 2048-wide world takes that difference across **~21 cells**, so the shade would be over-driven by a factor that is the world’s own width. **Correcting it means dividing `exag` by a stride nothing in the reference divides it by** — which is a deviation from the ported algorithm, not a bug fix, and needs to be recorded as one. **~12 lines if wanted.** The colour half is byte-exact against a numpy golden computed outside GDScript (0/6912 mismatched channels), so this is an addition, not a repair **2026-09-13: this is an owner "if wanted" call, not a canvas gap.** The DCC Environment canvas’s `pickerWorlds` thumbnails (~1898-1901) are flat three-stop CSS gradients with no relief, so the canvas does not ask for hillshade; the row’s own "~12 lines if wanted" stands |
| **Two handset route panes still overflow — and it is the BODY, not the footer** | `ANDROID_UI_SPEC.md` | small | **Diagnosed and declined deliberately 2026-09-07, then independently confirmed.** At 500×1080 against a room of **376 px**: `export_gis` body asks **476**, `export_maps` body **386**. **No footer assertion is red anywhere** — their footers ask 94 and 131. **So last batch’s footer remedy does not reach these, and neither does this batch’s wrap.** **The obvious fix is measured and wrong:** `AUTOWRAP` on the hint collapses the minimum to **1**, which is the `clip_text` trap in another costume — the lane measured that rather than discovering it later. **Needs a real answer for a body that is genuinely wider than a phone**, which is a design question, not a container swap **RE-ROUTED 2026-09-13:** both panes are `DataManagerWindow` routes (`export_maps`, `export_gis` in `data_manager_window.gd` ROUTES), not the journey planner — a lane was sent to the wrong file. The Android canvas’s `data-io` screen says route configuration is "desktop-parity mock in this prototype"; its `data-tiles` Scheme segments are XYZ / TMS / WMTS against the port’s single "grid + index.json" |
| **TABLET PORTRAIT: five Planet sliders are 290 px in a 232 px dock — the canvas answers with a 2-COLUMN GRID, not narrower rows** | `TABLET_UI_SPEC.md` §2.4 | medium | **Fully specified 2026-09-07. Nothing here is a guess; the two diagnoses I guessed at are recorded as retracted at the end.** **THE ROWS, named by the probe:** `Continentality`, `Fragmentation`, `Tectonic energy`, `Ocean depth`, `Hotspot density` — the five Planet sliders, **290 px each**. **THE ARITHMETIC:** `ROW_LABEL_W` **132** + tablet `slider_track_w` **90** + `ROW_VALUE_W` **44** = 266, plus separations = 290. **On tablet the TRACK grew (78→90 via the role) while the label and value stayed at their desktop `const`s** — `ROW_LABEL_W` and `ROW_VALUE_W` are plain constants, not role-resolved. **THE PROPAGATION:** `left_dock 331` ← `HBox 330` ← `VBox 324` ← **`ScrollContainer` with `h_scroll=DISABLED`** ← `VBox 316` ← `WorldWorkspace 316` ← the five rows. The scroller folds a child minimum outward, so the dock’s 232 role cannot win. `_scroll()`, `dcc_shell.gd:3963`. **WHAT THE CANVAS ACTUALLY DRAWS, and it is not narrower rows:** §2.4 records the open pipeline body as `padding:4px 12px 12px`, a needs note, then **a 2-column grid of `min-height:var(--rowD)` = 44 cells, sliders**, then `Run stage NN`. **At 232 px each cell is roughly 100 px** — so the tablet reflow is a LAYOUT MODE, not a constant change, and DS-03’s *"keep everything; reflow only"* is exactly that. **Two things NOT to do.** Do not enable horizontal scrolling — it stops the fold and hides the overflow behind a scrollbar the canvas does not draw. Do not shrink `ROW_LABEL_W`/`ROW_VALUE_W` globally — they are desktop constants and the desktop is conformant. **Retracted, kept because each was believed before it was checked:** *"the role is not applied"* (it is — `_left_width=232.0`, `ld_cms=232.0`) and *"hidden workspaces size it"* (`CivilizationWorkspace` is `own_vis=false`; that came from a probe reading `is_visible_in_tree()` where a container minimum uses the child’s **own** `visible`) **BATCH 2026-09-13 (wf49) — REVERTED, patch kept** (`.claude/resume-2026-09-12/grid_reverted_2026-09-13.patch`). **This row’s reading of the canvas is wrong, and my brief carried it:** the canvas’s 2-column grid (`st.grid`, `Cartalith Tablet.dc.html` ~159-168) holds **tap-to-cycle toggle/segment pill cells** and never a slider; sliders are **full-width two-line cells** (`st.sliders`, ~169-174: label and value on one line, the track below). The five sliders are `world_structure` sliders under *Generate*. The lane built the two-line slider cell (portrait dock widths 331/341/334/331/350 → 257/239/251/232/244/232/232/232/232) — **4 of 9 WORLD categories still exceed 232**, from section-header labels (205/222 px) and 14/12-padded VBoxes. Refuted as blocking: it built past its own G1 HIT; the layout is chosen at build time, so a rotate keeps the wrong one (landscape boot → portrait reads 331); right-click-reset moved from the row to the track. Desktop, laptop, landscape tablet and phone geometry were confirmed identical. **Next:** two-line slider cells from the canvas, rebuilt on orientation change, plus the header/padding overflow — briefed from the canvas markup, not this row **SECOND ATTEMPT 2026-09-13 (wf50) — REVERTED as blocking, patch kept** (`.claude/resume-2026-09-12/grid_v2_reverted_2026-09-13.patch`). **What worked:** the canvas’s two-line slider cells (`st.sliders`), rebuilt on rotation both ways through `_on_window_resized` (verified 1280×800 → 800×1280 → 1280×800 and the reverse), every slider’s type, range, tooltip, signals and right-click reset intact; portrait widths 331/341/334/331/350/232/232/232/331 → 232/239/232/232/244/232/232/232/232. **Why reverted:** an unconditional `clip_text` added to `DccTheme.header()` (8 callers; dock heads sit beside an `EXPAND_FILL` spacer) and to the stage `name_label` made **dock titles draw 1 px wide on every form factor** (WORLD, CIVILIZATION, CARTOGRAPHY, SAMPLE vanish) and cut stage names to 99 px on desktop — the lane reported R1 NOT HIT from fixed dock widths without a HEAD run; it also declared R2 HIT and shipped past it. **Still over 232:** Terrain 239 and Climate 244. **The drivers are `DccTheme.header()` labels and the hand-built stage names, not `DccWidgets.section()`/`note()` as the brief said.** **Next:** re-apply the patch, scope the clip to tablet portrait (or to the stage name only) and prove dock titles unchanged against a HEAD copy at every form factor **THIRD ATTEMPT 2026-09-13 (wf53) — COMMITTED; two residues keep the row open.** Verified: the canvas’s two-line slider cells in tablet portrait; header clipping is now opt-in (`DccTheme.header(…, elide)`, off by default, never on a dock head) and the stage name clips only in tablet portrait; `DccWidgets.toggle()` drops its inert spacer in tablet portrait. Portrait widths 331/341/334/331/350/232/232/232/331 → **232/232/232/232/244/232/232/232/232**; full dumps of every visible control under both docks at 1920×1080, 1366×768, 1280×800 and phone identical to HEAD (24 108-28 030 lines each); gestures (drag, drag_ended, right-click reset on label, track and readout) verified with routed pointer input. **Still open:** (1) **Climate 244** — the *Lapse rate* readout `6.5°C/km` (63 px, never clipped because it is a number) beside the 132 px label floor; closing it needs a shorter unit string from `params.rs` or a narrower label floor in portrait. (2) **Boot landscape, rotate to portrait: Terrain reads 239** — `DccWidgets.toggle()` decides the spacer at build time and nothing re-runs it on rotation. Also: 276 CIVIL section headers now clip in portrait (widths unchanged), and the `--rotate` probe prints *0 FAILURE(S)* after a SCRIPT ERROR (see the probe-honesty row) |
| **The label fix is INVISIBLE until `CARTO ▸ Generate labels` has been run** | `TERRAIN_APPEARANCE_SCOPE.md` | small | **Raised by the lane and verified 2026-09-08, and it decides whether the owner sees the fix at all.** `_draw_labels()` renders what `labels_generate` produced, and that has **exactly one shell call site**: `cartography_workspace.gd:2400` via `bridge.labels_generate`. `labels.rs::generate_labels` emits a **Settlement-class label per settlement** (`for s in world.settlements`, `:1321`), which is how settlement names reach that path at all. **So if the owner saw blurry names WITHOUT ever pressing Generate labels, they were looking at the PIN path** — which is already crisp (1.11 contrast retained at 3×, fixed 2026-08-24) — **and the fix is correct but is not what they saw.** **Ask before assuming the report is closed**: it is one question, and getting it wrong means declaring a defect fixed that the owner will see again |
| **Labels have no zoom-compensation term — a "fixed"-size label still scales with live camera zoom, same as "zoom" mode** | `map_overlay.gd` (labels/font follow-up) | medium | **Found 2026-09-21 while building the labels/font-selector work above (commit `f826932`) — a real gap between what the owner asked for and what `size_mode` actually does, disclosed rather than shipped as complete.** `_label_font_px()` never reads `_camera_zoom` in either `size_mode` branch — confirmed at the symbol (zero references) and independently reproduced by the coordinating session. What `size_mode` actually gates is the grid-to-window letterbox fit (`rect.size.x / _gw`, how a label's base size differs across worlds/window sizes), never live interactive zoom. Settlement pins already have the missing mechanism — `_civ_zoom_k()` divides out `_camera_zoom` so a pin holds roughly constant on-screen size across zoom — labels have no equivalent term, confirmed unaffected by `size_mode` either way; `_labelblur_probe.gd`'s own pre-existing assertion states outright that a "fixed"-mode label's "drawn width still scales with the camera and nothing else." **The fix**: add a `civ_zoom_k(_camera_zoom)`-style term to `_label_font_px()`'s `"fixed"` branch. This changes already-shipped, `_labelblur_probe.gd`-verified rendering behavior for every label using either mode, not just settlement/POI, and that probe's own assertions (which currently pin "scales with the camera" as the correct behavior) need rewriting alongside it — a real, if small, verified-behavior change, not a bug fix in isolation. Flagged to the owner; awaiting their answer on whether to build it. |
| ~~**Implement the owner's re-sorted PC left rail**~~ — **CLOSED 2026-09-21 (verified) — Ruling L complete, all three tabs** | `LARGE_ITEM_RULINGS.md` (Ruling L) | large | **Owner, 2026-09-13.** The full specification is the tree in `design/owner-references-2026-09-12/left_rail_tree_resorted.md`; Ruling L records its grouping rules and four resolved decisions. **Shape of the change:** WORLD becomes 7 PIPELINE categories + 2 SCULPT (Terrain, Biomes), with SCULPT mode as the only gate; CIVIL goes 15 → 13 (Civilizations becomes Populate, Trade folds into Economy, Politics + Simulation become Timeline); CARTO goes 10 → 7 (Style, Relief & light, Colours, Feature style, Layers, Labels, Icons), with every visibility toggle in Layers. **Removed:** the duplicate Populate/Diagnostics rebuild, the never-shown left-dock Biome-paint panel (made real as Sculpt ▸ Biomes, right-dock copy retired afterwards), the second *Network* and *Flows* sections, Resources. **Retarget pass:** cross-links, node-list labels and targets, rail footer TERRAIN → RELIEF. **Before building, read Ruling L’s two notes:** the tree’s *"disabled"* notes on Erode (droplet) and painted lakes are wrong (both enabled), and where the notes and the tree disagree the tree wins (owner: *"Tree is leading"*), so Center landmasses goes to Generate ▸ Run. **Mostly moves, not new controls** — split it by tab (WORLD, CIVIL, CARTO are separate workspace files) so lanes stay file-disjoint, and probe the move with a before/after control census so nothing silently drops **CIVIL STATE, 2026-09-13 late: built, held, not committed.** The CIVIL half is in the working tree (`civilization_workspace.gd`, `infrastructure_workspace.gd`, `dcc_shell.gd`, `phone_menu.gd` and six probes, plus the untracked `_civilcensus_probe`), saved as `.claude/resume-2026-09-12/rail_civil_held_2026-09-13.patch`. Its adversarial panel confirmed two behaviour bugs, both from a boot-time `app.apply_domain_mode(civilization, factions)` in `CivilizationWorkspace._build()`: with the Way tool armed, the first rail press, a `select_domain_category` jump, or phone MORE ▸ Simulation model keeps or drops the Commit/Discard row differently from HEAD (`app.gd::_on_workspace_changed` compares against `_last_workspace_mode`). **Expected fix:** remove that call, restore HEAD’s entry mode, and update the census probe’s E0/E1 lit-mode checks. Also confirmed and to fix: the Culture roster button moved out of Profiles without an owner instruction (put it back); three category-level expanders draw flush instead of at the section inset; Travel draws 241 px at 800x1280 against HEAD’s 232. **Owner call owed:** with the fix, CIVIL entry lights Landmarks on the rail while Populate opens (move plan C1). A Fable 5.1 fix run was started and stopped by the owner before the builder edited any file. **CIVIL HALF CLOSED 2026-09-20 (commit `5f839d7`):** both behaviour bugs and all three layout defects fixed and independently re-verified on all five form factors (271 HEAD diffs before the fix, 0 after). **Owner call C1 is still open**: CIVIL entry lights Landmarks on the rail while the tree’s Populate opens — left standing, per the plan, as the owner’s to answer. **WORLD HALF CLOSED and CARTO HALF CLOSED 2026-09-21 (commit `04b3b27`), completing Ruling L.** Two agents dispatched file-disjoint (`world_workspace.gd` / `cartography_workspace.gd`) were each cut off mid-task by a session rate limit at ~95% done, with one clearly diagnosed bug remaining each; resumed rather than discarded, per this session's practice for substantial interrupted work. **WORLD**: the mode pill is now the sole gate (superseding the 2026-09-07 armed-tool mechanism), fixing a real bug the switch introduced (`world/a`'s `RAIL_NODES` entry carried no `shows` key, and `apply_mode()` reads an absent key as "show everything" — correct pre-Ruling-L when Terrain held erosion parameters PIPELINE needed, wrong after they moved to Hydrology/Geology); a new `WorldWorkspace._floor_applies()` override stops the always-one-open-floor inference from re-breaking a 2026-09-05 fix now that both modes gate. **CARTO**: ten categories folded to seven, political layers consolidated back into Layers (Political display retired), Center landmasses in Generate ▸ Run per the tree-leads tie-break. **Found and fixed beyond both agents' own scope**: three dead cross-links in `civilization_workspace.gd` (Factions' Identity-colour button and both Place-an-icon buttons pointed at retired CARTO category names — fixed by the coordinating session per the tree's own Retarget pass) and four stale sibling probes (`_railfold_probe.gd`, `_v3menu_probe.gd`, `_presetgal_probe.gd`, `_leftdock12_probe.gd` — category lists, jump tables, and one inverted test: the Layers/Political-display *split* `_v3menu_probe.gd` used to assert is now a *consolidation*). **Two pre-existing, unrelated probe failures found and left as found, filed separately below**: `_railfold_probe.gd`'s Labels/Icons "inert placement controls" assertions, and `_v3menu_probe.gd`'s Data-menu-popup lookup (keys off "Journey planner" text, which moved to a CIVIL rail node under the *earlier* CIVIL half of this same ruling). Independently re-verified against the exact staged commit: `_worldcensus_probe` PASS, `_cartocensus_probe` PASS, `_leftdock12_probe` 0/24 failures, `_presetgal_probe` 0 failures (7 tiles), `_worlddockb_probe` 0 failures, all 13 touched `.gd` files parse clean. |
| ~~**`_railfold_probe.gd`'s Labels/Icons placement-rule controls are live, not inert**~~ — **CLOSED 2026-09-21 (verified — stale probe, not a bug)** | CARTO / `icon_bridge`, no owning scope document | small | Confirmed genuinely live, shipped under the owner's 2026-09-02/09-03 rulings (`LabelTypography`'s dials, the generated icon-placement pass) three weeks before this probe's §6 was last touched for an unrelated rail-fold rename — the shell's own build code already said "Both dials are live" / "All three are live now" in doc comments right next to the code, only the probe's assertions had drifted. Rewritten to assert and verify real effect (drag a dial, confirm the backing spec/engine value actually changes; toggle a checkbox, confirm its own distinct flag flips). Independently re-verified: `_railfold_probe: PASS`, matching the agent's report exactly. Commit `ab538a1`. |
| ~~**`_v3menu_probe.gd`'s Data-menu-popup lookup is stale**~~ — **CLOSED 2026-09-21 (verified)** | `LARGE_ITEM_RULINGS.md` (Ruling L, CIVIL half follow-up) | small | `_data_popup()` now finds the Data menu structurally — walking for the `MenuButton` whose `.text == "Data"` (set once at construction by `menus.gd`'s own `add_menu("Data", _data)` call) and reading its popup directly — instead of text-matching a submenu row that had already moved once. Second instance of the same failure class this helper's own doc comment already recorded once; picking another row to match on would only relocate the same fragility. Independently re-run: `V3 RESULT PASS (0 failures)`, full windowed run. Commit `bcb3878`. |
| **OWNER DIRECTION: water bodies must be classified by hydrological TOPOLOGY, not size — and today’s rule is size** | `HYDROLOGY_CLASSIFICATION_RESEARCH.md` | large | **Owner-supplied research, imported verbatim 2026-09-08 as `HYDROLOGY_CLASSIFICATION_RESEARCH.md`.** Its central rule: *"A water body’s classification should describe its physical relationship to the world’s hydrological system, not merely its size."* **WHAT THE PORT DOES TODAY, quoted from `build_water_bodies`’ own doc comment (`cartalith-civ/src/lib.rs:526`):** *"distinguishes the open OCEAN (**largest connected below-sea component**) from inland LAKES (**every other below-sea component**, plus above-sea depressions a priority-flood fill pools past `lakeDepth`, gated on local rainfall)."* The vocabulary is three states — `tools.rs:354`: **0 = land, 1 = ocean, 2 = lake**. **So the rule is size-primary.** Not an absolute km² threshold, which is what the paper argues against most directly — it is *relative* size, largest-wins — but size all the same, with **no connectivity, no basin topology, no connection type and no map-boundary state anywhere in it.** **Three failure modes follow directly, and §1 and §8 of the paper name two of them:** **(1)** a world with little ocean and one huge inland basin makes **the lake the ocean**, because largest wins; **(2)** a genuinely marine body truncated by the map edge that is *not* the largest component becomes a **lake** — the paper’s `MAP_BOUNDED / UNRESOLVED` case, which has no representation here at all; **(3)** the Caspian case is unrepresentable in either direction, because salinity is not a stored property and `endorheic` is not a state. **THE COST IS NOT THE ALGORITHM — IT IS THE PARITY BASELINE, and this is the part that needs an owner ruling before any code moves.** `build_water_bodies` **is** `buildWaterBodies` (reference HTML line 5753), and it is **golden-tested** (`build_water_bodies_largest_below_sea_component_is_ocean`). Changing the classification changes `water_bodies`, which feeds `build_biome_raster`, route costing (`RouteContext::water_bodies`), lake labelling (`lake_features` keys on `== 2`) and landmark placement. **A golden re-baseline needs an owner ruling** (`CLAUDE.md`), and the last one granted was spent. **The cheapest honest first step is ADDITIVE and breaks nothing:** keep the three-state raster exactly as it is, and build the paper’s `WaterBody` record beside it — `ocean_connected`, `map_boundary_contact`, `inflow_count`/`outflow_count`, `basin_type`, `salinity` — as derived metadata. **That answers "is this lake actually a truncated sea?" without moving a single classified cell**, and it is what the naming and cartography layers would consume anyway (§10 warns against *"rendering terminology contaminating the underlying physical simulation"*, which is exactly the risk of doing it the other way round) **ADDITIVE FIRST STEP BUILT 2026-09-21 (verified).** `cartalith_civ::water_body_topology(wb, gw, gh, world)` flood-fills the existing, unmodified `WaterBodies::classification` with the same 4-connectivity/world-wrap rule `build_water_bodies`'s own `cc_visit`/`wb_visit` use, and derives `ocean_connected`/`map_boundary_contact`/`basin_type` (`Ocean`/`EndoreicLake`/`MapBoundedWater`) beside it. `inflow_count`/`outflow_count`/`salinity` deliberately NOT attempted — they need a river/flow network this function doesn't carry, disclosed rather than invented. `build_water_bodies` itself untouched; `golden_parity_waterbodies.rs` confirmed byte-identical, no re-baseline. **Not wired into any Godot bridge or UI yet** — pure engine capability. Commit `c6de2a2`. **The large reclassification ask (moving `build_water_bodies`'s own size-primary rule) is UNCHANGED and still needs the owner ruling described above.** **AUTHORIZED IN FULL 2026-09-21 — Ruling Q (`LARGE_ITEM_RULINGS.md`).** ~~CLOSED 2026-09-21 (verified) — landed in `cc834d2`.~~ `build_water_bodies` now selects ocean by boundary contact (world-aware: an X edge counts only when `!world`, Y always counts), regardless of size, with the old largest-wins rule kept only as a disclosed fallback when nothing touches the boundary at all. Two required failure-mode tests added and passing: an interior "huge inland basin" no longer wins ocean by size; a small boundary-touching component is no longer demoted to lake. 13 golden files in `crates/cartalith-civ/tests/` re-baselined, every moved value re-derived from the crate's own new output and disclosed old → new — `settlement_suitability.rs`'s "legacy" proving arm needed a frozen local reproduction of the pre-Ruling-Q function (including the hand-ported `MinHeap`) so it keeps proving against the real JS capture rather than silently drifting. **Flagged, not silently resolved: a real design-fork judgment call.** The research doesn't address toroidal/wrapped-world topology — under `world=true`, X has no real edge at all, so §8's boundary-contact-as-marine-signal reasoning doesn't obviously transfer. The same uniform rule was applied to both `world` values rather than special-casing `world=true` to keep the old behaviour there; this does change `world=true` output (every golden file's `case_1_world_wrap` fixture moved). **This is a candidate for its own owner ruling if the wrapped-world case matters** — filed as a new open item below rather than assumed settled by Ruling Q. **Independently re-verified**: `cargo test --workspace --no-fail-fast` re-run myself, 163/3440/0/33 reproduced exactly; disclosures in `golden_parity_waterbodies.rs` and `golden_parity_settlement_suitability.rs` spot-checked directly against the diff; `project.godot` and the live settings file confirmed unchanged. |
| ~~**Hydrology reclassification's world-wrap judgment call**~~ — **CLOSED 2026-09-21 (verified) — Ruling T built** | `LARGE_ITEM_RULINGS.md` Ruling T | small | Commit `e99c6ac`. `build_water_bodies` now branches on `world`: `world=false` keeps Ruling Q's topology-primary rule unchanged; `world=true` restores the exact pre-Ruling-Q size-primary rule, via `git show c6de2a2` (the commit immediately before Ruling Q) rather than reconstructed from memory. Two load-bearing tests (a positive/control pair) prove the branch actually took effect rather than just compiling. 12 golden files checked against real output, not assumed — 10 reverted where the wrapped fixture's classification actually moved, 2 left unchanged with a disclosure comment where the largest and boundary-touching components coincide by chance, 1 (`golden_parity_faction_aggregates.rs`) untouched entirely since Ruling Q never moved its wrapped fixture in the first place. **Independently re-verified**: `cargo test --workspace --no-fail-fast` re-run myself, 163/3467/0/33 reproduced exactly; both load-bearing tests read directly and confirmed. `project.godot` and the live settings file confirmed unchanged. |
| **OWNER QUESTION: 149 of 200 addon villages have no road, and that is REFERENCE-SHAPED** | `PHASE2_SCOPE.md` | medium | **Measured, not screenshotted, then proven structurally.** Seed 483920 at 512×384: 240 settlements, 44 drawable ways. **Every named tier is 100% reached** — capitals 6/6, cities 2/2, towns 6/6, villages 12/12, non-addon hamlets 14/14. **149 of 200 ADDON villages** (`kind == "hamlet"`, `population == 0` — the tier revealed at `VILLAGE_ADDON_LOD = 2.4`, i.e. **exactly "the ones that appear on zoom"**) have none; worst gap **107.3 cells**. **Missing from the DATA, not merely undrawn** — which was the distinction that decided the fix, and it was settled by querying rather than looking. **Correct by construction, and the verifier proved it independently of the lane:** `lib.rs:2090` builds the topology from `placements`, `:2219` seeds villages, `:2243` does `settlements.extend(villages)`, and `:2418` smooths ways from that same pre-village topology — **so no `RoadEdge` can index an addon village.** The reference agrees: `_civSeedVillages(places, ways, rng, suit)` takes the network as an INPUT, and `_CIV_VILLAGE_CAP = 200` matches the measured n exactly. **So this is a DESIGN QUESTION for the owner, not a defect.** A hamlet of population 0 seeded *from* the road network may reasonably have no road of its own. **Connecting them means feeding villages back into `civ_hierarchical_network_topology`, which is a divergence from the reference and a golden re-baseline.** **No connector was built** |
| **OWNER-REPORTED: the tablet still shows the PC layout — the spec PARKED adoption behind a decision the owner had already given** | `TABLET_UI_SPEC.md` | large | **Owner, 2026-09-07: *"tablet still is the pc layout instead of the design I gave."* Traced through the documents; each is internally consistent and together they are wrong.** **1 — the block, `TABLET_UI_SPEC.md:9-14`:** *"This file is an inventory, not an implementation plan, and no shell code was written by the pass that produced it. The tablet canvas describes a **different shell** from the one this project ships, and **adopting it is an owner decision that has not been made.**"* **2 — but that decision HAD been made, the same day, by the instruction that delivered the canvas:** *"All designs layouts and styles should match 100%. Check all designs, pc, tablet and phone."* **The spec was written AFTER that and still parked adoption.** **3 — what the shell implements instead is ruling DS-03** (`LARGE_ITEM_RULINGS.md`, 2026-09-03): *"Keep everything; reflow only."* That is a **CONTENT** answer — it settles *which controls leave* (none) — and it predates the tablet canvas by four days. **Read as settling the tablet’s SHAPE, it produces exactly what the owner sees: the PC composition, reflowed.** **4 — the asymmetry that makes it visible:** `ANDROID_UI_SPEC.md` drove real phone work all day; `TABLET_UI_SPEC.md` drove none. Same day, same shape of document, opposite outcomes — because one declared itself inventory-only. **MOST OF IT IS RE-HOMING, NOT BUILDING.** §4.1 lists **13 canvas items that already exist**: the horizontal rail IS `tool_options_row` (55 px vs `--railH:56`), the domain rail IS `rail_column` (47 vs 52), measure sub-modes, brush controls, right-dock Sample, the timeline and its six layers, the status bar, the planner (a superset), landmarks, and ~20 of the canvas’s 32 menu rows. **BOTH EXCEPTIONS ARE NOW RULED (2026-09-07, `LARGE_ITEM_RULINGS.md` D and E), so nothing blocks the work.** **D:** `Run stage NN` is **drawn** and wired to a **full** run — visually 100%, no engine work, **and each button MUST carry a tooltip saying it recomputes the whole pipeline**, because the affordance otherwise implies a capability the engine lacks. Partial recompute was offered and **declined as Phase-scale** — do not re-propose it inside a tablet pass. **E:** the tablet shows the **engine’s** stage names, so the guard stays meaningful and **a progress bar reads the same words as the logs and the errors**; a display-map was declined because it buys a resemblance with a debugging cost paid forever. **Genuinely new and buildable:** the portrait/landscape dock split (232/320 — `DccTheme` has no orientation predicate and `ROLE` is a two-column table), `scrPlanner` and `scrPicker` as top-level screens, the tablet palette (11 dark + 14 light tokens differ), `--rCtl:12`/`--rPan:16`, 7→3 menus plus overflow, haptics, stylus pressure. **Sequence, cheapest-visible-win first:** the portrait/landscape dock split (232/320) and 7→3 menus are what read as "the PC layout"; the palette, radii and haptics are cosmetic and independent; `scrPlanner`/`scrPicker` as top-level screens are the largest single item because `DccShell` has no state above the domain layer. **The +285 px portrait overflow is a symptom of the same cause and should fall out of the composition work rather than being fixed separately** **BATCH 2026-09-12 — built, refuted in part, REVERTED with the patch kept** (`.claude/resume-2026-09-12/tablet_2026-09-12.patch`, 772 lines, re-appliable). **What worked:** a tablet-only `ScrollContainer` (horizontal AUTO) around `tool_options_row`, backed by the canvas’s own `overflow-x:auto` (`Cartalith Tablet.dc.html:90`), took portrait 800×1280 from +285 px overflow to 0 and 1024×768 from +61 to 0; and the MENUS lane collapsed the tablet menu bar to ☰ File World Data. **Why it was reverted:** the same patch moved landscape docks 400 → 320 in `ROLE`, but the docks still **draw** 331-382 px on 8 of 10 rail nodes because their content is wider — the lane measured the declared width, not the drawn rect — and three committed probes went red: `_roleresolve_probe` (expects 400), `_rdconform_probe` (`C_RDW_TOUCH` 400) and `_tabletparity_probe` (dock widths plus the pinned seven-menu order and count). **Next:** re-apply the rail fix and the menu collapse with those probes re-specified in the same change; take landscape 320 only after the eight rail nodes’ dock content reflows to fit it **BATCH 2026-09-13 — re-applied WITHOUT the 320 docks; verifier: one real bug, so HELD UNCOMMITTED in the tree for a fix-up lane.** Confirmed: `W_DOCK_TABLET` and `ROLE`’s tablet dock column back at 400; frames fit at 400 docks (800×1280 +285 → min 619; 1024×768 +61 → min 848; 1280×800 848); `_roleresolve`, `_rdconform`, `_dockfit`, PC and phone menu trees unchanged; tablet menus ☰ File World Data with every leaf row (by id) and all 18 accelerators present, each firing once. **The bug:** inside the new `ScrollContainer`, `pad` has no EXPAND flag, so on every tablet frame the options row collapses to its minimum — the spacer goes to 0 px, Bake/Refine and Commit/Discard jump left (x 2179 → 704 at 2560), and buttons shrink 55 → 44 px, top-aligned. **The lane reported G2 NOT HIT; it was hit** — `_ds03fit_probe` finds the band by tree path and read 56 → 55. **Also for the fix-up:** re-point that lookup by member; SS13 compares titles as a set, so a dropped row whose title repeats survived (*Keyboard shortcuts…*, ids 72 and 605) — compare by id and assert accelerator presence; four new comments credit the 331-382 px figure to `_tabletparity_probe` (it was `_ds03fit_probe --force-touch` at 2560×1600) and mis-date a paragraph; canvas gaps: ☰ Theme and Units lose their value suffix, World ▸ Run pipeline lacks the canvas accelerator, Toggle theme is built as a submenu **BATCH 2026-09-13 (wf49) — the rail and menu work is COMMITTED, with its fix-up.** Verified: the tablet options row fills the band exactly as HEAD did (buttons, 55 px heights and spacers identical at 2560×1600 and 1280×800) and scrolls only when it does not fit; `_ds03fit_probe` reads the band by the new `tool_options_bar` member (56); SS13 now counts titles and asserts every desktop accelerator is bound on tablet; World ▸ Run pipeline carries Ctrl+R (no other binding). **Still open on this row:** landscape 320 docks (after dock content reflows); the palette, radii, haptics, `scrPlanner`/`scrPicker`; ☰ *Units* reads `Units   Kilometres` where the canvas draws `Units — kilometres`; the canvas’s flat *Toggle theme* row was declined because flattening makes *Follow system* unreachable on tablet until `command_index.gd` `EXTRAS` carries it. Probe notes: SS13 counts (title, count) rather than (title, id), and `_ds03fit_probe`’s `if band != null` would skip silently if the member vanished **UPDATE 2026-09-13 (run wf51): two clauses above no longer hold.** ☰ carries the canvas’s flat *Toggle theme* row (`Cartalith Tablet.dc.html:1008`); Dark, Light and Follow system moved to ☰ ▸ Preferences ▸ Theme and each is reachable by pointer and found by the index; from Follow system the toggle flips the drawn palette and stores it. ☰ *Units* reads `Units — kilometres` / `Units — miles`, canvas-exact; PC Preferences unchanged. A new `Theme` search pointer (`command_index.gd` `EXTRAS`) also changes PC and phone results for *theme*, *dark* and *follow*, and shipped naming a *View* menu no form factor has — corrected to Preferences by the main loop before commit **Moved here 2026-09-13 from the closed `TABLET_UI_SPEC.md`-decision row, where they were the only record:** the LIVE / DRAFT / ARCHIVE state, the centred armed-tool chip, and the rail undo/redo — see `TABLET_UI_SPEC.md` for their canvas references |
| **RULED — the invisible OFF switch track is a CANVAS defect; fix the reference, NOT the shell** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | **Owner ruling C, 2026-09-07 (`LARGE_ITEM_RULINGS.md`). This row produces NO code change, deliberately.** On the light palette the OFF track is `#f4f2ee` against a `#fbfaf7` ground — **1.03:1**, so the track is invisible and the control reads as a floating dot rather than a switch. Visible on the planner’s *"Re-pack per stage"* and *"Auto-promote Walking"* rows. **`ENV:1354` specifies exactly this.** **Ruled: take it back to the designer.** Parity with the canvases is the standing definition of done, so patching the shell around a canvas defect would put the two out of step and guarantee a later conformance pass reverts it. **Same principle that settled the radius question: one source of truth, and when it is wrong you change it there.** **Until the canvas is corrected the shell keeps drawing `ENV:1354` as written**, and `_cklight_probe` pins it, so the behaviour cannot drift while the question is open. **DO NOT "fix" this in `dcc_widgets.gd`.** The action is: raise it with the design instance, correct the canvas, re-import to `design/mcp-2026-09-07/`, and let the shell follow |
| **The `--generate` leg of the capture probes crashes the GPU — three tablet figures are unconfirmed** | `TABLET_UI_SPEC.md` | medium | **The verifier could not reproduce three of the PROBE-SIGHT lane’s numbers 2026-09-07, and said so rather than passing them through.** `--generate` died **twice** with `VK_ERROR_DEVICE_LOST` / signal 4 — a GPU TDR with the engine’s `wgpu` compute and Godot’s Vulkan device both live. **No script fault; every non-generate leg runs clean**, and the lane’s own `tablet_800x1280.png` (770 KB) exists, so it succeeded once for them. **UNCONFIRMED and to be treated as such:** `status_row` min **1103**, floor **1147**, overflow **+347** with a world loaded. **The empty-shell figures ARE confirmed** — `tool_options_row` min 1057 + 28 margins = **1085**, and rail 47 + left 331 + vp 474 + right 232 = 1084 against an 800 px frame = **+285 overflow, 45 clipped text controls.** **Two hardware devices contending for one GPU is the likely cause and is itself worth knowing** — it will recur in any probe that generates a world while rendering one |
| **The planner fix lands the user on a sheet that covers the tab bar it was reached from** | `ANDROID_UI_SPEC.md` | small | **Found by the verifier 2026-09-07 while confirming the planner fix — created by it in the sense that the PLAN tab now reaches a surface that hides the PLAN tab.** The phone left-dock sheet is **full-rect**, so with the planner open MAP / GENERATE / PLAN / MORE are entirely hidden; a tap at the PLAN tab’s own coordinates hits form content instead, and only the sheet’s small ✕ exits. **This is a pre-existing property of the sheet** — `MORE ▸ Civilization` does the same — **so the fix did not introduce it, it made it reachable in one tap.** Whether the phone sheet should stop short of the tab bar is a design question that now applies to every sheet, not just this one |
| ~~**A zoom notch costs a 0.87 s frozen frame, 7× the recorded figure**~~ — **CLOSED 2026-09-21 (verified) — the frozen frame is gone, ~13× reduction** | `ANDROID_BUILD_SCOPE.md` | medium | **Measured on the 6T 2026-09-07.** Notches 1-2 clean, notch 3 stalled at max 868.16/884.75/884.88 ms across 3 runs — 7× the 2026-08-25 recorded figure of 117.0 ms. **Re-measured 2026-09-21 on the same device, a fresh build carrying every commit since including LOD-D6 (`c74a150`).** Third-notch max is now **66.77 ms, median of 3 repeat runs (66.76..66.79 ms)** — independently cross-checked by the coordinating session against the agent's raw log, exact match — a **~13× reduction**, now *better* than even the 2026-08-25 baseline. Reproduced across ocean-centred, land-centred (ruling out a cheap-tile confound) and a late-window pass (ruling out the stall being pushed past the measurement window) — zero runs of any shape produced a frame over 151 ms across 43 measured notches. Both controls held: positive (1.86-1.91M of 2.53M px changed per notch, confirming a real redraw) and negative (tap suppressed → 0 px changed, max 16.9-33.4 ms — the harness's own `screencap`/`dumpsys` cadence is not the source of jank). **Attribution caveat, stated honestly**: the APK carries every commit since 2026-09-07, not only LOD-D6; no with/without pair was built on-device, so the stall is confirmed gone in a build containing LOD-D6, not proven caused by LOD-D6 alone. Device left clean: pre-existing save untouched, test world generated fresh and never saved, app left on MAP tab at fit zoom. New, different-shaped stutter found in the same pass — see the row below, filed separately rather than reopening this one for a different defect. |
| **A zoom notch still costs visible stutter (not a freeze) — 16-23 frames over one vsync, worse on later/land-centred notches** | `ANDROID_BUILD_SCOPE.md` (zoom-notch follow-up) | small | **Found 2026-09-21 in the same pass that closed the frozen-frame row above — a different defect shape, not a residual of that one.** With the frozen-frame stall gone, notches 2 and 4 in the ocean-centred sequence still show 16-22 of 125 frames over one vsync (p99 ~50 ms); land-centred notch 5 is worse — 48-57 of 125 frames over vsync; a late-window check (settling 3.2 s post-tap) still shows 56-61 over-vsync frames for notches 2/4, so the stutter band persists for seconds, not one dropped frame. Real, pixel-controlled (positive control held on every notch), not a harness artefact (negative control with the tap suppressed shows 2-5 over-vsync frames, clean). Not yet investigated for cause. |
| **Zoom is clamped at the top of its range while the badge keeps advancing** | `ANDROID_BUILD_SCOPE.md` | small | **Measured on the 6T 2026-09-07 with a pixel control, which is what makes it a finding rather than an impression.** A notch reading **z8.3 → z11.2 changed 686 pixels** screen-wide (the diff bbox spans the map) against **1.2-1.4M px for a mid-range notch**; a further tap at z11.2 changed **zero**. **Pan (90.7% of px) and fit (1 881 294 px) both redraw normally in the same state**, so it is specific to zoom at the cap and not a frozen view. **So the badge is reporting a zoom the renderer is not applying** — a readout asserting a state that is not true, which is the class this project keeps finding **RE-MEASURED 2026-09-12 on desktop, and it does not reproduce as filed.** Windowed, default 2048×1311 / 800 km: the badge and `_zoom` are **both 160 at the cap**, and aimed at terrain every notch up to the cap moves pixels (z160: 24 103 px). From the default centre the view goes quiet from about z73 — but a 12 000 px pan then moves 851 583 px, so it is an **empty, detail-free patch, not a frozen view**; the phone’s z8.9 → 12.07 notch moved 679 361 px. Lane and verifier agree on the measurement; the verifier refused the *already-done* label only because a missing desktop repro does not prove the device fine. **The on-glass zero-change is most likely a featureless patch at deep zoom**, consistent with the LOD colour gap filed in this section. **Next:** one device check aimed at terrain; close the row if it moves **2026-09-13:** nothing new measured on desktop (the lane restated the 2026-09-12 re-measure); device-gated. **Device pass owed — the 6T is connected again**. ~~**CLOSED 2026-09-21 (verified) — confirmed not a bug on this build.**~~ Real device check aimed at land, per the row's own stated next step: 16 zoom notches over forested terrain with roads/rivers/settlements, badge read from the app's own on-screen readout. Every notch up to the cap (13 of 16, badge 338%→16000%) moved ≥98% of a 1.5M-px diff region; past the cap, badge and pixels stop TOGETHER (0 px, 3 notches) — the precise opposite of the filed defect ("badge advances, pixels don't"). Renderer confirmed live even at the cap: one zoom-out then one zoom-in from the frozen state moved real pixels both ways and returned a byte-identical frame. Both controls held (pan 99.999%, reset-view verified exact). **Left genuinely open, not claimed**: whether 2026-09-07's zero was a featureless-ocean patch (an ocean-sweep attempt drifted ashore by notch 3, inconclusive) — a more likely lead, unconfirmed: 2026-09-07's own numbers are quoted in ~11× multiplier units while today's cap is 160×, suggesting the zoom range was extended since and the old measurement is against a different build, not necessarily the same bug. One world/one seed — this project's own "one world is one sample" rule, so a second-seed sweep would be the belt-and-braces follow-up if anyone wants it. |
| **A drag on the map does nothing until the hand tool is armed, with no on-screen cue** | `ANDROID_BUILD_SCOPE.md` | small | **Found 2026-09-07 because it silently produced two invalid measurements before a pixel control caught it** — 0 of 1 468 800 px changed on two separate swipes inside the map area. With the tool armed the same swipe moves **1 370 640 of 1 512 000 px (90.7%)**. **May well be intended for a tool-based shell**, and the question is not whether the modality is right but that **nothing on screen says which tool is armed or that one must be**. On a phone there is no cursor to change shape, which is the desktop’s cue **DESKTOP HALF BUILT 2026-09-13 (verified):** the map overlay shows a drag cursor while pan mode is on (`set_pan_mode()`; real hover: cursor shape 6 on, 0 off; `_pancursor_probe`). **The touch half remains** and needs a design: no canvas draws a neutral or failed-drag hint beyond the navpad pan chip’s armed paint, which already exists |
| ~~**Boot to a painted welcome screen varies from ~6 s to 15.7 s and is not a polling artefact**~~ — **CLOSED 2026-09-21 (verified) — the slow boot is real and reproduces every time; the VARIANCE does not** | `ANDROID_BUILD_SCOPE.md` | small | **Measured 2026-09-07; reported as a range rather than a point estimate, which was the right call.** **Cold activity first frame is stable and fast** — `am start -W`, `LaunchState COLD`, **263 / 273 / 316 ms** (verifier: 272 ms). **Boot to a painted, usable welcome screen is the number that moves:** painted by ~6 s on the session's first launch, but **NOT painted at t+7.6, 7.7 or 10.6 s in three single-shot checks**, and 15.21-15.70 s in a polled run. **Re-measured 2026-09-21, 15 cold starts, independently spot-confirmed by the coordinating session** (one cold start: `TotalTime: 274 ms`, matching exactly): activity first frame **259 ms median (243..284 ms)**, unchanged from 2026-09-07. Painted welcome screen (a content-specific accent-pixel detector, not "non-black" — the naive check scored a false positive on Godot's own boot splash) is **15.15–15.65 s across 9 single-shot cold starts with zero overlap** — perfectly monotone, no variance, and the polling-artefact question the row itself asked is answered: no, polling inflates the reading by at most a few hundred ms. **So the ~15.4 s slow boot is real and consistent, not the 6–15.7 s spread originally filed** — the likely explanation for the old "~6 s" case, flagged as a suspicion not a finding: `OnGodotMainLoopStarted` lands at 6.01 s median, exactly when Godot's own boot splash first appears, and a black-screen-only detector (the same trap this pass's own first attempt fell into) would call that "painted." **Root cause measured, not guessed**: `/proc/<pid>/task/*/schedstat` shows one core CPU-saturated on the engine's `GLThread` for the entire ~9.4 s gap between `OnGodotMainLoopStarted` and the real paint — a synchronous main-thread stall in the main scene's own startup, not I/O, not a network call, not an asset unpack. The specific GDScript responsible was NOT identified — `simpleperf` refuses this build (`Permission denied`, the release APK is not profileable, independently confirmed by the coordinating session via a failed `run-as`). **New row filed below for the real defect this pass actually found: a ~9.4 s unattributed main-thread stall during boot, Godot's own stock splash on screen with no progress indication the whole time.** |
| **Boot has an unattributed ~9.4 s main-thread CPU stall between engine start and the painted welcome screen** | `ANDROID_BUILD_SCOPE.md` (boot-time follow-up) | medium | **Found 2026-09-21 closing the row above.** `schedstat`-measured: one core saturated on `GLThread` for the whole gap between `OnGodotMainLoopStarted` (6.01 s median) and the real paint (15.4 s median) — CPU-bound, not I/O/network/asset-unpack. The release APK is not profileable (`simpleperf`/`run-as` both refused, confirmed independently), so the responsible GDScript was not identified. A first-run user watches Godot's own stock blue-robot splash (this project sets no `boot_splash` keys) for ~9.4 s with zero progress indication. Needs either a debuggable/profileable build to attribute, or a manual bisection of the main scene's own startup sequence. |
| **OWNER-REPORTED ON GLASS: the generation menu cannot be found on the phone** | `ANDROID_BUILD_SCOPE.md` | medium | **Owner, 2026-09-07, using the APK on the 6T: *"I can't find the generation menu"*.** **It exists in code:** `PHONE_TABS` reads **MAP / GENERATE / PLAN / MORE**, and the empty-shell pass measured the GENERATE tab’s sheet carrying an enabled **392×126 px `GENERATE WORLD`** button that produces a world when pressed. **So this is a REACHABILITY defect, not a missing feature — and that is the worse kind.** **Our method cannot see it:** every phone check this session ran on the desktop under `--force-touch`, which boots the phone *composition* with synthesised pointer events, and the probes press controls via `pressed.emit()` or by calling `_set_*_open(true)` directly — **bypassing the whole touch path**. Investigate on glass, by tapping only what is visible from launch |
| **METHOD: the on-glass method has now been RUN once — it works, and it is calibrated** | `ANDROID_BUILD_SCOPE.md` | medium | **Named 2026-09-07 after the owner found two defects in minutes that a session of probes did not.** `--force-touch` on the desktop boots the phone composition with `device = -1` synthesised pointer events, and probes drive controls through `pressed.emit()`, `item_selected.emit()` and direct `_set_*_open()` calls. **That proves a screen renders and its handler runs. It cannot prove a finger reaches the handler, and it cannot prove a user can find the route at all.** `MOUSE_FILTER`, scrims, gesture handlers and hit areas are all downstream of where those probes inject. **Two rows already said so and were not joined up:** the sheet-scroll row records *"NEITHER HALF REPRODUCES IN THE SHELL; STILL UNCONFIRMED ON GLASS"*, and six features have stood *unverified on device* since 2026-08-24. **The replacement method, for anything phone-shaped: `adb exec-out screencap` to see, `adb shell input tap/swipe` at coordinates read off that image to act, and navigate from launch tapping only what is visible** — recording where the path dies. A desktop probe stays useful for regression, never for reachability **UPDATE 2026-09-08: the prescribed method was run for the first time, on the attached OnePlus, and it earned its place immediately** — one screenshot found a real defect that a session of probes had not. **But it also produced three findings that did not survive checking, so the method needs the calibration below before anyone trusts a result from it.** **The row’s blanket framing needs SPLITTING, and this is the correction.** *"A desktop probe is useful for regression, never for reachability"* is right about **reachability** and **wrong about geometry.** Measured: the same clipping defect reproduces on the desktop composition windowed at the device’s own 1080×2340 with `--force-touch`, nav-bar top at **y=2119** against the device’s **y=2116** — **3 px apart in the same UI state.** **So desktop layout measurements ARE trustworthy** and the phone probes in this tree are modelling the device; what they cannot model is a finger. **Distinguish the two before dismissing a desktop number**, or the cost of this row gets paid on every layout question that never needed a handset. **Three traps, all hit in the first hour, all now in `MISTAKES.md`:** **(1) Crop to full resolution BEFORE concluding.** Twice a downscaled view showed text the full-resolution crop of the same pixels did not contain — two phantom defects, neither filed. **(2) Run the control before believing a negative.** *"MORE does not respond to a tap"* looked like the exact unreachable-control class this row predicts; the control test killed it — MAP and PLAN had stopped responding too, because a full-screen panel had opened over the nav bar. **(3) Name the detent.** The one real defect was filed flat and is **conditional on the sheet’s collapsed state**; expanded, the chips draw in full. A fix verified in the wrong detent would look correct and change nothing. **Also: take two captures and diff them** (these differed only at the clock digit, which is what made the finding safe to file), and **record the installed build’s `lastUpdateTime`** — a screenshot is evidence only about the build it came from. **STILL OPEN, and why this row does not close: the six features unverified on device since 2026-08-24 are still unverified**, and one pass over four tabs is not the sweep this row asks for. **What changed is that the method is now proven and costed, not that the work is done** |
| Six features never driven on device since the 2026-08-24 USB disconnect — paint visibility, save/undo, the debug views, GeoJSON export, hand-drawn ways, civ-recompute | `ANDROID_BUILD_SCOPE.md` | medium | Recorded as *unverified on device*, not as verified. The 2026-08-25 pass drove a different list and did not pick these up |
| ~~**The phone MAP tab opens a half-detent sheet that is ~92% blank**~~ — **CLOSED 2026-09-21 (verified) — Ruling Z built** | `DESIGN_HANDOFF.md` | small | Commit `b1a8dd5`. `_phone_detent_height()`'s "half" branch now clamps MAP's height to its own real content (`_phone_map_sheet_content_height()`, reading the header/`tool_options_row`/stylebox margins live) instead of the flat transcribed `0.46`. `clampf` means it can only shrink toward content, never grow past the flat fraction. Gated on `_phone_tab == "map"` rather than a visibility flag — a real ordering bug caught by reading the call sequence (`_pick_phone_tab()` sets `_phone_tab` before the detent call but flips gen-panel visibility after, so the flag would name the previous tab on exactly the transition that matters). GENERATE/PLAN/MORE confirmed measured unchanged, not assumed. **Independently re-verified**: parse-checked `dcc_shell.gd` myself; re-ran `_detent_probe.gd --force-touch` myself, PASS 0 failures; viewed the before/after screenshots directly — the dead band under the header is gone. `project.godot` confirmed unchanged. |
| The default 2048×1311 new world costs ~878 MB peak on the phone | `STATUS.md` | medium | The "no progress indication" half is stale — a staged 10-stage readout ships off `cartalith-engine::progress`. The memory cost stands |
| The left-panel sheet retains its scroll offset across close/reopen and will not scroll back up — **NEITHER HALF REPRODUCES IN THE SHELL; STILL UNCONFIRMED ON GLASS** | `ANDROID_BUILD_SCOPE.md` | small | **Investigated 2026-09-06 and reproduced by the verifier at both densities, windowed — the two claims were separated and each has its own answer.** *Retains the offset*: **not reproducible.** `_set_sheet_open` calls `_reset_dock_scroll(_left_dock_scroll)` on every open, writing `scroll_vertical = 0` synchronously and again deferred; scrolled to 1234 → reopened at 0 at `_phone_scale` 1.0000, and 2658 → 0 at 2.6214. That landed in `0fc9d1c` on 2026-08-24 12:25:18, **2h31m after the device observation** in `2abf8df` at 09:54:18. *Will not scroll back up*: **also not reproducible, and a different cause** — 21 up-flicks of +251 and 21 down-flicks of −252 at 2.6214, 12 x positions across a row all at full delta, so there is no dead band; the likely fix is `phone_fit()`'s `MOUSE_FILTER_PASS` branches, whose bare-`Control` spacer arm landed `afe9016` on 2026-08-25, **a day after** the observation. **Do not read this as fixed on glass.** Both findings rest on synthesised input in the shell, and the first synthesised flick after a (re)open is a warm-up artefact (`delta=+0`, then full delta). **The next device pass is what closes this**, and the APK-rebuild row already gates it **2026-09-13, desktop re-check (verifier):** `_sheetscroll_probe --resolution 393x852 -- --force-touch --nowelcome` — left sheet 5 299 → 0 and right 3 859 → 0 after close and reopen; "retains offset" still does not reproduce in the shell at one density. **Device pass owed — the 6T is connected again** |
| Exercise **R1**'s Godot-side hunk inside a running Godot process on the handset | `MEMORY_OPTIMIZATION_SCOPE.md` | small | The case for R1 is four arguments, not a screenshot |
| The Android debug `.so` residue — 156 MB, 207 MB APK | `STATUS.md` | small | Reduced from 400 MB; still not the 18 MB a full strip gives. See §5 for why it stays |

### 2.8 Discipline debts

Small, cheap, and each one the kind of thing that silently invalidates a later
measurement.

| Item | Owns it | Size | Next step |
|---|---|---|---|
| **Re-freeze the reference and regenerate `FUNCTION_INDEX.md` in the same pass** | `CLAUDE.md`, `FUNCTIONAL_CONTRACT.md` | large | **This row said "to v2.11" and that is twelve versions out of date.** Measured 2026-09-17 in the working copy: the source repo holds **164** `Cartalith Gen1 v*.html` (newest **v2.22**) plus **44** DCC-line files (newest **v2.66**); `reference/` still holds only v2.10 and the index still enumerates v2.10's 1 094 functions. **Every capability tag in `FUNCTIONAL_CONTRACT.md` is measured against a reference the source moved past** — that file's own "no drift, no re-freeze question to raise" paragraph was corrected the same day. Re-freezing now also means **choosing a line**: the source forked at v2.22 and every engine change from v2.25 on exists only on the DCC line, so "newest" is two different files. See §2.9 |
| ~~Carry the Nortantis studied-not-copied disclosure into the credits screen~~ — **CLOSED 2026-09-21 (verified, already done)** | `PROVENANCE.md` | small | **Re-derived at the symbol: the row's own test is now false.** `credits.gd:44` carries the full disclosure verbatim (AGPL-3.0, studied-for-algorithm-only, the specific elevation thresholds and the naming-rejection rule it shaped), landed in commit `fb9c5b8`. `grep -i nortantis godot-project/credits.gd` returns real content, not nothing. |
| ~~Copy in the two upstream owner notes the research briefs cross-reference~~ (`Gravity influence.md`, `Weather Model.md`) — **CLOSED 2026-09-21 (verified, satisfied under the row's own fallback)** | `PROVENANCE.md` | small | `PROVENANCE.md:47-56` already keeps the disclosing paragraph — the exact alternative the row itself sanctions ("keep this paragraph so the dangling reference is at least a known one"). Confirmed `Cartalith_RC` (the only source of the real note text) is not present on this machine, so copying the files in is not currently possible; the fallback is what's live. |

### 2.9 Source-engine changes specified but not routed

**Added 2026-09-17. This is not new work — it is work that was never counted.**
`RC_ENGINE_CHANGES.md` is a full porting spec for the source engine's changes,
now **v2.11 to v2.73** (extended 2026-09-21, pulling in v2.69/v2.71/v2.72/v2.73
and §7.13 from a sibling branch that had kept documenting while this one did
porting — see the five new rows below), and until 2026-09-17 **no document in
this repository referenced it except `CLAUDE.md`** (`grep -rl RC_ENGINE_CHANGES
*.md` returned one file). So a 1 000-line (now 2 700-line) specification of
everything the source engine did after the freeze sat outside this ledger,
outside `STATUS.md`, and outside the count at the top of this file.

| Item | Owns it | Size | Next step |
|---|---|---|---|
| ~~**Establish which of the specified changes are already ported**~~ — **SURVEYED 2026-09-21, not a build (no count change)** | `RC_ENGINE_CHANGES.md` → `STATUS.md` | large | **Full survey complete, 7 parallel read-only passes over the whole spec, cross-checked against real Rust source (not code comments) — no files modified.** Of the ~31 core-simulation items (§1–§6s) plus the surveyable §8.1/§8.2 rows already-known-closed items excluded (v2.69/v2.70/v2.71-half-1/v2.72/v2.73-half-1, all closed this session; v2.71-half-2 declined; v2.73-half-2 blocked on source): **12 PORTED, 8 PARTIALLY PORTED, 26 NOT PORTED, 7 UNCLEAR/not deep-checked** (lower-priority §8.2 shell/UX rows, time-boxed out). **Spot-checked independently by the coordinating session** (2 of the highest-stakes claims, at the symbol): (1) confirmed `enforce_channel_descent` (`cartalith-hydrology/src/lib.rs:938`) is genuinely called live from `cartalith-engine/src/lib.rs:1807`, not dead code — matches the survey's claim that this port built the exact digging pass §6m says the reference reverted (v2.61). (2) confirmed `compute_flow` (`cartalith-hydrology/src/lib.rs:138`) sorts by raw height (`flow_sort_desc`) with no priority-flood depression fill — matches the survey's §6g claim, which the spec itself calls its most consequential finding. **Two sub-claims NOT confirmed by this spot-check**: (1) the survey's assertion that `PLATE_BASE_BLUR_K` is "hardcoded at old 0.35 in two call sites" — grepped `build_plates`/`assign_plates` in `cartalith-terrain/src/lib.rs` and every literal `0.35` in that file; none matches a plate-blur context. (2) **CORRECTION 2026-09-22 — this correction was itself wrong, found empirically, not by re-reading.** The §6r.5 "unbounded retry hang" claim WAS real: an agent building the market-town Rules preset hit an actual 35-second generation (against ~90ms) at `frontage_width_variance = 0.15`, and found the existing "Planned Grid" chip's own `0.10` value fails to finish a pop-4 000 town in 90 seconds — a real, reproducible hang in shipped code, not a hypothetical. The 2026-09-21 correction above looked at the loop's outer shape (`while acc < e_len - 3.0`) and concluded each iteration makes monotonic progress; it missed that a rejected candidate width (via the inner `for pw in &parts { if acc + pw > e_len - 2.0 { break } }`) can leave `acc` unchanged for an entire outer iteration, so the loop redraws a fresh random width and tries again — exactly the reject-and-retry-until-fits shape the spec describes, just nested one level deeper than the earlier grep looked. **No fix dispatched for this either** — it needs a bound (a golden re-baseline, per the spec's own fix shape) and is filed as its own row below rather than fixed inline. **Top flags for scheduling, from the survey (not independently re-checked beyond the two confirmed and two corrected above)**: §6g (flow over filled depressions) unported and called the spec's most consequential finding; §1.2's crater model looks ported (same shape, gated flag) but uses independently-derived constants, not the spec's v2.22 numbers — owner ruling needed on whether that's acceptable. (§6r.5's retry-hang claim is corrected above — not a live risk in this port's actual code. **§6l is CLOSED 2026-09-21 (verified)** — see the dedicated row below.) **This is a survey, not a build — no row below is scheduled or closed by it.** Whoever schedules from §2.9 next should read the full table in this row's own evidence trail (agent `a3b4c5e3e6eabc988`'s report) before picking an item. |
| **Decide which line the port follows** | `DECISIONS.md` | medium | The source forked at v2.22: v2.23 duplicated it to carry this port's own shell theme, and **every engine change from v2.25 on exists only on the DCC line**. "Track upstream" is therefore ambiguous and no decision records which branch is meant. Blocks the re-freeze in §2.8, which cannot pick a file without it |
| ~~**§6l · River rasterization breaks into disconnected segments on a diagonal receiver step**~~ — **CLOSED 2026-09-21 (verified)** | `RC_ENGINE_CHANGES.md` §6l; `cartalith-hydrology/src/lib.rs` | medium | **Confirmed real by direct code read before scheduling, not assumed from the survey.** `stamp_river_intensity` stamps each channel cell's disc independently; a cell with `half_w < 1.0` (common) inks only its own centre (`d==1` orthogonal / `d~=1.414` diagonal both exceed `half_w`), and a D8 receiver chain steps diagonally ~42% of the time — two consecutive narrow cells one diagonal step apart can paint discs that never touch. **Fixed surgically**: a second pass bridges only the diagonal-receiver case (the one case not already 4-connected on its own, since `d8_receiver` only ever returns an immediate neighbour or `-1`), raising the two shared-adjacency cells to the dimmer of the two endpoints — no river's rendered width changes anywhere else. Wrap-aware (`wrapped_axis_delta`, mirrors `d8_receiver`'s own "`x` only" wrap rule), correctly distinguished from the pre-existing v2.72 wrap-seam cut. Two new structural tests: one proves the gap mathematically (`half_w < 1.0` is a guaranteed-unreachable radius, not an assumption) then confirms 4-connectivity via flood-fill; one confirms the wrap case bridges only at the seam, never reaching the map's middle. **No golden-parity value moved** — `golden_parity_river.rs`, `tile_hydrology.rs`, `bake_raster.rs` all re-run before/after, none needed a hash change. Independently verified: full diff read, `cargo test --workspace --no-fail-fast` reproduced exactly (163/3490/0/34), `git status` confirms only `cartalith-hydrology/src/lib.rs` changed. Commit `f8d8bcd`. |
| ~~**Multi-ridge orogenic belts ("the Himalaya problem") — resolve a contradiction in the spec before scheduling**~~ — **RESOLVED 2026-09-21, §6b.2 was right, §9 was wrong (twice)** | `RC_ENGINE_CHANGES.md` §6b.2 vs §9 | large | **Owner supplied `Cartalith_v2.71_DCC_test.html` (`VERSION='2.71'`, its own line 2682) specifically to settle this — checked directly, not guessed from prose.** `buildOrogenyField` (line 3865) is real, shipped code: multi-sheet stacking (`nSheet`), `beltSpan=1.55*halfBelt` (line 3884, §6b.2's own cited figure), per-sheet tapering, plateau fill, foreland basin. It runs whenever `state.tect.tectonicGraph` is on — not a rare manual override, since any active World-Structure archetype sets it automatically (line 3252). §9's second orogeny claim was also false: `orogenyWidthScaleK(mapWidthKm)` (line 3472) exists, reads real km, and feeds every `buildOrogenyField` call — its own doc comment names it the sixth sibling of `terrainDetailK`/`riverWidthScaleK`, the same v2.36/v2.49 family already documented elsewhere in this spec. **`RC_ENGINE_CHANGES.md` §9 corrected in place**, both bullets struck through with the evidence rather than silently rewritten. **What this unblocks, not yet scheduled**: this port already has `cartalith-terrain::build_orogeny_field`/`smooth_orogeny` (`OrogenyParams`, no multi-sheet fields), golden-tested against v2.11's simpler shape (`golden_parity_orogeny.rs`) — that part stays correct and untouched. Both v2.35's junction-predicate fix (§6b.1) and v2.36's multi-sheet belt are gated behind `tectonicGraph`, off by default in the reference itself, so even a faithful port would be invisible at default settings. EF-6's boundary tracer (`cartalith-spatial::contour`, already built) may solve §6b.1's crossing-number problem as a side effect — worth checking before writing a second tracer. **Still needs its own scheduling decision** (large, multi-sheet belt geometry) — this row only removes the "maybe never shipped" blocker, it does not authorize building it. |
| **v2.69 · Tile refinement crosses sea level — CONFIRMED inherited and measured, needs an owner ruling to fix (golden re-baseline)** | `RC_ENGINE_CHANGES.md` §8.1; `cartalith-terrain/src/amplify.rs` | medium | **Checked at the symbol and measured, 2026-09-21 — not a "check whether," a confirmed live defect in this port's own EF-0 code.** `amplify_region`'s `underwater` term only reduces detail going DOWN into water (`if base < opts.sea { taper shrinks }`, else full `taper = relief` unconditionally) — the same one-sided shape the HTML's v2.69 fixed, and a synthetic land/sea gradient measured it symmetric at 0.32%/0.32% at this layer alone (real number, not dramatic — disclosed honestly rather than dropped). **The real defect is one layer up, in `add_zoom_detail`** (the deep-zoom octave-stacking pass): `if base < sea { continue; }` — a water cell gets ZERO extra octaves, a land cell gets up to 6 *unclamped* extra octaves unconditionally, **written back with no `[0,1]` clamp** ("Ported as written" per its own doc comment, matching the reference's own pre-v2.69 shape exactly). Measured on a synthetic coastal gradient: **0.00% land→sea at z=2, rising to 0.17% at z≥4, and 0.00% sea→land at every level tested (z=2/4/6/8)** — strictly one-directional, more extreme than the HTML's own 7.6:1 because this port's water cells are fully exempt rather than merely tapered. **Why this is not a same-session fix**: `cartalith-terrain/tests/golden_parity_amplify.rs` currently pins this exact asymmetric behaviour byte-exact against the v2.11 reference (correct parity against the FROZEN baseline, wrong relative to what the product should do and what the HTML itself later fixed) — changing it moves generated pixel output at every tile near every coastline, which is a golden re-baseline needing owner authorisation the same way Ruling N did, not something to land silently. **The fix, per RC_ENGINE_CHANGES.md's own v2.69 row**: clamp the excursion toward sea level at half the remaining headroom in both `amplify_region` and `add_zoom_detail` (both must be fixed — the HTML's own report: fixing one alone left a third of the drift). **AUTHORIZED 2026-09-21 — Ruling O (`LARGE_ITEM_RULINGS.md`).** ~~CLOSED 2026-09-21 (verified) — landed in `c37488a`.~~ `clamp_toward_sea(base, delta, sea)` added to `amplify.rs`, called at all 4 sites (`amplify_region`, `add_zoom_detail`, and both halves of `sample_elevation` — the latter a bit-identical hand-transcribed copy the fixing agent found and guarded, not in the original brief). Golden re-baseline: 7/12 amplify cases, all 6 zoom-detail cases, 7 bake cases, 3/4 region-export tiles moved hashes, each disclosed old → new in the test file's own header; deep-zoom (z=5/z=7) and fixture/seam-delta checks byte-identical as expected. Independently verified: full-workspace test count reproduced at 163/3435/0/33; `clamp_toward_sea` confirmed absent from the pre-fix `HEAD` via `git show`; the `* 0.5` constant mutated to `* 0.25`, confirmed to fail the literal-asserting unit test with the predicted panic, then reverted with a byte-identical file hash. One nuance for a future pass, not a defect: the agent's own measured coastal-asymmetry percentages differ numerically from this row's original pre-fix figures (same shape, different fixture) — the row above is left as originally measured; a reader diffing the two should not read that as a second bug. |
| ~~**v2.70 · A flat limited-palette "Village map" style**~~ — **CLOSED 2026-09-21 (verified) — one open flag, disclosed not verified** | `RC_ENGINE_CHANGES.md` §8.2 | small | `Npr::village`, the eleventh style in `apply_npr`'s existing ten-style gated chain, quantised as a REPLACEMENT at the end of both `apply_npr` (land) and `sea_color_core` (water), sharing `quantize_flat_palette`. GUI: a checkbox, a "Village" `STYLE_PRESETS` row, hillshade zeroed via a new optional 4th preset element. Purely additive — full `cartalith-godot` suite passes unchanged with the flag at its default-off value, no re-baseline. **Open**: the quantisation band count (3 divisions/4 levels) is this port's own choice, not the verified reference constant — `Cartalith_RC` was not present on this machine to check `landColorCore`/`seaColorCore` against. Flagged at `quantize_flat_palette`'s own doc for whoever can check it later. Commit `7adb600`. |
| ~~**v2.71, half 1 · Nothing the settlement layer draws may sit on water**~~ — **CLOSED 2026-09-21 (verified)** | `RC_ENGINE_CHANGES.md` §8.2 | medium | Confirmed this port had the HTML's exact defect: `Site::is_water` already reads a real 22 m mask and generation already keeps blocks/parcels/buildings off it, but `Site::water_poly` — the only water geometry crossing the Rust→Godot bridge — is deliberately empty on the real-map coastal path, so the mask never reached the renderer. `WaterCtx::water_runs()` (run-length-encoded rectangles, same local-box frame as `water_poly`/`river`) → `UrbanLayout::water_mask_runs` → `urban_bridge.rs`'s `"water_mask_runs"` key → a new `_draw_water_mask()` painted last, corners projected individually (correct under rotation), bridges/fords released by a distance test. Purely additive — empty on every synthetic site, so no golden (Rust or GDScript, confirmed none render/hash settlement-over-water pixels) needed re-baselining. New mutation-safe unit test (literal expected rectangles) + 708+591+268 passing lib tests across the three touched crates. Commit `0bf3152`. |
| ~~**v2.72 · Raster river geometry, antimeridian wrap + a detection-ease used as a display threshold**~~ — **CLOSED 2026-09-21 (verified) — fix 1 landed, fix 2 found genuinely inapplicable** | `RC_ENGINE_CHANGES.md` §7.13, §8.1 | large | **Owner supplied real `Cartalith_v2.71`/`v2.73` DCC-test source specifically to confirm both defects; both matched the spec exactly, and the owner authorized landing.** Commit `fcaafea`. **Fix 1 (detection ease spent as a display threshold) landed**: `RIVER_RENDER_AREA_K=2.0` (the reference's own literal), `river_render_area_bar()`, a new per-channel-cell `channel_cell_drainage` (channel-cell count via the receiver tree's own topological order, deliberately NOT `flow`/`flow_discharge` — the reference is explicit these are two different trees). Measured on a real 384×384 `world:true` pipeline: 40 000 km inked fraction 26.41% → 1.38% (−95%), 1200 km 5.06% → 3.46% (−32%, headwater tips only) — confirmed the bar does not gut the normal-scale network. No golden moved (grepped: nothing tests `stamp_river_intensity`'s output at all). `render.rs` untouched — the gate lives at generation time, `WorldState.channels.intensity` already carries it. **Fix 2 (antimeridian wrap) does NOT apply to this port** — a real structural finding, proven with an oracle test, not just reasoned: this port's disc-stamp technique paints AT each channel cell's own coordinate, never a connecting line between cells, so it cannot have the reference's stroke-renderer wrap defect. This port's own polyline tracer (`trace_river_polylines`, used for GeoJSON export/`get_rivers`) already has the equivalent wrap-cut via `split_river_polylines`. **Honest answer on the owner's "smooth, not pixelated" bar, stated when authorizing this build**: this fix makes the network sparser and more legible at coarse extents, but does not smooth any single river's stair-stepped raster-disc edge — that needs the separate vector overlay follow-up, queued next (see the row below). **Independently re-verified**: `cargo test -p cartalith-hydrology` clean; full workspace re-run after both this and v2.73 landed together, 163/3450/0/33; `render.rs`'s diff confirmed empty. |
| ~~**v2.73, half 1 · A village green as a distinct plaza kind**~~ — **CLOSED 2026-09-21 (verified) — rebuilt fresh from real v2.73 source, golden values confirmed exactly** | `RC_ENGINE_CHANGES.md` §8.2 | medium | **Owner supplied real `Cartalith_v2.73_DCC_test.html`; the earlier reverted implementation's four golden values were checked against it and reproduced exactly, not guessed.** Commit `68e5656`. `PlazaKind::{Market,Green}` on `Plaza`, `PLAZA_MARKET_POP=1500.0` reused from `build_civic`'s own chartered-town gate (also fixed a duplicate `1500.0` literal in `build_civic`'s own `size_mult` formula), market-cross push gated on `kind == Market`. Geometry verified byte-for-byte against the real reference — untouched, only `kind`/`prov` are new. **The four golden values confirmed exactly**: `landlockedHamlet` 115→114, `popFloorClamp` 113→112, `hamletBoundary` 120→119, `venusTinyCanal` 49→48, each the one withheld market cross. The 29-case orchestration golden's `hash` field (graph/blocks/parcels/buildings) confirmed byte-identical for all 29 cases — only the four disclosed rows' `details`/`detail_kinds` moved, disclosed old→new in `golden.rs`'s own new header. **Independently re-verified**: diffed `golden.rs` myself, zero `hash:` lines changed anywhere; full workspace test re-run after both this and v2.72 landed together, 163/3450/0/33. |
| **v2.73, half 2 · A footpath class from an always-on source** — **needs real v2.73+ source, not a same-session port** | `RC_ENGINE_CHANGES.md` §8.2 | medium | **2026-09-21: checked at the symbol, not assumed absent.** `cleanup.rs::privatize_alleys` is real (alley-privatisation, kills cart-routing edges) but records nothing it kills, and no footpath/snicket concept exists anywhere in `cartalith-urban`. The RC row gives the mechanism (paths from placed features to nearest street, gated on not crossing a building footprint) but only *outcome* measurements ("2 paths on a village, 4 at pop 12000, 7-17m, bounded at both ends") — no search algorithm, no bound semantics, no line-level spec to port bit-exact. Inventing one would be original code dressed as a port in a project whose whole test methodology is byte-for-byte JS parity (`MISTAKES.md`: port the reference's approximations, don't invent a mathematically-ideal one). **Also found**: `DEFAULT_RULES.street.dead_end_bias == 0.0` on every live culture profile (confirmed by an existing test), so even the recorded-closures half alone would be inert on every current golden — building only that half reproduces the exact "computes correctly, shows nothing" trap the RC row itself warns against. Same shape as v2.71's woodland half — needs `Cartalith_RC` access or an owner-supplied capture, not invention. No code written. |

**Why this was invisible.** The spec was written *for* the port and lives at this
repository's root, so it reads as already-integrated. It was never wired into the
router. That is the failure this file exists to catch (§6), reached from the
other side: not a document making a stale claim, but a document making no claim
anywhere anyone would look.

---

---

## 3. Blocked, with the blocker named

A row is here only if something concrete stops it. Where the blocker is an
owner answer, the question itself is in §4.

### 3.1 Blocked on an owner decision

| Item | Owns it | Size | Blocker |
|---|---|---|---|
| ~~**Refine detail placement**~~ — **CLOSED 2026-09-21 (verified) — Ruling U built** | `LARGE_ITEM_RULINGS.md` Ruling U | small | Commit `6d1c639`. Moved from the WORLD tool-options bar back to `Preferences ▸ Tiles & LOD ▸ Atlas cache` (its pre-2026-09-05 home), following the newer design canvas per the owner's ruling. `refine_current_view()` itself untouched — only its caller moved; `ID_LOD_REFINE_VIEW` (81, unallocated since 2026-09-05) reused; the `command_index.gd` EXTRAS duplicate removed now that it's a real `PopupMenu` row the live-tree walk finds for free. **Independently re-verified**: parse-checked `app.gd`/`menus.gd` myself; re-ran `_idxfind_probe.gd` (0 failures) and `_structmove_probe.tscn` (all six Refine-detail checks pass, including the exact refusal hint text reproduced verbatim); viewed the agent's screenshot directly — the row is exactly where claimed. `project.godot` and the live settings file confirmed unchanged. |
| ~~**Landmark M7 — viewshed / line-of-sight**~~ — **CLOSED 2026-09-21 (verified) — 5 of 7 kinds unblocked, 2 genuinely still blocked** | `LANDMARK_GENERATION_SCOPE.md` | large | A real R3 line-of-sight primitive (`cartalith_terrain::analysis::visibility`/`ViewObserver`/`ViewParams` — a ray to every perimeter cell of the radius square, each carrying the running max vertical angle plus refracted-curvature drop). **Sizing is a disclosed, conservative default, not an owner number waited on** (`VIEW_RADIUS_KM = 40`, clamped `[2,104]` cells; a sparse, strided, weighted observer set — settlements + route samples, capped 256 — turning an O(n²) whole-map problem into O(observers·r²) independent of grid size). Measured 2.75 ms at 256×192 to 138 ms at 2048×1536 for the field alone (~10% of the whole landmark pass at every size). `VIEW_MAX_CELLS` raised 48→104 on the strength of the measurement — at 48 the stated "40 km" silently meant 19 km at this project's own 2048 shipped default. **Unblocked, each verified placing on a real `generate_terrain` world**: `fort`, `watchtower`, `fortified_pass`, `fortified_crossing`, `volcanic_feature`. The four military roles are one detector partitioning one candidate set (not four independent ones, which would double-place a hilltop), the same discipline Ford/Bridge/`mountain_pass` already use. **`border_marker` and `sacred_mountain` stay genuinely blocked** — no faction/territory field exists for the former, no cultural-meaning input for the latter (`LANDMARK_GENERATION_SCOPE.md` §26 forbids hardcoding one) — their `not_built` reasons corrected to say so precisely rather than left stale. **`peak` deliberately NOT wired** despite `needs_viewshed: true`: it already generates, and adding a visibility term would move existing placements, a golden-shaped change needing an owner ruling, not a silent one. 12 new tests (6 terrain, 6 civ), 6 constants mutation-tested and killed. Independently re-verified by the coordinating session: `cargo build`/`cargo test --workspace` re-run against the exact staged commit, reproducing 163 result lines / 3431 passed / 0 failed exactly; every cited symbol confirmed by direct read. **Fixed a real ~90-second broken-HEAD window** this batch's own file-sharing caused — see `MISTAKES.md`'s new preflight row. Commit `222189f`. |
| **IN-13 — trade flows**: who trades with whom (bipartite match, network flow), prices, tariffs, caravans as entities | `STATUS.md` | large | Needs a decision about what a currency is in this world. `TradeBalance` names *what*, never *who*. **RULED 2026-09-21 — Ruling R (`LARGE_ITEM_RULINGS.md`): per-faction currencies, with an exchange rate between any two.** The currency-model question is settled; the subsystem itself (bipartite match, network flow, prices, tariffs, caravans as entities) is not yet scheduled — this row stays open as the build, now unblocked on design. |
| Resolution-range policy — 4096 needs 2.41 GiB and 8192 needs 9.65 GiB, so 2048×1311 is the last Android-viable preset | `MEMORY_OPTIMIZATION_SCOPE.md` §8 | small | A product decision. The doc twice refuses to change `RESOLUTION_PRESETS` unilaterally, and now has the numbers to support whichever way it goes |
| Save compression — the byte-plane shuffle (27-36% smaller, writes faster) | `STATUS.md` | medium | Needs a `format_version` bump and a fail-loud marker; **it ends `SAVEFILE_COMPAT.md` §8's bare-dump promise** |
| ~~**The in-session tile cache is not invalidated by a sculpt**~~ — **CLOSED 2026-09-21 (verified)** | `GUI_GAP_REGISTER.md` | small | All 11 call sites that write `map_view.texture` directly now call `invalidate_lod_tiles()`: `tool_bar.gd`'s Commit chip and paint commit, `right_dock.gd`'s Commit-to-map, revert and paint-commit-from-dock, `world_workspace.gd`'s erode and paint commit, `app.gd`'s undo/redo, `menus.gd`'s redo, `dcc_shell.gd`'s phone revert history. No shared choke point existed, so each site got its own call rather than forcing a riskier cross-file refactor. `_sculptlodcache_probe.gd` extended (sections 5-15) to drive each path through its exact production function and assert the on-screen LOD-composited crop moves with the base texture — **independently re-run, not just trusted**: full `PASS`, all 15 sections green, matching the agent's own result exactly. Two stale `GUI_GAP_REGISTER.md` citations fixed to `OUTSTANDING_WORK.md`, where the row lives; `invalidate_lod_tiles()`'s "a caller that ... `_run_erode()` beside it" doc comment, aspirational before this fix, is now literally true. Hitch cost not re-measured (out of scope — don't make it worse, not eliminate it); the method's pre-existing no-op guard already covers "nothing to invalidate." Commit `92d1edc`. |
| ~~**LOD tiles go stale after undo, redo, revert, erode, paint and the other two sculpt-commit buttons**~~ — **CLOSED 2026-09-21 (verified), same fix as the row above** | `OUTSTANDING_WORK.md` | small | Duplicate of the row above, filed separately by the verifier that found it; closed together. Commit `92d1edc`. |
| Save compression — quantising saved rasters to `u16` | `STATUS.md` | medium | Lossy. `PARITY_TESTING.md` and `DECISIONS.md` §7a bar it without a ruling |
| ~~**CA-19** — a writable biome colour table~~ — **CLOSED 2026-09-21 (verified)** | `STATUS.md`, `PARITY_AUDIT.md` | medium | **Ruling P landed, commit `cc0f561`.** `TerrainAppearance` gained `biome_cols: [(u8,u8,u8);15]`, defaulted to `CART_BIOME_COLS` byte-for-byte, plus a `WorldGen`-held override table layered on in `appearance()`. New `#[func]`s: `set_biome_color`/`reset_biome_color`/`reset_biome_colors`/`get_biome_color`. `swatch_color` (the pinned golden entry point) now delegates to a new `swatch_color_with(..., biome_cols)` sibling, so zero existing call sites or tests moved — **no golden re-baseline was actually needed**, contrary to what this row expected: the fix found a design that kept the default path bit-identical rather than requiring the constant to move. Deliberately left inconsistent and flagged, not silently assumed done: the CARTO legend (`sample_bridge.rs`) and the live paint-tool preview raster (`pack_window`/`preview_full`/`preview_patch`) still read `CART_BIOME_COLS` directly, not the override — GUI-adjacent plumbing out of this ruling's scope. **Independently re-verified**: `git diff` shows zero `+`/`-` lines inside the `CART_BIOME_COLS` literal itself; `cargo test -p cartalith-godot --no-fail-fast` re-run in isolation, 874 passed / 0 failed across 26 result lines including both new override-path tests; `project.godot` and the live settings file confirmed unchanged. Full-workspace re-run deferred: `cartalith-civ` was mid-edit under the concurrent hydrology reclassification (Ruling Q) at verification time. |
| ~~Delete the seven uncalled `cartalith-gpu` public functions (~70 lines)~~ — **CLOSED 2026-09-21 (verified)** | `GPU_LAYER_INTEGRATION_SCOPE.md` | small | Re-derived at the symbols: `init_gpu_f64` was already deleted 2026-09-06 (owner question 8, untouched); the other six (`heterogeneity_grid_gpu`, `gauss_blur_grid_gpu`, `assign_plates_grid_gpu`, `flow_accumulation_gpu_with`, `gpu_resistance_grid_cpu`, `warp_grid_gpu`) confirmed zero real callers workspace-wide and deleted, net −75 lines. The milestone-6 section header and the four surviving `_with` siblings' doc comments (previously rustdoc-linking to the now-deleted symbols) rewritten to carry the substance forward rather than dangle. `cargo check --workspace` clean, `cargo test -p cartalith-gpu --lib` 68/0/1-ignored, re-verified independently. Commit `46aff27`. |
| ~~The flaky GPU determinism test `generate_terrain_gpu_path_is_deterministic_and_valid`~~ — **CLOSED 2026-09-22 (verified, found already done)** | `STATUS.md` F1 | small | **Re-opened at the symbol per `MISTAKES.md`'s own rule, not assumed from this row's stale text.** The fix this row asks for already shipped a month ago, commit `803b725` ("The GPU determinism check now asks what §7a says to ask", 2026-08-25) — confirmed a real ancestor of HEAD (`git merge-base --is-ancestor`) and the file byte-identical to HEAD (`git diff` empty). The test now compares the worst per-element `f32` deviation against `GPU_DETERMINISM_TOL = 1e-6` (~8 ulps on a [0,1]-normalized field, derived from real measurement: ~1 ulp deviation between two GPU dispatches of the same seed under parallel-load scheduling noise) instead of a whole-field `assert_eq!`, matching `DECISIONS.md` §7a's principled-equivalence bar for GPU paths. **Verified independently**: 6 consecutive isolated runs all `ok`; full workspace `cargo test --workspace --no-fail-fast` reproduced 3485/0/34 exactly; a throwaway negative-control test (a synthetic field differing by 0.01, 10 000x the tolerance) confirmed the bound still catches a real divergence, then was fully removed, confirmed via empty `git diff`. Zero files changed this session — the row itself was simply stale. |
| Military manpower **finding 2** — standing armies land at Imperial Rome's ratio, not the era table's standing column | `MILITARY_MANPOWER_SCOPE.md` | medium | Correcting it means recalibrating outputs currently validated against the owner's worked example. Reported, not tuned |
| Shrink `STATUS.md` | `STATUS.md` own header | medium | An editorial decision for the owner, declined twice by audit passes as correctly out of their remit. Still not made — but **the size that motivated it is gone**: this cell said "8 122 lines with four lines over 15 000 characters" until 2026-09-01, contradicting this document's own header three paragraphs in. `wc -l` gives **1 445** today (1 157 at the 2026-08-31 rewrite, so it is growing again). The decision is open; the emergency is not |
| §20 — the high-precision display pipeline — **moved here 2026-09-23, real scope found before scheduling** | `TERRAIN_APPEARANCE_SCOPE.md`, `TERRAIN_APPEARANCE_RESEARCH.md` §20 | medium-to-large | **Was carried in §2.5 as a plain "next step" row; a scoping pass (not a build, deliberately) found it isn't one.** `TERRAIN_APPEARANCE_RESEARCH.md` §20 is a generic architectural wish (physical fields → linear/HDR-capable intermediate → tone mapping → display colour space, explicit "must gracefully fall back" on non-HDR hardware) with no numeric spec, no measured defect and no acceptance test named. **The owning scope doc has rejected building it twice already, on record**: milestone 5 calls it "real architectural work whose payoff is HDR/wide-gamut output that nothing in this port consumes yet"; milestone 6 rejects it again because its own measurements show clipping *falling* (0.78%→0.68% on Classic) — the defect §20 would fix is not currently present. **Real blast radius, if built anyway**: three whole-raster `u8` passes (`apply_local_contrast`/`apply_color_grade`/`apply_color_space` in `render.rs`) are called from all four pipelines — screen live-view (`build_color_texture`), layer PNG export (`export_raster.rs`, skips `apply_color_space` by a 2026-09-06 decision), the E1/E2 streaming raster export (`bake_export_band`), and LOD tile synthesis (`render_biome_tile_rgba`, which skips local-contrast but calls the other two). `cell_color` itself already computes in f64 — quantization to `u8` happens once, at each caller's own buffer write, not in the material math. Godot's screen path is hardcoded `Format::RGB8`; PNG/BigTIFF export are hardcoded 8-bit encoders (a 16-bit path is a new encoder, not a type swap); `tests/color_space.rs`'s `FINISHED_RENDER_FNV1A` hashes the whole finished u8 raster and would need re-pinning, not just re-verifying; the JS reference itself renders via 8-bit canvas `getImageData`, so there is no reference to golden-verify a higher-precision result against. **A narrower staged version exists** (widen only the intermediate buffer between the three correction passes to f32, quantize once at the very end) — architecturally cleaner, still touches every call site and still forces the same test re-pinning, but does not deliver the owner's actual HDR/wide-gamut ask. **Blocker, an owner ruling needed before this is scheduled at all**: is the real goal "fix a measured precision defect" (none currently measured — the scope doc found the opposite) or "build toward HDR/wide-gamut output" (real architecture, but `gl_compatibility` — this port's actual Android/desktop-fallback renderer — has documented limitations that leave the payoff's reachability unconfirmed)? Nothing has changed since the scope doc's own two rejections. |

### 3.2 Blocked on other work in this list

| Item | Owns it | Size | Blocker |
|---|---|---|---|
| ~~**LOD-D4 · Ice and snow from fields that already exist**~~ (Ruling K, step 2) — **CLOSED 2026-09-21 (verified) — 1 of 4 bars met, 1 partial, 1 explicitly unsatisfiable without a re-baseline, 1 barely measurable** | `LOD_DETAIL_SCOPE.md` | medium | `TerrainAppearance::ice_strength` gates a two-stage pipeline (`apply_ice_cover` rebalances material weights off `geo_exposure`'s own slope term so ice/rock-exposure logic can't disagree; `snow_material_col` tints via the existing `snow_glac` ramp), inert under `js_reference()` by control flow, not `* 0.0`. **A real parity leak found and fixed**: the lapse-correction stage was gated only by control flow at the wrong site, so every tile on the parity path carried a sub-cell temperature the reference has no concept of — fixed at both call sites. Mutation 9/10 killed (one flagged below). **Measured on 3 seeds at a 25 km tile view**: bar 2 (potential≥0.5 drawn as ice/snow, ≥80%) **passes at 100%** on all three. Bar 1a (snowline transition ≥15% of relief) passes 2/3. **Bar 1b (snow-vs-northness \|r\|≥0.2) FAILS on all three** (−0.035/+0.008/−0.084) — traced to the symbol: `material_weights`' snow term is `smoothstep(3,-5,t)`, a function of temperature *alone*; this row's own prior text ("snow follows temperature, precipitation, slope, aspect and curvature") named the requirement correctly and the shipped stage doesn't meet it. **Giving snow an aspect term is a golden re-baseline of the MAIN MAP's own `materialWeights`, needing an owner ruling — not done here.** Bar 3 (steep ground above snowline rock-dominant, ≥60%) is barely measurable at this zoom: p99 slope (0.004–0.073) sits mostly below the 0.08 knee, a tile-amplification property unrelated to this milestone. Budgets: glacier field build 27.3–27.7ms (<50ms, met); field size 10.24 MiB at 2048×1311 (the scope's own "≤10 MiB" reads as met or not depending on whether that means ×1024² or ×1000² — stated both ways); per-tile cost +5.5–6.6% (≤+10%, met). GUI: a new "Ice & snow" appearance group, built ahead of any canvas drawing one (flagged for an owner look, same as this session's other GUI-ahead-of-design calls). Commit `02f6d51`. |
| **Landmark M9** — cultural interpretation and temporal state | `LANDMARK_GENERATION_SCOPE.md` | large | `STORY_PLANNING_SCOPE.md` **SP-4**, which is not started and whose attachment model is undecided, plus open questions 1-2. **Two documents' largest remaining milestones sit behind one unasked question** |
| Story planning **SP-2** — journey progression over the cursor | `STORY_PLANNING_SCOPE.md` | large | §6's regenerate-semantics question explicitly gates it: whether a journey's route polyline is invalidated, re-snapped, or kept with a staleness mark "needs a ruling before SP-2 ships". The grain question (real date vs fraction of a year) is also unresolved |
| Story planning **SP-5** — the planning aid, joined up | `STORY_PLANNING_SCOPE.md` | medium | Deliberately last: worth nothing until at least two of SP-1…SP-4 exist. Only SP-1 is partly real |
**Closed from this table 2026-09-03** (batch 18, verified): **milestone 16** —
shipped in `cff1edc`, golden byte-reproducible from the frozen reference, 12 of 13
stage modules mutation-covered. **Milestone 17's five `_um*`** — all five exist,
all five are golden-covered, and all five survive mutation of a constant each
(`um_wall_spec` `age >= 260.0 → 261.0` KILLED; `um_site_profile` `gw/70 → gw/71`
KILLED). **Both rows' stated blockers were false.** Milestone 17's — "settlements
carry no `specialisation` and no `traits`" — was falsified **six minutes after it
was written**: `be2d5f7` 19:31:09 added the `economy: None` hardcode, `e63d5d9`
19:37:15 added the `PlaceExtras` that supplies it, and it stood for eleven days.
One genuine gap remains and is filed under §2.1: `urban_bridge.rs` still calls
`settlement_layout()` (which supplies `PlaceOverrides::default()`) rather than
`settlement_layout_with()`, so a per-settlement wall/age override is stored but
never reaches the layout.


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
| The 3D research's three commissioned questions (`gl_compatibility` rationale; wgpu/Godot GPU coexistence; what a raised device floor buys) | `3D_TERRAIN_RENDER_RESEARCH.md` | medium | Parked with the 3D viewport. Question 2 is named the highest-value unanswered question and gates `RenderingDevice`, compute shaders and GPU-driven culling. Resuming is cheap — the research is complete at 1 530 lines |
| Vault **milestone 4** — device pass verifying the Android SAF provider (folder picker, persisted grant, revocation) | `MARKDOWN_VAULT_SCOPE.md` | large | Needs a real Android device |

---

## 4. Open decisions the owner still owes

**ZERO. All nineteen were answered on 2026-09-06** and are recorded in
`LARGE_ITEM_RULINGS.md` as rulings 8-25, each with the reasoning it was given and
the cost it carries. Five went against the recommendation offered and say so;
one (the viewshed budget) was amended rather than chosen.

**This section stays, empty, on purpose.** It existed because decisions were
being made implicitly by whoever happened to touch a file next. An empty section
is the record that the queue was cleared, not that the practice stopped — a new
question belongs here rather than in a code comment.

Three of the nineteen created work rather than removing it, and those rows are in
§2: the **crate consolidation** (ruling 24), the **16K/32K export un-shelve** and
its now-live codec question (ruling 15), and folding **`performance_window.gd`**
away into Preferences rows (ruling 19).

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
- **v2.71 half 2, woodland as a spatial area.** Declined 2026-09-21 — Ruling S
  (`LARGE_ITEM_RULINGS.md`): *"Disregard the woodland as a spatial area for
  now. I'm not satisfied with it in the HTML version."* Not a resourcing
  deferral like most other rows here — the owner is dissatisfied with the
  *reference's own* woodland behaviour, not merely blocked on the source being
  unavailable to port it faithfully. Reversible by a word, but a future
  re-proposal should design something better than a faithful port, not just
  wait for `Cartalith_RC` to come back.
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

### 6.4 ~~Six scope documents are stale in the same direction~~ — CLOSED 2026-09-06

All twelve rows corrected and verified. The corrections are visible in each
document (struck through with what is true and the symbol or commit that settles
it) rather than silently deleted, because this project keeps re-examining its own
claims and the reasoning is what makes that possible.

**Two of the twelve were NOT stale and were deliberately left alone** —
`UNIFIED_TOOL_PLAN.md`/`STRANDED_TOOLS.md` and `GPU_LAYER_INTEGRATION_SCOPE.md`'s
`use_gpu` row had already been corrected by `fd9de7c` (2026-09-01) and are still
right today. Declining to "fix" a correct document is the outcome this section
wanted, not a shortfall.

**The batch also found three defects in its own corrections, all fixed:** a claim
that the vault's culture picker was missing (`civilization_workspace.gd:2277`
passes `"culture"` and has since 2026-09-01 — it re-opened a gap that closed five
days earlier); a claim that two milestones completed "the same day", collapsing a
fortnight into one date; and two internal contradictions in `STATUS.md` itself,
where a table row said `done` while the paragraph beneath it said not-started, and
a section header said "Five rows" over six.

**The highest-value correction was `04-left-dock.md`.** Its §0 truncation note and
§9.1's sixteen-row "Lost to truncation" table described a file that has not
existed in that state since `660cbef` re-imported the prototype whole. Thirteen
shipped `.gd` comments had been citing it as authority for values that are
readable, and it reached a ruling in `LARGE_ITEM_RULINGS.md`. That is the concrete
form of this section's own warning: *each of these will cause someone to skip
real, startable work.*

### 6.5 ~~`FUNCTIONAL_CONTRACT.md` disagrees with itself in four places~~ — **CLOSED 2026-09-21 (verified), all four**

Its bodies were not updated when its summary table and absent-list were. The
document explains why — it is a summary no feature commit is obliged to touch —
and it has now gone stale three times in eight days, with its own header
recording corrections on 2026-08-23, -24 and -25.

- ~~Capability 3's body says slider-triggered live re-tuning is absent~~ — **fixed**: `set_params`'s own doc comment confirmed to mark the staleness graph for 25 keys (`params::invalidates`); the narrower true remainder (marking, not auto-recompute) stated precisely rather than erased. Commit `b117c37`.
- ~~Capability 6's body says the atlas/tile cache and the bake lock "remain unbuilt"~~ — **fixed**, `cartalith_engine::bake::AtlasStore`/`FinalizeLock` confirmed live. Commit `66f77b6`.
- ~~Capability 6 lists AO toggles as absent~~ — **fixed**, `render.rs`'s param table and `render_workspace.gd`'s dock rows confirmed live (`ao_strength`, `svf_strength`, `shadow_strength`, `geo_micro`, `sdf_coast`/`sdf_rivers`/`sdf_biomes`). Commit `66f77b6`.
- ~~Capability 13's body says urban milestones 8-17 "remain entirely unbuilt"~~ — **fixed**, all 17 milestone modules and all 17 `_um*` adapter functions confirmed present in `cartalith-urban`/`urban_adapter.rs`, plus a real golden test (`golden_parity_urban_adapter.rs`) the old text said didn't exist. Same-day independent verification by two agents. Commit `66f77b6`.

### 6.6 The reference freeze has drifted twelve versions, not one

**Corrected 2026-09-17; this entry itself understated the drift by an order of
magnitude.** It used to say `Cartalith Gen1 v2.11.html` at the root was the whole
gap. Measured in the working copy: the source repo holds **164** mainline files
(newest **v2.22**) and **44** DCC-line files (newest **v2.66**), while
`reference/` holds only v2.10 and `FUNCTION_INDEX.md` indexes v2.10's 1 094
functions. `FUNCTIONAL_CONTRACT.md`'s *"no drift, no re-freeze question to
raise"* paragraph has been replaced with the measurement.

**The interval is not unknown, which is the one piece of good news here.**
`RC_ENGINE_CHANGES.md` specifies it change by change as a porting spec — but
nothing outside `CLAUDE.md` referenced that document until this pass, so it sat
outside the count entirely. It is §2.9 now.

Still real outstanding work, listed in §2.8, and `CLAUDE.md` requires the index
be regenerated in the same pass.

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

### 6.7 ~~Five documents claim a blocker that has already lifted~~ — CLOSED 2026-09-06

All twelve rows corrected and verified. The corrections are visible in each
document (struck through with what is true and the symbol or commit that settles
it) rather than silently deleted, because this project keeps re-examining its own
claims and the reasoning is what makes that possible.

**Two of the twelve were NOT stale and were deliberately left alone** —
`UNIFIED_TOOL_PLAN.md`/`STRANDED_TOOLS.md` and `GPU_LAYER_INTEGRATION_SCOPE.md`'s
`use_gpu` row had already been corrected by `fd9de7c` (2026-09-01) and are still
right today. Declining to "fix" a correct document is the outcome this section
wanted, not a shortfall.

**The batch also found three defects in its own corrections, all fixed:** a claim
that the vault's culture picker was missing (`civilization_workspace.gd:2277`
passes `"culture"` and has since 2026-09-01 — it re-opened a gap that closed five
days earlier); a claim that two milestones completed "the same day", collapsing a
fortnight into one date; and two internal contradictions in `STATUS.md` itself,
where a table row said `done` while the paragraph beneath it said not-started, and
a section header said "Five rows" over six.

**The highest-value correction was `04-left-dock.md`.** Its §0 truncation note and
§9.1's sixteen-row "Lost to truncation" table described a file that has not
existed in that state since `660cbef` re-imported the prototype whole. Thirteen
shipped `.gd` comments had been citing it as authority for values that are
readable, and it reached a ruling in `LARGE_ITEM_RULINGS.md`. That is the concrete
form of this section's own warning: *each of these will cause someone to skip
real, startable work.*

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

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-06. **This section is deliberately unnumbered, so the
row counter skips it** — it counts rows in numbered sections, which is why
the headline kept climbing while work was being finished: a closure was
filed as a struck row and stayed in the count.

**These are not a changelog.** Each one carries a measurement, a refuted
premise or a reason something was declined, and several were re-opened and
found stale before being closed properly. Read one before re-deriving what
it already answers.

### From 2.3 Civilisation, economy and journeys

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Three phone Controls never get a layout pass**~~ — **CLOSED 2026-09-06** | `DESIGN_HANDOFF.md` | small | **Warmed, not filtered away — and the verifier proved that by reverting rather than reading.** The three are `_phone_overflow_pop`'s rows (a11y `Save project` / `Theme` / `Close world`), reachable from the app bar `⋮` wired at `dcc_shell.gd`'s `_set_phone_overflow_open`. One added line opens that overflow during the probe's warm-up. Measured after warming: **`(601.0, 115.0)` against a 115 px floor at `_phone_scale` 2.6214**, and **`(228.0, 44.0)` against 44 px at 1.0000** — `width_ok` and `height_ok` both true, so no violation was hiding behind the unlaid state. Removing the added line again gives **the same 25 BaseButtons checked** and the three `UNLAID` rows back at `(0.0, 115.0)`: the count did not move, so the walk's filter was not widened. The floor is read from the shell at runtime (`_app.call("_pscale", 44)`), never written into the probe |
| ~~**An unrecognised `entity_kind` fails the WHOLE vault link store**~~ — **CLOSED 2026-09-06, found stale on re-opening the row at its symbol** | `SAVEFILE_COMPAT.md` §13.3.3 | small | **Already fixed, and by a stronger answer than the row asked for.** `KnowledgeLink::entity_kind` is no longer a bare `EntityKind`: it is an enum with a `Known(EntityKind)` arm and an arm carrying the document's own string, so an unrecognised value **keeps its row and its text** instead of dropping it — §13.3.5's *"unresolved"* disposition rather than §13.3.3's minimum. That is preserved-but-inert, which is the round-trip-safe choice: a document opened in an older build and re-saved no longer loses links a newer build understood. The file's own doc records the change and the date, and notes what still fails deliberately — a **non-string** `entity_kind` (`null`, `7`, an object), because absent is not the same as unrecognised. **Re-opened before dispatching a lane at it, per the preflight rule; the row was describing a state that had already gone** |
| ~~**`_phonechrome_probe`'s tap-floor walk goes blind on a fitted control**~~ — **CLOSED 2026-09-06** | `DESIGN_HANDOFF.md` | small | **The walk can no longer pass vacuously, and it was proved by mutation rather than asserted.** Breaking the meta key drops `checked` to 0 and reds the emptiness assertion while the violations assertion passes vacuously; scaling the floor by 1.5 gives `checked=1 violations=1` and reds the floor assertion. **The STOP filter was NOT widened** — that line is byte-identical to HEAD, only its comment changed, and a diff of every `_ok(` line shows two added and none removed, with all four hand-named button assertions surviving. A population of one is disclosed in the probe's own header rather than hidden. **Separately found and correctly left alone:** HEAD's own copy already reports `no tap-floor violations got=6` — six `_fill_scope_chips` chips at 43 px against the 115 px floor — which the lane reported rather than fixing in an unowned file |
| ~~**Ruling 14's spacing consequence is BUILT BUT INERT**~~ — **CLOSED 2026-09-06** | `URBAN_MORPHOLOGY_SCOPE.md` | small | **Wired, proved end to end, and the mutant killed by the verifier as well as the lane.** `landmark_run_inner` now builds `icon_marks` from `WorldGen::icons` through `icon_to_mark` and assigns `inputs.manual_icons`, matching the `settlements` shape three lines above it (`filter_map`, because `icon_to_mark` returns `Option`). `_lmicon_probe.tscn`, three runs on one world: 267 landmarks with `peak@74,11` present; a manual icon placed on that cell moves the kind to `peak@40,62`; deleting the icon reproduces run A, all 267 keys in order. Replacing the assignment with `let _ = &icon_marks;` gives `THE WIRING IS DEAD` and 2 FAILED. **No golden moved and none could** — the verifier derived independently that no golden covers the landmark pass at all; the only test file naming it is self-declared diagnostic-only, and the change is confined to `cartalith-godot`, which no golden crate reaches. **The last of ruling 14's three consequences** |
| ~~**Every export path skips `render::apply_color_space`**~~ — **CLOSED 2026-09-06 as CORRECT, with a reason** | `TERRAIN_APPEARANCE_SCOPE.md` | small | **Investigated and deliberately not changed — and my "latent" framing was wrong.** It is live today, not latent: `render_workspace.gd` already wires the picker to `set_color_space`. **The export writes the WORKING space on purpose.** `WorldGen::color_space`'s own doc says it *"describes the monitor in front of this session"*, which is why it is excluded from saved looks and from `AppearanceDoc` — `GUI_GAP_REGISTER.md`'s *"display = app, working space = document"*, and an exported PNG is a document. Decisive: `encode_png_rgb8` passes **no ICC profile**, so an untagged P3 file is read as sRGB by anything colour-managed — applying the transform would make the file wrong everywhere and leave the screen unchanged. **Proved rather than argued:** with a positive control showing Display P3 moves the viewport raster by up to 23 byte levels, the export written in that same session is **byte-identical** to the sRGB one (3 238 073 vs 3 238 073, whole-file compare). **The real fix if ever wanted is an ICC-tagged encoder, not a second display transform** |
| ~~**`_exportraster_probe` section 13 has been red at HEAD**~~ — **CLOSED 2026-09-06, and it was a real shipped rendering bug** | `TERRAIN_APPEARANCE_SCOPE.md` | small | **The cause was none of the three this row and its brief guessed at, and the framing was the reason.** Section 13 does not compare tiled against single-file (that is section 8, which passes) — it compares **`build_color_texture`'s on-screen texture against the exported PNG**. They ran the same code on **different river masks**: `58dd5b2` (2026-08-30, *"Rivers have a width now"*) added `stamp_river_intensity` and a private `RiverInk` beside `build_color_texture`, so the screen composites `r + (fr - r) * ink` from a continuous stamped disc, while `render::bake_rect` kept a binary `mask[i] != 0` and tinted at full strength. **Two-sided:** channel cells with a partial stamp were over-tinted in the PNG, and halo cells were drawn on screen and absent from it. **So since 2026-08-30 the exported image did not match the map.** `RiverInk` is now `pub` in `render`, `bake_rect` reads the same ink at the same nearest cell, and `WorldGen::river_ink()` is the single copy of the stamp-vs-flag rule for all three export sites. Section 13: **199 909 of 8 060 928 bytes differ / worst 73 → 15 differ / worst 1**, inside the documented f32-prologue bound. Nothing was loosened — both assertions are untouched |
| ~~**The 16K/32K codec question**~~ — **RULED 2026-09-06: PNG** | `EXPORT_SCOPE.md` | small | **Owner ruling 26:** *"For the codec in export use png, even if size balloons. We should just inform the user of the expected file size."* Well-founded on the survey already banked — **WebP dies at 16 383 px**, below the smaller target size, and **JPEG XL dies on its AGPL encoder**; size was the only argument against PNG and the owner spent it deliberately. **Scope clarified the same day:** *"a user generated monolithic image of the map. No layers, no extensive information. Just to be used outside of Cartalith in an image viewer"* — one flat raster, no sidecar, no tiling, nothing to negotiate. **Three consequences carried into the un-shelve row, not re-decided here:** export export **RGB, not RGBA** (no overlay data means no alpha — at §6's real target dimensions, **32 768 × 20 976 drops 2.75 GB → 2.06 GB** and **16 384 × 10 488 drops 687 MB → 515 MB**, and it removes the transparent-sea-on-white failure this use case invites); **estimate size AND memory before the run**, from measured bytes-per-pixel rather than a generic PNG rule of thumb, since map imagery compresses unusually well; and **PNG is scanline-ordered**, which is what makes the reverted byte-identical banded renderer directly reusable |
| ~~**The prototype's segment on-state is `--wash2`; the shell paints `accent_wash`**~~ — **CLOSED 2026-09-06, measured first** | `DESIGN_HANDOFF.md` | medium | **Measured before anything changed, which is what the row asked for, and the answer was that it is a real defect.** Windowed at two bands and both palettes: the `.09 → .16` step is **16/255 (light) and 15/255 (dark)** at worst. Against the cue the design already ships as legible — ground→lit is 34-37/255 in light at `.16` — **the missing step is roughly three quarters of the entire on/off signal**, so the lit state was reading as barely-lit. **No token value moved.** The fix is one line in `set_segment_on()` pointing at the `accent_wash_2` that already existed, so the blast radius is exactly the 18 `set_segment_on(.., true)` sites; `accent_wash`'s 23 row/hover/chart consumers keep `.09` by construction. Reverting the line gives `lit segments=0` in both palettes. Direction was never in doubt: the prototype uses `var(--wash2)` 36 times against `var(--wash)` 13, with the `seg` branch three lines above an override row using the lighter one |
| ~~**History draws its undone steps from `RedoTail`, which names only one**~~ — **CLOSED 2026-09-06** | `GUI_GAP_REGISTER.md` ED-02 | medium | **The dock now draws the whole undone list and can discard it, and undo semantics did not move.** `redo_labels()` returns the full tail as a `PackedStringArray`, `discard_redo_tail()` drops it, both now forwarded through `EngineBridge` — the missing forwarder is the failure that reads as `Nonexistent function` and had been misdiagnosed as a stale build twice in one week. **The artboard settled the shape and an earlier quote of it was truncated**: its footnote reads *"steps below it are undone, not deleted / **the next edit drops them** / reverting above COMMITTED asks first"*, and it draws *"(X) discard the 2 undone steps"* — so the second clause is the tail's lifetime, and no confirmation dialog was invented. **Invalidation display is the property that could have rotted and it was driven:** 2 undos → 2 drawn rows; a direct `erode_op()` drops the tail; rows still drawn that frame (positive control) and gone after exactly one. **The one-frame dead click is inert** — clicking a stale row left the ledger 3→3 and the undo depth 2→2 and resurrected nothing. Nothing reached the save format |
| ~~**History's `COMMITTED` rule is session-scoped**~~ — **CLOSED 2026-09-06, verified by a later batch's verifier** | `GUI_GAP_REGISTER.md` ED-02 | small | **The lane died before reporting; the next batch's verifier ran the probe it never ran.** Three separate processes: save, then `touch -d '3 days ago'` on the archive, then **open in a genuinely new process** — the rule is drawn at the Open-project floor for a file that process never wrote, `saved_seq=2`, `saved_at_ms` matching Godot's own `FileAccess.get_modified_time()` to the second, and the age reading **"saved 3 d ago"** — the *file's* clock, not the session's. A legacy phase over `cartalith-io`'s real archive fixture reads "saved 21 d ago". **Never-saved is spelled absent, not zero:** `undo_stats()` omits both keys, no rule is drawn at seq 0, and the note says *"has not been saved"* rather than naming a revert. **No second dialog:** `AcceptDialog|ConfirmationDialog` counts 21 in `shell/*.gd` at both `da57ca4^` and `da57ca4`; only `_confirm_revert()`'s predicate widened, `<` → `<=`. **Stated limit:** *only-on-`Ok`* was confirmed structurally — `mark_saved_now()` sits inside the `Ok(())` arm and every earlier path returns first — but **no test forces a refused write**, and nobody claimed otherwise |

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Six phone scope chips sit at 43 px against a 115 px touch floor**~~ — **CLOSED 2026-09-06** | `DESIGN_HANDOFF.md` | small | **Four options measured windowed at 412 dp rather than argued.** Baseline: chips 17 dp, scroll 340, 6 result rows fit. Flooring alone at the existing 460 sheet gives chips 92 dp but **evicts a row** (scroll 286, 5 rows) — the same eviction the 360→460 raise was made to prevent. **Chosen: floor all six and raise the sheet to `_pscale(514)`** — chips 92, scroll 340, 6 rows, and the sheet still fits at 412×915, 1080×2340 and 1440×3168. `no tap-floor violations got=6 → got=0`. **The exemption was declined on a real argument, not waved away:** exempting only the disabled `l` chip saved a row at 460, but `FindOnMap.dc.html` says that scope ships the day a placed landmark has an identifying name, so an exemption keyed on `disabled` would silently put a **live** chip back under the floor that day. **And `got=6` can no longer go quietly to zero** — new `_phonechrome_probe` §4a asserts all six drawn, none unlaid, none under the floor by name, two chip rows, six result rows and the sheet fitting |
| ~~**`_datapane_probe.gd:398` is red**~~ — **CLOSED 2026-09-06, seed-independently** | `GUI_GAP_REGISTER.md` | small | **The row was that the correct assertion is seed-dependent, and that is what got solved.** The new form asserts the *property* rather than a fixed expectation, and the verifier ran three worlds to prove it: seed 40417 (178 rivers, chip matches the document), 1234 (297, matches), and a 48×36 world at sea level 0.96 with **no rivers at all**, which exercises the dashed branch — *"dashed chip 'rivers' means the document carries none"*, `got=0 want=0`. Both directions, no pinned seed |
| ~~The phone shell with **no world open** scans 1 494 of 2 400 rows blank~~ — **CLOSED 2026-09-06, and the number was stale in the row itself** | `DESIGN_HANDOFF.md` | medium | **Re-derived per surface at 1080×2400, `phone_scale` 2.62, windowed, dark forced — and the register's own tracked probe now gives different figures on every leg**: 1 494 → **1 787**, 1 047 → 995, 694 → 642, 291 → 239. Nothing in that probe changed; the shell under it did. Opening the planner removes **792** rows, not 447 — same conclusion, wider margin. **The cause is not any of the three this row assumed.** No spacer, no empty list, no uncollapsed container: **1 556 of 1 787 (87%) is `_phone_content_gap`**, the `SIZE_EXPAND_FILL` map region with `ViewportHost` behind it, and it goes to **0 blank** on generate with no layout change at all. Three of the five remaining surfaces are unchanged (nav 88, status 48, app bar 26); the tool sheet drops 27 → 6 and the gesture inset 42 → 11, totalling 179. **And 1 787 is not the first screen:** cold boot as shipped is **212 of 2 400**, because `open_welcome()` draws the project picker over it — 1 787 is only reachable once the user dismisses that. **The first-screen question was already answered in code**: the welcome path *is* the first screen, and behind it the default tab carries an enabled 392×126 px `GENERATE WORLD` that was pressed, not read. **The one real defect was a route named in words** — see the row below |
| ~~**The empty phone shell told the user to use a menu the phone does not draw**~~ — **CLOSED 2026-09-06** | `DESIGN_HANDOFF.md` | small | **Found while censusing the empty shell.** `MORE ▸ STATUS ▸ Next` read *"File ▸ New world… to begin"*, but the `MenuBar` is parked in a hidden host and `phone_menu.gd` re-titles that menu **Project** — so **`File` is drawn nowhere on a phone**, and the one instruction the empty shell gives named a path that does not exist. Fixed with `DccShell.new_world_route()` (static, off `DccTheme.is_phone()`) across **6 call sites in 3 files, counted in the same edit that wrote the number**. Asserted in both directions rather than one: the phone leg reads *"MORE ▸ Project ▸ New world… to begin"*, the desktop leg still reads *"File ▸ …"*, and a hardcoded string fails one leg or the other |
| ~~The phone inspector's widest rows demand 1 408 px on a 1 080 px screen~~ — **CLOSED 2026-09-06** | `DESIGN_HANDOFF.md` | small | **Reproduced by revert-measure-restore before anything changed:** actions 1 396, footer 1 265, `_inspector_body` **1 408**, and `overrides: 0` sitting entirely off-screen at 1 278..1 406. **Fixed by flowing, not by stacking** — the actions row lays two lines with button minimums intact and no text wrapped; the footer flows four pairs with each pair an atomic `HBox`. **After: 0 of 25 rows exceed 1 080 in all nine phone samples**, body 733-826, and **23 tappable controls with 0 under the 115 px floor** (44 dp × `phone_scale` 2.6214). The 15 override rows were **left alone and proved reachable rather than assumed** — 469-588 min, laying out 22..1 049, no horizontal scroll needed; `inspector_scroll` stays `AUTO` as a guard |
| ~~The Colour relief layer row is live over a layer that draws nothing~~ — **CLOSED 2026-09-06: the row folds while the ramp is dark** | `GUI_GAP_REGISTER.md` | small | **Measured before it was judged.** At `ramp_strength` 0.0 all four controls on that row — hide, opacity 0.25, blend Multiply, reorder to the top — move **zero bytes** across three seeds; at 0.35 the same four move 39-55% of channel bytes. So the row now draws its name, an em-dash in the dot's slot and *"not drawing"*, and the controls appear when they do something. **Folded to a line rather than dropped from the list**, because reorder steps through the *engine's* stack positions and a hidden row is still a position — Hillshade's Down would otherwise swap with something invisible. **Both docks flip together without a second gesture:** `set_appearance` has no signal and the right dock rebuilds on four events a slider drag is none of, so the commit emits `layer_stack_changed` when the value crosses zero. **The retired left-dock note carried two false claims**, both found by the lane: it sent readers to *"Ramp strength (Rendering - advanced)"* — no such control, the slider is labelled **Colour relief** under **Terrain appearance** — and it gated on `appearance().get("ramp_strength", 0.0)` while `appearance()` returns `{}` with no API, so a build that could not answer read as one answering 0.0 |

### From 3.1 Blocked on an owner decision

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Timeline stores mutations, not per-year snapshots**~~ (ruling 27) — **BUILT 2026-09-06; verified without a lane report** | `SAVEFILE_COMPAT.md` §10.2 | medium | **The lane errored out and filed nothing; the verifier built its own oracle and confirmed the work directly.** An independent integration test held a whole `Vec<i32>` per year — the old shape — and compared **field by field**, naming the first divergent index, after every operation: **24 trials × 180 interleaved ops ≈ 4 320 mutations**, plus a 400-entry chain with 300 mid-chain inserts and removal of every keyframe down to two entries. All pass. **Measured, host-polled** (the console binary is a *wrapper*, so polling its PID reads ~6.4 MiB and means nothing — the verifier had to enumerate the spawned process): 0 years 323.5 MiB steady, default collapse **+11.2 MiB** against a predicted 11.16, 200 years 377.0, 800 years 507.0, 2 000 recorded 762.9. **`TERRITORY_KEYFRAME_INTERVAL = 64` is pinned two-sided** (32 kills 2 tests, 128 kills 1). **No format change was owed** — §10.2's on-disk shape is untouched, so no `format_version` bump; the brief's item 3 premise was false. **Still open:** carry-forward costs ~0.215 MiB per recorded year with empty payloads, linear and unexplained — a symptom, not a cause |
| ~~**LOD tiles stored in the save, optionally**~~ (ruling 28) — **SLOT BUILT, DEFAULT OFF, NOT YET WIRED** | `SAVEFILE_COMPAT.md` | medium | **Measured on three real 2048×1311 worlds before any default was chosen.** Levels 0..=6 deflate to **21.87 / 23.58 / 27.75 MiB against a whole archive of 24.9 MiB** — it roughly doubles the file, confirming this was the one slot able to outgrow the three float grids. **Ruling 28's "~4/3 of its base level" is exact for RAW bytes (1.333 measured) and wrong for what lands in the file: 1.23-1.26× compressed**, because deeper levels carry more sub-cell detail and compress less. No fixed depth covers the reachable zoom (`level_for_zoom` returns up to 10; level 7 alone extrapolates past 100 MiB), so the slot takes a **caller-chosen depth**, not "the pyramid". Single-channel `.u8` beat RGB PNG by **2.9-3.0×**. **Default OFF** with `pyramid_mask_bytes()` (exact, synthesizes nothing) as the size shown at save time. **StageGraph could not do the half that matters** and the lane said so rather than substituting silently: every stage starts at version 0 in a fresh graph, so two worlds are indistinguishable and nothing survives a process — a content key carries the cross-session half. **Remaining: the `ProjectWrite.lod_tiles` assignment in the save path, and a warnings channel for a dropped pyramid.** **Owner ruling 29 (2026-09-06) settles where the user meets this:** *"the tiled output should only live in the save menu. It has no merit in the export menu"* — so ruling 28's "size shown at save time" belongs to the **save affordance**, and the round-3 canvas's Gaea-derived build dialog and build manager under `Data ▸ Export` are **not built**. There is no build type, tile grid or blending percentage, because those describe a file layout the user chooses and the destination is the archive's own slot |

### From 3.3 Blocked on hardware, or on a design that does not exist

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**CA-09 — layer list search; Blocks / Verticality**~~ — **CLOSED 2026-09-06, verifier all-confirmed** | `GUI_GAP_REGISTER.md` §7.16 | small | **One field over two lists that stay separate.** `VISIBLE LAYERS` rows are `DccWidgets.toggle()` checkboxes writing `set_layer_visible()`; `DATA OVERLAYS` rows are `_row()` buttons calling `_on_pick()` — **different factories, so no row can inherit the other's behaviour**, and the verifier drove both under a live filter to prove it. **The pool is 51** (8 `LIVE_LAYERS` + 43 `debug_layers()` items), measured at runtime — my brief said "3 of 16", which would have been a plausible-looking wrong denominator. **The hotkey property held and was mutation-proven:** digits are assigned over the *unfiltered* walk and `_input()` dispatches from `_hotkey_ids` rather than a drawn row; moving the filter above the digit assignment makes 0 of 8 reach their view. **Found while measuring, not in the brief:** the new field drew **22.0 px against a 44 px floor** at tablet density, because `phone_fit()` floors `LineEdit`s but only runs on phone — now 44.0, asserted per density. **Blocks / Verticality: built nothing**, recorded in four places; `exag` is a live slider in CARTO ▸ Map style ▸ § Map view, tiles are `dcc_settings.gd` §2.5, bundles are `STYLE_PRESETS` |
| ~~**JP-05 — the calculation trace**~~ — **CLOSED 2026-09-06** | `JOURNEY_PLANNER_SCOPE.md` | medium | **Four columns — step · read off · value · leg — and a reconciliation row that can be caught lying.** The running column now reaches **days**, which is the actual complaint; it used to stop at km/day. **My brief said a non-zero reconciliation would reveal a term hidden inside `jp_plan_ex`. Nothing is hidden:** all 14 land and 8 water factors already cross as `land.trace`/`water.trace`, the product matches `daily_km` **to 12 decimal places on 29 legs across 3 seeds**, and no engine call was added. **And the six steps I specified would have shipped the lie the brief forbade** — they cover 4 of 14 land terms, leaving a **×25.8-49.3** discrepancy, so all 14/8 are drawn instead. **The positive control is the part that makes the row trustworthy:** popping the `hours` factor (×8) out of a live result redraws the row at +14.67 / +41.54 / +31.97 d, each exactly `days × (factor−1)`, in the block-coloured branch. **Also corrected:** a land leg has **no crossing term at all** — `riv_x` never reaches `daily_km`, its only consumer is a verdict reason |
| ~~**CA-08 — the style preset gallery**~~ — **CLOSED 2026-09-06** | `TERRAIN_APPEARANCE_SCOPE.md` | small | **Only the gallery was missing, as the design said, and the thumbnail was the one real decision.** Five `STYLE_PRESETS` bundles, chips, the custom-state sentence and ten authored blurbs all shipped already |
| ~~**WW-03 — four named sculpt falloffs**~~ — **CLOSED 2026-09-06; and the "one line each" estimate held** | `MVP_SCOPE.md` | small | **The first question was whether `hardness` already reached them, and it does not — measured, not argued.** Sweeping all 101 hardness steps at 0.01 against each shape, the closest smoothstep to **Linear is still 0.0962 off** in coverage and to **Sharp 0.1071**, and **Constant is unreachable outright** because `feather` is floored at 2 cells, so even `hardness = 1.0` keeps a ~2-cell ramp. All three needed a branch; none was a preset in disguise. **Smooth is bit-identical** — its arm is the same call expression the three `cov =` sites used before the enum existed, so identity is by control flow rather than arithmetic, and mutating it to `clamp01(t)` fails 21 of 23 golden cases. **11 of 13 features separate visibly** at the weakest pair (mountains 0.0386, plateau 0.0448, volcano 0.0475); ridge (0.000002) and basin (0.002421) fall below one 8-bit step and are disclosed, ridge pinned by its own test and named in the dropdown tooltip. **No curve editor**, on Blender 5's own reversal |
| ~~**`Falloff::Sharp`'s exponent was claimed covered and was not**~~ — **CLOSED 2026-09-06** | `MISTAKES.md` | small | **A floor cannot catch a shape change that stays above the floor.** `falloff_shapes_are_not_reachable_by_hardness` asserts `peak > 1/255`, which answers *"do these two draw differently"*, not *"is Sharp a cube"* — and its own comment claimed it would turn red *"say by changing `Sharp` from a cube to a square"*. **It does not:** the verifier made exactly that change and the whole crate stayed green. Fixed by `sharp_is_a_cube_and_not_merely_steeper_than_the_others`, pinned with **hand-computed literals** (0.25³ = 0.015625, 0.5³ = 0.125, 0.75³ = 0.421875) rather than `powi(3)` re-derived from the arm under test, plus a guard that the other three shapes do **not** satisfy the cube. Re-running the mutation now FAILS it; restored, 148 pass. The false comment is corrected in place |


### From 1. In flight right now (second sweep)

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Economy milestone 2**~~ — the food-surplus cluster | `ECONOMY_SCOPE.md` | small | **CLOSED 2026-09-05, and the row was wrong about what remained.** It said *"no dock or window calls `TradeStore.food_shed_for(index)` — confirmed by direct search"*. `place_editor_window.gd` already called it; `grep -rn food_shed --include=*.gd` finds it. The surface existed. **What the pass found instead was worth more than the row:** the food-shed, smelting and salt readouts each carried **two dashed branches holding each other's reason** — the same inversion defect Δ vertical shipped a batch earlier, so it is a pattern in how this shell writes two-branch dashes, not a one-off. All three fixed and each branch exercised in a probe rather than reasoned about. **One residual, disclosed and deliberately not fixed:** `food_shed_rows()` recomputes `lithology`/`soil` per call instead of reading a `CivData` field — an efficiency nicety in `lib.rs`, which was not that lane's file. `coppice_ha_needed` is emitted by the binding and drawn by no surface: a missing reader, not a missing value, so it is deliberately **not** dashed |

### From 2.7 Android and on-device verification (second sweep)

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**HE-02 / PR-16 — the keyboard shortcuts editor**~~ | `GUI_GAP_REGISTER.md` §7.9 | medium | ~~Open~~ **ALREADY BUILT, found 2026-09-06 by opening the symbol.** Committed in `1111611`: `shortcuts_dialog.gd` carries `open_editable()`, `_begin_capture`/`_commit_capture`/`_unhandled_key_input`, `_capture_chips` and `_reset_one`/`_reset_all`; the override store is `dcc_settings.gd:612-660` (`shortcut_binding`, `set_shortcut_binding`, `clear_shortcut_binding`, `reset_shortcuts`, one ConfigFile section per context); and **`menus.gd` already applies it** — `_bind_accelerator` reads `DccSettings.shortcut_binding(...)` at build, the single `set_item_accelerator` call site. Conflicts are shown and never blocked, naming **every** other row sharing the chord. **My brief called it unbuilt, taking that from the design document — the fifth existence claim from a design doc that did not survive contact with the code** |
| ~~**`lib.rs`'s `atlas_evict_to` doc described a floor that is no longer true**~~ | `GUI_GAP_REGISTER.md` | small | ~~Open~~ **CLOSED 2026-09-06 by the main loop.** The atlas lane changed the eviction floor and reported the stale prose rather than editing a file outside its grant; the verifier confirmed it at the symbol. Two clauses were false — *"a world is never taken below its coarsest level"* and *"nothing outside the protected coarsest levels is left to free"*. **The floor is one CHUNK, not one level:** a world is never emptied (`chunks > 0`, `finalized` false — the state `atlas_clear` refuses to manufacture), but a single-level world can be cut to one chunk, so a 64-chunk `z = 3` world drops to 1/64 coverage. Both clauses rewritten to say that |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-06. **This section is deliberately unnumbered, so the
row counter skips it** — it counts rows in numbered sections, which is why
the headline kept climbing while work was being finished: a closure was
filed as a struck row and stayed in the count.

**These are not a changelog.** Each one carries a measurement, a refuted
premise or a reason something was declined, and several were re-opened and
found stale before being closed properly. Read one before re-deriving what
it already answers.

### From 2.3 Civilisation, economy and journeys

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**The Asset Library window has no internal search**~~ — **STALE; the headline was false and the real defect was fixed 2026-09-06** | `GUI_FEATURE_PARITY_SCOPE.md` | medium | **Refuted by a lane and confirmed by a verifier, then left open here by me.** `_build_window_bar()` builds a search `LineEdit` on **both** compositions — `search_phone` with `SIZE_EXPAND_FILL`, `search` with `custom_minimum_size.x = 340` — each `text_changed`-connected to `_search_text` + `_refresh_grid()`. **The real defect was different and is closed:** the placeholder named four fields the filter never read. Verified windowed — `tund` narrows biomes 15 → 1 and clearing restores 15; in pixels, busy rows inside `_grid` go 269 → 69 → 269; mutating the `set` clause to `or false` kills it |
| ~~**Four probes still call `_set_panel_picker_open`**~~ — **CLOSED 2026-09-06; three were errorless and BLIND** | `GUI_GAP_REGISTER.md` | small | **The dangerous case was not the one that errored.** `_menuconf_probe`'s two overlay dumps ran clean and captured **zero rows of either overlay** — 72 and 100 lines of shell chrome — because `_dump_tree`'s `max_depth = 7` stops above `_phone_root`, where both overlays parent. Reproduced by the verifier on HEAD and after: now 22 and 74 lines carrying `Save project`, `Theme | light`, `Close world` and PhoneMenu's `MORE`. **`_appbar20_probe`'s call never executed at all** (inside the `else` of `if picker == null`), and `_shot_phone` needed no change but gained an argument allowlist, proved by negative control. **A probe that errors loudly is a nuisance; one that errors quietly and still prints PASS is a false instrument** |
| ~~**The timeline scrub row's 44 px touch floor is declared, not drawn-measured**~~ — **CLOSED 2026-09-06, and the defects were on the axis nobody asked about** | `DESIGN_HANDOFF.md` | small | **Every tappable HEIGHT on the tablet row was already 44 px. Both real defects were WIDTHS:** a transport square at **36 × 44** where `Timeline.dc.html` board H draws 44 × 44, and the collapse chevron at **7 × 44** — the narrowest target on the row, because `text_button()`'s tap floor is gated on `is_phone()` and `tablet_fit()` is called on `tool_options_row` alone, never on `timeline_row`. **A height-only walk would have reported the row green.** Also found: the speed pills **declare** 34 px (tier B) and draw 44 only because the transport squares set the HBox height — shrink those and all three silently drop, so the declared floor was moved to 44 at the call site. **The scrub track needed no exemption** — it already draws 44 from `maxf(timeline_track_h, btn_min_h)`, giving 60 368-111 408 px² of hit area, and its rail and marks are `MOUSE_FILTER_IGNORE`. The one exemption written is the collapsed 33 px strip, height-only and touch-gated, against board H's own 34 px. Cost: row minimum 1 094 → 1 155 px at tablet, unchanged at desktop |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-06. **This section is deliberately unnumbered, so the
row counter skips it** — it counts rows in numbered sections, which is why
the headline kept climbing while work was being finished: a closure was
filed as a struck row and stayed in the count.

**These are not a changelog.** Each one carries a measurement, a refuted
premise or a reason something was declined, and several were re-opened and
found stale before being closed properly. Read one before re-deriving what
it already answers.

### From 2.3 Civilisation, economy and journeys

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**The phone's year scrub draws 96 × 84 px against a 115 px floor**~~ — **CLOSED 2026-09-06** | `DESIGN_HANDOFF.md` | small | **Verified on all three phone compositions, both axes.** 1080×2340 (floor 115): play / ×1 / ×10 / ×100 / ✕ all 115×115 and the slider **966×115**; 1440×3168 (floor 154): five at 154×154, slider 1286×154; 2340×1080 landscape: five at 115×115, slider 888×115. **6 of 6 clear on both axes at every composition**, and nothing was starved to get there — the row's only other children are the year `Label` (139×40, not a target) and a `MOUSE_FILTER_IGNORE` spacer, with every `_ptap` sibling unchanged. `_tlfloor_probe --phone` now reports **0 of 6 under the floor, was 1 of 6** |
| ~~**CA-07 — label roles**~~ — **CLOSED 2026-09-06, and it overturned the brief twice** | `TERRAIN_APPEARANCE_SCOPE.md` | medium | **A role default is a TEMPLATE plus an explicit counted apply — never a silent overwrite, and deliberately not a fallback.** Reason at the symbol: `size`/`size_mode` are plain fields rather than `Option`, so a fallback needs a Rust change and a sentinel would be "no value as a plausible value" (8.0 is a real size). Proved by driving the real slider: a hand-edited label stayed **41.0 px** while the role base moved 15 → 19, and an untouched label kept its own 15.0 — *a default is not a fallback*. Apply names its count (*"Apply base to Water (2)"*), touches only the selected role and disables itself. **My brief was wrong twice and the lane refused both:** `halo` and `tracking` are **fully live** — `set_field` accepts them, `labels_render_list` emits `halo_em`/`tracking_em` for generated *and* hand-placed rows, and `map_overlay.gd` strokes them — so dashing halo would have shipped a false reason on the one field that keeps a name legible over terrain. Only `family`, `weight` and `case` are absent |
| ~~**A second route into a role desynced the `Class` dropdown**~~ — **CLOSED 2026-09-06; created by the fix, found by the verifier** | `GUI_GAP_REGISTER.md` | small | **`DccWidgets.choice()`'s return was discarded, which was harmless while the dropdown was the only route in.** Adding a row press gave the panel two views of `_label_class` that disagreed **on screen**: pressing Water moved the marks, the title, the three dials and the Apply button while the dropdown still read *Settlement*. No data was lost — every consumer reads `_label_class` — but the panel contradicted itself. **The lane's own probe asserted every other consumer and not this one.** Picker now retained and re-seated in `_sync_label_class()`, pinned per role (*"reads Landmark, want Landmark"*), and the probe fails loudly if it is not retained at all |

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**The 14 undesigned surfaces**~~ — **ALL DESIGNED AND ALL FOURTEEN BUILT** | `DESIGN_HANDOFF.md` | small | **CA-07 label roles landed 2026-09-06, the last buildable screen.** The owner supplied three Claude Design canvases covering exactly the 14. **PR-15 units is NOT an open decision — I was wrong about it twice** (see the row below): `Preferences ▸ Units` ships as a three-way radio, so nothing is owed. **Seven of the canvases' "Exists today" claims failed against the code** across the round — twice turning a build row into "already shipped", twice telling a lane to dash a field that was fully live |
| ~~**Does every readout go through `DccUnits`, or do some still print raw km?**~~ — **CLOSED by the 2026-09-08 units routing pass** (re-classified 2026-09-13) | `GUI_FEATURE_PARITY_SCOPE.md` | small | **Opened 2026-09-06 by correcting my own false claim, twice made.** I told the owner there is no km/mi toggle anywhere in `shell/*.gd` and that PR-15 needed a ruling to delete it from the canvas. **It exists and is built:** `Preferences ▸ Units` is a three-way radio — **km / mi / nmi** — persisted through `DccSettings.units_mode()`, with `DccUnits` as the display-layer conversion every readout picks up on its next draw and `viewport_host.gd`'s scale bar following it; `BUILD_ANSWERS.md` records units persisting beside device and theme. **My grep searched for `miles|km/mi` and the code says `DccUnits.label("mi")`** — a claim about absence resting on one search rather than on opening the symbol, which is the exact failure this file tracks in other people's work. **So PR-15's answer is Reading 2, not Reading 3** — it is a formatting-site conversion, which is what `DccUnits` already is, and the batch-3 canvas's *"the engine is kilometre-only"* is true but not a contradiction. **What is actually open is narrower and checkable:** whether every readout routes through `DccUnits`, or whether some still print raw kilometres. Belongs in the menu-conformance audit **Answered since:** the routing pass sent the CIVIL and INFRA readouts and `right_dock.gd` through `DccUnits` and `_civunits_probe` polices it (see the archived *"Units: `right_dock.gd` is the largest surface still printing raw kilometres"* row). |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-06. **This section is deliberately unnumbered, so the
row counter skips it** — it counts rows in numbered sections, which is why
the headline kept climbing while work was being finished: a closure was
filed as a struck row and stayed in the count.

**These are not a changelog.** Each one carries a measurement, a refuted
premise or a reason something was declined, and several were re-opened and
found stale before being closed properly. Read one before re-deriving what
it already answers.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Menu-by-menu design-conformance audit**~~ — **RUN 2026-09-07; seven menus, enumerated from the live MenuBar** | `DESIGN_HANDOFF.md` | medium | **Owner request 2026-09-04, and it changed nothing it audited** — the verifier stripped and filtered the whole diff: **117 changed lines in `menus.gd`, every one a comment**, no guarded file touched. Enumerated from the code rather than the design set, so it could find rows the designs do not list: **File 20 top-level entries / 31 with submenu leaves; Edit 16 / 22**, bar = 7 MenuButtons, identical with and without a world. **Findings are filed as their own rows below rather than folded into this one**, because three need an owner and the rest are separately sized |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-06. **This section is deliberately unnumbered, so the
row counter skips it** — it counts rows in numbered sections, which is why
the headline kept climbing while work was being finished: a closure was
filed as a struck row and stayed in the count.

**These are not a changelog.** Each one carries a measurement, a refuted
premise or a reason something was declined, and several were re-opened and
found stale before being closed properly. Read one before re-deriving what
it already answers.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**`_ds03fit_probe` is RED — a live canvas-width conformance failure**~~ — **FIXED; 372 HOLDS AND THE SHIPPED DOCK WAS WRONG, 2026-09-07** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | medium | **The mechanism accounts for exactly 32 px and was reproduced by the verifier in a clean worktree.** `DccWidgets.choice()` leaves `OptionButton.fit_to_longest_item` at Godot’s default `true`, so a dropdown’s minimum is its **longest list item**, not its selection: the sampled control shows **"Auto" at 59 px** and reserves **"Established Caravan Route" at 185 px**. `journey_planner_view.gd::_choice_field()`’s row is `Label(132) + OptionButton(185) + 8 = 325`, and `_build_left_dock()` writes `--ldW` as a `custom_minimum_size.x` **floor with no ceiling**, so `_scroll()`’s `SCROLL_MODE_DISABLED` folds the child minimum outward: row 343 → `MarginContainer` 369 → the planner’s nested `ScrollContainer` 397 → body 397 → +6 drag handle → +1 border = **404** against a **365** budget (372 − 7 furniture). **Nine of ten rail nodes measure +42…+149 headroom; the planner measures −32 — it is the outlier, not the token**, and the canvas is explicit (`ENV:25` `--ldW:372px`). **Fixed dock-scoped, not in `choice()`**: `dock_fit()` + `_on_dock_node_added()` in `dcc_shell.gd` clear the flag on dock `OptionButton`s **that expand**. It is not in `choice()` because that also serves the phone tool sheet, where `phone_fit()`’s `wide` guard deliberately KEEPS the flag — PAINT ▸ Class collapsed to 35 px when it did not. **PC at 1920 is now 10/10 conformant**, and `dcc_theme.gd` is byte-identical to HEAD so no token was re-based and the 20/20 palette is untouched by construction. **The cost is stated, not hidden:** 3 of the planner’s 24 dropdowns ellipsize *when their longest item is selected*, against a dock that was permanently stealing 32 px from the map. **And the failure was undercounted:** it is **four** failures, one cause — two on the desktop leg (404 vs 372) and two on `--force-touch` (455 vs `W_DOCK_TABLET` 400) |
| ~~**The zoom readout still says `z1.1` where the canvas says `zoom 110%`, and the comment defending it is false**~~ — **FIXED AND THE FALSE COMMENT IS GONE, 2026-09-07** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | **The retired claim survives only as a labelled quotation inside the paragraph that refutes it** — quote-then-refute, which is the right shape, not leftover misinformation. Re-verified at the symbol by the verifier: `cartalith-dcc-parts.js:221` is `'zoom '+Math.round(v.s*100)+'%'`; `ENV:1736` and `ENV:1716` clamp with `Math.min(4, Math.max(0.12, …))`; `view.s` opens at 0.34 at `ENV:1245`. **No world size enters the writer**, so the "no percentage referent" defence was false and the referent is a plain multiplier. **A brief error of mine went with it:** I cited `ENV:914` for the readout. Measured: **`ENV:913` is the readout span, `:914` the HUD column, `:915` the km label, `:916` the bar** — so my own brief was internally inconsistent, citing `:916` correctly for the bar and `:914` wrongly for the readout one sentence apart |
| ~~**Two mutation survivors in `PgField`: a dead default and an unasserted caret**~~ — **BOTH CLOSED, BOTH CLASSES, 2026-09-07** | `ANDROID_UI_SPEC.md` | small | **The dead default was fixed in `PgSlider` AND `PgField`, not one of them** — both are now `var slop: float` with no initialiser, so the attacher’s value is the only one. Verified by the re-check: re-introducing `400.0` in `PgSlider` leaves `_rangeswipe` green and in `PgField` leaves `_gestclass` green, **proving both defaults really were unreachable**; deleting either attacher’s set line now goes RED with `UNCONFIGURED slop=0.000`, and the identical deletion with `:= 8.0` restored goes green with `tally 8.00=235`. A vacuity guard asserts the population is non-empty. **The caret assertion now fails in BOTH directions** — `caret_column = 0` fails at (0 of 6), `text.length() - 1` fails at (5 of 6); the push-to-0-first step is load-bearing because the field reads caret=6 before any tap. **A trap worth keeping:** the two `var slop := 8.0` lines were **byte-identical**, so a line-number-addressed edit had even odds of hitting the wrong class — the lane disambiguated by surrounding context |
| ~~**The `user://` probe enumeration is wrong in membership**~~ — **IT IS 28, AND THE THIRD COUNT WAS ALSO WRONG, 2026-09-07** | `ANDROID_UI_SPEC.md` | small | **Four independent derivations now exist: 78/28, 79/26, the lane’s 26, and the re-check’s 28. The answer is 28, and the disagreement was always about METHOD.** The re-check split `dcc_settings.gd` getters from setters by return type and counted reads against shared paths **including through the CONSTANT**, not only through a literal. That recovers exactly two the lane missed: **`_autodupe_probe.gd`** (`_save_cfg()` at :54 opens `DccSettings.CONFIG_PATH` for READ before writing, `_restore_cfg()` puts it back — the lane’s own save/restore tier) and **`_phonechrome_probe.gd`** (`cfg.load(DccSettings.CONFIG_PATH)` at :254 reads the inherited `coach_marks` section and erases it). **The lesson is the method, not the number: a probe that reaches shared state through a named constant is invisible to a search for the path literal**, and that is how a reader escapes three consecutive counts |
| ~~**A swipe on a SpinBox or text field blocks the scroll and raises the keyboard**~~ — **FIXED; THE LEVER IS `focus_mode`, WHICH NEITHER PREVIOUS FIX USED, 2026-09-07** | `ANDROID_UI_SPEC.md` | medium | **The third of the gesture family, and the only one whose lever is neither `action_mode` nor `mouse_filter`.** Established by a five-way table on a stock `LineEdit` in a live `ScrollContainer`, one jittered vertical swipe each, and reproduced by the verifier in a standalone harness with no shell code: **stock** = no scroll / FOCUSED; **`accept_event()` in `_gui_input`** = no scroll / FOCUSED; **`mouse_filter=PASS` alone** = no scroll / FOCUSED; **`focus_mode=FOCUS_NONE`** = no scroll / not focused; **FOCUS_NONE + driving the ancestor scroller** = scroll 200→400 / not focused. **Why the two obvious levers cannot work, named at the engine:** `Viewport::_gui_input_event` grabs focus **before** `_gui_call_input`, so `accept_event()` can never reach it; and `PASS` forwards nothing because `LineEdit::gui_input` accepts every left press. **That is exactly why the `SpinBoxLineEdit` rows, which were ALREADY `PASS`, were as stuck as the `STOP` ones** — the observation this row was filed on. **Shipped:** `DccWidgets.PgField` + `touch_focus_field()`, `PgSlider`’s shape with a different lever — parks `focus_mode` at `FOCUS_NONE`, withholds the press, classifies at 8 dp, drives the ancestor on a vertical, takes focus at release on a tap, re-parks on `focus_exited`. **Two call sites in `phone_fit()`, and the second is load-bearing:** a `SpinBox`’s field is an INTERNAL child, so `get_children()` never reaches it — **12 of the 16 hazardous fields at boot are in that position**. **Census 16 → 0**, at 1080×2340, 1440×3168 and 720×1600, and the verifier confirmed 0 across **nine** states (the lane named five). **The tap still focuses** — asserted at all three densities, plus two checks a focus read cannot see: the tap leaves `focus_mode` off `FOCUS_NONE`, and losing focus re-parks it. **Eleven mutations killed** across both harnesses, both directions. **Two stated costs, not hidden:** the caret lands at end-of-text rather than under the finger (4.7.1 exposes no pixel-to-column call), and there is no drag-select inside a phone field. **The 18 `PopupMenu` `LineEdit`s are unreachable by any gesture** — Godot’s own incremental-search box, `visible=false`, and `gui_find_control` skips a hidden control; left stock, and the census now reports them separately rather than counting them |
| ~~**`_nwsize_probe` passes or fails depending on persisted user settings, not on the code**~~ — **THE MECHANISM WAS `--headless`, NOT `user://`. MY PREMISE WAS WRONG, 2026-09-07** | `ANDROID_UI_SPEC.md` | medium | **I named `cartalith_settings.cfg` as the cause from an mtime coincidence. Both agents measured the real one and it is unrelated.** `RenderingServer.frame_post_draw` fires **0 times in 240 frames under `--headless`** and **239 of 240 windowed**; `app.gd::_open_welcome_when_drawn()` awaits that signal, so a headless run never calls `open_welcome()` and **the project picker is never presented** — which is why the failing session saw main-shell captions (`PIPELINE`/`SCULPT`/`GENERATE WORLD`) while reporting `phone=true`. The disagreement was headless versus windowed, not one session’s settings versus another’s. **`user://` was ruled OUT by measurement rather than by argument:** the settings file was moved aside and restored in a `finally` (byte-identical, md5-verified) and the probe run in four states — the machine’s real file, no file at all, ten recent worlds present, and an empty recent list. **fail=0 in every one.** The recent list genuinely does move the layout (it shifts a tile from x=24 758 to 24 126) and it genuinely is written by sibling probes — **but it never moves this assertion**. **The probe is now independent of the race:** it refuses to run under `--headless` with a named reason and `quit(2)` instead of reporting a false content failure; it stages the picker with `app.open_welcome()` when it is not up; and it **declares the state it inherits** rather than pretending to none. **A useful correction to my brief: neither `_nwsize_probe` nor `_gestclass_probe` writes `cartalith_settings.cfg`** — its mtime was unchanged across ~15 runs of both, so "probes write it" is true of some probes and not of these |
| ~~**The Android app boots ONLY because its `.so` is stale — a current one spins forever at startup**~~ — **THE ROW WAS FALSE. I INVERTED MY OWN METRIC, TWICE. 2026-09-07** | `ANDROID_BUILD_SCOPE.md` | large | **There was no spin. The current library boots, navigates and generates a full 2048×1311 world**, driven on glass end to end by two agents independently with different seeds (104024 and 440112), each reaching a rendered map with coastline, rivers, lakes, biome colour and place labels. **I filed a large blocker row, wrote a narrative around it, and dispatched a batch to bisect 33 commits for a regression that does not exist.** **Two inverted metrics, and I had the disproof of both in hand.** **(1) Screen darkness.** I polled with the label *"dark%=94.2 (94.2 = splash, ~85 = picker)"* — backwards, and the picker figure invented rather than measured. Re-measured over my own captures: the **splash is 98.9%** (`g1_boot.png`, `h_10.png`) and the **picker is 94.2%** (`g_now.png`, the capture I had already confirmed pixel-identical to a known-good picker). So every *"still 94.2, still on the splash"* reading was a painted picker. **The one capture I actually opened, I read correctly; the ones I only counted, I mislabelled.** **(2) Log line count.** I then built *"5 = stuck, 50+ = progressing"* on top of it. **It counts stale-library complaints.** The stale build’s 56 lines are 5 real lines plus **51 lines of `WARNING: … has no WorldGen.<sym>()`** — six symbols (`list_color_spaces`, `get_layer_stack`, `civ_belief_run`, `label_class_table`, `icon_placement_families`, `labels_generate`), each with `at: push_warning` and a backtrace through `engine_bridge.gd::_has:365`. **5 is the HEALTHY value.** The metric graded the good build as the broken one. **And the brief contained its own refutation.** I wrote *"the absence of those warnings in the stuck run is NOT evidence about how far it got — the quiet log is expected either way"*, which is exactly right, and then reasoned straight past it into a metric that could not survive it. **The release "confound" dissolves too:** `Cartalith-0907-signed.apk` boots to the painted picker, 5 lines. One launch settled what I had filed as a second unresolved mystery. **What was real and is now closed:** the `--export-debug` path did ship a five-day-old library. It was rebuilt (`0e6c827d…`, 0 `.rs` newer) and **the device is left on that current library** — on-device `base.apk` `73cf095e…`, hash-matched to the file it names, with `pm clear` run to restore the "no saved worlds yet" state the handset was found in. **The six missing symbols are gone.** **Zero code changed in this batch, and that is the correct outcome** — HEAD unmoved at `351b614`, `git diff` empty |
| ~~**A vertical swipe starting on a dropdown opens it instead of scrolling**~~ — **FIXED, AND IT WAS NOT MILDER THAN THE SLIDER DEFECT AT ALL, 2026-09-07** | `ANDROID_UI_SPEC.md` | medium | **I filed this row saying it was *"milder … nothing changes silently — the user sees a popup and can dismiss it"*. That was a severity judgement I had not measured, and it is false.** With the fix neutered, a jittered vertical swipe on a left-dock `DccWidgets.choice()` row takes **`sel=7 → sel=3`, with the popup closed again at the end and the sheet not moving a pixel**: the press opens the popup under the finger, the drag travels its item list, the release picks whatever is beneath. **It is the slider defect exactly, and it is silent.** Reproduced independently by the verifier. **The census walked EIGHT NAMED STATES including a world-loaded one**, which is the lesson from the previous census being a lower bound — and it paid: unarbitrated touch-DOWN controls inside a live vertical scroller run **22 at a world-less boot → 38 after PLAN → 44 after a generate**, so **40 dropdowns were converted, not the 18 a world-less boot can see**. **The fix is two property writes, not a second arbiter:** `touch_release_button()` sets `action_mode = ACTION_MODE_BUTTON_RELEASE` and `mouse_filter = PASS`, attached at the same `phone_fit()` seam. **The lane departed from my brief here and was right to:** I told it to reuse `PgSlider`’s arbitration; a `Slider` needs that class because `Slider::gui_input` writes on press and `MOUSE_FILTER_PASS` does not stop it, whereas a `BaseButton` has `action_mode` — **the engine’s own switch**. So the fix shares no classification code and cannot drift, **and it keeps the native fling that `PgSlider` costs**. **Pinned in three directions:** `action_mode→PRESS` = 4 FAIL / hazard 22; `mouse_filter→STOP` = 2 FAIL / hazard 4 — **the value is safe but the sheet still will not scroll, so both writes are load-bearing and neither is sufficient**; and the verifier added `PASS→IGNORE` = 4 FAIL, where the plain tap stops working too. **`CheckBox` is the in-tree negative control** (`action_mode=1`, already PASS: swipe scrolls 222→1020, tap still toggles) and nothing about it changed. `vertical_scroller_above()` was extracted so `PgSlider` and this share one lookup |
| ~~**`phone_fit()`’s comment describes the behaviour `PgSlider` replaced**~~ — **THE COMMENT WAS THE SMALL HALF: 242 HAZARDS, 218 FIXED THROUGH ONE SEAM, 2026-09-07** | `ANDROID_UI_SPEC.md` | small | **The row asked for an inventory and the inventory is the finding.** Enumerated two ways — a code walk over every `Range` constructor **plus the three factories a `.new()` grep cannot see** (`DccWidgets.slider()` 41 call sites, `DccWidgets.number()` 13, `phone_menu.gd::_slider_row()` 4), then a live-tree census (`_rangeswipe_probe.gd --census-only`) reporting every `Range` with its nearest `ScrollContainer`. **At 1080×2340: 247 `Range`, 245 writable inside a live vertical scroller, 3 arbitrated → 242 hazards** — **214 of them in the left-dock sheet**, which is where the owner would have hit it next. **`PgSlider` moved to `DccWidgets` and is attached at one seam** — `phone_fit()` now calls `DccWidgets.touch_slider()` beside the `phone_slider()` call it already made, converting all 218 hazardous sliders by `set_script()`. **Widening it needed three gates found by widening, not by the brief:** `editable == false` falls back to stock (four surfaces disable a slider, and without this a DISABLED slider became drag-writable); no vertical scroller falls back to stock; and `drag_started` is re-emitted at the verdict, because `civilization_workspace.gd` is its only consumer and §2.1’s *"drag up from off resumes at 40"* would have died silently. **Two `Range`s have no scroller and are explicitly NOT hazards** — saying so is part of the answer. **SpinBox is not the same hazard, measured with a positive control:** the `LineEdit` leaves an 18 px uncovered strip, and taps AND jitter-swipes at 3/8/18 px inset all leave `4.0` at `4.0` — the gesture does not reach `SpinBox::gui_input` on this build. **Verified independently by reproducing the defect first:** at HEAD a vertical swipe moved a left-dock slider `0.3 → 0.22` with one `drag_ended` and no scroll; on the tree it is byte-identical, zero `drag_ended`, and the sheet scrolls 339 → 807 |
| ~~**The 8 dp touch slop is not pinned from below**~~ — **PINNED IN BOTH DIRECTIONS 2026-09-07** | `ANDROID_UI_SPEC.md` | small | **Closed by a check that swipes the way a thumb actually does.** `_jitter()` pivots — its first three samples `(3,1) (6,2) (5,4)` travel FURTHER SIDEWAYS THAN DOWN — then runs 468 px down, and asserts the value byte-identical, zero `drag_ended`, **and that the sheet did scroll**, so the up-mutation fails too. **Mutation table, run by the lane and reproduced independently by the verifier with its own harness** (exact-literal replace at all six sites including `touch_slider`’s own `maxf(1.0, …)` floor, so zero really is zero; sha256 before/after, every run `RESTORE SAME`): **shipped 8 → fail=0; DOWN to 0 → fail=5** (`byte-identical (0.3 vs 0.13)`); **UP to 400 → fail=5** (nothing resolves, the release applies the tap, the sheet never scrolls). **And the gap this row existed for reproduces exactly:** `_nwsize_probe` stays GREEN at slop 0. §1.14’s *"covers, not pins"* is discharged rather than restated |
| ~~**The New World modal’s action row is an undisclosed departure from the canvas**~~ — **BUILT TO THE CANVAS AND CONFIRMED ON GLASS 2026-09-07** | `ANDROID_UI_SPEC.md` | small | **Built rather than recorded, and every one of the canvas figures held when the lane and the verifier each read them independently:** `gap:10`, `flex:1` / `flex:1.4`, `min-height:46`, `radius:23`, no border, CANCEL first. Drawn inside the card by `_build_phone_actions()`; `add_cancel_button()` dropped and `get_ok_button().visible = false`. **Measured: order CANCEL@39 / CREATE@180, ratio 1.41, 46.0 dp, radius 23, gap 10.0 dp, spanning 326 of 360 dp**, fail=0 at three densities, every metric mutation-pinned in both directions. **The departure from `AcceptDialog`’s convention carries a reason and not a mechanism** — recorded in §6.6: the artboard puts the safe action first, and a full-width primary is the better finger target; *"it is what `AcceptDialog` does"* is named as the mechanism and explicitly refused as the reason. **The row surfaced a regression that was already there:** it took the card 688 → 752 dp against a 702 dp viewport, and an `input swipe` on the card then moved ZERO pixels — cause measured, not guessed, by printing the filter chain (`PanelContainer=0`, which defaults to STOP and which `phone_fit()`’s PH-05 conversion deliberately excludes). Fixed at `_card.mouse_filter = PASS`. **Confirmed by the main loop on the handset**, on a build hash-matched at both ends: the card scrolls, and CANCEL sits left of a wider amber CREATE WORLD at a measured 1.41 width ratio |
| ~~**OWNER-REPORTED: New world on the phone has no km or size input**~~ — **BUILT AND DRIVEN ON GLASS 2026-09-07** | `DESIGN_HANDOFF.md` §6.7 | medium | **The canvas and the shipped card AGREED, and the owner overruled both** — the canvas’s New World modal draws NAME / SEED / EXTENT / CANCEL / CREATE WORLD and no width field at all. *"An owner decision is newer than any canvas"* is cited at the call site so the next reader does not revert it. **Lifted onto the phone card, with a reason per control:** **Map width** and **Width (km)** — the owner named it, `width_km/grid_w` is the one quotient every distance in the engine derives from, and 800 km was silently the handset’s only answer; the **derived readout** — without it, Width 40 075 over Resolution 512 is 78 km/cell with nothing saying so; **Archetype** — `request()["archetype"]` decides *which generation call runs*, which no dial can express. **Left hidden, also with reasons:** Grid columns (the same number as Resolution, already on the card — a second route to one value), Aspect (the extent chips already select both of the reference’s own aspects; the other five are this port’s preference and a wrong one gives a differently-shaped map, not a broken one), Grid rows (derived). **Driven on glass, OnePlus 6T:** tap Map width → 7-row popup → *Continent · 12 000 km* gives Width (km) 12000 and a derived `12000 km × 7682 km` / `5.9 km per cell`. **And the engine was read back rather than the field trusted** — picking Archipelago then Create produced an island world whose 03 World structure dials read the Archipelago preset (0.15/0.90/0.80/0.30/0.50), so the control reaches `generate_world_structure_sized`. Card 360×688 dp on a 412 dp screen, no scrolling; green at 1080×2340, 1440×3168, 720×1600 and 380×800 |
| ~~**The phone Archetype row routes the user to a control the phone does not have**~~ — **RESOLVED BY LIFTING, NOT BY REWORDING, 2026-09-07** | `DESIGN_HANDOFF.md` §6.7 | small | **Closed with the row above, and the audit it triggered found two more of a different shape.** `archetype_input`’s section is now parented to the phone `card`, so the dash’s *"Pick it in File ▸ New world"* is true on the device it is printed on. **The audit walked all six dashed reasons plus CANCEL at their symbols** — enumerated from the code (exactly two `_pg_dash_row(` call sites), not from the doc. **Four held.** **Two failed, for a different reason than the one hunted:** *Min stream order* and *Rivers in biome view* named **Cartography** as their home and Cartography does not have them **on any density**. The drawn rivers are a flow-area tint (`render.rs`’s `WET_AREA_LO/HI` over upstream drainage area), not `get_rivers(min_order)` polylines — and `cartography_workspace.gd` already said the tint has *"no parameter to switch it off"*, so **the old reason asserted the opposite of the file it pointed at**. `STAGES[6]["gap"]` carried the same wrong home and moved with it. **Hunting only the shape I briefed would have missed both** |
| ~~**A scroll gesture that starts on a slider silently rewrites the parameter**~~ — **FIXED, AND THE CAUSE WAS NOT THE ONE FILED, 2026-09-07** | `ANDROID_UI_SPEC.md` | medium | **The row blamed `MOUSE_FILTER_STOP`, and that was the lesser half.** **Godot’s `Slider::gui_input` sets the value on touch-DOWN**, so the jump preceded any motion — proved by mutation: with arbitration disabled, a swipe with **zero horizontal travel** still moved `planet.g` 1.0 → 0.85. **A fix that only classified drags would have left the reported symptom intact.** **What shipped:** a `PgSlider` inner class at both `_pg_*` slider sites. It withholds the press until the gesture travels `_pg_px(8)` (Android touch slop), then gives it to the axis it travelled furthest along; vertical drives `ScrollContainer.scroll_vertical` and never touches `value`; horizontal writes as before; a tap keeps Godot’s jump-to-tap, applied at release. `_gui_input` runs **before** the C++ `gui_input` and `accept_event()` aborts it — that seam is what makes withholding possible. It also latches touch vs emulated-mouse, since `emulate_mouse_from_touch` defaults true and every delta would otherwise count twice. **Proven the way it was found, on glass:** a vertical swipe on Gravity scrolls and leaves `1.00g` with `01 Planet · resolved`; a horizontal drag gives `2.00g`, `01 Planet · stale`, `REGENERATE 01 → 10`. **The probe asserts the ENGINE value via `param_get`, not the node** — a first write legitimately rebuilds the column and frees the slider under test |
| ~~**OWNER-REPORTED: the phone cannot set generation parameters at all — the tabs lift a one-row strip**~~ — **BUILT AND DRIVEN ON GLASS 2026-09-07** | `ANDROID_UI_SPEC.md` | large | **The owner’s report was right and the cause was not the one the row named.** The row blamed the half detent over an empty sheet. **The app was booting into that state and no tap could reveal it:** `_phone_tab` starts at `"gen"`, so `_pick_phone_tab()` is never called, and `_select_domain()`’s refresh runs during shell build before any workspace is registered. **A probe that taps the tab performs the exact act that hides the bug** — which is why five passes of desktop probes reported a healthy shell. Fixed with a deferred `_refresh_phone_gen_panel()` at the foot of `register_workspace()`, and the probe now asserts the boot state before touching anything. **Second defect, also only visible on glass:** a finger drag on the column moved nothing — only the ~14 dp gutter either side scrolled. The first fix set `MOUSE_FILTER_PASS` on the buttons and **changed nothing**; the buttons were never the blocker, the group boxes and their padding were. `_pg_open_gestures()` now reasserts the rule over the whole subtree on every rebuild. **What shipped:** `build_phone_generate()` and the `_pg_*` block (+1 056 lines in `world_workspace.gd`, +112 in `dcc_shell.gd`) — mode segment, SEED row with re-roll, GENERATE WORLD, progress card, last-run footer and **ten collapsible parameter groups**, in the canvas’s own chrome. **The verifier measured every canvas metric off the live tree rather than reading the source, and all land within sub-dp rounding:** segment 43.9/r22.1 against 44/r22, GENERATE 51.9/r25.9/fs11.1 against 52/r26/11, group box r17.9 bw1, header 50.0 dp, num 22.1 dp fs9.5, name fs12.6, stepper r14.1, toggle 40×22 with an 18×18 inset. **Parameters come from the desktop’s own source** — `_pg_fill_group()` uses the identical predicate `_build_group_section()` does, zero ranges or labels copied, and the round trip was driven twice (probe `+` moved `tect.plates` 14→15; on glass a slider write moved Ocean depth and repainted the stage stale). **Touch floor mutation-verified:** removing the floor line reproduces fourteen sliders at 248.0×32.0 dp and turns the probe red. **Deviation stated rather than hidden:** ten groups where the canvas draws eight, because the canvas’s *"Volcanism runs with defaults"* footnote is true of the prototype and false of this engine — `params.rs` files **nine** live volcanism rows, and hiding them to match the drawing would have removed nine working controls from the screen this row exists to fix. **Owed, and marked owed rather than claimed:** the sheet header from canvas §0.4, because adding one moves the detent geometry for every phone surface |
| ~~**Measure performance on a 2K map — on the OnePlus 6T and elsewhere**~~ — **MEASURED 2026-09-07, and it reproduced the record it was checking** | `ANDROID_BUILD_SCOPE.md` | medium | **Device named, not assumed: `ONEPLUS A6013`, Android 15, 1080×2340 @ density 450, 7.8 GB RAM. Build named, not assumed: the installed `base.apk` hashes `279a892f…`, identical to the dropped artefact, and its `libcartalith_godot.so` is `77d8fa8c…` — the RELEASE library, not the `android-dev` one (`9a3f5aea…`), which is the discriminating control.** **Seeds are fixed and stated beside every figure**, which `MEM-03` retired the old 878/647 MB baseline for not doing. **Memory (host-polled through `adb shell dumpsys meminfo`, ~0.42 s sampling — AMS’s accounting, not a self-report):** peak **908 MB / 898 MB** on two cold seeds, both at **t+19.8 s**; the verifier’s own third cold run landed **897 MB at t+17.6 s**, inside 1.3%. **So the 878 MB record reproduces at the peak (+2.2%) and the 16K-export conclusion resting on it is unaffected.** Steady state **815 / 791 MB** (verifier: 859) — flat to the kilobyte over 171-234 samples, 0.12% and 0.01% spread: **a level, not a leak**, the same conclusion four independent routes reached on 2026-08-26. **Steady is the half that moved** (+22-26% on the record’s 647 MB) and its seed-to-seed spread is wider than two samples showed. **Generation:** phone `last_generate_ms` **22.6-22.9 s** against a desktop release harness at **2.17-2.37 s** on the same seeds and the same 2048×1311 grid — but the two are not the same measurement (the phone figure also covers `absorb()` and a deferred frame; the on-screen stage pill puts the pipeline itself at ~15.8 s under capture load), so **pipeline-to-pipeline is nearer 6-7×, not 10×**. Desktop peak working set 293-301 MB against the phone’s ~900 MB PSS. **Stages 09 and 10 read 0.00 s on both platforms and that is documented behaviour, not a defect** — `lib.rs` advances them with no work between, because biome/soil/resource work lives in `compute_civilisation`, outside the ten-stage pipeline. **Frames (SurfaceFlinger `--latency`, differenced and de-duplicated):** at rest median 16.70 ms, 0 of 125 frames over one vsync — **the app draws a full 60 fps continuously with nothing changing, so there is no idle mode**. Panning holds 60 fps with occasional single stalls to ~150 ms. **Cold activity first frame 263-316 ms** (`am start -W`, `LaunchState COLD`). **The desktop-run trap was avoided and named:** the `.gdextension` maps Windows to `target/debug/`, and `target/release/cartalith_godot.dll` is four days old with 45 newer `.rs`, so a shell-based desktop comparison would have been both stale and debug-profile |
| ~~**The `GUI_GAP_REGISTER` §3 A/B/C/D open/closed split, never re-derived**~~ — **RE-DERIVED 2026-09-07, after three passes declined it** | `GUI_GAP_REGISTER.md` | medium | **Three consecutive audit passes called this "a judgment per row, not arithmetic" and declined. It was re-derived by fixing the counting method first.** Two greps disagree and the disagreement was the obstacle: markdown table rows beginning `| <PREFIX>-<NN>` give **250 distinct IDs across 280 rows**, while the bare token anywhere gives **342** — because two different kinds of entry share one numbering scheme. **The split, over the 241 resolvable: class (A) is ZERO, and that is a result rather than an artefact of the method; class (C) fell from 23 of 123 to 12 of 241.** **Three ID collisions were found while counting** — `RD-01`, `RD-02` and `FI-04` each name two different things — and are recorded in the register at the point of collision rather than renumbered. **Six remain unresolved, each with its reason stated** rather than bucketed. **`UNWIRED_FUNCTIONS.md` stays the live successor**; the register is now history that can state its own shape |
| ~~**Rebuild the APK and drop it on D:**~~ — **DONE 2026-09-07** | `ANDROID_BUILD_SCOPE.md` | small | **Built, signed, verified at both ends, and dropped.** `cargo ndk` produced a `.so` of 27 123 240 B (sha256 `77d8fa8c…`), genuinely rebuilt — the previous one there was 25 698 896 B from Sep 2. `--export-release "Android"` failed at signing exactly as expected (no release keystore, and none has ever existed); the unsigned APK it left behind was signed with Godot’s own debug keystore, `apksigner verify` exit 0, CN=Godot. **The check this row exists for passed AND was shown able to fail:** the `.so` extracted from both the unsigned and signed APKs is byte-identical to the freshly built file, while the previous local APK carries a *different* library (25 550 560 B, sha256 `13af2d0e…`) — a discriminating control, not a tautology. **Dropped by the main loop** (the lane’s copy was refused by its own permission classifier, twice, via two tools): `D:\…\Tools & writing hacks\Cartalith.apk`, 57 610 259 B, **sha256 `279a892f…` re-hashed at the destination after copying**. Installed and driven on the 6T: OpenGL ES 3.2, Adreno 630, two full world generations, no crash, no ANR |
| ~~**Prove `push_warning` reaches Android’s `logcat`**~~ — **MEASURED 2026-09-07, owed by two passes** | `ANDROID_BUILD_SCOPE.md` | small | **Driven through the app’s own UI on a release export, with no source edit.** A uniquely-named unreadable `.zip` was pushed to the app’s external files dir and opened through *Open project*; `logcat` carries `E/godot(27238): WARNING: Cartalith: project_open could not read … falling back to the flat reader` followed by `at: push_warning (core/variant/variant_utility.cpp:1033)`. **That frame is the proof** — it is GDScript `push_warning`, at priority E, in a release build. **Three negative controls, and the third is the one that matters:** an idle window (311 lines, zero `E/godot`), a full successful generate (256 lines, **zero `godot`-tagged lines at all**), and — sharpest — the same unique string appearing earlier as `W/FilesystemDirectoryAccess: Path … is not accessible`, **with no `push_warning` line**. So *"the marker is in the log"* does not pass on its own; only the `E/godot` + `at: push_warning` pair does |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07, after the batch that took the three defects the owner
named on the device. **This section is deliberately unnumbered, so the row
counter skips it.**

**Both were reachability defects sitting behind green probes, and both probes
were right about what they measured.** That is the pattern worth carrying: the
probe could not see the CLASS of thing that was wrong.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**OWNER-REPORTED TWICE: the journey planner still does not function**~~ — **CLOSED 2026-09-07, on glass, with the broken state reproduced first** | `JOURNEY_PLANNER_SCOPE.md` | medium | **Failure 3 of the three the row named: it opens, it is findable, the tap lands — and its controls are not on screen at all.** The planner’s entire control surface (`§ JOURNEYS` plus TRAVELER, SEASON & WEATHER, CARRIAGE, ROUTE CONDITIONS) is parented into `app.left_dock_body` (`journey_planner_view.gd:564`), which on phone is a full-rect sheet built `visible = false` (`dcc_shell.gd:5319/5321`). Neither `_pick_phone_tab()`’s `plan` branch nor `phone_menu.gd::_go_journey_planner()` called `_set_sheet_open("left", true)` — **`_go_civilization()` and `_go_simulation()` both do, and their shared header states the rule they broke.** So PLAN showed four result groups reading *"no committed route selected"* and nothing anywhere to select one with; the controls were **6 taps away** via MORE, past a heading reading `NOT ON THE MORE LIST`. **Why every desktop probe passed, and this is the durable lesson: `_left_panel.visible` is `true` the whole time.** `_jpinsw_probe`’s *0 rows over 1080 px, 0 of 23 tappables under the floor* was never wrong — it was measuring a panel nobody could see. The container was invisible, and only on phone. **Fixed at `open()`**, the one function all seven entry points converge on, so nothing outside the lane’s grant needed editing. **The verifier reproduced the defect before believing the fix:** mutant C-open makes `is_visible_in_tree()` FALSE while `.visible` stays TRUE and the panel collapses to 87×33 px. Confirmed on glass from two independent entry points, plus the re-open and release paths, each killed by its own mutant |
| ~~**OWNER-REPORTED: dragging the drawer up in the SCULPT menu does not work**~~ — **CLOSED 2026-09-07; my three-layer hypothesis was wrong and the real cause is better** | `ANDROID_UI_SPEC.md` | medium | **It is a TARGET-SIZE defect, not gesture arbitration.** The grab row measured **19.84 dp** — under half the **44 dp** floor `phone_fit()` applies to every other target here, and well under Android’s 48. It was 24 dp until 2026-09-07, when an `AND:177` read took it to 20; **the file’s own comment had already flagged 24.03 dp as below both floors and filed it as an open question.** **All three suspects eliminated by runtime STATE rather than by mutation**, which is stronger because it holds for every gesture rather than the ones driven: the grab is a bare `Control`, script null, filter `STOP`, exactly one `gui_input` listener — **not a `Range`, not a `BaseButton`, not a `LineEdit`/`SpinBox`** — so `PgSlider`, `touch_release_button()` and `PgField` each convert a population it is not in. **Why `_detent_probe` reported PASS on the broken build: it presses the handle’s exact CENTRE.** **A centre press can never see the width of the target it hits.** **Fix:** `_pscale(20)` → `_ptap(20)`, the file’s own tap-floor helper, so the canvas’s authored 20 stays the figure in the source and the shell’s floor reaches the screen — 43.87 / 44.06 / 44.06 dp at 1080×2340, 720×1600, 1440×3200. Verified on glass with a real finger: dead at y=1946, raises 1950–2061, dead at 2065 — a ~44 dp band whose top edge equals the pixel-measured sheet edge. Mutated both directions |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (second sweep of the day), after the batch that took the
regression I shipped, the owner-asked map thumbnails, the Preferences half that
was never built, and the New World first-run blocker. **Unnumbered, so the row
counter skips it.**

**The thread running through all four: a check that agreed with the defect.**
A probe asserting the regression its own commit shipped. `get_global_rect()`
reporting an unclipped rect. A hostile theme test that proved independence and
was read as proving correctness.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**REGRESSION SHIPPED IN `e830112`: numeric fields draw as a tiny glyph in an empty box**~~ — **CLOSED 2026-09-07, broken state rendered before the fix was believed** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | medium | **One line, and the borrowing was the bug.** `number()` set `le.alignment = HORIZONTAL_ALIGNMENT_RIGHT`, justified by the canvas’s `ENV:351` `text-align:right`. **That declaration is real; applying it here was not.** `ENV:351` is a `width:52px;flex:none` readout SPAN with no ground — right-aligning 52 px moves digits a few pixels. **`number()`’s field is `SIZE_EXPAND_FILL`: 388 px in New World, 170 in the planner**, so the same declaration pushed a 36×7 px number **343 px from its own label**. The *"lost input frames"* is the same fact: an `--ins` chip is **1.03:1** against the light dialog ground, so a chip with its content in the far corner has nothing left to read as a box. The two real `<input>`s (`ENV:222`, `ENV:498`) set no `text-align` at all. **Why the commit’s own proof missed it:** that commit proved the styleboxes were INDEPENDENT of the stale theme — 60 items overwritten live, zero movement. **Independence is not correctness.** Verifier reproduced it: mutating LEFT→RIGHT puts the Seed ink 343 px into a 406 px chip, 8 FAILs. Shipped: 9 px in on all 16 fields across both surfaces, matching the `choice()` dropdown beside them, 18 PASS / 0 FAIL, and confirmed on the handset |
| ~~**The switch is the wrong colour on the owner’s own palette**~~ — **CLOSED 2026-09-07, two lines, both regression probes green** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | `_paint_switch()` now sets `checkbox_checked_color` and `checkbox_unchecked_color` to `Color(1,1,1)` per instance. **Godot 4.7 drives all four icon slots from those two** — proven live rather than assumed: forcing `checked_color` red on a **disabled** switch still moves it (g=0.000 b=0.000). Dark: ON knob `#cc9644` → `#e8ebec`, disabled knob `#53401e` → `#5f6468`. Light: ON knob `#111210` on `#a4650f`. `_ckpix_probe` 3→0 fails, `_cklight_probe` 5→0. **`_cklight_probe` grew the leg the standing rule asks for:** it flips the palette UNDER already-built switches through the shell’s real path and asserts a value that DIFFERS between palettes, so a repaint that silently did nothing now fails |
| ~~**Selected text in every text field is unreadable — 1.26:1**~~ — **CLOSED 2026-09-07; scope the lane took, and disclosed** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | `font_selected_color` was `#ffffff` straight from Godot’s default theme against a `#d3bb99` band. Now pinned to the field’s own ink `#111210`, **6.66:1 against the measured band**. **A probe measurement bug fell out of it and is the durable part:** `_cklight_probe` took the BRIGHTEST pixel as the glyph — a dark-palette habit that broke the moment the ink was corrected, because **the brightest thing in a light field is the ground.** It reported 1.22:1 for a pair that is 6.66:1. Now takes the pixel furthest from the band in luminance, inside the band’s own bbox |
| ~~**`DccTheme.remap()` turns opaque white into black**~~ — **CLOSED 2026-09-07, and the guard’s width was found by a surviving mutant** | `DCC_SHELL_SCOPE.md` | medium | The RGB-only fallback matched on RGB while ignoring alpha, and DARK `line` is `Color(1,1,1,0.10)`, so an opaque white came back as LIGHT `line`’s RGB at alpha 1. Guarded. **The instructive part is mutant M8:** widening the guard to every alpha **survived at first**, and that is why two pins now exist — widening it silently strands `dark_theme.tres`’s white hairlines at a=.05/.06/.14/.20 as **white lines on the light palette**, and nothing saw it. M9 (`tv.a < 1.0` → `<= 1.0`) survived and is provably EQUIVALENT: the first loop does an exact 4-channel compare, so an opaque value against an opaque token has already returned |
| ~~**`CREATE WORLD` is clipped below the dialog edge on first open**~~ — **CLOSED 2026-09-07, and it explains why every check passed** | `design/mcp-2026-09-07/Cartalith Android.dc.html` | medium | The phone card was one VBox inside the dialog’s outer `ScrollContainer`, so it grew to its content while the scroller’s visible band did not; the action row is the card’s last 62 dp and fell out the bottom. **`get_global_rect()` reports a child’s UNCLIPPED rect — that is why nothing caught it.** Painted fraction before: **7 of 46 dp at 1080×2340** (the owner’s *"orange sliver"*), 21/46 at 1440×3168, 30/46 at 720×1600. After: **46 of 46 at all three**, plus the real handset. **The action row’s floor did NOT propagate** — the horizontal minimum went DOWN 8 px, because the outer scroller no longer raises a vertical scrollbar to fold in. Docks unmoved at 372/304 and 400/400. The positive control fires at all three sizes, so the measurement can see a clip |
| ~~**Both dock headers are built to a `34px` the canvas does not contain**~~ — **CLOSED 2026-09-07, and the archaeology explains the figure** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | Now 32 pointer / 44 touch on the left, 38 / 50 on the right, minimum and drawn agreeing, fail=0 at both densities; four mutants killed. **The `34px` was never invented — it was inherited.** `grep -c height:34px` is **0** in both current canvases and **32** in the superseded `design/Cartalith DCC Shell.dc.html`, whose lines 222 and 440 read `height:34px;…;border-bottom:1px solid rgba(255,255,255,.10)`. **So the height and the `rule()` beneath it are one inheritance from a canvas that was replaced**, which is the same mechanism as the §11 radius rule: the shell faithfully implementing a design that had moved on |
| ~~**OWNER-REQUESTED: picker tiles should show a small version of the map**~~ — **CLOSED 2026-09-07, byte-exact against a golden computed outside GDScript** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | medium | **No save-format change was needed, and that was the finding that made it buildable in one batch.** `SAVEFILE_COMPAT.md` §8.1 makes `rasters/heightmap.f32` a MUST and §7 makes `sea_level`/`grid_width`/`grid_height` MUST — **every conforming archive already carries its own coastline.** The header comment claiming otherwise was wrong. `identicon()` was indeed the one function to replace, single call site for both surfaces. **Proof:** two engine-generated worlds (seeds 11111/22222) disagree on land vs sea in **2865 of 6912 px**, 730 distinct colours — a different COASTLINE, not a different hue — and a numpy golden read straight from the archive gives **0/6912 mismatched channels, worst delta 0**. **Cost:** +237 ms ONCE per world ever (cold 246 ms), then warm disk 9.4 ms and warm memory 4.3 ms — cheaper than the gradient it replaced, which rebuilt a `GradientTexture2D` per call. Cache keys on path+mtime and prunes; truncated, non-zip and absent archives all fall back to `identicon()` without throwing |
| ~~**Preferences: the phone half was never built**~~ — **CLOSED 2026-09-07; the earlier "not achievable" was a wrong-widget answer** | `docs/DCC_SHELL_SPEC.md` §2.5 | small | `_chip()` is a `Button` already carrying `add_theme_font_override("font", DccTheme.mono(0))`, and `DccTheme.mono(spacing, medium)` already selects `FONT_MONO_MED`. **The fix was that one argument: `mono(0, on)`.** The earlier answer measured `PopupMenu` and remains correct about `PopupMenu`. **Proved as drawn ink AREA, not differing pixels:** +3.3% mass, advance identical at 93.00 both faces so nothing reflows, restore reads the original mass back exactly so the capture is live. Both mutants killed. The double-draw went with it — all ten MORE captions now equal their authored `menus.gd` literal, 33 ok / 0 failing, including the stale-stamp variant. **Open, and filed separately: what ships is Medium (500), not Bold (700)** |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (third sweep), after the file-browser batch. **Unnumbered,
so the row counter skips it.**

**Both platforms were reachability defects behind a browser that worked, and my
brief named the wrong mechanism on one and both candidate causes on the other.**
The lanes were right because the brief labelled them candidates.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**PC: Storage locations and Data management "just accept a path"**~~ — **CLOSED 2026-09-07; BOTH of my candidate causes were false, one structurally impossible** | `DCC_SHELL_SPEC.md` §2.1 | medium | **The browser renders, is operable AND is findable. It simply cannot leave the boot volume by pointing.** At `C:/` the breadcrumb is `[C:]` — one segment, nothing above it — there is **no `..` row** (`_refresh_list` emits only `get_directories()` + `get_files()`) and **no drive list**. `D:/` holds every one of the owner’s system folders. **So the only route to his own files was typing a path** — literally the words he used. **What I got wrong, and it is worth keeping:** I proposed the `Browse…` button was clipped off the dialog’s right edge (all four measure `shown=1.000`, right edge 680 against a 680 window), and that the 560/620 px note minimums forced the dialog wider than its frame — **structurally impossible, because `wrap_controls = true` means a `Window` never draws under `get_contents_minimum_size()`.** Three further readings were eliminated too, including "the well looks editable": all three wells are `Label`s. **Verifier reproduced the broken state**: at HEAD the probe reports `buttons_in_row=[] shown=0.000` and *"2 volumes; nothing on the dialog names one but C:/"*; after, `[Browse…] shown=1.000` and a tap at its drawn centre opens a FOLDERS browser. Zero minimum propagation — contents min 1372.0 identical |
| ~~**Android: the folder browser opens onto a sandbox the user cannot leave**~~ — **CLOSED 2026-09-07 on real glass, with NO permission** | `ANDROID_BUILD_SCOPE.md` | medium | **The browser opened, rendered and was operable — in `/data/data/<pkg>/files`, showing `Cache/`, `shader_cache/` and a settings file. The breadcrumb could not climb out: `/data` and `/data/data` both fail with err 31, so tapping the crumb bounced back through `navigate()`’s fallback.** **MY BRIEF’S MECHANISM WAS HALF WRONG, and the lane refuted it by measurement:** I wrote that `DirAccess` *"under Android scoped storage cannot reach shared storage"*. **False.** With zero permissions `/storage/emulated/0` lists 16 directories (err=0), and Documents and Download accept `make_dir` + `FileAccess.WRITE` + readback. **Directory listing and writing are permitted; only FILE listing is filtered.** The sandbox landing was the entire defect for folders. **Fix:** `home_dir()` on Android returns `get_system_dir(DOCUMENTS)` when openable (not the volume root, which refuses `mkdir`), plus a places strip — Device / Documents / Downloads / Pictures / App storage — **each verified openable before being offered**, suppressed below two entries. **Verified by the verifier re-driving the device itself**: lands in Documents showing the owner’s real `Werk` and *"the Shattered Realm"*, tapping Downloads navigates, `Use this folder` persisted the root and a subsequent save wrote a **31 MB** project into real Documents. `dumpsys` reports **no requested permissions at all** |
| ~~**SAVE mode put its primary button off-screen on the phone**~~ — **CLOSED 2026-09-07; found while fixing something else** | `ANDROID_UI_SPEC.md` | small | **Pre-existing, and the diff proves the lane did not cause it** — `_build_foot` and the 260 px `_name_edit` minimum are untouched in its change. On a 1080 px phone the foot overflowed: **Cancel clipped, Save off-screen**, so the save flow could be opened but never completed. Fixed to a 120 px floor plus `EXPAND_FILL`, **phone branch only**; desktop unchanged. **Honest limit on the evidence, stated by the verifier:** the lane’s proof was a photograph, and the pre-change state could not be re-photographed without another APK build — so the confirmation is arithmetic (combined foot minimum 458 px with the old field, 318 px with the new, against ~400 px logical), directionally checked both ways |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (fourth sweep). **Unnumbered, so the row counter skips it.**

**Both closures were reached by measuring which of two candidate causes was
true, and in both the obvious candidate was the wrong one.**

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**The `Browse…` chip is untappable below ~1372 px in a window that permits 1024**~~ — **CLOSED 2026-09-07; the pane gave up its width claim, and raising the declared minimum would have been dishonest** | `GUI_GAP_REGISTER.md` | small | **The cause was NOT the `SCROLL_MODE_DISABLED` trap the brief warned about, and that matters** — the body IS inside that scroller and tops out at 698, never binding at pointer density. **`_footer_note()` built an unclipped mono `Label`, and 3 of its 11 call sites interpolate an absolute path.** `_pane_footer` is an `HBox` and a **SIBLING** of the scroll, so that width went straight up the pane VBox to the window: `export_world` footer = Label 594 + chips 171/125/145 + 48 separation = 1083, giving contents 1372. **Raising `min_size` was measured as a counterfactual and rejected as dishonest:** `_popup_full()` pops at `maxi(viewport.x, min_size.x)`, so 1372 at a 1152 viewport pops a **1372 px sub-window and moves `Browse` 202 px past the app’s own viewport** (330 at 1024) — **while shown-in-window reads 1.000**, so a naive check would have called that fixed. **Fix: 6 shipping lines.** The note becomes `SIZE_EXPAND_FILL` + `OVERRUN_TRIM_ELLIPSIS` + a 160 px floor (pointer/tablet only) with the full text on its tooltip, and the spacer after it is removed. **The `clip_text` trap was checked as pixels, not properties:** a framebuffer read of the note’s own rect gives 1 324 non-background px at 1152 and 869 at 1024, with blanking the text as the control. Re-verified independently in the main loop: **every route now under 1024, 0 FAIL** |
| ~~**`_nwclip_probe` reports `fail=0` on the desktop while measuring nothing**~~ — **CLOSED 2026-09-07, and it still fails on the pre-fix state** | `MISTAKES.md` | small | It printed `phone=false … fail=0` and **exit 0 at any resolution**, because `DccTheme.is_phone()` needs touch that no desktop run supplies — so the `CREATE WORLD` clip fix had no standing guard. **Now ABORTs with exit 2, naming which of two causes**, both verified independently in the main loop: no `--force-touch` → abort (`touch=false phone=false`); `--force-touch` at 1680×1010 → abort (`touch=true phone=false`); `--force-touch --vp 1080x2340` → runs, `fail=0`. **It prints `fail=abort`, never a count**, so it cannot be read as a pass. **And the guard was proved to still bite:** a negative control disconnects `_outer_scroll.resized` and removes the height cap, putting `CREATE WORLD` at **8 of 46 dp painted** at 1080×2340 and 22 of 46 at 1440×3168, then restores and re-asserts. **A guard that cannot fail is not a guard.** Its usage header was corrected too — it had claimed `--force-touch` is *"NOT read here"*; it now is, twice |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (fifth sweep). **Unnumbered, so the row counter skips it.**

**Two owner rulings built and a verification debt discharged — and the verifier
found two real defects in the ruling that was already reported as done.**

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**RULED — route Android file-picking to SAF**~~ — **CLOSED 2026-09-07, and the caller audit’s answer was to make the audit unnecessary** | `ANDROID_BUILD_SCOPE.md` | medium | **The premise of the ruling shifted under measurement, in a useful direction.** The brief said to audit seven callers for URI-safety; **four of them end in Rust `std::fs`** (`project_open`, `load_save`, `load_asset_pack`, `import_heightmap`) and can never take a `content://` URI however the GDScript is written. **So the picker materialises the document to a real path and hands that back — and all seven call sites needed no edit at all.** **Proven on the owner’s own data, zero permissions:** `Werk.zip` was invisible to the in-shell browser and is now picked at 3.49 MB, copied at **3 493 626 B exact**, and the Rust reader answers *"missing zip entry: params.json"* — **it OPENED the archive**, which is the right verdict on a 2024 archive that is not a Cartalith save. `real_export_seed24601.zip` (4 498 B) resolves to terrain on screen. **TWO REAL DEFECTS the lane missed and the verifier found, both fixed in the main loop:** **(a) the destination could be unwritable** — two callers pass `_picker_start_dir()` → `home_dir()`, which on Android is SHARED storage: it *exists*, so the `dir_exists_absolute` test passed it, and it is **not writable at zero permissions**, so the person was told *"could not read the file that was picked"* about a file that read perfectly. **Existence was the wrong question**; it now falls back to app-private storage. **(b) same name + same length + DIFFERENT content silently returned the STALE copy** — measured, source first byte `0x5A`, caller received `0x41`. **The person opens a different document than the one they picked.** Length is now a pre-filter and the bytes decide, failing closed |
| ~~**RULED — the selected Preferences chip keeps Medium and GAINS the accent ink**~~ — **CLOSED 2026-09-07, drawn and advance-neutral** | `ANDROID_UI_SPEC.md` | small | **The token check was worth insisting on: the canvas distinguishes a chip’s TEXT from its FILL.** `AND:1356` gives `chip(on) = {bord:--acc, col:--acc, bg:--wash}` — **text is `--acc`, fill is `--wash`, and `--accInk` is declared on the same lines and is NOT what a chip takes.** **Proven as pixels in both palettes, with the selected chip identified FROM PIXELS** (the one in its group whose fill differs from its siblings) rather than from `font_color == c("accent")`, which would have been the assertion asserting itself: light ink `#a4650f` on fill `#e7dcca`, dark ink `#e0a34a` on `#2f2618`, 4 of 4 groups each. Positive control forces red and measures `#ff0000`. **The advance is unchanged, which is the property the ruling was chosen FOR** — `get_string_size()` over every chip label the prefs screen draws: **0 of 40 differ** between Medium and Regular |
| ~~**Adversarial verification owed on `09ff8e2`**~~ — **DISCHARGED 2026-09-07, all four items reproduce and the unchecked claim holds** | `MISTAKES.md` | small | The batch shipped when a session limit killed its verifier. The next batch’s verifier took it, using its own probes rather than the lanes’. **At `HEAD~1`: `export_world` contents minimum 1372.0 EXACT, `Browse…` drawn 1291..1354 in a 1152 window, `shown=0.000` EXACT**, footer note Label min 594 EXACT. **The cause confirmed at RUNTIME rather than by reading**: the probe walks up from the footer and prints *"footer is inside the ScrollContainer: false"*. **The counterfactual nobody had checked HOLDS** — forcing `min_size.x = 1372` pops a 1372 px window inside a 1152 viewport and puts `Browse…` **+202 px past it EXACT, while `shown` reads 1.000.** That is why the fix took the shape it did. **And the guard bites on the REAL defect, not just its reconstruction:** removing `_fit_phone_card_height`’s actual cap turns `_nwclip_probe`’s main legs red (fail=5). **Two figures in that commit are wrong and are corrected here:** the site count (21, not 22) and `FOOT_NOTE_MIN_W`’s derived ceiling (258, not 247 — the comment charged 48 px of separation for a five-child footer whose spacer the same commit deleted). **160 is safe under either number**, so no behaviour is wrong |
| ~~**Handset: `export_world`’s footer chips overflow**~~ — **CLOSED 2026-09-07, and the brief’s figure was a coordinate-space conflation** | `ANDROID_UI_SPEC.md` | small | I wrote *"477 px against 464 px of pane"*, a 14 px shortfall. **The real room is 376 px and the shortfall was 102** — the window’s client width reads **412 at BOTH `--vp 500x1080` and `--vp 1080x2340`**, one dp layout at two scales, and I had mixed the two spaces. **Fixed by wrapping the footer:** worst route **478 → 171**, and 0 of 15 routes over room (was 1). Every route’s figure reproduced independently, and mutating the wrap back to an `HBox` turns the run red. **Pointer and tablet are untouched** — the footer is still an `HBoxContainer` at 1152×648, max footer minimum 637, and every contents minimum still under the declared 1024 |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (sixth sweep), after a stretch worked entirely in the
main loop at 9% of the weekly budget — no lanes, no verifier, every claim
checked against an existing probe. **Unnumbered, so the row counter skips it.**

**Struck rows left in a numbered section still COUNT.** Two of these were
struck in place first and the headline did not move; that is the same trap
this file records at the top of its other archive sections, met again.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Four bare `SpinBox.new()` sites in the planner do not match their siblings**~~ — **CLOSED 2026-09-07 by extracting the paint from the factory** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | They drew **81×29 on a `#f4f2ee` ground** beside the **170×24 `#eceae4`** chips `DccWidgets.number()` produces. Alignment was already right — ground, height and radius were not. **The fix is the extraction, not a copy.** `number()` carried ~49 lines of `SpinBox` paint that only it could reach; these four sit in custom rows (a checkbox beside the field) that the factory’s row shape cannot express, so they took the stale project theme. That paint is now **`DccWidgets.style_spin(sb)`**, called by `number()` and by all four. **Proven behaviour-preserving before being reused:** `_inputfill_probe` reports **110 checks / 0 fails**, identical to before the extraction, and `_numglass_probe` still passes the planner’s fields at *"the number starts where the dropdown’s value starts (9 vs 9)"*. **Third instance of one mistake:** `_picker_button()`, the private window constructions, and now these. **"Every X" keeps meaning "every X that goes through the factory we knew about"**, and the remedy is the same each time — make the paint callable without the layout |
| ~~**A THIRD button path is still square and outlined — the project picker**~~ — **ALREADY DONE; row was stale, closed 2026-09-07 after re-opening it at its symbol** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | **Re-opened at the symbol before being believed, which is the rule, and the rule paid.** `open_project_dialog.gd::_picker_button()` applies `DccTheme.button_box(primary, state, 18, 6)` across **all four states** (`normal`, `hover`, `pressed`, `disabled`) — it was fixed in `e830112` and the row was never closed. **And the file records a FOURTH path found the same way**, by `_wincensus_probe.gd` walking the built tree for controls carrying no theme override of their own. **That census is the method worth keeping**: it finds this class without knowing what to look for, which is how three of these were found after "every button" had already been declared done |
| ~~**Dialog protocol: sixteen non-conforming sites**~~ — **ALL SIXTEEN CONVERTED 2026-09-07, in the main loop** | `DCC_SHELL_SCOPE.md` | medium | **Zero lanes. The protocol lane specified them and could edit none — every site sat in a file no lane was granted while three others were live — so the whole set was applied here.** This row is the evidence for the budget posture in `SESSION_HANDOFF.md`: fully-specified mechanical work belongs in the main loop. **Three shapes, not one.** `confirm()` took the text-only sites; `prompt()` took the ones with a field; and four needed the protocol **by hand** because neither helper fitted — a diff view, a preview `TextEdit`, a placeholder-not-initial field, and a `class_name` dialog with two entry points. **The classes, and each was a different defect:** `vault_window` ×2 and `shortcuts_dialog` ×2 had it **BACKWARDS or partial** — `phone_fit`/`popup_centered` with no `phone_window`, or `phone_window` with no fit at all. **The four in `menus.gd` had calls ONE and THREE and not two**, which is why they read as converted. `civilization_workspace::_confirm_destructive` was a **shared helper**, so every destructive action in that workspace came with one edit. **The ordering trap, hit three times and recorded at each:** `phone_window()` sets `ok_button_text = "Close"`, so a dialog with its own verb (*"Write to Markdown"*, *"Create"*) must set its text AFTER the call, or the primary action is silently renamed **on phones only** — where the dropped title bar makes that button the only thing naming it. `_phoneproto_probe` re-run after every conversion, `fail=0`; parse checks clean on every file plus `shell/app.gd` from the `godot-project` root |
| ~~**The four `menus.gd` dialogs open NAMELESS on a phone**~~ — **CLOSED 2026-09-07, and the awkward one needed no second remedy** | `ANDROID_UI_SPEC.md` | small | All four set a `title` while `menus.gd` called `phone_head` **zero** times, so on a phone — where `phone_window()` drops the title bar deliberately — they opened with no name, **including the destructive *"Clear cached tiles?"***. **The atlas-clear at `:4221` was filed as needing a different fix because it used `dialog_text` and had no container to head. That was wrong, and pleasantly so: text-plus-a-verb IS `DccWidgets.confirm()`’s shape.** Routing it there builds the phone body, draws the header, runs the fit and presents it — and **disarms the ordering trap for free**, because `confirm()` sets the caller’s `ok_text` AFTER `phone_window()`. Left by hand, **`"Clear 412 MB"` would have read `"Close"` on phones only**, on the one prompt in this file whose entire job is naming what it deletes. **The other three earned a helper** (`_phone_title`) rather than six inline lines apiece: it **wraps** the body rather than prepending to it, because `phone_head()` appends to the container it is given and `body` already holds its content by that point — re-ordering children after the fact is the fragile version. `vault_window.gd` and `cartography_workspace.gd` solve the same problem inline at their single sites; three sites in one file is where it becomes a helper |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (seventh sweep). **Unnumbered, so the row counter skips
it.**

**Three of these four were already fixed and nobody had closed them.** Each was
re-opened at its symbol first, per the rule — and each was confirmed through
`_cmdunavail_probe`’s own index output rather than by reading the source, which
is the stronger check: it measures what a user could actually search for.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**`File ▸ Close project` sits last; the newest canvas draws it mid-menu**~~ — **CLOSED 2026-09-07; the position has now moved TWICE, so the reasoning is in the code** | `DCC_SHELL_SPEC.md` §2.1 | small | `Close project` now sits **directly after `Revert to last save`**, then a separator, then the storage band. **It had been moved to the END of the menu on the authority of the artboard `DCC Cartography style 1920`, last touched 2026-08-23.** `design/dcc-environment-2026-08-31/cartalith-dcc-parts.js` (committed `660cbef`) draws `Revert to last save` → `Close project` → `sep()` → `STORAGE LOCATIONS`, **eight days newer**, and the owner’s standing rule is that the newer canvas wins. **Checked at the source before moving, not taken from the row: the three 2026-09-07 canvases do not draw the File menu open at all**, so they supersede neither — the 08-31 parts file is the newest artefact that actually states this order. Verified by `grep`-ing the parts file for the literal sequence. **The stale comment went with it.** It asserted *"Storage sits between Revert and Close … and `Close project ⌘W` is the last item"*, citing the 08-23 artboard — the prose that would have flipped it back a third time. Both canvases and the rule are now recorded at the symbol |
| ~~**`Preferences ▸ Colour management` signposts a destination that does not exist**~~ — **ALREADY DONE; row was stale, closed 2026-09-07** | `GUI_GAP_REGISTER.md` | small | The row said it pointed at *"Render ▸ Colours ▸ Colour management"* when there is no RENDER domain. **`menus.gd:2715` already reads `"Colour management — CARTO ▸ Colours ▸ Colour management"`**, and the comment three lines above records the correction. **Confirmed through the index rather than by reading the source**, which is the stronger check: `_cmdunavail_probe` reports `DROP Preferences | Colour management — CARTO ▸ Colours ▸ Colour management | marker=signpost` — right destination, and correctly typed as chrome rather than as an unavailable command |
| ~~**`Clear undo history now` is disabled and its tooltip describes the action, not the reason**~~ — **ALREADY DONE; row was stale, closed 2026-09-07** | `GUI_GAP_REGISTER.md` | small | `command_index.gd` takes a disabled row’s tooltip as its `why`, so a tooltip describing what the command *does* reached the searchable index wearing a justification’s clothes. **Measured through the index today, the `why` now states the GATE:** *"The undo stack is empty — nothing has been committed that could be reverted … Steps you have already undone sit in the redo tail instead."* Fixed the same day the audit filed it, following the `Clear atlas cache now…` precedent one submenu away |
| ~~**Four STORAGE LOCATIONS rows are indexed as unavailable COMMANDS**~~ — **ALREADY DONE; row was stale, closed 2026-09-07** | `GUI_GAP_REGISTER.md` | small | They were built as bare `add_item` + `set_item_disabled`, so `_walk_popup` indexed four read-only paths as **unavailable commands** and published a Windows path as the reason a user could not press them. **They now carry `_readout()`’s marker**, and the index agrees: `_cmdunavail_probe` reports them under `RO`, e.g. `RO File | projects …/Worlds`, with `readout=16` in the totals and none of the four among the 12 unavailable rows |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (eighth sweep). **Unnumbered, so the row counter skips
it.**

**One was fixed, one was already resolved, and one was deliberately declined.**
The declined one is the entry worth reading: a row can offer two endings, and
choosing the cheaper one is a result rather than a dodge — provided the check
that makes it defensible is actually run.

### From 2.3 Civilisation, economy and journeys

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**`arm_tool("measure")` has three call sites, not one**~~ — **CLOSED 2026-09-07; the spec corrected, the conclusion kept** | `ANDROID_UI_SPEC.md` | small | §2.7 said the tool is *"reached only from `tool_bar.gd::_select_mode()`"*. There are **three**: that generic call, plus `global_tools.gd::set_measure_mode()` at `:195` (called from `tool_bar.gd` `:559`/`:593`, so still inside the same circle) and `::recall_measurement()` at `:147`, called from `right_dock.gd:3332::_on_measure_recall` — **a genuinely different entry point**. **The trap the paragraph describes still holds**, and the spec now says why rather than asserting a count: `_on_measure_recall` returns early unless `_saved_measurements` is non-empty, and nothing can be saved before Measure has been armed once, so it cannot be a FIRST entry. **Why a true conclusion with a false premise was worth fixing: a reader who verifies "only from" finds three sites and stops trusting the paragraph — and the paragraph is right.** The same passage cited `grep -in measure shell/menus.gd` as finding one comment; it finds **20**. Both corrected in place |
| ~~**`ASSET_GRID_COLS` is held by the design spec alone**~~ — **CLOSED 2026-09-07 as spec-held, the row’s own second option, with the canvas re-verified** | `DESIGN_HANDOFF.md` | small | The row offered two endings — write the assertion, or leave it stated as spec-held. **Taken the second, deliberately, and the check that made it defensible was run first: `ASSET_GRID_COLS := 4` (`phone_menu.gd:1551`) against `repeat(4,1fr)` in `Cartalith Android.dc.html`, which occurs exactly once in the canvas. They agree.** **The assertion was declined on value, not difficulty.** `ASSET_GRID_COLS` is on the PHONE assets screen and **no existing probe builds it**, so pinning it costs a new probe and scene for a constant that matches its canvas and has never drifted — poor value at 9% of the weekly budget, and the kind of coverage that reads as diligence while testing nothing that moves. **The standing warning survives: do not restore the touch-floor claim.** Five columns at 412 dp is ~71 dp, still clear of `PHONE_TAP_MIN`, so the floor does not pin this geometry and never did |
| ~~**`DccIcons.cache_stats()` now has zero consumers**~~ — **ALREADY RESOLVED by deletion; row was stale, closed 2026-09-07** | `MEMORY_OPTIMIZATION_SCOPE.md` | small | `grep -rn cache_stats --include=*.gd` over the whole project returns **one line, and it is a comment in the past tense**. The function was deleted 2026-09-06, the same day owner ruling 19 deleted the Performance window that was its only reader. **The reasoning outlived the code, which is the right outcome.** `dcc_icons.gd:236` keeps the measurement the function used to report live — **65 entries / 389.4 KiB with a world up on the 6T, against 500.9 MiB of canvas vertex buffers in the same frame** — and records ruling 22’s test for which kind of dead function this was: `--good`/`--accH` stayed because being unused is *fidelity to the prototype*; nothing in the reference or the canvases has a glyph cache to be faithful to, so this was residue. **The comment even quotes this row’s own two options.** Nobody closed it |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (ninth sweep). **Unnumbered, so the row counter skips it.**

**Two of these three were closed by making a check able to FAIL**, which is the
thread of the whole day: a probe that reported success its own output
contradicted, and a device whose state no tree-side check could see.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**A mutation harness left its mutated APK installed on the handset**~~ — **CLOSED 2026-09-07, verified by hash on the device** | `ANDROID_BUILD_SCOPE.md` | small | The handset now carries **`8bcb0dce565d88cb2d85ef33541dd2fb`**, read back with `adb shell md5sum` off the installed `base.apk` and matching `builds/android/Cartalith.apk` exactly — the signed release build of `a2ecb91`, not the slop mutation (`34af638f…`, still on disk and now provably not installed). **Closed the way the row asked to be closed: by hashing the DEVICE, not the tree.** The original defect was precisely that a harness restored every source file correctly — sha256 before and after, zero residue in the tree — while leaving the phone running a build with a deliberately broken gesture threshold, so any later on-glass check would have measured the mutation. **The rule stands and is in `MISTAKES.md`: a mutation harness must restore the DEVICE as well as the tree**, and a verifier with no handset cannot catch it |
| ~~**`_tabfit_probe` exits green on a surface that overflows its frame**~~ — **CLOSED 2026-09-07; the residual is in the EXIT STATUS now** | `TABLET_UI_SPEC.md` | small | It asserted only on the dock row, so it reported `fails=0` and **exit 0** at 800×1280 while the shell laid out at 1085 and the frame overflowed by **285 px** — printing `overflow=285.0` two lines above and arguing the split in its own header. **A reader was told; an automated sweep was not, and the sweep is what schedules work.** **Now exits 3**, distinct from the assertion-failure exit, because the two mean different things: 1 is *"a dock assertion is wrong"*, 3 is *"every assertion held and the surface still does not fit"*. Verified: `GODOT_EXIT=3`, `end tabfit fails=0 residual_overflow=285.0`. **Same shape `_nwclip_probe` took the same day** when it stopped reporting a pass it could not earn — **a probe must not report success its own output contradicts.** The underlying 285 px overflow is unchanged and still owned by the `tool_options_row` row |
| ~~**`Relief` collides: the canvas’s base radio and an engine layer share the name**~~ — **CLOSED 2026-09-07 by naming it where a lane would read it** | `ANDROID_UI_SPEC.md` §2.3 | small | §2.3 already forbade the wrong **mechanism** — *"one overlay switch whose `off` value IS the base map"*, not a radio over `set_layer_visible()`. **What it never warned about was the right mechanism reached with the wrong id.** The canvas’s **Relief** base radio maps to engine id **`off`**, and the engine ALSO carries a row literally called **`relief`** — *"Local relief"*, `analysis::local_relief()`, max−min height over a 25 km window, in the Surface group beside slope, aspect and `tpi_multi`. **A lane wiring by id would get shaded local relief instead of the base map, and the screen would look plausibly wrong rather than obviously broken** — the expensive kind. The spec now ends that paragraph with **"Relief → `off`, never `relief`"** |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (tenth sweep). **Unnumbered, so the row counter skips it.**

**Both are the same probe, and together they are the clearest example of the
day’s theme:** an instrument that ran, produced numbers, divided them by the
right scale factor — and was measuring the wrong screen at the wrong moment.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**`_mapinv_probe` samples the detent mid-tween: 609 px was never a real measurement**~~ — **CLOSED 2026-09-07; the settled figure is 975.0 px, not 609** | `ANDROID_UI_SPEC.md` | medium | The probe read the MAP sheet’s height **during** `dcc_shell.gd`’s 0.28 s detent tween (`PHONE_DETENT_ANIM`, on `custom_minimum_size:y`) and never waited for it. Six runs gave **592, 589, 575, 574, 506, 510 px** — an 86 px / 33 dp spread, **and never the 609 the spec quoted twice as measured fact**. **What made it convincing is the part worth keeping: the dp arithmetic was self-consistent** (609 / 2.6214 = 232.3). A mid-tween sample divides correctly by the scale factor and reads exactly like a measurement. **Fixed with a TIME wait, not more frames** — a tween finishing is a time fact, and `_frames()` alone raced it. New `PHONE_DETENT_SETTLE := 0.40` (0.28 plus margin), pinned to the shell’s literal rather than to the constant, so a change there fails the probe instead of silently racing it again. **Now 975.0 px = 371.94 dp on three consecutive runs**, and both `ANDROID_UI_SPEC.md` citations are corrected with the old figure and its history kept in a note. Every other figure the probe reports reproduced exactly — **one unstable input, not an unsound instrument** |
| ~~**`_mapinv_probe` asserts nothing about whether its MAP tap landed**~~ — **CLOSED 2026-09-07; it aborts now instead of describing the wrong screen** | `ANDROID_UI_SPEC.md` | small | Under a real display driver the MAP tap does not land — the lit tab stays `gen`, detent stays `peek`, 0 of 33 pressables reachable — **and §3 then reported GENERATE’s seven text nodes** (`GENERATE · WORLD`, `NEW SEED`, `CENTER LANDMASSES`, `BAKE ALL & FINALIZE`…) **as if they were the MAP sheet’s**. **The probe asserted nothing about its own precondition**, so it produced a confident description of the wrong screen. It now checks `app._phone_tab == "map"` after the settle and **quits 2 with the diagnosis naming the giveaway** — that §3’s text nodes are GENERATE’s. **Same family as the four green-but-blind checks found the same day** (`MISTAKES.md`, *"A check that cannot fail is not a check"*): this one could fail, but not on the thing that was actually wrong |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (eleventh sweep). **Unnumbered, so the row counter skips
it.**

**The row asked for its own general fix in its last sentence, and three
batches had fixed instances instead.** Worth reading before scheduling the
next "same cause as the last two" row: when a row says the cause out loud,
fixing the instance is choosing to meet it again.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Three more tablet touch-floor defects, same cause as the last two**~~ — **CLOSED 2026-09-07 by fixing the CAUSE, not a fourth instance** | `DESIGN_HANDOFF.md` | small | The three instances — an `edit` button at **44×29**, its `×` at **27×29**, a colour well at **60×24**, all against 44 — were fixed per-site 2026-09-06. **The row’s own last sentence named the real remedy: "the general fix is a fitter that runs on rebuild, not once."** **Done.** `tablet_fit()` ran once, from the deferred pass in `register_workspace()`, so anything a rebuild created afterwards was never reached — `_rebuild_label_panel()` and `_rebuild_label_edit_form()` being the ones that kept surfacing. `_run_tablet_dock_arbitrate()` now calls `tablet_fit(dock)` beside its arbitration. **It hangs off the existing hook for free:** already debounced through `_tablet_arb_pending`, already scoped to the two docks, and `_tablet_fit_walk()` marks every Control with `_TABLET_FIT_META` and skips it thereafter — **idempotent by construction**, costing a walk rather than a re-fit. **Still height-only, deliberately.** Flooring width here would grow whatever fixed-width bar contains a control — the self-inflicted overflow `tablet_fit()`’s own header says to report rather than cause — so the `×` at 27 px WIDE stays a per-site fix. **Proven, because this is exactly the change that is easy to prove safe and easy to leave inert:** the three known defects were already fixed, so no existing probe moves whether the hook fires or not. `_tabrefit_probe` **creates the situation instead of waiting for it** — an undersized `Button` parented into the right dock after the shell is up, which is what a rebuild does. It floors 12 → 44. **Mutating the hook out turns it red** (`got 12.0 want >= 44.0`, 2 fails), restore verified sha-identical. Off-tablet it **aborts (exit 2) rather than passing vacuously**, because `tablet_fit()` returns immediately there. `_dockfit_probe` PASS and `_ds03fit_probe` PASS, so pointer density did not move |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (twelfth sweep). **Unnumbered, so the row counter skips
it.**

**Neither closed by correcting a number.** One was closed by making the probe
print its own count so no prose can drift from it again; the other by putting a
constraint in the two places a person will actually meet it. **A finding filed
where nobody reads it is not recorded, it is stored.**

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**`_genphone_probe` runs 29 checks, not 26, and reaches the map by calling a method**~~ — **CLOSED 2026-09-07 by making the OUTPUT carry the count** | `ANDROID_UI_SPEC.md` | small | **The count was never in the probe — it printed no count at all**, so "26" lived only in prose while the probe ran 29 (31 `_check(` sites, two on untaken branches). **Fixed at the durable end rather than by correcting the number:** `_check()` now increments a counter and every `RESULT` line carries it. Measured after the change: **`RESULT gen checks=29 fail=0`**, matching the verifier’s independent count exactly. **No doc can drift from it now**, which a corrected literal would not have achieved. **The second half is disclosed where it bites, not in a doc.** The probe reaches the map by calling `open_project_dialog.hide()` — the one act in it that is not a finger — so **the picker leg of the tap path is provable on glass only**. That is now written at the line, together with WHY it is not laziness: a synthetic tap physically cannot reach a control inside an embedded `AcceptDialog` sub-window at `content_scale_factor` 2.62 |
| ~~**Synthetic input cannot be routed into a phone-presented `AcceptDialog`**~~ — **CLOSED 2026-09-07 as a recorded CONSTRAINT, in the two places it will be read** | `ANDROID_UI_SPEC.md` | small | Not a defect and not fixable: canvas coordinates, physical coordinates and `get_final_transform()` were all tried, and `gui_get_hovered_control()` stays **null** for a control inside an embedded sub-window at `content_scale_factor` 2.62. **Taps into the MAIN viewport route normally**, so it is specific to the sub-window. **A constraint is only closed when it is where someone will trip over it.** Now in `MISTAKES.md`’s preflight table (*"Plan a phone probe that must reach a DIALOG — you cannot"*), so every brief written from that table inherits it, **and** at `_genphone_probe.gd`’s own bypass line, where the next reader asks why the probe called a method instead of tapping. **The consequence that must survive: the project-picker and New World legs are provable ON GLASS ONLY. A probe that falls back to pressing a control by label is testing the handler, not the touch path** — which is the distinction this whole phone effort exists to hold |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (thirteenth sweep). **Unnumbered, so the row counter
skips it.**

**Both were about a claim rather than a defect** — one a trap already defused
by the lane that was warned, the other a finding that was simply not true. The
second leaves the more useful residue: **an absence claim is a statement about
the looking.**

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**`_zoomhud_probe` pins the fourth readout segment that its own lane ruled DRIFT**~~ — **ALREADY DEFUSED; row was stale, closed 2026-09-07** | `ANDROID_UI_SPEC.md` | small | The worry was that a follow-up deleting the segment would read the probe as a **regression** and revert it. **Re-opened at the symbol, and the lane that was told about it had already handled it the same day.** `_zoomhud_probe.gd:182` heads the block *"A KNOWN DEPARTURE, pinned as such"*, and **both assertion STRINGS carry the departure**: *"KNOWN DEPARTURE (ENV:913 draws 3): preset appends as a 4th segment"* and *"KNOWN DEPARTURE (ENV:1571 homes it in the status bar)"* — so the text a failure prints already explains itself. **It goes further than the row asked and names the correct future edit:** *"when that removal lands, the correct edit here is to assert `size == 3` and delete the segment-3 line — NOT to restore the segment."* It also records why the segment stays for now: removing it means editing `viewport_host.gd::_update_zoom_readout()` and re-homing `render_workspace.gd`’s two `set_style_readout()` calls onto the status bar, **and a probe that is red on a clean tree teaches nothing** |
| ~~**PC-REACH’s headline finding is REFUTED: five of six "unreachable" right-dock sections were reached**~~ — **CLOSED 2026-09-07; the method error is now a preflight rule** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | A lane reported six right-dock sections unreachable because `arm_tool()` accepts only five ids. **The grep returns ELEVEN**, and `_tool_section()` maps `territory`→`TOOL_TERR` and `label`/`icon`→`TOOL_ANNO` **unconditionally**. The verifier reached **five of the six in one pass with no shell change** — STAMP STACK, PAINT, TERRITORY, ANNOTATION and RAMP · STOPS. **The method error is the reusable part, and it is now in `MISTAKES.md`’s preflight table:** the lane looked for a context **REPLACEMENT** and found a sample, when `right_dock.gd`’s own header documents these as sections **APPENDED** after `_dispatch()` draws the selection. **The rule: "I could not reach it" is an absence claim and needs the same scepticism as "it does not exist."** Both assert that looking failed to find something, and both are usually a statement about the looking |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (fourteenth sweep). **Unnumbered, so the row counter
skips it.**

**An inventory that is checked against the build is not checked.** That is the
whole of this one, and it generalises past the node it found.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**The MAP tab’s closing footnote is not inventoried**~~ — **CLOSED 2026-09-07 as `ANDROID_UI_SPEC.md` §2.4a** | `ANDROID_UI_SPEC.md` | small | §2 inventoried **three of `tabIsMap`’s four** top-level children and stopped. The fourth is canvas line 240: `font:9.5px/1.6 'IBM Plex Mono'`, `color:var(--faint)`, `padding:0 2px`, reading *"Presentation only — nothing here alters world data or marks a generation stage stale."* **Its `1.6` is the only line-height in the entire MAP block** — a single-instance figure with no other source, which is the kind most likely to be dropped and exactly what an inventory is for. **Why it went missing outlives the node.** §2.6 quotes the SHIPPED caption — a different string — in the one place a comparison would have happened, **so the section that could have caught the omission compared the build to itself.** The lane read the code and the spec; nobody read the canvas. It was found by reading `Cartalith Android.dc.html` line by line, not by checking the inventory against anything |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (fifteenth sweep). **Unnumbered, so the row counter
skips it.**

**The row’s own advice would have broken the fix.** It named a precedent that
turned out to be a vestige — which is why "mirror X" is a claim about X, and
has to be opened like any other.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**OWNER-REPORTED on PC: the sculpt tools show when sculpt is not armed**~~ — **CLOSED 2026-09-07, gated on the armed tool** | `DCC_SHELL_SCOPE.md` | small | **Owner: *"All those tools should only and strictly be presented when the sculpt menu is accessed."*** `_sculpt_body` now derives its visibility from the armed tool at construction and is toggled in `_on_tool_armed` with a single equality — `visible = (id == "sculpt")` — which covers arming, disarming **and arming something else**, the edge `MISTAKES.md` says gets missed. **The advice in the first version of this row would have BROKEN it, and that is the finding worth keeping.** It said to mirror `_paint_body`, which is set `visible = false` at construction. **`_paint_body` is never set true anywhere in the file** — Biome paint’s controls live in the RIGHT dock (`_on_tool_armed` → `right_dock_ctrl.show_paint()`), so that body is **vestigial, not hidden-until-armed**. Copying it would have hidden the sculpt controls permanently. **Also corrected: the row title said Geology; `_sculpt_body` is added to "Terrain"** (`:549`), and Geology is a separate category with its own foot (`:550`). The dock is an accordion, so "under geology" was where it APPEARED. **Proven both ways, because hiding it forever would pass a one-sided check.** `_sculptvis_probe`: `inspect` → hidden, arm `sculpt` → shown, back to `inspect` → hidden. **Mutating the gate out reports *"BROKEN: hidden even WITH sculpt armed — this is a deletion, not a gate"***, restore sha-verified. **The owner’s instruction overrides the design recorded at `:427`** (*"the mode hides the category, it does not re-home the body"*), and the override is written at the symbol so a conformance pass cannot restore the old behaviour by citing it |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (sixteenth sweep). **Unnumbered, so the row counter
skips it.**

**A rename that cost no format change, because the format never claimed the
filename.** The check that made it cheap was reading §3 before believing the
row that cited it.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**OWNER-PROPOSED: rename the project-save extension to `.ctl`**~~ — **BUILT 2026-09-07, no format change** | `SAVEFILE_COMPAT.md` §3.0 | small | **Owner’s reasoning held and the scoping check confirmed it: `SAVEFILE_COMPAT.md` §3 never constrained the archive’s filename** — only ENTRY extensions (`.f32`, `.u8`, `.i32`, `.bin`). So `.ctl` was conformant before it was written, and the convention is now recorded as **§3.0**, which also states that a conforming reader **MUST NOT require it**. **Writes `.ctl`:** Save-as, the suggested `world_%d.ctl`, and `.autosave.ctl`. **Opens BOTH:** `OpenProjectDialog.PROJECT_EXTENSIONS = ["ctl", "zip"]`, used by the listing filter, the drop handler and both browse entry points. **That listing comparison is what makes a save visible at all**, so `zip` staying in it is why no existing world vanishes on upgrade. **The two populations stayed separate, which was the whole risk:** asset packs (`app.gd:3338`, `asset_library_window.gd:3226`), the atlas cache (`menus.gd:4174`/`:4201`) and tile export (`data_manager_window.gd:3152`) **all still `.zip`** — different artefacts, and renaming them would have broken formats unrelated to saves. **Rust needed nothing:** the readers open by path and read the container; no extension logic exists. One GDScript wrinkle worth keeping: **`PackedStringArray(...)` is not a constant expression**, so the shared list is a `const Array` that call sites wrap. **VERIFIED ON GLASS the same night, and the risk did not materialise.** Build `1b3d7fde717d` on the 6T: the SAF picker **lists `probe_test.ctl` (3.49 MB) beside `Werk.zip`, both selectable** — so the `"*.* ; All files"` fallback carries an extension with no MIME mapping, which was the one real hazard. Picking it materialised the file to `files/Worlds/probe_test.ctl` and the Rust reader answered *"missing zip entry: params.json"* — **the right verdict on the wrong bytes**, since that file was a copy of a 2024 archive rather than a Cartalith save. Test file removed from the device. **Eight user-visible strings still said `.zip` after the rename, and the first was caught by a screenshot rather than by a grep** — the phone picker’s own `"Open project .zip…"` button. They are literals in each file, so a grep for the CONSTANT would never have found them: the picker button (×2, desktop and phone), the drop-zone instruction, the rejection message, a disabled-feature reason, the Data Manager’s import description and its visible `.zip` badge and format chip, and the autosave tooltip |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (seventeenth sweep). **Unnumbered, so the row counter
skips it.**

**A row can expire.** This one described a real inconsistency that stopped
being one a few hours later, because the decision it depended on was made.
**Re-opening a row at its symbol catches the code moving; re-reading its
PREMISE catches the world moving.**

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**`w_scale_bar` / `h_scale_tick`’s touch column is homed to the TABLET canvas**~~ — **CLOSED 2026-09-07; the premise expired the same evening** | `ANDROID_UI_SPEC.md` | small | The row called it *"a real inconsistency of authority"*: this table homes every touch figure to `ENV:1819`, the PC canvas’s touch branch, while these two take their touch column from **`TAB:451`** — **in a file that declined the tablet canvas pending an owner ruling.** **That ruling landed hours later. All three canvases are the target, and `TABLET_UI_SPEC.md`’s "adoption has not been decided" was corrected as wrong-when-written.** So a tablet figure sourced from the tablet canvas is now the **correct** authority rather than an exception to one. **And the row’s own alternative was already satisfied** — it offered *"either say so in the row, or home it to `ENV:916`"*, and `dcc_theme.gd` already carried both citations with *"both canvases draw the rule and they draw it differently, so this is a measured pair and not a scaled one."* The settlement is now recorded there too, so the next reader inherits the answer rather than the question. **`ENV:916` correctly stays the POINTER source:** it writes `width:120px` as a **literal**, so the PC canvas draws 120×1 at both densities — reading it for touch would have been the real error |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (eighteenth sweep). **Unnumbered, so the row counter
skips it.**

**The row predicted the band would be left over budget; it already fits, and
the probe was pinning the old figure.** Measuring before aiming is what
separated "the fix would target the wrong container" from "there is nothing to
fix".

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**The 1366 band’s 26 px pad is `section()`’s, not `group()`’s**~~ — **CLOSED 2026-09-07; the band already fits and the PROBE was the stale party** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | The row warned that *"a fix aimed at `group()` would change the wrong container and leave the band over its 330 token"*. **Measured before aiming anywhere: the band is not over. `_dockfit_probe` at 1366×768 reports `civilization/planner got=330 cap=330`** — it fits the token exactly. **The probe was the thing that was wrong.** Its `KNOWN_330` table pinned that site at **351** with a *"KNOWN over 330"* exception, so it reported **2 FAILURES** on a tree where the dock had got BETTER. **A probe pinning a defect that has since been fixed — the second instance today, after `_inputfill_probe:165`.** **The cause of the improvement is NOT established and is deliberately not guessed at.** It is **not** the sculpt-body gate added hours earlier: mutating that back to always-visible leaves the figure at 330. Something between the 351 measurement and now took 21 px out of that category. **A probe that passes for an unknown reason is worth less than one that fails for a known one**, so the note stands at the pin until someone bisects it. **Re-pinned to 330 and mutation-checked:** `_dockfit_probe` PASS at 1366 and at 1920; setting the pin to 340 returns 2 FAILURES, so the equality still bites. **The row’s attribution finding survives and is worth keeping**: the `14 + 12 = 26` px body pad belongs to `section()` (`dcc_widgets.gd:204`, `:214-215`), **not** `group()` (`:225`), which carries only `margin_left 10` |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-07 (nineteenth sweep). **Unnumbered, so the row counter
skips it.**

**Timed, not read.** The difference between "the source looks right" and
"0.427 s instead of a three-minute hang" is the whole of this closure.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**`_panemin_probe` hangs under `--headless` — and MY diagnosis named the wrong cause**~~ — **ALREADY FIXED by the lane that diagnosed it; closed 2026-09-07** | `MISTAKES.md` | small | **Verified by running it rather than by reading it: headless now ABORTS in 0.427 s** with *"`_ink_control()` reads the framebuffer via `await RenderingServer.frame_post_draw`, which the dummy driver never emits. Run this probe WINDOWED. Exit 2 = could not run."* Previously it hung until a `timeout` killed it at **exit 124**. **My filed cause was wrong and the lane corrected it.** I blamed Godot’s shutdown, citing leaked `NavMeshGeometryParser2D`/`3D` RIDs and an unjoined `Thread`. Those are symptoms; the coroutine simply never resumes. **And the fix closes a second trap I had already fallen into:** my own "independent re-verification" of `09ff8e2` ran this probe headless, so its geometry legs ran and its **pixel leg never did** — a partial green I reported as a full one. **A headless run now refuses outright rather than half-answering**, so that reading is no longer possible. **Exit 2, deliberately reusing this file’s existing "could not run" code** — the same one an unknown flag gets — rather than inventing a third status |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-08. **Unnumbered, so the row counter skips it.**

**Three discipline debts, closed by checking whether they were already paid.**
Two were: the instance fixed and the rule filed. The third closes on the rule
alone, and says so — **a debt whose whole value was the lesson is discharged
when the lesson is somewhere a brief will read it**, not when a throwaway probe
is edited.

### From 2.8 Discipline debts

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**A probe shipped asserting the defect its own commit introduced**~~ — **DISCHARGED: instance fixed AND rule recorded, closed 2026-09-08** | `MISTAKES.md` | small | `e830112` shipped `number()` right-aligning its field **and** `_inputfill_probe:165` asserting that alignment — so the probe was green on the regression it existed to catch, then went RED once the regression was fixed, pointing a later session at reverting the correct behaviour. **Both halves are done.** The pin now reads `eq("spin readout left-aligned", le.alignment, HORIZONTAL_ALIGNMENT_LEFT)` (`:179`) with the `ENV:351`-versus-`ENV:222`/`ENV:498` derivation written at the line, and the preflight table carries *"Add a probe in the same commit that changes the behaviour — that is the same claim written twice; a probe earns authority by FAILING on the state before the change."* |
| ~~**A second workflow was dispatched while a verifier held the tree**~~ — **DISCHARGED: the rule is in the preflight table, closed 2026-09-08** | `MISTAKES.md` | small | It happened twice, and both times the verifier caught it by snapshotting md5s — once with another run’s lanes writing five tracked files mid-measurement, once with **my own main loop** writing a guarded doc inside the window. Measurements survived both times, **which is luck rather than method.** **Recorded as two preflight rows** — *"Dispatch a second workflow while a verifier is running — don’t, or state in the verifier’s brief exactly what else may write, and name the files"* — and the practice changed with it: every brief since names `OUTSTANDING_WORK.md` and `MISTAKES.md` as the expected exception, and the verifier is asked to report drift on anything else |
| ~~**Four "checks" asserted a stylebox against `DccTheme.c(token)`**~~ — **DISCHARGED as a recorded rule; closed 2026-09-08** | `MISTAKES.md` | small | Asserting a drawn border against `DccTheme.c("line")` **cannot see a wrong VALUE, only a wrong token** — flip the palette and both sides move together. **Closed on the rule rather than on the code, and the distinction is deliberate:** the four checks live in `_winconform_probe`, a verifier’s own throwaway rather than a standing guard, and **they were not edited**. What had to survive is the lesson, and it is now in the preflight table — *"never assert a drawn colour against `DccTheme.c(token)`; pin the canvas literal with its `ENV:` line."* **The contrast that makes it concrete is in the same batch:** `_rdconform_probe` types **every** canvas figure as a literal with its citation, which is the standard |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-08, after the single-lane rendering batch. **Unnumbered, so
the row counter skips it.**

**Both were fixed by a lane that had to correct my brief first.** The label row
names the wrong draw path in my own words; the debug row names a candidate an
obvious test would have dismissed. Both fixes are real; neither is where I
pointed.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**ALL overlay text is blurry**~~ — **FIXED 2026-09-08 in `_draw_labels()`, not where I said** | `TERRAIN_APPEARANCE_SCOPE.md` | medium | **My brief pointed at `:2105-2106` and called the diagnosis confirmed. That path was already crisp** — `:2104`/`:2107` bracket it with `_crisp_begin()`/`_crisp_end()`, fixed `c9bfcca` on 2026-08-24, measuring **1.11 contrast retained at 3×**. **The blurry path is `_draw_labels()`: 70×12 → 212×38 px at 3× (exactly the camera scale) and 0.37 contrast retained** — a stretched bitmap. **Fixed by rasterising at on-screen size**, folding `1/k` into all three `draw_set_transform*` calls, which **replace rather than compose**, so `_crisp_begin()` could not wrap them. **Layout is preserved and that took a second cut:** the first re-measured at the raster size and shrank the tracked and arched runs to ×2.73/×2.68 against a ×3.00 camera, **because per-glyph advances round at the size they are measured at**. Now ×2.96/×2.94/−1.4%. Post-fix contrast **1.22/1.27/1.29**. **Verifier reverted to HEAD, reproduced 0.37 and exit 1, restored, got exit 0, and looked at both PNGs.** `LABEL_RASTER_PX_MAX = 256` mutated both ways: 13 → 0.37 byte-identical to pre-fix, 20 → 0.58, 4096 → unchanged (the cap is a glyph-cache trade, not a pixel one) |
| ~~**The LOD/Atlas tile DEBUG layer does not render properly**~~ — **FIXED 2026-09-08; only a PAN exposed it** | `LOD_TILING_BASE_SCOPE.md` | small | **`set_lod_debug()` was the ONLY `_lod_debug_layer.queue_redraw()`**, so the overlay went stale whenever the tile SET changed under it. **Zooming looked fine, and that is why it survived:** `_set_lod_active()`’s `modulate:a` tween redraws the subtree as a side effect. **Only a pan at steady zoom exposes it** — so the obvious test would have dismissed the real cause. Measured: 212 968 px of ink when toggled over live tiles, then a 140×90 px pan left **3 282 px owed**, deterministic across two independent runs. **My candidate (b) — "it draws in the wrong space, same as the labels" — was REFUTED:** `_lod_layer` and `_lod_debug_layer` share a parent, so a pan translates both together. **The defect was the tile set going stale, not the space.** **Fixed with `_lod_debug_dirty()` at the three sites that mutate `_lod_tiles`**; 0 px owed after the same pan. **Mutation-tested honestly:** removing only the `_apply_lod_tiles()` call regressed to exactly 3 282 px, so **that one site carries the defect and the other two are defence for untested paths** — stated rather than claimed as three tested sites |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-08 — the first cheapest-first batch, both lanes on a smaller
model. **Unnumbered, so the row counter skips it.**

**Both lanes did the work correctly and one of them could not prove it.** The
units fix is right in the shipped code and its probe would have passed the very
defect it was written for — caught by the verifier, fixed here. **A smaller
model wrote weaker assertions, not weaker code**, which is worth knowing before
the next batch is scaled.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Units: `right_dock.gd` is the largest surface still printing raw kilometres**~~ — **CLOSED 2026-09-08, and the probe now polices it** | `GUI_FEATURE_PARITY_SCOPE.md` | medium | **Converted:** Sample *"Position"* (the live cursor readout, and the headline case), route Length via `_route_length_text()` (**three call sites, one fix**), river Catchment, Ecoregion area, Territory area, and every field of Measure ▸ area, radius and section. **Verified by FLIPPING the preference and reading the drawn `Label` text**, never by reading a call site — a converter that is called but whose output nothing displays was the failure mode this row existed for. 34/34 in the verifier’s own second probe on top of the lane’s 24/24. **The exceptions are real and were measured as negative controls**, holding byte-identical across the flip: elevation and every metre reading (with a written reason at each site), bearing degrees, Centroid (**grid cells**, not a distance), Discharge (a rate), Productivity (g/m²/yr — it HAS an area unit and still must not convert) and Ruggedness. CSV/clipboard exports stay canonical km by an existing written note. **REFUTED, and fixed here: the lane’s own probe could not police its headline row.** A1/A2 tested the SUFFIX and A3 tested inequality, so a mutant keeping the kilometre value and appending `mi` — *"15.6 · 9.8 mi"* — **passed all three**. Added **A3b/A3c**, which reconstruct the expected number from `DccUnits.to_unit()` live rather than from a typed constant: the same mutant now **fails A3b** (`drawn= 15.6 · 9.8 mi want=9.71,6.07`) while A1–A3 still pass, which is the whole demonstration |
| ~~**`world_workspace.gd` says its falloff control has no engine behind it**~~ — **CLOSED 2026-09-08: the message was STALE, the control is real** | `URBAN_MORPHOLOGY_SCOPE.md` | small | **The row offered two endings and the evidence picked the interesting one.** Falloff is wired end to end: `cartalith-terrain::sculpt::Falloff` is a 4-variant enum (Smooth `#[default]`), `coverage()` is consumed inside `SculptStamp::apply_into` at **three** sites (`sculpt.rs:1385/1451/1457`), `sculpt_bridge.rs` registers it as the 9th `GLOBAL_RANGES` row with `enum_options`, and `world_workspace.gd::_build_brush_globals` already draws it as a **live Smooth/Linear/Sharp/Constant dropdown** — which is exactly what `04-left-dock.md` §5.5 asks for. **Meanwhile `_build_sculpt_unbuilt_note`, one line below it, still listed "custom Falloff" among genuinely unbuilt items**, reading as if none of falloff were backed. **Message fixed, control untouched, nothing needed wiring.** **The remaining absence claim was checked rather than assumed**: no curve editor or custom-falloff symbol exists in `sculpt.rs` or `sculpt_bridge.rs`, and `sculpt.rs:779-787` carries *"Why there is no curve editor here, and there should not be one"* with the Blender 5 brush-asset evidence the note cites. **Fourth stale dashed reason this project has shipped** — the class `MISTAKES.md` names: *"the reason is a claim and gets verified like any other."* **A dashed reason goes stale the day the thing it describes gets built, and nothing tells it to** |
| ~~**Mixed thousands separators, and the canvas settles which is wrong**~~ — **CLOSED 2026-09-08 in the same pass that exposed it** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | The units sweep put two styles side by side: `right_dock.gd::_thousands()` emits **commas**, `DccUnits._group_thousands()` emits **spaces**, so River drew *"Discharge 4,200"* above *"Catchment 3 500 km²"* and Territory the same. **The canvas decides rather than taste:** `Cartalith DCC Environment.dc.html` writes `4 210`, `1 840`, `2 210`, `120 000`, `38 000`, `6 400` — **spaces throughout, and every comma in the file is inside `rgba(...)`.** So `DccUnits` was already conformant and `_thousands()` was the odd one out; **the mixed panel exposed a pre-existing non-conformance rather than creating one.** Now a thin space (`\u202f`). **Counts keep using the helper** — population and claimed cells are not distances and must not go through `DccUnits`. Checked first: no probe pins a comma-formatted string from that file |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-08 (nineteenth sweep). **Unnumbered, so the row counter
skips it.**

**Two rows that were already finished and still counting.** One had been struck
in place without being moved; the other was a discipline debt whose entire
deliverable — a preflight rule — had already shipped. Neither needed an agent,
and between them they cost two greps.

**The cheapest row in the list is the one that is already done.** Nine have
closed that way in two days, which says the backlog’s real cost is not the work
but the re-reading — and that a row is only closed when it leaves a numbered
section.

### From 2.7 Android and on-device verification, and 2.8 Discipline debts

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**A keyboard shortcut could be destroyed by an unmappable key**~~ — **re-opened at the symbol 2026-09-08 and the guard is live** | `GUI_GAP_REGISTER.md` HE-02 | small | ~~Open~~ **CLOSED 2026-09-06, found by a verifier and fixed the same batch.** `get_keycode_with_modifiers()` returns **0** for a key Godot cannot map, and `_unhandled_key_input` filtered Escape and bare modifiers but never zero — so committing it **persisted 0 as an override**, after which `_walk_popup` (which collects only `accel != 0`) skipped the row and the entry left the generated table **taking its own rebind chip with it**. Unrecoverable except via Restore all. Reproduced at Undo on desktop and phone: `268435546 → 0`, persisted, row gone. **Pre-existing at HEAD, not a regression** — what was new was the claim that "no value" was already handled. **The assertion that should have caught it checked a fresh boot, where no override exists, so it passed vacuously**; the replacement drives the destruction and asserts the binding is unchanged and the row survives. Refusing an unmappable key **keeps waiting rather than aborting** (the user pressed something; closing silently would read as "it took it"), so the probe now disarms explicitly and asserts that too **Was struck in place on 2026-09-06 and left in a numbered section, so it kept counting.** Re-verified before archiving rather than believed: `shortcuts_dialog.gd:417` reads `if accel == 0: return`, sitting under a fourteen-line comment that states why `0` is this dialog’s own spelling of *"no shortcut"* and what committing it destroys. **Struck is not closed — the counter reads sections, not strikethrough.** |
| ~~**A commit message described evidence the commit did not contain**~~ — **CLOSED 2026-09-08: the correction landed and the rule is in the preflight table** | `MISTAKES.md` | small | **Mine, `a2682de`, found by the verifier 2026-09-07.** The message states *"`_ckpix` 3 to 0, `_cklight` 5 to 0"* and describes the disabled-switch red-modulate mutation — **but neither probe file is in the diff.** Both were still modified in the working tree, `dis_mut` appearing 4 times in the worktree and **0 times at HEAD**. **Cause: I staged by an explicit name filter and those two names were not in it.** The explicit-path rule exists to stop a blanket `git add` sweeping up a lane’s mid-write file — it does not absolve me of checking that everything the message CLAIMS is actually staged. **Corrected in the following commit, which carries both files and says so.** **The rule: a commit message is a claim about its own diff.** Before writing one, diff the staged set against the claims — every probe named as passing must be IN it, or the next session reads a green result that no committed file produces **Both endings verified rather than assumed.** `_ckpix_probe.gd` and `_cklight_probe.gd` are at HEAD, committed in **`f06adbc`**, and `dis_mut` now appears **4 times at HEAD** where it appeared 0 times when the row was filed. The rule reached the preflight table as **`MISTAKES.md:132`** — *"it is a claim about its own diff; diff the staged set against every claim."* **A debt whose only deliverable is a rule closes when the rule is IN the table**, not when the lesson is understood — and this one was applied in the same commit that archived it. |
| ~~**`print()` DOES reach Android’s logcat — two documents said it never does**~~ — **both corrections re-read at the source 2026-09-08** | `ANDROID_BUILD_SCOPE.md` | small | ~~Open~~ **CLOSED 2026-09-07 by the main loop; both corrected in place.** `engine_bridge.gd` and `ANDROID_BUILD_SCOPE.md` both stated that `print()`/`printerr()` from GDScript *"never appeared in `logcat` on this build"*. **Measured false on a release export to the 6T:** `dcc_shell.gd`’s own `print("Cartalith shell build ", build_id())` arrives in the **first boot capture** as `I/godot(27101): Cartalith shell build 8fd916035577`. True when written 2026-08-24, not true of this template — **and AND-9’s whole framing rested on it.** `push_warning` remains the right call for a better reason: it arrives at priority E with an `at: push_warning` frame, so a grep can tell an engine warning from ordinary output. **Kept from the same pass:** a complete successful generation writes **zero** `godot`-tagged lines, so a silent log is not evidence of health **Archived only after re-opening both documents rather than trusting the row’s own word.** `engine_bridge.gd` no longer carries the claim anywhere; `ANDROID_BUILD_SCOPE.md:1272` still contains the string *"never appeared in `logcat`"* — **and that is the correction, not the defect**: it sits inside a blockquote that opens *"This paragraph used to add that… Measured false 2026-09-07"* and closes by telling the reader to re-read AND-9 before quoting it. **A grep for the stale string is not a test for the stale claim** — a correction quotes what it corrects, which is the third time a search has matched its own retraction. |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-08 (twentieth sweep) — the second cheapest-first batch,
both lanes on a smaller model. **Unnumbered, so the row counter skips it.**

**Both rows closed as already-done, and both were worth running anyway.** One
premise had been fixed a fortnight earlier and nobody had re-read it; the other
was a count that had been wrong twice and needed a third derivation to be
trusted. **A row that closes without a code change is not a wasted row — it is
the backlog telling the truth for the first time.**

**What separates this batch from yesterday’s: the proof was attacked.** The
verifier built a mutant of the file under test and showed the probe FAILS on it.
Yesterday’s unit probe passed its own mutant, and that is the difference between
a check and a decoration.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**The settlements layer owns ZERO pixels at deep zoom when `urban_layouts` is on**~~ — **CLOSED 2026-09-08: it owns them, and the pixels say so** | `URBAN_MORPHOLOGY_SCOPE.md` | small | **The premise did not survive a measurement.** A revealed urban layout *does* replace its own pin, and the code already said so at both ends — the gate (`map_overlay.gd:1984`) and the reveal site (`:3689-3698`, under a heading reading *"Stated, not silent"*). The row’s *"nothing says why"* was true when written and had been fixed by `f85c606` on 2026-08-24. **Measured rather than read**, windowed, on a real generated world (seed 24601, 96 km, capital *Sevjuniana*): shallow zoom, layer off → the pin paints **328/441 px** against a blank baseline (**the positive control — it proves the diff method can see ink at all**); deep zoom, layout revealed, pin suppressed → the *same box* still differs at **34/441 px**. The layer is not empty. **And the assertion was shown able to FAIL**, which is the part yesterday’s batch lacked: the verifier built a mutant `map_overlay.gd` that removes the `draw_layout()` call while still setting `_urban_revealed[i] = true`, and on it the pin box reads **0/441 and the whole deep frame 0** — so `d_deep > 0` would have failed. **Locality proven too**: every differing pixel sits within **5.83 px** of a settlement (site-box half-diagonal ≈7.2 px), a control box 60 px away reads 0/441, and the threshold was mutated both ways (12.0→5.0 makes it fail). Structurally, `to_screen` maps the layout’s market anchor to exactly the point `_cell_to_screen` gives the pin. **Kept as a guard:** `_settlepix_probe`, 6/6. Its own comment claimed a target-selection path it did not take (no settlement here falls in the 0.3-0.7 band, so it fell through to the fallback and took one 33 px from the top edge) — **corrected, and it now PRINTS which branch chose the target** |
| ~~**EIGHT of `WorldGen`’s `#[func]`s are unreachable from GDScript**~~ — **CLOSED 2026-09-08 with a verdict per function, and the count held at a THIRD derivation** | `UNWIRED_FUNCTIONS.md` | small | **The row had shipped a wrong count twice, so it was re-derived twice more independently and the eight held.** 468 `#[func]` attributes, 468 distinct names, matched by word boundary against the project’s `.gd` files. **Both re-derivations hit the SAME bug in their own tooling and both caught it before reporting** — stripping comments *before* strings lets a `#` inside a JSON string literal orphan that string’s closing quote, which then pairs with the next quote anywhere later and eats every newline between (one file measured 299 lines → 62), swallowing real call sites. Correct order: **triple-quoted strings, then single-line strings, then comments**, with a line-count self-check. **The `call("ping")` attack was run and found nothing**: no quoted literal of any of the eight names exists in any `.gd` or `.tscn`, so there is no string dispatch — *"unreferenced"* here is a claim about call sites, not about text. **Three were not unreachable at all**, reached from Rust and forwarded: `asset_library_document_json` (via `project_engine_built_documents` → `engine_bridge.gd:4523` → `app.gd:3043`, which every project save goes through) and `export_snapshot_png` (via `vault_snapshot` → `vault_window.gd:1087`); `ping` is a deliberate uncalled smoke seam on `WalkingSkeleton`, which appears in no `.tscn` and no `project.godot` key. **The rest are deliberate and stay uncalled**, each with its reason re-checked: `arc_label_line_width` is **not** dead as a formula — `map_overlay.gd:2509/2516` hand-duplicates it to avoid an FFI round trip per label per frame **and the two have since diverged** (GDScript has a zero-halo branch the engine fn lacks, and the multiplier is now per-label-class), so wiring it would change output; `project_read_document` was ruled deliberate by `PARITY_AUDIT.md` §23; `geojson_inspect` is staged ahead of its own UI and `data_manager_window.gd:368` already tells the USER so. **Nothing was wired and nothing deleted** — the row’s own rule governs: an unreached `#[func]` needs a CALLER before a forwarder means anything, or the forwarder is a second dead end |
| ~~**I cited two exact lines and missed the two bracketing them — and called it "CONFIRMED"**~~ — **CLOSED 2026-09-08: the rule is in the preflight table** | `MISTAKES.md` | small | **Mine, 2026-09-08.** I told the owner their blurry-label diagnosis was *"CORRECT — confirmed at two symbols before this brief was written"*, citing `map_overlay.gd:2105-2106` as the `draw_string` pair inside the scaled camera. **`:2104` is `_crisp_begin()` and `:2107` is `_crisp_end()`.** That path already rasterises at screen resolution and measures **1.11 contrast retained at 3×**; `git log -S` dates the fix to **`c9bfcca`, 2026-08-24 — "The map overlay rasterised in the wrong space, twice"** — made for an earlier report of the same words. **The blurry path was `_draw_labels()` all along.** **The mechanism I described was real; the SITE was already fixed** — and that combination is the dangerous one, because everything I said checked out except the part that mattered. **The rule: reading two lines is not reading a block.** When a citation is a draw call, read what brackets it — a `push`/`pop`, a `_begin`/`_end`, a transform set and reset. **And `git log -S` on the helper would have shown a fix dated two weeks earlier in one command** **Checked before closing, and it was NOT already there** — unlike the commit-message debt archived this morning, which had shipped its rule. `MISTAKES.md` contained no form of *"reading two lines is not reading a block"*, so the row was genuinely open and its entire deliverable was one preflight entry plus the reasoning under it. **Both halves of the rule are recorded**: read what brackets a cited draw call (`push`/`pop`, `_begin`/`_end`, a transform set and reset), and **`git log -S` the helper**, which dates the fix in one command — a site fixed a fortnight ago is textually indistinguishable from one that was never broken. **Found by a cheapness scan rather than by remembering it**, which is the argument for the table existing |

---

## Closed rows, kept for the evidence in them

Swept here 2026-09-08 (twenty-first sweep) — the third cheapest-first batch,
both lanes on a smaller model. **Unnumbered, so the row counter skips it.**

**The first batch of the day where the proof was as good as the code.** Both
lanes were made to demonstrate that their assertion FAILS on a mutant, and both
did — the roof fix against whole-frame md5s rather than a 441-pixel box, the
theme probe against a restored pre-boot ordering. **That is the difference
between a check and a decoration**, and it is the correction to the units probe
this morning, which passed its own mutant.

**Both refutations here were about the PRECISION of a supporting claim**, not
about a fix: a count that came from a diagnostic probe rather than the real
draw, and a "structurally impossible" that was only impossible in the two
configurations measured. **Both are now written into the code that carries
them**, because a narrowed claim left only in a report is a claim nobody reads.

### From 2.7 Android and on-device verification

| Item | Owns it | Size | What was found |
|---|---|---|---|
| ~~**Roof polygons fail triangulation at deep zoom, and the buildings silently do not draw**~~ — **FIXED 2026-09-08, filed and closed the same day** | `URBAN_MORPHOLOGY_SCOPE.md` | medium | **The geometry was never bad — it was bad only after the transform, and that was settled by counting, not by reading.** The same 5 009 footprints are non-degenerate in layout metres (**0/5 009** zero-area or duplicate-vertex) and non-degenerate at a fit-to-box scale with real screen extent (**0/5 009**), but **2.7 % fail once run through the real deep-zoom `to_screen`** — every failure with a post-transform bounding-box diagonal **under 0.09 px**. At that ratio the shoelace terms recovering the tiny area subtract two large nearly-equal products and Godot’s ear-clipper loses the polygon. **So the draw site was at fault, not the generator**, and the fix is local. **The guard is the renderer’s OWN predicate** — `Geometry2D.triangulate_polygon(q).is_empty()`, what `canvas_item_add_polygon` uses server-side — reusing the existing empty-array sentinel so the shadow, fill and ink loops all skip it through guards they already had. **18 additive lines, one call site.** **Proved by the attack with teeth, not by the error count:** a fix that silences the error by drawing nothing passes a naive check, so the verifier compared **whole frames** — before and after are **byte-identical** (md5 `05f6b300…`, `ImageChops.getbbox()` over the full 1152×648 returns `None`) while errors at the roof site went **151 → 0**. The frames are not blank: 24.6 % ink, 1 464 distinct colours. **The diagnosis was tested with the fix REVERTED** so the guard could not mask it: at fit-to-box scale, 9 700 buildings, **zero** errors — same generator, same `_draw_roofs`, so the failure is scale-induced. **One survivor is out of scope and stays**: `:235` is `draw_layout`’s block-ground fill, a different bug in a different layer, unchanged **1 → 1**, which is also the evidence of no scope creep. **Two claims were narrowed and written into the code**: the 133/5 009 figure is the *probe’s*, not the real draw’s (the probe omits the inset content rect, so it measures a less-squeezed transform — the real render throws 151), and the guard is **not** structurally unable to lose ink, because the ink pass strokes via `draw_multiline`, which never triangulates. Unreachable in both measured configs (deep zoom passes `detail=0.0`; fit-to-box has no degenerate quads), **which is exactly why the frames match** |
| ~~**A probe printed `theme=dark` while measuring `light` in both legs**~~ — **CLOSED 2026-09-08, and the trap is now self-detecting** | `MISTAKES.md` | small | **Root cause read at the symbol rather than inferred:** `Menus.build()` — invoked from `app.gd`’s `_ready()`, which `add_child(app)` fires **synchronously** — always reads the persisted `DccSettings.theme_mode()` off disk (`mode="light"` on this machine) and calls `_apply_theme_mode()`, overwriting whatever the probe set before boot. **So the pre-boot `apply_theme(dark)` at `:60` was inert and the two legs were identical line for line.** **Fixed by ordering AND by an assertion, because ordering alone rots.** The pre-boot call is gone; after `add_child(app)` the probe compares `DccTheme.is_dark()` to what was requested and, only on mismatch, calls the shell’s own public `toggle_theme()` rather than the private `apply_theme`+`rebuild_theme` pair. Then it **asserts a value that DIFFERS between palettes** before trusting the leg: `line` must be `(1,1,1,0.10)` dark, `(0,0,0,0.14)` light. Measured: default leg `(1,1,1,0.1)`, `--light` leg `(0,0,0,0.14)` — **genuinely different**, 48/48 each. **Proven by mutation, independently twice.** Restoring the pre-boot call and short-circuiting the fix-up makes the dark leg **exit 1** with *"palette actually in effect got=(0,0,0,0.14) want=(1,1,1,0.1)"*, reproducing the original defect exactly — so the assertion is not inert. The dark leg now prints *"booted light, requested dark"*, which is direct runtime proof of the mechanism rather than a claim about it. **The 47 geometric checks were never invalidated** — what was false was the COVERAGE claim, and coverage claims are what schedule the next batch. Machine preference ends `light`, cfg byte-identical across eight Godot runs |
| ~~**Button padding: the canvas DOES disambiguate it — by height role**~~ — **CLOSED 2026-09-08 after five counts** | `design/mcp-2026-09-07/Cartalith DCC Environment.dc.html` | small | **Re-opened at the canvas 2026-09-07 and the premise did not survive. Two different figures were in play and neither is the action button’s.** **This row said `4px 12px`.** That string occurs **4 times in the PC canvas and on none of the controls carrying `border-radius:8px`** — the shape the shell’s buttons now use. **`DccTheme.ROLE["btn_pad_x"]`’s own comment said *"Action button: `padding:3px 11px`"*.** Every instance of `3px 11px` in that canvas carries **`border-radius:999px`** — it is a **PILL**: the `add`/`subtract` mode chips at `ENV:255-256`, the breakpoint targets at `ENV:202`. **Mis-attributed, and it was a false claim sitting in shipping code**; corrected in place with the census. **What the canvas actually gives a radius-8 control, as a census rather than a pick:** **8 of 18 use `padding:2px 12px`**, then 5 at `2px 9px`, 2 at `3px 10px`, 2 at `0 14px`, 1 at `2px 13px`. `action()` resolves **10/4** on pointer. **Still not applied, deliberately.** x 10→12 widens and y 4→2 shrinks, and this is the change with a named regression history — the **265 px tool bar** and DS-03’s **eight over-wide minimums**. **It needs `_ds03fit_probe` and `_ds03shot_probe` as guards, and a decision on which census entry the action button is**, since the canvas does not draw one control that is unambiguously it. **Do not sweep this** **UPDATE 2026-09-08 — the row’s own blocking question was false, and answering it cost one ruling.** This row said *"a decision on which census entry the action button is, since the canvas does not draw one control that is unambiguously it."* **It does.** Every radius-8 node carries a height role, and the role is the population: **`--btnH` = 28px is the action button (N=14)**, `--ctl` = 24px the small inline chip (N=15), `--tool` = 30px the tool bar (N=1). **All eight `2px 12px` nodes are `--ctl` (7) or `--tool` (1) — not one is `--btnH`**, and `role_px("btn_pad_x"/"btn_pad_y")` feeds exactly `action()` and `modal_button()`, both `--btnH`. **A ruling was made on the plurality, applied, refuted by the verifier and reverted within the hour**; `LARGE_ITEM_RULINGS.md` keeps it as **Ruling G, withdrawn in place**, because how it was wrong is worth more than it was. **Counted independently twice more** — the verifier’s script and a fourth census over all 888 `style=` attributes — agreeing node for node. **What `--btnH` draws:** `4px 14px` ×3, `4px 15px` ×3, `4px 13px` ×3, `6px 18px` ×2, `0 14px` ×2, `4px 12px` ×1 — **y=4 on 10 of 14, x=14 on 5 of 14**, the only x with a plurality. **So the shipped code is already half right**: `action()` bypasses the ROLE row with its own `10`/`4`, and **4 is the canvas figure**. **WHAT IS LEFT is x alone** — 10 against a plurality of 14 — and that is the half carrying the named regression history (**the 265 px tool bar, DS-03’s eight over-wide minimums**), so it stays guarded by `_ds03fit_probe` and `_ds03shot_probe` and is **not** a sweep. The literal lives in `dcc_widgets.gd`, not `dcc_theme.gd`. **Four wrong figures have now passed through this row** — `4px 12px`, `3px 11px`, `2px 12px`, and 18-as-the-population. **Count the population before the plurality** **DONE: `action()`’s pointer x is 10 → 14; y stays 4, which was already the canvas figure.** **The census was counted a fifth time by the verifier** and matches node for node: exactly 14 `--btnH` nodes carry padding, `4px 14px` ×3, `4px 15px` ×3, `4px 13px` ×3, `6px 18px` ×2, `0 14px` ×2, `4px 12px` ×1 — **x=14 is 5/14, the only plurality; y=4 is 10/14.** **The guard ran BEFORE, which is the thing that is usually missing**, and file timestamps prove the order: `before.txt` 04:32:22 < `dcc_widgets.gd` 04:33:32 < `after.txt` 04:35:52. `_ds03fit_probe` passes both legs — docks 372/372 and 304/304, tool-options band 40 of cap 40 on all 12 rail nodes. **The named regression did not recur: the tool bar renders 1920×40 both ways**, and the band’s worst-case minimum width moves 1685 → 1717 against 1920 actual. **`modal_button()` was declined on evidence, not assumption**: it hardcodes `button_box(…, 18, 8)` and never calls `role_px`, citing its own canvas literal `padding:8px 18px` — a different class, zero lines changed. **One refutation, cosmetic**: the lane reported *"1617 lines, 43 differing, every one a +8"*; the real diff is **25 differing lines, +8 ×22, +32 ×1, +40 ×2**. The conclusion it supported — zero `pos=`/`size=`/`min.y` movement — was re-verified programmatically and holds |
| ~~**The phone dock sheets reserved the gesture inset but not the app’s own bar**~~ — **FOUND AND FIXED 2026-09-08 while looking for something else** | `ANDROID_UI_SPEC.md` | medium | **A second, genuinely different defect under the same symptom.** `left_dock`/`right_dock` (opened via MORE ▸ Window) set `offset_bottom = -_safe_bottom()` — the **system gesture inset only** — so an open dock sheet’s bottom **169 px sat drawn behind the app’s own MAP/GENERATE/PLAN/MORE bar**. **The two bands are disjoint and stacked, so subtracting both is correct rather than a double-count** — measured: bar `y=2119..2288` (h=169), gesture inset `2288..2340` (h=52), bar bottom **exactly** equals inset top. Now `-(_safe_bottom() + bar_reserve)` = **−221**, and both sheets end at 2119 with a **0.0 px gap** to the bar. **`bar_reserve` prefers the bar’s MEASURED `size.y` over the `H_PHONE_BOTTOM_NAV` constant** — 169 rendered against 168 declared, a 1 px gap the constant alone left uncorrected — mirroring `_phone_bottom_reserve()`’s existing pattern. **Two mutants, both killed**: reverting to `-_safe_bottom()` reproduces the pre-fix 2288-vs-2119 exactly; subtracting `_safe_bottom()` twice over-reserves to 2067. File restored sha256-identical each time. **The lane said plainly that this does NOT fix the device screenshot’s subject**, and the verifier confirmed the chip clip is unchanged after it — which is why the real cause got found instead of papered over |
| ~~**The phone shell surfaces no per-stage generation readout, and the canvas draws two**~~ — **CLOSED 2026-09-12: already built, and it matches the canvas** | `ANDROID_UI_SPEC.md` | medium | **Found on glass 2026-09-07 while timing a 2K generate, and it bears directly on the Android conformance row above.** `world_workspace.gd`’s **`Pipeline status`** section — ten rows plus a rolling `%02d NAME -- %.2fs` log, built in `_build_generate_head` — is reachable **only through the World workspace’s `Generate` L2 category**. The phone GENERATE sheet is a horizontally-scrolling action strip with an empty body; MORE carries totals only. **The app computes all ten per-stage times on the phone and shows none of them**, which is why the timing had to be read off the `GENERATING — STAGE NN` pill by screencap at ~1.02 s/frame. **`Cartalith Android.dc.html` draws exactly this, and I named the wrong collection when I first wrote this row — corrected 2026-09-07 after a lane refuted it and I re-checked the canvas myself.** GENERATE’s pipeline list is **`GENSTAGES`** plus **`progLog`**; **`stageRows` is the JOURNEY PLANNER’s leg list** — it sits past `tabIsPlan`, under the heading *"ROUTE · VHAL SERAI → PORT AMRE"*, with `st.days` and `st.ovNote`, and `resGroups` is on that side too. **Corrected again the same day, because my first correction manufactured an explanation for numbers I had not re-measured:** I labelled them "character offsets" and said the byte offsets ran "~330 higher". Both halves were false. The numbers came from a scratchpad copy read with newline translation on, so they are neither character nor byte offsets — measured three ways against the vendored file, `stageRows` is at **41 733 newline-translated / 42 115 characters / 42 190 bytes**, and the byte-minus-character delta across these six symbols runs **52 to 126, never ~330**. **So the fix is not a better number: grep the symbol and name the tab guard it sits under.** `progLog` is between `tabIsGen` and `tabIsPlan`; `stageRows`, `resGroups` and the route heading are past `tabIsPlan`; `GENSTAGES` is later still. **Building `stageRows` into the GENERATE sheet would have put a journey planner inside the generation screen.** **So the canvas and the measurement do agree on what is missing — just not under the name I gave it** **Re-opened at the symbols while choosing a concurrent batch, and the premise is gone.** The row says the phone GENERATE sheet has *"an empty body"*. **It does not.** The canvas’s GENERATE panel is a 10-name `GENSTAGES` list, a progress title `NN · name`, a percentage, a track and a three-line `progLog` (`Cartalith Android.dc.html` AND:706, AND:1010, AND:261). `world_workspace.gd` builds the same: `_pg_progress_card` with a `"%02d · %s"` stage title, a percentage and a track; `_pg_paint_progress` repainting them on every `generation_stage` tick; per-stage groups over `STAGES` (`_pg_group`); and the desktop’s own rolling `_stage_log` **tailed to the canvas’s three lines**. **One documented difference, and it is the engine’s:** the canvas’s percentage adds a within-stage fraction, and `GenerationProgress` carries a stage index and a count only, so the shell’s percentage moves in whole stages — its own comment says so. A 2026-09-08 screenshot of the expanded sheet on the handset already showed the stage list. **Closed on code and canvas, not re-measured on glass** |
| ~~**WORLD ▸ Biomes carries a vestigial Biome-paint panel that is rebuilt from nine sites and never shown — and one of its two comments still says it is shown**~~ — **CLOSED 2026-09-13: absorbed by the owner’s left-rail re-sort** | `GUI_GAP_REGISTER.md` | small | **Found 2026-09-12 by a PC left-rail inventory, then re-opened at the symbols.** `world_workspace.gd` creates `_paint_body`, parents it into the Biomes category and calls `_build_paint(_paint_body)` from nine sites, but **its only visibility write is `_paint_body.visible = false`**; `_on_tool_armed` sends an armed paint tool to `right_dock_ctrl.show_paint(_paint_layer, …)` — **the right dock owns Biome paint.** The comment block written for the 2026-09-07 sculpt gate already says so (*"that body is vestigial, not hidden-until-armed"*); **the comment directly above the panel contradicts it** (*"shown whenever the Biome-paint tool is armed"*) and is the stale one. **Before deleting the panel, check one coupling:** `_build_paint` does not initialise `_paint_layer` — it reads it — and the variable’s only assignment (`:2572`) sits behind the panel’s own *Target field* choice, bound to `_on_paint_layer_changed`. **If `right_dock.gd::show_paint` offers no target choice of its own, the paint target is stuck at its default today** and deleting the panel would bury that. **Waits on the owner’s re-sort of the left rail** (2026-09-12) **Absorbed, 2026-09-13:** the owner’s re-sort (Ruling L, decision 2) makes this panel real as WORLD ▸ Sculpt ▸ Biomes and retires the right-dock copy. Carried into *Implement the owner’s re-sorted PC left rail*. |
| ~~**CIVIL builds its Populate and Diagnostics block in two categories**~~ — **CLOSED 2026-09-13: absorbed by the owner’s left-rail re-sort** | `GUI_GAP_REGISTER.md` | small | **Found 2026-09-12 by the PC left-rail inventory and confirmed at the symbols.** `civilization_workspace.gd::_build_settlement_gaps` builds *Populate* (Auto-populate world, Clear places & routes) and calls `_build_settlement_diagnostics`. It is called from `_build_civilizations` **and** from `_fill_settlements` (once in each of that function’s empty and non-empty branches), so the same controls appear under **CIVIL ▸ Civilizations** and **CIVIL ▸ Settlements**, each copy rebuilt independently. No comment calls it deliberate. **The owner is re-sorting the left rail (2026-09-12); settle it there** **Absorbed, 2026-09-13:** the owner’s re-sort keeps a single Diagnostics under Settlements and turns Civilizations into Populate. Carried into *Implement the owner’s re-sorted PC left rail*. |

### From 2.7 Android and on-device verification (batch wf48, 2026-09-13)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**A view re-opened while already armed gets its workspace panel put back — general, not planner-specific**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf48)** | `DCC_SHELL_SCOPE.md` | medium | **Found by the planner lane 2026-09-07 while verifying its own fix on glass, and it would have made that fix wrong.** `app.gd::open_journey_planner()` calls `select_domain_mode()` **before** `open()`; `_select_domain()` re-shows `_workspace_panels["civilization"]`, and **`arm_tool()` early-returns when the tool is already armed, so `tool_armed` never fires and `_show()` never re-hides it.** On glass, PLAN → close → PLAN returned TOOLS / Civilizations / Factions instead of the party form. **Fixed for the planner** by re-asserting the three visibility writes when `_active`. **Filed because the PATTERN is general:** any view that hides a workspace panel on arm and is re-opened while already armed inherits the same defect. **Enumerate the other views that do this** rather than waiting for each to be reported **BATCH 2026-09-12 — enumeration verified, fix refuted as blocking, REVERTED** (`.claude/resume-2026-09-12/rearm_2026-09-12.patch`). **Verified: the journey planner is the ONLY view** that hides a workspace panel on arm — 8 `tool_armed` and 3 `workspace_changed` subscribers read statically and live via `get_connections()`, plus a sweep of 10 tools × 3 domains with no mismatch. **So the general case this row asked for does not exist.** **Refuted:** the lane marked its own gate G3 HIT (no other view has the pattern, so build nothing general) and shipped a 45-line change anyway, which swallowed real navigation into non-planner Civilization destinations while the planner is armed (Landmarks, Factions, phone Simulation, Military). **What remains is narrow:** two further entry points into the planner itself (a bare `select_domain` / `select_domain_mode` re-entry) still restore its panel, and any fix must not swallow navigation elsewhere **Fixed 2026-09-13 in `journey_planner_view.gd` together with the next row’s fix in `app.gd`:** the planner stays up on a bare `select_domain("civilization")`, `select_domain_mode(…, "planner")`, Window ▸ Workspace and the phone `_go_civilization`; with the planner armed every other route navigates (14 CIVIL categories, landmarks, factions, infra modes, rail nodes, other domains, phone Simulation). **The verifier found the pre-fix code swallowed all 14 non-planner CIVIL categories, not the 4 filed.** Known and measured harmless: `open_category()` changes mode through `apply_domain_mode()` without emitting `workspace_changed`, so the re-entry memo can go stale. |
| ~~**Re-entering a domain overwrites the armed tool’s options row and chrome — every tool, desktop and phone**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf48)** | `DCC_SHELL_SCOPE.md` | medium | **Found 2026-09-12 by a verifier; pre-existing** — identical on the pre-fix code. `app.gd::_on_workspace_changed` runs unconditionally on every domain change and rebuilds per-domain chrome, so a bare `select_domain` or `select_domain_mode` re-entry replaces whatever tool is armed: Way’s *INFRA · WAY … Commit / Discard* options row becomes *CIVIL · INSPECT*; with the planner armed its route picker is replaced and the desktop rail foot reads PLANNER instead of JOURNEY; on phone the timeline row rebuilds. **The planner’s own panel fix cannot reach this** — it is the tool-options row, not the workspace panel. Fix at `_on_workspace_changed`, preserving the armed tool’s chrome, and probe all 10 tools **Fixed 2026-09-13 at `app.gd::_on_workspace_changed`** with a re-entry memo compared against the current id and `active_mode()`: options row, rail foot, timeline children and panel visibility survive a bare `select_domain`, a `select_domain_mode` and a re-click of the lit rail node for 15 tool/domain cases, desktop and phone (pre-fix: 66 desktop / 55 phone failures). No signal re-fired; three mutations killed. Regression probe `_rearmscan_probe`. Side effect filed as its own row: a stale CARTO caption that re-entry used to correct now persists. |
| ~~**A seed typed into New World is sometimes not the seed you get, and nothing says so**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf48)** | `DESIGN_HANDOFF.md` §6.7 | medium | **Found on glass 2026-09-07 and INTERMITTENT — stated that way deliberately, because the flat version of this claim would route someone at a commit path that demonstrably works.** Typed **311447**, field showed 311447, keyboard dismissed with BACK, Create → the world is **`ELDRA · 51127`**; the typed value was then used by the *next* Generate World. The verifier reproduced it independently (typed 9991117 → `ELDRA · 366207`) **but the immediately prior run typed 424242 the same way and got 424242**, and an untouched rolled seed committed correctly. **Suspicion only, not established:** `DccWidgets.number` builds a bare `SpinBox` with no `update_on_text_changed` and no `text_submitted` wiring, and `_collect()` reads `.value` — so an uncommitted edit would read as the old value. **Open the commit path before believing that.** **The severity is that it is silent:** nothing on screen says the world is not the seed asked for **BATCH 2026-09-12 — mechanism found, fix refuted as blocking, REVERTED** (`.claude/resume-2026-09-12/seed_apply_2026-09-12.patch`). **Mechanism, measured on desktop:** Android BACK dismisses the keyboard with no focus change, so the `SpinBox` never commits the typed text and `.value` stays stale. The real read site is `new_world_dialog.gd::request()` — `_collect()` does not exist — and `request()` is also called by `app.gd::_run_pipeline()` and the heightmap import, which is how a stale seed surfaces on a later Generate. **The fix `seed_input.apply()` in `request()` broke rolled seeds:** while the dialog is hidden, `LineEdit.text` does not follow `.value`, so `apply()` wrote stale text back over a freshly rolled seed (verifier: rolled 52199, built 77777; the phone roll was undone in the same frame). **Next:** commit pending text only when the user actually edited it, or at edit time — not unconditionally at read. Also corrected: `DccWidgets.number(` has **13** real call sites in **5** files, and the place editor’s Population/Age and the journey planner act on every change, so `update_on_text_changed` would be per-keystroke work **Fixed 2026-09-13 in `new_world_dialog.gd`:** `_seed_dirty`, raised by the seed `LineEdit`’s `text_changed` (a real keystroke; measured NOT raised by `.value =`, `set_value_no_signal`, `text =`, `apply()` or `text_submitted.emit`), cleared by `randomise_seed()`; `request()` calls `apply()` only when dirty. Verifier matrix on the built world’s seed: typed-uncommitted, typed+Enter, rolled open, **rolled while hidden then `_run_pipeline()` (the case that sank the 2026-09-12 attempt)**, `_new_seed`, `_regenerate_now`, both orders of typed/rolled, and the phone layout roll (540×1031, `pressed.emit()`). **Known:** `request()` is no longer a pure read — `world_workspace._pg_seed_row` commits a pending edit when it displays the seed (measured harmless); a typed seed never Created is used by the next Run, matching what the field shows; `_seedcommit_probe` reads `--force-touch` from engine args, so its phone cell runs on the desktop layout. Also found: the seed-overflow row filed in §2.7. |

### From 2.7 Android and on-device verification (batch wf49, 2026-09-13)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**THE SHIPPED TABLET DOES NOT FIT ITS OWN FRAME IN PORTRAIT — and the docks were NEVER the cause**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf49)** | `TABLET_UI_SPEC.md` | large | **Still open, and the cause is now named. I attributed it to the docks and that was wrong** — my own figure (`rail 47 + left 400 + viewport 237 + right 400 = 1084`) reads the RESULT of the layout, not its cause: **`viewport_area` has no minimum of its own** (measured `get_combined_minimum_size().x = 0.0`) and simply expands into whatever the row is given. **The binding constraint is `tool_options_row` at a combined minimum of 1057 px** — six touch-sized text buttons — plus 28 px of bar margins, giving the shell VBox a floor of **1085**. The app is 800 wide, so **overflow is +285 whatever the docks do**, and the verifier confirmed `shellVBox.size.x = 1085` **before AND after** the dock fix, with the right dock edge still at x=1085. **The docks were still worth fixing and are now canvas-correct** (232/232, their share 848 → 611), but that could not and did not clear the frame. **What remains, in order:** the tool-options bar must stop propagating a minimum, then the 7→3+overflow menu collapse. **Do NOT trust the "1085 → 850" figure** an earlier report offered for the first step — the verifier found no measurement supporting it; the largest measured remaining contributors are dock_row 611, status_row 575, menu_bar_row 486. **And landscape is fine only by being wide enough, not by design:** the shell floor is 1085 and every landscape frame tested is ≥ 1280. **A 1024×768 landscape tablet would overflow by 61 px** **Fixed by the tablet options row’s `ScrollContainer` (canvas `overflow-x:auto`), re-applied 2026-09-12/13 with docks at 400.** Verifier-measured windowed: 800×1280 shell minimum 1085 → 619 (right dock ends at 800), 1024×768 +61 → 848, 1280×800 848. The portrait LEFT dock still draws 331 against 232 — that is the portrait slider row, still open. |
| ~~**Phone bottom nav: MAP and GENERATE draw different icons from the canvas**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf49)** | `ANDROID_UI_SPEC.md` | small | **Found 2026-09-12 by checking a lane’s citation finding at the canvas.** The Android canvas renders each tab’s glyph literally — `{{ t.g }}` in 14 px IBM Plex Mono — from `tabs=[{id:'map',g:'▤'},{id:'gen',g:'⌗'},{id:'plan',g:'➔'},{id:'more',g:'⋯'}]` (AND:1460). `dcc_shell.gd` builds the same four tabs with SVG icons: `domain_carto` (a folded map), `domain_world` (a globe), `tool_route` (a curved arrow), `nav_more` (three dots). **PLAN and MORE agree in form; MAP and GENERATE do not.** Also stale: `dcc_icons.gd`’s comment describing the nav as PANELS plus MORE *"because its tab set predates the v3 domain model"* — the canvas and the shell both have four function-keyed tabs. (`dcc_theme.gd`’s bottom-nav row reading "five" sits inside a supersession notice that already records the change, and is not stale.) **Fixed 2026-09-13:** `PHONE_TABS` carries the canvas glyphs ▤ ⌗ ➔ ⋯ as literal text, drawn by `_phone_bar_cell()` as a mono `Label` at the phone-scaled 14 px (they render; the mono font’s system fallback covers the two Plex Mono lacks). Tab buttons unchanged (270×168). Unreported by the lane: the pill grew 57 → 71 px (closer to the canvas’s ~68) and the caption moved 7 px; `dcc_icons.gd`’s new header cites `docs/ANDROID_UI_SPEC.md`, which lives at the repository root. |
| ~~**Arming Region in CARTO Labels paints the caption *CARTOGRAPHY · STYLE*, and re-entry no longer corrects it**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf49)** | `DCC_SHELL_SCOPE.md` | small | **Found 2026-09-13 by the wf48 verifier; pre-existing.** Identical on HEAD and on the fixed tree, so it comes from a `tool_armed` subscriber, not from `app.gd`. Before the re-entry fix a domain re-entry rebuilt the row and happened to correct it to *CARTOGRAPHY · LABELS*; now that the armed tool’s chrome is preserved, the stale caption persists. Find which subscriber writes the mode half of the caption on arm **Fixed 2026-09-13:** `app.gd::_tool_options_cartography_default()` is the one source, naming the current mode; `cartography_workspace.gd`’s hardcoded *CARTOGRAPHY · STYLE* duplicate is gone. `_cartocaption_probe`: 0 failures desktop and phone (hardcoding STYLE gives 12 each); `_rearmscan_probe` still 0 and 0. |

### From 2.7 Android and on-device verification (batch wf50, 2026-09-13)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**The GENERATE sheet shows half a body row at peek because the shell built it without the canvas’s header block**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf50)** | `ANDROID_UI_SPEC.md` | medium | **Found 2026-09-08 by doing what the METHOD row prescribes** — `adb exec-out screencap` on the attached OnePlus (`9608b26b`) rather than a desktop probe — **and it is exactly the class that row says probes cannot see**: the handler runs, the control renders, and a finger still cannot read it. **Measured, not eyeballed.** The nav bar’s opaque band begins at **y=2116** of 2340. The PIPELINE chip’s amber outline runs from **y=2067**; between 2105 and 2115 only **1-2 px per row** are amber (its left and right border strokes), then at **2116/2117/2118 the count jumps to 58/61/36** — those are the tops of the *"PIPELINE"* glyphs — **and at 2119 it is zero.** So roughly three pixel rows of a ~20 px label are visible; SCULPT is cut identically. **The chip is occluded, not merely short**: a rounded chip bottom tapers over several rows, and this terminates flat under the band. **Stability checked before filing**: two consecutive captures differ **only** at (96,31)-(108,47), the status-bar clock digit — the chip row is byte-identical, so this is not a mid-gesture frame. **The sheet does not reserve the nav bar’s height**, which is the shape of the fix; whether the bar is the app’s own row or the system gesture bar is the first thing to establish, because they need different insets. **Re-check after the next install**: the APK on the device is `lastUpdateTime=2026-09-08 00:00:24`, which **predates every commit in this session**. Nothing committed today touches this sheet, so it is very likely still present — but that is an inference, and the screenshot is evidence only about the build it came from **CORRECTED within the hour, and the correction is the useful part: it is CONDITIONAL on the detent.** The row first stated the clipping flatly. **Expanding the sheet draws PIPELINE and SCULPT in full, unclipped** — so the defect is not "the chips are clipped", it is **"the collapsed detent positions the chip row under the nav bar"**, and a fix verified only in the expanded state would look correct and change nothing. **Re-collapsed and re-measured to be sure**: the PIPELINE amber outline returns to exactly **2067..2118**, the same rows as the original capture, so it reproduces on demand rather than being a one-off frame. **How the correction was found is worth as much as the correction.** Navigating the four tabs by `adb shell input tap` moved the sheet between detents as a side effect — the defect only reappeared because the sheet was put back. **A screenshot captures ONE state of a multi-detent component, and naming the detent is part of the defect.** **Two phantom defects were also produced and discarded in the same session**, both from reading a downscaled view: two "clipped tab labels" at the top of the GENERATE frame, and a "cut-off row above the CIVILIZATION header". **Neither exists in the full-resolution crop of the same pixels.** A third line of investigation — *"the MORE tab does not respond to a tap"* — also collapsed: MORE, MAP and PLAN all stopped responding together, because a full-screen panel had opened over the nav bar and the taps were landing on its content. **Nothing from any of the three was filed** **RE-DIAGNOSED 2026-09-08 by a lane, and my filed cause was wrong.** I wrote *"occluded by the bottom nav bar"* from the screenshot. **The bar is not drawing over it.** The GENERATE sheet’s own `ScrollContainer` clip boundary sits at **y=2119** — and the bar’s top edge is **also y=2119**, because both are derived from the same `_phone_nav_reserve()`. **Two edges that coincide exactly look like one edge covering the other.** **Reproduced on the DESKTOP composition without a handset**, which is the finding that matters beyond this row: windowed 1080×2340 `--force-touch` gives bar top **2119** against the device’s measured **2116** — **3 px apart in the same UI state**, so the phone probes in this tree are modelling the device after all. The mode-segment button occupies y=2067..2182 and the clip admits only its top **45 %** (52 of 115 px); the label is centred lower, hence *"glyph tops only, flat cut"*. **Root cause, and it is a stale budget rather than a z-order bug:** `world_workspace.gd` places the PIPELINE/SCULPT segment as the **first scrollable row** of a peek-budgeted sheet, where the desktop dock pins its own copy of the same control above the scroll. **The peek height was sized for the OLD one-line tool-options strip and never revisited when the 2026-09-07 fix swapped in the taller multi-group column.** **Out of the fixing lane’s ownership and deliberately not reached across for**: the fix is in `world_workspace.gd`, and either option — pinning the segment above the clip, or growing the peek floor — **breaks `_detent_probe.gd`’s "peek within 1dp of 66" conformance check**, so it needs that probe re-specified in the same pass rather than tuned around **RE-DIAGNOSED A SECOND TIME 2026-09-12, at the canvas — and the first fix attempt was refuted and reverted.** **Root cause: the shell built the sheet without the canvas’s header block.** The Android canvas draws a header above the scrolling body — title `sheetTitle`, subtitle `sheetSub` and a 38 px close circle `hSheetClose` (AND:180-186) — then the body, `flex:1;overflow-y:auto` (AND:187), whose **first row** on GENERATE is the PIPELINE/SCULPT segment (AND:242-245). At peek (`_detH`, 66) handle and header fill the sheet and **no body shows**. `dcc_shell.gd::_build_phone_tool_sheet` says it outright: the prototype’s grab region is *"the 20 dp handle row plus a … roughly 68 dp of grabbable header. This sheet has no header block"*. So after its 20 dp grab row, part of the body leaks through at peek — the half-visible chips (52 of 115 px at 1080×2340). **Fix: build the header block per AND:180-186, not pin the segment.** **What was tried and refuted:** a lane pinned the segment above the scroll. The chips became 115/115 visible, but the sheet rendered **98 dp at peek against the canvas’s 66**, the growth emitted no `phone_insets_changed` (at cold boot the sheet covered 29 px of the navpad and hid the coordinate and scale labels), and a new probe check required the deviation. **The brief’s own gate had named that obstacle** (*"88 dp > 66 dp budget"*); the lane argued it into *"no collision"*. Reverted; the 241-line patch is kept in the session scratchpad. **Owns `dcc_shell.gd`, so it is serialized after the tablet COMPOSITION lane** **BATCH 2026-09-13 (wf49) — the header block was BUILT and works, but is HELD as a patch** (`.claude/resume-2026-09-12/held_shell_b_2026-09-13.patch`, `dcc_shell.gd` hunks plus `_detent_probe.gd`). Verified: at peek the chips are 0/115 px visible (was 52/115), 115/115 at half and full, `phone_insets_changed` fires as on HEAD, peek 66.38 dp. **Refuted on:** the title and subtitle use unscaled `FS_SMALL` 11 / `FS_MICRO` 9 at phone scale 2.62, drawing 16/13 px where the canvas’s 11 px / 9.5 px means about 29/25 px (use the phone font scaling), and the close glyph is unscaled too; subtitles come from `PHONE_TABS.tip` and match the canvas’s `tt` table on no tab (MAP should read *layers · style · annotation*, MORE *program · data · preferences*, GENERATE carries live seed/mode). Also: the sheet grew 173 → 174 px (labels up 1 px) though the lane called it byte-identical, and removing the whole title row still passes the peek check because the height floor alone hides the body. **Next:** re-apply the patch, fix fonts and subtitles, and make the probe fail when the header is absent **Fixed 2026-09-13 by building the canvas’s header block** in `dcc_shell.gd::_build_phone_tool_sheet()`: title, subtitle and close circle at the phone font scale (`_pfont` 11 / 9.5 / 13 → 29 / 25 / 34 px at 1080×2340). Verified windowed at 720, 1080 and 1440 wide: peek height equals HEAD (115 / 173 / 231); the PIPELINE chip is 0 px visible at peek (HEAD 34 / 52 / 69) and fully visible at half and full; `phone_insets_changed` unchanged. The 1 px growth of the first attempt came from the sheet stylebox’s top-border margin. MAP’s subtitle matches the canvas byte for byte; GENERATE’s follows the live seed and the pipeline/sculpt switch, and reads the no-world state after the main loop’s fix (the lane’s version said *seed 0* at boot). A re-entrant gdext panic the lane hit while wiring the live seed is guarded by `bridge.generating`. The probe now fails with the header removed or hidden. Details left out are filed as their own row. |
| ~~**A failed project open still dismisses the welcome gate**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf50)** | `GUI_GAP_REGISTER.md` | small | **Observed on the device 2026-09-07, and the desktop does not show it.** After `project_open` and the `load_save` fallback both refused a corrupt zip, the screen is the main shell with a dashed title and **no world** — the welcome gate is gone and nothing was opened. So a user who picks the wrong file lands in an empty shell with no way back to the picker except the menu. Screenshot kept **BATCH 2026-09-13 (wf49) — fixed on desktop, HELD as a patch** (`.claude/resume-2026-09-12/held_gate_b_2026-09-13.patch`: `app.gd` `_load_project() -> bool`, `open_project_dialog.gd`, `phone_project_picker.gd`, hide only on success). Verified: on desktop the dialog stays and the refusal shows in the status bar (welcome tile, Open, browse); a good save ends exactly as on HEAD. **Refuted on phone:** the picker stays, but the error toast is drawn UNDER the full-screen picker window and the status hint is hidden, so nothing on screen says it failed (HEAD showed the toast). The lane’s probe checked the hidden hint’s text. **Also found:** `load_save()` returns a bare `bool` on both sides of the boundary, while `world_gen.project_open()`’s dictionary carries a real `error` string (e.g. *zip error: invalid Zip archive: Could not find EOCD*) — show that **Fixed 2026-09-13:** `app.gd::_load_project()` returns whether it opened, and `open_project_dialog.gd` / `phone_project_picker.gd` hide only on success. Verified windowed on all five entry points: after a corrupt open the phone picker is the top-most window and its warn banner shows the refusal (HEAD: picker gone, nothing on screen); the desktop dialog stays; after a good open every end state matches HEAD. No canvas draws an error state here (concept grep of all three canvases); the banner uses the canvas’s own warn colour and `staleNote` pattern. The desktop placement and the missing real reason are filed as their own row. |

### From 2.7 Android and on-device verification, and 2.8 (batch wf53, 2026-09-13)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**CONFIRMED: `MORE ▸ Window ▸ Left dock` draws ON while the phone’s left sheet is closed**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf53)** | `ANDROID_UI_SPEC.md` | small | **Reported by the planner lane 2026-09-07, and deliberately NOT confirmed** — the verifier spent its device time on the three owner-reported defects and **carried this forward as unverified rather than counting it as found.** The claim: the toggle reads the desktop’s `_left_collapsed` rather than `_left_sheet_open`, so **the one switch that opens the sheet says it is already open.** **Re-observe before fixing.** It was reported from a screenshot (`10_window.png`) during a run whose main purpose was elsewhere, and this project has twice built work on an observation taken in passing **CONFIRMED 2026-09-13 (run wf51), with a different mechanism than filed.** Nothing in `menus.gd` or `phone_menu.gd` reads `_left_collapsed`. `menus.gd::_window()` builds its five region check items **hardcoded checked at build** and only flips that shadow locally when its own row is pressed — it never reads the real state. On phone the Left dock’s truth is `DccShell._left_sheet_open`, false at boot, so the switch reads ON with the sheet closed; opening the sheet any other way (`phone_menu.gd::_open_left_sheet()`, the sheet’s close button, `journey_planner_view.gd`) drifts it again, and `_capture_layout()` saves `left=true` while closed. **Fix (reported, not applied — `app.gd` was another lane’s):** a query counterpart to `app.gd::toggle_region()` (phone: `_left_sheet_open` / `_right_sheet_open` / status shown; else the region node’s `visible`) and an `about_to_popup` refresh of all five items in `_window()`. The untracked `_leftdockcheck_probe.gd` asserts the defect and must be inverted when fixed **Fixed 2026-09-13:** `app.gd` gained a query counterpart to `toggle_region()` (phone: the sheet-open flags and the status region; desktop: the region node’s `visible`), `menus.gd::_window()` refreshes all five check items on `about_to_popup`, and the layout capture reads the same query. Verified after every opener (the row, the sheet close button, MORE rows, the planner, the right sheet, status, Reset layout, domain switches): 0 mismatches (HEAD: 32 on phone, 6 on desktop). **A regression the verifier caught was fixed by the main loop before commit:** `_apply_layout()` compared layouts against the now-truthful checks and opened a full-height right sheet on phone; it now leaves phone sheets alone (`_postfix53_probe`). |
| ~~**Desktop: a failed project open reports outside the dialog, and never says why**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf53)** | `GUI_GAP_REGISTER.md` | small | **Found 2026-09-13 by the wf50 verifier.** After a corrupt open from the welcome tile, the Open button or browse, the dialog correctly stays — but the refusal is only in the status hint behind it (at 1600×900, at (1178, 880)), not in the dialog’s own `_picker_note` / `_foot_note`, which `_say()` already writes. **And the real reason exists:** `load_save()` calls `note_error("open %s failed: %s")`, and the public `bridge.last_error().text` reads *open … failed: zip error: invalid Zip archive…* right after the failure — so no bridge change is needed to show it, on desktop or phone. The phone picker’s warn banner (shipped) still shows generic wording **Fixed 2026-09-13:** the refusal, carrying `bridge.last_error()`’s reason (*open … failed: zip error: invalid Zip archive…*), is written into the surface the user is on — the desktop dialog’s own note and the phone picker’s banner. Verified through welcome tile, browse and drop; good opens identical to HEAD. **The gallery’s *Open selected* path wrote into `_foot_note`, a `clip_text` Label beside an `EXPAND_FILL` spacer that drew 1 px wide — so every gallery foot note had been invisible since it was built; the main loop made the note fill its row** (`_postfix53_probe`). |
| ~~**`_tabletparity_probe` rewrites the owner’s settings file on every run**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf53)** | `MISTAKES.md` | small | **Found 2026-09-13 by the wf51 verifier.** The run wf51 extension presses the theme and units radios with 14 `id_pressed.emit` calls (HEAD had none), so every run in the live project writes `user://cartalith_settings.cfg` and relies on its own restore — the thing every brief forbids. Its Gate A3 assertion also reads the drawn palette, not the stored mode. **Fix:** run the theme/units legs against an isolated user dir (the verifier’s recipe: a project copy with its own `config/name`), or snapshot and restore the file bytes inside the probe and assert the stored mode **Fixed 2026-09-13:** the probe snapshots the settings bytes, restores them on every exit path, asserts the restore is byte-identical and asserts the stored theme mode. Verified with a dark-theme fixture (restored exactly) and a no-restore mutant (1 failure). The lane itself wrote its fixture into the live file while testing, which the brief forbade; the live theme ended light. |
| ~~**Phone sheet header: the canvas details the first build left out**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf53)** | `ANDROID_UI_SPEC.md` | small | **Found 2026-09-13 by the wf50 verifier, after the header block shipped.** Title letter-spacing is `DccTheme.mono(2)` (2 raw px) against the canvas’s `.2em` (about 5.8 px at scale 2.62); the close circle is 115 px (the `_ptap` floor) against the canvas’s 38 (100 px) with a `line_soft` background instead of `--chip`; `hSheetClose` goes to peek rather than the canvas’s `tab:null`. **PLAN** shows *Journey planner*, which is not canvas vocabulary (`journey · …` / `PLAN · STAGE n`) — the live endpoints need a public selected-journey read in `journey_planner_view.gd`; **MORE**’s page titles need a public stack/title read in `phone_menu.gd`, and its header sits under the full-screen MORE menu anyway. The full detent covers the coordinate and scale readouts, identically on HEAD; the new probe comment calling that *by design* for the half detent is false. The dismiss drag’s mid-gesture height now floors at 173 px (HEAD 116) **Fixed 2026-09-13:** title tracking scaled to the canvas’s `.2em` (spacing 6 px at 1080 wide, was 2); the close circle draws at 38 dp (101 px ink, was 115) with the tap area kept at the floor; PLAN’s header reads `journey · <from> → <to>` from a new public read in `journey_planner_view.gd`; MORE left unwired because its full-screen menu always covers the header (0 px drawn, measured). Peek/half/full heights identical to HEAD at three sizes. Residue: the close fill is `line_soft` (~8% darker) against the canvas `--chip` (~5%); PLAN’s subtitle refreshes on tab pick, not when a new plan is computed — filed. |
| ~~**Block-ground fill still fails triangulation at deep zoom — left out of the roof fix’s scope, and never filed**~~ — **CLOSED 2026-09-13: fixed and verified (batch wf53)** | `URBAN_MORPHOLOGY_SCOPE.md` | small | **Filed 2026-09-12; known since 2026-09-08 with no row.** The roof fix (`360cb17`) removed 151 triangulation failures from `_draw_roofs` and deliberately left **one** survivor: a `draw_colored_polygon` in `draw_layout`’s block/market/farmland ground fill, at `shell/urban_layout_draw.gd:235` as of that commit, unchanged **1 → 1** across the fix — which was also that batch’s evidence of no scope creep. **It was recorded only inside the closed roof row**, and a closed row’s text is exactly where open work goes unseen. **Silent the same way the roof bug was**: Godot logs *"Invalid polygon data, triangulation failed"* and skips, so part of a revealed settlement’s ground is simply not drawn. **The roof bug’s cause — a polygon collapsed below a pixel by the deep-zoom transform — is a plausible candidate here and is NOT established**: log the vertex array at the failing call before reusing that fix. Repro: `_settlepix_probe.tscn`, windowed, emits it on every run **PARTIAL 2026-09-13 (run wf52) — the block fill is fixed; the parcel fill beside it fails the same way.** **Cause, measured:** block 324 of *Sevjuniana* (seed 24601) is a valid convex ~5 m quad; at deep zoom its screen polygon is 0.046 px across and sits at an absolute offset near (276, 33), where float32 products (~9 250, ulp ~0.001) swamp the 0.001 px² winding sum — it triangulates at the origin and fails only at that offset. **Fix:** `_fill_ground_polygon()` keeps the triangulate check and, where it fails, fans the polygon into triangles through `draw_primitive` instead of skipping it. Verified against a pure HEAD copy: engine error 1 → 0; 0 of 746 496 px change at the repro zoom (the polygon is sub-pixel); at z512, 142 px change, exactly the polygon’s area, all inside it (grey → ground). **Still open:** parcel district fills (drawn only when detail ≥ 1, so the repro zoom never reached them) fail from the same cause — **14 parcels, 42 errors over 3 draws at z64** in the same town, identical on HEAD. Route them through the same helper. The helper’s comment carried wrong prose (parcels *measured clean*, blocks *convex by construction* — 127-169 per town are not, up to 107 vertices) and was corrected by the main loop **Fixed 2026-09-13:** the parcel district fill now goes through `_fill_ground_polygon()` like the block fill. At z64 with a per-kind counter: seed 24601 42 → 0 engine errors, seed 777 384 → 0; market, farmland, water, roof and roof shadow fail 0 times on HEAD and tree; 0 malformed parcels of 4 683 and 3 138; both towns’ z64 frames byte-identical HEAD vs tree. `_settlepix_probe` still passes with the fix removed (errors only reach stderr) — see the probe-honesty row. |

### From 2.3, 2.7 and 2.8 (parallel small and medium runs, 2026-09-13)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**The dashed-way pixel residual has no established cause**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `MEMORY_OPTIMIZATION_SCOPE.md` | small | **Three explanations have now been offered and refuted, 2026-09-05.** (1) A join-vs-cap difference — ruled out by a single unsplit chain still differing. (2) Godot's antialiased `draw_polyline` treating a shorter array differently — refuted in isolation: 801 points vs the 77-point visible slice renders diff px 0, and the SOLID path is byte-identical at a 5.8× shorter array. (3) `_dash_phase_track`'s f32 phase accumulator — proposed by a verifier with a measured 43 px → 0, **which did not reproduce**: re-measured windowed both ways, `_segcull_probe` PASSes and `_cull_probe` FAILs 13 of 16 identically under f32 and f64. The widening was kept on principle (a ~3 100 px accumulator in f32 is wrong regardless) and **no probe pins it**. What is known: the residual is confined to the DASHED path. **Also open and owner-decidable:** the chain pad is monotone — `1.0/0.0`→14 cases, shipped `2.0/2.0`→13, `8.0/8.0`→11, `64.0/64.0`→2 — at a cost of deep-pan objects `22 849 → 37 726` (still a 97.2% cut against `1 344 502`) and the all-visible saving essentially gone. The shipped value is the cheap end of that trade **Cause established by measurement:** solid ways show the residual too (11 of 16 cases, up to 157 px) as well as dashed (10 of 16, up to 9 px), and all 13 `_cull_probe` FAIL frames are within `_segcull_probe`’s shipped AA tolerance (max delta 1/255, at most 0.03% of pixels) — Godot’s antialiased polyline rasteriser responds to how points are split into draw calls; a single whole-run chain makes both families exact. Recorded in `map_overlay.gd`; `_cull_probe`’s stale zero-tolerance bar is filed as its own row. |
| ~~**The drop-count warning is written but wired to nothing that tests it**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `SAVEFILE_COMPAT.md` §6.4a | small | **Three mutants SURVIVED, found by the verifier 2026-09-06.** Setting `let dropped = ...` to `0usize` in two places, and deleting `.chain(restore_warnings.iter())` from the `warnings` key, all pass the full `cartalith-godot --lib` suite. The lane's test re-derives the subtraction **inside the test body** rather than exercising the call site, so it stays green with the warning deleted outright. The real constraint is honest — `WorldGen` is a cdylib `GodotClass` and cannot be constructed in a unit test — but that makes this a probe's job, not an untested claim's. §6.4a rung 2 requires the report; nothing currently proves it happens **Fixed:** `_dropwarn_probe` saves a real project, splices one unresolvable landmark kind and one unresolvable icon family into the archive, reopens it and asserts both exact warnings (`1 of 225 landmarks skipped`, `1 of 1 icons skipped`) and 224 landmarks; a clean control fails 2 checks. The Rust mutants were inspection-verified only (no `cartalith-godot` compile that batch). |
| ~~**The de-dup probe pins its key in one direction only**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `GUI_GAP_REGISTER.md` | small | **Latent, and cheaper to record than to fake a fixture for.** The 2026-09-06 fix unified the key through one `_mark_cell()`, and the probe kills the shipped bug (icon side rounding against a truncating ring side). But the verifier reverted the **ring** side alone — the exact mirror — and it **SURVIVED, 13 checks PASS**, because the probe's landmark fixture is integer-only. It cannot be closed honestly today: `Landmark::x`/`y` are `usize`, so a non-integer landmark row **cannot occur**, and inventing one to satisfy the probe would be a fixture asserting a state the type forbids. **Close it when landmark coordinates can hold a sub-cell value** — the same latency the fix's own comment warns about, now pointing the other way **Not a gap:** the surviving ring-side mutant is equivalent for every reachable input — `Landmark::x`/`y` are `usize` (`cartalith-civ` `landmark.rs`), so a landmark cannot carry a sub-cell coordinate; the icon-side mutant still fails 4 checks. Recorded in the probe’s header. |
| ~~**`DccTheme._elide_labels` grows without limit on desktop**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `DCC_SHELL_SCOPE.md` | small | **Found 2026-09-13 by the wf53 verifier.** The new static registry of labels that elide in tablet portrait prunes freed entries only on rotation, which never happens on desktop: 104 entries at boot, 1 054 after 150 domain switches and 50 right-dock rebuilds, 951 of them freed. Prune on insert (or register only on tablet) **Fixed:** labels register only on tablet; `_elidechurn_probe` (HEAD: 104 → 1 104 entries; tree: 0) and `_elidehead_probe`; full dumps identical on desktop, laptop, tablet and phone, including live rotation. |
| ~~**Round 3 batch B: the diagnostic dump gets its review panel; the colour picker gets its warning**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `DESIGN_HANDOFF.md` | medium | **Built and verified 2026-09-06.** **Diagnostic:** `diagnostic_report.gd` already existed at HEAD with all five of the ruling's readouts resolving to real symbols, so the lane built the **review panel** over it rather than the dump — and made panel-and-file agreement *structural*: `DiagnosticReport.manifest()` returns one row array that the panel draws and `build_text()` writes, so "matches row for row" cannot be a coincidence. **It found a real defect while probing:** the filename stamp is one-second resolution, so two reports in the same second silently overwrote, against the file's own comment claiming they did not — fixed with a suffix loop and the false half of the comment corrected. **And a disclosure worth keeping:** turning the seed toggle off still leaves the seed in the log tail, because a `print` put it there; filtering would be guessing at what to redact, so the panel and the file both say the tail is verbatim and unfiltered. **Colour:** the picker changes the viewport and deliberately not exports, and nothing said so. Now it does. Proved end to end — the same world exported under sRGB, Display P3 and sRGB again gives three files of 2 103 237 B at **identical sha256** **Verified built:** `_diagreport_probe` 0, `_diagreview_probe` 0, `_colorspaceexport_probe` 0, `_colorspace_probe` ALL PASS in an isolated copy. The picker’s lead sentence is not asserted (cosmetic). |
| ~~**`TABLET_UI_SPEC.md` is written — the tablet decision can now be made from facts**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `TABLET_UI_SPEC.md` | medium | **786 lines, to the standard of `ANDROID_UI_SPEC.md` §0-§2, produced 2026-09-07 by walking all 49 `sc-if` blocks in the canvas, then finding each engine counterpart BY SYMBOL, then measuring the shipped side.** **Column 1 — already provided, so re-home rather than build (12 rows):** `tool_options_row` (measured 55 px) **IS** the canvas’s `--railH:56` rail; `rail_column` (47 px) **IS** `--railW:52`; `global_tools.gd`’s `MEASURE_GROUP_POINT`/`SECTION_CHANNELS` already draw the canvas’s five measure chips and CROSS-SECTION group in the canvas’s own composition; `right_dock.gd`’s sample is a **superset**; the six timeline layers match verbatim; `journey_planner_view.gd` (4 757 lines) is a superset of `scrPlanner` **including all four result groups by name**. **Column 2 — genuinely new (12 rows), and three cannot be backed at all:** per-stage `Run stage NN` (the engine has no partial recompute — one `generate()` resolves all ten), four of the canvas’s ten stage names have no engine stage, and LIVE/DRAFT/ARCHIVE has no project-state field on disk. **Column 3 — cost as what it touches:** cheapest is undo/redo in the rail (`tool_bar.gd` only, over commands that already exist); **highest-risk is the 7→3+overflow menu collapse**, because `command_index.gd` builds the searchable index by walking the live menu bar, so `MISTAKES.md`’s "move a command off the menu bar" rule applies and EXTRAS plus `shortcuts_dialog`’s UNLISTED rows must land in the same change. **A palette re-base costs every contrast pair** **Nothing left here:** the owner’s tablet decision and rulings D/E are recorded and the tablet work has shipped since; the three items only this row held (LIVE/DRAFT/ARCHIVE, the centred armed-tool chip, rail undo/redo) were moved to the open tablet row. |
| ~~**Tablet: `export_maps`’ pane BODY asks 887 px and the window needs 1176 against a declared 1024**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `TABLET_UI_SPEC.md` | small | **Declined deliberately by the PANE-MIN lane 2026-09-07 and filed with the driver named, so the red line is not misattributed to the fix that landed beside it.** Measured at `--force-touch --vp 1600x1000`. **Pre-existing, and in the BODY at touch density — not the footer**, whose claim passes there (637 + 288 <= 1024). It is the single failure in the tablet run. **The footer fix that closed the pointer-density overflow does not reach this**, because the binding term is different at touch density. **Do not assume the same remedy applies** **Fixed:** the tile-export pane stacks its two columns on tablet — `_panemin_probe` tablet 887/1 176 px → 489/812, fail 1 → 0; pointer and phone identical to HEAD. |
| ~~**`am start -n <pkg>/com.godot.game.GodotApp` throws, and our notes quote it**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `ANDROID_BUILD_SCOPE.md` | small | **Found by the verifier 2026-09-07 while reproducing a launch timing.** That activity is **not exported** — the command raises `SecurityException`. **The launchable activity is `com.godot.game.GodotAppLauncher`**; `am start -W` then reports `GodotApp` in its result line, which is very likely where the wrong name was copied from. **Cheap to fix and expensive to rediscover on a device someone else is holding** **Nothing to correct:** no note quotes the failing command — `ANDROID_BUILD_SCOPE.md` already uses `GodotAppLauncher` and warns against `.GodotApp`; `git log -S` finds only the correct note and this row. |
| ~~**Command-line args cannot be injected into a release Android build**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `ANDROID_BUILD_SCOPE.md` | small | **Measured 2026-09-07, and it constrains how any future device probe is written.** `am start --esa command_line_params "--verbose"` is accepted by `am` (it reports *"has extras"*) but arrives empty: `D/GodotActivity: Launch intent … with parameters []`. The dex **does** contain `command_line_params` and `retrieveCommandLineParamsFromLaunchIntent`, so the key is right — the launcher forwards without the extras. **This is why the logcat control had to be driven through the UI**, and it is worth knowing before anyone plans a probe build around `_cl_` flags **Recorded:** `ANDROID_BUILD_SCOPE.md` → Build and install now states the measurement (`--esa command_line_params` arrives empty) and not to plan a device probe build around command-line flags. |
| ~~**Three probes pass with their own fix removed**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `MISTAKES.md` | small | **Found 2026-09-13 by the wf53 verifier.** `_settlepix_probe` prints ALL PASS against HEAD’s 42 triangulation errors (they only reach stderr — count them in-process); `_worldportraitgrid_probe --rotate` hits a SCRIPT ERROR on HEAD code, still prints *0 FAILURE(S)* and exits 0, and a mutant removing the stage-name clip loop survives; `_worldportraitgrid_probe --force-touch` has a baseline of 2 failures (Climate), so its mutations were scored against a non-zero baseline. A probe that cannot fail is a decoration **Fixed:** `_settlepix_probe` counts triangulation errors in-process (42 with the parcel fix removed → FAIL, 0 with it); `_worldportraitgrid_probe` asserts stage-name clipping on each rotation leg (removing the clip loop → FAIL) and pins Climate 244 as a named residue, so its baselines are 0. The `--rotate` SCRIPT ERROR did not reproduce. |
| ~~**I wrote a main-loop doc during a verifier run for the THIRD time — and it was not on the allowed list**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `MISTAKES.md` | small | **Mine, 2026-09-07.** The brief named `OUTSTANDING_WORK.md` and `MISTAKES.md` as the expected exception. I wrote **`SESSION_HANDOFF.md`** at 22:23, inside the verifier’s window, recording the owner’s budget posture. The verifier caught it by md5 snapshot and reported it as drift, correctly. **Benign, and it touched nothing measured** — all six shell files were byte-identical start to finish. **But the pattern is now three for three**, and each time the fix was to widen the allowed list AFTER the fact. **The rule: name every file the main loop may write, and `SESSION_HANDOFF.md` belongs on that list** — the owner gives standing instructions mid-batch and they must be recorded when given, not queued **Rule completed** with the row above: the list the brief was missing is now in `SESSION_HANDOFF.md`. |
| ~~**I wrote a guarded file while a verifier was measuring — twice now**~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `MISTAKES.md` | small | **Mine, 2026-09-07, and the second consecutive batch.** The verifier’s brief said *"nothing else is running"*, and this time it was **my own main loop** that falsified it: `OUTSTANDING_WORK.md` was written at 18:04, inside the measurement window, filing the owner’s file-browser report. **Benign in content and no result was contaminated** — all seven measured shell files were byte-identical start to finish — but the claim was false as written for the second time, and the verifier caught it both times by snapshotting md5s. **The rule is not "stop filing rows"** — the owner reports defects mid-batch and they must be recorded. **It is: the brief must say the main loop may write the backlog docs**, and name them, so a verifier measuring drift knows which writes are expected and which are contamination **Rule completed:** `SESSION_HANDOFF.md`’s main-loop doc list now names `LARGE_ITEM_RULINGS.md` and `SESSION_HANDOFF.md` and says none is written while a verifier runs; the MISTAKES preflight row carries the ×4 instance. |
| ~~**15 menu commands unavailable**, each carrying a true reason~~ — **CLOSED 2026-09-13 (parallel runs, verified)** | `STATUS.md` | small | **Re-cut 2026-09-03, and the previous figures were wrong in both halves** — this row claimed *21 unavailable of 356 total*; the probe measures **374 total, 15 unavailable, 15 of 15 with a reason**. All 16 were opened at their symbols and **two were false**: `Clear atlas cache now` kept a build-time sentence describing what the command *does* while disabled, and `command_index.gd` reads a disabled row's tooltip as its stated reason — so the searchable index carried a description masquerading as a justification; and `No GPU detected` was minted with raw `add_item` + `set_item_disabled`, bypassing the `_todo`/`_readout`/`_signpost` vocabulary and defaulting to `_todo` when it is an empty-list placeholder. Both fixed, taking 16 → 15. The remaining 15 are genuinely blocked — mostly on the absent 3D viewport, the clipboard step of Cut/Copy/Paste, and stage groups that expose no parameters **Re-verified:** 11 unavailable today at the live location (a scratch copy shows 12 only because Help ▸ Documentation looks for the README relative to the tree), each with a true reason; no blocker had lifted. |

### From 2.6, 2.8 and 3.1 (seven-lane run, 2026-09-13)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~Hardware diagnostics panel (§23)~~ — **CLOSED 2026-09-13 (seven-lane run, verified)** | `GPU_COMPUTE_PILOT_SCOPE.md` | medium | Partly delivered by the multi-GPU work (`performance_window.gd:78`, `menus.gd:1663`); no §23 panel as specified **Declined by ruling 19, re-filed 2026-09-13:** the window this row extends, `performance_window.gd`, was deleted in `06c8963` under ruling 19 ("Diagnostics window? → NO WINDOW", `LARGE_ITEM_RULINGS.md`). The §23 items not already shown would each need a new Rust binding (no self-test, no `ComputeTier`, no vendor key). `STATUS.md` rows MEM-3, GFP-4, GGR-50 and RP-S7 still cite `performance_window.gd` and are corrected in the same change. |
| ~~**`_cull_probe` still asserts raw byte equality where `_segcull_probe` ships an AA tolerance**~~ — **CLOSED 2026-09-13 (seven-lane run, verified)** | `MISTAKES.md` | small | **Found 2026-09-13 while establishing the dashed-way residual.** `_cull_probe.gd` (2026-08-25) predates per-segment culling and asserts byte-identical frames between culled and `_NoCull` renders; its `_NoCull` twin overrides only `_visible_local_rect()`, so chaining still differs between the arms and 13 of 16 cases read FAIL although every frame is within `_segcull_probe.gd`’s `AA_AGREEMENT_MAX_DELTA = 10` / `_MAX_DIFF_FRACTION = 0.001`. Align its bar with that tolerance (or override `_segment_chains` in `_NoCull`) so a real culling defect is not hidden among 13 known-benign FAILs **Fixed:** `_cull_probe.gd` now grades culled vs unculled frames at `_segcull_probe`’s shipped tolerance — 16/16 cases pass (13 are AA-level, max delta 1), and both real culling mutants (visible-rect test inverted, a segment dropped) still fail with max delta 188. |
| ~~Five "left undetermined" questions from the unwired re-cut — light-theme inertness of the CARTO panels, the phone measure strip / label bar / way card, the 44 vs 48 dp target sweep, whether `sculpt_stroke_point` can reject an appended point, landscape composition beyond the sheet handle, and whether any `_todo` reason cites a `PARITY_AUDIT.md` section number that has moved~~ — **CLOSED 2026-09-13 (seven-lane run, verified)** | `UNWIRED_FUNCTIONS.md` | small | Three of the six need a handset or a light-theme capture, not a read **2026-09-13: a lane’s answer to the light-theme question was refuted and its probe deleted** — `_cartolight_probe` "forced LIGHT" with `DccTheme.apply_theme(false)`, which only sets `_dark` and restyles nothing, so its 1.64:1 contrast was just the palette the machine booted with. A real answer needs the theme switched through the shipped path (Preferences ▸ Theme) in an isolated user dir, with a dark control **Answered 2026-09-13 (verified):** **Q1 light theme** — switched through the shipped Preferences ▸ Theme path in an isolated copy; CARTO panel contrasts dark 3.94/15.52/8.42/3.11/11.41/5.92/8.23, light 3.11/18.00/4.52/2.64/14.97/4.90/10.22 (ghost ink on panel falls to 2.64:1 in light, the known tablet-palette pair); the panels are not inert. **Q2** — the phone has no CARTO measure-strip or label-bar screen (`phone_menu.gd::_fill_screen`), **but a Way can be committed or discarded on phone** (tool-options row in the phone sheet, auto-commit on tool switch, Escape) — the lane’s "no way to commit" was refuted. **Q3** — tap-floor sweep clean at 1080×2340 (floor 115 px; desktop-forced phone scale, not a device fact). **Q4** — `sculpt_add_point` returns −1 with no stroke and skips non-finite points; the shell ignores the return. **Q5** — landscape composition structurally sound, 207 visible controls, worst 1 919 px < 2 340 (desktop-forced). **Q6** — `engine_bridge.gd` cites PARITY_AUDIT §23 F16 as "declined" where that section says "open, unexamined"; F8 and F10 are reused ids. Q3 and Q5 join the 6T device pass. |

### From 2.7 Android and on-device verification (2026-09-20)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~Bottom-docked controls do not ride above the IME~~ — **CLOSED 2026-09-20** | `UNWIRED_FUNCTIONS.md` | small | The row’s own check (`get_virtual_keyboard_height`) named a method that does not exist on Godot 4.7.1; the real API (`DisplayServer.virtual_keyboard_get_height()`) was already wired into the shell’s own docked chrome since `fd9de7c` (2026-09-01) — half the row was already done and never recorded. The remaining gap, `DccWidgets.phone_present()` (the one shared routine ~15 phone dialog windows funnel through), fixed and independently verified with a full app boot, not just a parse check (commit `f2b0e33`); a verifier-found stale duplicate in the conformance grader `phone_protocol_grade()` fixed in the same commit. Disclosed, not fixed: the fold has no upper clamp (unreachable at any real keyboard height); `phone_menu.gd`’s L2-L5 sheets carry no keyboard term but have no focusable text field today |

### From 2.5, 2.6 and 2.7 (Ruling N river half, EF-6, PLAN subtitle; 2026-09-20)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**Settlement river binding — real geometry, not a proxy (Ruling N, river half)**~~ — **CLOSED 2026-09-20 (verified)** | `LARGE_ITEM_RULINGS.md` (Ruling N) | large | `build_settlement_suitability`’s river term now reads a `river_reach` raster (proximity to a real traced river polyline) instead of a raw flow/order sample; `build_site`’s `riverPath`-truthiness bug fixed in the same pass. Deliberate golden re-baseline (three settlement golden suites re-pinned), independently re-measured by an adversarial verifier: ground within 5 cells of a real river gains up to +0.048 suitability, the named disconnected-flow case loses up to -0.10. One shipped overclaim corrected — connectivity here is enforced by the order>=2 threshold structurally, not by the polyline itself (the coastal half has no such shortcut). Commit `649897f`. |
| ~~**EF-6 · Vector constraints beyond rivers (coastlines, faults, ridges)**~~ — **CLOSED 2026-09-20 (verified)** | `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` | medium | One new boundary-tracing primitive (`cartalith-spatial::contour`) covers coastlines and faults; a second, TPI-based tracer (`cartalith-terrain::vector`) covers ridges. Verified independently: 0 missing/0 spurious contour crossings across four fixtures; every traced ridge point above the land mean+1SD, strictly ascending, 8-neighbour steps only. Unblocks Ruling N’s coastal half. New capability, no reference equivalent. Commit `630c0dd`. |
| ~~**PLAN’s sheet subtitle goes stale when a new plan is computed**~~ — **CLOSED 2026-09-20 (verified)** | `ANDROID_UI_SPEC.md` | small | `_apply_result()` — where every recompute path (`_compute()`, `refresh_units()`) converges — now calls `app._refresh_phone_sheet_header()`. Verified with a positive control (the pre-fix file genuinely fails the row’s own scenario) in a fresh scratch copy; no regression in sibling probes. Two overclaims in the shipped comment corrected (a third, construction-only call site; "no-op" corrected to "idempotent"). Commit `554d953`. |

### From 2.7 (seed-overflow fix, committed earlier 2026-09-20, closed here)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**A seed near 2 147 483 647 panics the noise crate and the failure is silent**~~ — **CLOSED 2026-09-20 (verified; committed earlier this session, never closed here)** | `GENERATION_PARAMETERS.md` | small | Resolved as a genuine parity fix, not a disclosed deviation: `i32::wrapping_add`/`wrapping_mul` proved bit-identical to the reference’s own `(s|0)` (ECMAScript ToInt32) arithmetic, verified bit-identical to HEAD at ordinary seeds and panic-free at `i32::MAX`/`MIN`. The New World seed field is capped at `SEED_MAX = 2_000_000_000` rather than `i32::MAX`. Disclosed, not fixed: the same `seed + constant` shape exists in several other crates (named in the commit); the WGSL shaders are deliberately unwrapped (GPU work is principled-equivalence, not bit-parity); `engine_bridge.gd`’s `_worker()` never reassigns `ok` on the void-returning generate path. Commit `6482297`. |

### From 2.5 (Ruling N coastal half, 2026-09-20)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**Settlement coastal binding — real geometry, not a proxy (Ruling N, coastal half)**~~ — **CLOSED 2026-09-20 (verified)** | `LARGE_ITEM_RULINGS.md` (Ruling N) | large | `build_settlement_suitability`’s coastal term now reads `build_coast_reach` (proximity to a real traced coastline, EF-6) instead of a radius-proximity-to-any-ocean-cell proxy. `civ_is_coastal` itself (the port/sea-lane eligibility gate) deliberately untouched — Ruling N scopes to the suitability term only. Verified against two independent oracles: `build_coast_reach` matches a from-scratch coastline-crossing oracle with 0 cells differing on five worlds; a four-arm harness attributes the golden re-baseline to the coastal term alone, bit-for-bit. Three wrong causal claims in the shipped golden-test comments caught and corrected in the same commit. Commit `9ad4399`. |

### From 2.4, 2.6 (batch4, 2026-09-20)

| Item | Source | Size | Evidence |
|---|---|---|---|
| ~~**A dropped pyramid is silent — `write_project` has no warnings channel**~~ — **CLOSED 2026-09-20 (verified)** | `SAVEFILE_COMPAT.md` | small | `write_project` now returns `Result<Vec<String>, SaveError>`; a dropped stale pyramid pushes a warning naming the prefix, stale key and live key, mirroring `read_project`. No production caller sets `ProjectWrite::lod_tiles` yet, so this is behaviour-neutral today -- it opens the channel before the save path is wired. 5/5 independent mutations killed. Commit `ac9c3ad`. |
| ~~**Carry the eight deliberate re-baselines across as decisions, not as parity failures**~~ — **CLOSED 2026-09-20 (verified)** | `PARITY_TESTING.md`, `DECISIONS.md` | medium | `DECISIONS.md` §7n establishes the register; `PARITY_TESTING.md` carries the full one-line-each citations. All eight (v2.48, v2.50, v2.51, v2.57, v2.59, v2.60, v2.61, v2.49 above mapWidthKm 12800) re-opened at their `RC_ENGINE_CHANGES.md` citations and confirmed exact. Two wrong descriptions an adversarial verifier caught before landing: v2.60 had named an invented "4-connectivity finishing pass" where the actual field-moving half is §6l step 2c's sculpt-derived descent pass (reverted by v2.61, per §6m.4 a port that has not implemented it should not); v2.58's exclusion overclaimed that golden testing cannot see it at all, narrowed to "no value a golden fixture currently asserts can fail on it". No code touched. Commit `1cc7cb6`. |
