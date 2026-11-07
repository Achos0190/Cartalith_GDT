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
of tools and a short map of what exists. It asks the program what is available: by type, from the categories down to
a list of commands, or by search. Then it asks for the full description of only the few actions it needs, and
proposes them by id. The app checks every id against what it
handed out in that same request, validates the arguments, and shows a card that changes nothing until Apply.

This works the same on every provider, local or hosted. It is the pattern this code already uses for generation
parameters, where `get_param_schema` comes before `set_generation_params`, and Ruling DI P4 already rules it the
only one: *"Two-step catalogue only: no one-step 'send the whole catalogue', even for a large-context hosted model,
so there is one behaviour to test."* What is missing is the catalogue itself, and four tools over it.

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
the edge of a local window, with about 120 parameters and a handful of view verbs.** *(Measured by AC-0, 2026-10-31,
with real tokenizers: the worst case above is 6 617 to 7 097 tokens, not 6.3k, and the first turn 1 914 to 2 038. See
`cartalith-native/docs/evidence/assistant-actions-ac0-2026-10-31/REPORT.md`.)*

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

### 3.2 Four new tools, always offered

The model can find an action two ways. It can **browse by type**, from the categories down to a list of commands,
as a person reads a menu. Or it can **search** when the request already names the thing. Either way it then asks
for the full description of only the few actions it means to use. This is how the settings work today:
- `get_parameters` with no argument returns *"the groups with how many settings and how many changed"*, and with a
  group, that group's settings (`query.rs`);
- `get_param_schema` opens at most three groups (`catalogue.rs::MAX_SCHEMA_GROUPS` = 3);
- the setter accepts only keys from groups opened in that request.

The four tools give every action the same two-step discipline (Ruling DI P4).

| Tool | What it returns | Target size |
|---|---|---|
| `list_actions(path?, kind?)` | **Browse.** With no path: the top categories (domains and menus) with counts. With a path such as `civil/settlements`: its sub-categories with counts, or, at a leaf, its actions one line each (`{id, title, available}`, no schemas). `kind` filters by type of function (`query`, `view`, `select`, `setting`, `edit`, `window`), so "list the view commands for CARTO" is one call. At most 25 entries per call; a longer list pages | about 12 tokens an entry, 300 a call |
| `search_actions(query, group?)` | up to 8 matches: `{id, title, desc, kind, available}`, ranked by BM25 over title, description and aliases, the ranker `search_knowledge` already uses (`cartalith-knowledge/src/search.rs`) | about 40 tokens a match |
| `describe_action(ids)` | up to three actions' `args` schemas, effects and one example each, as `get_param_schema` opens up to three groups | 150–300 tokens an action |
| `propose_actions([{id, args}])` | nothing to the model; the shell gets cards. Each id must have come back from `list_actions`, `search_actions` or `describe_action` **in this request**, which is Ruling DO's rule for knowledge ids; the arguments are validated against the schema, rejected not clamped | — |

**How a model refines.** A typical path:
1. The user asks *"make the map look old and show who held what in 400"*.
2. The model reads the capability map (§3.3).
3. It browses with `list_actions("carto", kind: "view")` and `list_actions("story/time")`, two calls that can go in one
   reply.
4. It calls `describe_action` on the style preset, the Political layer and `timeline.set_year`.
5. It proposes one card.

That is three model turns, within today's `MAX_TURNS` of 5. Calls in one reply run together, within the existing
lookup caps (`query.rs::MAX_QUERY_CALLS_PER_REPLY` = 6, `MAX_QUERY_CALLS_PER_REQUEST` = 12).

**What keeps it safe.** The tree, the lists and the descriptions are built by the app from the catalogue, never from
text the model has read. So browsing can only reveal actions the app offers, and it cannot widen the set (scope V6).

**Additive first.** The thirteen existing tools and their scoped buttons stay as they are: they work and have their
corpora. The catalogue path is how the assistant reaches everything else. Folding the specialised tools into the
catalogue is a later, separate step (AC-6), done only once the measurements in AC-0 say it costs nothing.

### 3.3 A map instead of a manual

The system prompt gains a **capability map** of about 200 tokens. It is the first level of `list_actions`' tree,
sent up front so the model can skip the first browse call. It is generated from the catalogue's groups, with counts,
for example:

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
| Fixed per request: system prompt, capability map, the four catalogue tools, the knowledge tools | ≤ 1.6k tokens |
| Each list or search | ≤ 400 tokens |
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
| AC-2 | The four tools in `cartalith-llm`, and the request-size guard | Each tool's wire size within §3.6's targets. The tree is at most three levels deep, with at most 25 entries a call. An id not returned in the request is rejected. The guard refuses an oversized request with a message. AC-0's script runs both ways, by browsing and by search, and reports each |
| AC-3 | Execution through the click paths | Every `kind` applies through its existing path in one undo group. Session mode refuses on the card. A destructive action cannot share a card |
| AC-4 | Action packs and pre-retrieval | Each scoped button preloads its pack. Pre-retrieval of actions raises AC-0's local score, or stays off |
| AC-5 | Descriptions and the corpus | Every entry has a description, and none is a placeholder. The hostile-string corpus is extended to the catalogue (scope §4 bar 3) and runs green |
| AC-6 | Fold the specialised tools in (optional) | Only if AC-0's numbers show no loss. The thirteen tools become catalogue entries plus packs |
| AC-7 | The Ask tab's cards per kind | Setting, view, select, edit and window cards, as `PC_SHELL_PROPOSAL.md` §3.5 draws them |

### Build notes for AC-1 and AC-2 (2026-11-07)

What the build decided where this document left a choice, and what it found. Design notes, not a status column:
`cartalith-native/docs/STATUS.md` says what is built.

- **Where the generator lives.** `cartalith_llm::actions` builds the table from four inputs: the registry
  catalogue (parameters), `ViewCatalogue` (layers, the style preset, zoom, the landmark filter, windows),
  `ShellFacts` (the runnable `CommandIndex` rows, whether a world is open, the year track; read per request by
  `shell/llm_actions.gd`), and a committed scan of the shell source for the context-card actions and the map
  tools, which have no runtime registry (`data/action_static_sources.json`, with a test that fails when it
  goes stale). Nothing is typed in parallel with a source.
- **Menu-row ids are the `DccMenus` constant's name** (`menu:factions`), not the numeric `pin_key`, so an id
  survives a renumbering. Two constants can share a value (`ID_DESELECT` and `ID_LOD_EXPORT_ATLAS`); the row's
  title picks the name.
- **`pinnable` was confirmed not to be a safety mark, on the 66 real rows**: Save, Save as, Close, New world,
  Open project, Revert, three Exports, three Imports, Delete, Cut, Paste, Clear library, Clear caches and the
  Reset rows are all pinnable. The catalogue drops every entry whose id or title names a forbidden word (the
  view tool's own list plus six more) and records it; a row nobody reviewed is listed with effect `unreviewed`,
  `available: false`, and cannot be proposed. The `effect` vocabulary has no `destructive` member.
- **Windows are listed once.** A menu row that opens a curated window is merged into `window.<id>`.
- **Refusals that cost a harmless entry** (found by the real-row run, kept because a false positive is cheaper
  than a false negative): `Undo history`, `Undo last stamp` and `Undo` (the stem `undo`), `Start export
  region here` (`export`), `Discard ... draft` and `Erase` (`discard`, `erase`). AC-0's task t24, "Undo that",
  therefore has no offered action. A parameter's title is not checked (`carve_rivers` says "on generation"):
  its id still is.
- **`list_actions` and `search_actions` rows omit `available` when it is true** and carry `available: false`
  only on an action that cannot run now. With the explicit field the worst search was 1 615 characters, 15 over
  the 1 600 that 400 tokens is at AC-0's measured 4.0 characters a token.
- **A `list_actions` page stops at 25 rows or 1 400 characters, whichever comes first.** 25 rows of the
  `erosion` settings group were 2 086 characters (522 tokens) before the character cap.
- **Search is BM25 over title and aliases (twice), id, path and description**, with a small curated alias table
  for the two AC-0 misses ("road" now finds the Way tool, "mountains" finds Peak height).
- **The guard** divides the request body by 3.4 characters a token (AC-0's worst measured ratio was 3.47) and
  adds 8 tokens a message, then refuses when the prompt plus the request's own answer cap exceeds the window.
  It applies only to a local endpoint and only when the catalogue tools are in the request; the window is the
  8 192 of AQ6 unless the shell passes a context size.
- **`propose_actions` produces a PREVIEW card with no Apply.** Running a card through each kind's click path is
  AC-3. The selection verbs (`select`, `selection.back`) and `timeline.play`/`record_year` are not offered
  because no entry exists for them yet (`DccApp.select` has since appeared with PC-1; `timeline.play` and `record_year` have no shell code); `timeline.set_year` is, over `tl_set_year`.

## 5. Owner questions

**Answered: the design is approved and every default below is taken (Ruling DW, 2026-10-31), with AC-0 first.**

Each had a default.

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
