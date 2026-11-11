# SMALL_LLM_ADDON_RESEARCH.md — a very small Cartalith-tailored model as an optional add-on, no Ollama

**What this is:** research and a proposed milestone plan for the owner's idea (2026-10-26): a *very small* language model, a **separate download/add-on** tailored to Cartalith, that runs **without Ollama**. **Not scheduled by Ruling DK (2026-10-25: "Not now"); reopened as a request by the owner on 2026-11-11, so Part B (after section 10 below) adds the update, the hardware reality, the dataset, integration options, the evaluation gate and a course outline. Where Part A and Part B disagree, Part B is newer; its section 11 lists what it overrides.** The research stays on file, nothing in it is scheduled, and no SL milestone is open; the owner will adopt the go/no-go gate pass rates after seeing SL-0 numbers, so section 9's bars remain proposals. It is research plus a proposal, not a scope the owner has approved, and carries no status column (progress is `cartalith-native/docs/STATUS.md`). It **defines** milestones and bars; every number is a bar or an estimate, not a result. Evidence tags: **[verified]** opened in the repository; **[cited]** read on the web on 2026-10-26 (secondary sources are marked as such); **[recalled]** from memory, not re-checked, so check before relying; **[judgement]** a labelled recommendation the owner may overrule; **[estimate]** arithmetic from stated inputs. Standing decisions it must not break: `LLM_ASSISTANT_SCOPE.md` s.1 (generation model-free, whole feature off by default, the model proposes and never acts, validator before preview), Rulings DC, DD, DF, DI in `LARGE_ITEM_RULINGS.md`. Read `MISTAKES.md`'s preflight; the rows that bite: *Write "measured" anywhere* (only for a number just produced), *Quote a timing* (median with min..max, run alone), *Attribute a number to a cause*, *Write a test that pins a constant*.

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
| **Qwen3-4B** (and -1.7B, -0.6B) *(Part B s.11 and s.14: prefer Qwen3.5-4B first; there is no small Qwen3.8)* | 4B / 1.7B / 0.6B | Apache-2.0 for the family [cited indirectly, secondary pages; confirm on each model card] | Has a tool-calling chat template and a non-thinking mode [recalled]; qwen3:14b in the report is the same family, so the comparison is like-for-like. **Primary candidate** |
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

**Compute [estimate, cloud]:** QLoRA on a 4B model with 10M tokens per epoch fits one 24-48 GB GPU in roughly a few hours per epoch; three epochs plus ablations over 2-3 base models is on the order of tens of GPU-hours, i.e. tens of dollars at typical cloud rates [judgement, rates vary and are not checked]. *(Superseded by Part B s.12: current vendor sources list the RX 7800 XT for ROCm training stacks, so the card is the first trainer to try, cloud the fallback.)* Teacher cost: 5 000 traces x ~3 turns x ~4k tokens in plus ~0.6k out is tens of millions of input tokens [estimate]; the owner's Ruling DD Q8 (no enforced cap) does not apply to an offline data run, so **set a cap on this run** (Q6).

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

## 10a. References (Part A)

- `LLM_ASSISTANT_SCOPE.md`, `LLM_INTEGRATION_RESEARCH.md`, `cartalith-native/docs/evidence/llm-model-eval-2026-10-26/REPORT.md` (in this repository).
- `llama-cpp-2` and `llama_cpp` crate pages on docs.rs (search results of 2026-10-26: licence and feature flags secondary; Android absent); mistral.rs and candle (search of 2026-10-26: MIT for mistral.rs, GPU list without Vulkan); small-model licence summaries from third-party comparison pages (search of 2026-10-26). **None of these were opened on the primary repositories**; treat every licence line as needing confirmation against the licence file in the model or code repository before shipping.

---

# Part B. 2026-11-11 update: the owner's request, and a course outline

**Why this part exists.** On 2026-11-11 the owner asked (`OUTSTANDING_WORK.md`, the row *A Cartalith-specific small LLM*): *"training a custom llm (qwen 3.8 or similar) so it can be smaller, good at writing prose for villages but also excellent at handling cartalith functions ... a course outline on the tools we need and how we could go about it on my own local hardware ... a cartalith specific llm and see how we integrate it. Directly via cartalith or as current with a server."* **That request reopens what Ruling DK's "Not now" parked.** Nothing here is scheduled until the owner answers section 18; this part is research and a proposal, documentation only. No training was run, nothing was installed or downloaded, and LM Studio was not touched in producing it. Evidence tags are as in the header, plus **[cited 2026-11-11]** (a page opened or searched on that date; secondary sources named as such) and **[estimate]** (arithmetic from stated inputs, not a result).

## 11. What changed since 2026-10-26, and what Part A now gets wrong

1. **"Qwen3.8" has no small sizes.** The Hugging Face card for `Qwen/Qwen3.8-27B` lists Apache-2.0, 27B dense (28B tensors), 64 layers of a Gated DeltaNet / Gated Attention hybrid, 262 144 native context, August 2026, and no smaller sibling [cited 2026-11-11, primary model card]. The 2.4T flagship is under a different "Max" licence, and Qwen3.8-Flash-Next under "Qwen Community 1.0", not Apache-2.0 [cited, secondary news pages, unconfirmed on primary]. So "qwen 3.8 ... so it can be smaller" cannot be taken literally: **the small Qwen line is Qwen3.5 (0.8B, 2B, 4B, 9B, 2026-03-02; Apache-2.0 on the 4B and 9B model cards)** [cited, primary cards for 4B and 9B; the 0.8B and 2B licences not opened] and the older dense Qwen3 (0.6B to 14B). Part A's "Primary candidate Qwen3-4B" (section 4) should read **Qwen3.5-4B first, Qwen3-4B as the conservative fallback**, reasons in section 14.
2. **The training card Part A dismissed is now plausibly usable.** Part A section 5 said the RX 7800 XT is "not the right trainer (ROCm on Windows is limited [recalled])". Current sources say otherwise (section 12), so cloud training is the fallback, not the plan. The recalled claim is withdrawn; its replacement is cited, not measured on this machine.
3. **The local assistant baseline moved.** Part A's table compared 14B models on parameters only. The repository now holds AC-4's score of the catalogue path (qwen3-14b: 7/30 without catalogue help, 11 with the pack, 18 with pre-retrieval, 17 with both) and a 27B arm; section 16 uses them as the regression floor.
4. **The goal widened.** The owner now wants village prose in the same model. Part A section 10 Q10 recommended parameters and read-only questions only. Section 17 proposes **two adapters on one base** instead of forcing one adapter to carry both.
5. **llama.cpp no longer fine-tunes.** The `finetune` and `train-text-from-scratch` examples were removed upstream, `llama-finetune` was reported not working since its first version (issue #18499, 2025-12-30), and a vendor fork (Tether QVAC) carries LoRA training it says it will upstream [cited, secondary; mirror of 2025-11; current mainline not opened]. llama.cpp remains the **converter, quantiser and runtime**, not the trainer.
6. **Ruling DK's reopening is the owner's to record.** This file does not edit `LARGE_ITEM_RULINGS.md`; if the owner adopts a path, a new ruling should say so.

## 12. Hardware reality: the owner's PC (Ryzen 7 9800X3D, 31 GB RAM, RX 7800 XT 16 GB gfx1101, Windows 11, LM Studio on ROCm)

### 12.1 What the sources say about the stack (all [cited 2026-11-11]; nothing here was run on this machine)

| Question | What was found | Confidence |
|---|---|---|
| ROCm on **WSL2** | AMD's `radeon-ryzen` WSL compatibility page lists "Radeon RX 7800 XT"; its newest row pairs **PyTorch 2.9.1 on ROCm 7.2.1** as official production support (the 6.4.2 matrix: PyTorch 2.6.0 on ROCm 6.4.2). The PyTorch 2.7 stable row is marked unsupported for the 7000 series in both matrices. AMD notes a "VaMgr" message may appear in WSL2 PyTorch | primary AMD docs, via search extract |
| ROCm on **native Windows**, gfx1101 | ROCm Core SDK 7.10.0 preview notes list the RX 7800 XT and show gfx1101 on Windows 11 25H2; the Windows HIP SDK 7.1.1 requirements mark gfx1101 supported for runtime and HIP SDK. The "Windows PyTorch 7.2" release-note snippet listed the RX 7900 XTX and RX 9070 series and **did not show the 7800 XT; it may be a partial list, so a native Windows PyTorch wheel for gfx1101 is unconfirmed** | primary AMD docs, ambiguous extract |
| `HSA_OVERRIDE_GFX_VERSION` | Not needed where gfx1101 is officially listed. Setting it to `11.0.0` so a gfx1101 card runs gfx1100 kernels is **Linux** folklore [recalled]; no source opened says Windows honours it. A last-resort try on Linux/WSL2 only if PyTorch sees no usable device | recalled, unconfirmed |
| **Unsloth** on AMD | Unsloth's AMD page lists RDNA 3, 3.5 and 4 on Windows and Linux; AMD's own 2026 article (`amd.com/en/developer/resources/technical-articles/2026/train-and-run-models-on-amd-gpus-with-unsloth.html`) lists gfx1100, gfx1101 and gfx1102 as **full** on Windows, WSL and Linux. Installers: `irm https://unsloth.ai/install.ps1 | iex` (Windows) and `curl -fsSL https://unsloth.ai/install.sh | sh` (Linux). Flash-Attention 2 is not installed by default (PyTorch SDPA is used); `xformers` and PyPI `triton` are excluded; AMD distributed training is Linux-only (irrelevant to one card). **The Unsloth page is undated.** One third-party guide calls Windows "far less predictable than Linux" (a single blog's opinion) | vendor docs, undated; one dissenting secondary |
| **bitsandbytes** (the 4-bit of QLoRA) on ROCm | Unsloth's AMD page says all ROCm systems need a **pre-release bitsandbytes**: versions up to 0.49.2 have a **4-bit decode NaN bug on every AMD GPU**. A community project ships a Windows ROCm 7 bitsandbytes build. bitsandbytes' own release notes for RDNA3 were not opened | vendor doc plus community |
| **Flash-attention** ROCm | Not needed: SDPA is the documented default. A community repo packages ROCm 7 flash/sage attention for Windows; do not depend on it | community only |
| Overall maturity | A 2026 dev.to piece ("ROCm in 2026: why PyTorch on the RX 7900 XTX still falls short for research") and a later Unsloth guide describe the stack as fragile, failing mostly on **version mismatches** | secondary |

### 12.2 Where to train: ranked [judgement]

| Option | For | Against | Verdict |
|---|---|---|---|
| **A. Unsloth native on Windows** | one command, no second OS, the Windows ROCm driver LM Studio already uses; AMD lists the card as full | newest, least predictable; the Windows PyTorch wheel for gfx1101 is unconfirmed from AMD's own release note; failures are hard to diagnose | **Try first, time-boxed to one evening (L3)** |
| **B. WSL2 (Ubuntu 24.04) + AMD's WSL ROCm 7.2.1 + PyTorch 2.9.1**, then Unsloth or TRL/PEFT | the card is on AMD's official WSL list with a named PyTorch; Linux wheels are best trodden | disk image and `.wslconfig` memory limit (the 31 GB is shared); the "VaMgr" message; Windows and WSL share the one card | **The fallback if A fails L3** |
| **C. Native Linux dual-boot** | broadest support, no virtualisation | a partition, a reboot per session, nothing else (LM Studio, Godot) in that boot | only if A and B fail; no source found needing it for this card |
| **D. Rented cloud GPU** (one 24 to 48 GB card) | no ROCm risk; larger models | cost; data is synthetic so privacy is not the issue | Part A's estimate stands: **tens of dollars**; the escape hatch, and the route for any 8B or larger run |

**16 GB of VRAM cannot be shared between LM Studio's loaded model and a training run.** The owner decides when to unload in LM Studio; this document and its agents will not.

### 12.3 What fine-tuning is feasible on 16 GB (estimates, not results)

Inputs: the Unsloth Qwen3.5 fine-tune page, as mirrored by secondary sites, gives **bf16 LoRA about 10 GB for 4B and about 22 GB for 9B**, full fine-tuning about 4x LoRA, and advises against 4-bit QLoRA for Qwen3.5 (larger quantisation error), though other tutorials still use it [cited, secondary mirrors; the English vendor page was not opened, verify there]. Training arithmetic: LoRA costs about 4 x N x T floating-point operations (forward plus activation backward; the frozen base takes no weight gradient) and about a third more with gradient checkpointing [estimate, rule of thumb]; the RX 7800 XT's fp16 peak is of the order of 37 TFLOP/s dense and consumer utilisation of 10% to 20% is typical [estimate; spec recalled]. Hence:

| Base | Method | VRAM | 10M tokens (one epoch of the planned set) | Verdict |
|---|---|---|---|---|
| Qwen3.5-0.8B / 2B, Qwen3-1.7B | bf16 LoRA | about 4 to 8 GB [estimate] | about 1 to 4 h | feasible; the phone tier |
| **Qwen3.5-4B / Qwen3-4B** | bf16 LoRA, sequence 3 to 4k, batch 1 plus accumulation | **about 10 GB [cited, secondary]**, 6 GB headroom | **about 8 to 16 h** [estimate: 4 x 4e9 x 1e7 x 1.33 = 2.1e17 FLOP at 7e12 to 4e12 FLOP/s] | **feasible: the recommended size; plan overnight runs** |
| Qwen3-8B | QLoRA (bf16 will not fit) | about 8 to 11 GB [estimate] | about 1 to 2 days | feasible but slow, with NaN risk and an accuracy cost; only if 4B fails the gate |
| Qwen3.5-9B | bf16 LoRA about 22 GB [cited] does not fit; QLoRA fits | about 9 to 12 GB [estimate] | about 2 days | borderline; cloud is cheaper than the wait |
| Qwen3-14B | QLoRA | about 12 to 15 GB at short sequences [estimate] | 3 days or more | no; this is the model Cartalith already runs un-tuned |
| Qwen3.8-27B | QLoRA | does not fit in 16 GB with usable context [estimate] | n/a | **not feasible locally**; it stays the *teacher* (s.13) |

The 10M-token figure is Part A section 5's range; the real count comes from lesson L5's dry run, and the first epoch's own log replaces every time here. The CPU and RAM are not the bottleneck; **disk is**: budget about 60 to 100 GB for base weights, data, checkpoints, a merged 16-bit copy and GGUFs [estimate].

### 12.4 Tools compared

| Tool | Use here | AMD / this card | Verdict [judgement] |
|---|---|---|---|
| **Unsloth** (Studio and library) | LoRA/QLoRA SFT, a chat-template helper, GGUF export built in | vendor-documented full support for gfx1101 on Windows, WSL and Linux; pre-release bitsandbytes for 4-bit; undated page | **First choice**: one tool from data to GGUF |
| **Hugging Face TRL + PEFT** | the reference SFT/DPO trainers Unsloth sits on | ROCm PyTorch suffices; no Radeon-specific doc found | the fallback when Unsloth misbehaves; more code |
| **Axolotl** | YAML-driven training | the only AMD source found is an older community blog on an MI-series build warning installs "might overwrite BitsAndBytes, DeepSpeed, and PyTorch" | skip for one consumer card |
| **LLaMA-Factory** | web UI and YAML | AMD publishes a playbook (`developer.amd.com/playbooks/llama-factory-finetuning`); the ROCm docs' Llama-3.1-8B tutorial was tested on **Instinct MI300X, not Radeon** | optional second opinion |
| **torchtune** | PyTorch-native recipes | no Radeon source found; maintenance status not checked | skip |
| **llama.cpp `finetune` / `llama-finetune`** | was a LoRA trainer | removed upstream, reported not working | **not for training**; llama.cpp converts, quantises, serves |
| **MLX** | Apple silicon only | not applicable | none |

### 12.5 From adapter to LM Studio

1. Train the LoRA; sanity-check it on the Python side only (loss, a few samples).
2. **Merge** into the 16-bit base (Unsloth `save_pretrained_merged` or PEFT `merge_and_unload`) so template, tokenizer and `config.json` stay consistent.
3. **Convert** with llama.cpp's `convert_hf_to_gguf.py` at f16 or bf16, then `llama-quantize` to Q4_K_M (small) and Q5_K_M/Q6_K/Q8_0 (comparison). An **importance matrix** (`llama-imatrix` over a few hundred training prompts, never the test set) helps low-bit quality [recalled, standard; check flags against the llama.cpp build in use]. Unsloth can do 2 and 3.
4. **Load in LM Studio:** `lms import <file>.gguf --user-repo <name>/<model> --copy`, or place the file at `~/.lmstudio/models/<publisher>/<model>/<file>.gguf` and open My Models [cited, lmstudio.ai import docs; one source disagrees about `--symbolic-link` keeping the original, so use `--copy`]. `lms ls` shows the models directory actually in use.
5. **The check that matters: the tool-call wire.** LM Studio parses tool calls using the model's template and its own parser. A tune trained under a different template than the GGUF carries produces tool calls that come back as plain text. L9's checkpoint is a real `/v1/chat/completions` request returning structured `tool_calls`. The Qwen3.5 card recommends the `qwen3_coder` tool parser and says Qwen3.5 does **not** honour Qwen3's `/think` and `/nothink` switches; thinking is turned off with `chat_template_kwargs {"enable_thinking": false}` [cited, Qwen3.5-4B card]. How Cartalith's existing "thinking off" setting (`openai.rs`, the `Thinking` enum) reaches a Qwen3.5 template through LM Studio is **unverified**; it is an L9 step.
6. Whether LM Studio's runtime loads a **Qwen3.5** GGUF is unverified here. A Qwen3.8-27B (the same hybrid family) **did** run in it (`STATUS.md`; `llm-live-2026-10-26`): circumstantial only.

## 13. The dataset: generated from Cartalith itself

Part A section 5 specified the parameter traces. This section widens it to everything the assistant now does. Each source was opened at its symbol on 2026-11-11.

| Source (symbols) | What it gives | Hazard |
|---|---|---|
| **Action catalogue**, `cartalith-llm/src/actions.rs` (`ActionCatalogue::build`, `ActionEntry`, `ArgSpec`, `PACKS`, `MENU_REVIEW`, `WINDOW_REVIEW`, `CONTEXT_REVIEW`, `ALIASES`, `refusal`), `data/action_static_sources.json` | **205 entries** on the real inputs (114 parameters, 13 view verbs and layers, 10 windows, 31 menu rows, 23 context actions, 13 tools, `timeline.set_year`) [`STATUS.md`, AC-1]; id, kind, effect, route, a description capped at `MAX_DESC_CHARS` 120, typed args | 24 of the 205 have only a **derived** description (AC-5's list); the catalogue **grows**, so train on randomised subsets and renamed ids, not the literal 205 |
| **Tool schemas**: `action_tools.rs` (`list_actions`, `search_actions`, `describe_action`, `propose_actions`, the `MAX_*` caps, `ActionLedger`, `KNOWN_ID_RULE`, `validate`), `params_flow.rs` (`get_param_schema`, `propose_param_changes`, `set_generation_params`), `query.rs` (`get_parameters`, `get_changed_parameters`, `describe_parameter`, `get_ecology_summary`, `sample_point`, `get_world_summary`), `view_actions.rs` (`propose_view_actions`), `edit_actions.rs` (`propose_edit_actions`), `knowledge_tools.rs` (`search_knowledge`, `read_source`, `related_sources`) | the closed tool set the model must name and shape exactly | the request's "13 legacy tools" does not match what these files hold (about fourteen older tools besides the four action tools; the "13 tools" in the catalogue are map tools). **Generate the tool list by script in L5; do not type a count** |
| **Parameter registry**, `cartalith-godot/src/params.rs` (`PARAMS`), `param_catalogue.rs` | keys, ranges, units, defaults, groups for the randomised-catalogue traces | the evaluator re-derives ranges from the same file |
| **AC-0 task set**, `docs/evidence/assistant-actions-ac0-2026-10-31/tools/tasks.json` (30 tasks: `id`, `kind`, `prompt`, `slots`), `tools/run_ac0.py` | the **test** set (s.16) | **never train on these 30 prompts or near-paraphrases**; a contamination check in L5 |
| **AC-4 harness**, `cartalith-godot/tests/ac4_live.rs` (conditions `none`, `pack`, `pre`, `both`; env `AC4_LIVE_MODEL`, `_ADDR`, `_CONDS`, `_TASKS`, `_REPS`, `_OUT`) | the real prompt builder, validator and scorer against whatever model LM Studio has loaded (it never loads or unloads one) | it scores `propose_actions` only; AC-4's notes list t17, t18, t21 and the seven setting tasks as unable to discriminate and t30 as mis-scored. **Fix or exclude those before it is a gate** (SL-9) |
| **LLM-6c reports**, `llm-model-eval-2026-10-26/REPORT.md`, `llm-live-2026-10-26/REPORT.md` | failure modes to train against: protocol failure (a correct proposal written as prose, 12 of 15 for Ministral; never calling a tool, Helcyon), invented keys (all caught by V5), thinking on worse than off, Haiku proposing nearby changes to impossible requests, a second `get_param_schema` replacing the offered keys; the speed table (27B Q3_K_XL 43.7 tok/s tuned, 31.7 at first settings, 11.9 at LM Studio defaults) | single runs, n=3 per cell; the direction labels are one agent's reading |
| **Prose and grounding**: `place_facts.rs` (`not_available`, the 25 km radius), `shell/llm_place_facts.gd`, `grounding.rs` (the post-hoc check), `llm-vault-prose-2026-10-26/SANITY_CHECK.md` | grounded village prose: facts in, prose out, then the grounding check as the **validator** | that check found **qwen3-14b wrote an invented `Discrepancy:` sentence in 16 of 16 no-notes replies** (since fixed in the prompt), exact population stated 16 of 16 by the 27B against 9 of 16 by the 14B, invented demonyms, and unmarked flavour the checker cannot see |
| **RAG tools**: `knowledge_tools.rs`, `RAG_KNOWLEDGE_BASE_RESEARCH.md` | traces that search, read and cite, treating retrieved text as data | injection strings live in `llm9_corpus.rs`; keep them as negative examples |

**Trace kinds and target mix [judgement, set after the L5 dry run]:** parameter proposals (Part A's mix), catalogue navigation and `propose_actions` (list, search, describe, then propose only ids the ledger holds), **declines** (an action the catalogue lacks; destructive or session-gated entries per `refusal`), view and edit proposals, knowledge look-ups with citation, and grounded **village prose** (about 20% to 25% of tokens). No Vault text, no map image, no user data (Ruling DI P5): inputs are synthetic or built from generated worlds.

**Pipeline (the rule: never train on an unvalidated tool call).**

1. **Seeds.** Many worlds from the real engine at random seeds, so names, populations and facts are real and varied; randomised catalogues (random subsets, jittered values, renamed or added ids).
2. **Requests.** Hand-written intents plus paraphrases, **split by template and by seed into train, validation and test before anything is generated**; the 30 AC-0 prompts are removed from the pool.
3. **Teacher.** Default: the **local Qwen3.8-27B** (Apache-2.0, already tuned to 43.7 tok/s in LM Studio): a redistributable add-on trained on model output must be able to state that output's licence. Claude is the stronger teacher on declines (Part A s.3) but **Anthropic's commercial terms restrict using outputs to train competing models [recalled, not re-read; confirm the current terms before Claude-generated data enters any redistributed model]**; use Claude as an *evaluator*, or for a personal-use model if the owner so decides (Q-SL-2). Cost on the local teacher: 5 000 traces x about 600 generated tokens is about 3M tokens, about 19 h at 43.7 tok/s plus prefill [estimate]: an overnight-to-a-day job, using LM Studio's model, not a second GPU.
4. **Validators run by the real code.** Tool traces go through the real Rust validators (`ActionLedger`/`validate`, the V1 to V13 params validator, the view and edit validators) via a small test-only runner that reads JSON lines and returns the verdict, **the same code the app runs, never a Python re-implementation**. Keep a trace only if every call is accepted, values are in range and direction, and a decline made no change. Prose goes through `grounding.rs`: reject a reply with an unmatched number or an unverified invented proper noun above a stated tolerance, with the `Discrepancy:` and unmarked-invention patterns as explicit rejects.
5. **Assert, do not assume:** zero invented ids, zero out-of-range values, zero duplicates, mix within five points, splits disjoint on seed and template, the dataset non-empty and shaped (MISTAKES: *watch for silently-empty golden output*). A mutation: disable the validator in the runner and the check must go red.
6. **Hand audit:** 100 random accepted tool traces and 30 prose traces by the owner; at most two wrong directions (Part A's bar).
7. **Store** the dataset (synthetic, releasable), seeds, teacher id and quantisation, harness commit and registry commit with the add-on manifest (Part A s.7).

**Licences [cited 2026-11-11 unless marked].** Base weights: Qwen3.5-4B and 9B cards and the Qwen3.8-27B card say Apache-2.0 (the 2.4T and Flash-Next releases are not Apache-2.0; avoid as a base); Apache-2.0's duties (licence text, notice of changes, preserved notices) go in the add-on folder (Part A s.4, SL-5). Training data: Qwen3.8-27B output is covered by its Apache-2.0 release; Claude-generated data is gated by the terms question. A **LoRA adapter is a derivative of its base**; ship it with the base's licence file.

## 14. Which model, which size [judgement, with the evidence it rests on]

- **Recommended: Qwen3.5-4B**, thinking off, bf16 LoRA. Apache-2.0 on its card; the one size comfortably inside 16 GB (about 10 GB, cited secondary); the card calls tool calling a strength and names the parser; the family match with the 27B teacher and 14B control keeps prompts and templates alike. **Risks:** a newer hybrid (Gated DeltaNet) architecture, so training, merge, GGUF convert, LM Studio load and tool-call parse must each be proved in L3 to L9 and not assumed; the 4B's tensor metadata reads "5B params" because of the vision encoder [cited, card], which a text-only GGUF may drop [unverified].
- **Conservative fallback: Qwen3-4B (or -Instruct-2507)**, a plain transformer with the best-trodden toolchain, in the same family as the qwen3:14b control. If L3 to L9 stall on Qwen3.5 for more than a day, switch; the dataset is model-neutral (structured messages rendered through whichever template at training time).
- **Phone tier: Qwen3.5-2B or Qwen3-1.7B**, only if the 4B passes the gate and the owner wants it (s.15).
- **Not recommended:** 8B/9B (the wait, QLoRA risk), 14B/27B (not trainable locally), Llama and Gemma (licences, Part A s.4).
- **Prose.** A 4B will write *competent grounded village prose from supplied facts*; it will not match the 27B's care with exact figures (16 of 16 against 9 of 16 for the 14B on population). The tune teaches **style and the habit of using supplied numbers**, and the grounding check keeps it honest. It adds no world knowledge; the world is in the prompt.

## 15. Integration: ranked

The wire is OpenAI-compatible chat with tool calls; `presets.rs` carries `lm_studio` (`localhost:1234/v1`), `ollama` and `llama_cpp` presets. Every option keeps **the security surface unchanged**: the model proposes and never acts, the validator runs before a preview, a click applies, the key never in a save or log, zero sockets when the feature is off (`LLM_ASSISTANT_SCOPE.md` s.4; Rulings DC, DD, DF, DI).

| Option | What changes in Cartalith | Pro | Con | Verdict |
|---|---|---|---|---|
| **(a) As now: served by LM Studio** | **Nothing in the app.** The owner imports the GGUF (s.12.5) and picks it in the model dropdown (LLM-12); optionally a profile (`profiles.rs`, Ruling DQ) names it with thinking off | zero risk, immediate; the owner already tunes LM Studio (ROCm 2.64.0, `--gpu max -c 8192 --parallel 1`); it is also the evaluation harness | the user must run LM Studio; no distribution story | **Do this first** |
| **(b1) Bundled `llama-server` sidecar** started by Cartalith | a local preset plus a launcher (Part A s.2, SL-1): loopback, random port, key not in argv, killed on exit, feature off = no process | reuses the provider path, consent, token counter, cancel and tests; the app knows the adapter it launched (Ruling DF) | process hazards (orphans, antivirus on a bundled exe, port clash, VRAM contention with the renderer); a binary to ship (MIT [recalled]) | **The desktop add-on route**, after the gate |
| **(b2) In-process llama.cpp via a Rust binding** | C++ in the workspace (`bindgen`, CMake), a bigger `cartalith_godot` DLL, a new adapter | one process | **against the rule that `cartalith-llm` opens no socket and carries no heavy native dependency**; slower builds; a native panic takes down the process | only for Android, and only if a sidecar cannot be used there |
| **Android** | needs b2 (a binary cannot be `exec`'d from app data on current Android [recalled]) | a 2B model could answer offline | **Qwen3.5-4B Q4 on the OnePlus 6T (Snapdragon 845) is out of reach** (Part A: 1 to 3 tok/s). Recent Snapdragon sources give 7.6 to 24 tok/s for 4B-class models depending on context and SoC, decode versus prefill unclear [cited, secondary]; a **Pad 3-class device is plausible for 1.7B to 4B**, unmeasured (its SoC and RAM: recalled as Snapdragon 8 Elite, confirm). Phones keep using the PC's server over the LAN as now | SL-7 stays optional and last |

**How the model fits the catalogue.** The catalogue is *progressive disclosure through tools* (`list_actions`, `search_actions`, `describe_action`, `propose_actions`): a small model never holds all 205 entries; it must **choose the next tool and copy ids it was just shown**, the habit AC-4 shows is the weak spot (14B: 7/30 with no catalogue help, 17 to 18 with pre-retrieval). **Pre-retrieval is on by default** (`actions_pre`, `STATUS.md`), so the tune competes with an already-helped baseline and is measured against the `pre` and `both` rows, not `none`.

## 16. Evaluation, regression floor and go/no-go gate

**Rule:** no pass rate is invented in advance (LLM-6c); the bars are **proposals to put to the owner**, adopted or changed after the L2 baseline.

| Measure | How | Baseline to beat |
|---|---|---|
| **Action-card accuracy** | `ac4_live.rs`, the 30 AC-0 tasks, each condition, **three runs a condition** (AC-4's two samples were "not a variance"), model loaded by the owner, thinking off | qwen3-14b printed `none` 7, `pack` 11, `pre` 18, `both` 17 (16 with t30 corrected); run 2 on the 20 answerable tasks: `pack` 12, `both` 17 [`STATUS.md` AC-4 entry; raw in `assistant-actions-ac4-2026-11-08/`]. **Floor for the tuned 4B: at least the 14B's `both` score on the same tasks and `pre` not below the 14B's** |
| **Parameter proposals** | LLM-6c stage 1 (`llm-model-eval-2026-10-26/tools/run_eval.py`) on a held-out catalogue: usable, clean, direction right, impossible-declined, invented keys | qwen3:14b thinking off 15/15, 15/15, 20/20, 9/9 on the original sample; the 27B ranked second in stage 2 |
| **Protocol** | share of turns with a structured `tool_calls` entry; share with a repeated or hallucinated id (`KNOWN_ID_RULE`) | the 14B at 0 invented ids with the ledger on |
| **Hostile corpus** | `llm9_corpus.rs` strings through the real prompt | 0 valid out-of-scope proposals (as SL-6) |
| **Prose rubric** | the grounding check over **20 places not in training**, three samples each: unmatched numbers, unverified invented names, invented `Discrepancy:` lines, exact-population statements, refusal to invent when a fact is `not available`; **plus an owner blind A/B** of 10 places against the 27B (usable as-is, usable after edits, not usable) | the 27B and 14B counts of `SANITY_CHECK.md` (single runs, two seeds; re-run for the baseline) |
| **Speed** | tok/s and first-token time through LM Studio, alone on the machine, median with min..max, three runs | 14B about 44 to 50 tok/s; 27B Q3_K_XL 43.7 tuned, 31.7 first settings, 11.9 defaults (`llm-live-2026-10-26/REPORT.md`) |
| **Everything else** | (a) changes no app code; `cargo test --workspace` and the parse-checks stay green | green before and after |

**Gate [proposal].** (1) **No-go on training** if an un-tuned Qwen3.5-4B (or Qwen3-4B) with `pre` on already reaches the 14B's `both` score and the parameter and prose bars: ship a recommended GGUF only. (2) **Go** if the base 4B is within 25 points of the 14B on action-card and parameter measures and its failures are protocol (prose instead of a call, a wrong id), the failure SFT repairs. (3) **Ship** only if the tuned 4B meets the floor on **both** the AC-0 tasks and the held-out sets, beats its own base on at least three of five measures, has 0 hostile-corpus failures, and the owner's blind prose A/B finds at least 7 of 10 usable. (4) **Stop** if after one revision it misses the floor by more than 10 points: record the numbers and stay on the local 14B or 27B through LM Studio.

## 17. Two adapters on one base, not one blend [judgement]

Tool data wants low temperature, short output and strict format; prose wants longer, varied output. One adapter on both can degrade either, and prose is the part the owner judges by eye while tools are scored by script. So: **adapter T (tools and catalogue)** and **adapter P (village prose)** on the same base, each merged into its own GGUF (or one GGUF plus a runtime LoRA if LM Studio exposes llama.cpp's `--lora` [unverified]). Cartalith's profiles (Ruling DQ) already let a task choose a model, so "Commands and parameters use T; Describe place uses P" needs a profile each and **no app change**. If one model must ship, try a single adapter on the mix (Part A ratios plus 20% to 25% prose) and accept it only if both gates pass.

## 18. Owner questions (each with a default; none blocks L0 to L2)

- **Q-SL-1. Is "qwen 3.8" meant literally?** Default: **no**, it has no small size. Use **Qwen3.5-4B**, Qwen3-4B as fallback, Qwen3.8-27B as teacher.
- **Q-SL-2. Teacher for the data?** Default: the **local Qwen3.8-27B**. Claude only as evaluator, unless the owner reads the Anthropic terms and says otherwise for a model that will be redistributed.
- **Q-SL-3. Where to train?** Default: **Windows-native Unsloth for one evening, then WSL2**; rent a GPU only if both fail; no dual-boot unless needed.
- **Q-SL-4. One adapter or two?** Default: **two** (T and P).
- **Q-SL-5. Personal use or redistribution?** Default: **personal use first** (your own files in LM Studio); the add-on and its licence duties only after the gate. This settles the teacher and licence questions.
- **Q-SL-6. Integration first?** Default: **(a) LM Studio now**; (b1) only after the gate and only if you want it to run without LM Studio.
- **Q-SL-7. Who reviews the direction labels and the 30 prose traces?** Default: the owner, a few dozen lines, once.
- **Q-SL-8. GPU time?** Default: nights only; you unload LM Studio's model before each run; one 4B epoch of about 10M tokens is planned as an overnight.
- **Q-SL-9. Phone tier?** Default: no until the 4B passes; then a 1.7B or 2B and an SL-7 measurement on the Pad 3.
- **Q-SL-10. A ruling?** Default: if adopted, record it beside Ruling DK as the reopening.

## 19. The course outline

For the owner, on the owner's own machine. **Every command is a sketch from the cited documentation, not run here; each lesson's first step is to open the current doc and check the flag.** Times are [estimate] for one person new to this. Each lesson ends in a **checkpoint**: if it fails, stop and fix it before the next lesson. No lesson changes Cartalith's code.

**Preparation.** Make a folder outside the repository (for example `D:\cartalith-llm\`) with `data\`, `runs\`, `gguf\`, `eval\`. Write down the Windows driver, LM Studio and runtime versions on each run; version mismatch is the usual failure (s.12.1).

| Lesson | Goal | Tools and commands (sketch) | Time | Checkpoint |
|---|---|---|---|---|
| **L0. Orientation** | know what exists | read this file's Part B, `ASSISTANT_ACTIONS_PROPOSAL.md`, `LLM_ASSISTANT_SCOPE.md` s.5b, both LLM-6c reports | 2 h | two sentences: what the model must do, what the validator does |
| **L1. Read the real failures** | see the mistakes | `llm-model-eval-2026-10-26/results.jsonl`, the AC-4 per-task tables | 1 h | five failure types, each with one example |
| **L2. Baseline today's models** (decides whether to train) | the numbers the gate needs | in LM Studio you load a small base (Qwen3.5-4B, then Qwen3-4B) with thinking off; then `AC4_LIVE_MODEL=<id> AC4_LIVE_OUT=<file> cargo test -p cartalith-godot --test ac4_live -- --ignored --nocapture` three times; the stage-1 params script | 1 day | a dated table against the 14B and 27B rows; **gate (1) and (2) answered**; if no-go, **stop** |
| **L3. Training environment** | a GPU-visible PyTorch | Windows: `irm https://unsloth.ai/install.ps1 \| iex`, then `python -c "import torch;print(torch.cuda.is_available(), torch.version.hip)"` and a matmul timing. If it fails: `wsl --install -d Ubuntu-24.04`, AMD's WSL ROCm 7.2.1 guide at `rocm.docs.amd.com/projects/radeon-ryzen`, the PyTorch wheel the matrix names; `HSA_OVERRIDE_GFX_VERSION=11.0.0` only if no device appears [recalled]; pre-release bitsandbytes only for 4-bit | 1 evening to 2 days | `torch.cuda.is_available()` true on the 7800 XT; a 4096x4096 fp16 matmul runs; a 10-step toy LoRA on a 0.8B model with no NaN |
| **L4. A toy fine-tune** | learn the loop | Unsloth Studio or notebook, `Qwen3.5-0.8B`, 200 hand-written examples, 1 epoch | 3 h | loss falls; the adapter changes one output the way you wanted; you know where checkpoints go |
| **L5. Build the dataset** (longest) | validated traces | a test-only runner feeding JSON lines to the real validators; a Python script rendering catalogues, seeding worlds and calling the teacher (the local 27B at `127.0.0.1:1234`, loaded by you); the filters of s.13 steps 4 and 5; split files; the contamination check against `tasks.json` | 3 to 7 days (the teacher run about a day of wall clock) | a 200-trace dry run first (token counts, accept rate, mutation test red); then 5 000 accepted tool traces and 1 000 prose traces, 0 invented ids, splits disjoint; hand audits pass |
| **L6. Train adapter T on Qwen3.5-4B** | the first real tune | Unsloth, bf16 LoRA, rank 16 to 64 on all linear layers (a starting point, tuned by L7), the sequence cap from the dry run, completion-only loss on assistant turns and tool calls, thinking disabled in the template, learning rate near 1e-4 to 2e-4 [recalled start], 1 to 3 epochs, validation loss every few hundred steps, **LM Studio's model unloaded first** | 8 to 16 h an epoch | loss and validation loss fall, no NaN, 20 held-out prompts give a correct call in the Python test |
| **L7. Python-side evaluation, iterate** | catch problems before GGUF | greedy decoding over held-out and validation sets through the same validators | 2 to 4 h | held-out within 5 points of validation (no leakage), invented ids 0 |
| **L8. Merge and GGUF** | an LM Studio file | merge to 16-bit; `python convert_hf_to_gguf.py <merged> --outtype f16`; `llama-imatrix` on 300 training prompts; `llama-quantize` to Q4_K_M and Q5_K_M (a llama.cpp build near LM Studio's runtime) | 2 h | a GGUF of about 2.5 to 3.4 GB [estimate] that `llama-cli` answers from, template printed |
| **L9. Load it and check the tool wire** | prove integration (a) | `lms import <gguf> --user-repo <you>/cartalith-4b-t --copy`; load at `--gpu max -c 8192 --parallel 1`; one raw `/v1/chat/completions` with the `propose_actions` schema; confirm structured `tool_calls`; confirm thinking-off reaches the template | 3 h | a structured tool call appears; if text, fix the template, not the data |
| **L10. The real evaluation** | the gate | s.16 in full: three `ac4_live` runs a condition, params, hostile corpus, speed; then adapter P and the prose rubric with the owner's blind A/B | 2 days | the table, computed by script, beside the L2 baseline; the gate answered |
| **L11. Decide and integrate** | choose a path | (a) add a profile (no code); optionally (b1) the SL-1 sidecar | owner | a recorded owner decision; if go, SL-14 and SL-15 |

**Total:** about 2 to 4 weeks of evenings for a first tuned 4B adapter, dominated by L5 and the overnight runs [estimate; the unknown is L3].

## 20. Milestones (they extend SL-0 to SL-7; effort as in s.9; bars are proposals)

| ID | Milestone | Verifiable bar | Effort |
|---|---|---|---|
| **SL-8** | L2 baseline recorded (SL-0 widened to Qwen3.5-4B and the AC-4 harness) | a dated table, three runs a condition, median with min..max, thinking off, reproducible from s.19's commands; the 14B and 27B rows re-run beside | S |
| **SL-9** | `ac4_live.rs` fit to be a gate | t30's scoring fixed; t17, t18, t21 and the setting tasks excluded or scored through the older tool, the exclusions listed; a mutation (swap two ids) turns the score red | S |
| **SL-10** | Validator runner and data pipeline | the accepted-set assertions of s.13 step 5; a test proves the runner rejects an invented id; the split disjointness checked by script; the AC-0 prompts absent from train and validation | L |
| **SL-11** | Training environment proven on the owner's card | the L3 checkpoint; the winning route (native, WSL2, cloud) recorded with versions | S to M |
| **SL-12** | Adapter T trained and gated | L6 to L10 on action-card, params, protocol, hostile corpus | L |
| **SL-13** | Adapter P and the prose gate | the rubric measures; the owner's blind A/B at least 7 of 10 usable | L |
| **SL-14** | Integration (a) | a profile naming the model with its thinking-off request; consent text unchanged; a probe through the real panel on the owner's machine | S |
| **SL-15** | Optional integration (b1) | SL-1's bars; only after SL-12 and the owner's Q-SL-5 answer | M |

## 21. Risks added by this part

1. **ROCm version mismatch** (the dominant failure in the sources): pin and record versions (L3).
2. **A new architecture** (Qwen3.5 hybrid): train, merge, convert or load may not yet work; fallback Qwen3-4B (s.14).
3. **Template mismatch** (tool calls returned as text): the L9 checkpoint.
4. **Contamination** of the AC-0 test: L5 and SL-10.
5. **A teacher ceiling:** a student seldom exceeds its 27B teacher; the gate compares against the 14B and 27B, not Claude.
6. **Validated but wrong data** (a valid call that is not what the user asked): the validator proves *shape and range*, not *intent*; direction labels and the hand audit carry that.
7. **Licence of generated data:** Q-SL-2 and Q-SL-5.
8. **Time:** the s.12.3 estimates are arithmetic; the first epoch's log replaces them.
9. **Documents age:** the Unsloth page is undated and the ROCm matrices move monthly; re-open the cited pages at L3 (a document's claim about itself is a claim).

## 22. References for Part B (opened or searched 2026-11-11)

- Qwen cards: `huggingface.co/Qwen/Qwen3.8-27B` and `huggingface.co/Qwen/Qwen3.5-4B` (opened); the Qwen3.5 9B card and Qwen GitHub changelog (search extracts); Qwen3.8 licensing news (eWeek, runtimewire and others: secondary, conflicting on release dates; one page said the 27B had no official repository, contradicted by the card opened).
- Unsloth AMD install page `unsloth.ai/docs/get-started/install/amd.md` (opened, undated); AMD's article `amd.com/en/developer/resources/technical-articles/2026/train-and-run-models-on-amd-gpus-with-unsloth.html` (search extract); Unsloth's Qwen3.5 fine-tune page and mirrors (search extracts).
- AMD ROCm: the WSL compatibility matrices (`rocm.docs.amd.com/projects/radeon-ryzen/.../wsl_compatibility.html`, ROCm 6.4.2 and 7.2.1), ROCm Core SDK 7.10.0 and 7.11.0 preview notes, Windows HIP SDK 7.1.1 system requirements, the "Windows PyTorch 7.2" release note (search extracts; unreconciled on gfx1101 for Windows PyTorch).
- bitsandbytes: the Unsloth note on the pre-release requirement and 4-bit NaN bug; a community Windows ROCm 7 build (`github.com/guinmoon/rocm7-triton-flash-attention-sage-attention-bnb`); not opened: bitsandbytes' own release notes.
- llama.cpp: issue 18499 (2025-12-30) on `llama-finetune`; a mirror of the commit removing `finetune`; Tether QVAC's LoRA fork (search extracts).
- LM Studio import: `lmstudio.ai/docs/app/advanced/import-model` and `lmstudio.ai/docs/cli/local-models/import.md` (search extracts).
- Android: llama.cpp Snapdragon docs and benchmark pages (search extracts, unclear on decode versus prefill).
- Repository: `LLM_ASSISTANT_SCOPE.md` s.4; `ASSISTANT_ACTIONS_PROPOSAL.md`; `STATUS.md` (AC-1 and AC-4 entries); the evidence folders named in s.13.
