# Annotation-audit census (Ruling BK)

**2026-09-28. Findings only — a census, not an annotation pass.** This
document scopes the backlog row `OUTSTANDING_WORK.md` §2.8 ("Annotate the
existing code for a human auditor — Ruling BK") files against
`LARGE_ITEM_RULINGS.md`'s 2026-09-27 entry. It measures how much of the
existing tree (everything that predates Ruling BK; new code already follows
it) is missing the four things the ruling requires:

1. a doc comment on every function, module and type — what it does, why it
   exists, what it must never do;
2. a provenance line on every non-obvious constant;
3. a reason on every non-obvious branch or divergence;
4. a "protects" line on every test.

**Pending independent verification** — this is one lane's read of one run of
one heuristic script. See "Method's blind spots" before trusting a number, and
re-run the script (it is reproduced in full below) rather than copying a
figure from this table.

## Method

A Python line-scan script, not a parser (no `syn`, no GDScript grammar). It
counts:

- **LOC**: non-blank lines across every `.rs` file in the crate directory
  (`src/`, `tests/`, and any in-`src` fixture modules), or every `.gd` file
  under `godot-project/shell/`.
- **Undocumented items**: lines matching `fn`/`struct`/`enum`/`trait`/`mod`
  (optionally `pub`/`pub(crate)`/`unsafe`/`async`/`extern "C"`) whose nearest
  non-blank, non-attribute predecessor line is not a `///` or `//!` comment.
- **Unprotected tests**: `#[test]` functions whose body (scanned forward to
  its closing brace, capped at 4000 lines) contains no case-insensitive
  substring `"protect"`.
- **Under-provenanced consts**: `const NAME: T = <rhs>;` lines where the RHS
  contains a bare numeric literal and neither that line nor its immediate
  predecessor carries a `//` comment.
- **GDScript**: `func` declarations whose nearest non-blank predecessor is not
  a `#`/`##` comment line.

Script: `census.py` (reproduced verbatim below). Exact reproduction command,
run from the repository root:

```
mkdir -p /tmp/annot_census && cp census.py /tmp/annot_census/   # paste the script below into census.py first
python /tmp/annot_census/census.py "C:\Users\Vincent\Cartalith_GDT"
```

This lane ran it as (scratchpad path, session-specific — copy the script
below to reproduce):

```
python "<scratchpad>/annot_census/census.py" "C:\Users\Vincent\Cartalith_GDT"
```

Raw output captured 2026-09-28, one JSON object per line, is quoted in full in
"Raw output" below so a reader does not have to re-run it to see what this
document's table is built from.

## Results by crate (ranked highest-risk first)

Risk order follows the brief: golden-parity / engine crates first (the ones a
silent divergence in would move simulation output), then the two large
domain crates that also carry goldens, then the Godot-facing bridge crates,
then the GDScript shell. Ranking is a judgement call, not a metric the script
produces — stated so it can be disputed.

| Rank | Crate | LOC | Items (fn/struct/enum/trait/mod) | Undocumented | % undoc | Tests | Untagged "protects" | Consts w/ numeric | Unprovenanced |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | `cartalith-jsmath` | 2 065 | 57 | 22 | 39% | 17 | 17 | 69 | 68 |
| 2 | `cartalith-rng` | 225 | 10 | 6 | 60% | 4 | 4 | 0 | 0 |
| 3 | `cartalith-noise` | 1 209 | 37 | 17 | 46% | 16 | 16 | 0 | 0 |
| 4 | `cartalith-engine` | 14 344 | 520 | 233 | 45% | 221 | 221 | 73 | 51 |
| 5 | `cartalith-terrain` | 16 153 | 630 | 331 | 53% | 317 | 317 | 103 | 61 |
| 6 | `cartalith-hydrology` | 5 124 | 137 | 36 | 26% | 57 | 57 | 13 | 6 |
| 7 | `cartalith-erosion` | 4 371 | 162 | 72 | 44% | 80 | 80 | 21 | 15 |
| 8 | `cartalith-climate` | 3 789 | 148 | 81 | 55% | 65 | 65 | 20 | 15 |
| 9 | `cartalith-spatial` | 5 189 | 342 | 235 | 69% | 168 | 168 | 29 | 18 |
| 10 | `cartalith-civ` | 69 788 | 2 335 | 1 078 | 46% | 1 020 | 1 020 | 374 | 162 |
| 11 | `cartalith-urban` | 52 817 | 942 | 396 | 42% | 349 | 348 | 143 | 63 |
| 12 | `cartalith-godot` | 99 692 | 3 716 | 1 239 | 33% | 1 160 | 1 159 | 347 | 168 |
| 13 | `cartalith-io` | 8 703 | 390 | 228 | 58% | 182 | 180 | 17 | 8 |
| 14 | `cartalith-assets` | 11 950 | 601 | 323 | 54% | 262 | 262 | 13 | 6 |
| 15 | `cartalith-vault` | 7 621 | 439 | 210 | 48% | 129 | 129 | 11 | 1 |
| 16 | `cartalith-gpu` | 9 820 | 373 | 140 | 38% | 111 | 111 | 44 | 20 |
| 17 | `godot-project/shell` (GDScript) | 101 811 | 3 129 funcs | 1 167 | 37% | n/a | n/a | n/a | n/a |

**Rust total: 312 860 LOC, 10 839 items, 4 647 undocumented (43%); 4 158
tests, 4 154 with no "protects" substring found (99.9%); 1 277 consts with a
bare numeric literal on their declaration line, 662 with no adjacent
comment.**

**Shell (GDScript) total: 101 811 LOC, 3 129 `func`s, 1 167 with no preceding
`#`/`##` line (37%).**

### Reading the "unprotected tests" column

The near-100% figure across every crate is not a script bug — it is the
literal fact that this codebase's existing tests almost never spell the word
"protect" or "protects" in a nearby comment, which is exactly the gap Ruling
BK names. It does **not** mean the tests lack any comments at all (many do
have a one-line "why" above them); it means the specific "protects/fails if"
framing the ruling asks for is absent. Treat this column as "needs a protects
line added", not "needs a comment added from scratch".

## Raw output (2026-09-28, one run, unmodified)

```
{"crate": "cartalith-assets", "files": 23, "loc": 11950, "items_total": 601, "items_undoc": 323, "tests_total": 262, "tests_unprotected": 262, "consts_with_numeric": 13, "consts_undoc": 6}
{"crate": "cartalith-civ", "files": 58, "loc": 69788, "items_total": 2335, "items_undoc": 1078, "tests_total": 1020, "tests_unprotected": 1020, "consts_with_numeric": 374, "consts_undoc": 162}
{"crate": "cartalith-climate", "files": 16, "loc": 3789, "items_total": 148, "items_undoc": 81, "tests_total": 65, "tests_unprotected": 65, "consts_with_numeric": 20, "consts_undoc": 15}
{"crate": "cartalith-engine", "files": 34, "loc": 14344, "items_total": 520, "items_undoc": 233, "tests_total": 221, "tests_unprotected": 221, "consts_with_numeric": 73, "consts_undoc": 51}
{"crate": "cartalith-erosion", "files": 11, "loc": 4371, "items_total": 162, "items_undoc": 72, "tests_total": 80, "tests_unprotected": 80, "consts_with_numeric": 21, "consts_undoc": 15}
{"crate": "cartalith-godot", "files": 87, "loc": 99692, "items_total": 3716, "items_undoc": 1239, "tests_total": 1160, "tests_unprotected": 1159, "consts_with_numeric": 347, "consts_undoc": 168}
{"crate": "cartalith-gpu", "files": 10, "loc": 9820, "items_total": 373, "items_undoc": 140, "tests_total": 111, "tests_unprotected": 111, "consts_with_numeric": 44, "consts_undoc": 20}
{"crate": "cartalith-hydrology", "files": 6, "loc": 5124, "items_total": 137, "items_undoc": 36, "tests_total": 57, "tests_unprotected": 57, "consts_with_numeric": 13, "consts_undoc": 6}
{"crate": "cartalith-io", "files": 14, "loc": 8703, "items_total": 390, "items_undoc": 228, "tests_total": 182, "tests_unprotected": 180, "consts_with_numeric": 17, "consts_undoc": 8}
{"crate": "cartalith-jsmath", "files": 2, "loc": 2065, "items_total": 57, "items_undoc": 22, "tests_total": 17, "tests_unprotected": 17, "consts_with_numeric": 69, "consts_undoc": 68}
{"crate": "cartalith-noise", "files": 3, "loc": 1209, "items_total": 37, "items_undoc": 17, "tests_total": 16, "tests_unprotected": 16, "consts_with_numeric": 0, "consts_undoc": 0}
{"crate": "cartalith-rng", "files": 2, "loc": 225, "items_total": 10, "items_undoc": 6, "tests_total": 4, "tests_unprotected": 4, "consts_with_numeric": 0, "consts_undoc": 0}
{"crate": "cartalith-spatial", "files": 14, "loc": 5189, "items_total": 342, "items_undoc": 235, "tests_total": 168, "tests_unprotected": 168, "consts_with_numeric": 29, "consts_undoc": 18}
{"crate": "cartalith-terrain", "files": 34, "loc": 16153, "items_total": 630, "items_undoc": 331, "tests_total": 317, "tests_unprotected": 317, "consts_with_numeric": 103, "consts_undoc": 61}
{"crate": "cartalith-urban", "files": 58, "loc": 52817, "items_total": 942, "items_undoc": 396, "tests_total": 349, "tests_unprotected": 348, "consts_with_numeric": 143, "consts_undoc": 63}
{"crate": "cartalith-vault", "files": 10, "loc": 7621, "items_total": 439, "items_undoc": 210, "tests_total": 129, "tests_unprotected": 129, "consts_with_numeric": 11, "consts_undoc": 1}
TOTAL_RUST {"files": 382, "loc": 312860, "items_total": 10839, "items_undoc": 4647, "tests_total": 4158, "tests_unprotected": 4154, "consts_with_numeric": 1277, "consts_undoc": 662}
{"target": "godot-project/shell", "files": 58, "loc": 101811, "funcs_total": 3129, "funcs_undoc": 1167}
```

Sample verification (spot check, not exhaustive): `grep -n "^\s*const" cartalith-native/crates/cartalith-jsmath/src/libm.rs` shows the ported `fdlibm` magic constants (`O_THRESHOLD`, `LN2HI`, `P1`…`P5`, etc.) each on its own line with no per-constant provenance comment — confirming the crate's 68/69 "unprovenanced const" figure is a real finding, not a script artifact. The crate-level doc comment (if any) citing the fdlibm source does not count per the ruling's own wording ("every non-obvious constant says where its value comes from"), which is per-constant, not per-module.

## Method's blind spots (read before trusting a cell)

- **A doc comment that says nothing still counts as documented.** `/// TODO`
  or `/// see below` passes the script's check. This census cannot measure
  quality, only presence — the "documented" counts are an upper bound on how
  much of Ruling BK is actually satisfied, not the real figure.
- **Multi-line `const` declarations under-count numeric literals.** The
  script only inspects the text after `=` on the *declaration line itself*.
  A const like `const CASES: [(f64, f64); 8] = [` with its values on
  following lines is counted as "no numeric literal on this line" even
  though it plainly has one, and is excluded from both the numerator and
  denominator of the provenance column. `cartalith-jsmath::lib.rs`'s `CASES`,
  `SIN` and `COS` fixture tables are known instances. **The true
  under-provenanced-const count is higher than reported, in every crate that
  uses multi-line array/table constants** — this is a systematic
  undercount, not noise.
- **"Items" over-counts.** `impl` blocks are not scanned as items (the regex
  does not match `impl`), but every `fn` inside one *is* scanned as a
  standalone item, including trait-impl methods whose contract already lives
  on the trait declaration (arguably satisfying "why it exists" without a
  second doc comment on the impl). The census does not distinguish these
  from freestanding functions that truly lack any documentation.
- **Test fns are also counted in the general "items" column** (a `#[test] fn`
  matches the `fn` regex too), so the same function can appear in both the
  "undocumented items" and "unprotected tests" tallies. The two columns are
  not independent and should not be summed.
- **Macro-generated functions and constants are invisible.** Anything defined
  through a `macro_rules!` invocation or a derive that synthesizes methods
  is not scanned at all — the census only sees source text, not the expanded
  crate. `cargo doc` would catch some of this; the script does not.
- **The "protects" check is a substring match on a fixed brace-tracking
  window, not a scope-aware parser.** A comment mentioning "protects"
  anywhere between a `#[test]` line and its function's closing brace counts,
  even if it protects a *different*, adjacent test after a formatting quirk
  confuses the brace counter (capped at 4000 lines to avoid runaway scans on
  malformed input — no such cap was hit in this run, but it was not asserted
  against every file).
- **GDScript is only measured for `func`/comment adjacency**, not the other
  three Ruling BK requirements (constant provenance, branch reasons, or
  "protects" test comments) — GDScript has almost no `#[test]`-equivalent
  convention in this shell, and it uses `##` doc comments and plain `#` line
  comments interchangeably, both accepted here as "documented" without
  distinguishing them.
- **No `.tscn`, `.gdextension`, `Cargo.toml`, or build-script content is
  measured.** The census is source-code-body only.
- **This is one run.** No cross-check against `cargo doc --workspace` warnings
  (which would catch missing doc comments on `pub` items via `#[warn(missing_docs)]`
  if that lint were enabled — it is not, workspace-wide, as of this census) or a second
  independent implementation of the same heuristic.

## Ranked gaps and proposed batching order

Batches are sized 1 500-3 000 LOC (a single Sonnet lane, per the standing
build-lane convention) and ordered highest-risk first: golden-parity math and
simulation crates, then the two large domain crates, then the Godot-bridge
crates, then the GDScript shell. Splitting inside a crate follows its module
boundaries (`ls crates/<name>/src`), not a raw line cut, so a batch is a
coherent read for one lane.

**Batch sizing note:** consolidating small crates saves lane overhead;
splitting `cartalith-godot`, `cartalith-civ`, `cartalith-urban`, `cartalith-terrain`,
`cartalith-engine` and the GDScript shell by module keeps each batch inside
the 1 500-3 000 LOC band. Module names below are illustrative (drawn from
`ls`, not re-verified line count per module in this pass) — the lane that
takes a batch should re-run the census scoped to just its module directory
before starting, since a crate-wide percentage does not tell a module-level
lane what it will actually find.

| Order | Batch | Scope | Approx. LOC | Why this order |
|---|---|---|---|---|
| 1 | jsmath-math | `cartalith-jsmath` (whole crate) | 2 065 | Smallest, highest-leverage: every golden-parity crate calls into this one's `js_hypot`/`js_exp`/libm ports (`JS_SEMANTICS_AUDIT.md`). 68/69 consts unprovenanced despite being the crate where provenance matters most (ported magic numbers) |
| 2 | rng-noise | `cartalith-rng` + `cartalith-noise` | 1 434 | Foundational, small, no domain logic — cheap to close out and unblocks nothing being blocked on it |
| 3 | engine-core-1 | `cartalith-engine`, first half by module | ~2 800 | Core simulation entry point; 45% items undocumented, tied for highest const-numeric density outside jsmath |
| 4 | engine-core-2 | `cartalith-engine`, remaining modules | ~2 800 | Continuation of #3 |
| 5 | hydrology | `cartalith-hydrology` (whole crate) | 2 562 (split in 2 if a module boundary needs it) | Lowest undoc % (26%) of the core sim crates but still golden-parity; cheap to finish |
| 6 | erosion | `cartalith-erosion` (whole crate) | 2 186 (split in 2) | Golden-parity, GPU-adjacent (see `GPU_STREAM_POWER_SCOPE.md`) |
| 7 | climate | `cartalith-climate` (whole crate) | 1 895 (split in 2) | Golden-parity, smallest of the remaining core-sim crates |
| 8 | spatial-1/2 | `cartalith-spatial` | 2 595 each | Highest undoc % of any core crate (69%) — LOD/tiling base, worth prioritising before terrain depends further on it |
| 9 | terrain-1..6 | `cartalith-terrain` by module | ~2 700 each (6 batches) | Largest core-sim crate; 53% undoc |
| 10 | civ-1..25 | `cartalith-civ` by module | ~2 800 each (~25 batches) | Largest domain crate carrying its own goldens (economy, military manpower, religion diffusion) |
| 11 | urban-1..19 | `cartalith-urban` by module | ~2 800 each (~19 batches) | Second-largest domain crate, also golden-tested (fortify.rs mutation table etc.) |
| 12 | godot-1..35 | `cartalith-godot` by module (bridges, export, render, lod_worker, etc.) | ~2 800 each (~35 batches) | Bridge crate — largest of all, but a doc/comment-only pass here is lower risk than in a golden-parity crate since it has no reference to diverge from; still needed since it is the biggest single undocumented-item count (1 239) |
| 13 | io, assets, vault, gpu | remaining bridge/support crates | 8 703 / 11 950 / 7 621 / 9 820 (split each into 3-4) | Support crates, no golden parity risk |
| 14 | shell-1..35 | `godot-project/shell/*.gd` by window/panel file | ~2 900 each (~35 batches) | GDScript UI — presentation layer, changes here can't move simulation output, so it is last |

**First three batches to schedule, concretely:** (1) jsmath-math — all of
`cartalith-jsmath/src/lib.rs` and `libm.rs`, 2 065 LOC, fixing the 68
unprovenanced libm constants and 22 undocumented items; (2) rng-noise — both
crates together, 1 434 LOC, 23 undocumented items; (3) engine-core-1 — the
first half of `cartalith-engine` by module, ~2 800 LOC. All three are
doc-comment-and-provenance-only changes: no code changes, verified per the
existing backlog row's own bar (`cargo doc`/`--check-only` plus a sampled
human-readability review), each independently small enough for one lane
without touching the crates the concurrently-running export/render/lib.rs
lane owns.

## Full census script (`census.py`)

Paste this into a file and run `python <file> <repo_root>` to reproduce every
figure above.

```python
#!/usr/bin/env python3
"""
Ruling BK annotation-audit census.

Measures, per Rust crate (cartalith-native/crates/<name>) and for the
GDScript shell (cartalith-native/godot-project/shell), a heuristic count of:
  - lines of code (LOC): non-blank lines across the file set
  - fn/struct/enum/trait/mod items with no preceding /// or //! doc comment
  - #[test] fns with no comment containing "protect" in their vicinity
  - const items with a bare numeric literal and no adjacent // comment
  - (GDScript) func with no preceding # or ## comment

This is a regex/line-scan heuristic, not a parser. It is deliberately
conservative about false negatives (it will call something "documented" if
ANY comment-shaped line precedes it, even a non-explanatory one) -- see the
"blind spots" section this script's caller writes into ANNOTATION_AUDIT.md.

Usage: python census.py <repo_root>
Prints one JSON object per crate/target to stdout (one line each), then a
final line "TOTAL_RUST <json>".
"""
import sys, os, re, json, glob

def list_rs_files(root):
    out = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in ("target", ".git")]
        for f in filenames:
            if f.endswith(".rs"):
                out.append(os.path.join(dirpath, f))
    return sorted(out)

def list_gd_files(root):
    out = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in (".git",)]
        for f in filenames:
            if f.endswith(".gd"):
                out.append(os.path.join(dirpath, f))
    return sorted(out)

ITEM_RE = re.compile(
    r'^\s*(pub(\(crate\))?\s+)?(unsafe\s+)?(async\s+)?(extern\s+"[^"]*"\s+)?'
    r'(fn|struct|enum|trait|mod)\s+\w+'
)
TEST_ATTR_RE = re.compile(r'^\s*#\[test\]')
CONST_RE = re.compile(r'^\s*(pub(\(crate\))?\s+)?const\s+[A-Za-z_][A-Za-z0-9_]*\s*:.*=\s*(.*);?\s*$')
NUMERIC_RE = re.compile(r'(?<![A-Za-z_])-?\d[\d_]*(\.\d[\d_]*)?(e-?\d+)?(f32|f64|u8|u16|u32|u64|i8|i16|i32|i64|usize|isize)?(?![A-Za-z_])')
FUNC_RE = re.compile(r'^\s*func\s+\w+')

def is_doc_line(line):
    s = line.strip()
    return s.startswith('///') or s.startswith('//!')

def is_attr_line(line):
    s = line.strip()
    return s.startswith('#[') or s.startswith('#![')

def preceded_by_doc(lines, idx):
    """Walk upward from item at idx, skipping blank lines and attribute
    lines (derive macros etc.), and check whether the nearest real line
    is a /// or //! doc comment."""
    i = idx - 1
    while i >= 0:
        s = lines[i].strip()
        if s == '' or is_attr_line(lines[i]):
            i -= 1
            continue
        return is_doc_line(lines[i])
    return False

def preceded_by_any_comment(lines, idx, comment_prefixes):
    i = idx - 1
    while i >= 0:
        s = lines[i].strip()
        if s == '':
            i -= 1
            continue
        for p in comment_prefixes:
            if s.startswith(p):
                return True
        return False
    return False

def scan_rust_file(path):
    with open(path, 'r', encoding='utf-8', errors='replace') as f:
        text = f.read()
    lines = text.split('\n')
    loc = sum(1 for l in lines if l.strip() != '')

    items_total = 0
    items_undoc = 0
    tests_total = 0
    tests_unprotected = 0
    consts_with_numeric = 0
    consts_undoc = 0

    for idx, line in enumerate(lines):
        if ITEM_RE.match(line):
            items_total += 1
            if not preceded_by_doc(lines, idx):
                items_undoc += 1
        if TEST_ATTR_RE.match(line):
            tests_total += 1
            window = []
            j = idx
            fn_seen = False
            brace_depth = 0
            steps = 0
            while j < len(lines) and steps < 4000:
                window.append(lines[j])
                if re.match(r'^\s*(pub\s+)?(async\s+)?fn\s+\w+', lines[j]):
                    fn_seen = True
                if fn_seen:
                    brace_depth += lines[j].count('{') - lines[j].count('}')
                    if brace_depth <= 0 and '{' in ''.join(window):
                        break
                j += 1
                steps += 1
            body = '\n'.join(window)
            if 'protect' not in body.lower():
                tests_unprotected += 1
        m = CONST_RE.match(line)
        if m:
            rhs = m.group(3) if m.lastindex and m.lastindex >= 3 else ''
            has_numeric = bool(NUMERIC_RE.search(rhs)) if rhs else bool(NUMERIC_RE.search(line))
            has_inline_comment = '//' in line
            if has_numeric:
                consts_with_numeric += 1
                if not has_inline_comment and not preceded_by_any_comment(lines, idx, ('//',)):
                    consts_undoc += 1

    return dict(
        files=1, loc=loc,
        items_total=items_total, items_undoc=items_undoc,
        tests_total=tests_total, tests_unprotected=tests_unprotected,
        consts_with_numeric=consts_with_numeric, consts_undoc=consts_undoc,
    )

def scan_gd_file(path):
    with open(path, 'r', encoding='utf-8', errors='replace') as f:
        text = f.read()
    lines = text.split('\n')
    loc = sum(1 for l in lines if l.strip() != '')
    funcs_total = 0
    funcs_undoc = 0
    for idx, line in enumerate(lines):
        if FUNC_RE.match(line):
            funcs_total += 1
            if not preceded_by_any_comment(lines, idx, ('#',)):
                funcs_undoc += 1
    return dict(files=1, loc=loc, funcs_total=funcs_total, funcs_undoc=funcs_undoc)

def merge_rust(acc, r):
    for k in ('files','loc','items_total','items_undoc','tests_total','tests_unprotected','consts_with_numeric','consts_undoc'):
        acc[k] = acc.get(k, 0) + r[k]
    return acc

def merge_gd(acc, r):
    for k in ('files','loc','funcs_total','funcs_undoc'):
        acc[k] = acc.get(k, 0) + r[k]
    return acc

def main():
    repo = sys.argv[1]
    crates_root = os.path.join(repo, 'cartalith-native', 'crates')
    results = {}
    grand = {}
    for name in sorted(os.listdir(crates_root)):
        cdir = os.path.join(crates_root, name)
        if not os.path.isdir(cdir):
            continue
        acc = {}
        for f in list_rs_files(cdir):
            r = scan_rust_file(f)
            acc = merge_rust(acc, r)
        results[name] = acc
        grand = merge_rust(grand, acc)
    for name, acc in results.items():
        print(json.dumps({"crate": name, **acc}))
    print("TOTAL_RUST " + json.dumps(grand))

    shell_dir = os.path.join(repo, 'cartalith-native', 'godot-project', 'shell')
    gd_acc = {}
    for f in list_gd_files(shell_dir):
        r = scan_gd_file(f)
        gd_acc = merge_gd(gd_acc, r)
    print(json.dumps({"target": "godot-project/shell", **gd_acc}))

if __name__ == '__main__':
    main()
```
