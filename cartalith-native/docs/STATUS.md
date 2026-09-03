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

**Phase.** Phases 0, 1, 2 and 4 are complete. **Phase 3 (rendering) and Phase 5
(urban morphology) are both in progress**, and they are the only two phases with
work outstanding.

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
- **Phase 5** — **moved substantially on 2026-09-02 and this paragraph was
  rewritten with it.** Milestones 1-7 and 12 were complete, with 8a and 17a out
  of order and 17 at 13 of its 20 adapter functions. **Milestones 8, 9, 10, 11,
  13, 14 and 15 now have code**: `crates/cartalith-urban/src/lib.rs` declares
  sixteen modules, not the ten it declared the day before — `radial` (m8, 320
  lines), `water` (m9, 693), `fortify` (m10, 1 288), `cleanup` (m11, 645),
  `districts` (m13, 1 307), `amenities` (m14, 758) and `hinterland` (m15, ~1 054
  lines) joined `astar`, `blocks`, `geom`, `graph`, `growth`, `plaza`, `rng`,
  `routes`, `rules` and `site`.
  `cargo test -p cartalith-urban` went **119 → 258 passed, 0 failed**.
  **Still open: milestone 16** (blocked by definition on 8-15) and the rest of 17.

  **Read that as "code exists", not "milestone done".** Milestones 9, 10 and 13
  were ported by agents that died before reporting, so their claims have never
  been checked by anything but the compiler and their own tests — the largest
  two, 10 and 13, most of all. A verification pass is in flight. Until it
  reports, the honest status of 9/10/13 is *ported, unreviewed*.

| Phase | `ROADMAP.md` says | This file says | The one thing to know |
|---|---|---|---|
| **0** — walking skeleton | done | **done\*** | The `.exe` and `.apk` exist and the extension loads. `ROADMAP.md` says "all three targets"; `export_presets.cfg` defines **two** — Windows Desktop and Android. The third is WASM, which the same file elsewhere calls uncommitted |
| **1** — terrain MVP | done | **done** | All seven `MVP_SCOPE.md` criteria; the ocean-current stretch goal shipped too and was never recorded either way |
| **2** — civilisation layer | done | **done** | All 21 milestones, plus the Journey Planner sub-phase engine-complete at 66 of 74 `jp*` functions |
| **3** — rendering and 3D | partial | **partial** | 2D done, 3D absent and parked |
| **4** — Asset Library | done | **done** | Eight milestones, not the seven `ROADMAP.md` counts — the slicer landed 2026-08-20 |
| **5** — urban morphology | in progress | **in progress** | The largest outstanding block; see above |
| *not a phase* — LOD and large worlds | "revisit when a concrete need appears rather than building it speculatively" | **built and shipping** | A tiled deep-zoom pyramid with a persistent chunk atlas is on screen. `ROADMAP.md` has not been told |

**What landed most recently** (full week in *The last seven days* below):

1. **2026-09-02** — **the landmark ("point of interest") pass, reported broken
   by the owner, root-caused to three defects and fixed**; nine backlog rows
   closed across memory, Rayon and economy; and **Vulkan / DirectX /
   `RenderingDevice` answered by measurement — all three "no"**. The renderer
   answer is not caution: on the owner's RX 7800 XT, `forward_plus`/vulkan
   loses the GPU device during generate **3 runs of 3** (`VK_ERROR_DEVICE_LOST`),
   `forward_plus`/d3d12 segfaults (`DXGI_ERROR_DEVICE_REMOVED`), and
   `gl_compatibility` is clean. Four verified defects were found in passing
   that nobody was looking for — `use_gpu` forced on over a `false` default so
   **the shipped app does not generate the world the 88 golden suites verify**;
   a software-rasterizer fallback the code's own comment denies; no `log`
   backend anywhere, which makes the Android "zero wgpu lines in logcat" PASS
   condition unfalsifiable; and LOD tiles in the route-map cutout registered
   half a world cell off. Detail in *2026-09-02* below. **Uncommitted.**
   **Later the same day**, two further waves of five agents each, every engine
   lane adversarially verified: **urban 17a golden-verified** (UM-17A-G above —
   the blocker was wrong rather than stale, and two real port bugs fell out of
   it, including an `f64::hypot` that should have been `js_hypot`); **landmark
   M8's five way-graph kinds** took `kinds()` from 15 buildable to **20 of 50**,
   each verified placing on a real `generate_terrain` world rather than by
   flipping a flag, and `JUNCTION_MIN_WAYS` corrected 3 → 2 with its inherited
   rationale shown false; `TERRAIN_APPEARANCE_SCOPE.md` **§16** (multi-scale
   detail: `detail_macro/meso/micro_weight`, defaulting to the previously
   hardcoded 0.40/0.40/0.20 so `golden_parity_render.rs` is untouched) and
   **§19** (`atmo_desaturation`, `atmo_contrast`, both defaulting `0.0`);
   `UNWIRED_FUNCTIONS.md` re-cut a third time — **0 of 21 closed, which is the
   finding** — plus one new dangerous-class entry, the Settlement diagnostics
   overlay's tooltip citing a blocker that no longer holds. And the **"no JS
   runtime in this environment" blocker was found false and swept**: `node`
   v24.19.0 runs the frozen reference, proved two ways by
   `tools/jsruntime_probe.js`, and everything the claim gated had shipped on
   2026-08-15. `cargo test --workspace` **2 751 passed, 0 failed, 21 ignored**
   (floor was 2 734). **Uncommitted.**
2. **2026-09-01** — `OUTSTANDING_WORK.md` §1's eight in-flight items worked in
   parallel and independently re-verified against the code, not the reports:
   `UNWIRED_FUNCTIONS.md` re-cut from scratch (75 open rows → 23, dangerous
   class 25 → 3); `UNIFIED_TOOL_PLAN.md` got its "Milestone F as built"
   section; Vault §14 Compare shipped; route corridors/travel cost became a
   selectable analysis field; one more landmark kind went buildable (15 of 50 —
   *this entry read "14 of 49" until 2026-09-02; both figures were wrong, the
   denominator is `grep -c "KindSpec {"` = 50*);
   `civ_food_shed` was built, completing Economy milestone 2 at the crate level
   (Godot wiring still open); WORLD and CIVIL's Landmarks/Factions categories
   were restyled to the new left-dock spec; two real `statusMid` bugs were
   fixed. **Same-day second pass**, three of that morning's residuals
   independently re-verified against the code (not the agent reports) and
   advanced: **Paint brush falloff shipped** — `PaintStamp::with_falloff`
   wired end to end, bit-identical to the old hard disc at its default,
   closing the highest-severity `UNWIRED_FUNCTIONS.md` row and both remaining
   dangerous-class entries (3 → 0); **Economy milestone 2 reached Godot** —
   `civ_food_shed` `#[func]`, `engine_bridge.gd`, `trade_store.gd` all real
   and triggered by the existing "Match trade flows" button, leaving only a
   UI display as the gap; **GUI replacement stage 4 closed** — CIVIL's Ways &
   routes gained a live `ROUTES` teaser list, and Journey planner was
   deliberately restyled (a thin honest summary, not an embed) rather than
   left undone. **Third pass, same day**: a tectonics World-Structure
   override-disclosure bug the owner found by manual testing was fixed (the
   three overridden parameter rows now visibly disable, live, on toggle,
   generate and load); and all six of `OUTSTANDING_WORK.md` §2.3's
   journey/route cluster rows closed — four built (ocean/wind fields
   reaching the sea-lane router and `jp_sea_condition`, `_civSeaTimeEdgeCost`,
   `jp_road_cells` seeing hand-drawn ways, `DECISIONS.md` §7i's swamp/ford
   terms) and two confirmed already done. **All four Journey Planner quality
   ceilings are now closed.** Detail in each affected ledger row above and in
   *2026-09-01* below. **Still entirely uncommitted.**
3. **2026-08-31** — GUI replacement **stages 1 and 2** (`c03b43c`): the new
   token system, and the rail folding five domains into three
   (`dcc_shell.gd`'s `DOMAINS` now holds exactly world / civilization /
   cartography; `RAIL_NODES` holds 3 heads and 10 nodes — counted in the file).
4. **2026-08-30** — **landmark generation, end to end** (`a6feec3`,
   `ae62adf`, `f084650`): `crates/cartalith-civ/src/landmark.rs`,
   `crates/cartalith-terrain/src/analysis.rs`, `landmark_bridge.rs`, ten
   `#[func]`s, 49 glyphs, and a CIVIL ▸ Landmarks panel. **13 of 49 declared
   kinds generated that day** (14 of 49 as of 2026-09-01 — see the Landmark
   generation ledger below); the rest each carry a `not_built:` reason in
   source.

**The next three things.** These are the three with the most work behind them
and no blocker; the full list is `OUTSTANDING_WORK.md`.

1. **~~Commit the working tree.~~ Done — and this row was wrong for a day.**
   Corrected 2026-09-02. It claimed **132 tracked files / 17 576 insertions**
   uncommitted and that `LARGE_ITEM_RULINGS.md`, `OUTSTANDING_WORK.md` and
   `cartalith-native/docs/3D_TERRAIN_RENDER_RESEARCH.md` "exist only in the
   working tree", so "a clean checkout loses all three". **All three are
   tracked in `HEAD`** (`git cat-file -e HEAD:<path>` for each), and commit
   `fd9de7c` ("Three rounds finishing in-flight work, then two bugs found by
   hand") landed **237 files / 90 718 insertions**. The row survived its own
   resolution because nothing re-checked it — the exact failure this file
   exists to prevent, committed by this file. What *is* uncommitted today is
   the 2026-09-02 work below. **The live successor task is to commit that**,
   not to re-do this.
2. **Urban morphology milestones 8-16 and the rest of 17**
   (`URBAN_MORPHOLOGY_SCOPE.md`). ~28 reference functions, ~1 500 lines,
   nothing started, nothing blocking. Milestone 10 (fortification) alone is
   nine functions and the plan's self-declared largest.
3. **GUI replacement stages 3, 5, 6 and 7**
   (`design/dcc-environment-2026-08-31/spec/00-REPLACEMENT-PLAN.md`). Stages
   1, 2 and 4 landed; **stages 3, 5, 6 and 7 are unblocked and unstarted**.
   The plan itself still says stage 5 is *blocked* on a truncated prototype —
   that blocker was cleared the same day the plan was written (the file in
   the tree is 239 712 bytes and ends `</script></body></html>`), and the
   plan was never updated. Anyone reading only the plan will skip real work.

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
| **shelved** | Built or buildable, and stopped by the owner. Only `EXPORT_SCOPE.md`'s five rows carry this |
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

Dated, because this is what a returning session needs and it is exactly what
went missing from the old file. Commits are from `git log`; each claim below was
re-checked against the tree rather than copied from the commit message.

### 2026-09-02

Four parallel workflows (33 agents). Every claim below was re-verified against
the code by an agent that did not make it, and then re-run once more by hand
before being written here — `cargo test -p cartalith-civ -p cartalith-terrain
-p cartalith-engine -p cartalith-godot` aggregates **1 543 passed, 0 failed,
21 ignored** against a `cartalith_godot.dll` confirmed newer than every touched
`.rs`, and all seven touched `.gd` files are `--headless --check-only` clean.
**All of it is uncommitted.**

**The landmark / point-of-interest pass, reported broken by the owner** ("seems
to make the program freeze and doesn't render on the map"). Two symptoms, three
causes:

1. **`landmark_run()` was synchronous on Godot's main thread.** Measured before
   the fix by `_poifreeze_probe.tscn`: **0 main-loop frames served** during a
   1 224.9 ms pass, against 255 served during a `generate()` doing four times
   the work — because `engine_bridge.gd` runs *that* on a `Thread`. The landmark
   path never got the same treatment, and both its own doc comment and the run
   button's tooltip said so outright.
2. **Nothing pushed the placements at the map.** `MapOverlay._landmarks`' only
   writer in the entire shell is `ViewportHost.refresh_annotations()`, and
   `civilization_workspace.gd::_lm_run()` never called it — nor did
   `ViewportHost.refresh()`, so a regenerate also left world A's rings drawn
   over world B. Baseline: `overlay after the UI run: _landmarks=0 (engine has 239)`.
3. **The one nobody predicted: a `#[func]` that builds a `Dictionary` cannot be
   called from a worker thread.** The first fix simply moved `landmark_run` onto
   a `Thread` and produced
   `attempted to access binding from different thread than main thread; this is UB`
   out of `godot-ffi-0.5.5/src/binding/single_threaded.rs`. Without the
   `experimental-threads` feature — and `crates/cartalith-godot/Cargo.toml` pins
   `godot = "0.5.5"` with only `features = ["api-4-7"]` — every `Dictionary`,
   `Array` and `GString` operation routes through `ensure_main_thread()`.
   `generate_sized` has been thread-safe all along **only because it takes and
   returns primitives.** That forces the shape of the fix and is the reusable
   lesson: *a worker-thread `#[func]` must be primitives-in, primitive-out.*

   Fixed accordingly: `landmark_run` now returns `bool` with the reason in a
   plain `String` field, a new `#[func] landmark_last_run()` builds the
   `{ok, placed, seconds, error, funnels}` reply on the main thread,
   `engine_bridge.gd` reuses `generate()`'s exact `Thread` →
   `call_deferred` → signal pattern (reusing `generating` and `_thread`, but a
   *new* `landmark_finished` signal — 30-odd listeners read
   `generation_finished` as "the world was replaced"), and
   `viewport_host.gd` connects the push in `setup()` so a caller cannot forget
   it. Separately, `box_h`/`box_v` (`cartalith-terrain/src/analysis.rs`) and
   both halves of `sep_min_max` were parallelised over **output rows**
   (`par_chunks_mut(gw)`) — each cell's own accumulation runs in exactly the
   order it always did, so this is bit-identical, not a float reordering.
   **Measured at the shipping 2048×1311 default: 4.14 s → 0.39-0.86 s**, and
   off the main thread. `_poifreeze_probe.tscn` is the committed regression
   check: `fails=0`, 23 frames served during the pass, overlay count == engine
   count, and a regenerate clears the rings.

**Vulkan, DirectX and `RenderingDevice` — all three answered "no", the first by
measurement.** Driving the committed `_shot.tscn` harness on the owner's RX
7800 XT (driver 26.7.1, Godot 4.7.1) via `--rendering-method`/`--rendering-driver`
launch flags, so **no file was edited to produce the table**: `gl_compatibility`
boots *and generates* clean; `forward_plus`/vulkan loses the device during
generate **3 of 3** (`VK_ERROR_DEVICE_LOST`, signal 4); `forward_plus`/d3d12
segfaults (`DXGI_ERROR_DEVICE_REMOVED`); `mobile`/vulkan matches Vulkan. Boot is
clean on all of them — it is the *generate* that kills the device.
`RenderingDevice` is separately disqualified: null under `gl_compatibility`
(**both** `get_rendering_device()` and `create_local_rendering_device()`) and
null under `--headless`, which would delete the 68 `cartalith-gpu` tests with no
CI-shaped replacement. DirectX needs no work at all — `COMPUTE_BACKENDS` already
unions `DX12` and masks out only OpenGL (itself a bisected signal-11 fix), and
`backend_rank`'s Vulkan-first order turns out to restate wgpu-core 30's own HAL
registration order. 178 lines appended to `3D_TERRAIN_RENDER_RESEARCH.md`, zero
deletions, **3D left parked**.

**Four defects found while looking for something else**, each verified:

- **`engine_bridge.gd` forces `param_set("use_gpu", true)` at boot**, over a
  `WorldParams::defaults()` of `use_gpu: false` whose own comment says the GPU
  path "produces a different" world. **So the shipped app does not generate the
  world the 88 `golden_parity_*.rs` files verify.** Worse, the default grid is
  2 684 928 cells — *below* 2048², inside the band where
  `GPU_LAYER_INTEGRATION_SCOPE.md` m6 records "GPU loses". Untested and possibly
  slower. **Not fixed: this is a product default, an owner call.**
- **`multi.rs`'s `is_software` doc comment is wrong.** It says a software
  rasterizer is "never selected by default … every `request_adapter` in this
  crate already passes `force_fallback_adapter: false`". That flag means
  *restrict to* fallback adapters; `false` merely declines to restrict, and
  nothing filters `DeviceType::Cpu`. On a box with no working hardware adapter
  the pipeline opens Microsoft Basic Render Driver and runs on it. **Not fixed.**
- **No `log` backend is registered anywhere in the workspace**, so wgpu's
  logging is a runtime no-op on every platform. The Android passes' PASS
  condition — zero `wgpu` lines in logcat — **cannot fail**, and every "the
  handset runs pure CPU" claim rests on it. wgpu, wgpu-hal and ash *are*
  compiled into the shipped arm64 `.so`; there is no `cfg(target_os = "android")`
  gate; GPU is forced on at boot. §21 is unblocked by one 60-second device
  readout, not by a renderer migration. **Not fixed.**
- **The route-map cutout placed LOD tiles half a world cell off the colour they
  multiply** — a registration error that scales with zoom, live in this
  session's own in-flight work while its probe reported green. The probe
  asserted UVs lay in `0…1` but never that a tile's UV footprint agreed with
  where the sprite was placed. *Ranges are not registration.* Fixed, with two
  smaller defects beside it.

**Nine backlog rows closed** (`OUTSTANDING_WORK.md` §2.3/§2.6), each
golden-verified: Rayon across `road_dijkstra`'s three independent source maps
(ordering **proved by mutation** — reversing collection order fails four golden
tests, so the guarantee is tested, not argued); R8 (~45 MiB, by probe reduction
and early release — **the scope document's prescribed "chunk it" mechanism is
impossible**, since Prim reads an arbitrary source's result until the pass ends;
the saving is real, the named mechanism is not); R7 (`want_prev`, 10.24 MiB);
R5 (`jfa_dist` → i32/i32/u32, 32.2 MiB, bit-identity **proved by mutation** —
`dd + 1` fails `golden_parity_settlement_prereqs` 3/3); R4 (`plate_id` → `u16`,
15.36 MiB); `_civPlaceSmelting` and `_civSaltAccess` ported with a new
`golden_parity_smelting_salt` suite; the food-shed readout surfaced in the place
editor's Trade tab; and the Nortantis disclosure added to `credits.gd`.

**Documented-but-false claims corrected in place**, beyond the commit row above:
`DECISIONS.md` §7i, `JOURNEY_PLANNER_SCOPE.md`'s 2026-08-19 update (both by
dated correction, not silent rewrite), `world_workspace.gd`'s "58 parameters"
(really 81), the `paint_set_brush` doc comment, and `roster.rs`'s food-shed
self-claim. Also: **the GPU determinism flake is not open** — filed as
blocked-on-owner in four places, but `803b725` (2026-08-25) replaced the
`assert_eq!` with a 1e-6 worst-element tolerance. And **a `gl_compatibility`
rationale does exist**, in `.claude/skills/godot-shell/SKILL.md`, committed
alongside `project.godot` with its cost and a revisit trigger — four of five
investigators reported it as never recorded.

### 2026-09-01

Eight agents worked `OUTSTANDING_WORK.md` §1's eight in-flight rows in
parallel; a ninth pass verified each claim against the code before recording it
here — `cargo test -p cartalith-civ --lib` (513 passed, 0 failed) and
`cargo test -p cartalith-godot --lib` (406 passed, 0 failed, 6 ignored) both
re-run clean after a fresh `cargo build -p cartalith-godot` (the dll was stale
against `trade.rs`/`timeline.rs`/`roster.rs` before that rebuild — this
project's own recorded hazard, caught rather than repeated), and the five
touched `.gd` files were `--headless --check-only`-clean. **All of it is still
uncommitted** — it lands on top of the same working tree the rest of this
section describes.

- **`UNWIRED_FUNCTIONS.md` re-cut from scratch**, not patched: every one of the
  75 rows open at the 2026-08-31 cut was re-opened at its cited symbol and
  independently re-verified. 52 closed (all 17 trivial, 24 of 25 small, 13 of
  17 medium); 23 remain (1 small, 4 medium, 18 large — the 18 are exactly
  `LARGE_ITEM_RULINGS.md`'s 2026-08-31 **build** rulings, not a fresh gap). The
  dangerous class fell from 25 entries to 3. *(Both numbers move again the
  same day — see "Second pass, same day" below: 22 open, 1 dangerous; then
  "Third pass, same day" further below closed one more medium row: 21 open,
  1 dangerous.)* Two real bugs were fixed along the
  way in `app.gd`'s `statusMid` composite: stage names now truncate at `" &"`
  to match `BUILD_ANSWERS.md` §2.2's fixed string (`09 Ecology`, not
  `09 Ecology & biomes`), and a regenerate no longer shows a false
  self-contradictory "loaded — no generation this session" beside a `pass`
  slot reading "generating…" (`_refresh_status_mid` now gates both the ms
  figure and the loaded branch on `not bridge.generating`). `repaint NN ms`
  remains genuinely absent, disclosed in place, blocked on owner question 2.
- **`UNIFIED_TOOL_PLAN.md`'s "Milestone F as built" section written** — the
  gap `OUTSTANDING_WORK.md` §1 named. Enumerates all sixteen
  `STRANDED_TOOLS.md` tools against the code; `STRANDED_TOOLS.md`'s stale "44
  methods… not one wired" claim annotated false, dated, in place. See the Tool
  system ledger below.
- **Vault §14 Compare-with-source shipped** (`vault_window.gd`) — the diff
  view §14's three-way prompt was missing. Dynamically verified: opening
  Compare cannot itself clear a Stale status (it reads via
  `vault_preview_section_write`, never `vault_reload_link`). See the Markdown
  Vault ledger below (MV-5).
- **Route corridors / travel cost shipped as a selectable analysis field**
  (`sample_bridge.rs`) — `GUI_FEATURE_PARITY_SCOPE.md`'s last open item.
  Closes with two fixture tests proving the ramp is actually reached, not just
  non-empty. See the GUI feature parity ledger below (GFP-4).
- **Landmark `resource_extraction_site` went buildable** (`landmark.rs`) — 14
  of 49 kinds now generate (was 13). Reads `timber`/`sulfur`/`alum`, the three
  resource-potential fields Mine and Quarry don't claim, through their own
  already-validated detector. The other 35 `not_built` reasons were
  individually re-verified, six rewritten for precision with no change to
  their conclusion. See the Landmark generation ledger below (LM-8).
- **`civ_food_shed` built** (`trade.rs`) — a direct port of `_civFoodShed`,
  closing `ECONOMY_SCOPE.md` milestone 2 at the crate level. Distinct from
  `trade.rs`'s pre-existing 15-good trade match, which excludes `food` and so
  cannot substitute for it. Not yet reachable from Godot — no `#[func]` calls
  it. See the Economy ledger below (EC-3).
- **WORLD and CIVIL's Landmarks/Factions & settlements restyled** against
  `04-left-dock.md` (`world_workspace.gd`, `civilization_workspace.gd`) — a
  deliberate restyle rather than a rebuild, keeping the shipped
  one-accordion-per-domain model. CIVIL's Ways & routes and Journey planner
  remain untouched, owned by `infrastructure_workspace.gd`. See the GUI
  replacement ledger below (RP-S4).

**Second pass, same day.** Three of the morning's own residuals dispatched
and independently re-verified against the code before being recorded here —
not carried forward from the agent reports. `cargo test -p cartalith-spatial
--lib` (148/148), `--test golden_parity_paint` (7/7), `cargo test -p
cartalith-godot --lib` (409/409, 6 pre-existing ignores) and `cargo test -p
cartalith-civ --lib` (513/513) all re-run clean after a fresh `cargo build -p
cartalith-godot` (the dll was stale against exactly the files this pass
touched — this project's own recorded hazard, caught rather than repeated);
`cargo check --workspace` clean; every touched `.gd` file `--headless
--check-only` clean; `_railfold_probe.tscn` **PASS** and `_deadwire_probe.tscn`
**DONE fail=0** both re-run.

- **Paint brush falloff shipped**, closing the highest-severity
  `UNWIRED_FUNCTIONS.md` row and both remaining dangerous-class entries (3 →
  0; see the dangerous-class section there). `PaintStamp::hardness`/`softness`
  (`cartalith-spatial/src/paint.rs:143-144`), the `with_falloff` builder
  (`:180`), `feather_width` (`:199`), `passes_falloff` (`:219`) and
  `cell_dither` (`:249`) implement a deterministic probability-threshold edge
  band — never a blended palette index, the categorical-blending objection the
  reference itself raises — wired end to end from `Brush::hardness`/`softness`
  through `PaintEditor::stroke_at` (`paint_bridge.rs:466`), bit-identical to
  the old hard disc at `hardness=1.0, softness=0.0` (the golden-parity paint
  suite is unchanged, 7/7). The duplicate slider is resolved: `tool_bar.gd`'s
  copy is deleted (`:433-442`, comment explains why), `world_workspace.gd`'s
  survives with tooltips naming the real mechanism (`:2103,2105`). Recorded as
  a deliberate divergence from the reference at `DECISIONS.md` §7k, per the
  owner's ruling. **One disclosed residual, not closed this pass:**
  `lib.rs:6704-6705`'s `paint_set_brush` doc comment still says hardness/
  softness are "never consumed" — confirmed still present and now stale.
- **Economy milestone 2 reached Godot** (`civ_trade_bridge.rs`,
  `engine_bridge.gd`, `trade_store.gd`) — see EC-3 below for the full chain.
  The private `food_shed_rows()` builds one shared `RoadComponents` and calls
  `civ_food_shed` once per settlement, resolving `farmers_per_urbanite`
  through the same `civ_ag_tech_by_key` route the manpower model already
  uses; the `#[func] civ_food_shed` reads it out; the shell caches it
  alongside `civ_trade_flows`, populated by the existing "Match trade flows"
  button with no new UI trigger needed. **What remains is a UI surface, not a
  binding**: no dock reads `food_shed_for(index)` yet — confirmed,
  `place_editor_window.gd:385` still reads only `navigability`.
- **GUI replacement stage 4 closed** (`infrastructure_workspace.gd`) — see
  RP-S4 below. CIVIL's Ways & routes gained a real `ROUTES` teaser list
  (glyph, name, nearest settlements, km, click-to-plan); Journey planner was
  deliberately restyled — a thin, honest summary plus the one shared
  `open_journey_planner()` entry point — rather than embedding a second copy
  of `journey_planner_view.gd`'s private form state. One disclosed gap: no
  per-route preselect into the planner.

**Third pass, same day.** Two more items dispatched and independently
re-verified against the code before being recorded here — not carried
forward from the agent reports. `cargo check --workspace` clean; `cargo test
-p cartalith-civ` (all targets — 518 lib tests plus every `tests/*.rs`,
including the two touched golden-parity suites) and `cargo test -p
cartalith-godot` (all targets — 409/409 lib, 6 pre-existing ignores) both
re-run clean; `cargo clippy` on both crates re-run, no warning at any touched
symbol (the two collapsible-if fixes and the one
`#[allow(clippy::too_many_arguments)]` on `civ_sea_routes` hold);
`target/debug/cartalith_godot.dll` confirmed newer (13:13:05) than the newest
touched `.rs` file (`cartalith-civ/src/lib.rs`, 13:11:27) — this project's own
recorded hazard, checked rather than assumed; `world_workspace.gd` re-run
`--headless --check-only` clean.

- **A new defect, found by the owner's own manual testing rather than any
  audit document, closed: the three tectonics/volcanism parameter rows kept
  accepting drag input and regenerating from it while World Structure
  silently overrode the result.** `generate_terrain_inner`
  (`cartalith-engine/src/lib.rs:676-684`) has always replaced
  `p.tect.plates`/`p.tect.vel`/`p.volc.count` with
  `deriveFromWorldStructure()`'s own archetype-derived values whenever
  `p.world_structure.enabled` (default `false`) — real, and matching the
  reference's own comment exactly — but nothing in `world_workspace.gd`
  disclosed it, so a player could drag "Plates" for as long as they liked and
  never see it do anything. Fixed at build time (`_build_param_row`) and live
  (`_refresh_ws_override_rows`, wired from the toggle's own change handler,
  from `_on_generation_finished` — covering a File ▸ New world preset that
  turns World Structure on and then generates — and from `_on_world_loaded`
  — covering a save that already carries `enabled: true`): the three rows
  (`WS_OVERRIDDEN_KEYS`, `world_workspace.gd:57`) go `editable = false`, the
  whole row dims to 55% (`WS_OVERRIDE_DIM`), and the tooltip gains an
  explanatory prefix (`WS_OVERRIDE_REASON`, `:65`) whenever the toggle is on.
  The right-click "reset to default" path was generalised to read the row's
  own live `editable` state rather than a captured snapshot, closing a
  loophole where it could silently revert a disabled row without saying so.
  **Two disclosed, out-of-scope findings surfaced along the way, neither
  touched:** no other parameter row in this panel — of `params.rs`'s 81 —
  resyncs its displayed value after a load- or preset-driven generate at
  all, so these three are now the only ones that do; and the panel's own
  header (`world_workspace.gd:10`) still says "58 parameters" against the
  real count of 81 (`grep -c 'ParamSpec { key:'
  crates/cartalith-godot/src/params.rs`).
- **The journey/route dead-control cluster — all six `OUTSTANDING_WORK.md`
  §2.3 rows closed**, four built and two confirmed already done (stale rows,
  now deleted from that document). Three map onto declared Journey Planner
  quality ceilings — see JP-QC2/QC3/QC4 above, now all `done`. The fourth
  built item has no scope-document milestone ID of its own:
  `DECISIONS.md` §7i's swamp/floodplain penalty and river ford-vs-bridge
  cost, named there as "the obvious next step" and left unbuilt at the time.
  `RouteContext` (`cartalith-civ/src/tools.rs:352`) gained `flow`/
  `flow_thresh`; `civ_land_cost_grid`/`civ_mixed_cost_grid` (`tools.rs:564`,
  `:637`) now apply `civ_swamp_penalty`/`civ_river_crossing_cost`
  (`cartalith-civ/src/lib.rs:5583`, `:5599`) — the same two functions
  `civ_enhanced_travel_cost` itself already calls, so the formula cannot
  drift between the auto-populate road builder and the manual Route/Way
  tools. Wired at all three `RouteContext` construction sites in
  `cartalith-godot/src/lib.rs`: `way_commit` (`:7115`), `route_commit`
  (`:7204`), and `jp_reroute` (`:9871`) — the last of which is why this is
  recorded as a Journey Planner change and not only a Route/Way-tools one.
  The two confirmed-stale rows, independently re-verified rather than taken
  on trust: "wire wildlife richness into `jp_foraging`" was already done
  (`wildlife.rs` shipped 2026-08-23; this is JP-QC1 above); "`road_edges`
  not retained" was already done too — `CivData::road_edges` genuinely
  retains `civ_hierarchical_network_topology`'s output (a different producer
  than the never-called `build_road_network` the stale claim named) and both
  `jp_road_cells` call sites already used it, so only `journey_bridge.rs`'s
  own module doc was wrong, now corrected in place. **Verifying this pass
  found the identical stale claim in two more places, both now closed the
  same way**: `UNWIRED_FUNCTIONS.md`'s "Manual road tool / `road_edges` never
  retained" (Medium; 22 open rows there becomes 21 — see that document's own
  third-pass note), and `OUTSTANDING_WORK.md` §3.2's row of the same name,
  which cited it as a live blocker (34 blocked becomes 33). Neither was in
  either agent's original report; both are the same underlying fact recorded
  twice more. Five new tests:
  `civ_swamp_penalty_and_river_crossing_cost_match_the_reference_formula`,
  `civ_sea_time_edge_cost_is_none_without_any_field_and_penalises_a_current_aligned_edge`,
  `civ_sea_routes_still_connects_ports_with_or_without_current_and_wind_fields`,
  `jp_road_cells_reads_hand_drawn_ways_including_ancient`
  (`cartalith-civ/src/lib.rs`), and
  `swamp_and_ford_terms_scale_with_the_flow_field_and_touch_nothing_else`
  (`tools.rs`).

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
| In flight — code exists, uncommitted or partial | 3 | `OUTSTANDING_WORK.md` §1 |
| Ready to start — nothing blocks them | 99 | §2 |
| Blocked — a named blocker | 33 | §3 |
| Open owner decisions — not work yet | 20 | §4 |
| Declined / shelved — kept so nobody re-proposes them | 23 entries, 3 groups | §5 |

**155 outstanding items** (was 168 that morning, then 164 after four of §1's
eight rows closed outright 2026-09-01: Milestone F's closeout, the
`statusMid` composite, Vault §14 Compare, route corridors/travel cost as an
analysis field; then 162 after the same-day second pass closed GUI
replacement stage 4 outright and Paint brush falloff inside §2.2; then 156
after the same-day third pass closed all six of §2.3's journey/route cluster
rows — four built, two confirmed already done; then 155 once verifying that
pass closed §3.2's "Manual road tool" row too — it blocked on the identical
now-false claim `UNWIRED_FUNCTIONS.md`'s matching row made). Of the 140 that
carry a size: **42 large, 56 medium, 42 small**. **14 of the 33 blocked are
blocked on an owner decision and nothing else** — the largest single category
of stalled work in the project.

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

Twenty, in full in `OUTSTANDING_WORK.md` §4. The six that gate the most:

| # | Question | Gates |
|---|---|---|
| 1 | **What is a conflict attached to** — free geometry, or a reference to a settlement/province? | Story planning SP-4, and through it landmark M9. The highest-leverage unanswered question in the project: two documents' largest remaining milestones sit behind one unasked question |
| 2 | **The viewshed cost budget** — observer count, radius cap, grid resolution | Landmark M7 and six of the 49 landmark kinds. `needs_viewshed: true` already ships on six specs with nothing behind it |
| 3 | **Regenerate semantics for a journey's route polyline** — invalidated, re-snapped, or kept with a staleness mark? | Story planning SP-2, explicitly |
| 4 | **Does the landmark set live in the save tree, or regenerate on load?** | The record's shape. Storage is in memory today and the save format is untouched |
| 5 | **Does a landmark become a `cartalith_vault::EntityKind`?** | A `Landmark` template exists in `design/vault-templates/` and `template.rs` recognises it; `links.rs` has no variant |
| 6 | **Does `DECISIONS.md` §7a/§7d's parity contract apply to landmarks at all?** | `FUNCTION_INDEX.md` returns nothing for "landmark", so there is nothing to match against. `landmark.rs` was built assuming divergence-by-addition and no ruling is recorded |

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
| MVP-1 | Criterion 1 — height / temperature / rainfall / flow match golden data at a fixed seed | done | `cartalith-terrain/tests/golden_parity_height.rs`; `cartalith-climate/tests/golden_parity_temperature.rs` and `golden_parity_weather.rs`; `cartalith-hydrology/tests/golden_parity_flow.rs`; `cartalith-engine/tests/golden_parity_pipeline.rs` for the assembled run. 88 `golden_parity_*.rs` files exist across the workspace |
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
| P2-08 | 8 — settlement placement + faction assignment | done | `cartalith_civ::label_land_components`, the crate-private `civ_snap_land` / `civ_snap_coast` / `civ_is_coastal`, `assign_landmass_factions`, and the public `place_settlements_with_water_edge_snap`. Unit tests pin the snap quirks |
| P2-09i | 9 (first, renumbered) — investigation: territory/provinces is a dead end in the reference | unverified | A research finding about the reference HTML, not a deliverable. Superseded in-document by milestone 10; the 2026-08-19 correction notice is part of the record. Nothing in code to check |
| P2-09 | 9 — settlement population + naming | done | `cartalith_civ::{civ_default_culture, civ_name_rng, civ_settle_name, civ_base_pop_for_kind, name_and_populate_settlements}`, over `cartalith_rng::Mulberry32` |
| P2-10 | 10 — territory (cost-distance Voronoi from capitals, `DECISIONS.md` §7b) | done | `cartalith_civ::assign_territory`, reusing `road_dijkstra` and `build_travel_cost`. `engine_bridge.gd` exposes a territory texture; `viewport_host.gd` draws it beside `province_view` |
| P2-11 | 11 — road network (`buildTravelCost` / `roadDijkstra` / `buildRoadNetwork`) | done | `cartalith_civ::{build_travel_cost, build_road_network}` and crate-private `road_dijkstra`, with unit tests `road_dijkstra_flat_grid_diagonal_uses_sqrt2` and `road_dijkstra_impassable_water_stays_unreachable` |
| P2-12 | 12 — civ auto-populate road network (`_civHierarchicalNetwork`) | done | `cartalith_civ::civ_hierarchical_network_topology`; `tests/golden_parity_hierarchical_network.rs` |
| P2-13 | 13 — sea routes (`_civMstRoutes`) | done | `cartalith_civ::civ_sea_routes` → `WorldGen::get_sea_routes` → `engine_bridge.gd::sea_routes()` → **drawn** at `viewport_host.gd:1152` and in `civilization_workspace.gd`; also consumed by `place_search.gd` and `infrastructure_workspace.gd`. *The scope document's "not yet wired into rendering" is stale.* **Current/wind-costed as of 2026-09-01** — see JP-QC3 below |
| P2-14 | 14 — corridor consolidation + path smoothing | done | `cartalith_civ::civ_consolidate_and_smooth_ways`; `tests/golden_parity_road_consolidation.rs`. Output reaches the map through `get_roads` / `engine_bridge.gd`'s `roads()` |
| P2-15 | 15 — village seeding (`_civSeedVillages`) | done | `cartalith_civ::civ_seed_villages`; the toggle the milestone flagged as UI-less now exists — `new_world_dialog.gd`'s header names village seeding among the settings it owns |
| P2-16 | 16 — provinces (`_civGenerateProvinces`) | done | `cartalith_civ::civ_generate_provinces`; `WorldGen::get_provinces` and `build_province_boundary_texture`; the overlay is assigned to `viewport_host.gd`'s `province_view` every redraw, and provinces feed `world_data_window.gd` and `place_search.gd`. *The scope document's "deliberately not wired, no new `TextureRect`" is stale* |
| P2-17 | 17 — economy investigated, first slice ported | done | `cartalith_civ::civ_resource_trade_balance` plus `civ_world_mean_resources` / `civ_catchment_km2` / `civ_place_resource_context`; `WorldGen.trade_balances` → `civ_trade_bridge.rs` → `engine_bridge.gd` → `world_data_window.gd`. *The scope document contains both "not yet wired anywhere" and its own same-day retraction; only the second is true* |
| P2-18 | 18 — culture beyond naming (`_civCultureTerrainFit`) | done | `cartalith_civ::civ_culture_terrain_fit`, called inside a `#[func]` off a live `civ_faction_aggregates` result, surfaced in `shell/faction_roster_window.gd` (its header names the verdict explicitly) |
| P2-19 | 19 — Journey Planner milestone 1 (physical primitives + seasonal/closure logic) | done | `jp_fatigue` / `jp_load_penalty` / `jp_surface_gain` / `jp_can_use_wheels` and `jp_season_at` / `jp_rest_days` / `jp_seasonal_closure` / `jp_sea_closure`, imported by `cartalith-godot/src/journey_bridge.rs`, backing `shell/journey_planner_view.gd` (3 165 lines, with a real reroute control). *"Not wired to any caller … unstarted future work" is stale on both halves* |
| P2-20 | 20 — `_civFactionAggregates` | done | `cartalith_civ::civ_faction_aggregates`, called from `cartalith-godot/src/lib.rs` and `civ_military_bridge.rs`; `tests/golden_parity_faction_aggregates.rs`; reaches the UI through `faction_roster_window.gd`. *"No `#[func]`, no GDScript; all UI work is on hold" is stale twice over — the hold was lifted the same day it was called* |
| P2-21 | 21 — `_civSelectMetropolises` + `_civApplyRecovery` | done | `cartalith_civ::civ_select_metropolises` and `cartalith_civ::timeline::civ_apply_recovery`; `tests/golden_parity_metropolis_recovery.rs`. Surfaced in File ▸ New world ▸ Generation via `new_world_dialog.gd`'s `set_metropolis_enabled` / `set_recovery_phase` |

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
| JP-1 | Physical-modeling primitives + seasonal/closure cluster | done | `jp_fatigue`, `jp_load_penalty`, `jp_surface_gain`, `jp_can_use_wheels`, `jp_season_at`, `jp_rest_days`, `jp_seasonal_closure`, `jp_sea_closure` — 90 `pub fn jp_*` in `cartalith-civ/src/lib.rs` |
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
| JP-06/08 | Named journeys persisting across save/load | done | `journey_planner_view.gd::journeys_document()` / `restore_journeys_document()`, wired through `project_save_with_documents` into the `entities/journeys.json` slot, called from `app.gd`. *The scope document's "in-session only … no save-writer exists (FI-01)" is stale, and so are five header comments inside `journey_planner_view.gd` itself* |
| JP-CONF | Route-planner conformance re-check — `_jpEnsurePlan` reaching the shell | done | `cartalith_civ::civ_path_water_frac`; `WorldGen::jp_plan_for_route`; probe `godot-project/_routeplanner_probe.gd` |
| JP-QC1 | Quality ceiling — wildlife richness feeding `jp_foraging` | done | `cartalith-civ/src/wildlife.rs` ports `buildTRI` / `guildTrophic` / `build_ecoregions` / `region_richness` / `assign_wildlife` in full, with `golden_parity_wildlife.rs`; `cartalith-godot/src/lib.rs` passes a real `forage_mod` from `sample_bridge::WildlifeCache`. *Three files still say this is unported: the scope document, `journey_bridge.rs`'s module doc, and — corrected — `journey_planner_view.gd`* |
| JP-QC2 | Quality ceiling — ocean-current / wind coarse fields reaching the sea-lane router and `jp_sea_condition` | done | **Built 2026-09-01; the "blocked" framing this row used to carry was itself wrong** — no `WorldState` retention was ever needed. `cartalith_climate::current_wind_field`/`current_ocean_field` already existed as callable `pub fn`s, already used (deliberately uncached) by the Wind/Ocean-currents debug views' `sample_bridge::flow_fx_raster`. `coarse_ocean_wind_fields` (`cartalith-godot/src/lib.rs:786`) mirrors that same recipe; both `jp_plan`-driving `#[func]`s — `jp_plan_for_route` (`:9392`) and `jp_compute` (`:9524`) — now pass `Some(&JpCoarseField)` for `ocean_field`/`wind_field`, not `None`. The `cartalith-civ` consumer side (`jp_sea_condition`, `JpWorld::ocean_field`/`wind_field`) was already complete and already golden-tested (`m5_sea_condition_reads_the_real_wind_and_current_and_zeroes_an_oared_hull`) — the whole gap was two `None`s at the Godot boundary |
| JP-QC3 | Quality ceiling — `_civSeaTimeEdgeCost` (current/wind-costed sea lanes) | done | **Built 2026-09-01, reversing the prior decline.** `civ_sea_time_edge_cost` (`cartalith-civ/src/lib.rs:7522`) plus `CIV_LANE_REF_VESSEL`/`CIV_LANE_CURRENT_W`/`CIV_LANE_TACK_FLOOR` (reference lines 21197/21198/21203). `road_dijkstra` gained an additive `edge_cost: Option<&dyn Fn(usize,usize,isize,isize)->f64>` parameter (`lib.rs:5329`) — every pre-existing call site still passes `None`, bit-identical. `civ_sea_routes` (`lib.rs:7632`) takes `ocean_f`/`wind_f` and reproduces the reference's own passability wrap: an edge with either endpoint impassable in the land/lake cost grid never reaches the costed callback, so a strong current cannot make water sailable. Wired via `coarse_ocean_wind_fields` from both `compute_civilisation` callers, `absorb` (`cartalith-godot/src/lib.rs:2511`) and `recompute_civilisation` (`:3727`). Golden-tested: `civ_sea_time_edge_cost_is_none_without_any_field_and_penalises_a_current_aligned_edge`, `civ_sea_routes_still_connects_ports_with_or_without_current_and_wind_fields` |
| JP-QC4 | Quality ceiling — `jp_road_cells` seeing hand-drawn (manual) ways | done | **Built 2026-09-01.** `jp_road_cells` (`cartalith-civ/src/lib.rs:11778`) now takes a `manual_ways: &[tools::ManualWay]` parameter and applies the reference's `'ancient' -> ["Dirt Track","Deteriorated"]` mapping plus the manual Road/Track tuple, filtering `SeaLane`/hidden. Wired at both Godot call sites (inside `jp_plan_for_route` and `jp_compute`, `cartalith-godot/src/lib.rs`) via `self.infra.as_ref().map_or(&[], \|t\| &t.ways)`. Golden-tested: `jp_road_cells_reads_hand_drawn_ways_including_ancient` |
| JP-PARTY | Widen `JpParty` from four fixed species to a generic animal map | declined | `jp_capacity_ex`'s sums are pinned to `JP_ANIMAL_KEYS` and read `jp_seasonal_animal` / `jp_desert_animal_mod` per species; the shipped alternative is `travel_library.rs`'s "substitutes for" path. Declined in the document and matched by the code |

**Group total: 19 — 18 done, 1 declined.**
### Economy and trade · `ECONOMY_SCOPE.md`

This document carries no milestone numbering — its work sits in prose sections
and a "real next milestones" list. The IDs below are assigned here so the rows
can be referred to.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| EC-1 | Pass 1 — `civ_resource_trade_balance` + catchment resource context, wired end to end | done | `cartalith_civ::{civ_resource_trade_balance, civ_world_mean_resources, civ_catchment_km2, civ_place_resource_context}`, called per settlement in `compute_civilisation`, exposed as `get_trade_balances()`, read by `engine_bridge.gd` and three workspaces |
| EC-2 | `_civPlaceSmelting` | done | `trade.rs` defines `civ_place_smelting` with a three-case golden suite; committed in `4ec07f5` |
| EC-3 | Food-surplus cluster — `_civFoodShed` / `_civPlaceFoodSurplus` / `_civPlaceCatchmentCeiling` / `_civCatchmentPop` | done | **Crate-complete AND Godot-wired as of 2026-09-01 (second pass), with the UI display gap now closed.** All four are real: `civ_food_shed` (`trade.rs`, a direct port of `_civFoodShed`/`_civFoodConnected`/`_civRoadConnected`/`_civFoodMode`/`_civFoodDeliverable`/`_civGoodReach`, distinct from `trade.rs`'s separate, pre-existing 15-good trade match, which excludes `food`); `civ_place_food_surplus`/`food_surplus_ratio` (unit-tested this pass); `civ_catchment_pop` (`timeline.rs`). The chain: `civ_trade_bridge.rs`'s private `food_shed_rows()` (`:135-236`) builds one shared `RoadComponents` and calls `civ_food_shed` once per settlement, resolving `farmers_per_urbanite` through `civ_ag_tech_by_key`; the `#[func] civ_food_shed` (`:490-529`) reads it out; `engine_bridge.gd:2625` (`civ_food_shed()`) and `trade_store.gd` (`_food_shed` cache, `food_shed()`/`food_shed_for(index)`, populated by `refresh()` alongside `civ_trade_flows`) complete the chain to the shell. The existing "Match trade flows" button (`infrastructure_workspace.gd:604,758`, unmodified) already triggers it. **The UI gap is closed:** `place_editor_window.gd:385` now reads `TradeStore.food_shed_for(_index)` in the Trade tab. Verified: `cargo test -p cartalith-godot --lib` 409/409 after a fresh build, `cargo check --workspace` clean |
| EC-4 | `_civFactionAggregates` itself | done | `cartalith_civ::civ_faction_aggregates` with three live callers; `civ_ocean_dist_field` and the `CIV_TAX_RATE` / `CIV_PRIMARY_SPECIALISATION` tables ported alongside |
| EC-5 | The Journey Planner as the economy layer's consumer | done | Tracked above as JP-1…JP-INT-5 |
| EC-6 | `civ_culture_terrain_fit` genuinely callable | done | Called inside a `#[func]` returning key/value/world_mean/ratio/verdict per faction; consumed by `faction_roster_window.gd`'s "Territory fit" panel. *The scope document says "still deliberately not exposed … all UI work is on hold" — the hold was lifted the same day it was called* |
| EC-7 | `_civSaltAccess` | done | `trade.rs` defines `civ_salt_access` with a three-case golden suite; committed in `4ec07f5` |
| EC-8 | `_civFactionAggregates`' resource- and density-fed half, surfaced as a readout | blocked | `compute_civilisation` frees the resource rasters and never retains a population-density field; surfacing them is a memory decision, stated on screen at `faction_roster_window.gd`. The aggregate is computed but only its terrain / power / tax halves reach a control |
| EC-9 | Military manpower as the economy layer's first real consumer | done | `cartalith-civ/src/manpower.rs` reads `civ_current_agrarian_density`, `civ_faction_aggregates`, `civ_catchment_pop`'s tiers and `RoadComponents`/place navigability; surfaced in `civilization_workspace.gd`'s "Military" category |

**Group total: 9 — 8 done, 1 blocked.**

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
| MM-7 | Campaigns, unit movement, combat resolution | declined | No combat or campaign type anywhere in `cartalith-civ`; independently confirmed by `landmark.rs`'s `battlefield` `not_built` string, "There is no conflict entity in this port" |
| MM-8 | Change over time (manpower across the year cursor) | declined | `manpower.rs` takes no year argument and `TimelineSnapshot` carries no manpower field; the model reads the world as it stands, as §4 states |
| MM-F2 | Finding 2 — standing armies land at Imperial Rome's ratio, not the era table's | blocked | Owner decision. Correcting it means recalibrating outputs already validated against the specification's own worked example. The era rows' `standing: (0.00, 0.01)` / `(0.001, 0.01)` bands are in source unchanged; the model's shares are computed from the worked example, not from these. **Reported, not tuned** |

**Group total: 9 — 4 done, 4 declined, 1 blocked.**

### Story planning · `STORY_PLANNING_SCOPE.md`

One subsystem over the Timeline's year cursor. It carries the owner's three
2026-08-25 forks; two of them are still open (owner decisions 1 and 3 above).

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| SP-1 | The Journey entity | partial | **Half of the document's own "done means" is met.** A journey *does* survive save → load → reopen — `journey_planner_view.gd::journeys_document()` / `restore_journeys_document()` through `entities/journeys.json` — but as a **GDScript-owned dictionary, not an engine entity**. `grep "pub struct Journey"` across `crates/` returns only `JourneyLeg`, `JourneyCost` and `journey_bridge::JourneyWorld`, all plan-computation types; consequently `travel_bridge.rs::animal_usage_in_journeys` still reads `-> usize { 0 }` and no party-preset reference count is real |
| SP-2 | Journey progression over the cursor | blocked | Depends on SP-1's engine-side entity, and on owner decision 3 (regenerate semantics for a route polyline). Nothing couples a journey to the Timeline's year cursor: `journey_planner_view.gd` has no `civ_goto_year`, year cursor or party marker |
| SP-3 | The settlement timeline strip | not started | No per-settlement history accessor exists. `civ_year_diff()` returns tid sets only; the shell's own note in `civilization_workspace.gd` says the old snapshot's settlement data "no `#[func]` exposes yet". `cartalith-godot` exposes only `civ_add_year`, `civ_goto_year`, `civ_year_diff`, `civ_run_collapse_simulation` — all world-level — and `timeline_bridge.rs`'s public surface is `CollapseSimRequest` / `CollapseSimReport` / `run_collapse_simulation`, none per-settlement |
| SP-4 | The conflict overlay | blocked | Owner decision 1. No conflict or battle entity anywhere (`grep 'struct Conflict\|struct Battle'` → 0). The only "conflict" in the shell is a collapse-simulation *character* string. Independently corroborated by `landmark.rs`'s `battlefield` and `ruin` `not_built` strings, both naming SP-4 |
| SP-5 | The planning aid, joined up | blocked | Deliberately last; worth nothing until at least two of SP-1…SP-4 exist, and only SP-1 is partly there. SP-3 and SP-4 have no code to join |

**Group total: 5 — 1 partial, 3 blocked, 1 not started.**

### Markdown Vault · `MARKDOWN_VAULT_SCOPE.md`

Seven milestones (0-6). The entity audit that opened this work found that
continents did not exist as entities; milestone 0 created them.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| MV-0 | 0 — the addressable continent | done | `cartalith_civ::Continent` and `civ_continents(...)` with its own `civ_continent_name_rng` stream; exposed as `WorldGen::get_continents()`; read by `engine_bridge.gd` and turned into a vault-linkable row by `civilization_workspace.gd::_knowledge_row(cg, "continent", …)` |
| MV-1 | 1 — link, read, section-aware write-back | done | Crate `cartalith-vault` (`backlinks.rs`, `block.rs`, `export.rs`, `links.rs`, `markdown.rs`, `provider.rs`, `template.rs`); `cartalith-godot/src/vault_bridge.rs` carries 47 `#[func]`s; panels in `shell/vault_window.gd` (`_build_connection`, `_build_attach`, `_build_links`, `_build_reader`) and persistence in `shell/vault_store.gd` |
| MV-2 | 2 — the map snapshot (§21, §22) | done | `cartalith-vault/src/export.rs` implements the map snapshot; committed in `4ec07f5` |
| MV-3 | 3 — project-scoped links (§26) | done | **The defining document files this as *blocked*; the blocker has lifted.** The save format carries a civ layer: `project_bridge.rs` defines `SLOT_VAULT = "vault.json"`, writes `self.vault.store.to_json()` into the project's documents and restores it via `LinkStore::from_json`, and the same tree carries `entities/settlements.json`. The shell half is now wired: `shell/vault_store.gd` and `vault_bridge.rs` register the project-scoped `vault.json` slot; committed in `4ec07f5` |
| MV-4 | 4 — the Android provider (§6) | not started | Storage Access Framework: a tree URI, a persisted permission grant, and a provider beside `FsVault`. `cartalith-vault/src/provider.rs` names the SAF requirement in a comment and `FsVault` is the only implementation in the file |
| MV-5 | 5 — the conflict UI (§14's *Compare*) | done | Built 2026-09-01: `vault_window.gd`'s `_compare_link()`/`_compare_dialog()`/`_lcs_diff()`/`_build_diff_rows()` — an O(n·m) LCS diff between the on-disk file and the working copy's own preview, deliberately calling `vault_read_file`/`vault_preview_section_write` rather than `vault_reload_link`, so opening Compare cannot itself clear a Stale status. §14's three-way prompt (Reload source / Keep current / Compare…) is now complete. Dynamically verified end to end (edit externally → Stale → Compare shows the real diff without clearing Stale → Reload clears it) |
| MV-6 | 6 — search, the note as data, culture, and "confirm always" | partial | **Three of four panel pieces are built**: search (`vault_window.gd::_build_search()` over `engine_bridge.gd`'s `vault_search`), the "note says" readout (`_build_note_data()` / `_build_entity_data()` over `vault_file_data` / `vault_link_data`), and the three don't-ask-again checkboxes (`_build_write_prefs()` over `vault_write_prefs()`). **Missing: the culture picker.** `EntityKind::Culture` and `get_cultures()` both shipped, but nothing in the shell opens the vault scoped to a culture — the `_knowledge_row` call sites pass `"faction"`, `"province"`, `"continent"` and `"settlement"`. See the record defect below: the shell tells the user `get_cultures()` does not exist while it sits in `cartalith-godot/src/lib.rs` |

**Group total: 7 — 5 done, 1 partial, 1 not started.**

MV-3 is the row to watch: **the defining document files it as *blocked* and the
blocker is gone.** It is not-started, not blocked, and both `vault_store.gd` and
`vault_bridge.rs` still recite the retired reason in source.

### Landmark generation · `LANDMARK_GENERATION_SCOPE.md`

Nine milestones. The scope document's §0 says "**No code was written for this
pass**" and §3 opens "**Nothing below is started**"; both sentences were true
on the day they were written and false the next. Seven of the nine are
substantially built.

`crates/cartalith-civ/src/landmark.rs` is now **3 846 lines** (was 3 730): 49
kind specs, of which **14 are `buildable: true`** (was 13;
`resource_extraction_site` went buildable 2026-09-01 — see the residual note
below) and 35 carry a `not_built:` reason; **6 carry `needs_viewshed: true`**
with no implementation behind them.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| LM-1 | M1 — extract the analytical field library (Category A) | done | `cartalith-terrain/src/analysis.rs`, 725 lines — `slope`, `aspect` (the gap M1 existed to close), `curvature` / `curvature_at`, `tpi` / `tpi_multiscale`, `local_relief`, `ruggedness`, `normalise` — all `Vec<f32>` and resolution-scaled, with the §31 Category A notice in the module doc. **The consolidation half was declined in source** with the reason given: `build_ao` stays bit-identical because `DECISIONS.md` §7a protects rendered output. Reachable: `landmark.rs` calls `analysis::tpi` |
| LM-2 | M2 — hydrological candidates: waterfall, ford, confluence | done | `kinds()` marks `waterfall`, `ford`, `river_confluence` buildable, plus `spring`, `lake`, `gorge`, `cliff`, `harbour`; `LandmarkInputs` takes flow/channel/recv/order/water. Fixtures shaped to reach the code: `each_waterfall_constraint_is_load_bearing`, `dropping_the_strahler_field_still_places_confluences` |
| LM-3 | M3 — mountain-pass candidates | done | `LandmarkInputs::corridors` — "`build_route_corridors` … Mountain passes read this and nothing else can substitute for it"; `mountain_pass` buildable; the §8 `S_pass` weights are named constants. The 2D saddle half was **declined in source**: `saddle`'s `not_built` reads "A saddle with connectivity is a mountain pass, which is generated; a saddle without it is a shape, not a landmark" |
| LM-4 | M4 — peak / ridge / prominence candidates, generalised to 2D | done | `peak` and `ridge` buildable; the pass reads M1's extracted field through `Ctx::tpi_broad` built from `analysis::tpi` at the broad-scale radius; the two-cone fixture drives the spacing tests |
| LM-5 | M5 — resource- and settlement-linked candidates | partial | **The resource half is built and reachable**: `mine` and `quarry` buildable, driven by `MINE_RESOURCES` (8 keys) and `QUARRY_RESOURCES` (4 keys) over `LandmarkInputs::resources`, assembled by `WorldGen::landmark_resource_pairs`. **The accessibility half is absent**: `Ctx::influence` is straight-line Euclidean gravity over a wrap-corrected distance, so a resource cell with real road access scores identically to one with none. `civ_dijkstra_path` / `WayRouter` are not inputs and `LandmarkInputs` has no roads/ways field — so `market_site`, `trade_depot` and `caravan_station` all carry `not_built` strings naming the same missing §13 route-load term |
| LM-6 | M6 — spatial competition / Poisson-disc filtering | done | `landmark.rs::Buckets` / `Buckets::new(gw, gh, world, max_radius)`, used with a shared cross-type field. **Bridson (2007) was deliberately declined**, with the argument and the 0.866·r² vs π·r² packing measurement in the doc comment. Mutation/boundary tests present: `spacing_rejects_the_weaker_of_two_candidates_inside_one_radius`, `at_cap_and_spacing_are_different_answers`, `crowding_higher_packs_tighter`, `a_zero_or_nan_crowding_does_not_take_the_map_with_it`, `cross_type_competition_changes_the_answer`, `a_placed_landmark_is_never_inside_its_own_exclusion_radius` |
| LM-7 | M7 — viewshed (the expensive one, entirely new) | blocked | **Owner decision 2** (accuracy/cost budget). Zero geometric line-of-sight code in any of the sixteen crates — `grep 'fn viewshed\|fn line_of_sight\|fn los_'` returns nothing; the only `viewshed` hits are comments naming its absence. Gates six kinds by name (`fort`, `watchtower`, `sacred_mountain`, `border_marker`, `volcanic_feature`, plus peak's scoring); `fort`'s `not_built` reads "§18's own model puts F_visibility at 0.20 — the joint-largest term — and there is no viewshed field anywhere in this workspace" |
| LM-8 | M8 — Category C suitability synthesis + the Landmark object model | done | `landmark.rs::Landmark { id, kind, class, x, y, elevation, score, importance, causal, seed }` — §22's object model including the causal chain and §27's `seed_L`; `pub fn generate(...)` runs §30's twelve steps; the Category C weight/threshold block has every weight a named commented constant. Reachable end to end: `landmark_bridge.rs` + `landmark_kinds()`, `landmark_settings()`, `landmark_run()`, `landmarks()`, `landmark_funnels()`, `landmark_headroom()` → `engine_bridge.gd` → CIVIL ▸ Landmarks (`civilization_workspace.gd::_build_landmarks`) and CARTO ▸ Assets & landmarks (`cartography_workspace.gd`). Edge-case bar met by `degenerate_grids_do_not_panic` and `a_wrongly_sized_optional_input_degrades_rather_than_panicking` |
| LM-9 | M9 — cultural interpretation and temporal state (research §24-26) | blocked | Story planning **SP-4** (no conflict entity) plus owner decisions 1, 4 and 5. Every Cultural-family row in `kinds()` is `buildable: false`; `shrine`'s reason reads "§26 is explicit that cultural meaning must not be hardcoded into geography … That needs the civilisation's own traits as an input, which this pass does not take"; `ruin`'s names the Conflict link as SP-4. `cartalith_vault::EntityKind` still has no `Landmark` variant |

**Group total: 9 — 6 done, 1 partial, 2 blocked.**
**Residual inside M8:** 35 of 49 declared kinds still ship `buildable: false`
(was 36), each with its reason in source. `resource_extraction_site` went
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

Seven milestones from an owner-supplied paper, scoped 2026-08-29. **None is
started.** A foundation landed the same day that the scope document does not
number.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| RD-0 | Foundation — culture and religion as quantitative traits, and the compatibility relation | done, **consumed** | `cartalith-civ/src/belief.rs`, 945 lines — `culture_domain`, `ReligionDomain`, `CIV_RELIGION_DOMAIN`, `religion_domain`, `CompatBasis`, `Compat`, `compat`, `compat_value`, `NEUTRAL_COMPAT`, `COMPAT_WEIGHTS`. Its module doc says it is "the foundation both milestone 1 and milestone 3 need, and nothing above it". **Corrected 2026-09-03:** that claim was false. `grep 'belief::'` across `crates/` excluding the file itself returns **15 hits**, all in `cartalith-godot/src/lib.rs` (`belief_seed`, `belief_links_from_ways`, `BeliefNetwork::build`, `belief_step`, `BELIEF_STEP_RATE`). It has consumers and it is bound |
| RD-1 | 1 — MVP: network exposure and conversion, read-only | not started | `SettlementReligionState`, the type milestone 1 is defined by, exists only as a name in `belief.rs`'s module doc. **Corrected 2026-09-03:** the last clause was false — `get_settlements()` emits both `religion` and `adherents` (`lib.rs:6855-6856`), and a diffusion step exists (`belief_step`). What is genuinely absent is the named `SettlementReligionState` type and the retention split; re-scope this row against `belief.rs` before scheduling it |
| RD-2 | 2 — institutional presence and retention split (§11, §22) | not started | No `Inst_{i,R}`, no clergy count, no `P_retain` |
| RD-3 | 3 — religion trait vectors, authored (§6-§13) | not started | `COMPAT_WEIGHTS` is `[None, Some(1.0), None, None, None]` — one of five components populated. Primarily a content pass |
| RD-4 | 4 — prestige and success bias (§15-16) | not started | Needs `EliteAdherents` / `RulerReligion` / `MerchantStatus` terms this port does not compute |
| RD-5 | 5 — political modifiers (§25) | not started | Also the milestone where §4's unresolved fork must be decided: is a state religion the hand-set flag or a derived plurality? |
| RD-6 | 6 — competition (§18) and vertical/horizontal/oblique weighting (§23) | not started | — |
| RD-7 | 7 — sensitivity-analysis tooling (§31) | not started | Dev-facing |

**Group total: 8 — 1 done (unconsumed), 7 not started.**
The religion *screens* are separately blocked; see the gap-register group.

### Timeline · `TIMELINE_SCOPE.md`

Six milestones, all built 2026-08-19.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| TL-1 | 1 — shared prerequisites: population-ceiling chain + stable ids | done | `timeline.rs::{civ_subsistence_mode_at, civ_agrarian_density_km2, civ_current_agrarian_density, civ_settlement_population, civ_tier_floor, civ_tier_for_population, civ_assign_tid, civ_resync_next_tid}` |
| TL-2 | 2 — proximity graph + Brandes betweenness centrality | done | `timeline.rs::civ_proximity_adjacency` and `civ_betweenness_from_adjacency`, the latter documented as Brandes (2001), un-normalised, one BFS per source |
| TL-3 | 3 — the collapse and recovery step functions | done | `timeline.rs::{civ_collapse_step, civ_recovery_growth_step, civ_apply_recovery, civ_settlement_stress, civ_mortality_migration_rates, civ_gravity_migrate}` with `CollapseStepResult` / `RecoveryStepResult` |
| TL-4 | 4 — snapshot data model + orchestrator | done | `timeline.rs::{TimelineSnapshot, YearDiff, civ_year_diff, civ_snapshot_save, civ_snapshot_load, civ_simulate_timeline}` with `SimulateMode` / `SimulateTimelineOpts` |
| TL-5 | 5 — the Godot boundary | done | `cartalith-godot/src/timeline_bridge.rs` (`CollapseSimRequest`, `CollapseSimReport`, `run_collapse_simulation`) and the `#[func]`s `civ_add_year`, `civ_goto_year`, `civ_year_diff`, `civ_run_collapse_simulation` |
| TL-6 | 6 — UI playback controls | done | `shell/workspaces/civilization_workspace.gd`'s Timeline category (its own header cites `TIMELINE_SCOPE.md` milestone 6), with the "Exist only" filter and the year controls under Politics |

**Group total: 6 — 6 done.**
Two Timeline-adjacent items remain open as owner decisions, not milestones:
CV-24 (the year scrubber as program scope) and ED-02 (the undo-history panel) —
`OUTSTANDING_WORK.md` §3.1.

### Phase 3 — 2D terrain appearance · `TERRAIN_APPEARANCE_SCOPE.md`

Six milestones plus one follow-up. All built. The 3D half of Phase 3 is not in
this document and does not exist — see *Orientation*.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| TA-1 | 1 — audit + `TerrainAppearance` abstraction + one real editable ramp | done | `render.rs::TerrainAppearance`, owned by `RenderCtx`; the field-name→range table drives `WorldGen::set_appearance`, which `app.gd` calls at startup with `DccSettings.lighting_defaults()`, and `dcc_settings.gd::appearance_defaults()` persists. *"Not yet wired to any UI/`#[func]`" is stale* |
| TA-2 | 2 — multidirectional hillshade + ambient occlusion | done | `TerrainAppearance::{sun_alt_deg, relief_lights, ao_strength, ao_radius_frac}`; `render.rs::build_ao`. Exposed through the `set_appearance` key table |
| TA-3 | 3 — hydrology-based colour tint | done | `TerrainAppearance::{hydro_wet_strength, hydro_wet_radius_frac}` (defaults 0.38 / 0.006), computed by `render.rs::build_hydro_wetness`, tunable via the `"hydro_wet_strength" => "Wetness"` row |
| TA-4 | 4 — the atlas look: paper ground, forest stippling, plate border | done | `TerrainAppearance::{paper_strength, paper_tint, paper_grain, paper_mottle, paper_wash, stipple_strength, stipple_scale_frac, border}`; `render.rs::paper_tone` and `apply_border` |
| TA-4F | 4 follow-up — the overlays learn about the frame | done | `WorldGen::border_inset_frac` is consumed by every overlay draw call — `viewport_host.gd` and `civilization_workspace.gd` both pass `_bridge.border_inset_frac()` alongside the road/sea-route geometry |
| TA-5 | 5 — geological material exposure + local contrast | done | `TerrainAppearance::{litho_exposure, local_contrast, local_contrast_radius_frac, local_contrast_knee}`; `render.rs::apply_local_contrast`, a rayon-parallel neighbourhood pass over final colour |
| TA-6 | 6 — the GPU question answered by measurement; §29 quality tiers | done | Parallel pass: `use rayon::prelude::*` in `render.rs` with `par_chunks_mut` at three sites and a `rayon::join`. Tiers: `enum QualityTier`, `TerrainAppearance::for_tier`, `recommended_quality_tier` (with an explicit Android downgrade); `WorldGen::{get_quality_tier, set_quality_tier, list_quality_tiers, get_recommended_quality_tier}`; `engine_bridge.gd` and the tier picker in `menus.gd`; `tests/appearance_tiers.rs` |

**Group total: 7 — 7 done.**
The GUI for all of this is `shell/workspaces/render_workspace.gd` (1 055 lines),
composed into CARTO — see GFP-5.

### Sculpt live · `SCULPT_LIVE_SCOPE.md`

Five milestones (L0-L4). The sculpt **editor** shipped as tool-plan milestone B;
"live" — bounded preview during a stroke — did not.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| SL-0 | L0 — measure, before deciding anything else | not started | No instrumented harness for stamp `apply()`, the three `with_appearance` precomputes, the per-pixel loop, `enforce_river_channels`, `compute_flow` or `refresh_climate` at 512²/1024²/2048². The document states nothing below it should be built until this exists |
| SL-1 | L1 — bounded live preview | not started | `render.rs::build_ao` still takes `(field, gw, gh, sea_level, world, a)` — **no window parameter**, and neither do `smooth_sea_h` or `build_hydro_wetness`. `PassBuffer::touched_bounds()` exists in `cartalith-spatial`, and `cartalith-godot/src/lib.rs` refers to it in the subjunctive ("would give the rectangle a bounded …"), i.e. it is not called for this |
| SL-2 | L2 — live water, at proxy resolution | not started | Blocked by definition on L0's numbers, which decide between the proxy and full-resolution GPU routes |
| SL-3 | L3 — downstream (erosion, climate, biomes, civ): proxied, not live | declined | A recommendation, not a limitation to engineer away: the crates operate on the whole field, and erosion and climate are global equilibria — a locally-recomputed result is a different answer, not a preview. What ships instead is the staleness readout (tool-plan milestone F / GFP-3) |
| SL-4 | L4 — the three §5.2 blocks with no engine | not started | Independently scoped: neither v2.10 nor `sculpt.rs` has them |

**Group total: 5 — 4 not started, 1 declined.**

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
| AL-9 | §9 — the Asset library GUI window | done | `shell/asset_library_window.gd`, 160 685 bytes, reached from `menus.gd`'s `_live(p, "⧉ Asset library", ID_ASSET_LIBRARY, KEY_MASK_SHIFT \| KEY_A)`. The eight-family rail matches `cartalith-assets`' own `slots.rs` grouping. *§9's body enumerates eight gaps that §10 and §11 later close; both readings stand in the file and §9 is only true as of 2026-08-19* |
| AL-10 | §10 — the `AssetDB` `#[func]` surface (twenty `as_*` methods) | done | `cartalith-godot/src/asset_bridge.rs` — `AssetLibrarySession` with `import_item`, `add_custom_slot`, `remove_item`, `validate`, `thumbnail_png`, batch tag/collect/rename/duplicate/delete, `export_pack_bytes`; held as `WorldGen::asset_library` and surviving re-generate |

**Group total: 10 — 10 done.**
Three items are **declined because the engine has no counterpart** and should
not be re-proposed: AS-14 (user-picked active variant — variant choice is
weighted and seeded), AS-15 (per-slot Anchor — `Anchor` is a *family* property),
AS-16 (the 24-family rail — owner decision, disclosed in the window's header).
### Phase 5 — Urban morphology · `URBAN_MORPHOLOGY_SCOPE.md`

Nineteen rows (milestones 1-17, plus 8a and 17a which shipped out of order).
**This is the largest block of unbuilt work in the project.**

The single decisive check: `crates/cartalith-urban/src/lib.rs` declares exactly
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
| UM-5 | 5 — the site model | done | `site.rs` — `build_site`, `Site`, `SiteOpts`, `WaterCtx`, `TerrainCtx`, `Harbour`, `Hill`, `Economy`, `shore_from_mask`, `terrain_suitability`; `site/tests/golden.rs` is 4 502 lines. Called from `cartalith-civ/src/urban_adapter.rs` |
| UM-6 | 6 — anchors and primary routes | done | `routes.rs` — `place_anchors`, `build_primaries`, `build_primaries_from_paths`, `Anchors`, `Route`; called from `urban_adapter.rs` |
| UM-7 | 7 — organic growth | done | `growth.rs` — `grow`, `GrowOpts`, `Occupancy`, `WallBuilder` / `RecordingWallBuilder`, `WallState`, `WallGeneration`, `supersede_wall`, `estimate_carrying_capacity`, `logistic_ramp`, `ring_crossings`, `dist_to_line`; `growth/tests/golden.rs` is 2 159 lines |
| UM-8A | 8a — the plaza (`buildPlaza`) | done | `plaza.rs::build_plaza` with `plaza/tests.rs` + `golden.rs`; called from `urban_adapter.rs` on both the organic and radial branches |
| UM-8 | 8 — radial (Venus) streets, waterway | done | `radial.rs` — `build_radial_streets`, `build_waterway`; committed in `4ec07f5` |
| UM-9 | 9 — water infrastructure (`buildHarbour`, `addRiverBridges`, `detectRiverCrossings`) | done | `water.rs` — 693 lines; committed in `4ec07f5` |
| UM-10 | 10 — fortification (`buildWall`, `applyStarFort`, `townBank`, `builtMassHull` …) | done | `fortify.rs` — 1 288 lines; committed in `4ec07f5` |
| UM-11 | 11 — graph cleanup passes (`pruneLargest`, `removeWaterCrossings`, `privatizeAlleys`, `lanePass` …) | done | `cleanup.rs` — 645 lines; committed in `4ec07f5` |
| UM-12 | 12 — blocks and parcels | done | `blocks.rs` — `build_blocks`, `build_parcels`, `Block`, `Parcel`, with `blocks/tests.rs` and `blocks/tests/golden.rs`; called from `urban_adapter.rs` and drawn by `shell/urban_layout_draw.gd` |
| UM-13 | 13 — districts and buildings | done | `districts.rs` — 1 307 lines; committed in `4ec07f5` |
| UM-14 | 14 — amenities (markets, civic hall, games) | done | `amenities.rs` — 758 lines; committed in `4ec07f5` |
| UM-15 | 15 — hinterland, decay, details, metrics | done | `hinterland.rs` — ~1 054 lines with passing golden; committed in `4ec07f5` |
| UM-16 | 16 — `generate()` orchestration + `hashModel` | ready | Milestones 8-15 exist; the blocker has lifted. Ready to be scheduled |
| UM-17A | 17a — the adapter and the first consumer | done | `urban_adapter.rs::{um_place_context, run_layout, settlement_layout}` → `cartalith-godot/src/urban_bridge.rs::urban_layouts` → `engine_bridge.gd` → `city_viewer_window.gd` and `viewport_host.gd` (map deep-zoom town layer) |
| UM-17 | 17 — the civ adapter (20 pure `_um*` functions) | partial | **16 of 20 ported** (`grep -c "^pub fn um_" crates/cartalith-civ/src/urban_adapter.rs` = 16), verified against the port table at the head of `urban_adapter.rs`: `um_site_box_km`, `um_water_near_km`, `um_water_reach_km`, `um_site_kind_from_terrain`, `um_infer_age`, `um_ray_box_exit`, `um_way_bearing_from`, `um_route_ends`, `um_primary_paths`, `um_terrain_orient`, `um_water_ctx`, `um_terrain_ctx`, `um_place_context` (the last "minus four fields"). `um_wall_spec` / `um_infer_walls` live in `military.rs`. **The three formerly skipped landed in `cff1edc`** — `um_harbour_scale:372`, `um_site_profile:1240`, `um_ore_bearing:1504` — so this row's "three are deliberately skipped pending later milestones" is history as of 2026-09-02. Five cache/draw helpers are out of scope for every milestone by design |
| UM-17A-G | 17a — golden-verify the block-2 `_um*` adapter | done | **2026-09-02.** The recorded blocker — *"needs a block-2 capture harness that can run `_um*` inside the host's full civ scope; the existing harness slices block 4 only"* — was **wrong, not merely stale**: `cartalith-native/tools/um_block2_capture.js` drives the unmodified reference under Node (v24.19.0) and `crates/cartalith-civ/tests/golden_parity_urban_adapter.rs` holds 9 tests over the extracted fixtures. Mutation matrix **22/22 killed**; an independent verifier confirmed the fixtures are genuinely reference-extracted, not replayed from the port. **Two real port bugs found that 11 synthetic-field unit tests had not**: `slope_at` used `f64::hypot` where the reference uses `Math.hypot` (the V8-libm divergence `geom::js_hypot` exists for), and `um_site_profile` clamped the resource-context centre. A third defect was in the fixture itself and was caught before being committed as truth |

**Group total: 19 — 17 done, 1 partial, 1 ready.**

Two known count defects in the defining document, recorded here rather than
carried: it gives the `_um*` denominator as **20** in one place and **28** in
another (the 20-item list is the one that enumerates names, so it is the
checkable one), and it contains the sentence *"the crate is not a dependency of
`cartalith-godot`"* — a **quotation of a pre-milestone-17a finding**. Read out
of context that sentence produces a ruling to add a Cargo edge that would buy
nothing: `urban_bridge.rs` already does `use cartalith_civ::urban_adapter::{…}`
and `cartalith-civ/Cargo.toml` carries `cartalith-urban`, with a comment
defending the indirection as deliberate layering so `cartalith-urban`'s only
dependency stays `cartalith-rng`. The owner's 2026-08-31 authorisation to edit
that `Cargo.toml` was **withdrawn as unnecessary and not exercised**.

### Tool system · `UNIFIED_TOOL_PLAN.md`

Seven rows (A-F plus E2). **All complete**, and as of 2026-09-01 the plan's own
last line says so too, rather than still calling milestone F the only work
left.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| UTP-A | A — `PassBuffer` / staleness core | done | `cartalith-spatial/src/pass.rs::PassBuffer<S>`; `lib.rs::DirtyTracker` and `StageGraph`. Reached from the shell: `app.gd::_setup_staleness()` starts a 1 s poll onto `refresh_staleness()`, which writes the `stale` status slot from `stale_stages()` and mounts a `Recompute` action into `status_row` |
| UTP-B | B — Terrain group, the Sculpt-editor port | done | `cartalith-terrain/src/sculpt.rs` carries the 13-entry `SCULPT_FEATURES` transcription (registry order documented as load-bearing) and the stamp bbox/apply pipeline; `cartalith-godot/src/sculpt_bridge.rs` plus the `sculpt_set_feature` / `sculpt_begin_stroke` / `sculpt_add_point` / `sculpt_end_stroke` / `sculpt_commit` `#[func]`s. Reachable: `world_workspace.gd`'s `_sculpt_click` / `_sculpt_drag` / `_sculpt_release` |
| UTP-C | C — Water & ecology group (lake `water_only` commit path, biome paint override) | done | `cartalith-engine/src/sculpt_commit.rs` step (4) applies Lake stamps a second time with `water_only = true` against the final height; `sculpt.rs::apply_stamp(.., water_only: bool, ..)` with the unit test `the_lake_water_only_pass_never_touches_the_height_field`. Biome paint: `cartalith-godot/src/paint_bridge.rs`, registered under tool id `"paint"` |
| UTP-D | D — Civilization group (place settlement, draw way/route, territory override) | done | `cartalith-civ/src/tools.rs` — `ManualWay`, `civ_place_pick_radius`, `civ_place_pick_weight`; `infra_tools_bridge.rs`'s `way_begin` / `way_append_point` / `way_commit` / `way_discard`; `civ_tools_bridge.rs::paint_at`. Reachable: `civilization_workspace.gd` registers `"settlement"` + `"territory"`; `infrastructure_workspace.gd` registers `"way"` and `"route"` |
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

Six "done means" criteria. All met — but the document carries **no resolution
section at all**, and the only place the pilot is called done is the opening
line of `GPU_LAYER_INTEGRATION_SCOPE.md`.

| ID | Criterion | Status | Evidence |
|---|---|---|---|
| PILOT-1 | Minimal wgpu hardware path: Instance/Adapter/Device + §9 self-test | done | `cartalith-gpu/src/lib.rs` — `GpuContext`, `init_gpu()`, `GpuInitError`, `self_test()` (an 8×8 known-input GPU-vs-CPU gate), test `gpu_context_creates_on_this_hardware` |
| PILOT-2 | One compute kernel: vnoise in WGSL at a real field size | done | `shaders/vnoise.wgsl` and `vnoise_f64.wgsl`; `dispatch_gpu()`; CPU reference `vnoise_grid_cpu()` |
| PILOT-3 | CPU-parity test at an explicit, documented tolerance | done, **answered negatively** | `F32_TOLERANCE = 1e-4` with a doc comment naming `cartalith_noise::hash`'s ~2^61 middle product as the reason, and the test that carries the finding, `f32_hash_diverges_from_cpu_reference`. **The criterion asked the kernel to match the golden-verified CPU output; the code's answer is that the JS-matching `hash` is not f32-portable.** That finding is the pilot's whole value and it is written down only in the *other* document — recorded here so it stops being |
| PILOT-4 | CPU fallback path exercised by a real test | done | `vnoise_grid(ctx: Option<&GpuContext>, ...)` gates GPU behind `self_test`; tests `gpu_fallback_path_matches_cpu_reference` and `cpu_path_is_deterministic`; `ComputePath::{Gpu,Cpu}` reports which ran |
| PILOT-5 | Real measured GPU-vs-CPU numbers at several field sizes | done | `measured_gpu_vs_cpu_timing`; `VnoiseResult` carries `gpu_dispatch_and_readback` / `cpu_duration` |
| PILOT-6 | Lives in its own crate with no gdext dependency | done | `cartalith-gpu/Cargo.toml` `[dependencies]` = `cartalith-noise`, `wgpu` 30, `pollster`, `bytemuck` only; no godot/gdext entry. Test-only dev-deps on `cartalith-terrain` / `-climate` / `-hydrology` |

**Group total: 6 — 6 done.**
`init_gpu_f64` is the pilot's undisposed residue and is owner decision 11.

### GPU layer integration · `GPU_LAYER_INTEGRATION_SCOPE.md`

Nine milestones, two named deferrals, and one shipped subsystem the document
does not mention at all.

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
| GLI-8 | GPU context reuse across `generate_terrain`'s stages | done | `GpuDevice`, `init_gpu_shared_device()`, the `init_gpu_{warp,heterogeneity,jfa_plates,gauss_blur}_with` family and the `*_grid_gpu_with` wrappers. `cartalith-engine` opens one device per call via `init_gpu_device_set()` and passes `set.primary()` to every stage |
| GLI-9 | Flow accumulation on GPU — the first sequential algorithm redesigned | done | `shaders/gpu_flow.wgsl`; `GpuFlowContext`, `init_gpu_flow_with`, `dispatch_gpu_flow`, `GpuFlowResult`; `FLOW_TOLERANCE = 1e-3` and `FLOW_ANY_CELL_TOLERANCE = 5e-3`. Wired once per generate with a `flow_on_gpu` closure used at all four `compute_flow` call sites. Tests `gpu_flow_matches_real_cpu_compute_flow`, `gpu_flow_is_bit_reproducible`, `gpu_flow_downstream_river_network_divergence`; example `examples/flow_downstream_settlements.rs` |
| GLI-D1 | Deferred at m5 — `compute_stress` gather reformulation | not started | WGSL has no f32 atomic add, so the scatter must be rewritten as a gather, which reorders summation and needs its own float-equivalence verification. No stress kernel exists (`grep compute_stress` in `cartalith-gpu/src/lib.rs` → nothing); `cartalith-engine` calls `compute_stress(...)` unconditionally, outside every `if p.use_gpu` branch |
| GLI-D2 | Deferred at m2 — world-wrap support for the milestone 1-5 kernels | not started | `pfbm` / `pridged` periodic variants were never ported to WGSL, so `world=true` silently takes the CPU path for warp and heterogeneity even with the GPU toggle on. `cartalith-engine` has `if p.use_gpu && !world` on both, each with an inline comment naming milestone 2's deferral. Milestone 9's flow kernel *does* support wrap |
| GLI-M | Multi-GPU device set, VRAM budgeting and split-tiles warp — **shipped, and this document has no milestone for it** | done | `cartalith-gpu/src/multi.rs`, 1 291 lines: `MultiGpuMode`, `VramFallback`, `GpuPreferences`, `enumerate_devices()`, `vram_verdict()`, `gpu_allowed_for_grid()`, `device_supports_grid()`, `GpuDeviceSet`, `init_gpu_device_set()`, `split_rows()`, `set_weights()`; `warp_grid_gpu_split` / `warp_band_gpu_with` in `src/lib.rs`. Reached from `cartalith-engine` **before every other GPU stage**, gating the whole GPU path on a VRAM verdict, and from the shell: `menus.gd::_build_gpu_mode_menu()` plus the `gpu_vram_budget_gb` / `gpu_set_vram_fallback` / `gpu_vram_estimate` `#[func]`s. `AlternateFrames` and `ReduceWorkingRes` are deliberately unimplemented variants whose `is_implemented()` returns false |

**Group total: 13 — 10 done, 2 not started, 1 declined.**

**Seven public `cartalith-gpu` functions have zero callers**, re-verified by
grep on 2026-08-31: `heterogeneity_grid_gpu`, `gauss_blur_grid_gpu`,
`assign_plates_grid_gpu`, `flow_accumulation_gpu_with`, `gpu_resistance_grid_cpu`
and `init_gpu_f64` have **0** external hits; `warp_grid_gpu` has exactly **1**,
a doc comment in `cartalith-engine/src/lib.rs`. Deleting them is one small row
in `OUTSTANDING_WORK.md` §3.1, blocked only on the ponytail pass's refusal to
delete public API on its own authority, plus owner decision 11 for
`init_gpu_f64`. Milestone 6's prose still describes the pipeline as calling four
of them; the correction sits two milestones later, under milestone 8.

### CPU multithreading · `CPU_MULTITHREADING_SCOPE.md`

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| CPU-1 | Pass 1 — `cartalith-terrain` (warp, heterogeneity, height, resistance, `gauss_blur`) | done | `rayon = "1"` in `Cargo.toml`; all five parallelised — `compute_warp` (`par_chunks_mut`/`zip`), `box_h`, `box_v` (column-major scratch), `compute_heterogeneity`, `compute_resistance` (`par_iter_mut`), `compute_height` |
| CPU-2 | Pass 2 — `cartalith-civ` | done | Spot-verified in place: `build_lithology`, `build_slope_field`, `build_soil_fertility`, `build_water_access`, `build_biome_raster`, `build_wetland_mask`, `build_carrying_capacity`, `build_npp`, `apply_resource_scarcity` (incl. `par_sort_unstable_by`), `build_resource_potentials`, `build_route_corridors`, `build_settlement_suitability`, `build_travel_cost`, `assign_territory`'s inner loop. **The named-sequential set is genuinely sequential**: `jfa_dist` and `road_dijkstra` carry no rayon |
| CPU-3 | Pass 3 — `cartalith-climate` / `-erosion` / `-hydrology` | done | Hydrology's two wins are exactly the two claimed — `compute_flow`'s rain rescale `acc.par_iter_mut()` and `build_channels`' triple `par_chunks_mut`. The confirmed-unsafe cases stayed sequential: `cartalith-erosion`'s `droplet_kernel` contains zero rayon calls |
| CPU-4 | 2026-08-19 investigation: "only GPU active, no parallelisation" — working as designed | done | Both structural claims re-verified: `rayon = "1"` is present in `-terrain`, `-civ`, `-climate`, `-erosion`, `-hydrology`, `-godot` (and now `-engine`); `compute_height` and `compute_resistance` are called unconditionally from `cartalith-engine`, outside every `if p.use_gpu` branch, so the CPU+Rayon phase still runs on every generate. The timing tables themselves are device measurements — **not checkable from code** |
| CPU-5 | 2026-08-25 ponytail pass — duplicate `build_water_bodies` / `build_slope_field` removed, LOD tile passes parallelised | done | `build_water_bodies` now has exactly one call site in `cartalith-godot/src/lib.rs`, with `CivData::water_bodies` holding the classification for `absorb`/PaintEditor and a doc comment saying why; `build_slope_field` likewise. Sixth crate confirmed: `cartalith-terrain/src/amplify.rs`'s `par_chunks_mut` in `amplify_region` and `add_zoom_detail`, and `tile_render.rs`'s `shade_tile` |
| CPU-6 | Integrated-GPU / multi-adapter idea — recorded, explicitly not scoped | **built, contrary to this document** | See GLI-M. `multi.rs::enumerate_devices()` walks every adapter, `GpuDeviceSet` opens more than one, `MultiGpuMode::SplitTiles` partitions the warp grid across them, reachable at `Preferences ▸ GPU`. The document's sentences — "`cartalith-gpu` currently only ever requests a single `PowerPreference::HighPerformance` adapter … the integrated GPU is never enumerated or used at all, for anything" and "not scoped here" — are **both now false** |
| CPU-7 | Remaining hard-hazard functions (CPU flow accumulation, priority-flood, scatter-writes, per-droplet state) | declined | Confirmed still sequential by reading each: `compute_flow` (only its rain rescale is parallel), `build_water_bodies`' priority-flood (`MinHeap::with_capacity(n)`), `chamfer_dist` / `jfa_dist`, `road_dijkstra`, `droplet_kernel`. The document's own rule — genuine cross-cell state, not "hasn't been tried" — holds in the code |

**Group total: 7 — 6 done, 1 declined.**

**Census drift, re-measured 2026-08-31** (pattern:
`par_iter|par_chunks|par_bridge|par_sort|rayon::join|collect_into_vec|par_extend`
over each crate's `src/`): terrain **11**, civ **30**, climate **60**, erosion
**15**, hydrology **4**, godot **16**, engine **1**. The document's 2026-08-19
census read 8 / 25 / 44 / 13 / 5. **Hydrology went down**, which is exactly the
direction that census was written to detect — the 2026-08-25 ponytail pass
removed duplicated work rather than adding parallelism.

### Memory optimisation · `MEMORY_OPTIMIZATION_SCOPE.md`

Fifteen rows: five landed passes and the ranked R1-R8 list the 2026-08-25 audit
produced. **R1-R3 landed; R4-R8 did not**, and each was re-verified absent.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| MEM-1 | Instrument, confirm the dominant cost, fix (2026-08-16): six unused resource fields | done | `compute_civilisation` sets `resources.clay/buildstone/flint/obsidian/sulfur/alum = Vec::new()` immediately after `build_resource_potentials` returns, with a comment citing the scope document. `ResourcePotentials` still computes all 15, as the fix intended |
| MEM-2 | Tracked budget line item: the global undo stack (2026-08-23) | done | `cartalith-godot/src/undo.rs` — `MAX_STEPS = 5`, `DEFAULT_BUDGET_BYTES = 256 * 1024 * 1024`, constructor takes the budget, module doc states the whichever-binds-first rule. UI: `menus.gd`'s `Undo history` submenu with `Clear undo history now` and a live-cost tooltip |
| MEM-3 | Android budget measured by category + kept instrumentation (2026-08-25) | done\* | Instrumentation is real: `shell/performance_window.gd` reads `RENDER_VIDEO_MEM_USED` / `RENDER_TEXTURE_MEM_USED` / `RENDER_BUFFER_MEM_USED` and the draw-call/object counters beside `OS.get_static_memory_usage()`, labelled as outside it. The PSS/category tables are handset measurements — **not checkable from code** |
| MEM-4 | Generation peak measured field by field, ranked list R1-R8 (2026-08-25) | done | This milestone produced an audit, not production code; its conclusions are the R-rows below. **The three probes it was built on are deleted as of 2026-09-03** — `cartalith-civ/examples/_peakaudit_peak.rs`, `_peakaudit_block.rs`, `_peakaudit_hash.rs` — which `MEMORY_OPTIMIZATION_SCOPE.md` scheduled for exactly this point ("named for deletion when the audit closes"). *This cell read "the probes … are present and uncalled, exactly as recorded" until they were removed, and a verifier caught it the same day; a deleted file named as present is the defect this file exists to prevent.* |
| MEM-5 | Overlay lever 1 — collapse the dash loop into one `draw_multiline` | declined | Measured a no-op and reverted: `godot-project/map_overlay.gd` still emits per-dash lines, and the evidence probe was kept — `_dashbatch_probe.gd` / `.tscn` exist. The document says the probe is retained "as the reason not to try it again"; that is what the tree shows |
| MEM-6 | Overlay lever 2 — bound the overlay by zoom (`_run_offscreen`) | done | `map_overlay.gd` — `_visible_local_rect()` inverts the canvas transform, `_run_offscreen(pts, k, pad)` rejects a run whose bounds miss it, cached once per `_draw()` and applied at the way draw site. Pixel-identity probe kept: `_cull_probe.gd` / `.tscn` |
| MEM-7 | R1 — free the previous world before generating the next | done | `cartalith-godot/src/lib.rs::release_world(&mut self)`, called from `generate_sized` and — the audit's own correction — from `generate_world_structure_sized`. Both call sites sit below their function's refusal checks, as the safety argument requires |
| MEM-8 | R2 — delete four dead resident grids | done | `WorldState` no longer declares `flexure_field`, `heterogeneity_field` or `flow_area` — they are locals inside `generate_terrain`. `ChannelResult::slope` was **deliberately not deleted** and is released instead (`ch.slope = Vec::new()`), because `golden_parity_river.rs` asserts it in all three cases. *§6's R2 table still says `slope` is read by "nobody, anywhere"; the later "Where the audit was wrong" section retracts it and the table was never edited* |
| MEM-9 | R3 — block `build_resource_potentials`' `per_cell` buffer | done | `const RESOURCE_BLOCK: usize = 1 << 18;` with `per_cell: Vec<[f32; 15]>` allocated at `RESOURCE_BLOCK.min(n)`, a `while block_start < n` loop, `collect_into_vec(&mut per_cell)` and a per-block sequential scatter |
| MEM-10 | R4 — `plate_id: Vec<usize>` → `Vec<u16>` | not started | `WorldState::plate_id: Vec<usize>` and `assign_plates(...) -> Vec<usize>` are unchanged |
| MEM-11 | R5 — `jfa_dist`'s three scratch grids to i32/i32/u32 | not started | `jfa_dist` still allocates `sx = vec![-1i64; n]`, `sy = vec![-1i64; n]`, `d2 = vec![0f64; n]` |
| MEM-12 | R6 — the two `with_capacity(n)` heap reservations | not started | `MinHeap::with_capacity(n)` in `build_water_bodies` and `DijkstraHeap::with_capacity(n)` in `road_dijkstra` are both unchanged |
| MEM-13 | R7 — `road_dijkstra`'s discarded `prev` | not started | `let (dist, _prev) = road_dijkstra(cost, gw, gh, …)` is still there; no `want_prev` parameter exists on `road_dijkstra` |
| MEM-14 | R8 — chunk `civ_hierarchical_network_topology`'s parallel Dijkstras | not started | Still collects every settlement's `road_dijkstra` result in one parallel pass; no chunking constant or `chunks(8)` appears in it |
| MEM-15 | Per-segment overlay culling (still open after `_run_offscreen`) | not started | `_run_offscreen` rejects at whole-run granularity only — it takes the run's `PackedVector2Array` and returns a single bool. A long way whose bounding box crosses the window is still walked and dashed in full |

**Group total: 15 — 8 done, 6 not started, 1 declined.**

§6's walk-down table projects 618.28 → 469.56 MiB for all eight R-changes. Only
R1-R3 landed (518.92 MiB), and the "still on the table" status for R4-R8
survives only in one sentence at the very end of a 1 031-line document. It is
recorded properly here.
### LOD and tiling · `LOD_TILING_BASE_SCOPE.md` + `LOD_TILING_INTEGRATION_SCOPE.md`

`ROADMAP.md` files this under "Not a phase" and still says *"revisit when a
concrete need appears rather than building it speculatively"*. **It was built,
it is wired, and it is on screen.**

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| LODB-1 | Base — new crate `cartalith-spatial`: `TiledField`, packed `QuadTree`, `DirtyTracker`, serde round-trip | done | `cartalith-spatial/src/lib.rs` — `TiledField<T>` with `tile_size` as a constructor parameter and the comment defending that; packed `QuadTree<T>` (`Vec<Node<T>>` with index children); `DirtyTracker` with caller-supplied reason strings and a per-tile `version`. All three derive `Serialize`/`Deserialize`. **144 `#[test]`s across eight modules** (the document records 24, from the three modules that then existed) |
| LODB-2 | Integration — the tool system picked the base up (2026-08-18) | done | `cartalith-spatial/src/pass.rs::PassBuffer<S>` and `staleness.rs::StageGraph`, both built on `TiledField`/`DirtyTracker`. **Now depended on by five external crates, not one**: `cartalith-civ`, `-engine`, `-godot`, `-io`, `-terrain` each list it. The crate has also grown five modules the document does not mention: `geo`, `measure`, `paint`, `pyramid`, `region` |
| LODI-M0 | Integration M0 — confirm Z1 needs nothing once the camera lands | done\* | Verification, not new work; the confirming pass is a device measurement — **not checkable from code** |
| LODI-M1 | Integration M1 — a minimal interactive Z2: tile the deep-zoom case only | done | `cartalith-spatial/src/pyramid.rs` (`pyramid_dims`, `pyramid_tile_bounds`, `pyramid_level_for_zoom`, `tiles_in_view`); `cartalith-terrain/src/amplify.rs` (`amplify_region`, `add_zoom_detail`) and `tile_render.rs::shade_tile`; `cartalith-godot/src/lod_bridge.rs` (783 lines); `engine_bridge.gd`'s `lod_level_for_zoom` and `lod_synthesize_tile`; the deep-zoom tile scheduler in `shell/viewport_host.gd` with `shell/lod_tile.gdshader`. The 2026-08-19 bug-fix pass (dropped tiles never reconsidered once the camera stopped) is recorded in the scope document and its fix is in the scheduler |
| LODI-M2 | Integration M2 — nothing; the Data manager export panel, not a new milestone | declined | Declared not a milestone by the document itself. The export panel exists (`shell/data_manager_window.gd`) |
| LODI-M3 | Integration M3 — atlas cache (Z5) | done | `cartalith-io/src/atlas.rs` — `AtlasStore`, `encode_chunk`, `put`, `put_meta`; plus `cartalith-engine/src/bake.rs`. Deferred in the plan until M1 shipped and was kept; both conditions were met |

**Group total: 6 — 5 done, 1 declined.**
Shell surface: `Preferences ▸ Tiles & LOD` ships LOD levels 0-8 and
auto/manual, both landed 2026-08-30 (`5f11b27`, `3338a79`).

### Save file and project archive · `SAVEFILE_COMPAT.md`

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| SF-1 | Reading the HTML app's `.zip` | done | `cartalith_io::load_save`; `tests/golden_parity_real_export.rs`. Same as MVP-7 |
| SF-2 | Writing a save (`ROADMAP.md`'s first "option kept open", closed 2026-08-23) | done | `cartalith_io::write_save` in `crates/cartalith-io/src/save.rs`, `WorldGen::save_project`, and the golden test `tests/golden_parity_save_writer.rs`. File ▸ Save / Save as… / Autosave / Revert / Close project are real controls |
| SF-3 | The project archive as a tree, carrying the whole project | done | `cartalith-godot/src/project_bridge.rs` defines fifteen slots — `entities/{settlements,factions,ways,provinces,continents}.json`, `history/timeline.json`, `annotations/{labels,icons,regions}.json`, `appearance.json`, `vault.json`, `drafts/{paint,sculpt}.json`, `library/{assets,travel}.json` — plus `project_save` and `project_save_with_documents` for caller-owned documents (which is how journeys persist, JP-06/08) |
| SF-4 | `state.erosion` written to saves | declined | Only 2 of 16 keys are modelled by the reference's `loadZip()`, so it is deliberately not written rather than written partially. The limitation is disclosed in `SAVEFILE_COMPAT.md`'s own "Writing a save" section |
| SF-5 | Save compression — the byte-plane shuffle (27-36 % smaller, writes faster) | blocked | Owner decision. Needs a `format_version` bump and a fail-loud marker, **and it ends `SAVEFILE_COMPAT.md` §8's bare-dump promise** |
| SF-6 | Save compression — quantising saved rasters to `u16` | blocked | Owner decision. Lossy; `PARITY_TESTING.md` and `DECISIONS.md` §7a bar it without a ruling |

**Group total: 6 — 3 done, 2 blocked, 1 declined.**
Four save slots are written but not yet read back by a caller; a fifth (saved
measurements) was scheduled by the 2026-08-31 rulings. That was owner question
7 and is now answered by implication.

### Android build and device · `ANDROID_BUILD_SCOPE.md`

Twelve rows. Most of this document's content is device measurement, which this
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
| AND-9 | Positive control that `push_warning` reaches Android logcat | not started | Owed since 2026-08-24 and explicitly not done by the 2026-08-25 pass either. Until it runs, a clean logcat is an *argument* that shell and native library match, not a measurement — precisely the failure mode AND-7 exists to catch. The mechanism it would test is present (`engine_bridge.gd::_has`'s `push_warning`) |
| AND-10 | Landscape / rotation driven over adb | blocked | `project.godot` sets SCREEN_SENSOR (`orientation=6`), which follows the accelerometer and overrides `settings put system user_rotation`; `wm user-rotation lock` works only with auto-rotate off. **Not a code defect** — an interaction between the correct setting and adb. Needs the owner to physically rotate the handset |
| AND-11 | Release keystore / signed release export | not started | `export_presets.cfg` contains `package/signed=true` and **no `keystore/*` entry at all**; every device pass has sideloaded a debug-signed APK. Note the discrepancy this document does not record: `builds/android/Cartalith-release.apk` and `Cartalith-perf-rel.apk` exist on disk and how they were signed is unrecorded |
| AND-12 | APK cruft — development probe/shot scenes ship inside the APK; release profile unstripped | declined | The owner's call, not fixed. Real and still present: **84 `_*.gd` probe/shot scripts** sit at the root of `godot-project/` (counted 2026-08-31; the document says "~100", a 2026-08-25 audit counted 76), plus their `.tscn` and `.uid` siblings, all inside the exported filesystem |

**Group total: 16 — 10 done, 2 unverified, 2 not started, 1 blocked, 1 declined.**

### DCC shell · `DCC_SHELL_SCOPE.md`

Five rows. All complete — but **milestones 1 and 2 cite files that no longer
exist**, which makes them unverifiable at face value even though the capability
survived.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| DCC-1 | GUI milestone 1 — the six-region DCC shell (menu bar, workspace tabs/rail, tool options bar, tool rail, viewport, right dock, status bar) | done, **evidence re-pointed** | `project.godot` `run/main_scene="res://shell/app.tscn"`. `shell/dcc_shell.gd` declares and builds `menu_bar_row`, `tool_options_row`, `left_dock`/`left_dock_body`, `right_dock`/`right_dock_body`, `timeline_row`, `status_row`, plus `_build_menu_bar()` and `_build_rail()`. **The document's evidence is written entirely against `main.gd`/`main.tscn`/`map_overlay.gd` and against a 7→8 menu change producing File/Edit/Generate/Simulate/Render/Assets/View/Help. Neither `main.gd` nor `main.tscn` exists, and the menu bar is now seven menus with no Generate, Simulate, Render or View** (`menus.gd` adds File, Edit, Assets, Data, Preferences, Window, Help) |
| DCC-2 | GUI milestone 2 — the Generate menu's six live stage parameter dialogs (57 controls) | done, **re-homed** | There is no Generate menu and no `main.gd`. The parameter surface moved into the WORLD workspace's ten-stage dock: `shell/workspaces/world_workspace.gd` carries the ten-stage table and builds live parameter rows per stage from the engine's own table (`_build_param_row`, `_build_erosion_passes`, `_build_droplet_erosion`); `cartalith-godot/src/params.rs` is the flat dotted-key API those rows read. The milestone's "no per-stage staleness indicator" decision is superseded by the real staleness slot |
| DCC-3 | GUI milestone 3 — the World Setup dialog (map size, resolution, dimensions, aspect) | done | `shell/new_world_dialog.gd` builds the Extent section, the "Map width & resolution" section (Map width preset / Resolution / Aspect rows) and the derived `Grid` / `Extent` / `Cell size` / `Aspect` readout. `func request() -> Dictionary` is what `app.gd` hands to `bridge.import_heightmap(...)` and to generation |
| DCC-T2 | Tool-track milestone 2 — write `UNIFIED_TOOL_PLAN.md` for real (planning only) | unverified | The deliverable is the document itself. It exists at the repository root, 2 268 lines, with the reference's Sculpt editor read out, the pass-buffer model, the tool-by-tool table and the A-F breakdown |
| DCC-T3 | Tool-track milestone 3+ — the tool system itself | done | *The document says "Milestone 3+ (not yet dispatched)", repeated at the end of the milestone-1 entry. It was dispatched and completed* — this is UTP-A…UTP-F above, all in the tree |
| DCC-P412 | The 412 dp phone migration (geometry from `Cartalith Android Phone.dc.html`, content from Menu Structure v3) | done | `dcc_theme.gd`'s `PHONE_REF_SHORT` and the phone density set; `dcc_shell.gd::_build_phone_menu_bar()` and `_phone_menu: PhoneMenu`; the ☰ side drawer is gone in favour of the domain drill. Probe present: `godot-project/_ph412_probe.gd` |

**Group total: 6 — 5 done, 1 unverified.**

### GUI feature parity · `GUI_FEATURE_PARITY_SCOPE.md`

Eight milestones. **Seven are done and one is superseded by another route** —
the document still reads as an open plan and should be closed out.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| GFP-1 | 1 — Category 1 sweep (10 rows of real-backing-needs-wiring) | done | Row 5 (faction culture-terrain-fit): `WorldGen::civ_faction_terrain_fits()` over `civ_faction_aggregates` + `civ_culture_terrain_fit`; `engine_bridge.gd` wraps it; `faction_roster_window.gd::_build_terrain_fit()` is the panel. Row 7 (GPU toggle): `engine_bridge.gd` sets `param_set("use_gpu", true)` as the shell default with `Preferences ▸ GPU acceleration` able to turn it off, and `gpu_stages_used()` over `WorldGen::get_gpu_stages_used`. Rows 1/9/10 are in the DCC shell (asset import, per-layer toggles in `layers_popover.gd`, click-to-pin in `right_dock.gd`). *The outcome table still records rows 5 and 7 as open, which is harder to spot than a plain to-do because it is presented as a closed milestone's result* |
| GFP-2 | 2 — Category 4 visual-consistency sweep (`PopupMenu` / `Tooltip` / `ScrollBar` entries in `dark_theme.tres`) | partial | `theme/dark_theme.tres` contains **no** `PopupMenu`, `TooltipPanel`, `TooltipLabel`, `VScrollBar` or `HScrollBar` entry (grep → zero hits). **PopupMenu was solved by another route** — programmatically, via `DccWidgets.style_popup()` reached through `DccShell.style_popup()` — so the milestone's largest item is closed elsewhere. **Tooltip chrome and scrollbar grabbers are still Godot stock**; the grabber overrides in `dcc_widgets.gd` are `HSlider`, not `ScrollBar`. The milestone as written targets a resource the DCC shell no longer drives its chrome from |
| GFP-3 | 3 — stale-field tracking | done | `cartalith-spatial`'s `StageGraph`/`DirtyTracker` behind `WorldGen::stale_stages()`; `app.gd::_setup_staleness()` (1 s poll, `Recompute` action in `status_row`) and the corrected header comment recording that the *tools*, not the dials, are what leave staleness behind |
| GFP-4 | 4 — Category 2 small items (heightmap import, GeoJSON export, CPU/memory readout, route-corridor/travel-cost fields) | done | Heightmap import: `WorldGen::import_heightmap` reached from `app.gd` and offered as a live row in `data_manager_window.gd`. GeoJSON export: `geojson_bridge.rs` over `cartalith_engine::geojson::export_geojson`, surfaced as `data_manager_window.gd`'s `export_gis`. Readout: `shell/performance_window.gd`. Travel cost (per-settlement string): `journey_bridge.rs` and `journey_planner_view.gd`. **Fully closed 2026-09-01**: route corridors / travel cost as a *selectable analysis field* shipped — `sample_bridge.rs`'s `LAYER_GROUPS`/`legend()`/`debug_raster()` all carry `corridor` and `travel_cost` ids now, each with a dedicated fixture test proving the ramp is actually reached (`corridor_view_reaches_a_real_pass_and_marks_water_distinctly`, `travel_cost_view_spans_its_ramp_and_marks_water_impassable`), reachable through the existing Layers popover with no GDScript change needed |
| GFP-5 | 5 — Terrain appearance GUI | done | `shell/workspaces/render_workspace.gd` (1 055 lines) builds the appearance groups against `WorldGen::{get_appearance, set_appearance, list_appearance_tunables, reset_appearance}`, owns the ramp editor and preset table, and is composed into CARTO rather than owning a rail button |
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
| RP-S7 | 7 — the nine windows, restyled to the new tokens | not started | `faction_roster_window.gd`, `city_viewer_window.gd`, `place_editor_window.gd`, `world_data_window.gd`, `performance_window.gd`, `vault_window.gd`, `travel_library_window.gd`, `data_manager_window.gd`, `asset_library_window.gd` are all untouched by `c03b43c` except a 5-line change to `place_editor_window.gd`, and carry no new-token pass. The plan itself notes the prototypes do not specify these windows — they keep their implementations and are re-pointed at the new frame |

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
This document's header still asserts *"UI work is now on hold entirely (owner,
2026-08-18)"*. That hold was **lifted later the same day**. This is the third
copy of that stale sentence the project has found — see the record defects below.

### Export · `EXPORT_SCOPE.md`

Five milestones, **all shelved by the owner on 2026-08-25**. The document is
findings only; nothing here is a gap in the ordinary sense.

| ID | Milestone | Status | Evidence |
|---|---|---|---|
| EXP-E1 | E1 — the banded terrain renderer (`ExportBandPlan`, `apply_local_contrast` / `build_grade_influence` splits, `tests/export_bands.rs`) | shelved / not started | The prototype was written, **measured byte-identical at five band heights**, and then deliberately reverted; nothing of it is in the tree. `grep ExportBandPlan\|BandWriter` across `crates/` → **0 hits**; no `tests/export_bands.rs`. `export_raster.rs` still reads `const BAKE_WIDTHS: [i64; 3] = [2048, 4096, 8192];` and still refuses anything outside it |
| EXP-E2 | E2 — the streaming writer (PNG first, BigTIFF second, band-in / file-out) | shelved / not started | §6.2's `BandWriter` sketch was never compiled or run; no `tiff` dependency is referenced from any crate source. `export_raster_png` still renders once into RAM |
| EXP-E3 | E3 — the options struct (one dictionary: width, format, style override, content set, settlement tier) | shelved / not started | `export_raster_png(&self, path: GString, width: i64, tiled: bool)` — three positional parameters, no options dictionary; `export_raster_estimate(width)` reports peak memory only, with no per-format file-size estimate |
| EXP-E4 | E4 — the overlay session (cross-frame begin / band / composite / write / finish) | shelved / not started | `export_raster.rs` routes through `render::bake_rect` only; no `begin`/`finish` session `#[func]`s. `map_overlay.gd` still draws in camera space and is not reachable from any export path. **The one milestone with no reference behaviour to port against** — the reference's own bake draws terrain and nothing else — and §5's constraint stands: a synchronous `#[func]` cannot `await RenderingServer.frame_post_draw`, so it must be a GDScript-driven session |
| EXP-E5 | E5 — the export dialog | shelved / blocked | Blocked on E3, which does not exist. `menus.gd` and `data_manager_window.gd` offer the shipped 2K/4K/8K raster export only; no `export_raster_*` call site in `godot-project/shell/` passes anything beyond `(path, width, tiled)` |

**Group total: 5 — 5 shelved.**
Un-shelving is **owner decision 17** and costs four things in order, listed in
`OUTSTANDING_WORK.md` §5. Codec survey conclusion, kept because it is expensive
to redo: WebP is eliminated at 16 383 px and JPEG XL at its AGPL encoder.

### Gap register · `GUI_GAP_REGISTER.md`

**Read this document as history.** Its ID total was re-counted three times
(123 → 215 → 300) and its A/B/C/D open/closed split was never re-derived once;
a class marker survives on only 54 of 215 rows, so the register cannot say how
many of its own IDs are open. `UNWIRED_FUNCTIONS.md` is the live successor,
re-cut 2026-08-31 against the three-domain shell.

| ID | Section | Status | Evidence |
|---|---|---|---|
| GGR-10 | §10 — the actionable (A) list, twelve ranked rows | done | Sampled rows check out: light theme (PR-13/14) → `dcc_theme.gd`'s `const LIGHT` and `var pal: Dictionary = DARK if _dark else LIGHT`; RD-06/08 → `faction_roster_window.gd`'s `bridge.get_factions()`; JP-13/14 → `journey_planner_view.gd`; CA-05 icon resize → `icon_bridge.rs::icon_handle` + `cartography_workspace.gd`'s `"icon"` drag handler. SH-01 is recorded done-then-withdrawn, which the rail's own header confirms ("Removed 2026-08-24 with the expansion itself") |
| GGR-49 | §49 — headed "REGISTERED, NOT FIXED"; **all three findings are now fixed** | done | KV-04: `shell/vault_store.gd::save_from()` now parses only to validate and splices the engine's own string into the document unchanged (`"store": state`), with a comment naming `entity_id` / `source_modified` and the silent-loss failure — §49's own "stop re-parsing" fix. WW-16: `world_workspace.gd` replaces the stage-06 `gap` string with a note dated "Corrected 2026-08-30 … it was stale on six of its seven claims", and `_build_droplet_erosion(...)` gates on `bridge._has("erode_op")`, i.e. a live `#[func]`. CV-12: `civilization_workspace.gd`'s button is still correctly disabled and its tooltip was rewritten ("the previous wording ended 'the crate has no consumer at all', which was false") to name the real per-line blocker — urban milestones 9, 10 and 13 |
| GGR-58 | §58 / PH-28 — a 16:9 tablet is classified as a phone and its controls are clipped off the screen | done | §58 says "**Not applied here** … this pass has a measurement, not a mandate". **It was applied**: `dcc_shell.gd` now reads `_phone = _touch and (short_side / long_side) < _PHONE_ASPECT_MAX and not _is_tablet_sized(short_side)`, with `const _TABLET_MIN_DP := 900.0` and `_is_tablet_sized()` returning `short_side_px / (dpi / 160.0) >= _TABLET_MIN_DP`. §58 proposed ~600 dp (Android's `sw600dp`); commit `660cbef` records the owner ruling **900 dp** instead, on the 48 dp rail + 400 dp dock chrome-floor arithmetic. Both the status and the number in §58 are wrong |
| GGR-53 | §53 — four registered-not-fixed phone items (`⌕` and `⋮` not in the app bar, bottom-nav/sheet colour literals off by 2/255, tool-options sheet still resident, phone pill upper-cases once) | partial | **The `⌕` half is unblocked and its stated reason is stale**: §53 says "`⌕` has no destination — `menus.gd`'s Edit ▸ Find on map… is a `_todo()` row". It is now `_live(p, "Find on map…", ID_FIND_ON_MAP, KEY_MASK_CTRL \| KEY_F)` backed by `shell/place_search.gd`. The `⋮` overflow still exists as floating phone furniture (`_phone_overflow_pop`), the two colour deltas are unchanged, and `dcc_shell.gd` still calls `phone_fit(tool_options_row, …)` on a resident row rather than presenting a sheet |
| GGR-50 | §50 — six registered-not-fixed items from the OnePlus 6T pass | partial | **The memory item is disclosed, not fixed**: `performance_window.gd` still reports `OS.get_static_memory_usage()`, which excludes the Rust allocations and the ~544 MB of Gfx dev that `dumpsys meminfo` counts, with a note naming the source and §50's own handset figure (0.2 GB on screen vs 818 MB PSS). The other five (Label ellipsis hole, DS-12 duplicate class token, navpad hover tint, stock focused pane-switcher chip, two `✕` on the L2 sheet header) were not re-verified individually this pass |
| GGR-51 | §51 — the registered-not-fixed menu-conformance set | partial | Mixed, and worth splitting. **Settled by a newer authority**: the tool-options bar height, now `--tbH` 40/56 in the 2026-08-31 spec, superseding §51's "third owner decision". **Disclosed rather than fixed**: `tool_bar.gd` builds the freehand row from `bridge.get_sculpt_freehand_modes()` live and adds the note "The canvas's Flatten, Noise and Mask have no engine mode at all; these eight are `FreehandMode`'s own list, read live." **Unchanged**: the badge right-column (a real Godot `PopupMenu` limitation), the tablet dock contents (that is DS-03), and the sculpt feature-name gap (an engine gap). Two rows §51 listed were already proven stale by §53's own probe |
| GGR-DS03 | DS-03 — the tablet interior is desktop-sized; the tablet artboard is a content decision, not a scaling layer | blocked | An owner **content** decision — *which controls leave the tablet* — not answerable from the canvas. Under it sits a real architectural blocker: `dcc_theme.gd`'s `const TABLET := {36: 52, 34: 52, 30: 44, …}` is keyed by the bare desktop integer, and the artboard maps one desktop figure to two tablet figures in at least five places, so a value-keyed table cannot express it; §57 also refuted the obvious role-keyed placement. One mechanism claim in §51 has drifted: `phone_fit()` no longer opens `if not _phone: return` — the gate moved to its callers — but the **verdict** DS-03 rests on is still correct |
| GGR-DS13 | DS-13 — the phone viewport control column (navpad, zoom, layers FAB) does not match the canvas | not started | §57's pass produced a design and its own audit refuted it on four high-severity counts. Two re-confirmed here: `viewport_host.gd`'s left-button branch is `elif mb.button_index == MOUSE_BUTTON_LEFT and _pan_mode:` with no armed-tool condition (so `tool_pan` is navigation after all), and `_build_navpad()` opens `if not _touch: return`, so touch tablets get the same pills. `menus.gd` has no View menu, confirming that zoom's stated destination does not exist. **Nothing was built and the register says so** |
| GGR-RELIG | The religion-diffusion screens (§57) — designed, audited, refuted as premature | blocked | **No data path.** `get_settlements()` emits x, y, name, population, kind, faction, capital, coastal, tid — no religion field and no adherent counts. The only religion surfaces are faction-level (`civ_religion_vocabulary()`, the roster's `has_religion_flags()`). `cartalith-civ/src/belief.rs` exists but has no settlement-level bridge — and, as recorded above, no consumer at all. The screens cannot be designed before RD-1 exists |
| GGR-05 | §5 — omissions: designed, not present, not even as a disabled item | unverified | Not re-verified item by item. §37, §39, §42 and §45 close large parts of it and §5 was never rewritten to reflect that, so its current accuracy is unknown. Confirming each absence requires the canvas alongside the shell — **flagged rather than guessed** |
| GGR-BULK | §15-§56 — the fixed/closed body of the register (RF-01, SB-01, BK-01/02, IN-10/11/12, CA-13, SH-01, MT-01, RD-01/01b/02, MR-01…03, TO-01/02, CV-20/23/25/26, MN-09, SH-15, KV-01…03, FR-02, PE-01, SH-11, WW-13, IN-13, VA-01, ED-02, MN-10, RL-01, CA-20, RF-02…05, FI-04, PH-12…PH-27, HD-01…04, DS-01…14, MEM-01…04, FX-01…03, BI-01) | done, **spot-verified only** | Every spot check held: SH-01's withdrawal is recorded in `dcc_shell.gd`; MEM-02's culling fix and the LOD backlog are in `viewport_host.gd`; the light theme is `dcc_theme.gd`; the 412 dp phone set is `dcc_shell.gd`'s phone half plus `_ph412_probe.gd`. The named probe harnesses are all present in `godot-project/` (`_rf01_probe`, `_backnav_probe`, `_in13_probe`, `_phonechrome_probe`, `_tabletparity_probe`, `_hidpi_probe`, `_flowzoom_probe`, `_cull_probe`, `_menuconf_probe`, `_railalign_probe`). **Not individually re-derived** |

**Group total: 11 — 3 done, 3 partial, 2 blocked, 1 not started, 1 unverified,
1 done-spot-verified.**

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

**264 milestone rows across 29 subsystem groups**, counted from the tables
above. Shares are rounded and do not sum to 100.

| Status | Count | Share |
|---|---:|---:|
| **done** | 174 | 66 % |
| **not started** | 41 | 16 % |
| **declined** (deliberate, with the reason in code or a ruling) | 16 | 6 % |
| **blocked** (a named blocker) | 13 | 5 % |
| **partial** | 10 | 4 % |
| **shelved** (owner, 2026-08-25 — all of `EXPORT_SCOPE.md`) | 5 | 2 % |
| **unverified** (not a code artefact) | 5 | 2 % |

**Where the 41 not-started rows are.** Seven subsystems hold 33 of them; the
other eight are singletons and pairs across Economy, Story planning, Android,
the gap register and the two options kept open. **Journey Planner dropped out
of this list 2026-09-01**: JP-QC4 (`jp_road_cells`/`ManualWay`) was its only
not-started row and is now done, alongside JP-QC2 and JP-QC3 — see the Journey
Planner ledger above.

| Subsystem | Not started | Note |
|---|---:|---|
| Urban morphology | 7 | Milestones 8, 9, 10, 11, 13, 14, 15 — the largest block in the project |
| Religion diffusion | 7 | RD-1…RD-7; the foundation shipped, nothing above it |
| Memory optimisation | 6 | R4-R8 plus per-segment overlay culling — all small |
| Markdown Vault | 3 | Map snapshot, project-scoped links, Android SAF — Compare view shipped 2026-09-01 |
| Sculpt live | 4 | L0 gates the rest; L3 is declined by design |
| GUI replacement | 4 | Stages 3, 5, 6, 7 — **all unblocked** |
| GPU layer integration | 2 | Both are the document's own named deferrals |

**Where the 13 blocked rows are.** Seven of them trace to just three open owner
decisions: conflict attachment (SP-2, SP-4, SP-5, LM-9), the viewshed budget
(LM-7), and save compression (SF-5, SF-6). **Answering decision 1 alone unblocks
three rows across two documents** — SP-4 directly, LM-9 which names SP-4 as its
blocker, and SP-5 which needs two of SP-1…SP-4. The other six are blocked on
a memory decision (EC-8), the era-table recalibration (MM-F2), urban
milestones 8-15 (UM-16), hardware (AND-10), an owner content decision
(GGR-DS03) and a missing data path (GGR-RELIG). **JP-QC2 dropped off this list
2026-09-01** — no longer blocked on `cartalith-engine` retention work it never
actually needed; see the Journey Planner ledger above.

**Read the `done` figure carefully.** 66 % of rows done is not 66 % of the
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
golden suites are the actual correctness bar and there are **88
`golden_parity_*.rs` files** in the workspace; whether they currently pass is
not recorded here. One test is known intermittent —
`generate_terrain_gpu_path_is_deterministic_and_valid` in
`cartalith-engine/src/lib.rs` fails roughly one run in three under full-workspace
parallel load, by about 1 ulp; whether an `assert_eq!` on a whole f32 field is
the right bar for a path `DECISIONS.md` §7a holds only to principled equivalence
is an open owner decision, not a result.

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

This is the defect class that caused this rewrite. Five instances survive:

| Document | What it still says | What the code says |
|---|---|---|
| `ROADMAP.md`, "Options kept open" | Landmark generation was imported and cataloged 2026-08-30 with "**no code written**" | `landmark.rs` is 3 730 lines with `generate()`, ten `#[func]`s, a `landmark_store` field on `WorldGen`, 49 glyphs and a CIVIL ▸ Landmarks panel — all landed the same day |
| `LANDMARK_GENERATION_SCOPE.md` §0, §3 | "**No code was written for this pass.**" / "**Nothing below is started.**" | Seven of nine milestones are substantially built (LM-1…LM-6, LM-8) |
| `ROADMAP.md`, "Not a phase: LOD" | "Revisit when a concrete need appears rather than building it speculatively" | A tiled deep-zoom pyramid with a persistent chunk atlas is shipping and is on screen — `pyramid.rs`, `atlas.rs`, `lod_bridge.rs` (783 lines), the `viewport_host.gd` scheduler |
| `CPU_MULTITHREADING_SCOPE.md` | "`cartalith-gpu` currently only ever requests a single `PowerPreference::HighPerformance` adapter … the integrated GPU is never enumerated or used at all, for anything" | `multi.rs::enumerate_devices()` walks every adapter and `GpuDeviceSet` opens more than one |
| `GPU_LAYER_INTEGRATION_SCOPE.md` m6 | `use_gpu` "stays off by default and unexposed in the UI"; "generating a new map today still runs on CPU by construction" | `engine_bridge.gd` does `param_set("use_gpu", true)` in `_ready()`; `Preferences ▸ GPU acceleration` ships with `GPU_TOGGLE_TIP` |

### The stale UI hold — the third and fourth copies

The UI hold called by the owner on **2026-08-18 was lifted later the same day**.
`CLAUDE.md` carried the stale version until 2026-08-23, when `PARITY_AUDIT.md`
caught it. Two more copies are still in the tree and are cited as live blockers:
`GUI_SHELL_SCOPE.md`'s header ("UI work is now on hold entirely"), and
`UNIFIED_TOOL_PLAN.md` / `STRANDED_TOOLS.md` ("all UI work is on hold"). Three
scope-document milestone entries rest their "not wired" verdict on it
(PHASE2 m20, ECONOMY's `civ_culture_terrain_fit`, TERRAIN_APPEARANCE m1) and all
three are wrong in code.

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
2. **`cartalith-godot/src/vault_bridge.rs`** still asserts that "`cartalith-io`'s
   save format … carries no civ data at all". That describes the retired
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

- **`PHASE2_SCOPE.md` m17** — "Not yet wired anywhere — no real caller exists",
  then "Resolved same day" four lines down. Both sentences stand.
- **`ASSET_LIBRARY_SCOPE.md` §9** — enumerates eight gaps that §10 and §11 later
  close. Nothing marks §9 as a 2026-08-19 snapshot.
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

- `ROADMAP.md` Phase 4 says "all seven milestones"; the **eighth** (the
  sprite-sheet slicer) landed 2026-08-20.
- `ROADMAP.md` Phase 0 says the skeleton "builds and runs on all three targets";
  `export_presets.cfg` defines **two**, and the same file calls the third
  (WASM) uncommitted.
- `URBAN_MORPHOLOGY_SCOPE.md` gives the `_um*` denominator as **20** in one
  place and **28** in another.
- `UNWIRED_FUNCTIONS.md`'s 2026-08-31 headline **77** double-counted two rows
  its own "fixed during the audit" section closed; 75 were genuinely open at
  that cut, and its Large section heading read "(16)" where the intro said 18.
  **Historical**: the 2026-09-01 re-cut was written from scratch against the
  tree rather than patched, closed 52 of those 75 rows, and carries
  internally-consistent counts throughout (18 Large in both the heading and
  the running total; see the Tool system and GUI feature parity rows above for
  two of the closures) — 23 rows remain open.
- `LOD_TILING_BASE_SCOPE.md` records 24 unit tests and one dependent crate;
  there are **144** tests across eight modules and **five** external dependents.
- `CPU_MULTITHREADING_SCOPE.md`'s rayon census is four crates stale — see the
  re-measured figures in that group.
- `ANDROID_BUILD_SCOPE.md` says "~100" probe scenes; a 2026-08-25 audit counted
  76; **84** `_*.gd` files sit in `godot-project/`'s root today.
- `LARGE_ITEM_RULINGS.md` says the 3D research "stands complete at 1 486 lines";
  it is longer. Drift inside a single day.

### The reference freeze has drifted

`CLAUDE.md` and `FUNCTIONAL_CONTRACT.md` both assert the frozen v2.10 is the
live repository's latest and that there is "no re-freeze question to raise".
**`Cartalith Gen1 v2.11.html` is committed at this repository's root** while
`reference/` holds only v2.10 and `FUNCTION_INDEX.md` indexes v2.10's 1 094
functions. This is the one stale sentence in this section that is also **real
outstanding work**: `CLAUDE.md` requires the index be regenerated in the same
pass as any re-freeze. It is listed in `OUTSTANDING_WORK.md` §2.8.

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
