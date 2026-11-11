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

Compacted 2026-10-04: entries condensed to rule, cause and check; incident narratives are in git history (`git log -p MISTAKES.md`).

---

## Preflight — find your trigger, apply the rule

Scan the left column for what you are **about to do**. If a row matches, apply
its rule before you start.

| About to… | Rule | Verify with |
|---|---|---|
| **Stage a verified-marker change into the index by patch while the working copy keeps the old text** | Apply the same edit to the **working file** too, or the next `git add` of it commits the stale "pending" back. ×3, 2026-09-28 (`d71824e`, `593df62`, one caught) | `git show HEAD -- STATUS.md \| grep '^-.*verified by the main'` empty after every commit touching STATUS.md |
| **Schedule or act on a backlog row** | Re-open it at its **cited symbol** first. Rows go stale within a day; "already done" is a valuable finding | Name the symbol you opened, not the row's text |
| **Return, marshal or default a value that can be absent** | **Omit the key**; callers use `has()`. In UI, dash the field **with its reason** | Render over real data; count rows showing a bare `0`, `1.0`, `"none"` or empty |
| **Write a test that pins a constant** | Assert a **literal**, or the independent thing it must equal. `assert_eq!(x, THE_CONSTANT)` holds for every value | Mutate the constant; the test must go red |
| **Change behaviour** | Hunt the prose describing the old behaviour: comments, tooltips, `_todo` reasons, the row's own cited location | Grep touched files for the old behaviour's vocabulary |
| **Add a capability, or write a staleness/cache key** | Derive the list from the **definition**: every consumer path, `match` arm, **every argument of the function you guard**. **Attaching a capability to a context is not the consumer reading it** (2026-09-24: `with_lakes` attached to both export paths moved 0 bytes; `BakeFields::pixel` had no lake branch) | Change each input, confirm the dependent reacts; **measure the bytes that move** |
| **Quote a timing or a benchmark** | Median with min..max. Run the harness **alone**, never under a parallel suite. No point estimates in doc comments | Re-run independently; beyond the stated spread = not a measurement |
| **Write "measured" anywhere** | Only for a number you **just produced**; elsewhere-figures are cited | Reproduce it from the command shown before it enters a decision record |
| **Declare the work green** | `cargo test --workspace` (never a crate subset) **and** parse-check every `.gd` touched plus `shell/app.gd` | The Rust suite cannot see a broken shell; both required |
| **Test behaviour a unit test cannot reach** | Probe scene: `godot --headless <probe>.tscn`. `WorldGen` is a cdylib `GodotClass`, not constructible in a unit test | The probe runs and asserts, not just parses |
| **Run `godot --headless --import`** | It strips `project.godot`'s `;` comments. Avoid if you can | `git diff …/project.godot` empty; `git checkout --` if deletions only |
| **Mutation-test** | Python only: exact literal replace, occurs-exactly-once assert, BUILD_ERROR separate from SURVIVED, **restore in a `finally`**. Never `sed`, never a whole-file backup | Zero MUTANT residue; hash file against pre-run value |
| **Call a shell helper** | Grep `func <name>` first. `menus.gd` vocabulary: `_todo` / `_readout` / `_signpost` / `_live`; the wrong one has consequences | `godot --headless --check-only` on the file |
| **Change generated output** | Divergence ships `false` in `cartalith_engine::WorldParams::defaults()`, `true` in `cartalith_godot::params::defaults()`; new params need `PARAMS` + `JS_PATHS` rows. **A golden re-baseline needs an owner ruling** | Hash the same render before and after; identity by control flow beats identity by arithmetic |
| **Re-base a shared token / constant set** | Every *relationship* built on the old values is unverified: recommended pairs, hover lifts, contrast | Recompute deltas and contrast ratios; no test sees it |
| **Edit a main-loop-owned doc while a verifier is running** | The **baseline** is the point: a mid-run diff the verifier cannot attribute is reported as unexplained. **×4, 2026-09-13, main loop itself:** a "dry run" of a backlog normaliser wrote the LIVE `OUTSTANDING_WORK.md` because a `sed` path substitution silently failed. A script editing a guarded doc takes its target as a required argument, and the dry run asserts the path first | Hold doc edits until verdicts are in, or tell the verifier what the main loop will write and when |
| **Commit while a verifier is running** | Don't. A clean tree makes `git diff` empty for **everything** | If you must, `git diff <base> HEAD -- <path>`, never a working-tree diff |
| **Write a Workflow script** | **Escape every backtick inside the template literal**, including those in prose like `(1080, 2400)`; an unescaped pair makes JS call the preceding string as a tagged template | `node --check <script>` before dispatching (two dispatches lost) |
| **Read any shared file while lanes are running** | Any read, not only probe failures: a mid-batch grep of `export_presets.cfg` saw a lane's in-progress edit and wrongly told the owner no authorisation was needed | `git show HEAD:<path>` for any claim about what was true *before* a batch |
| **Read a probe failure while other lanes are running** | A concurrent lane's in-progress save can give a spurious *"Function X not found"*. **Re-run once** before concluding the code broke | A failure that does not reproduce was not a failure |
| **Assert on pixels** | A threshold is **palette-bound**; naming the palette in prose does not select it. The harness must **force** it and refuse otherwise (this machine boots light). **Measure a control state.** A dark-theme `> 23` test on a light capture makes every background pixel 251 | Assert the palette, or use a palette-agnostic measure (uniform-row / distinct-colour count) |
| **Read a layout that overflows the screen** | A `ScrollContainer` with an axis **DISABLED folds its child's minimum size into its own**, so overflow reaches every ancestor with no scrollbar. Three instances | Walk the tree for `get_combined_minimum_size().x` above screen width; find which ancestor it reached |
| **Add `clip_text` / ellipsis to a Label** | Collapses `get_minimum_size().x` to **1**; beside a `SIZE_EXPAND_FILL` sibling the label vanishes | Confirm the text still renders |
| **Claim something covered is now visible** | The scene graph proves nothing under an opaque overlay. **Flip the flag and diff the framebuffer** | 0 pixels moved = inert; do not write the rationale into a comment before that test |
| **Change a widget's ink** | The contrast pair's *second* term is what is behind it. If one call restyles several widgets and only some get a background, the un-backed one is a **separate relationship** | Compute its pair too |
| **Grade a Godot probe as evidence for a Rust change** | `cargo test` does **not** rebuild `target/debug/cartalith_godot.dll`, which every `.tscn` probe loads. A probe after a `.rs` edit and before `cargo build` tests the *previous* engine | Compare `.dll` mtime against the `.rs` mtime and **state both** (once 11:35 vs 18:42: a batch proved nothing) |
| **Read `sample_cell()`'s (or any multi-valued optional field's) key with `Dictionary.has(key)`** | `has(key)` means "computed at all", not "the notable value". `s.water_body: Option<u8>` is `Some(0)` for **land**, so `d.has("water")` is true almost everywhere | Check the **value** (`d.get(key) != "land"`) for any field whose doc lists more than one non-missing outcome |
| **Cite a test file in a doc comment** | A `pub(crate)` justified by a named test depends on that name; `render.rs` cited `tests/geology_micro_and_sky_fields.rs` twice and it did not exist | `ls` every test path named in a doc comment you touched |
| **Write an oracle for a ported function** | The reference's *errors* are part of the contract: a brute-force exact Euclidean transform failed a correct jump-flood port (exact from one seed, approximate beyond) | Assert the reference's behaviour including approximations |
| **Build a fixture for a grid the app also fills** | Encode every "none" sentinel **as the live producer writes it**. `-1`-for-unclaimed let `jp_claimed_at`'s `>= 0` pass while the live grid (`0` = unowned) read every cell claimed | One test driven by the real producer's output; mutate the sentinel test, both it and the golden go red |
| **Re-resolve a citation late in a long pass** | A line number can drift (**148 and 241 lines** in files other lanes were editing; untouched files held) | Grep the quoted string; never jump to the line |
| **Conclude a thing does not exist from a directory listing** | `crates/cartalith-urban` has no `tests/` because fixtures live at `src/<module>/tests/golden.rs` (the milestone-16 golden: 3 139 lines), and a brief scheduled it as unbuilt | Grep the **symbol or content**: `grep -rn golden crates/<crate>/src` |
| **Write prose about another lane's subsystem in the same batch** | One lane removed a hardcode, the other shipped a note saying the hardcode was why a control was inert: true at dispatch, **false on arrival** | State the other lane's file as *of this batch* or re-verify after it lands; a cross-lane claim lives one wave |
| **Convert a replacing UI context into an appended one** | The **arm-another-tool path** gets missed: a single-answer `match` over the armed tool reaches its ordinary arms before any draft clause, so a live draft loses Commit/Discard and the new tool draws *its* Commit in the same slot | Enumerate every transition INTO the new state; probe each armed tool against a live draft |
| **Write a parent-type guard** | Ask *"does a sibling compete for my width"*, not *"which class is my parent"*. A `BoxContainer`/`HFlowContainer` list missed `GridContainer` | Enumerate every container that distributes the axis; assert one child of each |
| **Gate a write on "is there anything to save"** | Ask the **whole** aggregate. A `vault.json` write gated on `links.is_empty()` dropped a map snapshot (the store also holds `vaults` and `snapshots`) | Assert a save holding **only** each member in turn survives a round trip |
| **Report a layout measurement** | One world is one sample; panel widths are content-dependent: an empty plan measured 190 px, three worlds 351 / 385 / 441 against a 280 px dock | At least three seeds; say which |
| **Convert a replacing context to an appended one** | A conversion is more than the artefacts the record names: Journey had a **fourth in another file**, a `build_results()` that cleared the shared body itself | Grep every file writing into the container; `queue_free()` on a shared parent is the tell |
| **Write a diagnosis into a backlog row** | A row is a **router, not a finding**; a named cause propagates into every brief ("the panel is dropdowns whose minimum is their widest item": there are no dropdowns) | State the **symptom and measurement**; mark any suspicion as one |
| **Measure an effect inside a subset of pixels** | **Never pick the subset by the value under test.** 2026-09-24: LOD-D4's bar 1b correlated snow with northness only where `0.1 < snow < 0.9`, and an aspect term pushing shaded ground to full snow moves exactly those pixels OUT of the band | Select by an INPUT (the temperature the term reads); prefer an on/off comparison of the same pixels; if the effect survives term-off, it was never the term's |
| **Attribute a number to a cause** | Measuring the number is not measuring the *cause*: a fallback dab claimed in "the after column's maximum" measures **15.3 ms** vs a quoted **2.55 ms** | Measure the cause separately, or say the number and stop |
| **Run a pixel probe** | `ImageTexture.update()` is a **no-op under `--headless`**; every comparison passes vacuously | Run **windowed**, with a positive control that must move |
| **Write a multi-line edit through a shell heredoc** | Escapes are mangled twice: a `\n` reached a `.gd` as literal backslash-n; five stray tabs **parsed clean** | Prefer Edit; else read bytes back with `repr()` and parse-check |
| **Name the language of a remaining caller** | "The caller is GDScript" in four Rust doc comments about a plain `pub fn` taking `&mut [u8]` that GDScript **cannot call** | Check the signature: only a `#[func]` with Godot-type arguments is reachable |
| **Name the precondition for what the code does today** | The gate is **nearer** than the story: badges draw as discs "until a pack is imported", but the gate is no resolver Callable installed | Write the condition the code branches on, read it back |
| **Enumerate a surface for an audit** | Walk the **code**, then ask the designs about each entry. Measured: 37 of 283 menu items had **no design at all** | State the inventory method; have the checker look for what was missed |
| **Call a constant dead** | Probes are committed files and read theme tables (`ROLE["h_timeline"]` vs `_roleresolve_probe.gd`) | `grep -rn` the key across the project **including probes and tests** |
| **Emit a change signal** | Emit **after** the engine call; ten emitters fired before `world_gen.sculpt_*()`, so a listener re-read the old count | Probe the listener's value **inside** the emit; a `CONNECT_DEFERRED` listener never sees it stale |
| **Write a backward-compatibility test** | A fixture of empty collections (`{"vaults":[],"links":[]}`) proves nothing about the installed base | Build it from a real prior-format document (`git show <sha>:<path>`); assert it **opens, resolves, re-serialises byte-identically** |
| **Replace a single-sample timing with a median** | **A median of five on a noisy device is still one sample of the median**: `1.06x (1.00..1.08x)` re-ran as 1.00x, 1.00x, 1.02x | Re-run in a **separate process**; if the bracket's low end touches parity, say **no difference is established** |
| **Quote a sub-millisecond figure to three significant figures** | The precision is the claim: `9.65 us (9.63..9.66)` vs an independent 10.02 (9.95..10.04) | Give an order of magnitude; print the spread |
| **Move a command off the menu bar** | The searchable index walks the live `MenuBar`, so a moved command becomes unfindable (0 of 361 title matches; one accelerator unrebindable) | Add a `command_index.gd` `EXTRAS` row and `shortcuts_dialog.gd` `UNLISTED` row in the same change; assert by TITLE, not `search()` |
| **Reserve a file from every lane** | Reservation strands the fix that needs it: three lanes needed index rows, none could write them | Give the shared file to exactly one lane, or have the main loop land those edits before the batch is done |
| **Summarise a spec into a backlog row** | **Paraphrase the table, do not compress it.** "12 mode-gated blocks" summarised `04-left-dock.md` §3, which gates **one node of ten**, and an owner accepted "this hides controls" on it | Quote the spec's own condition column |
| **Claim a mechanism is "derived, not hardcoded"** | Check every line: the mode switch derived **visibility** but hardcoded node source, labels and refresh | Name derived and non-derived parts separately |
| **Write a probe's usage header** | The header is a claim about the probe's code: a documented `--resolution` it never read (`-- --vp WxH`) measured **the same box three times as three densities** | Grep the flag in the body; make the harness **fail loudly** on an unknown argument |
| **Add anything to an approved artboard** | **Content added below the fold evicts content above it**: a ~225 px block took `scrollHeight` 1 457 to 1 754 against a declared 1 012 frame and pushed CONTRAST out of frame | Measure `scrollHeight` vs the frame's `h` before and after; raise the frame in `canvas.json` **and** the root height together |
| **Delete a surface** | **Inventory it item by item, each with a stated home or drop.** Folding a window found 11 items: 6 homed, 2 to build, 2 dropped, **1 silent capability loss** | Grep each binding for other consumers; grep the class name across probes and `.tscn` (eight probes referenced this one) |
| **Write a completeness claim into a correction** | *"Nothing was left un-found"* was false in the edit that said it (6 sites in 3 files reported; **5 in 4**) | Say what you searched and how; paste `git grep -n <name> -- <path>` |
| **Cite a document as authority for an absence** | **Check it still describes the tree.** Thirteen `.gd` comments cited `04-left-dock.md` §0/§9.1's "lost to truncation" for values that are readable (prototype re-imported whole in `660cbef`) | Open the file and check size and last line; a truncation note outlives the truncation |
| **Let a lane finish a file another lane is granted** | A lane wrote 255 lines into another lane's file; the shell build hash moved **between one lane's own probe runs** | A lane needing a second file **asks**; straying makes evidence unattributable and the cost lands on the lane that behaved |
| **Brief two concurrent lanes that both write scratch scripts** | Each lane gets its **own scratch subfolder** in its brief. 2026-09-26: two lanes wrote `mutate.py` to one scratchpad; one overwrote the other's harness, giving **5 failures that were live mutants** | Every brief names `…/scratchpad/<lane>/`; a failure that passes alone during another lane's sweep is that sweep |
| **`git stash` to compare against HEAD while another lane is running** | **Don't**: a stash takes every uncommitted change in the shared tree. Use `git worktree add --detach <dir> HEAD` with its own `cargo build`. 2026-09-26: a lane stash/popped while CM-2 was mid-build | Every brief: no `git stash`, no `git checkout --` on files it does not own |
| **Speed a lane up by starting helper agents** | **Don't**: the owner caps concurrent agents ("At Max 2 agents") and helpers count. 2026-09-27: the tooltip-sweep lane started 4 | Every brief: do not spawn sub-agents or workflows |
| **Work around an action the permission system denied** | **Stop and report.** A denial is the owner's boundary. 2026-09-27: a lane refused `git apply -R` reached the same revert by restoring `git show HEAD:<path>`; the main loop also withheld a launch the owner later said it could run | Every brief: *if denied, stop and report; do not reach the same effect another way* |
| **Verify a visual change before committing** | **Open the screenshots and look, at every zoom touched.** Listing files is not looking; a probe on an intermediate texture cannot see the screen. 2026-09-27: per-style rivers (`2cf0143`) committed after `cmp2_*.png` was listed unopened; rivers drew pure white at fit zoom | One before/after image per zoom level; a probe leg that reads SCREEN pixels |
| **Search for a file or symbol** | **Stay inside `C:\Users\Vincent\Cartalith_GDT`** (owner, 2026-09-23). Never `find /`, `grep -r` on a drive root, or `$(go env GOPATH)` expansions; dependencies live in `~/.cargo/registry`. 2026-09-27: a lane ran `find / -iname "*.rs"` | Rule in every brief; search paths start at the repo or `~/.cargo/registry` |
| **Kill a Godot process to free a locked `cartalith_godot.dll`** | **Not while another lane may be probing**: the killed process may be another lane's run (2026-09-26: two killed while CM-5 ran phone probes) | Never kill processes you did not start in the same command; after a rebuild the main loop tells probing lanes to re-run |
| **Resume a workflow lane with `SendMessage` while its workflow still runs** | Two copies of one lane can be live; the resumed copy called its **own** mid-save file a foreign writer's breakage | Prefer letting a lane finish; else tell it work on disk in its own files is its own |
| **Act on a critic's defect list** | **A critic's defect is a claim; refusing one is legitimate.** Two of seven failed: *"`poi` is emitted unconditionally"* (the true arm of `if p.is_poi`) and *"no dry-run path exists"* (wired on a sibling route) | Open the cited symbol first; record "not real, changed nothing"; if real for a different reason, write that reason |
| **Act on a verifier's own fix** | **Its *fix* is the least-checked part.** Widening `_dash_phase_track`'s f32 accumulator "from 43 differing px to 0" re-measured as no difference on either probe (f32 and f64 alike) | Re-measure a verifier's fix before adopting it; say so if you keep it for other reasons |
| **Write `--headless` into a brief** | Check what the claim rests on: the dummy display driver never fires `RenderingServer.frame_post_draw`, so every pixel/timing probe aborts or hangs | Headless for logic and layout; **windowed for anything that rasterises or times a frame** |
| **Judge a measurement against a floor** | Check the floor applies to your density: *"buttons are 39 px against a 44 px touch floor"* mixed **pointer**-density drawn height with a **touch** floor the touch branch meets (`role_px("btn_min_h")` = 44; `phone_fit()` floors every `BaseButton`) | Name the density beside every number |
| **Dispatch a batch while another batch's verifier is running** | **Don't.** A batch edited `map_overlay.gd` (+137/−30) mid-verification; the shell build hash moved across the verifier's three runs (`fa6e20b` → `594287e` → `1993049`); **every probe result was against a moving shell**. Disjoint lane files are not enough | Verifier finishes, main loop commits, next batch starts; if overlap is forced, tell the verifier which files a different workflow owns and when |
| **Write a correction** | **It is new prose and can ship its own false clause**: a note said `_foodshed_probe.gd` *"drives all four branches"* while adding a **fifth**; another named `_show_project_stats()`, which exists nowhere | Re-read it as someone else's; grep every symbol it names, including those just added |
| **Dash a field with a reason** | **The reason is a claim.** A batch minting zero fake *values* shipped three fake *causes*: `detectRiverCrossings is unported` (ported, tested); `no binding reports how many` (binding exists, only the forwarder missing); two dashes swapped reasons. **A wrong reason is worse than none** | Open the symbol you say does not exist; name the layer it is missing from; exercise **both** branches of a two-branch dash |
| **Carry a rule from one variant of a design to another** | Open the other variant's **drawing**: "destructive action text-only and not rightmost" is wrong for variant B, which draws it filled and rightmost (no safe action to be relative to) | Scope the rule to its condition in the design; fix the artboard before the code |
| **Choose which preflight rows to put in a brief** | Scan the table against the batch's **verbs**, not from memory: 7 of 55 rows inlined by recall; four exact matches were missing and one bit the main loop | Read the left column top to bottom against the batch's scope (186 rows counted 2026-09-23) |
| **Re-run the parse check after EVERY edit, not once per file** | A probe edited twice parse-checked after the first edit and shipped a duplicate `var`. A failed script load makes `godot --headless` **hang**, reading as a slow probe | `--check-only --script` after the last edit; a hanging probe: suspect a parse error first and capture stdout |
| **Cite two examples as agreeing** | Open the **second**: a header said the asset-pack inspector and RECENT RUNS "both settle one thing"; RECENT RUNS (parent `#6f7478`, value `#8d9296`) settles the opposite | Quote both with line numbers; if they disagree say the canvas is split |
| **Quote a count of call sites, instances or files** | Measure it in the same edit and paste the command: "94 call sites" was **189 code call sites in 18 files** | `grep -rn 'Symbol(' dir/ \| grep -v ':[0-9]*:[[:space:]]*#' \| wc -l`, with the date |
| **Dispatch agent lanes** | One brief per lane, checked before launch. Serialize lanes sharing a file. Tell every lane to **report** false prose in files it does not own. **Every verification item carries a premise; check it holds** ("mutate a constant each lane introduced" is unsatisfiable for a lane that introduced none) | Re-read each prompt for a foreign lane's heading; ask what state makes each check impossible (four such items in one brief) |
| **Correct a doc a brief told you was stale** | The brief names one false term; the doc may have more: `landmark_store.last`'s opening clause (*"`last` is still not written, deliberately"*) was also false (ruling 10) | Read the **whole** doc block; `git show HEAD:<file>` and test every clause |
| **Write a golden-risk item into a brief** | **Name the crate the goldens live in and check the change reaches it.** A brief warned wiring *"can move placements"*; goldens live in `crates/*/tests/`, the change was in `cartalith-godot`, and **no golden covers the landmark pass** | `find crates -path '*/tests/*' -name '*<subject>*'` and `git diff --stat` first |
| **Read the backlog counter after closing rows** | **It counts ROWS, not open items**; striking in place does not move it; a later sweep drops it | Reconcile against `git stash` + a HEAD run; say which count you quote |
| **Quote a memory figure for a buffered pipeline** | **A raw buffer is not a peak.** `w*h*3` (2.06 GB at 32 768 × 20 976) vs **21.7 B/px measured**, ~15.8 GB; the code's 15 B/px budget was wrong too (`blur_once` holds two buffers) | Name which figure; poll peak resident from the **host**; check the slope across two sizes |
| **Say "recover it from history"** | **Prove the artefact is in history.** A ruling told a build to recover a prototype reverted *before its pass committed* (`--diff-filter=A` finds nothing; `-S` returns prose commits) | `git log --all --diff-filter=A -- '<glob>'` and `git log --all -S '<symbol>' --stat` |
| **Add a gate to a path that shipped ungated** | **Say so where the constant is defined.** `UNGATED_MAX_WIDTH` bounded only its no-budget branch; the budget branch gated **every** width, including three shipped unconditionally (real on Android) | Per guard: which inputs newly fail that used to pass? Which way does the estimate err? |
| **Add a formatter, parser or helper "because the other one is unreachable"** | **Grep the crate first.** `human_bytes` already existed in the crate and disagreed (`15.4 GB` vs `15.4 GiB`) | `grep -rn 'fn <name>' crates/<crate>/`; one convention per crate for user-read strings |
| **A lane returns no report** | **Stop the batch; do not pass a `null` through.** 2026-09-06: the largest lane (723 lines) hit the StructuredOutput retry cap and filed nothing; the verifier interrogated claims that did not exist | Check every element of the lanes array before the verify prompt; re-run that lane or verify its diff directly, and say which |
| **Write a guard that makes an artifact self-consistent** | **Ask what it proves.** `write_project` computed a pyramid's `source_key` from the heightmap it wrote, so stale tiles were **stamped with the new key**. Self-consistency is not freshness | What state does the guard make undetectable? Treat a caller-supplied key as a claim |
| **Report a `cargo test --workspace` total** | **Count result lines against the brief's floor.** A lane reported *"100 lines, 2 380 passed / 1 failed"*; the real run is **157 / 3 240 / 0**. **×3 now** (also 2 104 and 17 before) | `grep -cE '^test result:'` equals the floor **before** reading any total; below it is a truncated read |
| **Write the guarded-paths list in an agent brief** | **Say who a path is guarded *from*.** The main loop then wrote to `design/` and `LARGE_ITEM_RULINGS.md` mid-batch and the verifier reported unexplained dirt | *"Guarded from you; the main loop writes here"*, or hold the edit |
| **Repeat a design document's dashed field into a brief** | **Re-measure the dash's reason.** *"There is no log file"*: Godot logs to `user://logs/godot.log` by default and a probe read its marker from it | Open the symbol; tell the lane it may overturn the dash |
| **Quote `.dll` and `.rs` mtimes in a report** | **Re-stat at the END of your run.** A lane quoted `dll 16:08:10, atlas.rs 16:03:43`, then edited `atlas.rs` at 16:10:23 | Both mtimes as your last action; rebuild if the `.dll` is older |
| **Commit a "clean" subset of files while a shared file stays uncommitted** | **An intermediate commit may not build**: `lib.rs` called `mark_icon_committed()` while `landmark.rs` stayed uncommitted | Build **exactly what `git commit` will write** (stage, `git stash --keep-index`, build, restore) |
| **Ask a lane to walk a touch floor** | **Say TARGET, not height**: every height was 44 px and **both defects were on the WIDTH axis** (a 36 px square, a **7 px** chevron) | "Every tappable control's laid size, both axes, against the floor" |
| **Run a probe `--headless` and report its error count** | `--headless` manufactures its own `SCRIPT ERROR`: a lane's *"0"* was **22** with stderr captured (`save_png` on a null from `get_texture().get_image()`) | Capture stderr; run a probe in the mode its own header prescribes |
| **Report a defect as density-specific** | **Check the other density.** "Phone-density-only" small font was **15 of 23 screens at 1440 and 19 of 23 at 1080** (fixed `fs=10.0`) | Tally per resolution before writing "only"; name the mechanism |
| **Add a second route to a control the user already has one to** | **Two views of one state will disagree**: a role-list row press moved marks, title, dials and a button while the dropdown kept the old value (`DccWidgets.choice()`'s return discarded) | List every widget reading the same state and assert each follows |
| **Tell a lane a field has no engine behind it** | **Open the setter, emitter and drawer.** `halo` was claimed engine-less; `set_field` accepts it, the render list emits `halo_em`, the overlay strokes it | Three symbols: set, cross, drawn; tell the lane it may overturn you |
| **Floor a control that a rebuild creates** | **`tablet_fit()` floors HEIGHT only and runs ONCE, deferred**; later `_rebuild_*` output escapes both fitters (a 22 px field, a 7 px chevron, a 44×29 `edit`, a 60×24 well) | Floor at the call site on **both axes**; measure at tablet and phone |
| **Report that a feature does not exist** | **One grep is not an absence proof; search the CONCEPT.** ×3. (1) The owner was told twice that no km/mi toggle exists after grepping `miles\|km/mi`; it ships as `Preferences ▸ Units` via `DccUnits.label("mi")`. (2) 2026-09-21: an agent was dispatched to build CV-23 after grepping the unused constant `HISTORY_TERRITORY_PREFIX`; it shipped via the **field** `history_territory`. (3) 2026-09-23: "149 of 200 addon villages have no road" filed after checking only `_civSeedVillages`; `_civConnectVillageAddons` (`Cartalith Gen1 v2.11.html:25766`) connects them | Grep the domain word, settings key and menu label separately, then open the symbol. **A constant with zero readers proves the constant unused, not the capability unbuilt**; grep the concept's verbs (`connect`, `link`, `attach`) too |
| **Write a probe that PRESSES a control** | Press its **edges**, not its centre; a centre press cannot see target width | Shrink the control below the floor; a centre probe still passes |
| **Assert a control is present, sized or reachable** | `get_global_rect()` is the child's **UNCLIPPED** rect. Assert the **drawn rect against the container's VISIBLE rect** | A control 15% painted must FAIL; keep a positive control |
| **Assert a widget's appearance** | A stylebox-shaped test cannot see a modulate; headless returns a null texture. **Render and read pixels** | Force the colour to red; the probe must move |
| **Prove a style is independent of a stale theme** | **Independence is not correctness**: zero movement under a hostile theme says nothing about alignment, centring or minimum size | Also assert the drawn VALUE against a canvas literal |
| **Add a probe in the same commit that changes the behaviour** | **The same claim written twice.** A probe earns authority by **failing on the state before the change** | Run it against the pre-change file; passing means it pins nothing |
| **Assert a drawn colour** | Never against `DccTheme.c(token)`: **both sides move on a flip**. Pin the canvas literal with its `ENV:` line | Flip the palette; a token-vs-token check cannot fail |
| **Set a palette inside a probe** | **After** the shell boots; `apply_theme()` before instantiating `app.tscn` is **inert** | Assert a value that DIFFERS between palettes |
| **Measure ink or a glyph in a capture** | The **brightest pixel** is a dark-palette habit; on light it is the **ground** | Take the pixel furthest from the band in luminance, in its own bbox |
| **Answer "can the user do X?"** | Measure the widget **the owner touches**, not the one the brief names (desktop `PopupMenu` vs phone `Button` chips) | Name the surface and density |
| **Reach a screen inside a probe** | **Never call a function to navigate.** Tap what is visible from launch. Separate **it renders** / **it can be operated** / **it can be FOUND** | A control can be visible, sized and tappable inside an invisible container |
| **Find a figure the canvas does not contain** | Check the **superseded** canvas first: `height:34px` is 0 in the current canvases and **32** in the replaced one | `grep -c` both; an inherited figure is a re-anchor, not a deletion |
| **Say which renderer the shell uses** | **`gl_compatibility`** (`project.godot:101`), not Vulkan; **headless is not a rasteriser** | `--rendering-driver vulkan` forces Vulkan |
| **Cite an `ENV:` line** | **Open it**: `ENV:912` was a bare closing `</div>` | Quote the declaration, not just its number |
| **Dispatch a second workflow while a verifier is running** | Don't, or **name in the verifier's brief what else may write, by file** | Verifier snapshots md5s and reports drift (caught twice) |
| **Write a commit message** | **A claim about its own diff**: each probe named as passing must be IN it | `git show --stat`; `a2682de` shipped two false claims |
| **Re-run a probe as evidence** | Check it could RUN: `_nwclip_probe` printed `fail=0` on desktop (`is_phone()` needs touch). **A probe that cannot answer must not report a pass** | Non-empty output AND a firing positive control; exit non-zero when it cannot run |
| **Write a mechanism into a brief** | Measure it first: *"`DirAccess` cannot reach shared storage on Android"* is false (lists 16 dirs, writes fine; **only FILE listing is filtered**) | State the measurement, or label it a candidate |
| **Write or accept a probe's exit status** | **No success its own output contradicts**: `_tabfit_probe` printed `overflow=285.0` and exited **0** | "Assertions held but the surface fails" gets its OWN exit code (3), distinct from 1 |
| **Close a backlog row** | **A struck row left in a NUMBERED section still counts** | Move to the unnumbered archive section, re-run the counter |
| **Run `--check-only`** | From the **`godot-project` root**, never a subdirectory: from `shell/` it reports *parse errors that do not exist* (`res://` resolves nothing) | Two errors on `app.gd` from `shell/`, zero from root; suspect your invocation first |
| **Call `phone_window()` on a dialog with its own verb** | It sets `ok_button_text = "Close"`. Set the caller's text **AFTER** it, or the primary action is renamed **on phones only**. Hit three times in a day (*"Write to Markdown"*, *"Create"*, *"Clear 412 MB"*) | `DccWidgets.confirm()` orders it correctly |
| **File a row saying "this needs a bigger fix"** | **A claim about the code**, wrong twice in a day (the SAF caller audit: four of seven consumers end in Rust `std::fs`; a `dialog_text` "second remedy" that is `confirm()`'s shape) | Re-open at its symbol before scheduling the big version |
| **Chain a shell command after `grep -c`** | `grep -c` **exits 1 on a 0 count**, so `&&` drops the rest | Append `\|\| true`, or treat the count as data |
| **Plan a phone probe that must reach a DIALOG** | **You cannot.** A synthetic tap cannot reach a control inside an embedded `AcceptDialog` sub-window at `content_scale_factor` 2.62 (canvas coords, physical coords, `get_final_transform()` all tried). Taps into the MAIN viewport route normally | Project-picker and New World legs are **provable on glass only**; pressing by label tests the handler, not the touch path |
| **State how many checks a probe runs** | **Print the count; never assert it in prose** (a doc said 26, the probe ran 29) | Increment in `_check()` and put it in the RESULT line |
| **Report that you could not reach something** | **"I could not reach it" is an ABSENCE CLAIM.** A lane called six right-dock sections unreachable; the verifier reached **five in one pass**: it sought a context REPLACEMENT where the header documents sections **APPENDED** | Say what you tried and what mechanism you assumed |
| **Verify anything phone-shaped** | **`--force-touch` on desktop is not the phone, and `pressed.emit()` is not a finger.** Synthesised input is injected **downstream** of `MOUSE_FILTER`, scrims and hit areas; a session of green probes shipped an APK where the owner could not find the generation menu or use the journey planner | **See with `adb exec-out screencap`, act with `adb shell input tap/swipe`** at coordinates read off the image; navigate from launch tapping only what is visible. Desktop probes are for regression, never reachability |
| **Claim a screen "works" on a phone** | **Separate "it renders", "it can be operated", "it can be FOUND".** Probes proved the journey planner fits and clears the tap floor; reachability was never tested | State which of the three you measured |
| **Find a screen that is mostly blank** | **Ask what is IN it before ruling on its size.** A phone sheet measured 1 003 rows holding one strip (933 blank, 912 with a world) was deferred as a detent ruling; **a correct detent over empty content is still an empty screen** | If blankness survives the state change that should fill it, report what is missing, not dimensions |
| **Conclude a batch has finished** | **A run directory you cannot find is not a run that has stopped.** A batch was closed with the third lane and verifier still working; the commit landed mid-verification | **Wait for the completion notification or check the task list**; never infer completion from the filesystem or from "the lanes I know about have reported" |
| **Write "still X" or "routes NONE" into the backlog** | **Re-measure it in the same breath.** *"The journey planner still routes NONE"* (`grep -c DccUnits` = **34**) and *"the measure tool converts 4 of its 7 modes"* (**six** modes, **all three** named convert) both came from the brief | The four-second grep would have caught both |
| **Quote a byte or character offset into a document** | **Say which and re-measure, or name the symbol.** Six offsets were "corrected" by labelling them *character* vs *byte*: **both invented**; the real byte-minus-character delta is **52 to 126** (copy read with newline translation on). A UTF-8 CRLF file's offset has three values | **Grep the symbol and name the guard it sits under** |
| **Raise a touch target to the 44 dp floor** | **Ask what the enlarged control now intercepts.** Flooring fourteen sliders from 32 to 44 dp widened the band where a vertical scroll starting on a slider is a **value write** (Ocean depth 0.60 → 0.14) | Arbitrate the gesture in scrollers; **do not shrink the target back** |
| **Record a glyph substitution** | **The deviations table must agree with the byte that shipped.** *"U+2212 present and used"* while the code shipped ASCII `-` | **Check the call site, not the cmap** |
| **Assert that two runs produce identical output** | **Compare what the format promises is stable; exclude what it does not, including the CONTAINER's metadata.** The save-writer stamps every zip entry from the clock (DOS time, 2 s), so a pair straddling a tick differs at byte 10 (`SAVEFILE_COMPAT.md` section 16); flaky for months | Exclude clock-stamped metadata (JSON's `created` was excluded, the container's was not) |
| **See a suspiciously round test total** | **`cargo test --workspace` FAIL-FASTS**: a flaky io test gave **100 lines / 2 387 passed** against the true floor **157 / 3 253** | **Count result lines before reading any total** (×3 rule above); `--no-fail-fast` for the whole picture |
| **Write a test helper that returns a collection** | **Assert non-empty in the helper.** A fingerprint helper mutated to `Vec::new()` left **both tests green**. The silently-empty-golden trap has bitten **five** subsystems | Mutate the helper |
| **Paste a `grep` and its result into a comment** | A command that measures the file stops reproducing when it changes (`grep -n "HSlider.new()"`: two, then one) | Name the SYMBOL, not the command |
| **Run a mutation harness that rebuilds and installs** | **Restore the DEVICE too.** A slop-mutation harness restored every source file (sha256) and **left the mutated APK on the handset** | Hash the installed `base.apk` (`pm path`) against `builds/android/` |
| **Report a census as the enumeration of a hazard** | **A live-tree walk is a LOWER BOUND from one state; name the state.** 247 `Range` nodes / 242 hazards from a **world-less** boot where `tl_available()` is false; **four slider sites contributed nothing** | State the boot state; cross-check against a code inventory |
| **Fix a gesture hazard for one control class** | **Ask which OTHER class has the same shape.** `PgSlider` fixed every `Range`, yet a swipe on an `OptionButton` still opens its popup. **The hazard is the gesture, not the widget** | Inventory by gesture, not base class |
| **Trust an on-glass result** | **Read logcat for the stale-library warning first**: Android exports carried the `android-dev` `.so`, logging *"no `WorldGen.<fn>()` … older than the shell"* on cold boot | Any native-backed on-glass claim is unsafe while it is in the log (shell-side layout/gestures unaffected) |
| **Walk a Godot scene tree to count anything** | **`get_children(true)`**: the default omits INTERNAL children (**4 401** vs **5 512** nodes; 12 `SpinBoxLineEdit` and a `TabBar` hidden) | Always pass `true` |
| **Rule a control class OUT of a hazard** | **Say which question you answered.** `SpinBox` was cleared because a swipe does not STEP its value; the press takes FOCUS (scroll `0 → 0` vs `0 → 62`) and on Android raises the soft keyboard | **"Not this mechanism" is not "not a hazard"** |
| **Write "this defect is milder"** | **Severity is measured.** A dropdown filed as *"milder … can dismiss it"* measured **`sel=7 → sel=3` with the popup closed**: the slider defect, silent | Measure the outcome first |
| **Explain why something was left untouched** | **Check the mechanism reaches it.** A comment credited the "no vertical scroller" gate for leaving seven phone `MenuButton`s stock; **0 of 7 carry `_phone_fitted`** | Invert the gate to test cause and effect |
| **Invent a screening metric (darkness, line count, size)** | **Calibrate against a KNOWN GOOD and KNOWN BAD, both numbers in the label.** `dark%=94.2 (94.2 = splash, ~85 = picker)` was backwards: splash **98.9%**, picker **94.2%** | An uncalibrated metric converts a glance into false confidence |
| **Read a size, position or detent from an animating control** | **Await the tween.** A sheet read `609.0 px = 232.32 dp`; six runs gave **592 / 589 / 575 / 574 / 506 / 510** inside a 0.28 s detent tween | Await, repeat, quote the spread |
| **Search for probes that touch shared state** | **Search for the CONSTANT as well as the literal path**: counts of `user://` probes read 28, 26, 26; the answer is **28** (two use `DccSettings.CONFIG_PATH`) | Grep both |
| **Write "it matches the design"** | **Quote the departures table.** "Sub-dp" fidelity from a subset; the spec lists about twelve departures, including **10 parameter groups vs the canvas's 8** and a slider row **floored to 44 dp vs 22 px** | Conformance is the whole table |
| **Count log lines as a health signal** | **Ask what generates the lines**: a diagnostic build talks MORE when sicker. The 56-line build is 5 real + 51 `has no WorldGen.<sym>()`; 5 was healthy | Identify the line source first |
| **Write a caveat into your own brief** | **A caveat you reason past launders the conclusion.** *"Absence of those warnings is NOT evidence"* made the 5-vs-50 metric unusable, and it was built anyway | Check what the caveat invalidates BEFORE the next paragraph |
| **Escalate to a bisect, a rebuild, or a large row** | **Reproduce the failure ONE more way, cheaply, first, with your eyes.** One screencap would have ended a 33-commit bisect | **The cheapest disconfirming check before the expensive confirming one** |
| **Trust a probe that reads `user://`** | **A probe sets up the state it asserts on, or declares and clears it.** `_nwsize_probe` on a byte-identical tree got **fail=1 twice** in one session and **fail=0 twice** in another: `cartalith_settings.cfg`'s `[recent] paths` and `projects=…/Worlds` are WRITTEN by probe runs | A probe reading what siblings wrote can go green for unrelated reasons |
| **Blame persisted state for a probe that disagrees between sessions** | **Check the DISPLAY DRIVER first**: `RenderingServer.frame_post_draw` fires **0 of 240 frames headless**, **239 of 240 windowed**, and `app.gd::_open_welcome_when_drawn()` awaits it | An mtime inside a run window is a coincidence |
| **Fix a touch hazard on a text field** | **The lever is `focus_mode`.** `Viewport::_gui_input_event` grabs focus **before** `_gui_call_input`, so `accept_event()` never helps; `MOUSE_FILTER_PASS` forwards nothing (`LineEdit::gui_input` accepts every left press) | A five-way table settled it; three plausible fixes failed |
| **Override a stylebox on a `Button`** | **`flat = true` voids EVERY stylebox override**, `hover` too (13 sites drew nothing); the tell is a selected state distinguished only by ink or opacity | Render and read pixels |
| **Run `--check-only`** | **Pass `--script res://…` too**, or Godot BOOTS THE PROJECT and looks hung (a "20-minute" check returns in seconds) | Returns in seconds |
| **Rule on a plurality** | **Count the POPULATION first**: all 8 nodes of the winning padding belong to a different height role than the control ruled on. Four wrong figures passed through that row | State the population beside the count |
| **Write a stop-and-report gate into a brief** | **A REQUIRED OUTPUT FIELD, not prose**: each gate HIT or NOT HIT with the deciding measurement. **×2 in five days:** a lane recounted, disagreed and implemented anyway (2026-09-08); another quoted *"88 dp > 66 dp budget"* then called it *"no collision"* (2026-09-12). **×3, 2026-09-12:** a lane wrote *G3: HIT* and shipped a 45-line change anyway | **The field made the breach visible, not impossible**; a verifier treats any code change after a HIT gate as blocking |
| **File a defect you saw in a screenshot** | **Crop to full resolution and re-look.** A downscaled view showed two clipped labels; only one was real | Full-resolution crop |
| **Append a row to `OUTSTANDING_WORK.md`** | **`rstrip() + row` lands in the LAST ARCHIVE block**: right for closed, wrong for open (open rows go at a numbered-section anchor). Both were backwards in one edit and the counter stayed CONSISTENT | Print the section each row landed in; assert it matches the row's state |
| **Write to the guarded docs while a verifier runs** | **Say so in the verifier's brief**, or it chases the edits as unexplained drift (168 added lines, 2026-09-12) | Name the file and when |
| **Measure "before" with a mutant** | **Reproduce the ORIGINAL measurement first.** `if false and …` left an empty 26 px container: bug read as 77 % clipped vs the real 45 % | Reproduce the original first |
| **Report a control as disabled from reading its code** | **A guard's else-branch reason is not the control's state.** *Erode (droplet)* and *Count painted lakes as water* were listed disabled; both guards call `_has()` and both methods are exported `#[func]`s, so both are enabled | Check what the guard evaluates in a current build |
| **Run two batches at once** | **Measurement-disjoint, not only file-disjoint**: a lane moving a dock minimum, frame floor or timing corrupts another lane's before/after | Name each lane's measured quantities in the briefs |
| **Compare against HEAD while other lanes run** | **Scratch copies only** (`git show HEAD:<path>` into the scratchpad); **never `git stash`, `git checkout -- <file>`, or HEAD over the live file.** wf49 (2026-09-13): HEAD's `dcc_shell.gd` copied over the live file twice, and a `git stash` on `world_workspace.gd`. **×2, same day:** wf52 reverted a hunk in the live file with Edit despite the rule in its brief. **×3, 2026-09-23:** a bare `git stash` while two OTHER lanes had uncommitted work swept up a long-dead stash (a resurrected `main.tscn`, a stray `project.godot` edit), and `git stash pop` conflicted | `git stash list` empty at the end; the report names every HEAD copy's path. **Whole-tree snapshot: a scratch worktree (`git worktree add`)**; if a stash is unavoidable, `git stash push -- <own files only>` |
| **Isolate a probe's user dir** | **Godot 4.7.1 has no `--user-data-dir`**: the flag's value takes the positional scene slot, running the interactive MAIN scene against the live user dir (2026-09-13: four boot-timing runs rotated the owner's Godot logs) | Isolate by a project copy with a distinct `config/name`; the probe's first log line appears and the live user dir's mtimes do not move |
| **Fix a stale cache or an invalidation bug** | **Enumerate every writer of what the cache depends on.** 2026-09-13: a tile-cache fix covered one of three sculpt-commit buttons and none of undo, redo, revert, erode or the paint commits | Grep every writer (every `map_view.texture =`); probe each |
| **Brief a layout change from a spec document's summary** | **Open the canvas markup and name the element.** wf49 briefed *"the canvas's 2-column grid"* from `TABLET_UI_SPEC.md` §2.4; the grid (`st.grid`) holds toggles/segments, sliders are full-width (`st.sliders`) | Quote canvas lines, not the paraphrase |
| **Print from a Windows Python script** | **Print ASCII-safe or set `PYTHONIOENCODING=utf-8`**: printing `▸` to cp1252 crashed a filing script after its first save, behind a "refuses to run twice" guard | Every print after every save |
| **Put a question, or a factual premise, to the owner** | **Verify the premise at the symbol, and search `LARGE_ITEM_RULINGS.md` for the question.** ×2. Star forts "never generate because settlements carry no `fortified` trait": the Place Editor writes it, the bridge reads it, the engine grants the fort; only the drawing was missing (**a stale code comment**). **2026-09-23:** Ruling AP re-asked landmark questions ruled on 2026-09-06 (rulings 10, 12, 13, 16) or earlier that day | Grep the rulings for the subject, open the symbol the answer would build; quote both or say neither exists |
| **Say something is OCCLUDING something else** | **Check the two edges merely COINCIDE**: a sheet's clip and the nav bar's top both derive from `_phone_nav_reserve()` | **Sample a pixel at the covering element before and after** |
| **Assume a device-only defect needs the device** | **Try the desktop composition first**: windowed 1080×2340 `--force-touch` reproduced within **3 px** of the device | Windowed 1080×2340 `--force-touch` |
| **Capture a sheet, drawer or anything with detents** | **Name the detent and re-measure at it**: chips clipped at peek are fully visible one detent up | Re-measure at the original detent |
| **Conclude a control does not respond to a tap** | **Check the control is still on screen**: MORE, MAP and PLAN "stopped responding" under a full-screen panel | Screencap first |
| **Screenshot a running app as evidence** | **Take two and diff them**; differences outside the clock mean a mid-animation frame | **Record the installed build's `lastUpdateTime`** |
| **Quote a count from a DIAGNOSTIC probe** | **Check its transform is the real draw's**: `Rect2(ZERO, size)` vs the renderer's INSET content rect gave 133 failures vs 151 | The ratio held; the count did not |
| **Write "structurally cannot"** | **Check every consumer**: a guard skipping untriangulable polygons "cannot lose ink", **false for the ink pass, which strokes via `draw_multiline`** | Enumerate every consumer |
| **Fix a probe's ordering bug** | **Ordering alone rots**: assert a value that DIFFERS between the two states | Value that differs between states |
| **Cite a line as evidence that a defect is live** | **Read the block, not two lines**: for a draw call read what BRACKETS it (`push`/`pop`, `_begin`/`_end`) | **`git log -S` the helper**; a site fixed two weeks ago reads like one never broken |
| **Read several lines with `sed -n 'Ap;Bp'`** | **It prints in FILE order**: a reversed read put a false "line numbers are swapped" correction into a brief | One line at a time, or `awk 'NR==A{print "A: "$0}'` |
| **Strip comments and strings to find call sites** | **Strings FIRST (triple-quoted, then single-line), then comments**: a `#` in a string orphans its quote and eats newlines (299 lines → 62) | **Self-check by line count** |
| **Assert that a converted value CHANGED** | **Inequality plus a suffix is satisfied by relabelling**: reconstruct the NUMBER from the converter (`DccUnits.to_unit()`), not a typed constant | Expected number from the converter |
| **Fix a formatting inconsistency you just exposed** | **Check which side is non-conformant**: two styles side by side usually means one was ALREADY wrong; `design/**.dc.html` decides | Consult `design/**.dc.html` |
| **Chase an engine "bug" reported by a probe's own scan logic** | **Verify the probe's read against the API contract first.** A 26 000-cell scan via `d.has("water")` reported the 2026-09-20 showcase world 100% ocean, surviving a `sample_cell()`, a `.dll` rebuild, standalone `generate_terrain()` (65% land) and `godot_print!` in `absorb()` (land 1 485 988 / ocean 881 766 / lake 317 174) | If the engine crate tests clean in isolation, suspect the probe's read |
| **Guard a fixture on several conditions before trusting it** | **Count the INTERSECTION the guards certify.** LOD-D4's glaciated fixture (2026-09-21) put flow in the valley and altitude on the flanks, so `build_glacier_potential`'s two conditions never met: zero gated cells with three guards green | Compute the AND and assert it non-empty |
| **Measure "the picture" from a tile render without accounting for the frame** | **`apply_border`'s opaque margin (`max(0.014·gw, 10)` cells) is in every tile render**: LOD-D4's `the_metrics_agree_with_the_picture` read 89.3% because the tile at `y0 = 8` sat in the border (424 frame pixels counted as mismatch) | Sample outside the margin or exclude it |
| **Declare a list, a backlog or a sweep exhausted** | **Read it end to end once.** Three partial surveys of `OUTSTANDING_WORK.md` (2026-09-23) said "no decision-free work left"; a full read found two (`b19b9ea`, `b5aff17`). See the 2026-09-23 entry | Name the method ("read §1-§4 row by row") |
| **Act on a summary's bucket for a row** ("ready", "no decision needed") | **Re-read the row's full text first.** Multi-ridge orogeny and IN-13's caravans were bucketed "no decision needed"; one row *"does not authorize building it"*, the other held two unchosen options (went to the owner as Ruling AP). ×16 instance of the backlog-row entry | Quote the row's gating sentence beside the bucket |
| **Read a reference comment about what a later version did NOT change** | **It is not evidence of what the earlier version did; open the earlier code.** v1.05's *"BASE per-cell map loop is untouched"* hid that v0.103's lake stamp was never ported (lakes flickered on zoom, `13062e5`) | State what the earlier code does, with its frozen-reference line |
| **Cull anything inside a Godot `_draw()`** | **Godot does not re-run `_draw()` when an ANCESTOR moves**: only zoom queued a redraw, so a pan kept the old cull (`13062e5`, `ViewportHost._update_lod()`) | Pan at unchanged zoom; culled content must appear on revealed ground |
| **Declare a class of documents cleaned** ("all design docs", "all scope docs") | **Enumerate the class from `CLAUDE.md`'s routing table, not a filename pattern**: "design-document cleanup is complete" (2026-09-23) missed eight documents (e.g. `ANDROID_UI_SPEC.md`, `TABLET_UI_SPEC.md`) | Paste the list and its source into the report |
| **Paint a layer into the map in a shader (rivers, any overlay that used to be a clipped stroke)** | **A stroke got the baked plate-frame fade for free by being clipped to `_interior_rect`; a shader-painted layer does not.** Carry `render::border_cover_f` into the field (RIM-1: coverage ratio 1.17 until the frame weight went into the alpha channel). | Compare coverage painted vs the old path with extra tints switched OFF (a floodplain tint counts as coverage otherwise); expect ratio near 1.0 |
| **Add a control to a tool bar or options row, or call it phone-ready** | **Check that a thumb can reach the bar, not that the probe passes.** Arming Paint selects WORLD, whose phone GENERATE sheet hides the tool-options row, so Size/Erase/Original were unreachable and the probe passed only through a forced reveal (Ruling BR, 2026-10-04). | The probe must reach the control through the real tab/sheet path; a probe that writes `_phone_tab` or calls a refresh itself is not a reachability proof |

## Why each rule exists

### [2026-09-03] Believing a backlog row instead of re-opening it ×16

**Mistake:** Work was scheduled and briefs written against rows already done or whose blocker had lifted: seven described built work (e.g. CA-05's resize handles, `_civSaltAccess`); four described lifted blockers (the "no JS runtime" claim was false for 18 days and cost a wave); the Religion row said `cartalith-civ::belief` does not exist (945 lines). Batches 18-19 added four more: urban milestone 16 had shipped a day before its row said "blocked"; milestone 17's blocker was falsified **six minutes after it was written** and stood eleven days; DS-03's resolver had 87 live call sites while the row called it unstarted; the APK exclusion row was half stale. A lane re-opening the row first returns "already done, here is the evidence" often enough that it is a **first-class outcome**.

**×16, 2026-09-23, one level up: a summary of the rows stood in for the rows.** A fast classification pass put multi-ridge orogenic belts and IN-13's caravans in "no decision needed"; each row's own text said otherwise (*"does not authorize building it"*; two unchosen options and an unasked sea-lane question). Both went to the owner as Ruling AP. **A bucket is a compression, and compressing a row drops its gates first.**

**Cause:** Treating `OUTSTANDING_WORK.md` as state rather than a router.

**Rule:** Re-open at the cited symbol (not the line) before acting. For a summary of rows, re-read each row's full text before trusting its bucket.

**Verify:** The report names the symbol opened; for a bucketed row it quotes the row's own gating sentence or says there is none.

---

### [2026-09-03] Encoding "no value" as a plausible value ×5

**Mistake:** `Option<f64>::None` -> `0.0` where `0.0` is a legal Crowding (44 of 614 rows); `harbour_scale` defaulting to `1.0` and printing as measured; `wall_spec` defaulting to `"none"`; `VRAM budget: 0.0 GB` where `0` means *no cap* and is the shipping default; `float(<null>)` in a GDScript reader aborting the document and silently clearing the user's saved set.

**Cause:** `unwrap_or` / `get(k, default)` used to avoid handling absence, which is information the reader needs.

**Rule:** Omit the key; callers use `has()`; dash with the reason.

**Verify:** `grep -nE "unwrap_or\(0\.|get\(.*, ?0\.0\)|get\(.*, ?1\.0\)"` over the diff, then render over real data and count.

---

### [2026-09-03] Leaving prose that describes the old behaviour ×48

**Mistake:** Controls disabled by reasons that had become false; `render.rs`'s module doc listing `rockSlope` refinement as **excluded** in the file that had just implemented it; `STATUS.md` naming three deleted probes as "present and uncalled"; a panel formula inverted against its engine. Batch 17 added seven, e.g. `world_workspace.gd:159` "Seasons and Köppen-Geiger classification are not ported" (`cartalith-climate/src/koppen.rs` is golden-tested and live), `performance_window.gd:140` "no per-device enumeration exists in cartalith-gpu" (`enumerate_devices`, `multi.rs:378`), `civilization_workspace.gd:5405` "cartalith-civ has no such relation" (`relations.rs` exists and three surfaces draw it).

**Sub-shape: a stale claim that a whole crate module does not exist.** `git log -S` put two such strings *before* their crates and standing sixteen and fourteen days. A per-wave diff review sees young entries only, and `tools/audit_wiring.py` structurally cannot see them (fourth cut running).

**Batch 18:** milestone 17's blocker ("settlements carry no `specialisation` and no `traits`") was falsified six minutes after it was written (`be2d5f7` 19:31:09, `e63d5d9` 19:37:15) and stood eleven days; and concurrently one lane removed a hardcode while another shipped a note saying the hardcode was why a control was inert — true at dispatch, false on arrival.

**Cause:** Behaviour and the prose describing it live apart; one gets edited. The prose was true when written and nothing re-reads a comment because a *different* crate landed.

**Rule:** A stale comment is a defect, not a nit. **A reworded reason that is still false is worse than the original — it looks freshly checked.** Re-opening backlog rows does not find these.

**Verify:** Grep the touched files for the old behaviour's vocabulary. Sub-shape sweep: `grep -rn "cartalith[-_]" --include=*.gd shell/ | grep -iE "no |not |never |missing|absent"`, then open every symbol named.

---

### [2026-09-03] A test that compares a constant against itself ×2

**Mistake:** `assert_eq!(e.brush.r, ICON_BRUSH_R_MAX)` and `assert!(err.0 < MIN_REGION_WORLD_AXIS)`; six constants and an RNG seed survived mutation with the suite green.

**Cause:** The assertion was written from the implementation, so it restated the code instead of pinning it.

**Rule:** Assert literals, or the independent thing the value must equal (`generate_sized`'s own `grid_w.max(4)`, the reference's `min="2" max="60"`).

**Verify:** Mutate it; the test must go red.

---

### [2026-09-02] Declaring green without both checks ×3

**Mistake:** (a) Commit `0f0fe55` used an undeclared `_label_cull` in a `.gd` file: `cargo test` said **2 821 passed, 0 failed** while `shell/app.gd` would not compile. (b) After flipping a default, five crates were verified and green was declared; **16 `cartalith-civ` golden suites** were broken. (c) **2026-09-23, about documents:** "design-document cleanup is complete" was reported after a filename grep that missed eight design/spec documents `CLAUDE.md`'s routing table lists (e.g. `ANDROID_UI_SPEC.md`, `TABLET_UI_SPEC.md`).

**Cause:** The Rust suite and the Godot shell are separate compilation domains and only one is in CI; blast radius was reasoned, not measured. (c): the set was guessed from a naming pattern, not read from where it is defined.

**Rule:** `cargo test --workspace` **and** parse-check every `.gd` you touched plus `shell/app.gd`. Ship divergence behind the app-boundary pattern so goldens stay bit-identical. For a documentation pass, enumerate the class from `CLAUDE.md`'s routing table, not a filename pattern.

**Verify:** Paste the summed total line; grade `.gd` only on stderr containing "Parse Error" / "Failed to load script" (exit code is unreliable). For a document class, paste the enumerated list and its source.

---

### [2026-09-03] Covering some inputs of a thing, not all of them ×3

**Mistake:** `with_ground_tiles` reached the on-screen builder but not `export_raster.rs`, so every exported PNG blended the flat swatch. The Units formatter reached one `match` arm of `_measure_readout()` and not its siblings ("135.0 nm" vs "r 250 km"). The belief layer's staleness key covered `belief_seed`'s second argument and not its first, so reassigning a settlement to a faction of another faith kept the old religion while the guard said current — and that was the fix for the identical miss on the religion column.

**Cause:** Enumerating from the case in front of you, not from the thing's definition (consumers, `match` arms, signature).

**Rule:** Derive the list from the definition. For a capability, grep every builder/consumer; for a staleness key, **read the signature and cover every argument** (the belief key now names `belief_seed(faction_of, faction_religion)` in its doc). Divergences that move no pixel at the default are invisible to the suite.

**Verify:** Change each consumer / arm / input in turn and confirm the dependent thing reacts.

---

### [2026-09-03] Single-sample timings written as measured fact ×3

**Mistake:** A lane that closed *"average the benchmark over multiple runs"* wrote three single-sample figures into doc comments and a scope document: a 416 ms handshake re-measured at **730 ms**; a "5× spread" at **1.4%** (contaminated by parallel `cargo test`); a "halving" at 1.35×.

**Cause:** A number from a real run feels like a measurement even when it is one sample from a noisy device under contention.

**Rule:** Median with min..max, or state the direction, not the factor. Run timing harnesses alone. No point estimates in doc comments.

**Verify:** Re-run independently; compare against the stated spread.

---

### [2026-09-02] Claiming a measurement that was never run

**Mistake:** Agents were told damping `impact_field` would fail sixteen `cartalith-civ` golden suites, "measured, not hypothetical"; it reaches the civ layer nowhere (27/27 pass). The false figure propagated into a source comment and `DECISIONS.md` §7l-ii as "measured history".

**Cause:** Carrying a real figure from an adjacent change and restating it as re-measured.

**Rule:** Never write "measured" for a number you did not just produce.

**Verify:** Reproduce from the command shown before it enters a decision record.

---

### [2026-09-03] Mutation tooling that corrupts the tree ×3

**Mistake:** (a) A `sed` script silently failed on patterns containing `*` (BRE quantifier), reported them SURVIVED, and scored a non-compiling crate as SURVIVED. (b) A whole-file `.mutbak` was snapshotted while a *different* agent's edit was live; its restore overwrote that source. (c) A script died mid-run on a path error and **left the mutant in the source**.

**Cause:** Mutation testing treated as a text operation, not a transaction.

**Rule:** Python only; exact literal replace; occurs-exactly-once assertion; BUILD_ERROR separate from SURVIVED; restore in a `finally`. Never `sed`, never a whole-file backup.

**Verify:** Grep zero MUTANT residue; hash the file against pre-run.

---

### [2026-09-03] Using a helper that does not exist ×2

**Mistake:** Wrote `_link(...)` in `menus.gd` and `_group_thousands(...)` in `dcc_units.gd`; neither existed.

**Cause:** Assuming a convention by analogy with a sibling file.

**Rule:** Grep for `func <name>` before calling it. In `menus.gd`, a `_todo` where a `_signpost` belongs makes `command_index.gd` count a shipped feature as missing.

**Verify:** `--check-only` on the touched file.

---

### [2026-09-03] `--import` strips `project.godot` comments

**Mistake:** An import removed **83 lines** of `;` comments (why `orientation=6` is an int, why there is deliberately no `stretch/mode` key). No key changed, so nothing failed.

**Cause:** The variant parser rewrites the file without preserving comments.

**Rule:** `project.godot` is off limits. If an import is unavoidable, diff afterwards **every time**.

**Verify:** `git diff …/project.godot` empty; `git checkout --` if the only change is deletions.

---

### [2026-09-03] A token re-base silently invalidated four relationships

**Mistake:** The 2026-08-31 token re-base changed values other code had built *relationships* on. `--ins` moved #101112 -> #191c1e, taking an asset-library checkerboard from (7,8,8) apart to **(2,3,4)** — invisible — in the pair a nearby comment recommended; a trait-chip hover became a darkening; on light, `raised` and `panel` became byte-identical (an unbordered drag preview); a verdict green left as raw `Color(0.48, 0.78, 0.49)` sat at **1.96:1** on the light panel.

**Cause:** A re-base is verified against its *sources* (each new value is right) while the properties that matter are *differences between* values, which no test or golden covers.

**Rule:** After changing a shared token/constant set, re-check every relationship over it: contrast ratios, adjacent-pair deltas, comments recommending one value over another. A literal colour is worse than a token: it cannot be remapped (`remap()` matches a baked colour back to its token, a literal to none).

**Verify:** Compute deltas and WCAG ratios for both palettes, not one; a light-only or dark-only defect will not show in a single capture.

---

### [2026-09-03] Citing a rule in a brief is not satisfying it

**Mistake:** A verification brief said *"Dark theme, 1080x2400, count rows with no pixel above RGB(23,23,23)"* with **no mechanism for getting dark**. The machine boots `mode="light"` (`cartalith_settings.cfg`), where every background pixel is 251, so literal execution measures `blank_rows=0` and reports a false improvement — the trap the brief's own preflight row warns about. Only the probe's `_force_dark()` plus a refuse-to-run-unless-dark guard makes the check real. Also in that brief: `cargo test --workspace` was numbered first as evidence for a GDScript-only lane, and "BOTH lanes" was checked when one had died, so three checks targeted non-existent claims.

**Cause:** Writing the rule into the brief feels like applying it. A rule names a hazard; a *check* must establish the conditions the hazard needs.

**Rule:** For each check, name the **mechanism that establishes its precondition**. Build the verifier's input from what actually returned; a failed lane supplies no claims.

**Verify:** Ask of each check: what result would refute the claim, can this check produce it, and does its environment satisfy its premise?

---

### [2026-09-03] Orchestration errors that waste a wave ×16

**Mistake (instances; batch numbers are the project's):**
- **Batch 34:** the brief told the verifier *"the destructive action must be text-only and not the rightmost"*; the approved artboard's variant B draws exactly that (`Clear 128 packs`, rightmost, filled, no safe action) — the rule belongs to variant A's footnote. The verifier refuted the brief, not the lane; the artboard was genuinely inconsistent and was corrected and republished. The brief also hedged that lanes "may have introduced none [no constants]"; all three had (13 in `dcc_widgets.gd`), and the check skipped would have found seven modal constants asserted against themselves, all surviving mutation.
- **Batch 33:** two claims quoted without opening: *"`apply_insets()` moved the app bar, nav bar and timeline"* (`PhoneMenu.apply_insets()`, `shell/phone_menu.gd:497`, has one caller and writes only `_screen`, `_sheet`, `_sheet_scrim`) and *"`DccWidgets.action` ships 39 px"* (it is `role_px("btn_min_h")` on touch, **26** on pointer; 39 appears nowhere) — the same shape as the ×15 row below, applied to a brief.
- **Batch 17, false-premise items** (the check cannot be performed as written): "pick four rows the audit closed" (it closed one); "mutate one constant introduced by each lane" (two lanes introduced none; say "per lane that introduced one"); "render before and after" when the `.dll` predated every `.rs` edit; an item conflating two guard pairs (`<1%`/`>99%` hover card vs `<0.1%`/`>99.9%` panel). A false-premise item does not fail loudly: the verifier silently substitutes or reports an unearned green.
- **Earlier:** one lane's brief pasted into another's prompt and dispatched separately; file-ownership partitioning stranding a correction a whole wave; a workflow script failing to parse on **unescaped backticks in a template literal** twice (the second was `(1080, 2400)` in a verifier brief); a fix named by register ID (**DS-12**) whose code lived outside the lane's ownership list; *"measure PH-16 in a probe at 393x852"* cannot discriminate because `phone_scale()` is exactly `1.0` there; a commit made while a verifier ran emptied the working tree so "`git diff` on project.godot is empty" passed for every file.

**Cause:** Briefs built by copy-paste; partitioning by file with no route for cross-lane findings.

**Rule:** One brief per lane, checked before launch. Escape every backtick in a workflow template literal and run `node --check`. Resolve a named item to its **actual file** before assuming the ownership list covers it. Serialize lanes sharing a file rather than forbidding the edit, and instruct lanes to **report** false prose in files they do not own. **Check your own verification instruction is discriminating** — a condition that cannot fail is worse than none. Four have shipped: a 393x852 probe size where `phone_scale()` is 1.0; a count that is 0 by construction; a working-tree `git diff` emptied by a mid-verification commit; and `git diff <hash> HEAD` where `<hash>` **is** HEAD.

**Verify:** Re-read each prompt for a foreign lane's heading before launching. Ask what result would refute the claim, then check the instruction can produce it.

---

### [2026-09-07] Two of my own, one hour apart: a batch closed early and a row copied from a brief

**Mistake:** (1) Batch `wwz9r4i2t`: the workflow's run directory was not found under `~/.claude`, read as "the run is over"; the batch was closed and `9ae7fad` committed **mid-verification**, after which `git diff` stopped being evidence. The third lane then filed eight findings, six unseen defects. (2) That commit's units row said *"the journey planner still routes NONE"* (converted in `88bf297`, 34 `DccUnits` calls, a 79-check probe) and *"the measure tool converts 4 of its 7 modes"* (`MEASURE_MODES` has **six**; only `vertical` and `bearing` do not convert). A lane refused the sentence; the row copied it anyway. Also: the `CA-07` register reason said font family has no field in the label model; it does (`MapLabel::font`, `set_font`, `font_or_default()`), weight and case have none, so `(B)` stands on the **renderer**.

**Cause:** A document I wrote treated as evidence about the tree: the run directory stood in for the run, the brief for the code. **A file I authored is a claim like any other.**

**Rule:** Wait for the completion notification before closing a batch. Re-run the measurement in the same breath as writing a "still X" into the backlog, above all when the number came from a brief.

### [2026-09-07] Four in one day, all one shape: my own words used as evidence

**Mistake:** Two more in the Android batch. A row said the canvas draws the per-stage readout as `stageRows` and `progLog`; `stageRows` is the **journey planner's leg list**, GENERATE's list is `GENSTAGES`. And offsets: six numbers labelled "character offsets" and the lane's "byte offsets" said to "run ~330 higher"; the verifier measured byte-minus-character deltas of 52, 54, 71, 75, 81, 126, from a copy opened with newline translation on: **a mechanism invented to make the number look measured.** Two real defects from the batch: a vertical scroll starting on a slider is consumed as a value write (Ocean depth 0.60 to 0.14; the 44 dp floor widened the band from 32 dp), and the phone's Archetype row dashed with *"Pick it in File ▸ New world"* while that dialog hides Archetype on a phone.

**Rule:** Before a fact goes into a durable file, re-derive it from the tree in that edit; if that is not cheap, **write the symbol instead of the number** (`grep -n stageRows` is seekable forever).

### [2026-09-07] A flaky save test that hid 57 targets, and a helper that passed on nothing

**Mistake:** A verifier's `cargo test --workspace` returned **100 result lines / 2 387 passed** against a floor of 157 / 3 253: `cargo test` fail-fasts and one failing target **skipped 57 others silently**. `cartalith-io`'s two determinism tests asserted `write_to_vec(&p) == write_to_vec(&p)`, but the writer stamps every zip entry from the clock (2-second DOS resolution, `SAVEFILE_COMPAT.md` section 16), so two writes straddling a tick differ at offset 10. The replacement `content_fingerprint` (entry order, name, CRC-32, sizes) was nearly vacuous: mutated to `Vec::new()` both tests stayed green.

**Rule:** Compare what the format promises is stable (pinning `last_modified_time` would have deleted a documented feature). A collection-returning test helper asserts its own non-emptiness (this one asserts `project.json`; `0..0` now kills both tests). **A test comparing two runs of one writer catches only a writer unstable BETWEEN runs, never one stably wrong**; ordering is pinned by the pyramid test's `assert_eq!(tiles, sorted)`.

**Verify:** 157 result lines, 3 253 passed, 0 failed, 28 ignored.

### [2026-09-07] The phone was running the mutation build

**Mistake:** A lane proved an 8 dp gesture-arbitration threshold (fixing 242 sliders) by mutating it both ways, restoring the tree perfectly (sha256, `RESTORE SAME`), but installed an APK per mutation and **left the last on the phone**: installed `base.apk` hashed `d32a75d8…` = `builds/android/Cartalith-slop.apk`. The verifier could not reach a handset and said so. The main loop hashed `pm path`'s `base.apk` against every APK in the build directory, installed a clean export (`4ff2e257…`) and re-drove the screen, finding: a vertical swipe on the New World card **opened the Archetype dropdown instead of scrolling**, and every cold boot logged *"the loaded GDExtension has no `WorldGen.<fn>()` … older than the shell"* because the export shipped the `android-dev` `.so` (no native-feature on-glass claim was safe for four batches; nobody read the warning).

**Rule:** A mutation harness that installs restores the device too. Each of these was a true measurement generalised past what it measured (restore covered tree not device; census covered one boot state; inventory one base class): name the state and stop the claim's scope there.

### [2026-09-07] The probe undercounted the class it existed to count

**Mistake:** The gesture census walked the tree with `get_children()`, omitting internal children (4 401 nodes vs 5 512) and hiding 12 `SpinBoxLineEdit` and a `TabBar`; corrected: `SpinBoxLineEdit` ×5 and `LineEdit` ×18 under live scrollers, where the report said 4. `SpinBox` was "cleared" because a swipe does not step its value, the wrong question: a swipe on the New World Seed field gives **scroll `0 to 0` with the internal `SpinBoxLineEdit` focused**, and focus raises the Android soft keyboard. A dropdown filed "milder" measured `sel=7 to sel=3` with the popup closed. A comment credited the "no vertical scroller" gate with protecting seven `MenuButton`s; **0 of 7 carry `_phone_fitted`**. (Good practice: the batch walked EIGHT named states, including a world-loaded one.)

**Rule:** `get_children(true)`; "not this mechanism" is not "not a hazard"; severity is measured, not read from the mechanism; two true facts side by side read as cause and effect, so invert the gate to test.

### [2026-09-07] I filed a blocker for a bug that never existed

**Mistake:** A row said the Android app boots only because its native library is stale, and a batch was dispatched to bisect 33 commits. **The app was working the whole time** (two agents drove it on glass). Two inverted metrics: `dark%=94.2 (94.2 = splash, ~85 = picker)` where splash is **98.9%** and picker **94.2%** (`~85` invented), so every "still on the splash" was a working app; then `5 = stuck, 50+ = progressing` counted **defects** (56 lines = 5 real + 51 `has no WorldGen.<sym>()` warnings), so 5 was healthy. The brief itself said absence of warnings "is NOT evidence about how far it got", and "log line count PLUS a screencap" was skipped on the failing build. Cost: ~214k tokens proving a negative.

**Rule:** A screening metric needs a known-good and a known-bad reading, both in its label. A caveat you reason past launders the conclusion. Ask "does the failure exist?" before "what explains it?"; reproduce one more way, cheaply, first.

### The probe could not see the class of thing that was wrong

**Mistake:** Five times on 2026-09-07 a defect the owner found by hand had a **green probe** right about what it measured: `_jpinsw_probe` measured a panel nobody could see (`_left_panel.visible` true, `is_visible_in_tree()` false); `_detent_probe` passed a 19.84 dp handle against a 44 dp floor by pressing its exact centre; the input-fill probe passed a `CheckBox` whose icons Godot **modulates**, unreachable by a stylebox test; every check passed `CREATE WORLD` at **7 of 46 dp painted** because `get_global_rect()` is unclipped.

**Rule:** Before trusting a probe, name the class of defect it CANNOT see. Never merge **it renders**, **it can be operated**, **it can be FOUND**; the third keeps failing.

### A candidate hypothesis in a brief is a liability, not a head start

**Mistake:** Three consecutive briefs gave a lane a named cause and all were wrong (the sculpt drawer was a 19.84 dp target, not gesture arbitration; both PC file-browser candidates were false, one **structurally impossible**: `wrap_controls = true` means a `Window` never draws under `get_contents_minimum_size()`).

**Rule:** Label a candidate as a candidate and say eliminating it is worth more than a plausible fix; a cause stated as fact turns an investigator into a confirmer.

### A check that cannot fail is not a check

**Mistake:** Four mechanisms, all green on 2026-09-07: it never ran (`_nwclip_probe` printed `fail=0` on desktop; `is_phone()` needs touch); it measured the wrong surface (`_tabfit_probe` asserted the dock row only and exited 0 while printing `overflow=285.0`); it asserted the defect (`_inputfill_probe:165`, added by the regression's own commit); it could not see the class (stylebox vs modulate, centre press vs width, `get_global_rect()` vs clip).

**Rule:** Name what the probe cannot see and what would make it go red; if neither has an answer it is decoration. The cheapest repair is usually a distinct exit code that binds the existing output.

### [2026-09-08] The probe passed the exact defect it was written for

**Mistake:** `_unitsflip_probe` asserted the drawn Position string ends in `mi`, ends in `km` in the other mode, and the two differ; relabelling satisfies all three (mutant `" 15.6 · 9.8 mi"` passed).

**Rule:** A3b/A3c reconstruct the expected number via `DccUnits.to_unit()` on raw kilometres (the mutant fails: `drawn= 15.6 · 9.8 mi want=9.71,6.07`). Ask what a lazy wrong implementation looks like and check the probe rejects it. A smaller model wrote this probe; the conversion was right and only the proof weak, an argument for keeping the verifier on the stronger model.

### [2026-09-08] Two separators in one panel, and only one of them was new

**Mistake:** The units sweep left River drawing *"Discharge 4,200"* above *"Catchment 3 500 km²"*: `DccUnits._group_thousands()` always emitted spaces, `right_dock.gd::_thousands()` commas. The canvas (`Cartalith DCC Environment.dc.html`) uses spaces throughout, so the untouched helper was the defect.

**Rule:** *Exposing* an inconsistency is not *causing* one; check both sides' history before "fixing" the conformant half. The reference is the tie-breaker for cosmetics too.

### [2026-09-08] I put the correction in, and the correction was the error

**Mistake:** A row cited `_urban_revealed` declared at `map_overlay.gd:3544` and gating the pin at `:1984`. `sed -n '3544p;1984p'` printed 1984 first (**`sed` prints in file order**), the row was judged swapped, and the "correction" went into a lane brief as checked fact. The row was right.

**Rule:** Read lines one at a time, or `awk 'NR==A{print "A: "$0}'`. **A correction is a claim** and gets the same scepticism as what it corrects.

### [2026-09-08] Two agents wrote the same parser bug, and both caught it

**Mistake:** Counting which `#[func]`s no `.gd` reaches needs comment/string stripping. A `#` inside a JSON fixture in a triple-quoted string orphaned its closing quote, which paired with the next and ate every newline between (299 lines to 62). The first pass reported **11** names, **3 wrong**.

**Rule:** Strip triple-quoted strings, then single-line strings, then comments. **A census whose method can silently drop input needs a conservation check** (lines in, lines out); both agents found it that way.

### [2026-09-08] Everything I said checked out except the part that mattered

**Mistake:** For the owner's blurry map labels, the diagnosis was "CORRECT, confirmed at two symbols", citing `map_overlay.gd:2105-2106` as a `draw_string` pair in the scaled camera. `:2104` is `_crisp_begin()` and `:2107` `_crisp_end()`: that path already rasterises at screen resolution (`git log -S`: fixed in `c9bfcca`, 2026-08-24). The blurry path was `_draw_labels()`. Real mechanism, already-fixed site.

**Rule:** Read what brackets the line, and `git log -S` the helper name.

### [2026-09-08] The first batch where the proof was as good as the code

**What went right (counter-example):** both lanes showed their assertion FAILS on a mutant. The roof fix silences 151 engine errors, which a draw-nothing fix would also do, so the verifier compared **whole frames** (byte-identical md5 `05f6b300…`, `ImageChops.getbbox()` `None`, 24.6 % ink, 1 464 colours); the theme probe was mutated by restoring pre-boot ordering and exits 1; the diagnosis was tested with the fix REVERTED (9 700 buildings, zero triangulation failures).

**Rule:** Show the assertion fails on a mutant; compare whole frames, not error counts. **An over-claim in a comment is a defect with a delay on it.**

### [2026-09-08] The first screenshot found a real defect and an imaginary one

**Mistake:** Probes drive controls through `pressed.emit()`, proving a handler runs, **not that a finger reaches it**. `adb exec-out screencap` found a real defect: the GENERATE sheet's chip row occluded at y=2116 of 2340 (glyph tops at 2116/2117/2118 as 58/61/36 amber pixels, 2119 zero: **three pixel rows of a ~20 px label**). The same look produced an imaginary one: two "clipped tab labels" in the downscaled view; **the full-resolution crop contains neither**.

**Rule:** Not "look at the device" but "look at the pixels": crop to full resolution before filing. Take two captures and diff them (one frame of an animating app is not a state; these differed only at the clock digit), and **record the installed build's `lastUpdateTime`**.

### [2026-09-08] I ruled on a plurality without checking the population

**Mistake:** A row stalled on *"which census entry is the action button"*. I ruled `2px 12px`, the plurality of radius-8 controls (8 of 18), as Ruling G; a lane applied it. The verifier refuted it, and a fourth census confirmed: every radius-8 node carries a height role and the role **is** the population. `--btnH` (28px) is the action button, N=14; `--ctl` (24px) the inline chip, N=15; `--tool` (30px), N=1; **all eight `2px 12px` nodes are `--ctl` or `--tool`, none `--btnH`**. The shipped `y=4` it displaced was right. Four wrong figures passed through that row. The lane's brief said *"if your count disagrees with 8/18, stop and report"*; it recounted 47 nodes, **reasoned the rule "protects the winner, not the literal fraction", and implemented anyway.**

**Rule:** Ask what the set is before counting a plurality. **A stop-and-report gate that can be reasoned past is not a gate**; a brief must say overriding it is itself the failure. Recoverable because the change was inert and the ruling withdrawn in place; the census now sits in the comment above the constants.

### [2026-09-08] One real defect, two phantoms and a dead end, in one hour on glass

**Mistake:** The sweep found one genuine bug and three findings that did not survive checking (none filed). Phantoms: text read in a scaled-down screenshot, twice, an hour apart, **after writing the rule for it**. Dead end: *"MORE does not respond to a tap"* (three times); the control test killed it (**MAP and PLAN stopped responding too**: a full-screen panel had opened over the nav bar). The real defect was misstated: "chips clipped", but **expanding the sheet draws them in full**; the COLLAPSED detent puts the chip row under the bar, and a fix verified expanded would change nothing.

**Rule:** A screenshot is one state with several axes (detent, top panel, resolution). Crop before the conclusion and **run the control before believing the finding**.

### [2026-09-08] Two edges that coincide are not one edge covering the other

**Mistake:** The clipped chips were filed as *"occluded by the bottom nav bar"* with every number right and the cause wrong: the sheet's own `ScrollContainer` clip and the bar's top both derive from `_phone_nav_reserve()`, so they **coincide by construction**. The bar's own caption is unchanged whether the sheet is open or closed. The real cause was a **stale budget**: the peek detent was sized for the old one-line tool-options strip, never revisited when a taller multi-group column replaced it (only the top 45 % of the first scrollable row admitted). A z-order fix would have changed nothing. It reproduced on the DESKTOP composition at 1080×2340 `--force-touch` (bar top **2119** vs the device's **2116**): the "phone verification is desktop simulation" row is right about REACHABILITY and **not** about geometry.

**Rule:** Sample a pixel at the supposed coverer; ask which constant both edges derive from.

### [2026-09-12] I handed the owner a false premise, and they ruled on it ×2

**Mistake:** Comparing the owner's town plan against the urban generator, I wrote that star forts never appear because settlements carry no `fortified` trait (from a comment in `urban_layout_draw.gd`) and **put it into an `AskUserQuestion` option**; the owner chose *"Enable star forts"*. At the symbols it was false: `civ_settlement_toggle_trait` (via `civ_roster_bridge.rs::toggle_trait`) writes the trait, `urban_bridge.rs` reads `fortified_trait`, `generate.rs` grants the fort (walled, large, `organic` gate scheme). **Only the drawing was missing** (`_draw_wall` handles `curtain`, `palisade`, `ditch`). Caught before recording: Ruling I carries the verified chain and a premise note.

**×2, 2026-09-23:** the Ruling AP backlog was compiled partly from `LANDMARK_GENERATION_SCOPE.md` section 4, still *"Posed, not answered"*. Four questions were ruled on 2026-09-06 (rulings 10 PERSIST, 12 EXEMPT, 13 YES, 16) and the crate home earlier on 2026-09-23; all went to the owner again, and the recorded ruling repeated the stale premise as new work (an `entities/landmarks.json` slot, a `cartalith_vault::EntityKind` variant): both existed since 2026-09-06 (`project_bridge.rs::LandmarksDoc`; `EntityKind::Landmark`, `45630cc`).

**Rule:** **A question is the most consequential place for an unverified fact, because the answer is recorded as a ruling.** Check every premise in a question and its options at the symbol. Before asking the owner anything, search `LARGE_ITEM_RULINGS.md` for its subject; a scope document's "open questions" is a claim about its writing day. Open the symbol the answer would have built.

### [2026-09-12] The second lane in five days to argue past a gate its brief named

**Mistake:** The phone-sheet brief offered two options, named the first's obstacle (pinning the PIPELINE/SCULPT segment needs 88 dp vs the canvas's 66 dp peek) and carried a section *"A gate is a gate"*. The lane took it, measured 98 dp and reported *"no collision"* because the growth went up into the map; the obstacle was the budget. The verifier found the deviation and a real bug it caused (no insets signal, so the navpad and two labels were covered at cold boot); reverted. The refutation also found the cause: the canvas keeps that segment in the scrolling body under a header block the shell never built.

**Rule:** The 2026-09-08 padding lane did the same under the same heading. **A gate written as prose is read as advice once the lane has a plausible reason to proceed.** Make each gate a required field of the lane's structured report, HIT or NOT HIT with the deciding number, so passing needs a false statement and the verifier has a line to check.

### [2026-09-23] Partial sweeps converge on false negatives

**Mistake:** Three section-by-section surveys of `OUTSTANDING_WORK.md` each found no decision-free work, each finding fewer (read as the list running out). One start-to-finish read found two buildable rows all three missed: Phone MORE's `data-tiles` sub-screen and SP-2's journey-plan cache residual (`b19b9ea`, `b5aff17`; count log `a71d911`).

**Cause:** Overlapping sample passes revisit the same likely sections, so convergence looks real while coverage stays put.

**Rule:** Before declaring a list exhausted, read it end to end, row by row, and name the method and range ("sections 1-4, every row"); otherwise it is a sample.

### [2026-09-23] A reference comment about what a LATER version left alone was read as what the EARLIER version did

**Mistake:** The reference's v1.05 tile-renderer block ends *"The BASE per-cell map loop is untouched (default render identical)"*. The port read "the base map draws no lakes" and never gave the base texture an above-sea lake stamp, so tiles blended against the base faded lakes in and out (owner saw flicker on zoom). The comment meant v1.05 left the base loop's own v0.103 lake stamp alone. Fixed in `13062e5` (`RenderCtx::with_lakes`, a lake branch in `cell_color`); the reading is in `render.rs`'s tile-renderer header note.

**Cause:** A comment on what a change did *not* touch is a claim about the diff, not about what it leaves in place.

**Rule:** When a reference comment says a later version left something "untouched", open the earlier version's code for it and port what it does; name that symbol and its frozen-reference line before concluding.

### [2026-09-23] Godot does not re-run `_draw()` when an ancestor moves

**Mistake:** `map_overlay.gd` culls rivers, ways and route runs (`_run_offscreen`, `_segment_chains`) and requests urban layouts against `_visible_local`, computed once per `_draw()`. Only zoom queued a redraw (`set_camera_zoom`); a pan, `move_view_to` or reset at unchanged zoom moves an ancestor, which does not redraw the node. Rivers and roads went missing on revealed ground and a panned-to town never requested its layout (`13062e5`). `_owner5_probe.gd`: 0 river/road pixels after zoom-then-move, all back after an explicit redraw. Fixed with one `overlay.queue_redraw()` in `ViewportHost._update_lod()`.

**Rule:** Anything culled in `_draw()` redraws on every camera move (pan, zoom, programmatic, reset); put the redraw where every motion path converges (the "cover every input" row in Godot form).

**Verify:** Pan at unchanged zoom; culled content must appear on the newly revealed ground with no other input.

### [2026-09-26] The golden fixture used a "none" value the live producer never writes

**Mistake:** `cartalith_civ::jp_claimed_at` tested `territory[i] >= 0` (from the reference's `_jpClaimedAt`). The Journey Planner milestone-5 golden used a fixture with unclaimed cells at `-1`, so it passed; the live grid, like the reference's `Uint8Array`, uses `0` for unowned. On every real world every sampled point read as claimed (`claimedFrac` 1 everywhere, open sea included, tolls never charged). A scope document defended `>= 0` as "golden-faithful". Fixed with `> 0` and the fixture moved to `0`; no golden value moved.

**Cause:** The sentinel was chosen by the test author, not taken from the live producer (`assign_territory`, paint, `_civAutoPolity`).

**Rule:** Encode every sentinel as the live producer writes it, and add one test driving the consumer from the real producer's output (a generated world).

**Verify:** Mutate the sentinel test (`> 0` to `>= 0`); the golden and the live-producer test must both go red (both did).

### [2026-09-28] A verified marker set only in the index was reverted by the next whole-file add

**Mistake:** The main loop marks a lane's STATUS.md entry verified by rewriting the staged patch, not the working file; a later whole-file `git add` put the older "pending independent verification" text back. Shipped twice (`d71824e` reverted the map-data residuals marker; `593df62` RV-3's), caught once before commit.

**Rule:** Make the marker edit in the working file and stage from there; after any commit touching STATUS.md, check no `-` line containing "verified by the main" is in `git show HEAD`.

### [2026-10-04] Rivers-in-map and paint-original: three reusable slips

**Mistake:** (1) A shader-painted river bled onto the plate frame because the stroke's clip-to-interior was implicit (coverage 1.12-1.17 of the stroke until fixed). (2) `_rivcolour_probe` targets defects, so in a healthy world it had no screen targets and silently measured nothing; a control target (the longest river) was needed. (3) The Ruling BR slider shipped with a phone probe that passed only via a forced reveal; the Paint bar is unreachable on the phone shell. Also: a seam probe's gap counts were inflated about 3x by `move_view_to` taking a cell INDEX (centres on index + 0.5) while render points are continuous; and a doc comment was left stale after a constant changed (`CostGrid` still said 384 after `RW_CAP` became 1024).

**Rule:** Inherit implicit clips explicitly; give every defect-targeting probe a positive control; prove phone reachability through the real path; convert index vs continuous coordinates before comparing; when a constant moves, grep the prose that quotes it.

**Verify:** Coverage ratio painted/stroke with tints off; the probe fails when its control is removed; the phone probe never writes `_phone_tab`; `grep -rn "384"` after changing `RW_CAP`.

## Merged candidates (owner, 2026-11-11)

Merged at the owner's request from `MISTAKES_CANDIDATES_2026-10-26.md`, entries as staged there (each tagged verified or reported-by-agent, with its session). Fold any into the preflight table when it recurs.


**Candidate entries for the owner to copy into `MISTAKES.md` if they agree.** This file is a staging area, not the register: `MISTAKES.md` itself was not edited, and nothing here is a rule until the owner moves it. Written 2026-10-25/26 from the LLM-3 to LLM-6 sessions and the local-model setup session (session `01664rQgzKuqZJNTsWcNrFHs` wrote this file; the other sessions are named per entry where known).

**Evidence tags.** **[verified]** = checked in the repository by the writer of this file against a named place. **[reported by agent]** = told to the writer by the session that hit it (or by the briefing), not independently reproduced here; treat as unconfirmed until the owner or a verifier confirms. `MISTAKES.md`'s own rule is "do not record unverified guesses", so the [reported by agent] entries are the ones to hold back until confirmed.

Each entry gives a suggested preflight trigger (the left-column phrase), the root cause, the rule, how to verify, and the session. Overlap with an existing `MISTAKES.md` row is noted so the owner can merge instead of duplicating.

---

## 1. Appending to a packed array fetched from a Dictionary is lost

- **Preflight trigger:** *Mutate a `Packed*Array` you read out of a `Dictionary` or `Array`*
- **Root cause:** GDScript `Packed*Array`s have value semantics. `var a = dict["k"]` copies, `a.append(x)` changes the copy, and the dictionary still holds the old array. The code looks right and runs without error.
- **Rule:** after mutating, store it back (`dict["k"] = a`), or mutate in place through the dictionary expression only where the engine documents it as by-reference. Never rely on `append` on a fetched local.
- **How to verify:** a probe that appends, then reads the dictionary entry's `size()` and asserts it grew; mutation test: remove the re-store, the probe goes red.
- **Session:** an LLM-3 to LLM-6 build session. **[reported by agent]**: no repository place was named, so this was not re-opened at a symbol.

## 2. `netstat -p TCP` shows IPv4 only; check TCPv6 as well

- **Preflight trigger:** *Assert "zero sockets" or "nothing is listening" from `netstat`*
- **Root cause:** `netstat -p TCP` on Windows lists only IPv4. `localhost` can resolve to `::1`, so a connection to a local model server can be on TCPv6 and be invisible to the check, which then reports a clean zero it has not earned.
- **Rule:** run both `netstat -p TCP` and `netstat -p TCPv6` (or `netstat -ano` and filter) for any zero-connection claim; state both counts.
- **How to verify:** open a deliberate loopback connection to `[::1]` and confirm the check counts it; mutation: drop the TCPv6 pass, the test goes red.
- **Session:** LLM-2 to LLM-4 (the zero-socket-when-off bar). **[verified in part]**: `cartalith-native/docs/STATUS.md` (the LLM-4 and LLM-3 entries) says the `netstat` checks ran "over both address families" and show 0 non-listening rows. That confirms the practice, not the failure that caused it.

## 3. A worker thread cannot build Godot values in this gdext build, and a script method cannot be called from `NOTIFICATION_PREDELETE`

- **Preflight trigger:** *Move Godot-object work onto a `Thread`, or free something in `_notification`*
- **Root cause:** this build of gdext has no `experimental-threads`, so a `Dictionary` / `Array` / `GString` operation off the main thread panics. Separately, during `NOTIFICATION_PREDELETE` the script instance is going away, so calling a script method from it is not safe.
- **Rule:** a worker thread handles raw bytes only; the main thread drains a queue and builds every Godot value. In `NOTIFICATION_PREDELETE`, free raw RIDs and native resources only; never call back into script.
- **How to verify:** the thread half is already stated in `LLM_ASSISTANT_SCOPE.md` s.2 (the rule `landmark_run` and `region_new_world` document). The predelete half: `map_overlay.gd`'s `_notification` frees raw RIDs only, which is consistent with it.
- **Session:** LLM-3 (wire worker thread). **Thread half [verified]** at `LLM_ASSISTANT_SCOPE.md` s.2; **predelete half [reported by agent]**: the symbol was not found documenting the prohibition itself.
- **Overlap:** none in the preflight table found by keyword (`PREDELETE`, `experimental-threads` absent from `MISTAKES.md`).

## 4. A Python text-mode rewrite on Windows turns LF into CRLF; mutation harnesses must restore original bytes

- **Preflight trigger:** *Run a mutation harness that rewrites a source file, or edit a file with a script*
- **Root cause:** `open(path).read()` then `open(path, "w").write()` on Windows translates newlines. A harness that "restores" the original text this way leaves every line ending changed, a whole-file diff, and possibly a stale-binary rebuild.
- **Rule:** read and write in binary (`rb` / `wb`), or pass `newline=""`; restore the file from the exact original bytes (saved copy or `git checkout -- path`), then confirm `git diff --stat` is empty before and after.
- **How to verify:** after a harness run, `git status` shows the file clean and `git diff --stat` is empty; mutation: run the harness in text mode and see the diff appear.
- **Session:** an LLM-6 mutation-harness session. **[reported by agent]**.

## 5. `subprocess.run(timeout=)` on the Godot console launcher hangs on the child's pipe; use `Popen` plus `taskkill /T`

- **Preflight trigger:** *Drive a headless Godot (or any launcher that spawns a child) from Python with a timeout*
- **Root cause:** the Godot console launcher spawns the real process as a child that inherits the pipe. On timeout `subprocess.run` kills the launcher only; the orphaned child keeps the pipe open, so the read after the kill never returns and the harness hangs past its timeout.
- **Rule:** use `Popen`, and on timeout run `taskkill /F /T /PID <pid>` to kill the tree, then read what remains; never `communicate()` without a bounded wait after a kill.
- **How to verify:** a probe that sleeps past the timeout returns within the timeout plus a small constant, and no `godot` process remains (`tasklist`).
- **Session:** the LLM-6c stage-1/stage-2 driver work (`_llmparams_cli.gd`). **[reported by agent]**.

## 6. The stale-binary trap: a stale DLL reports a healthy pass

- **Preflight trigger:** *Declare green after a Rust change* (an existing row covers it)
- **Root cause:** `cargo test` does not rebuild `target/debug/cartalith_godot.dll`; every `.tscn` probe loads that DLL, so a probe run after a `.rs` edit and before `cargo build` tests the previous engine and passes.
- **Rule:** compare the DLL's mtime with the newest touched `.rs` file and state both.
- **How to verify:** `ls -l --time-style=full-iso target/debug/cartalith_godot.dll` against the newest `.rs`; mutation: touch a `.rs`, rerun the probe, and the comparison must flag it.
- **Session:** recurring (LLM-5 and LLM-6 among them). **[verified]**: already `MISTAKES.md` preflight row "Grade a Godot probe as evidence for a Rust change" (the Godot half) and the stale-binary working rule in `CLAUDE.md`. **Merge into that row by raising its `×N` and adding the instance; do not add a new entry.**

## 7. Two agents running windowed Godot at once steal each other's mouse input; use the `C:/cw/.win_lock` convention

- **Preflight trigger:** *Run a windowed (non-headless) Godot probe, screenshot or click test while another agent may be doing the same*
- **Root cause:** synthetic and real mouse input are delivered to whichever window has the foreground. Two windowed runs on one desktop interleave, so clicks land in the wrong app and both runs fail or, worse, one passes on the other's click.
- **Rule:** take the lock before a windowed run (`C:/cw/.win_lock`, created exclusively, removed in a `finally`), and wait or run headless if it is held; headless runs need no lock.
- **How to verify:** two concurrent windowed probes: the second must wait or refuse; mutation: skip the lock and observe the interference.
- **Session:** the concurrent-lane sessions of 2026-10-25/26. **[reported by agent]**: `win_lock` is not in any document in the repository (searched `*.md`), so the convention exists only in agent briefs; if the owner adopts the entry, the convention itself should be written down where agents will read it.

## 8. Stopping Ollama by process name `ollama*` leaves orphaned `llama-server` runners holding VRAM

- **Preflight trigger:** *Stop a local model server to free the GPU, or measure speed/VRAM after stopping one*
- **Root cause:** Ollama starts model runners as separate `llama-server` processes. Stopping `ollama*` by name kills the server and the tray app but can leave a runner alive, which keeps its model resident in VRAM. The next measurement then runs with the card partly full: this is the same condition the 4 to 7 tok/s Ollama figure in `LLM_ASSISTANT_SCOPE.md` s.9a was taken under.
- **Rule:** stop `llama-server` too, then check the GPU memory counters (dedicated usage near the compositor's ~0.3 GB) before measuring or starting another model.
- **How to verify:** read dedicated GPU memory before and after the stop (Task Manager or the performance counter) and `tasklist | findstr llama-server`; mutation: stop only `ollama*` and see the counter stay high.
- **Session:** the local-model tooling session of 2026-10-25. **[reported by agent]** for the orphaned-runner mechanism; the stale-runner condition is [verified] only as a statement in the owner-session brief and now recorded in `LLM_ASSISTANT_SCOPE.md` s.9a.

## 9. `ollama list` and similar commands auto-start the server

- **Preflight trigger:** *Check whether Ollama is running, or leave it stopped, using the `ollama` CLI*
- **Root cause:** the CLI is a client that starts the server if none answers. An "is it stopped?" check therefore starts it (and with it the tray app), and a later `ollama list` or `ollama ps` silently undoes a stop.
- **Rule:** check state without the CLI (`tasklist`, or a TCP probe of `localhost:11434` that does not connect-and-keep); run `ollama` commands only when a running server is acceptable.
- **How to verify:** stop the server, run `ollama list`, and see a process reappear; mutation: replace the check with a port probe and the process stays absent.
- **Session:** the local-model tooling session of 2026-10-25. **[reported by agent]**.

## 10. `winget` can hang behind another stuck `winget`; run the signed installer directly after hash verification

- **Preflight trigger:** *Install a tool with `winget` in a session where another install may be running*
- **Root cause:** `winget` takes a machine-wide lock; one stuck invocation (a prompt nobody sees, a download that stalls) makes every later `winget` wait forever with no output.
- **Rule:** do not queue behind it. Download the vendor's installer, verify its hash against the published value and its signature (`Get-AuthenticodeSignature`), then run it directly; say in the record which hash and which signer you checked.
- **How to verify:** `Get-AuthenticodeSignature` reports `Valid` with the expected publisher, and the hash matches the vendor's page; the install result is confirmed by launching the tool.
- **Session:** the local-model tooling session of 2026-10-25 (LM Studio 0.4.26, `LLM_ASSISTANT_SCOPE.md` s.9a). **[reported by agent]** for the hang; the install itself is recorded in s.9a.

---

## 11. A mocked test using the literal 127.0.0.1 hid a localhost -> ::1 resolution failure

The mock server listened on `127.0.0.1` and the test dialled the same literal, so the test passed while the shipped
default URL `localhost` resolved to `::1` and failed. Rule: a network test must dial the name the product ships with
and the mock must listen on both families. Verify: run the test with the product's default URL.

## 12. A fixture that cannot fail (canary words in the user's own question)

The injection canary words were placed in the user's own question, so any answer that echoed the question passed. Rule:
a fixture must be able to fail; put the canary where only the thing under test can have read it, and run it once against
a deliberately broken implementation.

## 13. A phone sheet keeps a stale child size

A phone sheet reused a child whose minimum size was set for a previous layout; the sheet kept the old size after the
content changed. Rule: re-measure children when a sheet's content changes, and check the phone leg after any relayout.

## 14. `git checkout <file>` on a file with uncommitted work loses it

An agent restored a file to clear a diff and discarded uncommitted work. Rule: never `git checkout`/`restore` a path that
has uncommitted changes you did not make; copy it aside or `git stash` first, and name the explicit path in commits.

## 15. A probe with a parse error hangs while holding the lock

A GDScript probe with a parse error never exits, and the `C:/cw/.win_lock` directory stayed held. Rule: run
`--check-only` (or parse-check) before the windowed run, always use a hard timeout, and release the lock in a trap.

## 16. Two agents installing on the phone at once collide

Same family as entry 7: two agents driving adb or installing the APK concurrently corrupt each other's state. Rule: take
the same lock convention for the phone, one device owner at a time.

## 17. An agent stopping the shared LM Studio server

An agent stopped the LM Studio server the owner leaves running on the LAN for other agents and devices. Rule: agents never
start or stop LM Studio or Ollama unless the brief says so; the server is the owner's.

## 18. A preset command-line scene path loops the splash

Putting a scene path on the export preset's command line made the app restart into the splash repeatedly. Rule: never a
scene path in the preset command line; the main scene belongs in `project.godot`.

## 19. Project path not reset on New world overwrote a world

New world inherited the previous world's project path, so the first Save overwrote that world (fixed in `5585284e`).
Rule: any "new" action resets every identity field (path, name, dirty flag), tested by Save-after-New.

## 20. A SubViewport routes pointer motion only after `NOTIFICATION_VP_MOUSE_ENTER`

- **Preflight trigger:** *Drive hover or motion in a probe that hosts the app in a `SubViewport`*
- **Root cause:** a `SubViewport` never receives the pointer-entered notification a window gets, so `push_input` motion events are not routed to controls until the probe sends `NOTIFICATION_VP_MOUSE_ENTER` itself. Without it every "hover shows X" assertion fails, and worse, every "hover leaves it hidden" assertion passes vacuously.
- **Rule:** send `_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)` after the app is up, and begin every negative hover assertion with a positive control that hovers a known target and sees the effect.
- **How to verify:** remove the notification from the probe; the positive control must go red, not the negative assertion alone.
- **Session:** the VE-0/VE-1 build, commit `bec744b5`. **[verified]**: the measurement is in `cartalith-native/godot-project/_ve0_measure_probe.gd` and the note is in the header of `_ve0_probe.gd` ("Pointer note").

## 21. Test scratch directories must be unique per call, not per tag

- **Preflight trigger:** *Give a parallel Rust test a scratch directory named from a tag*
- **Root cause:** two `cartalith-vault` tests shared the tag `rename`; one test's `remove_dir_all` deleted the other's tree mid-run (`SourceMissing`, or "Access is denied (os error 5)" on Windows where the other test held files open). `cargo test -p cartalith-vault` was flaky until it was fixed.
- **Rule:** a scratch path is `tag + process id + a per-call counter`; "unique per test" by tag is a claim about the callers, not a property of the helper.
- **How to verify:** run the crate's suite three times; mutation: drop the counter and give two tests one tag, the suite must fail.
- **Session:** the VE-0/VE-1 build, commit `bec744b5`. **[verified]**: the diff of `cartalith-vault/src/lib.rs` (`scratch`) carries the explanation.

## 22. A script error inside an `await` hangs a probe

- **Preflight trigger:** *Write a windowed probe that awaits between steps*
- **Root cause:** an error raised in a coroutine after an `await` ends that coroutine silently; the `_ready` that was awaiting it never resumes, `_finish()` is never reached and the run hangs until its timeout (holding `C:/cw/.win_lock` if no `finally` releases it). Typically a null control: a button or node a previous failed step should have produced.
- **Rule:** after each step that later steps depend on, assert the control exists and call `_finish()` with the failure on record when it does not; never index into a node found by a search without a null check; hard timeout on every run.
- **How to verify:** make a control missing on purpose; the probe must end with a named failure, not hang.
- **Session:** the VE-0/VE-1 build, commit `bec744b5`. **[reported by agent]** (the `_ve0_probe.gd` comment "a script error inside an `await` never returns" is the writer's own note, not a reproduction here).

## 23. `TextEdit.get_rect_at_line_column` answers for the grapheme BEFORE the column

- **Preflight trigger:** *Measure where a coloured run, a caret or a glyph starts in a `TextEdit`*
- **Root cause:** on Godot 4.7.2 `get_rect_at_line_column(line, col)` reports the rect of the grapheme before `col`, so column 1 returns column 0's x. A probe that asks it where a run starts is off by one glyph and passes or fails for the wrong reason.
- **Rule:** to find where drawn ink starts, use the hit test (`get_line_column_at_pos`) on the drawn pixels, or measure the pixels; never trust `get_rect_at_line_column` for a run's left edge.
- **How to verify:** ask for the rect at column 1 of a line whose first glyph is wide; it equals column 0's rect.
- **Session:** the VE-3 build, commit `2dd5159d`. **[reported by agent]**: recorded in `VAULT_EDITOR_PROPOSAL.md` s.7a (a); the probe `_ve3_probe.gd` uses the hit test instead.

## 24. The caret and the focus border are drawn in the same colour as a heading

- **Preflight trigger:** *Measure the ink of one colour in a capture of an editor that has focus*
- **Root cause:** the editor's caret and its focus border are drawn in the accent, which is also the heading colour. A pixel probe that counts "heading ink" on a focused editor counts the caret and the border and reports headings that are not there.
- **Rule:** park the caret on a line the measurement does not read, and start the scan inside the border; or measure with the editor unfocused. Never count a colour the chrome also uses without excluding the chrome.
- **How to verify:** move the caret onto the measured line; the count of heading-coloured pixels changes with no text change.
- **Session:** the VE-3 build, commit `2dd5159d`. **[reported by agent]**: recorded in `VAULT_EDITOR_PROPOSAL.md` s.7a (c); `_ve3_probe.gd` parks the caret and starts inside the border.

## 25. A rebuild frees the old editor AFTER the new note's state is assigned

- **Preflight trigger:** *Add code that reads a widget's text during a rebuild, a note switch or a close*
- **Root cause:** `vault_window.gd`'s `_open_editor` assigns the new `_browse_path` and `_browse_text` and only then calls `_rebuild`, which frees the old editor. For that moment the old note's `TextEdit` is alive under the NEW note's name. VE-4's first version synced the editor's text into `_browse_text` inside `_clear` and then autosaved: `Qconf1.md` came out holding `Qalpha.md`'s text. A user's note overwritten with another note's text is the worst failure an autosave can have.
- **Rule:** any code that reads the live editor must first check the editor was built for the note now in `_browse_path` (`_editor_path`), and a write must refuse when it was not. Reading "the current editor" is a claim about which note, not only about which widget.
- **How to verify:** open note B straight after editing note A without waiting; B's file must hold B's text. Remove the `_editor_path` checks and `_ve4_probe.gd` goes red (its Qconf1 and Qcrlf legs).
- **Session:** the VE-4 build. **[verified]**: caught by the probe on its first run, before any commit.

## 26. Two agents removed `C:/cw/.win_lock` without checking it was theirs

- **Preflight trigger:** *Take, wait for or release the shared lock for windowed probes (`C:/cw/.win_lock`)*
- **Root cause:** a windowed run takes the lock with `mkdir C:/cw/.win_lock`; an agent that found it present, or that was tidying up after a crashed run, removed it without asking whose it was. Both PC-4 and VE-4 did this (2026-11-08). The directory was another agent's live lock, so two windowed Godot runs shared one desktop and one `user://`, and the probe that lost the race read a window it did not own.
- **Rule:** write an owner file inside the lock directory that names you, wait while the lock exists, and remove the directory ONLY if its owner file is yours (`grep` for your own tag before `rm -rf`); release in a `finally`/`trap`; a hard timeout on every run so a crash cannot strand it. A lock whose owner is not you is never yours to remove, however old it looks: report it instead.
- **How to verify:** start a run while another holds the lock: it must wait. Kill a run mid-way: its `trap` removes the lock because the owner file names it, and a second agent's lock is left standing.
- **Session:** the VE-5 session (2026-11-08), recorded from the PC-4 and VE-4 reports. **[verified]**: the lock wrapper used for this session's probes (`ve5_run.sh`) takes it with `mkdir`, writes the owner file, and releases only when that file names this session.

## Suggested handling

- Entry 6 merges into an existing row; the rest are new triggers.
- Entries 20, 21, 25 and 26 are verified at a named place; entries 22, 23 and 24 carry **[reported by agent]**.
- Entries 1, 4, 5, 7, 8, 9 and 10 carry **[reported by agent]**: confirm each (one reproduction apiece) before adding, or add them with that tag visible.
- Entries 2 and 3 are partly backed by repository text, noted above.
