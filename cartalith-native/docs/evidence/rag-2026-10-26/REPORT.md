# RK-0 to RK-3 measurements, 2026-10-26

Retrieval only: no model was run, no socket opened. Every number below was produced on 2026-10-26 on this
machine by the commands named; single runs unless a median and range are stated. Worlds are real
generated worlds (seeds 71077345, 24601, 9137; 512 cells wide, 800 km, 8 factions, 40 settlements each),
their civilisation layer collected by `knowledge_bridge.rs`; the synthetic Vault has 200 notes per world
(9 private, so 191 searched), regenerated from the world deterministically.

## 1. Evaluation set (RK-0)

65 questions per world in nine categories (exact_name 11, figure 10, relational 4, story 10, multi_hop 8,
paraphrase 8, unanswerable 4, conflict 5, private 5). Metrics are retrieval-side: recall@k over gold ids,
MRR, `exp@5` (every expected literal is in the top-5 excerpts: the precondition for a grounded answer),
`expX` (the same with the related documents of the top 10), `citP5` (of the top-5 hits carrying an
expected literal, the share that are gold sources: a proxy, not model citation accuracy), refusal (an
unanswerable question is flagged by an unknown-name signal or by no hit) and leaks. `-` means the category
has no such measure. Whether a model's answer is grounded is RK-8's measurement and is not claimed.

Baseline: the real `VaultSession::search` queried with the string a person would type (favourable to it);
it cannot see the world. Controls: no retrieval (every answerable cell is 0) and shuffled ids (collapses).
Command: `cargo test -p cartalith-knowledge --test knowledge_eval -- --nocapture`.

```
==== seed 71077345: 65 questions, 191 notes, 297 indexed chunks, 1037 documents ====
system: substring (VaultSession::search)  (n=65, overall recall@5 0.170, recall@10 0.170, MRR 0.250, leaks 0)
category       n  gold  R@5    R@10   hit10  MRR    expR10 exp@5  expX   citP5  refuse leaks
conflict        5     5  0.500  0.500  1.000  0.800  0.500  0.000  0.000  0.000    -    0
exact_name     11    11  0.500  0.500  1.000  0.636  0.500    -      -      -      -    0
figure         10    10  0.000  0.000  0.000  0.000  0.000  0.000  0.000  0.000    -    0
multi_hop       8     8  0.188  0.188  0.375  0.375  0.188  0.000  0.000  0.000    -    0
paraphrase      8     8  0.000  0.000  0.000  0.000  0.000    -      -      -      -    0
private         5     0    -      -      -      -      -      -      -      -      -    0
relational      4     4  0.000  0.000  0.000  0.000  0.000  0.250  0.500  0.000    -    0
story          10    10  0.000  0.000  0.000  0.000  0.000  0.000  0.000  0.000    -    0
unanswerable    4     0    -      -      -      -      -      -      -      -    1.000  0
authority: figure-from-world 0.000, story-from-vault 0.000; conflict shows both sources 0.000
system: BM25 over both sources  (n=65, overall recall@5 0.765, recall@10 0.771, MRR 0.797, leaks 0)
category       n  gold  R@5    R@10   hit10  MRR    expR10 exp@5  expX   citP5  refuse leaks
conflict        5     5  1.000  1.000  1.000  1.000  1.000  1.000  1.000  1.000    -    0
exact_name     11    11  1.000  1.000  1.000  1.000  1.000    -      -      -      -    0
figure         10    10  1.000  1.000  1.000  0.800  1.000  1.000  1.000  0.535    -    0
multi_hop       8     8  0.354  0.396  1.000  0.889  0.792  0.625  1.000  0.400    -    0
paraphrase      8     8  0.000  0.000  0.000  0.000  0.000    -      -      -      -    0
private         5     0    -      -      -      -      -      -      -      -      -    0
relational      4     4  1.000  1.000  1.000  0.875  1.000  1.000  1.000  0.229    -    0
story          10    10  1.000  1.000  1.000  1.000  1.000  1.000  1.000  1.000    -    0
unanswerable    4     0    -      -      -      -      -      -      -      -    1.000  0
authority: figure-from-world 1.000, story-from-vault 1.000; conflict shows both sources 1.000
==== seed 24601: 65 questions, 191 notes, 297 indexed chunks, 1020 documents ====
system: substring (VaultSession::search)  (n=65, overall recall@5 0.170, recall@10 0.170, MRR 0.250, leaks 0)
category       n  gold  R@5    R@10   hit10  MRR    expR10 exp@5  expX   citP5  refuse leaks
conflict        5     5  0.500  0.500  1.000  0.800  0.500  0.000  0.000  0.000    -    0
exact_name     11    11  0.500  0.500  1.000  0.636  0.500    -      -      -      -    0
figure         10    10  0.000  0.000  0.000  0.000  0.000  0.000  0.000  0.000    -    0
multi_hop       8     8  0.188  0.188  0.375  0.375  0.188  0.000  0.000  0.000    -    0
paraphrase      8     8  0.000  0.000  0.000  0.000  0.000    -      -      -      -    0
private         5     0    -      -      -      -      -      -      -      -      -    0
relational      4     4  0.000  0.000  0.000  0.000  0.000  0.250  0.500  0.000    -    0
story          10    10  0.000  0.000  0.000  0.000  0.000  0.000  0.000  0.000    -    0
unanswerable    4     0    -      -      -      -      -      -      -      -    0.750  0
authority: figure-from-world 0.000, story-from-vault 0.000; conflict shows both sources 0.000
system: BM25 over both sources  (n=65, overall recall@5 0.771, recall@10 0.771, MRR 0.756, leaks 0)
category       n  gold  R@5    R@10   hit10  MRR    expR10 exp@5  expX   citP5  refuse leaks
conflict        5     5  1.000  1.000  1.000  1.000  1.000  1.000  1.000  0.933    -    0
exact_name     11    11  1.000  1.000  1.000  1.000  1.000    -      -      -      -    0
figure         10    10  1.000  1.000  1.000  0.750  1.000  1.000  1.000  0.543    -    0
multi_hop       8     8  0.396  0.396  1.000  0.792  0.792  1.000  1.000  0.312    -    0
paraphrase      8     8  0.000  0.000  0.000  0.000  0.000    -      -      -      -    0
private         5     0    -      -      -      -      -      -      -      -      -    0
relational      4     4  1.000  1.000  1.000  0.625  1.000  1.000  1.000  0.438    -    0
story          10    10  1.000  1.000  1.000  1.000  1.000  1.000  1.000  1.000    -    0
unanswerable    4     0    -      -      -      -      -      -      -      -    1.000  0
authority: figure-from-world 0.900, story-from-vault 1.000; conflict shows both sources 1.000
==== seed 9137: 65 questions, 191 notes, 297 indexed chunks, 1185 documents ====
system: substring (VaultSession::search)  (n=65, overall recall@5 0.170, recall@10 0.170, MRR 0.250, leaks 0)
category       n  gold  R@5    R@10   hit10  MRR    expR10 exp@5  expX   citP5  refuse leaks
conflict        5     5  0.500  0.500  1.000  0.800  0.500  0.000  0.000  0.000    -    0
exact_name     11    11  0.500  0.500  1.000  0.636  0.500    -      -      -      -    0
figure         10    10  0.000  0.000  0.000  0.000  0.000  0.000  0.000  0.000    -    0
multi_hop       8     8  0.188  0.188  0.375  0.375  0.188  0.000  0.000  0.000    -    0
paraphrase      8     8  0.000  0.000  0.000  0.000  0.000    -      -      -      -    0
private         5     0    -      -      -      -      -      -      -      -      -    0
relational      4     4  0.000  0.000  0.000  0.000  0.000  0.250  0.500  0.000    -    0
story          10    10  0.000  0.000  0.000  0.000  0.000  0.000  0.000  0.000    -    0
unanswerable    4     0    -      -      -      -      -      -      -      -    0.750  0
authority: figure-from-world 0.000, story-from-vault 0.000; conflict shows both sources 0.000
system: BM25 over both sources  (n=65, overall recall@5 0.753, recall@10 0.771, MRR 0.753, leaks 0)
category       n  gold  R@5    R@10   hit10  MRR    expR10 exp@5  expX   citP5  refuse leaks
conflict        5     5  1.000  1.000  1.000  1.000  1.000  1.000  1.000  1.000    -    0
exact_name     11    11  1.000  1.000  1.000  1.000  1.000    -      -      -      -    0
figure         10    10  1.000  1.000  1.000  0.800  1.000  1.000  1.000  0.507    -    0
multi_hop       8     8  0.271  0.396  1.000  0.521  0.792  0.875  1.000  0.357    -    0
paraphrase      8     8  0.000  0.000  0.000  0.000  0.000    -      -      -      -    0
private         5     0    -      -      -      -      -      -      -      -      -    0
relational      4     4  1.000  1.000  1.000  1.000  1.000  1.000  1.000  0.167    -    0
story          10    10  1.000  1.000  1.000  1.000  1.000  1.000  1.000  1.000    -    0
unanswerable    4     0    -      -      -      -      -      -      -      -    1.000  0
authority: figure-from-world 1.000, story-from-vault 1.000; conflict shows both sources 1.000
```

## 2. Bars, as written, against the numbers

| Bar (RK-3 targets, finalised unchanged) | Measured |
|---|---|
| recall@10 exact-name at least 0.90 | 1.000 on all three worlds |
| recall@10 overall at least 0.75 | 0.771, 0.771, 0.771 (paraphrase 0 by design; multi-hop 0.396 before expansion, 0.79 with it) |
| strictly above the substring baseline on every category it can answer | conflict, exact_name, multi_hop: 1.0 vs 0.5, 1.0 vs 0.5, 0.396 vs 0.188; all three worlds |
| identical ordered top-10 over 100 runs and across two processes | passes (`rankings_are_identical_across_100_runs_and_two_builds`, `..._across_two_processes`) |
| a query of only stop words or empty returns `available: false` with a reason | passes (unit test) |
| baseline overall recall@10 | 0.170 (BM25 0.771) |

The paraphrase category (8 questions sharing no word with their note) scores 0 for both systems: the
vocabulary-miss subset RK-9 will gate on. Unanswerable: BM25 flags all four on every world, the baseline
1.0, 0.75, 0.75. Private notes: 0 leaks for BM25; the positive control (markers stripped) leaks, so the
check can fail.

An evaluation finding that changed code (not a bar): the first excerpt builder kept only lines matching the
query, so "how many people live in X?" returned the settlement's document without its population line
(figure `exp@5` 0.70). It now spends the remaining budget on the document's opening lines (figure 1.00). The
numbers above are after that fix; before it, figure exp@5 was 0.700 and multi-hop exp@5 0.375 on the first world.

## 3. Cold build, memory and latency (RK-3, owner answer K9)

Synthetic notes averaging about 5 KB (bar: 4 KB at most, so slightly conservative), 10 chunks per note,
cold-built from disk (list, read, chunk, tokenize, index), release profile, three runs, run alone as far
as this shared machine allows.
Command: `cargo test -p cartalith-knowledge --release --test knowledge_scale -- --ignored --nocapture --test-threads 1`.

| Notes | Chunks | Cold build median (min..max) | Index by field sizes | Process peak working set |
|---:|---:|---|---:|---:|
| 2 000 | 19 849 | 0.65 s (0.64..0.69) | 73 MB | 316 MB |
| 5 000 | 49 586 | 1.63 s (1.62..1.66) | 183 MB | 762 MB |
| 10 000 | 99 532 | 3.45 s (3.44..4.53) | 368 MB | 1 502 MB |

Bar: 2 000 notes at most 3 s: 0.65 s. Query latency over 300 queries on the 99 532-chunk index: p50 0.90 ms,
p95 1.71 ms, max 2.34 ms (bar: p95 at most 50 ms on 20 000 chunks). The persistence question (K4) is not
triggered. **A finding to carry:** 1.5 GB peak at 10 000 notes is large for a phone; the cost is one
`String` per term per chunk. Interning terms would cut it by about a third; not done, nothing asked for it.

## 4. RK-1 render time at the default 2 048-wide grid (dev profile, opt-level 1)

Median over the three worlds: collecting the input from engine state 0.283 s (this includes the trade
match and the faction relations), rendering 0.004 s, building the index 0.013 s; render plus build
0.017 s (bar: at most 1 s). Command: `cargo test -p cartalith-godot --lib measure_knowledge_render -- --ignored --nocapture`.
