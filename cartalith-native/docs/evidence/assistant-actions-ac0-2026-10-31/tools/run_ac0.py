"""AC-0, the model half: do models pick the right action through the catalogue tools? (ASSISTANT_ACTIONS_PROPOSAL.md)

Run on the owner's PC, where the models are (stdlib only, Python 3.10+):
  python -I run_ac0.py --repo <repo root> --out <dir> --models qwen3:14b#nothink gpt-oss:20b <ministral> claude-sonnet-5-5
  python -I run_ac0.py --repo <repo root> --out <dir> --aggregate          (tables from <dir>/results.jsonl)
  python -I run_ac0.py --repo <repo root> --out <dir> --models mock-search mock-browse   (no model: plumbing and a baseline)
Local models go to Ollama at http://localhost:11434 through the 2026-10-26 harness's own `ollama_chat` (num_ctx 16384,
temperature 0.3); Claude models through its `claude_chat`, with the key read in-process from Windows Credential Manager
(Cartalith/LLM/anthropic), never printed or written, and scrubbed from every output. Both helpers are imported unchanged
from ../../llm-model-eval-2026-10-26/tools/run_eval.py, so the reviewed key handling is the only key handling.

Four conditions per task (each 1 run; --reps for more):
  browse     list_actions + describe_action + propose_actions
  search     search_actions + describe_action + propose_actions
  both       all four tools
  both+pre   all four, plus the top 5 search hits for the request in the system prompt (pre-retrieval, AQ4)
Rules enforced here exactly as AC-2 must: only ids returned in this request by list/search/describe may be described or
proposed; at most 3 ids per describe, at most 5 actions per proposal; MAX_TURNS 5 (params_flow.rs). A proposal ends the
request. Scoring: every slot of the task satisfied and no proposed id outside the task's slots; 'none' tasks are right
only when nothing is proposed.
The two mock "models" are not models: mock-search proposes the top search hit for each request (a greedy baseline that
shows what search alone gets right); mock-browse walks the expected path (a plumbing check only; its score means nothing).
Never: sends a request anywhere but localhost and api.anthropic.com; writes a key; changes anything in the repository.
"""
import argparse, json, os, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "../../llm-model-eval-2026-10-26/tools")))
import actions as A

MAX_TURNS = 5          # params_flow.rs::MAX_TURNS
CONDITIONS = ["browse", "search", "both", "both+pre"]


def tools_for(cond):
    t = {s["name"]: s for s in A.tool_specs()}
    names = {"browse": ["list_actions", "describe_action", "propose_actions"], "search": ["search_actions", "describe_action", "propose_actions"]}.get(cond, list(t))
    return [t[n] for n in names]


def system_for(cond, cmap, pre):
    s = A.system_prompt(cmap)
    if cond == "browse":
        s = s.replace("Find actions in one of two ways: browse with list_actions (categories, then a list of commands; filter by kind), or search_actions when the user named the thing.", "Find actions by browsing with list_actions (categories, then a list of commands; filter by kind).")
    if cond == "search":
        s = s.replace("Find actions in one of two ways: browse with list_actions (categories, then a list of commands; filter by kind), or search_actions when the user named the thing.", "Find actions with search_actions.")
    if pre:
        s += "\n\nActions that may fit this request (ids you may describe or propose): " + json.dumps(pre, ensure_ascii=False)
    return s


def score(task, proposed):
    if not task["slots"]:
        return not proposed
    if not proposed:
        return False
    allowed = {i for s in task["slots"] for i in s}
    return all(any(i in proposed for i in s) for s in task["slots"]) and all(p in allowed for p in proposed)


def args_of(c):
    a = c.get("arguments")
    if isinstance(a, str):
        try:
            a = json.loads(a)
        except Exception:
            a = {}
    return a if isinstance(a, dict) else {}


def run_case(model, task, cond, ctx, key, cstate):
    cat, T, S, by_id, cmap = ctx
    pre = [{"id": e["id"], "title": e["title"]} for e in S(task["prompt"], 5)] if cond == "both+pre" else []
    returned = {p["id"] for p in pre}
    system = system_for(cond, cmap, pre)
    tools = tools_for(cond)
    is_claude = model.startswith("claude-")
    msgs = [{"role": "user", "content": task["prompt"]}] if is_claude else [{"role": "system", "content": system}, {"role": "user", "content": task["prompt"]}]
    turns, errors = [], []
    for turn in range(MAX_TURNS):
        if model.startswith("mock-"):
            r = mock_turn(model, task, cond, ctx, msgs, returned)
        elif is_claude:
            import run_eval as R
            r = R.claude_chat(model, system, msgs, tools, 0.3, key, cstate)
        else:
            import run_eval as R
            r = R.ollama_chat(model, msgs, tools, 0.3, 2048)
        turns.append({k: v for k, v in r.items() if k not in ("raw_content",)})
        if r.get("error"):
            return dict(outcome="error", error=r["error"], turns=turns, correct=False, proposed=[])
        calls = r.get("tool_calls") or []
        if not calls:
            return dict(outcome="no_tool_call", text=(r.get("content") or "")[:500], turns=turns, proposed=[], correct=score(task, []), errors=errors)
        if is_claude:
            msgs.append({"role": "assistant", "content": r["raw_content"]})
        else:
            msgs.append({"role": "assistant", "content": r.get("content", ""), "tool_calls": [{"function": {"name": c["name"], "arguments": c["arguments"]}} for c in calls]})
        results = []
        for c in calls:
            a = args_of(c); name = c["name"]
            if name == "propose_actions":
                acts = a.get("actions") if isinstance(a.get("actions"), list) else []
                ids = [x.get("id") for x in acts if isinstance(x, dict)]
                bad = [i for i in ids if i not in returned]
                if bad or not ids or len(ids) > 5:
                    errors.append({"turn": turn, "rejected": bad or ids, "why": "not returned in this request" if bad else "1 to 5 actions"})
                    results.append((c, json.dumps({"error": "rejected: only ids returned by list_actions, search_actions or describe_action in this request; 1 to 5 actions", "rejected": bad}, ensure_ascii=False)))
                    continue
                return dict(outcome="proposal", proposed=ids, args=acts, turns=turns, correct=score(task, ids), errors=errors, n_turns=turn + 1)
            if name == "list_actions" and cond != "search":
                res = A.list_actions(T, a.get("path", ""), a.get("kind"), int(a.get("page") or 0))
                returned.update(x["id"] for x in res.get("entries", []) if "id" in x)
            elif name == "search_actions" and cond != "browse":
                res = A.search_result(S(str(a.get("query", "")), 8, a.get("group")))
                returned.update(x["id"] for x in res["matches"])
            elif name == "describe_action":
                ids = [i for i in (a.get("ids") or []) if isinstance(i, str)][:3]
                bad = [i for i in ids if i not in returned]
                res = A.describe(by_id, [i for i in ids if i in returned])
                if bad:
                    res["rejected"] = bad; errors.append({"turn": turn, "rejected": bad, "why": "describe of an id not returned"})
            else:
                res = {"error": "tool %s is not offered" % name}; errors.append({"turn": turn, "why": "tool not offered: " + name})
            results.append((c, json.dumps(res, ensure_ascii=False)))
        if is_claude:
            msgs.append({"role": "user", "content": [{"type": "tool_result", "tool_use_id": c["id"], "content": res_} for c, res_ in results]})
        else:
            for c, res_ in results:
                msgs.append({"role": "tool", "tool_name": c["name"], "content": res_})
    return dict(outcome="max_turns", turns=turns, proposed=[], correct=False, errors=errors)


def mock_turn(model, task, cond, ctx, msgs, returned):
    """Not a model. mock-search: search the request, describe the top hit, propose it. mock-browse: list the expected path."""
    cat, T, S, by_id, cmap = ctx
    n_tool = sum(1 for m in msgs if m.get("role") == "tool")
    if model == "mock-search":
        if n_tool == 0:
            if cond == "browse":
                return dict(tool_calls=[{"name": "list_actions", "arguments": {}}], content="")
            return dict(tool_calls=[{"name": "search_actions", "arguments": {"query": task["prompt"]}}], content="")
        hits = [e["id"] for e in S(task["prompt"], 8)]
        if not hits or cond == "browse":
            return dict(tool_calls=[], content="Nothing fits.")
        return dict(tool_calls=[{"name": "propose_actions", "arguments": {"summary": task["prompt"], "actions": [{"id": hits[0], "args": {}}]}}], content="")
    # mock-browse: the oracle path (plumbing only)
    want = [s[0] for s in task["slots"]]
    if not want:
        return dict(tool_calls=[], content="That is not something I can do here.")
    if n_tool == 0:
        if cond == "search":
            return dict(tool_calls=[{"name": "search_actions", "arguments": {"query": by_id[w]["title"]}} for w in want], content="")
        return dict(tool_calls=[{"name": "list_actions", "arguments": {"path": by_id[w]["path"]}} for w in want], content="")
    return dict(tool_calls=[{"name": "propose_actions", "arguments": {"summary": task["prompt"], "actions": [{"id": w, "args": {}} for w in want]}}], content="")


def aggregate(out):
    rows = [json.loads(l) for l in open(os.path.join(out, "results.jsonl"), encoding="utf8")]
    models = list(dict.fromkeys(r["model"] for r in rows))
    lines = ["| Model | " + " | ".join(CONDITIONS) + " |", "|---|" + "---|" * len(CONDITIONS)]
    for m in models:
        cells = []
        for c in CONDITIONS:
            rs = [r for r in rows if r["model"] == m and r["cond"] == c]
            if not rs:
                cells.append("—"); continue
            ok = sum(1 for r in rs if r["correct"]); pt = [t.get("stats", {}).get("prompt_eval_count") or t.get("stats", {}).get("input_tokens") or 0 for r in rs for t in r.get("turns", [])]
            peak = max(pt) if pt else 0
            cells.append("%d/%d · peak %s tok" % (ok, len(rs), peak or "?"))
        lines.append("| %s | %s |" % (m, " | ".join(cells)))
    txt = "\n".join(lines)
    open(os.path.join(out, "tables.md"), "w", encoding="utf8").write(txt + "\n")
    print(txt)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True); ap.add_argument("--out", required=True)
    ap.add_argument("--models", nargs="*", default=[]); ap.add_argument("--conds", nargs="*", default=CONDITIONS)
    ap.add_argument("--reps", type=int, default=1); ap.add_argument("--aggregate", action="store_true")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    if a.aggregate:
        return aggregate(a.out)
    cat = A.build(a.repo); ctx = (cat, A.tree(cat), A.Search(cat), {e["id"]: e for e in cat}, A.capability_map(cat))
    tasks = json.load(open(os.path.join(HERE, "tasks.json"), encoding="utf8"))["tasks"]
    path = os.path.join(a.out, "results.jsonl")
    done = set()
    if os.path.exists(path):
        for l in open(path, encoding="utf8"):
            r = json.loads(l); done.add((r["model"], r["cond"], r["task"], r["rep"]))
    key, cstate = None, {"calls": 0, "in": 0, "out": 0}
    if any(m.startswith("claude-") for m in a.models):
        import run_eval as R
        key = R.read_cred(); R.SECRET[0] = key
    with open(path, "a", encoding="utf8") as f:
        for m in a.models:
            for cond in a.conds:
                for t in tasks:
                    for rep in range(a.reps):
                        if (m, cond, t["id"], rep) in done:
                            continue
                        t0 = time.time()
                        r = run_case(m, t, cond, ctx, key, cstate)
                        r.update(model=m, cond=cond, task=t["id"], rep=rep, wall_s=round(time.time() - t0, 2))
                        line = json.dumps(r, ensure_ascii=False, default=str)
                        if key:
                            line = line.replace(key, "[REDACTED]")
                        f.write(line + "\n"); f.flush()
                        print(m, cond, t["id"], r["outcome"], "OK" if r["correct"] else "--", r.get("proposed"))
    aggregate(a.out)


if __name__ == "__main__":
    main()
