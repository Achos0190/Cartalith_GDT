# MISTAKES.md

**This file is preemptive. Read the preflight table before you act, not after
something breaks.** The entries below it exist to explain *why* each rule is
there — every one is a mistake that actually shipped into this tree and was
caught by measurement, a verifier, or the owner. The `×N` is how many separate
times it has been made, which is the best available guide to what will go wrong
next.

Add an entry after a confirmed mistake or a user correction. Keep entries short,
specific, and project-related. Merge repeated mistakes rather than duplicating
them — raise the `×N` and add the new instance to the evidence. Do not record
transient tool failures or unverified guesses.

---

## Preflight — find your trigger, apply the rule

Scan the left column for what you are **about to do**. If a row matches, apply
its rule before you start.

| About to… | Rule | Verify with |
|---|---|---|
| **Schedule or act on a backlog row** | Re-open it at its **cited symbol** first. Rows here go stale within a day; "already done" is a valuable finding, not a failed task | Name the symbol you opened, not the row's text |
| **Return, marshal or default a value that can be absent** | **Omit the key**; callers use `has()`. In UI, dash the field **with its reason** | Render over real data; count rows showing a bare `0`, `1.0`, `"none"` or empty |
| **Write a test that pins a constant** | Assert a **literal**, or the independent thing the value must equal. `assert_eq!(x, THE_CONSTANT)` holds for every value of it | Mutate the constant; the test must go red |
| **Change behaviour** | Hunt the prose that described the old behaviour — comments, tooltips, `_todo` reasons, **and the row's own cited location** | Grep the touched files for the old behaviour's vocabulary |
| **Add a capability, or write a staleness/cache key** | Derive the list from the **definition** — every consumer path, every `match` arm, **every argument of the function you are guarding** | Exercise each in turn; change each input and confirm the dependent thing reacts |
| **Quote a timing or a benchmark** | Median with min..max. Run the harness **alone**, never under a parallel suite. No point estimates in doc comments | Re-run independently; if it moves beyond the stated spread it was not a measurement |
| **Write "measured" anywhere** | Only for a number you **just produced**. A figure from elsewhere is cited, not asserted | Reproduce it from the command shown before it enters a decision record |
| **Declare the work green** | `cargo test --workspace` — never a crate subset — **and** parse-check every `.gd` you touched plus `shell/app.gd` | The Rust suite cannot see a broken shell; both checks are required |
| **Test behaviour a unit test cannot reach** | Use a probe scene: `godot --headless <probe>.tscn`. `WorldGen` is a cdylib `GodotClass` and cannot be constructed in a unit test | The probe runs and asserts, not just parses |
| **Run `godot --headless --import`** | Expect it to strip `project.godot`'s `;` comments. Avoid the import if you can | `git diff …/project.godot` must be empty; `git checkout --` if deletions only |
| **Mutation-test** | Python only: exact literal replace, occurs-exactly-once assertion, BUILD_ERROR classified separately from SURVIVED, **restore in a `finally`**. Never `sed`. Never a whole-file backup | Grep zero MUTANT residue; hash the file against its pre-run value |
| **Call a shell helper** | Grep for `func <name>` first. In `menus.gd` the vocabulary is `_todo` / `_readout` / `_signpost` / `_live`, and the wrong one has consequences | `godot --headless --check-only` on the file |
| **Change generated output** | Divergence ships `false` in `cartalith_engine::WorldParams::defaults()`, `true` in `cartalith_godot::params::defaults()`; new params need `PARAMS` + `JS_PATHS` rows. **A golden re-baseline needs an owner ruling** | Hash the same render before and after; identity by control flow beats identity by arithmetic |
| **Dispatch agent lanes** | One brief per lane, checked before launch. Serialize lanes sharing a file rather than forbidding the edit. Tell every lane to **report** false prose in files it does not own | Re-read each prompt for a foreign lane's heading |

---

## Why each rule exists

### [2026-09-03] Believing a backlog row instead of re-opening it ×11

**Mistake:** Work was scheduled, and briefs written, against rows already done or
whose blocker had lifted. Seven described built work (the manual-icon tool's
three gaps, CA-05's resize handles, the layer sync, `_civSaltAccess`); four
described lifted blockers — the "no JS runtime" claim was false for 18 days and
cost a whole dispatched wave. The Religion row claimed `cartalith-civ::belief`
does not exist; it is 945 lines.

**Root cause:** Treating `OUTSTANDING_WORK.md` as state rather than as a router.

**Prevention:** Re-open at the cited symbol before acting. Find the symbol, not
the line — line numbers drift daily here.

**Verification:** The report names the symbol opened, not the row's text.

---

### [2026-09-03] Encoding "no value" as a plausible value ×5

**Mistake:** `Option<f64>::None` → `0.0` where `0.0` is a legal Crowding (44 of
614 rows read as real); `harbour_scale` defaulting to `1.0` and printing as
though measured; `wall_spec` defaulting to `"none"`, identical to a real
`"none"`; `VRAM budget: 0.0 GB` where `0` means *no cap* **and is the shipping
default**; `float(<null>)` in a GDScript reader, which is a runtime error that
aborted the whole document and silently cleared the user's saved set.

**Root cause:** Reaching for `unwrap_or` / `get(k, default)` to avoid handling
the absent case, when absence is information the reader needs.

**Prevention:** Omit the key; callers use `has()`; dash with the reason.

**Verification:** `grep -nE "unwrap_or\(0\.|get\(.*, ?0\.0\)|get\(.*, ?1\.0\)"`
over the diff, then render over real data and count.

---

### [2026-09-03] Leaving prose that describes the old behaviour ×6

**Mistake:** Controls disabled by reasons that had become false; `render.rs`'s
module doc listing `rockSlope` refinement as **excluded** in the file that had
just implemented it — and that doc is the row's own cited location; `STATUS.md`
naming three deleted probes as "present and uncalled"; a `_todo` false in every
clause; two chips citing capabilities built hours earlier; a panel formula
inverted against its own engine, agreeing only at the default.

**Root cause:** Behaviour and the prose describing it live apart; one gets edited.

**Prevention:** A stale comment is a defect, not a nit. **A reworded reason that
is still false is worse than the original — it looks freshly checked.**

**Verification:** Grep the touched files for the old behaviour's vocabulary.

---

### [2026-09-03] A test that compares a constant against itself ×2

**Mistake:** `assert_eq!(e.brush.r, ICON_BRUSH_R_MAX)` and
`assert!(err.0 < MIN_REGION_WORLD_AXIS)`. Six constants and an RNG seed survived
mutation with the suite green.

**Root cause:** The assertion was written from the implementation, so it
restated the code instead of pinning it.

**Prevention:** Assert literals, or the independent thing the value must equal —
`generate_sized`'s own `grid_w.max(4)`, the reference's `min="2" max="60"`.

**Verification:** Mutate it and watch the test go red.

---

### [2026-09-02] Declaring green without both checks

**Mistake:** (a) Commit `0f0fe55` used an undeclared `_label_cull` in a `.gd`
file: `cargo test` said **2 821 passed, 0 failed** while `shell/app.gd` — the
application root — would not compile and the app could not boot. (b) After
flipping a default, verified five crates, declared green, and had broken **16
`cartalith-civ` golden suites**.

**Root cause:** The Rust suite and the Godot shell are separate compilation
domains, and only one is in CI. Blast radius was reasoned about, not measured.

**Prevention:** `cargo test --workspace` **and** parse-check every `.gd` you
touched plus `shell/app.gd`. Ship divergence behind the app-boundary pattern so
goldens stay bit-identical.

**Verification:** Paste the summed total line; grade `.gd` only on stderr
containing "Parse Error" / "Failed to load script" — the exit code is unreliable.

---

### [2026-09-03] Covering some inputs of a thing, not all of them ×3

**Mistake:** `with_ground_tiles` reached the on-screen builder but not
`export_raster.rs`, so the map blended the pack tile and **every exported PNG
blended the flat swatch**. The Units formatter reached one `match` arm of
`_measure_readout()` and not its siblings — "135.0 nm" in one mode, "r 250 km"
in another. The belief layer's staleness key covered `belief_seed`'s second
argument and not its first, so reassigning a settlement to a faction of another
faith left it showing the old religion while the guard reported itself current —
**and that was the fix for the identical miss on the religion column**, made one
lane earlier.

**Root cause:** Enumerating from the case in front of you rather than from the
thing's own definition — its consumers, its `match` arms, or its signature.

**Prevention:** Derive the list from the definition, not from memory. For a
capability, grep every builder/consumer. For a staleness key, **read the
function's signature and cover every argument** — that is why the belief key now
names `belief_seed(faction_of, faction_religion)` in its own doc. Divergences
that move no pixel at the default are invisible to the suite.

**Verification:** Exercise each consumer / arm / argument, not the one you
edited. Change each input in turn and confirm the dependent thing reacts.

---

### [2026-09-03] Single-sample timings written as measured fact ×3

**Mistake:** The lane that closed *"average the benchmark over multiple runs"*
then wrote three single-sample figures into two doc comments and a scope
document. A 416 ms handshake re-measured at **730 ms**; a "5× spread" at
**1.4%** (contaminated by parallel `cargo test` contention); a "halving" at
1.35×.

**Root cause:** A number from a real run feels like a measurement even when it
is one sample from a noisy device under contention.

**Prevention:** Median with min..max, or state the direction not the factor. Run
timing harnesses alone. No point estimates in doc comments.

**Verification:** Re-run independently; compare against the stated spread.

---

### [2026-09-02] Claiming a measurement that was never run

**Mistake:** Told agents that damping `impact_field` would fail sixteen
`cartalith-civ` golden suites, "measured, not hypothetical". It reaches the civ
layer nowhere; the crate passes 27/27. The false figure propagated into a source
comment and into `DECISIONS.md` §7l-ii as "measured history".

**Root cause:** Carrying a real figure from an adjacent change and restating it
as if re-measured.

**Prevention:** Never write "measured" for a number you did not just produce.

**Verification:** Reproduce from the command shown before it enters a decision
record.

---

### [2026-09-03] Mutation tooling that corrupts the tree ×3

**Mistake:** (a) A `sed` script silently failed on patterns containing `*` (BRE
quantifier) and reported them SURVIVED, and scored a non-compiling crate as
SURVIVED. (b) An agent's whole-file `.mutbak` was snapshotted while a *different*
agent's edit was live; its restore overwrote that agent's source. (c) A script
died mid-run on a path error and **left the mutant in the source**.

**Root cause:** Treating mutation testing as a text operation, not a transaction.

**Prevention:** Python only; exact literal replace; occurs-exactly-once
assertion; BUILD_ERROR separate from SURVIVED; restore in a `finally`. Never
`sed`, never a whole-file backup.

**Verification:** Grep zero MUTANT residue; hash the file against pre-run.

---

### [2026-09-03] Using a helper that does not exist ×2

**Mistake:** Wrote `_link(...)` in `menus.gd` and `_group_thousands(...)` in
`dcc_units.gd`. Neither existed.

**Root cause:** Assuming a convention by analogy with a sibling file.

**Prevention:** Grep for `func <name>` before calling it. In `menus.gd`, a
`_todo` where a `_signpost` belongs makes `command_index.gd` count a shipped
feature as missing.

**Verification:** `--check-only` on the touched file.

---

### [2026-09-03] `--import` strips `project.godot` comments

**Mistake:** An import removed **83 lines** of `;` comments — the block
explaining why `orientation=6` is an int not a string, and why there is
deliberately no `stretch/mode` key. No key changed, so nothing failed.

**Root cause:** The variant parser rewrites the file without preserving comments.

**Prevention:** `project.godot` is off limits. If an import is unavoidable,
diff afterwards **every time**.

**Verification:** `git diff …/project.godot` empty; `git checkout --` if the
only change is deletions.

---

### [2026-09-03] Orchestration errors that waste a wave ×4

**Mistake:** Pasted one lane's brief into another's prompt *and* dispatched it
separately, so a single agent received both. File-ownership partitioning stranded
corrections twice — a lane found false prose in a file it was forbidden to touch
and the fix waited a whole wave. A workflow script failed to parse on **unescaped
backticks inside a template literal**, costing a dispatch. And a verification
instruction was itself wrong: *"measure PH-16 in a probe at 393x852"* cannot
discriminate, because `phone_scale()` is exactly `1.0` at that size — the lane's
choice of 1080x2400 was correct and the brief called it an evasion.

**Root cause:** Building briefs by copy-paste; partitioning by file with no
route for cross-lane findings.

**Prevention:** One brief per lane, checked before launch. Escape every backtick
inside a workflow template literal. Serialize lanes sharing a file rather than
forbidding the edit, and always instruct lanes to **report** false prose in files
they do not own. **Check your own verification instruction is discriminating**
before demanding a lane satisfy it — a test condition that cannot fail is worse
than none, because it looks like rigour.

**Verification:** Re-read each prompt for a foreign lane's heading before
launching.
