"""Builds the tables and raw appendices of REPORT.md from results.jsonl (stdlib only).
python -I aggregate.py <results.jsonl> <out_tables.md>
"""
import json, sys, statistics as S, collections

res = [json.loads(l) for l in open(sys.argv[1], encoding="utf8")]
MODELS = []
for r in res:
    if r["model"] not in MODELS: MODELS.append(r["model"])
short = lambda m: m.replace("hf.co/bartowski/mistralai_Ministral-3-14B-Instruct-2512-GGUF:Q5_K_M", "Ministral-3-14B Q5_K_M") \
                   .replace("hf.co/XeyonAI/Helcyon-Claude-Mythic-14b-v2.0-GGUF:Q5_K_M", "Helcyon-Mythic-14B Q5_K_M")
REF = {"no_sea", "drier_lower", "cold_big_mountains"}
IMP = {"imp_river", "imp_prettier", "imp_pacific"}
OPEN = ["no_sea", "more_volcanoes", "islands"]
out = []
w = out.append


def med(xs):
    xs = [x for x in xs if x is not None]
    return "n/a" if not xs else "%.1f" % S.median(xs) + (" (%.1f..%.1f)" % (min(xs), max(xs)))


CONNECT = 2.045  # measured: urllib to `localhost` spent 2.04 s per request on this machine (IPv6 fallback), 0.001 s to 127.0.0.1


def turn_stats(r):
    ts = r.get("turns", [])
    loc = not r["model"].startswith("claude-")
    ft = ts[0].get("first_token_s") if ts else None
    if ft is not None and loc: ft -= CONNECT
    tot = sum((t.get("total_s") or 0) - (CONNECT if loc else 0) for t in ts)
    ec = sum((t.get("stats") or {}).get("eval_count") or 0 for t in ts)
    ed = sum((t.get("stats") or {}).get("eval_duration") or 0 for t in ts)
    tps = ec / (ed / 1e9) if ed else None
    return ft, tot, tps


w("## Tool-use comparison (24 runs per model: 8 prompts x 3 repetitions)\n")
w("Definitions. **Proposal** = the run ended with a `set_generation_params` call after a valid `get_param_schema` call. "
  "**Clean** = a proposal with zero rejected changes under the strict validator (V1-V13). "
  "**Rejected changes** counts V4-V10 rejections across all proposals (unknown key V5, not-offered V6, duplicate V7, wrong type or fractional V8, out of range V10). "
  "**Keys** = median number of valid changes per proposal. **Direction** = on the three reference prompts "
  "(no sea, drier+lower, colder+big mountains), of the changes to keys with a scored direction, the share with the expected sign "
  "(scored keys are listed under Limits; expectations are my labelled judgement from `GENERATION_PARAMETERS.md`, not shown to any model). "
  "**Declined** = on the three impossible prompts, the run produced no valid change (a refusal, a question, or an explanation). "
  "Latency columns are medians (local models: client timings minus a constant 2.045 s connection delay measured on this machine, see Limits) with min..max over all tool runs (seconds; first token of the first turn; total over all turns); "
  "tok/s is generation speed from Ollama's own counters (local models only).\n")
hdr = "| Model | Proposal | Clean | Rejected | Keys | Direction (ref.) | Declined (imp.) | First token s | Total s | tok/s | Thinking tok/run (est.) |\n|---|---|---|---|---|---|---|---|---|---|---|"
w(hdr)
for m in MODELS:
    rs = [r for r in res if r["model"] == m and r["suite"] == "tools"]
    if not rs: continue
    prop = [r for r in rs if r["outcome"] == "proposal"]
    # a proposal whose valid list is empty is not a usable proposal
    usable = [r for r in prop if r["verdict"]["valid"] and not r["verdict"]["structural"]]
    clean = [r for r in prop if not r["verdict"]["structural"] and not r["verdict"]["rejected"] and r["verdict"]["valid"]]
    rej = collections.Counter(x[1] for r in prop for x in r["verdict"]["rejected"])
    nrej = sum(rej.values())
    keys = [len(r["verdict"]["valid"]) for r in usable]
    right = wrong = 0
    for r in rs:
        if r["pid"] in REF and r.get("direction"):
            right += r["direction"]["right"]; wrong += r["direction"]["wrong"]
    imp = [r for r in rs if r["pid"] in IMP]
    declined = sum(1 for r in imp if not (r["outcome"] == "proposal" and r["verdict"]["valid"]))
    fts, tots, tpss = [], [], []
    for r in rs:
        a, b, c = turn_stats(r); fts.append(a); tots.append(b); tpss.append(c)
    th = [sum(len(t.get("thinking") or "") for t in r.get("turns", [])) / 4 for r in rs]
    thc = "%d" % S.median(th) if th and max(th) > 0 else "0"
    d = "n/a" if right + wrong == 0 else "%d/%d (%d%%)" % (right, right + wrong, round(100 * right / (right + wrong)))
    w("| %s | %d/%d | %d/%d | %d (%s) | %s | %s | %d/%d | %s | %s | %s | %s |" % (
        short(m), len(prop), len(rs), len(clean), len(rs), nrej, ", ".join("%s x%d" % kv for kv in sorted(rej.items())) or "-",
        "%g" % S.median(keys) if keys else "-", d, declined, len(imp), med(fts), med(tots), med(tpss), thc))
w("")

w("### Outcome by prompt (counts over 3 repetitions: P = proposal with valid changes, T = answered in text without a tool call after the schema step, N = no tool call at all, E = error, M = ran out of turns)\n")
pids = [p for p, _ in __import__("run_eval").TOOLS] if False else ["no_sea", "more_volcanoes", "drier_lower", "cold_big_mountains", "islands", "imp_river", "imp_prettier", "imp_pacific"]
w("| Prompt | " + " | ".join(short(m) for m in MODELS) + " |\n|---|" + "---|" * len(MODELS))
for p in pids:
    cells = []
    for m in MODELS:
        c = collections.Counter()
        for r in res:
            if r["model"] == m and r["suite"] == "tools" and r["pid"] == p:
                o = r["outcome"]
                if o == "proposal": c["P" if r["verdict"]["valid"] and not r["verdict"]["structural"] else "P0"] += 1
                else: c[{"no_tool_after_schema": "T", "no_tool_call": "N", "error": "E", "max_turns": "M"}.get(o, o)] += 1
        cells.append(" ".join("%s%d" % kv for kv in sorted(c.items())) or "-")
    w("| `%s` | " % p + " | ".join(cells) + " |")
w("\n(P0 = a set_generation_params call whose changes were all rejected or empty.)\n")

w("## Prose (3 prompts x 3 repetitions, temperature 0.7, no tools)\n")
w("| Model | Runs | First token s | Total s | tok/s | Median words | Errors |\n|---|---|---|---|---|---|---|")
for m in MODELS:
    rs = [r for r in res if r["model"] == m and r["suite"] == "prose"]
    if not rs: continue
    fts, tots, tpss, ws, er = [], [], [], [], 0
    for r in rs:
        t = r["turns"][0]
        if t.get("error"): er += 1; continue
        a, b, c = turn_stats(r); fts.append(a); tots.append(b); tpss.append(c); ws.append(len((t.get("content") or "").split()))
    w("| %s | %d | %s | %s | %s | %s | %d |" % (short(m), len(rs), med(fts), med(tots), med(tpss), "%d" % S.median(ws) if ws else "-", er))
w("")

w("## Vision (one run per image and question; images are synthetic Cartalith renders from `cartalith-native/docs/evidence/`)\n")
w("| Model | Runs | First token s | Total s | tok/s | Errors |\n|---|---|---|---|---|---|")
for m in MODELS:
    rs = [r for r in res if r["model"] == m and r["suite"] == "vision"]
    if not rs: continue
    fts, tots, tpss, er = [], [], [], 0
    for r in rs:
        t = r["turns"][0]
        if t.get("error"): er += 1; continue
        a = t.get("first_token_s"); fts.append(a - CONNECT if (a is not None and not r["model"].startswith("claude-")) else a); tots.append((t.get("total_s") or 0) - (0 if r["model"].startswith("claude-") else CONNECT))
        st = t.get("stats") or {}
        if st.get("eval_duration"): tpss.append(st["eval_count"] / (st["eval_duration"] / 1e9))
    w("| %s | %d | %s | %s | %s | %d |" % (short(m), len(rs), med(fts), med(tots), med(tpss), er))
w("")

# ------------------------------------------------------------------ appendices
def fmt_val(v):
    return ("%g" % v) if isinstance(v, float) else str(v)


def render_run(r):
    L = []
    o = r["outcome"]
    gs = ",".join(r.get("shown_groups") or []) or "-"
    if o == "proposal":
        v = r["verdict"]; a = r["args"]
        summ = a.get("summary") if isinstance(a, dict) else None
        L.append("- rep %d, groups `%s`%s: %s" % (r["rep"], gs, "" if not r.get("schema_errors") else " (schema errors %d)" % r["schema_errors"],
                                                 ("structural failure: " + v["structural"]) if v["structural"] else "%d valid, %d rejected, %d no-op" % (len(v["valid"]), len(v["rejected"]), len(v["noop"]))))
        if isinstance(a, dict) and isinstance(a.get("changes"), list):
            for c in a["changes"]:
                if isinstance(c, dict):
                    L.append("  - `%s` -> %s%s" % (c.get("key"), json.dumps(c.get("value")), (" (%s)" % c["reason"]) if c.get("reason") else ""))
        if v["rejected"]: L.append("  - rejected: " + "; ".join("%s %s %s" % (k, ru, wh) for k, ru, wh in v["rejected"]))
        if summ: L.append("  - summary: %s" % summ)
    else:
        L.append("- rep %d, groups `%s`, outcome `%s`%s" % (r["rep"], gs, o, (": " + r["error"]) if r.get("error") else ""))
        t = (r.get("text") or "").strip()
        if t: L.append("  > " + t.replace("\n", "\n  > "))
    return L


w("## Appendix A: raw tool outputs, every prompt, every repetition\n")
w("Values are the model's proposed numbers; the current values are the registry defaults listed in the catalogue (e.g. `sea_level` 0.42, `peak_m` 4000, `volc.count` 20).\n")
for p in ["no_sea", "more_volcanoes", "islands", "drier_lower", "cold_big_mountains", "imp_river", "imp_prettier", "imp_pacific"]:
    pr = next((r["prompt"] for r in res if r["pid"] == p and r["suite"] == "tools"), "")
    w("### `%s`: \"%s\"\n" % (p, pr))
    for m in MODELS:
        rs = sorted([r for r in res if r["model"] == m and r["suite"] == "tools" and r["pid"] == p], key=lambda r: r["rep"])
        if not rs: continue
        w("**%s**" % short(m))
        for r in rs:
            for ln in render_run(r): w(ln)
            if r.get("direction"): w("  - direction: " + ", ".join("%s %s" % tuple(h) for h in r["direction"]["hits"]))
        w("")

w("## Appendix B: key frequency for the open questions (valid changes, 3 repetitions per model)\n")
for p in OPEN:
    w("**`%s`**\n" % p)
    for m in MODELS:
        c = collections.Counter(k for r in res if r["model"] == m and r["suite"] == "tools" and r["pid"] == p and r["outcome"] == "proposal"
                                for k, _ in r["verdict"]["valid"])
        w("- %s: %s" % (short(m), ", ".join("`%s` x%d" % kv for kv in c.most_common()) or "none"))
    w("")

w("## Appendix C: vision answers, verbatim\n")
for img in sorted({r["image"] for r in res if r["suite"] == "vision"}):
    w("### `%s`\n" % img)
    for q in [r["pid"].split("_", 2)[-1] for r in res if r["suite"] == "vision" and r["image"] == img and r["model"] == next(x["model"] for x in res if x["suite"] == "vision")]:
        rr = [r for r in res if r["suite"] == "vision" and r["image"] == img and r["pid"].endswith("_" + q)]
        if not rr: continue
        w("**Q (%s):** %s\n" % (q, rr[0]["prompt"]))
        for r in rr:
            t = r["turns"][0]
            w("- *%s*: %s" % (short(r["model"]), (t.get("content") or t.get("error") or "").strip().replace("\n", " ")))
        w("")

w("## Appendix D: prose outputs, first repetition of each prompt\n")
for p in ["harbour", "watchtower", "volcanic_island"]:
    w("### `%s`\n" % p)
    for m in MODELS:
        r = next((r for r in res if r["model"] == m and r["suite"] == "prose" and r["pid"] == p and r["rep"] == 0), None)
        if r: w("**%s**\n\n> %s\n" % (short(m), (r["turns"][0].get("content") or r["turns"][0].get("error") or "").strip().replace("\n", "\n> ")))

cl = [r for r in res if r["model"].startswith("claude-")]
w("## Appendix E: Claude usage\n")
for m in sorted({r["model"] for r in cl}):
    rs = [r for r in cl if r["model"] == m]
    w("- `%s`: %d runs, %d API calls, %d input tokens, %d output tokens" % (m, len(rs), sum(r["api_calls"] for r in rs), sum(r["in_tokens"] for r in rs), sum(r["out_tokens"] for r in rs)))
w("- total: %d API calls, %d input tokens, %d output tokens" % (sum(r["api_calls"] for r in cl), sum(r["in_tokens"] for r in cl), sum(r["out_tokens"] for r in cl)))
open(sys.argv[2], "w", encoding="utf8").write("\n".join(out) + "\n")
