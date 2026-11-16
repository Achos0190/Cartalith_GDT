# Codebase inventory and crate refactor: the proposal and the prompts (written 2026-11-16, for later)

**Not to be started now.** The owner's order (2026-11-16): finish the current outstanding list first; then *"a proper
inventory of the codebase and refractor the crates so all similar systems get to be optimised."* This document is the
ready-to-use plan: the phases, the rules, the verification gates, which agent does what, and the exact prompts.
It defines; it does not track (status goes to `cartalith-native/docs/STATUS.md`).

---

## 1. Goal and non-goals

**Goal.** One optimised implementation per kind of system. Today the same idea is often written several times across
crates (several flood fills, several distance transforms, several per-cell grids and samplers, several tile caches,
several noise and blur paths, several JSON document stores, GDScript widgets rebuilt per panel). After the refactor:
a shared, tested, annotated core per kind; callers use it; dead and duplicate code is gone; the crate graph is clean.

**Non-goals.** No new features. **No behaviour change** -- every world, every golden, every save and every rendered
frame stays byte-identical unless an owner ruling says otherwise (a discovered bug is reported and routed, not fixed
inside a refactor step). No change to `reference/` or `html/`. No rewrite of the GDScript shell's design.

## 2. The rules every agent carries (put them in every brief)

1. Read `MISTAKES.md`'s preflight table first and again before declaring green; apply every matching row.
2. **Refactor = proven identical.** Each step lands only when its proof gate (section 5) passes. Never "looks the same".
3. **§7p** (`DECISIONS.md`): the reference baseline (`WorldParams::defaults`, `js_reference`) and the app defaults
   must both stay byte-identical; JS goldens never move. Moving a golden is a stop-and-report.
4. **JS semantics**: keep `cartalith-jsmath` (`js_hypot`, `js_exp`, `js_min/max`) wherever it is used; never "improve"
   float order or precision (`cartalith-rust-conventions`).
5. **GPU determinism**: generation kernels stay integer/exact (Ruling EZ item 3); never merge a float GPU path into a
   generation path.
6. **Ruling BK**: every new or moved function, type and constant is annotated (what, why, what it must never do,
   sources); fix prose a move makes stale.
7. Never `git stash`; never force-push; push only to `main` and `claude/cartalith-rust-godot-setup-lhgtgh`; explicit
   paths when committing; fresh worktrees under `C:/cw/` from `origin/main`; own `CARGO_TARGET_DIR`; remove both after.
8. Searches stay inside the Cartalith folder or `C:/cw`. Scratch APPDATA for windowed probes; the window lock
   (`mkdir C:/cw/.win_lock` + owner file; never delete another's).
9. Do not stage `project.godot`, `.import`, `.uid`, or `MISTAKES.md` unless the step is about them; a new `class_name`
   needs `godot --headless --import` and a check that `project.godot`'s `;` comments survived.
10. Agents do not spawn agents (the coordinator runs verifiers); the standing cap applies (up to 3 Sonnet, or 2 if any
    is Opus or Fable).

## 3. Skills to load

| Skill | When |
|---|---|
| `cartalith-porting-discipline` | any step touching ported reference code, crate boundaries, goldens, a new crate |
| `cartalith-rust-conventions` | every Rust step (float precision, op order, NaN, panics across the gdext boundary) |
| `rust-craft` | API design of the shared cores, error types, iterators, clippy/fmt |
| `godot-shell` | any GDScript, `.gdextension`, export or shell step |
| `ponytail` | the design review: the simplest shared core that serves all callers; refuse speculative generality |
| `engineering:tech-debt` / `engineering:architecture` | the inventory report and the ADR for the target crate graph |
| `code-review` (high) | the coordinator's review of each landed step (owner opts into max/ultra only explicitly) |

## 4. Phases

**R0 Inventory (read-only).** Census every crate and the shell: modules, public items, dependencies, LOC, test counts,
`#[ignore]` harnesses, dead code (`cargo udeps`-style and `rg` for unused pub items), duplicated algorithms
(near-identical function bodies, same algorithm under different names), hot paths (from `PERFORMANCE_BENCHMARKS.md` and
`device_stage_bench`). Output: `CODEBASE_INVENTORY.md` with a **similarity table** (family → every implementation →
callers → differences that matter: precision, tie-break, NaN, order) and a dependency graph.

**R1 Design (read-only).** For each family: the target shared core (crate, module, signature), which callers move,
which differences are real (must be parameters) and which are accidental (bugs to report, not fix), and the proof gate.
Output: an ADR per family plus `CRATE_GRAPH_TARGET.md`; owner questions for anything that would change behaviour.
**Owner approval before R2.**

**R2 Staged moves.** One family per step, smallest blast radius first. Each step: introduce the core with its own tests,
move callers one at a time, delete the duplicates, run the proof gate, land. No step mixes two families.

**R3 Optimise.** Only after a family is unified: profile its core, optimise once (parallelism, allocation, cache),
prove identical, measure the gain on PC and 6T.

**R4 Close-out.** Update `ARCHITECTURE.md`, `README.md`'s crate table, `STATUS.md`; retire stale docs; a final
independent review.

## 5. Proof gates (what "identical" means)

| Gate | Command / check |
|---|---|
| Workspace tests | `cargo test --workspace --no-fail-fast` -- same pass count, no new failure (known reds named) |
| JS goldens | every golden test byte-identical; a moved golden stops the step |
| World digests | `device_stage_bench` CPU digest (e.g. `terrain=632b22edf25908e1 civ=…`) identical; GPU-path digest identical |
| Baselines | `fh_r2_baseline`, `g1_baseline` (and other `#[ignore]` harnesses) pass unchanged |
| Saves | an old save opens and re-saves byte-identical (`SAVEFILE_COMPAT.md`) |
| Rendered frames | fixed-seed frame sets (desktop, phone) pixel-identical before/after (`_viewerframe_probe`, `_declutter_probe`, `cel_toon`/`export_raster` census) |
| Shell | the shell's probes green on desktop and phone forms; `--check-only` on touched scripts and `shell/app.gd` |
| Mutation | the shared core's constants and branches mutation-tested (literal mutants; survivors explained) |
| Performance | R3 only: before/after medians on PC and 6T (`MISTAKES.md` timing rules) |

## 6. Who does what

| Phase | Agent | Why |
|---|---|---|
| R0 census (mechanical counting, `rg` sweeps, dependency graph) | **Sonnet** ×2-3 in parallel, split by crate group | cheap, reliable, parallel |
| R0 similarity judgement and R1 design | **Fable** as lead architect, **Opus** as second opinion | deep cross-crate reasoning; Fable judges what is "the same system" |
| R2 moves | **Opus** for engine cores (erosion, hydrology, fill, grids, civ); **Sonnet** for shell/GDScript and mechanical caller moves | correctness-critical vs clear-spec |
| Verification of each step | a **fresh** Sonnet or Opus verifier, not the builder | independent eyes; MISTAKES rule |
| R3 optimise | **Opus** | measurement plus subtle determinism |
| Final review | **Fable** | owner's preferred judge |

## 7. The prompts

### 7.1 Coordinator kick-off (paste to a fresh session when the time comes)

> Read `CLAUDE.md`, `MISTAKES.md`, `SESSION_HANDOFF.md`, `cartalith-native/docs/STATUS.md`, and
> `CODEBASE_REFACTOR_PROPOSAL.md`. Load the skills `cartalith-porting-discipline`, `cartalith-rust-conventions`,
> `rust-craft`, `godot-shell`, `ponytail`. Run phase R0 exactly as the proposal describes: launch the census agents
> per section 6 with the rules of section 2 in every brief, then hand the census to Fable for the similarity judgement.
> Change no code until the owner approves R1's design.

### 7.2 Fable: lead architect for R0 judgement and R1 design

> You are the lead architect for Cartalith's codebase refactor (the owner's world-building app: a Rust workspace in
> `cartalith-native/crates/` plus a Godot 4.7.2 GDScript shell in `cartalith-native/godot-project/`). **Read-only: change
> no code; write documents only.** Do not spawn agents.
>
> Read first: `CODEBASE_REFACTOR_PROPOSAL.md` (this plan; follow its rules in section 2), `CODEBASE_INVENTORY.md` (the
> census just produced), `ARCHITECTURE.md`, `DECISIONS.md` (especially §7p), `MISTAKES.md`, `JS_SEMANTICS_AUDIT.md`,
> `PERFORMANCE_BENCHMARKS.md`. Read code at the symbols, not from docs; prefer naming symbols over line numbers.
>
> Task:
> 1. **Judge similarity.** For every family in the census's similarity table, decide which implementations are truly
>    the same system, which differ for a real reason (precision, tie-break order, NaN handling, JS parity, GPU exactness,
>    reference fidelity) and which differ by accident. Evidence per claim: a symbol and the difference that matters.
> 2. **Design the target.** Per family: the one shared core (crate, module, public signature, parameters that carry
>    the real differences), the callers that move, what is deleted, and the proof gate that shows identity (section 5).
>    Apply `ponytail`: the simplest core that serves every real caller; no speculative generality.
> 3. **Order the work.** Rank families by (duplication removed + performance headroom) / (risk + blast radius); the
>    first steps must be low-risk and high-confidence.
> 4. **Flag behaviour changes.** Anything that cannot stay byte-identical becomes an owner question, not a step.
>
> Write `REFACTOR_DESIGN.md` (repo root): a verdict summary; one section per family (same-or-not with evidence, target
> core, callers, deletions, gate, risk); `CRATE_GRAPH_TARGET.md` (today's crate graph vs the target, with every move);
> the ordered step list; owner questions with defaults. Report the path, the top ten families by value, and the questions.

### 7.3 Builder brief template (R2, one family per agent)

> Refactor step R2-<n>: unify <family> into <target core> per `REFACTOR_DESIGN.md` §<x>. Fresh worktree from
> `origin/main` under `C:/cw/`, own `CARGO_TARGET_DIR`. Rules: `CODEBASE_REFACTOR_PROPOSAL.md` section 2 (paste them).
> Skills: `cartalith-rust-conventions`, `rust-craft` (+ `godot-shell` if GDScript). Steps: add the core with its own
> annotated tests; move callers one at a time, running the gate after each; delete the duplicates; run the full proof
> gate of section 5 and report every line of it (counts, digests, golden status, frame diffs). If anything is not
> byte-identical, stop, revert your change, and report the difference with symbols -- do not "fix" it. Commit by explicit
> paths with the session trailer; rebase; push to the two branches only. Do not spawn agents.

### 7.4 Verifier brief template

> Independently verify refactor step R2-<n> (`<commit>`). Do not trust its report. Re-run the section 5 gate from a fresh
> worktree; diff the deleted implementations against the core for any lost case (a tie-break, a NaN path, a JS helper
> dropped); mutation-test the core; check Ruling BK annotations and stale prose. Report confirmed / refuted per claim.

## 8. Families already visible (starting hypotheses for R0, not conclusions)

Depression filling and flow routing (several floods across erosion, hydrology, civ and the zoom-refine harness);
distance transforms and coast SDF; grid/cell samplers and bilinear/bicubic upsamplers; blur and box filters (CPU and
GPU mirrors); noise and octave sums (terrain amplify, render grain, zoom detail); tile caches and stores (atlas,
LOD RAM caches, plain-tile cache); JSON document stores in `annotations/` and `entities/` (POIs, landmarks, labels,
paint); pathfinding (Dijkstra/A* for roads, routes, planner, sea lanes); polygon and polyline utilities in urban and
civ; the GDScript widget builders (sliders, chips, dialogs) and the probe harness boilerplate (`_frames`, `_shot`,
`_ok`, window lock). Each needs R0's evidence before it is treated as one family.

## 9. Cost and pacing

R0 about a day of agent time (parallel Sonnet census + one Fable pass); R1 a Fable pass plus owner review; R2 one family
per batch with its verifier; R3 per family after unification. Run under the standing agent cap; budget checks per the
owner's usage readings.
