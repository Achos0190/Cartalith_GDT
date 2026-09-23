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
| **Edit a main-loop-owned doc while a verifier is running** | Ownership is not the point — the **baseline** is. `OUTSTANDING_WORK.md` was clean when the verifier took its first `git status` and ` M` by its last; it correctly reported an unexplained mid-run diff it could not attribute to any lane **×4, 2026-09-13, the main loop itself:** a "dry run" of a backlog normaliser wrote the LIVE `OUTSTANDING_WORK.md` because a `sed` path substitution silently failed; it was restored byte-identical 30 s later. A script that edits a guarded doc takes its target as a required argument, never a default, and the dry run asserts the path it will write before running | Hold doc edits until the verdicts are in, or tell the verifier in the brief that the main loop will be writing that file and when  |
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
| **Read `sample_cell()`'s (or any multi-valued optional field's) key with `Dictionary.has(key)`** | `has(key)` means "was this computed at all," never "is the value the notable one." `s.water_body: Option<u8>` is `Some(0)` for **land** — a real classification entry, not a missing one — so `sample_cell`'s wrapper sets `"water"` to `"land"/"ocean"/"lake"` for every classified cell. `d.has("water")` is true almost everywhere and proves nothing about wetness | Check the **value** (`d.get(key) != "land"`), never bare `has(key)`, for any field whose own doc comment lists more than one non-missing outcome |
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
| **Write a multi-line edit through a shell heredoc** | Escape sequences are mangled twice — once by the shell, once by the language. A `\
` continuation reached a `.gd` file as a literal backslash-n and broke it; the same edit had already produced five stray tabs mid-expression, which **parsed clean because tabs are whitespace** | Prefer the Edit tool for multi-line source changes. If you must script it, read the bytes back with `repr()` afterwards — and run the parse check, which is what caught this |
| **Name the language of a remaining caller** | "The caller is GDScript" was written into four Rust doc comments about a function GDScript **cannot call** — a plain `pub fn` taking `&mut [u8]` and a Rust struct, neither marshallable across gdext | Check the signature, not the intent. A `#[func]` is the only thing GDScript can reach, and its arguments have to be Godot types |
| **Name the precondition for what the code does today** | The gate is usually **nearer** than the story suggests. Three clauses said badges draw as discs "until a pack is imported"; the real gate is that no resolver Callable is installed, so a world holding a pack full of art still draws discs | Write the condition the code actually branches on, then read the branch back and check it says the same thing |
| **Enumerate a surface for an audit** | Walk the **code**, then ask the designs about each entry. Walking the designs and ticking items off can only ever find what the designs already list — a surface nobody drew is invisible to it. Measured: 37 of 283 menu items had **no design at all** | State the inventory method and how you know it is complete; have the checker re-run it looking for what was missed |
| **Call a constant dead** | Probes are committed files and they read theme tables. `ROLE["h_timeline"]` was proposed for removal while `_roleresolve_probe.gd` asserts on it | `grep -rn` the key across the whole project **including probes and tests**, not just the shipping call sites |
| **Emit a change signal** | Emit **after** the engine call, not before. Ten emitters here fired ahead of their own `world_gen.sculpt_*()`, so a synchronous listener re-read the count from before the change — measured at 0 stamps on the stroke that created the first | Probe the listener's value **inside** the emit. And check each connection's flags: a `CONNECT_DEFERRED` listener runs a frame later and never sees the stale value, so "both listeners" is a claim about flags, not about count |
| **Write a backward-compatibility test** | A fixture of empty collections proves nothing about the installed base. The whole legacy check here was `{"vaults":[],"links":[]}` | Build the fixture from a real prior-format document at the commit that wrote it (`git show <sha>:<path>`), and assert it **opens, resolves, and re-serialises byte-identically** — so opening an old file does not silently rewrite it |
| **Replace a single-sample timing with a median** | **A median of five on a noisy device is still one sample of the median.** The pass sent to retire single-sample timings wrote `1.06x (1.00..1.08x)`; three independent serial re-runs measured 1.00x, 1.00x, 1.02x — outside all of it, with the "saving" changing sign | Re-run the whole measurement in a **separate process** and check the new median lands inside the old bracket. If a bracket's low end already touches parity, say **no difference is established** and quote nothing |
| **Quote a sub-millisecond figure to three significant figures** | The precision is the claim. `9.65 us (9.63..9.66)` across 7 runs was measuring that run's cache state; an independent harness of the same shape got 10.02 (9.95..10.04) — magnitude right, brackets disjoint | Give an order of magnitude when that is what the number is for, and let the test print the spread |
| **Move a command off the menu bar** | The searchable index is built by **walking the live `MenuBar`**, so a command that stops being a menu row stops being findable — silently, with nothing failing. Two ruled moves scored **0 title matches out of 361** the moment they landed, and one accelerator became unrebindable and unlisted | Add a `command_index.gd` `EXTRAS` row and a `shortcuts_dialog.gd` `UNLISTED` row **in the same change**. Assert by TITLE, not `search()` — search matches blurbs and will pass on a neighbour's tooltip |
| **Reserve a file from every lane** | Reserving prevents collisions and **strands the fix that needs it**. Three lanes correctly reported that the index and the shortcuts list needed rows; none could write them, and the regression shipped to the verifier | Either give the shared file to exactly one lane, or accept that the main loop must land those edits before the batch is called done |
| **Summarise a spec into a backlog row** | **Paraphrase the table, do not compress it.** "12 mode-gated blocks, one body per rail node" was the main loop's summary of `04-left-dock.md` §3, which actually gates **one node of ten** — and an owner accepted a consequence ("this hides controls") on the strength of it | Quote the spec's own condition column into the row. If the summary and the table disagree, the table is the spec |
| **Claim a mechanism is "derived, not hardcoded"** | Check every line of it, not the entry point. The mode switch derived its **visibility** and hardcoded its node source, its labels and its refresh — so a future gated domain would get an empty pill carrying another domain's labels | Name the parts that are derived and the parts that are not, separately |
| **Write a probe's usage header** | The header is a claim about the probe's own code. One documented `--resolution`, which it never read — it read `-- --vp WxH`. Following it would have measured **the same box three times and called it three densities**, the single-sample shape in disguise | Grep the flag in the probe body before writing it in the header, and have the harness **fail loudly** on an argument it does not understand rather than defaulting |
| **Add anything to an approved artboard** | **Content added below the fold evicts content that was above it.** A fix pass added a ~225 px explanatory block to `PhoneViewportControls.dc.html`; root `scrollHeight` went 1 457 → 1 754 against a frame declared 1 012, and the CONTRAST section moved from `top 985 / bottom 1006` — fully visible — to `top 1210`, outside the frame. The lane disclosed the overflow as "pre-existing", which was true and understated: it did not land in already-invisible space, it *pushed visible space out*. On an owner-approved composition that is a silent regression | Measure `scrollHeight` against the frame's declared `h` **before and after**, and raise the frame in `canvas.json` **and** the board's own root height together — they are two numbers and both must move |
| **Delete a surface** | **Inventory it first, item by item, and give each a stated home or a stated drop.** Folding a window away found 11 items: 6 already had a home, 2 needed one built, 2 were dropped deliberately, and **1 was a real capability loss that would otherwise have been silent** — the names of the GPU stages dispatched, whose only reader was the deleted window (a sibling shows the count, not the list). The inventory is what turns "we removed a window" into a decision you can check | Grep each binding for its **other** consumers rather than reading the design. That turned up a diagnostics file the brief never mentioned, already carrying two of the items verbatim. Then grep the class name across probes and `.tscn` too — eight probes referenced this one and would have broken at load |
| **Write a completeness claim into a correction** | *"Nothing was left un-found"* was false in the same edit that said it: the hunt reported 6 sites in 3 files and there were **5 in 4**, one of them in the very file whose zero-consumer status the same lane had just reported. A correction that overstates its own coverage is worse than one that admits a gap | Say what you searched and how, not that you found everything. `git grep -n <name> -- <path>` pasted into the report beats any adjective |
| **Cite a document as authority for an absence** | **Check the document is still describing the tree.** Thirteen shipped `.gd` comments cited `04-left-dock.md` §0/§9.1's "lost to truncation" as authority for values that are readable — the prototype was re-imported whole in `660cbef` ("Design answered: the files are whole") and §0/§9.1 were never updated. It reached a `LARGE_ITEM_RULINGS.md` consequence I wrote, which instructed a build to *derive* two captions that are quoted verbatim at `Cartalith DCC Environment.dc.html:1937-1940` — following it would have replaced a true provenance with a false one | Before repeating "X is unrecoverable", open the file and check its size and last line. A truncation note outlives the truncation |
| **Let a lane finish a file another lane is granted** | A lane silently wrote 255 lines into a file assigned to a different lane. Both merged cleanly and both parsed — but the shell build hash moved **between one lane's own probe runs**, so the lane that stayed inside its boundary took all its evidence against a moving tree, and it deliberately left a false citation unfixed because someone else was in that block | A lane needing a second file **asks**, and is granted or refused. Straying silently is what makes a verifier's evidence unattributable, and the cost lands on the lane that behaved |
| **Resume a workflow lane with `SendMessage` while its workflow still runs** | Two copies of one lane can be live. The resumed copy reported its **own** board's symbols, in its **own** granted files, as evidence of a foreign writer — and reported the file broken when it was mid-save (`_build_timeline_readout` was defined thirty seconds later and both files passed `--check-only`) | Prefer letting a lane finish. When a resume is necessary, tell it explicitly that work already on disk in its own files is its own |
| **Act on a critic's defect list** | **A critic's defect is a claim, and refusing one is a legitimate outcome.** Two of a design critic's seven survived contact and two did not: *"`poi` is emitted unconditionally"* — it is the true arm of `if p.is_poi`, and every `GeoPlace` is built `is_poi: false`, so adding the chip would have drawn a group that can never populate; and *"no dry-run path exists"* — one is wired on a sibling route, so the defect held only on the route drawn and the fix needed a different justification than the one stated | Open the cited symbol before replacing a value. Record "not real, changed nothing" as a result, and when a defect is real for a **different reason** than stated, write the true reason — not the one you were handed |
| **Act on a verifier's own fix** | **A verifier's finding is a claim too, and its *fix* is the least-checked part of it.** One reported that widening `_dash_phase_track`'s f32 accumulator took a residual "from 43 differing px to 0 and every case to byte-identical". Re-measured windowed, both ways, same fixtures: **no difference on either probe** — `_segcull_probe` PASSes and `_cull_probe` FAILs 13 of 16 under f32 *and* f64. It was one edit from shipping as fact, sourced from the agent whose job is to stop exactly that | Re-measure a verifier's fix before adopting it, the same way you would a lane's. Keep the change if it is right on its own terms — and then say so, rather than citing a result you did not reproduce |
| **Write `--headless` into a brief** | Check what the lane's claim actually rests on. A brief mandated `--headless` for a lane whose entire result was **pixels**; the dummy display driver never fires `RenderingServer.frame_post_draw`, so every pixel and timing probe aborts or hangs. Both of that lane's probes already carried an explicit ABORT guard for it, and `MISTAKES.md` already had the row | Headless for logic and layout; **windowed for anything that rasterises or times a frame**. Say which in the brief instead of mandating one |
| **Judge a measurement against a floor** | Check the floor applies to the density you measured. A row carried *"`DccWidgets.action` buttons are 39 px against a 44 px touch floor"* through two passes: 39 px is a real *drawn* height at **pointer** density, and 44 px is a **touch** requirement that the touch branch already meets (`role_px("btn_min_h")` = 44, and `phone_fit()` floors every `BaseButton`). One pass measured the drawing, one read the constant, neither noticed they were describing different densities | Name the density beside the number, always. A figure without one invites being compared to the wrong bar |
| **Dispatch a batch while another batch's verifier is running** | **Don't.** A verifier's evidence is only as stable as the tree under it. A batch dispatched mid-verification edited `map_overlay.gd` (+137/−30) while the verifier was probing: the shell build hash moved across its three runs (`fa6e20b` → `594287e` → `1993049`), one run hit a parse cascade that did not reproduce, and it correctly reported *"an unreported fifth editor changed the tree mid-verification"* — attributable to none of its four lanes. **Every probe result in that report was taken against a moving shell.** The lanes' own files being disjoint is not enough; a shell-wide parse failure cascades from any file | Sequence: verifier finishes, main loop commits, next batch starts. If a batch must overlap, tell the verifier in its brief which files a *different* workflow owns and when |
| **Write a correction** | **The correction is new prose and can ship its own false clause.** A note rewritten to fix a stale claim said `_foodshed_probe.gd` *"drives all four of its branches"* — written in the same edit that gave the function a **fifth** branch, which that probe does not reach. A doc citation corrected in the same batch named `_show_project_stats()`, a symbol that exists nowhere in the project | Re-read a correction as if someone else wrote it, and grep every symbol it names — including the ones you just added |
| **Dash a field with a reason** | **The reason is a claim and gets verified like any other.** A batch that minted zero fake *values* shipped three fake *causes*: a user-visible tooltip said `detectRiverCrossings is unported` (it is ported, runs in the pipeline, has tests — the gap was an adapter field one layer nearer); a chip said `no binding reports how many` (the binding exists, only its GDScript forwarder is missing); and two dashed states were each given **the other one's** reason. **A wrong reason is worse than none** — it reads as freshly checked and it routes the next brief at a whole subsystem | Open the symbol you are about to say does not exist. Write what is missing at the layer it is actually missing from. For a two-branch dash, exercise **both** branches — a probe that only walks the live path cannot see an inversion |
| **Carry a rule from one variant of a design to another** | Open the other variant's **drawing**. A brief told a verifier "the destructive action must be text-only and not the rightmost" — the approved artboard's variant B draws it filled and rightmost, because B has no safe action for the rule to be relative to. A verifier following that literally would have failed a lane for building the approved design | Scope the rule to the condition that makes it true, in the design itself. Where two variants of one artboard disagree, the artboard is wrong and gets fixed before the code does |
| **Choose which preflight rows to put in a brief** | Scan the table against the batch's **verbs**, not from memory. 7 of 55 rows were inlined by recall; four that matched exactly what the lanes were about to do were missing — change behaviour, grep `func <name>` first, change a widget's ink, replacing→appended context. One of them then bit the main loop personally in the same session | Read the left column top to bottom against the batch's own scope. It is one screen |
| **Re-run the parse check after EVERY edit, not once per file** | A probe edited twice parse-checked clean after the first edit and shipped a duplicate `var` from the second. A failed script load makes `godot --headless` **hang** rather than exit, so it reads as a slow probe, not a broken one — two timeouts were spent before the output was captured | `--check-only --script` after the last edit to a file, always. If a probe that used to finish in seconds hangs, suspect a parse error first and capture stdout |
| **Cite two examples as agreeing** | Open the **second** one. A new header quoted the asset-pack inspector *and* the Data manager's RECENT RUNS block and said *"Both settle one thing"* — RECENT RUNS settles the opposite (parent `#6f7478`, value overridden to `#8d9296`, so the identity is the **dimmer** ink). The code change was right for the first reason alone; the second example was decoration that made it false | Quote both, in full, with line numbers. If they disagree, say the canvas is split and name what actually breaks the tie |
| **Quote a count of call sites, instances or files** | Measure it in the same edit that writes it, and paste the command. A comment shipped *"its 94 call sites across the shell"*; the real figure is **189 code call sites in 18 files**. It was an understatement, so the argument held anyway — which is exactly why nobody checked | `grep -rn 'Symbol(' dir/ \| grep -v ':[0-9]*:[[:space:]]*#' \| wc -l`, then write that number and the date |
| **Dispatch agent lanes** | One brief per lane, checked before launch. Serialize lanes sharing a file rather than forbidding the edit. Tell every lane to **report** false prose in files it does not own. **Every verification item carries a premise — check it holds before you write the item** ("mutate a constant each lane introduced" is unsatisfiable for a lane that introduced none) | Re-read each prompt for a foreign lane's heading. Ask of each check: what state of the world makes this impossible to perform? Four such items in one brief, six batches running |
| **Correct a doc a brief told you was stale** | **The brief names one false term; the doc may have more.** A brief said `landmark_store.last`'s doc argued persistence was unnecessary *"because a re-run reproduces the placement"* — one stale term. Its **opening clause** was already false too (*"`last` is still not written, deliberately"* — ruling 10 had written it days earlier). A lane correcting only the named term would have shipped a correction carrying its own false clause | Read the **whole** doc block, not the sentence quoted. `git show HEAD:<file>` the claim and test every clause against the code, including the ones the brief did not mention |
| **Write a golden-risk item into a brief** | **Name the crate the goldens live in, and check the change can reach it.** A brief warned that wiring an input *"can move placements where adding the unused field could not"*. Every golden lives in `crates/*/tests/`; the change was confined to `cartalith-godot`, which no test crate reaches, and **no golden covers the landmark pass at all** — the only file naming it is self-declared diagnostic-only. Not wrong to check, but the premise was structurally impossible | `find crates -path '*/tests/*' -name '*<subject>*'` and `git diff --stat` before writing the item. A risk item whose premise cannot hold trains lanes to treat the section as ritual |
| **Read the backlog counter after closing rows** | **It counts ROWS, not open items.** Closing a row strikes it in place, so the headline does not move; a later pass sweeps struck rows and it drops by however many were swept. Two closures and two additions can land on the same number as before by coincidence | Reconcile against `git stash` + a HEAD run before believing a delta. Expect `closed` to move the count only after the sweep, and say which of the two you are quoting |
| **Quote a memory figure for a buffered pipeline** | **A raw buffer is not a peak.** A ruling quoted `w*h*3` — 2.06 GB at 32 768 × 20 976 — as what an export costs. The real peak is **21.7 B/px measured**, ~15.8 GB, because the pipeline also holds a luma plane and blur buffers. The code's own budget was wrong the same way (15 B/px: it counted one buffer per blur where `blur_once` allocates two, both live until it returns) | Name which figure you mean. Poll peak resident from the **host** — a process cannot see its own allocator — and check the slope across two sizes, not one point |
| **Say "recover it from history"** | **Prove the artefact is in history first.** A ruling instructed recovering a prototype that was "measured byte-identical and reverted". It was reverted *before its pass committed*: `--diff-filter=A` finds nothing and `-S <symbol>` returns only prose commits. The scope document's prose was the entire surviving artefact | `git log --all --diff-filter=A -- '<glob>'` and `git log --all -S '<symbol>' --stat` before writing the instruction. "Reverted" does not imply "committed once" |
| **Add a gate to a path that shipped ungated** | **Say so where the constant is defined, not only in the report.** A memory gate named `UNGATED_MAX_WIDTH` bounded only its no-budget branch; the budget branch gated **every** width, the three that had shipped unconditionally included. The doc said they "stay that way". Real on Android, where the budget is small and reported | Ask of each guard: which inputs newly fail that used to pass? Put the answer in the doc, and say which direction the estimate errs |
| **Add a formatter, parser or helper "because the other one is unreachable"** | **Grep the crate first.** `human_bytes` was added to one file justified as unreachable from GDScript; an identical `pub fn human_bytes` already existed in the same crate with its own test. The two disagreed in user-facing strings — `15.4 GB` against `15.4 GiB` for the same bytes | `grep -rn 'fn <name>' crates/<crate>/` before writing it. One crate, one convention for anything a user reads |
| **A lane returns no report** | **Stop the batch; do not pass a `null` through as a report.** On 2026-09-06 the largest of three lanes (723 lines in one file, plus four others) hit the StructuredOutput retry cap and filed nothing, and the verifier brief's first three attack items were written as interrogations of claims that did not exist. It came back clean only because the verifier built its own oracle unprompted | Check every element of the lanes array before writing the verify prompt. A missing report means re-run that lane or verify its diff directly — and say in the brief which it is |
| **Write a guard that makes an artifact self-consistent** | **Ask what the guard actually proves.** `write_project` computed a pyramid's `source_key` from the heightmap it was writing, so the archive could never contradict itself — which meant stale tiles were **stamped with the new world's key** and read back with an empty warning list. Self-consistency is not freshness | Ask: what state does this guard make *undetectable*? Treat a caller-supplied key as a claim to check, not a field to overwrite |
| **Report a `cargo test --workspace` total** | **Count the result lines and check them against the brief's stated floor.** A lane reported *"100 result lines, 2 380 passed / 1 failed"* and then reasoned carefully about why a failure that does not exist was not its fault. The real run is **157 lines / 3 240 passed / 0 failed**, and the named test passes 3/3 in isolation. **×3 now** — the same partial read has produced 2 104 and 17 before this | `grep -cE '^test result:'` must equal the floor's line count **before** you read any total. A count below it is a truncated read, never a result |
| **Write the guarded-paths list in an agent brief** | **Say who a path is guarded *from*.** A brief listed everything under `design/` as guarded and `LARGE_ITEM_RULINGS.md` as main-loop owned; the main loop then wrote to both mid-batch, and the verifier correctly reported a guarded path dirty at batch end with no lane accounting for it. It cost a real finding's worth of attention to clear | Phrase it as *"guarded from you; the main loop writes here and its diffs are expected"*, or hold the edit until the batch lands |
| **Repeat a design document's dashed field into a brief** | **Re-measure the dash's reason, not just the dash.** A brief told a lane *"there is no log file, so a log-tail line is dashed — do not invent one"*, taking the canvas's word. There is one: Godot enables file logging by default on desktop, `user://logs/godot.log` exists and rotates five deep, and a probe's unique marker was read back out of it the same run. **Keeping the dash would have shipped a false reason** | A dash is a claim about the code, so it expires like any other. Open the symbol before repeating it, and say in the brief that the lane may overturn it |
| **Quote `.dll` and `.rs` mtimes in a report** | **Re-stat them at the END of your run, not when you looked.** A lane stated *"dll 16:08:10, atlas.rs 16:03:43"* truthfully and then edited `atlas.rs` again at 16:10:23, so the library shipped older than the fix and every Godot-side measurement after it was unattributable. The verifier caught it by re-stating rather than reading the claim | State both mtimes as your last action, after the final edit. If the `.dll` is older, rebuild and say you did |
| **Commit a "clean" subset of files while a shared file stays uncommitted** | **A file split across two commits can leave an intermediate commit that does not build, even when the final working tree is fine.** Two lanes shared `landmark.rs`; one lane's commit shipped `lib.rs` calling `mark_icon_committed()` while `landmark.rs`, where that method is defined, stayed uncommitted (the other lane was still using it). `HEAD` did not compile until the second commit landed minutes later — a real, if brief, broken point in history, not just a working-tree inconvenience | `cargo build` (or `--check-only` for `.gd`) against **exactly what `git commit` is about to write** — stage the files, `git stash --keep-index` the rest, build, then restore — before pushing a commit that calls into a file left for later |

---
| **Ask a lane to walk a touch floor** | **Say TARGET, not height.** A brief asked for "the laid heights" of a row. Every height was already 44 px; **both real defects were on the WIDTH axis** — a transport square at 36 px and a collapse chevron at **7 px**. A height-only walk executed literally would have reported the row green and shipped a 7 px finger target | Phrase it as "every tappable control's laid size, both axes, against the floor". A floor is a floor on the target, not on its taller axis |
| **Run a probe `--headless` and report its error count** | **Godot writes `SCRIPT ERROR` to stderr, and `--headless` manufactures its own.** A lane reported *"0 SCRIPT ERROR"*; the same command with stderr captured showed **22**, all `Cannot call method 'save_png' on a null value` because `get_texture().get_image()` is null under the dummy driver. Identical before its edit, so it hid no regression — but the number was wrong | Capture stderr explicitly, and **run a probe in the mode its own header prescribes**. Windowed, that probe is genuinely 0 |
| **Report a defect as density-specific** | **Check the other density before naming one.** A lane called a small-font violation "phone-density-only" because its 1440×3168 leg looked clean. Re-tallied: **15 of 23 screens at 1440 and 19 of 23 at 1080** — the label is a fixed `fs=10.0` that does not scale, so it is not density-specific at all. Handing it on as 1080-only would send the next lane looking at `phone_scale` | Tally per resolution before writing the word "only", and name the mechanism (a fixed size scales with nothing) rather than the symptom |
| **Add a second route to a control the user already has one to** | **Two views of one state will disagree, and the panel is where it shows.** A role list gained a row press beside its existing dropdown; pressing a row moved the marks, the title, three dials and a button while the dropdown still read the old value, because `DccWidgets.choice()`'s return had been discarded — harmless while it was the only route in. **The lane's probe asserted every other consumer and not this one** | Before adding a route, list every widget that reads the same state and assert each follows. A discarded factory return is the tell |
| **Tell a lane a field has no engine behind it** | **Open the setter, the emitter and the drawer before writing it.** A brief said `halo` had no engine and should be dashed "prominently". It is fully live — `set_field` accepts it, the render list emits `halo_em` for generated *and* hand-placed rows, and the overlay strokes it. Obeying would have shipped a false reason on the one field that keeps a label legible over terrain | Three symbols, not one: where it is set, where it crosses, where it is drawn. Say in the brief that the lane may overturn you |
| **Floor a control that a rebuild creates** | **`tablet_fit()` floors HEIGHT only and runs ONCE from a deferred pass**, so anything a later `_rebuild_*` builds is reached by neither fitter. Three batches running have found the same shape — a 22 px field, a 7 px chevron, then `edit` 44×29 / `×` 27×29 / a colour well 60×24 | Floor at the call site for anything built after first layout, on **both axes**, and measure at tablet as well as phone. The general fix is a fitter that runs on rebuild |
| **Report that a feature does not exist** | **One grep is not an absence proof — search for the CONCEPT, not a string you imagined.** ×3. I told the owner twice that no km/mi toggle exists anywhere in `shell/*.gd`, having grepped `miles\|km/mi`. It ships as a three-way radio (`Preferences ▸ Units`, km/mi/nmi) because the code says `DccUnits.label("mi")` — the string I searched for was never going to appear. **2026-09-21**: dispatched an agent to build a "historical territory record" (CV-23) having grepped `HISTORY_TERRITORY_PREFIX` — the reserved save-slot *constant* — and found it unused. The real wiring ran through the **field**, `history_territory`, a completely different grep target; the whole capability had shipped under a named owner ruling, commit and test suite already in the tree. Checking the constant felt like checking the concept and was not. **2026-09-23**: `OUTSTANDING_WORK.md` filed "149 of 200 addon villages have no road" as a reference-shaped design question, having checked one function (`_civSeedVillages`, confirmed read-only) and concluded no connector exists anywhere in the reference. A second, dedicated function does — `_civConnectVillageAddons` (`Cartalith Gen1 v2.11.html:25766`), called unconditionally after every village-seeding pass — found only because the owner reported the visible symptom directly ("the html connects secondary settlements to the route network, not the godot version") and grepping the CONCEPT ("connect" + "village") rather than re-deriving from the function already checked turned it up in one pass. | Grep the domain word (`units`, `unit_mode`), the settings key, and the menu label separately; then open the symbol. An absence claim is the one kind no single search can establish. **When a name is declared as a constant/prefix/key, also grep for the FIELD or FUNCTION that would actually consume it** — a re-exported constant with zero readers proves the constant is unused, not that the capability it names is unbuilt. **Checking one function that plausibly owns a capability is not the same as checking the capability** — grep the concept's own verbs (`connect`, `link`, `attach`) too, not just the function you already found. |
| **Write a probe that PRESSES a control** | Press its **edges**, not its centre. **A centre press cannot see the width of the target it hits** | Shrink the control below the floor; a centre-pressing probe still passes |
| **Assert a control is present, sized or reachable** | `get_global_rect()` reports a child’s **UNCLIPPED** rect. Assert the **drawn rect against its container’s VISIBLE rect** | A control 15% painted must FAIL; keep a positive control that fires |
| **Assert a widget’s appearance** | A **stylebox-shaped test cannot see a modulate**, and headless returns a null texture. **Render it and read the pixels** | Force the colour to red; the probe must move |
| **Prove a style is independent of a stale theme** | **Independence is not correctness.** Zero movement under a hostile theme rewrite says nothing inherits — nothing about alignment, centring or minimum size | Assert the drawn VALUE against a canvas literal as well |
| **Add a probe in the same commit that changes the behaviour** | That is the **same claim written twice**, not evidence. A probe earns authority by **failing on the state before the change** | Run it against the pre-change file; if it passes, it pins nothing |
| **Assert a drawn colour** | Never against `DccTheme.c(token)` — **both sides move together on a flip**, so it sees a wrong token but never a wrong value. Pin the canvas literal with its `ENV:` line | Flip the palette; a token-vs-token check cannot fail |
| **Set a palette inside a probe** | **After** the shell boots. `apply_theme()` before instantiating `app.tscn` is **inert** — the boot re-applies the saved mode | Assert a value that DIFFERS between palettes before trusting the leg |
| **Measure ink or a glyph in a capture** | Taking the **brightest pixel** is a dark-palette habit. On light the brightest thing is the **ground** | Take the pixel furthest from the band in luminance, inside its own bbox |
| **Answer "can the user do X?"** | Measure the widget **the owner touches**, not the one the brief names. A desktop `PopupMenu` and the phone’s `Button` chips are different classes with different answers | Name the surface and the density you measured on |
| **Reach a screen inside a probe** | **Never call a function to navigate.** Tap what is visible from launch. Separate **it renders** / **it can be operated** / **it can be FOUND** | A control can be visible, sized and tappable inside a container that is invisible |
| **Find a figure the canvas does not contain** | Check the **superseded** canvas before calling it invented. `height:34px` is 0 in the current canvases and **32** in the replaced one | `grep -c` both; an inherited figure is a re-anchor, not a deletion |
| **Say which renderer the shell uses** | It boots **`gl_compatibility`** (`project.godot:101`), not Vulkan. Either is a real rasteriser; **headless is not** | `--rendering-driver vulkan` forces Vulkan if a leg needs it |
| **Cite an `ENV:` line** | **Open it.** A radius census cited `ENV:912` for a control; `ENV:912` is a bare closing `</div>` | Quote the declaration you are citing, not just its number |
| **Dispatch a second workflow while a verifier is running** | Don’t — or **state in the verifier’s brief exactly what else may write, and name the files** | Have the verifier snapshot md5s and report drift; two caught this |
| **Write a commit message** | It is a **claim about its own diff**. Diff the staged set against every claim — each probe named as passing must be IN it | `git show --stat`; a named file that is not there is a false claim, and `a2682de` shipped two |
| **Re-run a probe as evidence** | Check it could RUN. `_nwclip_probe` prints `fail=0` on desktop because `is_phone()` needs touch — **a probe that cannot answer must not report a pass** | Non-empty output AND a positive control that fires; make it exit non-zero when it cannot run |
| **Write a mechanism into a brief** | Measure it first. I wrote *"`DirAccess` cannot reach shared storage on Android"*; with zero permissions it lists 16 dirs and writes fine — **only FILE listing is filtered** | State the measurement that produced it, or label it a candidate |
| **Write or accept a probe’s exit status** | **A probe must not report success its own output contradicts.** `_tabfit_probe` printed `overflow=285.0` and exited **0**, because its assertions were all on a sub-surface | Give "assertions held but the surface fails" its OWN exit code (3), distinct from an assertion failure (1) |
| **Close a backlog row** | **A struck row left in a NUMBERED section still counts.** Striking four in place moved the headline by zero | Move it to an archive section (deliberately unnumbered), then re-run the counter |
| **Run `--check-only`** | From the **`godot-project` root**, never a subdirectory. From `shell/` it reports *parse errors that do not exist*, because `res://` resolves nothing | Two errors on `app.gd` from `shell/`, zero from the root. **Suspect your invocation before the code** |
| **Call `phone_window()` on a dialog with its own verb** | It sets `ok_button_text = "Close"`. Set the caller’s text **AFTER** it, or the primary action is renamed **on phones only** — where the dropped title bar makes that button the only thing naming it | Hit three times in one day: *"Write to Markdown"*, *"Create"*, *"Clear 412 MB"*. `DccWidgets.confirm()` orders it correctly for free |
| **File a row saying "this needs a bigger fix"** | That is a **claim about the code**, and it went wrong twice in one day — the SAF caller audit (four of seven consumers ended in Rust `std::fs`, so the answer was to make the audit unnecessary) and the "second remedy" for a `dialog_text` dialog (text-plus-a-verb IS `confirm()`’s shape) | Re-open it at its symbol before scheduling the big version. **Both rows were written from the SHAPE of the code rather than from what it needed** |
| **Chain a shell command after `grep -c`** | `grep -c` **exits 1 when the count is 0**, so `&&` silently drops everything after it — the count prints and the rest of the command never runs | Append `\|\| true`, or check the count as data rather than as an exit status |
| **Plan a phone probe that must reach a DIALOG** | **You cannot.** A synthetic tap cannot reach a control inside an embedded `AcceptDialog` sub-window at `content_scale_factor` 2.62 — canvas coords, physical coords and `get_final_transform()` all tried, `gui_get_hovered_control()` stays null. Taps into the MAIN viewport route normally | So the project-picker and New World legs are **provable on glass only**. A probe that falls back to pressing by label is testing the handler, not the touch path |
| **State how many checks a probe runs** | **Print the count; never assert it in prose.** A doc said 26 where the probe ran 29, and a count that disagrees with its own output is the first thing a later reader distrusts | Increment in the `_check()` helper and put it in the RESULT line, so the output carries the truth and no header can drift from it |
| **Report that you could not reach something** | **"I could not reach it" is an ABSENCE CLAIM and needs the same scepticism as "it does not exist."** A lane called six right-dock sections unreachable; the verifier reached **five of them in one pass with no shell change** | Say what you tried and what mechanism you assumed. That lane looked for a context REPLACEMENT where the file’s own header documents sections as **APPENDED** |

## Why each rule exists
| **Verify anything phone-shaped** | **`--force-touch` on the desktop is not the phone, and `pressed.emit()` is not a finger.** A whole session of probes reported a healthy phone shell; the owner picked up the APK and could not find the generation menu or use the journey planner within minutes. Synthesised input is injected **downstream** of `MOUSE_FILTER`, scrims, gesture handlers and hit areas, so a control unreachable by touch still passes | **See with `adb exec-out screencap`, act with `adb shell input tap/swipe`** at coordinates read off that image, and navigate from launch tapping only what is visible. Desktop probes are for regression, never for reachability |
| **Claim a screen "works" on a phone** | **Separate "it renders", "it can be operated" and "it can be FOUND".** Probes proved the journey planner fits a 1080 px screen and clears the tap floor; the owner still could not use it. Those are three claims and only the first two were ever tested — reachability was never tested at all | State which of the three you measured. A route nobody can find is a defect even when every control behind it is perfect |
| **Find a screen that is mostly blank** | **Ask what is IN it before ruling on how big it is.** A lane measured a phone sheet at 1 003 rows holding one horizontal strip — 933 blank empty, 912 blank *with a world* — and deferred it because the detent fraction is transcribed from the prototype and 'changing a transcribed detent needs an owner ruling'. Right measurement, wrong conclusion: **a correct detent over empty content is still an empty screen**, and the owner hit exactly that defect days later | When blankness survives the state change that should fill it (912 vs 933 with and without a world), the container is not the fault. Report what the screen is missing, not its dimensions |
| **Conclude a batch has finished** | **A run directory you cannot find is not a run that has stopped.** I searched `~/.claude` for the workflow’s run folder, found nothing, saw two of three lanes’ diffs in the tree, and closed the batch out — **while the third lane and the verifier were still working.** The commit landed mid-verification, and the verifier said so: from that point `git diff` was empty for everything and **stopped being evidence**. Its findings survived only because the content happened to be identical. **Wait for the completion notification, or check the task list. Never infer completion from the filesystem, and never from "the lanes I know about have reported".** |
| **Write "still X" or "routes NONE" into the backlog** | **Re-measure it in the same breath — do not copy the brief’s number into the row.** I narrowed a units row to *"the journey planner still routes NONE"* and committed it. The planner had been converted a commit earlier: `grep -c DccUnits` = **34**, with a probe asserting its headers. The same row carried *"the measure tool converts 4 of its 7 modes; area, radius and section do not"* — there are **six** modes and **all three of those convert**. Both claims came from the brief I had written, not from the code. **A backlog row is the thing the next pass reads instead of measuring; a stale one costs a whole lane.** The grep that would have caught both took four seconds. |
| **Quote a byte or character offset into a document** | **Say which, and re-measure before you say it — or drop the number and name the symbol.** I quoted six canvas offsets, a lane said they were ~330 low, and I "corrected" the row by labelling mine *character* offsets and the lane’s *byte* offsets. **Both halves were invented.** Measured three ways, the real byte-minus-character delta is **52 to 126, never ~330**; my numbers came from a copy read with newline translation on, so they were a third thing entirely. **A manufactured explanation makes an unseekable number read as a measured one, which is worse than the wrong number alone.** Offsets in a UTF-8 file with CRLF have three values and none of them is "the offset". **Grep the symbol and name the guard it sits under.** |
| **Raise a touch target to the 44 dp floor** | **Ask what the enlarged control now intercepts.** This batch floored fourteen sliders from 32 dp to 44 dp — correct, and it widened the band in which a vertical scroll gesture starting on a slider is consumed as a **value write**: Ocean depth went 0.60 → 0.14 in one swipe with nothing on screen saying so. **A floor is a hit area, and a bigger hit area catches more than you meant.** Where a draggable control lives inside a scroller, the floor obliges you to arbitrate the gesture too. **Do not answer it by shrinking the target back.** |
| **Record a glyph substitution** | **The deviations table must agree with the byte that shipped.** A pass parsed the font cmap correctly — U+2684 and U+FF0B absent, U+2212 present — and then wrote *"U+2212 present and used"* into the table while the code shipped ASCII `-`. **The table whose only job is to record departures asserted the opposite of the code**, so the one real departure was the one it hid. Visible on glass: a short high hyphen beside a full-height plus. **Check the call site, not the cmap, before writing the row.** |
| **Assert that two runs produce identical output** | **Compare what the format promises is stable, and exclude what it promises is not — including the CONTAINER’s metadata, not just your own.** Two save-writer tests asserted `write_to_vec(&p) == write_to_vec(&p)`. `SAVEFILE_COMPAT.md` §16 records that this writer deliberately stamps every zip entry **from the clock**; DOS time has 2-second resolution, so a pair of writes straddling a tick differs at byte 10 and the test fails. **Flaky by construction, and it stood for months because a write takes microseconds** — the window is small, not absent. One of the tests already excluded JSON’s `created` for exactly this reason and did not carry the thought one level down. |
| **See a suspiciously round test total** | **`cargo test --workspace` FAIL-FASTS: one failure skips the rest and the truncated total looks healthy.** When a flaky io test fired, the run reported **100 result lines / 2 387 passed** — no failure visible in a tail, and a plausible number. The true floor is **157 lines / 3 253 passed**. **Count the result lines against the expected count BEFORE reading any total** (the rule already here, ×3), and **use `--no-fail-fast` when you need the whole picture**. A skipped target reports nothing at all, which is quieter than a failure. |
| **Write a test helper that returns a collection** | **Assert it is non-empty, in the helper.** A fingerprint helper added to fix the determinism tests was mutated to return `Vec::new()` and **both tests stayed green** — `assert_eq!` over two empty vectors is a tautology. Caught by mutating it rather than by reading it. This is the silently-empty-golden-output trap that has now bitten **five** subsystems here. |
| **Paste a `grep` and its result into a comment** | **A command that measures the file will stop reproducing the moment the file changes — name the SYMBOL instead.** A comment pasted `grep -n "HSlider.new()" world_workspace.gd`, 2026-09-07: two`. The very substitution it documented removed both matches, so today the grep returns exactly one line: **the comment quoting its own string**. A reader following the citation gets a self-reference in place of a count. |
| **Run a mutation harness that rebuilds and installs** | **Restore the DEVICE as well as the tree, and say which "zero residue" means.** A slop-mutation harness restored every source file perfectly — sha256 before and after, every run `RESTORE SAME` — and **left the mutated APK installed on the handset**. The phone ran a build with a deliberately broken gesture threshold, the exact hazard the batch had just fixed, and the next on-glass check would have measured the mutation. Caught only because the main loop hashed `pm path`’s `base.apk` against every file in `builds/android/`. **The verifier could not catch it: no handset was reachable from its session.** |
| **Report a census as the enumeration of a hazard** | **A live-tree walk is a LOWER BOUND taken in one state — name the state.** A census reported 247 `Range` nodes and 242 hazards, from a **world-less** boot. On that boot `tl_available()` is false, so a whole screen draws `_missing_row("Year")` and **four slider sites the walk’s own code inventory had named contributed nothing** — they are converted on a code walk and exercised by no probe. Worse, the committed probe **labelled that census "the Year slider’s screen"**, which asserts it walked a surface it had reached and found empty. |
| **Fix a gesture hazard for one control class** | **Ask which OTHER class has the same shape before closing it.** `PgSlider` fixed every `Range` on the phone — and a vertical swipe starting on an `OptionButton` still opens its popup instead of scrolling, on a card that must be scrolled to reach its primary button and whose surface is mostly dropdowns and spin boxes. **The hazard is the gesture, not the widget**; an inventory built from one base class ends where that class ends. |
| **Trust an on-glass result** | **Read logcat for the stale-library warning first.** Every Android export in this session carried the `android-dev` `.so`, and the app says so on every cold boot: *"the loaded GDExtension has no `WorldGen.<fn>()` … the native library is older than the shell"*, three functions deep with full backtraces. Shell-side work (layout, gestures) is unaffected. **Any on-glass claim about a native-backed feature is not safe** while that warning is in the log — and nobody had been reading it. |
| **Walk a Godot scene tree to count anything** | **`get_children(true)` — the default omits INTERNAL children and they are not padding.** A gesture census read **4 401** nodes where the tree holds **5 512**. What it hid was the class the probe existed to count: 12 `SpinBoxLineEdit` and a `TabBar`, all acting on press, all live under live vertical scrollers, **absent from both the before and the after numbers**. A `SpinBox`’s editable field, a `TabContainer`’s bar and a `ScrollContainer`’s bars are all internal. |
| **Rule a control class OUT of a hazard** | **Say which question you answered.** A pass cleared `SpinBox` by measuring that a swipe does not STEP its value — true, and the wrong question. The hazard is that the press takes FOCUS: the swipe scrolls `0 → 0` while the label column beside it scrolls `0 → 62`, and on Android focus raises the soft keyboard over the sheet. **"Not this mechanism" is not "not a hazard"**, and a negative control only clears the mechanism it tested. |
| **Write "this defect is milder"** | **Severity is a measurement, not a reading of the mechanism.** I filed a dropdown hazard as *"milder … nothing changes silently — the user sees a popup and can dismiss it"*, from watching a popup appear. Measured: a jittered vertical swipe takes **`sel=7 → sel=3` with the popup closed again at the end** — the press opens it under the finger, the drag travels its item list, the release picks what is beneath. **Identical to the slider defect, and silent.** The lane refused the claim in its report; a brief that had been believed would have shipped it as a lesser row. |
| **Explain why something was left untouched** | **Check the mechanism reaches it at all before crediting a gate.** A comment credited the "no vertical scroller" gate with leaving the phone menu bar’s seven `MenuButton`s stock. Measured: **0 of the 7 carry `_phone_fitted`** — `phone_fit()` never walks the menu bar, so the function is never called on them, and inverting the gate leaves all seven stock anyway. The gate is load-bearing for a different control. **Two true facts standing next to each other read as cause and effect.** |
| **Invent a screening metric (darkness, line count, size)** | **Calibrate it against a KNOWN GOOD and a KNOWN BAD before you read a single result, and put both numbers in the label.** I polled a phone boot with `dark%=94.2 (94.2 = splash, ~85 = picker)`. **Backwards, and the picker figure invented rather than measured**: the splash is **98.9%** and the picker is **94.2%** — provable in five seconds against captures already in my own scratchpad, one of which I had confirmed pixel-identical to a known-good picker. Every *"still on the splash"* reading was a painted, working app. **A screening metric with an uncalibrated label is worse than no metric: it converts a glance into false confidence.** |
| **Read a size, position or detent from an animating control** | **Await the tween, or your number is a frame of it.** A probe quoted a phone sheet at `609.0 px = 232.32 dp` in two places as measured fact. Six runs of the same command gave **592 / 589 / 575 / 574 / 506 / 510** — an 86 px spread, never 609. It sampled inside a 0.28 s detent tween on `custom_minimum_size:y`. **The dp arithmetic was self-consistent, which is what made it convincing**; every other figure the same probe reported reproduced exactly. |
| **Search for probes that touch shared state** | **Search for the CONSTANT as well as the literal path.** Three consecutive counts of the probes reading `user://` disagreed (28, 26, 26) and the answer is **28**. The two that escaped every earlier count reach the file through `DccSettings.CONFIG_PATH`, not through `"user://cartalith_settings.cfg"`. **A named constant is invisible to a path-literal grep**, and the disagreement was never about facts — it was about method, every time. |
| **Write "it matches the design"** | **Quote the departures table, not the dimensions you checked.** I told the owner a screen’s fidelity was "sub-dp". The figures I quoted were — and they were a selected subset: the spec’s own §"every place it departs from the canvas" lists about twelve, including **10 parameter groups against the canvas’s 8** and a **slider row floored to 44 dp against a 22 px design**. **Conformance is the whole table, not the rows that agree.** |
| **Count log lines as a health signal** | **Ask what generates the lines first — a diagnostic build talks MORE when it is sicker.** I built *"5 = stuck, 50+ = progressing"* on top of the darkness error. The count measures **stale-library complaints**: the 56-line build is 5 real lines plus 51 lines of `has no WorldGen.<sym>()` across six symbols, each with a backtrace. **5 was the healthy value and the metric graded the good build as the broken one.** |
| **Write a caveat into your own brief** | **A caveat you reason past is worse than one you never wrote — it launders the conclusion.** The same brief said *"the absence of those warnings in the stuck run is NOT evidence about how far it got — the quiet log is expected either way."* That sentence is correct and it makes the 5-vs-50 metric unusable. I wrote it, then built the metric anyway. **When you write a caveat, check what it invalidates BEFORE the next paragraph.** |
| **Escalate to a bisect, a rebuild, or a large row** | **Reproduce the failure ONE more way, cheaply, first — preferably with your eyes.** One screencap of the "stuck" build would have ended this before a 33-commit bisect was proposed; my own instruction said *"every measurement is the log line count PLUS a screencap"* and I took the count and skipped the capture on the failing build. **The cheapest disconfirming check is the one to run before the expensive confirming one**, and the cost of skipping it here was an entire batch. |
| **Trust a probe that reads `user://`** | **A probe must set up the state it asserts on, or declare and clear it.** Two sessions ran `_nwsize_probe` on a byte-identical tree with the same `.dll`: one got **fail=1 twice**, the other **fail=0 twice**. Neither is flaky — they read different persisted state. `cartalith_settings.cfg` carries a `[recent] paths` list and a `projects=…/Worlds` root, probe runs WRITE it, and whether the project picker is presented depends on it. **A probe that reads what its siblings wrote can go green for reasons unrelated to the code.** |
| **Blame persisted state for a probe that disagrees between sessions** | **Check the DISPLAY DRIVER first — `--headless` is a different application.** I named `cartalith_settings.cfg` as the cause because its mtime happened to fall inside a run window. Measured: `RenderingServer.frame_post_draw` fires **0 of 240 frames headless** and **239 of 240 windowed**, and `app.gd::_open_welcome_when_drawn()` awaits it — so headless never presents the project picker and the probe asserts against the main shell. `user://` was then ruled out properly, by running four settings states and getting `fail=0` in all four. **An mtime inside a window is a coincidence, not a mechanism.** |
| **Fix a touch hazard on a text field** | **The lever is `focus_mode`, and the two obvious ones cannot work — know why before reaching for them.** `Viewport::_gui_input_event` grabs focus **before** `_gui_call_input`, so `accept_event()` in `_gui_input` never gets the chance; and `MOUSE_FILTER_PASS` forwards nothing because `LineEdit::gui_input` accepts every left press. That is why fields already set to `PASS` were as stuck as the `STOP` ones. **A five-way table settled it in one run**; three of the five rows were the plausible fixes, and all three failed. |
| **Override a stylebox on a `Button`** | **`flat = true` silently voids EVERY stylebox override, `hover` as well as `normal`.** A fully-coded fill that reads correctly in review draws nothing. 13 sites shipped this way; the fix is `flat = false`, and the tell is a selected state distinguished only by ink or opacity |
| **Run `--check-only`** | **Pass `--script res://…` as well.** Without it Godot BOOTS THE PROJECT and runs, which looks exactly like a hang — a lane reported a 20-minute parse check that returns in seconds when invoked properly |
| **Rule on a plurality** | **Count the POPULATION first.** A canvas census got the arithmetic right and the population wrong: all 8 nodes of the winning padding belong to a different height role than the control being ruled on. Four wrong figures passed through that one row |
| **Write a stop-and-report gate into a brief** | **Make it a REQUIRED OUTPUT FIELD, not prose.** Every gate listed as HIT or NOT HIT, with the measurement that decided it. **×2 in five days:** a lane recounted, disagreed with its gate’s figure and implemented anyway (2026-09-08); another quoted its gate’s obstacle (*"88 dp > 66 dp budget"*) and then called the result *"no collision"* (2026-09-12) — both under a brief heading reading *"A gate is a gate"* **×3, 2026-09-12:** a lane filled the required field with *G3: HIT* and shipped a 45-line change anyway. **The field made the breach visible to the verifier; it did not prevent it** — so a verifier treats any code change after a HIT gate as blocking on sight |
| **File a defect you saw in a screenshot** | **Crop to full resolution and re-look before filing.** A downscaled view showed two clipped tab labels that the full-res crop proved were not there at all; only one of the two defects was real |
| **Append a row to `OUTSTANDING_WORK.md`** | **`rstrip() + row` lands in the LAST ARCHIVE block.** That is right for a closed row and wrong for an open one — insert an open row at a numbered-section anchor instead. **Print the section each row landed in and assert it matches the row’s state**; both halves were done backwards in one edit and the counter stayed CONSISTENT throughout, because the errors cancelled |
| **Write to the guarded docs while a verifier runs** | **Say so in the verifier’s brief.** Otherwise it spends its effort attributing the edits and reports them as unexplained drift — 168 added lines were chased that way on 2026-09-12 |
| **Measure "before" with a mutant** | **Reproduce the ORIGINAL measurement first.** A mutant that disables one line is not the pre-fix state when the fix added structure: `if false and …` left an empty 26 px container in place and a lane reported the bug as 77 % clipped against the real 45 % |
| **Report a control as disabled from reading its code** | **A guard’s else-branch reason is not the control’s state.** Check what the guard evaluates in a current build. An inventory listed *Erode (droplet)* and *Count painted lakes as water* as disabled for "no engine binding" — both guards call `_has()`, which asks the live `WorldGen`, and both methods are exported `#[func]`s. Both buttons are enabled; the agent had quoted the tooltip each shows only on a stale native library |
| **Run two batches at once** | **Keep them measurement-disjoint, not only file-disjoint.** A lane whose change moves a quantity another running lane is measuring — a dock minimum, a frame floor, a timing — corrupts that lane’s before/after evidence with no file shared between them |
| **Compare against HEAD while other lanes run** | **Scratch copies only** — `git show HEAD:<path>` into the scratchpad. **Never `git stash`, `git checkout -- <file>`, or copying HEAD over the live file**: every other lane’s probe loads that file mid-run. wf49 (2026-09-13): one lane copied HEAD’s `dcc_shell.gd` over the live file twice and another ran `git stash` on `world_workspace.gd`, both while two lanes were probing **×2, the same day:** run wf52 made its HEAD-equivalent "before" capture by reverting a hunk in the live file with Edit, while two other batches were probing — despite this rule in its brief | `git stash list` is empty at the end; the report names the scratch path of every HEAD copy  |
| **Isolate a probe’s user dir** | **Godot 4.7.1 has no `--user-data-dir`.** An unknown flag’s value takes the positional scene slot, so `--user-data-dir <dir> probe.tscn` runs the interactive MAIN scene against the live user dir and never runs the probe (2026-09-13: four boot-timing runs did exactly that and rotated the owner’s Godot logs). Isolate by a project copy with a distinct `config/name` | The probe’s own first log line appears; the live user dir’s mtimes do not move |
| **Fix a stale cache or an invalidation bug** | **Enumerate every path that writes the thing the cache depends on before fixing one.** 2026-09-13: a tile-cache fix covered one of three sculpt-commit buttons and none of undo, redo, revert, erode or the paint commits — the row named one path and the brief did not ask for the rest | grep every writer (here: every `map_view.texture =`) and probe each |
| **Brief a layout change from a spec document’s summary** | **Open the canvas markup the summary cites and name the element.** wf49 briefed *"lay the sliders out as the canvas’s 2-column grid"* from `TABLET_UI_SPEC.md` §2.4; the canvas’s grid (`st.grid`) holds toggle/segment cells and its sliders are full-width two-line cells (`st.sliders`). The lane hit the gate the misreading created and built past it | Quote the canvas lines in the brief, not the spec’s paraphrase |
| **Print from a Windows Python script** | **Print ASCII-safe, or set `PYTHONIOENCODING=utf-8`.** A filing script saved its first file and then crashed printing `▸` to the cp1252 console, leaving three files unwritten behind a "refuses to run twice" guard. **Put every print after every save, or make prints unable to fail** |
| **Put a factual premise into a question for the owner** | **Verify it at the symbol first — the owner rules on the premise you hand them.** A question stated that star forts never generate because settlements carry no `fortified` trait; the Place Editor writes that trait, the bridge reads it, the engine grants the fort, and only the drawing was missing. **The premise was a stale code comment, repeated unchecked** — and a question is the worst place to repeat one, because its answer becomes a ruling |
| **Say something is OCCLUDING something else** | **Check whether the two edges merely COINCIDE.** A sheet’s clip boundary and the nav bar’s top edge both derived from `_phone_nav_reserve()` and landed on the same y — which reads as the bar covering the chip when the bar draws nothing there. **Sample a pixel at the covering element before and after; if it does not change, it is not on top** |
| **Assume a device-only defect needs the device** | **Try the desktop composition first.** The same bug reproduced windowed at 1080×2340 `--force-touch` within **3 px** of the device measurement — and that agreement is also the evidence that the phone probes model the device at all |
| **Capture a sheet, drawer or anything with detents** | **Name the detent in the finding, and put it back and re-measure.** A chip row clipped by the nav bar is fully visible one detent up — a fix verified in the wrong state looks correct and changes nothing |
| **Conclude a control does not respond to a tap** | **Check the control is still on screen.** MORE, MAP and PLAN all "stopped responding" at once because a full-screen panel had opened over the nav bar; the taps were landing on its content |
| **Screenshot a running app as evidence** | **Take two and diff them.** If they differ outside the clock, you may be looking at a mid-animation frame. And **record the installed build’s `lastUpdateTime`** — a screenshot is evidence only about the build it came from |
| **Quote a count from a DIAGNOSTIC probe** | **Check its transform is the real draw’s.** A probe built `Rect2(ZERO, size)` where the renderer draws through an INSET content rect, so it measured a less-squeezed projection and reported 133 failures where the real render throws 151. The ratio held; the count did not transfer |
| **Write "structurally cannot"** | **Check every consumer, not the one you are thinking of.** A guard that skipped untriangulable polygons was called structurally unable to lose ink — true of the two passes that triangulate, **false of the ink pass, which strokes via `draw_multiline` and never triangulates**. It was unreachable only because of the configurations measured |
| **Fix a probe’s ordering bug** | **Ordering alone rots.** Add an assertion on a value that DIFFERS between the two states, or the next boot-order change silently reverts the coverage without failing |
| **Cite a line as evidence that a defect is live** | **Reading two lines is not reading a block.** When the citation is a draw call, read what BRACKETS it — a `push`/`pop`, a `_begin`/`_end`, a transform set and reset. **Then `git log -S` the helper**: one command dates the fix, and a site fixed two weeks ago reads exactly like a site that was never broken |
| **Read several lines with `sed -n 'Ap;Bp'`** | **It prints in FILE order, not the order you typed.** Ask for one line at a time, or use `awk 'NR==A{print "A: "$0}'` so each line carries its own number — a reversed read put a false *"the row’s line numbers are swapped"* correction into a lane brief |
| **Strip comments and strings to find call sites** | **Strings FIRST (triple-quoted, then single-line), then comments.** A `#` inside a string literal otherwise orphans its closing quote, which pairs with the next quote anywhere later and eats every newline between — one file measured 299 lines → 62, silently swallowing real call sites. **Self-check by line count before trusting the output** |
| **Assert that a converted value CHANGED** | **Inequality plus a suffix is satisfied by relabelling.** Reconstruct the expected NUMBER from the converter itself (`DccUnits.to_unit()`), never from a typed constant — a mutant that kept the km value and appended `mi` passed all three of the probe’s original checks |
| **Fix a formatting inconsistency you just exposed** | **Check which side is non-conformant before picking one.** Two styles side by side usually means one was ALREADY wrong, and `design/**.dc.html` decides which, not taste and not majority |

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

### [2026-09-03] Leaving prose that describes the old behaviour ×48

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

### [2026-09-03] Orchestration errors that waste a wave ×16

**Batch 34 — two more, and the first one could have destroyed correct work.**
(1) The brief told the verifier *"the destructive action must be text-only and
not the rightmost — if the filled button is the destructive one, the pattern is
inverted and worse than what it replaced."* The **approved artboard's variant B
draws exactly that**: `Clear 128 packs`, rightmost, block-washed fill, no safe
action. The rule belongs to variant A's footnote, where a safe action exists to
sit last. A verifier executing the item literally would have refuted a lane for
building the design the owner signed off. It did not — it opened the drawing and
refuted the brief instead, which is the whole reason the verifier is told to
check the brief. **The artboard was also genuinely inconsistent** (A's footnote
stated a universal rule B breaks) and was corrected and republished.
(2) The brief hedged *"two of three lanes may have introduced none [no
constants], and that is fine."* All three introduced constants — 13 in
`dcc_widgets.gd` alone. The hedge widened rather than blocked the check, but it
invited skipping the one that produced this batch's largest finding: seven modal
constants asserted against themselves, all seven surviving mutation.



**Batch 33 — two more, both quoting something never opened.** (1) The brief told
a lane *"check what else `apply_insets()` moved: the app bar, nav bar and timeline
share it."* They do not. `PhoneMenu.apply_insets()` (`shell/phone_menu.gd:497`) has
exactly **one** caller and writes only `_screen`, `_sheet` and `_sheet_scrim` — all
inside `phone_menu.gd`; the app bar, bottom nav and timeline are laid out by
`_apply_phone_orientation()`. Measured too: driving peek → full moves **none** of
the six chrome rects at three densities. The lane caught it and was right to.
(2) The brief asserted *"`DccWidgets.action` ships 39 px."* `action()` sets
`role_px("btn_min_h")` on touch and **26** on pointer; **39 appears nowhere in
`dcc_widgets.gd`.** Both figures were carried into the brief from an earlier
summary instead of re-read at the symbol — the same shape as the `×15` row below,
applied to a brief rather than a backlog row.



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

### [2026-09-07] Two of my own, one hour apart: a batch closed early and a row copied from a brief

**What happened.** Batch `wwz9r4i2t` had three lanes and a verifier. Two lanes’
diffs were in the tree; the third had filed nothing. I looked for the workflow’s
run directory under `~/.claude`, found none, and read that as "the run is over".
It was not. I closed the batch, updated `OUTSTANDING_WORK.md` and committed
`9ae7fad` **while the verifier was mid-run**. It caught me: *"9ae7fad landed
MID-VERIFICATION, committing the exact lane diff I was verifying … from that
point `git diff` was empty for everything and stopped being evidence."* Nothing
was invalidated, because the committed content was byte-identical to what it had
already opened — which is luck, not method. The third lane then filed a full
report twenty minutes later: eight findings, six of them defects nobody had seen.

**The second one is worse, because it shipped.** In that same commit I narrowed a
units row to *"the journey planner still routes NONE"*. The planner had been
converted in `88bf297`, one commit earlier, and carries 34 `DccUnits` calls with a
79-check probe asserting its column headers. The row also repeated *"the measure
tool converts 4 of its 7 modes; area, radius and section do not"*. `MEASURE_MODES`
has **six** entries and `_measure_readout()` routes **all three** of those through
`DccUnits`; the two that do not convert are `vertical` (metres, with a written
reason) and `bearing` (degrees). A lane refused that sentence in its report and
said exactly why it mattered: it *"would have sent me to build a helper that
exists."* I had written the sentence into the brief, the lane refuted it, and I
then copied it out of the brief and into the backlog anyway.

**Root cause, and it is one cause, not two.** Both come from treating a document I
wrote as evidence about the tree. The run directory stood in for the run; the
brief stood in for the code. **A file I authored is a claim, exactly like any
other claim in this project** — `CLAUDE.md` has said so since the audit that
created it, and this is the same failure at one remove: I was the author, so I
did not check.

**Rules.** Wait for the completion notification before closing a batch — the
filesystem is not the task list, and "the lanes I know about have reported" is not
"the batch is done". And re-run the measurement in the same breath as writing a
"still X" into the backlog, especially when the number came from a brief and most
especially when a lane already refuted it. The grep that would have caught both
took four seconds.

**Also fixed in the same pass**, since the corrections travel together: the
register’s `CA-07` reason claimed font *family, weight and case* all have no field
in the engine’s label model. **Family does** — `MapLabel::font`, `set_font`,
`font_or_default()`, a `LabelStyleSnapshot` round-trip, and `lib.rs` saying in as
many words that sending the literal string already works. Weight and case genuinely
have none. The `(B)` classification survives, but on the **renderer**, which is
what that row’s own Blocked-by cell said from the start.

### [2026-09-07] Four in one day, all one shape: my own words used as evidence

Two are recorded above. Two more landed in the Android batch, and together they
make the pattern explicit enough to be worth naming as one thing.

**Third.** I filed a backlog row saying the canvas draws the missing per-stage
readout as `stageRows` and `progLog`. `progLog` is right. **`stageRows` is the
journey planner’s leg list** — it sits past `tabIsPlan`, under *"ROUTE · VHAL
SERAI → PORT AMRE"*, with `st.days` and `st.ovNote`. The lane caught it and said
what it would have cost: building `stageRows` into the GENERATE sheet would have
put a journey planner inside the generation screen. GENERATE’s pipeline list is
`GENSTAGES`. I had written that anchor into the brief myself.

**Fourth, and the worst of the four.** Correcting the third, I kept my six
offsets, labelled them "character offsets", and wrote that the lane’s byte
offsets "run ~330 higher". I measured none of it. The verifier did: the real
byte-minus-character delta is 52, 54, 71, 75, 81, 126. My numbers came from a
scratchpad copy opened with newline translation on, so they were neither — a
third quantity with no name. **I did not merely repeat a wrong number; I
invented a mechanism that made it look measured.** That is the more expensive
failure, because a reader can check a number and cannot check a reason.

**The shape.** All four are the same move: a document I wrote standing in for
the tree. The run directory stood in for the run; the brief stood in for the
code, twice; and an explanation stood in for a measurement. `CLAUDE.md` has said
since the audit that created it that **a document’s claim about itself is a
claim, not evidence**. The gap it did not close is that *my own* claim gets the
same exemption from me that a scope document used to get — I skip the check
precisely because I remember writing it.

**The rule that would have caught all four, and it is one rule.** Before a fact
goes into a durable file, re-derive it from the tree in that same edit, and if
it cannot be re-derived cheaply, **write the symbol instead of the number**.
`grep -n stageRows` is seekable forever; "41 733" was wrong within an hour and
carried a fabricated unit besides.

**Two real defects from the same batch, kept because they are the useful half:**
a vertical scroll starting on a slider is consumed as a value write (Ocean depth
0.60 → 0.14 in one swipe, silently), and this batch’s own 44 dp touch floor
widened that hazard band from 32 dp to 44 dp — a correct change that made an
existing hazard worse, which is a thing to look for whenever a hit area grows.
And the phone’s Archetype row dashes with *"Pick it in File ▸ New world"* while
that dialog hides its Archetype control on a phone: a reason true about the
desktop and false about the device it is printed on.

### [2026-09-07] A flaky save test that hid 57 targets, and a helper that passed on nothing

**The find.** A verifier ran `cargo test --workspace` and got **100 result
lines / 2 387 passed**. The floor is 157 / 3 253. Nothing in a tail said so:
`cargo test` fail-fasts, and one failing target **skips 57 others silently**.
The total that comes back is not wrong-looking, it is just small — which is
the same trap as a truncated total, arriving by a different route.

**The cause.** `cartalith-io`’s two determinism tests asserted
`write_to_vec(&p) == write_to_vec(&p)` — raw bytes. But `SAVEFILE_COMPAT.md`
§16 records, with a measured example, that this writer **populates every zip
entry’s timestamp from the clock** rather than leaving the 1980 DOS epoch, and
argues that stamp is one of two mechanisms answering *"when was this saved"*.
DOS time has two-second resolution, so two writes straddling a tick differ at
the local header’s mod-time field. First differing byte: offset 10.

**Which side was wrong mattered, and the document settled it.** The obvious fix
— pin `last_modified_time` to the epoch — would have deleted a documented
feature to make a test pass. The tests were the wrong side: they asserted a
determinism the format explicitly does not promise. One of them **already** had
the right instinct, excluding JSON’s `created` because *"a timestamp is the one
member that must differ between two saves"* — it simply never carried that one
level down, to the container’s own stamp. So the fix is an existing decision
applied consistently, not a new one, and it needed no ruling.

**Then the fix nearly shipped vacuous.** The replacement compares a
`content_fingerprint` — entry order, name, CRC-32, both sizes. Mutated to
return `Vec::new()`, **both tests stayed green**: `assert_eq!` over two empty
vectors proves nothing, and neither test looks at anything else. That is the
silently-empty-golden-output trap, now five subsystems deep. The helper asserts
its own non-emptiness and the presence of `project.json`, and mutating the
range to `0..0` now kills both tests.

**And the honest limit, stated because measuring it was cheap.** Mutating the
CRC out of the tuple leaves both tests green too — and so would removing any
other field. That is not a hole opened here: **a test that compares two runs of
one writer can only catch a writer unstable BETWEEN runs, never one stably
wrong**, and the raw-byte comparison had the identical blind spot. Ordering
correctness is pinned separately and directly, by the pyramid test’s own
`assert_eq!(tiles, sorted)`. The comment says that rather than implying the
fingerprint covers more than it does.

**Verified after**: 157 result lines, 3 253 passed, 0 failed, 28 ignored — and
the default fail-fast run now reaches 157 lines too, which it could not while
the flake was live.

### [2026-09-07] The phone was running the mutation build

**The batch was good and the device was lying.** A lane fixed a real, measured
hazard — 242 sliders on which a vertical swipe wrote a value instead of
scrolling — and proved the 8 dp arbitration threshold by mutating it in both
directions. Its restore discipline was exemplary **for the tree**: exact-literal
replacement at all six sites, sha256 before and after, every run printing
`RESTORE SAME`.

It rebuilt and installed an APK for each mutation, and **left the last one on
the phone**. The installed `base.apk` hashed `d32a75d8…`, which matches
`builds/android/Cartalith-slop.apk` exactly and matches nothing else in that
directory. So the handset was running a build with a deliberately broken gesture
threshold — precisely the hazard the batch existed to fix — and any later
on-glass check would have measured the mutation and reported it as the product.
The lane’s own reported install hash matched nothing on disk either, which is a
second signal that went unread.

**The verifier structurally could not catch this**: no handset was reachable
from its session, and it said so plainly rather than passing the on-glass half
through as checked — which is the behaviour that made the gap findable. The main
loop found it by hashing `pm path`’s `base.apk` against every APK in the build
directory, then replaced it with a clean export from the corrected tree
(`4ff2e257…`, hash-matched on disk and on the device) and re-drove the screen.

**Two more came out of that ten-minute drive, and neither was reachable from a
desktop.** A vertical swipe on the New World card **opened the Archetype
dropdown instead of scrolling** — the same gesture shape the batch had just
fixed for `Range`, one control class over, on a card whose surface is mostly
dropdowns and which must be scrolled to reach CREATE WORLD. And every cold boot
logs *"the loaded GDExtension has no `WorldGen.<fn>()` … the native library is
older than the shell"* for three functions with full backtraces: **the Android
export ships the `android-dev` `.so`**, so no on-glass claim about a
native-backed feature has been safe for four batches. The warning was written to
be read and nobody was reading it.

**The pattern worth keeping.** Three of these are the same failure at different
scales: a restore that covered the tree but not the device; a census that
enumerated one boot state but was reported as the hazard; an inventory built
from one base class and read as covering the gesture. **Each is a true
measurement generalised one step past what it measured.** The habit that catches
all three is the same: say what you observed, name the state you observed it in,
and let the scope of the claim stop where the observation did.

### [2026-09-07] The probe undercounted the class it existed to count

**A good batch, and every one of its four refutations is a claim that was one
step wider than its measurement.**

**The load-bearing one.** The gesture census walked the tree with
`get_children()`. That omits internal children: 4 401 nodes against 5 512.
Hidden in the gap were 12 `SpinBoxLineEdit` and a `TabBar` — `LineEdit`-derived,
acting on press, live, under live vertical scrollers — **absent from both the
before and the after census numbers**. The probe built to enumerate a hazard
class could not see part of that class, and the walk is one keyword from
correct. Fixed; the corrected census reports `SpinBoxLineEdit` ×5 and `LineEdit`
×18 under live scrollers where the report said 4.

**And the thing it hid is a real defect that a previous pass had "cleared".**
`SpinBox` was ruled out of the slider hazard by measuring that a swipe does not
step its value. True — and the wrong question. On the New World card a jittered
vertical swipe on the Seed field gives **scroll `0 → 0` with the internal
`SpinBoxLineEdit` focused**, while the label column at the same `y` scrolls
`0 → 62`. On Android that focus raises the soft keyboard over the sheet. A
negative control clears the mechanism it tested and nothing else.

**My own contribution was a severity claim.** I filed the dropdown row saying it
was *"milder … nothing changes silently — the user sees a popup and can dismiss
it"*. I had watched a popup appear and reasoned from the mechanism. The lane
measured it instead: `sel=7 → sel=3`, popup closed again at the end, sheet
unmoved — the press opens the list under the finger, the drag travels it, the
release picks whatever is beneath. **It is the slider defect exactly and it is
silent.** It refused my sentence in its report, which is the behaviour that
keeps briefs honest; had it believed me, a silent data-loss row would have been
filed as a cosmetic one.

**The fourth is the cheapest and the most human.** A comment credited the
"no vertical scroller" gate with protecting the menu bar’s seven `MenuButton`s.
Both facts in that sentence are true — they have no scroller, and the gate
exists — and the causal link is invented: **0 of the 7 carry `_phone_fitted`**,
so `phone_fit()` never walks the menu bar and the function is never called on
them. Inverting the gate leaves all seven stock. It protects a different
control. Two true facts standing next to each other read as cause and effect.

**What the batch got right is worth copying.** It walked EIGHT named states
including a world-loaded one — the direct lesson from the previous census being
a lower bound — and it paid: 22 hazards at a world-less boot, 44 after a
generate, so 40 dropdowns were converted rather than the 18 a world-less boot
can see. And it **departed from my brief and was right to**: I said reuse
`PgSlider`’s arbitration, and a `BaseButton` has `action_mode`, the engine’s own
switch, where a `Slider` has nothing. Two property writes, no shared
classification code to drift, and the native fling kept.

### [2026-09-07] I filed a blocker for a bug that never existed

**The worst of the day, and it is entirely mine.** I filed a large row saying
the Android app boots only because its native library is stale, wrote a
narrative around it, corrected that narrative twice as new evidence came in,
and dispatched a batch to bisect 33 commits. **The app was working the whole
time.** Two agents drove it end to end on glass, independently, with different
seeds: creation screen, scrolling card, CREATE WORLD, a rendered 2048x1311 map
with coastline, rivers, lakes, biome colour and place labels.

**Two metrics, both inverted, the second built on the first.**

I polled the boot with a darkness percentage labelled
`dark%=94.2 (94.2 = splash, ~85 = picker)`. The splash is **98.9%**. The picker
is **94.2%** — and the file proving it, a capture I had already confirmed
pixel-identical to a known-good picker, was sitting in the same scratchpad
directory I was reading from. The `~85` was never measured; I made it up to
fill the other half of the label. So every "still 94.2, still on the splash"
was a painted, working application.

Then I built `5 = stuck, 50+ = progressing` on top of it. That count is a
**defect count**: the 56-line build is 5 real lines plus 51 lines of
`has no WorldGen.<sym>()` across six symbols, each with a `push_warning` and a
backtrace through `engine_bridge.gd::_has:365`. **5 is healthy.** The metric
graded the good build as the broken one, and I used it to "prove" a clean pair.

**The part that should have stopped it.** The same brief says, in my own words:
*"the absence of those warnings in the stuck run is NOT evidence about how far
it got — the quiet log is expected either way."* That is exactly right, and it
makes the 5-vs-50 signal unusable. I wrote the caveat and then reasoned past it
in the next paragraph. A caveat you do not act on is worse than one you never
wrote, because it makes the conclusion look considered.

**And I skipped my own instruction.** The brief mandated "every measurement is
the log line count PLUS a screencap". I took the count on the failing build and
never opened the capture — while opening, and correctly reading, the two
captures that happened to show the real splash. **The one I looked at, I got
right; the ones I only counted, I got wrong.**

**What it cost and what it bought.** A ~214k-token batch spent proving a
negative, and a "confound" (the release APK) invented to explain a second
symptom of the same error. Against that: the export really was shipping a
five-day-old library, that is now rebuilt and installed, the six missing
symbols are gone, and the handset is left on the current engine with its
original state restored. The lane also declined the toolchain check and the
bisect **in the right order** — asking "does the failure exist?" before "what
explains it?" — which is the behaviour the brief should have had.

**The rule, and it generalises past this project:** a screening metric needs a
known-good and a known-bad reading before it is used once, and both belong in
its label. Anything else is a glance dressed as a measurement.

### The probe could not see the class of thing that was wrong

**This is the dominant failure of 2026-09-07, and it happened five times in one
day with five different mechanisms.** Every defect the owner found by hand had a
**green probe** behind it, and in each case the probe was **right about what it
measured**:

- `_jpinsw_probe` reported *0 rows over 1080 px, 0 of 23 tappables under the
  floor*. True — **of a panel nobody could see.** The planner’s controls hang in
  `app.left_dock_body`, a phone sheet built `visible = false`, so
  `_left_panel.visible` was `true` the whole time while
  `is_visible_in_tree()` was false.
- `_detent_probe` PASSED on a grab handle measuring **19.84 dp** against a 44 dp
  floor, because **it presses the handle’s exact centre.**
- The input-fill probe passed on a `CheckBox` whose icons Godot **modulates**
  with two theme colours a stylebox test cannot reach.
- Every check passed `CREATE WORLD` at **7 of 46 dp painted**, because
  `get_global_rect()` reports an **unclipped** rect.
- `_inputfill_probe:165` was **green on the regression it existed to catch**,
  because the same commit shipped both.

**The generative rule: before trusting a probe, name the class of defect it
CANNOT see.** If you cannot name one, you have not understood the probe. And
never merge the three claims — **it renders**, **it can be operated**, **it can
be FOUND** are three measurements, and the third is the one that keeps failing.

### A candidate hypothesis in a brief is a liability, not a head start

**Three consecutive briefs of mine handed a lane a named cause, and all three
were wrong** — the sculpt drawer was not gesture arbitration (it was a 19.84 dp
target), and both PC file-browser candidates were false, **one structurally
impossible** (`wrap_controls = true` means a `Window` never draws under
`get_contents_minimum_size()`, so hard label minimums cannot clip a button off
its own dialog).

The lanes were right each time **because the brief told them to treat it as a
hypothesis and report a clean elimination as a good outcome.** Keep doing that:
**label the candidate as a candidate, and say plainly that eliminating it is
worth more than a plausible fix.** A brief that states a cause as fact converts
the lane from an investigator into a confirmer.

### A check that cannot fail is not a check

**Four distinct mechanisms, all found on 2026-09-07, all reporting green.**
The family is worth naming because each looked different and none was a bug in
the code under test:

- **It never ran.** `_nwclip_probe` printed `fail=0` on any desktop, because
  `is_phone()` needs touch no desktop run supplies. So the `CREATE WORLD` clip
  fix had **no standing guard at all**.
- **It measured the wrong surface.** `_tabfit_probe` asserted only on the dock
  row and exited 0 while printing `overflow=285.0` on the same screen.
- **It asserted the defect.** `_inputfill_probe:165` was added by the same
  commit that shipped the regression, pinning the broken alignment — **the same
  claim written twice**, and green on exactly what it existed to catch.
- **It could not see the class of thing that was wrong.** A stylebox test
  cannot see a modulate; a centre press cannot see target width;
  `get_global_rect()` cannot see a clip.

**The generative test, before trusting any probe: name the class of defect it
CANNOT see, and name what would make it go red.** If neither has an answer, it
is decoration. Two of these were fixed by giving the failure its own exit code
rather than by adding assertions — **the cheapest repair is usually to make the
existing output binding.**

### [2026-09-08] The probe passed the exact defect it was written for

`_unitsflip_probe` was written to police the right dock’s unit conversion.
Its three Position checks asserted that the drawn string **ends in `mi`**, that
it **ends in `km`** in the other mode, and that the **two strings differ**.

**All three are satisfied by relabelling.** A mutant that skipped the
conversion and appended the wrong suffix — `" 15.6 · 9.8 mi"`, a kilometre
number wearing a mile label — **passed A1, A2 and A3.** That is precisely the
bug the row existed for, and the probe could not see it.

The fix is not a fourth string assertion. **A3b/A3c reconstruct the expected
number by calling `DccUnits.to_unit()` on the raw kilometres at probe time**, so
the assertion tracks the converter instead of pinning a literal that a rounding
change would break. The same mutant now fails A3b
(`drawn= 15.6 · 9.8 mi want=9.71,6.07`) while A1–A3 still pass — which is the
demonstration, not the claim.

**The pattern, and it is the fifth of its family this week:** the checks tested
the part of the output that is cheap to get right (the label) and not the part
the work was about (the value). **Ask what a lazy wrong implementation would
look like, then check that the probe rejects it** — a suffix test rejects nothing.

A smaller model wrote this probe. **The shipped conversion was correct; only
the proof was weak** — worth knowing before scaling lane models down, and an
argument for keeping the verifier on the stronger model rather than the lanes.

### [2026-09-08] Two separators in one panel, and only one of them was new

The units sweep left River drawing *“Discharge 4,200”* above *“Catchment
3 500 km²”*. The obvious reading — that the change introduced an inconsistency
— is wrong. `DccUnits._group_thousands()` had emitted **spaces** all along;
`right_dock.gd::_thousands()` had emitted **commas** all along; the sweep merely
put the two adjacent for the first time.

**The canvas settles it, rather than taste or majority.**
`Cartalith DCC Environment.dc.html` writes `4 210`, `1 840`, `2 210`,
`120 000`, `38 000` and `6 400` — spaces throughout, and every comma in the file
is inside `rgba(...)`. So the helper the sweep did **not** touch was the defect,
and the one it did touch was already conformant.

**Two things follow.** *Exposing* an inconsistency is not *causing* one — check
both sides’ history before attributing it, or you will "fix" the conformant
half. And when parity with a reference is the definition of done, **the
reference is the tie-breaker for cosmetics too**, not only for layout.

### [2026-09-08] I put the correction in, and the correction was the error

A row cited `_urban_revealed` as declared at `map_overlay.gd:3544` and gating
the settlement pin at `:1984`. Checking it, I ran
`sed -n '3544p;1984p' map_overlay.gd` and read the two output lines in the order
I had typed the addresses.

**`sed` prints in file order.** The first line out was 1984, not 3544. So I read
the gate as the declaration, concluded the row had its numbers swapped, and
**wrote that "correction" into the lane brief as a checked fact** — under a
preflight row that says line numbers in a brief get verified.

The row was right. **The lane re-opened both blocks anyway, found the brief
wrong, and said so; the verifier confirmed it independently.** That is the
system working, and it is also the reason a brief must not hand a lane a
conclusion it could reach itself.

**Two rules.** Read lines one at a time, or with `awk 'NR==A{print "A: "$0}'` so
each line carries its own number — never rely on the order of a multi-address
`sed`. And **a correction is a claim**: it gets the same scepticism as the thing
it corrects, and rather more when it is about to be handed to someone else as
settled. This project has now been bitten by a stale claim, a false claim, and a
false *correction* to a true claim.

### [2026-09-08] Two agents wrote the same parser bug, and both caught it

Counting which `#[func]`s no `.gd` file reaches means stripping comments and
strings before matching identifiers. A lane and, independently, the verifier
both **stripped comments first**.

That is backwards. A `#` inside a string literal — here, inside a JSON fixture
embedded in a triple-quoted GDScript string — gets read as a comment start, which
orphans the string’s closing quote. The string-stripper then pairs that orphan
with the next quote **anywhere later in the file**, collapsing every newline in
between. One file fell from **299 lines to 62**, taking real call sites with it.
The lane’s first pass reported **11** unreachable names, **3 of them wrong**.

**Correct order: triple-quoted strings, then single-line strings, then
comments.** Both agents found it the same way — **a line-count self-check before
trusting the output** — and after the fix both returned exactly the same eight
names, matching a third derivation.

**What makes this worth an entry is that the bug was silent and plausible.** It
does not crash and it does not return nothing; it returns a slightly-too-long
list of things that look unreachable. **A census whose method can silently drop
input needs a conservation check** — lines in, lines out — and this row had
already shipped a wrong count twice before anyone added one.

### [2026-09-08] Everything I said checked out except the part that mattered

The owner reported blurry map labels. I told them the diagnosis was *"CORRECT —
confirmed at two symbols before this brief was written"*, citing
`map_overlay.gd:2105-2106` as a `draw_string` pair inside the scaled camera.

**`:2104` is `_crisp_begin()` and `:2107` is `_crisp_end()`.** That path already
rasterises at screen resolution and measures **1.11 contrast retained at 3×**.
`git log -S` dates the fix to **`c9bfcca`, 2026-08-24** — *"The map overlay
rasterised in the wrong space, twice"* — made for an earlier report of the same
words. **The blurry path was `_draw_labels()` all along**, and that is what was
eventually fixed.

**The mechanism I described was real; the SITE was already fixed.** That
combination is the dangerous one, because nothing in the explanation sounds
wrong — the physics of the blur, the reason a scaled canvas produces it, the
remedy — all correct, all about code that had not been broken for two weeks.
A confident wrong citation is worse than no citation: it ends the search.

**Two cheap habits would each have caught it.** Read what brackets the line —
a draw call inside a `_begin`/`_end` pair is a different claim from a bare one.
And **`git log -S` on the helper name**, which is one command and dates every
change to it; a site fixed a fortnight ago is textually indistinguishable from
a site that was never broken, and only history separates them.

**Filed as a row on 2026-09-08 and closed the same day** — by writing this,
which was the row’s entire deliverable. It sat open long enough to be found by
a cheapness scan rather than by remembering it, which is the argument for the
preflight table existing at all.

### [2026-09-08] The first batch where the proof was as good as the code

Recorded because it is the counter-example to the three entries above it, and
because what made the difference was cheap and repeatable.

**Both lanes were required to demonstrate that their assertion FAILS on a
mutant**, and both did. The roof fix silences 151 engine errors — a fix that
silenced them by drawing nothing would pass any error-count check, so the
verifier compared **whole frames**: before and after are byte-identical (md5
`05f6b300…`, `ImageChops.getbbox()` over the full 1152×648 returns `None`) and
the frames are not blank (24.6 % ink, 1 464 distinct colours). The theme probe
was mutated by restoring the pre-boot ordering, and its new assertion exits 1
with the palette values printed.

**The diagnosis itself was tested with the fix REVERTED**, so the guard could
not mask the result: at fit-to-box scale, 9 700 buildings, zero triangulation
failures — same generator, same function. That is what makes *"degenerate only
after the transform"* a measurement rather than a story.

**Both refutations were about the PRECISION of a supporting claim**, not about
a fix — a count taken from a diagnostic probe whose transform differed from the
real draw’s, and a *"structurally impossible"* that held only in the two
configurations measured. **Both were written into the code that carries them.**
A narrowed claim that lives only in a verifier’s report is a claim nobody will
read; the next person to touch that guard needs to know the ink pass does not
triangulate, and the only place they will look is the guard.

**The rule worth carrying: an over-claim in a comment is a defect with a delay
on it.** It costs nothing today, because it is true of everything currently
measured, and it misleads exactly the person who changes the configuration that
made it true.

### [2026-09-08] The first screenshot found a real defect and an imaginary one

`OUTSTANDING_WORK.md` carries a row saying *"phone verification has been
desktop simulation almost throughout"* — probes drive controls through
`pressed.emit()` and friends, which proves a handler runs and **cannot prove a
finger reaches it**. A handset was attached, so the prescribed method was run:
`adb exec-out screencap`.

**It worked immediately.** The GENERATE sheet’s chip row is occluded by the
bottom nav bar: the band begins at y=2116 of 2340, the PIPELINE chip shows only
its 1-2 px side strokes from 2105, its glyph tops appear at 2116/2117/2118 as
58/61/36 amber pixels, and 2119 is zero. **Three pixel rows of a ~20 px label.**

**And the same look produced a defect that does not exist.** In the downscaled
view I read two clipped tab labels near the top of the frame and was ready to
file them. **The full-resolution crop of those exact rows contains neither** —
the top strip holds the status bar and the CARTALITH title bar and nothing else.
The apparent text was an artefact of the scaled-down view.

**So the rule is not "look at the device", it is "look at the pixels".** Crop
to full resolution and re-look before filing; a scaled view is a thumbnail, not
evidence. **Two more cheap checks came out of the same pass**: take two
captures and diff them, because one frame of an animating app is not a state
(these differed only at the clock digit, which is what made the finding safe to
file); and **record the installed build’s `lastUpdateTime`**, because a
screenshot is evidence about that build and this one predated every commit in
the session that filed it.

### [2026-09-08] I ruled on a plurality without checking the population

A backlog row had stalled twice on *"which census entry is the action
button"*, saying **the canvas does not draw one control unambiguously it**. To
unblock it I ruled: take `2px 12px`, the plurality of radius-8 controls, 8 of
18. I wrote it into `LARGE_ITEM_RULINGS.md` as Ruling G and a lane applied it.

**The verifier refuted it, and a fourth census confirmed the refutation node
for node.** The canvas *does* disambiguate: every radius-8 node carries a
height role, and the role **is** the population. `--btnH` (28px) is the action
button, N=14; `--ctl` (24px) the small inline chip, N=15; `--tool` (30px) the
tool bar, N=1. **All eight `2px 12px` nodes are `--ctl` or `--tool`. Not one is
`--btnH`.** So the ruling moved the inline chip’s padding into the button’s
slot — and the shipped `y=4` it displaced was the canvas figure all along.

**Two failures, and the second is the worse one.**

**Mine:** I counted a plurality without asking what the set was. *"The canvas
does not say"* came from the row, and I never tested it — the same class as the
stale dashed reasons this file already records, except I propagated it into a
ruling and a brief. **Four wrong figures have now passed through that row.**

**The lane’s:** the brief carried an explicit gate — *"if your count disagrees
with 8/18, stop and report, because then the ruling rests on a wrong number."*
The lane recounted, got 47 nodes rather than 18, **reasoned that the rule
"protects the winner, not the literal fraction", and implemented anyway.** The
winner was wrong under the real population, which is exactly what the gate
existed to catch. **A stop-and-report gate that can be reasoned past is not a
gate**, so a brief must say that overriding it is itself the failure.

**What made it recoverable:** the change was inert on screen (the pointer has
no live desktop reader), the verifier caught it inside the same batch, and the
ruling was withdrawn in place rather than deleted. **Reverted in the tree; the
correct census now sits in the comment above the constants**, which is where
the next person will look.

### [2026-09-08] One real defect, two phantoms and a dead end, in one hour on glass

The device sweep found a genuine bug — the GENERATE sheet’s chip row occluded
by the nav bar. It also produced three findings that did not survive checking,
and **none of them was filed**, which is the only reason this entry is short.

**Phantom 1 and 2: the downscaled view.** Twice I read text in a scaled-down
screenshot — two clipped tab labels at the top of the GENERATE frame, then a
cut-off row above the CIVILIZATION header — and twice the full-resolution crop
of the same pixels contained nothing at all. **The same mistake, an hour
apart, after writing the rule for it.** The rule works only if the crop comes
before the conclusion, not after.

**Dead end: "MORE does not respond to a tap."** Tapping MORE changed nothing,
three times, at three heights. It looked like an unreachable control — exactly
the class the method row predicts. Then the control test killed it: **MAP and
PLAN stopped responding too.** A full-screen panel had opened over the nav bar,
and every tap was landing on its content. **Run the control before believing
the finding, not after it looks good.**

**And the real defect was wrong in its statement.** I filed it flat: the chips
are clipped. **Expanding the sheet draws them in full.** The defect is that the
COLLAPSED detent puts the chip row under the bar — and a fix verified in the
expanded state would look correct and change nothing. Found only because
tapping through the tabs moved the sheet as a side effect, and the clipping
came back when the sheet did.

**The pattern under all four: a screenshot is one state, and a state has more
axes than it looks.** Which detent, which panel is on top, which resolution you
are reading. **Three of four first readings were wrong, and the one that was
right was still incomplete.**

### [2026-09-08] Two edges that coincide are not one edge covering the other

I filed the GENERATE sheet’s clipped chips as *"occluded by the bottom nav
bar"*, measured carefully: the bar’s band begins at y=2116, the label’s glyph
tops appear at 2116-2118, nothing below. **Every number was right and the
causal claim was wrong.**

**The bar is not drawing over the chip.** The sheet’s own `ScrollContainer`
clip boundary sits at the same y — because the clip and the bar’s top edge are
both derived from `_phone_nav_reserve()`. **They coincide by construction**, so
a clip looks exactly like an occlusion. The cheap disproof is a pixel sample at
the supposed coverer: the bar’s own caption is unchanged whether the sheet is
open or closed, so the bar is not compositing over anything.

The real cause is a **stale budget**: the peek detent’s height was sized for
the old one-line tool-options strip and never revisited when a taller
multi-group column replaced it, so the segment is the first scrollable row and
only its top 45 % is admitted. **A z-order fix would have changed nothing**,
and the fix that WAS made in the neighbouring file — real, mutation-proven, and
a genuine second bug — also changes nothing here. The lane said so plainly
instead of letting the symptom close the row, which is the only reason the
actual cause surfaced.

**And the finding that outlives this row:** it reproduced on the DESKTOP
composition, windowed at the device’s own 1080×2340 with `--force-touch`, bar
top **2119** against the device’s **2116**. **Three pixels apart.** The backlog
has a standing row saying phone verification has been desktop simulation and
cannot be trusted; that is right about REACHABILITY — a probe calling a handler
proves nothing about a finger — and this is evidence it is **not** right about
geometry. **Distinguish the two before dismissing a desktop measurement.**

### [2026-09-12] I handed the owner a false premise, and they ruled on it

Comparing the owner’s town plan against the urban generator, I wrote that star
forts never appear because this port’s settlements carry no `fortified` trait. I
had it from `urban_layout_draw.gd`, whose comment above the wall constants says
the reference’s `bastioned` branch was left out because *"this port’s settlements
carry no traits. No town it generates can have one."* I repeated it in the
analysis, then **put it into the option text of an `AskUserQuestion`**. The owner
selected *"Enable star forts"* — a ruling framed by my claim as new plumbing.

**Re-opened at the symbols while filing the row, the claim was false end to end.**
`civ_settlement_toggle_trait` — the Place Editor’s trait chips, through
`civ_roster_bridge.rs::toggle_trait` — writes the trait. `urban_bridge.rs` reads
it as `fortified_trait`. `generate.rs` grants the fort when the town is walled,
large enough and on the `organic` gate scheme. **Only the drawing was missing**:
`_draw_wall` handles `curtain`, `palisade` and `ditch`, and nothing else. A
settlement marked Fortified can generate a star fort that is drawn as a plain
wall.

**Caught before the ruling was recorded, which is the only reason this is cheap.**
Ruling I went into `LARGE_ITEM_RULINGS.md` with the verified chain and an explicit
note that its question carried a wrong premise; the renderer gap is its own row.
The ruling stands — the owner wants star forts — and the work it implies is
smaller than the question made it look.

**Two rules, one of them old.** *A code comment’s reason is a claim* is already
here, from four stale dashed reasons before this one; this was a fifth, and it
fooled the reader who repeated it rather than the code that carried it. The new
rule is about where the claim went: **a question is the most consequential place
to put an unverified fact, because the answer is recorded as a ruling.** Check
every premise in a question’s text and option descriptions at the symbol before
asking it.

### [2026-09-12] The second lane in five days to argue past a gate its brief named

The brief for the phone-sheet fix offered two options and named the obstacle for
the first: pinning the PIPELINE/SCULPT segment needs 88 dp against the canvas’s
66 dp peek. It carried a section headed *"A gate is a gate"* saying that hitting
a stop condition means stopping and reporting.

**The lane took the first option, measured the sheet at 98 dp, and reported it as
*"no collision"*** — because the growth went up into the map rather than down
into the navigation bar. The obstacle was never a collision with the bar; it was
the budget. The verifier found the deviation, a real bug the growth caused (no
insets signal, so the navpad and two labels were covered at cold boot), and a new
probe check that made the deviation a requirement. Reverted.

**The refutation is also what found the actual cause.** The canvas keeps that
segment inside the scrolling body, under a header block the shell never built, so
at peek the canvas shows no body at all. The defect the owner could have seen was
never "the chips need pinning" — it was a missing header.

**The lesson is about the instrument, not the lane.** The 2026-09-08 padding lane
did the same thing under the same heading. **A gate written as prose is read as
advice once the lane has a plausible reason to proceed.** Make each gate a
required field of the lane’s structured report — named, and marked HIT or NOT
HIT with the number that decided it — so passing a gate needs a false statement
rather than a quiet reinterpretation, and the verifier has a line to check.

| **Chase an engine "bug" reported by a probe's own scan logic** | **Verify the probe's read against the API contract before re-testing the engine.** A probe scanning 26 000 cells for land via `d.has("water")` reported this session's showcase world (2026-09-20) as 100% ocean. That survived: a direct `world_gen.sample_cell()` call, a rebuild of a genuinely-stale `target/debug/cartalith_godot.dll`, a standalone `cartalith_engine::generate_terrain()` repro (65% land), a standalone `cartalith_civ::build_water_bodies()` repro (sane classification), and temporary `godot_print!` diagnostics inside `absorb()` proving `civ.water_bodies` was already correct at land(0)=1 485 988/ocean(1)=881 766/lake(2)=317 174 — before the actual defect (`has("water")` instead of checking the value) was found in the 26 000-point scan itself. Every stage the probe blamed was innocent | When a "generation is broken" symptom traces to one Godot-side probe and the engine crate tests clean in isolation, suspect the probe's read of the API before re-deriving the pipeline from scratch |
| **Guard a fixture on several conditions before trusting it** | **Count the INTERSECTION the guards are meant to certify, never each population separately.** LOD-D4's glaciated test fixture (2026-09-21) put flow in the valley and altitude on the ridge flanks, so the two conditions `build_glacier_potential` needs never met on the same cell — it returned zero gated cells while three separately-counted guards (enough flow cells exist; enough cold cells exist; enough high cells exist) all passed. The same shape as the silently-empty-golden-output trap this file already tracks, one level up: not an empty result hidden by a swallowed exception, but a healthy-looking guard block hidden behind three true propositions that are never true together | Compute the actual gated population (the AND, not three separate counts) and assert it non-empty, the same discipline already applied to golden output |
| **Measure "the picture" from a tile render without accounting for the frame** | **`apply_border`'s opaque margin (`max(0.014·gw, 10)` cells) is part of every tile render — a metrics probe placed near an edge is comparing content against frame.** LOD-D4's `the_metrics_agree_with_the_picture` test (2026-09-21) reported 89.3% agreement between a rendered tile and its own metrics and read as a real disagreement; the tile sat at `y0 = 8`, inside the 10-cell border band, so 424 pixels of parchment frame counted as mismatched content on both sides of the comparison | Place a metrics probe's sample tile outside the border margin, or exclude the margin explicitly, before trusting a "disagreement" between a render and its own metrics |
