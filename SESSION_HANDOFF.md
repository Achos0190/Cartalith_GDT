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
whole list has been finished.**

**Three build lanes, plus the verifier** (owner, 2026-09-05 — the current
instruction; the count has moved 3 -> 2 -> 4 -> 2 -> 3 over two days, so **read
this line rather than a batch's precedent**). Three concurrent build lanes, then
the verifier. The
adversarial verifier has found a real defect in every batch it has run,
including in the brief itself in sixteen consecutive batches, so when the budget
is two it is the **second** agent, not the one dropped.

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

**The design audit was re-ordered by the owner the same day: it runs BEFORE the
rest of the outstanding list, not after GUI work completes.** Its fixes are
**Opus 5 at Ultracode, with Fable 5.1 where the call is a design judgment**
rather than mechanical conformance — the model holding the design context
decides what conformance means, the other applies it. **The APK build keeps its
original trigger** and does not start early; say so rather than quietly deciding
GUI work is finished.

**Commit per verified batch** (owner, 2026-09-03). One commit per batch, after its
verifier reports — not before. Two constraints follow and neither is optional:

- **Never commit while a verifier is running.** A clean tree makes `git diff` empty
  for *every* path, so each "the diff is empty" check silently stops being evidence
  rather than failing. This has already happened once here.
- **Explicit paths only** — never `git add -A` / `-a`, never `git commit -- <paths>`.
  A batch's lanes and the main loop edit the tree concurrently; a blanket add
  captures whatever a lane happened to be mid-write on.

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

One `Workflow` per batch: **2 build lanes + 1 adversarial verifier = 3 agents.**

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
- **Commit only when the owner asks.** Explicit-path `git add` only — never
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
- Do not write `OUTSTANDING_WORK.md`, `STATUS.md` or `MISTAKES.md` from a
  subagent — the main loop owns them.

---

## Owner priority for GUI work (standing, 2026-09-03)

1. Rows blocked on **other work** first.
2. Then the **unblocked** rows.
3. Then rows blocked on a **design that does not exist**.
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
