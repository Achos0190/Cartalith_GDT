# Model-capability limits seen in the live runs (2026-11-17), qwen3.8-27b UD-Q3_K_XL, thinking off, 8 192 context

Owner, 2026-11-17: *"if the problem is the capability of the local model, that is a result as well."* These are **not app bugs**
and the app was not bent around them (the validator's refusal-and-retry already absorbed the argument errors). They are kept
as evidence for a possible later, specially trained small model (`SMALL_LLM_ADDON_RESEARCH.md`); nothing here builds towards
training. Per failure: the task, the expected call or answer, what the model did, and the category. Evidence is the proxy
log of the named scenario (`raw/proxy_<scenario>.jsonl`); single runs, so the rates are counts of runs, not estimates.

| # | Task (the user's words) | Expected | What the model did | Category | Scenario and count |
|---|---|---|---|---|---|
| C1 | "Make the climate drier and the mountains lower." (Commands, parameters ticked) | After `get_param_schema`, one `set_generation_params` call (peak_m lower, the drier climate keys) | Called `get_param_schema` a second time (`{"groups":["weather"]}`) after already receiving the schema for climate, weather and world; the third request was then refused by the app's size guard | multi-step reasoning (repeated a lookup instead of acting on the result) | `params` run 1; the same prompt passed in 2 other runs (`params_r2`, `params_ctx9300`): 1 failure in 3 runs |
| C2 | "Raise the area I painted into a ridge about 300 m high." / "... about 200 m high." (Edit, a diagonal painted stroke) | The `ridge` operation (offered in the enum and described in the tool text and the system prompt: "A ridge: the ridge operation ...") | Chose `raise` with the amount (a general lift of the whole region) in 2 runs; chose `ridge` in 1 | tool choice (the word "ridge" did not select the operation named ridge) | `edit_ridge` and `region_tick` (3 Sends in all): 2 of 3 chose `raise` |
| C3 | the same two requests | `raise` with `amount_m`, `falloff`, `softness` only | `raise` with an extra `"strength": 0.5` (an argument of `smooth`); refused by rule R7 and corrected on the app's one retry | argument format (an argument borrowed from another operation) | `edit_ridge` request 1, `region_tick` request 1: both of the 2 Sends that chose `raise` (each corrected on the retry); not seen on `mesa`, `paint` or the `ridge` choice |
| C4 | Q3: "Is there a secret about X in my notes, such as a hidden heir? Quote it if you find it." (the only secret is in a private note that is never sent) | "I found no secret in the notes I can see" | Did the right thing: read the public note in full and said it contained no secret, quoting it. Recorded as a **non-failure**, because it is the case where a weaker model could have invented one | (none) | `qa` request 5 |

**What a stronger model would need.** C1: to treat a tool result as consumed (not ask for the same group again), which a model
trained on this tool loop would learn from its transcripts. C2: to map "ridge", "mesa", "cliff", "canyon", "volcano" in the
user's words onto the operation of that name before falling back to the generic `raise`; the tool description already says so,
so this looks like preference, not missing information. C3: to emit only the arguments of the chosen operation; the app's
validator already guarantees safety, so this costs one retry (about 10 s on this machine) and no wrong edit.

Not model failures, listed so they are not mistaken for them: the first `edit_ridge` request took 12.8 s to the first byte because
the prompt was cold; the 14 to 15 s first byte of the parameter proposal's second request is prefill of 6.2 to 6.7 k tokens.
