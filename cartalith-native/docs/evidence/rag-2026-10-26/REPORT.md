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
`String` per term per chunk. Interning terms would cut it by about a third (**done 2026-10-26 on the owner's answer: section 5, 917 MB**; the table above is the before).

## 4. RK-1 render time at the default 2 048-wide grid (dev profile, opt-level 1)

Median over the three worlds: collecting the input from engine state 0.283 s (this includes the trade
match and the faction relations), rendering 0.004 s, building the index 0.013 s; render plus build
0.017 s (bar: at most 1 s). Command: `cargo test -p cartalith-godot --lib measure_knowledge_render -- --ignored --nocapture`.


## 5. Term interning (owner answer to the 1.5 GB finding, 2026-10-26)

The finding of section 3 was that every chunk owned a `String` per term it contained. `bm25::TermId` is now a four-byte
id into a process-wide, append-only table; `TermStats` holds `(TermId, [u16; 3])`; `Bm25Index::build` gathers postings
by id and keys them by text once. **Ids mean nothing**: they are handed out in first-seen order, and every order that
reaches a score is the term's TEXT (`TermStats::from_fields` sorts by text, the postings map is keyed by text), which
is why ranking cannot move.

Same command, same machine, release profile, three runs (min..max), run before the change (commit `644e398f`) and after
(`3dab18ef`). The process peak includes the previous build still alive while the next one runs, in both columns, so it
overstates one index and compares like with like. `cargo test -p cartalith-knowledge --release --test knowledge_scale
-- --ignored --nocapture --test-threads 1`.

| Notes | Cold build before | after | Index by field sizes before | after | Process peak before | after |
|---:|---|---|---:|---:|---:|---:|
| 2 000 | 0.70 s (0.69..0.71) | 0.55 s (0.55..0.59) | 73 MB | 35 MB | 316 MB | 200 MB |
| 5 000 | 1.69 s (1.68..1.72) | 1.33 s (1.33..1.34) | 183 MB | 88 MB | 762 MB | 472 MB |
| 10 000 | 3.46 s (3.43..3.49) | 2.68 s (2.62..2.70) | 368 MB | 177 MB | 1 504 MB | 917 MB |

Query latency over 300 queries on 99 532 chunks: p50 0.89 to 0.83 ms, p95 1.70 to 1.66 ms, max 2.38 to 2.34 ms.

**Bars, as written:** peak at 10 000 notes at most 1.1 GB: **917 MB, met**. Build time not worse than +15 percent:
**22 percent faster, met** (a `u32` id is cheaper to move and hash than a `String` clone). Recall identical to the digit
on the three seeds: **identical**. The full `knowledge_eval` output (three worlds, nine categories, both systems, every
column) before and after was diffed and is byte-identical; command `cargo test -p cartalith-knowledge --release --test
knowledge_eval -- --nocapture --test-threads 1`. Mutation (literal, Python): the term list sorted by id instead of text,
one text given two ids and postings keyed by the wrong text were each killed.

Not changed, stated: the table never shrinks (a term, once seen, is kept for the process; it is bounded by the distinct
tokens ever indexed, not by chunks) and holds no note text beyond single tokens.

## 6. RK-5 to RK-7: what was built and what the gates measured

Code: `crates/cartalith-llm/src/knowledge_tools.rs` (the three tools, strict arguments, result rebuild, hardening, pack,
citation check), `params_flow.rs` (questions-only conversation, the known-id rule, the ledger), `prompt.rs` (the pack on
the prose path), `crates/cartalith-godot/src/knowledge_service.rs` and `knowledge_bridge.rs` (the in-memory index, the
executor, `WorldGen::knowledge_call` / `knowledge_search_json`), `llm_stream.rs` (`knowledge_plan`, `knowledge_limits`,
sources and citations on every answer), `shell/llm_knowledge.gd`, `llm_run.gd`, `llm_assistant_window.gd`,
`llm_settings_window.gd`, `llm_consent.gd`, `dcc_settings.gd`.

Tests, `cargo test --workspace --no-fail-fast -j 2` on the final merged tree (this change on top of LLM-11, origin
`10653402`): 259 result lines, 7 575 passed, 0 failed, 142 ignored (the figure at the previous commit was 258 / 7 503).
New or changed: `cartalith-llm` lib 210 (was 156 before LLM-11), `cartalith-knowledge` lib 42 (was 40),
`cartalith-godot` lib (five new in `knowledge_service`), `tests/rk7_knowledge_hardening.rs` 11, `tests/llm9_corpus.rs` 7
(now 99 rows x 2 vectors x 2 wires = 396 runs, and 99 x 5 prose tasks = 495 builds, the grounded path included). The three RK-3 bars still hold: the `knowledge_eval` table is byte-identical to section 1.

Mutation (literal, Python: exact once-only replace, BUILD_ERROR classified apart, restored in a `finally`, sha256 checked):
**48 mutants, all killed on the final merged tree** (`mutation.txt`). Five survived their first round and were closed: an id minimum of 1 instead
of 3 (a test now refuses two characters), the result cap not enforced after the rebuild (a worst-case eight-hit test
now exceeds it), a redundant source check (removed, so the remaining one is the mutated one), the mentioned-id source
filter (a test now passes a world id with only the Vault ticked) and a forged-proposal path in the questions-only
conversation. One mutant, "a miss returns a body", survived at the service level because the Rust-side rebuild hid it;
the service test now reads the RAW miss.

**Defects the work found and fixed (in code, not in a constant):**

1. **`prompt.rs::defuse_fence` did not see a zero-width character INSIDE the word `context`** (corpus row K09:
   `</con` + U+200B + `text>`); it skipped invisible characters around the word only. Fixed, and its invisible-character
   set is now the one definition in `knowledge_tools::is_invisible`, widened to the soft hyphen, the invisible operators,
   the bidirectional isolates and the Unicode tag block.
2. **A fixture that could not fail.** The first privacy check put the canary words in the user's own question, so they
   were in every request body whatever the index did. The positive control (an unmarked note IS retrieved) now asks a
   question that names nothing secret, in both modes.
3. **A phone layout fault, found by the probe, not caused by the knowledge feature but exposed by it.** The panel's
   sheet keeps the content size from its last window layout (`wrap_controls` is off and `_refit` skips a phone). The
   default Prose task's first layout measured 887 px in an 832 px window; after the user switched to Commands the
   content stayed oversize and Send sat under the dialog's Close row, where a tap never reaches it (the row's
   `mouse_filter` is PASS). `_relayout_phone` nudges the window size after a task change and after `open()`: the content
   now measures 793 px in 832 and Send ends at 733.

**Merged with LLM-11 (grounded place prose, origin `10653402`, which landed while this was being built).** Both rewrote
`prompt.rs`, `llm_stream.rs` and the panel. The merge keeps LLM-11's grounded prompts and place bundle unchanged and adds the
pre-retrieval pack to a grounded request as a THIRD fence after WORLD DATA and VAULT NOTES, with `GROUNDED_ADDENDUM` in place
of the plain sources paragraph (the grounded prompt says "use ONLY these blocks", which the plain paragraph would
contradict); the search for a place's prose adds the place's name to the words, so its own document is found. Two private-mark
conventions now exist side by side and are NOT unified here: this feature's (K3: `dm` or `private` frontmatter, the tags
`dm-only` and `secret`, an inline hashtag, a `DM`, `Private` or `Secrets` folder) and LLM-11's `place_vault` (`private: true`,
`dm-only`, `#dm`, `#secret`, `#no-ai`); an owner question.

**Windowed probes**, `_llmknow_probe.gd` (loopback mock server on both wires, a REAL generated world and a REAL Vault
written to a scratch directory, a wiped user dir, the debug DLL rebuilt after the last `.rs` edit, each run under the
window lock): see section 7.

## 7. Windowed probe results (final DLL, sequential, under the window lock)

| Probe | Size | Result |
|---|---|---|
| `_llmknow_probe` | desktop | PASS (0 failed) checks=103 |
| `_llmknow_probe` | tablet | PASS (0 failed) checks=103 |
| `_llmknow_probe` | phone | PASS (0 failed) checks=103 |
| `_llmplace_probe` | desktop | PASS (0 failed) |
| `_llmplace_probe` | tablet | PASS (0 failed) |
| `_llmplace_probe` | phone | PASS (0 failed) |
| `_llmparams_probe` | desktop | PASS (0 failed) checks=137 |
| `_llmparams_probe` | tablet | PASS (0 failed) checks=141 |
| `_llmparams_probe` | phone | PASS (0 failed) checks=141 |
| `_llmquery_probe` | desktop | PASS (0 failed) checks=1101 |
| `_llmquery_probe` | tablet | PASS (0 failed) checks=1101 |
| `_llmquery_probe` | phone | PASS (0 failed) checks=1101 |
| `_llmview_probe` | desktop | FAIL (8 failed) checks=382 (exit 1) |
| `_llmview_probe` | tablet | PASS (0 failed) checks=388 |
| `_llmview_probe` | phone | FAIL (8 failed) checks=388 (exit 1) |
| `_llmpanel_probe` | desktop | PASS (0 failed) |
| `_llmpanel_probe` | tablet | PASS (0 failed) |
| `_llmpanel_probe` | phone | PASS (0 failed) |
| `_llmset_probe` | desktop | PASS (0 failed) |
| `_llmset_probe` | tablet | PASS (0 failed) |
| `_llmset_probe` | phone | PASS (0 failed) |
| `_llmhosted_probe` | one | PASS (0 failed of 65 checks) |
| `_llmstream_probe` | one | PASS (0 failed) |
| `_llmview_probe` | desktop, phone | **8 checks fail at each (tablet passes): a failure that is NOT caused by this change** (it reproduces with origin's own shell scripts; the Help > Keyboard shortcuts command row is not offered as available now, so its `open_window` action is dropped). Reported, not fixed. The table above is the merged tree; `_llmview` was not in the brief's list |

An earlier pass of the same probes on the same build failed four runs while a 27B model was generating and a mutation run was compiling (the knowledge probe at desktop, `_llmparams` at desktop, `_llmquery` at desktop and tablet); each passed on a re-run on a quiet machine, so those were load, not defects. `probe_results.txt` has the raw lines.

Screenshots (opened and read; desktop 1600x1000, tablet 1280x800, phone 393x852) are in this directory as `rk_*.png`.
Read, not only produced: the sources card wraps its title and locator beside a fixed-width chip on every size (the first
version put the chip and the text in one flow row and gave the text a column of three-word lines; fixed), the pre-retrieval
note and the sources list sit under the answer, the settings window shows the two boxes below the fold and a click scrolls
to them, and the consent card quotes the caps.

## 8. Single-run observations with real models (LM Studio, ROCm runtime 2.64.0, 2026-10-26)

**Labelled single runs, not rates.** qwen3.8-27b (UD-Q3_K_XL, offload max, context 8 192, parallel 1, MTP on, thinking
off) and qwen3-14b (thinking off), through the app's own path (`_llmknow_cli.gd`: the real gate, conversation, index and
wire), on seeds 9137 and 24601 (256 wide), a Vault written from the world. Latencies are on a machine
shared with other work and are not benchmarks. Raw JSON for every run is in `live/`.

Questions: **port** "Which settlements trade with S?" (S a settlement with 10 or 11 trade partners; the expected partner
set is read from `civ_trade_flows`, independent of the knowledge base) and **faith** "What do we know about Ancestor Rites?"
(a faith of the world that a planted note also describes). Modes: **pre** (the app searches first and the model gets the
pack and no tool), **tools** (the model has the three tools and no pack), **both**.

| Model | Seed | Question | Mode | Tool calls | Pack | Result | Citations verified / named, not read / unverified | Sources listed | Prompt tokens | Seconds |
|---|---:|---|---|---|---|---|---|---:|---:|---:|
| qwen3.8-27b | 9137 | port | pre | none | 5 hits, 562 tokens | 6 / 11 names | 2 / 0 / 0 | 5 | 1086 | 10.6 |
| qwen3.8-27b | 9137 | port | tools | search_knowledge, related_sources | - | 11 / 11 names | 13 / 0 / 0 | 16 | 2773 | 17.6 |
| qwen3.8-27b | 9137 | port | both | related_sources | 5 hits, 562 tokens | 11 / 11 names | 13 / 0 / 0 | 16 | 2593 | 18.9 |
| qwen3.8-27b | 24601 | port | pre | none | 5 hits, 654 tokens | 8 / 10 names | 3 / 0 / 0 | 5 | 1262 | 9.0 |
| qwen3.8-27b | 24601 | port | tools | search_knowledge, related_sources | - | 10 / 10 names | 12 / 0 / 0 | 15 | 2923 | 15.2 |
| qwen3.8-27b | 24601 | port | both | related_sources | 5 hits, 654 tokens | 10 / 10 names | 11 / 0 / 0 | 15 | 2681 | 12.4 |
| qwen3.8-27b | 9137 | faith | pre | none | 2 hits, 201 tokens | faith doc and note in sources | 2 / 0 / 0 | 2 | 519 | 7.4 |
| qwen3.8-27b | 9137 | faith | tools | search_knowledge, read_source, read_source | - | faith doc and note in sources | 2 / 0 / 0 | 2 | 1724 | 10.9 |
| qwen3.8-27b | 9137 | faith | both | related_sources | 2 hits, 201 tokens | faith doc and note in sources | 2 / 0 / 0 | 2 | 1459 | 7.9 |
| qwen3.8-27b | 24601 | faith | pre | none | 2 hits, 201 tokens | faith doc and note in sources | 2 / 0 / 0 | 2 | 519 | 6.1 |
| qwen3.8-27b | 24601 | faith | tools | search_knowledge, read_source, read_source | - | faith doc and note in sources | 2 / 0 / 0 | 2 | 1724 | 11.0 |
| qwen3.8-27b | 24601 | faith | both | related_sources | 2 hits, 201 tokens | faith doc and note in sources | 2 / 0 / 0 | 2 | 1459 | 6.5 |
| qwen3-14b | 9137 | port | pre | none | 5 hits, 562 tokens | 5 / 11 names | 1 / 0 / 0 | 5 | 1064 | 11.0 |
| qwen3-14b | 9137 | port | tools | related_sources, search_knowledge, related_sources | - | 11 / 11 names | 0 / 0 / 0 | 16 | 2848 | 15.4 |
| qwen3-14b | 9137 | port | both | related_sources | 5 hits, 562 tokens | 11 / 11 names | 11 / 0 / 0 | 16 | 2394 | 14.3 |
| qwen3-14b | 24601 | port | pre | none | 5 hits, 654 tokens | 6 / 10 names | 1 / 0 / 0 | 5 | 1245 | 9.1 |
| qwen3-14b | 24601 | port | tools | related_sources, search_knowledge | - | 5 / 10 names | 1 / 0 / 0 | 5 | 2420 | 11.0 |
| qwen3-14b | 24601 | port | both | none | 5 hits, 654 tokens | 6 / 10 names | 1 / 0 / 0 | 5 | 1976 | 10.9 |
| qwen3-14b | 9137 | faith | pre | none | 2 hits, 201 tokens | faith doc and note in sources | 2 / 0 / 0 | 2 | 511 | 6.9 |
| qwen3-14b | 9137 | faith | tools | search_knowledge | - | faith doc only in sources | 1 / 0 / 0 | 1 | 1189 | 5.9 |
| qwen3-14b | 9137 | faith | both | none | 2 hits, 201 tokens | faith doc and note in sources | 2 / 0 / 0 | 2 | 1242 | 2.8 |
| qwen3-14b | 24601 | faith | pre | none | 2 hits, 201 tokens | faith doc and note in sources | 2 / 0 / 0 | 2 | 511 | 10.6 |
| qwen3-14b | 24601 | faith | tools | search_knowledge | - | faith doc only in sources | 1 / 0 / 0 | 1 | 1189 | 8.0 |
| qwen3-14b | 24601 | faith | both | none | 2 hits, 201 tokens | faith doc and note in sources | 2 / 0 / 0 | 2 | 1242 | 2.7 |

What the runs showed, each tied to a change it caused:

* **The excerpt cut a list without saying so.** With the pack only, the 27B listed five of 11 partners as if they were all
  (5 and 6 of 11 names in the first runs). `excerpt` now ends a cut list with `- ... and N more not shown` and the sources
  paragraph says such a list is incomplete; the next runs said "only the five listed above are shown; the remaining 6 are
  not displayed". The retrieval evaluation is byte-identical with the marker.
* **A relational question wants `related_sources`, not a search excerpt.** With the old tool text the 27B read the
  settlement and cited partner ids it had only seen inside the document (11 of 12 citations unverified, all correct). The
  tool descriptions now say to use `related_sources` for the whole list; the final runs call it (11 of 11 and 10 of 10 names,
  13 of 13 and 12 of 12 citations verified). The panel gained a third citation state, **NAMED, NOT READ** (an id a sent
  source names and the index holds, never itself retrieved; computed by the executor against the index, so a hostile name
  typed as `(W:settlement:424242)` mentions nothing).
* **A 14B model guessed an id.** It called `related_sources` with `W:settlement:12`, an id it had never been given, and
  answered about the wrong place with verified-looking citations. `read_source` and `related_sources` now run only for an id
  the request already returned (a retrieved unit, the note of a retrieved Vault chunk, or a name a retrieved source gives);
  a guessed id is refused with a message addressed to the assistant ("call search_knowledge yourself now"), and is not
  executed. The first wording of the refusal made the 14B tell the USER to search; the final wording fixed that
  (single run).
* **Models that never call a tool** still answer from the pack: in `both` mode the 14B made no tool call in 3 of its 4 final
  runs and its faith answers still ended on the two right sources (the `pre` runs offer it no tool at all).
* **Honest limit:** with the pack only (no tool), a long list is partial by construction; the 14B named 5 or 6 of 10 or 11
  and the 27B 6 or 8. The marker makes that visible; completeness needs the tool.

Everything was unloaded and the server stopped; no `llama-server` remained.
