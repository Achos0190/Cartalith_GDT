# Every action, few tokens: an action catalogue for the assistant

**A proposal of 2026-10-31.** It defines; it does not track. What has been built is
`cartalith-native/docs/STATUS.md`'s answer. It has no status column.

**The owner's question, verbatim:** *"the assistant function (an implementation to use an either local or
external llm with toolcalling now is very limited. But there already are buttons to basically guide the scoped
actions. Now I'd like to make any and all available actions in cartalith available by this function. But at the
same time we shouldn't bury the agent in a ton of tokens outlining all functions. Is there a proper way to do
this? Analyse and read into the current implementation first."*

**The short answer: yes.**

The proper way is **progressive disclosure over one action catalogue**. The model always sees a small, fixed set
of tools and a short map of what exists. It searches the catalogue for what the user asked, asks for the full
description of the one or two actions it needs, and proposes them by id. The app checks every id against what it
handed out in that same request, validates the arguments, and shows a card that changes nothing until Apply.

This works the same on every provider, local or hosted. It is the pattern this code already uses for generation
parameters, where `get_param_schema` comes before `set_generation_params`, and Ruling DI P4 already rules it the
only one: *"Two-step catalogue only: no one-step 'send the whole catalogue', even for a large-context hosted model,
so there is one behaviour to test."* What is missing is the catalogue itself, and three tools over it.

**How this was read.** A read-only analysis (an agent, 2026-10-31) walked `cartalith-llm`, `cartalith-godot`'s LLM
modules and the shell's `llm_*.gd`, `command_index.gd` and `quick_bar.gd`. The facts this proposal rests on were
then re-checked at their symbols in this session. Where a figure comes from elsewhere, it is cited, not asserted.

---

## 1. What exists

**Two wires, one request type** (`cartalith-llm/src/request.rs::build_request`).
- OpenAI-compatible chat completions (`openai.rs`) serves Ollama, LM Studio, llama.cpp, vLLM, OpenRouter, OpenAI,
  Gemini and custom servers.
- Anthropic Messages (`anthropic.rs`) serves the External profile.
- No request sends `tool_choice`, `cache_control`, `defer_loading` or a context size *(analysis)*.

**Thirteen tools, all declared in Rust** (re-checked by grep over `cartalith-llm/src`):

| Group | Tools |
|---|---|
| Parameters | `get_param_schema`, `set_generation_params` |
| Queries | `get_parameters`, `get_changed_parameters`, `describe_parameter`, `get_ecology_summary`, `sample_point`, `get_world_summary` |
| Knowledge | `search_knowledge`, `read_source`, `related_sources` |
| Proposals | `propose_view_actions`, `propose_edit_actions` |

**A per-turn router already exists** (`params_flow.rs::ParamsConversation`):
- `MAX_TURNS` = 5, with one proposal retry (`MAX_PROPOSAL_RETRIES` = 1).
- A schema call opens a group, and `set_generation_params` then accepts only the keys shown (`MAX_SHOWN_KEYS` = 80).
- Both decoders reject a tool name that was not offered in that request.
- Every proposal has its own Rust validator, which rejects bad numbers rather than clamping them (Ruling DI P1).
- Nothing runs before an Apply click. Apply is one undo group, and a setting regenerates the world (Rulings DD, DS).

**The scoped buttons choose the tool list per request** (`llm_assistant_window.gd::_TASKS`, `ParamsConversation::tools`):

| Button | Tools |
|---|---|
| Describe place, Story, Note | None: the context is assembled by the app |
| Commands | Schema, queries and view tools, then the parameter setter, plus the knowledge tools if ticked |
| Edit | `propose_edit_actions` over a painted region |

**The token budget as built** is the analysis's figures, characters ÷ 4, calibrated at 3.8–4.1 characters per
token against Ollama's own `prompt_eval_count`. They are estimates, not measurements:

| Request | About |
|---|---|
| Commands, first turn | 1.9–2.1k tokens |
| Commands, after opening the three largest parameter groups | 6.3k |
| The same, with the knowledge tools and a pack | 7.9–8.6k |

The owner's Local reference setup runs a context of **8 192** (`cartalith-native/docs/evidence/llm-live-2026-10-26/REPORT.md`, "ctx 8192"). `llm_stream.rs`
caps a reply at `MAX_TOKENS_RANGE` = (16, 8192), and no request-size guard exists. **The present design is already at
the edge of a local window, with about 120 parameters and a handful of view verbs.**

**What "every action" means.** These sources would have to be reachable:

| Source | Size |
|---|---|
| `CommandIndex` | 693 rows, 65 runnable by id; CQ-0 probe, 2026-10-25, cited, not re-run here |
| Menu ids | 130 (`grep -cE '^const ID_[A-Z_0-9]+ := ' menus.gd`) |
| Generation parameters | about 120 |
| Context-card actions | 57 dotted ids (e.g. `civ.edit`) |
| Armed map tools | 13 |
| Curated windows | `llm_view.gd::WINDOWS` |
| The engine bridge | 903 `#[func]` methods *(analysis)*. These are not user actions: they bypass the shell's undo and confirmations |

**Hazards a "do anything" design must respect:**
- **No descriptions.** Enabled menu rows carry no description in the index; only disabled rows index their tooltip.
- **Stale availability.** Availability is a snapshot from when each menu was built.
- **Dialogs.** Many rows open a modal window, and running one from the assistant logs Godot's "exclusive child"
  error (`LLM_ASSISTANT_SCOPE.md` §5b.10).
- **`pinnable` is not a safety mark.** Save, Close, New world, Export and Open project are all pinnable (scope
  §5b.10 item 3).
- **Session mode.** No `is_session_mode()` call appears in `llm_assistant_window.gd` or in `world_workspace.gd`'s
  regenerate path. It is checked from the menus and `app.gd`. Whether a gate further up catches it is not shown by
  grep.
- **Anthropic's server-side tool search would not round-trip.** `anthropic.rs` drops server tool blocks (`_ => {}
  // thinking, server tools, unknown: not shown`).

## 2. What the field does

The evidence was collected by the analysis, with sources. Claims marked *from skill* came from the Anthropic API
reference.

- **Anthropic's tool search** (`tool_search_tool_regex_20251119` / `_bm25_`) marks tools `defer_loading: true`.
  The model loads 3–5 at a time. Every deferred definition is still sent in the request body; it is just kept out
  of the model's context. Anthropic reports definitions of about 55k tokens cut by over 85 %, and tool selection
  degrading past 30–50 loaded tools. (https://platform.claude.com/docs/en/agents-and-tools/tool-use/tool-search-tool)
- **OpenAI** advises fewer than 20 functions at the start of a turn, and offers `allowed_tools` and a deferred
  `tool_search` on its newest models. (https://developers.openai.com/api/docs/guides/function-calling)
- **Local servers have no deferral of any kind.** Ollama's OpenAI endpoint supports tools but not `tool_choice`, and
  cannot set the context size; Ollama's default context is 4k under 24 GiB of VRAM. llama.cpp needs `--jinja` for
  tool calls. (https://docs.ollama.com/api/openai-compatibility, https://docs.ollama.com/context-length,
  llama.cpp `docs/function-calling.md`)
- **RAG-MCP** (Gan et al., 2025) retrieved tool descriptions instead of listing them all. Prompt tokens fell by more
  than half, and tool-selection accuracy rose from 13.6 % to 43.1 %. (https://arxiv.org/abs/2505.03275)
- **MCP** pages its tool lists and expects a human able to deny any call.
  (https://modelcontextprotocol.io/specification/2025-06-18/server/tools)

**The conclusion.** Native deferral exists only on two hosted APIs, and only for their newest models. A design that
must work on Ollama, LM Studio and llama.cpp at an 8k window has to do the disclosure itself, in the app. That is
also what Ruling DI P4 asks for.

## 3. The proposal

### 3.1 One action catalogue

The catalogue is one table, **generated** from the sources that exist, never written by hand in parallel with them.
Its sources:
- `CommandIndex` rows, by `pin_key`;
- the parameter registry;
- `llm_view.gd`'s view verbs and windows;
- the context card's dotted action ids;
- the map tools;
- the selection and the year, from `PC_SHELL_PROPOSAL.md`.

**Each entry carries:**

| Field | What |
|---|---|
| `id` | stable across builds: `menu:world.generate`, `param:hydrology.sea_level`, `view.layer.political`, `civ.settle`, `window.factions`, `select`, `timeline.set_year` |
| `title`, `desc` | the row's label, and **one line of at most 120 characters**: from the row's tooltip, the parameter's `DESCS`, or a curated table for the rest |
| `group` | domain and menu: `world/generation/hydrology`, `civil/settlements`, `carto/style`, `story/time`, `window` |
| `kind` | `query`, `view`, `select`, `setting`, `edit`, `window`, `file` |
| `args` | a typed JSON schema; most actions take none or one |
| `effect` | `read`, `view` (no undo needed), `undoable`, `regenerates`, `opens-window`, `destructive` |
| availability | `available` / `why`, **evaluated when the request is built**, never the menu-build snapshot |

**The safety rule stays the existing one.** It is a curated allow-list intersected with the index, plus Rust's
forbidden stems (`view_actions.rs`). The bridge's `#[func]`s are never in the catalogue. File actions (Save, Close,
New world, Open, Export) are out of v1 (AQ1).

### 3.2 Three new tools, always offered

| Tool | What it returns | Target size |
|---|---|---|
| `search_actions(query, group?)` | up to 8 matches: `{id, title, desc, kind, available}`, ranked by BM25 over title, description and aliases, the ranker `search_knowledge` already uses (`cartalith-knowledge/src/search.rs`) | about 40 tokens a match |
| `describe_action(id)` | one action's `args` schema, effect and one example | 150–300 tokens |
| `propose_actions([{id, args}])` | nothing to the model; the shell gets cards. Each id must have come back from `search_actions` or `describe_action` **in this request**, which is Ruling DO's rule for knowledge ids; the arguments are validated against the schema, rejected not clamped | — |

**Additive first.** The thirteen existing tools and their scoped buttons stay as they are: they work and have their
corpora. The catalogue path is how the assistant reaches everything else. Folding the specialised tools into the
catalogue is a later, separate step (AC-6), done only once the measurements in AC-0 say it costs nothing.

### 3.3 A map instead of a manual

The system prompt gains a **capability map** of about 200 tokens. It is generated from the catalogue's groups, with
counts, for example:

> world: generation (114 settings), sculpt (13 tools), erosion; civil: settlements, factions, routes, journeys, landmarks; carto: layers, style, labels, export; story: time, conflicts, notes; windows; selection and year.

That tells the model what to search for, without a single definition.

### 3.4 The scoped buttons become action packs

Each existing button preloads the three to eight actions its task needs, already described. That way a small local
model has a direct path, and does not have to discover search. Ruling DO records that some local models never call
tools at all. For those, **pre-retrieval** is extended from knowledge to actions: before the first turn, the app runs
`search_actions` over the user's message and puts the top five descriptions in the context. That is RAG-MCP's
method, and the same switch that pre-retrieves knowledge today turns it on.

### 3.5 How an action runs

Every action runs through the **same path as a person's click**, never a new one:
- menu rows through `CommandIndex.run_row` / `DccQuickBar.run_pin_on`;
- parameters through `undo_group.write_param`;
- view changes through `LlmView.apply`;
- region edits through `llm_edit.gd`;
- selection and year through `PC_SHELL_PROPOSAL.md`'s `DccApp.select` and `tl_set_year`.

Then:
- **Cards, not actions.** A proposal is a card in the Ask tab (`PC_SHELL_PROPOSAL.md` §3.5), and nothing runs before
  Apply (scope bar 7, *"no auto-run of any action in v1"*).
- **Undo.** One Apply is one undo group (Ruling DD). A card may hold up to five actions, applied together as one step
  (AQ3).
- **Dialogs** are opened and left to the person. The model never operates inside one, so `opens-window` actions are
  open-only. With `PC_SHELL_PROPOSAL.md`'s non-modal windows, opening one no longer blocks anything.
- **Session mode** is checked on every Apply, and an action it refuses shows the refusal on the card (AC-3 fixes the
  gap in §1).
- **Destructive actions** are never batched, and need a second confirmation on their own card.

### 3.6 The budget, and a guard

These are the design targets; AC-0 measures them with real tokenizers:

| Part | Target |
|---|---|
| Fixed per request: system prompt, capability map, the three catalogue tools, the knowledge tools | ≤ 1.5k tokens |
| Each search | ≤ 400 tokens |
| Each description | ≤ 300 tokens |
| A typical three-turn request ("show the political layer and go to year 398") | ≤ 3.5k tokens |

That fits an 8k local window with room for the answer, and does not grow with the number of actions. A request-size
guard, missing today, refuses a request that would not fit the profile's context, or drops the pack first, and says
so. It never truncates silently.

### 3.7 What is not proposed

- **Native deferred loading** (Anthropic's tool search, OpenAI's `tool_search`). It would send the whole catalogue
  to the hosted wires only. That is the second behaviour Ruling DI P4 rules out ("so there is one behaviour to
  test"), and `anthropic.rs` would need to round-trip server tool blocks. Reopening P4 is AQ5, asked by name.
- **Programmatic tool calling**: Anthropic-hosted code execution, Anthropic-only.
- **Any action that skips the click path.**

## 4. Milestones

| ID | Delivers | Bar |
|---|---|---|
| AC-0 | Measure first | Token counts of today's requests and of AC-2's, with each local tokenizer and Anthropic's token count. A 30-task script on three local models and the External model: correct action chosen, with and without pre-retrieval |
| AC-1 | The catalogue generator | Every runnable `CommandIndex` row, parameter, view verb, window, context action and map tool has an entry with a stable id, a description of at most 120 characters and an effect. Availability is evaluated per request. The ids are identical across two builds |
| AC-2 | The three tools in `cartalith-llm`, and the request-size guard | Each tool's wire size within §3.6's targets. An id not returned in the request is rejected. The guard refuses an oversized request with a message |
| AC-3 | Execution through the click paths | Every `kind` applies through its existing path in one undo group. Session mode refuses on the card. A destructive action cannot share a card |
| AC-4 | Action packs and pre-retrieval | Each scoped button preloads its pack. Pre-retrieval of actions raises AC-0's local score, or stays off |
| AC-5 | Descriptions and the corpus | Every entry has a description, and none is a placeholder. The hostile-string corpus is extended to the catalogue (scope §4 bar 3) and runs green |
| AC-6 | Fold the specialised tools in (optional) | Only if AC-0's numbers show no loss. The thirteen tools become catalogue entries plus packs |
| AC-7 | The Ask tab's cards per kind | Setting, view, select, edit and window cards, as `PC_SHELL_PROPOSAL.md` §3.5 draws them |

## 5. Owner questions

Each has a default.

- **AQ1.** Which kinds are in v1? *Default: query, view, select, setting, edit and window (open only). File actions
  (Save, Close, New world, Open, Export) are left out.*
- **AQ2.** Should actions that open a window open it and stop? *Default: yes; the model never works inside a dialog.*
- **AQ3.** Today one proposal ends the request. Should one card instead hold several actions applied as one step?
  *Default: yes, up to five; never a destructive one among them.*
- **AQ4.** Should pre-retrieval of actions be on for local profiles? *Default: on, if AC-0 shows a gain; the same
  switch as knowledge pre-retrieval (Ruling DO).*
- **AQ5.** Should Ruling DI P4 be reopened for native deferred loading on hosted models? *Default: no; one behaviour
  everywhere.*
- **AQ6.** Should the design aim at the 8 192 window of the Local reference, or at Ollama's default 4k? *Default:
  8 192, with the guard refusing below it; a profile that runs at 4k gets a warning.*

## 6. Limits

- **Figures.** The token figures in §1 are the analysis's estimates from rebuilt request strings, not tokenizer
  counts. §3.6's are targets. AC-0 replaces both.
- **Unconfirmed counts.** The `CommandIndex` counts are from the CQ-0 probe (2026-10-25), not re-run here, and the
  900-odd bridge methods are counted by the analysis only.
- **Nothing was built.**
