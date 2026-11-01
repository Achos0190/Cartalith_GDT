"""Today's assistant requests, rebuilt from the Rust source (cartalith-llm, cartalith-godot), for AC-0's counts.

Adapted from the read-only analysis of 2026-10-31 (its measure.py): it decodes the Rust string literals of the system
prompts and tool descriptions and rebuilds the tool JSON as it goes on the wire (compact serde_json). The value lists
the app fills at run time (landmark kinds, layers, presets) are stand-ins of realistic size, labelled where they are
built. Read-only over the repository.
"""
import json, re, sys

import os
ROOT = os.path.join(os.environ.get("CARTALITH_REPO", os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../../.."))), "cartalith-native/crates")
LLM = ROOT + "/cartalith-llm/src/"
GOD = ROOT + "/cartalith-godot/src/"


def src(path):
    return open(path, encoding="utf8").read()


def rust_lit(text, anchor, nth=0):
    """Decode the first Rust string literal after `anchor` (the nth after it)."""
    i = text.index(anchor)
    for _ in range(nth + 1):
        i = text.index('"', i)
        out, j = [], i + 1
        while True:
            c = text[j]
            if c == '\\':
                n = text[j + 1]
                if n == '\n':
                    j += 2
                    while text[j] in ' \t\n\r':
                        j += 1
                    continue
                out.append({'n': '\n', 't': '\t', '"': '"', '\\': '\\', "'": "'", '0': '\0'}.get(n, n))
                j += 2
                continue
            if c == '"':
                break
            out.append(c)
            j += 1
        lit = "".join(out)
        i = j + 1
    return lit


def fmt(s, named=None, pos=()):
    named = named or {}
    pos = list(pos)
    out, i = [], 0
    while i < len(s):
        if s.startswith("{{", i):
            out.append("{"); i += 2; continue
        if s.startswith("}}", i):
            out.append("}"); i += 2; continue
        if s[i] == "{":
            k = s.index("}", i)
            name = s[i + 1:k]
            out.append(str(named[name]) if name else str(pos.pop(0)))
            i = k + 1
            continue
        out.append(s[i]); i += 1
    return "".join(out)


def tok(n):
    return round(n / 4)


def wire_openai(tools):
    return json.dumps([{"type": "function", "function": {"name": t["name"], "description": t["description"], "parameters": t["parameters"]}} for t in tools], separators=(",", ":"), ensure_ascii=False)


def wire_anthropic(tools):
    return json.dumps([{"name": t["name"], "description": t["description"], "input_schema": t["parameters"]} for t in tools], separators=(",", ":"), ensure_ascii=False)


cat = src(LLM + "catalogue.rs")
qry = src(LLM + "query.rs")
kno = src(LLM + "knowledge_tools.rs")
vw = src(LLM + "view_actions.rs")
ed = src(LLM + "edit_actions.rs")
pr = src(LLM + "prompt.rs")
pc = src(GOD + "param_catalogue.rs")
pa = src(GOD + "params.rs")

# ---- groups and blurbs (param_catalogue.rs BLURBS) -------------------------------------
blurbs = re.findall(r'\("([a-z_]+)",\s*"([^"]*)"\)', pc[pc.index("pub const BLURBS"):pc.index("];", pc.index("pub const BLURBS"))])
groups = [g for g, _ in blurbs]
excluded = re.findall(r'"([a-z_]+)"', pc[pc.index("pub const EXCLUDED"):pc.index(";", pc.index("pub const EXCLUDED"))])
descs = dict(re.findall(r'\(\s*"([a-z_.0-9]+)",\s*"((?:[^"\\]|\\.)*)"\s*\)', pc[pc.index("pub const DESCS"):pc.index("\n];", pc.index("pub const DESCS"))]))

# ---- registry rows (params.rs PARAMS) -------------------------------------------------
rows = []
for m in re.finditer(r'ParamSpec \{\s*key: "([^"]+)", group: "([^"]+)", kind: Kind::(\w+), min: ([-0-9.e_]+), max: ([-0-9.e_]+), step: ([-0-9.e_]+),\s*label: "((?:[^"\\]|\\.)*)", unit: "((?:[^"\\]|\\.)*)"', pa):
    k, g, kind, mn, mx, st, label, unit = m.groups()
    if k in excluded:
        continue
    rows.append(dict(key=k, group=g, kind=kind.lower(), min=float(mn.replace("_", "")), max=float(mx.replace("_", "")), step=float(st.replace("_", "")), label=label, unit=unit, desc=descs.get(k, "")))


def entry(r):
    e = {"key": r["key"], "type": {"float": "number", "int": "integer", "bool": "boolean"}.get(r["kind"], r["kind"]), "label": r["label"]}
    if r["unit"]:
        e["unit"] = r["unit"]
    if r["kind"] != "bool":
        num = (lambda x: int(x)) if r["kind"] == "int" else (lambda x: x)
        e["min"] = num(r["min"]); e["max"] = num(r["max"]); e["step"] = num(max(r["step"], 1.0) if r["kind"] == "int" else r["step"])
        e["current"] = num(r["min"]); e["default"] = num(r["min"])  # stand-ins of realistic width
    else:
        e["current"] = False; e["default"] = False
    e["desc"] = r["desc"]
    return e


def schema(gs):
    shown = [r for r in rows if r["group"] in gs]
    return json.dumps({"groups": gs, "settings": [entry(r) for r in shown]}, separators=(",", ":"), ensure_ascii=False), [r["key"] for r in shown]


# ---- system prompt (Catalogue::system_prompt_full) ------------------------------------
q_par = rust_lit(cat, 'let q = if queries {')
main = rust_lit(cat, 'format!(\n            "You help a user adjust')
view_par = rust_lit(vw, "pub fn prompt_paragraph()")
table = "\n".join("  %-15s %s" % (g, b) for g, b in blurbs)
consts = {"MAX_PARAM_CHANGES": 12, "MAX_SCHEMA_GROUPS": 3, "MAX_QUERY_KEYS": 40, "MAX_CHANGED_LISTED": 40,
          "MAX_SAMPLES_PER_REQUEST": 8, "MAX_VIEW_ACTIONS": 8, "DEFAULT_LIMIT": 5, "MAX_LIMIT": 8, "MAX_READ_CHARS": 6000,
          "MAX_RELATED_LIMIT": 12, "MAX_EDIT_OPS": 3, "MAX_AMOUNT_M": 500.0, "MAX_SMOOTH_PASSES": 8, "DEFAULT_SOFTNESS": 0.5,
          "MAX_TOTAL_AMOUNT_M": 600.0, "q": None, "v": None}


def sys_prompt(queries, views):
    c = dict(consts); c["q"] = q_par if queries else ""; c["v"] = view_par if views else ""
    return fmt(main, c, [table])


SOURCES = rust_lit(kno, "pub const SOURCES_PARAGRAPH")
TOOLS_P = rust_lit(kno, "pub const TOOLS_PARAGRAPH")
KONLY_HEAD = rust_lit(kno, "pub fn knowledge_only_prompt")

# ---- tools --------------------------------------------------------------------------
t_schema = {"name": "get_param_schema", "description": rust_lit(cat, "pub fn tool_get_schema"),
            "parameters": {"type": "object", "properties": {"groups": {"type": "array", "items": {"type": "string", "enum": groups}, "minItems": 1, "maxItems": 3}}, "required": ["groups"]}}


def t_set(keys):
    return {"name": "set_generation_params", "description": fmt(rust_lit(cat, "pub fn tool_set_params"), consts),
            "parameters": {"type": "object", "properties": {"summary": {"type": "string"}, "changes": {"type": "array", "maxItems": 12, "items": {"type": "object", "properties": {"key": {"type": "string", "enum": sorted(keys)}, "value": {"type": ["number", "boolean"]}, "reason": {"type": "string"}}, "required": ["key", "value"]}}}, "required": ["changes"]}}


no_args = {"type": "object", "properties": {}}
q_descs = [rust_lit(qry, 'name: GET_PARAMETERS_TOOL'), rust_lit(qry, 'name: GET_CHANGED_TOOL'), rust_lit(qry, 'name: DESCRIBE_TOOL'),
           rust_lit(qry, 'name: GET_ECOLOGY_TOOL'), rust_lit(qry, 'name: SAMPLE_POINT_TOOL'), rust_lit(qry, 'name: GET_WORLD_SUMMARY_TOOL')]
q_tools = [
    {"name": "get_parameters", "description": fmt(q_descs[0], consts), "parameters": {"type": "object", "properties": {"group": {"type": "string", "enum": groups}, "keys": {"type": "array", "items": {"type": "string"}, "maxItems": 40}}}},
    {"name": "get_changed_parameters", "description": fmt(q_descs[1], consts), "parameters": no_args},
    {"name": "describe_parameter", "description": q_descs[2], "parameters": {"type": "object", "properties": {"key": {"type": "string"}}, "required": ["key"]}},
    {"name": "get_ecology_summary", "description": q_descs[3], "parameters": no_args},
    {"name": "sample_point", "description": fmt(q_descs[4], consts), "parameters": {"type": "object", "properties": {"x": {"type": "integer", "minimum": 0}, "y": {"type": "integer", "minimum": 0}}, "required": ["x", "y"]}},
    {"name": "get_world_summary", "description": q_descs[5], "parameters": no_args},
]
relations = re.findall(r'"([a-z_]+)"', kno[kno.index("pub const RELATIONS"):kno.index("];", kno.index("pub const RELATIONS"))])
what_both = "the world's own data (WORLD DATA) and the user's Markdown Vault notes (VAULT NOTE)"
k_descs = [rust_lit(kno, 'name: SEARCH_KNOWLEDGE_TOOL'), rust_lit(kno, 'name: READ_SOURCE_TOOL'), rust_lit(kno, 'name: RELATED_SOURCES_TOOL')]
k_tools = [
    {"name": "search_knowledge", "description": fmt(k_descs[0], dict(consts, what=what_both)), "parameters": {"type": "object", "properties": {"query": {"type": "string", "minLength": 1, "maxLength": 200}, "scope": {"type": "string", "enum": ["all", "world", "vault"]}, "limit": {"type": "integer", "minimum": 1, "maximum": 8}}, "required": ["query"]}},
    {"name": "read_source", "description": fmt(k_descs[1], consts), "parameters": {"type": "object", "properties": {"id": {"type": "string", "minLength": 3, "maxLength": 300}}, "required": ["id"]}},
    {"name": "related_sources", "description": fmt(k_descs[2], consts), "parameters": {"type": "object", "properties": {"id": {"type": "string", "minLength": 3, "maxLength": 300}, "relation": {"type": "string", "enum": relations}, "limit": {"type": "integer", "minimum": 1, "maximum": 12}}, "required": ["id"]}},
]

# view tool with realistic facts: the 10 WINDOWS of llm_view.gd, 10 layers, 16 presets, N landmark kinds
def view_tool(n_kinds):
    layers = "|".join("%s (now shown)" % x for x in ["terrain", "rivers", "lakes", "roads", "settlements", "borders", "labels", "landmarks", "grid", "contours"])
    presets = "|".join(["parchment_atlas (now the style)"] + ["preset_%02d (now not the style)" % i for i in range(15)])
    kinds = "|".join("kind_%02d (now shown)" % i for i in range(n_kinds))
    windows = "asset_library|factions|world_data|vault|timeline|journey_planner|journeys_list|travel_library|conflicts|shortcuts"
    lines = ["set_layer_visible {layer: %s, visible: true|false}" % layers, "set_style_preset {preset: %s}" % presets,
             "zoom_to {x: 0..2047, y: 0..1023 (grid cells), zoom: optional 0.1..64}", "set_landmark_filter {kind: %s, visible: true|false}" % kinds,
             "open_window {window: %s}" % windows]
    d = fmt(rust_lit(vw, "description: format!(\n                \"Propose changes to how the map"), consts, ["; ".join(lines)])
    return {"name": "propose_view_actions", "description": d, "parameters": {"type": "object", "properties": {"summary": {"type": "string"}, "actions": {"type": "array", "maxItems": 8, "items": {"type": "object", "properties": {"action": {"type": "string", "enum": ["set_layer_visible", "set_style_preset", "zoom_to", "set_landmark_filter", "open_window"]}, "args": {"type": "object"}, "reason": {"type": "string"}}, "required": ["action", "args"]}}}, "required": ["actions"]}}


edit_desc = fmt(rust_lit(ed, "description: format!(\n                \"Propose edits to the ground"), dict(consts, id="region_0123456789abcdef"), [0.05, 1.0, "smooth|linear|sharp|constant"])
t_edit = {"name": "propose_edit_actions", "description": edit_desc, "parameters": {"type": "object", "properties": {"summary": {"type": "string"}, "edits": {"type": "array", "maxItems": 3, "items": {"type": "object", "properties": {"operation": {"type": "string", "enum": ["raise", "lower", "smooth"]}, "region_id": {"type": "string", "enum": ["region_0123456789abcdef"]}, "amount_m": {"type": "number", "minimum": 0, "maximum": 500.0}, "strength": {"type": "number", "minimum": 0.05, "maximum": 1.0}, "passes": {"type": "integer", "minimum": 1, "maximum": 8}, "falloff": {"type": "string", "enum": ["smooth", "linear", "sharp", "constant"]}, "softness": {"type": "number", "minimum": 0, "maximum": 1}, "reason": {"type": "string"}}, "required": ["operation", "region_id"]}}}, "required": ["edits"]}}
edit_sys = fmt(rust_lit(ed, "pub fn system_prompt(&self) -> String {"), consts, ["<REGION block>"])


def shapes():
    """The requests of today's scoped buttons: name -> (system prompt, tools, extra text in the request).

    The Commands rows are the first request of that scope; "after 3 groups" adds the get_param_schema result for the
    three largest groups and the set_generation_params tool those keys unlock (the scope's worst first-two-turn case)."""
    by = {g: sum(1 for r in rows if r["group"] == g) for g in groups}
    big = sorted(groups, key=lambda g: -by[g])[:3]
    sj, keys = schema(big)
    base_tools = [t_schema] + q_tools + [view_tool(20)]
    return {
        "today: Commands, first turn": (sys_prompt(True, True), base_tools, ""),
        "today: Commands + knowledge, first turn": (sys_prompt(True, True) + "\n\n" + SOURCES + "\n" + TOOLS_P, base_tools + k_tools, ""),
        "today: Commands, after opening the 3 largest groups": (sys_prompt(True, True), base_tools + [t_set(keys)], sj),
        "today: Questions only (knowledge)": (KONLY_HEAD + SOURCES + "\n\n" + TOOLS_P, k_tools, ""),
        "today: Edit (region block not counted)": (edit_sys, [t_edit], ""),
    }


def knowledge_tools():
    return k_tools, SOURCES + "\n" + TOOLS_P
