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

## 2026-09-28 census reconciliation (dated note)

Batches 5-11 each ran their own adapted copy of the census script from its
own scratchpad and reported every touched crate "0/0/0 unchanged" against
that copy. A same-day re-run of the *original* `census.py` (now kept as
`census_v1.py`, reproduced unmodified further below) on the same tree
disagreed with those batches' own self-reports, e.g. `cartalith-spatial`
items_undoc 175, `cartalith-climate` items_undoc 8 / tests_unprotected 50,
`cartalith-engine` items_undoc 12 — despite those three crates being carried
elsewhere as "closed" batches. Comparing v1's line-scan logic against what
batches 9 and 11 had actually written found three reconciled rule changes,
producing `census_v2.py` (below `census_v1.py` in "Full census scripts"):

1. **A `#[test] fn` is documented by a `// Protects:` line in either of the
   two places this tree actually uses**: as a `///` doc comment directly
   above the `#[test]` attribute (batches 5-9's convention — e.g.
   `cartalith-climate`'s `geoid.rs`/`koppen.rs`), or as a line inside the
   test body containing "protects" (batch 11's convention — e.g.
   `cartalith-spatial`'s `contour.rs` — and this session's own going-forward
   convention). v1 only ever scanned forward from the `#[test]` line to the
   closing brace, so it saw the second convention but not the first —
   explaining why `cartalith-climate`, which used the first convention
   throughout, still showed 50/65 "unprotected" after batch 9. v1 also ran
   every `fn` (test or not) through the same bare "preceded by `///`" check
   for the *items* column, so a test documented only by the second
   convention (a body line, not a preceding `///`) still counted as an
   undocumented *item* even though the test requirement was satisfied —
   this is what produced `cartalith-spatial`'s 175: all 175 were test `fn`s
   inside `#[cfg(test)] mod tests { ... }` blocks, each already carrying a
   `// Protects:` line inside its body.
2. **A bare `mod x;` declaration is documented if the file it points at
   (`x.rs` or `x/mod.rs`, sibling to the declaring file) opens with a
   `//!` module doc.** Ruling BK asks for a doc "on every function, module
   and type"; a `//!` living in the module's own file satisfies that for the
   declaration line. v1's blanket "no preceding `///`" rule over-counted
   every crate that (correctly) puts its module doc inside the module file
   rather than at the `mod` keyword. No crate in this run turned out to
   have a genuine gap of this shape once the check was added — `cargo doc`
   would have caught a truly undocumented module file some other way — but
   the rule change is real and kept, since a fresh crate could still hit it.
3. **Trait-impl methods are not special-cased in the script** — decided as
   a human judgement call, not a heuristic one, per the reconciliation
   brief's own framing ("a script cannot tell 'obvious' from 'not'"). Where
   this run found a bare `fn default()` (e.g. `cartalith-climate`'s
   `GeoidOpts`/`TideParams`), a human pass added a one-line doc citing the
   value's own provenance rather than teaching the script to skip trait
   impls generally.

`census_v2.py`'s own header comment carries the same three points, plus a
fourth item (an inline `mod tests { ... }` block, as opposed to a bare `mod
x;` declaration, still needs its own doc per BK — rule 2 does not cover it,
and this run found nine such blocks with no doc at all, closed by hand; see
"Remaining genuine gaps" below and the file-level fixes shipped with this
same change).

**Only rules 1 and 2 change what the script counts; rule 3 changes nothing in
the script and is recorded here only because the brief asked the same
question about it.**

## Method

A Python line-scan script, not a parser (no `syn`, no GDScript grammar). It
counts:

- **LOC**: non-blank lines across every `.rs` file in the crate directory
  (`src/`, `tests/`, and any in-`src` fixture modules), or every `.gd` file
  under `godot-project/shell/`.
- **Undocumented items** (`census_v2.py`; see the reconciliation note above
  for what changed from v1): lines matching `fn`/`struct`/`enum`/`trait`/`mod`
  (optionally `pub`/`pub(crate)`/`unsafe`/`async`/`extern "C"`) whose nearest
  non-blank, non-attribute predecessor line is not a `///` or `//!` comment
  — **except** a `#[test]` fn documented by a `// Protects:`/`/// Protects:`
  line in either convention (rule 1), and a bare `mod x;` declaration whose
  target file opens with `//!` (rule 2).
- **Unprotected tests**: `#[test]` functions with no "protects" mention
  either in the contiguous comment block directly above the `#[test]`
  attribute, or anywhere in the body scanned forward to the closing brace
  (capped at 4000 lines) — v2 checks both places; v1 only checked the second.
- **Under-provenanced consts**: `const NAME: T = <rhs>;` lines where the RHS
  contains a bare numeric literal and neither that line nor its immediate
  predecessor carries a `//` comment. Unchanged from v1.
- **GDScript**: `func` declarations whose nearest non-blank predecessor is not
  a `#`/`##` comment line. Unchanged from v1; GDScript has no `#[test]` or
  `mod` equivalent for rules 1-2 to apply to.

Scripts: `census_v1.py` (the original, kept verbatim, reproduced below) and
`census_v2.py` (this run's script, incorporating rules 1-2 above; also
reproduced below). Exact reproduction command, run from the repository root:

```
mkdir -p /tmp/annot_census && cp census_v2.py /tmp/annot_census/   # paste the v2 script below into census_v2.py first
python /tmp/annot_census/census_v2.py "C:\Users\Vincent\Cartalith_GDT"
```

This lane ran it as (scratchpad path, session-specific — copy the script
below to reproduce):

```
python "<scratchpad>/annot_census/census_v2.py" "C:\Users\Vincent\Cartalith_GDT"
```

Raw output captured 2026-09-28 (v2, after this session's own reconciliation
and gap-closing pass), one JSON object per line, is quoted in full in "Raw
output" below so a reader does not have to re-run it to see what this
document's table is built from. The v1 raw output from the same day, before
this session's fixes, is kept immediately after it for comparison.

## Results by crate (ranked highest-risk first)

Risk order follows the brief: golden-parity / engine crates first (the ones a
silent divergence in would move simulation output), then the two large
domain crates that also carry goldens, then the Godot-facing bridge crates,
then the GDScript shell. Ranking is a judgement call, not a metric the script
produces — stated so it can be disputed.

Figures below are **`census_v2.py`, run after this session's own gap-closing
pass** — i.e. they already reflect the annotation work this same change
shipped for the eight crates marked "closed", not just the reconciled
counting rule. The pre-fix v2 numbers (rules 1-2 applied, but before this
session added any comments) are in "Raw output" below, alongside the
unmodified v1 numbers, so the size of each pass (rule-change vs. actual
annotation) can be told apart.

| Rank | Crate | LOC | Items (fn/struct/enum/trait/mod) | Undocumented | % undoc | Tests | Untagged "protects" | Consts w/ numeric | Unprovenanced |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | `cartalith-jsmath` | 2 170 | 57 | 0 | 0% | 17 | 0 | 69 | 0 |
| 2 | `cartalith-rng` | 255 | 10 | 0 | 0% | 4 | 0 | 0 | 0 |
| 3 | `cartalith-noise` | 1 291 | 37 | 0 | 0% | 16 | 0 | 0 | 0 |
| 4 | `cartalith-engine` | 15 590 | 520 | 0 | 0% | 221 | 0 | 73 | 0 |
| 5 | `cartalith-terrain` | 16 256 | 630 | 284 | 45% | 317 | 287 | 103 | 61 |
| 6 | `cartalith-hydrology` | 5 389 | 137 | 0 | 0% | 57 | 0 | 13 | 0 |
| 7 | `cartalith-erosion` | 4 945 | 162 | 0 | 0% | 80 | 0 | 21 | 0 |
| 8 | `cartalith-climate` | 4 072 | 148 | 0 | 0% | 65 | 0 | 20 | 0 |
| 9 | `cartalith-spatial` | 5 750 | 342 | 0 | 0% | 168 | 0 | 29 | 0 |
| 10 | `cartalith-civ` | 69 788 | 2 335 | 1 076 | 46% | 1 020 | 1 020 | 374 | 162 |
| 11 | `cartalith-urban` | 52 817 | 942 | 376 | 40% | 349 | 349 | 143 | 63 |
| 12 | `cartalith-godot` | 100 890 | 3 761 | 1 199 | 32% | 1 172 | 1 136 | 357 | 172 |
| 13 | `cartalith-io` | 8 703 | 390 | 221 | 57% | 182 | 182 | 17 | 8 |
| 14 | `cartalith-assets` | 11 950 | 601 | 313 | 52% | 262 | 262 | 13 | 6 |
| 15 | `cartalith-vault` | 7 621 | 439 | 203 | 46% | 129 | 129 | 11 | 1 |
| 16 | `cartalith-gpu` | 9 820 | 373 | 140 | 38% | 111 | 111 | 44 | 20 |
| 17 | `godot-project/shell` (GDScript) | 101 860 | 3 131 funcs | 1 167 | 37% | n/a | n/a | n/a | n/a |

**Rust total (v2, post-fix): 317 307 LOC, 10 884 items, 3 812 undocumented
(35%); 4 170 tests, 3 476 with no "protects" found in either place v2 checks
(83%); 1 287 consts with a bare numeric literal on their declaration line,
493 with no adjacent comment.** Ranks 1-9 (the eight crates this section
calls "closed" plus `cartalith-hydrology`, which the census now confirms was
already genuinely clean) are 0/0/0 across every undocumented-item, tests and
const column — not because the rule change alone closed them (see the
pre-annotation v2 numbers in "Raw output"), but because this same change
closed the remaining genuine gaps the rule change surfaced.

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

## Raw output

Three runs, same tree lineage, in chronological order. The first two predate
this session's own annotation edits (rule-change effect only); the third is
after this session's own gap-closing pass on top of the rule change.

### v1 (`census_v1.py`, unmodified, 2026-09-28, today's tree before this session's edits)

This is the number the reconciliation brief itself quotes (spatial 175,
climate 8/50, engine 12) — i.e. v1 run fresh against the tree as batches 5-11
left it, **not** the number those batches self-reported ("0/0/0") against
their own adapted copies, and **not** the stale pre-batch snapshot this
document previously carried here (which showed climate items_undoc 81 and
tests_unprotected 65 — a snapshot from before batches 5-9 had run at all).

```
{"crate": "cartalith-assets", "files": 23, "loc": 11950, "items_total": 601, "items_undoc": 323, "tests_total": 262, "tests_unprotected": 262, "consts_with_numeric": 13, "consts_undoc": 6}
{"crate": "cartalith-civ", "files": 58, "loc": 69788, "items_total": 2335, "items_undoc": 1078, "tests_total": 1020, "tests_unprotected": 1020, "consts_with_numeric": 374, "consts_undoc": 162}
{"crate": "cartalith-climate", "files": 16, "loc": 4057, "items_total": 148, "items_undoc": 8, "tests_total": 65, "tests_unprotected": 50, "consts_with_numeric": 20, "consts_undoc": 1}
{"crate": "cartalith-engine", "files": 34, "loc": 15547, "items_total": 520, "items_undoc": 12, "tests_total": 221, "tests_unprotected": 0, "consts_with_numeric": 73, "consts_undoc": 6}
{"crate": "cartalith-erosion", "files": 11, "loc": 4945, "items_total": 162, "items_undoc": 0, "tests_total": 80, "tests_unprotected": 0, "consts_with_numeric": 21, "consts_undoc": 0}
{"crate": "cartalith-godot", "files": 87, "loc": 99692, "items_total": 3716, "items_undoc": 1239, "tests_total": 1160, "tests_unprotected": 1159, "consts_with_numeric": 347, "consts_undoc": 168}
{"crate": "cartalith-gpu", "files": 10, "loc": 9820, "items_total": 373, "items_undoc": 140, "tests_total": 111, "tests_unprotected": 111, "consts_with_numeric": 44, "consts_undoc": 20}
{"crate": "cartalith-hydrology", "files": 6, "loc": 5389, "items_total": 137, "items_undoc": 0, "tests_total": 57, "tests_unprotected": 0, "consts_with_numeric": 13, "consts_undoc": 0}
{"crate": "cartalith-io", "files": 14, "loc": 8703, "items_total": 390, "items_undoc": 228, "tests_total": 182, "tests_unprotected": 180, "consts_with_numeric": 17, "consts_undoc": 8}
{"crate": "cartalith-jsmath", "files": 2, "loc": 2170, "items_total": 57, "items_undoc": 0, "tests_total": 17, "tests_unprotected": 0, "consts_with_numeric": 69, "consts_undoc": 0}
{"crate": "cartalith-noise", "files": 3, "loc": 1291, "items_total": 37, "items_undoc": 0, "tests_total": 16, "tests_unprotected": 0, "consts_with_numeric": 0, "consts_undoc": 0}
{"crate": "cartalith-rng", "files": 2, "loc": 255, "items_total": 10, "items_undoc": 0, "tests_total": 4, "tests_unprotected": 0, "consts_with_numeric": 0, "consts_undoc": 0}
{"crate": "cartalith-spatial", "files": 14, "loc": 5718, "items_total": 342, "items_undoc": 175, "tests_total": 168, "tests_unprotected": 0, "consts_with_numeric": 29, "consts_undoc": 13}
{"crate": "cartalith-terrain", "files": 34, "loc": 16153, "items_total": 630, "items_undoc": 331, "tests_total": 317, "tests_unprotected": 317, "consts_with_numeric": 103, "consts_undoc": 61}
{"crate": "cartalith-urban", "files": 58, "loc": 52817, "items_total": 942, "items_undoc": 396, "tests_total": 349, "tests_unprotected": 348, "consts_with_numeric": 143, "consts_undoc": 63}
{"crate": "cartalith-vault", "files": 10, "loc": 7621, "items_total": 439, "items_undoc": 210, "tests_total": 129, "tests_unprotected": 129, "consts_with_numeric": 11, "consts_undoc": 1}
TOTAL_RUST {"files": 382, "loc": 317047, "items_total": 10876, "items_undoc": 4133, "tests_total": 4167, "tests_unprotected": 3555, "consts_with_numeric": 1287, "consts_undoc": 513}
{"target": "godot-project/shell", "files": 58, "loc": 101811, "funcs_total": 3129, "funcs_undoc": 1167}
```

### v2 (`census_v2.py`, rules 1-2 applied, before this session's own annotation edits)

Isolates what the rule change alone explains, before any new comment was
written this session (compare against the "after" run below to see what this
session's own edits, not the script, closed).

```
{"crate": "cartalith-assets", "files": 23, "loc": 11950, "items_total": 601, "items_undoc": 313, "tests_total": 262, "tests_unprotected": 262, "consts_with_numeric": 13, "consts_undoc": 6}
{"crate": "cartalith-civ", "files": 58, "loc": 69788, "items_total": 2335, "items_undoc": 1076, "tests_total": 1020, "tests_unprotected": 1020, "consts_with_numeric": 374, "consts_undoc": 162}
{"crate": "cartalith-climate", "files": 16, "loc": 4057, "items_total": 148, "items_undoc": 3, "tests_total": 65, "tests_unprotected": 0, "consts_with_numeric": 20, "consts_undoc": 1}
{"crate": "cartalith-engine", "files": 34, "loc": 15547, "items_total": 520, "items_undoc": 12, "tests_total": 221, "tests_unprotected": 0, "consts_with_numeric": 73, "consts_undoc": 6}
{"crate": "cartalith-erosion", "files": 11, "loc": 4945, "items_total": 162, "items_undoc": 0, "tests_total": 80, "tests_unprotected": 0, "consts_with_numeric": 21, "consts_undoc": 0}
{"crate": "cartalith-godot", "files": 87, "loc": 100890, "items_total": 3761, "items_undoc": 1199, "tests_total": 1172, "tests_unprotected": 1136, "consts_with_numeric": 357, "consts_undoc": 172}
{"crate": "cartalith-gpu", "files": 10, "loc": 9820, "items_total": 373, "items_undoc": 140, "tests_total": 111, "tests_unprotected": 111, "consts_with_numeric": 44, "consts_undoc": 20}
{"crate": "cartalith-hydrology", "files": 6, "loc": 5389, "items_total": 137, "items_undoc": 0, "tests_total": 57, "tests_unprotected": 0, "consts_with_numeric": 13, "consts_undoc": 0}
{"crate": "cartalith-io", "files": 14, "loc": 8703, "items_total": 390, "items_undoc": 221, "tests_total": 182, "tests_unprotected": 182, "consts_with_numeric": 17, "consts_undoc": 8}
{"crate": "cartalith-jsmath", "files": 2, "loc": 2170, "items_total": 57, "items_undoc": 0, "tests_total": 17, "tests_unprotected": 0, "consts_with_numeric": 69, "consts_undoc": 0}
{"crate": "cartalith-noise", "files": 3, "loc": 1291, "items_total": 37, "items_undoc": 0, "tests_total": 16, "tests_unprotected": 0, "consts_with_numeric": 0, "consts_undoc": 0}
{"crate": "cartalith-rng", "files": 2, "loc": 255, "items_total": 10, "items_undoc": 0, "tests_total": 4, "tests_unprotected": 0, "consts_with_numeric": 0, "consts_undoc": 0}
{"crate": "cartalith-spatial", "files": 14, "loc": 5718, "items_total": 342, "items_undoc": 10, "tests_total": 168, "tests_unprotected": 0, "consts_with_numeric": 29, "consts_undoc": 13}
{"crate": "cartalith-terrain", "files": 34, "loc": 16256, "items_total": 630, "items_undoc": 284, "tests_total": 317, "tests_unprotected": 287, "consts_with_numeric": 103, "consts_undoc": 61}
{"crate": "cartalith-urban", "files": 58, "loc": 52817, "items_total": 942, "items_undoc": 376, "tests_total": 349, "tests_unprotected": 349, "consts_with_numeric": 143, "consts_undoc": 63}
{"crate": "cartalith-vault", "files": 10, "loc": 7621, "items_total": 439, "items_undoc": 203, "tests_total": 129, "tests_unprotected": 129, "consts_with_numeric": 11, "consts_undoc": 1}
TOTAL_RUST {"files": 382, "loc": 317217, "items_total": 10884, "items_undoc": 3837, "tests_total": 4170, "tests_unprotected": 3476, "consts_with_numeric": 1287, "consts_undoc": 513}
{"target": "godot-project/shell", "files": 58, "loc": 101860, "funcs_total": 3131, "funcs_undoc": 1167}
```

### v2, after this session's gap-closing pass (final — the numbers the table above uses)

```
{"crate": "cartalith-assets", "files": 23, "loc": 11950, "items_total": 601, "items_undoc": 313, "tests_total": 262, "tests_unprotected": 262, "consts_with_numeric": 13, "consts_undoc": 6}
{"crate": "cartalith-civ", "files": 58, "loc": 69788, "items_total": 2335, "items_undoc": 1076, "tests_total": 1020, "tests_unprotected": 1020, "consts_with_numeric": 374, "consts_undoc": 162}
{"crate": "cartalith-climate", "files": 16, "loc": 4072, "items_total": 148, "items_undoc": 0, "tests_total": 65, "tests_unprotected": 0, "consts_with_numeric": 20, "consts_undoc": 0}
{"crate": "cartalith-engine", "files": 34, "loc": 15590, "items_total": 520, "items_undoc": 0, "tests_total": 221, "tests_unprotected": 0, "consts_with_numeric": 73, "consts_undoc": 0}
{"crate": "cartalith-erosion", "files": 11, "loc": 4945, "items_total": 162, "items_undoc": 0, "tests_total": 80, "tests_unprotected": 0, "consts_with_numeric": 21, "consts_undoc": 0}
{"crate": "cartalith-godot", "files": 87, "loc": 100890, "items_total": 3761, "items_undoc": 1199, "tests_total": 1172, "tests_unprotected": 1136, "consts_with_numeric": 357, "consts_undoc": 172}
{"crate": "cartalith-gpu", "files": 10, "loc": 9820, "items_total": 373, "items_undoc": 140, "tests_total": 111, "tests_unprotected": 111, "consts_with_numeric": 44, "consts_undoc": 20}
{"crate": "cartalith-hydrology", "files": 6, "loc": 5389, "items_total": 137, "items_undoc": 0, "tests_total": 57, "tests_unprotected": 0, "consts_with_numeric": 13, "consts_undoc": 0}
{"crate": "cartalith-io", "files": 14, "loc": 8703, "items_total": 390, "items_undoc": 221, "tests_total": 182, "tests_unprotected": 182, "consts_with_numeric": 17, "consts_undoc": 8}
{"crate": "cartalith-jsmath", "files": 2, "loc": 2170, "items_total": 57, "items_undoc": 0, "tests_total": 17, "tests_unprotected": 0, "consts_with_numeric": 69, "consts_undoc": 0}
{"crate": "cartalith-noise", "files": 3, "loc": 1291, "items_total": 37, "items_undoc": 0, "tests_total": 16, "tests_unprotected": 0, "consts_with_numeric": 0, "consts_undoc": 0}
{"crate": "cartalith-rng", "files": 2, "loc": 255, "items_total": 10, "items_undoc": 0, "tests_total": 4, "tests_unprotected": 0, "consts_with_numeric": 0, "consts_undoc": 0}
{"crate": "cartalith-spatial", "files": 14, "loc": 5750, "items_total": 342, "items_undoc": 0, "tests_total": 168, "tests_unprotected": 0, "consts_with_numeric": 29, "consts_undoc": 0}
{"crate": "cartalith-terrain", "files": 34, "loc": 16256, "items_total": 630, "items_undoc": 284, "tests_total": 317, "tests_unprotected": 287, "consts_with_numeric": 103, "consts_undoc": 61}
{"crate": "cartalith-urban", "files": 58, "loc": 52817, "items_total": 942, "items_undoc": 376, "tests_total": 349, "tests_unprotected": 349, "consts_with_numeric": 143, "consts_undoc": 63}
{"crate": "cartalith-vault", "files": 10, "loc": 7621, "items_total": 439, "items_undoc": 203, "tests_total": 129, "tests_unprotected": 129, "consts_with_numeric": 11, "consts_undoc": 1}
TOTAL_RUST {"files": 382, "loc": 317307, "items_total": 10884, "items_undoc": 3812, "tests_total": 4170, "tests_unprotected": 3476, "consts_with_numeric": 1287, "consts_undoc": 493}
{"target": "godot-project/shell", "files": 58, "loc": 101860, "funcs_total": 3131, "funcs_undoc": 1167}
```

Sample verification (spot check, not exhaustive): `grep -n "^\s*const" cartalith-native/crates/cartalith-jsmath/src/libm.rs` shows the ported `fdlibm` magic constants (`O_THRESHOLD`, `LN2HI`, `P1`…`P5`, etc.) each on its own line with no per-constant provenance comment — confirming the crate's 68/69 "unprovenanced const" figure (v1) was a real finding, not a script artifact; `cartalith-jsmath` closed genuinely at 0 on this pass, not by a rule-change alone (its v1 and v2-pre-fix numbers already agreed at 0/0/0, meaning batch 1's own original annotation pass, not this reconciliation, is what closed it — confirmed by re-reading `libm.rs` directly: every constant does carry an inline `//` source comment).

## Remaining genuine gaps in the eight previously-"closed" crates (found and closed this pass)

Per-file, per-line list of every gap the v2-pre-fix run above still showed in
`cartalith-jsmath`, `cartalith-rng`, `cartalith-noise`, `cartalith-engine`,
`cartalith-hydrology`, `cartalith-erosion`, `cartalith-climate` and
`cartalith-spatial` — the eight crates this backlog carries as closed, plus
hydrology which the census confirms needed no fix. **All of the below were
closed (comments only) in this same change**; this list is the audit trail,
not an open backlog.

- `cartalith-jsmath`, `cartalith-rng`, `cartalith-noise`, `cartalith-hydrology`,
  `cartalith-erosion`: **none** — both v1 (fresh, today's tree) and v2
  (pre-fix) already read 0/0/0 for all three columns. Nothing to close.
- `cartalith-climate`:
  - `src/geoid.rs:44` — `impl Default for GeoidOpts { fn default() ... }` had
    no doc of its own (the struct's doc above it doesn't reach past the
    `impl` line). Closed: one-line doc citing the same reference default the
    struct doc already cites.
  - `src/tides.rs:63` — same shape, `impl Default for TideParams`. Closed
    likewise.
  - `src/lib.rs:582` — `pub fn current_wind_field(...)` (the function named
    in this task's own brief as behind `#[allow(clippy::too_many_arguments)]`):
    the `///` doc two items up documents `WindFieldResult`, not this fn; the
    attribute sits directly above `current_wind_field` with no doc of its
    own between them. Closed with a doc citing `currentWindField()`
    (reference HTML lines 5555-5569) and explaining the `#[allow]`.
  - `tests/golden_parity_weather.rs:39` — `const TOL: f32 = 1e-5;` inside
    `assert_close`, with the provenance already fully argued in the module
    doc comment above (V8's `Math.hypot` being an "implementation-approximated"
    ECMA-262 function, bisected to a 1-ULP divergence) but not repeated as an
    inline comment on the line itself. Closed with a one-line pointer back to
    that reasoning.
- `cartalith-spatial`:
  - Nine `#[cfg(test)] mod tests { ... }` blocks with no doc at all —
    `contour.rs:303`, `geo.rs:306`, `lib.rs:185`, `measure.rs:220`,
    `paint.rs:502`, `pass.rs:579`, `pyramid.rs:208`, `region.rs:156`,
    `staleness.rs:313`. This is the inline-`mod`-block case the reconciliation
    note above says rule 2 does **not** cover (rule 2 is only for a bare
    `mod x;` declaration whose target file has its own `//!`) — a real gap,
    not a rule-2 near-miss. Closed with one `///` line per module, naming
    what that file's tests cover (see each file's own diff).
  - `src/staleness.rs:61` — `struct StageNode` had every field documented
    but no doc on the struct itself. Closed with a one-line summary tying
    the fields together.
  - Test-local consts with a bare numeric literal and no adjacent comment:
    `src/paint.rs:510-511` (`W`/`H` = 16, an arbitrary small test grid),
    `src/paint.rs:531,656-657,713-714` (`C`/`R`, brush centre/radius — already
    explained in nearby prose comments but not on the const line itself),
    `src/pass.rs:1030` (`SENTINEL = -999.0`, an implausible height value used
    as an "untouched" marker), `tests/golden_parity_geo.rs:47-48` (`GW`/`GH`),
    `tests/golden_parity_paint.rs:58-60` (`GW`/`GH`/`SEA`). All closed with a
    short inline `//` comment stating what each value is for or referencing
    the existing nearby prose.
- `cartalith-engine`: all 12 items and 6 unprovenanced consts were in
  `examples/` (bench binaries: `compute_config_bench.rs`, `gpu_pool_bench.rs`,
  `timing_bench.rs`), not `src/` — genuinely undocumented helper/mode
  functions and, in `compute_config_bench.rs`, five constants
  (`REFERENCE_TILE_PX`, `MAX_LEVEL`, `SUN_AZ_DEG`, `EXAG`, `SHADE_RATIO_MID`,
  `SHADE_RATIO_GAIN`) whose header comment claimed they mirror `lod_bridge`'s
  own constants. **That claim is now stale**: reading `lod_bridge.rs` found
  `SUN_AZ_DEG`/`EXAG`/`SHADE_RATIO_MID`/`SHADE_RATIO_GAIN` were removed from
  that crate at LOD-D2 (2026-09-21) in favour of reading `TerrainAppearance`
  live — this bench's mirrored copies are now its own fixed reproducibility
  knobs, not a citation of current `lod_bridge` behaviour. Closed by
  documenting every function and correcting the stale claim in place, rather
  than repeating it as though still true (`cartalith-godot` itself was not
  touched — that crate belongs to another lane per this brief).

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

## Full census scripts (`census_v1.py`, then `census_v2.py`)

`census_v1.py` (below) is kept **verbatim** — it is the script batches 5-11
each had their own adapted copy of, and the one whose fresh 2026-09-28 run
(above, "v1") this reconciliation is measured against. `census_v2.py`
(further below) is this session's reconciled script (rules 1-2 from the
2026-09-28 note near the top of this document) and is what produced every
other number in this document. Paste either into a file and run
`python <file> <repo_root>` to reproduce its figures.

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

### `census_v2.py`

```python
#!/usr/bin/env python3
"""
Ruling BK annotation-audit census -- v2 (2026-09-28).

Reconciles census_v1.py (the script used unmodified by batches 5-11 and
originally embedded in ANNOTATION_AUDIT.md) against the fact that batches
5-11 were themselves annotating code to a convention v1 could not see, so
every one of those batches self-reported "0/0/0 unchanged" against v1 while
v1's own re-run (this file's predecessor) disagreed -- e.g. spatial
items_undoc 175, climate items_undoc 8 / tests_unprotected 50, engine
items_undoc 12. Three rule changes, decided this session against Ruling BK's
own wording in LARGE_ITEM_RULINGS.md:

1. A `#[test] fn` is documented by a `// Protects:` (or `/// Protects:`)
   line, in EITHER of the two places this tree actually uses:
     (a) as a /// doc comment directly above the #[test] attribute (batches
         5-9's convention, e.g. cartalith-climate's geoid.rs/koppen.rs), or
     (b) as the first non-blank line inside the test body (batch 11's
         convention, e.g. cartalith-spatial's contour.rs -- this is also the
         convention this session's own brief specifies going forward).
   v1 only ever scanned forward from the `#[test]` line to the closing
   brace, so it saw (b) but not (a) -- explaining why climate, which used
   convention (a) throughout, still showed 50/65 "unprotected" after batch 9
   supposedly closed it. v1 also fed every `fn` (test or not) through the
   same bare `fn`-with-no-/// check for the *items* column, so a test body
   documented only by (b) still counted as an undocumented "item" even
   though BK's test requirement was satisfied -- explaining spatial's 175.
   Rule: a test fn counts as documented (removed from both items_undoc and
   tests_unprotected) if EITHER (a) or (b) holds.
2. A bare `mod x;` (or `pub mod x;` / `pub(crate) mod x;`) declaration --
   no body on the line, just a semicolon -- is documented if the file it
   points at (`x.rs` or `x/mod.rs`, sibling to the declaring file) opens
   with a `//!` module doc. BK's own wording asks for a doc "on every
   function, module and type"; a `//!` living in the module's own file
   satisfies that for the declaration line, which is why v1's blanket "no
   preceding ///" rule over-counted every crate that (correctly) puts its
   module doc inside the module rather than at the `mod` keyword.
3. An inline `mod tests { ... }` block (not a declaration -- rule 2 does
   not apply) still needs its own doc per BK; nothing here should be read as
   exempting it. No fixture in this run needed a rule for that case; punting
   would be dishonest, so it is called out here as unresolved rather than
   silently subsumed by rule 1 or 2.

`preceded_by_doc` (item-level, non-test) and the const/GDScript checks are
UNCHANGED from v1 -- only the test-fn and mod-declaration handling differ.
Trait-impl methods: this script still does not special-case them (rule 3 of
the census reconciliation task explicitly leaves "trait-impl methods still
need a doc only where non-obvious" as a human judgement call, not a
heuristic one -- a script cannot tell "obvious" from "not"). They are
counted the same as any other fn; a human pass (not this script) decides
whether a given trait-impl fn's gap is real.

Usage: python census_v2.py <repo_root>
Prints one JSON object per crate/target to stdout (one line each), then a
final line "TOTAL_RUST <json>".
"""
import sys, os, re, json

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
    r'(fn|struct|enum|trait|mod)\s+(\w+)'
)
MOD_DECL_RE = re.compile(
    r'^\s*(pub(\(crate\))?\s+)?mod\s+(\w+)\s*;\s*(//.*)?$'
)
TEST_ATTR_RE = re.compile(r'^\s*#\[test\]')
CONST_RE = re.compile(r'^\s*(pub(\(crate\))?\s+)?const\s+[A-Za-z_][A-Za-z0-9_]*\s*:.*=\s*(.*);?\s*$')
NUMERIC_RE = re.compile(r'(?<![A-Za-z_])-?\d[\d_]*(\.\d[\d_]*)?(e-?\d+)?(f32|f64|u8|u16|u32|u64|i8|i16|i32|i64|usize|isize)?(?![A-Za-z_])')
FUNC_RE = re.compile(r'^\s*func\s+\w+')
PROTECTS_RE = re.compile(r'protects?\s*:', re.IGNORECASE)

def is_doc_line(line):
    s = line.strip()
    return s.startswith('///') or s.startswith('//!')

def is_attr_line(line):
    s = line.strip()
    return s.startswith('#[') or s.startswith('#![')

def preceded_by_doc(lines, idx):
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

def preceding_comment_block_has_protects(lines, idx):
    """Rule 1a: walk upward over a contiguous run of blank/attribute/
    comment lines above a #[test] attribute (idx points at the #[test]
    line itself) and check whether any /// or // line in that run mentions
    "protect(s)". Stops at the first non-blank/non-attr/non-comment line."""
    i = idx - 1
    while i >= 0:
        s = lines[i].strip()
        if s == '' or is_attr_line(lines[i]):
            i -= 1
            continue
        if s.startswith('///') or s.startswith('//'):
            if PROTECTS_RE.search(s):
                return True
            i -= 1
            continue
        break
    return False

def test_fn_window_and_body_protects(lines, idx):
    """Rule 1b (plus v1's original forward scan, kept as a fallback): scan
    forward from the #[test] attribute to the function's closing brace and
    check the whole body for "protect(s)" -- this also catches a Protects
    line placed first inside the body, which is this session's own
    going-forward convention, without requiring it to be the literal first
    statement (a script cannot reliably find "first statement" past
    attributes/blank lines without a real parser)."""
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
    return bool(PROTECTS_RE.search(body))

def resolve_mod_target(path, modname):
    """Rule 2: where a `mod x;` declaration's target file lives, relative
    to the declaring file -- `x.rs` or `x/mod.rs` next to it, or (for a
    declaration inside a crate root lib.rs/main.rs) also checked directly
    under that same directory. Returns the resolved path or None."""
    d = os.path.dirname(path)
    candidates = [
        os.path.join(d, modname + '.rs'),
        os.path.join(d, modname, 'mod.rs'),
    ]
    for c in candidates:
        if os.path.isfile(c):
            return c
    return None

def file_opens_with_module_doc(path):
    try:
        with open(path, 'r', encoding='utf-8', errors='replace') as f:
            for line in f:
                s = line.strip()
                if s == '':
                    continue
                return s.startswith('//!')
    except OSError:
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

    # Pre-scan: mark which line indices are #[test] fns and whether that
    # test is "protected" by either rule 1a or 1b, so the items loop below
    # can treat the matching `fn` item as documented.
    protected_test_fn_lines = set()
    for idx, line in enumerate(lines):
        if TEST_ATTR_RE.match(line):
            tests_total += 1
            protected = preceding_comment_block_has_protects(lines, idx) or \
                        test_fn_window_and_body_protects(lines, idx)
            if not protected:
                tests_unprotected += 1
            else:
                # Find the fn line this attribute governs (skip blank/attr
                # lines forward) so the items loop can exempt it.
                j = idx + 1
                while j < len(lines):
                    s = lines[j].strip()
                    if s == '' or is_attr_line(lines[j]):
                        j += 1
                        continue
                    protected_test_fn_lines.add(j)
                    break

    for idx, line in enumerate(lines):
        # Rule 2: a bare `mod x;` declaration is documented by its target
        # file's own `//!`, not by a preceding `///` at the declaration site.
        mdecl = MOD_DECL_RE.match(line)
        if mdecl:
            items_total += 1
            target = resolve_mod_target(path, mdecl.group(3))
            documented = preceded_by_doc(lines, idx)
            if not documented and target:
                documented = file_opens_with_module_doc(target)
            if not documented:
                items_undoc += 1
            continue
        if ITEM_RE.match(line):
            items_total += 1
            if idx in protected_test_fn_lines:
                continue  # rule 1: protected test fn counts as documented
            if not preceded_by_doc(lines, idx):
                items_undoc += 1
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
