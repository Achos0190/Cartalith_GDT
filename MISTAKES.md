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
| **Re-base a shared token / constant set** | Every *relationship* built on the old values is now unverified — a comment recommending a pair, a hover that was a lift, a contrast that passed. Re-check the pairs, not just the values | Compute the deltas and contrast ratios again; a re-base moves them silently and no test sees it |
| **Edit a main-loop-owned doc while a verifier is running** | Ownership is not the point — the **baseline** is. `OUTSTANDING_WORK.md` was clean when the verifier took its first `git status` and ` M` by its last; it correctly reported an unexplained mid-run diff it could not attribute to any lane | Hold doc edits until the verdicts are in, or tell the verifier in the brief that the main loop will be writing that file and when |
| **Commit while a verifier is running** | Don't. A clean tree makes `git diff` empty for **everything**, so every "the diff is empty" check silently stops being evidence | If you must, verify with `git diff <base> HEAD -- <path>`, never a working-tree diff |
| **Write a Workflow script** | **Escape every backtick inside the template literal**, including ones in prose like `(1080, 2400)`. An unescaped pair terminates the literal and JS then calls the preceding string as a tagged template — `"..." is not a function` | `node --check <script>` before dispatching. Two dispatches lost to this |
| **Read any shared file while lanes are running** | Not just probe *failures* — any read. The main loop grepped `export_presets.cfg` mid-batch, saw the exclusion already present, and told the owner the row would close without needing their authorisation. The lane was mid-edit; at HEAD the key was absent and the authorisation **was** required | Read the committed state — `git show HEAD:<path>` — for any claim about what was true *before* a running batch |
| **Read a probe failure while other lanes are running** | A concurrent lane's in-progress save to a shared `.gd` can produce a spurious *"Function X not found"* pointing at a call site whose definition is a few lines below. **Re-run once** before concluding the code broke | The same re-run discipline as a stale binary: a failure that does not reproduce was not a failure |
| **Assert on pixels** | A threshold is **palette-bound**, and naming the palette in prose does not select it — the harness must **force** it and refuse to run otherwise (this machine boots light). **And measure a control state**: a number with nothing to compare against cannot tell you whose defect it is. Borrowing a dark-theme `> 23` test and running it on a light capture makes every background pixel 251 — the check cannot fail | Assert the palette the threshold was written for, or use a palette-agnostic measure (uniform-row / distinct-colour count) |
| **Read a layout that overflows the screen** | A `ScrollContainer` with an axis **DISABLED folds its child's minimum size into its own** on that axis, so the overflow propagates to every ancestor with no scrollbar to reveal it. Three instances in this tree | Walk the tree for `get_combined_minimum_size().x` above the screen width and check which ancestor it reached |
| **Add `clip_text` / ellipsis to a Label** | It collapses `get_minimum_size().x` to **1**. Beside a `SIZE_EXPAND_FILL` sibling the label then vanishes entirely | Confirm the text still renders. Measured once by *removing* a line of real text and watching the blank-row count **rise** |
| **Claim something covered is now visible** | Reasoning from the scene graph proves nothing under an opaque overlay. **Flip the flag and diff the framebuffer** | A change that moves 0 pixels is inert, whatever the node tree says — and do not write the rationale into a comment before running that test |
| **Change a widget's ink** | The contrast pair's *second* term is whatever is behind it. If one call restyles several widgets and only some get a background, the un-backed one is a **separate relationship** | Compute its pair too. Nine correct ratios for the scrimmed siblings say nothing about the one without a scrim |
| **Grade a Godot probe as evidence for a Rust change** | `cargo test` does **not** rebuild `target/debug/cartalith_godot.dll`, and every `.tscn` probe loads it. A probe run after a `.rs` edit and before `cargo build` tests the *previous* engine — the Godot half of the stale-binary rule | Compare the `.dll` mtime against the touched `.rs` mtime and **state both numbers**. Measured once at 11:35 vs 18:42 — a whole batch of probes proved nothing about that batch's Rust |
| **Cite a test file in a doc comment** | A `pub(crate)` justified by a named test is a load-bearing dependency on that name. `render.rs` cited `tests/geology_micro_and_sky_fields.rs` twice, once as the visibility rationale; the file did not exist | `ls` every test path named in a doc comment in the file you touched |
| **Write an oracle for a ported function** | The reference's *errors* are part of the contract. A brute-force exact Euclidean transform failed a correct jump-flood port, because the reference jump-flood is exact from one seed and approximate beyond it | Assert the reference's behaviour including its approximations, not the mathematically ideal answer |
| **Re-resolve a citation late in a long pass** | A line number checked at the start can be stale by the end. Measured this session: **148 and 241 lines** of drift in files other lanes were editing; untouched files held exactly | Grep the quoted string. Never jump to the line |
| **Conclude a thing does not exist from a directory listing** | Absence of a path is not absence of the thing. `crates/cartalith-urban` has no `tests/` directory because the crate puts fixtures at `src/<module>/tests/golden.rs` — the milestone-16 golden was 3 139 lines of it, and a brief scheduled it as unbuilt work | Grep for the **symbol or its content**, never for the conventional location. `grep -rn golden crates/<crate>/src`, or grep the symbol, before concluding |
| **Write prose about another lane's subsystem in the same batch** | Two lanes ran concurrently: one removed a hardcode, the other shipped a note explaining that the hardcode was why a control was inert. The note was true at dispatch and **false on arrival** — and it reads as freshly checked | State the other lane's file as *of this batch*, or re-verify at the symbol after the batch lands. A cross-lane claim has a shelf life of one wave |
| **Convert a replacing UI context into an appended one** | The disarm path is the obvious one and the **arm-another-tool path is the one that gets missed**. A single-answer `match` over the armed tool reaches its ordinary arms before any draft clause, so a live uncommitted draft loses its Commit/Discard — and the newly armed tool draws *its* Commit in the same slot | Enumerate every transition INTO the new state, not just out of it. Probe each armed tool against a live draft and assert the draft survives all of them |
| **Write a parent-type guard** | The question is *"does a sibling compete for my width"*, not *"which class is my parent"*. A guard listing `BoxContainer`/`HFlowContainer` missed `GridContainer`, which shares width across columns exactly as an HBox shares it across children | Enumerate every container that distributes the axis you care about, and assert one child of each. Caught by a verifier, not by the guard's own author |
| **Gate a write on "is there anything to save"** | Ask the **whole** aggregate, not one member of it. A `vault.json` write gated on `links.is_empty()` silently dropped a map snapshot, because the store also holds `vaults` and `snapshots`. The correct predicate existed and was mutation-tested in the same batch, and was simply not wired | Enumerate every member the container can hold and assert a save containing **only** each one in turn survives a round trip |
| **Report a layout measurement** | One world is one sample, and panel widths are **content-dependent**. A lane measured an empty plan at 190 px and reported "no overflow"; three worlds measure 351 / 385 / 441 against a 280 px dock | Measure at least three seeds, and say which you used. A single-sample layout number is the same error class as a single-sample timing |
| **Convert a replacing context to an appended one** | A conversion is not only the artefacts the record names. Sculpt and Journey were both described as three (a `CTX_` constant, a titles row, a dispatch arm); Journey had a **fourth in another file** — a `build_results()` that cleared the shared body itself, harmless while it replaced and destructive once it appends | Grep every file that writes into the container, not just the one that owns the context. `queue_free()` on a shared parent is the tell |
| **Write a diagnosis into a backlog row** | The row is a **router, not a finding**. A row that names a cause propagates it into every brief written from it: "the panel is dropdowns whose minimum is their widest item" was wrong (there are no dropdowns), and it reached a verifier as an instruction to count dropdown items — a check that could not be performed | State the **symptom and its measurement**; leave the cause to whoever opens the code. If you must record a suspicion, mark it as one |
| **Attribute a number to a cause** | Measuring the number is not measuring the *cause*. A doc said a fallback dab "shows up in the after column's own maximum"; the fallback measures **15.3 ms** against a quoted maximum of **2.55 ms**. An explanation makes a single-sample claim look like an analysis | Measure the cause separately and quote it. If you cannot, say the number and stop |
| **Run a pixel probe** | `ImageTexture.update()` is a **no-op under `--headless`** — the texture never reaches `get_image()`, so every comparison passes vacuously | Run pixel probes **windowed**, and give the probe a positive control that must move. Reproduced independently 2026-09-04 |
| **Dispatch agent lanes** | One brief per lane, checked before launch. Serialize lanes sharing a file rather than forbidding the edit. Tell every lane to **report** false prose in files it does not own. **Every verification item carries a premise — check it holds before you write the item** ("mutate a constant each lane introduced" is unsatisfiable for a lane that introduced none) | Re-read each prompt for a foreign lane's heading. Ask of each check: what state of the world makes this impossible to perform? Four such items in one brief, six batches running |

---

## Why each rule exists

### [2026-09-03] Believing a backlog row instead of re-opening it ×15

**Mistake:** Work was scheduled, and briefs written, against rows already done or
whose blocker had lifted. Seven described built work (the manual-icon tool's
three gaps, CA-05's resize handles, the layer sync, `_civSaltAccess`); four
described lifted blockers — the "no JS runtime" claim was false for 18 days and
cost a whole dispatched wave. The Religion row claimed `cartalith-civ::belief`
does not exist; it is 945 lines.

**Four more, batches 18-19, and the pattern is now the expectation rather than the exception.** Urban milestone 16 had shipped a day before the row said it "remains … blocked by definition". Milestone 17's blocker was falsified **six minutes after it was written** and stood eleven days. DS-03's resolver had 87 live call sites across eight shell files while the row called it unstarted — for three days. The APK exclusion row was **half** stale: `_*` was already excluded, the `addons/` payload was not.

**What this changes:** a lane told to re-open a row before acting returns "already done, here is the evidence" often enough that it is a **first-class outcome, not a failed task** — and every one of these four still produced real work, because guarding built-but-unguarded code found defects (`ROLE["h_rail_head"]`, a golden covering 12 of 13 stage modules).

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

### [2026-09-03] Leaving prose that describes the old behaviour ×27

**Mistake:** Controls disabled by reasons that had become false; `render.rs`'s
module doc listing `rockSlope` refinement as **excluded** in the file that had
just implemented it — and that doc is the row's own cited location; `STATUS.md`
naming three deleted probes as "present and uncalled"; a `_todo` false in every
clause; two chips citing capabilities built hours earlier; a panel formula
inverted against its own engine, agreeing only at the default.

**Seven more, batch 17:** `world_workspace.gd:159` "Seasons and Köppen-Geiger
classification are not ported" (`cartalith-climate/src/koppen.rs` is golden-tested
and drives a live layer); `performance_window.gd:140` "no per-device enumeration
exists in cartalith-gpu" (`enumerate_devices`, `multi.rs:378`);
`civilization_workspace.gd:5405` "cartalith-civ has no such relation to record"
(`relations.rs` exists to create that edge, and three surfaces already draw it —
one 330 lines above the note in the same file); `FUNCTIONAL_CONTRACT.md:627` and
`:583`; `STATUS.md` RD-0 and RD-1.

**Root cause:** Behaviour and the prose describing it live apart; one gets edited.

**A sub-shape worth naming: a stale claim that asserts a whole crate module does
not exist.** b-7 and b-8 do not say "this control is unwired" — they say the Rust
does not exist. `git log -S` puts both strings *before* their crates (five days,
one day) and standing sixteen and fourteen. A per-wave diff review sees the young
entries and neither of these, and `tools/audit_wiring.py` structurally cannot see
any of them — fourth cut running. All three of batch 17's highest-severity finds
sit in surfaces that otherwise work, so no disabled-control sweep reaches them.

**Root cause of the sub-shape:** the prose was true when written. Nothing re-reads
a comment because a *different* crate landed.

**Batch 18 added the shortest-lived instance yet, and a new mechanism.** Milestone
17's backlog blocker — "settlements carry no `specialisation` and no `traits`" —
was falsified **six minutes after it was written** (`be2d5f7` 19:31:09 added the
hardcode, `e63d5d9` 19:37:15 added the `PlaceExtras` that supplies it) and stood
for eleven days. And a *concurrent* instance: one lane removed a hardcode while
another lane, in the same wave, shipped a note explaining that the hardcode was
why a control was inert. True at dispatch, false on arrival, and worded as though
freshly checked. The verifier caught it; nothing else would have.

**Prevention:** A stale comment is a defect, not a nit. **A reworded reason that
is still false is worse than the original — it looks freshly checked.** Re-opening
backlog rows does not find these; the grep below does.

**Verification:** Grep the touched files for the old behaviour's vocabulary. To
sweep for the sub-shape:
`grep -rn "cartalith[-_]" --include=*.gd shell/ | grep -iE "no |not |never |missing|absent"`
and open every symbol named.

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

### [2026-09-03] A token re-base silently invalidated four relationships

**Mistake:** The 2026-08-31 token re-base changed values that other code had
built *relationships* on, and nothing re-checked them. `--ins` moved #101112 ->
#191c1e, which turned an asset-library checkerboard from (7,8,8) apart to
**(2,3,4)** — invisible — in the exact pair a comment two lines above had
recommended. A trait-chip hover became a *darkening* where it had been a lift. On
light, `raised` and `panel` became byte-identical, so a drag preview was an
unbordered rectangle the colour of the surface under it. And a verdict green left
as a raw `Color(0.48, 0.78, 0.49)` sat at **1.96:1** on the light panel.

**Root cause:** A re-base is verified against its *sources* — each new value is
right — while the properties that matter are *differences between* values, which
no test and no golden covers.

**Prevention:** After changing a shared token or constant set, re-check every
relationship expressed over it: contrast ratios, adjacent-pair deltas, and any
comment recommending one value over another. A literal colour is worse than a
token twice over — it cannot be remapped, and `remap()` matches a baked colour
back to its token while matching a literal to none.

**Verification:** Compute the deltas and WCAG ratios for both palettes, not one.
A defect visible only on light, or only on dark, will not appear in a single
capture.

---

### [2026-09-03] Citing a rule in a brief is not satisfying it

**Mistake:** A verification brief said *"Dark theme, 1080x2400, count rows with
no pixel above RGB(23,23,23)"* — and named **no mechanism for getting dark**.
This machine boots `mode="light"` (`cartalith_settings.cfg`), where every
background pixel is 251, so a literal execution measures `blank_rows=0` and
reports a false improvement. **That is the exact trap the brief's own preflight
row warns about, reproduced inside the brief that cites it.** Only the probe's
`_force_dark()` plus a refuse-to-run-unless-dark guard makes the check real.

Two smaller ones in the same brief: `cargo test --workspace` was numbered first
as evidence for a **GDScript-only** lane, where no test result can be caused by
the work under test (a collateral floor, not evidence — and the brief's own
preflight row says the Rust suite cannot see the shell); and it asked a verifier
to check "BOTH lanes" when one had died, so three checks targeted claims that did
not exist and could not be refuted.

**Root cause:** Writing the rule into the brief feels like applying it. A rule
names a hazard; a *check* has to establish the conditions the hazard needs.

**Prevention:** For each check, name the **mechanism that establishes its
precondition**, not just the condition. And build the verifier's input from what
actually returned — a lane that failed supplies no claims.

**Verification:** Ask of each check: what result would refute the claim, can this
check produce it, and does the environment it runs in satisfy its premise?

---

### [2026-09-03] Orchestration errors that waste a wave ×12

**Batch 17 — a whole class of brief defect: an item whose premise is false, so
the check cannot be performed as written.** The verifier found four, three of
them premise failures. (1) *"Pick at least four rows it calls closed and re-open
them"* — the audit lane closed exactly **one** thing and said so; there were not
four closures to sample, and the over-eager-closure risk the check exists to
catch was structurally absent. (2) *"Mutate one constant introduced by each
lane"* — two of three lanes introduced no constant (one edited only Markdown,
one changed a sentinel string). Say **"per lane that introduced one."**
(3) *"Render before and after"* — the Godot `.dll` predated every `.rs` edit in
the batch, so no before could be staged and rebuilding destroys it; the real
before/after was a source-level pinned hash. (4) An item conflated two guard
pairs (`<1%`/`>99%` is the hover card's; the panel's is `<0.1%`/`>99.9%`), so a
verifier reading literally checks one surface and reports the other green.

**Why this matters more than a typo:** a false-premise item does not fail loudly.
The verifier either silently substitutes something else or reports a green it did
not earn. Three of these were caught only because this verifier was told to check
the brief itself — the sixth consecutive batch in which it found a brief defect.

**Earlier instances.** Pasted one lane's brief into another's prompt *and* dispatched it
separately, so a single agent received both. File-ownership partitioning stranded
corrections twice — a lane found false prose in a file it was forbidden to touch
and the fix waited a whole wave. A workflow script failed to parse on **unescaped
backticks inside a template literal** — twice, costing two dispatches; the second
was `(1080, 2400)` written in prose inside a verifier's brief, which killed the
verify phase after both build lanes had already completed. And a brief named a
fix by its register ID (**DS-12**) whose code lived outside the lane's declared
file-ownership list, because the ID's backlog wording implied a different file
than the one it is in. And a verification
instruction was itself wrong: *"measure PH-16 in a probe at 393x852"* cannot
discriminate, because `phone_scale()` is exactly `1.0` at that size — the lane's
choice of 1080x2400 was correct and the brief called it an evasion. And a
commit made *while a verifier was running* emptied the working tree, so its
"confirm `git diff` on project.godot is empty" check passed for every file
whether or not it had changed.

**Root cause:** Building briefs by copy-paste; partitioning by file with no
route for cross-lane findings.

**Prevention:** One brief per lane, checked before launch. Escape every backtick
inside a workflow template literal and run `node --check` on the script. Resolve a
named item to its **actual file** before assuming the ownership list covers it. Serialize lanes sharing a file rather than
forbidding the edit, and always instruct lanes to **report** false prose in files
they do not own. **Check your own verification instruction is discriminating**
before demanding a lane satisfy it — a test condition that cannot fail is worse
than none, because it looks like rigour. Four such have now shipped in briefs:
a 393x852 probe size where `phone_scale()` is 1.0; a count that is 0 by
construction; a working-tree `git diff` emptied by a mid-verification commit; and
`git diff <hash> HEAD` where `<hash>` **is** HEAD — written while anticipating the
previous failure and inheriting the same one. **Ask what result would refute the
claim, then check the instruction can produce it.**

**Verification:** Re-read each prompt for a foreign lane's heading before
launching.
