"""AC-0 measurement fixture: a prototype action catalogue and the four tools of ASSISTANT_ACTIONS_PROPOSAL.md s.3.2.

What it is: a stdlib-only stand-in, built by regex from the shipped sources, so AC-0 can measure token sizes and
selection before AC-1 builds the real generator. It is NOT AC-1's generator and must never be imported by the app.

Sources read (read-only):
  - cartalith-godot/src/params.rs (PARAMS rows) and param_catalogue.rs (EXCLUDED, BLURBS, DESCS): generation settings;
  - godot-project/shell/menus.gd: `_live(<popup>, "<label>", ID_*, <accel>)` rows, check/radio items and the tooltip
    set right after each row, grouped by the builder function they sit in (_edit, _assets, _data, _story, ...);
  - godot-project/shell/llm_view.gd WINDOWS: the curated windows;
  - global_tools.gd and workspaces/*.gd: context-card actions with a literal label;
  - every `arm_tool("<id>")` literal: the map tools;
  - PC_SHELL_PROPOSAL.md s.3.1/s.3.2: selection and year actions (proposed, not shipped: marked `proposed`).
Never: invents a description (an entry with none keeps `desc` empty, and the report counts them); offers a File-menu
row (Ruling DW AQ1 leaves file actions out); offers a bridge #[func] (not user actions).
"""
import json, math, os, re

# ---------------------------------------------------------------- sources
def _read(repo, rel):
    return open(os.path.join(repo, rel), encoding="utf8").read()

# menus.gd builder function -> catalogue category (the desktop menu it fills). File is excluded (AQ1).
BUILDERS = {"_edit": "edit", "_assets": "assets", "_data": "records", "_story": "story",
            "_preferences": "preferences", "_window": "window", "_help": "help", "_file": None,
            # helpers each top-level builder calls, read at menus.gd's call sites (2026-10-31)
            "_build_file_export": None, "_build_undo_redo": "edit", "_build_edit_actions": "edit", "_build_reset_generation": "edit",
            "_build_asset_library_row": "assets", "_build_vault_rows": "records", "_build_help_shortcuts_row": "help",
            "_build_gpu_retry_row": "preferences", "_build_lod_debug_submenu": "preferences", "_build_tiled_lod_menu": "preferences",
            "_on_quality": "preferences", "_build_theme_submenu": "preferences", "_build_handedness_submenu": "preferences",
            "_world": "world/pipeline", "_overflow": None}
KIND_BY_CAT = {"edit": "select", "assets": "window", "records": "window", "story": "window",
               "preferences": "setting", "window": "view", "help": "window", "world/pipeline": "edit"}
# the layers set_layer_visible takes come from CartographyWorkspace.LIVE_LAYERS (read below); the style presets from
# RenderWorkspace.STYLE_PRESETS (read below): both as shipped on 2026-10-31.
TOOL_HOME = {"sculpt": "world", "paint": "world", "region": "world", "measure": "map", "inspect": "map",
             "settlement": "civil", "way": "civil", "route": "civil", "territory": "civil", "journey": "civil",
             "conflict": "story", "label": "carto", "icon": "carto"}
PARAM_GROUP_HOME = {"civ": "civil/generation"}
FILE_ID = re.compile(r"ID_(SAVE|SAVE_AS|OPEN_PROJECT|NEW_WORLD|NEW_WORLD_FROM_SELECTION|CLOSE_PROJECT|REVERT|EXPORT[A-Z_]*|RECENT[A-Z_]*)\b")  # every other parameter group lives under world/generation/<group>


def _unesc(s):
    return s.replace('\\"', '"').replace("\\\\", "\\")


def _params(repo):
    pa = _read(repo, "cartalith-native/crates/cartalith-godot/src/params.rs")
    pc = _read(repo, "cartalith-native/crates/cartalith-godot/src/param_catalogue.rs")
    excluded = set(re.findall(r'"([a-z_]+)"', pc[pc.index("pub const EXCLUDED"):pc.index(";", pc.index("pub const EXCLUDED"))]))
    descs = dict(re.findall(r'\(\s*"([a-z_.0-9]+)",\s*"((?:[^"\\]|\\.)*)"\s*\)', pc[pc.index("pub const DESCS"):pc.index("\n];", pc.index("pub const DESCS"))]))
    out = []
    for m in re.finditer(r'ParamSpec \{\s*key: "([^"]+)", group: "([^"]+)", kind: Kind::(\w+), min: ([-0-9.e_]+), max: ([-0-9.e_]+), step: ([-0-9.e_]+),\s*label: "((?:[^"\\]|\\.)*)", unit: "((?:[^"\\]|\\.)*)"', pa):
        k, g, kind, mn, mx, st, label, unit = m.groups()
        if k in excluded:
            continue
        typ = {"Float": "number", "Int": "integer", "Bool": "boolean"}[kind]
        arg = {"type": typ}
        if typ != "boolean":
            arg.update(minimum=float(mn.replace("_", "")), maximum=float(mx.replace("_", "")))
        home = PARAM_GROUP_HOME.get(g, "world/generation/" + g)
        out.append(dict(id="param:" + k, title=_unesc(label) + (" (" + _unesc(unit) + ")" if unit else ""), desc=_unesc(descs.get(k, ""))[:160],
                        path=home, kind="setting", effect="regenerates", args={"value": arg}, source="params.rs"))
    return out


def _menus(repo):
    src = _read(repo, "cartalith-native/godot-project/shell/menus.gd").split("\n")
    func_at, cur = [], None
    for i, ln in enumerate(src):
        m = re.match(r"func (_[a-z_]+)\(", ln)
        if m:
            cur = m.group(1)
        func_at.append(cur)
    out, seen = [], set()
    row_re = re.compile(r'_live\(\s*[a-z_]+\s*,\s*"((?:[^"\\]|\\.)*)"\s*,\s*(ID_[A-Z0-9_]+)\s*(?:,\s*([^)]*))?\)')
    chk_re = re.compile(r'add_(radio_)?check_item\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*(ID_[A-Z0-9_]+)')
    tip_re = re.compile(r'set_item_tooltip\([^,]+,\s*"((?:[^"\\]|\\.)*)"')
    for i, ln in enumerate(src):
        cat = BUILDERS.get(func_at[i] or "", "other")
        if cat is None:          # the File menu: AQ1 leaves file actions out
            continue
        if FILE_ID.search(ln):   # and a file action defined in another builder (the tablet's World menu has Save)
            continue
        for m in list(row_re.finditer(ln)) + list(chk_re.finditer(ln)):
            if m.re is row_re:
                label, ident, accel, kind = m.group(1), m.group(2), (m.group(3) or "").strip(), KIND_BY_CAT.get(cat, "window")
            else:
                label, ident, accel, kind = m.group(2), m.group(3), "", ("view" if cat == "window" else "setting")
            if ident in seen:
                continue
            seen.add(ident)
            tip = ""
            for j in range(i + 1, min(i + 4, len(src))):
                t = tip_re.search(src[j])
                if t:
                    tip = _unesc(t.group(1)); break
            out.append(dict(id="menu:" + ident[3:].lower(), title=_unesc(label), desc=tip[:160], path=cat if cat != "other" else "other/" + (func_at[i] or "?").strip("_"),
                            kind=kind, effect="opens-window" if label.endswith("…") else ("view" if kind == "view" else "undoable"),
                            accel=re.sub(r"KEY_MASK_|KEY_", "", accel).replace(" | ", "+") if accel else "", args={}, source="menus.gd"))
    return out


def _windows(repo):
    v = _read(repo, "cartalith-native/godot-project/shell/llm_view.gd")
    ids = re.findall(r'\{"id": "([a-z_]+)", "menu":', v[v.index("const WINDOWS"):v.index("]", v.index("const WINDOWS"))])
    return [dict(id="window." + w, title="Open " + w.replace("_", " "), desc="", path="window/open", kind="window", effect="opens-window", args={}, source="llm_view.gd")
            for w in ids]


def _ctx_actions(repo):
    out, seen = [], set()
    base = os.path.join(repo, "cartalith-native/godot-project/shell")
    files = ["global_tools.gd"] + ["workspaces/" + f for f in sorted(os.listdir(os.path.join(base, "workspaces"))) if f.endswith(".gd")]
    for f in files:
        for m in re.finditer(r'"id": *"((global|civ|carto|world)\.[a-z_.]+)", *"label": *"((?:[^"\\]|\\.){1,80})"', _read(repo, "cartalith-native/godot-project/shell/" + f)):
            aid, dom, label = m.group(1), m.group(2), _unesc(m.group(3))
            if aid in seen or "%" in label:
                continue
            seen.add(aid)
            home = {"global": "map/here", "civ": "civil/here", "carto": "carto/here", "world": "world/here"}[dom]
            out.append(dict(id=aid, title=label, desc="", path=home, kind="edit" if dom != "global" else "view", effect="undoable",
                            args={"at": {"type": "object", "description": "a map point {x, y} in grid cells, or the selection"}}, source=f))
    return out


def _tools(repo):
    base = os.path.join(repo, "cartalith-native/godot-project/shell")
    ids = set()
    for root, _, fs in os.walk(base):
        for f in fs:
            if f.endswith(".gd"):
                ids.update(re.findall(r'arm_tool\("([a-z_]+)"', open(os.path.join(root, f), encoding="utf8").read()))
    return [dict(id="tool." + t, title=t.replace("_", " ").capitalize() + " tool", desc="", path=TOOL_HOME.get(t, "map") + "/tools", kind="edit",
                 effect="undoable", args={}, source="arm_tool literals") for t in sorted(ids)]


def _view_and_context(repo):
    cw = _read(repo, "cartalith-native/godot-project/shell/workspaces/cartography_workspace.gd")
    seg = cw[cw.index("const LIVE_LAYERS"):]
    seg = seg[:seg.index("\n]")]
    layers = re.findall(r'"id": *"([a-z_]+)", *"label": *"([^"]+)"', seg)
    rw = _read(repo, "cartalith-native/godot-project/shell/workspaces/render_workspace.gd")
    seg = rw[rw.index("const STYLE_PRESETS := ["):]
    presets = re.findall(r'^\s*\["([^"]+)"', seg[:seg.index("\n]")], re.M)
    out = [dict(id="view.layer." + i, title=label + " layer", desc="Show or hide the " + label + " layer on the map.", path="carto/layers",
                kind="view", effect="view", args={"visible": {"type": "boolean"}}, source="LIVE_LAYERS") for i, label in layers]
    out.append(dict(id="view.style_preset", title="Style preset", desc=("Switch the map's whole look. Presets: " + ", ".join(presets))[:400], path="carto/style",
                    kind="view", effect="undoable", args={"preset": {"type": "string", "enum": presets}}, source="STYLE_PRESETS"))
    out.append(dict(id="view.zoom_to", title="Centre and zoom the map", desc="Move the view to a grid cell, optionally zooming.", path="map/view",
                    kind="view", effect="view", args={"x": {"type": "integer"}, "y": {"type": "integer"}, "zoom": {"type": "number"}}, source="llm_view.gd verb"))
    proposed = [
        ("select", "Select a place, faction, landmark, conflict or note", "Make it the one selection: the map, Inspect, Note and Ask follow it.", "selection", {"kind": {"type": "string"}, "name": {"type": "string"}}),
        ("selection.back", "Previous selection", "Go back to what was selected before.", "selection", {}),
        ("selection.clear", "Deselect", "Clear the selection everywhere.", "selection", {}),
        ("timeline.set_year", "Go to year", "Move the shared timeline to a year: the map, Inspect and notes follow.", "story/time", {"year": {"type": "integer"}}),
        ("timeline.play", "Play the timeline", "Play forward at x1, x10 or x100.", "story/time", {"speed": {"type": "integer", "enum": [1, 10, 100]}}),
        ("timeline.record_year", "Record this year", "Keep who holds what at the shown year as a snapshot.", "story/time", {}),
        ("context.tab", "Show Inspect, Note or Ask", "Switch the Context panel's tab.", "window/context", {"tab": {"type": "string", "enum": ["inspect", "note", "ask"]}}),
    ]
    for aid, title, desc, path, args in proposed:
        out.append(dict(id=aid, title=title, desc=desc, path=path, kind="select" if path in ("selection",) else "view", effect="view", args=args, source="PC_SHELL_PROPOSAL.md (proposed)"))
    return out


def build(repo):
    """The fixture catalogue: a list of entries {id, title, desc, path, kind, effect, args, source[, accel]}."""
    cat = _params(repo) + _menus(repo) + _windows(repo) + _ctx_actions(repo) + _tools(repo) + _view_and_context(repo)
    ids = [e["id"] for e in cat]
    assert len(ids) == len(set(ids)), "duplicate ids in the fixture"
    return cat


# ---------------------------------------------------------------- the tree (list_actions)
def tree(cat):
    """path -> {"children": {name: count}, "actions": [entries]} for every prefix of every entry's path."""
    T = {}
    for e in cat:
        parts = e["path"].split("/")
        for d in range(len(parts) + 1):
            node = T.setdefault("/".join(parts[:d]), {"children": {}, "actions": []})
            if d < len(parts):
                node["children"][parts[d]] = node["children"].get(parts[d], 0) + 1
            else:
                node["actions"].append(e)
    return T


PAGE = 25


def list_actions(T, path="", kind=None, page=0):
    path = (path or "").strip("/")
    node = T.get(path)
    if node is None:
        return {"error": "no such category: " + path, "top": sorted(T[""]["children"])}
    acts = [e for e in node["actions"] if not kind or e["kind"] == kind]
    items = [{"category": (path + "/" if path else "") + c, "actions": n} for c, n in sorted(node["children"].items())]
    if kind:   # a type filter counts only actions of that kind below each child
        items = [it for it in items if any(e["kind"] == kind for e in _below(T, it["category"]))]
        for it in items:
            it["actions"] = sum(1 for e in _below(T, it["category"]) if e["kind"] == kind)
    rows = items + [{"id": e["id"], "title": e["title"], "kind": e["kind"], "available": True} for e in acts]
    page_rows = rows[page * PAGE:(page + 1) * PAGE]
    res = {"path": path or "(top)", "entries": page_rows}
    if len(rows) > (page + 1) * PAGE:
        res["next_page"] = page + 1
    return res


def _below(T, path):
    out = list(T.get(path, {"actions": []})["actions"])
    for c in T.get(path, {"children": {}})["children"]:
        out += _below(T, path + "/" + c)
    return out


def capability_map(cat):
    T = tree(cat)
    lines = []
    for top, n in sorted(T[""]["children"].items()):
        subs = T[top]["children"]
        sub = ", ".join("%s %d" % (s, len(_below(T, top + "/" + s))) for s in sorted(subs)) if subs else ""
        lines.append("%s (%d)%s" % (top, n, ": " + sub if sub else ""))
    return "; ".join(lines)


# ---------------------------------------------------------------- search (BM25)
STOP = set("a an the of to and or in on for with my me i it this that is be make set show turn change please".split())


def _toks(s):
    out = []
    for w in re.findall(r"[a-z0-9]+", s.lower()):
        if w in STOP:
            continue
        if len(w) > 4 and w.endswith("s"):
            w = w[:-1]
        out.append(w)
    return out


class Search:
    """BM25 (k1 1.2, b 0.75) over title (x2), category path and description: the ranker family the knowledge tool
    already uses (cartalith-knowledge/src/search.rs); a prototype, so its weights are a labelled judgement."""
    def __init__(self, cat):
        self.cat = cat
        self.docs = [_toks(e["title"]) * 2 + _toks(e["path"].replace("/", " ")) + _toks(e["desc"]) for e in cat]
        self.N = len(self.docs)
        self.avg = sum(len(d) for d in self.docs) / max(1, self.N)
        df = {}
        for d in self.docs:
            for t in set(d):
                df[t] = df.get(t, 0) + 1
        self.idf = {t: math.log(1 + (self.N - n + 0.5) / (n + 0.5)) for t, n in df.items()}

    def __call__(self, query, k=8, group=None):
        q = _toks(query)
        scored = []
        for i, d in enumerate(self.docs):
            e = self.cat[i]
            if group and not e["path"].startswith(group):
                continue
            s = 0.0
            for t in q:
                f = d.count(t)
                if f:
                    s += self.idf.get(t, 0) * f * 2.2 / (f + 1.2 * (0.25 + 0.75 * len(d) / self.avg))
            if s > 0:
                scored.append((s, i))
        scored.sort(key=lambda x: -x[0])
        return [self.cat[i] for _, i in scored[:k]]


def search_result(hits):
    return {"matches": [{"id": e["id"], "title": e["title"], "desc": e["desc"][:120], "kind": e["kind"], "available": True} for e in hits]}


def describe(cat_by_id, ids):
    out = []
    for i in ids[:3]:
        e = cat_by_id.get(i)
        if not e:
            out.append({"id": i, "error": "unknown id"}); continue
        out.append({"id": i, "title": e["title"], "desc": e["desc"], "kind": e["kind"], "effect": e["effect"],
                    "args": {"type": "object", "properties": e["args"], "required": sorted(e["args"])}, "path": e["path"]})
    return {"actions": out}


# ---------------------------------------------------------------- the four tool specs and the system prompt (drafts)
def tool_specs():
    return [
        {"name": "list_actions", "description": "Browse what Cartalith can do, by category and type. With no path: the top categories with counts. With a path from a previous list: its sub-categories, or its actions one line each. Optional kind filters by type: query, view, select, setting, edit, window. At most 25 entries; use page for more.",
         "parameters": {"type": "object", "properties": {"path": {"type": "string"}, "kind": {"type": "string", "enum": ["query", "view", "select", "setting", "edit", "window"]}, "page": {"type": "integer", "minimum": 0}}}},
        {"name": "search_actions", "description": "Search Cartalith's actions by words, when you know what the user named. Returns up to 8 matches with their ids.",
         "parameters": {"type": "object", "properties": {"query": {"type": "string", "minLength": 1, "maxLength": 200}, "group": {"type": "string"}}, "required": ["query"]}},
        {"name": "describe_action", "description": "Get the full description and arguments of up to 3 actions, by the ids a list or search returned in this request.",
         "parameters": {"type": "object", "properties": {"ids": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 3}}, "required": ["ids"]}},
        {"name": "propose_actions", "description": "Propose up to 5 actions as one card the user reviews and applies as one undo step. Only ids returned by list_actions, search_actions or describe_action in this request are accepted; arguments must match describe_action's schema. Nothing runs until the user presses Apply.",
         "parameters": {"type": "object", "properties": {"summary": {"type": "string"}, "actions": {"type": "array", "minItems": 1, "maxItems": 5, "items": {"type": "object", "properties": {"id": {"type": "string"}, "args": {"type": "object"}}, "required": ["id"]}}}, "required": ["summary", "actions"]}},
    ]


def system_prompt(cmap):
    return ("You help a user work in Cartalith, a world-map maker. You can do what its menus, tools and settings do, by proposing "
            "actions the user reviews and applies. Find actions in one of two ways: browse with list_actions (categories, then a "
            "list of commands; filter by kind), or search_actions when the user named the thing. Then describe_action the few you "
            "will use, and propose_actions once. Never invent an id; only ids returned in this request are accepted. File actions "
            "(save, open, export, new world) are not available: say so. If nothing fits, say so instead of proposing.\n\n"
            "What exists (category (actions): sub-categories): " + cmap)


if __name__ == "__main__":
    import sys
    repo = sys.argv[1] if len(sys.argv) > 1 else "."
    cat = build(repo)
    by = {}
    for e in cat:
        by[e["source"].split(" ")[0]] = by.get(e["source"].split(" ")[0], 0) + 1
    print(len(cat), "entries", by)
    print("with a description:", sum(1 for e in cat if e["desc"]), "of", len(cat))
    print(capability_map(cat))
