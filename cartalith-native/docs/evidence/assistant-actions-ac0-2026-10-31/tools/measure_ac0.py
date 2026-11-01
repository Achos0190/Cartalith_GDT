"""AC-0, the model-free half: token counts with real tokenizers, and how far search and browsing reach on the 30 tasks.

Run (any OS, Python 3.10+):  python -I measure_ac0.py --repo <repo root> --tok <dir of tokenizer.json files> --out <dir>
Tokenizers are optional: with `tokenizers` installed and the files present, it counts with Qwen3 (Qwen/Qwen3-14B),
gpt-oss (openai/gpt-oss-20b) and Ministral-3 (mistralai/Ministral-3-14B-Instruct-2512); chars/4 always.

What it counts is the text a request carries: the system prompt, the tool definitions as OpenAI-wire JSON, and any tool
results in the history. A chat template adds a model-specific overhead on top (role markers, tool framing), so these
are lower bounds for a real request; run_ac0.py records the servers' own prompt counts.
Never: sends anything anywhere. It reads the repository and the tokenizer files only.
"""
import argparse, json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)


def counters(tokdir):
    out = {"chars/4": lambda s: round(len(s) / 4)}
    try:
        from tokenizers import Tokenizer
    except ImportError:
        return out
    for name, f in [("qwen3", "qwen3.tokenizer.json"), ("gpt-oss", "gptoss.tokenizer.json"), ("ministral-3", "ministral3.tokenizer.json")]:
        p = os.path.join(tokdir or "", f)
        if os.path.exists(p):
            tk = Tokenizer.from_file(p)
            out[name] = (lambda s, tk=tk: len(tk.encode(s, add_special_tokens=False).ids))
    return out


def wire(tools):
    return json.dumps([{"type": "function", "function": t} for t in tools], separators=(",", ":"), ensure_ascii=False)


def j(x):
    return json.dumps(x, separators=(",", ":"), ensure_ascii=False)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True)
    ap.add_argument("--tok", default="")
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    os.environ["CARTALITH_REPO"] = os.path.abspath(a.repo)
    import actions as A, today as TD
    C = counters(a.tok)
    names = list(C)
    cat = A.build(a.repo)
    T = A.tree(cat)
    S = A.Search(cat)
    by_id = {e["id"]: e for e in cat}
    cmap = A.capability_map(cat)
    sysp = A.system_prompt(cmap)
    ktools, kpar = TD.knowledge_tools()
    rows = []

    def add(label, text):
        rows.append({"shape": label, "chars": len(text), **{n: C[n](text) for n in names}})

    for label, (system, tools, extra) in TD.shapes().items():
        add(label, system + wire(tools) + extra)
    add("new: fixed request (system prompt with the map, 4 catalogue tools)", sysp + wire(A.tool_specs()))
    add("new: fixed request + knowledge tools", sysp + "\n\n" + kpar + wire(A.tool_specs() + ktools))
    add("new: capability map alone", cmap)
    add("new: list_actions() (top)", j(A.list_actions(T, "")))
    add("new: list_actions('carto/layers')", j(A.list_actions(T, "carto/layers")))
    add("new: list_actions('world/generation')", j(A.list_actions(T, "world/generation")))
    add("new: list_actions('world/generation/climate'), page 0", j(A.list_actions(T, "world/generation/climate")))
    add("new: search_actions('raise the sea level'), 8 matches", j(A.search_result(S("raise the sea level", 8))))
    add("new: describe_action(3 ids)", j(A.describe(by_id, ["param:sea_level", "view.layer.territory", "timeline.set_year"])))

    # a whole request, turn by turn: t27 "Show the political map as it was in the year 300", browsing with the map
    fixed = sysp + wire(A.tool_specs())
    u = "Show the political map as it was in the year 300"
    c1 = j({"tool_calls": [{"name": "list_actions", "arguments": {"path": "carto/layers"}}, {"name": "list_actions", "arguments": {"path": "story/time"}}]})
    r1 = j(A.list_actions(T, "carto/layers")) + j(A.list_actions(T, "story/time"))
    c2 = j({"tool_calls": [{"name": "describe_action", "arguments": {"ids": ["view.layer.territory", "timeline.set_year"]}}]})
    r2 = j(A.describe(by_id, ["view.layer.territory", "timeline.set_year"]))
    c3 = j({"tool_calls": [{"name": "propose_actions", "arguments": {"summary": "Political territory in year 300", "actions": [{"id": "view.layer.territory", "args": {"visible": True}}, {"id": "timeline.set_year", "args": {"year": 300}}]}}]})
    turns = [fixed + u, fixed + u + c1 + r1, fixed + u + c1 + r1 + c2 + r2]
    for i, t in enumerate(turns):
        add("new: t27 by browsing, request of turn %d" % (i + 1), t)
    total = {n: sum(C[n](t) for t in turns) for n in names}
    add("new: t27 by browsing, the model's last reply (the proposal)", c3)

    # the 30 tasks: how far search and browsing reach, with no model in the loop
    tasks = json.load(open(os.path.join(HERE, "tasks.json"), encoding="utf8"))["tasks"]
    reach = []
    for t in tasks:
        hits8 = [e["id"] for e in S(t["prompt"], 8)]
        hits5 = hits8[:5]
        slots = t["slots"]
        r = {"task": t["id"], "kind": t["kind"], "prompt": t["prompt"]}
        if not slots:
            r.update(search8=None, search5=None, rank=None, browse_calls_no_map=None, browse_calls_with_map=None, entries_read=None)
            reach.append(r); continue
        ok8 = all(any(i in hits8 for i in s) for s in slots)
        ok5 = all(any(i in hits5 for i in s) for s in slots)
        ranks = [min([hits8.index(i) + 1 for i in s if i in hits8] or [99]) for s in slots]
        calls0 = calls1 = read = 0
        for s in slots:   # the cheapest path to any acceptable id of each slot
            best = min((by_id[i]["path"].count("/") + 1, i) for i in s if i in by_id)
            parts = by_id[best[1]]["path"].split("/")
            calls0 += len(parts) + 1                 # top, then one list per level, with no map
            start = min(2, len(parts))               # the map already shows the top two levels, with counts
            calls1 += len(parts) - start + 1
            for d in range(start, len(parts) + 1):   # the lists a model reads on the cheapest path, with the map
                read += len(A.list_actions(T, "/".join(parts[:d]))["entries"])
        r.update(search8=ok8, search5=ok5, rank=max(ranks), browse_calls_no_map=calls0, browse_calls_with_map=calls1, entries_read=read)
        reach.append(r)

    os.makedirs(a.out, exist_ok=True)
    json.dump({"tokenizers": names, "shapes": rows, "t27_total_processed": total, "reach": reach,
               "catalogue": {"entries": len(cat), "with_desc": sum(1 for e in cat if e["desc"]), "by_kind": {k: sum(1 for e in cat if e["kind"] == k) for k in sorted({e["kind"] for e in cat})},
                             "top": T[""]["children"], "map_chars": len(cmap)}},
              open(os.path.join(a.out, "measure.json"), "w", encoding="utf8"), indent=1, ensure_ascii=False)
    # markdown
    out = ["| Request shape | chars | " + " | ".join(names) + " |", "|---|---:|" + "---:|" * len(names)]
    for r in rows:
        out.append("| %s | %d | %s |" % (r["shape"], r["chars"], " | ".join(str(r[n]) for n in names)))
    out.append("")
    out.append("t27 by browsing, prompt tokens processed over its three requests (no prompt caching): " + ", ".join("%s %d" % (n, total[n]) for n in names))
    out.append("")
    scored = [r for r in reach if r["search8"] is not None]
    out.append("| Task | Kind | Search top 8 | Search top 5 | Rank | Browse calls (no map / with map) | List entries read |")
    out.append("|---|---|---|---|---:|---|---:|")
    for r in reach:
        if r["search8"] is None:
            out.append("| %s %s | %s | — | — | — | — | — |" % (r["task"], r["prompt"], r["kind"])); continue
        out.append("| %s %s | %s | %s | %s | %s | %d / %d | %d |" % (r["task"], r["prompt"], r["kind"], "yes" if r["search8"] else "no", "yes" if r["search5"] else "no", r["rank"] if r["rank"] < 99 else "—", r["browse_calls_no_map"], r["browse_calls_with_map"], r["entries_read"]))
    out.append("")
    nslots = {t["id"]: len(t["slots"]) for t in tasks}
    out.append("Search finds every needed action in its top 8 for %d of %d tasks, and in its top 5 for %d. Browsing reaches all %d, because every action has a path; with the capability map, %d of them need one list call per action." % (
        sum(1 for r in scored if r["search8"]), len(scored), sum(1 for r in scored if r["search5"]), len(scored), sum(1 for r in scored if r["browse_calls_with_map"] == nslots[r["task"]])))
    open(os.path.join(a.out, "measure.md"), "w", encoding="utf8").write("\n".join(out) + "\n")
    print("\n".join(out))


if __name__ == "__main__":
    main()
