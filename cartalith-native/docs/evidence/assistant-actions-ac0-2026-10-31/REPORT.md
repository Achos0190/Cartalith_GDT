# AC-0: measure first (the assistant's action catalogue)

Date: 2026-10-31. AC-0 of `ASSISTANT_ACTIONS_PROPOSAL.md`, scheduled first by Ruling DW. **No application code
changed.** Everything here is a standalone measurement. `tools/actions.py` builds a **fixture** catalogue from the
shipped sources by regex; it is not AC-1's generator.

AC-0's bar has two halves:
- **Token counts** of today's requests and of AC-2's, with each local tokenizer and Anthropic's own count.
- **A 30-task check** on three local models and the External model: is the correct action chosen, with and without
  pre-retrieval, by browsing and by search?

This session ran everything that needs no model. The model runs need Ollama and the Anthropic key on the owner's
PC; the commands are under "To run on the PC".

**Reproduce the model-free half:**
- `python -I tools/measure_ac0.py --repo <repo> --tok <dir> --out results`
- `python -I tools/run_ac0.py --repo <repo> --out results/mock --models mock-search mock-browse`

The tokenizer files are not committed (9–28 MB each, third-party). They are the `tokenizer.json` of
`Qwen/Qwen3-14B`, `openai/gpt-oss-20b` and `mistralai/Ministral-3-14B-Instruct-2512` on Hugging Face, saved as
`qwen3.tokenizer.json`, `gptoss.tokenizer.json` and `ministral3.tokenizer.json`, and counted with `tokenizers` 0.23.3.

## The fixture catalogue

It has **260 entries**, from these sources:

| Source | Entries |
|---|---:|
| Generation settings (`params.rs` rows less `EXCLUDED`, with `DESCS`) | 114 |
| Menu rows (`menus.gd`) | 74 |
| Context-card actions with a literal label | 30 |
| Map tools (`arm_tool` literals) | 13 |
| Layers (`LIVE_LAYERS`) | 10 |
| Curated windows (`llm_view.gd::WINDOWS`) | 10 |
| Style preset (one action; its 18 presets come from `STYLE_PRESETS`) | 1 |
| Zoom | 1 |
| Selection, year and Context-panel actions (`PC_SHELL_PROPOSAL.md`, proposed) | 7 |

AQ1's exclusion holds in the fixture: there are no file actions, and Save, which the tablet's World builder also
defines, is excluded by id.

**Only 133 of the 260 have a description.** The generation settings carry `DESCS`. Menu rows have one only where a
tooltip follows them; context actions and tools have none. AC-5's work is that gap, made measurable.

**The tree** has 12 top categories: world 117, civil 31, preferences 22, carto 18, story 14, window 14, edit 11,
assets 10, records 8, help 6, map 6, selection 3. It is at most three levels deep. Every list call returns at most 25
entries; one leaf, a settings group, can need a second page.

## Token counts (real tokenizers; text only)

These count the text a request carries: the system prompt, the tool JSON on the OpenAI wire, and any tool results.
A chat template adds each model's own framing on top, so read them as lower bounds; the PC run records the servers'
own counts.

| Request shape | chars/4 | Qwen3 | gpt-oss | Ministral-3 |
|---|---:|---:|---:|---:|
| **Today:** Commands, first turn | 1 958 | 1 963 | 1 914 | 2 038 |
| **Today:** Commands + knowledge, first turn | 2 761 | 2 746 | 2 683 | 2 848 |
| **Today:** Commands after opening the 3 largest groups | 6 163 | **6 787** | **6 617** | **7 097** |
| **Today:** Questions only (knowledge) | 848 | 822 | 807 | 849 |
| **Today:** Edit (region block not counted) | 714 | 732 | 704 | 758 |
| **New:** fixed request (system prompt with the map, 4 catalogue tools) | 696 | **694** | **682** | **716** |
| **New:** fixed request + knowledge tools | 1 499 | 1 477 | 1 451 | 1 526 |
| **New:** the capability map alone | 82 | 136 | 120 | 136 |
| **New:** `list_actions()`, the top | 111 | 115 | 108 | 130 |
| **New:** `list_actions("carto/layers")` | 243 | 242 | 242 | 259 |
| **New:** `list_actions("world/generation/climate")`, page 0 | 239 | 262 | 275 | 280 |
| **New:** `search_actions`, 8 matches | 405 | 401 | 405 | 432 |
| **New:** `describe_action`, 3 ids | 228 | 226 | 226 | 243 |
| **New:** t27 by browsing, the largest of its 3 requests | 1 221 | **1 207** | **1 196** | **1 273** |

Over its three requests, t27 ("Show the political map as it was in the year 300") processes about 2 950 tokens:
Qwen3 2 965, gpt-oss 2 928, Ministral-3 3 102.

**What this says:**

1. **The new fixed request is about a third of today's Commands first turn** (about 700 against about 1 950 tokens),
   and it reaches every one of the 260 actions, not one tool's worth.
2. **A whole two-action request peaks at about 1 200–1 270 tokens**: a sixth of the owner's 8 192 Local window. That
   is §3.6's target, met with room.
3. **Today's worst case is larger than the proposal said.** Opening the three largest settings groups is **6 617–7 097
   tokens** with the real tokenizers, against the analysis's chars÷4 estimate of 6 163. That is 7–15 % over, and
   Ministral-3 reaches 87 % of an 8 192 window before the answer. `ASSISTANT_ACTIONS_PROPOSAL.md` §1's
   "Commands … 6.3k" is low by that much.
4. **chars÷4 is close for tool JSON and prose (within about 5 %), and low for the capability map (82 against
   120–136).** The map is mostly numbers and separators, which tokenize densely. Future estimates of compact
   listings should not use chars÷4.

## Reach on the 30 tasks, with no model

`tools/tasks.json` holds 30 tasks: 7 settings, 6 view, 5 selection and year, 5 windows, 3 edit and tools, 2 with two
actions, and 2 the assistant must decline (a file action, and an impossible one). Each lists the ids that count as
right. The ids are a labelled judgement.

| Measure | Result |
|---|---|
| **Search** finds every needed action in its top 8 | **27 of 28** tasks that need one |
| … in its top 5 (what pre-retrieval injects) | 26 of 28 |
| **Browsing** reaches every needed action | 28 of 28, because every action has a path |
| … with one `list_actions` call per action, using the capability map | 22 of 28 |
| … for the settings tasks | 2 calls each: `world/generation`, then the group |
| List entries a model reads on the cheapest browse path | 1–27 per task |

**The miss:** "Let me draw a new road". The tool is called "Way" and the card action "Start way here"; neither says
"road". The fix is a description or an alias, which is AC-5's work. The settings task "Make the mountains higher"
ranks its answer 7th, because "Peak height" does not say "mountain".

**The greedy baseline** (`mock-search`, not a model) always proposes the top search hit, to show what search alone
would decide:
- It is right on **22 of 30** in every search condition.
- It cannot do the two-action tasks.
- It proposes for both tasks it should decline. "Save the project" becomes *Save diagnostic report*, and **"Delete the
  Pacific Ocean" becomes the menu's *Delete***.

That last case is why the design leaves the choice to the model and keeps three rules in force:
- every proposed id must have been returned in that request;
- nothing runs before Apply;
- a destructive action needs its own card and a second confirmation.

**The plumbing check** (`mock-browse`, which walks the known path) is right 30 of 30 under every rule: ids returned in
this request, at most 3 per describe, at most 5 per card, `MAX_TURNS` 5. That proves only that the runner works.

## To run on the PC

These runs need Ollama with the models pulled, and the Anthropic key in Windows Credential Manager under
`Cartalith/LLM/anthropic`, as for the 2026-10-26 evaluation. Use the same models as that run, so the two can be
compared:

```
python -I tools/run_ac0.py --repo <repo> --out results/models --models qwen3:14b#nothink gpt-oss:20b <ministral-3 tag> claude-sonnet-5-5
python -I tools/run_ac0.py --repo <repo> --out results/models --aggregate
```

`run_ac0.py` imports `ollama_chat` and `claude_chat` unchanged from `../llm-model-eval-2026-10-26/tools/run_eval.py`,
so the reviewed key handling and scrubbing are the only ones. It runs each task under four conditions:
- `browse`: list, describe, propose;
- `search`: search, describe, propose;
- `both`: all four tools;
- `both+pre`: all four, plus the top 5 search hits in the system prompt.

That is 120 runs a model. It records every turn, with Ollama's `prompt_eval_count` and Claude's `input_tokens`, and is
resumable. For Anthropic's own count of the shapes above, its `count_tokens` endpoint needs the same key. That is
still to do on the PC, and the table above has no Claude column until it is.

**What the owner gets back** is one table: correct out of 30 per model and condition, with peak prompt tokens. It
settles the questions this proposal left open:
- **AQ4**: does pre-retrieval help a local model?
- **Browse or search**: which do models use well?
- **AC-6**: is folding the thirteen tools in worth it?

## Limits

- **A fixture, not the catalogue.** Regex over source misses rows whose label is computed (context actions with `%`
  labels are skipped), and the menu-row kinds are assigned by menu, not read. AC-1 replaces this with the generator.
- **Lower bounds.** No chat template is applied, so the token counts are lower bounds. No Claude count was made here.
- **Small n.** 30 tasks with one expected-id judgement each. The model runs use one repetition by default; `--reps`
  raises it.
