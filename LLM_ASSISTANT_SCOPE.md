# LLM_ASSISTANT_SCOPE.md — the optional, user-keyed language-model assistant

**What this is:** the scope for the feature `LLM_INTEGRATION_RESEARCH.md` investigated. It **defines** milestones and bars; it does not track them (no status column; progress is `cartalith-native/docs/STATUS.md`). Decisions are the owner's: Ruling DC (what, which providers, which platforms, per-capability settings) and Ruling DD (transport, key storage, consent, Vault writes, cost caps, sequencing, edit actions) and Ruling DF (the local-provider GPU rule, s.5a), all in `LARGE_ITEM_RULINGS.md`. Evidence tags as in the research: **[verified]** opened in the repository, **[cited]**/**[recalled]** as the research tagged them, **[judgement]** a labelled recommendation the owner may overrule. Nothing here is "measured"; every number below is a bar to be met, not a result.

Read `MISTAKES.md`'s preflight first. The rows that bite here: *Write a test that pins a constant* (mutate it), *Return a value that can be absent* (omit, do not default), *Declare the work green* (`cargo test --workspace` plus parse-check every touched `.gd` and `shell/app.gd`), *Change generated output* (this feature must change none).

## 1. Standing decisions (from the rulings, not restated as options)

1. **Generation stays model-free.** The model is never on the seed-deterministic path; a world opens with no model. Anything the model produces enters a world only as data the user stored (a note). [Ruling DC; research §2]
2. **Whole feature off by default; each capability its own setting** (§5). With everything off, a full session opens zero sockets. [DC]
3. **Providers:** an OpenAI-compatible adapter (local servers and hosted routers) and an Anthropic Messages adapter, both in v1. [DC]
4. **Transport = hybrid:** a Rust protocol crate over Godot `HTTPClient`. [DD Q4]
5. **Key:** OS credential store where one exists (Windows Credential Manager, Android Keystore); otherwise an environment variable; otherwise refuse. **No plaintext key file.** [DD Q5]
6. **Consent:** off by default plus one consent screen naming what is sent. [DD Q6]
7. **Vault prose:** written only on a user click, marked model-generated, through the hash-guarded write. [DD Q7]
8. **Cost:** **no enforced cap** (owner's choice, a deviation from the research's recommended default cap). A running token counter is shown. [DD Q8]
9. **Phone is in v1; sequencing desktop first, then phone** (the research default, kept). [DC, DD Q9]
10. **Edit actions (sculpt/paint) are allowed, but only after a grouped undo exists and is tested.** [DD Q10]

## 2. Architecture

`user request -> [model proposes] -> structured proposal (JSON) -> validator -> preview/confirm -> existing tool executes -> existing undo`. The model never mutates state. Proposals are a **closed enum** validated against `CommandIndex` availability (`shell/command_index.gd`) and `ParamSpec` ranges (`params.rs`). The tool set is fixed per request by the app, never widened by text the model reads. No network tool: the model cannot make the app fetch a URL. [research §2, §5.4; verified surfaces]

- **`cartalith-llm` crate (new, no Godot types):** request building, both adapters behind one trait returning `TextDelta`, `ToolCall`, `Usage`, `Done`, `Error`; SSE parser; JSON-schema validation of proposals; redaction; a `Transport` trait. Tests use a mock transport. No HTTP or TLS dependency (the wire is Godot's). [DD Q4]
- **Wire:** a thin GDScript class owning `HTTPClient` polling from the main thread (matches the rule in `cartalith-godot/src/lib.rs` that nothing off-thread calls Godot), feeding bytes to the crate through a `cartalith-godot` class. TLS limits of `HTTPClient` (TLSv1.2/1.3, no pinning) are accepted. [research §4]
- **Grouped undo (new, prerequisite):** one command-bus-level group that wraps several per-tool undo entries (height edit, way, `LookHistory`, `ColourPaintTool`) into a single step. Per-tool undo is all that exists today [verified, `context_pick_bridge.rs`].
- **Where state lives:** settings in `DccSettings` under a new `[llm]` section holding no secret; the key never in `cartalith_settings.cfg`. Nothing from this feature is written into a world `.zip`.

## 3. Milestones

Order differs from the research: grouped undo (LLM-5) now precedes every command or edit action (LLM-6, LLM-7), per Ruling DD. IDs LLM-0 to LLM-4 keep the research's meaning; LLM-8 (Android) and the hardening milestone are renumbered. Effort is relative: S about 1 day, M 2-4 days, L 1-2 weeks, one lane.

| ID | Milestone | Verifiable bar | Effort |
|---|---|---|---|
| **LLM-0** | Design pass: settings section, consent screen, assistant panel, proposal list, phone sheet, in the DCC vocabulary (no canvas exists; `SCREENS_NEEDED.md` precedent) | Artboards for desktop, tablet and phone exist and are imported; every control is in `DCC_CONTROL_INDEX.md` with its disabled-reason text; newest canvas wins (`DCC_SHELL_SCOPE.md`) | S + design |
| **LLM-1** | `cartalith-llm` protocol crate: OpenAI-compatible adapter, SSE parser, request builder, redaction, event enum, mock transport | `cargo test -p cartalith-llm` green with: a recorded multi-chunk SSE stream parses to the expected text; a `data:` line split across chunks parses; a malformed tool-call JSON is rejected; a request containing the key never appears in `Debug` or error output (mutation: put the key in an error, test red). **Zero network use in tests.** Fixtures date-stamped | M |
| **LLM-2** | Settings, key handling, consent, off-by-default, and the local-provider GPU rule (s.5a) | **Address classification** (`classify_host`, a pure function in `cartalith-llm`, no network) is table-tested: `localhost`, `127.0.0.1`, `127.5.5.5`, `::1`, `[::1]:8080`, `::ffff:127.0.0.1` are same machine; `10.0.0.5`, `172.16.0.1`, `172.31.255.255`, `192.168.1.9`, `169.254.1.1`, `fe80::1` are LAN; `172.15.0.1`, `172.32.0.1`, `8.8.8.8`, `api.openai.com` are remote; `host.docker.internal`, `myhost.local` are unknown (never resolved); ports and brackets parse; `user:pw@host` is refused (mutation: widen 172.16/12 to 172/8, test red). The warning names the detected class and the action; the three override values behave as s.5a says; the "detected as" line is visible. With the feature off, a socket-open check over a full session (generate, save, export, open Vault) shows **0** outbound connections. The key is absent from `cartalith_settings.cfg`, a saved project and `godot.log` (grep test over all three; mutation: write it, test red). Windows: key round-trips through Credential Manager; with no store and no env var a remote call is **refused with a reason**, never falling back to a file. The consent screen cannot be skipped and appears before the first call to each new endpoint | M |
| **LLM-3** | Local path end to end (Ollama or LM Studio on localhost): prose and Vault assist (U1, U3, U6), streamed, cancellable | First token visible; cancel stops the stream and closes the socket with no thread left on library unload; the Vault write happens only on a click, goes through `write_file(expect_hash)`, a conflicting edit is rejected, and the note carries the model-generated marker. A "tested with" list is kept, dated | M |
| **LLM-4** | Hosted OpenAI-compatible and Anthropic adapters; token counter | Mock-server tests for both payload shapes, including Anthropic's typed stream events and a stream that ends without usage; every request carries `max_tokens`; **no enforced cap exists by default** (test: 10 sequential calls past any counter value all go out); the counter is shown and a cap setting, if present, defaults to off; at most one automatic retry | M |
| **LLM-5** | **Grouped undo** | A group of N mixed edits (height, way, appearance, colour paint) undoes in exactly **one** step and redoes in one; the world state after undo hashes equal to the pre-group state on 3 seeds; a failure inside a group rolls the whole group back; an existing single-tool undo behaves as before (no regression in its tests). Mutation: drop one member from the group, test red | L |
| **LLM-6** | Proposal pipeline: view, layer and parameter actions | Every `ParamSpec` key validates range-clamped input and rejects 100% of a fuzzed set of at least 500 out-of-range or unknown-key proposals; applying goes through the same setter as the UI and undoes as one group (LLM-5); a proposal naming an unavailable command is refused with the `CommandIndex` reason; no action runs without a click | M-L |
| **LLM-7** | Edit actions (sculpt, paint) behind their own switch | Off by default and separate from LLM-6's switch; refuses to enable unless LLM-5's tests are present and green in the build; a proposed edit shows a preview and needs a click; applying creates exactly one undo group; stroke counts and area are bounded per proposal by a constant with a named source; a hostile corpus (LLM-9) cannot trigger one | L |
| **LLM-8** | Read-only query tools for Q&A (U5) | For a fixed fixture world, 10 scripted questions answered by tool calls return values equal to the engine's own getters (compared to getter output, not wording); a question with no backing data returns "not available". Start with five tools | L |
| **LLM-9** | Injection hardening corpus | At least 20 hostile vault or faction strings (instructions to call tools, exfiltrate the key, write paths, widen the tool set) leave the allowed action set unchanged and write nothing without a click; the corpus runs in CI and only grows | M |
| **LLM-10** | Android and phone | `INTERNET` permission in the export preset; cleartext/LAN policy decided and documented; Android Keystore plugin; phone sheet. On device: feature off shows no network use (as LLM-2); on, a hosted call succeeds over TLS; the key survives a restart and is absent from a `run-as` copy of app storage; with no Keystore and no env var, refuses. Gated on LLM-2 to LLM-4 | L |

Deferred, not in v1: embeddings over the Vault (U8), image generation (U9), a Rust transport (revisit only if testing or streaming throughput demands it), dollar display.

## 4. Security bars (apply to every milestone)

1. **Zero outbound connections when off**, asserted by a test, not by reading code. Turning one capability on opens sockets only for that capability's endpoint class (local or remote).
2. **The key is never in** a save `.zip`, a project archive, an export, `cartalith_settings.cfg`, a preset, `godot.log`, a diagnostic report, an error string or a `Debug` dump. The UI shows the last four characters only. Each location has a grep test with a leak mutation.
3. **Hostile-string corpus** (LLM-9) is a regression gate for every later change to the prompt or tool set.
4. **Model output is data:** never executed as code or used as a path; actions are a closed enum.
5. **Context is opt-in per source**; whole notes, the whole Vault, the save file, file paths and the machine name are never sent unasked. Vault notes can hold private story material and nothing can be marked DM-only yet [verified, `export_dialog.gd`], so this is a default-closed rule.
6. **Consent text must be true:** it names the endpoint, what is sent, that the Q8 ruling means no spend limit is enforced, and links, not paraphrases, the provider's terms.
7. **No automatic retry beyond one**, no auto-run of any action in v1.

## 5. Settings list (`[llm]`, all default off, none holds a secret)

| Setting | Default | Notes |
|---|---|---|
| Assistant enabled (master) | off | gates everything; off means zero sockets |
| Prose and Vault assist | off | LLM-3 |
| Commands: view, layer, parameter | off | needs LLM-5 and LLM-6 present |
| Edit actions: sculpt, paint | off | needs LLM-5 and LLM-7 present; separate switch |
| Local providers allowed | off | localhost and LAN |
| GPU while a local provider is active | Automatic | **Ruling DF.** Automatic / Always low-power GPU / Never change; see s.5a |
| Remote providers allowed | off | hosted endpoints |
| Provider kind, base URL, model | empty | model is free text; never a hardcoded list; "list models" where the server supports it |
| Key source | env var | OS store where available; never a file |
| Token counter visible | on | no cap enforced; optional opt-in cap, default off |
| Context sources allowed | none | per source: current entity, selection, named Vault notes |

Unavailability shows its reason (the `CommandIndex` rule): disabled, offline, key missing, store unavailable, each distinct, never a dead button or a bare `0`.

## 5a. Local-provider GPU rule (Ruling DF)

When a local preset (`Preset::is_local`) is selected the settings screen shows a warning: Cartalith and the language model may compete for the GPU. It always states the detected class (same machine, local network, remote, unknown) and what will happen.

- **Detection** is a pure function of the base-URL host (Ruling DF rule 1); no DNS, no network. Same machine: `localhost`, `127.0.0.0/8`, `::1`, `::ffff:127.0.0.1`. Local network (no switch): `10/8`, `172.16/12`, `192.168/16`, link-local. Remote or unresolvable names: no switch. Userinfo in the URL is refused.
- **Override:** Automatic (default) / Always low-power GPU / Never change.
- **Docker, WSL, tunnels, hostnames, `host.docker.internal`** are not guessed at: the "detected as" line plus the override cover them.
- **What can actually change (findings, 2026-10-25):**
  - **Compute path (wgpu): [verified in the repository]** `cartalith-gpu` (`multi.rs`) already enumerates adapters (`enumerate_devices`) and holds a process-global `GpuPreferences` whose `selected_keys` choose the device; empty means automatic `PowerPreference::HighPerformance` (the discrete card). `WorldGen.gpu_set_selected_devices` sets it and "takes effect on the next generate", so no restart. The rule needs one thing built: choosing the lowest-power key (prefer an integrated adapter, else the software path) and restoring the previous selection when the provider is deselected or the override is Never. Device keys are machine-specific, so the choice is computed at runtime and never saved into a world.
  - **Godot display adapter: [verified in docs, partly recalled]** the Godot 4 command line has `--gpu-index`, "only available on the Forward+/Mobile renderers; run with `--verbose` to get the available device list" (docs.godotengine.org command line tutorial, fetched 2026-10-25; the page is the stable docs, not specifically 4.7). It is a start-up argument, so changing it needs a restart. This project's `project.godot` sets `renderer/rendering_method="gl_compatibility"` [verified], and a forum answer says GPU selection cannot be overridden under OpenGL because the system decides [cited, forum, not confirmed against the engine source]. So the display adapter is **not** switchable by the app in the shipped renderer; the OS per-app graphics preference (on Windows, Settings, Graphics) is the user's lever [recalled]. No project setting for the adapter index turned up [not verified: an empty search is not proof].
  - Therefore in v1 "scale back" means the compute path switches automatically; the display adapter gets the warning only, unless a later check proves otherwise.

## 6. UI forms (DCC shell; design pass LLM-0 first)

- **Desktop:** a docked Assistant panel (message list, input, proposal list with Apply and Dismiss, token counter) and a settings group beside the Preferences and Performance groups in `dcc_settings.gd`; the consent screen is a modal.
- **Tablet:** the same panel in the right dock at the tablet touch-target scale.
- **Phone:** a bottom sheet in the tool-sheet pattern, one proposal at a time; the `CommandIndex` search field is the entry, with a "Do it for me" row on no match. Phone comes after desktop in sequencing (LLM-10).
- Follow `DESIGN_HANDOFF.md` and the three-shell rule; where no canvas exists, derive from the DCC canvases' vocabulary and record it. Probe scenes (`godot --headless <probe>.tscn`) cover the shell; `cargo test` cannot see it.

## 7. Risks

1. **New network surface:** one bug sends private story text to a third party. Mitigation: off by default, consent, opt-in context, the zero-socket test.
2. **No enforced cap (Ruling DD Q8):** a runaway or looping client can spend the user's money. Mitigated only by `max_tokens`, one retry, a visible counter and consent wording; an opt-in cap is the user's remedy. Owner chose this knowingly.
3. **Edit actions by a model:** correctness depends wholly on LLM-5; bound per-proposal size; preview and click always.
4. **Local-model quality varies:** tool calls and JSON output are model-dependent even where the server accepts the field; validate, retry once, then show the failure.
5. **Provider drift:** payloads and model names change; dated fixtures, free-text model.
6. **Android:** TLS roots under `HTTPClient`, the Keystore plugin, cleartext LAN policy and the open SAF Vault milestone are the largest unknowns.
7. **No credential store and no env var** leaves a user unable to use a remote provider; this is intended (refuse), but the message must say how to fix it.
8. **Grouped undo is large:** it touches four undo systems; LLM-5 may reveal that a tool's undo cannot be wrapped without change.

8. **GPU rule overreach:** a wrong "same machine" call (tunnel, WSL) would drop a user to slower generation for nothing. Mitigated by the override, the visible "detected as" line, and switching only the compute path, restored on deselect.

## 8. Owner questions still open

None blocking. Optional: whether the opt-in cap (§5) should exist at all; whether Android Keystore alone suffices or an env var is meaningful on a phone.
