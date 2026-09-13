# Cartalith — session handoff prompt

Paste everything below the line into a new session working in
`C:\Users\Vincent\Cartalith_GDT`.

Written to stay true: it contains **no counts, no commit hash, no test total**.
Anything that moves is derived by a command in step 1.

---

## Your standing goal

**Keep working through the items on the outstanding work list. Verify each piece
of work done. Use at most 3 agents at a time. Every time a batch is done, update
the outstanding work file before starting the next. The goal is complete when the
whole list has been finished. GUI work still is the leading priority.**

*That is the goal as the owner last set it (2026-09-12). Its "3 agents" predates the
owner changing the builder count later that same day (to four, then five, then three) — **the count line
directly below governs**, not the goal text.*

**Three workers to build or fix, plus a verifier per batch** — the standing count. **On 2026-09-13 the owner
raised it twice, only to spend budget left before the weekly reset:** *"78% and 50min remaining spool up another
agent"* (four), then *"15% left with 25min before reset. I think there is room for another agent"* (five), each
started at once as a file-disjoint workflow beside the running batch. After the reset the standing instruction
applies: *"When the limit resets use 3 concurrent building agents"*. (Owner, 2026-09-12:
*"When the 5 finished scale back to 3"* — applied from the first dispatch after the
five concurrent builders of that evening finished, not by stopping them. Before it:
*"Use 1 extra agent, so we got a total of 5 concurrent workers"*, after
*"You can
use 2 extra agents. So 4 total to build or fix"* and *"As long as they don’t work
on items that might cause conflicts"* — raised from two build lanes). **The count
has now moved 3 → 2 → 4 → 2 → 3 → 2 → 4 → 2 → 4 → 5 → 3 → 4 → 5 → 3 across seven days, so read THIS LINE
rather than a batch’s precedent, and re-read it after every owner message** — it
has changed mid-session more than once. **Several builders can run as concurrent
batches, each with its own verifier** — and then the batches must be
**measurement-disjoint as well as file-disjoint**: a lane that moves a quantity
another running lane is measuring corrupts that lane’s evidence with no file
shared between them. Near a usage limit, go back to one workflow at a time.

**Lanes may run on a smaller model — the verifier should not.** The owner
allowed Sonnet lanes at Ultracode on 2026-09-08, for token burn, and on 2026-09-12
set the model policy as *"either opus or Sonnet 5 at Ultracode or Max effort"*. **Four batches
in, the finding is consistent and specific: the CODE a smaller lane writes has
been right every time; the PROOF has been the weak half.** The clearest case is
the units probe of that morning — its three checks tested a `mi` suffix, a `km`
suffix and inequality, **all three satisfied by relabelling**, so a mutant that
kept the kilometre value and appended `mi` passed every one. That is exactly the
defect the row existed for. The fix was to rebuild the expected NUMBER from the
converter at probe time.

**So write the proof requirement into the brief rather than hoping for it.** The
sentence that worked: *"mutate the code under test and prove the assertion
FAILS — a probe that passes its own mutant is a decoration."* The batch that
carried it came back with whole-frame md5s and a reverted-fix control, and its
only two refutations were about the precision of supporting claims rather than
about a fix.
**The verifier is never one of the lanes the count refers to, and is never the
thing cut.** It has found a real defect in **every** batch it has run, and a
defect in the brief itself in twenty-eight consecutive batches. Recent examples
of what only it caught: a probe that shipped asserting the very regression its
own commit introduced; a lane that measured a desktop `PopupMenu` and declared
the owner’s request impossible while the owner was holding a phone; and, twice,
**my own claim that nothing else was writing the tree.**

## Budget posture (owner, 2026-09-07 — standing guidance, not a snapshot)

*Written at 9 % of the weekly limit. The limit was then reached mid-batch on
2026-09-08 and reset on 2026-09-12; the rules below held throughout and do not
depend on that figure.* **A batch killed by the limit fails its agents at
dispatch**: check `git status` for partial writes, then relaunch with
`resumeFromRunId` — finished agents return cached and only the rest re-run.

**When the weekly budget is tight, lanes are the wrong first economy — cut the
work that does not need a lane at all.** Three moves, in the order they pay:

1. **Mechanical work fully specified at its symbols belongs in the MAIN LOOP,
   not in a lane.** A lane costs a full context to re-derive what a previous
   lane already wrote down. When a report says *"the conversions are mechanical
   and ready"* and names each site, apply them directly and run the existing
   probe as the check. **That is the single largest saving available here.**
2. **Prefer ONE lane on genuinely uncertain work over two on adjacent work.**
   Two lanes are worth it when the two jobs are independent enough that neither
   waits; they are waste when the second is a variation of the first.
3. **Never cut the verifier.** It has found a real defect in every batch it has
   run and a defect in the brief itself in twenty-eight consecutive batches.
   **A cheap batch nobody checked is not cheap** — `09ff8e2` shipped unverified
   when a session limit killed its verifier, and paying that debt cost a place
   in the NEXT batch's verifier brief.

**And never kill a verifier that is already running to save budget.** Its lanes
are already paid for; stopping it discards their output and leaves a debt that
costs more to discharge later than the verifier would have cost to finish.

**Report spend honestly when asked.** Recent batches have run roughly 400k–800k
subagent tokens each (measured 2026-09-08: 408k, 534k, 572k, 787k); that is the number that matters, not the lane count.

Keep lanes **file-disjoint**: assign by crate or by directory, and where two rows
want the same file, serialize them across batches rather than forbidding the edit
(forbidding it stranded corrections twice, and a concurrent read produced a false
claim once). **At four lanes the disjointness is the hard part, not the work** —
`godot-project/shell/`, `godot-project/` root (`map_overlay.gd`), and each crate
are separate territories; pair GDScript lanes with Rust lanes so the two
verification domains stay independent (`cargo test` cannot see a broken shell).
Batch 27 hit the cost of getting this wrong the cheap way: a lane finished its
work and could not write the **one line** that switched it on, because that line
lived in the other lane's file.

**Two deferred tasks are gated on "GUI work is done", and both are filed as
rows rather than kept in a head:** rebuild the APK and drop it on the D: drive
(recipe in memory `cartalith-apk-build-and-drop` — expect `--export-release` to
fail at signing, that is normal, and verify the `.so` inside the APK is the one
just built), and a menu-by-menu design-conformance audit using **Fable 5.1 at
Ultracode, minimum 2 agents** (owner, 2026-09-04).

**The design audit HAS RUN and is no longer the gate.** The owner supplied three
live canvases through the `claude_design` MCP on 2026-09-07 — imported verbatim
to `design/mcp-2026-09-07/` (PC, Tablet, Android, plus `support.js` and the
capture recipe in `CAPTURE.md`). **A `.dc.html` is a complete HTML document and
renders standalone in HEADED Chrome; the canvas editor never renders headless**
(five configurations were tried). `TABLET_UI_SPEC.md` and `ANDROID_UI_SPEC.md`
were written from them.

**What replaced it is the standing definition of done for GUI work, in the
owner’s words (2026-09-07):**

> *"All designs layouts and styles should match 100%. Check all designs, pc,
> tablet and phone. Check - build - check again all checks against visual
> information and code information from the Claude design instance."*

> *"As soon as we reach 100% parity with the reference for the result I’ll waive
> the 100% checks between before, mid and after."*

**So the before/mid/after discipline is scaffolding the owner has already agreed
to drop at parity — it is not the goal. The reference is.** Where the canvases
and a project document disagree, **the canvases win**. The standing example:
`DCC_SHELL_SPEC.md` §11 said *"Radius 0 everywhere"* until owner ruling
(*"The radius should follow the newest designs"*, 2026-09-07) superseded it, and
the sentence was **corrected at the source on 2026-09-08** (`f9dba6c`) — radius
follows `design/mcp-2026-09-07/`. **Where a document still disagrees with a
canvas, correct the document in the same change**, or the next port re-infects
itself from it.

**Defects the owner reports ON GLASS outrank every conformance row.** They have
been right every time and have found in minutes what full sessions of green
probes did not — see `MISTAKES.md`’s *"The probe could not see the class of thing
that was wrong"*. **Take them first, and diagnose before fixing**: "it renders",
"it can be operated" and "it can be FOUND" are three different measurements.

**The APK build keeps its original trigger** and does not start early — though
the owner has asked for an interim drop directly, twice; do it when asked and
say so rather than quietly deciding GUI work is finished.

**Commit per verified batch** (owner, 2026-09-03). One commit per batch, after its
verifier reports — not before. Two constraints follow and neither is optional:

- **Never commit while a verifier is running.** A clean tree makes `git diff` empty
  for *every* path, so each "the diff is empty" check silently stops being evidence
  rather than failing. This has already happened once here.
- **Explicit paths only** — never `git add -A` / `-a`, never `git commit -- <paths>`.
  A batch's lanes and the main loop edit the tree concurrently; a blanket add
  captures whatever a lane happened to be mid-write on.

**Every agent brief’s "standing rules" block must be DERIVED FROM `MISTAKES.md`,
not written from memory.** Owner instruction, 2026-09-07, after `MISTAKES.md`
went nine commits without an update while the rules were being retyped from
recall into each brief. Open the preflight table, take the rows that match what
the batch is about to do, and paste those. **A rule you remember but cannot find
in that table is a rule you have not filed yet — file it first.**

Set this with `/goal` so it persists across check-ins.

---

## Step 1 — establish the state before believing anything

Run these first. **Do not carry a number from this prompt, from a document, or
from an agent's report — every one of them has been wrong here.**

```bash
cd /c/Users/Vincent/Cartalith_GDT
git log --oneline -3 && git rev-parse --abbrev-ref HEAD
git status --short | grep -v "__pycache__\|^?? nul"     # real uncommitted work
cd cartalith-native && cargo test --workspace --no-fail-fast 2>&1 \
  | grep -E "^test result:" \
  | awk -F'[ ;]' '{p+=$4; f+=$6; i+=$8} END{print p" passed; "f" failed; "i" ignored"}'
```

Then the backlog count. **The arithmetic here is not delegated and not done by
hand** — this file's totals have disagreed with themselves four times. Write this
script and run it; every figure you publish comes from one run of it:

```python
# count_outstanding.py — run from the repo root
import collections, io, re, sys
HEADERS = {"Item","Question","#","Milestone","Claim",""}
rows, sizes, sec = collections.OrderedDict(), collections.Counter(), None
for line in io.open("OUTSTANDING_WORK.md", encoding="utf-8"):
    h = re.match(r"^(#{2,4})\s+(.*)", line)
    if h:
        t = h.group(2).strip()
        sec = t if re.match(r"^\d", t) else None
        if sec and sec not in rows: rows[sec] = 0
        continue
    if not (sec and line.startswith("|")): continue
    cells = [c.strip() for c in line.split("|")[1:-1]]
    if not cells or all(re.fullmatch(r"[:\-]{2,}", c) for c in cells): continue
    if cells[0] in HEADERS and rows[sec] == 0: continue
    rows[sec] += 1
    if sec.startswith(("1.","2.","3.")):
        for c in cells:
            if c in ("large","medium","small"): sizes[c] += 1; break
g = collections.Counter()
for k, v in rows.items(): g[k.split(".")[0].split(" ")[0]] += v
head = g["1"] + g["2"] + g["3"] + g["4"]
print(f"S1={g['1']} S2={g['2']} S3={g['3']} S4={g['4']}  HEADLINE={head}")
print(f"sizes {dict(sizes)} = {sum(sizes.values())}; headline-S4 = {head - g['4']}")
print("CONSISTENT" if sum(sizes.values()) == head - g["4"] else "*** MISMATCH ***")
```

Finally, check for work still running (`/workflows`, or the task list). A verifier
may be mid-flight; **its refutations are work, not noise.**

---

## Step 2 — read these, in order

1. **`MISTAKES.md`** (repo root) — **preemptive, not a log.** It opens with a
   preflight table keyed to what you are *about to do*: scan the left column,
   apply the matching rule before you start. Every entry is a mistake that
   shipped into this tree, and each `×N` is the best available predictor of what
   will go wrong next. **Put its rules in every agent brief** — a `CLAUDE.md`
   obligation, and these recur precisely because a fresh agent does not know
   them.
2. **`CLAUDE.md`** — auto-loads. Constraints, the three naming hazards, the
   routing table.
3. **`OUTSTANDING_WORK.md`** — the routed backlog. It is a **router, not state**:
   every row names the document that owns the work.
4. **`cartalith-native/docs/STATUS.md`** — the single source of truth for
   progress. Where it and the backlog disagree, STATUS wins.
5. **`LARGE_ITEM_RULINGS.md`** — every owner ruling. Read both sections.

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

One `Workflow` per batch: **its build lanes + 1 adversarial verifier.** The builder count is the line at the top of this prompt, not this sentence — it read "2 build lanes = 3 agents" until 2026-09-12.

- Every lane brief carries `MISTAKES.md`'s relevant preflight rows inline, the
  hard constraints, and the verification bar.
- The verifier is told to **try to refute**, to default to refuted when
  uncertain, **and to check the brief itself.** It has found a defect in the
  brief in three consecutive batches — a `git diff` check made vacuous by a
  mid-verification commit, a probe guard that could not discriminate, and a probe
  size at which the defect was arithmetically invisible.
- After the batch: verify claims **at their symbols yourself**, fix what the
  verifier found, update `OUTSTANDING_WORK.md`, then dispatch the next.

**Serialize lanes that share a file** rather than forbidding the edit —
forbidding strands corrections. Tell every lane to **report** false prose in
files it does not own.

---

## Non-negotiables

- **Never edit**: `reference/*.html` (both frozen snapshots), `.gitignore`,
  `project.godot`, any `Cargo.toml`. `export_presets.cfg` is off limits except
  one scoped owner authorisation (probe-scene `exclude_filter`).
- **Commit once per verified batch, after its verifier reports** (owner, 2026-09-03 —
  see *Commit per verified batch* above; until 2026-09-12 this line read *"Commit only
  when the owner asks"* and contradicted it). Explicit-path `git add` only — never
  `-A`/`-a`. Never `--force` on push. Branch, don't commit to `main`.
- **Declaring green needs both checks**: `cargo test --workspace` (never a crate
  subset) **and** `godot --headless --check-only --script` on every touched `.gd`
  plus `shell/app.gd`, graded only on stderr containing "Parse Error" /
  "Failed to load script" — the exit code is unreliable. A commit once reported
  2 821 passed while the application root would not compile.
- **A golden re-baseline needs an owner ruling.** Divergence ships `false` in
  `cartalith_engine::WorldParams::defaults()`, `true` in
  `cartalith_godot::params::defaults()`; new params need `PARAMS` + `JS_PATHS`
  rows. Prefer identity by **control flow** over identity by arithmetic.
- Do not write `OUTSTANDING_WORK.md`, `STATUS.md`, `MISTAKES.md`,
  `LARGE_ITEM_RULINGS.md` or `SESSION_HANDOFF.md` from a subagent — the main loop
  owns them — **and the main loop does not write any of them while a verifier is
  running**: verifiers fingerprint them, and an unexplained diff costs a finding’s worth
  of attention (MISTAKES ×4).

---

## Owner priority for GUI work

**0. Anything the owner reported on glass, first — ahead of all of these**
(2026-09-07, and it has outranked the list every time it has come up).

**Then the tablet interface, which has not been overhauled yet** (owner, 2026-09-12:
*"Starting with the gui points (as tablet interface hasn’t been overhauled yet)"*).
The tablet canvas is a different shell from the desktop one, not a scaled copy —
`TABLET_UI_SPEC.md` inventories it region by region, and the backlog’s own tablet
rows give the order: composition and the menu bar first, since those are what read
as "the PC layout".

Then the standing 2026-09-03 order:

1. Rows blocked on **other work** first.
2. Then the **unblocked** rows.
3. Then rows blocked on a **design that does not exist** — **note that this
   class has largely emptied**, since the three live canvases now cover PC,
   tablet and phone.
4. Rows blocked on the **owner** → ask, with `AskUserQuestion`.

## Live owner rulings (2026-09-03, full text in `LARGE_ITEM_RULINGS.md`)

- **DS-03 tablet: keep everything, reflow only.** Retires the content question;
  what remains is `DccTheme.TABLET`'s exhausted key space.
- **Right dock: selection wins, an armed tool *appends* a section.**
- **CV-24 / ED-02: both wait for a design pass.** Not startable.
- **APK probe scenes: excluded**, under a scoped `export_presets.cfg`
  `exclude_filter` authorisation and nothing else in that file.

**Open and worth asking:** owner question 8 — what should `statusMid`'s
`repaint NN ms` measure: frame time, texture-upload time, or `_refresh_map()`
wall time? Leave the field dashed with its reason until it is answered.

---

## Where the work is

Derive it rather than trusting this paragraph: read `OUTSTANDING_WORK.md` §2 for
what is startable, §3.1 for what is waiting on the owner, and
`design/dcc-environment-2026-08-31/spec/00-REPLACEMENT-PLAN.md` §3 for the shell
replacement's build order and which stages remain.

Two live sources of real defects, both productive every time they are opened:
`UNWIRED_FUNCTIONS.md` (a backlog with a `file:line` per row, plus a
dangerous-class section for controls disabled by reasons that have become false),
and §6 of `OUTSTANDING_WORK.md` (contradictions in the project record).
