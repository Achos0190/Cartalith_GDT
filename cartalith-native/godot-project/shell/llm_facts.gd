extends RefCounted
class_name LlmFacts

## Reads the world for the assistant's read-only lookups (`LLM_ASSISTANT_SCOPE.md` s.5b.5,
## milestone LLM-8), from the ENGINE'S OWN GETTERS, and nothing else.
##
## Rust (`LlmStream.pending_needs`) says which of three world readings the model's waiting
## lookups need; this file takes exactly those and returns the Dictionary Rust reads as
## `args.facts` (`llm_stream.rs`, `DictFacts`). The split exists because the Rust class must
## never `bind()` a `WorldGen` (it fails while a generation holds the engine on its worker thread;
## `llm_bridge.rs`), so the one place that may touch the engine is the shell, and only after this
## file has checked `generating`.
##
## ## Rules this file keeps
##
## * **Read-only.** It calls `ecology_summary`, `sample_cell`, `world_summary`, `get_width` and
##   `get_height`, all `&self` getters. It writes nothing, regenerates nothing, and opens no
##   socket.
## * **Never reads a generating engine.** While `generating` is true, or when no world is open,
##   it returns `{reason: ...}` and calls no getter: every world lookup then answers "not
##   available" with that reason (the model is never handed a stale or invented number).
## * **Only what was asked.** `needs.samples` is already inside the request's remaining cell
##   allowance (Rust trimmed it); this file reads each listed cell and no neighbour.
## * **No free text from the world.** The three getters' fields that Rust forwards are numbers and
##   the engine's own water and biome words; a faction's name, a note or a place name is never read
##   here (`sample_cell` does return `controlling_faction_name`; `LlmStream`'s `DictFacts` ignores
##   it, and `tests/` of the probe assert it is absent from the model's request).
##
## ABSENT MEANS ABSENT: a getter that returns an empty Dictionary is passed on empty, and Rust
## reports "not available"; it is never replaced by a zero here.


## `needs` is `LlmStream.pending_needs()`: `{ecology: bool, summary: bool, samples: [{x, y}, ...]}`.
## `bridge` is the `EngineBridge` (its `has_world`, `generating` and `world_gen`).
static func read(bridge: Object, needs: Dictionary) -> Dictionary:
	var out := {"generating": bool(bridge.get("generating"))}
	var samples: Array = needs.get("samples", [])
	var wants := bool(needs.get("ecology", false)) or bool(needs.get("summary", false)) or not samples.is_empty()
	if not wants:
		return out
	if not bool(bridge.get("has_world")):
		out["reason"] = "No world is open, so there is nothing to read."
		return out
	if bool(bridge.get("generating")):
		out["reason"] = "A world is generating; world figures are not available until it finishes."
		return out
	var wg: Object = bridge.get("world_gen")
	if wg == null or not wg.has_method("world_summary") or not wg.has_method("ecology_summary") or not wg.has_method("sample_cell"):
		out["reason"] = "This build of the Cartalith library cannot read the world for the assistant."
		return out
	out["grid_width"] = int(wg.call("get_width"))
	out["grid_height"] = int(wg.call("get_height"))
	if bool(needs.get("ecology", false)):
		out["ecology"] = wg.call("ecology_summary")
	if bool(needs.get("summary", false)):
		out["summary"] = wg.call("world_summary")
	if not samples.is_empty():
		var cells := {}
		for s in samples:
			var c: Dictionary = s
			var x := int(c.get("x", -1))
			var y := int(c.get("y", -1))
			## An out-of-grid cell comes back as an empty Dictionary from `sample_cell`; kept
			## empty so Rust can say "outside the map" instead of reading a neighbour.
			cells["%d,%d" % [x, y]] = wg.call("sample_cell", x, y)
		out["samples"] = cells
	return out
