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
//! LLM-6d adds a second proposal tool, `propose_view_actions` (`crate::view_actions`), offered from
//! the first turn when the shell's facts offer an action, and the "Choose groups" fallback
//! ([`ParamsConversation::open_groups`]), where the USER performs step 1 and the conversation starts
//! with the groups already opened. A reply may carry either proposal tool or both; each is validated
//! on its own and the one retry is granted only when every verdict present wants it.
//!
//! MUST NEVER: apply anything (this module has no setter to call); offer
//! `set_generation_params` before a schema was shown; let text the model read
//! widen the tool set or the opened groups.

use serde_json::{Value, json};

use crate::catalogue::{Catalogue, GET_SCHEMA_TOOL, SET_PARAMS_TOOL, Verdict};
use crate::openai::{ChatRequest, MaxTokensField, Message, Role, Thinking, ToolCallRecord, ToolSpec, ToolTurn};
use crate::prompt::MAX_REQUEST_CHARS;
use crate::view_actions::{VIEW_TOOL, ViewCatalogue, ViewVerdict};
use crate::query::{
    MAX_QUERY_CALLS_PER_REPLY, MAX_SAMPLES_PER_REQUEST, QueryBudget, QueryCall, WorldFacts, answer as answer_query, is_query_tool, parse as parse_query,
    tool_specs as query_tool_specs,
};

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
    /// LLM-6d: the view proposal was refused and its errors went back for the one retry
    /// (a reply that carried only `propose_view_actions`).
    ViewRetry(ViewVerdict),
    /// LLM-6d: terminal, a reply that carried only `propose_view_actions`: the view card.
    ViewProposal(ViewVerdict),
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
    /// LLM-8: whether the read-only query tools are offered this request (the "generation
    /// parameters and summary statistics" context source, Ruling DI P3). Off means the tools are
    /// not in the request at all, the prompt does not name them, and a call to one is "unknown tool".
    queries: bool,
    /// Query calls the model made whose answers are not yet in the conversation: the world is read
    /// at the moment of the NEXT request ([`ParamsConversation::answer_pending`]), not when the
    /// reply ended, because the world can change (a generate can start) in between.
    pending: Vec<PendingQuery>,
    /// Counters the query caps read.
    budget: QueryBudget,
    /// LLM-6d: the view and layer action catalogue of THIS turn (the shell hands the facts over
    /// every turn, so a layer switched in between is what the validator compares against). `None`
    /// means the tool is not in the request at all.
    view: Option<ViewCatalogue>,
    /// LLM-6d: the view verdict that rode along a terminal or retried [`Step::Proposal`] /
    /// [`Step::Retry`] when one reply carried both tools. Read with [`ParamsConversation::take_view_verdict`].
    view_out: Option<ViewVerdict>,
}

/// What a batch of waiting query calls needs read from the world.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorldNeeds {
    /// `get_ecology_summary` was called.
    pub ecology: bool,
    /// `get_world_summary` was called.
    pub summary: bool,
    /// The `sample_point` cells to read (already inside the request's remaining allowance).
    pub samples: Vec<(i64, i64)>,
}

/// One query call waiting for its answer: its id (to pair the result) and the validated call, or
/// the reason its arguments were refused (told to the model as the result).
#[derive(Debug, Clone)]
struct PendingQuery {
    id: String,
    name: String,
    call: Result<QueryCall, String>,
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
            queries: false,
            pending: Vec::new(),
            budget: QueryBudget::default(),
            view: None,
            view_out: None,
        })
    }

    /// LLM-6d: offer (or stop offering) the view and layer action tool for this request. Call it
    /// right after [`ParamsConversation::new`] with the facts of the first turn: it rewrites the
    /// system prompt so the prompt names the tool exactly when it is offered (the same rule as
    /// [`ParamsConversation::with_queries`]). Pass `None`, or a catalogue with nothing offered,
    /// for no tool and no prompt text.
    pub fn with_view(mut self, cat: &Catalogue, view: Option<ViewCatalogue>) -> Self {
        let on = view.as_ref().is_some_and(ViewCatalogue::is_offered);
        self.view = view;
        if let Some(m) = self.messages.first_mut() {
            m.content = cat.system_prompt_full(self.queries, on);
        }
        self
    }

    /// LLM-6d: replace the view catalogue with this turn's facts (the shell rebuilds them from
    /// the live app before every request). Does not touch the system prompt.
    pub fn set_view(&mut self, view: Option<ViewCatalogue>) {
        self.view = view;
    }

    /// LLM-6d: the view verdict that accompanied the last [`Step::Proposal`] or [`Step::Retry`],
    /// taken (a second call answers `None`).
    pub fn take_view_verdict(&mut self) -> Option<ViewVerdict> {
        self.view_out.take()
    }

    /// LLM-6d, the "Choose groups" fallback (scope s.5b.1): the USER picked the groups, so the
    /// app performs step one itself. Appends exactly what a model's valid `get_param_schema`
    /// call would have produced (an assistant tool call and its result) and opens the groups,
    /// so the first request already offers `set_generation_params` with the key enum of those
    /// groups. The same validation as the model's call applies (1 to 3 real groups), so this
    /// cannot open more than the model could. Refused after any model reply.
    pub fn open_groups(&mut self, cat: &Catalogue, groups: &[String]) -> Result<(), String> {
        if self.turns > 0 || !self.shown_groups.is_empty() {
            return Err("groups can only be chosen before the first model reply".into());
        }
        let arg = json!(groups);
        let (chosen, value, keys) = cat.schema_for(Some(&arg))?;
        let id = format!("call_cartalith_{}", self.next_id);
        self.next_id += 1;
        self.messages.push(Message {
            role: Role::Assistant,
            content: String::new(),
            tool: Some(ToolTurn::Calls(vec![ToolCallRecord { id: id.clone(), name: GET_SCHEMA_TOOL.into(), arguments: json!({"groups": chosen}) }])),
        });
        self.messages.push(Message { role: Role::User, content: value.to_string(), tool: Some(ToolTurn::Result { call_id: id }) });
        self.shown_groups = chosen;
        self.shown_keys = keys;
        Ok(())
    }

    /// Offer (or stop offering) the six read-only query tools of LLM-8 for this request. Call it
    /// right after [`ParamsConversation::new`], before the first request: it rewrites the system
    /// prompt so the prompt names the tools exactly when they are offered.
    pub fn with_queries(mut self, cat: &Catalogue, on: bool) -> Self {
        self.queries = on;
        if let Some(m) = self.messages.first_mut() {
            m.content = cat.system_prompt_for(on);
        }
        self
    }

    /// True when query tools are offered this request.
    pub fn queries_offered(&self) -> bool {
        self.queries
    }

    /// The names of the query calls waiting for an answer, in order (for the panel's "looked up" line).
    pub fn pending_query_names(&self) -> Vec<String> {
        self.pending.iter().map(|p| p.name.clone()).collect()
    }

    /// True when a model reply's query calls still need [`ParamsConversation::answer_pending`]
    /// before the next request may be built (a request without them would have tool calls with no
    /// result, which every provider refuses).
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// What the waiting query calls need read from the WORLD (the three tools that are not answered
    /// from the catalogue). The shell reads exactly these from the engine's own getters, and only
    /// when no generation is running, then hands them to [`ParamsConversation::answer_pending`]:
    /// the engine is never read for a tool the model did not call, and never for more
    /// `sample_point` cells than the request's remaining allowance.
    pub fn pending_needs(&self) -> WorldNeeds {
        let mut n = WorldNeeds::default();
        let mut room = MAX_SAMPLES_PER_REQUEST.saturating_sub(self.budget.samples);
        for p in &self.pending {
            match &p.call {
                Ok(QueryCall::Ecology) => n.ecology = true,
                Ok(QueryCall::WorldSummary) => n.summary = true,
                Ok(QueryCall::Sample { x, y }) if room > 0 => {
                    room -= 1;
                    n.samples.push((*x, *y));
                }
                _ => {}
            }
        }
        n
    }

    /// Answer every waiting query call from `facts` and append the results to the conversation.
    /// Read-only: it reads `cat` and `facts` and writes only this conversation's own messages. It
    /// never touches `shown_groups` or `shown_keys`, so a query cannot widen what a proposal may
    /// name (V6). Idempotent when nothing is pending.
    pub fn answer_pending(&mut self, cat: &Catalogue, facts: &dyn WorldFacts) {
        let pending = std::mem::take(&mut self.pending);
        for p in pending {
            let body = match &p.call {
                Ok(call) => answer_query(call, cat, facts, &mut self.budget),
                Err(why) => json!({"error": why}),
            };
            self.messages.push(Message { role: Role::User, content: body.to_string(), tool: Some(ToolTurn::Result { call_id: p.id }) });
        }
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
        if self.queries {
            t.extend(query_tool_specs(cat));
        }
        if let Some(v) = self.view.as_ref().filter(|v| v.is_offered()) {
            t.push(v.tool_spec());
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

        // A proposal ends the conversation unless the outcome rule grants the retry. LLM-6d: a
        // reply may carry `set_generation_params`, `propose_view_actions` or both. Each is
        // validated on its own; the retry is granted only when EVERY verdict present wants it,
        // so a usable half is never thrown away because the other half was bad. With only
        // `set_generation_params` present this is the LLM-6b rule unchanged.
        let view_on = self.view.as_ref().is_some_and(ViewCatalogue::is_offered);
        let params_rec = records.iter().find(|r| r.name == SET_PARAMS_TOOL);
        let view_rec = if view_on { records.iter().find(|r| r.name == VIEW_TOOL) } else { None };
        if params_rec.is_some() || view_rec.is_some() {
            let pv = params_rec.map(|p| cat.validate(&p.arguments, &self.shown_groups));
            let vv = match (view_rec, self.view.as_ref()) {
                (Some(r), Some(v)) => Some(v.validate(&r.arguments)),
                _ => None,
            };
            let wants = pv.as_ref().is_none_or(Verdict::wants_retry) && vv.as_ref().is_none_or(ViewVerdict::wants_retry);
            let can_retry = wants && self.retries < MAX_PROPOSAL_RETRIES && self.turns < MAX_TURNS;
            if !can_retry {
                return match (pv, vv) {
                    (Some(p), v) => {
                        self.view_out = v;
                        self.finish(Step::Proposal(p))
                    }
                    (None, Some(v)) => self.finish(Step::ViewProposal(v)),
                    (None, None) => self.finish(Step::Failed("The proposal could not be read.".into())),
                };
            }
            self.retries += 1;
            let p_feedback = pv.as_ref().map(|v| v.feedback().to_string());
            let v_feedback = vv.as_ref().map(|v| v.feedback().to_string());
            for r in &records {
                let body = match (r.name.as_str(), &p_feedback, &v_feedback) {
                    (SET_PARAMS_TOOL, Some(f), _) => f.clone(),
                    (VIEW_TOOL, _, Some(f)) => f.clone(),
                    _ => json!({"error": "answer ignored: a proposal was made in the same turn"}).to_string(),
                };
                self.messages.push(Message { role: Role::User, content: body, tool: Some(ToolTurn::Result { call_id: r.id.clone() }) });
            }
            return match (pv, vv) {
                (Some(p), v) => {
                    self.view_out = v;
                    Step::Retry(p)
                }
                (None, Some(v)) => Step::ViewRetry(v),
                (None, None) => Step::Failed("The proposal could not be read.".into()),
            };
        }

        // Otherwise every call is a schema call or a query call (the decoder admits only offered
        // names). Query answers wait for the world (`answer_pending`); everything else is answered now.
        let mut query_calls_this_reply = 0usize;
        for r in &records {
            if self.queries && is_query_tool(&r.name) {
                query_calls_this_reply += 1;
                let call = if query_calls_this_reply > MAX_QUERY_CALLS_PER_REPLY {
                    Err(format!("at most {MAX_QUERY_CALLS_PER_REPLY} lookups are answered per reply"))
                } else {
                    parse_query(&r.name, &r.arguments, cat)
                };
                self.pending.push(PendingQuery { id: r.id.clone(), name: r.name.clone(), call });
                continue;
            }
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
    // ---- LLM-8: the read-only query tools ----

    use crate::query::{GET_CHANGED_TOOL, GET_PARAMETERS_TOOL, NoWorld, QUERY_TOOLS, SAMPLE_POINT_TOOL};

    fn with_q(cat: &Catalogue, req_text: &str) -> ParamsConversation {
        ParamsConversation::new(cat, req_text).unwrap().with_queries(cat, true)
    }

    /// Protects Ruling DI P3 at the request: with the query switch off the six tools are NOT in the
    /// request (turn 1 offers the schema tool alone) and the system prompt does not name them; with
    /// it on they are offered from turn 1 and the prompt names them. MUTATION: offer the tools
    /// regardless of `queries`, or name them in the prompt regardless, and this goes red.
    #[test]
    fn query_tools_are_offered_only_when_the_source_is_ticked() {
        let cat = fixture();
        let off = ParamsConversation::new(&cat, "what is the sea level?").unwrap();
        let names = |c: &ParamsConversation| c.tools(&cat).into_iter().map(|t| t.name).collect::<Vec<_>>();
        assert_eq!(names(&off), vec![GET_SCHEMA_TOOL]);
        assert!(!off.queries_offered());
        for q in QUERY_TOOLS {
            assert!(!off.messages()[0].content.contains(q), "prompt names {q} with the tools off");
        }
        let on = with_q(&cat, "what is the sea level?");
        let mut want = vec![GET_SCHEMA_TOOL.to_string()];
        want.extend(QUERY_TOOLS.iter().map(|s| s.to_string()));
        assert_eq!(names(&on), want);
        for q in QUERY_TOOLS {
            assert!(on.messages()[0].content.contains(q), "prompt omits {q}");
        }
        // Back off again: the prompt is rewritten, not left naming tools.
        let again = on.with_queries(&cat, false);
        assert_eq!(names(&again), vec![GET_SCHEMA_TOOL]);
        assert!(!again.messages()[0].content.contains("get_world_summary"));
    }

    /// Protects the question flow end to end: a query call is recorded, its answer is NOT in the
    /// conversation until `answer_pending`, then it is, paired to the call id, and equals the
    /// catalogue's own value. `has_pending` is the guard a request must check first.
    #[test]
    fn a_query_is_answered_at_the_next_request_from_the_world() {
        let cat = fixture();
        let mut c = with_q(&cat, "what is the peak height?");
        let s = c.on_reply(&cat, "", vec![call(Some("q1"), GET_PARAMETERS_TOOL, json!({"keys": ["peak_m"]}))]);
        assert_eq!(s, Step::Continue);
        assert!(c.has_pending());
        assert_eq!(c.pending_query_names(), vec![GET_PARAMETERS_TOOL.to_string()]);
        let before = c.messages().len();
        assert!(matches!(&c.messages()[before - 1].tool, Some(ToolTurn::Calls(_))), "no result yet");
        c.answer_pending(&cat, &NoWorld("n/a".into()));
        assert!(!c.has_pending());
        let last = c.messages().last().unwrap();
        assert!(matches!(&last.tool, Some(ToolTurn::Result { call_id }) if call_id == "q1"));
        let v: Value = serde_json::from_str(&last.content).unwrap();
        assert_eq!(v["settings"][0]["key"], "peak_m");
        assert_eq!(v["settings"][0]["current"], 4000.0);
        // A world question with no world: "not available", not a made-up number.
        c.on_reply(&cat, "", vec![call(Some("q2"), SAMPLE_POINT_TOOL, json!({"x": 3, "y": 4}))]);
        c.answer_pending(&cat, &NoWorld("No world is open.".into()));
        let v: Value = serde_json::from_str(&c.messages().last().unwrap().content).unwrap();
        assert_eq!(v, json!({"available": false, "reason": "No world is open."}));
    }

    /// Protects V6 against the read tools: reading the `tectonics` group (a query) does not make a
    /// tectonics key proposable; only get_param_schema opens a group. MUTATION: make a query call
    /// extend `shown_keys` and the proposal becomes valid, so this goes red.
    #[test]
    fn reading_a_setting_does_not_open_it_for_proposals() {
        let cat = fixture();
        let mut c = with_q(&cat, "set plates to 20");
        c.on_reply(&cat, "", vec![call(Some("a"), GET_SCHEMA_TOOL, json!({"groups": ["weather"]}))]);
        c.on_reply(&cat, "", vec![call(Some("b"), GET_PARAMETERS_TOOL, json!({"group": "tectonics"}))]);
        c.answer_pending(&cat, &NoWorld("n/a".into()));
        assert_eq!(c.shown_groups(), &["weather".to_string()], "a query opened nothing");
        let r = req(&c, &cat);
        let en: Vec<String> = r.tools.iter().find(|t| t.name == SET_PARAMS_TOOL).unwrap().parameters["properties"]["changes"]["items"]["properties"]["key"]["enum"]
            .as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_owned()).collect();
        assert!(!en.iter().any(|k| k.starts_with("tect.")), "enum widened by a read: {en:?}");
        let Step::Retry(v) = c.on_reply(&cat, "", vec![call(Some("c"), SET_PARAMS_TOOL, json!({"changes": [{"key": "tect.plates", "value": 20}]}))]) else { panic!() };
        assert_eq!(v.rejected[0].rule, crate::catalogue::Rule::V6);
    }

    /// Protects the per-reply cap: of seven query calls in one reply the first six are answered and
    /// the seventh is told why not; every call still gets exactly one result (providers refuse a
    /// call without one). MUTATION: 6 -> 7.
    #[test]
    fn at_most_six_queries_are_answered_per_reply() {
        assert_eq!(MAX_QUERY_CALLS_PER_REPLY, 6);
        let cat = fixture();
        let mut c = with_q(&cat, "x");
        let calls: Vec<IncomingCall> = (0..7).map(|i| call(Some(&format!("q{i}")), GET_CHANGED_TOOL, json!({}))).collect();
        c.on_reply(&cat, "", calls);
        c.answer_pending(&cat, &NoWorld("n/a".into()));
        let results: Vec<&Message> = c.messages().iter().filter(|m| matches!(m.tool, Some(ToolTurn::Result { .. }))).collect();
        assert_eq!(results.len(), 7);
        assert!(results[5].content.contains("changed_count"));
        assert!(results[6].content.contains("at most 6 lookups"), "{}", results[6].content);
    }

    /// Protects the closed set when the switch is off: a model that calls a query tool anyway (a
    /// server ignoring the tool list) gets "unknown tool" and nothing is pending. MUTATION: handle
    /// query calls regardless of `queries`.
    #[test]
    fn a_query_call_with_the_switch_off_is_an_unknown_tool() {
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "x").unwrap();
        c.on_reply(&cat, "", vec![call(Some("q"), GET_CHANGED_TOOL, json!({}))]);
        assert!(!c.has_pending());
        assert!(c.messages().last().unwrap().content.contains("unknown tool"));
    }

    /// Protects the argument errors: a refused argument is a result the model reads, not a panic and
    /// not a guess; the request continues.
    #[test]
    fn a_bad_query_argument_is_told_to_the_model() {
        let cat = fixture();
        let mut c = with_q(&cat, "x");
        assert_eq!(c.on_reply(&cat, "", vec![call(Some("q"), SAMPLE_POINT_TOOL, json!({"x": "3", "y": 4}))]), Step::Continue);
        c.answer_pending(&cat, &NoWorld("n/a".into()));
        let v: Value = serde_json::from_str(&c.messages().last().unwrap().content).unwrap();
        assert!(v["error"].as_str().unwrap().contains("whole number"), "{v}");
    }
    /// Protects what the shell is told to read: `pending_needs` names the world tools asked for and
    /// only as many `sample_point` cells as the request still has room for (8 in all), never a cell
    /// whose arguments were refused. MUTATION: ignore the remaining allowance and the second batch
    /// asks for 6 cells instead of 2.
    #[test]
    fn pending_needs_trims_to_the_remaining_cell_allowance() {
        let cat = fixture();
        let mut c = with_q(&cat, "x");
        let sample = |i: i64| call(Some(&format!("s{i}")), SAMPLE_POINT_TOOL, json!({"x": i, "y": 0}));
        c.on_reply(&cat, "", (1..=6).map(sample).collect());
        let n = c.pending_needs();
        assert_eq!(n.samples.len(), 6);
        assert!(!n.ecology && !n.summary);
        c.answer_pending(&cat, &NoWorld("n/a".into()));
        // Six calls fit one reply: ecology, five cells (the bad one would be a seventh).
        let mut second: Vec<IncomingCall> = vec![call(Some("e"), "get_ecology_summary", json!({}))];
        second.extend((7..=11).map(sample));
        c.on_reply(&cat, "", second);
        let n = c.pending_needs();
        assert_eq!(n.samples, vec![(7, 0), (8, 0)], "6 already read, room for 2");
        assert!(n.ecology && !n.summary);
    }

    fn view_cat() -> ViewCatalogue {
        ViewCatalogue::from_json(&json!({
            "layers": [{"id": "roads", "label": "Roads", "on": true}, {"id": "rivers", "label": "Rivers", "on": false}],
            "windows": [{"id": "vault", "label": "Vault"}]
        }))
        .unwrap()
    }

    fn view_call(id: &str, args: Value) -> IncomingCall {
        call(Some(id), VIEW_TOOL, args)
    }

    /// Protects the closed tool set for LLM-6d: `propose_view_actions` is in the request (and the
    /// prompt names it) exactly when the shell handed over facts that offer an action, from the
    /// FIRST turn (it needs no schema step), and not otherwise. MUTATION: always push the tool, or
    /// always add the prompt paragraph, and the "none" half fails.
    #[test]
    fn the_view_tool_is_offered_only_with_facts() {
        let cat = fixture();
        let names = |c: &ParamsConversation| req(c, &cat).tools.iter().map(|t| t.name.clone()).collect::<Vec<_>>();
        let none = ParamsConversation::new(&cat, "hide the roads").unwrap().with_view(&cat, None);
        assert_eq!(names(&none), vec![GET_SCHEMA_TOOL]);
        assert!(!none.messages()[0].content.contains("propose_view_actions"));
        let empty = ParamsConversation::new(&cat, "x").unwrap().with_view(&cat, Some(ViewCatalogue::from_json(&json!({})).unwrap()));
        assert_eq!(names(&empty), vec![GET_SCHEMA_TOOL]);
        let on = ParamsConversation::new(&cat, "hide the roads").unwrap().with_view(&cat, Some(view_cat()));
        assert_eq!(names(&on), vec![GET_SCHEMA_TOOL, VIEW_TOOL]);
        assert!(on.messages()[0].content.contains("propose_view_actions") && on.messages()[0].content.contains("cannot generate, save, close"));
        // Facts of a later turn replace the earlier ones; an emptied catalogue withdraws the tool.
        let mut c = on;
        c.set_view(None);
        assert_eq!(names(&c), vec![GET_SCHEMA_TOOL]);
    }

    /// Protects the view-only path: one valid call ends the request with the view card, no schema
    /// step needed; a refused one is retried ONCE and the second answer is final. MUTATION:
    /// MAX_PROPOSAL_RETRIES 1 -> 2 and the second block goes red.
    #[test]
    fn a_view_proposal_is_one_step_and_retried_once() {
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "hide the roads").unwrap().with_view(&cat, Some(view_cat()));
        let ok = json!({"actions": [{"action": "set_layer_visible", "args": {"layer": "roads", "visible": false}}]});
        let Step::ViewProposal(v) = c.on_reply(&cat, "", vec![view_call("v1", ok)]) else { panic!() };
        assert_eq!(v.valid.len(), 1);
        assert!(c.is_done() && c.turns() == 1 && c.take_view_verdict().is_none());

        let mut c = ParamsConversation::new(&cat, "x").unwrap().with_view(&cat, Some(view_cat()));
        let bad = json!({"actions": [{"action": "set_layer_visible", "args": {"layer": "road", "visible": false}}]});
        let Step::ViewRetry(v) = c.on_reply(&cat, "", vec![view_call("v1", bad.clone())]) else { panic!() };
        assert_eq!(v.rejected[0].rule.id(), "A8");
        // The errors went back as the tool result, in the call's own id.
        let last = c.messages().last().unwrap();
        assert!(matches!(&last.tool, Some(ToolTurn::Result { call_id }) if call_id == "v1"));
        assert!(last.content.contains("nearest: roads"), "{}", last.content);
        assert!(matches!(c.on_reply(&cat, "", vec![view_call("v2", bad)]), Step::ViewProposal(_)));
        assert_eq!(c.turns(), 2);
    }

    /// Protects the mixed reply: `set_generation_params` and `propose_view_actions` in one reply
    /// are validated separately and both reach the shell; a bad half does not discard a good half
    /// (no retry unless EVERY verdict wants one). MUTATION: retry when either wants it, and the
    /// second block returns Retry instead of Proposal.
    #[test]
    fn one_reply_may_carry_both_proposals() {
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "drier and hide roads").unwrap().with_view(&cat, Some(view_cat()));
        c.on_reply(&cat, "", vec![call(Some("s"), GET_SCHEMA_TOOL, json!({"groups": ["weather", "world", "tectonics"]}))]);
        let params: Value = serde_json::from_str(WORKED).unwrap();
        let view = json!({"actions": [{"action": "open_window", "args": {"window": "vault"}}]});
        let Step::Proposal(p) = c.on_reply(&cat, "", vec![call(Some("p"), SET_PARAMS_TOOL, params), view_call("v", view)]) else { panic!() };
        assert_eq!(p.valid.len(), 5);
        let vv = c.take_view_verdict().expect("the view half rode along");
        assert_eq!(vv.valid.len(), 1);
        assert!(c.take_view_verdict().is_none(), "taken once");

        let mut c = ParamsConversation::new(&cat, "x").unwrap().with_view(&cat, Some(view_cat()));
        c.on_reply(&cat, "", vec![call(Some("s"), GET_SCHEMA_TOOL, json!({"groups": ["weather"]}))]);
        let bad_params = json!({"changes": [{"key": "peak_m", "value": 9000}]}); // peak_m is in `world`, not opened: V6
        let view = json!({"actions": [{"action": "open_window", "args": {"window": "vault"}}]});
        let Step::Proposal(p) = c.on_reply(&cat, "", vec![call(Some("p"), SET_PARAMS_TOOL, bad_params), view_call("v", view)]) else {
            panic!("retried a usable half")
        };
        assert!(p.valid.is_empty() && !p.rejected.is_empty());
        assert_eq!(c.take_view_verdict().unwrap().valid.len(), 1);
    }

    /// Protects the closed tool set the other way: with no facts, OR facts that offer nothing, a
    /// call to `propose_view_actions` is an unknown tool (answered with an error, never validated,
    /// never a card). MUTATION: admit the call whenever a catalogue exists (`self.view.is_some()`)
    /// and the empty-facts half goes red (it would end the request with a refusal card).
    #[test]
    fn a_view_call_without_facts_is_an_unknown_tool() {
        let cat = fixture();
        let args = json!({"actions": [{"action": "open_window", "args": {"window": "vault"}}]});
        let mut c = ParamsConversation::new(&cat, "x").unwrap();
        assert_eq!(c.on_reply(&cat, "", vec![view_call("v", args.clone())]), Step::Continue);
        assert!(c.messages().last().unwrap().content.contains("unknown tool propose_view_actions"));
        let mut e = ParamsConversation::new(&cat, "x").unwrap().with_view(&cat, Some(ViewCatalogue::from_json(&json!({})).unwrap()));
        assert_eq!(e.on_reply(&cat, "", vec![view_call("v", args)]), Step::Continue, "a catalogue that offers nothing is no catalogue");
        assert!(e.messages().last().unwrap().content.contains("unknown tool propose_view_actions"));
        assert!(!e.is_done());
    }

    /// Protects the "Choose groups" fallback: the user's groups are opened by the app, so the FIRST
    /// request already offers the proposal tool with the key enum of exactly those groups, the
    /// replayed history holds a synthetic schema call and its result on both wires, and the same
    /// limits as a model's call apply (1 to 3 real groups; nothing after a model reply).
    /// MUTATION: skip `shown_keys` and the enum is empty/absent.
    #[test]
    fn choose_groups_performs_step_one_for_the_user() {
        let cat = fixture();
        let mut c = ParamsConversation::new(&cat, "drier please").unwrap();
        c.open_groups(&cat, &["weather".to_owned(), "tectonics".to_owned()]).unwrap();
        let r = req(&c, &cat);
        let names: Vec<_> = r.tools.iter().map(|t| t.name.clone()).collect();
        assert_eq!(names, vec![GET_SCHEMA_TOOL, SET_PARAMS_TOOL]);
        let en = r.tools[1].parameters["properties"]["changes"]["items"]["properties"]["key"]["enum"].as_array().unwrap();
        assert_eq!(en.len(), 8, "weather 4 + tectonics 4 in the fixture, world not opened");
        assert!(!en.iter().any(|k| k == "peak_m"));
        assert_eq!(c.shown_groups(), &["weather".to_owned(), "tectonics".to_owned()]);
        assert_eq!(c.turns(), 0, "no model reply was consumed");
        // OpenAI wire: assistant tool_calls turn then a role:tool result.
        let o = crate::openai::build_chat_request("http://localhost:11434/v1", None, &r).unwrap();
        let b: Value = serde_json::from_slice(&o.body).unwrap();
        let m = b["messages"].as_array().unwrap();
        assert_eq!(m.len(), 4);
        assert_eq!(m[2]["tool_calls"][0]["function"]["name"], GET_SCHEMA_TOOL);
        assert_eq!(m[3]["role"], "tool");
        assert_eq!(m[3]["tool_call_id"], m[2]["tool_calls"][0]["id"]);
        // Anthropic wire: tool_use then tool_result.
        let key = crate::secret::ApiKey::new("sk-ant-test-0123456789").unwrap();
        let a = crate::anthropic::build_messages_request("https://api.anthropic.com/v1", Some(&key), &r).unwrap();
        let b: Value = serde_json::from_slice(&a.body).unwrap();
        let m = b["messages"].as_array().unwrap();
        assert_eq!(m.len(), 3);
        assert_eq!(m[1]["content"][0]["type"], "tool_use", "an empty assistant text is not sent as a text block: {m:?}");
        assert_eq!(m[2]["content"][0]["type"], "tool_result");
        // A proposal over a chosen group validates (V6 satisfied).
        let ok = json!({"changes": [{"key": "climate.ocean", "value": 0.7}]});
        let Step::Proposal(v) = c.on_reply(&cat, "", vec![call(Some("p"), SET_PARAMS_TOOL, ok)]) else { panic!() };
        assert_eq!(v.valid.len(), 1);

        // The limits.
        let mut c = ParamsConversation::new(&cat, "x").unwrap();
        for bad in [vec![], vec!["nope".to_owned()], vec!["weather".into(), "world".into(), "tectonics".into(), "civ".into()]] {
            assert!(c.open_groups(&cat, &bad).is_err(), "{bad:?}");
            assert!(c.shown_groups().is_empty() && c.messages().len() == 2, "a refused list opened nothing");
        }
        c.open_groups(&cat, &["weather".to_owned()]).unwrap();
        assert!(c.open_groups(&cat, &["world".to_owned()]).is_err(), "a second choice is refused");
        let mut late = ParamsConversation::new(&cat, "x").unwrap();
        late.on_reply(&cat, "", vec![call(Some("s"), GET_SCHEMA_TOOL, json!({"groups": ["weather"]}))]);
        assert!(late.open_groups(&cat, &["world".to_owned()]).is_err(), "not after a model reply");
    }
}
