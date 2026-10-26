extends RefCounted
class_name LlmKnowledge

## The shell half of the assistant's knowledge base (`RAG_KNOWLEDGE_BASE_RESEARCH.md` milestones RK-5
## and RK-6; Rulings DM and DN). Rust (`LlmStream`) decides WHICH sources may be used for a request
## (`knowledge_plan`: the two checkboxes, the hosted opt-in, the two modes) and which lookups the model
## asked for (`pending_needs().knowledge`); this file only EXECUTES those against the in-memory index in
## `WorldGen` (`knowledge_call`, `knowledge_search_json`) and hands the raw text back to Rust, which
## rebuilds it (`knowledge_tools::sanitize` / `build_pack`) before any model sees a word of it.
##
## ## Rules this file keeps
##
## * **Read-only.** It calls two `WorldGen` methods that build or refresh an in-memory index and search
##   it. It writes no note, no setting and nothing in the world, and opens no socket.
## * **Only what the plan allows.** Every call passes the plan's `{world, vault}` flags; a flag that is
##   false means that side is neither listed, collected nor held (Rust re-checks, and `sanitize` drops
##   a hit of a source that is off). With both false nothing is called at all.
## * **Never while a world generates**: `WorldGen` cannot be borrowed then (`llm_bridge.rs`), so every
##   lookup answers "not available" with the reason, as `llm_facts.gd` does. ABSENT MEANS ABSENT: the
##   reason travels to the model; it is never an empty hit list that reads as "nothing exists".
## * **No free text of its own.** The strings it builds are fixed reasons.
##
## MUST NEVER: pass a source the plan did not allow, call `knowledge_call` with a call Rust did not
## validate, cache a result across requests, or log a note's text.

const _GENERATING := "A world is generating; the knowledge base is not available until it finishes."
const _NO_BUILD := "This build of the Cartalith library has no knowledge base."


## `LlmStream.knowledge_plan` for the current settings: `{ok, world, vault, pre, tools, note, hosted}`.
## `capability` is `"prose"` or `"commands"`. Never raises: a build without the library answers
## `{ok: false, ...}` with both sources off.
static func plan(stream: Object, capability: String) -> Dictionary:
	if stream == null or not stream.has_method("knowledge_plan"):
		return {"ok": false, "world": false, "vault": false, "pre": false, "tools": false, "note": _NO_BUILD}
	return stream.knowledge_plan(DccSettings.llm_gate_dict(), capability, DccSettings.llm_consented_origins())


## True when the plan allows at least one source.
static func any(p: Dictionary) -> bool:
	return bool(p.get("ok", false)) and (bool(p.get("world", false)) or bool(p.get("vault", false)))


## Why the index cannot be read right now, or "" when it can.
static func unreadable(bridge: Object) -> String:
	if bool(bridge.get("generating")):
		return _GENERATING
	var wg: Object = bridge.get("world_gen")
	if wg == null or not wg.has_method("knowledge_call"):
		return _NO_BUILD
	return ""


## The raw JSON text of `{"available": false, "reason": why}`, the shape Rust reads as "not available".
static func _unavailable(why: String) -> String:
	return JSON.stringify({"available": false, "reason": why})


## The pre-retrieval search for `query` (RK-6): the raw result text for `args.knowledge_pack`, or "" when
## the plan does not ask for it (then the key is simply absent from the request).
static func search_pack(bridge: Object, p: Dictionary, query: String) -> String:
	if not any(p) or not bool(p.get("pre", false)):
		return ""
	var why := unreadable(bridge)
	if why != "":
		return _unavailable(why)
	var wg: Object = bridge.get("world_gen")
	return String(wg.knowledge_search_json(query, {"world": bool(p.get("world", false)), "vault": bool(p.get("vault", false))}))


## Executes the model's waiting knowledge calls (`needs` is `LlmStream.pending_needs()`): `{id: raw JSON
## text}`. An empty Dictionary when nothing was asked. Each call is validated again inside `WorldGen`.
static func run(bridge: Object, p: Dictionary, needs: Dictionary) -> Dictionary:
	var out := {}
	var calls: Array = needs.get("knowledge", [])
	if calls.is_empty():
		return out
	var why := "" if any(p) else "No knowledge source is turned on."
	if why == "":
		why = unreadable(bridge)
	for c in calls:
		var d: Dictionary = c
		var id := String(d.get("id", ""))
		if id == "":
			continue
		if why != "":
			out[id] = _unavailable(why)
			continue
		var wg: Object = bridge.get("world_gen")
		out[id] = String(wg.knowledge_call(String(d.get("call", "")), {"world": bool(p.get("world", false)), "vault": bool(p.get("vault", false))}))
	return out
