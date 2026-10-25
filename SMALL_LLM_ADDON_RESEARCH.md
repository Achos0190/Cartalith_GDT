# SMALL_LLM_ADDON_RESEARCH.md — a very small Cartalith-tailored model as an optional add-on, no Ollama

**What this is:** research and a proposed milestone plan for the owner's idea (2026-10-26): a *very small* language model, a **separate download/add-on** tailored to Cartalith, that runs **without Ollama**. It is research plus a proposal, not a scope the owner has approved, and carries no status column (progress is `cartalith-native/docs/STATUS.md`). It **defines** milestones and bars; every number is a bar or an estimate, not a result. Evidence tags: **[verified]** opened in the repository; **[cited]** read on the web on 2026-10-26 (secondary sources are marked as such); **[recalled]** from memory, not re-checked, so check before relying; **[judgement]** a labelled recommendation the owner may overrule; **[estimate]** arithmetic from stated inputs. Standing decisions it must not break: `LLM_ASSISTANT_SCOPE.md` s.1 (generation model-free, whole feature off by default, the model proposes and never acts, validator before preview), Rulings DC, DD, DF, DI in `LARGE_ITEM_RULINGS.md`. Read `MISTAKES.md`'s preflight; the rows that bite: *Write "measured" anywhere* (only for a number just produced), *Quote a timing* (median with min..max, run alone), *Attribute a number to a cause*, *Write a test that pins a constant*.

**Short answer.** The task the model must do is narrow (read a short catalogue, call one tool with valid numbers in the right direction, decline what cannot be done), which is exactly what a 1-4B model can be fine-tuned to do, and the evidence we already have says the *failure of weak models here is a protocol failure, not an intelligence failure* (s.3). Two things should come first, before any training: **(1)** measure whether an *unmodified* 4B model already clears the bar (s.9, SL-0), and **(2)** the app already speaks the OpenAI-compatible wire, so the cheapest "no Ollama" runtime is a bundled `llama-server` sidecar (s.2). Fine-tuning is justified only if (1) fails.

## 1. Where the idea fits what exists [verified]

- The app already has a provider-agnostic path: a Rust protocol crate (`crates/cartalith-llm`) over Godot `HTTPClient`, an OpenAI-compatible adapter, a presets list (llama.cpp's `llama-server` is one, port 9931 corrected after a vendor-doc pass per `OUTSTANDING_WORK.md`), the local GPU rule (Ruling DF), a consent screen and a zero-socket-when-off bar. A bundled local server is just another *local* preset; no new adapter is needed (`LLM_ASSISTANT_SCOPE.md` s.2, s.5, s.5a, s.9).
- The parameter task is specified: the two-step flow (`get_param_schema`, then `propose_param_changes` / `set_generation_params`), 114-117 offered keys in nine groups, the strict validator V1-V13, group schemas of at most 12 000 bytes (`LLM_ASSISTANT_SCOPE.md` s.5b). The model is handed each key's current value, default, range and unit, so the work is arithmetic on supplied numbers, not recall of the engine.
- Real comparison data exist: `cartalith-native/docs/evidence/llm-model-eval-2026-10-26/REPORT.md` (LLM-6c stage 1: 7 configurations, 8 prompts x 3, labels only, no `desc`, offline; reproducible with `tools/run_eval.py`).

## 2. Runtimes without Ollama

Ollama itself is a wrapper over llama.cpp [recalled]; "no Ollama" means shipping or embedding an inference engine ourselves. Options:

| Option | Licence | Windows | GPU backends | Android | Fit |
|---|---|---|---|---|---|
| **llama.cpp `llama-server`, bundled as a sidecar process** speaking `/v1/chat/completions` | MIT [recalled; confirm in the repository's LICENSE] | prebuilt Windows binaries are published, CPU and Vulkan builds among them [recalled] | CPU (AVX2), **Vulkan** (works on AMD, including the RX 7800 XT), CUDA, Metal, HIP [recalled] | cannot `exec` a binary from app data on current Android; a native-library packaging route or in-process use is needed [recalled] | **Best for desktop v1.** Zero new Rust or protocol code; supports GBNF/JSON-schema constrained decoding, which turns the V1-V3 failure class into near-zero [recalled; verify the server flags in SL-1] |
| **llama.cpp in-process via a Rust binding** (`llama-cpp-2`, `llama_cpp`, others) | the `llama-cpp-2` crate is listed as Apache-2.0 by an aggregator, the older `llama_cpp` crate as MIT or Apache-2.0 [cited, secondary, not confirmed on crates.io] | the crate builds llama.cpp from source and needs `clang` for `bindgen` [cited]; CMake and a C++ toolchain too [recalled] | Cargo feature flags for CUDA, Metal, Vulkan, OpenCL [cited, secondary] | no Android build instructions found [cited: none]; llama.cpp itself has an Android example [recalled] | **Needed for Android**, and for a single-process desktop later. Cost: C++ in the workspace, bigger `cartalith_godot` DLL, and the project's rule that nothing off the main thread may touch a Godot type (the same `LlmStream` pattern would serve) |
| **candle** (Hugging Face; pure Rust) | Apache-2.0 / MIT dual [recalled] | yes, pure Rust | CUDA and Metal; **no Vulkan or ROCm** [recalled] | CPU only, ARM NEON [recalled] | **Poor fit on the owner's AMD card**: CPU only there; useful only if we insisted on no C++ |
| **mistral.rs** (built on candle) | MIT [cited: two secondary pages] | yes | CUDA, Metal; the search found **no Vulkan** [cited, absence of evidence] | not shown | Same AMD problem as candle; also a larger API than we need |

**Recommendation [judgement]:** desktop: bundle `llama-server` (Vulkan build by default, CPU build as fallback) as an add-on, launched on demand on `127.0.0.1` with a random port and an `--api-key` known to the app, so the existing local-provider path, consent, token counter, cancel and tests are reused. Ruling DF's same-machine GPU rule then has a *known* answer (the app launched it and knows which adapter it asked for), and it needs the owner's attention (Q8). Phone: a second milestone (SL-7) using in-process llama.cpp through a Rust binding or an NDK build; do not promise it.

Process-management duties the sidecar adds (each a bar in SL-1): start on demand, kill on app exit and crash (Windows job object [recalled]), loopback only (no `0.0.0.0`), no key or prompt in logs, port-in-use handling, a visible "model server running" state, and the zero-socket-when-off bar (feature off = no process, no socket).

## 3. What the real comparison tells us (REPORT.md)

Headline table, 5 possible prompts x 3 repetitions (15 runs) and 3 impossible prompts x 3 (9 runs), no `desc`, labels only [verified, copied from the report]:

| Model | Usable proposal | Clean | Direction right | Impossible: declined |
|---|---|---|---|---|
| claude-sonnet-5-5 | 15/15 | 15/15 | 31/31 | 9/9 |
| claude-haiku-5-5 | 15/15 | 15/15 | 30/30 | 3/9 |
| qwen3:14b, thinking **off** | 15/15 | 15/15 | 20/20 | 9/9 |
| qwen3:14b, thinking on | 15/15 | 14/15 (7 invented keys) | 22/27 | 1/9 |
| gpt-oss:20b | 13/15 | 11/15 (invented key) | 19/19 | 4/9 |
| Ministral-3-14B Q5_K_M | 3/15 | 3/15 | 8/8 | 7/9 |
| Helcyon-Mythic-14B Q5_K_M | 0/15 | 0/15 | n/a | 9/9 (never called a tool) |

What it supports, and what it does not:

1. **The two weakest models failed at the protocol.** Ministral usually wrote a correct proposal *in prose* ("set sea level to 0.0 ... Would you like me to proceed?") instead of calling the tool (12 of 15 possible-request runs); Helcyon, a writer fine-tune, never called a tool in 24 runs and invented its own schemas and keys. This is the failure a **narrow tool-calling fine-tune fixes directly**, because it is a matter of format and habit, not knowledge.
2. **A 14B model with thinking off was perfect on this sample**, at about 3 s median per request and about 50 tok/s on this machine. Thinking made things worse (invented keys, wrong directions, acted on impossible requests). So the target is a *non-thinking* tool-caller; a 1-4B model needs to match qwen3:14b#nothink, not Claude.
3. **Where models invented keys, the validator caught it** (all 12 rejections were V5). A fine-tune should reduce invented keys further, but the validator remains the safety layer, not the model.
4. **Claude is a strong teacher** on this task: valid, in-range, right-direction proposals on every possible request, and Sonnet declined or asked on every impossible one. Haiku proposed a "nearby" whole-world change on 6 of 9 impossible requests, so for *negative* examples use Sonnet, or filter Haiku's.
5. **Limits:** n=3 per cell, one run of Claude, direction labels are one agent's reading of `GENERATION_PARAMETERS.md`, no model was shown `desc`, and the 20-request x 5 stage-2 run (LLM-6c, in the app) has not been done. A 4B model has not been tested at all; **the report contains no evidence about any model under 14B.** Everything in this document about small-model quality is therefore hypothesis until SL-0.

## 4. Candidate base models (1-4B)

Licence matters because the add-on is a *redistributed derivative* (fine-tuned weights shipped in an installer). Prefer permissive licences with no use-field restrictions.

| Model | Size | Licence | Notes |
|---|---|---|---|
| **Qwen3-4B** (and -1.7B, -0.6B) | 4B / 1.7B / 0.6B | Apache-2.0 for the family [cited indirectly, secondary pages; confirm on each model card] | Has a tool-calling chat template and a non-thinking mode [recalled]; qwen3:14b in the report is the same family, so the comparison is like-for-like. **Primary candidate** |
| **Phi-4-mini-instruct** | 3.8B | MIT [cited, secondary] | Strong small reasoner with function calling [recalled]; permissive; good ablation partner |
| **SmolLM3-3B** | 3B | Apache-2.0 [cited, secondary] | Fully open recipe, long context [cited, secondary]; tool use less proven [recalled] |
| Llama 3.2 3B | 3B | Llama 3.2 Community License: not OSI, attribution and naming conditions on derivatives, a 700M-MAU clause [cited, secondary] | Avoid unless nothing else works |
| Gemma 3 4B | 4B | Gemma Terms of Use (use restrictions that must be passed on) [recalled] | Avoid for redistribution; confirm before any use |
| Ministral 3 3B | 3B | not checked (the report used the 14B; the family may be Apache-2.0) [recalled, unverified] | Re-check; the 14B failed the protocol, which says nothing about a tuned 3B |

Each model's licence file is copied into the add-on, with its notice, and recorded in `REFERENCES.md`/`PROVENANCE.md` before any weights ship (bar in SL-5).

## 5. Fine-tuning on the narrow task

**Goal.** Teach a protocol and a habit, not facts: given the system prompt skeleton (`LLM_ASSISTANT_SCOPE.md` s.5b.6) and a user request, (1) call `get_param_schema` with at most three valid group names; (2) after the schema arrives, call `propose_param_changes` once, only with keys that were shown, values inside `[min, max]`, integers whole, at most 12 changes, each with a short reason; (3) when the request cannot be done with the offered keys, **make no change** and say so or ask; (4) answer read-only questions via the read-only tools (LLM-8) later.

**Key design point: put the catalogue in the context, vary it in training.** The runtime prompt carries each key's name, range, unit, current, default and (after LLM-6a) `desc`. If training data always used today's 114 keys at default values, the model would memorise them and break when a parameter is added. So generate training prompts from *randomised catalogues*: random subsets of groups, current values drawn from the range, ranges and defaults jittered, a fraction of keys renamed or added, so the only way to score is to read the schema in front of it. This also means the catalogue growing (Ruling DJ added three keys the day after the 117 count) does not by itself force a retrain; a schema-format or tool-contract change does (s.8).

**Synthetic data, generated by Claude and filtered by the app's own validator [judgement, plan]:**

1. **Requests.** A seed list of intents per group written by hand plus paraphrases generated by Claude: modest vs strong ("a bit drier", "much colder"), compound ("drier and lower mountains"), vague, misspelled, non-English phrasings, impossible ("delete the Pacific", "add a river here", "make it prettier", "change the seed"), questions, and injection-shaped strings (LLM-9's corpus, so the model learns to treat data as data). Target mix [judgement]: 55% possible single/compound, 15% impossible/decline, 10% ambiguous (ask), 10% already-at-limit/no-op, 10% injection or off-topic.
2. **Teacher.** Claude (Sonnet for the labels, since Haiku proposed nearby changes on impossible requests) is run through the *real two-step flow* against the randomised catalogue and produces full multi-turn traces including tool calls. The existing `tools/run_eval.py` and `tools/catalogue.py` are the starting harness [verified].
3. **Filters (hard).** Keep a trace only if: the strict validator accepts it with zero rejected changes (V1-V13); the number of changes is at most 12; every value is a *distinct and plausible* size for the stated strength (modest 15-30%, strong 40-60%, per the prompt skeleton); the **direction** matches the labelled expected sign for the request; the impossible-class traces made no change. Reject-and-log everything else, and keep the rejects as **negative examples for a preference/DPO stage** only if SFT is not enough.
4. **Direction labels are the weak link.** The report's labels are one agent's reading of `GENERATION_PARAMETERS.md`. They must be reviewed by the owner or a second pass (and reconciled with `desc` once LLM-6a lands) before they gate training data; a wrong label would teach the wrong direction at scale.
5. **Size.** 3 000-8 000 filtered traces of about 1.5-2.5k tokens each, i.e. roughly 5-20M tokens per epoch [estimate]. Hold out a **disjoint test set**: different catalogues and phrasings from training, plus the 20 scripted LLM-6c requests, never trained on.
6. **Method.** Supervised fine-tuning with LoRA/QLoRA on the chosen base, completion-only loss on the assistant turns and tool calls, thinking disabled in the template; merge and export to GGUF with Q4_K_M (and Q5/Q8 as comparison) [recalled, standard toolchain]. If SFT leaves invented keys, add constrained decoding at inference (GBNF from the group's key enum) rather than more training.
7. **Privacy and content rules.** No private Vault text, no map images (Ruling DI P5), no user data in training. All inputs are synthetic and generated from the registry.

**Compute [estimate, cloud]:** QLoRA on a 4B model with 10M tokens per epoch fits one 24-48 GB GPU in roughly a few hours per epoch; three epochs plus ablations over 2-3 base models is on the order of tens of GPU-hours, i.e. tens of dollars at typical cloud rates [judgement, rates vary and are not checked]. The owner's RX 7800 XT (16 GB) is not the right trainer (ROCm on Windows is limited [recalled]); the card is for *inference* testing. Teacher cost: 5 000 traces x ~3 turns x ~4k tokens in plus ~0.6k out is tens of millions of input tokens [estimate]; the owner's Ruling DD Q8 (no enforced cap) does not apply to an offline data run, so **set a cap on this run** (Q6).

## 6. Size and speed estimates

Inputs: the report measured **about 50 tok/s for 14B Q4/Q5 models on this machine** (Ryzen 7 9800X3D, RX 7800 XT 16 GB, 31 GB RAM; Ollama GPU, `qwen3:14b` 100% GPU at context 16 384) and 95 tok/s for gpt-oss:20b (MoE) [verified in REPORT.md]. Decode speed is memory-bandwidth bound [recalled], so it scales inversely with weight size.

| Config | Weights (Q4_K_M) | Desktop GPU (Vulkan) | Desktop CPU only | Phone (modern Snapdragon-class, 12 GB) | OnePlus 6T (Snapdragon 845, 6-8 GB) |
|---|---|---|---|---|---|
| 4B | about 2.5 GB [estimate: 4B x 4.8 bits/8] | about 120-180 tok/s [estimate: 14B at about 9 GB gives 50 tok/s, scale by size, discount fixed overhead] | about 20-35 tok/s [estimate; DDR5 dual channel] | about 5-12 tok/s [judgement] | too slow to be pleasant (about 1-3 tok/s) [judgement] |
| 1.7B | about 1.1 GB | about 200+ tok/s (launch-bound) | about 40-70 tok/s | about 12-25 tok/s | about 3-6 tok/s |
| 0.6B | about 0.4 GB | latency-bound | about 100+ tok/s | about 25-50 tok/s | about 8-15 tok/s |

What a user feels is *prompt processing plus about 150-400 output tokens*: a two-step request with a 2-3k token prompt [estimate from the report's prompts under about 2 100 tokens] is a few seconds on the desktop GPU and tens of seconds on CPU only [judgement]. **Memory:** with the context at 4 096-8 192 tokens the KV cache adds a few hundred MB for a 4B model [estimate]. **Contention with the app:** the model and the map renderer share the GPU; Ruling DF's rule and a CPU-only preset (the 9800X3D has headroom) are the answers; the add-on should default to Vulkan on a discrete card only when the user has not enabled the heavy 3D view [Q8]. These are estimates; SL-1 measures the real figures (median with min..max, model server run alone).

**Download size:** 4B Q4_K_M about 2.5 GB, 1.7B about 1.1 GB, plus the server binary (tens of MB) [estimate]. A separate download keeps the installer unchanged; both can sit behind the same consent screen.

## 7. Packaging and versioning against the parameter catalogue

- **Add-on = a folder** holding `manifest.json` (name, base model, licence name and file, quantisation, SHA-256 of the weights, size, context length, **tool-contract version**, **catalogue-schema version**, training-set id, build date), the weights, the licence/notice files and the server binary build(s). The app verifies the hash before first launch and shows the manifest on the consent screen (a consent text must be true; `LLM_ASSISTANT_SCOPE.md` s.4 item 6).
- **Two versions, kept separate:** `tool_contract` (the shape of `get_param_schema` / `propose_param_changes` and the system prompt skeleton) is what the *model* is trained on; `catalogue_schema` (the entry format) is what the *prompt* is built from. A catalogue that merely gains keys does **not** change either; renaming a field or the tool's name does. The app refuses to enable the add-on on a mismatch and says why (the `CommandIndex` rule: a reason, not a dead button).
- **Provenance:** the add-on records the registry commit and the `PARAMS` count it was trained against; a test fails the build when `PARAMS`' group list changes and the add-on's manifest is not updated or explicitly marked compatible.
- **Hosting/delivery** (Q4): a URL the app downloads from with hash check, versus the owner placing files manually. Recommend manual placement plus a "browse for add-on folder" in SL-3 first; automatic download is a network surface that needs its own consent, so it is deferred.
- **Never in a save, export or archive:** the add-on location is per-device settings, like the quick bar (Ruling DH Q7).

## 8. Risks

1. **Premature fine-tuning:** an unmodified Qwen3-4B with thinking off may already pass; training would then add maintenance for no gain. Mitigation: SL-0 is a go/no-go gate.
2. **Overfitting to today's catalogue and prompts** (a model that memorised keys). Mitigation: randomised catalogues, a held-out catalogue test.
3. **Label error** (wrong direction taught). Mitigation: owner review of the labelled sets; a sampled hand audit of 100 accepted traces (bar in SL-2).
4. **A small model is more injectable** (LLM-9). Mitigation: the validator and closed tool set remain the defence; the injection mix in training is a mitigation, not a guarantee; LLM-9's corpus is run against the add-on.
5. **Licence mistakes** on redistribution (Llama/Gemma terms, derivative naming). Mitigation: permissive base only, notices shipped, recorded in `REFERENCES.md`.
6. **Sidecar process hazards:** orphaned server, port collision, antivirus flags on a bundled `.exe` [recalled], key leakage in command line (use a stdin/env-var key, not a visible argument). Each a bar in SL-1.
7. **GPU contention** with the renderer on one card. Mitigation: Ruling DF plus a CPU option.
8. **Android scope creep:** in-process inference, memory pressure and thermals on a phone are the largest unknowns; SL-7 is explicitly optional.
9. **Reproducibility drift:** training depends on a hosted teacher whose behaviour changes. Mitigation: store the filtered dataset (it is synthetic and releasable) and the harness commit with the add-on's manifest.
10. **Value risk:** if the owner mostly uses hosted Claude or a local 14B, the add-on's benefit is "works with nothing installed", not "better answers". Say that plainly in the consent text.

## 9. Milestones and the go/no-go gate

Effort: S about 1 day, M 2-4 days, L 1-2 weeks, one lane. Numbers are bars; none has been measured.

| ID | Milestone | Verifiable bar | Effort |
|---|---|---|---|
| **SL-0** | **Baseline: unmodified 1.7B and 4B models in the existing harness**, then the **stage-2 in-app evaluation** (LLM-6c: 20 scripted requests x 5 runs, through the real panel, validator and `desc`-bearing catalogue) | A dated "tested with" table for Qwen3-0.6B/1.7B/4B, Phi-4-mini and SmolLM3-3B (thinking off, same prompts as the report, Q4_K_M): usable-proposal rate, clean rate, direction agreement, impossible-declined rate, invented-key count, median with min..max latency and tok/s with the server alone on the machine; reproducible from one command; the 14B qwen3 thinking-off row is re-run beside them as the control. No pass rate is invented in advance (LLM-6c rule); see the gate below | M |
| **SL-1** | `llama-server` sidecar as a bundled local preset (desktop) | Start on demand, loopback only, key not in argv/logs (grep test with a leak mutation); killed on app exit and after a forced app kill within 5 s (job-object test); feature off = 0 processes and 0 sockets over a full session (socket-open check); cancel stops generation and closes the socket (LLM-3 bar re-run); first-token and tok/s on the owner's machine measured for GPU (Vulkan) and CPU builds, median with min..max, at 4B Q4_K_M; constrained-decoding flag verified to reject an invented key (mutation: disable it, test red) | M |
| **SL-2** | Data generation and filtering pipeline | At least 5 000 accepted traces, each passing validator V1-V13; 0 invented keys, 0 out-of-range, 0 fractional ints, 0 duplicates in the accepted set (asserted, not assumed); mix within 5 points of the stated proportions; held-out catalogue and phrasing sets share no catalogue seed with training (a test checks the seeds are disjoint); owner-reviewed direction labels for every request template; a hand audit of 100 random accepted traces finds at most 2 wrong directions; the dataset and harness commit are stored | L |
| **SL-3** | Fine-tune, quantise, evaluate against the held-out set and the 20 x 5 | Per base model: held-out usable-proposal rate, clean rate, direction agreement, impossible-declined rate, reported next to the SL-0 base numbers and the qwen3:14b control; the **gate bars** below are computed by script; GGUF loads in the SL-1 server and passes the LLM-1 mock-wire tests; 3 seeds of training reported with the spread | L |
| **SL-4** | Add-on packaging, manifest and version gate | Manifest hash verified (mutation: flip one weight byte, enable refuses with the reason); tool-contract or catalogue-schema mismatch refuses with a reason (mutation: bump either); a test that the manifest lists the licence file and that it exists; a build test fails when `PARAMS` groups change and the manifest is stale | M |
| **SL-5** | Licence, notice and consent text | Base model's licence and the training-data statement appear on the consent screen and in the add-on folder; `REFERENCES.md` and `PROVENANCE.md` rows exist; the consent says plainly that it is a small local model, its measured limits (from SL-3) and that the validator and click-to-apply still apply | S |
| **SL-6** | In-app acceptance | The LLM-6b card works end to end with the add-on on the owner's machine: 20 requests x 5 runs, all validator verdicts logged; LLM-9's hostile corpus (at least 30 strings, 10 parameter-shaped) produces 0 valid out-of-scope proposals; the model server uses a measured amount of GPU/CPU while the map is panned (median with min..max, stated) | M |
| **SL-7** | Optional: phone (in-process llama.cpp) | Only after SL-6 and LLM-10. 1.7B Q4 loads on the OnePlus 6T build within a stated memory budget, one proposal completes, and the thermal/battery cost of 10 requests is measured and reported; if it cannot, SL-7 is recorded as not feasible with numbers | L |

**Go/no-go gate after SL-0 (stage-2 in-app evaluation).** The LLM-6c rule is that the owner sets the pass rate after seeing numbers; below are the **proposed** bars to put to the owner [judgement], defined so each is computed by script:

- **No-go for fine-tuning (use a recommended base model, no training)** if the best unmodified model of 4B or less, thinking off, reaches on the 20 x 5 run: usable proposal on possible requests >= 90%; clean (no rejected change) >= 95% of proposals; direction agreement >= 90% on scored keys; impossible requests with no change proposed >= 80%; and invented keys <= 1 in 100 proposals. Ship it as a recommended GGUF and the SL-1 sidecar only.
- **Go for fine-tuning** if the best base model misses one or more of those bars but is within 25 points of the qwen3:14b thinking-off control on usable-proposal rate (i.e. the capacity is there and the gap is protocol/format), which is the pattern the report showed for Ministral and Helcyon.
- **Stop** if the best 4B model, after SL-3, still misses the same bars by more than 10 points: recommend a 7-14B local model (the existing path) instead and record the result.
- **Add-on is worth shipping** only if SL-3's tuned model meets the bars above on the held-out *and* the 20 x 5 sets, with the tuned model beating its own base on at least three of the five metrics.

## 10. Owner questions (each with a recommended default; none blocks SL-0)

- **Q1. Do SL-0 first, before any training?** Recommended **yes**: it costs a day or two and may remove the need for fine-tuning.
- **Q2. Base model?** Recommended **Qwen3-4B** primary (Apache-2.0 [verify on the model card], same family as the control), Qwen3-1.7B for weak machines and phones, Phi-4-mini (MIT) as the ablation. Not Llama or Gemma.
- **Q3. Runtime?** Recommended **bundled `llama-server` sidecar** for desktop (reuses the whole provider path); in-process llama.cpp only for Android later.
- **Q4. Delivery?** Recommended **owner places the folder and the app verifies it**; an in-app downloader later, with its own consent.
- **Q5. Who reviews the direction labels?** Recommended **the owner reviews the template list once** (a few dozen lines), since a wrong label poisons thousands of traces.
- **Q6. Teacher budget?** Recommended a **fixed cap on the data-generation run** (the no-cap ruling DD Q8 was for the user's interactive assistant, not an offline job), figure chosen by the owner after SL-2's dry run reports token counts.
- **Q7. Is the 1.7B tier worth building?** Recommended **only if SL-0 shows it passes the no-go bars or SL-3 does**; otherwise drop it and keep 4B.
- **Q8. GPU default for the add-on?** Recommended **Vulkan on the discrete card when the app is not under heavy 3D load, CPU build as a switch**, since the app launches the server and can apply Ruling DF with certainty; confirm the owner is happy with that default.
- **Q9. Open-source the dataset and harness with the add-on?** Recommended **yes** (synthetic, no private data), which also answers the reproducibility risk.
- **Q10. Scope of the add-on's capability?** Recommended **parameters and read-only questions only** (LLM-6, LLM-8); prose/Vault assist stays with larger models because a 4B model's prose is a different quality bar, and edit actions (LLM-7) stay out of this add-on until LLM-5 exists.

## 11. References

- `LLM_ASSISTANT_SCOPE.md`, `LLM_INTEGRATION_RESEARCH.md`, `cartalith-native/docs/evidence/llm-model-eval-2026-10-26/REPORT.md` (in this repository).
- `llama-cpp-2` and `llama_cpp` crate pages on docs.rs (search results of 2026-10-26: licence and feature flags secondary; Android absent); mistral.rs and candle (search of 2026-10-26: MIT for mistral.rs, GPU list without Vulkan); small-model licence summaries from third-party comparison pages (search of 2026-10-26). **None of these were opened on the primary repositories**; treat every licence line as needing confirmation against the licence file in the model or code repository before shipping.
