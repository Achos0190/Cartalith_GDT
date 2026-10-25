# LLM_INTEGRATION_RESEARCH.md — an optional, user-keyed language-model assistant

**What this is:** research, not a scope and not a status record (no status column; where anything stands is `cartalith-native/docs/STATUS.md`). Agent-produced 2026-10-25 from the owner's question: *"what if we added LLM integration to Cartalith with an API; call a local model or whatever the user has; it has to work with an API key. What could be implementations?"*

**Evidence tags.** **[verified]** = opened in this repository this session, named at its symbol or file. **[cited]** = read at the URL given on 2026-10-25. **[recalled]** = from the author's background knowledge, not re-fetched this session; re-check before relying on it. **[judgement]** = a labelled recommendation. Nothing here is "measured": no code was written and no timing was taken.

---

## 0. The short answer

1. The app has **no networking of any kind today**, no secret store, and no Android `INTERNET` permission declared. An LLM feature is therefore the first outbound connection the product would ever make, and that is a **disclosure and defaults** decision before it is an engineering one. [verified, §1]
2. The one protocol that reaches nearly everything the owner named is the **OpenAI-compatible `/v1/chat/completions` API**: local servers (Ollama, LM Studio, llama.cpp, vLLM) and hosted routers (OpenRouter, OpenAI, Gemini's compatibility endpoint) all speak it, and every one takes an API key (local ones ignore the value). [cited, §3] One adapter plus an optional Anthropic Messages adapter covers the field. [judgement]
3. The model must **never sit on the generation path**. Cartalith's worlds are seed-deterministic and golden-tested; a model call there breaks reproducibility, saves and parity. The model is an *assistant at the edges*: prose, suggestions, and proposed *structured actions that the user confirms and the existing undoable tools execute*. [judgement, §2, §6]
4. Recommended first build: a **Rust `cartalith-llm` crate** (blocking HTTP client on a worker thread, no Godot types) behind gdext, with a **GDScript-only fallback** (`HTTPClient`) judged not worth it except as a spike. [judgement, §4]
5. Everything is **off until the user turns it on**, per machine, with the key in the OS credential store (or an environment variable), never in a `.zip`, a project archive, an export or a log. [judgement, §5]

---

## 1. What the app has today (read at the symbols)

| Concern | Finding | Evidence |
|---|---|---|
| Outbound networking in Godot code | None. A search of `cartalith-native/` `.gd`, `.rs`, `.toml`, `.cfg` for `HTTPRequest`, `HTTPClient`, `StreamPeerTLS`, `TLSOptions`, `reqwest`, `ureq` returned no hits | [verified] |
| Rust dependencies | Workspace crates depend on `serde`, `serde_json`, `zip`, `rayon`, `wgpu`, `pollster`, `bytemuck`, `godot` (gdext 0.5.5, feature `api-4-7`) and the in-tree crates. No HTTP or TLS crate. `cartalith-godot` is "the only crate that depends on gdext" | [verified] `cartalith-native/Cargo.toml`, `crates/*/Cargo.toml`, `crates/cartalith-godot/Cargo.toml` |
| Secrets | None. A search for `api_key`, `password`, `credential`, `keystore` found only test fixture names (`secret.png` in asset-import path-escape tests). No secret store exists | [verified] |
| Settings | `DccSettings` (`shell/dcc_settings.gd`) over one `ConfigFile` at `user://cartalith_settings.cfg`, sectioned (`_SEC_ROOTS`, `_SEC_RECENT`, `_SEC_GPU`, `_SEC_CPU`…). Its header states it is machine state and "nothing in it is ever written into a world `.zip`". Plain text, so unsuitable for a key | [verified] |
| Other `user://` files | `markdown_vault.json` (vault device binding, `vault_store.gd`), `appearance_presets/` (`engine_bridge.gd` `PRESET_DIR`), `logs/godot.log` (rotated five deep, `diagnostic_report.gd`) | [verified] |
| Logs | `diagnostic_report.gd` reads `user://logs/godot.log`; anything printed lands there and may be attached to a report. A key or a prompt must never be logged | [verified] |
| Android permissions | `export_presets.cfg` has `permissions/custom_permissions=PackedStringArray()` and no `permissions/internet` line. Godot adds permissions from checkboxes in the preset; the `INTERNET` one is not set in the committed preset (whether the debug template adds it for remote debugging was not tested) | [verified] file; [recalled] Godot behaviour. Re-test on a device before any Android milestone |
| Android cleartext | Android 9+ blocks `http://` by default unless a network-security config allows it. A LAN Ollama (`http://192.168.x.x:11434`) therefore needs an explicit exception or a TLS front | [recalled] Android network security config docs |
| Background threads | Generation runs on a GDScript `Thread` (`engine_bridge.gd`, five `Thread.new()` sites); `cartalith-godot/src/lib.rs` documents that gdext's `experimental-threads` is **not** enabled and that no hook may call into Godot off the main thread; the lib stops its own worker threads on unload | [verified] |
| Undo | Height-edit and way undo stacks exist in `cartalith-godot` (`context_pick_bridge.rs`: `WayUndoEntry`, `choose_undo`); the engine has `set_undo_budget_mb`; `LookHistory` for appearance; `ColourPaintTool` has its own. There is **no single command bus**: undo is per-tool | [verified] |
| Command surface | `CommandIndex` (`shell/command_index.gd`) builds one searchable list of every parameter, menu command and declared extra *from the live app*, each with `available` and a `why` when not. This is the natural tool catalogue for a model | [verified] |
| Name generation | `cartalith-civ` `civ_settle_name` consumes a fixed RNG count and is golden-pinned; `naming.rs` shapes names only where "the reference has no behaviour to match" (continents, provinces, the re-roll button) | [verified] |
| Vault | `cartalith-vault` exposes `VaultService`-style `list`, `read`, `read_for_edit`, `write_file(rel, text, expect_hash)` (hash-guarded write-back), `entity_backlinks`, `entity_mentions`; Android's SAF provider is a separate milestone (`MARKDOWN_VAULT_SCOPE.md` Milestone 4) | [verified] `crates/cartalith-vault/src/lib.rs` |
| Story data | Authored events are Chronos blocks in the Vault (`STORY_PLANNING_SCOPE.md` §3, Ruling AM); settlement timelines, conflicts and journeys run over one year cursor | [verified] doc |

Consequence: **networking, a secret store, an Android permission, a settings section and a consent screen are all new.** None can be borrowed from existing code.

---

## 2. Use cases, audited against what the engine really backs

Rating: **Backed** = the data and an executing tool exist; **Partly** = data exists, no safe executor; **Unbacked** = would invent state.

| # | Use case | Rating | Why |
|---|---|---|---|
| U1 | **Prose for the user's own notes**: describe a settlement/province/faction in a few paragraphs, written into a Vault note | Backed | Vault has a hash-guarded write (`write_file(... expect_hash)`); the model sees the entity's real fields (population, culture, religion, government, trade) as context. Output is text the user edits; no engine state changes [verified] |
| U2 | **Name suggestions** for user-created things and for re-rolls (not for generated settlements) | Backed | Only where `naming.rs` already says no parity contract exists. The model proposes a list; the user picks; the existing rename path applies it. Must not touch `civ_settle_name` or its RNG [verified] |
| U3 | **Story prose from timeline data**: "summarise what happened to this faction between years X and Y" | Backed | Timeline snapshots and Chronos blocks are real; the model narrates, it does not invent events. Label as model prose [verified, `STORY_PLANNING_SCOPE.md`] |
| U4 | **Natural-language commands** ("make the mountains in the north taller", "zoom to the largest port", "turn on rivers and borders") emitting structured actions | Partly | `CommandIndex` is a ready catalogue for *menu/param* actions. Executing a parameter change is cheap; **sculpt or paint actions have per-tool undo only**, so a model-emitted batch cannot be undone as one step today. Start with view/layer/parameter-setting actions, which have no destructive effect, and defer edit actions until a grouped-undo exists [verified, judgement] |
| U5 | **Map Q&A** ("which faction has the largest standing army?", "what is east of Hurngarn?") | Partly | Answers should come from **tool calls into real queries** (faction roster, military figures, vault backlinks), not from the model's memory. Without tools the model would hallucinate. Needs a read-only query layer exposed as tools [judgement] |
| U6 | **Vault note assist**: summarise, link suggestion (`entity_mentions`), cleanup | Backed | Reads and hash-guarded writes exist [verified] |
| U7 | **Generating the world** (terrain, rivers, settlements, borders) with a model | **Rejected** | See below |
| U8 | **Embedding search over the vault** | Unbacked now | Needs an embeddings endpoint and an index; the Vault has text search (Milestone 6). Defer [judgement] |
| U9 | **Image generation / texture painting** | Out of scope | Different API family, different licensing question; not asked [judgement] |

### Why generation stays deterministic and model-free

- A world is a function of a seed and parameters; saves, goldens (`DECISIONS.md` §7, §7a) and "same seed, same world" are the product's identity. A model's output varies by provider, version, temperature and day, and cloud models are retired on schedules the project does not control.
- Golden-matching and principled-equivalence bars (`DECISIONS.md` §7a) cannot be applied to a stochastic external service. A re-baseline needs an owner ruling (`MISTAKES.md` preflight: *Change generated output*).
- Offline operation is a feature. Anything the model writes into a world must be **stored in the world as data** (a note, a name) so the world still opens with no model. [judgement]

### The architecture all useful cases share

`user request -> [model proposes] -> structured proposal (JSON) -> validator -> preview/confirm -> existing tool executes -> existing undo`. The model never mutates state; it returns *proposals* that the app validates against the same schemas the UI uses (`ParamSpec` ranges from `params.rs`, `CommandIndex` availability). [judgement]

---

## 3. Provider options

All checked on 2026-10-25 where marked [cited]; unmarked items are [recalled].

| Provider / server | Protocol | Streaming | Tools / function calling | Structured output | Key | Source |
|---|---|---|---|---|---|---|
| **Ollama** (local) | OpenAI-compatible `/v1/chat/completions`; also native `/api/chat` | Yes | `tools` supported; `tool_choice` not | `response_format` listed; `json_schema` not confirmed on the page | Local: value required by clients but ignored; cloud `https://ollama.com/v1` uses `OLLAMA_API_KEY` | [cited] https://docs.ollama.com/api/openai-compatibility |
| **LM Studio** (local) | OpenAI-compatible server, default port 1234 | Yes | Yes (model-dependent) | JSON schema supported | Ignored locally | [recalled] https://lmstudio.ai/docs/app/api |
| **llama.cpp `llama-server`** | OpenAI-compatible; grammar/JSON-schema constrained decoding | Yes | Yes (needs a chat template that supports it) | GBNF / `response_format` | Optional `--api-key` | [recalled] https://github.com/ggml-org/llama.cpp/tree/master/tools/server |
| **vLLM** | OpenAI-compatible | Yes | Yes | Guided decoding | Optional `--api-key` | [recalled] https://docs.vllm.ai |
| **OpenRouter** (hosted router, many models) | OpenAI-compatible, Bearer key | Yes | Model-dependent | Model-dependent | Bearer | [recalled] https://openrouter.ai/docs |
| **OpenAI** | Chat Completions and Responses APIs, Bearer key | Yes | Yes | `json_schema` structured outputs | Bearer | [recalled] https://platform.openai.com/docs |
| **Anthropic Messages API** | `POST /v1/messages`; header `anthropic-version`; key header `x-api-key` | SSE with `"stream": true` | `tools` with `input_schema`; model returns `tool_use` blocks, caller returns `tool_result`; `tool_choice` `auto`/`any`/`tool`/`none` | `output_config.format` with `json_schema` | `x-api-key` (the fetched page showed it only in the curl example) | [cited] https://platform.claude.com/docs/en/api/messages |
| **Google Gemini** | Native API, and an OpenAI-compatible endpoint at `https://generativelanguage.googleapis.com/v1beta/openai/` (beta) | Yes | Yes | Yes (JSON matching a schema) | Gemini key as Bearer | [cited] https://ai.google.dev/gemini-api/docs/openai |
| Others (Mistral, Groq, Together, Azure OpenAI, AWS Bedrock) | Mostly OpenAI-compatible or near it; Azure and Bedrock differ in auth/URL | Yes | Yes | Varies | Varies | [recalled] |

**Findings**
- Tool calling and JSON-schema output are **model-dependent even where the server accepts the field**; small local models often emit invalid JSON or ignore tools. The app must validate and retry once, then show the failure; it must not assume the field was honoured. [judgement]
- Streaming is SSE (`data:` lines) on all of them; the framing is shared, the event payloads are not (Anthropic's stream uses typed events, OpenAI-style uses `choices[].delta`). [cited for Anthropic's `stream:true`; recalled for the rest]
- Model names, prices and deprecations change; **never hardcode a model list**. Use a free-text model field with a "list models" call (`GET /v1/models` on OpenAI-style servers) where available. [judgement]

**Recommended adapter set.** (A) OpenAI-compatible, configurable base URL + key + model: covers local, OpenRouter, OpenAI, Gemini-compat, vLLM. (B) Anthropic Messages as a second adapter, because its tool/streaming payloads differ. Both sit behind one trait returning the same internal events (`TextDelta`, `ToolCall`, `Done`, `Error`). Native Gemini, Bedrock and Azure are reachable later as adapters or through a router. [judgement]

---

## 4. Implementation options

### 4A. Godot `HTTPRequest` / `HTTPClient` from GDScript

- `HTTPClient` supports polling, chunked bodies (`read_response_body_chunk`, `is_response_chunked`) and `TLSOptions`; TLS is limited to TLSv1.2/1.3 and certificate pinning/revocation are unsupported. [cited] https://docs.godotengine.org/en/stable/classes/class_httpclient.html
- `HTTPRequest` is the high-level node; the doc page did not say whether it buffers; treat it as **whole-body** (no incremental streaming) and use `HTTPClient` polling for SSE. [cited for HTTPClient; judgement for HTTPRequest]
- Pros: no new Rust dependency, no cross-compile of a TLS library, Godot's own CA bundle and Android networking. Cons: JSON/SSE parsing and the provider adapters in GDScript, which the CLAUDE.md "Rust suite cannot see a broken shell" rule makes harder to test; `shell/` already holds very large GDScript files; request logic cannot be unit-tested headless without a server. [judgement]

### 4B. Rust crate through gdext on a worker thread

- New workspace crate `cartalith-llm` (no Godot types): request building, SSE parsing, adapters, retry, redaction, a trait for transport. `cartalith-godot` wraps it in a small class that polls a channel from `_process`, matching the existing rule that no off-thread code calls Godot (`lib.rs` threading notes). [verified constraint, judgement design]
- Blocking client on a `std::thread` (e.g. `ureq`) is simpler than `reqwest` + `tokio` and avoids an async runtime in a cdylib that must stop its threads on unload. [judgement]. **TLS on Android is the risk**: `ureq`/`reqwest` need a TLS stack (`rustls` is pure Rust and avoids an OpenSSL cross-compile; root certificates need `rustls-native-certs` (not reliable on Android) or bundled `webpki-roots`). Binary size and cross-build for `aarch64-linux-android` (the only enabled ABI per `export_presets.cfg` `arch/arm64=true`) must be measured. [recalled; verify per `ANDROID_BUILD_SCOPE.md` §1.2]
- Pros: testable with a local mock server in `cargo test`; one place for redaction; same code on both platforms. Cons: new dependencies in a workspace that has none for networking; `.so` size (currently ~35 MB with the `android-dev` profile, per `Cargo.toml` comment) grows. [verified for size comment]

### 4C. Hybrid

Rust crate owns **protocol, parsing, validation, redaction and mock tests**; transport is a trait whose production implementation is Godot's `HTTPClient` (passed in as callbacks) or `ureq`. Lets the spike use Godot's TLS and CA store on Android while keeping the logic testable. Most code, but isolates the Android TLS risk. [judgement]

### Comparison

| | 4A GDScript | 4B Rust crate | 4C Hybrid |
|---|---|---|---|
| New dependencies | none | HTTP + rustls + roots | none or same as 4B |
| Testable without Godot | no | yes (mock server) | protocol yes, transport no |
| Android TLS risk | low | **highest** (roots, size, cross-build) | low |
| Streaming | `HTTPClient` polling | native | `HTTPClient` polling |
| Fits existing threading rule | main-thread polling, simple | worker thread + channel | main-thread polling |
| Effort (relative) | small | medium-large | medium |
| Recommended | spike only | desktop-first | **default** |

**Recommendation:** 4C. Build the protocol crate first (it is valuable under any transport), use Godot's `HTTPClient` for the wire, and revisit a Rust transport only if streaming throughput or testing demands it. [judgement]

---

## 5. Keys, privacy, cost, safety, UI

### 5.1 Key storage (never in saves, exports, logs)

| Platform | Mechanism | Notes |
|---|---|---|
| Windows | Credential Manager (DPAPI-backed), via the `keyring` crate or a Win32 call | [recalled]. Not reachable from GDScript; needs the Rust side |
| Android | Android Keystore: generate an AES key in the Keystore, encrypt the secret, store only ciphertext in app storage | [cited] https://developer.android.com/privacy-and-security/keystore says keys never enter the app process, and that the page does not describe storing raw secrets; the encrypt-then-store pattern is the page's implied use. Godot has no Keystore API, so this needs an Android plugin (Kotlin/Java) or JNI; an effort and risk line item |
| Any (fallback) | Environment variable (`CARTALITH_LLM_KEY` or the provider's usual one) read at start-up | Zero storage; right for power users and CI. [judgement] |
| Last resort | Plain `user://` file | **Refuse by default**; if offered, label it as unencrypted and exclude from backups. `export_presets.cfg` already has `user_data_backup/allow=false` [verified] |

Rules: the key lives in memory only while a request is built; never in `cartalith_settings.cfg`, a `.zip`, a project archive, a preset, an export, `godot.log`, a diagnostic report or an error string; redact `Authorization`/`x-api-key` in any debug dump; the UI shows the last 4 characters only. A test must assert the key string appears in no serialised project, settings or log output (mutation: leak it, test goes red). [judgement]

### 5.2 Privacy and disclosure

- **Off by default**, per machine, enabled in one settings section with a plain statement of what is sent: the prompt, and *the map/vault/faction text the user's request includes*. Never sent unasked: the save file, the vault as a whole, file paths, the machine name. [judgement]
- Show the exact outbound payload size and a "preview what will be sent" disclosure before the first call to a **remote** endpoint; a local `localhost`/LAN endpoint shows a lighter notice. [judgement]
- Vault notes may contain private story material (`EXPORT` has a "secret place" caveat: nothing can be marked DM-only yet, `shell/export_dialog.gd`) [verified]; the context builder must be *opt-in per source* and never include whole notes by default.
- Remote providers' retention and training terms are theirs; link to them, do not paraphrase them. [judgement]

### 5.3 Cost caps

Per-session and per-day caps in tokens (the API returns usage; local servers may not), `max_tokens` on every request, a hard stop and a visible running counter, and no automatic retries beyond one. No dollar figures are shown unless the user supplies a price per million tokens, since prices change. [judgement]

### 5.4 Prompt injection and untrusted model output

Treat **everything the model returns, and everything it reads** (vault notes, imported text, faction names the user typed or imported) as untrusted data. [judgement]
- Model output is never executed as code or as a path. Actions are a closed enum validated against `CommandIndex` entries and `ParamSpec` ranges; unknown keys, out-of-range values and unavailable commands are rejected with the reason.
- Every action shows a preview and needs a click; no "auto-run" in v1. Destructive or bulk actions are not offered until a grouped undo exists (§2 U4).
- Retrieved text is delimited and labelled as data in the prompt; an instruction inside a vault note must not widen the allowed tool set (tools are fixed per request by the app, not by the text).
- Model prose written into the vault is marked as model-generated (front matter or a block tag) and written only with the user's confirmation through the existing hash-guarded write.
- No network tools: the model cannot ask the app to fetch URLs.

### 5.5 UI per shell (DCC rules)

Design must follow the DCC canvases' vocabulary and the three-shell rule (`DESIGN_HANDOFF.md`, `DCC_SHELL_SCOPE.md`: newer canvas wins; where none exists, derive from existing vocabulary). No LLM surface is in any current canvas, so this needs a design pass before UI work (`SCREENS_NEEDED.md` precedent). [verified doc conventions; judgement]
- **Desktop**: a docked "Assistant" panel (message list, input, a proposal list with Apply/Dismiss), settings under the Preferences/Performance-style group of `dcc_settings.gd` (provider, base URL, model, key status, caps, the off switch).
- **Tablet**: same panel in the right dock; touch targets at the existing tablet scale.
- **Phone**: a bottom sheet from the tool sheet pattern, one proposal at a time; the `CommandIndex` search field is the natural entry (a "Do it for me" row on no match).
- **Unavailability is part of the entry**: when disabled, offline or key-missing, the control shows its reason (the `CommandIndex` rule), not a dead button.

---

## 6. Milestones (bars are verifiable)

Proposed names `LLM-0`…`LLM-8`. Effort is relative (S ≈ 1 day, M ≈ 2-4 days, L ≈ 1-2 weeks) for one agent lane. [judgement]

| ID | Milestone | Bar | Effort | Main risk |
|---|---|---|---|---|
| **LLM-0** | Owner decisions + design pass (§7) | Owner answers Q1-Q9; a design for the settings section, panel and phone sheet exists in the canvas vocabulary | S + design | No canvas exists |
| **LLM-1** | `cartalith-llm` protocol crate: OpenAI-compatible adapter, SSE parser, request builder, redaction, event enum, mock-transport tests | `cargo test -p cartalith-llm` green with at least: a recorded multi-chunk SSE stream parses to the expected text; a split-across-chunk `data:` line; a malformed JSON tool call is rejected; a request containing the key never appears in `Debug`/error output (mutation: add the key to an error, test fails). **No network in tests** | M | Provider payload drift; keep fixtures dated |
| **LLM-2** | Settings + key storage on Windows; consent screen; off-by-default | With the feature off, a packet capture or a socket-open check shows **zero** outbound connections from a full session (generate, save, export); key absent from `cartalith_settings.cfg`, a saved project and `godot.log` (grep test over all three) | M | Credential Manager access from Rust |
| **LLM-3** | Local-model path end to end (Ollama or LM Studio on localhost): U1/U3 prose into a Vault note, streamed, cancellable | On a developer machine with a local model: first token visible, cancel stops the stream and closes the socket (check no thread left on library unload, per `lib.rs` rule); the note write goes through `write_file(expect_hash)` and a conflicting edit is rejected | M | Small-model quality; document a "tested with" list rather than a promise |
| **LLM-4** | Hosted OpenAI-compatible + Anthropic adapter; token caps | Mock-server tests for both payload shapes; cap test: with a 200-token cap the request carries `max_tokens` <= 200 and the session counter stops further calls; usage parsing covers a stream that ends without a usage field | M | Anthropic vs OpenAI event differences |
| **LLM-5** | Proposal pipeline: closed action schema, validator, preview/apply for **view/layer/parameter** actions only (U4 subset) | Every `ParamSpec` key in `params.rs` validates range-clamped input and rejects 100% of a fuzzed set of out-of-range or unknown keys; applying a proposal goes through the same setter as the UI and is undone by the existing path; a proposal naming an unavailable command is refused with the `CommandIndex` reason | M-L | Param setters are not all one call path (audit first) |
| **LLM-6** | Read-only query tools for Q&A (U5): faction roster, military figures, settlement lookup, vault mentions | For a fixed fixture world, 10 scripted questions answered by tool calls return values equal to the engine's own getters (compare to the getter output, not to the model's wording); a question with no backing data returns "not available" | L | Query layer surface is large; start with five tools |
| **LLM-7** | Injection hardening test set | A corpus of at least 20 hostile vault/faction strings (instructions to call tools, exfiltrate the key, write paths) leaves the allowed action set unchanged and writes nothing without a click | M | Never "finished"; the bar is a regression corpus |
| **LLM-8** | Android: `INTERNET` permission in the preset, cleartext/LAN exception policy, Keystore plugin, phone sheet | On the device: a localhost-less run with the feature off shows no network use (as LLM-2); with it on, a hosted call succeeds over TLS; the key survives an app restart and is absent from a `run-as` copy of app storage | L | Android TLS roots, Keystore plugin, SAF vault (Milestone 4) |

Deferred: grouped undo for sculpt/paint proposals (prerequisite for U4's edit actions), embeddings (U8), image generation (U9).

### Risks, collected

1. Network surface is new; one bug sends private story text to a third party. Mitigation: off by default, previews, opt-in context sources, tests that assert zero connections when off.
2. Local-model quality varies; "works with a local model" is a claim to test per model, not to make. Keep a dated tested-with list.
3. Provider APIs and model names drift; fixtures date-stamped; model field free-text.
4. Android TLS roots and Keystore plugin are the largest unknowns (4B/LLM-8).
5. Per-tool undo means model-emitted edit batches are not one-step undoable today.
6. Binary size and a new cdylib dependency set (only if the Rust transport is chosen).

---

## 7. Owner questions, with recommended defaults

| # | Question | Recommended default |
|---|---|---|
| Q1 | Do you want this feature at all, given the product is offline and deterministic today? | Yes, as an optional assistant, never in generation |
| Q2 | Scope of v1: prose + name suggestions + Vault assist only, or also natural-language commands? | Prose and Vault assist first (LLM-3); commands (LLM-5) after, limited to view/layer/parameter actions |
| Q3 | Providers for v1 | OpenAI-compatible (covers local and hosted) first; Anthropic second; others via routers |
| Q4 | Transport | Hybrid (4C): Rust protocol crate over Godot `HTTPClient`; revisit a Rust transport only if testing or throughput demands it |
| Q5 | Key storage where no OS store is available | Environment variable or refuse; no plaintext file by default |
| Q6 | Is the feature enabled by default on first run? | Off; one explicit consent screen naming what is sent |
| Q7 | May model prose be written into the Vault, and how marked? | Only with a click, marked as model-generated |
| Q8 | Cost caps | On by default: per-session token cap, visible counter; no dollar display unless the user enters prices |
| Q9 | Android in v1? | Defer to LLM-8; desktop first, because of the TLS and Keystore unknowns and the open SAF vault milestone |
| Q10 | Is the model allowed to run **edit** actions (sculpt/paint) ever? | Not until a grouped undo exists and the owner rules |

---

## 8. Sources

- Ollama OpenAI compatibility: https://docs.ollama.com/api/openai-compatibility (read 2026-10-25)
- Anthropic Messages API: https://platform.claude.com/docs/en/api/messages (first 100 000 of 1 289 756 characters read 2026-10-25; the `x-api-key` header appeared only in the example)
- Gemini OpenAI compatibility: https://ai.google.dev/gemini-api/docs/openai (read 2026-10-25)
- Godot `HTTPClient`: https://docs.godotengine.org/en/stable/classes/class_httpclient.html (read 2026-10-25)
- Android Keystore: https://developer.android.com/privacy-and-security/keystore (read 2026-10-25)
- Not re-fetched this session (tagged [recalled] above): LM Studio, llama.cpp server, vLLM, OpenRouter and OpenAI documentation; Android network-security-config; `keyring`, `ureq`, `rustls` crate behaviour on Android.
- In-repo: `cartalith-native/godot-project/shell/dcc_settings.gd`, `command_index.gd`, `engine_bridge.gd`, `export_dialog.gd`; `cartalith-native/godot-project/export_presets.cfg`; `cartalith-native/Cargo.toml`; `crates/cartalith-godot/src/lib.rs`, `context_pick_bridge.rs`; `crates/cartalith-vault/src/lib.rs`; `crates/cartalith-civ/src/naming.rs`; `MARKDOWN_VAULT_SCOPE.md`; `STORY_PLANNING_SCOPE.md`; `ANDROID_BUILD_SCOPE.md`; `DECISIONS.md` §7, §7a.
