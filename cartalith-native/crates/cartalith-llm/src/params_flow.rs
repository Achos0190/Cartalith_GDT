//! The two-step parameter conversation (`LLM_ASSISTANT_SCOPE.md` s.5b.1-s.5b.2,
//! milestones LLM-6a/6b): one user request, several model turns, one proposal.
//!
//! WHY a state machine here and not in GDScript: every turn is a separate HTTP
//! request whose body must replay the conversation (the assistant's tool calls
//! and the app's tool results) in each wire's own shape, and which tools are
//! offered depends on what the model has already been shown. Keeping that in
//! Rust keeps the closed-tool-set rule (s.2: "the tool set is fixed per request
//! by the app, never widened by text the model reads") in one tested place.
//!
//! The turn loop:
//!
//! 1. The first request offers ONLY `get_param_schema`.
//! 2. A schema call that names valid groups is answered with their entries and
//!    the next request offers `get_param_schema` plus `set_generation_params`,
//!    whose key enum is exactly the keys just shown. A schema call naming bad
//!    groups is answered with the error (naming the valid groups).
//! 3. A `set_generation_params` call is validated ([`Catalogue::validate`]). If the
//!    outcome rule wants a retry (a structural failure, or nothing valid) and the
//!    one retry is unused, the errors go back as the tool result
//!    ([`Step::Retry`]); otherwise the verdict is the proposal ([`Step::Proposal`]).
//! 4. A reply with no tool call is the model answering in words ([`Step::Answer`]):
//!    "this cannot be done with these settings" is a legitimate outcome.
//! 5. After [`MAX_TURNS`] model replies the conversation stops ([`Step::Failed`]).
//!
//! MUST NEVER: apply anything (this module has no setter to call); offer
//! `set_generation_params` before a schema was shown; let text the model read
//! widen the tool set or the opened groups.

use serde_json::{Value, json};

use crate::catalogue::{Catalogue, GET_SCHEMA_TOOL, SET_PARAMS_TOOL, Verdict};
use crate::openai::{ChatRequest, MaxTokensField, Message, Role, Thinking, ToolCallRecord, ToolSpec, ToolTurn};
use crate::prompt::MAX_REQUEST_CHARS;

/// Most model replies one user request may take. Copied from the stage-1
/// prototype's `MAX_TURNS` (`docs/evidence/llm-model-eval-2026-10-26/tools/run_eval.py`),
/// the bound the measured runs used: schema, proposal, one retry, and slack for a
/// model that asks for a wrong group first.
pub const MAX_TURNS: usize = 5;

/// The most settings one request may have opened across all its schema calls
/// (the union of the groups shown, scope s.5b.1/5b.8). [judgement, 2026-10-26]:
/// 80 is above the three largest groups together (erosion 29 + volcanism 19 +
/// tectonics 16 = 64 offered keys on 2026-10-26) so any reasonable request fits,
/// and below the 114 offered keys so the V6 set and the proposal tool's key enum
/// cannot grow to the whole catalogue by repeated calls. Each call is still
/// capped at three groups by the catalogue.
pub const MAX_SHOWN_KEYS: usize = 80;

/// Automatic retries after a proposal the validator refused (scope s.5b.2
/// outcome rule and s.4 item 7: "allows **one** automatic retry").
pub const MAX_PROPOSAL_RETRIES: usize = 1;

/// One call the decoder delivered (already a JSON object with an offered name).
#[derive(Debug, Clone, PartialEq)]
pub struct IncomingCall {
    /// Provider id, absent from some local servers.
    pub id: Option<String>,
    /// Tool name.
    pub name: String,
    /// Arguments object.
    pub arguments: Value,
}

/// What a model reply leads to.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Tool results were appended: send the next request.
    Continue,
    /// The proposal was refused and the errors were sent back for the one
    /// retry; the verdict is shown to the user as the reason.
    Retry(Verdict),
    /// Terminal: the proposal to show on the card.
    Proposal(Verdict),
    /// Terminal: the model answered in words without proposing.
    Answer(String),
    /// Terminal: the conversation could not finish, in words.
    Failed(String),
}

/// One user request's conversation.
#[derive(Debug, Clone)]
pub struct ParamsConversation {
    messages: Vec<Message>,
    shown_groups: Vec<String>,
    shown_keys: Vec<String>,
    turns: usize,
    retries: usize,
    next_id: u64,
    done: bool,
}

impl ParamsConversation {
    /// Start with the catalogue's system prompt and the user's words. Refuses an
    /// empty request or one over the prose path's own [`MAX_REQUEST_CHARS`].
    pub fn new(cat: &Catalogue, request: &str) -> Result<Self, &'static str> {
        let request = request.trim();
        if request.is_empty() {
            return Err("Type a request first.");
        }
        if request.chars().count() > MAX_REQUEST_CHARS {
            return Err("The request is too long; shorten it.");
        }
        Ok(ParamsConversation {
            messages: vec![Message::text(Role::System, cat.system_prompt()), Message::text(Role::User, request)],
            shown_groups: Vec::new(),
            shown_keys: Vec::new(),
            turns: 0,
            retries: 0,
            next_id: 1,
            done: false,
        })
    }

    /// The groups the model has opened so far (the union of its valid schema calls).
    pub fn shown_groups(&self) -> &[String] {
        &self.shown_groups
    }

    /// Model replies consumed so far.
    pub fn turns(&self) -> usize {
        self.turns
    }

    /// True once a terminal step was returned.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// The messages so far (for tests and the stage-2 driver's transcript).
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// The tools the NEXT request offers: the schema tool always, the proposal
    /// tool only once a schema was shown, with its enum of the shown keys.
    pub fn tools(&self, cat: &Catalogue) -> Vec<ToolSpec> {
        let mut t = vec![cat.tool_get_schema()];
        if !self.shown_keys.is_empty() {
            t.push(Catalogue::tool_set_params(&self.shown_keys));
        }
        t
    }

    /// The next request.
    pub fn request(
        &self,
        cat: &Catalogue,
        model: &str,
        max_tokens: u32,
        max_tokens_field: MaxTokensField,
        include_usage: bool,
        thinking: Thinking,
    ) -> ChatRequest {
        ChatRequest {
            model: model.to_owned(),
            messages: self.messages.clone(),
            tools: self.tools(cat),
            max_tokens,
            max_tokens_field,
            include_usage,
            thinking,
        }
    }

    fn finish(&mut self, s: Step) -> Step {
        self.done = true;
        s
    }

    /// Feed one finished model reply: its text and its tool calls, in order.
    /// `cat` is the catalogue rebuilt from the registry for this reply.
    pub fn on_reply(&mut self, cat: &Catalogue, text: &str, calls: Vec<IncomingCall>) -> Step {
        if self.done {
            return Step::Failed("This request is already finished.".into());
        }
        self.turns += 1;
        if calls.is_empty() {
            let t = text.trim();
            return if t.is_empty() {
                self.finish(Step::Failed("The model returned no text and no proposal.".into()))
            } else {
                self.finish(Step::Answer(t.to_owned()))
            };
        }
        // Record the assistant turn, giving every call an id so its result pairs.
        let records: Vec<ToolCallRecord> = calls
            .into_iter()
            .map(|c| {
                let id = c.id.filter(|i| !i.trim().is_empty()).unwrap_or_else(|| {
                    let id = format!("call_cartalith_{}", self.next_id);
                    self.next_id += 1;
                    id
                });
                ToolCallRecord { id, name: c.name, arguments: c.arguments }
            })
            .collect();
        self.messages.push(Message { role: Role::Assistant, content: text.to_owned(), tool: Some(ToolTurn::Calls(records.clone())) });

        // A proposal ends the conversation unless the outcome rule grants the retry.
        if let Some(p) = records.iter().find(|r| r.name == SET_PARAMS_TOOL) {
            let verdict = cat.validate(&p.arguments, &self.shown_groups);
            let can_retry = verdict.wants_retry() && self.retries < MAX_PROPOSAL_RETRIES && self.turns < MAX_TURNS;
            if !can_retry {
                return self.finish(Step::Proposal(verdict));
            }
            self.retries += 1;
            let feedback = verdict.feedback().to_string();
            for r in &records {
                let body = if r.name == SET_PARAMS_TOOL {
                    feedback.clone()
                } else {
                    json!({"error": "answer ignored: a proposal was made in the same turn"}).to_string()
                };
                self.messages.push(Message { role: Role::User, content: body, tool: Some(ToolTurn::Result { call_id: r.id.clone() }) });
            }
            return Step::Retry(verdict);
        }

        // Otherwise every call is a schema call (the decoder admits only offered names).
        for r in &records {
            let body = if r.name == GET_SCHEMA_TOOL {
                match cat.schema_for(r.arguments.get("groups")) {
                    Ok((groups, value, keys)) => {
                        // UNION, never replace: the live comparison of 2026-10-26 showed a
                        // second call (world, then volcanism) dropping the first call's keys
                        // and costing Claude its volcano card (REPORT.md, Defects 1). Order is
                        // first-shown first, so the offered enum is stable across calls.
                        let mut g = self.shown_groups.clone();
                        for x in groups {
                            if !g.contains(&x) {
                                g.push(x);
                            }
                        }
                        let mut k = self.shown_keys.clone();
                        for x in keys {
                            if !k.contains(&x) {
                                k.push(x);
                            }
                        }
                        if k.len() > MAX_SHOWN_KEYS {
                            // Nothing is added: the earlier groups stay offered and the model
                            // is told why, so a loop of schema calls cannot widen V6's set
                            // without bound.
                            json!({"error": format!(
                                "that would open more than {MAX_SHOWN_KEYS} settings in this request;                                  propose with the groups already opened ({})",
                                self.shown_groups.join(", "))})
                            .to_string()
                        } else {
                            self.shown_groups = g;
                            self.shown_keys = k;
                            value.to_string()
                        }
                    }
                    Err(e) => json!({"error": e}).to_string(),
                }
            } else {
                json!({"error": format!("unknown tool {}", r.name)}).to_string()
            };
            self.messages.push(Message { role: Role::User, content: body, tool: Some(ToolTurn::Result { call_id: r.id.clone() }) });
        }
        if self.turns >= MAX_TURNS {
            return self.finish(Step::Failed(format!(
                "The model did not make a proposal within {MAX_TURNS} replies, so the request was stopped."
            )));
        }
        Step::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::tests::{WORKED, fixture};

    fn call(id: Option<&str>, name: &str, args: Value) -> IncomingCall {
        IncomingCall { id: id.map(str::to_owned), name: name.into(), arguments: args }
    }

    fn req(c: &ParamsConversation, cat: &Catalogue) -> ChatRequest {
        c.request(cat, "m", 512, MaxTokensField::MaxTokens, false, Thinking::Default)
    }

    /// Protects the two-step flow end to end on the worked example: the first
    /// request offers only the schema tool; the schema call opens three groups;
    /// the second request offers both tools with the key enum of exactly the
    /// shown keys; the proposal yields the five changes. MUTATION: offer
    /// set_generation_params from the first turn, and the first assertion fails.
    #[test]
    fn worked_example_through_the_conversation() {
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "Make the climate drier and the mountains lower.").unwrap();
        let names = |r: &ChatRequest| r.tools.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(&req(&c, &cat)), vec![GET_SCHEMA_TOOL]);
        let s = c.on_reply(&cat, "", vec![call(Some("call_1"), GET_SCHEMA_TOOL, json!({"groups": ["weather", "world", "tectonics"]}))]);
        assert_eq!(s, Step::Continue);
        let r = req(&c, &cat);
        assert_eq!(names(&r), vec![GET_SCHEMA_TOOL, SET_PARAMS_TOOL]);
        let en = r.tools[1].parameters["properties"]["changes"]["items"]["properties"]["key"]["enum"].as_array().unwrap().len();
        assert_eq!(en, 10, "weather 4 + world 2 + tectonics 4 in the fixture");
        // The replayed conversation: system, user, assistant(call), tool result.
        assert_eq!(r.messages.len(), 4);
        assert!(matches!(&r.messages[3].tool, Some(ToolTurn::Result { call_id }) if call_id == "call_1"));
        assert!(r.messages[3].content.contains("\"climate.rain_dep\""));
        let args: Value = serde_json::from_str(WORKED).unwrap();
        let Step::Proposal(v) = c.on_reply(&cat, "", vec![call(Some("call_2"), SET_PARAMS_TOOL, args)]) else { panic!() };
        assert_eq!(v.valid.len(), 5);
        assert!(c.is_done() && c.turns() == 2);
    }

    /// Protects the one retry: a refused proposal goes back once with its errors
    /// and every call of the turn is answered; a second refused proposal is the
    /// final card (no third request). MUTATION: MAX_PROPOSAL_RETRIES 1 -> 2.
    #[test]
    fn one_retry_then_the_card() {
        assert_eq!(MAX_PROPOSAL_RETRIES, 1);
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "drier").unwrap();
        c.on_reply(&cat, "", vec![call(None, GET_SCHEMA_TOOL, json!({"groups": ["weather"]}))]);
        let bad = json!({"changes": [{"key": "climate.aridity", "value": 0.8}]});
        let Step::Retry(v) = c.on_reply(&cat, "", vec![call(None, SET_PARAMS_TOOL, bad.clone())]) else { panic!() };
        assert_eq!(v.rejected.len(), 1);
        let last = c.messages().last().unwrap();
        assert!(last.content.contains("V5"), "{}", last.content);
        assert!(matches!(&last.tool, Some(ToolTurn::Result { call_id }) if call_id == "call_cartalith_2"), "ids assigned and paired");
        let Step::Proposal(v2) = c.on_reply(&cat, "", vec![call(None, SET_PARAMS_TOOL, bad)]) else { panic!() };
        assert!(v2.valid.is_empty() && c.is_done());
    }

    /// Protects V6 through the flow: a key from a group the model never opened
    /// is refused even if its text asked for it; bad groups get the error naming
    /// the valid ones and open nothing; text without a call is an answer.
    #[test]
    fn groups_are_opened_only_by_a_valid_schema_call() {
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "more factions").unwrap();
        assert_eq!(c.on_reply(&cat, "", vec![call(None, GET_SCHEMA_TOOL, json!({"groups": ["factions"]}))]), Step::Continue);
        assert!(c.shown_groups().is_empty());
        assert!(c.messages().last().unwrap().content.contains("world, tectonics, weather, civ"));
        assert_eq!(c.on_reply(&cat, "", vec![call(None, GET_SCHEMA_TOOL, json!({"groups": ["weather"]}))]), Step::Continue);
        let Step::Retry(v) = c.on_reply(&cat, "", vec![call(None, SET_PARAMS_TOOL, json!({"changes": [{"key": "civ.factions", "value": 9}]}))]) else { panic!() };
        assert_eq!(v.rejected[0].rule, crate::catalogue::Rule::V6);
        let mut d = ParamsConversation::new(&cat, "delete the Pacific").unwrap();
        assert_eq!(d.on_reply(&cat, "I cannot delete a named ocean.", vec![]), Step::Answer("I cannot delete a named ocean.".into()));
    }

    /// Protects the replay on BOTH wires: after a schema turn, the OpenAI body has
    /// an assistant message with `tool_calls` (arguments re-encoded as a string)
    /// and a `role: tool` message carrying `tool_call_id`; the Anthropic body has
    /// an assistant `tool_use` block (input an object) and a user `tool_result`
    /// block with the same id, two results of one turn in ONE user message, and
    /// no system message among `messages`. The thinking switch is written only on
    /// the OpenAI wire, as the field each variant names. MUTATION: send the
    /// arguments as an object on the OpenAI wire, or split the two results.
    #[test]
    fn both_wires_replay_the_tool_turns() {
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "drier").unwrap();
        c.on_reply(&cat, "Looking.", vec![
            call(Some("t1"), GET_SCHEMA_TOOL, json!({"groups": ["weather"]})),
            call(Some("t2"), GET_SCHEMA_TOOL, json!({"groups": ["world"]})),
        ]);
        let mut r = req(&c, &cat);
        r.thinking = Thinking::ReasoningEffortNone;
        let o = crate::openai::build_chat_request("http://localhost:11434/v1", None, &r).unwrap();
        let b: Value = serde_json::from_slice(&o.body).unwrap();
        let m = b["messages"].as_array().unwrap();
        assert_eq!(m[2]["role"], "assistant");
        assert_eq!(m[2]["tool_calls"][0]["id"], "t1");
        assert_eq!(m[2]["tool_calls"][0]["function"]["arguments"], json!(r#"{"groups":["weather"]}"#));
        assert_eq!((m[3]["role"].as_str(), m[3]["tool_call_id"].as_str()), (Some("tool"), Some("t1")));
        assert_eq!((m[4]["role"].as_str(), m[4]["tool_call_id"].as_str()), (Some("tool"), Some("t2")));
        assert_eq!(b["reasoning_effort"], "none");
        assert!(b.get("chat_template_kwargs").is_none());
        r.thinking = Thinking::ChatTemplateNoThinking;
        let b: Value = serde_json::from_slice(&crate::openai::build_chat_request("http://localhost:9931/v1", None, &r).unwrap().body).unwrap();
        assert_eq!(b["chat_template_kwargs"], json!({"enable_thinking": false}));
        assert!(b.get("reasoning_effort").is_none());
        r.thinking = Thinking::Default;
        let b: Value = serde_json::from_slice(&crate::openai::build_chat_request("http://localhost:9931/v1", None, &r).unwrap().body).unwrap();
        assert!(b.get("chat_template_kwargs").is_none() && b.get("reasoning_effort").is_none());

        r.thinking = Thinking::ReasoningEffortNone;
        let key = crate::secret::ApiKey::new("sk-ant-test-0123456789").unwrap();
        let a = crate::anthropic::build_messages_request("https://api.anthropic.com/v1", Some(&key), &r).unwrap();
        let b: Value = serde_json::from_slice(&a.body).unwrap();
        assert!(b["system"].as_str().unwrap().contains("get_param_schema"));
        assert!(b.get("reasoning_effort").is_none(), "thinking is never sent on the Anthropic wire");
        let m = b["messages"].as_array().unwrap();
        assert_eq!(m.len(), 3, "user, assistant(tool_use), user(two tool_results): {m:?}");
        assert_eq!(m[1]["content"][0], json!({"type": "text", "text": "Looking."}));
        assert_eq!(m[1]["content"][1], json!({"type": "tool_use", "id": "t1", "name": GET_SCHEMA_TOOL, "input": {"groups": ["weather"]}}));
        assert_eq!(m[2]["role"], "user");
        let results = m[2]["content"].as_array().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!((results[0]["type"].as_str(), results[0]["tool_use_id"].as_str()), (Some("tool_result"), Some("t1")));
        assert_eq!(results[1]["tool_use_id"], "t2");
        assert_eq!(b["tools"][1]["name"], SET_PARAMS_TOOL);
        assert!(b["tools"][1]["input_schema"]["properties"]["changes"].is_object());
    }

    /// Protects the turn cap as a literal: a model that only ever asks for the
    /// schema is stopped after 5 replies. MUTATION: 5 -> 6.
    #[test]
    fn the_turn_cap_stops_a_looping_model() {
        assert_eq!(MAX_TURNS, 5);
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "x").unwrap();
        for i in 1..MAX_TURNS {
            assert_eq!(c.on_reply(&cat, "", vec![call(None, GET_SCHEMA_TOOL, json!({"groups": ["world"]}))]), Step::Continue, "turn {i}");
        }
        assert!(matches!(c.on_reply(&cat, "", vec![call(None, GET_SCHEMA_TOOL, json!({"groups": ["world"]}))]), Step::Failed(_)));
        assert!(ParamsConversation::new(&cat, "   ").is_err());
        assert!(ParamsConversation::new(&cat, &"a".repeat(MAX_REQUEST_CHARS + 1)).is_err());
    }

    /// Protects the union: schema(climate) then schema(tectonics) keeps both, so
    /// a climate key and a tectonics key are both accepted; a volcanism key never
    /// shown is refused V6. MUTATION: restore `shown_groups = groups` (replace) and
    /// the climate key becomes V6; or make V6 accept every group and the
    /// volcanism key is accepted. Fixture groups are those of `catalogue::tests`.
    #[test]
    fn a_second_schema_call_adds_to_the_offered_keys() {
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "x").unwrap();
        c.on_reply(&cat, "", vec![call(None, GET_SCHEMA_TOOL, json!({"groups": ["weather"]}))]);
        c.on_reply(&cat, "", vec![call(None, GET_SCHEMA_TOOL, json!({"groups": ["tectonics"]}))]);
        assert_eq!(c.shown_groups(), &["weather".to_string(), "tectonics".to_string()], "union in first-shown order");
        let en = req(&c, &cat).tools[1].parameters["properties"]["changes"]["items"]["properties"]["key"]["enum"].as_array().unwrap().len();
        assert_eq!(en, 8, "weather 4 + tectonics 4 in the fixture, not tectonics alone");
        let ok = json!({"changes": [{"key": "climate.ocean", "value": 0.7}, {"key": "tect.alpha", "value": 0.65}]});
        let v = cat.validate(&ok, c.shown_groups());
        assert_eq!(v.valid.len(), 2, "both the first-opened and the later-opened group's key are accepted: {v:?}");
        assert!(v.rejected.is_empty());
        // A group never opened stays refused, whichever other groups are open.
        let bad = json!({"changes": [{"key": "civ.factions", "value": 9}]});
        let v = cat.validate(&bad, c.shown_groups());
        assert_eq!(v.rejected.len(), 1);
        assert_eq!(v.rejected[0].rule, crate::catalogue::Rule::V6);
    }

    /// Protects the budget: a call that would push the union over
    /// [`MAX_SHOWN_KEYS`] opens nothing and the error names the groups still
    /// open; the cap is a literal. MUTATION: 80 -> 81, or drop the check.
    #[test]
    fn the_union_is_capped() {
        assert_eq!(MAX_SHOWN_KEYS, 80);
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "x").unwrap();
        // Force the budget low by pre-filling shown_keys (the fixture has only ~20 rows).
        c.shown_groups = vec!["weather".into()];
        c.shown_keys = (0..MAX_SHOWN_KEYS).map(|i| format!("pad.{i}")).collect();
        c.on_reply(&cat, "", vec![call(None, GET_SCHEMA_TOOL, json!({"groups": ["tectonics"]}))]);
        assert_eq!(c.shown_groups(), &["weather".to_string()], "nothing added");
        assert_eq!(c.shown_keys.len(), MAX_SHOWN_KEYS);
        let m = &c.messages().last().unwrap().content;
        assert!(m.contains("more than 80") && m.contains("weather"), "{m}");
    }
}
