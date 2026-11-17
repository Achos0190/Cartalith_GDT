# Live assistant runs, 2026-11-17

Owner permission (2026-11-17): the live runs may be done by agents with LM Studio and qwen3.8-27b `UD-Q3_K_XL` under the saved
preset `cartalith-qwen3.8-27b-q3kxl`. **What was run, and how.** One machine (Ryzen 7 9800X3D, RX 7800 XT 16 GB), LM Studio
0.4.x with the llama.cpp **ROCm 2.73.0** runtime (the latest installed; Vulkan never selected), `qwen3.8-27b` (reported
quantisation `Q3_K_XL`, 14.08 GB), loaded for the runs with `lms load qwen3.8-27b --gpu max -c 8192 --parallel 1` (the key
`qwen3.8-27b@q3_k_xl` is not valid on this install: the model's key is `qwen3.8-27b`; `lms ps` confirmed context 8192,
parallel 1). Thinking off by the app's own `reasoning_effort: "none"` (all 18 chat requests carried it; every reply reported 0
reasoning tokens). The saved preset file holds the same four load fields; the load flags were given on the command line, so
the preset file itself was not exercised (it was not edited). The server was already running when the session began and was
left running; **the model was loaded by this session and unloaded at the end**; no `llama-server` was started separately.

**Real UI path.** A scratch `APPDATA` (wiped per run), a scratch world (seed 9137, 256 x 164 cells, generated through the shell's
own generate call) and a scratch Vault under `C:/cw/asst_scratch/`; nothing of the owner's settings, worlds or Vault was read.
The probe `_llmlive_run_probe.gd` types into the panel's own input, picks the task chip, and presses the real Send, Apply and
Undo-and-regenerate buttons' `pressed` (the buttons' handlers are the click path; the mouse is not synthesised). Every request
went through the recording proxy `asst_proxy.py` (loopback 1235 to LM Studio 1234), so the **request bodies below are the bytes
the app sent**. Single runs, one machine: observations, not a study; variance between runs is itself a finding (below).

Files: `raw/proxy_<scenario>.jsonl` (every request body and the full SSE response with timings), `raw/result_<scenario>.json`
(the probe's events and the final transcript text), `raw/summary_<scenario>.json` (parsed: tools offered, tool calls, usage),
`raw/requests_table.md`, `raw/godot_<scenario>.log`, `shots/live_*.png`. No key exists or was used (LM Studio takes none; the
proxy never logs headers).

## Per scenario

| # | Scenario | Outcome | Tokens (prompt / completion) | Latency (first byte / total, per request) |
|---|---|---|---|---|
| 1 | **Q&A with citations** (Commands task, both search boxes ticked, 3 questions) | Pass. Q1 "Which settlements trade with X?": the model called `related_sources` (trades_with), then listed 9 partners each cited `[W:settlement:N]`, with goods and volumes, and said honestly that two had no trade detail. Q2 (Vault): quoted the note's 1204 walls with a `[V:...]` citation and noted the note's population (26 639) against the world data (24 417) without inventing a resolution. Q3 (planted private note): the model called `read_source` on the one public note and answered "no secret". Sources list and citations drawn by the app: all verified, none unverified. | 2 032 / 57, 2 547 / 308, 2 234 / 101, 1 970 / 46, 2 167 / 152 | first byte 0.6 to 3.1 s; total 2.6 to 7.0 s |
| 2 | **Parameter proposal, Apply and regenerate, Undo and regenerate** (Commands, parameters ticked, "Make the climate drier and the mountains lower.") | **Run 1, default window (guard 8 192): not completed.** The model asked for the groups climate, weather, world, then (request 2, 6 208 prompt tokens) called `get_param_schema` AGAIN (weather) instead of the proposal; the app's request-size guard then refused the third request ("about 7 978 tokens plus 1 024 ... more than the 8 192-token window, so it was not sent"). **Run 2, same settings: passed.** Card with 3 changes, Apply 3 changes and regenerate: peak 4 000 -> 2 800 m; Undo and regenerate: peak 4 000 and the field digest equal to the original. A third run with the guard set to 9 300 (server still 8 192) also passed with the same two-request flow. | req 1 3 578 / 35; req 2 6 208 / 252 (run 2), 6 740 / 209 (guard 9 300) | req 1 2.0 s first byte; req 2 first byte 14 to 15 s (prefill of 6.2 to 6.7 k tokens at about 450 tok/s), total 24.6 to 25.0 s |
| 3 | **Painted region, ridge** (Edit, "Raise the area I painted into a ridge about 300 m high.") | Card, Apply, 2 901 cells changed, one undo group, undo restored the field digest. The model chose `raise` 300 m, NOT the `ridge` stamp (see capability limits). The first reply added `strength` to a raise (R7), the app refused it with the rule and retried once, the retry was valid. | 1 863 / 112 then 2 054 / 104 | first byte 12.8 s (cold prompt) then 2.1 s; totals 19.9 s and 9.7 s |
| 4 | **Painted region, mesa** ("Turn the area I painted into a mesa ... about 250 m high.") | Pass at the first request: `mesa`, 250 m; preview 839 cells; Apply, one undo group, undo restored. | 1 873 / 121 | 1.8 s / 11.0 s |
| 5 | **Painted region, colour paint** ("Paint the area I painted red.") | Pass at the first request: `paint` colour red, opacity 0.8 (blend default, said on the card); 0 height cells changed (as designed), Apply = one undo group, undo restored the field digest; frame `shots/live_edit_paint_applied.png` shows the red area on the map inside the painted shape. | 1 854 / 101 | 1.9 s / 6.6 s |
| 6 | **Profile region tick on and off** (built-in Local profile, region tick on; a copy of Local with it off; back to Local) | Pass. Local stored `ctx.region` true and asked: card. The copy (profile `p2`) stored false: Send refused with the reason naming the tick, **no request reached the proxy**. Back on Local the tick came back on and the request carried the region. All three requests that were sent carry `REGION region_...` in the system message; none carries an image. | 1 863 / 112, 2 054 / 104, 1 863 / 110 | 0.4 to 1.0 s first byte; 3.5 to 3.9 s total |
| 7 | **A private note is never sent** | Pass. Across all eight proxy logs (18 chat requests) the planted private note's marker, its sentence and its file name appear in **no request body** (`raw/proxy_*.jsonl`, checked by search; the canary word was kept out of the user's own typing, because a first version of Q3 contained it and flagged itself). The model's answers never contained it. The note was a `private: true` Vault note; the same Vault's public note reached the model. | | |

## What the runs show about the app

* The shipped default, a Local profile at the owner's reference window of 8 192, **runs a three-group parameter proposal close to
  its ceiling**: the proposal request is 6.2 to 6.7 k real tokens plus 1 024 kept for the answer, and one extra
  `get_param_schema` round (what the model did in run 1) is refused by the guard. See `OUTSTANDING_WORK.md` ("Live runs"): not
  fixed here (a design choice, AQ6).
* Retry-and-validate worked as designed in 3 of 3 cases of a wrong argument (the model's `strength` on a raise).
* Apply, regenerate, Undo-and-regenerate, the edit Apply/undo group and the region tick all behaved as specified on a real model.
* The panel, Send, and card buttons worked through the real UI path on desktop 1600 x 900. Phone and tablet were not run live.
