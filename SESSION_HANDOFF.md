# Cartalith — session handoff prompt

Paste everything below the line into a new session working in
`C:\Users\Vincent\Cartalith_GDT`.

Written to stay true: it contains **no counts, no commit hash, no test total**.
Anything that moves is derived by a command in step 1. *(Refreshed 2026-10-19.
The previous version carried hashes, batch tallies and a 2026-09-13 resume note
that had been acted on; all were removed rather than updated.)*

---

## Your standing goal

**Keep working through the items on the outstanding work list
(`OUTSTANDING_WORK.md`). Verify each piece of work done. Every time a batch is
done, update `cartalith-native/docs/STATUS.md` (and the outstanding work file)
before starting the next. The goal is complete when the whole list has been
finished. GUI work is the leading priority; anything the owner reports on glass
comes first.**

*The goal text is the owner's (2026-09-12), with its agent count removed: the
count is set separately and changes often, so it lives in the next paragraph.*

**Agent cap: ONE agent at a time unless the owner says otherwise.** That is the
owner's latest instruction (given near a usage limit, 2026-10-17). The cap has
moved many times — up to five and down to one — so **re-read the owner's most
recent message about it before every dispatch**; the latest message always wins
over this file. When more than one agent is allowed, lanes must be
**file-disjoint and measurement-disjoint** (a lane that moves a quantity another
lane is measuring corrupts that evidence with no file shared), and the verifier
is never the thing cut.

**Model per task, by judgement (owner, 2026-09-26).** Sonnet at max effort for
tightly specified work with a named symbol and a clear done-means (doc and record
passes, string and comment fixes, probe fixes, small wiring that follows a
pattern, mutation sweeps with a given harness, re-runs). Opus for generation
changes that move goldens or need a re-baseline decision, cross-crate or
save-format design, merges, reading and interpreting the reference, and
adversarial verification of hard subsystems. When unsure, start on Sonnet with an
instruction to stop and report if the task needs design judgement.

**Whatever the model, write the proof requirement into the brief.** The code a
smaller lane writes has usually been right; the *proof* has been the weak half.
The sentence that works: *"mutate the code under test and prove the assertion
FAILS — a probe that passes its own mutant is a decoration."*

**Every agent brief's standing-rules block is DERIVED FROM `MISTAKES.md`'s
preflight table, not written from memory** (owner, 2026-09-07). Take the rows
that match what the lane is about to do and paste them. Also put in every brief:
the annotation rule (Ruling BK, `CLAUDE.md`), file searches stay inside the
Cartalith folder (owner, 2026-09-23), and "report false prose in files you do not
own".

Set this with `/goal` so it persists across check-ins.

---

## Commit and push protocol

- **One commit per verified batch, after its verifier reports — never while a
  verifier is running** (a clean tree makes every "the diff is empty" check
  silently vacuous).
- **Explicit paths only.** Never `git add -A` / `-a` / `.`, never
  `git commit -- <paths>`. **Never stage `MISTAKES.md` or
  `cartalith-native/godot-project/project.godot`** unless the owner asks for that
  exact file — the main tree routinely carries uncommitted edits to both. (A
  session that is told to *add a MISTAKES.md entry* does so in its own commit
  of that file alone.) `git show --stat` the commit and check every file the
  message names is in it.
- **Trailers**, at the end of every commit message:

  ```
  Co-Authored-By: Claude <model> <noreply@anthropic.com>
  Claude-Session: <this session's URL>
  ```

  Use the model that actually wrote the change and the session URL the harness
  gives you.
- **Push after every verified commit, never force** (no `--force`, no
  `--force-with-lease`). Rebase onto the remote first:

  ```bash
  git fetch origin
  git rebase origin/claude/cartalith-rust-godot-setup-lhgtgh
  for b in claude/cartalith-rust-godot-setup-lhgtgh claude/busy-archimedes-pd308f \
           claude/cartalith-sync-rc-6il498 main; do
    git push origin HEAD:$b
  done
  ```

  The four branches are kept at the same commit. If any push is rejected, stop
  and report; do not rewrite history.
- **Work in a worktree with a short path** (`C:/cw/<short-name>`), not under
  the repository's own long path — Windows path-length limits bite deep
  `target/` and `.godot/` trees (a standing session rule; the reason is a
  labelled judgement, not a recorded incident).

  ```bash
  git worktree add C:/cw/<name> -b <topic-branch> origin/claude/cartalith-rust-godot-setup-lhgtgh
  ```

  **Give each worktree its own `CARGO_TARGET_DIR`** (for example
  `C:/cw/<name>-target`), so two builds never share or clobber one target
  directory, and run heavy cargo work (the full workspace, Android) only when no
  other build is running — memory pressure has killed full runs here.
- **Clean up with literal paths only**: `rm -rf C:/cw/<name>` spelled out in
  full — never a glob, a variable or a path built by a script — then
  `git worktree prune`, and delete the topic branch once it is pushed under the
  four names above.

---

## Step 1 — establish the state before believing anything

Run these first. **Do not carry a number from this prompt, from a document, or
from an agent's report — every one of them has been wrong here.**

```bash
cd /c/Users/Vincent/Cartalith_GDT
git fetch origin
git log --oneline -5 && git rev-parse --abbrev-ref HEAD
git status --short | grep -v "__pycache__\|^?? nul"     # real uncommitted work
git worktree list                                      # leftovers from a killed session
git log --since="3 days ago" --format="%h %ad %s" --date=short
cd cartalith-native && CARGO_TARGET_DIR=<your target dir> \
  cargo test --workspace --no-fail-fast -j 2 2>&1 \
  | grep -E "^test result:" \
  | awk -F'[ ;]' '{p+=$4; f+=$6; i+=$8} END{print p" passed; "f" failed; "i" ignored"}'
```

Then the backlog count. **The arithmetic here is not delegated and not done by
hand** — totals in the record have disagreed with themselves repeatedly. Write
this script and run it; every figure you publish comes from one run of it:

```python
# count_outstanding.py — run from the repo root
# Open = a table row in §1–§4 whose first cell is NOT struck through.
# Closed rows stay in the file as evidence (`~~row~~ — **CLOSED …**`), so a
# count of every row is not the backlog.
import collections, io, re
HEADERS = {"Item","Question","#","Milestone","Claim",""}
STRUCK = re.compile(r"^(\*\*)?~~")
openr, closed, sizes, sec = collections.Counter(), collections.Counter(), collections.Counter(), None
for line in io.open("OUTSTANDING_WORK.md", encoding="utf-8"):
    h = re.match(r"^(#{2,4})\s+(.*)", line)
    if h:
        t = h.group(2).strip()
        sec = t.split(" ")[0] if re.match(r"^\d", t) else None
        continue
    if not (sec and line.startswith("|")): continue
    cells = [c.strip() for c in line.split("|")[1:-1]]
    if not cells or all(re.fullmatch(r"[:\-]{2,}", c) for c in cells) or cells[0] in HEADERS: continue
    top = sec.split(".")[0]
    if STRUCK.match(cells[0]):
        closed[top] += 1
        continue
    openr[top] += 1
    for c in cells:
        if c in ("large","medium","small"): sizes[c] += 1; break
print(f"OPEN   S1={openr['1']} S2={openr['2']} S3={openr['3']} S4={openr['4']}  HEADLINE={sum(openr[k] for k in '1234')}")
print(f"CLOSED S1={closed['1']} S2={closed['2']} S3={closed['3']} S4={closed['4']}  (kept as evidence)")
print(f"open rows by size (rows with a size cell only): {dict(sizes)}")
```

Finally, check for work still running (`/workflows`, or the task list). A verifier
may be mid-flight; **its refutations are work, not noise.**

---

## Step 2 — read these, in order

1. **`MISTAKES.md`** (repo root) — **preemptive, not a log.** Its preflight
   table is keyed to what you are *about to do*: scan the whole left column and
   apply each matching rule before you start. Each `×N` is the best available
   predictor of what will go wrong next.
2. **`CLAUDE.md`** — auto-loads. Constraints, the three naming hazards, the
   routing table.
3. **`cartalith-native/docs/STATUS.md`** — the single source of truth for
   progress. Read its *Orientation* and *Where to find things* first; the dated
   entries below them carry the measurements, and an entry a later commit
   changed carries a *superseded by* pointer.
4. **`OUTSTANDING_WORK.md`** — the routed backlog, a **router, not state**:
   every row names the document that owns the work. Where it and `STATUS.md`
   disagree, `STATUS.md` wins.
5. **`LARGE_ITEM_RULINGS.md`** — every owner ruling, dated, newest at the end.
   Cite a ruling rather than re-asking.

---

## Skills

Installed under `.claude/skills/`. Load per task.

| Skill | When |
|---|---|
| **`ponytail`** | Always. YAGNI ladder, shortest working diff. Default `full`. |
| **`cartalith-porting-discipline`** | Any port from the reference HTML — golden parity, crate placement. |
| **`cartalith-rust-conventions`** | Any Rust — match the original's precision, NaN policy, no panic across the gdext boundary. |
| **`godot-shell`** | Any `.gd` or scene work. Rust owns logic; Godot draws. |
| **`rust-craft`** | General Rust quality. |
| **`ui-ux-pro-max`** | **Any control, panel or visual change.** Search it before designing. |
| **`workflow-authoring`** | Before writing a Workflow script. |
| `godotprompter-*` | Export pipeline, GDExtension, mobile, multithreading — as needed. |

---

## The working method

A batch is **its build lanes + 1 adversarial verifier**, within the agent cap
above. With a cap of one, the build lane runs alone and the main loop verifies
inline — re-running the tests and probes itself, at the symbols — before it
commits.

- Every lane brief carries `MISTAKES.md`'s relevant preflight rows inline, the
  hard constraints, the verification bar, and the files it owns.
- The verifier is told to **try to refute**, to default to refuted when
  uncertain, **and to check the brief itself** — it has repeatedly found defects
  in the brief as well as in the work.
- After the batch: verify claims **at their symbols yourself**, fix what the
  verifier found, update `STATUS.md` and `OUTSTANDING_WORK.md`, commit, push,
  then dispatch the next.

**Serialize lanes that share a file** rather than forbidding the edit —
forbidding strands corrections. Tell every lane to **report** false prose in
files it does not own.

**Budget.** When the weekly budget is tight, cut work that does not need a lane
before cutting lanes: mechanical work already specified at its symbols belongs in
the main loop; prefer one lane on uncertain work over two on adjacent work;
**never cut or kill a verifier** (its lanes are already paid for). A batch killed
by the limit fails at dispatch — check `git status` for partial writes, then
relaunch.

---

## Non-negotiables

- **Never edit**: `reference/*.html` (both frozen snapshots), `.gitignore`,
  `project.godot`, any `Cargo.toml` without an owner ruling.
  `export_presets.cfg` is off limits except one scoped owner authorisation
  (probe-scene `exclude_filter`).
- **Declaring green needs both checks**: `cargo test --workspace` (never a crate
  subset) **and** `godot --headless --check-only --script` on every touched `.gd`
  plus `shell/app.gd`, graded on stderr containing "Parse Error" / "Failed to
  load script" — the exit code is unreliable. A visual change also needs its
  screenshots **opened and looked at, at every zoom it touches** (zoom gains
  detail is a hard requirement, owner 2026-10-17).
- **Probes are hermetic about settings.** Run each from a project copy with a
  unique `config/name` and a cleared user-data dir (Godot has no
  `--user-data-dir`), and a probe restores any persisted
  `cartalith_settings.cfg` setting it changes, at every exit path.
- **A golden re-baseline needs an owner ruling.** Divergence ships `false` in
  `cartalith_engine::WorldParams::defaults()`, `true` in
  `cartalith_godot::params::defaults()`; new params need `PARAMS` + `JS_PATHS`
  rows. Prefer identity by **control flow** over identity by arithmetic. A
  deliberate engine improvement away from the legacy HTML is correct
  (`DECISIONS.md` §7p): fix the new pipeline, scrub the superseded approach,
  do not dual-path it.
- `OUTSTANDING_WORK.md`, `STATUS.md`, `MISTAKES.md`, `LARGE_ITEM_RULINGS.md` and
  this file are **main-loop owned**: a subagent writes one only when its brief
  assigns it that file, and **nobody writes them while a verifier is running**
  (verifiers fingerprint them).

---

## Owner priority for GUI work

**0. Anything the owner reported on glass, first.** Diagnose before fixing:
"it renders", "it can be operated" and "it can be FOUND" are three different
measurements.

**Then GUI conformance.** Done means 100 % parity with the Claude Design
reference canvases (PC, tablet, phone — `design/mcp-2026-09-07/` and the newer
canvases; the newer one wins). The owner waives the before/mid/after checks at
parity. Where a project document disagrees with a canvas, correct the document
in the same change.

Then the standing order: rows blocked on other work first, then unblocked rows,
then rows blocked on a design that does not exist, then rows blocked on the
owner (ask, with `AskUserQuestion`).

**The APK** is rebuilt and dropped on the D: drive when the owner asks (recipe
in `GUI_GAP_REGISTER.md`; expect `--export-release` to fail at signing, and
verify the `.so` inside the APK is the one just built). Never install a stale
APK on the emulator — rebuild first.

---

## What was in flight, and what the owner still owes (2026-10-19)

Pointers only; the rows carry the detail and the current state.

- **In flight:** this file was refreshed while sibling documentation lanes
  were finishing before the owner switched the computer off; no build lane was
  known to be running. Check `git worktree list` and `git status` first — a
  killed session leaves worktrees under `C:/cw/` and partial writes. `OUTSTANDING_WORK.md` §1 is the
  in-flight list.
- **Owner questions open:** `OUTSTANDING_WORK.md` §3.1 (blocked on an owner
  decision) and §4, plus each scope document's owner-question section. Recent
  ones worth knowing are recorded in `STATUS.md`'s dated entries: confirming the
  colour-paint *layers* interpretation (Ruling CS asked for it against the
  build); the mountain-relief channel-fraction bar, whose proxy itself is in
  question (`MOUNTAIN_RELIEF_INVESTIGATION_2026-10-17.md`); and the volcano
  colour-and-form research request (`VOLCANO_APPEARANCE_NOTE_2026-10-19.md`).
- **Waiting on hardware:** 3D-0(c) and the CP-4 / Android folder-import checks
  need the OnePlus 6T on glass (`OUTSTANDING_WORK.md` §2.7 and §3.3).
- **Next candidates** are in `OUTSTANDING_WORK.md` §2; `STATUS.md`'s *Where to
  find things* table names the scope document for each recent subsystem (3D-2
  is next for the 3D viewport; CP-4…CP-7 for colour paint; the multi-sheet
  orogen belt for mountain ranges).

Two live sources of real defects, productive every time they are opened:
`UNWIRED_FUNCTIONS.md` (a backlog with a `file:line` per row, plus a
dangerous-class section) and §6 of `OUTSTANDING_WORK.md` (contradictions in the
project record).
