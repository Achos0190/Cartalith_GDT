# AC-1 and AC-2: measurements and the real-model check (2026-11-07)

Milestones AC-1 and AC-2 of `ASSISTANT_ACTIONS_PROPOSAL.md` (Ruling DW). The model-in-the-loop half of AC-0 was
not re-run (the earlier model tests stand in for it, by the owner's decision); this is a handful of single runs
that check a real local model can browse and find actions.

## Model-free measurements (`crates/cartalith-godot/tests/ac2_actions.rs`, real registry and 66 real index rows)

| Measure | Result | Target (s.3.6) |
|---|---|---|
| Entries | 205 (114 params, 13 view, 10 windows, 31 menu, 23 context, 13 tools, 1 timeline) | every runnable row, parameter, verb, window, context action, tool |
| Descriptions longer than 120 | 0; 24 derived (AC-5's list) | at most 120 |
| 66 runnable index rows | 31 entries, 10 merged into windows, 25 refused (destructive words) | none lost |
| Four tool specs + prompt paragraph | 2 720 characters, about 680 tokens | fixed part at most 1.6k |
| Worst list page (all 45 category prefixes, every page) | 1 441 characters, 361 tokens (424 at the guard's 3.4) | at most 400 |
| Worst search of 8 matches | 1 479 characters, 370 tokens (435 at 3.4) | at most 400 |
| Worst single describe | 625 characters, 157 tokens | at most 300 |
| Search reach on AC-0's 30 tasks | 25 of 25 offered slots in the top 8, 24 in the top 5 | AC-0: 27 of 28, 26 of 28 |
| Browse reach | 25 of 25, 3.2 list calls each, worst 4 | - |

Tokens are characters over 4.0, AC-0's measured ratio for compact JSON with three real tokenizers; the guard's own
estimate (3.4) is pessimistic and shown beside it. Two bars failed on the first run and were fixed by design, not by
retuning the target: a search of 8 matches was 1 615 characters (the explicit `available: true` on every row), and a
page of 25 `erosion` settings was 2 086 characters (a character cap on pages now binds). Five of AC-0's 30 task
slots have no action: `select`, `selection.back`, `timeline.play`, `timeline.record_year` (not offered yet; PC-1 has since added `DccApp.select`) and
`menu:undo` (the stem `undo` is forbidden).

## Real model (qwen3-14b, thinking off, LM Studio ROCm, context 8 192, one request at a time)

The model was already loaded by another session at context 8 192 (parallel 4); it was used as found and left loaded.
Driver: `crates/cartalith-godot/tests/ac2_live.rs` (`--ignored`), the real `ParamsConversation`, `StreamSession`
decoder and the guard. Full transcript: `live_qwen3-14b_transcript.txt`.

| Request | What the model did | Server prompt tokens (peak) |
|---|---|---|
| turn off the rivers layer | used the existing `propose_view_actions`: valid | 2 741 |
| open the faction roster | guessed `open_window_factions`: refused B6 with the rule; searched "faction roster"; proposed `window.factions`: valid | 3 061 |
| what can I do with the vault? | searched "vault", answered in words from the 8 matches | 2 874 |
| raise the sea level a bit | used the existing schema and settings tools: valid | 3 391 |
| let me draw a new road | answered in words that it cannot, offered to search, never called a tool | 2 742 |

First request with every tool: 2 741-2 744 server tokens against the guard's estimate of 2 974-2 976 (8 % high, as
intended). The existing tools stay and the model still prefers them where they fit; the catalogue is how it reaches
the rest. The ledger rule worked on a real guess. One miss: the 14B model did not call `search_actions` for the
road request, so the alias that fixes the search is not reached by this model on that prompt.
