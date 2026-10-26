"""Builds the side-by-side tables of the grounded-prose live comparison (LLM-11) from the probe's JSONL.

Usage: python -I aggregate.py <raw dir>   (reads A.jsonl and B.jsonl; prints Markdown)

Inputs are the `LIVE` records `_llmplace_live_probe.gd` wrote: one `prose` record per request. Nothing here
is estimated: every figure is a count or a median of the recorded values, and an absent value prints as
"n/a" (never 0). Prints ASCII. The accuracy columns come from `cartalith_llm::grounding` as the panel ran
it; the contradiction column is the probe's keyword heuristic and is only a pointer for the human reading.
"""
import json
import os
import statistics
import sys


def load(path):
    rows = []
    if not os.path.exists(path):
        return rows
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if line:
            rows.append(json.loads(line))
    return rows


def med(vals):
    vals = [v for v in vals if v is not None]
    if not vals:
        return None
    return statistics.median(vals)


def fmt(v, nd=1):
    return "n/a" if v is None else (f"{v:.{nd}f}" if isinstance(v, float) else str(v))


def n(rec, key, sub=None):
    g = rec.get("grounding") or {}
    v = g.get(key)
    return len(v) if isinstance(v, list) else None


def row(rec):
    u = rec.get("usage") or {}
    use = rec.get("use") or {}
    ft = rec.get("first_token_ms")
    tm = rec.get("total_ms")
    flags = use.get("contradiction_candidates") or []
    return [
        rec["arm"], rec["condition"], str(rec["seed"]), rec["place"] + " (" + rec["why_chosen"].split(" ")[0] + ")", str(rec["rep"]),
        fmt(ft / 1000.0 if isinstance(ft, int) and ft >= 0 else None, 2), fmt(tm / 1000.0 if isinstance(tm, int) else None, 1),
        fmt(rec.get("completion_tokens")), fmt(rec.get("tok_per_s")), str(rec.get("reply_words", "n/a")),
        "%s/%s/%s" % (fmt(n(rec, "numbers_matched")), fmt(n(rec, "numbers_approx")), fmt(n(rec, "numbers_unverified"))),
        "%s/%s" % (fmt(n(rec, "names_matched")), fmt(n(rec, "names_unverified"))),
        "".join(k if use.get(f) else "-" for k, f in (("N", "uses_neighbours"), ("T", "uses_trade"), ("R", "uses_religion"), ("L", "uses_relations"))),
        "; ".join(flags) if flags else "-",
    ]


def summary(rows):
    out = {}
    out["runs"] = len(rows)
    out["first_token_s"] = med([r["first_token_ms"] / 1000.0 for r in rows if isinstance(r.get("first_token_ms"), int) and r["first_token_ms"] >= 0])
    out["total_s"] = med([r["total_ms"] / 1000.0 for r in rows if isinstance(r.get("total_ms"), int)])
    out["tokens"] = med([r.get("completion_tokens") for r in rows])
    out["prompt_tokens"] = med([r.get("prompt_tokens") for r in rows])
    out["tok_s"] = med([r.get("tok_per_s") for r in rows])
    out["words"] = med([r.get("reply_words") for r in rows])
    unv_n = sum(n(r, "numbers_unverified") or 0 for r in rows)
    unv_w = sum(n(r, "names_unverified") or 0 for r in rows)
    ok_n = sum((n(r, "numbers_matched") or 0) + (n(r, "numbers_approx") or 0) for r in rows)
    ok_w = sum(n(r, "names_matched") or 0 for r in rows)
    words = sum(r.get("reply_words", 0) for r in rows) or 1
    out["numbers_matched"] = ok_n
    out["numbers_unverified"] = unv_n
    out["names_matched"] = ok_w
    out["names_unverified"] = unv_w
    out["unverified_per_100_words"] = 100.0 * (unv_n + unv_w) / words
    for label, f in (("uses_neighbours", "uses_neighbours"), ("uses_trade", "uses_trade"), ("uses_religion", "uses_religion"), ("uses_relations", "uses_relations")):
        out[label] = sum(1 for r in rows if (r.get("use") or {}).get(f))
    out["flagged_contradiction_candidates"] = sum(1 for r in rows if (r.get("use") or {}).get("contradiction_candidates"))
    out["cut_by_length"] = sum(1 for r in rows if r.get("finish_reason") == "length")
    out["failed"] = sum(1 for r in rows if r.get("failed") or not r.get("finished", True))
    return out


def main():
    raw = sys.argv[1]
    arms = {}
    for a in ("A", "B"):
        arms[a] = [r for r in load(os.path.join(raw, a + ".jsonl")) if r.get("kind") == "prose"]
    head = ["Arm", "Condition", "Seed", "Place", "Rep", "First token s", "Total s", "Out tok", "tok/s", "Words", "Numbers ok/approx/unverified",
            "Names ok/unverified", "Uses N/T/R/L", "Contradiction candidates"]
    print("## Per request\n")
    print("| " + " | ".join(head) + " |")
    print("|" + "---|" * len(head))
    for a in ("A", "B"):
        for r in arms[a]:
            print("| " + " | ".join(row(r)) + " |")
    print("\n## Per arm\n")
    sa, sb = summary(arms["A"]), summary(arms["B"])
    labels = [("runs", "Requests"), ("first_token_s", "Median first token, s"), ("total_s", "Median total, s"), ("tokens", "Median output tokens"),
              ("prompt_tokens", "Median prompt tokens"), ("tok_s", "Median generation tok/s"), ("words", "Median reply words"),
              ("numbers_matched", "Numbers matched (incl. rounded), all runs"), ("numbers_unverified", "Numbers unverified, all runs"),
              ("names_matched", "Names matched, all runs"), ("names_unverified", "Names unverified, all runs"),
              ("unverified_per_100_words", "Unverified claims per 100 words"), ("uses_neighbours", "Runs naming a neighbour"),
              ("uses_trade", "Runs using trade"), ("uses_religion", "Runs naming the faith"), ("uses_relations", "Runs using relations"),
              ("flagged_contradiction_candidates", "Runs with a contradiction candidate"), ("cut_by_length", "Runs cut at the length limit"), ("failed", "Runs that failed")]
    print("| Measure | A: qwen3-14b, thinking off | B: Qwen3.8-27B UD-Q3_K_XL, thinking off |")
    print("|---|---|---|")
    for k, lab in labels:
        print("| %s | %s | %s |" % (lab, fmt(sa.get(k), 2 if isinstance(sa.get(k), float) else 1), fmt(sb.get(k), 2 if isinstance(sb.get(k), float) else 1)))


main()
