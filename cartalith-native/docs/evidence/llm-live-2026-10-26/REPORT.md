# LLM-6c stage 2: the same requests through the real Cartalith assistant panel

Measured 2026-10-26 on the owner's machine (Ryzen 7 9800X3D, 31 GB RAM, RX 7800 XT 16 GB), driven by `godot-project/_llmlive_params_probe.gd` through the real panel: Send, proposal card, a real Apply click, the user's Generate path, statistics, screenshots, one `app.undo_last()`, a digest check. The agent that ran it (commit `8f33c44b`) could not file this report (tool guard); it was assembled from the agent's hand-back and the generated tables below, which come from `tools/aggregate.py` and `tools/tune_table.py` over `raw/*.jsonl`. Stage 1 (no app, labels only) is `../llm-model-eval-2026-10-26/REPORT.md`.

## Headline and ranking (single runs; seed 71077345, plus seed 24601 for the islands request)

1. **C, Claude Sonnet 5.5.** Valid, correct-direction cards and the best island maps (open sea with scattered islets, land 20% and 22%). It refused "add a river here" and offered whole-map options. Median total 6.1 s. It missed "more volcanoes" only because of defect 1 below; it then told the user to set `volc.count` by hand.
2. **B2, Qwen3.8-27B UD-Q3_K_XL, thinking off.** 5 of 5 valid, right-signed cards: no sea (`sea_level` 0.42 to 0); more volcanoes (`volc.count` 20 to 35, 33 volcanoes hit); drier and lower (`climate.ocean` and `climate.evap` down, `peak_m` 4000 to 2800, mean land height 1402 to about 1000 m); islands (steering on, fragmentation 0.85, `sea_level` up, land 30%). It refused the river request. Median 9.9 s.
3. **D, Qwen3.8-27B UD-Q4_K_M, thinking off.** Mostly B2's choices at about half the speed; it acted on the river request. Median 19.2 s.
4. **A, qwen3-14b, thinking off.** Fastest (median 5.1 s, first output 0.8 s) but wrong on the open requests: "drier" became temperature changes; islands raised continentality so land went from 55% to 70%; it acted on the river request; its card summaries describe changes it did not make.
5. **B1, Qwen3.8-27B, thinking on (template default effort `xhigh`).** Median 46 s. Islands on 71077345 failed with a truncated tool call, consistent with the 1,024-token reply cap, though not proven by a `finish_reason`. It acted on the river request and swapped "more volcanoes" for hotspot density.

Plain-language reading: for turning an everyday request into a good map, Claude Sonnet is clearly first; Qwen3.8-27B with thinking off is a close second and is local; qwen3-14b is fast but often chooses the wrong knobs; thinking mode made the local model slower and less reliable here.

## Per-prompt choices on the open requests

- **Islands:** C: steering on, fragmentation 0.8, continentality 0.2, hotspot density up, plus ocean depth on 71077345. B2: steering on, fragmentation 0.85, `sea_level` 0.49. D: steering on, fragmentation 0.95, and `sea_level` 0.62 on 71077345 or continentality 0.12 on 24601. A: steering on, continentality 0.7, fragmentation 0.8, plates 14 to 20, coastal pass on, plus glacial off or sediment fill on.
- **More volcanoes:** A: `volc.count` 30 and `volc.age` 0.25. B2 and D: `volc.count` 35. B1: hotspot density 0.3 with steering on. C: no card (defect 1).
- **Add a river here (impossible):** B2 and C answered in words; A, B1 and D proposed `river_density` 1.5, 1.2 and 1.3.

## Undo and reproducibility

Every applied card, in every arm, undid in ONE step even after a regenerate (params digest equal). The regenerated base world was bit-identical each time.

## Setup

- Seed 71077345 at the app default (2048 x 1311 cells, 800 km wide); the island request was repeated on seed 24601.
- Every local arm ran on LM Studio 0.4.26 with the ROCm 2.64.0 runtime (owner: LM Studio and ROCm only; no Ollama arm, no Vulkan).
- Arms: **A** qwen3-14b thinking off; **B1** Qwen3.8-27B UD-Q3_K_XL thinking default; **B2** the same, thinking off; **D** UD-Q4_K_M thinking off (hard-linked into LM Studio; its projector is the same blob as the existing link); **C** claude-sonnet-5-5 (no auth error).
- The DLL was a fresh debug build (23:45:48) newer than the newest `.rs` (23:40:09).
- Thinking off on LM Studio: LM Studio honours `reasoning_effort: "none"` (0 reasoning tokens, measured), but the app's `lm_studio` preset sends nothing, so the thinking-off arms used the `ollama` preset id pointed at `http://127.0.0.1:1234/v1` (it differs only in sending that field). Giving `lm_studio` the field is a one-line change, not made.

## GPU rule (Ruling DF)

For every local arm the plan read "Detected as: same machine." and switched Cartalith's compute from automatic to the integrated GPU (`1002:13c0` AMD Radeon(TM) Graphics). A generate took 13.4 to 18.9 s there, against 12.7 to 14.2 s on the RX 7800 XT for arm C. **Finding:** the same seed gives a different field digest on the two adapters (`b5386800...` against `946e53e2...`) while the printed statistics are equal; the rule therefore changes the exact world a seed produces. This is an owner question: see OUTSTANDING_WORK.md.

## Tuning (owner deliverable; ROCm only)

Method: three timed runs of a fixed prompt of about 505 tokens, median taken, dedicated and shared GPU memory recorded. Where the knobs are: `lms load` sets only offload, context, parallel and MTP; the REST load endpoint refuses `gpu_offload_ratio`, KV-cache type, threads and mmap and cannot set full offload; `--engine-config-file` is vLLM-only; LM Studio's per-model saved load settings file is read (the server log shows `n_threads=8` and `n_ctx`), and was used for everything, then deleted.

Chosen settings and results:

- **Q3_K_XL (B1/B2):** offload 1.0, context 8192, parallel 1, flash attention on, KV f16, MTP on: **43.7 tok/s**, against 11.9 at LM Studio's defaults (5.3 GB spilled to shared memory) and 31.7 with the first-run settings (`--gpu max -c 16384`, 4 slots, 8.6 GB spilled): +38% over the first runs and about 3.7x over defaults. MTP is worth +34%.
- **qwen3-14b (A):** the same settings give **50.6 tok/s** against 45.9; flash attention is worth +10%. This model has no MTP head (the flag is refused at load).
- **Q4_K_M (D):** it does not fit on the 16 GB card. **Offload 0.9 with 8 threads gives 20.9 tok/s**, against 15.0 at full offload (4 GB spilled) and 12.0 at defaults.
- **No difference established** for KV-cache q8/q4, threads, batch size or mmap on the fully offloaded models. No KV quantisation was adopted; the only trade was context 16384 down to 8192.
- Spot check: the untuned first runs (B1u, B2u) chose the same or equally valid keys as the tuned runs; B1u failed 3 of 5, consistent with the thinking-length cap, not the context.

The generated full tuning table follows.

## Defects found (not fixed; neither blocked the run)

1. **A second schema call narrows what the model may propose.** In `crates/cartalith-llm/src/params_flow.rs`, the schema branch does `self.shown_groups = groups; self.shown_keys = keys;`, so a second `get_param_schema` call replaces the offered keys instead of adding to them. This cost Claude its volcano card.
2. **The assistant panel errors on old cards after it is reopened.** In `shell/llm_assistant_window.gd`, `open()` calls `_rebuild()`, which frees the cards' buttons, but `cards` keeps them; `_refresh_cards`'s first line, `var b: Button = card["apply"]`, then raises "previously freed instance" 132 to 437 times per arm log. It is inferred, not measured, that this skips the V13 "Apply disabled while generating" state for cards made after a reopen.

Smaller: a length-cut tool call is reported as "malformed tool call", not as a length cut; step lines are drawn below the reply bubble; Markdown in replies is shown raw.

## Vision check (Qwen3.8-27B through LM Studio, never through the app)

It described single images well, but failed at "what changed": it called the islands pair a pan and the no-sea pair a palette change.

## Limits

Single runs; two seeds; server-default temperature; one machine; ROCm only; quantisations as listed. "First output" is the first word or decoded tool call, not a strict first token. Volcano count is distinct edifices on a 4-cell lattice, so it is a lower bound. "x1" is the app's reset view (141%, map edges cropped).

## Screenshots

`shots/A_base_s71077345_x1.jpg` and `_x4.jpg`; `shots/<arm>_<request>_s<seed>_after_x1.jpg`; `*_after_x4.jpg` for the islands request (A, B2, C); `*_card.jpg` (C more_volcanoes, C imp_river, B1 islands, A drier, B2 imp_river, B2 and D islands). 38 of 143 screenshots are committed, as 800 px JPEGs.

---

# Generated tables

## Tuning table (`python -I tools/tune_table.py raw/tune.jsonl`)

| Model | Setting | Load s | First token s (median, min..max) | Generation tok/s (median, min..max) | Dedicated GiB | Shared GiB | Prompt tokens |
|---|---|---|---|---|---|---|---|
| qwen3.8-27b@q3_k_xl | defaults: lms load, no flags, no saved settings | 13.8 | 0.231 (0.23..0.268) | 11.92 (10.16..12.04) | 15.64 | 5.25 | 505 |
| qwen3.8-27b@q3_k_xl | --gpu max -c 16384 (first stage-2 runs) | 15.1 | 0.136 (0.136..0.146) | 31.67 (31.6..31.69) | 15.66 | 8.55 | 505 |
| qwen3.8-27b@q3_k_xl | ref: offload 1, ctx 8192, parallel 1, FA on, KV f16, MTP default | 12.7 | 0.109 (0.105..0.121) | 43.65 (43.53..43.68) | 15.68 | 1.08 | 505 |
| qwen3.8-27b@q3_k_xl | ref + ctx 16384 | 12.7 | 0.114 (0.1..0.122) | 43.5 (43.45..43.56) | 15.66 | 1.54 | 505 |
| qwen3.8-27b@q3_k_xl | ref + FA off | 13.3 | 0.108 (0.1..0.116) | 42.57 (42.47..42.83) | 15.66 | 5.89 | 505 |
| qwen3.8-27b@q3_k_xl | ref + KV q8_0/q8_0 | 14.3 | 0.118 (0.111..0.143) | 43.56 (42.97..44.8) | 15.68 | 0.76 | 505 |
| qwen3.8-27b@q3_k_xl | ref + KV q4_0/q4_0 | 13.0 | 0.113 (0.108..0.118) | 42.47 (42.41..42.53) | 15.66 | 0.66 | 505 |
| qwen3.8-27b@q3_k_xl | ref + threads 8 | 12.7 | 0.109 (0.107..0.164) | 42.95 (41.89..43.25) | 15.68 | 1.08 | 505 |
| qwen3.8-27b@q3_k_xl | ref + eval batch 512 | 12.7 | 0.107 (0.105..0.118) | 43.32 (43.3..43.36) | 15.68 | 1.02 | 505 |
| qwen3.8-27b@q3_k_xl | ref + no mmap, keep in memory | 5.5 | 0.105 (0.1..0.111) | 43.38 (43.27..43.45) | 15.68 | 1.59 | 505 |
| qwen3.8-27b@q3_k_xl | ref + MTP off | 12.5 | 0.1 (0.099..0.114) | 32.69 (32.57..32.7) | 15.01 | 0.19 | 505 |
| qwen3.8-27b@q3_k_xl | ref + MTP draft max 5 | 12.8 | 0.124 (0.104..0.137) | 34.46 (34.33..34.47) | 15.66 | 1.31 | 505 |
| qwen3.8-27b@q3_k_xl | ref + parallel 4 | 14.9 | 0.133 (0.127..0.153) | 31.77 (31.49..31.78) | 15.64 | 2.79 | 505 |
| qwen3-14b | defaults: lms load, no flags, no saved settings | 8.1 | 0.036 (0.033..0.045) | 45.92 (45.88..45.96) | 11.3 | 0.19 | 504 |
| qwen3-14b | --gpu max -c 16384 (first stage-2 runs) | 8.3 | 0.036 (0.036..0.044) | 45.92 (45.91..45.94) | 13.2 | 0.21 | 504 |
| qwen3-14b | ref: offload 1, ctx 8192, parallel 1, FA on, KV f16, MTP default | 8.1 | 0.033 (0.032..0.048) | 50.63 (50.61..50.65) | 10.77 | 0.18 | 504 |
| qwen3-14b | ref + ctx 16384 | 8.3 | 0.032 (0.031..0.047) | 50.62 (50.59..50.64) | 12.03 | 0.19 | 504 |
| qwen3-14b | ref + FA off | 8.3 | 0.038 (0.036..0.045) | 45.9 (45.88..45.94) | 11.3 | 0.19 | 504 |
| qwen3-14b | ref + KV q8_0/q8_0 | 8.0 | 0.04 (0.037..0.052) | 49.21 (49.18..49.22) | 10.18 | 0.18 | 504 |
| qwen3-14b | ref + KV q4_0/q4_0 | 8.1 | 0.036 (0.035..0.04) | 48.02 (47.98..48.03) | 9.87 | 0.18 | 504 |
| qwen3-14b | ref + threads 8 | 8.0 | 0.035 (0.034..0.043) | 50.81 (50.75..50.84) | 10.77 | 0.18 | 504 |
| qwen3-14b | ref + eval batch 512 | 8.0 | 0.045 (0.045..0.046) | 50.64 (50.62..50.66) | 10.77 | 0.18 | 504 |
| qwen3-14b | ref + no mmap, keep in memory | 3.4 | 0.042 (0.034..0.053) | 50.65 (50.64..50.66) | 10.77 | 0.59 | 504 |
| qwen3-14b | ref + MTP off | 8.1 | 0.043 (0.04..0.053) | 50.64 (50.64..50.65) | 10.77 | 0.18 | 504 |
| qwen3-14b | ref + MTP draft max 5 | 0.2 | 0.046 (0.035..0.051) | 50.66 (50.59..50.66) | 10.77 | 0.18 | 504 |
| qwen3-14b | ref + parallel 4 | 8.1 | 0.049 (0.036..0.052) | 50.66 (50.66..50.7) | 10.77 | 0.18 | 504 |
| qwen3.8-27b@q4_k_m | q4km defaults: lms load, no flags, no saved settings | 19.7 | 0.259 (0.257..0.266) | 11.97 (11.93..11.98) | 15.61 | 5.28 | 505 |
| qwen3.8-27b@q4_k_m | q4km ref: offload 1, ctx 8192, parallel 1, FA on, KV f16, MTP default | 16.7 | 0.189 (0.18..0.189) | 14.97 (14.97..14.98) | 15.64 | 4.07 | 505 |
| qwen3.8-27b@q4_k_m | q4km ref + KV q8_0/q8_0 | 15.5 | 0.327 (0.157..0.466) | 13.26 (6.79..14.4) | 15.64 | 3.75 | 505 |
| qwen3.8-27b@q4_k_m | q4km ref + KV q4_0/q4_0 | 18.8 | 0.183 (0.174..0.689) | 16.07 (15.22..16.11) | 15.64 | 3.63 | 505 |
| qwen3.8-27b@q4_k_m | q4km ref + MTP off | 16.9 | 0.152 (0.147..0.164) | 13.69 (13.67..13.69) | 15.66 | 2.45 | 505 |
| qwen3.8-27b@q4_k_m | q4km ref + KV q8_0 + MTP off | 16.2 | 0.154 (0.153..0.155) | 12.35 (12.35..12.35) | 15.48 | 1.9 | 505 |
| qwen3.8-27b@q4_k_m | q4km offload 0.95, ctx 8192, parallel 1, FA on | 16.8 | 0.183 (0.172..0.203) | 16.96 (15.89..17.19) | 15.24 | 3.88 | 505 |
| qwen3.8-27b@q4_k_m | q4km offload 0.9, ctx 8192, parallel 1, FA on | 15.8 | 0.182 (0.181..0.186) | 20.04 (20.02..20.07) | 15.63 | 2.62 | 505 |
| qwen3.8-27b@q4_k_m | q4km offload 0.9 + threads 8 | 15.9 | 0.17 (0.167..0.174) | 20.94 (20.61..20.97) | 15.64 | 2.62 | 505 |
| qwen3.8-27b@q4_k_m | q4km offload 0.85 + threads 8 | 15.8 | 0.165 (0.161..0.185) | 19.93 (19.89..20.84) | 15.64 | 2.01 | 505 |
| qwen3.8-27b@q4_k_m | q4km offload 0.8 + threads 8 | 15.5 | 0.188 (0.178..0.189) | 18.03 (18.02..18.1) | 15.64 | 1.33 | 505 |

## Per-request tables (`python -I tools/aggregate.py raw A B1 B2 D C`)

## Base worlds

| Arm | Seed | Compute adapter | Generate s | Field digest | Statistics |
|---|---|---|---|---|---|
| A | 71077345 | AMD Radeon(TM) Graphics | 13.5 | `b538680075549a70` | land 54.8%, lake 0.1%, mean land 1402 m, max 7054 m, volcanoes hit 17 |
| A | 24601 | AMD Radeon(TM) Graphics | 14.9 | `7889ff8e96330d73` | land 89.5%, lake 0.8%, mean land 1824 m, max 7065 m, volcanoes hit 11 |
| B1 | 71077345 | AMD Radeon(TM) Graphics | 15.3 | `b538680075549a70` | land 54.8%, lake 0.1%, mean land 1402 m, max 7054 m, volcanoes hit 17 |
| B1 | 24601 | AMD Radeon(TM) Graphics | 14.9 | `7889ff8e96330d73` | land 89.5%, lake 0.8%, mean land 1824 m, max 7065 m, volcanoes hit 11 |
| B2 | 71077345 | AMD Radeon(TM) Graphics | 13.4 | `b538680075549a70` | land 54.8%, lake 0.1%, mean land 1402 m, max 7054 m, volcanoes hit 17 |
| B2 | 24601 | AMD Radeon(TM) Graphics | 14.9 | `7889ff8e96330d73` | land 89.5%, lake 0.8%, mean land 1824 m, max 7065 m, volcanoes hit 11 |
| D | 71077345 | AMD Radeon(TM) Graphics | 13.7 | `b538680075549a70` | land 54.8%, lake 0.1%, mean land 1402 m, max 7054 m, volcanoes hit 17 |
| D | 24601 | AMD Radeon(TM) Graphics | 14.8 | `7889ff8e96330d73` | land 89.5%, lake 0.8%, mean land 1824 m, max 7065 m, volcanoes hit 11 |
| C | 71077345 | AMD Radeon RX 7800 XT | 12.7 | `946e53e2bdf443e5` | land 54.8%, lake 0.1%, mean land 1402 m, max 7054 m, volcanoes hit 17 |
| C | 24601 | AMD Radeon RX 7800 XT | 14.2 | `57f82f7c900233e1` | land 89.5%, lake 0.8%, mean land 1824 m, max 7065 m, volcanoes hit 11 |

## Per request

### `no_sea` on seed 71077345: "I don't want any sea, change the map"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| A | proposal, applied | `sea_level` 0.42 -> 0 | - | 1/1 | land 96.1%, lake 3.9%, mean land 1902 m, max 5805 m, volcanoes hit 17 | 1.2 | 4.0 | 2105/126 | yes | yes |
| B1 | proposal, applied | `sea_level` 0.42 -> 0 | - | 1/1 | land 96.1%, lake 3.9%, mean land 1902 m, max 5805 m, volcanoes hit 17 | 4.4 | 15.2 | 2449/436 | yes | yes |
| B2 | proposal, applied | `sea_level` 0.42 -> 0 | - | 1/1 | land 96.1%, lake 3.9%, mean land 1902 m, max 5805 m, volcanoes hit 17 | 2.8 | 8.5 | 2377/113 | yes | yes |
| D | proposal, applied | `sea_level` 0.42 -> 0 | - | 1/1 | land 96.1%, lake 3.9%, mean land 1902 m, max 5805 m, volcanoes hit 17 | 3.7 | 11.2 | 2377/113 | yes | yes |
| C | proposal, applied | `sea_level` 0.42 -> 0 | - | 1/1 | land 96.1%, lake 3.9%, mean land 1902 m, max 5805 m, volcanoes hit 17 | 1.6 | 4.9 | 4151/478 | yes | yes |

### `more_volcanoes` on seed 71077345: "I want more volcanoes"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| A | proposal, applied | `volc.count` 20 -> 30<br>`volc.age` 0.4 -> 0.25 | - | 1/1 | land 56.2%, lake 0.2%, mean land 1384 m, max 7554 m, volcanoes hit 23 | 0.7 | 4.5 | 3226/126 | yes | yes |
| B1 | proposal, applied | `world_structure.hotspot_density` 0.2 -> 0.3<br>`world_structure.enabled` off -> on | - | none scored | land 30.3%, lake 0.0%, mean land 1622 m, max 7094 m, volcanoes hit 14 | 2.1 | 55.4 | 6699/872 | yes | yes |
| B2 | proposal, applied | `volc.count` 20 -> 35 | - | 1/1 | land 54.8%, lake 0.1%, mean land 1376 m, max 6167 m, volcanoes hit 33 | 1.2 | 8.9 | 3495/123 | yes | yes |
| D | proposal, applied | `volc.count` 20 -> 35 | - | 1/1 | land 54.8%, lake 0.1%, mean land 1376 m, max 6167 m, volcanoes hit 33 | 2.0 | 18.3 | 3495/126 | yes | yes |
| C | answered in words | - | - | none scored | - | 1.1 | 8.5 | 9619/803 | - | - |

### `drier_lower` on seed 71077345: "drier climate, lower mountains"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| A | proposal, applied | `climate.equator_temp` 30 -> 32.5<br>`climate.pole_temp` -25 -> -15<br>`climate.lapse_rate` 6.5 -> 8<br>`peak_m` 4000 -> 3000 | - | 1/1 | land 54.8%, lake 0.1%, mean land 1068 m, max 5312 m, volcanoes hit 17 | 0.7 | 5.7 | 2789/203 | yes | yes |
| B1 | proposal, applied | `climate.ocean` 1 -> 0.75<br>`climate.evap` 0.12 -> 0.09<br>`peak_m` 4000 -> 3000 | - | 3/3 | land 54.8%, lake 0.1%, mean land 1068 m, max 5312 m, volcanoes hit 17 | 7.8 | 48.8 | 4593/818 | yes | yes |
| B2 | proposal, applied | `climate.ocean` 1 -> 0.7<br>`climate.evap` 0.12 -> 0.084<br>`peak_m` 4000 -> 2800 | - | 3/3 | land 54.8%, lake 0.1%, mean land 1001 m, max 5029 m, volcanoes hit 17 | 0.6 | 12.7 | 3337/241 | yes | yes |
| D | proposal, applied | `peak_m` 4000 -> 2800<br>`climate.ocean` 1 -> 0.75<br>`climate.evap` 0.12 -> 0.09<br>`climate.rain_dep` 0.35 -> 0.45 | - | 3/3 | land 54.8%, lake 0.1%, mean land 1001 m, max 5030 m, volcanoes hit 17 | 2.1 | 22.6 | 3308/232 | yes | yes |
| C | proposal, applied | `climate.ocean` 1 -> 0.75<br>`climate.evap` 0.12 -> 0.09<br>`climate.rain_dep` 0.35 -> 0.43<br>`peak_m` 4000 -> 3000<br>`tect.alpha` 0.85 -> 0.68 | - | 4/4 | land 54.9%, lake 0.1%, mean land 1011 m, max 5249 m, volcanoes hit 17 | 2.2 | 6.1 | 6597/695 | yes | yes |

### `islands` on seed 71077345: "make it look like a chain of islands"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| A | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.continentality` 0.3 -> 0.7<br>`world_structure.fragmentation` 0.5 -> 0.8<br>`tect.plates` 14 -> 20<br>`passes.coastal` off -> on<br>`passes.glacial` on -> off | - |  | land 69.7%, lake 0.7%, mean land 1440 m, max 4434 m, volcanoes hit 13 | 0.9 | 10.0 | 5755/257 | yes | yes |
| B1 | failed: malformed tool call: arguments are not valid JSON (EOF while parsing a string at line 1 column 548) | - | - |  | - | 8.4 | 53.3 | 892/103 | - | - |
| B2 | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.85<br>`sea_level` 0.42 -> 0.49 | - |  | land 30.2%, lake 0.1%, mean land 1594 m, max 5653 m, volcanoes hit 13 | 1.4 | 10.9 | 2914/209 | yes | yes |
| D | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.95<br>`sea_level` 0.42 -> 0.62 | - |  | land 29.8%, lake 0.2%, mean land 1690 m, max 7687 m, volcanoes hit 10 | 1.9 | 20.2 | 2914/214 | yes | yes |
| C | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.8<br>`world_structure.continentality` 0.3 -> 0.2<br>`world_structure.hotspot_density` 0.2 -> 0.35<br>`world_structure.ocean_depth` 0.6 -> 0.75 | - |  | land 20.0%, lake 0.0%, mean land 1678 m, max 6245 m, volcanoes hit 19 | 1.8 | 5.8 | 5988/644 | yes | yes |

### `imp_river` on seed 71077345: "add a river here"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| A | proposal, applied | `river_density` 1 -> 1.5 | - |  | land 54.8%, lake 0.1%, mean land 1400 m, max 7054 m, volcanoes hit 17 | 0.8 | 3.5 | 2093/123 | yes | yes |
| B1 | proposal, applied | `river_density` 1 -> 1.2 | - |  | land 54.8%, lake 0.1%, mean land 1401 m, max 7054 m, volcanoes hit 17 | 16.4 | 43.6 | 4754/828 | yes | yes |
| B2 | answered in words | - | - |  | - | 0.5 | 24.1 | 4761/392 | - | - |
| D | proposal, applied | `river_density` 1 -> 1.3 | - |  | land 54.8%, lake 0.1%, mean land 1401 m, max 7054 m, volcanoes hit 17 | 1.9 | 27.6 | 4682/181 | yes | yes |
| C | answered in words | - | - |  | - | 3.2 | 6.2 | 1172/572 | - | - |

### `islands` on seed 24601: "make it look like a chain of islands"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| A | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.continentality` 0.3 -> 0.7<br>`world_structure.fragmentation` 0.5 -> 0.8<br>`tect.plates` 14 -> 20<br>`passes.coastal` off -> on<br>`passes.sediment_fill` off -> on | - |  | land 69.6%, lake 0.6%, mean land 1565 m, max 8000 m, volcanoes hit 10 | 0.9 | 9.9 | 5755/251 | yes | yes |
| B1 | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.8<br>`world_structure.continentality` 0.3 -> 0.2 | - |  | land 20.0%, lake 0.2%, mean land 1483 m, max 4269 m, volcanoes hit 10 | 3.8 | 38.1 | 2986/1045 | yes | yes |
| B2 | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.85<br>`sea_level` 0.42 -> 0.49 | - |  | land 30.1%, lake 0.0%, mean land 1403 m, max 4379 m, volcanoes hit 11 | 1.5 | 7.5 | 2914/207 | yes | yes |
| D | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.95<br>`world_structure.continentality` 0.3 -> 0.12 | - |  | land 12.0%, lake 0.1%, mean land 1926 m, max 6978 m, volcanoes hit 16 | 2.1 | 11.2 | 2914/207 | yes | yes |
| C | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.8<br>`world_structure.continentality` 0.3 -> 0.22<br>`world_structure.hotspot_density` 0.2 -> 0.4 | - |  | land 22.1%, lake 0.1%, mean land 1839 m, max 5967 m, volcanoes hit 25 | 1.6 | 7.0 | 6016/803 | yes | yes |

## Model words for requests answered without a proposal, and summaries

- `no_sea` s71077345 **A**: Removed all sea and adjusted the map framing to eliminate any ocean coverage.
- `no_sea` s71077345 **B1**: Lowered sea level to its minimum (0) so the map has as little sea as possible.
- `no_sea` s71077345 **B2**: Lower sea level to minimum so the map is entirely land.
- `no_sea` s71077345 **D**: Lower sea level to minimum so the map is entirely land.
- `no_sea` s71077345 **C**: Set sea level to its minimum (0.0, down from 0.42) to remove as much sea as possible.
- `more_volcanoes` s71077345 **A**: Increased the number of volcanoes and adjusted their age for a more dynamic volcanic landscape.
- `more_volcanoes` s71077345 **B1**: Increase volcano count by ~30% and enable continental steering with a slightly higher hotspot density for extra volcanoes.
- `more_volcanoes` s71077345 **B2**: Increase the number of volcanoes (20 -> 35).
- `more_volcanoes` s71077345 **D**: Increase the number of volcanoes from 20 to 35.
- `more_volcanoes` s71077345 **C**: I can't propose the setting that does this, so I haven't made any changes.  The right control is **Volcanoes** (`volc.count`), which is currently 20. For a modest increase you could raise it to about 26. For a strong one, try about 32 to 40. The setting is in the volcanism group, but my proposal tool only accepts continental-steering keys, so you'd need to set it by hand.  You could also lower **Volcano age** (`volc.age`, currently 0.4) to about 0.3. That makes volcanoes younger, taller and more active. It doesn't add any.  There is a way I could propose changes myself, but I'd advise against 
- `drier_lower` s71077345 **A**: Adjusting climate settings for a drier world and lowering mountain altitudes.
- `drier_lower` s71077345 **B1**: Drier climate (lower ocean supply + evaporation) and lower mountains (peak altitude 4000 → 3000 m).
- `drier_lower` s71077345 **B2**: Drier climate (lower ocean supply + evaporation) and lower mountains (reduced peak altitude).
- `drier_lower` s71077345 **D**: Drier climate (lower ocean supply, evaporation, higher dryness) and ~30% lower mountains.
- `drier_lower` s71077345 **C**: Drier climate (less ocean moisture, less evaporation, earlier rain-out) and lower mountains (lower peak altitude, weaker tectonics).
- `islands` s71077345 **A**: Changes made to create a chain of islands with fragmented landmasses and active coastal processes.
- `islands` s71077345 **B1**: The key knob is fragmentation ("many smaller landmasses and island chains"), but the steering system that drives it is currently off, so I'll enable it first. I'll also nudge continentality up slightly so there's enough total land to read as a chain rather than scattered specks. Note: with steering on, sea level is derived from continentality, so I'm not touching sea level directly.
- `islands` s71077345 **B2**: Enable continental steering, push fragmentation high for many small landmasses/chains, and raise sea level so the result reads as an island chain rather than a continent.
- `islands` s71077345 **D**: Turn on continental steering and push fragmentation high while raising sea level, so the map breaks into a chain of smaller islands rather than one large continent.
- `islands` s71077345 **C**: Turn on continental steering and push it toward a sea-dominated, fragmented, volcanic archipelago.
- `imp_river` s71077345 **A**: Enabled river valley carving and increased river density to create a more detailed river system.
- `imp_river` s71077345 **B1**: Slightly raise river density to encourage one or two additional river channels on regeneration.
- `imp_river` s71077345 **B2**: Here's the honest answer: **I can't add a river at one specific spot.** I don't have any setting that places a river "here" — rivers form from the terrain and rainfall, not from a position on the map. What I *can* do is nudge the global settings so more (or denser) river channels appear overall when you regenerate.  The most direct lever is **River density** (`river_density`), currently at 1.0× — raising it gives more, smaller river channels across the whole map. A modest bump to ~1.3× would add noticeably more rivers without turning everything into a swamp of tiny streams.  A couple of things
- `imp_river` s71077345 **D**: Increase overall river density so more channels appear (placement is seed-dependent).
- `imp_river` s71077345 **C**: I can't add a river at a specific spot. I only change global generation settings, and I can't see or edit your map. Rivers come out of the terrain and rainfall when the world is regenerated, so I can't place one by hand.  I can nudge the world toward more rivers overall. These are the options:  - **River density** (in the `world` group) raises the overall number of rivers. - **Rainfall** (in the `weather` group) makes rivers more likely in dry regions. - **Erosion** (in the `erosion` group) affects how strongly rivers cut valleys.  Which of these do you want?  1. **More rivers everywhere.** I'
- `islands` s24601 **B1**: Enable continental steering, push fragmentation high for island chains, and lower continentality so most of the map is sea.
- `islands` s24601 **B2**: Turn on continental steering with high fragmentation and a higher sea level to break the land into an archipelago / island chain.
- `islands` s24601 **D**: Enable continental steering and push fragmentation high while lowering continentality to produce an archipelago / island-chain look.
- `islands` s24601 **C**: Turn on continental steering, then raise fragmentation, lower continentality and add hotspots. The aim is archipelagos with volcanic island chains.


## Side table: the untuned first runs (`python -I tools/aggregate.py raw B1u B2u`)

## Base worlds

| Arm | Seed | Compute adapter | Generate s | Field digest | Statistics |
|---|---|---|---|---|---|
| B1u | 71077345 | AMD Radeon(TM) Graphics | 13.7 | `b538680075549a70` | land 54.8%, lake 0.1%, mean land 1402 m, max 7054 m, volcanoes hit 17 |
| B1u | 24601 | AMD Radeon(TM) Graphics | 15.0 | `7889ff8e96330d73` | land 89.5%, lake 0.8%, mean land 1824 m, max 7065 m, volcanoes hit 11 |
| B2u | 71077345 | AMD Radeon(TM) Graphics | 14.0 | `b538680075549a70` | land 54.8%, lake 0.1%, mean land 1402 m, max 7054 m, volcanoes hit 17 |
| B2u | 24601 | AMD Radeon(TM) Graphics | 15.0 | `7889ff8e96330d73` | land 89.5%, lake 0.8%, mean land 1824 m, max 7065 m, volcanoes hit 11 |

## Per request

### `no_sea` on seed 71077345: "I don't want any sea, change the map"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| B1u | proposal, applied | `sea_level` 0.42 -> 0 | - | 1/1 | land 96.1%, lake 3.9%, mean land 1902 m, max 5805 m, volcanoes hit 17 | 7.6 | 20.9 | 2449/301 | yes | yes |
| B2u | proposal, applied | `sea_level` 0.42 -> 0 | - | 1/1 | land 96.1%, lake 3.9%, mean land 1902 m, max 5805 m, volcanoes hit 17 | 6.1 | 17.7 | 2377/113 | yes | yes |

### `more_volcanoes` on seed 71077345: "I want more volcanoes"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| B1u | proposal, applied | `volc.count` 20 -> 26 | - | 1/1 | land 54.8%, lake 0.1%, mean land 1400 m, max 7054 m, volcanoes hit 21 | 5.7 | 36.9 | 3567/391 | yes | yes |
| B2u | proposal, applied | `volc.count` 20 -> 35 | - | 1/1 | land 54.8%, lake 0.1%, mean land 1376 m, max 6167 m, volcanoes hit 33 | 3.8 | 20.9 | 3495/123 | yes | yes |

### `drier_lower` on seed 71077345: "drier climate, lower mountains"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| B1u | failed: The model returned no text and no proposal. | - | - | none scored | - | 17.7 | 96.8 | 4593/1288 | - | - |
| B2u | proposal, applied | `peak_m` 4000 -> 2800<br>`climate.ocean` 1 -> 0.75<br>`climate.evap` 0.12 -> 0.09<br>`climate.ocean_hum` 1 -> 0.75 | - | 4/4 | land 54.8%, lake 0.1%, mean land 1001 m, max 5030 m, volcanoes hit 17 | 4.1 | 28.3 | 3308/241 | yes | yes |

### `islands` on seed 71077345: "make it look like a chain of islands"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| B1u | failed: malformed tool call: arguments are not valid JSON (EOF while parsing a string at line 1 column 450) | - | - |  | - | 7.4 | 80.1 | 892/103 | - | - |
| B2u | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.85<br>`world_structure.continentality` 0.3 -> 0.12 | - |  | land 12.1%, lake 0.0%, mean land 1861 m, max 5739 m, volcanoes hit 12 | 4.1 | 25.5 | 2914/212 | yes | yes |

### `imp_river` on seed 71077345: "add a river here"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| B1u | proposal, applied | `river_density` 1 -> 1.2 | - |  | land 54.8%, lake 0.1%, mean land 1401 m, max 7054 m, volcanoes hit 17 | 37.8 | 96.2 | 4754/1228 | yes | yes |
| B2u | answered in words | - | - |  | - | 2.3 | 57.1 | 4759/422 | - | - |

### `islands` on seed 24601: "make it look like a chain of islands"

| Arm | Outcome | Changes proposed (valid) | Refused | Direction (scored keys) | After regenerate | First output s | Total s | Tokens in/out | Undo one step | Base restored |
|---|---|---|---|---|---|---|---|---|---|---|
| B1u | failed: The model returned no text and no proposal. | - | - |  | - | 8.3 | 78.0 | 2986/1130 | - | - |
| B2u | proposal, applied | `world_structure.enabled` off -> on<br>`world_structure.fragmentation` 0.5 -> 0.85<br>`world_structure.continentality` 0.3 -> 0.12 | - |  | land 11.9%, lake 0.0%, mean land 2105 m, max 4540 m, volcanoes hit 9 | 4.4 | 19.2 | 2914/212 | yes | yes |

## Model words for requests answered without a proposal, and summaries

- `no_sea` s71077345 **B1u**: Lower sea level to its minimum so the map is essentially all land.
- `no_sea` s71077345 **B2u**: Lower sea level to minimum so the map is all land.
- `more_volcanoes` s71077345 **B1u**: Increase the number of volcanoes from 20 to 26.
- `more_volcanoes` s71077345 **B2u**: Increase the number of volcanoes (20 → 35).
- `drier_lower` s71077345 **B2u**: Lower mountains (~30%) and a drier climate: reduced ocean rain budget, evaporation and sea-surface humidity.
- `islands` s71077345 **B1u**: I've checked the settings. The key lever is **fragmentation** ("many smaller landmasses and island chains"), but it only works when continental steering is enabled — which is currently off. I'll enable that, push fragmentation up strongly, and trim continentality modestly so there's more sea around the islands.
- `islands` s71077345 **B2u**: Turn on continental steering and push fragmentation high while keeping land scarce, to break the landmass into a chain of islands across mostly sea.
- `imp_river` s71077345 **B1u**: Modestly increase river density (1.0 → 1.2) so more, smaller river channels form on regeneration.
- `imp_river` s71077345 **B2u**: Here's what I found for rivers:  - **River density** (currently 1.0×, range 0.3–3.0) — raising this produces *more, smaller* river channels across the map; lowering it gives fewer, larger ones. - **Channeling / stream.k** (currently 0.012) — raises how deeply and densely river valleys are cut.  A couple of honest caveats before I propose anything:  1. **I can't target a specific spot.** These settings shift rivers across the *whole* map, not at one location you're pointing at. If you want a river in a particular place, that's really down to the seed and terrain — no setting will guarantee it t
- `islands` s24601 **B2u**: Turn on continental steering and push fragmentation high while keeping continentality low, to break land into a chain of islands across mostly-ocean.

