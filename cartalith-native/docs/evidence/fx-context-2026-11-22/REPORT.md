# Ruling FX: the assistant's context budget, 2026-11-22

Machine: Ryzen 7 9800X3D, RX 7800 XT 16 GB, LM Studio llama.cpp **ROCm 2.80.1** (Vulkan never selected), `qwen3.8-27b`
(UD-Q3_K_XL, 14.08 GB), loaded for each size by `lms load qwen3.8-27b --gpu max -c <N> --parallel 1` (`lms ps` confirmed the
context each time; `lms ps` was empty at the start and the model is unloaded at the end; no `llama-server` was started by this
session). Thinking off by the app's own `reasoning_effort: "none"`. The preset's flash-attention field is not a `lms load`
flag, so the loads follow LM Studio's own setting. Other lanes ran builds and census runs on the same machine meanwhile, so
every speed figure carries that noise (see the two bench passes). Real UI path: `_llmlive_run_probe.gd` through the panel, a
scratch world (seed 9137), scratch APPDATA and Vault, requests recorded by `tools/asst_proxy.py`. `tools/` holds every script;
`raw/` the results, the compacted request logs (usage line only for responses), VRAM samples and the load listings.

## 1. The trim (tokens per request, the server's own `usage.prompt_tokens`)

`get_param_schema` results are built by `Catalogue::schema_entry` (new) instead of `entry`: no `step`; `type` only for int/bool;
`default` only when it differs from `current`; `advanced` is `true` (its meaning now stated once in the tool description) and an
advanced row shows its label only. A schema call that names groups already shown is answered with no settings and a note; one
that overlaps sends only the new keys (`ParamsConversation`). Queries (`get_parameters`) keep the full entry.

| | before (origin/main) | after |
|---|---|---|
| schema JSON, climate + weather + world | 11 782 bytes | 7 550 bytes (-36 %) |
| request 1, prompt tokens | 3 575 | 3 626 (the tool description grew by three sentences) |
| request 2 (the proposal request), prompt tokens | 7 172 (27 174 bytes), 3 of 3 runs | 6 148 (22 968 bytes), 5 of 5 runs |
| schema's own share of request 2 | 3 597 tokens | 2 522 tokens (-30 %) |
| a repeated `get_param_schema` round | a third request of 8 091 tokens (2 of 3 runs) | no schema re-sent; the third request is the first plus the 200-byte note |

(`raw/before/`: three runs of the old flow at 12 288; `raw/c8192/`: the new flow. Before and after chose the same three groups.)

## 2. The window

Per size: 5 parameter runs ("Make the climate drier and the mountains lower." with Apply and regenerate) and 5 edit runs
("Raise the area I painted into a ridge about 300 m high."), the guard set to the loaded size. Success: params = a card with at
least one valid change that Apply took; edit = a card with a valid op that Apply changed the field for.

| context | params ok | edit ok | request 2 / edit request (real tokens) | first byte, median (max), s | generation, median tok/s, params / edit | VRAM dedicated max / shared max (GB) |
|---|---|---|---|---|---|---|
| 8 192 | 5/5 | 5/5 | 6 148 / 2 475 | 2.1 (23.0) / 2.4 (9.2) | 57.3 / 68.5 | 15.71 / 6.18 |
| 12 288 | 5/5 | 5/5 | 6 148 / 2 475 | 2.6 (14.6) / 2.6 (6.7) | 29.9 / 53.4 | 15.75 / 6.55 |
| 16 384 | 5/5 | 5/5 | 6 148 / 2 475 | 2.6 (40.9) / 3.9 (26.0) | 30.5 / 25.4 | 15.66 / 7.97 |

The in-app speed columns are not an ordering of the sizes: another lane's work was running, and the controlled bench shows it
(`tools/bench.py`, no app, a 5 122-token prompt, 200 generated tokens, 3 reps per size, run twice in opposite order):

| context | pass A first token s / prefill tok/s / gen tok/s | pass B (reverse order) |
|---|---|---|
| 8 192 | 18.0-19.0 / 269-285 / 21.6-31.9 | 20.0-20.5 / 250-256 / 45.2-49.5 |
| 12 288 | 27.7-28.3 / 181-185 / 20.6-26.6 (slowed by outside load) | 17.4-17.7 / 290-294 / 40.7-46.7 |
| 16 384 | 18.1-18.7 / 274-282 / 36.1-40.7 | 20.8-21.5 / 238-246 / 34.4-48.7 |

Same size, different pass, differs more than the sizes differ from each other: **no speed effect of 8 192 / 12 288 / 16 384 is
measurable on this machine**. (A first bench with a 9 162-token prompt could not run at 8 192 and its other two sizes were
likewise disturbed; kept out of the table.) VRAM: the weights and compute buffers fill the card (dedicated 15.6-15.75 of 16 GB, a
headroom of about 0.3 GB, at **every** size); each extra 4 096 tokens of context moves its KV cache into shared (host) memory,
which rose from 6.2 to 6.6 to 8.0 GB (bench idle readings 5.1 / 5.6 / 6.4 GB, peaks 6.5 / 7.3 / 9.0). There is no VRAM headroom
to buy a bigger window with; the price is system memory.

## 3. The choice: 12 288

* 8 192 now passes the trimmed three-group flow (request 2 is 6 148 real tokens), but the guard's pessimistic 3.4 chars/token
  puts that request at about 7 000 tokens plus the 1 024 kept for the answer, so it passes by a few hundred tokens only, and the
  heavier group sets (civ, erosion and world together are about 22 000 bytes untrimmed) are refused. The ruling asked for room.
* 12 288 clears the guard's estimate of the heaviest three-group request with about 2 000 tokens spare, costs about 0.4 GB of
  shared memory over 8 192, and measured no slower.
* 16 384 gave 5/5 as well but needs 1.8 GB more shared memory than 8 192 for room the flow does not use, and the one run
  with a 40.9 s first byte (a cold load) is what the extra spill risks on a card that is already full.

Sample size: five runs per scenario and size, one scenario of each kind, a single model: a measurement of this machine, not a
study. All 30 runs succeeded, so the success rate does not separate the sizes; the decision rests on the guard arithmetic, the
VRAM and the absence of a speed cost.

## 4. What changed

`cartalith-llm/src/catalogue.rs` (`schema_entry`, the tool description), `params_flow.rs` (the repeat/delta answer),
`action_tools.rs` (`DESIGN_WINDOW_TOKENS` 8 192 -> 12 288, the warning text), the preset `cartalith-qwen3.8-27b-q3kxl`
(`llm.load.contextLength` 8 192 -> 12 288; `preset_after.json`), and the prose in `dcc_settings.gd`, `llm_assistant_window.gd`,
`llm_run.gd`, `llm_stream.rs`. `context_tokens` stays 0 = "the design window", so the setting and the guard agree with the
preset. A server loaded at a smaller context needs `context_tokens` set (the guard warns below 12 288).
Mutants (`tools/mutants.py`, `raw/mutants.txt`): nine literal mutants, nine killed.
